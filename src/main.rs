//! STRATO command-line program (Code A in the v1.0 description).
//!
//! Reads one TOML input file, then runs
//! - Code B (`strato::numerical::finite_volume`): the conservative spherical finite-volume
//!   method with Backward Euler time stepping and a tridiagonal solve, and
//! - Code C (`strato::analytical::sturm_liouville`): the multilayer Sturm-Liouville
//!   eigenfunction series,
//!
//! on the same inputs. It writes the CSV results, the comparison of B with C, the
//! refinement study and the SVG figures.

use anyhow::Result;
use std::{env, fs, path::Path};
use strato::{
    config::Config,
    numerical::finite_volume as fv,
    postprocessing::{comparison as compare, plots},
};

/// Skip the refinement study when it would need more than this many cell-steps.
const STUDY_COST_LIMIT: f64 = 4.0e9;

fn flag(args: &mut Vec<String>, name: &str) -> bool {
    let before = args.len();
    args.retain(|a| a != name);
    args.len() != before
}

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let fv_only = flag(&mut args, "--fv-only");
    let no_study = flag(&mut args, "--no-convergence");
    let path = args
        .first()
        .cloned()
        .unwrap_or_else(|| "inputs/synthetic/synthetic_five_layer.toml".to_string());
    let cfg = Config::load(&path)?;
    let mesh = fv::build_mesh(&cfg)?;
    let (rows, final_c) = fv::run(&cfg, &mesh)?;
    fs::create_dir_all(&cfg.output.directory)?;
    let out = Path::new(&cfg.output.directory);

    let ts_path = out.join("timeseries.csv");
    let mut w = csv::Writer::from_path(&ts_path)?;
    w.write_record([
        "time_s",
        "inventory_mol",
        "cumulative_release_mol",
        "release_fraction",
        "release_rate_mol_s",
        "balance_residual_mol",
        "min_concentration_mol_m3",
    ])?;
    for (t, d) in &rows {
        w.serialize((
            t,
            d.inventory,
            d.cumulative_release,
            d.release_fraction,
            d.release_rate,
            d.balance_residual,
            d.min_concentration,
        ))?;
    }
    w.flush()?;

    let profile_path = out.join("final_profile.csv");
    let mut p = csv::Writer::from_path(&profile_path)?;
    p.write_record(["cell", "material", "radius_m", "concentration_mol_m3"])?;
    for (i, (cell, c)) in mesh.cells.iter().zip(&final_c).enumerate() {
        p.serialize((i, cell.material, cell.centroid, c))?;
    }
    p.flush()?;

    let last = &rows.last().expect("initial row").1;
    println!("case={}", cfg.case_name);
    println!("cells={}", mesh.cells.len());
    println!("explicit_dt_bound_s={:.12e}", fv::explicit_dt_bound(&mesh));
    println!("final_release_fraction={:.12e}", last.release_fraction);
    println!("final_inventory_mol={:.12e}", last.inventory);
    println!(
        "max_final_balance_residual_mol={:.12e}",
        last.balance_residual.abs()
    );
    println!("min_final_concentration={:.12e}", last.min_concentration);
    println!("timeseries={}", ts_path.display());
    println!("profile={}", profile_path.display());

    let radii: Vec<f64> = mesh.cells.iter().map(|c| c.centroid).collect();
    let layers: Vec<(String, f64)> = cfg
        .regions
        .iter()
        .map(|r| (r.name.clone(), r.outer_radius_m))
        .collect();
    let fig = out.join("final_profile.svg");
    plots::profile(&fig, &cfg.case_name, &radii, &final_c, &layers)?;
    println!("figure={}", fig.display());

    if fv_only {
        return Ok(());
    }

    // Analytical cross-check. A failure here never invalidates the FV output above.
    match compare::compare_release(&cfg, &rows) {
        Ok(cmp) => {
            let cmp_path = out.join("comparison_fv_vs_analytical.csv");
            let mut c = csv::Writer::from_path(&cmp_path)?;
            c.write_record(["time_s", "release_fv", "release_analytical", "abs_error"])?;
            for r in &cmp.rows {
                match r.exact {
                    Some(e) => c.serialize((r.time_s, r.fv, e, (r.fv - e).abs()))?,
                    None => {
                        let (t, f) = (r.time_s.to_string(), r.fv.to_string());
                        c.write_record([t.as_str(), f.as_str(), "", ""])?
                    }
                }
            }
            c.flush()?;
            let fig = out.join("release_fv_vs_analytical.svg");
            plots::release_comparison(&fig, &cfg.case_name, &cmp)?;
            println!("analytical_modes={}", cmp.modes);
            println!("analytical_min_flux_ratio={:.3e}", cmp.min_flux_ratio);
            println!("analytical_accuracy_floor_abs={:.3e}", cmp.accuracy_floor);
            if cmp.max_abs_error < 3.0 * cmp.accuracy_floor {
                println!(
                    "WARNING: FV-analytical differences are within 3x of the analytical series accuracy floor; they are not resolved."
                );
            }
            println!("analytical_valid_from_s={:.6e}", cmp.min_valid_time_s);
            println!("max_abs_error_vs_analytical={:.6e}", cmp.max_abs_error);
            println!("final_abs_error_vs_analytical={:.6e}", cmp.final_abs_error);
            println!("comparison={}", cmp_path.display());
            println!("figure={}", fig.display());
        }
        Err(e) => {
            println!("analytical_cross_check=SKIPPED ({e})");
            return Ok(());
        }
    }

    if no_study {
        return Ok(());
    }
    let cost = compare::study_cost(&cfg);
    if cost > STUDY_COST_LIMIT {
        println!(
            "convergence_study=SKIPPED (estimated {cost:.2e} cell-steps; reduce end_time/dt or use fewer cells)"
        );
        return Ok(());
    }
    match compare::convergence_study(&cfg) {
        Ok(study) => {
            let study_path = out.join("convergence_study.csv");
            let mut s = csv::Writer::from_path(&study_path)?;
            s.write_record([
                "study",
                "total_cells",
                "dt_s",
                "fv_release",
                "abs_error_vs_exact",
            ])?;
            for (name, pts) in [("spatial", &study.spatial), ("temporal", &study.temporal)] {
                for q in pts {
                    s.serialize((name, q.cells, q.dt_s, q.fv_release, q.abs_error))?;
                }
            }
            s.flush()?;
            let fig = out.join("convergence.svg");
            plots::convergence(&fig, &cfg.case_name, &study)?;
            println!("exact_final_release_fraction={:.12e}", study.exact_final);
            let fmt = |v: Vec<f64>| {
                v.iter()
                    .map(|o| format!("{o:.2}"))
                    .collect::<Vec<_>>()
                    .join(",")
            };
            let note = |ok: bool| {
                if ok {
                    ""
                } else {
                    " (WARNING: finest errors are within 3x of the analytical accuracy floor, orders may be unreliable)"
                }
            };
            println!(
                "spatial_observed_orders={}{}",
                fmt(compare::ConvergenceStudy::orders(&study.spatial)),
                note(study.resolved(&study.spatial))
            );
            println!(
                "temporal_observed_orders={}{}",
                fmt(compare::ConvergenceStudy::orders(&study.temporal)),
                note(study.resolved(&study.temporal))
            );
            println!("convergence_study={}", study_path.display());
            println!("figure={}", fig.display());
        }
        Err(e) => println!("convergence_study=SKIPPED ({e})"),
    }
    Ok(())
}
