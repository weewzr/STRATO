//! Experimental comparison metrics.
//!
//! Everything here is a pure function on plain numbers. Nothing reads a file, and no
//! accuracy threshold is built in: every cut-off (for example the floor below which a
//! relative deviation is not defined) is an explicit argument of the caller.
//!
//! Notation: `F_model,k` is the model value and `F_exp,k` the experimental value at
//! comparison point `k`; `N` is the number of comparable point values.
//!
//! A measurement is either a point value (optionally with a reported uncertainty `u_k`)
//! or an upper bound `F_exp,k <= L_k`. An upper bound is an inequality: it never enters a
//! deviation, an aggregate or a residual. Only whether the model satisfies it is reported.
//!
//! "Not defined" is returned as `None`; no value is ever invented for it.
//!
//! A numerical agreement computed here is a comparison, not an experimental validation.

/// One experimental measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Measurement {
    /// A measured point value with an optional reported uncertainty `u` (same unit as the
    /// value). `u` is used as given; no coverage factor is assumed. A `u` that is not
    /// finite or not strictly positive is treated as "no uncertainty reported".
    Point {
        value: f64,
        uncertainty: Option<f64>,
    },
    /// A measured upper bound `L`: the true value is reported to be at most `L`.
    UpperBound { limit: f64 },
}

/// A model value paired with the experimental measurement at the same point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pair {
    pub model: f64,
    pub experiment: Measurement,
}

/// Outcome of comparing one pair.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointResult {
    /// Point value compared with a point measurement.
    Point {
        /// `|F_model - F_exp|`.
        absolute_deviation: f64,
        /// Relative percentage deviation, `None` when not defined.
        relative_percent: Option<f64>,
        /// `(F_model - F_exp) / u`, `None` when no usable uncertainty is reported.
        normalised_residual: Option<f64>,
        /// `|F_model - F_exp| <= u`, `None` when no usable uncertainty is reported.
        within_uncertainty: Option<bool>,
    },
    /// Upper bound: only whether `F_model <= L` holds.
    UpperBound { satisfied: bool },
    /// A non-finite number in the pair: nothing can be computed.
    NotComparable,
}

fn usable_uncertainty(u: Option<f64>) -> Option<f64> {
    u.filter(|u| u.is_finite() && *u > 0.0)
}

/// Absolute deviation, `|F_model - F_exp|`.
///
/// `None` if either input is not finite.
pub fn absolute_deviation(model: f64, experiment: f64) -> Option<f64> {
    if model.is_finite() && experiment.is_finite() {
        Some((model - experiment).abs())
    } else {
        None
    }
}

/// Relative percentage deviation,
/// `100 * |F_model - F_exp| / |F_exp|`.
///
/// `None` (not defined) when an input is not finite, when `floor` is not finite or is
/// negative, when `F_exp = 0`, or when `|F_exp| < floor`. The floor is the caller's
/// explicit choice; a value with `|F_exp| == floor` is still defined.
pub fn relative_percent_deviation(model: f64, experiment: f64, floor: f64) -> Option<f64> {
    if !floor.is_finite() || floor < 0.0 {
        return None;
    }
    let abs = absolute_deviation(model, experiment)?;
    let denom = experiment.abs();
    if denom == 0.0 || denom < floor {
        return None;
    }
    Some(abs / denom * 100.0)
}

/// Uncertainty-normalised residual, `r = (F_model - F_exp) / u`.
///
/// `None` if an input is not finite or `u` is not finite and strictly positive.
pub fn normalised_residual(model: f64, experiment: f64, uncertainty: f64) -> Option<f64> {
    if !(model.is_finite() && experiment.is_finite() && uncertainty.is_finite())
        || uncertainty <= 0.0
    {
        return None;
    }
    Some((model - experiment) / uncertainty)
}

/// Whether the model satisfies the upper bound, `F_model <= L`.
///
/// `None` if an input is not finite. This is an inequality check, never a deviation.
pub fn satisfies_upper_bound(model: f64, limit: f64) -> Option<bool> {
    if model.is_finite() && limit.is_finite() {
        Some(model <= limit)
    } else {
        None
    }
}

/// Compare one pair. `relative_floor` is passed to [`relative_percent_deviation`].
pub fn compare_pair(pair: &Pair, relative_floor: f64) -> PointResult {
    match pair.experiment {
        Measurement::UpperBound { limit } => match satisfies_upper_bound(pair.model, limit) {
            Some(satisfied) => PointResult::UpperBound { satisfied },
            None => PointResult::NotComparable,
        },
        Measurement::Point { value, uncertainty } => {
            let Some(abs) = absolute_deviation(pair.model, value) else {
                return PointResult::NotComparable;
            };
            let u = usable_uncertainty(uncertainty);
            PointResult::Point {
                absolute_deviation: abs,
                relative_percent: relative_percent_deviation(pair.model, value, relative_floor),
                normalised_residual: u.and_then(|u| normalised_residual(pair.model, value, u)),
                within_uncertainty: u.map(|u| abs <= u),
            }
        }
    }
}

/// Aggregate deviations over comparable point values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aggregate {
    /// Number of points used, `N >= 1`.
    pub count: usize,
    /// `RMSE = sqrt( (1/N) * sum_k (F_model,k - F_exp,k)^2 )`.
    pub rmse: f64,
    /// `MAE = (1/N) * sum_k |F_model,k - F_exp,k|`.
    pub mae: f64,
    /// `max_k |F_model,k - F_exp,k|`.
    pub max_absolute_deviation: f64,
}

/// RMSE, MAE and maximum absolute deviation over the comparable point values.
///
/// Upper bounds and pairs with a non-finite number are left out. `None` when no point is
/// comparable (empty input, all upper bounds, all non-finite).
pub fn aggregate(pairs: &[Pair]) -> Option<Aggregate> {
    let mut n = 0usize;
    let (mut sum_abs, mut sum_sq, mut max) = (0.0_f64, 0.0_f64, 0.0_f64);
    for pair in pairs {
        if let Measurement::Point { value, .. } = pair.experiment {
            if let Some(d) = absolute_deviation(pair.model, value) {
                n += 1;
                sum_abs += d;
                sum_sq += d * d;
                max = max.max(d);
            }
        }
    }
    if n == 0 {
        return None;
    }
    let nf = n as f64;
    Some(Aggregate {
        count: n,
        rmse: (sum_sq / nf).sqrt(),
        mae: sum_abs / nf,
        max_absolute_deviation: max,
    })
}

/// Comparison status of a dataset (directive section 15). Assigned from data only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    NotAssessed,
    DataIncomplete,
    NotDirectlyComparable,
    ComparisonAvailable,
    WithinReportedUncertainty,
    OutsideReportedUncertainty,
}

impl Status {
    /// The label shown to the reader.
    pub fn label(self) -> &'static str {
        match self {
            Status::NotAssessed => "NOT ASSESSED",
            Status::DataIncomplete => "DATA INCOMPLETE",
            Status::NotDirectlyComparable => "NOT DIRECTLY COMPARABLE",
            Status::ComparisonAvailable => "COMPARISON AVAILABLE",
            Status::WithinReportedUncertainty => "WITHIN REPORTED UNCERTAINTY",
            Status::OutsideReportedUncertainty => "OUTSIDE REPORTED UNCERTAINTY",
        }
    }
}

/// Assign a [`Status`]. Rules, applied in this order:
///
/// 1. `assessed == false`: `NotAssessed`.
/// 2. `definitions_compatible == false` (the caller's judgement from the dataset
///    metadata that model and experiment quantities differ in definition):
///    `NotDirectlyComparable`.
/// 3. No evaluable pair (empty input, or every pair has a non-finite number):
///    `DataIncomplete`.
/// 4. Some evaluable point with a usable uncertainty lies outside it, i.e.
///    `|F_model - F_exp| > u`: `OutsideReportedUncertainty`.
/// 5. Every evaluable point value has a usable uncertainty and lies within it, with at
///    least one such point: `WithinReportedUncertainty`.
/// 6. Otherwise: `ComparisonAvailable`. This includes all-upper-bound input and
///    point values without an uncertainty; upper-bound violations are reported through
///    [`compare_pair`], not through this status.
///
/// No accuracy threshold is used: the only yardstick is the uncertainty reported with
/// the measurement itself.
pub fn assign_status(assessed: bool, definitions_compatible: bool, pairs: &[Pair]) -> Status {
    if !assessed {
        return Status::NotAssessed;
    }
    if !definitions_compatible {
        return Status::NotDirectlyComparable;
    }
    let mut evaluable = 0usize;
    let mut points = 0usize;
    let mut within = 0usize;
    for pair in pairs {
        match compare_pair(pair, 0.0) {
            PointResult::NotComparable => {}
            PointResult::UpperBound { .. } => evaluable += 1,
            PointResult::Point {
                within_uncertainty, ..
            } => {
                evaluable += 1;
                points += 1;
                match within_uncertainty {
                    Some(false) => return Status::OutsideReportedUncertainty,
                    Some(true) => within += 1,
                    None => {}
                }
            }
        }
    }
    if evaluable == 0 {
        Status::DataIncomplete
    } else if points > 0 && within == points {
        Status::WithinReportedUncertainty
    } else {
        Status::ComparisonAvailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;

    fn pt(model: f64, value: f64, u: Option<f64>) -> Pair {
        Pair {
            model,
            experiment: Measurement::Point {
                value,
                uncertainty: u,
            },
        }
    }
    fn ub(model: f64, limit: f64) -> Pair {
        Pair {
            model,
            experiment: Measurement::UpperBound { limit },
        }
    }

    #[test]
    fn absolute_deviation_values() {
        assert_eq!(absolute_deviation(3.0, 5.0), Some(2.0));
        assert_eq!(absolute_deviation(5.0, 3.0), Some(2.0));
        assert_eq!(absolute_deviation(0.0, 0.0), Some(0.0));
        assert_eq!(absolute_deviation(f64::NAN, 1.0), None);
        assert_eq!(absolute_deviation(1.0, f64::INFINITY), None);
    }

    #[test]
    fn relative_deviation_values() {
        // |1.5 - 2| / 2 * 100 = 25
        assert!((relative_percent_deviation(1.5, 2.0, 0.0).unwrap() - 25.0).abs() < EPS);
        // sign of the experimental value does not matter: |3-(-2)|/2*100 = 250
        assert!((relative_percent_deviation(3.0, -2.0, 0.0).unwrap() - 250.0).abs() < EPS);
        // exactly at the floor is defined
        assert!((relative_percent_deviation(1.0, 0.5, 0.5).unwrap() - 100.0).abs() < EPS);
    }

    #[test]
    fn relative_deviation_not_defined() {
        assert_eq!(relative_percent_deviation(1.0, 0.0, 0.0), None); // zero denominator
        assert_eq!(relative_percent_deviation(1.0, 0.0, 1e-3), None);
        assert_eq!(relative_percent_deviation(1.0, 0.4, 0.5), None); // below floor
        assert_eq!(relative_percent_deviation(1.0, -0.4, 0.5), None);
        assert_eq!(relative_percent_deviation(1.0, 2.0, -1.0), None); // bad floor
        assert_eq!(relative_percent_deviation(1.0, 2.0, f64::NAN), None);
        assert_eq!(relative_percent_deviation(f64::NAN, 2.0, 0.0), None);
        assert_eq!(relative_percent_deviation(1.0, f64::INFINITY, 0.0), None);
    }

    #[test]
    fn normalised_residual_values() {
        assert!((normalised_residual(5.0, 3.0, 0.5).unwrap() - 4.0).abs() < EPS);
        assert!((normalised_residual(3.0, 5.0, 0.5).unwrap() + 4.0).abs() < EPS);
        assert_eq!(normalised_residual(1.0, 1.0, 0.0), None);
        assert_eq!(normalised_residual(1.0, 1.0, -1.0), None);
        assert_eq!(normalised_residual(1.0, 1.0, f64::NAN), None);
        assert_eq!(normalised_residual(f64::NAN, 1.0, 1.0), None);
    }

    #[test]
    fn upper_bound_inequality() {
        assert_eq!(satisfies_upper_bound(1.0, 2.0), Some(true));
        assert_eq!(satisfies_upper_bound(2.0, 2.0), Some(true));
        assert_eq!(satisfies_upper_bound(2.5, 2.0), Some(false));
        assert_eq!(satisfies_upper_bound(f64::NAN, 2.0), None);
        assert_eq!(satisfies_upper_bound(1.0, f64::INFINITY), None);
    }

    #[test]
    fn compare_pair_point_and_bound() {
        match compare_pair(&pt(1.5, 2.0, Some(0.25)), 0.0) {
            PointResult::Point {
                absolute_deviation: a,
                relative_percent: Some(r),
                normalised_residual: Some(n),
                within_uncertainty: Some(false),
            } => {
                assert!((a - 0.5).abs() < EPS);
                assert!((r - 25.0).abs() < EPS);
                assert!((n + 2.0).abs() < EPS);
            }
            other => panic!("unexpected {other:?}"),
        }
        // no uncertainty, zero experimental value
        assert_eq!(
            compare_pair(&pt(1.0, 0.0, None), 0.0),
            PointResult::Point {
                absolute_deviation: 1.0,
                relative_percent: None,
                normalised_residual: None,
                within_uncertainty: None
            }
        );
        // unusable uncertainty is treated as absent
        match compare_pair(&pt(1.0, 2.0, Some(0.0)), 0.0) {
            PointResult::Point {
                normalised_residual: None,
                within_uncertainty: None,
                ..
            } => {}
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(
            compare_pair(&ub(1.0, 2.0), 0.0),
            PointResult::UpperBound { satisfied: true }
        );
        assert_eq!(
            compare_pair(&ub(3.0, 2.0), 0.0),
            PointResult::UpperBound { satisfied: false }
        );
        assert_eq!(
            compare_pair(&pt(f64::NAN, 2.0, None), 0.0),
            PointResult::NotComparable
        );
        assert_eq!(
            compare_pair(&ub(1.0, f64::NAN), 0.0),
            PointResult::NotComparable
        );
    }

    #[test]
    fn aggregate_hand_computed() {
        // deviations 0, 1, 2 -> MAE 1, RMSE sqrt(5/3), max 2
        let pairs = [
            pt(1.0, 1.0, None),
            pt(2.0, 3.0, None),
            pt(4.0, 2.0, Some(0.1)),
        ];
        let a = aggregate(&pairs).unwrap();
        assert_eq!(a.count, 3);
        assert!((a.mae - 1.0).abs() < EPS);
        assert!((a.rmse - (5.0_f64 / 3.0).sqrt()).abs() < EPS);
        assert_eq!(a.max_absolute_deviation, 2.0);
    }

    #[test]
    fn aggregate_uses_comparable_points_only() {
        let pairs = [
            pt(1.0, 3.0, None),
            ub(100.0, 1.0), // violated bound must not enter the aggregate
            pt(f64::NAN, 1.0, None),
            pt(2.0, f64::INFINITY, None),
        ];
        let a = aggregate(&pairs).unwrap();
        assert_eq!(a.count, 1);
        assert_eq!(a.mae, 2.0);
        assert_eq!(a.rmse, 2.0);
        assert_eq!(a.max_absolute_deviation, 2.0);
    }

    #[test]
    fn aggregate_none_cases() {
        assert_eq!(aggregate(&[]), None);
        assert_eq!(aggregate(&[ub(1.0, 2.0), ub(3.0, 2.0)]), None);
        assert_eq!(aggregate(&[pt(f64::NAN, 1.0, None)]), None);
    }

    #[test]
    fn status_labels() {
        assert_eq!(Status::NotAssessed.label(), "NOT ASSESSED");
        assert_eq!(Status::DataIncomplete.label(), "DATA INCOMPLETE");
        assert_eq!(
            Status::NotDirectlyComparable.label(),
            "NOT DIRECTLY COMPARABLE"
        );
        assert_eq!(Status::ComparisonAvailable.label(), "COMPARISON AVAILABLE");
        assert_eq!(
            Status::WithinReportedUncertainty.label(),
            "WITHIN REPORTED UNCERTAINTY"
        );
        assert_eq!(
            Status::OutsideReportedUncertainty.label(),
            "OUTSIDE REPORTED UNCERTAINTY"
        );
    }

    #[test]
    fn status_not_assessed_takes_precedence() {
        assert_eq!(assign_status(false, false, &[]), Status::NotAssessed);
        assert_eq!(
            assign_status(false, true, &[pt(1.0, 1.0, None)]),
            Status::NotAssessed
        );
    }

    #[test]
    fn status_not_directly_comparable() {
        assert_eq!(
            assign_status(true, false, &[pt(1.0, 1.0, Some(1.0))]),
            Status::NotDirectlyComparable
        );
        assert_eq!(
            assign_status(true, false, &[]),
            Status::NotDirectlyComparable
        );
    }

    #[test]
    fn status_data_incomplete() {
        assert_eq!(assign_status(true, true, &[]), Status::DataIncomplete);
        assert_eq!(
            assign_status(
                true,
                true,
                &[pt(f64::NAN, 1.0, None), ub(1.0, f64::INFINITY)]
            ),
            Status::DataIncomplete
        );
    }

    #[test]
    fn status_comparison_available() {
        // no uncertainty reported
        assert_eq!(
            assign_status(true, true, &[pt(1.0, 5.0, None)]),
            Status::ComparisonAvailable
        );
        // all upper bounds, even a violated one: no deviation, no uncertainty verdict
        assert_eq!(
            assign_status(true, true, &[ub(1.0, 2.0), ub(9.0, 2.0)]),
            Status::ComparisonAvailable
        );
        // one point within uncertainty, one without uncertainty: cannot claim "within"
        assert_eq!(
            assign_status(true, true, &[pt(1.0, 1.1, Some(0.2)), pt(1.0, 5.0, None)]),
            Status::ComparisonAvailable
        );
    }

    #[test]
    fn status_within_and_outside() {
        // deviation 0.1 <= 0.1 counts as within (boundary), 0.05 <= 0.2
        assert_eq!(
            assign_status(
                true,
                true,
                &[
                    pt(1.0, 1.5, Some(0.5)),
                    pt(2.0, 2.05, Some(0.2)),
                    ub(1.0, 2.0)
                ]
            ),
            Status::WithinReportedUncertainty
        );
        // one outside makes the whole dataset outside, a large deviation stays visible
        assert_eq!(
            assign_status(
                true,
                true,
                &[pt(1.0, 1.05, Some(0.2)), pt(1.0, 5.0, Some(0.2))]
            ),
            Status::OutsideReportedUncertainty
        );
        // outside wins over a point lacking uncertainty
        assert_eq!(
            assign_status(true, true, &[pt(1.0, 5.0, Some(0.2)), pt(1.0, 1.0, None)]),
            Status::OutsideReportedUncertainty
        );
    }
}
