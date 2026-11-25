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
        assert!((c.k_outer[1] - expect).abs() <= 64.0 * f64::EPSILON * expect);
        // Series check inside the buffer: sum of reciprocals telescopes to (1/r_a - 1/r_b)/D.
        let (a, b) = (g.interface_nodes[0], g.interface_nodes[1]);
        let recip: f64 = (a..b).map(|i| 1.0 / c.k_outer[i]).sum();
        let exact = (1.0 / g.r[a] - 1.0 / g.r[b]) / D[1];
        assert!((recip - exact).abs() <= 64.0 * f64::EPSILON * exact);
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
