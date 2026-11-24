//! Native checks of the wrapper (prompt/TESTING_AND_VALIDATION.md Section 3, row 3):
//! - the wrapper's time series and final profile are bit-identical to `finite_volume::run`
//!   for the same configuration (same platform);
//! - they equal tests/golden/v1.0/ to the regression tolerance (relative 1e-12);
//! - the series (Code C) and its difference from Code B equal
//!   tests/golden/v1.0/comparison_fv_vs_analytical.csv;
//! - the balance residual is reported and no worse than the v1.0 level;
//! - the layer inventories of every profile sum to its inventory within 1e-12 relative;
//! - invalid and placeholder inputs are rejected with messages, and no input panics;
//! - a request naming a mechanism outside the capabilities is rejected;
//! - `compare_v1` returns modified minus baseline point by point.

use serde_json::{json, Value};
use std::panic::{catch_unwind, AssertUnwindSafe};
use strato::{config::Config, numerical::finite_volume as fv};
use strato_wasm::tolerances::{GOLDEN_RTOL, LAYER_SUM_RTOL, V1_BALANCE_REL_LEVEL};

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
    json!({"schema": 1, "config": config}).to_string()
}

fn run(req: &str) -> Value {
    serde_json::from_str(&strato_wasm::run_v1(req)).unwrap()
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

fn read_rows(name: &str) -> Vec<Vec<f64>> {
    let mut r = csv::Reader::from_path(path(&format!("tests/golden/v1.0/{name}"))).unwrap();
    r.records()
        .map(|rec| {
            rec.unwrap()
                .iter()
                .map(|s| {
                    if s.is_empty() {
                        f64::NAN
                    } else {
                        s.parse().unwrap()
                    }
                })
                .collect()
        })
        .collect()
}

/// The comparison rule of tests/phase1_regression.rs.
fn close(a: f64, b: f64, scale: f64) -> bool {
    (a - b).abs() <= GOLDEN_RTOL * scale.max(a.abs()).max(b.abs())
}

#[test]
fn time_series_is_bit_identical_to_finite_volume_run() {
    let cfg = Config::load(path("inputs/synthetic/synthetic_five_layer.toml")).unwrap();
    let mesh = fv::build_mesh(&cfg).unwrap();
    let (rows, final_c) = fv::run(&cfg, &mesh).unwrap();

    // The snapshot loop itself.
    let snap = strato_wasm::run_with_profiles(&cfg, &mesh).unwrap();
    assert_eq!(snap.rows.len(), rows.len());
    for ((ta, a), (tb, b)) in snap.rows.iter().zip(&rows) {
        assert_eq!(ta.to_bits(), tb.to_bits());
        for (x, y) in [
            (a.inventory, b.inventory),
            (a.cumulative_release, b.cumulative_release),
            (a.release_fraction, b.release_fraction),
            (a.release_rate, b.release_rate),
            (a.balance_residual, b.balance_residual),
            (a.min_concentration, b.min_concentration),
        ] {
            assert_eq!(x.to_bits(), y.to_bits(), "t = {ta}");
        }
    }
    let last = snap.profiles.last().unwrap();
    assert_eq!(last.len(), final_c.len());
    assert!(last
        .iter()
        .zip(&final_c)
        .all(|(a, b)| a.to_bits() == b.to_bits()));

    // The same through the JSON boundary: serde_json writes the shortest text that reads
    // back to the same double, so the response carries the identical values.
    let v = run_ok(&request(synthetic_config_value()));
    let t = floats(&v["times_s"]);
    let f = floats(&v["release_fraction"]);
    let m = floats(&v["inventory_mol"]);
    let r = floats(&v["cumulative_release_mol"]);
    let q = floats(&v["release_rate_mol_s"]);
    let res = floats(&v["balance_residual_mol"]);
    let mn = floats(&v["min_concentration_mol_m3"]);
    assert_eq!(t.len(), rows.len());
    for (i, (ti, d)) in rows.iter().enumerate() {
        assert_eq!(t[i].to_bits(), ti.to_bits());
        assert_eq!(f[i].to_bits(), d.release_fraction.to_bits());
        assert_eq!(m[i].to_bits(), d.inventory.to_bits());
        assert_eq!(r[i].to_bits(), d.cumulative_release.to_bits());
        assert_eq!(q[i].to_bits(), d.release_rate.to_bits());
        assert_eq!(res[i].to_bits(), d.balance_residual.to_bits());
        assert_eq!(mn[i].to_bits(), d.min_concentration.to_bits());
    }
    let profiles = v["profiles"].as_array().unwrap();
    let c_end = floats(&profiles.last().unwrap()["c_mol_per_m3"]);
    assert!(c_end
        .iter()
        .zip(&final_c)
        .all(|(a, b)| a.to_bits() == b.to_bits()));
    // Every profile's inventory is finite_volume::inventory of that profile, equal to the row's.
    for (p, (_, d)) in profiles.iter().zip(&rows) {
        assert_eq!(
            p["inventory_mol"].as_f64().unwrap().to_bits(),
            d.inventory.to_bits()
        );
    }
}

#[test]
fn time_series_and_final_profile_match_v1_golden() {
    let v = run_ok(&request(synthetic_config_value()));
    let golden = read_rows("timeseries.csv");
    let t = floats(&v["times_s"]);
    let m = floats(&v["inventory_mol"]);
    let r = floats(&v["cumulative_release_mol"]);
    let f = floats(&v["release_fraction"]);
    assert_eq!(t.len(), golden.len());
    let m0 = golden[0][1];
    for (i, g) in golden.iter().enumerate() {
        assert!(close(t[i], g[0], 1.0));
        assert!(close(m[i], g[1], m0), "inventory at {}", g[0]);
        assert!(close(r[i], g[2], m0), "release at {}", g[0]);
        assert!(close(f[i], g[3], 1.0), "F at {}", g[0]);
    }
    let profile = read_rows("final_profile.csv");
    let c = floats(&v["profiles"].as_array().unwrap().last().unwrap()["c_mol_per_m3"]);
    let x = floats(&v["mesh"]["cell_centroids_m"]);
    let region = v["mesh"]["region_of_cell"].as_array().unwrap();
    assert_eq!(c.len(), profile.len());
    let cmax = profile.iter().map(|g| g[3]).fold(0.0, f64::max);
    for (i, g) in profile.iter().enumerate() {
        assert_eq!(
            region[i].as_u64().unwrap() as f64,
            g[1],
            "region of cell {i}"
        );
        assert!(close(x[i], g[2], x[i]), "radius of cell {i}");
        assert!(close(c[i], g[3], cmax), "concentration of cell {i}");
    }
}

#[test]
fn code_c_and_its_difference_from_code_b_match_v1_golden() {
    let v = run_ok(&request(synthetic_config_value()));
    let golden = read_rows("comparison_fv_vs_analytical.csv");
    let a = &v["analytical"];
    let t = floats(&a["times_s"]);
    let f = floats(&a["release_fraction"]);
    assert!(
        a.get("abs_difference_fv").is_none(),
        "the pointwise difference is not returned (parity, see PROGRESS.md)"
    );
    // Code B at the same times, from the run's own series (the series times are a subset).
    let tb = floats(&v["times_s"]);
    let fb = floats(&v["release_fraction"]);
    let kept: Vec<&Vec<f64>> = golden.iter().filter(|g| g[2].is_finite()).collect();
    assert_eq!(t.len(), kept.len());
    for (i, g) in kept.iter().enumerate() {
        assert!(close(t[i], g[0], 1.0));
        assert!(
            close(f[i], g[2], 1.0),
            "series at t = {}: {} vs {}",
            g[0],
            f[i],
            g[2]
        );
        let j = tb
            .iter()
            .position(|x| x.to_bits() == t[i].to_bits())
            .unwrap();
        let d = (fb[j] - f[i]).abs();
        assert!(
            close(d, g[3], 1.0),
            "difference at t = {}: {} vs {}",
            g[0],
            d,
            g[3]
        );
    }
    // Diagnostics: the largest and the final difference, the floor and the validity time, as
    // the CLI prints them (tests/golden/v1.0/main_run_stdout.txt).
    let dc = &v["diagnostics"]["code_c"];
    let max_golden = kept.iter().map(|g| g[3]).fold(0.0, f64::max);
    assert!(close(
        dc["max_abs_difference"].as_f64().unwrap(),
        max_golden,
        1.0
    ));
    assert!(close(
        dc["final_abs_difference"].as_f64().unwrap(),
        kept.last().unwrap()[3],
        1.0
    ));
    assert_eq!(
        format!("{:.3e}", dc["accuracy_floor"].as_f64().unwrap()),
        "3.997e-12"
    );
    assert_eq!(
        format!("{:.6e}", dc["valid_from_s"].as_f64().unwrap()),
        "1.777778e3"
    );
    assert_eq!(dc["modes"].as_u64().unwrap(), 32);
    assert_eq!(dc["resolved"], true);
    assert_eq!(
        format!("{:.6e}", dc["max_abs_difference"].as_f64().unwrap()),
        "2.537522e-4"
    );
}

#[test]
fn balance_residual_is_reported_and_no_worse_than_v1() {
    let v = run_ok(&request(synthetic_config_value()));
    let golden = read_rows("timeseries.csv");
    let m0 = golden[0][1];
    let golden_max = golden.iter().map(|g| g[5].abs()).fold(0.0, f64::max);
    let b = &v["balance"];
    let max_abs = b["max_abs_residual_mol"].as_f64().unwrap();
    let rel = b["max_abs_residual_relative"].as_f64().unwrap();
    assert!(
        close(max_abs, golden_max, m0),
        "{max_abs} vs golden {golden_max}"
    );
    assert!(rel <= V1_BALANCE_REL_LEVEL, "relative residual {rel:e}");
    assert_eq!(
        b["start_inventory_mol"].as_f64().unwrap().to_bits(),
        m0.to_bits()
    );
    // The final terms are the library's own values (main_run_stdout.txt prints them at 12 digits).
    let fin = &b["final"];
    assert_eq!(
        format!("{:.12e}", fin["inventory_mol"].as_f64().unwrap()),
        "1.901949993620e-11"
    );
    assert_eq!(
        format!("{:.12e}", fin["residual_mol"].as_f64().unwrap().abs()),
        "9.402717119255e-24"
    );
    let dg = &v["diagnostics"];
    assert_eq!(dg["cells"], 200);
    assert_eq!(dg["steps"], 1000);
    assert_eq!(dg["dt_used_s"].as_f64().unwrap(), 200.0);
    assert_eq!(dg["output_rows"], 101);
    assert_eq!(
        format!("{:.12e}", dg["explicit_dt_bound_s"].as_f64().unwrap()),
        "7.812107958751e-3"
    );
    assert_eq!(
        format!("{:.12e}", dg["final_release_fraction"].as_f64().unwrap()),
        "4.324286068785e-1"
    );
    assert_eq!(
        format!("{:.12e}", dg["min_concentration_mol_m3"].as_f64().unwrap()),
        "0.000000000000e0"
    );
}

#[test]
fn layer_inventories_sum_to_the_total_inventory() {
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
    // At t = 0 everything is in the kernel.
    let first = floats(&profiles[0]["layer_share"]);
    assert_eq!(first, vec![1.0, 0.0, 0.0, 0.0, 0.0]);
}

fn with(mut v: Value, pointer: &str, new: Value) -> Value {
    *v.pointer_mut(pointer)
        .unwrap_or_else(|| panic!("no {pointer}")) = new;
    v
}

fn assert_rejected(req: &str, needle: &str) {
    let out = catch_unwind(AssertUnwindSafe(|| strato_wasm::run_v1(req)));
    let text = out.unwrap_or_else(|_| panic!("run_v1 panicked on {req}"));
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["status"], "rejected", "{v}");
    let errors = v["errors"].as_array().unwrap();
    assert!(!errors.is_empty());
    assert!(
        errors.iter().any(|e| e.as_str().unwrap().contains(needle)),
        "expected '{needle}' in {errors:?}"
    );
    // validate_config agrees for the same request.
    let val: Value = serde_json::from_str(&strato_wasm::validate_config(req)).unwrap();
    assert_eq!(val["ok"], false, "validate_config accepted {req}");
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
            with(base.clone(), "/temperature_k", json!(-5.0)),
            "temperature_k",
        ),
        (
            with(base.clone(), "/regions/2/outer_radius_m", json!(3.0e-4)),
            "invalid region IPyC",
        ),
        (
            with(base.clone(), "/regions/0/outer_radius_m", json!(0.0)),
            "invalid region Kernel",
        ),
        (
            with(base.clone(), "/regions/1/diffusivity_m2_s", json!(0.0)),
            "invalid region Buffer",
        ),
        (
            with(base.clone(), "/regions/3/diffusivity_m2_s", json!(-1e-12)),
            "invalid region SiC",
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
            with(base.clone(), "/time/end_time_s", json!(-1.0)),
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
            "initial inventory must be positive",
        ),
        (
            with(base.clone(), "/temperature_k", json!("hot")),
            "config:",
        ),
        (
            with(base.clone(), "/regions/0/cells", json!(2001)),
            "MAX_TOTAL_CELLS",
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
    // Missing fields: every field of Config is required, temperature_k included.
    let mut missing = synthetic_config_value();
    missing.as_object_mut().unwrap().remove("temperature_k");
    assert_rejected(&request(missing), "missing field `temperature_k`");
    // Malformed requests.
    assert_rejected("not json", "request:");
    assert_rejected("{}", "request:");
    assert_rejected(
        &json!({"schema": 2, "config": base.clone()}).to_string(),
        "unsupported request schema 2",
    );
    assert_rejected(
        &json!({"schema": 1, "config": base.clone(), "extra": 1}).to_string(),
        "unknown field",
    );
    assert_rejected(
        "{\"schema\":1,\"config\":{\"temperature_k\":1e400}}",
        "request:",
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
    // The TOML template form, through validate_config.
    let toml = synthetic_toml().replace(
        "diffusivity_m2_s = 1.0e-12",
        "# INSERT LITERATURE-VERIFIED KERNEL DIFFUSIVITY (m^2/s) BELOW\ndiffusivity_m2_s = \"INSERT_VALUE_HERE\"",
    );
    let v: Value = serde_json::from_str(&strato_wasm::validate_config(&toml)).unwrap();
    assert_eq!(v["ok"], false);
    let e = v["errors"][0].as_str().unwrap();
    assert!(
        e.contains("KERNEL DIFFUSIVITY") && e.contains("line "),
        "{e}"
    );
    // A commented placeholder is not a placeholder (rule of src/config.rs).
    let commented = format!("{}\n# d = \"INSERT_VALUE_HERE\"\n", synthetic_toml());
    let v: Value = serde_json::from_str(&strato_wasm::validate_config(&commented)).unwrap();
    assert_eq!(v["ok"], true, "{v}");
}

#[test]
fn unsupported_mechanisms_are_rejected() {
    for m in [
        "decay",
        "generation",
        "diffusivity_temperature",
        "diffusivity_layer_laws",
        "inventory_balance_v2",
        "",
    ] {
        let req = json!({"schema": 1, "config": synthetic_config_value(), "mechanisms": ["diffusion_layered", m]}).to_string();
        assert_rejected(&req, "is not supported by this solver build");
    }
    let ok = json!({"schema": 1, "config": synthetic_config_value(), "mechanisms": ["diffusion_layered"]}).to_string();
    assert_eq!(run_ok(&ok)["mechanisms"], json!(["diffusion_layered"]));
}

#[test]
fn awkward_inputs_never_panic() {
    // Extreme but valid configurations: each must return ok or rejected, never panic.
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
    ];
    for cfg in cases {
        let req = request(cfg);
        let out = catch_unwind(AssertUnwindSafe(|| strato_wasm::run_v1(&req)));
        let text = out.unwrap_or_else(|_| panic!("panicked on {req}"));
        let v: Value = serde_json::from_str(&text).unwrap();
        assert!(v["status"] == "ok" || v["status"] == "rejected", "{v}");
    }
}

#[test]
fn series_mode_bound_is_an_upper_bound_of_the_library_count() {
    use strato::{
        analytical::sturm_liouville::eigenvalue_count,
        postprocessing::comparison::layers_from_config,
    };
    let base = synthetic_config_value();
    let mut checked = 0;
    for d_sic in [2e-14, 1e-16, 1e-12, 4e-10] {
        for t_min in [2000.0, 100.0, 10.0, 2.5, 0.5] {
            let v = with(base.clone(), "/regions/3/diffusivity_m2_s", json!(d_sic));
            let mut v = v;
            v.as_object_mut()
                .unwrap()
                .insert("output".into(), json!({"directory": "unused"}));
            let cfg: Config = serde_json::from_value(v).unwrap();
            let n = eigenvalue_count(45.0 / t_min, &layers_from_config(&cfg));
            let bound = strato_wasm::series_mode_bound(&cfg, t_min);
            assert!(
                n as f64 <= bound,
                "count {n} above bound {bound} (D_SiC = {d_sic}, t_min = {t_min})"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 20);
    // The synthetic case keeps 32 modes; the bound allows the series.
    let cfg = Config::load(path("inputs/synthetic/synthetic_five_layer.toml")).unwrap();
    assert!(strato_wasm::series_mode_bound(&cfg, 2000.0) <= strato_wasm::MAX_SERIES_MODES as f64);
    // A tiny first output time is refused before the library is called, and the
    // finite-volume result is still returned.
    let tiny = with(synthetic_config_value(), "/time/end_time_s", json!(1e-300));
    let v = run_ok(&request(tiny));
    assert!(v["analytical"].is_null());
    assert!(v["messages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m.as_str().unwrap().contains("MAX_SERIES_MODES")));
}

#[test]
fn analytical_release_equals_the_run_series() {
    let cfg = synthetic_config_value();
    let run = run_ok(&request(cfg.clone()));
    let a: Value = serde_json::from_str(&strato_wasm::analytical_release(&request(cfg))).unwrap();
    assert_eq!(a["status"], "ok");
    let ta = floats(&a["times_s"]);
    let fa = floats(&a["release_fraction"]);
    let tr = floats(&run["analytical"]["times_s"]);
    let fr = floats(&run["analytical"]["release_fraction"]);
    assert_eq!(ta.len(), tr.len());
    assert!(ta.iter().zip(&tr).all(|(x, y)| x.to_bits() == y.to_bits()));
    assert!(fa.iter().zip(&fr).all(|(x, y)| x.to_bits() == y.to_bits()));
    assert_eq!(a["accuracy_floor"], run["analytical"]["accuracy_floor"]);
    let bad: Value = serde_json::from_str(&strato_wasm::analytical_release(
        "{\"schema\":1,\"config\":{}}",
    ))
    .unwrap();
    assert_eq!(bad["status"], "rejected");
}

#[test]
fn compare_returns_modified_minus_baseline() {
    let base = synthetic_config_value();
    let modified = with(base.clone(), "/regions/3/diffusivity_m2_s", json!(4.0e-14));
    let req = json!({"schema": 1, "baseline": {"schema": 1, "config": base}, "modified": {"schema": 1, "config": modified}}).to_string();
    let v: Value = serde_json::from_str(&strato_wasm::compare_v1(&req)).unwrap();
    assert_eq!(v["status"], "ok", "{v}");
    let fb = floats(&v["baseline"]["release_fraction"]);
    let fm = floats(&v["modified"]["release_fraction"]);
    let d = floats(&v["difference"]["release_fraction"]);
    assert_eq!(d.len(), fb.len());
    for i in 0..d.len() {
        assert_eq!(d[i].to_bits(), (fm[i] - fb[i]).to_bits());
    }
    assert!(d.last().unwrap() > &0.0, "a faster SiC releases more");
    // Different time grids are rejected, never interpolated.
    let other = with(
        synthetic_config_value(),
        "/time/output_every_steps",
        json!(20),
    );
    let req = json!({"schema": 1, "baseline": {"schema": 1, "config": synthetic_config_value()}, "modified": {"schema": 1, "config": other}}).to_string();
    let v: Value = serde_json::from_str(&strato_wasm::compare_v1(&req)).unwrap();
    assert_eq!(v["status"], "rejected");
    // A rejected half names itself.
    let bad = with(synthetic_config_value(), "/time/dt_s", json!(0.0));
    let req = json!({"schema": 1, "baseline": {"schema": 1, "config": synthetic_config_value()}, "modified": {"schema": 1, "config": bad}}).to_string();
    let v: Value = serde_json::from_str(&strato_wasm::compare_v1(&req)).unwrap();
    assert_eq!(v["status"], "rejected");
    assert!(v["errors"][0].as_str().unwrap().starts_with("modified: "));
}

#[test]
fn build_info_reports_the_v1_capabilities() {
    let v: Value = serde_json::from_str(&strato_wasm::build_info()).unwrap();
    assert_eq!(v["abi"], 1);
    assert_eq!(strato_wasm::abi_version(), 1);
    assert_eq!(v["capabilities"], json!(["diffusion_layered"]));
    assert_eq!(v["label"], "COMPUTED v1.0");
    let cargo = std::fs::read_to_string(path("Cargo.toml")).unwrap();
    assert!(cargo.contains(&format!(
        "version = \"{}\"",
        v["strato_version"].as_str().unwrap()
    )));
    // The pinned wasm-bindgen equals the version in this crate's Cargo.lock.
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock")).unwrap();
    assert!(lock.contains(&format!(
        "name = \"wasm-bindgen\"\nversion = \"{}\"",
        v["wasm_bindgen"].as_str().unwrap()
    )));
}

#[test]
fn validate_config_accepts_the_synthetic_case_in_both_forms() {
    let toml: Value =
        serde_json::from_str(&strato_wasm::validate_config(&synthetic_toml())).unwrap();
    assert_eq!(toml["ok"], true, "{toml}");
    assert_eq!(toml["plan"]["steps"], 1000);
    let json_req: Value = serde_json::from_str(&strato_wasm::validate_config(&request(
        synthetic_config_value(),
    )))
    .unwrap();
    assert_eq!(json_req["ok"], true);
    let bare: Value = serde_json::from_str(&strato_wasm::validate_config(
        &synthetic_config_value().to_string(),
    ))
    .unwrap();
    assert_eq!(bare["ok"], true);
    // A step that does not divide the end time is reported as a warning.
    let odd = with(synthetic_config_value(), "/time/dt_s", json!(300.0));
    let w: Value = serde_json::from_str(&strato_wasm::validate_config(&request(odd))).unwrap();
    assert_eq!(w["ok"], true);
    assert!(w["warnings"][0].as_str().unwrap().contains("will use dt"));
}
