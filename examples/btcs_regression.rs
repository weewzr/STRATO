//! Generator of the BTCS regression reference files (directive Section 11, test I).
//!
//! ```text
//! cargo run --release --example btcs_regression -- [--root DIR] [--write DIR] --commit HASH
//! ```
//!
//! For every case of `strato::validation::regression::CASES` it runs the independent check (the
//! BTCS release fraction against the Sturm-Liouville series or the closed form of the homogeneous
//! sphere) and prints one line per checkpoint and ends with `BTCS_REGRESSION_CHECK PASS` (exit code 0) when every checkpoint passes,
//! otherwise it fails. With `--write DIR` it also writes
//! `DIR/<case>/timeseries.csv`, `final_profile.csv` and `independent_check.csv`, but only if every
//! checkpoint passes. `--root` is the repository root that holds the TOML inputs (default: the
//! current directory). `--commit` is the commit of the source tree the values are produced at; it
//! is written into the files. The files are regression references, not truth: see
//! `src/validation/regression.rs`.

use anyhow::{bail, Context, Result};
use std::{env, fs, path::Path};
use strato::{
    config::Config,
    validation::regression::{generate, independent_check, CASES},
};

fn take(args: &mut Vec<String>, name: &str) -> Result<Option<String>> {
    if let Some(pos) = args.iter().position(|a| a == name) {
        if pos + 1 >= args.len() {
            bail!("{name} needs a value");
        }
        let v = args.remove(pos + 1);
        args.remove(pos);
        return Ok(Some(v));
    }
    Ok(None)
}

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let root = take(&mut args, "--root")?.unwrap_or_else(|| ".".to_string());
    let write = take(&mut args, "--write")?;
    let commit = take(&mut args, "--commit")?;
    if !args.is_empty() {
        bail!("unexpected arguments: {args:?}");
    }
    if write.is_some() && commit.is_none() {
        bail!("--write needs --commit: the files record the commit they were produced at");
    }
    let mut all_passed = true;
    for case in CASES.iter() {
        let path = Path::new(&root).join(case.input);
        let cfg = Config::load(&path).with_context(|| format!("loading {}", case.input))?;
        let report = independent_check(&cfg, case)?;
        println!("case={} reference={}", case.id, report.reference_text);
        all_passed &= report.rows.iter().all(|r| r.passed());
        for r in &report.rows {
            println!(
                "  t={:e} F_run={:e} F_ref={:e} |F_run-F_ref|={:e} orders (time, space)=({:.3}, {:.3}) |F_ref-F_limit|={:e} tolerance={:e} ratio={:.3e} passed={}",
                r.t,
                r.f_run,
                r.f_reference,
                r.abs_difference,
                r.time_order,
                r.space_order,
                r.limit_difference,
                r.tolerance,
                r.limit_difference / r.tolerance,
                r.passed()
            );
        }
        if let Some(dir) = &write {
            let files = generate(&cfg, case, case.input, commit.as_deref().unwrap_or(""))?;
            let out = Path::new(dir).join(case.id);
            fs::create_dir_all(&out)?;
            fs::write(out.join("timeseries.csv"), files.timeseries)?;
            fs::write(out.join("final_profile.csv"), files.final_profile)?;
            fs::write(out.join("independent_check.csv"), files.independent_check)?;
            println!("  written: {}", case.id);
        }
    }
    if !all_passed {
        bail!("BTCS_REGRESSION_CHECK FAIL: at least one checkpoint failed its stated convention");
    }
    println!("BTCS_REGRESSION_CHECK PASS");
    Ok(())
}
