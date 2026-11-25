//! `data/experimental/release_measurements.csv`: measured release fractions and counts.
//!
//! The qualifier of a row decides its type: a printed point value, a digitised point value, an
//! upper bound, a lower bound, a range, or a count of failed over tested particles. An upper bound
//! is never a point value and is never read as one. The unit (fraction, particles, per hour) is
//! part of the row. The reader keeps `particle_applicability`, the note that says at what level
//! (compact, particle batch, sphere) a value was measured.

use crate::common::{EvidenceClass, NuclideId, Status};
use crate::error::{DataError, Result};
use crate::string_enum;
use crate::table::{Printed, Row, Table};

/// Path of the file below the data folder.
pub const FILE: &str = "experimental/release_measurements.csv";

/// The exact header.
pub const HEADER: [&str; 24] = [
    "id",
    "program",
    "compact_or_sample",
    "fuel_type",
    "group",
    "nuclide",
    "T_C",
    "time_h",
    "time_basis",
    "quantity",
    "qualifier",
    "value",
    "value_min",
    "value_max",
    "unit",
    "value_class",
    "evidence_class",
    "particle_applicability",
    "parfume_value_same_source",
    "source",
    "source_location",
    "link",
    "status",
    "notes",
];

string_enum! {
    /// The experimental programme.
    pub enum Program ("programme") {
        /// AGR-1 irradiation and safety tests.
        Agr1 => "AGR-1",
        /// JAERI heating tests.
        Jaeri => "JAERI",
        /// IAEA CRP-6 benchmark text values.
        Crp6 => "CRP-6",
    }
}

string_enum! {
    /// The fuel type.
    pub enum FuelType ("fuel type") {
        /// AGR-1 baseline.
        Baseline => "Baseline",
        /// AGR-1 variant 1.
        Variant1 => "Variant 1",
        /// AGR-1 variant 3.
        Variant3 => "Variant 3",
        /// Mixed fuel types.
        Mixed => "mixed",
        /// UO2 TRISO particles.
        Uo2Triso => "UO2 TRISO",
        /// UO2 TRISO fuel sphere.
        Uo2TrisoSphere => "UO2 TRISO sphere",
        /// UO2 TRISO compact.
        Uo2TrisoCompact => "UO2 TRISO compact",
        /// Not stated.
        NotStated => "not stated",
    }
}

string_enum! {
    /// What the number is a number of (the quantity and its denominator as the register words them).
    pub enum Quantity ("release quantity") {
        /// Compact release fraction.
        CompactReleaseFraction => "compact release fraction",
        /// Total compact release fraction.
        TotalCompactReleaseFraction => "total compact release fraction",
        /// Cumulative compact release fraction, an upper bound.
        CumulativeCompactReleaseFractionUpperBound => "cumulative compact release fraction (upper bound)",
        /// Compact release fraction, an upper bound.
        CompactReleaseFractionUpperBound => "compact release fraction (upper bound)",
        /// Prompt compact release fraction, a range.
        PromptCompactReleaseFractionRange => "prompt compact release fraction (range)",
        /// Compact release fraction indicating a non-intact SiC layer.
        CompactReleaseFractionIndicatingFailedSic => "compact release fraction indicating a non-intact SiC",
        /// Compact release rate.
        CompactReleaseRate => "compact release rate",
        /// Fractional release of a loose-particle batch.
        FractionalRelease => "fractional release",
        /// Release in particle equivalents, a range.
        ReleaseInParticleEquivalentsRange => "release in particle equivalents (range)",
        /// Release fraction equivalent to one particle.
        ReleaseFractionEquivalentToOneParticle => "release fraction equivalent to one particle",
        /// Fraction found in the matrix, an upper bound.
        FractionInMatrixUpperBound => "fraction found in matrix (upper bound)",
        /// Fraction of the silver inventory in the matrix, a range.
        FractionOfAgInventoryInMatrixRange => "fraction of Ag inventory in matrix (range)",
        /// Sphere release fraction, an approximate text value.
        SphereReleaseFractionApproximate => "sphere release fraction (approximate text value)",
        /// Number of particles with failed IPyC and SiC.
        ParticlesWithFailedIpycAndSic => "particles with failed IPyC and SiC",
        /// Failed SiC over particles tested.
        ParticlesWithFailedSicOverTested => "particles with failed SiC / particles tested",
    }
}

string_enum! {
    /// The `unit` column.
    pub enum ReleaseUnit ("release unit") {
        /// A fraction (dimensionless).
        Fraction => "1",
        /// A number of particles.
        Particles => "particles",
        /// Per hour.
        PerHour => "h^-1",
    }
}

string_enum! {
    /// The `value_class` column.
    pub enum ValueClass ("value class") {
        /// A printed table value.
        PrintedTable => "M printed table",
        /// A value read from a figure by digitising.
        Digitised => "R-dig",
        /// A value read visually from a figure.
        VisualReading => "R-vis",
        /// A value printed in text.
        Text => "M text",
        /// A value read from a vector figure.
        VectorFigure => "M (vector figure)",
        /// A post-irradiation-examination estimate.
        PieEstimate => "M (PIE estimate)",
        /// An upper bound.
        UpperBound => "M-UB",
    }
}

/// What was measured: a nuclide, an element, or nothing (a count).
#[derive(Debug, Clone, PartialEq)]
pub enum Measured {
    /// A nuclide such as `Cs-134`.
    Nuclide(NuclideId),
    /// An element label only (`Cs`, `Ag`, `Sr`): the source does not name an isotope.
    Element(String),
    /// `n/a`: not a nuclide quantity.
    NotApplicable,
}

/// The test temperature.
#[derive(Debug, Clone, PartialEq)]
pub enum TestTemperature {
    /// One temperature, degC.
    Celsius(Printed),
    /// A range printed as `lo-hi`, degC.
    Range {
        /// Lower end.
        lo: Printed,
        /// Upper end.
        hi: Printed,
    },
    /// No temperature in the row.
    Absent,
}

/// The value of a row, typed by its qualifier.
#[derive(Debug, Clone, PartialEq)]
pub enum ReleaseValue {
    /// `=`: a printed or stated value.
    Point(Printed),
    /// `~`: an approximate value, read from a figure or printed with `~`.
    Digitised(Printed),
    /// `<`: an upper bound. The measurement is at most this value.
    UpperBound(Printed),
    /// `>`: a lower bound.
    LowerBound(Printed),
    /// `range`: both ends.
    Range {
        /// Lower end.
        min: Printed,
        /// Upper end.
        max: Printed,
    },
    /// `=` with `value_max`: failed particles over particles tested.
    FailedOverTested {
        /// Failed.
        failed: Printed,
        /// Tested.
        tested: Printed,
    },
}

impl ReleaseValue {
    /// True for a bound (an inequality), false for a value.
    pub fn is_bound(&self) -> bool {
        matches!(
            self,
            ReleaseValue::UpperBound(_) | ReleaseValue::LowerBound(_)
        )
    }

    /// The qualifier text of the file.
    pub fn qualifier(&self) -> &'static str {
        match self {
            ReleaseValue::Point(_) | ReleaseValue::FailedOverTested { .. } => "=",
            ReleaseValue::Digitised(_) => "~",
            ReleaseValue::UpperBound(_) => "<",
            ReleaseValue::LowerBound(_) => ">",
            ReleaseValue::Range { .. } => "range",
        }
    }
}

/// One row of `release_measurements.csv`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRow {
    /// Line in the file.
    pub line: usize,
    /// Row identifier (`E-001`).
    pub id: String,
    /// Programme.
    pub program: Program,
    /// Compact or sample label.
    pub compact_or_sample: String,
    /// Fuel type.
    pub fuel_type: FuelType,
    /// Group as printed (`intact`, `failed SiC`, ...).
    pub group: String,
    /// What was measured.
    pub measured: Measured,
    /// Test temperature.
    pub temperature: TestTemperature,
    /// Time, hours, when the row has one.
    pub time_h: Option<Printed>,
    /// Basis of the time as printed.
    pub time_basis: String,
    /// Quantity.
    pub quantity: Quantity,
    /// The typed value.
    pub value: ReleaseValue,
    /// Unit.
    pub unit: ReleaseUnit,
    /// Value class.
    pub value_class: ValueClass,
    /// Evidence class.
    pub evidence_class: EvidenceClass,
    /// At what level the value was measured and what that means for a single particle (may be empty; the audit checks it).
    pub particle_applicability: String,
    /// The PARFUME value from the same source, as printed.
    pub parfume_value_same_source: Option<String>,
    /// Short citation.
    pub source: String,
    /// Location in the source.
    pub source_location: String,
    /// Link.
    pub link: String,
    /// Status.
    pub status: Status,
    /// Notes.
    pub notes: String,
}

fn read_value(row: &Row<'_>, quantity: Quantity) -> Result<ReleaseValue> {
    let q = row.required("qualifier")?;
    let (v, lo, hi) = (
        row.optional("value")?,
        row.optional("value_min")?,
        row.optional("value_max")?,
    );
    let need_value_only = |what: &str| -> Result<Printed> {
        if lo.is_some() || hi.is_some() {
            return Err(row.err(
                Some("value_min"),
                format!("a `{what}` row must not carry value_min or value_max"),
            ));
        }
        row.number("value")
    };
    match q {
        "=" => match hi {
            Some(_) => {
                if lo.is_some() || quantity != Quantity::ParticlesWithFailedSicOverTested {
                    return Err(row.err(
                        Some("value_max"),
                        "value_max on a `=` row is only allowed for failed over tested particles",
                    ));
                }
                Ok(ReleaseValue::FailedOverTested {
                    failed: row.number("value")?,
                    tested: row.number("value_max")?,
                })
            }
            None => Ok(ReleaseValue::Point(need_value_only("=")?)),
        },
        "~" => Ok(ReleaseValue::Digitised(need_value_only("~")?)),
        "<" => Ok(ReleaseValue::UpperBound(need_value_only("<")?)),
        ">" => Ok(ReleaseValue::LowerBound(need_value_only(">")?)),
        "range" => {
            if v.is_some() {
                return Err(row.err(Some("value"), "a range row has no single value"));
            }
            Ok(ReleaseValue::Range {
                min: row.number("value_min")?,
                max: row.number("value_max")?,
            })
        }
        other => Err(row.err(
            Some("qualifier"),
            format!("`{other}` is not one of =, ~, <, >, range"),
        )),
    }
}

fn read_temperature(row: &Row<'_>) -> Result<TestTemperature> {
    match row.optional("T_C")? {
        None => Ok(TestTemperature::Absent),
        Some(t) => {
            if let Ok(p) = Printed::parse(t) {
                return Ok(TestTemperature::Celsius(p));
            }
            match t.split_once('-') {
                Some((a, b)) => Ok(TestTemperature::Range {
                    lo: Printed::parse(a).map_err(|m| row.err(Some("T_C"), m))?,
                    hi: Printed::parse(b).map_err(|m| row.err(Some("T_C"), m))?,
                }),
                None => Err(row.err(
                    Some("T_C"),
                    format!("`{t}` is neither a temperature nor a range"),
                )),
            }
        }
    }
}

fn read_row(row: Row<'_>) -> Result<ReleaseRow> {
    let quantity: Quantity = row.vocab("quantity")?;
    let measured = match row.required("nuclide")? {
        "n/a" => Measured::NotApplicable,
        e @ ("Cs" | "Ag" | "Sr") => Measured::Element(e.to_string()),
        n => Measured::Nuclide(NuclideId::parse(n).map_err(|m| row.err(Some("nuclide"), m))?),
    };
    Ok(ReleaseRow {
        line: row.line(),
        id: row.required("id")?.to_string(),
        program: row.vocab("program")?,
        compact_or_sample: row.required("compact_or_sample")?.to_string(),
        fuel_type: row.vocab("fuel_type")?,
        group: row.required("group")?.to_string(),
        measured,
        temperature: read_temperature(&row)?,
        time_h: row.opt_number("time_h")?,
        time_basis: row.required("time_basis")?.to_string(),
        quantity,
        value: read_value(&row, quantity)?,
        unit: row.vocab("unit")?,
        value_class: row.vocab("value_class")?,
        evidence_class: row.vocab("evidence_class")?,
        particle_applicability: row.text("particle_applicability")?.to_string(),
        parfume_value_same_source: row
            .optional("parfume_value_same_source")?
            .map(str::to_string),
        source: row.required("source")?.to_string(),
        source_location: row.required("source_location")?.to_string(),
        link: row.required("link")?.to_string(),
        status: row.vocab("status")?,
        notes: row.text("notes")?.to_string(),
    })
}

/// All rows of `release_measurements.csv`.
#[derive(Debug, Clone)]
pub struct Releases {
    /// The rows in file order.
    pub rows: Vec<ReleaseRow>,
}

impl Releases {
    /// Read and validate the file text.
    pub fn parse(text: &str) -> Result<Releases> {
        let table = Table::parse(FILE, text, &HEADER)?;
        let rows = table.rows().map(read_row).collect::<Result<Vec<_>>>()?;
        let mut seen = std::collections::HashSet::new();
        for r in &rows {
            if !seen.insert(r.id.as_str()) {
                return Err(DataError::line(
                    FILE,
                    r.line,
                    format!("the id {} appears twice", r.id),
                ));
            }
        }
        Ok(Releases { rows })
    }

    /// The row with identifier `id`.
    pub fn get(&self, id: &str) -> Option<&ReleaseRow> {
        self.rows.iter().find(|r| r.id == id)
    }
}
