//! STRATO WebAssembly wrapper (prompt/TECHNICAL_ARCHITECTURE.md Section 3).
//!
//! JSON in, JSON out. Every number this crate returns is computed by the verified `strato`
//! library (`numerical::btcs`, Method A, and `analytical::sturm_liouville`, Method B, through
//! `postprocessing::comparison`) or is bookkeeping over its data: sums of library values,
//! divisions of one library value by another, selection of output rows. No physics is written
//! here, and the frontend does no scientific arithmetic (owner decision M4).
//!
//! The interface is [`btcs_api`]: method-neutral `validate`, `run`, `compare` and `export_run`
//! (capability `diffusion_btcs_layered`, result label `COMPUTED v2.0`). ABI 3 removed the
//! finite-volume functions of version 1.x; there is no other solver in this module.
//!
//! Rules: no file I/O, no `SystemTime` or `Instant`, no panics across the boundary. On
//! `wasm32-unknown-unknown` a panic aborts and traps the instance, so every rejection rule of
//! the wrapper facts table is applied before `strato` is called, the work is capped by the
//! limits of [`btcs_api`], and the worker contains what remains (time-out and instance
//! recreation).

// The BTCS reply is one large `json!` literal.
#![recursion_limit = "512"]

pub mod btcs_api;
pub mod tolerances;

use serde_json::{json, Map, Value};
use strato::config::{Config, PLACEHOLDER};
use wasm_bindgen::prelude::*;

/// ABI version of the exported functions; the worker reports it in its hello message. ABI 3
/// removed the finite-volume exports `validate_config`, `run_v1`, `analytical_release` and
/// `compare_v1` of ABI 1 and 2.
pub const ABI_VERSION: u32 = 3;
/// Version of the wrapped `strato` package, read from its Cargo.toml by build.rs.
pub const STRATO_VERSION: &str = env!("STRATO_PACKAGE_VERSION");
/// Version of this wrapper crate.
pub const WRAPPER_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Commit of the tree this module was built from (passed in by the build script).
pub const BUILD_COMMIT: &str = env!("STRATO_WRAPPER_COMMIT");
const BUILD_DIRTY: &str = env!("STRATO_WRAPPER_DIRTY");
/// The wasm-bindgen version this crate is pinned to (Cargo.toml).
pub const WASM_BINDGEN_VERSION: &str = "0.2.130";

/// Wrapper limit on the series (Method B): the most eigenmodes the wrapper lets
/// `ModalRelease::new` compute. The series keeps every mode up to `45 / t_min` (t_min the
/// earliest positive output time), and its cost grows with the square of the mode count. The
/// wrapper bounds the count from above before it calls the library (see
/// [`series_mode_bound`]) and leaves the series out, with a message, when the bound exceeds
/// this limit; the BTCS result is unaffected. Below the library's own cap
/// `MAX_MODES = 20000`, so the library's capping branch is never entered from the wrapper.
pub const MAX_SERIES_MODES: usize = 1_000;

/// `output.directory` is required by `Config` but used only by the command line. The wrapper
/// never writes a file; it fills this value when a request leaves the field out.
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

/// Writes a data file of a BTCS run (CSV time series, CSV profiles or the JSON of the run) with
/// its provenance header, by the library's export, and returns it as text in a reply.
#[wasm_bindgen]
pub fn export_run(request_json: &str) -> String {
    btcs_api::export_run(request_json)
}

// ---------------------------------------------------------------------------------------
// Build information
// ---------------------------------------------------------------------------------------

pub fn build_info_value() -> Value {
    json!({
        "abi": ABI_VERSION,
        "crate": "strato-wasm",
        "wrapper_version": WRAPPER_VERSION,
        "strato_version": STRATO_VERSION,
        "commit": BUILD_COMMIT,
        "dirty": BUILD_DIRTY == "1",
        "wasm_bindgen": WASM_BINDGEN_VERSION,
        "btcs": btcs_api::build_info_value(),
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

fn engine_value() -> Value {
    json!({"strato_version": STRATO_VERSION, "commit": BUILD_COMMIT, "abi": ABI_VERSION})
}

// ---------------------------------------------------------------------------------------
// Request parsing helpers shared by the interface
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
