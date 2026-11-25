//! Numerical verification of the five-layer BTCS solver (directive Section 11, tests A to G;
//! Document 2 properties P1 to P18; verification plan rules R1 to R5).
//!
//! These tests check the numerical solution of the stated model against analytical
//! references that contain no BTCS code. They are NOT experimental validation.
//!
//! # Tolerances, fixed before the first run
//!
//! They were fixed before the first run, except the three revisions listed under "Revisions"
//! below and the later replacement of underived bounds by derived ones (R5, and the unit
//! tests of `numerical::btcs`). No tolerance was loosened to make a check pass.
//!
//! * R1 (series floor): a difference is "resolved" only if it exceeds 3 x the accuracy floor
//!   of the reference. `ModalRelease::accuracy_floor` for the layered series; for the
//!   homogeneous series 3 x (1e-14 requested tail tolerance + 1e-15 rounding).
//! * R2 (absolute accuracy): `err <= max(3 floor, E_h)`, with the Richardson estimate
//!   `E_h = 1.25 |F_h - F_{h/2}| / (2^p - 1)`, p the order observed on the last three grids
//!   (1.25 is the usual safety factor of the verification plan); and an absolute convention
//!   `err <= 1e-6` on the finest grid (the value the finite-volume driver used).
//! * R3 (order band): observed order within 0.2 of the theoretical order on the last two
//!   pairs: space 2 (band 1.8 to 2.2), time 1 (band 0.8 to 1.2), when the errors are
//!   resolved. Theory: central differences are second order, Backward Euler first order.
//! * R4 (balance): `|eps^j| / M^0 <=` an a-posteriori rounding bound that contains the mesh
//!   Fourier numbers through `|A||x|` and the step count through the sum over steps (see
//!   `balance_bound_history`); the measured ratio is printed.
//!
//! # Revisions after the first run (recorded, not hidden)
//!
//! Items 1 to 3 followed first-run failures whose cause lay in the test design (a bound
//! without the Fourier number, a step outside the asymptotic range, a slack that left out
//! the rounding of the solve); the replacements are derived, none merely loosened.
//!
//! 1. R4 as first written was `u N sqrt(steps)` (the heuristic of the verification plan). It
//!    failed at n = 20 (observed 2.3e-11 against 3.5e-13) because it contains no Fourier
//!    number, whereas Document 2 7.4 and Document 3 RP-4 require one. It was replaced by
//!    the derived a-posteriori bound above.
//! 2. Test A first used the base step `dt = 200 s` for the extrapolation triple. At
//!    t = 5e3 s (25 steps, F = 1.8e-7, an exponentially small tail) that triple is not in the
//!    asymptotic range: a same-grid step sweep (n = 160) shows F(5e3) = 1.24e-6, 3.4e-7,
//!    2.16e-7, 1.90e-7, 1.84e-7, 1.82e-7 for dt = 200, 50, 12.5, 3.1, 0.78, 0.20 s,
//!    first order in dt. The base step was changed to 25 s (triple 25, 12.5, 6.25 s) for
//!    every time of test A.
//! 3. The inventory-monotonicity check allows a rounding slack, because `M^j` moves by
//!    rounding while the release is still exactly zero. The first slack, `2 (N+1) u`
//!    (rounding of the two sums only), was too small at n = 320 (observed increase 9.3e-13
//!    against 3.6e-13) because it left out the rounding error of the solve itself, which
//!    is the per-step increment of the balance bound of item 1; the slack is that increment
//!    plus `2 (N+1) u`.
//! 4. R5 (below) first read `max C <= C_init (1 + 10 N u)` with no derivation (observed
//!    excess 1.1e-14, far inside it). It was replaced by the derived bound that follows.
//!
//! # Positivity and maximum principle
//!
//! * R5 (positivity): `min C >= 0` exactly (property P6: all operations in the solve add
//!   non-negative numbers), `max C <= C_init (1 - kappa)^-k` after k steps (maximum
//!   principle P3, P4; the rounding bound and `kappa = (4 u + 2 gamma_5) B_max` are derived
//!   in the test).
//! * Time error is removed from space studies by Richardson extrapolation over `dt, dt/2,
//!   dt/4`: `(8 F(dt/4) - 6 F(dt/2) + F(dt)) / 3`, which cancels the `dt` and `dt^2` terms.
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
            assert!(
                (step as f64 * dt - t).abs() < 1e-9 * t,
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

fn assert_band(order: f64, theory: f64, label: &str) {
    assert!(
        (order - theory).abs() <= 0.2,
        "{label}: observed order {order} outside {theory} +- 0.2"
    );
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
    // Python-oracle anchor pinned by the repository for the series itself.
    let oracle = 0.4325556390930383;
    assert!((exact[4] - oracle).abs() <= 1e-10, "series vs oracle");

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
        let finest = errs[n_last];
        if finest > (3.0 * floor).max(e_h) {
            failures.push(format!(
                "R2 at t={t}: err {finest} > max(3 floor {}, E_h {e_h})",
                3.0 * floor
            ));
        }
        if finest > 1e-6 {
            failures.push(format!("absolute target at t={t}: {finest}"));
        }
        // R3 on the last two pairs, only if both errors are resolved (R1).
        let resolved = errs[n_last - 2..].iter().all(|&e| e > 3.0 * floor);
        if resolved {
            for (which, o) in [
                ("last", ords[ords.len() - 1]),
                ("prev", ords[ords.len() - 2]),
            ] {
                if (o - 2.0).abs() > 0.2 {
                    failures.push(format!("spatial order t={t} {which}: {o} outside 2 +- 0.2"));
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
    let floor: f64 = 3.0 * (1e-14 + 1e-15);
    let exact: Vec<f64> = times
        .iter()
        .map(|&t| sphere.release_fraction(t, 1e-14).unwrap().value)
        .collect();
    let mut f_by_grid: Vec<Vec<f64>> = Vec::new();
    for &n in &GRIDS {
        f_by_grid.push(f_time_extrapolated(&homogeneous_cfg(n, dt0, end), &times));
    }
    let n_last = GRIDS.len() - 1;
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
            let p = ((f_by_grid[n_last - 2][ti] - f_by_grid[n_last - 1][ti]).abs()
                / (f_by_grid[n_last - 1][ti] - f_by_grid[n_last][ti]).abs())
            .log2();
            let e_h = 1.25 * (f_by_grid[n_last - 1][ti] - f_by_grid[n_last][ti]).abs()
                / (2f64.powf(p) - 1.0);
            assert!(
                errs[n_last] <= floor.max(e_h),
                "R2 tau={tau}: err {} E_h {e_h}",
                errs[n_last]
            );
            assert!(errs[n_last] <= 1e-6, "absolute target tau={tau}");
            if errs[n_last - 2..].iter().all(|&e| e > floor) {
                assert_band(ords[ords.len() - 1], 2.0, &format!("B tau={tau} last"));
                assert_band(ords[ords.len() - 2], 2.0, &format!("B tau={tau} prev"));
            }
        }
    }

    // Profile and release rate at tau = 0.5 on the finest grid, observed only (P12, P17).
    let t = 0.5 * tau_scale;
    let mut prof_err = Vec::new();
    let mut rate_err = Vec::new();
    let rate_exact = sphere.surface_release_rate(t, 1e-14).unwrap().value;
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
            let ex = sphere.concentration(grid.r[i], t, 1e-14).unwrap().value;
            linf = linf.max((c - ex).abs());
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
    assert!(prof_err[n_last] < prof_err[0] && rate_err[n_last] < rate_err[0]);
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
    for o in o_inf.iter().rev().take(2) {
        assert_band(*o, 2.0, "smooth Linf");
    }
    for o in o_l2.iter().rev().take(2) {
        assert_band(*o, 2.0, "smooth L2");
    }

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
    for o in ords.iter().rev().take(2) {
        assert_band(*o, 1.0, "smooth temporal");
    }
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
        for o in ords.iter().rev().take(2) {
            assert_band(*o, 1.0, &format!("temporal t={t}"));
        }
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
            let tol = 100.0 * U * scale;
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
            // Pre-registered: the relative jump of the independent one-sided fluxes falls by
            // at least a factor 1.8 per halving over the last two halvings (first order or
            // better; the second-order rate is OBSERVED only, property P8b).
            let nn = js.len();
            assert!(
                js[nn - 2] / js[nn - 1] >= 1.8 && js[nn - 3] / js[nn - 2] >= 1.8,
                "flux jump not decreasing fast enough at {} t={t}: {:?}",
                names[k],
                js
            );
        }
    }
}

// ------------------------------------------------------------------------------------------
// F: balance identity;  G: positivity;  matrix structure
// ------------------------------------------------------------------------------------------

/// A-posteriori rounding bound on `|eps^j| / M^0` after each step `j = 0 ..= steps`.
///
/// Rounding analysis (Document 2 7.4 asks that Fo and the step count enter): the identity
/// (F29) follows by summing the exact-form rows (F10) over `i = 1 ..= N-1`. The computed
/// `x` satisfies the stored rows up to the backward error of tridiagonal elimination,
/// `|delta_i| <= 4u (|A||x|)_i` (Higham, LU of a diagonally dominant tridiagonal matrix),
/// and the stored entries differ from the exact `-dt K / V` and `1 - L - U` by at most
/// `2u` relative (two roundings; `B` by `2u B`). Hence per step
/// `|Delta eps| <= 4 pi (4u + 2u) sum_i V_i (|A||x|)_i`; the constant used is 16u (a safety
/// factor of about 2.7 over 6u). The final rounding of `M^j` (N terms), of `dt Phi`
/// (three products) and of the compensated sum adds `(N + 8) u M^0`. This is fixed before
/// any run and replaces the heuristic `u N sqrt(steps)` of the verification plan, which
/// omits Fo (it failed at n = 20 by a factor 65; recorded in the results file).
fn balance_bound_history(cfg: &Config) -> Vec<f64> {
    let d = [
        cfg.regions[0].diffusivity_m2_s,
        cfg.regions[1].diffusivity_m2_s,
        cfg.regions[2].diffusivity_m2_s,
        cfg.regions[3].diffusivity_m2_s,
        cfg.regions[4].diffusivity_m2_s,
    ];
    let r = run(cfg, &LIMITS, &[]).unwrap();
    let grid = &r.grid;
    let dt = r.meta.dt;
    let m0 = r.meta.initial_inventory;
    let mut stepper = Stepper::new(grid, d, dt).unwrap();
    let (mut c_old, _, _) = initial_vector(grid, cfg.initial.kernel_concentration_mol_m3);
    let mut c_new = vec![0.0; grid.n + 1];
    let tail = (grid.n as f64 + 8.0) * U;
    let mut cum = 0.0;
    let mut out = vec![tail];
    for _ in 0..r.meta.steps {
        stepper.step(&c_old, &mut c_new).unwrap();
        let a = &stepper.assembly().matrix;
        let mut s = 0.0;
        for i in 1..grid.n {
            let left = if i >= 1 {
                a.lower[i].abs() * c_new[i - 1].abs()
            } else {
                0.0
            };
            let right = a.upper[i].abs() * c_new[i + 1].abs();
            s += grid.v[i] * (left + a.diag[i].abs() * c_new[i].abs() + right);
        }
        cum += 4.0 * PI * 16.0 * U * s / m0;
        out.push(cum + tail);
        std::mem::swap(&mut c_old, &mut c_new);
    }
    out
}

fn check_balance(label: &str, cfg: &Config) {
    let r = run(cfg, &LIMITS, &[]).unwrap();
    let bounds = balance_bound_history(cfg);
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
    assert!(eps_wrong > 1e3 * eps_right.max(1e-16));
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
        let kappa = (4.0 * U + 2.0 * gamma5) * b_max;
        assert!(
            kappa < 1e-3,
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
            // |dA| <= gamma_5 |A| (operation count in btcs::tests::one_step_matches_dense_solve)
            // and signs preserved. At the node of largest x, row i gives
            // x_max (rowsum_i(A) - sum_j |dA_ij|) <= (A+dA)x ... <= max c_old, where
            // rowsum_i(A) >= 1 - 4 u B_max (P1 margin, tested below) and
            // sum_j |dA_ij| <= gamma_5 (2 B_max - 1) because B = 1 + |L| + |U|. Hence per step
            // max x <= max c_old / (1 - kappa), kappa = (4 u + 2 gamma_5) B_max, and after k
            // steps max C <= C_init (1 - kappa)^-k.
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
        // an increase can only be rounding: at most the per-step increment of the balance
        // bound plus the rounding of two N-term sums, 2 (N+1) u (first slack, derived from
        // the sum alone, was too small at n = 320; see "Revisions").
        report(
            "G",
            &format!("max_inventory_increase_rel_n{n}"),
            rep.max_inventory_increase_rel,
        );
        let bounds = balance_bound_history(&s5(n, 200.0));
        let m0 = r.meta.initial_inventory;
        let sum_slack = 2.0 * (r.grid.n as f64 + 1.0) * U;
        for k in 1..r.series.len() {
            let inc = (r.series[k].1.inventory - r.series[k - 1].1.inventory) / m0;
            let allowed = (bounds[k] - bounds[k - 1]) + sum_slack;
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
        // Pre-registered: B = (1 - L) - U is rounded, so the margin deviates from 1 by at
        // most about 2 u B_max; the bound used is 4 u B_max.
        let b_max = a.matrix.diag.iter().cloned().fold(0.0, f64::max);
        let tol = 4.0 * U * b_max;
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
