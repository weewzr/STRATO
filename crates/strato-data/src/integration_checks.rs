//! Checks of the data integration (family `DI`): the columns added by decision 44, the selection
//! of the published coefficient (decisions 39 to 41), and the irradiation files with the fission
//! rate derived from them (decision 42). These are new in Rust; the Python audit scripts have no
//! counterpart. Every expected number below is printed in one of the three research reports
//! (diffusivity sources section 4 and 5.3; AGR-1 irradiation section 5, steps 1 to 9; release data
//! sections 2, 12 and 13) or is a count stated in the commit that added the data.

use std::collections::BTreeMap;

use crate::arrhenius::{ArrheniusRow, CoefficientClass, PhaseOfUse, Species};
use crate::checks::Report;
use crate::irradiation::{
    BurnupQuantity, BurnupScope, Irradiation, SeriesTarget, CYCLES, DERIVED_LABEL,
};
use crate::register::Register;
use crate::release::{Denominator, Group, ReadingUncertainty, ReleaseRow, ValueClass};
use crate::select::{default_entries, ParticleLayer, Phase, Selector, Variant};

fn rel(a: f64, b: f64) -> f64 {
    (a / b - 1.0).abs()
}

fn items(r: &ArrheniusRow) -> Vec<String> {
    match &r.species {
        Species::Named { items, .. } => items.clone(),
        Species::AllSpecies | Species::AllMetallicFp => vec!["all".to_string()],
    }
}

fn prefix(id: &str) -> &str {
    id.split('-').next().unwrap_or(id)
}

/// Run the family `DI`.
pub fn integration_checks(rep: &mut Report, reg: &Register) {
    class_checks(rep, reg);
    selection_checks(rep, reg);
    release_checks(rep, reg);
    irradiation_checks(rep, &reg.irradiation);
}

// ------------------------------------------------------------ diffusivity columns

fn class_checks(rep: &mut Report, reg: &Register) {
    let rows = &reg.arrhenius.rows;
    // one class per row, by set
    let mut bad = Vec::new();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for r in rows {
        let expect = match prefix(&r.id) {
            "C15" | "C16" | "H21" => CoefficientClass::PublishedParfumeCorrelation,
            "S20" => CoefficientClass::Agr1SpecificFittedCorrelation,
            _ => CoefficientClass::TransferredMaterialCorrelation,
        };
        if r.coefficient_class != expect {
            bad.push(r.id.clone());
        }
        *counts.entry(r.coefficient_class_text()).or_default() += 1;
    }
    rep.add(
        "DI.class.one-per-row",
        bad.is_empty() && rows.iter().all(|r| r.coefficient_class != CoefficientClass::Unverified),
        "every row of arrhenius.csv has exactly one of the four classes of ruling 16, the class follows the set (TECDOC-978, BUF, E410, T71, E109: transferred; C15, C16, H21: PARFUME; S20: AGR-1 fitted), and no row is left unverified",
        format!("{counts:?}; wrong: {bad:?}"),
    );
    rep.add(
        "DI.class.no-measurement",
        rows.iter()
            .all(|r| r.coefficient_class != CoefficientClass::Agr1ExperimentalMeasurement),
        "no row is classed an AGR-1 experimental measurement: no AGR-1 diffusion coefficient was measured (decision 40)",
        "",
    );
    // phase
    let mut bad = Vec::new();
    for r in rows {
        let expect = match r.id.as_str() {
            "A1-05" => PhaseOfUse::Irradiation,
            "A1-06" | "A1-07" | "E410-01" | "A3-14" => PhaseOfUse::HeatingTest,
            id => match prefix(id) {
                "C15" | "H21" => PhaseOfUse::Irradiation,
                "C16" | "S20" => PhaseOfUse::HeatingTest,
                _ => PhaseOfUse::NotSpecified,
            },
        };
        if r.phase_of_use != expect {
            bad.push(r.id.clone());
        }
    }
    rep.add(
        "DI.phase.rules",
        bad.is_empty(),
        "phase_of_use: the irradiation-phase set (C15, H21, NOC row A1-05) is irradiation, the heating-test set (C16, S20, AC rows A1-06/07, the isothermal heating fits E410-01 and A3-14) is heating test, every other row names no phase",
        format!("wrong: {bad:?}"),
    );
    // reference radius
    let mut bad = Vec::new();
    for r in rows {
        let kernel_250 = r.id.starts_with("A1-")
            || prefix(&r.id) == "T71"
            || (matches!(prefix(&r.id), "C15" | "C16" | "H21")
                && r.layer == crate::arrhenius::DiffusionLayer::Kernel);
        let ok = match (&r.reference_radius_um, kernel_250) {
            (Some(p), true) => p.value == 250.0,
            (None, false) => true,
            _ => false,
        };
        if !ok {
            bad.push(r.id.clone());
        }
    }
    let n250 = rows
        .iter()
        .filter(|r| r.reference_radius_um.is_some())
        .count();
    rep.add(
        "DI.radius.kernel",
        bad.is_empty()
            && (crate::arrhenius::REDUCED_COEFFICIENT_KERNEL_RADIUS_M - 250e-6).abs() < 1e-15,
        "reference_radius_um is 250 on exactly the kernel rows that take their value from TECDOC-978 Table A-1 or Table 7.1 of TECDOC-1674 (A1, T71, and the kernel rows of C15, C16, H21) and empty elsewhere",
        format!("{n250} rows with a radius; wrong: {bad:?}"),
    );
    // fluence definition
    let mut with: Vec<&str> = rows
        .iter()
        .filter(|r| !r.fluence_definition.is_empty())
        .map(|r| r.id.as_str())
        .collect();
    with.sort_unstable();
    let documented_have_definition = rows.iter().all(|r| {
        matches!(r.fluence, crate::arrhenius::FluenceFactor::Absent)
            || !r.fluence_definition.is_empty()
    });
    rep.add(
        "DI.fluence.definition",
        with == ["A3-01", "C15-07", "C16-08", "E109-01", "H21-06"] && documented_have_definition,
        "fluence_definition is filled exactly on the five rows of Cs in SiC that carry or omit the exp(Gamma/5) factor (A3-01, E109-01, C15-07, C16-08, H21-06) and on every row that documents the factor",
        format!("{with:?}"),
    );
}

// --------------------------------------------------------------------- selection

fn selection_checks(rep: &mut Report, reg: &Register) {
    let sel = Selector {
        arrhenius: &reg.arrhenius,
        irradiation: &reg.irradiation,
    };
    let layer_of = |l: crate::arrhenius::DiffusionLayer| match l {
        crate::arrhenius::DiffusionLayer::Kernel => ParticleLayer::Kernel,
        crate::arrhenius::DiffusionLayer::Buffer => ParticleLayer::Buffer,
        crate::arrhenius::DiffusionLayer::SiC => ParticleLayer::SiC,
        _ => ParticleLayer::IPyC,
    };
    // every default entry resolves and matches the data
    let mut bad = Vec::new();
    let mut n = 0;
    for (el, layer, phase, ids) in default_entries() {
        n += 1;
        match sel.select(el, layer_of(layer), phase, Variant::Default) {
            Err(e) => bad.push(format!("{el} {layer:?} {phase:?}: {e}")),
            Ok(s) => {
                let ids_ok = s.row_ids() == ids;
                let same_set = s.rows.iter().all(|r| r.set_id == s.rows[0].set_id);
                let species_ok = s.rows.iter().all(|r| {
                    let it = items(r);
                    it.iter().any(|x| x == el || x == "all")
                });
                let layer_ok = s.rows.iter().all(|r| r.layer == layer);
                let terms_ok = s
                    .rows
                    .iter()
                    .enumerate()
                    .all(|(i, r)| usize::from(r.term) == i + 1);
                let class_ok = s.rows.iter().all(|r| r.coefficient_class == s.class);
                if !(ids_ok && same_set && species_ok && layer_ok && terms_ok && class_ok) {
                    bad.push(format!("{el} {layer:?} {phase:?}: {ids:?}"));
                }
            }
        }
    }
    rep.add(
        "DI.select.resolves",
        bad.is_empty() && n > 0,
        "every entry of the default table resolves to rows of one set whose nuclide, layer and term order agree with the file, with one class per selection",
        format!("{n} entries; bad: {bad:?}"),
    );

    // decision 39: irradiation single term, heating tests two terms where TECDOC-978 prints two
    let mut bad = Vec::new();
    let two_term_heating = [
        ("Cs", ParticleLayer::Kernel),
        ("Cs", ParticleLayer::SiC),
        ("Sr", ParticleLayer::SiC),
        ("Kr", ParticleLayer::Kernel),
        ("Kr", ParticleLayer::IPyC),
    ];
    for el in ["Ag", "Cs", "Sr", "Kr"] {
        for layer in [
            ParticleLayer::Kernel,
            ParticleLayer::IPyC,
            ParticleLayer::SiC,
        ] {
            if let Ok(s) = sel.select(el, layer, Phase::Irradiation, Variant::Default) {
                let exempt = el == "Kr" && layer != ParticleLayer::SiC; // A1-05 and the Kr PyC rows, labelled
                if s.terms() != 1 && !(exempt && s.terms() == 2) {
                    bad.push(format!(
                        "irradiation {el} {layer:?} has {} terms",
                        s.terms()
                    ));
                }
                if !exempt && prefix(&s.rows[0].id) != "C15" {
                    bad.push(format!("irradiation {el} {layer:?} uses {}", s.rows[0].id));
                }
            }
            if let Ok(s) = sel.select(el, layer, Phase::HeatingTest, Variant::Default) {
                let want = if two_term_heating.contains(&(el, layer)) {
                    2
                } else {
                    1
                };
                if s.terms() != want || prefix(&s.rows[0].id) != "C16" {
                    bad.push(format!(
                        "heating {el} {layer:?}: {} terms, {}",
                        s.terms(),
                        s.rows[0].id
                    ));
                }
            }
        }
    }
    rep.add(
        "DI.select.decision39",
        bad.is_empty(),
        "decision 39: irradiation-phase selections use the single-term C15 rows (Kr: the labelled A1-05 and C16 Kr PyC rows), heating-test selections use the C16 rows with both terms exactly for Cs and Kr in the kernel, Cs and Sr in SiC and Kr in PyC",
        format!("{bad:?}"),
    );

    // C15 equals C16 where both exist (single-term rows)
    let mut bad = Vec::new();
    for r15 in reg.arrhenius.rows.iter().filter(|r| prefix(&r.id) == "C15") {
        let c16 = reg.arrhenius.rows.iter().find(|r| {
            prefix(&r.id) == "C16"
                && r.nuclide_text() == r15.nuclide_text()
                && r.layer == r15.layer
                && r.term == 1
        });
        if let Some(c) = c16 {
            let (a0, aq) = (r15.d0_value(), r15.q_value());
            let (b0, bq) = (c.d0_value(), c.q_value());
            if a0 != b0 || aq != bq {
                bad.push(format!("{} vs {}", r15.id, c.id));
            }
        }
    }
    rep.add(
        "DI.select.c15-equals-c16",
        bad.is_empty(),
        "the first term of every heating-test row C16 equals the single term of the irradiation-phase row C15 for the same species and layer (D0 and Q identical), so the phases differ only by the second terms",
        format!("{bad:?}"),
    );

    // published numbers of the diffusivity report, section 4 (ratio of the sum to the first term)
    let ratio = |el: &str, layer: ParticleLayer, t: f64| -> Option<f64> {
        let s = sel
            .select(el, layer, Phase::HeatingTest, Variant::Default)
            .ok()?;
        let total = s.diffusivity_extrapolated(t).diffusivity.m2_per_s;
        let first = {
            let r = s.rows[0];
            let rg = match &r.gas_constant {
                crate::arrhenius::GasConstant::Printed(p) => p.value,
                crate::arrhenius::GasConstant::NotPrinted => 8.3145,
            };
            r.diffusivity(t + 273.15, rg, 0.0)?.m2_per_s
        };
        Some(total / first)
    };
    let cases = [
        ("Cs", ParticleLayer::Kernel, 1600.0, 1.503),
        ("Cs", ParticleLayer::Kernel, 1800.0, 2.297),
        ("Kr", ParticleLayer::Kernel, 1600.0, 91.1),
        ("Cs", ParticleLayer::SiC, 1500.0, 2.011),
        ("Cs", ParticleLayer::SiC, 1600.0, 5.134),
        ("Cs", ParticleLayer::SiC, 1800.0, 47.0),
        ("Sr", ParticleLayer::SiC, 1700.0, 1.461),
        ("Sr", ParticleLayer::SiC, 1800.0, 3.581),
        ("Kr", ParticleLayer::IPyC, 1800.0, 1.001),
    ];
    let mut worst = 0.0_f64;
    let mut bad = Vec::new();
    for (el, layer, t, want) in cases {
        match ratio(el, layer, t) {
            Some(got) => {
                // the report prints 3 to 4 significant figures
                let tol = if want < 10.0 { 1.1e-3 } else { 2.5e-3 };
                worst = worst.max(rel(got, want));
                if rel(got, want) > tol {
                    bad.push(format!("{el} {layer:?} {t}: {got} vs {want}"));
                }
            }
            None => bad.push(format!("{el} {layer:?} {t}: no value")),
        }
    }
    rep.add(
        "DI.select.second-term-need",
        bad.is_empty(),
        "section 4 of the diffusivity report: the ratio of the two-term coefficient to its first term (R = 8.3145, fluence 0) is reproduced from the file for Cs and Kr in the kernel, Cs and Sr in SiC and Kr in PyC",
        format!("worst relative difference {worst:.2e}; bad: {bad:?}"),
    );

    // refusal outside the printed range
    let kr_sic = sel.select(
        "Kr",
        ParticleLayer::SiC,
        Phase::HeatingTest,
        Variant::Default,
    );
    let cs_kernel = sel.select(
        "Cs",
        ParticleLayer::Kernel,
        Phase::HeatingTest,
        Variant::Default,
    );
    let ok = match (&kr_sic, &cs_kernel) {
        (Ok(k), Ok(c)) => {
            k.published_range_c == Some((1400.0, 1750.0))
                && k.diffusivity_at(1600.0).is_ok()
                && k.diffusivity_at(1800.0).is_err()
                && k.diffusivity_at(1399.0).is_err()
                && matches!(
                    k.diffusivity_extrapolated(1800.0).range,
                    crate::select::RangeStatus::Extrapolated { .. }
                )
                && c.published_range_c.is_none()
                && c.diffusivity_at(1800.0).is_ok()
        }
        _ => false,
    };
    rep.add(
        "DI.select.refuses-outside-range",
        ok,
        "a selection refuses a temperature outside the printed range of its rows (Kr in SiC, C16-20: 1400 to 1750 C, so 1800 C is refused and an extrapolation is marked) and gives a value anywhere for rows that print no range",
        "",
    );

    // alternative and sensitivity variants, unavailable data
    let mut bad = Vec::new();
    for (el, id) in [("Ag", "S20-01"), ("Cs", "S20-02"), ("Sr", "S20-03")] {
        match sel.select(
            el,
            ParticleLayer::SiC,
            Phase::HeatingTest,
            Variant::SchappelFitted,
        ) {
            Ok(s) => {
                if s.row_ids() != [id] || s.class != CoefficientClass::Agr1SpecificFittedCorrelation
                {
                    bad.push(format!("{el}: {:?}", s.row_ids()));
                }
            }
            Err(e) => bad.push(format!("{el}: {e}")),
        }
        if sel
            .select(
                el,
                ParticleLayer::SiC,
                Phase::Irradiation,
                Variant::SchappelFitted,
            )
            .is_ok()
        {
            bad.push(format!("{el}: the fitted set was offered for irradiation"));
        }
        // the default is never the fitted set
        if let Ok(d) = sel.select(el, ParticleLayer::SiC, Phase::HeatingTest, Variant::Default) {
            if d.class == CoefficientClass::Agr1SpecificFittedCorrelation {
                bad.push(format!("{el}: the fitted set is the default"));
            }
        }
    }
    rep.add(
        "DI.select.alternative",
        bad.is_empty(),
        "decision 40: the Schappel fitted rows S20-01 to S20-03 are offered only for heating-test runs of Ag, Cs and Sr in SiC, with their own class, and are never the default",
        format!("{bad:?}"),
    );
    let mut bad = Vec::new();
    for s in default_entries()
        .into_iter()
        .filter_map(|(el, l, p, _)| sel.select(el, layer_of(l), p, Variant::Default).ok())
    {
        let expect = if s.rows[0].id == "A1-05" {
            CoefficientClass::TransferredMaterialCorrelation
        } else {
            CoefficientClass::PublishedParfumeCorrelation
        };
        if s.class != expect {
            bad.push(s.rows[0].id.clone());
        }
    }
    rep.add(
        "DI.select.default-class",
        bad.is_empty(),
        "decision 40: every default selection has the class published PARFUME correlation, except the Kr kernel row A1-05 of the irradiation phase (transferred material correlation)",
        format!("{bad:?}"),
    );
    let un = [
        sel.select(
            "Eu",
            ParticleLayer::SiC,
            Phase::HeatingTest,
            Variant::Default,
        ),
        sel.select(
            "Kr",
            ParticleLayer::SiC,
            Phase::Irradiation,
            Variant::Default,
        ),
        sel.select(
            "Cs-137",
            ParticleLayer::Kernel,
            Phase::HeatingTest,
            Variant::SchappelFitted,
        )
        .and_then(|s| {
            if s.rows[0].set_id.starts_with("S20") {
                Ok(s)
            } else {
                Err(crate::select::Unavailable {
                    reason: "default rows".into(),
                })
            }
        }),
    ];
    rep.add(
        "DI.select.unavailable",
        un.iter().all(|r| r.is_err()),
        "no coefficient is invented: Eu has none, Kr in SiC is not used during irradiation, and the Schappel set offers nothing for the kernel",
        "",
    );
    // kernel scaling (decision 41)
    let scaled = sel.select(
        "Cs",
        ParticleLayer::Kernel,
        Phase::HeatingTest,
        Variant::KernelScaledSensitivity,
    );
    let plain = sel.select(
        "Cs",
        ParticleLayer::Kernel,
        Phase::HeatingTest,
        Variant::Default,
    );
    let ok = match (&scaled, &plain) {
        (Ok(a), Ok(b)) => {
            let k = a.kernel_scale.expect("scale");
            let da = a.diffusivity_at(1600.0).unwrap().m2_per_s;
            let db = b.diffusivity_at(1600.0).unwrap().m2_per_s;
            (k.factor - (174.85_f64 / 250.0).powi(2)).abs() < 1e-12
                && (k.factor - 0.489).abs() < 5e-4
                && rel(da / db, k.factor) < 1e-12
                && b.kernel_scale.is_none()
        }
        _ => false,
    };
    rep.add(
        "DI.select.kernel-scale",
        ok,
        "decision 41: the sensitivity case multiplies the kernel coefficient by (174.85 um / 250 um)^2 = 0.4892 (decision text: 0.489), computed from the kernel diameter of Collin 2015 Table A-1; the default is unscaled",
        "",
    );
}

// ------------------------------------------------------------------- release file

fn release_checks(rep: &mut Report, reg: &Register) {
    let rows = &reg.release.rows;
    let digitised = |r: &ReleaseRow| {
        matches!(
            r.value_class,
            ValueClass::Digitised | ValueClass::VisualReading
        )
    };
    let bad: Vec<&str> = rows
        .iter()
        .filter(|r| {
            let nd = r.reading_uncertainty == ReadingUncertainty::NotDigitised;
            nd == digitised(r)
        })
        .map(|r| r.id.as_str())
        .collect();
    let n_rec = rows
        .iter()
        .filter(|r| r.reading_uncertainty == ReadingUncertainty::NotRecorded)
        .count();
    let n_stated = rows
        .iter()
        .filter(|r| matches!(r.reading_uncertainty, ReadingUncertainty::Stated { .. }))
        .count();
    rep.add(
        "DI.release.reading-uncertainty",
        bad.is_empty() && n_stated == 148 && n_rec == 71,
        "reading_uncertainty is empty exactly for the rows that were not read from a figure; each of the 219 digitised rows states it (148) or says not recorded (71)",
        format!("{n_stated} stated, {n_rec} not recorded; wrong: {bad:?}"),
    );
    let nd: Vec<&str> = rows
        .iter()
        .filter(|r| r.value_class == ValueClass::NonDetect)
        .map(|r| r.id.as_str())
        .collect();
    let all_bounds = rows
        .iter()
        .filter(|r| r.value_class == ValueClass::NonDetect)
        .all(|r| matches!(r.value, crate::release::ReleaseValue::UpperBound(_)));
    rep.add(
        "DI.release.non-detect",
        nd == ["E-217", "E-220", "E-221", "E-225", "E-228", "E-231"] && all_bounds,
        "the non-detect class M-ND is applied to exactly the six Kr-85 N/D rows named by the release report, each an upper bound",
        format!("{nd:?}"),
    );
    let g: Vec<&str> = rows
        .iter()
        .filter(|r| r.group == Group::NoCountedFailureReleasingParticle)
        .map(|r| r.id.as_str())
        .collect();
    rep.add(
        "DI.release.group-label",
        g == ["E-406", "E-417"],
        "the group label for no counted failure but a particle that released fission products is applied to exactly E-406 and E-417 (AGR-2 compact 2-3-2)",
        format!("{g:?}"),
    );
    let ids: std::collections::HashSet<u32> = rows
        .iter()
        .filter_map(|r| r.id.strip_prefix("E-").and_then(|n| n.parse().ok()))
        .collect();
    let withdrawn_absent = (316..=373).all(|n| !ids.contains(&n));
    rep.add(
        "DI.release.withdrawn-ids",
        withdrawn_absent && ids.contains(&315) && ids.contains(&374) && ids.len() == rows.len(),
        "the withdrawn ids E-316 to E-373 stay as gaps and are never reused",
        format!("{} rows", rows.len()),
    );
    let mut bad = Vec::new();
    for r in rows {
        let n: u32 = r.id[2..].parse().unwrap_or(0);
        let want = match n {
            157..=267 | 374..=492 => Some(Denominator::CompactInventory),
            268..=315 => Some(Denominator::CapsuleInventory),
            493..=540 => Some(Denominator::ParticleInventory),
            _ => None,
        };
        if let Some(w) = want {
            if r.denominator != w {
                bad.push(r.id.clone());
            }
        }
        if r.denominator == Denominator::ParticleInventory
            && r.unit != crate::release::ReleaseUnit::Particles
        {
            bad.push(r.id.clone());
        }
        if r.denominator == Denominator::CompactInventory
            && r.unit != crate::release::ReleaseUnit::Fraction
        {
            bad.push(r.id.clone());
        }
    }
    let ne = rows
        .iter()
        .filter(|r| r.denominator == Denominator::NotEstablished)
        .count();
    rep.add(
        "DI.release.denominator",
        bad.is_empty() && ne == 61,
        "denominator: compact inventory for the tabulated and digitised compact fractions, capsule inventory for the mass-balance rows, average particle inventory for the particle equivalents; 61 rows (JAERI, CRP-6, matrix-corrected and a few text rows) say not established and stay DATA INCOMPLETE",
        format!("{ne} not established; wrong: {bad:?}"),
    );
}

// -------------------------------------------------------------------- irradiation

/// Step 9 table of the AGR-1 irradiation report: capsule-average fission-rate density per cycle,
/// in 1e19 fissions m-3 s-1, columns capsule 6, 5, 4, 3, 2, 1.
#[allow(clippy::approx_constant)]
const STEP9: [[f64; 6]; 13] = [
    [3.98, 4.35, 4.85, 5.09, 4.72, 4.53],
    [4.43, 5.10, 6.28, 6.73, 5.83, 5.22],
    [4.98, 6.00, 7.53, 8.04, 7.02, 6.06],
    [5.35, 6.72, 8.47, 9.09, 7.84, 6.47],
    [5.27, 6.89, 8.59, 9.16, 8.19, 6.65],
    [6.61, 9.02, 11.35, 11.88, 10.81, 8.31],
    [6.69, 8.99, 10.79, 11.10, 10.43, 8.08],
    [6.90, 9.52, 10.63, 10.80, 10.52, 8.35],
    [7.70, 10.12, 10.54, 10.48, 10.54, 8.82],
    [7.02, 8.74, 8.84, 8.74, 8.99, 7.83],
    [7.48, 8.88, 8.74, 8.61, 8.81, 8.08],
    [7.78, 8.73, 8.34, 8.17, 8.51, 8.06],
    [6.61, 7.30, 6.88, 6.67, 6.99, 6.83],
];

fn irradiation_checks(rep: &mut Report, irr: &Irradiation) {
    // completeness
    let mut keys: BTreeMap<(String, u8, bool), usize> = BTreeMap::new();
    for r in &irr.burnup {
        let who = match r.scope {
            BurnupScope::Compact => r.compact.clone().unwrap_or_default(),
            BurnupScope::Capsule => format!("capsule {}", r.capsule),
        };
        *keys
            .entry((who, r.cycle, r.quantity == BurnupQuantity::Burnup))
            .or_default() += 1;
    }
    let compacts = irr
        .burnup
        .iter()
        .filter(|r| r.scope == BurnupScope::Compact)
        .filter_map(|r| r.compact.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let complete = keys.len() == 78 * 13 * 2
        && keys.values().all(|n| *n == 1)
        && compacts.len() == 72
        && irr.burnup.len() == 2028;
    rep.add(
        "DI.irr.completeness",
        complete,
        "agr1_burnup_by_cycle.csv holds burnup and fluence for 72 compacts and 6 capsule averages after each of the 13 cycles, once each (2028 rows)",
        format!("{} rows, {} compacts", irr.burnup.len(), compacts.len()),
    );
    // monotone and consistent with cycle lengths
    let mut bad = Vec::new();
    let mut by: BTreeMap<(String, bool), Vec<(u8, f64)>> = BTreeMap::new();
    for r in &irr.burnup {
        let who = match r.scope {
            BurnupScope::Compact => r.compact.clone().unwrap_or_default(),
            BurnupScope::Capsule => format!("capsule {}", r.capsule),
        };
        by.entry((who, r.quantity == BurnupQuantity::Burnup))
            .or_default()
            .push((r.cycle, r.value.value));
    }
    for ((who, _), v) in &mut by {
        v.sort_by_key(|x| x.0);
        if v.windows(2).any(|w| w[1].1 < w[0].1) {
            bad.push(who.clone());
        }
    }
    let mut efpd_bad = Vec::new();
    let mut cum = 0.0;
    let mut ends: BTreeMap<u8, f64> = BTreeMap::new();
    for n in 1..=CYCLES {
        if let Ok((len, _)) = irr.cycle_efpd(n) {
            cum += len;
            ends.insert(n, cum);
        }
    }
    for r in &irr.burnup {
        match ends.get(&r.cycle) {
            Some(e) if (e - r.cycle_end_efpd.value).abs() < 0.051 => {}
            _ => efpd_bad.push(r.id.clone()),
        }
    }
    rep.add(
        "DI.irr.monotone",
        bad.is_empty() && efpd_bad.is_empty() && ends.len() == 13,
        "cumulative burnup and fluence never decrease from cycle to cycle, and the cycle end in effective full-power days of every row equals the running sum of the 13 cycle lengths of Table 2 to within the printed 0.1 EFPD",
        format!("decreasing: {bad:?}; cycle-end mismatches: {}", efpd_bad.len()),
    );
    // cycle 13 against the end-of-irradiation tables
    let mut lo = f64::MAX;
    let mut hi = f64::MIN;
    let mut missing = 0;
    for r in irr.burnup.iter().filter(|r| {
        r.cycle == 13 && r.scope == BurnupScope::Compact && r.quantity == BurnupQuantity::Burnup
    }) {
        let name = r.compact.clone().unwrap_or_default();
        let end = irr.conditions.iter().find(|c| {
            c.quantity == "compact burnup at end of irradiation"
                && c.compact.as_deref() == Some(&name)
        });
        match end.map(|c| &c.value) {
            Some(crate::irradiation::ConditionValue::Number(p)) => {
                let d = r.value.value - p.value;
                lo = lo.min(d);
                hi = hi.max(d);
            }
            _ => missing += 1,
        }
    }
    rep.add(
        "DI.irr.rev3-difference",
        missing == 0 && lo > -0.0101 && hi < 0.0501 && lo < -0.0099 && hi > 0.0499,
        "Appendix D after cycle 13 minus the end-of-irradiation Tables 3 and 4 lies between -0.01 and +0.05 percent FIMA for the 72 compacts (the Rev. 3 difference of the report, reason not established)",
        format!("{lo:.2} to {hi:.2}"),
    );
    // power windows
    let mut ok = irr.power.len() == 744;
    for c in 1..=6u8 {
        let mut w: Vec<f64> = irr
            .power
            .iter()
            .filter(|p| p.capsule == c)
            .map(|p| p.efpd_window_centre.value)
            .collect();
        w.sort_by(f64::total_cmp);
        ok &= w.len() == 124
            && w.iter()
                .enumerate()
                .all(|(i, x)| *x == 5.0 * (i as f64 + 1.0));
    }
    rep.add(
        "DI.irr.power-windows",
        ok,
        "the digitised heat history has 124 five-EFPD windows (centres 5 to 620) for each of the 6 capsules, all with status digitised",
        "",
    );
    // inventory (steps 1 to 4)
    match irr.kernel_inventory() {
        Err(e) => rep.add("DI.irr.inventory", false, "kernel inventory", e.to_string()),
        Ok(k) => {
            let ok = rel(k.atoms_per_compact, 2.32627e21) < 2e-5
                && rel(k.atoms_per_compact / k.particles_per_compact, 5.6001e17) < 2e-5
                && rel(k.kernel_volume_m3, 2.23916e-11) < 2e-5
                && rel(k.atoms_per_m3, 2.50097e28) < 2e-5;
            rep.add(
                "DI.irr.inventory",
                ok,
                "steps 1 to 4 of the irradiation report: 2.32627e21 heavy-metal atoms per compact, 5.6001e17 per particle, kernel volume 2.23916e-11 m3, 2.50097e28 atoms per m3 of kernel (relative 2e-5)",
                format!("{:.5e} atoms per m3", k.atoms_per_m3),
            );
        }
    }
    // step 8: time averages from end-of-irradiation burnup
    let t = irr.cycle_efpd(1).map(|_| ()).is_ok();
    let total_efpd = irr.condition("cumulative burnup of the AGR-1 test train at end of irradiation (cumulative EFPD, Table 2)");
    let mut bad = Vec::new();
    if let (true, Ok(row)) = (t, total_efpd) {
        let efpd = match &row.value {
            crate::irradiation::ConditionValue::Number(p) => p.value,
            _ => 0.0,
        };
        for (bu, want) in [
            (11.30, 5.27e19),
            (13.36, 6.24e19),
            (15.33, 7.15e19),
            (16.50, 7.70e19),
            (17.84, 8.33e19),
            (18.23, 8.51e19),
            (18.64, 8.70e19),
            (19.56, 9.13e19),
        ] {
            let got = irr.time_average_from_end_burnup(bu, efpd).unwrap_or(0.0);
            if rel(got, want) > 1.2e-3 {
                bad.push(format!("{bu}: {got:.3e} vs {want:.3e}"));
            }
        }
        if efpd != 620.2 {
            bad.push(format!("irradiation length {efpd}"));
        }
    } else {
        bad.push("no irradiation length".into());
    }
    rep.add(
        "DI.irr.step8",
        bad.is_empty(),
        "step 8 of the irradiation report: the time-average fission-rate density over 620.2 EFPD for the eight burnups of the table (5.27e19 to 9.13e19 fissions per m3 per s), to the three figures printed",
        format!("{bad:?}"),
    );
    // step 9
    let mut bad = Vec::new();
    let mut worst = 0.0_f64;
    for (col, capsule) in [6u8, 5, 4, 3, 2, 1].into_iter().enumerate() {
        match irr.derive_series(SeriesTarget::Capsule(capsule)) {
            Err(e) => bad.push(e.to_string()),
            Ok(s) => {
                if s.label != DERIVED_LABEL || s.cycles.len() != 13 {
                    bad.push(format!("capsule {capsule}: label or length"));
                }
                for (i, c) in s.cycles.iter().enumerate() {
                    let want = STEP9[i][col] * 1e19;
                    let d = (c.fissions_per_m3_per_s - want).abs() / want;
                    worst = worst.max(d);
                    // the report prints three significant figures (half a unit of the last digit)
                    let tol = 0.005 / STEP9[i][col] * 1.0001;
                    if d > tol {
                        bad.push(format!(
                            "capsule {capsule} cycle {}: {:.4e} vs {:.3e}",
                            i + 1,
                            c.fissions_per_m3_per_s,
                            want
                        ));
                    }
                }
            }
        }
    }
    rep.add(
        "DI.irr.step9",
        bad.is_empty(),
        "step 9 of the irradiation report: all 78 capsule-average cycle values of the derived fission-rate density equal the printed table to its printed digits",
        format!("worst relative difference {worst:.2e}; {bad:?}"),
    );
    // weighted mean, extremes and label
    let c3 = irr
        .derive_series(SeriesTarget::Capsule(3))
        .map(|s| s.time_average());
    let c6 = irr
        .derive_series(SeriesTarget::Capsule(6))
        .map(|s| s.time_average());
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for name in &compacts {
        if let Ok(s) = irr.derive_series(SeriesTarget::Compact(name.clone())) {
            for c in &s.cycles {
                lo = lo.min(c.fissions_per_m3_per_s);
                hi = hi.max(c.fissions_per_m3_per_s);
            }
        }
    }
    let ok = matches!((&c3, &c6), (Ok(a), Ok(b)) if rel(*a, 8.709e19) < 5e-4 && rel(*b, 6.240e19) < 5e-4)
        && rel(lo, 2.485e19) < 1e-3
        && rel(hi, 1.269e20) < 1e-3
        && DERIVED_LABEL == "derived from published AGR-1 burnup (calculated)";
    rep.add(
        "DI.irr.derived-series",
        ok,
        "the cycle-length-weighted mean of the derived series is 8.709e19 (capsule 3) and 6.240e19 (capsule 6), the extremes over the 72 compacts and 13 cycles are 2.485e19 and 1.269e20, and the series is labelled derived from published AGR-1 burnup (calculated)",
        format!("min {lo:.4e}, max {hi:.4e}"),
    );
}
