//! The `units` map of every reply that has one names every number, array of numbers and flag of
//! the reply (decision 76(d)): a reader of the reply never meets a quantity without its unit.
//!
//! Structural entries are exempt: the schema and ABI numbers, the echo of the stage, and the
//! sections of text, rows and per-layer objects (`profiles`, `rows`, `ranges`, `parts`).

use serde_json::{json, Value};
use std::collections::BTreeSet;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");

/// Keys of numbers that describe the reply itself, not a quantity.
const STRUCTURAL: [&str; 5] = ["schema", "abi", "stage", "status", "physics_stage"];
/// Sections that are not quantities of the reply or hold objects of their own.
const SKIPPED: [&str; 11] = [
    "engine",
    "solver",
    "mechanisms",
    "messages",
    "profiles",
    "rows",
    "symbolic",
    "equations",
    "ranges",
    "parts",
    "units",
];

fn is_quantity(v: &Value) -> bool {
    match v {
        Value::Number(_) | Value::Bool(_) => true,
        Value::Array(a) => {
            !a.is_empty()
                && a.iter().all(|x| match x {
                    Value::Null | Value::Number(_) | Value::Bool(_) => true,
                    Value::Array(b) => b.iter().all(|y| y.is_null() || y.is_number()),
                    _ => false,
                })
        }
        _ => false,
    }
}

fn collect(v: &Value, keys: &mut BTreeSet<String>) {
    if let Value::Object(m) = v {
        for (k, x) in m {
            if SKIPPED.contains(&k.as_str()) {
                continue;
            }
            if x.is_object() {
                collect(x, keys);
            } else if is_quantity(x) && !STRUCTURAL.contains(&k.as_str()) {
                keys.insert(k.clone());
            }
        }
    }
}

fn missing(reply: &Value, units: &Value) -> Vec<String> {
    let mut keys = BTreeSet::new();
    collect(reply, &mut keys);
    let have = units.as_object().expect("a units map");
    keys.into_iter().filter(|k| !have.contains_key(k)).collect()
}

fn request_text(name: &str) -> Value {
    let base: Value = toml::from_str(
        &std::fs::read_to_string(format!("{ROOT}inputs/synthetic/synthetic_five_layer.toml"))
            .unwrap(),
    )
    .unwrap();
    let mut small = base.clone();
    for k in 0..5 {
        small["regions"][k]["cells"] = json!(5);
    }
    small["time"] = json!({"dt_s": 2000.0, "end_time_s": 6000.0, "output_every_steps": 1});
    let ramp = json!({"unit": "degC", "interpolation": "linear", "times_s": [0.0, 2.0e5], "values": [900.0, 1100.0]});
    let names = ["Kernel", "Buffer", "IPyC", "SiC", "OPyC"];
    let d0 = [1.0e-6, 2.0e-7, 5.0e-6, 3.0e-5, 8.0e-6];
    let ea = [200.0, 150.0, 220.0, 260.0, 210.0];
    let laws: Vec<Value> = (0..5)
        .map(|k| {
            json!({"region": names[k], "source": "test input", "class": "user-supplied, no source",
                   "terms": [{"d0_m2_s": d0[k], "activation_energy_kj_per_mol": ea[k],
                              "valid_from_k": 1000.0, "valid_to_k": 1500.0}]})
        })
        .collect();
    let physics = |stage: u8| {
        let mut p = serde_json::Map::new();
        p.insert(
            "source".into(),
            json!({"mode": "kernel_fission_yield", "product": "direct_fission_product",
                   "yield_kind": "cumulative", "yield_atoms_per_fission": 0.06,
                   "fission_rate_origin": "test input", "fission_rate_density": {"value": 1.0e19}}),
        );
        p.insert("decay".into(), json!({"half_life_s": 1.0e5}));
        if stage >= 4 {
            p.insert("temperature".into(), ramp.clone());
            p.insert("arrhenius".into(), Value::Array(laws.clone()));
        }
        Value::Object(p)
    };
    match name {
        "run_stage" => json!({"schema": 3, "config": base, "stage": 4, "physics": physics(4)}),
        "analytical" => json!({"schema": 3, "config": base, "stage": 3, "physics": physics(3),
                               "times_s": [1.0e4, 5.0e4, 1.0e5, 2.0e5]}),
        "btcs_system" => json!({"schema": 3, "config": small, "stage": 4, "physics": physics(4),
                                "step": 2, "rows": ["first", 1, "interface:1", "last"]}),
        "run_scenario" => json!({"schema": 3, "scenario_id": "steady-state", "nuclide": "cs137",
                                 "grid": {"intervals_per_layer": [40, 10, 10, 40, 10]},
                                 "check_resolution": true, "max_rows": 120}),
        other => panic!("{other}"),
    }
}

fn reply(name: &str) -> Value {
    let text = request_text(name).to_string();
    let out = match name {
        "run_stage" => strato_wasm::stage_api::run_stage(&text),
        "analytical" => strato_wasm::stage_api::analytical(&text),
        "btcs_system" => strato_wasm::stage_api::btcs_system(&text),
        "run_scenario" => strato_wasm::scenario_api::run_scenario(&text),
        other => panic!("{other}"),
    };
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["status"], "ok", "{name}: {out:.300}");
    v
}

#[test]
fn every_quantity_of_a_stage_reply_has_a_unit_entry() {
    let r = reply("run_stage");
    assert_eq!(missing(&r, &r["units"]), Vec::<String>::new());
}

#[test]
fn every_quantity_of_an_analytical_reply_has_a_unit_entry() {
    let r = reply("analytical");
    assert_eq!(missing(&r, &r["units"]), Vec::<String>::new());
}

#[test]
fn every_quantity_of_a_system_reply_has_a_unit_entry() {
    let r = reply("btcs_system");
    assert_eq!(missing(&r, &r["units"]), Vec::<String>::new());
}

#[test]
fn every_quantity_of_a_scenario_reply_has_a_unit_entry() {
    let r = reply("run_scenario");
    // The Method B part carries its own map.
    let mut own = r.clone();
    let method_b = own.as_object_mut().unwrap().remove("method_b").unwrap();
    assert_eq!(missing(&own, &r["units"]), Vec::<String>::new());
    assert_eq!(missing(&method_b, &method_b["units"]), Vec::<String>::new());
}

#[test]
fn the_balance_note_of_the_run_reply_is_a_sentence_without_the_name_of_a_brief() {
    let config: Value = toml::from_str(
        &std::fs::read_to_string(format!("{ROOT}inputs/synthetic/synthetic_five_layer.toml"))
            .unwrap(),
    )
    .unwrap();
    let request = json!({"schema": 2, "config": config}).to_string();
    let v: Value = serde_json::from_str(&strato_wasm::run(&request)).unwrap();
    for field in ["statement", "generated_and_decayed_note"] {
        let s = v["balance"][field].as_str().unwrap();
        assert!(s.chars().next().unwrap().is_uppercase(), "{s}");
        assert!(!s.to_lowercase().contains("directive"), "{s}");
    }
    let stage = reply("run_stage");
    let s = stage["balance"]["statement"].as_str().unwrap();
    assert!(s.chars().next().unwrap().is_uppercase() && !s.to_lowercase().contains("directive"));
}
