//! Five-layer spherical BTCS (Backward Time, Centred Space) finite-difference solver.
//!
//! Implements Document 2 (`docs/mathematics/btcs/02_final_formulation.md`) following the
//! pseudocode of Document 3: input validation (PS-01), matrix assembly (PS-05, equations
//! (F9) to (F13), (F17), (F19)), one Backward Euler step (PS-11, (F20) and the tridiagonal
//! solve of `numerical::tridiagonal`), and the time loop with inventory, release and balance
//! accounting (PS-16, (F25) to (F30)).
//!
//! Scope: constant diffusivities, no source, no decay, constant temperature, kernel-only
//! start, absorbing outer surface. It is a numerical solution of the stated model; it is not
//! experimental validation. The finite-volume module is untouched and unused here.
//!
//! The module uses only `core`/`alloc`-level features of `std` (no threads, files, clocks or
//! randomness), so it builds for `wasm32-unknown-unknown` unchanged.

use super::{
    boundary_conditions::{centre_row, release_rate},
    coefficients::Coefficients,
    diagnostics::{
        check_matrix_structure, BalanceBound, MatrixReport, StepDiagnostics, TimeSeries,
    },
    grid::{Grid, ValidatedInput},
    inventory::{balance_residual, initial_vector, inventory, release_fraction, Accumulator},
    tridiagonal::ThomasFactors,
};
use crate::config::Config;
use anyhow::{bail, Result};

/// Caller-supplied maxima that guard a run (not constants of the mathematics).
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Largest allowed `N`.
    pub max_intervals_total: usize,
    /// Largest allowed number of time steps.
    pub max_steps: usize,
}

/// PS-01 input validation. Rejects every input for which the standing hypotheses (H) of
/// Document 2 Section 8 do not hold: five regions with `0 < R_1 < ... < R_5`, finite positive
/// `D`, at least one interval per material, positive finite `C_init`, `dt` and end time.
/// The existing configuration field `cells` is read as the number of grid intervals `n`.
/// The step count is `ceil(end / dt)` and the effective step is `end / steps` (review point
/// RP-2 of Document 3).
///
/// # Errors
///
/// Any violated condition; the message names the region and the quantity.
pub fn validate_input(cfg: &Config, limits: &Limits) -> Result<ValidatedInput> {
    if cfg.regions.len() != 5 {
        bail!("exactly five regions are required");
    }
    if cfg.species.trim().is_empty() {
        bail!("species must not be empty");
    }
    if !(cfg.temperature_k.is_finite() && cfg.temperature_k > 0.0) {
        bail!("temperature_k must be a positive finite number (kelvin)");
    }
    let mut radii = [0.0; 5];
    let mut d = [0.0; 5];
    let mut n_intervals = [0usize; 5];
    let mut previous = 0.0;
    for (k, region) in cfg.regions.iter().enumerate() {
        if !(region.outer_radius_m.is_finite() && region.outer_radius_m > previous) {
            bail!(
                "region {}: outer_radius_m must exceed the previous radius",
                region.name
            );
        }
        if !(region.diffusivity_m2_s.is_finite() && region.diffusivity_m2_s > 0.0) {
            bail!(
                "region {}: diffusivity_m2_s must be finite and positive",
                region.name
            );
        }
        if region.cells < 1 {
            bail!("region {}: needs at least one interval", region.name);
        }
        radii[k] = region.outer_radius_m;
        d[k] = region.diffusivity_m2_s;
        n_intervals[k] = region.cells;
        previous = region.outer_radius_m;
    }
    let n: usize = n_intervals.iter().sum();
    if n > limits.max_intervals_total {
        bail!(
            "total interval count {n} exceeds the limit {}",
            limits.max_intervals_total
        );
    }
    let c_init = cfg.initial.kernel_concentration_mol_m3;
    if !(c_init.is_finite() && c_init > 0.0) {
        bail!("kernel concentration must be finite and positive (review point RP-1)");
    }
    let t = &cfg.time;
    if !(t.dt_s.is_finite() && t.dt_s > 0.0) {
        bail!("dt_s must be finite and positive");
    }
    if !(t.end_time_s.is_finite() && t.end_time_s > 0.0) {
        bail!("end_time_s must be finite and positive");
    }
    if t.output_every_steps < 1 {
        bail!("output_every_steps must be at least 1");
    }
    let steps_f = (t.end_time_s / t.dt_s).ceil();
    if !(steps_f.is_finite() && steps_f >= 1.0 && steps_f <= limits.max_steps as f64) {
        bail!("step count is not within 1 ..= {}", limits.max_steps);
    }
    let steps = steps_f as usize;
    let dt = t.end_time_s / steps as f64;
    Ok(ValidatedInput {
        radii,
        d,
        n_intervals,
        c_init,
        dt,
        steps,
        output_every: t.output_every_steps,
    })
}

/// Tridiagonal matrix of order `N`, stored by three vectors of length `N`
/// (`lower[0]` and `upper[N-1]` are unused and zero). Entries are dimensionless and keep
/// their signs (`L_i <= 0`, `U_i < 0`).
#[derive(Clone, Debug)]
pub struct Tridiagonal {
    /// `L_i`.
    pub lower: Vec<f64>,
    /// `B_i`.
    pub diag: Vec<f64>,
    /// `U_i`.
    pub upper: Vec<f64>,
}

/// The assembled system of one step size.
#[derive(Clone, Debug)]
pub struct Assembly {
    /// Matrix `A` of (F19).
    pub matrix: Tridiagonal,
    /// Conductances and Fourier numbers used to build it.
    pub coeff: Coefficients,
    /// `Delta t` in s.
    pub dt: f64,
}

/// PS-05 matrix assembly. Rows `1 <= i <= N-1` are the one general row (F11) with the stored
/// conductances (F6) and weights (F7), for every node class; row 0 is the centre row (F12);
/// `L_1 = 0` exactly (F13); in row `N-1` the entry `U_{N-1}` is dropped from the matrix after
/// it has entered the diagonal (F17, outer value (F18)).
///
/// # Errors
///
/// A non-finite matrix entry.
pub fn assemble(grid: &Grid, coeff: Coefficients, dt: f64) -> Result<Assembly> {
    let n = grid.n;
    let mut lower = vec![0.0; n];
    let mut diag = vec![0.0; n];
    let mut upper = vec![0.0; n];

    let (b0, u0) = centre_row(coeff.fo[0]);
    diag[0] = b0;
    upper[0] = u0;

    for i in 1..n {
        let k_in = coeff.k_outer[i - 1];
        let k_out = coeff.k_outer[i];
        let l = -dt * k_in / grid.v[i];
        let u = -dt * k_out / grid.v[i];
        lower[i] = l;
        diag[i] = 1.0 - l - u;
        upper[i] = u;
    }
    lower[1] = 0.0;
    upper[n - 1] = 0.0;

    for i in 0..n {
        if !(lower[i].is_finite() && diag[i].is_finite() && upper[i].is_finite()) {
            bail!("matrix entry in row {i} is not finite");
        }
    }
    Ok(Assembly {
        matrix: Tridiagonal { lower, diag, upper },
        coeff,
        dt,
    })
}

/// PS-11 one Backward Euler step for a fixed assembled matrix.
#[derive(Clone, Debug)]
pub struct Stepper {
    assembly: Assembly,
    factors: ThomasFactors,
    rhs: Vec<f64>,
    work: Vec<f64>,
}

impl Stepper {
    /// Builds the coefficients (PS-04), the matrix (PS-05) and its Thomas factors (PS-10,
    /// the form (F24)).
    ///
    /// # Errors
    ///
    /// As [`Coefficients::new`] and [`assemble`]; a computed pivot that is not positive.
    pub fn new(grid: &Grid, d: [f64; 5], dt: f64) -> Result<Stepper> {
        let coeff = Coefficients::new(grid, d, dt)?;
        let assembly = assemble(grid, coeff, dt)?;
        let n = grid.n;
        let m = &assembly.matrix;
        let factors = ThomasFactors::new(&m.lower[1..n], &m.diag, &m.upper[..n - 1])?;
        Ok(Stepper {
            assembly,
            factors,
            rhs: vec![0.0; n],
            work: vec![0.0; n],
        })
    }

    /// The Thomas factors (pivots `m_i` and multipliers `w_i`) of the assembled matrix.
    pub fn factors(&self) -> &ThomasFactors {
        &self.factors
    }

    /// The assembled system.
    pub fn assembly(&self) -> &Assembly {
        &self.assembly
    }

    /// `K_{N-1,outer}`, the stored value used by row `N-1` and by the release (F26).
    pub fn k_last(&self) -> f64 {
        self.assembly.coeff.k_last()
    }

    /// Advances `C^j` (`c_old`, length `N+1`) to `C^{j+1}` (`c_new`, length `N+1`):
    /// right-hand side `b_i^j = C_i^j` (F20), the solve of (F19) for `C_0 .. C_{N-1}` by the
    /// Thomas algorithm in the form (F24) (`tridiagonal::ThomasFactors`), and `C_N^{j+1} = 0`
    /// (F18). `c_old[N]` is not read.
    ///
    /// # Errors
    ///
    /// A non-finite result of the solve.
    pub fn step(&mut self, c_old: &[f64], c_new: &mut [f64]) -> Result<()> {
        let n = self.assembly.matrix.diag.len();
        self.rhs.copy_from_slice(&c_old[..n]);
        self.factors
            .solve(&self.rhs, &mut c_new[..n], &mut self.work)?;
        c_new[n] = 0.0;
        Ok(())
    }
}

/// Run metadata.
#[derive(Clone, Debug)]
pub struct RunMeta {
    /// Solver label.
    pub solver: &'static str,
    /// Effective `Delta t` in s.
    pub dt: f64,
    /// Number of steps.
    pub steps: usize,
    /// `N`.
    pub n: usize,
    /// `theta` of (F22).
    pub theta: f64,
    /// `M^0` in mol.
    pub initial_inventory: f64,
    /// `Fo_material` (F8).
    pub fo: [f64; 5],
    /// Sign pattern and row margins of the assembled matrix (P1).
    pub matrix: MatrixReport,
    /// Smallest `m_i - |U_i|` over `0 <= i <= N-2` (P5 says at least 1 in exact arithmetic).
    pub min_pivot_margin: f64,
    /// Largest multiplier `|U_i| / m_i` over `0 <= i <= N-2` (P5 says below 1).
    pub max_multiplier: f64,
}

/// Result of [`run`].
#[derive(Clone, Debug)]
pub struct RunResult {
    /// The grid.
    pub grid: Grid,
    /// Output rows including `j = 0`.
    pub series: TimeSeries,
    /// Profile `C_i^J` at the end, length `N + 1`.
    pub final_profile: Vec<f64>,
    /// Profiles at the steps nearest the requested snapshot times (never interpolated).
    pub snapshots: Vec<(f64, Vec<f64>)>,
    /// Metadata.
    pub meta: RunMeta,
}

/// PS-16 time stepping from a validated configuration. See [`run_input`].
///
/// # Errors
///
/// Invalid input or a failed solve.
pub fn run(cfg: &Config, limits: &Limits, snapshot_times: &[f64]) -> Result<RunResult> {
    let input = validate_input(cfg, limits)?;
    run_input(&input, snapshot_times)
}

/// Runs the simulation: grid (PS-02), coefficients and matrix (PS-04, PS-05), initial vector
/// (F21, F22), then for each step the solve (F24, ThomasFactors), the
/// release rate (F26) at the new level, the cumulative release (F27), and at output steps the
/// inventory (F25), the release fraction (F28) and the residual (F30). The release is
/// accumulated at every step whatever the output stride. A non-finite output row is an error.
///
/// # Errors
///
/// A failed solve or a non-finite output.
pub fn run_input(input: &ValidatedInput, snapshot_times: &[f64]) -> Result<RunResult> {
    let grid = Grid::build(input)?;
    let mut stepper = Stepper::new(&grid, input.d, input.dt)?;
    let n = grid.n;
    let (mut c_old, theta, m0) = initial_vector(&grid, input.c_init);
    if !(m0.is_finite() && m0 > 0.0) {
        bail!("initial inventory is not finite and positive");
    }
    let mut c_new = vec![0.0; n + 1];
    let mut released = Accumulator::new();

    let (cmin, cmax) = min_max(&c_old);
    let mut bound = BalanceBound::new(n, input.steps, m0);
    let mut series: TimeSeries = vec![(
        0.0,
        StepDiagnostics {
            inventory: m0,
            cumulative_release: 0.0,
            release_fraction: 0.0,
            release_rate: None,
            balance_residual: balance_residual(m0, 0.0, m0),
            balance_residual_bound: bound.first_row(m0) * m0,
            min_concentration: cmin,
            max_concentration: cmax,
        },
    )];

    // Snapshot steps: nearest step to each requested time, clamped to 0 ..= steps.
    let mut wanted: Vec<usize> = snapshot_times
        .iter()
        .map(|&t| ((t / input.dt).round().max(0.0) as usize).min(input.steps))
        .collect();
    wanted.sort_unstable();
    wanted.dedup();
    let mut snapshots: Vec<(f64, Vec<f64>)> = Vec::new();
    if wanted.first() == Some(&0) {
        snapshots.push((0.0, c_old.clone()));
    }

    for step in 1..=input.steps {
        stepper.step(&c_old, &mut c_new)?;
        let phi = release_rate(stepper.k_last(), c_new[n - 1]);
        bound.add_step(&stepper.assembly().matrix, &grid.v, &c_new, input.dt * phi);
        released.add(input.dt * phi);
        std::mem::swap(&mut c_old, &mut c_new);
        let t = step as f64 * input.dt;
        if step % input.output_every == 0 || step == input.steps {
            let m = inventory(&grid, &c_old);
            let (cmin, cmax) = min_max(&c_old);
            let row = StepDiagnostics {
                inventory: m,
                cumulative_release: released.value(),
                release_fraction: release_fraction(released.value(), m0),
                release_rate: Some(phi),
                balance_residual: balance_residual(m, released.value(), m0),
                balance_residual_bound: bound.row(m, released.value()) * m0,
                min_concentration: cmin,
                max_concentration: cmax,
            };
            let finite = row.inventory.is_finite()
                && row.cumulative_release.is_finite()
                && row.release_fraction.is_finite()
                && phi.is_finite()
                && row.balance_residual.is_finite()
                && cmin.is_finite()
                && cmax.is_finite();
            if !finite {
                bail!("non-finite output at step {step}");
            }
            series.push((t, row));
        }
        if wanted.binary_search(&step).is_ok() {
            snapshots.push((t, c_old.clone()));
        }
    }

    let matrix_report = check_matrix_structure(&stepper.assembly().matrix)?;
    let pivots = stepper.factors().pivots();
    let upper = &stepper.assembly().matrix.upper;
    let mut min_pivot_margin = f64::INFINITY;
    let mut max_multiplier: f64 = 0.0;
    for i in 0..n - 1 {
        min_pivot_margin = min_pivot_margin.min(pivots[i] - upper[i].abs());
        max_multiplier = max_multiplier.max(upper[i].abs() / pivots[i]);
    }

    Ok(RunResult {
        grid,
        series,
        final_profile: c_old,
        snapshots,
        meta: RunMeta {
            solver: "STRATO BTCS five-layer (numerical solution of the stated model)",
            dt: input.dt,
            steps: input.steps,
            n,
            theta,
            initial_inventory: m0,
            fo: stepper.assembly().coeff.fo,
            matrix: matrix_report,
            min_pivot_margin,
            max_multiplier,
        },
    })
}

fn min_max(c: &[f64]) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &x in c {
        lo = lo.min(x);
        hi = hi.max(x);
    }
    (lo, hi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{InitialConfig, OutputConfig, RegionConfig, TimeConfig};
    use crate::numerical::interfaces::{interface_row_explicit, interior_row_explicit};
    use crate::numerical::tridiagonal;

    const RADII: [f64; 5] = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4];
    const D: [f64; 5] = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13];
    const LIMITS: Limits = Limits {
        max_intervals_total: 100_000,
        max_steps: 10_000_000,
    };

    fn config(cells: [usize; 5], dt: f64, end: f64) -> Config {
        let names = ["Kernel", "Buffer", "IPyC", "SiC", "OPyC"];
        Config {
            case_name: "unit".into(),
            species: "synthetic".into(),
            temperature_k: 1000.0,
            regions: (0..5)
                .map(|k| RegionConfig {
                    name: names[k].into(),
                    outer_radius_m: RADII[k],
                    diffusivity_m2_s: D[k],
                    cells: cells[k],
                })
                .collect(),
            initial: InitialConfig {
                kernel_concentration_mol_m3: 1.0,
            },
            time: TimeConfig {
                dt_s: dt,
                end_time_s: end,
                output_every_steps: 1,
            },
            output: OutputConfig {
                directory: "unused".into(),
            },
        }
    }

    fn stepper_for(cells: [usize; 5], dt: f64) -> (Grid, Stepper) {
        let g = Grid::from_radii(RADII, cells).unwrap();
        let s = Stepper::new(&g, D, dt).unwrap();
        (g, s)
    }

    fn dense_solve(a: &Tridiagonal, b: &[f64]) -> Vec<f64> {
        let n = a.diag.len();
        let mut m = vec![vec![0.0; n + 1]; n];
        for i in 0..n {
            m[i][i] = a.diag[i];
            if i > 0 {
                m[i][i - 1] = a.lower[i];
            }
            if i + 1 < n {
                m[i][i + 1] = a.upper[i];
            }
            m[i][n] = b[i];
        }
        for c in 0..n {
            let p = (c..n)
                .max_by(|&r, &s| m[r][c].abs().total_cmp(&m[s][c].abs()))
                .unwrap();
            m.swap(c, p);
            for r in c + 1..n {
                let f = m[r][c] / m[c][c];
                for k in c..=n {
                    let v = m[c][k];
                    m[r][k] -= f * v;
                }
            }
        }
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let s: f64 = (i + 1..n).map(|k| m[i][k] * x[k]).sum();
            x[i] = (m[i][n] - s) / m[i][i];
        }
        x
    }

    #[test]
    fn one_step_matches_dense_solve() {
        let (g, mut s) = stepper_for([6, 5, 4, 5, 4], 200.0);
        let (c_old, _, _) = initial_vector(&g, 1.0);
        let mut c_new = vec![0.0; g.n + 1];
        s.step(&c_old, &mut c_new).unwrap();
        let a = &s.assembly().matrix;
        let dense = dense_solve(a, &c_old[..g.n]);
        // Derived bounds (u = 2^-53, gamma_k = k u / (1 - k u)).
        // (1) Backward error of (F24). For this M-matrix (P1, P2) the factors satisfy
        //     |L||U| <= (1 + O(u)) |A| (L, U have the sign pattern of A, so no cancellation). The
        //     backward error of tridiagonal LU followed by the two bidiagonal solves is derived
        //     in `tridiagonal::tests::thomas_matches_dense_f24` (item 1 there):
        //     (A + dA) x = b with |dA| <= kappa_t |A|, kappa_t = 4u + O(u^2) <= gamma_5.
        //     Hence the exact residual obeys |b - A x|_i <= gamma_5 (|A||x|)_i.
        // (2) The residual computed here has 3 products, 3 additions and the b term, so it
        //     differs from the exact residual by at most gamma_4 (|A||x| + |b|)_i.
        // (3) The pivoted dense reference y has no backward-error bound assumed: its own
        //     measured residual enters instead. Since x - y = A^-1 ((b - A y) - (b - A x)),
        //     |x - y|_inf <= ||A^-1||_inf (||r_x||_inf + ||r_y||_inf), with
        //     ||A^-1||_inf <= 1 / (min row margin |B_i| - |L_i| - |U_i|) (row dominance, P1;
        //     Varah's bound), evaluated from the stored matrix.
        let n = g.n;
        let residual = |x: &[f64]| -> (Vec<f64>, Vec<f64>) {
            let mut r = vec![0.0; n];
            let mut sc = vec![0.0; n];
            for i in 0..n {
                let mut v = c_old[i] - a.diag[i] * x[i];
                let mut s = c_old[i].abs() + (a.diag[i] * x[i]).abs();
                if i > 0 {
                    v -= a.lower[i] * x[i - 1];
                    s += (a.lower[i] * x[i - 1]).abs();
                }
                if i + 1 < n {
                    v -= a.upper[i] * x[i + 1];
                    s += (a.upper[i] * x[i + 1]).abs();
                }
                r[i] = v;
                sc[i] = s;
            }
            (r, sc)
        };
        let u = f64::EPSILON / 2.0;
        let gamma = |k: f64| k * u / (1.0 - k * u);
        // The margin is evaluated in floating point (two subtractions, error at most
        // gamma_2 (|B| + |L| + |U|)); that error is subtracted, so `margin` is a rigorous lower
        // bound of the true row margin. No fixed value (such as 1) is assumed: Varah's bound
        // only needs margin > 0.
        let margin = (0..n)
            .map(|i| {
                let l = if i > 0 { a.lower[i].abs() } else { 0.0 };
                let up = if i + 1 < n { a.upper[i].abs() } else { 0.0 };
                (a.diag[i].abs() - l - up) - gamma(2.0) * (a.diag[i].abs() + l + up)
            })
            .fold(f64::INFINITY, f64::min);
        assert!(margin > 0.0, "row margin {margin}");
        let inv_norm = 1.0 / margin;
        let (rx, sx) = residual(&c_new[..n]);
        let (ry, sy) = residual(&dense);
        let mut res_x = 0.0f64;
        let mut res_y = 0.0f64;
        for i in 0..n {
            // |A||x|_i = sx_i - |b_i|.
            let abs_ax = sx[i] - c_old[i].abs();
            assert!(
                rx[i].abs() <= gamma(5.0) * abs_ax + gamma(4.0) * sx[i],
                "residual row {i}: {}",
                rx[i]
            );
            res_x = res_x.max(rx[i].abs() + gamma(4.0) * sx[i]);
            res_y = res_y.max(ry[i].abs() + gamma(4.0) * sy[i]);
        }
        let tol = inv_norm * (res_x + res_y);
        for i in 0..n {
            assert!(
                (c_new[i] - dense[i]).abs() <= tol,
                "node {i}: {} vs {} (tol {tol})",
                c_new[i],
                dense[i]
            );
        }
        assert_eq!(c_new[n], 0.0);
    }
    #[test]
    fn production_solve_matches_general_tridiagonal_module() {
        // Decision 5: the production path is the F24 form; the general module is the
        // cross-check. Bound: each of the two solves has backward error |dA| <= gamma_5 |A|
        // (Higham, Section 9.6; see one_step_matches_dense_solve), so with ||A^-1||_inf <= 1
        // they differ by at most 2 gamma_5 ||A||_inf ||x||_inf, which is the tolerance used (with
        // gamma_5 = 5u / (1 - 5u) exactly, no margin).
        for (counts, dt) in [([6, 5, 4, 5, 4], 200.0), ([20, 20, 20, 20, 20], 1.0e5)] {
            let (g, mut s) = stepper_for(counts, dt);
            let (mut c_old, _, _) = initial_vector(&g, 1.0);
            let mut c_new = vec![0.0; g.n + 1];
            for step in 0..5 {
                s.step(&c_old, &mut c_new).unwrap();
                let a = &s.assembly().matrix;
                let n = g.n;
                let general =
                    tridiagonal::solve(&a.lower[1..n], &a.diag, &a.upper[..n - 1], &c_old[..n])
                        .unwrap();
                let norm = (0..n)
                    .map(|i| a.diag[i].abs() + a.lower[i].abs() + a.upper[i].abs())
                    .fold(0.0, f64::max);
                let xmax = general.iter().cloned().fold(0.0, f64::max);
                let u = f64::EPSILON / 2.0;
                let tol = 2.0 * (5.0 * u / (1.0 - 5.0 * u)) * norm * xmax;
                for i in 0..n {
                    assert!(
                        (c_new[i] - general[i]).abs() <= tol,
                        "step {step} node {i}: {} vs {} (tol {tol})",
                        c_new[i],
                        general[i]
                    );
                }
                std::mem::swap(&mut c_old, &mut c_new);
            }
        }
    }

    #[test]
    fn rows_satisfy_flux_balance_f10() {
        let (g, mut s) = stepper_for([8, 6, 5, 7, 6], 50.0);
        let (c_old, _, _) = initial_vector(&g, 1.0);
        let mut c_new = vec![0.0; g.n + 1];
        s.step(&c_old, &mut c_new).unwrap();
        let k = &s.assembly().coeff.k_outer;
        for i in 1..g.n {
            let lhs = g.v[i] * (c_new[i] - c_old[i]) / 50.0;
            let rhs = k[i] * (c_new[i + 1] - c_new[i]) - k[i - 1] * (c_new[i] - c_new[i - 1]);
            // Derived bound (units of V/dt times concentration), u = 2^-53. Per unit of S below:
            //  - backward error of the solve, |dA| <= kappa_t |A|, kappa_t = 4u + O(u^2)
            //    (see one_step_matches_dense_solve, item 1): 4u;
            //  - stored entries. The product dt K is rounded once and shared by the two rows that
            //    use K_i (the same operands), which is a relative change u of K_i (it defines
            //    K' = P/(-dt)); against the exact-form row with K' and the same V, the stored L
            //    and U carry one rounding each (u |L||x_{i-1}|, u |U||x_{i+1}|), and B =
            //    fl(fl(1 - l) - u) has two roundings of positive terms plus the change of
            //    |L| + |U|: gamma_2 B + u (|L| + |U|)/(1 - u) <= 3u B + O(u^2) B, so 3u in all
            //    (no modified V is needed);
            //  - the shared dt K rounding against the K used in rhs here: u;
            //  - evaluating lhs here (3 roundings) and rhs (two flux terms of 2 roundings each
            //    and the final subtraction: 3 roundings): 6u.
            // The counts add to (4 + 3 + 1 + 6) u = 14 u S. (An earlier version of this comment
            // counted 13 u S; it missed the shared dt K rounding.) With
            // S = (V/dt)(|c_i^new| + |c_i^old|) + k_i (|c_{i+1}| + |c_i|) + k_{i-1} (|c_i| + |c_{i-1}|),
            // which bounds each partial scale. O(u^2) terms are below 1e-30 S and are absorbed by
            // the factor (1 + 100 u) used on the bound.
            let scale = g.v[i] / 50.0 * (c_new[i].abs() + c_old[i].abs())
                + k[i] * (c_new[i + 1].abs() + c_new[i].abs())
                + k[i - 1] * (c_new[i].abs() + c_new[i - 1].abs());
            let u = f64::EPSILON / 2.0;
            assert!(
                (lhs - rhs).abs() <= 14.0 * u * (1.0 + 100.0 * u) * scale,
                "row {i}: {lhs} vs {rhs}"
            );
        }
    }

    #[test]
    fn general_row_f11_matches_explicit_forms() {
        // Derived tolerance, relative to the largest entry of the row, u = 2^-53,
        // rho = r_N / dr_min. A stored node is r_i = fl(R_prev + fl(j dr)) with
        // dr = fl(fl(R_k - R_prev)/n_k): its distance to the ideal node R_prev + j h is at most
        // 3 u r_i (u r_i for the sum, gamma_2 j h for dr and the product); interface and outer
        // radii are exact. Hence the stored difference a = r_i - r_{i-1} satisfies
        // |a/h - 1| <= 6 u rho + u, and (a + b)/(2h) - 1 = (r_{i+1} - r_{i-1})/(2h) - 1 is at
        // most 3 u rho + u; the explicit forms use dr = h (1 + gamma_2) instead.
        // The assembled entry is -2 dt D r_{i-1} / (a r_i (a + b)) (1 + theta) with 8 roundings
        // (K: 3, dt K: 1, V: 3, final division: 1); the explicit one is -fo (1 - dr/r_i) with
        // 6 roundings (fo: 3, dr/r: 1, 1 - .: 1, product: 1), and the cancellation in 1 - dr/r_i
        // (i >= 2 in the kernel, dr/r <= 1/2) costs at most 2u more. The numerators differ by
        // (a - dr), at most 6 u rho + 3u in relative size of r_i - dr >= r_i/2. The first-order
        // terms add up: the three radius effects (a against dr: 6 rho + 3; the numerator: 6 rho + 3;
        // (a + b) against 2 dr: 3 rho + 3) give 15 u rho + 9u, the roundings 8 + 6 + 2 = 16u, so an
        // off-diagonal entry has relative error E <= 15 u rho + 25 u; the diagonal
        // B = 1 + |L| + |U| adds two roundings and one more for the explicit 1 + 2 fo: E + 3u;
        // interface rows add three roundings (s, 2 dr/s, one product) and have exact interface
        // radii. One bound is used for all rows: E_tot = (15 rho + 31) u, plus the second-order
        // allowance 2 E_tot^2 (the neglected terms are products of two such quantities).
        let counts = [7, 5, 6, 9, 8];
        let (g, s) = stepper_for(counts, 120.0);
        let a = &s.assembly();
        let c = &a.coeff;
        let dr_min = g.dr_region.iter().cloned().fold(f64::INFINITY, f64::min);
        let rho = g.r[g.n] / dr_min;
        let e_tot = (15.0 * rho + 31.0) * (f64::EPSILON / 2.0);
        let rel = e_tot + 2.0 * e_tot * e_tot;
        let within = |x: f64, y: f64, scale: f64| (x - y).abs() <= rel * scale;
        for i in 1..g.n {
            let (l, b, u) = (a.matrix.lower[i], a.matrix.diag[i], a.matrix.upper[i]);
            let scale = l.abs().max(b.abs()).max(u.abs());
            match g.node_class(i) {
                crate::numerical::grid::NodeClass::Interior(reg) => {
                    let k = reg.index();
                    let (le, be, ue) = interior_row_explicit(c.fo[k], g.dr_region[k], g.r[i]);
                    assert!(within(l, le, scale), "L interior {i}: {l} {le}");
                    assert!(within(b, be, scale), "B interior {i}");
                    // U_{N-1} is not a matrix entry (F17); its value is inside B_{N-1}, checked above.
                    if i + 1 < g.n {
                        assert!(within(u, ue, scale), "U interior {i}");
                    }
                }
                crate::numerical::grid::NodeClass::Interface { inner, outer } => {
                    let (ki, ko) = (inner.index(), outer.index());
                    let (le, be, ue) = interface_row_explicit(
                        c.fo[ki],
                        c.fo[ko],
                        g.dr_region[ki],
                        g.dr_region[ko],
                        g.r[i],
                    );
                    assert!(within(l, le, scale), "L interface {i}: {l} {le}");
                    assert!(within(b, be, scale), "B interface {i}");
                    assert!(within(u, ue, scale), "U interface {i}");
                }
                _ => unreachable!(),
            }
            // Row-sum identity of (F11), computed with the stored numbers.
            let (lm, bm, um) = (a.matrix.lower[i], a.matrix.diag[i], a.matrix.upper[i]);
            // (In row N-1 the entry U is not stored (F17); its value sits inside B.)
            if i + 1 < g.n {
                assert_eq!(bm, 1.0 - lm - um);
            }
        }
    }

    #[test]
    fn kernel_rows_use_one_over_i_f15_and_negative_control() {
        let (g, s) = stepper_for([7, 5, 6, 9, 8], 120.0);
        let a = s.assembly();
        let fo = a.coeff.fo[0];
        // Derived bound, with the notation of general_row_f11_matches_explicit_forms (u = 2^-53,
        // rho = r_kernel / dr = n_kernel here). The radius effects give 15 u rho + 9u relative
        // error of the entry. Roundings: assembled entry 8; the expected value
        // -fo (1 - 1/i) has 3 more (1/i, 1 - ., product) and fo itself 3 (it is shared with the
        // ideal D dt/dr^2); the cancellation in 1 - 1/i (i >= 2) at most 2u: 16u. Relative error
        // E = 15 u rho + 25 u; the entries are at most 2 fo in magnitude (|1 +- 1/i| <= 2
        // for i >= 1), so tol = 2 fo (E + 2 E^2), the second term being the allowance for the
        // neglected products of two small quantities.
        let r_kernel = g.r[g.interface_nodes[0]];
        let rho = r_kernel / g.dr_region[0];
        let e_f15 = (15.0 * rho + 25.0) * (f64::EPSILON / 2.0);
        let tol_f15 = 2.0 * fo * (e_f15 + 2.0 * e_f15 * e_f15);
        for i in 2..g.interface_nodes[0] {
            let expect_l = -fo * (1.0 - 1.0 / i as f64);
            let expect_u = -fo * (1.0 + 1.0 / i as f64);
            assert!((a.matrix.lower[i] - expect_l).abs() <= tol_f15);
            assert!((a.matrix.upper[i] - expect_u).abs() <= tol_f15);
        }
        // The 1/i factor applied in a buffer node is not the matrix row.
        let i = g.interface_nodes[0] + 2;
        let wrong_l = -a.coeff.fo[1] * (1.0 - 1.0 / (i as f64));
        // The control must differ from the stored entry by more than the rounding tolerance of
        // the comparison itself (the F15 tolerance scaled to the buffer's Fourier number), so
        // that the test could not mistake it for a rounding difference.
        let e_buf = (15.0 * (g.r[g.n] / g.dr_region.iter().cloned().fold(f64::INFINITY, f64::min))
            + 31.0)
            * (f64::EPSILON / 2.0);
        assert!(
            (a.matrix.lower[i] - wrong_l).abs()
                > 2.0 * a.coeff.fo[1] * (e_buf + 2.0 * e_buf * e_buf)
        );
    }

    #[test]
    fn centre_is_one_way_f13() {
        let (g, mut s) = stepper_for([6, 5, 4, 5, 4], 200.0);
        assert_eq!(s.assembly().matrix.lower[1].to_bits(), 0.0f64.to_bits());
        let (mut c_a, _, _) = initial_vector(&g, 1.0);
        let mut c_b = c_a.clone();
        c_b[0] = 12345.0;
        let mut out_a = vec![0.0; g.n + 1];
        let mut out_b = vec![0.0; g.n + 1];
        s.step(&c_a, &mut out_a).unwrap();
        s.step(&c_b, &mut out_b).unwrap();
        for i in 1..=g.n {
            assert_eq!(out_a[i].to_bits(), out_b[i].to_bits());
        }
        assert!(out_a[0] != out_b[0]);
        c_a[0] = 0.0;
    }

    #[test]
    fn last_row_keeps_full_diagonal_f17_and_single_k() {
        for counts in [[6, 5, 4, 5, 4], [6, 5, 4, 5, 1]] {
            let (g, s) = stepper_for(counts, 200.0);
            let a = s.assembly();
            let n = g.n;
            let u_last = -200.0 * a.coeff.k_outer[n - 1] / g.v[n - 1];
            let expect = 1.0 + a.matrix.lower[n - 1].abs() + u_last.abs();
            // No tolerance: `assemble` computes B = fl(fl(1 - l) - u) with the same l and
            // u (the same operands, so the same roundings) and 1 - l = 1 + |l|, - u = + |u|.
            assert_eq!(a.matrix.diag[n - 1].to_bits(), expect.to_bits());
            assert_eq!(a.matrix.upper[n - 1], 0.0);
            assert_eq!(s.k_last().to_bits(), a.coeff.k_outer[n - 1].to_bits());
        }
    }

    #[test]
    fn matrix_is_tridiagonal_and_reassembly_differs_f19() {
        let (g, s1) = stepper_for([6, 5, 4, 5, 4], 200.0);
        let s2 = Stepper::new(&g, D, 100.0).unwrap();
        assert_eq!(s1.assembly().matrix.diag.len(), g.n);
        assert_eq!(s1.assembly().matrix.lower[0], 0.0);
        assert!(s1.assembly().matrix.diag[3] != s2.assembly().matrix.diag[3]);
    }

    #[test]
    fn rhs_is_old_profile_f20_small_dt_returns_old() {
        let (g, mut s) = stepper_for([6, 5, 4, 5, 4], 1e-9);
        let (c_old, _, _) = initial_vector(&g, 1.0);
        let mut c_new = vec![0.0; g.n + 1];
        s.step(&c_old, &mut c_new).unwrap();
        // Derived bound, u = 2^-53, gamma_k = k u/(1 - k u). Let x be the computed new profile
        // and r = c - A x the exact residual. Then A x = c - r, so A (x - c) = -r - (A - I) c,
        // and
        //   |x - c|_inf <= ||A^-1||_inf (|r|_inf + |(A - I) c|_inf),
        // with ||A^-1||_inf <= 1/margin (Varah, row dominance, margin = min row margin, from
        // the stored matrix with its evaluation error subtracted). |r|_i is at most the
        // evaluated residual plus gamma_4 (|A||x| + |c|)_i (3 products, 3 sums, c);
        // |(A - I) c|_i <= |L_i||c_{i-1}| + |B_i - 1||c_i| + |U_i||c_{i+1}|, evaluated with the
        // factor (1 + gamma_5). For dt = 1e-9 s every |L|, |U|, |B - 1| is about 1e-15 or
        // smaller, so the bound is of that size, not a fixed constant.
        let a = &s.assembly().matrix;
        let n = g.n;
        let u = f64::EPSILON / 2.0;
        let gamma = |k: f64| k * u / (1.0 - k * u);
        let margin = (0..n)
            .map(|i| {
                let l = if i > 0 { a.lower[i].abs() } else { 0.0 };
                let up = if i + 1 < n { a.upper[i].abs() } else { 0.0 };
                (a.diag[i].abs() - l - up) - gamma(2.0) * (a.diag[i].abs() + l + up)
            })
            .fold(f64::INFINITY, f64::min);
        assert!(margin > 0.0);
        let mut worst = 0.0f64;
        for i in 0..n {
            let (cm, cp) = (
                if i > 0 { c_old[i - 1] } else { 0.0 },
                if i + 1 < n { c_old[i + 1] } else { 0.0 },
            );
            let (lo, up) = (
                if i > 0 { a.lower[i] } else { 0.0 },
                if i + 1 < n { a.upper[i] } else { 0.0 },
            );
            let mut r = c_old[i] - a.diag[i] * c_new[i];
            let mut sc = c_old[i].abs() + (a.diag[i] * c_new[i]).abs();
            if i > 0 {
                r -= lo * c_new[i - 1];
                sc += (lo * c_new[i - 1]).abs();
            }
            if i + 1 < n {
                r -= up * c_new[i + 1];
                sc += (up * c_new[i + 1]).abs();
            }
            let a_minus_i = (lo.abs() * cm.abs()
                + (a.diag[i] - 1.0).abs() * c_old[i].abs()
                + up.abs() * cp.abs())
                * (1.0 + gamma(5.0));
            worst = worst.max(r.abs() + gamma(4.0) * sc + a_minus_i);
        }
        let tol = worst / margin;
        for i in 0..n {
            assert!((c_new[i] - c_old[i]).abs() <= tol, "node {i}: tol {tol}");
        }
    }

    #[test]
    fn rejects_invalid_input() {
        let ok = config([3, 3, 3, 3, 3], 10.0, 100.0);
        assert!(validate_input(&ok, &LIMITS).is_ok());
        let mut c = ok.clone();
        c.regions[2].outer_radius_m = c.regions[1].outer_radius_m;
        assert!(validate_input(&c, &LIMITS).is_err());
        let mut c = ok.clone();
        c.regions[1].diffusivity_m2_s = 0.0;
        assert!(validate_input(&c, &LIMITS).is_err());
        let mut c = ok.clone();
        c.regions[1].diffusivity_m2_s = f64::INFINITY;
        assert!(validate_input(&c, &LIMITS).is_err());
        let mut c = ok.clone();
        c.regions[4].cells = 0;
        assert!(validate_input(&c, &LIMITS).is_err());
        let mut c = ok.clone();
        c.time.dt_s = 0.0;
        assert!(validate_input(&c, &LIMITS).is_err());
        let mut c = ok.clone();
        c.time.end_time_s = f64::NAN;
        assert!(validate_input(&c, &LIMITS).is_err());
        let mut c = ok.clone();
        c.initial.kernel_concentration_mol_m3 = 0.0;
        assert!(validate_input(&c, &LIMITS).is_err());
        let mut c = ok.clone();
        c.time.output_every_steps = 0;
        assert!(validate_input(&c, &LIMITS).is_err());
        let mut c = ok.clone();
        c.regions.pop();
        assert!(validate_input(&c, &LIMITS).is_err());
        let tight = Limits {
            max_intervals_total: 10,
            max_steps: 10_000_000,
        };
        assert!(validate_input(&ok, &tight).is_err());
        let few = Limits {
            max_intervals_total: 1000,
            max_steps: 5,
        };
        assert!(validate_input(&ok, &few).is_err());
    }

    #[test]
    fn step_count_rule() {
        let c = config([3, 3, 3, 3, 3], 30.0, 100.0);
        let v = validate_input(&c, &LIMITS).unwrap();
        assert_eq!(v.steps, 4);
        assert_eq!(v.dt, 25.0);
    }

    #[test]
    fn run_records_row_zero_and_snapshots() {
        let c = config([6, 5, 4, 5, 4], 200.0, 2000.0);
        let r = run(&c, &LIMITS, &[0.0, 1000.0, 2000.0, 1.0e9]).unwrap();
        assert_eq!(r.series.len(), 11);
        let row0 = r.series[0].1;
        assert_eq!(row0.release_rate, None);
        assert_eq!(row0.release_fraction, 0.0);
        assert_eq!(row0.balance_residual, 0.0);
        assert_eq!(r.series[10].0, 2000.0);
        assert!(r.series[1].1.release_rate.is_some());
        // Snapshots: 0, 1000 (step 5), 2000 (step 10; the 1e9 request clamps to the last step).
        let times: Vec<f64> = r.snapshots.iter().map(|s| s.0).collect();
        assert_eq!(times, vec![0.0, 1000.0, 2000.0]);
        assert_eq!(r.snapshots[2].1, r.final_profile);
        assert_eq!(*r.final_profile.last().unwrap(), 0.0);
    }

    #[test]
    fn cumulative_release_is_sum_of_rates() {
        let c = config([6, 5, 4, 5, 4], 200.0, 4000.0);
        let r = run(&c, &LIMITS, &[]).unwrap();
        // No tolerance: (F27) is the compensated sum of the increments dt * Phi in the order of
        // the steps. Re-accumulating the same increments with the same accumulator must give the
        // same bits; the quantitative accuracy of that accumulator enters the balance bound of
        // `tests/btcs_verification.rs`.
        let mut acc = crate::numerical::inventory::Accumulator::new();
        for (_, row) in r.series.iter().skip(1) {
            acc.add(r.meta.dt * row.release_rate.unwrap());
        }
        let last = r.series.last().unwrap().1;
        assert_eq!(acc.value().to_bits(), last.cumulative_release.to_bits());
    }
}
