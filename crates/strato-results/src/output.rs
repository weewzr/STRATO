//! Provenance, the in-memory file set, and the writing and checking of it.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use strato::postprocessing::export::{sha256_hex, LABEL, SOLVER_ID};
use strato_compare::json::J;
use strato_compare::outputs::Csv;
use strato_data::register::RegisterTexts;
use strato_data::scenarios::ScenarioTexts;

/// The version of the layout of the output files.
pub const SCHEMA_VERSION: &str = "1";
/// The version of the pipeline program.
pub const PIPELINE_VERSION: &str = "0.1.0";

/// The files of one run: relative path to text, in a fixed order.
pub type Files = BTreeMap<String, String>;

/// Where every output file came from.
#[derive(Debug, Clone)]
pub struct Prov {
    /// Commit of the source tree the numbers were computed from (informational: the check mode
    /// does not compare it, because committing the outputs moves the commit).
    pub commit: String,
    /// Version of the solver package.
    pub solver_version: String,
    /// SHA-256 of every input file, by name.
    pub inputs: Vec<(String, String)>,
    /// SHA-256 over the sources of the solver package (`src/**/*.rs`, line endings normalised),
    /// so that a change to the solver makes the committed outputs stale.
    pub solver_source_sha256: String,
}

fn normalised_hash(text: &str) -> String {
    sha256_hex(text.replace('\r', "").as_bytes())
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out)?;
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
    Ok(())
}

/// The hash of the solver sources below `repo/src`.
///
/// # Errors
///
/// A file that cannot be read.
pub fn solver_source_hash(repo: &Path) -> Result<String, String> {
    let root = repo.join("src");
    let mut files = Vec::new();
    collect_rs(&root, &mut files).map_err(|e| format!("{}: {e}", root.display()))?;
    let mut all = String::new();
    for f in files {
        let rel = f
            .strip_prefix(&root)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(&f).map_err(|e| format!("{}: {e}", f.display()))?;
        let _ = writeln!(all, "{rel}\0{}", normalised_hash(&text));
    }
    Ok(sha256_hex(all.as_bytes()))
}

impl Prov {
    /// Collect the provenance.
    ///
    /// # Errors
    ///
    /// The solver sources cannot be read.
    pub fn new(
        repo: &Path,
        commit: &str,
        texts: &RegisterTexts,
        scen: &ScenarioTexts,
    ) -> Result<Prov, String> {
        let mut inputs: Vec<(String, String)> = vec![
            ("geometry.csv".into(), normalised_hash(&texts.geometry)),
            ("arrhenius.csv".into(), normalised_hash(&texts.arrhenius)),
            ("decay.csv".into(), normalised_hash(&texts.decay)),
            ("fission_yields.csv".into(), normalised_hash(&texts.yields)),
            (
                "temperature_histories.csv".into(),
                normalised_hash(&texts.temperature),
            ),
            (
                "release_measurements.csv".into(),
                normalised_hash(&texts.release),
            ),
            (
                "agr1_irradiation_conditions.csv".into(),
                normalised_hash(&texts.irradiation_conditions),
            ),
            (
                "agr1_burnup_by_cycle.csv".into(),
                normalised_hash(&texts.irradiation_burnup),
            ),
            (
                "agr1_power_history_digitised.csv".into(),
                normalised_hash(&texts.irradiation_power),
            ),
        ];
        for (name, text) in strato_data::scenarios::FILES.iter().zip(&scen.files) {
            inputs.push(((*name).to_string(), normalised_hash(text)));
        }
        Ok(Prov {
            commit: if commit.trim().is_empty() {
                "unknown".into()
            } else {
                commit.trim().into()
            },
            solver_version: strato::VERSION.to_string(),
            inputs,
            solver_source_sha256: solver_source_hash(repo)?,
        })
    }

    /// The provenance as a JSON object, written into every JSON file.
    pub fn json(&self) -> J {
        J::obj(vec![
            ("label", J::s(LABEL)),
            ("solver", J::s(SOLVER_ID)),
            ("solver_version", J::s(self.solver_version.clone())),
            (
                "pipeline",
                J::s(format!("strato-results {PIPELINE_VERSION}")),
            ),
            ("schema_version", J::s(SCHEMA_VERSION)),
            ("commit", J::s(self.commit.clone())),
            (
                "solver_source_sha256",
                J::s(self.solver_source_sha256.clone()),
            ),
            (
                "input_sha256",
                J::Obj(
                    self.inputs
                        .iter()
                        .map(|(k, v)| (k.clone(), J::s(v.clone())))
                        .collect(),
                ),
            ),
        ])
    }

    /// The comment header of a CSV file.
    pub fn csv_header(&self, what: &str) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "# STRATO shared results: {what}");
        let _ = writeln!(s, "# label: {LABEL}");
        let _ = writeln!(
            s,
            "# solver: {SOLVER_ID}, package version {}",
            self.solver_version
        );
        let _ = writeln!(
            s,
            "# pipeline: strato-results {PIPELINE_VERSION}, schema {SCHEMA_VERSION}"
        );
        let _ = writeln!(s, "# commit: {}", self.commit);
        let _ = writeln!(s, "# solver_source_sha256: {}", self.solver_source_sha256);
        for (k, v) in &self.inputs {
            let _ = writeln!(s, "# input_sha256 {k}: {v}");
        }
        s
    }

    /// A CSV table with this header.
    pub fn csv(&self, what: &str, columns: &[&str]) -> Csv {
        Csv::new(&self.csv_header(what), columns)
    }
}

/// A JSON file whose top level object starts with the provenance.
pub fn json_file(prov: &Prov, what: &str, mut body: Vec<(&str, J)>) -> String {
    let mut items = vec![
        ("schema_version", J::s(SCHEMA_VERSION)),
        ("what", J::s(what)),
        ("provenance", prov.json()),
    ];
    items.append(&mut body);
    J::obj(items).render()
}

/// Drop the lines that record the commit: committing the outputs moves the commit, so the check
/// mode compares everything else.
pub fn masked(text: &str) -> String {
    text.lines()
        .filter(|l| {
            let t = l.trim_start();
            !(t.starts_with("\"commit\":") || t.starts_with("# commit:"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The first line at which the committed text and the regenerated text differ, each cut to 100
/// characters, for the report of the check mode (empty where only the length differs).
fn first_difference(committed: &str, regenerated: &str) -> String {
    let cut = |s: &str| s.chars().take(100).collect::<String>();
    committed
        .lines()
        .zip(regenerated.lines())
        .enumerate()
        .find(|(_, (a, b))| a != b)
        .map(|(i, (a, b))| {
            format!(
                " (line {}: committed `{}`, regenerated `{}`)",
                i + 1,
                cut(a.trim()),
                cut(b.trim())
            )
        })
        .unwrap_or_default()
}

/// Write every file below `out`.
///
/// # Errors
///
/// A file that cannot be written.
pub fn write_all(out: &Path, files: &Files) -> Result<(), String> {
    for (rel, text) in files {
        let p = out.join(rel);
        if let Some(dir) = p.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        fs::write(&p, text.as_bytes()).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    Ok(())
}

/// The files that are stale: missing, different (apart from the commit lines) or not produced
/// any more. Empty when the committed outputs are current.
pub fn stale(out: &Path, files: &Files) -> Vec<String> {
    let mut bad = Vec::new();
    for (rel, text) in files {
        match fs::read_to_string(out.join(rel)) {
            Err(_) => bad.push(format!("{rel}: missing")),
            Ok(old) => {
                let (a, b) = (masked(&old.replace("\r\n", "\n")), masked(text));
                if a != b {
                    bad.push(format!(
                        "{rel}: differs from the regenerated file{}",
                        first_difference(&a, &b)
                    ));
                }
            }
        }
    }
    // A committed file that the program no longer writes.
    let mut present = Vec::new();
    collect_all(out, out, &mut present);
    for rel in present {
        if !files.contains_key(&rel) {
            bad.push(format!("{rel}: not produced by the program any more"));
        }
    }
    bad
}

/// The result of the check mode: the exit code (0 current, 1 stale) and the text for the
/// terminal.
pub fn check_report(out: &Path, files: &Files, seconds: f64) -> (u8, String) {
    let bad = stale(out, files);
    if bad.is_empty() {
        return (
            0,
            format!(
                "strato-results: {} files are current ({seconds:.1} s)",
                files.len()
            ),
        );
    }
    let mut text = String::new();
    for b in &bad {
        let _ = writeln!(text, "STALE {b}");
    }
    let _ = write!(
        text,
        "strato-results: {} problems among {} files; run `strato-results run` and commit the result",
        bad.len(),
        files.len()
    );
    (1, text)
}

fn collect_all(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        if p.is_dir() {
            collect_all(root, &p, out);
        } else if let Ok(rel) = p.strip_prefix(root) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_commit_lines_are_ignored_and_nothing_else() {
        let a = "{\n  \"commit\": \"aaa\",\n  \"x\": 1\n}\n";
        let b = "{\n  \"commit\": \"bbb\",\n  \"x\": 1\n}\n";
        let c = "{\n  \"commit\": \"aaa\",\n  \"x\": 2\n}\n";
        assert_eq!(masked(a), masked(b));
        assert_ne!(masked(a), masked(c));
        assert_eq!(masked("# commit: 1\n# x\n"), masked("# commit: 2\n# x\n"));
    }

    #[test]
    fn stale_reports_missing_changed_and_extra_files() {
        let dir = std::env::temp_dir().join(format!("strato-results-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let mut files = Files::new();
        files.insert(
            "a.json".into(),
            "{\n  \"commit\": \"1\",\n  \"v\": 1\n}\n".into(),
        );
        files.insert("sub/b.csv".into(), "# commit: 1\nx\n1\n".into());
        write_all(&dir, &files).unwrap();
        assert!(stale(&dir, &files).is_empty());
        // The commit moves: still current.
        let mut moved = files.clone();
        moved.insert(
            "a.json".into(),
            "{\n  \"commit\": \"2\",\n  \"v\": 1\n}\n".into(),
        );
        assert!(stale(&dir, &moved).is_empty());
        // A number changes: stale.
        let mut changed = files.clone();
        changed.insert("sub/b.csv".into(), "# commit: 1\nx\n2\n".into());
        assert_eq!(stale(&dir, &changed).len(), 1);
        // A file is missing, and one is extra.
        let mut more = files.clone();
        more.insert("c.json".into(), "{}\n".into());
        assert!(stale(&dir, &more)
            .iter()
            .any(|s| s.starts_with("c.json: missing")));
        let mut fewer = files.clone();
        fewer.remove("a.json");
        assert!(stale(&dir, &fewer)
            .iter()
            .any(|s| s.starts_with("a.json: not produced")));
        let _ = fs::remove_dir_all(&dir);
    }
}
