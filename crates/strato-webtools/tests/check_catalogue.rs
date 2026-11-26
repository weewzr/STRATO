//! Tests of the catalogue check against its fixture folders (port of
//! scripts/web/tests/test_check_catalogue.py, same fixtures).
//!
//! prompt/TESTING_AND_VALIDATION.md Section 3.1: the check passes on the repository and on the
//! good fixture, and each bad fixture fails with the matching FAIL line. A bad fixture is a
//! folder with a case.json that names the edits applied to a copy of the good fixture, the rules
//! that must fail and the messages they must print.

mod common;

use common::{copy_dir, fixtures, repo_root, TempDir};
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;
use strato_webtools::catalogue::{self, CATALOGUE_JSON};
use strato_webtools::check::{self, Report, OVERVIEW_PAGES};
use strato_webtools::pyfmt::json_file;

fn apply(root: &Path, patches: &[Value]) {
    for p in patches {
        let path = root.join(p["file"].as_str().expect("file"));
        let mut text = std::fs::read_to_string(&path).expect("read patched file");
        if let Some(a) = p.get("append").and_then(Value::as_str) {
            text.push_str(a);
        } else {
            let find = p["find"].as_str().expect("find");
            assert!(
                text.contains(find),
                "fixture edit does not apply: {find:?} not in {}",
                p["file"]
            );
            text = text.replacen(find, p["replace"].as_str().expect("replace"), 1);
        }
        std::fs::write(&path, text).expect("write patched file");
    }
}

fn copy_good() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new("check");
    let root = tmp.0.join("root");
    copy_dir(&fixtures().join("good"), &root);
    (tmp, root)
}

fn generate(root: &Path) {
    let target = root.join(CATALOGUE_JSON);
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(&target, json_file(&catalogue::build(root).unwrap())).unwrap();
}

fn run(root: &Path) -> Report {
    check::check(root).expect("the check runs")
}

fn text(rep: &Report) -> String {
    rep.lines.join("\n")
}

#[test]
fn repository_passes() {
    let rep = run(&repo_root());
    assert!(!rep.failed, "{}", text(&rep));
}

#[test]
fn good_fixture_passes_every_rule_without_skipping() {
    let (_tmp, root) = copy_good();
    generate(&root);
    let rep = run(&root);
    assert!(!rep.failed, "{}", text(&rep));
    assert!(
        !rep.lines.iter().any(|l| l.starts_with("SKIP")),
        "{}",
        text(&rep)
    );
    let passed: HashSet<u32> = rep
        .lines
        .iter()
        .filter(|l| l.starts_with("PASS"))
        .map(|l| l.split_whitespace().nth(1).unwrap().parse().unwrap())
        .collect();
    assert_eq!(passed, (1..10).collect::<HashSet<u32>>());
}

#[test]
fn good_fixture_extracts_the_research_concept_equation() {
    let (_tmp, root) = copy_good();
    let doc = catalogue::build(&root).unwrap();
    let entry = &doc["data"]["entries"]["docs/research/fixture_pending.md"];
    assert_eq!(
        entry["concept_equations"],
        serde_json::json!([r"\frac{\partial c}{\partial t} = -k\,c"])
    );
}

#[test]
fn missing_generated_file_fails_rule_9() {
    let (_tmp, root) = copy_good();
    let rep = run(&root);
    assert_eq!(rep.failed_rules(), HashSet::from([9]));
}

#[test]
fn crlf_line_ends_fail_rule_1() {
    // Made here, not stored as a fixture: .gitattributes normalises stored line ends to LF.
    let (_tmp, root) = copy_good();
    let path = root.join("data/research/references.csv");
    let bytes = std::fs::read(&path).unwrap();
    let crlf = String::from_utf8(bytes).unwrap().replace('\n', "\r\n");
    std::fs::write(&path, crlf).unwrap();
    generate(&root);
    let rep = run(&root);
    assert!(rep.failed_rules().contains(&1), "{}", text(&rep));
    assert!(rep.lines.iter().any(|l| l.contains("CR line ends")));
}

#[test]
fn overview_pages_need_no_row() {
    let (_tmp, root) = copy_good();
    for rel in OVERVIEW_PAGES {
        std::fs::write(
            root.join(rel),
            "# Overview\n\nFixture overview page [ref:fixture2025].\n",
        )
        .unwrap();
    }
    generate(&root);
    let rep = run(&root);
    assert!(!rep.failed, "{}", text(&rep));
}

#[test]
fn other_unlisted_page_fails_rule_4() {
    let (_tmp, root) = copy_good();
    std::fs::write(root.join("docs/research/regime-d.md"), "# Stray\n").unwrap();
    generate(&root);
    let rep = run(&root);
    assert_eq!(rep.failed_rules(), HashSet::from([4]), "{}", text(&rep));
    assert!(rep
        .lines
        .iter()
        .any(|l| l.contains("docs/research/regime-d.md is referenced by no catalogue row")));
}

#[test]
fn overview_page_reference_ids_are_checked() {
    let (_tmp, root) = copy_good();
    std::fs::write(
        root.join("docs/research/roadmap.md"),
        "# Roadmap\n\n[ref:nosuchref]\n",
    )
    .unwrap();
    generate(&root);
    let rep = run(&root);
    assert_eq!(rep.failed_rules(), HashSet::from([3]), "{}", text(&rep));
}

#[test]
fn each_bad_fixture_fails_with_its_line() {
    let mut cases: Vec<_> = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir() && p.file_name().unwrap().to_string_lossy().starts_with("bad_"))
        .collect();
    cases.sort();
    assert!(cases.len() >= 14, "{} bad fixtures", cases.len());
    for case in cases {
        let name = case.file_name().unwrap().to_string_lossy().into_owned();
        let spec: Value =
            serde_json::from_str(&std::fs::read_to_string(case.join("case.json")).unwrap())
                .unwrap();
        let (_tmp, root) = copy_good();
        let patches = spec["patches"].as_array().unwrap();
        if spec
            .get("regenerate_after_patch")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        {
            apply(&root, patches);
            generate(&root);
        } else {
            generate(&root);
            apply(&root, patches);
        }
        let rep = run(&root);
        let failed = rep.failed_rules();
        let expected: HashSet<u32> = spec["fail_rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u32)
            .collect();
        let out = text(&rep);
        if spec
            .get("allow_other_failures")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            assert!(expected.is_subset(&failed), "{name}\n{out}");
        } else {
            assert_eq!(failed, expected, "{name}\n{out}");
        }
        for msg in spec["messages"].as_array().unwrap() {
            let msg = msg.as_str().unwrap();
            assert!(
                rep.lines
                    .iter()
                    .any(|l| l.starts_with("FAIL ") && l.contains(msg)),
                "{name}: {msg:?} not printed:\n{out}"
            );
        }
    }
}

/// The exit code and the printed lines of the command line itself, on the good fixture.
#[test]
fn command_line_exit_codes_and_lines() {
    let (_tmp, root) = copy_good();
    generate(&root);
    let root_s = root.to_string_lossy().into_owned();
    let ok = std::process::Command::new(env!("CARGO_BIN_EXE_strato-webtools"))
        .args(["check-catalogue", "--root", &root_s])
        .output()
        .unwrap();
    assert_eq!(ok.status.code(), Some(0));
    let out = String::from_utf8(ok.stdout).unwrap();
    assert!(out.starts_with("PASS 1 headers\n"), "{out}");
    assert!(
        out.ends_with("PASS 9 generated catalogue\nRESULT PASS\n"),
        "{out}"
    );

    std::fs::remove_file(root.join(CATALOGUE_JSON)).unwrap();
    let bad = std::process::Command::new(env!("CARGO_BIN_EXE_strato-webtools"))
        .args(["check-catalogue", "--root", &root_s])
        .output()
        .unwrap();
    assert_eq!(bad.status.code(), Some(1));
    let out = String::from_utf8(bad.stdout).unwrap();
    assert!(out.contains("FAIL 9 generated catalogue: web/src/generated/research_catalogue.json is missing; run strato-webtools catalogue\n"), "{out}");
    assert!(out.ends_with("RESULT FAIL\n"), "{out}");
}
