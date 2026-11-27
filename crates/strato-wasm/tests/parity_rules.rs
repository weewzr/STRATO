//! The rules of the native-versus-WebAssembly comparison (`strato_wasm::parity`, decision 76(a)).
//!
//! A physical value that differs beyond the per-value rule still fails; a balance residual passes
//! only within its own bound on both platforms; a flag passes only with evidence at the margin;
//! no physical field is in the identity lists.

use serde_json::{json, Value};
use strato_wasm::parity::{compare_case, flag_identity, pattern, residual_identity};

/// A small reply of the shape of `run_stage`: release fractions (physical), residuals with their
/// bounds, generated amounts.
fn stage_reply(residuals: [f64; 3], bounds: [f64; 3], release: [f64; 3]) -> Value {
    let max = residuals.iter().fold(0.0_f64, |m, r| m.max(r.abs()));
    json!({
        "balance": {
            "start_inventory_mol": 1.0e-11,
            "final": {"residual_mol": residuals[2], "residual_bound_mol": bounds[2]},
            "max_abs_residual_mol": max,
            "max_abs_residual_relative_to_total": 2.0e-14,
        },
        "series": {
            "release_fraction": release,
            "generated_mol": [0.0, 1.0e-12, 2.0e-12],
            "balance_residual_mol": residuals,
            "balance_residual_bound_mol": bounds,
            "balance_residual_relative": [1.0e-15, 2.0e-15, 3.0e-15],
        },
    })
}

#[test]
fn a_physical_value_beyond_the_per_value_rule_fails() {
    let n = stage_reply([1e-26, 2e-26, 3e-26], [1e-22; 3], [0.1, 0.2, 0.3]);
    let mut w = n.clone();
    // 1e-9 relative: far outside 1e-12, and the field is not in any identity list.
    w["series"]["release_fraction"][1] = json!(0.2 * (1.0 + 1.0e-9));
    let t = compare_case("run_stage_4_ramp", &n, &w);
    assert_eq!(t.failures.len(), 1);
    assert!(t.failures.contains_key(".series.release_fraction[]"));
    // The same difference in a residual-like name that is not in the list also fails.
    let mut w = n.clone();
    w["series"]["generated_mol"][2] = json!(2.0e-12 * (1.0 + 1.0e-6));
    assert!(compare_case("run_stage_4_ramp", &n, &w)
        .failures
        .contains_key(".series.generated_mol[]"));
}

#[test]
fn a_residual_within_its_bound_on_both_platforms_passes() {
    let n = stage_reply([1e-26, 2e-26, 3e-26], [1e-22; 3], [0.1, 0.2, 0.3]);
    let mut w = n.clone();
    // 7 % relative difference of the residual, both far inside the bound.
    w["series"]["balance_residual_mol"][0] = json!(0.93e-26);
    w["balance"]["final"]["residual_mol"] = json!(3.2e-26);
    w["balance"]["max_abs_residual_mol"] = json!(3.2e-26);
    let t = compare_case("run_stage_4_ramp", &n, &w);
    assert!(t.failures.is_empty(), "{:?}", t.failures);
    assert_eq!(t.residual_excused, 3);
    assert!(t.residual_covered >= 5);
}

#[test]
fn a_residual_outside_its_bound_fails() {
    let n = stage_reply([1e-26, 2e-26, 3e-26], [1e-22; 3], [0.1, 0.2, 0.3]);
    // The WebAssembly residual exceeds its bound at row 1.
    let mut w = n.clone();
    w["series"]["balance_residual_mol"][1] = json!(2.0e-22);
    let t = compare_case("run_stage_4_ramp", &n, &w);
    assert!(t.failures.contains_key(".series.balance_residual_mol[]"));
    // The native residual exceeds its bound.
    let n2 = stage_reply([1e-26, 5e-22, 3e-26], [1e-22; 3], [0.1, 0.2, 0.3]);
    let w2 = stage_reply([1e-26, 4e-26, 3e-26], [1e-22; 3], [0.1, 0.2, 0.3]);
    assert!(compare_case("run_stage_4_ramp", &n2, &w2)
        .failures
        .contains_key(".series.balance_residual_mol[]"));
    // The final residual outside its bound.
    let mut w = n.clone();
    w["balance"]["final"]["residual_mol"] = json!(4.0e-22);
    assert!(compare_case("run_stage_4_ramp", &n, &w)
        .failures
        .contains_key(".balance.final.residual_mol"));
}

#[test]
fn the_bound_field_obeys_the_per_value_rule() {
    let n = stage_reply([1e-26, 2e-26, 3e-26], [1e-22; 3], [0.1, 0.2, 0.3]);
    let mut w = n.clone();
    w["series"]["balance_residual_mol"][0] = json!(0.9e-26);
    // The bound differs between the platforms beyond rule 1: the residual rule does not apply,
    // and the bound field itself fails the per-value rule.
    w["series"]["balance_residual_bound_mol"][0] = json!(1.1e-22);
    let t = compare_case("run_stage_4_ramp", &n, &w);
    assert!(t
        .failures
        .contains_key(".series.balance_residual_bound_mol[]"));
    assert!(t.failures.contains_key(".series.balance_residual_mol[]"));
}

#[test]
fn the_identity_lists_hold_no_physical_field() {
    for physical in [
        ".release_fraction[]",
        ".series.release_fraction[]",
        ".series.inventory_mol[]",
        ".series.generated_mol[]",
        ".series.decayed_mol[]",
        ".series.cumulative_release_mol[]",
        ".series.balance_residual_bound_mol[]",
        ".balance.final.residual_bound_mol",
        ".balance.final.released_mol",
        ".balance.final.remaining_mol",
        ".balance.start_inventory_mol",
        ".profiles[].concentration[]",
        ".analytical.release_fraction[]",
        ".method_b.release_fraction[]",
        ".method_b.accuracy_floor",
        ".accuracy_floor",
        ".missing_weight",
        ".valid_from_s",
        ".series.diffusivity_used_m2_s[]",
    ] {
        assert_eq!(residual_identity(physical), None, "{physical}");
        assert!(!flag_identity(physical), "{physical}");
    }
    // And the lists name residuals and flags only.
    assert!(residual_identity(".series.balance_residual_mol[]").is_some());
    assert!(residual_identity(".balance.final.residual_mol").is_some());
    assert!(flag_identity(".withheld_reason_codes[]"));
    assert!(flag_identity(".method_b.drawn[]"));
    assert_eq!(
        pattern(".series.balance_residual_mol[12]"),
        ".series.balance_residual_mol[]"
    );
}

fn analytical_reply(codes: [Value; 2], cover: [f64; 2]) -> Value {
    json!({
        "accuracy_floor": 4.5e-14,
        "release_fraction": [null, null],
        "drawn": [false, false],
        "drawn_count": 0,
        "withheld_reason_codes": codes,
        "mask_relative_tolerance": 0.05,
        "cover_relative_difference": cover,
        "relative_error_estimate": [null, null],
    })
}

#[test]
fn a_flag_without_evidence_fails() {
    let n = analytical_reply(
        [json!("COVER_SENSITIVITY"), json!("COVER_SENSITIVITY")],
        [0.06, 0.06],
    );
    let w = analytical_reply(
        [
            json!("ERROR_ESTIMATE_UNDEFINED"),
            json!("COVER_SENSITIVITY"),
        ],
        [0.06, 0.06],
    );
    // The estimates agree but do not lie on either side of the threshold: no margin.
    let t = compare_case("analytical_stage_5_constant", &n, &w);
    assert!(t.failures.contains_key(".withheld_reason_codes[]"));
    // A reply without estimates gives no evidence either.
    let bare = |c: Value| json!({"accuracy_floor": 4.5e-14, "withheld_reason_codes": [c]});
    let t = compare_case(
        "analytical_stage_5_constant",
        &bare(json!("COVER_SENSITIVITY")),
        &bare(json!("ERROR_ESTIMATE_UNDEFINED")),
    );
    assert!(t.failures.contains_key(".withheld_reason_codes[]"));
}

#[test]
fn a_flag_at_the_margin_with_agreeing_estimates_passes() {
    // Both platforms put the estimate at the threshold to the last digits, one just above, one just below.
    let above = 0.05 * (1.0 + 4.0e-13);
    let n = analytical_reply(
        [json!("COVER_SENSITIVITY"), json!("COVER_SENSITIVITY")],
        [above, 0.06],
    );
    let w = analytical_reply(
        [
            json!("ERROR_ESTIMATE_UNDEFINED"),
            json!("COVER_SENSITIVITY"),
        ],
        [0.05, 0.06],
    );
    let t = compare_case("analytical_stage_5_constant", &n, &w);
    assert!(
        t.failures.contains_key(".cover_relative_difference[]") == false,
        "{:?}",
        t.failures
    );
    // The estimate field itself obeys the per-value rule (4e-13 < 1e-12), and the flag is excused.
    assert!(t.failures.is_empty(), "{:?}", t.failures);
    assert_eq!(t.flag_excused, 1);
    // Estimates on either side of the threshold that do NOT agree by the per-value rule: fails.
    let n = analytical_reply(
        [json!("COVER_SENSITIVITY"), json!("COVER_SENSITIVITY")],
        [0.0508, 0.06],
    );
    let w = analytical_reply(
        [
            json!("ERROR_ESTIMATE_UNDEFINED"),
            json!("COVER_SENSITIVITY"),
        ],
        [0.0484, 0.06],
    );
    let t = compare_case("analytical_stage_5_constant", &n, &w);
    assert!(t.failures.contains_key(".withheld_reason_codes[]"));
}

#[test]
fn a_drawn_flag_that_changes_with_a_physical_value_fails() {
    let mut n = analytical_reply([json!("COVER_SENSITIVITY"), json!(null)], [0.06, 0.01]);
    n["drawn"] = json!([false, true]);
    n["drawn_count"] = json!(1);
    n["release_fraction"] = json!([null, 0.2]);
    let mut w = n.clone();
    w["drawn"] = json!([false, false]);
    w["drawn_count"] = json!(0);
    w["release_fraction"] = json!([null, null]);
    w["withheld_reason_codes"] = json!(["COVER_SENSITIVITY", "COVER_SENSITIVITY"]);
    let t = compare_case("analytical_stage_5_constant", &n, &w);
    assert!(!t.failures.is_empty());
    // A drawn value that differs beyond the rule fails whatever the flag does.
    let mut w = n.clone();
    w["release_fraction"] = json!([null, 0.2 * (1.0 + 1.0e-3)]);
    let t = compare_case("run_stage_1", &n, &w);
    assert!(t.failures.contains_key(".release_fraction[]"));
}
