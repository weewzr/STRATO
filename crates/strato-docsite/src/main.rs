//! Command line of strato-docsite.
//!
//! ```text
//! strato-docsite build --repo <repository root> --out <web build folder> [--commit <full hash>]
//! strato-docsite check-links <web build folder> [--base /STRATO/]
//! ```
//!
//! `build` writes the document pages into the web build (run it after `npm run build` in web/,
//! which empties that folder first). `check-links` fails if any link of the built site does not
//! resolve. Both print one line per item and exit with 1 on any failure.

use std::path::PathBuf;
use std::process::ExitCode;
use strato_docsite::{check, site};

const USAGE: &str = "usage:\n  strato-docsite build --repo <dir> --out <dir> [--commit <full hash>]\n  strato-docsite check-links <dir> [--base /STRATO/]";

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|k| args.get(k + 1))
        .cloned()
}

fn build(args: &[String]) -> Result<(), String> {
    let repo = PathBuf::from(flag(args, "--repo").ok_or("--repo is required")?);
    let out = PathBuf::from(flag(args, "--out").ok_or("--out is required")?);
    let commit = site::resolve_commit(&repo, flag(args, "--commit").as_deref())?;
    let report = site::build(&site::Config { repo, out, commit })?;
    for line in &report.lines {
        println!("{line}");
    }
    println!("RESULT PASS");
    Ok(())
}

fn check_links(args: &[String]) -> Result<(), String> {
    let dist = PathBuf::from(
        args.first()
            .filter(|a| !a.starts_with("--"))
            .ok_or("the build folder is required")?,
    );
    let base = flag(args, "--base").unwrap_or_else(|| "/STRATO/".to_string());
    let r = check::check_links(&dist, &base)?;
    for f in &r.failures {
        println!("FAIL {f}");
    }
    println!(
        "{} pages, {} links ({} to other sites, not fetched), {} failures",
        r.pages,
        r.links,
        r.external,
        r.failures.len()
    );
    if r.failures.is_empty() {
        println!("RESULT PASS");
        Ok(())
    } else {
        Err("link check: RESULT FAIL".to_string())
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("build") => build(&args[1..]),
        Some("check-links") => check_links(&args[1..]),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            for line in e.lines() {
                println!("FAIL {line}");
            }
            ExitCode::from(1)
        }
    }
}
