//! All register files read together: the six register files and the three irradiation files.

use std::path::Path;

use crate::arrhenius::{self, Arrhenius};
use crate::curve::{self, Curves};
use crate::decay::{self, Decay};
use crate::error::{DataError, Result};
use crate::geometry::{self, Geometry};
use crate::irradiation::{self, Irradiation};
use crate::release::{self, Releases};
use crate::temperature::{self, Temperatures};
use crate::yields::{self, Yields};

/// File name of the parameter register document, below `data/reference`.
pub const REGISTER_MARKDOWN: &str = "STRATO_Parameter_Register.md";

/// File name of the audit log, below `data/reference`.
pub const AUDIT_LOG: &str = "B6_audit_log.md";

/// The raw text of the six files, in the order geometry, Arrhenius, decay, yields, temperature, release.
#[derive(Debug, Clone)]
pub struct RegisterTexts {
    /// `geometry/geometry.csv`.
    pub geometry: String,
    /// `diffusivity/arrhenius.csv`.
    pub arrhenius: String,
    /// `nuclear/decay.csv`.
    pub decay: String,
    /// `nuclear/fission_yields.csv`.
    pub yields: String,
    /// `temperature/temperature_histories.csv`.
    pub temperature: String,
    /// `experimental/release_measurements.csv`.
    pub release: String,
    /// `irradiation/agr1_irradiation_conditions.csv`.
    pub irradiation_conditions: String,
    /// `irradiation/agr1_burnup_by_cycle.csv`.
    pub irradiation_burnup: String,
    /// `irradiation/agr1_power_history_digitised.csv`.
    pub irradiation_power: String,
    /// `reference/STRATO_Parameter_Register.md` when the file exists (the check that compares its
    /// tables with the CSV files skips itself when it is absent).
    pub register_markdown: Option<String>,
    /// `experimental/curve_assessment.csv`: one record per release-versus-time curve.
    pub curve_assessment: String,
    /// `reference/B6_audit_log.md` when the file exists (the check that every digitised row has an
    /// audit entry skips itself when it is absent).
    pub audit_log: Option<String>,
}

impl RegisterTexts {
    /// Read the six files below `data_dir` (the `data` folder of the repository).
    pub fn read(data_dir: &Path) -> Result<RegisterTexts> {
        let rd = |rel: &str| -> Result<String> {
            let p = data_dir.join(rel);
            std::fs::read_to_string(&p)
                .map_err(|e| DataError::file(rel, format!("cannot read the file: {e}")))
        };
        Ok(RegisterTexts {
            geometry: rd(geometry::FILE)?,
            arrhenius: rd(arrhenius::FILE)?,
            decay: rd(decay::FILE)?,
            yields: rd(yields::FILE)?,
            temperature: rd(temperature::FILE)?,
            release: rd(release::FILE)?,
            irradiation_conditions: rd(irradiation::CONDITIONS_FILE)?,
            irradiation_burnup: rd(irradiation::BURNUP_FILE)?,
            irradiation_power: rd(irradiation::POWER_FILE)?,
            register_markdown: std::fs::read_to_string(
                data_dir.join("reference").join(REGISTER_MARKDOWN),
            )
            .ok(),
            curve_assessment: rd(curve::FILE)?,
            audit_log: std::fs::read_to_string(data_dir.join("reference").join(AUDIT_LOG)).ok(),
        })
    }
}

/// The six typed registers.
#[derive(Debug, Clone)]
pub struct Register {
    /// Geometry.
    pub geometry: Geometry,
    /// Arrhenius terms.
    pub arrhenius: Arrhenius,
    /// Decay data.
    pub decay: Decay,
    /// Fission yields.
    pub yields: Yields,
    /// Temperature histories.
    pub temperature: Temperatures,
    /// Release measurements.
    pub release: Releases,
    /// AGR-1 irradiation conditions, burnup history and heat history.
    pub irradiation: Irradiation,
    /// Assessments of the release-versus-time curves.
    pub curves: Curves,
}

impl Register {
    /// Type and validate the six texts.
    pub fn from_texts(t: &RegisterTexts) -> Result<Register> {
        Ok(Register {
            geometry: Geometry::parse(&t.geometry)?,
            arrhenius: Arrhenius::parse(&t.arrhenius)?,
            decay: Decay::parse(&t.decay)?,
            yields: Yields::parse(&t.yields)?,
            temperature: Temperatures::parse(&t.temperature)?,
            release: Releases::parse(&t.release)?,
            irradiation: Irradiation::parse(
                &t.irradiation_conditions,
                &t.irradiation_burnup,
                &t.irradiation_power,
            )?,
            curves: Curves::parse(&t.curve_assessment)?,
        })
    }

    /// Read the six files below `data_dir` and type them.
    pub fn load(data_dir: &Path) -> Result<Register> {
        Register::from_texts(&RegisterTexts::read(data_dir)?)
    }
}
