//! The production side of the R2 records (`metric-r2.md`, decisions 72 and 74): for every curve
//! record of `data/experimental/curve_assessment.csv` it reads the registered rows, runs Method A
//! with the model configuration the assessment gives (geometry, nuclide, the registered
//! temperature history with its ramps, the diffusivity rows), runs Method B only where the
//! applicability function allows, hands everything to `r2curve` and writes one comparison file per
//! curve.
//!
//! Nothing here relaxes a check. A curve whose history, nuclide or geometry cannot be configured
//! gets no model curve and the record says which check failed.

use std::path::Path;

use strato::analytical::applicability::{SourceHistory, Stage};
use strato::analytical::extensions::Quantity;
use strato::numerical::physics_run::{Interpolation, TimeFunction};
use strato::numerical::scenario::{
    run_scenario, Diffusion, InitialState, ScenarioConfig, ScenarioDefinition, ScenarioRun,
    StepSegment,
};
use strato_compare::data::Radii;
use strato_compare::json::J;
use strato_data::curve::{BoundClass, Curve, SpecimenLevel};
use strato_data::geometry::GeometryValue;
use strato_data::release::{
    Denominator, ReadingUncertainty, ReleaseRow, ReleaseUnit, ReleaseValue, ValueClass,
    ValueReading,
};
use strato_data::select::{Phase, Selection, Variant};
use strato_data::table::Vocabulary;
use strato_data::temperature::TemperatureRow;
use strato_data::Register;

use crate::curves::lin;
use crate::experiment::{A1, BP1, BP2};
use crate::methodb::{method_b, temperature_of, MethodB, Needs};
use crate::output::{json_file, Files, Prov};
use crate::params::{decay_constant, Laws, LAYERS};
use crate::r2::{self, ReasonCode};
use crate::r2curve::{
    assess, caption_json, classes_of, CaptionWords, CurveInput, DecayReference, InventoryReference,
    Method, MethodInput, PointClass, R2Record, RowInput, SourceInfo,
};

const H: f64 = 3600.0;
/// The documented application of the heating-test set C16 (science design Section 2.1: applied by
/// Collin (2016) over histories spanning 400 to 1800 degC; no range is printed).
const C16_APPLICATION_C: (f64, f64) = (400.0, 1800.0);

/// What the run of all curves returns.
pub struct R2Run {
    /// The records, two per curve (A then B), in the order of the assessment file.
    pub records: Vec<R2Record>,
    /// The caption records, one per curve.
    pub captions: Vec<J>,
    /// The comparison files, `comparison/<curve_id>.json`.
    pub files: Files,
    /// `(curve_id, first failing check of record A)`, for the table of Section 3.5.
    pub first_failing: Vec<(String, Option<&'static str>)>,
}

// ---------------------------------------------------------------------------------------------
// Registered text helpers
// ---------------------------------------------------------------------------------------------

struct SourceRow {
    key: String,
    apa: String,
    link: String,
}

fn source_table(markdown: &str) -> Vec<SourceRow> {
    let mut rows = Vec::new();
    let mut inside = false;
    for line in markdown.lines() {
        if line.starts_with("## ") {
            inside = line.starts_with("## 3. Sources");
            continue;
        }
        if !inside || !line.starts_with('|') || line.starts_with("|---") {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() >= 4 && cells[0] != "Key" {
            rows.push(SourceRow {
                key: cells[0].to_string(),
                apa: cells[1].to_string(),
                link: cells[3].to_string(),
            });
        }
    }
    rows
}

fn source_info(table: &[SourceRow], source: &str, link: &str, location: &str) -> SourceInfo {
    let by_key = table.iter().find(|r| r.key == source);
    let by_report = || {
        let id = source.split(';').nth(1)?.trim().trim_end_matches(')');
        table.iter().find(|r| r.apa.contains(id))
    };
    let row = by_key.or_else(by_report);
    let (apa, base_key) = match row {
        // The register writes titles in markdown italics; the record carries plain text.
        Some(r) => (r.apa.replace('*', ""), r.key.clone()),
        None => (
            format!("{source} (the register holds no APA reference for this source key)"),
            source.to_string(),
        ),
    };
    let verified = table
        .iter()
        .any(|r| r.key.starts_with(&base_key) && r.link.contains("HTTP 200"))
        || table
            .iter()
            .any(|r| r.key.starts_with(&base_key) && r.apa.contains("was opened on"));
    // A limitation sentence that the register appends to a reference belongs beside the citation,
    // not inside it.
    const LIMIT: &str = " Title and full author list were not recorded by the research task.";
    let (apa, note) = match apa.strip_suffix(LIMIT) {
        Some(a) => (a.to_string(), Some(LIMIT.trim().to_string())),
        None => (apa, None),
    };
    SourceInfo {
        apa,
        note,
        link: link.to_string(),
        link_status: if verified {
            "verified".to_string()
        } else {
            "no link check is recorded in the register".to_string()
        },
        location: location.to_string(),
    }
}

/// The number of particles in `(75 particles)` or `(75 loose particles)`.
fn particle_count(text: &str) -> Option<f64> {
    for part in text.split('(').skip(1) {
        let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
        let rest = &part[digits.len()..];
        if !digits.is_empty()
            && (rest.starts_with(" particles") || rest.starts_with(" loose particles"))
        {
            return digits.parse().ok();
        }
    }
    None
}

fn value_of(row: &ReleaseRow) -> Option<f64> {
    match &row.value {
        ReleaseValue::Point(p)
        | ReleaseValue::Digitised(p)
        | ReleaseValue::UpperBound(p)
        | ReleaseValue::LowerBound(p) => Some(p.value),
        _ => None,
    }
}

fn axis_scale(first_notes: &str) -> &'static str {
    if first_notes.contains("linear or logarithmic") {
        "linear"
    } else if first_notes.contains("logarithmic") {
        "log"
    } else {
        "linear"
    }
}

fn level_word(l: Option<SpecimenLevel>) -> &'static str {
    match l {
        Some(SpecimenLevel::SingleParticle) => "single particle",
        Some(SpecimenLevel::ParticleBatch) => "particle batch",
        Some(SpecimenLevel::Compact) => "compact",
        Some(SpecimenLevel::Sphere) => "fuel sphere",
        None => "specimen",
    }
}

fn specimen_label(level: Option<SpecimenLevel>, id: &str, aggregated: Option<bool>) -> String {
    let base = if let Some((prefix, _)) = id.split_once(" loose particles") {
        format!("loose particles from compact {prefix}")
    } else if level == Some(SpecimenLevel::Sphere) {
        format!("{} {}", level_word(level), id.trim_end_matches(" sphere"))
    } else if level == Some(SpecimenLevel::ParticleBatch) {
        format!("particle batch, {id}")
    } else {
        format!("{} {id}", level_word(level))
    };
    if aggregated == Some(true) {
        format!("{base}, source mean")
    } else {
        base
    }
}

fn time_axis_label(time_basis: &str) -> String {
    let head = time_basis.split('(').next().unwrap_or(time_basis).trim();
    format!("{head} (h)")
}

fn degrees(t: f64) -> String {
    if t.fract() == 0.0 {
        format!("{} \u{b0}C", t as i64)
    } else {
        format!("{t} \u{b0}C")
    }
}

// ---------------------------------------------------------------------------------------------
// The registered history
// ---------------------------------------------------------------------------------------------

struct Hist {
    /// Elapsed time (h) and temperature (degC) of each knot.
    knots: Vec<(f64, f64)>,
    /// Elapsed time of the first knot (the run starts here), h.
    origin_h: f64,
    /// Elapsed time of the last registered knot, h.
    end_h: f64,
    /// Elapsed time at the start of the last hold, h.
    hold_start_h: f64,
    /// `(temperature, duration)` of each hold.
    holds: Vec<(f64, f64)>,
    /// True if every temperature up to the end is registered and no row is a jump.
    complete: bool,
    /// The ramp from an unregistered start that the run omits: duration (h) and end temperature.
    omitted_first_ramp: Option<(f64, f64)>,
    /// True if no row registers a duration and the run is an isothermal idealisation.
    idealised: bool,
    /// Row ids.
    row_ids: Vec<String>,
    /// True if every row is verified (digitised rows are not: no structured verification).
    all_verified: bool,
}

fn celsius(p: &Option<strato_data::temperature::TemperaturePoint>) -> Option<f64> {
    p.as_ref().and_then(|x| x.celsius())
}

fn build_history(rows: &[&TemperatureRow], isothermal_c: Option<f64>) -> Hist {
    let mut h = Hist {
        knots: Vec::new(),
        origin_h: 0.0,
        end_h: 0.0,
        hold_start_h: 0.0,
        holds: Vec::new(),
        complete: true,
        omitted_first_ramp: None,
        idealised: false,
        row_ids: rows.iter().map(|r| r.id.clone()).collect(),
        all_verified: rows
            .iter()
            .all(|r| r.status == strato_data::common::Status::Verified),
    };
    let mut t = 0.0;
    let mut cur: Option<f64> = None;
    for r in rows {
        let (ts, te) = (celsius(&r.t_start), celsius(&r.t_end));
        let ramp = r.ramp_h.as_ref().map(|p| p.value).filter(|x| *x > 0.0);
        let hold = r.hold_h.as_ref().map(|p| p.value).filter(|x| *x > 0.0);
        let mut ramp_done = false;
        match (cur, ts) {
            (None, Some(s)) => {
                h.knots.push((t, s));
                cur = Some(s);
            }
            (Some(c), Some(s)) if (c - s).abs() > 1e-9 => {
                h.complete = false;
                break;
            }
            (None, None) => match (ramp, te) {
                (Some(rp), Some(e)) => {
                    t += rp;
                    h.knots.push((t, e));
                    h.origin_h = t;
                    h.omitted_first_ramp = Some((rp, e));
                    cur = Some(e);
                    ramp_done = true;
                }
                _ => {
                    h.complete = false;
                    break;
                }
            },
            _ => {}
        }
        if let (Some(rp), false) = (ramp, ramp_done) {
            let Some(e) = te else {
                // The ramp ends at a temperature the register does not state.
                h.complete = false;
                break;
            };
            t += rp;
            h.knots.push((t, e));
            cur = Some(e);
        }
        if let Some(hd) = hold {
            let Some(c) = cur.or(te) else {
                h.complete = false;
                break;
            };
            if h.knots.is_empty() {
                h.knots.push((t, c));
            }
            h.hold_start_h = t;
            t += hd;
            h.knots.push((t, c));
            h.holds.push((c, hd));
            cur = Some(c);
        }
    }
    h.end_h = h.knots.last().map_or(0.0, |k| k.0);
    let no_duration = rows
        .iter()
        .all(|r| r.ramp_h.is_none() && r.hold_h.is_none());
    if no_duration {
        // No row registers a duration: only an isothermal idealisation at the registered test
        // temperature is possible, and it is labelled so.
        h.complete = false;
        h.idealised = isothermal_c.is_some();
        h.knots.clear();
        h.holds.clear();
        if let Some(c) = isothermal_c {
            h.knots.push((0.0, c));
        }
        h.origin_h = 0.0;
        h.hold_start_h = 0.0;
        h.end_h = 0.0;
    }
    h
}

// ---------------------------------------------------------------------------------------------
// Radii and diffusivity laws
// ---------------------------------------------------------------------------------------------

fn radii_of(reg: &Register, design_id: &str) -> Result<Radii, String> {
    if design_id == strato_compare::data::GEOMETRY_DESIGN {
        return strato_compare::data::radii(reg);
    }
    let mut diameter = None;
    let mut thickness = [None; 4];
    let mut ids = Vec::new();
    for row in reg
        .geometry
        .rows
        .iter()
        .filter(|r| r.design_id == design_id)
    {
        let GeometryValue::Point(p) = &row.value else {
            continue;
        };
        if row.unit.as_str() != "um" {
            continue;
        }
        let slot = match (row.layer.as_str(), row.quantity.as_str()) {
            ("kernel", "diameter") => {
                diameter = Some(p.value);
                ids.push(row.id.clone());
                continue;
            }
            ("buffer", "thickness") => 0,
            ("IPyC", "thickness") => 1,
            ("SiC", "thickness") => 2,
            ("OPyC", "thickness") => 3,
            _ => continue,
        };
        thickness[slot] = Some(p.value);
        ids.push(row.id.clone());
    }
    let d = diameter.ok_or_else(|| format!("{design_id}: no kernel diameter"))?;
    let mut outer = [0.0; 5];
    outer[0] = d / 2.0 * 1e-6;
    for k in 0..4 {
        let t =
            thickness[k].ok_or_else(|| format!("{design_id}: a coating thickness is missing"))?;
        outer[k + 1] = outer[k] + t * 1e-6;
    }
    Ok(Radii {
        outer_m: outer,
        row_ids: ids,
    })
}

/// The FRG reference correlations of IAEA-TECDOC-978 (Tables A-1 to A-3, origin FRG) for one
/// element, with the buffer convention of the same source (decision 72(b)).
fn frg_laws<'a>(reg: &'a Register, element: &str) -> Result<Laws<'a>, String> {
    let ids: [&[&str]; 4] = match element {
        "Cs" => [
            &["A1-01", "A1-02"],
            &["BUF-01"],
            &["A2-01"],
            &["A3-01", "A3-02"],
        ],
        "Sr" => [&["A1-03"], &["BUF-01"], &["A2-02"], &["A3-03", "A3-04"]],
        "Ag" => [&["A1-04"], &["BUF-01"], &["A2-03"], &["A3-05"]],
        other => return Err(format!("no FRG reference rows for {other}")),
    };
    let per_layer = [ids[0], ids[1], ids[2], ids[3], ids[2]];
    let mut sel = Vec::new();
    for ((_, layer), rows_ids) in LAYERS.iter().zip(per_layer) {
        let mut rows = Vec::new();
        for id in rows_ids {
            rows.push(
                reg.arrhenius
                    .get(id)
                    .ok_or_else(|| format!("the row {id} is not in the diffusivity register"))?,
            );
        }
        let first = rows[0];
        sel.push(Selection {
            element: element.to_string(),
            layer: *layer,
            phase: Phase::HeatingTest,
            variant: Variant::Default,
            class: first.coefficient_class,
            source: first.source.clone(),
            source_location: first.source_location.clone(),
            published_range_c: None,
            range_kind: None,
            kernel_scale: None,
            labels: vec![
                "FRG reference correlation of IAEA-TECDOC-978 (German fuel), not the AGR-1 heating-test set; no validity range printed".to_string(),
            ],
            rows,
        });
    }
    Ok(Laws { sel })
}

// ---------------------------------------------------------------------------------------------
// The model runs
// ---------------------------------------------------------------------------------------------

struct Runs {
    base: ScenarioRun,
    fine: ScenarioRun,
    run_end_s: f64,
}

fn run_pair(
    laws: &Laws<'_>,
    radii: &Radii,
    def: &ScenarioDefinition,
    extra_s: &[f64],
    quick: bool,
) -> Result<Runs, String> {
    let f = |_t: f64, t_c: f64| laws.at(t_c);
    let intervals: [usize; 5] = if quick {
        [20, 15, 10, 20, 10]
    } else {
        [160, 20, 20, 160, 20]
    };
    let go = |scale: f64, mult: usize| -> Result<ScenarioRun, String> {
        let mut d = def.clone();
        for s in &mut d.plan {
            s.dt_max_s *= scale;
        }
        let cfg = ScenarioConfig {
            radii_m: radii.outer_m,
            intervals: intervals.map(|n| n * mult),
            initial: InitialState::KernelUniform { c_init: 1.0 },
            decay_constant_per_s: 0.0,
            reference_generation_mol_m3_s: 0.0,
            diffusion: Diffusion::OfTemperature(&f),
            profile_times_s: vec![],
            extra_boundaries_s: extra_s.to_vec(),
        };
        run_scenario(&d, &cfg).map_err(|e| e.to_string())
    };
    Ok(Runs {
        base: go(1.0, 1)?,
        fine: go(0.5, 2)?,
        run_end_s: def.end_s(),
    })
}

fn f_at(run: &ScenarioRun, t_s: f64) -> Option<f64> {
    if t_s <= 0.0 {
        return Some(0.0);
    }
    run.row_at(t_s)?.release_fraction_of_initial
}

fn series_hash(run: &ScenarioRun) -> String {
    let mut s = String::new();
    for r in &run.rows {
        s.push_str(&format!(
            "{:e};{:e}\n",
            r.t_s,
            r.release_fraction_of_initial.unwrap_or(0.0)
        ));
    }
    strato::postprocessing::export::sha256_hex(s.as_bytes())
}

// ---------------------------------------------------------------------------------------------
// One curve
// ---------------------------------------------------------------------------------------------

struct Built {
    input: CurveInput,
    a: Option<MethodInput>,
    b: Option<MethodInput>,
    drawn: [bool; 2],
    model_json: J,
    experiment_json: J,
}

fn first_token(text: &str) -> &str {
    text.split([' ', ',', '(']).next().unwrap_or(text).trim()
}

fn design_of(geometry_ids: &str) -> &str {
    let t = first_token(geometry_ids);
    if t.starts_with("G-") {
        strato_compare::data::GEOMETRY_DESIGN
    } else {
        t
    }
}

fn reading_of(row: &ReleaseRow, value_used: f64, count: Option<f64>) -> (Option<f64>, Option<f64>) {
    match &row.reading_uncertainty {
        ReadingUncertainty::Stated { value, time_h } => {
            let u = match value {
                Some(ValueReading::RelativePercent(p)) => Some(p.value / 100.0 * value_used),
                Some(ValueReading::RelativePercentRange { hi, .. }) => {
                    Some(hi.value / 100.0 * value_used)
                }
                Some(ValueReading::Absolute(p)) => Some(p.value / count.unwrap_or(1.0)),
                _ => None,
            };
            (u, time_h.as_ref().map(|t| t.value))
        }
        _ => (None, None),
    }
}

fn class_word(c: PointClass) -> &'static str {
    match c {
        PointClass::Measured => "measured",
        PointClass::DetectionBound => "detection bound",
        PointClass::OtherUpperBound => "other upper bound",
        PointClass::WindowBound => "window bound",
        PointClass::CompactTotalBound => "compact-total bound",
        PointClass::Unclassified => "unclassified",
    }
}

#[allow(clippy::too_many_lines)]
fn build(
    prov: &Prov,
    reg: &Register,
    table: &[SourceRow],
    curve: &Curve,
    quick: bool,
) -> Result<Built, String> {
    let rows: Vec<&ReleaseRow> = curve
        .row_ids
        .iter()
        .map(|id| {
            reg.release
                .get(id)
                .ok_or_else(|| format!("{}: the row {id} is not in the register", curve.dataset_id))
        })
        .collect::<Result<_, _>>()?;
    let first = rows[0];
    let element = curve.nuclide.split('-').next().unwrap_or(&curve.nuclide);
    let t_c_text = curve.t_c.trim();
    let t_c_number: Option<f64> = t_c_text.parse().ok();
    let frg = first.program.as_str() == "FRG";
    // ---- Registered rows (time basis, particle equivalents, reading uncertainty).
    let count = first
        .denominator
        .as_str()
        .contains("particle equivalents")
        .then(|| {
            particle_count(&first.compact_or_sample)
                .or_else(|| particle_count(first.quantity.as_str()))
        })
        .flatten();
    let conversion = count.map(|n| {
        format!("registered particle equivalents divided by the registered particle count {n} (calculated from registered values, decision 65(b))")
    });
    // ---- The history and the time mapping.
    let hist_id = first_token(&curve.history_id).to_string();
    let hist_rows: Vec<&TemperatureRow> = {
        let mut v: Vec<&TemperatureRow> = reg
            .temperature
            .rows
            .iter()
            .filter(|r| r.history_id == hist_id)
            .collect();
        v.sort_by_key(|r| r.step);
        v
    };
    let hist = build_history(&hist_rows, t_c_number);
    let at_temperature = !first.time_basis.contains("elapsed");
    let basis_offset_h = if at_temperature && !hist.idealised {
        hist.hold_start_h
    } else {
        0.0
    };
    let mut inputs: Vec<RowInput> = Vec::new();
    for row in &rows {
        let registered = value_of(row).ok_or_else(|| format!("{}: no value", row.id))?;
        let used = registered / count.unwrap_or(1.0);
        let (u_value, u_time) = reading_of(row, used, count);
        let time_h = row
            .time_h
            .as_ref()
            .map(|p| p.value)
            .ok_or_else(|| format!("{}: no time", row.id))?;
        inputs.push(RowInput {
            id: row.id.clone(),
            time_h,
            value: used,
            registered_value: registered,
            qualifier: row.value.qualifier().to_string(),
            value_class: row.value_class.as_str().to_string(),
            bound_class: Some(curve.bound_class),
            status: row.status.as_str().to_string(),
            digitised: row.value_class == ValueClass::Digitised,
            // The audit log records the independent checks in prose, not in a field of the row.
            digitisation_verification_ref: None,
            u_value,
            u_time_h: u_time,
            counting_delay_h: None,
            duplicate_of: None,
            negative_presented_as_release: false,
            time_el_h: basis_offset_h + time_h,
        });
    }
    // ---- Method A.
    let design = design_of(&curve.geometry_ids).to_string();
    let radii = radii_of(reg, &design);
    let laws = if frg {
        frg_laws(reg, element)
    } else {
        Laws::new(reg, element, Phase::HeatingTest)
    };
    let step_h = if quick { 0.5 } else { 0.05 };
    let t_mod: Vec<Option<f64>> = inputs
        .iter()
        .map(|r| {
            let t = r.time_el_h - hist.origin_h;
            (t >= -1e-9).then_some(t.max(0.0))
        })
        .collect();
    let needed_h = inputs
        .iter()
        .zip(&t_mod)
        .filter_map(|(r, t)| t.map(|t| t + r.u_time_h.unwrap_or(0.0)))
        .fold(0.0_f64, f64::max);
    let hist_len_h = hist.end_h - hist.origin_h;
    let run_end_h = if hist.idealised {
        needed_h
    } else {
        needed_h.min(hist_len_h)
    };
    let reaches = !hist.knots.is_empty() && (hist.idealised || needed_h <= hist_len_h + 1e-9);
    let mut notes: Vec<String> = Vec::new();
    let mut premises: Vec<(String, String)> = vec![(
        "A2".into(),
        "kernel-only start (A2, decision 20): the run starts with the inventory uniform in the kernel".into(),
    )];
    if curve.bound_class == BoundClass::CompactTotal {
        premises.push(("BP1".into(), BP1.into()));
        premises.push(("BP2".into(), BP2.into()));
        premises.push(("A1".into(), A1.into()));
    }
    if let Some(c) = &conversion {
        premises.push(("particle_equivalents".into(), c.clone()));
    }
    if frg {
        premises.push((
            "frg_reference_correlations".into(),
            "diffusivities: the FRG reference correlations of IAEA-TECDOC-978 Tables A-1 to A-3 (German fuel; rows A1-01..A1-04, BUF-01, A2-01..A2-03, A3-01..A3-05), not the AGR-1 heating-test set C16 (decision 72(b)); no validity range printed".into(),
        ));
    } else {
        premises.push((
            "c16_heating_test_set".into(),
            "diffusivities: the published heating-test rows C16 as printed (decision 39); no validity range printed, applied by Collin (2016) over 400 to 1800 degC".into(),
        ));
    }
    if hist.idealised {
        premises.push((
            "temperature_history_idealised".into(),
            format!("temperature ramp not modelled: the register gives no durations for the history {hist_id}; the run is isothermal at the registered test temperature from t = 0"),
        ));
    }
    if let Some((rp, e)) = hist.omitted_first_ramp {
        premises.push((
            "first_ramp_not_modelled".into(),
            format!("the first ramp of the history ({rp} h to {e} degC) starts at a temperature the register does not state: the run starts at {e} degC at its end, and the release during that ramp is neglected"),
        ));
    }
    let model_config_id = format!(
        "btcs.{design}.{}.{hist_id}.kernel_uniform.decay_free.intervals{}.dt{step_h}h",
        if frg { "T978-FRG" } else { "C16" },
        if quick { "quick" } else { "160-20-20-160-20" }
    );
    // The run (or the reason there is none).
    let mut a_input = MethodInput::empty(Method::A, inputs.len());
    let mut b_input = MethodInput::empty(Method::B, inputs.len());
    b_input.applicable = false;
    b_input.mask_passed = false;
    let mut samples: Option<(Vec<f64>, Vec<f64>)> = None;
    let mut b_samples: Option<(Vec<f64>, Vec<Option<f64>>)> = None;
    let mut run_note = String::new();
    let mut steps = 0usize;
    let mut hold_temps_c: Vec<f64> = hist.holds.iter().map(|h| h.0).collect();
    hold_temps_c.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if let (Ok(radii), Ok(laws)) = (&radii, &laws) {
        let m_verified = reg
            .geometry
            .rows
            .iter()
            .filter(|r| r.design_id == design)
            .all(|r| r.status == strato_data::common::Status::Verified)
            && laws.sel.iter().all(|s| {
                s.rows
                    .iter()
                    .all(|r| r.status == strato_data::common::Status::Verified)
            });
        if !hist.knots.is_empty() && run_end_h > 0.0 {
            // Knots of the run, in seconds from the origin, cut at the run end.
            let mut times = vec![0.0];
            let mut temps = vec![hist.knots[0].1];
            let mut plan = Vec::new();
            if hist.idealised {
                times.push(run_end_h * H);
                temps.push(hist.knots[0].1);
                plan.push(StepSegment {
                    end_s: run_end_h * H,
                    dt_max_s: step_h * H,
                });
            } else {
                for (tk, ck) in hist.knots.iter().skip(1) {
                    let rel = (tk - hist.origin_h) * H;
                    times.push(rel);
                    temps.push(*ck);
                    let end = rel.min(run_end_h * H);
                    if plan
                        .last()
                        .is_none_or(|s: &StepSegment| end > s.end_s + 1e-9)
                    {
                        plan.push(StepSegment {
                            end_s: end,
                            dt_max_s: step_h * H,
                        });
                    }
                    if rel >= run_end_h * H {
                        break;
                    }
                }
            }
            let constant = temps.iter().all(|c| (*c - temps[0]).abs() < 1e-9);
            let temperature_c = if constant {
                TimeFunction::constant(temps[0])
            } else {
                TimeFunction::table(times.clone(), temps.clone(), Interpolation::Linear)
            }
            .map_err(|e| e.to_string())?;
            let def = ScenarioDefinition {
                id: format!("r2.{}", curve.dataset_id),
                generation_factor: TimeFunction::constant(0.0).map_err(|e| e.to_string())?,
                temperature_c,
                plan,
                event_times_s: vec![],
            };
            // The exact times: every matched time and every shifted time.
            let end_s = def.end_s();
            let mut extra: Vec<f64> = Vec::new();
            let mut shifted: Vec<(Option<f64>, Option<f64>)> = Vec::new();
            for (r, t) in inputs.iter().zip(&t_mod) {
                let ut = r.u_time_h.unwrap_or(0.0);
                match t {
                    Some(t) if *t * H <= end_s + 1e-6 => {
                        extra.push((t * H).min(end_s));
                        if ut > 0.0 {
                            let lo = ((t - ut).max(0.0) * H).min(end_s);
                            let hi = ((t + ut).max(0.0) * H).min(end_s);
                            extra.push(lo);
                            extra.push(hi);
                            shifted.push((Some(lo), Some(hi)));
                        } else {
                            shifted.push((None, None));
                        }
                    }
                    _ => shifted.push((None, None)),
                }
            }
            extra.retain(|t| *t > 1e-6 && *t < end_s - 1e-6);
            extra.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            extra.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
            let runs = run_pair(laws, radii, &def, &extra, quick)?;
            steps = runs.base.steps;
            for (i, (r, t)) in inputs.iter().zip(&t_mod).enumerate() {
                let Some(t) = t else { continue };
                let ts = (t * H).min(runs.run_end_s);
                if t * H > runs.run_end_s + 1e-6 {
                    continue;
                }
                let (Some(fb), Some(ff)) = (f_at(&runs.base, ts), f_at(&runs.fine, ts)) else {
                    continue;
                };
                a_input.model_times_h[i] = Some(*t);
                a_input.f_model[i] = Some(fb);
                a_input.e_num[i] = Some((fb - ff).abs());
                a_input.resolved[i] = ff > 0.0 && (fb - ff).abs() <= 0.05 * ff;
                if let (Some(lo), Some(hi)) = shifted[i] {
                    a_input.f_minus[i] = f_at(&runs.base, lo);
                    a_input.f_plus[i] = f_at(&runs.base, hi);
                    a_input.t_minus_h[i] = Some(lo / H);
                    a_input.t_plus_h[i] = Some(hi / H);
                }
                let _ = r;
            }
            // M1: a law evaluated outside its printed range, or a hold outside the documented
            // application of C16.
            let outside_printed = runs.base.ranges.iter().any(|c| c.outside > 0);
            let outside_documented = !frg
                && hist.holds.iter().any(|(c, _)| {
                    *c < C16_APPLICATION_C.0 - 1e-9 || *c > C16_APPLICATION_C.1 + 1e-9
                });
            a_input.validity_ok = !(outside_printed || outside_documented);
            a_input.reaches_last_point = reaches
                && inputs.iter().zip(&t_mod).all(|(r, t)| {
                    t.is_some_and(|t| (t + r.u_time_h.unwrap_or(0.0)) * H <= runs.run_end_s + 1e-6)
                });
            a_input.model_rows_verified = m_verified;
            a_input.run_ref = Some(J::obj(vec![
                ("solver_package_version", J::s(prov.solver_version.clone())),
                ("commit", J::s(prov.commit.clone())),
                ("model_config_id", J::s(model_config_id.clone())),
                ("base_output_sha256", J::s(series_hash(&runs.base))),
                ("refined_output_sha256", J::s(series_hash(&runs.fine))),
                ("steps", J::Int(runs.base.steps as i64)),
                ("matrix_assemblies", J::Int(runs.base.assemblies as i64)),
            ]));
            // The drawn curve: geometric samples of the run and every matched time.
            // x is the time in the time basis of the experimental points; t_run = x + x0.
            let x0 = basis_offset_h - hist.origin_h;
            let end_h = runs.run_end_s / H;
            let x_start = (-x0).max(0.0);
            let extra_x: Vec<f64> = t_mod.iter().flatten().map(|t| t - x0).collect();
            let mut xs: Vec<f64> =
                crate::curves::sample_times(0.05, (end_h - x0).max(0.05), &extra_x);
            xs.push(x_start);
            xs.retain(|x| *x >= x_start - 1e-9);
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            xs.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
            let ts: Vec<f64> = runs.base.rows.iter().map(|r| r.t_s).collect();
            let fs: Vec<f64> = runs
                .base
                .rows
                .iter()
                .map(|r| r.release_fraction_of_initial.unwrap_or(0.0))
                .collect();
            let mut sx = Vec::new();
            let mut sf = Vec::new();
            for x in &xs {
                let t_run = x + x0;
                if t_run * H <= runs.run_end_s + 1e-9 {
                    if let Some(v) = lin(&ts, &fs, t_run * H) {
                        sx.push(*x);
                        sf.push(v);
                    }
                }
            }
            samples = Some((sx, sf));
            // Method B: only where the applicability function allows it.
            let needs = if constant {
                Needs::baseline(temps[0])
            } else {
                Needs {
                    stage: Stage::TemperatureDependentDiffusivity,
                    source: SourceHistory::None,
                    decay_per_s: 0.0,
                    temperature: temperature_of(&def.temperature_c),
                    kernel_step_start: true,
                }
            };
            let d = laws.at(temps[0]).map_err(|e| e.to_string())?.0;
            let mut bt: Vec<f64> = xs.iter().map(|x| (x + x0) * H).collect();
            bt.extend(t_mod.iter().flatten().map(|t| t * H));
            for (lo, hi) in shifted.iter() {
                bt.extend(lo.iter().chain(hi.iter()).copied());
            }
            bt.retain(|t| *t > 0.0);
            bt.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            bt.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
            match method_b(
                &needs,
                Quantity::ReleaseFraction,
                1.0,
                &radii.outer_m,
                &d,
                &bt,
            ) {
                MethodB::Available(a) => {
                    b_input.applicable = true;
                    let at = |t_s: f64| -> Option<(f64, f64)> {
                        let k = a.times_s.iter().position(|x| (x - t_s).abs() < 1e-6)?;
                        a.values[k].map(|v| (v, a.accuracy_floor))
                    };
                    let mut mask = true;
                    for (i, t) in t_mod.iter().enumerate() {
                        let Some(t) = t else {
                            mask = false;
                            continue;
                        };
                        match at(*t * H) {
                            Some((v, e)) => {
                                b_input.model_times_h[i] = Some(*t);
                                b_input.f_model[i] = Some(v);
                                b_input.e_num[i] = Some(e);
                                b_input.resolved[i] = true;
                            }
                            None => mask = false,
                        }
                        if let (Some(lo), Some(hi)) = shifted[i] {
                            b_input.f_minus[i] = at(lo).map(|x| x.0);
                            b_input.f_plus[i] = at(hi).map(|x| x.0);
                            b_input.t_minus_h[i] = Some(lo / H);
                            b_input.t_plus_h[i] = Some(hi / H);
                            mask =
                                mask && b_input.f_minus[i].is_some() && b_input.f_plus[i].is_some();
                        }
                    }
                    b_input.mask_passed = mask;
                    b_input.reaches_last_point = a_input.reaches_last_point;
                    b_input.validity_ok = a_input.validity_ok;
                    b_input.model_rows_verified = m_verified;
                    b_input.run_ref = Some(J::obj(vec![
                        ("solver_package_version", J::s(prov.solver_version.clone())),
                        ("commit", J::s(prov.commit.clone())),
                        ("model_config_id", J::s(model_config_id.clone())),
                        ("alpha_max_per_s", J::Num(a.alpha_max_per_s)),
                        ("modes", J::Int(a.modes as i64)),
                        ("e_num_basis", J::s("the accuracy floor of the series; the difference to the series with a quarter of the eigenvalue cover is not returned by the single Method B function and is not added")),
                    ]));
                    b_samples = Some((
                        xs.clone(),
                        xs.iter().map(|x| at((x + x0) * H).map(|v| v.0)).collect(),
                    ));
                }
                MethodB::NotAvailable(n) => {
                    b_input.reaches_last_point = a_input.reaches_last_point;
                    b_input.validity_ok = a_input.validity_ok;
                    b_input.model_rows_verified = m_verified;
                    notes.push(format!(
                        "Method B: not available ({}: {})",
                        n.code, n.reason
                    ));
                }
            }
        } else {
            run_note = "no run: the registered history gives no covered time".to_string();
        }
    } else {
        run_note = format!(
            "no run: {}",
            radii
                .as_ref()
                .err()
                .or(laws.as_ref().err())
                .cloned()
                .unwrap_or_default()
        );
    }
    let history_covers = hist.omitted_first_ramp.is_none()
        && !hist.idealised
        && !hist.knots.is_empty()
        && needed_h <= hist_len_h + 1e-9;
    // ---- Labels, words, source.
    let temperature_label = match t_c_number {
        Some(t) => degrees(t),
        None => match (hold_temps_c.first(), hold_temps_c.last()) {
            (Some(lo), Some(hi)) => format!("stepped {} to {}", lo, degrees(*hi)),
            _ => "temperature history not registered".to_string(),
        },
    };
    let l = Some(curve.specimen_level);
    let words = CaptionWords {
        time_axis_label: time_axis_label(&first.time_basis),
        specimen_label: specimen_label(l, &first.compact_or_sample, Some(curve.source_aggregated)),
        y_axis_scale: axis_scale(&first.notes).to_string(),
    };
    let denominator_established = match first.denominator {
        Denominator::ParticleInventory => count.is_some(),
        Denominator::CompactInventory => true,
        _ => false,
    };
    let known_basis = ["elapsed", "time at temperature", "isothermal test duration"]
        .iter()
        .any(|b| first.time_basis.contains(b));
    let unit_known = first.unit != ReleaseUnit::PerHour;
    let data_kind = if count.is_some() {
        "calculated"
    } else if inputs.iter().all(|r| r.digitised) {
        "digitised"
    } else if inputs.iter().all(|r| !r.digitised) {
        "measured_tabulated"
    } else {
        "mixed"
    };
    let independence = if curve.dataset_id.contains("JAERI") {
        "calibration_check"
    } else {
        "not_established"
    };
    let lambda_per_h = decay_constant(reg, &curve.nuclide).ok().map(|l| l * H);
    let u_value_summary = rows.iter().find_map(|r| match &r.reading_uncertainty {
        ReadingUncertainty::Stated { value: Some(v), .. } => match v {
            ValueReading::RelativePercent(p) => Some(("rel_pct".to_string(), p.value)),
            ValueReading::Absolute(p) => Some(("abs".to_string(), p.value / count.unwrap_or(1.0))),
            _ => None,
        },
        _ => None,
    });
    let u_time_summary = inputs
        .iter()
        .filter_map(|r| r.u_time_h)
        .fold(None, |a: Option<f64>, x| Some(a.map_or(x, |y| y.max(x))));
    let configurable = reg.arrhenius.rows.iter().any(|r| {
        r.nuclide_text()
            .split(['/', ' ', '(', ')'])
            .any(|p| p == element)
    });
    let mut missing: Vec<String> = Vec::new();
    if curve.nuclide.is_empty() {
        missing.push("nuclide".into());
    }
    if first.compact_or_sample.is_empty() {
        missing.push("specimen identity".into());
    }
    if !known_basis || !unit_known {
        missing.push("time basis and unit".into());
    }
    if radii.is_err() {
        missing.push("particle geometry".into());
    }
    let register_marks_failure = rows
        .iter()
        .any(|r| r.group == strato_data::release::Group::FailedSic);
    let input = CurveInput {
        curve_id: curve.dataset_id.clone(),
        nuclide: curve.nuclide.clone(),
        program: first.program.as_str().to_string(),
        specimen_id: first.compact_or_sample.clone(),
        specimen_level: Some(curve.specimen_level),
        source_aggregated: Some(curve.source_aggregated),
        temperature_c: t_c_number,
        temperature_label,
        temperature_history_id: Some(curve.history_id.clone()),
        release_definition: first.quantity.as_str().to_string(),
        denominator: first.denominator.as_str().to_string(),
        time_basis: first.time_basis.clone(),
        model_config_id: model_config_id.clone(),
        independence: independence.to_string(),
        source: source_info(table, &first.source, &curve.link, &curve.source_location),
        data_kind: data_kind.to_string(),
        registered_unit: first.unit.as_str().to_string(),
        conversion,
        rows: inputs,
        coating_failure_basis: Some(curve.coating_failure_basis.clone()),
        register_marks_failure,
        matrix_separated_by_source: false,
        decay_correction: curve.decay_correction,
        // The curve assessment file has no columns for these fields: they are unset, never
        // defaulted.
        decay_reference: None::<DecayReference>,
        inventory_reference: None::<InventoryReference>,
        inventory_reference_offset_h: None,
        lambda_per_h,
        assessment_verification_ref: (!curve.verification_ref.is_empty())
            .then(|| curve.verification_ref.clone()),
        r2_verification_ref: None,
        nuclide_configurable: configurable,
        denominator_established,
        definition_matches: known_basis,
        history_covers,
        history_rows_verified: hist.all_verified,
        missing_conditions: missing,
        premises,
        u_systematic: None,
        u_value_summary,
        u_time_summary_h: u_time_summary,
        words,
    };
    let a_out = configurable.then_some(a_input);
    let b_out = configurable.then_some(b_input);
    let drawn = [
        samples.is_some() && configurable,
        b_samples
            .as_ref()
            .is_some_and(|s| s.1.iter().any(Option::is_some)),
    ];
    // ---- The model part of the comparison file.
    let model_json = J::obj(vec![
        (
            "method_a",
            match &samples {
                Some((x, f)) if configurable => J::obj(vec![
                    ("label", J::s(Method::A.name())),
                    ("status", J::s("computed")),
                    ("time_h", J::Arr(x.iter().map(|v| J::Num(*v)).collect())),
                    (
                        "release_fraction",
                        J::Arr(f.iter().map(|v| J::Num(*v)).collect()),
                    ),
                    ("model_config_id", J::s(model_config_id.clone())),
                    ("steps", J::Int(steps as i64)),
                    ("history_row_ids", J::Arr(hist.row_ids.iter().map(J::s).collect())),
                    (
                        "note",
                        J::s("the line is a drawing between output times; the values that enter R\u{b2} are the exact step values in r2.json"),
                    ),
                ]),
                _ => J::obj(vec![
                    ("label", J::s(Method::A.name())),
                    ("status", J::s("not computed")),
                    (
                        "reason_code",
                        J::s(if configurable {
                            ReasonCode::MissingTemperatureHistory.code()
                        } else {
                            ReasonCode::ModelNotConfigurable.code()
                        }),
                    ),
                    ("reason", J::s(if run_note.is_empty() { "no diffusivity row for the nuclide".to_string() } else { run_note.clone() })),
                ]),
            },
        ),
        (
            "method_b",
            match &b_samples {
                Some((x, f)) => J::obj(vec![
                    ("label", J::s(Method::B.name())),
                    ("status", J::s("available")),
                    ("time_h", J::Arr(x.iter().map(|v| J::Num(*v)).collect())),
                    ("release_fraction", J::Arr(f.iter().map(|v| J::opt(*v)).collect())),
                ]),
                None => J::obj(vec![
                    ("label", J::s(Method::B.name())),
                    ("status", J::s("not available")),
                    ("reason", J::s(notes.first().cloned().unwrap_or_else(|| "no model".to_string()))),
                ]),
            },
        ),
    ]);
    let classes = classes_of(&input);
    let points: Vec<J> = input
        .rows
        .iter()
        .zip(&classes)
        .map(|(r, c)| {
            J::obj(vec![
                ("id", J::s(r.id.clone())),
                ("time_h", J::Num(r.time_h)),
                ("value", J::Num(r.value)),
                ("registered_value", J::Num(r.registered_value)),
                ("registered_unit", J::s(input.registered_unit.clone())),
                ("qualifier", J::s(r.qualifier.clone())),
                ("value_class", J::s(r.value_class.clone())),
                ("point_class", J::s(class_word(*c))),
                ("status", J::s(r.status.clone())),
                (
                    "reading_uncertainty",
                    J::obj(vec![
                        ("value_abs", J::opt(r.u_value)),
                        ("time_h", J::opt(r.u_time_h)),
                    ]),
                ),
            ])
        })
        .collect();
    let experiment_json = J::obj(vec![
        ("marker_colour", J::s(r2_colour_exp())),
        ("connect", J::Bool(false)),
        ("points", J::Arr(points)),
    ]);
    Ok(Built {
        input,
        a: a_out,
        b: b_out,
        drawn,
        model_json,
        experiment_json,
    })
}

fn r2_colour_exp() -> &'static str {
    crate::r2curve::COLOUR_EXP
}

/// Run every curve of the assessment file.
///
/// # Errors
///
/// A row or file that cannot be read, or a model run that fails.
pub fn run_all(prov: &Prov, reg: &Register, repo: &Path, quick: bool) -> Result<R2Run, String> {
    let markdown = std::fs::read_to_string(
        repo.join("data")
            .join("reference")
            .join(strato_data::register::REGISTER_MARKDOWN),
    )
    .map_err(|e| format!("the parameter register cannot be read: {e}"))?;
    let table = source_table(&markdown);
    let mut out = R2Run {
        records: Vec::new(),
        captions: Vec::new(),
        files: Files::new(),
        first_failing: Vec::new(),
    };
    for curve in &reg.curves.curves {
        let built = build(prov, reg, &table, curve, quick)?;
        let [a, b] = assess(&built.input, built.a.as_ref(), built.b.as_ref());
        let caption = caption_json(&built.input, &a, &b, built.drawn, None);
        out.first_failing
            .push((curve.dataset_id.clone(), a.first_failing.map(r2::Check::id)));
        out.files.insert(
            format!("comparison/{}.json", curve.dataset_id),
            json_file(
                prov,
                "one experimental curve with its model curves and its R\u{b2} records",
                vec![
                    ("curve_id", J::s(curve.dataset_id.clone())),
                    ("definition_id", J::s(r2::DEFINITION_ID)),
                    ("nuclide", J::s(curve.nuclide.clone())),
                    (
                        "register",
                        J::s("data/experimental/release_measurements.csv; data/experimental/curve_assessment.csv"),
                    ),
                    ("experiment", built.experiment_json.clone()),
                    ("model", built.model_json.clone()),
                    ("r2_records", J::Arr(vec![a.json.clone(), b.json.clone()])),
                    ("caption", caption.clone()),
                ],
            ),
        );
        out.captions.push(caption);
        out.records.push(a);
        out.records.push(b);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_particle_count_is_read_from_the_registered_text() {
        assert_eq!(
            particle_count("4-4-2 loose particles (75 particles)"),
            Some(75.0)
        );
        assert_eq!(
            particle_count("cumulative release in particle equivalents (75 loose particles)"),
            Some(75.0)
        );
        assert_eq!(particle_count("4-3-3"), None);
    }

    #[test]
    fn the_words_of_the_caption_come_from_the_register() {
        assert_eq!(
            time_axis_label("time at temperature (heating time at 1600 C)"),
            "time at temperature (h)"
        );
        assert_eq!(
            time_axis_label("elapsed time from start of heating"),
            "elapsed time from start of heating (h)"
        );
        assert_eq!(
            specimen_label(
                Some(SpecimenLevel::ParticleBatch),
                "4-4-2 loose particles (75 particles)",
                Some(false)
            ),
            "loose particles from compact 4-4-2"
        );
        assert_eq!(
            specimen_label(Some(SpecimenLevel::Compact), "6-4-3", Some(false)),
            "compact 6-4-3"
        );
        assert_eq!(
            specimen_label(Some(SpecimenLevel::Sphere), "HFR-K3/1 sphere", Some(true)),
            "fuel sphere HFR-K3/1, source mean"
        );
        assert_eq!(axis_scale("logarithmic from 1e-9 to 1e0"), "log");
        assert_eq!(axis_scale("linear or logarithmic scale to 0"), "linear");
        assert_eq!(degrees(1600.0), "1600 \u{b0}C");
    }

    #[test]
    fn the_source_table_gives_the_apa_reference_and_the_link_check() {
        let md = "## 3. Sources (APA 7)\n| Key | Reference | Local file | Link |\n|---|---|---|---|\n| IAEA (1997) | International Atomic Energy Agency. (1997). *Fuel performance*. IAEA. | 2.pdf | candidate URL (NOT VERIFIED) |\n| IAEA (1997), URL check of 2026-11-26 | Note. | 2.pdf | answers HTTP 200 |\n| Hunn et al. (2014) 4-4-2 | Hunn et al. (2014). Loose-particle test (ORNL/LTR-2014/486). ORNL. | none | https://x |\n## 4. Geometry\n";
        let t = source_table(md);
        assert_eq!(t.len(), 3);
        let s = source_info(&t, "IAEA (1997)", "https://iaea", "Fig. 1; printed p. 2");
        assert!(s.apa.starts_with("International Atomic Energy Agency."));
        assert!(!s.apa.contains('*'), "markdown italics are removed");
        assert_eq!(s.link_status, "verified");
        let h = source_info(
            &t,
            "Hunn et al. (2014; ORNL/LTR-2014/486)",
            "https://x",
            "Fig. 4",
        );
        assert!(h.apa.contains("ORNL/LTR-2014/486"));
        assert!(h.link_status.starts_with("no link check"));
    }
}
