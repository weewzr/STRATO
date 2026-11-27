//! What the comparison takes from the registers: the registered geometry, the registered
//! diffusion coefficients per layer, and the registered time histories of the tests.
//!
//! Nothing is invented here. A coefficient is the sum of the register rows named by id, evaluated
//! at the test temperature (decision 16); a radius is built from the register's diameter and
//! thicknesses; a hold duration is the one the temperature register states.

use std::path::Path;

use strato_data::arrhenius::{BoundKind, DiffusionLayer, Form, GasConstant};
use strato_data::select::{
    ParticleLayer, Phase, Selection, Selector, Variant, GAS_CONSTANT_ASSUMED,
};
use strato_data::table::Vocabulary;
use strato_data::temperature::TemperaturePoint;
use strato_data::Register;

// The registered radii are built by `strato-scenarios`, shared with the WebAssembly wrapper.
pub use strato_scenarios::geometry::{radii, Radii, GEOMETRY_DESIGN};

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
    /// Statements the selection function attaches to the coefficients (decision 47), per layer.
    pub labels: Vec<String>,
    /// Coefficient class (owner ruling 16) and phase of use of the rows of each region, as the
    /// register states them: kernel, buffer, IPyC, SiC, OPyC.
    pub classes: [(String, String); 5],
}

const LAYERS: [(&str, ParticleLayer); 5] = [
    ("kernel", ParticleLayer::Kernel),
    ("buffer", ParticleLayer::Buffer),
    ("IPyC", ParticleLayer::IPyC),
    ("SiC", ParticleLayer::SiC),
    ("OPyC", ParticleLayer::OPyC),
];

fn selector(reg: &Register) -> Selector<'_> {
    Selector {
        arrhenius: &reg.arrhenius,
        irradiation: &reg.irradiation,
    }
}

fn select_layer<'a>(
    reg: &'a Register,
    species: Species,
    layer: ParticleLayer,
    variant: Variant,
) -> Result<Selection<'a>, String> {
    selector(reg)
        .select(species.label(), layer, Phase::HeatingTest, variant)
        .map_err(|e| e.to_string())
}

/// Proper unit typography for a label that the register or the selection function wrote in ASCII:
/// the unit "um" becomes the micrometre symbol "µm" and "degC" the degree Celsius symbol "°C"
/// (decision 30). A lone "C" that follows a number and a space is a temperature unit and becomes
/// the degree Celsius symbol too, and "^2" becomes the superscript two. Only whole words are
/// changed.
pub fn typography(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        let after_number_and_space = {
            let mut back = out.chars().rev();
            back.next() == Some(' ') && back.next().is_some_and(|c| c.is_ascii_digit())
        };
        match word.as_str() {
            "um" => out.push_str("\u{b5}m"),
            "degC" => out.push_str("\u{b0}C"),
            "C" if after_number_and_space => out.push_str("\u{b0}C"),
            other => out.push_str(other),
        }
        word.clear();
    };
    for ch in text.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            word.push(ch);
        } else {
            flush(&mut word, &mut out);
            out.push(ch);
        }
    }
    flush(&mut word, &mut out);
    out.replace("^2", "\u{b2}")
}

/// The default set of an element, chosen by the selection function of `strato-data` (decisions 39,
/// 40 and 47): the published heating-test correlation as printed, with the second term where the
/// source prints one.
pub fn default_set(reg: &Register, species: Species) -> Result<DiffusivitySet, String> {
    let mut ids: [Vec<String>; 5] = Default::default();
    let mut labels = Vec::new();
    let mut sic_class = String::new();
    let mut sic_phase = String::new();
    let mut classes: [(String, String); 5] = Default::default();
    for (k, (name, layer)) in LAYERS.iter().enumerate() {
        let sel = select_layer(reg, species, *layer, Variant::Default)?;
        ids[k] = sel.row_ids().iter().map(|s| s.to_string()).collect();
        classes[k] = (
            sel.class.as_str().to_string(),
            sel.rows[0].phase_of_use.as_str().to_string(),
        );
        labels.extend(
            sel.labels
                .iter()
                .map(|l| typography(&format!("{name}: {l}"))),
        );
        if *layer == ParticleLayer::SiC {
            sic_class = sel.class.as_str().to_string();
            sic_phase = sel.rows[0].phase_of_use.as_str().to_string();
        }
    }
    Ok(DiffusivitySet {
        id: format!("{HEATING_SET}-published"),
        label: format!(
            "published heating-test correlation as printed (Collin 2016 Table 2, rows {HEATING_SET} and {BUFFER_SET}), chosen by the strato-data selection function"
        ),
        ids,
        kernel_scale: 1.0,
        sic_class,
        sic_phase,
        labels,
        classes,
    })
}

/// The labelled alternatives of an element (sensitivity cases, never defaults): the kernel-radius
/// variant of decision 41 and the AGR-1 fitted SiC row of Schappel 2020 (decision 40), both from
/// the selection function, and further registered SiC rows named by id.
pub fn variant_sets(reg: &Register, species: Species) -> Result<Vec<DiffusivitySet>, String> {
    let base = default_set(reg, species)?;
    let mut out = vec![base.clone()];
    let kernel = select_layer(
        reg,
        species,
        ParticleLayer::Kernel,
        Variant::KernelScaledSensitivity,
    )?;
    let scale = kernel
        .kernel_scale
        .ok_or("the kernel-scaled selection carries no scale")?;
    let mut scaled = base.clone();
    scaled.id = "kernel-radius-scaled".to_string();
    scaled.label = typography(&format!(
        "kernel coefficient scaled by (kernel radius {} µm / reference radius {} µm)^2 = {:.3} (decision 41; sensitivity case, not a published coefficient)",
        scale.agr1_radius_um, scale.reference_radius_um, scale.factor
    ));
    scaled.kernel_scale = scale.factor;
    out.push(scaled);
    // The AGR-1 fitted SiC row of Schappel 2020, offered by the selection function.
    let fitted = select_layer(reg, species, ParticleLayer::SiC, Variant::SchappelFitted)?;
    let mut v = base.clone();
    v.id = "S20-fitted-SiC".to_string();
    v.label = format!(
        "SiC replaced by the AGR-1 fitted row {} (Schappel 2020, fitted to the 1600 and 1800 °C heating tests); labelled alternative, never the default; it disagrees with Collin's correction factors by 4 to 220 times (decision 40)",
        fitted.row_ids().join("+")
    );
    v.sic_class = fitted.class.as_str().to_string();
    v.sic_phase = fitted.rows[0].phase_of_use.as_str().to_string();
    v.classes[3] = (v.sic_class.clone(), v.sic_phase.clone());
    v.ids[3] = fitted.row_ids().iter().map(|s| s.to_string()).collect();
    out.push(v);
    // The single-particle fit of Schappel 2020 (set S20-T3), registered but not offered by the
    // selection function.
    for row in reg.arrhenius.rows.iter().filter(|r| {
        r.set_id == "S20-T3"
            && r.species_text == species.label()
            && r.layer == DiffusionLayer::SiC
            && r.form == Form::Fitted
    }) {
        let mut v = base.clone();
        v.id = "S20-T3-fitted-SiC".to_string();
        v.label = format!(
            "SiC replaced by the AGR-1 single-particle fit row {} (S20-T3); labelled alternative, not the default",
            row.id
        );
        v.sic_class = row.coefficient_class.as_str().to_string();
        v.sic_phase = row.phase_of_use.as_str().to_string();
        v.classes[3] = (v.sic_class.clone(), v.sic_phase.clone());
        v.ids[3] = vec![row.id.clone()];
        out.push(v);
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
            v.label = typography(&format!(
                "SiC replaced by register row {id} ({}; printed range {}-{} °C)",
                row.origin,
                row.t_min_c
                    .as_ref()
                    .map_or("none".into(), |p| p.text.clone()),
                row.t_max_c
                    .as_ref()
                    .map_or("none".into(), |p| p.text.clone()),
            ));
            v.sic_class = row.coefficient_class.as_str().to_string();
            v.sic_phase = row.phase_of_use.as_str().to_string();
            v.classes[3] = (v.sic_class.clone(), v.sic_phase.clone());
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

/// Evaluate a set at a temperature in °C.
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
            GasConstant::NotPrinted => GAS_CONSTANT_ASSUMED,
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
