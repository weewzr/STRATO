//! Tridiagonal linear solvers (Thomas algorithm) in double precision.
//!
//! Two routines live here.
//!
//! * [`ThomasFactors`] is the production solver of the BTCS time loop. It is coded exactly in
//!   the form (F24) of Document 2 (Document 1 Section 9.2 (vi)), so that property P6
//!   (non-negativity in floating point) applies to the code as written. Its matrix storage is
//!   that of `btcs::Tridiagonal`: three vectors of length `n`, `lower[i] = L_i` (`lower[0]`
//!   unused), `diag[i] = B_i`, `upper[i] = U_i` (`upper[n - 1]` unused), `L_i` and `U_i`
//!   stored with their signs.
//! * [`solve_in_place`] and [`solve`] are the general textbook elimination (separate
//!   sub-diagonal and super-diagonal slices of length `n - 1`). They are kept as a utility
//!   and as the cross-check of [`ThomasFactors`] in tests; the BTCS time loop does not call
//!   them.
//!
//! The general routine solves `A x = d` where `A` is an `n` by `n` tridiagonal matrix given by its
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
    /// [`ThomasFactors`] only: the computed pivot `m_i` at the given row is not positive.
    /// Property P6 of Document 2 is proved under the hypothesis that every computed pivot is
    /// positive, so a matrix that violates it is refused instead of solved.
    NonPositivePivot {
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
            Self::NonPositivePivot { row } => write!(f, "non-positive pivot at row {row}"),
        }
    }
}

impl std::error::Error for TridiagonalError {}

/// The Thomas algorithm in the form (F24) of Document 2, with the elimination of the matrix
/// done once ([`ThomasFactors::new`]) and the work on the right-hand side done in every step
/// ([`ThomasFactors::solve`]). The split changes no floating-point operation: the numbers `w_i`
/// and `m_i` are those that (F24) computes in every step from the same matrix.
///
/// ```text
/// m_0 = B_0,   b'_0 = b_0
/// w_i = L_i / m_{i-1},   m_i = B_i - w_i U_{i-1},   b'_i = b_i - w_i b'_{i-1}     (i = 1 .. n-1)
/// x_{n-1} = b'_{n-1} / m_{n-1},   x_i = (b'_i - U_i x_{i+1}) / m_i                (i = n-2 .. 0)
/// ```
///
/// `L_i` and `U_i` are stored with their signs (`<= 0`). There is no pivoting and no division by
/// an off-diagonal entry. Property P6 (a non-negative right-hand side gives a non-negative
/// computed solution) holds for exactly these operations when every computed pivot is
/// positive, which `new` checks.
#[derive(Clone, Debug)]
pub struct ThomasFactors {
    /// Pivots `m_i`.
    m: Vec<f64>,
    /// Multipliers `w_i = L_i / m_{i-1}` (`w_0` unused, zero).
    w: Vec<f64>,
    /// `U_i`, with the sign it has in the matrix (`upper[n - 1]` unused).
    upper: Vec<f64>,
}

impl ThomasFactors {
    /// Forward elimination of the matrix.
    ///
    /// `lower`, `diag` and `upper` all have length `n >= 1`; `lower[0]` and `upper[n - 1]`
    /// are not read.
    ///
    /// # Errors
    ///
    /// [`TridiagonalError`] for `n = 0`, mismatched lengths, a non-finite pivot, or a pivot
    /// that is not positive ([`TridiagonalError::NonPositivePivot`]).
    pub fn new(
        lower: &[f64],
        diag: &[f64],
        upper: &[f64],
    ) -> Result<ThomasFactors, TridiagonalError> {
        let n = diag.len();
        if n == 0 {
            return Err(TridiagonalError::Empty);
        }
        for (what, found) in [("lower", lower.len()), ("upper", upper.len())] {
            if found != n {
                return Err(TridiagonalError::LengthMismatch {
                    what,
                    expected: n,
                    found,
                });
            }
        }
        let mut m = vec![0.0; n];
        let mut w = vec![0.0; n];
        m[0] = diag[0];
        for i in 0..n {
            if i > 0 {
                w[i] = lower[i] / m[i - 1];
                m[i] = diag[i] - w[i] * upper[i - 1];
            }
            if !m[i].is_finite() {
                return Err(TridiagonalError::NonFinite { row: i });
            }
            if m[i] <= 0.0 {
                return Err(if m[i] == 0.0 {
                    TridiagonalError::ZeroPivot { row: i }
                } else {
                    TridiagonalError::NonPositivePivot { row: i }
                });
            }
        }
        Ok(ThomasFactors {
            m,
            w,
            upper: upper.to_vec(),
        })
    }

    /// Order `n` of the system.
    pub fn len(&self) -> usize {
        self.m.len()
    }

    /// `true` if the system has no unknowns (never the case for a value built by `new`).
    pub fn is_empty(&self) -> bool {
        self.m.is_empty()
    }

    /// The pivots `m_i`.
    pub fn pivots(&self) -> &[f64] {
        &self.m
    }

    /// The multipliers `w_i = L_i / m_{i-1}`, with `w_0 = 0` (not used).
    pub fn multipliers(&self) -> &[f64] {
        &self.w
    }

    /// Solves `A x = rhs`. `work` (length `n`) receives `b'`; `x` receives the solution.
    /// Nothing is allocated.
    ///
    /// # Errors
    ///
    /// Mismatched lengths, or a non-finite result.
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
        if let Some(row) = x.iter().position(|v| !v.is_finite()) {
            return Err(TridiagonalError::NonFinite { row });
        }
        Ok(())
    }
}

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

    // ---- ThomasFactors: the form (F24) of Document 2 ------------------------------------

    const U_ROUND: f64 = f64::EPSILON / 2.0;

    /// A matrix with the sign pattern of (F19): `L_0` and `U_{n-1}` unused (zero), `L_i <= 0`,
    /// `U_i < 0`, `B_i = 1 + |L_i| + |U_i|` (the margin 1 of P1, rounded once). The magnitudes
    /// are log-uniform over `10^-3 .. 10^e_max`, standing for mesh Fourier numbers.
    fn f19_matrix(n: usize, seed: u64, e_max: f64) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let mut g = Lcg(seed);
        let mut lower = vec![0.0; n];
        let mut upper = vec![0.0; n];
        let mut diag = vec![0.0; n];
        let mag = |g: &mut Lcg| 10f64.powf(-3.0 + (3.0 + e_max) * g.next());
        for i in 0..n {
            if i >= 2 {
                lower[i] = -mag(&mut g);
            }
            if i + 1 < n {
                upper[i] = -mag(&mut g);
            }
        }
        for i in 0..n {
            diag[i] = 1.0 + lower[i].abs() + upper[i].abs();
        }
        // Row n-1 keeps the full diagonal of (F17) although its U is not a matrix entry.
        diag[n - 1] += mag(&mut g);
        (lower, diag, upper)
    }

    fn norm_inf(l: &[f64], d: &[f64], u: &[f64]) -> f64 {
        (0..d.len())
            .map(|i| d[i].abs() + l[i].abs() + u[i].abs())
            .fold(0.0, f64::max)
    }

    #[test]
    fn thomas_f24_matches_general_module() {
        // Pre-registered bound (Document 1 Section 9.2 and Higham, LU of a diagonally dominant
        // tridiagonal matrix): each solver has a backward error of at most 4 u (|A||x|)_i, and
        // ||A^-1||_inf <= 1 (P2), so each solution differs from the exact one by at most
        // 4 u ||A||_inf ||x||_inf; two solvers: 8 u. The bound used has a safety factor 2: 16 u.
        let mut worst: f64 = 0.0;
        for (seed, n, e_max) in [
            (1u64, 5usize, 0.0),
            (2, 40, 3.0),
            (3, 200, 6.0),
            (4, 400, 11.0),
        ] {
            let (lower, diag, upper) = f19_matrix(n, seed, e_max);
            let mut g = Lcg(seed + 100);
            let rhs: Vec<f64> = (0..n).map(|_| g.next()).collect();
            let f = ThomasFactors::new(&lower, &diag, &upper).unwrap();
            let mut x = vec![0.0; n];
            let mut work = vec![0.0; n];
            f.solve(&rhs, &mut x, &mut work).unwrap();
            let reference = solve(&lower[1..], &diag, &upper[..n - 1], &rhs).unwrap();
            let xmax = reference.iter().cloned().fold(0.0, f64::max);
            let tol = 16.0 * U_ROUND * norm_inf(&lower, &diag, &upper) * xmax;
            for i in 0..n {
                let e = (x[i] - reference[i]).abs();
                assert!(
                    e <= tol,
                    "n={n} i={i}: {} vs {} tol {tol}",
                    x[i],
                    reference[i]
                );
                worst = worst.max(e / tol);
            }
            // Residual of the system itself: backward error 4 u |A||x|, safety factor 2.
            let ax = multiply(&lower[1..], &diag, &upper[..n - 1], &x);
            let rtol = 8.0 * U_ROUND * norm_inf(&lower, &diag, &upper) * xmax;
            for i in 0..n {
                assert!((ax[i] - rhs[i]).abs() <= rtol, "residual row {i}");
            }
        }
        assert!(worst <= 1.0);
    }

    #[test]
    fn thomas_f24_pivot_bound_p5() {
        // P5: m_i >= 1 + |U_i| and |U_i| / m_i < 1 (multipliers below one). In floating point
        // with the stored entries: m_i = fl(B_i - fl(fl(L_i / m_{i-1}) U_{i-1})). With
        // |U_{i-1}| / m_{i-1} <= 1 the product is at most |L_i| (1 + 3u), so
        // m_i >= (B_i - |L_i|) - 4 u B_i. B_i - |L_i| is 1 + |U_i| up to the rounding of B_i.
        for (seed, n, e_max) in [(11u64, 6usize, 0.0), (12, 50, 4.0), (13, 300, 11.0)] {
            let (lower, diag, upper) = f19_matrix(n, seed, e_max);
            let f = ThomasFactors::new(&lower, &diag, &upper).unwrap();
            let mut worst_margin = f64::INFINITY;
            for i in 0..n {
                let bound = (diag[i] - lower[i].abs()) - 4.0 * U_ROUND * diag[i];
                assert!(
                    f.pivots()[i] >= bound,
                    "row {i}: m {} bound {bound}",
                    f.pivots()[i]
                );
                if i + 1 < n {
                    assert!(upper[i].abs() / f.pivots()[i] < 1.0, "multiplier row {i}");
                    worst_margin = worst_margin.min(f.pivots()[i] - upper[i].abs());
                }
            }
            assert!(worst_margin >= 1.0 - 8.0 * U_ROUND * diag.iter().cloned().fold(0.0, f64::max));
            assert!(f.multipliers().iter().all(|&w| w <= 0.0));
        }
    }

    #[test]
    fn thomas_f24_nonnegative_in_floating_point_p6_and_negative_control() {
        // P6: b >= 0 gives x >= 0 exactly (every operation acts on non-negative operands).
        for seed in 0..60u64 {
            let n = 3 + (seed as usize * 7) % 120;
            let (lower, diag, upper) = f19_matrix(n, 500 + seed, 11.0 * (seed % 5) as f64 / 4.0);
            let f = ThomasFactors::new(&lower, &diag, &upper).unwrap();
            let mut g = Lcg(900 + seed);
            let mut rhs: Vec<f64> = (0..n).map(|_| g.next()).collect();
            rhs[(seed as usize) % n] = 0.0;
            let mut x = vec![0.0; n];
            let mut work = vec![0.0; n];
            f.solve(&rhs, &mut x, &mut work).unwrap();
            assert!(x.iter().all(|&v| v >= 0.0), "negative entry, seed {seed}");
            assert!(
                work.iter().all(|&v| v >= 0.0),
                "negative b' entry, seed {seed}"
            );
        }
        // Negative control 1: flip the sign of the lower entries. The matrix is no longer an
        // M-matrix and the same algorithm with a non-negative right-hand side returns a negative
        // entry, so the test above can fail.
        let (mut lower, diag, upper) = f19_matrix(30, 7, 1.0);
        for l in lower.iter_mut() {
            *l = -*l;
        }
        let f = ThomasFactors::new(&lower, &diag, &upper).unwrap();
        let rhs = vec![1.0; 30];
        let mut x = vec![0.0; 30];
        let mut work = vec![0.0; 30];
        f.solve(&rhs, &mut x, &mut work).unwrap();
        assert!(
            x.iter().any(|&v| v < 0.0),
            "control must produce a negative entry"
        );
        // Negative control 2: a non-positive pivot is refused (the hypothesis of P6).
        let bad = ThomasFactors::new(&[0.0, 1.0], &[1.0, 1.0], &[1.0, 0.0]);
        assert_eq!(bad.unwrap_err(), TridiagonalError::ZeroPivot { row: 1 });
        let bad = ThomasFactors::new(&[0.0, 2.0], &[1.0, 1.0], &[1.0, 0.0]);
        assert_eq!(
            bad.unwrap_err(),
            TridiagonalError::NonPositivePivot { row: 1 }
        );
    }

    #[test]
    fn thomas_f24_is_the_written_form_f24_bit_for_bit() {
        // Evaluate (F24) literally, step by step, and compare bit for bit: the split into
        // `new` and `solve` changes no floating-point operation.
        let n = 25;
        let (lower, diag, upper) = f19_matrix(n, 21, 3.0);
        let mut g = Lcg(77);
        let b: Vec<f64> = (0..n).map(|_| g.next()).collect();
        let mut m = vec![0.0; n];
        let mut bp = vec![0.0; n];
        m[0] = diag[0];
        bp[0] = b[0];
        for i in 1..n {
            let w = lower[i] / m[i - 1];
            m[i] = diag[i] - w * upper[i - 1];
            bp[i] = b[i] - w * bp[i - 1];
        }
        let mut x = vec![0.0; n];
        x[n - 1] = bp[n - 1] / m[n - 1];
        for i in (0..n - 1).rev() {
            x[i] = (bp[i] - upper[i] * x[i + 1]) / m[i];
        }
        let f = ThomasFactors::new(&lower, &diag, &upper).unwrap();
        let mut y = vec![0.0; n];
        let mut work = vec![0.0; n];
        f.solve(&b, &mut y, &mut work).unwrap();
        for i in 0..n {
            assert_eq!(x[i].to_bits(), y[i].to_bits());
        }
    }

    #[test]
    fn thomas_f24_error_cases() {
        assert_eq!(
            ThomasFactors::new(&[], &[], &[]).unwrap_err(),
            TridiagonalError::Empty
        );
        assert!(matches!(
            ThomasFactors::new(&[0.0], &[1.0, 1.0], &[0.0, 0.0]),
            Err(TridiagonalError::LengthMismatch { .. })
        ));
        assert!(matches!(
            ThomasFactors::new(&[0.0, 0.0], &[1.0, f64::NAN], &[0.0, 0.0]),
            Err(TridiagonalError::NonFinite { row: 1 })
        ));
        let f = ThomasFactors::new(&[0.0, -1.0], &[2.0, 3.0], &[-1.0, 0.0]).unwrap();
        assert_eq!(f.len(), 2);
        assert!(!f.is_empty());
        let mut x = vec![0.0; 2];
        let mut work = vec![0.0; 1];
        assert!(f.solve(&[1.0, 1.0], &mut x, &mut work).is_err());
    }
}
