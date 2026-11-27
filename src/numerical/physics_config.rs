//! The optional `[physics]` section of the TOML input: generation, radioactive decay and
//! temperature-dependent diffusion (directive section 17).
//!
//! The section is read from the same file as the baseline input (the baseline reader ignores
//! it). Nothing in it has a default that stands for data:
//!
//! * the fission-rate density is an explicit history with no default (decisions 14 and 42);
//! * the yield, its kind and the product kind are explicit, and a direct fission-yield source
//!   for an activation product is refused (decision 15);
//! * the Arrhenius law of each material is a list of terms: one term or the published sum of
//!   terms. Which form applies is decided by the data row the user takes it from, never by
//!   the code (decisions 13 and 39);
//! * the decay constant is given as a half-life or directly, not looked up.
//!
//! ```toml
//! [physics.decay]
//! half_life_s = 1.0e6                 # or decay_constant_per_s = ...
//!
//! [physics.source]
//! mode = "kernel_fission_yield"       # or "per_material" or "none"
//! product = "direct_fission_product"  # or "activation_product" (refused here)
//! yield_kind = "cumulative"           # or "independent"
//! yield_atoms_per_fission = 0.06
//! fission_rate_origin = "user-supplied"
//! [physics.source.fission_rate_density]   # fissions per cubic metre per second
//! value = 1.0e19                      # or interpolation = "step"/"linear" with times_s, values
//!
//! [physics.temperature]
//! unit = "K"                          # or "degC"
//! interpolation = "linear"
//! times_s = [0.0, 2.0e5]
//! values = [900.0, 1000.0]
//! [[physics.arrhenius]]               # one entry for each of the five materials
//! region = "Kernel"
//! source = "register row id"          # recorded in the export
//! class = "published PARFUME correlation"
//! [[physics.arrhenius.terms]]
//! d0_m2_s = 1.0e-6
//! activation_energy_kj_per_mol = 200.0   # or activation_energy_j_per_mol
//! valid_from_k = 1000.0               # optional, as printed by the source
//! valid_to_k = 1800.0
//! ```
//!
//! With a temperature history the `diffusivity_m2_s` of the regions is not used (the
//! Arrhenius laws replace it); it must still be present and positive because the baseline
//! reader validates it.

use super::{
    grid::{Region, ValidatedInput},
    physics_run::{DiffusivityModel, Interpolation, PhysicsInput, SourceModel, TimeFunction},
};
use crate::physics::{
    arrhenius::{
        ArrheniusLaw, ArrheniusTerm, EnergyUnit, GasConstant, Temperature, TemperatureRange,
    },
    decay::{decay_constant_from_half_life, TimeUnit},
    source::{FissionYield, ProductKind, YieldKind},
    CELSIUS_OFFSET_K,
};
use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;

/// A history in the TOML file: `value` (constant) or `interpolation` with `times_s`, `values`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistorySection {
    /// `"constant"`, `"step"` or `"linear"`; may be left out for a constant `value`.
    pub interpolation: Option<String>,
    /// The constant value.
    pub value: Option<f64>,
    /// Time points in s.
    pub times_s: Option<Vec<f64>>,
    /// Values at the points.
    pub values: Option<Vec<f64>>,
}

impl HistorySection {
    /// The history as a [`TimeFunction`].
    ///
    /// # Errors
    ///
    /// An inconsistent or invalid table.
    pub fn to_time_function(&self, what: &str) -> Result<TimeFunction> {
        let mode = self.interpolation.as_deref();
        match (self.value, &self.times_s, &self.values) {
            (Some(v), None, None) => {
                if !matches!(mode, None | Some("constant")) {
                    bail!("{what}: a constant value needs no interpolation, or \"constant\"");
                }
                TimeFunction::constant(v).with_context(|| what.to_string())
            }
            (None, Some(t), Some(v)) => {
                let m = match mode {
                    Some("step") => Interpolation::Step,
                    Some("linear") => Interpolation::Linear,
                    _ => bail!("{what}: interpolation must be \"step\" or \"linear\" for a table"),
                };
                TimeFunction::table(t.clone(), v.clone(), m).with_context(|| what.to_string())
            }
            _ => bail!("{what}: give either `value`, or `times_s` with `values`"),
        }
    }
}

/// `[physics.decay]`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecaySection {
    /// Half-life in s.
    pub half_life_s: Option<f64>,
    /// Decay constant in s^-1.
    pub decay_constant_per_s: Option<f64>,
}

/// `[physics.source]`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSection {
    /// `"none"`, `"kernel_fission_yield"` or `"per_material"`.
    pub mode: String,
    /// `"direct_fission_product"` or `"activation_product"`.
    pub product: Option<String>,
    /// `"independent"` or `"cumulative"`.
    pub yield_kind: Option<String>,
    /// Atoms per fission.
    pub yield_atoms_per_fission: Option<f64>,
    /// Fission-rate density in fissions m^-3 s^-1; explicit, no default.
    pub fission_rate_density: Option<HistorySection>,
    /// Where the fission-rate density comes from, recorded in the export.
    pub fission_rate_origin: Option<String>,
    /// `per_material`: kernel source in mol m^-3 s^-1.
    pub kernel: Option<HistorySection>,
    /// `per_material`: buffer source.
    pub buffer: Option<HistorySection>,
    /// `per_material`: IPyC source.
    pub ipyc: Option<HistorySection>,
    /// `per_material`: SiC source.
    pub sic: Option<HistorySection>,
    /// `per_material`: OPyC source.
    pub opyc: Option<HistorySection>,
}

/// `[physics.temperature]`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemperatureSection {
    /// `"K"` or `"degC"`.
    pub unit: String,
    /// See [`HistorySection`].
    pub interpolation: Option<String>,
    /// Constant temperature.
    pub value: Option<f64>,
    /// Time points in s.
    pub times_s: Option<Vec<f64>>,
    /// Temperatures at the points.
    pub values: Option<Vec<f64>>,
    /// Gas constant of the fit, J mol^-1 K^-1; the exact SI value if left out.
    pub gas_constant_j_mol_k: Option<f64>,
}

/// One term of an Arrhenius law.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TermSection {
    /// Pre-exponential factor in m^2 s^-1.
    pub d0_m2_s: f64,
    /// Activation energy in J mol^-1.
    pub activation_energy_j_per_mol: Option<f64>,
    /// Activation energy in kJ mol^-1.
    pub activation_energy_kj_per_mol: Option<f64>,
    /// Lower end of the printed validity range in K.
    pub valid_from_k: Option<f64>,
    /// Upper end of the printed validity range in K.
    pub valid_to_k: Option<f64>,
}

/// `[[physics.arrhenius]]`: the law of one material.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrheniusSection {
    /// `Kernel`, `Buffer`, `IPyC`, `SiC` or `OPyC`.
    pub region: String,
    /// Source row of the law (decision 31), recorded in the export. Required: [`load_physics`]
    /// refuses a law without it.
    pub source: Option<String>,
    /// Coefficient class (decision 16), recorded in the export. Required as `source`.
    pub class: Option<String>,
    /// One term, or the published sum of terms.
    pub terms: Vec<TermSection>,
}

/// The `[physics]` table.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicsSection {
    /// Radioactive decay.
    pub decay: Option<DecaySection>,
    /// Generation.
    pub source: Option<SourceSection>,
    /// Temperature history.
    pub temperature: Option<TemperatureSection>,
    /// Laws of the five materials.
    pub arrhenius: Option<Vec<ArrheniusSection>>,
}

#[derive(Deserialize)]
struct FileWithPhysics {
    physics: Option<PhysicsSection>,
}

/// Texts that the export records beside the numbers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhysicsLabels {
    /// What the source is.
    pub source: String,
    /// Origin of the fission-rate density, if there is one.
    pub fission_rate_origin: Option<String>,
    /// The fission-rate density history in fissions m^-3 s^-1, if there is one.
    pub fission_rate: Option<TimeFunction>,
    /// Per material: the law, its source row and class, as a line of text.
    pub diffusivity: Vec<String>,
    /// What the decay constant is.
    pub decay: String,
}

/// The physics input read from a file, with its labels.
#[derive(Clone, Debug)]
pub struct PhysicsSetup {
    /// The input of [`super::physics_run::run_physics`].
    pub input: PhysicsInput,
    /// Labels for the export.
    pub labels: PhysicsLabels,
}

fn region_index(name: &str) -> Result<usize> {
    Region::ALL
        .iter()
        .position(|r| r.name().eq_ignore_ascii_case(name))
        .ok_or_else(|| {
            anyhow!("unknown region {name:?}; use Kernel, Buffer, IPyC, SiC or OPyC (any case)")
        })
}

fn build_law(sec: &ArrheniusSection) -> Result<ArrheniusLaw> {
    if sec.terms.is_empty() {
        bail!(
            "region {}: an Arrhenius law needs at least one term",
            sec.region
        );
    }
    let mut terms = Vec::new();
    for (m, t) in sec.terms.iter().enumerate() {
        let (ea, unit) = match (
            t.activation_energy_j_per_mol,
            t.activation_energy_kj_per_mol,
        ) {
            (Some(e), None) => (e, EnergyUnit::JoulePerMole),
            (None, Some(e)) => (e, EnergyUnit::KilojoulePerMole),
            _ => bail!(
                "region {} term {}: give the activation energy once, in J/mol or kJ/mol",
                sec.region,
                m + 1
            ),
        };
        let range = match (t.valid_from_k, t.valid_to_k) {
            (Some(a), Some(b)) => Some(
                TemperatureRange::new(Temperature::Kelvin(a), Temperature::Kelvin(b))
                    .map_err(|e| anyhow!("region {} term {}: {e}", sec.region, m + 1))?,
            ),
            (None, None) => None,
            _ => bail!(
                "region {} term {}: give both ends of the validity range or none",
                sec.region,
                m + 1
            ),
        };
        terms.push(
            ArrheniusTerm::new(t.d0_m2_s, ea, unit, range)
                .map_err(|e| anyhow!("region {} term {}: {e}", sec.region, m + 1))?,
        );
    }
    if terms.len() == 1 {
        Ok(ArrheniusLaw::single(terms[0]))
    } else {
        ArrheniusLaw::sum(terms).map_err(|e| anyhow!("region {}: {e}", sec.region))
    }
}

fn history(opt: &Option<HistorySection>, what: &str) -> Result<TimeFunction> {
    match opt {
        Some(h) => h.to_time_function(what),
        None => Ok(TimeFunction::Constant(0.0)),
    }
}

/// Reads the `[physics]` section of `raw_toml`, if there is one, and builds the input of the
/// combined solver on `base` (the validated baseline input of the same file).
///
/// # Errors
///
/// A malformed or incomplete section: a missing fission-rate density, yield or product kind, a
/// direct-yield source for an activation product, a temperature history without the five
/// Arrhenius laws, or any invalid number.
pub fn load_physics(
    raw_toml: &str,
    base: ValidatedInput,
    end_time_s: f64,
) -> Result<Option<PhysicsSetup>> {
    let file: FileWithPhysics =
        toml::from_str(raw_toml).context("parsing the [physics] section")?;
    let Some(sec) = file.physics else {
        return Ok(None);
    };
    let mut labels = PhysicsLabels::default();

    // Decay.
    let lambda = match &sec.decay {
        None => {
            labels.decay = "no decay (stable species)".to_string();
            0.0
        }
        Some(d) => match (d.half_life_s, d.decay_constant_per_s) {
            (Some(h), None) => {
                labels.decay = "decay constant from the half-life given in the input".to_string();
                decay_constant_from_half_life(h, TimeUnit::Seconds)?
            }
            (None, Some(l)) => {
                labels.decay = "decay constant given in the input".to_string();
                l
            }
            _ => bail!("[physics.decay]: give exactly one of half_life_s and decay_constant_per_s"),
        },
    };

    // Source.
    let source = match &sec.source {
        None => {
            labels.source = "no source".to_string();
            SourceModel::none()
        }
        Some(s) => match s.mode.as_str() {
            "none" => {
                labels.source = "no source".to_string();
                SourceModel::none()
            }
            "kernel_fission_yield" => {
                let rate = s.fission_rate_density.as_ref().ok_or_else(|| {
                    anyhow!(
                        "[physics.source]: the fission-rate density is an explicit input with no \
                         default; give [physics.source.fission_rate_density]"
                    )
                })?;
                let rate = rate.to_time_function("fission-rate density")?;
                let product = match s.product.as_deref() {
                    Some("direct_fission_product") => ProductKind::DirectFissionProduct,
                    Some("activation_product") => ProductKind::ActivationProduct,
                    _ => bail!(
                        "[physics.source]: product must be \"direct_fission_product\" or \
                         \"activation_product\""
                    ),
                };
                let kind = match s.yield_kind.as_deref() {
                    Some("independent") => YieldKind::Independent,
                    Some("cumulative") => YieldKind::Cumulative,
                    _ => bail!(
                        "[physics.source]: yield_kind must be \"independent\" or \"cumulative\""
                    ),
                };
                let y = s.yield_atoms_per_fission.ok_or_else(|| {
                    anyhow!("[physics.source]: yield_atoms_per_fission is required")
                })?;
                let fy = FissionYield::new(kind, y)?;
                labels.source = format!(
                    "kernel source from fission-rate density times {} yield, direct fission product",
                    match kind {
                        YieldKind::Independent => "independent",
                        YieldKind::Cumulative => "cumulative",
                    }
                );
                labels.fission_rate_origin = Some(
                    s.fission_rate_origin
                        .clone()
                        .unwrap_or_else(|| "user-supplied".to_string()),
                );
                let model = SourceModel::kernel_fission(&rate, fy, product)?;
                labels.fission_rate = Some(rate);
                model
            }
            "per_material" => {
                // Decision 15: a species declared as an activation product has no source until
                // a production model is approved. The fields of the fission-yield mode would be
                // dropped silently here, so they are refused instead of ignored.
                let stray: Vec<&str> = [
                    ("product", s.product.is_some()),
                    ("yield_kind", s.yield_kind.is_some()),
                    (
                        "yield_atoms_per_fission",
                        s.yield_atoms_per_fission.is_some(),
                    ),
                    ("fission_rate_density", s.fission_rate_density.is_some()),
                    ("fission_rate_origin", s.fission_rate_origin.is_some()),
                ]
                .into_iter()
                .filter_map(|(n, present)| present.then_some(n))
                .collect();
                if !stray.is_empty() {
                    bail!(
                        "[physics.source] mode \"per_material\" takes only the five per-material \
                         histories; {} belong to mode \"kernel_fission_yield\" and would be \
                         ignored (an activation product has no source until a production model \
                         is approved, decision 15)",
                        stray.join(", ")
                    );
                }
                let hs = [&s.kernel, &s.buffer, &s.ipyc, &s.sic, &s.opyc];
                let names = ["kernel", "buffer", "ipyc", "sic", "opyc"];
                let mut per: Vec<TimeFunction> = Vec::new();
                for (h, nm) in hs.iter().zip(names) {
                    per.push(history(h, &format!("source of {nm}"))?);
                }
                labels.source = "source per material in mol m^-3 s^-1, user-supplied".to_string();
                SourceModel {
                    per_material: per.try_into().map_err(|_| anyhow!("five histories"))?,
                }
            }
            other => bail!("[physics.source]: unknown mode {other:?}"),
        },
    };

    // Diffusivity.
    let diffusivity = match (&sec.temperature, &sec.arrhenius) {
        (None, None) => DiffusivityModel::Constant(base.d),
        (None, Some(_)) => bail!("[[physics.arrhenius]] needs a [physics.temperature] history"),
        (Some(_), None) => {
            bail!("[physics.temperature] needs one [[physics.arrhenius]] law per material")
        }
        (Some(t), Some(list)) => {
            let hist = HistorySection {
                interpolation: t.interpolation.clone(),
                value: t.value,
                times_s: t.times_s.clone(),
                values: t.values.clone(),
            };
            let tf = hist.to_time_function("temperature history")?;
            let tf = match t.unit.as_str() {
                "K" => tf,
                "degC" => tf.map_values(|x| Ok(x + CELSIUS_OFFSET_K))?,
                other => {
                    bail!("[physics.temperature]: unit must be \"K\" or \"degC\", not {other:?}")
                }
            };
            let gas = match t.gas_constant_j_mol_k {
                Some(r) => GasConstant::custom(r)?,
                None => GasConstant::SI,
            };
            if list.len() != 5 {
                bail!("exactly five [[physics.arrhenius]] laws are required (one per material)");
            }
            let mut laws: [Option<ArrheniusLaw>; 5] = [None, None, None, None, None];
            let mut rows = vec![String::new(); 5];
            for sec in list {
                let k = region_index(&sec.region)?;
                if laws[k].is_some() {
                    bail!("region {} has two Arrhenius laws", sec.region);
                }
                // Decision 31: every exported parameter carries its source row and class. A
                // law without them is refused; a user's own value is declared as such in the
                // two fields (for example class "user-supplied, no source").
                let (Some(src), Some(class)) = (sec.source.as_deref(), sec.class.as_deref()) else {
                    bail!(
                        "region {}: `source` and `class` are required for every Arrhenius law \
                         (decision 31); for a value of your own write so in both fields",
                        sec.region
                    );
                };
                if src.trim().is_empty() || class.trim().is_empty() {
                    bail!(
                        "region {}: `source` and `class` must not be empty",
                        sec.region
                    );
                }
                laws[k] = Some(build_law(sec)?);
                rows[k] = format!(
                    "{}: {} term(s); source row: {src}; class: {class}",
                    sec.region,
                    sec.terms.len(),
                );
            }
            let laws: Vec<ArrheniusLaw> = laws
                .into_iter()
                .map(|l| l.ok_or_else(|| anyhow!("a material has no Arrhenius law")))
                .collect::<Result<_>>()?;
            labels.diffusivity = rows;
            DiffusivityModel::Arrhenius {
                laws: Box::new(laws.try_into().map_err(|_| anyhow!("five laws"))?),
                gas,
                temperature_k: tf,
            }
        }
    };
    if labels.diffusivity.is_empty() {
        labels.diffusivity = vec!["constant diffusivity from the regions of the input".to_string()];
    }

    let input = PhysicsInput {
        base,
        end_time_s,
        decay_constant_per_s: lambda,
        source,
        diffusivity,
    };
    input.validate()?;
    Ok(Some(PhysicsSetup { input, labels }))
}
