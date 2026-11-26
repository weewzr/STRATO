//! End-to-end tests of the results program on the real registers with the coarse `quick` grids:
//! every output file carries its provenance, no R2 value exists for the registered curves and the
//! first failing check of each is the one of `metric-r2.md` Section 3.5, nothing private is
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
        "r2.json",
        "bounds.json",
        "labels.json",
        "manifest.json",
        "verification/convergence.json",
        "verification/agreement.json",
        "verification/conservation.json",
        "csv/curves.csv",
        "csv/r2.csv",
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

    // The metric of the superseded decisions 64 and 65 appears nowhere, and no file is named for it.
    for (name, text) in &files {
        assert!(!name.to_lowercase().contains("nrmse"), "{name}");
        for word in text.split(|c: char| !c.is_alphanumeric()) {
            assert!(
                !matches!(word, "NRMSE" | "RMSE" | "MAE") && !word.to_lowercase().contains("rmse"),
                "{name}: the word {word}"
            );
        }
    }

    // No registered curve has an R2 value; every record says why.
    let r2 = &files["r2.json"];
    assert!(r2.contains("\"definition_id\": \"R2_LINEAR_PER_CURVE_V1\""));
    let valued = r2
        .lines()
        .filter(|l| l.trim_start().starts_with("\"r2\":") && !l.contains("null"))
        .count();
    assert_eq!(
        valued, 0,
        "an R2 value exists although every curve fails a check"
    );
    assert_eq!(r2.matches("\"status\": \"unavailable\"").count(), 30);
    assert!(!r2.contains("\"status\": \"available\""));
    for line in files["csv/r2.csv"]
        .lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
    {
        let cols: Vec<&str> = line.split(',').collect();
        assert!(cols[8].is_empty(), "an R2 value in {line}");
        assert!(!cols[6].is_empty(), "no reason code in {line}");
    }
    assert_eq!(
        files
            .keys()
            .filter(|k| k.starts_with("comparison/"))
            .count(),
        15
    );
    let labels = &files["labels.json"];
    assert!(labels.contains("\"r2_captions\"") && labels.contains("\"r2_display_rule\""));

    // The C15 documented-application label (science design 5.1): the cooldown, the startup ramp
    // and the 900 degC part carry it, in the file and (Scenario 4) in the panel title; normal
    // operation at 1028 degC and at 1197 degC does not.
    let label = "C15 evaluated outside its documented application (955 to 1197 \u{b0}C)";
    let curve = |part: &str| -> &String {
        let k = files
            .keys()
            .find(|k| k.starts_with("curves/") && k.contains(part))
            .unwrap_or_else(|| panic!("no curve file with {part}"));
        &files[k]
    };
    let labelled = |part: &str| -> usize { curve(part).matches(label).count() };
    for part in [
        "s5.shutdown-cooldown.sr90.1028c",
        "s5.startup.sr90.1028c",
        "s5.control-rod-insertion.sr90.1028c",
    ] {
        // The panel title, the label list and the field of the per-segment status.
        assert_eq!(labelled(part), 3, "{part}: the C15 label");
        assert!(!curve(part).contains("\"steps_outside\": 0,"), "{part}");
    }
    for part in [
        "s5.steady-state.sr90.1028c",
        "s5.steady-state.exploration.sr90.1197c",
    ] {
        // Only the field that holds the text for the case that steps lie outside.
        assert_eq!(
            labelled(part),
            1,
            "{part}: a C15 label without a step outside"
        );
        assert!(curve(part).contains("\"steps_outside\": 0,"), "{part}");
    }
    let s4 = curve("s5.control-rod-insertion.sr90.1028c");
    let title = s4
        .lines()
        .find(|l| l.contains("\"panel_title\""))
        .expect("panel title");
    assert!(title.contains(label), "{title}");
    // The kernel-only comparison of the Scenario 5 heating cells.
    let heating = curve("s5.refuelling.heating1600.sr90.1028c");
    assert!(heating.contains("\"kernel_only_comparison\""));
    assert!(heating.contains("F_post_kernel_only"));
    assert!(curve("s5.shutdown-cooldown.sr90.1028c").contains("\"kernel_only_comparison\": null"));
    // The window-start sensitivity of the matrix-corrected rows.
    let bounds = &files["bounds.json"];
    assert!(bounds.contains("\"window_start_sensitivity\""));
    assert!(bounds.contains("\"window_start_h\": 2e1"));
    assert!(bounds.contains("\"window_start_h\": 2.8e1"));
    assert!(bounds.contains("\"label\": \"prescribed input (assumed)\""));
    // One grid for the family: the Scenario 2 state at shutdown is the Scenario 1 end state.
    // (The per-nuclide interval counts are the same in every base-family file.)
    let n_of = |text: &str| -> String {
        text.lines()
            .skip_while(|l| !l.contains("\"intervals_per_layer\""))
            .take(1)
            .collect::<Vec<_>>()
            .join("")
            .replace(' ', "")
    };
    assert_eq!(
        n_of(curve("s5.steady-state.sr90.1028c")),
        n_of(curve("s5.shutdown-cooldown.sr90.1028c"))
    );
    assert_eq!(
        n_of(curve("s5.steady-state.sr90.1028c")),
        n_of(curve("s5.refuelling.heating1600.sr90.1028c"))
    );

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

/// The table of `metric-r2.md` Section 3.5: the first failing check and its reason code for each of
/// the fifteen registered curves, against the verified assessment of the data and against the
/// first token of the reason that `curve_assessment.csv` records for the curve.
#[test]
fn the_first_failing_check_of_the_fifteen_curves_is_the_table_of_section_3_5() {
    const TABLE: [(&str, &str, &str); 15] = [
        ("HUNN16-442-CS134", "C2", "coating_failure"),
        ("HUNN16-442-CS137", "C2", "coating_failure"),
        ("HUNN14-442-AG110M", "C2", "coating_failure"),
        ("HUNN14-442-SR90", "C2", "coating_failure"),
        ("DEM15-433-CS134", "C1", "upper_bound"),
        ("DEM15-433-SR90", "C1", "upper_bound"),
        ("IAEA97-K31-AG110M", "C1", "upper_bound"),
        ("IAEA97-K31-CS137", "C1", "upper_bound"),
        ("IAEA97-K31-SR90", "C1", "upper_bound"),
        ("STEMPIEN23-433-AG110M", "C1", "upper_bound"),
        ("STEMPIEN23-643-AG110M", "C1", "upper_bound"),
        ("STEMPIEN23-643-SR90", "C1", "upper_bound"),
        ("HUNN14-422-AG110M", "C1", "upper_bound"),
        ("IAEA97-JAERI-A-CS137-1600", "C4", "incompatible_definition"),
        ("IAEA97-JAERI-A-CS137-1800", "C4", "incompatible_definition"),
    ];
    let files = generate(&Options {
        repo: repo(),
        commit: "test".into(),
        quick: true,
    })
    .expect("the pipeline runs");
    let register =
        fs::read_to_string(repo().join("data/experimental/curve_assessment.csv")).unwrap();
    let csv = &files["csv/r2.csv"];
    for (id, check, code) in TABLE {
        // The assessment of the data states the same first check.
        let line = register
            .lines()
            .find(|l| l.starts_with(&format!("{id},")))
            .unwrap_or_else(|| panic!("{id} is not in the curve assessment"));
        assert!(
            line.contains(&format!(",PLOT ONLY,{check} ")),
            "{id}: the assessment does not say {check}"
        );
        for method in ['A', 'B'] {
            let want = format!("r2.{id}.{method},");
            let row = csv
                .lines()
                .find(|l| l.starts_with(&want))
                .unwrap_or_else(|| panic!("no record {want}"));
            let cols: Vec<&str> = row.split(',').collect();
            assert_eq!(cols[5], "unavailable", "{row}");
            assert_eq!(cols[6], code, "{row}");
            assert_eq!(cols[7].split(';').next(), Some(check), "{row}");
            assert!(cols[8].is_empty() && cols[9].is_empty(), "{row}");
        }
        // The comparison file holds the points exactly as registered, the model and the caption.
        let file = &files[&format!("comparison/{id}.json")];
        assert!(
            file.contains("\"points\"") && file.contains("\"caption_text\""),
            "{id}"
        );
        assert!(file.contains("\"label_plain\": null"), "{id}");
    }
}
