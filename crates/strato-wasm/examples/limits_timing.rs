//! Native timing of `run_v1` at the wrapper limits (development aid, not a test). The limits
//! themselves are set from browser measurements recorded in prompt/PROGRESS.md; this example
//! gives the native times of the same configurations for comparison.
//!
//!     cargo run --release --manifest-path crates/strato-wasm/Cargo.toml --example limits_timing

use serde_json::{json, Value};
use std::time::Instant;

fn main() {
    let raw = std::fs::read_to_string("inputs/synthetic/synthetic_five_layer.toml")
        .expect("run from the repository root");
    let base: Value = toml::from_str(&raw).unwrap();
    for (name, cfg) in limits_cases(&base) {
        let req = json!({"schema": 1, "config": cfg}).to_string();
        let t0 = Instant::now();
        let out = strato_wasm::run_v1(&req);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        let v: Value = serde_json::from_str(&out).unwrap();
        let modes = v["diagnostics"]["code_c"]["modes"]
            .as_u64()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "none".into());
        println!(
            "{name:<34} status={:<8} cells={:<5} steps={:<6} rows={:<6} modes={:<5} bytes={:<9} ms={ms:.1}",
            v["status"].as_str().unwrap_or("?"),
            v["diagnostics"]["cells"],
            v["diagnostics"]["steps"],
            v["diagnostics"]["output_rows"],
            modes,
            out.len()
        );
        if v["status"] != "ok" {
            println!("    {}", v["errors"]);
        }
    }
}

/// The configurations measured in the browser by web/e2e/limits.spec.ts (same list).
pub fn limits_cases(base: &Value) -> Vec<(&'static str, Value)> {
    let set = |mut v: Value, edits: &[(&str, Value)]| {
        for (p, x) in edits {
            *v.pointer_mut(p).unwrap() = x.clone();
        }
        v
    };
    let cells = |n: u64| -> Vec<(&'static str, Value)> {
        vec![
            ("/regions/0/cells", json!(n)),
            ("/regions/1/cells", json!(n)),
            ("/regions/2/cells", json!(n)),
            ("/regions/3/cells", json!(n)),
            ("/regions/4/cells", json!(n)),
        ]
    };
    vec![
        ("default", base.clone()),
        ("max cells, default steps", set(base.clone(), &cells(400))),
        (
            "max steps, default cells",
            set(base.clone(), &[("/time/dt_s", json!(10.0))]),
        ),
        (
            "max cells and steps",
            set(
                set(base.clone(), &cells(400)),
                &[
                    ("/time/dt_s", json!(10.0)),
                    ("/time/output_every_steps", json!(50)),
                ],
            ),
        ),
        (
            "series near its mode limit",
            set(
                set(base.clone(), &cells(20)),
                &[
                    ("/time/dt_s", json!(0.5)),
                    ("/time/end_time_s", json!(1.0e4)),
                    ("/time/output_every_steps", json!(5)),
                ],
            ),
        ),
        (
            "max profile values",
            set(
                set(base.clone(), &cells(198)),
                &[
                    ("/time/dt_s", json!(10.0)),
                    ("/time/output_every_steps", json!(20)),
                ],
            ),
        ),
    ]
}
