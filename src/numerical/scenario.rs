//! The scenario engine: the BTCS solver with generation, decay and temperature histories run
//! through a prescribed-input scenario (science design Section 5).
//!
//! A scenario is a pair of histories, the kernel generation factor `G_kernel_factor(t)` (a
//! multiple of the reference generation rate) and the particle temperature `T_C(t)` in degrees
//! Celsius, together with a step plan. Nothing else changes: the boundary and interface conditions,
//! the geometry and the physics of Documents 2 and 5 stay as they are. The engine does not model
//! neutron kinetics, control-rod motion, thermal-hydraulics, power redistribution or mechanical
//! refuelling; a scenario that sets the factor to zero is a particle without generation, nothing
//! more. The result is a numerical solution of the stated model, not a validated reactor
//! prediction and not experimental validation.
//!
//! The time loop is the one of [`super::physics_run::run_physics`] (step average of the source
//! (F32), nodal weights (F33), (F34), the matrix with the decay number on every diagonal entry
//! (F35), (F37), the diffusivity at the middle of the step (F36), the right-hand side (F38), the
//! release with the outer-node source (F39), the amounts (F40), the balance (F41) and its rounding
//! bound (F46)), with these differences, each of which the science design asks for:
//!
//! * the step is piecewise uniform: every history breakpoint, event time and requested output
//!   time is a segment boundary, a segment of length `L` with the stated largest step gets
//!   `n = ceil(L / dt_max)` equal steps (decision 8 applied per segment), and the matrix is
//!   reassembled where the diffusivity or the step changes (Document 2 Section 11.2; Document 5
//!   Section 4.2: in a sum over steps, `dt` and `kappa` are those of the step that produced the
//!   term);
//! * the start may be a fresh particle (`C = 0`, so `M^0 = 0` and the release fraction is
//!   `M_rel / (M^0 + M_gen)`, reported once `M_gen > 0`), the kernel-only start of the baseline,
//!   or a given profile (the state of a preceding run);
//! * the diffusivity comes from a caller-supplied function of the temperature in degrees Celsius
//!   (the engine reads no file and chooses no correlation) that also states, per material,
//!   whether the temperature lies inside the stated range of the law.
//!
//! A scenario that starts from the state of a preceding scenario at an event time is one
//! continuous history from the fresh particle ([`ScenarioDefinition::with_lead_in`]), so the
//! balance (F41) is checked over the whole run.
//!
//! Like the rest of the solver the module uses only `std`: no files, clocks or threads, so it
//! builds for `wasm32-unknown-unknown`.

use super::{
    btcs::Stepper,
    diagnostics::{check_matrix_structure, BalanceBound},
    grid::Grid,
    inventory::{initial_vector, inventory, Accumulator},
    physics_run::{combined_balance_bound, source_nodes, BoundTerms, Interpolation, TimeFunction},
};
use anyhow::{bail, Result};
use std::f64::consts::PI;

/// Seconds in a day and in an hour (conversions of the data files).
pub const SECONDS_PER_DAY: f64 = 86_400.0;
/// Seconds in an hour.
pub const SECONDS_PER_HOUR: f64 = 3_600.0;

/// Where a temperature lies against the stated range of the diffusivity law of one material.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangeCode {
    /// Inside the printed range.
    Within,
    /// The law states no range.
    NotStated,
    /// Outside the stated range: the value is an extrapolation of the correlation.
    Outside,
}

/// Per material: in how many steps the temperature lay inside the stated range of the law,
/// outside it, or the law stated no range.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RangeCounts {
    /// Steps inside the stated range.
    pub within: usize,
    /// Steps outside the stated range.
    pub outside: usize,
    /// Steps of a law that states no range.
    pub not_stated: usize,
}

/// The diffusivities of the five materials at one temperature and where that temperature lies
/// against each law's range.
pub type DiffusivityAt = ([f64; 5], [RangeCode; 5]);

/// How the diffusivities are obtained.
pub enum Diffusion<'a> {
    /// Constant in time.
    Constant([f64; 5]),
    /// A function of the time in s and the temperature in degrees Celsius, both at the middle of\n    /// each step (the time lets the caller switch the correlation set between phases of a\n    /// scenario; the engine itself chooses nothing).
    OfTemperature(&'a dyn Fn(f64, f64) -> Result<DiffusivityAt>),
}

/// One segment of the step plan: up to `end_s` no step exceeds `dt_max_s`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepSegment {
    /// End of the segment in s (the segments are contiguous from 0).
    pub end_s: f64,
    /// Largest step in s inside the segment (a numerical setting, refinement-checked by the
    /// caller).
    pub dt_max_s: f64,
}

/// A scenario: two histories and a step plan. All times are in s from the start of the run.
#[derive(Clone, Debug)]
pub struct ScenarioDefinition {
    /// Identifier.
    pub id: String,
    /// `G_kernel_factor(t)`, dimensionless; the kernel source is this factor times the reference
    /// generation rate of the configuration. Linear or step history.
    pub generation_factor: TimeFunction,
    /// `T_C(t)` in degrees Celsius.
    pub temperature_c: TimeFunction,
    /// The step plan.
    pub plan: Vec<StepSegment>,
    /// Further times that must be step boundaries (an event such as shutdown).
    pub event_times_s: Vec<f64>,
}

fn breakpoints(f: &TimeFunction) -> Vec<f64> {
    match f {
        TimeFunction::Constant(_) => vec![],
        TimeFunction::Table { times, .. } => times.clone(),
    }
}

fn shifted(f: &TimeFunction, shift_s: f64) -> Result<(Vec<f64>, Vec<f64>, Interpolation)> {
    match f {
        TimeFunction::Constant(v) => Ok((vec![shift_s], vec![*v], Interpolation::Step)),
        TimeFunction::Table {
            times,
            values,
            mode,
        } => Ok((
            times.iter().map(|t| shift_s + t).collect(),
            values.clone(),
            *mode,
        )),
    }
}

/// Joins a lead-in history that ends at `t_e` with the history of the tail, whose time zero is
/// `t_e`. The joined table keeps the lead-in values up to `t_e` and the tail values after it. A
/// jump at `t_e` (a step of the generation to zero) is written as two points at the same time
/// in effect, by a step entry: the table is a step table when either part is.
fn join(lead: &TimeFunction, t_e: f64, tail: &TimeFunction) -> Result<TimeFunction> {
    let (lt, lv, lm) = shifted(lead, 0.0)?;
    let (tt, tv, tm) = shifted(tail, t_e)?;
    let mut times: Vec<f64> = Vec::new();
    let mut values: Vec<f64> = Vec::new();
    for (t, v) in lt.iter().zip(lv.iter()) {
        if *t < t_e {
            times.push(*t);
            values.push(*v);
        }
    }
    if times.is_empty() {
        times.push(0.0);
        values.push(lv[0]);
    }
    // The value of the lead-in at t_e, to continue a linear lead-in to the join.
    let lead_end = lead.value_at(t_e);
    if lm == Interpolation::Linear && tm == Interpolation::Linear {
        if *times.last().unwrap() < t_e {
            times.push(t_e);
            values.push(lead_end);
        }
        if tv[0] != lead_end {
            bail!(
                "a linear lead-in and a linear tail must meet at the join (lead-in {lead_end}, tail {})",
                tv[0]
            );
        }
        for (t, v) in tt.iter().zip(tv.iter()).skip(1) {
            times.push(*t);
            values.push(*v);
        }
        TimeFunction::table(times, values, Interpolation::Linear)
    } else {
        // At least one part is a step history: the joined history is a step history, and a
        // linear piece is not allowed to straddle the join.
        if lm == Interpolation::Linear && !lead.is_constant() {
            bail!("a varying linear lead-in cannot be joined to a step tail");
        }
        for (t, v) in tt.iter().zip(tv.iter()) {
            times.push(*t);
            values.push(*v);
        }
        TimeFunction::table(times, values, Interpolation::Step)
    }
}

impl ScenarioDefinition {
    /// All times that are step boundaries: 0, every breakpoint of both histories, every event
    /// time, every end of a plan segment; ascending, with times closer than `1e-12` of the
    /// run length taken as one.
    ///
    /// # Errors
    ///
    /// An empty plan, a plan that is not increasing, or a non-positive step.
    pub fn boundaries(&self, extra_s: &[f64]) -> Result<Vec<f64>> {
        if self.plan.is_empty() {
            bail!("the step plan is empty");
        }
        let mut last = 0.0;
        for s in &self.plan {
            if !(s.end_s > last && s.dt_max_s > 0.0 && s.dt_max_s.is_finite()) {
                bail!("the step plan must be increasing with positive finite steps");
            }
            last = s.end_s;
        }
        let end = last;
        let mut all = vec![0.0];
        all.extend(self.plan.iter().map(|s| s.end_s));
        all.extend(breakpoints(&self.generation_factor));
        all.extend(breakpoints(&self.temperature_c));
        all.extend(self.event_times_s.iter().copied());
        all.extend(extra_s.iter().copied());
        all.retain(|t| t.is_finite() && *t >= 0.0 && *t <= end * (1.0 + 1e-12));
        all.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        let mut out: Vec<f64> = Vec::with_capacity(all.len());
        for t in all {
            let t = t.min(end);
            match out.last() {
                Some(p) if (t - p).abs() <= 1e-12 * end => {}
                _ => out.push(t),
            }
        }
        if let Some(l) = out.last_mut() {
            *l = end;
        }
        Ok(out)
    }

    fn dt_max_at(&self, t: f64) -> f64 {
        for s in &self.plan {
            if t < s.end_s {
                return s.dt_max_s;
            }
        }
        self.plan[self.plan.len() - 1].dt_max_s
    }

    /// The step sequence for the given boundaries: `(t_a, t_b)` of every step. Inside a segment
    /// the steps are equal, `dt = L / n` with `n = ceil(L / dt_max)`; the last step of a segment
    /// ends exactly at the boundary, so no step straddles a breakpoint.
    ///
    /// # Errors
    ///
    /// As [`ScenarioDefinition::boundaries`].
    pub fn steps(&self, extra_s: &[f64]) -> Result<Vec<(f64, f64)>> {
        let b = self.boundaries(extra_s)?;
        let mut out = Vec::new();
        for w in b.windows(2) {
            let (a, c) = (w[0], w[1]);
            let l = c - a;
            let dtm = self.dt_max_at(0.5 * (a + c));
            let n = ((l / dtm) * (1.0 - 1e-12)).ceil().max(1.0) as usize;
            let mut prev = a;
            for k in 1..=n {
                let t = if k == n {
                    c
                } else {
                    a + l * (k as f64) / (n as f64)
                };
                out.push((prev, t));
                prev = t;
            }
        }
        Ok(out)
    }

    /// The run length in s.
    pub fn end_s(&self) -> f64 {
        self.plan.last().map_or(0.0, |s| s.end_s)
    }

    /// The scenario that follows `lead` up to `t_e` (taken as is) and continues with `self`,
    /// whose time zero is moved to `t_e`: one continuous history. `lead` must end at or after
    /// `t_e`. The plan of the lead-in applies up to `t_e`, the plan of `self` after it.
    ///
    /// # Errors
    ///
    /// A lead-in that ends before `t_e`, or histories that cannot be joined.
    pub fn with_lead_in(&self, lead: &ScenarioDefinition, t_e: f64) -> Result<ScenarioDefinition> {
        if !(t_e > 0.0) || lead.end_s() < t_e * (1.0 - 1e-12) {
            bail!("the lead-in must reach the join time");
        }
        let mut plan: Vec<StepSegment> = Vec::new();
        for s in &lead.plan {
            if s.end_s < t_e * (1.0 - 1e-12) {
                plan.push(*s);
            } else {
                plan.push(StepSegment {
                    end_s: t_e,
                    dt_max_s: s.dt_max_s,
                });
                break;
            }
        }
        for s in &self.plan {
            plan.push(StepSegment {
                end_s: t_e + s.end_s,
                dt_max_s: s.dt_max_s,
            });
        }
        let mut events: Vec<f64> = lead
            .event_times_s
            .iter()
            .copied()
            .filter(|t| *t < t_e)
            .collect();
        events.push(t_e);
        events.extend(self.event_times_s.iter().map(|t| t_e + t));
        Ok(ScenarioDefinition {
            id: self.id.clone(),
            generation_factor: join(&lead.generation_factor, t_e, &self.generation_factor)?,
            temperature_c: join(&lead.temperature_c, t_e, &self.temperature_c)?,
            plan,
            event_times_s: events,
        })
    }

    /// The reference run of normal operation for comparison: the same step plan and the same
    /// step boundaries, with the generation factor and the temperature of normal operation
    /// (`factor` and `temperature_c`, constants) for the whole run. A scenario whose histories
    /// equal these values everywhere runs the same floating-point operations in the same order
    /// and gives the same numbers bit for bit.
    ///
    /// # Errors
    ///
    /// A value that is not finite.
    pub fn normal_operation(&self, factor: f64, temperature_c: f64) -> Result<ScenarioDefinition> {
        let mut def = self.clone();
        def.id = format!("{}-normal-operation", self.id);
        def.generation_factor = TimeFunction::constant(factor)?;
        def.temperature_c = TimeFunction::constant(temperature_c)?;
        def.event_times_s = self.boundaries(&[])?;
        Ok(def)
    }
}

/// The initial state.
#[derive(Clone, Debug)]
pub enum InitialState {
    /// `C = 0` everywhere (a fresh particle, `M^0 = 0`).
    Fresh,
    /// The kernel-only start of the baseline (F21), (F22): `C = C_init` in the kernel, the
    /// kernel-surface node with the inventory-matched value.
    KernelUniform {
        /// Initial kernel concentration in mol/m^3.
        c_init: f64,
    },
    /// The state of a preceding run: the nodal profile (length `N + 1`, mol/m^3). Its inventory is
    /// the initial inventory `M^0` of this run, and the balance starts again from it.
    Profile(Vec<f64>),
}

/// The configuration of a run, besides the scenario.
pub struct ScenarioConfig<'a> {
    /// Outer radii of kernel, buffer, IPyC, SiC and OPyC in m.
    pub radii_m: [f64; 5],
    /// Intervals per material.
    pub intervals: [usize; 5],
    /// The initial state.
    pub initial: InitialState,
    /// Decay constant in s^-1 (0 for a stable species).
    pub decay_constant_per_s: f64,
    /// Reference kernel generation rate in mol m^-3 s^-1, `y F_dot / N_A` (Document 5 (2)); it is
    /// multiplied by the generation factor of the scenario. Zero for no generation.
    pub reference_generation_mol_m3_s: f64,
    /// The diffusivities.
    pub diffusion: Diffusion<'a>,
    /// Times (s) at which a profile is stored; each is a step boundary.
    pub profile_times_s: Vec<f64>,
    /// Further times that must be step boundaries (requested output times).
    pub extra_boundaries_s: Vec<f64>,
}

/// One output row, at the end of a step (row 0 is the initial state).
#[derive(Clone, Debug)]
pub struct ScenarioRow {
    /// Time in s.
    pub t_s: f64,
    /// Inventory `M` in the particle, mol.
    pub inventory_mol: f64,
    /// `M_gen`, mol.
    pub generated_mol: f64,
    /// `M_rel`, mol.
    pub released_mol: f64,
    /// `M_dec`, mol.
    pub decayed_mol: f64,
    /// `eps = M + M_rel - (M^0 + M_gen) + M_dec` (F41), mol.
    pub balance_residual_mol: f64,
    /// The rounding bound (F46) of `|eps|`, mol.
    pub balance_bound_mol: f64,
    /// `F_gen = M_rel / (M^0 + M_gen)` (F42); `None` while `M^0 + M_gen = 0`.
    pub release_fraction_of_total: Option<f64>,
    /// `M_rel / M^0`; `None` for a fresh start.
    pub release_fraction_of_initial: Option<f64>,
    /// `beta = Phi / (generation rate)` (F42); `None` at row 0 and while the generation rate is 0.
    pub release_to_generation_ratio: Option<f64>,
    /// Temperature at the end of the step in degrees Celsius.
    pub temperature_c: f64,
    /// Generation factor at the end of the step.
    pub generation_factor: f64,
    /// Kernel source density `G_kernel` of the step that ended here, mol m^-3 s^-1.
    pub kernel_source_mol_m3_s: f64,
    /// Diffusivities used in the step that ended here, m^2/s.
    pub d_m2_s: [f64; 5],
    /// Smallest nodal concentration, mol/m^3.
    pub min_concentration: f64,
}

/// A finished run.
#[derive(Clone, Debug)]
pub struct ScenarioRun {
    /// The grid.
    pub grid: Grid,
    /// Rows including row 0.
    pub rows: Vec<ScenarioRow>,
    /// Profiles at the requested times.
    pub profiles: Vec<(f64, Vec<f64>)>,
    /// Profile at the end.
    pub final_profile: Vec<f64>,
    /// `M^0` in mol.
    pub initial_inventory_mol: f64,
    /// Number of steps.
    pub steps: usize,
    /// Number of distinct step lengths.
    pub distinct_steps: usize,
    /// Matrix assemblies.
    pub assemblies: usize,
    /// Smallest row margin of the matrices (P1, P24: at least 1).
    pub matrix_min_row_margin: f64,
    /// Smallest `m_i - |U_i|` of the factorisations.
    pub min_pivot_margin: f64,
    /// Steps per material by position against the stated range of the law.
    pub ranges: [RangeCounts; 5],
    /// Largest `|eps| / bound` over the rows with a positive bound.
    pub max_residual_over_bound: f64,
    /// Largest `|eps| / (M^0 + M_gen)` over the rows with a positive total.
    pub max_relative_residual: f64,
}

impl ScenarioRun {
    /// Row at the step boundary `t_s` (relative tolerance `1e-12` of the run length), if any.
    pub fn row_at(&self, t_s: f64) -> Option<&ScenarioRow> {
        let end = self.rows.last()?.t_s;
        self.rows
            .iter()
            .find(|r| (r.t_s - t_s).abs() <= 1e-12 * end)
    }
}

/// Runs a scenario.
///
/// # Errors
///
/// Invalid histories or plan, a diffusivity that is not finite and positive, a failed solve, a
/// non-finite output row, or a run so long that the bound (F46) does not apply (`eta > 1`).
pub fn run_scenario(def: &ScenarioDefinition, cfg: &ScenarioConfig<'_>) -> Result<ScenarioRun> {
    let grid = Grid::from_radii(cfg.radii_m, cfg.intervals)?;
    let n = grid.n;
    let lambda = cfg.decay_constant_per_s;
    if !(lambda.is_finite() && lambda >= 0.0) {
        bail!("the decay constant must be finite and not negative");
    }
    let g_ref = cfg.reference_generation_mol_m3_s;
    if !(g_ref.is_finite() && g_ref >= 0.0) {
        bail!("the reference generation rate must be finite and not negative");
    }
    if def.generation_factor.min_value() < 0.0 {
        bail!("the generation factor must not be negative");
    }
    let end = def.end_s();
    if !def.generation_factor.covers(end) || !def.temperature_c.covers(end) {
        bail!("a history ends before the end of the run");
    }
    let mut extra = cfg.extra_boundaries_s.clone();
    extra.extend(cfg.profile_times_s.iter().copied());
    let steps = def.steps(&extra)?;
    let eta_last = 4.0 * (3.0 * n as f64 + steps.len() as f64 + 10.0) * BalanceBound::U;
    if eta_last > 1.0 {
        bail!("the run has too many steps for the rounding bound (eta > 1)");
    }
    let p_hat = 4.0 * PI;

    let (mut c_old, m0) = match &cfg.initial {
        InitialState::Fresh => (vec![0.0; n + 1], 0.0),
        InitialState::KernelUniform { c_init } => {
            if !(c_init.is_finite() && *c_init > 0.0) {
                bail!("the initial kernel concentration must be finite and positive");
            }
            let (c, _theta, m0) = initial_vector(&grid, *c_init);
            (c, m0)
        }
        InitialState::Profile(p) => {
            if p.len() != n + 1 || p.iter().any(|x| !x.is_finite() || *x < 0.0) {
                bail!("the initial profile must have N + 1 finite non-negative values");
            }
            let mut c = p.clone();
            c[n] = 0.0;
            let m0 = inventory(&grid, &c);
            (c, m0)
        }
    };
    let t_of = |t: f64| def.temperature_c.value_at(t);
    let d_at = |t_s: f64, t_c: f64| -> Result<DiffusivityAt> {
        match &cfg.diffusion {
            Diffusion::Constant(d) => Ok((*d, [RangeCode::NotStated; 5])),
            Diffusion::OfTemperature(f) => f(t_s, t_c),
        }
    };

    let mut sum_s = 0.0;
    let mut chi_inventory = 0.0;
    let mut c_new = vec![0.0; n + 1];
    let mut released = Accumulator::new();
    let mut generated = Accumulator::new();
    let mut decayed = Accumulator::new();
    let mut m_current = inventory(&grid, &c_old);
    let mut stepper: Option<Stepper> = None;
    let mut last_key: Option<([u64; 5], u64)> = None;
    let mut dt_seen: Vec<u64> = Vec::new();
    let mut ranges = [RangeCounts::default(); 5];
    let mut assemblies = 0usize;
    let mut meta_margin = f64::INFINITY;
    let mut meta_pivot = f64::INFINITY;
    let mut last_gbar: Option<[u64; 5]> = None;
    let mut nodes = source_nodes(&grid, [0.0; 5]);
    let mut max_over_bound: f64 = 0.0;
    let mut max_rel: f64 = 0.0;

    let (cmin0, _) = min_max(&c_old);
    let (d0, _) = d_at(0.0, t_of(0.0))?;
    let bound0 = combined_balance_bound(&BoundTerms {
        n,
        steps_taken: 0,
        sum_s: 0.0,
        chi_inventory: 0.0,
        inventory: m0,
        initial_inventory: m0,
        released: 0.0,
        generated: 0.0,
        decayed: 0.0,
    });
    let make_row = |t: f64,
                    m: f64,
                    rel: f64,
                    gen: f64,
                    dec: f64,
                    bound: f64,
                    phi: Option<f64>,
                    gen_rate: f64,
                    gfac: f64,
                    gk: f64,
                    d: [f64; 5],
                    cmin: f64|
     -> ScenarioRow {
        let total = m0 + gen;
        let eps = ((m + rel) - total) + dec;
        ScenarioRow {
            t_s: t,
            inventory_mol: m,
            generated_mol: gen,
            released_mol: rel,
            decayed_mol: dec,
            balance_residual_mol: eps,
            balance_bound_mol: bound,
            release_fraction_of_total: (total > 0.0).then(|| rel / total),
            release_fraction_of_initial: (m0 > 0.0).then(|| rel / m0),
            release_to_generation_ratio: match phi {
                Some(p) if gen_rate > 0.0 => Some(p / gen_rate),
                _ => None,
            },
            temperature_c: t_of(t),
            generation_factor: gfac,
            kernel_source_mol_m3_s: gk,
            d_m2_s: d,
            min_concentration: cmin,
        }
    };
    let mut rows = vec![make_row(
        0.0,
        m_current,
        0.0,
        0.0,
        0.0,
        bound0,
        None,
        0.0,
        def.generation_factor.value_at(0.0),
        0.0,
        d0,
        cmin0,
    )];
    let mut profile_wanted: Vec<f64> = cfg.profile_times_s.clone();
    profile_wanted.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    let mut profiles: Vec<(f64, Vec<f64>)> = Vec::new();
    if profile_wanted.first().is_some_and(|t| *t <= 0.0) {
        profiles.push((0.0, c_old.clone()));
    }
    let mut next_profile = profiles.len();

    for (step_index, &(t_a, t_b)) in steps.iter().enumerate() {
        let step = step_index + 1;
        let dt = t_b - t_a;
        let kappa = lambda * dt;
        // (F36): the diffusivities of the step, from the temperature at its middle.
        let t_mid_c = t_of(0.5 * (t_a + t_b));
        let (d_step, codes) = d_at(0.5 * (t_a + t_b), t_mid_c)?;
        for (k, d) in d_step.iter().enumerate() {
            if !(d.is_finite() && *d > 0.0) {
                bail!("the diffusivity of material {k} at {t_mid_c} C is not finite and positive");
            }
            match codes[k] {
                RangeCode::Within => ranges[k].within += 1,
                RangeCode::NotStated => ranges[k].not_stated += 1,
                RangeCode::Outside => ranges[k].outside += 1,
            }
        }
        let key = (
            std::array::from_fn::<u64, 5, _>(|k| d_step[k].to_bits()),
            dt.to_bits(),
        );
        if stepper.is_none() || last_key != Some(key) {
            let s = Stepper::new_with_decay(&grid, d_step, dt, kappa)?;
            let rep = check_matrix_structure(&s.assembly().matrix)?;
            meta_margin = meta_margin.min(rep.min_row_margin);
            let pivots = s.factors().pivots();
            let upper = &s.assembly().matrix.upper;
            for (m_i, u_i) in pivots.iter().zip(upper.iter()).take(n - 1) {
                meta_pivot = meta_pivot.min(m_i - u_i.abs());
            }
            assemblies += 1;
            last_key = Some(key);
            if !dt_seen.contains(&key.1) {
                dt_seen.push(key.1);
            }
            stepper = Some(s);
        }
        let st = stepper.as_mut().expect("assembled above");

        // (F32) to (F34): the source of the step.
        let gfac_mean = def.generation_factor.mean_over(t_a, t_b);
        let gk = gfac_mean * g_ref;
        let gbar = [gk, 0.0, 0.0, 0.0, 0.0];
        let gbar_bits: [u64; 5] = std::array::from_fn(|k| gbar[k].to_bits());
        if last_gbar != Some(gbar_bits) {
            nodes = source_nodes(&grid, gbar);
            last_gbar = Some(gbar_bits);
        }
        st.step_with_source(&c_old, &nodes.g, &mut c_new)?;

        // (F39), (F40).
        let phi = p_hat * st.k_last() * c_new[n - 1] + p_hat * nodes.q[n];
        let release_increment = dt * phi;
        released.add(release_increment);
        let gen_rate = p_hat * nodes.sum_q;
        generated.add(dt * gen_rate);
        let m_old = m_current;
        m_current = inventory(&grid, &c_new);
        if kappa != 0.0 {
            decayed.add(kappa * m_current);
        }
        // Bound bookkeeping (F43), (F46).
        {
            let a = &st.assembly().matrix;
            let mut s = 0.0;
            for i in 1..n {
                let left = a.lower[i].abs() * c_new[i - 1].abs();
                let right = a.upper[i].abs() * c_new[i + 1].abs();
                s += grid.v[i] * (left + a.diag[i].abs() * c_new[i].abs() + right);
            }
            sum_s += s;
            if nodes.active {
                chi_inventory += m_old;
            }
        }
        std::mem::swap(&mut c_old, &mut c_new);
        let bound = combined_balance_bound(&BoundTerms {
            n,
            steps_taken: step,
            sum_s,
            chi_inventory,
            inventory: m_current,
            initial_inventory: m0,
            released: released.value(),
            generated: generated.value(),
            decayed: decayed.value(),
        });
        let (cmin, _) = min_max(&c_old);
        let row = make_row(
            t_b,
            m_current,
            released.value(),
            generated.value(),
            decayed.value(),
            bound,
            Some(phi),
            gen_rate,
            def.generation_factor.value_at(t_b),
            gk,
            d_step,
            cmin,
        );
        let finite = row.inventory_mol.is_finite()
            && row.released_mol.is_finite()
            && row.balance_residual_mol.is_finite()
            && bound.is_finite()
            && cmin.is_finite()
            && phi.is_finite();
        if !finite {
            bail!("non-finite output at step {step}");
        }
        if bound > 0.0 {
            max_over_bound = max_over_bound.max(row.balance_residual_mol.abs() / bound);
        }
        let total = m0 + row.generated_mol;
        if total > 0.0 {
            max_rel = max_rel.max(row.balance_residual_mol.abs() / total);
        }
        rows.push(row);
        while next_profile < profile_wanted.len()
            && profile_wanted[next_profile] <= t_b * (1.0 + 1e-12)
            && profile_wanted[next_profile] > t_a
        {
            profiles.push((t_b, c_old.clone()));
            next_profile += 1;
        }
    }

    Ok(ScenarioRun {
        grid,
        rows,
        profiles,
        final_profile: c_old,
        initial_inventory_mol: m0,
        steps: steps.len(),
        distinct_steps: dt_seen.len(),
        assemblies,
        matrix_min_row_margin: meta_margin,
        min_pivot_margin: meta_pivot,
        ranges,
        max_residual_over_bound: max_over_bound,
        max_relative_residual: max_rel,
    })
}

fn min_max(c: &[f64]) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &x in c {
        lo = lo.min(x);
        hi = hi.max(x);
    }
    (lo, hi)
}

/// The post-event release fraction of Section 5.1,
/// `F_post(t) = (M_rel(t) - M_rel(t_e)) / M(t_e)` for `t >= t_e`; `None` if `t_e` is not a row
/// of the run, `t` precedes it, or `M(t_e)` is not positive.
pub fn post_event_fraction(run: &ScenarioRun, t_e_s: f64, t_s: f64) -> Option<f64> {
    let e = run.row_at(t_e_s)?;
    let r = run.row_at(t_s)?;
    if r.t_s < e.t_s || !(e.inventory_mol > 0.0) {
        return None;
    }
    Some((r.released_mol - e.released_mol) / e.inventory_mol)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(plan: Vec<StepSegment>) -> ScenarioDefinition {
        ScenarioDefinition {
            id: "t".into(),
            generation_factor: TimeFunction::constant(1.0).unwrap(),
            temperature_c: TimeFunction::constant(1000.0).unwrap(),
            plan,
            event_times_s: vec![],
        }
    }

    #[test]
    fn a_segment_gets_equal_steps_not_above_the_maximum() {
        let d = def(vec![StepSegment {
            end_s: 10.0,
            dt_max_s: 3.0,
        }]);
        let s = d.steps(&[]).unwrap();
        assert_eq!(s.len(), 4);
        assert!((s[0].1 - s[0].0 - 2.5).abs() < 1e-15);
        assert_eq!(s.last().unwrap().1, 10.0);
    }

    #[test]
    fn breakpoints_and_events_are_boundaries() {
        let mut d = def(vec![StepSegment {
            end_s: 10.0,
            dt_max_s: 4.0,
        }]);
        d.generation_factor = TimeFunction::table(
            vec![0.0, 3.0, 10.0],
            vec![0.0, 1.0, 1.0],
            Interpolation::Linear,
        )
        .unwrap();
        d.event_times_s = vec![7.0];
        let b = d.boundaries(&[]).unwrap();
        assert_eq!(b, vec![0.0, 3.0, 7.0, 10.0]);
        let s = d.steps(&[]).unwrap();
        assert!(s.iter().any(|x| x.1 == 3.0) && s.iter().any(|x| x.1 == 7.0));
        assert!(s.iter().all(|x| x.1 - x.0 <= 4.0));
    }

    #[test]
    fn a_lead_in_is_joined_into_one_history() {
        let lead = def(vec![StepSegment {
            end_s: 10.0,
            dt_max_s: 5.0,
        }]);
        let mut tail = def(vec![StepSegment {
            end_s: 4.0,
            dt_max_s: 2.0,
        }]);
        tail.generation_factor =
            TimeFunction::table(vec![0.0, 4.0], vec![0.0, 0.0], Interpolation::Step).unwrap();
        let j = tail.with_lead_in(&lead, 10.0).unwrap();
        assert_eq!(j.end_s(), 14.0);
        assert_eq!(j.generation_factor.value_at(9.0), 1.0);
        assert_eq!(j.generation_factor.value_at(10.0), 0.0);
        assert_eq!(j.generation_factor.value_at(12.0), 0.0);
        assert!(j.boundaries(&[]).unwrap().contains(&10.0));
    }
}
