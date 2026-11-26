//! The registers and the scenario files, embedded at build time and typed once.
//!
//! The wrapper reads no file. The files of `data/` that the scenario engine and the diffusivity
//! laws need are compiled into the module with `include_str!` and typed by `strato-data`, the
//! crate that also types them for `strato-results`, which writes `data/results/`. A scenario
//! definition is therefore built by the same function from the same text in the page and in the
//! results files (decision 59). The text is parsed on the first call that needs it, never at
//! start-up.

use std::sync::OnceLock;
use strato_data::register::{Register, RegisterTexts};
use strato_data::scenarios::{ScenarioTexts, Scenarios};

/// The embedded files: the path below `data/` and the size in bytes.
pub const EMBEDDED_FILES: [(&str, usize); 14] = [
    (
        "geometry/geometry.csv",
        include_str!("../../../data/geometry/geometry.csv").len(),
    ),
    (
        "diffusivity/arrhenius.csv",
        include_str!("../../../data/diffusivity/arrhenius.csv").len(),
    ),
    (
        "nuclear/decay.csv",
        include_str!("../../../data/nuclear/decay.csv").len(),
    ),
    (
        "nuclear/fission_yields.csv",
        include_str!("../../../data/nuclear/fission_yields.csv").len(),
    ),
    (
        "temperature/temperature_histories.csv",
        include_str!("../../../data/temperature/temperature_histories.csv").len(),
    ),
    (
        "experimental/release_measurements.csv",
        include_str!("../../../data/experimental/release_measurements.csv").len(),
    ),
    (
        "irradiation/agr1_irradiation_conditions.csv",
        include_str!("../../../data/irradiation/agr1_irradiation_conditions.csv").len(),
    ),
    (
        "irradiation/agr1_burnup_by_cycle.csv",
        include_str!("../../../data/irradiation/agr1_burnup_by_cycle.csv").len(),
    ),
    (
        "irradiation/agr1_power_history_digitised.csv",
        include_str!("../../../data/irradiation/agr1_power_history_digitised.csv").len(),
    ),
    (
        "scenarios/s1_normal_operation.csv",
        include_str!("../../../data/scenarios/s1_normal_operation.csv").len(),
    ),
    (
        "scenarios/s2_shutdown_cooldown.csv",
        include_str!("../../../data/scenarios/s2_shutdown_cooldown.csv").len(),
    ),
    (
        "scenarios/s3_startup.csv",
        include_str!("../../../data/scenarios/s3_startup.csv").len(),
    ),
    (
        "scenarios/s4_control_rod_insertion.csv",
        include_str!("../../../data/scenarios/s4_control_rod_insertion.csv").len(),
    ),
    (
        "scenarios/s5_fuel_removal.csv",
        include_str!("../../../data/scenarios/s5_fuel_removal.csv").len(),
    ),
];

fn register_texts() -> RegisterTexts {
    RegisterTexts {
        geometry: include_str!("../../../data/geometry/geometry.csv").to_string(),
        arrhenius: include_str!("../../../data/diffusivity/arrhenius.csv").to_string(),
        decay: include_str!("../../../data/nuclear/decay.csv").to_string(),
        yields: include_str!("../../../data/nuclear/fission_yields.csv").to_string(),
        temperature: include_str!("../../../data/temperature/temperature_histories.csv")
            .to_string(),
        release: include_str!("../../../data/experimental/release_measurements.csv").to_string(),
        irradiation_conditions: include_str!(
            "../../../data/irradiation/agr1_irradiation_conditions.csv"
        )
        .to_string(),
        irradiation_burnup: include_str!("../../../data/irradiation/agr1_burnup_by_cycle.csv")
            .to_string(),
        irradiation_power: include_str!(
            "../../../data/irradiation/agr1_power_history_digitised.csv"
        )
        .to_string(),
        register_markdown: None,
    }
}

fn scenario_texts() -> ScenarioTexts {
    ScenarioTexts {
        files: vec![
            include_str!("../../../data/scenarios/s1_normal_operation.csv").to_string(),
            include_str!("../../../data/scenarios/s2_shutdown_cooldown.csv").to_string(),
            include_str!("../../../data/scenarios/s3_startup.csv").to_string(),
            include_str!("../../../data/scenarios/s4_control_rod_insertion.csv").to_string(),
            include_str!("../../../data/scenarios/s5_fuel_removal.csv").to_string(),
        ],
    }
}

static REGISTER: OnceLock<Result<Register, String>> = OnceLock::new();
static SCENARIOS: OnceLock<Result<Scenarios, String>> = OnceLock::new();

/// The typed registers, parsed on the first call.
///
/// # Errors
///
/// A register file that does not type (cannot happen for the files of a verified commit; the
/// tests parse them).
pub fn register() -> Result<&'static Register, String> {
    REGISTER
        .get_or_init(|| {
            Register::from_texts(&register_texts())
                .map_err(|e| format!("the embedded registers do not type: {e}"))
        })
        .as_ref()
        .map_err(Clone::clone)
}

/// The five typed scenario files, parsed on the first call.
///
/// # Errors
///
/// A scenario file that does not type.
pub fn scenarios() -> Result<&'static Scenarios, String> {
    SCENARIOS
        .get_or_init(|| {
            Scenarios::parse(&scenario_texts())
                .map_err(|e| format!("the embedded scenario files do not type: {e}"))
        })
        .as_ref()
        .map_err(Clone::clone)
}
