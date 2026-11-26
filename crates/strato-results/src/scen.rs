//! The scenarios: definitions from `data/scenarios/`, runs through the scenario engine, and the
//! result files (science design Section 5 and 7.3).

use strato::numerical::physics_run::{Interpolation, TimeFunction};
use strato::numerical::scenario::{
    post_event_fraction, run_scenario, Diffusion, InitialState, ScenarioConfig, ScenarioDefinition,
    ScenarioRun, StepSegment,
};
use strato_compare::json::J;
use strato_data::scenarios::{unit_seconds, Record, ScenarioFile, ScenarioRow, Scenarios};
use strato_data::select::Phase;
use strato_data::Register;

use crate::methodb::{applicability, Needs};
use crate::output::{json_file, Files, Prov};
use crate::params::{
    condition, generation_nuclides, rate_density, reference_generation, Laws, Nuclide, RateDensity,
};

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
fn rate_target(f: &ScenarioFile, variant: &str) -> Result<String, String> {
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

/// Linear interpolation of a series at `x` (clamped to the ends).
fn interp(t: &[f64], f: &[Option<f64>], x: f64) -> Option<f64> {
    let k = t.partition_point(|v| *v <= x);
    if k == 0 {
        return f[0];
    }
    if k >= t.len() {
        return f[t.len() - 1];
    }
    let (a, b) = (f[k - 1]?, f[k]?);
    Some(a + (x - t[k - 1]) / (t[k] - t[k - 1]) * (b - a))
}

/// Indices of at most `max` rows, always including the first, the last and every `must` row.
fn pick(n: usize, max: usize, must: &[usize]) -> Vec<usize> {
    let mut idx: Vec<usize> = if n <= max {
        (0..n).collect()
    } else {
        let stride = n.div_ceil(max);
        (0..n).step_by(stride).collect()
    };
    idx.push(n - 1);
    idx.extend(must.iter().copied().filter(|i| *i < n));
    idx.sort_unstable();
    idx.dedup();
    idx
}

fn opt_arr(v: impl Iterator<Item = Option<f64>>) -> J {
    J::Arr(v.map(J::opt).collect())
}

fn num_arr(v: impl Iterator<Item = f64>) -> J {
    J::Arr(v.map(J::Num).collect())
}

/// What one scenario run reports.
pub struct Outcome {
    /// The cell key.
    pub key: String,
    /// The JSON text of the curve file.
    pub json: String,
    /// Maximum `|eps| / bound` over the rows.
    pub max_residual_over_bound: f64,
    /// Largest `|eps| / (M^0 + M_gen)`.
    pub max_relative_residual: f64,
    /// Number of steps.
    pub steps: usize,
    /// Number of matrix assemblies.
    pub assemblies: usize,
    /// Number of distinct step lengths.
    pub distinct_steps: usize,
    /// Largest relative difference of the base and refined inventory at the base times.
    pub max_inventory_refinement_difference: f64,
    /// Fraction of drawable points of the plotted release fraction that are resolved.
    pub resolved_fraction: Option<f64>,
    /// CSV rows `[key, t_h, t_d, ...]` of the decimated series.
    pub csv_rows: Vec<Vec<String>>,
}

/// Run a scenario cell (base, refined, reference) and make its file.
///
/// # Errors
///
/// A failed run.
#[allow(clippy::too_many_arguments)]
pub fn cell(
    prov: &Prov,
    reg: &Register,
    sc: &Scenarios,
    id: &str,
    variant: &str,
    nuclide: &Nuclide,
    stage: u8,
    radii_m: [f64; 5],
    quick: bool,
    n_k: usize,
) -> Result<Outcome, String> {
    let target_text = rate_target(sc.get(id).ok_or("scenario")?, variant)?;
    let rate: RateDensity = rate_density(reg, &target_text)?;
    let built = build(sc, reg, id, variant, rate.value)?;
    let g_ref = reference_generation(nuclide.yield_per_fission, rate.value);
    let lambda = if stage == 2 {
        0.0
    } else {
        nuclide.lambda_per_s
    };
    // The kernel interval count `n_k` is the same for every run of the nuclide and the family of
    // variants (see `family_kernel_intervals`), so that every scenario reproduces the Scenario 1
    // state and the normal-operation reference on one grid.
    let (n_rest, dt_mult) = if quick {
        ([6, 6, 10, 6], 4.0)
    } else {
        ([20, 20, 160, 20], 1.0)
    };
    let intervals = [n_k, n_rest[0], n_rest[1], n_rest[2], n_rest[3]];
    let mut def = built.def.clone();
    for seg in &mut def.plan {
        seg.dt_max_s *= dt_mult;
    }
    let built_run = Built {
        def: def.clone(),
        ..built.clone()
    };
    let setup = |scale: f64, mult: usize| Setup {
        reg,
        radii_m,
        nuclide,
        g_ref,
        lambda,
        numerics: Numerics {
            intervals: intervals.map(|n| n * mult),
            dt_scale: scale,
        },
        profile_times_s: vec![],
        initial: InitialState::Fresh,
    };
    let base = run(&def, &built_run, &setup(1.0, 1))?;
    let fine = run(&def, &built_run, &setup(0.5, 2))?;
    let reference_def = def
        .normal_operation(1.0, built.t_normal_c)
        .map_err(|e| e.to_string())?;
    let reference = run(&reference_def, &built_run, &setup(1.0, 1))?;

    // The post-event fraction counts from the event (removal, shutdown); in a heating variant of
    // Scenario 5 it counts from the start of the heating history and is normalised to the
    // inventory there, so that the 8760 h of storage with decay does not enter the fraction.
    let t_e = built.t_switch_s.or(built.event_s);
    let n = base.rows.len();
    let times: Vec<f64> = base.rows.iter().map(|r| r.t_s).collect();
    let fine_t: Vec<f64> = fine.rows.iter().map(|r| r.t_s).collect();
    // F_gen (null until generation starts) and F_post, base and refined.
    let f_gen: Vec<Option<f64>> = base
        .rows
        .iter()
        .map(|r| r.release_fraction_of_total)
        .collect();
    let f_gen_fine: Vec<Option<f64>> = fine
        .rows
        .iter()
        .map(|r| r.release_fraction_of_total)
        .collect();
    let post = |run: &ScenarioRun| -> Vec<Option<f64>> {
        run.rows
            .iter()
            .map(|r| t_e.and_then(|e| post_event_fraction(run, e, r.t_s)))
            .collect()
    };
    let f_post = post(&base);
    let f_post_fine = post(&fine);
    let plotted_post = t_e.is_some() && matches!(id, "S2" | "S5");
    let (plot, plot_fine) = if plotted_post {
        (&f_post, &f_post_fine)
    } else {
        (&f_gen, &f_gen_fine)
    };
    let resolved: Vec<bool> = (0..n)
        .map(|k| match (plot[k], interp(&fine_t, plot_fine, times[k])) {
            (Some(a), Some(b)) => b > 0.0 && (a - b).abs() <= 0.05 * b,
            _ => false,
        })
        .collect();
    let drawable = plot.iter().filter(|v| v.is_some_and(|x| x > 0.0)).count();
    let resolved_fraction = (drawable > 0).then(|| {
        resolved
            .iter()
            .zip(plot)
            .filter(|(r, v)| **r && v.is_some_and(|x| x > 0.0))
            .count() as f64
            / drawable as f64
    });
    // Refinement difference of the inventory.
    let inv_fine: Vec<Option<f64>> = fine.rows.iter().map(|r| Some(r.inventory_mol)).collect();
    let mut max_inv: f64 = 0.0;
    for r in &base.rows {
        if let Some(b) = interp(&fine_t, &inv_fine, r.t_s) {
            if b > 0.0 {
                max_inv = max_inv.max((r.inventory_mol - b).abs() / b);
            }
        }
    }

    // Decimated rows, with every event and boundary row kept.
    let mut must = Vec::new();
    for e in [t_e, built.event_s].into_iter().flatten() {
        if let Some(k) = times
            .iter()
            .position(|t| (*t - e).abs() <= 1e-9 * e.max(1.0))
        {
            must.push(k);
        }
    }
    let sel = pick(n, if quick { 40 } else { 220 }, &must);
    let ref_times: Vec<f64> = reference.rows.iter().map(|r| r.t_s).collect();
    let ref_ok = ref_times.len() == times.len();
    let slug = SLUGS
        .iter()
        .find(|(i, _)| *i == id)
        .map_or("scenario", |s| s.1);
    let suffix = match variant {
        "" => String::new(),
        "exploration-1197C" => ".exploration".to_string(),
        v => format!(".{}", v.replace('-', "")),
    };
    let t_key = built.t_normal_c.round() as i64;
    let key = format!("s{stage}.{slug}{suffix}.{}.{t_key}c", nuclide.id);

    let needs = Needs {
        generation: g_ref > 0.0,
        decay: lambda > 0.0,
        temperature_varies: !built.def.temperature_c.is_constant(),
        kernel_step_start: true,
    };
    let b = match applicability(needs) {
        Ok(()) => J::obj(vec![("available", J::Bool(true))]),
        Err(n) => J::obj(vec![
            ("available", J::Bool(false)),
            ("reason_code", J::s(n.code)),
            ("reason", J::s(n.reason)),
        ]),
    };
    let h = |t: f64| t / 3600.0;
    let d = |t: f64| t / 86_400.0;
    let col = |f: &dyn Fn(usize) -> f64| num_arr(sel.iter().map(|k| f(*k)));
    let ocol = |f: &dyn Fn(usize) -> Option<f64>| opt_arr(sel.iter().map(|k| f(*k)));
    let rows = &base.rows;
    let ranges = |r: &ScenarioRun| -> J {
        J::Arr(
            r.ranges
                .iter()
                .zip(crate::params::LAYERS)
                .map(|(c, (name, _))| {
                    J::obj(vec![
                        ("layer", J::s(name)),
                        ("steps_inside_printed_range", J::Int(c.within as i64)),
                        ("steps_outside_printed_range", J::Int(c.outside as i64)),
                        ("steps_no_range_printed", J::Int(c.not_stated as i64)),
                    ])
                })
                .collect(),
        )
    };
    let c15 = c15_status(reg, &built_run, &def, &base)?;
    let outside = c15.outside_steps > 0;
    let panel_title = if outside {
        format!("{}; {}", built.description, c15.label)
    } else {
        built.description.clone()
    };
    let kernel_only = match (&built.heating_only, built.t_switch_s) {
        (Some((hdef, hold_end)), Some(ts)) => {
            let setup_h = |scale: f64, mult: usize| Setup {
                reg,
                radii_m,
                nuclide,
                g_ref: 0.0,
                lambda,
                numerics: Numerics {
                    intervals: intervals.map(|n| n * mult),
                    dt_scale: scale,
                },
                profile_times_s: vec![],
                initial: InitialState::KernelUniform { c_init: 1.0 },
            };
            let mut hd = hdef.clone();
            for seg in &mut hd.plan {
                seg.dt_max_s *= dt_mult;
            }
            let hb = Built {
                def: hd.clone(),
                t_switch_s: Some(0.0),
                heating_only: None,
                event_s: None,
                ..built.clone()
            };
            let kb = run(&hd, &hb, &setup_h(1.0, 1))?;
            let kf = run(&hd, &hb, &setup_h(0.5, 2))?;
            kernel_only_json(&KernelOnly {
                t_switch_s: ts,
                hold_end_s: *hold_end,
                kb: &kb,
                kf: &kf,
                base: &base,
                fine_t: &fine_t,
                f_post_fine: &f_post_fine,
                quick,
            })
        }
        _ => J::Null,
    };
    let labels: Vec<J> = {
        let mut l = vec![J::s(
            "prescribed-input simulation; not a validated reactor prediction; numerical solution of the stated model; not experimental validation",
        )];
        if outside {
            l.push(J::s(c15.label.clone()));
        }
        l.push(J::s(
            "U-235 thermal cumulative yield; the Pu-239 share of fission is not represented",
        ));
        l.push(J::s(rate.label.clone()));
        l
    };
    let mut csv_rows = Vec::new();
    for k in &sel {
        let r = &rows[*k];
        csv_rows.push(vec![
            key.clone(),
            format!("{:e}", h(r.t_s)),
            format!("{:e}", d(r.t_s)),
            plot[*k].map_or(String::new(), |x| format!("{x:e}")),
            resolved[*k].to_string(),
            format!("{:e}", r.inventory_mol),
            format!("{:e}", r.generated_mol),
            format!("{:e}", r.released_mol),
            format!("{:e}", r.decayed_mol),
            format!("{:e}", r.temperature_c),
            format!("{:e}", r.generation_factor),
            format!("{:e}", r.balance_residual_mol),
            format!("{:e}", r.balance_bound_mol),
        ]);
    }
    let reference_json = if ref_ok {
        let rr = &reference.rows;
        let ref_f: Vec<Option<f64>> = rr.iter().map(|r| r.release_fraction_of_total).collect();
        let ref_post: Vec<Option<f64>> = rr
            .iter()
            .map(|r| t_e.and_then(|e| post_event_fraction(&reference, e, r.t_s)))
            .collect();
        J::obj(vec![
            (
                "label",
                J::s("normal operation: generation factor 1 and the normal-operation temperature for the whole run (for S2 and S5 beyond 620.2 EFPD this is prescribed input (assumed): a reference, not a registered irradiation)"),
            ),
            ("grey_line_colour", J::s("#6B7280")),
            ("t_s", num_arr(sel.iter().map(|k| rr[*k].t_s))),
            ("inventory_mol", num_arr(sel.iter().map(|k| rr[*k].inventory_mol))),
            ("generated_mol", num_arr(sel.iter().map(|k| rr[*k].generated_mol))),
            ("released_mol", num_arr(sel.iter().map(|k| rr[*k].released_mol))),
            ("decayed_mol", num_arr(sel.iter().map(|k| rr[*k].decayed_mol))),
            (
                if plotted_post { "post_event_fraction" } else { "release_fraction_of_generated" },
                opt_arr(sel.iter().map(|k| if plotted_post { ref_post[*k] } else { ref_f[*k] })),
            ),
        ])
    } else {
        J::Null
    };
    let (axis_x, axis_y) = match id {
        "S2" | "S5" => (
            "Time after the event (h)",
            "Fraction of the inventory at the event released (-)",
        ),
        _ => ("Time (d)", "Fraction of generated inventory released (-)"),
    };
    let json = json_file(
        prov,
        "scenario curve",
        vec![
            ("key", J::s(key.clone())),
            (
                "selection",
                J::obj(vec![
                    ("stage", J::Int(stage as i64)),
                    ("context", J::s(id)),
                    ("scenario", J::s(slug)),
                    ("variant", J::s(variant)),
                    ("nuclide", J::s(nuclide.name)),
                    ("normal_operation_temperature_c", J::Num(built.t_normal_c)),
                    ("event_time_s", J::opt(built.event_s)),
                    ("post_event_fraction_reference_time_s", J::opt(t_e)),
                    ("heating_switch_time_s", J::opt(built.t_switch_s)),
                ]),
            ),
            ("panel_title", J::s(panel_title)),
            ("labels", J::Arr(labels)),
            (
                "config",
                J::obj(vec![
                    ("nuclide", J::s(nuclide.name)),
                    ("decay_constant_per_s", J::Num(lambda)),
                    ("cumulative_yield_per_fission", J::Num(nuclide.yield_per_fission)),
                    ("fission_rate_density_per_m3_s", J::Num(rate.value)),
                    ("fission_rate_density_target", J::s(rate.target.clone())),
                    ("fission_rate_density_label", J::s(rate.label.clone())),
                    ("fission_rate_density_rows", J::Arr(rate.source_ids.iter().map(J::s).collect())),
                    ("reference_generation_mol_m3_s", J::Num(g_ref)),
                    ("initial_state", J::s("fresh particle: C = 0, M^0 = 0")),
                    ("outer_radii_m", num_arr(radii_m.iter().copied())),
                ]),
            ),
            (
                "numerics",
                J::obj(vec![
                    ("intervals_per_layer", J::Arr(intervals.iter().map(|n| J::Int(*n as i64)).collect())),
                    ("refined_intervals_per_layer", J::Arr(intervals.iter().map(|n| J::Int(2 * *n as i64)).collect())),
                    ("steps", J::Int(base.steps as i64)),
                    ("refined_steps", J::Int(fine.steps as i64)),
                    ("distinct_step_lengths", J::Int(base.distinct_steps as i64)),
                    ("matrix_assemblies", J::Int(base.assemblies as i64)),
                    ("max_inventory_refinement_difference", J::Num(max_inv)),
                    ("resolved_fraction_of_drawable_points", J::opt(resolved_fraction)),
                    ("resolution_rule", J::s("numerical setting (chosen; refinement-checked per Section 3.5): a point is resolved if the base and the refined run (twice the intervals per layer, half the step) differ by at most 5 % of the refined value; the 5 % is a labelled convention, not a derived bound")),
                    ("max_abs_balance_residual_over_bound", J::Num(base.max_residual_over_bound)),
                    ("max_relative_balance_residual", J::Num(base.max_relative_residual)),
                    ("balance_bound", J::s("(F46), Document 5 Section 5.8")),
                    ("diffusivity_ranges", ranges(&base)),
                ("c15_documented_application", c15.json.clone()),
                    ("min_matrix_row_margin", J::Num(base.matrix_min_row_margin)),
                ]),
            ),
            (
                "axes",
                J::obj(vec![
                    ("x", J::obj(vec![("quantity", J::s(axis_x)), ("unit", J::s(if id == "S2" || id == "S5" { "h" } else { "d" }))])),
                    ("y", J::obj(vec![("quantity", J::s(axis_y)), ("unit", J::s("dimensionless"))])),
                    ("default_y_scale", J::s("log")),
                ]),
            ),
            (
                "method_a",
                J::obj(vec![
                    ("label", J::s("Method A: BTCS")),
                    ("t_s", col(&|k| rows[k].t_s)),
                    ("time_h", col(&|k| h(rows[k].t_s))),
                    ("time_d", col(&|k| d(rows[k].t_s))),
                    (
                        "plotted_quantity",
                        J::s(if plotted_post {
                            "post-event release fraction F_post (Section 5.1)"
                        } else {
                            "release fraction of the generated inventory F_gen (Section 5.1)"
                        }),
                    ),
                    ("release_fraction", ocol(&|k| plot[k])),
                    ("resolved", J::Arr(sel.iter().map(|k| J::Bool(resolved[*k])).collect())),
                    ("release_fraction_of_generated", ocol(&|k| f_gen[k])),
                    ("post_event_fraction", ocol(&|k| f_post[k])),
                    ("release_to_generation_ratio", ocol(&|k| rows[k].release_to_generation_ratio)),
                ]),
            ),
            (
                "method_b",
                b,
            ),
            (
                "quantities",
                J::obj(vec![
                    ("inventory_mol", col(&|k| rows[k].inventory_mol)),
                    ("generated_mol", col(&|k| rows[k].generated_mol)),
                    ("released_mol", col(&|k| rows[k].released_mol)),
                    ("decayed_mol", col(&|k| rows[k].decayed_mol)),
                    ("balance_residual_mol", col(&|k| rows[k].balance_residual_mol)),
                    ("balance_bound_mol", col(&|k| rows[k].balance_bound_mol)),
                    ("temperature_c", col(&|k| rows[k].temperature_c)),
                    ("generation_factor", col(&|k| rows[k].generation_factor)),
                    ("kernel_source_mol_m3_s", col(&|k| rows[k].kernel_source_mol_m3_s)),
                ]),
            ),
            ("reference_normal_operation", reference_json),
            ("kernel_only_comparison", kernel_only),
        ],
    );
    Ok(Outcome {
        key,
        json,
        max_residual_over_bound: base.max_residual_over_bound,
        max_relative_residual: base.max_relative_residual,
        steps: base.steps,
        assemblies: base.assemblies,
        distinct_steps: base.distinct_steps,
        max_inventory_refinement_difference: max_inv,
        resolved_fraction,
        csv_rows,
    })
}

/// Where the C15 correlations are evaluated against their documented application.
struct C15Status {
    /// The label text (bounds read from the registers).
    label: String,
    /// Steps outside the documented application.
    outside_steps: usize,
    /// The per-segment status for the curve file.
    json: J,
}

/// The documented application of the C15 correlations, 955 to 1197 degC (science design 2.2 and
/// 5.1): the lowest time-average volume-average temperature (IRR-361) and the highest time-average
/// peak temperature (IRR-097) of the as-run AGR-1 thermal analysis, read from the register. For
/// each segment of the step plan it counts the steps in which a C15 law is evaluated at the
/// temperature at the middle of the step below, inside or above that interval; steps after the
/// switch to the C16 set are counted apart.
fn c15_status(
    reg: &Register,
    built: &Built,
    def: &ScenarioDefinition,
    run: &ScenarioRun,
) -> Result<C15Status, String> {
    let lo = condition(reg, "IRR-361")?;
    let hi = condition(reg, "IRR-097")?;
    let label =
        format!("C15 evaluated outside its documented application ({lo:.0} to {hi:.0} \u{b0}C)");
    #[derive(Default, Clone, Copy)]
    struct Seg {
        below: usize,
        inside: usize,
        above: usize,
        c16: usize,
    }
    let mut counts = vec![Seg::default(); def.plan.len()];
    for w in run.rows.windows(2) {
        let t_mid = 0.5 * (w[0].t_s + w[1].t_s);
        let k = def
            .plan
            .iter()
            .position(|s| t_mid < s.end_s)
            .unwrap_or(def.plan.len() - 1);
        if built.t_switch_s.is_some_and(|ts| t_mid >= ts) {
            counts[k].c16 += 1;
            continue;
        }
        let t = def.temperature_c.value_at(t_mid);
        if t < lo {
            counts[k].below += 1;
        } else if t > hi {
            counts[k].above += 1;
        } else {
            counts[k].inside += 1;
        }
    }
    let mut start = 0.0;
    let mut segs = Vec::new();
    let mut outside_steps = 0;
    for (s, c) in def.plan.iter().zip(&counts) {
        let c15 = c.below + c.inside + c.above;
        outside_steps += c.below + c.above;
        let status = if c15 == 0 {
            "C16 heating-test correlations (the documented application of C15 does not apply)"
        } else if c.below + c.above == 0 {
            "inside the documented application of C15"
        } else if c.inside == 0 {
            "outside the documented application of C15"
        } else {
            "partly outside the documented application of C15"
        };
        segs.push(J::obj(vec![
            ("start_s", J::Num(start)),
            ("end_s", J::Num(s.end_s)),
            ("steps_c15_below", J::Int(c.below as i64)),
            ("steps_c15_inside", J::Int(c.inside as i64)),
            ("steps_c15_above", J::Int(c.above as i64)),
            ("steps_c16", J::Int(c.c16 as i64)),
            ("status", J::s(status)),
        ]));
        start = s.end_s;
    }
    let json = J::obj(vec![
        (
            "documented_application_c",
            J::Arr(vec![J::Num(lo), J::Num(hi)]),
        ),
        (
            "bound_rows",
            J::Arr(vec![J::s("IRR-361"), J::s("IRR-097")]),
        ),
        (
            "rule",
            J::s("C15 rows print no range; their basis is their application over the AGR-1 irradiation. A step counts as outside when the temperature at its middle lies below or above this interval; the law is then evaluated through diffusivity_extrapolated."),
        ),
        ("label_when_outside", J::s(label.clone())),
        ("steps_outside", J::Int(outside_steps as i64)),
        ("segments", J::Arr(segs)),
    ]);
    Ok(C15Status {
        label,
        outside_steps,
        json,
    })
}

/// The inputs of the kernel-only comparison of a Scenario 5 heating cell.
struct KernelOnly<'a> {
    /// Start of the heating history in the Scenario 5 run, s.
    t_switch_s: f64,
    /// End of the last hold at the highest temperature, s from the start of the history.
    hold_end_s: f64,
    /// Kernel-only run of the history, base and refined.
    kb: &'a ScenarioRun,
    kf: &'a ScenarioRun,
    /// The Scenario 5 base run.
    base: &'a ScenarioRun,
    /// Times of the refined Scenario 5 run and its `F_post` series (reference time: the heating
    /// start).
    fine_t: &'a [f64],
    f_post_fine: &'a [Option<f64>],
    quick: bool,
}

/// `F_post` of the Scenario 5 heating run beside the run of the same registered history from a
/// kernel-only start (science design Section 5.6). Both fractions are the amount released since
/// the start of the history over the inventory at that start; the kernel-only run starts from
/// `C = C_init` in the kernel (decision 20), the Scenario 5 run from the state the irradiation
/// and the storage leave.
fn kernel_only_json(k: &KernelOnly<'_>) -> J {
    let n = k.kb.rows.len();
    let rel: Vec<f64> = k.kb.rows.iter().map(|r| r.t_s).collect();
    let ko: Vec<Option<f64>> =
        k.kb.rows
            .iter()
            .map(|r| r.release_fraction_of_initial)
            .collect();
    let ko_fine_t: Vec<f64> = k.kf.rows.iter().map(|r| r.t_s).collect();
    let ko_fine: Vec<Option<f64>> =
        k.kf.rows
            .iter()
            .map(|r| r.release_fraction_of_initial)
            .collect();
    let s5: Vec<Option<f64>> = rel
        .iter()
        .map(|t| post_event_fraction(k.base, k.t_switch_s, k.t_switch_s + t))
        .collect();
    let close = |a: Option<f64>, b: Option<f64>| match (a, b) {
        (Some(a), Some(b)) => b > 0.0 && (a - b).abs() <= 0.05 * b,
        _ => false,
    };
    let s5_res: Vec<bool> = (0..n)
        .map(|i| {
            close(
                s5[i],
                interp(k.fine_t, k.f_post_fine, k.t_switch_s + rel[i]),
            )
        })
        .collect();
    let ko_res: Vec<bool> = (0..n)
        .map(|i| close(ko[i], interp(&ko_fine_t, &ko_fine, rel[i])))
        .collect();
    let ratio: Vec<Option<f64>> = (0..n)
        .map(|i| match (s5[i], ko[i]) {
            (Some(a), Some(b)) if b > 0.0 => Some(a / b),
            _ => None,
        })
        .collect();
    let idx_of = |t: f64| rel.iter().position(|x| (x - t).abs() <= 1e-9 * t.max(1.0));
    let hold_idx = idx_of(k.hold_end_s);
    let must: Vec<usize> = hold_idx.into_iter().collect();
    let sel = pick(n, if k.quick { 40 } else { 220 }, &must);
    let point = |i: Option<usize>| -> J {
        match i {
            Some(i) => J::obj(vec![
                ("t_h", J::Num(rel[i] / 3600.0)),
                ("F_post_scenario5", J::opt(s5[i])),
                ("F_post_kernel_only", J::opt(ko[i])),
                ("ratio_scenario5_over_kernel_only", J::opt(ratio[i])),
                ("resolved_scenario5", J::Bool(s5_res[i])),
                ("resolved_kernel_only", J::Bool(ko_res[i])),
            ]),
            None => J::Null,
        }
    };
    J::obj(vec![
        (
            "label",
            J::s("kernel-only start (decision 20: C = C_init in the kernel, G = 0, C16 throughout) of the same registered heating history, beside the Scenario 5 heating run; numerical comparison of two model starts, not an experimental comparison"),
        ),
        (
            "normalisation",
            J::s("both series are the amount released since the start of the heating history divided by the inventory present at that start"),
        ),
        ("t_h", num_arr(sel.iter().map(|i| rel[*i] / 3600.0))),
        ("F_post_scenario5", opt_arr(sel.iter().map(|i| s5[*i]))),
        ("F_post_kernel_only", opt_arr(sel.iter().map(|i| ko[*i]))),
        (
            "ratio_scenario5_over_kernel_only",
            opt_arr(sel.iter().map(|i| ratio[*i])),
        ),
        (
            "resolved_scenario5",
            J::Arr(sel.iter().map(|i| J::Bool(s5_res[*i])).collect()),
        ),
        (
            "resolved_kernel_only",
            J::Arr(sel.iter().map(|i| J::Bool(ko_res[*i])).collect()),
        ),
        ("end_of_last_hold_at_highest_temperature", point(hold_idx)),
        ("end_of_history", point(n.checked_sub(1))),
        (
            "resolution_rule",
            J::s("a value is resolved if the base and the refined run (twice the intervals per layer, half the step) differ by at most 5 % of the refined value (labelled convention)"),
        ),
    ])
}

/// The number of kernel intervals shared by every run of one nuclide and one family of variants:
/// the exploration family at 1197 degC, and all other variants (the base scenarios, the cycle
/// variant, the heating variants), which share the Scenario 1 irradiation. It is chosen once from
/// the longest run of the family so that `sqrt(D t_run) / dr >= 10` holds for all of them (science
/// design 5.1), and then every scenario reproduces the Scenario 1 state at the end of its lead-in
/// and the reference of normal operation on the same grid.
///
/// # Errors
///
/// A scenario that cannot be built, or a family whose runs do not share one normal-operation
/// temperature.
pub fn family_kernel_intervals(
    reg: &Register,
    sc: &Scenarios,
    nuclide: &Nuclide,
    exploration: bool,
    runs: &[(&str, String)],
    radii_m: [f64; 5],
    quick: bool,
) -> Result<usize, String> {
    let mut longest = 0.0_f64;
    let mut t_normal: Option<f64> = None;
    for (id, v) in runs
        .iter()
        .filter(|(_, v)| (v == "exploration-1197C") == exploration)
    {
        let target = rate_target(sc.get(id).ok_or("scenario")?, v)?;
        let rate = rate_density(reg, &target)?;
        let built = build(sc, reg, id, v, rate.value)?;
        longest = longest.max(built.def.end_s());
        match t_normal {
            None => t_normal = Some(built.t_normal_c),
            Some(t) if t == built.t_normal_c => {}
            Some(t) => {
                return Err(format!(
                    "{id} {v}: normal-operation temperature {} differs from {t} in its family",
                    built.t_normal_c
                ))
            }
        }
    }
    let t = t_normal.ok_or("an empty family")?;
    let irr = Laws::new(reg, nuclide.element, Phase::Irradiation)?;
    let d_k = irr.at(t).map_err(|e| e.to_string())?.0[0];
    Ok(kernel_intervals(radii_m[0], d_k, longest, quick))
}

/// The `scenarios.json` file: the definitions with the source of every number, and the step
/// sequences that follow from the plan.
///
/// # Errors
///
/// A definition that cannot be built.
pub fn definitions_json(prov: &Prov, sc: &Scenarios, reg: &Register) -> Result<String, String> {
    let mut list = Vec::new();
    for f in &sc.files {
        let mut variants = vec![String::new()];
        variants.extend(f.variants());
        let mut vs = Vec::new();
        for v in variants {
            let target = rate_target(f, &v)?;
            let rate = rate_density(reg, &target)?;
            let built = build(sc, reg, &f.id, &v, rate.value)?;
            let steps = built.def.steps(&[]).map_err(|e| e.to_string())?;
            // Piecewise-uniform segments: (start, end, number of steps, length).
            let bounds = built.def.boundaries(&[]).map_err(|e| e.to_string())?;
            let mut segs = Vec::new();
            for w in bounds.windows(2) {
                let n = steps
                    .iter()
                    .filter(|s| s.0 >= w[0] - 1e-9 && s.1 <= w[1] + 1e-9)
                    .count();
                if n > 0 {
                    segs.push(J::obj(vec![
                        ("start_s", J::Num(w[0])),
                        ("end_s", J::Num(w[1])),
                        ("steps", J::Int(n as i64)),
                        ("step_s", J::Num((w[1] - w[0]) / n as f64)),
                    ]));
                }
            }
            let hist = |q: &str| -> J {
                J::Arr(
                    f.history(&v, q)
                        .iter()
                        .map(|r| {
                            J::obj(vec![
                                ("row_id", J::s(r.row_id.clone())),
                                ("t_value", J::opt(r.t_value.as_ref().map(|t| t.value))),
                                ("t_unit", J::s(r.t_unit.clone())),
                                ("t_s", J::opt(r.t_seconds().ok())),
                                ("value", J::opt(r.value.as_ref().map(|t| t.value))),
                                ("unit", J::s(r.unit.clone())),
                                ("interpolation", J::s(r.interpolation.clone())),
                                (
                                    "source_ids",
                                    J::Arr(r.source_ids.iter().map(J::s).collect()),
                                ),
                                ("basis", J::s(r.basis.clone())),
                                ("reason", J::s(r.text.clone())),
                            ])
                        })
                        .collect(),
                )
            };
            let meta: Vec<J> = f
                .rows
                .iter()
                .filter(|r| r.record == Record::Meta && (r.variant.is_empty() || r.variant == v))
                .map(|r| {
                    J::obj(vec![
                        ("row_id", J::s(r.row_id.clone())),
                        ("key", J::s(r.quantity.clone())),
                        ("text", J::s(r.text.clone())),
                        ("t_value", J::opt(r.t_value.as_ref().map(|t| t.value))),
                        ("t_unit", J::s(r.t_unit.clone())),
                        ("basis", J::s(r.basis.clone())),
                        (
                            "source_ids",
                            J::Arr(r.source_ids.iter().map(J::s).collect()),
                        ),
                    ])
                })
                .collect();
            let g_key = if f
                .history(&v, "G_kernel_factor")
                .first()
                .is_some_and(|r| r.quantity == "fission_rate_density")
            {
                "fission_rate_density"
            } else {
                "G_kernel_factor"
            };
            vs.push(J::obj(vec![
                ("variant", J::s(v.clone())),
                ("rate_density_target", J::s(target)),
                ("rate_density_per_m3_s", J::Num(rate.value)),
                ("rate_density_label", J::s(rate.label.clone())),
                ("normal_operation_temperature_c", J::Num(built.t_normal_c)),
                ("event_time_s", J::opt(built.event_s)),
                ("end_time_s", J::Num(built.def.end_s())),
                ("total_steps", J::Int(steps.len() as i64)),
                ("step_sequence", J::Arr(segs)),
                (g_key, hist(g_key)),
                ("T_C", hist("T_C")),
                ("metadata", J::Arr(meta)),
            ]));
        }
        list.push(J::obj(vec![
            ("id", J::s(f.id.clone())),
            ("file", J::s(f.file)),
            ("variants", J::Arr(vs)),
        ]));
    }
    let nuclides = generation_nuclides(reg)?;
    Ok(json_file(
        prov,
        "scenario definitions (prescribed inputs)",
        vec![
            (
                "statement",
                J::s("Every scenario is a prescribed-input simulation. Each number carries its source row id or the label prescribed input (assumed). No scenario is a validated reactor prediction. STRATO does not model neutron kinetics, control-rod motion, thermal-hydraulics, power redistribution or mechanical refuelling."),
            ),
            (
                "nuclides_meaningful",
                J::Arr(nuclides.iter().map(|n| J::s(n.name)).collect()),
            ),
            (
                "nuclides_not_meaningful",
                J::s("Ag-110m, Cs-134 and Eu-154 get no direct fission-yield source term (decision 15)"),
            ),
            ("scenarios", J::Arr(list)),
        ],
    ))
}

/// Add the scenario files to the file set; returns the outcomes for the verification files.
///
/// # Errors
///
/// A failed run.
pub fn all(
    prov: &Prov,
    reg: &Register,
    sc: &Scenarios,
    radii_m: [f64; 5],
    quick: bool,
    files: &mut Files,
) -> Result<Vec<Outcome>, String> {
    files.insert("scenarios.json".into(), definitions_json(prov, sc, reg)?);
    let nuclides = generation_nuclides(reg)?;
    let mut out = Vec::new();
    // (scenario, variant, stages)
    let mut plan: Vec<(&str, String, Vec<u8>)> = vec![("S1", String::new(), vec![2, 3, 5])];
    plan.push(("S1", "cycles".into(), vec![5]));
    for id in ["S2", "S3", "S4", "S5"] {
        plan.push((id, String::new(), vec![5]));
    }
    for v in ["heating-1600", "heating-1700", "heating-1800"] {
        plan.push(("S5", v.into(), vec![5]));
    }
    for id in ["S1", "S2", "S3", "S4", "S5"] {
        plan.push((id, "exploration-1197C".into(), vec![5]));
    }
    let runs: Vec<(&str, String)> = plan.iter().map(|(i, v, _)| (*i, v.clone())).collect();
    for n in &nuclides {
        let n_base = family_kernel_intervals(reg, sc, n, false, &runs, radii_m, quick)?;
        let n_expl = family_kernel_intervals(reg, sc, n, true, &runs, radii_m, quick)?;
        for (id, v, stages) in &plan {
            let n_k = if v == "exploration-1197C" {
                n_expl
            } else {
                n_base
            };
            for st in stages {
                let o = cell(prov, reg, sc, id, v, n, *st, radii_m, quick, n_k)?;
                files.insert(format!("curves/{}.json", o.key), o.json.clone());
                out.push(o);
            }
        }
    }
    Ok(out)
}
