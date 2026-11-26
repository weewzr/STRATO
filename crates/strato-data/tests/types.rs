//! The distinctions of the owner's directive are types, and every column of every file survives.

mod common;

use common::{data_dir, real_register};
use strato_data::arrhenius::{BoundKind, FluenceFactor, Law};
use strato_data::common::{EvidenceClass, ProductKind, Status};
use strato_data::decay::Stability;
use strato_data::geometry::GeometryValue;
use strato_data::release::{Measured, ReleaseValue, TestTemperature};
use strato_data::table::Vocabulary;
use strato_data::temperature::TemperaturePoint;
use strato_data::yields::{
    Cumulative, EndfYield, Independent, JeffYield, Parent, SourceTermRefusal, YieldValue,
};

#[test]
fn row_counts_are_those_of_the_readme() {
    let r = real_register();
    assert_eq!(r.geometry.rows.len(), 149);
    assert_eq!(r.arrhenius.rows.len(), 141);
    assert_eq!(r.decay.rows.len(), 16);
    assert_eq!(r.yields.rows.len(), 56);
    assert_eq!(r.temperature.rows.len(), 164);
    assert_eq!(r.release.rows.len(), 600);
}

#[test]
fn status_counts_match_the_audit() {
    let r = real_register();
    let n = |s: Status| r.arrhenius.rows.iter().filter(|x| x.status == s).count();
    assert_eq!((n(Status::Verified), n(Status::NotVerified)), (138, 3));
    let n = |s: Status| r.release.rows.iter().filter(|x| x.status == s).count();
    assert_eq!(
        (
            n(Status::Verified),
            n(Status::Digitised),
            n(Status::NotVerified)
        ),
        (263, 335, 1)
    );
    let n = |s: Status| r.temperature.rows.iter().filter(|x| x.status == s).count();
    assert_eq!(
        (
            n(Status::Verified),
            n(Status::Digitised),
            n(Status::NotVerified)
        ),
        (110, 49, 5)
    );
    assert!(r.geometry.rows.iter().all(|x| x.status == Status::Verified));
}

#[test]
fn every_row_keeps_its_identifier_and_its_status() {
    let r = common::real_register();
    assert_eq!(r.geometry.rows[0].id, "G-01");
    assert_eq!(r.release.get("E-077").unwrap().status, Status::NotVerified);
    assert_eq!(
        r.arrhenius.get("A3-18").unwrap().status,
        Status::NotVerified
    );
    assert_eq!(
        r.arrhenius.get("T71-05").unwrap().status,
        Status::NotVerified
    );
}

#[test]
fn the_expected_headers_are_the_headers_of_the_files() {
    // the readers refuse any other header, so a column cannot be added or lost without a code change
    let d = data_dir();
    let first = |rel: &str| {
        std::fs::read_to_string(d.join(rel))
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .trim_start_matches('\u{feff}')
            .to_string()
    };
    assert_eq!(
        first("geometry/geometry.csv"),
        strato_data::geometry::HEADER.join(",")
    );
    assert_eq!(
        first("diffusivity/arrhenius.csv"),
        strato_data::arrhenius::HEADER.join(",")
    );
    assert_eq!(
        first("nuclear/decay.csv"),
        strato_data::decay::HEADER.join(",")
    );
    assert_eq!(
        first("nuclear/fission_yields.csv"),
        strato_data::yields::HEADER.join(",")
    );
    assert_eq!(
        first("temperature/temperature_histories.csv"),
        strato_data::temperature::HEADER.join(",")
    );
    assert_eq!(
        first("experimental/release_measurements.csv"),
        strato_data::release::HEADER.join(",")
    );
}

// ---- independent versus cumulative yield

fn takes_independent(_: &YieldValue<Independent>) {}
fn takes_cumulative(y: &YieldValue<Cumulative>) -> f64 {
    y.value.value
}

#[test]
fn independent_and_cumulative_yields_are_different_types() {
    let r = real_register();
    let cs = r.yields.get(Parent::U235, "Cs-137").unwrap();
    takes_independent(cs.jeff_independent.reported().unwrap());
    assert_eq!(
        takes_cumulative(cs.jeff_cumulative.reported().unwrap()),
        6.2208e-2
    );
    assert_eq!(
        cs.jeff_independent.reported().unwrap().value.value,
        7.2248e-4
    );
    // the ENDF cross-check is typed the same way
    match &cs.endf_cumulative {
        EndfYield::Reported(y) => assert!((y.value.value - 6.18832e-2).abs() < 1e-12),
        EndfYield::NotInCrossCheck => panic!("Cs-137 has an ENDF value"),
    }
}

#[test]
fn a_yield_the_library_leaves_blank_is_not_zero() {
    let r = real_register();
    let ag = r
        .yields
        .rows
        .iter()
        .find(|y| y.parent == Parent::U235 && y.product.name() == "Ag-110m")
        .unwrap();
    assert!(matches!(ag.jeff_cumulative, JeffYield::BlankInApi));
    assert_eq!(ag.status, Status::NotVerified);
}

// ---- direct fission product versus activation product

#[test]
fn only_direct_fission_products_give_a_source_term() {
    let r = real_register();
    for parent in [Parent::U235, Parent::Pu239] {
        let cs = r.yields.get(parent, "Cs-137").unwrap();
        assert!(cs.fission_source_yield().is_ok());
        assert!(r
            .yields
            .get(parent, "Sr-90")
            .unwrap()
            .fission_source_yield()
            .is_ok());
        for name in [
            "Ag-110m", "Cs-134", "Eu-154", "Ag-110", "Cs-134m", "Eu-154m",
        ] {
            let row = r.yields.get(parent, name).unwrap();
            assert_eq!(
                row.fission_source_yield(),
                Err(SourceTermRefusal::ActivationProduct),
                "{name} must be refused a fission source term"
            );
        }
    }
}

#[test]
fn activation_products_are_a_class_in_decay_csv_too() {
    let r = real_register();
    for n in ["Ag-110m", "Cs-134", "Eu-154"] {
        let row = r.decay.get(n).unwrap();
        assert!(row.product_class.is_activation_product(), "{n}");
        assert!(row.product_class.states_no_fission_yield_source, "{n}");
        assert!(!row.product_class.is_direct_fission_product(), "{n}");
    }
    for n in ["Cs-137", "Sr-90", "I-131", "Kr-85"] {
        assert!(
            r.decay
                .get(n)
                .unwrap()
                .product_class
                .is_direct_fission_product(),
            "{n}"
        );
    }
    assert!(matches!(
        r.decay.get("Y-90").unwrap().product_class.kind,
        ProductKind::DecayDaughter { parent: Some(ref p), .. } if p == "Sr-90"
    ));
}

#[test]
fn stable_nuclides_have_no_half_life_and_the_others_have_all_of_it() {
    let r = real_register();
    match &r.decay.get("Cs-133").unwrap().stability {
        Stability::Stable { lambda_per_s } => assert_eq!(lambda_per_s.value, 0.0),
        other => panic!("{other:?}"),
    }
    match &r.decay.get("Cs-137").unwrap().stability {
        Stability::Radioactive(x) => {
            assert_eq!(x.half_life.text, "30.08");
            assert_eq!(x.lambda_per_s.text, "7.302187e-10");
            assert!(x.branch_2.is_none());
        }
        other => panic!("{other:?}"),
    }
    match &r.decay.get("Ag-110m").unwrap().stability {
        Stability::Radioactive(x) => assert_eq!(x.branch_2.as_ref().unwrap().percent.text, "1.33"),
        other => panic!("{other:?}"),
    }
}

// ---- point value versus upper bound

#[test]
fn an_upper_bound_is_not_a_point_value() {
    let r = real_register();
    assert!(matches!(
        r.release.get("E-027").unwrap().value,
        ReleaseValue::UpperBound(_)
    ));
    assert!(matches!(
        r.release.get("E-001").unwrap().value,
        ReleaseValue::Point(_)
    ));
    assert!(matches!(
        r.release.get("E-010").unwrap().value,
        ReleaseValue::Digitised(_)
    ));
    assert!(matches!(
        r.release.get("E-031").unwrap().value,
        ReleaseValue::LowerBound(_)
    ));
    assert!(matches!(
        r.release.get("E-030").unwrap().value,
        ReleaseValue::Range { .. }
    ));
    assert!(matches!(
        r.release.get("E-114").unwrap().value,
        ReleaseValue::FailedOverTested { .. }
    ));
    let bounds: Vec<&str> = r
        .release
        .rows
        .iter()
        .filter(|x| x.value.is_bound())
        .map(|x| x.id.as_str())
        .collect();
    // 23 upper bounds and 1 lower bound
    assert_eq!(bounds.len(), 24);
    assert!(
        r.release
            .rows
            .iter()
            .filter(|x| matches!(x.value, ReleaseValue::UpperBound(_)))
            .count()
            == 23
    );
    // the value kinds partition the 600 rows
    let counts = r.release.rows.iter().fold([0usize; 6], |mut a, x| {
        let i = match x.value {
            ReleaseValue::Point(_) => 0,
            ReleaseValue::Digitised(_) => 1,
            ReleaseValue::UpperBound(_) => 2,
            ReleaseValue::LowerBound(_) => 3,
            ReleaseValue::Range { .. } => 4,
            ReleaseValue::FailedOverTested { .. } => 5,
        };
        a[i] += 1;
        a
    });
    assert_eq!(counts, [226, 341, 23, 1, 5, 3]);
}

#[test]
fn a_diffusivity_that_is_only_an_upper_bound_says_so() {
    let a = &real_register().arrhenius;
    match &a.get("A3-13").unwrap().law {
        Law::Arrhenius { d0_kind, d0, .. } => {
            assert_eq!(*d0_kind, BoundKind::UpperBound);
            assert_eq!(d0.text, "6.8e-12");
        }
        other => panic!("{other:?}"),
    }
    let d = a
        .get("A3-13")
        .unwrap()
        .diffusivity(1573.15, 8.3143, 0.0)
        .unwrap();
    assert_eq!(d.kind, BoundKind::UpperBound);
    assert_eq!(
        a.sum_of_terms(&["A3-13"], 1300.0, 8.3143, 0.0)
            .unwrap()
            .kind,
        BoundKind::UpperBound
    );
    assert_eq!(
        a.sum_of_terms(&["A3-05"], 1300.0, 8.3143, 0.0)
            .unwrap()
            .kind,
        BoundKind::PointValue
    );
    // an upper bound in the sum makes the sum an upper bound
    assert_eq!(
        a.sum_of_terms(&["A3-05", "A3-13"], 1300.0, 8.3143, 0.0)
            .unwrap()
            .kind,
        BoundKind::UpperBound
    );
}

#[test]
fn geometry_keeps_ranges_bounds_and_texts_apart() {
    let g = &real_register().geometry;
    let by = |id: &str| &g.rows.iter().find(|r| r.id == id).unwrap().value;
    assert!(matches!(by("G-01"), GeometryValue::Point(_)));
    assert!(matches!(by("G-09"), GeometryValue::UpperBound(_)));
    assert!(matches!(by("G-19"), GeometryValue::Text(t) if t == "64/16/20"));
    assert!(matches!(by("G-43"), GeometryValue::Range { .. }));
    assert!(matches!(by("G-54"), GeometryValue::PointWithRange { .. }));
}

// ---- evaluated data versus modelling assumption

#[test]
fn evaluated_data_and_modelling_conventions_are_different_classes() {
    let r = real_register();
    assert!(r
        .decay
        .rows
        .iter()
        .all(|x| x.evidence_class.is_evaluated_data()));
    assert!(r
        .yields
        .rows
        .iter()
        .all(|x| x.evidence_class.is_evaluated_data()));
    for id in ["BUF-01", "BUF-02", "C16-22", "C16-23", "H21-10"] {
        let row = r.arrhenius.get(id).unwrap();
        assert!(row.evidence_class.is_modelling_assumption(), "{id}");
        assert!(!row.evidence_class.is_evaluated_data(), "{id}");
    }
    assert_eq!(EvidenceClass::ALL.len(), 13);
    // the modelled safety-test schedule is a convention, not a measurement
    let th = r
        .temperature
        .rows
        .iter()
        .find(|x| x.id == "TH-008")
        .unwrap();
    assert!(th.evidence_class.is_modelling_assumption());
}

// ---- the register's own distinctions inside Arrhenius, temperature and release

#[test]
fn arrhenius_laws_fluence_and_reduced_coefficients_are_typed() {
    let a = &real_register().arrhenius;
    assert!(matches!(a.get("A4-08").unwrap().law, Law::Instantaneous));
    assert!(a
        .get("A4-08")
        .unwrap()
        .diffusivity(1873.15, 8.3143, 0.0)
        .is_none());
    assert!(matches!(
        a.get("A3-20").unwrap().law,
        Law::Constant { q_kj_mol: None, .. }
    ));
    assert!(matches!(
        a.get("BUF-01").unwrap().law,
        Law::Constant {
            q_kj_mol: Some(_),
            ..
        }
    ));
    assert_eq!(
        a.get("A3-01").unwrap().fluence,
        FluenceFactor::Documented { divisor: 5.0 }
    );
    assert_eq!(
        a.get("C16-08").unwrap().fluence,
        FluenceFactor::DocumentedNotUsed
    );
    assert_eq!(a.get("A3-02").unwrap().fluence, FluenceFactor::Absent);
    assert_eq!(
        a.get("T71-01")
            .unwrap()
            .reduced
            .as_ref()
            .unwrap()
            .d_prime_per_s
            .value,
        0.90
    );
    assert!(a.get("A1-01").unwrap().reduced.is_none());
    // the fluence factor multiplies only when a fluence is given
    let t = 1873.15;
    let d0 = a
        .get("A3-01")
        .unwrap()
        .diffusivity(t, 8.3143, 0.0)
        .unwrap()
        .m2_per_s;
    let d2 = a
        .get("A3-01")
        .unwrap()
        .diffusivity(t, 8.3143, 2.0)
        .unwrap()
        .m2_per_s;
    assert!((d2 / d0 - (2.0f64 / 5.0).exp()).abs() < 1e-12);
    // a two-term coefficient is the sum of its two rows
    let sum = a
        .sum_of_terms(&["A1-01", "A1-02"], 1600.0, 8.3143, 0.0)
        .unwrap()
        .m2_per_s;
    let one = a
        .get("A1-01")
        .unwrap()
        .diffusivity(t, 8.3143, 0.0)
        .unwrap()
        .m2_per_s
        + a.get("A1-02")
            .unwrap()
            .diffusivity(t, 8.3143, 0.0)
            .unwrap()
            .m2_per_s;
    assert_eq!(sum, one);
    assert_eq!(a.get("A1-01").unwrap().term, 1);
    assert_eq!(a.get("A1-02").unwrap().term, 2);
}

#[test]
fn symbolic_temperatures_are_not_numbers() {
    let t = &real_register().temperature;
    let row = t.rows.iter().find(|r| r.id == "TH-006").unwrap();
    assert!(matches!(
        row.t_start,
        Some(TemperaturePoint::TestTemperature(_))
    ));
    assert!(row.t_start.as_ref().unwrap().celsius().is_none());
    let hist = t.histories();
    assert_eq!(hist.len(), 62);
    assert_eq!(
        hist.iter()
            .find(|(h, _)| *h == "AGR1-ST-C16-1600")
            .unwrap()
            .1
            .len(),
        7
    );
}

#[test]
fn release_rows_keep_what_was_measured_and_at_what_temperature() {
    let r = &real_register().release;
    assert!(matches!(&r.get("E-028").unwrap().measured, Measured::Element(e) if e == "Cs"));
    assert!(matches!(
        &r.get("E-099").unwrap().measured,
        Measured::NotApplicable
    ));
    assert!(
        matches!(&r.get("E-010").unwrap().measured, Measured::Nuclide(n) if n.name() == "Cs-134")
    );
    assert!(matches!(
        r.get("E-060").unwrap().temperature,
        TestTemperature::Range { .. }
    ));
    assert!(matches!(
        r.get("E-061").unwrap().temperature,
        TestTemperature::Absent
    ));
    assert!(!r.get("E-001").unwrap().particle_applicability.is_empty());
}

#[test]
fn the_vocabularies_are_closed() {
    assert_eq!(Status::from_cell("verified"), Some(Status::Verified));
    assert_eq!(Status::from_cell("Verified"), None);
}

#[test]
fn row_ids_of_a_curve_may_mix_ranges_and_single_ids_in_curve_order() {
    use strato_data::curve::expand_row_ids;
    assert_eq!(
        expand_row_ids("E-616,E-637,E-617..E-619").unwrap(),
        ["E-616", "E-637", "E-617", "E-618", "E-619"]
    );
    assert_eq!(expand_row_ids("E-541..E-543").unwrap().len(), 3);
    assert!(expand_row_ids("E-620..E-617").is_err());
    assert!(expand_row_ids("E-616,X-1").is_err());
}

#[test]
fn the_hfr_k3_1_curves_are_sphere_curves_and_the_cs_curve_keeps_its_time_order() {
    use strato_data::curve::SpecimenLevel;
    let r = real_register();
    for id in ["IAEA97-K31-AG110M", "IAEA97-K31-CS137", "IAEA97-K31-SR90"] {
        let c = r.curves.get(id).unwrap();
        assert_eq!(c.specimen_level, SpecimenLevel::Sphere, "{id}");
    }
    let cs = r.curves.get("IAEA97-K31-CS137").unwrap();
    assert_eq!(cs.row_ids.len(), 22);
    assert_eq!(&cs.row_ids[..3], ["E-616", "E-637", "E-617"]);
    assert_eq!(r.curves.get("IAEA97-K31-SR90").unwrap().row_ids.len(), 20);
    // all 22 printed Ag-110m markers: 21 by the resolved interior, the 22nd (E-658) from its outline
    let ag = r.curves.get("IAEA97-K31-AG110M").unwrap();
    assert_eq!(ag.row_ids.len(), 22);
    assert_eq!(ag.row_ids.last().map(String::as_str), Some("E-658"));
}
