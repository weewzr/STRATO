//! BTCS regression suite (directive Section 11, test I).
//!
//! The files in `tests/golden/btcs/` are regression references: the output of the BTCS solver
//! itself at the commit written in their headers. They are NOT truth and are never edited by
//! hand. Truth is established by `independent_check.csv` beside each case (agreement with the
//! Sturm-Liouville series or the closed form of the homogeneous sphere, by an error estimate from the run's own
//! refinements and a stated convention (not a derived bound); see `src/validation/regression.rs`), and by
//! `tests/btcs_verification.rs`. The finite-volume golden files of `tests/golden/v1.0/` are not
//! used here.
//!
//! Comparison rule, fixed when the files were first written: `|a - b| <= 1e-12 max(scale, |a|,
//! |b|)` per value (the rule of `tests/phase1_regression.rs`), with `scale` 1 for times and
//! release fractions, the initial inventory for mol, the largest concentration of the file for
//! concentrations, and the largest radius for radii. A mismatch means the code changed.

use std::{fs, path::PathBuf};
use strato::{
    config::Config,
    postprocessing::export::key_of_header,
    validation::regression::{generate, independent_check, Case, CASES, EXPLAINED_FRACTION, RTOL},
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn golden(case: &Case, file: &str) -> PathBuf {
    root().join("tests/golden/btcs").join(case.id).join(file)
}

fn config(case: &Case) -> Config {
    Config::load(root().join(case.input)).unwrap()
}

/// Numeric table of a CSV text with `#` comment lines: (header, rows); an empty cell is NaN.
fn table(text: &str) -> (Vec<String>, Vec<Vec<f64>>) {
    let mut r = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .from_reader(text.as_bytes());
    let header = r.headers().unwrap().iter().map(str::to_string).collect();
    let rows = r
        .records()
        .map(|rec| {
            rec.unwrap()
                .iter()
                .map(|s| {
                    if s.is_empty() {
                        f64::NAN
                    } else if s == "true" || s == "false" {
                        if s == "true" {
                            1.0
                        } else {
                            0.0
                        }
                    } else {
                        // Text columns (region, node_class) read as infinity and are skipped
                        // by `compare`.
                        s.parse::<f64>().unwrap_or(f64::INFINITY)
                    }
                })
                .collect()
        })
        .collect();
    (header, rows)
}

/// Cells of the data rows of a CSV text as strings (comment lines and the header skipped).
fn text_cells(text: &str) -> Vec<Vec<String>> {
    let mut r = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .from_reader(text.as_bytes());
    r.records()
        .map(|rec| rec.unwrap().iter().map(str::to_string).collect())
        .collect()
}

fn close(a: f64, b: f64, scale: f64) -> bool {
    if a.is_nan() || b.is_nan() {
        return a.is_nan() && b.is_nan();
    }
    (a - b).abs() <= RTOL * scale.max(a.abs()).max(b.abs())
}

/// Position of the column with the machine key `key` (headers are the descriptive names).
fn col_of(header: &[String], key: &str) -> usize {
    header
        .iter()
        .position(|h| key_of_header(h) == Some(key))
        .unwrap_or_else(|| panic!("no column {key}"))
}

/// Scale of column `name` in a table, from the frozen values.
fn scale_of(name: &str, header: &[String], rows: &[Vec<f64>]) -> f64 {
    let col = |n: &str| col_of(header, n);
    let max_abs = |n: &str| {
        rows.iter()
            .map(|r| r[col(n)])
            .filter(|v| v.is_finite())
            .fold(0.0, |m: f64, v| m.max(v.abs()))
    };
    match name {
        "t_s" | "release_fraction" | "node_index" | "f_btcs" | "f_reference" => 1.0,
        "inventory_mol"
        | "cumulative_release_mol"
        | "balance_residual_mol"
        | "balance_residual_bound_mol" => rows[0][col("inventory_mol")],
        "balance_residual_relative" => 1.0,
        "release_rate_mol_s" => max_abs("release_rate_mol_s"),
        "min_concentration_mol_m3" | "max_concentration_mol_m3" => {
            max_abs("max_concentration_mol_m3")
        }
        "r_m" => max_abs("r_m"),
        "concentration_mol_m3" => max_abs("concentration_mol_m3"),
        other => panic!("no scale rule for column {other}"),
    }
}

/// Compares a fresh table with the frozen one; returns the first mismatch as text.
fn compare(label: &str, frozen: &str, fresh: &str) -> Result<(), String> {
    let (hf, rf) = table(frozen);
    let (hn, rn) = table(fresh);
    let frozen_text = text_cells(frozen);
    let fresh_text = text_cells(fresh);
    if hf != hn {
        return Err(format!("{label}: columns differ: {hf:?} vs {hn:?}"));
    }
    if rf.len() != rn.len() {
        return Err(format!(
            "{label}: {} frozen rows, {} fresh rows",
            rf.len(),
            rn.len()
        ));
    }
    for (j, header) in hf.iter().enumerate() {
        let name = key_of_header(header).unwrap_or_else(|| panic!("unknown header {header}"));
        if matches!(name, "region" | "node_class") {
            // Text columns are compared exactly.
            for (i, (a, b)) in frozen_text.iter().zip(&fresh_text).enumerate() {
                if a[j] != b[j] {
                    return Err(format!(
                        "{label}: row {i}, column {name}: frozen {} vs fresh {}",
                        a[j], b[j]
                    ));
                }
            }
            continue;
        }
        let scale = scale_of(name, &hf, &rf);
        for (i, (a, b)) in rf.iter().zip(&rn).enumerate() {
            if !close(a[j], b[j], scale) {
                return Err(format!(
                    "{label}: row {i}, column {name}: frozen {} vs fresh {}",
                    a[j], b[j]
                ));
            }
        }
    }
    Ok(())
}

#[test]
fn series_and_profile_match_frozen() {
    for case in CASES.iter() {
        let cfg = config(case);
        let fresh = generate(&cfg, case, case.input, "comparison").unwrap();
        for (file, text) in [
            ("timeseries.csv", &fresh.timeseries),
            ("final_profile.csv", &fresh.final_profile),
        ] {
            let frozen = fs::read_to_string(golden(case, file)).unwrap();
            if let Err(e) = compare(&format!("{}/{file}", case.id), &frozen, text) {
                panic!("regression mismatch (the code changed; the frozen file is not edited by hand): {e}");
            }
        }
    }
}

#[test]
fn comparison_rejects_a_perturbed_value_negative_control() {
    let case = &CASES[0];
    let frozen = fs::read_to_string(golden(case, "timeseries.csv")).unwrap();
    let fresh = generate(&config(case), case, case.input, "comparison").unwrap();
    assert!(compare("control", &frozen, &fresh.timeseries).is_ok());
    // Change one release fraction in the middle of the fresh text by a relative 1e-9.
    let mut lines: Vec<String> = fresh.timeseries.lines().map(str::to_string).collect();
    let data: Vec<usize> = (0..lines.len())
        .filter(|&i| !lines[i].starts_with('#') && !lines[i].starts_with("Time (s),"))
        .collect();
    let k = data[data.len() / 2];
    let mut cells: Vec<String> = lines[k].split(',').map(str::to_string).collect();
    let f: f64 = cells[3].parse().unwrap();
    cells[3] = format!("{:e}", f * (1.0 + 1e-9));
    lines[k] = cells.join(",");
    let perturbed = lines.join("\n");
    assert!(compare("control", &frozen, &perturbed).is_err());
}

#[test]
fn independent_checks_pass_live_and_match_the_recorded_ones() {
    for case in CASES.iter() {
        let cfg = config(case);
        let report = independent_check(&cfg, case).unwrap();
        for r in &report.rows {
            assert!(
                r.passed(),
                "{}: t={} |F_ref - F_limit| = {:e} > tolerance {:e}",
                case.id,
                r.t,
                r.limit_difference,
                r.tolerance
            );
            // The criterion is the convention of the module documentation, nothing else.
            assert_eq!(
                r.tolerance.to_bits(),
                (EXPLAINED_FRACTION * r.abs_difference + r.reference_floor).to_bits(),
                "{}: tolerance is not the stated convention",
                case.id
            );
            assert!(r.reference_floor >= 0.0);
        }
        // The recorded independent check is the same computation: same reference values.
        let frozen = fs::read_to_string(golden(case, "independent_check.csv")).unwrap();
        let (header, rows) = table(&frozen);
        assert_eq!(rows.len(), report.rows.len());
        let col = |n: &str| col_of(&header, n);
        for (row, live) in rows.iter().zip(&report.rows) {
            assert!(close(row[col("t_s")], live.t, 1.0));
            assert!(close(row[col("f_reference")], live.f_reference, 1.0));
            assert!(close(row[col("f_btcs")], live.f_run, 1.0));
            assert!(close(row[col("f_limit")], live.f_limit, 1.0));
            assert!(close(row[col("time_order")], live.time_order, 1.0));
            assert!(close(row[col("space_order")], live.space_order, 1.0));
            assert_eq!(row[col("passed")], 1.0, "recorded check must have passed");
        }
    }
}

#[test]
fn frozen_files_are_labelled_and_carry_nothing_private() {
    for case in CASES.iter() {
        for file in [
            "timeseries.csv",
            "final_profile.csv",
            "independent_check.csv",
        ] {
            let text = fs::read_to_string(golden(case, file)).unwrap();
            assert!(
                text.starts_with("# REGRESSION REFERENCE, NOT TRUTH."),
                "{}/{file} is not labelled as a regression reference",
                case.id
            );
            assert!(text.contains("independent_check.csv"));
            assert!(
                !text.contains(":\\")
                    && !text.contains("/Users/")
                    && !text.contains("/home/")
                    && !text.contains('@'),
                "{}/{file} contains a path or an address",
                case.id
            );
            assert!(!text.to_lowercase().contains("validated against experiment"));
        }
        let ts = fs::read_to_string(golden(case, "timeseries.csv")).unwrap();
        let commit_line = ts.lines().find(|l| l.starts_with("# commit: ")).unwrap();
        let commit = commit_line.trim_start_matches("# commit: ");
        assert!(
            commit.len() >= 7 && commit.chars().all(|c| c.is_ascii_hexdigit()),
            "{}: the stated commit {commit:?} is not a commit hash",
            case.id
        );
        assert!(ts.contains("# input_sha256: "));
        assert!(ts.contains("# solver_package_version: "));
    }
}
