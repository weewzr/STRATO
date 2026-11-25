//! Tridiagonal linear solver (Thomas algorithm) in double precision.
//!
//! Solves `A x = d` where `A` is an `n` by `n` tridiagonal matrix given by its
//! sub-diagonal `lower` (length `n - 1`, entry `i` is `A[i + 1][i]`), diagonal `diag`
//! (length `n`) and super-diagonal `upper` (length `n - 1`, entry `i` is `A[i][i + 1]`).
//!
//! # Stability
//!
//! The algorithm is Gaussian elimination without row exchanges, in `O(n)` operations.
//! It is stable without pivoting when `A` is strictly diagonally dominant by rows
//! (`|diag[i]| > |lower[i - 1]| + |upper[i]|`), or symmetric positive definite, or an
//! M-matrix; the backward-Euler matrix of a diffusion problem (positive diagonal,
//! non-positive off-diagonals, dominant rows) is of this kind.
//!
//! For a matrix outside these classes the elimination can meet a zero or tiny pivot
//! or suffer large element growth although `A` is nonsingular. This module does not
//! pivot: a zero or non-finite pivot is reported as [`TridiagonalError::ZeroPivot`] or
//! [`TridiagonalError::NonFinite`], and nothing is promised about the accuracy of
//! the result when pivots are small but nonzero. Callers that cannot guarantee
//! dominance should check the residual.

use std::fmt;

/// Failure of a tridiagonal solve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TridiagonalError {
    /// The system has no unknowns.
    Empty,
    /// A slice length does not match the system size `n`.
    LengthMismatch {
        /// Name of the offending argument.
        what: &'static str,
        /// Required length.
        expected: usize,
        /// Length received.
        found: usize,
    },
    /// The pivot at the given row is exactly zero.
    ZeroPivot {
        /// Row index (0-based).
        row: usize,
    },
    /// A non-finite value (NaN or infinity) arose at the given row, from the input
    /// data or from an overflowing pivot.
    NonFinite {
        /// Row index (0-based).
        row: usize,
    },
}

impl fmt::Display for TridiagonalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "tridiagonal system has no unknowns"),
            Self::LengthMismatch {
                what,
                expected,
                found,
            } => write!(f, "{what} has length {found}, expected {expected}"),
            Self::ZeroPivot { row } => write!(f, "zero pivot at row {row}"),
            Self::NonFinite { row } => write!(f, "non-finite value at row {row}"),
        }
    }
}

impl std::error::Error for TridiagonalError {}

/// Solves the system in place without allocating.
///
/// On entry `x` holds the right-hand side; on success it holds the solution. `work`
/// is scratch space of length `n` (its content on exit is unspecified). `lower` and
/// `upper` have length `n - 1`, `diag`, `x` and `work` length `n`. The coefficient
/// slices are not modified. On error `x` and `work` hold unspecified values.
///
/// # Errors
///
/// [`TridiagonalError`] for `n = 0`, mismatched lengths, a zero pivot, or a non-finite
/// value met during elimination.
pub fn solve_in_place(
    lower: &[f64],
    diag: &[f64],
    upper: &[f64],
    x: &mut [f64],
    work: &mut [f64],
) -> Result<(), TridiagonalError> {
    let n = diag.len();
    if n == 0 {
        return Err(TridiagonalError::Empty);
    }
    let check = |what: &'static str, found: usize, expected: usize| {
        if found == expected {
            Ok(())
        } else {
            Err(TridiagonalError::LengthMismatch {
                what,
                expected,
                found,
            })
        }
    };
    check("lower", lower.len(), n - 1)?;
    check("upper", upper.len(), n - 1)?;
    check("rhs", x.len(), n)?;
    check("work", work.len(), n)?;

    // Forward sweep: work[i] = modified super-diagonal, x[i] = modified right-hand side.
    let mut pivot = diag[0];
    for i in 0..n {
        if i > 0 {
            pivot = diag[i] - lower[i - 1] * work[i - 1];
        }
        if !pivot.is_finite() {
            return Err(TridiagonalError::NonFinite { row: i });
        }
        if pivot == 0.0 {
            return Err(TridiagonalError::ZeroPivot { row: i });
        }
        let prev = if i > 0 { lower[i - 1] * x[i - 1] } else { 0.0 };
        x[i] = (x[i] - prev) / pivot;
        if i + 1 < n {
            work[i] = upper[i] / pivot;
        }
        if !x[i].is_finite() || (i + 1 < n && !work[i].is_finite()) {
            return Err(TridiagonalError::NonFinite { row: i });
        }
    }
    // Back substitution.
    for i in (0..n - 1).rev() {
        x[i] -= work[i] * x[i + 1];
    }
    if let Some(row) = x.iter().position(|v| !v.is_finite()) {
        return Err(TridiagonalError::NonFinite { row });
    }
    Ok(())
}

/// Convenience wrapper that allocates the scratch space and the solution.
///
/// # Errors
///
/// As [`solve_in_place`].
pub fn solve(
    lower: &[f64],
    diag: &[f64],
    upper: &[f64],
    rhs: &[f64],
) -> Result<Vec<f64>, TridiagonalError> {
    let mut x = rhs.to_vec();
    let mut work = vec![0.0; diag.len()];
    solve_in_place(lower, diag, upper, &mut x, &mut work)?;
    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic generator in [0, 1) (no external dependency).
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> f64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (self.0 >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    fn multiply(l: &[f64], d: &[f64], u: &[f64], x: &[f64]) -> Vec<f64> {
        let n = d.len();
        (0..n)
            .map(|i| {
                let mut s = d[i] * x[i];
                if i > 0 {
                    s += l[i - 1] * x[i - 1];
                }
                if i + 1 < n {
                    s += u[i] * x[i + 1];
                }
                s
            })
            .collect()
    }

    /// Dense Gaussian elimination with partial pivoting.
    fn dense_solve(l: &[f64], d: &[f64], u: &[f64], b: &[f64]) -> Vec<f64> {
        let n = d.len();
        let mut a = vec![vec![0.0; n + 1]; n];
        for i in 0..n {
            a[i][i] = d[i];
            if i > 0 {
                a[i][i - 1] = l[i - 1];
            }
            if i + 1 < n {
                a[i][i + 1] = u[i];
            }
            a[i][n] = b[i];
        }
        for c in 0..n {
            let p = (c..n)
                .max_by(|&r, &s| a[r][c].abs().total_cmp(&a[s][c].abs()))
                .unwrap();
            a.swap(c, p);
            for r in c + 1..n {
                let f = a[r][c] / a[c][c];
                for k in c..=n {
                    let v = a[c][k];
                    a[r][k] -= f * v;
                }
            }
        }
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let s: f64 = (i + 1..n).map(|k| a[i][k] * x[k]).sum();
            x[i] = (a[i][n] - s) / a[i][i];
        }
        x
    }

    fn random_dominant(n: usize, seed: u64) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
        let mut g = Lcg(seed);
        let l: Vec<f64> = (0..n - 1).map(|_| g.next() * 2.0 - 1.0).collect();
        let u: Vec<f64> = (0..n - 1).map(|_| g.next() * 2.0 - 1.0).collect();
        let d: Vec<f64> = (0..n)
            .map(|i| {
                let off = if i > 0 { l[i - 1].abs() } else { 0.0 }
                    + if i + 1 < n { u[i].abs() } else { 0.0 };
                (off + 0.1 + g.next()) * if g.next() < 0.5 { 1.0 } else { -1.0 }
            })
            .collect();
        let b: Vec<f64> = (0..n).map(|_| g.next() * 10.0 - 5.0).collect();
        (l, d, u, b)
    }

    #[test]
    fn identity() {
        let b = [3.0, -1.0, 4.0, 1.5];
        let x = solve(&[0.0; 3], &[1.0; 4], &[0.0; 3], &b).unwrap();
        assert_eq!(x, b);
    }

    #[test]
    fn size_one() {
        assert_eq!(solve(&[], &[4.0], &[], &[10.0]).unwrap(), vec![2.5]);
    }

    #[test]
    fn size_two() {
        // [2 1; 1 3] x = [4; 7] -> x = [1; 2]
        let x = solve(&[1.0], &[2.0, 3.0], &[1.0], &[4.0, 7.0]).unwrap();
        assert!((x[0] - 1.0).abs() < 1e-15 && (x[1] - 2.0).abs() < 1e-15);
    }

    #[test]
    fn size_three() {
        // [4 1 0; 1 4 1; 0 1 4] x = [5; 6; 5] -> x = [1; 1; 1]
        let x = solve(&[1.0; 2], &[4.0; 3], &[1.0; 2], &[5.0, 6.0, 5.0]).unwrap();
        for v in x {
            assert!((v - 1.0).abs() < 1e-15);
        }
    }

    #[test]
    fn second_difference_closed_form() {
        // tridiag(-1, 2, -1) x = 1 has x_i = i (n + 1 - i) / 2, i = 1..=n.
        for &n in &[5usize, 100, 10_000] {
            let x = solve(
                &vec![-1.0; n - 1],
                &vec![2.0; n],
                &vec![-1.0; n - 1],
                &vec![1.0; n],
            )
            .unwrap();
            for (k, v) in x.iter().enumerate() {
                let i = (k + 1) as f64;
                let exact = i * (n as f64 + 1.0 - i) / 2.0;
                assert!(
                    (v - exact).abs() <= 1e-12 * exact.max(1.0) * n as f64,
                    "n={n} i={i}"
                );
            }
        }
    }

    #[test]
    fn random_dominant_residual_and_dense_agreement() {
        for (n, seed) in [
            (2usize, 1u64),
            (3, 2),
            (10, 3),
            (57, 4),
            (200, 5),
            (50_000, 6),
        ] {
            let (l, d, u, b) = random_dominant(n, seed);
            let x = solve(&l, &d, &u, &b).unwrap();
            let r = multiply(&l, &d, &u, &x);
            let rmax = r
                .iter()
                .zip(&b)
                .map(|(p, q)| (p - q).abs())
                .fold(0.0, f64::max);
            let bmax = b.iter().fold(0.0f64, |m, v| m.max(v.abs()));
            let xmax = x.iter().fold(0.0f64, |m, v| m.max(v.abs()));
            let dmax = d.iter().fold(0.0f64, |m, v| m.max(v.abs()));
            // Backward-stable bound with a modest constant.
            assert!(
                rmax <= 8.0 * f64::EPSILON * (dmax * 3.0 * xmax + bmax),
                "n={n} r={rmax}"
            );
            if n <= 200 {
                let y = dense_solve(&l, &d, &u, &b);
                for (p, q) in x.iter().zip(&y) {
                    assert!((p - q).abs() <= 1e-10 * (1.0 + q.abs()), "n={n}");
                }
            }
        }
    }

    #[test]
    fn in_place_matches_allocating_and_keeps_coefficients() {
        let (l, d, u, b) = random_dominant(30, 9);
        let (l0, d0, u0) = (l.clone(), d.clone(), u.clone());
        let mut x = b.clone();
        let mut w = vec![0.0; 30];
        solve_in_place(&l, &d, &u, &mut x, &mut w).unwrap();
        assert_eq!(x, solve(&l, &d, &u, &b).unwrap());
        assert_eq!((l, d, u), (l0, d0, u0));
    }

    #[test]
    fn error_cases() {
        assert_eq!(solve(&[], &[], &[], &[]), Err(TridiagonalError::Empty));
        assert!(matches!(
            solve(&[1.0], &[1.0, 1.0], &[], &[1.0, 1.0]),
            Err(TridiagonalError::LengthMismatch {
                what: "upper",
                expected: 1,
                found: 0
            })
        ));
        assert!(matches!(
            solve(&[1.0], &[1.0, 1.0], &[1.0], &[1.0]),
            Err(TridiagonalError::LengthMismatch { what: "rhs", .. })
        ));
        let mut x = [1.0, 1.0];
        let mut w = [0.0; 1];
        assert!(matches!(
            solve_in_place(&[1.0], &[1.0, 1.0], &[1.0], &mut x, &mut w),
            Err(TridiagonalError::LengthMismatch { what: "work", .. })
        ));
        // Zero first pivot.
        assert_eq!(
            solve(&[1.0], &[0.0, 1.0], &[1.0], &[1.0, 1.0]),
            Err(TridiagonalError::ZeroPivot { row: 0 })
        );
        // Singular: second pivot 1 - 1*1 = 0.
        assert_eq!(
            solve(&[1.0], &[1.0, 1.0], &[1.0], &[1.0, 1.0]),
            Err(TridiagonalError::ZeroPivot { row: 1 })
        );
        // Non-finite inputs.
        assert!(matches!(
            solve(&[1.0], &[f64::NAN, 1.0], &[1.0], &[1.0, 1.0]),
            Err(TridiagonalError::NonFinite { row: 0 })
        ));
        assert!(matches!(
            solve(&[1.0], &[1.0, 1.0], &[1.0], &[f64::INFINITY, 1.0]),
            Err(TridiagonalError::NonFinite { .. })
        ));
        assert!(!TridiagonalError::ZeroPivot { row: 3 }
            .to_string()
            .is_empty());
    }
}
