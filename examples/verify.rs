//! Verification driver. Every FV result is compared with an independent analytical
//! answer, and every claim printed here is also an assertion, so a PASS line means the
//! stated tolerance was actually met.

use anyhow::Result;
use std::f64::consts::PI;
use std::time::Instant;
use strato::{
    analytical::sturm_liouville::{Layer, ModalRelease},
    config::Config,
    numerical::finite_volume as fv,
    postprocessing::comparison::{self as compare, ConvergenceStudy},
};

const SYNTHETIC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/inputs/synthetic/synthetic_five_layer.toml"
);

fn uniform_d(cfg: &mut Config, d: f64) {
    for r in &mut cfg.regions {
        r.diffusivity_m2_s = d;
    }
}

fn final_release(cfg: &Config) -> Result<f64> {
    let mesh = fv::build_mesh(cfg)?;
    let (rows, _) = fv::run(cfg, &mesh)?;
    Ok(rows.last().expect("initial row").1.release_fraction)
}

/// FV error against `exact` at dt and dt/2, plus the error of the time-extrapolated value
/// (see `compare::time_extrapolated_release`). Backward Euler is first order in time, so
/// the first two errors should differ by about a factor 2 and the extrapolated error
/// should be much smaller.
fn time_check(cfg: &Config, exact: f64) -> Result<(f64, f64, f64)> {
    let mut fine = cfg.clone();
    fine.time.dt_s = cfg.time.dt_s / 2.0;
    let (f1, f2) = (final_release(cfg)?, final_release(&fine)?);
    let extrapolated = compare::time_extrapolated_release(cfg)?;
    Ok((
        (f1 - exact).abs(),
        (f2 - exact).abs(),
        (extrapolated - exact).abs(),
    ))
}

/// Classical result (Crank): uniform initial concentration in a homogeneous sphere,
/// F = 1 - (6/pi^2) sum exp(-n^2 pi^2 D t / R^2) / n^2.
fn sphere_release_uniform(t: f64, r: f64, d: f64) -> f64 {
    let mut s = 0.0;
    for n in 1..10000 {
        let term = (-((n * n) as f64) * PI * PI * d * t / (r * r)).exp() / (n * n) as f64;
        s += term;
        if term.abs() < 1e-15 {
            break;
        }
    }
    1.0 - 6.0 / (PI * PI) * s
}

fn homogeneous_uniform() -> Result<()> {
    let mut cfg = Config::load(SYNTHETIC)?;
    uniform_d(&mut cfg, 1e-10);
    for r in &mut cfg.regions {
        r.cells = 40;
    }
    cfg.time.dt_s = 0.5;
    cfg.time.end_time_s = 1.0e3;
    let mesh = fv::build_mesh(&cfg)?;
    let mut c = vec![1.0; mesh.cells.len()];
    let m0 = fv::inventory(&mesh, &c);
    let mut rel = 0.0;
    let steps = (cfg.time.end_time_s / cfg.time.dt_s) as usize;
    for _ in 0..steps {
        c = fv::backward_euler_step(&mesh, &c, cfg.time.dt_s)?;
        rel += cfg.time.dt_s * mesh.outer_conductance * c[c.len() - 1];
    }
    let f = rel / m0;
    let exact = sphere_release_uniform(cfg.time.end_time_s, mesh.outer_radius, 1e-10);
    let err = (f - exact).abs();
    println!("HOMOGENEOUS_UNIFORM_CRANK fv={f:.10} exact={exact:.10} abs={err:.3e}");
    assert!(err < 1e-4, "homogeneous FV error {err:e}");
    Ok(())
}

fn exact_at(cfg: &Config, t: f64) -> Result<f64> {
    compare::exact_release_at(cfg, t)
}

fn homogeneous_kernel() -> Result<()> {
    let mut cfg = Config::load(SYNTHETIC)?;
    uniform_d(&mut cfg, 1e-10);
    cfg.time.dt_s = 0.0625;
    cfg.time.end_time_s = 500.0;
    let exact = exact_at(&cfg, 500.0)?;
    let (e1, e2, ex) = time_check(&cfg, exact)?;
    println!(
        "HOMOGENEOUS_KERNEL_MODAL exact={exact:.10} err(dt)={e1:.3e} err(dt/2)={e2:.3e} err(extrapolated)={ex:.3e}"
    );
    assert!(
        (1.8..=2.2).contains(&(e1 / e2)),
        "time order ratio {}",
        e1 / e2
    );
    assert!(ex < 1e-5, "extrapolated error {ex:e}");
    Ok(())
}

fn two_layer_control() -> Result<()> {
    // Five labels, but regions 2-5 share one diffusivity, so the only D jump is at r = 50 um.
    let mut cfg = Config::load(SYNTHETIC)?;
    let radii = [50e-6, 60e-6, 70e-6, 85e-6, 100e-6];
    let ds = [1e-10, 1e-9, 1e-9, 1e-9, 1e-9];
    for (i, r) in cfg.regions.iter_mut().enumerate() {
        r.outer_radius_m = radii[i];
        r.diffusivity_m2_s = ds[i];
    }

    // Mesh refinement at two mid-transient times where F is far from 0 and 1, so the
    // interface treatment actually matters. Time error is removed by extrapolation.
    for t in [1.0, 4.0] {
        cfg.time.end_time_s = t;
        cfg.time.dt_s = 0.002;
        let exact = exact_at(&cfg, t)?;
        let mut errors = Vec::new();
        for n in [20usize, 40, 80, 160] {
            for r in &mut cfg.regions {
                r.cells = n;
            }
            let (_, _, extrapolated) = time_check(&cfg, exact)?;
            println!("TWO_LAYER_MODAL t={t} cells/region={n} exact={exact:.10} err(extrapolated)={extrapolated:.3e}");
            errors.push(extrapolated);
        }
        for (i, w) in errors.windows(2).enumerate() {
            let order = (w[0] / w[1]).log2();
            assert!(
                (1.8..=2.2).contains(&order),
                "t={t} spatial order {i}: {order}"
            );
        }
        assert!(
            *errors.last().unwrap() < 3e-5,
            "t={t}: finest error {:e}",
            errors.last().unwrap()
        );
    }

    // Late time (1 - F ~ 1e-5): the slowest mode only.
    cfg.time.end_time_s = 32.0;
    cfg.time.dt_s = 0.004;
    for r in &mut cfg.regions {
        r.cells = 20;
    }
    let exact = exact_at(&cfg, 32.0)?;
    let (_, _, ex) = time_check(&cfg, exact)?;
    println!("TWO_LAYER_MODAL t=32 exact={exact:.10} err(extrapolated)={ex:.3e}");
    assert!(ex < 1e-6, "t=32: extrapolated error {ex:e}");
    Ok(())
}

fn five_layer() -> Result<()> {
    let cfg = Config::load(SYNTHETIC)?;
    let start = Instant::now();
    let study = compare::convergence_study(&cfg)?;
    println!("FIVE_LAYER exact_F_end={:.12e}", study.exact_final);

    for p in &study.spatial {
        println!(
            "SPATIAL cells={} F_time_extrapolated={:.12e} abs_error_vs_exact={:.3e}",
            p.cells, p.fv_release, p.abs_error
        );
    }
    let s_orders = ConvergenceStudy::orders(&study.spatial);
    println!("SPATIAL_ORDERS {s_orders:.3?}");
    for p in &study.temporal {
        println!(
            "TEMPORAL dt={:.3e} F={:.12e} abs_error_vs_exact={:.3e}",
            p.dt_s, p.fv_release, p.abs_error
        );
    }
    let t_orders = ConvergenceStudy::orders(&study.temporal);
    println!("TEMPORAL_ORDERS {t_orders:.3?}");

    // Observed orders. Spatial: second order. Temporal: Backward Euler is first order.
    for (i, o) in s_orders.iter().enumerate() {
        assert!((1.8..=2.2).contains(o), "spatial order {i}: {o}");
    }
    for (i, o) in t_orders.iter().enumerate() {
        assert!((0.8..=1.2).contains(o), "temporal order {i}: {o}");
    }
    let finest = study.spatial.last().unwrap();
    assert!(
        finest.abs_error < 1e-6,
        "finest spatial error {:e}",
        finest.abs_error
    );

    // Positivity and conservation across the whole mesh sequence.
    for n in [20usize, 40, 80, 160, 320] {
        let mut c = cfg.clone();
        for r in &mut c.regions {
            r.cells = n;
        }
        let mesh = fv::build_mesh(&c)?;
        let (rows, _) = fv::run(&c, &mesh)?;
        let last = &rows.last().unwrap().1;
        let rel = last.balance_residual.abs() / rows[0].1.inventory;
        println!(
            "CONSERVATION cells={} F={:.12e} balance_rel={rel:.3e} min_c={:.3e}",
            mesh.cells.len(),
            last.release_fraction,
            last.min_concentration
        );
        assert!(last.min_concentration >= 0.0, "negative concentration");
        assert!((0.0..=1.0).contains(&last.release_fraction));
        assert!(rel < 1e-12, "balance residual {rel:e}");
        let worst = rows
            .iter()
            .map(|(_, d)| d.balance_residual.abs() / rows[0].1.inventory)
            .fold(0.0, f64::max);
        assert!(worst < 1e-12, "worst balance residual {worst:e}");
    }
    println!("VERIFY_RUNTIME_S {:.6}", start.elapsed().as_secs_f64());
    Ok(())
}

fn modal_completeness() -> Result<()> {
    // Cross-check of the analytical module itself: the weights of the eigenfunction
    // expansion must sum to 1 as more modes are included (F(0) = 0).
    let cfg = Config::load(SYNTHETIC)?;
    let layers: Vec<Layer> = compare::layers_from_config(&cfg);
    let mut last_gap = f64::INFINITY;
    for lambda_max in [2.0e-3, 2.0e-2, 2.0e-1] {
        let s = ModalRelease::new(&layers, lambda_max, compare::MAX_MODES)?;
        let gap = (1.0 - s.weights.iter().sum::<f64>()).abs();
        println!(
            "MODAL_COMPLETENESS modes={} 1-sum(w)={gap:.3e}",
            s.eigenvalues.len()
        );
        assert!(gap < last_gap, "weight deficit must shrink with more modes");
        last_gap = gap;
    }
    assert!(last_gap < 5e-2);
    Ok(())
}

fn main() -> Result<()> {
    homogeneous_uniform()?;
    homogeneous_kernel()?;
    two_layer_control()?;
    modal_completeness()?;
    five_layer()?;
    println!("FV_VERIFICATION PASS");
    Ok(())
}
