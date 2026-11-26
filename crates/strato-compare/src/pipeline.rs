//! The whole comparison: read the registers, classify and run every row, run the studies, write
//! the files, draw the figures and render the report.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use strato_data::register::RegisterTexts;
use strato_data::Register;

use crate::compare::{datasets, run_rows, Dataset, RowRecord};
use crate::data::{default_set, evaluate, radii, Radii, Species};
use crate::figures::{self, Inputs, TEMPERATURES};
use crate::json::J;
use crate::model::{Curve, Runner, Settings};
use crate::outputs::{self, Prov};
use crate::report::{self, ReportInput};
use crate::studies::{
    analytical_series, homogeneous_case_description, profiles, sensitivity, spatial_convergence,
    spatial_floor, temporal_convergence, AnalyticSeries, ConvergenceRow, ProfileRow,
    SensitivityRow,
};

/// What to generate and where to put it.
#[derive(Debug, Clone)]
pub struct Options {
    /// Root of the repository checkout (it holds `data/`).
    pub repo: PathBuf,
    /// Folder for the CSV and JSON files.
    pub data_out: PathBuf,
    /// Folder for the SVG figures.
    pub figures_out: PathBuf,
    /// File for the report, if it is to be written.
    pub report_out: Option<PathBuf>,
    /// Commit of the source tree, recorded in every file.
    pub commit: String,
    /// Coarser settings and fewer levels, for tests.
    pub quick: bool,
}

/// Everything the run produced, in memory.
pub struct Outcome {
    /// Every row with its status and model comparison.
    pub records: Vec<RowRecord>,
    /// The aggregates per dataset.
    pub datasets: Vec<Dataset>,
    /// Sensitivity rows.
    pub sensitivity: Vec<SensitivityRow>,
    /// Spatial convergence.
    pub spatial: Vec<ConvergenceRow>,
    /// Temporal convergence.
    pub temporal: Vec<ConvergenceRow>,
}

/// The sample times of the independent analytical series, hours.
pub const ANALYTIC_TIMES_H: [f64; 11] = [
    1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 150.0, 200.0, 300.0, 400.0,
];

fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::write(path, text.as_bytes()).map_err(|e| format!("{}: {e}", path.display()))
}

fn solver_version() -> Result<String, String> {
    // The version of the solver package, read through its own provenance record.
    use strato::postprocessing::export::Provenance;
    let reg_radii = Radii {
        outer_m: [1.0e-4, 2.0e-4, 3.0e-4, 4.0e-4, 5.0e-4],
        row_ids: vec![],
    };
    let cfg = crate::model::config(
        "version probe",
        "probe",
        1000.0,
        &reg_radii,
        &[1e-12; 5],
        Settings::BASE,
        1.0,
        1.0,
    );
    let input = strato::numerical::btcs::validate_input(
        &cfg,
        &strato::numerical::btcs::Limits {
            max_intervals_total: 1_000_000,
            max_steps: 10_000_000,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(Provenance::new(&cfg, &input, "", "unknown").package_version)
}

/// Run everything.
pub fn generate(opts: &Options) -> Result<Outcome, String> {
    let data_dir = opts.repo.join("data");
    let texts = RegisterTexts::read(&data_dir).map_err(|e| e.to_string())?;
    let reg = Register::from_texts(&texts).map_err(|e| e.to_string())?;
    let geometry = radii(&reg)?;
    let settings = if opts.quick {
        Settings {
            intervals: [20, 15, 10, 20, 10],
            dt_h: 0.25,
        }
    } else {
        Settings::BASE
    };

    let mut set_notes = Vec::new();
    let mut regions: Vec<J> = Vec::new();
    for s in Species::ALL {
        let d = default_set(&reg, s)?;
        let rows: Vec<String> = d
            .ids
            .iter()
            .zip(["kernel", "buffer", "IPyC", "SiC", "OPyC"])
            .map(|(ids, name)| format!("{name}: {}", ids.join("+")))
            .collect();
        for ((ids, (class, phase)), name) in d
            .ids
            .iter()
            .zip(d.classes.iter())
            .zip(["kernel", "buffer", "IPyC", "SiC", "OPyC"])
        {
            regions.push(J::obj(vec![
                ("element", J::s(s.label())),
                ("region", J::s(name)),
                ("register_rows", J::Arr(ids.iter().map(J::s).collect())),
                ("coefficient_class", J::s(class)),
                ("phase_of_use", J::s(phase)),
            ]));
        }
        let classes: Vec<String> = d
            .classes
            .iter()
            .zip(["kernel", "buffer", "IPyC", "SiC", "OPyC"])
            .map(|((class, phase), name)| format!("{name}: {class}, phase {phase}"))
            .collect();
        set_notes.push((
            format!("{} ({})", d.id, s.label()),
            format!(
                "{} - {}: {}. Class of each coefficient (owner ruling 16) and phase of use: {}. Statements of the selection function: {}",
                s.label(),
                d.label,
                rows.join("; "),
                classes.join("; "),
                d.labels.join("; ")
            ),
        ));
    }
    let parameter_source = format!(
        "geometry design {} (register rows {}); diffusion coefficients chosen by the strato-data selection function (decision 47) for the heating-test phase, that is the rows of register sets C16-T2 and C16-BUF as printed, evaluated at the test temperature (the kernel rows are UO2 surrogates: no UCO measurement exists, decision 40); the class of each coefficient (owner ruling 16) and its phase of use are in the sensitivity file and in the coefficient_sets of comparison.json",
        crate::data::GEOMETRY_DESIGN,
        geometry.row_ids.join(" ")
    );
    let version = solver_version()?;
    let prov = Prov::new(&opts.commit, &version, &texts, parameter_source);

    // Rows.
    let mut runner = Runner::new(&reg, geometry.clone(), settings)?;
    let records = run_rows(&reg, &mut runner)?;
    let sets = datasets(&records);

    // Default curves for every species and test temperature.
    for s in Species::ALL {
        for t in TEMPERATURES {
            runner.get(s, t, 400.0)?;
        }
    }
    let curves: Vec<(Species, f64, &Curve, String)> = Species::ALL
        .iter()
        .flat_map(|s| {
            TEMPERATURES.iter().filter_map(|t| {
                runner
                    .lookup(*s, *t)
                    .map(|c| (*s, *t, c, runner.sets[s].id.clone()))
            })
        })
        .collect();
    let mut analytic: AnalyticSeries = BTreeMap::new();
    let mut analytic_floor: BTreeMap<(Species, i64), f64> = BTreeMap::new();
    for (s, t, c, _) in &curves {
        let r = analytical_series(&geometry, &c.d_m2_s, &ANALYTIC_TIMES_H);
        let key = (*s, (*t * 10.0).round() as i64);
        if let Some(f) = r.accuracy_floor {
            analytic_floor.insert(key, f);
        }
        analytic.insert(
            key,
            ANALYTIC_TIMES_H.iter().copied().zip(r.values).collect(),
        );
    }

    // Profiles at 1600 degC.
    let times_h = [1.0, 10.0, 24.0, 100.0, 300.0];
    let mut profile_rows: Vec<ProfileRow> = Vec::new();
    for s in Species::ALL {
        let ev = evaluate(&reg, &runner.sets[&s], 1600.0)?;
        profile_rows.extend(profiles(s, 1600.0, &geometry, &ev, settings, &times_h)?);
    }

    // Sensitivity.
    let sens_settings = Settings {
        dt_h: if opts.quick { 0.5 } else { 0.1 },
        ..settings
    };
    let sens = sensitivity(&reg, &geometry, sens_settings)?;

    // Convergence.
    let (sp_levels, sp_steps, tm_levels, tm_steps, tm_mesh) = if opts.quick {
        (3, 200, 4, 10, 2)
    } else {
        (6, 2000, 9, 25, 32)
    };
    let spatial = spatial_convergence(&geometry, sp_levels, sp_steps)?;
    let temporal = temporal_convergence(&geometry, tm_levels, tm_steps, tm_mesh)?;
    let floor = spatial_floor(&geometry, tm_mesh, sp_steps)?;
    let (dh, end_h, big_r) = homogeneous_case_description(&geometry);
    let homogeneous_note = format!(
        "all five regions share the diffusion coefficient {dh:.3e} m²/s, the outer radius is {:.1} µm, the kernel radius {:.1} µm, the end time {end_h} h, so the dimensionless Fourier number D t / R² of the whole sphere, with R the outer radius, is 0.1; the reference is the exact series of a sphere with the same diffusivity everywhere.",
        big_r * 1e6,
        geometry.outer_m[0] * 1e6
    );

    // Files.
    let out = &opts.data_out;
    write(
        &out.join("comparison_rows.csv"),
        &outputs::rows_csv(&prov, &records),
    )?;
    write(
        &out.join("comparison_datasets.csv"),
        &outputs::datasets_csv(&prov, &sets),
    )?;
    write(
        &out.join("release_curves.csv"),
        &outputs::curves_csv(&prov, &curves, &analytic, &analytic_floor, 10),
    )?;
    write(
        &out.join("deviation_curves.csv"),
        &outputs::deviation_csv(&prov, &records),
    )?;
    write(
        &out.join("radial_profiles.csv"),
        &outputs::profiles_csv(&prov, &profile_rows),
    )?;
    write(
        &out.join("inventory_balance.csv"),
        &outputs::balance_csv(&prov, &curves, 10),
    )?;
    write(
        &out.join("temperature_sensitivity.csv"),
        &outputs::sensitivity_csv(&prov, &sens),
    )?;
    write(
        &out.join("convergence_spatial.csv"),
        &outputs::convergence_csv(
            &prov,
            &format!("spatial convergence of the release fraction; {homogeneous_note}"),
            "Kernel mesh width (m)",
            &spatial,
        ),
    )?;
    write(
        &out.join("convergence_temporal.csv"),
        &outputs::convergence_csv(
            &prov,
            &format!(
                "temporal convergence of the release fraction on a fixed mesh; {homogeneous_note}"
            ),
            "Time step (s)",
            &temporal,
        ),
    )?;
    write(
        &out.join("comparison.json"),
        &outputs::comparison_json(&prov, &records, &sets, &geometry, &set_notes, regions),
    )?;

    // Figures.
    let curve_refs: Vec<(Species, f64, &Curve)> =
        curves.iter().map(|(s, t, c, _)| (*s, *t, *c)).collect();
    let radii_um = geometry.outer_m.map(|r| r * 1e6);
    let inp = Inputs {
        reg: &reg,
        records: &records,
        curves: &curve_refs,
        analytic: &analytic,
        profiles: &profile_rows,
        sensitivity: &sens,
        spatial: &spatial,
        temporal: &temporal,
        temporal_spatial_floor: floor,
        homogeneous_note: homogeneous_note.clone(),
        prov: &prov,
        radii_um,
    };
    let f = &opts.figures_out;
    fs::create_dir_all(f).map_err(|e| e.to_string())?;
    figures::figure_a(&f.join("figure_A_release_vs_time.svg"), &inp)?;
    figures::figure_b(&f.join("figure_B_absolute_deviation.svg"), &inp)?;
    figures::figure_c(&f.join("figure_C_relative_deviation.svg"), &inp)?;
    figures::figure_d(&f.join("figure_D_radial_profiles.svg"), &inp)?;
    figures::figure_e(&f.join("figure_E_inventory_balance.svg"), &inp)?;
    figures::figure_f(&f.join("figure_F_spatial_convergence.svg"), &inp)?;
    figures::figure_g(&f.join("figure_G_temporal_convergence.svg"), &inp)?;
    figures::figure_h(&f.join("figure_H_temperature_sensitivity.svg"), &inp)?;
    figures::figure_i(&f.join("figure_I_silver_110m_comparison.svg"), &inp)?;
    figures::figure_j(&f.join("figure_J_strontium_90_comparison.svg"), &inp)?;
    figures::figure_k(&f.join("figure_K_caesium_137_data_unavailable.svg"), &inp)?;

    if let Some(path) = &opts.report_out {
        let text = report::render(&ReportInput {
            reg: &reg,
            records: &records,
            datasets: &sets,
            sensitivity: &sens,
            spatial: &spatial,
            temporal: &temporal,
            curves: &curve_refs,
            analytic: &analytic,
            prov: &prov,
            geometry: &geometry,
            temporal_spatial_floor: floor,
            homogeneous_note: &homogeneous_note,
            set_notes: &set_notes,
        });
        write(path, &text)?;
    }

    Ok(Outcome {
        records,
        datasets: sets,
        sensitivity: sens,
        spatial,
        temporal,
    })
}
