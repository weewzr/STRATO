//! The comparison of every row of the release register with the BTCS model.
//!
//! Each row gets a status from [`crate::status::classify`]. Rows with a model plan are run through
//! the solver; bound rows become WITHIN or EXCEEDS REPORTED UPPER BOUND from the inequality; point
//! rows get the deviations of section 13. Rows whose denominator is not established get a model value for scale
//! only (a provisional value): it never enters a status or an aggregate.

use std::collections::BTreeMap;

use strato_data::release::{ReadingUncertainty, ReleaseRow, ValueReading};
use strato_data::table::Vocabulary;
use strato_data::Register;

use crate::data::Species;
use crate::metrics::{
    bound_aggregate, bound_margin, point_aggregate, point_deviation, BoundAggregate, BoundMargin,
    PointAggregate, PointDeviation,
};
use crate::model::Runner;
use crate::status::{
    classify, measured_text, single_value, BoundSource, Plan, RunSpec, Status, Verdict,
};

/// The model value behind a row.
#[derive(Debug, Clone)]
pub struct ModelValue {
    /// The run.
    pub spec: RunSpec,
    /// Release fraction at the end of the window.
    pub f_end: f64,
    /// Release fraction at the start of the window (0 for a cumulative value).
    pub f_start: f64,
    /// The value to compare: `f_end - f_start`.
    pub f_window: f64,
    /// `|f_window - f_window of the refined run|`, the numerical difference of the model value.
    pub numerical_difference: f64,
    /// The five coefficients, m2/s.
    pub d_m2_s: [f64; 5],
    /// Identifier of the coefficient set.
    pub set_id: String,
}

/// The evaluation of a row against its model value.
#[derive(Debug, Clone)]
pub enum Eval {
    /// Nothing was evaluated.
    None,
    /// An inequality.
    Bound {
        /// The bound.
        limit: f64,
        /// How the number is read as a bound.
        source: BoundSource,
        /// The margin.
        margin: BoundMargin,
    },
    /// A point comparison.
    Point {
        /// The measured value.
        value: f64,
        /// The deviations.
        dev: PointDeviation,
    },
    /// A model value for scale only.
    Provisional {
        /// The measured value.
        value: f64,
        /// The deviations, never aggregated into a status.
        dev: PointDeviation,
    },
}

/// One row, its verdict and its model comparison.
#[derive(Debug, Clone)]
pub struct RowRecord {
    /// The register row.
    pub row: ReleaseRow,
    /// The rules' verdict.
    pub verdict: Verdict,
    /// The status after the model has run.
    pub status: Status,
    /// The model value, if a run was needed.
    pub model: Option<ModelValue>,
    /// The comparison.
    pub eval: Eval,
}

/// Text of a reading uncertainty.
pub fn reading_text(u: &ReadingUncertainty) -> String {
    match u {
        ReadingUncertainty::NotDigitised => "not digitised".to_string(),
        ReadingUncertainty::NotRecorded => {
            "digitised, reading uncertainty not recorded".to_string()
        }
        ReadingUncertainty::Stated { value, time_h } => {
            let mut parts = Vec::new();
            match value {
                Some(ValueReading::RelativePercent(p)) => {
                    parts.push(format!("value +/- {} %", p.text))
                }
                Some(ValueReading::RelativePercentRange { lo, hi }) => {
                    parts.push(format!("value +/- {} to {} %", lo.text, hi.text))
                }
                Some(ValueReading::Absolute(p)) => {
                    parts.push(format!("value +/- {} (absolute)", p.text))
                }
                Some(ValueReading::Factor(p)) => parts.push(format!("value factor {}", p.text)),
                None => {}
            }
            if let Some(t) = time_h {
                parts.push(format!("time +/- {} h", t.text));
            }
            format!("digitising reading uncertainty only: {}", parts.join("; "))
        }
    }
}

/// Run the whole register through the rules and the model.
pub fn run_rows(reg: &Register, runner: &mut Runner<'_>) -> Result<Vec<RowRecord>, String> {
    // Pass 1: verdicts and the longest time needed per species and temperature.
    let verdicts: Vec<Verdict> = reg.release.rows.iter().map(|r| classify(reg, r)).collect();
    let mut need: BTreeMap<(Species, i64), f64> = BTreeMap::new();
    for v in &verdicts {
        if let Some(spec) = spec_of(&v.plan) {
            let e = need
                .entry((spec.species, (spec.t_c * 10.0).round() as i64))
                .or_insert(0.0);
            *e = e.max(spec.t_end_h);
        }
    }
    for ((species, t10), end) in &need {
        runner.get(*species, *t10 as f64 / 10.0, *end)?;
    }
    // Pass 2: evaluate.
    let mut out = Vec::with_capacity(verdicts.len());
    for (row, verdict) in reg.release.rows.iter().zip(verdicts) {
        let (model, eval, status) = evaluate_row(runner, row, &verdict)?;
        out.push(RowRecord {
            row: row.clone(),
            verdict,
            status,
            model,
            eval,
        });
    }
    Ok(out)
}

fn spec_of(plan: &Plan) -> Option<&RunSpec> {
    match plan {
        Plan::NoModel => None,
        Plan::Inequality { spec, .. }
        | Plan::Point { spec, .. }
        | Plan::Provisional { spec, .. } => Some(spec),
    }
}

fn evaluate_row(
    runner: &mut Runner<'_>,
    row: &ReleaseRow,
    verdict: &Verdict,
) -> Result<(Option<ModelValue>, Eval, Status), String> {
    let Some(spec) = spec_of(&verdict.plan) else {
        return Ok((None, Eval::None, verdict.status));
    };
    let key_end = spec.t_end_h;
    let curve = runner.get(spec.species, spec.t_c, key_end)?.clone();
    let f_end = curve
        .at(spec.t_end_h)
        .ok_or_else(|| format!("row {}: the run does not reach {} h", row.id, spec.t_end_h))?;
    let f_start = if spec.window_start_h > 0.0 {
        curve.at(spec.window_start_h).ok_or_else(|| {
            format!(
                "row {}: the run does not reach {} h",
                row.id, spec.window_start_h
            )
        })?
    } else {
        0.0
    };
    let f_window = f_end - f_start;
    let refined_end = curve.at_refined(spec.t_end_h).unwrap_or(f_end);
    let refined_start = if spec.window_start_h > 0.0 {
        curve.at_refined(spec.window_start_h).unwrap_or(f_start)
    } else {
        0.0
    };
    let numerical_difference = (f_window - (refined_end - refined_start)).abs();
    let model = ModelValue {
        spec: spec.clone(),
        f_end,
        f_start,
        f_window,
        numerical_difference,
        d_m2_s: curve.d_m2_s,
        set_id: runner.sets[&spec.species].id.clone(),
    };
    let (eval, status) = match &verdict.plan {
        Plan::Inequality { limit, source, .. } => {
            let margin = bound_margin(f_window, *limit)
                .ok_or_else(|| format!("row {}: the bound is not usable", row.id))?;
            let st = if margin.satisfied {
                Status::WithinReportedUpperBound
            } else {
                Status::ExceedsReportedUpperBound
            };
            (
                Eval::Bound {
                    limit: *limit,
                    source: *source,
                    margin,
                },
                st,
            )
        }
        Plan::Point { value, .. } => {
            let dev = point_deviation(f_window, *value, None)
                .ok_or_else(|| format!("row {}: no deviation", row.id))?;
            // No row carries a reported measurement uncertainty, so the status stays
            // COMPARISON AVAILABLE; a digitising uncertainty never decides a status.
            (
                Eval::Point { value: *value, dev },
                Status::ComparisonAvailable,
            )
        }
        Plan::Provisional { value, .. } => {
            let dev = point_deviation(f_window, *value, None)
                .ok_or_else(|| format!("row {}: no deviation", row.id))?;
            (Eval::Provisional { value: *value, dev }, verdict.status)
        }
        Plan::NoModel => (Eval::None, verdict.status),
    };
    Ok((Some(model), eval, status))
}

/// The aggregates of one group of rows that measure the same quantity of the same nuclide.
#[derive(Debug, Clone)]
pub struct Dataset {
    /// Programme.
    pub program: String,
    /// Nuclide or element label.
    pub nuclide: String,
    /// Group of particles.
    pub group: String,
    /// Quantity as the register words it.
    pub quantity: String,
    /// Source of the rows (the first row's).
    pub source: String,
    /// Ids of the rows.
    pub row_ids: Vec<String>,
    /// Number of rows per status.
    pub counts: BTreeMap<Status, usize>,
    /// Aggregates of the point comparisons (COMPARISON AVAILABLE).
    pub point: Option<PointAggregate>,
    /// Aggregates of the bound comparisons.
    pub bound: Option<BoundAggregate>,
    /// Aggregates of the provisional values, kept apart: they are not a comparison.
    pub provisional: Option<PointAggregate>,
    /// Temperatures of the rows, degC, sorted and unique.
    pub temperatures_c: Vec<f64>,
    /// Largest test duration among the compared rows, hours.
    pub max_duration_h: Option<f64>,
}

/// Group the records into datasets and aggregate. Groups never mix nuclides or quantity
/// definitions.
pub fn datasets(records: &[RowRecord]) -> Vec<Dataset> {
    let mut order: Vec<(String, String, String, String)> = Vec::new();
    let mut map: BTreeMap<(String, String, String, String), Vec<&RowRecord>> = BTreeMap::new();
    for r in records {
        let key = (
            r.row.program.as_str().to_string(),
            measured_text(&r.row),
            r.row.group.as_str().to_string(),
            r.row.quantity.as_str().to_string(),
        );
        if !map.contains_key(&key) {
            order.push(key.clone());
        }
        map.entry(key).or_default().push(r);
    }
    order
        .into_iter()
        .map(|key| {
            let rows = &map[&key];
            let mut counts: BTreeMap<Status, usize> = BTreeMap::new();
            let mut points = Vec::new();
            let mut margins = Vec::new();
            let mut provisional = Vec::new();
            let mut temps: Vec<f64> = Vec::new();
            let mut max_dur: Option<f64> = None;
            for r in rows {
                *counts.entry(r.status).or_insert(0) += 1;
                if let strato_data::release::TestTemperature::Celsius(t) = &r.row.temperature {
                    if !temps.contains(&t.value) {
                        temps.push(t.value);
                    }
                }
                if let Some(m) = &r.model {
                    max_dur = Some(max_dur.map_or(m.spec.t_end_h, |x: f64| x.max(m.spec.t_end_h)));
                }
                match (&r.eval, &r.model) {
                    (Eval::Point { value, .. }, Some(m)) => points.push((m.f_window, *value)),
                    (Eval::Bound { margin, .. }, _) => margins.push(*margin),
                    (Eval::Provisional { value, .. }, Some(m)) => {
                        provisional.push((m.f_window, *value))
                    }
                    _ => {}
                }
            }
            temps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            Dataset {
                program: key.0.clone(),
                nuclide: key.1.clone(),
                group: key.2.clone(),
                quantity: key.3.clone(),
                source: rows[0].row.source.clone(),
                row_ids: rows.iter().map(|r| r.row.id.clone()).collect(),
                counts,
                point: point_aggregate(&points),
                bound: bound_aggregate(&margins),
                provisional: point_aggregate(&provisional),
                temperatures_c: temps,
                max_duration_h: max_dur,
            }
        })
        .collect()
}

/// The measured number of a row, when it is a single number.
pub fn measured_number(row: &ReleaseRow) -> Option<f64> {
    single_value(row)
}
