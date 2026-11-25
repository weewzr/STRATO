//! Native checks of the BTCS interface of the wrapper (`validate`, `run`, `compare`):
//! - every series value, every profile and the final profile are bit-identical to
//!   `numerical::btcs::run` for the same configuration (same platform);
//! - the grid block is the library grid, with exact interface radii;
//! - the balance terms are the library's, the residual lies within the a-posteriori bound and
//!   the bound reproduces the values recorded in stage-b/btcs/solver-core-results.md;
//! - layer inventories sum to the inventory, shares are their quotients;
//! - the two expressions of the interface flux agree within the rounding-level tolerance;
//! - requested profile times map to the nearest step and are never interpolated;
//! - the analytical series is the library series at the same times;
//! - invalid input, placeholders, wrong schema, unsupported mechanisms and awkward numbers are
//!   rejected with messages, and nothing panics;
//! - `compare` returns modified minus baseline point by point.

use serde_json::{json, Value};
use std::panic::{catch_unwind, AssertUnwindSafe};
use strato::{
    analytical::sturm_liouville::ModalRelease,
    config::Config,
    numerical::{
        btcs::{run as btcs_run, Limits},
        diagnostics::StepDiagnostics,
        inventory::{inventory, inventory_by_region},
    },
    postprocessing::comparison::{layers_from_config, MAX_MODES},
};
use strato_wasm::tolerances::LAYER_SUM_RTOL;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");

fn path(rel: &str) -> String {
    format!("{ROOT}{rel}")
}

fn synthetic_toml() -> String {
    std::fs::read_to_string(path("inputs/synthetic/synthetic_five_layer.toml")).unwrap()
}

fn synthetic_config_value() -> Value {
    toml::from_str(&synthetic_toml()).unwrap()
}

fn request(config: Value) -> String {
    json!({"schema": 2, "config": config}).to_string()
}

fn run(req: &str) -> Value {
    serde_json::from_str(&strato_wasm::run(req)).unwrap()
}

fn run_ok(req: &str) -> Value {
    let v = run(req);
    assert_eq!(v["status"], "ok", "{v}");
    v
}

fn floats(v: &Value) -> Vec<f64> {
    v.as_array()
        .unwrap_or_else(|| panic!("not an array: {v}"))
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect()
}

fn with(mut v: Value, pointer: &str, new: Value) -> Value {
    *v.pointer_mut(pointer)
        .unwrap_or_else(|| panic!("no {pointer}")) = new;
    v
}

/// True when `x` rounds to `printed` at the three significant digits in which the results
/// file prints it (half a unit of the last printed digit; no tolerance is added).
fn equals_printed(x: f64, printed: f64) -> bool {
    let unit = 10f64.powi(printed.abs().log10().floor() as i32 - 2);
    (x - printed).abs() <= 0.5 * unit
}

/// A column of the output series: the library row to the value the reply must hold.
type Column = Box<dyn Fn(&StepDiagnostics) -> f64>;

const LIMITS: Limits = Limits {
    max_intervals_total: 100_000,
    max_steps: 10_000_000,
};

#[test]
fn run_is_bit_identical_to_the_library_solver() {
    let cfg = Config::load(path("inputs/synthetic/synthetic_five_layer.toml")).unwrap();
    // Every output row is also a snapshot step, so the library is asked for the same profiles.
    let reference = btcs_run(&cfg, &LIMITS, &[]).unwrap();
    let v = run_ok(&request(synthetic_config_value()));
    let t = floats(&v["times_s"]);
    assert_eq!(t.len(), reference.series.len());
    let cols: [(&str, Column); 7] = [
        ("release_fraction", Box::new(|d| d.release_fraction)),
        ("inventory_mol", Box::new(|d| d.inventory)),
        ("cumulative_release_mol", Box::new(|d| d.cumulative_release)),
        ("balance_residual_mol", Box::new(|d| d.balance_residual)),
        (
            "min_concentration_mol_m3",
            Box::new(|d| d.min_concentration),
        ),
        (
            "max_concentration_mol_m3",
            Box::new(|d| d.max_concentration),
        ),
        (
            "release_rate_mol_s",
            Box::new(|d| d.release_rate.unwrap_or(f64::NAN)),
        ),
    ];
    for (name, f) in &cols {
        let got = v[*name].as_array().unwrap();
        for (i, (ti, d)) in reference.series.iter().enumerate() {
            assert_eq!(t[i].to_bits(), ti.to_bits());
            let want = f(d);
            if want.is_nan() {
                // The release rate is not defined at j = 0 (Document 2, 7.2): null, not zero.
                assert!(got[i].is_null(), "{name}[{i}] = {}", got[i]);
            } else {
                assert_eq!(
                    got[i].as_f64().unwrap().to_bits(),
                    want.to_bits(),
                    "{name}[{i}]"
                );
            }
        }
    }
    assert!(v["release_rate_mol_s"][0].is_null());
    assert!(v["release_rate_mol_s"][1].as_f64().unwrap() > 0.0);
    let profiles = v["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), reference.series.len());
    let last = floats(&profiles.last().unwrap()["c_mol_per_m3"]);
    assert_eq!(last.len(), reference.final_profile.len());
    assert!(last
        .iter()
        .zip(&reference.final_profile)
        .all(|(a, b)| a.to_bits() == b.to_bits()));
    // The solver stamp.
    assert_eq!(v["label"], "COMPUTED v2.0");
    assert_eq!(v["solver"]["capability"], "diffusion_btcs_layered");
    assert_eq!(v["mechanisms"], json!(["diffusion_btcs_layered"]));
    assert_eq!(v["solver"]["version"], v["engine"]["strato_version"]);
}

#[test]
fn grid_block_is_the_library_grid() {
    let v = run_ok(&request(synthetic_config_value()));
    let g = &v["grid"];
    let r = floats(&g["node_radii_m"]);
    assert_eq!(g["nodes"], 201);
    assert_eq!(g["intervals_total"], 200);
    assert_eq!(r.len(), 201);
    assert_eq!(r[0], 0.0);
    assert!(r.windows(2).all(|w| w[0] < w[1]));
    let radii: [f64; 5] = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4];
    // Interface and outer radii are assigned, never accumulated: equal to the inputs bit for bit.
    let ir = floats(&g["interface_radii_m"]);
    for (a, b) in ir.iter().zip(&radii) {
        assert_eq!(a.to_bits(), b.to_bits());
    }
    assert_eq!(r[200].to_bits(), radii[4].to_bits());
    assert_eq!(g["interface_nodes"], json!([40, 80, 120, 160]));
    assert_eq!(g["intervals_per_layer"], json!([40, 40, 40, 40, 40]));
    // Node classes: one centre, four interfaces, one outer node.
    let class: Vec<&str> = g["node_class"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    assert_eq!(class[0], "centre");
    assert_eq!(class[200], "outer");
    assert_eq!(class.iter().filter(|c| **c == "interface").count(), 4);
    for (k, &node) in [40usize, 80, 120, 160].iter().enumerate() {
        assert_eq!(class[node], "interface");
        assert_eq!(g["node_layer_inner"][node], k);
        assert_eq!(g["node_layer_outer"][node], k + 1);
    }
    // Ring edges: N + 2 values, increasing from 0 to the outer radius.
    let e = floats(&g["ring_edges_m"]);
    assert_eq!(e.len(), 202);
    assert_eq!(e[0], 0.0);
    assert_eq!(e[201].to_bits(), radii[4].to_bits());
    assert!(e.windows(2).all(|w| w[0] < w[1]));
    assert_eq!(
        g["layer_names"],
        json!(["Kernel", "Buffer", "IPyC", "SiC", "OPyC"])
    );
}

#[test]
fn balance_terms_and_residual_bound() {
    let v = run_ok(&request(synthetic_config_value()));
    let m0 = v["balance"]["start_inventory_mol"].as_f64().unwrap();
    let inv = floats(&v["inventory_mol"]);
    let rel = floats(&v["cumulative_release_mol"]);
    let res = floats(&v["balance_residual_mol"]);
    let bound = floats(&v["balance_residual_bound_mol"]);
    assert_eq!(inv[0].to_bits(), m0.to_bits());
    for i in 0..inv.len() {
        // Equation F30, as the library computes it.
        assert_eq!(
            res[i].to_bits(),
            (inv[i] + rel[i] - m0).to_bits(),
            "row {i}"
        );
        assert!(
            res[i].abs() <= bound[i],
            "row {i}: {} > {}",
            res[i],
            bound[i]
        );
    }
    let b = &v["balance"];
    assert_eq!(b["generated_mol"], 0.0);
    assert_eq!(b["decayed_mol"], 0.0);
    assert_eq!(b["tolerance"]["within_bound"], true);
    let last = res.len() - 1;
    assert_eq!(
        b["final"]["residual_mol"].as_f64().unwrap().to_bits(),
        res[last].to_bits()
    );
    assert_eq!(
        b["final"]["released_mol"].as_f64().unwrap().to_bits(),
        rel[last].to_bits()
    );
    let max_rel = b["max_abs_residual_relative"].as_f64().unwrap();
    assert!(max_rel <= b["tolerance"]["final_bound_relative"].as_f64().unwrap());
    // The residual is the rounding quantity recorded for this case (n = 40, dt = 200 s):
    // worst |eps|/M0 = 9.45e-11 and final bound 2.62e-8 (solver-core-results.md, section F).
    assert!(equals_printed(max_rel, 9.45e-11), "{max_rel:e}");
    let fb = b["tolerance"]["final_bound_relative"].as_f64().unwrap();
    assert!(equals_printed(fb, 2.62e-8), "{fb:e}");
}

#[test]
fn the_bound_reproduces_the_recorded_values_at_n20() {
    // Recorded for n = 20, dt = 200 s: worst |eps|/M0 = 2.30e-11, final bound 6.56e-9.
    let mut cfg = synthetic_config_value();
    for r in cfg["regions"].as_array_mut().unwrap() {
        r["cells"] = json!(20);
    }
    let v = run_ok(&request(cfg));
    let b = &v["balance"];
    let max_rel = b["max_abs_residual_relative"].as_f64().unwrap();
    let fb = b["tolerance"]["final_bound_relative"].as_f64().unwrap();
    assert!(equals_printed(max_rel, 2.30e-11), "{max_rel:e}");
    assert!(equals_printed(fb, 6.56e-9), "{fb:e}");
}

#[test]
fn layer_inventories_shares_and_the_library_quadrature() {
    let cfg = Config::load(path("inputs/synthetic/synthetic_five_layer.toml")).unwrap();
    let reference = btcs_run(&cfg, &LIMITS, &[100_000.0]).unwrap();
    let v = run_ok(&request(synthetic_config_value()));
    let profiles = v["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 101);
    for p in profiles {
        let inv = p["inventory_mol"].as_f64().unwrap();
        let layers = floats(&p["layer_inventory_mol"]);
        let share = floats(&p["layer_share"]);
        assert_eq!(layers.len(), 5);
        let sum: f64 = layers.iter().sum();
        assert!(
            (sum - inv).abs() <= LAYER_SUM_RTOL * inv,
            "t = {}: {sum} vs {inv}",
            p["t_s"]
        );
        for (l, s) in layers.iter().zip(&share) {
            assert_eq!(s.to_bits(), (l / inv).to_bits());
        }
    }
    // The profile at 1e5 s is the library snapshot, and its layers are the library quadrature.
    let at = profiles
        .iter()
        .find(|p| p["t_s"].as_f64().unwrap() == 100_000.0)
        .unwrap();
    let c = floats(&at["c_mol_per_m3"]);
    let (_, want) = &reference.snapshots[0];
    assert!(c.iter().zip(want).all(|(a, b)| a.to_bits() == b.to_bits()));
    let layers = floats(&at["layer_inventory_mol"]);
    let lib = inventory_by_region(&reference.grid, want);
    assert!(layers
        .iter()
        .zip(&lib)
        .all(|(a, b)| a.to_bits() == b.to_bits()));
    assert_eq!(
        at["inventory_mol"].as_f64().unwrap().to_bits(),
        inventory(&reference.grid, want).to_bits()
    );
    assert!(
        v["diagnostics"]["layer_sum_max_relative_difference"]
            .as_f64()
            .unwrap()
            <= LAYER_SUM_RTOL
    );
}

#[test]
fn interface_fluxes_agree_within_the_rounding_tolerance() {
    let v = run_ok(&request(synthetic_config_value()));
    let profiles = v["profiles"].as_array().unwrap();
    assert!(
        profiles[0]["interface_flux"].is_null(),
        "no step before j = 0"
    );
    let mut seen = 0;
    for p in &profiles[1..] {
        let f = &p["interface_flux"];
        assert_eq!(f["within_bound"], true, "t = {}", p["t_s"]);
        assert_eq!(f["interface_nodes"], json!([40, 80, 120, 160]));
        let a = floats(&f["outward_from_inner_expression_mol_s"]);
        let b = floats(&f["outward_from_outer_expression_mol_s"]);
        let d = floats(&f["difference_mol_s"]);
        let tol = floats(&f["difference_bound_mol_s"]);
        for k in 0..4 {
            assert_eq!(d[k].to_bits(), (a[k] - b[k]).abs().to_bits());
            assert!(d[k] <= tol[k]);
        }
        seen += 1;
    }
    assert_eq!(seen, 100);
    let diag = &v["diagnostics"]["interface_flux"];
    assert_eq!(diag["profiles_with_flux"], 100);
    assert_eq!(diag["all_within_bound"], true);
    // At the last step every flux through an interface is outward (positive).
    let last = &profiles[100]["interface_flux"];
    assert!(floats(&last["outward_from_inner_expression_mol_s"])
        .iter()
        .all(|x| *x > 0.0));
}

#[test]
fn requested_profile_times_use_the_nearest_step_and_are_never_interpolated() {
    let req = json!({"schema": 2, "config": synthetic_config_value(),
        "profile_times_s": [0.0, 50.0, 10_000.0, 123_456.7, 200_000.0, 999_999.0, 10_050.0]})
    .to_string();
    let v = run_ok(&req);
    assert_eq!(v["profile_times_requested"], true);
    let profiles = v["profiles"].as_array().unwrap();
    // dt = 200 s: 0 and 50 map to step 0; 10000 and 10050 map to steps 50 and 50 (10050/200 =
    // 50.25 rounds to 50); 123456.7 to step 617; 200000 and 999999 to step 1000.
    let steps: Vec<u64> = profiles
        .iter()
        .map(|p| p["step"].as_u64().unwrap())
        .collect();
    assert_eq!(steps, vec![0, 50, 617, 1000]);
    let t: Vec<f64> = profiles
        .iter()
        .map(|p| p["t_s"].as_f64().unwrap())
        .collect();
    assert_eq!(t, vec![0.0, 10_000.0, 123_400.0, 200_000.0]);
    assert_eq!(profiles[0]["requested_times_s"], json!([0.0, 50.0]));
    assert_eq!(
        profiles[1]["requested_times_s"],
        json!([10_000.0, 10_050.0])
    );
    assert_eq!(
        profiles[3]["requested_times_s"],
        json!([200_000.0, 999_999.0])
    );
    // The series rows are those of output_every_steps, independent of the profile times.
    assert_eq!(v["times_s"].as_array().unwrap().len(), 101);
    // Too many times, bad times.
    let many: Vec<f64> = (0..1001).map(|i| i as f64).collect();
    for (times, needle) in [
        (json!(many), "MAX_PROFILE_TIMES"),
        (json!([-1.0]), "non-negative"),
    ] {
        let r = json!({"schema": 2, "config": synthetic_config_value(), "profile_times_s": times})
            .to_string();
        let out: Value = serde_json::from_str(&strato_wasm::run(&r)).unwrap();
        assert_eq!(out["status"], "rejected", "{out}");
        assert!(out["errors"][0].as_str().unwrap().contains(needle));
    }
    // The values limit counts the profiles returned: 101 profiles of 201 nodes are fine; 1000
    // distinct profiles of 2001 nodes are not.
    let mut big = synthetic_config_value();
    for r in big["regions"].as_array_mut().unwrap() {
        r["cells"] = json!(400);
    }
    let times: Vec<f64> = (0..600).map(|i| i as f64 * 200.0).collect();
    let r = json!({"schema": 2, "config": big, "profile_times_s": times}).to_string();
    let out: Value = serde_json::from_str(&strato_wasm::run(&r)).unwrap();
    assert_eq!(out["status"], "rejected");
    assert!(out["errors"][0]
        .as_str()
        .unwrap()
        .contains("MAX_PROFILE_VALUES"));
}

#[test]
fn the_series_is_the_library_series_and_the_comparison_is_reported() {
    let cfg = Config::load(path("inputs/synthetic/synthetic_five_layer.toml")).unwrap();
    let v = run_ok(&request(synthetic_config_value()));
    let a = &v["analytical"];
    assert_eq!(a["applicable"], true);
    let times = floats(&a["times_s"]);
    let f = floats(&a["release_fraction"]);
    let t_min = floats(&v["times_s"])[1];
    let series = ModalRelease::new(&layers_from_config(&cfg), 45.0 / t_min, MAX_MODES).unwrap();
    for (t, x) in times.iter().zip(&f) {
        assert_eq!(x.to_bits(), series.release_fraction(*t).to_bits());
    }
    assert_eq!(
        a["accuracy_floor"].as_f64().unwrap().to_bits(),
        series.accuracy_floor().to_bits()
    );
    // The largest difference is the largest |F_BTCS - F_series| over the resolved rows.
    let all_t = floats(&v["times_s"]);
    let all_f = floats(&v["release_fraction"]);
    let mut worst = 0.0f64;
    for (t, x) in times.iter().zip(&f) {
        let i = all_t.iter().position(|u| u == t).unwrap();
        worst = worst.max((all_f[i] - x).abs());
    }
    let cmp = &v["diagnostics"]["analytical_comparison"];
    assert_eq!(
        cmp["max_abs_difference"].as_f64().unwrap().to_bits(),
        worst.to_bits()
    );
    assert_eq!(cmp["resolved"], true);
    // A tiny first output time is refused before the series is computed, and the BTCS result
    // is still returned.
    let tiny = with(synthetic_config_value(), "/time/end_time_s", json!(1e-300));
    let v = run(&request(tiny));
    if v["status"] == "ok" {
        assert!(v["analytical"].is_null());
        assert!(v["messages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m.as_str().unwrap().contains("MAX_SERIES_MODES")));
    }
}

#[test]
fn validate_reports_plan_grid_and_limits() {
    let v: Value = serde_json::from_str(&strato_wasm::validate(&synthetic_toml())).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["plan"]["intervals_total"], 200);
    assert_eq!(v["plan"]["nodes"], 201);
    assert_eq!(v["plan"]["steps"], 1000);
    assert_eq!(v["plan"]["output_rows"], 101);
    assert_eq!(v["grid"]["interface_nodes"], json!([40, 80, 120, 160]));
    assert_eq!(v["limits"]["min_intervals_per_layer"], 1);
    assert_eq!(v["limits"]["max_total_intervals"], 2000);
    // As JSON, bare and as a request; the same plan.
    let bare = synthetic_config_value().to_string();
    let w: Value = serde_json::from_str(&strato_wasm::validate(&bare)).unwrap();
    assert_eq!(w["plan"], v["plan"]);
    let w: Value =
        serde_json::from_str(&strato_wasm::validate(&request(synthetic_config_value()))).unwrap();
    assert_eq!(w["plan"], v["plan"]);
    // `intervals` and `cells` are the same number; both in one region is an error.
    let mut cfg = synthetic_config_value();
    for r in cfg["regions"].as_array_mut().unwrap() {
        let c = r.as_object_mut().unwrap().remove("cells").unwrap();
        r["intervals"] = c;
    }
    let w: Value = serde_json::from_str(&strato_wasm::validate(&cfg.to_string())).unwrap();
    assert_eq!(w["plan"], v["plan"]);
    let echo = run_ok(&request(cfg.clone()));
    assert_eq!(echo["inputs"]["regions"][0]["intervals"], 40);
    cfg["regions"][0]["cells"] = json!(40);
    let w: Value = serde_json::from_str(&strato_wasm::validate(&cfg.to_string())).unwrap();
    assert_eq!(w["ok"], false);
    assert!(w["errors"][0].as_str().unwrap().contains("both"));
    // A time step that does not divide the end time is used as end / ceil(end / dt), and the
    // warning says so.
    let w: Value = serde_json::from_str(&strato_wasm::validate(
        &with(synthetic_config_value(), "/time/dt_s", json!(300.0)).to_string(),
    ))
    .unwrap();
    assert_eq!(w["ok"], true);
    assert_eq!(w["plan"]["steps"], 667);
    assert!(w["warnings"][0].as_str().unwrap().contains("will use dt"));
}

fn assert_rejected(req: &str, needle: &str) {
    let out = catch_unwind(AssertUnwindSafe(|| strato_wasm::run(req)));
    let text = out.unwrap_or_else(|_| panic!("run panicked on {req}"));
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["status"], "rejected", "{v}");
    let errors = v["errors"].as_array().unwrap();
    assert!(
        errors.iter().any(|e| e.as_str().unwrap().contains(needle)),
        "expected '{needle}' in {errors:?}"
    );
    // validate agrees for the same request.
    let val: Value = serde_json::from_str(&strato_wasm::validate(req)).unwrap();
    assert_eq!(val["ok"], false, "validate accepted {req}");
}

#[test]
fn invalid_inputs_are_rejected_with_messages() {
    let base = synthetic_config_value();
    let cases: Vec<(Value, &str)> = vec![
        (
            with(base.clone(), "/regions", json!([])),
            "exactly five physical regions",
        ),
        (
            with(base.clone(), "/species", json!("  ")),
            "species must not be empty",
        ),
        (
            with(base.clone(), "/temperature_k", json!(0.0)),
            "temperature_k",
        ),
        (
            with(base.clone(), "/regions/2/outer_radius_m", json!(3.0e-4)),
            "invalid region IPyC",
        ),
        (
            with(base.clone(), "/regions/1/diffusivity_m2_s", json!(0.0)),
            "invalid region Buffer",
        ),
        (
            with(base.clone(), "/regions/4/cells", json!(0)),
            "invalid region OPyC",
        ),
        (with(base.clone(), "/regions/4/cells", json!(-3)), "config:"),
        (
            with(base.clone(), "/regions/4/cells", json!(2.5)),
            "config:",
        ),
        (
            with(base.clone(), "/time/dt_s", json!(0.0)),
            "invalid time controls",
        ),
        (
            with(base.clone(), "/time/output_every_steps", json!(0)),
            "invalid time controls",
        ),
        (
            with(
                base.clone(),
                "/initial/kernel_concentration_mol_m3",
                json!(-1.0),
            ),
            "nonnegative",
        ),
        (
            with(
                base.clone(),
                "/initial/kernel_concentration_mol_m3",
                json!(0.0),
            ),
            "kernel concentration must be finite and positive",
        ),
        (
            with(base.clone(), "/regions/0/cells", json!(2001)),
            "MAX_TOTAL_INTERVALS",
        ),
        (
            with(
                base.clone(),
                "/regions/0/cells",
                json!(18446744073709551615u64),
            ),
            "MAX_TOTAL_INTERVALS",
        ),
        (with(base.clone(), "/time/dt_s", json!(1.0)), "MAX_STEPS"),
        (with(base.clone(), "/time/dt_s", json!(5e-324)), "MAX_STEPS"),
        (
            with(
                with(base.clone(), "/time/dt_s", json!(10.0)),
                "/time/output_every_steps",
                json!(1),
            ),
            "MAX_PROFILE_VALUES",
        ),
    ];
    for (cfg, needle) in cases {
        assert_rejected(&request(cfg), needle);
    }
    let mut missing = synthetic_config_value();
    missing.as_object_mut().unwrap().remove("temperature_k");
    assert_rejected(&request(missing), "missing field `temperature_k`");
    assert_rejected("not json", "request:");
    assert_rejected("{}", "request:");
    assert_rejected(
        &json!({"schema": 1, "config": base.clone()}).to_string(),
        "unsupported request schema 1",
    );
    assert_rejected(
        &json!({"schema": 2, "config": base.clone(), "extra": 1}).to_string(),
        "unknown field",
    );
}

#[test]
fn placeholder_inputs_are_rejected() {
    let req = request(with(
        synthetic_config_value(),
        "/regions/0/diffusivity_m2_s",
        json!("INSERT_VALUE_HERE"),
    ));
    assert_rejected(&req, "INSERT_VALUE_HERE");
}

#[test]
fn only_the_btcs_capability_is_accepted() {
    for m in [
        "diffusion_layered",
        "decay",
        "generation",
        "diffusivity_temperature",
        "",
    ] {
        let req = json!({"schema": 2, "config": synthetic_config_value(),
            "mechanisms": ["diffusion_btcs_layered", m]})
        .to_string();
        assert_rejected(&req, "is not supported by this solver build");
    }
    let ok = json!({"schema": 2, "config": synthetic_config_value(),
        "mechanisms": ["diffusion_btcs_layered"]})
    .to_string();
    assert_eq!(run_ok(&ok)["mechanisms"], json!(["diffusion_btcs_layered"]));
}

#[test]
fn awkward_inputs_never_panic() {
    let base = synthetic_config_value();
    let cases = vec![
        with(base.clone(), "/regions/1/diffusivity_m2_s", json!(1e300)),
        with(base.clone(), "/regions/1/diffusivity_m2_s", json!(1e-300)),
        with(base.clone(), "/regions/0/outer_radius_m", json!(1e-300)),
        with(base.clone(), "/regions/4/outer_radius_m", json!(1e300)),
        with(
            base.clone(),
            "/initial/kernel_concentration_mol_m3",
            json!(1e300),
        ),
        with(
            base.clone(),
            "/initial/kernel_concentration_mol_m3",
            json!(1e-320),
        ),
        with(base.clone(), "/time/end_time_s", json!(1e-300)),
        with(
            base.clone(),
            "/time/output_every_steps",
            json!(18446744073709551615u64),
        ),
        with(
            with(base.clone(), "/time/dt_s", json!(1e300)),
            "/time/end_time_s",
            json!(1e300),
        ),
        with(base.clone(), "/regions/2/cells", json!(1)),
    ];
    for cfg in cases {
        let req = request(cfg);
        let out = catch_unwind(AssertUnwindSafe(|| strato_wasm::run(&req)));
        let text = out.unwrap_or_else(|_| panic!("panicked on {req}"));
        let v: Value = serde_json::from_str(&text).unwrap();
        assert!(v["status"] == "ok" || v["status"] == "rejected", "{v}");
        let val = catch_unwind(AssertUnwindSafe(|| strato_wasm::validate(&req)));
        assert!(val.is_ok(), "validate panicked on {req}");
    }
}

#[test]
fn the_smallest_grid_runs() {
    // Every layer has one interval: the grid of the hypotheses (H) with every n >= 1.
    let mut cfg = synthetic_config_value();
    for r in cfg["regions"].as_array_mut().unwrap() {
        r["cells"] = json!(1);
    }
    let v = run_ok(&request(cfg));
    assert_eq!(v["grid"]["nodes"], 6);
    assert_eq!(v["balance"]["tolerance"]["within_bound"], true);
    assert_eq!(v["diagnostics"]["non_negative"], true);
}

#[test]
fn compare_returns_modified_minus_baseline() {
    let base = synthetic_config_value();
    let modified = with(base.clone(), "/regions/3/diffusivity_m2_s", json!(4.0e-14));
    let req = json!({"schema": 2,
        "baseline": {"schema": 2, "config": base},
        "modified": {"schema": 2, "config": modified}})
    .to_string();
    let v: Value = serde_json::from_str(&strato_wasm::compare(&req)).unwrap();
    assert_eq!(v["status"], "ok", "{v}");
    for name in [
        "release_fraction",
        "inventory_mol",
        "cumulative_release_mol",
    ] {
        let b = floats(&v["baseline"][name]);
        let m = floats(&v["modified"][name]);
        let d = floats(&v["difference"][name]);
        assert_eq!(d.len(), b.len());
        for i in 0..d.len() {
            assert_eq!(d[i].to_bits(), (m[i] - b[i]).to_bits(), "{name}[{i}]");
        }
    }
    let rate = v["difference"]["release_rate_mol_s"].as_array().unwrap();
    assert!(
        rate[0].is_null(),
        "the release rate is not defined at j = 0"
    );
    assert!(rate[1].as_f64().is_some());
    let d = floats(&v["difference"]["release_fraction"]);
    assert!(*d.last().unwrap() > 0.0, "a faster SiC releases more");
    assert_eq!(
        v["summary"]["max_abs_release_fraction_difference"]
            .as_f64()
            .unwrap()
            .to_bits(),
        d.iter().fold(0.0f64, |a, x| a.max(x.abs())).to_bits()
    );
    // Exactly one input differs, and the page is told which.
    let changes = v["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0]["path"], "/regions/3/diffusivity_m2_s");
    assert_eq!(changes[0]["baseline"], 2.0e-14);
    assert_eq!(changes[0]["modified"], 4.0e-14);
    // Different time grids are rejected, never interpolated.
    let other = with(
        synthetic_config_value(),
        "/time/output_every_steps",
        json!(20),
    );
    let req = json!({"schema": 2,
        "baseline": {"schema": 2, "config": synthetic_config_value()},
        "modified": {"schema": 2, "config": other}})
    .to_string();
    let v: Value = serde_json::from_str(&strato_wasm::compare(&req)).unwrap();
    assert_eq!(v["status"], "rejected");
    // A rejected half names itself.
    let bad = with(synthetic_config_value(), "/time/dt_s", json!(0.0));
    let req = json!({"schema": 2,
        "baseline": {"schema": 2, "config": synthetic_config_value()},
        "modified": {"schema": 2, "config": bad}})
    .to_string();
    let v: Value = serde_json::from_str(&strato_wasm::compare(&req)).unwrap();
    assert_eq!(v["status"], "rejected");
    assert!(v["errors"][0].as_str().unwrap().starts_with("modified: "));
    // The finite-volume request schema is not accepted.
    let req = json!({"schema": 1,
        "baseline": {"schema": 1, "config": synthetic_config_value()},
        "modified": {"schema": 1, "config": synthetic_config_value()}})
    .to_string();
    let v: Value = serde_json::from_str(&strato_wasm::compare(&req)).unwrap();
    assert_eq!(v["status"], "rejected");
}

#[test]
fn build_info_reports_abi_2_and_the_btcs_capability() {
    let v: Value = serde_json::from_str(&strato_wasm::build_info()).unwrap();
    assert_eq!(v["abi"], 2);
    assert_eq!(strato_wasm::abi_version(), 2);
    let b = &v["btcs"];
    assert_eq!(b["capabilities"], json!(["diffusion_btcs_layered"]));
    assert_eq!(b["label"], "COMPUTED v2.0");
    assert_eq!(b["schema"], 2);
    assert_eq!(b["functions"], json!(["validate", "run", "compare"]));
    assert_eq!(b["solver"]["id"], "btcs_five_layer");
    assert_eq!(b["solver"]["version"], v["strato_version"]);
    assert_eq!(b["limits"]["max_total_intervals"], 2000);
    // The finite-volume functions stay, and say they are the legacy ones.
    assert_eq!(v["capabilities"], json!(["diffusion_layered"]));
    assert!(v["legacy_finite_volume"]["functions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f == "run_v1"));
}
