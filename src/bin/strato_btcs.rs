//! `strato-btcs`: command-line entry for the five-layer spherical BTCS solver.
//!
//! Reads the existing TOML input format (see `inputs/synthetic/synthetic_five_layer.toml`),
//! where the field `cells` of each region is the number of grid intervals `n` of that
//! material, `dt_s` the requested step (the effective step is `end_time_s / ceil(end_time_s /
//! dt_s)`) and `output_every_steps` the output stride. It runs the BTCS solver and writes
//!
//! * `btcs_timeseries.csv`: inventory, release, balance residual with its derived bound,
//! * `btcs_profiles.csv`: radial profiles at the requested snapshot times,
//! * `btcs_run.json`: everything above plus grid, input, matrix and series diagnostics,
//!
//! each with a provenance header (solver version, commit, input digest). The result is a
//! numerical solution of the stated model; it is not experimental validation.
//!
//! ```text
//! strato-btcs <input.toml> [--out DIR] [--snapshots T1,T2,...] [--commit HASH]
//!                          [--max-nodes N] [--max-steps N]
//! ```
//!
//! `--out` defaults to the `[output] directory` of the input. Snapshots default to the start
//! and the end of the run; each requested time is served by the nearest step, never
//! interpolated. `--commit` (or the environment variable `STRATO_BUILD_COMMIT`, or a compile
//! time value of it) records the source commit in the headers.

use anyhow::{bail, Context, Result};
use std::{env, fs, path::Path};
use strato::{
    config::Config,
    numerical::btcs::{run_input, validate_input, Limits},
    postprocessing::export::{build_commit, profile_csv, run_json, timeseries_csv, Provenance},
};

/// Largest allowed number of nodes times steps, a guard against runs that cannot finish.
const WORK_LIMIT: f64 = 4.0e10;

fn take_value(args: &mut Vec<String>, name: &str) -> Result<Option<String>> {
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
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!(
            "usage: strato-btcs <input.toml> [--out DIR] [--snapshots T1,T2,...] [--commit HASH] [--max-nodes N] [--max-steps N]"
        );
        return Ok(());
    }
    let out_dir = take_value(&mut args, "--out")?;
    let snapshots_arg = take_value(&mut args, "--snapshots")?;
    let commit_arg = take_value(&mut args, "--commit")?;
    let max_nodes = take_value(&mut args, "--max-nodes")?
        .map(|v| v.parse::<usize>().context("--max-nodes"))
        .transpose()?
        .unwrap_or(1_000_000);
    let max_steps = take_value(&mut args, "--max-steps")?
        .map(|v| v.parse::<usize>().context("--max-steps"))
        .transpose()?
        .unwrap_or(10_000_000);
    if args.len() != 1 {
        bail!("expected exactly one input file; see --help");
    }
    let input_path = args.remove(0);

    let cfg = Config::load(&input_path)?;
    let limits = Limits {
        max_intervals_total: max_nodes,
        max_steps,
    };
    let input = validate_input(&cfg, &limits)?;
    let nodes: usize = input.n_intervals.iter().sum();
    if nodes as f64 * input.steps as f64 > WORK_LIMIT {
        bail!(
            "{nodes} intervals times {} steps exceeds the work limit {WORK_LIMIT:e}; reduce the grid or the number of steps",
            input.steps
        );
    }
    let end = cfg.time.end_time_s;
    let snapshot_times: Vec<f64> = match snapshots_arg {
        Some(list) => {
            let mut v = Vec::new();
            for item in list.split(',').filter(|s| !s.trim().is_empty()) {
                let t: f64 = item
                    .trim()
                    .parse()
                    .with_context(|| format!("snapshot time {item:?}"))?;
                if !(t.is_finite() && t >= 0.0) {
                    bail!("snapshot time {item:?} must be finite and not negative");
                }
                v.push(t);
            }
            v
        }
        None => vec![0.0, end],
    };

    let run = run_input(&input, &snapshot_times)?;

    let commit = commit_arg
        .or_else(|| env::var("STRATO_BUILD_COMMIT").ok())
        .unwrap_or_else(|| build_commit().to_string());
    let prov = Provenance::new(&cfg, &input, &input_path, &commit);

    let dir = out_dir.unwrap_or_else(|| cfg.output.directory.clone());
    fs::create_dir_all(&dir).with_context(|| format!("creating {dir}"))?;
    let out = Path::new(&dir);
    let ts_path = out.join("btcs_timeseries.csv");
    let pr_path = out.join("btcs_profiles.csv");
    let js_path = out.join("btcs_run.json");
    fs::write(
        &ts_path,
        timeseries_csv(&prov, &run.series, run.meta.initial_inventory)?,
    )?;
    fs::write(&pr_path, profile_csv(&prov, &run.grid, &run.snapshots)?)?;
    fs::write(&js_path, run_json(&prov, &input, &run)?)?;

    let last = &run.series[run.series.len() - 1].1;
    let m0 = run.meta.initial_inventory;
    let worst = run
        .series
        .iter()
        .map(|(_, r)| r.balance_residual.abs() / m0)
        .fold(0.0, f64::max);
    println!("case={}", cfg.case_name);
    println!("solver={}", prov.solver);
    println!("package_version={}", prov.package_version);
    println!("commit={}", prov.commit);
    println!("input_sha256={}", prov.input_sha256);
    println!("nodes_N={}", run.meta.n);
    println!("dt_s={:e}", run.meta.dt);
    println!("steps={}", run.meta.steps);
    println!("final_release_fraction={:e}", last.release_fraction);
    println!("final_inventory_mol={:e}", last.inventory);
    println!("worst_balance_residual_relative={worst:e}");
    println!(
        "final_balance_residual_bound_relative={:e}",
        last.balance_residual_bound / m0
    );
    println!("min_concentration_mol_m3={:e}", last.min_concentration);
    println!("timeseries={}", ts_path.display());
    println!("profiles={}", pr_path.display());
    println!("run_json={}", js_path.display());
    Ok(())
}
