//! The studies of figures D to H: radial profiles, the inventory balance, spatial and temporal
//! convergence, and the temperature sensitivity of the registered correlations.

use strato::analytical::homogeneous_sphere::HomogeneousSphere;
use strato::analytical::sturm_liouville::{exact_release, Layer};
use strato::numerical::grid::{NodeClass, Region};
use strato_data::Register;

use crate::data::{evaluate, variant_sets, Evaluated, Radii, Species};
use crate::model::{config, curve, run, Settings, HOUR_S};

/// Analytical release fractions at sample times, per species and temperature (tenths of degC).
pub type AnalyticSeries = std::collections::BTreeMap<(Species, i64), Vec<(f64, Option<f64>)>>;

/// The analytical release fraction of the five-layer particle (independent of the solver), at
/// times in hours, where the series is usable.
pub fn analytical_reference(
    radii: &Radii,
    d: &[f64; 5],
    times_h: &[f64],
) -> (Vec<Option<f64>>, Option<String>) {
    let layers: Vec<Layer> = (0..5)
        .map(|k| Layer {
            outer_radius: radii.outer_m[k],
            diffusivity: d[k],
        })
        .collect();
    let times_s: Vec<f64> = times_h.iter().map(|t| t * HOUR_S).collect();
    match exact_release(&layers, &times_s, 20_000) {
        Ok(v) => (v, None),
        Err(e) => (vec![None; times_h.len()], Some(e.to_string())),
    }
}

/// One concentration profile.
#[derive(Debug, Clone)]
pub struct ProfileRow {
    /// Species label.
    pub species: Species,
    /// Test temperature, degC.
    pub t_c: f64,
    /// Time, hours.
    pub t_h: f64,
    /// Node index.
    pub node: usize,
    /// Radius, micrometre.
    pub r_um: f64,
    /// Region of the node (`a|b` at an interface).
    pub region: String,
    /// Concentration relative to the initial kernel concentration.
    pub c_over_c0: f64,
}

/// Radial profiles at selected times for one case.
pub fn profiles(
    species: Species,
    t_c: f64,
    radii: &Radii,
    ev: &Evaluated,
    settings: Settings,
    times_h: &[f64],
) -> Result<Vec<ProfileRow>, String> {
    let end = times_h.iter().copied().fold(0.0, f64::max);
    let cfg = config(
        &format!("profiles {} {t_c}", species.label()),
        species.label(),
        t_c,
        radii,
        &ev.d_m2_s,
        settings,
        end,
        1.0,
    );
    let snaps: Vec<f64> = times_h.iter().map(|t| t * HOUR_S).collect();
    let res = run(&cfg, &snaps)?;
    let mut out = Vec::new();
    for (t_s, c) in &res.snapshots {
        for (i, v) in c.iter().enumerate() {
            let region = match res.grid.node_class(i) {
                NodeClass::Centre => Region::Kernel.name().to_string(),
                NodeClass::Interior(r) => r.name().to_string(),
                NodeClass::Interface { inner, outer } => {
                    format!("{}|{}", inner.name(), outer.name())
                }
                NodeClass::Outer => Region::Opyc.name().to_string(),
            };
            out.push(ProfileRow {
                species,
                t_c,
                t_h: *t_s / HOUR_S,
                node: i,
                r_um: res.grid.r[i] * 1e6,
                region,
                c_over_c0: *v,
            });
        }
    }
    Ok(out)
}

/// One point of the temperature sensitivity.
#[derive(Debug, Clone)]
pub struct SensitivityRow {
    /// Species.
    pub species: Species,
    /// Variant identifier.
    pub variant_id: String,
    /// Variant description.
    pub variant_label: String,
    /// Coefficient class of the SiC rows of the variant.
    pub sic_class: String,
    /// Phase of use of the SiC rows of the variant.
    pub sic_phase: String,
    /// Temperature, degC.
    pub t_c: f64,
    /// The five coefficients, m2/s.
    pub d_m2_s: [f64; 5],
    /// Release fractions at 24, 100 and 300 h.
    pub f_24: f64,
    /// Release fraction at 100 h.
    pub f_100: f64,
    /// Release fraction at 300 h.
    pub f_300: f64,
    /// A printed validity range exists for some row of the variant.
    pub any_printed_range: bool,
    /// The temperature lies outside a printed range of some row.
    pub outside_printed_range: bool,
}

/// Temperatures of the sweep: the registered heating-test temperatures 1600 to 1800 degC in
/// steps of 25 degC. The default rows print no validity range; the sweep therefore stays inside
/// the temperatures the register records for the tests, and the rows that do print a range say
/// whether the temperature is outside it.
pub fn sweep_temperatures() -> Vec<f64> {
    (0..=8).map(|k| 1600.0 + 25.0 * k as f64).collect()
}

/// The sweep over temperature for every registered variant.
pub fn sensitivity(
    reg: &Register,
    radii: &Radii,
    settings: Settings,
) -> Result<Vec<SensitivityRow>, String> {
    let mut out = Vec::new();
    for species in Species::ALL {
        for set in variant_sets(reg, species)? {
            for t_c in sweep_temperatures() {
                let ev = evaluate(reg, &set, t_c)?;
                let c = curve(
                    &format!("sensitivity {} {} {t_c}", species.label(), set.id),
                    species,
                    t_c,
                    radii,
                    &ev,
                    settings,
                    300.0,
                    false,
                )?;
                let at = |h: f64| c.at(h).ok_or_else(|| "run too short".to_string());
                out.push(SensitivityRow {
                    species,
                    variant_id: set.id.clone(),
                    variant_label: set.label.clone(),
                    sic_class: set.sic_class.clone(),
                    sic_phase: set.sic_phase.clone(),
                    t_c,
                    d_m2_s: ev.d_m2_s,
                    f_24: at(24.0)?,
                    f_100: at(100.0)?,
                    f_300: at(300.0)?,
                    any_printed_range: ev.any_printed_range,
                    outside_printed_range: ev.outside_printed_range,
                });
            }
        }
    }
    Ok(out)
}

/// One refinement level of a convergence study.
#[derive(Debug, Clone)]
pub struct ConvergenceRow {
    /// Refinement factor relative to the coarsest level.
    pub level: usize,
    /// Mesh width of the kernel, metre, or time step, second.
    pub step: f64,
    /// Number of nodes (spatial) or steps (temporal).
    pub count: usize,
    /// Release fraction of the solver.
    pub f_solver: f64,
    /// Analytical release fraction.
    pub f_analytic: f64,
    /// Truncation bound the analytical series reports for itself.
    pub f_analytic_bound: f64,
    /// `|f_solver - f_analytic|`.
    pub error: f64,
    /// Observed order from this and the previous level, `None` for the first.
    pub order: Option<f64>,
}

/// The synthetic homogeneous verification case: all five regions share one diffusion coefficient,
/// chosen so that the mesh Fourier number `D t / R^2` is 0.1 at 100 h. It is a numerical
/// verification case, not physical data.
pub struct HomogeneousCase {
    /// Diffusion coefficient, m2/s.
    pub d: f64,
    /// End time, hours.
    pub end_h: f64,
    /// Outer radius, m.
    pub big_r: f64,
    /// Kernel radius, m.
    pub kernel_r: f64,
}

impl HomogeneousCase {
    /// The case on the registered radii.
    pub fn new(radii: &Radii) -> Self {
        let end_h = 100.0;
        let big_r = radii.outer_m[4];
        HomogeneousCase {
            d: 0.1 * big_r * big_r / (end_h * HOUR_S),
            end_h,
            big_r,
            kernel_r: radii.outer_m[0],
        }
    }

    fn analytic(&self) -> Result<(f64, f64), String> {
        let s = HomogeneousSphere::core(self.big_r, self.kernel_r, self.d, 1.0)
            .map_err(|e| e.to_string())?;
        let e = s
            .release_fraction(self.end_h * HOUR_S, 1e-14)
            .map_err(|e| e.to_string())?;
        Ok((e.value, e.error_bound))
    }

    fn solver(&self, radii: &Radii, intervals: [usize; 5], steps: usize) -> Result<f64, String> {
        let settings = Settings {
            intervals,
            dt_h: self.end_h / steps as f64,
        };
        let cfg = config(
            "homogeneous verification case",
            "synthetic",
            0.0 + 1000.0,
            radii,
            &[self.d; 5],
            settings,
            self.end_h,
            1.0,
        );
        let r = run(&cfg, &[])?;
        r.series
            .last()
            .map(|(_, d)| d.release_fraction)
            .ok_or_else(|| "empty series".to_string())
    }
}

fn orders(rows: &mut [ConvergenceRow], ratio: f64) {
    for k in 1..rows.len() {
        let (e0, e1) = (rows[k - 1].error, rows[k].error);
        rows[k].order = if e0 > 0.0 && e1 > 0.0 {
            Some((e0 / e1).ln() / ratio.ln())
        } else {
            None
        };
    }
}

/// Spatial convergence: the intervals of every region are multiplied by 1, 2, 4, ... while the
/// time-step error is removed by Richardson extrapolation of two step sizes (the scheme is first
/// order in time): `F = 2 F(dt/2) - F(dt)`. The error is measured against the analytical series.
pub fn spatial_convergence(
    radii: &Radii,
    levels: usize,
    base_steps: usize,
) -> Result<Vec<ConvergenceRow>, String> {
    let case = HomogeneousCase::new(radii);
    let (fa, bound) = case.analytic()?;
    let base = [4usize, 3, 2, 4, 2];
    let mut rows = Vec::new();
    for lvl in 0..levels {
        let m = 1usize << lvl;
        let n = base.map(|b| b * m);
        let f1 = case.solver(radii, n, base_steps)?;
        let f2 = case.solver(radii, n, 2 * base_steps)?;
        let f = 2.0 * f2 - f1;
        let nodes: usize = n.iter().sum::<usize>() + 1;
        rows.push(ConvergenceRow {
            level: m,
            step: radii.outer_m[0] / n[0] as f64,
            count: nodes,
            f_solver: f,
            f_analytic: fa,
            f_analytic_bound: bound,
            error: (f - fa).abs(),
            order: None,
        });
    }
    orders(&mut rows, 2.0);
    Ok(rows)
}

/// Temporal convergence on a fixed fine mesh: the number of steps doubles from level to level.
/// The error against the analytical series contains the fixed spatial error of the mesh; the
/// solver's own difference to the finest step is reported by the caller from the same rows.
pub fn temporal_convergence(
    radii: &Radii,
    levels: usize,
    base_steps: usize,
    mesh_multiplier: usize,
) -> Result<Vec<ConvergenceRow>, String> {
    let case = HomogeneousCase::new(radii);
    let (fa, bound) = case.analytic()?;
    let n = [4usize, 3, 2, 4, 2].map(|b| b * mesh_multiplier);
    let mut rows = Vec::new();
    for lvl in 0..levels {
        let steps = base_steps << lvl;
        let f = case.solver(radii, n, steps)?;
        rows.push(ConvergenceRow {
            level: 1 << lvl,
            step: case.end_h * HOUR_S / steps as f64,
            count: steps,
            f_solver: f,
            f_analytic: fa,
            f_analytic_bound: bound,
            error: (f - fa).abs(),
            order: None,
        });
    }
    orders(&mut rows, 2.0);
    Ok(rows)
}

/// The spatial error of the fixed mesh used by the temporal study, from the spatial study at the
/// same multiplier, so the reader can see where the temporal curve flattens.
pub fn spatial_floor(radii: &Radii, mesh_multiplier: usize, steps: usize) -> Result<f64, String> {
    let case = HomogeneousCase::new(radii);
    let (fa, _) = case.analytic()?;
    let n = [4usize, 3, 2, 4, 2].map(|b| b * mesh_multiplier);
    let f1 = case.solver(radii, n, steps)?;
    let f2 = case.solver(radii, n, 2 * steps)?;
    Ok(((2.0 * f2 - f1) - fa).abs())
}

/// Diffusion coefficient and end time of the verification case, for the files.
pub fn homogeneous_case_description(radii: &Radii) -> (f64, f64, f64) {
    let c = HomogeneousCase::new(radii);
    (c.d, c.end_h, c.big_r)
}
