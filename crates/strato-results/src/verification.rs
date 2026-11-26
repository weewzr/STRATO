//! The verification figures: observed orders, balance residuals against their derived bounds,
//! and the solver-to-solver agreement of Method A and Method B. They are numerical verification
//! of the solvers against each other and against exact references. They are never called
//! experimental validation.
//!
//! The orders come from `strato_compare::studies`, the study the comparison program and its
//! tests already use (the synthetic homogeneous case against the exact series); this module
//! does not recompute them differently. The orders of generation, decay and temperature
//! dependence are the subject of `tests/btcs_physics.rs` and are not duplicated here.

use strato_compare::data::Radii;
use strato_compare::json::J;
use strato_compare::studies::{spatial_convergence, temporal_convergence, ConvergenceRow};

use crate::curves::Stage1;
use crate::output::{json_file, Prov};

/// The study sizes of the full run (those of `strato-compare`) and of the quick run.
fn sizes(quick: bool) -> (usize, usize, usize, usize, usize) {
    if quick {
        (3, 200, 4, 10, 2)
    } else {
        (6, 2000, 9, 25, 32)
    }
}

fn rows_json(rows: &[ConvergenceRow]) -> J {
    J::Arr(
        rows.iter()
            .map(|r| {
                J::obj(vec![
                    ("level", J::Int(r.level as i64)),
                    ("step", J::Num(r.step)),
                    ("count", J::Int(r.count as i64)),
                    ("release_fraction_solver", J::Num(r.f_solver)),
                    ("release_fraction_analytical", J::Num(r.f_analytic)),
                    ("analytical_truncation_bound", J::Num(r.f_analytic_bound)),
                    ("absolute_error", J::Num(r.error)),
                    ("observed_order", J::opt(r.order)),
                ])
            })
            .collect(),
    )
}

/// The convergence file and its CSV rows.
///
/// # Errors
///
/// A study that fails.
pub fn convergence(
    prov: &Prov,
    radii: &Radii,
    quick: bool,
) -> Result<(String, Vec<Vec<String>>), String> {
    let (sl, ss, tl, ts, tm) = sizes(quick);
    let spatial = spatial_convergence(radii, sl, ss)?;
    let temporal = temporal_convergence(radii, tl, ts, tm)?;
    let mut csv = Vec::new();
    for (name, rows) in [("spatial", &spatial), ("temporal", &temporal)] {
        for r in rows.iter() {
            csv.push(vec![
                name.to_string(),
                r.level.to_string(),
                format!("{:e}", r.step),
                r.count.to_string(),
                format!("{:e}", r.error),
                r.order.map_or(String::new(), |o| format!("{o:e}")),
            ]);
        }
    }
    let last_order = |rows: &[ConvergenceRow]| rows.last().and_then(|r| r.order);
    let json = json_file(
        prov,
        "numerical verification: convergence orders",
        vec![
            (
                "case",
                J::s("synthetic homogeneous verification case against the exact series (numerical verification, not experimental data); the solver of every stage shares its spatial and temporal operators with this case"),
            ),
            ("spatial", rows_json(&spatial)),
            ("temporal", rows_json(&temporal)),
            ("last_spatial_order", J::opt(last_order(&spatial))),
            ("last_temporal_order", J::opt(last_order(&temporal))),
            (
                "expected_orders",
                J::s("second order in space and first order in time (Documents 1 and 2)"),
            ),
            (
                "stages_2_to_5",
                J::s("the orders of generation, decay and temperature-dependent diffusion are asserted by tests/btcs_physics.rs (observed orders 1 in time and 2 in space for the combined scheme) and are not recomputed here"),
            ),
        ],
    );
    Ok((json, csv))
}

/// One entry of the conservation file.
pub struct Conservation {
    /// Key of the curve.
    pub key: String,
    /// Largest `|eps| / bound`.
    pub residual_over_bound: f64,
    /// Largest relative residual.
    pub relative_residual: f64,
    /// Steps.
    pub steps: usize,
}

/// The conservation file: the balance residuals with their derived bounds.
pub fn conservation(prov: &Prov, entries: &[Conservation]) -> (String, Vec<Vec<String>>) {
    let rows = entries
        .iter()
        .map(|e| {
            J::obj(vec![
                ("key", J::s(e.key.clone())),
                ("steps", J::Int(e.steps as i64)),
                (
                    "max_abs_residual_over_derived_bound",
                    J::Num(e.residual_over_bound),
                ),
                ("max_relative_residual", J::Num(e.relative_residual)),
                ("within_bound", J::Bool(e.residual_over_bound <= 1.0)),
            ])
        })
        .collect();
    let csv = entries
        .iter()
        .map(|e| {
            vec![
                e.key.clone(),
                e.steps.to_string(),
                format!("{:e}", e.residual_over_bound),
                format!("{:e}", e.relative_residual),
            ]
        })
        .collect();
    (
        json_file(
            prov,
            "numerical verification: conservation of the inventory balance",
            vec![
                (
                    "statement",
                    J::s("the balance residual of (F41) is reported beside its derived rounding bound ((F46) for runs with generation, decay or a changing temperature; the baseline bound otherwise); a ratio of at most 1 means the balance closes within the documented bound"),
                ),
                ("entries", J::Arr(rows)),
            ],
        ),
        csv,
    )
}

/// Solver-to-solver agreement of Method A and Method B at the points where both are drawn.
pub fn agreement(prov: &Prov, curves: &[Stage1]) -> (String, Vec<Vec<String>>) {
    let mut entries = Vec::new();
    let mut csv = Vec::new();
    for c in curves {
        let mut largest: Option<(f64, f64, f64)> = None; // (relative difference, time, absolute)
        let mut final_diff: Option<(f64, f64)> = None;
        let mut n = 0usize;
        for k in 0..c.t_h.len() {
            if let (Some(b), true) = (c.f_b[k], c.resolved[k]) {
                if c.f_a[k] > 0.0 && b > 0.0 {
                    n += 1;
                    let abs = (c.f_a[k] - b).abs();
                    let rel = abs / b;
                    if largest.is_none_or(|l| rel > l.0) {
                        largest = Some((rel, c.t_h[k], abs));
                    }
                    final_diff = Some((rel, c.t_h[k]));
                }
            }
        }
        entries.push(J::obj(vec![
            ("key", J::s(c.key.clone())),
            ("points_compared", J::Int(n as i64)),
            ("largest_relative_difference", J::opt(largest.map(|l| l.0))),
            ("time_of_largest_h", J::opt(largest.map(|l| l.1))),
            ("largest_absolute_difference", J::opt(largest.map(|l| l.2))),
            ("final_relative_difference", J::opt(final_diff.map(|l| l.0))),
            ("final_time_h", J::opt(final_diff.map(|l| l.1))),
            ("method_b_accuracy_floor", J::opt(c.b_floor)),
            ("method_b_valid_from_h", J::opt(c.b_valid_from_h)),
        ]));
        csv.push(vec![
            c.key.clone(),
            n.to_string(),
            largest.map_or(String::new(), |l| format!("{:e}", l.0)),
            final_diff.map_or(String::new(), |l| format!("{:e}", l.0)),
            c.b_floor.map_or(String::new(), |v| format!("{v:e}")),
            c.b_valid_from_h.map_or(String::new(), |v| format!("{v:e}")),
        ]);
    }
    (
        json_file(
            prov,
            "numerical verification: Method A against Method B",
            vec![
                (
                    "statement",
                    J::s("the two methods solve the same stage-1 problem independently; the relative difference is taken where both are drawn (resolved Method A, masked Method B). This is solver-to-solver verification, not agreement with an experiment."),
                ),
                ("entries", J::Arr(entries)),
            ],
        ),
        csv,
    )
}
