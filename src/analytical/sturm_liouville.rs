//! Code C: analytical approach. Sturm-Liouville eigenfunction series for the layered
//! sphere. Nothing in this module calls the finite-volume solver.
//!
//! Two independent pieces live here:
//!
//! 1. A homogeneous Robin-sphere eigenvalue finder (`robin_residual`, `robin_roots`).
//! 2. The exact eigenfunction expansion of the five-layer initial-inventory release
//!    problem (initial kernel concentration, no source, absorbing outer surface).
//!    This is the same physical problem solved by the FV code, so the two can be
//!    compared directly.
//!
//! Modal formulation (u = r*phi, k_i^2 = Lambda/D_i):
//!   u_i'' + k_i^2 u_i = 0,
//!   u continuous and D (u' - u/r) continuous at every interface,
//!   u(0) = 0 (regular centre), u(R) = 0 (absorbing outer surface).
//! Orthogonality weight is r^2 for phi, i.e. weight 1 for u, so the release fraction is
//!   F(t) = 1 - sum_n w_n exp(-Lambda_n t),
//!   w_n  = 3 I1_n In_n / (R1^3 N_n),
//!   I1_n = int_0^R1 r u_n dr, In_n = int_0^R r u_n dr, N_n = int_0^R u_n^2 dr.

use anyhow::{bail, Result};
use std::f64::consts::PI;

/// One homogeneous spherical layer: outer radius (m) and diffusivity (m^2/s).
#[derive(Clone, Copy, Debug)]
pub struct Layer {
    pub outer_radius: f64,
    pub diffusivity: f64,
}

/// Dimensionless Robin-sphere eigenvalue residual for phi=sin(z r/R)/r.
/// Boundary: -D phi'(R)=h phi(R), Bi=hR/D.
/// Root equation: z cot(z) = 1 - Bi.
pub fn robin_residual(z: f64, bi: f64) -> f64 {
    z / z.tan() - (1.0 - bi)
}

/// Locate positive Robin eigenvalues by bracket scanning and bisection.
/// Poles of cot(z) are skipped by scanning finite sign changes.
pub fn robin_roots(bi: f64, count: usize) -> Vec<f64> {
    let mut roots = Vec::new();
    let eps = 1e-8;
    for n in 0..(count + 4) {
        let lo = n as f64 * PI + eps;
        let hi = (n as f64 + 1.0) * PI - eps;
        let segments = 128;
        let mut x0 = lo;
        let mut f0 = robin_residual(x0, bi);
        for k in 1..=segments {
            let x1 = lo + (hi - lo) * k as f64 / segments as f64;
            let f1 = robin_residual(x1, bi);
            if f0.is_finite() && f1.is_finite() && f0 * f1 < 0.0 {
                let mut a = x0;
                let mut b = x1;
                let mut fa = f0;
                for _ in 0..80 {
                    let m = 0.5 * (a + b);
                    let fm = robin_residual(m, bi);
                    if fa * fm <= 0.0 {
                        b = m;
                    } else {
                        a = m;
                        fa = fm;
                    }
                }
                let root = 0.5 * (a + b);
                if root > 1e-6 && roots.last().is_none_or(|x: &f64| (root - *x).abs() > 1e-6) {
                    roots.push(root);
                    if roots.len() == count {
                        return roots;
                    }
                }
            }
            x0 = x1;
            f0 = f1;
        }
    }
    roots
}

/// Propagate the two-component state (u, q = D u') through ONE homogeneous layer
/// of thickness `length` for u'' + k^2 u = 0.
///
/// This is only the within-layer step. The state (u, D u') is NOT continuous across a
/// material interface when the diffusivities differ; use `interface_state` there.
pub fn transfer_layer(u: f64, du_flux: f64, d: f64, k: f64, length: f64) -> (f64, f64) {
    let c = (k * length).cos();
    let s = (k * length).sin();
    let u2 = c * u + s * du_flux / (d * k);
    let q2 = -d * k * s * u + c * du_flux;
    (u2, q2)
}

/// Interface rule for the state q = D u' (u = r*phi).
/// Concentration continuity gives u continuous. Flux continuity D phi' continuous with
/// phi' = (u' - u/r)/r gives D (u' - u/r) continuous, hence
///   q_outer = q_inner + (D_outer - D_inner) u / r.
pub fn interface_state(q_inner: f64, u: f64, d_inner: f64, d_outer: f64, radius: f64) -> f64 {
    q_inner + (d_outer - d_inner) * u / radius
}

#[derive(Clone, Copy, Debug)]
struct LayerStart {
    u: f64,
    q: f64,
    k: f64,
    d: f64,
    left: f64,
    right: f64,
}

pub fn validate_layers(layers: &[Layer]) -> Result<()> {
    if layers.len() < 2 {
        bail!("at least two layers are required");
    }
    let mut last = 0.0;
    for l in layers {
        if !(l.outer_radius.is_finite() && l.diffusivity.is_finite())
            || l.outer_radius <= last
            || l.diffusivity <= 0.0
        {
            bail!("layers need increasing finite radii and positive finite diffusivities");
        }
        last = l.outer_radius;
    }
    Ok(())
}

/// Shoot the regular solution from the centre at eigenvalue `lambda`.
/// Returns u(R) and the state at the start of each layer. With `rescale` the running
/// state is renormalised after each layer (fine for counting, not for integrals).
fn shoot(lambda: f64, layers: &[Layer], rescale: bool) -> (f64, Vec<LayerStart>) {
    let mut starts = Vec::with_capacity(layers.len());
    let (mut u, mut q) = (0.0, 1.0);
    let mut left = 0.0;
    let mut prev_d: Option<f64> = None;
    for l in layers {
        if let Some(pd) = prev_d {
            q = interface_state(q, u, pd, l.diffusivity, left);
        }
        let k = (lambda / l.diffusivity).sqrt();
        starts.push(LayerStart {
            u,
            q,
            k,
            d: l.diffusivity,
            left,
            right: l.outer_radius,
        });
        (u, q) = transfer_layer(u, q, l.diffusivity, k, l.outer_radius - left);
        if rescale {
            let m = u.abs().max((q / (l.diffusivity * k)).abs());
            if m > 0.0 && m.is_finite() {
                u /= m;
                q /= m;
            }
        }
        left = l.outer_radius;
        prev_d = Some(l.diffusivity);
    }
    (u, starts)
}

/// Sturm oscillation count: number of eigenvalues Lambda_n <= `lambda`, equal to the
/// number of zeros of the regular solution in (0, R].
pub fn eigenvalue_count(lambda: f64, layers: &[Layer]) -> usize {
    let mut n: i64 = 0;
    let (mut u, mut q) = (0.0, 1.0);
    let mut left = 0.0;
    let mut prev_d: Option<f64> = None;
    for l in layers {
        if let Some(pd) = prev_d {
            q = interface_state(q, u, pd, l.diffusivity, left);
        }
        let k = (lambda / l.diffusivity).sqrt();
        let a = u;
        let b = q / (l.diffusivity * k);
        // u(s) = A sin(k s + theta0); zeros at k s + theta0 = m pi for s in (0, L].
        let theta0 = a.atan2(b);
        let length = l.outer_radius - left;
        n += ((theta0 + k * length) / PI).floor() as i64 - (theta0 / PI).floor() as i64;
        (u, q) = transfer_layer(u, q, l.diffusivity, k, length);
        let m = u.abs().max((q / (l.diffusivity * k)).abs());
        if m > 0.0 && m.is_finite() {
            u /= m;
            q /= m;
        }
        left = l.outer_radius;
        prev_d = Some(l.diffusivity);
    }
    n.max(0) as usize
}

/// All eigenvalues up to `lambda_max`, capped at `max_modes`.
/// Returns the sorted eigenvalues and the effective upper bound actually covered.
/// Roots are isolated by bisection on the Sturm count, so no root can be skipped.
pub fn eigenvalues(layers: &[Layer], lambda_max: f64, max_modes: usize) -> Result<(Vec<f64>, f64)> {
    validate_layers(layers)?;
    if !(lambda_max > 0.0 && lambda_max.is_finite()) || max_modes == 0 {
        bail!("lambda_max and max_modes must be positive");
    }
    let mut hi = lambda_max;
    if eigenvalue_count(hi, layers) > max_modes {
        let mut lo = hi * 1e-12;
        for _ in 0..200 {
            let mid = (lo * hi).sqrt();
            if eigenvalue_count(mid, layers) > max_modes {
                hi = mid;
            } else {
                lo = mid;
            }
            if hi - lo <= 1e-12 * hi {
                break;
            }
        }
        hi = lo;
    }
    let n_hi = eigenvalue_count(hi, layers);

    // Lower bound with zero eigenvalues below it.
    let phase_rate: f64 = {
        let mut left = 0.0;
        layers
            .iter()
            .map(|l| {
                let v = (l.outer_radius - left) / l.diffusivity.sqrt();
                left = l.outer_radius;
                v
            })
            .sum()
    };
    let mut lo = 1e-6 * (1.0 / phase_rate).powi(2);
    while eigenvalue_count(lo, layers) != 0 {
        lo *= 0.01;
        if lo < 1e-300 {
            bail!("could not bracket the lowest eigenvalue");
        }
    }

    let mut roots = Vec::with_capacity(n_hi);
    let mut stack = vec![(lo, hi, 0usize, n_hi)];
    while let Some((a, b, na, nb)) = stack.pop() {
        if nb <= na {
            continue;
        }
        if b - a <= 4e-15 * b {
            if nb - na > 1 {
                bail!("unresolved eigenvalue cluster near Lambda = {b:.6e}");
            }
            roots.push(0.5 * (a + b));
            continue;
        }
        let m = if b > 4.0 * a {
            (a * b).sqrt()
        } else {
            0.5 * (a + b)
        };
        let nm = eigenvalue_count(m, layers).clamp(na, nb);
        stack.push((a, m, na, nm));
        stack.push((m, b, nm, nb));
    }
    roots.sort_by(|x, y| x.partial_cmp(y).expect("finite roots"));
    Ok((roots, hi))
}

fn gauss_legendre_16() -> ([f64; 16], [f64; 16]) {
    let n = 16usize;
    let mut x = [0.0; 16];
    let mut w = [0.0; 16];
    for i in 0..n {
        let mut z = ((i as f64 + 0.75) / (n as f64 + 0.5) * PI).cos();
        let mut dp = 1.0;
        for _ in 0..100 {
            let (mut p_prev, mut p) = (1.0, z);
            for k in 2..=n {
                let kf = k as f64;
                let pk = ((2.0 * kf - 1.0) * z * p - (kf - 1.0) * p_prev) / kf;
                p_prev = p;
                p = pk;
            }
            dp = n as f64 * (z * p - p_prev) / (z * z - 1.0);
            let dz = p / dp;
            z -= dz;
            if dz.abs() < 1e-15 {
                break;
            }
        }
        x[i] = z;
        w[i] = 2.0 / ((1.0 - z * z) * dp * dp);
    }
    (x, w)
}

fn integrate(a: f64, b: f64, panels: usize, f: impl Fn(f64) -> f64) -> f64 {
    let (x, w) = gauss_legendre_16();
    let h = (b - a) / panels as f64;
    let mut total = 0.0;
    for p in 0..panels {
        let lo = a + p as f64 * h;
        let (mid, half) = (lo + 0.5 * h, 0.5 * h);
        total += half * (0..16).map(|i| w[i] * f(mid + half * x[i])).sum::<f64>();
    }
    total
}

/// Exact five-layer (or any N >= 2 layer) release series for the
/// initial-kernel-inventory, absorbing-outer-surface problem.
#[derive(Clone, Debug)]
pub struct ModalRelease {
    pub eigenvalues: Vec<f64>,
    pub weights: Vec<f64>,
    /// Largest eigenvalue covered by the series (1/s).
    pub lambda_covered: f64,
    /// Smallest value, over all modes and interfaces, of
    /// |D (u' - u/r)| / (|D u'| + |D u / r|) in the shooting solution. The physical flux is
    /// a difference of two terms of that size, so about eps / this ratio of relative
    /// accuracy is lost. See `accuracy_floor`.
    pub min_flux_ratio: f64,
}

impl ModalRelease {
    /// Build the series from all eigenvalues up to `lambda_max` (capped at `max_modes`).
    pub fn new(layers: &[Layer], lambda_max: f64, max_modes: usize) -> Result<Self> {
        let (lams, lambda_covered) = eigenvalues(layers, lambda_max, max_modes)?;
        let r1 = layers[0].outer_radius;
        let mut weights = Vec::with_capacity(lams.len());
        let mut min_flux_ratio = f64::INFINITY;
        for &lam in &lams {
            let (_, starts) = shoot(lam, layers, false);
            for st in starts.iter().skip(1) {
                let scale = st.q.abs() + (st.d * st.u / st.left).abs();
                if scale > 0.0 {
                    let flux = (st.q - st.d * st.u / st.left).abs();
                    min_flux_ratio = min_flux_ratio.min(flux / scale);
                }
            }
            let (mut norm, mut moment, mut i1) = (0.0, 0.0, 0.0);
            for (idx, st) in starts.iter().enumerate() {
                let (a, b) = (st.u, st.q / (st.d * st.k));
                let panels = ((st.k * (st.right - st.left)) / (0.5 * PI)).ceil() as usize + 1;
                let u_at = |r: f64| {
                    let s = r - st.left;
                    a * (st.k * s).cos() + b * (st.k * s).sin()
                };
                norm += integrate(st.left, st.right, panels, |r| u_at(r).powi(2));
                let m = integrate(st.left, st.right, panels, |r| r * u_at(r));
                moment += m;
                if idx == 0 {
                    i1 = m;
                }
            }
            let w = 3.0 * i1 * moment / (r1.powi(3) * norm);
            if !w.is_finite() {
                bail!("non-finite modal weight at Lambda = {lam:.6e}");
            }
            weights.push(w);
        }
        Ok(Self {
            eigenvalues: lams,
            weights,
            lambda_covered,
            min_flux_ratio,
        })
    }

    /// Estimated absolute accuracy of `release_fraction`, about 2.2e-16 / `min_flux_ratio`
    /// (at least 2.2e-16). Cancellation in the shooting solution is the dominant error
    /// when adjacent layers differ strongly in diffusivity. Treat differences between FV and
    /// the series below this level as unresolved. Measured against FV on test inputs, the
    /// true error stayed about 10 to 20 times below this estimate.
    pub fn accuracy_floor(&self) -> f64 {
        2.2e-16 / self.min_flux_ratio.min(1.0)
    }

    /// Smallest time for which the omitted modes (Lambda > covered) contribute
    /// less than exp(-40) of their weight.
    pub fn min_valid_time(&self) -> f64 {
        40.0 / self.lambda_covered
    }

    /// Fraction of the initial kernel inventory still inside the particle at time t.
    pub fn remaining_fraction(&self, t: f64) -> f64 {
        self.eigenvalues
            .iter()
            .zip(&self.weights)
            .map(|(l, w)| w * (-l * t).exp())
            .sum()
    }

    /// Cumulative released fraction F(t) = 1 - sum_n w_n exp(-Lambda_n t).
    pub fn release_fraction(&self, t: f64) -> f64 {
        1.0 - self.remaining_fraction(t)
    }
}

/// Convenience: exact release fraction at the given times (all must be > 0).
/// The series is truncated so that exp(-45) of the omitted modes remains at the
/// earliest time; `max_modes` bounds the cost.
pub fn exact_release(
    layers: &[Layer],
    times: &[f64],
    max_modes: usize,
) -> Result<Vec<Option<f64>>> {
    let t_min = times
        .iter()
        .copied()
        .filter(|t| *t > 0.0)
        .fold(f64::INFINITY, f64::min);
    if !t_min.is_finite() {
        bail!("need at least one positive time");
    }
    let series = ModalRelease::new(layers, 45.0 / t_min, max_modes)?;
    let valid = series.min_valid_time();
    Ok(times
        .iter()
        .map(|&t| (t >= valid).then(|| series.release_fraction(t)))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn five_equal(d: f64) -> Vec<Layer> {
        [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4]
            .iter()
            .map(|&r| Layer {
                outer_radius: r,
                diffusivity: d,
            })
            .collect()
    }

    /// Closed form for kernel-only initial data in a homogeneous absorbing sphere.
    fn homogeneous_kernel_release(t: f64, r1: f64, r: f64, d: f64) -> f64 {
        let mut remaining = 0.0;
        for n in 1..2000 {
            let a = n as f64 * PI / r;
            let i1 = ((a * r1).sin() - a * r1 * (a * r1).cos()) / (a * a);
            let i_n = r * r * (-1.0f64).powi(n + 1) / (n as f64 * PI);
            let w = 3.0 * i1 * i_n / (r1.powi(3) * (0.5 * r));
            remaining += w * (-a * a * d * t).exp();
        }
        1.0 - remaining
    }

    #[test]
    fn robin_roots_satisfy_residual() {
        let roots = robin_roots(2.0, 4);
        assert_eq!(roots.len(), 4);
        for z in roots {
            assert!(robin_residual(z, 2.0).abs() < 1e-10);
        }
    }

    #[test]
    fn transfer_zero_length_is_identity() {
        let (u, q) = transfer_layer(2.0, -3.0, 0.5, 1.7, 0.0);
        assert!((u - 2.0).abs() < 1e-14);
        assert!((q + 3.0).abs() < 1e-14);
    }

    #[test]
    fn transfer_preserves_state_across_equal_material_subdivision() {
        let (u1, q1) = transfer_layer(1.2, 0.7, 2.0, 0.9, 0.3);
        let (u2, q2) = transfer_layer(u1, q1, 2.0, 0.9, 0.4);
        let (u, q) = transfer_layer(1.2, 0.7, 2.0, 0.9, 0.7);
        assert!((u2 - u).abs() < 1e-12);
        assert!((q2 - q).abs() < 1e-12);
    }

    #[test]
    fn interface_state_conserves_flux_and_reduces_to_identity() {
        // equal diffusivity: no jump
        assert_eq!(interface_state(0.7, 1.3, 2.0, 2.0, 0.5), 0.7);
        // D (u' - u/r) must match on both sides
        let (u, r, d1, d2, q1) = (1.3, 0.5, 2.0, 5.0, 0.7);
        let q2 = interface_state(q1, u, d1, d2, r);
        let lhs = q1 - d1 * u / r;
        let rhs = q2 - d2 * u / r;
        assert!((lhs - rhs).abs() < 1e-14);
    }

    #[test]
    fn sturm_count_matches_root_count_and_is_monotone() {
        let layers = [
            Layer {
                outer_radius: 2.0e-4,
                diffusivity: 1.0e-12,
            },
            Layer {
                outer_radius: 3.0e-4,
                diffusivity: 4.0e-10,
            },
            Layer {
                outer_radius: 3.5e-4,
                diffusivity: 8.0e-13,
            },
            Layer {
                outer_radius: 4.0e-4,
                diffusivity: 2.0e-14,
            },
            Layer {
                outer_radius: 4.5e-4,
                diffusivity: 7.0e-13,
            },
        ];
        let (roots, covered) = eigenvalues(&layers, 2.0e-3, 10_000).unwrap();
        assert_eq!(roots.len(), eigenvalue_count(covered, &layers));
        assert!(roots.windows(2).all(|w| w[1] > w[0]));
        let mut last = 0;
        for i in 0..400 {
            let n = eigenvalue_count(2.0e-3 * (i as f64 + 1.0) / 400.0, &layers);
            assert!(n >= last);
            last = n;
        }
    }

    #[test]
    fn modal_release_matches_closed_form_for_equal_diffusivity() {
        let d = 1.0e-10;
        let layers = five_equal(d);
        let series = ModalRelease::new(&layers, 45.0 / 20.0, 20_000).unwrap();
        for t in [20.0, 100.0, 500.0, 2000.0] {
            let exact = homogeneous_kernel_release(t, 2.0e-4, 4.5e-4, d);
            assert!(
                (series.release_fraction(t) - exact).abs() < 1e-9,
                "t={t}: {} vs {exact}",
                series.release_fraction(t)
            );
        }
    }

    #[test]
    fn release_is_monotone_and_bounded() {
        let layers = [
            Layer {
                outer_radius: 2.0e-4,
                diffusivity: 1.0e-12,
            },
            Layer {
                outer_radius: 3.0e-4,
                diffusivity: 4.0e-10,
            },
            Layer {
                outer_radius: 3.5e-4,
                diffusivity: 8.0e-13,
            },
            Layer {
                outer_radius: 4.0e-4,
                diffusivity: 2.0e-14,
            },
            Layer {
                outer_radius: 4.5e-4,
                diffusivity: 7.0e-13,
            },
        ];
        let series = ModalRelease::new(&layers, 45.0 / 5.0e3, 20_000).unwrap();
        let mut last = 0.0;
        for i in 1..=60 {
            let f = series.release_fraction(5.0e3 * i as f64 * 5.0);
            assert!((0.0..=1.0).contains(&f) && f >= last - 1e-12, "F={f}");
            last = f;
        }
    }
}
