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
    assert_eq!(reg.arrhenius.rows.len(), 140);
    // but the B6 re-read of the audit, which lists A1-04, notices the loss
    assert_fails(&t, "D5.V.b6-coverage");
}

// ---- the data-integration checks can fail

#[test]
fn a_class_outside_the_four_is_an_error() {
    let mut t = real_texts();
    t.arrhenius = mutate_line(
        &t.arrhenius,
        "A1-01,",
        ",transferred material correlation,",
        ",measured at AGR-1,",
    );
    let e = load_err(&t);
    assert!(
        e.contains("coefficient_class") && e.contains("A1-01"),
        "{e}"
    );
}

#[test]
fn a_wrong_class_for_a_set_fails() {
    let mut t = real_texts();
    t.arrhenius = mutate_line(
        &t.arrhenius,
        "S20-01,",
        ",AGR-1-specific fitted correlation,",
        ",published PARFUME correlation,",
    );
    assert_fails(&t, "DI.class.one-per-row");
}

#[test]
fn a_lost_printed_range_makes_the_selection_stop_refusing() {
    let mut t = real_texts();
    t.arrhenius = mutate_line(
        &t.arrhenius,
        "C16-20,",
        ",1400,1750,printed range,",
        ",,,printed range,",
    );
    t.arrhenius = mutate_line(
        &t.arrhenius,
        "C16-20,",
        ",heating test,,,printed",
        ",heating test,,,",
    );
    assert_fails(&t, "DI.select.refuses-outside-range");
}

#[test]
fn a_range_without_its_kind_or_a_kind_without_a_range_is_a_load_error() {
    let mut t = real_texts();
    t.arrhenius = mutate_line(
        &t.arrhenius,
        "C16-20,",
        ",heating test,,,printed",
        ",heating test,,,",
    );
    assert!(Register::from_texts(&t).is_err());
    let mut t = real_texts();
    // C16-22 has no range: giving it a kind must fail
    let hit = t
        .arrhenius
        .lines()
        .find(|l| l.starts_with("C16-22,"))
        .unwrap()
        .to_string();
    assert!(hit.ends_with(",heating test,,,"), "{hit}");
    t.arrhenius = t.arrhenius.replace(&hit, &format!("{hit}fitted"));
    assert!(Register::from_texts(&t).is_err());
}

#[test]
fn the_fitted_range_of_the_schappel_rows_is_not_called_published() {
    use strato_data::arrhenius::RangeKind;
    use strato_data::select::{ParticleLayer, Phase, Selector, Variant};
    let r = common::real_register();
    let sel = Selector {
        arrhenius: &r.arrhenius,
        irradiation: &r.irradiation,
    };
    let s = sel
        .select(
            "Ag",
            ParticleLayer::SiC,
            Phase::HeatingTest,
            Variant::SchappelFitted,
        )
        .unwrap();
    assert_eq!(s.range_kind, Some(RangeKind::Fitted));
    let msg = s.diffusivity_at(1000.0).unwrap_err().to_string();
    assert!(
        msg.contains("fitted range (not printed by the source)"),
        "{msg}"
    );
    assert!(!msg.contains("published range"), "{msg}");
}

#[test]
fn a_schappel_row_changed_from_fitted_to_printed_fails() {
    for id in ["S20-01,", "S20-02,", "S20-03,"] {
        let mut t = real_texts();
        t.arrhenius = mutate_line(
            &t.arrhenius,
            id,
            ",heating test,,,fitted",
            ",heating test,,,printed",
        );
        assert_fails(&t, "DI.select.schappel-range");
    }
}

#[test]
fn a_row_with_a_printed_range_changed_to_fitted_fails() {
    let mut t = real_texts();
    let hit = t
        .arrhenius
        .split('\n')
        .find(|l| l.starts_with("C16-20,"))
        .unwrap()
        .to_string();
    assert!(hit.ends_with(",printed"), "{hit}");
    let changed = format!("{}fitted", &hit[..hit.len() - "printed".len()]);
    t.arrhenius = t.arrhenius.replace(&hit, &changed);
    assert_fails(&t, "DI.select.schappel-range");
}

#[test]
fn a_changed_default_row_breaks_the_equality_of_the_phases() {
    let mut t = real_texts();
    t.arrhenius = mutate_line(&t.arrhenius, "C15-05,", ",5.6e-8,", ",5.7e-8,");
    assert_fails(&t, "DI.select.c15-equals-c16");
}

#[test]
fn a_non_detect_that_loses_its_class_fails() {
    let mut t = real_texts();
    t.release = mutate_line(&t.release, "E-217,", ",M-ND,", ",M-UB,");
    assert_fails(&t, "DI.release.non-detect");
}

#[test]
fn a_matrix_corrected_row_that_loses_its_denominator_fails() {
    let mut t = real_texts();
    t.release = mutate_line(
        &t.release,
        "E-033,",
        ",calculated compact inventory",
        ",not established",
    );
    assert_fails(&t, "DI.release.denominator");
}

#[test]
fn a_digitised_row_without_reading_uncertainty_fails() {
    let mut t = real_texts();
    t.release = mutate_line(&t.release, "E-425,", ",rel_value_pct=0.5;time_h=0.2,", ",,");
    assert_fails(&t, "DI.release.reading-uncertainty");
}

#[test]
fn a_malformed_reading_uncertainty_is_an_error() {
    let mut t = real_texts();
    t.release = mutate_line(
        &t.release,
        "E-425,",
        ",rel_value_pct=0.5;time_h=0.2,",
        ",about half a percent,",
    );
    let e = load_err(&t);
    assert!(
        e.contains("reading_uncertainty") && e.contains("E-425"),
        "{e}"
    );
}

#[test]
fn a_reused_withdrawn_id_fails() {
    let mut t = real_texts();
    t.release = mutate_line(&t.release, "E-315,", "E-315,", "E-340,");
    assert_fails(&t, "DI.release.withdrawn-ids");
}

#[test]
fn a_burnup_that_falls_between_cycles_fails() {
    let mut t = real_texts();
    t.irradiation_burnup = mutate_line(&t.irradiation_burnup, "IRRB-0001,", ",0.72,", ",9.72,");
    assert_fails(&t, "DI.irr.monotone");
}

#[test]
fn a_changed_particle_count_changes_the_derived_rate() {
    let mut t = real_texts();
    t.irradiation_conditions =
        mutate_line(&t.irradiation_conditions, "IRR-456,", ",4154,", ",4145,");
    assert_fails(&t, "DI.irr.inventory");
}

#[test]
fn a_register_row_that_drifts_from_the_csv_fails() {
    // the old wording of the non-detect rows: class M-UB and the obsolete note
    let mut t = real_texts();
    let md = t
        .register_markdown
        .clone()
        .expect("the register document exists");
    let lines: Vec<&str> = md.split('\n').collect();
    let n = lines.iter().filter(|l| l.starts_with("| E-217 |")).count();
    assert_eq!(n, 1);
    let drifted = md.replace(
        lines.iter().find(|l| l.starts_with("| E-217 |")).unwrap(),
        &lines
            .iter()
            .find(|l| l.starts_with("| E-217 |"))
            .unwrap()
            .replace("| M-ND; t =", "| M-UB; t ="),
    );
    t.register_markdown = Some(drifted);
    assert_fails(&t, "DI.register-md.matches-csv");
}

#[test]
fn a_changed_or_truncated_coefficient_in_the_register_fails() {
    // main coefficient table (the row that starts "| C16-20 | Kr") and the section-6 copy
    // (the row that starts "| C16-08 | Term"); a changed Q, a changed exponent, a truncated one
    let md = real_texts()
        .register_markdown
        .expect("the register document exists");
    for (prefix, from, to) in [
        ("| C16-20 | Kr", "| 657 |", "| 658 |"),
        ("| C16-20 | Kr", "$3.7\\times10^{1}$", "$3.7\\times10^{2}$"),
        ("| C16-20 | Kr", "$3.7\\times10^{1}$", "$3.7$"),
        ("| C16-08 | Term", "| 5.5e-14 |", "| 5.5e-1 |"),
        ("| C16-08 | Term", "| 5.5e-14 |", "| 5.5 |"),
    ] {
        let mut t = real_texts();
        let line = md.split('\n').find(|l| l.starts_with(prefix)).unwrap();
        assert!(line.contains(from), "{prefix}: {from}");
        t.register_markdown = Some(md.replacen(line, &line.replacen(from, to, 1), 1));
        assert_fails(&t, "DI.register-md.matches-csv");
    }
}

#[test]
fn a_changed_register_value_or_range_fails() {
    let md = real_texts()
        .register_markdown
        .expect("the register document exists");
    for (prefix, from, to) in [
        ("| E-001 |", "$2.0\\times10^{-4}$", "$2.1\\times10^{-4}$"),
        (
            "| C16-20 | Term",
            "| 1400 to 1750 (printed",
            "| accident conditions (printed",
        ),
    ] {
        let mut t = real_texts();
        let line = md
            .split('\n')
            .find(|l| l.starts_with(prefix) && l.contains(from))
            .unwrap();
        assert!(line.contains(from), "{prefix}: {from}");
        t.register_markdown = Some(md.replacen(line, &line.replacen(from, to, 1), 1));
        assert_fails(&t, "DI.register-md.matches-csv");
    }
}

#[test]
fn a_kernel_label_that_says_unscaled_for_the_scaled_case_fails_the_labels_check_by_design() {
    // the selection of the sensitivity case never carries the label "unscaled"
    let r = common::real_register();
    let sel = strato_data::select::Selector {
        arrhenius: &r.arrhenius,
        irradiation: &r.irradiation,
    };
    use strato_data::select::{ParticleLayer, Phase, Variant};
    let s = sel
        .select(
            "Cs",
            ParticleLayer::Kernel,
            Phase::HeatingTest,
            Variant::KernelScaledSensitivity,
        )
        .unwrap();
    assert!(s.kernel_scale.is_some());
    assert!(
        !s.labels.iter().any(|l| l.contains("unscaled")),
        "{:?}",
        s.labels
    );
}
