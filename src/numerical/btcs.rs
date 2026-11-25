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
    diagnostics::{StepDiagnostics, TimeSeries},
    grid::{Grid, ValidatedInput},
    inventory::{balance_residual, initial_vector, inventory, release_fraction, Accumulator},
    tridiagonal,
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
    work: Vec<f64>,
}

impl Stepper {
    /// Builds the coefficients (PS-04) and the matrix (PS-05).
    ///
    /// # Errors
    ///
    /// As [`Coefficients::new`] and [`assemble`].
    pub fn new(grid: &Grid, d: [f64; 5], dt: f64) -> Result<Stepper> {
        let coeff = Coefficients::new(grid, d, dt)?;
        let assembly = assemble(grid, coeff, dt)?;
        let n = grid.n;
        Ok(Stepper {
            assembly,
            work: vec![0.0; n],
        })
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
    /// right-hand side `b_i^j = C_i^j` (F20), tridiagonal solve of (F19) for `C_0 .. C_{N-1}`
    /// by `numerical::tridiagonal`, and `C_N^{j+1} = 0` (F18). `c_old[N]` is not read.
    ///
    /// # Errors
    ///
    /// A failed tridiagonal solve (zero or non-finite pivot).
    pub fn step(&mut self, c_old: &[f64], c_new: &mut [f64]) -> Result<()> {
        let n = self.assembly.matrix.diag.len();
        c_new[..n].copy_from_slice(&c_old[..n]);
        let m = &self.assembly.matrix;
        tridiagonal::solve_in_place(
            &m.lower[1..n],
            &m.diag,
            &m.upper[..n - 1],
            &mut c_new[..n],
            &mut self.work,
        )?;
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
/// (F21, F22), then for each step the solve (F24 form via the tridiagonal module), the
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
    let mut series: TimeSeries = vec![(
        0.0,
        StepDiagnostics {
            inventory: m0,
            cumulative_release: 0.0,
            release_fraction: 0.0,
            release_rate: None,
            balance_residual: balance_residual(m0, 0.0, m0),
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
        // Pre-registered bound: error of a backward-stable solve is about cond * u * |x|,
        // with cond <= ||A||_inf * ||A^-1||_inf <= ||A||_inf (P2).
        let norm = (0..g.n)
            .map(|i| a.diag[i].abs() + a.lower[i].abs() + a.upper[i].abs())
            .fold(0.0, f64::max);
        let tol = 100.0 * norm * f64::EPSILON / 2.0;
        for i in 0..g.n {
            assert!(
                (c_new[i] - dense[i]).abs() <= tol,
                "node {i}: {} vs {} (tol {tol})",
                c_new[i],
                dense[i]
            );
        }
        assert_eq!(c_new[g.n], 0.0);
        // Residual of the system.
        for i in 0..g.n {
            let mut r = a.diag[i] * c_new[i] - c_old[i];
            if i > 0 {
                r += a.lower[i] * c_new[i - 1];
            }
            if i + 1 < g.n {
                r += a.upper[i] * c_new[i + 1];
            }
            assert!(r.abs() <= tol, "residual row {i}: {r}");
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
            let scale = (k[i] + k[i - 1]) * c_new[i].abs().max(1e-300)
                + (k[i] * c_new[i + 1].abs() + k[i - 1] * c_new[i - 1].abs())
                + lhs.abs();
            assert!(
                (lhs - rhs).abs() <= 1e4 * f64::EPSILON * scale + 1e-30,
                "row {i}: {lhs} vs {rhs}"
            );
        }
    }

    #[test]
    fn general_row_f11_matches_explicit_forms() {
        // Pre-registered tolerance: stored radii carry u*r absolute error, so a difference of
        // radii has relative error up to 2u*r/dr, and L, U contain three such differences:
        // tol = 8 eps (r_N / dr_min) + 64 eps (relative to the largest entry of the row).
        let counts = [7, 5, 6, 9, 8];
        let (g, s) = stepper_for(counts, 120.0);
        let a = &s.assembly();
        let c = &a.coeff;
        let dr_min = g.dr_region.iter().cloned().fold(f64::INFINITY, f64::min);
        let rel = 8.0 * f64::EPSILON * g.r[g.n] / dr_min + 64.0 * f64::EPSILON;
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
        for i in 2..g.interface_nodes[0] {
            let expect_l = -fo * (1.0 - 1.0 / i as f64);
            let expect_u = -fo * (1.0 + 1.0 / i as f64);
            assert!((a.matrix.lower[i] - expect_l).abs() <= 1e-12 * fo);
            assert!((a.matrix.upper[i] - expect_u).abs() <= 1e-12 * fo);
        }
        // The 1/i factor applied in a buffer node is not the matrix row.
        let i = g.interface_nodes[0] + 2;
        let wrong_l = -a.coeff.fo[1] * (1.0 - 1.0 / (i as f64));
        assert!((a.matrix.lower[i] - wrong_l).abs() > 1e-6 * a.coeff.fo[1]);
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
            assert!((a.matrix.diag[n - 1] - expect).abs() <= 8.0 * f64::EPSILON * expect);
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
        for i in 0..g.n {
            assert!((c_new[i] - c_old[i]).abs() < 1e-6, "node {i}");
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
        let mut sum = 0.0;
        for (_, row) in r.series.iter().skip(1) {
            sum += r.meta.dt * row.release_rate.unwrap();
        }
        let last = r.series.last().unwrap().1;
        assert!((sum - last.cumulative_release).abs() <= 1e-12 * sum.abs());
    }
}
