//! Regression of the analytical series (Method B) of the synthetic five-layer case.
//!
//! The reference file `tests/golden/analytical/series_synthetic_five_layer.csv` holds the release
//! fraction of the Sturm-Liouville series that the version 1.0.1 command line wrote, copied
//! unchanged from the series column of its comparison file (see
//! `tests/golden/analytical/README.md`). The series module has not changed since; a change that
//! moves these values beyond round-off is a change to the analytical reference and must be
//! documented, not silently accepted.

use strato::{config::Config, postprocessing::comparison};

const SYNTHETIC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/inputs/synthetic/synthetic_five_layer.toml"
);
const GOLDEN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/golden/analytical/series_synthetic_five_layer.csv"
);

/// Relative tolerance for values that are pure round-off apart (platform libm).
const RTOL: f64 = 1e-12;

fn close(a: f64, b: f64, scale: f64) -> bool {
    (a - b).abs() <= RTOL * scale.max(a.abs()).max(b.abs())
}

#[test]
fn analytical_series_matches_its_reference_file() {
    let cfg = Config::load(SYNTHETIC).unwrap();
    let mut r = csv::Reader::from_path(GOLDEN).unwrap();
    // The series is only resolved after its stated validity time.
    let mut checked = 0;
    for rec in r.records() {
        let rec = rec.unwrap();
        let t: f64 = rec[0].parse().unwrap();
        let g: f64 = rec[1].parse().unwrap();
        if t <= 2.0e4 {
            continue;
        }
        let f = comparison::exact_release_at(&cfg, t).unwrap();
        assert!(close(f, g, 1.0), "t={t} series {f} vs reference {g}");
        checked += 1;
    }
    assert!(checked > 50, "only {checked} reference points checked");
}
