//! Numerical verification of the five-layer BTCS solver (directive Section 11, tests A to G;
//! Document 2 properties P1 to P18).
//!
//! These tests check the numerical solution of the stated model against analytical
//! references that contain no BTCS code. They are NOT experimental validation.
//!
//! # What is asserted, and where each bound comes from
//!
//! No assertion in this file rests on a literal tolerance without a derivation. Each bound
//! is one of the following.
//!
//! * A rounding bound derived from the operation count (u = 2^-53, gamma_k = k u/(1 - k u)):
//!   the F31 identity (`interfaces_concentration_and_flux_continuity`), the balance bound
//!   (`balance_bound_history`, with its derivation), the maximum-principle bound and the
//!   margin deviation of the matrix (P1), and the check that a time is a multiple of the
//!   step. Their constants are `kappa_t` (backward error of the F24 solve, derived in
//!   `numerical::tridiagonal::tests::thomas_matches_dense_f24`) and `c_step`. The O(u^2)
//!   terms of these derivations are taken into the stated factors (see each comment); no
//!   safety factor is applied anywhere. The bound expressions are themselves evaluated in
//!   round-to-nearest and that evaluation (a few u relative) is taken as exact: where a
//!   bound is evaluated, the observed value is at most 0.4 of it (the multiple-of-dt check
//!   about 0.4; P1 up to 0.25; the Kahan unit test about 0.25; table F of the results file at
//!   most 1.3e-2), so the omitted few-u relative error cannot change an outcome; it is not
//!   added to any bound.
//! * A sign or a monotone sequence, which needs no tolerance: the error against the series
//!   falls at every grid refinement (test A, B, C3), the Cauchy differences fall at every
//!   step halving (D1, D2), the relative flux jump at each interface falls (E), the
//!   positivity `min C >= 0` is exact (G), and the negative control of the balance identity
//!   exceeds the derived rounding bound (F). Where the reference carries an uncertainty (the
//!   Sturm-Liouville series reports `accuracy_floor`, an estimate; the homogeneous series
//!   reports a truncation `error_bound`), the monotone check allows the sum of the two
//!   uncertainties and nothing more.
//! * The accuracy of the Sturm-Liouville series against the independent value of
//!   `reconciliation.md` (item 18, 14 decimals): `accuracy_floor` plus the 5e-15 rounding of
//!   that decimal value.
//!
//! The orders are NOT asserted: Documents 1 and 2 state second order in space and first order
//! in time as OBSERVED (properties P8b, P16, P17; P14 and P15 are proof outlines of
//! `O(dr^2 + dt)` and `O(dr + dt)` without a constant), they state no band, and decision 7
//! withdrew the plus-or-minus 0.2 order bands and the 1e-6 target inherited from the
//! finite-volume work. The observed orders are printed (`BTCS_RESULT|...|order_...`) and
//! recorded in `solver-core-results.md`. What is lost by this: a regression that merely
//! degrades the order while the error still falls is not caught here; the frozen regression
//! files of the separate regression task are meant to catch it.
//!
//! # Not derived, and said so
//!
//! * `accuracy_floor` of the layered series is an estimate of the module (its documentation
//!   says the true error stayed 10 to 20 times below it), not a proved bound.
//! * The compensated (Kahan) accumulation of the release is not bounded by an assumed
//!   constant: the balance bound uses the exactly measured error of that accumulation
//!   (`Expansion`), so it is an a-posteriori bound (a function of the run). Goldberg's Theorem 8
//!   gives `2u + O(N u^2)`; the constant hidden in the O() is not used.
//! * The references in the smooth-mode tests (an exponential times a sinc) are evaluated in
//!   floating point; their rounding (a few u) is far below every error compared (at least 6e-13
//!   in the weighted L2 norm, 4e-7 in the maximum norm) and is not added to a bound.
//!
//! Where the results of derivation and an earlier literal differ, see the results file
//! ("Revisions", item 7).
//!
//! # Revisions after the first run (recorded, not hidden)
//!
//! 1. The balance bound as first written was `u N sqrt(steps)` (the heuristic of the
//!    verification plan). It failed at n = 20 (observed 2.3e-11 against 3.5e-13) because it
//!    contains no Fourier number, whereas Document 2 7.4 and Document 3 RP-4 require one. It
//!    was replaced by a derived a-posteriori bound, later by the one documented in
//!    `balance_bound_history`.
//! 2. Test A first used the base step `dt = 200 s` for the extrapolation triple. At
//!    t = 5e3 s (25 steps, F = 1.8e-7, an exponentially small tail) that triple is not in the
//!    asymptotic range: a same-grid step sweep (n = 160) shows F(5e3) = 1.24e-6, 3.4e-7,
//!    2.16e-7, 1.90e-7, 1.84e-7, 1.82e-7 for dt = 200, 50, 12.5, 3.1, 0.78, 0.20 s,
//!    first order in dt. The base step was changed to 25 s (triple 25, 12.5, 6.25 s) for
//!    every time of test A.
//! 3. The inventory-monotonicity check allows a rounding slack because `M^j` moves by
//!    rounding while the release is still exactly zero; the slack is the accumulating
//!    per-step term of the balance bound plus the rounding of the two reported inventories.
//! 4. The maximum bound `max C <= C_init (1 - kappa)^-k` (maximum principle P3, P4;
//!    `kappa = (g1 + 2 gamma_5) B_max`, derived in the test) replaced `max C <= C_init
//!    (1 + 10 N u)`, which had no derivation.
//! 5. Order bands, the 1e-6 target, the factor 1.25 Richardson check and the factor 1.8 on the
//!    flux jump were replaced by monotone checks (see above); the observed orders are
//!    unchanged and still printed.
//!
//! Time error is removed from space studies by Richardson extrapolation over `dt, dt/2,
//! dt/4`: `(8 F(dt/4) - 6 F(dt/2) + F(dt)) / 3`, which cancels the `dt` and `dt^2` terms.
//!
//! Measured numbers are printed as `BTCS_RESULT|test|quantity|value` lines.
use std::f64::consts::PI;
use strato::{
    analytical::{
        homogeneous_sphere::HomogeneousSphere,
        sturm_liouville::{Layer, ModalRelease},
    },
    config::Config,
    numerical::{
        btcs::{assemble, run, Limits, Stepper},
        coefficients::Coefficients,
        diagnostics::{check_matrix_structure, check_series, check_step_properties},
        grid::Grid,
        interfaces::interface_flux_both_sides,
        inventory::{initial_vector, inventory},
    },
};

/// Unit roundoff of f64, `u = 2^-53`.
const U: f64 = f64::EPSILON / 2.0;
const LIMITS: Limits = Limits {
    max_intervals_total: 1_000_000,
    max_steps: 10_000_000,
};
const S5_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/inputs/synthetic/synthetic_five_layer.toml"
);
const GRIDS: [usize; 5] = [20, 40, 80, 160, 320];

fn report(test: &str, quantity: &str, value: f64) {
    eprintln!("BTCS_RESULT|{test}|{quantity}|{value:.6e}");
}

fn s5(n: usize, dt: f64) -> Config {
    let mut cfg = Config::load(S5_FILE).unwrap();
    for r in cfg.regions.iter_mut() {
        r.cells = n;
    }
    cfg.time.dt_s = dt;
    cfg.time.output_every_steps = 1;
    cfg
}

fn layers(cfg: &Config) -> Vec<Layer> {
    cfg.regions
        .iter()
        .map(|r| Layer {
            outer_radius: r.outer_radius_m,
            diffusivity: r.diffusivity_m2_s,
        })
        .collect()
}

/// Release fraction at the given times (multiples of dt), output every step.
fn f_at(cfg: &Config, times: &[f64]) -> Vec<f64> {
    let r = run(cfg, &LIMITS, &[]).unwrap();
    let dt = r.meta.dt;
    times
        .iter()
        .map(|&t| {
            let step = (t / dt).round() as usize;
            // Derived (u = 2^-53, |d_k| <= u each). With tau a decimal literal, s the shared
            // double tau_scale (test B uses the same rounded s in the times and in `end`) and
            // end = s/2 exactly: t = fl(fl(tau) s) = tau s (1 + d1)(1 + d2), one rounding for the
            // literal and one for the product. The step count is the integer 2000 2^k, so
            // step/steps = 2 tau exactly; dt = fl(end/steps) = (end/steps)(1 + d3) is one
            // division, and fl(step dt) = step dt (1 + d4) one product. Hence
            // step dt / t = (1 + d3)(1 + d4)/((1 + d1)(1 + d2)), and
            // |step dt - t| <= ((1 + u)^2/(1 - u)^2 - 1) t = 4u/(1 - u)^2 t <= gamma_4 t.
            // For the times of tests A and D1 (dyadic dt, integer t) both sides are exact. If the
            // run reported another step count the assertion would fail, which is the check.
            assert!(
                (step as f64 * dt - t).abs() <= gamma(4.0) * t,
                "time {t} is not a multiple of dt {dt}"
            );
            r.series[step].1.release_fraction
        })
        .collect()
}

fn richardson(f1: f64, f2: f64, f4: f64) -> f64 {
    (8.0 * f4 - 6.0 * f2 + f1) / 3.0
}

/// F at the times with the time error removed (dt, dt/2, dt/4).
fn f_time_extrapolated(base: &Config, times: &[f64]) -> Vec<f64> {
    let dt = base.time.dt_s;
    let mut cfgs = Vec::new();
    for k in [1.0, 2.0, 4.0] {
        let mut c = base.clone();
        c.time.dt_s = dt / k;
        cfgs.push(f_at(&c, times));
    }
    (0..times.len())
        .map(|i| richardson(cfgs[0][i], cfgs[1][i], cfgs[2][i]))
        .collect()
}

fn orders(errors: &[f64]) -> Vec<f64> {
    errors.windows(2).map(|w| (w[0] / w[1]).log2()).collect()
}

/// `gamma_k = k u / (1 - k u)`.
fn gamma(k: f64) -> f64 {
    k * U / (1.0 - k * U)
}

/// Backward-error constant of the F24 Thomas solve for the sign pattern of (F19), derived in
/// `numerical::tridiagonal::tests::thomas_matches_dense_f24`: 4u + O(u^2), at most gamma_5.
fn kappa_t() -> f64 {
    let a = U / (1.0 - U);
    (2.0 * a + gamma(2.0) + a * gamma(2.0)) * (1.0 - U) / (1.0 - 2.0 * U)
}

/// Per-step coefficient of the exact-form row residual, `|rho_i| <= c_step (|A||x|)_i`:
/// the solve (kappa_t) plus the stored entries (3 u/(1 - 3u): one rounding each in L and U and
/// two in B, against the exact-form matrix with data `K' = P/(-dt)`). Derived in item 2 of
/// `balance_bound_history`.
fn c_step() -> f64 {
    kappa_t() + 3.0 * U / (1.0 - 3.0 * U)
}

/// Monotone decrease of an error sequence on successive refinements, with no order band.
/// The documents state the orders (second in space, first in time) as observed, not as
/// bands (Document 2, P8b, P16, P17: OBSERVED), and the plus-or-minus 0.2 bands of the
/// finite-volume work are withdrawn (decision 7). If the reference value carries an
/// uncertainty `floor` (series truncation), the true errors can differ from the measured ones
/// by `floor` each, so a true decrease implies `err_next <= err + 2 floor`; with `floor = 0`
/// the decrease is strict.
fn assert_decreasing(errors: &[f64], floor: f64, label: &str) {
    for (k, w) in errors.windows(2).enumerate() {
        let ok = if floor == 0.0 {
            w[1] < w[0]
        } else {
            w[1] <= w[0] + 2.0 * floor
        };
        assert!(ok, "{label}: error {} -> {} at refinement {k}", w[0], w[1]);
    }
}

/// Exact running sum of doubles (Shewchuk's grow-expansion): the partials are non-overlapping
/// and hold the exact value of everything added so far.
#[derive(Clone, Default)]
struct Expansion {
    partials: Vec<f64>,
}

impl Expansion {
    fn add(&mut self, x0: f64) {
        let mut x = x0;
        let mut i = 0;
        for j in 0..self.partials.len() {
            let mut y = self.partials[j];
            if x.abs() < y.abs() {
                std::mem::swap(&mut x, &mut y);
            }
            let hi = x + y;
            let lo = y - (hi - x);
            if lo != 0.0 {
                self.partials[i] = lo;
                i += 1;
            }
            x = hi;
        }
        self.partials.truncate(i);
        self.partials.push(x);
    }

    /// `|reported + (exact value held)|`; with the held value `-sum x_j` this is the exact
    /// accumulation error of `reported`. The m partials are summed from the largest; recursive
    /// summation of m terms satisfies |fl(sum) - sum| <= gamma_{m-1} sum |p_k| (each term
    /// meets at most m - 1 additions, each a factor 1 + d with |d| <= u), so the exact
    /// magnitude is at most |fl(sum)| + gamma_{m-1} sum |p_k|, which is returned (the
    /// evaluation of this bound itself is taken as exact to O(u), see the header).
    fn magnitude_with(&self, reported: f64) -> f64 {
        let mut e = self.clone();
        e.add(reported);
        let approx = e.partials.iter().rev().sum::<f64>().abs();
        let abs_sum: f64 = e.partials.iter().map(|p| p.abs()).sum();
        approx + gamma(e.partials.len().saturating_sub(1) as f64) * abs_sum
    }
}

// ------------------------------------------------------------------------------------------
// A, C1: five-layer synthetic case against the Sturm-Liouville series; spatial convergence
// ------------------------------------------------------------------------------------------

#[test]
fn five_layer_release_vs_sturm_liouville_and_spatial_order() {
    let times = [5.0e3, 2.0e4, 5.0e4, 1.0e5, 2.0e5];
    let base = s5(20, 25.0);
    let series = ModalRelease::new(&layers(&base), 45.0 / times[0], 20_000).unwrap();
    assert!(series.min_valid_time() <= times[0]);
    let floor = series.accuracy_floor();
    report("A", "series_accuracy_floor", floor);
    let exact: Vec<f64> = times.iter().map(|&t| series.release_fraction(t)).collect();
    // Independent high-precision value of F(2e5 s) recorded in `reconciliation.md` (item 18),
    // given to 14 decimals: its own rounding is at most 5e-15. The series is allowed its own
    // accuracy estimate (`accuracy_floor`, the module's estimate; its documentation says the
    // true error stayed 10 to 20 times below it). No other slack.
    let oracle = 0.43255563909304;
    assert!(
        (exact[4] - oracle).abs() <= floor + 5.0e-15,
        "series vs oracle: {} vs {oracle}",
        exact[4]
    );

    let mut failures: Vec<String> = Vec::new();
    let mut f_by_grid: Vec<Vec<f64>> = Vec::new();
    for &n in &GRIDS {
        f_by_grid.push(f_time_extrapolated(&s5(n, 25.0), &times));
    }
    for (ti, &t) in times.iter().enumerate() {
        let errs: Vec<f64> = GRIDS
            .iter()
            .enumerate()
            .map(|(gi, _)| (f_by_grid[gi][ti] - exact[ti]).abs())
            .collect();
        for (gi, &n) in GRIDS.iter().enumerate() {
            report("A", &format!("abs_error_t{t:.0e}_n{n}"), errs[gi]);
            report("A", &format!("F_btcs_t{t:.0e}_n{n}"), f_by_grid[gi][ti]);
        }
        report("A", &format!("F_series_t{t:.0e}"), exact[ti]);
        let ords = orders(&errs);
        for (k, o) in ords.iter().enumerate() {
            report("C1", &format!("order_t{t:.0e}_pair{k}"), *o);
        }
        // R2 on the finest grid.
        let n_last = GRIDS.len() - 1;
        let p = ((f_by_grid[n_last - 2][ti] - f_by_grid[n_last - 1][ti]).abs()
            / (f_by_grid[n_last - 1][ti] - f_by_grid[n_last][ti]).abs())
        .log2();
        let e_h =
            1.25 * (f_by_grid[n_last - 1][ti] - f_by_grid[n_last][ti]).abs() / (2f64.powf(p) - 1.0);
        report("A", &format!("richardson_estimate_t{t:.0e}"), e_h);
        report("A", &format!("richardson_order_t{t:.0e}"), p);
        // No fixed target and no order band (decision 7): the error must fall at every
        // refinement (up to the reference's own estimated accuracy), and the observed orders
        // and the Richardson estimate are reported as OBSERVED values.
        if errs.iter().all(|&e| e > floor) {
            for w in errs.windows(2).enumerate() {
                if w.1[1] > w.1[0] + 2.0 * floor {
                    failures.push(format!(
                        "t={t}: error {} -> {} at refinement {}",
                        w.1[0], w.1[1], w.0
                    ));
                }
            }
        } else {
            report("C1", &format!("unresolved_t{t:.0e}"), 1.0);
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

// ------------------------------------------------------------------------------------------
// B: homogeneous sphere against the independent analytical series
// ------------------------------------------------------------------------------------------

fn homogeneous_cfg(n: usize, dt: f64, end: f64) -> Config {
    let mut cfg = s5(n, dt);
    for r in cfg.regions.iter_mut() {
        r.diffusivity_m2_s = 1.0e-10;
    }
    cfg.time.end_time_s = end;
    cfg
}

#[test]
fn homogeneous_sphere_vs_analytical_reference() {
    let d = 1.0e-10;
    let r5 = 4.5e-4;
    let r1 = 2.0e-4;
    let sphere = HomogeneousSphere::core(r5, r1, d, 1.0).unwrap();
    let tau_scale = r5 * r5 / d; // seconds per unit of D t / R^2
    let taus = [0.002, 0.01, 0.05, 0.2, 0.5];
    let end = 0.5 * tau_scale;
    let dt0 = end / 2000.0;
    let times: Vec<f64> = taus.iter().map(|tau| tau * tau_scale).collect();
    // The series reports a bound of its own truncation error with every value
    // (`Estimate::error_bound`); the rounding of the series evaluation is not bounded by the
    // module and is far below every error asserted on here (those are above 1e-8 relative to
    // the value; F itself is at most 1, and the absolute release-rate errors are of order
    // 1e-18 to 1e-21 mol/s, which is 6e-4 to 1.6e-6 of the rate).
    let estimates: Vec<_> = times
        .iter()
        .map(|&t| sphere.release_fraction(t, 1e-14).unwrap())
        .collect();
    let exact: Vec<f64> = estimates.iter().map(|e| e.value).collect();
    let mut f_by_grid: Vec<Vec<f64>> = Vec::new();
    for &n in &GRIDS {
        f_by_grid.push(f_time_extrapolated(&homogeneous_cfg(n, dt0, end), &times));
    }
    for (ti, &tau) in taus.iter().enumerate() {
        let errs: Vec<f64> = (0..GRIDS.len())
            .map(|gi| (f_by_grid[gi][ti] - exact[ti]).abs())
            .collect();
        report("B", &format!("F_exact_tau{tau}"), exact[ti]);
        for (gi, &n) in GRIDS.iter().enumerate() {
            report("B", &format!("abs_error_tau{tau}_n{n}"), errs[gi]);
        }
        let ords = orders(&errs);
        for (k, o) in ords.iter().enumerate() {
            report("B", &format!("order_tau{tau}_pair{k}"), *o);
        }
        // Asserted for the resolved, late-time part; earlier taus are printed as diagnostics
        // (F is below 1e-3 or the BE time error at 8 to 40 steps is not removed; decided
        // before the first run).
        if tau >= 0.05 {
            // No fixed target and no order band (decision 7); see `assert_decreasing`.
            assert_decreasing(&errs, estimates[ti].error_bound, &format!("B tau={tau}"));
        }
    }

    // Profile and release rate at tau = 0.5 on the finest grid, observed only (P12, P17).
    let t = 0.5 * tau_scale;
    let mut prof_err = Vec::new();
    let mut rate_err = Vec::new();
    let rate_est = sphere.surface_release_rate(t, 1e-14).unwrap();
    let rate_exact = rate_est.value;
    let mut prof_bound: f64 = 0.0;
    for &n in &GRIDS {
        let mut prof = Vec::new();
        let mut rate = Vec::new();
        let mut grid = None;
        for k in [1.0, 2.0, 4.0] {
            let mut cfg = homogeneous_cfg(n, dt0 / k, end);
            cfg.time.output_every_steps = 1;
            let r = run(&cfg, &LIMITS, &[t]).unwrap();
            let step = (t / r.meta.dt).round() as usize;
            rate.push(r.series[step].1.release_rate.unwrap());
            prof.push(r.snapshots[0].1.clone());
            grid = Some(r.grid);
        }
        let grid = grid.unwrap();
        let nn = grid.r.len();
        let mut linf: f64 = 0.0;
        for i in 0..nn {
            let c = richardson(prof[0][i], prof[1][i], prof[2][i]);
            let est = sphere.concentration(grid.r[i], t, 1e-14).unwrap();
            prof_bound = prof_bound.max(est.error_bound);
            linf = linf.max((c - est.value).abs());
        }
        prof_err.push(linf);
        rate_err.push((richardson(rate[0], rate[1], rate[2]) - rate_exact).abs());
    }
    for (gi, &n) in GRIDS.iter().enumerate() {
        report(
            "B",
            &format!("profile_linf_error_tau0.5_n{n}"),
            prof_err[gi],
        );
        report(
            "B",
            &format!("release_rate_abs_error_tau0.5_n{n}"),
            rate_err[gi],
        );
    }
    for (k, o) in orders(&prof_err).iter().enumerate() {
        report("B", &format!("profile_linf_order_pair{k}"), *o);
    }
    for (k, o) in orders(&rate_err).iter().enumerate() {
        report("B", &format!("release_rate_order_pair{k}"), *o);
    }
    report("B", "release_rate_exact_tau0.5_mol_s", rate_exact);
    assert_decreasing(&prof_err, prof_bound, "B profile Linf");
    assert_decreasing(&rate_err, rate_est.error_bound, "B release rate");
}

// ------------------------------------------------------------------------------------------
// C3: smooth single-mode solution in one material, profile errors in two norms
// ------------------------------------------------------------------------------------------

fn sinc_mode(r: f64, radius: f64) -> f64 {
    if r == 0.0 {
        1.0
    } else {
        let x = PI * r / radius;
        x.sin() / x
    }
}

/// Runs the Stepper from the single-mode start for `steps` steps; returns the final profile.
fn smooth_run(grid: &Grid, d: f64, dt: f64, steps: usize, radius: f64) -> Vec<f64> {
    let mut stepper = Stepper::new(grid, [d; 5], dt).unwrap();
    let mut c_old: Vec<f64> = grid.r.iter().map(|&r| sinc_mode(r, radius)).collect();
    c_old[grid.n] = 0.0;
    let mut c_new = vec![0.0; grid.n + 1];
    for _ in 0..steps {
        stepper.step(&c_old, &mut c_new).unwrap();
        std::mem::swap(&mut c_old, &mut c_new);
    }
    c_old
}

#[test]
fn smooth_mode_spatial_order_and_temporal_order() {
    let d = 1.0e-10;
    let radius = 4.5e-4;
    let radii = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4];
    let t_end = 0.05 * radius * radius / d;
    let decay = (-PI * PI * d * t_end / (radius * radius)).exp();
    // Spatial: time error removed by Richardson on the profile.
    let mut linf = Vec::new();
    let mut l2 = Vec::new();
    for &n in &GRIDS {
        let grid = Grid::from_radii(radii, [n; 5]).unwrap();
        let steps0 = 200usize;
        let dt0 = t_end / steps0 as f64;
        let p1 = smooth_run(&grid, d, dt0, steps0, radius);
        let p2 = smooth_run(&grid, d, dt0 / 2.0, 2 * steps0, radius);
        let p4 = smooth_run(&grid, d, dt0 / 4.0, 4 * steps0, radius);
        let mut m: f64 = 0.0;
        let mut s2 = 0.0;
        for i in 0..=grid.n {
            let c = richardson(p1[i], p2[i], p4[i]);
            let e = (c - decay * sinc_mode(grid.r[i], radius)).abs();
            m = m.max(e);
            if i >= 1 && i < grid.n {
                s2 += grid.v[i] * e * e;
            }
        }
        linf.push(m);
        l2.push(s2.sqrt());
    }
    for (gi, &n) in GRIDS.iter().enumerate() {
        report("C3", &format!("linf_error_n{n}"), linf[gi]);
        report("C3", &format!("weighted_l2_error_n{n}"), l2[gi]);
    }
    let o_inf = orders(&linf);
    let o_l2 = orders(&l2);
    for (k, o) in o_inf.iter().enumerate() {
        report("C3", &format!("linf_order_pair{k}"), *o);
    }
    for (k, o) in o_l2.iter().enumerate() {
        report("C3", &format!("l2_order_pair{k}"), *o);
    }
    // No order band (decision 7): strict decrease at every refinement. The exact reference
    // sinc(...) exp(...) is evaluated to a few u in magnitude (below 1e-15), against errors
    // of at least 4e-7 (Linf) and 6e-13 (weighted L2, whose scale is the square root of the
    // particle volume, so a reference rounding of 1e-15 contributes below 1e-20).
    assert_decreasing(&linf, 0.0, "smooth Linf");
    assert_decreasing(&l2, 0.0, "smooth L2");

    // D2: temporal order on one grid, Cauchy ratio of successive profiles in the max norm.
    let grid = Grid::from_radii(radii, [40; 5]).unwrap();
    let step_counts = [10usize, 20, 40, 80, 160];
    let profiles: Vec<Vec<f64>> = step_counts
        .iter()
        .map(|&s| smooth_run(&grid, d, t_end / s as f64, s, radius))
        .collect();
    let diffs: Vec<f64> = profiles
        .windows(2)
        .map(|w| {
            w[0].iter()
                .zip(&w[1])
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f64::max)
        })
        .collect();
    for (k, v) in diffs.iter().enumerate() {
        report("D2", &format!("cauchy_diff_{k}"), *v);
    }
    let ords = orders(&diffs);
    for (k, o) in ords.iter().enumerate() {
        report("D2", &format!("cauchy_order_{k}"), *o);
    }
    // The Cauchy differences of successive step halvings fall strictly; their ratio is the
    // OBSERVED order printed above (first order in time is stated in Document 1, 9.5, not
    // as a band).
    assert_decreasing(&diffs, 0.0, "smooth temporal Cauchy differences");
}

// ------------------------------------------------------------------------------------------
// D1: temporal convergence on the five-layer synthetic case, same-grid Cauchy ratio
// ------------------------------------------------------------------------------------------

#[test]
fn five_layer_temporal_order_cauchy() {
    let times = [2.0e4, 1.0e5, 2.0e5];
    let dts = [800.0, 400.0, 200.0, 100.0, 50.0, 25.0];
    let base = s5(80, 200.0);
    let series = ModalRelease::new(&layers(&base), 45.0 / times[0], 20_000).unwrap();
    let f: Vec<Vec<f64>> = dts
        .iter()
        .map(|&dt| {
            let mut c = base.clone();
            c.time.dt_s = dt;
            f_at(&c, &times)
        })
        .collect();
    for (ti, &t) in times.iter().enumerate() {
        let diffs: Vec<f64> = f.windows(2).map(|w| (w[0][ti] - w[1][ti]).abs()).collect();
        for (k, v) in diffs.iter().enumerate() {
            report("D1", &format!("cauchy_diff_t{t:.0e}_{k}"), *v);
        }
        let ords = orders(&diffs);
        for (k, o) in ords.iter().enumerate() {
            report("D1", &format!("cauchy_order_t{t:.0e}_{k}"), *o);
        }
        for (k, row) in f.iter().enumerate() {
            report(
                "D1",
                &format!("error_vs_series_t{t:.0e}_dt{}", dts[k]),
                (row[ti] - series.release_fraction(t)).abs(),
            );
        }
        assert_decreasing(&diffs, 0.0, &format!("temporal Cauchy differences t={t}"));
    }
}

// ------------------------------------------------------------------------------------------
// E: interfaces
// ------------------------------------------------------------------------------------------

/// Outward flow from one-sided three-point gradients, -4 pi R^2 D dC/dr, both sides.
fn one_sided_fluxes(grid: &Grid, d: &[f64; 5], c: &[f64], i: usize) -> (f64, f64) {
    let radius = grid.r[i];
    let region_in = grid.region_of_interval[i - 1].index();
    let region_out = grid.region_of_interval[i].index();
    let (dr_in, dr_out) = (grid.dr_inner(i), grid.dr_outer(i));
    let grad_in = (3.0 * c[i] - 4.0 * c[i - 1] + c[i - 2]) / (2.0 * dr_in);
    let grad_out = (-3.0 * c[i] + 4.0 * c[i + 1] - c[i + 2]) / (2.0 * dr_out);
    let w = -4.0 * PI * radius * radius;
    (w * d[region_in] * grad_in, w * d[region_out] * grad_out)
}

#[test]
fn interfaces_concentration_and_flux_continuity() {
    let times = [2.0e4, 1.0e5, 2.0e5];
    let base = s5(20, 200.0);
    let d = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13];
    let mut jumps: Vec<Vec<Vec<f64>>> = Vec::new(); // [time][interface][grid]
    jumps.resize(times.len(), vec![Vec::new(); 4]);
    let mut worst_identity: f64 = 0.0;
    for &n in &GRIDS {
        // Profiles at the three times for dt, dt/2, dt/4 (Richardson on the profile).
        let mut profs: Vec<Vec<Vec<f64>>> = Vec::new();
        let mut grid = None;
        for k in [1.0, 2.0, 4.0] {
            let mut cfg = s5(n, 200.0 / k);
            cfg.time = base.time.clone();
            cfg.time.dt_s = 200.0 / k;
            let r = run(&cfg, &LIMITS, &times).unwrap();
            profs.push(r.snapshots.iter().map(|s| s.1.clone()).collect());
            grid = Some(r.grid);
        }
        let grid = grid.unwrap();
        for (ti, _) in times.iter().enumerate() {
            let c: Vec<f64> = (0..=grid.n)
                .map(|i| richardson(profs[0][ti][i], profs[1][ti][i], profs[2][ti][i]))
                .collect();
            for (k, &node) in grid.interface_nodes.iter().enumerate() {
                let (qm, qp) = one_sided_fluxes(&grid, &d, &c, node);
                jumps[ti][k].push((qp - qm).abs() / (qp.abs() + qm.abs()));
            }
        }
        // F31 identity at rounding level, on the dt = 200 s run, at the last step.
        let cfg = s5(n, 200.0);
        let r = run(&cfg, &LIMITS, &[199_800.0, 200_000.0]).unwrap();
        let (c_old, c_new) = (&r.snapshots[0].1, &r.snapshots[1].1);
        let coeff = Coefficients::new(&r.grid, d, 200.0).unwrap();
        for &i in &r.grid.interface_nodes {
            let (fi, fo) = interface_flux_both_sides(&r.grid, &coeff, c_old, c_new, 200.0, i);
            let radius = r.grid.r[i];
            let scale = 4.0
                * PI
                * (coeff.k_outer[i - 1] * (c_new[i - 1].abs() + c_new[i].abs())
                    + coeff.k_outer[i] * (c_new[i].abs() + c_new[i + 1].abs())
                    + (radius * radius * (r.grid.dr_inner(i) + r.grid.dr_outer(i)) / 2.0 / 200.0)
                        * (c_new[i].abs() + c_old[i].abs()));
            // Derived bound: fi - fo is 4 pi times the residual of row (F10) at node i
            // (the two halves of V_i add to r^2 (a + b)/2, which the solver stores with 3
            // roundings). In units of `scale` (which bounds the (F10) scale S, times 4 pi):
            //  - the exact-form row residual of the computed x: c_step (backward error of the
            //    solve plus the stored entries, `c_step`);
            //  - r^2 (a + b)/2 against the stored V_i: 3 roundings (gamma_3); a fourth rounding
            //    is kept in gamma_4 as slack: the stored-entry roundings are already in `c_step`
            //    (item 2 of `balance_bound_history`, no modified V), so it is not needed;
            //  - the shared product dt K against K itself: gamma_1;
            //  - evaluating each of fi and fo: k (c - c) 2 roundings; half = r r a / 2 and
            //    (c_new - c_old)/dt (2) and their product: 5; the difference of the two terms
            //    and the factor 4 pi: 2; gamma_7 each, and the two evaluations together
            //    use at most the whole scale.
            let tol = (c_step() + gamma(4.0) + gamma(1.0) + gamma(7.0)) * scale;
            assert!(
                (fi - fo).abs() <= tol,
                "F31 n={n} node {i}: {fi} vs {fo} tol {tol}"
            );
            worst_identity = worst_identity.max((fi - fo).abs() / tol);
        }
        // Concentration continuity is structural: one unknown per interface node.
        assert_eq!(r.final_profile.len(), r.grid.n + 1);
    }
    report("E", "worst_F31_difference_over_tolerance", worst_identity);
    let names = [
        "R1_kernel_buffer",
        "R2_buffer_IPyC",
        "R3_IPyC_SiC",
        "R4_SiC_OPyC",
    ];
    for (ti, &t) in times.iter().enumerate() {
        for k in 0..4 {
            let js = &jumps[ti][k];
            for (gi, &n) in GRIDS.iter().enumerate() {
                report("E1", &format!("jump_{}_t{t:.0e}_n{n}", names[k]), js[gi]);
            }
            let ords = orders(js);
            for (j, o) in ords.iter().enumerate() {
                report("E1", &format!("order_{}_t{t:.0e}_pair{j}", names[k]), *o);
            }
            // The relative jump of the independent one-sided fluxes falls at every halving
            // (no factor is asserted; the second-order rate is OBSERVED only, property P8b,
            // for which Document 4 records "no assertion of an order").
            assert_decreasing(js, 0.0, &format!("flux jump {} t={t}", names[k]));
        }
    }
}

// ------------------------------------------------------------------------------------------
// F: balance identity;  G: positivity;  matrix structure
// ------------------------------------------------------------------------------------------

/// A-posteriori rounding bound on `|eps^j| / M^0` for every row `j = 0 ..= steps`, and the
/// per-step accumulating part of it.
struct BalanceBound {
    /// Bound on `|eps^j| / M^0`.
    bound: Vec<f64>,
    /// Bound on the part of `M^j - M^{j-1}` that is not the release (relative to `M^0`),
    /// `0` for `j = 0`.
    step_term: Vec<f64>,
}

/// Rounding analysis (Document 2 7.4 and Document 3 RP-4 ask that the mesh Fourier numbers and
/// the step count enter). Notation: u = 2^-53, gamma_k = k u/(1 - k u); `x` is the computed
/// profile of a step, `c` the old one; pi stands for the double `PI` used by the solver.
///
/// 1. Exact identity. Rows (F10) with data `K'` (defined in 2) and the stored `V`, multiplied by
///    `V`, summed over `i = 1 ..= N-1` telescope (`K_0 = 0`, `x_N = 0`):
///    `sum V (x - c) = -dt K'_{N-1} x_{N-1} + sum V rho`, with `rho = (A' x - c)` the
///    residual of the exact-form matrix `A'` (entries `P/V`, diagonal `1 + |L'| + |U'|`). Hence
///    `M(x) - M(c) + dt Phi'(x) = 4 pi sum_i V_i rho_i` exactly, where
///    `M` is `4 pi sum V x` and `Phi' = 4 pi K' x_{N-1}`.
/// 2. Stored matrix. `assemble` forms `P = fl(-dt K)` (one rounding, the same operands in the
///    two rows that use `K_i`), `L = fl(P/V)`, `U = fl(P/V)`, `B = fl(fl(1 - L) - U)`. Put
///    `K' = P/(-dt)` (relative change u from K); then `A'` has `L' = P_{i-1}/V_i`,
///    `U' = P_i/V_i`, `B' = 1 + |L'| + |U'|`. Stored against exact-form: `L` and `U` each carry
///    one rounding (`|L - L'| <= u|L'|`, same for U); `B` carries two roundings of positive
///    terms against `1 + |L| + |U|` (`gamma_2 B`) and the change of `|L| + |U|` against
///    `|L'| + |U'|` adds `u(|L| + |U|)/(1 - u) <= u B/(1 - u)^3`. So an off-diagonal entry
///    differs by at most `u/(1 - u)` of its stored magnitude, the diagonal entry by at most
///    `(2u + u^2)/(1 - u)^2 B + u B/(1 - u)^3 = (3u + 8u^2 + O(u^3)) B`, and
///    `|(A - A') x|_i <= 3u/(1 - 3u) (|A||x|)_i`. No modified `V` is needed.
/// 3. Solve. The computed `x` satisfies `(A + dA) x = c` with `|dA| <= kappa_t |A|`,
///    `kappa_t = 4u + O(u^2)` (derived in `numerical::tridiagonal::tests::thomas_matches_dense_f24`).
///    Together: `|rho_i| <= c_step (|A||x|)_i`, `c_step = kappa_t + 3u/(1 - 3u)` (about 7u),
///    so per step `|M(x) - M(c) + dt Phi'| <= 4 pi c_step sum_i V_i (|A||x|)_i (1 + gamma_{N+6})`
///    (the evaluation of the sum of N terms needs gamma_{N+5} at most; the larger factor kept
///    here was written for the earlier modified-`V` form and is left as slack, not tightened).
/// 4. What the solver reports differs from the exact quantities by: `M` (N - 1 products, N - 2
///    additions, the factor 4 pi: gamma_N; the factor gamma_{N+1} kept here again exceeds that by
///    one rounding): gamma_{N+1} M, at
///    `j` and at `0`; the increments `dt Phi` (3 products, with `K'` against `K`: u):
///    gamma_4 per increment, so gamma_4 times the cumulative release; the compensated sum of
///    the increments against their exact sum: the *measured* exact accumulation error `E_K`
///    (evaluated with exact expansion arithmetic from the same increments, so no assumption is
///    made on the Kahan constant; Goldberg, Theorem 8, gives `2u + O(N u^2)` for it); and the
///    residual `fl(fl(M + R) - M^0)`: one rounding of `M + R` (the subtraction is exact
///    within a factor 2 of its operands and otherwise rounds the small result), so u (M + R),
///    divided by (1 - u) because the result itself rounds.
/// 5. The per-step terms are added in the order of the steps (`steps` additions): the factor
///    (1 + gamma_{steps + 8}) on their sum.
///
/// None of this is tuned: no safety factor is applied, and the observed ratio is printed. The
/// bound replaces the heuristic `u N sqrt(steps)` of the verification plan, which omits Fo (it
/// failed at n = 20 by a factor 65; recorded in the results file), and the earlier constant
/// 16u, which had no derivation.
fn balance_bound_history(cfg: &Config) -> BalanceBound {
    let d = [
        cfg.regions[0].diffusivity_m2_s,
        cfg.regions[1].diffusivity_m2_s,
        cfg.regions[2].diffusivity_m2_s,
        cfg.regions[3].diffusivity_m2_s,
        cfg.regions[4].diffusivity_m2_s,
    ];
    let r = run(cfg, &LIMITS, &[]).unwrap();
    assert_eq!(r.series.len(), r.meta.steps + 1, "output every step");
    let grid = &r.grid;
    let dt = r.meta.dt;
    let m0 = r.meta.initial_inventory;
    let n_nodes = grid.n as f64;
    let steps = r.meta.steps as f64;
    let mut stepper = Stepper::new(grid, d, dt).unwrap();
    let (mut c_old, _, _) = initial_vector(grid, cfg.initial.kernel_concentration_mol_m3);
    let mut c_new = vec![0.0; grid.n + 1];
    let g_inv = gamma(n_nodes + 1.0) / (1.0 - gamma(n_nodes + 1.0));
    let g_inc = gamma(4.0) / (1.0 - gamma(4.0));
    let mut neg_increments = Expansion::default();
    let mut cum = 0.0;
    let row0 = r.series[0].1;
    let first = (g_inv * (row0.inventory + m0) + U * (row0.inventory + row0.cumulative_release))
        / (1.0 - U)
        / m0;
    let mut bound = vec![first];
    let mut step_term = vec![0.0];
    for (_, (_, row)) in (1..=r.meta.steps).zip(r.series.iter().skip(1)) {
        stepper.step(&c_old, &mut c_new).unwrap();
        let increment = dt
            * strato::numerical::boundary_conditions::release_rate(
                stepper.k_last(),
                c_new[grid.n - 1],
            );
        neg_increments.add(-increment);
        let a = &stepper.assembly().matrix;
        let mut s = 0.0;
        for i in 1..grid.n {
            let left = a.lower[i].abs() * c_new[i - 1].abs();
            let right = a.upper[i].abs() * c_new[i + 1].abs();
            s += grid.v[i] * (left + a.diag[i].abs() * c_new[i].abs() + right);
        }
        let term = 4.0 * PI * c_step() * s * (1.0 + gamma(n_nodes + 6.0)) / m0;
        cum += term;
        step_term.push(term);
        let e_k = neg_increments.magnitude_with(row.cumulative_release);
        let rounding = g_inv * (row.inventory + m0)
            + g_inc * row.cumulative_release
            + e_k
            + U * (row.inventory + row.cumulative_release);
        bound.push(cum * (1.0 + gamma(steps + 8.0)) + rounding / (1.0 - U) / m0);
        std::mem::swap(&mut c_old, &mut c_new);
    }
    BalanceBound { bound, step_term }
}

fn check_balance(label: &str, cfg: &Config) {
    let r = run(cfg, &LIMITS, &[]).unwrap();
    let bounds = balance_bound_history(cfg).bound;
    assert_eq!(bounds.len(), r.series.len());
    let m0 = r.meta.initial_inventory;
    let mut worst_ratio: f64 = 0.0;
    let mut worst_rel: f64 = 0.0;
    for (k, (_, row)) in r.series.iter().enumerate() {
        let rel = row.balance_residual.abs() / m0;
        worst_rel = worst_rel.max(rel);
        worst_ratio = worst_ratio.max(rel / bounds[k]);
        assert!(
            rel <= bounds[k],
            "{label}: step {k}: residual {rel} exceeds bound {}",
            bounds[k]
        );
    }
    let heuristic = U * (r.grid.n as f64 + 1.0) * (r.meta.steps as f64).sqrt().max(1.0);
    report("F", &format!("worst_relative_residual_{label}"), worst_rel);
    report(
        "F",
        &format!("final_bound_{label}"),
        *bounds.last().unwrap(),
    );
    report(
        "F",
        &format!("worst_observed_over_bound_{label}"),
        worst_ratio,
    );
    report("F", &format!("plan_heuristic_bound_R4_{label}"), heuristic);
    report(
        "F",
        &format!("worst_over_plan_heuristic_{label}"),
        worst_rel / heuristic,
    );
}

#[test]
fn balance_identity_to_rounding_every_step() {
    for &n in &GRIDS {
        check_balance(&format!("n{n}"), &s5(n, 200.0));
    }
    // One single step with a very large dt (Fo up to 1e11), and a high-contrast case.
    let mut single = s5(40, 2.0e5);
    single.time.end_time_s = 2.0e5;
    check_balance("single_step", &single);
    let mut contrast = s5(40, 200.0);
    let dd = [1.0e-14, 1.0e-10, 1.0e-14, 1.0e-10, 1.0e-14];
    for (reg, d) in contrast.regions.iter_mut().zip(dd) {
        reg.diffusivity_m2_s = d;
    }
    check_balance("contrast_1e4", &contrast);
}

#[test]
fn production_balance_bound_equals_the_test_derivation() {
    // The bound reported in every output row of the run (`BalanceBound`) is the same quantity
    // as the history derived above, accumulated in the same order; it is not a second formula.
    for n in [20usize, 40] {
        let cfg = s5(n, 200.0);
        let r = run(&cfg, &LIMITS, &[]).unwrap();
        let history = balance_bound_history(&cfg);
        let history = history.bound;
        assert_eq!(history.len(), r.series.len());
        let m0 = r.meta.initial_inventory;
        for (k, (_, row)) in r.series.iter().enumerate() {
            // The same operations in the same order; the run stores the bound in mol (one
            // multiplication by m0) and it is divided by m0 here (one division): relative
            // difference at most gamma_2.
            let rel = row.balance_residual_bound / m0;
            assert!(
                (rel - history[k]).abs() <= gamma(2.0) * history[k],
                "n={n} row {k}: {rel} vs {}",
                history[k]
            );
            // The observed residual is reported beside the bound and lies below it.
            assert!(row.balance_residual.abs() <= row.balance_residual_bound);
        }
    }
}

#[test]
fn balance_negative_control_wrong_release_level() {
    // Release computed from the OLD level C_{N-1}^j gives a residual of discretisation size,
    // not rounding size (D2 Section 7.2).
    let cfg = s5(40, 200.0);
    let r0 = run(&cfg, &LIMITS, &[]).unwrap();
    let grid = &r0.grid;
    let mut stepper =
        Stepper::new(grid, [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13], 200.0).unwrap();
    let (mut c_old, _, m0) = initial_vector(grid, 1.0);
    let mut c_new = vec![0.0; grid.n + 1];
    let mut released_old_level = 0.0;
    let mut released_new_level = 0.0;
    for _ in 0..1000 {
        stepper.step(&c_old, &mut c_new).unwrap();
        released_old_level += 200.0 * 4.0 * PI * stepper.k_last() * c_old[grid.n - 1];
        released_new_level += 200.0 * 4.0 * PI * stepper.k_last() * c_new[grid.n - 1];
        std::mem::swap(&mut c_old, &mut c_new);
    }
    let m = inventory(grid, &c_old);
    let eps_wrong = ((m + released_old_level - m0) / m0).abs();
    let eps_right = ((m + released_new_level - m0) / m0).abs();
    report("F", "negative_control_residual_old_level", eps_wrong);
    report("F", "residual_new_level_plain_sum", eps_right);
    // The control must exceed the derived rounding bound of the correct identity, which is
    // the statement that the identity is sensitive to the release rule.
    let bound = balance_bound_history(&cfg);
    let final_bound = *bound.bound.last().unwrap();
    assert!(
        eps_wrong > final_bound,
        "negative control {eps_wrong} does not exceed the bound {final_bound}"
    );
}

#[test]
fn positivity_maximum_principle_and_monotone_release() {
    let stress_dd = [1.0e-14, 1.0e-10, 1.0e-14, 1.0e-10, 1.0e-14];
    for (label, n, dt, steps, d) in [
        (
            "S5_n40_dt200",
            40usize,
            200.0,
            1000usize,
            [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13],
        ),
        (
            "S5_n40_single_huge_step",
            40,
            2.0e5,
            1,
            [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13],
        ),
        ("contrast_1e4_n40", 40, 200.0, 1000, stress_dd),
        ("contrast_1e4_huge_step", 40, 1.0e6, 3, stress_dd),
    ] {
        let radii = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4];
        let grid = Grid::from_radii(radii, [n; 5]).unwrap();
        let mut stepper = Stepper::new(&grid, d, dt).unwrap();
        let (mut c_old, _, _) = initial_vector(&grid, 1.0);
        let mut c_new = vec![0.0; grid.n + 1];
        let mut min_seen = f64::INFINITY;
        let mut max_excess: f64 = 0.0;
        let b_max = stepper
            .assembly()
            .matrix
            .diag
            .iter()
            .cloned()
            .fold(0.0, f64::max);
        let gamma5 = 5.0 * U / (1.0 - 5.0 * U);
        let g1 = gamma(2.0) / (1.0 - gamma(2.0));
        let kappa = (g1 + 2.0 * gamma5) * b_max;
        // The bound below needs 1 - kappa > 0 (nothing smaller is assumed).
        assert!(
            kappa < 1.0,
            "{label}: kappa {kappa} too large for the bound"
        );
        let mut steps_done: i32 = 0;
        for _ in 0..steps {
            stepper.step(&c_old, &mut c_new).unwrap();
            let rep = check_step_properties(&c_old, &c_new);
            assert!(
                rep.non_negative,
                "{label}: negative concentration {}",
                rep.min_new
            );
            // Derived maximum-principle bound. The computed x solves (A + dA) x = c_old with
            // |dA| <= kappa_t |A| <= gamma_5 |A| (derived in tridiagonal::tests::thomas_matches_dense_f24)
            // and signs preserved. At the node of largest x, row i gives
            // x_max (rowsum_i(A) - sum_j |dA_ij|) <= (A+dA)x ... <= max c_old, where
            // rowsum_i(A) = B - |L| - |U| >= 1 - g1 B_max with g1 = gamma_2/(1 - gamma_2) (B is
            // two roundings of the positive terms 1, |L|, |U|, so B >= (1 + |L| + |U|)(1 - gamma_2))
            // and sum_j |dA_ij| <= gamma_5 (2 B_max - 1) because B = 1 + |L| + |U| up to rounding.
            // Hence per step max x <= max c_old / (1 - kappa), kappa = (g1 + 2 gamma_5) B_max,
            // and after k steps max C <= C_init (1 - kappa)^-k.
            steps_done += 1;
            assert!(
                rep.max_new <= (1.0 - kappa).powi(-steps_done),
                "{label}: max {} exceeds C_init (1-kappa)^-k",
                rep.max_new
            );
            min_seen = min_seen.min(rep.min_new);
            max_excess = max_excess.max(rep.max_principle_excess);
            std::mem::swap(&mut c_old, &mut c_new);
        }
        report("G", &format!("min_concentration_{label}"), min_seen);
        report("G", &format!("max_principle_excess_{label}"), max_excess);
    }
    // F in [0, 1] and monotone on every grid of the refinement sequence (P4).
    for &n in &GRIDS {
        let r = run(&s5(n, 200.0), &LIMITS, &[]).unwrap();
        let rep = check_series(&r.series, r.meta.initial_inventory);
        assert!(rep.release_monotone, "F not monotone n={n}");
        assert!(rep.release_in_range, "F out of [0,1] n={n}");
        // M^{j+1} - M^j = -dt Phi^{j+1} + (rounding discrepancy of the step) with Phi >= 0, so
        // an increase can only be rounding: at most the accumulating per-step term of the balance
        // bound (item 3 of `balance_bound_history`) plus the rounding of the two reported
        // inventories, gamma_{N+1}/(1 - gamma_{N+1}) (M^j + M^{j-1}).
        report(
            "G",
            &format!("max_inventory_increase_rel_n{n}"),
            rep.max_inventory_increase_rel,
        );
        let bounds = balance_bound_history(&s5(n, 200.0));
        let m0 = r.meta.initial_inventory;
        let g_inv = gamma(r.grid.n as f64 + 1.0) / (1.0 - gamma(r.grid.n as f64 + 1.0));
        for k in 1..r.series.len() {
            let (inv_k, inv_p) = (r.series[k].1.inventory, r.series[k - 1].1.inventory);
            let inc = (inv_k - inv_p) / m0;
            let allowed = bounds.step_term[k] + g_inv * (inv_k + inv_p) / m0;
            assert!(
                inc <= allowed,
                "inventory increase {inc} > {allowed} at step {k}, n={n}"
            );
        }
        assert!(rep.min_concentration >= 0.0);
        report(
            "G",
            &format!("series_min_concentration_n{n}"),
            rep.min_concentration,
        );
        report(
            "G",
            &format!("series_max_concentration_n{n}"),
            rep.max_concentration,
        );
    }
}

#[test]
fn matrix_sign_pattern_and_diagonal_dominance_p1() {
    let radii = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4];
    let d = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13];
    for (n, dt) in [
        (5usize, 200.0),
        (40, 200.0),
        (320, 200.0),
        (40, 1.0e6),
        (40, 1.0e-6),
    ] {
        let grid = Grid::from_radii(radii, [n; 5]).unwrap();
        let coeff = Coefficients::new(&grid, d, dt).unwrap();
        let a = assemble(&grid, coeff, dt).unwrap();
        let rep = check_matrix_structure(&a.matrix).unwrap();
        // Derived: B = fl(fl(1 - L) - U) is two roundings of positive terms, so the real margin
        // B - |L| - |U| deviates from 1 by at most gamma_2 B_max (1 + O(u)); `check_matrix_structure`
        // evaluates it in floating point with two subtractions, which adds at most gamma_2 B_max
        // more. So the deviation is at most 2 gamma_2 B_max, and the smallest margin is at least
        // 1 minus that.
        let b_max = a.matrix.diag.iter().cloned().fold(0.0, f64::max);
        let tol = 2.0 * gamma(2.0) * b_max;
        report(
            "P1",
            &format!("max_margin_deviation_n{n}_dt{dt:e}"),
            rep.max_interior_margin_deviation,
        );
        report("P1", &format!("margin_tolerance_n{n}_dt{dt:e}"), tol);
        assert!(rep.max_interior_margin_deviation <= tol);
        assert!(rep.min_row_margin >= 1.0 - tol);
    }
}
