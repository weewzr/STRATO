//! The scenario files of `data/scenarios/` pass the data checks, and each check fails on a
//! mutation of the file that it is meant to catch.

mod common;

use common::{data_dir, mutate_line, real_register};
use strato_data::checks::Report;
use strato_data::scenarios::{run_checks, ScenarioTexts, Scenarios};

fn report_of(texts: &ScenarioTexts) -> Report {
    let sc = Scenarios::parse(texts).expect("the scenario texts parse");
    run_checks(real_register(), &sc)
}

fn real() -> ScenarioTexts {
    ScenarioTexts::read(&data_dir()).expect("the scenario files are readable")
}

fn failing(rep: &Report) -> Vec<&str> {
    rep.results
        .iter()
        .filter(|r| !r.passed)
        .map(|r| r.id.as_str())
        .collect()
}

#[test]
fn the_real_scenario_files_pass_every_check() {
    let rep = report_of(&real());
    assert!(rep.results.len() >= 10, "{} checks", rep.results.len());
    assert!(failing(&rep).is_empty(), "{:?}", failing(&rep));
}

#[test]
fn a_time_that_goes_backwards_fails_the_monotone_check() {
    let mut t = real();
    t.files[0] = mutate_line(&t.files[0], "S1-023,", ",620.2,d,", ",0,d,");
    assert!(failing(&report_of(&t)).contains(&"SC.times"));
}

#[test]
fn an_unknown_source_id_fails_the_resolution_check() {
    let mut t = real();
    t.files[0] = mutate_line(&t.files[0], "S1-022,", "\"IRR-059\"", "\"IRR-9999\"");
    assert!(failing(&report_of(&t)).contains(&"SC.sources"));
}

#[test]
fn an_unlabelled_number_fails_the_label_check() {
    let mut t = real();
    // Scenario 2: the assumed 365 d storage loses its label.
    t.files[1] = mutate_line(
        &t.files[1],
        "S2-021,",
        "prescribed input (assumed)",
        "plausible guess",
    );
    assert!(failing(&report_of(&t)).contains(&"SC.labels"));
    // A registered label without a source id fails too.
    let mut t = real();
    t.files[2] = mutate_line(&t.files[2], "S3-025,", "\"IRR-059;IRR-002\"", "");
    assert!(failing(&report_of(&t)).contains(&"SC.labels"));
}

#[test]
fn a_wrong_unit_fails_the_unit_check() {
    let mut t = real();
    t.files[0] = mutate_line(&t.files[0], "S1-022,", ",degC,", ",K,");
    assert!(failing(&report_of(&t)).contains(&"SC.units"));
    let mut t = real();
    t.files[3] = mutate_line(&t.files[3], "S4-041,", ",7442.4,h,", ",7442.4,min,");
    assert!(failing(&report_of(&t)).contains(&"SC.units"));
}

#[test]
fn a_changed_register_value_fails_the_value_check() {
    let mut t = real();
    t.files[0] = mutate_line(&t.files[0], "S1-022,", ",1028,", ",1030,");
    assert!(failing(&report_of(&t)).contains(&"SC.values"));
}

#[test]
fn a_nuclide_without_a_source_term_cannot_be_called_meaningful() {
    let mut t = real();
    t.files[2] = mutate_line(
        &t.files[2],
        "S3-004,",
        "\"Sr-90;Cs-137\"",
        "\"Sr-90;Cs-137;Ag-110m\"",
    );
    assert!(failing(&report_of(&t)).contains(&"SC.nuclides"));
}

#[test]
fn a_history_that_ends_before_the_step_plan_fails_the_coverage_check() {
    let mut t = real();
    t.files[0] = mutate_line(&t.files[0], "S1-021,", ",620.2,d,", ",600,d,");
    assert!(failing(&report_of(&t)).contains(&"SC.coverage"));
}
