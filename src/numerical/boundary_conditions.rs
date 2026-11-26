//! Centre row, absorbing outer boundary and release rate of the BTCS solver.
//!
//! Implements Document 2 equations (F12), (F18) and (F26); pseudocode routines PS-06, PS-08
//! and the release rate of PS-13.

use std::f64::consts::PI;

/// Centre row (F12): `B_0 = 1 + 6 Fo_kernel`, `U_0 = -6 Fo_kernel`. Returns `(B_0, U_0)`.
/// The right-hand side is `b_0 = C_0^j` (F20). Row 0 is not a flux balance and carries no
/// inventory because `V_0 = 0` (F7).
pub fn centre_row(fo_kernel: f64) -> (f64, f64) {
    let six_fo = 6.0 * fo_kernel;
    (1.0 + six_fo, -six_fo)
}

/// The absorbing boundary value `C_N^j = 0` (F18), in mol/m^3.
pub fn outer_value() -> f64 {
    0.0
}

/// Extends the `N` unknowns `C_0 .. C_{N-1}` to the full profile of length `N + 1` with
/// `C_N = 0` (F18). `full` must have length `c_unknowns.len() + 1`.
pub fn extend_with_boundary(c_unknowns: &[f64], full: &mut [f64]) {
    let n = c_unknowns.len();
    full[..n].copy_from_slice(c_unknowns);
    full[n] = outer_value();
}

/// Release rate through `R_5` at the new time level (F26):
/// `Phi^{j+1} = 4 pi K_{N-1,outer} C_{N-1}^{j+1}` in mol/s, with `k_last` the stored
/// conductance that also sits in row `N-1`.
pub fn release_rate(k_last: f64, c_last: f64) -> f64 {
    4.0 * PI * k_last * c_last
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centre_row_f12() {
        let (b, u) = centre_row(0.25);
        assert_eq!(b, 2.5);
        assert_eq!(u, -1.5);
    }

    #[test]
    fn outer_value_is_zero_f18() {
        let mut full = vec![9.0; 4];
        extend_with_boundary(&[1.0, 2.0, 3.0], &mut full);
        assert_eq!(full, vec![1.0, 2.0, 3.0, 0.0]);
        assert_eq!(outer_value(), 0.0);
    }

    #[test]
    fn release_rate_f26() {
        assert_eq!(release_rate(2.0, 3.0), 4.0 * PI * 6.0);
    }
}
