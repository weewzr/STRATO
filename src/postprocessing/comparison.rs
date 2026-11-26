//! Helpers that tie the analytical Sturm-Liouville series (Method B) to a [`Config`]: the layer
//! list, the cap on the eigenmodes, and the exact release fraction at one time. The comparison
//! of the BTCS run with the series and the refinement study are in
//! [`crate::validation::regression`] and in the wrapper's `compare` function.

use crate::{
    analytical::sturm_liouville::{self as analytical, Layer},
    config::Config,
};
use anyhow::Result;

/// Upper bound on eigenmodes kept for the exact series.
pub const MAX_MODES: usize = 20_000;

pub fn layers_from_config(cfg: &Config) -> Vec<Layer> {
    cfg.regions
        .iter()
        .map(|r| Layer {
            outer_radius: r.outer_radius_m,
            diffusivity: r.diffusivity_m2_s,
        })
        .collect()
}

/// Exact release fraction at one time for a configuration (helper for tests and tools).
pub fn exact_release_at(cfg: &Config, t: f64) -> Result<f64> {
    let v = analytical::exact_release(&layers_from_config(cfg), &[t], MAX_MODES)?;
    v[0].ok_or_else(|| anyhow::anyhow!("exact series does not resolve t = {t:e} s"))
}
