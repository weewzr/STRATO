//! `precomputed`: run the native BTCS command line and the native wasm wrapper on the synthetic
//! case and write their outputs to web/public/data/ as JSON and CSV with a PRECOMPUTED
//! provenance header (prompt/TECHNICAL_ARCHITECTURE.md Section 5): the time series, profiles and
//! diagnostics of the run, the release fraction against the analytical (Sturm-Liouville series)
//! solution, the independent check with the observed orders, and the native wrapper reply, the
//! reference of the browser parity test. Values are copied from the program outputs; the only
//! arithmetic is the absolute difference of two release fractions of the same output time.
//!
//! How the programs are called, and which files they write, is in [`crate::solver`].

use serde_json::{json, Map, Value};
use std::path::Path;
use std::process::Command;

use crate::colormaps::num;
use crate::pyfmt::{float_repr, json_canonical, json_file, splitlines, universal_newlines};
use crate::solver as s;
use crate::util::{self, Error, Result};
use crate::{bail, catalogue, CSV_DIR, SCHEMA};

/// Name and format version of the converter that the headers record.
pub const CONVERTER: &str = "strato-webtools precomputed";
pub const CONVERTER_VERSION: u32 = 2;

/// Header fields that record when, where and with which tools a file was written; `--check`
/// ignores them (the data and the input hashes pin the content).
pub const PRECOMPUTED_VOLATILE: [&str; 3] = ["created_utc", "commit", "rustc"];

/// What the programs produced.
pub struct RawRun {
    /// Standard output of the command line, with `\n` line ends.
    pub stdout: String,
    /// The time-series CSV file the command line wrote.
    pub timeseries_csv: String,
    /// The profile CSV file the command line wrote.
    pub profiles_csv: String,
    /// The JSON file the command line wrote.
    pub run_json: Value,
    /// The wrapper's reply.
    pub native: Value,
    /// The cases of the parity battery: name to {function, input, reply}.
    pub battery: Value,
    /// The independent-check CSV file of the regression suite.
    pub check_csv: String,
}

/// Facts about the run that go into every header.
pub struct Context {
    pub commit: String,
    pub version: String,
    pub created: String,
    pub rustc: String,
    pub input_sha256: String,
    pub check_sha256: String,
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
    h.insert("solver".into(), json!(s::SOLVER_ID));
    h.insert("generator".into(), json!(format!("strato {}", ctx.version)));
    h.insert(
        "converter".into(),
        json!(format!("{CONVERTER} v{CONVERTER_VERSION}")),
    );
    h.insert("strato_version".into(), json!(ctx.version));
    h.insert("commit".into(), json!(ctx.commit));
    h.insert("input_file".into(), json!(s::INPUT));
    // The SHA-256 of the input file as it is on disk. The solver's own header, inside the CSV
    // bodies, has a different quantity (the digest of the canonical text of the validated input).
    h.insert("input_file_sha256".into(), json!(ctx.input_sha256));
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

fn csv_with_header(ctx: &Context, kind: &str, command: &str, body: &str) -> String {
    let h = header(ctx, kind, command, None, "");
    let mut lines = Vec::new();
    for k in [
        "label",
        "kind",
        "solver",
        "generator",
        "converter",
        "strato_version",
        "commit",
        "input_file",
        "input_file_sha256",
        "command",
        "rustc",
        "created_utc",
    ] {
        lines.push(format!("# {k}: {}", h[k].as_str().unwrap_or("")));
    }
    format!("{}\n{}", lines.join("\n"), body.replace("\r\n", "\n"))
}

/// The numbers of a column of the native reply as doubles.
fn float_array(v: &Value, what: &str) -> Result<Vec<f64>> {
    let Some(a) = v.as_array() else {
        bail!("{what}: not an array");
    };
    a.iter()
        .map(|x| {
            x.as_f64()
                .ok_or_else(|| Error(format!("{what}: '{x}' is not a number")))
        })
        .collect()
}

/// The release fraction of the BTCS run against the analytical series, from the wrapper's reply:
/// one row per output time of the run; the analytical column is empty (null) where the series is
/// not valid, and the absolute difference is the difference of the two numbers of the same row.
/// The columns are returned as JSON and as the body of a CSV file with the full physical names.
pub fn vs_analytical(native: &Value) -> Result<(Value, String)> {
    let times = float_array(&native["times_s"], "times_s")?;
    let btcs = float_array(&native["release_fraction"], "release_fraction")?;
    if times.len() != btcs.len() {
        bail!("times_s and release_fraction differ in length");
    }
    let a = &native["analytical"];
    if !a.is_object() {
        bail!("the wrapper reply has no analytical series");
    }
    let a_times = float_array(&a["times_s"], "analytical.times_s")?;
    let a_values = float_array(&a["release_fraction"], "analytical.release_fraction")?;
    if a_times.len() != a_values.len() {
        bail!("analytical times and values differ in length");
    }
    let mut matched = 0usize;
    let mut series: Vec<Option<f64>> = Vec::new();
    for t in &times {
        let k = a_times.iter().position(|x| x == t);
        series.push(k.map(|k| a_values[k]));
        matched += usize::from(k.is_some());
    }
    if matched != a_times.len() {
        bail!(
            "{} analytical times are not output times of the run",
            a_times.len() - matched
        );
    }
    let diff: Vec<Option<f64>> = btcs
        .iter()
        .zip(&series)
        .map(|(b, s)| s.map(|s| (b - s).abs()))
        .collect();
    let opt = |v: &[Option<f64>]| -> Result<Vec<Value>> {
        v.iter()
            .map(|x| x.map_or(Ok(Value::Null), num))
            .collect::<Result<Vec<_>>>()
    };
    let cols = |v: &[f64]| -> Result<Vec<Value>> { v.iter().map(|x| num(*x)).collect() };
    let cmp = &native["diagnostics"]["analytical_comparison"];
    let data = json!({
        "t_s": cols(&times)?,
        "release_fraction_btcs": cols(&btcs)?,
        "release_fraction_analytical": opt(&series)?,
        "abs_difference": opt(&diff)?,
        "analytical": {
            "name": a["name"],
            "modes": a["modes"],
            "accuracy_floor": a["accuracy_floor"],
            "valid_from_s": a["valid_from_s"],
            "max_abs_difference": cmp["max_abs_difference"],
            "final_abs_difference": cmp["final_abs_difference"],
            "resolved": cmp["resolved"],
        },
        "column_headers": {
            "t_s": "Time (s)",
            "release_fraction_btcs": "Release fraction by Method A: BTCS (dimensionless)",
            "release_fraction_analytical": "Release fraction by Method B: Sturm–Liouville (dimensionless)",
            "abs_difference": "Absolute difference of the release fractions (dimensionless)",
        },
    });
    let mut body = String::from(
        "Time (s),Release fraction by Method A: BTCS (dimensionless),Release fraction by Method B: Sturm–Liouville (dimensionless),Absolute difference of the release fractions (dimensionless)\n",
    );
    for i in 0..times.len() {
        let f = |x: Option<f64>| x.map_or(String::new(), float_repr);
        body.push_str(&format!(
            "{},{},{},{}\n",
            float_repr(times[i]),
            float_repr(btcs[i]),
            f(series[i]),
            f(diff[i])
        ));
    }
    Ok((data, body))
}

/// The independent-check file of the regression suite as columns under the machine keys of
/// [`s::CHECK_KEYS`], with the header comments as notes and the descriptive headers as
/// `column_headers`.
pub fn check_data(text: &str) -> Result<Value> {
    let mut notes: Vec<Value> = Vec::new();
    let mut rows: Vec<Vec<&str>> = Vec::new();
    let mut head: Option<Vec<&str>> = None;
    for line in splitlines(text) {
        if let Some(c) = line.strip_prefix('#') {
            notes.push(json!(c.trim()));
        } else if line.trim().is_empty() {
            continue;
        } else if head.is_none() {
            head = Some(line.split(',').collect());
        } else {
            rows.push(line.split(',').collect());
        }
    }
    let Some(head) = head else {
        bail!("{}: no header row", s::CHECK_FILE);
    };
    if head.len() != s::CHECK_KEYS.len() {
        bail!(
            "{}: {} columns, {} expected",
            s::CHECK_FILE,
            head.len(),
            s::CHECK_KEYS.len()
        );
    }
    let mut data = Map::new();
    let mut names = Map::new();
    for (k, (key, h)) in s::CHECK_KEYS.iter().zip(&head).enumerate() {
        let mut col: Vec<Value> = Vec::new();
        for r in &rows {
            let Some(cell) = r.get(k) else {
                bail!(
                    "{}: a row has fewer than {} cells",
                    s::CHECK_FILE,
                    head.len()
                );
            };
            col.push(if *key == "passed" {
                match *cell {
                    "true" => json!(true),
                    "false" => json!(false),
                    other => bail!("{}: '{other}' is not true or false", s::CHECK_FILE),
                }
            } else {
                match cell.trim().parse::<f64>() {
                    Ok(f) => num(f)?,
                    Err(_) => bail!("{}: '{cell}' is not a number", s::CHECK_FILE),
                }
            });
        }
        data.insert((*key).to_string(), Value::Array(col));
        names.insert((*key).to_string(), json!(h));
    }
    data.insert("column_headers".into(), Value::Object(names));
    data.insert("notes".into(), Value::Array(notes));
    Ok(Value::Object(data))
}

/// The files of web/public/data/ as (name, text), in the order of [`s::PRECOMPUTED_FILES`].
pub fn build_files(ctx: &Context, raw: &RawRun) -> Result<Vec<(String, String)>> {
    let cli = s::CLI_TEXT;
    let summary = stdout_summary(&raw.stdout);
    let mut files: Vec<(String, String)> = Vec::new();
    let mut put = |name: &str, doc: &Value| files.push((name.to_string(), json_file(doc)));

    put(
        "btcs_run.json",
        &with_data(
            header(ctx, "btcs_run", cli, None, s::NOTE_RUN),
            json!({"run": raw.run_json.clone(), "summary": summary}),
        ),
    );
    let (vs, vs_body) = vs_analytical(&raw.native)?;
    put(
        "btcs_vs_analytical.json",
        &with_data(
            header(
                ctx,
                "btcs_vs_analytical",
                s::NATIVE_TEXT,
                None,
                s::NOTE_COMPARISON,
            ),
            vs,
        ),
    );
    let check_inputs = vec![json!({"path": s::CHECK_FILE, "sha256": ctx.check_sha256})];
    put(
        "btcs_independent_check.json",
        &with_data(
            header(
                ctx,
                "btcs_independent_check",
                &format!("copy of {}", s::CHECK_FILE),
                Some(check_inputs),
                s::NOTE_CHECK,
            ),
            check_data(&raw.check_csv)?,
        ),
    );
    put(
        "wrapper_native_battery.json",
        &with_data(
            header(
                ctx,
                "wrapper_native_battery",
                s::BATTERY_TEXT,
                None,
                s::NOTE_BATTERY,
            ),
            raw.battery.clone(),
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
        "btcs_run.timeseries.csv".into(),
        csv_with_header(ctx, "btcs_run_timeseries", cli, &raw.timeseries_csv),
    ));
    files.push((
        "btcs_run.profiles.csv".into(),
        csv_with_header(ctx, "btcs_run_profiles", cli, &raw.profiles_csv),
    ));
    files.push((
        "btcs_vs_analytical.csv".into(),
        csv_with_header(ctx, "btcs_vs_analytical", s::NATIVE_TEXT, &vs_body),
    ));
    files.push((
        "btcs_independent_check.csv".into(),
        csv_with_header(
            ctx,
            "btcs_independent_check",
            &format!("copy of {}", s::CHECK_FILE),
            &raw.check_csv,
        ),
    ));
    Ok(files)
}

/// Removes, at every depth, the `commit` field of an object (it records where the file was written, not
/// data: the header, the wrapper's `engine` and `solver` blocks, the provenance block of the command
/// line's JSON), and the `# commit:` line of a text that carries a provenance header.
fn strip_commits(v: &mut Value) {
    match v {
        Value::Object(m) => {
            m.shift_remove("commit");
            for x in m.values_mut() {
                strip_commits(x);
            }
        }
        Value::Array(a) => a.iter_mut().for_each(strip_commits),
        Value::String(s) if s.contains("# commit:") => {
            *s = s
                .lines()
                .filter(|l| !l.starts_with("# commit:"))
                .collect::<Vec<_>>()
                .join("\n");
        }
        _ => {}
    }
}

/// A file is the same data as another when this form is equal.
pub fn comparable(name: &str, text: &str) -> Result<String> {
    if name.ends_with(".json") {
        let mut v: Value = serde_json::from_str(text)?;
        if let Value::Object(m) = &mut v {
            for k in PRECOMPUTED_VOLATILE {
                m.shift_remove(k);
            }
        }
        strip_commits(&mut v);
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

/// The snapshot times of the command line: [`s::SNAPSHOT_FRACTIONS`] of the end time of the input,
/// as the shortest decimal text that reads back to the same number.
fn snapshot_list(root: &Path) -> Result<String> {
    let input: toml::Value = toml::from_str(&util::read_text(&root.join(s::INPUT))?)
        .map_err(|e| Error(format!("{}: {e}", s::INPUT)))?;
    let end = input
        .get("time")
        .and_then(|t| t.get("end_time_s"))
        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
        .ok_or_else(|| Error(format!("{}: no time.end_time_s", s::INPUT)))?;
    Ok(s::SNAPSHOT_FRACTIONS
        .iter()
        .map(|f| float_repr(f * end))
        .collect::<Vec<_>>()
        .join(","))
}

/// The consistency guard of the two runs: the command line and the wrapper solve the same input
/// with the same solver, so the final release fraction they report is the same number.
pub fn cross_check(raw: &RawRun) -> Result<()> {
    let printed = stdout_summary(&raw.stdout)
        .get("final_release_fraction")
        .and_then(Value::as_str)
        .and_then(|t| t.parse::<f64>().ok());
    let reply = raw
        .native
        .get("diagnostics")
        .and_then(|d| d.get("final_release_fraction"))
        .and_then(Value::as_f64);
    match (printed, reply) {
        (Some(a), Some(b)) if a == b => Ok(()),
        (Some(a), Some(b)) => bail!(
            "the command line and the wrapper report different final release fractions: {a:e} against {b:e}"
        ),
        _ => bail!("a final release fraction is missing from the command line output or the reply"),
    }
}

/// Run both programs and read the files the command line wrote.
pub fn collect(root: &Path) -> Result<RawRun> {
    let commit: String = commit_full(root).chars().take(7).collect();
    let cmd = s::cli_command(&snapshot_list(root)?, &commit);
    let stdout = run_command(root, &cmd, &[])?;
    let out_dir = root.join(s::CLI_OUT_DIR);
    let timeseries_csv = util::read_text(&out_dir.join(s::CLI_TIMESERIES))?;
    let profiles_csv = util::read_text(&out_dir.join(s::CLI_PROFILES))?;
    let run_json: Value = serde_json::from_str(&util::read_text(&out_dir.join(s::CLI_RUN_JSON))?)?;
    let env = [
        ("STRATO_BUILD_COMMIT", commit),
        ("STRATO_BUILD_DIRTY", "0".to_string()),
    ];
    let native_text = run_command(root, &s::native_command(), &env)?;
    let native: Value = serde_json::from_str(&native_text)?;
    let battery = collect_battery(root, &env)?;
    let raw = RawRun {
        stdout,
        timeseries_csv,
        profiles_csv,
        run_json,
        native,
        battery,
        check_csv: read_check(root)?.0,
    };
    cross_check(&raw)?;
    Ok(raw)
}

/// The small cases of the parity battery with their requests and the native replies.
fn collect_battery(root: &Path, env: &[(&str, String)]) -> Result<Value> {
    let requests: Value =
        serde_json::from_str(&run_command(root, &s::battery_command("requests"), env)?)?;
    let replies: Value =
        serde_json::from_str(&run_command(root, &s::battery_command("native"), env)?)?;
    let (Some(req), Some(rep)) = (requests.as_object(), replies.as_object()) else {
        bail!("the battery output is not a JSON object");
    };
    let mut out = Map::new();
    for name in s::BATTERY_CASES {
        let (Some(r), Some(p)) = (req.get(name), rep.get(name)) else {
            bail!("battery case {name} is missing");
        };
        out.insert(
            name.to_string(),
            json!({"function": r["function"], "input": r["input"], "reply": p}),
        );
    }
    Ok(Value::Object(out))
}

/// The independent-check file: its text and the SHA-256 of its bytes.
pub fn read_check(root: &Path) -> Result<(String, String)> {
    let p = root.join(s::CHECK_FILE);
    Ok((
        util::read_text(&p)?.replace("\r\n", "\n"),
        util::sha256_hex(&util::read_bytes(&p)?),
    ))
}

fn commit_full(root: &Path) -> String {
    util::git(root, &["rev-parse", "HEAD"]).unwrap_or_else(|_| "unknown".to_string())
}

fn rustc_version(root: &Path) -> String {
    run_command(root, &["rustc".to_string(), "-V".to_string()], &[])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

pub fn run(root: &Path, check: bool) -> Result<i32> {
    let raw = collect(root)?;
    let input = root.join(s::INPUT);
    let ctx = Context {
        commit: commit_full(root),
        version: catalogue::strato_version(root)?,
        created: util::now_utc(),
        rustc: rustc_version(root),
        input_sha256: util::sha256_hex(&util::read_bytes(&input)?),
        check_sha256: read_check(root)?.1,
    };
    let files = build_files(&ctx, &raw)?;
    if check {
        let mut bad = 0;
        for name in s::RETIRED_FILES {
            let path = root.join(CSV_DIR).join(name);
            if path.exists() {
                println!(
                    "RETIRED {} (a file of an earlier solver)",
                    util::rel_posix(root, &path)
                );
                bad += 1;
            }
        }
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
