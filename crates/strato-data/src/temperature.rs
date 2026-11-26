//! `data/temperature/temperature_histories.csv`: temperature histories.
//!
//! One row is one step of one history (`history_id`, `step`). Temperatures are in degC, rates in
//! degC per hour, durations in hours. A temperature may be the symbolic test temperature
//! (`T_test`) of a plan instead of a number: that is a different type, not a missing value.

use crate::common::{EvidenceClass, Status};
use crate::error::{DataError, Result};
use crate::string_enum;
use crate::table::{Printed, Row, Table};

/// Path of the file below the data folder.
pub const FILE: &str = "temperature/temperature_histories.csv";

/// The exact header.
pub const HEADER: [&str; 18] = [
    "id",
    "history_id",
    "history",
    "step",
    "segment",
    "T_start_C",
    "T_end_C",
    "rate_C_per_h",
    "ramp_h",
    "hold_h",
    "cumulative_end_h_calc",
    "history_class",
    "evidence_class",
    "source",
    "source_location",
    "link",
    "status",
    "notes",
];

string_enum! {
    /// The `history_class` column: what kind of history the step belongs to.
    pub enum HistoryClass ("history class") {
        /// A benchmark specification.
        SpecifiedBenchmark => "specified (benchmark)",
        /// A synthetic benchmark specification.
        SpecifiedSyntheticBenchmark => "specified (synthetic benchmark)",
        /// A specification as the register models it.
        SpecifiedAsModelled => "specified (as modelled)",
        /// A specification.
        Specified => "specified",
        /// Test conditions as specified.
        SpecifiedTestConditions => "specified (test conditions)",
        /// A thermal analysis of the as-run test.
        CalculatedAsRunThermalAnalysis => "calculated (as-run thermal analysis)",
        /// Printed in a preliminary form without a stated basis.
        AsPrintedPreliminary => "as printed (preliminary; basis not stated in notes)",
        /// Reconstructed from figures.
        Reconstructed => "reconstructed",
        /// Measured, as text.
        MeasuredText => "measured (text)",
        /// Measured or specified, as printed.
        MeasuredOrSpecifiedAsPrinted => "measured/specified (as printed)",
        /// Measured temperature with a reconstructed duration.
        MeasuredTextAndReconstructedDuration => "measured (text) and reconstructed (duration)",
        /// Specified or calculated, as run.
        SpecifiedOrCalculatedAsRun => "specified/calculated (as-run)",
    }
}

/// A start or end temperature.
#[derive(Debug, Clone, PartialEq)]
pub enum TemperaturePoint {
    /// A number, degC.
    Celsius(Printed),
    /// The test temperature of a plan, written `T_test` (possibly with the list of test temperatures).
    TestTemperature(String),
}

impl TemperaturePoint {
    /// The number, when it is one.
    pub fn celsius(&self) -> Option<f64> {
        match self {
            TemperaturePoint::Celsius(p) => Some(p.value),
            TemperaturePoint::TestTemperature(_) => None,
        }
    }
}

/// One row of `temperature_histories.csv`.
#[derive(Debug, Clone, PartialEq)]
pub struct TemperatureRow {
    /// Line in the file.
    pub line: usize,
    /// Row identifier (`TH-001`).
    pub id: String,
    /// History identifier.
    pub history_id: String,
    /// History description.
    pub history: String,
    /// Step number within the history, from 1.
    pub step: u32,
    /// Segment description.
    pub segment: String,
    /// Temperature at the start of the step.
    pub t_start: Option<TemperaturePoint>,
    /// Temperature at the end of the step.
    pub t_end: Option<TemperaturePoint>,
    /// Ramp rate, degC per hour.
    pub rate_c_per_h: Option<Printed>,
    /// Ramp duration, hours (calculated or printed).
    pub ramp_h: Option<Printed>,
    /// Hold duration, hours.
    pub hold_h: Option<Printed>,
    /// Cumulative end time, hours, as the register calculated it.
    pub cumulative_end_h_calc: Option<Printed>,
    /// History class.
    pub history_class: HistoryClass,
    /// Evidence class.
    pub evidence_class: EvidenceClass,
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

fn point(row: &Row<'_>, column: &str) -> Result<Option<TemperaturePoint>> {
    match row.optional(column)? {
        None => Ok(None),
        Some(t) => match Printed::parse(t) {
            Ok(p) => Ok(Some(TemperaturePoint::Celsius(p))),
            Err(_) if t.starts_with("T_test") => {
                Ok(Some(TemperaturePoint::TestTemperature(t.to_string())))
            }
            Err(m) => Err(row.err(Some(column), m)),
        },
    }
}

fn read_row(row: Row<'_>) -> Result<TemperatureRow> {
    Ok(TemperatureRow {
        line: row.line(),
        id: row.required("id")?.to_string(),
        history_id: row.required("history_id")?.to_string(),
        history: row.required("history")?.to_string(),
        step: row.uint("step")?,
        segment: row.required("segment")?.to_string(),
        t_start: point(&row, "T_start_C")?,
        t_end: point(&row, "T_end_C")?,
        rate_c_per_h: row.opt_number("rate_C_per_h")?,
        ramp_h: row.opt_number("ramp_h")?,
        hold_h: row.opt_number("hold_h")?,
        cumulative_end_h_calc: row.opt_number("cumulative_end_h_calc")?,
        history_class: row.vocab("history_class")?,
        evidence_class: row.vocab("evidence_class")?,
        source: row.required("source")?.to_string(),
        source_location: row.required("source_location")?.to_string(),
        link: row.required("link")?.to_string(),
        status: row.vocab("status")?,
        notes: row.text("notes")?.to_string(),
    })
}

/// All rows of `temperature_histories.csv`.
#[derive(Debug, Clone)]
pub struct Temperatures {
    /// The rows in file order.
    pub rows: Vec<TemperatureRow>,
}

impl Temperatures {
    /// Read and validate the file text. Ids are unique and the steps of each history count 1, 2, 3, ... in file order.
    pub fn parse(text: &str) -> Result<Temperatures> {
        let table = Table::parse(FILE, text, &HEADER)?;
        let rows = table.rows().map(read_row).collect::<Result<Vec<_>>>()?;
        let mut seen = std::collections::HashSet::new();
        let mut next_step: Vec<(String, u32)> = Vec::new();
        for r in &rows {
            if !seen.insert(r.id.as_str()) {
                return Err(DataError::line(
                    FILE,
                    r.line,
                    format!("the id {} appears twice", r.id),
                ));
            }
            match next_step.iter_mut().find(|(h, _)| *h == r.history_id) {
                Some((_, n)) => {
                    if r.step != *n {
                        return Err(DataError::line(
                            FILE,
                            r.line,
                            format!(
                                "step {} of history {} where step {} is expected",
                                r.step, r.history_id, n
                            ),
                        ));
                    }
                    *n += 1;
                }
                None => {
                    if r.step != 1 {
                        return Err(DataError::line(
                            FILE,
                            r.line,
                            format!(
                                "history {} starts with step {}, not 1",
                                r.history_id, r.step
                            ),
                        ));
                    }
                    next_step.push((r.history_id.clone(), 2));
                }
            }
        }
        Ok(Temperatures { rows })
    }

    /// The histories in order of first appearance, each with its rows in step order.
    pub fn histories(&self) -> Vec<(&str, Vec<&TemperatureRow>)> {
        let mut out: Vec<(&str, Vec<&TemperatureRow>)> = Vec::new();
        for r in &self.rows {
            match out.iter_mut().find(|(h, _)| *h == r.history_id) {
                Some((_, v)) => v.push(r),
                None => out.push((r.history_id.as_str(), vec![r])),
            }
        }
        out
    }
}
