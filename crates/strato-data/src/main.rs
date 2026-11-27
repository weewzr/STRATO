//! `strato-data`: command line of the register data layer.
//!
//! ```text
//! strato-data check [--repo <folder>]    run every consistency check on the six register files
//! strato-data summary [--repo <folder>]  count the rows of the six files as the typed readers see them
//! strato-data register (--check | --write) [--repo <folder>]
//!                                         the generated tables of STRATO_Parameter_Register.md
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
    let mut mode: Option<&str> = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--check" => mode = Some("check"),
            "--write" => mode = Some("write"),
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
        "register" => match mode {
            Some(m) => run_register(&data, m == "write"),
            None => usage("register needs --check or --write"),
        },
        _ => usage("expected a command: check, summary or register"),
    }
}

fn usage(why: &str) -> ExitCode {
    eprintln!("strato-data: {why}");
    eprintln!("usage: strato-data (check | summary) [--repo <folder>]");
    eprintln!("       strato-data register (--check | --write) [--repo <folder>]");
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
    let mut report = strato_data::checks::run_all(&texts, species.as_deref());
    // The scenario definitions of data/scenarios/ against the registers.
    match scenario_report(data, &texts) {
        Ok(r) => report.results.extend(r.results),
        Err(e) => report.results.push(strato_data::checks::CheckResult {
            id: "SC.load".to_string(),
            passed: false,
            description: "the scenario files are read by the typed reader".to_string(),
            detail: e,
        }),
    }
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

fn scenario_report(
    data: &std::path::Path,
    texts: &strato_data::register::RegisterTexts,
) -> Result<strato_data::checks::Report, String> {
    let reg = Register::from_texts(texts).map_err(|e| e.to_string())?;
    let st = strato_data::scenarios::ScenarioTexts::read(data).map_err(|e| e.to_string())?;
    let sc = strato_data::scenarios::Scenarios::parse(&st).map_err(|e| e.to_string())?;
    Ok(strato_data::scenarios::run_checks(&reg, &sc))
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

/// The generated tables of the register document: the same tables as the CSV files say, or (with
/// `--write`) the document rewritten so that they are; the hand-written text is never touched.
fn run_register(data: &std::path::Path, write: bool) -> ExitCode {
    let path = data
        .join("reference")
        .join(strato_data::register::REGISTER_MARKDOWN);
    let texts = match strato_data::register::RegisterTexts::read(data) {
        Ok(t) => t,
        Err(e) => {
            println!("FAIL [load] {e}");
            return ExitCode::from(1);
        }
    };
    let document = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(e) => {
            println!("FAIL [load] {}: {e}", path.display());
            return ExitCode::from(1);
        }
    };
    let tables = match strato_data::register_render::Tables::read(&texts) {
        Ok(t) => t,
        Err(e) => {
            println!("FAIL [load] {e}");
            return ExitCode::from(1);
        }
    };
    let (text, reports) = match strato_data::register_render::process(&document, &tables, write) {
        Ok(r) => r,
        Err(e) => {
            println!("FAIL [markers] {e}");
            return ExitCode::from(1);
        }
    };
    let stale = reports.iter().filter(|r| !r.current).count();
    for r in &reports {
        println!(
            "{} line {}: {} ({} lines){}",
            if r.current {
                "PASS"
            } else if write {
                "WROTE"
            } else {
                "FAIL"
            },
            r.line,
            r.spec,
            r.lines,
            r.first_difference
                .as_ref()
                .map(|d| format!(": {d}"))
                .unwrap_or_default()
        );
    }
    if write {
        if text != document {
            if let Err(e) = std::fs::write(&path, text.as_bytes()) {
                println!("FAIL [write] {}: {e}", path.display());
                return ExitCode::from(1);
            }
        }
        println!(
            "SUMMARY: {} generated tables, {stale} rewritten",
            reports.len()
        );
        return ExitCode::SUCCESS;
    }
    println!(
        "SUMMARY: {} generated tables, {stale} differ from the CSV files",
        reports.len()
    );
    if stale == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
