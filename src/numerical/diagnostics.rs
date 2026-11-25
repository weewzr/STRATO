//! Output rows and verification diagnostics of the BTCS solver.
//!
//! Implements pseudocode routine PS-17 and the output row of PS-16. The checks report
//! measured values and booleans; they never alter a result and carry no default tolerance.
//! Properties P1, P3, P4, P5, P9, P10 and P18 of Document 2 Section 8 are the targets.

use super::btcs::Tridiagonal;
use anyhow::{bail, Result};

/// One output row (mol, mol/s, mol/m^3 as named).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepDiagnostics {
    /// `M^j` (F25), mol.
    pub inventory: f64,
    /// `M_rel^j` (F27), mol.
    pub cumulative_release: f64,
    /// `F^j` (F28).
    pub release_fraction: f64,
    /// `Phi^j` (F26), mol/s; `None` at `j = 0` where it is not defined (Document 2 7.2).
    pub release_rate: Option<f64>,
    /// `eps^j` (F30), mol.
    pub balance_residual: f64,
    /// Derived a-posteriori rounding bound on `|eps^j|`, mol (see [`BalanceBound`]). The
    /// observed residual is always reported beside it (coordinator decision 6).
    pub balance_residual_bound: f64,
    /// Smallest nodal concentration, mol/m^3.
    pub min_concentration: f64,
    /// Largest nodal concentration, mol/m^3.
    pub max_concentration: f64,
}

/// `(t^j in s, row)`.
pub type TimeSeries = Vec<(f64, StepDiagnostics)>;

/// Exact running sum of doubles (Shewchuk's grow-expansion): the partials are non-overlapping
/// and hold the exact value of everything added so far.
#[derive(Clone, Debug, Default)]
pub struct Expansion {
    partials: Vec<f64>,
}

impl Expansion {
    /// Adds `x0` exactly.
    pub fn add(&mut self, x0: f64) {
        let mut x = x0;
        let mut i = 0;
        for j in 0..self.partials.len() {
            let mut y = self.partials[j];
            if x.abs() < y.abs() {
                std::mem::swap(&mut x, &mut y);
            }
            let hi = x + y;
            let lo = y - (hi - x);
            if lo != 0.0 {
                self.partials[i] = lo;
                i += 1;
            }
            x = hi;
        }
        self.partials.truncate(i);
        self.partials.push(x);
    }

    /// `|reported + (exact value held)|`; with the held value `-sum x_j` this is the exact
    /// accumulation error of `reported`. The final summation from the largest partial is
    /// accurate to a relative `u`, which the factor allows for.
    pub fn magnitude_with(&self, reported: f64) -> f64 {
        let mut e = self.clone();
        e.add(reported);
        e.partials.iter().rev().sum::<f64>().abs() * (1.0 + 2.0 * BalanceBound::U)
    }
}

/// A-posteriori rounding bound on the balance residual `|eps^j| / M^0` (property P18).
///
/// The derivation is written in full beside the test that checks it
/// (`balance_bound_history` in `tests/btcs_verification.rs`); this is the same computation in
/// the production loop, so the observed residual is reported beside its derived bound in every
/// output row (coordinator decision 6). Notation: `u = 2^-53`, `gamma_k = k u / (1 - k u)`.
///
/// 1. The identity (F29) is the sum of the exact-form rows (F10) over `i = 1 ..= N-1`, which
///    telescopes; per step it holds up to the residual `rho` of the exact-form matrix.
/// 2. The stored matrix differs from the exact-form one by `3u/(1 - 3u) (|A||x|)`.
/// 3. The solve (F24) has backward error `kappa_t = 4u + O(u^2)` (derived in
///    `numerical::tridiagonal::tests::thomas_matches_dense_f24`, at most `gamma_5`). Together
///    `c_step = kappa_t + 3u/(1 - 3u)`, about `7u`, and per step
///    `4 pi c_step sum_i V_i (|A||x|)_i (1 + gamma_{N+6})`.
/// 4. What the solver reports differs from the exact quantities by `gamma_{N+1}` of `M` (at `j`
///    and at 0), by `gamma_4` per increment `dt Phi`, by the exactly measured accumulation
///    error of the compensated sum ([`Expansion`], no assumed constant for it) and by one
///    rounding of `M + R`.
/// 5. The per-step terms are added in step order: the factor `1 + gamma_{steps + 8}`.
///
/// No safety factor is applied and nothing is tightened empirically.
#[derive(Clone, Debug)]
pub struct BalanceBound {
    n_nodes: f64,
    steps: f64,
    m0: f64,
    cumulative: f64,
    neg_increments: Expansion,
}

impl BalanceBound {
    /// Unit roundoff of `f64`.
    pub const U: f64 = f64::EPSILON / 2.0;

    fn gamma(k: f64) -> f64 {
        k * Self::U / (1.0 - k * Self::U)
    }

    /// Backward-error constant of the F24 solve for the sign pattern of (F19): `4u + O(u^2)`.
    pub fn kappa_t() -> f64 {
        let u = Self::U;
        let a = u / (1.0 - u);
        (2.0 * a + Self::gamma(2.0) + a * Self::gamma(2.0)) * (1.0 - u) / (1.0 - 2.0 * u)
    }

    /// `c_step = kappa_t + 3u/(1 - 3u)`.
    pub fn c_step() -> f64 {
        Self::kappa_t() + 3.0 * Self::U / (1.0 - 3.0 * Self::U)
    }

    /// Bound for a grid whose outer node has index `n` (`N`), a run of `steps` steps and the
    /// initial inventory `m0` (mol).
    pub fn new(n: usize, steps: usize, m0: f64) -> BalanceBound {
        BalanceBound {
            n_nodes: n as f64,
            steps: steps as f64,
            m0,
            cumulative: 0.0,
            neg_increments: Expansion::default(),
        }
    }

    fn g_inv(&self) -> f64 {
        Self::gamma(self.n_nodes + 1.0) / (1.0 - Self::gamma(self.n_nodes + 1.0))
    }

    /// The bound `|eps^0| / M^0` of the first row, whose reported inventory is `inventory`.
    pub fn first_row(&self, inventory: f64) -> f64 {
        (self.g_inv() * (inventory + self.m0) + Self::U * inventory) / (1.0 - Self::U) / self.m0
    }

    /// Accumulates one step: `a` is the assembled matrix, `v` the weights `V_i`, `c_new` the
    /// profile after the step and `increment` the release `dt Phi` of the step. Must be called
    /// for every step, whatever the output stride.
    pub fn add_step(&mut self, a: &Tridiagonal, v: &[f64], c_new: &[f64], increment: f64) {
        let n = a.diag.len();
        let mut s = 0.0;
        for i in 1..n {
            let left = a.lower[i].abs() * c_new[i - 1].abs();
            let right = a.upper[i].abs() * c_new[i + 1].abs();
            s += v[i] * (left + a.diag[i].abs() * c_new[i].abs() + right);
        }
        self.cumulative += 4.0
            * std::f64::consts::PI
            * Self::c_step()
            * s
            * (1.0 + Self::gamma(self.n_nodes + 6.0))
            / self.m0;
        self.neg_increments.add(-increment);
    }

    /// The bound on `|eps^j| / M^0` of the row after the steps added so far, whose reported
    /// inventory and cumulative release are given.
    pub fn row(&self, inventory: f64, cumulative_release: f64) -> f64 {
        let e_k = self.neg_increments.magnitude_with(cumulative_release);
        let rounding = self.g_inv() * (inventory + self.m0)
            + Self::gamma(4.0) / (1.0 - Self::gamma(4.0)) * cumulative_release
            + e_k
            + Self::U * (inventory + cumulative_release);
        self.cumulative * (1.0 + Self::gamma(self.steps + 8.0))
            + rounding / (1.0 - Self::U) / self.m0
    }
}
/// Result of [`check_matrix_structure`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MatrixReport {
    /// Smallest `B_i - |L_i| - |U_i|` over the matrix entries (P1; 1 in exact arithmetic for
    /// interior rows, larger for row `N-1`).
    pub min_row_margin: f64,
    /// Largest `|B_i - |L_i| - |U_i| - 1|` over rows `0 ..= N-2`.
    pub max_interior_margin_deviation: f64,
    /// Largest row sum of absolute values, an upper bound for the matrix infinity norm.
    pub norm_inf: f64,
}

/// Sign pattern of the assembled matrix (P1): `L_i <= 0` with `L_1 == 0`, `U_i < 0` for
/// `0 <= i <= N-2`, `B_i > 0`; reports the row margins. The margin is not asserted here.
///
/// # Errors
///
/// A violated sign condition, or a non-finite entry.
pub fn check_matrix_structure(a: &Tridiagonal) -> Result<MatrixReport> {
    let n = a.diag.len();
    if n < 3 || a.lower.len() != n || a.upper.len() != n {
        bail!("tridiagonal storage has inconsistent lengths");
    }
    if a.lower[1] != 0.0 {
        bail!("L_1 is not exactly zero (F13)");
    }
    let mut min_margin = f64::INFINITY;
    let mut max_dev: f64 = 0.0;
    let mut norm_inf: f64 = 0.0;
    for i in 0..n {
        let l = if i >= 1 { a.lower[i] } else { 0.0 };
        let u = if i + 1 < n { a.upper[i] } else { 0.0 };
        let b = a.diag[i];
        if !(l.is_finite() && u.is_finite() && b.is_finite()) {
            bail!("non-finite entry in row {i}");
        }
        if l > 0.0 || (i + 1 < n && u >= 0.0) || b <= 0.0 {
            bail!("sign pattern violated in row {i}: L={l}, B={b}, U={u}");
        }
        let margin = b - l.abs() - u.abs();
        min_margin = min_margin.min(margin);
        if i + 1 < n {
            max_dev = max_dev.max((margin - 1.0).abs());
        }
        norm_inf = norm_inf.max(b.abs() + l.abs() + u.abs());
    }
    Ok(MatrixReport {
        min_row_margin: min_margin,
        max_interior_margin_deviation: max_dev,
        norm_inf,
    })
}

/// Result of [`check_step_properties`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepReport {
    /// Smallest entry of the new profile.
    pub min_new: f64,
    /// Largest entry of the new profile.
    pub max_new: f64,
    /// Largest entry of the old profile.
    pub max_old: f64,
    /// `min_new >= 0` (P4, P6), exact comparison with zero.
    pub non_negative: bool,
    /// `max(0, max_new - max_old)`: excess over the maximum principle (P3, P4); zero or a
    /// rounding quantity.
    pub max_principle_excess: f64,
}

/// Step-level properties P3, P4 and P6 between two profiles.
pub fn check_step_properties(c_old: &[f64], c_new: &[f64]) -> StepReport {
    let min_new = c_new.iter().copied().fold(f64::INFINITY, f64::min);
    let max_new = c_new.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let max_old = c_old.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    StepReport {
        min_new,
        max_new,
        max_old,
        non_negative: min_new >= 0.0,
        max_principle_excess: (max_new - max_old).max(0.0),
    }
}

/// Result of [`check_series`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeriesReport {
    /// Worst `|eps^j| / M^0` over the rows (P9, P18), reported not asserted.
    pub worst_relative_residual: f64,
    /// Row index of that worst residual.
    pub worst_residual_row: usize,
    /// `F` is non-decreasing over the rows.
    pub release_monotone: bool,
    /// `0 <= F <= 1` for all rows.
    pub release_in_range: bool,
    /// `M^j` is non-increasing over the rows (valid for no source and no decay), exact
    /// comparison. Where the release is zero (early steps) the computed sum may change by
    /// rounding, so this can be false although the identity holds; see the next field.
    pub inventory_non_increasing: bool,
    /// Largest relative increase `(M^{j+1} - M^j) / M^0` over consecutive rows (0 if none):
    /// a rounding quantity, to be compared with a bound from the operation count.
    pub max_inventory_increase_rel: f64,
    /// Smallest concentration over all rows.
    pub min_concentration: f64,
    /// Largest concentration over all rows.
    pub max_concentration: f64,
}

/// Series-level properties P4, P9 and P18. `initial_inventory` is `M^0` in mol.
pub fn check_series(series: &TimeSeries, initial_inventory: f64) -> SeriesReport {
    let mut worst = 0.0f64;
    let mut worst_row = 0usize;
    let mut mono = true;
    let mut in_range = true;
    let mut inv_mono = true;
    let mut max_increase = 0.0f64;
    let mut cmin = f64::INFINITY;
    let mut cmax = f64::NEG_INFINITY;
    for (k, (_, row)) in series.iter().enumerate() {
        let rel = row.balance_residual.abs() / initial_inventory;
        if rel > worst {
            worst = rel;
            worst_row = k;
        }
        if !(0.0..=1.0).contains(&row.release_fraction) {
            in_range = false;
        }
        if k > 0 {
            let prev = &series[k - 1].1;
            if row.release_fraction < prev.release_fraction {
                mono = false;
            }
            if row.inventory > prev.inventory {
                inv_mono = false;
                max_increase =
                    max_increase.max((row.inventory - prev.inventory) / initial_inventory);
            }
        }
        cmin = cmin.min(row.min_concentration);
        cmax = cmax.max(row.max_concentration);
    }
    SeriesReport {
        worst_relative_residual: worst,
        worst_residual_row: worst_row,
        release_monotone: mono,
        release_in_range: in_range,
        inventory_non_increasing: inv_mono,
        max_inventory_increase_rel: max_increase,
        min_concentration: cmin,
        max_concentration: cmax,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(inv: f64, rel: f64, f: f64, eps: f64) -> StepDiagnostics {
        StepDiagnostics {
            inventory: inv,
            cumulative_release: rel,
            release_fraction: f,
            release_rate: None,
            balance_residual: eps,
            balance_residual_bound: 1e-9,
            min_concentration: 0.0,
            max_concentration: 1.0,
        }
    }

    #[test]
    fn series_checks_detect_violations() {
        let good: TimeSeries = vec![
            (0.0, row(1.0, 0.0, 0.0, 0.0)),
            (1.0, row(0.8, 0.2, 0.2, 1e-16)),
            (2.0, row(0.5, 0.5, 0.5, -2e-16)),
        ];
        let r = check_series(&good, 1.0);
        assert!(r.release_monotone && r.release_in_range && r.inventory_non_increasing);
        assert_eq!(r.worst_residual_row, 2);
        let bad: TimeSeries = vec![
            (0.0, row(1.0, 0.0, 0.0, 0.0)),
            (1.0, row(0.8, 0.2, 0.2, 0.0)),
            (2.0, row(0.9, 0.1, 0.1, 0.0)),
        ];
        let r = check_series(&bad, 1.0);
        assert!(!r.release_monotone && !r.inventory_non_increasing);
    }

    #[test]
    fn step_properties() {
        let r = check_step_properties(&[1.0, 0.5], &[0.9, 0.0]);
        assert!(r.non_negative && r.max_principle_excess == 0.0);
        let r = check_step_properties(&[1.0, 0.5], &[1.1, -1e-30]);
        assert!(!r.non_negative);
        // No tolerance: the excess is the one subtraction 1.1 - 1.0, exact by Sterbenz.
        assert_eq!(r.max_principle_excess.to_bits(), (1.1f64 - 1.0).to_bits());
    }

    #[test]
    fn sign_pattern_negative_control() {
        let mut a = Tridiagonal {
            lower: vec![0.0, 0.0, -1.0, -1.0],
            diag: vec![2.0, 3.0, 3.0, 3.0],
            upper: vec![-1.0, -1.0, -1.0, 0.0],
        };
        assert!(check_matrix_structure(&a).is_ok());
        a.lower[2] = 0.5;
        assert!(check_matrix_structure(&a).is_err());
        a.lower[2] = -1.0;
        a.lower[1] = -0.1;
        assert!(check_matrix_structure(&a).is_err());
    }
}
