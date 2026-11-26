//! Native timing of the stage and scenario functions at their limits (development aid, not a
//! test). The wrapper limits of `run_scenario` are set from the work of a call (grid nodes times
//! time steps); this example prints the native time of the largest calls.
//!
//!     cargo run --release --manifest-path crates/strato-wasm/Cargo.toml --example stage_timing

use serde_json::{json, Value};
use std::time::Instant;

fn timed(name: &str, f: fn(&str) -> String, req: &Value) {
    let t0 = Instant::now();
    let out = f(&req.to_string());
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    let v: Value = serde_json::from_str(&out).unwrap();
    println!(
        "{name:<46} status={:<8} bytes={:<9} ms={ms:.1}",
        v["status"].as_str().unwrap_or("?"),
        out.len()
    );
    if v["status"] != "ok" {
        println!("    {}", v["errors"]);
    }
}

fn main() {
    timed(
        "options (first call: parses the registers)",
        strato_wasm::options,
        &json!({}),
    );
    timed("options (cached)", strato_wasm::options, &json!({}));
    for (id, nuclide, variant) in [
        ("steady-state", "sr90", ""),
        ("steady-state", "cs137", ""),
        ("shutdown-cooldown", "sr90", ""),
        ("refuelling", "sr90", "heating-1700"),
        ("steady-state", "sr90", "cycles"),
    ] {
        let mut r = json!({"schema": 3, "scenario_id": id, "nuclide": nuclide});
        if !variant.is_empty() {
            r["variant"] = json!(variant);
        }
        timed(
            &format!("run_scenario {id} {nuclide} {variant} (default grid)"),
            strato_wasm::run_scenario,
            &r,
        );
        r["check_resolution"] = json!(true);
        timed(
            &format!("run_scenario {id} {nuclide} {variant} + resolution"),
            strato_wasm::run_scenario,
            &r,
        );
    }
}
