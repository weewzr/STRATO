//! Test H of the directive (native versus WebAssembly parity) for the BTCS functions.
//!
//! Three subcommands, run from the repository root:
//!
//!     btcs_parity requests [NAME...]  prints the battery of requests as JSON:
//!                                     {name: {"function": "run" | ..., "input": "<text>"}}
//!     btcs_parity native [NAME...]    runs the battery natively and prints {name: reply}
//!                                     (with names, only those cases of the battery)
//!     btcs_parity compare N.json W.json
//!                                     compares a native reply file with a reply file produced
//!                                     by the WebAssembly module for the same requests
//!
//! The rule is the existing per-value rule of the wrapper (`tolerances::PARITY_RTOL`): every
//! number of the two replies must satisfy |a - b| <= rtol * max(|a|, |b|), array lengths and
//! object keys must be identical, and strings, booleans and nulls must be equal. One stated
//! addition applies to the values that come from the independent Sturm-Liouville series
//! (`analytical.*` and `diagnostics.analytical_comparison.*`, see `tolerances::SERIES_FLOOR_RULE`):
//! the series itself reports an absolute accuracy `accuracy_floor` (its own estimate, from the
//! cancellation in its shooting solution), and the native and WebAssembly builds use different
//! exp/cos/sin routines, so those values may differ by up to that floor in absolute terms. The
//! floor is read from the native reply of the same case and added to the allowance of those
//! values only; `accuracy_floor` and `min_flux_ratio` themselves, and every BTCS value, keep the
//! strict rule. The top-level `release_fraction`, `release_fraction_of_total`, `missing_weight`
//! and `withheld_reason_codes` count as series values only in replies of the `analytical`
//! function (cases named `analytical_*`); in the replies of `run`, `compare` and `run_stage` the
//! top-level `release_fraction` is the BTCS series and stays strict. Every leaf of
//! the reply is compared, including the diagnostics; nothing is left out. Every leaf that fails
//! is reported with its path (array positions shown as `[]`), the number of failures, the
//! first failing pair and the worst relative difference. The exit status is 1 when any value
//! fails. The module must be built with the same `STRATO_BUILD_COMMIT` as this example so that
//! the build stamp in the replies agrees.

use serde_json::{json, Map, Value};
use std::{collections::BTreeMap, env, fs, process::ExitCode};
use strato_wasm::tolerances::PARITY_RTOL;

const SYNTHETIC: &str = "inputs/synthetic/synthetic_five_layer.toml";

fn synthetic() -> Value {
    let raw = fs::read_to_string(SYNTHETIC).expect("run from the repository root");
    toml::from_str(&raw).expect("the synthetic case parses")
}

fn with(mut v: Value, pointer: &str, new: Value) -> Value {
    *v.pointer_mut(pointer).expect("pointer exists") = new;
    v
}

/// The battery: name, function, input text.
fn battery() -> Vec<(&'static str, &'static str, String)> {
    let base = synthetic();
    let req = |cfg: Value| json!({"schema": 2, "config": cfg});
    let mut out: Vec<(&'static str, &'static str, String)> = Vec::new();
    out.push((
        "validate_toml",
        "validate",
        fs::read_to_string(SYNTHETIC).unwrap(),
    ));
    out.push((
        "validate_rejected_dt",
        "validate",
        req(with(base.clone(), "/time/dt_s", json!(0.0))).to_string(),
    ));
    out.push(("run_synthetic", "run", req(base.clone()).to_string()));
    out.push((
        "run_profile_times",
        "run",
        json!({"schema": 2, "config": base.clone(),
               "profile_times_s": [0.0, 10000.0, 50000.0, 100000.0, 123456.7, 200000.0]})
        .to_string(),
    ));
    let coarse = with(
        with(
            base.clone(),
            "/regions",
            json!([
                {"name": "Kernel", "outer_radius_m": 2.0e-4, "diffusivity_m2_s": 1.0e-12, "intervals": 5},
                {"name": "Buffer", "outer_radius_m": 3.0e-4, "diffusivity_m2_s": 4.0e-10, "intervals": 5},
                {"name": "IPyC", "outer_radius_m": 3.5e-4, "diffusivity_m2_s": 8.0e-13, "intervals": 5},
                {"name": "SiC", "outer_radius_m": 4.0e-4, "diffusivity_m2_s": 2.0e-14, "intervals": 5},
                {"name": "OPyC", "outer_radius_m": 4.5e-4, "diffusivity_m2_s": 7.0e-13, "intervals": 5}
            ]),
        ),
        "/time/dt_s",
        json!(5000.0),
    );
    out.push((
        "run_coarse_large_dt",
        "run",
        req(coarse.clone()).to_string(),
    ));
    // A small comparison and its export, in the battery file that the browser parity test reads.
    let coarse_modified = with(
        coarse.clone(),
        "/regions/3/diffusivity_m2_s",
        json!(4.0e-14),
    );
    out.push((
        "compare_coarse_sic_diffusivity",
        "compare",
        json!({"schema": 2,
               "baseline": {"schema": 2, "config": coarse.clone()},
               "modified": {"schema": 2, "config": coarse_modified.clone()}})
        .to_string(),
    ));
    out.push((
        "export_comparison_csv",
        "export_run",
        json!({"schema": 2, "format": "comparison_csv",
               "config": coarse_modified, "baseline_config": coarse,
               "input_file": "synthetic_five_layer.toml",
               "parameter_source": "synthetic benchmark input with values changed by the user in the web controls",
               "parameter_class": "user-supplied value, no source class recorded"})
        .to_string(),
    ));
    let mut fine = base.clone();
    for r in fine["regions"].as_array_mut().unwrap() {
        r["cells"] = json!(100);
    }
    let fine = with(fine, "/time/dt_s", json!(100.0));
    out.push((
        "run_fine_500_intervals",
        "run",
        json!({"schema": 2, "config": fine, "profile_times_s": [50000.0, 200000.0]}).to_string(),
    ));
    let contrast = with(
        with(base.clone(), "/regions/1/diffusivity_m2_s", json!(1.0e-10)),
        "/regions/3/diffusivity_m2_s",
        json!(1.0e-14),
    );
    out.push(("run_contrast", "run", req(contrast).to_string()));
    out.push((
        "run_single_step",
        "run",
        req(with(base.clone(), "/time/dt_s", json!(200000.0))).to_string(),
    ));
    out.push((
        "compare_sic_diffusivity",
        "compare",
        json!({"schema": 2,
               "baseline": {"schema": 2, "config": base.clone()},
               "modified": {"schema": 2, "config": with(base.clone(), "/regions/3/diffusivity_m2_s", json!(4.0e-14))}})
        .to_string(),
    ));
    out.push((
        "run_rejected_steps",
        "run",
        req(with(base.clone(), "/time/dt_s", json!(1.0))).to_string(),
    ));
    for (name, format) in [
        ("export_timeseries_csv", "timeseries_csv"),
        ("export_profiles_csv", "profiles_csv"),
    ] {
        out.push((
            name,
            "export_run",
            json!({"schema": 2, "config": base.clone(), "format": format,
                   "input_file": "synthetic_five_layer.toml",
                   "parameter_source": "synthetic benchmark, not literature data",
                   "parameter_class": "synthetic benchmark"})
            .to_string(),
        ));
    }
    out.push(("build_info", "build_info", String::new()));
    out.extend(stage_battery(&base));
    out
}

/// Test inputs of the stage cases: numbers for the numerics, not register values.
fn stage_physics(stage: u8, temperature: Value) -> Value {
    let mut p = Map::new();
    if stage >= 2 {
        p.insert(
            "source".into(),
            json!({"mode": "kernel_fission_yield", "product": "direct_fission_product",
                   "yield_kind": "cumulative", "yield_atoms_per_fission": 0.06,
                   "fission_rate_origin": "parity test input",
                   "fission_rate_density": {"value": 1.0e19}}),
        );
    }
    if stage >= 3 {
        p.insert("decay".into(), json!({"half_life_s": 1.0e5}));
    }
    if stage >= 4 {
        p.insert("temperature".into(), temperature);
        let d0 = [1.0e-6, 2.0e-7, 5.0e-6, 3.0e-5, 8.0e-6];
        let ea = [200.0, 150.0, 220.0, 260.0, 210.0];
        let names = ["Kernel", "Buffer", "IPyC", "SiC", "OPyC"];
        let laws: Vec<Value> = (0..5)
            .map(|k| {
                json!({"region": names[k], "source": "parity test input, no source",
                       "class": "user-supplied, no source",
                       "terms": [{"d0_m2_s": d0[k], "activation_energy_kj_per_mol": ea[k],
                                  "valid_from_k": 1000.0, "valid_to_k": 1500.0}]})
            })
            .collect();
        p.insert("arrhenius".into(), Value::Array(laws));
    }
    Value::Object(p)
}

/// The cases of the functions of ABI 4 (schema 3). They are not in the committed file of the
/// small cases (`strato-webtools` names the cases it keeps); the whole battery is the input of
/// the native-versus-WebAssembly parity run.
fn stage_battery(base: &Value) -> Vec<(&'static str, &'static str, String)> {
    let ramp = json!({"unit": "degC", "interpolation": "linear",
                      "times_s": [0.0, 2.0e5], "values": [900.0, 1100.0]});
    let flat = json!({"unit": "degC", "value": 1000.0});
    let stage = |n: u8, t: Value| json!({"schema": 3, "config": base, "stage": n, "physics": stage_physics(n, t)});
    let mut small = base.clone();
    for k in 0..5 {
        small = with(small, &format!("/regions/{k}/cells"), json!(5));
    }
    small = with(
        small,
        "/time",
        json!({"dt_s": 2000.0, "end_time_s": 6000.0, "output_every_steps": 1}),
    );
    let mut sys = json!({"schema": 3, "config": small, "stage": 5,
                         "physics": stage_physics(5, flat.clone()), "step": 2});
    sys["rows"] = json!([
        "first",
        1,
        "interface:1",
        "interface:2",
        "interface:3",
        "interface:4",
        "last"
    ]);
    let mut ana1 = json!({"schema": 3, "config": base, "stage": 1});
    ana1["times_s"] = json!([1.0e4, 5.0e4, 1.0e5, 2.0e5]);
    let scen = |id: &str, nuclide: &str, grid: [u32; 5]| {
        json!({"schema": 3, "scenario_id": id, "nuclide": nuclide,
               "grid": {"intervals_per_layer": grid}, "check_resolution": true, "max_rows": 120})
    };
    vec![
        (
            "run_stage_1",
            "run_stage",
            json!({"schema": 3, "config": base, "stage": 1}).to_string(),
        ),
        (
            "run_stage_2",
            "run_stage",
            stage(2, ramp.clone()).to_string(),
        ),
        (
            "run_stage_3",
            "run_stage",
            stage(3, ramp.clone()).to_string(),
        ),
        (
            "run_stage_4_ramp",
            "run_stage",
            stage(4, ramp.clone()).to_string(),
        ),
        (
            "run_stage_5_constant",
            "run_stage",
            stage(5, flat.clone()).to_string(),
        ),
        (
            "run_stage_rejected",
            "run_stage",
            json!({"schema": 3, "config": base, "stage": 1,
                   "physics": stage_physics(2, ramp.clone())})
            .to_string(),
        ),
        ("btcs_system_step_2", "btcs_system", sys.to_string()),
        ("analytical_stage_1", "analytical", ana1.to_string()),
        (
            "analytical_stage_3",
            "analytical",
            stage(3, ramp.clone()).to_string(),
        ),
        (
            "analytical_stage_4_ramp",
            "analytical",
            stage(4, ramp).to_string(),
        ),
        (
            "analytical_stage_5_constant",
            "analytical",
            stage(5, flat).to_string(),
        ),
        (
            "run_scenario_steady_state_cs137",
            "run_scenario",
            scen("steady-state", "cs137", [40, 10, 10, 40, 10]).to_string(),
        ),
        (
            "run_scenario_shutdown_sr90",
            "run_scenario",
            scen("shutdown-cooldown", "sr90", [60, 10, 10, 40, 10]).to_string(),
        ),
        (
            "run_scenario_rejected",
            "run_scenario",
            json!({"schema": 3, "scenario_id": "steady-state", "nuclide": "ag110m"}).to_string(),
        ),
        ("options", "options", json!({"schema": 3}).to_string()),
    ]
}

/// The battery, or only the cases named on the command line.
fn selected(names: &[String]) -> Vec<(&'static str, &'static str, String)> {
    let all = battery();
    if names.is_empty() {
        return all;
    }
    for n in names {
        assert!(all.iter().any(|c| c.0 == n), "no case named {n}");
    }
    all.into_iter()
        .filter(|c| names.iter().any(|n| n == c.0))
        .collect()
}

fn call(function: &str, input: &str) -> String {
    match function {
        "validate" => strato_wasm::validate(input),
        "run" => strato_wasm::run(input),
        "compare" => strato_wasm::compare(input),
        "export_run" => strato_wasm::export_run(input),
        "build_info" => strato_wasm::build_info(),
        "run_stage" => strato_wasm::run_stage(input),
        "run_scenario" => strato_wasm::run_scenario(input),
        "btcs_system" => strato_wasm::btcs_system(input),
        "analytical" => strato_wasm::analytical(input),
        "options" => strato_wasm::options(input),
        other => panic!("unknown function {other}"),
    }
}

#[derive(Default)]
struct Failure {
    count: usize,
    first: Option<(String, String)>,
    worst_rel: f64,
}

#[derive(Default)]
struct Tally {
    /// The case name; it says which function produced the reply.
    case: String,
    compared: usize,
    identical: usize,
    worst_rel: f64,
    failures: BTreeMap<String, Failure>,
}

/// Fields that come from the independent Sturm-Liouville series, or from a difference with it.
/// The label only classifies a failure for the report; it never excuses one.
/// A `compare` reply holds two runs, `.baseline` and `.modified`, each with its own series.
fn is_series_derived(case: &str, pattern: &str) -> bool {
    let p = pattern
        .strip_prefix(".baseline")
        .or_else(|| pattern.strip_prefix(".modified"))
        .unwrap_or(pattern);
    // The top-level series fields are the series only in the reply of the `analytical` function
    // (its cases are named `analytical_*`). In the replies of `run`, `compare` and `run_stage`
    // the top-level `release_fraction[]` is the BTCS model series and stays on the bare rule.
    // `run_scenario` returns the series under `.method_b`; `run_stage` only says whether it applies.
    let analytical_reply = case.starts_with("analytical");
    p.starts_with(".analytical.")
        || p.starts_with(".diagnostics.analytical_comparison")
        || p.starts_with(".method_b.release_fraction")
        || p.starts_with(".method_b.missing_weight")
        || p.starts_with(".method_b.withheld_reason_codes")
        || (analytical_reply
            && (p.starts_with(".release_fraction[]")
                || p.starts_with(".release_fraction_of_total[]")
                || p.starts_with(".missing_weight")
                || p.starts_with(".withheld_reason_codes")))
}

/// The series accuracy floor of the native reply that a leaf belongs to: that of the run itself, or
/// for a `compare` reply that of the run (`.baseline` or `.modified`) the leaf lies in.
fn floor_at(native: &Value, path: &str) -> f64 {
    let run = if path.starts_with(".baseline") {
        &native["baseline"]
    } else if path.starts_with(".modified") {
        &native["modified"]
    } else {
        native
    };
    run["analytical"]["accuracy_floor"]
        .as_f64()
        .or_else(|| run["method_b"]["accuracy_floor"].as_f64())
        .or_else(|| run["accuracy_floor"].as_f64())
        .unwrap_or(0.0)
}

fn pattern(path: &str) -> String {
    let mut out = String::new();
    let mut in_index = false;
    for ch in path.chars() {
        match ch {
            '[' => {
                in_index = true;
                out.push_str("[]");
            }
            ']' => in_index = false,
            c if !in_index => out.push(c),
            _ => {}
        }
    }
    out
}

fn fail(t: &mut Tally, path: &str, a: String, b: String, rel: f64) {
    let f = t.failures.entry(pattern(path)).or_default();
    f.count += 1;
    f.worst_rel = f.worst_rel.max(rel);
    f.first.get_or_insert((a, b));
}

/// The absolute allowance added to the per-value rule for one leaf: the series accuracy floor of
/// the case for series-derived values, except for the two numbers that define the floor.
fn extra_allowance(case: &str, path: &str, native: &Value) -> f64 {
    let pat = pattern(path);
    if is_series_derived(case, &pat)
        && !pat.ends_with(".accuracy_floor")
        && !pat.ends_with(".min_flux_ratio")
    {
        floor_at(native, path)
    } else {
        0.0
    }
}

fn walk(path: &str, a: &Value, b: &Value, native: &Value, t: &mut Tally) {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (
                x.as_f64().unwrap_or(f64::NAN),
                y.as_f64().unwrap_or(f64::NAN),
            );
            t.compared += 1;
            if x.to_bits() == y.to_bits() || x == y {
                t.identical += 1;
                return;
            }
            let rel = (x - y).abs() / x.abs().max(y.abs());
            t.worst_rel = t.worst_rel.max(rel);
            if (x - y).abs() > PARITY_RTOL * x.abs().max(y.abs()) + extra_allowance(&t.case, path, native) {
                fail(t, path, format!("{x:e}"), format!("{y:e}"), rel);
            }
        }
        (Value::Array(x), Value::Array(y)) => {
            if x.len() != y.len() {
                fail(
                    t,
                    &format!("{path}.length"),
                    x.len().to_string(),
                    y.len().to_string(),
                    f64::INFINITY,
                );
            }
            for (i, (p, q)) in x.iter().zip(y).enumerate() {
                walk(&format!("{path}[{i}]"), p, q, native, t);
            }
        }
        (Value::Object(x), Value::Object(y)) => {
            for k in x.keys().chain(y.keys().filter(|k| !x.contains_key(*k))) {
                match (x.get(k), y.get(k)) {
                    (Some(p), Some(q)) => walk(&format!("{path}.{k}"), p, q, native, t),
                    _ => fail(
                        t,
                        &format!("{path}.{k} (key)"),
                        "present/absent".into(),
                        "absent/present".into(),
                        f64::INFINITY,
                    ),
                }
            }
        }
        (p, q) => {
            t.compared += 1;
            if p == q {
                t.identical += 1;
            } else {
                fail(t, path, p.to_string(), q.to_string(), f64::INFINITY);
            }
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("requests") => {
            let mut m = Map::new();
            for (name, function, input) in selected(&args[1..]) {
                m.insert(name.into(), json!({"function": function, "input": input}));
            }
            println!("{}", Value::Object(m));
            ExitCode::SUCCESS
        }
        Some("native") => {
            let mut m = Map::new();
            for (name, function, input) in selected(&args[1..]) {
                let text = call(function, &input);
                m.insert(
                    name.into(),
                    serde_json::from_str(&text).expect("reply is JSON"),
                );
            }
            println!("{}", Value::Object(m));
            ExitCode::SUCCESS
        }
        Some("compare") if args.len() == 3 => {
            let read = |p: &str| -> Value {
                let text = fs::read_to_string(p).unwrap_or_else(|e| panic!("reading {p}: {e}"));
                serde_json::from_str(text.trim_start_matches('\u{feff}'))
                    .unwrap_or_else(|e| panic!("parsing {p}: {e}"))
            };
            let (n, w) = (read(&args[1]), read(&args[2]));
            let (Some(n), Some(w)) = (n.as_object(), w.as_object()) else {
                eprintln!("both files must hold a JSON object");
                return ExitCode::FAILURE;
            };
            let mut any_fail = false;
            let (mut solver_fail, mut series_fail) = (0usize, 0usize);
            for (name, nv) in n {
                let Some(wv) = w.get(name) else {
                    println!("FAIL {name}: missing from the WebAssembly file");
                    any_fail = true;
                    continue;
                };
                let mut t = Tally {
                    case: name.clone(),
                    ..Tally::default()
                };
                walk("", nv, wv, nv, &mut t);
                let status = if t.failures.is_empty() {
                    "PASS"
                } else {
                    "FAIL"
                };
                println!(
                    "{status} {name}: {} values, {} bit-identical, worst relative difference {:e}",
                    t.compared, t.identical, t.worst_rel
                );
                for (pat, f) in &t.failures {
                    let (a, b) = f.first.clone().unwrap_or_default();
                    let kind = if is_series_derived(name, pat) {
                        series_fail += 1;
                        "series-derived"
                    } else {
                        solver_fail += 1;
                        "BTCS"
                    };
                    println!(
                        "    outside the rule [{kind}]: {pat}: {} value(s), worst relative {:e}, first native {a} wasm {b}",
                        f.count, f.worst_rel
                    );
                }
                any_fail |= !t.failures.is_empty();
            }
            for name in w.keys().filter(|k| !n.contains_key(*k)) {
                println!("FAIL {name}: missing from the native file");
                any_fail = true;
            }
            println!(
                "RESULT {} ({solver_fail} failing BTCS field(s), {series_fail} failing series-derived field(s))",
                if any_fail { "FAIL" } else { "PASS" }
            );
            if any_fail {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        _ => {
            eprintln!("usage: btcs_parity requests [NAME...] | native [NAME...] | compare NATIVE.json WASM.json");
            ExitCode::FAILURE
        }
    }
}
