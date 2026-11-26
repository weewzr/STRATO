//! Interval conductances and mesh Fourier numbers of the BTCS solver.
//!
//! Implements Document 2 equations (F6) and (F8), pseudocode routine PS-04.

// The loops mirror the index form of the equations in Document 2 (node `i`, material `k`).
#![allow(clippy::needless_range_loop)]

use super::grid::Grid;
use anyhow::{bail, Result};

/// Coefficients that depend on the grid, the diffusivities and the time step.
#[derive(Clone, Debug)]
pub struct Coefficients {
    /// `D_material` in m^2/s.
    pub d: [f64; 5],
    /// `Fo_material` (F8), dimensionless.
    pub fo: [f64; 5],
    /// `K_{i,outer}` (F6) in m^3/s, length `N`, `k_outer[0] = 0.0` exactly.
    /// `K_{i,inner}` is `k_outer[i - 1]`.
    pub k_outer: Vec<f64>,
}

impl Coefficients {
    /// Computes `Fo_material = D dt / (dr)^2` (F8) and the interval conductance
    /// `K_{i,outer} = D_{i,outer} r_i r_{i+1} / Delta r_{i,outer}` (F6) with the stored
    /// difference of radii (F5). `K_{0,outer}` is the literal zero because `r_0 = 0`.
    ///
    /// # Errors
    ///
    /// A non-finite or non-positive conductance, Fourier number or time step.
    pub fn new(grid: &Grid, d: [f64; 5], dt: f64) -> Result<Coefficients> {
        if !(dt.is_finite() && dt > 0.0) {
            bail!("time step must be finite and positive");
        }
        let mut fo = [0.0; 5];
        for k in 0..5 {
            fo[k] = d[k] * dt / (grid.dr_region[k] * grid.dr_region[k]);
            if !(fo[k].is_finite() && fo[k] > 0.0) {
                bail!("mesh Fourier number of material {k} is not finite and positive");
            }
        }
        let mut k_outer = vec![0.0; grid.n];
        for i in 1..grid.n {
            let dr = grid.dr_outer(i);
            k_outer[i] = d[grid.region_of_interval[i].index()] * grid.r[i] * grid.r[i + 1] / dr;
            if !(k_outer[i].is_finite() && k_outer[i] > 0.0) {
                bail!("interval conductance {i} is not finite and positive");
            }
        }
        Ok(Coefficients { d, fo, k_outer })
    }

    /// `K_{N-1,outer}`: the one stored value used by row `N-1` and by the release (F26).
    pub fn k_last(&self) -> f64 {
        self.k_outer[self.k_outer.len() - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RADII: [f64; 5] = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4];
    const D: [f64; 5] = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13];

    #[test]
    fn conductance_f6() {
        let g = Grid::from_radii(RADII, [3, 2, 4, 5, 2]).unwrap();
        let c = Coefficients::new(&g, D, 200.0).unwrap();
        assert_eq!(c.k_outer[0].to_bits(), 0.0f64.to_bits());
        // Hand value, kernel interval 1: D r_1 r_2 / dr with r_1 = R1/3, r_2 = 2 R1/3.
        let dr = RADII[0] / 3.0;
        let expect = D[0] * dr * (2.0 * dr) / dr;
        // No tolerance: the stored nodes are r_1 = fl(1 * dr) = dr and r_2 = fl(2 * dr) = 2 dr
        // (doubling is exact), so Delta r_{1,outer} = 2 dr - dr = dr exactly, and (F6) executes
        // the very operations of `expect`, in the same order: bitwise equality.
        assert_eq!(c.k_outer[1].to_bits(), expect.to_bits());
        // Series check inside the buffer. In exact arithmetic, with the stored radii and the
        // stored differences dr_i = r_{i+1} - r_i, 1/K_i = dr_i / (D r_i r_{i+1}) =
        // (1/r_i - 1/r_{i+1}) / D, so the sum over the buffer telescopes to
        // (r_b - r_a) / (D r_a r_b). Rounding (u = 2^-53, gamma_k = k u / (1 - k u)):
        // the buffer radii satisfy r_b <= 2 r_a (Sterbenz), so every stored difference and
        // r_b - r_a is exact. K_i has 3 roundings (two products, one quotient), 1/K_i one
        // more: gamma_4 per term; recursive summation of n positive terms adds gamma_{n-1}:
        // |recip - S| <= gamma_{n+3} S. The reference ((r_b - r_a) / (r_a r_b)) / D has 3
        // roundings: |exact - S| <= gamma_3 S. Hence
        // |recip - exact| <= (gamma_{n+3} + gamma_3) S, and S <= exact / (1 - gamma_3).
        let (a, b) = (g.interface_nodes[0], g.interface_nodes[1]);
        assert!(g.r[b] <= 2.0 * g.r[a], "Sterbenz precondition");
        let recip: f64 = (a..b).map(|i| 1.0 / c.k_outer[i]).sum();
        let exact = ((g.r[b] - g.r[a]) / (g.r[a] * g.r[b])) / D[1];
        let u = f64::EPSILON / 2.0;
        let gamma = |k: f64| k * u / (1.0 - k * u);
        let n_terms = (b - a) as f64;
        let tol = (gamma(n_terms + 3.0) + gamma(3.0)) / (1.0 - gamma(3.0)) * exact;
        assert!(
            (recip - exact).abs() <= tol,
            "recip {recip} exact {exact} tol {tol}"
        );
    }

    #[test]
    fn fourier_number_f8() {
        let g = Grid::from_radii(RADII, [3, 2, 4, 5, 2]).unwrap();
        let c = Coefficients::new(&g, D, 200.0).unwrap();
        for k in 0..5 {
            let expect = D[k] * 200.0 / (g.dr_region[k] * g.dr_region[k]);
            assert_eq!(c.fo[k], expect);
        }
    }

    #[test]
    fn rejects_bad_time_step() {
        let g = Grid::from_radii(RADII, [3, 2, 4, 5, 2]).unwrap();
        assert!(Coefficients::new(&g, D, 0.0).is_err());
        assert!(Coefficients::new(&g, D, f64::NAN).is_err());
    }
}
