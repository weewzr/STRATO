//! Native run of the BTCS interface: reads a STRATO TOML input case and prints the `run` reply
//! for it as JSON on standard output. It is the native reference of the browser parity test
//! (test H of the directive: native versus WebAssembly, relative 1e-12 per value). The request
//! is built from the TOML exactly as the browser builds it from the generated case JSON: the
//! same keys, the same values.
//!
//!     cargo run --release --manifest-path crates/strato-wasm/Cargo.toml --example native_btcs_run [case.toml]

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
    let request = serde_json::json!({"schema": 2, "config": config});
    let out = strato_wasm::run(&request.to_string());
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
