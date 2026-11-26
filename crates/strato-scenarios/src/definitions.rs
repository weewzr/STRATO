//! The scenario definitions of `data/scenarios/`: the histories and the step plan that the
//! scenario engine runs, built from the typed scenario files and the registers, and a run of a
//! definition with the registered diffusivity laws (science design Section 5).
//!
//! Moved unchanged from `strato-results`, so that the results pipeline and the WebAssembly
//! wrapper build every definition with the same function.

use strato::numerical::physics_run::{Interpolation, TimeFunction};
use strato::numerical::scenario::{
    run_scenario, Diffusion, InitialState, ScenarioConfig, ScenarioDefinition, ScenarioRun,
    StepSegment,
};
use strato_data::scenarios::{unit_seconds, ScenarioFile, ScenarioRow, Scenarios};
use strato_data::select::Phase;
use strato_data::Register;

use crate::laws::{Laws, Nuclide};
/// The scenario ids and their slugs for keys.
pub const SLUGS: [(&str, &str); 5] = [
    ("S1", "steady-state"),
    ("S2", "shutdown-cooldown"),
    ("S3", "startup"),
    ("S4", "control-rod-insertion"),
    ("S5", "refuelling"),
];

/// A scenario with a variant, ready to run.
#[derive(Debug, Clone)]
pub struct Built {
    /// `S1` to `S5`.
    pub id: String,
    /// Variant id, empty for the base.
    pub variant: String,
    /// The definition, one continuous history from the fresh particle.
    pub def: ScenarioDefinition,
    /// Time of the event (shutdown, insertion, removal) in s, if the scenario has one.
    pub event_s: Option<f64>,
    /// Time at which the diffusivity set switches from C15 to C16, in s.
    pub t_switch_s: Option<f64>,
    /// The registered heating history on its own (time zero at its start, no generation, C16
    /// throughout) and the end of its last hold at the highest temperature, in s from its start;
    /// for the comparison with the kernel-only start (science design Section 5.6).
    pub heating_only: Option<(ScenarioDefinition, f64)>,
    /// The normal-operation temperature of the same variant, degC.
    pub t_normal_c: f64,
    /// `rate_density_reference` of the variant (`capsule:3` or `compact:6-2-1`).
    pub rate_target: String,
    /// Title of the scenario.
    pub description: String,
}

fn mode_of(rows: &[&ScenarioRow]) -> Result<Interpolation, String> {
    let first = rows
        .first()
        .ok_or("an empty history")?
        .interpolation
        .as_str();
    if rows.iter().any(|r| r.interpolation != first) {
        return Err(format!(
            "{}: a history mixes linear and step rows",
            rows[0].row_id
        ));
    }
    match first {
        "linear" => Ok(Interpolation::Linear),
        "step" => Ok(Interpolation::Step),
        other => Err(format!("interpolation `{other}`")),
    }
}

/// A history table from rows: times in s, values scaled by `scale`.
fn table_of(
    rows: &[&ScenarioRow],
    scale: f64,
) -> Result<(Vec<f64>, Vec<f64>, Interpolation), String> {
    let mode = mode_of(rows)?;
    let mut t = Vec::new();
    let mut v = Vec::new();
    for r in rows {
        t.push(r.t_seconds()?);
        v.push(r.value.as_ref().ok_or("no value")?.value * scale);
    }
    Ok((t, v, mode))
}

fn plan_of(rows: &[&ScenarioRow]) -> Result<Vec<StepSegment>, String> {
    rows.iter()
        .map(|r| {
            Ok(StepSegment {
                end_s: r.t_seconds()?,
                dt_max_s: r.value.as_ref().ok_or("no step")?.value * unit_seconds(&r.unit)?,
            })
        })
        .collect()
}

/// The rate density target of a variant.
pub fn rate_target(f: &ScenarioFile, variant: &str) -> Result<String, String> {
    f.meta(variant, "rate_density_reference")
        .map(|r| r.text.clone())
        .ok_or_else(|| format!("{}: no rate density reference", f.id))
}

/// Build the definition of a scenario and variant. `rate_value` is the reference fission-rate
/// density of the variant, used to turn a per-cycle rate into a generation factor.
///
/// # Errors
///
/// A file that does not say what the builder needs.
pub fn build(
    sc: &Scenarios,
    reg: &Register,
    id: &str,
    variant: &str,
    rate_value: f64,
) -> Result<Built, String> {
    let f = sc.get(id).ok_or_else(|| format!("no scenario {id}"))?;
    // The own histories. A variant that has no row of a quantity uses the base rows.
    let g_rows = f.history(variant, "G_kernel_factor");
    let t_rows = f.history(variant, "T_C");
    let s_rows = f.steps(variant);
    let g_scale = if g_rows
        .first()
        .is_some_and(|r| r.quantity == "fission_rate_density")
    {
        1e19 / rate_value
    } else {
        1.0
    };
    let (mut gt, mut gv, gm) = table_of(&g_rows, g_scale)?;
    let (mut tt, mut tv, tm) = table_of(&t_rows, 1.0)?;
    // The normal-operation temperature of the scenario: its value at the end of its own
    // history, before any heating history is appended (Scenario 3 starts cold and reaches it).
    let own_final_c = *tv.last().ok_or("an empty temperature history")?;
    let mut plan = plan_of(&s_rows)?;
    let mut events: Vec<f64> = f
        .rows
        .iter()
        .filter(|r| r.quantity == "event_time" && (r.variant.is_empty() || r.variant == variant))
        .map(|r| r.t_seconds())
        .collect::<Result<_, _>>()?;
    let mut t_switch = None;
    let mut heating_only: Option<(ScenarioDefinition, f64)> = None;

    // The registered heating history of Scenario 5, appended after the storage.
    if let Some(h) = f
        .rows
        .iter()
        .find(|r| r.quantity == "heating_history" && r.variant == variant && !variant.is_empty())
    {
        let hist = h.text.trim();
        let dt_row = f.meta(variant, "heating_dt_max").ok_or("no heating step")?;
        let dt_max =
            dt_row.value.as_ref().ok_or("no heating step")?.value * unit_seconds(&dt_row.unit)?;
        let start = gt.last().copied().ok_or("empty history")?;
        let mut clock = start;
        let mut rows: Vec<_> = reg
            .temperature
            .rows
            .iter()
            .filter(|r| r.history_id == hist)
            .collect();
        rows.sort_by_key(|r| r.step);
        if rows.is_empty() {
            return Err(format!("no registered history {hist}"));
        }
        t_switch = Some(start);
        // The start of the heating history is a stored row: the heating-phase fraction is
        // normalised to the inventory there (science design review finding m2).
        events.push(start);
        // The same history on its own, from time zero, for the kernel-only comparison.
        let mut h_times = vec![0.0];
        let mut h_temps = vec![*tv.last().ok_or("an empty temperature history")?];
        let mut h_plan: Vec<StepSegment> = Vec::new();
        for r in rows {
            let hours = r
                .ramp_h
                .as_ref()
                .or(r.hold_h.as_ref())
                .ok_or_else(|| format!("{}: neither a ramp nor a hold duration", r.id))?
                .value;
            let end_c = r
                .t_end
                .as_ref()
                .and_then(|p| p.celsius())
                .ok_or_else(|| format!("{}: no end temperature", r.id))?;
            clock += hours * 3600.0;
            tt.push(clock);
            tv.push(end_c);
            plan.push(StepSegment {
                end_s: clock,
                dt_max_s: dt_max,
            });
            h_times.push(clock - start);
            h_temps.push(end_c);
            h_plan.push(StepSegment {
                end_s: clock - start,
                dt_max_s: dt_max,
            });
        }
        gt.push(clock);
        gv.push(0.0);
        // The end of the last hold at the highest temperature of the history (the cool-down
        // follows it).
        let t_top = h_temps.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let hold_end = h_times
            .iter()
            .zip(&h_temps)
            .rev()
            .find(|(_, t)| **t == t_top)
            .map(|(s, _)| *s)
            .unwrap_or(0.0);
        heating_only = Some((
            ScenarioDefinition {
                id: format!("{id}-{variant}-heating-only"),
                generation_factor: TimeFunction::constant(0.0).map_err(|e| e.to_string())?,
                temperature_c: TimeFunction::table(h_times, h_temps, Interpolation::Linear)
                    .map_err(|e| e.to_string())?,
                plan: h_plan,
                event_times_s: vec![],
            },
            hold_end,
        ));
    }

    let mut def = ScenarioDefinition {
        id: if variant.is_empty() {
            id.to_string()
        } else {
            format!("{id}-{variant}")
        },
        generation_factor: TimeFunction::table(gt, gv, gm).map_err(|e| e.to_string())?,
        temperature_c: TimeFunction::table(tt, tv, tm).map_err(|e| e.to_string())?,
        plan,
        event_times_s: std::mem::take(&mut events),
    };
    let mut event_s = None;
    let mut t_normal_c = own_final_c;
    if let Some(lead) = f.rows.iter().find(|r| r.quantity == "lead_in") {
        let lead_id = lead.text.split(':').next().unwrap_or("").trim();
        let t_e = lead.t_seconds()?;
        let lead_built = build(sc, reg, lead_id, variant, rate_value)?;
        t_normal_c = lead_built.t_normal_c;
        def = def
            .with_lead_in(&lead_built.def, t_e)
            .map_err(|e| format!("{id}: {e}"))?;
        t_switch = t_switch.map(|s| s + t_e);
        event_s = Some(t_e);
    }
    Ok(Built {
        id: id.to_string(),
        variant: variant.to_string(),
        def,
        event_s,
        t_switch_s: t_switch,
        heating_only,
        t_normal_c,
        rate_target: rate_target(f, variant)?,
        description: f
            .meta("", "title")
            .map(|r| r.text.clone())
            .unwrap_or_default(),
    })
}

/// How the grid and the step of a run are set (numerical settings, refinement-checked).
#[derive(Debug, Clone, Copy)]
pub struct Numerics {
    /// Intervals per material.
    pub intervals: [usize; 5],
    /// Factor on every largest step of the plan (1 for the base run, 0.5 for the refined run).
    pub dt_scale: f64,
}

/// Kernel intervals so that `sqrt(D t_run) / dr >= 10` (science design 5.1), at least 40 and at
/// most 50 000.
pub fn kernel_intervals(kernel_radius_m: f64, d_kernel: f64, t_run_s: f64, quick: bool) -> usize {
    if quick {
        return 20;
    }
    let n = (10.0 * kernel_radius_m / (d_kernel * t_run_s).sqrt()).ceil();
    n.clamp(40.0, 50_000.0) as usize
}

/// Everything a run needs.
pub struct Setup<'a> {
    /// Registers.
    pub reg: &'a Register,
    /// Outer radii in m.
    pub radii_m: [f64; 5],
    /// The nuclide.
    pub nuclide: &'a Nuclide,
    /// Reference kernel generation rate, mol m^-3 s^-1 (0 for no generation).
    pub g_ref: f64,
    /// Decay constant in s^-1.
    pub lambda: f64,
    /// Grid and step.
    pub numerics: Numerics,
    /// Times at which a profile is stored.
    pub profile_times_s: Vec<f64>,
    /// The initial state of the run.
    pub initial: InitialState,
}

/// Run a definition.
///
/// # Errors
///
/// A missing law, an invalid history or a failed run.
pub fn run(def: &ScenarioDefinition, built: &Built, s: &Setup<'_>) -> Result<ScenarioRun, String> {
    let irr = Laws::new(s.reg, s.nuclide.element, Phase::Irradiation)?;
    let heat = match built.t_switch_s {
        Some(_) => Some(Laws::new(s.reg, s.nuclide.element, Phase::HeatingTest)?),
        None => None,
    };
    let switch = built.t_switch_s;
    let f = |t_s: f64, t_c: f64| match (&heat, switch) {
        (Some(h), Some(ts)) if t_s >= ts => h.at(t_c),
        _ => irr.at(t_c),
    };
    let mut d = def.clone();
    for seg in &mut d.plan {
        seg.dt_max_s *= s.numerics.dt_scale;
    }
    let cfg = ScenarioConfig {
        radii_m: s.radii_m,
        intervals: s.numerics.intervals,
        initial: s.initial.clone(),
        decay_constant_per_s: s.lambda,
        reference_generation_mol_m3_s: s.g_ref,
        diffusion: Diffusion::OfTemperature(&f),
        profile_times_s: s.profile_times_s.clone(),
        extra_boundaries_s: vec![],
    };
    run_scenario(&d, &cfg).map_err(|e| format!("{}: {e}", def.id))
}
