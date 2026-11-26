//! `strato-compare`: command line of the experimental comparison.
//!
//! ```text
//! strato-compare run [--repo DIR] [--data-out DIR] [--figures-out DIR] [--report FILE]
//!                    [--commit HASH] [--quick]
//! ```
//!
//! `--repo` is a checkout of the repository (default: the current folder). The defaults write the
//! files into `data/comparison/`, the figures into `docs/validation/figures/` and the report into
//! `docs/validation/experimental_comparison.md` below the repository. `--commit` (or the
//! environment variable `STRATO_BUILD_COMMIT`) records the source commit in every file.

use std::path::PathBuf;
use std::process::ExitCode;

use strato_compare::pipeline::{generate, Options};
use strato_compare::status::Status;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_default();
    if cmd != "run" {
        eprintln!("usage: strato-compare run [--repo DIR] [--data-out DIR] [--figures-out DIR] [--report FILE] [--commit HASH] [--quick]");
        return ExitCode::from(2);
    }
    let mut repo = PathBuf::from(".");
    let mut data_out: Option<PathBuf> = None;
    let mut figures_out: Option<PathBuf> = None;
    let mut report: Option<PathBuf> = None;
    let mut commit = std::env::var("STRATO_BUILD_COMMIT").unwrap_or_default();
    let mut quick = false;
    while let Some(a) = args.next() {
        let mut value = |name: &str| -> Result<String, String> {
            args.next().ok_or_else(|| format!("{name} needs a value"))
        };
        let r = match a.as_str() {
            "--repo" => value("--repo").map(|v| repo = PathBuf::from(v)),
            "--data-out" => value("--data-out").map(|v| data_out = Some(PathBuf::from(v))),
            "--figures-out" => value("--figures-out").map(|v| figures_out = Some(PathBuf::from(v))),
            "--report" => value("--report").map(|v| report = Some(PathBuf::from(v))),
            "--commit" => value("--commit").map(|v| commit = v),
            "--quick" => {
                quick = true;
                Ok(())
            }
            other => Err(format!("unknown argument {other}")),
        };
        if let Err(e) = r {
            eprintln!("strato-compare: {e}");
            return ExitCode::from(2);
        }
    }
    let opts = Options {
        data_out: data_out.unwrap_or_else(|| repo.join("data").join("comparison")),
        figures_out: figures_out
            .unwrap_or_else(|| repo.join("docs").join("validation").join("figures")),
        report_out: Some(report.unwrap_or_else(|| {
            repo.join("docs")
                .join("validation")
                .join("experimental_comparison.md")
        })),
        repo,
        commit,
        quick,
    };
    match generate(&opts) {
        Ok(out) => {
            println!("rows: {}", out.records.len());
            for s in Status::ALL {
                let n = out.records.iter().filter(|r| r.status == s).count();
                println!("{:>34}: {n}", s.label());
            }
            println!("datasets: {}", out.datasets.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("strato-compare: {e}");
            ExitCode::FAILURE
        }
    }
}
