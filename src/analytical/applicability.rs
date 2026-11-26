//! Where Method B (Sturm-Liouville) may be shown: the decision table of science design
//! Section 4.3 as a function.
//!
//! Given the equation stage, the source history, the decay constant and the temperature
//! history it answers: exact (the existing series), applicable through an extension
//! ((B2) decay, (B3) to (B5) source), or not applicable with the reason. A temperature that
//! varies in time is never applicable: the per-layer diffusivities then change by different
//! factors, so the eigenfunctions change in time (science design Section 4.2).
//!
//! The answer is about the mathematics. Whether an extension is shown on a page also depends
//! on its verification status, which this function does not know.
//!
//! Pure function in `f64`, no file access; compiles for `wasm32-unknown-unknown`.

use super::extensions::{Decay, PiecewiseSource};
use std::fmt;

/// The five equation stages (decision 57).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// 1: baseline spherical diffusion.
    Baseline,
    /// 2: generation S(r, t).
    Generation,
    /// 3: decay, `- lambda C`.
    Decay,
    /// 4: temperature-dependent diffusivity D(T).
    TemperatureDependentDiffusivity,
    /// 5: combined.
    Combined,
}

/// What the source term of the run is.
#[derive(Debug, Clone, PartialEq)]
pub enum SourceHistory {
    /// No source.
    None,
    /// Uniform in the kernel, piecewise constant in time (formulas (B3) to (B5)).
    KernelPiecewiseConstant(PiecewiseSource),
    /// Uniform in the kernel but varying continuously in time (for example a ramp).
    KernelTimeVarying,
    /// Varying inside the kernel, or acting in a coating.
    NotKernelUniform,
}

/// The temperature history of the run, in degrees Celsius.
#[derive(Debug, Clone, PartialEq)]
pub enum TemperatureHistory {
    /// One temperature for the whole run.
    Constant(f64),
    /// `(time in s, temperature in degrees Celsius)` breakpoints. The history is constant
    /// only when every temperature is exactly equal.
    Series(Vec<(f64, f64)>),
}

impl TemperatureHistory {
    /// `Ok(true)` for a constant history, `Ok(false)` when it varies in time, `Err` for a
    /// history that is not valid input.
    fn is_constant(&self) -> Result<bool, &'static str> {
        let ok = |t: f64| t.is_finite() && t > -273.15;
        match self {
            Self::Constant(t) => {
                if ok(*t) {
                    Ok(true)
                } else {
                    Err("temperature must be finite and above absolute zero")
                }
            }
            Self::Series(points) => {
                let first = points.first().ok_or("an empty temperature series")?;
                if points.iter().any(|p| !p.0.is_finite() || !ok(p.1)) {
                    return Err("temperature series must be finite and above absolute zero");
                }
                if points.windows(2).any(|w| w[1].0 <= w[0].0) {
                    return Err("the times of the temperature series must be strictly increasing");
                }
                Ok(points.iter().all(|p| p.1 == first.1))
            }
        }
    }
}

/// The extension through which Method B applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extension {
    /// E-decay, formula (B2): no source, decay in every layer.
    Decay,
    /// E-source, formulas (B3) to (B5) with `lambda = 0`.
    Source,
    /// Both extensions, formulas (B3) to (B5) with `lambda > 0`.
    SourceWithDecay,
}

impl Extension {
    /// The formulas of the science design that the extension uses.
    pub fn formulas(&self) -> &'static str {
        match self {
            Self::Decay => "(B2)",
            Self::Source => "(B3) to (B5), lambda = 0",
            Self::SourceWithDecay => "(B3) to (B5)",
        }
    }
}

/// Why Method B is not applicable.
#[derive(Debug, Clone, PartialEq)]
pub enum Reason {
    /// The temperature varies in time, so the eigenfunctions change in time.
    TemperatureVaries,
    /// The source is not piecewise constant in time (for example a ramp).
    SourceNotPiecewiseConstant,
    /// The source varies inside the kernel or acts in a coating.
    SourceNotKernelUniform,
    /// The decay constants of the layers differ.
    DecayConstantsDiffer,
    /// The stage does not contain the mechanism that was supplied.
    MechanismNotInStage(&'static str),
    /// The input is not valid.
    InvalidInput(&'static str),
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TemperatureVaries => write!(
                f,
                "the temperature varies in time, so the per-layer diffusivities change by \
                 different factors and the eigenfunctions change in time"
            ),
            Self::SourceNotPiecewiseConstant => {
                write!(f, "the source is not piecewise constant in time")
            }
            Self::SourceNotKernelUniform => write!(
                f,
                "the source varies inside the kernel or acts in a coating"
            ),
            Self::DecayConstantsDiffer => {
                write!(f, "the decay constants of the layers are not all equal")
            }
            Self::MechanismNotInStage(m) => {
                write!(f, "the equation stage does not contain {m}")
            }
            Self::InvalidInput(why) => write!(f, "invalid input: {why}"),
        }
    }
}

/// The answer of the decision table.
#[derive(Debug, Clone, PartialEq)]
pub enum Applicability {
    /// The existing series is exact.
    Exact,
    /// Applicable through the named extension.
    Extension(Extension),
    /// Not applicable, with the reason.
    NotApplicable(Reason),
}

impl Applicability {
    /// True when Method B may be computed (exact or through an extension).
    pub fn is_applicable(&self) -> bool {
        !matches!(self, Self::NotApplicable(_))
    }
}

/// The decision table of science design Section 4.3.
///
/// Order of the checks: invalid input; a temperature that varies in time; a stage that does
/// not contain a supplied mechanism (stage 1 has neither source nor decay, stage 2 no decay;
/// stage 4 keeps every term of stage 3, so it refuses neither); the source; the decay
/// constants. A source whose rates are all zero counts as no source, a decay constant of zero
/// as no decay.
pub fn applicability(
    stage: Stage,
    source: &SourceHistory,
    decay: &Decay,
    temperature: &TemperatureHistory,
) -> Applicability {
    use Applicability::NotApplicable;
    let constant = match temperature.is_constant() {
        Ok(c) => c,
        Err(why) => return NotApplicable(Reason::InvalidInput(why)),
    };
    if !constant {
        return NotApplicable(Reason::TemperatureVaries);
    }
    let lambda = match decay.constant() {
        Ok(l) => l,
        Err(super::extensions::ExtensionError::UnequalDecayConstants) => {
            // Reported after the stage and source checks below only when they pass; the
            // order is part of the documented behaviour, so check the cheaper ones first.
            f64::NAN
        }
        Err(_) => return NotApplicable(Reason::InvalidInput("decay constant")),
    };
    let decay_unequal = lambda.is_nan();
    let has_decay = decay_unequal || lambda > 0.0;
    let has_source = match source {
        SourceHistory::None => false,
        SourceHistory::KernelPiecewiseConstant(s) => !s.is_zero(),
        SourceHistory::KernelTimeVarying | SourceHistory::NotKernelUniform => true,
    };
    match stage {
        Stage::Baseline => {
            if has_source {
                return NotApplicable(Reason::MechanismNotInStage("a source"));
            }
            if has_decay {
                return NotApplicable(Reason::MechanismNotInStage("decay"));
            }
        }
        Stage::Generation => {
            if has_decay {
                return NotApplicable(Reason::MechanismNotInStage("decay"));
            }
        }
        // Stage 4 keeps every term of stage 3 (science design Section 6): `G = 0` and
        // `lambda = 0` are default inputs, not a different equation. At a constant
        // temperature its mathematics is that of stages 3 and 5.
        Stage::Decay | Stage::TemperatureDependentDiffusivity | Stage::Combined => {}
    }
    match source {
        SourceHistory::KernelTimeVarying => {
            return NotApplicable(Reason::SourceNotPiecewiseConstant)
        }
        SourceHistory::NotKernelUniform => return NotApplicable(Reason::SourceNotKernelUniform),
        _ => {}
    }
    if decay_unequal {
        return NotApplicable(Reason::DecayConstantsDiffer);
    }
    match (has_source, has_decay) {
        (false, false) => Applicability::Exact,
        (false, true) => Applicability::Extension(Extension::Decay),
        (true, false) => Applicability::Extension(Extension::Source),
        (true, true) => Applicability::Extension(Extension::SourceWithDecay),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn constant_source() -> SourceHistory {
        SourceHistory::KernelPiecewiseConstant(PiecewiseSource::constant(1.0e-6, 1.0e7).unwrap())
    }

    const T0: TemperatureHistory = TemperatureHistory::Constant(1028.0);

    fn ramp() -> TemperatureHistory {
        TemperatureHistory::Series(vec![(0.0, 30.0), (1.0e5, 1028.0)])
    }

    fn flat_series() -> TemperatureHistory {
        TemperatureHistory::Series(vec![(0.0, 1600.0), (1.0e5, 1600.0), (2.0e5, 1600.0)])
    }

    use Applicability::{Exact, Extension as Ext, NotApplicable as No};

    #[test]
    fn the_decision_table_row_by_row() {
        let none = SourceHistory::None;
        let nodecay = Decay::None;
        let lam = Decay::Uniform(7.3e-10);
        // stage 1: isothermal baseline is exact
        assert_eq!(applicability(Stage::Baseline, &none, &nodecay, &T0), Exact);
        assert_eq!(
            applicability(Stage::Baseline, &none, &nodecay, &flat_series()),
            Exact
        );
        // stage 2, constant T: source extension; ramp of the source with T: not applicable
        assert_eq!(
            applicability(Stage::Generation, &constant_source(), &nodecay, &T0),
            Ext(Extension::Source)
        );
        assert_eq!(
            applicability(
                Stage::Generation,
                &SourceHistory::KernelTimeVarying,
                &nodecay,
                &ramp()
            ),
            No(Reason::TemperatureVaries)
        );
        // stage 3: decay extension, and with a source
        assert_eq!(
            applicability(Stage::Decay, &none, &lam, &T0),
            Ext(Extension::Decay)
        );
        assert_eq!(
            applicability(Stage::Decay, &constant_source(), &lam, &T0),
            Ext(Extension::SourceWithDecay)
        );
        // stage 4: constant T exact (stage 1 with D evaluated at T), varying T not applicable
        assert_eq!(
            applicability(Stage::TemperatureDependentDiffusivity, &none, &nodecay, &T0),
            Exact
        );
        assert_eq!(
            applicability(
                Stage::TemperatureDependentDiffusivity,
                &none,
                &nodecay,
                &ramp()
            ),
            No(Reason::TemperatureVaries)
        );
        // stage 5: Scenario 1 (constant T and G) through both; the other scenarios (T varies) not
        assert_eq!(
            applicability(Stage::Combined, &constant_source(), &lam, &T0),
            Ext(Extension::SourceWithDecay)
        );
        assert_eq!(
            applicability(Stage::Combined, &constant_source(), &lam, &ramp()),
            No(Reason::TemperatureVaries)
        );
        // stage 5 with the source switched off or decay switched off
        assert_eq!(
            applicability(Stage::Combined, &none, &lam, &T0),
            Ext(Extension::Decay)
        );
        assert_eq!(
            applicability(Stage::Combined, &constant_source(), &nodecay, &T0),
            Ext(Extension::Source)
        );
        assert_eq!(
            applicability(Stage::Combined, &none, &Decay::Uniform(0.0), &T0),
            Exact
        );
    }

    #[test]
    fn refusals_for_every_inapplicable_case() {
        let lam = Decay::Uniform(7.3e-10);
        // temperature varying in time, at every stage
        for stage in [
            Stage::Baseline,
            Stage::Generation,
            Stage::Decay,
            Stage::TemperatureDependentDiffusivity,
            Stage::Combined,
        ] {
            assert_eq!(
                applicability(stage, &SourceHistory::None, &Decay::None, &ramp()),
                No(Reason::TemperatureVaries),
                "{stage:?}"
            );
        }
        // a source that is not piecewise constant, not uniform in the kernel
        assert_eq!(
            applicability(
                Stage::Combined,
                &SourceHistory::KernelTimeVarying,
                &lam,
                &T0
            ),
            No(Reason::SourceNotPiecewiseConstant)
        );
        assert_eq!(
            applicability(Stage::Combined, &SourceHistory::NotKernelUniform, &lam, &T0),
            No(Reason::SourceNotKernelUniform)
        );
        // different decay constants
        assert_eq!(
            applicability(
                Stage::Decay,
                &SourceHistory::None,
                &Decay::PerLayer(vec![1e-9, 2e-9, 1e-9, 1e-9, 1e-9]),
                &T0
            ),
            No(Reason::DecayConstantsDiffer)
        );
        assert_eq!(
            applicability(
                Stage::Decay,
                &SourceHistory::None,
                &Decay::PerLayer(vec![1e-9; 5]),
                &T0
            ),
            Ext(Extension::Decay)
        );
        // a mechanism the stage does not have
        assert_eq!(
            applicability(Stage::Baseline, &constant_source(), &Decay::None, &T0),
            No(Reason::MechanismNotInStage("a source"))
        );
        assert_eq!(
            applicability(Stage::Baseline, &SourceHistory::None, &lam, &T0),
            No(Reason::MechanismNotInStage("decay"))
        );
        // stage 4 keeps every term of stage 3 (science design Section 6): at constant T a
        // source and decay are answered from the inputs, never refused as "not in the stage"
        let s4 = Stage::TemperatureDependentDiffusivity;
        assert_eq!(
            applicability(s4, &constant_source(), &Decay::None, &T0),
            Ext(Extension::Source)
        );
        assert_eq!(
            applicability(s4, &SourceHistory::None, &lam, &T0),
            Ext(Extension::Decay)
        );
        assert_eq!(
            applicability(s4, &constant_source(), &lam, &T0),
            Ext(Extension::SourceWithDecay)
        );
        assert_eq!(
            applicability(s4, &constant_source(), &lam, &ramp()),
            No(Reason::TemperatureVaries)
        );
        // input checks: times strictly increasing, one decay constant per layer
        assert!(matches!(
            applicability(
                Stage::Baseline,
                &SourceHistory::None,
                &Decay::None,
                &TemperatureHistory::Series(vec![(0.0, 900.0), (5.0, 900.0), (5.0, 900.0)])
            ),
            No(Reason::InvalidInput(_))
        ));
        assert!(matches!(
            applicability(
                Stage::Baseline,
                &SourceHistory::None,
                &Decay::None,
                &TemperatureHistory::Series(vec![(10.0, 900.0), (0.0, 900.0)])
            ),
            No(Reason::InvalidInput(_))
        ));
        for n in [1usize, 4, 6] {
            assert!(
                matches!(
                    applicability(
                        Stage::Decay,
                        &SourceHistory::None,
                        &Decay::PerLayer(vec![1e-9; n]),
                        &T0
                    ),
                    No(Reason::InvalidInput(_))
                ),
                "{n} decay constants"
            );
        }
        assert_eq!(
            applicability(Stage::Generation, &constant_source(), &lam, &T0),
            No(Reason::MechanismNotInStage("decay"))
        );
        // invalid input
        assert!(matches!(
            applicability(
                Stage::Baseline,
                &SourceHistory::None,
                &Decay::None,
                &TemperatureHistory::Constant(f64::NAN)
            ),
            No(Reason::InvalidInput(_))
        ));
        assert!(matches!(
            applicability(
                Stage::Baseline,
                &SourceHistory::None,
                &Decay::None,
                &TemperatureHistory::Constant(-300.0)
            ),
            No(Reason::InvalidInput(_))
        ));
        assert!(matches!(
            applicability(
                Stage::Baseline,
                &SourceHistory::None,
                &Decay::None,
                &TemperatureHistory::Series(vec![])
            ),
            No(Reason::InvalidInput(_))
        ));
        assert!(matches!(
            applicability(
                Stage::Decay,
                &SourceHistory::None,
                &Decay::Uniform(-1.0),
                &T0
            ),
            No(Reason::InvalidInput(_))
        ));
        assert!(matches!(
            applicability(
                Stage::Decay,
                &SourceHistory::None,
                &Decay::Uniform(f64::NAN),
                &T0
            ),
            No(Reason::InvalidInput(_))
        ));
        // every refusal has a readable reason
        let r = Reason::TemperatureVaries.to_string();
        assert!(r.contains("temperature"));
        assert!(!Applicability::NotApplicable(Reason::TemperatureVaries).is_applicable());
        assert!(Applicability::Exact.is_applicable());
        assert_eq!(Extension::Decay.formulas(), "(B2)");
    }
}
