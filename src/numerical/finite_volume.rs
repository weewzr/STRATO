//! Code B: discrete approach. Conservative spherical finite volumes, Backward Euler in
//! time, solved with a (subtraction-free) tridiagonal elimination.

use crate::config::Config;
use anyhow::{bail, Result};
use std::f64::consts::PI;

#[derive(Clone, Debug)]
pub struct Cell {
    pub left: f64,
    pub right: f64,
    pub centroid: f64,
    pub volume: f64,
    pub material: usize,
    pub diffusivity: f64,
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub cells: Vec<Cell>,
    pub conductance: Vec<f64>,
    pub outer_conductance: f64,
    pub outer_radius: f64,
}

pub type TimeSeries = Vec<(f64, StepDiagnostics)>;

#[derive(Clone, Debug)]
pub struct StepDiagnostics {
    pub inventory: f64,
    pub cumulative_release: f64,
    pub release_fraction: f64,
    pub release_rate: f64,
    pub balance_residual: f64,
    pub min_concentration: f64,
}

pub fn shell_volume(a: f64, b: f64) -> f64 {
    4.0 * PI / 3.0 * (b.powi(3) - a.powi(3))
}

pub fn volume_centroid(a: f64, b: f64) -> f64 {
    if a == 0.0 {
        0.75 * b
    } else {
        0.75 * (b.powi(4) - a.powi(4)) / (b.powi(3) - a.powi(3))
    }
}

pub fn build_mesh(cfg: &Config) -> Result<Mesh> {
    cfg.validate()?;
    let mut cells = Vec::new();
    let mut inner = 0.0;
    for (mat, region) in cfg.regions.iter().enumerate() {
        let width = (region.outer_radius_m - inner) / region.cells as f64;
        for k in 0..region.cells {
            let left = inner + k as f64 * width;
            let right = if k + 1 == region.cells {
                region.outer_radius_m
            } else {
                inner + (k + 1) as f64 * width
            };
            cells.push(Cell {
                left,
                right,
                centroid: volume_centroid(left, right),
                volume: shell_volume(left, right),
                material: mat,
                diffusivity: region.diffusivity_m2_s,
            });
        }
        inner = region.outer_radius_m;
    }
    if cells.len() < 2 {
        bail!("mesh requires at least two cells");
    }

    let mut conductance = Vec::with_capacity(cells.len() - 1);
    for pair in cells.windows(2) {
        let left = &pair[0];
        let right = &pair[1];
        let face = left.right;
        let area = 4.0 * PI * face * face;
        let dl = face - left.centroid;
        let dr = right.centroid - face;
        conductance.push(area / (dl / left.diffusivity + dr / right.diffusivity));
    }
    let last = cells.last().expect("nonempty mesh");
    let outer_radius = cfg.regions.last().expect("five regions").outer_radius_m;
    let outer_area = 4.0 * PI * outer_radius * outer_radius;
    let outer_conductance = outer_area * last.diffusivity / (outer_radius - last.centroid);

    Ok(Mesh {
        cells,
        conductance,
        outer_conductance,
        outer_radius,
    })
}

pub fn initial_concentration(cfg: &Config, mesh: &Mesh) -> Vec<f64> {
    mesh.cells
        .iter()
        .map(|c| {
            if c.material == 0 {
                cfg.initial.kernel_concentration_mol_m3
            } else {
                0.0
            }
        })
        .collect()
}

pub fn inventory(mesh: &Mesh, c: &[f64]) -> f64 {
    mesh.cells
        .iter()
        .zip(c)
        .map(|(cell, &x)| cell.volume * x)
        .sum()
}

/// Solve the Backward-Euler system
///   -gw_i x_{i-1} + (v_i + gw_i + ge_i) x_i - ge_i x_{i+1} = rhs_i,   v_i = V_i / dt,
/// where gw_{i+1} = ge_i and the last cell has a Dirichlet (absorbing) face.
///
/// This is Thomas elimination written without any subtraction. The textbook pivot
/// b_i - a_i cp_{i-1} subtracts two nearly equal numbers when the Fourier number
/// D dt / dr^2 is large, which loses digits and breaks the inventory balance. Writing
/// the pivot as den_i = e_i + ge_i with e_i = v_i + gw_i e_{i-1} / (e_{i-1} + gw_i)
/// uses only sums, products and quotients of nonnegative numbers, so every operation
/// is componentwise accurate.
fn solve_diffusion_system(v: &[f64], gw: &[f64], ge: &[f64], rhs: &[f64]) -> Result<Vec<f64>> {
    let n = v.len();
    if gw.len() != n || ge.len() != n || rhs.len() != n || n == 0 {
        bail!("invalid system dimensions");
    }
    let mut den = vec![0.0; n];
    let mut dp = vec![0.0; n];
    let mut e_prev = 0.0;
    for i in 0..n {
        let e = if i == 0 {
            v[0]
        } else {
            v[i] + gw[i] * e_prev / (e_prev + gw[i])
        };
        den[i] = e + ge[i];
        if den[i] <= 0.0 || !den[i].is_finite() {
            bail!("singular or non-finite pivot at cell {i}");
        }
        let carry = if i == 0 { 0.0 } else { gw[i] * dp[i - 1] };
        dp[i] = (rhs[i] + carry) / den[i];
        e_prev = e;
    }
    let mut x = vec![0.0; n];
    x[n - 1] = dp[n - 1];
    for i in (0..n - 1).rev() {
        x[i] = dp[i] + ge[i] / den[i] * x[i + 1];
    }
    Ok(x)
}

pub fn backward_euler_step(mesh: &Mesh, old: &[f64], dt: f64) -> Result<Vec<f64>> {
    let n = mesh.cells.len();
    if old.len() != n || dt <= 0.0 || !dt.is_finite() {
        bail!("invalid state or timestep");
    }
    let mut v = vec![0.0; n];
    let mut gw = vec![0.0; n];
    let mut ge = vec![0.0; n];
    let mut rhs = vec![0.0; n];
    for i in 0..n {
        gw[i] = if i == 0 { 0.0 } else { mesh.conductance[i - 1] };
        ge[i] = if i + 1 == n {
            mesh.outer_conductance
        } else {
            mesh.conductance[i]
        };
        v[i] = mesh.cells[i].volume / dt;
        rhs[i] = v[i] * old[i];
    }
    solve_diffusion_system(&v, &gw, &ge, &rhs)
}

pub fn run(cfg: &Config, mesh: &Mesh) -> Result<(TimeSeries, Vec<f64>)> {
    let mut c = initial_concentration(cfg, mesh);
    let initial = inventory(mesh, &c);
    if initial <= 0.0 {
        bail!("initial inventory must be positive");
    }
    let steps = (cfg.time.end_time_s / cfg.time.dt_s).ceil() as usize;
    let dt = cfg.time.end_time_s / steps as f64;
    let mut released = 0.0;
    let mut release_compensation = 0.0;
    let mut rows = Vec::new();

    rows.push((
        0.0,
        StepDiagnostics {
            inventory: initial,
            cumulative_release: 0.0,
            release_fraction: 0.0,
            release_rate: mesh.outer_conductance * c[c.len() - 1],
            balance_residual: 0.0,
            min_concentration: c.iter().copied().fold(f64::INFINITY, f64::min),
        },
    ));

    for step in 1..=steps {
        let next = backward_euler_step(mesh, &c, dt)?;
        let release_rate = mesh.outer_conductance * next[next.len() - 1];
        // Kahan compensated accumulation prevents long-run roundoff in the
        // independently accumulated boundary-release conservation check.
        let increment = dt * release_rate;
        let y = increment - release_compensation;
        let t = released + y;
        release_compensation = (t - released) - y;
        released = t;
        c = next;
        let inv = inventory(mesh, &c);
        let residual = inv + released - initial;
        if step % cfg.time.output_every_steps == 0 || step == steps {
            rows.push((
                step as f64 * dt,
                StepDiagnostics {
                    inventory: inv,
                    cumulative_release: released,
                    release_fraction: released / initial,
                    release_rate,
                    balance_residual: residual,
                    min_concentration: c.iter().copied().fold(f64::INFINITY, f64::min),
                },
            ));
        }
    }
    Ok((rows, c))
}

pub fn explicit_dt_bound(mesh: &Mesh) -> f64 {
    mesh.cells
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            let gw = if i == 0 { 0.0 } else { mesh.conductance[i - 1] };
            let ge = if i + 1 == mesh.cells.len() {
                mesh.outer_conductance
            } else {
                mesh.conductance[i]
            };
            cell.volume / (gw + ge)
        })
        .fold(f64::INFINITY, f64::min)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, InitialConfig, OutputConfig, RegionConfig, TimeConfig};

    fn cfg(cells: usize) -> Config {
        Config {
            case_name: "test".into(),
            species: "synthetic".into(),
            temperature_k: 1000.0,
            regions: (0..5)
                .map(|i| RegionConfig {
                    name: format!("r{i}"),
                    outer_radius_m: (i + 1) as f64 * 1e-4,
                    diffusivity_m2_s: 1e-10,
                    cells,
                })
                .collect(),
            initial: InitialConfig {
                kernel_concentration_mol_m3: 1.0,
            },
            time: TimeConfig {
                dt_s: 0.1,
                end_time_s: 1.0,
                output_every_steps: 1,
            },
            output: OutputConfig {
                directory: "target/test".into(),
            },
        }
    }

    #[test]
    fn interfaces_are_faces() {
        let c = cfg(4);
        let m = build_mesh(&c).unwrap();
        for r in c.regions.iter().take(4) {
            assert!(m
                .cells
                .iter()
                .any(|x| (x.right - r.outer_radius_m).abs() < 1e-15));
        }
    }

    #[test]
    fn internal_conductances_are_positive() {
        let m = build_mesh(&cfg(3)).unwrap();
        assert!(m.conductance.iter().all(|&g| g > 0.0));
        assert!(m.outer_conductance > 0.0);
    }

    #[test]
    fn backward_euler_is_positive_and_conservative() {
        let c = cfg(3);
        let m = build_mesh(&c).unwrap();
        let (rows, _) = run(&c, &m).unwrap();
        assert!(rows.iter().all(|(_, d)| d.min_concentration >= -1e-13));
        let initial = rows[0].1.inventory;
        let max_res = rows
            .iter()
            .map(|(_, d)| d.balance_residual.abs() / initial)
            .fold(0.0, f64::max);
        assert!(max_res < 1e-10);
    }

    /// Dense Gaussian elimination with partial pivoting, test-only reference.
    fn dense_solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Vec<f64> {
        let n = b.len();
        for col in 0..n {
            let piv = (col..n)
                .max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))
                .unwrap();
            a.swap(col, piv);
            b.swap(col, piv);
            let pivot_row = a[col].clone();
            for row in col + 1..n {
                let f = a[row][col] / pivot_row[col];
                for (x, p) in a[row][col..].iter_mut().zip(&pivot_row[col..]) {
                    *x -= f * p;
                }
                b[row] -= f * b[col];
            }
        }
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let s: f64 = (i + 1..n).map(|k| a[i][k] * x[k]).sum();
            x[i] = (b[i] - s) / a[i][i];
        }
        x
    }

    #[test]
    fn backward_euler_step_matches_dense_solve() {
        let m = build_mesh(&cfg(3)).unwrap();
        let n = m.cells.len();
        let dt = 7.0;
        let old: Vec<f64> = (0..n).map(|i| 1.0 + 0.1 * i as f64).collect();
        let mut a = vec![vec![0.0; n]; n];
        for i in 0..n {
            let gw = if i == 0 { 0.0 } else { m.conductance[i - 1] };
            let ge = if i + 1 == n {
                m.outer_conductance
            } else {
                m.conductance[i]
            };
            a[i][i] = m.cells[i].volume / dt + gw + ge;
            if i > 0 {
                a[i][i - 1] = -gw;
            }
            if i + 1 < n {
                a[i][i + 1] = -ge;
            }
        }
        let rhs: Vec<f64> = (0..n).map(|i| m.cells[i].volume * old[i] / dt).collect();
        let want = dense_solve(a, rhs);
        let got = backward_euler_step(&m, &old, dt).unwrap();
        for (g, w) in got.iter().zip(&want) {
            assert!((g - w).abs() <= 1e-12 * w.abs(), "{g} vs {w}");
        }
    }

    #[test]
    fn conservation_holds_at_very_large_fourier_number() {
        // dr = 2e-6 m, D dt / dr^2 = 2.5e5: the textbook Thomas pivot loses ~6 digits here.
        let mut c = cfg(50);
        c.time.dt_s = 1.0e4;
        c.time.end_time_s = 1.0e6;
        let m = build_mesh(&c).unwrap();
        let (rows, _) = run(&c, &m).unwrap();
        let initial = rows[0].1.inventory;
        let worst = rows
            .iter()
            .map(|(_, d)| d.balance_residual.abs() / initial)
            .fold(0.0, f64::max);
        assert!(worst < 1e-12, "relative balance residual {worst:e}");
    }

    #[test]
    fn explicit_bound_is_finite() {
        assert!(explicit_dt_bound(&build_mesh(&cfg(2)).unwrap()).is_finite());
    }
}
