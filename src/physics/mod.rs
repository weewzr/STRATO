//! Property functions and closed-form references for the Phase G mechanisms
//! (directive section 17): temperature-dependent diffusivity, radioactive decay and the
//! volumetric fission-product source.
//!
//! Everything here is a pure function of its arguments in `f64`: no file access, no global
//! state, no data values. Parameters (D0, Ea, half-lives, yields) are always supplied by the
//! caller; the only constants stored are the exact SI constants (Avogadro, Boltzmann) and the
//! exact unit conversions (minute, hour, day, kelvin offset). The module has no dependency
//! beyond `std`, so it compiles unchanged for `wasm32-unknown-unknown`.
//!
//! The derivations are in `docs/mathematics/STRATO_Property_Functions.md`.
//!
//! - [`arrhenius`]: `D(T) = sum_m D0_m exp(-Ea_m / (R T))`, unit conversions, validity-range
//!   report (never a clamp);
//! - [`decay`]: `lambda = ln 2 / t_half` and the reverse, with time units;
//! - [`source`]: `S = F_dot y / N_A`, with typed yield and product kinds;
//! - [`reference`]: closed-form solutions for later verification of the coupled solver.
//!
//! Nothing in this module is coupled into a solver.

pub mod arrhenius;
pub mod decay;
pub mod reference;
pub mod source;

use std::fmt;

/// Avogadro constant in mol^-1 (exact since the 2019 SI revision).
pub const AVOGADRO: f64 = 6.022_140_76e23;

/// Boltzmann constant in J K^-1 (exact since the 2019 SI revision).
pub const BOLTZMANN: f64 = 1.380_649e-23;

/// Offset between degrees Celsius and kelvin (exact by definition).
pub const CELSIUS_OFFSET_K: f64 = 273.15;

/// Reasons a property function refuses its input. A refusal never produces a number.
#[derive(Debug, Clone, PartialEq)]
pub enum PhysicsError {
    /// The named argument is NaN or infinite.
    NotFinite(&'static str),
    /// The named argument must be strictly positive.
    NotPositive(&'static str),
    /// The named argument must be zero or positive.
    Negative(&'static str),
    /// A direct fission-yield source was requested for an activation product
    /// (Ag-110m, Cs-134, Eu-154 and similar): they are made by neutron capture, so a
    /// production model is required and none is approved yet.
    DirectYieldForActivationProduct,
    /// The named argument lies outside its domain.
    OutOfRange(&'static str),
    /// The temperature range has its minimum above its maximum.
    EmptyTemperatureRange,
    /// An Arrhenius law needs at least one term.
    NoTerms,
    /// A series needs at least one term.
    NoSeriesTerms,
}

impl fmt::Display for PhysicsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFinite(n) => write!(f, "{n} must be finite"),
            Self::NotPositive(n) => write!(f, "{n} must be strictly positive"),
            Self::Negative(n) => write!(f, "{n} must not be negative"),
            Self::DirectYieldForActivationProduct => write!(
                f,
                "a direct fission-yield source is refused for an activation product; \
                 it needs a neutron-capture production model"
            ),
            Self::OutOfRange(n) => write!(f, "{n}"),
            Self::EmptyTemperatureRange => write!(f, "temperature range has min above max"),
            Self::NoTerms => write!(f, "an Arrhenius law needs at least one term"),
            Self::NoSeriesTerms => write!(f, "a series needs at least one term"),
        }
    }
}

impl std::error::Error for PhysicsError {}

pub(crate) fn finite(name: &'static str, x: f64) -> Result<f64, PhysicsError> {
    if x.is_finite() {
        Ok(x)
    } else {
        Err(PhysicsError::NotFinite(name))
    }
}

pub(crate) fn positive(name: &'static str, x: f64) -> Result<f64, PhysicsError> {
    finite(name, x)?;
    if x > 0.0 {
        Ok(x)
    } else {
        Err(PhysicsError::NotPositive(name))
    }
}

pub(crate) fn non_negative(name: &'static str, x: f64) -> Result<f64, PhysicsError> {
    finite(name, x)?;
    if x >= 0.0 {
        Ok(x)
    } else {
        Err(PhysicsError::Negative(name))
    }
}
