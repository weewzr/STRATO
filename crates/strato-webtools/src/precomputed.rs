//! `precomputed`: run the native release binary and the native wasm wrapper on the synthetic
//! case and write their outputs to web/public/data/ as JSON and CSV with a PRECOMPUTED
//! provenance header (prompt/TECHNICAL_ARCHITECTURE.md Section 5): the time series and final
//! profile, the comparison with the exact series, the convergence study, a copy of the golden
//! folder for the browser parity check, and the native wrapper response, the reference of the
//! browser parity test. Values are copied from the program outputs; nothing is computed here.
//!
//! How the programs are called, and which files they write, is in [`crate::solver`].

use serde_json::{json, Map, Value};
use std::path::Path;
use std::process::Command;

use crate::colormaps::num;
use crate::pyfmt::{json_canonical, json_file, splitlines, universal_newlines};
use crate::solver as s;
use crate::util::{self, Error, Result};
use crate::{bail, catalogue, CSV_DIR, GENERATOR, GENERATOR_VERSION, SCHEMA};

/// Header fields that record when, where and with which tools a file was written; `--check`
/// ignores them (the data and the input hashes pin the content).
pub const PRECOMPUTED_VOLATILE: [&str; 3] = ["created_utc", "commit", "rustc"];

/// What the two programs produced.
pub struct RawRun {
    /// Standard output of the command line, with `\n` line ends.
    pub stdout: String,
    /// The four CSV files in the order of [`s::CLI_CSV`].
    pub csv: [String; 4],
    /// The wrapper's response.
    pub native: Value,
}

/// The golden files that the converted `golden_v1.json` copies: text and SHA-256 of the bytes.
pub struct GoldenFile {
    pub text: String,
    pub sha256: String,
}

/// Facts about the run that go into every header.
pub struct Context {
    pub commit: String,
    pub version: String,
    pub created: String,
    pub rustc: String,
    pub input_sha256: String,
}

/// CSV text to column arrays. Numbers are read as doubles (the double that the shortest
/// round-trip text of the CSV writer denotes); an empty cell is null; a cell that is not a
/// number stays text.
pub fn columns(text: &str) -> Result<Value> {
    let rows = util::csv_rows(text);
    let Some((head, body)) = rows.split_first() else {
        bail!("an empty CSV text");
    };
    let mut cols: Vec<(String, Vec<Value>)> = Vec::new();
    for h in head {
        if !cols.iter().any(|(k, _)| k == h) {
            cols.push((h.clone(), Vec::new()));
        }
    }
    for r in body {
        for (h, cell) in head.iter().zip(r) {
            let v = if cell.is_empty() {
                Value::Null
            } else if s::INTEGER_COLUMNS.contains(&h.as_str()) {
                match cell.trim().parse::<i64>() {
                    Ok(i) => json!(i),
                    Err(_) => bail!("column {h}: '{cell}' is not an integer"),
                }
            } else {
                match cell.trim().parse::<f64>() {
                    Ok(f) => num(f)?,
                    Err(_) => json!(cell),
                }
            };
            cols.iter_mut()
                .find(|(k, _)| k == h)
                .expect("column exists")
                .1
                .push(v);
        }
    }
    let mut m = Map::new();
    for (k, v) in cols {
        m.insert(k, Value::Array(v));
    }
    Ok(Value::Object(m))
}

/// The `key=value` lines of the command line's standard output, without the lines that name files.
pub fn stdout_summary(text: &str) -> Map<String, Value> {
    let mut out = Map::new();
    for line in splitlines(text) {
        if !line.contains('=') || line.starts_with("WARNING") {
            continue;
        }
        let (k, v) = line.split_once('=').expect("contains '='");
        if s::PATH_KEYS.contains(&k) {
            continue;
        }
        out.insert(k.to_string(), json!(v));
    }
    out
}

fn header(
    ctx: &Context,
    kind: &str,
    command: &str,
    extra_inputs: Option<Vec<Value>>,
    note: &str,
) -> Map<String, Value> {
    let mut h = Map::new();
    h.insert("schema".into(), json!(SCHEMA));
    h.insert("label".into(), json!("PRECOMPUTED"));
    h.insert("kind".into(), json!(kind));
    h.insert("generator".into(), json!(format!("strato {}", ctx.version)));
    h.insert(
        "converter".into(),
        json!(format!("{GENERATOR} v{GENERATOR_VERSION}")),
    );
    h.insert("strato_version".into(), json!(ctx.version));
    h.insert("commit".into(), json!(ctx.commit));
    h.insert("input_file".into(), json!(s::INPUT));
    h.insert("input_sha256".into(), json!(ctx.input_sha256));
    h.insert("command".into(), json!(command));
    h.insert("rustc".into(), json!(ctx.rustc));
    h.insert("created_utc".into(), json!(ctx.created));
    if let Some(inputs) = extra_inputs {
        h.insert("inputs".into(), json!(inputs));
    }
    if !note.is_empty() {
        h.insert("note".into(), json!(note));
    }
    h
}

fn with_data(mut h: Map<String, Value>, data: Value) -> Value {
    h.insert("data".into(), data);
    Value::Object(h)
}

fn csv_with_header(ctx: &Context, kind: &str, body: &str) -> String {
    let h = header(ctx, kind, s::CLI_TEXT, None, "");
    let mut lines = Vec::new();
    for k in [
        "label",
        "kind",
        "generator",
        "converter",
        "strato_version",
        "commit",
        "input_file",
        "input_sha256",
        "command",
        "rustc",
        "created_utc",
    ] {
        lines.push(format!("# {k}: {}", h[k].as_str().unwrap_or("")));
    }
    format!("{}\n{}", lines.join("\n"), body.replace("\r\n", "\n"))
}

/// The nine files of web/public/data/ as (name, text), in the order of
/// [`s::PRECOMPUTED_FILES`].
pub fn build_files(
    ctx: &Context,
    raw: &RawRun,
    golden: &[GoldenFile; 4],
) -> Result<Vec<(String, String)>> {
    let cli = s::CLI_TEXT;
    let summary = stdout_summary(&raw.stdout);
    let mut files: Vec<(String, String)> = Vec::new();
    let mut put = |name: &str, doc: &Value| files.push((name.to_string(), json_file(doc)));

    put(
        "synthetic_run.json",
        &with_data(
            header(ctx, "synthetic_run", cli, None, s::NOTE_RUN),
            json!({
                "timeseries": columns(&raw.csv[s::ROLE_TIMESERIES])?,
                "final_profile": columns(&raw.csv[s::ROLE_PROFILE])?,
                "summary": summary.clone(),
            }),
        ),
    );
    put(
        "comparison_fv_vs_analytical.json",
        &with_data(
            header(
                ctx,
                "comparison_fv_vs_analytical",
                cli,
                None,
                s::NOTE_COMPARISON,
            ),
            columns(&raw.csv[s::ROLE_COMPARISON])?,
        ),
    );
    let pick = |k: &str| summary.get(k).cloned().unwrap_or(Value::Null);
    put(
        "convergence_study.json",
        &with_data(
            header(ctx, "convergence_study", cli, None, s::NOTE_CONVERGENCE),
            json!({
                "points": columns(&raw.csv[s::ROLE_CONVERGENCE])?,
                "exact_final_release_fraction": pick(s::SUMMARY_KEYS[0]),
                "spatial_observed_orders": pick(s::SUMMARY_KEYS[1]),
                "temporal_observed_orders": pick(s::SUMMARY_KEYS[2]),
            }),
        ),
    );
    let golden_inputs: Vec<Value> = s::CLI_CSV
        .iter()
        .zip(golden)
        .map(|(n, g)| json!({"path": format!("{}/{n}", s::GOLDEN_DIR), "sha256": g.sha256}))
        .collect();
    let mut golden_data = Map::new();
    for (n, g) in s::CLI_CSV.iter().zip(golden) {
        golden_data.insert(n.trim_end_matches(".csv").to_string(), columns(&g.text)?);
    }
    put(
        "golden_v1.json",
        &with_data(
            header(
                ctx,
                "golden_v1",
                &format!("copy of {}/", s::GOLDEN_DIR),
                Some(golden_inputs),
                s::NOTE_GOLDEN,
            ),
            Value::Object(golden_data),
        ),
    );
    put(
        "wrapper_native_synthetic.json",
        &with_data(
            header(
                ctx,
                "wrapper_native_run",
                s::NATIVE_TEXT,
                None,
                s::NOTE_WRAPPER,
            ),
            raw.native.clone(),
        ),
    );
    files.push((
        "synthetic_run.timeseries.csv".into(),
        csv_with_header(
            ctx,
            "synthetic_run_timeseries",
            &raw.csv[s::ROLE_TIMESERIES],
        ),
    ));
    files.push((
        "synthetic_run.final_profile.csv".into(),
        csv_with_header(
            ctx,
            "synthetic_run_final_profile",
            &raw.csv[s::ROLE_PROFILE],
        ),
    ));
    files.push((
        "comparison_fv_vs_analytical.csv".into(),
        csv_with_header(
            ctx,
            "comparison_fv_vs_analytical",
            &raw.csv[s::ROLE_COMPARISON],
        ),
    ));
    files.push((
        "convergence_study.csv".into(),
        csv_with_header(ctx, "convergence_study", &raw.csv[s::ROLE_CONVERGENCE]),
    ));
    Ok(files)
}

/// A file is the same data as another when this form is equal.
pub fn comparable(name: &str, text: &str) -> Result<String> {
    if name.ends_with(".json") {
        let mut v: Value = serde_json::from_str(text)?;
        if let Value::Object(m) = &mut v {
            for k in PRECOMPUTED_VOLATILE {
                m.shift_remove(k);
            }
            if let Some(Value::Object(engine)) = m
                .get_mut("data")
                .and_then(Value::as_object_mut)
                .and_then(|d| d.get_mut("engine"))
            {
                engine.shift_remove("commit");
            }
        }
        return Ok(json_canonical(&v));
    }
    Ok(crate::comparable_text(text, &PRECOMPUTED_VOLATILE))
}

fn run_command(root: &Path, cmd: &[String], env: &[(&str, String)]) -> Result<String> {
    let mut c = Command::new(&cmd[0]);
    c.args(&cmd[1..]).current_dir(root);
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c
        .output()
        .map_err(|e| Error(format!("{}: {e}", cmd.join(" "))))?;
    if !out.status.success() {
        bail!(
            "{} failed ({}): {}",
            cmd.join(" "),
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(universal_newlines(&String::from_utf8_lossy(&out.stdout)))
}

/// Run both programs and read the files the command line wrote.
pub fn collect(root: &Path) -> Result<RawRun> {
    let stdout = run_command(root, &s::cli_command(), &[])?;
    let out_dir = root.join(s::CLI_OUT_DIR);
    let mut csv: Vec<String> = Vec::new();
    for name in s::CLI_CSV {
        csv.push(util::read_text(&out_dir.join(name))?);
    }
    let commit = commit_full(root);
    let env = [
        (
            "STRATO_BUILD_COMMIT",
            commit.chars().take(7).collect::<String>(),
        ),
        ("STRATO_BUILD_DIRTY", "0".to_string()),
    ];
    let native_text = run_command(root, &s::native_command(), &env)?;
    let native: Value = serde_json::from_str(&native_text)?;
    Ok(RawRun {
        stdout,
        csv: csv
            .try_into()
            .map_err(|_| Error("four CSV files expected".into()))?,
        native,
    })
}

fn commit_full(root: &Path) -> String {
    util::git(root, &["rev-parse", "HEAD"]).unwrap_or_else(|_| "unknown".to_string())
}

fn rustc_version(root: &Path) -> String {
    run_command(root, &["rustc".to_string(), "-V".to_string()], &[])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

pub fn read_golden(root: &Path) -> Result<[GoldenFile; 4]> {
    let mut v = Vec::new();
    for n in s::CLI_CSV {
        let p = root.join(s::GOLDEN_DIR).join(n);
        v.push(GoldenFile {
            text: util::read_text(&p)?,
            sha256: util::sha256_hex(&util::read_bytes(&p)?),
        });
    }
    v.try_into()
        .map_err(|_| Error("four golden files expected".into()))
}

/// The comparison of docs/development/provenance.md Section 3, automated: the four CSV files
/// byte for byte, the standard output after normalising path separators. Returns the number of
/// differences.
pub fn golden_comparison(root: &Path, raw: &RawRun) -> Result<usize> {
    let mut bad = 0;
    for (name, text) in s::CLI_CSV.iter().zip(&raw.csv) {
        let golden = util::read_bytes(&root.join(s::GOLDEN_DIR).join(name))?;
        let same = text.as_bytes() == golden.as_slice();
        println!(
            "{} {}/{name} against {}/{name} (byte for byte)",
            if same { "SAME   " } else { "DIFFERS" },
            s::CLI_OUT_DIR,
            s::GOLDEN_DIR
        );
        bad += usize::from(!same);
    }
    let want_text = util::read_text(&root.join(s::GOLDEN_DIR).join(s::GOLDEN_STDOUT))?;
    let want = splitlines(&want_text);
    let have_text = raw.stdout.replace('\\', "/");
    let have = splitlines(&have_text);
    let same = want == have;
    println!(
        "{} main-run standard output against {}/{} (path separators normalised, {} lines)",
        if same { "SAME   " } else { "DIFFERS" },
        s::GOLDEN_DIR,
        s::GOLDEN_STDOUT,
        have.len()
    );
    Ok(bad + usize::from(!same))
}

pub fn run(root: &Path, check: bool) -> Result<i32> {
    let raw = collect(root)?;
    let golden_bad = golden_comparison(root, &raw)?;
    let input = root.join(s::INPUT);
    let ctx = Context {
        commit: commit_full(root),
        version: catalogue::strato_version(root)?,
        created: util::now_utc(),
        rustc: rustc_version(root),
        input_sha256: util::sha256_hex(&util::read_bytes(&input)?),
    };
    let files = build_files(&ctx, &raw, &read_golden(root)?)?;
    if check {
        let mut bad = golden_bad;
        for (name, text) in &files {
            let path = root.join(CSV_DIR).join(name);
            let rel = util::rel_posix(root, &path);
            if !path.exists() {
                println!("MISSING {rel}");
                bad += 1;
            } else if comparable(name, &util::read_text(&path)?)? != comparable(name, text)? {
                println!("DIFFERS {rel}");
                bad += 1;
            } else {
                println!("SAME    {rel}");
            }
        }
        return Ok(i32::from(bad > 0));
    }
    for (name, text) in &files {
        let path = root.join(CSV_DIR).join(name);
        util::write_text(&path, text)?;
        println!("WROTE   {}", util::rel_posix(root, &path));
    }
    Ok(0)
}
