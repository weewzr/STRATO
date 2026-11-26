//! `case`: a STRATO input case (TOML) as `web/src/generated/<stem>.case.json` (imported by the web
//! build) and `web/public/data/<stem>.case.csv` (downloadable). Every value is copied unchanged,
//! so the floating-point values are the ones the TOML text denotes. No physics is computed here.

use serde_json::{json, Map, Value};
use std::path::Path;

use crate::csv_line;
use crate::pyfmt::{json_file, value_repr};
use crate::util::{self, Error, Result};
use crate::{bail, catalogue, CSV_DIR, GENERATOR, GENERATOR_VERSION, JSON_DIR, SCHEMA};

/// Units of the fields of strato::config::Config (src/config.rs at v1.0.1).
const UNITS: [(&str, &str); 8] = [
    ("temperature_k", "K"),
    ("outer_radius_m", "m"),
    ("diffusivity_m2_s", "m^2 s^-1"),
    ("cells", "1"),
    ("kernel_concentration_mol_m3", "mol m^-3"),
    ("dt_s", "s"),
    ("end_time_s", "s"),
    ("output_every_steps", "1"),
];

fn unit(key: &str) -> &'static str {
    UNITS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, u)| *u)
        .unwrap_or("")
}

fn leading_comments(text: &str) -> Vec<String> {
    let mut notes = Vec::new();
    for line in crate::pyfmt::splitlines(text) {
        let s = line.trim();
        if !s.starts_with('#') {
            break;
        }
        notes.push(s.trim_start_matches('#').trim().to_string());
    }
    notes
}

fn to_json(v: &toml::Value) -> Result<Value> {
    Ok(match v {
        toml::Value::String(s) => json!(s),
        toml::Value::Integer(i) => json!(i),
        toml::Value::Float(f) => match serde_json::Number::from_f64(*f) {
            Some(n) => Value::Number(n),
            None => bail!("a non-finite number cannot be written as JSON"),
        },
        toml::Value::Boolean(b) => json!(b),
        toml::Value::Array(a) => Value::Array(a.iter().map(to_json).collect::<Result<_>>()?),
        toml::Value::Table(t) => {
            let mut m = Map::new();
            for (k, v) in t {
                m.insert(k.clone(), to_json(v)?);
            }
            Value::Object(m)
        }
        toml::Value::Datetime(d) => json!(d.to_string()),
    })
}

fn key<'a>(t: &'a toml::Value, name: &str) -> Result<&'a toml::Value> {
    t.get(name)
        .ok_or_else(|| Error(format!("the input case has no key '{name}'")))
}

/// The JSON document and the CSV text of an input case.
pub fn build(root: &Path, input_path: &Path) -> Result<(Value, String)> {
    let rel = util::rel_posix(root, input_path);
    if rel.is_empty() || !input_path.starts_with(root) {
        bail!(
            "{}: the input case is not inside the repository",
            input_path.display()
        );
    }
    let raw = util::read_bytes(input_path)?;
    let text = util::decode_utf8(&raw, &rel)?;
    let cfg: toml::Value = text.parse().map_err(|e| Error(format!("{rel}: {e}")))?;

    // The keys mirror strato::config::Config exactly, so the object can later be sent to the
    // wasm wrapper unchanged. A missing key is an error here.
    let mut regions = Vec::new();
    let region_list = key(&cfg, "regions")?
        .as_array()
        .ok_or_else(|| Error("'regions' is not a list".into()))?;
    for r in region_list {
        regions.push(json!({
            "name": to_json(key(r, "name")?)?,
            "outer_radius_m": to_json(key(r, "outer_radius_m")?)?,
            "diffusivity_m2_s": to_json(key(r, "diffusivity_m2_s")?)?,
            "cells": to_json(key(r, "cells")?)?,
        }));
    }
    let initial = key(&cfg, "initial")?;
    let time = key(&cfg, "time")?;
    let config = json!({
        "case_name": to_json(key(&cfg, "case_name")?)?,
        "species": to_json(key(&cfg, "species")?)?,
        "temperature_k": to_json(key(&cfg, "temperature_k")?)?,
        "regions": regions,
        "initial": {
            "kernel_concentration_mol_m3": to_json(key(initial, "kernel_concentration_mol_m3")?)?
        },
        "time": {
            "dt_s": to_json(key(time, "dt_s")?)?,
            "end_time_s": to_json(key(time, "end_time_s")?)?,
            "output_every_steps": to_json(key(time, "output_every_steps")?)?,
        },
        "output": {"directory": to_json(key(key(&cfg, "output")?, "directory")?)?},
    });

    let notes = leading_comments(&text);
    let mut units = Map::new();
    for (k, u) in UNITS {
        units.insert(k.to_string(), json!(u));
    }
    let input_sha = util::sha256_hex(&raw);
    let input_commit = util::git(root, &["log", "-1", "--format=%H", "--", &rel])?;
    let version = catalogue::strato_version(root)?;
    let generator = format!("{GENERATOR} v{GENERATOR_VERSION}");
    let command = format!("python -I -X utf8 {GENERATOR} case {rel}");
    let created = util::now_utc();

    let mut doc = Map::new();
    doc.insert("schema".into(), json!(SCHEMA));
    doc.insert("label".into(), json!("INPUT"));
    doc.insert("kind".into(), json!("input_case"));
    doc.insert("generator".into(), json!(generator));
    doc.insert("strato_version".into(), json!(version));
    doc.insert("input_file".into(), json!(rel));
    doc.insert("input_sha256".into(), json!(input_sha));
    doc.insert("input_commit".into(), json!(input_commit));
    doc.insert("command".into(), json!(command));
    doc.insert("created_utc".into(), json!(created));
    doc.insert("notes".into(), json!(notes));
    doc.insert("units".into(), Value::Object(units));
    doc.insert("data".into(), json!({"config": config}));

    // CSV: provenance header as '# key: value' lines, then one row per value.
    let mut buf = String::new();
    for (k, v) in [
        ("label", "INPUT"),
        ("kind", "input_case"),
        ("generator", generator.as_str()),
        ("strato_version", version.as_str()),
        ("input_file", rel.as_str()),
        ("input_sha256", input_sha.as_str()),
        ("input_commit", input_commit.as_str()),
        ("command", command.as_str()),
        ("created_utc", created.as_str()),
    ] {
        buf.push_str(&format!("# {k}: {v}\n"));
    }
    for n in &notes {
        buf.push_str(&format!("# note: {n}\n"));
    }
    let s = |v: &Value| value_repr(v);
    let text_of = |v: &Value| v.as_str().unwrap_or("").to_string();
    buf.push_str(&csv_line!(
        "group",
        "region_index",
        "region_name",
        "key",
        "value",
        "unit"
    ));
    buf.push_str(&csv_line!(
        "case",
        "",
        "",
        "case_name",
        &text_of(&config["case_name"]),
        ""
    ));
    buf.push_str(&csv_line!(
        "case",
        "",
        "",
        "species",
        &text_of(&config["species"]),
        ""
    ));
    buf.push_str(&csv_line!(
        "case",
        "",
        "",
        "temperature_k",
        &s(&config["temperature_k"]),
        unit("temperature_k"),
    ));
    for (i, r) in config["regions"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        for k in ["outer_radius_m", "diffusivity_m2_s", "cells"] {
            buf.push_str(&csv_line!(
                "region",
                &i.to_string(),
                &text_of(&r["name"]),
                k,
                &s(&r[k]),
                unit(k),
            ));
        }
    }
    buf.push_str(&csv_line!(
        "initial",
        "",
        "",
        "kernel_concentration_mol_m3",
        &s(&config["initial"]["kernel_concentration_mol_m3"]),
        unit("kernel_concentration_mol_m3"),
    ));
    for k in ["dt_s", "end_time_s", "output_every_steps"] {
        buf.push_str(&csv_line!(
            "time",
            "",
            "",
            k,
            &s(&config["time"][k]),
            unit(k)
        ));
    }
    Ok((Value::Object(doc), buf))
}

pub fn run(root: &Path, input: &str, check: bool) -> Result<i32> {
    let input_path = if Path::new(input).is_absolute() {
        util::absolute(Path::new(input))?
    } else {
        util::absolute(&root.join(input))?
    };
    let stem = input_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let (doc, csv_text) = build(root, &input_path)?;
    let targets = [
        crate::Target {
            path: root.join(JSON_DIR).join(format!("{stem}.case.json")),
            text: json_file(&doc),
            is_json: true,
        },
        crate::Target {
            path: root.join(CSV_DIR).join(format!("{stem}.case.csv")),
            text: csv_text,
            is_json: false,
        },
    ];
    crate::write_or_check(root, &targets, check, &["created_utc"])
}
