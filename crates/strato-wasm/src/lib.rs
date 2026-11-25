//! STRATO WebAssembly wrapper (prompt/TECHNICAL_ARCHITECTURE.md Section 3).
//!
//! JSON in, JSON out. Every number this crate returns is computed by the verified `strato`
//! v1.0 library (`numerical::finite_volume`, Code B; `analytical::sturm_liouville` through
//! `postprocessing::comparison`, Code C) or is bookkeeping over its data: the per-layer
//! inventories sum `Cell::volume * c` over the cells of a region, the inventory balance
//! reports the library's own inventory, release and residual, and the difference series of
//! `compare_v1` subtracts two library results. No physics is written here, and the frontend
//! does no scientific arithmetic (owner decision M4).
//!
//! Rules: no file I/O, no `SystemTime` or `Instant`, no panics across the boundary. On
//! `wasm32-unknown-unknown` a panic aborts and traps the instance, so every rejection rule of
//! the wrapper facts table is applied before `strato` is called, the work is capped by
//! [`MAX_TOTAL_CELLS`], [`MAX_STEPS`] and [`MAX_PROFILE_VALUES`], and the worker contains what
//! remains (time-out and instance recreation).

// The BTCS reply is one large `json!` literal.
#![recursion_limit = "512"]

pub mod btcs_api;
pub mod tolerances;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use strato::{
    analytical::sturm_liouville::ModalRelease,
    config::{Config, PLACEHOLDER},
    numerical::finite_volume::{self as fv, Mesh, StepDiagnostics, TimeSeries},
    postprocessing::comparison::{self as cmp, MAX_MODES},
};
use wasm_bindgen::prelude::*;

/// ABI version of the exported functions; the worker reports it in its hello message.
pub const ABI_VERSION: u32 = 2;
/// Schema version of every request and response object of the finite-volume functions
/// (`run_v1`, `analytical_release`, `compare_v1`, `validate_config`). The BTCS functions use
/// [`btcs_api::SCHEMA_BTCS`].
pub const SCHEMA: u32 = 1;
/// Result label of every value this build computes.
pub const LABEL: &str = "COMPUTED v1.0";
/// Version of the wrapped `strato` package, read from its Cargo.toml by build.rs.
pub const STRATO_VERSION: &str = env!("STRATO_PACKAGE_VERSION");
/// Version of this wrapper crate.
pub const WRAPPER_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Commit of the tree this module was built from (passed in by the build script).
pub const BUILD_COMMIT: &str = env!("STRATO_WRAPPER_COMMIT");
const BUILD_DIRTY: &str = env!("STRATO_WRAPPER_DIRTY");
/// The wasm-bindgen version this crate is pinned to (Cargo.toml).
pub const WASM_BINDGEN_VERSION: &str = "0.2.130";

/// Mechanism ids (`mechanism_id` of data/research/mechanisms.csv) that this build solves.
/// In v1.0 exactly the layered constant-diffusivity diffusion of builder stage 1.
pub const CAPABILITIES: &[&str] = &["diffusion_layered"];

/// Wrapper limit: total cells over the five regions. Not a physical parameter; the native
/// CLI keeps no such limit. Value from measured browser run times (prompt/PROGRESS.md,
/// "Phase 3 record").
pub const MAX_TOTAL_CELLS: usize = 2_000;
/// Wrapper limit: `ceil(end_time_s / dt_s)`, the number of Backward Euler steps.
pub const MAX_STEPS: usize = 20_000;
/// Wrapper limit: stored profile values, (number of output rows) x (total cells). It bounds
/// the size of the response and the memory of the instance.
pub const MAX_PROFILE_VALUES: usize = 1_000_000;

/// Wrapper limit on the series (Code C): the most eigenmodes the wrapper lets
/// `ModalRelease::new` compute. The series keeps every mode up to `45 / t_min` (t_min the
/// earliest positive output time), and its cost grows with the square of the mode count. The
/// wrapper bounds the count from above before it calls the library (see
/// [`series_mode_bound`]) and leaves the series out, with a message, when the bound exceeds
/// this limit; the finite-volume result is unaffected. Below the library's own cap
/// `MAX_MODES = 20000`, so the library's capping branch is never entered from the wrapper.
pub const MAX_SERIES_MODES: usize = 1_000;

/// `output.directory` is required by `Config` but used only by the CLI. The wrapper never
/// writes a file; it fills this value when a request leaves the field out.
const UNUSED_OUTPUT_DIRECTORY: &str = "(not used by the wrapper)";

// ---------------------------------------------------------------------------------------
// Exported functions (names binding for the frontend)
// ---------------------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(message: &str);
}

/// Runs once when the module is instantiated: report any panic to the console before the
/// instance traps (defence 3 of prompt/TECHNICAL_ARCHITECTURE.md Section 3).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        console_error(&format!("strato-wasm panic: {info}"));
    }));
}

#[wasm_bindgen]
pub fn abi_version() -> u32 {
    ABI_VERSION
}

#[wasm_bindgen]
pub fn build_info() -> String {
    build_info_value().to_string()
}

#[wasm_bindgen]
pub fn validate_config(toml_or_json: &str) -> String {
    validate_value(toml_or_json).to_string()
}

#[wasm_bindgen]
pub fn run_v1(request_json: &str) -> String {
    match run_request(request_json) {
        Ok(v) => v.to_string(),
        Err(errors) => rejected(errors).to_string(),
    }
}

#[wasm_bindgen]
pub fn analytical_release(request_json: &str) -> String {
    match analytical_request(request_json) {
        Ok(v) => v.to_string(),
        Err(errors) => rejected(errors).to_string(),
    }
}

#[wasm_bindgen]
pub fn compare_v1(request_json: &str) -> String {
    match compare_request(request_json) {
        Ok(v) => v.to_string(),
        Err(errors) => rejected(errors).to_string(),
    }
}

// The BTCS interface (ABI 2): method-neutral names. See `btcs_api` and
// stage-b/btcs/wasm-interface.md for the fields.

/// Validates a configuration (TOML or JSON, a bare configuration or a request) for the BTCS
/// solver and returns the plan, the grid and the limits.
#[wasm_bindgen]
pub fn validate(toml_or_json: &str) -> String {
    btcs_api::validate(toml_or_json)
}

/// Runs the BTCS solver (schema 2 request) and returns the complete reply.
#[wasm_bindgen]
pub fn run(request_json: &str) -> String {
    btcs_api::run(request_json)
}

/// Runs a baseline and a modified configuration and returns both runs and their difference.
#[wasm_bindgen]
pub fn compare(request_json: &str) -> String {
    btcs_api::compare(request_json)
}

// ---------------------------------------------------------------------------------------
// Build information
// ---------------------------------------------------------------------------------------

pub fn build_info_value() -> Value {
    json!({
        "schema": SCHEMA,
        "abi": ABI_VERSION,
        "crate": "strato-wasm",
        "wrapper_version": WRAPPER_VERSION,
        "strato_version": STRATO_VERSION,
        "commit": BUILD_COMMIT,
        "dirty": BUILD_DIRTY == "1",
        "wasm_bindgen": WASM_BINDGEN_VERSION,
        "label": LABEL,
        "capabilities": CAPABILITIES,
        "limits": limits_value(),
        "legacy_finite_volume": {
            "note": "label, capabilities and limits above describe the finite-volume functions, which stay until the retirement task",
            "functions": ["validate_config", "run_v1", "analytical_release", "compare_v1"],
        },
        "btcs": btcs_api::build_info_value(),
    })
}

fn limits_value() -> Value {
    json!({
        "max_total_cells": MAX_TOTAL_CELLS,
        "max_steps": MAX_STEPS,
        "max_profile_values": MAX_PROFILE_VALUES,
        "max_series_modes": MAX_SERIES_MODES,
        "library_max_modes": MAX_MODES,
    })
}

/// An upper bound on the number of eigenvalues the series keeps for the earliest positive
/// output time `t_min`, i.e. on `sturm_liouville::eigenvalue_count(45 / t_min, layers)`.
/// That function adds, per layer, `floor((theta0 + k L) / pi) - floor(theta0 / pi)` with
/// `k = sqrt(Lambda / D)`, which is at most `k L / pi + 1`; the bound is the sum of these
/// terms, evaluated in floating point so that no integer can overflow. It is a guard of the
/// wrapper's work, never shown as a result.
pub fn series_mode_bound(cfg: &Config, t_min: f64) -> f64 {
    let lambda = 45.0 / t_min;
    let mut inner = 0.0;
    let mut bound = 0.0;
    for r in &cfg.regions {
        let k = (lambda / r.diffusivity_m2_s).sqrt();
        bound += k * (r.outer_radius_m - inner) / std::f64::consts::PI + 1.0;
        inner = r.outer_radius_m;
    }
    bound
}

fn series_allowed(cfg: &Config, t_min: f64) -> Result<(), String> {
    let bound = series_mode_bound(cfg, t_min);
    if bound.is_finite() && bound <= MAX_SERIES_MODES as f64 {
        Ok(())
    } else {
        Err(format!(
            "series (Code C) not computed: the earliest output time {t_min:e} s needs up to {bound:.3e} eigenmodes, more than the wrapper limit MAX_SERIES_MODES = {MAX_SERIES_MODES}; a larger output interval (dt x output_every_steps) needs fewer"
        ))
    }
}

fn engine_value() -> Value {
    json!({"strato_version": STRATO_VERSION, "commit": BUILD_COMMIT, "abi": ABI_VERSION})
}

fn rejected(errors: Vec<String>) -> Value {
    json!({"schema": SCHEMA, "status": "rejected", "errors": errors, "engine": engine_value()})
}

// ---------------------------------------------------------------------------------------
// Request parsing and the rejection rules (wrapper facts table)
// ---------------------------------------------------------------------------------------

/// The rule of `unfilled_placeholders` in `src/config.rs` (line 45 at v1.0.1), copied because
/// that function is private and `Config::load` reads a file: every line that is not a comment
/// and contains `INSERT_VALUE_HERE` is reported with the instruction comment above it.
pub fn unfilled_placeholders(raw: &str) -> Vec<String> {
    let mut last_instruction = String::new();
    let mut found = Vec::new();
    for (i, line) in raw.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            if trimmed.starts_with("# INSERT") && trimmed.ends_with("BELOW") {
                last_instruction = trimmed.trim_start_matches('#').trim().to_string();
            }
        } else if trimmed.contains(PLACEHOLDER) {
            found.push(format!("line {}: {}", i + 1, last_instruction));
        }
    }
    found
}

fn placeholder_errors(raw: &str) -> Vec<String> {
    let missing = unfilled_placeholders(raw);
    if missing.is_empty() {
        return Vec::new();
    }
    vec![format!(
        "{} literature value(s) in the request are still {PLACEHOLDER}: {}",
        missing.len(),
        missing.join("; ")
    )]
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunRequest {
    schema: u32,
    config: Value,
    #[serde(default)]
    mechanisms: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnalyticalRequest {
    schema: u32,
    config: Value,
    #[serde(default)]
    times_s: Option<Vec<f64>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CompareRequest {
    schema: u32,
    baseline: Value,
    modified: Value,
}

fn check_schema(schema: u32) -> Result<(), Vec<String>> {
    check_schema_value(schema, SCHEMA)
}

fn check_schema_value(schema: u32, expected: u32) -> Result<(), Vec<String>> {
    if schema != expected {
        return Err(vec![format!(
            "unsupported request schema {schema}; this build reads schema {expected}"
        )]);
    }
    Ok(())
}

/// Deserialises a `strato::config::Config` from a JSON value, filling the CLI-only
/// `output.directory` when it is absent. Serde rejects a missing field and a wrong type.
fn config_from_value(mut v: Value) -> Result<Config, Vec<String>> {
    if let Value::Object(map) = &mut v {
        if !map.contains_key("output") {
            let mut out = Map::new();
            out.insert(
                "directory".into(),
                Value::String(UNUSED_OUTPUT_DIRECTORY.into()),
            );
            map.insert("output".into(), Value::Object(out));
        }
    }
    serde_json::from_value::<Config>(v).map_err(|e| vec![format!("config: {e}")])
}

/// The mechanism ids a request may name: only those of [`CAPABILITIES`].
fn check_mechanisms(requested: &Option<Vec<String>>) -> Result<(), Vec<String>> {
    let unsupported: Vec<String> = requested
        .iter()
        .flatten()
        .filter(|m| !CAPABILITIES.contains(&m.as_str()))
        .map(|m| {
            format!(
                "mechanism {m} is not supported by this solver build (capabilities: {})",
                CAPABILITIES.join(", ")
            )
        })
        .collect();
    if unsupported.is_empty() {
        Ok(())
    } else {
        Err(unsupported)
    }
}

/// The work a validated configuration asks for, computed with the step rule of
/// `finite_volume::run`. Every value is checked against the wrapper limits.
#[derive(Clone, Copy, Debug)]
pub struct Plan {
    pub cells: usize,
    pub steps: usize,
    pub dt_s: f64,
    pub output_rows: usize,
}

/// `Config::validate`, the wrapper limits and the positive start concentration, in this order,
/// before any other `strato` call.
pub fn check_config(cfg: &Config) -> Result<Plan, Vec<String>> {
    cfg.validate().map_err(|e| vec![format!("{e:#}")])?;
    let mut errors = Vec::new();
    // validate() guarantees cells >= 1 per region; saturate so a huge value cannot overflow.
    let cells = cfg
        .regions
        .iter()
        .fold(0usize, |a, r| a.saturating_add(r.cells));
    if cells > MAX_TOTAL_CELLS {
        errors.push(format!(
            "total cell count {cells} exceeds the wrapper limit MAX_TOTAL_CELLS = {MAX_TOTAL_CELLS}"
        ));
    }
    let ratio = (cfg.time.end_time_s / cfg.time.dt_s).ceil();
    if ratio.is_nan() || ratio > MAX_STEPS as f64 {
        errors.push(format!(
            "step count ceil(end_time_s / dt_s) = {ratio:e} exceeds the wrapper limit MAX_STEPS = {MAX_STEPS}"
        ));
    }
    if cfg.initial.kernel_concentration_mol_m3 <= 0.0 {
        // finite_volume::run rejects a non-positive start inventory with this message
        // (src/numerical/finite_volume.rs line 187); the wrapper steps itself, so it checks first.
        errors.push("initial inventory must be positive (kernel_concentration_mol_m3 > 0)".into());
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let steps = ratio as usize;
    let oe = cfg.time.output_every_steps;
    let output_rows = 1 + steps / oe + usize::from(!steps.is_multiple_of(oe));
    let values = output_rows.saturating_mul(cells);
    if values > MAX_PROFILE_VALUES {
        return Err(vec![format!(
            "{output_rows} output rows of {cells} cells give {values} profile values, more than the wrapper limit MAX_PROFILE_VALUES = {MAX_PROFILE_VALUES}; raise output_every_steps"
        )]);
    }
    Ok(Plan {
        cells,
        steps,
        dt_s: cfg.time.end_time_s / steps as f64,
        output_rows,
    })
}

fn parse_run_request(raw: &str) -> Result<(Config, Plan, Vec<String>), Vec<String>> {
    let errors = placeholder_errors(raw);
    if !errors.is_empty() {
        return Err(errors);
    }
    let req: RunRequest = serde_json::from_str(raw).map_err(|e| vec![format!("request: {e}")])?;
    check_schema(req.schema)?;
    check_mechanisms(&req.mechanisms)?;
    let cfg = config_from_value(req.config)?;
    let plan = check_config(&cfg)?;
    let mechanisms = req
        .mechanisms
        .unwrap_or_else(|| CAPABILITIES.iter().map(|s| s.to_string()).collect());
    Ok((cfg, plan, mechanisms))
}

// ---------------------------------------------------------------------------------------
// The run (Code B) with snapshots, exactly as finite_volume::run steps
// ---------------------------------------------------------------------------------------

/// Result of [`run_with_profiles`]: the time series of `finite_volume::run` and the profile
/// at every recorded row.
pub struct Snapshots {
    pub rows: TimeSeries,
    pub profiles: Vec<Vec<f64>>,
    pub initial_inventory: f64,
    pub steps: usize,
    pub dt_s: f64,
}

fn min_of(c: &[f64]) -> f64 {
    c.iter().copied().fold(f64::INFINITY, f64::min)
}

/// Steps with `backward_euler_step` using exactly the step rule, the Kahan update order and
/// the row rule of `finite_volume::run` (src/numerical/finite_volume.rs lines 184 to 236 at
/// v1.0.1), and also keeps the profile of every recorded row. The tests show that the rows
/// are bit-identical to `finite_volume::run` for the same configuration.
pub fn run_with_profiles(cfg: &Config, mesh: &Mesh) -> Result<Snapshots> {
    let mut c = fv::initial_concentration(cfg, mesh);
    let initial = fv::inventory(mesh, &c);
    if initial <= 0.0 {
        anyhow::bail!("initial inventory must be positive");
    }
    let steps = (cfg.time.end_time_s / cfg.time.dt_s).ceil() as usize;
    let dt = cfg.time.end_time_s / steps as f64;
    let mut released = 0.0;
    let mut release_compensation = 0.0;
    let mut rows = Vec::new();
    let mut profiles = Vec::new();

    rows.push((
        0.0,
        StepDiagnostics {
            inventory: initial,
            cumulative_release: 0.0,
            release_fraction: 0.0,
            release_rate: mesh.outer_conductance * c[c.len() - 1],
            balance_residual: 0.0,
            min_concentration: min_of(&c),
        },
    ));
    profiles.push(c.clone());

    for step in 1..=steps {
        let next = fv::backward_euler_step(mesh, &c, dt)?;
        let release_rate = mesh.outer_conductance * next[next.len() - 1];
        let increment = dt * release_rate;
        let y = increment - release_compensation;
        let t = released + y;
        release_compensation = (t - released) - y;
        released = t;
        c = next;
        let inv = fv::inventory(mesh, &c);
        let residual = inv + released - initial;
        if step % cfg.time.output_every_steps == 0 || step == steps {
            rows.push((
                step as f64 * dt,
                StepDiagnostics {
                    inventory: inv,
                    cumulative_release: released,
                    release_fraction: released / initial,
                    release_rate,
                    balance_residual: residual,
                    min_concentration: min_of(&c),
                },
            ));
            profiles.push(c.clone());
        }
    }
    Ok(Snapshots {
        rows,
        profiles,
        initial_inventory: initial,
        steps,
        dt_s: dt,
    })
}

/// Inventory of each region (`Cell::material`), summed over its cells in cell order.
pub fn layer_inventories(mesh: &Mesh, regions: usize, c: &[f64]) -> Vec<f64> {
    let mut out = vec![0.0; regions];
    for (cell, &x) in mesh.cells.iter().zip(c) {
        out[cell.material] += cell.volume * x;
    }
    out
}

#[derive(Serialize)]
struct RegionEcho<'a> {
    name: &'a str,
    outer_radius_m: f64,
    diffusivity_m2_s: f64,
    cells: usize,
}

/// Serialisable mirror of the validated `Config` (which derives `Deserialize` only).
/// `output.directory` is left out: the wrapper writes no file.
#[derive(Serialize)]
struct ConfigEcho<'a> {
    case_name: &'a str,
    species: &'a str,
    temperature_k: f64,
    regions: Vec<RegionEcho<'a>>,
    initial: Value,
    time: Value,
}

fn echo(cfg: &Config) -> Value {
    serde_json::to_value(ConfigEcho {
        case_name: &cfg.case_name,
        species: &cfg.species,
        temperature_k: cfg.temperature_k,
        regions: cfg
            .regions
            .iter()
            .map(|r| RegionEcho {
                name: &r.name,
                outer_radius_m: r.outer_radius_m,
                diffusivity_m2_s: r.diffusivity_m2_s,
                cells: r.cells,
            })
            .collect(),
        initial: json!({"kernel_concentration_mol_m3": cfg.initial.kernel_concentration_mol_m3}),
        time: json!({
            "dt_s": cfg.time.dt_s,
            "end_time_s": cfg.time.end_time_s,
            "output_every_steps": cfg.time.output_every_steps,
        }),
    })
    .unwrap_or(Value::Null)
}

fn column(rows: &TimeSeries, f: impl Fn(&StepDiagnostics) -> f64) -> Vec<f64> {
    rows.iter().map(|(_, d)| f(d)).collect()
}

/// Runs a validated configuration and builds the RunResponse object.
pub fn run_config(cfg: &Config, plan: Plan, mechanisms: Vec<String>) -> Result<Value> {
    let mesh = fv::build_mesh(cfg)?;
    let snap = run_with_profiles(cfg, &mesh)?;
    let regions = cfg.regions.len();
    let mut messages: Vec<String> = Vec::new();

    if snap.dt_s != cfg.time.dt_s {
        messages.push(format!(
            "time step used: {:e} s ({} steps of t_end / ceil(t_end / dt)); requested {:e} s",
            snap.dt_s, snap.steps, cfg.time.dt_s
        ));
    }

    let times: Vec<f64> = snap.rows.iter().map(|(t, _)| *t).collect();
    let mut profiles = Vec::with_capacity(snap.profiles.len());
    let mut worst_layer_sum = 0.0f64;
    for ((t, _), c) in snap.rows.iter().zip(&snap.profiles) {
        let inv = fv::inventory(&mesh, c);
        let layers = layer_inventories(&mesh, regions, c);
        let share = if inv > 0.0 {
            Value::from(layers.iter().map(|l| l / inv).collect::<Vec<f64>>())
        } else {
            messages.push(format!(
                "t = {t:e} s: inventory is not positive, layer shares not given"
            ));
            Value::Null
        };
        let sum: f64 = layers.iter().sum();
        if inv > 0.0 {
            worst_layer_sum = worst_layer_sum.max((sum - inv).abs() / inv);
        }
        profiles.push(json!({
            "t_s": t,
            "c_mol_per_m3": c,
            "inventory_mol": inv,
            "layer_inventory_mol": layers,
            "layer_share": share,
        }));
    }

    let initial = snap.initial_inventory;
    let (last_t, last) = snap
        .rows
        .last()
        .map(|(t, d)| (*t, d.clone()))
        .expect("the initial row is always present");
    let max_abs_residual = snap
        .rows
        .iter()
        .map(|(_, d)| d.balance_residual.abs())
        .fold(0.0f64, f64::max);
    let min_c = snap
        .rows
        .iter()
        .map(|(_, d)| d.min_concentration)
        .fold(f64::INFINITY, f64::min);

    // Code C on the same output times, exactly as the CLI calls it. Its failure never
    // invalidates the finite-volume result.
    let t_min = snap
        .rows
        .iter()
        .map(|(t, _)| *t)
        .filter(|t| *t > 0.0)
        .fold(f64::INFINITY, f64::min);
    let series = series_allowed(cfg, t_min).and_then(|()| {
        cmp::compare_release(cfg, &snap.rows)
            .map_err(|e| format!("series (Code C) not available: {e:#}"))
    });
    let (analytical, code_c) = match series {
        Ok(rc) => {
            let resolved: Vec<_> = rc
                .rows
                .iter()
                .filter_map(|r| r.exact.map(|e| (r.time_s, r.fv, e)))
                .collect();
            if rc.max_abs_error < 3.0 * rc.accuracy_floor {
                messages.push(
                    "finite-volume and series differences are within 3x of the series accuracy floor; they are not resolved".into(),
                );
            }
            // The pointwise difference |F_B - F_C| is not returned: it fails the browser parity
            // rule (relative 1e-12 per value) because F_C differs in its last bits between the
            // native and the wasm math library and the subtraction cancels about four digits
            // (prompt/PROGRESS.md, "Phase 3 record"). The largest and the final difference are
            // returned in the diagnostics.
            let a = json!({
                "times_s": resolved.iter().map(|r| r.0).collect::<Vec<f64>>(),
                "release_fraction": resolved.iter().map(|r| r.2).collect::<Vec<f64>>(),
                "accuracy_floor": rc.accuracy_floor,
                "valid_from_s": rc.min_valid_time_s,
                "modes": rc.modes,
            });
            let d = json!({
                "modes": rc.modes,
                "max_abs_difference": rc.max_abs_error,
                "final_abs_difference": rc.final_abs_error,
                "accuracy_floor": rc.accuracy_floor,
                "valid_from_s": rc.min_valid_time_s,
                "min_flux_ratio": rc.min_flux_ratio,
                "resolved": rc.max_abs_error >= 3.0 * rc.accuracy_floor,
            });
            (a, d)
        }
        Err(e) => {
            messages.push(e);
            (Value::Null, Value::Null)
        }
    };

    let faces: Vec<f64> = std::iter::once(0.0)
        .chain(mesh.cells.iter().map(|c| c.right))
        .collect();
    Ok(json!({
        "schema": SCHEMA,
        "status": "ok",
        "label": LABEL,
        "engine": engine_value(),
        "mechanisms": mechanisms,
        "inputs": echo(cfg),
        "mesh": {
            "cell_centroids_m": mesh.cells.iter().map(|c| c.centroid).collect::<Vec<f64>>(),
            "cell_faces_m": faces,
            "region_of_cell": mesh.cells.iter().map(|c| c.material).collect::<Vec<usize>>(),
            "region_names": cfg.regions.iter().map(|r| r.name.as_str()).collect::<Vec<&str>>(),
        },
        "times_s": times,
        "release_fraction": column(&snap.rows, |d| d.release_fraction),
        "inventory_mol": column(&snap.rows, |d| d.inventory),
        "cumulative_release_mol": column(&snap.rows, |d| d.cumulative_release),
        "release_rate_mol_s": column(&snap.rows, |d| d.release_rate),
        "balance_residual_mol": column(&snap.rows, |d| d.balance_residual),
        "min_concentration_mol_m3": column(&snap.rows, |d| d.min_concentration),
        "profiles": profiles,
        "balance": {
            "form": "v1.0",
            "statement": "inventory + released - start inventory = residual",
            "source": "docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md Section 3.5.4; residual as finite_volume::run reports it",
            "start_inventory_mol": initial,
            "final": {
                "t_s": last_t,
                "inventory_mol": last.inventory,
                "released_mol": last.cumulative_release,
                "residual_mol": last.balance_residual,
            },
            "max_abs_residual_mol": max_abs_residual,
            "max_abs_residual_relative": max_abs_residual / initial,
        },
        "diagnostics": {
            "cells": plan.cells,
            "regions": regions,
            "steps": snap.steps,
            "dt_requested_s": cfg.time.dt_s,
            "dt_used_s": snap.dt_s,
            "end_time_s": cfg.time.end_time_s,
            "output_rows": snap.rows.len(),
            "explicit_dt_bound_s": fv::explicit_dt_bound(&mesh),
            "final_release_fraction": last.release_fraction,
            "max_abs_balance_residual_mol": max_abs_residual,
            "max_abs_balance_residual_relative": max_abs_residual / initial,
            "min_concentration_mol_m3": min_c,
            "layer_sum_max_relative_difference": worst_layer_sum,
            "code_c": code_c,
        },
        "analytical": analytical,
        "messages": messages,
    }))
}

fn run_request(raw: &str) -> Result<Value, Vec<String>> {
    let (cfg, plan, mechanisms) = parse_run_request(raw)?;
    run_config(&cfg, plan, mechanisms).map_err(|e| vec![format!("{e:#}")])
}

// ---------------------------------------------------------------------------------------
// Code C on its own, and the comparison of two runs
// ---------------------------------------------------------------------------------------

/// The output times of `finite_volume::run` for a configuration (step rule and row rule).
fn output_times(cfg: &Config, plan: Plan) -> Vec<f64> {
    let mut t = vec![0.0];
    for step in 1..=plan.steps {
        if step % cfg.time.output_every_steps == 0 || step == plan.steps {
            t.push(step as f64 * plan.dt_s);
        }
    }
    t
}

fn analytical_request(raw: &str) -> Result<Value, Vec<String>> {
    let errors = placeholder_errors(raw);
    if !errors.is_empty() {
        return Err(errors);
    }
    let req: AnalyticalRequest =
        serde_json::from_str(raw).map_err(|e| vec![format!("request: {e}")])?;
    check_schema(req.schema)?;
    let cfg = config_from_value(req.config)?;
    let plan = check_config(&cfg)?;
    let times = match req.times_s {
        Some(t) => {
            if t.len() > MAX_STEPS + 1 {
                return Err(vec![format!(
                    "more than {} requested times (wrapper limit MAX_STEPS + 1)",
                    MAX_STEPS + 1
                )]);
            }
            if t.iter().any(|x| !x.is_finite() || *x < 0.0) {
                return Err(vec![
                    "requested times must be finite and non-negative".into()
                ]);
            }
            t
        }
        None => output_times(&cfg, plan),
    };
    let positive: Vec<f64> = times.iter().copied().filter(|t| *t > 0.0).collect();
    if positive.is_empty() {
        return Err(vec!["no positive output times to compare".into()]);
    }
    // The same truncation as postprocessing::comparison::compare_release: exp(-45) of the
    // omitted modes at the earliest positive time, at most MAX_MODES modes.
    let t_min = positive.iter().copied().fold(f64::INFINITY, f64::min);
    series_allowed(&cfg, t_min).map_err(|e| vec![e])?;
    let series = ModalRelease::new(&cmp::layers_from_config(&cfg), 45.0 / t_min, MAX_MODES)
        .map_err(|e| vec![format!("{e:#}")])?;
    let valid = series.min_valid_time();
    let kept: Vec<f64> = positive.into_iter().filter(|t| *t >= valid).collect();
    if kept.is_empty() {
        return Err(vec![format!(
            "exact series (capped at {MAX_MODES} modes) resolves only t >= {valid:.3e} s, later than every requested time"
        )]);
    }
    Ok(json!({
        "schema": SCHEMA,
        "status": "ok",
        "label": LABEL,
        "engine": engine_value(),
        "times_s": kept,
        "release_fraction": kept.iter().map(|t| series.release_fraction(*t)).collect::<Vec<f64>>(),
        "accuracy_floor": series.accuracy_floor(),
        "valid_from_s": valid,
        "modes": series.eigenvalues.len(),
        "messages": [],
    }))
}

fn compare_request(raw: &str) -> Result<Value, Vec<String>> {
    let req: CompareRequest =
        serde_json::from_str(raw).map_err(|e| vec![format!("request: {e}")])?;
    check_schema(req.schema)?;
    let run_one = |which: &str, v: Value| -> Result<Value, Vec<String>> {
        let text = v.to_string();
        run_request(&text)
            .map_err(|errs| errs.into_iter().map(|e| format!("{which}: {e}")).collect())
    };
    let baseline = run_one("baseline", req.baseline)?;
    let modified = run_one("modified", req.modified)?;
    let tb = &baseline["times_s"];
    let tm = &modified["times_s"];
    let (Some(tb), Some(tm)) = (tb.as_array(), tm.as_array()) else {
        return Err(vec!["internal: run without times".into()]);
    };
    let same_grid = tb.len() == tm.len()
        && tb
            .iter()
            .zip(tm)
            .all(|(a, b)| a.as_f64().map(f64::to_bits) == b.as_f64().map(f64::to_bits));
    if !same_grid {
        return Err(vec![
            "the baseline and modified runs have different output times; the difference is defined only on a common time grid and is never interpolated".into(),
        ]);
    }
    let fb = baseline["release_fraction"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let fm = modified["release_fraction"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let difference: Vec<f64> = fm
        .iter()
        .zip(&fb)
        .map(|(m, b)| m.as_f64().unwrap_or(f64::NAN) - b.as_f64().unwrap_or(f64::NAN))
        .collect();
    Ok(json!({
        "schema": SCHEMA,
        "status": "ok",
        "label": LABEL,
        "engine": engine_value(),
        "baseline": baseline,
        "modified": modified,
        "difference": {"times_s": tb, "release_fraction": difference},
    }))
}

// ---------------------------------------------------------------------------------------
// validate_config: TOML or JSON, a bare config or a run request
// ---------------------------------------------------------------------------------------

fn validate_value(input: &str) -> Value {
    let mut warnings = Vec::new();
    let result = (|| -> Result<Plan, Vec<String>> {
        let errors = placeholder_errors(input);
        if !errors.is_empty() {
            return Err(errors);
        }
        let value: Value = if input.trim_start().starts_with('{') {
            serde_json::from_str(input).map_err(|e| vec![format!("JSON: {e}")])?
        } else {
            toml::from_str::<Value>(input).map_err(|e| vec![format!("TOML: {e}")])?
        };
        let config = match value {
            Value::Object(ref m) if m.contains_key("config") => {
                let req: RunRequest =
                    serde_json::from_value(value).map_err(|e| vec![format!("request: {e}")])?;
                check_schema(req.schema)?;
                check_mechanisms(&req.mechanisms)?;
                req.config
            }
            v => v,
        };
        let cfg = config_from_value(config)?;
        let plan = check_config(&cfg)?;
        if plan.dt_s != cfg.time.dt_s {
            warnings.push(format!(
                "the solver will use dt = {:e} s ({} steps of t_end / ceil(t_end / dt)), not the requested {:e} s",
                plan.dt_s, plan.steps, cfg.time.dt_s
            ));
        }
        Ok(plan)
    })();
    match result {
        Ok(plan) => json!({
            "schema": SCHEMA,
            "ok": true,
            "errors": [],
            "warnings": warnings,
            "plan": {"cells": plan.cells, "steps": plan.steps, "dt_used_s": plan.dt_s, "output_rows": plan.output_rows},
            "limits": limits_value(),
        }),
        Err(errors) => json!({
            "schema": SCHEMA,
            "ok": false,
            "errors": errors,
            "warnings": warnings,
            "plan": Value::Null,
            "limits": limits_value(),
        }),
    }
}
