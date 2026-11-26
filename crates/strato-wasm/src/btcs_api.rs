//! BTCS interface of the wrapper: method-neutral `validate`, `run` and `compare`.
//!
//! Every number returned here is computed by the `strato` library (`numerical::btcs` and its
//! modules; the independent series of `analytical::sturm_liouville`) or is bookkeeping over
//! its data (sums of library values, divisions of one library value by another, selection of
//! output rows). The frontend does no scientific arithmetic. No physics is written here.
//!
//! Scope of the solver behind this interface (Document 2 of `docs/mathematics/btcs/`):
//! constant diffusivity in each of five layers, no source, no decay, constant temperature
//! (recorded only), kernel-only start, absorbing outer surface. The reply states this.
//!
//! Inputs use the configuration of the library; the number of grid intervals of a layer is
//! given as `intervals` (the field `cells` of the library configuration is accepted as the
//! same number). No file is read or written, and no panic crosses the boundary: every rule is
//! checked against the wrapper limits before the library is called.

use super::{
    check_schema_value, config_from_value, engine_value, placeholder_errors, series_mode_bound,
    BUILD_COMMIT, STRATO_VERSION,
};
use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use strato::{
    analytical::sturm_liouville::ModalRelease,
    config::Config,
    numerical::{
        btcs::{self, Limits, Stepper},
        btcs_report::{interface_flux_report, layer_shares, ring_edges},
        coefficients::Coefficients,
        diagnostics::{check_matrix_structure, check_series, check_step_properties},
        grid::{Grid, NodeClass, ValidatedInput},
        inventory::inventory,
    },
    postprocessing::{
        comparison::{layers_from_config, MAX_MODES},
        export::{comparison_csv, profile_csv, run_json, timeseries_csv, Provenance},
    },
};

/// Schema version of every request and response object of the BTCS interface.
pub const SCHEMA_BTCS: u32 = 2;
/// Result label of every value the BTCS interface computes (decision 10 of the coordinator:
/// the BTCS solver is version 2.0.0 of the crate from the commit that retires finite volume).
pub const LABEL_BTCS: &str = "COMPUTED v2.0";
/// Mechanism ids (`mechanism_id` of data/research/mechanisms.csv) this interface solves.
pub const CAPABILITIES_BTCS: &[&str] = &["diffusion_btcs_layered"];
/// Identifier of the solver behind this interface.
pub const SOLVER_ID: &str = "btcs_five_layer";

/// Wrapper limit: total grid intervals over the five layers (the library's `N`). Not a
/// physical parameter. Native timing is measured by `examples/limits_timing.rs`; no browser
/// measurement of the page was made.
pub const MAX_TOTAL_INTERVALS: usize = 2_000;
/// Wrapper limit: `ceil(end_time_s / dt_s)`, the number of Backward Euler steps.
pub const MAX_STEPS_BTCS: usize = 20_000;
/// Wrapper limit: stored profile values, (number of returned profiles) x (N + 1).
pub const MAX_PROFILE_VALUES_BTCS: usize = 1_000_000;
/// Wrapper limit: requested profile times in one request.
pub const MAX_PROFILE_TIMES: usize = 1_000;

const LIMITS: Limits = Limits {
    max_intervals_total: MAX_TOTAL_INTERVALS,
    max_steps: MAX_STEPS_BTCS,
};

pub fn limits_value() -> Value {
    json!({
        "max_total_intervals": MAX_TOTAL_INTERVALS,
        "min_intervals_per_layer": 1,
        "max_steps": MAX_STEPS_BTCS,
        "max_profile_values": MAX_PROFILE_VALUES_BTCS,
        "max_profile_times": MAX_PROFILE_TIMES,
        "max_series_modes": super::MAX_SERIES_MODES,
    })
}

pub fn solver_value() -> Value {
    json!({
        "id": SOLVER_ID,
        "method": "BTCS five-layer spherical finite difference (nodal)",
        "capability": CAPABILITIES_BTCS[0],
        "version": STRATO_VERSION,
        "commit": BUILD_COMMIT,
        "label": LABEL_BTCS,
        "formulation": "docs/mathematics/btcs/02_final_formulation.md",
        "scope": "constant diffusivity in each layer, no source, no decay, constant temperature (recorded, not used), kernel-only start, absorbing outer surface",
        "nature": "numerical solution of the stated model; not experimental validation",
    })
}

pub fn build_info_value() -> Value {
    json!({
        "solver": solver_value(),
        "schema": SCHEMA_BTCS,
        "functions": ["validate", "run", "compare"],
        "export_functions": ["export_run"],
        "export_formats": EXPORT_FORMATS,
        "capabilities": CAPABILITIES_BTCS,
        "label": LABEL_BTCS,
        "limits": limits_value(),
    })
}

fn engine() -> Value {
    let mut e = engine_value();
    if let Value::Object(m) = &mut e {
        m.insert("solver".into(), json!(SOLVER_ID));
    }
    e
}

fn rejected(errors: Vec<String>) -> Value {
    json!({"schema": SCHEMA_BTCS, "status": "rejected", "errors": errors, "engine": engine()})
}

// ---------------------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunRequest {
    schema: u32,
    config: Value,
    #[serde(default)]
    mechanisms: Option<Vec<String>>,
    #[serde(default)]
    profile_times_s: Option<Vec<f64>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CompareRequest {
    schema: u32,
    baseline: Value,
    modified: Value,
}

fn check_mechanisms(requested: &Option<Vec<String>>) -> Result<(), Vec<String>> {
    let unsupported: Vec<String> = requested
        .iter()
        .flatten()
        .filter(|m| !CAPABILITIES_BTCS.contains(&m.as_str()))
        .map(|m| {
            format!(
                "mechanism {m} is not supported by this solver build (capabilities: {})",
                CAPABILITIES_BTCS.join(", ")
            )
        })
        .collect();
    if unsupported.is_empty() {
        Ok(())
    } else {
        Err(unsupported)
    }
}

/// Renames `regions[*].intervals` to the library's `cells` (the same number: the count of
/// grid intervals of the layer). Both names in one region is an error.
fn normalise_config(mut v: Value) -> Result<Value, Vec<String>> {
    if let Some(Value::Array(regions)) = v.get_mut("regions") {
        for (k, region) in regions.iter_mut().enumerate() {
            if let Value::Object(map) = region {
                if map.contains_key("intervals") {
                    if map.contains_key("cells") {
                        return Err(vec![format!(
                            "config: region {k} has both `intervals` and `cells`; give one"
                        )]);
                    }
                    if let Some(x) = map.remove("intervals") {
                        map.insert("cells".into(), x);
                    }
                }
            }
        }
    }
    Ok(v)
}

pub(crate) fn config_from_request_value(v: Value) -> Result<Config, Vec<String>> {
    config_from_value(normalise_config(v)?)
}

/// The work a validated configuration asks for.
#[derive(Clone, Copy, Debug)]
pub struct Plan {
    pub intervals: usize,
    pub nodes: usize,
    pub steps: usize,
    pub dt_s: f64,
    pub output_rows: usize,
}

/// A value as the page shows it: zero as `0`, a whole number below 10^15 in full, anything else
/// in scientific notation.
fn value_text(x: f64) -> String {
    if x == 0.0 {
        "0".into()
    } else if x.is_finite() && x.fract() == 0.0 && x.abs() < 1e15 {
        format!("{x}")
    } else {
        format!("{x:e}")
    }
}

/// Every violated input rule of `Config::validate` and `btcs::validate_input`, each as one
/// sentence with the full name of the quantity, its unit, the value given and the rule.
fn describe_input_problems(cfg: &Config) -> Vec<String> {
    let mut out = Vec::new();
    if cfg.regions.len() != 5 {
        out.push(format!(
            "Exactly five layers are required, but {} were given",
            cfg.regions.len()
        ));
        return out;
    }
    if cfg.species.trim().is_empty() {
        out.push("The name of the species must not be empty".into());
    }
    if !(cfg.temperature_k.is_finite() && cfg.temperature_k > 0.0) {
        out.push(format!(
            "Temperature (K) is {}; it must be a finite number larger than 0 K",
            value_text(cfg.temperature_k)
        ));
    }
    let mut last = 0.0;
    for r in &cfg.regions {
        let name = &r.name;
        if r.outer_radius_m.is_finite() && r.outer_radius_m > last {
            last = r.outer_radius_m;
        } else {
            out.push(format!(
                "Outer radius of the {name} layer (m) is {}; it must be a finite number larger than {} m, the outer radius of the layer inside it (0 m at the centre)",
                value_text(r.outer_radius_m),
                value_text(last)
            ));
        }
        if !(r.diffusivity_m2_s.is_finite() && r.diffusivity_m2_s > 0.0) {
            out.push(format!(
                "Diffusivity of the {name} layer (m²/s) is {}; it must be a finite number larger than 0 m²/s",
                value_text(r.diffusivity_m2_s)
            ));
        }
        if r.cells == 0 {
            out.push(format!(
                "Number of grid intervals in the {name} layer (dimensionless) is 0; it must be a whole number of at least 1"
            ));
        }
    }
    let t = &cfg.time;
    if !(t.dt_s.is_finite() && t.dt_s > 0.0) {
        out.push(format!(
            "Time step (s) is {}; it must be a finite number larger than 0 s",
            value_text(t.dt_s)
        ));
    }
    if !(t.end_time_s.is_finite() && t.end_time_s > 0.0) {
        out.push(format!(
            "Simulation duration (s) is {}; it must be a finite number larger than 0 s",
            value_text(t.end_time_s)
        ));
    }
    if t.output_every_steps == 0 {
        out.push("Number of time steps between two output rows (dimensionless) is 0; it must be a whole number of at least 1".into());
    }
    let c0 = cfg.initial.kernel_concentration_mol_m3;
    if !(c0.is_finite() && c0 > 0.0) {
        out.push(format!(
            "Initial concentration in the kernel (mol/m³) is {}; it must be a finite number larger than 0 mol/m³",
            value_text(c0)
        ));
    }
    out
}

/// `Config::validate`, the library's BTCS input rules (PS-01) and the wrapper limits, in this
/// order, before any other library call.
pub fn check_config(cfg: &Config) -> Result<(ValidatedInput, Plan), Vec<String>> {
    // The input rules are stated first, in words that name each quantity in full with its unit
    // and its value, because the page shows these messages as they are. They cover the rules of
    // `Config::validate` and of `btcs::validate_input`, which stay as the final guard.
    let problems = describe_input_problems(cfg);
    if !problems.is_empty() {
        return Err(problems);
    }
    cfg.validate().map_err(|e| vec![format!("{e:#}")])?;
    // validate() guarantees at least one interval per layer; saturate so that a huge value
    // cannot overflow the sum the library takes.
    let total = cfg
        .regions
        .iter()
        .fold(0usize, |a, r| a.saturating_add(r.cells));
    if total > MAX_TOTAL_INTERVALS {
        return Err(vec![format!(
            "The total number of grid intervals in all layers is {total}, above the limit of {MAX_TOTAL_INTERVALS} grid intervals"
        )]);
    }
    let ratio = (cfg.time.end_time_s / cfg.time.dt_s).ceil();
    if ratio.is_nan() || ratio > MAX_STEPS_BTCS as f64 {
        return Err(vec![format!(
            "The number of time steps (the simulation duration divided by the time step, rounded up) is {}, above the limit of {MAX_STEPS_BTCS} time steps",
            value_text(ratio)
        )]);
    }
    let input = btcs::validate_input(cfg, &LIMITS).map_err(|e| vec![format!("{e:#}")])?;
    let oe = input.output_every;
    let output_rows = 1 + input.steps / oe + usize::from(!input.steps.is_multiple_of(oe));
    let plan = Plan {
        intervals: total,
        nodes: total + 1,
        steps: input.steps,
        dt_s: input.dt,
        output_rows,
    };
    Ok((input, plan))
}

struct Prepared {
    cfg: Config,
    input: ValidatedInput,
    plan: Plan,
    mechanisms: Vec<String>,
    /// `(step, requested times mapped to it)`, ascending by step; the profiles to return.
    profile_steps: Vec<(usize, Vec<f64>)>,
    requested: bool,
}

/// Steps at which a row is written: `j = 0`, every `output_every`-th step and the last.
pub(crate) fn output_steps(input: &ValidatedInput) -> Vec<usize> {
    let mut steps = vec![0];
    for s in 1..=input.steps {
        if s % input.output_every == 0 || s == input.steps {
            steps.push(s);
        }
    }
    steps
}

/// The step nearest to `t`, clamped to `0 ..= steps` (the library's snapshot rule; a profile
/// is never interpolated in time).
fn nearest_step(t: f64, input: &ValidatedInput) -> usize {
    ((t / input.dt).round().max(0.0) as usize).min(input.steps)
}

fn prepare_value(request: Value) -> Result<Prepared, Vec<String>> {
    let req: RunRequest =
        serde_json::from_value(request).map_err(|e| vec![format!("request: {e}")])?;
    check_schema_value(req.schema, SCHEMA_BTCS)?;
    check_mechanisms(&req.mechanisms)?;
    let cfg = config_from_request_value(req.config)?;
    let (input, plan) = check_config(&cfg)?;
    let (profile_steps, requested) = match req.profile_times_s {
        Some(times) => {
            if times.len() > MAX_PROFILE_TIMES {
                return Err(vec![format!(
                    "{} profile times were requested, above the limit of {MAX_PROFILE_TIMES} profile times",
                    times.len()
                )]);
            }
            if times.iter().any(|t| !t.is_finite() || *t < 0.0) {
                return Err(vec![
                    "Every requested profile time (s) must be a finite number of at least 0 s"
                        .into(),
                ]);
            }
            let mut map: BTreeMap<usize, Vec<f64>> = BTreeMap::new();
            for t in times {
                map.entry(nearest_step(t, &input)).or_default().push(t);
            }
            (map.into_iter().collect::<Vec<_>>(), true)
        }
        None => (
            output_steps(&input)
                .into_iter()
                .map(|s| (s, Vec::new()))
                .collect(),
            false,
        ),
    };
    let values = profile_steps.len().saturating_mul(plan.nodes);
    if values > MAX_PROFILE_VALUES_BTCS {
        return Err(vec![format!(
            "{} concentration profiles of {} grid nodes each give {values} stored concentration values, above the limit of {MAX_PROFILE_VALUES_BTCS} values; {}",
            profile_steps.len(),
            plan.nodes,
            if requested {
                "request fewer profile times"
            } else {
                "raise the number of time steps between two output rows or request profile times"
            }
        )]);
    }
    let mechanisms = req
        .mechanisms
        .unwrap_or_else(|| CAPABILITIES_BTCS.iter().map(|s| s.to_string()).collect());
    Ok(Prepared {
        cfg,
        input,
        plan,
        mechanisms,
        profile_steps,
        requested,
    })
}

fn prepare(raw: &str) -> Result<Prepared, Vec<String>> {
    let errors = placeholder_errors(raw);
    if !errors.is_empty() {
        return Err(errors);
    }
    let v: Value = serde_json::from_str(raw).map_err(|e| vec![format!("request: {e}")])?;
    prepare_value(v)
}

// ---------------------------------------------------------------------------------------
// Echo and grid
// ---------------------------------------------------------------------------------------

fn echo(cfg: &Config) -> Value {
    json!({
        "case_name": cfg.case_name,
        "species": cfg.species,
        "temperature_k": cfg.temperature_k,
        "temperature_used_by_solver": false,
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
    })
}

/// The grid as Rust built it. `ring_edges_m` is presentation geometry: the radial extent of
/// the colour band of each node (midpoints between neighbouring nodes, `0` at the centre and
/// the outer radius at the last node); it is not a property of the discretisation.
pub(crate) fn grid_value(grid: &Grid, cfg: &Config, input: &ValidatedInput) -> Value {
    let n = grid.n;
    let mut class = Vec::with_capacity(n + 1);
    let mut inner = Vec::with_capacity(n + 1);
    let mut outer = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let (c, a, b) = match grid.node_class(i) {
            NodeClass::Centre => ("centre", 0, 0),
            NodeClass::Interior(r) => ("interior", r.index(), r.index()),
            NodeClass::Interface { inner, outer } => ("interface", inner.index(), outer.index()),
            NodeClass::Outer => ("outer", 4, 4),
        };
        class.push(c);
        inner.push(a);
        outer.push(b);
    }
    let edges = ring_edges(grid);
    json!({
        "intervals_total": n,
        "nodes": n + 1,
        "intervals_per_layer": input.n_intervals,
        "node_radii_m": grid.r,
        "node_class": class,
        "node_layer_inner": inner,
        "node_layer_outer": outer,
        "layer_of_interval": grid.region_of_interval.iter().map(|r| r.index()).collect::<Vec<usize>>(),
        "interface_nodes": grid.interface_nodes,
        "interface_radii_m": grid.interface_nodes.iter().map(|&i| grid.r[i]).collect::<Vec<f64>>(),
        "spacing_per_layer_m": grid.dr_region,
        "layer_names": cfg.regions.iter().map(|r| r.name.as_str()).collect::<Vec<&str>>(),
        "ring_edges_m": edges,
    })
}

// ---------------------------------------------------------------------------------------
// Run
// ---------------------------------------------------------------------------------------

pub(crate) fn finite_or_null(x: f64) -> Value {
    if x.is_finite() {
        json!(x)
    } else {
        Value::Null
    }
}

/// Interface fluxes of one step as a JSON object, from the library's report; the second value is
/// the worst relative difference between the two expressions.
fn interface_flux_value(
    grid: &Grid,
    coeff: &Coefficients,
    c_old: &[f64],
    c_new: &[f64],
    dt: f64,
) -> (Value, f64) {
    let r = interface_flux_report(grid, coeff, c_old, c_new, dt);
    (
        json!({
            "interface_nodes": r.nodes,
            "radii_m": r.radii_m,
            "outward_from_inner_expression_mol_s": r.from_inner,
            "outward_from_outer_expression_mol_s": r.from_outer,
            "difference_mol_s": r.difference,
            "difference_bound_mol_s": r.bound,
            "within_bound": r.within_bound,
        }),
        r.worst_relative,
    )
}

/// Runs a prepared request and builds the reply object.
fn run_prepared(p: &Prepared) -> Result<Value, Vec<String>> {
    let input = &p.input;
    let cfg = &p.cfg;
    let dt = input.dt;
    let mut messages: Vec<String> = Vec::new();
    if dt != cfg.time.dt_s {
        messages.push(format!(
            "time step used: {:e} s ({} steps of t_end / ceil(t_end / dt)); requested {:e} s",
            dt, input.steps, cfg.time.dt_s
        ));
    }

    // Snapshots: the profiles asked for, the step before each (for the interface flux) and the
    // last two steps (for the step diagnostics).
    let mut want: BTreeSet<usize> = BTreeSet::new();
    for (s, _) in &p.profile_steps {
        want.insert(*s);
        if *s >= 1 {
            want.insert(*s - 1);
        }
    }
    want.insert(input.steps);
    want.insert(input.steps - 1);
    let times: Vec<f64> = want.iter().map(|&s| s as f64 * dt).collect();
    let result = btcs::run_input(input, &times).map_err(|e| vec![format!("{e:#}")])?;
    let grid = &result.grid;
    let m0 = result.meta.initial_inventory;
    let snap: BTreeMap<usize, &Vec<f64>> = result
        .snapshots
        .iter()
        .map(|(t, c)| (((t / dt).round()) as usize, c))
        .collect();
    let stepper = Stepper::new(grid, input.d, dt).map_err(|e| vec![format!("{e:#}")])?;
    let coeff = stepper.assembly().coeff.clone();

    let row_steps = output_steps(input);
    debug_assert_eq!(row_steps.len(), result.series.len());
    // The rounding bound of the balance residual is the one the solver derived during this very run
    // (`balance_residual_bound` of each output row, in mol): the page, the exports and the command
    // line show one value. It is kept in mol and relative to the initial inventory, indexed by
    // step; only the steps of the output rows are filled.
    let mut bounds_mol = vec![f64::NAN; input.steps + 1];
    let mut bounds = vec![f64::NAN; input.steps + 1];
    for (&s, row) in row_steps.iter().zip(&result.series) {
        bounds_mol[s] = row.1.balance_residual_bound;
        bounds[s] = row.1.balance_residual_bound / m0;
    }

    // Profiles.
    let mut profiles = Vec::with_capacity(p.profile_steps.len());
    let mut worst_layer_sum = 0.0f64;
    let mut worst_flux_rel = 0.0f64;
    let mut flux_within = true;
    let mut flux_count = 0usize;
    for (s, requested) in &p.profile_steps {
        let Some(c) = snap.get(s) else {
            return Err(vec!["internal: a requested profile was not recorded".into()]);
        };
        let inv = inventory(grid, c);
        let shares = layer_shares(grid, c, inv);
        let share = match &shares.share {
            Some(v) => {
                worst_layer_sum = worst_layer_sum.max(shares.sum_relative_error);
                json!(v)
            }
            None => {
                messages.push(format!(
                    "step {s}: inventory is not positive, layer shares not given"
                ));
                Value::Null
            }
        };
        let layers = shares.layer_inventory_mol;
        let flux = if *s >= 1 {
            match snap.get(&(*s - 1)) {
                Some(old) => {
                    let (v, rel) = interface_flux_value(grid, &coeff, old, c, dt);
                    worst_flux_rel = worst_flux_rel.max(rel);
                    flux_within &= v["within_bound"].as_bool().unwrap_or(false);
                    flux_count += 1;
                    v
                }
                None => Value::Null,
            }
        } else {
            Value::Null
        };
        profiles.push(json!({
            "step": s,
            "t_s": *s as f64 * dt,
            "requested_times_s": requested,
            "c_mol_per_m3": c,
            "inventory_mol": inv,
            "layer_inventory_mol": layers,
            "layer_share": share,
            "interface_flux": flux,
        }));
    }

    // Columns.
    let rows = &result.series;
    let col = |f: &dyn Fn(&strato::numerical::diagnostics::StepDiagnostics) -> f64| -> Vec<f64> {
        rows.iter().map(|(_, d)| f(d)).collect()
    };
    let t_col: Vec<f64> = rows.iter().map(|(t, _)| *t).collect();
    let rate_col: Vec<Value> = rows
        .iter()
        .map(|(_, d)| d.release_rate.map_or(Value::Null, |x| json!(x)))
        .collect();
    let residual_col = col(&|d| d.balance_residual);
    let bound_col: Vec<f64> = row_steps.iter().map(|&s| bounds_mol[s]).collect();
    let max_abs_residual = residual_col.iter().fold(0.0f64, |a, x| a.max(x.abs()));
    let worst_over_bound = rows
        .iter()
        .zip(&row_steps)
        .map(|((_, d), &s)| d.balance_residual.abs() / m0 / bounds[s])
        .fold(0.0f64, f64::max);
    let within_bound = rows
        .iter()
        .zip(&row_steps)
        .all(|((_, d), &s)| d.balance_residual.abs() / m0 <= bounds[s]);
    let (last_t, last) = rows
        .last()
        .map(|(t, d)| (*t, *d))
        .ok_or_else(|| vec!["internal: no output row".to_string()])?;

    // Series (independent Sturm-Liouville solution), where its assumptions hold: they are the
    // assumptions of this solver, so it applies to every run of this interface.
    let positive: Vec<f64> = t_col.iter().copied().filter(|t| *t > 0.0).collect();
    let t_min = positive.iter().copied().fold(f64::INFINITY, f64::min);
    let series = {
        let bound = series_mode_bound(cfg, t_min);
        if !(bound.is_finite() && bound <= super::MAX_SERIES_MODES as f64) {
            Err(format!(
                "Method B is not computed: the earliest output time ({t_min:e} s) needs up to {bound:.3e} eigenmodes of the series, above the limit of {} eigenmodes; a larger time between two output rows (the time step times the number of time steps between output rows) needs fewer",
                super::MAX_SERIES_MODES
            ))
        } else {
            ModalRelease::new(&layers_from_config(cfg), 45.0 / t_min, MAX_MODES)
                .map_err(|e| format!("Method B is not available: {e:#}"))
        }
    };
    let (analytical, analytical_cmp) = match series {
        Ok(s) => {
            let valid = s.min_valid_time();
            let resolved_rows: Vec<(f64, f64, f64)> = rows
                .iter()
                .filter(|(t, _)| *t > 0.0 && *t >= valid)
                .map(|(t, d)| (*t, s.release_fraction(*t), d.release_fraction))
                .collect();
            if resolved_rows.is_empty() {
                messages.push(format!(
                    "Method B (the series, capped at {MAX_MODES} eigenmodes) resolves only times of at least {valid:.3e} s, which is later than every output time"
                ));
                (Value::Null, Value::Null)
            } else {
                let floor = s.accuracy_floor();
                let diffs: Vec<f64> = resolved_rows.iter().map(|r| (r.2 - r.1).abs()).collect();
                let max_diff = diffs.iter().copied().fold(0.0f64, f64::max);
                let final_diff = diffs.last().copied().unwrap_or(f64::NAN);
                if max_diff < 3.0 * floor {
                    messages.push(
                        "The differences between Method A and Method B are within three times the accuracy floor of Method B; they are not resolved".into(),
                    );
                }
                (
                    json!({
                        "applicable": true,
                        "name": "analytical (Sturm-Liouville series)",
                        "assumptions": "constant diffusivity per layer, no source, no decay, constant temperature, kernel-only start, absorbing outer surface",
                        "times_s": resolved_rows.iter().map(|r| r.0).collect::<Vec<f64>>(),
                        "release_fraction": resolved_rows.iter().map(|r| r.1).collect::<Vec<f64>>(),
                        "accuracy_floor": floor,
                        "valid_from_s": valid,
                        "modes": s.eigenvalues.len(),
                    }),
                    json!({
                        "modes": s.eigenvalues.len(),
                        "max_abs_difference": max_diff,
                        "final_abs_difference": final_diff,
                        "accuracy_floor": floor,
                        "valid_from_s": valid,
                        "min_flux_ratio": s.min_flux_ratio,
                        "resolved": max_diff >= 3.0 * floor,
                    }),
                )
            }
        }
        Err(e) => {
            messages.push(e);
            (Value::Null, Value::Null)
        }
    };

    // Library diagnostics.
    let matrix = check_matrix_structure(&stepper.assembly().matrix);
    let matrix_value = match &matrix {
        Ok(m) => json!({
            "min_row_margin": m.min_row_margin,
            "max_interior_margin_deviation": m.max_interior_margin_deviation,
            "norm_inf": m.norm_inf,
        }),
        Err(e) => {
            messages.push(format!("matrix structure check failed: {e:#}"));
            Value::Null
        }
    };
    let final_step = match (snap.get(&input.steps), snap.get(&(input.steps - 1))) {
        (Some(new), Some(old)) => {
            let r = check_step_properties(old, new);
            json!({
                "step": input.steps,
                "min_new_mol_m3": r.min_new,
                "max_new_mol_m3": r.max_new,
                "max_old_mol_m3": r.max_old,
                "non_negative": r.non_negative,
                "max_principle_excess_mol_m3": r.max_principle_excess,
            })
        }
        _ => Value::Null,
    };
    let sr = check_series(rows, m0);
    let min_c = sr.min_concentration;

    Ok(json!({
        "schema": SCHEMA_BTCS,
        "status": "ok",
        "label": LABEL_BTCS,
        "engine": engine(),
        "solver": solver_value(),
        "mechanisms": p.mechanisms,
        "inputs": echo(cfg),
        "grid": grid_value(grid, cfg, input),
        "times_s": t_col,
        "steps_of_rows": row_steps,
        "release_fraction": col(&|d| d.release_fraction),
        "release_rate_mol_s": rate_col,
        "inventory_mol": col(&|d| d.inventory),
        "cumulative_release_mol": col(&|d| d.cumulative_release),
        "balance_residual_mol": residual_col,
        "balance_residual_bound_mol": bound_col,
        "min_concentration_mol_m3": col(&|d| d.min_concentration),
        "max_concentration_mol_m3": col(&|d| d.max_concentration),
        "profiles": profiles,
        "profile_times_requested": p.requested,
        "balance": {
            "form": "btcs-F30",
            "statement": "remaining + released + decayed - start - generated = residual (equation F30); the directive writes the same identity with the opposite sign of the residual",
            "source": "docs/mathematics/btcs/02_final_formulation.md, equations F25, F27, F29, F30",
            "generated_mol": 0.0,
            "decayed_mol": 0.0,
            "generated_and_decayed_note": "the solved model has no source and no decay, so both terms are exactly zero by construction; they are not measured",
            "start_inventory_mol": m0,
            "final": {
                "t_s": last_t,
                "remaining_mol": last.inventory,
                "released_mol": last.cumulative_release,
                "residual_mol": last.balance_residual,
                "residual_bound_mol": bounds_mol[input.steps],
            },
            "max_abs_residual_mol": max_abs_residual,
            "max_abs_residual_relative": max_abs_residual / m0,
            "tolerance": {
                "kind": "a-posteriori rounding bound (coordinator decision 6); not a discretisation error",
                "final_bound_relative": bounds[input.steps],
                "max_ratio_residual_to_bound": worst_over_bound,
                "within_bound": within_bound,
            },
        },
        "diagnostics": {
            "intervals_total": p.plan.intervals,
            "nodes": p.plan.nodes,
            "intervals_per_layer": input.n_intervals,
            "steps": input.steps,
            "dt_requested_s": cfg.time.dt_s,
            "dt_used_s": dt,
            "end_time_s": cfg.time.end_time_s,
            "output_rows": rows.len(),
            "profiles_returned": p.profile_steps.len(),
            "kernel_surface_start_fraction": result.meta.theta,
            "fourier_numbers": coeff.fo,
            "matrix": matrix_value,
            "final_step": final_step,
            "series_properties": json!({
                "release_monotone": sr.release_monotone,
                "release_in_range": sr.release_in_range,
                "inventory_non_increasing": sr.inventory_non_increasing,
                "max_inventory_increase_relative": sr.max_inventory_increase_rel,
            }),
            "final_release_fraction": last.release_fraction,
            "max_abs_balance_residual_mol": max_abs_residual,
            "max_abs_balance_residual_relative": max_abs_residual / m0,
            "min_concentration_mol_m3": finite_or_null(min_c),
            "max_concentration_mol_m3": finite_or_null(sr.max_concentration),
            "non_negative": min_c >= 0.0,
            "layer_sum_max_relative_difference": worst_layer_sum,
            "interface_flux": {
                "profiles_with_flux": flux_count,
                "max_relative_difference": worst_flux_rel,
                "all_within_bound": flux_within,
            },
            "analytical_comparison": analytical_cmp,
        },
        "analytical": analytical,
        "messages": messages,
    }))
}

fn run_value(request: Value) -> Result<Value, Vec<String>> {
    let p = prepare_value(request)?;
    run_prepared(&p)
}

pub fn run(request_json: &str) -> String {
    let result = prepare(request_json).and_then(|p| run_prepared(&p));
    match result {
        Ok(v) => v.to_string(),
        Err(errors) => rejected(errors).to_string(),
    }
}

// ---------------------------------------------------------------------------------------
// Export (data download)
// ---------------------------------------------------------------------------------------

/// The export formats of `export_run`, the builders of `postprocessing::export`. The format
/// `comparison_csv` needs `baseline_config` besides `config`.
pub const EXPORT_FORMATS: [&str; 4] = [
    "timeseries_csv",
    "profiles_csv",
    "run_json",
    "comparison_csv",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExportRequest {
    schema: u32,
    config: Value,
    #[serde(default)]
    profile_times_s: Option<Vec<f64>>,
    format: String,
    /// File name recorded in the provenance header (the library keeps the last path component).
    #[serde(default)]
    input_file: Option<String>,
    /// Source row of the parameters (decision 31); without it the header says that none is recorded.
    #[serde(default)]
    parameter_source: Option<String>,
    /// Class of the parameters (decision 31).
    #[serde(default)]
    parameter_class: Option<String>,
    /// The baseline configuration of the format `comparison_csv`; `config` is the current one.
    #[serde(default)]
    baseline_config: Option<Value>,
    /// Source row of the baseline's parameters (comparison export only).
    #[serde(default)]
    baseline_parameter_source: Option<String>,
    /// Class of the baseline's parameters (comparison export only).
    #[serde(default)]
    baseline_parameter_class: Option<String>,
}

fn export_value(request: Value) -> Result<Value, Vec<String>> {
    let req: ExportRequest =
        serde_json::from_value(request).map_err(|e| vec![format!("request: {e}")])?;
    if !EXPORT_FORMATS.contains(&req.format.as_str()) {
        return Err(vec![format!(
            "unknown export format {:?}; expected one of {}",
            req.format,
            EXPORT_FORMATS.join(", ")
        )]);
    }
    let comparison = req.format == "comparison_csv";
    if comparison != req.baseline_config.is_some() {
        return Err(vec![if comparison {
            "The comparison export needs the baseline configuration".to_string()
        } else {
            "A baseline configuration is used only by the comparison export".to_string()
        }]);
    }
    let mut run_request = json!({"schema": req.schema, "config": req.config});
    if comparison {
        // Only the series of both runs is written: no profile is stored.
        run_request["profile_times_s"] = json!([0.0]);
    } else if let Some(times) = req.profile_times_s {
        run_request["profile_times_s"] = json!(times);
    }
    let p = prepare_value(run_request)?;
    let dt = p.input.dt;
    let times: Vec<f64> = p
        .profile_steps
        .iter()
        .map(|(s, _)| *s as f64 * dt)
        .collect();
    let result = btcs::run_input(&p.input, &times).map_err(|e| vec![format!("{e:#}")])?;
    let mut prov = Provenance::new(
        &p.cfg,
        &p.input,
        req.input_file.as_deref().unwrap_or(""),
        BUILD_COMMIT,
    );
    if req.parameter_source.is_some() || req.parameter_class.is_some() {
        let source = req
            .parameter_source
            .unwrap_or_else(|| prov.parameter_source.clone());
        let class = req
            .parameter_class
            .unwrap_or_else(|| prov.parameter_class.clone());
        prov = prov.with_parameter_source(&source, &class);
    }
    let (text, media_type) = match req.format.as_str() {
        "timeseries_csv" => (
            timeseries_csv(&prov, &result.series, result.meta.initial_inventory),
            "text/csv",
        ),
        "profiles_csv" => (
            profile_csv(&prov, &result.grid, &result.snapshots),
            "text/csv",
        ),
        "comparison_csv" => (Ok(String::new()), "text/csv"),
        _ => (run_json(&prov, &p.input, &result), "application/json"),
    };
    let text = if comparison {
        let base = prepare_value(
            json!({"schema": req.schema, "config": req.baseline_config, "profile_times_s": [0.0]}),
        )?;
        let base_result =
            btcs::run_input(&base.input, &[0.0]).map_err(|e| vec![format!("{e:#}")])?;
        let mut base_prov = Provenance::new(
            &base.cfg,
            &base.input,
            req.input_file.as_deref().unwrap_or(""),
            BUILD_COMMIT,
        );
        if req.baseline_parameter_source.is_some() || req.baseline_parameter_class.is_some() {
            let source = req
                .baseline_parameter_source
                .unwrap_or_else(|| base_prov.parameter_source.clone());
            let class = req
                .baseline_parameter_class
                .unwrap_or_else(|| base_prov.parameter_class.clone());
            base_prov = base_prov.with_parameter_source(&source, &class);
        }
        comparison_csv(&prov, &base_prov, &base_result.series, &result.series)
    } else {
        text
    };
    let text = text.map_err(|e| vec![format!("{e:#}")])?;
    Ok(json!({
        "schema": SCHEMA_BTCS,
        "status": "ok",
        "label": LABEL_BTCS,
        "engine": engine(),
        "format": req.format,
        "media_type": media_type,
        "text": text,
    }))
}

/// The text of a data file of a run, written by the library's export (the CSV time series, the
/// CSV profiles, the JSON of the whole run or the CSV comparison of two runs), each with its provenance header: solver
/// version and commit, input digest, parameter source and class. The page saves the text
/// unchanged. Request: `{schema: 2, config, profile_times_s?, format, input_file?,
/// parameter_source?, parameter_class?}`, `format` one of [`EXPORT_FORMATS`].
pub fn export_run(request_json: &str) -> String {
    let result = placeholder_check(request_json).and_then(|()| {
        let v: Value =
            serde_json::from_str(request_json).map_err(|e| vec![format!("request: {e}")])?;
        export_value(v)
    });
    match result {
        Ok(v) => v.to_string(),
        Err(errors) => rejected(errors).to_string(),
    }
}

fn placeholder_check(raw: &str) -> Result<(), Vec<String>> {
    let errors = placeholder_errors(raw);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

// ---------------------------------------------------------------------------------------
// Compare
// ---------------------------------------------------------------------------------------

fn flatten(prefix: &str, v: &Value, out: &mut BTreeMap<String, Value>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                flatten(&format!("{prefix}/{k}"), x, out);
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                flatten(&format!("{prefix}/{i}"), x, out);
            }
        }
        other => {
            out.insert(prefix.to_string(), other.clone());
        }
    }
}

fn floats_of(v: &Value) -> Vec<Option<f64>> {
    v.as_array()
        .map(|a| a.iter().map(Value::as_f64).collect())
        .unwrap_or_default()
}

fn difference(a: &[Option<f64>], b: &[Option<f64>]) -> Vec<Value> {
    // modified minus baseline; null where either value is not defined (the release rate at 0).
    b.iter()
        .zip(a)
        .map(|(m, base)| match (m, base) {
            (Some(m), Some(b)) => json!(m - b),
            _ => Value::Null,
        })
        .collect()
}

fn compare_value(raw: &str) -> Result<Value, Vec<String>> {
    let req: CompareRequest =
        serde_json::from_str(raw).map_err(|e| vec![format!("request: {e}")])?;
    check_schema_value(req.schema, SCHEMA_BTCS)?;
    let run_one = |which: &str, v: Value| -> Result<Value, Vec<String>> {
        let text = v.to_string();
        let errors = placeholder_errors(&text);
        let r = if errors.is_empty() {
            run_value(v)
        } else {
            Err(errors)
        };
        r.map_err(|errs| errs.into_iter().map(|e| format!("{which}: {e}")).collect())
    };
    let baseline = run_one("baseline", req.baseline)?;
    let modified = run_one("modified", req.modified)?;
    let same_grid = {
        let tb = floats_of(&baseline["times_s"]);
        let tm = floats_of(&modified["times_s"]);
        tb.len() == tm.len()
            && tb
                .iter()
                .zip(&tm)
                .all(|(a, b)| a.map(f64::to_bits) == b.map(f64::to_bits))
    };
    if !same_grid {
        return Err(vec![
            "the baseline and modified runs have different output times; the difference is defined only on a common time grid and is never interpolated".into(),
        ]);
    }
    let fb = floats_of(&baseline["release_fraction"]);
    let fm = floats_of(&modified["release_fraction"]);
    let d_release = difference(&fb, &fm);
    let abs: Vec<f64> = d_release
        .iter()
        .filter_map(Value::as_f64)
        .map(f64::abs)
        .collect();
    let mut a = BTreeMap::new();
    let mut b = BTreeMap::new();
    flatten("", &baseline["inputs"], &mut a);
    flatten("", &modified["inputs"], &mut b);
    let mut changes = Vec::new();
    for key in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
        if a.get(key) != b.get(key) {
            changes.push(json!({
                "path": key,
                "baseline": a.get(key).cloned().unwrap_or(Value::Null),
                "modified": b.get(key).cloned().unwrap_or(Value::Null),
            }));
        }
    }
    let sub = |name: &str| difference(&floats_of(&baseline[name]), &floats_of(&modified[name]));
    let mut diff = Map::new();
    diff.insert("times_s".into(), baseline["times_s"].clone());
    diff.insert("release_fraction".into(), json!(d_release));
    diff.insert(
        "release_rate_mol_s".into(),
        json!(sub("release_rate_mol_s")),
    );
    diff.insert("inventory_mol".into(), json!(sub("inventory_mol")));
    diff.insert(
        "cumulative_release_mol".into(),
        json!(sub("cumulative_release_mol")),
    );
    Ok(json!({
        "schema": SCHEMA_BTCS,
        "status": "ok",
        "label": LABEL_BTCS,
        "engine": engine(),
        "solver": solver_value(),
        "baseline": baseline,
        "modified": modified,
        "difference": Value::Object(diff),
        "difference_definition": "modified minus baseline, point by point on the common output times",
        "summary": {
            "max_abs_release_fraction_difference": abs.iter().copied().fold(0.0f64, f64::max),
            "final_release_fraction_difference": d_release.last().cloned().unwrap_or(Value::Null),
        },
        "changes": changes,
    }))
}

pub fn compare(request_json: &str) -> String {
    match compare_value(request_json) {
        Ok(v) => v.to_string(),
        Err(errors) => rejected(errors).to_string(),
    }
}

// ---------------------------------------------------------------------------------------
// Validate
// ---------------------------------------------------------------------------------------

pub fn validate(input_text: &str) -> String {
    let mut warnings = Vec::new();
    let result = (|| -> Result<(Plan, Value), Vec<String>> {
        let errors = placeholder_errors(input_text);
        if !errors.is_empty() {
            return Err(errors);
        }
        let value: Value = if input_text.trim_start().starts_with('{') {
            serde_json::from_str(input_text).map_err(|e| vec![format!("JSON: {e}")])?
        } else {
            toml::from_str::<Value>(input_text).map_err(|e| vec![format!("TOML: {e}")])?
        };
        let (config, profile_times) = match value {
            Value::Object(ref m) if m.contains_key("config") => {
                let req: RunRequest =
                    serde_json::from_value(value).map_err(|e| vec![format!("request: {e}")])?;
                check_schema_value(req.schema, SCHEMA_BTCS)?;
                check_mechanisms(&req.mechanisms)?;
                (req.config, req.profile_times_s)
            }
            v => (v, None),
        };
        let cfg = config_from_request_value(config)?;
        let (input, plan) = check_config(&cfg)?;
        if let Some(times) = &profile_times {
            if times.len() > MAX_PROFILE_TIMES || times.iter().any(|t| !t.is_finite() || *t < 0.0) {
                return Err(vec![format!(
                    "The profile times (s) must be at most {MAX_PROFILE_TIMES} finite numbers of at least 0 s"
                )]);
            }
        }
        if plan.dt_s != cfg.time.dt_s {
            warnings.push(format!(
                "The solver will use a time step of {:e} s ({} equal steps), not the requested time step of {:e} s, because the simulation duration is divided into a whole number of equal steps",
                plan.dt_s, plan.steps, cfg.time.dt_s
            ));
        }
        if profile_times.is_none() {
            let values = plan.output_rows.saturating_mul(plan.nodes);
            if values > MAX_PROFILE_VALUES_BTCS {
                return Err(vec![format!(
                    "{} output rows of {} grid nodes each give {values} stored concentration values, above the limit of {MAX_PROFILE_VALUES_BTCS} values; raise the number of time steps between two output rows or request profile times",
                    plan.output_rows, plan.nodes
                )]);
            }
        }
        let grid = Grid::build(&input).map_err(|e| vec![format!("{e:#}")])?;
        Ok((plan, grid_value(&grid, &cfg, &input)))
    })();
    match result {
        Ok((plan, grid)) => json!({
            "schema": SCHEMA_BTCS,
            "ok": true,
            "errors": [],
            "warnings": warnings,
            "plan": {
                "intervals_total": plan.intervals,
                "nodes": plan.nodes,
                "steps": plan.steps,
                "dt_used_s": plan.dt_s,
                "output_rows": plan.output_rows,
                "profile_values_at_output_rows": plan.output_rows.saturating_mul(plan.nodes),
            },
            "grid": grid,
            "solver": solver_value(),
            "limits": limits_value(),
        }),
        Err(errors) => json!({
            "schema": SCHEMA_BTCS,
            "ok": false,
            "errors": errors,
            "warnings": warnings,
            "plan": Value::Null,
            "grid": Value::Null,
            "solver": solver_value(),
            "limits": limits_value(),
        }),
    }
    .to_string()
}
