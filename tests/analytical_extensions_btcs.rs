//! Verifier check (iv) of the science design (Section 4.2, open point O10): the production BTCS
//! solver with the generation and decay rows (Document 5), run on the five different layers
//! D_FIVE with a constant kernel source and a decay constant, converges to the Method B
//! extension (B3) to (B5) at order 2 in space and order 1 in time.
//!
//! The extension is evaluated with `KernelRun` on 20 000 modes of the layered series. The
//! BTCS values are the inventory `M`, the cumulative release `M_rel` and the decayed amount
//! `M_dec` at the end time, each divided by `M^0 + M_gen`.
//!
//! Time orders: Cauchy differences of five step counts at 40 intervals per material (the spatial
//! error cancels). Space orders: the time error is removed by the Richardson extrapolation
//! `2 F(2J) - F(J)` of two fine step counts, then the error against the extension is formed at
//! 10, 20, 40 and 80 intervals per material. The observed orders are asserted to round to 1 in
//! time and 2 in space (decision 49), and the remaining time error to be small against the
//! spatial error at 80 intervals. Numerical verification of the BTCS against the analytical
//! solution of the same equation, with synthetic diffusivities: not experimental validation.
//!
//! The extension is raw here (`KernelRun::state`): the end time is far above the relaxation
//! time `t_v` of the series, so the values are not in the region where the raw values are wrong.

use std::f64::consts::PI;
use strato::{
    analytical::{
        extensions::{Decay, KernelRun, PiecewiseSource},
        sturm_liouville::{Layer, ModalRelease},
    },
    numerical::{
        grid::ValidatedInput,
        physics_run::{
            run_physics, DiffusivityModel, PhysicsInput, PhysicsRun, SourceModel, TimeFunction,
        },
    },
};

const RADII: [f64; 5] = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4];
/// Synthetic diffusivities in m^2 s^-1 (D_kernel, D_buffer, D_IPyC, D_SiC, D_OPyC).
const D_FIVE: [f64; 5] = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13];
const LAMBDA: f64 = 2.0e-6; // s^-1
const G_KERNEL: f64 = 5.0e-6; // mol m^-3 s^-1
const T_END: f64 = 3.0e5; // s

fn vk() -> f64 {
    4.0 / 3.0 * PI * RADII[0].powi(3)
}

fn btcs(cells: usize, steps: usize) -> PhysicsRun {
    let base = ValidatedInput {
        radii: RADII,
        d: D_FIVE,
        n_intervals: [cells; 5],
        c_init: 1.0,
        dt: T_END / steps as f64,
        steps,
        output_every: steps,
    };
    let mut source = SourceModel::none();
    source.per_material[0] = TimeFunction::Constant(G_KERNEL);
    let input = PhysicsInput {
        base,
        end_time_s: T_END,
        decay_constant_per_s: LAMBDA,
        source,
        diffusivity: DiffusivityModel::Constant(D_FIVE),
    };
    run_physics(&input, &[]).unwrap()
}

/// `(M, M_rel, M_dec) / (M^0 + M_gen)` of the BTCS run at the end time.
fn btcs_fractions(cells: usize, steps: usize) -> [f64; 3] {
    let run = btcs(cells, steps);
    let (_, d, p) = run.series.last().unwrap();
    let total = run.meta.initial_inventory + p.generated;
    [
        d.inventory / total,
        d.cumulative_release / total,
        p.decayed / total,
    ]
}

/// The same fractions from the extension (B3) to (B5).
fn extension_fractions() -> [f64; 3] {
    let layers: Vec<Layer> = RADII
        .iter()
        .zip(D_FIVE)
        .map(|(&r, d)| Layer {
            outer_radius: r,
            diffusivity: d,
        })
        .collect();
    let series = ModalRelease::new(&layers, 45.0 / 5.0e3, 20_000).unwrap();
    assert!(
        T_END > 10.0 * series.min_valid_time(),
        "the end time must lie far above the relaxation time"
    );
    let m0 = vk(); // C_init = 1 mol m^-3 in the kernel
    let source = PiecewiseSource::constant(G_KERNEL, 10.0 * T_END).unwrap();
    let run = KernelRun::new(&series, m0, vk(), source, &Decay::Uniform(LAMBDA)).unwrap();
    let st = run.state(T_END).unwrap();
    let total = st.total_input();
    assert!(st.balance_residual().abs() <= 1e-12 * total);
    [
        st.inventory / total,
        st.released / total,
        st.decayed / total,
    ]
}

#[test]
fn check_iv_btcs_with_the_d5_rows_converges_to_the_extension_at_orders_2_and_1() {
    let reference = extension_fractions();
    println!(
        "RESULT check (iv) extension: M {:.9} M_rel {:.9} M_dec {:.9} (fractions of M^0 + M_gen)",
        reference[0], reference[1], reference[2]
    );
    // Time orders at 40 intervals per material.
    let times: Vec<[f64; 3]> = [100usize, 200, 400, 800, 1600]
        .iter()
        .map(|&j| btcs_fractions(40, j))
        .collect();
    // Space study after Richardson extrapolation in time.
    let extrap = |n: usize, j: usize| {
        let a = btcs_fractions(n, j);
        let b = btcs_fractions(n, 2 * j);
        [2.0 * b[0] - a[0], 2.0 * b[1] - a[1], 2.0 * b[2] - a[2]]
    };
    let j_space = 800;
    let spaces = [10usize, 20, 40, 80];
    let fine: Vec<[f64; 3]> = spaces.iter().map(|&n| extrap(n, j_space)).collect();
    let finer_80 = extrap(80, 2 * j_space);
    for (k, name) in ["M", "M_rel", "M_dec"].iter().enumerate() {
        let diffs: Vec<f64> = times.windows(2).map(|w| w[0][k] - w[1][k]).collect();
        let t_orders: Vec<f64> = diffs.windows(2).map(|w| (w[0] / w[1]).log2()).collect();
        let errs: Vec<f64> = fine.iter().map(|f| (f[k] - reference[k]).abs()).collect();
        let s_orders: Vec<f64> = errs.windows(2).map(|w| (w[0] / w[1]).log2()).collect();
        let remaining_time_error = (finer_80[k] - fine[3][k]).abs();
        println!(
            "RESULT check (iv) {name}: errors at 10, 20, 40, 80 per material {:?}, space orders {:?}, time orders {:?}, remaining time error {:.2e}",
            errs.iter().map(|e| format!("{e:.3e}")).collect::<Vec<_>>(),
            s_orders.iter().map(|o| format!("{o:.3}")).collect::<Vec<_>>(),
            t_orders.iter().map(|o| format!("{o:.3}")).collect::<Vec<_>>(),
            remaining_time_error
        );
        for o in &t_orders {
            assert!(
                (0.5..1.5).contains(o),
                "{name}: time order {o} does not round to 1"
            );
        }
        for o in &s_orders {
            assert!(
                (1.5..2.5).contains(o),
                "{name}: space order {o} does not round to 2"
            );
        }
        assert!(
            remaining_time_error <= 0.1 * errs[3],
            "{name}: time error {remaining_time_error} not negligible against {}",
            errs[3]
        );
    }
}
