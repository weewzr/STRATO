//! All six register files read together.

use std::path::Path;

use crate::arrhenius::{self, Arrhenius};
use crate::decay::{self, Decay};
use crate::error::{DataError, Result};
use crate::geometry::{self, Geometry};
use crate::release::{self, Releases};
use crate::temperature::{self, Temperatures};
use crate::yields::{self, Yields};

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
        })
    }

    /// Read the six files below `data_dir` and type them.
    pub fn load(data_dir: &Path) -> Result<Register> {
        Register::from_texts(&RegisterTexts::read(data_dir)?)
    }
}
