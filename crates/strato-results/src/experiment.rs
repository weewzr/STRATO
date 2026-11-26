//! The experimental side: observations (bounds and points) with their class, premises and
//! reading uncertainty, the bound statuses, and the NRMSE records of `metric-nrmse.md` Section 7.
//!
//! Today the registers hold no compatible point set (science design Section 1.3), so every NRMSE
//! record is empty with its exclusion reasons and no value is produced. The code that would form
//! a set when one exists is exercised by tests through the [`Candidate`] fixture interface and
//! never through production output.

use std::collections::BTreeMap;

use strato_compare::compare::{Eval, RowRecord};
use strato_compare::data::{Radii, Species};
use strato_compare::json::J;
use strato_compare::model::Runner;
use strato_compare::status::{end_time, Plan, Status};
use strato_data::release::{
    Measured, ReadingUncertainty, ReleaseRow, ReleaseValue, TestTemperature, ValueReading,
};
use strato_data::table::Vocabulary;
use strato_data::Register;

use crate::curves::{lin, EngineCurve, Nuc, NUCS};
use crate::methodb::{method_b, MethodB, Needs, NotAvailable};
use crate::nrmse::{self, EmptyReason, Point, FSCALE_DEFINITION_ID};

/// A Method B series at the measurement times: time in h and the value where it is drawn.
type BSeries = Vec<(f64, Option<f64>)>;

/// The bound premise BP1 (science design Section 1.4), verbatim.
pub const BP1: &str = "The reported total contains all material released by the compact during the test (no loss to uncounted surfaces; open point O13).";
/// The bound premise BP2 (science design Section 1.4), verbatim.
pub const BP2: &str = "The kernel-only start (decision 20) differs from the real initial state in two ways with opposite effects: it ignores the inventory already outside the SiC (raises F_p) and the inventory already in the coatings (lowers F_p). The size of the difference is not stated here: the results pipeline computes it for Sr-90 and Cs-137 from the Scenario 5 heating runs against the kernel-only runs of the same histories and writes it into the files curves/s5.refuelling.heating*.json, field kernel_only_comparison. The net direction is not established for Ag-110m and Cs-134.";
/// The assumption A1 (single ideal particle against compact-level data).
pub const A1: &str = "A1: the model fraction is a fraction of one intact particle's inventory; a compact total that is not matrix-corrected includes inventory that was already outside the SiC, so it is at most an upper bound.";
/// The systematic uncertainty the source states for the AGR-1 gamma counting (not combined into any number).
pub const SYSTEMATIC_AGR1: &str = "5 % gamma counting (Stempien 2023 p. 10), kept separate and not combined into any number on a graph";

/// Bound comparisons of intact compacts that the science design adds to the register-based ones
/// (Section 1.3): end-of-test totals of Sr-90 and Ag-110m, and the rows of the time series.
const DESIGN_BOUND_RANGES: [(&str, u32, u32); 18] = [
    ("sr90", 63, 68),
    ("sr90", 157, 157),
    ("sr90", 163, 163),
    ("sr90", 165, 165),
    ("sr90", 168, 169),
    ("sr90", 171, 171),
    ("sr90", 472, 482),
    ("ag110m", 46, 46),
    ("ag110m", 48, 48),
    ("ag110m", 51, 53),
    ("ag110m", 55, 55),
    ("ag110m", 172, 172),
    ("ag110m", 178, 178),
    ("ag110m", 180, 180),
    ("ag110m", 183, 184),
    ("ag110m", 186, 186),
    ("ag110m", 425, 434),
    ("ag110m", 462, 471),
];

/// The matrix-corrected rows of intact compacts (decision 67): fractions of the as-run calculated
/// compact inventory over a window from about 24 h at temperature to the end of the 300 h plateau.
/// They are window upper bounds under premise BP1 and never point comparisons: the source calls
/// the correction an attempt that cannot separate particle release from matrix release.
const MATRIX_CORRECTED_RANGES: [(&str, u32, u32); 2] = [("cs134", 10, 14), ("ag110m", 33, 37)];

/// Length of the plateau that ends the window of a matrix-corrected row, hours.
const PLATEAU_H: f64 = 300.0;
/// The start of the window of a matrix-corrected row ("about 24 h" in the source), hours.
const WINDOW_START_H: f64 = 24.0;

/// The note that goes with BP1 for the matrix-corrected rows (decision 67).
pub const BP1_SOURCE_NOTE: &str = "For the matrix-corrected rows BP1 is the source's own assumption, not an established fact: the authors call the correction an attempt that cannot separate particle release from matrix release. The window runs from about 24 h at temperature to the end of the 300 h plateau, or to the end of the test if shorter.";

/// True if the row is a matrix-corrected row of an intact compact (decision 67).
pub fn is_matrix_corrected(nuc_id: &str, id: &str) -> bool {
    let Some(n) = id_number(id) else { return false };
    MATRIX_CORRECTED_RANGES
        .iter()
        .any(|(nu, a, b)| *nu == nuc_id && (*a..=*b).contains(&n))
}

fn id_number(id: &str) -> Option<u32> {
    id.strip_prefix("E-")?.parse().ok()
}

/// True if the science design lists the row as a bound comparison of an intact compact (a total
/// that includes the matrix, used as an upper bound, Section 1.4).
pub fn is_design_bound(nuc_id: &str, id: &str) -> bool {
    let Some(n) = id_number(id) else { return false };
    DESIGN_BOUND_RANGES
        .iter()
        .any(|(nu, a, b)| *nu == nuc_id && (*a..=*b).contains(&n))
}
/// The time series whose increases from the first point are window bounds (Section 1.4).
const WINDOW_SERIES: [(&str, &str, u32, u32, f64); 3] = [
    ("sr90", "Sr-90 compact 6-4-3 (Fig. 12)", 472, 482, 1600.0),
    ("ag110m", "Ag-110m compact 4-3-3 (Fig. 9)", 425, 434, 1600.0),
    ("ag110m", "Ag-110m compact 6-4-3 (Fig. 9)", 462, 471, 1600.0),
];

/// A nuclide of the cells by id.
fn nuc_by_id(id: &str) -> Option<&'static Nuc> {
    NUCS.iter().find(|n| n.id == id)
}

/// The nuclide id of a register row, if it measures one of the cell nuclides (or the element of
/// one of them, compared as the element).
pub fn row_nuclide(row: &ReleaseRow) -> Option<&'static Nuc> {
    match &row.measured {
        Measured::Nuclide(n) => NUCS.iter().find(|x| x.name == n.name()),
        Measured::Element(e) => NUCS.iter().find(|x| x.species.label() == e.as_str()),
        Measured::NotApplicable => None,
    }
}

/// The test temperature of a row, degC, when it is one temperature.
pub fn row_temperature(row: &ReleaseRow) -> Option<f64> {
    match &row.temperature {
        TestTemperature::Celsius(p) => Some(p.value),
        _ => None,
    }
}

/// Absolute reading uncertainty of the value and of the time (hours) of a row.
pub fn reading(row: &ReleaseRow, value: f64) -> (Option<f64>, Option<f64>) {
    match &row.reading_uncertainty {
        ReadingUncertainty::Stated { value: v, time_h } => {
            let u = match v {
                Some(ValueReading::RelativePercent(p)) => Some(p.value / 100.0 * value),
                Some(ValueReading::Absolute(p)) => Some(p.value),
                _ => None,
            };
            (u, time_h.as_ref().map(|t| t.value))
        }
        _ => (None, None),
    }
}

fn reading_text(row: &ReleaseRow) -> String {
    strato_compare::compare::reading_text(&row.reading_uncertainty)
}

fn single(row: &ReleaseRow) -> Option<f64> {
    strato_compare::status::single_value(row)
}

/// One bound comparison with its status.
#[derive(Debug, Clone)]
pub struct BoundRec {
    /// Row id or `W:<start>:<end>`.
    pub id: String,
    /// `reported_bound`, `total_as_bound` or `window_bound`.
    pub kind: &'static str,
    /// Nuclide id.
    pub nuc: &'static Nuc,
    /// Test temperature, degC.
    pub t_c: f64,
    /// Compact.
    pub compact: String,
    /// Hold time of the bound, hours (end of the window).
    pub t_h: f64,
    /// Start of the window, hours.
    pub t_start_h: f64,
    /// The bound.
    pub bound: f64,
    /// Reading interval of the bound.
    pub interval: (f64, f64),
    /// Time reading uncertainty, hours.
    pub u_time_h: Option<f64>,
    /// Model A.
    pub model_a: Option<f64>,
    /// Model A of the refined run.
    pub model_a_refined: Option<f64>,
    /// Model A at `t - u_t` and `t + u_t`.
    pub model_a_pm_t: Option<(f64, f64)>,
    /// Method B where the mask accepts it.
    pub model_b: Option<f64>,
    /// Status text.
    pub status: String,
    /// Reason when the status is NOT ASSESSED.
    pub reason: Option<&'static str>,
    /// True if the status is the same at every combination of the reading ends.
    pub robust: Option<bool>,
    /// Value class text of the row.
    pub value_class: String,
    /// Source text.
    pub source: String,
    /// Source location.
    pub source_location: String,
    /// Denominator text.
    pub denominator: String,
    /// Reading uncertainty text.
    pub reading: String,
    /// Draw the marker (false for a digitised duplicate of a printed value).
    pub drawn: bool,
    /// The printed row that replaces a digitised duplicate.
    pub duplicate_of: Option<String>,
    /// Row ids of a window bound's two ends.
    pub ends: Option<(String, String)>,
    /// The window-start sensitivity of a matrix-corrected window row (science design 3.3 item 3).
    pub sensitivity: Vec<StartSensitivity>,
}

/// The model increase of a matrix-corrected window for another start of the window.
#[derive(Debug, Clone)]
pub struct StartSensitivity {
    /// The assumed start of the window, hours at temperature.
    pub start_h: f64,
    /// Method A increase `F(t_end) - F(start)`.
    pub model_a: Option<f64>,
    /// The same from the refined run.
    pub model_a_refined: Option<f64>,
    /// Method B increase where it applies.
    pub model_b: Option<f64>,
}

/// The starts of the window tried as a sensitivity (science design Section 3.3 item 3), hours.
pub const SENSITIVITY_STARTS_H: [f64; 2] = [20.0, 28.0];

fn status_of(model: f64, bound: f64) -> &'static str {
    if model > bound {
        "EXCEEDS REPORTED UPPER BOUND"
    } else {
        "WITHIN REPORTED UPPER BOUND"
    }
}

/// All bound comparisons of the cells (decision 23, science design Section 1.4) with their
/// statuses, computed at the as-run end time of the test.
///
/// # Errors
///
/// A model run that fails.
pub fn bounds(
    reg: &Register,
    records: &[RowRecord],
    runner: &mut Runner<'_>,
    radii: &Radii,
) -> Result<Vec<BoundRec>, String> {
    // Candidates: (row, nuclide, temperature, hold time, window start, kind).
    struct Cand<'a> {
        row: &'a ReleaseRow,
        nuc: &'static Nuc,
        t_c: f64,
        t_h: f64,
        t_start_h: f64,
        kind: &'static str,
    }
    let mut cands: Vec<Cand<'_>> = Vec::new();
    for rec in records {
        let row = &rec.row;
        let (Some(nuc), Some(t_c)) = (row_nuclide(row), row_temperature(row)) else {
            continue;
        };
        if !super::curves::exp_temperatures(nuc.id).contains(&t_c) {
            continue;
        }
        if is_matrix_corrected(nuc.id, &row.id) {
            // Decision 67: a window upper bound under BP1, never a point.
            // The comparison program plans these rows as a point or as a provisional model value
            // (before the register fixes the denominator); the window is the same.
            if let Plan::Point { spec, .. } | Plan::Provisional { spec, .. } = &rec.verdict.plan {
                cands.push(Cand {
                    row,
                    nuc,
                    t_c,
                    t_h: spec.t_end_h.min(PLATEAU_H),
                    t_start_h: if spec.window_start_h > 0.0 {
                        spec.window_start_h
                    } else {
                        WINDOW_START_H
                    },
                    kind: "window_bound",
                });
            }
            continue;
        }
        let from_compare = match &rec.verdict.plan {
            Plan::Inequality { spec, .. } if matches!(rec.eval, Eval::Bound { .. }) => {
                Some((spec.t_end_h, spec.window_start_h))
            }
            _ => None,
        };
        let design = is_design_bound(nuc.id, &row.id);
        if from_compare.is_none() && !design {
            continue;
        }
        let (t_h, t_start) = match from_compare {
            Some(x) => x,
            None => match end_time(reg, row, t_c) {
                Some((t, _)) => (t, 0.0),
                None => continue,
            },
        };
        let kind = if matches!(row.value, ReleaseValue::UpperBound(_)) {
            "reported_bound"
        } else {
            "total_as_bound"
        };
        cands.push(Cand {
            row,
            nuc,
            t_c,
            t_h,
            t_start_h: t_start,
            kind,
        });
    }
    // Window bounds from the three series.
    struct Win<'a> {
        a: &'a ReleaseRow,
        b: &'a ReleaseRow,
        nuc: &'static Nuc,
        t_c: f64,
    }
    let mut wins: Vec<Win<'_>> = Vec::new();
    for (nuc_id, _name, lo, hi, t_c) in WINDOW_SERIES {
        let nuc = nuc_by_id(nuc_id).ok_or("nuclide")?;
        let mut rows: Vec<&ReleaseRow> = reg
            .release
            .rows
            .iter()
            .filter(|r| id_number(&r.id).is_some_and(|n| (lo..=hi).contains(&n)))
            .collect();
        rows.sort_by(|a, b| {
            let (ta, tb) = (
                a.time_h.as_ref().map(|x| x.value),
                b.time_h.as_ref().map(|x| x.value),
            );
            ta.partial_cmp(&tb).unwrap_or(std::cmp::Ordering::Equal)
        });
        if let Some(first) = rows.first().copied() {
            for b in rows.iter().skip(1) {
                wins.push(Win {
                    a: first,
                    b,
                    nuc,
                    t_c,
                });
            }
        }
    }
    // Curves up to the longest time.
    for c in &cands {
        runner.get(c.nuc.species, c.t_c, c.t_h.max(1.0))?;
    }
    for w in &wins {
        runner.get(
            w.nuc.species,
            w.t_c,
            w.b.time_h.as_ref().map_or(1.0, |t| t.value),
        )?;
    }
    // Method B at every time of a cell, in one call per cell.
    let mut b_times: BTreeMap<(&'static str, i64), Vec<f64>> = BTreeMap::new();
    for c in &cands {
        b_times.entry((c.nuc.id, c.t_c as i64)).or_default().extend(
            [c.t_h * 3600.0, c.t_start_h * 3600.0]
                .into_iter()
                .filter(|t| *t > 0.0),
        );
        if c.kind == "window_bound" {
            b_times
                .entry((c.nuc.id, c.t_c as i64))
                .or_default()
                .extend(SENSITIVITY_STARTS_H.iter().map(|h| h * 3600.0));
        }
    }
    for w in &wins {
        let e = b_times.entry((w.nuc.id, w.t_c as i64)).or_default();
        e.push(w.b.time_h.as_ref().map_or(0.0, |t| t.value) * 3600.0);
        e.push(w.a.time_h.as_ref().map_or(0.0, |t| t.value) * 3600.0);
    }
    let needs = Needs {
        generation: false,
        decay: false,
        temperature_varies: false,
        kernel_step_start: true,
    };
    let mut b_series: BTreeMap<(&'static str, i64), BSeries> = BTreeMap::new();
    for ((nid, t), times) in &b_times {
        let nuc = nuc_by_id(nid).ok_or("nuclide")?;
        let curve = runner.get(nuc.species, *t as f64, 1.0)?;
        let d = curve.d_m2_s;
        let mut ts = times.clone();
        ts.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        ts.dedup();
        if let MethodB::Available(a) = method_b(needs, &radii.outer_m, &d, &ts) {
            b_series.insert(
                (*nid, *t),
                a.times_s
                    .iter()
                    .copied()
                    .zip(a.values.iter().copied())
                    .collect(),
            );
        }
    }
    let b_at = |nid: &'static str, t_c: f64, t_h: f64| -> Option<f64> {
        let s = b_series.get(&(nid, t_c as i64))?;
        s.iter()
            .find(|(t, _)| (t - t_h * 3600.0).abs() < 1e-6)
            .and_then(|(_, v)| *v)
    };

    let mut out: Vec<BoundRec> = Vec::new();
    for c in &cands {
        let row = c.row;
        let Some(limit) = single(row) else { continue };
        let curve = runner.get(c.nuc.species, c.t_c, c.t_h.max(1.0))?.clone();
        let (u_v, u_t) = reading(row, limit);
        let fa = |t: f64| curve.at(t);
        let model = fa(c.t_h).map(|e| {
            e - if c.t_start_h > 0.0 {
                fa(c.t_start_h).unwrap_or(0.0)
            } else {
                0.0
            }
        });
        let refined = curve.at_refined(c.t_h).map(|e| {
            e - if c.t_start_h > 0.0 {
                curve.at_refined(c.t_start_h).unwrap_or(0.0)
            } else {
                0.0
            }
        });
        let resolved =
            matches!((model, refined), (Some(m), Some(r)) if r > 0.0 && (m - r).abs() <= 0.05 * r);
        let ut = u_t.unwrap_or(0.0);
        let start_value = if c.t_start_h > 0.0 {
            fa(c.t_start_h).unwrap_or(0.0)
        } else {
            0.0
        };
        let pm = match (fa(c.t_h - ut), fa(c.t_h + ut)) {
            (Some(a), Some(b)) => Some((a - start_value, b - start_value)),
            _ => None,
        };
        let (status, reason) = match (model, resolved) {
            (Some(m), true) => (status_of(m, limit).to_string(), None),
            _ => ("NOT ASSESSED".to_string(), Some("model_not_resolved")),
        };
        let ua = u_v.unwrap_or(0.0);
        let interval = (limit - ua, limit + ua);
        let robust = match (model, pm, resolved) {
            (Some(_), Some((lo, hi)), true) => {
                let mut s = std::collections::BTreeSet::new();
                for m in [lo, hi] {
                    for b in [interval.0, interval.1] {
                        s.insert(status_of(m, b));
                    }
                }
                Some(s.len() == 1)
            }
            _ => None,
        };
        let sensitivity: Vec<StartSensitivity> = if c.kind == "window_bound" {
            SENSITIVITY_STARTS_H
                .iter()
                .map(|&s| StartSensitivity {
                    start_h: s,
                    model_a: fa(c.t_h).zip(fa(s)).map(|(e, b)| e - b),
                    model_a_refined: curve
                        .at_refined(c.t_h)
                        .zip(curve.at_refined(s))
                        .map(|(e, b)| e - b),
                    model_b: b_at(c.nuc.id, c.t_c, c.t_h)
                        .zip(b_at(c.nuc.id, c.t_c, s))
                        .map(|(e, b)| e - b),
                })
                .collect()
        } else {
            Vec::new()
        };
        out.push(BoundRec {
            id: row.id.clone(),
            kind: c.kind,
            nuc: c.nuc,
            t_c: c.t_c,
            compact: row.compact_or_sample.clone(),
            t_h: c.t_h,
            t_start_h: c.t_start_h,
            bound: limit,
            interval,
            u_time_h: u_t,
            model_a: model,
            model_a_refined: refined,
            model_a_pm_t: pm,
            model_b: if c.t_start_h > 0.0 {
                b_at(c.nuc.id, c.t_c, c.t_h)
                    .zip(b_at(c.nuc.id, c.t_c, c.t_start_h))
                    .map(|(e, s)| e - s)
            } else {
                b_at(c.nuc.id, c.t_c, c.t_h)
            },
            status,
            reason,
            robust,
            value_class: row.value_class.as_str().to_string(),
            source: row.source.clone(),
            source_location: row.source_location.clone(),
            denominator: row.denominator.as_str().to_string(),
            reading: reading_text(row),
            drawn: true,
            duplicate_of: None,
            ends: None,
            sensitivity,
        });
    }
    for w in &wins {
        let (Some(ta), Some(tb)) = (w.a.time_h.as_ref(), w.b.time_h.as_ref()) else {
            continue;
        };
        let (Some(va), Some(vb)) = (single(w.a), single(w.b)) else {
            continue;
        };
        let curve = runner.get(w.nuc.species, w.t_c, tb.value)?.clone();
        let limit = vb - va;
        let (ua, uta) = reading(w.a, va);
        let (ub, utb) = reading(w.b, vb);
        let ru = ua.unwrap_or(0.0) + ub.unwrap_or(0.0);
        let interval = (limit - ru, limit + ru);
        let inc = |f: &dyn Fn(f64) -> Option<f64>| Some(f(tb.value)? - f(ta.value)?);
        let model = inc(&|t| curve.at(t));
        let refined = inc(&|t| curve.at_refined(t));
        let resolved =
            matches!((model, refined), (Some(m), Some(r)) if r > 0.0 && (m - r).abs() <= 0.05 * r);
        let (status, reason) = match (model, resolved) {
            (Some(m), true) => (status_of(m, limit).to_string(), None),
            _ => ("NOT ASSESSED".to_string(), Some("model_not_resolved")),
        };
        let ut = uta.unwrap_or(0.0) + utb.unwrap_or(0.0);
        let pm = match (
            curve
                .at(tb.value - utb.unwrap_or(0.0))
                .zip(curve.at(ta.value + uta.unwrap_or(0.0))),
            curve
                .at(tb.value + utb.unwrap_or(0.0))
                .zip(curve.at(ta.value - uta.unwrap_or(0.0))),
        ) {
            (Some((b1, a1)), Some((b2, a2))) => Some((b1 - a1, b2 - a2)),
            _ => None,
        };
        let robust = match (model, pm, resolved) {
            (Some(_), Some((lo, hi)), true) => {
                let mut s = std::collections::BTreeSet::new();
                for m in [lo, hi] {
                    for b in [interval.0, interval.1] {
                        s.insert(status_of(m, b));
                    }
                }
                Some(s.len() == 1)
            }
            _ => None,
        };
        out.push(BoundRec {
            id: format!("W:{}:{}", w.a.id, w.b.id),
            kind: "window_bound",
            nuc: w.nuc,
            t_c: w.t_c,
            compact: w.b.compact_or_sample.clone(),
            t_h: tb.value,
            t_start_h: ta.value,
            bound: limit,
            interval,
            u_time_h: Some(ut),
            model_a: model,
            model_a_refined: refined,
            model_a_pm_t: pm,
            model_b: match (
                b_at(w.nuc.id, w.t_c, tb.value),
                b_at(w.nuc.id, w.t_c, ta.value),
            ) {
                (Some(b), Some(a)) => Some(b - a),
                _ => None,
            },
            status,
            reason,
            robust,
            value_class: "calculated from registered values".to_string(),
            source: w.b.source.clone(),
            source_location: w.b.source_location.clone(),
            denominator: w.b.denominator.as_str().to_string(),
            reading: "the sum of the two absolute reading uncertainties, no independence assumed"
                .to_string(),
            drawn: false,
            duplicate_of: None,
            ends: Some((w.a.id.clone(), w.b.id.clone())),
            sensitivity: Vec::new(),
        });
    }
    // A digitised total and a printed value of the same compact, nuclide and time are the same
    // quantity: the graph draws the printed one (science design Section 1.4).
    let by_key: Vec<(usize, String)> = out
        .iter()
        .enumerate()
        .filter(|(_, b)| b.kind != "window_bound")
        .map(|(i, b)| {
            (
                i,
                format!("{}|{}|{:.1}|{}", b.compact, b.nuc.id, b.t_h, b.t_c),
            )
        })
        .collect();
    for (i, key) in &by_key {
        let printed: Vec<usize> = by_key
            .iter()
            .filter(|(j, k)| {
                k == key
                    && j != i
                    && out[*j].value_class.starts_with('M')
                    && !out[*j].value_class.contains("dig")
            })
            .map(|(j, _)| *j)
            .collect();
        let is_dig = out[*i].value_class.contains("dig");
        if is_dig && !printed.is_empty() {
            out[*i].drawn = false;
            out[*i].duplicate_of = Some(out[printed[0]].id.clone());
        }
    }
    Ok(out)
}

/// The JSON of a bound comparison (science design Section 7.3, `bounds`).
pub fn bound_json(b: &BoundRec) -> J {
    let ratio = |m: Option<f64>| J::opt(m.map(|m| m / b.bound));
    J::obj(vec![
        ("id", J::s(b.id.clone())),
        ("kind", J::s(b.kind)),
        ("nuclide", J::s(b.nuc.name)),
        ("temperature_c", J::Num(b.t_c)),
        ("compact", J::s(b.compact.clone())),
        ("time_h", J::Num(b.t_h)),
        (
            "window_start_h",
            if b.kind == "window_bound" {
                J::Num(b.t_start_h)
            } else {
                J::Null
            },
        ),
        ("bound_value", J::Num(b.bound)),
        (
            "bound_reading_interval",
            J::Arr(vec![J::Num(b.interval.0), J::Num(b.interval.1)]),
        ),
        ("model_A", J::opt(b.model_a)),
        ("model_A_refined", J::opt(b.model_a_refined)),
        (
            "model_at_t_pm_ut",
            match b.model_a_pm_t {
                Some((a, c)) => J::Arr(vec![J::Num(a), J::Num(c)]),
                None => J::Null,
            },
        ),
        ("model_B", J::opt(b.model_b)),
        ("ratio_A", ratio(b.model_a)),
        ("ratio_B", ratio(b.model_b)),
        ("status", J::s(b.status.clone())),
        ("not_assessed_reason", b.reason.map_or(J::Null, J::s)),
        (
            "status_robust_to_reading_uncertainty",
            b.robust.map_or(J::Null, J::Bool),
        ),
        ("value_class", J::s(b.value_class.clone())),
        (
            "drawn_on_graph",
            J::Bool(b.drawn && b.kind != "window_bound"),
        ),
        ("duplicate_of", b.duplicate_of.clone().map_or(J::Null, J::s)),
        (
            "window_ends",
            match &b.ends {
                Some((a, c)) => J::Arr(vec![J::s(a.clone()), J::s(c.clone())]),
                None => J::Null,
            },
        ),
        (
            "window_start_sensitivity",
            if b.sensitivity.is_empty() {
                J::Null
            } else {
                J::obj(vec![
                    ("label", J::s("prescribed input (assumed)")),
                    (
                        "reason",
                        J::s("the source gives no exact start of the window (\"about 24 h\"); the registered value used above is 24 h"),
                    ),
                    (
                        "starts",
                        J::Arr(
                            b.sensitivity
                                .iter()
                                .map(|s| {
                                    let resolved = matches!(
                                        (s.model_a, s.model_a_refined),
                                        (Some(m), Some(r)) if r > 0.0 && (m - r).abs() <= 0.05 * r
                                    );
                                    J::obj(vec![
                                        ("window_start_h", J::Num(s.start_h)),
                                        ("model_A", J::opt(s.model_a)),
                                        ("model_A_refined", J::opt(s.model_a_refined)),
                                        ("model_A_resolved", J::Bool(resolved)),
                                        ("ratio_A", J::opt(s.model_a.map(|m| m / b.bound))),
                                        ("model_B", J::opt(s.model_b)),
                                        ("ratio_B", J::opt(s.model_b.map(|m| m / b.bound))),
                                    ])
                                })
                                .collect(),
                        ),
                    ),
                ])
            },
        ),
        ("source", J::s(b.source.clone())),
        ("source_location", J::s(b.source_location.clone())),
        ("denominator", J::s(b.denominator.clone())),
        ("reading_uncertainty", J::s(b.reading.clone())),
        ("systematic_uncertainty", J::s(SYSTEMATIC_AGR1)),
        (
            "premises",
            J::Arr(if b.kind == "window_bound" && b.ends.is_none() {
                vec![J::s(BP1), J::s(BP1_SOURCE_NOTE), J::s(BP2), J::s(A1)]
            } else {
                vec![J::s(BP1), J::s(BP2), J::s(A1)]
            }),
        ),
    ])
}

// ---------------------------------------------------------------------------------------
// NRMSE records
// ---------------------------------------------------------------------------------------

/// A row as it enters the formation of a comparison set.
#[derive(Debug, Clone)]
pub enum Candidate {
    /// The row never enters N; the reason code of `metric-nrmse.md` Section 4.
    Excluded {
        /// Register row id.
        id: String,
        /// Reason code.
        reason: &'static str,
    },
    /// A compatible point with its aligned model values.
    Compatible {
        /// Register row id.
        id: String,
        /// Compact or test.
        test_id: String,
        /// Registered value.
        f_exp: f64,
        /// Method A at the aligned time or window.
        f_a: f64,
        /// True if Method A is resolved at this point (Section 3.5).
        resolved_a: bool,
        /// Method B at the aligned time or window, if the mask accepts it.
        f_b: Option<f64>,
        /// Reading uncertainty of the value, absolute.
        u_value: Option<f64>,
        /// Reading uncertainty of the time, hours.
        u_time_h: Option<f64>,
        /// Slope of Method A per hour.
        slope_a_per_h: f64,
        /// Slope of Method B per hour.
        slope_b_per_h: f64,
    },
}

/// What identifies one comparison set.
#[derive(Debug, Clone)]
pub struct SetMeta {
    /// `nrmse.<config>`.
    pub set_id: String,
    /// Nuclide name.
    pub nuclide: String,
    /// Test temperature.
    pub temperature_c: f64,
    /// The model configuration id.
    pub model_config_id: String,
}

/// One record of `metric-nrmse.md` Section 7.
#[derive(Debug, Clone)]
pub struct NrmseRecord {
    /// The set.
    pub meta: SetMeta,
    /// `A` or `B`.
    pub method: char,
    /// `N`.
    pub n: usize,
    /// The value, when not empty.
    pub value: Option<nrmse::Value>,
    /// The display string.
    pub display: Option<String>,
    /// Empty reason.
    pub empty_reason: Option<EmptyReason>,
    /// Test ids.
    pub test_ids: Vec<String>,
    /// Rows used.
    pub rows_used: Vec<String>,
    /// Rows excluded as `E-id:reason`.
    pub rows_excluded: Vec<String>,
    /// Whether Method B applies (always true for A).
    pub method_applicable: bool,
    /// Unresolved row ids.
    pub unresolved: Vec<String>,
}

/// Form the records of one set for Method A and Method B from its candidates. `b_applicable` is
/// the applicability of Method B for the configuration (`Ok`) or the reason it is not.
pub fn records(
    meta: &SetMeta,
    cands: &[Candidate],
    b_applicable: &Result<(), NotAvailable>,
) -> [NrmseRecord; 2] {
    let mut used = Vec::new();
    let mut excluded = Vec::new();
    let mut tests: Vec<String> = Vec::new();
    let mut pa: Vec<Point> = Vec::new();
    let mut pb: Vec<Option<Point>> = Vec::new();
    let mut unresolved_a: Vec<String> = Vec::new();
    for c in cands {
        match c {
            Candidate::Excluded { id, reason } => excluded.push(format!("{id}:{reason}")),
            Candidate::Compatible {
                id,
                test_id,
                f_exp,
                f_a,
                resolved_a,
                f_b,
                u_value,
                u_time_h,
                slope_a_per_h,
                slope_b_per_h,
            } => {
                if *f_exp <= 0.0 {
                    excluded.push(format!("{id}:nonpositive_value"));
                    continue;
                }
                used.push(id.clone());
                if !tests.contains(test_id) {
                    tests.push(test_id.clone());
                }
                if !resolved_a {
                    unresolved_a.push(id.clone());
                }
                pa.push(Point {
                    id: id.clone(),
                    f_exp: *f_exp,
                    f_model: *f_a,
                    u_value: *u_value,
                    u_time_h: *u_time_h,
                    slope_per_h: *slope_a_per_h,
                });
                pb.push(f_b.map(|fb| Point {
                    id: id.clone(),
                    f_exp: *f_exp,
                    f_model: fb,
                    u_value: *u_value,
                    u_time_h: *u_time_h,
                    slope_per_h: *slope_b_per_h,
                }));
            }
        }
    }
    let make = |method: char,
                outcome: Result<nrmse::Value, EmptyReason>,
                applicable: bool,
                unresolved: Vec<String>| {
        let (value, display, empty, n) = match outcome {
            Ok(v) => {
                let d = nrmse::display(v.nrmse_percent);
                let n = v.n;
                (Some(v), Some(d), None, n)
            }
            Err(e) => (None, None, Some(e), pa.len()),
        };
        NrmseRecord {
            meta: meta.clone(),
            method,
            n,
            value,
            display,
            empty_reason: empty,
            test_ids: tests.clone(),
            rows_used: used.clone(),
            rows_excluded: excluded.clone(),
            method_applicable: applicable,
            unresolved,
        }
    };
    // Method A: unresolved points make the whole record empty, never a subset (Section 3.5).
    let a_outcome = if !unresolved_a.is_empty() {
        Err(EmptyReason::ModelNotResolved)
    } else {
        nrmse::compute(&pa)
    };
    // Method B: only where it applies, and only on the same points.
    let b_outcome = if b_applicable.is_err() {
        Err(EmptyReason::MethodBNotApplicable)
    } else if pa.is_empty() {
        Err(EmptyReason::NoCompatiblePoints)
    } else if pb.iter().any(Option::is_none) {
        Err(EmptyReason::ModelNotResolved)
    } else {
        nrmse::compute(&pb.iter().flatten().cloned().collect::<Vec<_>>())
    };
    let a = make('A', a_outcome, true, unresolved_a);
    let b = make('B', b_outcome, b_applicable.is_ok(), vec![]);
    [a, b]
}

/// Where the model values of the candidates come from.
pub enum ModelSource<'a> {
    /// The isothermal kernel-only run of stage 1 (Method A from the runner, Method B at the same
    /// times where it applies).
    Isothermal {
        /// The particle radii.
        radii: &'a Radii,
    },
    /// The run of the registered ramp history (stage 4): values at `t_mod = t_ramp_end + t_hold`.
    Ramp(&'a EngineCurve),
}

/// The `Candidate`s of a cell from the comparison records: every register row of the nuclide at
/// the temperature, each either compatible or excluded with its reason code
/// (`metric-nrmse.md` Section 4).
pub fn candidates(
    records: &[RowRecord],
    runner: &mut Runner<'_>,
    nuc: &Nuc,
    t_c: f64,
    source: ModelSource<'_>,
) -> Result<Vec<Candidate>, String> {
    let mut out = Vec::new();
    for rec in records {
        let row = &rec.row;
        if row_nuclide(row).map(|n| n.id) != Some(nuc.id) || row_temperature(row) != Some(t_c) {
            continue;
        }
        let id = row.id.clone();
        let excluded = |reason: &'static str| Candidate::Excluded {
            id: id.clone(),
            reason,
        };
        if matches!(row.value_class, strato_data::release::ValueClass::NonDetect) {
            out.push(excluded("non_detect"));
            continue;
        }
        if matches!(row.value, ReleaseValue::UpperBound(_))
            || matches!(rec.eval, Eval::Bound { .. })
            || is_design_bound(nuc.id, &row.id)
            || is_matrix_corrected(nuc.id, &row.id)
        {
            out.push(excluded("upper_bound"));
            continue;
        }
        if rec.status == Status::DataIncomplete {
            out.push(excluded("denominator_not_established"));
            continue;
        }
        let Eval::Point { value, .. } = rec.eval else {
            out.push(excluded("comparison_incompatible"));
            continue;
        };
        if value <= 0.0 {
            out.push(excluded("nonpositive_value"));
            continue;
        }
        let Plan::Point { spec, .. } = &rec.verdict.plan else {
            out.push(excluded("comparison_incompatible"));
            continue;
        };
        if row.time_basis.contains("elapsed") {
            out.push(excluded("time_origin_not_established"));
            continue;
        }
        let (u_value, u_time) = reading(row, value);
        let dt = 0.05;
        let t_end = spec.t_end_h;
        let t_start = spec.window_start_h.max(0.0);
        // Model values at the aligned time: the isothermal run at the hold time (stage 1), or the
        // ramp run at t_mod = t_ramp_end + t_hold (stage 4, the sample times of the ramp curve are
        // hold times from the end of the last ramp).
        let (f_a, resolved_a, slope_a, f_b, slope_b) = match &source {
            ModelSource::Isothermal { radii } => {
                let curve = runner.get(nuc.species, t_c, t_end)?.clone();
                let at = |t: f64| curve.at(t);
                let (Some(f_end), Some(r_end)) = (at(t_end), curve.at_refined(t_end)) else {
                    out.push(excluded("beyond_model_end_time"));
                    continue;
                };
                let (f_start, r_start) = if t_start > 0.0 {
                    (
                        at(t_start).unwrap_or(0.0),
                        curve.at_refined(t_start).unwrap_or(0.0),
                    )
                } else {
                    (0.0, 0.0)
                };
                let f_a = f_end - f_start;
                let r = r_end - r_start;
                let slope_a = match (at(t_end + dt), at((t_end - dt).max(0.0))) {
                    (Some(p), Some(m)) => (p - m) / (2.0 * dt),
                    _ => 0.0,
                };
                // Method B at the same times, where it applies and the drawing mask accepts it.
                let needs = Needs {
                    generation: false,
                    decay: false,
                    temperature_varies: false,
                    kernel_step_start: true,
                };
                let mut ts = vec![
                    t_end * 3600.0,
                    (t_end + dt) * 3600.0,
                    ((t_end - dt).max(0.0)) * 3600.0,
                ];
                if t_start > 0.0 {
                    ts.push(t_start * 3600.0);
                }
                ts.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
                ts.dedup();
                let (f_b, slope_b) = match method_b(needs, &radii.outer_m, &curve.d_m2_s, &ts) {
                    MethodB::Available(a) => {
                        let get = |h: f64| {
                            a.times_s
                                .iter()
                                .position(|t| (t - h * 3600.0).abs() < 1e-6)
                                .and_then(|k| a.values[k])
                        };
                        let start = if t_start > 0.0 {
                            get(t_start)
                        } else {
                            Some(0.0)
                        };
                        let f_b = get(t_end).zip(start).map(|(e, s)| e - s);
                        let slope_b = match (get((t_end + dt).max(0.0)), get((t_end - dt).max(0.0)))
                        {
                            (Some(p), Some(m)) => (p - m) / (2.0 * dt),
                            _ => 0.0,
                        };
                        (f_b, slope_b)
                    }
                    MethodB::NotAvailable(_) => (None, 0.0),
                };
                (
                    f_a,
                    r > 0.0 && (f_a - r).abs() <= 0.05 * r,
                    slope_a,
                    f_b,
                    slope_b,
                )
            }
            ModelSource::Ramp(ec) => {
                let at = |h: f64| lin(&ec.t_h, &ec.f_a, h);
                let flag = |h: f64| {
                    ec.t_h
                        .iter()
                        .position(|t| (t - h).abs() < 1e-9)
                        .map(|k| ec.resolved[k])
                };
                let Some(f_end) = at(t_end) else {
                    out.push(excluded("beyond_model_end_time"));
                    continue;
                };
                let f_start = if t_start > 0.0 {
                    at(t_start).unwrap_or(0.0)
                } else {
                    0.0
                };
                let resolved = flag(t_end).unwrap_or(false)
                    && (t_start <= 0.0 || flag(t_start).unwrap_or(false));
                let slope_a = match (at(t_end + dt), at((t_end - dt).max(0.0))) {
                    (Some(p), Some(m)) => (p - m) / (2.0 * dt),
                    _ => 0.0,
                };
                (f_end - f_start, resolved, slope_a, None, 0.0)
            }
        };
        out.push(Candidate::Compatible {
            id: id.clone(),
            test_id: row.compact_or_sample.clone(),
            f_exp: value,
            f_a,
            resolved_a,
            f_b,
            u_value,
            u_time_h: u_time,
            slope_a_per_h: slope_a,
            slope_b_per_h: slope_b,
        });
    }
    // Rows of the same nuclide that the comparison program leaves out entirely (statuses with no
    // model) are already in `records`; nothing else exists.
    Ok(out)
}

/// The JSON of a record (the fields of `metric-nrmse.md` Section 7).
pub fn record_json(r: &NrmseRecord) -> J {
    let v = r.value.as_ref();
    J::obj(vec![
        ("set_id", J::s(format!("{}.{}", r.meta.set_id, r.method))),
        ("nuclide", J::s(r.meta.nuclide.clone())),
        (
            "quantity",
            J::s("cumulative release fraction of the particle inventory (dimensionless)"),
        ),
        (
            "denominator_class",
            J::s("established inventory equal to the particle-inventory basis of the model"),
        ),
        ("time_basis", J::s("time at test temperature (h)")),
        ("temperature_C", J::Num(r.meta.temperature_c)),
        ("test_ids", J::Arr(r.test_ids.iter().map(J::s).collect())),
        ("model_config_id", J::s(r.meta.model_config_id.clone())),
        ("method", J::s(r.method.to_string())),
        ("N", J::Int(r.n as i64)),
        ("f_scale", J::opt(v.map(|v| v.f_scale))),
        ("f_scale_definition_id", J::s(FSCALE_DEFINITION_ID)),
        (
            "f_scale_row_id",
            v.map_or(J::Null, |v| J::s(v.f_scale_row_id.clone())),
        ),
        ("rmse", J::opt(v.map(|v| v.rmse))),
        ("nrmse_percent", J::opt(v.map(|v| v.nrmse_percent))),
        ("display", r.display.clone().map_or(J::Null, J::s)),
        (
            "label_latex",
            r.display
                .as_ref()
                .map_or(J::Null, |d| J::s(nrmse::label(r.method, d).0)),
        ),
        (
            "label_plain",
            r.display
                .as_ref()
                .map_or(J::Null, |d| J::s(nrmse::label(r.method, d).1)),
        ),
        (
            "reading_uncertainty_pp",
            J::opt(v.and_then(|v| v.reading_uncertainty_pp)),
        ),
        (
            "reading_uncertainty_status",
            J::s(if v.is_some_and(|v| v.reading_uncertainty_pp.is_some()) {
                "computed"
            } else {
                "not available"
            }),
        ),
        ("rows_used", J::Arr(r.rows_used.iter().map(J::s).collect())),
        (
            "rows_excluded",
            J::Arr(r.rows_excluded.iter().map(J::s).collect()),
        ),
        (
            "unresolved_rows",
            J::Arr(r.unresolved.iter().map(J::s).collect()),
        ),
        (
            "empty_reason",
            r.empty_reason.map_or(J::Null, |e| J::s(e.code())),
        ),
        ("method_applicable", J::Bool(r.method_applicable)),
    ])
}

/// All species of the cells (for callers that iterate).
pub fn species_of(nuc: &Nuc) -> Species {
    nuc.species
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> SetMeta {
        SetMeta {
            set_id: "nrmse.test".into(),
            nuclide: "Ag-110m".into(),
            temperature_c: 1600.0,
            model_config_id: "test".into(),
        }
    }

    fn compat(id: &str, f_exp: f64, f_a: f64, resolved: bool, f_b: Option<f64>) -> Candidate {
        Candidate::Compatible {
            id: id.into(),
            test_id: "4-3-3".into(),
            f_exp,
            f_a,
            resolved_a: resolved,
            f_b,
            u_value: Some(0.005 * f_exp),
            u_time_h: Some(0.2),
            slope_a_per_h: 0.0,
            slope_b_per_h: 0.0,
        }
    }

    /// The fixture of worked example 1: illustrative model values injected through the test
    /// interface, never through production output.
    #[test]
    fn a_set_from_the_fixture_reproduces_worked_example_1() {
        let exp = [
            0.335, 0.338, 0.338, 0.338, 0.339, 0.339, 0.339, 0.340, 0.340, 0.340,
        ];
        let cands: Vec<Candidate> = exp
            .iter()
            .enumerate()
            .map(|(i, e)| compat(&format!("E-{}", 425 + i), *e, 0.330, true, Some(0.332)))
            .collect();
        let [a, b] = records(&meta(), &cands, &Ok(()));
        assert_eq!(a.n, 10);
        assert!((a.value.as_ref().unwrap().nrmse_percent - 2.5641).abs() < 5e-4);
        assert_eq!(a.display.as_deref(), Some("2.6"));
        // Method B uses the same N points and the same F_scale.
        let vb = b.value.as_ref().unwrap();
        assert_eq!(vb.n, 10);
        assert_eq!(vb.f_scale, a.value.as_ref().unwrap().f_scale);
        assert_eq!(vb.f_scale_row_id, "E-434");
        // The JSON carries the label strings and no status word.
        let j = record_json(&a).render();
        assert!(
            j.contains("\\\\mathrm{NRMSE}_{\\\\mathrm{A}} = 2.6\\\\,\\\\%"),
            "{j}"
        );
        assert!(j.contains("\"empty_reason\": null"));
    }

    #[test]
    fn the_empty_state_names_every_excluded_row_and_gives_no_value() {
        let cands = vec![
            Candidate::Excluded {
                id: "E-015".into(),
                reason: "upper_bound",
            },
            Candidate::Excluded {
                id: "E-010".into(),
                reason: "denominator_not_established",
            },
            Candidate::Excluded {
                id: "E-099".into(),
                reason: "non_detect",
            },
        ];
        let [a, b] = records(&meta(), &cands, &Ok(()));
        for r in [&a, &b] {
            assert_eq!(r.n, 0);
            assert!(r.value.is_none() && r.display.is_none());
            assert_eq!(r.empty_reason, Some(EmptyReason::NoCompatiblePoints));
            assert_eq!(r.rows_excluded.len(), 3);
            assert!(r.rows_excluded.contains(&"E-015:upper_bound".to_string()));
        }
        let j = record_json(&a).render();
        assert!(j.contains("\"nrmse_percent\": null"));
        assert!(j.contains("\"display\": null"));
        assert!(j.contains("\"label_latex\": null"));
        assert!(j.contains("\"reading_uncertainty_status\": \"not available\""));
    }

    #[test]
    fn an_unresolved_point_empties_the_record_instead_of_dropping_the_point() {
        let cands = vec![
            compat("E-1", 1.0e-6, 1.0e-9, false, Some(2.0e-9)),
            compat("E-2", 1.1e-5, 8.0e-2, true, Some(8.0e-2)),
        ];
        let [a, b] = records(&meta(), &cands, &Ok(()));
        assert_eq!(a.empty_reason, Some(EmptyReason::ModelNotResolved));
        assert_eq!(a.unresolved, vec!["E-1".to_string()]);
        assert!(a.value.is_none());
        // Method B has both points here.
        assert!(b.value.is_some());
    }

    #[test]
    fn method_b_is_empty_where_it_does_not_apply_and_never_on_a_subset() {
        let cands = vec![
            compat("E-1", 0.1, 0.12, true, Some(0.11)),
            compat("E-2", 0.2, 0.22, true, None),
        ];
        let not = NotAvailable {
            code: "TIME_VARYING_TEMPERATURE",
            reason: "x".into(),
        };
        let [a, b] = records(&meta(), &cands, &Err(not));
        assert!(a.value.is_some());
        assert_eq!(b.empty_reason, Some(EmptyReason::MethodBNotApplicable));
        assert!(!b.method_applicable);
        // Applicable, but one point is masked for Method B: no subset is used.
        let [_, b2] = records(&meta(), &cands, &Ok(()));
        assert_eq!(b2.empty_reason, Some(EmptyReason::ModelNotResolved));
    }

    #[test]
    fn a_nonpositive_registered_value_is_excluded_with_its_reason() {
        let cands = vec![
            compat("E-1", 0.0, 0.1, true, Some(0.1)),
            compat("E-2", 0.2, 0.22, true, Some(0.21)),
        ];
        let [a, _] = records(&meta(), &cands, &Ok(()));
        assert_eq!(a.n, 1);
        assert!(a
            .rows_excluded
            .contains(&"E-1:nonpositive_value".to_string()));
    }

    #[test]
    fn the_design_bound_lists_cover_the_registered_ids() {
        assert!(is_design_bound("sr90", "E-171"));
        assert!(is_design_bound("sr90", "E-475"));
        assert!(is_design_bound("ag110m", "E-432"));
        assert!(is_design_bound("ag110m", "E-186"));
        assert!(!is_design_bound("ag110m", "E-033"));
        assert!(!is_design_bound("cs134", "E-171"));
    }

    #[test]
    fn the_matrix_corrected_rows_are_window_bounds_and_never_points() {
        // Decision 67: E-010..E-014 (Cs-134) and E-033..E-037 (Ag-110m).
        for id in ["E-010", "E-014"] {
            assert!(is_matrix_corrected("cs134", id));
            assert!(!is_matrix_corrected("ag110m", id));
        }
        for id in ["E-033", "E-037"] {
            assert!(is_matrix_corrected("ag110m", id));
            assert!(!is_matrix_corrected("cs134", id));
        }
        for id in ["E-009", "E-015", "E-032", "E-038", "E-187"] {
            assert!(!is_matrix_corrected("cs134", id) && !is_matrix_corrected("ag110m", id));
        }
        assert!(BP1_SOURCE_NOTE.contains("not an established fact"));
    }

    // Fixtures for `candidates()`: no register row is a compatible point today (N = 0), so a
    // synthetic record is built from a real row of a cell nuclide with the fields that decide
    // compatibility set explicitly. The values are illustrative and never reach production output.
    fn fixture_record(reg: &Register, t_end: f64, t_start: f64) -> (RowRecord, &'static Nuc, f64) {
        let base = reg
            .release
            .rows
            .iter()
            .find(|r| {
                row_nuclide(r).is_some_and(|n| {
                    !is_matrix_corrected(n.id, &r.id) && !is_design_bound(n.id, &r.id)
                }) && row_temperature(r).is_some()
                    && !matches!(r.value_class, strato_data::release::ValueClass::NonDetect)
                    && !matches!(r.value, ReleaseValue::UpperBound(_))
            })
            .expect("a register row of a cell nuclide at one temperature exists");
        let nuc = row_nuclide(base).expect("some");
        let t_c = row_temperature(base).expect("some");
        let mut row = base.clone();
        row.time_basis = "hours at temperature".into();
        let value = 0.01;
        let spec = strato_compare::status::RunSpec {
            species: nuc.species,
            element_label_only: false,
            t_c,
            window_start_h: t_start,
            t_end_h: t_end,
            time_source: "fixture".into(),
        };
        let rec = RowRecord {
            row,
            verdict: strato_compare::status::Verdict {
                status: Status::ComparisonAvailable,
                category: "fixture",
                reason: "fixture".into(),
                plan: Plan::Point { spec, value },
            },
            status: Status::ComparisonAvailable,
            model: None,
            eval: Eval::Point {
                value,
                dev: strato_compare::metrics::PointDeviation {
                    signed: 0.0,
                    absolute: 0.0,
                    relative_percent: None,
                    log10_ratio: None,
                    normalised_residual: None,
                },
            },
        };
        (rec, nuc, t_c)
    }

    fn fixture_register() -> Register {
        Register::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data"))
            .expect("the registers load")
    }

    fn compat_fields(c: &Candidate) -> (f64, bool, Option<f64>, f64, f64) {
        match c {
            Candidate::Compatible {
                f_a,
                resolved_a,
                f_b,
                slope_a_per_h,
                slope_b_per_h,
                ..
            } => (*f_a, *resolved_a, *f_b, *slope_a_per_h, *slope_b_per_h),
            Candidate::Excluded { id, reason } => panic!("{id} excluded: {reason}"),
        }
    }

    #[test]
    fn the_isothermal_path_takes_f_a_f_b_slope_and_flag_from_the_same_curve() {
        let reg = fixture_register();
        let radii = strato_compare::data::radii(&reg).expect("radii");
        let settings = strato_compare::model::Settings {
            intervals: [20, 15, 10, 20, 10],
            dt_h: 5.0,
        };
        let mut runner = Runner::new(&reg, radii.clone(), settings).expect("runner");
        let (t_end, t_start) = (20.0, 5.0);
        let (rec, nuc, t_c) = fixture_record(&reg, t_end, t_start);
        let got = candidates(
            &[rec],
            &mut runner,
            nuc,
            t_c,
            ModelSource::Isothermal { radii: &radii },
        )
        .expect("candidates");
        assert_eq!(got.len(), 1);
        let (f_a, resolved, f_b, slope_a, slope_b) = compat_fields(&got[0]);
        let curve = runner.get(nuc.species, t_c, t_end).expect("curve").clone();
        let dt = 0.05;
        let want_a = curve.at(t_end).expect("end") - curve.at(t_start).expect("start");
        let want_r = curve.at_refined(t_end).expect("end") - curve.at_refined(t_start).expect("s");
        assert_eq!(f_a, want_a);
        assert_eq!(
            resolved,
            want_r > 0.0 && (want_a - want_r).abs() <= 0.05 * want_r
        );
        let want_slope =
            (curve.at(t_end + dt).expect("p") - curve.at(t_end - dt).expect("m")) / (2.0 * dt);
        assert_eq!(slope_a, want_slope);
        // Method B on the same times, on the same coefficients.
        let needs = Needs {
            generation: false,
            decay: false,
            temperature_varies: false,
            kernel_step_start: true,
        };
        let ts = [
            t_start * 3600.0,
            (t_end - dt) * 3600.0,
            t_end * 3600.0,
            (t_end + dt) * 3600.0,
        ];
        match method_b(needs, &radii.outer_m, &curve.d_m2_s, &ts) {
            MethodB::Available(a) => {
                let v = |k: usize| a.values[k];
                if let (Some(e), Some(s)) = (v(2), v(0)) {
                    assert_eq!(f_b, Some(e - s));
                    if let (Some(p), Some(m)) = (v(3), v(1)) {
                        assert_eq!(slope_b, (p - m) / (2.0 * dt));
                    }
                } else {
                    assert_eq!(f_b, None);
                }
            }
            MethodB::NotAvailable(_) => assert_eq!(f_b, None),
        }
    }

    #[test]
    fn the_ramp_path_reads_hold_times_and_needs_both_window_ends_resolved() {
        let reg = fixture_register();
        let radii = strato_compare::data::radii(&reg).expect("radii");
        let settings = strato_compare::model::Settings {
            intervals: [20, 15, 10, 20, 10],
            dt_h: 5.0,
        };
        let mut runner = Runner::new(&reg, radii, settings).expect("runner");
        let ec = EngineCurve {
            key: "fixture".into(),
            t_h: vec![0.0, 10.0, 20.0, 30.0],
            f_a: vec![0.0, 0.1, 0.3, 0.6],
            resolved: vec![true, true, false, true],
            max_residual_over_bound: 0.0,
            max_relative_residual: 0.0,
            steps: 0,
            assemblies: 0,
            csv_rows: vec![],
        };
        let run = |runner: &mut Runner<'_>, t_end: f64, t_start: f64| {
            let (rec, nuc, t_c) = fixture_record(&reg, t_end, t_start);
            let got = candidates(&[rec], runner, nuc, t_c, ModelSource::Ramp(&ec)).expect("ok");
            assert_eq!(got.len(), 1);
            compat_fields(&got[0])
        };
        // Cumulative at a resolved sample: the piecewise-linear value and the central slope
        // (0.1 over 0..10, 0.2 over 10..20: (0.101 - 0.0995) / 0.1 = 0.015 per hour).
        let (f, r, f_b, slope, slope_b) = run(&mut runner, 10.0, 0.0);
        assert!((f - 0.1).abs() < 1e-15 && r && f_b.is_none() && slope_b == 0.0);
        assert!((slope - 0.015).abs() < 1e-12, "slope {slope}");
        // An unresolved end sample makes the point unresolved.
        let (f, r, _, _, _) = run(&mut runner, 20.0, 0.0);
        assert!((f - 0.3).abs() < 1e-15 && !r);
        // A window: difference of the two ends, resolved only when both ends are; the slope
        // beyond the last sample is zero.
        let (f, r, _, slope, _) = run(&mut runner, 30.0, 10.0);
        assert!((f - 0.5).abs() < 1e-15 && r && slope == 0.0);
        let (_, r, _, _, _) = run(&mut runner, 30.0, 20.0);
        assert!(!r, "the window start sample is unresolved");
    }
}
