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
    /// Smallest nodal concentration, mol/m^3.
    pub min_concentration: f64,
    /// Largest nodal concentration, mol/m^3.
    pub max_concentration: f64,
}

/// `(t^j in s, row)`.
pub type TimeSeries = Vec<(f64, StepDiagnostics)>;

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
        assert!((r.max_principle_excess - 0.1).abs() < 1e-15);
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
