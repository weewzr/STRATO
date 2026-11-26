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
use strato_webtools::precomputed::{self, Context, GoldenFile, RawRun};
use strato_webtools::solver as s;
use strato_webtools::{case, catalogue, colormaps, figures, geometry, util};

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

#[test]
fn guide_lines_are_reproduced() {
    let root = repo_root();
    let (doc, csv) = figures::build(&root).unwrap();
    assert_same_bytes(
        &root,
        "web/public/data/convergence_guides.json",
        &strato_webtools::pyfmt::json_file(&doc),
    );
    assert_same_bytes(&root, "web/public/data/convergence_guides.csv", &csv);
}

/// The precomputed files, converted from the golden CSV files (which the native run reproduces
/// byte for byte, checked by `precomputed --check`) and from the committed wrapper response. This
/// tests the conversion without building and running the solver.
#[test]
fn precomputed_files_are_reproduced_from_the_golden_run() {
    let root = repo_root();
    let golden_dir = root.join(s::GOLDEN_DIR);
    let read = |n: &str| util::read_text(&golden_dir.join(n)).unwrap();
    let csv: [String; 4] = [
        read(s::CLI_CSV[0]),
        read(s::CLI_CSV[1]),
        read(s::CLI_CSV[2]),
        read(s::CLI_CSV[3]),
    ];
    let wrapper: serde_json::Value = serde_json::from_str(
        &util::read_text(&root.join("web/public/data/wrapper_native_synthetic.json")).unwrap(),
    )
    .unwrap();
    let raw = RawRun {
        stdout: read(s::GOLDEN_STDOUT),
        csv,
        native: wrapper["data"].clone(),
    };
    let golden: [GoldenFile; 4] = std::array::from_fn(|k| {
        let p = golden_dir.join(s::CLI_CSV[k]);
        GoldenFile {
            text: util::read_text(&p).unwrap(),
            sha256: util::sha256_hex(&util::read_bytes(&p).unwrap()),
        }
    });
    let ctx = Context {
        commit: "X".into(),
        version: catalogue::strato_version(&root).unwrap(),
        created: "X".into(),
        rustc: "X".into(),
        input_sha256: util::sha256_hex(&util::read_bytes(&root.join(s::INPUT)).unwrap()),
    };
    let files = precomputed::build_files(&ctx, &raw, &golden).unwrap();
    let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, s::PRECOMPUTED_FILES);
    for (name, text) in &files {
        assert_same_bytes(&root, &format!("web/public/data/{name}"), text);
    }
}

#[test]
fn golden_stdout_summary_is_the_committed_summary() {
    let root = repo_root();
    let text = util::read_text(&root.join(s::GOLDEN_DIR).join(s::GOLDEN_STDOUT)).unwrap();
    let summary = precomputed::stdout_summary(&text);
    let committed: serde_json::Value = serde_json::from_str(
        &util::read_text(&root.join("web/public/data/synthetic_run.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        serde_json::Value::Object(summary),
        committed["data"]["summary"]
    );
}
