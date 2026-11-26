//! The files this tool writes are, byte for byte, the files committed in the repository.
//!
//! The committed files were written by the Python generator that this tool replaces. Every
//! subcommand is run in memory and its text is compared with the committed file as bytes. The
//! only fields left out are those that record when, where and with which tool a file was
//! written (`created_utc`, `commit`, `generator_sha256`, `rustc`): they are replaced by a
//! placeholder on both sides, line by line, so every other byte of the file is compared.

mod common;

use common::repo_root;
use std::path::Path;
use strato_webtools::precomputed::{self, Context, RawRun};
use strato_webtools::solver as s;
use strato_webtools::{case, catalogue, colormaps, geometry, util};

const VOLATILE_KEYS: [&str; 4] = ["created_utc", "commit", "generator_sha256", "rustc"];

/// The text with the value of each volatile field replaced by `X`: the field at the top level of
/// a JSON file (two-space indent), or a `# key:` line of a CSV header.
fn normalise(text: &str) -> String {
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        let mut done = false;
        for k in VOLATILE_KEYS {
            let json = format!("  \"{k}\": ");
            let csv = format!("# {k}: ");
            if line.starts_with(&json) {
                out.push_str(&json);
                out.push_str("X\n");
                done = true;
            } else if line.starts_with(&csv) {
                out.push_str(&csv);
                out.push_str("X\n");
                done = true;
            }
        }
        if !done {
            out.push_str(line);
        }
    }
    out
}

fn assert_same_bytes(root: &Path, rel: &str, fresh: &str) {
    let committed = std::fs::read(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
    let committed = String::from_utf8(committed).expect("UTF-8");
    let (a, b) = (normalise(&committed), normalise(fresh));
    if a != b {
        let first = a
            .lines()
            .zip(b.lines())
            .enumerate()
            .find(|(_, (x, y))| x != y)
            .map(|(i, (x, y))| format!("line {}:\n  committed: {x}\n  fresh:     {y}", i + 1))
            .unwrap_or_else(|| format!("lengths differ: {} against {}", a.len(), b.len()));
        panic!("{rel} differs from a fresh run\n{first}");
    }
}

#[test]
fn case_files_are_reproduced() {
    let root = repo_root();
    let input = root.join(s::INPUT);
    let (doc, csv) = case::build(&root, &input).unwrap();
    assert_same_bytes(
        &root,
        "web/src/generated/synthetic_five_layer.case.json",
        &strato_webtools::pyfmt::json_file(&doc),
    );
    assert_same_bytes(&root, "web/public/data/synthetic_five_layer.case.csv", &csv);
}

#[test]
fn catalogue_is_reproduced() {
    let root = repo_root();
    let doc = catalogue::build(&root).unwrap();
    assert_same_bytes(
        &root,
        catalogue::CATALOGUE_JSON,
        &strato_webtools::pyfmt::json_file(&doc),
    );
}

#[test]
fn colour_tables_are_reproduced() {
    let root = repo_root();
    let doc = colormaps::build(&root).unwrap();
    assert_same_bytes(
        &root,
        "web/src/generated/colormaps.json",
        &strato_webtools::pyfmt::json_file(&doc),
    );
}

#[test]
fn reference_geometry_is_reproduced() {
    let root = repo_root();
    let (doc, csv) = geometry::build(&root).unwrap();
    assert_same_bytes(
        &root,
        "web/src/generated/agr1_reference.json",
        &strato_webtools::pyfmt::json_file(&doc),
    );
    assert_same_bytes(&root, "web/public/data/agr1_reference_geometry.csv", &csv);
}

/// Every line of `text` from the first one that starts with `marker`: the body of a file that the
/// converter wrote, without the header lines it put in front of the command line's own.
fn body_from(text: &str, marker: &str) -> String {
    let at = text.find(marker).unwrap_or_else(|| panic!("no line starting with {marker}"));
    text[at..].to_string()
}

fn committed_json(root: &Path, name: &str) -> serde_json::Value {
    serde_json::from_str(&util::read_text(&root.join("web/public/data").join(name)).unwrap())
        .unwrap()
}

/// The precomputed files, converted again from what they were converted from: the wrapper reply
/// (committed in `wrapper_native_synthetic.json`), the command line's three files (recovered from
/// the committed copies) and the independent-check file of the regression suite (read from
/// tests/golden/btcs, the original). This tests the conversion without building and running the
/// solver; `precomputed --check` in the web workflow runs the solver as well.
#[test]
fn precomputed_files_are_reproduced_from_their_sources() {
    let root = repo_root();
    let read = |n: &str| util::read_text(&root.join("web/public/data").join(n)).unwrap();
    let summary = committed_json(&root, "btcs_run.json")["data"]["summary"].clone();
    let stdout: String = summary
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| format!("{k}={}\n", v.as_str().unwrap()))
        .collect();
    let (check_csv, check_sha256) = precomputed::read_check(&root).unwrap();
    let raw = RawRun {
        stdout,
        timeseries_csv: body_from(&read("btcs_run.timeseries.csv"), "# STRATO BTCS"),
        profiles_csv: body_from(&read("btcs_run.profiles.csv"), "# STRATO BTCS"),
        run_json: committed_json(&root, "btcs_run.json")["data"]["run"].clone(),
        native: committed_json(&root, "wrapper_native_synthetic.json")["data"].clone(),
        check_csv,
    };
    precomputed::cross_check(&raw).unwrap();
    let ctx = Context {
        commit: "X".into(),
        version: catalogue::strato_version(&root).unwrap(),
        created: "X".into(),
        rustc: "X".into(),
        input_sha256: util::sha256_hex(&util::read_bytes(&root.join(s::INPUT)).unwrap()),
        check_sha256,
    };
    let files = precomputed::build_files(&ctx, &raw).unwrap();
    let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, s::PRECOMPUTED_FILES);
    for (name, text) in &files {
        assert_same_bytes(&root, &format!("web/public/data/{name}"), text);
    }
}

/// The independent-check copy is the regression suite's file: same rows, in the same order, under
/// the machine keys, with the observed orders the suite asserts (about 1 in time, 2 in space).
#[test]
fn independent_check_copy_holds_the_suite_rows_and_orders() {
    let root = repo_root();
    let (csv, _) = precomputed::read_check(&root).unwrap();
    let data = precomputed::check_data(&csv).unwrap();
    let committed = committed_json(&root, "btcs_independent_check.json");
    assert_eq!(committed["data"]["t_s"], data["t_s"]);
    assert_eq!(committed["data"]["f_btcs"], data["f_btcs"]);
    let rows = data["t_s"].as_array().unwrap().len();
    assert!(rows >= 2, "{rows} checkpoint times");
    for v in data["time_order"].as_array().unwrap() {
        assert!((v.as_f64().unwrap() - 1.0).abs() < 0.5, "time order {v}");
    }
    for v in data["space_order"].as_array().unwrap() {
        assert!((v.as_f64().unwrap() - 2.0).abs() < 0.5, "space order {v}");
    }
    assert!(data["passed"].as_array().unwrap().iter().all(|p| p == true));
}

/// The comparison with the analytical series is the wrapper reply's two columns and their
/// difference: no value is invented, a time the series does not cover stays empty.
#[test]
fn comparison_columns_come_from_the_reply() {
    let root = repo_root();
    let native = committed_json(&root, "wrapper_native_synthetic.json")["data"].clone();
    let (data, body) = precomputed::vs_analytical(&native).unwrap();
    assert_eq!(data["t_s"], native["times_s"]);
    assert_eq!(data["release_fraction_btcs"], native["release_fraction"]);
    let n = native["times_s"].as_array().unwrap().len();
    assert_eq!(body.lines().count(), n + 1);
    let valid = native["analytical"]["times_s"].as_array().unwrap().len();
    let present = data["abs_difference"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| !v.is_null())
        .count();
    assert_eq!(present, valid);
    for ((b, a), d) in data["release_fraction_btcs"]
        .as_array()
        .unwrap()
        .iter()
        .zip(data["release_fraction_analytical"].as_array().unwrap())
        .zip(data["abs_difference"].as_array().unwrap())
    {
        match a.as_f64() {
            Some(a) => assert_eq!(d.as_f64().unwrap(), (b.as_f64().unwrap() - a).abs()),
            None => assert!(d.is_null()),
        }
    }
}

/// A reply without an analytical series, or with a series time the run does not have, is refused
/// rather than matched loosely.
#[test]
fn comparison_refuses_what_it_cannot_match() {
    let root = repo_root();
    let mut native = committed_json(&root, "wrapper_native_synthetic.json")["data"].clone();
    native["analytical"]["times_s"][0] = serde_json::json!(0.123456);
    assert!(precomputed::vs_analytical(&native).is_err());
    native["analytical"] = serde_json::Value::Null;
    assert!(precomputed::vs_analytical(&native).is_err());
}

/// The files of the earlier solver are gone from the data folder.
#[test]
fn no_file_of_the_earlier_solver_remains() {
    let root = repo_root();
    for n in s::RETIRED_FILES {
        assert!(
            !root.join("web/public/data").join(n).exists(),
            "{n} still exists"
        );
    }
}