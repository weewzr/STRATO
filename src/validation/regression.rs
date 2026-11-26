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
//! At each checkpoint time `t_k` the run's release fraction `F_run` is compared with the reference
//! `F_ref` by an error estimate that comes from refinements of the same case; the pass/fail
//! criterion is a stated convention (see below). Notation: the run `F_0 = F_run`, then `F_1`, `F_2`, `F_3` with
//! half, a quarter and an eighth of the step (`dt/2`, `dt/4`, `dt/8` in time; `2n`, `4n`, `8n`
//! intervals in every material in space). The two directions are treated separately.
//!
//! *Leading order.* If the error of order `q` (`q = 1` Backward Euler, `q = 2` central
//! differences) is `e(h) = a h^q`, then `F_0 - F_1 = e (1 - 2^-q)`.
//!
//! *Observed order.* Let `D_1 = F_0 - F_1` and `D_2 = F_1 - F_2`. If the error is asymptotic with
//! some order `p`, then `D_2 / D_1 = 2^-p`. This gives `p = log2(D_1 / D_2)` and the estimate of
//! the error of `F_0`, `e_0 = D_1 / (1 - D_2 / D_1)`, in which the order is read from the data and
//! not assumed. The estimate of the limit along that direction is `L_0 = F_0 - e_0`.
//!
//! *Remainder of the estimate.* The same formula applied to `(F_1, F_2, F_3)` gives `e_1`, the
//! error of `F_1`, and a second estimate `L_1 = F_1 - e_1` of the same limit. `u = |L_0 - L_1|`
//! measures how much the estimate still moves between successive refinements; it is recorded for
//! each direction. If `D_2 / D_1` (or the ratio of the finer triple) is not in `(0, 1)`, no order
//! can be read; the leading-order estimate `D_1 / (1 - 2^-q)` replaces `e` and `u` is that whole
//! estimate.
//!
//! *Cross term.* The error expansion is `F = F* + a dt + b h^2 + c dt h^2`. The time estimate is
//! made at the fixed mesh `h`, so it is `e_t = (a + c h^2) dt` and already holds the cross term;
//! the space estimate is made at the fixed step `dt`, so it is `e_h = (b + c dt) h^2` and holds
//! it as well. The mixed difference of the four runs `(dt, h)`, `(dt/2, h)`, `(dt, h/2)`,
//! `(dt/2, h/2)` is `c dt h^2 (1 - 1/2)(1 - 1/4)`, so `e_x = c dt h^2 = (8/3)(F_0 - F_t - F_h +
//! F_th)`.
//!
//! The estimate of the numerical limit of the case removes `e_t` and `e_h` and adds the cross term
//! back, because it was subtracted twice: `F_lim = F_0 - e_t - e_h + e_x` (for an exact expansion
//! this equals `F*`; the unit test `limit_of_an_exact_expansion_is_recovered` checks it). The
//! first version of this module had `- e_x`; that sign was wrong and is corrected here, with the
//! three `independent_check.csv` files regenerated for that reason.
//!
//! # What is measured, and what is a convention
//!
//! Three parts of the check are asserted. (a) The observed orders round to the orders that
//! Document 1 proves or observes: `|p_t - 1| < 1/2` in time and `|p_h - 2| < 1/2` in space, so a
//! solver whose spatial error is only first order (for example a wrong diffusivity on one
//! interval of a coating) fails. No rounding allowance is added to the `1/2`: the observed orders
//! deviate from the integer by at most 0.07. (b) The limit estimate above. (c) The convention below.
//!
//! Measured and recorded in every check file: the observed orders (0.93 to 1.02 in time, 2.000 in
//! space), the uncertainties `u`, and `|F_ref - F_lim|`. On the three cases `|F_ref - F_lim|` is
//! between 0.01 % and 1.91 % of `|F_run - F_ref|`: the refinement analysis explains the difference
//! between the run and the reference to that fraction, the rest is left unexplained. It is **not**
//! proved to be a bound: the kernel-step start is not smooth, the time orders read 0.93 to 1.02
//! rather than 1, and `u` (which only measures how much the estimate moves) is in several rows
//! smaller than the remaining difference, so it is not a bound either.
//!
//! The pass/fail criterion of the generator on the limit estimate is a **convention**, not a
//! derived tolerance:
//!
//! ```text
//! |F_ref - F_lim| <= EXPLAINED_FRACTION |F_run - F_ref| + floor,   EXPLAINED_FRACTION = 0.05
//! ```
//!
//! "The discretisation error that the refinements estimate explains the difference from the
//! independent reference to within 5 %." The 5 % is a round number that was chosen after the
//! measurement, above the 1.91 % measured; it is not derived from Document 1, it is fixed
//! (decision 49) and a smaller or larger measured value does not change it. The part of the check
//! that discriminates a defect is the order assertion (a): under the mutation recorded in the
//! results file the ratio test alone passed on the mild-contrast case. `floor` is the accuracy
//! estimate of the reference (`ModalRelease::accuracy_floor`, or the tail bound plus `1e-15` of
//! the homogeneous series); its documentation says the true error is about 10 to 20 times below
//! it, so one times the floor is used. What the criterion protects against is a gross
//! disagreement (an error in the BTCS run or in the reference that is much larger than the
//! discretisation error); it does not certify the last digits of the run. A failed check refuses
//! the file: nothing is written.
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

/// Convention of the independent check: the part of `|F_run - F_ref|` that the refinement analysis
/// may leave unexplained (see the module documentation; not derived).
pub const EXPLAINED_FRACTION: f64 = 0.05;

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
    /// Estimated error of the run in time, `e_0` (signed).
    pub time_error: f64,
    /// Observed order in time from `(F_0, F_1, F_2)`; NaN if it cannot be read.
    pub time_order: f64,
    /// Uncertainty `u` of the time estimate, `|L_0 - L_1|`.
    pub time_uncertainty: f64,
    /// Estimated error of the run in space, `e_0` (signed).
    pub space_error: f64,
    /// Observed order in space from `(F_0, F_1, F_2)`; NaN if it cannot be read.
    pub space_order: f64,
    /// Uncertainty of the space estimate.
    pub space_uncertainty: f64,
    /// Estimated cross term of the time and space errors (signed).
    pub cross_error: f64,
    /// Estimated numerical limit `F_lim = F_run - e_t - e_h + e_x`.
    pub f_limit: f64,
    /// `|F_ref - F_lim|`.
    pub limit_difference: f64,
    /// Accuracy floor of the reference.
    pub reference_floor: f64,
    /// `EXPLAINED_FRACTION |F_run - F_ref| + floor` (a convention; see the module documentation).
    pub tolerance: f64,
}

impl CheckRow {
    /// The observed orders round to the orders of the scheme: `|p_t - 1| < 1/2` and
    /// `|p_h - 2| < 1/2` (decision 49, part (a)). An order that could not be read (NaN) fails.
    pub fn orders_round(&self) -> bool {
        (self.time_order - 1.0).abs() < 0.5 && (self.space_order - 2.0).abs() < 0.5
    }

    /// The limit estimate agrees with the reference to the convention of the module
    /// documentation: `limit_difference <= tolerance`.
    pub fn limit_agrees(&self) -> bool {
        self.limit_difference <= self.tolerance
    }

    /// The check passes if the orders round to 1 and 2 and the limit estimate agrees.
    pub fn passed(&self) -> bool {
        self.orders_round() && self.limit_agrees()
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

/// Result of `estimate`.
struct Estimate {
    error: f64,
    order: f64,
    uncertainty: f64,
}

/// Error estimate of one refinement direction from the runs `f = [F_0, F_1, F_2, F_3]` (each with
/// half the step of the previous one) for a leading order `q`; see the module documentation.
fn estimate(f: [f64; 4], q: f64) -> Estimate {
    let leading = |a: f64, b: f64| (a - b) / (1.0 - 0.5_f64.powf(q));
    // Error of `a` from the triple (a, b, c) with the order read from the data, and that order.
    let observed = |a: f64, b: f64, c: f64| -> Option<(f64, f64)> {
        let (d1, d2) = (a - b, b - c);
        let r = d2 / d1;
        (r > 0.0 && r < 1.0).then(|| (d1 / (1.0 - r), -r.log2()))
    };
    match (observed(f[0], f[1], f[2]), observed(f[1], f[2], f[3])) {
        (Some((e0, order)), Some((e1, _))) => Estimate {
            error: e0,
            order,
            uncertainty: ((f[0] - e0) - (f[1] - e1)).abs(),
        },
        (first, _) => {
            let e = leading(f[0], f[1]);
            Estimate {
                error: e,
                order: first.map_or(f64::NAN, |(_, p)| p),
                uncertainty: e.abs(),
            }
        }
    }
}

/// Limit estimate `F_0 - e_t - e_h + e_x` of the module documentation.
fn limit_estimate(f_run: f64, time_error: f64, space_error: f64, cross: f64) -> f64 {
    f_run - time_error - space_error + cross
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
    let finer_t = run_input(&refined(&input, 1, 4), &[])?;
    let finer_h = run_input(&refined(&input, 4, 1), &[])?;
    let mixed = run_input(&refined(&input, 2, 2), &[])?;
    let finest_t = run_input(&refined(&input, 1, 8), &[])?;
    let finest_h = run_input(&refined(&input, 8, 1), &[])?;
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
        let time = estimate(
            [
                f_run,
                f_at(&fine_t.series, t)?,
                f_at(&finer_t.series, t)?,
                f_at(&finest_t.series, t)?,
            ],
            1.0,
        );
        let space = estimate(
            [
                f_run,
                f_at(&fine_h.series, t)?,
                f_at(&finer_h.series, t)?,
                f_at(&finest_h.series, t)?,
            ],
            2.0,
        );
        // Cross term of the error expansion `a dt + b h^2 + c dt h^2`: the mixed second difference
        // of the four runs (dt, h), (dt/2, h), (dt, h/2), (dt/2, h/2) equals `c dt h^2 (1 - 1/2)
        // (1 - 1/4)`, hence the factor 8/3.
        let cross = 8.0 / 3.0
            * (f_run - f_at(&fine_t.series, t)? - f_at(&fine_h.series, t)?
                + f_at(&mixed.series, t)?);
        let f_limit = limit_estimate(f_run, time.error, space.error, cross);
        rows.push(CheckRow {
            t,
            f_run,
            f_reference: reference[k],
            abs_difference: (f_run - reference[k]).abs(),
            time_error: time.error,
            time_order: time.order,
            time_uncertainty: time.uncertainty,
            space_error: space.error,
            space_order: space.order,
            space_uncertainty: space.uncertainty,
            cross_error: cross,
            f_limit,
            limit_difference: (reference[k] - f_limit).abs(),
            reference_floor: floor,
            tolerance: EXPLAINED_FRACTION * (f_run - reference[k]).abs() + floor,
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
            "independent check failed at t = {} s: observed orders {} (time) and {} (space), must round to 1 and 2: {}; |F_ref - F_limit| = {:e}, tolerance {:e}: {}",
            bad.t,
            bad.time_order,
            bad.space_order,
            if bad.orders_round() { "ok" } else { "FAILED" },
            bad.limit_difference,
            bad.tolerance,
            if bad.limit_agrees() { "ok" } else { "FAILED" },
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
        "# check: (a) the observed orders round to 1 in time and 2 in space (|order - 1| < 0.5 and |order - 2| < 0.5); (b) |F_reference - F_limit| <= 0.05 |F_btcs - F_reference| + floor, where F_limit = F_btcs - e_time - e_space + e_cross is the limit estimated from the dt/2, dt/4, dt/8, 2n, 4n, 8n and (dt/2, 2n) refinements with the order read from the data (the cross term is added back because the time and space estimates each contain it). The 0.05 is a convention, not a derived tolerance; the orders, the uncertainties u of the estimates and the limit difference are measurements. See the module documentation of validation::regression."
    );
    if let (Some((ta, fa)), Some(diff)) = (case.anchor, report.anchor_difference) {
        let _ = writeln!(
            check,
            "# separate-language anchor: F({ta:e} s) = {fa:e} from the Python oracle pinned in tests/analytical_reference.rs; |series - anchor| = {diff:e}"
        );
    }
    let _ = writeln!(check, "# {}", crate::postprocessing::export::LABEL);
    check.push_str(&crate::postprocessing::export::header_line(
        &crate::postprocessing::export::CHECK_COLUMNS,
    ));
    for r in &report.rows {
        let _ = writeln!(
            check,
            "{:e},{:e},{:e},{:e},{:e},{:e},{:e},{:e},{:e},{:e},{:e},{:e},{:e},{:e},{:e},{:e},{}",
            r.t,
            r.f_run,
            r.f_reference,
            r.abs_difference,
            r.time_error,
            r.time_order,
            r.time_uncertainty,
            r.space_error,
            r.space_order,
            r.space_uncertainty,
            r.cross_error,
            r.f_limit,
            r.limit_difference,
            r.reference_floor,
            r.tolerance,
            r.limit_difference / r.tolerance,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// An exact expansion `F = F* + a dt + b h^2 + c dt h^2` sampled the way the check samples a
    /// case: the time estimate at the fixed mesh `h`, the space estimate at the fixed step `dt`,
    /// and the four runs of the cross term.
    fn synthetic(f_star: f64, a: f64, b: f64, c: f64, dt: f64, h: f64) -> (f64, f64, f64, f64) {
        let f = |dt: f64, h: f64| f_star + a * dt + b * h * h + c * dt * h * h;
        let time_runs = [0, 1, 2, 3].map(|k| f(dt / 2f64.powi(k), h));
        let space_runs = [0, 1, 2, 3].map(|k| f(dt, h / 2f64.powi(k)));
        let time = estimate(time_runs, 1.0);
        let space = estimate(space_runs, 2.0);
        let cross = 8.0 / 3.0 * (f(dt, h) - f(dt / 2.0, h) - f(dt, h / 2.0) + f(dt / 2.0, h / 2.0));
        (
            limit_estimate(f(dt, h), time.error, space.error, cross),
            time.error,
            space.error,
            cross,
        )
    }

    #[test]
    fn limit_of_an_exact_expansion_is_recovered() {
        // The cross term is large on purpose: c dt h^2 = 0.076 against F* = 0.4.
        let (f_star, a, b, c, dt, h) = (0.4, 0.3, -0.7, 19.0, 0.1, 0.2);
        let (lim, e_t, e_h, e_x) = synthetic(f_star, a, b, c, dt, h);
        // Each estimate holds the cross term, as derived in the module documentation.
        assert!((e_t - (a + c * h * h) * dt).abs() < 1e-15);
        assert!((e_h - (b + c * dt) * h * h).abs() < 1e-15);
        assert!((e_x - c * dt * h * h).abs() < 1e-15);
        // Every quantity is O(1) or smaller and each is a handful of operations: a few times
        // the unit roundoff of 1.1e-16 relative to 1, the largest value involved.
        assert!(
            (lim - f_star).abs() < 1e-14,
            "limit {lim} differs from F* = {f_star}"
        );
        // The wrong sign of the first version is off by 2 c dt h^2 and is therefore detected.
        let wrong = (lim - 2.0 * e_x) - f_star;
        assert!((wrong + 2.0 * c * dt * h * h).abs() < 1e-14);
        assert!(wrong.abs() > 1e-3);
    }

    #[test]
    fn orders_must_round_to_one_and_two() {
        let row = |p_t: f64, p_h: f64| CheckRow {
            t: 1.0,
            f_run: 0.0,
            f_reference: 0.0,
            abs_difference: 0.0,
            time_error: 0.0,
            time_order: p_t,
            time_uncertainty: 0.0,
            space_error: 0.0,
            space_order: p_h,
            space_uncertainty: 0.0,
            cross_error: 0.0,
            f_limit: 0.0,
            limit_difference: 0.0,
            reference_floor: 0.0,
            tolerance: 0.0,
        };
        assert!(row(0.93, 2.0).passed());
        assert!(row(1.02, 1.9994).passed());
        // A first-order spatial defect, a zeroth-order time error and an unreadable order fail.
        assert!(!row(1.0, 1.0).passed());
        assert!(!row(0.4, 2.0).passed());
        assert!(!row(1.0, f64::NAN).passed());
        assert!(!row(f64::NAN, 2.0).passed());
    }
}
