//! Verification of generation, radioactive decay and temperature-dependent diffusion in the
//! BTCS solver (directive section 17, G1 to G4; Document 2 Section 11, Document 5).
//!
//! Each mechanism is checked alone (the other two switched off) before the coupled runs. Every
//! tolerance is derived in the comment beside it, from the unit roundoff `u = 2^-53` with
//! `gamma_k = k u / (1 - k u)`, from the derived rounding bound (F46) of Document 2 Section
//! 11.7, or from the observed orders of Documents 1 and 5 (decision 49: the orders round to 1
//! in time and 2 in space). No tolerance is chosen from a measured residual. Numerical
//! verification is not experimental validation: every case is synthetic.
//!
//! Lines that begin with `RESULT` are printed for the results file
//! (`cargo test --test btcs_physics -- --nocapture`).
//!
//! The "no diffusion" cases use the diffusivity `1e-40 m^2/s` in all five materials. Then every
//! off-diagonal entry of the assembled matrix is below `1e-25`, which is far below half the
//! unit in the last place of the entries it meets; the Thomas solve then reduces to
//! `x_i = fl(b_i / B_i)` at every node whose value is above `1e-11` of its neighbours, as the
//! comments state, and no product underflows.

use std::f64::consts::PI;
use strato::{
    numerical::{
        boundary_conditions::release_rate,
        btcs::{run_input, Stepper},
        coefficients::Coefficients,
        diagnostics::{BalanceBound, StepDiagnostics},
        grid::{Grid, ValidatedInput},
        inventory::{initial_vector, kernel_surface_theta, Accumulator},
        physics_run::{
            combined_balance_bound, run_physics, source_nodes, BoundKind, BoundTerms,
            DiffusivityModel, Interpolation, PhysicsInput, PhysicsRun, SourceModel, TimeFunction,
        },
    },
    physics::{
        arrhenius::{
            ArrheniusLaw, ArrheniusTerm, EnergyUnit, GasConstant, Temperature, TemperatureRange,
        },
        source::{FissionYield, ProductKind, YieldKind},
    },
};

const U: f64 = f64::EPSILON / 2.0;
const RADII: [f64; 5] = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4];
/// Synthetic diffusivities at 1000 K (the synthetic benchmark of the repository).
const D5: [f64; 5] = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13];
/// Synthetic activation temperatures `E_a / R` in K (Document 5, Section 3.2, case 2).
const THETA_K: [f64; 5] = [1.0e4, 2.0e4, 1.5e4, 3.0e4, 1.5e4];
const TINY_D: [f64; 5] = [1.0e-40; 5];

fn gamma(k: f64) -> f64 {
    k * U / (1.0 - k * U)
}

fn vi(cells: [usize; 5], d: [f64; 5], dt_req: f64, end: f64, every: usize) -> ValidatedInput {
    let steps = (end / dt_req).ceil() as usize;
    vi_steps(RADII, cells, d, steps, end, every)
}

fn vi_steps(
    radii: [f64; 5],
    cells: [usize; 5],
    d: [f64; 5],
    steps: usize,
    end: f64,
    every: usize,
) -> ValidatedInput {
    ValidatedInput {
        radii,
        d,
        n_intervals: cells,
        c_init: 1.0,
        dt: end / steps as f64,
        steps,
        output_every: every,
    }
}

fn pin(
    base: ValidatedInput,
    end: f64,
    lambda: f64,
    source: SourceModel,
    diffusivity: DiffusivityModel,
) -> PhysicsInput {
    PhysicsInput {
        base,
        end_time_s: end,
        decay_constant_per_s: lambda,
        source,
        diffusivity,
    }
}

fn kernel_source(g: f64) -> SourceModel {
    let mut s = SourceModel::none();
    s.per_material[0] = TimeFunction::Constant(g);
    s
}

fn uniform_source(g: f64) -> SourceModel {
    SourceModel {
        per_material: std::array::from_fn(|_| TimeFunction::Constant(g)),
    }
}

fn synthetic_law(d1000: f64, theta: f64) -> ArrheniusLaw {
    let ea = theta * GasConstant::SI.value();
    let d0 = d1000 * (theta / 1000.0).exp();
    ArrheniusLaw::single(ArrheniusTerm::new(d0, ea, EnergyUnit::JoulePerMole, None).unwrap())
}

fn synthetic_laws() -> Box<[ArrheniusLaw; 5]> {
    Box::new(std::array::from_fn(|k| synthetic_law(D5[k], THETA_K[k])))
}

fn arrhenius(laws: Box<[ArrheniusLaw; 5]>, temperature_k: TimeFunction) -> DiffusivityModel {
    DiffusivityModel::Arrhenius {
        laws,
        gas: GasConstant::SI,
        temperature_k,
    }
}

fn ramp(t0: f64, t1: f64, end: f64) -> TimeFunction {
    TimeFunction::table(vec![0.0, end], vec![t0, t1], Interpolation::Linear).unwrap()
}

fn same_bits(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits()
}

fn assert_same_row(a: &StepDiagnostics, b: &StepDiagnostics, ctx: &str) {
    assert!(same_bits(a.inventory, b.inventory), "{ctx}: inventory");
    assert!(
        same_bits(a.cumulative_release, b.cumulative_release),
        "{ctx}: release"
    );
    assert!(
        same_bits(a.release_fraction, b.release_fraction),
        "{ctx}: fraction"
    );
    match (a.release_rate, b.release_rate) {
        (None, None) => {}
        (Some(x), Some(y)) => assert!(same_bits(x, y), "{ctx}: rate"),
        _ => panic!("{ctx}: rate defined in only one run"),
    }
    assert!(
        same_bits(a.balance_residual, b.balance_residual),
        "{ctx}: residual"
    );
    assert!(
        same_bits(a.balance_residual_bound, b.balance_residual_bound),
        "{ctx}: bound"
    );
    assert!(
        same_bits(a.min_concentration, b.min_concentration),
        "{ctx}: min"
    );
    assert!(
        same_bits(a.max_concentration, b.max_concentration),
        "{ctx}: max"
    );
}

fn assert_bitwise_baseline(pi: &PhysicsInput, reference_base: &ValidatedInput, ctx: &str) {
    let snaps = [0.0, 0.5 * pi.end_time_s, pi.end_time_s];
    let phys = run_physics(pi, &snaps).unwrap();
    let base = run_input(reference_base, &snaps).unwrap();
    assert_eq!(phys.meta.bound_kind, BoundKind::Baseline, "{ctx}");
    assert_eq!(phys.meta.assemblies, 1, "{ctx}");
    assert_eq!(phys.series.len(), base.series.len(), "{ctx}");
    for (k, ((t1, d1, p1), (t2, d2))) in phys.series.iter().zip(base.series.iter()).enumerate() {
        assert!(same_bits(*t1, *t2), "{ctx}: time {k}");
        assert_same_row(d1, d2, &format!("{ctx} row {k}"));
        assert_eq!(p1.generated, 0.0, "{ctx}");
        assert_eq!(p1.decayed, 0.0, "{ctx}");
    }
    assert_eq!(phys.final_profile.len(), base.final_profile.len());
    for (a, b) in phys.final_profile.iter().zip(base.final_profile.iter()) {
        assert!(same_bits(*a, *b), "{ctx}: final profile");
    }
    assert_eq!(phys.snapshots.len(), base.snapshots.len());
    for ((t1, c1), (t2, c2)) in phys.snapshots.iter().zip(base.snapshots.iter()) {
        assert!(same_bits(*t1, *t2));
        assert!(c1.iter().zip(c2).all(|(a, b)| same_bits(*a, *b)), "{ctx}");
    }
}

// ---------------------------------------------------------------------------------------
// Histories
// ---------------------------------------------------------------------------------------

#[test]
fn history_means_are_exact_for_piecewise_constant_and_linear_forms() {
    // No tolerance: every number below is a dyadic rational, so each operation is exact.
    let step = TimeFunction::table(
        vec![0.0, 10.0, 20.0],
        vec![2.0, 6.0, 4.0],
        Interpolation::Step,
    )
    .unwrap();
    assert_eq!(step.mean_over(0.0, 10.0), 2.0);
    assert_eq!(
        step.mean_over(12.0, 18.0),
        6.0,
        "inside one piece: the value itself"
    );
    // [5, 15]: 5 s at 2 and 5 s at 6.
    assert_eq!(step.mean_over(5.0, 15.0), 4.0);
    // [5, 25]: 5 s at 2, 10 s at 6, 5 s at 4 (the last value holds): (10 + 60 + 20) / 20.
    assert_eq!(step.mean_over(5.0, 25.0), 4.5);
    let lin = TimeFunction::table(
        vec![0.0, 10.0, 20.0],
        vec![0.0, 8.0, 0.0],
        Interpolation::Linear,
    )
    .unwrap();
    // One linear piece: the mean is the value at the middle of the step.
    assert_eq!(lin.mean_over(2.0, 6.0), lin.value_at(4.0));
    assert_eq!(lin.value_at(5.0), 4.0);
    // [5, 15] across the kink: (5 to 10: (4 + 8)/2 * 5 = 30) + (10 to 15: (8 + 4)/2 * 5 = 30).
    assert_eq!(lin.mean_over(5.0, 15.0), 6.0);
    // A linear piece with equal end values is that value, with no rounding.
    let flat = TimeFunction::table(vec![0.0, 10.0], vec![0.1, 0.1], Interpolation::Linear).unwrap();
    assert_eq!(flat.mean_over(1.0, 3.0), 0.1);
    assert_eq!(TimeFunction::Constant(0.3).mean_over(1.0, 7.0), 0.3);
    assert!(lin.covers(20.0) && !lin.covers(20.5) && step.covers(1.0e9));
    assert!(TimeFunction::table(vec![1.0], vec![1.0], Interpolation::Step).is_err());
    assert!(TimeFunction::table(vec![0.0, 0.0], vec![1.0, 1.0], Interpolation::Step).is_err());
    assert!(TimeFunction::table(vec![0.0], vec![f64::NAN], Interpolation::Step).is_err());
}

// ---------------------------------------------------------------------------------------
// Zero source, zero decay, constant temperature: the baseline, bit for bit (P19, P29)
// ---------------------------------------------------------------------------------------

#[test]
fn zero_source_zero_decay_reproduce_the_baseline_bit_for_bit() {
    for (cells, dt, end) in [
        ([6, 5, 4, 5, 4], 200.0, 4000.0),
        ([10, 10, 10, 10, 10], 5.0e3, 2.0e5),
    ] {
        let base = vi(cells, D5, dt, end, 1);
        // (a) the baseline input itself;
        let a = PhysicsInput::baseline(base.clone(), end);
        assert_bitwise_baseline(&a, &base, "baseline input");
        // (b) explicit zero histories of both forms and a zero decay constant;
        let mut zero_src = SourceModel::none();
        zero_src.per_material[0] =
            TimeFunction::table(vec![0.0, end], vec![0.0, 0.0], Interpolation::Linear).unwrap();
        zero_src.per_material[3] =
            TimeFunction::table(vec![0.0, 1.0], vec![0.0, 0.0], Interpolation::Step).unwrap();
        let b = pin(
            base.clone(),
            end,
            0.0,
            zero_src,
            DiffusivityModel::Constant(D5),
        );
        assert_bitwise_baseline(&b, &base, "zero histories");
        // (c) the kernel fission source with a zero fission rate (a direct fission product);
        let y = FissionYield::new(YieldKind::Cumulative, 0.06).unwrap();
        let src = SourceModel::kernel_fission(
            &TimeFunction::Constant(0.0),
            y,
            ProductKind::DirectFissionProduct,
        )
        .unwrap();
        let c = pin(base.clone(), end, 0.0, src, DiffusivityModel::Constant(D5));
        assert_bitwise_baseline(&c, &base, "zero fission rate");
    }
}

#[test]
fn constant_temperature_equals_the_constant_diffusivity_run_bit_for_bit_p29() {
    let end = 4.0e4;
    let base = vi([8, 6, 5, 6, 5], D5, 200.0, end, 1);
    let laws = synthetic_laws();
    for t_k in [900.0, 1000.0, 1234.5] {
        // The diffusivities the constant-T run would be given: the same function, same inputs.
        let d: [f64; 5] = std::array::from_fn(|k| {
            laws[k]
                .evaluate(Temperature::Kelvin(t_k), GasConstant::SI)
                .unwrap()
                .d_m2_s
        });
        let mut reference_base = base.clone();
        reference_base.d = d;
        for history in [
            TimeFunction::Constant(t_k),
            TimeFunction::table(vec![0.0, end], vec![t_k, t_k], Interpolation::Linear).unwrap(),
            TimeFunction::table(vec![0.0, 100.0], vec![t_k, t_k], Interpolation::Step).unwrap(),
        ] {
            let pi = pin(
                base.clone(),
                end,
                0.0,
                SourceModel::none(),
                arrhenius(laws.clone(), history),
            );
            assert_bitwise_baseline(&pi, &reference_base, "constant temperature");
        }
    }
}

// ---------------------------------------------------------------------------------------
// G1: the source weights and the generated amount
// ---------------------------------------------------------------------------------------

#[test]
fn source_weights_f33_hand_values() {
    let g = Grid::from_radii(RADII, [3, 2, 4, 5, 2]).unwrap();
    let nk = g.interface_nodes[0];
    // (a) A node inside one material with the same source on both sides: the stored radii and
    // spacings are the same numbers in Q_i and in V_i, so g_i = Q_i / V_i differs from G by
    // the roundings only. Q_i = fl(fl(fl(r r)/2) fl(fl(a G) + fl(b G))) has 5 roundings (the
    // product r r, the two products with G, their sum, the final product; the division by 2 is
    // exact), V_i = fl(fl(fl(r r) fl(a + b))/2) has 3, the quotient 1: |g_i / G - 1| <= gamma_9.
    let gv = 0.7;
    let nodes = source_nodes(&g, [gv; 5]);
    for (i, gi) in nodes.g.iter().enumerate().skip(1) {
        assert!((gi / gv - 1.0).abs() <= gamma(9.0), "node {i}: {gi}");
    }
    assert_eq!(nodes.g[0], gv);
    assert_eq!(nodes.q[0], 0.0);
    // (b) Interface nodes other than N_kernel: the half-interval weighted mean of the two
    // materials (archive B, Section 9). The reference mean has 4 roundings, g_i at most 9
    // relative to the same ideal: gamma_13.
    let gbar = [0.5, 1.5, 2.5, 3.5, 4.5];
    let nodes = source_nodes(&g, gbar);
    for (k, &i) in g.interface_nodes.iter().enumerate().skip(1) {
        let (a, b) = (g.dr_inner(i), g.dr_outer(i));
        let mean = (a * gbar[k] + b * gbar[k + 1]) / (a + b);
        assert!(
            (nodes.g[i] - mean).abs() <= gamma(13.0) * mean,
            "interface {i}: {} {mean}",
            nodes.g[i]
        );
        assert!(nodes.g[i] > gbar[k] && nodes.g[i] < gbar[k + 1]);
    }
    // (c) Node N_kernel with a kernel-only source: g = omega drk/(drk + drb) G, with
    // omega = 1 - 1/(3 n_kernel) (F33). Roundings: Q has 6 (omega 2, omega G, drk (...), the two
    // products with half r^2), V has 3, the quotient 1; the test reference has omega 2,
    // quotient 2, products 2: gamma_16.
    let nodes = source_nodes(&g, [gv, 0.0, 0.0, 0.0, 0.0]);
    let omega = 1.0 - 1.0 / (3.0 * nk as f64);
    let (a, b) = (g.dr_inner(nk), g.dr_outer(nk));
    let expect = omega * a / (a + b) * gv;
    assert!(
        (nodes.g[nk] - expect).abs() <= gamma(16.0) * expect,
        "N_kernel: {} {expect}",
        nodes.g[nk]
    );
    // It differs from the plain half-interval weight a / (a + b) by the factor omega.
    assert!(nodes.g[nk] < a / (a + b) * gv * (1.0 - 0.5 / (3.0 * nk as f64)));
    // (d) Outer node: Q_N = V_N G_OPyC with V_N = (R_5)^2 dr / 2, and no row (g has length N).
    let nodes = source_nodes(&g, [0.0, 0.0, 0.0, 0.0, 2.0]);
    let v_n = RADII[4] * RADII[4] * g.dr_inner(g.n) / 2.0;
    assert!((nodes.q[g.n] - 2.0 * v_n).abs() <= gamma(4.0) * 2.0 * v_n);
    assert_eq!(nodes.g.len(), g.n);
    assert!(nodes.active);
    // (e) No source: no weight and not active.
    let nodes = source_nodes(&g, [0.0; 5]);
    assert!(!nodes.active && nodes.sum_q == 0.0 && nodes.q.iter().all(|q| *q == 0.0));
}

/// Relative bound for `sum_{i<n} W_i + T` against `R_1^3/3`, as `f23_relative_bound` in the unit
/// tests of `numerical::inventory` (derivation (a) to (d) there), where `extra` is the number of
/// additional roundings per term. A source weight `Q_i` of an ordinary kernel node has 5
/// roundings against the 3 of `V_i` (extra = 2); the term of node `N_kernel` has at most
/// `3 n + 10` against the `3 n + 14` of that derivation.
fn f23_relative_bound(nk: usize, extra: f64) -> f64 {
    let n = nk as f64;
    let g_t = gamma(3.0 * n + 14.0 + extra);
    gamma(n - 1.0) * (1.0 + g_t)
        + gamma(2.0 * n + 6.0 + extra)
        + g_t * (3.0 * n - 1.0) / (2.0 * n * n)
}

#[test]
fn generated_amount_is_the_exact_kernel_integral_p20() {
    for cells in [[3, 2, 4, 5, 2], [10, 10, 10, 10, 10], [7, 11, 3, 5, 13]] {
        let g = Grid::from_radii(RADII, cells).unwrap();
        let nk = g.interface_nodes[0];
        let gk = 0.37;
        let nodes = source_nodes(&g, [gk, 0.0, 0.0, 0.0, 0.0]);
        // sum_i Q_i against G R_1^3 / 3 (P20): the f23 bound, the rounded sum of the N
        // non-negative terms (N - 1 additions, gamma_{N-1}) and the 3 roundings of the reference.
        let exact = RADII[0] * RADII[0] * RADII[0] / 3.0 * gk;
        let tol = (f23_relative_bound(nk, 2.0) + gamma(g.n as f64 - 1.0) + gamma(3.0))
            / (1.0 - gamma(3.0));
        let rel = (nodes.sum_q - exact).abs() / exact;
        assert!(
            rel <= tol,
            "{cells:?}: sum {} exact {exact} rel {rel} tol {tol}",
            nodes.sum_q
        );
        // The plain half-interval weight at N_kernel would give the larger sum
        // (1 + 1/(2 n^2)): the omega factor is what makes the sum exact (Document 5, 1.3).
        let plain = 1.0 + 1.0 / (2.0 * (nk * nk) as f64);
        assert!(
            plain - 1.0 > 100.0 * tol,
            "{cells:?}: the omega factor must be resolvable"
        );
        println!("RESULT P20 sum_Q relative error {cells:?}: {rel:.3e} (bound {tol:.3e})");
        // Nodes outside the kernel and the coating weights are zero for a kernel-only source
        // except the buffer-side half interval of N_kernel, which is multiplied by zero.
        assert!(nodes.q[nk + 1..].iter().all(|q| *q == 0.0));
    }
}

// ---------------------------------------------------------------------------------------
// G1 alone, no diffusion: exact accumulation (P20, P21)
// ---------------------------------------------------------------------------------------

#[test]
fn source_only_without_diffusion_accumulates_c_init_plus_g_t_p20_p21() {
    for cells in [[6, 5, 4, 5, 4], [10, 10, 10, 10, 10]] {
        let (end, steps) = (4000.0, 40usize);
        let gk = 0.37;
        let base = vi_steps(RADII, cells, TINY_D, steps, end, 1);
        let pi = pin(
            base,
            end,
            0.0,
            kernel_source(gk),
            DiffusivityModel::Constant(TINY_D),
        );
        let run = run_physics(&pi, &[1000.0, 2000.0, 4000.0]).unwrap();
        let nk = run.grid.interface_nodes[0];
        let theta = kernel_surface_theta(run.grid.dr_region[0], run.grid.dr_region[1], nk);
        assert_eq!(run.meta.bound_kind, BoundKind::Combined);
        for (t, c) in &run.snapshots {
            let j = (t / run.meta.dt).round();
            // Kernel nodes: with every |L|, |U| below 1e-25 the Thomas steps leave b_i
            // unchanged (b'_i = b_i - w b'_{i-1} and x_i = (b'_i - U x_{i+1})/1 add terms below
            // half an ulp), so C_i^{j+1} = fl(C_i^j + fl(dt g_i)). Per node: g_i = G (1 + theta_9)
            // (see source_weights_f33_hand_values), fl(dt g_i) one more rounding, J additions
            // of positive terms: C_i^J = (c0 + J dt G (1 + theta_10)) (1 + theta_J). The
            // reference c0 + G t has 3 roundings (t = J dt, G t, the sum).
            let reference = 1.0 + gk * t;
            for (i, ci) in c.iter().enumerate().take(nk) {
                assert!(
                    (ci - reference).abs() <= gamma(j + 13.0) * reference,
                    "{cells:?} t {t} node {i}: {ci} vs {reference}"
                );
            }
            // Node N_kernel keeps the shape of the start vector (P21): theta (c0 + G t). Its
            // source density is theta G up to the roundings of the stored radii, which are
            // bounded as in the f23 derivation, gamma_{3 n + 24} (three more roundings than the
            // gamma_{3 n + 14} there for theta; the quotient and product here).
            let ref_k = theta * reference;
            assert!(
                (c[nk] - ref_k).abs() <= gamma(j + 13.0 + 3.0 * nk as f64 + 24.0) * ref_k,
                "{cells:?} t {t} N_kernel: {} vs {ref_k}",
                c[nk]
            );
            // Coating nodes only receive the 1e-27-sized coupling: never negative.
            assert!(c.iter().all(|x| *x >= 0.0));
            assert_eq!(*c.last().unwrap(), 0.0);
        }
        // P20: the generated amount is the exact kernel integral. Relative bound: the sum of
        // the weights (as in generated_amount_is_the_exact_kernel_integral_p20), fl(p S) and
        // fl(dt (.)) one rounding each per step (both are factors of every increment, so they
        // act once on the total: gamma_2), the compensated accumulator at most 3u (Goldberg,
        // Theorem 8: 2u, with the O(J u^2) remainder of that theorem allowed one more u while
        // J u <= 1), and the reference (4 pi/3) R^3 G t with 6 roundings (pi is the same stored
        // constant in both).
        let last = run.series.last().unwrap();
        let t = last.0;
        let exact = 4.0 * PI / 3.0 * RADII[0] * RADII[0] * RADII[0] * gk * t;
        let tol = (f23_relative_bound(nk, 2.0) + gamma(run.grid.n as f64 - 1.0) + gamma(3.0))
            / (1.0 - gamma(3.0))
            + gamma(2.0)
            + 3.0 * U
            + gamma(6.0);
        let rel = (last.2.generated - exact).abs() / exact;
        assert!(
            rel <= tol,
            "M_gen {} exact {exact} rel {rel} tol {tol}",
            last.2.generated
        );
        println!("RESULT G1 M_gen relative error {cells:?}: {rel:.3e} (bound {tol:.3e})");
        // Nothing decays and, with no diffusion, essentially nothing is released.
        assert_eq!(last.2.decayed, 0.0);
        assert!(last.1.cumulative_release < 1.0e-20 * exact);
    }
}

// ---------------------------------------------------------------------------------------
// G2 alone, no diffusion: (1 + lambda dt)^-j and e^(-lambda t)
// ---------------------------------------------------------------------------------------

fn decay_only_final(lambda: f64, steps: usize, end: f64) -> (PhysicsRun, f64) {
    let cells = [6, 5, 4, 5, 4];
    let base = vi_steps(RADII, cells, TINY_D, steps, end, 1);
    let pi = pin(
        base,
        end,
        lambda,
        SourceModel::none(),
        DiffusivityModel::Constant(TINY_D),
    );
    let run = run_physics(&pi, &[]).unwrap();
    let kappa = lambda * run.meta.dt;
    (run, kappa)
}

#[test]
fn decay_only_without_diffusion_discrete_and_continuous_closed_forms() {
    let lambda = 1.0 / 400.0;
    let end = 400.0;
    let mut errors = Vec::new();
    for steps in [50usize, 100, 200, 400] {
        let (run, kappa) = decay_only_final(lambda, steps, end);
        assert_eq!(run.meta.decay_number, kappa);
        let j = steps as f64;
        let t = run.series.last().unwrap().0;
        // Kernel nodes 0 .. N_kernel - 1: x^{j+1} = fl(x^j / B) with B = fl(1 + fl(lambda dt))
        // (no diffusion: every off-diagonal term is absorbed; see the module comment). That
        // is J divisions (J roundings) against c0 B^-J, and B against 1 + lambda dt (the
        // product lambda dt and the sum, 2 roundings per factor): the ratio of the computed
        // value to c0 (1 + lambda dt)^-J lies within a factor (1 +- gamma_{3J}).
        let c = &run.final_profile;
        let nk = run.grid.interface_nodes[0];
        let ideal = (1.0 + kappa).powi(-(steps as i32));
        // Discrete closed form, tolerance gamma_{3 J + 6}: the 3J above and 6 for the library
        // power (repeated squaring: at most J - 1 further roundings are covered by the J + 3 of
        // the sum; the generous constant 6 is the rounding of the reference only).
        let tol_discrete = gamma(3.0 * j + 6.0);
        // Continuous closed form. For kappa = lambda dt: (1 + kappa)^-J / e^{-J kappa} - 1 lies
        // in [e^{J (kappa^2/2 - kappa^3/3)} - 1, e^{J kappa^2/2} - 1], because
        // kappa - kappa^2/2 <= ln(1 + kappa) <= kappa - kappa^2/2 + kappa^3/3 holds for
        // 0 <= kappa <= 1 (alternating Taylor series). The reference exp(-lambda t) has its
        // argument rounded (2 roundings: gamma_2 relative in the exponent, so 2 u lambda t) and
        // one library rounding (taken as 2u, a faithful exp).
        let reference = (-lambda * t).exp();
        let extra = 4.0 * lambda * t * U + 2.0 * U;
        let upper = (j * kappa * kappa / 2.0).exp_m1() + tol_discrete + extra;
        let lower = (j * (kappa * kappa / 2.0 - kappa * kappa * kappa / 3.0)).exp_m1()
            - tol_discrete
            - extra;
        for (i, ci) in c.iter().enumerate().take(nk) {
            assert!(
                (ci / ideal - 1.0).abs() <= tol_discrete,
                "J {steps} node {i}: discrete closed form"
            );
            // x / reference - 1 = (1 + kappa)^-J / e^{-J kappa} - 1 lies in [lower, upper] up
            // to tol_discrete, which both ends already contain.
            let ratio = ci / reference - 1.0;
            assert!(
                ratio <= upper && ratio >= lower,
                "J {steps} node {i}: ratio-1 {ratio} not in [{lower}, {upper}]"
            );
        }
        let ratio0 = c[0] / reference - 1.0;
        errors.push(ratio0.abs());
        println!(
            "RESULT G2 decay-only no diffusion J={steps}: relative error against exp(-lambda t) {ratio0:.4e} (bracket [{lower:.4e}, {upper:.4e}])"
        );
    }
    // First order in time: the error halves with the step (the rounding is 1e-13 or smaller).
    for w in errors.windows(2) {
        let order = (w[0] / w[1]).log2();
        assert!((0.5..1.5).contains(&order), "order {order} rounds to 1");
    }
}

#[test]
fn decay_with_source_without_diffusion_closed_form() {
    let (lambda, gk, c0) = (1.0 / 400.0, 3.0e-3, 1.0);
    let end = 800.0;
    let mut errors = Vec::new();
    for steps in [50usize, 100, 200, 400] {
        let cells = [6, 5, 4, 5, 4];
        let base = vi_steps(RADII, cells, TINY_D, steps, end, 1);
        let pi = pin(
            base,
            end,
            lambda,
            kernel_source(gk),
            DiffusivityModel::Constant(TINY_D),
        );
        let run = run_physics(&pi, &[]).unwrap();
        let j = steps as f64;
        let t = run.series.last().unwrap().0;
        let kappa = lambda * run.meta.dt;
        // Per step x^{j+1} = fl(fl(x^j + fl(dt g)) / B): every number is positive, so the
        // relative perturbations multiply: the sum (1), the division (1), B against 1 + kappa (2)
        // act on everything accumulated so far (4 per step), and the source term carries
        // g = G (1 + theta_9) and the product dt g (1) once more: the computed value is within
        // (1 +- gamma_{4 J + 10}) of the exact solution of the rounded-free recurrence
        // x_{j+1} = (x_j + dt G)/(1 + kappa), which is x* + (c0 - x*)(1 + kappa)^-j with
        // x* = G/lambda.
        let x_star = gk / lambda;
        let c = run.final_profile[1];
        let discrete = x_star + (c0 - x_star) * (1.0 + kappa).powi(-(steps as i32));
        // The reference of the library: C_init e^{-lambda t} + (G/lambda)(1 - e^{-lambda t}).
        let reference = strato::physics::reference::source_with_decay(c0, gk, lambda, t).unwrap();
        // Distance of the discrete solution from the continuous one:
        // |(c0 - x*) ((1 + kappa)^-J - e^{-J kappa})| <= |c0 - x*| e^{-J kappa} (e^{J kappa^2/2} - 1)
        // (same inequality as in the decay-only test). The two references add 6 roundings of
        // their own, 4 lambda t u from the arguments and 2u for exp.
        let tol_rounding = gamma(4.0 * j + 10.0) + gamma(10.0) + 4.0 * lambda * t * U + 2.0 * U;
        let bound = (c0 - x_star).abs() * (-j * kappa).exp() * (j * kappa * kappa / 2.0).exp_m1()
            + tol_rounding * reference;
        assert!(
            (c - reference).abs() <= bound,
            "J {steps}: {c} vs {reference}, bound {bound}"
        );
        assert!(
            (c - discrete).abs() <= tol_rounding * discrete,
            "J {steps}: discrete {c} vs {discrete}"
        );
        errors.push((c - reference).abs());
        println!(
            "RESULT G1+G2 source with decay no diffusion J={steps}: |C - reference| = {:.4e} (bound {bound:.4e})",
            (c - reference).abs()
        );
    }
    for w in errors.windows(2) {
        let order = (w[0] / w[1]).log2();
        assert!((0.5..1.5).contains(&order), "order {order} rounds to 1");
    }
}

// ---------------------------------------------------------------------------------------
// The closed forms of Document 5 with diffusion: kernel-step series, decay and source
// ---------------------------------------------------------------------------------------

/// Weight of mode `mu` of the kernel-step initial profile (Document 5, Eq. (21)):
/// `w = 6 (-1)^(mu+1) (sin x - x cos x) / x^3`, `x = mu pi R_1 / R_5`; `sum w = 1`.
fn mode_weight(mu: usize, r1: f64, r5: f64) -> f64 {
    let x = mu as f64 * PI * r1 / r5;
    let sign = if mu % 2 == 1 { 1.0 } else { -1.0 };
    6.0 * sign * (x.sin() - x * x.cos()) / (x * x * x)
}

/// Retained, released and decayed fractions of the initial inventory `M^0` for a homogeneous
/// sphere (one `d`), decay constant `lambda`, kernel source of `q = G / C_init` per second and
/// a kernel-step start. Per mode, with `a = d (mu pi/R_5)^2`, `s = a + lambda`:
/// retained `w [e^{-s t} + q (1 - e^{-s t})/s]`, decayed `w lambda int_0^t m`, with
/// `int_0^t m = (1 - e^{-s t})/s + q (t/s - (1 - e^{-s t})/s^2)`. The released fraction is
/// obtained from the balance of the exact solution, `1 + q t - retained - decayed` (the series
/// `sum w a int m` does not decay in `mu`, but `sum w = 1` is exact). 400 modes: the neglected
/// terms of the retained and decayed series are below `1e-300` at the times used.
fn kernel_step_fractions(d: f64, lambda: f64, q: f64, t: f64, r1: f64, r5: f64) -> (f64, f64, f64) {
    let (mut ret, mut dec) = (0.0, 0.0);
    for mu in 1..=400usize {
        let k = mu as f64 * PI / r5;
        let a = d * k * k;
        let s = a + lambda;
        let w = mode_weight(mu, r1, r5);
        let e = (-s * t).exp();
        let g1 = -(-s * t).exp_m1() / s;
        let m = e + q * g1;
        let int_m = g1 + q * (t / s - g1 / s);
        ret += w * m;
        dec += w * lambda * int_m;
    }
    (ret, 1.0 + q * t - ret - dec, dec)
}

fn homogeneous_run(n: usize, steps: usize, d: f64, lambda: f64, gk: f64, t_end: f64) -> PhysicsRun {
    let base = vi_steps(RADII, [n; 5], [d; 5], steps, t_end, steps);
    let source = if gk > 0.0 {
        kernel_source(gk)
    } else {
        SourceModel::none()
    };
    let pi = pin(
        base,
        t_end,
        lambda,
        source,
        DiffusivityModel::Constant([d; 5]),
    );
    run_physics(&pi, &[]).unwrap()
}

fn fractions_of(run: &PhysicsRun) -> (f64, f64, f64) {
    let (_, d, p) = run.series.last().unwrap();
    let m0 = run.meta.initial_inventory;
    (d.inventory / m0, d.cumulative_release / m0, p.decayed / m0)
}

/// Orders of convergence of the three fractions against a reference. Time: Cauchy orders of
/// five step counts at 40 intervals per material (the spatial error cancels). Space: the time
/// error is removed by Richardson extrapolation `2 F(2J) - F(J)` of two fine step counts, then
/// the error against the reference is formed at 5, 10, 20 and 40 intervals per material; the
/// orders are `log2` of consecutive error ratios. The result is asserted to round to 1 in time
/// and 2 in space (decision 49: Documents 1 and 5 observe these orders and prove the former
/// for the baseline). As a check that the resolution is sufficient, the difference between the
/// extrapolations of two further step counts (an estimate of the time error that remains) must
/// be below a tenth of the error at 40 intervals; no bound is changed in response to it.
fn convergence_study(
    label: &str,
    run_at: &dyn Fn(usize, usize) -> PhysicsRun,
    reference: (f64, f64, f64),
    j_space: usize,
) {
    let pick = |f: (f64, f64, f64), which: usize| [f.0, f.1, f.2][which];
    let names = ["retained", "released", "decayed"];
    // Time orders.
    let times: Vec<(f64, f64, f64)> = [100usize, 200, 400, 800, 1600]
        .iter()
        .map(|&j| fractions_of(&run_at(40, j)))
        .collect();
    // Space study.
    let spaces: Vec<usize> = vec![5, 10, 20, 40];
    let extrap = |n: usize, j: usize| {
        let a = fractions_of(&run_at(n, j));
        let b = fractions_of(&run_at(n, 2 * j));
        (2.0 * b.0 - a.0, 2.0 * b.1 - a.1, 2.0 * b.2 - a.2)
    };
    let fine: Vec<(f64, f64, f64)> = spaces.iter().map(|&n| extrap(n, j_space)).collect();
    let finer_40 = extrap(40, 2 * j_space);
    for (which, name) in names.iter().enumerate() {
        if which == 2 && pick(reference, 2).abs() < 1.0e-14 {
            continue;
        }
        let diffs: Vec<f64> = times
            .windows(2)
            .map(|w| pick(w[0], which) - pick(w[1], which))
            .collect();
        let mut t_orders = Vec::new();
        for w in diffs.windows(2) {
            t_orders.push((w[0] / w[1]).log2());
        }
        let errs: Vec<f64> = fine
            .iter()
            .map(|f| (pick(*f, which) - pick(reference, which)).abs())
            .collect();
        let s_orders: Vec<f64> = errs.windows(2).map(|w| (w[0] / w[1]).log2()).collect();
        let remaining_time_error = (pick(finer_40, which) - pick(fine[3], which)).abs();
        println!(
            "RESULT {label} {name}: reference {:.9}, error at 40/material {:.3e}, time orders {:?}, space orders {:?}, remaining time error estimate {:.2e}",
            pick(reference, which),
            errs[3],
            t_orders.iter().map(|o| format!("{o:.3}")).collect::<Vec<_>>(),
            s_orders.iter().map(|o| format!("{o:.3}")).collect::<Vec<_>>(),
            remaining_time_error
        );
        for o in &t_orders {
            assert!(
                (0.5..1.5).contains(o),
                "{label} {name}: time order {o} does not round to 1"
            );
        }
        for o in &s_orders {
            assert!(
                (1.5..2.5).contains(o),
                "{label} {name}: space order {o} does not round to 2"
            );
        }
        assert!(
            remaining_time_error <= 0.1 * errs[3],
            "{label} {name}: time error {remaining_time_error} not negligible against {}",
            errs[3]
        );
    }
}

#[test]
fn decay_with_diffusion_matches_the_series_with_orders_1_and_2() {
    // Five equal layers, D = 1e-10 m^2/s, lambda = 1/400 1/s, t = 405 s (Document 5, 2.3).
    let (d, lambda, t) = (1.0e-10, 1.0 / 400.0, 405.0);
    let reference = kernel_step_fractions(d, lambda, 0.0, t, RADII[0], RADII[4]);
    println!(
        "RESULT G2 decay with diffusion reference: retained {:.9} released {:.9} decayed {:.9}",
        reference.0, reference.1, reference.2
    );
    // The three exact fractions add up to 1 (continuous balance, sum w = 1).
    assert!((reference.0 + reference.1 + reference.2 - 1.0).abs() < 1.0e-15);
    convergence_study(
        "G2 decay+diffusion",
        &|n, j| homogeneous_run(n, j, d, lambda, 0.0, t),
        reference,
        400,
    );
}

#[test]
fn source_with_diffusion_and_with_decay_matches_the_duhamel_series() {
    let (d, t) = (1.0e-10, 405.0);
    // Source only (lambda = 0): G / C_init = 2e-3 per second.
    let reference = kernel_step_fractions(d, 0.0, 2.0e-3, t, RADII[0], RADII[4]);
    println!(
        "RESULT G1 source with diffusion reference: retained {:.9} released {:.9}",
        reference.0, reference.1
    );
    // The remaining time error of the extrapolation is larger with a source (the first check
    // at 400 and 800 steps showed 4e-7 against a spatial error of 1.8e-6); the step counts of
    // the space study are therefore 1600 and 3200, which the resolution check demands.
    convergence_study(
        "G1 source+diffusion",
        &|n, j| homogeneous_run(n, j, d, 0.0, 2.0e-3, t),
        reference,
        1600,
    );
    // Source with decay and diffusion.
    let lambda = 1.0 / 400.0;
    let reference = kernel_step_fractions(d, lambda, 2.0e-3, t, RADII[0], RADII[4]);
    println!(
        "RESULT G1+G2 source and decay with diffusion reference: retained {:.9} released {:.9} decayed {:.9}",
        reference.0, reference.1, reference.2
    );
    convergence_study(
        "G1+G2 source+decay+diffusion",
        &|n, j| homogeneous_run(n, j, d, lambda, 2.0e-3, t),
        reference,
        1600,
    );
}

// ---------------------------------------------------------------------------------------
// G1 with G2 and an OPyC source: the steady state of one material (P22a, P23)
// ---------------------------------------------------------------------------------------

#[test]
fn steady_state_with_source_and_decay_in_one_material_has_second_order() {
    // One material (equal D, equal source everywhere including the OPyC), one spacing (equal
    // thickness and interval count): the discrete steady state is the limit of a Backward Euler
    // step of enormous length. Exact steady state: C(r) = (G/lambda)(1 - R sinh(b r)/(r sinh(b R))),
    // b = sqrt(lambda/D), R = R_5; D (C'' + 2 C'/r) - lambda C + G = 0 holds, C is regular at 0
    // and vanishes at R. The release rate is Phi = 4 pi R D (G/lambda)(bR coth(bR) - 1).
    let radii = [1.0e-4, 2.0e-4, 3.0e-4, 4.0e-4, 5.0e-4];
    let (d, g, br) = (1.0e-10, 1.0e-3, 4.5f64);
    let r_out = radii[4];
    let b = br / r_out;
    let lambda = b * b * d;
    let a = g / lambda;
    let exact = |r: f64| -> f64 {
        if r == 0.0 {
            a * (1.0 - br / br.sinh())
        } else {
            a * (1.0 - r_out * (b * r).sinh() / (r * br.sinh()))
        }
    };
    let phi_exact = 4.0 * PI * r_out * d * a * (br / br.tanh() - 1.0);
    let mut node_errors = Vec::new();
    let mut release_errors = Vec::new();
    for n in [5usize, 10, 20, 40] {
        // One step of 1e13 s: lambda dt = 8e10, so the initial profile contributes 1e-11 of
        // its size, which is far below the spatial errors measured (1e-3 to 1e-6).
        let end = 1.0e13;
        let base = vi_steps(radii, [n; 5], [d; 5], 1, end, 1);
        let pi = pin(
            base,
            end,
            lambda,
            uniform_source(g),
            DiffusivityModel::Constant([d; 5]),
        );
        let run = run_physics(&pi, &[]).unwrap();
        let grid = &run.grid;
        let c = &run.final_profile;
        let err = (0..grid.n)
            .map(|i| (c[i] - exact(grid.r[i])).abs())
            .fold(0.0, f64::max)
            / exact(0.0);
        let phi = run.series.last().unwrap().1.release_rate.unwrap();
        node_errors.push(err);
        release_errors.push((phi / phi_exact - 1.0).abs());
        // Non-negative and below max(C_init, G/lambda) = 1 (P31, Eq. (28)): the step starts
        // from C_init = 1 and the bound shrinks by 1/(1 + kappa) per step, so one step of
        // kappa = 8e10 leaves at most 1.3e-11 + G/lambda, far below 1.
        assert!(c.iter().all(|x| *x >= 0.0 && *x <= 1.0));
    }
    let fmt = |v: &[f64]| {
        v.iter()
            .map(|x| format!("{x:.3e}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    println!(
        "RESULT G1+G2 steady state nodal errors at 5, 10, 20, 40 intervals per material: {}",
        fmt(&node_errors)
    );
    println!(
        "RESULT G1+G2 steady state release errors at 5, 10, 20, 40 intervals per material: {}",
        fmt(&release_errors)
    );
    for errs in [&node_errors, &release_errors] {
        for w in errs.windows(2) {
            let order = (w[0] / w[1]).log2();
            assert!(
                (1.5..2.5).contains(&order),
                "order {order} does not round to 2"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------
// G3: temperature-dependent diffusion
// ---------------------------------------------------------------------------------------

#[test]
fn step_temperature_history_equals_a_chain_of_constant_runs_bit_for_bit() {
    // Reassembly: the temperature steps at multiples of dt, so each plateau is a constant-D run.
    let cells = [6, 5, 4, 5, 4];
    let (steps, end) = (60usize, 12000.0);
    let base = vi_steps(RADII, cells, D5, steps, end, 1);
    let dt = base.dt;
    let laws = synthetic_laws();
    let plateaus = [(0usize, 900.0), (20, 1000.0), (45, 1100.0)];
    let times: Vec<f64> = plateaus.iter().map(|(j, _)| *j as f64 * dt).collect();
    let values: Vec<f64> = plateaus.iter().map(|(_, t)| *t).collect();
    let hist = TimeFunction::table(times, values, Interpolation::Step).unwrap();
    let pi = pin(
        base.clone(),
        end,
        0.0,
        SourceModel::none(),
        arrhenius(laws.clone(), hist),
    );
    let run = run_physics(&pi, &[]).unwrap();
    assert_eq!(run.meta.assemblies, 3, "reassembled once per plateau");
    // The hand chain with the baseline stepper.
    let grid = Grid::build(&base).unwrap();
    let (mut c_old, _, _) = initial_vector(&grid, 1.0);
    let mut c_new = vec![0.0; grid.n + 1];
    let mut acc = Accumulator::new();
    let mut stepper: Option<Stepper> = None;
    for j in 0..steps {
        if let Some((_, t_k)) = plateaus.iter().rev().find(|(js, _)| j >= *js) {
            let d: [f64; 5] = std::array::from_fn(|k| {
                laws[k]
                    .evaluate(Temperature::Kelvin(*t_k), GasConstant::SI)
                    .unwrap()
                    .d_m2_s
            });
            if plateaus.iter().any(|(js, _)| *js == j) {
                stepper = Some(Stepper::new(&grid, d, dt).unwrap());
            }
        }
        let s = stepper.as_mut().unwrap();
        s.step(&c_old, &mut c_new).unwrap();
        acc.add(dt * release_rate(s.k_last(), c_new[grid.n - 1]));
        std::mem::swap(&mut c_old, &mut c_new);
    }
    assert!(run
        .final_profile
        .iter()
        .zip(c_old.iter())
        .all(|(a, b)| same_bits(*a, *b)));
    let last = run.series.last().unwrap();
    assert!(same_bits(last.1.cumulative_release, acc.value()));
}

#[test]
fn temperature_range_is_reported_and_never_clamped() {
    let end = 2.0e4;
    let base = vi([4, 4, 4, 4, 4], D5, 500.0, end, 1);
    let r =
        TemperatureRange::new(Temperature::Kelvin(1000.0), Temperature::Kelvin(1800.0)).unwrap();
    let law = |k: usize| {
        let ea = THETA_K[k] * GasConstant::SI.value();
        let d0 = D5[k] * (THETA_K[k] / 1000.0).exp();
        ArrheniusLaw::single(ArrheniusTerm::new(d0, ea, EnergyUnit::JoulePerMole, Some(r)).unwrap())
    };
    let laws: Box<[ArrheniusLaw; 5]> = Box::new(std::array::from_fn(law));
    let pi = pin(
        base,
        end,
        0.0,
        SourceModel::none(),
        arrhenius(laws, ramp(900.0, 1100.0, end)),
    );
    let run = run_physics(&pi, &[]).unwrap();
    let steps = run.meta.steps;
    for rep in &run.meta.ranges {
        assert_eq!(rep.within + rep.below, steps);
        assert!(rep.below > 0 && rep.within > 0 && rep.above == 0);
        // The ramp crosses 1000 K at half of the steps; the step middle (j + 1/2) dt decides.
        assert_eq!(rep.below, steps / 2);
    }
    // The temperature column starts at 900 K and ends at 1100 K.
    assert_eq!(run.series[0].2.temperature_k, Some(900.0));
    assert_eq!(run.series.last().unwrap().2.temperature_k, Some(1100.0));
}

/// `tau(t) = int_0^t D(T(s)) ds` by composite 5-point Gauss-Legendre quadrature on 200 panels
/// (error negligible for the smooth Arrhenius ramp; the nodes and weights are the standard
/// ones).
fn tau_of(law: &ArrheniusLaw, hist: &TimeFunction, t_end: f64) -> f64 {
    let x = [
        0.0,
        0.538_469_310_105_683_1,
        -0.538_469_310_105_683_1,
        0.906_179_845_938_664,
        -0.906_179_845_938_664,
    ];
    let w = [
        0.568_888_888_888_888_9,
        0.478_628_670_499_366_5,
        0.478_628_670_499_366_5,
        0.236_926_885_056_189_1,
        0.236_926_885_056_189_1,
    ];
    let panels = 200;
    let h = t_end / panels as f64;
    let mut sum = 0.0;
    for p in 0..panels {
        let mid = (p as f64 + 0.5) * h;
        for (xi, wi) in x.iter().zip(w) {
            let t = mid + 0.5 * h * xi;
            let d = law
                .evaluate(Temperature::Kelvin(hist.value_at(t)), GasConstant::SI)
                .unwrap()
                .d_m2_s;
            sum += 0.5 * h * wi * d;
        }
    }
    sum
}

fn temperature_run(
    n: usize,
    steps: usize,
    law: &ArrheniusLaw,
    hist: &TimeFunction,
    end: f64,
) -> PhysicsRun {
    let base = vi_steps(RADII, [n; 5], [1.0e-10; 5], steps, end, steps);
    let laws: Box<[ArrheniusLaw; 5]> = Box::new(std::array::from_fn(|_| law.clone()));
    let pi = pin(
        base,
        end,
        0.0,
        SourceModel::none(),
        arrhenius(laws, hist.clone()),
    );
    run_physics(&pi, &[]).unwrap()
}

#[test]
fn homogeneous_sphere_with_a_temperature_ramp_matches_the_change_of_variable_p28() {
    // All five laws equal: the exact solution is the unit-diffusivity series at
    // tau(t) = int D dt (Document 5, Eq. (25)). Reference: kernel-step series with D = 1 at
    // time tau. Orders as in `convergence_study` (no decay, no source: the released fraction
    // is 1 - retained).
    let end = 400.0;
    let law = synthetic_law(5.0e-11, 1.5e4);
    let hist = ramp(900.0, 1000.0, end);
    let tau = tau_of(&law, &hist, end);
    let (ret, rel, dec) = kernel_step_fractions(1.0, 0.0, 0.0, tau, RADII[0], RADII[4]);
    println!("RESULT G3 homogeneous ramp 900 K to 1000 K: tau = {tau:.6e} m^2, retained {ret:.9}, released {rel:.9}");
    assert_eq!(dec, 0.0);
    convergence_study(
        "G3 ramp (P28)",
        &|n, j| temperature_run(n, j, &law, &hist, end),
        (ret, rel, dec),
        400,
    );
}

#[test]
fn first_order_in_time_with_five_different_laws_and_a_temperature_ramp() {
    // Five layers with different activation temperatures (Document 5, 3.2, case 2): no series
    // reference exists, so the orders are the Cauchy orders of the release fraction at fixed
    // grid, which must round to 1.
    let end = 2.0e5;
    let hist = ramp(900.0, 1000.0, end);
    let mut f = Vec::new();
    for steps in [100usize, 200, 400, 800, 1600] {
        let base = vi_steps(RADII, [20; 5], D5, steps, end, steps);
        let pi = pin(
            base,
            end,
            0.0,
            SourceModel::none(),
            arrhenius(synthetic_laws(), hist.clone()),
        );
        let run = run_physics(&pi, &[]).unwrap();
        assert_eq!(
            run.meta.assemblies, steps,
            "the matrix changes at every step"
        );
        f.push(run.series.last().unwrap().1.release_fraction);
    }
    let diffs: Vec<f64> = f.windows(2).map(|w| w[0] - w[1]).collect();
    let orders: Vec<f64> = diffs.windows(2).map(|w| (w[0] / w[1]).log2()).collect();
    println!("RESULT G3 five laws ramp: F = {f:.9?}, Cauchy orders {orders:.3?}");
    for o in &orders {
        assert!((0.5..1.5).contains(o), "order {o} does not round to 1");
    }
}

// ---------------------------------------------------------------------------------------
// G4: the combined scheme: balance at every step, non-negativity, bounds
// ---------------------------------------------------------------------------------------

fn combined_cases() -> Vec<(&'static str, PhysicsInput)> {
    let cells = [12, 8, 6, 8, 6];
    let (end, steps) = (2.0e5, 500usize);
    let base = vi_steps(RADII, cells, D5, steps, end, 1);
    let y = FissionYield::new(YieldKind::Cumulative, 0.06).unwrap();
    let kernel_fission = SourceModel::kernel_fission(
        &TimeFunction::table(
            vec![0.0, 5.0e4, 2.0e5],
            vec![2.0e19, 5.0e19, 1.0e19],
            Interpolation::Linear,
        )
        .unwrap(),
        y,
        ProductKind::DirectFissionProduct,
    )
    .unwrap();
    // Synthetic source in all five materials (so that the OPyC source, hence Q_N, is not zero).
    let all_five = SourceModel {
        per_material: [
            TimeFunction::Constant(1.0e-5),
            TimeFunction::Constant(2.0e-6),
            TimeFunction::table(vec![0.0, 1.0e5], vec![0.0, 3.0e-6], Interpolation::Step).unwrap(),
            TimeFunction::Constant(1.0e-6),
            TimeFunction::table(
                vec![0.0, 2.0e5],
                vec![4.0e-6, 1.0e-6],
                Interpolation::Linear,
            )
            .unwrap(),
        ],
    };
    let temp = ramp(1000.0, 1100.0, end);
    vec![
        (
            "source only",
            pin(
                base.clone(),
                end,
                0.0,
                kernel_fission,
                DiffusivityModel::Constant(D5),
            ),
        ),
        (
            "decay only",
            pin(
                base.clone(),
                end,
                1.0e-5,
                SourceModel::none(),
                DiffusivityModel::Constant(D5),
            ),
        ),
        (
            "temperature only",
            pin(
                base.clone(),
                end,
                0.0,
                SourceModel::none(),
                arrhenius(synthetic_laws(), temp.clone()),
            ),
        ),
        (
            "source, decay and temperature, all five materials",
            pin(
                base.clone(),
                end,
                1.0e-5,
                all_five,
                arrhenius(synthetic_laws(), temp),
            ),
        ),
    ]
}

#[test]
fn balance_holds_at_every_step_within_the_rounding_bound_f46() {
    for (label, pi) in combined_cases() {
        let run = run_physics(&pi, &[]).unwrap();
        assert_eq!(run.meta.bound_kind, BoundKind::Combined, "{label}");
        assert_eq!(
            run.series.len(),
            run.meta.steps + 1,
            "{label}: every step is output"
        );
        let mut worst_rel = 0.0f64;
        let mut worst_ratio = 0.0f64;
        for (k, (_, d, p)) in run.series.iter().enumerate() {
            assert!(
                d.balance_residual.abs() <= d.balance_residual_bound,
                "{label} step {k}: |eps| {:e} > bound {:e}",
                d.balance_residual.abs(),
                d.balance_residual_bound
            );
            worst_rel = worst_rel.max(p.balance_residual_relative.abs());
            if d.balance_residual_bound > 0.0 {
                worst_ratio = worst_ratio.max(d.balance_residual.abs() / d.balance_residual_bound);
            }
        }
        let last = run.series.last().unwrap();
        let total = run.meta.initial_inventory + last.2.generated;
        println!(
            "RESULT G4 {label}: worst |eps|/(M0+Mgen) = {worst_rel:.3e}, worst |eps|/bound = {worst_ratio:.3e}, final bound/(M0+Mgen) = {:.3e}, M_gen/M0 = {:.4e}, M_dec/M0 = {:.4e}, F = {:.6}, F_gen = {:.6}",
            last.1.balance_residual_bound / total,
            last.2.generated / run.meta.initial_inventory,
            last.2.decayed / run.meta.initial_inventory,
            last.1.release_fraction,
            last.2.release_fraction_of_total
        );
        // The decayed amount is not part of the release.
        if pi.decay_constant_per_s > 0.0 {
            assert!(last.2.decayed > 0.0);
        } else {
            assert_eq!(last.2.decayed, 0.0);
        }
    }
}

#[test]
fn concentrations_are_non_negative_release_is_monotone_and_fgen_is_in_range() {
    for (label, pi) in combined_cases() {
        let run = run_physics(&pi, &[]).unwrap();
        for (k, (_, d, p)) in run.series.iter().enumerate() {
            // P31 and P6: exact comparison with zero.
            assert!(
                d.min_concentration >= 0.0,
                "{label} step {k}: min {}",
                d.min_concentration
            );
            assert!(p.release_fraction_of_total >= 0.0);
            // 0 <= F_gen <= 1 (P31) in exact arithmetic; in floating point M_rel = M0 + M_gen -
            // M - M_dec - eps with M, M_dec >= 0, so F_gen <= 1 + |eps| / (M0 + M_gen) and the
            // bound of the residual limits it.
            let total = run.meta.initial_inventory + p.generated;
            assert!(
                p.release_fraction_of_total <= 1.0 + d.balance_residual_bound / total,
                "{label} step {k}: F_gen {}",
                p.release_fraction_of_total
            );
            if k > 0 {
                // M_rel non-decreasing: every increment dt Phi is non-negative (Phi >= 0 by P31).
                assert!(d.release_rate.unwrap() >= 0.0);
                assert!(
                    d.cumulative_release >= run.series[k - 1].1.cumulative_release,
                    "{label} step {k}: release decreased"
                );
                assert!(p.generated >= run.series[k - 1].2.generated);
                assert!(p.decayed >= run.series[k - 1].2.decayed);
            }
        }
        // The row margin of every assembled matrix is at least 1 + kappa (P24). The computed
        // margin fl(fl(B - |L|) - |U|) of the stored entries differs from the exact one by the
        // 3 roundings of B = fl(fl(fl(1 - L) - U) + kappa) and the 2 of the subtractions, each
        // relative to the row sum: gamma_5 times the largest row sum of absolute values.
        let kappa = run.meta.decay_number;
        let slack = gamma(5.0) * run.meta.matrix_norm_inf;
        assert!(
            run.meta.matrix_min_row_margin >= (1.0 + kappa) - slack,
            "{label}: margin {} norm {}",
            run.meta.matrix_min_row_margin,
            run.meta.matrix_norm_inf
        );
        println!(
            "RESULT P24 {label}: smallest row margin {:.12} (1 + kappa = {:.12}, rounding allowance {slack:.2e}), smallest pivot margin {:.6}, largest multiplier {:.6}",
            run.meta.matrix_min_row_margin,
            1.0 + kappa,
            run.meta.min_pivot_margin,
            run.meta.max_multiplier
        );
    }
}

#[test]
fn combined_bound_function_matches_a_hand_evaluation() {
    // One evaluation of (F46) written out term by term with the same constants, for numbers
    // that make every term visible. This checks the code against the transcription of the
    // formula; the derivation of the formula itself is Document 5, Section 5.8.
    let t = BoundTerms {
        n: 100,
        steps_taken: 1000,
        sum_s: 3.0e-4,
        chi_inventory: 2.0e-3,
        inventory: 4.0e-5,
        initial_inventory: 5.0e-5,
        released: 1.0e-5,
        generated: 2.0e-5,
        decayed: 3.0e-6,
    };
    let u = U;
    let g = |k: f64| k * u / (1.0 - k * u);
    let c_a = g(4.0) / (1.0 - g(4.0)) + g(4.0) / (1.0 - g(1.0));
    let eta = 4.0 * (3.0 * 100.0 + 1000.0 + 10.0) * u;
    let bracket = 4.0 * PI * c_a * 3.0e-4
        + u * 2.0e-3
        + g(100.0) * (4.0e-5 + 5.0e-5)
        + (g(5.0) + 1000.0 * u) * 1.0e-5
        + (g(101.0) + g(3.0) + 1000.0 * u) * 2.0e-5
        + (g(101.0) + 1000.0 * u) * 3.0e-6
        + g(2.0) * (4.0e-5 + 1.0e-5 + 5.0e-5 + 2.0e-5);
    let expect = (1.0 + eta) * bracket;
    let got = combined_balance_bound(&t);
    // Both sides evaluate the same sum of non-negative terms with a different association: they
    // differ by at most gamma_16 (about 14 operations on each side).
    assert!(
        (got - expect).abs() <= gamma(16.0) * expect,
        "{got} {expect}"
    );
    // The bound grows with every term.
    let mut t2 = t;
    t2.generated *= 2.0;
    assert!(combined_balance_bound(&t2) > got);
}

#[test]
fn combined_bound_also_covers_a_run_without_source_and_decay() {
    // With G = 0, lambda = 0 and a temperature ramp that changes the matrix, (F46) is the
    // bound in use (the baseline bound is for constant D). The residual is rounding.
    let end = 2.0e5;
    let base = vi_steps(RADII, [20; 5], D5, 400, end, 1);
    let pi = pin(
        base,
        end,
        0.0,
        SourceModel::none(),
        arrhenius(synthetic_laws(), ramp(1000.0, 1200.0, end)),
    );
    let run = run_physics(&pi, &[]).unwrap();
    assert_eq!(run.meta.bound_kind, BoundKind::Combined);
    for (k, (_, d, _)) in run.series.iter().enumerate() {
        assert!(
            d.balance_residual.abs() <= d.balance_residual_bound,
            "step {k}"
        );
    }
}

// ---------------------------------------------------------------------------------------
// Refusals and input rules
// ---------------------------------------------------------------------------------------

#[test]
fn direct_yield_source_is_refused_for_an_activation_product() {
    // Decision 15: Ag-110m, Cs-134 and Eu-154 get no direct fission-yield source term.
    let y = FissionYield::new(YieldKind::Cumulative, 0.01).unwrap();
    for rate in [TimeFunction::Constant(1.0e19), TimeFunction::Constant(0.0)] {
        let e = SourceModel::kernel_fission(&rate, y, ProductKind::ActivationProduct).unwrap_err();
        assert!(e.to_string().contains("activation product"), "{e}");
    }
    // A direct product is accepted, with G = y F_dot / N_A.
    let ok = SourceModel::kernel_fission(
        &TimeFunction::Constant(6.022_140_76e19),
        y,
        ProductKind::DirectFissionProduct,
    )
    .unwrap();
    match &ok.per_material[0] {
        TimeFunction::Constant(v) => {
            // 6.02214076e19 * 0.01 / 6.02214076e23 = 1e-6 (3 roundings of the reference).
            assert!((v - 1.0e-6).abs() <= gamma(4.0) * 1.0e-6, "{v}");
        }
        other => panic!("{other:?}"),
    }
    assert!(ok.per_material[1..].iter().all(TimeFunction::is_zero));
    // A negative or non-finite fission rate is refused.
    assert!(SourceModel::kernel_fission(
        &TimeFunction::Constant(-1.0),
        y,
        ProductKind::DirectFissionProduct
    )
    .is_err());
}

#[test]
fn invalid_physics_inputs_are_refused() {
    let end = 1000.0;
    let base = vi([3, 3, 3, 3, 3], D5, 100.0, end, 1);
    let mk =
        |lambda: f64, src: SourceModel, d: DiffusivityModel| pin(base.clone(), end, lambda, src, d);
    let ok = mk(0.0, SourceModel::none(), DiffusivityModel::Constant(D5));
    assert!(run_physics(&ok, &[]).is_ok());
    assert!(run_physics(
        &mk(-1.0, SourceModel::none(), DiffusivityModel::Constant(D5)),
        &[]
    )
    .is_err());
    assert!(run_physics(
        &mk(
            f64::NAN,
            SourceModel::none(),
            DiffusivityModel::Constant(D5)
        ),
        &[]
    )
    .is_err());
    assert!(run_physics(
        &mk(0.0, kernel_source(-1.0), DiffusivityModel::Constant(D5)),
        &[]
    )
    .is_err());
    // A linear history that ends before the run is refused, not extended.
    let short =
        TimeFunction::table(vec![0.0, 500.0], vec![1.0, 1.0], Interpolation::Linear).unwrap();
    let mut s = SourceModel::none();
    s.per_material[0] = short.clone();
    assert!(run_physics(&mk(0.0, s, DiffusivityModel::Constant(D5)), &[]).is_err());
    assert!(run_physics(
        &mk(0.0, SourceModel::none(), arrhenius(synthetic_laws(), short)),
        &[]
    )
    .is_err());
    // A non-positive temperature is refused.
    assert!(run_physics(
        &mk(
            0.0,
            SourceModel::none(),
            arrhenius(synthetic_laws(), TimeFunction::Constant(0.0))
        ),
        &[]
    )
    .is_err());
}

// ---------------------------------------------------------------------------------------
// G1: the discrete steady state of the weights (P22, P22a), the release ratio, stability (P26)
// ---------------------------------------------------------------------------------------

/// Residuals of the discrete steady rows `K_out (C_{i+1} - C_i) - K_in (C_i - C_{i-1}) + Q_i`
/// (centre row: `6 D (C_1 - C_0)/h^2 + G`) for the exact steady profile `c`, divided by the
/// scale `S_i = K_out (C_{i+1} + C_i) + K_in (C_i + C_{i-1}) + Q_i`. Returns the largest ratio
/// over the rows `0 ..= N-1`.
fn steady_row_ratio(grid: &Grid, d: [f64; 5], q: &[f64], c: &[f64], g0: f64) -> (f64, f64, f64) {
    let k = Coefficients::new(grid, d, 1.0).unwrap().k_outer;
    let n = grid.n;
    let mut worst = 0.0f64;
    let (mut sum_res, mut sum_scale) = (0.0f64, 0.0f64);
    // Centre row (F12) with the pointwise source g_0.
    let h = grid.dr_region[0];
    let fo6 = 6.0 * d[0] / (h * h);
    let res0 = fo6 * (c[1] - c[0]) + g0;
    worst = worst.max(res0.abs() / (fo6 * (c[1] + c[0]) + g0));
    for i in 1..n {
        let res = k[i] * (c[i + 1] - c[i]) - k[i - 1] * (c[i] - c[i - 1]) + q[i];
        let scale = k[i] * (c[i + 1] + c[i]) + k[i - 1] * (c[i] + c[i - 1]) + q[i];
        worst = worst.max(res.abs() / scale);
        sum_res += res.abs();
        sum_scale += scale;
    }
    (worst, sum_res, sum_scale)
}

/// Radii, interval counts, diffusivities and whether the source is uniform in all materials.
type SteadyCase = ([f64; 5], [usize; 5], [f64; 5], bool);

#[test]
fn steady_state_rows_of_the_weights_are_exact_p22_and_p22a() {
    // P22: for a kernel-only source the nodal values of the exact steady profile (14) solve the
    // discrete steady rows exactly, for any grid and any diffusivities; P22a: also for one
    // material with one spacing and a uniform source. Evaluated here in double precision.
    // Tolerance, per row, relative to the scale S_i of the absolute terms: the nodal values are
    // sums of non-negative terms formed outward from R_5 (at most 9 roundings per term, one per
    // hop and per interface), so each has relative error at most gamma_30; the stored K has 3
    // roundings, the difference and the product 2 more, the weights Q_i 5, and the three
    // additions of the row 3: gamma_30 + gamma_5 + gamma_5 + gamma_3, which gamma_45 covers. The
    // stored radii are not exactly uniform in the kernel (r_i = fl(i dr)), which changes the
    // quadratic-profile identity by a relative 2 u at most: gamma_45 still covers it.
    let bound = gamma(45.0);
    let g = 0.37;
    let uniform_radii = [1.0e-4, 2.0e-4, 3.0e-4, 4.0e-4, 5.0e-4];
    let cases: [SteadyCase; 4] = [
        (RADII, [7, 11, 3, 5, 13], D5, false),
        (RADII, [3, 1, 2, 1, 4], D5, false),
        (RADII, [10; 5], D5, false),
        (uniform_radii, [8; 5], [3.0e-12; 5], true),
    ];
    for (radii, cells, d, uniform) in cases {
        let grid = Grid::from_radii(radii, cells).unwrap();
        let gbar = if uniform {
            [g; 5]
        } else {
            [g, 0.0, 0.0, 0.0, 0.0]
        };
        let nodes = source_nodes(&grid, gbar);
        // The exact steady profile at the stored node radii.
        let n = grid.n;
        let mut c = vec![0.0; n + 1];
        if uniform {
            for i in 0..=n {
                c[i] = g * (radii[4] * radii[4] - grid.r[i] * grid.r[i]) / (6.0 * d[0]);
            }
        } else {
            // C at the outer radius of each material, from R_5 inward.
            let mut c_at = [0.0; 5];
            let a = |k: usize| g * radii[0] * radii[0] * radii[0] / (3.0 * d[k]);
            for k in (1..5).rev() {
                c_at[k - 1] = c_at[k] + a(k) * (1.0 / radii[k - 1] - 1.0 / radii[k]);
            }
            for i in 0..=n {
                let r = grid.r[i];
                let k = if i == 0 {
                    0
                } else {
                    grid.region_of_interval[i - 1].index()
                };
                c[i] = if k == 0 {
                    c_at[0] + g * (radii[0] * radii[0] - r * r) / (6.0 * d[0])
                } else {
                    c_at[k] + a(k) * (1.0 / r - 1.0 / radii[k])
                };
            }
            c[n] = 0.0;
        }
        let (worst, sum_res, sum_scale) = steady_row_ratio(&grid, d, &nodes.q, &c, nodes.g[0]);
        println!(
            "RESULT P22{} steady rows {cells:?}: worst |row residual| / scale = {worst:.3e} (bound {bound:.3e})",
            if uniform { "a" } else { "" }
        );
        assert!(worst <= bound, "{cells:?}: {worst} > {bound}");
        if !uniform {
            // The steady release equals the generation rate. The rows of 1 ..= N-1 add up
            // (the inner flows cancel the outer flows of the next row, K_1,inner = 0, C_N = 0) to
            // sum_i res_i = sum_i Q_i - K_{N-1} C_{N-1}, so |flow - sum Q| <= sum |res_i|. The
            // computed residuals carry their own roundings (at most gamma_5 S_i each, as above)
            // and the product K C and the sum of the weights gamma_{N+3} of their size.
            let k = Coefficients::new(&grid, d, 1.0).unwrap();
            let flow = k.k_last() * c[n - 1];
            let allowed =
                sum_res + gamma(5.0) * sum_scale + gamma(n as f64 + 3.0) * (flow + nodes.sum_q);
            assert!(
                (flow - nodes.sum_q).abs() <= allowed,
                "{cells:?}: release {flow} generation {}",
                nodes.sum_q
            );
            println!(
                "RESULT P22 steady release against generation {cells:?}: relative difference {:.3e}",
                (flow - nodes.sum_q).abs() / nodes.sum_q
            ); // Negative control: the plain half-interval weight at N_kernel breaks the steady
               // state at a size that dwarfs the bound (Document 5, Section 1.6).
            let nk = grid.interface_nodes[0];
            let mut q_plain = nodes.q.clone();
            q_plain[nk] = grid.r[nk] * grid.r[nk] / 2.0 * grid.dr_inner(nk) * g;
            let (bad, _, _) = steady_row_ratio(&grid, d, &q_plain, &c, nodes.g[0]);
            assert!(bad > 1.0e3 * bound, "{cells:?}: control {bad}");
            println!("RESULT P22 negative control (plain weight at N_kernel) {cells:?}: {bad:.3e}");
        }
    }
}

#[test]
fn release_to_generation_ratio_is_one_at_steady_state_without_decay() {
    // One Backward Euler step of 1e13 s with a uniform source and no decay reaches the steady
    // state, where the release equals the generation (D2, F42). From the balance (F41) with
    // lambda = 0 and one step, M_gen = dt p S and dt Phi = M_gen + M^0 - M^1 - eps, so
    // beta - 1 = (M^0 - M^1 - eps) / M_gen; the division by M_gen adds gamma_3.
    let radii = [1.0e-4, 2.0e-4, 3.0e-4, 4.0e-4, 5.0e-4];
    let end = 1.0e13;
    let base = vi_steps(radii, [10; 5], [1.0e-10; 5], 1, end, 1);
    let pi = pin(
        base,
        end,
        0.0,
        uniform_source(1.0e-3),
        DiffusivityModel::Constant([1.0e-10; 5]),
    );
    let run = run_physics(&pi, &[]).unwrap();
    let (_, d, p) = run.series.last().unwrap();
    let beta = p.release_to_generation_ratio.unwrap();
    let m0 = run.meta.initial_inventory;
    let allowed = (m0 + d.inventory + d.balance_residual_bound) / p.generated + gamma(3.0);
    println!(
        "RESULT F42 release/generation at steady state: beta - 1 = {:.3e} (allowed {allowed:.3e})",
        beta - 1.0
    );
    assert!((beta - 1.0).abs() <= allowed);
    assert!(
        allowed < 1.0e-8,
        "the case must resolve beta = 1: {allowed}"
    );
    // At j = 0 and for a run without source the ratio is not defined.
    assert!(run.series[0].2.release_to_generation_ratio.is_none());
    let none = run_physics(
        &PhysicsInput::baseline(vi([3; 5], D5, 100.0, 1000.0, 1), 1000.0),
        &[],
    )
    .unwrap();
    assert!(none
        .series
        .iter()
        .all(|(_, _, p)| p.release_to_generation_ratio.is_none()));
}

#[test]
fn maximum_norm_does_not_grow_with_a_temperature_history_p26() {
    // No source, no decay, a changing temperature: ||C^{j+1}||_inf <= ||C^j||_inf (P26). In
    // floating point the solve satisfies (A + E) x = b with |E| <= kappa_t |A| (derivation in
    // `numerical::diagnostics::BalanceBound`) and the stored matrix has the row margin 1 up to
    // gamma_5 times its row sum; hence ||x||_inf <= ||b||_inf / (1 - (kappa_t + gamma_5) ||A||_inf)
    // with b = C^j exactly (the source term adds an exact zero). ||A||_inf is the largest row
    // sum over all assembled matrices (reported by the run).
    let end = 2.0e5;
    let base = vi_steps(RADII, [20; 5], D5, 400, end, 1);
    let pi = pin(
        base,
        end,
        0.0,
        SourceModel::none(),
        arrhenius(synthetic_laws(), ramp(1000.0, 1200.0, end)),
    );
    let run = run_physics(&pi, &[]).unwrap();
    let factor = 1.0 / (1.0 - (BalanceBound::kappa_t() + gamma(5.0)) * run.meta.matrix_norm_inf);
    let mut worst = 0.0f64;
    for w in run.series.windows(2) {
        let (a, b) = (w[0].1.max_concentration, w[1].1.max_concentration);
        assert!(b <= a * factor, "max {b} after {a}, factor {factor}");
        worst = worst.max(b / a);
    }
    println!(
        "RESULT P26 largest ratio of consecutive maxima {worst:.12} (allowed factor {factor:.12})"
    );
}
