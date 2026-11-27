//! Operating scenarios and the options of the selectors: `run_scenario` and `options`.
//!
//! A scenario is a prescribed-input simulation: its generation, power and temperature histories
//! are the rows of `data/scenarios/` (each number a registered source row or labelled
//! `prescribed input (assumed)`), turned into the histories and the step plan of the scenario
//! engine by `strato_scenarios::definitions::build`, the function that writes
//! `data/results/scenarios.json`. The files are embedded in the module (`embedded.rs`). No
//! scenario is a validated reactor prediction; the engine models no neutron kinetics, control-rod
//! motion, thermal-hydraulics, power redistribution or mechanical refuelling.

use super::btcs_api::{finite_or_null, LABEL_BTCS};
use super::embedded;
use super::stage_api::{
    engine, method_b_json, rejected, MAX_SYSTEM_INTERVALS, MAX_SYSTEM_ROWS, SCHEMA_STAGE,
};
use super::{check_schema_value, MAX_SERIES_MODES};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::sync::OnceLock;
use strato::analytical::applicability::Stage;
use strato::analytical::extensions::Quantity;
use strato::numerical::physics_run::{Interpolation, TimeFunction};
use strato::numerical::scenario::{post_event_fraction, InitialState, ScenarioRun};
use strato_data::scenarios::{
    Record, ScenarioFile, ScenarioRow, Scenarios, MEANINGFUL, NOT_MEANINGFUL,
};
use strato_data::select::{Phase, Selection};
use strato_data::table::Vocabulary;
use strato_data::Register;
use strato_scenarios::definitions::{
    build, interp, kernel_intervals, pick, rate_target, run, Built, Numerics, Setup, SLUGS,
};
use strato_scenarios::geometry::radii;
use strato_scenarios::laws::{
    generation_nuclides, rate_density, reference_generation, Laws, Nuclide, LAYERS,
};
use strato_scenarios::methodb::{
    method_b, source_of, temperature_of, MethodB, Needs, NotAvailable,
};
use strato_scenarios::support::{
    exp_temperatures, EXPL_HEATING_C, EXPL_IRRADIATION_C, EXP_NUCLIDES, STAGES,
};

/// Wrapper limit: time steps of the base run of a scenario (the refined run has twice as many).
pub const MAX_SCENARIO_STEPS: usize = 20_000;
/// Wrapper limit: total grid intervals of a scenario run.
pub const MAX_SCENARIO_INTERVALS: usize = 60_000;
/// Wrapper limit: grid nodes times steps summed over the runs of one call (the base run, the run
/// of normal operation and, if asked for, the refined run with four times the work). Not a
/// physical parameter: it bounds the work of one call.
pub const MAX_SCENARIO_WORK: f64 = 4.0e8;
/// The largest kernel grid that the default grid chooses; the rule of the results pipeline asks
/// for `sqrt(D t) / dr >= 10`, which needs up to 50 000 intervals for Sr-90.
pub const DEFAULT_KERNEL_CAP: usize = 2_000;
/// Default and largest number of rows of the returned series.
pub const DEFAULT_ROWS: usize = 400;
/// The most rows a reply may carry.
pub const MAX_ROWS: usize = 2_000;
/// The intervals of the four coatings of the default grid (the pipeline's base setting).
const DEFAULT_COATING_INTERVALS: [usize; 4] = [20, 20, 160, 20];
/// The statement of every scenario (decision 58), as in `scenarios.json`.
pub const SCENARIO_STATEMENT: &str = "Every scenario is a prescribed-input simulation. Each number carries its source row id or the label prescribed input (assumed). No scenario is a validated reactor prediction. STRATO does not model neutron kinetics, control-rod motion, thermal-hydraulics, power redistribution or mechanical refuelling.";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GridIn {
    intervals_per_layer: [usize; 5],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioRequest {
    schema: u32,
    scenario_id: String,
    #[serde(default)]
    variant: Option<String>,
    nuclide: String,
    #[serde(default)]
    physics_stage: Option<u8>,
    #[serde(default)]
    grid: Option<GridIn>,
    #[serde(default)]
    step_multiplier: Option<f64>,
    #[serde(default)]
    check_resolution: Option<bool>,
    #[serde(default)]
    max_rows: Option<usize>,
}

fn slug_of(id: &str) -> &'static str {
    SLUGS.iter().find(|s| s.0 == id).map_or("scenario", |s| s.1)
}

fn interpolation_name(m: Interpolation) -> &'static str {
    match m {
        Interpolation::Step => "step",
        Interpolation::Linear => "linear",
    }
}

fn time_function_value(f: &TimeFunction, scale: f64) -> Value {
    match f {
        TimeFunction::Constant(v) => json!({"kind": "constant", "value": v * scale}),
        TimeFunction::Table {
            times,
            values,
            mode,
        } => json!({
            "kind": "table",
            "interpolation": interpolation_name(*mode),
            "times_s": times,
            "times_h": times.iter().map(|t| t / 3600.0).collect::<Vec<f64>>(),
            "values": values.iter().map(|v| v * scale).collect::<Vec<f64>>(),
        }),
    }
}

fn row_value(r: &ScenarioRow) -> Value {
    json!({
        "row_id": r.row_id,
        "record": match r.record {
            Record::Meta => "meta",
            Record::History => "history",
            Record::Step => "step",
        },
        "variant": r.variant,
        "quantity": r.quantity,
        "time": r.t_value.as_ref().map(|t| t.value),
        "time_unit": r.t_unit,
        "t_s": r.t_seconds().ok(),
        "value": r.value.as_ref().map(|v| v.value),
        "unit": r.unit,
        "interpolation": r.interpolation,
        "source_ids": r.source_ids,
        "basis": r.basis,
        "text": r.text,
    })
}

/// Every row that defines the scenario variant: the metadata and the rows of the variant, or the
/// base rows where the variant has none of that quantity, as `build` reads them.
fn prescribed_inputs(f: &ScenarioFile, variant: &str) -> Vec<Value> {
    let mut quantities: Vec<&str> = Vec::new();
    for r in &f.rows {
        if r.record != Record::Meta
            && (r.variant.is_empty() || r.variant == variant)
            && !quantities.contains(&r.quantity.as_str())
        {
            quantities.push(&r.quantity);
        }
    }
    let mut out: Vec<Value> = f
        .rows
        .iter()
        .filter(|r| r.record == Record::Meta && (r.variant.is_empty() || r.variant == variant))
        .map(row_value)
        .collect();
    for q in quantities {
        let rows: Vec<&ScenarioRow> =
            if q == "step" || f.steps(variant).iter().any(|r| r.quantity == q) {
                f.steps(variant)
            } else {
                f.history(variant, q)
            };
        out.extend(rows.into_iter().map(row_value));
    }
    out
}

fn find_scenario<'a>(sc: &'a Scenarios, id: &str) -> Result<&'a ScenarioFile, Vec<String>> {
    let key = SLUGS
        .iter()
        .find(|(i, slug)| *i == id || *slug == id)
        .map(|s| s.0)
        .ok_or_else(|| {
            vec![format!(
                "scenario_id `{id}` is not one of: {}",
                SLUGS
                    .iter()
                    .map(|(i, s)| format!("{s} ({i})"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )]
        })?;
    sc.get(key)
        .ok_or_else(|| vec![format!("the scenario file {key} is not embedded")])
}

fn find_nuclide(reg: &Register, id: &str) -> Result<Nuclide, Vec<String>> {
    let all = generation_nuclides(reg).map_err(|e| vec![e])?;
    all.iter().find(|n| n.id == id).cloned().ok_or_else(|| {
        vec![format!(
            "nuclide `{id}` has no scenario: a scenario needs a direct fission-product source, which {} have (decision 15); {} get none. Choose one of: {}",
            MEANINGFUL.join(" and "),
            NOT_MEANINGFUL.join(", "),
            all.iter().map(|n| n.id).collect::<Vec<_>>().join(", ")
        )]
    })
}

fn allowed_stages(scenario_id: &str) -> &'static [u8] {
    if scenario_id == "S1" {
        &[2, 3, 5]
    } else {
        &[5]
    }
}

fn work_of(intervals: &[usize; 5], steps: usize) -> f64 {
    (intervals.iter().sum::<usize>() + 1) as f64 * steps as f64
}

fn series_of(rows: &[usize], run: &ScenarioRun, post: &[Option<f64>]) -> Value {
    let r = &run.rows;
    let col =
        |f: &dyn Fn(usize) -> f64| -> Vec<Value> { rows.iter().map(|&k| json!(f(k))).collect() };
    let ocol = |f: &dyn Fn(usize) -> Option<f64>| -> Vec<Value> {
        rows.iter()
            .map(|&k| f(k).map_or(Value::Null, finite_or_null))
            .collect()
    };
    let d_layers: Vec<Vec<Value>> = (0..5)
        .map(|l| rows.iter().map(|&k| json!(r[k].d_m2_s[l])).collect())
        .collect();
    json!({
        "times_s": col(&|k| r[k].t_s),
        "times_h": col(&|k| r[k].t_s / 3600.0),
        "times_d": col(&|k| r[k].t_s / 86_400.0),
        "inventory_mol": col(&|k| r[k].inventory_mol),
        "generated_mol": col(&|k| r[k].generated_mol),
        "released_mol": col(&|k| r[k].released_mol),
        "decayed_mol": col(&|k| r[k].decayed_mol),
        "balance_residual_mol": col(&|k| r[k].balance_residual_mol),
        "balance_residual_bound_mol": col(&|k| r[k].balance_bound_mol),
        "release_fraction_of_generated": ocol(&|k| r[k].release_fraction_of_total),
        "release_fraction_of_initial": ocol(&|k| r[k].release_fraction_of_initial),
        "release_to_generation_ratio": ocol(&|k| r[k].release_to_generation_ratio),
        "post_event_fraction": ocol(&|k| post[k]),
        "temperature_c": col(&|k| r[k].temperature_c),
        "generation_factor": col(&|k| r[k].generation_factor),
        "kernel_source_mol_m3_s": col(&|k| r[k].kernel_source_mol_m3_s),
        "diffusivity_m2_s": d_layers,
        "min_concentration_mol_m3": col(&|k| r[k].min_concentration),
    })
}

fn ranges_value(run: &ScenarioRun) -> Value {
    Value::Array(
        run.ranges
            .iter()
            .zip(LAYERS)
            .map(|(c, (name, _))| {
                json!({
                    "layer": name,
                    "steps_inside_printed_range": c.within,
                    "steps_outside_printed_range": c.outside,
                    "steps_no_range_printed": c.not_stated,
                })
            })
            .collect(),
    )
}

fn run_scenario_value(raw: &str) -> Result<Value, Vec<String>> {
    let req: ScenarioRequest =
        serde_json::from_str(raw).map_err(|e| vec![format!("request: {e}")])?;
    check_schema_value(req.schema, SCHEMA_STAGE)?;
    let reg = embedded::register().map_err(|e| vec![e])?;
    let sc = embedded::scenarios().map_err(|e| vec![e])?;
    let file = find_scenario(sc, &req.scenario_id)?;
    let id = file.id.as_str();
    let variant = req.variant.clone().unwrap_or_default();
    if !variant.is_empty() && !file.variants().contains(&variant) {
        let mut v = vec!["\"\" (the base)".to_string()];
        v.extend(file.variants().iter().map(|x| format!("\"{x}\"")));
        return Err(vec![format!(
            "variant `{variant}` is not a variant of scenario {} ({id}); the variants are: {}",
            slug_of(id),
            v.join(", ")
        )]);
    }
    let nuclide = find_nuclide(reg, &req.nuclide)?;
    let stage = req.physics_stage.unwrap_or(5);
    if !allowed_stages(id).contains(&stage) {
        return Err(vec![format!(
            "physics_stage {stage} is not available for scenario {} ({id}): a scenario starts from a fresh particle and needs generation, so it runs at stage {}{}",
            slug_of(id),
            allowed_stages(id)
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            if id == "S1" {
                ""
            } else {
                " (the page moves to stage 5 when a scenario other than steady-state operation is chosen)"
            }
        )]);
    }
    let mult = req.step_multiplier.unwrap_or(1.0);
    if !(mult.is_finite() && mult >= 1.0) {
        return Err(vec![format!(
            "step_multiplier is {mult}; it must be a finite number of at least 1 (it multiplies every largest step of the scenario's step plan)"
        )]);
    }
    let max_rows = req.max_rows.unwrap_or(DEFAULT_ROWS);
    if !(2..=MAX_ROWS).contains(&max_rows) {
        return Err(vec![format!(
            "max_rows is {max_rows}; it must be a whole number from 2 to {MAX_ROWS}"
        )]);
    }
    let check = req.check_resolution.unwrap_or(false);

    let geometry = radii(reg).map_err(|e| vec![e])?;
    let radii_m = geometry.outer_m;
    let target = rate_target(file, &variant).map_err(|e| vec![e])?;
    let rate = rate_density(reg, &target).map_err(|e| vec![e])?;
    let built = build(sc, reg, id, &variant, rate.value).map_err(|e| vec![e])?;
    let g_ref = reference_generation(nuclide.yield_per_fission, rate.value);
    let lambda = if stage == 2 {
        0.0
    } else {
        nuclide.lambda_per_s
    };

    // The grid: the rule of the pipeline for the kernel (capped for the page), its base setting
    // for the coatings, or the grid the caller gives.
    let mut messages: Vec<String> = Vec::new();
    let irr = Laws::new(reg, nuclide.element, Phase::Irradiation).map_err(|e| vec![e])?;
    let mut def = built.def.clone();
    for seg in &mut def.plan {
        seg.dt_max_s *= mult;
    }
    let intervals: [usize; 5] = match &req.grid {
        Some(g) => g.intervals_per_layer,
        None => {
            let d_k = irr.at(built.t_normal_c).map_err(|e| vec![e.to_string()])?.0[0];
            let rule = kernel_intervals(radii_m[0], d_k, def.end_s(), false);
            let n_k = rule.min(DEFAULT_KERNEL_CAP);
            if n_k < rule {
                messages.push(format!(
                    "The default kernel grid was limited to {n_k} intervals; the resolution rule of the results files asks for {rule}. Give grid.intervals_per_layer to choose, and check_resolution to see where the result is resolved"
                ));
            }
            [
                n_k,
                DEFAULT_COATING_INTERVALS[0],
                DEFAULT_COATING_INTERVALS[1],
                DEFAULT_COATING_INTERVALS[2],
                DEFAULT_COATING_INTERVALS[3],
            ]
        }
    };
    if intervals.contains(&0) {
        return Err(vec![
            "grid.intervals_per_layer: every layer needs at least 1 interval".into(),
        ]);
    }
    let total: usize = intervals.iter().sum();
    if total > MAX_SCENARIO_INTERVALS {
        return Err(vec![format!(
            "The total number of grid intervals in all layers is {total}, above the limit of {MAX_SCENARIO_INTERVALS} grid intervals of a scenario"
        )]);
    }
    let steps_planned = def.steps(&[]).map_err(|e| vec![format!("{e:#}")])?.len();
    let factor = if check { 2 } else { 1 };
    if steps_planned * factor > MAX_SCENARIO_STEPS {
        let need = ((steps_planned * factor) as f64 / MAX_SCENARIO_STEPS as f64 * mult).ceil();
        return Err(vec![format!(
            "The scenario needs {steps_planned} time steps{}, above the limit of {MAX_SCENARIO_STEPS} time steps; give step_multiplier {need} or more",
            if check { " (twice that for the refined run of check_resolution)" } else { "" }
        )]);
    }
    // The base run and the run of normal operation, and with check_resolution the refined run
    // (twice the nodes and twice the steps: four times the work).
    let work = work_of(&intervals, steps_planned) * if check { 6.0 } else { 2.0 };
    if work > MAX_SCENARIO_WORK {
        return Err(vec![format!(
            "The work of the call ({} grid nodes times {steps_planned} time steps, for {} runs of that size = {work:.3e}) is above the limit of {MAX_SCENARIO_WORK:.1e}; use fewer grid intervals or a larger step_multiplier{}",
            total + 1,
            if check { "six" } else { "two" },
            if check { ", or leave check_resolution off" } else { "" }
        )]);
    }

    let built_run = Built {
        def: def.clone(),
        ..built.clone()
    };
    let setup = |scale: f64, mult_n: usize| Setup {
        reg,
        radii_m,
        nuclide: &nuclide,
        g_ref,
        lambda,
        numerics: Numerics {
            intervals: intervals.map(|n| n * mult_n),
            dt_scale: scale,
        },
        profile_times_s: vec![],
        initial: InitialState::Fresh,
    };
    let base = run(&def, &built_run, &setup(1.0, 1)).map_err(|e| vec![e])?;
    let reference_def = def
        .normal_operation(1.0, built.t_normal_c)
        .map_err(|e| vec![format!("{e:#}")])?;
    let reference = run(&reference_def, &built_run, &setup(1.0, 1)).map_err(|e| vec![e])?;
    let fine = if check {
        Some(run(&def, &built_run, &setup(0.5, 2)).map_err(|e| vec![e])?)
    } else {
        None
    };

    let t_e = built.t_switch_s.or(built.event_s);
    let plotted_post = t_e.is_some() && matches!(id, "S2" | "S5");
    let post = |r: &ScenarioRun| -> Vec<Option<f64>> {
        r.rows
            .iter()
            .map(|row| t_e.and_then(|e| post_event_fraction(r, e, row.t_s)))
            .collect()
    };
    let post_base = post(&base);
    let n = base.rows.len();
    let times: Vec<f64> = base.rows.iter().map(|r| r.t_s).collect();
    // The rows to return: at most `max_rows`, with the event rows kept.
    let mut must = Vec::new();
    for e in [t_e, built.event_s].into_iter().flatten() {
        if let Some(k) = times
            .iter()
            .position(|t| (*t - e).abs() <= 1e-9 * e.max(1.0))
        {
            must.push(k);
        }
    }
    let sel = pick(n, max_rows, &must);
    let plot_of = |r: &ScenarioRun, p: &[Option<f64>]| -> Vec<Option<f64>> {
        if plotted_post {
            p.to_vec()
        } else {
            r.rows.iter().map(|x| x.release_fraction_of_total).collect()
        }
    };
    let resolved: Option<Vec<Value>> = fine.as_ref().map(|f| {
        let plot = plot_of(&base, &post_base);
        let post_f = post(f);
        let plot_f = plot_of(f, &post_f);
        let ft: Vec<f64> = f.rows.iter().map(|r| r.t_s).collect();
        sel.iter()
            .map(|&k| match (plot[k], interp(&ft, &plot_f, times[k])) {
                (Some(a), Some(b)) => json!(b > 0.0 && (a - b).abs() <= 0.05 * b),
                _ => json!(false),
            })
            .collect()
    });
    let max_inv_diff = fine.as_ref().map(|f| {
        let ft: Vec<f64> = f.rows.iter().map(|r| r.t_s).collect();
        let inv: Vec<Option<f64>> = f.rows.iter().map(|r| Some(r.inventory_mol)).collect();
        base.rows
            .iter()
            .filter_map(|r| {
                interp(&ft, &inv, r.t_s)
                    .filter(|b| *b > 0.0)
                    .map(|b| (r.inventory_mol - b).abs() / b)
            })
            .fold(0.0, f64::max)
    });

    // Method B through the one function (Scenario 1 at a constant temperature only).
    let times_sel: Vec<f64> = sel.iter().map(|&k| times[k]).collect();
    let needs = Needs {
        stage: match stage {
            2 => Stage::Generation,
            3 => Stage::Decay,
            _ => Stage::Combined,
        },
        source: source_of(&built.def.generation_factor, g_ref, built.def.end_s()),
        decay_per_s: lambda,
        temperature: temperature_of(&built.def.temperature_c),
        kernel_step_start: true,
    };
    let b_answer = match strato_scenarios::methodb::applicability(&needs) {
        Err(nv) => MethodB::NotAvailable(nv),
        Ok(_) if plotted_post => MethodB::NotAvailable(NotAvailable {
            code: "POST_EVENT_NOT_DEFINED",
            reason: "the post-event fraction is not defined for Method B".into(),
        }),
        Ok(_) => {
            let d = irr
                .at(built.def.temperature_c.value_at(0.0))
                .map_err(|e| vec![e.to_string()])?
                .0;
            method_b(
                &needs,
                Quantity::ReleaseFractionGenerated,
                0.0,
                &radii_m,
                &d,
                &times_sel,
            )
        }
    };

    let ref_ok = reference.rows.len() == n;
    let ref_post = post(&reference);
    let ref_json = if ref_ok {
        json!({
            "grey_line_colour": "#6B7280",
            "temperature_c": built.t_normal_c,
            "series": series_of(&sel, &reference, &ref_post),
        })
    } else {
        Value::Null
    };

    let slug = slug_of(id);
    let prescribed = prescribed_inputs(file, &variant);
    let c15_outside_steps: usize = base.ranges.iter().map(|c| c.outside).sum();
    if c15_outside_steps > 0 {
        messages.push(
            "At least one diffusivity law was evaluated outside its printed validity range; the value is an extrapolation of the correlation (steps per layer in diagnostics.ranges)".into(),
        );
    }
    let series = series_of(&sel, &base, &post_base);
    Ok(json!({
        "schema": SCHEMA_STAGE,
        "status": "ok",
        "label": LABEL_BTCS,
        "engine": engine(),
        "solver": {
            "id": super::btcs_api::SOLVER_ID,
            "capability": "diffusion_btcs_layered",
            "scope": "the scenario engine of the library: the BTCS scheme with generation, decay and a temperature history; the diffusivity of each layer from the registered law at the temperature in the middle of each step",
            "nature": "numerical solution of a prescribed-input simulation; not a validated reactor prediction and not experimental validation",
        },
        "scenario": {
            "id": slug,
            "definition": id,
            "variant": variant,
            "title": built.description,
            "statement": SCENARIO_STATEMENT,
            "prescribed_inputs": prescribed,
            "rate_density": {
                "target": rate.target,
                "value_fissions_m3_s": rate.value,
                "label": rate.label,
                "source_ids": rate.source_ids,
            },
            "normal_operation_temperature_c": built.t_normal_c,
            "event_time_s": built.event_s,
            "diffusivity_set_switch_time_s": built.t_switch_s,
            "end_time_s": built.def.end_s(),
            "steps_of_plan": steps_planned,
        },
        "stage": {
            "physics_stage": stage,
            "mechanisms": if stage == 2 {
                json!(["generation"])
            } else {
                json!(["generation", "decay", "diffusivity_temperature"])
            },
        },
        "nuclide": {
            "id": nuclide.id,
            "name": nuclide.name,
            "element": nuclide.element,
            "decay_constant_per_s": nuclide.lambda_per_s,
            "decay_constant_used_per_s": lambda,
            "yield_per_fission": nuclide.yield_per_fission,
            "reference_generation_mol_m3_s": g_ref,
        },
        "grid": {
            "intervals_per_layer": intervals,
            "intervals_total": total,
            "node_radii_outer_m": radii_m,
            "refined_intervals_per_layer": fine.as_ref().map(|_| intervals.map(|v| v * 2)),
        },
        "series": series,
        "resolved": resolved,
        "histories": {
            "generation_factor": time_function_value(&built.def.generation_factor, 1.0),
            "temperature_c": time_function_value(&built.def.temperature_c, 1.0),
            "kernel_source_mol_m3_s": time_function_value(&built.def.generation_factor, g_ref),
            "at_rows": {
                "times_s": times_sel,
                "power_fraction": sel.iter().map(|&k| base.rows[k].generation_factor).collect::<Vec<f64>>(),
                "temperature_c": sel.iter().map(|&k| base.rows[k].temperature_c).collect::<Vec<f64>>(),
                "kernel_source_mol_m3_s": sel.iter().map(|&k| base.rows[k].kernel_source_mol_m3_s).collect::<Vec<f64>>(),
            },
        },
        "reference_normal_operation": ref_json,
        "method_b": method_b_json(&b_answer, &times_sel),
        "plotted_quantity": if plotted_post {
            "post-event fraction (released since the event / inventory at the event)"
        } else {
            "release fraction of the generated inventory F_gen = released / (M0 + generated)"
        },
        "numerics": {
            "steps": base.steps,
            "distinct_step_lengths": base.distinct_steps,
            "matrix_assemblies": base.assemblies,
            "matrix_min_row_margin": base.matrix_min_row_margin,
            "min_pivot_margin": base.min_pivot_margin,
            "step_multiplier": mult,
            "resolution_checked": check,
            "max_inventory_refinement_difference": max_inv_diff,
            "refined_steps": fine.as_ref().map(|f| f.steps),
            "resolution_rule": "a point is resolved if the base run and the refined run (twice the intervals per layer, half the step) differ by at most 5 % of the refined value; the 5 % is a labelled convention, not a derived bound",
        },
        "balance": {
            "max_abs_residual_over_bound": base.max_residual_over_bound,
            "max_relative_residual": base.max_relative_residual,
            "bound": "(F46), Document 5 Section 5.8",
            "within_bound": base.max_residual_over_bound <= 1.0,
        },
        "diagnostics": {
            "ranges": ranges_value(&base),
            "rows_returned": sel.len(),
            "rows_computed": n,
        },
        "units": {
            "times_s": "s", "times_h": "h", "times_d": "d",
            "inventory_mol": "mol", "generated_mol": "mol", "released_mol": "mol", "decayed_mol": "mol",
            "balance_residual_mol": "mol", "balance_residual_bound_mol": "mol",
            "release_fraction_of_generated": "1 (released / (M0 + generated); null until generation starts)",
            "release_fraction_of_initial": "1 (released / M0; null where M0 = 0)",
            "release_to_generation_ratio": "1 (release rate / generation rate; null where undefined)",
            "post_event_fraction": "1 (null before the event or where undefined)",
            "temperature_c": "degC", "generation_factor": "1 (multiple of the reference generation rate)",
            "kernel_source_mol_m3_s": "mol/(m^3 s)", "diffusivity_m2_s": "m^2/s per layer kernel, buffer, IPyC, SiC, OPyC",
            "min_concentration_mol_m3": "mol/m^3",
            "rate_density": "fissions/(m^3 s)",
            "value_fissions_m3_s": "fissions/(m^3 s)",
            "decay_constant_per_s": "1/s", "decay_constant_used_per_s": "1/s",
            "yield_per_fission": "1 (atoms per fission)", "reference_generation_mol_m3_s": "mol/(m^3 s)",
            "normal_operation_temperature_c": "degC", "end_time_s": "s", "event_time_s": "s",
            "steps_of_plan": "1 (steps of the planned run)", "steps": "1", "refined_steps": "1 (steps of the refined run)",
            "distinct_step_lengths": "1 (number of different step lengths)", "matrix_assemblies": "1",
            "matrix_min_row_margin": "1", "min_pivot_margin": "1", "step_multiplier": "1",
            "resolution_checked": "boolean", "resolved": "boolean per row (base and refined runs agree within 5 %)",
            "max_inventory_refinement_difference": "1 (relative)",
            "intervals_per_layer": "1 (per layer kernel, buffer, IPyC, SiC, OPyC)", "refined_intervals_per_layer": "1 (per layer)",
            "intervals_total": "1", "node_radii_outer_m": "m (outer radius of each layer)",
            "max_abs_residual_over_bound": "1 (largest residual / its bound)", "max_relative_residual": "1 (largest residual / (M0 + generated))",
            "within_bound": "boolean", "rows_returned": "1", "rows_computed": "1",
            "power_fraction": "1 (multiple of the reference power)",
            "value": "unit of the history named by the parent key: generation_factor 1, temperature_c degC, kernel_source_mol_m3_s mol/(m^3 s)",
            "values": "unit of the history named by the parent key: generation_factor 1, temperature_c degC, kernel_source_mol_m3_s mol/(m^3 s)",
        },
        "messages": messages,
    }))
}

/// Runs a scenario (schema 3 request) with the engine of the library and returns its series with
/// the normal-operation reference.
pub fn run_scenario(request_json: &str) -> String {
    match run_scenario_value(request_json) {
        Ok(v) => v.to_string(),
        Err(e) => rejected(e).to_string(),
    }
}

// ---------------------------------------------------------------------------------------
// options
// ---------------------------------------------------------------------------------------

fn validity_intersection(sel: &[Selection<'_>]) -> (Value, Vec<&'static str>) {
    let mut lo = f64::NEG_INFINITY;
    let mut hi = f64::INFINITY;
    let mut none = Vec::new();
    for (s, (name, _)) in sel.iter().zip(LAYERS) {
        match s.published_range_c {
            Some((a, b)) => {
                lo = lo.max(a);
                hi = hi.min(b);
            }
            None => none.push(name),
        }
    }
    let v = if lo.is_finite() && hi.is_finite() && lo <= hi {
        json!({"min": lo, "max": hi})
    } else {
        Value::Null
    };
    (v, none)
}

fn options_value() -> Result<Value, Vec<String>> {
    let reg = embedded::register().map_err(|e| vec![e])?;
    let sc = embedded::scenarios().map_err(|e| vec![e])?;
    let nuclides = generation_nuclides(reg).map_err(|e| vec![e])?;
    let mut scenarios = Vec::new();
    for (id, slug) in SLUGS {
        let file = sc
            .get(id)
            .ok_or_else(|| vec![format!("the scenario file {id} is not embedded")])?;
        let mut names = vec![String::new()];
        names.extend(file.variants());
        let mut variants = Vec::new();
        for v in &names {
            let target = rate_target(file, v).map_err(|e| vec![e])?;
            let rate = rate_density(reg, &target).map_err(|e| vec![e])?;
            let built = build(sc, reg, id, v, rate.value).map_err(|e| vec![e])?;
            let steps = built
                .def
                .steps(&[])
                .map_err(|e| vec![format!("{e:#}")])?
                .len();
            variants.push(json!({
                "variant": v,
                "rate_density_target": rate.target,
                "rate_density_value_fissions_m3_s": rate.value,
                "rate_density_label": rate.label,
                "normal_operation_temperature_c": built.t_normal_c,
                "event_time_s": built.event_s,
                "end_time_s": built.def.end_s(),
                "end_time_d": built.def.end_s() / 86_400.0,
                "steps_of_plan": steps,
                "has_heating_history": built.heating_only.is_some(),
            }));
        }
        scenarios.push(json!({
            "id": slug,
            "definition": id,
            "title": file.meta("", "title").map(|r| r.text.clone()),
            "supported": true,
            "physics_stages": allowed_stages(id),
            "min_stage": if id == "S1" { 2 } else { 5 },
            "nuclides": nuclides.iter().map(|n| n.id).collect::<Vec<_>>(),
            "variants": variants,
        }));
    }
    let mut exploration = Vec::new();
    for nuc in &EXP_NUCLIDES {
        let mut phases = Vec::new();
        for phase in [Phase::Irradiation, Phase::HeatingTest] {
            let laws = Laws::new(reg, nuc.element, phase).map_err(|e| vec![e])?;
            let temps: &[f64] = if phase == Phase::Irradiation {
                &EXPL_IRRADIATION_C
            } else {
                &EXPL_HEATING_C
            };
            let (inter, none) = validity_intersection(&laws.sel);
            phases.push(json!({
                "phase": phase.as_str(),
                "temperatures_c": temps,
                "validity_intersection_c": inter,
                "layers_without_printed_range": none,
                "layers": laws.sel.iter().zip(LAYERS).map(|(s, (name, _))| json!({
                    "layer": name,
                    "register_rows": s.row_ids(),
                    "published_range_c": s.published_range_c.map(|(a, b)| json!({"min": a, "max": b})),
                    "coefficient_class": s.class.as_str(),
                })).collect::<Vec<Value>>(),
            }));
        }
        exploration.push(json!({"nuclide": nuc.id, "name": nuc.name, "phases": phases}));
    }
    Ok(json!({
        "schema": SCHEMA_STAGE,
        "status": "ok",
        "label": LABEL_BTCS,
        "engine": engine(),
        "stages": STAGES.iter().map(super::stage_api::stage_label).collect::<Vec<Value>>(),
        "mechanisms": [
            {"id": "generation", "term": "+ G(r, t)", "implemented": true, "first_stage": 2},
            {"id": "decay", "term": "- lambda C", "implemented": true, "first_stage": 3},
            {"id": "diffusivity_temperature", "term": "D(r) becomes D(r, T(t)) in the operator", "implemented": true, "first_stage": 4},
        ],
        "scenarios": scenarios,
        "scenario_statement": SCENARIO_STATEMENT,
        "nuclides": {
            "scenario": nuclides.iter().map(|n| json!({
                "id": n.id, "name": n.name, "element": n.element,
                "decay_constant_per_s": n.lambda_per_s,
                "yield_per_fission": n.yield_per_fission,
            })).collect::<Vec<Value>>(),
            "no_generation_source": NOT_MEANINGFUL,
            "experiment": EXP_NUCLIDES.iter().map(|n| json!({
                "id": n.id, "name": n.name, "label": n.label, "element": n.element,
                "main_comparison_temperatures_c": exp_temperatures(n.id),
            })).collect::<Vec<Value>>(),
        },
        "temperatures": {
            "main_comparison_c": EXP_NUCLIDES.iter().map(|n| (n.id.to_string(), json!(exp_temperatures(n.id)))).collect::<Map<String, Value>>(),
            "exploration": exploration,
        },
        "limits": {
            "stage_run": super::btcs_api::limits_value(),
            "btcs_system": {"max_total_intervals": MAX_SYSTEM_INTERVALS, "max_rows": MAX_SYSTEM_ROWS},
            "scenario": {
                "max_total_intervals": MAX_SCENARIO_INTERVALS,
                "max_steps": MAX_SCENARIO_STEPS,
                "max_work_nodes_times_steps": MAX_SCENARIO_WORK,
                "default_kernel_cap": DEFAULT_KERNEL_CAP,
                "default_rows": DEFAULT_ROWS,
                "max_rows": MAX_ROWS,
            },
            "analytical": {"max_times": super::stage_api::MAX_ANALYTICAL_TIMES, "max_series_modes": MAX_SERIES_MODES},
        },
        "embedded_files": super::embedded::EMBEDDED_FILES.iter().map(|(p, b)| json!({"path": p, "bytes": b})).collect::<Vec<Value>>(),
        "units": {
            "temperatures_c": "degC", "decay_constant_per_s": "1/s",
            "yield_per_fission": "atoms per fission", "end_time_s": "s", "end_time_d": "d",
            "rate_density": "fissions/(m^3 s)",
        },
        "messages": Vec::<String>::new(),
    }))
}

static OPTIONS: OnceLock<String> = OnceLock::new();

/// The supported stages, scenarios, nuclides and temperatures, from the embedded definitions.
/// The request may be empty or `{"schema":3}`.
pub fn options(request_json: &str) -> String {
    let trimmed = request_json.trim();
    if !trimmed.is_empty() {
        match serde_json::from_str::<Value>(trimmed) {
            Ok(Value::Object(m)) => {
                if let Some(s) = m.get("schema").and_then(Value::as_u64) {
                    if let Err(e) = check_schema_value(s as u32, SCHEMA_STAGE) {
                        return rejected(e).to_string();
                    }
                }
                if m.keys().any(|k| k != "schema") {
                    return rejected(vec![
                        "request: options takes no field other than schema".into()
                    ])
                    .to_string();
                }
            }
            Ok(_) => return rejected(vec!["request: must be a JSON object".into()]).to_string(),
            Err(e) => return rejected(vec![format!("request: {e}")]).to_string(),
        }
    }
    if let Some(s) = OPTIONS.get() {
        return s.clone();
    }
    match options_value() {
        Ok(v) => OPTIONS.get_or_init(|| v.to_string()).clone(),
        Err(e) => rejected(e).to_string(),
    }
}
