//! Closed-form references for verifying the coupled solver later.
//!
//! Each function is the exact solution of a stated problem, derived in
//! `docs/mathematics/STRATO_Property_Functions.md`. They are references, not a solver, and
//! share no code with any discretisation.
//!
//! Well-mixed volume (no spatial dependence), concentration `C(t)` in any unit with
//! `G` in that unit per second and `lambda` in s^-1:
//!
//! - decay only: `C = C_init exp(-lambda t)`;
//! - source only: `C = C_init + G t`;
//! - source with decay: `C = C_init exp(-lambda t) + (G / lambda)(1 - exp(-lambda t))`.
//!
//! Homogeneous sphere of radius `R`, diffusivity `D`, decay `lambda`, uniform initial
//! concentration `C_init`, `C(R, t) = 0` (absorbing surface), no source: the diffusion series
//! times `exp(-lambda t)`. Series are truncated at `n_terms`; the tail is bounded in
//! [`series_terms_for_tolerance`].

use super::{non_negative, positive, PhysicsError};
use std::f64::consts::PI;

/// Decay only, well mixed: `C_init exp(-lambda t)`.
pub fn decay_only(c_init: f64, lambda: f64, t: f64) -> Result<f64, PhysicsError> {
    non_negative("initial concentration", c_init)?;
    non_negative("decay constant", lambda)?;
    non_negative("time", t)?;
    Ok(c_init * (-lambda * t).exp())
}

/// Source only, well mixed: `C_init + G t` (linear growth).
pub fn source_only(c_init: f64, g: f64, t: f64) -> Result<f64, PhysicsError> {
    non_negative("initial concentration", c_init)?;
    non_negative("source", g)?;
    non_negative("time", t)?;
    Ok(c_init + g * t)
}

/// Source with decay, well mixed:
/// `C_init exp(-lambda t) + (G / lambda)(1 - exp(-lambda t))`; for `lambda = 0` this is
/// [`source_only`]. The factor `(1 - exp(-lambda t)) / lambda` is formed with `expm1`, so it
/// stays accurate for small `lambda t` and is exactly `t` at `lambda = 0`.
pub fn source_with_decay(c_init: f64, g: f64, lambda: f64, t: f64) -> Result<f64, PhysicsError> {
    non_negative("initial concentration", c_init)?;
    non_negative("source", g)?;
    non_negative("decay constant", lambda)?;
    non_negative("time", t)?;
    let growth = if lambda == 0.0 {
        t
    } else {
        -(-lambda * t).exp_m1() / lambda
    };
    Ok(c_init * (-lambda * t).exp() + g * growth)
}

/// The equilibrium `G / lambda` that [`source_with_decay`] approaches as `t` grows
/// (`lambda > 0`).
pub fn equilibrium_concentration(g: f64, lambda: f64) -> Result<f64, PhysicsError> {
    non_negative("source", g)?;
    positive("decay constant", lambda)?;
    Ok(g / lambda)
}

/// Largest term count [`series_terms_for_tolerance`] will return (1e9); a tolerance that needs
/// more is refused, so a later series sum cannot run for ever.
pub const MAX_SERIES_TERMS: usize = 1_000_000_000;

/// Number of series terms `N` such that the neglected tail of every series below is at most
/// `tol` (absolute, on a fraction of the initial inventory). Tail bound: each term is at
/// most `(6 / pi^2) / n^2` in magnitude, and `sum_{n>N} 1/n^2 < 1/N`, so
/// `N = ceil(6 / (pi^2 tol))` suffices for the three inventory fractions.
pub fn series_terms_for_tolerance(tol: f64) -> Result<usize, PhysicsError> {
    positive("tolerance", tol)?;
    let n = (6.0 / (PI * PI * tol)).ceil().max(1.0);
    if n > MAX_SERIES_TERMS as f64 {
        return Err(PhysicsError::OutOfRange(
            "tolerance needs more than MAX_SERIES_TERMS series terms",
        ));
    }
    Ok(n as usize)
}

/// A homogeneous sphere with an absorbing surface and a uniform initial concentration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HomogeneousSphere {
    radius_m: f64,
    diffusivity_m2_s: f64,
    lambda_per_s: f64,
}

impl HomogeneousSphere {
    /// `radius` > 0 (m), `diffusivity` >= 0 (m^2 s^-1), `lambda` >= 0 (s^-1).
    pub fn new(
        radius_m: f64,
        diffusivity_m2_s: f64,
        lambda_per_s: f64,
    ) -> Result<Self, PhysicsError> {
        positive("radius", radius_m)?;
        non_negative("diffusivity", diffusivity_m2_s)?;
        non_negative("decay constant", lambda_per_s)?;
        Ok(Self {
            radius_m,
            diffusivity_m2_s,
            lambda_per_s,
        })
    }

    /// Diffusion rate of mode `n`: `a_n = n^2 pi^2 D / R^2` (s^-1).
    fn mode_rate(&self, n: usize) -> f64 {
        let k = n as f64 * PI;
        k * k * self.diffusivity_m2_s / (self.radius_m * self.radius_m)
    }

    fn check(t: f64, n_terms: usize) -> Result<(), PhysicsError> {
        non_negative("time", t)?;
        if n_terms == 0 {
            return Err(PhysicsError::NoSeriesTerms);
        }
        Ok(())
    }

    /// Fraction of the initial inventory still inside (not decayed, not released):
    /// `(6 / pi^2) exp(-lambda t) sum_n exp(-a_n t) / n^2`.
    pub fn retained_fraction(&self, t: f64, n_terms: usize) -> Result<f64, PhysicsError> {
        Self::check(t, n_terms)?;
        let s: f64 = (1..=n_terms)
            .map(|n| (-self.mode_rate(n) * t).exp() / (n as f64 * n as f64))
            .sum();
        Ok(6.0 / (PI * PI) * (-self.lambda_per_s * t).exp() * s)
    }

    /// Fraction of the initial inventory that has left through the surface by diffusion
    /// (counted at the moment it left, so atoms that decayed inside are not released):
    /// `(6 / pi^2) sum_n (a_n / (a_n + lambda)) (1 - exp(-(a_n + lambda) t)) / n^2`.
    pub fn released_fraction(&self, t: f64, n_terms: usize) -> Result<f64, PhysicsError> {
        Self::check(t, n_terms)?;
        Ok(self.partition_sum(t, n_terms, |a, l| a / (a + l)))
    }

    /// Fraction of the initial inventory lost by decay inside the sphere:
    /// `(6 / pi^2) sum_n (lambda / (a_n + lambda)) (1 - exp(-(a_n + lambda) t)) / n^2`.
    /// Kept separate from the released fraction. For `lambda = 0` it is 0.
    pub fn decayed_fraction(&self, t: f64, n_terms: usize) -> Result<f64, PhysicsError> {
        Self::check(t, n_terms)?;
        Ok(self.partition_sum(t, n_terms, |a, l| l / (a + l)))
    }

    fn partition_sum(&self, t: f64, n_terms: usize, share: impl Fn(f64, f64) -> f64) -> f64 {
        let l = self.lambda_per_s;
        let s: f64 = (1..=n_terms)
            .map(|n| {
                let a = self.mode_rate(n);
                let total = a + l;
                if total == 0.0 {
                    0.0
                } else {
                    share(a, l) * (-(-total * t).exp_m1()) / (n as f64 * n as f64)
                }
            })
            .sum();
        6.0 / (PI * PI) * s
    }

    /// Concentration ratio `C(r, t) / C_init` at radius `r` in `[0, R]`:
    /// `2 exp(-lambda t) sum_n (-1)^(n+1) sinc(n pi r / R) exp(-a_n t)` with
    /// `sinc(x) = sin(x) / x`, `sinc(0) = 1` (so the centre needs no special case).
    /// Gibbs oscillation near `t = 0` needs many terms; the pointwise tail is not bounded
    /// by [`series_terms_for_tolerance`], which covers the inventory fractions only.
    pub fn concentration_ratio(&self, r: f64, t: f64, n_terms: usize) -> Result<f64, PhysicsError> {
        Self::check(t, n_terms)?;
        non_negative("radius coordinate", r)?;
        if r > self.radius_m {
            return Err(PhysicsError::OutOfRange("radius coordinate above R"));
        }
        let s: f64 = (1..=n_terms)
            .map(|n| {
                let x = n as f64 * PI * r / self.radius_m;
                let sinc = if x == 0.0 { 1.0 } else { x.sin() / x };
                let sign = if n % 2 == 1 { 1.0 } else { -1.0 };
                sign * sinc * (-self.mode_rate(n) * t).exp()
            })
            .sum();
        Ok(2.0 * (-self.lambda_per_s * t).exp() * s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rel(a: f64, b: f64) -> f64 {
        ((a - b) / b).abs()
    }

    // ---- well-mixed references -------------------------------------------------------

    #[test]
    fn decay_only_hand_values() {
        let l = std::f64::consts::LN_2 / 10.0; // half-life 10 s
        assert_eq!(decay_only(8.0, l, 0.0).unwrap(), 8.0);
        assert!(rel(decay_only(8.0, l, 10.0).unwrap(), 4.0) < 1e-15);
        assert!(rel(decay_only(8.0, l, 30.0).unwrap(), 1.0) < 1e-15);
        // Limits: lambda = 0 keeps C_init; C_init = 0 stays 0.
        assert_eq!(decay_only(5.0, 0.0, 1.0e9).unwrap(), 5.0);
        assert_eq!(decay_only(0.0, l, 10.0).unwrap(), 0.0);
        assert!(decay_only(8.0, l, 1.0e6).unwrap() < 1e-300);
    }

    #[test]
    fn source_only_hand_values() {
        assert_eq!(source_only(0.0, 3.0, 4.0).unwrap(), 12.0);
        assert_eq!(source_only(1.0, 3.0, 4.0).unwrap(), 13.0);
        assert_eq!(source_only(2.0, 0.0, 4.0).unwrap(), 2.0);
        assert_eq!(source_only(2.0, 3.0, 0.0).unwrap(), 2.0);
    }

    #[test]
    fn source_with_decay_hand_values_and_limits() {
        // G = 2, lambda = ln2 / 10: at one half-life, C = (G / lambda) / 2 from C_init = 0.
        let l = std::f64::consts::LN_2 / 10.0;
        let c = source_with_decay(0.0, 2.0, l, 10.0).unwrap();
        assert!(rel(c, 0.5 * 2.0 / l) < 1e-15);
        // Three half-lives: (G / lambda)(1 - 1/8).
        let c3 = source_with_decay(0.0, 2.0, l, 30.0).unwrap();
        assert!(rel(c3, 0.875 * 2.0 / l) < 1e-14);
        // Approach to G / lambda.
        let eq = equilibrium_concentration(2.0, l).unwrap();
        assert!(rel(source_with_decay(0.0, 2.0, l, 1000.0).unwrap(), eq) < 1e-15);
        // Starting at equilibrium stays there.
        assert!(rel(source_with_decay(eq, 2.0, l, 17.0).unwrap(), eq) < 1e-14);
        // lambda = 0 is source only, exactly.
        assert_eq!(source_with_decay(1.0, 3.0, 0.0, 4.0).unwrap(), 13.0);
        // G = 0 is decay only, exactly.
        assert_eq!(
            source_with_decay(8.0, 0.0, l, 10.0).unwrap(),
            decay_only(8.0, l, 10.0).unwrap()
        );
        // lambda t tiny: first order G t (1 - lambda t / 2).
        let (lam, t) = (1.0e-12, 1.0);
        assert!(
            rel(
                source_with_decay(0.0, 1.0, lam, t).unwrap(),
                t * (1.0 - lam * t / 2.0)
            ) < 1e-15
        );
        // Small lambda continuity with the source-only line.
        assert!(rel(source_with_decay(0.0, 1.0, 1.0e-300, 5.0).unwrap(), 5.0) < 1e-15);
    }

    #[test]
    fn source_with_decay_satisfies_its_ode() {
        // dC/dt = G - lambda C, checked by a centred difference of the closed form.
        let (c_init, g, l) = (0.7, 1.3, 0.21);
        let t = 2.4;
        let h = 1.0e-5;
        let d = (source_with_decay(c_init, g, l, t + h).unwrap()
            - source_with_decay(c_init, g, l, t - h).unwrap())
            / (2.0 * h);
        let c = source_with_decay(c_init, g, l, t).unwrap();
        assert!((d - (g - l * c)).abs() < 1e-9);
    }

    #[test]
    fn well_mixed_invalid_inputs() {
        assert!(decay_only(-1.0, 1.0, 1.0).is_err());
        assert!(decay_only(1.0, -1.0, 1.0).is_err());
        assert!(decay_only(1.0, 1.0, -1.0).is_err());
        assert!(source_only(1.0, f64::NAN, 1.0).is_err());
        assert!(source_with_decay(1.0, 1.0, 1.0, f64::INFINITY).is_err());
        assert!(equilibrium_concentration(1.0, 0.0).is_err());
    }

    // ---- homogeneous sphere ----------------------------------------------------------

    fn sphere(lambda: f64) -> HomogeneousSphere {
        // R = 1 m, D = 1 m^2/s: a_n = n^2 pi^2 s^-1, tau = 1 / pi^2 s.
        HomogeneousSphere::new(1.0, 1.0, lambda).unwrap()
    }

    #[test]
    fn terms_for_tolerance_is_ceil_formula() {
        let n = series_terms_for_tolerance(1.0e-6).unwrap();
        assert_eq!(n, (6.0e6 / (PI * PI)).ceil() as usize);
        assert_eq!(series_terms_for_tolerance(10.0).unwrap(), 1);
        assert!(series_terms_for_tolerance(0.0).is_err());
        // Beyond the documented limit the request is refused, never saturated.
        assert!(matches!(
            series_terms_for_tolerance(1.0e-300),
            Err(PhysicsError::OutOfRange(_))
        ));
        // 6 / (pi^2 tol) = 1e9 at tol = 6e-10 / pi^2 is the boundary: just inside succeeds.
        assert!(series_terms_for_tolerance(1.0e-9).is_ok());
        assert!(series_terms_for_tolerance(1.0e-10).is_err());
    }

    #[test]
    fn sphere_t0_all_retained_within_tail_bound() {
        let n = series_terms_for_tolerance(1.0e-6).unwrap();
        let s = sphere(0.3);
        let r = s.retained_fraction(0.0, n).unwrap();
        assert!((1.0 - r) >= 0.0 && (1.0 - r) <= 1.0e-6);
        assert_eq!(s.released_fraction(0.0, n).unwrap(), 0.0);
        assert_eq!(s.decayed_fraction(0.0, n).unwrap(), 0.0);
    }

    #[test]
    fn sphere_late_time_is_first_mode_times_decay() {
        // t = 2: mode n = 2 is below exp(-3 pi^2 t) / 4 = 4e-27 relative to mode 1.
        let l = 0.25;
        let s = sphere(l);
        let t = 2.0;
        let expect = 6.0 / (PI * PI) * (-(PI * PI + l) * t).exp();
        assert!(rel(s.retained_fraction(t, 200).unwrap(), expect) < 1e-14);
        // Centre concentration ratio: 2 exp(-(pi^2 + lambda) t) (sinc(0) = 1, n = 1).
        let c = s.concentration_ratio(0.0, t, 200).unwrap();
        assert!(rel(c, 2.0 * (-(PI * PI + l) * t).exp()) < 1e-14);
    }

    #[test]
    fn sphere_no_diffusion_limit_is_pure_decay() {
        // D = 0: nothing leaves, so retained = exp(-lambda t) times the truncated 6/pi^2 sum
        // (which tends to 1) and the released fraction is 0.
        let s = HomogeneousSphere::new(1.0, 0.0, 0.7).unwrap();
        let n = series_terms_for_tolerance(1.0e-7).unwrap();
        let r = s.retained_fraction(2.0, n).unwrap();
        assert!(((-1.4f64).exp() - r).abs() < 1.0e-7);
        assert_eq!(s.released_fraction(2.0, n).unwrap(), 0.0);
        let dec = s.decayed_fraction(2.0, n).unwrap();
        assert!(((1.0 - (-1.4f64).exp()) - dec).abs() < 1.0e-7);
    }

    #[test]
    fn sphere_no_decay_limit_is_booth_and_decay_loss_is_zero() {
        let s = sphere(0.0);
        let n = series_terms_for_tolerance(1.0e-6).unwrap();
        // Booth, long time: released = 1 - (6/pi^2) exp(-pi^2 t) to the second mode.
        let t = 0.5;
        let f = s.released_fraction(t, n).unwrap();
        let r = s.retained_fraction(t, n).unwrap();
        assert!((f + r - 1.0).abs() < 1.0e-6);
        assert!((r - 6.0 / (PI * PI) * (-PI * PI * t).exp()).abs() < 1.0e-3 * r);
        assert_eq!(s.decayed_fraction(t, n).unwrap(), 0.0);
        // Short time, small-t law F = 6 sqrt(D t / (pi R^2)) - 3 D t / R^2 (leading terms).
        let t = 1.0e-4;
        let f = s.released_fraction(t, 2_000_000).unwrap();
        let approx = 6.0 * (t / PI).sqrt() - 3.0 * t;
        assert!((f - approx).abs() < 1.0e-4 * f);
    }

    #[test]
    fn sphere_partition_adds_up_termwise() {
        // retained + released + decayed is the truncated sum (6/pi^2) sum_{n<=N} 1/n^2 for
        // every t, every lambda: each term of the three sums adds to 1/n^2.
        for (lam, t) in [(0.0, 0.3), (0.5, 0.02), (5.0, 1.0), (1.0e-6, 3.0)] {
            let s = sphere(lam);
            let n = 50;
            let total = s.retained_fraction(t, n).unwrap()
                + s.released_fraction(t, n).unwrap()
                + s.decayed_fraction(t, n).unwrap();
            let trunc: f64 =
                (1..=n).map(|k| 1.0 / (k as f64 * k as f64)).sum::<f64>() * 6.0 / (PI * PI);
            assert!(rel(total, trunc) < 1e-13, "lambda {lam} t {t}");
        }
    }

    #[test]
    fn sphere_decay_loss_equals_lambda_times_integral_of_retained() {
        // d(decayed)/dt = lambda * retained, by a centred difference of the closed form.
        let s = sphere(0.8);
        let (t, h, n) = (0.05, 1.0e-6, 400);
        let d = (s.decayed_fraction(t + h, n).unwrap() - s.decayed_fraction(t - h, n).unwrap())
            / (2.0 * h);
        assert!(rel(d, 0.8 * s.retained_fraction(t, n).unwrap()) < 1e-6);
    }

    #[test]
    fn sphere_profile_integrates_to_retained_fraction() {
        // Independent check: Simpson quadrature of 4 pi r^2 C(r, t) / (4/3 pi R^3 C_init).
        let s = sphere(0.4);
        let t = 0.08;
        let n = 400;
        let m = 2000; // even number of Simpson intervals
        let h = 1.0 / m as f64;
        let f = |r: f64| 3.0 * r * r * s.concentration_ratio(r, t, n).unwrap();
        let mut sum = f(0.0) + f(1.0);
        for i in 1..m {
            sum += f(i as f64 * h) * if i % 2 == 1 { 4.0 } else { 2.0 };
        }
        let integral = sum * h / 3.0;
        assert!(rel(integral, s.retained_fraction(t, n).unwrap()) < 1e-8);
        // The surface value is zero (absorbing), to rounding.
        assert!(s.concentration_ratio(1.0, t, n).unwrap().abs() < 1e-12);
    }

    #[test]
    fn sphere_profile_scales_with_exp_minus_lambda_t() {
        // C(r, t; lambda) = exp(-lambda t) C(r, t; 0).
        let (a, b) = (sphere(0.9), sphere(0.0));
        let (r, t) = (0.37, 0.2);
        let ca = a.concentration_ratio(r, t, 300).unwrap();
        let cb = b.concentration_ratio(r, t, 300).unwrap();
        assert!(rel(ca, (-0.9 * t).exp() * cb) < 1e-14);
    }

    #[test]
    fn sphere_invalid_inputs() {
        assert!(HomogeneousSphere::new(0.0, 1.0, 0.0).is_err());
        assert!(HomogeneousSphere::new(1.0, -1.0, 0.0).is_err());
        assert!(HomogeneousSphere::new(1.0, 1.0, -1.0).is_err());
        let s = sphere(0.0);
        assert_eq!(
            s.retained_fraction(1.0, 0),
            Err(PhysicsError::NoSeriesTerms)
        );
        assert!(s.retained_fraction(-1.0, 5).is_err());
        assert!(s.concentration_ratio(1.5, 0.1, 5).is_err());
        assert!(s.concentration_ratio(-0.1, 0.1, 5).is_err());
    }
}
