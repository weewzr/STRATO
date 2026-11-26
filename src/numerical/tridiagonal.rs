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

/// Factors of the Thomas algorithm in the form of Document 2 Section 6.3, equation (F24),
/// and Document 3 PS-10. The forward elimination of the matrix (multipliers `w` and pivots
/// `m`) is done once in [`ThomasFactors::new`]; [`ThomasFactors::solve`] then processes a
/// right-hand side with exactly the floating-point operations of (F24).
///
/// Unlike [`solve_in_place`], this form requires every pivot to be finite and strictly
/// positive, the hypothesis of property P6 (a non-negative right-hand side gives a
/// non-negative computed solution, because for a matrix with a positive diagonal and
/// non-positive off-diagonals the multipliers `w` are non-positive and every operation of
/// the two sweeps adds non-negative terms). Do not replace this routine by a textbook
/// variant without a new review (Document 3, PS-10).
///
/// Coefficient convention as in this module: `lower[i]` is `A[i + 1][i]`, `upper[i]` is
/// `A[i][i + 1]`, both of length `n - 1`.
#[derive(Clone, Debug, PartialEq)]
pub struct ThomasFactors {
    /// Multipliers; `w[i] = L_i / m[i-1]` for `i >= 1` (`w[0]` unused, 0).
    w: Vec<f64>,
    /// Pivots `m_i = B_i - w_i U_{i-1}`.
    m: Vec<f64>,
    /// Super-diagonal, `upper[i] = A[i][i+1]`.
    upper: Vec<f64>,
}

impl ThomasFactors {
    /// Forward elimination of the matrix, (F24) first two lines.
    ///
    /// # Errors
    ///
    /// [`TridiagonalError`] for `n = 0`, mismatched lengths, or a pivot that is not finite
    /// and strictly positive (reported as [`TridiagonalError::ZeroPivot`] when it is zero
    /// or negative, [`TridiagonalError::NonFinite`] otherwise).
    pub fn new(lower: &[f64], diag: &[f64], upper: &[f64]) -> Result<Self, TridiagonalError> {
        let n = diag.len();
        if n == 0 {
            return Err(TridiagonalError::Empty);
        }
        for (what, found) in [("lower", lower.len()), ("upper", upper.len())] {
            if found != n - 1 {
                return Err(TridiagonalError::LengthMismatch {
                    what,
                    expected: n - 1,
                    found,
                });
            }
        }
        let mut w = vec![0.0; n];
        let mut m = vec![0.0; n];
        m[0] = diag[0];
        for i in 1..n {
            if !(m[i - 1].is_finite() && m[i - 1] > 0.0) {
                break;
            }
            w[i] = lower[i - 1] / m[i - 1];
            m[i] = diag[i] - w[i] * upper[i - 1];
        }
        for (i, &mi) in m.iter().enumerate() {
            if !mi.is_finite() || !w[i].is_finite() {
                return Err(TridiagonalError::NonFinite { row: i });
            }
            if mi <= 0.0 {
                return Err(TridiagonalError::ZeroPivot { row: i });
            }
        }
        Ok(Self {
            w,
            m,
            upper: upper.to_vec(),
        })
    }

    /// Order `n` of the system.
    pub fn len(&self) -> usize {
        self.m.len()
    }

    /// Always false: a factorisation has at least one unknown.
    pub fn is_empty(&self) -> bool {
        self.m.is_empty()
    }

    /// The pivots `m_i`.
    pub fn pivots(&self) -> &[f64] {
        &self.m
    }

    /// Solves `A x = rhs` with the right-hand-side sweeps of (F24). `work` (length `n`)
    /// holds `b'`. Allocates nothing. Lengths must be `n`; this is checked.
    ///
    /// # Errors
    ///
    /// [`TridiagonalError::LengthMismatch`], or [`TridiagonalError::NonFinite`] if the
    /// right-hand side makes the result non-finite.
    pub fn solve(
        &self,
        rhs: &[f64],
        x: &mut [f64],
        work: &mut [f64],
    ) -> Result<(), TridiagonalError> {
        let n = self.m.len();
        for (what, found) in [("rhs", rhs.len()), ("x", x.len()), ("work", work.len())] {
            if found != n {
                return Err(TridiagonalError::LengthMismatch {
                    what,
                    expected: n,
                    found,
                });
            }
        }
        work[0] = rhs[0];
        for i in 1..n {
            work[i] = rhs[i] - self.w[i] * work[i - 1];
        }
        x[n - 1] = work[n - 1] / self.m[n - 1];
        for i in (0..n - 1).rev() {
            x[i] = (work[i] - self.upper[i] * x[i + 1]) / self.m[i];
        }
        match x.iter().position(|v| !v.is_finite()) {
            Some(row) => Err(TridiagonalError::NonFinite { row }),
            None => Ok(()),
        }
    }
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
        // [2 1; 1 3] x = [4; 7] -> x = [1; 2]. No tolerance: the operations are
        // 4/2 = 2, 1/2 = 0.5, 3 - 0.5 = 2.5, (7 - 2)/2.5 = 2, 2 - 0.5 * 2 = 1, all exact.
        let x = solve(&[1.0], &[2.0, 3.0], &[1.0], &[4.0, 7.0]).unwrap();
        assert_eq!(x, vec![1.0, 2.0]);
    }

    #[test]
    fn size_three() {
        // [2 1 0; 1 2.5 1; 0 1 2.5] x = [4; 9; 9.5] -> x = [1; 2; 3]. No tolerance: the
        // pivots are 2, 2.5 - 0.5 = 2, 2.5 - 0.5 = 2 and every intermediate value
        // (2, 3.5, 3, 0.5) is a dyadic rational of few bits, so each operation is exact.
        let x = solve(&[1.0; 2], &[2.0, 2.5, 2.5], &[1.0; 2], &[4.0, 9.0, 9.5]).unwrap();
        assert_eq!(x, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn second_difference_closed_form() {
        // tridiag(-1, 2, -1) x = 1 has x_i = i (n + 1 - i) / 2, i = 1..=n.
        // Derived a-posteriori bound (u = 2^-53, gamma_k = k u / (1 - k u)): the exact inverse
        // is positive with entries i (n + 1 - j) / (n + 1) (i <= j), so
        // ||A^-1||_inf = max_i (A^-1 1)_i = max_i x_i = (n + 1)^2 / 8. With the exact residual
        // r = b - A x_hat, x - x_hat = A^-1 r, so |x - x_hat|_inf <= ||A^-1||_inf |r|_inf. The
        // exact residual differs from the floating-point one evaluated here by at most
        // gamma_4 (|A||x_hat| + |b|)_i (3 products, 3 additions and b). The bound is rigorous
        // and, because cond(A) ~ n^2/4, is of relative size about n^2 u; it is looser than a
        // fixed constant would be at n = 10^4 and no constant is asserted.
        for &n in &[5usize, 100, 10_000] {
            let (l, d, up, b) = (
                vec![-1.0; n - 1],
                vec![2.0; n],
                vec![-1.0; n - 1],
                vec![1.0; n],
            );
            let x = solve(&l, &d, &up, &b).unwrap();
            let u = f64::EPSILON / 2.0;
            let g4 = 4.0 * u / (1.0 - 4.0 * u);
            let (r, s) = residual_and_scale(&l, &d, &up, &b, &x);
            let rmax = (0..n).map(|i| r[i].abs() + g4 * s[i]).fold(0.0, f64::max);
            let inv_norm = (n as f64 + 1.0) * (n as f64 + 1.0) / 8.0;
            let tol = inv_norm * rmax;
            for (k, v) in x.iter().enumerate() {
                let i = (k + 1) as f64;
                let exact = i * (n as f64 + 1.0 - i) / 2.0;
                assert!((v - exact).abs() <= tol, "n={n} i={i} tol {tol}");
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
            // Derived bounds, u = 2^-53, gamma_k = k u / (1 - k u), a = u / (1 - u).
            // (1) Backward error of `solve_in_place` (pivots m_i = d_i - l w_{i-1} with
            //     w_i = fl(u_i / m_i), y_i = fl(fl(b_i - fl(l y_{i-1})) / m_i),
            //     x_i = fl(y_i - fl(w_i x_{i+1}))). Writing out the roundings gives
            //     |A - L U| <= a |L||U| (L lower with diagonal m_i, U unit upper with w_i),
            //     (L + dL) y = b with |dL| <= gamma_2 |L|, (U + dU) x = y with |dU| <= a |U|,
            //     hence (A + dA) x = b with |dA| <= kappa |L||U|,
            //     kappa = 2 a + gamma_2 + a gamma_2 (= 4u + O(u^2)).
            // (2) Row dominance with margin >= 0.1 (by construction of `random_dominant`) gives
            //     |w_i| <= 1 + O(u) by induction (|m_i| >= |d_i| - |l| (1 + O(u)) >= |u_i|),
            //     so (|L||U|)_ii <= |d_i| + 2 |l| (1 + 3u) <= 3 (1 + 3u) |d_i| and the off-diagonal
            //     entries are at most (1 + u) |A|. Hence |dA| <= 3 (1 + 3u) kappa |A|.
            // (3) The residual below is evaluated in floating point: at most gamma_4 (|A||x| + |b|)_i
            //     away from the exact one (3 products, 3 additions, b).
            let u_r = f64::EPSILON / 2.0;
            let g = |k: f64| k * u_r / (1.0 - k * u_r);
            let a_u = u_r / (1.0 - u_r);
            let kappa = 2.0 * a_u + g(2.0) + a_u * g(2.0);
            let c_back = 3.0 * (1.0 + 3.0 * u_r) * kappa;
            let (rr, ss) = residual_and_scale(&l, &d, &u, &b, &x);
            for i in 0..n {
                let abs_ax = ss[i] - b[i].abs();
                assert!(
                    rr[i].abs() <= c_back * abs_ax + g(4.0) * ss[i],
                    "n={n} row {i}: residual {}",
                    rr[i]
                );
            }
            if n <= 200 {
                // |x - y|_inf <= ||A^-1||_inf (||r_x|| + ||r_y||) with Varah's bound
                // ||A^-1||_inf <= 1 / margin for row dominance. The margin is evaluated in
                // floating point (two subtractions: error at most gamma_2 (|d| + |l| + |u|)),
                // which is subtracted.
                let y = dense_solve(&l, &d, &u, &b);
                let (ry, sy) = residual_and_scale(&l, &d, &u, &b, &y);
                let margin = (0..n)
                    .map(|i| {
                        let lo = if i > 0 { l[i - 1].abs() } else { 0.0 };
                        let up = if i + 1 < n { u[i].abs() } else { 0.0 };
                        (d[i].abs() - lo - up) - g(2.0) * (d[i].abs() + lo + up)
                    })
                    .fold(f64::INFINITY, f64::min);
                assert!(margin > 0.0, "n={n}");
                let nx = (0..n)
                    .map(|i| rr[i].abs() + g(4.0) * ss[i])
                    .fold(0.0, f64::max);
                let ny = (0..n)
                    .map(|i| ry[i].abs() + g(4.0) * sy[i])
                    .fold(0.0, f64::max);
                let tol = (nx + ny) / margin;
                for (p, q) in x.iter().zip(&y) {
                    assert!((p - q).abs() <= tol, "n={n}: {p} vs {q} tol {tol}");
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

    /// Dense reference for the F24 tests, sign pattern of (F19): positive diagonal,
    /// non-positive off-diagonals, strict dominance.
    fn f24_matrix(n: usize, seed: u64) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let mut g = Lcg(seed);
        let l: Vec<f64> = (0..n - 1).map(|_| -g.next() * 5.0).collect();
        let u: Vec<f64> = (0..n - 1).map(|_| -g.next() * 5.0).collect();
        let d: Vec<f64> = (0..n)
            .map(|i| {
                let off = if i > 0 { -l[i - 1] } else { 0.0 } + if i + 1 < n { -u[i] } else { 0.0 };
                1.0 + off + g.next() * 3.0
            })
            .collect();
        (l, d, u)
    }

    /// Residual `b - A x` evaluated in floating point, and `(|A||x| + |b|)_i`.
    fn residual_and_scale(
        l: &[f64],
        d: &[f64],
        u: &[f64],
        b: &[f64],
        x: &[f64],
    ) -> (Vec<f64>, Vec<f64>) {
        let n = d.len();
        let mut r = vec![0.0; n];
        let mut s = vec![0.0; n];
        for i in 0..n {
            let mut v = b[i] - d[i] * x[i];
            let mut sc = b[i].abs() + (d[i] * x[i]).abs();
            if i > 0 {
                v -= l[i - 1] * x[i - 1];
                sc += (l[i - 1] * x[i - 1]).abs();
            }
            if i + 1 < n {
                v -= u[i] * x[i + 1];
                sc += (u[i] * x[i + 1]).abs();
            }
            r[i] = v;
            s[i] = sc;
        }
        (r, s)
    }

    #[test]
    fn thomas_matches_dense_f24() {
        // Derived bounds, u = 2^-53, gamma_k = k u / (1 - k u).
        // (1) Backward error of (F24), derived here (no theorem is quoted). Let a = u/(1-u),
        //     gamma_2 = 2u/(1-2u). Factor: w_i = fl(L_i/m_{i-1}) and
        //     m_i = fl(B_i - fl(w_i U_{i-1})). Writing out the roundings, the product of the
        //     computed factors (Lh unit lower with w_i, Uh upper with m_i and U_i) satisfies
        //     (Lh Uh)_{i,i-1} = L_i (1 + d), |d| <= u; (Lh Uh)_{i,i+1} = U_i exactly;
        //     (Lh Uh)_ii = B_i + e_i with |e_i| <= a (|Lh||Uh|)_ii. So |A - Lh Uh| <= a |Lh||Uh|.
        //     Forward sweep y_i = fl(c_i - fl(w_i y_{i-1})): (Lh + dL) y = c, |dL| <= a |Lh|.
        //     Back sweep x_i = fl(fl(y_i - fl(U_i x_{i+1})) / m_i) (3 roundings):
        //     (Uh + dU) x = y, |dU| <= gamma_2 |Uh|. Hence (A + dA) x = c with
        //     |dA| <= (2a + gamma_2 + a gamma_2) |Lh||Uh| = (4u + O(u^2)) |Lh||Uh|.
        //     For a positive diagonal and non-positive off-diagonals (P1), w_i <= 0 and
        //     U_i < 0, so (|Lh||Uh|)_ii = m_i + w_i U_{i-1} has no cancellation and equals
        //     (Lh Uh)_ii <= B_i (1 - u)/(1 - 2u); off-diagonals are at most (1 + u)|A|.
        //     Therefore |dA| <= kappa_t |A|, kappa_t = (2a + gamma_2 + a gamma_2)(1 - u)/(1 - 2u)
        //     <= gamma_5 (checked below), and the exact residual obeys
        //     |b - A x|_i <= gamma_5 (|A||x|)_i. (Higham, Accuracy and Stability of Numerical
        //     Algorithms, 2nd ed., Section 9.6 states the same 4u + O(u^2) for tridiagonal LU;
        //     the theorem number was not checked against the book, so the derivation above is
        //     what this test relies on.)
        // (2) The residual evaluated below has at most 3 products, 3 additions and b, so it
        //     differs from the exact one by at most gamma_4 (|A||x| + |b|)_i.
        // (3) The pivoted dense reference y: its own backward error is not assumed; its
        //     measured residual enters. x - y = A^-1 ((b - A x) - (b - A y)) gives
        //     |x - y|_inf <= ||A^-1||_inf (||r_x|| + ||r_y||), and ||A^-1||_inf <= 1/margin
        //     (Varah) with margin = min_i (B_i - |L_i| - |U_i|), evaluated in floating point
        //     (two subtractions, error at most gamma_2 (B + |L| + |U|)), which is subtracted.
        let u = f64::EPSILON / 2.0;
        let gamma = |k: f64| k * u / (1.0 - k * u);
        {
            let a_u = u / (1.0 - u);
            let kappa_t = (2.0 * a_u + gamma(2.0) + a_u * gamma(2.0)) * (1.0 - u) / (1.0 - 2.0 * u);
            assert!(kappa_t <= gamma(5.0));
        }
        for (n, seed) in [(1usize, 1u64), (2, 2), (3, 3), (20, 4), (150, 5)] {
            let (l, d, u_) = f24_matrix(n, seed);
            let mut g = Lcg(seed + 100);
            let b: Vec<f64> = (0..n).map(|_| g.next() * 4.0).collect();
            let f = ThomasFactors::new(&l, &d, &u_).unwrap();
            assert!(f.pivots().iter().all(|&m| m > 0.0));
            let mut x = vec![0.0; n];
            let mut w = vec![0.0; n];
            f.solve(&b, &mut x, &mut w).unwrap();
            let y = dense_solve(&l, &d, &u_, &b);
            let margin = (0..n)
                .map(|i| {
                    let lo = if i > 0 { l[i - 1].abs() } else { 0.0 };
                    let up = if i + 1 < n { u_[i].abs() } else { 0.0 };
                    (d[i] - lo - up) - gamma(2.0) * (d[i] + lo + up)
                })
                .fold(f64::INFINITY, f64::min);
            assert!(margin > 0.0, "n={n} margin {margin}");
            let (rx, sx) = residual_and_scale(&l, &d, &u_, &b, &x);
            let (ry, sy) = residual_and_scale(&l, &d, &u_, &b, &y);
            let (mut nx, mut ny) = (0.0f64, 0.0f64);
            for i in 0..n {
                let abs_ax = sx[i] - b[i].abs();
                assert!(
                    rx[i].abs() <= gamma(5.0) * abs_ax + gamma(4.0) * sx[i],
                    "n={n} row {i}: residual {}",
                    rx[i]
                );
                nx = nx.max(rx[i].abs() + gamma(4.0) * sx[i]);
                ny = ny.max(ry[i].abs() + gamma(4.0) * sy[i]);
            }
            let tol = (nx + ny) / margin;
            for (p, q) in x.iter().zip(&y) {
                assert!((p - q).abs() <= tol, "n={n}: {p} vs {q}, tol {tol}");
            }
        }
    }

    #[test]
    fn pivot_bound_p5() {
        // Exact arithmetic (P5): m_i >= 1 + |U_i| (m_{n-1} >= 1), by induction using
        // |U_{i-1}|/m_{i-1} <= |U_{i-1}|/(1 + |U_{i-1}|) <= 5/6 for |U| <= 5, so the correction
        // w_i U_{i-1} (>= 0) is at most (5/6)|L_{i-1}|.
        // Floating point, u = 2^-53, gamma_k = k u / (1 - k u). The stored diagonal of the test
        // matrices is d = fl(fl(fl(1 + off) + fl(3 g))) with off = fl(|L| + |U|): 4 roundings
        // of non-negative terms (off, 1 + off, 3 g, the last sum), so
        // B_i >= (1 + |L_{i-1}| + |U_i|)(1 - gamma_4) (the term 3 g >= 0 is dropped). Forming
        // w (1 rounding) and w U (1) perturbs the correction c = |w||U_{i-1}| <= (5/6)|L|
        // by the factor (1 + gamma_2), and the subtraction rounds once more:
        //   m_i >= (1 - u) [ B_i - (5/6)|L| (1 + gamma_2) ]
        //       >= (1 - u) [ (1 + |U_i|) + |L|/6 - gamma_4 (1 + |L| + |U_i|) - (5/6) gamma_2 |L| ]
        //       >= (1 - u)(1 - gamma_4)(1 + |U_i|),
        // because |L|/6 >= gamma_4 |L| + (5/6) gamma_2 |L|. The same holds for row 0 (m_0 = B_0).
        // The induction hypothesis |U|/m <= 5/6 is kept since m >= (1 + |U|)(1 - gamma_5) with
        // |U| <= 5. Hence the asserted bound m_i >= (1 + |U_i|)(1 - gamma_5), where
        // (1 - u)(1 - gamma_4) >= 1 - gamma_5.
        let u = f64::EPSILON / 2.0;
        let gamma5 = 5.0 * u / (1.0 - 5.0 * u);
        for (n, seed) in [
            (1usize, 21u64),
            (2, 22),
            (3, 23),
            (20, 24),
            (150, 25),
            (400, 26),
        ] {
            let (l, d, up) = f24_matrix(n, seed);
            let f = ThomasFactors::new(&l, &d, &up).unwrap();
            for (i, &m) in f.pivots().iter().enumerate() {
                let un = if i + 1 < n { up[i].abs() } else { 0.0 };
                let bound = (1.0 + un) * (1.0 - gamma5);
                assert!(m >= bound, "n={n} row {i}: m = {m}, bound {bound}");
            }
            // Multipliers |U_i|/m_i < 1 in floating point as well.
            for (i, (uv, m)) in up.iter().zip(f.pivots()).enumerate() {
                assert!(uv.abs() / m < 1.0, "n={n} row {i}");
            }
        }
    }
    #[test]
    fn thomas_pivot_sign_negative_control() {
        // A negative diagonal entry makes a pivot negative: refused, not solved.
        let (l, mut d, u) = f24_matrix(6, 7);
        d[3] = -d[3];
        assert!(matches!(
            ThomasFactors::new(&l, &d, &u),
            Err(TridiagonalError::ZeroPivot { .. })
        ));
        // A positive off-diagonal (wrong sign pattern) that drives a pivot to zero.
        assert_eq!(
            ThomasFactors::new(&[1.0], &[1.0, 1.0], &[1.0]),
            Err(TridiagonalError::ZeroPivot { row: 1 })
        );
        assert!(matches!(
            ThomasFactors::new(&[], &[f64::NAN], &[]),
            Err(TridiagonalError::NonFinite { row: 0 })
        ));
        assert_eq!(
            ThomasFactors::new(&[], &[], &[]),
            Err(TridiagonalError::Empty)
        );
        assert!(matches!(
            ThomasFactors::new(&[1.0], &[1.0, 1.0], &[]),
            Err(TridiagonalError::LengthMismatch { what: "upper", .. })
        ));
    }

    #[test]
    fn thomas_nonnegative_in_floating_point_p6() {
        for (n, seed) in [(5usize, 11u64), (60, 12), (400, 13)] {
            let (l, d, u) = f24_matrix(n, seed);
            let f = ThomasFactors::new(&l, &d, &u).unwrap();
            let mut g = Lcg(seed);
            let mut x = vec![0.0; n];
            let mut w = vec![0.0; n];
            for _ in 0..50 {
                let b: Vec<f64> = (0..n)
                    .map(|_| if g.next() < 0.3 { 0.0 } else { g.next() })
                    .collect();
                f.solve(&b, &mut x, &mut w).unwrap();
                assert!(x.iter().all(|&v| v >= 0.0), "n={n}");
            }
        }
    }

    #[test]
    fn thomas_solve_checks_lengths() {
        let f = ThomasFactors::new(&[-1.0], &[2.0, 2.0], &[-1.0]).unwrap();
        let mut x = [0.0; 2];
        let mut w = [0.0; 1];
        assert!(matches!(
            f.solve(&[1.0, 1.0], &mut x, &mut w),
            Err(TridiagonalError::LengthMismatch { what: "work", .. })
        ));
        assert_eq!(f.len(), 2);
        assert!(!f.is_empty());
    }
}
