//! Checks of the analytical reference (Method B, the Sturm-Liouville series) and of the input
//! templates that run under plain `cargo test`. None of them runs a numerical solver.

use strato::{
    analytical::sturm_liouville::ModalRelease,
    config::Config,
    postprocessing::comparison::{exact_release_at, layers_from_config, MAX_MODES},
};

const SYNTHETIC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/inputs/synthetic/synthetic_five_layer.toml"
);
const TEMPLATE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/inputs/literature/literature_inputs.template.toml"
);

#[test]
fn exact_series_matches_independent_python_oracle() {
    // Independent eigenfunction-expansion script (different code, different language,
    // scripts/derivation_checks/sec1_synthetic_case.py): F(2e5 s) = 0.4325556390930383 for the
    // synthetic five-layer case.
    let cfg = Config::load(SYNTHETIC).unwrap();
    let exact = exact_release_at(&cfg, 2.0e5).unwrap();
    assert!((exact - 0.4325556390930383).abs() < 1e-10, "{exact}");
}

#[test]
fn modal_weights_approach_one_as_more_modes_are_included() {
    // The weights of the eigenfunction expansion sum to 1 when every mode is included
    // (F(0) = 0); the deficit must shrink as the covered eigenvalue range grows.
    let cfg = Config::load(SYNTHETIC).unwrap();
    let layers = layers_from_config(&cfg);
    let mut last_gap = f64::INFINITY;
    for lambda_max in [2.0e-3, 2.0e-2, 2.0e-1] {
        let s = ModalRelease::new(&layers, lambda_max, MAX_MODES).unwrap();
        let gap = (1.0 - s.weights.iter().sum::<f64>()).abs();
        assert!(gap < last_gap, "weight deficit must shrink with more modes");
        last_gap = gap;
    }
    assert!(last_gap < 5e-2, "{last_gap}");
}

#[test]
fn unfilled_template_is_rejected_with_a_readable_list() {
    let err = Config::load(TEMPLATE).unwrap_err().to_string();
    assert!(err.contains("INSERT_VALUE_HERE"), "{err}");
    assert!(err.contains("KERNEL OUTER RADIUS"), "{err}");
    assert!(err.contains("INITIAL KERNEL CONCENTRATION"), "{err}");
}
