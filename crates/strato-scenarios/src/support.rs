//! The equation stages, the nuclides and the temperatures that the selectors of the website
//! offer (decisions 56, 57 and 66; science design Sections 2 and 6).

/// One equation stage (decision 57): the mechanisms that are on in it. The stages are cumulative
/// as the equation is: stage 3 keeps the term of stage 2, stage 4 changes the diffusion operator
/// and keeps both, and stage 5 is the combined equation (science design Section 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageDef {
    /// Stage number 1 to 5.
    pub stage: u8,
    /// Stable id.
    pub id: &'static str,
    /// Name shown by the page.
    pub name: &'static str,
    /// What this stage adds to the equation of the stage before it.
    pub adds: &'static str,
    /// Mechanism ids that are on, in the order the equation gains them.
    pub mechanisms: &'static [&'static str],
}

/// The mechanism ids of the equation builder (`SWITCH_MECHANISM`).
pub const MECHANISM_GENERATION: &str = "generation";
/// Radioactive decay, the term `- lambda C`.
pub const MECHANISM_DECAY: &str = "decay";
/// Temperature-dependent diffusivity, a change of the diffusion operator.
pub const MECHANISM_DIFFUSIVITY_TEMPERATURE: &str = "diffusivity_temperature";

/// The five equation stages.
pub const STAGES: [StageDef; 5] = [
    StageDef {
        stage: 1,
        id: "baseline",
        name: "baseline spherical diffusion",
        adds: "nothing: the diffusion equation with D per layer",
        mechanisms: &[],
    },
    StageDef {
        stage: 2,
        id: "generation",
        name: "generation",
        adds: "the source term + G(r, t) on the right",
        mechanisms: &[MECHANISM_GENERATION],
    },
    StageDef {
        stage: 3,
        id: "decay",
        name: "radioactive decay",
        adds: "the term - lambda C on the right",
        mechanisms: &[MECHANISM_GENERATION, MECHANISM_DECAY],
    },
    StageDef {
        stage: 4,
        id: "diffusivity_temperature",
        name: "temperature-dependent diffusivity",
        adds: "no term: D(r) becomes D(r, T(t)) inside the diffusion operator",
        mechanisms: &[
            MECHANISM_GENERATION,
            MECHANISM_DECAY,
            MECHANISM_DIFFUSIVITY_TEMPERATURE,
        ],
    },
    StageDef {
        stage: 5,
        id: "combined",
        name: "combined",
        adds: "nothing: generation, decay and D(r, T(t)) together",
        mechanisms: &[
            MECHANISM_GENERATION,
            MECHANISM_DECAY,
            MECHANISM_DIFFUSIVITY_TEMPERATURE,
        ],
    },
];

/// The stage of a number, if it is 1 to 5.
pub fn stage_def(stage: u8) -> Option<&'static StageDef> {
    STAGES.iter().find(|s| s.stage == stage)
}

/// A nuclide of the experimental comparison and of model exploration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpNuclide {
    /// Short id used in keys.
    pub id: &'static str,
    /// Name in the registers.
    pub name: &'static str,
    /// Full name for labels.
    pub label: &'static str,
    /// Element of the diffusivity rows.
    pub element: &'static str,
}

/// Ag-110m, Cs-134 (caesium rows) and Sr-90.
pub const EXP_NUCLIDES: [ExpNuclide; 3] = [
    ExpNuclide {
        id: "ag110m",
        name: "Ag-110m",
        label: "Silver-110m",
        element: "Ag",
    },
    ExpNuclide {
        id: "cs134",
        name: "Cs-134",
        label: "Caesium-134 (caesium diffusivity rows)",
        element: "Cs",
    },
    ExpNuclide {
        id: "sr90",
        name: "Sr-90",
        label: "Strontium-90",
        element: "Sr",
    },
];

/// The temperatures of the main experimental comparison per nuclide (science design 2.1).
pub fn exp_temperatures(nuc_id: &str) -> &'static [f64] {
    match nuc_id {
        "ag110m" | "sr90" => &[1600.0, 1700.0],
        _ => &[1600.0, 1700.0, 1800.0],
    }
}

/// The irradiation-phase exploration temperatures (science design 2.2: IRR-361, IRR-059,
/// IRR-136, IRR-097).
pub const EXPL_IRRADIATION_C: [f64; 4] = [955.0, 1028.0, 1136.0, 1197.0];
/// The heating-test exploration temperatures: every hold temperature of a registered history.
pub const EXPL_HEATING_C: [f64; 11] = [
    400.0, 1000.0, 1075.0, 1150.0, 1250.0, 1300.0, 1375.0, 1450.0, 1600.0, 1700.0, 1800.0,
];
