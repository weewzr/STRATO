//! Initial vector, discrete inventory, cumulative release and balance residual.
//!
//! Implements Document 2 equations (F21) to (F23) (initial vector), (F25) (inventory), (F27)
//! and (F28) (cumulative release and release fraction) and (F30) (balance residual);
//! pseudocode routines PS-09, PS-12, PS-13 and PS-14.

use super::grid::{Grid, Region};
use std::f64::consts::PI;

/// Inventory-matched start value of the kernel-surface node as a fraction of `C_init` (F22):
/// `theta = dr_k / (dr_k + dr_b) * (1 - 1 / (3 n_kernel))`, dimensionless, in `(0, 1)`.
pub fn kernel_surface_theta(dr_kernel: f64, dr_buffer: f64, n_kernel: usize) -> f64 {
    dr_kernel / (dr_kernel + dr_buffer) * (1.0 - 1.0 / (3.0 * n_kernel as f64))
}

/// Initial vector (F21) with `theta` of (F22). Returns `(c, theta, M^0)` where `c` has length
/// `N + 1` (mol/m^3) and `M^0` is the discrete inventory (F25) in mol.
pub fn initial_vector(grid: &Grid, c_init: f64) -> (Vec<f64>, f64, f64) {
    let nk = grid.interface_nodes[0];
    let theta = kernel_surface_theta(grid.dr_region[0], grid.dr_region[1], nk);
    let mut c = vec![0.0; grid.n + 1];
    for value in c.iter_mut().take(nk) {
        *value = c_init;
    }
    c[nk] = theta * c_init;
    let m0 = inventory(grid, &c);
    (c, theta, m0)
}

/// Discrete inventory (F25): `M^j = 4 pi sum_{i=1}^{N-1} V_i C_i^j`, in mol. The factor
/// `4 pi` is applied once at the end; the loop runs in increasing `i`.
pub fn inventory(grid: &Grid, c: &[f64]) -> f64 {
    let mut sum = 0.0;
    for i in 1..grid.n {
        sum += grid.v[i] * c[i];
    }
    4.0 * PI * sum
}

/// Inventory in each material, mol (presentation aid for (F25), not used by the balance).
/// The weight of a node is split into its inner and outer half-interval parts, the two terms
/// of `V_i` in (F7); the five parts add up to [`inventory`] to rounding.
pub fn inventory_by_region(grid: &Grid, c: &[f64]) -> [f64; 5] {
    let mut parts = [0.0; 5];
    for i in 1..grid.n {
        let r2 = grid.r[i] * grid.r[i];
        let inner = r2 * grid.dr_inner(i) / 2.0;
        let outer = r2 * grid.dr_outer(i) / 2.0;
        parts[grid.region_of_interval[i - 1].index()] += inner * c[i];
        parts[grid.region_of_interval[i].index()] += outer * c[i];
    }
    for p in parts.iter_mut() {
        *p *= 4.0 * PI;
    }
    parts
}

/// Names of the five regions in the order of [`inventory_by_region`].
pub fn region_names() -> [&'static str; 5] {
    [
        Region::Kernel.name(),
        Region::Buffer.name(),
        Region::Ipyc.name(),
        Region::Sic.name(),
        Region::Opyc.name(),
    ]
}

/// Compensated (Kahan) accumulator for the sum of release increments in (F27). It changes
/// only the rounding of `M_rel^j`, not the identity.
#[derive(Clone, Copy, Debug, Default)]
pub struct Accumulator {
    sum: f64,
    compensation: f64,
}

impl Accumulator {
    /// Empty accumulator, `M_rel^0 = 0`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one increment `dt * Phi^{j+1}` (mol).
    pub fn add(&mut self, x: f64) {
        let y = x - self.compensation;
        let t = self.sum + y;
        self.compensation = (t - self.sum) - y;
        self.sum = t;
    }

    /// Current `M_rel^j` in mol.
    pub fn value(&self) -> f64 {
        self.sum
    }
}

/// Release fraction `F^j = M_rel^j / M^0` (F28), dimensionless.
pub fn release_fraction(released: f64, initial_inventory: f64) -> f64 {
    released / initial_inventory
}

/// Balance residual `eps^j = M^j + M_rel^j - M^0` in mol (F30). It reports; it never
/// corrects anything.
pub fn balance_residual(inventory: f64, released: f64, initial_inventory: f64) -> f64 {
    inventory + released - initial_inventory
}

#[cfg(test)]
mod tests {
    use super::*;

    const RADII: [f64; 5] = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4];

    #[test]
    fn theta_f22() {
        // Hand value: dr_k = 1, dr_b = 1, n = 3: 1/2 * (1 - 1/9).
        assert!((kernel_surface_theta(1.0, 1.0, 3) - 0.5 * (1.0 - 1.0 / 9.0)).abs() < 1e-15);
        for n in [1usize, 2, 5, 40, 1000] {
            for ratio in [1e-3, 0.1, 1.0, 10.0, 1e3] {
                let t = kernel_surface_theta(1.0, ratio, n);
                assert!(t > 0.0 && t < 1.0, "theta {t} n {n} ratio {ratio}");
            }
        }
        // Not 1/2 for unequal spacings.
        assert!((kernel_surface_theta(1.0, 3.0, 10) - 0.5).abs() > 0.1);
    }

    #[test]
    fn theta_solves_f23_and_initial_inventory_is_exact_p10() {
        for counts in [[3, 2, 4, 5, 2], [40; 5], [7, 11, 3, 5, 13]] {
            let g = Grid::from_radii(RADII, counts).unwrap();
            let (c, theta, m0) = initial_vector(&g, 2.5);
            let nk = g.interface_nodes[0];
            let sum: f64 = (1..nk).map(|i| g.v[i]).sum::<f64>() + theta * g.v[nk];
            let exact = RADII[0].powi(3) / 3.0;
            let tol = 4.0 * g.n as f64 * f64::EPSILON / 2.0;
            assert!((sum - exact).abs() <= tol * exact, "F23 {sum} {exact}");
            let m_exact = 4.0 * PI / 3.0 * RADII[0].powi(3) * 2.5;
            assert!((m0 - m_exact).abs() <= tol * m_exact, "P10 {m0} {m_exact}");
            assert_eq!(c[g.n], 0.0);
            assert_eq!(c[nk], theta * 2.5);
            assert_eq!(c[nk - 1], 2.5);
        }
    }

    #[test]
    fn inventory_f25_hand_value_and_region_sum() {
        let g = Grid::from_radii(RADII, [3, 2, 4, 5, 2]).unwrap();
        let c: Vec<f64> = (0..=g.n).map(|i| 1.0 + 0.1 * i as f64).collect();
        let mut c_zero_end = c.clone();
        c_zero_end[g.n] = 0.0;
        let m = inventory(&g, &c_zero_end);
        let hand: f64 = (1..g.n).map(|i| g.v[i] * c_zero_end[i]).sum::<f64>() * 4.0 * PI;
        assert!((m - hand).abs() <= 8.0 * f64::EPSILON * hand);
        let parts = inventory_by_region(&g, &c_zero_end);
        let total: f64 = parts.iter().sum();
        assert!((total - m).abs() <= 8.0 * f64::EPSILON * m);
        // Including the centre and outer terms changes nothing: V_0 = 0 and C_N = 0.
        let full: f64 = 4.0 * PI * (0..=g.n).map(|i| g.v[i] * c_zero_end[i]).sum::<f64>();
        assert!((full - m).abs() <= 8.0 * f64::EPSILON * m);
    }

    #[test]
    fn accumulator_f27() {
        let mut acc = Accumulator::new();
        let mut plain = 0.0;
        for _ in 0..1_000_000 {
            acc.add(0.1);
            plain += 0.1;
        }
        // Exact sum of 1e6 copies of the double 0.1 is 1e6 * 0.1(double) up to one rounding.
        let exact = 1.0e6 * 0.1;
        assert!((acc.value() - exact).abs() < (plain - exact).abs());
        assert!((acc.value() - exact).abs() <= 2.0 * f64::EPSILON * exact);
    }

    #[test]
    fn residual_f30() {
        assert_eq!(balance_residual(3.0, 2.0, 5.0), 0.0);
        let (m, rel, m0) = (7.0, 2.0, 10.0);
        let eps = balance_residual(m, rel, m0);
        let f = release_fraction(rel, m0);
        assert!((eps / m0 - (f - (1.0 - m / m0))).abs() < 1e-15);
    }
}
