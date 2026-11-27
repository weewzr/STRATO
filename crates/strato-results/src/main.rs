//! `strato-results`: the shared results pipeline.
//!
//! ```text
//! strato-results run   [--repo <folder>] [--out <folder>] [--commit <hash>] [--quick]
//! strato-results run --check [--repo <folder>] [--out <folder>] [--quick]
//! ```
//!
//! `run` writes every output file below `--out` (default `<repo>/data/results`). With `--check`
//! nothing is written: the program regenerates everything in memory and exits with status 1,
//! listing the files, if a committed output is missing, differs (apart from the commit lines) or
//! is no longer produced. `--quick` uses coarse grids for tests of the program and must never be
//! used for committed files.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use strato_results::output::{check_report, write_all};
use strato_results::pipeline::{generate, Options};

fn usage(why: &str) -> ExitCode {
    eprintln!("strato-results: {why}");
    eprintln!("usage: strato-results run [--check] [--repo <folder>] [--out <folder>] [--commit <hash>] [--quick]");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("run") {
        return usage("expected the command `run`");
    }
    let mut repo = PathBuf::from(".");
    let mut out: Option<PathBuf> = None;
    let mut commit = String::new();
    let mut quick = false;
    let mut check = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--repo" => match args.next() {
                Some(p) => repo = PathBuf::from(p),
                None => return usage("--repo needs a folder"),
            },
            "--out" => match args.next() {
                Some(p) => out = Some(PathBuf::from(p)),
                None => return usage("--out needs a folder"),
            },
            "--commit" => match args.next() {
                Some(c) => commit = c,
                None => return usage("--commit needs a hash"),
            },
            "--quick" => quick = true,
            "--check" => check = true,
            other => return usage(&format!("unknown argument `{other}`")),
        }
    }
    let out = out.unwrap_or_else(|| repo.join("data").join("results"));
    let started = Instant::now();
    let files = match generate(&Options {
        repo,
        commit,
        quick,
    }) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("strato-results: {e}");
            return ExitCode::from(1);
        }
    };
    let seconds = started.elapsed().as_secs_f64();
    if check {
        let (code, text) = check_report(&out, &files, seconds);
        println!("{text}");
        return ExitCode::from(code);
    }
    if let Err(e) = write_all(&out, &files) {
        eprintln!("strato-results: {e}");
        return ExitCode::from(1);
    }
    let bytes: usize = files.values().map(String::len).sum();
    println!(
        "strato-results: wrote {} files ({} bytes) to {} in {seconds:.1} s",
        files.len(),
        bytes,
        out.display()
    );
    ExitCode::SUCCESS
}
