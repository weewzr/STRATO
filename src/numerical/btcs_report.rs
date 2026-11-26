//! Reporting quantities of the BTCS solver that a front end shows beside the solution: the
//! a-posteriori rounding bound of the inventory balance, the interface-flux consistency check,
//! the layer shares of a profile and the radial extent of the colour band of each node. They
//! live here, with the solver and its tests, so that the WebAssembly wrapper only forwards them.
//!
//! None of them changes the solution; they are computed from the same matrix, vectors and grid
//! as a run.

use super::{
    btcs::Stepper,
    coefficients::Coefficients,
    grid::{Grid, ValidatedInput},
    interfaces::interface_flux_both_sides,
    inventory::{initial_vector, inventory_by_region},
};
use anyhow::Result;
use std::f64::consts::PI;

/// Unit roundoff of `f64`, `u = 2^-53`.
pub const UNIT_ROUNDOFF: f64 = f64::EPSILON / 2.0;

/// A-posteriori rounding bound on `|eps^j| / M^0` after each step `j = 0 ..= steps`, where
/// `eps^j` is the balance residual (F29) and `M^0` the initial inventory `m0`.
///
/// The identity (F29) follows by summing the exact-form rows over `i = 1 ..= N-1`; the computed
/// solution satisfies the stored rows up to the backward error of tridiagonal elimination
/// (`4u |A||x|`), the stored entries differ from the exact ones by at most `2u` relative, hence
/// per step `|Delta eps| <= 4 pi 16 u sum_i V_i (|A||x|)_i`, and the final roundings add
/// `(N + 8) u M^0`. The constant 16 is the safety factor chosen before any run. It is a bound
/// on rounding only; it is not a discretisation error. The function steps once more with the
/// library's [`Stepper`], so the matrix and the vectors are exactly those of the run.
pub fn balance_bound_history(input: &ValidatedInput, grid: &Grid, m0: f64) -> Result<Vec<f64>> {
    let mut stepper = Stepper::new(grid, input.d, input.dt)?;
    let (mut c_old, _, _) = initial_vector(grid, input.c_init);
    let mut c_new = vec![0.0; grid.n + 1];
    let tail = (grid.n as f64 + 8.0) * UNIT_ROUNDOFF;
    let mut cumulative = 0.0;
    let mut out = Vec::with_capacity(input.steps + 1);
    out.push(tail);
    for _ in 0..input.steps {
        stepper.step(&c_old, &mut c_new)?;
        let a = &stepper.assembly().matrix;
        let mut s = 0.0;
        for i in 1..grid.n {
            let left = a.lower[i].abs() * c_new[i - 1].abs();
            let right = a.upper[i].abs() * c_new[i + 1].abs();
            s += grid.v[i] * (left + a.diag[i].abs() * c_new[i].abs() + right);
        }
        cumulative += 4.0 * PI * 16.0 * UNIT_ROUNDOFF * s / m0;
        out.push(cumulative + tail);
        std::mem::swap(&mut c_old, &mut c_new);
    }
    Ok(out)
}

/// The two expressions (F31) of the outward flux through each interface node for one step, their
/// difference and its rounding-level bound.
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceFluxReport {
    /// Node indices of the four interfaces.
    pub nodes: Vec<usize>,
    /// Radii of those nodes (m).
    pub radii_m: Vec<f64>,
    /// Outward flux from the inner-side expression (mol/s).
    pub from_inner: Vec<f64>,
    /// Outward flux from the outer-side expression (mol/s).
    pub from_outer: Vec<f64>,
    /// `|from_inner - from_outer|` (mol/s).
    pub difference: Vec<f64>,
    /// Bound on that difference: `100 u` times the sum of the magnitudes of the terms (mol/s).
    pub bound: Vec<f64>,
    /// Whether every difference is within its bound.
    pub within_bound: bool,
    /// Largest of `difference / (|from_inner| + |from_outer|)` over the interfaces.
    pub worst_relative: f64,
}

/// Interface fluxes of one step (equation F31, both expressions) with the rounding-level
/// tolerance `100 u` times the sum of the magnitudes of the terms, as in the library's test E.
pub fn interface_flux_report(
    grid: &Grid,
    coeff: &Coefficients,
    c_old: &[f64],
    c_new: &[f64],
    dt: f64,
) -> InterfaceFluxReport {
    let mut rep = InterfaceFluxReport {
        nodes: grid.interface_nodes.to_vec(),
        radii_m: Vec::new(),
        from_inner: Vec::new(),
        from_outer: Vec::new(),
        difference: Vec::new(),
        bound: Vec::new(),
        within_bound: true,
        worst_relative: 0.0,
    };
    for &i in &grid.interface_nodes {
        let (fi, fo) = interface_flux_both_sides(grid, coeff, c_old, c_new, dt, i);
        let radius = grid.r[i];
        let scale = 4.0
            * PI
            * (coeff.k_outer[i - 1] * (c_new[i - 1].abs() + c_new[i].abs())
                + coeff.k_outer[i] * (c_new[i].abs() + c_new[i + 1].abs())
                + (radius * radius * (grid.dr_inner(i) + grid.dr_outer(i)) / 2.0 / dt)
                    * (c_new[i].abs() + c_old[i].abs()));
        let tol = 100.0 * UNIT_ROUNDOFF * scale;
        let diff = (fi - fo).abs();
        rep.within_bound &= diff <= tol;
        let denom = fi.abs() + fo.abs();
        if denom > 0.0 {
            rep.worst_relative = rep.worst_relative.max(diff / denom);
        }
        rep.radii_m.push(radius);
        rep.from_inner.push(fi);
        rep.from_outer.push(fo);
        rep.difference.push(diff);
        rep.bound.push(tol);
    }
    rep
}

/// Inventory of each layer of one profile, and each layer's share of the total.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerShares {
    /// Layer inventories (mol), in the order kernel to outer PyC.
    pub layer_inventory_mol: [f64; 5],
    /// Layer inventory divided by the sum of the layers (dimensionless). `None` when the total
    /// inventory is not positive, where a share is undefined.
    pub share: Option<Vec<f64>>,
    /// `|sum of layers - total| / total` where the total is positive, else 0.
    pub sum_relative_error: f64,
}

/// Layer inventories and shares of the profile `c`, with `total_mol` its total inventory
/// (`inventory(grid, c)`).
pub fn layer_shares(grid: &Grid, c: &[f64], total_mol: f64) -> LayerShares {
    let layers = inventory_by_region(grid, c);
    if total_mol > 0.0 {
        LayerShares {
            layer_inventory_mol: layers,
            share: Some(layers.iter().map(|l| l / total_mol).collect()),
            sum_relative_error: (layers.iter().sum::<f64>() - total_mol).abs() / total_mol,
        }
    } else {
        LayerShares {
            layer_inventory_mol: layers,
            share: None,
            sum_relative_error: 0.0,
        }
    }
}

/// Radial extent of the colour band of each node, `N + 2` edges: `0`, the midpoints between
/// neighbouring nodes, and the outer radius. Presentation geometry; it is not a property of the
/// discretisation.
pub fn ring_edges(grid: &Grid) -> Vec<f64> {
    let n = grid.n;
    let mut edges = Vec::with_capacity(n + 2);
    edges.push(0.0);
    for i in 0..n {
        edges.push((grid.r[i] + grid.r[i + 1]) / 2.0);
    }
    edges.push(grid.r[n]);
    edges
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numerical::{
        btcs::run_input,
        grid::Grid,
        inventory::{inventory, release_fraction},
    };

    const RADII: [f64; 5] = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4];
    const D: [f64; 5] = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13];

    fn input(cells: [usize; 5], dt: f64, steps: usize) -> ValidatedInput {
        ValidatedInput {
            radii: RADII,
            d: D,
            n_intervals: cells,
            c_init: 1.0,
            dt,
            steps,
            output_every: 1,
        }
    }

    #[test]
    fn bound_history_starts_at_the_tail_and_never_decreases() {
        let inp = input([4, 4, 4, 4, 4], 1000.0, 20);
        let g = Grid::build(&inp).unwrap();
        let (_, m0, _) = initial_vector(&g, 1.0);
        let h = balance_bound_history(&inp, &g, m0).unwrap();
        assert_eq!(h.len(), inp.steps + 1);
        assert_eq!(h[0], (g.n as f64 + 8.0) * UNIT_ROUNDOFF);
        assert!(h.windows(2).all(|w| w[1] >= w[0]));
    }

    #[test]
    fn observed_residual_is_inside_the_bound_at_every_step() {
        let inp = input([6, 6, 6, 6, 6], 2000.0, 30);
        let g = Grid::build(&inp).unwrap();
        let times: Vec<f64> = (0..=inp.steps).map(|s| s as f64 * inp.dt).collect();
        let res = run_input(&inp, &times).unwrap();
        let m0 = res.meta.initial_inventory;
        let h = balance_bound_history(&inp, &g, m0).unwrap();
        for (k, (t, d)) in res.series.iter().enumerate() {
            let step = (t / inp.dt).round() as usize;
            assert!(
                d.balance_residual.abs() / m0 <= h[step],
                "row {k}: residual {:e} above bound {:e}",
                d.balance_residual.abs() / m0,
                h[step]
            );
        }
    }

    #[test]
    fn interface_fluxes_agree_within_their_bound_and_are_continuous() {
        let inp = input([5, 5, 5, 5, 5], 5000.0, 6);
        let g = Grid::build(&inp).unwrap();
        let times: Vec<f64> = (0..=inp.steps).map(|s| s as f64 * inp.dt).collect();
        let res = run_input(&inp, &times).unwrap();
        let stepper = Stepper::new(&g, D, inp.dt).unwrap();
        let coeff = stepper.assembly().coeff.clone();
        let old = &res.snapshots[inp.steps - 1].1;
        let new = &res.snapshots[inp.steps].1;
        let rep = interface_flux_report(&g, &coeff, old, new, inp.dt);
        assert_eq!(rep.nodes.len(), 4);
        assert_eq!(rep.radii_m.len(), 4);
        assert!(rep.within_bound, "{rep:?}");
        for k in 0..4 {
            assert_eq!(
                rep.difference[k],
                (rep.from_inner[k] - rep.from_outer[k]).abs()
            );
            assert!(rep.difference[k] <= rep.bound[k]);
        }
        let worst = (0..4)
            .map(|k| rep.difference[k] / (rep.from_inner[k].abs() + rep.from_outer[k].abs()))
            .fold(0.0f64, f64::max);
        assert_eq!(rep.worst_relative, worst);
    }

    #[test]
    fn layer_shares_sum_to_one_and_undefined_for_empty_profile() {
        let inp = input([4, 4, 4, 4, 4], 1000.0, 3);
        let g = Grid::build(&inp).unwrap();
        let (c, _, _) = initial_vector(&g, 1.0);
        let total = inventory(&g, &c);
        let ls = layer_shares(&g, &c, total);
        let share = ls.share.expect("positive inventory");
        assert!((share.iter().sum::<f64>() - 1.0).abs() <= 1e-15 * 5.0);
        assert!(ls.sum_relative_error <= 1e-14);
        assert_eq!(release_fraction(0.0, total), 0.0);
        let zero = vec![0.0; g.n + 1];
        let empty = layer_shares(&g, &zero, 0.0);
        assert!(empty.share.is_none());
        assert_eq!(empty.sum_relative_error, 0.0);
    }

    #[test]
    fn ring_edges_cover_the_particle_in_order() {
        let inp = input([3, 4, 5, 6, 7], 1000.0, 1);
        let g = Grid::build(&inp).unwrap();
        let e = ring_edges(&g);
        assert_eq!(e.len(), g.n + 2);
        assert_eq!(e[0], 0.0);
        assert_eq!(*e.last().unwrap(), g.r[g.n]);
        assert!(e.windows(2).all(|w| w[1] > w[0]));
        for i in 0..=g.n {
            assert!(
                e[i] <= g.r[i] && g.r[i] <= e[i + 1],
                "node {i} outside its band"
            );
        }
    }
}
