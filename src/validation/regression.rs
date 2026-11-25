//! BTCS regression suite (directive Section 11, test I): cases, independent checks and the
//! text of the frozen reference files.
//!
//! # What these files are
//!
//! A regression reference is the output of this solver at a stated commit, kept so that a later
//! change of the code that moves a value is noticed. It is **not truth**. A mismatch means that
//! the code changed; the frozen file is never edited by hand and never regenerated to make a test
//! pass without a recorded reason. The finite-volume golden files of `tests/golden/v1.0/` are not
//! used here, not as truth and not as reference.
//!
//! # Why each file is believable: the independent check
//!
//! Every case is generated together with a check against a reference that contains no BTCS code:
//!
//! * the Sturm-Liouville eigenfunction series of `analytical::sturm_liouville` (five layers), or
//! * the closed-form series of the homogeneous sphere of `analytical::homogeneous_sphere`.
//!
//! At each checkpoint time `t_k` the run's release fraction `F_run` must agree with the reference
//! `F_ref` to a tolerance that is derived from the run itself, by the error estimate of rule R2
//! of the verification plan, written for the run being checked (the coarse member of a pair):
//!
//! ```text
//! tolerance_k = 2.5 |F_run - F_(dt/2)| + (5/3) |F_run - F_(2n)| + 3 floor
//! ```
//!
//! `F_(dt/2)` is the same case with half the step, `F_(2n)` the same case with twice the intervals
//! in every material. For a first-order time error `e` the finer run has error `e/2`, so
//! `|F_run - F_(dt/2)| = e/2` and `e = 2 |F_run - F_(dt/2)|`; the factor 2.5 is that 2 times the
//! safety factor 1.25 of rule R2. For a second-order space error the finer run has `e/4`, so
//! `e = (4/3) |F_run - F_(2n)|` and the factor is `(4/3) 1.25 = 5/3`. `floor` is the accuracy
//! floor of the reference (`ModalRelease::accuracy_floor`, or the tail bound plus `1e-15` of the
//! homogeneous series). The orders 1 and 2 are the theoretical orders of Backward Euler and of
//! central differences; the observed orders on the synthetic case are in
//! `tests/btcs_verification.rs`. A failed check refuses the file: nothing is written.
//!
//! Numerical verification only: agreement with these references is not experimental validation.

use crate::{
    analytical::{
        homogeneous_sphere::HomogeneousSphere,
        sturm_liouville::{Layer, ModalRelease},
    },
    config::Config,
    numerical::{
        btcs::{run_input, validate_input, Limits},
        diagnostics::TimeSeries,
        grid::ValidatedInput,
    },
    postprocessing::export::{profile_csv, timeseries_csv, Provenance},
};
use anyhow::{anyhow, bail, Result};
use std::fmt::Write as _;

/// Relative tolerance of the comparison with the frozen files: the existing regression tolerance
/// of `tests/phase1_regression.rs` (`|a - b| <= RTOL max(scale, |a|, |b|)`).
pub const RTOL: f64 = 1e-12;

/// Independent reference of a case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reference {
    /// Multilayer Sturm-Liouville series, kernel-step start.
    SturmLiouville,
    /// Closed-form series of the homogeneous sphere (all five diffusivities equal).
    HomogeneousSphere,
}

/// One regression case.
#[derive(Clone, Copy, Debug)]
pub struct Case {
    /// Directory name of the frozen files.
    pub id: &'static str,
    /// TOML input, relative to the repository root.
    pub input: &'static str,
    /// Checkpoint times of the independent check, s (multiples of the output stride).
    pub checkpoints: &'static [f64],
    /// The independent reference.
    pub reference: Reference,
    /// A value from a separate-language script that the repository pins, `(t_s, F)`, recorded in
    /// the header of the check file (it is asserted by `tests/btcs_verification.rs`).
    pub anchor: Option<(f64, f64)>,
}

/// The cases of the suite.
pub const CASES: [Case; 3] = [
    Case {
        id: "s5_n40_dt200",
        input: "inputs/synthetic/synthetic_five_layer.toml",
        checkpoints: &[2.0e4, 5.0e4, 1.0e5, 2.0e5],
        reference: Reference::SturmLiouville,
        anchor: Some((2.0e5, 0.4325556390930383)),
    },
    Case {
        id: "mild_contrast_n40_dt10",
        input: "inputs/btcs_regression/synthetic_mild_contrast.toml",
        checkpoints: &[500.0, 1000.0, 2000.0],
        reference: Reference::SturmLiouville,
        anchor: None,
    },
    Case {
        id: "homogeneous_n40_steps200",
        input: "inputs/btcs_regression/synthetic_homogeneous_sphere.toml",
        checkpoints: &[101.25, 405.0, 1012.5],
        reference: Reference::HomogeneousSphere,
        anchor: None,
    },
];

const LIMITS: Limits = Limits {
    max_intervals_total: 1_000_000,
    max_steps: 10_000_000,
};

/// One row of the independent check.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckRow {
    /// Checkpoint time, s.
    pub t: f64,
    /// Release fraction of the run.
    pub f_run: f64,
    /// Release fraction of the independent reference.
    pub f_reference: f64,
    /// `|F_run - F_ref|`.
    pub abs_difference: f64,
    /// `2.5 |F_run - F_(dt/2)|`.
    pub time_error_estimate: f64,
    /// `(5/3) |F_run - F_(2n)|`.
    pub space_error_estimate: f64,
    /// `3 floor` of the reference.
    pub reference_floor_3x: f64,
    /// Sum of the three terms above.
    pub tolerance: f64,
}

impl CheckRow {
    /// The check passes if `abs_difference <= tolerance`.
    pub fn passed(&self) -> bool {
        self.abs_difference <= self.tolerance
    }
}

/// Result of the independent check of a case.
#[derive(Clone, Debug)]
pub struct CheckReport {
    /// Rows, one per checkpoint.
    pub rows: Vec<CheckRow>,
    /// Text describing the reference.
    pub reference_text: String,
    /// `|F_series(t_anchor) - anchor|` if the case has an anchor.
    pub anchor_difference: Option<f64>,
}

fn f_at(series: &TimeSeries, t: f64) -> Result<f64> {
    series
        .iter()
        .find(|(ts, _)| (ts - t).abs() <= 1e-9 * t)
        .map(|(_, r)| r.release_fraction)
        .ok_or_else(|| {
            anyhow!(
                "no output row at t = {t} s; the checkpoint is not a multiple of the output stride"
            )
        })
}

/// The case refined in time (`time` times as many steps) or in space (`space` times the
/// intervals of every material); the output stride keeps the same output times.
fn refined(input: &ValidatedInput, space: usize, time: usize) -> ValidatedInput {
    let mut r = input.clone();
    for n in r.n_intervals.iter_mut() {
        *n *= space;
    }
    r.steps *= time;
    r.dt /= time as f64;
    r.output_every *= time;
    r
}

/// Runs the independent check of a case on its configuration.
///
/// # Errors
///
/// An invalid input, a checkpoint that is not an output time, a reference that is not valid at a
/// checkpoint, or a case whose diffusivities are not equal for the homogeneous reference.
pub fn independent_check(cfg: &Config, case: &Case) -> Result<CheckReport> {
    let input = validate_input(cfg, &LIMITS)?;
    let base = run_input(&input, &[])?;
    let fine_t = run_input(&refined(&input, 1, 2), &[])?;
    let fine_h = run_input(&refined(&input, 2, 1), &[])?;
    let t_min = case
        .checkpoints
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);

    let mut anchor_difference = None;
    let (reference, floor, reference_text): (Vec<f64>, f64, String) = match case.reference {
        Reference::SturmLiouville => {
            let layers: Vec<Layer> = cfg
                .regions
                .iter()
                .map(|r| Layer {
                    outer_radius: r.outer_radius_m,
                    diffusivity: r.diffusivity_m2_s,
                })
                .collect();
            let series = ModalRelease::new(&layers, 45.0 / t_min, 20_000)?;
            if series.min_valid_time() > t_min {
                bail!("the series is not valid from t = {t_min} s");
            }
            if let Some((ta, fa)) = case.anchor {
                anchor_difference = Some((series.release_fraction(ta) - fa).abs());
            }
            let values = case
                .checkpoints
                .iter()
                .map(|&t| series.release_fraction(t))
                .collect();
            (
                values,
                series.accuracy_floor(),
                format!(
                    "multilayer Sturm-Liouville eigenfunction series (analytical::sturm_liouville), {} modes, accuracy floor {:e}",
                    series.eigenvalues.len(),
                    series.accuracy_floor()
                ),
            )
        }
        Reference::HomogeneousSphere => {
            let d = cfg.regions[0].diffusivity_m2_s;
            if cfg.regions.iter().any(|r| r.diffusivity_m2_s != d) {
                bail!("the homogeneous reference needs five equal diffusivities");
            }
            let sphere = HomogeneousSphere::core(
                cfg.regions[4].outer_radius_m,
                cfg.regions[0].outer_radius_m,
                d,
                cfg.initial.kernel_concentration_mol_m3,
            )?;
            let mut values = Vec::new();
            let mut bound: f64 = 0.0;
            for &t in case.checkpoints {
                let e = sphere.release_fraction(t, 1e-14)?;
                values.push(e.value);
                bound = bound.max(e.error_bound + 1e-15);
            }
            (
                values,
                bound,
                format!(
                    "closed-form series of the homogeneous sphere with a kernel-step start (analytical::homogeneous_sphere), tail tolerance 1e-14, floor {bound:e}"
                ),
            )
        }
    };

    let mut rows = Vec::new();
    for (k, &t) in case.checkpoints.iter().enumerate() {
        let f_run = f_at(&base.series, t)?;
        let time_error_estimate = 2.5 * (f_run - f_at(&fine_t.series, t)?).abs();
        let space_error_estimate = 5.0 / 3.0 * (f_run - f_at(&fine_h.series, t)?).abs();
        let reference_floor_3x = 3.0 * floor;
        rows.push(CheckRow {
            t,
            f_run,
            f_reference: reference[k],
            abs_difference: (f_run - reference[k]).abs(),
            time_error_estimate,
            space_error_estimate,
            reference_floor_3x,
            tolerance: time_error_estimate + space_error_estimate + reference_floor_3x,
        });
    }
    Ok(CheckReport {
        rows,
        reference_text,
        anchor_difference,
    })
}

/// The text of the three frozen files of a case.
#[derive(Clone, Debug)]
pub struct Generated {
    /// `timeseries.csv`.
    pub timeseries: String,
    /// `final_profile.csv`.
    pub final_profile: String,
    /// `independent_check.csv`.
    pub independent_check: String,
}

fn regression_banner(case: &Case, commit: &str) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "# REGRESSION REFERENCE, NOT TRUTH.");
    let _ = writeln!(
        s,
        "# These values were produced by the BTCS solver itself at commit {commit}. A mismatch with a later run means that the code changed; the file is never edited by hand."
    );
    let _ = writeln!(
        s,
        "# Why the values are believable: independent_check.csv beside this file compares the release fraction of this case with a reference that contains no BTCS code."
    );
    let _ = writeln!(s, "# case: {}", case.id);
    let _ = writeln!(s, "# input: {}", case.input);
    let _ = writeln!(
        s,
        "# Comparison rule: |a - b| <= {RTOL:e} max(scale, |a|, |b|), scale = 1 for times and fractions, the initial inventory for mol, the largest concentration for mol/m^3."
    );
    s
}

/// Runs a case and builds the text of its frozen files. The independent check must pass.
///
/// # Errors
///
/// A failed independent check (nothing is generated), or any error of the run.
pub fn generate(cfg: &Config, case: &Case, input_file: &str, commit: &str) -> Result<Generated> {
    let report = independent_check(cfg, case)?;
    if let Some(bad) = report.rows.iter().find(|r| !r.passed()) {
        bail!(
            "independent check failed at t = {} s: |F_run - F_ref| = {:e} > tolerance {:e}",
            bad.t,
            bad.abs_difference,
            bad.tolerance
        );
    }
    let input = validate_input(cfg, &LIMITS)?;
    let end = input.dt * input.steps as f64;
    let run = run_input(&input, &[end])?;
    // All regression cases are synthetic numerical benchmarks (see the README of the files).
    let prov = Provenance::new(cfg, &input, input_file, commit).with_parameter_source(
        crate::postprocessing::export::SYNTHETIC_SOURCE,
        "synthetic benchmark",
    );
    let banner = regression_banner(case, commit);

    let mut check = banner.clone();
    let _ = writeln!(check, "# reference: {}", report.reference_text);
    let _ = writeln!(
        check,
        "# tolerance_k = 2.5 |F_run - F_(dt/2)| + (5/3) |F_run - F_(2n)| + 3 floor (rule R2 of the verification plan, written for the coarse run of a pair; see validation::regression)"
    );
    if let (Some((ta, fa)), Some(diff)) = (case.anchor, report.anchor_difference) {
        let _ = writeln!(
            check,
            "# separate-language anchor: F({ta:e} s) = {fa:e} from the Python oracle pinned in tests/finite_volume_vs_analytical.rs; |series - anchor| = {diff:e}"
        );
    }
    let _ = writeln!(check, "# {}", crate::postprocessing::export::LABEL);
    check.push_str(&crate::postprocessing::export::header_line(
        &crate::postprocessing::export::CHECK_COLUMNS,
    ));
    for r in &report.rows {
        let _ = writeln!(
            check,
            "{:e},{:e},{:e},{:e},{:e},{:e},{:e},{:e},{:e},{}",
            r.t,
            r.f_run,
            r.f_reference,
            r.abs_difference,
            r.time_error_estimate,
            r.space_error_estimate,
            r.reference_floor_3x,
            r.tolerance,
            r.abs_difference / r.tolerance,
            r.passed()
        );
    }

    let ts = timeseries_csv(&prov, &run.series, run.meta.initial_inventory)?;
    let pr = profile_csv(&prov, &run.grid, &run.snapshots)?;
    Ok(Generated {
        timeseries: format!("{banner}{ts}"),
        final_profile: format!("{banner}{pr}"),
        independent_check: check,
    })
}
