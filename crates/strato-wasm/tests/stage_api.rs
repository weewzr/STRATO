//! Native checks of the stage and scenario functions of the wrapper (ABI 4, schema 3):
//! `run_stage`, `btcs_system`, `analytical`, `run_scenario` and `options`.
//!
//! - a stage run is bit-identical to the library's `run_physics` for the same input written as
//!   the library's own TOML table (an independent route to the same numbers), and stage 1 is
//!   bit-identical to the schema 2 `run`;
//! - the assembled system of `btcs_system` is the solver's matrix, entry for entry, and its
//!   solution is the solver's next profile;
//! - the analytical function returns the one Method B function's values, withheld values are
//!   `null` with a reason and never zero, and a temperature that varies is refused with its
//!   reason;
//! - a scenario run equals the committed curve file of the results pipeline for the same grid;
//! - the options come from the embedded definitions;
//! - invalid input is rejected with a message, and nothing panics.

use serde_json::{json, Value};
use std::panic::{catch_unwind, AssertUnwindSafe};
use strato::config::Config;
use strato::numerical::{
    btcs::{validate_input, Limits, Stepper},
    grid::Grid,
    inventory::initial_vector,
    physics_config::load_physics,
    physics_run::{run_physics, source_nodes},
};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");

fn path(rel: &str) -> String {
    format!("{ROOT}{rel}")
}

fn synthetic_toml() -> String {
    std::fs::read_to_string(path("inputs/synthetic/synthetic_five_layer.toml")).unwrap()
}

fn config_value() -> Value {
    toml::from_str(&synthetic_toml()).unwrap()
}

const LIMITS: Limits = Limits {
    max_intervals_total: 100_000,
    max_steps: 10_000_000,
};

/// The five Arrhenius laws of the tests: test inputs for the numerics, not register values.
fn laws_json() -> Value {
    let d0 = [1.0e-6, 2.0e-7, 5.0e-6, 3.0e-5, 8.0e-6];
    let ea = [200.0, 150.0, 220.0, 260.0, 210.0];
    let names = ["Kernel", "Buffer", "IPyC", "SiC", "OPyC"];
    Value::Array(
        (0..5)
            .map(|k| {
                json!({
                    "region": names[k],
                    "source": "test input, no source",
                    "class": "user-supplied, no source",
                    "terms": [{"d0_m2_s": d0[k], "activation_energy_kj_per_mol": ea[k],
                               "valid_from_k": 1000.0, "valid_to_k": 1500.0}],
                })
            })
            .collect(),
    )
}

fn physics_json(stage: u8, temperature: Value) -> Value {
    let mut p = serde_json::Map::new();
    if stage >= 2 {
        p.insert(
            "source".into(),
            json!({
                "mode": "kernel_fission_yield",
                "product": "direct_fission_product",
                "yield_kind": "cumulative",
                "yield_atoms_per_fission": 0.06,
                "fission_rate_origin": "test input",
                "fission_rate_density": {"value": 1.0e19},
            }),
        );
    }
    if stage >= 3 {
        p.insert("decay".into(), json!({"half_life_s": 1.0e5}));
    }
    if stage >= 4 {
        p.insert("temperature".into(), temperature);
        p.insert("arrhenius".into(), laws_json());
    }
    Value::Object(p)
}

fn ramp() -> Value {
    json!({"unit": "degC", "interpolation": "linear", "times_s": [0.0, 2.0e5], "values": [900.0, 1100.0]})
}

fn flat(c: f64) -> Value {
    json!({"unit": "degC", "value": c})
}

fn stage_request(stage: u8, physics: Option<Value>) -> Value {
    let mut r = json!({"schema": 3, "config": config_value(), "stage": stage});
    if let Some(p) = physics {
        r["physics"] = p;
    }
    r
}

/// The same physics as the library's own TOML table, written by hand.
fn library_toml(stage: u8, temperature_toml: &str) -> String {
    let mut s = String::new();
    if stage >= 2 {
        s.push_str(
            "[physics.source]\nmode = \"kernel_fission_yield\"\nproduct = \"direct_fission_product\"\nyield_kind = \"cumulative\"\nyield_atoms_per_fission = 0.06\nfission_rate_origin = \"test input\"\n[physics.source.fission_rate_density]\nvalue = 1.0e19\n",
        );
    }
    if stage >= 3 {
        s.push_str("[physics.decay]\nhalf_life_s = 1.0e5\n");
    }
    if stage >= 4 {
        s.push_str(temperature_toml);
        let d0 = ["1.0e-6", "2.0e-7", "5.0e-6", "3.0e-5", "8.0e-6"];
        let ea = ["200.0", "150.0", "220.0", "260.0", "210.0"];
        let names = ["Kernel", "Buffer", "IPyC", "SiC", "OPyC"];
        for k in 0..5 {
            s.push_str(&format!(
                "[[physics.arrhenius]]\nregion = \"{}\"\nsource = \"test input, no source\"\nclass = \"user-supplied, no source\"\n[[physics.arrhenius.terms]]\nd0_m2_s = {}\nactivation_energy_kj_per_mol = {}\nvalid_from_k = 1000.0\nvalid_to_k = 1500.0\n",
                names[k], d0[k], ea[k]
            ));
        }
    }
    s
}

const RAMP_TOML: &str = "[physics.temperature]\nunit = \"degC\"\ninterpolation = \"linear\"\ntimes_s = [0.0, 2.0e5]\nvalues = [900.0, 1100.0]\n";
const FLAT_TOML: &str = "[physics.temperature]\nunit = \"degC\"\nvalue = 1000.0\n";

fn call(f: fn(&str) -> String, req: &Value) -> Value {
    serde_json::from_str(&f(&req.to_string())).unwrap()
}

fn ok(f: fn(&str) -> String, req: &Value) -> Value {
    let v = call(f, req);
    assert_eq!(v["status"], "ok", "{v}");
    v
}

fn rejected(f: fn(&str) -> String, req: &Value) -> Vec<String> {
    let v = call(f, req);
    assert_eq!(v["status"], "rejected", "{v}");
    v["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e.as_str().unwrap().to_string())
        .collect()
}

fn floats(v: &Value) -> Vec<f64> {
    v.as_array()
        .unwrap_or_else(|| panic!("not an array: {v}"))
        .iter()
        .map(|x| x.as_f64().unwrap_or_else(|| panic!("not a number: {x}")))
        .collect()
}

fn nullable(v: &Value) -> Vec<Option<f64>> {
    v.as_array().unwrap().iter().map(Value::as_f64).collect()
}

/// The library's run for a stage, through the library's TOML table.
fn library_run(stage: u8, temperature_toml: &str) -> strato::numerical::physics_run::PhysicsRun {
    let cfg: Config = toml::from_str(&synthetic_toml()).unwrap();
    let input = validate_input(&cfg, &LIMITS).unwrap();
    let end = input.steps as f64 * input.dt;
    let text = library_toml(stage, temperature_toml);
    let pin = if text.is_empty() {
        strato::numerical::physics_run::PhysicsInput::baseline(input, end)
    } else {
        load_physics(&text, input, end).unwrap().unwrap().input
    };
    run_physics(&pin, &[]).unwrap()
}

#[test]
fn abi_four_and_the_new_functions_are_listed() {
    assert_eq!(strato_wasm::abi_version(), 4);
    let b: Value = serde_json::from_str(&strato_wasm::build_info()).unwrap();
    assert_eq!(b["abi"], 4);
    let funcs: Vec<&str> = b["stages"]["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f.as_str().unwrap())
        .collect();
    assert_eq!(
        funcs,
        [
            "run_stage",
            "run_scenario",
            "btcs_system",
            "analytical",
            "options"
        ]
    );
    // The functions the site calls today are listed as before.
    assert_eq!(
        b["btcs"]["functions"],
        json!(["validate", "run", "compare"])
    );
    assert_eq!(b["btcs"]["capabilities"], json!(["diffusion_btcs_layered"]));
    let caps: Vec<&str> = b["stages"]["capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f.as_str().unwrap())
        .collect();
    for c in ["generation", "decay", "diffusivity_temperature"] {
        assert!(caps.contains(&c), "{c}");
    }
}

#[test]
fn stage_1_is_the_schema_2_run_bit_for_bit() {
    let v = ok(strato_wasm::run_stage, &stage_request(1, None));
    let old: Value = serde_json::from_str(&strato_wasm::run(
        &json!({"schema": 2, "config": config_value()}).to_string(),
    ))
    .unwrap();
    let s = &v["series"];
    for (new, old_key) in [
        ("times_s", "times_s"),
        ("release_fraction", "release_fraction"),
        ("inventory_mol", "inventory_mol"),
        ("cumulative_release_mol", "cumulative_release_mol"),
        ("balance_residual_mol", "balance_residual_mol"),
        ("balance_residual_bound_mol", "balance_residual_bound_mol"),
        ("min_concentration_mol_m3", "min_concentration_mol_m3"),
        ("max_concentration_mol_m3", "max_concentration_mol_m3"),
    ] {
        assert_eq!(floats(&s[new]), floats(&old[old_key]), "{new}");
    }
    // No source and no decay: the generated and decayed amounts are zero at every row.
    assert!(floats(&s["generated_mol"]).iter().all(|x| *x == 0.0));
    assert!(floats(&s["decayed_mol"]).iter().all(|x| *x == 0.0));
    assert_eq!(v["grid"]["node_radii_m"], old["grid"]["node_radii_m"]);
    assert_eq!(v["stage"]["mechanisms"], json!([]));
    assert_eq!(v["inputs"]["temperature_used_by_solver"], false);
    assert_eq!(v["label"], "COMPUTED v2.0");
    // The release rate is null at the first row and a number after it.
    assert!(v["series"]["release_rate_mol_s"][0].is_null());
    assert!(v["series"]["release_rate_mol_s"][1].is_number());
    assert!(v["series"]["temperature_k"][0].is_null());
}

#[test]
fn every_stage_is_bit_identical_to_the_library_run() {
    for (stage, temp, temp_toml) in [
        (2u8, ramp(), ""),
        (3, ramp(), ""),
        (4, ramp(), RAMP_TOML),
        (5, flat(1000.0), FLAT_TOML),
    ] {
        let v = ok(
            strato_wasm::run_stage,
            &stage_request(stage, Some(physics_json(stage, temp))),
        );
        let lib = library_run(stage, temp_toml);
        let s = &v["series"];
        let col = |f: &dyn Fn(usize) -> f64| -> Vec<f64> { (0..lib.series.len()).map(f).collect() };
        assert_eq!(
            floats(&s["times_s"]),
            col(&|k| lib.series[k].0),
            "t {stage}"
        );
        assert_eq!(
            floats(&s["release_fraction"]),
            col(&|k| lib.series[k].1.release_fraction),
            "F {stage}"
        );
        assert_eq!(
            floats(&s["inventory_mol"]),
            col(&|k| lib.series[k].1.inventory),
            "M {stage}"
        );
        assert_eq!(
            floats(&s["generated_mol"]),
            col(&|k| lib.series[k].2.generated),
            "gen {stage}"
        );
        assert_eq!(
            floats(&s["decayed_mol"]),
            col(&|k| lib.series[k].2.decayed),
            "dec {stage}"
        );
        assert_eq!(
            floats(&s["release_fraction_of_total"]),
            col(&|k| lib.series[k].2.release_fraction_of_total),
            "F_total {stage}"
        );
        assert_eq!(
            floats(&s["balance_residual_bound_mol"]),
            col(&|k| lib.series[k].1.balance_residual_bound),
            "bound {stage}"
        );
        // The balance holds within the derived rounding bound.
        assert_eq!(v["balance"]["tolerance"]["within_bound"], true, "{stage}");
        // The stage block names the mechanisms that are on.
        let n_mech = v["stage"]["mechanisms"].as_array().unwrap().len();
        assert_eq!(n_mech, [0, 1, 2, 3, 3][stage as usize - 1], "{stage}");
        if stage >= 2 {
            assert!(floats(&s["generated_mol"]).last().unwrap() > &0.0);
        }
        if stage >= 3 {
            assert!(floats(&s["decayed_mol"]).last().unwrap() > &0.0);
        }
    }
}

#[test]
fn stage_4_reports_the_temperature_and_the_diffusivity_of_each_step() {
    let v = ok(
        strato_wasm::run_stage,
        &stage_request(4, Some(physics_json(4, ramp()))),
    );
    let s = &v["series"];
    let k = nullable(&s["temperature_k"]);
    let c = nullable(&s["temperature_c"]);
    assert_eq!(k.len(), c.len());
    for (a, b) in k.iter().zip(&c) {
        assert_eq!(a.map(|x| x - 273.15), *b);
    }
    // The ramp of the request: 900 degC at the start, 1100 degC at the end.
    assert!((c[0].unwrap() - 900.0).abs() < 1e-9);
    assert!((c.last().unwrap().unwrap() - 1100.0).abs() < 1e-9);
    // One diffusivity series per layer, null at the first row, and it rises with temperature.
    let d = s["diffusivity_used_m2_s"].as_array().unwrap();
    assert_eq!(d.len(), 5);
    for layer in d {
        let col = nullable(layer);
        assert!(col[0].is_none());
        assert!(col.last().unwrap().unwrap() > col[1].unwrap());
    }
    // 900 to 1100 degC (1173 to 1373 K) lies inside the printed range 1000 to 1500 K of every law.
    for r in v["diagnostics"]["ranges"].as_array().unwrap() {
        assert_eq!(r["steps_inside_printed_range"], v["diagnostics"]["steps"]);
        assert_eq!(r["steps_below_printed_range"], 0);
        assert_eq!(r["steps_above_printed_range"], 0);
    }
    assert!(v["messages"].as_array().unwrap().is_empty());
    // A ramp from 600 to 1300 degC leaves the range on both sides; the counts say so and the
    // reply carries the message (the value is an extrapolation, never clamped).
    let wide = json!({"unit": "degC", "interpolation": "linear", "times_s": [0.0, 2.0e5], "values": [600.0, 1300.0]});
    let w = ok(
        strato_wasm::run_stage,
        &stage_request(4, Some(physics_json(4, wide))),
    );
    let r = &w["diagnostics"]["ranges"][0];
    assert!(r["steps_below_printed_range"].as_u64().unwrap() > 0);
    assert!(r["steps_above_printed_range"].as_u64().unwrap() > 0);
    assert!(w["messages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m.as_str().unwrap().contains("printed validity range")));
    // The analytical method is not applicable while the temperature changes.
    assert_eq!(v["analytical"]["applicable"], false);
    assert_eq!(v["analytical"]["reason_code"], "TIME_VARYING_TEMPERATURE");
}

#[test]
fn sections_of_a_mechanism_that_is_off_are_refused() {
    let e = rejected(
        strato_wasm::run_stage,
        &stage_request(1, Some(physics_json(2, ramp()))),
    );
    assert!(
        e[0].contains("physics.source") && e[0].contains("no generation"),
        "{e:?}"
    );
    let e = rejected(
        strato_wasm::run_stage,
        &stage_request(2, Some(physics_json(3, ramp()))),
    );
    assert!(
        e.iter()
            .any(|m| m.contains("physics.decay") && m.contains("no decay")),
        "{e:?}"
    );
    let e = rejected(
        strato_wasm::run_stage,
        &stage_request(3, Some(physics_json(4, ramp()))),
    );
    assert!(e.iter().any(|m| m.contains("physics.temperature")), "{e:?}");
    // Stage 4 needs the laws.
    let e = rejected(strato_wasm::run_stage, &stage_request(4, None));
    assert!(e[0].contains("temperature-dependent diffusivity"), "{e:?}");
    let mut p = physics_json(4, ramp());
    p.as_object_mut().unwrap().remove("arrhenius");
    let e = rejected(strato_wasm::run_stage, &stage_request(4, Some(p)));
    assert!(e[0].contains("physics.arrhenius"), "{e:?}");
    // A law without its source row is refused by the library (decision 31), with its message.
    let mut p = physics_json(4, ramp());
    p["arrhenius"][0].as_object_mut().unwrap().remove("source");
    let e = rejected(strato_wasm::run_stage, &stage_request(4, Some(p)));
    assert!(e[0].contains("source") && e[0].contains("class"), "{e:?}");
    // Stage numbers, schema and unknown fields.
    for stage in [0u8, 6] {
        let e = rejected(strato_wasm::run_stage, &stage_request(stage, None));
        assert!(e[0].contains("from 1 to 5"), "{e:?}");
    }
    let mut r = stage_request(1, None);
    r["schema"] = json!(2);
    assert!(rejected(strato_wasm::run_stage, &r)[0].contains("schema"));
    let mut r = stage_request(1, None);
    r["surprise"] = json!(1);
    assert!(rejected(strato_wasm::run_stage, &r)[0].contains("surprise"));
    // A source history that stops before the end of the run is a library message.
    let mut p = physics_json(2, ramp());
    p["source"]["fission_rate_density"] =
        json!({"interpolation": "step", "times_s": [0.0, 1.0e4], "values": [1.0e19, 0.0]});
    ok(strato_wasm::run_stage, &stage_request(2, Some(p)));
}

#[test]
fn the_system_is_the_matrix_the_solver_assembled() {
    // A small grid: 5 intervals per layer, a step of 2000 s, three steps.
    let mut cfg = config_value();
    for r in cfg["regions"].as_array_mut().unwrap() {
        r["cells"] = json!(5);
    }
    cfg["time"] = json!({"dt_s": 2000.0, "end_time_s": 6000.0, "output_every_steps": 1});
    let mut req = json!({"schema": 3, "config": cfg, "stage": 5,
        "physics": physics_json(5, flat(1000.0)), "step": 2});
    req["rows"] = json!((0..25).collect::<Vec<u32>>());
    let v = ok(strato_wasm::btcs_system, &req);
    assert_eq!(v["rows"].as_array().unwrap().len(), 25);
    // The library's own assembly at the same step.
    let cfg_lib: Config = serde_json::from_value({
        let mut c = req["config"].clone();
        c["output"] = json!({"directory": "x"});
        c
    })
    .unwrap();
    let input = validate_input(&cfg_lib, &LIMITS).unwrap();
    let grid = Grid::build(&input).unwrap();
    let end = input.steps as f64 * input.dt;
    let text = library_toml(5, FLAT_TOML);
    let pin = load_physics(&text, input.clone(), end)
        .unwrap()
        .unwrap()
        .input;
    let d: Vec<f64> = floats(&v["diffusivity_used_m2_s"]);
    let kappa = v["decay_number_kappa"].as_f64().unwrap();
    assert_eq!(kappa, pin.decay_constant_per_s * input.dt);
    let d5: [f64; 5] = d.try_into().unwrap();
    let mut st = Stepper::new_with_decay(&grid, d5, input.dt, kappa).unwrap();
    let base = Stepper::new(&grid, d5, input.dt).unwrap();
    let m = st.assembly().matrix.clone();
    let m0 = &base.assembly().matrix;
    // The state before step 2 is the state after step 1 of the library run.
    let run = run_physics(&pin, &[input.dt]).unwrap();
    let c_old = &run.snapshots[0].1;
    let gbar: [f64; 5] =
        std::array::from_fn(|k| pin.source.per_material[k].mean_over(input.dt, 2.0 * input.dt));
    let nodes = source_nodes(&grid, gbar);
    let mut c_new = vec![0.0; grid.n + 1];
    st.step_with_source(c_old, &nodes.g, &mut c_new).unwrap();
    for row in v["rows"].as_array().unwrap() {
        let i = row["row"].as_u64().unwrap() as usize;
        assert_eq!(row["diag"].as_f64().unwrap(), m.diag[i], "diag {i}");
        if i > 0 {
            assert_eq!(row["lower"].as_f64().unwrap(), m.lower[i], "lower {i}");
        } else {
            assert!(row["lower"].is_null());
        }
        if i < grid.n - 1 {
            assert_eq!(row["upper"].as_f64().unwrap(), m.upper[i], "upper {i}");
        } else {
            assert!(row["upper"].is_null());
            assert_eq!(row["upper_dropped_into_diagonal"], true);
        }
        // The diagonal is the diffusion part plus the decay number (F37), exactly.
        let t = &row["terms"];
        assert_eq!(t["diag_diffusion"].as_f64().unwrap(), m0.diag[i]);
        assert_eq!(t["diag_diffusion"].as_f64().unwrap() + kappa, m.diag[i]);
        assert_eq!(t["decay_on_diagonal"].as_f64().unwrap(), kappa);
        assert_eq!(
            t["source_in_rhs_mol_m3"].as_f64().unwrap(),
            input.dt * nodes.g[i]
        );
        assert_eq!(
            row["rhs_mol_m3"].as_f64().unwrap(),
            c_old[i] + input.dt * nodes.g[i]
        );
        assert_eq!(row["solution_c_new_mol_m3"].as_f64().unwrap(), c_new[i]);
        assert!(!row["symbolic"]["rhs"].as_str().unwrap().is_empty());
        assert!(!row["equations"].as_array().unwrap().is_empty());
    }
    // A c_new = b row by row (the displayed system is solved by the displayed solution), to the
    // rounding of one row: the sum of the magnitudes of its terms times a few units of roundoff.
    let rows = v["rows"].as_array().unwrap();
    for (i, row) in rows.iter().enumerate() {
        let b = row["rhs_mol_m3"].as_f64().unwrap();
        let mut lhs = row["diag"].as_f64().unwrap() * c_new[i];
        let mut mag = lhs.abs();
        if i > 0 {
            let l = row["lower"].as_f64().unwrap() * c_new[i - 1];
            lhs += l;
            mag += l.abs();
        }
        if i < rows.len() - 1 {
            let u = row["upper"].as_f64().unwrap() * c_new[i + 1];
            lhs += u;
            mag += u.abs();
        }
        let tol = 8.0 * f64::EPSILON * (mag + b.abs());
        assert!((lhs - b).abs() <= tol, "row {i}: {lhs} vs {b}");
    }
    // The first step starts from the initial vector.
    let mut req1 = req.clone();
    req1["step"] = json!(1);
    req1["rows"] = json!(["first", 1, "interface:1", "last"]);
    let v1 = ok(strato_wasm::btcs_system, &req1);
    let (c0, _, _) = initial_vector(&grid, input.c_init);
    assert_eq!(
        v1["rows"][0]["terms"]["c_previous_step_mol_m3"]
            .as_f64()
            .unwrap(),
        c0[0]
    );
    assert_eq!(v1["rows"].as_array().unwrap().len(), 4);
    assert_eq!(v1["rows"][1]["lower"], json!(0.0)); // L_1 = 0 (F13)
    assert_eq!(v1["rows"][2]["class"], "interface");
}

#[test]
fn the_system_refuses_a_large_grid_and_bad_rows() {
    // The synthetic case has 200 intervals, the limit; one more is refused.
    ok(strato_wasm::btcs_system, &stage_request(1, None));
    let mut big = stage_request(1, None);
    big["config"]["regions"][0]["cells"] = json!(41);
    let e = rejected(strato_wasm::btcs_system, &big);
    assert!(e[0].contains("small grid"), "{e:?}");
    let mut cfg = config_value();
    for r in cfg["regions"].as_array_mut().unwrap() {
        r["cells"] = json!(4);
    }
    let base = json!({"schema": 3, "config": cfg, "stage": 1});
    for (extra, needle) in [
        (json!({"step": 0}), "step"),
        (json!({"step": 1_000_000}), "step"),
        (json!({"rows": ["middle"]}), "rows"),
        (json!({"rows": [20]}), "not a row"),
        (json!({"rows": ["interface:5"]}), "interface"),
        (json!({"rows": 3}), "rows"),
    ] {
        let mut r = base.clone();
        for (k, x) in extra.as_object().unwrap() {
            r[k] = x.clone();
        }
        let e = rejected(strato_wasm::btcs_system, &r);
        assert!(e[0].contains(needle), "{e:?}");
    }
    // Stage 1: no decay and no source, the terms are zero.
    let v = ok(strato_wasm::btcs_system, &base);
    assert_eq!(v["decay_number_kappa"], json!(0.0));
    assert_eq!(v["rows"][0]["terms"]["source_in_rhs_mol_m3"], json!(0.0));
}

#[test]
fn the_analytical_function_withholds_with_a_reason_and_refuses_a_varying_temperature() {
    // Stage 1: exact series. Values after the first valid time are drawn, earlier ones are null
    // with a reason.
    let v = ok(strato_wasm::analytical, &stage_request(1, None));
    assert_eq!(v["applicable"], true);
    assert!(v["extension"].is_null());
    let f = nullable(&v["release_fraction"]);
    let reasons = v["withheld_reason_codes"].as_array().unwrap();
    assert_eq!(f.len(), reasons.len());
    let mut drawn = 0;
    for (value, reason) in f.iter().zip(reasons) {
        // A value is either there with no reason, or absent with one; never zero by default.
        assert_eq!(value.is_some(), reason.is_null());
        if value.is_some() {
            drawn += 1;
        }
    }
    assert!(drawn > 0);
    assert_eq!(drawn as u64, v["drawn_count"].as_u64().unwrap());
    // The series is the BTCS release within the grid error: late values agree to a few percent
    // of the value on the 40-interval grid.
    let run = ok(strato_wasm::run_stage, &stage_request(1, None));
    let btcs = floats(&run["series"]["release_fraction"]);
    let last = f.len() - 1;
    let b = f[last].unwrap();
    let a = btcs[btcs.len() - 1];
    assert!((a - b).abs() <= 0.02 * b, "BTCS {a} vs series {b}");
    // Decay applies through its extension at a constant temperature; the temperature function
    // of a constant history is accepted.
    let v = ok(
        strato_wasm::analytical,
        &stage_request(3, Some(physics_json(3, ramp()))),
    );
    assert_eq!(v["applicable"], true);
    assert_eq!(v["extension"], "(B3) to (B5)");
    // Stage 4 with a ramp: refused, with the code and the reason, no values and no zeros.
    let v = ok(
        strato_wasm::analytical,
        &stage_request(4, Some(physics_json(4, ramp()))),
    );
    assert_eq!(v["applicable"], false);
    assert_eq!(v["reason_code"], "TIME_VARYING_TEMPERATURE");
    assert!(v["reason"].as_str().unwrap().contains("temperature"));
    assert!(v["release_fraction"].is_null());
    // Stage 4 at a constant temperature is the exact series with D evaluated at it.
    let v = ok(
        strato_wasm::analytical,
        &stage_request(4, Some(physics_json(4, flat(1000.0)))),
    );
    assert_eq!(v["applicable"], true);
    // The requested times are honoured, and a list that is too long is refused.
    let mut r = stage_request(1, None);
    r["times_s"] = json!([1.0e5, 2.0e5]);
    let v = ok(strato_wasm::analytical, &r);
    assert_eq!(v["times_s"], json!([1.0e5, 2.0e5]));
    r["times_s"] = json!(vec![1.0; 2001]);
    assert!(rejected(strato_wasm::analytical, &r)[0].contains("limit"));
}

#[test]
fn the_options_come_from_the_embedded_definitions() {
    let v = ok(strato_wasm::options, &json!({"schema": 3}));
    let stages = v["stages"].as_array().unwrap();
    assert_eq!(stages.len(), 5);
    assert_eq!(stages[0]["mechanisms"], json!([]));
    assert_eq!(
        stages[3]["mechanisms"],
        json!(["generation", "decay", "diffusivity_temperature"])
    );
    let sc = v["scenarios"].as_array().unwrap();
    let ids: Vec<&str> = sc.iter().map(|s| s["id"].as_str().unwrap()).collect();
    assert_eq!(
        ids,
        [
            "steady-state",
            "shutdown-cooldown",
            "startup",
            "control-rod-insertion",
            "refuelling"
        ]
    );
    assert!(sc[0]["variants"].as_array().unwrap().len() >= 3);
    assert_eq!(sc[0]["physics_stages"], json!([2, 3, 5]));
    let nuc: Vec<&str> = v["nuclides"]["scenario"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();
    assert_eq!(nuc, ["sr90", "cs137"]);
    let main = &v["temperatures"]["main_comparison_c"];
    assert_eq!(main["ag110m"], json!([1600.0, 1700.0]));
    assert_eq!(main["cs134"], json!([1600.0, 1700.0, 1800.0]));
    let expl = v["temperatures"]["exploration"].as_array().unwrap();
    assert_eq!(expl.len(), 3);
    assert_eq!(v["embedded_files"].as_array().unwrap().len(), 14);
    // The same text twice, and an empty request is accepted.
    assert_eq!(
        strato_wasm::options(""),
        strato_wasm::options(r#"{"schema":3}"#)
    );
    assert!(rejected(strato_wasm::options, &json!({"schema": 2}))[0].contains("schema"));
    assert!(rejected(strato_wasm::options, &json!({"x": 1}))[0].contains("schema"));
}

fn scenario_request(id: &str, nuclide: &str) -> Value {
    json!({"schema": 3, "scenario_id": id, "nuclide": nuclide})
}

#[test]
fn a_scenario_run_equals_the_curve_file_of_the_results_pipeline() {
    // The committed file of the pipeline for steady-state operation of Sr-90, run on the grid
    // that file states, with the same row selection.
    let file: Value = serde_json::from_str(
        &std::fs::read_to_string(path("data/results/curves/s5.steady-state.sr90.1028c.json"))
            .unwrap()
            .lines()
            .filter(|l| !l.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
    let grid: Vec<u64> = file["numerics"]["intervals_per_layer"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_u64().unwrap())
        .collect();
    let mut req = scenario_request("steady-state", "sr90");
    req["grid"] = json!({"intervals_per_layer": grid});
    req["max_rows"] = json!(220);
    let v = ok(strato_wasm::run_scenario, &req);
    let t_file = floats(&file["method_a"]["t_s"]);
    let t = floats(&v["series"]["times_s"]);
    assert_eq!(t.len(), t_file.len());
    let rel = |a: f64, b: f64| (a - b).abs() <= 1e-12 * a.abs().max(b.abs());
    for (a, b) in t.iter().zip(&t_file) {
        assert!(rel(*a, *b), "{a} {b}");
    }
    let f_file = nullable(&file["method_a"]["release_fraction"]);
    let f = nullable(&v["series"]["release_fraction_of_generated"]);
    for (a, b) in f.iter().zip(&f_file) {
        match (a, b) {
            (Some(a), Some(b)) => assert!(rel(*a, *b), "{a} {b}"),
            (None, None) => {}
            other => panic!("{other:?}"),
        }
    }
    let inv_file = floats(&file["quantities"]["inventory_mol"]);
    let inv = floats(&v["series"]["inventory_mol"]);
    for (a, b) in inv.iter().zip(&inv_file) {
        assert!(rel(*a, *b), "{a} {b}");
    }
    // The reference of normal operation is returned with its colour, on the same rows.
    let r = &v["reference_normal_operation"];
    assert_eq!(r["grey_line_colour"], "#6B7280");
    assert_eq!(
        floats(&r["series"]["times_s"]).len(),
        t.len(),
        "the reference is on the same rows"
    );
    // Steady state at a constant temperature: Method B is asked, and the answer is stated.
    assert!(v["method_b"]["applicable"].is_boolean());
    assert!(v["balance"]["within_bound"].as_bool().unwrap());
    assert_eq!(v["scenario"]["id"], "steady-state");
    assert!(!v["scenario"]["prescribed_inputs"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(v["nuclide"]["id"], "sr90");
}

#[test]
fn a_scenario_with_a_changing_temperature_has_no_method_b_value() {
    let mut req = scenario_request("shutdown-cooldown", "cs137");
    req["grid"] = json!({"intervals_per_layer": [40, 10, 10, 40, 10]});
    let v = ok(strato_wasm::run_scenario, &req);
    assert_eq!(v["method_b"]["applicable"], false);
    assert_eq!(v["method_b"]["reason_code"], "TIME_VARYING_TEMPERATURE");
    assert!(v["method_b"]["release_fraction"].is_null());
    // The event row is kept, and the post-event fraction starts there.
    let post = nullable(&v["series"]["post_event_fraction"]);
    assert!(post.iter().any(Option::is_some));
    assert!(post[0].is_none());
    // The temperature history of the definition is returned as a table.
    assert_eq!(v["histories"]["temperature_c"]["kind"], "table");
}

#[test]
fn scenario_requests_are_refused_with_messages() {
    let cases: Vec<(Value, &str)> = vec![
        (scenario_request("nothing", "sr90"), "scenario_id"),
        (
            scenario_request("steady-state", "ag110m"),
            "direct fission-product source",
        ),
        (
            {
                let mut r = scenario_request("steady-state", "sr90");
                r["variant"] = json!("no-such-variant");
                r
            },
            "variant",
        ),
        (
            {
                let mut r = scenario_request("startup", "sr90");
                r["physics_stage"] = json!(3);
                r
            },
            "physics_stage",
        ),
        (
            {
                let mut r = scenario_request("steady-state", "sr90");
                r["physics_stage"] = json!(4);
                r
            },
            "physics_stage",
        ),
        (
            {
                let mut r = scenario_request("steady-state", "sr90");
                r["step_multiplier"] = json!(0.5);
                r
            },
            "step_multiplier",
        ),
        (
            {
                let mut r = scenario_request("steady-state", "sr90");
                r["grid"] = json!({"intervals_per_layer": [1, 1, 1, 1, 100_000]});
                r
            },
            "limit",
        ),
        (
            {
                let mut r = scenario_request("steady-state", "sr90");
                r["grid"] = json!({"intervals_per_layer": [0, 1, 1, 1, 1]});
                r
            },
            "at least 1",
        ),
        (
            {
                let mut r = scenario_request("steady-state", "sr90");
                r["max_rows"] = json!(1);
                r
            },
            "max_rows",
        ),
    ];
    for (req, needle) in cases {
        let e = rejected(strato_wasm::run_scenario, &req);
        assert!(e[0].contains(needle), "{needle}: {e:?}");
    }
    let mut r = scenario_request("steady-state", "sr90");
    r["schema"] = json!(2);
    assert!(rejected(strato_wasm::run_scenario, &r)[0].contains("schema"));
}

#[test]
fn nothing_panics_on_awkward_input() {
    let inputs = [
        "",
        "null",
        "[]",
        "{}",
        r#"{"schema":3}"#,
        r#"{"schema":3,"config":5,"stage":1}"#,
        r#"{"schema":3,"config":{},"stage":"x"}"#,
        r#"{"schema":3,"stage":1,"config":{"regions":[]}}"#,
        "\u{0}\u{1}",
    ];
    let fns: [fn(&str) -> String; 4] = [
        strato_wasm::run_stage,
        strato_wasm::btcs_system,
        strato_wasm::analytical,
        strato_wasm::run_scenario,
    ];
    for f in fns {
        for i in inputs {
            let out = catch_unwind(AssertUnwindSafe(|| f(i))).expect("no panic");
            let v: Value = serde_json::from_str(&out).unwrap();
            assert_eq!(v["status"], "rejected", "{i:?}: {v}");
            assert!(!v["errors"].as_array().unwrap().is_empty());
        }
    }
    // The options function accepts an empty request.
    let v: Value = serde_json::from_str(&strato_wasm::options("")).unwrap();
    assert_eq!(v["status"], "ok");
}
