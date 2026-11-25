//! Helpers shared by the integration tests.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::OnceLock;

use strato_data::checks::{run_all, Report};
use strato_data::register::RegisterTexts;
use strato_data::Register;

/// The `data` folder of the repository this crate lives in.
pub fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

/// The real register texts.
pub fn real_texts() -> RegisterTexts {
    RegisterTexts::read(&data_dir()).expect("the six register files are readable")
}

/// The text of `data/research/species.csv`, when it exists.
pub fn species_csv() -> Option<String> {
    std::fs::read_to_string(data_dir().join("research").join("species.csv")).ok()
}

/// The typed real register.
pub fn real_register() -> &'static Register {
    static R: OnceLock<Register> = OnceLock::new();
    R.get_or_init(|| Register::load(&data_dir()).expect("the real register loads"))
}

/// The report of the whole check set on the real files, computed once.
pub fn real_report() -> &'static Report {
    static R: OnceLock<Report> = OnceLock::new();
    R.get_or_init(|| run_all(&real_texts(), species_csv().as_deref()))
}

/// Replace `from` by `to` once, inside the line that starts with `line_prefix`; panics unless the
/// line exists exactly once and contains `from` exactly once.
pub fn mutate_line(text: &str, line_prefix: &str, from: &str, to: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let hits: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.starts_with(line_prefix))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "line starting {line_prefix:?} must exist exactly once"
    );
    let line = lines[hits[0]];
    assert_eq!(
        line.matches(from).count(),
        1,
        "{from:?} must occur exactly once in the line {line_prefix:?}"
    );
    let mut out: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    out[hits[0]] = line.replace(from, to);
    out.join("\n")
}

/// Remove the line that starts with `line_prefix`.
pub fn drop_line(text: &str, line_prefix: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let n = lines.iter().filter(|l| l.starts_with(line_prefix)).count();
    assert_eq!(
        n, 1,
        "line starting {line_prefix:?} must exist exactly once"
    );
    lines
        .into_iter()
        .filter(|l| !l.starts_with(line_prefix))
        .collect::<Vec<_>>()
        .join("\n")
}
