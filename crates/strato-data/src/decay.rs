//! `data/nuclear/decay.csv`: ENSDF half-lives, decay constants and production class.
//!
//! A nuclide is either stable (decay constant 0, no half-life) or radioactive (half-life with
//! its unit, half-life in seconds, decay constant, all with uncertainties). The reader refuses a
//! row that mixes the two. The product class is a type, so that an activation product can be told
//! from a direct fission product without reading a comment.

use crate::common::{EvidenceClass, NuclideId, ProductClass, Status};
use crate::error::{DataError, Result};
use crate::string_enum;
use crate::table::{Printed, Row, Table, Vocabulary};

/// Path of the file below the data folder.
pub const FILE: &str = "nuclear/decay.csv";

/// The exact header.
pub const HEADER: [&str; 30] = [
    "nuclide",
    "Z",
    "A",
    "level_keV",
    "Jpi",
    "stable",
    "half_life",
    "unc_half_life",
    "unit",
    "half_life_s",
    "unc_half_life_s",
    "lambda_s-1",
    "unc_lambda_s-1",
    "decay_mode_1",
    "branch_1_pct",
    "unc_branch_1",
    "decay_mode_2",
    "branch_2_pct",
    "unc_branch_2",
    "daughters",
    "production_route",
    "ENSDF_evaluation",
    "ENSDF_cutoff",
    "data_type",
    "evidence_class",
    "source",
    "crosscheck_NuDat3_chart_half_life",
    "product_class",
    "source_location",
    "status",
];

string_enum! {
    /// The unit of the `half_life` column.
    pub enum TimeUnit ("time unit") {
        /// Second.
        Seconds => "s",
        /// Minute.
        Minutes => "m",
        /// Hour.
        Hours => "h",
        /// Day.
        Days => "d",
        /// Year.
        Years => "Y",
    }
}

impl TimeUnit {
    /// Seconds per unit, as the Python audit script `b4_decay_source_pseudocode.py` uses them
    /// (the year is 31556925.97 s, the value the IAEA LiveChart API converts with).
    pub fn seconds(&self) -> f64 {
        match self {
            TimeUnit::Seconds => 1.0,
            TimeUnit::Minutes => 60.0,
            TimeUnit::Hours => 3600.0,
            TimeUnit::Days => 86400.0,
            TimeUnit::Years => 31_556_925.97,
        }
    }
}

string_enum! {
    /// A decay mode as ENSDF writes it.
    pub enum DecayMode ("decay mode") {
        /// Beta minus.
        BetaMinus => "B-",
        /// Isomeric transition.
        IsomericTransition => "IT",
        /// Electron capture.
        ElectronCapture => "EC",
        /// Electron capture and beta plus.
        ElectronCaptureBetaPlus => "EC+B+",
    }
}

/// One decay branch.
#[derive(Debug, Clone, PartialEq)]
pub struct Branch {
    /// The mode.
    pub mode: DecayMode,
    /// Branching, percent.
    pub percent: Printed,
    /// Uncertainty of the branching, percent, when the library gives one.
    pub unc_percent: Option<Printed>,
}

/// Data of a radioactive nuclide.
#[derive(Debug, Clone, PartialEq)]
pub struct Radioactive {
    /// Half-life in `unit`.
    pub half_life: Printed,
    /// Uncertainty of the half-life in units of the last printed digit (ENSDF notation).
    pub unc_half_life_last_digits: Printed,
    /// Unit of `half_life`.
    pub unit: TimeUnit,
    /// Half-life, seconds.
    pub half_life_s: Printed,
    /// Uncertainty of the half-life, seconds.
    pub unc_half_life_s: Printed,
    /// Decay constant, s-1.
    pub lambda_per_s: Printed,
    /// Uncertainty of the decay constant, s-1.
    pub unc_lambda_per_s: Printed,
    /// First decay branch.
    pub branch_1: Branch,
    /// Second decay branch, when there is one.
    pub branch_2: Option<Branch>,
}

/// Stable or radioactive.
#[derive(Debug, Clone, PartialEq)]
pub enum Stability {
    /// Stable: the register prints the half-life `STABLE` and the decay constant 0.
    Stable {
        /// The decay constant cell (must be 0; the audit checks it).
        lambda_per_s: Printed,
    },
    /// Radioactive.
    Radioactive(Box<Radioactive>),
}

/// One row of `decay.csv`.
#[derive(Debug, Clone, PartialEq)]
pub struct DecayRow {
    /// Line in the file.
    pub line: usize,
    /// Nuclide.
    pub nuclide: NuclideId,
    /// Atomic number.
    pub z: u32,
    /// Mass number.
    pub a: u32,
    /// Level energy, keV (0 for the ground state).
    pub level_kev: Printed,
    /// Spin and parity as printed (empty for stable rows).
    pub jpi: String,
    /// Stable or radioactive data.
    pub stability: Stability,
    /// Daughters as printed.
    pub daughters: String,
    /// Production route as printed.
    pub production_route: String,
    /// ENSDF evaluation as printed.
    pub ensdf_evaluation: String,
    /// ENSDF cut-off date as printed (empty for stable rows).
    pub ensdf_cutoff: String,
    /// Data type as printed.
    pub data_type: String,
    /// Evidence class.
    pub evidence_class: EvidenceClass,
    /// Source.
    pub source: String,
    /// NuDat 3 cross-check text.
    pub crosscheck_nudat3: String,
    /// Product class: direct fission product, activation product, daughter, ...
    pub product_class: ProductClass,
    /// Location of the data in the source.
    pub source_location: String,
    /// Status.
    pub status: Status,
}

/// Decay constant `ln 2 / T` for a half-life `value` in `unit`; a non-positive or non-finite
/// half-life is an error (the guard of `b4_decay_source_pseudocode.py`, block A).
pub fn decay_constant(half_life: f64, unit: TimeUnit) -> std::result::Result<f64, String> {
    let t = half_life * unit.seconds();
    if !(t.is_finite() && t > 0.0) {
        return Err("half-life must be positive and finite".to_string());
    }
    Ok(std::f64::consts::LN_2 / t)
}

fn branch(row: &Row<'_>, mode: &str, pct: &str, unc: &str) -> Result<Option<Branch>> {
    let m = row.optional(mode)?;
    let p = row.opt_number(pct)?;
    let u = row.opt_number(unc)?;
    match (m, p) {
        (None, None) => {
            if u.is_some() {
                return Err(row.err(Some(unc), "an uncertainty of a branch that is not given"));
            }
            Ok(None)
        }
        (Some(m), Some(p)) => Ok(Some(Branch {
            mode: DecayMode::from_cell(m).ok_or_else(|| {
                row.err(
                    Some(mode),
                    format!("`{m}` is not a known decay mode: {:?}", DecayMode::ALL),
                )
            })?,
            percent: p,
            unc_percent: u,
        })),
        _ => Err(row.err(
            Some(mode),
            "a decay mode and its branching must be given together",
        )),
    }
}

fn read_row(row: Row<'_>) -> Result<DecayRow> {
    let name = row.required("nuclide")?;
    let nuclide = NuclideId::parse(name).map_err(|m| row.err(Some("nuclide"), m))?;
    let z = row.uint("Z")?;
    let a = row.uint("A")?;
    if a != nuclide.mass {
        return Err(row.err(
            Some("A"),
            format!("A = {a} but the name says mass {}", nuclide.mass),
        ));
    }
    let stable_cell = row.required("stable")?;
    let lambda = row.number("lambda_s-1")?;
    let stability = match stable_cell {
        "yes" => {
            for c in [
                "unc_half_life",
                "unit",
                "half_life_s",
                "unc_half_life_s",
                "unc_lambda_s-1",
                "decay_mode_1",
                "branch_1_pct",
                "decay_mode_2",
                "ENSDF_cutoff",
            ] {
                if !row.text(c)?.is_empty() {
                    return Err(row.err(Some(c), "a stable nuclide has no value in this column"));
                }
            }
            if row.text("half_life")? != "STABLE" {
                return Err(row.err(
                    Some("half_life"),
                    "a stable nuclide prints the half-life STABLE",
                ));
            }
            Stability::Stable {
                lambda_per_s: lambda,
            }
        }
        "no" => {
            let unc_cell = row.required("unc_half_life")?;
            let digits = unc_cell
                .strip_suffix(" (ENSDF last-digit notation)")
                .ok_or_else(|| {
                    row.err(
                        Some("unc_half_life"),
                        "the uncertainty must be in ENSDF last-digit notation",
                    )
                })?;
            let b1 =
                branch(&row, "decay_mode_1", "branch_1_pct", "unc_branch_1")?.ok_or_else(|| {
                    row.err(
                        Some("decay_mode_1"),
                        "a radioactive nuclide needs a decay mode",
                    )
                })?;
            let b2 = branch(&row, "decay_mode_2", "branch_2_pct", "unc_branch_2")?;
            Stability::Radioactive(Box::new(Radioactive {
                half_life: row.number("half_life")?,
                unc_half_life_last_digits: Printed::parse(digits)
                    .map_err(|m| row.err(Some("unc_half_life"), m))?,
                unit: row.vocab("unit")?,
                half_life_s: row.number("half_life_s")?,
                unc_half_life_s: row.number("unc_half_life_s")?,
                lambda_per_s: lambda,
                unc_lambda_per_s: row.number("unc_lambda_s-1")?,
                branch_1: b1,
                branch_2: b2,
            }))
        }
        other => {
            return Err(row.err(
                Some("stable"),
                format!("`{other}` is neither `yes` nor `no`"),
            ))
        }
    };
    let pc = row.required("product_class")?;
    Ok(DecayRow {
        line: row.line(),
        nuclide,
        z,
        a,
        level_kev: row.number("level_keV")?,
        jpi: row.text("Jpi")?.to_string(),
        stability,
        daughters: row.text("daughters")?.to_string(),
        production_route: row.required("production_route")?.to_string(),
        ensdf_evaluation: row.required("ENSDF_evaluation")?.to_string(),
        ensdf_cutoff: row.text("ENSDF_cutoff")?.to_string(),
        data_type: row.required("data_type")?.to_string(),
        evidence_class: row.vocab("evidence_class")?,
        source: row.required("source")?.to_string(),
        crosscheck_nudat3: row
            .required("crosscheck_NuDat3_chart_half_life")?
            .to_string(),
        product_class: ProductClass::parse(pc).map_err(|m| row.err(Some("product_class"), m))?,
        source_location: row.required("source_location")?.to_string(),
        status: row.vocab("status")?,
    })
}

/// All rows of `decay.csv`.
#[derive(Debug, Clone)]
pub struct Decay {
    /// The rows in file order.
    pub rows: Vec<DecayRow>,
}

impl Decay {
    /// Read and validate the file text.
    pub fn parse(text: &str) -> Result<Decay> {
        let table = Table::parse(FILE, text, &HEADER)?;
        let rows = table.rows().map(read_row).collect::<Result<Vec<_>>>()?;
        let mut seen = std::collections::HashSet::new();
        for r in &rows {
            if !seen.insert(r.nuclide.name()) {
                return Err(DataError::line(
                    FILE,
                    r.line,
                    format!("the nuclide {} appears twice", r.nuclide.name()),
                ));
            }
        }
        Ok(Decay { rows })
    }

    /// The row of a nuclide by name.
    pub fn get(&self, name: &str) -> Option<&DecayRow> {
        self.rows.iter().find(|r| r.nuclide.name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decay_constant_rejects_non_positive_half_lives() {
        assert!(decay_constant(-1.0, TimeUnit::Seconds).is_err());
        assert!(decay_constant(0.0, TimeUnit::Seconds).is_err());
        assert!(decay_constant(f64::NAN, TimeUnit::Seconds).is_err());
        let l = decay_constant(30.08, TimeUnit::Years).unwrap();
        assert!((l / 7.302187e-10 - 1.0).abs() < 1e-6);
    }
}
