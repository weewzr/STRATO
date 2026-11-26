//! Material parameters, diffusivity laws, yields and the fission-rate density of the registers.

use strato::numerical::scenario::{DiffusivityAt, RangeCode};
use strato_compare::json::J;
use strato_data::decay::Stability;
use strato_data::irradiation::{SeriesTarget, AVOGADRO_PER_MOL};
use strato_data::select::{ParticleLayer, Phase, RangeStatus, Selection, Selector, Variant};
use strato_data::table::Vocabulary;
use strato_data::yields::Parent;
use strato_data::Register;

/// The layers in the order of the solver.
pub const LAYERS: [(&str, ParticleLayer); 5] = [
    ("kernel", ParticleLayer::Kernel),
    ("buffer", ParticleLayer::Buffer),
    ("IPyC", ParticleLayer::IPyC),
    ("SiC", ParticleLayer::SiC),
    ("OPyC", ParticleLayer::OPyC),
];

/// The five selections of one element and phase.
pub struct Laws<'a> {
    /// Kernel, buffer, IPyC, SiC, OPyC.
    pub sel: Vec<Selection<'a>>,
}

impl<'a> Laws<'a> {
    /// The published rows of an element for a phase (decisions 39, 40, 47).
    ///
    /// # Errors
    ///
    /// A layer without a published coefficient.
    pub fn new(reg: &'a Register, element: &str, phase: Phase) -> Result<Laws<'a>, String> {
        let selector = Selector {
            arrhenius: &reg.arrhenius,
            irradiation: &reg.irradiation,
        };
        let mut sel = Vec::new();
        for (name, layer) in LAYERS {
            sel.push(
                selector
                    .select(element, layer, phase, Variant::Default)
                    .map_err(|e| format!("{element} {name} {}: {e}", phase.as_str()))?,
            );
        }
        Ok(Laws { sel })
    }

    /// The five diffusivities at a temperature in degC, each with its position against the
    /// printed range (an extrapolation is marked, never clamped).
    ///
    /// # Errors
    ///
    /// Never at present; the signature is the engine's.
    pub fn at(&self, t_c: f64) -> anyhow::Result<DiffusivityAt> {
        let mut d = [0.0; 5];
        let mut codes = [RangeCode::NotStated; 5];
        for (k, s) in self.sel.iter().enumerate() {
            let e = s.diffusivity_extrapolated(t_c);
            d[k] = e.diffusivity.m2_per_s;
            codes[k] = match e.range {
                RangeStatus::NoRangePrinted => RangeCode::NotStated,
                RangeStatus::Inside => RangeCode::Within,
                RangeStatus::Extrapolated { .. } => RangeCode::Outside,
            };
        }
        Ok((d, codes))
    }

    /// The description of the selections for the parameters file.
    pub fn json(&self, probe_c: &[f64]) -> J {
        J::Arr(
            self.sel
                .iter()
                .zip(LAYERS)
                .map(|(s, (name, _))| {
                    J::obj(vec![
                        ("layer", J::s(name)),
                        (
                            "register_rows",
                            J::Arr(s.row_ids().iter().map(|i| J::s(*i)).collect()),
                        ),
                        ("terms", J::Int(s.terms() as i64)),
                        ("coefficient_class", J::s(s.class.as_str())),
                        ("phase_of_use", J::s(s.rows[0].phase_of_use.as_str())),
                        ("source", J::s(s.source.clone())),
                        ("source_location", J::s(s.source_location.clone())),
                        (
                            "published_range_c",
                            match s.published_range_c {
                                Some((a, b)) => J::Arr(vec![J::Num(a), J::Num(b)]),
                                None => J::Null,
                            },
                        ),
                        (
                            "range_kind",
                            s.range_kind.map_or(J::Null, |k| J::s(k.as_str())),
                        ),
                        (
                            "labels",
                            J::Arr(s.labels.iter().map(|l| J::s(l.clone())).collect()),
                        ),
                        (
                            "diffusivity_m2_s_at_c",
                            J::Arr(
                                probe_c
                                    .iter()
                                    .map(|t| {
                                        let e = s.diffusivity_extrapolated(*t);
                                        J::obj(vec![
                                            ("temperature_c", J::Num(*t)),
                                            ("diffusivity_m2_s", J::Num(e.diffusivity.m2_per_s)),
                                            (
                                                "range",
                                                J::s(match e.range {
                                                    RangeStatus::NoRangePrinted => {
                                                        "no range printed"
                                                    }
                                                    RangeStatus::Inside => {
                                                        "inside the printed range"
                                                    }
                                                    RangeStatus::Extrapolated { .. } => {
                                                        "outside the printed range"
                                                    }
                                                }),
                                            ),
                                        ])
                                    })
                                    .collect(),
                            ),
                        ),
                    ])
                })
                .collect(),
        )
    }
}

/// The value of a numeric condition row.
///
/// # Errors
///
/// A row that is missing or not a number.
pub fn condition(reg: &Register, id: &str) -> Result<f64, String> {
    use strato_data::irradiation::ConditionValue;
    reg.irradiation
        .conditions
        .iter()
        .find(|r| r.id == id)
        .and_then(|r| match &r.value {
            ConditionValue::Number(p) => Some(p.value),
            ConditionValue::Text(_) => None,
        })
        .ok_or_else(|| format!("condition row {id} is missing or not a number"))
}

/// A fission-rate density with the rows it comes from.
#[derive(Debug, Clone)]
pub struct RateDensity {
    /// Fissions per cubic metre of kernel per second.
    pub value: f64,
    /// The label that goes with it (decision 42).
    pub label: String,
    /// What it is derived for (`capsule:3` or `compact:6-2-1`).
    pub target: String,
    /// Register rows it comes from.
    pub source_ids: Vec<String>,
}

/// The reference fission-rate density named by a scenario (`capsule:3`, `compact:6-2-1`).
///
/// # Errors
///
/// An unknown target or a register row that is missing.
pub fn rate_density(reg: &Register, target: &str) -> Result<RateDensity, String> {
    let label = strato_data::irradiation::DERIVED_LABEL.to_string();
    if target == "capsule:3" {
        let bu = condition(reg, "IRR-053")?;
        let efpd = condition(reg, "IRR-002")?;
        let v = reg
            .irradiation
            .time_average_from_end_burnup(bu, efpd)
            .map_err(|e| e.to_string())?;
        let mut ids = reg
            .irradiation
            .kernel_inventory()
            .map_err(|e| e.to_string())?
            .source_ids;
        ids.splice(0..0, ["IRR-053".to_string(), "IRR-002".to_string()]);
        return Ok(RateDensity {
            value: v,
            label,
            target: target.into(),
            source_ids: ids,
        });
    }
    if let Some(name) = target.strip_prefix("compact:") {
        let s = reg
            .irradiation
            .derive_series(SeriesTarget::Compact(name.to_string()))
            .map_err(|e| e.to_string())?;
        return Ok(RateDensity {
            value: s.time_average(),
            label,
            target: target.into(),
            source_ids: s.source_ids.clone(),
        });
    }
    Err(format!("unknown rate density target `{target}`"))
}

/// A nuclide of a scenario: its decay constant, cumulative yield and the rows they come from.
#[derive(Debug, Clone)]
pub struct Nuclide {
    /// Short id (`sr90`).
    pub id: &'static str,
    /// Name in the registers (`Sr-90`).
    pub name: &'static str,
    /// Element whose diffusivity rows apply.
    pub element: &'static str,
    /// Decay constant in s^-1.
    pub lambda_per_s: f64,
    /// Cumulative yield of U-235 thermal fission, atoms per fission, with the JEFF-3.1.1 column.
    pub yield_per_fission: f64,
}

/// The nuclides for which generation is meaningful (Sr-90, Cs-137).
///
/// # Errors
///
/// A register row that is missing.
pub fn generation_nuclides(reg: &Register) -> Result<Vec<Nuclide>, String> {
    let mut out = Vec::new();
    for (id, name, element) in [("sr90", "Sr-90", "Sr"), ("cs137", "Cs-137", "Cs")] {
        out.push(Nuclide {
            id,
            name,
            element,
            lambda_per_s: decay_constant(reg, name)?,
            yield_per_fission: reg
                .yields
                .get(Parent::U235, name)
                .ok_or_else(|| format!("no yield row for {name}"))?
                .fission_source_yield()
                .map_err(|e| format!("{name}: {e:?}"))?
                .value
                .value,
        });
    }
    Ok(out)
}

/// The decay constant of a nuclide in s^-1 from `decay.csv`.
///
/// # Errors
///
/// A nuclide that is missing or stable.
pub fn decay_constant(reg: &Register, name: &str) -> Result<f64, String> {
    match reg.decay.get(name).map(|r| &r.stability) {
        Some(Stability::Radioactive(r)) => Ok(r.lambda_per_s.value),
        _ => Err(format!("{name} has no decay constant in the register")),
    }
}

/// The reference generation rate `y F_dot / N_A` in mol m^-3 s^-1 (Document 5, Eq. (2)).
pub fn reference_generation(yield_per_fission: f64, rate_density: f64) -> f64 {
    yield_per_fission * rate_density / AVOGADRO_PER_MOL
}
