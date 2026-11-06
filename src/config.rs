use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::{fs, path::Path};

pub const PLACEHOLDER: &str = "INSERT_VALUE_HERE";

#[derive(Clone, Debug, Deserialize)]
pub struct RegionConfig {
    pub name: String,
    pub outer_radius_m: f64,
    pub diffusivity_m2_s: f64,
    pub cells: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InitialConfig {
    pub kernel_concentration_mol_m3: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeConfig {
    pub dt_s: f64,
    pub end_time_s: f64,
    pub output_every_steps: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OutputConfig {
    pub directory: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub case_name: String,
    pub species: String,
    pub temperature_k: f64,
    pub regions: Vec<RegionConfig>,
    pub initial: InitialConfig,
    pub time: TimeConfig,
    pub output: OutputConfig,
}

/// Report every uncommented `INSERT_VALUE_HERE` with the instruction comment above it,
/// so a half-filled template fails with a readable list instead of a TOML type error.
fn unfilled_placeholders(raw: &str) -> Vec<String> {
    let mut last_instruction = String::new();
    let mut found = Vec::new();
    for (i, line) in raw.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            if trimmed.starts_with("# INSERT") && trimmed.ends_with("BELOW") {
                last_instruction = trimmed.trim_start_matches('#').trim().to_string();
            }
        } else if trimmed.contains(PLACEHOLDER) {
            found.push(format!("line {}: {}", i + 1, last_instruction));
        }
    }
    found
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let raw = fs::read_to_string(path.as_ref())
            .with_context(|| format!("reading {}", path.as_ref().display()))?;
        let missing = unfilled_placeholders(&raw);
        if !missing.is_empty() {
            bail!(
                "{} literature value(s) in {} are still {PLACEHOLDER}:\n  {}",
                missing.len(),
                path.as_ref().display(),
                missing.join("\n  ")
            );
        }
        let cfg: Self = toml::from_str(&raw).context("parsing TOML")?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<()> {
        if self.regions.len() != 5 {
            bail!("exactly five physical regions are required");
        }
        if self.species.trim().is_empty() {
            bail!("species must not be empty");
        }
        if !(self.temperature_k.is_finite() && self.temperature_k > 0.0) {
            bail!("temperature_k must be a positive finite number (kelvin)");
        }
        let mut last = 0.0;
        for r in &self.regions {
            if !(r.outer_radius_m.is_finite() && r.diffusivity_m2_s.is_finite())
                || r.outer_radius_m <= last
                || r.diffusivity_m2_s <= 0.0
                || r.cells == 0
            {
                bail!("invalid region {}", r.name);
            }
            last = r.outer_radius_m;
        }
        let t = &self.time;
        if !(t.dt_s.is_finite() && t.end_time_s.is_finite())
            || t.dt_s <= 0.0
            || t.end_time_s <= 0.0
            || t.output_every_steps == 0
        {
            bail!("invalid time controls");
        }
        let c0 = self.initial.kernel_concentration_mol_m3;
        if !c0.is_finite() || c0 < 0.0 {
            bail!("initial concentration must be nonnegative and finite");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_scan_reports_instruction_and_line() {
        let raw = "a = 1\n# INSERT LITERATURE-VERIFIED KERNEL DIFFUSIVITY (m^2/s) BELOW\nd = \"INSERT_VALUE_HERE\"\n# END X\n# d2 = \"INSERT_VALUE_HERE\"\n";
        let found = unfilled_placeholders(raw);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("line 3"));
        assert!(found[0].contains("KERNEL DIFFUSIVITY"));
    }
}
