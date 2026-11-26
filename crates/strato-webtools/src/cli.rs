//! Command line of strato-webtools.
//!
//! ```text
//! strato-webtools case <input.toml> [--check]
//! strato-webtools catalogue [--check]
//! strato-webtools colormaps [--check]
//! strato-webtools geometry [--check]
//! strato-webtools precomputed [--check]
//! strato-webtools check-catalogue
//! strato-webtools build-wasm [--check]
//! ```
//!
//! Every subcommand takes `--root <repository>`; without it the repository is the nearest
//! ancestor of the working directory that holds web/package.json and crates/strato-wasm.
//! Exit code 0 is success, 1 a failed check or an error, 2 a wrong command line.

use std::path::Path;

use crate::util::{self, Error, Result};
use crate::{case, catalogue, check, colormaps, geometry, precomputed, wasm};

pub const USAGE: &str = "usage:
  strato-webtools case <input.toml> [--check] [--root <dir>]
  strato-webtools catalogue [--check] [--root <dir>]
  strato-webtools colormaps [--check] [--root <dir>]
  strato-webtools geometry [--check] [--root <dir>]
  strato-webtools precomputed [--check] [--root <dir>]
  strato-webtools check-catalogue [--root <dir>]
  strato-webtools build-wasm [--check] [--root <dir>]";

struct Args {
    positional: Vec<String>,
    check: bool,
    root: Option<String>,
}

fn parse(args: &[String]) -> std::result::Result<Args, String> {
    let mut out = Args {
        positional: Vec::new(),
        check: false,
        root: None,
    };
    let mut k = 0;
    while k < args.len() {
        match args[k].as_str() {
            "--check" => out.check = true,
            "--root" => {
                k += 1;
                out.root = Some(args.get(k).ok_or("--root needs a folder")?.clone());
            }
            a if a.starts_with("--") => return Err(format!("unknown option {a}")),
            a => out.positional.push(a.to_string()),
        }
        k += 1;
    }
    Ok(out)
}

fn none_expected(a: &Args) -> std::result::Result<(), String> {
    match a.positional.first() {
        Some(p) => Err(format!("unexpected argument {p}")),
        None => Ok(()),
    }
}

fn check_catalogue(root: &Path) -> Result<i32> {
    let rep = check::check(root)?;
    for line in &rep.lines {
        println!("{line}");
    }
    println!(
        "{}",
        if rep.failed {
            "RESULT FAIL"
        } else {
            "RESULT PASS"
        }
    );
    Ok(i32::from(rep.failed))
}

fn catalogue_run(root: &Path, check: bool) -> Result<i32> {
    let doc = catalogue::build(root)?;
    let target = root.join(catalogue::CATALOGUE_JSON);
    let rel = util::rel_posix(root, &target);
    let text = crate::pyfmt::json_file(&doc);
    if check {
        if !target.exists() {
            println!("MISSING {rel}");
            return Ok(1);
        }
        let same =
            crate::comparable_json(&util::read_text(&target)?, &catalogue::CATALOGUE_VOLATILE)?
                == crate::comparable_json(&text, &catalogue::CATALOGUE_VOLATILE)?;
        println!("{} {rel}", if same { "SAME   " } else { "DIFFERS" });
        return Ok(i32::from(!same));
    }
    util::write_text(&target, &text)?;
    println!("WROTE   {rel}");
    Ok(0)
}

/// Run the command line; returns the exit code.
pub fn main_with(args: &[String]) -> i32 {
    let Some((cmd, rest)) = args.split_first() else {
        eprintln!("{USAGE}");
        return 2;
    };
    let a = match parse(rest) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}\n{USAGE}");
            return 2;
        }
    };
    let known = [
        "case",
        "catalogue",
        "colormaps",
        "geometry",
        "precomputed",
        "check-catalogue",
        "build-wasm",
    ];
    if !known.contains(&cmd.as_str()) {
        eprintln!("unknown subcommand {cmd}\n{USAGE}");
        return 2;
    }
    if cmd == "case" {
        if a.positional.len() != 1 {
            eprintln!("case needs exactly one input file\n{USAGE}");
            return 2;
        }
    } else if let Err(e) = none_expected(&a) {
        eprintln!("{e}\n{USAGE}");
        return 2;
    }
    if cmd == "check-catalogue" && a.check {
        eprintln!("check-catalogue takes no --check\n{USAGE}");
        return 2;
    }
    let result = (|| -> Result<i32> {
        let root = util::find_root(a.root.as_deref())?;
        match cmd.as_str() {
            "case" => case::run(&root, &a.positional[0], a.check),
            "catalogue" => catalogue_run(&root, a.check),
            "colormaps" => colormaps::run(&root, a.check),
            "geometry" => geometry::run(&root, a.check),
            "precomputed" => precomputed::run(&root, a.check),
            "check-catalogue" => check_catalogue(&root),
            "build-wasm" => wasm::run(&root, a.check),
            _ => Err(Error("unreachable".into())),
        }
    })();
    match result {
        Ok(code) => code,
        Err(e) => {
            if e.0.starts_with("FAIL") {
                println!("{e}");
            } else {
                println!("FAIL {e}");
            }
            1
        }
    }
}
