//! The checks and the readers must be able to fail. Each test damages one cell of a copy of the
//! real register text (never of the file on disk) and asserts what then happens.

mod common;

use common::{drop_line, mutate_line, real_texts, species_csv};
use strato_data::checks::run_all;
use strato_data::register::RegisterTexts;
use strato_data::Register;

fn failed_ids(t: &RegisterTexts) -> Vec<String> {
    run_all(t, species_csv().as_deref())
        .results
        .into_iter()
        .filter(|r| !r.passed)
        .map(|r| r.id)
        .collect()
}

fn assert_fails(t: &RegisterTexts, id: &str) {
    let f = failed_ids(t);
    assert!(f.iter().any(|x| x == id), "{id} should fail, failed: {f:?}");
}

#[test]
fn a_changed_outer_radius_fails_the_geometry_checks() {
    let mut t = real_texts();
    t.geometry = mutate_line(&t.geometry, "G-02,", ",278.5,", ",278.6,");
    assert_fails(&t, "D5.G.radii");
    assert_fails(&t, "GEN.geometry.radii");
}

#[test]
fn a_changed_decay_constant_fails_the_decay_checks() {
    let mut t = real_texts();
    t.decay = mutate_line(&t.decay, "Cs-137,", ",7.302187e-10,", ",7.5e-10,");
    assert_fails(&t, "D5.L.lambda");
    assert_fails(&t, "B4D.A.lambda");
}

#[test]
fn a_bound_that_loses_its_class_fails() {
    let mut t = real_texts();
    t.release = mutate_line(
        &t.release,
        "E-027,",
        ",M-UB,measured bound,",
        ",M-UB,measured,",
    );
    assert_fails(&t, "D5.V.bounds");
}

#[test]
fn an_activation_product_without_its_warning_fails() {
    let mut t = real_texts();
    t.yields = mutate_line(
        &t.yields,
        "235U,thermal,Ag-110m,",
        "do NOT use as a fission-yield source",
        "usable as a source",
    );
    assert_fails(&t, "D5.V.activation-yields");
}

#[test]
fn a_wrong_cumulative_time_fails() {
    let mut t = real_texts();
    t.temperature = mutate_line(&t.temperature, "TH-008,", ",2,5.0833,", ",2,6.0833,");
    assert_fails(&t, "D5.T.cumulative");
}

#[test]
fn a_changed_diffusivity_fails_the_b6_and_audit_table_checks() {
    let mut t = real_texts();
    t.arrhenius = mutate_line(&t.arrhenius, "A3-05,", ",3.6e-9,", ",3.7e-9,");
    assert_fails(&t, "D5.V.b6-values");
    assert_fails(&t, "D5.A.frg-values");
}

#[test]
fn a_changed_reduced_coefficient_fails() {
    let mut t = real_texts();
    t.arrhenius = mutate_line(
        &t.arrhenius,
        "T71-01,",
        "D'0 = 0.90 s^-1",
        "D'0 = 0.95 s^-1",
    );
    assert_fails(&t, "D5.K.T71-01");
}

#[test]
fn a_changed_failure_count_fails() {
    let mut t = real_texts();
    // E-110 counts 11 failed particles at 1800 C; 12 breaks the sum of Collin (2016) Table 3
    t.release = mutate_line(
        &t.release,
        "E-110,",
        ",=,11,,,particles,",
        ",=,12,,,particles,",
    );
    assert_fails(&t, "D5.E.failures-1800");
}

#[test]
fn a_catalogue_nuclide_missing_from_decay_csv_fails() {
    let t = real_texts();
    let species =
        "species,element,nuclides\nZz,zz,Zz-999\nAg,silver,Ag-110m;Pd-105 (not in register)\n";
    let r = run_all(&t, Some(species));
    assert!(!r.get("CAT.nuclides").unwrap().passed);
}

// ---- the readers refuse what they cannot type

fn load_err(t: &RegisterTexts) -> String {
    Register::from_texts(t)
        .expect_err("the damaged text must be refused")
        .to_string()
}

#[test]
fn an_unknown_status_is_an_error_naming_file_line_row_and_column() {
    let mut t = real_texts();
    t.decay = mutate_line(&t.decay, "Cs-137,", ",verified", ",probably");
    let e = load_err(&t);
    assert!(e.contains("nuclear/decay.csv"), "{e}");
    assert!(e.contains("Cs-137"), "{e}");
    assert!(e.contains("status"), "{e}");
    assert!(e.contains("probably"), "{e}");
}

#[test]
fn a_ragged_row_is_an_error() {
    let mut t = real_texts();
    t.geometry = mutate_line(&t.geometry, "G-03,", ",verified,", ",verified,extra,");
    assert!(load_err(&t).contains("cells"));
    // the schema check on the raw text sees it as well
    assert_fails(&t, "D5.S.rect.geometry");
}

#[test]
fn a_renamed_column_is_an_error() {
    let mut t = real_texts();
    t.release = t
        .release
        .replacen("particle_applicability", "applicability", 1);
    let e = load_err(&t);
    assert!(e.contains("header"), "{e}");
}

#[test]
fn a_duplicate_id_is_an_error() {
    let mut t = real_texts();
    t.arrhenius = mutate_line(&t.arrhenius, "A1-02,", "A1-02,T978-A1", "A1-01,T978-A1");
    assert!(load_err(&t).contains("twice"));
    assert_fails(&t, "D5.S.ids.arrhenius");
}

#[test]
fn a_number_that_is_not_a_number_is_an_error() {
    let mut t = real_texts();
    t.arrhenius = mutate_line(&t.arrhenius, "A1-03,", ",2.2e-3,", ",2.2e-3x,");
    let e = load_err(&t);
    assert!(e.contains("D0_m2_s") && e.contains("A1-03"), "{e}");
}

#[test]
fn a_blank_yield_is_not_read_as_zero() {
    let r = common::real_register();
    let row = r
        .yields
        .rows
        .iter()
        .find(|y| y.product.name() == "Ag-110m" && y.parent == strato_data::yields::Parent::U235)
        .unwrap();
    assert!(row.jeff_cumulative.reported().is_none());
    assert!(row.jeff_independent.reported().is_none());
}

#[test]
fn a_dropped_row_is_noticed_by_the_row_count_check_of_the_other_readers() {
    // dropping a data row from the text is a different file, not a silent loss: the reader holds
    // every row it is given and the count follows the file
    let mut t = real_texts();
    t.arrhenius = drop_line(&t.arrhenius, "A1-04,");
    let reg = Register::from_texts(&t).expect("still a valid file");
    assert_eq!(reg.arrhenius.rows.len(), 117);
    // but the B6 re-read of the audit, which lists A1-04, notices the loss
    assert_fails(&t, "D5.V.b6-coverage");
}
