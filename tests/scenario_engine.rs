//! Invariants of the scenario engine (`strato::numerical::scenario`), science design Section 5.
//!
//! The geometry and the numbers below are small illustrative settings chosen so that the runs
//! take milliseconds and release is visible; they are not registered values and no result of
//! these tests is a prediction.

use strato::numerical::physics_run::{Interpolation, TimeFunction};
use strato::numerical::scenario::{
    post_event_fraction, run_scenario, Diffusion, DiffusivityAt, InitialState, RangeCode,
    ScenarioConfig, ScenarioDefinition, StepSegment,
};

const RADII: [f64; 5] = [1.0e-4, 1.5e-4, 1.8e-4, 2.2e-4, 2.5e-4];
const INTERVALS: [usize; 5] = [20, 6, 6, 10, 6];
const LAMBDA: f64 = 2.0e-6;
const G_REF: f64 = 3.0e-3;

/// Diffusivity law used only by these tests: Arrhenius-like in the temperature, different in every
/// layer, large enough that release is visible in a few hundred hours.
fn law(_t_s: f64, t_c: f64) -> anyhow::Result<DiffusivityAt> {
    let t_k = t_c + 273.15;
    let d = std::array::from_fn(|k| {
        let e = 2.0e3 * (1.0 + 0.2 * k as f64);
        1.0e-10 * (1.0 + 0.5 * k as f64) * (-e / t_k).exp()
    });
    Ok((d, [RangeCode::Within; 5]))
}

fn config<'a>(initial: InitialState, profile_times_s: Vec<f64>) -> ScenarioConfig<'a> {
    ScenarioConfig {
        radii_m: RADII,
        intervals: INTERVALS,
        initial,
        decay_constant_per_s: LAMBDA,
        reference_generation_mol_m3_s: G_REF,
        diffusion: Diffusion::OfTemperature(&law),
        profile_times_s,
        extra_boundaries_s: vec![],
    }
}

fn linear(times: &[f64], values: &[f64]) -> TimeFunction {
    TimeFunction::table(times.to_vec(), values.to_vec(), Interpolation::Linear).unwrap()
}

/// Operation with a shutdown: 600 s of normal operation, then generation ceases (a step to zero)
/// while the temperature falls along a ramp.
fn shutdown() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "test-shutdown".into(),
        generation_factor: TimeFunction::table(
            vec![0.0, 600.0, 2000.0],
            vec![1.0, 0.0, 0.0],
            Interpolation::Step,
        )
        .unwrap(),
        temperature_c: linear(&[0.0, 600.0, 900.0, 2000.0], &[1000.0, 1000.0, 300.0, 300.0]),
        plan: vec![
            StepSegment {
                end_s: 600.0,
                dt_max_s: 30.0,
            },
            StepSegment {
                end_s: 900.0,
                dt_max_s: 10.0,
            },
            StepSegment {
                end_s: 2000.0,
                dt_max_s: 100.0,
            },
        ],
        event_times_s: vec![600.0],
    }
}

#[test]
fn generation_ceases_at_shutdown_while_decay_and_diffusion_continue() {
    let def = shutdown();
    let run = run_scenario(&def, &config(InitialState::Fresh, vec![])).unwrap();
    let at = run.row_at(600.0).expect("shutdown is a step boundary");
    assert!(at.generated_mol > 0.0);
    let after: Vec<_> = run.rows.iter().filter(|r| r.t_s > 600.0).collect();
    assert!(after.len() > 10);
    // Generation: the generated amount does not change after the event, and the source of every
    // later step is exactly zero.
    for r in &after {
        assert_eq!(r.generated_mol, at.generated_mol);
        assert_eq!(r.kernel_source_mol_m3_s, 0.0);
    }
    // Decay continues: the decayed amount grows at every step while the inventory is positive.
    let mut prev = at.decayed_mol;
    for r in &after {
        assert!(r.decayed_mol > prev);
        prev = r.decayed_mol;
    }
    // Diffusion continues: the release grows (the particle is still releasing through the
    // coatings) and the inventory falls.
    let mut prev_rel = at.released_mol;
    let mut prev_inv = at.inventory_mol;
    for r in &after {
        // While the temperature falls (up to 900 s) the release grows at every step; on the cold
        // plateau the diffusivity of the coatings is so small that a step can add nothing in
        // double precision, so there the release does not decrease.
        if r.t_s <= 900.0 {
            assert!(r.released_mol > prev_rel);
        } else {
            assert!(r.released_mol >= prev_rel);
        }
        assert!(r.inventory_mol < prev_inv);
        prev_rel = r.released_mol;
        prev_inv = r.inventory_mol;
    }
    assert!(prev_rel > at.released_mol);
    // The post-event release fraction of Section 5.1 is defined from the event on.
    let f_end = post_event_fraction(&run, 600.0, 2000.0).unwrap();
    assert!(f_end > 0.0 && f_end < 1.0);
    assert_eq!(post_event_fraction(&run, 600.0, 300.0), None);
}

#[test]
fn the_balance_closes_at_every_step_within_the_documented_bound() {
    let def = shutdown();
    let cfg = config(InitialState::Fresh, vec![]);
    let run = run_scenario(&def, &cfg).unwrap();
    assert_eq!(run.steps, 20 + 30 + 11);
    assert!(run.distinct_steps >= 3, "piecewise-uniform steps");
    for r in &run.rows {
        assert!(
            r.balance_residual_mol.abs() <= r.balance_bound_mol,
            "|eps| = {:e} exceeds the bound {:e} at t = {} s",
            r.balance_residual_mol.abs(),
            r.balance_bound_mol,
            r.t_s
        );
    }
    assert!(run.max_residual_over_bound <= 1.0);
    // Positivity (P-series) of the scheme: no concentration is negative.
    assert!(run.rows.iter().all(|r| r.min_concentration >= 0.0));
    // The kernel-only start closes the same balance.
    let run2 = run_scenario(
        &def,
        &config(InitialState::KernelUniform { c_init: 1.0 }, vec![]),
    )
    .unwrap();
    assert!(run2
        .rows
        .iter()
        .all(|r| r.balance_residual_mol.abs() <= r.balance_bound_mol));
    // A start from a stored profile closes it with the inventory of the profile as M^0.
    let mid = run_scenario(&def, &config(InitialState::Fresh, vec![600.0])).unwrap();
    let (_, profile) = &mid.profiles[0];
    let run3 = run_scenario(
        &def,
        &config(InitialState::Profile(profile.clone()), vec![]),
    )
    .unwrap();
    assert!(run3.initial_inventory_mol > 0.0);
    assert!(run3
        .rows
        .iter()
        .all(|r| r.balance_residual_mol.abs() <= r.balance_bound_mol));
}

#[test]
fn a_zero_change_scenario_equals_normal_operation_bit_for_bit() {
    // Histories that never change, written as tables with several breakpoints, against the
    // reference run of normal operation on the same step boundaries.
    let def = ScenarioDefinition {
        id: "test-zero-change".into(),
        generation_factor: linear(&[0.0, 700.0, 1500.0, 2000.0], &[1.0, 1.0, 1.0, 1.0]),
        temperature_c: linear(&[0.0, 400.0, 900.0, 2000.0], &[1000.0, 1000.0, 1000.0, 1000.0]),
        plan: vec![
            StepSegment {
                end_s: 900.0,
                dt_max_s: 45.0,
            },
            StepSegment {
                end_s: 2000.0,
                dt_max_s: 120.0,
            },
        ],
        event_times_s: vec![],
    };
    let reference = def.normal_operation(1.0, 1000.0).unwrap();
    let cfg = config(InitialState::Fresh, vec![]);
    let a = run_scenario(&def, &cfg).unwrap();
    let b = run_scenario(&reference, &cfg).unwrap();
    assert_eq!(a.rows.len(), b.rows.len());
    for (x, y) in a.rows.iter().zip(&b.rows) {
        assert_eq!(x.t_s.to_bits(), y.t_s.to_bits());
        assert_eq!(x.inventory_mol.to_bits(), y.inventory_mol.to_bits());
        assert_eq!(x.released_mol.to_bits(), y.released_mol.to_bits());
        assert_eq!(x.generated_mol.to_bits(), y.generated_mol.to_bits());
        assert_eq!(x.decayed_mol.to_bits(), y.decayed_mol.to_bits());
        assert_eq!(
            x.balance_residual_mol.to_bits(),
            y.balance_residual_mol.to_bits()
        );
        assert_eq!(x.balance_bound_mol.to_bits(), y.balance_bound_mol.to_bits());
    }
    assert_eq!(a.final_profile.len(), b.final_profile.len());
    for (x, y) in a.final_profile.iter().zip(&b.final_profile) {
        assert_eq!(x.to_bits(), y.to_bits());
    }
    // And a scenario that does change something differs from the reference.
    let mut changed = def.clone();
    changed.generation_factor = linear(&[0.0, 700.0, 1500.0, 2000.0], &[1.0, 1.0, 0.5, 0.5]);
    let c = run_scenario(&changed.normal_operation(1.0, 1000.0).unwrap(), &cfg).unwrap();
    let d = run_scenario(&changed, &cfg).unwrap();
    assert_ne!(
        c.rows.last().unwrap().generated_mol.to_bits(),
        d.rows.last().unwrap().generated_mol.to_bits()
    );
}

#[test]
fn a_fresh_particle_reports_the_release_fraction_of_the_generated_amount_from_the_first_step() {
    let def = shutdown();
    let run = run_scenario(&def, &config(InitialState::Fresh, vec![])).unwrap();
    let r0 = &run.rows[0];
    assert_eq!(r0.release_fraction_of_total, None);
    assert_eq!(r0.release_fraction_of_initial, None);
    assert_eq!(run.initial_inventory_mol, 0.0);
    let r1 = &run.rows[1];
    let f = r1.release_fraction_of_total.unwrap();
    assert!((f - r1.released_mol / r1.generated_mol).abs() <= 1e-15 * f.abs());
    assert!(run.rows.iter().skip(1).all(|r| {
        let f = r.release_fraction_of_total.unwrap();
        (0.0..=1.0 + 1e-12).contains(&f)
    }));
}

#[test]
fn the_diffusivity_is_taken_at_the_middle_of_each_step() {
    // A linear temperature ramp that is not aligned with the steps: the diffusivity of a step is
    // the law at the temperature of the middle of the step (F36), not at its start or end.
    let def = ScenarioDefinition {
        id: "test-ramp".into(),
        generation_factor: TimeFunction::constant(0.0).unwrap(),
        temperature_c: linear(&[0.0, 1000.0], &[500.0, 1100.0]),
        plan: vec![StepSegment {
            end_s: 1000.0,
            dt_max_s: 70.0,
        }],
        event_times_s: vec![],
    };
    let run = run_scenario(&def, &config(InitialState::KernelUniform { c_init: 1.0 }, vec![]))
        .unwrap();
    let steps = def.steps(&[]).unwrap();
    for (row, (a, b)) in run.rows.iter().skip(1).zip(&steps) {
        let mid_c = 500.0 + 0.6 * (0.5 * (a + b));
        let (expected, _) = law(0.5 * (a + b), mid_c).unwrap();
        for k in 0..5 {
            assert_eq!(row.d_m2_s[k].to_bits(), expected[k].to_bits());
        }
    }
}
