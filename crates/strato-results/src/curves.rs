//! The curves of the experimental-comparison context (EXP) and of model exploration (EXPL):
//! Method A (BTCS) and Method B (Sturm-Liouville) release fractions, per equation stage.
//!
//! Stage 1 is the baseline (kernel-only start, constant diffusivity); stage 3 adds decay; stage 4
//! runs the registered heating-test history with the diffusivity of each layer at the
//! temperature of each step. Stages 2 and 5 need generation, which a post-irradiation heating
//! test does not have (G = 0); they are not defined in these contexts and the support file says
//! so. Stage 1 uses the same machinery as the comparison program (`strato-compare`), so a value
//! here equals the value of the comparison.

use strato::numerical::physics_run::{Interpolation, TimeFunction};
use strato::numerical::scenario::{
    run_scenario, Diffusion, InitialState, ScenarioConfig, ScenarioDefinition, ScenarioRun,
    StepSegment,
};
use strato_compare::data::{Evaluated, Radii, Species};
use strato_compare::json::J;
use strato_compare::model::{curve as model_curve, Curve, Runner, Settings};
use strato_data::select::Phase;
use strato_data::Register;

use crate::methodb::{method_b, temperature_of, MethodB, Needs};
use crate::output::{json_file, Prov};
use crate::params::{decay_constant, Laws};
use strato::analytical::applicability::{SourceHistory, Stage, TemperatureHistory};
use strato::analytical::extensions::Quantity;

/// A nuclide of the experimental comparison.
#[derive(Debug, Clone, Copy)]
pub struct Nuc {
    /// Short id used in keys.
    pub id: &'static str,
    /// Name in the registers.
    pub name: &'static str,
    /// Full name for labels.
    pub label: &'static str,
    /// Element of the diffusivity rows.
    pub species: Species,
}

/// Ag-110m, Cs-134 (caesium rows) and Sr-90.
pub const NUCS: [Nuc; 3] = [
    Nuc {
        id: "ag110m",
        name: "Ag-110m",
        label: "Silver-110m",
        species: Species::Ag,
    },
    Nuc {
        id: "cs134",
        name: "Cs-134",
        label: "Caesium-134 (caesium diffusivity rows)",
        species: Species::Cs,
    },
    Nuc {
        id: "sr90",
        name: "Sr-90",
        label: "Strontium-90",
        species: Species::Sr,
    },
];

// The temperatures of the main experimental comparison and of model exploration are shared with
// the WebAssembly wrapper and live in `strato-scenarios`.
pub use strato_scenarios::support::{exp_temperatures, EXPL_HEATING_C, EXPL_IRRADIATION_C};

/// Seconds per hour.
const H: f64 = 3600.0;
/// The end of every heating-test curve: the longest registered as-run hold (TH-032), hours.
pub const HOLD_END_H: f64 = 400.0;

/// Linear interpolation of a series at `x`; `None` beyond the series.
pub fn lin(t: &[f64], f: &[f64], x: f64) -> Option<f64> {
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
    let w = (x - t[k - 1]) / (t[k] - t[k - 1]);
    Some(f[k - 1] + w * (f[k] - f[k - 1]))
}

/// The sample times of a curve file: geometric from `first` to `end` (ratio 1.08), the origin
/// and every extra time (measurement times), ascending.
pub fn sample_times(first: f64, end: f64, extra: &[f64]) -> Vec<f64> {
    let mut t = vec![0.0];
    let mut x = first;
    while x < end {
        t.push(x);
        x *= 1.08;
    }
    t.push(end);
    t.extend(extra.iter().copied().filter(|e| *e > 0.0 && *e <= end));
    t.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    t.dedup_by(|a, b| (*a - *b).abs() <= 1e-9 * a.abs().max(1.0));
    t
}

/// The JSON of the Method B part of a curve file, the drawn values, the accuracy floor and the
/// first valid time in hours. A value the drawing mask withholds is `null` with its reason code at
/// the same index of `withheld_reason_codes`.
pub fn b_series_json(
    b: &MethodB,
    t_h: &[f64],
    quantity: &str,
) -> (Vec<Option<f64>>, Option<f64>, Option<f64>, J) {
    match b {
        MethodB::Available(a) => (
            a.values.clone(),
            Some(a.accuracy_floor),
            Some(a.valid_from_s / H),
            J::obj(vec![
                ("available", J::Bool(true)),
                ("label", J::s("Method B: Sturm\u{2013}Liouville")),
                (
                    "extension",
                    a.extension.map_or(J::Null, |e| J::s(e.formulas())),
                ),
                ("plotted_quantity", J::s(quantity)),
                ("time_h", J::Arr(t_h.iter().map(|t| J::Num(*t)).collect())),
                ("release_fraction", J::Arr(a.values.iter().map(|v| J::opt(*v)).collect())),
                (
                    "withheld_reason_codes",
                    J::Arr(a.withheld.iter().map(|c| c.map_or(J::Null, J::s)).collect()),
                ),
                (
                    "withheld_reason_legend",
                    J::obj(
                        crate::methodb::WITHHELD_LEGEND
                            .iter()
                            .map(|(c, t)| (*c, J::s(*t)))
                            .collect(),
                    ),
                ),
                ("valid_from_h", J::Num(a.valid_from_s / H)),
                ("accuracy_floor", J::Num(a.accuracy_floor)),
                ("modes", J::Int(a.modes as i64)),
                ("alpha_max_per_s", J::Num(a.alpha_max_per_s)),
                ("min_valid_time_s", J::Num(a.valid_from_s)),
                ("truncation_missing_weight", J::Num(a.missing_weight)),
                (
                    "mask",
                    J::s("drawn only where t >= min_valid_time, the series with the eigenvalue cover reduced by 4 differs by at most 5 % and the estimated error is at most 5 % of the value (science design Section 4.2; Document 6 Section 8 for decay and a source, inside its measured regime; the 5 % is a labelled convention)"),
                ),
            ]),
        ),
        MethodB::NotAvailable(n) => (
            vec![None; t_h.len()],
            None,
            None,
            J::obj(vec![
                ("available", J::Bool(false)),
                ("reason_code", J::s(n.code)),
                ("reason", J::s(n.reason.clone())),
            ]),
        ),
    }
}

/// A stage-1 curve with its Method A and Method B values at the sample times.
#[derive(Debug, Clone)]
pub struct Stage1 {
    /// Cell key.
    pub key: String,
    /// Sample times, hours.
    pub t_h: Vec<f64>,
    /// Method A at the sample times.
    pub f_a: Vec<f64>,
    /// Refined run at the sample times.
    pub f_refined: Vec<f64>,
    /// Resolved flags (base and refined within 5 % of the refined value).
    pub resolved: Vec<bool>,
    /// Method B at the sample times where the drawing mask accepts it.
    pub f_b: Vec<Option<f64>>,
    /// Method B accuracy floor and first valid time (hours), when available.
    pub b_floor: Option<f64>,
    /// First valid time of Method B, hours.
    pub b_valid_from_h: Option<f64>,
    /// CSV rows of the curve.
    pub csv_rows: Vec<Vec<String>>,
}

/// Which context a curve belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    /// The main experimental comparison.
    Exp,
    /// Model exploration.
    Expl,
}

impl Context {
    fn slug(self) -> &'static str {
        match self {
            Context::Exp => "exp",
            Context::Expl => "expl",
        }
    }
}

fn resolved(a: f64, refined: f64) -> bool {
    refined > 0.0 && (a - refined).abs() <= 0.05 * refined
}

fn csv_cell(x: f64) -> String {
    format!("{x:e}")
}

/// Stage 1 (the baseline) for one cell: Method A from the comparison machinery, Method B through
/// [`method_b`]. Returns the file text and the numbers other files need.
///
/// # Errors
///
/// A failed run.
#[allow(clippy::too_many_arguments)]
pub fn stage1(
    prov: &Prov,
    ctx: Context,
    nuc: &Nuc,
    t_c: f64,
    curve: &Curve,
    radii: &Radii,
    phase: Phase,
    extra_times_h: &[f64],
    labels: Vec<String>,
) -> Result<(String, Stage1), String> {
    let end = curve.end_h();
    let first = if phase == Phase::Irradiation {
        5.0
    } else {
        0.05
    };
    let t_h = sample_times(first, end, extra_times_h);
    let f_a: Vec<f64> = t_h
        .iter()
        .map(|t| curve.at(*t).unwrap_or(f64::NAN))
        .collect();
    let f_r: Vec<f64> = t_h
        .iter()
        .map(|t| curve.at_refined(*t).unwrap_or(f64::NAN))
        .collect();
    let res: Vec<bool> = f_a
        .iter()
        .zip(&f_r)
        .map(|(a, r)| resolved(*a, *r))
        .collect();
    let times_s: Vec<f64> = t_h.iter().map(|t| t * H).collect();
    let b = method_b(
        &Needs::baseline(t_c),
        Quantity::ReleaseFraction,
        1.0,
        &radii.outer_m,
        &curve.d_m2_s,
        &times_s,
    );
    let key = format!("s1.{}.{}.{}c", ctx.slug(), nuc.id, t_c.round() as i64);
    let (f_b, floor, valid_h, b_json) = b_series_json(
        &b,
        &t_h,
        "fraction of the initial inventory released (dimensionless)",
    );
    let mut csv_rows = Vec::new();
    for k in 0..t_h.len() {
        csv_rows.push(vec![
            key.clone(),
            csv_cell(t_h[k]),
            csv_cell(f_a[k]),
            res[k].to_string(),
            f_b[k].map_or(String::new(), csv_cell),
        ]);
    }
    let json = json_file(
        prov,
        "curve",
        vec![
            ("key", J::s(key.clone())),
            (
                "selection",
                J::obj(vec![
                    ("stage", J::Int(1)),
                    ("context", J::s(ctx.slug())),
                    ("nuclide", J::s(nuc.name)),
                    ("temperature_c", J::Num(t_c)),
                    ("phase_of_correlations", J::s(phase.as_str())),
                ]),
            ),
            ("labels", J::Arr(labels.iter().map(J::s).collect())),
            (
                "axes",
                J::obj(vec![
                    (
                        "x",
                        J::obj(vec![
                            (
                                "quantity",
                                J::s(if ctx == Context::Exp {
                                    "Time at test temperature"
                                } else {
                                    "Time at constant temperature"
                                }),
                            ),
                            ("unit", J::s("h")),
                        ]),
                    ),
                    (
                        "y",
                        J::obj(vec![
                            ("quantity", J::s("Fraction released")),
                            ("unit", J::s("dimensionless")),
                        ]),
                    ),
                    ("default_y_scale", J::s("log")),
                ]),
            ),
            (
                "method_a",
                J::obj(vec![
                    ("label", J::s("Method A: BTCS")),
                    ("time_h", J::Arr(t_h.iter().map(|t| J::Num(*t)).collect())),
                    ("release_fraction", J::Arr(f_a.iter().map(|v| J::Num(*v)).collect())),
                    ("release_fraction_refined", J::Arr(f_r.iter().map(|v| J::Num(*v)).collect())),
                    ("resolved", J::Arr(res.iter().map(|r| J::Bool(*r)).collect())),
                    (
                        "grid",
                        J::obj(vec![
                            ("intervals_per_layer", J::Arr(curve.settings.intervals.iter().map(|n| J::Int(*n as i64)).collect())),
                            ("dt_h", J::Num(curve.settings.dt_h)),
                        ]),
                    ),
                    ("diffusivity_m2_s", J::Arr(curve.d_m2_s.iter().map(|v| J::Num(*v)).collect())),
                    ("start", J::s("kernel-only start (decision 20); the temperature ramp is not modelled in stage 1")),
                ]),
            ),
            ("method_b", b_json),
            (
                "numerics",
                J::obj(vec![
                    ("resolution_rule", J::s("numerical setting (chosen; refinement-checked per Section 3.5): resolved if the base and the refined run (twice the intervals, half the step) differ by at most 5 % of the refined value; a labelled convention, not a derived bound")),
                    ("worst_relative_balance_residual", J::Num(curve.worst_balance_residual)),
                    ("worst_relative_balance_bound", J::Num(curve.worst_balance_bound)),
                ]),
            ),
            ("reference_normal_operation", J::Null),
        ],
    );
    Ok((
        json,
        Stage1 {
            key,
            t_h,
            f_a,
            f_refined: f_r,
            resolved: res,
            f_b,
            b_floor: floor,
            b_valid_from_h: valid_h,
            csv_rows,
        },
    ))
}

/// Make the comparison machinery's curve for an irradiation-phase exploration temperature (the
/// C15 rows).
///
/// # Errors
///
/// A missing law or a failed run.
pub fn irradiation_curve(
    reg: &Register,
    nuc: &Nuc,
    t_c: f64,
    radii: &Radii,
    quick: bool,
) -> Result<Curve, String> {
    let laws = Laws::new(reg, nuc.species.label(), Phase::Irradiation)?;
    let (d, _) = laws.at(t_c).map_err(|e| e.to_string())?;
    let ev = Evaluated {
        d_m2_s: d,
        outside_printed_range: false,
        any_printed_range: false,
        any_upper_bound: false,
    };
    let settings = if quick {
        Settings {
            intervals: [20, 15, 10, 20, 10],
            dt_h: 500.0,
        }
    } else {
        Settings {
            intervals: [160, 20, 20, 160, 20],
            dt_h: 5.0,
        }
    };
    model_curve(
        &format!("{} {} degC (irradiation correlations)", nuc.name, t_c),
        nuc.species,
        t_c,
        radii,
        &ev,
        settings,
        620.2 * 24.0,
        true,
    )
}

/// A registered heating history as a temperature table.
pub struct History {
    /// Breakpoint times in s.
    pub times_s: Vec<f64>,
    /// Temperatures in degC.
    pub temps_c: Vec<f64>,
    /// Step plan, one segment per history row.
    pub plan: Vec<StepSegment>,
    /// Start of the final hold in s.
    pub hold_start_s: f64,
    /// Source rows.
    pub row_ids: Vec<String>,
}

/// The registered history `hist` up to the end of the final hold, which is extended to
/// `hold_h` hours (the registered nominal hold is 300 h; as-run holds reach 400 h).
///
/// # Errors
///
/// A history that is missing or has no hold.
pub fn history(reg: &Register, hist: &str, hold_h: f64, dt_max_s: f64) -> Result<History, String> {
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
    let last_hold = rows
        .iter()
        .rposition(|r| r.hold_h.is_some())
        .ok_or("the history has no hold")?;
    let mut times = vec![0.0];
    let mut temps = vec![rows[0]
        .t_start
        .as_ref()
        .and_then(|p| p.celsius())
        .ok_or("no start temperature")?];
    let mut plan = Vec::new();
    let mut ids = Vec::new();
    let mut clock = 0.0;
    let mut hold_start = 0.0;
    for (k, r) in rows.iter().enumerate().take(last_hold + 1) {
        let hours = if k == last_hold {
            hold_start = clock;
            hold_h
        } else {
            r.ramp_h
                .as_ref()
                .or(r.hold_h.as_ref())
                .ok_or_else(|| format!("{}: no duration", r.id))?
                .value
        };
        clock += hours * H;
        times.push(clock);
        temps.push(
            r.t_end
                .as_ref()
                .and_then(|p| p.celsius())
                .ok_or_else(|| format!("{}: no end temperature", r.id))?,
        );
        plan.push(StepSegment {
            end_s: clock,
            dt_max_s,
        });
        ids.push(r.id.clone());
    }
    Ok(History {
        times_s: times,
        temps_c: temps,
        plan,
        hold_start_s: hold_start,
        row_ids: ids,
    })
}

/// A decaying or ramped heating-test run of the engine with a kernel-only start.
struct EngineRun {
    base: ScenarioRun,
    fine: ScenarioRun,
}

fn engine_pair(
    reg: &Register,
    nuc: &Nuc,
    def: &ScenarioDefinition,
    lambda: f64,
    radii: &Radii,
    quick: bool,
) -> Result<EngineRun, String> {
    let laws = Laws::new(reg, nuc.species.label(), Phase::HeatingTest)?;
    let f = |_t: f64, t_c: f64| laws.at(t_c);
    let intervals: [usize; 5] = if quick {
        [20, 15, 10, 20, 10]
    } else {
        [160, 20, 20, 160, 20]
    };
    let go = |scale: f64, mult: usize| -> Result<ScenarioRun, String> {
        let mut d = def.clone();
        for s in &mut d.plan {
            s.dt_max_s *= scale;
        }
        let cfg = ScenarioConfig {
            radii_m: radii.outer_m,
            intervals: intervals.map(|n| n * mult),
            initial: InitialState::KernelUniform { c_init: 1.0 },
            decay_constant_per_s: lambda,
            reference_generation_mol_m3_s: 0.0,
            diffusion: Diffusion::OfTemperature(&f),
            profile_times_s: vec![],
            extra_boundaries_s: vec![],
        };
        run_scenario(&d, &cfg).map_err(|e| e.to_string())
    };
    Ok(EngineRun {
        base: go(1.0, 1)?,
        fine: go(0.5, 2)?,
    })
}

/// What the engine curves return to the callers.
#[derive(Debug, Clone)]
pub struct EngineCurve {
    /// Cell key.
    pub key: String,
    /// Sample hold times, hours.
    pub t_h: Vec<f64>,
    /// Method A at the sample times.
    pub f_a: Vec<f64>,
    /// Resolved flags.
    pub resolved: Vec<bool>,
    /// Largest `|eps| / bound`.
    pub max_residual_over_bound: f64,
    /// Largest relative residual.
    pub max_relative_residual: f64,
    /// Steps and assemblies.
    pub steps: usize,
    /// Matrix assemblies.
    pub assemblies: usize,
    /// CSV rows.
    pub csv_rows: Vec<Vec<String>>,
    /// The Method B JSON of the cell (`available`, the values or the reason).
    pub method_b: J,
    /// True if at least one Method B value is drawn.
    pub b_available: bool,
    /// The temperature history of the run, as Method B's applicability reads it.
    pub temperature: TemperatureHistory,
    /// Method B at the sample times where the drawing mask accepts it.
    pub f_b: Vec<Option<f64>>,
    /// Method B accuracy floor, when it is available.
    pub b_floor: Option<f64>,
    /// First valid time of Method B, hours.
    pub b_valid_from_h: Option<f64>,
}

/// Stage 3 (decay at constant temperature) or stage 4 (the registered ramp history) of an EXP
/// cell.
///
/// # Errors
///
/// A failed run.
#[allow(clippy::too_many_arguments)]
pub fn engine_cell(
    prov: &Prov,
    reg: &Register,
    nuc: &Nuc,
    t_c: f64,
    stage: u8,
    radii: &Radii,
    quick: bool,
    extra_times_h: &[f64],
) -> Result<(String, EngineCurve), String> {
    let lambda = if stage == 3 {
        decay_constant(reg, nuc.name)?
    } else {
        0.0
    };
    let dt = if quick { 0.5 * H } else { 0.05 * H };
    let (def, hold_start_s, hist_ids) = if stage == 4 {
        let h = history(
            reg,
            &format!("AGR1-ST-C16-{}", t_c.round() as i64),
            HOLD_END_H,
            dt,
        )?;
        let d = ScenarioDefinition {
            id: format!("stage4-{}-{}", nuc.id, t_c),
            generation_factor: TimeFunction::constant(0.0).map_err(|e| e.to_string())?,
            temperature_c: TimeFunction::table(
                h.times_s.clone(),
                h.temps_c.clone(),
                Interpolation::Linear,
            )
            .map_err(|e| e.to_string())?,
            plan: h.plan.clone(),
            event_times_s: vec![],
        };
        (d, h.hold_start_s, h.row_ids)
    } else {
        let end = HOLD_END_H * H;
        (
            ScenarioDefinition {
                id: format!("stage3-{}-{}", nuc.id, t_c),
                generation_factor: TimeFunction::constant(0.0).map_err(|e| e.to_string())?,
                temperature_c: TimeFunction::constant(t_c).map_err(|e| e.to_string())?,
                plan: vec![StepSegment {
                    end_s: end,
                    dt_max_s: dt,
                }],
                event_times_s: vec![],
            },
            0.0,
            vec![],
        )
    };
    let pair = engine_pair(reg, nuc, &def, lambda, radii, quick)?;
    let times: Vec<f64> = pair
        .base
        .rows
        .iter()
        .map(|r| (r.t_s - hold_start_s) / H)
        .collect();
    let ts: Vec<f64> = pair.base.rows.iter().map(|r| r.t_s).collect();
    let f_base: Vec<f64> = pair
        .base
        .rows
        .iter()
        .map(|r| r.release_fraction_of_initial.unwrap_or(0.0))
        .collect();
    let tf: Vec<f64> = pair.fine.rows.iter().map(|r| r.t_s).collect();
    let ff: Vec<f64> = pair
        .fine
        .rows
        .iter()
        .map(|r| r.release_fraction_of_initial.unwrap_or(0.0))
        .collect();
    let end_h = HOLD_END_H;
    let sample = sample_times(0.05, end_h, extra_times_h);
    let mut f_a = Vec::new();
    let mut res = Vec::new();
    let mut inv = Vec::new();
    let mut dec = Vec::new();
    let mut temp = Vec::new();
    let m0 = pair.base.initial_inventory_mol;
    let inv_b: Vec<f64> = pair
        .base
        .rows
        .iter()
        .map(|r| r.inventory_mol / m0)
        .collect();
    let dec_b: Vec<f64> = pair.base.rows.iter().map(|r| r.decayed_mol / m0).collect();
    let tmp_b: Vec<f64> = pair.base.rows.iter().map(|r| r.temperature_c).collect();
    for x in &sample {
        let t_s = hold_start_s + x * H;
        let a = lin(&ts, &f_base, t_s).unwrap_or(f64::NAN);
        let r = lin(&tf, &ff, t_s).unwrap_or(f64::NAN);
        f_a.push(a);
        res.push(resolved(a, r));
        inv.push(lin(&ts, &inv_b, t_s).unwrap_or(f64::NAN));
        dec.push(lin(&ts, &dec_b, t_s).unwrap_or(f64::NAN));
        temp.push(lin(&ts, &tmp_b, t_s).unwrap_or(f64::NAN));
    }
    let key = format!("s{stage}.exp.{}.{}c", nuc.id, t_c.round() as i64);
    // Method B through the one function: the registered temperature history decides whether it
    // applies (a ramp never), and decay at a constant temperature goes through extension (B2).
    let needs = Needs {
        stage: if stage == 3 {
            Stage::Decay
        } else {
            Stage::TemperatureDependentDiffusivity
        },
        source: SourceHistory::None,
        decay_per_s: lambda,
        temperature: temperature_of(&def.temperature_c),
        kernel_step_start: true,
    };
    let laws = Laws::new(reg, nuc.species.label(), Phase::HeatingTest)?;
    let (d_at_t, _) = laws.at(t_c).map_err(|e| e.to_string())?;
    let sample_s: Vec<f64> = sample.iter().map(|x| x * H).collect();
    let b = method_b(
        &needs,
        Quantity::ReleaseFraction,
        1.0,
        &radii.outer_m,
        &d_at_t,
        &sample_s,
    );
    let (f_b, b_floor, b_valid_from_h, b_json) = b_series_json(
        &b,
        &sample,
        "fraction of the initial inventory released (dimensionless)",
    );
    let b_available = f_b.iter().any(Option::is_some);
    // The whole temperature history, in hours from the start of heating, for the replaced panel.
    let hist_t: Vec<f64> = if stage == 4 {
        pair.base.rows.iter().map(|r| r.t_s / H).collect()
    } else {
        vec![]
    };
    let hist_pick: Vec<usize> = if stage == 4 {
        let n = hist_t.len();
        let stride = n.div_ceil(300).max(1);
        let mut v: Vec<usize> = (0..n).step_by(stride).collect();
        v.push(n - 1);
        v.dedup();
        v
    } else {
        vec![]
    };
    let mut csv_rows = Vec::new();
    for k in 0..sample.len() {
        csv_rows.push(vec![
            key.clone(),
            csv_cell(sample[k]),
            csv_cell(f_a[k]),
            res[k].to_string(),
            f_b[k].map_or(String::new(), csv_cell),
        ]);
    }
    let labels = [
        "numerical solution of the stated model; not experimental validation".to_string(),
        "kernel-only start (decision 20); published heating-test correlations C16 as printed, evaluated at the temperature of each step".to_string(),
        if stage == 4 {
            format!(
                "registered history AGR1-ST-C16-{} (rows {}); the hold is extended to {} h to cover as-run holds (the registered nominal hold is 300 h); the time axis is the time at test temperature",
                t_c.round() as i64,
                hist_ids.join(", "),
                HOLD_END_H
            )
        } else {
            format!(
                "decay constant of {} from decay.csv; the decay-free comparison of Section 3.6 is stage 1",
                nuc.name
            )
        },
    ];
    let json = json_file(
        prov,
        "curve",
        vec![
            ("key", J::s(key.clone())),
            (
                "selection",
                J::obj(vec![
                    ("stage", J::Int(stage as i64)),
                    ("context", J::s("exp")),
                    ("nuclide", J::s(nuc.name)),
                    ("temperature_c", J::Num(t_c)),
                    ("history_row_ids", J::Arr(hist_ids.iter().map(J::s).collect())),
                    ("phase_of_correlations", J::s("heating test")),
                ]),
            ),
            ("labels", J::Arr(labels.iter().map(J::s).collect())),
            (
                "axes",
                J::obj(vec![
                    ("x", J::obj(vec![("quantity", J::s("Time at test temperature")), ("unit", J::s("h"))])),
                    ("y", J::obj(vec![("quantity", J::s("Fraction released")), ("unit", J::s("dimensionless"))])),
                    ("default_y_scale", J::s("log")),
                ]),
            ),
            (
                "method_a",
                J::obj(vec![
                    ("label", J::s("Method A: BTCS")),
                    ("time_h", J::Arr(sample.iter().map(|t| J::Num(*t)).collect())),
                    ("release_fraction", J::Arr(f_a.iter().map(|v| J::Num(*v)).collect())),
                    ("resolved", J::Arr(res.iter().map(|r| J::Bool(*r)).collect())),
                    ("decay_constant_per_s", J::Num(lambda)),
                    ("steps", J::Int(pair.base.steps as i64)),
                    ("matrix_assemblies", J::Int(pair.base.assemblies as i64)),
                ]),
            ),
            ("method_b", b_json.clone()),
            (
                "quantities",
                J::obj(vec![
                    (
                        "remaining_inventory_fraction",
                        J::obj(vec![
                            ("unit", J::s("dimensionless (M / M^0)")),
                            ("time_h", J::Arr(sample.iter().map(|t| J::Num(*t)).collect())),
                            ("values", J::Arr(inv.iter().map(|v| J::Num(*v)).collect())),
                        ]),
                    ),
                    (
                        "decayed_fraction",
                        J::obj(vec![
                            ("unit", J::s("dimensionless (M_dec / M^0)")),
                            ("time_h", J::Arr(sample.iter().map(|t| J::Num(*t)).collect())),
                            ("values", J::Arr(dec.iter().map(|v| J::Num(*v)).collect())),
                        ]),
                    ),
                    (
                        "temperature",
                        J::obj(vec![
                            ("unit", J::s("\u{b0}C")),
                            ("time_h", J::Arr(sample.iter().map(|t| J::Num(*t)).collect())),
                            ("values", J::Arr(temp.iter().map(|v| J::Num(*v)).collect())),
                        ]),
                    ),
                    (
                        "temperature_history",
                        J::obj(vec![
                            ("unit", J::s("\u{b0}C")),
                            ("time_since_start_of_heating_h", J::Arr(hist_pick.iter().map(|k| J::Num(hist_t[*k])).collect())),
                            ("values", J::Arr(hist_pick.iter().map(|k| J::Num(tmp_b[*k])).collect())),
                        ]),
                    ),
                ]),
            ),
            (
                "numerics",
                J::obj(vec![
                    ("intervals_per_layer", J::Arr(if quick { [20, 15, 10, 20, 10] } else { [160, 20, 20, 160, 20] }.iter().map(|n| J::Int(*n as i64)).collect())),
                    ("max_abs_balance_residual_over_bound", J::Num(pair.base.max_residual_over_bound)),
                    ("max_relative_balance_residual", J::Num(pair.base.max_relative_residual)),
                    ("balance_bound", J::s("(F46), Document 5 Section 5.8")),
                    ("diffusivity_ranges_note", J::s("the C16 rows print no validity range; the heating-test correlation is applied over the registered history as Collin (2016) did")),
                ]),
            ),
            ("reference_normal_operation", J::Null),
        ],
    );
    let _ = (&times, &hist_t);
    Ok((
        json,
        EngineCurve {
            key,
            t_h: sample,
            f_a,
            resolved: res,
            max_residual_over_bound: pair.base.max_residual_over_bound,
            max_relative_residual: pair.base.max_relative_residual,
            steps: pair.base.steps,
            assemblies: pair.base.assemblies,
            csv_rows,
            method_b: b_json,
            b_available,
            temperature: needs.temperature.clone(),
            f_b,
            b_floor,
            b_valid_from_h,
        },
    ))
}

/// Runner helper: the stage-1 curve of a nuclide at a heating-test temperature.
///
/// # Errors
///
/// A failed run.
pub fn heating_curve<'r>(
    runner: &'r mut Runner<'_>,
    nuc: &Nuc,
    t_c: f64,
) -> Result<&'r Curve, String> {
    runner.get(nuc.species, t_c, HOLD_END_H)
}
