//! Classical analytical solution for diffusion in a homogeneous sphere with an absorbing
//! surface. Independent reference for verifying the BTCS solver: this module calls no
//! numerical PDE solver and shares no code with the finite-volume or Sturm-Liouville modules.
//!
//! # Problem
//!
//! Sphere of radius `R`, constant diffusivity `D`, spherical symmetry, no source, no decay:
//!
//! ```text
//! dC/dt = D (1/r^2) d/dr ( r^2 dC/dr ),   0 < r < R,  t > 0
//! C(R, t) = 0                                  (absorbing surface)
//! C(r, 0) = C0 for r < a, 0 for a < r < R      (a <= R)
//! ```
//!
//! Case (a) is the uniform start `a = R`. Case (b) is a concentric core `a < R`.
//!
//! # Derivation (Fourier sine series of `r C`)
//!
//! Put `u(r, t) = r C(r, t)`. Because `(1/r^2)(r^2 C_r)_r = u_rr / r`, the problem becomes
//! the one-dimensional heat equation
//!
//! ```text
//! u_t = D u_rr,   u(0, t) = 0,   u(R, t) = 0,   u(r, 0) = r C0 for r < a, else 0.
//! ```
//!
//! `u(0,t) = 0` holds because `C` is bounded at the centre. Separation of variables gives
//! `sin(k_n r) exp(-D k_n^2 t)` with `k_n = n pi / R`, `n = 1, 2, ...`. Hence
//!
//! ```text
//! u(r, t) = sum_n b_n sin(k_n r) exp(-lambda_n t),   lambda_n = D k_n^2,
//! b_n = (2/R) int_0^a r C0 sin(k_n r) dr = (2 C0 / R) g(k_n a) / k_n^2,
//! g(x) = sin x - x cos x          (from int r sin(k r) dr = (sin kr - kr cos kr)/k^2).
//! ```
//!
//! Concentration:
//!
//! ```text
//! C(r, t) = (1/r) sum_n b_n sin(k_n r) exp(-lambda_n t).
//! ```
//!
//! Centre limit (`sin(k r)/r -> k` as `r -> 0`):
//!
//! ```text
//! C(0, t) = sum_n b_n k_n exp(-lambda_n t).
//! ```
//!
//! Inventory `M(t) = 4 pi int_0^R r^2 C dr = 4 pi int_0^R r u dr`. With
//! `int_0^R r sin(k_n r) dr = (-1)^(n+1) R / k_n`:
//!
//! ```text
//! M(t) = M0 sum_n w_n exp(-lambda_n t),   M0 = 4 pi C0 a^3 / 3,
//! w_n = 6 (-1)^(n+1) g(k_n a) / (k_n a)^3.
//! ```
//!
//! For `a = R`, `g(n pi) = (-1)^(n+1) n pi`, so `w_n = 6 / (n pi)^2` and `sum w_n = 1`.
//! The fractional release is `F(t) = 1 - M(t)/M0`.
//!
//! Surface release rate (outflow), `J = -4 pi R^2 D C_r(R, t)`. Since `u(R) = 0`,
//! `C_r(R) = u_r(R)/R`, and `u_r(R) = sum b_n k_n (-1)^n exp(-lambda_n t)`, so
//!
//! ```text
//! J(t) = 4 pi R D sum_n (-1)^(n+1) b_n k_n exp(-lambda_n t),    dM/dt = -J.
//! ```
//!
//! # Truncation rule and tail error estimate
//!
//! Series are summed to `N` terms. `N` is the smallest integer for which a rigorous bound on
//! the omitted tail is at most the requested tolerance (searched by doubling and bisection;
//! the bound decreases monotonically in `N`). If that needs more than [`MAX_TERMS`] terms
//! (very small `t`) an error is returned instead of a silently inaccurate value.
//!
//! Using `|g(x)| <= 1 + x`, the weights obey `|w_n| <= W(n) = 6 (1 + x_n) / x_n^3`,
//! `x_n = n pi a / R` (`W(n) = 6/(n pi)^2` when `a = R`), and `W` decreases with `n`. With
//! `m = N + 1`, `lambda_n - lambda_m = lambda_1 j (2m + j) >= 2 m lambda_1 j` for `n = m + j`,
//! so the exponentials are dominated by a geometric series of ratio
//! `q = exp(-2 m lambda_1 t)` and, for `t > 0`,
//!
//! ```text
//! |tail of M(t)/M0| <= W(m) exp(-lambda_m t) / (1 - q).
//! ```
//!
//! At `t = 0` the integral bound `sum_{n>N} W(n) <= int_N^inf W` is used. For concentration
//! and release rate the terms obey `|b_n k_n sin(k_n r)/(r k_n)| <= |b_n| k_n <=
//! C0 E(n)` with `E(n) = 2 (1/(n pi) + a/R)` (`E = 2` when `a = R`), giving the same
//! geometric tail with `E(m)` in place of `W(m)`. Concentration errors are reported in
//! concentration units (the bound times `C0`).
//!
//! For `g(x)` at small `x` (small core, low modes) a Taylor series is used to avoid
//! cancellation in `sin x - x cos x`.

use anyhow::{bail, Result};
use std::f64::consts::PI;

/// Largest number of series terms the truncation rule will use.
pub const MAX_TERMS: usize = 5_000_000;

/// A series value together with a rigorous bound on the omitted tail and the term count.
#[derive(Clone, Copy, Debug)]
pub struct Estimate {
    pub value: f64,
    /// Upper bound on the absolute truncation error of `value` (the rounding error is not
    /// included; it is of order 1e-16 times the sum of the absolute terms).
    pub error_bound: f64,
    pub terms: usize,
}

/// Homogeneous sphere, absorbing surface, initial concentration `c0` for `r < core_radius`.
#[derive(Clone, Copy, Debug)]
pub struct HomogeneousSphere {
    radius: f64,
    core_radius: f64,
    diffusivity: f64,
    c0: f64,
}

/// `g(x) = sin x - x cos x`, with a Taylor series for small `x`.
fn g(x: f64) -> f64 {
    if x.abs() < 0.1 {
        let x2 = x * x;
        // x^3/3 - x^5/30 + x^7/840 - x^9/45360 + x^11/3991680
        x * x2
            * (1.0 / 3.0
                - x2 * (1.0 / 30.0 - x2 * (1.0 / 840.0 - x2 * (1.0 / 45360.0 - x2 / 3_991_680.0))))
    } else {
        x.sin() - x * x.cos()
    }
}

fn sign(n: usize) -> f64 {
    // (-1)^(n+1)
    if n % 2 == 1 {
        1.0
    } else {
        -1.0
    }
}

impl HomogeneousSphere {
    /// Case (a): uniform start profile `c0` in the whole sphere.
    pub fn uniform(radius: f64, diffusivity: f64, c0: f64) -> Result<Self> {
        Self::core(radius, radius, diffusivity, c0)
    }

    /// Case (b): start profile `c0` for `r < core_radius`, zero outside
    /// (`0 < core_radius <= radius`; `core_radius == radius` is case (a)).
    pub fn core(radius: f64, core_radius: f64, diffusivity: f64, c0: f64) -> Result<Self> {
        let ok = radius.is_finite()
            && core_radius.is_finite()
            && diffusivity.is_finite()
            && c0.is_finite()
            && radius > 0.0
            && core_radius > 0.0
            && core_radius <= radius
            && diffusivity > 0.0;
        if !ok {
            bail!(
                "need finite radius > 0, 0 < core_radius <= radius, diffusivity > 0 and finite c0"
            );
        }
        Ok(Self {
            radius,
            core_radius,
            diffusivity,
            c0,
        })
    }

    pub fn radius(&self) -> f64 {
        self.radius
    }

    pub fn core_radius(&self) -> f64 {
        self.core_radius
    }

    fn is_uniform(&self) -> bool {
        self.core_radius == self.radius
    }

    /// Wave number `k_n = n pi / R` (1/m).
    pub fn wave_number(&self, n: usize) -> f64 {
        n as f64 * PI / self.radius
    }

    /// Eigenvalue `lambda_n = D k_n^2` (1/s).
    pub fn eigenvalue(&self, n: usize) -> f64 {
        self.diffusivity * self.wave_number(n).powi(2)
    }

    /// Initial inventory `M0 = 4 pi c0 a^3 / 3`.
    pub fn initial_inventory(&self) -> f64 {
        4.0 * PI * self.c0 * self.core_radius.powi(3) / 3.0
    }

    /// Inventory mode weight `w_n` (sum of all weights is 1).
    pub fn weight(&self, n: usize) -> f64 {
        if self.is_uniform() {
            let x = n as f64 * PI;
            return 6.0 / (x * x);
        }
        let x = self.wave_number(n) * self.core_radius;
        6.0 * sign(n) * g(x) / (x * x * x)
    }

    /// `b_n k_n` for mode `n`, in units of `c0`-independent concentration (multiply by nothing:
    /// the value already contains `c0`).
    fn bk(&self, n: usize) -> f64 {
        if self.is_uniform() {
            return 2.0 * self.c0 * sign(n);
        }
        let k = self.wave_number(n);
        2.0 * self.c0 / self.radius * g(k * self.core_radius) / k
    }

    fn weight_bound(&self, m: usize) -> f64 {
        let mf = m as f64;
        if self.is_uniform() {
            6.0 / (mf * PI).powi(2)
        } else {
            let x = mf * PI * self.core_radius / self.radius;
            6.0 * (1.0 + x) / (x * x * x)
        }
    }

    fn coeff_bound(&self, m: usize) -> f64 {
        if self.is_uniform() {
            2.0
        } else {
            2.0 * (1.0 / (m as f64 * PI) + self.core_radius / self.radius)
        }
    }

    /// Rigorous bound on the tail `sum_{n > terms}` of the inventory fraction series at time
    /// `t >= 0`.
    pub fn remaining_tail_bound(&self, t: f64, terms: usize) -> f64 {
        self.tail(t, terms, true)
    }

    fn tail(&self, t: f64, terms: usize, weights: bool) -> f64 {
        let m = terms + 1;
        let coef = if weights {
            self.weight_bound(m)
        } else {
            self.coeff_bound(m)
        };
        if t > 0.0 {
            let l1 = self.eigenvalue(1);
            let lm = self.eigenvalue(m);
            let one_minus_q = -(-2.0 * m as f64 * l1 * t).exp_m1();
            return coef * (-lm * t).exp() / one_minus_q;
        }
        // t = 0: integral bound; only meaningful for the (decaying) weight series.
        if !weights {
            return f64::INFINITY;
        }
        let nf = terms.max(1) as f64;
        if self.is_uniform() {
            6.0 / (PI * PI * nf)
        } else {
            let c = PI * self.core_radius / self.radius;
            6.0 * (1.0 / (2.0 * c.powi(3) * nf * nf) + 1.0 / (c * c * nf))
        }
    }

    /// Smallest `N` with `tail(N) <= tol`, by doubling then bisection.
    fn terms_for(&self, t: f64, tol: f64, weights: bool) -> Result<usize> {
        if !(tol > 0.0 && tol.is_finite()) || !(t >= 0.0 && t.is_finite()) {
            bail!("need finite t >= 0 and a positive finite tolerance");
        }
        let mut hi = 1usize;
        while self.tail(t, hi, weights) > tol {
            if hi >= MAX_TERMS {
                bail!(
                    "tolerance {tol:.1e} at t = {t:.3e} s needs more than {MAX_TERMS} series terms"
                );
            }
            hi = (hi * 2).min(MAX_TERMS);
        }
        let mut lo = hi / 2; // tail(lo) > tol, or lo = 0
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            if self.tail(t, mid, weights) > tol {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Ok(hi)
    }

    /// Number of terms the truncation rule selects for the inventory series at `(t, tol)`.
    pub fn remaining_terms(&self, t: f64, tol: f64) -> Result<usize> {
        self.terms_for(t, tol, true)
    }

    /// Inventory fraction `M(t)/M0` summed to exactly `terms` terms (no error control).
    pub fn remaining_fraction_with_terms(&self, t: f64, terms: usize) -> f64 {
        // Sum from the smallest term (large n) to limit rounding.
        (1..=terms)
            .rev()
            .map(|n| self.weight(n) * (-self.eigenvalue(n) * t).exp())
            .sum()
    }

    /// Remaining fraction `M(t)/M0` with tail error at most `tol` (absolute).
    pub fn remaining_fraction(&self, t: f64, tol: f64) -> Result<Estimate> {
        let terms = self.remaining_terms(t, tol)?;
        Ok(Estimate {
            value: self.remaining_fraction_with_terms(t, terms),
            error_bound: self.tail(t, terms, true),
            terms,
        })
    }

    /// Fractional release `F(t) = 1 - M(t)/M0`, tail error at most `tol`.
    pub fn release_fraction(&self, t: f64, tol: f64) -> Result<Estimate> {
        let e = self.remaining_fraction(t, tol)?;
        Ok(Estimate {
            value: 1.0 - e.value,
            ..e
        })
    }

    /// Remaining inventory `M(t)` in concentration times volume; tail error `tol * M0`.
    pub fn inventory(&self, t: f64, tol: f64) -> Result<Estimate> {
        let m0 = self.initial_inventory();
        let e = self.remaining_fraction(t, tol)?;
        Ok(Estimate {
            value: m0 * e.value,
            error_bound: m0 * e.error_bound,
            terms: e.terms,
        })
    }

    /// Released amount `M0 - M(t)`.
    pub fn released(&self, t: f64, tol: f64) -> Result<Estimate> {
        let m0 = self.initial_inventory();
        let e = self.release_fraction(t, tol)?;
        Ok(Estimate {
            value: m0 * e.value,
            error_bound: m0 * e.error_bound,
            terms: e.terms,
        })
    }

    fn check_r(&self, r: f64) -> Result<()> {
        if !(r.is_finite() && (0.0..=self.radius).contains(&r)) {
            bail!("radius must lie in [0, R]");
        }
        Ok(())
    }

    /// Concentration `C(r, t)`, including the centre limit at `r = 0`, with truncation error
    /// at most `tol` in concentration units. At `t = 0` the initial step profile is returned
    /// exactly (value `c0` for `r < a`, `c0/2` at `r = a < R`, 0 beyond, 0 at `r = R`).
    pub fn concentration(&self, r: f64, t: f64, tol: f64) -> Result<Estimate> {
        self.check_r(r)?;
        if t == 0.0 {
            let value = if r == self.radius {
                0.0
            } else if r < self.core_radius {
                self.c0
            } else if r == self.core_radius {
                0.5 * self.c0
            } else {
                0.0
            };
            return Ok(Estimate {
                value,
                error_bound: 0.0,
                terms: 0,
            });
        }
        let tol_rel = tol / self.c0.abs().max(f64::MIN_POSITIVE);
        let terms = self.terms_for(t, tol_rel, false)?;
        let value = (1..=terms)
            .rev()
            .map(|n| {
                let k = self.wave_number(n);
                let s = if r == 0.0 { k } else { (k * r).sin() / r };
                self.bk(n) / k * s * (-self.eigenvalue(n) * t).exp()
            })
            .sum();
        Ok(Estimate {
            value,
            error_bound: self.c0.abs() * self.tail(t, terms, false),
            terms,
        })
    }

    /// Outflow rate through the surface, `J(t) = -4 pi R^2 D dC/dr (R, t) = -dM/dt`, for
    /// `t > 0`, truncation error at most `tol * 4 pi R D` (the bound is reported scaled).
    pub fn surface_release_rate(&self, t: f64, tol: f64) -> Result<Estimate> {
        if t.is_nan() || t <= 0.0 {
            bail!("surface release rate needs t > 0");
        }
        let scale = 4.0 * PI * self.radius * self.diffusivity;
        let tol_rel = tol / (scale * self.c0.abs().max(f64::MIN_POSITIVE));
        let terms = self.terms_for(t, tol_rel, false)?;
        let sum: f64 = (1..=terms)
            .rev()
            .map(|n| sign(n) * self.bk(n) * (-self.eigenvalue(n) * t).exp())
            .sum();
        Ok(Estimate {
            value: scale * sum,
            error_bound: scale * self.c0.abs() * self.tail(t, terms, false),
            terms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytical::sturm_liouville::{Layer, ModalRelease};

    const R: f64 = 4.5e-4;
    const D: f64 = 1.0e-10;

    fn simpson(a: f64, b: f64, n: usize, f: impl Fn(f64) -> f64) -> f64 {
        let h = (b - a) / n as f64;
        let mut s = f(a) + f(b);
        for i in 1..n {
            s += f(a + i as f64 * h) * if i % 2 == 1 { 4.0 } else { 2.0 };
        }
        s * h / 3.0
    }

    fn tau() -> f64 {
        R * R / (D * PI * PI) // 1 / lambda_1
    }

    #[test]
    fn constructor_rejects_bad_input() {
        assert!(HomogeneousSphere::core(R, 2.0 * R, D, 1.0).is_err());
        assert!(HomogeneousSphere::core(R, 0.0, D, 1.0).is_err());
        assert!(HomogeneousSphere::uniform(R, -1.0, 1.0).is_err());
        assert!(HomogeneousSphere::uniform(f64::NAN, D, 1.0).is_err());
    }

    #[test]
    fn weights_sum_to_one_within_tail_bound_at_t0() {
        for a in [R, 2.0e-4, 5.0e-5] {
            let s = HomogeneousSphere::core(R, a, D, 3.0).unwrap();
            for n_terms in [1000usize, 100_000] {
                let sum: f64 = (1..=n_terms).rev().map(|n| s.weight(n)).sum();
                let bound = s.remaining_tail_bound(0.0, n_terms);
                assert!(
                    (sum - 1.0).abs() <= bound,
                    "a={a} N={n_terms}: |{sum}-1| > {bound}"
                );
            }
        }
    }

    #[test]
    fn t0_inventory_and_profile() {
        let s = HomogeneousSphere::core(R, 2.0e-4, D, 5.0).unwrap();
        let m0 = 4.0 * PI * 5.0 * 2.0e-4f64.powi(3) / 3.0;
        assert!((s.initial_inventory() - m0).abs() < 1e-30);
        assert_eq!(s.concentration(0.0, 0.0, 1e-9).unwrap().value, 5.0);
        assert_eq!(s.concentration(1.0e-4, 0.0, 1e-9).unwrap().value, 5.0);
        assert_eq!(s.concentration(3.0e-4, 0.0, 1e-9).unwrap().value, 0.0);
        let u = HomogeneousSphere::uniform(R, D, 1.0).unwrap();
        // early-time boundary-layer law F ~ 6 sqrt(D t / (pi R^2)) for the uniform sphere
        let t = 1.0e-3 * tau();
        let f = u.release_fraction(t, 1e-10).unwrap().value;
        let approx = 6.0 * (D * t / (PI * R * R)).sqrt() - 3.0 * D * t / (R * R);
        assert!((f - approx).abs() < 2e-3 * f, "{f} vs {approx}");
    }

    #[test]
    fn late_time_decays_with_first_mode_rate() {
        for a in [R, 2.0e-4] {
            let s = HomogeneousSphere::core(R, a, D, 1.0).unwrap();
            let (t1, t2) = (6.0 * tau(), 8.0 * tau());
            let m1 = s.remaining_fraction(t1, 1e-14).unwrap().value;
            let m2 = s.remaining_fraction(t2, 1e-14).unwrap().value;
            let rate = (m1 / m2).ln() / (t2 - t1);
            let l1 = s.eigenvalue(1);
            assert!((rate / l1 - 1.0).abs() < 1e-5, "a={a}: {rate} vs {l1}");
            // amplitude: M/M0 -> w_1 exp(-lambda_1 t)
            let amp = m2 / (-l1 * t2).exp();
            assert!((amp / s.weight(1) - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn release_is_in_unit_interval_and_increasing() {
        for a in [R, 3.0e-4, 1.0e-4] {
            let s = HomogeneousSphere::core(R, a, D, 1.0).unwrap();
            let mut last = 0.0;
            for i in 1..=80 {
                let t = tau() * 1e-3 * 1.2f64.powi(i);
                let f = s.release_fraction(t, 1e-12).unwrap().value;
                assert!((-1e-12..=1.0 + 1e-12).contains(&f), "a={a} t={t} F={f}");
                assert!(f >= last - 1e-12, "a={a} t={t}: {f} < {last}");
                last = f;
            }
            assert!(last > 1.0 - 1e-6);
        }
    }

    #[test]
    fn core_equal_radius_matches_uniform_case() {
        // The generic (a < R) formulas, evaluated with a a hair below R, must reproduce the
        // closed-form uniform case.
        let u = HomogeneousSphere::uniform(R, D, 2.0).unwrap();
        let c = HomogeneousSphere::core(R, R * (1.0 - 1e-12), D, 2.0).unwrap();
        assert!(!c.is_uniform());
        for t in [0.05 * tau(), 0.5 * tau(), 2.0 * tau()] {
            let (fu, fc) = (
                u.release_fraction(t, 1e-13).unwrap().value,
                c.release_fraction(t, 1e-13).unwrap().value,
            );
            assert!((fu - fc).abs() < 1e-9, "t={t}: {fu} vs {fc}");
            for r in [0.0, 0.3 * R, 0.9 * R] {
                let (cu, cc) = (
                    u.concentration(r, t, 1e-13).unwrap().value,
                    c.concentration(r, t, 1e-13).unwrap().value,
                );
                assert!((cu - cc).abs() < 1e-9, "t={t} r={r}: {cu} vs {cc}");
            }
        }
    }

    #[test]
    fn centre_limit_is_continuous() {
        for a in [R, 2.0e-4] {
            let s = HomogeneousSphere::core(R, a, D, 1.0).unwrap();
            let t = 0.2 * tau();
            let c0 = s.concentration(0.0, t, 1e-13).unwrap().value;
            let near = s.concentration(1e-9 * R, t, 1e-13).unwrap().value;
            assert!(c0.is_finite() && (c0 - near).abs() < 1e-9, "{c0} {near}");
            let surf = s.concentration(R, t, 1e-13).unwrap().value;
            assert!(surf.abs() < 1e-12);
            // centre value is the maximum
            let mid = s.concentration(0.5 * R, t, 1e-13).unwrap().value;
            assert!(c0 >= mid);
        }
    }

    #[test]
    fn concentration_is_nonnegative() {
        // The solution of the heat equation with nonnegative data is nonnegative.
        let s = HomogeneousSphere::core(R, 2.0e-4, D, 1.0).unwrap();
        for t in [0.01 * tau(), 0.1 * tau(), tau()] {
            for i in 0..=100 {
                let c = s
                    .concentration(R * i as f64 / 100.0, t, 1e-12)
                    .unwrap()
                    .value;
                assert!(c >= -2e-12, "t={t} i={i} c={c}");
            }
        }
    }

    #[test]
    fn inventory_integral_matches_series_inventory() {
        for a in [R, 2.0e-4] {
            let s = HomogeneousSphere::core(R, a, D, 1.5).unwrap();
            for t in [0.05 * tau(), 0.5 * tau()] {
                let quad = simpson(0.0, R, 4000, |r| {
                    4.0 * PI * r * r * s.concentration(r, t, 1e-14).unwrap().value
                });
                let series = s.inventory(t, 1e-14).unwrap().value;
                let m0 = s.initial_inventory();
                assert!(
                    (quad - series).abs() < 1e-8 * m0,
                    "a={a} t={t}: {quad} vs {series}"
                );
            }
        }
    }

    #[test]
    fn mass_balance_release_equals_integrated_surface_flux() {
        for a in [R, 2.0e-4] {
            let s = HomogeneousSphere::core(R, a, D, 1.5).unwrap();
            let m0 = s.initial_inventory();
            let (t1, t2) = (0.02 * tau(), 0.8 * tau());
            let dm = s.inventory(t1, 1e-14).unwrap().value - s.inventory(t2, 1e-14).unwrap().value;
            let integral = simpson(t1, t2, 2000, |t| {
                s.surface_release_rate(t, 1e-14 * m0).unwrap().value
            });
            assert!(
                (dm - integral).abs() < 1e-8 * m0,
                "a={a}: {dm} vs {integral}"
            );
            // released + remaining = M0 exactly by construction of the two series
            let rel = s.released(t2, 1e-14).unwrap().value;
            let rem = s.inventory(t2, 1e-14).unwrap().value;
            assert!((rel + rem - m0).abs() < 1e-14 * m0);
        }
    }

    #[test]
    fn tail_bound_covers_actual_truncation_error() {
        for a in [R, 2.0e-4, 5.0e-5] {
            let s = HomogeneousSphere::core(R, a, D, 1.0).unwrap();
            for t in [0.001 * tau(), 0.01 * tau(), 0.1 * tau()] {
                let reference = s.remaining_fraction_with_terms(t, 20_000);
                for n in [3usize, 10, 40] {
                    let part = s.remaining_fraction_with_terms(t, n);
                    let bound = s.remaining_tail_bound(t, n);
                    assert!(
                        (part - reference).abs() <= bound + 1e-15,
                        "a={a} t={t} N={n}: {} > {bound}",
                        (part - reference).abs()
                    );
                }
                let e = s.remaining_fraction(t, 1e-10).unwrap();
                assert!(e.error_bound <= 1e-10 && (e.value - reference).abs() <= 1e-10);
            }
        }
    }

    #[test]
    fn unreachable_tolerance_is_an_error_not_a_wrong_number() {
        let s = HomogeneousSphere::uniform(R, D, 1.0).unwrap();
        assert!(s.remaining_fraction(1e-20 * tau(), 1e-14).is_err());
        assert!(s.release_fraction(-1.0, 1e-6).is_err());
        assert!(s.concentration(2.0 * R, 1.0, 1e-6).is_err());
    }

    #[test]
    fn agrees_with_sturm_liouville_for_equal_layer_diffusivities() {
        let layers: Vec<Layer> = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4]
            .iter()
            .map(|&r| Layer {
                outer_radius: r,
                diffusivity: D,
            })
            .collect();
        let series = ModalRelease::new(&layers, 45.0 / 20.0, 20_000).unwrap();
        let mine = HomogeneousSphere::core(4.5e-4, 2.0e-4, D, 1.0).unwrap();
        for t in [20.0, 100.0, 500.0, 2000.0] {
            let a = series.release_fraction(t);
            let b = mine.release_fraction(t, 1e-13).unwrap().value;
            assert!((a - b).abs() < 1e-9, "t={t}: SL {a} vs closed form {b}");
        }
    }
}
