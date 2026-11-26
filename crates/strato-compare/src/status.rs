//! The status of every row of the release register, decided from the row itself and from the
//! decisions of the coordinator (decisions 15 to 24 of `stage-b/btcs/DECISIONS.md`).
//!
//! The rules run in a fixed order and the first one that applies decides. A status is never
//! computed from a deviation: it comes from what the row measures, what its denominator is and
//! whether the registered model can represent it. Every verdict carries a reason in words and a
//! category code, so a reader can see why a row is where it is.

use strato_data::release::{
    Denominator, Group, Measured, Program, Quantity, ReleaseRow, ReleaseValue, TestTemperature,
};
use strato_data::Register;

use crate::data::{hold_of, nominal_hold, Species};

/// The comparison statuses: the six of directive section 15 and the two of decision 23.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Status {
    /// The row is not a release value, or needs a mechanism the solver does not yet have.
    NotAssessed,
    /// A denominator, a test sequence or a coefficient set is missing from the register.
    DataIncomplete,
    /// The row measures something the model quantity is not.
    NotDirectlyComparable,
    /// A model value exists to be compared with the row.
    ComparisonAvailable,
    /// A point value lies within an uncertainty the source reports.
    WithinReportedUncertainty,
    /// A point value lies outside an uncertainty the source reports.
    OutsideReportedUncertainty,
    /// The model satisfies a reported upper bound (decision 23).
    WithinReportedUpperBound,
    /// The model exceeds a reported upper bound (decision 23); the excess stays visible.
    ExceedsReportedUpperBound,
}

impl Status {
    /// The label shown to the reader.
    pub fn label(self) -> &'static str {
        match self {
            Status::NotAssessed => "NOT ASSESSED",
            Status::DataIncomplete => "DATA INCOMPLETE",
            Status::NotDirectlyComparable => "NOT DIRECTLY COMPARABLE",
            Status::ComparisonAvailable => "COMPARISON AVAILABLE",
            Status::WithinReportedUncertainty => "WITHIN REPORTED UNCERTAINTY",
            Status::OutsideReportedUncertainty => "OUTSIDE REPORTED UNCERTAINTY",
            Status::WithinReportedUpperBound => "WITHIN REPORTED UPPER BOUND",
            Status::ExceedsReportedUpperBound => "EXCEEDS REPORTED UPPER BOUND",
        }
    }

    /// All labels in a fixed order.
    pub const ALL: [Status; 8] = [
        Status::NotAssessed,
        Status::DataIncomplete,
        Status::NotDirectlyComparable,
        Status::ComparisonAvailable,
        Status::WithinReportedUncertainty,
        Status::OutsideReportedUncertainty,
        Status::WithinReportedUpperBound,
        Status::ExceedsReportedUpperBound,
    ];
}

/// How a measured number is read as a bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundSource {
    /// The source states the number as an upper bound.
    ReportedUpperBound,
    /// A digitised total that includes matrix and background, read as an upper bound (decision 19).
    DigitisedTotal,
    /// A printed total that includes matrix and background, read as an upper bound on the same
    /// grounds as a digitised total.
    PrintedTotal,
    /// A matrix-corrected fraction of an intact compact over a window: an upper bound on the
    /// release through intact coatings in that window only under premise BP1, which is the
    /// source's assumption (decisions 67 and 68).
    WindowUnderBp1,
}

/// The class of the ten intact matrix-corrected rows (decision 68(a)).
pub const WINDOW_BOUND_CLASS: &str = "window upper bound under premise BP1";

/// The denominator of those rows as the comparison states it (decision 68(a)): the register cell
/// says "calculated compact inventory"; the source calculates it for the as-run irradiation.
pub const WINDOW_BOUND_DENOMINATOR: &str = "as-run calculated compact inventory";

/// The end of the 300 h plateau of the safety tests, hours at temperature (Collin et al. 2016,
/// manuscript p. 4: the experimental fractions were determined at the end of the 300 h plateaus).
pub const PLATEAU_H: f64 = 300.0;

/// Start of the window of a matrix-corrected value, hours at temperature: the release during
/// the first 24 h at temperature is not counted by the source's correction.
pub const WINDOW_START_H: f64 = 24.0;

impl BoundSource {
    /// Text for the files.
    pub fn text(self) -> &'static str {
        match self {
            BoundSource::ReportedUpperBound => "reported upper bound",
            BoundSource::DigitisedTotal => "digitised total read as an upper bound",
            BoundSource::PrintedTotal => "printed total read as an upper bound",
            BoundSource::WindowUnderBp1 => {
                "window upper bound under premise BP1 (the premise is the source's assumption, not an established fact)"
            }
        }
    }
}

/// Which model run answers a row.
#[derive(Debug, Clone, PartialEq)]
pub struct RunSpec {
    /// Element whose registered coefficients are used.
    pub species: Species,
    /// True when the row names the element only (decision 21).
    pub element_label_only: bool,
    /// Test temperature, degC.
    pub t_c: f64,
    /// Start of the compared window, hours at temperature (0 for a cumulative value).
    pub window_start_h: f64,
    /// End of the compared window, hours at temperature.
    pub t_end_h: f64,
    /// Where the end time comes from.
    pub time_source: String,
}

/// What the model has to do for a row.
#[derive(Debug, Clone, PartialEq)]
pub enum Plan {
    /// No model run: the row is not compared.
    NoModel,
    /// An inequality: the model release must not exceed the number.
    Inequality {
        /// The run.
        spec: RunSpec,
        /// The bound.
        limit: f64,
        /// How the number came to be a bound.
        source: BoundSource,
    },
    /// A point comparison with an established denominator.
    Point {
        /// The run.
        spec: RunSpec,
        /// The measured value.
        value: f64,
    },
    /// A model value shown for scale only: the denominator is not established, so no status of
    /// comparison and no aggregate is built from it.
    Provisional {
        /// The run.
        spec: RunSpec,
        /// The measured value.
        value: f64,
    },
}

/// The verdict of the rules for one row.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    /// The status before the model has run (a bound row becomes WITHIN or EXCEEDS afterwards).
    pub status: Status,
    /// Short code of the rule that decided.
    pub category: &'static str,
    /// The reason in words.
    pub reason: String,
    /// The model work to do.
    pub plan: Plan,
}

fn verdict(status: Status, category: &'static str, reason: impl Into<String>) -> Verdict {
    Verdict {
        status,
        category,
        reason: reason.into(),
        plan: Plan::NoModel,
    }
}

/// The measured number of a row when it is a single number.
pub fn single_value(row: &ReleaseRow) -> Option<f64> {
    match &row.value {
        ReleaseValue::Point(p)
        | ReleaseValue::Digitised(p)
        | ReleaseValue::UpperBound(p)
        | ReleaseValue::LowerBound(p) => Some(p.value),
        _ => None,
    }
}

/// The nuclide or element text of a row.
pub fn measured_text(row: &ReleaseRow) -> String {
    match &row.measured {
        Measured::Nuclide(n) => n.name(),
        Measured::Element(e) => e.clone(),
        Measured::NotApplicable => "n/a".to_string(),
    }
}

fn element_of(row: &ReleaseRow) -> Option<(String, bool)> {
    match &row.measured {
        Measured::Nuclide(n) => Some((n.element.clone(), false)),
        Measured::Element(e) => Some((e.clone(), true)),
        Measured::NotApplicable => None,
    }
}

/// The end of the test in hours at temperature: the time the row states, else the hold of the
/// as-run history of the compact when the register has one at this temperature, else the nominal
/// hold of the modelled safety test (300 h).
pub fn end_time(reg: &Register, row: &ReleaseRow, t_c: f64) -> Option<(f64, String)> {
    if let Some(t) = &row.time_h {
        return Some((t.value, format!("time_h of row {}", row.id)));
    }
    if let Some(h) = hold_of(reg, &format!("AGR1-ASRUN-{}", row.compact_or_sample), t_c) {
        return Some((h.hours, format!("as-run hold, {}", h.source)));
    }
    nominal_hold(reg, t_c).map(|h| (h.hours, format!("nominal hold, {}", h.source)))
}

/// The end of the window of a matrix-corrected value: the end of the 300 h plateau, or the end of
/// the test where the register gives an as-run hold at this temperature that is shorter
/// (decisions 67 and 68). Where the register holds no as-run duration of the compact at this
/// temperature, the plateau end is used and the origin text says so.
pub fn window_end(reg: &Register, row: &ReleaseRow, t_c: f64) -> (f64, String) {
    match hold_of(reg, &format!("AGR1-ASRUN-{}", row.compact_or_sample), t_c) {
        Some(h) if h.hours < PLATEAU_H => (
            h.hours,
            format!(
                "end of the test: the as-run hold of {} h ({}) is shorter than the {} h plateau",
                h.hours, h.source, PLATEAU_H
            ),
        ),
        Some(h) => (
            PLATEAU_H,
            format!(
                "end of the {} h plateau (Collin et al. 2016, manuscript p. 4); the as-run hold of {} h ({}) is longer and is not the window end",
                PLATEAU_H, h.hours, h.source
            ),
        ),
        None => (
            PLATEAU_H,
            format!(
                "end of the {} h plateau (Collin et al. 2016, manuscript p. 4); the register holds no as-run duration of compact {} at {} degC, so the plateau end is used and the test may have been shorter",
                PLATEAU_H, row.compact_or_sample, t_c
            ),
        ),
    }
}

/// Decide the status and the plan of one row.
pub fn classify(reg: &Register, row: &ReleaseRow) -> Verdict {
    use Quantity as Q;

    // 1. Counts, constants and thresholds are not release values.
    match row.quantity {
        Q::ParticlesWithFailedIpycAndSic
        | Q::ParticlesWithFailedSicOverTested
        | Q::ReleaseFractionEquivalentToOneParticle
        | Q::CompactReleaseFractionIndicatingFailedSic => {
            return verdict(
                Status::NotAssessed,
                "not_a_release_value",
                "a count, a one-particle constant or a failure threshold: it gives a denominator or an interpretation, not a release to compare",
            );
        }
        _ => {}
    }

    // 2. A range of the prompt release needs the temperature ramps.
    if row.quantity == Q::PromptCompactReleaseFractionRange && row.group == Group::Intact {
        return verdict(
            Status::NotAssessed,
            "needs_ramps",
            "the prompt release covers the heating ramps and the first hours at temperature; the solver has a constant diffusion coefficient, and temperature-dependent diffusion is not yet in the solver",
        );
    }

    // 3. Quantities that describe the irradiation phase, not a heating test.
    if matches!(
        row.quantity,
        Q::CapsuleFractionOutsideCompacts
            | Q::FractionInMatrixAndOpycDlbl
            | Q::FractionInMatrixUpperBound
            | Q::FractionOfAgInventoryInMatrixRange
    ) {
        return verdict(
            Status::NotDirectlyComparable,
            "irradiation_phase_quantity",
            "measures where the inventory sat after irradiation (capsule balance or deposit in matrix and OPyC), not release during a heating test",
        );
    }

    // 4. Other programmes and fuel forms.
    match row.program {
        Program::Crp6 => {
            return verdict(
                Status::NotDirectlyComparable,
                "fuel_sphere",
                "fuel-sphere value with matrix from a benchmark, approximate text value; not a single-particle quantity",
            );
        }
        Program::Agr2 => {
            return verdict(
                Status::NotDirectlyComparable,
                "other_fuel_batch",
                "AGR-2 UCO fuel from another irradiation; the registered geometry and coefficients are those of AGR-1, and the totals include inventory released in pile",
            );
        }
        _ => {}
    }

    // 5. Particles with failed layers, mixtures, loose particles with failures.
    match row.group {
        Group::FailedSic
        | Group::FailedSicIndicator
        | Group::BetweenFailures
        | Group::NoCountedFailureReleasingParticle
        | Group::FailureCount => {
            return verdict(
                Status::NotDirectlyComparable,
                "failed_or_released_particles",
                "measures compacts in which particles with failed SiC (or a releasing particle) dominate; the model has intact coatings",
            );
        }
        Group::All => {
            return verdict(
                Status::NotDirectlyComparable,
                "mixed_particles",
                "mixes failed and intact particles or several compacts",
            );
        }
        Group::LooseParticles => {
            return verdict(
                Status::NotDirectlyComparable,
                "loose_particles_with_failures",
                "75 loose particles of which 5 had failed TRISO layers; the unit is particle equivalents summed over the sample, cup-efficiency scaled, on the clock of the whole heating including the ramp",
            );
        }
        _ => {}
    }
    if row.quantity == Q::CumulativeFractionOnDepositionCups {
        return verdict(
            Status::NotDirectlyComparable,
            "deposition_cup_history",
            "silver collected on deposition cups over a multi-temperature history; collection efficiency not corrected, a lower bound of release, much of the inventory already left the compact in pile",
        );
    }
    if matches!(row.temperature, TestTemperature::Varied { .. }) {
        return verdict(
            Status::NotDirectlyComparable,
            "multi_temperature_history",
            "the temperature changes during the test; the solver has a constant diffusion coefficient",
        );
    }

    // 6. A denominator that the register does not establish (decision 22).
    let corrected = row.time_basis.contains("matrix-corrected");
    if row.denominator == Denominator::NotEstablished {
        let mut v = verdict(
            Status::DataIncomplete,
            "denominator_not_established",
            "the register does not establish what this fraction is a fraction of (decision 22)",
        );
        if row.program == Program::Jaeri {
            v.reason.push_str(
                "; the heat-up ramp of the loose-particle tests is not stated and the end-of-irradiation profile is missing (see particle_applicability); the particle design (UO2 TRISO of another design) also differs from the registered AGR1-B-M16 geometry",
            );
        }
        if corrected && row.group == Group::Intact {
            if let (Some((element, label_only)), TestTemperature::Celsius(t), Some(value)) =
                (element_of(row), &row.temperature, single_value(row))
            {
                if let Some(species) = Species::of_element(&element) {
                    // The window is the release after the first 24 h to the end of the plateau.
                    let (t_end, source) = window_end(reg, row, t.value);
                    v.plan = Plan::Provisional {
                        spec: RunSpec {
                            species,
                            element_label_only: label_only,
                            t_c: t.value,
                            window_start_h: WINDOW_START_H,
                            t_end_h: t_end,
                            time_source: source,
                        },
                        value,
                    };
                }
            }
        }
        return v;
    }

    // 6b. JAERI loose particles are UO2 TRISO of another design: even with an established
    // denominator they are not run against the registered AGR1-B-M16 geometry.
    if row.program == Program::Jaeri {
        return verdict(
            Status::NotDirectlyComparable,
            "other_particle_design",
            "JAERI UO2 TRISO particles of another design; the registered geometry and coefficients are those of AGR-1, so the row is not run against them",
        );
    }

    // 7. Intact compacts: what the model can be compared with.
    let Some((element, label_only)) = element_of(row) else {
        return verdict(
            Status::NotAssessed,
            "no_nuclide",
            "the row names no nuclide",
        );
    };
    let TestTemperature::Celsius(t) = &row.temperature else {
        return verdict(
            Status::DataIncomplete,
            "temperature_not_a_single_value",
            "the register gives no single test temperature for this row",
        );
    };
    if element == "Kr" {
        return verdict(
            Status::DataIncomplete,
            "noble_gas",
            "krypton: the bounds are limited by background, the heating-test krypton SiC row is under source re-read (decision 45) and noble-gas retention is not assessed",
        );
    }
    let in_pile_dominated = matches!(element.as_str(), "Ag" | "Sr" | "Eu");
    let is_total = matches!(
        row.quantity,
        Q::TotalCompactReleaseFraction | Q::CumulativeCompactReleaseFraction
    );
    if in_pile_dominated && (is_total || (!corrected && !row.value.is_bound())) {
        return verdict(
            Status::NotDirectlyComparable,
            "includes_in_pile_inventory",
            "compact total, not matrix-corrected: includes inventory already outside the SiC before the test; published work attributes most Ag, Eu and Sr release to the irradiation, so the number is not release through intact coatings during the test",
        );
    }
    let Some(species) = Species::of_element(&element) else {
        return verdict(
            Status::DataIncomplete,
            "no_registered_coefficients",
            format!("the register has no complete heating-test coefficient set for {element}"),
        );
    };
    let Some(value) = single_value(row) else {
        return verdict(
            Status::NotAssessed,
            "not_a_single_number",
            "the row is not a single number",
        );
    };
    let Some((t_end, source)) = end_time(reg, row, t.value) else {
        return verdict(
            Status::DataIncomplete,
            "no_test_duration",
            "neither the row nor the temperature register gives the test duration at this temperature",
        );
    };
    let spec = |window_start_h: f64| RunSpec {
        species,
        element_label_only: label_only,
        t_c: t.value,
        window_start_h,
        t_end_h: t_end,
        time_source: source.clone(),
    };

    // 7a. A reported upper bound.
    if let ReleaseValue::UpperBound(_) = row.value {
        return Verdict {
            status: Status::ComparisonAvailable,
            category: "inequality_reported_bound",
            reason: "reported upper bound: the model release must not exceed it (decision 23)"
                .to_string(),
            plan: Plan::Inequality {
                spec: spec(0.0),
                limit: value,
                source: BoundSource::ReportedUpperBound,
            },
        };
    }
    // 7b. A total that includes matrix and background: an upper bound on release through intact
    // coatings where the in-pile inventory is small (caesium), decision 19.
    if element == "Cs" && is_total {
        let digitised = matches!(row.value, ReleaseValue::Digitised(_));
        return Verdict {
            status: Status::ComparisonAvailable,
            category: "inequality_total_as_bound",
            reason: "caesium compact total (mean particle release plus matrix plus background, all non-negative): read as an upper bound on release through intact coatings, labelled digitised where it is (decision 19)"
                .to_string(),
            plan: Plan::Inequality {
                spec: spec(0.0),
                limit: value,
                source: if digitised {
                    BoundSource::DigitisedTotal
                } else {
                    BoundSource::PrintedTotal
                },
            },
        };
    }
    // 7c. A matrix-corrected value of an intact compact with an established denominator. It is
    // a fraction of the as-run calculated compact inventory (Collin et al. 2016, manuscript p. 3)
    // over the window from about 24 h at temperature to the end of the 300 h plateau, or to the
    // end of the test if shorter (p. 4), and the source calls the correction an attempt that
    // cannot separate particle release from matrix release (p. 6). It is therefore never a
    // point comparison: it is a window upper bound under premise BP1, which is the source's
    // assumption, not an established fact (decisions 67 and 68(a)). It never enters N.
    if corrected {
        let (window_end_h, window_origin) = window_end(reg, row, t.value);
        return Verdict {
            status: Status::ComparisonAvailable,
            category: "window_upper_bound_bp1",
            reason: format!(
                "{WINDOW_BOUND_CLASS}: matrix-corrected fraction of the {WINDOW_BOUND_DENOMINATOR} over the window from about {WINDOW_START_H} h at temperature to the end of the {PLATEAU_H} h plateau (or the test end if shorter); the model release over the same window must not exceed it; the correction is the source's attempt and cannot separate particle release from matrix release (Collin et al. 2016, manuscript p. 6); not a point comparison, no NRMSE"
            ),
            plan: Plan::Inequality {
                spec: RunSpec {
                    species,
                    element_label_only: label_only,
                    t_c: t.value,
                    window_start_h: WINDOW_START_H,
                    t_end_h: window_end_h,
                    time_source: window_origin,
                },
                limit: value,
                source: BoundSource::WindowUnderBp1,
            },
        };
    }
    verdict(
        Status::NotAssessed,
        "no_rule",
        "no comparison rule covers this row",
    )
}

/// The model-side limitations that every result carries.
pub const MODEL_LIMITATIONS: &str = "kernel-only start (inventory-matched start node); constant diffusion coefficient per layer evaluated at the test temperature from the first hour (heating ramps are not modelled); no radioactive decay and no fission-product generation in the solver yet; perfect interface contact; absorbing outer surface; one intact particle, no matrix; the model fraction is a fraction of the particle's own inventory";

/// Short code list of the categories, for the documentation and the schema.
pub const CATEGORIES: &[(&str, &str)] = &[
    ("not_a_release_value", "count, constant or threshold"),
    ("needs_ramps", "prompt range over the heating ramps"),
    (
        "irradiation_phase_quantity",
        "irradiation-phase inventory location",
    ),
    ("fuel_sphere", "fuel sphere of a benchmark"),
    ("other_fuel_batch", "AGR-2 fuel"),
    (
        "other_particle_design",
        "JAERI UO2 TRISO of another design than the registered geometry",
    ),
    (
        "failed_or_released_particles",
        "failed SiC or releasing particle",
    ),
    ("mixed_particles", "mixture of failed and intact particles"),
    (
        "loose_particles_with_failures",
        "loose particles including failed ones",
    ),
    ("deposition_cup_history", "deposition-cup collection"),
    ("multi_temperature_history", "multi-temperature history"),
    ("denominator_not_established", "denominator not established"),
    (
        "temperature_not_a_single_value",
        "no single test temperature",
    ),
    ("noble_gas", "krypton"),
    (
        "includes_in_pile_inventory",
        "total includes in-pile inventory",
    ),
    ("no_registered_coefficients", "no coefficient set"),
    ("no_test_duration", "no test duration"),
    ("inequality_reported_bound", "reported upper bound"),
    ("inequality_total_as_bound", "total read as an upper bound"),
    (
        "window_upper_bound_bp1",
        "window upper bound under premise BP1",
    ),
];
