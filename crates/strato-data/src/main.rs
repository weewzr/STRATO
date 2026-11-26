//! `strato-data`: command line of the register data layer.
//!
//! ```text
//! strato-data check [--repo <folder>]    run every consistency check on the six register files
//! strato-data summary [--repo <folder>]  count the rows of the six files as the typed readers see them
//! ```
//!
//! `check` prints one `PASS` or `FAIL` line per check and a summary line, and exits with status 1
//! if any check fails or a file cannot be read. It writes nothing.

use std::path::PathBuf;
use std::process::ExitCode;

use strato_data::Register;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_default();
    let mut repo = PathBuf::from(".");
    while let Some(a) = args.next() {
        match a.as_str() {
            "--repo" => match args.next() {
                Some(p) => repo = PathBuf::from(p),
                None => return usage("--repo needs a folder"),
            },
            other => return usage(&format!("unknown argument `{other}`")),
        }
    }
    let data = repo.join("data");
    match cmd.as_str() {
        "check" => run_check(&data),
        "summary" => run_summary(&data),
        _ => usage("expected a command: check or summary"),
    }
}

fn usage(why: &str) -> ExitCode {
    eprintln!("strato-data: {why}");
    eprintln!("usage: strato-data (check | summary) [--repo <folder>]");
    ExitCode::from(2)
}

fn run_check(data: &std::path::Path) -> ExitCode {
    let texts = match strato_data::register::RegisterTexts::read(data) {
        Ok(t) => t,
        Err(e) => {
            println!("FAIL [load] {e}");
            return ExitCode::from(1);
        }
    };
    let species = std::fs::read_to_string(data.join("research").join("species.csv")).ok();
    let report = strato_data::checks::run_all(&texts, species.as_deref());
    for r in &report.results {
        println!("{r}");
    }
    println!("{}", report.summary());
    if report.all_passed() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_summary(data: &std::path::Path) -> ExitCode {
    match Register::load(data) {
        Ok(r) => {
            println!("geometry rows:                {}", r.geometry.rows.len());
            println!("arrhenius rows:               {}", r.arrhenius.rows.len());
            println!("decay rows:                   {}", r.decay.rows.len());
            println!("fission yield rows:           {}", r.yields.rows.len());
            println!("temperature history rows:     {}", r.temperature.rows.len());
            println!("release measurement rows:     {}", r.release.rows.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("FAIL [load] {e}");
            ExitCode::from(1)
        }
    }
}
