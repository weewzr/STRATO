//! Method B: the Sturm-Liouville analytical solution, behind one function.
//!
//! [`method_b`] is the only place that asks which configurations Method B can do and that draws
//! a value. The applicability is the function of `strato::analytical::applicability` (the
//! decision table of science design Section 4.3, with the extensions for decay (B2) and for a
//! piecewise-constant kernel source (B3) to (B5)); the values and the drawing mask are those of
//! `strato::analytical::extensions` (`KernelRun::drawn`, science design Section 4.2 and
//! Document 6 Section 8). A value the mask withholds stays absent and carries its reason.

use strato::analytical::applicability::{
    applicability as table, Applicability, Extension, Reason, SourceHistory, Stage,
    TemperatureHistory,
};
use strato::analytical::extensions::{
    Decay, Drawn, KernelRun, PiecewiseSource, Quantity, Withheld,
};
use strato::analytical::sturm_liouville::{Layer, ModalRelease};
use strato::numerical::physics_run::{Interpolation, TimeFunction};

/// Seconds per hour.
const HOUR_S: f64 = 3600.0;
/// Largest number of modes of one series.
const MAX_MODES: usize = 1_000;

/// What a configuration needs from the analytical solution.
#[derive(Debug, Clone, PartialEq)]
pub struct Needs {
    /// The equation stage of the configuration.
    pub stage: Stage,
    /// The source history of the run.
    pub source: SourceHistory,
    /// The decay constant in s^-1 (0 for no decay).
    pub decay_per_s: f64,
    /// The temperature history of the run.
    pub temperature: TemperatureHistory,
    /// The run starts from the kernel-step state (true) or from another state (false). A fresh
    /// particle (no inventory) is the kernel step with zero inventory.
    pub kernel_step_start: bool,
}

impl Needs {
    /// Stage 1: baseline diffusion at a constant temperature, kernel-step start.
    pub fn baseline(t_c: f64) -> Self {
        Needs {
            stage: Stage::Baseline,
            source: SourceHistory::None,
            decay_per_s: 0.0,
            temperature: TemperatureHistory::Constant(t_c),
            kernel_step_start: true,
        }
    }
}

/// The temperature history of a scenario definition, from its own table.
pub fn temperature_of(f: &TimeFunction) -> TemperatureHistory {
    match f {
        TimeFunction::Constant(c) => TemperatureHistory::Constant(*c),
        TimeFunction::Table { times, values, .. } => {
            TemperatureHistory::Series(times.iter().copied().zip(values.iter().copied()).collect())
        }
    }
}

/// The kernel source of a run from its generation-factor history: `rate_mol_m3_s` times the
/// factor, piecewise constant on `[0, end_s]` and zero after. A factor that changes linearly in
/// time is not piecewise constant ([`SourceHistory::KernelTimeVarying`]); a zero source is
/// [`SourceHistory::None`].
pub fn source_of(factor: &TimeFunction, rate_mol_m3_s: f64, end_s: f64) -> SourceHistory {
    let pieces: Option<(Vec<f64>, Vec<f64>)> = match factor {
        TimeFunction::Constant(f) => Some((vec![0.0, end_s], vec![*f])),
        TimeFunction::Table {
            times,
            values,
            mode: Interpolation::Step,
        } => {
            let mut breaks = Vec::new();
            let mut rates = Vec::new();
            for (&t, &v) in times.iter().zip(values) {
                if t >= end_s {
                    break;
                }
                breaks.push(t);
                rates.push(v);
            }
            breaks.push(end_s);
            Some((breaks, rates))
        }
        TimeFunction::Table {
            values,
            mode: Interpolation::Linear,
            ..
        } => values
            .windows(2)
            .all(|w| w[0] == w[1])
            .then(|| (vec![0.0, end_s], vec![values[0]])),
    };
    let Some((breaks, rates)) = pieces else {
        return SourceHistory::KernelTimeVarying;
    };
    let rates: Vec<f64> = rates.iter().map(|f| f * rate_mol_m3_s).collect();
    match PiecewiseSource::new(breaks, rates) {
        Ok(s) if s.is_zero() => SourceHistory::None,
        Ok(s) => SourceHistory::KernelPiecewiseConstant(s),
        Err(_) => SourceHistory::NotKernelUniform,
    }
}

/// Why Method B is not shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotAvailable {
    /// Stable code of the files.
    pub code: &'static str,
    /// Text of the files and of the page.
    pub reason: String,
}

/// The series of Method B at the requested times.
#[derive(Debug, Clone)]
pub struct Available {
    /// Requested times, s.
    pub times_s: Vec<f64>,
    /// The value, `None` where the drawing mask does not accept it.
    pub values: Vec<Option<f64>>,
    /// The code of the reason a value is withheld, `None` where it is drawn.
    pub withheld: Vec<Option<&'static str>>,
    /// The extension through which the series is computed, `None` for the existing series.
    pub extension: Option<Extension>,
    /// Estimated absolute accuracy of the series (`accuracy_floor`).
    pub accuracy_floor: f64,
    /// Smallest time at which the omitted modes are negligible (`min_valid_time`), s.
    pub valid_from_s: f64,
    /// Number of modes.
    pub modes: usize,
    /// Eigenvalue cover `alpha_max` in s^-1 (the module's `lambda_covered`).
    pub alpha_max_per_s: f64,
    /// Weight missing from the partial sum, `1 - sum w`, a diagnostic and never the mask.
    pub missing_weight: f64,
}

/// The answer.
#[derive(Debug, Clone)]
pub enum MethodB {
    /// Drawn where unmasked.
    Available(Available),
    /// Not shown.
    NotAvailable(NotAvailable),
}

/// The code and the text of a refusal of the decision table.
fn refusal(r: &Reason) -> NotAvailable {
    let (code, reason) = match r {
        Reason::TemperatureVaries => (
            "TIME_VARYING_TEMPERATURE",
            "the temperature changes in time, so the eigenfunctions change in time; the analytical solution is not defined for this case (science design Section 4.2)".to_string(),
        ),
        Reason::SourceNotPiecewiseConstant => (
            "SOURCE_NOT_PIECEWISE_CONSTANT",
            r.to_string(),
        ),
        Reason::SourceNotKernelUniform => ("SOURCE_NOT_KERNEL_UNIFORM", r.to_string()),
        Reason::DecayConstantsDiffer => ("DECAY_CONSTANTS_DIFFER", r.to_string()),
        Reason::MechanismNotInStage(_) => ("MECHANISM_NOT_IN_STAGE", r.to_string()),
        Reason::InvalidInput(_) => ("INVALID_INPUT", r.to_string()),
    };
    NotAvailable { code, reason }
}

/// The codes of withheld values with their meaning, for the curve files.
pub const WITHHELD_LEGEND: [(&str, &str); 8] = [
    ("BEFORE_MIN_VALID_TIME", "t is before the first time at which the omitted modes are negligible (min_valid_time)"),
    ("COVER_SENSITIVITY", "the series with the eigenvalue cover reduced by a factor 4 differs from the full series by more than 5 % of the value"),
    ("BELOW_ACCURACY_FLOOR", "the value is below the accuracy floor of the existing series (4 times accuracy_floor against 5 % of the value)"),
    ("EXTENSION_ERROR_TOO_LARGE", "the estimated error of the decay or source extension is above 5 % of the value (Document 6 Section 8)"),
    ("OUTSIDE_ESTIMATED_REGIME", "the run lies outside the regime in which the error estimate of the extensions was measured"),
    ("QUANTITY_UNDEFINED", "the plotted quantity is not defined for this state (for example a fraction of a zero input)"),
    ("ERROR_ESTIMATE_UNDEFINED", "the error estimate is not defined because the amount released so far is not positive"),
    ("RUN_REFUSED", "the extension refuses the input"),
];

/// The code of a withheld value.
fn withheld_code(w: &Withheld) -> &'static str {
    match w {
        Withheld::BeforeMinValidTime => "BEFORE_MIN_VALID_TIME",
        Withheld::CoverSensitivity { .. } => "COVER_SENSITIVITY",
        Withheld::BelowAccuracyFloor { .. } => "BELOW_ACCURACY_FLOOR",
        Withheld::ExtensionErrorTooLarge { .. } => "EXTENSION_ERROR_TOO_LARGE",
        Withheld::OutsideEstimatedRegime(_) => "OUTSIDE_ESTIMATED_REGIME",
        // The mask's own texts: the quantity itself, or its error estimate, is not defined (a
        // zero or negative amount released so far).
        Withheld::Undefined(why) if why.contains("relative error") => "ERROR_ESTIMATE_UNDEFINED",
        Withheld::Undefined(_) => "QUANTITY_UNDEFINED",
        Withheld::FloorNotReestimated => "FLOOR_NOT_REESTIMATED",
    }
}

/// The applicability of Method B for a configuration, without computing anything: `Ok(None)` for
/// the existing series, `Ok(Some(e))` through the extension `e`.
///
/// # Errors
///
/// [`NotAvailable`] with the code and the reason.
pub fn applicability(needs: &Needs) -> Result<Option<Extension>, NotAvailable> {
    if !needs.kernel_step_start {
        return Err(NotAvailable {
            code: "START_STATE_NOT_KERNEL_STEP",
            reason: "the analytical solution is derived for the kernel-step initial state; this run starts from another state".into(),
        });
    }
    let decay = if needs.decay_per_s > 0.0 {
        Decay::Uniform(needs.decay_per_s)
    } else {
        Decay::None
    };
    match table(needs.stage, &needs.source, &decay, &needs.temperature) {
        Applicability::Exact => Ok(None),
        Applicability::Extension(e) => Ok(Some(e)),
        Applicability::NotApplicable(r) => Err(refusal(&r)),
    }
}

fn layers(radii_m: &[f64; 5], d: &[f64; 5]) -> Vec<Layer> {
    (0..5)
        .map(|k| Layer {
            outer_radius: radii_m[k],
            diffusivity: d[k],
        })
        .collect()
}

/// Method B for a configuration at times in s: the quantity `q` of a kernel-step start with the
/// initial kernel inventory `m0_mol` (1 for a fraction of the initial inventory, 0 for a fresh
/// particle), at constant temperature with the diffusivities `d` evaluated at it. The kernel
/// volume of the source is that of the first layer of `radii_m`.
///
/// A value is drawn only where the drawing mask of science design Section 4.2 accepts it:
/// `t >= min_valid_time`, the series with the eigenvalue cover reduced by a factor 4 differs by at
/// most 5 % (the labelled 5 % convention of decision 49, not a derived bound), and the estimated
/// error is at most 5 % of the value (the floor test for the existing series, the rule of
/// Document 6 Section 8 for decay and a source, inside the measured regime). The other values stay
/// absent with their reason in [`Available::withheld`].
pub fn method_b(
    needs: &Needs,
    q: Quantity,
    m0_mol: f64,
    radii_m: &[f64; 5],
    d: &[f64; 5],
    times_s: &[f64],
) -> MethodB {
    let extension = match applicability(needs) {
        Ok(e) => e,
        Err(n) => return MethodB::NotAvailable(n),
    };
    let ls = layers(radii_m, d);
    let t_min = times_s
        .iter()
        .copied()
        .filter(|t| *t > 0.0)
        .fold(f64::INFINITY, f64::min);
    if !t_min.is_finite() {
        return MethodB::NotAvailable(NotAvailable {
            code: "NO_POSITIVE_TIME",
            reason: "the series needs at least one positive time".into(),
        });
    }
    // The oscillation count of the eigenvalue search takes the integer part of k L / pi in every
    // layer; it is reliable only while that number stays far below 2^53. The cover is limited so
    // that it stays below 1e9 (at extreme diffusivity contrasts, for example Sr-90 at 400 degC,
    // an unlimited cover makes the root search diverge). The limit only hides early times.
    const MAX_HALF_WAVES: f64 = 1.0e9;
    let mut left = 0.0;
    let mut cover_limit = f64::INFINITY;
    for (r, dk) in radii_m.iter().zip(d) {
        let len = r - left;
        left = *r;
        cover_limit = cover_limit.min(dk * (std::f64::consts::PI * MAX_HALF_WAVES / len).powi(2));
    }
    let lambda_max = (45.0 / t_min).min(cover_limit);
    // The cost of the weights grows with the square of the number of modes, so the number is
    // capped; the cap only moves the first drawn time (`min_valid_time`) later, never the values.
    let full = match ModalRelease::new(&ls, lambda_max, MAX_MODES) {
        Ok(s) => s,
        Err(e) => {
            return MethodB::NotAvailable(NotAvailable {
                code: "SERIES_NOT_BUILT",
                reason: format!("the modal series could not be built: {e}"),
            })
        }
    };
    let source = match &needs.source {
        SourceHistory::KernelPiecewiseConstant(s) => s.clone(),
        _ => PiecewiseSource::none(),
    };
    let decay = if needs.decay_per_s > 0.0 {
        Decay::Uniform(needs.decay_per_s)
    } else {
        Decay::None
    };
    let kernel_volume = 4.0 / 3.0 * std::f64::consts::PI * radii_m[0].powi(3);
    let run = match KernelRun::new(&full, m0_mol, kernel_volume, source, &decay) {
        Ok(r) => r.with_geometry(radii_m[0], radii_m[4]),
        Err(e) => {
            return MethodB::NotAvailable(NotAvailable {
                code: "RUN_REFUSED",
                reason: format!("the extension refuses the input: {e:?}"),
            })
        }
    };
    let mut values = Vec::with_capacity(times_s.len());
    let mut withheld = Vec::with_capacity(times_s.len());
    for &t in times_s {
        match run.drawn(q, t) {
            Ok(Drawn::Value(v)) if v.is_finite() => {
                values.push(Some(v));
                withheld.push(None);
            }
            Ok(Drawn::Value(_)) => {
                values.push(None);
                withheld.push(Some("QUANTITY_UNDEFINED"));
            }
            Ok(Drawn::Withheld(w)) => {
                values.push(None);
                withheld.push(Some(withheld_code(&w)));
            }
            Err(_) => {
                values.push(None);
                withheld.push(Some("RUN_REFUSED"));
            }
        }
    }
    if values.iter().all(Option::is_none) {
        // The most common reason, for the text.
        let mut counts: Vec<(&'static str, usize)> = Vec::new();
        for c in withheld.iter().flatten() {
            match counts.iter_mut().find(|(k, _)| k == c) {
                Some(e) => e.1 += 1,
                None => counts.push((c, 1)),
            }
        }
        counts.sort_by_key(|a| std::cmp::Reverse(a.1));
        let list: Vec<String> = counts.iter().map(|(c, n)| format!("{c} x {n}")).collect();
        return MethodB::NotAvailable(NotAvailable {
            code: "NOT_DRAWN_AT_ANY_TIME",
            reason: format!(
                "the drawing mask of science design Section 4.2 accepts no value of this curve ({}; first valid time {:.3e} s, accuracy floor {:.1e})",
                list.join(", "),
                full.min_valid_time(),
                full.accuracy_floor()
            ),
        });
    }
    MethodB::Available(Available {
        times_s: times_s.to_vec(),
        values,
        withheld,
        extension,
        accuracy_floor: full.accuracy_floor(),
        valid_from_s: full.min_valid_time(),
        modes: full.eigenvalues.len(),
        alpha_max_per_s: full.lambda_covered,
        missing_weight: 1.0 - full.weights.iter().sum::<f64>(),
    })
}

/// Hours of a time in s (helper for the files).
pub fn hours(t_s: f64) -> f64 {
    t_s / HOUR_S
}

#[cfg(test)]
mod tests {
    use super::*;

    const RADII: [f64; 5] = [175.0e-6, 278.5e-6, 317.9e-6, 353.2e-6, 394.2e-6];
    const D: [f64; 5] = [1.0e-13, 1.0e-12, 1.0e-14, 1.0e-15, 1.0e-14];

    fn base() -> Needs {
        Needs::baseline(1600.0)
    }

    #[test]
    fn the_decision_table_of_section_4_3() {
        assert_eq!(applicability(&base()), Ok(None));
        let e = applicability(&Needs {
            temperature: TemperatureHistory::Series(vec![(0.0, 30.0), (1.0e5, 1028.0)]),
            ..base()
        })
        .unwrap_err();
        assert_eq!(e.code, "TIME_VARYING_TEMPERATURE");
        // Decay and a source apply through their extensions at a constant temperature.
        let decay = Needs {
            stage: Stage::Decay,
            decay_per_s: 1.0e-9,
            ..base()
        };
        assert_eq!(applicability(&decay), Ok(Some(Extension::Decay)));
        let src = Needs {
            stage: Stage::Generation,
            source: SourceHistory::KernelPiecewiseConstant(
                PiecewiseSource::constant(1.0e-6, 1.0e7).unwrap(),
            ),
            ..base()
        };
        assert_eq!(applicability(&src), Ok(Some(Extension::Source)));
        let both = Needs {
            stage: Stage::Combined,
            decay_per_s: 1.0e-9,
            ..src.clone()
        };
        assert_eq!(applicability(&both), Ok(Some(Extension::SourceWithDecay)));
        // A varying temperature wins over an extension.
        let e = applicability(&Needs {
            temperature: TemperatureHistory::Series(vec![(0.0, 30.0), (1.0e5, 1028.0)]),
            ..both
        })
        .unwrap_err();
        assert_eq!(e.code, "TIME_VARYING_TEMPERATURE");
        assert_eq!(
            applicability(&Needs {
                kernel_step_start: false,
                ..base()
            })
            .unwrap_err()
            .code,
            "START_STATE_NOT_KERNEL_STEP"
        );
        // A ramp in the source is not piecewise constant.
        let ramp =
            TimeFunction::table(vec![0.0, 10.0], vec![0.0, 1.0], Interpolation::Linear).unwrap();
        assert_eq!(
            source_of(&ramp, 1.0, 10.0),
            SourceHistory::KernelTimeVarying
        );
    }

    #[test]
    fn the_source_of_a_step_history_has_one_piece_per_step_and_ends_at_the_run_end() {
        let f = TimeFunction::table(
            vec![0.0, 100.0, 200.0],
            vec![1.0, 2.0, 0.5],
            Interpolation::Step,
        )
        .unwrap();
        let SourceHistory::KernelPiecewiseConstant(s) = source_of(&f, 3.0, 300.0) else {
            panic!("a piecewise source")
        };
        assert_eq!(s.breaks(), &[0.0, 100.0, 200.0, 300.0]);
        assert_eq!(s.rates(), &[3.0, 6.0, 1.5]);
        assert_eq!(
            source_of(&TimeFunction::constant(0.0).unwrap(), 3.0, 300.0),
            SourceHistory::None
        );
    }

    #[test]
    fn the_mask_hides_early_values_and_keeps_late_ones() {
        let times: Vec<f64> = vec![60.0, 3600.0, 36000.0, 360000.0, 1.0e6];
        match method_b(&base(), Quantity::ReleaseFraction, 1.0, &RADII, &D, &times) {
            MethodB::Available(a) => {
                assert_eq!(a.values.len(), 5);
                assert!(a.values[4].is_some_and(|v| v > 0.0 && v < 1.0));
                assert!(a.accuracy_floor > 0.0 && a.valid_from_s > 0.0);
                assert!(a.extension.is_none());
                // Every absent value carries its reason, every drawn value none.
                for (v, w) in a.values.iter().zip(&a.withheld) {
                    assert_eq!(v.is_some(), w.is_none());
                }
            }
            MethodB::NotAvailable(n) => panic!("{n:?}"),
        }
    }

    #[test]
    fn decay_goes_through_the_extension_and_lowers_the_release() {
        // A long half-life barely changes the release of the kernel step; a short one lowers it.
        // The first time sets the eigenvalue cover (and with it the number of modes).
        let times: Vec<f64> = vec![60.0, 3.6e5, 1.0e6, 3.0e6];
        let plain = method_b(&base(), Quantity::ReleaseFraction, 1.0, &RADII, &D, &times);
        let decaying = Needs {
            stage: Stage::Decay,
            decay_per_s: 1.0e-7,
            ..base()
        };
        let with = method_b(
            &decaying,
            Quantity::ReleaseFraction,
            1.0,
            &RADII,
            &D,
            &times,
        );
        let (MethodB::Available(p), MethodB::Available(w)) = (&plain, &with) else {
            panic!("both series are available: {plain:?} {with:?}")
        };
        assert_eq!(w.extension, Some(Extension::Decay));
        let mut compared = 0;
        for k in 0..times.len() {
            if let (Some(a), Some(b)) = (p.values[k], w.values[k]) {
                assert!(b < a && b > 0.0, "decay lowers the release: {a} {b}");
                compared += 1;
            }
        }
        assert!(compared >= 1, "no decay value was drawn: {:?}", w.withheld);
        // Whether each value is drawn is decided by the mask; a withheld value has a reason.
        for (v, r) in w.values.iter().zip(&w.withheld) {
            assert_eq!(v.is_some(), r.is_none());
        }
    }

    #[test]
    fn a_varying_temperature_is_refused_before_anything_is_computed() {
        let n = Needs {
            temperature: TemperatureHistory::Series(vec![(0.0, 30.0), (1.0e5, 1028.0)]),
            ..base()
        };
        match method_b(&n, Quantity::ReleaseFraction, 1.0, &RADII, &D, &[3600.0]) {
            MethodB::NotAvailable(r) => assert_eq!(r.code, "TIME_VARYING_TEMPERATURE"),
            MethodB::Available(_) => panic!("must be refused"),
        }
    }
}
