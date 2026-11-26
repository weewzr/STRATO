//! The registered geometry of the particle: the five outer radii of the measured AGR-1 baseline
//! design, built from the register's kernel diameter and coating thicknesses.
//!
//! Moved unchanged from `strato-compare` so that the results pipeline, the comparison and the
//! WebAssembly wrapper build the radii with one function.

use strato_data::geometry::GeometryValue;
use strato_data::table::Vocabulary;
use strato_data::Register;

/// The registered geometry used for every run: the measured AGR-1 baseline particle.
pub const GEOMETRY_DESIGN: &str = "AGR1-B-M16";
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
