//! Cross-check of the FV solver against the exact analytical release series for the
//! SAME inputs (same radii, diffusivities, initial kernel inventory, absorbing outer
//! surface), plus a mesh/time refinement study measured against the exact answer.

use crate::{
    analytical::sturm_liouville::{self as analytical, Layer, ModalRelease},
    config::Config,
    numerical::finite_volume as fv,
};
use anyhow::{bail, Result};

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

#[derive(Clone, Debug)]
pub struct CompareRow {
    pub time_s: f64,
    pub fv: f64,
    /// `None` when the time is earlier than the exact series can resolve.
    pub exact: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct ReleaseComparison {
    pub rows: Vec<CompareRow>,
    pub modes: usize,
    pub lambda_covered: f64,
    pub min_valid_time_s: f64,
    pub max_abs_error: f64,
    pub final_abs_error: f64,
    /// See `ModalRelease::min_flux_ratio`.
    pub min_flux_ratio: f64,
    /// See `ModalRelease::accuracy_floor`.
    pub accuracy_floor: f64,
}

/// Compare FV release fractions with the exact series at every positive output time.
pub fn compare_release(cfg: &Config, fv_rows: &fv::TimeSeries) -> Result<ReleaseComparison> {
    let layers = layers_from_config(cfg);
    let times: Vec<f64> = fv_rows
        .iter()
        .map(|(t, _)| *t)
        .filter(|t| *t > 0.0)
        .collect();
    if times.is_empty() {
        bail!("no positive output times to compare");
    }
    let t_min = times.iter().copied().fold(f64::INFINITY, f64::min);
    let series = ModalRelease::new(&layers, 45.0 / t_min, MAX_MODES)?;
    let valid = series.min_valid_time();
    let mut rows = Vec::new();
    let (mut max_err, mut final_err) = (0.0f64, f64::NAN);
    for (t, d) in fv_rows.iter().filter(|(t, _)| *t > 0.0) {
        let exact = (*t >= valid).then(|| series.release_fraction(*t));
        if let Some(e) = exact {
            let err = (d.release_fraction - e).abs();
            max_err = max_err.max(err);
            final_err = err;
        }
        rows.push(CompareRow {
            time_s: *t,
            fv: d.release_fraction,
            exact,
        });
    }
    if rows.iter().all(|r| r.exact.is_none()) {
        bail!(
        "exact series (capped at {MAX_MODES} modes) resolves only t >= {valid:.3e} s, later than every output time"
    );
    }
    Ok(ReleaseComparison {
        rows,
        modes: series.eigenvalues.len(),
        lambda_covered: series.lambda_covered,
        min_valid_time_s: valid,
        max_abs_error: max_err,
        final_abs_error: final_err,
        min_flux_ratio: series.min_flux_ratio,
        accuracy_floor: series.accuracy_floor(),
    })
}

#[derive(Clone, Debug)]
pub struct ConvergencePoint {
    /// Total FV cells (spatial study) or timestep in seconds (temporal study).
    pub cells: usize,
    pub dt_s: f64,
    pub fv_release: f64,
    pub abs_error: f64,
}

#[derive(Clone, Debug)]
pub struct ConvergenceStudy {
    pub exact_final: f64,
    /// Estimated absolute accuracy of the exact series (see `ModalRelease::accuracy_floor`).
    pub accuracy_floor: f64,
    pub spatial: Vec<ConvergencePoint>,
    pub temporal: Vec<ConvergencePoint>,
}

impl ConvergenceStudy {
    /// Observed order between successive refinements (each refinement halves h or dt).
    pub fn orders(points: &[ConvergencePoint]) -> Vec<f64> {
        points
            .windows(2)
            .map(|w| (w[0].abs_error / w[1].abs_error).log2())
            .collect()
    }

    /// True when every error is well above the accuracy floor of the exact series, so
    /// observed orders mean something.
    pub fn resolved(&self, points: &[ConvergencePoint]) -> bool {
        points
            .iter()
            .all(|p| p.abs_error > 3.0 * self.accuracy_floor)
    }
}

fn scaled_cells(cfg: &Config, factor: f64) -> Config {
    let mut c = cfg.clone();
    for r in &mut c.regions {
        r.cells = ((r.cells as f64 * factor).round() as usize).max(1);
    }
    c
}

fn with_dt(cfg: &Config, dt: f64) -> Config {
    let mut c = cfg.clone();
    c.time.dt_s = dt;
    c.time.output_every_steps = usize::MAX;
    c
}

fn final_release(cfg: &Config) -> Result<f64> {
    let mesh = fv::build_mesh(cfg)?;
    let (rows, _) = fv::run(cfg, &mesh)?;
    Ok(rows.last().expect("initial row").1.release_fraction)
}

/// Release fraction with the Backward-Euler time error removed by Richardson
/// extrapolation, (8 F(dt/4) - 6 F(dt/2) + F(dt)) / 3. The error expansion of Backward
/// Euler is a1 dt + a2 dt^2 + ..., so this cancels both leading terms.
/// Used to isolate the spatial error in refinement studies.
pub fn time_extrapolated_release(cfg: &Config) -> Result<f64> {
    let f = |m: f64| final_release(&with_dt(cfg, cfg.time.dt_s * m));
    Ok((8.0 * f(0.25)? - 6.0 * f(0.5)? + f(1.0)?) / 3.0)
}

/// Rough work estimate (cell-steps) for one FV run.
pub fn run_cost(cfg: &Config) -> f64 {
    let cells: usize = cfg.regions.iter().map(|r| r.cells).sum();
    cfg.time.end_time_s / cfg.time.dt_s * cells as f64
}

/// Estimated cell-steps for the whole refinement study.
pub fn study_cost(cfg: &Config) -> f64 {
    let spatial: f64 = [0.5, 1.0, 2.0, 4.0, 8.0]
        .iter()
        .map(|f| run_cost(&scaled_cells(cfg, *f)) * 1.75)
        .sum();
    let temporal: f64 = [4.0, 2.0, 1.0, 0.5, 0.25]
        .iter()
        .map(|m| run_cost(&with_dt(cfg, cfg.time.dt_s * m)))
        .sum();
    spatial + temporal
}

/// Spatial study: cells per region scaled by 0.5, 1, 2, 4, 8. To keep the Backward-Euler
/// time error from masking the spatial error, each FV value is the time-extrapolated
/// value from `time_extrapolated_release`.
/// Temporal study: dt multiplied by 4, 2, 1, 1/2, 1/4 on the configured mesh.
/// Errors are measured against the exact series at the end time.
pub fn convergence_study(cfg: &Config) -> Result<ConvergenceStudy> {
    let layers = layers_from_config(cfg);
    let t_end = cfg.time.end_time_s;
    let series = ModalRelease::new(&layers, 45.0 / t_end, MAX_MODES)?;
    if t_end < series.min_valid_time() {
        bail!("exact series does not resolve the end time");
    }
    let exact_final = series.release_fraction(t_end);

    let mut spatial = Vec::new();
    for f in [0.5, 1.0, 2.0, 4.0, 8.0] {
        let c = scaled_cells(cfg, f);
        let extrapolated = time_extrapolated_release(&c)?;
        spatial.push(ConvergencePoint {
            cells: c.regions.iter().map(|r| r.cells).sum(),
            dt_s: cfg.time.dt_s,
            fv_release: extrapolated,
            abs_error: (extrapolated - exact_final).abs(),
        });
    }

    let mut temporal = Vec::new();
    for m in [4.0, 2.0, 1.0, 0.5, 0.25] {
        let c = with_dt(cfg, cfg.time.dt_s * m);
        let f = final_release(&c)?;
        temporal.push(ConvergencePoint {
            cells: c.regions.iter().map(|r| r.cells).sum(),
            dt_s: cfg.time.dt_s * m,
            fv_release: f,
            abs_error: (f - exact_final).abs(),
        });
    }
    Ok(ConvergenceStudy {
        exact_final,
        accuracy_floor: series.accuracy_floor(),
        spatial,
        temporal,
    })
}

/// Exact release fraction at one time for a configuration (helper for tests and tools).
pub fn exact_release_at(cfg: &Config, t: f64) -> Result<f64> {
    let v = analytical::exact_release(&layers_from_config(cfg), &[t], MAX_MODES)?;
    v[0].ok_or_else(|| anyhow::anyhow!("exact series does not resolve t = {t:e} s"))
}
