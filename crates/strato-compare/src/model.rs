//! Runs of the BTCS solver for the registered geometry and the registered diffusion coefficients.
//!
//! One run is one species at one temperature with one coefficient set: five layers, constant
//! diffusion coefficient per layer evaluated at the test temperature, kernel-only start, absorbing
//! outer surface. The release fraction is read at the requested times by linear interpolation
//! between the solver's time steps. A second run with twice the intervals and twice the steps
//! gives the numerical difference that is reported beside every model value.

use std::collections::BTreeMap;

use strato::config::{Config, InitialConfig, OutputConfig, RegionConfig, TimeConfig};
use strato::numerical::btcs::{run_input, validate_input, Limits, RunResult};
use strato::numerical::grid::Region;
use strato_data::Register;

use crate::data::{evaluate, DiffusivitySet, Evaluated, Radii, Species};

/// Seconds per hour.
pub const HOUR_S: f64 = 3600.0;

/// Numerical settings of a family of runs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// Intervals in kernel, buffer, IPyC, SiC, OPyC.
    pub intervals: [usize; 5],
    /// Requested time step, hours (the solver uses `end / ceil(end / dt)`).
    pub dt_h: f64,
}

impl Settings {
    /// The base settings of the comparison.
    pub const BASE: Settings = Settings {
        intervals: [160, 20, 20, 160, 20],
        dt_h: 0.05,
    };

    /// Twice the intervals and half the step.
    pub fn refined(self) -> Settings {
        Settings {
            intervals: self.intervals.map(|n| 2 * n),
            dt_h: self.dt_h / 2.0,
        }
    }
}

/// The solution of one run, reduced to what the comparison needs.
#[derive(Debug, Clone)]
pub struct Curve {
    /// Times in hours, from 0.
    pub t_h: Vec<f64>,
    /// Release fraction at those times.
    pub f: Vec<f64>,
    /// Release fraction of the refined run at its own times (twice as many).
    pub t_h_refined: Vec<f64>,
    /// Release fraction of the refined run.
    pub f_refined: Vec<f64>,
    /// The five coefficients used, m2/s.
    pub d_m2_s: [f64; 5],
    /// Worst relative balance residual of the base run.
    pub worst_balance_residual: f64,
    /// Largest derived bound of the relative balance residual of the base run.
    pub worst_balance_bound: f64,
    /// The base settings.
    pub settings: Settings,
    /// Fraction of the initial inventory still inside the particle, M / M0, at the base times.
    pub remaining: Vec<f64>,
    /// Relative balance residual (M + M_rel - M0) / M0 at the base times.
    pub residual_rel: Vec<f64>,
    /// Derived a-posteriori bound of the relative residual at the base times.
    pub bound_rel: Vec<f64>,
}

fn interpolate(t: &[f64], f: &[f64], x: f64) -> Option<f64> {
    let last = *t.last()?;
    if x.is_nan() || x < 0.0 || x > last * (1.0 + 1e-12) {
        return None;
    }
    let x = x.min(last);
    let k = t.partition_point(|v| *v <= x);
    if k == 0 {
        return Some(f[0]);
    }
    if k >= t.len() {
        return Some(f[t.len() - 1]);
    }
    let (t0, t1) = (t[k - 1], t[k]);
    let w = (x - t0) / (t1 - t0);
    Some(f[k - 1] + w * (f[k] - f[k - 1]))
}

impl Curve {
    /// Release fraction at `x` hours (linear interpolation in time); `None` beyond the run.
    pub fn at(&self, x_h: f64) -> Option<f64> {
        interpolate(&self.t_h, &self.f, x_h)
    }

    /// Release fraction of the refined run at `x` hours.
    pub fn at_refined(&self, x_h: f64) -> Option<f64> {
        interpolate(&self.t_h_refined, &self.f_refined, x_h)
    }

    /// Last time of the run, hours.
    pub fn end_h(&self) -> f64 {
        *self.t_h.last().unwrap_or(&0.0)
    }
}

/// Build the solver configuration of a run.
#[allow(clippy::too_many_arguments)]
pub fn config(
    name: &str,
    species: &str,
    t_c: f64,
    radii: &Radii,
    d: &[f64; 5],
    settings: Settings,
    end_h: f64,
    c_init: f64,
) -> Config {
    let regions = (0..5)
        .map(|k| RegionConfig {
            name: Region::ALL[k].name().to_string(),
            outer_radius_m: radii.outer_m[k],
            diffusivity_m2_s: d[k],
            cells: settings.intervals[k],
        })
        .collect();
    Config {
        case_name: name.to_string(),
        species: species.to_string(),
        temperature_k: t_c + 273.15,
        regions,
        initial: InitialConfig {
            kernel_concentration_mol_m3: c_init,
        },
        time: TimeConfig {
            dt_s: settings.dt_h * HOUR_S,
            end_time_s: end_h * HOUR_S,
            output_every_steps: 1,
        },
        output: OutputConfig {
            directory: String::new(),
        },
    }
}

/// Run the solver once.
pub fn run(cfg: &Config, snapshots_s: &[f64]) -> Result<RunResult, String> {
    let limits = Limits {
        max_intervals_total: 100_000,
        max_steps: 2_000_000,
    };
    let input = validate_input(cfg, &limits).map_err(|e| e.to_string())?;
    run_input(&input, snapshots_s).map_err(|e| e.to_string())
}

fn release_series(r: &RunResult) -> (Vec<f64>, Vec<f64>) {
    let t: Vec<f64> = r.series.iter().map(|(t, _)| *t / HOUR_S).collect();
    let f: Vec<f64> = r.series.iter().map(|(_, d)| d.release_fraction).collect();
    (t, f)
}

/// Run base and refined solutions of one species at one temperature.
#[allow(clippy::too_many_arguments)]
pub fn curve(
    name: &str,
    species: Species,
    t_c: f64,
    radii: &Radii,
    ev: &Evaluated,
    settings: Settings,
    end_h: f64,
    with_refined: bool,
) -> Result<Curve, String> {
    let m0 = 1.0;
    let cfg = config(
        name,
        species.label(),
        t_c,
        radii,
        &ev.d_m2_s,
        settings,
        end_h,
        m0,
    );
    let base = run(&cfg, &[])?;
    let (t_h, f) = release_series(&base);
    let m_init = base.meta.initial_inventory;
    let worst = base
        .series
        .iter()
        .map(|(_, r)| r.balance_residual.abs() / m_init)
        .fold(0.0, f64::max);
    let bound = base
        .series
        .iter()
        .map(|(_, r)| r.balance_residual_bound / m_init)
        .fold(0.0, f64::max);
    let (t_h_refined, f_refined) = if with_refined {
        let fine = settings.refined();
        let cfg2 = config(
            name,
            species.label(),
            t_c,
            radii,
            &ev.d_m2_s,
            fine,
            end_h,
            m0,
        );
        release_series(&run(&cfg2, &[])?)
    } else {
        (t_h.clone(), f.clone())
    };
    let remaining = base
        .series
        .iter()
        .map(|(_, r)| r.inventory / m_init)
        .collect();
    let residual_rel = base
        .series
        .iter()
        .map(|(_, r)| r.balance_residual / m_init)
        .collect();
    let bound_rel = base
        .series
        .iter()
        .map(|(_, r)| r.balance_residual_bound / m_init)
        .collect();
    Ok(Curve {
        t_h,
        f,
        t_h_refined,
        f_refined,
        d_m2_s: ev.d_m2_s,
        worst_balance_residual: worst,
        worst_balance_bound: bound,
        settings,
        remaining,
        residual_rel,
        bound_rel,
    })
}

/// A cache of curves, one per species, temperature and coefficient set.
pub struct Runner<'a> {
    /// The registers.
    pub reg: &'a Register,
    /// The registered radii.
    pub radii: Radii,
    /// Base settings.
    pub settings: Settings,
    /// Default sets by species.
    pub sets: BTreeMap<Species, DiffusivitySet>,
    curves: BTreeMap<(Species, i64), Curve>,
}

impl<'a> Runner<'a> {
    /// A runner with the default sets of the three elements.
    pub fn new(reg: &'a Register, radii: Radii, settings: Settings) -> Result<Self, String> {
        let mut sets = BTreeMap::new();
        for s in Species::ALL {
            sets.insert(s, crate::data::default_set(reg, s)?);
        }
        Ok(Runner {
            reg,
            radii,
            settings,
            sets,
            curves: BTreeMap::new(),
        })
    }

    /// The curve of a species at a temperature that reaches at least `end_h` hours; it is
    /// computed once, with the largest end time asked for beforehand (see `prepare`).
    pub fn get(&mut self, species: Species, t_c: f64, end_h: f64) -> Result<&Curve, String> {
        let key = (species, (t_c * 10.0).round() as i64);
        let need_new = match self.curves.get(&key) {
            Some(c) => c.end_h() < end_h * (1.0 - 1e-12),
            None => true,
        };
        if need_new {
            let set = self.sets[&species].clone();
            let ev = evaluate(self.reg, &set, t_c)?;
            let c = curve(
                &format!("AGR-1 {} {} degC", species.label(), t_c),
                species,
                t_c,
                &self.radii,
                &ev,
                self.settings,
                end_h.max(400.0),
                true,
            )?;
            self.curves.insert(key, c);
        }
        Ok(&self.curves[&key])
    }

    /// The curve of a species at a temperature, if it has been computed.
    pub fn lookup(&self, species: Species, t_c: f64) -> Option<&Curve> {
        self.curves.get(&(species, (t_c * 10.0).round() as i64))
    }

    /// All curves computed so far.
    pub fn curves(&self) -> impl Iterator<Item = (Species, f64, &Curve)> {
        self.curves
            .iter()
            .map(|((s, t10), c)| (*s, *t10 as f64 / 10.0, c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolation_is_linear_and_refuses_beyond_the_run() {
        let t = [0.0, 1.0, 2.0];
        let f = [0.0, 0.5, 1.0];
        assert_eq!(interpolate(&t, &f, 0.5), Some(0.25));
        assert_eq!(interpolate(&t, &f, 2.0), Some(1.0));
        assert_eq!(interpolate(&t, &f, 0.0), Some(0.0));
        assert_eq!(interpolate(&t, &f, 2.5), None);
        assert_eq!(interpolate(&t, &f, -1.0), None);
    }
}
