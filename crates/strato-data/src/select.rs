//! Selection of the published diffusion coefficient for one nuclide, one particle layer and one
//! phase, following decisions 39 to 41 of the coordinator.
//!
//! * Decision 39: every run evaluates the correlation exactly as published for its temperature
//!   range. Irradiation-phase runs use the published single-term rows (Collin 2015 Table 2, the
//!   rows `C15-*`); heating-test runs use the published rows of the heating-test set (Collin 2016
//!   Table 2, the rows `C16-*`), which carry a second term where TECDOC-978 prints one. Nothing is
//!   simplified and nothing is added.
//! * Decision 40: the defaults follow the published AGR-1 modelling (class `published PARFUME
//!   correlation`); the kernel rows are UO2 surrogates and keep the label that no UCO measurement
//!   exists. The AGR-1-fitted set of Schappel (2020, rows `S20-01` to `S20-03`, 1600 to 1800 C
//!   only) is a labelled alternative for heating-test runs, never the default.
//! * Decision 41: the kernel coefficients refer to a 250 um kernel radius and are applied
//!   unscaled by default; the variant [`Variant::KernelScaledSensitivity`] multiplies the kernel
//!   coefficient by (r / 250 um)^2 with r the AGR-1 kernel radius and is a labelled sensitivity
//!   case, not a published coefficient.
//!
//! A [`Selection`] refuses to give a coefficient outside the temperature range printed for its
//! rows ([`Selection::diffusivity_at`]) and says so; the extrapolated value is available only
//! through [`Selection::diffusivity_extrapolated`], which marks it.

use std::fmt;

use crate::arrhenius::{
    Arrhenius, ArrheniusRow, BoundKind, CoefficientClass, DiffusionLayer, Diffusivity, PhaseOfUse,
};
use crate::irradiation::{ConditionValue, Irradiation};

/// The gas constant, J mol-1 K-1, used for rows whose source prints none (the value printed by
/// Collin 2016 for its own set); a selection carrying such a row says so in its labels.
pub const GAS_CONSTANT_ASSUMED: f64 = 8.3145;

/// The phase of the AGR-1 experiment a run represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// The irradiation (about 800 to 1250 C compact temperatures).
    Irradiation,
    /// A post-irradiation heating (safety) test (1600 to 1800 C).
    HeatingTest,
}

impl Phase {
    /// The text used for the phase in the register.
    pub fn as_str(self) -> &'static str {
        match self {
            Phase::Irradiation => "irradiation",
            Phase::HeatingTest => "heating test",
        }
    }
}

/// A layer of the TRISO particle of the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParticleLayer {
    /// Kernel.
    Kernel,
    /// Buffer.
    Buffer,
    /// Inner pyrocarbon (shares its set with the outer pyrocarbon, register GAP-02).
    IPyC,
    /// Silicon carbide.
    SiC,
    /// Outer pyrocarbon.
    OPyC,
}

impl ParticleLayer {
    /// The register layer whose rows apply.
    pub fn register_layer(self) -> DiffusionLayer {
        match self {
            ParticleLayer::Kernel => DiffusionLayer::Kernel,
            ParticleLayer::Buffer => DiffusionLayer::Buffer,
            ParticleLayer::IPyC | ParticleLayer::OPyC => DiffusionLayer::PyC,
            ParticleLayer::SiC => DiffusionLayer::SiC,
        }
    }
}

/// Which published set is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    /// The published AGR-1 modelling (decisions 39 and 40).
    Default,
    /// The AGR-1-specific fitted set of Schappel 2020 for Ag, Cs and Sr in SiC (heating tests only).
    SchappelFitted,
    /// The kernel coefficient scaled by (AGR-1 kernel radius / 250 um)^2 (decision 41 sensitivity case).
    KernelScaledSensitivity,
}

/// The reason no coefficient is returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unavailable {
    /// What is missing and why.
    pub reason: String,
}

impl fmt::Display for Unavailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DATA UNAVAILABLE: {}", self.reason)
    }
}

impl std::error::Error for Unavailable {}

/// A temperature outside the published range of the selected rows.
#[derive(Debug, Clone, PartialEq)]
pub struct OutsidePublishedRange {
    /// The temperature asked for, degC.
    pub t_celsius: f64,
    /// The published range, degC.
    pub published_c: (f64, f64),
    /// The rows whose printed range applies.
    pub rows: Vec<String>,
}

impl fmt::Display for OutsidePublishedRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} C is outside the published range {} to {} C of the rows {}; no coefficient is returned (use diffusivity_extrapolated to obtain a marked extrapolation)",
            self.t_celsius,
            self.published_c.0,
            self.published_c.1,
            self.rows.join(", ")
        )
    }
}

impl std::error::Error for OutsidePublishedRange {}

/// The scaling of a kernel coefficient of the sensitivity case.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KernelScale {
    /// Kernel radius to which the published coefficient refers, um.
    pub reference_radius_um: f64,
    /// AGR-1 kernel radius, um (half the mean kernel diameter of Collin 2015 Table A-1).
    pub agr1_radius_um: f64,
    /// The factor (agr1 / reference)^2.
    pub factor: f64,
}

/// A coefficient evaluated at a temperature, with the position of the temperature in the range.
#[derive(Debug, Clone, PartialEq)]
pub struct Evaluated {
    /// The coefficient, m2 s-1, and whether it is a value or an upper bound.
    pub diffusivity: Diffusivity,
    /// Inside the printed range, or beyond it (an extrapolation), or no range is printed.
    pub range: RangeStatus,
}

/// Where a temperature lies against the printed range.
#[derive(Debug, Clone, PartialEq)]
pub enum RangeStatus {
    /// The rows print no temperature range.
    NoRangePrinted,
    /// Inside the printed range.
    Inside,
    /// Outside: the value is an extrapolation of the published correlation.
    Extrapolated {
        /// The printed range, degC.
        published_c: (f64, f64),
    },
}

/// The published correlation chosen for a nuclide, a layer and a phase.
#[derive(Debug, Clone)]
pub struct Selection<'a> {
    /// Element symbol.
    pub element: String,
    /// Layer asked for.
    pub layer: ParticleLayer,
    /// Phase asked for.
    pub phase: Phase,
    /// Variant.
    pub variant: Variant,
    /// The terms to add (one or two rows of `arrhenius.csv`), in term order.
    pub rows: Vec<&'a ArrheniusRow>,
    /// The class of owner ruling 16 (all rows of a selection share it).
    pub class: CoefficientClass,
    /// Short citation of the source of the rows.
    pub source: String,
    /// Location of the rows in the source.
    pub source_location: String,
    /// Printed temperature range common to the rows, degC (`None` when none is printed).
    pub published_range_c: Option<(f64, f64)>,
    /// Kernel scaling, for the sensitivity case on the kernel layer.
    pub kernel_scale: Option<KernelScale>,
    /// Statements that must be shown wherever the coefficient is shown.
    pub labels: Vec<String>,
}

fn range_of(row: &ArrheniusRow) -> Option<(f64, f64)> {
    match (&row.t_min_c, &row.t_max_c) {
        (Some(a), Some(b)) => Some((a.value, b.value)),
        _ => None,
    }
}

impl Selection<'_> {
    /// Ids of the rows.
    pub fn row_ids(&self) -> Vec<&str> {
        self.rows.iter().map(|r| r.id.as_str()).collect()
    }

    /// Number of terms (1 or 2).
    pub fn terms(&self) -> usize {
        self.rows.len()
    }

    fn raw(&self, t_celsius: f64) -> Diffusivity {
        let mut total = 0.0;
        let mut kind = BoundKind::PointValue;
        for r in &self.rows {
            let rg = match &r.gas_constant {
                crate::arrhenius::GasConstant::Printed(p) => p.value,
                crate::arrhenius::GasConstant::NotPrinted => GAS_CONSTANT_ASSUMED,
            };
            // fluence 0: the fluence factor is never applied (decision 39, GAP-04)
            if let Some(d) = r.diffusivity(t_celsius + 273.15, rg, 0.0) {
                total += d.m2_per_s;
                if d.kind == BoundKind::UpperBound {
                    kind = BoundKind::UpperBound;
                }
            }
        }
        let scale = self.kernel_scale.map_or(1.0, |k| k.factor);
        Diffusivity {
            m2_per_s: total * scale,
            kind,
        }
    }

    /// The coefficient at `t_celsius`, or a refusal when the temperature is outside the printed
    /// range of the rows.
    pub fn diffusivity_at(&self, t_celsius: f64) -> Result<Diffusivity, OutsidePublishedRange> {
        if let Some((lo, hi)) = self.published_range_c {
            if t_celsius < lo || t_celsius > hi {
                return Err(OutsidePublishedRange {
                    t_celsius,
                    published_c: (lo, hi),
                    rows: self.rows.iter().map(|r| r.id.clone()).collect(),
                });
            }
        }
        Ok(self.raw(t_celsius))
    }

    /// The coefficient at `t_celsius` with the position against the printed range stated.
    pub fn diffusivity_extrapolated(&self, t_celsius: f64) -> Evaluated {
        let range = match self.published_range_c {
            None => RangeStatus::NoRangePrinted,
            Some((lo, hi)) if t_celsius >= lo && t_celsius <= hi => RangeStatus::Inside,
            Some(p) => RangeStatus::Extrapolated { published_c: p },
        };
        Evaluated {
            diffusivity: self.raw(t_celsius),
            range,
        }
    }
}

/// One entry of the default table: the rows of a published set for (species, layer, phase).
struct Entry {
    elements: &'static [&'static str],
    layer: DiffusionLayer,
    phase: Phase,
    ids: &'static [&'static str],
    labels: &'static [&'static str],
}

const KERNEL_LABELS: &[&str] = &[
    "UO2 coefficient used for the UCO kernel; no UCO measurement exists",
    "referred to a 250 um kernel radius and applied unscaled, as the published AGR-1 modelling does",
];
const PYC_LABELS: &[&str] =
    &["one pyrocarbon set for the inner and the outer pyrocarbon (register GAP-02)"];
const CS_SIC_LABELS: &[&str] =
    &["fluence factor not applied; correlation form under source review"];
const AG_SIC_LABELS: &[&str] = &[
    "the published correlation over-estimates the silver release of AGR-1 (factor about 1000 at 1600 to 1700 C and 100 at 1800 C, Collin 2016)",
];
const BUFFER_LABELS: &[&str] = &["modelling convention: the buffer is taken as non-retentive"];
const KR_PYC_IRR_LABELS: &[&str] = &[
    "heating-test set applied in the irradiation phase: no irradiation-phase Kr row exists; the second term is below 0.1 percent of the total up to 1800 C",
    "one pyrocarbon set for the inner and the outer pyrocarbon (register GAP-02)",
];
const KR_SIC_LABELS: &[&str] = &[
    "Xe-in-SiC coefficient applied to Kr; printed range 1400 to 1750 C, the 1800 C heating tests lie above it",
];
const KR_KERNEL_IRR_LABELS: &[&str] = &[
    "UO2 coefficient for normal operating conditions (TECDOC-978 Table A-1); no UCO measurement exists",
    "referred to a 250 um kernel radius and applied unscaled",
];

const TABLE: &[Entry] = &[
    // ---- irradiation phase: published single-term set of Collin 2015 Table 2
    Entry {
        elements: &["Ag"],
        layer: DiffusionLayer::Kernel,
        phase: Phase::Irradiation,
        ids: &["C15-01"],
        labels: KERNEL_LABELS,
    },
    Entry {
        elements: &["Ag"],
        layer: DiffusionLayer::PyC,
        phase: Phase::Irradiation,
        ids: &["C15-02"],
        labels: PYC_LABELS,
    },
    Entry {
        elements: &["Ag"],
        layer: DiffusionLayer::SiC,
        phase: Phase::Irradiation,
        ids: &["C15-03"],
        labels: AG_SIC_LABELS,
    },
    Entry {
        elements: &["Cs"],
        layer: DiffusionLayer::Kernel,
        phase: Phase::Irradiation,
        ids: &["C15-05"],
        labels: KERNEL_LABELS,
    },
    Entry {
        elements: &["Cs"],
        layer: DiffusionLayer::PyC,
        phase: Phase::Irradiation,
        ids: &["C15-06"],
        labels: PYC_LABELS,
    },
    Entry {
        elements: &["Cs"],
        layer: DiffusionLayer::SiC,
        phase: Phase::Irradiation,
        ids: &["C15-07"],
        labels: CS_SIC_LABELS,
    },
    Entry {
        elements: &["Sr"],
        layer: DiffusionLayer::Kernel,
        phase: Phase::Irradiation,
        ids: &["C15-09"],
        labels: KERNEL_LABELS,
    },
    Entry {
        elements: &["Sr"],
        layer: DiffusionLayer::PyC,
        phase: Phase::Irradiation,
        ids: &["C15-10"],
        labels: PYC_LABELS,
    },
    Entry {
        elements: &["Sr"],
        layer: DiffusionLayer::SiC,
        phase: Phase::Irradiation,
        ids: &["C15-11"],
        labels: &[],
    },
    Entry {
        elements: &["Kr"],
        layer: DiffusionLayer::Kernel,
        phase: Phase::Irradiation,
        ids: &["A1-05"],
        labels: KR_KERNEL_IRR_LABELS,
    },
    Entry {
        elements: &["Kr"],
        layer: DiffusionLayer::PyC,
        phase: Phase::Irradiation,
        ids: &["C16-18", "C16-19"],
        labels: KR_PYC_IRR_LABELS,
    },
    Entry {
        elements: &["Ag", "Cs", "Sr", "Kr"],
        layer: DiffusionLayer::Buffer,
        phase: Phase::Irradiation,
        ids: &["C15-13"],
        labels: BUFFER_LABELS,
    },
    // ---- heating tests: published set of Collin 2016 Table 2, second terms where printed
    Entry {
        elements: &["Ag"],
        layer: DiffusionLayer::Kernel,
        phase: Phase::HeatingTest,
        ids: &["C16-01"],
        labels: KERNEL_LABELS,
    },
    Entry {
        elements: &["Ag"],
        layer: DiffusionLayer::PyC,
        phase: Phase::HeatingTest,
        ids: &["C16-02"],
        labels: PYC_LABELS,
    },
    Entry {
        elements: &["Ag"],
        layer: DiffusionLayer::SiC,
        phase: Phase::HeatingTest,
        ids: &["C16-03"],
        labels: AG_SIC_LABELS,
    },
    Entry {
        elements: &["Cs"],
        layer: DiffusionLayer::Kernel,
        phase: Phase::HeatingTest,
        ids: &["C16-05", "C16-06"],
        labels: KERNEL_LABELS,
    },
    Entry {
        elements: &["Cs"],
        layer: DiffusionLayer::PyC,
        phase: Phase::HeatingTest,
        ids: &["C16-07"],
        labels: PYC_LABELS,
    },
    Entry {
        elements: &["Cs"],
        layer: DiffusionLayer::SiC,
        phase: Phase::HeatingTest,
        ids: &["C16-08", "C16-09"],
        labels: CS_SIC_LABELS,
    },
    Entry {
        elements: &["Sr"],
        layer: DiffusionLayer::Kernel,
        phase: Phase::HeatingTest,
        ids: &["C16-11"],
        labels: KERNEL_LABELS,
    },
    Entry {
        elements: &["Sr"],
        layer: DiffusionLayer::PyC,
        phase: Phase::HeatingTest,
        ids: &["C16-12"],
        labels: PYC_LABELS,
    },
    Entry {
        elements: &["Sr"],
        layer: DiffusionLayer::SiC,
        phase: Phase::HeatingTest,
        ids: &["C16-13", "C16-14"],
        labels: &[],
    },
    Entry {
        elements: &["Kr"],
        layer: DiffusionLayer::Kernel,
        phase: Phase::HeatingTest,
        ids: &["C16-16", "C16-17"],
        labels: KERNEL_LABELS,
    },
    Entry {
        elements: &["Kr"],
        layer: DiffusionLayer::PyC,
        phase: Phase::HeatingTest,
        ids: &["C16-18", "C16-19"],
        labels: PYC_LABELS,
    },
    Entry {
        elements: &["Kr"],
        layer: DiffusionLayer::SiC,
        phase: Phase::HeatingTest,
        ids: &["C16-20"],
        labels: KR_SIC_LABELS,
    },
    Entry {
        elements: &["Ag", "Cs", "Sr", "Kr"],
        layer: DiffusionLayer::Buffer,
        phase: Phase::HeatingTest,
        ids: &["C16-22"],
        labels: BUFFER_LABELS,
    },
];

/// The Schappel (2020) rows offered as the alternative for Ag, Cs and Sr in SiC.
const SCHAPPEL: &[(&str, &str)] = &[("Ag", "S20-01"), ("Cs", "S20-02"), ("Sr", "S20-03")];

const SCHAPPEL_LABELS: &[&str] = &[
    "AGR-1-specific fitted correlation, labelled alternative: fitted to the 1600 and 1800 C heating tests only, never the default",
    "disagrees with the correction factors of Collin 2016 by 4 to 220 times",
];

/// The selector: the diffusivity register and the irradiation conditions (for the kernel radius).
#[derive(Debug, Clone, Copy)]
pub struct Selector<'a> {
    /// The diffusion coefficient rows.
    pub arrhenius: &'a Arrhenius,
    /// The irradiation conditions (kernel diameter of AGR-1).
    pub irradiation: &'a Irradiation,
}

impl<'a> Selector<'a> {
    /// Select the published correlation for `nuclide` (an element symbol or a nuclide name such as
    /// `Cs-137`; the element decides), `layer`, `phase` and `variant`.
    ///
    /// Returns the rows, their class and source, the printed temperature range and the labels. A
    /// combination for which no published coefficient exists is `Err(Unavailable)`; nothing is
    /// invented in its place.
    pub fn select(
        &self,
        nuclide: &str,
        layer: ParticleLayer,
        phase: Phase,
        variant: Variant,
    ) -> Result<Selection<'a>, Unavailable> {
        let element = nuclide.split('-').next().unwrap_or(nuclide);
        let unavailable = |reason: String| Err(Unavailable { reason });
        if element == "Eu" {
            return unavailable("no diffusion coefficient of Eu in any source opened (register GAP-26 and the diffusivity research report)".into());
        }
        let reg_layer = layer.register_layer();
        let requested = element.to_string();
        let mut xe_label = None;
        let element = if element == "Xe" {
            // The Kr rows of the modelling sets are printed for Kr only (Collin 2015, 2016), except
            // the UO2 kernel row A1-05 of TECDOC-978, which is printed for I, Kr and Xe.
            if reg_layer == DiffusionLayer::Kernel && phase == Phase::Irradiation {
                xe_label = Some(
                    "Xe: the row A1-05 is printed for I, Kr and Xe (TECDOC-978 Table A-1), so the Kr selection applies to Xe"
                        .to_string(),
                );
                "Kr"
            } else {
                return unavailable(format!(
                    "no Xe row is offered for the {reg_layer:?} layer in the {} phase: the Kr rows of the published AGR-1 modelling are printed for Kr only, and only the UO2 kernel row A1-05 of the irradiation phase is printed for Xe as well",
                    phase.as_str()
                ));
            }
        } else {
            element
        };
        let entry = TABLE
            .iter()
            .find(|e| e.elements.contains(&element) && e.layer == reg_layer && e.phase == phase);
        let Some(entry) = entry else {
            let why = if phase == Phase::Irradiation
                && layer == ParticleLayer::SiC
                && element == "Kr"
            {
                "Kr and Xe in SiC are not used during irradiation (published modelling uses an accident-condition coefficient only for the heating tests)".to_string()
            } else {
                format!(
                    "no published AGR-1 modelling coefficient for {element} in the {reg_layer:?} layer in the {} phase",
                    phase.as_str()
                )
            };
            return unavailable(why);
        };
        let mut ids: Vec<&str> = entry.ids.to_vec();
        let mut labels: Vec<String> = entry.labels.iter().map(|s| s.to_string()).collect();
        let mut kernel_scale = None;
        if let Some(l) = xe_label {
            labels.push(l);
        }
        match variant {
            Variant::Default => {}
            Variant::SchappelFitted => {
                if phase != Phase::HeatingTest {
                    return unavailable("the fitted set of Schappel (2020) refers to the 1600 and 1800 C heating tests only".into());
                }
                if reg_layer != DiffusionLayer::SiC {
                    return unavailable(format!(
                        "the fitted set of Schappel (2020) covers Ag, Cs and Sr in SiC only; it offers nothing for the {reg_layer:?} layer"
                    ));
                }
                if let Some((_, id)) = SCHAPPEL.iter().find(|(e, _)| *e == element) {
                    ids = vec![id];
                    labels = SCHAPPEL_LABELS.iter().map(|s| s.to_string()).collect();
                } else {
                    return unavailable(format!(
                        "Schappel (2020) fitted no {element} coefficient in SiC"
                    ));
                }
            }
            Variant::KernelScaledSensitivity => {
                if reg_layer == DiffusionLayer::Kernel {
                    let ks = self.kernel_scale(ids[0])?;
                    // the published-modelling statement "applied unscaled" is false for this case
                    labels.retain(|l| !l.contains("unscaled"));
                    labels.push(format!(
                        "sensitivity case, not a published coefficient: the published kernel coefficient, referred to a {} um kernel radius, is scaled by ({} um / {} um)^2 = {:.4} (AGR-1 kernel radius over reference radius)",
                        ks.reference_radius_um, ks.agr1_radius_um, ks.reference_radius_um, ks.factor
                    ));
                    kernel_scale = Some(ks);
                }
            }
        }
        let mut rows = Vec::new();
        for id in &ids {
            match self.arrhenius.get(id) {
                Some(r) => rows.push(r),
                None => {
                    return unavailable(format!("the row {id} is not in the diffusivity register"))
                }
            }
        }
        if rows
            .iter()
            .any(|r| matches!(r.gas_constant, crate::arrhenius::GasConstant::NotPrinted))
        {
            labels.push(format!(
                "R not printed by the source; {GAS_CONSTANT_ASSUMED} J/(mol K) used"
            ));
        }
        let class = rows[0].coefficient_class;
        let range =
            rows.iter()
                .filter_map(|r| range_of(r))
                .fold(None, |acc: Option<(f64, f64)>, r| {
                    Some(match acc {
                        None => r,
                        Some(a) => (a.0.max(r.0), a.1.min(r.1)),
                    })
                });
        Ok(Selection {
            element: requested,
            layer,
            phase,
            variant,
            class,
            source: rows[0].source.clone(),
            source_location: rows[0].source_location.clone(),
            published_range_c: range,
            kernel_scale,
            labels,
            rows,
        })
    }

    fn kernel_scale(&self, id: &str) -> Result<KernelScale, Unavailable> {
        let row = self.arrhenius.get(id).ok_or_else(|| Unavailable {
            reason: format!("the row {id} is not in the diffusivity register"),
        })?;
        let reference = row
            .reference_radius_um
            .as_ref()
            .ok_or_else(|| Unavailable {
                reason: format!("the row {id} has no reference kernel radius"),
            })?;
        let d = self
            .irradiation
            .condition("kernel diameter")
            .map_err(|e| Unavailable {
                reason: e.to_string(),
            })?;
        let d_um = match &d.value {
            ConditionValue::Number(p) => p.value,
            ConditionValue::Text(_) => {
                return Err(Unavailable {
                    reason: "the kernel diameter is not a number".into(),
                })
            }
        };
        let r = d_um / 2.0;
        Ok(KernelScale {
            reference_radius_um: reference.value,
            agr1_radius_um: r,
            factor: (r / reference.value).powi(2),
        })
    }
}

/// True when the row ties itself to a phase that differs from `phase` (a check helper).
pub fn row_phase_conflicts(row: &ArrheniusRow, phase: Phase) -> bool {
    !matches!(
        (row.phase_of_use, phase),
        (PhaseOfUse::NotSpecified, _)
            | (PhaseOfUse::Irradiation, Phase::Irradiation)
            | (PhaseOfUse::HeatingTest, Phase::HeatingTest)
    )
}

/// Every (element, layer, phase) of the default table, for tests and for listing what exists.
pub fn default_entries() -> Vec<(&'static str, DiffusionLayer, Phase, &'static [&'static str])> {
    let mut v = Vec::new();
    for e in TABLE {
        for el in e.elements {
            v.push((*el, e.layer, e.phase, e.ids));
        }
    }
    v
}
