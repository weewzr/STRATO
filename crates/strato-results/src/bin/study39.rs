//! Canonical Phase 2 39-graph simulation runner for STRATO (`Cs-137`, `AGR1-B-M16`, C16/C15).
//!
//! Runs Method A (BTCS, `ScenarioConfig` / `run_scenario`) and Method B (Sturm-Liouville)
//! across Stages A, B, C, and D at all six heating-test temperatures (1200, 1350, 1500, 1600,
//! 1700, 1800 degC) plus AGR-1 irradiation temperatures (955, 1028, 1136 degC), radial
//! profiles, and numerical convergence studies (`spatial_convergence` / `temporal_convergence`).
//!
//! Outputs canonical JSON and CSV files into `<repo>/data/study39/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use strato::analytical::applicability::{SourceHistory, Stage};
use strato::analytical::extensions::Quantity;
use strato::numerical::physics_run::{Interpolation, TimeFunction};
use strato::numerical::scenario::{
    run_scenario, Diffusion, InitialState, ScenarioConfig, ScenarioDefinition, ScenarioRun,
    StepSegment,
};
use strato_compare::data::radii;
use strato_compare::json::J;
use strato_compare::studies::{spatial_convergence, temporal_convergence, ConvergenceRow};
use strato_data::register::RegisterTexts;
use strato_data::scenarios::ScenarioTexts;
use strato_data::select::Phase;
use strato_data::Register;
use strato_results::curves::{b_series_json, history, lin, sample_times};
use strato_results::methodb::{method_b, source_of, temperature_of, MethodB, Needs};
use strato_results::output::{json_file, Prov};
use strato_results::params::{
    decay_constant, generation_nuclides, rate_density, reference_generation, Laws,
};

const H: f64 = 3600.0;
const BASE_INTERVALS: [usize; 5] = [160, 20, 20, 160, 20];
const REFINE_INTERVALS: [usize; 5] = [320, 40, 40, 320, 40];
const N_TIME_STEPS: usize = 10_000;

struct TempSpec {
    t_c: f64,
    hold_h: f64,
    end_h: f64,
    registered_hist: Option<&'static str>,
    extra_h: &'static [f64],
}

const TEMPS: [TempSpec; 6] = [
    TempSpec {
        t_c: 1200.0,
        hold_h: 10138.0,
        end_h: 10138.0,
        registered_hist: None,
        extra_h: &[10138.0],
    },
    TempSpec {
        t_c: 1350.0,
        hold_h: 10685.0,
        end_h: 10685.0,
        registered_hist: None,
        extra_h: &[10685.0],
    },
    TempSpec {
        t_c: 1500.0,
        hold_h: 11866.0,
        end_h: 11866.0,
        registered_hist: None,
        extra_h: &[816.0, 3370.0, 11800.0, 11866.0, 11900.0],
    },
    TempSpec {
        t_c: 1600.0,
        hold_h: 400.0,
        end_h: 400.0,
        registered_hist: Some("AGR1-ST-C16-1600"),
        extra_h: &[160.0, 300.0, 400.0],
    },
    TempSpec {
        t_c: 1700.0,
        hold_h: 272.0,
        end_h: 272.0,
        registered_hist: Some("AGR1-ST-C16-1700"),
        extra_h: &[72.0, 102.0, 130.0, 175.0, 210.0, 240.0, 270.0, 271.0, 272.0],
    },
    TempSpec {
        t_c: 1800.0,
        hold_h: 222.0,
        end_h: 222.0,
        registered_hist: Some("AGR1-ST-C16-1800"),
        extra_h: &[4.6, 15.0, 25.0, 50.0, 78.7, 110.0, 150.0, 185.0, 222.0],
    },
];

fn write_text(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(path, content).map_err(|e| format!("{}: {e}", path.display()))
}

fn resolved(a: f64, refined: f64) -> bool {
    refined > 0.0 && (a - refined).abs() <= 0.05 * refined
}

fn build_plan_and_extra(
    end_h: f64,
    nominal_h: f64,
    extra_h: &[f64],
) -> (Vec<StepSegment>, Vec<f64>) {
    let dt_nom = (nominal_h * H) / (N_TIME_STEPS as f64);
    let plan = if end_h > nominal_h {
        vec![
            StepSegment {
                end_s: nominal_h * H,
                dt_max_s: dt_nom,
            },
            StepSegment {
                end_s: end_h * H,
                dt_max_s: 0.025 * H,
            },
        ]
    } else {
        vec![StepSegment {
            end_s: end_h * H,
            dt_max_s: dt_nom,
        }]
    };
    let extra_s: Vec<f64> = extra_h
        .iter()
        .copied()
        .filter(|t| *t > 0.0 && *t <= end_h)
        .map(|t| t * H)
        .collect();
    (plan, extra_s)
}

fn build_ramp_history(
    reg: &Register,
    spec: &TempSpec,
    hold_h: f64,
    dt_ramp_h: f64,
    dt_hold_s: f64,
) -> Result<(ScenarioDefinition, f64, String, Vec<String>), String> {
    if let Some(hist_id) = spec.registered_hist {
        let h = history(reg, hist_id, hold_h, dt_ramp_h * H)?;
        let mut plan = h.plan.clone();
        if let Some(last) = plan.last_mut() {
            last.dt_max_s = dt_hold_s;
        }
        let def = ScenarioDefinition {
            id: format!("ramp-{}", spec.t_c.round() as i64),
            generation_factor: TimeFunction::constant(0.0).map_err(|e| e.to_string())?,
            temperature_c: TimeFunction::table(
                h.times_s.clone(),
                h.temps_c.clone(),
                Interpolation::Linear,
            )
            .map_err(|e| e.to_string())?,
            plan,
            event_times_s: vec![h.hold_start_s],
        };
        return Ok((def, h.hold_start_s, hist_id.to_string(), h.row_ids));
    }

    // For 1200, 1350, 1500 C: constructed pre-heating ramp matching AGR-1 bake-out + 50 C/h ramp
    let ramp1_h = (400.0 - 30.0) / 120.0;
    let hold1_h = 2.0;
    let ramp2_h = (spec.t_c - 400.0) / 50.0;
    let t0 = 0.0;
    let t1 = ramp1_h * H;
    let t2 = t1 + hold1_h * H;
    let t3 = t2 + ramp2_h * H;
    let t4 = t3 + hold_h * H;
    let def = ScenarioDefinition {
        id: format!("ramp-constructed-{}", spec.t_c.round() as i64),
        generation_factor: TimeFunction::constant(0.0).map_err(|e| e.to_string())?,
        temperature_c: TimeFunction::table(
            vec![t0, t1, t2, t3, t4],
            vec![30.0, 400.0, 400.0, spec.t_c, spec.t_c],
            Interpolation::Linear,
        )
        .map_err(|e| e.to_string())?,
        plan: vec![
            StepSegment {
                end_s: t1,
                dt_max_s: dt_ramp_h * H,
            },
            StepSegment {
                end_s: t2,
                dt_max_s: dt_ramp_h * H,
            },
            StepSegment {
                end_s: t3,
                dt_max_s: dt_ramp_h * H,
            },
            StepSegment {
                end_s: t4,
                dt_max_s: dt_hold_s,
            },
        ],
        event_times_s: vec![t3],
    };
    Ok((
        def,
        t3,
        format!("CONSTRUCTED-50C_PER_H-{}C", spec.t_c.round() as i64),
        vec![],
    ))
}

#[allow(clippy::too_many_arguments)]
fn sample_and_save_case(
    prov: &Prov,
    out_dir: &Path,
    case_key: &str,
    stage_label: &str,
    t_c: f64,
    base: &ScenarioRun,
    fine: &ScenarioRun,
    hold_start_s: f64,
    sample_h: &[f64],
    b_res: &MethodB,
    use_total_gen_fraction: bool,
    quantity_label: &str,
    extra_meta: Vec<(&str, J)>,
    balance_rows: &mut Vec<String>,
) -> Result<(f64, Option<f64>, bool, f64, f64), String> {
    let tb: Vec<f64> = base.rows.iter().map(|r| r.t_s).collect();
    let tf: Vec<f64> = fine.rows.iter().map(|r| r.t_s).collect();
    let extract = |run: &ScenarioRun| -> Vec<f64> {
        run.rows
            .iter()
            .map(|r| {
                if use_total_gen_fraction {
                    r.release_fraction_of_total.unwrap_or(0.0)
                } else {
                    r.release_fraction_of_initial.unwrap_or(0.0)
                }
            })
            .collect()
    };
    let fb_raw = extract(base);
    let ff_raw = extract(fine);

    let (f_b, b_floor, b_valid_from_h, b_json) = b_series_json(b_res, sample_h, quantity_label);

    let mut f_a = Vec::with_capacity(sample_h.len());
    let mut f_ref = Vec::with_capacity(sample_h.len());
    let mut res = Vec::with_capacity(sample_h.len());
    let mut csv_lines = vec![
        "time_h,time_s,f_method_a,f_refined,resolved,f_method_b,residual_over_bound".to_string(),
    ];

    for (i, &xh) in sample_h.iter().enumerate() {
        let t_s = hold_start_s + xh * H;
        let a = lin(&tb, &fb_raw, t_s).unwrap_or(f64::NAN);
        let r = lin(&tf, &ff_raw, t_s).unwrap_or(f64::NAN);
        let is_res = resolved(a, r);
        f_a.push(a);
        f_ref.push(r);
        res.push(is_res);
        let b_str = f_b[i].map_or(String::new(), |v| format!("{v:.16e}"));
        csv_lines.push(format!(
            "{xh:.8},{t_s:.6},{a:.16e},{r:.16e},{is_res},{b_str},{:.6e}",
            base.max_residual_over_bound
        ));
    }

    let b_available = f_b.iter().any(Option::is_some);
    let final_a = *f_a.last().unwrap_or(&f64::NAN);
    let final_b = *f_b.last().unwrap_or(&None);
    let all_resolved = res.iter().zip(f_a.iter()).all(|(ok, v)| *ok || *v <= 0.0);

    let mut fields = vec![
        ("key", J::s(case_key)),
        ("stage", J::s(stage_label)),
        ("nuclide", J::s("Cs-137")),
        ("temperature_c", J::Num(t_c)),
        ("hold_start_s", J::Num(hold_start_s)),
        ("plotted_quantity", J::s(quantity_label)),
        (
            "method_a",
            J::obj(vec![
                ("label", J::s("Method A: FDM-BTCS")),
                (
                    "time_h",
                    J::Arr(sample_h.iter().map(|t| J::Num(*t)).collect()),
                ),
                (
                    "release_fraction",
                    J::Arr(f_a.iter().map(|v| J::Num(*v)).collect()),
                ),
                (
                    "refined_release_fraction",
                    J::Arr(f_ref.iter().map(|v| J::Num(*v)).collect()),
                ),
                (
                    "resolved",
                    J::Arr(res.iter().map(|r| J::Bool(*r)).collect()),
                ),
                ("steps", J::Int(base.steps as i64)),
                ("distinct_steps", J::Int(base.distinct_steps as i64)),
                ("matrix_assemblies", J::Int(base.assemblies as i64)),
                (
                    "intervals_per_layer",
                    J::Arr(BASE_INTERVALS.iter().map(|n| J::Int(*n as i64)).collect()),
                ),
            ]),
        ),
        ("method_b", b_json),
        (
            "numerics",
            J::obj(vec![
                (
                    "max_residual_over_bound",
                    J::Num(base.max_residual_over_bound),
                ),
                ("max_relative_residual", J::Num(base.max_relative_residual)),
                ("matrix_min_row_margin", J::Num(base.matrix_min_row_margin)),
                ("min_pivot_margin", J::Num(base.min_pivot_margin)),
                ("b_accuracy_floor", J::opt(b_floor)),
                ("b_valid_from_h", J::opt(b_valid_from_h)),
            ]),
        ),
    ];
    fields.extend(extra_meta);
    let json = json_file(prov, "study39_curve", fields);

    write_text(
        &out_dir.join("curves").join(format!("{case_key}.json")),
        &json,
    )?;
    write_text(
        &out_dir.join("curves").join(format!("{case_key}.csv")),
        &csv_lines.join("\n"),
    )?;

    balance_rows.push(format!(
        "{case_key},{stage_label},{t_c},{},{},{:.6e},{:.6e},{:.6e},{:.6e}",
        base.steps,
        base.assemblies,
        base.max_residual_over_bound,
        base.max_relative_residual,
        base.matrix_min_row_margin,
        base.min_pivot_margin
    ));

    Ok((
        final_a,
        final_b,
        b_available,
        base.max_residual_over_bound,
        if all_resolved { 1.0 } else { 0.0 },
    ))
}

fn conv_json_rows(rows: &[ConvergenceRow]) -> J {
    J::Arr(
        rows.iter()
            .map(|r| {
                J::obj(vec![
                    ("level", J::Int(r.level as i64)),
                    ("step", J::Num(r.step)),
                    ("count", J::Int(r.count as i64)),
                    ("f_solver", J::Num(r.f_solver)),
                    ("f_analytic", J::Num(r.f_analytic)),
                    ("f_analytic_bound", J::Num(r.f_analytic_bound)),
                    ("error", J::Num(r.error)),
                    ("order", J::opt(r.order)),
                ])
            })
            .collect(),
    )
}

fn main() -> ExitCode {
    let mut repo = PathBuf::from(".");
    let mut commit = String::from("HEAD");
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--repo" => {
                if let Some(p) = args.next() {
                    repo = PathBuf::from(p);
                }
            }
            "--commit" => {
                if let Some(c) = args.next() {
                    commit = c;
                }
            }
            _ => {}
        }
    }

    if let Err(e) = run_study39(&repo, &commit) {
        eprintln!("study39 failed: {e}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn run_study39(repo: &Path, commit: &str) -> Result<(), String> {
    let t0 = Instant::now();
    let data_dir = repo.join("data");
    let out_dir = data_dir.join("study39");

    let texts = RegisterTexts::read(&data_dir).map_err(|e| e.to_string())?;
    let scen_texts = ScenarioTexts::read(&data_dir).map_err(|e| e.to_string())?;
    let reg = Register::from_texts(&texts).map_err(|e| e.to_string())?;
    let prov = Prov::new(repo, commit, &texts, &scen_texts)?;

    let geometry = radii(&reg)?;
    let r_m = geometry.outer_m;

    let nucs = generation_nuclides(&reg)?;
    let cs137 = nucs
        .iter()
        .find(|n| n.name == "Cs-137")
        .ok_or("Cs-137 not found in generation_nuclides")?;
    let lambda = decay_constant(&reg, "Cs-137")?;
    let rate = rate_density(&reg, "capsule:3")?;
    let g_ref = reference_generation(cs137.yield_per_fission, rate.value);

    let laws_c16 = Laws::new(&reg, "Cs", Phase::HeatingTest)?;
    let laws_c15 = Laws::new(&reg, "Cs", Phase::Irradiation)?;

    println!(
        "study39 starting: Cs-137 lambda={lambda:.6e} s^-1, y={:.6}, Fdot={:.6e} m^-3 s^-1, G_ref={g_ref:.6e} mol m^-3 s^-1",
        cs137.yield_per_fission, rate.value
    );

    // 1. Write parameters.json
    let mut c16_rows_j = Vec::new();
    for spec in &TEMPS {
        let (d, _) = laws_c16.at(spec.t_c).map_err(|e| e.to_string())?;
        c16_rows_j.push(J::obj(vec![
            ("temperature_c", J::Num(spec.t_c)),
            ("temperature_k", J::Num(spec.t_c + 273.15)),
            (
                "diffusivities_m2_s",
                J::Arr(d.iter().map(|v| J::Num(*v)).collect()),
            ),
        ]));
    }
    let mut c15_rows_j = Vec::new();
    for &t_c in &[955.0_f64, 1028.0, 1136.0] {
        let (d, _) = laws_c15.at(t_c).map_err(|e| e.to_string())?;
        c15_rows_j.push(J::obj(vec![
            ("temperature_c", J::Num(t_c)),
            ("temperature_k", J::Num(t_c + 273.15)),
            (
                "diffusivities_m2_s",
                J::Arr(d.iter().map(|v| J::Num(*v)).collect()),
            ),
        ]));
    }
    let params_json = json_file(
        &prov,
        "study39_parameters",
        vec![
            ("nuclide", J::s("Cs-137")),
            ("geometry_id", J::s("AGR1-B-M16")),
            (
                "outer_radii_m",
                J::Arr(r_m.iter().map(|r| J::Num(*r)).collect()),
            ),
            (
                "outer_radii_um",
                J::Arr(r_m.iter().map(|r| J::Num(*r * 1e6)).collect()),
            ),
            ("decay_constant_per_s", J::Num(lambda)),
            ("yield_per_fission", J::Num(cs137.yield_per_fission)),
            ("fission_rate_density_per_m3_s", J::Num(rate.value)),
            ("reference_generation_mol_m3_s", J::Num(g_ref)),
            (
                "intervals_base",
                J::Arr(BASE_INTERVALS.iter().map(|n| J::Int(*n as i64)).collect()),
            ),
            (
                "intervals_refined",
                J::Arr(REFINE_INTERVALS.iter().map(|n| J::Int(*n as i64)).collect()),
            ),
            ("c16_diffusivities", J::Arr(c16_rows_j)),
            ("c15_diffusivities", J::Arr(c15_rows_j)),
        ],
    );
    write_text(&out_dir.join("parameters.json"), &params_json)?;

    // Helper closures for dynamic C16 diffusivity
    let dyn_c16 = |_t: f64, tc: f64| laws_c16.at(tc);
    let dyn_c15 = |_t: f64, tc: f64| laws_c15.at(tc);

    let mut balance_rows = vec![
        "case_key,stage,t_c,steps,assemblies,max_residual_over_bound,max_relative_residual,min_row_margin,min_pivot_margin".to_string(),
    ];
    let mut summary_rows = vec![
        "temperature_c,duration_h,stage_key,final_f_method_a,final_f_method_b,b_available,max_residual_over_bound".to_string(),
    ];

    let mut profile_entries: Vec<J> = Vec::new();
    let mut c02_entries: Vec<J> = Vec::new();
    let mut c02_all24_entries: Vec<J> = Vec::new();
    let mut stage_b_check_rows: Vec<J> = Vec::new();
    let c_init_ctrl = g_ref * (620.2 * 24.0 * H);

    let mut push_c02_24 = |t_c: f64,
                           cfg_id: &str,
                           stage_name: &str,
                           run_base: &ScenarioRun,
                           b_res: &MethodB,
                           use_total: bool,
                           sample_h: &[f64],
                           sample_s: &[f64]| {
        match b_res {
            MethodB::Available(avail) => {
                let tb: Vec<f64> = run_base.rows.iter().map(|r| r.t_s).collect();
                let fb_raw: Vec<f64> = run_base
                    .rows
                    .iter()
                    .map(|r| {
                        if use_total {
                            r.release_fraction_of_total.unwrap_or(0.0)
                        } else {
                            r.release_fraction_of_initial.unwrap_or(0.0)
                        }
                    })
                    .collect();
                let mut abs_diffs = Vec::new();
                for (k, &ts) in sample_s.iter().enumerate() {
                    if let Some(vb) = avail.values[k] {
                        let va = lin(&tb, &fb_raw, ts).unwrap_or(f64::NAN);
                        abs_diffs.push((va - vb).abs());
                    }
                }
                let max_abs = abs_diffs.iter().copied().fold(0.0_f64, f64::max);
                let final_a =
                    lin(&tb, &fb_raw, *sample_s.last().unwrap_or(&0.0)).unwrap_or(f64::NAN);
                let final_b = avail.values.last().copied().flatten();
                c02_all24_entries.push(J::obj(vec![
                    ("temperature_c", J::Num(t_c)),
                    ("config_id", J::s(cfg_id)),
                    ("stage", J::s(stage_name)),
                    ("b_available", J::Bool(true)),
                    ("compared_points", J::Int(abs_diffs.len() as i64)),
                    ("total_sample_points", J::Int(sample_h.len() as i64)),
                    ("valid_from_h", J::Num(avail.valid_from_s / H)),
                    ("accuracy_floor", J::Num(avail.accuracy_floor)),
                    ("max_abs_diff_pp", J::Num(max_abs * 100.0)),
                    ("final_f_a", J::Num(final_a)),
                    ("final_f_b", J::opt(final_b)),
                ]));
            }
            MethodB::NotAvailable(na) => {
                c02_all24_entries.push(J::obj(vec![
                    ("temperature_c", J::Num(t_c)),
                    ("config_id", J::s(cfg_id)),
                    ("stage", J::s(stage_name)),
                    ("b_available", J::Bool(false)),
                    ("reason_code", J::s(na.code)),
                    ("reason", J::s(na.reason.clone())),
                ]));
            }
        }
    };

    for spec in &TEMPS {
        let t_i = spec.t_c.round() as i64;
        let nominal_h = if (spec.t_c - 1700.0).abs() < 0.5 {
            270.0
        } else {
            spec.end_h
        };
        let (plan_base, extra_s) = build_plan_and_extra(spec.end_h, nominal_h, spec.extra_h);
        let (plan_fine, _) = build_plan_and_extra(spec.end_h, nominal_h, spec.extra_h);
        let plan_fine: Vec<StepSegment> = plan_fine
            .into_iter()
            .map(|s| StepSegment {
                end_s: s.end_s,
                dt_max_s: 0.5 * s.dt_max_s,
            })
            .collect();

        let (d_const, _) = laws_c16.at(spec.t_c).map_err(|e| e.to_string())?;
        let sample_h = sample_times(0.05, spec.end_h, spec.extra_h);
        let sample_s: Vec<f64> = sample_h.iter().map(|x| x * H).collect();

        let profile_times_s = if t_i == 1700 || t_i == 1800 {
            vec![0.1 * spec.end_h * H, 0.5 * spec.end_h * H, spec.end_h * H]
        } else {
            vec![]
        };

        // ---- Stage A: Constant D(T_nom), G=0, lambda=0, KernelUniform(1.0) ----
        let def_a = ScenarioDefinition {
            id: format!("stage_a_{t_i}c"),
            generation_factor: TimeFunction::constant(0.0).map_err(|e| e.to_string())?,
            temperature_c: TimeFunction::constant(spec.t_c).map_err(|e| e.to_string())?,
            plan: plan_base.clone(),
            event_times_s: vec![],
        };
        let def_a_fine = ScenarioDefinition {
            plan: plan_fine.clone(),
            ..def_a.clone()
        };
        let run_a_base = run_scenario(
            &def_a,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::KernelUniform { c_init: 1.0 },
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: 0.0,
                diffusion: Diffusion::Constant(d_const),
                profile_times_s: profile_times_s.clone(),
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let run_a_fine = run_scenario(
            &def_a_fine,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: REFINE_INTERVALS,
                initial: InitialState::KernelUniform { c_init: 1.0 },
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: 0.0,
                diffusion: Diffusion::Constant(d_const),
                profile_times_s: vec![],
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let b_a = method_b(
            &Needs::baseline(spec.t_c),
            Quantity::ReleaseFraction,
            1.0,
            &r_m,
            &d_const,
            &sample_s,
        );
        let key_a = format!("{t_i}c_stage_a");
        let (fa_end, fb_end, b_ok, r_bnd, _) = sample_and_save_case(
            &prov,
            &out_dir,
            &key_a,
            "Stage A",
            spec.t_c,
            &run_a_base,
            &run_a_fine,
            0.0,
            &sample_h,
            &b_a,
            false,
            "F = M_rel / M^0",
            vec![],
            &mut balance_rows,
        )?;
        summary_rows.push(format!(
            "{t_i},{},stage_a,{fa_end:.12e},{},{b_ok},{r_bnd:.6e}",
            spec.end_h,
            fb_end.map_or(String::new(), |v| format!("{v:.12e}"))
        ));

        // Collect Method A vs Method B comparison metrics for C02
        if let MethodB::Available(ref avail) = b_a {
            let mut abs_diffs = Vec::new();
            let mut rel_diffs = Vec::new();
            let tb: Vec<f64> = run_a_base.rows.iter().map(|r| r.t_s).collect();
            let fb_raw: Vec<f64> = run_a_base
                .rows
                .iter()
                .map(|r| r.release_fraction_of_initial.unwrap_or(0.0))
                .collect();
            for (k, &ts) in sample_s.iter().enumerate() {
                if let Some(vb) = avail.values[k] {
                    let va = lin(&tb, &fb_raw, ts).unwrap_or(f64::NAN);
                    let diff = (va - vb).abs();
                    abs_diffs.push(diff);
                    if vb > 0.0 {
                        rel_diffs.push(diff / vb);
                    }
                }
            }
            let max_abs = abs_diffs.iter().copied().fold(0.0_f64, f64::max);
            let max_rel = rel_diffs.iter().copied().fold(0.0_f64, f64::max);
            c02_entries.push(J::obj(vec![
                ("temperature_c", J::Num(spec.t_c)),
                ("stage", J::s("Stage A (baseline)")),
                ("compared_points", J::Int(abs_diffs.len() as i64)),
                ("total_sample_points", J::Int(sample_s.len() as i64)),
                ("valid_from_h", J::Num(avail.valid_from_s / H)),
                ("accuracy_floor", J::Num(avail.accuracy_floor)),
                ("modes", J::Int(avail.modes as i64)),
                ("max_abs_diff_fraction", J::Num(max_abs)),
                ("max_abs_diff_pp", J::Num(max_abs * 100.0)),
                ("max_rel_diff", J::Num(max_rel)),
                ("final_f_a", J::Num(fa_end)),
                ("final_f_b", J::opt(fb_end)),
            ]));
        }
        push_c02_24(
            spec.t_c,
            "P01",
            "Stage A (baseline)",
            &run_a_base,
            &b_a,
            false,
            &sample_h,
            &sample_s,
        );

        // ---- Stage B: Constant D(T_nom), G = G_ref, lambda = 0, Fresh ----
        let def_b = ScenarioDefinition {
            id: format!("stage_b_{t_i}c"),
            generation_factor: TimeFunction::constant(1.0).map_err(|e| e.to_string())?,
            temperature_c: TimeFunction::constant(spec.t_c).map_err(|e| e.to_string())?,
            plan: plan_base.clone(),
            event_times_s: vec![],
        };
        let def_b_fine = ScenarioDefinition {
            plan: plan_fine.clone(),
            ..def_b.clone()
        };
        let run_b_base = run_scenario(
            &def_b,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::Fresh,
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::Constant(d_const),
                profile_times_s: profile_times_s.clone(),
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let run_b_fine = run_scenario(
            &def_b_fine,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: REFINE_INTERVALS,
                initial: InitialState::Fresh,
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::Constant(d_const),
                profile_times_s: vec![],
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        // Cross-check C_init = 1e-9 stand-in vs exact Fresh
        let run_b_eps = run_scenario(
            &def_b,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::KernelUniform { c_init: 1e-9 },
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::Constant(d_const),
                profile_times_s: vec![],
                extra_boundaries_s: vec![],
            },
        )
        .map_err(|e| e.to_string())?;
        let f_fresh_end = run_b_base
            .rows
            .last()
            .and_then(|r| r.release_fraction_of_total)
            .unwrap_or(0.0);
        let f_eps_end = run_b_eps
            .rows
            .last()
            .and_then(|r| r.release_fraction_of_total)
            .unwrap_or(0.0);
        stage_b_check_rows.push(J::obj(vec![
            ("temperature_c", J::Num(spec.t_c)),
            ("f_fresh_end", J::Num(f_fresh_end)),
            ("f_eps_1e_9_end", J::Num(f_eps_end)),
            ("abs_diff_end", J::Num((f_fresh_end - f_eps_end).abs())),
        ]));

        let b_b = method_b(
            &Needs {
                stage: Stage::Generation,
                source: source_of(&def_b.generation_factor, g_ref, def_b.end_s()),
                decay_per_s: 0.0,
                temperature: temperature_of(&def_b.temperature_c),
                kernel_step_start: true,
            },
            Quantity::ReleaseFractionGenerated,
            0.0,
            &r_m,
            &d_const,
            &sample_s,
        );
        let key_b = format!("{t_i}c_stage_b");
        let (fb_a_end, fb_b_end, bb_ok, rb_bnd, _) = sample_and_save_case(
            &prov,
            &out_dir,
            &key_b,
            "Stage B",
            spec.t_c,
            &run_b_base,
            &run_b_fine,
            0.0,
            &sample_h,
            &b_b,
            true,
            "F_gen = M_rel / (M^0 + M_gen)",
            vec![],
            &mut balance_rows,
        )?;
        summary_rows.push(format!(
            "{t_i},{},stage_b,{fb_a_end:.12e},{},{bb_ok},{rb_bnd:.6e}",
            spec.end_h,
            fb_b_end.map_or(String::new(), |v| format!("{v:.12e}"))
        ));
        push_c02_24(
            spec.t_c,
            "P02",
            "Stage B (+G, Fresh)",
            &run_b_base,
            &b_b,
            true,
            &sample_h,
            &sample_s,
        );

        // ---- Stage C (progressive: G + decay, Fresh, F_gen) ----
        let run_c_base = run_scenario(
            &def_b,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::Fresh,
                decay_constant_per_s: lambda,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::Constant(d_const),
                profile_times_s: profile_times_s.clone(),
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let run_c_fine = run_scenario(
            &def_b_fine,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: REFINE_INTERVALS,
                initial: InitialState::Fresh,
                decay_constant_per_s: lambda,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::Constant(d_const),
                profile_times_s: vec![],
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let b_c = method_b(
            &Needs {
                stage: Stage::Decay,
                source: source_of(&def_b.generation_factor, g_ref, def_b.end_s()),
                decay_per_s: lambda,
                temperature: temperature_of(&def_b.temperature_c),
                kernel_step_start: true,
            },
            Quantity::ReleaseFractionGenerated,
            0.0,
            &r_m,
            &d_const,
            &sample_s,
        );
        let key_c = format!("{t_i}c_stage_c");
        let (fc_a_end, fc_b_end, bc_ok, rc_bnd, _) = sample_and_save_case(
            &prov,
            &out_dir,
            &key_c,
            "Stage C (G + decay)",
            spec.t_c,
            &run_c_base,
            &run_c_fine,
            0.0,
            &sample_h,
            &b_c,
            true,
            "F_gen = M_rel / (M^0 + M_gen)",
            vec![],
            &mut balance_rows,
        )?;
        summary_rows.push(format!(
            "{t_i},{},stage_c,{fc_a_end:.12e},{},{bc_ok},{rc_bnd:.6e}",
            spec.end_h,
            fc_b_end.map_or(String::new(), |v| format!("{v:.12e}"))
        ));
        push_c02_24(
            spec.t_c,
            "P05",
            "Stage C (+G, +lambda, Fresh)",
            &run_c_base,
            &b_c,
            true,
            &sample_h,
            &sample_s,
        );

        // ---- Stage C decay-only (G=0, decay ON, KernelUniform, for heating-test comparison) ----
        let run_cd_base = run_scenario(
            &def_a,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::KernelUniform { c_init: 1.0 },
                decay_constant_per_s: lambda,
                reference_generation_mol_m3_s: 0.0,
                diffusion: Diffusion::Constant(d_const),
                profile_times_s: vec![],
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let run_cd_fine = run_scenario(
            &def_a_fine,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: REFINE_INTERVALS,
                initial: InitialState::KernelUniform { c_init: 1.0 },
                decay_constant_per_s: lambda,
                reference_generation_mol_m3_s: 0.0,
                diffusion: Diffusion::Constant(d_const),
                profile_times_s: vec![],
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let b_cd = method_b(
            &Needs {
                stage: Stage::Decay,
                source: SourceHistory::None,
                decay_per_s: lambda,
                temperature: temperature_of(&def_a.temperature_c),
                kernel_step_start: true,
            },
            Quantity::ReleaseFraction,
            1.0,
            &r_m,
            &d_const,
            &sample_s,
        );
        let key_cd = format!("{t_i}c_stage_c_decay_only");
        let (fcd_a_end, fcd_b_end, bcd_ok, rcd_bnd, _) = sample_and_save_case(
            &prov,
            &out_dir,
            &key_cd,
            "Stage C (decay-only; G=0)",
            spec.t_c,
            &run_cd_base,
            &run_cd_fine,
            0.0,
            &sample_h,
            &b_cd,
            false,
            "F = M_rel / M^0",
            vec![],
            &mut balance_rows,
        )?;
        summary_rows.push(format!(
            "{t_i},{},stage_c_decay_only,{fcd_a_end:.12e},{},{bcd_ok},{rcd_bnd:.6e}",
            spec.end_h,
            fcd_b_end.map_or(String::new(), |v| format!("{v:.12e}"))
        ));
        push_c02_24(
            spec.t_c,
            "P03",
            "Stage C decay-only (G=0, +lambda)",
            &run_cd_base,
            &b_cd,
            false,
            &sample_h,
            &sample_s,
        );

        // ---- Stage D Isothermal (D(T) dynamic Arrhenius mid-step, G=0, decay=0 -> verifies C16=Stage A) ----
        let run_di_base = run_scenario(
            &def_a,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::KernelUniform { c_init: 1.0 },
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: 0.0,
                diffusion: Diffusion::OfTemperature(&dyn_c16),
                profile_times_s: vec![],
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let run_di_fine = run_scenario(
            &def_a_fine,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: REFINE_INTERVALS,
                initial: InitialState::KernelUniform { c_init: 1.0 },
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: 0.0,
                diffusion: Diffusion::OfTemperature(&dyn_c16),
                profile_times_s: vec![],
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let b_di = method_b(
            &Needs {
                stage: Stage::TemperatureDependentDiffusivity,
                source: SourceHistory::None,
                decay_per_s: 0.0,
                temperature: temperature_of(&def_a.temperature_c),
                kernel_step_start: true,
            },
            Quantity::ReleaseFraction,
            1.0,
            &r_m,
            &d_const,
            &sample_s,
        );
        let key_di = format!("{t_i}c_stage_d_isothermal");
        let (fdi_a_end, fdi_b_end, bdi_ok, rdi_bnd, _) = sample_and_save_case(
            &prov,
            &out_dir,
            &key_di,
            "Stage D (isothermal dynamic C16)",
            spec.t_c,
            &run_di_base,
            &run_di_fine,
            0.0,
            &sample_h,
            &b_di,
            false,
            "F = M_rel / M^0",
            vec![],
            &mut balance_rows,
        )?;
        summary_rows.push(format!(
            "{t_i},{},stage_d_isothermal,{fdi_a_end:.12e},{},{bdi_ok},{rdi_bnd:.6e}",
            spec.end_h,
            fdi_b_end.map_or(String::new(), |v| format!("{v:.12e}"))
        ));

        // ---- Stage D Transient Ramp (D(r, T(t)) over heating ramp + hold, G=0, decay OFF & ON) ----
        let dt_hold_s = (spec.end_h * H) / 4000.0;
        let (def_ramp, hold_start_s, hist_label, hist_ids) =
            build_ramp_history(&reg, spec, spec.hold_h, 0.25, dt_hold_s)?;
        let mut def_ramp_fine = def_ramp.clone();
        for seg in &mut def_ramp_fine.plan {
            seg.dt_max_s *= 0.5;
        }
        let ramp_extra_s: Vec<f64> = extra_s.iter().map(|t| hold_start_s + *t).collect();
        let ramp_profile_s: Vec<f64> = profile_times_s.iter().map(|t| hold_start_s + *t).collect();

        // Decay OFF ramp
        let run_dr_off_base = run_scenario(
            &def_ramp,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::KernelUniform { c_init: 1.0 },
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: 0.0,
                diffusion: Diffusion::OfTemperature(&dyn_c16),
                profile_times_s: ramp_profile_s.clone(),
                extra_boundaries_s: ramp_extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let run_dr_off_fine = run_scenario(
            &def_ramp_fine,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: REFINE_INTERVALS,
                initial: InitialState::KernelUniform { c_init: 1.0 },
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: 0.0,
                diffusion: Diffusion::OfTemperature(&dyn_c16),
                profile_times_s: vec![],
                extra_boundaries_s: ramp_extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let b_ramp_na = method_b(
            &Needs {
                stage: Stage::TemperatureDependentDiffusivity,
                source: SourceHistory::None,
                decay_per_s: 0.0,
                temperature: temperature_of(&def_ramp.temperature_c),
                kernel_step_start: true,
            },
            Quantity::ReleaseFraction,
            1.0,
            &r_m,
            &d_const,
            &sample_s,
        );
        let key_dr_off = format!("{t_i}c_stage_d_ramp_decay_off");
        let (fdro_a_end, _, bdro_ok, rdro_bnd, _) = sample_and_save_case(
            &prov,
            &out_dir,
            &key_dr_off,
            "Stage D (heating ramp; decay OFF)",
            spec.t_c,
            &run_dr_off_base,
            &run_dr_off_fine,
            hold_start_s,
            &sample_h,
            &b_ramp_na,
            false,
            "F = M_rel / M^0",
            vec![
                ("history_id", J::s(hist_label.clone())),
                (
                    "history_row_ids",
                    J::Arr(hist_ids.iter().map(J::s).collect()),
                ),
            ],
            &mut balance_rows,
        )?;
        summary_rows.push(format!(
            "{t_i},{},stage_d_ramp_decay_off,{fdro_a_end:.12e},,{bdro_ok},{rdro_bnd:.6e}",
            spec.end_h
        ));

        // Decay ON ramp
        let run_dr_on_base = run_scenario(
            &def_ramp,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::KernelUniform { c_init: 1.0 },
                decay_constant_per_s: lambda,
                reference_generation_mol_m3_s: 0.0,
                diffusion: Diffusion::OfTemperature(&dyn_c16),
                profile_times_s: vec![],
                extra_boundaries_s: ramp_extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let run_dr_on_fine = run_scenario(
            &def_ramp_fine,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: REFINE_INTERVALS,
                initial: InitialState::KernelUniform { c_init: 1.0 },
                decay_constant_per_s: lambda,
                reference_generation_mol_m3_s: 0.0,
                diffusion: Diffusion::OfTemperature(&dyn_c16),
                profile_times_s: vec![],
                extra_boundaries_s: ramp_extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let key_dr_on = format!("{t_i}c_stage_d_ramp_decay_on");
        let (fdron_a_end, _, bdron_ok, rdron_bnd, _) = sample_and_save_case(
            &prov,
            &out_dir,
            &key_dr_on,
            "Stage D (heating ramp; decay ON)",
            spec.t_c,
            &run_dr_on_base,
            &run_dr_on_fine,
            hold_start_s,
            &sample_h,
            &b_ramp_na,
            false,
            "F = M_rel / M^0",
            vec![("history_id", J::s(hist_label.clone()))],
            &mut balance_rows,
        )?;
        summary_rows.push(format!(
            "{t_i},{},stage_d_ramp_decay_on,{fdron_a_end:.12e},,{bdron_ok},{rdron_bnd:.6e}",
            spec.end_h
        ));

        // ---- Stage D Combined transient (G=G_ref, decay ON, D(r, T(t)), Fresh) ----
        let def_comb = ScenarioDefinition {
            id: format!("stage_d_comb_{t_i}c"),
            generation_factor: TimeFunction::constant(1.0).map_err(|e| e.to_string())?,
            ..def_ramp.clone()
        };
        let mut def_comb_fine = def_comb.clone();
        for seg in &mut def_comb_fine.plan {
            seg.dt_max_s *= 0.5;
        }
        let run_dc_base = run_scenario(
            &def_comb,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::Fresh,
                decay_constant_per_s: lambda,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::OfTemperature(&dyn_c16),
                profile_times_s: ramp_profile_s.clone(),
                extra_boundaries_s: ramp_extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let run_dc_fine = run_scenario(
            &def_comb_fine,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: REFINE_INTERVALS,
                initial: InitialState::Fresh,
                decay_constant_per_s: lambda,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::OfTemperature(&dyn_c16),
                profile_times_s: vec![],
                extra_boundaries_s: vec![],
            },
        )
        .map_err(|e| e.to_string())?;
        let key_dc = format!("{t_i}c_stage_d_combined");
        let (fdc_a_end, _, bdc_ok, rdc_bnd, _) = sample_and_save_case(
            &prov,
            &out_dir,
            &key_dc,
            "Stage D (combined: G + decay + D(r; T(t)))",
            spec.t_c,
            &run_dc_base,
            &run_dc_fine,
            hold_start_s,
            &sample_h,
            &b_ramp_na,
            true,
            "F_gen = M_rel / (M^0 + M_gen)",
            vec![("history_id", J::s(hist_label.clone()))],
            &mut balance_rows,
        )?;
        summary_rows.push(format!(
            "{t_i},{},stage_d_combined,{fdc_a_end:.12e},,{bdc_ok},{rdc_bnd:.6e}",
            spec.end_h
        ));

        // ---- P06 (Generation + D(r, T(t)) ramp, decay OFF, Fresh, F_gen) ----
        let run_p06_base = run_scenario(
            &def_comb,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::Fresh,
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::OfTemperature(&dyn_c16),
                profile_times_s: vec![],
                extra_boundaries_s: ramp_extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let run_p06_fine = run_scenario(
            &def_comb_fine,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: REFINE_INTERVALS,
                initial: InitialState::Fresh,
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::OfTemperature(&dyn_c16),
                profile_times_s: vec![],
                extra_boundaries_s: vec![],
            },
        )
        .map_err(|e| e.to_string())?;
        let key_p06 = format!("{t_i}c_p06_gen_temp");
        let (fp06_a_end, _, bp06_ok, rp06_bnd, _) = sample_and_save_case(
            &prov,
            &out_dir,
            &key_p06,
            "P06 (+G; decay OFF; D(r; T(t)); Fresh)",
            spec.t_c,
            &run_p06_base,
            &run_p06_fine,
            hold_start_s,
            &sample_h,
            &b_ramp_na,
            true,
            "F_gen = M_rel / (M^0 + M_gen)",
            vec![("history_id", J::s(hist_label))],
            &mut balance_rows,
        )?;
        summary_rows.push(format!(
            "{t_i},{},p06_gen_temp,{fp06_a_end:.12e},,{bp06_ok},{rp06_bnd:.6e}",
            spec.end_h
        ));

        // ---- Controlled matched-inventory runs (C_init = g_ref * 620.2 EFPD, common F = M_rel/(M^0+M_gen)) ----
        let run_ctrl_gd0_base = run_scenario(
            &def_b,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::KernelUniform {
                    c_init: c_init_ctrl,
                },
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::Constant(d_const),
                profile_times_s: vec![],
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let run_ctrl_gd0_fine = run_scenario(
            &def_b_fine,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: REFINE_INTERVALS,
                initial: InitialState::KernelUniform {
                    c_init: c_init_ctrl,
                },
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::Constant(d_const),
                profile_times_s: vec![],
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let (fct0_a_end, _, _, rct0_bnd, _) = sample_and_save_case(
            &prov,
            &out_dir,
            &format!("{t_i}c_ctrl_gen_on_decay_off"),
            "Controlled (+G matched C_init; decay OFF)",
            spec.t_c,
            &run_ctrl_gd0_base,
            &run_ctrl_gd0_fine,
            0.0,
            &sample_h,
            &b_ramp_na,
            true,
            "F = M_rel / (M^0 + M_gen)",
            vec![("c_init_mol_m3", J::Num(c_init_ctrl))],
            &mut balance_rows,
        )?;
        summary_rows.push(format!(
            "{t_i},{},ctrl_gen_on_decay_off,{fct0_a_end:.12e},,false,{rct0_bnd:.6e}",
            spec.end_h
        ));

        let run_ctrl_gd1_base = run_scenario(
            &def_b,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::KernelUniform {
                    c_init: c_init_ctrl,
                },
                decay_constant_per_s: lambda,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::Constant(d_const),
                profile_times_s: vec![],
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let run_ctrl_gd1_fine = run_scenario(
            &def_b_fine,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: REFINE_INTERVALS,
                initial: InitialState::KernelUniform {
                    c_init: c_init_ctrl,
                },
                decay_constant_per_s: lambda,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::Constant(d_const),
                profile_times_s: vec![],
                extra_boundaries_s: extra_s.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let (fct1_a_end, _, _, rct1_bnd, _) = sample_and_save_case(
            &prov,
            &out_dir,
            &format!("{t_i}c_ctrl_gen_on_decay_on"),
            "Controlled (+G +lambda matched C_init)",
            spec.t_c,
            &run_ctrl_gd1_base,
            &run_ctrl_gd1_fine,
            0.0,
            &sample_h,
            &b_ramp_na,
            true,
            "F = M_rel / (M^0 + M_gen)",
            vec![("c_init_mol_m3", J::Num(c_init_ctrl))],
            &mut balance_rows,
        )?;
        summary_rows.push(format!(
            "{t_i},{},ctrl_gen_on_decay_on,{fct1_a_end:.12e},,false,{rct1_bnd:.6e}",
            spec.end_h
        ));

        // Record radial concentration profiles for 1700 C and 1800 C
        if t_i == 1700 || t_i == 1800 {
            let r_um: Vec<f64> = run_a_base.grid.r.iter().map(|r| *r * 1e6).collect();
            let push_prof = |out: &mut Vec<J>, tag: &str, run: &ScenarioRun, h_off: f64| {
                let mut snaps = Vec::new();
                for (ts, prof) in &run.profiles {
                    snaps.push(J::obj(vec![
                        ("time_h", J::Num((*ts - h_off) / H)),
                        (
                            "c_mol_m3",
                            J::Arr(prof.iter().map(|c| J::Num(*c)).collect()),
                        ),
                    ]));
                }
                out.push(J::obj(vec![
                    ("temperature_c", J::Num(spec.t_c)),
                    ("stage", J::s(tag)),
                    ("r_um", J::Arr(r_um.iter().map(|r| J::Num(*r)).collect())),
                    ("snapshots", J::Arr(snaps)),
                ]));
            };
            push_prof(&mut profile_entries, "stage_a", &run_a_base, 0.0);
            push_prof(&mut profile_entries, "stage_b", &run_b_base, 0.0);
            push_prof(&mut profile_entries, "stage_c", &run_c_base, 0.0);
            push_prof(
                &mut profile_entries,
                "stage_d_ramp_decay_off",
                &run_dr_off_base,
                hold_start_s,
            );
            push_prof(
                &mut profile_entries,
                "stage_d_combined",
                &run_dc_base,
                hold_start_s,
            );
        }

        println!("  completed {t_i} C (Stage A end={fa_end:.4e}, Stage B={fb_a_end:.4e}, Stage C={fc_a_end:.4e}, Stage D ramp={fdro_a_end:.4e})");
    }

    // 2. D06 AGR-1 irradiation temperatures (955, 1028, 1136 C, 620.2 EFPD = 14884.8 h, C15 laws)
    let irr_end_h = 620.2 * 24.0;
    let irr_sample_h = sample_times(5.0, irr_end_h, &[irr_end_h]);
    let irr_sample_s: Vec<f64> = irr_sample_h.iter().map(|x| x * H).collect();
    let mut irr_curves_j = Vec::new();
    for &tc in &[955.0_f64, 1028.0, 1136.0] {
        let t_i = tc.round() as i64;
        let (d_irr, _) = laws_c15.at(tc).map_err(|e| e.to_string())?;
        let def_ia = ScenarioDefinition {
            id: format!("irr_a_{t_i}c"),
            generation_factor: TimeFunction::constant(0.0).map_err(|e| e.to_string())?,
            temperature_c: TimeFunction::constant(tc).map_err(|e| e.to_string())?,
            plan: vec![StepSegment {
                end_s: irr_end_h * H,
                dt_max_s: (irr_end_h * H) / 4000.0,
            }],
            event_times_s: vec![],
        };
        let run_ia = run_scenario(
            &def_ia,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::KernelUniform { c_init: 1.0 },
                decay_constant_per_s: 0.0,
                reference_generation_mol_m3_s: 0.0,
                diffusion: Diffusion::Constant(d_irr),
                profile_times_s: vec![],
                extra_boundaries_s: vec![],
            },
        )
        .map_err(|e| e.to_string())?;
        let def_ic = ScenarioDefinition {
            id: format!("irr_c_{t_i}c"),
            generation_factor: TimeFunction::constant(1.0).map_err(|e| e.to_string())?,
            ..def_ia.clone()
        };
        let run_ic = run_scenario(
            &def_ic,
            &ScenarioConfig {
                radii_m: r_m,
                intervals: BASE_INTERVALS,
                initial: InitialState::Fresh,
                decay_constant_per_s: lambda,
                reference_generation_mol_m3_s: g_ref,
                diffusion: Diffusion::OfTemperature(&dyn_c15),
                profile_times_s: vec![],
                extra_boundaries_s: vec![],
            },
        )
        .map_err(|e| e.to_string())?;

        let b_ia = method_b(
            &Needs::baseline(tc),
            Quantity::ReleaseFraction,
            1.0,
            &r_m,
            &d_irr,
            &irr_sample_s,
        );
        let (_, _, _, b_ia_json) = b_series_json(&b_ia, &irr_sample_h, "F = M_rel / M^0 (Stage A)");
        let tb_a: Vec<f64> = run_ia.rows.iter().map(|r| r.t_s).collect();
        let fa_vals: Vec<f64> = run_ia
            .rows
            .iter()
            .map(|r| r.release_fraction_of_initial.unwrap_or(0.0))
            .collect();
        let tb_c: Vec<f64> = run_ic.rows.iter().map(|r| r.t_s).collect();
        let fc_vals: Vec<f64> = run_ic
            .rows
            .iter()
            .map(|r| r.release_fraction_of_total.unwrap_or(0.0))
            .collect();
        let sa: Vec<f64> = irr_sample_s
            .iter()
            .map(|ts| lin(&tb_a, &fa_vals, *ts).unwrap_or(0.0))
            .collect();
        let sc: Vec<f64> = irr_sample_s
            .iter()
            .map(|ts| lin(&tb_c, &fc_vals, *ts).unwrap_or(0.0))
            .collect();
        irr_curves_j.push(J::obj(vec![
            ("temperature_c", J::Num(tc)),
            ("duration_h", J::Num(irr_end_h)),
            (
                "diffusivities_c15_m2_s",
                J::Arr(d_irr.iter().map(|v| J::Num(*v)).collect()),
            ),
            (
                "time_h",
                J::Arr(irr_sample_h.iter().map(|t| J::Num(*t)).collect()),
            ),
            (
                "stage_a_f_method_a",
                J::Arr(sa.iter().map(|v| J::Num(*v)).collect()),
            ),
            (
                "stage_c_f_gen_method_a",
                J::Arr(sc.iter().map(|v| J::Num(*v)).collect()),
            ),
            ("stage_a_method_b", b_ia_json),
            (
                "stage_a_max_res_over_bound",
                J::Num(run_ia.max_residual_over_bound),
            ),
            (
                "stage_c_max_res_over_bound",
                J::Num(run_ic.max_residual_over_bound),
            ),
        ]));
        println!(
            "  completed D06 irradiation {t_i} C: F_A(end)={:.4e}, F_gen(C/D, end)={:.4e}",
            sa.last().unwrap_or(&0.0),
            sc.last().unwrap_or(&0.0)
        );
    }

    // 3. C01 convergence & conservation diagnostics
    println!("  running C01 spatial/temporal convergence...");
    let spatial = spatial_convergence(&geometry, 6, 2000)?;
    let temporal = temporal_convergence(&geometry, 9, 25, 32)?;
    let c01_json = json_file(
        &prov,
        "study39_c01_convergence",
        vec![
            ("spatial", conv_json_rows(&spatial)),
            ("temporal", conv_json_rows(&temporal)),
            (
                "last_spatial_order",
                J::opt(spatial.last().and_then(|r| r.order)),
            ),
            (
                "last_temporal_order",
                J::opt(temporal.last().and_then(|r| r.order)),
            ),
            ("stage_b_fresh_vs_1e_9", J::Arr(stage_b_check_rows)),
        ],
    );
    write_text(
        &out_dir.join("diagnostics").join("c01_convergence.json"),
        &c01_json,
    )?;

    let c02_json = json_file(
        &prov,
        "study39_c02_method_a_vs_b",
        vec![
            ("comparisons", J::Arr(c02_entries)),
            ("all_24_constant_t", J::Arr(c02_all24_entries)),
        ],
    );
    write_text(
        &out_dir.join("diagnostics").join("c02_method_a_vs_b.json"),
        &c02_json,
    )?;

    let d06_json = json_file(
        &prov,
        "study39_d06_irradiation",
        vec![("temperatures", J::Arr(irr_curves_j))],
    );
    write_text(
        &out_dir.join("diagnostics").join("d06_irradiation.json"),
        &d06_json,
    )?;

    let prof_json = json_file(
        &prov,
        "study39_radial_profiles",
        vec![("profiles", J::Arr(profile_entries))],
    );
    write_text(
        &out_dir
            .join("diagnostics")
            .join("radial_profiles_1700_1800.json"),
        &prof_json,
    )?;

    write_text(
        &out_dir.join("diagnostics").join("mass_balance_audit.csv"),
        &balance_rows.join("\n"),
    )?;
    write_text(
        &out_dir.join("summary_curves.csv"),
        &summary_rows.join("\n"),
    )?;

    println!(
        "study39 finished cleanly in {:.1} s",
        t0.elapsed().as_secs_f64()
    );
    Ok(())
}
