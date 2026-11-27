//! The experimental side: the bound comparisons of the registered compact totals, window upper
//! bounds and reported bounds, with their class, premises, reading uncertainty and statuses.
//!
//! The whole-curve agreement metric R2 is not here: it is in `r2`, `r2curve` and `r2run`.

use std::collections::BTreeMap;

use strato_compare::compare::{Eval, RowRecord};
use strato_compare::data::{Radii, Species};
use strato_compare::json::J;
use strato_compare::model::Runner;
use strato_compare::status::{end_time, Plan};
use strato_data::release::{
    Measured, ReadingUncertainty, ReleaseRow, ReleaseValue, TestTemperature, ValueReading,
};
use strato_data::table::Vocabulary;
use strato_data::Register;

use crate::curves::{Nuc, NUCS};
use crate::methodb::{method_b, MethodB, Needs};
use strato::analytical::extensions::Quantity;

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

/// The note that goes with BP1 for the matrix-corrected rows (decision 67). The window numbers
/// are those of the comparison program, which applies decision 68: the matrix-corrected rows of
/// intact compacts are window upper bounds under premise BP1 and never point comparisons, because
/// the source calls the correction an attempt that cannot separate particle release from matrix
/// release. Which rows they are is the comparison program's decision (`RowRecord::is_window_bound`),
/// not a list kept here.
pub fn bp1_source_note() -> String {
    format!(
        "For the matrix-corrected rows BP1 is the source's own assumption, not an established fact: the authors call the correction an attempt that cannot separate particle release from matrix release. The window runs from about {} h at temperature to the end of the {} h plateau, or to the end of the test if shorter.",
        strato_compare::status::WINDOW_START_H,
        strato_compare::status::PLATEAU_H
    )
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
    /// Denominator text, as the comparison program states it.
    pub denominator: String,
    /// The class of the comparison, as the comparison program states it.
    pub comparison_class: String,
    /// The register's time label of the row (`time_basis`).
    pub time_window: String,
    /// The register's statement of what the value means for one particle.
    pub particle_applicability: String,
    /// For a window bound: the premise as the comparison program states it.
    pub premise_source: Option<&'static str>,
    /// For a window bound: where the end of the window comes from.
    pub window_end_origin: Option<String>,
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
        rec: &'a RowRecord,
        nuc: &'static Nuc,
        t_c: f64,
        t_h: f64,
        t_start_h: f64,
        kind: &'static str,
        window_end_origin: Option<String>,
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
        if rec.is_window_bound() {
            // Decisions 67 and 68(a): a window upper bound under BP1, never a point. The comparison
            // program decides which rows these are and gives the window (about 24 h at temperature
            // to the end of the 300 h plateau or the shorter test end) in its plan.
            if let Plan::Inequality { spec, .. } = &rec.verdict.plan {
                cands.push(Cand {
                    row,
                    rec,
                    nuc,
                    t_c,
                    t_h: spec.t_end_h,
                    t_start_h: spec.window_start_h,
                    kind: "window_bound",
                    window_end_origin: Some(spec.time_source.clone()),
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
            rec,
            nuc,
            t_c,
            t_h,
            t_start_h: t_start,
            kind,
            window_end_origin: None,
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
    let mut b_series: BTreeMap<(&'static str, i64), BSeries> = BTreeMap::new();
    for ((nid, t), times) in &b_times {
        let nuc = nuc_by_id(nid).ok_or("nuclide")?;
        let curve = runner.get(nuc.species, *t as f64, 1.0)?;
        let d = curve.d_m2_s;
        let mut ts = times.clone();
        ts.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        ts.dedup();
        let needs = Needs::baseline(*t as f64);
        if let MethodB::Available(a) = method_b(
            &needs,
            Quantity::ReleaseFraction,
            1.0,
            &radii.outer_m,
            &d,
            &ts,
        ) {
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
            denominator: c.rec.comparison_denominator(),
            comparison_class: c.rec.comparison_class().to_string(),
            time_window: row.time_basis.clone(),
            particle_applicability: row.particle_applicability.clone(),
            premise_source: match &c.rec.verdict.plan {
                Plan::Inequality { source, .. } => Some(source.text()),
                _ => None,
            },
            window_end_origin: c.window_end_origin.clone(),
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
            comparison_class: "window upper bound from two time-series points".to_string(),
            time_window: w.b.time_basis.clone(),
            particle_applicability: w.b.particle_applicability.clone(),
            premise_source: None,
            window_end_origin: None,
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
                        J::s(format!("the source gives no exact start of the window (\"about {0} h\"); the value used above is {0} h", strato_compare::status::WINDOW_START_H)),
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
        ("comparison_class", J::s(b.comparison_class.clone())),
        ("time_window", J::s(b.time_window.clone())),
        (
            "particle_applicability",
            J::s(b.particle_applicability.clone()),
        ),
        ("premise_source", b.premise_source.map_or(J::Null, J::s)),
        (
            "window_end_origin",
            b.window_end_origin.clone().map_or(J::Null, J::s),
        ),
        ("reading_uncertainty", J::s(b.reading.clone())),
        ("systematic_uncertainty", J::s(SYSTEMATIC_AGR1)),
        (
            "premises",
            J::Arr(if b.kind == "window_bound" && b.ends.is_none() {
                vec![J::s(BP1), J::s(bp1_source_note()), J::s(BP2), J::s(A1)]
            } else {
                vec![J::s(BP1), J::s(BP2), J::s(A1)]
            }),
        ),
    ])
}

/// All species of the cells (for callers that iterate).
pub fn species_of(nuc: &Nuc) -> Species {
    nuc.species
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Decisions 67 and 68: E-010..E-014 (Cs-134, 1600 degC) and E-033..E-037 (Ag-110m, 1600
        // and 1700 degC) are window upper bounds under premise BP1. Which rows they are comes from
        // the comparison program; class, denominator, premise and window are read from it and
        // from the register row, not typed here.
        let reg = fixture_register();
        let radii = strato_compare::data::radii(&reg).expect("radii");
        let settings = strato_compare::model::Settings {
            intervals: [20, 15, 10, 20, 10],
            dt_h: 5.0,
        };
        let mut runner = Runner::new(&reg, radii.clone(), settings).expect("runner");
        let records = strato_compare::compare::run_rows(&reg, &mut runner).expect("rows");
        let mut ids: Vec<String> = records
            .iter()
            .filter(|r| r.is_window_bound())
            .map(|r| r.row.id.clone())
            .collect();
        ids.sort();
        let want: Vec<String> = (10..=14)
            .chain(33..=37)
            .map(|n| format!("E-{n:03}"))
            .collect();
        assert_eq!(ids, want);
        let recs = bounds(&reg, &records, &mut runner, &radii).expect("bounds");
        let windows: Vec<&BoundRec> = recs
            .iter()
            .filter(|b| b.kind == "window_bound" && b.ends.is_none())
            .collect();
        assert_eq!(windows.len(), 10);
        for b in windows {
            let row = reg
                .release
                .rows
                .iter()
                .find(|r| r.id == b.id)
                .expect("the register row");
            assert_eq!(
                b.comparison_class,
                strato_compare::status::WINDOW_BOUND_CLASS
            );
            assert_eq!(
                b.denominator,
                strato_compare::status::WINDOW_BOUND_DENOMINATOR
            );
            assert_eq!(b.time_window, row.time_basis);
            assert!(b.time_window.contains("plateau"), "{}", b.time_window);
            assert_eq!(b.particle_applicability, row.particle_applicability);
            assert!(b.particle_applicability.contains("BP1"));
            assert_eq!(
                b.premise_source,
                Some(strato_compare::status::BoundSource::WindowUnderBp1.text())
            );
            assert_eq!(b.t_start_h, strato_compare::status::WINDOW_START_H);
            assert!(b.t_h <= strato_compare::status::PLATEAU_H);
            assert!(b.window_end_origin.is_some());
            let j = bound_json(b).render();
            assert!(
                !j.contains("\"denominator\": \"not established\""),
                "{}",
                b.id
            );
            assert!(j.contains("\"denominator\": \"as-run calculated compact inventory\""));
        }
        assert!(bp1_source_note().contains("not an established fact"));
    }

    fn fixture_register() -> Register {
        Register::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data"))
            .expect("the registers load")
    }
}
