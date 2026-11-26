//! End-to-end tests of the results program on the real registers with the coarse `quick` grids:
//! every output file carries its provenance, no NRMSE value exists while N = 0, nothing private is
//! written, the manifest lists every file, and the check mode fails when a committed output is
//! stale, missing or no longer produced.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use strato_results::output::{check_report, write_all, Files};
use strato_results::pipeline::{generate, Options};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("strato-results-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    d
}

#[test]
fn the_pipeline_writes_a_checked_and_clean_file_set() {
    let files: Files = generate(&Options {
        repo: repo(),
        commit: "test".into(),
        quick: true,
    })
    .expect("the pipeline runs on the registers of the repository");

    // The layout of the website specification (part 4), with the differences noted in the README.
    for must in [
        "support.json",
        "parameters.json",
        "nrmse.json",
        "bounds.json",
        "labels.json",
        "manifest.json",
        "verification/convergence.json",
        "verification/agreement.json",
        "verification/conservation.json",
        "csv/curves.csv",
        "csv/nrmse.csv",
    ] {
        assert!(files.contains_key(must), "missing output {must}");
    }
    assert!(files.keys().any(|k| k.starts_with("curves/")));
    assert!(files.keys().any(|k| k.starts_with("experiment/")));

    for (name, text) in &files {
        // Provenance in every file.
        if name.ends_with(".json") {
            assert!(
                text.contains("\"provenance\""),
                "{name}: no provenance header"
            );
            assert!(
                text.contains("\"solver_source_sha256\""),
                "{name}: no solver hash"
            );
        } else {
            assert!(
                text.starts_with("# STRATO shared results"),
                "{name}: no CSV header"
            );
            assert!(
                text.contains("# solver_source_sha256:"),
                "{name}: no solver hash"
            );
        }
        // Nothing private: no absolute local path, no e-mail address.
        assert!(
            !text.contains(":\\\\") && !text.contains("C:\\"),
            "{name}: a Windows path"
        );
        assert!(
            !text.contains("/Users/") && !text.contains("/home/"),
            "{name}: a path"
        );
        assert!(!text.contains('@'), "{name}: an at sign (e-mail address?)");
        // No byte-order mark.
        assert!(!text.starts_with('\u{feff}'), "{name}: byte-order mark");
    }

    // N = 0 everywhere today: not one record carries a value, and the record says why.
    let nrmse = &files["nrmse.json"];
    assert!(nrmse.contains("\"N\": 0"));
    assert!(!nrmse.contains("\"N\": 1") && !nrmse.contains("\"N\": 2"));
    let valued = nrmse
        .lines()
        .filter(|l| l.trim_start().starts_with("\"nrmse_percent\":") && !l.contains("null"))
        .count();
    assert_eq!(
        valued, 0,
        "an NRMSE value exists although the registers hold no point set"
    );
    for line in files["csv/nrmse.csv"]
        .lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
    {
        let cols: Vec<&str> = line.split(',').collect();
        assert_eq!(cols[4], "0", "N is not 0 in {line}");
        assert!(cols[7].is_empty(), "an NRMSE value in {line}");
    }

    // The manifest lists every other file.
    let manifest = &files["manifest.json"];
    for name in files.keys().filter(|k| *k != "manifest.json") {
        assert!(
            manifest.contains(&format!("\"{name}\"")),
            "{name} not in the manifest"
        );
    }

    // The check mode.
    let out = temp("check");
    write_all(&out, &files).unwrap();
    let (code, text) = check_report(&out, &files, 0.0);
    assert_eq!(code, 0, "{text}");
    // Another commit hash alone is not stale.
    let mut moved = files.clone();
    let s = moved["support.json"].replace("\"commit\": \"test\"", "\"commit\": \"other\"");
    moved.insert("support.json".into(), s);
    assert_eq!(check_report(&out, &moved, 0.0).0, 0);
    // A changed number is stale and is named.
    let victim = files
        .keys()
        .find(|k| k.starts_with("curves/"))
        .unwrap()
        .clone();
    let mut text = fs::read_to_string(out.join(&victim)).unwrap();
    let at = text.find("release_fraction").unwrap();
    text.insert(at, 'x');
    fs::write(out.join(&victim), text).unwrap();
    let (code, text) = check_report(&out, &files, 0.0);
    assert_eq!(code, 1);
    assert!(text.contains(&format!("STALE {victim}")), "{text}");
    // A missing file is stale.
    write_all(&out, &files).unwrap();
    fs::remove_file(out.join("bounds.json")).unwrap();
    let (code, text) = check_report(&out, &files, 0.0);
    assert_eq!(code, 1);
    assert!(text.contains("STALE bounds.json: missing"), "{text}");
    // A committed file that is no longer produced is stale.
    write_all(&out, &files).unwrap();
    fs::write(out.join("old.json"), "{}\n").unwrap();
    let (code, text) = check_report(&out, &files, 0.0);
    assert_eq!(code, 1);
    assert!(text.contains("STALE old.json: not produced"), "{text}");
    let _ = fs::remove_dir_all(&out);
}

#[test]
fn the_command_line_refuses_what_it_does_not_know() {
    let exe = env!("CARGO_BIN_EXE_strato-results");
    let code = |args: &[&str]| Command::new(exe).args(args).output().unwrap().status.code();
    assert_eq!(code(&[]), Some(2));
    assert_eq!(code(&["build"]), Some(2));
    assert_eq!(code(&["run", "--unknown"]), Some(2));
    assert_eq!(code(&["run", "--out"]), Some(2));
    // A repository without registers fails (status 1) in the check mode as well.
    let empty = temp("empty");
    fs::create_dir_all(&empty).unwrap();
    let r = empty.to_string_lossy().to_string();
    assert_eq!(code(&["run", "--check", "--repo", &r]), Some(1));
    let _ = fs::remove_dir_all(&empty);
}
