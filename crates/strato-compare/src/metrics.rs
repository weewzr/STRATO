//! The per-point diagnostics of directive section 13 and the signed log10 ratio of decision 18.
//!
//! Absolute deviation, relative deviation, the maximum absolute deviation and the
//! uncertainty-normalised residual come from the verified `strato::validation::metrics` module.
//! This file adds the signed log10 ratio, the upper-bound margin (decision 23) and the rules of
//! decisions 17, 19 and 23. No whole-curve metric is formed here: the only whole-curve metric of
//! the pipeline is R-squared (decision 69(a), `metric-r2.md`). No accuracy threshold and no
//! near-zero floor is built in: a floor would have to be stated by the source (decision 17), and
//! the register states none.

use strato::validation::metrics::{
    absolute_deviation, aggregate, normalised_residual, relative_percent_deviation, Measurement,
    Pair,
};

/// The deviations of one model value from one measured point value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointDeviation {
    /// `F_model - F_exp` (signed, dimensionless).
    pub signed: f64,
    /// `|F_model - F_exp|`.
    pub absolute: f64,
    /// `100 |F_model - F_exp| / |F_exp|`, undefined for `F_exp <= 0` (decision 17).
    pub relative_percent: Option<f64>,
    /// `log10(F_model / F_exp)` when both are positive (decision 18).
    pub log10_ratio: Option<f64>,
    /// `(F_model - F_exp) / u` when the source reports a measurement uncertainty `u`.
    pub normalised_residual: Option<f64>,
}

/// Signed decimal logarithm of the ratio of two positive numbers.
pub fn log10_ratio(model: f64, experiment: f64) -> Option<f64> {
    if model.is_finite() && experiment.is_finite() && model > 0.0 && experiment > 0.0 {
        Some((model / experiment).log10())
    } else {
        None
    }
}

/// Deviations of a model value from a point value. `reported_uncertainty` is a measurement
/// uncertainty the source reports; a reading uncertainty of a digitised value is not one and is
/// never passed here.
pub fn point_deviation(
    model: f64,
    experiment: f64,
    reported_uncertainty: Option<f64>,
) -> Option<PointDeviation> {
    let absolute = absolute_deviation(model, experiment)?;
    let relative_percent = if experiment > 0.0 {
        relative_percent_deviation(model, experiment, 0.0)
    } else {
        None
    };
    let normalised = reported_uncertainty.and_then(|u| normalised_residual(model, experiment, u));
    Some(PointDeviation {
        signed: model - experiment,
        absolute,
        relative_percent,
        log10_ratio: log10_ratio(model, experiment),
        normalised_residual: normalised,
    })
}

/// The standing of a model value against an upper bound (an inequality, never a deviation).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundMargin {
    /// `F_model <= limit`.
    pub satisfied: bool,
    /// `F_model / limit` (below one when satisfied).
    pub ratio: f64,
    /// `max(0, F_model - limit)`, the excess of a violated bound.
    pub excess: f64,
    /// `log10(F_model / limit)` when both are positive.
    pub log10_ratio: Option<f64>,
}

/// Compare a model value with an upper bound.
pub fn bound_margin(model: f64, limit: f64) -> Option<BoundMargin> {
    if !(model.is_finite() && limit.is_finite() && limit > 0.0) {
        return None;
    }
    Some(BoundMargin {
        satisfied: model <= limit,
        ratio: model / limit,
        excess: (model - limit).max(0.0),
        log10_ratio: log10_ratio(model, limit),
    })
}

/// Aggregates of a group of point comparisons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointAggregate {
    /// Number of point values.
    pub n: usize,
    /// Largest `|F_model - F_exp|`.
    pub max_absolute: f64,
    /// Number of points with a positive model and measured value (the log ratio is defined).
    pub n_log: usize,
    /// Number of points whose relative deviation is defined.
    pub n_relative: usize,
}

/// Aggregate pairs of `(model, experiment)` point values. `None` when the group is empty.
pub fn point_aggregate(pairs: &[(f64, f64)]) -> Option<PointAggregate> {
    let p: Vec<Pair> = pairs
        .iter()
        .map(|&(m, e)| Pair {
            model: m,
            experiment: Measurement::Point {
                value: e,
                uncertainty: None,
            },
        })
        .collect();
    let a = aggregate(&p)?;
    let logs: Vec<f64> = pairs
        .iter()
        .filter_map(|&(m, e)| log10_ratio(m, e))
        .collect();
    let n_rel = pairs
        .iter()
        .filter(|&&(m, e)| e > 0.0 && m.is_finite())
        .count();
    Some(PointAggregate {
        n: a.count,
        max_absolute: a.max_absolute_deviation,
        n_log: logs.len(),
        n_relative: n_rel,
    })
}

/// Aggregates of a group of upper-bound comparisons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundAggregate {
    /// Number of bounds.
    pub n: usize,
    /// Bounds the model satisfies.
    pub satisfied: usize,
    /// Bounds the model exceeds.
    pub exceeded: usize,
    /// Smallest `F_model / limit`.
    pub ratio_min: f64,
    /// Largest `F_model / limit`.
    pub ratio_max: f64,
}

/// Aggregate bound margins. `None` when the group is empty.
pub fn bound_aggregate(margins: &[BoundMargin]) -> Option<BoundAggregate> {
    if margins.is_empty() {
        return None;
    }
    let satisfied = margins.iter().filter(|m| m.satisfied).count();
    Some(BoundAggregate {
        n: margins.len(),
        satisfied,
        exceeded: margins.len() - satisfied,
        ratio_min: margins
            .iter()
            .map(|m| m.ratio)
            .fold(f64::INFINITY, f64::min),
        ratio_max: margins
            .iter()
            .map(|m| m.ratio)
            .fold(f64::NEG_INFINITY, f64::max),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_deviation_values_and_undefined_cases() {
        let d = point_deviation(3.0e-3, 2.0e-3, None).unwrap();
        assert!((d.signed - 1.0e-3).abs() < 1e-18);
        assert!((d.absolute - 1.0e-3).abs() < 1e-18);
        assert!((d.relative_percent.unwrap() - 50.0).abs() < 1e-9);
        assert!((d.log10_ratio.unwrap() - (1.5_f64).log10()).abs() < 1e-12);
        assert!(d.normalised_residual.is_none());
        // F_exp = 0: relative and log ratio are undefined, the absolute deviation is not.
        let z = point_deviation(1.0e-3, 0.0, None).unwrap();
        assert_eq!(z.relative_percent, None);
        assert_eq!(z.log10_ratio, None);
        assert_eq!(z.absolute, 1.0e-3);
        // a negative measured value has no relative deviation either
        assert_eq!(
            point_deviation(1.0, -1.0, None).unwrap().relative_percent,
            None
        );
        assert!(point_deviation(f64::NAN, 1.0, None).is_none());
        // a reported uncertainty gives the residual
        let r = point_deviation(5.0, 3.0, Some(0.5)).unwrap();
        assert!((r.normalised_residual.unwrap() - 4.0).abs() < 1e-12);
    }

    #[test]
    fn bound_margin_is_an_inequality() {
        let ok = bound_margin(1e-6, 1e-5).unwrap();
        assert!(ok.satisfied);
        assert!((ok.ratio - 0.1).abs() < 1e-12);
        assert_eq!(ok.excess, 0.0);
        let bad = bound_margin(3e-5, 1e-5).unwrap();
        assert!(!bad.satisfied);
        assert!((bad.excess - 2e-5).abs() < 1e-18);
        assert!((bad.log10_ratio.unwrap() - 3.0_f64.log10()).abs() < 1e-12);
        assert!(bound_margin(1.0, 0.0).is_none());
    }

    #[test]
    fn aggregates_match_hand_values() {
        // pairs (model, exp): errors +1, -3 -> max 3; no aggregate deviation is formed
        // (decision 69(a) removed every aggregate deviation over points)
        let a = point_aggregate(&[(2.0, 1.0), (1.0, 4.0)]).unwrap();
        assert_eq!(a.n, 2);
        assert!((a.max_absolute - 3.0).abs() < 1e-12);
        assert_eq!(a.n_log, 2);
        assert_eq!(a.n_relative, 2);
        assert!(point_aggregate(&[]).is_none());
        // n = 1: the maximum equals the absolute deviation and n is reported
        let one = point_aggregate(&[(0.5, 0.25)]).unwrap();
        assert_eq!(one.n, 1);
        assert!((one.max_absolute - 0.25).abs() < 1e-15);
    }

    #[test]
    fn bound_aggregate_counts() {
        let m = [
            bound_margin(1e-7, 1e-5).unwrap(),
            bound_margin(2e-5, 1e-5).unwrap(),
        ];
        let a = bound_aggregate(&m).unwrap();
        assert_eq!((a.n, a.satisfied, a.exceeded), (2, 1, 1));
        assert!((a.ratio_min - 0.01).abs() < 1e-12);
        assert!((a.ratio_max - 2.0).abs() < 1e-12);
        assert!(bound_aggregate(&[]).is_none());
    }
}
