//! The whole pipeline: read the registers and the scenario files, run everything, and produce the
//! file set.

use std::path::{Path, PathBuf};

use strato_compare::compare::run_rows;
use strato_compare::data::{radii, Species};
use strato_compare::json::J;
use strato_compare::model::{Runner, Settings};
use strato_data::register::RegisterTexts;
use strato_data::scenarios::{run_checks, ScenarioTexts, Scenarios};
use strato_data::select::Phase;
use strato_data::Register;

use crate::curves::{
    engine_cell, exp_temperatures, heating_curve, irradiation_curve, stage1, Context, Stage1,
    EXPL_HEATING_C, EXPL_IRRADIATION_C, NUCS,
};
use crate::experiment::{
    bound_json, bounds, candidates, record_json, records, row_nuclide, row_temperature, BoundRec,
    ModelSource, NrmseRecord, SetMeta,
};
use crate::labels::labels_json;
use crate::methodb::{applicability, Needs};
use crate::output::{json_file, masked, Files, Prov, SCHEMA_VERSION};
use crate::params::{generation_nuclides, rate_density, reference_generation, Laws};
use crate::scen;
use crate::verification::{agreement, conservation, convergence, Conservation};

/// What to run.
#[derive(Debug, Clone)]
pub struct Options {
    /// Root of the repository checkout (it holds `data/` and `src/`).
    pub repo: PathBuf,
    /// Commit of the source tree (informational).
    pub commit: String,
    /// Coarse grids and few levels, for tests of the program itself. A quick run is never
    /// committed.
    pub quick: bool,
}

fn read_inputs(data: &Path) -> Result<(RegisterTexts, ScenarioTexts), String> {
    Ok((
        RegisterTexts::read(data).map_err(|e| e.to_string())?,
        ScenarioTexts::read(data).map_err(|e| e.to_string())?,
    ))
}

fn fmt_c(t: f64) -> i64 {
    t.round() as i64
}

/// Build every file.
///
/// # Errors
///
/// A register or scenario file that cannot be read or fails its data checks, or a failed run.
pub fn generate(opts: &Options) -> Result<Files, String> {
    let data_dir = opts.repo.join("data");
    let (texts, scen_texts) = read_inputs(&data_dir)?;
    let reg = Register::from_texts(&texts).map_err(|e| e.to_string())?;
    let sc = Scenarios::parse(&scen_texts).map_err(|e| e.to_string())?;
    let checks = run_checks(&reg, &sc);
    if !checks.all_passed() {
        let bad: Vec<String> = checks
            .results
            .iter()
            .filter(|r| !r.passed)
            .map(|r| r.to_string())
            .collect();
        return Err(format!("the scenario data checks fail: {}", bad.join("; ")));
    }
    let prov = Prov::new(&opts.repo, &opts.commit, &texts, &scen_texts)?;
    let geometry = radii(&reg)?;
    let settings = if opts.quick {
        Settings {
            intervals: [20, 15, 10, 20, 10],
            dt_h: 0.5,
        }
    } else {
        Settings::BASE
    };
    let mut runner = Runner::new(&reg, geometry.clone(), settings)?;
    let clock = std::time::Instant::now();
    let progress = |what: &str| eprintln!("[{:7.1} s] {what}", clock.elapsed().as_secs_f64());
    progress("comparison rows");
    let comparison = run_rows(&reg, &mut runner)?;
    progress("bounds");
    let mut files = Files::new();
    let mut cells: Vec<J> = Vec::new();
    let mut curves_csv = prov.csv(
        "curves of the experimental-comparison and exploration contexts (Method A and Method B)",
        &[
            "Curve key",
            "Time at test temperature (h)",
            "Method A: BTCS release fraction (dimensionless)",
            "Method A resolved (true/false)",
            "Method B: Sturm-Liouville release fraction (dimensionless)",
        ],
    );
    let mut stage1_curves: Vec<Stage1> = Vec::new();
    let mut conservation_entries: Vec<Conservation> = Vec::new();

    // Bound comparisons (statuses for the technical section) and their model values.
    let bound_recs: Vec<BoundRec> = bounds(&reg, &comparison, &mut runner, &geometry)?;

    progress("EXP cells");
    // ---- EXP: the main experimental comparison, per nuclide and supported temperature.
    let mut nrmse_records: Vec<NrmseRecord> = Vec::new();
    for nuc in &NUCS {
        for &t_c in exp_temperatures(nuc.id) {
            // The measurement times of this cell become sample times.
            let mut extra: Vec<f64> = bound_recs
                .iter()
                .filter(|b| b.nuc.id == nuc.id && b.t_c == t_c)
                .flat_map(|b| [b.t_h, b.t_start_h])
                .collect();
            extra.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            let curve = heating_curve(&mut runner, nuc, t_c)?.clone();
            let labels = vec![
                "numerical solution of the stated model; not experimental validation".to_string(),
                "kernel-only start (decision 20); constant diffusivity per layer at the test temperature from the first hour (heating ramps are not modelled in stage 1)".to_string(),
                "published heating-test rows C16 as printed (two terms where printed); the rows print no validity range; correlation form under source review".to_string(),
            ];
            let (json, s1) = stage1(
                &prov,
                Context::Exp,
                nuc,
                t_c,
                &curve,
                &geometry,
                Phase::HeatingTest,
                &extra,
                labels,
            )?;
            files.insert(format!("curves/{}.json", s1.key), json);
            for r in &s1.csv_rows {
                curves_csv.row(r.clone());
            }
            conservation_entries.push(Conservation {
                key: s1.key.clone(),
                residual_over_bound: if curve.worst_balance_bound > 0.0 {
                    curve.worst_balance_residual / curve.worst_balance_bound
                } else {
                    0.0
                },
                relative_residual: curve.worst_balance_residual,
                steps: curve.t_h.len().saturating_sub(1),
            });
            let ds = format!("agr1.{}.{}c", nuc.id, fmt_c(t_c));
            let b_ok = s1.b_floor.is_some();
            cells.push(cell_json(
                &s1.key,
                1,
                "exp",
                nuc.name,
                t_c,
                None,
                J::obj(vec![("available", J::Bool(true))]),
                if b_ok {
                    J::obj(vec![("available", J::Bool(true))])
                } else {
                    J::obj(vec![("available", J::Bool(false))])
                },
                &ds,
                &bound_recs,
                nuc.id,
                t_c,
                vec!["release_fraction"],
                None,
            ));
            stage1_curves.push(s1);
            let mut ramp_curve = None;
            for stage in [3u8, 4u8] {
                let (json, ec) =
                    engine_cell(&prov, &reg, nuc, t_c, stage, &geometry, opts.quick, &extra)?;
                if stage == 4 {
                    ramp_curve = Some(ec.clone());
                }
                files.insert(format!("curves/{}.json", ec.key), json);
                for r in &ec.csv_rows {
                    curves_csv.row(r.clone());
                }
                conservation_entries.push(Conservation {
                    key: ec.key.clone(),
                    residual_over_bound: ec.max_residual_over_bound,
                    relative_residual: ec.max_relative_residual,
                    steps: ec.steps,
                });
                let needs = Needs {
                    generation: false,
                    decay: stage == 3,
                    temperature_varies: stage == 4,
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
                cells.push(cell_json(
                    &ec.key,
                    stage,
                    "exp",
                    nuc.name,
                    t_c,
                    None,
                    J::obj(vec![("available", J::Bool(true))]),
                    b,
                    &ds,
                    &bound_recs,
                    nuc.id,
                    t_c,
                    if stage == 3 {
                        vec![
                            "release_fraction",
                            "remaining_inventory_fraction",
                            "decayed_fraction",
                        ]
                    } else {
                        vec!["release_fraction", "temperature_history"]
                    },
                    Some(if stage == 3 {
                        "Decayed amount as a fraction of the initial inventory"
                    } else {
                        "Temperature history T(t)"
                    }),
                ));
            }
            // The unsupported stages of this context, with their reasons.
            for stage in [2u8, 5u8] {
                cells.push(J::obj(vec![
                    ("stage", J::Int(stage as i64)),
                    ("context", J::s("exp")),
                    ("nuclide", J::s(nuc.name)),
                    ("temperature_c", J::Num(t_c)),
                    ("supported", J::Bool(false)),
                    (
                        "reason",
                        J::s("a post-irradiation heating test has no fission source (G = 0); generation belongs to the scenarios"),
                    ),
                ]));
            }

            // NRMSE records: stage 1 (isothermal, Methods A and B) and stage 4 (ramp, Method A).
            let cands = candidates(
                &comparison,
                &mut runner,
                nuc,
                t_c,
                ModelSource::Isothermal { radii: &geometry },
            )?;
            let ramp = ramp_curve.as_ref().ok_or("no stage 4 curve")?;
            let cands4 = candidates(&comparison, &mut runner, nuc, t_c, ModelSource::Ramp(ramp))?;
            let m1 = SetMeta {
                set_id: format!("nrmse.s1.exp.{}.{}c", nuc.id, fmt_c(t_c)),
                nuclide: nuc.name.to_string(),
                temperature_c: t_c,
                model_config_id: format!("s1.exp.{}.{}c", nuc.id, fmt_c(t_c)),
            };
            for r in records(&m1, &cands, &Ok(())) {
                nrmse_records.push(r);
            }
            let m4 = SetMeta {
                set_id: format!("nrmse.s4.exp.{}.{}c", nuc.id, fmt_c(t_c)),
                nuclide: nuc.name.to_string(),
                temperature_c: t_c,
                model_config_id: format!("s4.exp.{}.{}c", nuc.id, fmt_c(t_c)),
            };
            let na = applicability(Needs {
                generation: false,
                decay: false,
                temperature_varies: true,
                kernel_step_start: true,
            });
            for r in records(&m4, &cands4, &na) {
                nrmse_records.push(r);
            }
        }
    }

    progress("EXPL cells");
    // ---- EXPL: model exploration at the registered temperatures.
    for nuc in &NUCS {
        for &t_c in EXPL_IRRADIATION_C.iter().chain(EXPL_HEATING_C.iter()) {
            let irradiation = EXPL_IRRADIATION_C.contains(&t_c) && !EXPL_HEATING_C.contains(&t_c);
            progress(&format!("EXPL {} {} degC", nuc.id, t_c));
            let (phase, curve) = if irradiation {
                (
                    Phase::Irradiation,
                    irradiation_curve(&reg, nuc, t_c, &geometry, opts.quick)?,
                )
            } else {
                (
                    Phase::HeatingTest,
                    heating_curve(&mut runner, nuc, t_c)?.clone(),
                )
            };
            let labels = vec![
                "model exploration: nothing is claimed about experiment".to_string(),
                "numerical solution of the stated model; not experimental validation".to_string(),
                format!(
                    "{} correlations of the register as printed; the rows print no validity range",
                    phase.as_str()
                ),
            ];
            let (json, s1) = stage1(
                &prov,
                Context::Expl,
                nuc,
                t_c,
                &curve,
                &geometry,
                phase,
                &[],
                labels,
            )?;
            files.insert(format!("curves/{}.json", s1.key), json);
            for r in &s1.csv_rows {
                curves_csv.row(r.clone());
            }
            cells.push(J::obj(vec![
                ("key", J::s(s1.key.clone())),
                ("stage", J::Int(1)),
                ("context", J::s("expl")),
                ("nuclide", J::s(nuc.name)),
                ("temperature_c", J::Num(t_c)),
                ("supported", J::Bool(true)),
                ("method_a", J::obj(vec![("available", J::Bool(true))])),
                (
                    "method_b",
                    J::obj(vec![("available", J::Bool(s1.b_floor.is_some()))]),
                ),
                (
                    "experiment",
                    J::obj(vec![
                        ("available", J::Bool(false)),
                        (
                            "replacement",
                            J::s("Diffusivity of each layer at the temperature (m\u{b2} s\u{207b}\u{b9}) as a labelled table"),
                        ),
                    ]),
                ),
                ("curve_file", J::s(format!("curves/{}.json", s1.key))),
                ("phase_of_correlations", J::s(phase.as_str())),
                ("default_y_scale", J::s("log")),
            ]));
            stage1_curves.push(s1);
        }
    }

    progress("scenarios");
    // ---- Scenarios.
    let radii_m = geometry.outer_m;
    let outcomes = scen::all(&prov, &reg, &sc, radii_m, opts.quick, &mut files)?;
    let mut scen_csv = prov.csv(
        "scenario runs (decimated rows of Method A)",
        &[
            "Curve key",
            "Time (h)",
            "Time (d)",
            "Plotted release fraction (dimensionless)",
            "Resolved (true/false)",
            "Inventory in one particle (mol)",
            "Generated amount (mol)",
            "Released amount (mol)",
            "Decayed amount (mol)",
            "Particle temperature (degC)",
            "Generation factor (dimensionless)",
            "Balance residual (mol)",
            "Rounding bound of the balance residual (mol)",
        ],
    );
    for o in &outcomes {
        for r in &o.csv_rows {
            scen_csv.row(r.clone());
        }
        conservation_entries.push(Conservation {
            key: o.key.clone(),
            residual_over_bound: o.max_residual_over_bound,
            relative_residual: o.max_relative_residual,
            steps: o.steps,
        });
        cells.push(J::obj(vec![
            ("key", J::s(o.key.clone())),
            ("context", J::s("scenario")),
            ("supported", J::Bool(true)),
            ("curve_file", J::s(format!("curves/{}.json", o.key))),
            ("steps", J::Int(o.steps as i64)),
            (
                "resolved_fraction_of_drawable_points",
                J::opt(o.resolved_fraction),
            ),
        ]));
    }
    for (nuc, why) in [
        ("Ag-110m", "no direct fission-yield source term (decision 15)"),
        ("Cs-134", "no direct fission-yield source term (decision 15)"),
        ("Eu-154", "no direct fission-yield source term (decision 15) and no diffusivity (DATA UNAVAILABLE)"),
    ] {
        cells.push(J::obj(vec![
            ("context", J::s("scenario")),
            ("nuclide", J::s(nuc)),
            ("supported", J::Bool(false)),
            ("reason", J::s(why)),
        ]));
    }

    progress("experiment files");
    // ---- Experiment files, bounds, NRMSE.
    let mut obs_csv = prov.csv(
        "experimental bound comparisons (statuses for the technical section)",
        &[
            "Row id",
            "Kind",
            "Nuclide",
            "Test temperature (degC)",
            "Compact",
            "Time at test temperature (h)",
            "Reported value (dimensionless)",
            "Bound lower reading end",
            "Bound upper reading end",
            "Method A model value",
            "Method B model value",
            "Ratio model A over bound",
            "Status",
            "Reason",
            "Status robust to reading uncertainty",
            "Value class",
            "Drawn on graph",
        ],
    );
    for b in &bound_recs {
        obs_csv.row(vec![
            b.id.clone(),
            b.kind.to_string(),
            b.nuc.name.to_string(),
            format!("{}", b.t_c),
            b.compact.clone(),
            format!("{:e}", b.t_h),
            format!("{:e}", b.bound),
            format!("{:e}", b.interval.0),
            format!("{:e}", b.interval.1),
            b.model_a.map_or(String::new(), |v| format!("{v:e}")),
            b.model_b.map_or(String::new(), |v| format!("{v:e}")),
            b.model_a
                .map_or(String::new(), |v| format!("{:e}", v / b.bound)),
            b.status.clone(),
            b.reason.unwrap_or("").to_string(),
            b.robust.map_or(String::new(), |r| r.to_string()),
            b.value_class.clone(),
            (b.drawn && b.kind != "window_bound").to_string(),
        ]);
    }
    for nuc in &NUCS {
        for &t_c in exp_temperatures(nuc.id) {
            let ds = format!("agr1.{}.{}c", nuc.id, fmt_c(t_c));
            let mine: Vec<&BoundRec> = bound_recs
                .iter()
                .filter(|b| b.nuc.id == nuc.id && b.t_c == t_c)
                .collect();
            let observations: Vec<J> = mine
                .iter()
                .filter(|b| b.kind != "window_bound")
                .map(|b| bound_json(b))
                .collect();
            let windows: Vec<J> = mine
                .iter()
                .filter(|b| b.kind == "window_bound")
                .map(|b| bound_json(b))
                .collect();
            // Point rows (none today).
            let points = comparison
                .iter()
                .filter(|r| {
                    row_nuclide(&r.row).map(|n| n.id) == Some(nuc.id)
                        && row_temperature(&r.row) == Some(t_c)
                        && matches!(r.eval, strato_compare::compare::Eval::Point { .. })
                })
                .count();
            files.insert(
                format!("experiment/{ds}.json"),
                json_file(
                    &prov,
                    "experimental observations of one dataset",
                    vec![
                        ("dataset_id", J::s(ds.clone())),
                        ("nuclide", J::s(nuc.name)),
                        ("temperature_c", J::Num(t_c)),
                        (
                            "quantity",
                            J::s("cumulative release fraction (dimensionless)"),
                        ),
                        (
                            "register",
                            J::s("data/experimental/release_measurements.csv"),
                        ),
                        (
                            "markers",
                            J::s("every release record is a set of discrete markers: bound markers (black downward triangles) at the bound value with no deviation and no status word; point markers only where a point comparison exists; no experimental line is drawn"),
                        ),
                        ("point_comparisons", J::Int(points as i64)),
                        ("connect", J::Bool(false)),
                        ("observations", J::Arr(observations)),
                        ("window_bounds", J::Arr(windows)),
                    ],
                ),
            );
        }
    }
    files.insert(
        "bounds.json".into(),
        json_file(
            &prov,
            "bound comparisons and their statuses (technical section)",
            vec![
                (
                    "statement",
                    J::s("An exceeded bound stays visible (decision 23). A bound comparison is neither an NRMSE point nor a validation: the register total includes inventory outside the SiC, so the model is expected not to exceed the bound only if no more than the compact inventory is released (science design Section 1.4)."),
                ),
                (
                    "status_texts",
                    J::s("WITHIN REPORTED UPPER BOUND, EXCEEDS REPORTED UPPER BOUND, NOT ASSESSED (reason model_not_resolved)"),
                ),
                ("bounds", J::Arr(bound_recs.iter().map(bound_json).collect())),
            ],
        ),
    );
    let mut nrmse_csv = prov.csv(
        "NRMSE records (metric-nrmse.md Section 7); no value exists while N = 0",
        &[
            "Set id",
            "Nuclide",
            "Test temperature (degC)",
            "Method",
            "N",
            "F_scale (dimensionless)",
            "F_scale definition id",
            "NRMSE (percent)",
            "Display",
            "Empty reason",
            "Method applicable",
            "Rows used",
            "Rows excluded (row id:reason)",
        ],
    );
    for r in &nrmse_records {
        nrmse_csv.row(vec![
            format!("{}.{}", r.meta.set_id, r.method),
            r.meta.nuclide.clone(),
            format!("{}", r.meta.temperature_c),
            r.method.to_string(),
            r.n.to_string(),
            r.value
                .as_ref()
                .map_or(String::new(), |v| format!("{:e}", v.f_scale)),
            crate::nrmse::FSCALE_DEFINITION_ID.to_string(),
            r.value
                .as_ref()
                .map_or(String::new(), |v| format!("{:e}", v.nrmse_percent)),
            r.display.clone().unwrap_or_default(),
            r.empty_reason
                .map_or(String::new(), |e| e.code().to_string()),
            r.method_applicable.to_string(),
            r.rows_used.join(" "),
            r.rows_excluded.join(" "),
        ]);
    }
    let with_value = nrmse_records.iter().filter(|r| r.value.is_some()).count();
    files.insert(
        "nrmse.json".into(),
        json_file(
            &prov,
            "NRMSE records (the experimental comparison metric of decisions 64 and 65)",
            vec![
                (
                    "definition",
                    J::obj(vec![
                        ("id", J::s(crate::nrmse::FSCALE_DEFINITION_ID)),
                        (
                            "formula",
                            J::s("NRMSE(%) = 100 sqrt((1/N) sum_i (F_model,i - F_exp,i)^2) / F_scale, F_scale = max_i F_exp,i over the compatible points of the set"),
                        ),
                        ("document", J::s("metric-nrmse.md")),
                    ]),
                ),
                (
                    "present_state",
                    J::s(format!(
                        "{with_value} of {} records have a value. The registers hold no compatible point set today (science design Section 1.3: the AGR-1 heating-test totals and time series are upper bounds, not point comparisons), so N = 0 and no NRMSE value is produced; none is formed by relaxing a rule. The record names every excluded row and its reason.",
                        nrmse_records.len()
                    )),
                ),
                (
                    "difference_from_the_web_specification",
                    J::s("deviations.json of web-v3-spec Section 4.2 is replaced by this file (nrmse.json): science design Section 7.3 and metric-nrmse.md Section 7 define the NRMSE record, and the spec's delta_exp definition is superseded by decision 64."),
                ),
                ("records", J::Arr(nrmse_records.iter().map(record_json).collect())),
            ],
        ),
    );

    // ---- Parameters.
    files.insert(
        "parameters.json".into(),
        parameters_json(&prov, &reg, &geometry)?,
    );

    progress("verification");
    // ---- Verification.
    let (conv_json, conv_csv) = convergence(&prov, &geometry, opts.quick)?;
    files.insert("verification/convergence.json".into(), conv_json);
    let (agr_json, agr_csv_rows) = agreement(&prov, &stage1_curves);
    files.insert("verification/agreement.json".into(), agr_json);
    let (cons_json, cons_csv_rows) = conservation(&prov, &conservation_entries);
    files.insert("verification/conservation.json".into(), cons_json);
    let mut c1 = prov.csv(
        "observed convergence orders (synthetic homogeneous case against the exact series)",
        &[
            "Study",
            "Refinement level",
            "Step (m or s)",
            "Count",
            "Absolute error (dimensionless)",
            "Observed order",
        ],
    );
    for r in conv_csv {
        c1.row(r);
    }
    let mut c2 = prov.csv(
        "solver-to-solver agreement of Method A and Method B",
        &[
            "Curve key",
            "Points compared",
            "Largest relative difference",
            "Final relative difference",
            "Method B accuracy floor",
            "Method B first valid time (h)",
        ],
    );
    for r in agr_csv_rows {
        c2.row(r);
    }
    let mut c3 = prov.csv(
        "balance residuals against their derived bounds",
        &[
            "Curve key",
            "Steps",
            "Largest absolute residual over its bound",
            "Largest relative residual",
        ],
    );
    for r in cons_csv_rows {
        c3.row(r);
    }

    // ---- Labels and support.
    files.insert("labels.json".into(), labels_json(&prov, &nrmse_records));
    files.insert("support.json".into(), support_json(&prov, &reg, cells)?);
    files.insert("csv/curves.csv".into(), curves_csv.finish());
    files.insert("csv/scenario_series.csv".into(), scen_csv.finish());
    files.insert("csv/bounds.csv".into(), obs_csv.finish());
    files.insert("csv/nrmse.csv".into(), nrmse_csv.finish());
    files.insert("csv/convergence.csv".into(), c1.finish());
    files.insert("csv/agreement.csv".into(), c2.finish());
    files.insert("csv/conservation.csv".into(), c3.finish());
    files.insert(
        "csv/scenario_histories.csv".into(),
        histories_csv(&prov, &sc),
    );

    // ---- Manifest, last: it lists every other file with the hash of its content apart from the
    // commit lines.
    let mut listed: Vec<J> = Vec::new();
    for (name, text) in &files {
        listed.push(J::obj(vec![
            ("path", J::s(name.clone())),
            (
                "sha256_without_commit_lines",
                J::s(strato::postprocessing::export::sha256_hex(
                    masked(text).as_bytes(),
                )),
            ),
            // Counted without the commit lines, like the hash, so that the manifest does not
            // depend on the commit the outputs are generated with.
            (
                "bytes_without_commit_lines",
                J::Int(masked(text).len() as i64),
            ),
        ]));
    }
    files.insert(
        "manifest.json".into(),
        json_file(
            &prov,
            "manifest of the shared results",
            vec![
                ("schema", J::s(SCHEMA_VERSION)),
                ("quick_run", J::Bool(opts.quick)),
                ("files", J::Arr(listed)),
            ],
        ),
    );
    Ok(files)
}

#[allow(clippy::too_many_arguments)]
fn cell_json(
    key: &str,
    stage: u8,
    context: &str,
    nuclide: &str,
    t_c: f64,
    scenario: Option<&str>,
    method_a: J,
    method_b: J,
    dataset: &str,
    bounds: &[BoundRec],
    nuc_id: &str,
    t: f64,
    quantities: Vec<&str>,
    replacement: Option<&str>,
) -> J {
    let rows: Vec<J> = bounds
        .iter()
        .filter(|b| b.nuc.id == nuc_id && b.t_c == t && b.kind != "window_bound")
        .map(|b| J::s(b.id.clone()))
        .collect();
    J::obj(vec![
        ("key", J::s(key)),
        ("stage", J::Int(stage as i64)),
        ("context", J::s(context)),
        ("scenario", scenario.map_or(J::Null, J::s)),
        ("nuclide", J::s(nuclide)),
        ("temperature_c", J::Num(t_c)),
        ("supported", J::Bool(true)),
        ("method_a", method_a),
        ("method_b", method_b),
        (
            "experiment",
            J::obj(vec![
                ("available", J::Bool(!rows.is_empty())),
                (
                    "kind",
                    J::s("bound markers only (no point comparison exists)"),
                ),
                ("dataset_ids", J::Arr(vec![J::s(dataset)])),
                ("row_ids", J::Arr(rows)),
                ("nrmse_label", J::s("none: no value exists while N = 0")),
            ]),
        ),
        ("curve_file", J::s(format!("curves/{key}.json"))),
        (
            "quantities",
            J::Arr(quantities.into_iter().map(J::s).collect()),
        ),
        ("replacement_quantity", replacement.map_or(J::Null, J::s)),
        ("default_y_scale", J::s("log")),
    ])
}

fn support_json(prov: &Prov, reg: &Register, cells: Vec<J>) -> Result<String, String> {
    let mut exploration = Vec::new();
    for nuc in &NUCS {
        let mut phases = Vec::new();
        for phase in [Phase::Irradiation, Phase::HeatingTest] {
            let laws = Laws::new(reg, nuc.species.label(), phase)?;
            let temps: &[f64] = if phase == Phase::Irradiation {
                &EXPL_IRRADIATION_C
            } else {
                &EXPL_HEATING_C
            };
            phases.push(J::obj(vec![
                ("phase", J::s(phase.as_str())),
                (
                    "temperatures_c",
                    J::Arr(temps.iter().map(|t| J::Num(*t)).collect()),
                ),
                (
                    "layers",
                    J::Arr(
                        laws.sel
                            .iter()
                            .zip(crate::params::LAYERS)
                            .map(|(s, (name, _))| {
                                J::obj(vec![
                                    ("layer", J::s(name)),
                                    (
                                        "register_rows",
                                        J::Arr(s.row_ids().iter().map(|i| J::s(*i)).collect()),
                                    ),
                                    (
                                        "published_range_c",
                                        match s.published_range_c {
                                            Some((a, b)) => J::Arr(vec![J::Num(a), J::Num(b)]),
                                            None => J::Null,
                                        },
                                    ),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ]));
        }
        exploration.push(J::obj(vec![
            ("nuclide", J::s(nuc.name)),
            ("phases", J::Arr(phases)),
        ]));
    }
    Ok(json_file(
        prov,
        "support matrix: every selector option and every reason why not",
        vec![
            (
                "stages",
                J::Arr(
                    [
                        (1, "baseline spherical diffusion"),
                        (2, "generation"),
                        (3, "radioactive decay"),
                        (4, "temperature-dependent diffusivity"),
                        (5, "combined"),
                    ]
                    .iter()
                    .map(|(n, name)| {
                        J::obj(vec![
                            ("stage", J::Int(*n)),
                            ("name", J::s(*name)),
                            ("computed", J::Bool(true)),
                        ])
                    })
                    .collect(),
                ),
            ),
            (
                "scenarios",
                J::Arr(
                    scen::SLUGS
                        .iter()
                        .map(|(id, slug)| {
                            J::obj(vec![
                                ("id", J::s(*slug)),
                                ("definition", J::s(*id)),
                                ("supported", J::Bool(true)),
                                ("min_stage", J::Int(if *id == "S1" { 2 } else { 5 })),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "main_comparison_temperatures_c",
                J::obj(
                    NUCS.iter()
                        .map(|n| {
                            (
                                n.name,
                                J::Arr(exp_temperatures(n.id).iter().map(|t| J::Num(*t)).collect()),
                            )
                        })
                        .collect(),
                ),
            ),
            ("cells", J::Arr(cells)),
            ("exploration", J::Arr(exploration)),
            (
                "method_b_note",
                J::s("Method B is exact for the baseline and for a constant temperature; the extensions for decay and for a kernel source are pending (not available: extension pending); it is never shown when the temperature changes in time"),
            ),
        ],
    ))
}

fn parameters_json(
    prov: &Prov,
    reg: &Register,
    geometry: &strato_compare::data::Radii,
) -> Result<String, String> {
    let mut species = Vec::new();
    for s in Species::ALL {
        let heating = Laws::new(reg, s.label(), Phase::HeatingTest)?;
        let irradiation = Laws::new(reg, s.label(), Phase::Irradiation)?;
        species.push(J::obj(vec![
            ("element", J::s(s.label())),
            ("heating_test_rows", heating.json(&[1600.0, 1700.0, 1800.0])),
            (
                "irradiation_rows",
                irradiation.json(&[955.0, 1028.0, 1136.0, 1197.0]),
            ),
        ]));
    }
    let nuclides = generation_nuclides(reg)?;
    let mut nuc_json = Vec::new();
    for target in ["capsule:3", "compact:6-2-1"] {
        let rate = rate_density(reg, target)?;
        let mut per = Vec::new();
        for n in &nuclides {
            per.push(J::obj(vec![
                ("nuclide", J::s(n.name)),
                ("decay_constant_per_s", J::Num(n.lambda_per_s)),
                ("cumulative_yield_per_fission", J::Num(n.yield_per_fission)),
                (
                    "reference_generation_mol_m3_s",
                    J::Num(reference_generation(n.yield_per_fission, rate.value)),
                ),
            ]));
        }
        nuc_json.push(J::obj(vec![
            ("fission_rate_density_target", J::s(target)),
            ("fission_rate_density_per_m3_s", J::Num(rate.value)),
            ("label", J::s(rate.label.clone())),
            (
                "source_rows",
                J::Arr(rate.source_ids.iter().map(J::s).collect()),
            ),
            ("nuclides", J::Arr(per)),
        ]));
    }
    Ok(json_file(
        prov,
        "materials, coefficients and inputs with class and source row",
        vec![
            (
                "geometry",
                J::obj(vec![
                    ("design", J::s(strato_compare::data::GEOMETRY_DESIGN)),
                    ("outer_radii_m", J::Arr(geometry.outer_m.iter().map(|r| J::Num(*r)).collect())),
                    ("register_rows", J::Arr(geometry.row_ids.iter().map(J::s).collect())),
                    ("layers", J::strs(&["kernel", "buffer", "IPyC", "SiC", "OPyC"])),
                ]),
            ),
            ("diffusivity", J::Arr(species)),
            ("generation", J::Arr(nuc_json)),
            (
                "notes",
                J::s("Every coefficient carries its register row, class and phase of use (owner ruling 16); the kernel rows are UO2 surrogates (no UCO measurement exists, decision 40); rows print no validity range; the fission-rate density is derived from published burnup and is calculated, never measured (decision 42); the Pu-239 share of fission is not represented."),
            ),
        ],
    ))
}

fn histories_csv(prov: &Prov, sc: &Scenarios) -> String {
    let mut csv = prov.csv(
        "scenario histories (prescribed inputs) with the source of every number",
        &[
            "Scenario",
            "Variant",
            "Row id",
            "Record",
            "Quantity",
            "Time",
            "Time unit",
            "Value",
            "Unit",
            "Interpolation",
            "Source row ids",
            "Basis",
            "Reason or statement",
        ],
    );
    for f in &sc.files {
        for r in &f.rows {
            csv.row(vec![
                f.id.clone(),
                r.variant.clone(),
                r.row_id.clone(),
                format!("{:?}", r.record).to_lowercase(),
                r.quantity.clone(),
                r.t_value.as_ref().map_or(String::new(), |t| t.text.clone()),
                r.t_unit.clone(),
                r.value.as_ref().map_or(String::new(), |t| t.text.clone()),
                r.unit.clone(),
                r.interpolation.clone(),
                r.source_ids.join(";"),
                r.basis.clone(),
                r.text.clone(),
            ]);
        }
    }
    csv.finish()
}
