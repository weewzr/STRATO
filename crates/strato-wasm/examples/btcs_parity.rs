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
//! The rules are those of `strato_wasm::parity` (read its header): the per-value rule of the
//! wrapper (`tolerances::PARITY_RTOL`, relative 1e-12) for every physical value, with the series'
//! own `accuracy_floor` added for the values of the independent Sturm-Liouville series
//! (`tolerances::SERIES_FLOOR_RULE`); the residual rule for the balance residuals (within the
//! derived bound on both platforms); the flag rule for the drawn-or-withheld flags at the margin
//! of the drawing mask. Every leaf of the reply is compared, including the diagnostics; nothing is
//! left out. Every leaf that fails is reported with its path (array positions shown as `[]`), the
//! number of failures, the first failing pair and the worst relative difference; each case line
//! says how many fields the residual rule and the flag rule cover and how many of them needed
//! the rule. The exit status is 1 when any value fails. The module must be built with the same
//! `STRATO_BUILD_COMMIT` as this example so that the build stamp in the replies agrees.
use serde_json::{json, Map, Value};
use std::{env, fs, process::ExitCode};
use strato_wasm::parity::{compare_case, kind_of};

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
            let (mut residual_total, mut flag_total) = ((0usize, 0usize), (0usize, 0usize));
            for (name, nv) in n {
                let Some(wv) = w.get(name) else {
                    println!("FAIL {name}: missing from the WebAssembly file");
                    any_fail = true;
                    continue;
                };
                let t = compare_case(name, nv, wv);
                let status = if t.failures.is_empty() {
                    "PASS"
                } else {
                    "FAIL"
                };
                println!(
                    "{status} {name}: {} values, {} bit-identical, worst relative difference {:e}; residual rule covers {} field(s) ({} needed it), flag rule covers {} field(s) ({} needed it)",
                    t.compared, t.identical, t.worst_rel, t.residual_covered, t.residual_excused, t.flag_covered, t.flag_excused
                );
                residual_total.0 += t.residual_covered;
                residual_total.1 += t.residual_excused;
                flag_total.0 += t.flag_covered;
                flag_total.1 += t.flag_excused;
                for (pat, f) in &t.failures {
                    let (a, b) = f.first.clone().unwrap_or_default();
                    let kind = kind_of(name, pat);
                    if kind == "series-derived" {
                        series_fail += 1;
                    } else {
                        solver_fail += 1;
                    }
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
                "RULES residual rule: {} field(s) covered over all cases, {} needed it; flag rule: {} field(s) covered, {} needed it",
                residual_total.0, residual_total.1, flag_total.0, flag_total.1
            );
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
