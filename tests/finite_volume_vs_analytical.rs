//! Cross-module checks that run under plain `cargo test`.

use strato::{config::Config, postprocessing::comparison as compare};

const SYNTHETIC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/inputs/synthetic/synthetic_five_layer.toml"
);
const TEMPLATE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/inputs/literature/literature_inputs.template.toml"
);

#[test]
fn five_layer_fv_matches_exact_series_after_time_extrapolation() {
    // 200 cells: the spatial error at this mesh is about 5.5e-6.
    let cfg = Config::load(SYNTHETIC).unwrap();
    let exact = compare::exact_release_at(&cfg, cfg.time.end_time_s).unwrap();
    let fv = compare::time_extrapolated_release(&cfg).unwrap();
    assert!((fv - exact).abs() < 2e-5, "fv {fv} exact {exact}");
}

#[test]
fn exact_series_matches_independent_python_oracle() {
    // Independent eigenfunction-expansion script (different code, different language):
    // F(2e5 s) = 0.4325556390930383 for the synthetic five-layer case.
    let cfg = Config::load(SYNTHETIC).unwrap();
    let exact = compare::exact_release_at(&cfg, 2.0e5).unwrap();
    assert!((exact - 0.4325556390930383).abs() < 1e-10, "{exact}");
}

#[test]
fn unfilled_template_is_rejected_with_a_readable_list() {
    let err = Config::load(TEMPLATE).unwrap_err().to_string();
    assert!(err.contains("INSERT_VALUE_HERE"), "{err}");
    assert!(err.contains("KERNEL OUTER RADIUS"), "{err}");
    assert!(err.contains("INITIAL KERNEL CONCENTRATION"), "{err}");
}
