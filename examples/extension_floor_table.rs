//! Prints the estimate table of Document 6, Section 8: the error of the Method B extensions
//! (B2) to (B5) against closed-form references and self references over the designed set.
//!
//! `cargo run --release --features extension-estimate --example extension_floor_table` runs
//! the test set (about two minutes), `... -- full` the full set of the document (more than an
//! hour in one thread, the references of the layered geometries have 16 384 modes), `... -- full setup`
//! prints Table 8.1 only.

use strato::analytical::extension_floor::*;
use strato::analytical::extensions::{
    EXT_FACTOR_FLUX, EXT_FACTOR_INVENTORY, EXT_FACTOR_RELEASE, MASK_RELATIVE_TOLERANCE,
};

fn main() {
    let full = std::env::args().any(|a| a == "full");
    let cfg = if full {
        SetConfig::full_set()
    } else {
        SetConfig::test_set()
    };
    if std::env::args().any(|a| a == "setup") {
        println!("Table 8.1: geometries, diffusivities, slowest mode rate alpha_1, accuracy_floor per mode count");
        for geo in geometries(&cfg) {
            let floors: Vec<String> = cfg
                .mode_counts
                .iter()
                .map(|&n| format!("N {n}: {:.2e}", (geo.build)(n).accuracy_floor()))
                .collect();
            println!(
                "{} | {} | alpha_1 {:.3e} s^-1 | {}",
                geo.name,
                geo.diffusivities,
                geo.alpha_1,
                floors.join(", ")
            );
        }
        println!("Decay constants of the registered nuclides used (s^-1):");
        for (name, l) in registered_decay_constants() {
            let used = cfg.nuclides.contains(&name.as_str());
            println!(
                "{name} | {l:.4e} | {}",
                if used {
                    "in the set"
                } else {
                    "registered, not in the set"
                }
            );
        }
        return;
    }
    eprintln!("running the {} set", if full { "full" } else { "test" });
    let runs = run_set(&cfg);
    let cells = summarise(&cfg, &runs);

    println!("Table 8.2: per geometry, mode count and quantity");
    println!("excess = max (error - computed bound) / (accuracy_floor * scale); ratio = max error / estimate");
    println!(
        "geometry | N | quantity | points | accuracy_floor | max excess | max error / estimate"
    );
    for c in &cells {
        println!(
            "{} | {} | {:?} | {} | {:.2e} | {:.3} | {:.3}",
            c.geometry, c.n, c.q, c.points, c.floor, c.max_excess, c.max_error_over_estimate
        );
    }

    println!();
    println!("Largest excess over the whole set, and the factor of the rule");
    println!("(the rows of the physics-module reference are left out: that reference has no computed rounding bound, its excess is noise)");
    for (q, f) in [
        (Q::Inventory, EXT_FACTOR_INVENTORY),
        (Q::Release, EXT_FACTOR_RELEASE),
        (Q::Flux, EXT_FACTOR_FLUX),
    ] {
        let bounded = |c: &&Cell| c.q == q && !c.geometry.contains("physics module");
        let m = cells
            .iter()
            .filter(bounded)
            .map(|c| c.max_excess)
            .fold(f64::MIN, f64::max);
        let worst = cells
            .iter()
            .filter(bounded)
            .fold(None, |acc: Option<&Cell>, c| {
                if acc.is_none_or(|a| c.max_excess > a.max_excess) {
                    Some(c)
                } else {
                    acc
                }
            });
        println!(
            "{q:?}: max excess {m:.3} (in {}, N {}), factor {f}, margin {:.2}",
            worst.map_or("", |c| c.geometry.as_str()),
            worst.map_or(0, |c| c.n),
            f / m
        );
    }

    println!();
    println!("Weight errors of the shooting solution, homogeneous particle: N | sum |dw| | max |dw| | accuracy_floor");
    for n in [16, 64, 256, 1024] {
        let (sum, max, floor) = homogeneous_weight_errors(n);
        println!("{n} | {sum:.2e} | {max:.2e} | {floor:.2e}");
    }

    let cov = coverage(&cfg);
    println!();
    println!("Table 8.1: coverage of the set");
    println!(
        "modes {}..{}; lambda/alpha_1 up to {:.3e}; alpha_1 {:.2e}..{:.2e} s^-1; alpha_1 t up to {}; pieces up to {}; accuracy_floor up to {:.2e}",
        cov.modes.0, cov.modes.1, cov.lambda_over_alpha1_max, cov.alpha1_range.0, cov.alpha1_range.1,
        cov.alpha1_t_max, cov.pieces_max, cov.floor_max
    );

    let rep = run_mask_set(&cfg, MASK_RELATIVE_TOLERANCE);
    println!();
    println!("Mask (threshold {MASK_RELATIVE_TOLERANCE}):");
    println!(
        "asked {}, drawn {}, withheld before t_v {}, outside the regime {}, estimated error too large {}",
        rep.asked, rep.drawn, rep.before_valid, rep.outside_regime, rep.error_too_large
    );
    println!(
        "largest relative error of a drawn value {:.3e}; wrong drawn values {}; unmasked values with error above the threshold {} (withheld: {})",
        rep.max_drawn_error,
        rep.wrong_drawn.len(),
        rep.raw_wrong,
        rep.raw_wrong_withheld
    );
    println!(
        "of the values asked, F_post: asked {}, drawn {}",
        rep.post_event_asked, rep.post_event_drawn
    );
}
