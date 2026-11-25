//! What the comparison takes from the registers: the registered geometry, the registered
//! diffusion coefficients per layer, and the registered time histories of the tests.
//!
//! Nothing is invented here. A coefficient is the sum of the register rows named by id, evaluated
//! at the test temperature (decision 16); a radius is built from the register's diameter and
//! thicknesses; a hold duration is the one the temperature register states.

use std::path::Path;

use strato::physics::arrhenius::GasConstant as SiGas;
use strato_data::arrhenius::{ArrheniusRow, BoundKind, DiffusionLayer, Form, GasConstant};
use strato_data::geometry::GeometryValue;
use strato_data::table::Vocabulary;
use strato_data::temperature::TemperaturePoint;
use strato_data::Register;

/// The registered geometry used for every run: the measured AGR-1 baseline particle.
pub const GEOMETRY_DESIGN: &str = "AGR1-B-M16";
/// The registered set of diffusion coefficients of the heating tests (Collin 2016, Table 2).
pub const HEATING_SET: &str = "C16-T2";
/// The registered buffer coefficient of the heating-test set.
pub const BUFFER_SET: &str = "C16-BUF";

/// The three elements for which the register holds a complete heating-test coefficient set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Species {
    /// Silver.
    Ag,
    /// Caesium.
    Cs,
    /// Strontium.
    Sr,
}

impl Species {
    /// The element label of the register rows.
    pub fn label(self) -> &'static str {
        match self {
            Species::Ag => "Ag",
            Species::Cs => "Cs",
            Species::Sr => "Sr",
        }
    }

    /// The nuclide the figures and files name for this element.
    pub fn nuclide(self) -> &'static str {
        match self {
            Species::Ag => "Ag-110m",
            Species::Cs => "Cs-134",
            Species::Sr => "Sr-90",
        }
    }

    /// The element of a nuclide or element label.
    pub fn of_element(element: &str) -> Option<Species> {
        match element {
            "Ag" => Some(Species::Ag),
            "Cs" => Some(Species::Cs),
            "Sr" => Some(Species::Sr),
            _ => None,
        }
    }

    /// All three, in a fixed order.
    pub const ALL: [Species; 3] = [Species::Ag, Species::Cs, Species::Sr];
}

/// The five radii in metre and the register rows they come from.
#[derive(Debug, Clone)]
pub struct Radii {
    /// Outer radius of kernel, buffer, IPyC, SiC and OPyC, in metre.
    pub outer_m: [f64; 5],
    /// Ids of the geometry rows used.
    pub row_ids: Vec<String>,
}

/// Build the radii of [`GEOMETRY_DESIGN`]: kernel radius is half the registered diameter, each
/// further radius adds the registered thickness.
pub fn radii(reg: &Register) -> Result<Radii, String> {
    let mut diameter_um = None;
    let mut thickness_um = [None; 4];
    let mut ids = Vec::new();
    for row in reg
        .geometry
        .rows
        .iter()
        .filter(|r| r.design_id == GEOMETRY_DESIGN)
    {
        let value = match &row.value {
            GeometryValue::Point(p) => p.value,
            _ => continue,
        };
        if row.unit.as_str() != "um" {
            continue;
        }
        match (row.layer.as_str(), row.quantity.as_str()) {
            ("kernel", "diameter") => {
                diameter_um = Some(value);
                ids.push(row.id.clone());
            }
            ("buffer", "thickness") => {
                thickness_um[0] = Some(value);
                ids.push(row.id.clone());
            }
            ("IPyC", "thickness") => {
                thickness_um[1] = Some(value);
                ids.push(row.id.clone());
            }
            ("SiC", "thickness") => {
                thickness_um[2] = Some(value);
                ids.push(row.id.clone());
            }
            ("OPyC", "thickness") => {
                thickness_um[3] = Some(value);
                ids.push(row.id.clone());
            }
            _ => {}
        }
    }
    let d = diameter_um.ok_or("the geometry register has no kernel diameter for the design")?;
    let mut outer = [0.0; 5];
    outer[0] = d / 2.0 * 1e-6;
    for k in 0..4 {
        let t = thickness_um[k].ok_or("the geometry register lacks a coating thickness")?;
        outer[k + 1] = outer[k] + t * 1e-6;
    }
    Ok(Radii {
        outer_m: outer,
        row_ids: ids,
    })
}

/// One selection of register rows that gives the five diffusion coefficients of a particle.
#[derive(Debug, Clone)]
pub struct DiffusivitySet {
    /// Short identifier of the variant.
    pub id: String,
    /// What the variant is.
    pub label: String,
    /// Register row ids per region: kernel, buffer, IPyC, SiC, OPyC.
    pub ids: [Vec<String>; 5],
    /// Factor applied to the kernel coefficient (1 unless the variant is the kernel-radius case).
    pub kernel_scale: f64,
    /// Coefficient class of the SiC rows, as the register states it.
    pub sic_class: String,
    /// Phase of use of the SiC rows, as the register states it.
    pub sic_phase: String,
}

fn rows_of<'a>(
    reg: &'a Register,
    set: &str,
    species: Species,
    layer: DiffusionLayer,
) -> Vec<&'a ArrheniusRow> {
    reg.arrhenius
        .rows
        .iter()
        .filter(|r| r.set_id == set && r.species_text == species.label() && r.layer == layer)
        .collect()
}

fn ids(rows: Vec<&ArrheniusRow>) -> Vec<String> {
    rows.into_iter().map(|r| r.id.clone()).collect()
}

/// The default set of an element: the published heating-test correlation as printed, kernel and
/// PyC and SiC by element, buffer common to all species (decision 39: heating-test runs use the
/// published two-term rows).
pub fn default_set(reg: &Register, species: Species) -> Result<DiffusivitySet, String> {
    let kernel = rows_of(reg, HEATING_SET, species, DiffusionLayer::Kernel);
    let pyc = rows_of(reg, HEATING_SET, species, DiffusionLayer::PyC);
    let sic = rows_of(reg, HEATING_SET, species, DiffusionLayer::SiC);
    let buffer: Vec<&ArrheniusRow> = reg
        .arrhenius
        .rows
        .iter()
        .filter(|r| r.set_id == BUFFER_SET && r.layer == DiffusionLayer::Buffer)
        .collect();
    if kernel.is_empty() || pyc.is_empty() || sic.is_empty() || buffer.len() != 1 {
        return Err(format!(
            "the register lacks a complete {HEATING_SET} set for {}",
            species.label()
        ));
    }
    let sic_class = sic[0].coefficient_class.as_str().to_string();
    let sic_phase = sic[0].phase_of_use.as_str().to_string();
    let pyc_ids = ids(pyc);
    Ok(DiffusivitySet {
        id: format!("{HEATING_SET}-published"),
        label: format!(
            "published heating-test correlation as printed ({HEATING_SET} and {BUFFER_SET})"
        ),
        ids: [ids(kernel), ids(buffer), pyc_ids.clone(), ids(sic), pyc_ids],
        kernel_scale: 1.0,
        sic_class,
        sic_phase,
    })
}

/// The labelled alternatives of an element (sensitivity cases, never defaults): the AGR-1 fitted
/// SiC rows of Schappel 2020 (decision 40), other registered heating-test SiC rows, and the
/// kernel-radius variant of decision 41.
pub fn variant_sets(reg: &Register, species: Species) -> Result<Vec<DiffusivitySet>, String> {
    let base = default_set(reg, species)?;
    let mut out = vec![base.clone()];
    // Kernel coefficients of the register refer to a reference radius (um); the variant scales them
    // by (r_kernel / r_reference)^2 (decision 41). The factor comes from the registers.
    let radii = radii(reg)?;
    let reference_um = reg
        .arrhenius
        .rows
        .iter()
        .find(|r| base.ids[0].contains(&r.id))
        .and_then(|r| r.reference_radius_um.as_ref())
        .map(|p| p.value)
        .ok_or("the kernel rows carry no reference radius")?;
    let ratio = radii.outer_m[0] * 1e6 / reference_um;
    let mut scaled = base.clone();
    scaled.id = "kernel-radius-scaled".to_string();
    scaled.label = format!(
        "kernel coefficient scaled by (kernel radius / {reference_um} um)^2 = {:.3} (decision 41)",
        ratio * ratio
    );
    scaled.kernel_scale = ratio * ratio;
    out.push(scaled);
    // Fitted SiC rows (form "1T (fitted)") of each Schappel set.
    for set in ["S20-T6", "S20-T3"] {
        let rows: Vec<&ArrheniusRow> = reg
            .arrhenius
            .rows
            .iter()
            .filter(|r| {
                r.set_id == set
                    && r.species_text == species.label()
                    && r.layer == DiffusionLayer::SiC
                    && r.form == Form::Fitted
            })
            .collect();
        if rows.len() == 1 {
            let mut v = base.clone();
            v.id = format!("{set}-fitted-SiC");
            v.label = format!(
                "SiC replaced by the AGR-1 fitted row {} ({set}); labelled alternative, not the default",
                rows[0].id
            );
            v.sic_class = rows[0].coefficient_class.as_str().to_string();
            v.sic_phase = rows[0].phase_of_use.as_str().to_string();
            v.ids[3] = vec![rows[0].id.clone()];
            out.push(v);
        }
    }
    // Other registered heating-test or wide-range SiC rows.
    let extra: &[&str] = match species {
        Species::Ag => &["A3-21"],
        Species::Cs => &["A3-14"],
        Species::Sr => &["A3-15"],
    };
    for id in extra {
        if let Some(row) = reg.arrhenius.get(id) {
            let mut v = base.clone();
            v.id = format!("{id}-SiC");
            v.label = format!(
                "SiC replaced by register row {id} ({}; printed range {}-{} degC)",
                row.origin,
                row.t_min_c
                    .as_ref()
                    .map_or("none".into(), |p| p.text.clone()),
                row.t_max_c
                    .as_ref()
                    .map_or("none".into(), |p| p.text.clone()),
            );
            v.sic_class = row.coefficient_class.as_str().to_string();
            v.sic_phase = row.phase_of_use.as_str().to_string();
            v.ids[3] = vec![(*id).to_string()];
            out.push(v);
        }
    }
    Ok(out)
}

/// The five coefficients in m2/s at a temperature, with a note on the printed temperature ranges.
#[derive(Debug, Clone)]
pub struct Evaluated {
    /// Kernel, buffer, IPyC, SiC, OPyC, in m2/s.
    pub d_m2_s: [f64; 5],
    /// True if some row evaluated has a printed range that does not contain the temperature.
    pub outside_printed_range: bool,
    /// True if some row evaluated prints a range at all.
    pub any_printed_range: bool,
    /// True if a coefficient is an upper bound (never for the default sets).
    pub any_upper_bound: bool,
}

/// Evaluate a set at a temperature in degC.
pub fn evaluate(reg: &Register, set: &DiffusivitySet, t_c: f64) -> Result<Evaluated, String> {
    let mut d = [0.0; 5];
    let (mut outside, mut any_range, mut any_ub) = (false, false, false);
    for (k, row_ids) in set.ids.iter().enumerate() {
        let first = reg
            .arrhenius
            .get(&row_ids[0])
            .ok_or("unknown diffusivity row id")?;
        let r_gas = match &first.gas_constant {
            GasConstant::Printed(p) => p.value,
            GasConstant::NotPrinted => SiGas::SI.value(),
        };
        let refs: Vec<&str> = row_ids.iter().map(String::as_str).collect();
        let v = reg
            .arrhenius
            .sum_of_terms(&refs, t_c, r_gas, 0.0)
            .ok_or("a diffusivity row is instantaneous or unknown")?;
        if v.kind == BoundKind::UpperBound {
            any_ub = true;
        }
        d[k] = v.m2_per_s;
        for id in row_ids {
            let row = reg.arrhenius.get(id).ok_or("unknown diffusivity row id")?;
            if let (Some(lo), Some(hi)) = (&row.t_min_c, &row.t_max_c) {
                any_range = true;
                if t_c < lo.value || t_c > hi.value {
                    outside = true;
                }
            }
        }
    }
    d[0] *= set.kernel_scale;
    for v in d {
        if !(v.is_finite() && v > 0.0) {
            return Err("a diffusion coefficient is not a positive finite number".into());
        }
    }
    Ok(Evaluated {
        d_m2_s: d,
        outside_printed_range: outside,
        any_printed_range: any_range,
        any_upper_bound: any_ub,
    })
}

/// A hold duration of a registered history and where it comes from.
#[derive(Debug, Clone, PartialEq)]
pub struct Hold {
    /// Hold duration at the test temperature, hours.
    pub hours: f64,
    /// Start of the hold on the history clock, hours (None if the register gives no cumulative time).
    pub starts_at_h: Option<f64>,
    /// Which history and rows it comes from.
    pub source: String,
}

/// The hold at `t_c` in the history `history_id`, if the register has one.
pub fn hold_of(reg: &Register, history_id: &str, t_c: f64) -> Option<Hold> {
    let rows: Vec<_> = reg
        .temperature
        .rows
        .iter()
        .filter(|r| r.history_id == history_id)
        .collect();
    for r in rows {
        let Some(hold_cell) = r.hold_h.as_ref() else {
            continue;
        };
        let hold = hold_cell.value;
        let at = match r.t_end.as_ref() {
            Some(TemperaturePoint::Celsius(p)) => p.value,
            _ => continue,
        };
        if (at - t_c).abs() < 1e-9 && hold > 0.0 {
            let start = r.cumulative_end_h_calc.as_ref().map(|c| c.value - hold);
            return Some(Hold {
                hours: hold,
                starts_at_h: start,
                source: format!("{} (history {history_id})", r.id),
            });
        }
    }
    None
}

/// The registered nominal hold of the safety test modelled at `t_c` ("AGR1-ST-C16-<T>").
pub fn nominal_hold(reg: &Register, t_c: f64) -> Option<Hold> {
    hold_of(reg, &format!("AGR1-ST-C16-{}", t_c.round() as i64), t_c)
}

/// Load the register from the data folder of a checkout.
pub fn load(repo: &Path) -> Result<Register, String> {
    Register::load(&repo.join("data")).map_err(|e| e.to_string())
}
