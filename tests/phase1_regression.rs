//! Phase 1 (v1.0) regression: the current build must reproduce the golden v1.0 outputs.
//!
//! The golden files in `tests/golden/v1.0/` are the CI outputs of the original Phase 1
//! deliverables (see `docs/development/provenance.md`). Any change to the numerics that
//! moves these values beyond round-off is a change to the v1.0 results and must be
//! documented, not silently accepted.

use strato::{config::Config, numerical::finite_volume as fv, postprocessing::comparison};

const SYNTHETIC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/inputs/synthetic/synthetic_five_layer.toml"
);
const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/v1.0/");

/// Relative tolerance for values that are pure round-off apart (platform libm).
const RTOL: f64 = 1e-12;

fn read_rows(name: &str) -> Vec<Vec<f64>> {
    let mut r = csv::Reader::from_path(format!("{GOLDEN}{name}")).unwrap();
    r.records()
        .map(|rec| {
            rec.unwrap()
                .iter()
                .map(|s| {
                    if s.is_empty() {
                        f64::NAN
                    } else {
                        s.parse().unwrap()
                    }
                })
                .collect()
        })
        .collect()
}

fn close(a: f64, b: f64, scale: f64) -> bool {
    (a - b).abs() <= RTOL * scale.max(a.abs()).max(b.abs())
}

#[test]
fn finite_volume_timeseries_matches_v1_golden() {
    let cfg = Config::load(SYNTHETIC).unwrap();
    let mesh = fv::build_mesh(&cfg).unwrap();
    let (rows, _) = fv::run(&cfg, &mesh).unwrap();
    let golden = read_rows("timeseries.csv");
    assert_eq!(rows.len(), golden.len());
    let m0 = golden[0][1];
    for ((t, d), g) in rows.iter().zip(&golden) {
        assert!(close(*t, g[0], 1.0), "time {t} vs {}", g[0]);
        assert!(close(d.inventory, g[1], m0), "inventory at t={t}");
        assert!(close(d.cumulative_release, g[2], m0), "release at t={t}");
        assert!(close(d.release_fraction, g[3], 1.0), "F at t={t}");
    }
}

#[test]
fn finite_volume_final_profile_matches_v1_golden() {
    let cfg = Config::load(SYNTHETIC).unwrap();
    let mesh = fv::build_mesh(&cfg).unwrap();
    let (_, c) = fv::run(&cfg, &mesh).unwrap();
    let golden = read_rows("final_profile.csv");
    assert_eq!(c.len(), golden.len());
    let cmax = golden.iter().map(|g| g[3]).fold(0.0, f64::max);
    for ((ci, cell), g) in c.iter().zip(&mesh.cells).zip(&golden) {
        assert!(close(cell.centroid, g[2], cell.centroid), "radius");
        assert!(close(*ci, g[3], cmax), "concentration {ci} vs {}", g[3]);
    }
}

#[test]
fn analytical_series_matches_v1_golden() {
    let cfg = Config::load(SYNTHETIC).unwrap();
    let golden = read_rows("comparison_fv_vs_analytical.csv");
    // The series is only resolved after its stated validity time; empty cells are NaN.
    let mut checked = 0;
    for g in golden.iter().filter(|g| g[2].is_finite() && g[0] > 2.0e4) {
        let f = comparison::exact_release_at(&cfg, g[0]).unwrap();
        assert!(
            close(f, g[2], 1.0),
            "t={} series {f} vs golden {}",
            g[0],
            g[2]
        );
        checked += 1;
    }
    assert!(checked > 50, "only {checked} golden points checked");
}
