//! `data/experimental/curve_assessment.csv`: one record per release-versus-time curve.
//!
//! A curve is one time series of `release_measurements.csv`: one specimen, one nuclide, one
//! temperature history and one release definition (`metric-r2.md`, Section 1.2). The assessment of
//! a curve records what decides whether the whole-curve metric R2 may be computed for it: the
//! level of the specimen, the class of the values (a compact total, an upper bound, a non-detect),
//! the decay correction, the basis of a suspected coating failure, the reason code, the registered
//! history and geometry that a model run uses, and where the curve comes from. The class and the
//! reason are read from this file and never defaulted: a missing field is an error.
//!
//! The rows of a curve stay in `release_measurements.csv`; a curve names them by `row_ids`, a
//! range `E-541..E-556` or a list `E-117,E-119`. A row belongs to at most one curve.

use std::collections::HashMap;

use crate::error::{DataError, Result};
use crate::string_enum;
use crate::table::{Row, Table};

/// Path of the file below the data folder.
pub const FILE: &str = "experimental/curve_assessment.csv";

/// The exact header.
pub const HEADER: [&str; 25] = [
    "dataset_id",
    "row_ids",
    "nuclide",
    "T_C",
    "specimen",
    "specimen_level",
    "source_aggregated",
    "bound_class",
    "decay_correction",
    "coating_failure_basis",
    "r2_class",
    "reason_code",
    "history_id",
    "geometry_ids",
    "model_configuration",
    "method_b",
    "source",
    "source_location",
    "link",
    "value_kind",
    "n_points",
    "verification_status",
    "reviewer",
    "verification_ref",
    "notes",
];

string_enum! {
    /// The physical level of the specimen of a curve (a sphere is treated at the compact level).
    pub enum SpecimenLevel ("specimen level") {
        /// One particle.
        SingleParticle => "single_particle",
        /// A batch of loose particles.
        ParticleBatch => "particle_batch",
        /// A compact (or a fuel sphere with matrix).
        Compact => "compact",
    }
}

string_enum! {
    /// What kind of number the values of the curve are.
    pub enum BoundClass ("bound class") {
        /// Point values of the quantity.
        NoBound => "none",
        /// Upper bounds.
        UpperBound => "upper_bound",
        /// Non-detects.
        NonDetect => "non_detect",
        /// Window upper bounds under a premise.
        WindowUpperBound => "window_upper_bound",
        /// Totals of a whole compact (or sphere), an upper bound of the single-particle release.
        CompactTotal => "compact_total",
    }
}

string_enum! {
    /// Whether the source corrects its activities for decay.
    pub enum DecayCorrection ("decay correction") {
        /// The source states that the values are decay-corrected.
        Corrected => "corrected",
        /// The source states that they are not.
        NotCorrected => "not_corrected",
        /// The source does not say.
        NotStated => "not_stated",
    }
}

string_enum! {
    /// The verdict of the assessment.
    pub enum R2Class ("R2 class") {
        /// An R2 may be computed (only after independent verification).
        Suitable => "R2 SUITABLE",
        /// The curve is kept and may be plotted; no R2.
        PlotOnly => "PLOT ONLY",
    }
}

/// One curve assessment.
#[derive(Debug, Clone, PartialEq)]
pub struct Curve {
    /// Line in the file.
    pub line: usize,
    /// Dataset identifier.
    pub dataset_id: String,
    /// The release rows of the curve, in time order as the file lists them.
    pub row_ids: Vec<String>,
    /// Nuclide as the release rows write it.
    pub nuclide: String,
    /// Test temperature as the release rows write it.
    pub t_c: String,
    /// The specimen in words.
    pub specimen: String,
    /// Level of the specimen.
    pub specimen_level: SpecimenLevel,
    /// True when the source itself prints one curve over several specimens.
    pub source_aggregated: bool,
    /// Class of the values.
    pub bound_class: BoundClass,
    /// Decay correction.
    pub decay_correction: DecayCorrection,
    /// Basis of a suspected or known coating failure, or `none`.
    pub coating_failure_basis: String,
    /// Verdict.
    pub r2_class: R2Class,
    /// Reason code of a PLOT ONLY curve (`C1 compact_total`, `C2 coating_failure`, ...).
    pub reason_code: String,
    /// Registered temperature history of a model run.
    pub history_id: String,
    /// Registered geometry rows of a model run.
    pub geometry_ids: String,
    /// The model configuration the assessment gives.
    pub model_configuration: String,
    /// Whether Method B applies.
    pub method_b: String,
    /// Short citation, as in the `source` column of the release rows.
    pub source: String,
    /// Figure, table or page.
    pub source_location: String,
    /// Link.
    pub link: String,
    /// `digitised (...)` or `measured_tabulated`.
    pub value_kind: String,
    /// Number of points.
    pub n_points: usize,
    /// Verification status of the assessment.
    pub verification_status: String,
    /// Who set the assessment.
    pub reviewer: String,
    /// Where the independent verification is recorded; empty until it is.
    pub verification_ref: String,
    /// Notes and limitations.
    pub notes: String,
}

/// Expand `E-541..E-556` or `E-117,E-119` into the list of ids.
pub fn expand_row_ids(text: &str) -> std::result::Result<Vec<String>, String> {
    let number = |s: &str| -> std::result::Result<u32, String> {
        s.strip_prefix("E-")
            .and_then(|n| n.parse::<u32>().ok())
            .ok_or_else(|| format!("`{s}` is not a row id of the form E-123"))
    };
    if let Some((a, b)) = text.split_once("..") {
        let (a, b) = (number(a)?, number(b)?);
        if b < a {
            return Err(format!("the range `{text}` runs backwards"));
        }
        return Ok((a..=b).map(|n| format!("E-{n:03}")).collect());
    }
    text.split(',')
        .map(|s| number(s).map(|n| format!("E-{n:03}")))
        .collect()
}

fn read_curve(row: Row<'_>) -> Result<Curve> {
    let row_ids =
        expand_row_ids(row.required("row_ids")?).map_err(|m| row.err(Some("row_ids"), m))?;
    let source_aggregated = match row.required("source_aggregated")? {
        "true" => true,
        "false" => false,
        other => {
            return Err(row.err(
                Some("source_aggregated"),
                format!("`{other}` is neither true nor false"),
            ))
        }
    };
    let n_points = row.uint("n_points")? as usize;
    if n_points != row_ids.len() {
        return Err(row.err(
            Some("n_points"),
            format!(
                "n_points is {n_points} but row_ids names {} rows",
                row_ids.len()
            ),
        ));
    }
    let r2_class: R2Class = row.vocab("r2_class")?;
    let reason_code = row.text("reason_code")?.to_string();
    if r2_class == R2Class::PlotOnly && reason_code.is_empty() {
        return Err(row.err(
            Some("reason_code"),
            "a PLOT ONLY curve needs its reason code",
        ));
    }
    Ok(Curve {
        line: row.line(),
        dataset_id: row.required("dataset_id")?.to_string(),
        row_ids,
        nuclide: row.required("nuclide")?.to_string(),
        t_c: row.required("T_C")?.to_string(),
        specimen: row.required("specimen")?.to_string(),
        specimen_level: row.vocab("specimen_level")?,
        source_aggregated,
        bound_class: row.vocab("bound_class")?,
        decay_correction: row.vocab("decay_correction")?,
        coating_failure_basis: row.required("coating_failure_basis")?.to_string(),
        r2_class,
        reason_code,
        history_id: row.required("history_id")?.to_string(),
        geometry_ids: row.required("geometry_ids")?.to_string(),
        model_configuration: row.required("model_configuration")?.to_string(),
        method_b: row.required("method_b")?.to_string(),
        source: row.required("source")?.to_string(),
        source_location: row.required("source_location")?.to_string(),
        link: row.required("link")?.to_string(),
        value_kind: row.required("value_kind")?.to_string(),
        n_points,
        verification_status: row.required("verification_status")?.to_string(),
        reviewer: row.required("reviewer")?.to_string(),
        verification_ref: row.text("verification_ref")?.to_string(),
        notes: row.required("notes")?.to_string(),
    })
}

/// All curve assessments.
#[derive(Debug, Clone)]
pub struct Curves {
    /// The records in file order.
    pub curves: Vec<Curve>,
    by_row: HashMap<String, usize>,
}

impl Curves {
    /// Read and validate the file text: exact header, closed vocabularies, unique dataset ids, and
    /// no release row in two curves.
    pub fn parse(text: &str) -> Result<Curves> {
        let table = Table::parse(FILE, text, &HEADER)?;
        let curves = table.rows().map(read_curve).collect::<Result<Vec<_>>>()?;
        let mut ids = std::collections::HashSet::new();
        let mut by_row = HashMap::new();
        for (i, c) in curves.iter().enumerate() {
            if !ids.insert(c.dataset_id.as_str()) {
                return Err(DataError::line(
                    FILE,
                    c.line,
                    format!("the dataset id {} appears twice", c.dataset_id),
                ));
            }
            for r in &c.row_ids {
                if let Some(j) = by_row.insert(r.clone(), i) {
                    return Err(DataError::line(
                        FILE,
                        c.line,
                        format!(
                            "the row {r} belongs to the curves {} and {}",
                            curves[j].dataset_id, c.dataset_id
                        ),
                    ));
                }
            }
        }
        Ok(Curves { curves, by_row })
    }

    /// The curve with dataset id `id`.
    pub fn get(&self, id: &str) -> Option<&Curve> {
        self.curves.iter().find(|c| c.dataset_id == id)
    }

    /// The curve that holds the release row `row_id`.
    pub fn curve_of_row(&self, row_id: &str) -> Option<&Curve> {
        self.by_row.get(row_id).map(|&i| &self.curves[i])
    }
}
