//! strato-webtools: the generators and checkers of the STRATO web demo.
//!
//! One command-line tool replaces `scripts/web/generate_data.py`, `check_catalogue.py`,
//! `build_wasm.py` and the two shell wrappers. The files it writes (web/src/generated and
//! web/public/data) have the same content as the Python generator wrote, apart from the
//! generator and command fields, which now name this tool, and the fields that record when,
//! where and with which tool a file was written (`created_utc`, `commit`, `generator_sha256`,
//! `rustc`), which `--check` ignores.
//!
//! The tool does not know which numerical method produced the results it converts. The solver is
//! reached only through [`solver`], which names its command lines and its output files.

pub mod case;
pub mod catalogue;
pub mod check;
pub mod cli;
pub mod colormaps;
pub mod geometry;
pub mod precomputed;
pub mod pyfmt;
pub mod solver;
pub mod util;
pub mod wasm;

use serde_json::Value;
use std::path::{Path, PathBuf};
use util::{Error, Result};

/// The identifier that the provenance headers of the generated files carry: this tool's own
/// name. (The retired Python generator wrote `scripts/web/generate_data.py` here.)
pub const GENERATOR: &str = "strato-webtools";
pub const GENERATOR_VERSION: u32 = 1;
pub const SCHEMA: u32 = 1;

/// The crate folder whose sources `generator_sha256` identifies.
pub const TOOL_DIR: &str = "crates/strato-webtools";

pub const JSON_DIR: &str = "web/src/generated";
pub const CSV_DIR: &str = "web/public/data";

/// Fields that describe the run, not the data: not compared by `--check` of the colour tables,
/// the geometry and the guide lines.
pub const VOLATILE: [&str; 3] = ["created_utc", "commit", "generator_sha256"];

/// SHA-256 over the sources of this tool (file names and bytes in name order): it identifies the
/// generator that wrote a file even when the tree was not clean.
pub fn generator_sha256(root: &Path) -> String {
    let dir = root.join(TOOL_DIR).join("src");
    let mut names: Vec<PathBuf> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.path())).collect(),
        Err(_) => return "unknown".to_string(),
    };
    names.sort();
    let mut data: Vec<u8> = Vec::new();
    for p in names {
        if let Ok(bytes) = std::fs::read(&p) {
            data.extend_from_slice(
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .as_bytes(),
            );
            data.push(0);
            data.extend_from_slice(&bytes);
        }
    }
    util::sha256_hex(&data)
}

/// The fixed form `json.dumps(d, sort_keys=True)` of a JSON text without the top-level fields
/// in `volatile`: two texts are the same data when this form is equal.
pub fn comparable_json(text: &str, volatile: &[&str]) -> Result<String> {
    let mut v: Value = serde_json::from_str(text)?;
    if let Value::Object(m) = &mut v {
        for k in volatile {
            m.shift_remove(*k);
        }
    }
    Ok(pyfmt::json_canonical(&v))
}

/// A text file without the `# key:` header lines named in `volatile`, lines joined by `\n`.
pub fn comparable_text(text: &str, volatile: &[&str]) -> String {
    pyfmt::splitlines(text)
        .into_iter()
        .filter(|l| !volatile.iter().any(|k| l.starts_with(&format!("# {k}:"))))
        .collect::<Vec<_>>()
        .join("\n")
}

/// One file to write or check: path, text and whether it is JSON.
pub struct Target {
    pub path: PathBuf,
    pub text: String,
    pub is_json: bool,
}

pub fn comparable(text: &str, is_json: bool, volatile: &[&str]) -> Result<String> {
    if is_json {
        comparable_json(text, volatile)
    } else {
        Ok(comparable_text(text, volatile))
    }
}

/// Print one line per file and write the files, or with `check` compare them with the committed
/// ones, ignoring the `volatile` fields. Returns the exit code.
pub fn write_or_check(
    root: &Path,
    targets: &[Target],
    check: bool,
    volatile: &[&str],
) -> Result<i32> {
    if check {
        let mut bad = 0;
        for t in targets {
            let rel = util::rel_posix(root, &t.path);
            if !t.path.exists() {
                println!("MISSING {rel}");
                bad += 1;
            } else if comparable(&util::read_text(&t.path)?, t.is_json, volatile)?
                != comparable(&t.text, t.is_json, volatile)?
            {
                println!("DIFFERS {rel}");
                bad += 1;
            } else {
                println!("SAME    {rel}");
            }
        }
        return Ok(i32::from(bad > 0));
    }
    for t in targets {
        util::write_text(&t.path, &t.text)?;
        println!("WROTE   {}", util::rel_posix(root, &t.path));
    }
    Ok(0)
}

/// An object member as text, for the many header fields that are plain strings.
pub fn str_field<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| Error(format!("missing text field '{key}'")))
}
