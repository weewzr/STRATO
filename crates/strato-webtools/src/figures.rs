//! `figures`: reading aids of the verification figures (web phase 5).
//!
//! Reads web/public/data/convergence_study.json (PRECOMPUTED) and writes
//! web/public/data/convergence_guides.json and .csv: the guide lines of slope 2 (spatial) and 1
//! (temporal) through the coarsest point of each refinement study, drawn on the convergence plots
//! of the Verify section as a reading aid (label DISPLAY, not a result). The SHA-256 of the
//! input's `data` object pins the input.

use serde_json::{json, Map, Value};
use std::path::Path;

use crate::colormaps::num;
use crate::pyfmt::{float_repr, json_canonical, json_file, value_repr};
use crate::util::{self, Error, Result};
use crate::{bail, catalogue, csv_line, CSV_DIR, GENERATOR, GENERATOR_VERSION, SCHEMA, TOOL_DIR};

pub const CONVERGENCE_JSON: &str = "web/public/data/convergence_study.json";

/// Reference slopes of prompt/DESIGN_SPECIFICATION.md Section 8.3: second order in space, first
/// order in time (docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md Section 6.7).
const GUIDE_SLOPES: [(&str, i32); 2] = [("spatial", 2), ("temporal", 1)];

fn column<'a>(pts: &'a Value, name: &str) -> Result<&'a Vec<Value>> {
    pts.get(name)
        .and_then(Value::as_array)
        .ok_or_else(|| Error(format!("{CONVERGENCE_JSON}: no column '{name}'")))
}

fn number(v: &Value) -> Result<f64> {
    v.as_f64()
        .ok_or_else(|| Error(format!("{CONVERGENCE_JSON}: '{v}' is not a number")))
}

/// Guide lines of slope 2 (spatial) and 1 (temporal) for the convergence plots.
///
/// Each guide passes through the coarsest point of its study: for the spatial study
/// `e_0 (N_0 / N)^2` over the total cell counts N, for the temporal study `e_0 (dt / dt_0)^1`
/// over the time steps dt, where `e_0` is the error the native binary printed for the coarsest
/// level. A reading aid on a log-log plot (the straight line a method of that order would
/// follow), not a result: label DISPLAY.
pub fn build(root: &Path) -> Result<(Value, String)> {
    let src: Value = serde_json::from_str(&util::read_text(&root.join(CONVERGENCE_JSON))?)?;
    if src.get("label").and_then(Value::as_str) != Some("PRECOMPUTED")
        || src.get("kind").and_then(Value::as_str) != Some("convergence_study")
    {
        bail!("{CONVERGENCE_JSON}: not the precomputed convergence study");
    }
    let pts = &src["data"]["points"];
    let studies = column(pts, "study")?;
    let errors = column(pts, "abs_error_vs_exact")?;
    let mut guides = Map::new();
    for (study, slope) in GUIDE_SLOPES {
        let idx: Vec<usize> = studies
            .iter()
            .enumerate()
            .filter(|(_, s)| s.as_str() == Some(study))
            .map(|(i, _)| i)
            .collect();
        if idx.len() < 2 {
            bail!("{CONVERGENCE_JSON}: fewer than two {study} points");
        }
        let e0 = number(&errors[idx[0]])?;
        let (x_name, x_col) = if study == "spatial" {
            ("total_cells", column(pts, "total_cells")?)
        } else {
            ("dt_s", column(pts, "dt_s")?)
        };
        let x: Vec<Value> = idx.iter().map(|&i| x_col[i].clone()).collect();
        let x0 = number(&x[0])?;
        let mut y = Vec::new();
        for xv in &x {
            let xf = number(xv)?;
            // Python: (x0 / n) ** 2 for the spatial study, (dt / dt0) ** 1 for the temporal one.
            let ratio = if study == "spatial" { x0 / xf } else { xf / x0 };
            let powered = match slope {
                1 => ratio,
                2 => ratio * ratio,
                _ => ratio.powi(slope),
            };
            y.push(num(e0 * powered)?);
        }
        guides.insert(
            study.to_string(),
            json!({
                "x_name": x_name,
                "x": x,
                "slope": slope,
                "guide_abs_error": y,
                "anchor_abs_error": num(e0)?,
            }),
        );
    }
    let data_sha = util::sha256_hex(json_canonical(&src["data"]).as_bytes());
    let generator = format!("{GENERATOR} v{GENERATOR_VERSION}");
    let gen_sha = crate::generator_sha256(root);
    let version = catalogue::strato_version(root)?;
    let commit = util::source_commit(root, &[TOOL_DIR, CONVERGENCE_JSON]);
    let command = format!("python -I -X utf8 {GENERATOR} figures");
    let created = util::now_utc();
    let note = concat!(
        "Guide lines of the convergence plots: slope 2 over the total cell count (spatial study) and slope 1 over the ",
        "time step (temporal study), each through the coarsest point of its study. A reading aid, not a result."
    );
    let src_commit = src.get("commit").cloned().unwrap_or(json!(""));
    let mut doc = Map::new();
    doc.insert("schema".into(), json!(SCHEMA));
    doc.insert("label".into(), json!("DISPLAY"));
    doc.insert("kind".into(), json!("convergence_guides"));
    doc.insert("generator".into(), json!(generator));
    doc.insert("generator_sha256".into(), json!(gen_sha));
    doc.insert("strato_version".into(), json!(version));
    doc.insert("commit".into(), json!(commit));
    doc.insert("command".into(), json!(command));
    doc.insert("created_utc".into(), json!(created));
    doc.insert("note".into(), json!(note));
    doc.insert(
        "inputs".into(),
        json!([{"path": CONVERGENCE_JSON, "data_sha256": data_sha, "commit": src_commit}]),
    );
    doc.insert("data".into(), Value::Object(guides.clone()));

    let mut buf = String::new();
    for (k, v) in [
        ("label", "DISPLAY"),
        ("kind", "convergence_guides"),
        ("generator", generator.as_str()),
        ("generator_sha256", gen_sha.as_str()),
        ("strato_version", version.as_str()),
        ("commit", commit.as_str()),
        ("command", command.as_str()),
        ("created_utc", created.as_str()),
        ("note", note),
    ] {
        buf.push_str(&format!("# {k}: {v}\n"));
    }
    buf.push_str(&format!(
        "# input: {CONVERGENCE_JSON} (data_sha256 {data_sha})\n"
    ));
    buf.push_str(&csv_line!(
        "study",
        "x_name",
        "x",
        "slope",
        "guide_abs_error"
    ));
    for (study, g) in &guides {
        let xs = g["x"].as_array().cloned().unwrap_or_default();
        let ys = g["guide_abs_error"].as_array().cloned().unwrap_or_default();
        for (xv, yv) in xs.iter().zip(&ys) {
            buf.push_str(&csv_line!(
                study.as_str(),
                g["x_name"].as_str().unwrap_or(""),
                value_repr(xv),
                value_repr(&g["slope"]),
                float_repr(number(yv)?)
            ));
        }
    }
    Ok((Value::Object(doc), buf))
}

pub fn run(root: &Path, check: bool) -> Result<i32> {
    let (doc, csv_text) = build(root)?;
    let targets = [
        crate::Target {
            path: root.join(CSV_DIR).join("convergence_guides.json"),
            text: json_file(&doc),
            is_json: true,
        },
        crate::Target {
            path: root.join(CSV_DIR).join("convergence_guides.csv"),
            text: csv_text,
            is_json: false,
        },
    ];
    crate::write_or_check(root, &targets, check, &crate::VOLATILE)
}
