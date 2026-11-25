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

    /// Relative bound for `sum_{i=1}^{n-1} V_i + theta V_n` (F23) and for the discrete
    /// initial inventory (P10), derived in `theta_solves_f23_and_initial_inventory_is_exact_p10`.
    /// `extra` is the number of additional roundings per term (0 for F23, 1 for P10).
    fn f23_relative_bound(nk: usize, extra: f64) -> f64 {
        let u = f64::EPSILON / 2.0;
        let gamma = |k: f64| k * u / (1.0 - k * u);
        let n = nk as f64;
        let g_t = gamma(3.0 * n + 14.0 + extra);
        gamma(n - 1.0) * (1.0 + g_t)
            + gamma(2.0 * n + 6.0 + extra)
            + g_t * (3.0 * n - 1.0) / (2.0 * n * n)
    }

    #[test]
    fn theta_f22() {
        // Hand value: dr_k = 1, dr_b = 1, n = 3: 1/2 * (1 - 1/9). No tolerance: (F22) executes
        // 1/(1+1) = 0.5 exactly, 3 * 3 = 9 exactly, then the same rounded 1/9 and 1 - 1/9 as
        // the hand expression: bitwise equal.
        assert_eq!(
            kernel_surface_theta(1.0, 1.0, 3).to_bits(),
            (0.5f64 * (1.0 - 1.0 / 9.0)).to_bits()
        );
        for n in [1usize, 2, 5, 40, 1000] {
            for ratio in [1e-3, 0.1, 1.0, 10.0, 1e3] {
                let t = kernel_surface_theta(1.0, ratio, n);
                assert!(t > 0.0 && t < 1.0, "theta {t} n {n} ratio {ratio}");
            }
        }
        // Not 1/2 for unequal spacings: (F22) with dr_k = 1, dr_b = 3, n = 10 is
        // 1/4 * (1 - 1/30) (1/4 and 3 * 10 exact, same rounded 1/30), which is below 1/2.
        let t = kernel_surface_theta(1.0, 3.0, 10);
        assert_eq!(t.to_bits(), (0.25f64 * (1.0 - 1.0 / 30.0)).to_bits());
        assert!(t < 0.5);
    }

    #[test]
    fn theta_solves_f23_and_initial_inventory_is_exact_p10() {
        // Derivation of the tolerances (u = 2^-53, gamma_k = k u / (1 - k u), n = n_kernel,
        // h = R_1 / n, h_b = buffer spacing; all "ideal" quantities are real numbers built
        // from the stored doubles R_1 and the stored spacings).
        // Stored nodes: r_i = fl(i dr) with dr = fl(R_1/n): r_i = i h (1 + eps_i), |eps_i| <= gamma_2,
        // and r_n = R_1 exactly. Ideal weights W_i = (i h)^2 h (1 <= i <= n-1).
        //  (a) V_i = fl(fl(fl(r_i r_i) fl(a + b)) / 2), a, b the stored differences. fl(r r): the
        //      two radius errors and one rounding give gamma_5; r_{i+1} - r_{i-1} = 2h (1 + eta_i)
        //      with |eta_i| <= i gamma_2 (the stored differences amplify the radius errors by
        //      i), two subtractions and one addition add gamma_2, the product one more:
        //      |V_i - W_i| <= gamma_{2i+8} W_i.
        //  (b) Summation of n terms (n-1 additions, positive terms): gamma_{n-1}.
        //  (c) The kernel-surface term T = theta V_n. Ideal T = R_1^3 (3n - 1)/(6 n^2), which is
        //      a fraction (3n - 1)/(2 n^2) <= 3/(2n) of R_1^3/3 (this is (F23) solved for theta).
        //      theta has 5 roundings plus gamma_2 from the stored spacings: gamma_7; V_n has
        //      R^2 (1), a = h(1 + alpha), |alpha| <= (n-1) gamma_2 + u, b = h_b + O(u (R_1 + h_b))
        //      (the node after the interface is rounded at the size of R_1, which costs at most
        //      u R_1 / h_b relative to h_b, i.e. weight u n relative to a + b), giving
        //      (3n + 4) u and with the two remaining roundings gamma_{3n+6}; the product theta V_n
        //      one more: |T - T_ideal| <= gamma_{3n+14} T_ideal.
        //  (d) The reference R_1 R_1 R_1 / 3 has 3 roundings: gamma_3.
        //  Sum of the ideal terms: sum_{i<n} W_i + T_ideal = R_1^3/3, and sum_{i<n} W_i <= R_1^3/3,
        //  so   |sum - R_1^3/3| <= [gamma_{n-1}(1 + gamma_{3n+14}) + gamma_{2n+6}
        //       + gamma_{3n+14} (3n - 1)/(2 n^2)] R_1^3/3,   see `f23_relative_bound`.
        // For P10 every term carries one more rounding (the product with C_init, and theta C_init
        // before V_n C), the weights are multiplied by 4 pi (1 rounding; 4 * PI is exact) and the
        // reference (4 PI / 3) (R R R) C_init has 5 roundings.
        let u = f64::EPSILON / 2.0;
        let gamma = |k: f64| k * u / (1.0 - k * u);
        for counts in [[3, 2, 4, 5, 2], [40; 5], [7, 11, 3, 5, 13]] {
            let g = Grid::from_radii(RADII, counts).unwrap();
            let (c, theta, m0) = initial_vector(&g, 2.5);
            let nk = g.interface_nodes[0];
            let sum: f64 = (1..nk).map(|i| g.v[i]).sum::<f64>() + theta * g.v[nk];
            let exact = RADII[0] * RADII[0] * RADII[0] / 3.0;
            let tol = (f23_relative_bound(nk, 0.0) + gamma(3.0)) / (1.0 - gamma(3.0));
            assert!((sum - exact).abs() <= tol * exact, "F23 {sum} {exact}");
            let m_exact = 4.0 * PI / 3.0 * (RADII[0] * RADII[0] * RADII[0]) * 2.5;
            let tol10 =
                (f23_relative_bound(nk, 1.0) + gamma(1.0) + gamma(5.0)) / (1.0 - gamma(5.0));
            assert!(
                (m0 - m_exact).abs() <= tol10 * m_exact,
                "P10 {m0} {m_exact}"
            );
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
        // No tolerance: the loop and the iterator sum perform the same additions in the same
        // order, and (sum * 4) * PI and (4 * PI) * sum are the same correctly rounded product
        // of the exact reals sum * 4 PI (multiplication by 4 is exact).
        assert_eq!(m.to_bits(), hand.to_bits());
        // Including the centre and outer terms changes nothing: V_0 = 0 and V_N = 0 (the
        // additional terms are exact zeros added to the running sum), so also bitwise equal.
        let full: f64 = 4.0 * PI * (0..=g.n).map(|i| g.v[i] * c_zero_end[i]).sum::<f64>();
        assert_eq!(full.to_bits(), m.to_bits());
    }

    #[test]
    fn region_split_of_inventory_f25() {
        // Dyadic grid: radii k * 2^-12 with spacings 1, 2, 1, 4, 2 (units 2^-12), so every
        // radius, difference, r^2 dr / 2 and product with the integer concentrations below is
        // exactly representable; the per-region sums before the factor 4 pi are then exact,
        // and the 4 pi factor is applied once per region.
        let s = 2f64.powi(-12);
        let radii = [4.0 * s, 8.0 * s, 12.0 * s, 16.0 * s, 20.0 * s];
        let counts = [4usize, 2, 4, 1, 2];
        let g = Grid::from_radii(radii, counts).unwrap();
        let c: Vec<f64> = (0..=g.n).map(|i| 1.0 + i as f64).collect();
        let mut c_zero_end = c;
        c_zero_end[g.n] = 0.0;
        // Exact per-region sums by direct evaluation (all operations exact here).
        let mut exact = [0.0f64; 5];
        for i in 1..g.n {
            let r2 = g.r[i] * g.r[i];
            exact[g.region_of_interval[i - 1].index()] += r2 * g.dr_inner(i) / 2.0 * c_zero_end[i];
            exact[g.region_of_interval[i].index()] += r2 * g.dr_outer(i) / 2.0 * c_zero_end[i];
        }
        let parts = inventory_by_region(&g, &c_zero_end);
        for k in 0..5 {
            // One rounding of the product with 4 pi, nothing else.
            assert_eq!(
                parts[k].to_bits(),
                (exact[k] * (4.0 * PI)).to_bits(),
                "region {k}"
            );
        }
        // The total of the parts against the inventory: the inventory is (exact sum) * 4 pi,
        // one rounding (u); each part has one rounding and the sum of the five positive parts
        // 4 additions (gamma_4): |total - m| <= (gamma_5 + u) * 4 pi * S and
        // S 4 pi <= m / (1 - u).
        let u = f64::EPSILON / 2.0;
        let gamma = |k: f64| k * u / (1.0 - k * u);
        let m = inventory(&g, &c_zero_end);
        let total: f64 = parts.iter().sum();
        let tol = (gamma(5.0) + gamma(1.0)) / (1.0 - gamma(1.0)) * m;
        assert!((total - m).abs() <= tol, "total {total} inventory {m}");
    }

    #[test]
    fn accumulator_f27() {
        let mut acc = Accumulator::new();
        let mut plain = 0.0;
        for _ in 0..1_000_000 {
            acc.add(0.1);
            plain += 0.1;
        }
        // The exact sum of 1e6 copies of the double 0.1 is 1e6 * 0.1(double) as a real number;
        // the product 1.0e6 * 0.1 below is that number rounded once. The compensated sum is
        // closer to it than the plain sum (no tolerance). The quantitative accuracy of the
        // compensated sum, |S - sum x_j| <= (2u + O(N u^2)) sum |x_j| (Goldberg, "What Every
        // Computer Scientist Should Know About Floating-Point Arithmetic", Theorem 8), enters
        // the balance bound of `tests/btcs_verification.rs`, where it is used with an explicit
        // statement of what is assumed about the O(N u^2) term.
        let exact = 1.0e6 * 0.1;
        assert!((acc.value() - exact).abs() < (plain - exact).abs());
    }

    #[test]
    fn residual_f30() {
        assert_eq!(balance_residual(3.0, 2.0, 5.0), 0.0);
        let (m, rel, m0) = (7.0, 2.0, 10.0);
        let eps = balance_residual(m, rel, m0);
        let f = release_fraction(rel, m0);
        // eps = 7 + 2 - 10 = -1 exactly, so eps / m0 = fl(-1/10). The right side is
        // fl(f - fl(1 - m/m0)) with f = fl(2/10), m/m0 = fl(7/10). Both subtractions are exact
        // (Sterbenz: 0.7 >= 1/2 and 0.2 >= 0.3/2). The mathematical values are equal
        // (-0.1 = 0.2 - 0.3), so the difference is the sum of the three representation errors,
        // each at most u times the value: tolerance u (0.1 + 0.2 + 0.7).
        let u = f64::EPSILON / 2.0;
        assert!((eps / m0 - (f - (1.0 - m / m0))).abs() <= u * (0.1 + 0.2 + 0.7));
    }
}
