//! Tests of the experimental comparison on the real registers of the repository.

use std::path::{Path, PathBuf};

use strato::physics::arrhenius::{
    ArrheniusLaw, ArrheniusTerm, EnergyUnit, GasConstant, Temperature,
};
use strato_compare::compare::{run_rows, Eval};
use strato_compare::data::{
    default_set, evaluate, hold_of, nominal_hold, radii, Species, GEOMETRY_DESIGN,
};
use strato_compare::model::{Runner, Settings};
use strato_compare::pipeline::{generate, Options};
use strato_compare::status::{classify, Plan, Status};
use strato_data::release::Releases;
use strato_data::Register;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn register() -> Register {
    Register::load(&repo().join("data")).expect("the registers load")
}

#[test]
fn every_row_gets_one_status_and_no_row_falls_through() {
    let reg = register();
    let mut total = 0;
    for row in &reg.release.rows {
        let v = classify(&reg, row);
        assert_ne!(
            v.category, "no_rule",
            "row {} is covered by no rule: {}",
            row.id, v.reason
        );
        assert!(!v.reason.is_empty());
        total += 1;
    }
    assert_eq!(total, reg.release.rows.len());
    assert!(total >= 400, "the register has the extended row set");
}

#[test]
fn statuses_of_named_rows_follow_the_decisions() {
    let reg = register();
    let st = |id: &str| classify(&reg, reg.release.get(id).expect("row exists"));
    // A failed-SiC compact is not an intact particle.
    assert_eq!(st("E-001").status, Status::NotDirectlyComparable);
    // A count is not a release value.
    assert_eq!(st("E-099").status, Status::NotAssessed);
    // The matrix-corrected intact rows have no established denominator (decision 22).
    let e10 = st("E-010");
    assert_eq!(e10.status, Status::DataIncomplete);
    assert!(matches!(e10.plan, Plan::Provisional { .. }));
    // A reported upper bound is an inequality.
    let e27 = st("E-027");
    assert_eq!(e27.status, Status::ComparisonAvailable);
    assert!(matches!(e27.plan, Plan::Inequality { .. }));
    // A row labelled only "Cs" is compared as elemental caesium (decision 21).
    match st("E-028").plan {
        Plan::Inequality { spec, .. } => {
            assert!(spec.element_label_only);
            assert_eq!(spec.species, Species::Cs);
            assert_eq!(spec.t_end_h, 300.0, "row time at temperature");
        }
        other => panic!("expected an inequality, got {other:?}"),
    }
    // Totals of silver and strontium include inventory released in the reactor.
    assert_eq!(st("E-063").status, Status::NotDirectlyComparable);
    assert_eq!(st("E-172").status, Status::NotDirectlyComparable);
    // JAERI: denominator not established.
    assert_eq!(st("E-117").status, Status::DataIncomplete);
    // Loose particles with failed TRISO.
    assert_eq!(st("E-493").status, Status::NotDirectlyComparable);
}

#[test]
fn a_confirmed_denominator_turns_a_provisional_row_into_a_point_comparison() {
    // The program decides from the register cell: edit the cell of E-010 and the plan changes.
    let text = std::fs::read_to_string(
        repo()
            .join("data")
            .join("experimental")
            .join("release_measurements.csv"),
    )
    .expect("read the register");
    let mut changed = String::new();
    let mut edited = false;
    for line in text.lines() {
        if line.starts_with("E-010,") && line.contains("not established") {
            changed.push_str(&line.replacen("not established", "calculated compact inventory", 1));
            edited = true;
        } else {
            changed.push_str(line);
        }
        changed.push('\n');
    }
    assert!(edited, "E-010 has the denominator `not established`");
    let releases = Releases::parse(&changed).expect("the edited register parses");
    let mut reg = register();
    reg.release = releases;
    let v = classify(&reg, reg.release.get("E-010").expect("row"));
    assert_eq!(v.status, Status::ComparisonAvailable);
    match v.plan {
        Plan::Point { spec, value } => {
            assert_eq!(spec.window_start_h, 24.0);
            assert!((value - 8.5e-8).abs() < 1e-20);
        }
        other => panic!("expected a point plan, got {other:?}"),
    }
}

#[test]
fn registered_geometry_and_holds_are_read_from_the_registers() {
    let reg = register();
    let r = radii(&reg).expect("radii");
    // G-01: kernel diameter 350 um; thicknesses 103.5, 39.4, 35.3, 41.0 um (design AGR1-B-M16).
    let um = r.outer_m.map(|x| x * 1e6);
    let expect = [175.0, 278.5, 317.9, 353.2, 394.2];
    for k in 0..5 {
        // The radii are sums of a few printed decimals; a rounding error of 1e-9 um covers it.
        assert!((um[k] - expect[k]).abs() < 1e-9, "radius {k}: {}", um[k]);
    }
    assert_eq!(GEOMETRY_DESIGN, "AGR1-B-M16");
    let h = nominal_hold(&reg, 1600.0).expect("nominal 1600 hold");
    assert_eq!(h.hours, 300.0);
    let asrun = hold_of(&reg, "AGR1-ASRUN-6-4-3", 1600.0).expect("as-run hold");
    assert_eq!(asrun.hours, 400.0);
    assert!(hold_of(&reg, "AGR1-ASRUN-6-4-3", 1700.0).is_none());
}

#[test]
fn registered_coefficients_match_the_independent_property_functions() {
    // Cs in SiC at 1600 degC: the sum of the two published terms (C16-08 and C16-09), evaluated
    // by the data layer and by the property functions of the solver package.
    let reg = register();
    let set = default_set(&reg, Species::Cs).expect("set");
    let ev = evaluate(&reg, &set, 1600.0).expect("evaluate");
    let t1 = ArrheniusTerm::new(5.5e-14, 125.0, EnergyUnit::KilojoulePerMole, None).unwrap();
    let t2 = ArrheniusTerm::new(1.6e-2, 514.0, EnergyUnit::KilojoulePerMole, None).unwrap();
    let law = ArrheniusLaw::sum(vec![t1, t2]).unwrap();
    let d = law
        .evaluate(
            Temperature::Celsius(1600.0),
            GasConstant::custom(8.3145).unwrap(),
        )
        .unwrap()
        .d_m2_s;
    // Two evaluations of the same formula in different arithmetic order differ by a few units of
    // the rounding error of exp (about 1e-16 relative each); 1e-12 covers that with margin.
    assert!(
        ((ev.d_m2_s[3] - d) / d).abs() < 1e-12,
        "{} vs {}",
        ev.d_m2_s[3],
        d
    );
    // The two-term sum is larger than either term alone (heating-test runs keep both terms).
    assert!(ev.d_m2_s[3] > 5.5e-14 * (-125.0e3_f64 / (8.3145 * 1873.15)).exp());
    assert!(!ev.outside_printed_range);
}

#[test]
fn model_values_agree_with_the_independent_analytical_series() {
    // Ag at 1800 degC over 300 h: the release is large, the analytical series is usable and the
    // two methods share no code.
    let reg = register();
    let geometry = radii(&reg).unwrap();
    let mut runner = Runner::new(&reg, geometry.clone(), Settings::BASE).unwrap();
    let c = runner.get(Species::Ag, 1800.0, 300.0).unwrap().clone();
    let (an, err) =
        strato_compare::studies::analytical_reference(&geometry, &c.d_m2_s, &[100.0, 300.0]);
    assert!(err.is_none(), "{err:?}");
    for (k, h) in [100.0, 300.0].iter().enumerate() {
        let a = an[k].expect("series usable");
        let b = c.at(*h).unwrap();
        // The measured base-run difference is about 2e-4 (relative) at 100 h and 4e-5 at 300 h;
        // 1e-2 only separates a working solver from a wrong one and is not an accuracy claim.
        assert!(
            ((b - a) / a).abs() < 1e-2,
            "t = {h} h: BTCS {b}, series {a}"
        );
        // The refined run is closer than the base run (second order in space).
        let fine = c.at_refined(*h).unwrap();
        assert!((fine - a).abs() < (b - a).abs());
    }
}

#[test]
fn spatial_convergence_is_second_order_and_temporal_first_order() {
    let reg = register();
    let g = radii(&reg).unwrap();
    let sp = strato_compare::studies::spatial_convergence(&g, 3, 200).unwrap();
    // Halving the mesh divides the error by about 4 for a second-order scheme and by about 2 for
    // a first-order one; a ratio above 3 tells them apart with margin.
    for w in sp.windows(2) {
        assert!(w[0].error / w[1].error > 3.0, "{:?}", sp);
    }
    let tm = strato_compare::studies::temporal_convergence(&g, 3, 10, 2).unwrap();
    // Halving the step divides the error by about 2 (first order), well below the factor 4 of
    // second order: the ratio lies between 1.5 and 3.
    for w in tm.windows(2) {
        let q = w[0].error / w[1].error;
        assert!(q > 1.5 && q < 3.0, "ratio {q}");
    }
}

#[test]
fn bound_rows_are_inequalities_and_never_enter_point_aggregates() {
    let reg = register();
    let g = radii(&reg).unwrap();
    let mut runner = Runner::new(
        &reg,
        g,
        Settings {
            intervals: [20, 15, 10, 20, 10],
            dt_h: 0.5,
        },
    )
    .unwrap();
    let records = run_rows(&reg, &mut runner).unwrap();
    for r in &records {
        match &r.eval {
            Eval::Bound { margin, .. } => {
                let want = if margin.satisfied {
                    Status::WithinReportedUpperBound
                } else {
                    Status::ExceedsReportedUpperBound
                };
                assert_eq!(r.status, want, "row {}", r.row.id);
                assert!((margin.ratio <= 1.0) == margin.satisfied);
            }
            Eval::Provisional { .. } => {
                assert_eq!(r.status, Status::DataIncomplete, "row {}", r.row.id);
            }
            Eval::Point { .. } => assert_eq!(r.status, Status::ComparisonAvailable),
            Eval::None => {}
        }
        if r.model.is_none() {
            assert!(matches!(r.eval, Eval::None), "row {}", r.row.id);
        }
    }
    let sets = strato_compare::compare::datasets(&records);
    for d in &sets {
        let bounds = d.bound.map_or(0, |b| b.n);
        let in_status: usize = [
            Status::WithinReportedUpperBound,
            Status::ExceedsReportedUpperBound,
        ]
        .iter()
        .map(|s| d.counts.get(s).copied().unwrap_or(0))
        .sum();
        assert_eq!(bounds, in_status, "dataset {} {}", d.nuclide, d.quantity);
        if let Some(p) = d.point {
            assert!(
                d.counts
                    .get(&Status::ComparisonAvailable)
                    .copied()
                    .unwrap_or(0)
                    >= p.n
            );
        }
    }
}

#[test]
fn generated_files_carry_provenance_and_no_private_paths() {
    let dir = std::env::temp_dir().join(format!("strato-compare-test-{}", std::process::id()));
    let opts = Options {
        repo: repo(),
        data_out: dir.join("data"),
        figures_out: dir.join("fig"),
        report_out: Some(dir.join("report.md")),
        commit: "0123456789abcdef".to_string(),
        quick: true,
    };
    let out = generate(&opts).expect("the comparison runs");
    assert_eq!(out.records.len(), register().release.rows.len());
    for name in [
        "comparison_rows.csv",
        "comparison_datasets.csv",
        "release_curves.csv",
        "deviation_curves.csv",
        "radial_profiles.csv",
        "inventory_balance.csv",
        "temperature_sensitivity.csv",
        "convergence_spatial.csv",
        "convergence_temporal.csv",
    ] {
        let t = std::fs::read_to_string(opts.data_out.join(name)).expect(name);
        assert!(
            t.starts_with("# STRATO experimental comparison: "),
            "{name}"
        );
        assert!(t.contains("# commit: 0123456789abcdef"), "{name}");
        assert!(t.contains("# model_limitations: "), "{name}");
        assert!(t.contains("decay"), "{name}");
        assert!(t.contains("# release_measurements_sha256: "), "{name}");
        assert!(!t.contains(":\\"), "{name} holds a Windows path");
        assert!(!t.contains("/Users/"), "{name} holds a home path");
        assert!(!t.contains('@'), "{name} holds an at sign");
        // No cell is "NaN" or "inf".
        assert!(!t.contains("NaN") && !t.contains(",inf"), "{name}");
    }
    let j = std::fs::read_to_string(opts.data_out.join("comparison.json")).unwrap();
    assert!(j.contains("\"decay_and_generation_in_solver\": false"));
    // Decision 16: the class of the coefficient of every region of every element is stated.
    assert_eq!(j.matches("\"coefficient_class\"").count(), 15);
    // A digitised value with a stated reading uncertainty carries the interval it allows.
    assert!(j.contains("\"reading_interval\": ["));
    assert!(j.contains("Experimental Comparison: Data Interpretation and Limitations"));
    assert!(!j.contains(":\\\\"));
    let report = std::fs::read_to_string(opts.report_out.as_ref().unwrap()).unwrap();
    assert!(report.starts_with("# Experimental Comparison: Data Interpretation and Limitations\n"));
    assert!(report.contains("Nothing in this report calls the model validated"));
    for f in std::fs::read_dir(&opts.figures_out).unwrap() {
        let p = f.unwrap().path();
        let t = std::fs::read_to_string(&p).unwrap();
        assert!(
            t.starts_with("<svg") || t.contains("<svg"),
            "{}",
            p.display()
        );
        assert!(t.contains("0123456789abcdef"), "{}", p.display());
        assert!(!t.contains(":\\"), "{}", p.display());
    }
    // Decision 24: the nuclide figures exist, say what the markers are, and Cs-137 says so.
    let fig = |n: &str| std::fs::read_to_string(opts.figures_out.join(n)).expect(n);
    for n in [
        "figure_I_silver_110m_comparison.svg",
        "figure_J_strontium_90_comparison.svg",
    ] {
        let t = fig(n);
        assert!(t.contains("digitised"), "{n}");
        assert!(t.contains("reading uncertainty"), "{n}");
        assert!(t.contains("Numerical deviations of the model"), "{n}");
        assert!(t.contains("log-space root-mean-square"), "{n}");
    }
    assert!(fig("figure_K_caesium_137_data_unavailable.svg").contains("DATA UNAVAILABLE"));
    assert!(report.contains("DATA UNAVAILABLE"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn unit_typography_changes_whole_words_only() {
    use strato_compare::data::typography;
    assert_eq!(
        typography("a 250 um kernel radius at 1600 degC"),
        "a 250 \u{b5}m kernel radius at 1600 \u{b0}C"
    );
    assert_eq!(
        typography("column, umbrella, degCx"),
        "column, umbrella, degCx"
    );
    assert_eq!(typography("(um)"), "(\u{b5}m)");
}

#[test]
fn unit_typography_covers_the_celsius_letter_and_the_square() {
    use strato_compare::data::typography;
    assert_eq!(
        typography("factor about 1000 at 1600 to 1700 C and 100 at 1800 C"),
        "factor about 1000 at 1600 to 1700 \u{b0}C and 100 at 1800 \u{b0}C"
    );
    // A letter C that does not follow a number is not a unit.
    assert_eq!(
        typography("Table C and set C16-T2"),
        "Table C and set C16-T2"
    );
    assert_eq!(
        typography("(174.85 um / 250 um)^2 = 0.489"),
        "(174.85 \u{b5}m / 250 \u{b5}m)\u{b2} = 0.489"
    );
}

#[test]
fn bounds_and_context_are_never_drawn_as_exact_values_and_nothing_is_drawn_off_the_axis() {
    use strato_compare::figures::{drawable, glyph_of, marker_pass, placed_rows, Glyph, T_AXIS_H};
    let reg = register();
    let dir = std::env::temp_dir().join(format!("strato-compare-glyphs-{}", std::process::id()));
    let opts = Options {
        repo: repo(),
        data_out: dir.join("data"),
        figures_out: dir.join("fig"),
        report_out: None,
        commit: "0123456789abcdef".to_string(),
        quick: true,
    };
    let out = generate(&opts).expect("the comparison runs");
    let placed = placed_rows(&reg, &out.records);
    assert!(placed.iter().any(|p| p.is_bound));
    assert!(placed
        .iter()
        .any(|p| p.status == Status::NotDirectlyComparable));
    for p in &placed {
        let g = glyph_of(p);
        if p.is_bound {
            assert!(
                matches!(g, Glyph::BoundDown { .. } | Glyph::Cross),
                "{} is an upper bound but is drawn as {g:?}",
                p.id
            );
        }
        if p.status == Status::NotDirectlyComparable || p.status == Status::NotAssessed {
            assert!(matches!(g, Glyph::Cross), "{}: {g:?}", p.id);
            assert_eq!(marker_pass(p), 0);
        }
        if drawable(p) {
            assert!(p.t_h > 0.0 && p.t_h <= T_AXIS_H, "{} at {} h", p.id, p.t_h);
        }
    }
    // E-151 (CRP-6 fuel sphere, Sr, 1600 C, "<1e-5" at 500 h) is an upper bound beyond the axis:
    // it is not drawn, and in particular not moved onto the edge.
    let e151 = placed.iter().find(|p| p.id == "E-151").expect("E-151");
    assert!(e151.is_bound && e151.t_h > T_AXIS_H && !drawable(e151));
    // In the figures of the nuclides no marker is a hollow or filled circle: the legends of
    // figures I and J name no circle, so a circle would be an unexplained exact value.
    for f in [
        "figure_I_silver_110m_comparison.svg",
        "figure_J_strontium_90_comparison.svg",
    ] {
        let svg = std::fs::read_to_string(dir.join("fig").join(f)).expect("the figure exists");
        for piece in svg.split("<circle").skip(1) {
            let tag = piece.split("/>").next().unwrap_or("");
            assert!(!tag.contains("fill=\"none\""), "{f}: a hollow circle {tag}");
        }
        if f.contains("strontium") {
            // The footer lists the row that is not drawn.
            assert!(svg.contains("E-151"), "{f}");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_deviation_lines_of_a_nuclide_say_what_is_defined_and_what_is_not() {
    let reg = register();
    let dir = std::env::temp_dir().join(format!("strato-compare-lines-{}", std::process::id()));
    let opts = Options {
        repo: repo(),
        data_out: dir.join("data"),
        figures_out: dir.join("fig"),
        report_out: None,
        commit: "0123456789abcdef".to_string(),
        quick: true,
    };
    let out = generate(&opts).expect("the comparison runs");
    let _ = reg;
    for sp in [Species::Ag, Species::Sr] {
        let lines = strato_compare::figures::deviation_lines(&out.records, sp);
        let all = lines.join("\n");
        assert!(all.contains("digitised"), "{sp:?}");
        let has_point = out
            .records
            .iter()
            .any(|r| matches!(r.eval, Eval::Point { .. }));
        if !has_point {
            // No point comparison exists, so no aggregate may be printed.
            assert!(all.contains("are not defined"), "{sp:?}\n{all}");
            assert!(!all.contains("Over "), "{sp:?}");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_default_sets_are_the_selection_function_of_strato_data() {
    // Decision 47: the coefficient rows come from the selection function. The evaluation of the
    // set must equal the selection's own evaluation (same rows, same gas constant).
    use strato_data::select::{ParticleLayer, Phase, Selector, Variant};
    let reg = register();
    let sel = Selector {
        arrhenius: &reg.arrhenius,
        irradiation: &reg.irradiation,
    };
    let layers = [
        ParticleLayer::Kernel,
        ParticleLayer::Buffer,
        ParticleLayer::IPyC,
        ParticleLayer::SiC,
        ParticleLayer::OPyC,
    ];
    for sp in Species::ALL {
        let set = default_set(&reg, sp).unwrap();
        for t in [1600.0, 1700.0, 1800.0] {
            let ev = evaluate(&reg, &set, t).unwrap();
            for (k, layer) in layers.iter().enumerate() {
                let s = sel
                    .select(sp.label(), *layer, Phase::HeatingTest, Variant::Default)
                    .unwrap();
                let d = s.diffusivity_extrapolated(t).diffusivity.m2_per_s;
                // The same rows summed in the same order give the same number up to the rounding
                // of one addition and one exponential (about 1e-15 relative).
                assert!(
                    ((ev.d_m2_s[k] - d) / d).abs() < 1e-12,
                    "{sp:?} {layer:?} {t}: {} vs {d}",
                    ev.d_m2_s[k]
                );
            }
        }
    }
}

#[test]
fn the_analytical_series_exports_no_unresolved_or_negative_value() {
    // Cs at 1600 degC releases very little at 1 to 50 h: the series' own cancellation floor is
    // above the true value there, and those values must be left empty, never exported.
    let reg = register();
    let geometry = radii(&reg).unwrap();
    for sp in Species::ALL {
        let set = default_set(&reg, sp).unwrap();
        for t in [1600.0, 1700.0, 1800.0] {
            let ev = evaluate(&reg, &set, t).unwrap();
            let r = strato_compare::studies::analytical_series(
                &geometry,
                &ev.d_m2_s,
                &[1.0, 2.0, 5.0, 10.0, 50.0, 100.0, 400.0],
            );
            let floor = r.accuracy_floor.expect("the series is built");
            for v in r.values.iter().flatten() {
                assert!(*v > 0.0 && *v > floor, "{sp:?} {t}: {v} floor {floor}");
            }
        }
    }
}
