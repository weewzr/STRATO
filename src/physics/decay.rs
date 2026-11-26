//! Radioactive decay constant and half-life: `lambda = ln 2 / t_half`.
//!
//! The decay constant is in s^-1 and the half-life is converted to seconds first. A year is
//! not a unit with one length (sources use different ones), so the caller passes the number
//! of seconds per year explicitly with [`TimeUnit::Years`]. Decay is a loss of atoms and is
//! tracked separately from physical release by the callers.

use super::{positive, PhysicsError};

/// Seconds in a Julian year (365.25 d), a convenience for [`TimeUnit::Years`]. Sources that
/// use another year length must pass their own value.
pub const JULIAN_YEAR_S: f64 = 365.25 * 86_400.0;

/// Unit of a half-life.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeUnit {
    /// Seconds.
    Seconds,
    /// Minutes (60 s).
    Minutes,
    /// Hours (3600 s).
    Hours,
    /// Days (86 400 s).
    Days,
    /// Years of the stated length in seconds.
    Years(f64),
}

impl TimeUnit {
    /// Seconds in one of this unit.
    pub fn seconds(self) -> Result<f64, PhysicsError> {
        match self {
            Self::Seconds => Ok(1.0),
            Self::Minutes => Ok(60.0),
            Self::Hours => Ok(3600.0),
            Self::Days => Ok(86_400.0),
            Self::Years(s) => positive("seconds per year", s),
        }
    }
}

/// Decay constant in s^-1 from a half-life in `unit`. The half-life must be finite and
/// strictly positive (a stable nuclide has `lambda = 0` and no half-life; callers handle it).
pub fn decay_constant_from_half_life(half_life: f64, unit: TimeUnit) -> Result<f64, PhysicsError> {
    positive("half-life", half_life)?;
    let t_s = positive("half-life (s)", half_life * unit.seconds()?)?;
    Ok(std::f64::consts::LN_2 / t_s)
}

/// Half-life in seconds from a decay constant in s^-1 (finite, strictly positive).
pub fn half_life_seconds_from_decay_constant(lambda_per_s: f64) -> Result<f64, PhysicsError> {
    positive("decay constant", lambda_per_s)?;
    positive("half-life (s)", std::f64::consts::LN_2 / lambda_per_s)
}

/// Half-life expressed in `unit` from a decay constant in s^-1.
pub fn half_life_from_decay_constant(
    lambda_per_s: f64,
    unit: TimeUnit,
) -> Result<f64, PhysicsError> {
    Ok(half_life_seconds_from_decay_constant(lambda_per_s)? / unit.seconds()?)
}

/// Mean life `1 / lambda` in seconds.
pub fn mean_life_seconds(lambda_per_s: f64) -> Result<f64, PhysicsError> {
    positive("decay constant", lambda_per_s).map(|l| 1.0 / l)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::LN_2;

    fn rel(a: f64, b: f64) -> f64 {
        ((a - b) / b).abs()
    }

    #[test]
    fn one_second_half_life_gives_ln2() {
        let l = decay_constant_from_half_life(1.0, TimeUnit::Seconds).unwrap();
        // exp(-ln 2) = 1/2, the definition of the half-life.
        assert!(((-l).exp() - 0.5).abs() < 1e-16);
        assert_eq!(l, LN_2);
    }

    #[test]
    fn units_are_converted() {
        // 1 min = 60 s, 1 h = 3600 s, 1 d = 86400 s: the same decay constant per equal time.
        let by_s = decay_constant_from_half_life(86_400.0, TimeUnit::Seconds).unwrap();
        for (v, u) in [
            (1440.0, TimeUnit::Minutes),
            (24.0, TimeUnit::Hours),
            (1.0, TimeUnit::Days),
        ] {
            assert!(rel(decay_constant_from_half_life(v, u).unwrap(), by_s) < 1e-15);
        }
        // Julian year: 31 557 600 s.
        let y = decay_constant_from_half_life(1.0, TimeUnit::Years(JULIAN_YEAR_S)).unwrap();
        assert!(rel(y, LN_2 / 31_557_600.0) < 1e-15);
        // A different year length changes the result in proportion.
        let y2 = decay_constant_from_half_life(1.0, TimeUnit::Years(3.0e7)).unwrap();
        assert!(rel(y2, LN_2 / 3.0e7) < 1e-15);
    }

    #[test]
    fn hand_value_ten_seconds() {
        // ln 2 / 10 = 0.0693147180559945 s^-1; units: s^-1.
        let l = decay_constant_from_half_life(10.0, TimeUnit::Seconds).unwrap();
        assert!(rel(l, 0.069_314_718_055_994_5) < 1e-15);
    }

    #[test]
    fn round_trip_both_directions() {
        for t in [1.0e-3, 0.5, 24.56, 2.4985e7, 9.49e8] {
            let l = decay_constant_from_half_life(t, TimeUnit::Seconds).unwrap();
            let back = half_life_seconds_from_decay_constant(l).unwrap();
            assert!(rel(back, t) < 4e-16 * 4.0);
        }
        let l = 7.5e-10;
        let h = half_life_from_decay_constant(l, TimeUnit::Days).unwrap();
        let l2 = decay_constant_from_half_life(h, TimeUnit::Days).unwrap();
        assert!(rel(l2, l) < 1e-15);
    }

    #[test]
    fn half_life_is_the_time_the_amount_halves() {
        let l = decay_constant_from_half_life(3.0, TimeUnit::Hours).unwrap();
        let t = 3.0 * 3600.0;
        assert!(((-l * t).exp() - 0.5).abs() < 1e-15);
        assert!(rel(mean_life_seconds(l).unwrap(), t / LN_2) < 1e-15);
    }

    #[test]
    fn invalid_inputs_are_refused() {
        assert!(decay_constant_from_half_life(0.0, TimeUnit::Seconds).is_err());
        assert!(decay_constant_from_half_life(-1.0, TimeUnit::Days).is_err());
        assert!(decay_constant_from_half_life(f64::NAN, TimeUnit::Days).is_err());
        assert!(decay_constant_from_half_life(f64::INFINITY, TimeUnit::Days).is_err());
        assert!(decay_constant_from_half_life(1.0, TimeUnit::Years(0.0)).is_err());
        assert!(half_life_seconds_from_decay_constant(0.0).is_err());
        assert!(half_life_seconds_from_decay_constant(-1.0).is_err());
        assert!(mean_life_seconds(0.0).is_err());
    }
}
