//! `colormaps`: the cividis and viridis tables of data/colormaps/ (copied from matplotlib, see
//! data/colormaps/README.md) as `web/src/generated/colormaps.json`, the colour tables of the
//! particle colouring. Display data only.

use serde_json::{json, Map, Value};
use std::path::Path;

use crate::pyfmt::json_file;
use crate::util::{self, Result};
use crate::{bail, catalogue, GENERATOR, GENERATOR_VERSION, JSON_DIR, SCHEMA};

pub const COLORMAP_DIR: &str = "data/colormaps";
pub const COLORMAP_NAMES: [&str; 2] = ["cividis", "viridis"];

fn parse_float(s: &str) -> Result<f64> {
    s.trim()
        .parse::<f64>()
        .map_err(|_| util::Error(format!("'{s}' is not a number")))
}

/// Copy the 256-entry tables of data/colormaps/ and add their 8-bit form.
///
/// The 8-bit form is matplotlib's own conversion (`Colormap.__call__` with `bytes=True`:
/// `(lut * 255).astype(uint8)`, which truncates), so a colour on the page equals
/// `matplotlib.colormaps[name](i, bytes=True)`.
pub fn build(root: &Path) -> Result<Value> {
    let mut inputs = Vec::new();
    let mut maps = Map::new();
    for name in COLORMAP_NAMES {
        let rel = format!("{COLORMAP_DIR}/{name}.csv");
        let raw = util::read_bytes(&root.join(&rel))?;
        inputs.push(json!({"path": rel, "sha256": util::sha256_hex(&raw)}));
        let rows = util::csv_rows(&util::decode_utf8(&raw, &rel)?);
        let want: Vec<String> = ["index", "r", "g", "b"].map(String::from).to_vec();
        if rows.first() != Some(&want) {
            bail!("{rel}: unexpected header {:?}", rows.first());
        }
        let body: Vec<&Vec<String>> = rows[1..].iter().filter(|r| !r.is_empty()).collect();
        let mut indices = Vec::new();
        for r in &body {
            match r[0].trim().parse::<i64>() {
                Ok(i) => indices.push(i),
                Err(_) => bail!("{rel}: expected the indices 0 to 255 in order"),
            }
        }
        if indices != (0..256).collect::<Vec<i64>>() {
            bail!("{rel}: expected the indices 0 to 255 in order");
        }
        let mut rgb: Vec<[f64; 3]> = Vec::new();
        for r in &body {
            rgb.push([
                parse_float(&r[1])?,
                parse_float(&r[2])?,
                parse_float(&r[3])?,
            ]);
        }
        for t in &rgb {
            if !t.iter().all(|v| (0.0..=1.0).contains(v)) {
                bail!("{rel}: component outside [0, 1]");
            }
        }
        let rgb_json: Vec<Value> = rgb
            .iter()
            .map(|t| {
                t.iter()
                    .map(|v| num(*v))
                    .collect::<Result<Vec<_>>>()
                    .map(Value::Array)
            })
            .collect::<Result<_>>()?;
        let rgb8: Vec<Value> = rgb
            .iter()
            .map(|t| json!(t.iter().map(|v| (v * 255.0) as i64).collect::<Vec<_>>()))
            .collect();
        maps.insert(
            name.to_string(),
            json!({"entries": rgb.len(), "rgb": rgb_json, "rgb8": rgb8}),
        );
    }
    let readme = format!("{COLORMAP_DIR}/README.md");
    inputs.push(json!({
        "path": readme,
        "sha256": util::sha256_hex(&util::read_bytes(&root.join(&readme))?),
    }));
    let mut doc = Map::new();
    doc.insert("schema".into(), json!(SCHEMA));
    doc.insert("label".into(), json!("DISPLAY"));
    doc.insert("kind".into(), json!("colormaps"));
    doc.insert(
        "generator".into(),
        json!(format!("{GENERATOR} v{GENERATOR_VERSION}")),
    );
    doc.insert(
        "strato_version".into(),
        json!(catalogue::strato_version(root)?),
    );
    doc.insert("commit".into(), json!(util::head_commit(root)));
    doc.insert(
        "command".into(),
        json!(format!("python -I -X utf8 {GENERATOR} colormaps")),
    );
    doc.insert("created_utc".into(), json!(util::now_utc()));
    doc.insert(
        "note".into(),
        json!(concat!(
            "Colour tables for displaying solver values; not data and not a result label. ",
            "Copied from matplotlib 3.11.2 lib/matplotlib/_cm_listed.py (see data/colormaps/README.md). ",
            "rgb8 = int(255 * rgb), matplotlib's bytes conversion."
        )),
    );
    doc.insert("inputs".into(), json!(inputs));
    doc.insert("data".into(), Value::Object(maps));
    Ok(Value::Object(doc))
}

/// A finite double as a JSON number.
pub fn num(v: f64) -> Result<Value> {
    match serde_json::Number::from_f64(v) {
        Some(n) => Ok(Value::Number(n)),
        None => bail!("a non-finite number cannot be written as JSON"),
    }
}

pub fn run(root: &Path, check: bool) -> Result<i32> {
    let doc = build(root)?;
    let targets = [crate::Target {
        path: root.join(JSON_DIR).join("colormaps.json"),
        text: json_file(&doc),
        is_json: true,
    }];
    crate::write_or_check(root, &targets, check, &crate::VOLATILE)
}
