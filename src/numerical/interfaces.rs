//! Interface-node and interior-node row forms, and the interface flow from both sides.
//!
//! Implements Document 2 equations (F14), (F15), (F16) with (F16a) to (F16d), and (F31);
//! pseudocode routines PS-07 and PS-15. The explicit forms are checks: the matrix itself is
//! filled by the one general row (F11) in `btcs::assemble`.

use super::{coefficients::Coefficients, grid::Grid};
use std::f64::consts::PI;

/// Interior row written by material (F14): returns `(L_i, B_i, U_i)` for a node strictly
/// inside a material with mesh Fourier number `fo`, spacing `dr` (m) and radius `r_i` (m).
/// In the kernel `dr / r_i = 1 / i` (F15); that factor is not valid in a coating.
pub fn interior_row_explicit(fo: f64, dr: f64, r_i: f64) -> (f64, f64, f64) {
    let l = -fo * (1.0 - dr / r_i);
    let u = -fo * (1.0 + dr / r_i);
    (l, 1.0 + 2.0 * fo, u)
}

/// Interface row written out (F16), for the four nodes (F16a) to (F16d): returns
/// `(L_i, B_i, U_i)` for an interface node at radius `radius` (m) between an inner material
/// (`fo_in`, `dr_in`) and an outer material (`fo_out`, `dr_out`). The inner flow uses the
/// inner diffusivity only and the outer flow the outer diffusivity only; no diffusivity is
/// averaged.
pub fn interface_row_explicit(
    fo_in: f64,
    fo_out: f64,
    dr_in: f64,
    dr_out: f64,
    radius: f64,
) -> (f64, f64, f64) {
    let s = dr_in + dr_out;
    let l = -fo_in * (2.0 * dr_in / s) * (1.0 - dr_in / radius);
    let u = -fo_out * (2.0 * dr_out / s) * (1.0 + dr_out / radius);
    (l, 1.0 - l - u, u)
}

/// Outward flow `Phi_R^{j+1}` (mol/s) through an interface node `i`, evaluated by both
/// expressions of (F31): returns `(from_inner, from_outer)`. They are equal in exact
/// arithmetic (property P8); their floating-point difference is a diagnostic.
pub fn interface_flux_both_sides(
    grid: &Grid,
    coeff: &Coefficients,
    c_old: &[f64],
    c_new: &[f64],
    dt: f64,
    i: usize,
) -> (f64, f64) {
    let radius = grid.r[i];
    let rate_of_change = (c_new[i] - c_old[i]) / dt;
    let k_in = coeff.k_outer[i - 1];
    let k_out = coeff.k_outer[i];
    let half_in = radius * radius * grid.dr_inner(i) / 2.0;
    let half_out = radius * radius * grid.dr_outer(i) / 2.0;
    let from_inner = 4.0 * PI * (k_in * (c_new[i - 1] - c_new[i]) - half_in * rate_of_change);
    let from_outer = 4.0 * PI * (k_out * (c_new[i] - c_new[i + 1]) + half_out * rate_of_change);
    (from_inner, from_outer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interior_row_matches_hand_values() {
        // Kernel node i = 2: dr / r = 1/2.
        let (l, b, u) = interior_row_explicit(3.0, 1.0, 2.0);
        assert_eq!(l, -3.0 * 0.5);
        assert_eq!(b, 7.0);
        assert_eq!(u, -3.0 * 1.5);
    }

    #[test]
    fn interface_row_equal_spacing_reduces_to_interior_row() {
        // Equal spacings and equal Fo: the interface row equals the interior row (F14).
        let (l, b, u) = interface_row_explicit(2.0, 2.0, 0.5, 0.5, 4.0);
        let (li, bi, ui) = interior_row_explicit(2.0, 0.5, 4.0);
        assert!((l - li).abs() < 1e-14 && (b - bi).abs() < 1e-14 && (u - ui).abs() < 1e-14);
    }

    #[test]
    fn no_averaged_diffusivity() {
        // Equal spacings, different D: L depends on fo_in only, U on fo_out only.
        let (l1, _, u1) = interface_row_explicit(2.0, 5.0, 0.5, 0.5, 4.0);
        let (l2, _, u2) = interface_row_explicit(2.0, 9.0, 0.5, 0.5, 4.0);
        let (l3, _, u3) = interface_row_explicit(7.0, 5.0, 0.5, 0.5, 4.0);
        assert_eq!(l1, l2);
        assert_eq!(u1, u3);
        assert!(u1 != u2 && l1 != l3);
    }
}
