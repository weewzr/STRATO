//! Equation stages of the wrapper (ABI 4, schema 3): `run_stage`, `btcs_system` and `analytical`.
//!
//! A stage request is the configuration of `run` (schema 2) plus a stage number 1 to 5 and the
//! optional `physics` table of the library's `[physics]` section (`numerical::physics_config`),
//! written as JSON. The stage says which mechanisms are on (`strato_scenarios::support::STAGES`);
//! a section of a mechanism that is off is refused, never ignored. Every number comes from the
//! solver of the library (`physics_run::run_physics`, `btcs::Stepper`) or from the one Method B
//! function of `strato-scenarios`; the wrapper adds only bookkeeping over their values.
//!
//! Nothing here reads a file or a clock, and no rule is left to the library to panic on: the
//! limits of `run` apply (2 000 intervals, 20 000 steps, 1 000 000 profile values).

use super::btcs_api::{
    check_config, config_from_request_value, finite_or_null, grid_value, output_steps, Plan,
    LABEL_BTCS, MAX_PROFILE_TIMES, MAX_PROFILE_VALUES_BTCS, SOLVER_ID,
};
use super::{check_schema_value, engine_value};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use strato::analytical::applicability::{SourceHistory, Stage, TemperatureHistory};
use strato::analytical::extensions::Quantity;
use strato::config::Config;
use strato::numerical::{
    btcs::Stepper,
    grid::{Grid, NodeClass, ValidatedInput},
    inventory::{initial_vector, inventory},
    physics_config::{load_physics, PhysicsLabels, PhysicsSection},
    physics_run::{run_physics, source_nodes, DiffusivityModel, PhysicsInput},
};
use strato::physics::arrhenius::Temperature;
use strato::physics::CELSIUS_OFFSET_K;
use strato_scenarios::methodb::{
    method_b, source_of, temperature_of, MethodB, Needs, NotAvailable, WITHHELD_LEGEND,
};
use strato_scenarios::support::{
    stage_def, StageDef, MECHANISM_DECAY, MECHANISM_DIFFUSIVITY_TEMPERATURE, MECHANISM_GENERATION,
};

/// Schema version of every request and reply of the stage and scenario functions.
pub const SCHEMA_STAGE: u32 = 3;
/// Wrapper limit of `btcs_system`: the system is shown for a small grid.
pub const MAX_SYSTEM_INTERVALS: usize = 200;
/// Wrapper limit of `btcs_system`: rows shown in one reply.
pub const MAX_SYSTEM_ROWS: usize = 50;
/// Wrapper limit of `analytical`: times at which the series is evaluated.
pub const MAX_ANALYTICAL_TIMES: usize = 2_000;
/// The capability ids of the new functions (the ids of the equation builder's switches and the
/// two engines).
pub const CAPABILITIES_STAGE: &[&str] = &[
    MECHANISM_GENERATION,
    MECHANISM_DECAY,
    MECHANISM_DIFFUSIVITY_TEMPERATURE,
    "scenario_engine",
    "analytical_applicability",
];

/// The block of `build_info()` that describes the functions of ABI 4. The page reads which
/// mechanisms and engines the build implements from `capabilities`, never from a flag of its own.
pub fn build_info_value() -> Value {
    json!({
        "schema": SCHEMA_STAGE,
        "functions": ["run_stage", "run_scenario", "btcs_system", "analytical", "options"],
        "capabilities": CAPABILITIES_STAGE,
        "label": LABEL_BTCS,
        "stages": strato_scenarios::support::STAGES.iter().map(stage_label).collect::<Vec<Value>>(),
        "limits": {
            "max_total_intervals": super::btcs_api::MAX_TOTAL_INTERVALS,
            "max_steps": super::btcs_api::MAX_STEPS_BTCS,
            "max_profile_values": MAX_PROFILE_VALUES_BTCS,
            "max_profile_times": MAX_PROFILE_TIMES,
            "btcs_system_max_total_intervals": MAX_SYSTEM_INTERVALS,
            "btcs_system_max_rows": MAX_SYSTEM_ROWS,
            "analytical_max_times": MAX_ANALYTICAL_TIMES,
            "scenario_max_total_intervals": super::scenario_api::MAX_SCENARIO_INTERVALS,
            "scenario_max_steps": super::scenario_api::MAX_SCENARIO_STEPS,
            "scenario_max_work": super::scenario_api::MAX_SCENARIO_WORK,
        },
    })
}

pub(crate) fn rejected(errors: Vec<String>) -> Value {
    json!({"schema": SCHEMA_STAGE, "status": "rejected", "errors": errors, "engine": engine()})
}

pub(crate) fn engine() -> Value {
    let mut e = engine_value();
    if let Value::Object(m) = &mut e {
        m.insert("solver".into(), json!(SOLVER_ID));
    }
    e
}

pub(crate) fn stage_label(def: &StageDef) -> Value {
    json!({
        "stage": def.stage,
        "id": def.id,
        "name": def.name,
        "adds": def.adds,
        "mechanisms": def.mechanisms,
    })
}

// ---------------------------------------------------------------------------------------
// The common request
// ---------------------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StageCore {
    schema: u32,
    config: Value,
    stage: u8,
    #[serde(default)]
    physics: Option<Value>,
}

/// A prepared stage request: validated configuration, the library input of the stage and the
/// labels the library records beside it.
pub(crate) struct Prepared {
    pub cfg: Config,
    pub input: ValidatedInput,
    pub plan: Plan,
    pub def: &'static StageDef,
    pub pin: PhysicsInput,
    pub labels: PhysicsLabels,
}

/// Splits `extra` keys off a request object and types the rest.
fn split_request(
    raw: &str,
    extra: &[&str],
) -> Result<(StageCore, Map<String, Value>), Vec<String>> {
    let mut v: Value = serde_json::from_str(raw).map_err(|e| vec![format!("request: {e}")])?;
    let Value::Object(map) = &mut v else {
        return Err(vec!["request: the request must be a JSON object".into()]);
    };
    let mut taken = Map::new();
    for k in extra {
        if let Some(x) = map.remove(*k) {
            taken.insert((*k).to_string(), x);
        }
    }
    let core: StageCore = serde_json::from_value(v).map_err(|e| vec![format!("request: {e}")])?;
    Ok((core, taken))
}

fn strip_nulls(v: Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(
            m.into_iter()
                .filter(|(_, x)| !x.is_null())
                .map(|(k, x)| (k, strip_nulls(x)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.into_iter().map(strip_nulls).collect()),
        other => other,
    }
}

/// The `[physics]` table of the library as TOML text, after the rule that a section of a
/// mechanism that is off is refused and a mechanism that needs inputs has them.
fn physics_text(def: &StageDef, physics: &Option<Value>) -> Result<Option<String>, Vec<String>> {
    let on = |m: &str| def.mechanisms.contains(&m);
    let Some(v) = physics else {
        if on(MECHANISM_DIFFUSIVITY_TEMPERATURE) {
            return Err(vec![format!(
                "Stage {} has a temperature-dependent diffusivity: give physics.temperature and \
                 one physics.arrhenius law for each of the five layers",
                def.stage
            )]);
        }
        return Ok(None);
    };
    let section: PhysicsSection =
        serde_json::from_value(v.clone()).map_err(|e| vec![format!("physics: {e}")])?;
    let mut errors = Vec::new();
    if section.source.is_some() && !on(MECHANISM_GENERATION) {
        errors.push(format!(
            "physics.source is given, but stage {} ({}) has no generation; choose stage 2 or higher or remove the source",
            def.stage, def.name
        ));
    }
    if section.decay.is_some() && !on(MECHANISM_DECAY) {
        errors.push(format!(
            "physics.decay is given, but stage {} ({}) has no decay; choose stage 3 or higher or remove the decay",
            def.stage, def.name
        ));
    }
    let d_t = on(MECHANISM_DIFFUSIVITY_TEMPERATURE);
    if (section.temperature.is_some() || section.arrhenius.is_some()) && !d_t {
        errors.push(format!(
            "physics.temperature or physics.arrhenius is given, but stage {} ({}) has a constant diffusivity per layer; choose stage 4 or 5 or remove them",
            def.stage, def.name
        ));
    }
    if d_t && (section.temperature.is_none() || section.arrhenius.is_none()) {
        errors.push(format!(
            "Stage {} has a temperature-dependent diffusivity: give physics.temperature and one physics.arrhenius law for each of the five layers",
            def.stage
        ));
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let text = toml::to_string(&json!({ "physics": strip_nulls(v.clone()) })).map_err(|e| {
        vec![format!(
            "physics: the table cannot be passed to the library: {e}"
        )]
    })?;
    Ok(Some(text))
}

fn prepare_core(core: StageCore) -> Result<Prepared, Vec<String>> {
    check_schema_value(core.schema, SCHEMA_STAGE)?;
    let def = stage_def(core.stage).ok_or_else(|| {
        vec![format!(
            "stage is {}; it must be a whole number from 1 to 5",
            core.stage
        )]
    })?;
    let cfg = config_from_request_value(core.config)?;
    let (input, plan) = check_config(&cfg)?;
    let end_time_s = input.steps as f64 * input.dt;
    let text = physics_text(def, &core.physics)?;
    let (pin, labels) = match text {
        None => (
            PhysicsInput::baseline(input.clone(), end_time_s),
            PhysicsLabels {
                source: "no source".into(),
                decay: "no decay (stable species)".into(),
                diffusivity: vec!["constant diffusivity from the regions of the input".into()],
                ..PhysicsLabels::default()
            },
        ),
        Some(t) => {
            let setup = load_physics(&t, input.clone(), end_time_s)
                .map_err(|e| vec![format!("physics: {e:#}")])?
                .ok_or_else(|| vec!["physics: the table is empty".to_string()])?;
            (setup.input, setup.labels)
        }
    };
    Ok(Prepared {
        cfg,
        input,
        plan,
        def,
        pin,
        labels,
    })
}

fn prepare(raw: &str, extra: &[&str]) -> Result<(Prepared, Map<String, Value>), Vec<String>> {
    let (core, taken) = split_request(raw, extra)?;
    Ok((prepare_core(core)?, taken))
}

fn numbers(v: Option<&Value>, what: &str) -> Result<Option<Vec<f64>>, Vec<String>> {
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(a)) => a
            .iter()
            .map(|x| {
                x.as_f64()
                    .ok_or_else(|| vec![format!("{what}: every entry must be a number")])
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        Some(_) => Err(vec![format!("{what}: must be a list of numbers")]),
    }
}

// ---------------------------------------------------------------------------------------
// Echo
// ---------------------------------------------------------------------------------------

fn temperature_used(p: &Prepared) -> bool {
    matches!(p.pin.diffusivity, DiffusivityModel::Arrhenius { .. })
}

fn inputs_value(p: &Prepared) -> Value {
    let cfg = &p.cfg;
    let source_at_start: Vec<f64> = p
        .pin
        .source
        .per_material
        .iter()
        .map(|g| g.value_at(0.0))
        .collect();
    json!({
        "case_name": cfg.case_name,
        "species": cfg.species,
        "temperature_k": cfg.temperature_k,
        "temperature_used_by_solver": temperature_used(p),
        "regions": cfg.regions.iter().map(|r| json!({
            "name": r.name,
            "outer_radius_m": r.outer_radius_m,
            "diffusivity_m2_s": r.diffusivity_m2_s,
            "intervals": r.cells,
        })).collect::<Vec<Value>>(),
        "initial": {"kernel_concentration_mol_m3": cfg.initial.kernel_concentration_mol_m3},
        "time": {
            "dt_s": cfg.time.dt_s,
            "end_time_s": cfg.time.end_time_s,
            "output_every_steps": cfg.time.output_every_steps,
        },
        "decay_constant_per_s": p.pin.decay_constant_per_s,
        "source_at_start_mol_m3_s": source_at_start,
        "labels": {
            "source": p.labels.source,
            "fission_rate_origin": p.labels.fission_rate_origin,
            "diffusivity": p.labels.diffusivity,
            "decay": p.labels.decay,
        },
    })
}

fn solver_scope(p: &Prepared) -> Value {
    json!({
        "id": SOLVER_ID,
        "capability": "diffusion_btcs_layered",
        "label": LABEL_BTCS,
        "formulation": "docs/mathematics/btcs/02_final_formulation.md and 05 (generation, decay, temperature-dependent diffusivity)",
        "scope": format!(
            "stage {} ({}): kernel-only start, absorbing outer surface; {}",
            p.def.stage,
            p.def.name,
            if temperature_used(p) {
                "diffusivity of each layer from its Arrhenius law at the temperature in the middle of each step"
            } else {
                "constant diffusivity in each layer"
            }
        ),
        "nature": "numerical solution of the stated model; not experimental validation",
    })
}

// ---------------------------------------------------------------------------------------
// run_stage
// ---------------------------------------------------------------------------------------

fn d_used_in_step(pin: &PhysicsInput, step: usize) -> Result<[f64; 5], String> {
    match &pin.diffusivity {
        DiffusivityModel::Constant(d) => Ok(*d),
        DiffusivityModel::Arrhenius {
            laws,
            gas,
            temperature_k,
        } => {
            // The expression of `run_physics` (F36): the temperature at the middle of the step.
            let t_mid = temperature_k.value_at(((step - 1) as f64 + 0.5) * pin.base.dt);
            let mut d = [0.0; 5];
            for (k, law) in laws.iter().enumerate() {
                d[k] = law
                    .evaluate(Temperature::Kelvin(t_mid), *gas)
                    .map_err(|e| e.to_string())?
                    .d_m2_s;
            }
            Ok(d)
        }
    }
}

fn source_in_step(pin: &PhysicsInput, step: usize) -> [f64; 5] {
    let dt = pin.base.dt;
    let (t_a, t_b) = ((step - 1) as f64 * dt, step as f64 * dt);
    std::array::from_fn(|k| pin.source.per_material[k].mean_over(t_a, t_b))
}

/// The one-line answer of the applicability table for the stage of a prepared request.
fn needs_of(p: &Prepared) -> Needs {
    let pin = &p.pin;
    let end = pin.end_time_s;
    let coating_source = pin.source.per_material[1..].iter().any(|g| !g.is_zero());
    let source = if pin.source.is_zero() {
        SourceHistory::None
    } else if coating_source {
        SourceHistory::NotKernelUniform
    } else {
        source_of(&pin.source.per_material[0], 1.0, end)
    };
    let temperature = match &pin.diffusivity {
        DiffusivityModel::Arrhenius { temperature_k, .. } => temperature_of(
            &temperature_k
                .map_values(|k| Ok(k - CELSIUS_OFFSET_K))
                .unwrap_or_else(|_| temperature_k.clone()),
        ),
        DiffusivityModel::Constant(_) => {
            TemperatureHistory::Constant(p.cfg.temperature_k - CELSIUS_OFFSET_K)
        }
    };
    Needs {
        stage: match p.def.stage {
            1 => Stage::Baseline,
            2 => Stage::Generation,
            3 => Stage::Decay,
            4 => Stage::TemperatureDependentDiffusivity,
            _ => Stage::Combined,
        },
        source,
        decay_per_s: pin.decay_constant_per_s,
        temperature,
        kernel_step_start: true,
    }
}

fn applicability_value(needs: &Needs) -> Value {
    match strato_scenarios::methodb::applicability(needs) {
        Ok(ext) => json!({
            "applicable": true,
            "extension": ext.map(|e| e.formulas()),
            "reason_code": null,
            "reason": null,
        }),
        Err(NotAvailable { code, reason }) => json!({
            "applicable": false,
            "extension": null,
            "reason_code": code,
            "reason": reason,
        }),
    }
}

fn run_stage_value(raw: &str) -> Result<Value, Vec<String>> {
    let (p, extra) = prepare(raw, &["profile_times_s"])?;
    let profile_times = numbers(extra.get("profile_times_s"), "profile_times_s")?;
    if let Some(t) = &profile_times {
        if t.len() > MAX_PROFILE_TIMES {
            return Err(vec![format!(
                "{} profile times were requested, above the limit of {MAX_PROFILE_TIMES} profile times",
                t.len()
            )]);
        }
        if t.iter().any(|x| !x.is_finite() || *x < 0.0) {
            return Err(vec![
                "Every requested profile time (s) must be a finite number of at least 0 s".into(),
            ]);
        }
    }
    let n_profiles = profile_times.as_ref().map_or(1, Vec::len);
    let values = n_profiles.saturating_mul(p.plan.nodes);
    if values > MAX_PROFILE_VALUES_BTCS {
        return Err(vec![format!(
            "{n_profiles} concentration profiles of {} grid nodes each give {values} stored concentration values, above the limit of {MAX_PROFILE_VALUES_BTCS} values; request fewer profile times",
            p.plan.nodes
        )]);
    }
    let snapshot_times = profile_times.clone().unwrap_or_default();
    let run = run_physics(&p.pin, &snapshot_times)
        .map_err(|e| vec![format!("the solver stopped: {e:#}")])?;
    let steps_of_rows = output_steps(&p.input);
    if steps_of_rows.len() != run.series.len() {
        return Err(vec![
            "internal check failed: the output rows of the run do not match the output schedule"
                .into(),
        ]);
    }
    let m0 = run.meta.initial_inventory;
    let col = |f: &dyn Fn(usize) -> f64| -> Vec<Value> {
        (0..run.series.len()).map(|k| json!(f(k))).collect()
    };
    let opt_col = |f: &dyn Fn(usize) -> Option<f64>| -> Vec<Value> {
        (0..run.series.len())
            .map(|k| f(k).map_or(Value::Null, finite_or_null))
            .collect()
    };
    let s = &run.series;
    let times_s = col(&|k| s[k].0);
    let times_h = col(&|k| s[k].0 / 3600.0);
    let temp_k = opt_col(&|k| s[k].2.temperature_k);
    let temp_c = opt_col(&|k| s[k].2.temperature_k.map(|t| t - CELSIUS_OFFSET_K));
    // Per layer, the value used in the step that ends at each row (null at the first row).
    let mut d_layers: Vec<Vec<Value>> = vec![Vec::new(); 5];
    let mut g_layers: Vec<Vec<Value>> = vec![Vec::new(); 5];
    for &step in &steps_of_rows {
        if step == 0 {
            for k in 0..5 {
                d_layers[k].push(Value::Null);
                g_layers[k].push(Value::Null);
            }
        } else {
            let d = d_used_in_step(&p.pin, step).map_err(|e| vec![e])?;
            let g = source_in_step(&p.pin, step);
            for k in 0..5 {
                d_layers[k].push(json!(d[k]));
                g_layers[k].push(json!(g[k]));
            }
        }
    }
    let max_abs = s
        .iter()
        .map(|r| r.1.balance_residual.abs())
        .fold(0.0, f64::max);
    let max_rel = s
        .iter()
        .map(|r| r.2.balance_residual_relative.abs())
        .fold(0.0, f64::max);
    let within = s
        .iter()
        .all(|r| r.1.balance_residual.abs() <= r.1.balance_residual_bound);
    let max_ratio = s
        .iter()
        .filter(|r| r.1.balance_residual_bound > 0.0)
        .map(|r| r.1.balance_residual.abs() / r.1.balance_residual_bound)
        .fold(0.0, f64::max);
    let last = s.last().expect("a run has at least the initial row");
    let profiles: Vec<Value> = if profile_times.is_some() {
        run.snapshots
            .iter()
            .map(|(t, c)| {
                json!({
                    "t_s": t,
                    "t_h": t / 3600.0,
                    "c_mol_per_m3": c,
                    "inventory_mol": inventory(&run.grid, c),
                })
            })
            .collect()
    } else {
        vec![json!({
            "t_s": last.0,
            "t_h": last.0 / 3600.0,
            "c_mol_per_m3": run.final_profile,
            "inventory_mol": inventory(&run.grid, &run.final_profile),
        })]
    };
    let needs = needs_of(&p);
    let mut messages: Vec<String> = Vec::new();
    if (p.plan.dt_s - p.cfg.time.dt_s).abs() > f64::EPSILON * p.cfg.time.dt_s {
        messages.push(format!(
            "The time step was changed from {} s to {} s so that a whole number of steps fills the simulation duration",
            p.cfg.time.dt_s, p.plan.dt_s
        ));
    }
    let ranges: Vec<Value> = run
        .meta
        .ranges
        .iter()
        .zip(["kernel", "buffer", "IPyC", "SiC", "OPyC"])
        .map(|(r, name)| {
            json!({
                "layer": name,
                "steps_inside_printed_range": r.within,
                "steps_below_printed_range": r.below,
                "steps_above_printed_range": r.above,
                "steps_no_range_printed": r.not_stated,
            })
        })
        .collect();
    if run.meta.ranges.iter().any(|r| r.below + r.above > 0) {
        messages.push(
            "The temperature left the printed validity range of at least one diffusivity law; the value is an extrapolation of the correlation (the counts per layer are in diagnostics.ranges)".into(),
        );
    }
    Ok(json!({
        "schema": SCHEMA_STAGE,
        "status": "ok",
        "label": LABEL_BTCS,
        "engine": engine(),
        "solver": solver_scope(&p),
        "stage": stage_label(p.def),
        "mechanisms": p.def.mechanisms,
        "inputs": inputs_value(&p),
        "grid": grid_value(&run.grid, &p.cfg, &p.input),
        "series": {
            "times_s": times_s,
            "times_h": times_h,
            "steps_of_rows": steps_of_rows,
            "release_fraction": col(&|k| s[k].1.release_fraction),
            "release_fraction_of_total": col(&|k| s[k].2.release_fraction_of_total),
            "release_to_generation_ratio": opt_col(&|k| s[k].2.release_to_generation_ratio),
            "release_rate_mol_s": opt_col(&|k| s[k].1.release_rate),
            "inventory_mol": col(&|k| s[k].1.inventory),
            "remaining_fraction": col(&|k| s[k].1.inventory / m0),
            "cumulative_release_mol": col(&|k| s[k].1.cumulative_release),
            "generated_mol": col(&|k| s[k].2.generated),
            "decayed_mol": col(&|k| s[k].2.decayed),
            "decayed_fraction_of_total": col(&|k| s[k].2.decayed / (m0 + s[k].2.generated)),
            "balance_residual_mol": col(&|k| s[k].1.balance_residual),
            "balance_residual_bound_mol": col(&|k| s[k].1.balance_residual_bound),
            "balance_residual_relative": col(&|k| s[k].2.balance_residual_relative),
            "min_concentration_mol_m3": col(&|k| s[k].1.min_concentration),
            "max_concentration_mol_m3": col(&|k| s[k].1.max_concentration),
            "temperature_k": temp_k,
            "temperature_c": temp_c,
            "diffusivity_used_m2_s": d_layers,
            "source_step_average_mol_m3_s": g_layers,
        },
        "profiles": profiles,
        "balance": {
            "form": "btcs-F41",
            "statement": "Residual = remaining + released + decayed - start - generated (F41); the page prints this statement and does not re-sign the number",
            "start_inventory_mol": m0,
            "final": {
                "t_s": last.0,
                "remaining_mol": last.1.inventory,
                "released_mol": last.1.cumulative_release,
                "generated_mol": last.2.generated,
                "decayed_mol": last.2.decayed,
                "residual_mol": last.1.balance_residual,
                "residual_bound_mol": last.1.balance_residual_bound,
            },
            "max_abs_residual_mol": max_abs,
            "max_abs_residual_relative_to_total": max_rel,
            "tolerance": {
                "kind": match run.meta.bound_kind {
                    strato::numerical::physics_run::BoundKind::Baseline => "derived a-posteriori rounding bound of the baseline scheme (Document 2 Section 7.4)",
                    strato::numerical::physics_run::BoundKind::Combined => "derived a-posteriori rounding bound (F46), Document 5 Section 5.8",
                },
                "max_ratio_residual_to_bound": max_ratio,
                "within_bound": within,
            },
        },
        "diagnostics": {
            "steps": run.meta.steps,
            "dt_s": run.meta.dt,
            "intervals_total": run.meta.n,
            "kernel_surface_start_fraction": run.meta.theta,
            "decay_number": run.meta.decay_number,
            "matrix_assemblies": run.meta.assemblies,
            "matrix_min_row_margin": run.meta.matrix_min_row_margin,
            "matrix_max_interior_margin_deviation": run.meta.matrix_max_interior_margin_deviation,
            "matrix_norm_inf": run.meta.matrix_norm_inf,
            "min_pivot_margin": run.meta.min_pivot_margin,
            "max_multiplier": run.meta.max_multiplier,
            "first_step_generation_weight_sum_mol_s": run.meta.first_step_generation_weight_sum,
            "ranges": ranges,
            "final_release_fraction": last.1.release_fraction,
            "final_release_fraction_of_total": last.2.release_fraction_of_total,
        },
        "analytical": applicability_value(&needs),
        "units": {
            "times_s": "s", "times_h": "h", "release_fraction": "1 (released / initial inventory M0)",
            "release_fraction_of_total": "1 (released / (M0 + generated))",
            "release_to_generation_ratio": "1 (release rate / generation rate; null where the generation rate is zero or at the first row)",
            "release_rate_mol_s": "mol/s (null at the first row)",
            "inventory_mol": "mol", "remaining_fraction": "1 (inventory / M0)",
            "cumulative_release_mol": "mol", "generated_mol": "mol", "decayed_mol": "mol",
            "decayed_fraction_of_total": "1 (decayed / (M0 + generated))",
            "balance_residual_mol": "mol", "balance_residual_bound_mol": "mol",
            "balance_residual_relative": "1 (residual / (M0 + generated))",
            "min_concentration_mol_m3": "mol/m^3", "max_concentration_mol_m3": "mol/m^3",
            "temperature_k": "K (null at a constant diffusivity)", "temperature_c": "degC (null at a constant diffusivity)",
            "diffusivity_used_m2_s": "m^2/s, per layer kernel, buffer, IPyC, SiC, OPyC, in the step that ends at the row (temperature in the middle of the step, F36); null at the first row",
            "source_step_average_mol_m3_s": "mol/(m^3 s), per layer, the step average of the source (F32) of the step that ends at the row; null at the first row",
            "c_mol_per_m3": "mol/m^3",
            "steps_of_rows": "1 (index of the time step that each row ends at)",
            "applicable": "boolean (true where Method B applies to this configuration)",
            "decay_constant_per_s": "1/s (zero without decay)",
            "decay_number": "1 (decay constant times the step length)",
            "dt_s": "s", "end_time_s": "s", "t_s": "s",
            "steps": "1 (number of time steps)", "output_every_steps": "1 (steps between two rows)",
            "intervals_total": "1 (number of intervals of all layers)",
            "intervals_per_layer": "1 (per layer kernel, buffer, IPyC, SiC, OPyC)",
            "nodes": "1 (number of nodes)",
            "interface_nodes": "1 (node index of each of the four interfaces)",
            "interface_radii_m": "m", "node_radii_m": "m", "ring_edges_m": "m (radial extent of the colour band of each node, presentation geometry)",
            "spacing_per_layer_m": "m (interval length per layer)",
            "layer_of_interval": "1 (layer index 0 to 4 of each interval)",
            "node_layer_inner": "1 (layer index on the inner side of the node)", "node_layer_outer": "1 (layer index on the outer side of the node)",
            "kernel_concentration_mol_m3": "mol/m^3", "source_at_start_mol_m3_s": "mol/(m^3 s)",
            "temperature_used_by_solver": "boolean (true where the diffusivity follows a temperature history)",
            "start_inventory_mol": "mol", "remaining_mol": "mol", "released_mol": "mol",
            "residual_mol": "mol (residual of F41, not re-signed)", "residual_bound_mol": "mol",
            "max_abs_residual_mol": "mol", "max_abs_residual_relative_to_total": "1 (largest residual / (M0 + generated))",
            "max_ratio_residual_to_bound": "1 (largest residual / its bound)", "within_bound": "boolean (every residual within its bound)",
            "final_release_fraction": "1 (released / M0 at the last row)", "final_release_fraction_of_total": "1 (released / (M0 + generated) at the last row)",
            "kernel_surface_start_fraction": "1 (theta of F22)",
            "matrix_assemblies": "1 (number of matrix assemblies)",
            "matrix_min_row_margin": "1 (smallest diagonal-dominance margin of the rows)",
            "matrix_max_interior_margin_deviation": "1 (largest deviation of an interior row margin from its analytical value)",
            "matrix_norm_inf": "1 (infinity norm of the matrix)", "min_pivot_margin": "1 (smallest pivot margin of the elimination)",
            "max_multiplier": "1 (largest multiplier of the elimination)",
            "first_step_generation_weight_sum_mol_s": "mol/s",
        },
        "messages": messages,
    }))
}

/// Runs a stage (schema 3 request) and returns the complete reply.
pub fn run_stage(request_json: &str) -> String {
    match run_stage_value(request_json) {
        Ok(v) => v.to_string(),
        Err(e) => rejected(e).to_string(),
    }
}

// ---------------------------------------------------------------------------------------
// analytical
// ---------------------------------------------------------------------------------------

fn series_diffusivity(p: &Prepared) -> Result<[f64; 5], String> {
    match &p.pin.diffusivity {
        DiffusivityModel::Constant(d) => Ok(*d),
        DiffusivityModel::Arrhenius {
            laws,
            gas,
            temperature_k,
        } => {
            // A varying temperature is refused by the applicability table before the values are
            // used; at a constant temperature the law is evaluated once.
            let t = temperature_k.value_at(0.0);
            let mut d = [0.0; 5];
            for (k, law) in laws.iter().enumerate() {
                d[k] = law
                    .evaluate(Temperature::Kelvin(t), *gas)
                    .map_err(|e| e.to_string())?
                    .d_m2_s;
            }
            Ok(d)
        }
    }
}

fn analytical_value(raw: &str) -> Result<Value, Vec<String>> {
    let (p, extra) = prepare(raw, &["times_s"])?;
    let times_s: Vec<f64> = match numbers(extra.get("times_s"), "times_s")? {
        Some(t) => t,
        None => output_steps(&p.input)
            .into_iter()
            .skip(1)
            .map(|s| s as f64 * p.input.dt)
            .collect(),
    };
    if times_s.len() > MAX_ANALYTICAL_TIMES {
        return Err(vec![format!(
            "{} times were requested, above the limit of {MAX_ANALYTICAL_TIMES} times",
            times_s.len()
        )]);
    }
    if times_s.iter().any(|t| !t.is_finite() || *t < 0.0) {
        return Err(vec![
            "Every time (s) of the analytical series must be a finite number of at least 0 s"
                .into(),
        ]);
    }
    let needs = needs_of(&p);
    let d = series_diffusivity(&p).map_err(|e| vec![e])?;
    let radii = p.input.radii;
    let kernel_volume = 4.0 / 3.0 * std::f64::consts::PI * radii[0].powi(3);
    let m0 = p.input.c_init * kernel_volume;
    let answer = method_b(&needs, Quantity::ReleaseFraction, m0, &radii, &d, &times_s);
    let total = if p.pin.source.is_zero() {
        None
    } else {
        Some(method_b(
            &needs,
            Quantity::ReleaseFractionGenerated,
            m0,
            &radii,
            &d,
            &times_s,
        ))
    };
    let mut out = json!({
        "schema": SCHEMA_STAGE,
        "status": "ok",
        "label": LABEL_BTCS,
        "engine": engine(),
        "stage": stage_label(p.def),
        "name": "Method B: Sturm-Liouville",
        "assumptions": "kernel-step initial state, absorbing outer surface, one temperature for the whole run with the diffusivity of each layer evaluated at it; the source is uniform in the kernel and piecewise constant in time; one decay constant for all layers; the initial inventory is the exact kernel inventory C_init times the kernel volume",
    });
    let obj = out.as_object_mut().expect("an object");
    if let Value::Object(fields) = method_b_json(&answer, &times_s) {
        obj.extend(fields);
    }
    if let Some(MethodB::Available(a)) = &total {
        obj.insert(
            "release_fraction_of_total".into(),
            Value::Array(
                a.values
                    .iter()
                    .map(|v| v.map_or(Value::Null, finite_or_null))
                    .collect(),
            ),
        );
        obj.insert("withheld_reason_codes_of_total".into(), json!(a.withheld));
    }
    Ok(out)
}

/// The answer of the one Method B function as the reply of the page reads it: applicability, the
/// reason, the values at `times_s` with `null` where the drawing mask withholds a value (and the
/// reason code of that value), and the quantities of the mask. A withheld value is never zero.
pub(crate) fn method_b_json(answer: &MethodB, times_s: &[f64]) -> Value {
    let legend: Map<String, Value> = WITHHELD_LEGEND
        .iter()
        .map(|(c, t)| ((*c).to_string(), json!(t)))
        .collect();
    let mut obj = json!({
        "name": "Method B: Sturm-Liouville",
        "times_s": times_s,
        "times_h": times_s.iter().map(|t| t / 3600.0).collect::<Vec<f64>>(),
        "withheld_reason_legend": legend,
        "units": {
            "times_s": "s", "times_h": "h",
            "release_fraction": "1 (released / initial inventory M0), null where the drawing mask withholds the value",
            "release_fraction_of_total": "1 (released / (M0 + generated)), null where withheld",
            "accuracy_floor": "1 (absolute)", "valid_from_s": "s", "valid_from_h": "h",
            "alpha_max_per_s": "1/s",
            "modes": "1 (count of eigenmodes of the series)",
            "missing_weight": "1 (one minus the sum of the mode weights, dimensionless)",
            "applicable": "boolean (true where Method B applies to this configuration)",
            "drawn": "boolean per time, true where the drawing mask accepts the value",
            "drawn_count": "1 (number of times at which the value is drawn)",
            "withheld_reason_codes": "code per time (text of each code in withheld_reason_legend), null where the value is drawn",
            "withheld_reason_codes_of_total": "code per time for release_fraction_of_total, null where drawn",
        },
    });
    let m = obj.as_object_mut().expect("an object");
    match answer {
        MethodB::NotAvailable(n) => {
            m.insert("applicable".into(), json!(false));
            m.insert("extension".into(), Value::Null);
            m.insert("reason_code".into(), json!(n.code));
            m.insert("reason".into(), json!(n.reason));
            m.insert("release_fraction".into(), Value::Null);
        }
        MethodB::Available(a) => {
            m.insert("applicable".into(), json!(true));
            m.insert("extension".into(), json!(a.extension.map(|e| e.formulas())));
            m.insert("reason_code".into(), Value::Null);
            m.insert("reason".into(), Value::Null);
            m.insert(
                "release_fraction".into(),
                Value::Array(
                    a.values
                        .iter()
                        .map(|v| v.map_or(Value::Null, finite_or_null))
                        .collect(),
                ),
            );
            m.insert("withheld_reason_codes".into(), json!(a.withheld));
            m.insert(
                "drawn".into(),
                Value::Array(a.values.iter().map(|v| json!(v.is_some())).collect()),
            );
            m.insert(
                "drawn_count".into(),
                json!(a.values.iter().filter(|v| v.is_some()).count()),
            );
            m.insert("accuracy_floor".into(), json!(a.accuracy_floor));
            m.insert("valid_from_s".into(), json!(a.valid_from_s));
            m.insert("valid_from_h".into(), json!(a.valid_from_s / 3600.0));
            m.insert("modes".into(), json!(a.modes));
            m.insert("alpha_max_per_s".into(), json!(a.alpha_max_per_s));
            m.insert("missing_weight".into(), json!(a.missing_weight));
        }
    }
    obj
}

/// The analytical method (Method B) for the stage of a request: applicability, reason, values
/// at the run's output times (or `times_s`) and the drawing mask. A value the mask withholds is
/// `null` with its reason code, never zero.
pub fn analytical(request_json: &str) -> String {
    match analytical_value(request_json) {
        Ok(v) => v.to_string(),
        Err(e) => rejected(e).to_string(),
    }
}

// ---------------------------------------------------------------------------------------
// btcs_system
// ---------------------------------------------------------------------------------------

fn row_indices(spec: Option<&Value>, grid: &Grid) -> Result<Vec<usize>, Vec<String>> {
    let n = grid.n; // unknowns are rows 0 ..= N-1
    let default: Vec<Value> = vec![
        json!("first"),
        json!(1),
        json!("interface:1"),
        json!("interface:2"),
        json!("interface:3"),
        json!("interface:4"),
        json!("last"),
    ];
    let list = match spec {
        None | Some(Value::Null) => default,
        Some(Value::Array(a)) => a.clone(),
        Some(_) => return Err(vec!["rows: must be a list".into()]),
    };
    if list.len() > MAX_SYSTEM_ROWS {
        return Err(vec![format!(
            "{} rows were requested, above the limit of {MAX_SYSTEM_ROWS} rows",
            list.len()
        )]);
    }
    let mut out = Vec::new();
    for item in &list {
        let idx = match item {
            Value::String(s) if s == "first" => 0,
            Value::String(s) if s == "last" => n - 1,
            Value::String(s) if s.starts_with("interface:") => {
                let k: usize = s["interface:".len()..]
                    .parse()
                    .map_err(|_| vec![format!("rows: `{s}` is not interface:1 to interface:4")])?;
                if !(1..=4).contains(&k) {
                    return Err(vec![format!(
                        "rows: `{s}` is not interface:1 to interface:4"
                    )]);
                }
                grid.interface_nodes[k - 1]
            }
            Value::Number(x) => x.as_u64().ok_or_else(|| {
                vec![format!(
                    "rows: {x} is not a node index (a whole number from 0)"
                )]
            })? as usize,
            other => {
                return Err(vec![format!(
                    "rows: {other} is not first, last, interface:k or a node index"
                )])
            }
        };
        if idx >= n {
            return Err(vec![format!(
                "rows: node {idx} is not a row of the system; the rows are 0 to {} (the outer node {n} is fixed at zero)",
                n - 1
            )]);
        }
        if !out.contains(&idx) {
            out.push(idx);
        }
    }
    Ok(out)
}

fn class_name(c: &NodeClass) -> (&'static str, usize, usize) {
    match c {
        NodeClass::Centre => ("centre", 0, 0),
        NodeClass::Interior(r) => ("interior", r.index(), r.index()),
        NodeClass::Interface { inner, outer } => ("interface", inner.index(), outer.index()),
        NodeClass::Outer => ("outer", 4, 4),
    }
}

fn part(symbol: &str, label: &str, value: Option<f64>, unit: &str) -> Value {
    json!({"symbol_latex": symbol, "label": label, "value": value, "unit": unit})
}

fn btcs_system_value(raw: &str) -> Result<Value, Vec<String>> {
    let (p, extra) = prepare(raw, &["step", "rows"])?;
    if p.plan.intervals > MAX_SYSTEM_INTERVALS {
        return Err(vec![format!(
            "The system is shown for a small grid: the total number of grid intervals is {}, above the limit of {MAX_SYSTEM_INTERVALS} grid intervals",
            p.plan.intervals
        )]);
    }
    let step = match extra.get("step") {
        None | Some(Value::Null) => 1usize,
        Some(v) => v
            .as_u64()
            .map(|s| s as usize)
            .ok_or_else(|| vec!["step: must be a whole number of at least 1".to_string()])?,
    };
    if step < 1 || step > p.input.steps {
        return Err(vec![format!(
            "step is {step}; it must be a whole number from 1 to {} (the number of time steps)",
            p.input.steps
        )]);
    }
    let grid = Grid::build(&p.input).map_err(|e| vec![format!("{e:#}")])?;
    let rows = row_indices(extra.get("rows"), &grid)?;
    let dt = p.input.dt;
    let n = grid.n;
    let kappa = p.pin.decay_constant_per_s * dt;
    let d = d_used_in_step(&p.pin, step).map_err(|e| vec![e])?;
    let gbar = source_in_step(&p.pin, step);
    let nodes = source_nodes(&grid, gbar);
    // The state at the end of the step before.
    let c_old: Vec<f64> = if step == 1 {
        initial_vector(&grid, p.input.c_init).0
    } else {
        let t_prev = (step - 1) as f64 * dt;
        let run = run_physics(&p.pin, &[t_prev])
            .map_err(|e| vec![format!("the solver stopped: {e:#}")])?;
        run.snapshots.first().map(|s| s.1.clone()).ok_or_else(|| {
            vec!["internal check failed: no profile at the previous step".to_string()]
        })?
    };
    let mut full = Stepper::new_with_decay(&grid, d, dt, kappa)
        .map_err(|e| vec![format!("the matrix cannot be assembled: {e:#}")])?;
    let base = Stepper::new(&grid, d, dt)
        .map_err(|e| vec![format!("the matrix cannot be assembled: {e:#}")])?;
    let mut c_new = vec![0.0; n + 1];
    full.step_with_source(&c_old, &nodes.g, &mut c_new)
        .map_err(|e| vec![format!("the step cannot be solved: {e:#}")])?;
    let a = &full.assembly().matrix;
    let a0 = &base.assembly().matrix;
    let fo = full.assembly().coeff.fo;
    let t_a = (step - 1) as f64 * dt;
    let t_b = step as f64 * dt;
    let t_mid = ((step - 1) as f64 + 0.5) * dt;
    let temperature_k = match &p.pin.diffusivity {
        DiffusivityModel::Arrhenius { temperature_k, .. } => Some(temperature_k.value_at(t_mid)),
        DiffusivityModel::Constant(_) => None,
    };
    let mut out_rows = Vec::new();
    for &i in &rows {
        let (class, inner, outer) = class_name(&grid.node_class(i));
        let g_i = nodes.g[i];
        let source_term = dt * g_i;
        let rhs = c_old[i] + dt * g_i;
        let lower = if i == 0 { None } else { Some(a.lower[i]) };
        let (symbolic, equations, parts_diag): (Value, Vec<&str>, Vec<Value>) = if i == 0 {
            (
                json!({
                    "lower": null,
                    "diag": "B_0 = 1 + 6\\,\\mathrm{Fo}_{\\mathrm{kernel}} + \\kappa",
                    "upper": "U_0 = -6\\,\\mathrm{Fo}_{\\mathrm{kernel}}",
                    "rhs": "b_0 = C_0^{j-1} + \\Delta t\\, g_0",
                }),
                vec!["F12", "F37", "F38"],
                vec![
                    part(
                        "1 + 6\\,\\mathrm{Fo}_{\\mathrm{kernel}}",
                        "diffusion part of the diagonal (centre row)",
                        Some(a0.diag[0]),
                        "1",
                    ),
                    part(
                        "\\kappa = \\lambda\\,\\Delta t",
                        "decay number added to every diagonal entry",
                        Some(kappa),
                        "1",
                    ),
                ],
            )
        } else {
            (
                json!({
                    "lower": if i == 1 { "L_1 = 0" } else { "L_i = -\\Delta t\\, K_{i-1/2} / V_i" },
                    "diag": if i == n - 1 { "B_{N-1} = 1 - L_{N-1} - U_{N-1} + \\kappa  (U_{N-1} enters here and is dropped from the matrix)" } else { "B_i = 1 - L_i - U_i + \\kappa" },
                    "upper": if i == n - 1 { "U_{N-1} = 0 (dropped; C_N = 0)" } else { "U_i = -\\Delta t\\, K_{i+1/2} / V_i" },
                    "rhs": "b_i = C_i^{j-1} + \\Delta t\\, g_i",
                }),
                if i == 1 {
                    vec!["F11", "F13", "F37", "F38"]
                } else if i == n - 1 {
                    vec!["F11", "F17", "F18", "F37", "F38"]
                } else if class == "interface" {
                    vec!["F11", "F16", "F37", "F38"]
                } else {
                    vec!["F11", "F37", "F38"]
                },
                vec![
                    part(
                        "1 - L_i - U_i",
                        "diffusion part of the diagonal",
                        Some(a0.diag[i]),
                        "1",
                    ),
                    part(
                        "\\kappa = \\lambda\\,\\Delta t",
                        "decay number added to every diagonal entry",
                        Some(kappa),
                        "1",
                    ),
                ],
            )
        };
        out_rows.push(json!({
            "row": i,
            "node": i,
            "class": class,
            "layer_inner": inner,
            "layer_outer": outer,
            "radius_m": grid.r[i],
            "lower": lower,
            "diag": a.diag[i],
            "upper": if i == n - 1 { Value::Null } else { json!(a.upper[i]) },
            "upper_dropped_into_diagonal": i == n - 1,
            "rhs_mol_m3": rhs,
            "solution_c_new_mol_m3": c_new[i],
            "terms": {
                "diag_diffusion": a0.diag[i],
                "decay_on_diagonal": kappa,
                "c_previous_step_mol_m3": c_old[i],
                "source_in_rhs_mol_m3": source_term,
                "source_density_mol_m3_s": g_i,
                "d_used_inner_layer_m2_s": d[inner],
                "d_used_outer_layer_m2_s": d[outer],
            },
            "parts": {
                "diag": parts_diag,
                "rhs": [
                    part("C_i^{j-1}", "concentration at the end of the previous step", Some(c_old[i]), "mol/m^3"),
                    part("\\Delta t\\, g_i", "source of the step times the step length (F38)", Some(source_term), "mol/m^3"),
                ],
            },
            "symbolic": symbolic,
            "equations": equations,
        }));
    }
    let head: Vec<f64> = nodes.g.iter().take(5).copied().collect();
    Ok(json!({
        "schema": SCHEMA_STAGE,
        "status": "ok",
        "label": LABEL_BTCS,
        "engine": engine(),
        "solver": solver_scope(&p),
        "stage": stage_label(p.def),
        "mechanisms": p.def.mechanisms,
        "step": {
            "index": step,
            "steps_total": p.input.steps,
            "t_start_s": t_a,
            "t_end_s": t_b,
            "t_mid_s": t_mid,
            "dt_s": dt,
            "temperature_mid_step_k": temperature_k,
            "temperature_mid_step_c": temperature_k.map(|t| t - CELSIUS_OFFSET_K),
        },
        "grid": {
            "intervals_total": n,
            "unknowns": n,
            "intervals_per_layer": p.input.n_intervals,
            "interface_nodes": grid.interface_nodes,
            "node_radii_m": grid.r,
        },
        "diffusivity_used_m2_s": d,
        "fourier_numbers": fo,
        "decay_number_kappa": kappa,
        "rows": out_rows,
        "source_vector_head_mol_m3_s": head,
        "system": "A C^j = b with A tridiagonal of order N (the unknowns C_0 .. C_{N-1}; C_N = 0), (F19); the rows are shown as the solver assembled and used them",
        "units": {
            "lower, diag, upper": "1 (dimensionless matrix entries)",
            "rhs_mol_m3": "mol/m^3",
            "solution_c_new_mol_m3": "mol/m^3",
            "source_density_mol_m3_s": "mol/(m^3 s)",
            "source_in_rhs_mol_m3": "mol/m^3 (the source density times the step length)",
            "radius_m": "m",
            "diffusivity_used_m2_s": "m^2/s (per layer: kernel, buffer, IPyC, SiC, OPyC)",
            "index": "1 (the number of the step shown)", "steps_total": "1 (steps of the run)",
            "t_start_s": "s", "t_mid_s": "s (the temperature of the step is taken here)", "t_end_s": "s", "dt_s": "s",
            "temperature_mid_step_k": "K (null at a constant diffusivity)", "temperature_mid_step_c": "degC (null at a constant diffusivity)",
            "intervals_total": "1", "unknowns": "1 (order of the matrix)", "intervals_per_layer": "1 (per layer)",
            "interface_nodes": "1 (node index of each interface)", "node_radii_m": "m",
            "fourier_numbers": "1 (D dt / h^2 per layer)", "decay_number_kappa": "1 (decay constant times the step length)",
            "source_vector_head_mol_m3_s": "mol/(m^3 s) (the first source densities)",
        },
        "messages": Vec::<String>::new(),
    }))
}

/// The assembled BTCS system of one step for a small grid: matrix rows and right-hand side as the
/// solver assembles them, with the terms each mechanism adds and the labelled symbolic parts.
pub fn btcs_system(request_json: &str) -> String {
    match btcs_system_value(request_json) {
        Ok(v) => v.to_string(),
        Err(e) => rejected(e).to_string(),
    }
}
