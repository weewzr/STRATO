//! Native run of the wrapper: reads a STRATO TOML input case and prints the `run_v1`
//! response for it as JSON on standard output. `strato-webtools precomputed` uses it to write
//! web/public/data/wrapper_native_synthetic.json, the native reference for the browser parity
//! test (prompt/TESTING_AND_VALIDATION.md Section 2: browser wasm versus native, relative
//! 1e-12 per value). The request is built from the TOML exactly as the browser builds it from
//! the generated case JSON: the same keys, the same values.

use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "inputs/synthetic/synthetic_five_layer.toml".to_string());
    let raw = match fs::read_to_string(&path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("reading {path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let config: serde_json::Value = match toml::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("parsing {path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let request = serde_json::json!({"schema": 1, "config": config});
    let out = strato_wasm::run_v1(&request.to_string());
    let status = serde_json::from_str::<serde_json::Value>(&out)
        .ok()
        .and_then(|v| v["status"].as_str().map(str::to_string));
    println!("{out}");
    if status.as_deref() == Some("ok") {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
