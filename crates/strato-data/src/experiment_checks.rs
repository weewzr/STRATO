//! Checks of the experimental time series and their curve assessments (family EX).
//!
//! These checks read `release_measurements.csv`, `curve_assessment.csv`, the new temperature
//! history and, when they exist, the register document and the audit log. They check what a
//! release-versus-time curve needs before a whole-curve comparison may use it: that its rows exist
//! and agree with its record, that one curve has one release definition, that its times increase,
//! that its values do not fall by more than the reading uncertainty, that every digitised row states
//! its reading uncertainty and has an audit entry, that its source resolves, and that a curve is
//! never called suitable without an independent verification.

use crate::checks::Report;
use crate::curve::{Curve, R2Class};
use crate::register::{Register, RegisterTexts};
use crate::release::{
    Measured, ReadingUncertainty, ReleaseRow, ReleaseValue, TestTemperature, ValueClass,
    ValueReading,
};

/// The first row id added by the experiment-data integration (E-001..E-540 are the earlier
/// register).
const FIRST_NEW_ROW: u32 = 541;

fn row_number(id: &str) -> u32 {
    id.strip_prefix("E-")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

fn nuclide_text(r: &ReleaseRow) -> String {
    match &r.measured {
        Measured::Nuclide(n) => n.name(),
        Measured::Element(e) => e.clone(),
        Measured::NotApplicable => "n/a".to_string(),
    }
}

fn temperature_text(r: &ReleaseRow) -> String {
    match &r.temperature {
        TestTemperature::Celsius(p) => p.text.clone(),
        TestTemperature::Range { lo, hi } => format!("{}-{}", lo.text, hi.text),
        TestTemperature::Varied { lo, hi } => format!("varied ({}-{})", lo.text, hi.text),
        TestTemperature::Absent => String::new(),
    }
}

fn single_value(r: &ReleaseRow) -> Option<f64> {
    match &r.value {
        ReleaseValue::Point(p) | ReleaseValue::Digitised(p) => Some(p.value),
        _ => None,
    }
}

/// The stated reading uncertainty of the value, in the unit of the row (0 when none is stated).
fn value_uncertainty(r: &ReleaseRow) -> f64 {
    let v = single_value(r).unwrap_or(0.0);
    match &r.reading_uncertainty {
        ReadingUncertainty::Stated {
            value: Some(reading),
            ..
        } => match reading {
            ValueReading::RelativePercent(p) => p.value / 100.0 * v.abs(),
            ValueReading::RelativePercentRange { hi, .. } => hi.value / 100.0 * v.abs(),
            ValueReading::Absolute(p) => p.value,
            ValueReading::Factor(f) => v.abs() * (f.value - 1.0).abs(),
        },
        _ => 0.0,
    }
}

fn curve_rows<'a>(reg: &'a Register, c: &Curve) -> (Vec<&'a ReleaseRow>, Vec<String>) {
    let mut rows = Vec::new();
    let mut missing = Vec::new();
    for id in &c.row_ids {
        match reg.release.get(id) {
            Some(r) => rows.push(r),
            None => missing.push(id.clone()),
        }
    }
    (rows, missing)
}

/// Run the family EX.
pub fn experiment_checks(rep: &mut Report, reg: &Register, texts: &RegisterTexts) {
    let curves = &reg.curves.curves;

    // 1. every row of a curve exists and agrees with the record of the curve
    let mut bad = Vec::new();
    for c in curves {
        let (rows, missing) = curve_rows(reg, c);
        if !missing.is_empty() {
            bad.push(format!("{}: missing rows {missing:?}", c.dataset_id));
        }
        for r in rows {
            if nuclide_text(r) != c.nuclide {
                bad.push(format!("{}: {} is {}", c.dataset_id, r.id, nuclide_text(r)));
            }
            if temperature_text(r) != c.t_c {
                bad.push(format!(
                    "{}: {} is at {}",
                    c.dataset_id,
                    r.id,
                    temperature_text(r)
                ));
            }
            if r.source != c.source {
                bad.push(format!(
                    "{}: {} has source {}",
                    c.dataset_id, r.id, r.source
                ));
            }
        }
    }
    rep.add(
        "EX.curve.rows-agree",
        bad.is_empty() && !curves.is_empty(),
        "every row named by a curve assessment exists in release_measurements.csv and has the nuclide, the test temperature and the source of its curve",
        format!("{} curves; {bad:?}", curves.len()),
    );

    // 2. one curve is one release definition
    let mut bad = Vec::new();
    for c in curves {
        let (rows, _) = curve_rows(reg, c);
        let Some(first) = rows.first() else { continue };
        for r in &rows {
            let same = r.unit == first.unit
                && r.quantity == first.quantity
                && r.denominator == first.denominator
                && r.time_basis == first.time_basis
                && r.program == first.program
                && r.compact_or_sample == first.compact_or_sample
                && r.fuel_type == first.fuel_type
                && r.group == first.group
                && r.value_class == first.value_class;
            if !same {
                bad.push(format!(
                    "{}: {} differs from {}",
                    c.dataset_id, r.id, first.id
                ));
            }
        }
        let digitised = first.value_class == ValueClass::Digitised;
        let kind_ok = if digitised {
            c.value_kind.starts_with("digitised")
        } else {
            c.value_kind == "measured_tabulated"
        };
        if !kind_ok {
            bad.push(format!(
                "{}: value kind `{}` against the value class of its rows",
                c.dataset_id, c.value_kind
            ));
        }
    }
    rep.add(
        "EX.curve.one-definition",
        bad.is_empty(),
        "the rows of one curve share the unit, quantity, denominator, time basis, programme, specimen, fuel type, group and value class, and the value kind of the record (digitised or measured_tabulated) agrees with the value class of the rows",
        format!("{bad:?}"),
    );

    // 3. times are present, in hours, and strictly increasing within a curve
    let mut bad = Vec::new();
    for c in curves {
        let (rows, _) = curve_rows(reg, c);
        let mut last: Option<f64> = None;
        for r in rows {
            match &r.time_h {
                None => bad.push(format!("{}: {} has no time_h", c.dataset_id, r.id)),
                Some(t) => {
                    if !t.value.is_finite() || t.value < 0.0 {
                        bad.push(format!("{}: {} time {}", c.dataset_id, r.id, t.text));
                    }
                    if let Some(l) = last {
                        if t.value <= l {
                            bad.push(format!(
                                "{}: {} time {} h does not exceed {l} h",
                                c.dataset_id, r.id, t.text
                            ));
                        }
                    }
                    last = Some(t.value);
                }
            }
        }
    }
    rep.add(
        "EX.curve.times-monotone",
        bad.is_empty(),
        "every row of a curve has a time in hours (column time_h) and the times increase strictly in the order of the curve",
        format!("{bad:?}"),
    );

    // 4. cumulative values do not fall by more than the reading uncertainty
    let mut bad = Vec::new();
    for c in curves {
        let (rows, _) = curve_rows(reg, c);
        for w in rows.windows(2) {
            if let (Some(a), Some(b)) = (single_value(w[0]), single_value(w[1])) {
                let slack = value_uncertainty(w[0]) + value_uncertainty(w[1]);
                if b < a - slack {
                    bad.push(format!(
                        "{}: {} falls from {a:e} to {b:e}",
                        c.dataset_id, w[1].id
                    ));
                }
            } else {
                bad.push(format!(
                    "{}: {} is not a single value",
                    c.dataset_id, w[0].id
                ));
            }
        }
    }
    rep.add(
        "EX.curve.cumulative",
        bad.is_empty(),
        "the values of a cumulative release curve do not fall by more than the sum of the reading uncertainties of two neighbouring points",
        format!("{bad:?}"),
    );

    // 5. every digitised row of a curve states the reading uncertainty of its value and its time
    let mut bad = Vec::new();
    let mut n_digitised = 0;
    for c in curves {
        let (rows, _) = curve_rows(reg, c);
        for r in rows
            .iter()
            .filter(|r| r.value_class == ValueClass::Digitised)
        {
            n_digitised += 1;
            let ok = matches!(
                &r.reading_uncertainty,
                ReadingUncertainty::Stated {
                    value: Some(_),
                    time_h: Some(_)
                }
            );
            if !ok {
                bad.push(r.id.clone());
            }
        }
    }
    rep.add(
        "EX.curve.digitised-uncertainty",
        bad.is_empty() && n_digitised > 0,
        "every digitised row of a curve states a reading uncertainty of its value and of its time",
        format!("{n_digitised} digitised rows; without: {bad:?}"),
    );

    // 6. every digitised row added by the experiment-data integration has an audit entry
    let new_digitised: Vec<&ReleaseRow> = reg
        .release
        .rows
        .iter()
        .filter(|r| row_number(&r.id) >= FIRST_NEW_ROW && r.value_class == ValueClass::Digitised)
        .collect();
    match &texts.audit_log {
        None => rep.add(
            "EX.curve.audit-entry",
            true,
            "every digitised row from E-541 on has an entry in the audit log",
            "skipped: the audit log is absent from this data folder",
        ),
        Some(log) => {
            let missing: Vec<&str> = new_digitised
                .iter()
                .filter(|r| !log.contains(&format!("| {} |", r.id)))
                .map(|r| r.id.as_str())
                .collect();
            let th_missing: Vec<&str> = reg
                .temperature
                .rows
                .iter()
                .filter(|r| r.history_id == NEW_HISTORY)
                .filter(|r| !log.contains(&r.id))
                .map(|r| r.id.as_str())
                .collect();
            rep.add(
                "EX.curve.audit-entry",
                missing.is_empty() && th_missing.is_empty() && !new_digitised.is_empty(),
                "every digitised row from E-541 on, and every row of the history AGR1-FACS-4-3-3, has an entry in the audit log (one table line `| E-xxx |` per row)",
                format!(
                    "{} rows; without: {missing:?} {th_missing:?}",
                    new_digitised.len()
                ),
            );
        }
    }

    // 7. the source of every curve resolves
    let mut bad = Vec::new();
    for c in curves {
        let (rows, _) = curve_rows(reg, c);
        let all_new = rows.iter().all(|r| row_number(&r.id) >= FIRST_NEW_ROW);
        if all_new {
            // a new curve: its key is a key of the register's source table and its link is the link of its rows
            if rows.iter().any(|r| r.link != c.link) {
                bad.push(format!("{}: the link differs from the rows", c.dataset_id));
            }
            if let Some(md) = &texts.register_markdown {
                let key = format!("| {} | ", c.source);
                if !md.lines().any(|l| l.starts_with(&key)) {
                    bad.push(format!(
                        "{}: `{}` is not a key of the source table of the register",
                        c.dataset_id, c.source
                    ));
                }
            }
        }
        if c.link.is_empty() || c.source_location.is_empty() {
            bad.push(format!("{}: no link or location", c.dataset_id));
        }
    }
    rep.add(
        "EX.curve.source-resolves",
        bad.is_empty(),
        "every curve names a source, a location and a link; a curve of the new rows uses the link of its rows and a key that the source table of the register lists",
        format!("{bad:?}"),
    );

    // 8. a curve is never called suitable without an independent verification
    let mut bad = Vec::new();
    for c in curves {
        if c.r2_class == R2Class::Suitable
            && (c.verification_ref.is_empty()
                || !c.verification_status.starts_with("independently verified"))
        {
            bad.push(c.dataset_id.clone());
        }
        if c.r2_class == R2Class::PlotOnly && c.reason_code.is_empty() {
            bad.push(c.dataset_id.clone());
        }
    }
    let n_suitable = curves
        .iter()
        .filter(|c| c.r2_class == R2Class::Suitable)
        .count();
    rep.add(
        "EX.curve.fail-closed",
        bad.is_empty(),
        "a curve is R2 SUITABLE only with a verification reference and the status independently verified; a PLOT ONLY curve has its reason code",
        format!("{} curves, {n_suitable} suitable; {bad:?}", curves.len()),
    );

    // 9. every row from E-541 on belongs to one curve and names its dataset in the notes
    let mut bad = Vec::new();
    for r in reg
        .release
        .rows
        .iter()
        .filter(|r| row_number(&r.id) >= FIRST_NEW_ROW)
    {
        match reg.curves.curve_of_row(&r.id) {
            None => bad.push(format!("{} belongs to no curve", r.id)),
            Some(c) => {
                let head = format!("Dataset {} (curve_assessment.csv).", c.dataset_id);
                if !r.notes.starts_with(&head) {
                    bad.push(format!("{} notes do not start with `{head}`", r.id));
                }
            }
        }
    }
    rep.add(
        "EX.release.dataset-id",
        bad.is_empty(),
        "every release row from E-541 on belongs to exactly one curve assessment and its notes start with its dataset id",
        format!("{bad:?}"),
    );

    // 10. the new temperature history is a chain of steps
    let rows: Vec<_> = reg
        .temperature
        .rows
        .iter()
        .filter(|r| r.history_id == NEW_HISTORY)
        .collect();
    let mut bad = Vec::new();
    let mut cum = 0.0;
    for (i, r) in rows.iter().enumerate() {
        if r.step as usize != i + 1 {
            bad.push(format!("{} step {}", r.id, r.step));
        }
        let ramp = r.ramp_h.as_ref().map_or(0.0, |p| p.value);
        let hold = r.hold_h.as_ref().map_or(0.0, |p| p.value);
        cum += ramp + hold;
        match &r.cumulative_end_h_calc {
            Some(p) if (p.value - cum).abs() < 0.15 => {}
            other => bad.push(format!("{} cumulative {other:?} against {cum}", r.id)),
        }
        cum = r.cumulative_end_h_calc.as_ref().map_or(cum, |p| p.value);
        if i + 1 < rows.len() {
            let end = r.t_end.as_ref().and_then(|t| t.celsius());
            let next = rows[i + 1].t_start.as_ref().and_then(|t| t.celsius());
            if end != next {
                bad.push(format!(
                    "{} ends at {end:?}, the next step starts at {next:?}",
                    r.id
                ));
            }
        }
    }
    rep.add(
        "EX.temperature.new-history",
        bad.is_empty() && rows.len() == 7,
        "the history AGR1-FACS-4-3-3 has 7 steps numbered from 1, each step ends at the temperature at which the next starts, and the cumulative end time of each step is the previous one plus its ramp and hold within 0.15 h",
        format!("{} rows; {bad:?}", rows.len()),
    );
}

/// The temperature history added by the experiment-data integration.
const NEW_HISTORY: &str = "AGR1-FACS-4-3-3";
