//! Radial grid of the five-layer BTCS solver: nodes on every material interface,
//! material of each interval, and the storage weights.
//!
//! Implements Document 2 (`docs/mathematics/btcs/02_final_formulation.md`) Sections 2 to 4:
//! equations (F1) to (F5) and (F7), following pseudocode routines PS-02 and PS-03 of
//! Document 3. Every unknown is a nodal value `C_i^j` at a node `r_i`; there are no cells.
//! All quantities are SI base units and `f64`.

// The loops mirror the index form of the equations in Document 2 (node `i`, material `k`).
#![allow(clippy::needless_range_loop)]

use anyhow::{bail, Result};

/// The five materials in radial order. The index is the position in every per-material array.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    /// Fuel kernel, `0 <= r <= R_1`.
    Kernel,
    /// Porous buffer, `R_1 <= r <= R_2`.
    Buffer,
    /// Inner pyrolytic carbon, `R_2 <= r <= R_3`.
    Ipyc,
    /// Silicon carbide, `R_3 <= r <= R_4`.
    Sic,
    /// Outer pyrolytic carbon, `R_4 <= r <= R_5`.
    Opyc,
}

impl Region {
    /// All five materials in radial order.
    pub const ALL: [Region; 5] = [
        Region::Kernel,
        Region::Buffer,
        Region::Ipyc,
        Region::Sic,
        Region::Opyc,
    ];

    /// Position 0..=4 in every per-material array.
    pub fn index(self) -> usize {
        match self {
            Region::Kernel => 0,
            Region::Buffer => 1,
            Region::Ipyc => 2,
            Region::Sic => 3,
            Region::Opyc => 4,
        }
    }

    /// Descriptive name used in outputs and error messages.
    pub fn name(self) -> &'static str {
        match self {
            Region::Kernel => "kernel",
            Region::Buffer => "buffer",
            Region::Ipyc => "IPyC",
            Region::Sic => "SiC",
            Region::Opyc => "OPyC",
        }
    }
}

/// What a node is, by position (Document 2 Section 3, "Node classes").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeClass {
    /// `i = 0`.
    Centre,
    /// Strictly inside one material.
    Interior(Region),
    /// On the interface between two materials, `i = N_kernel ..= N_SiC`.
    Interface {
        /// Material on the inner side.
        inner: Region,
        /// Material on the outer side.
        outer: Region,
    },
    /// `i = N`, the absorbing outer surface.
    Outer,
}

/// Input that has passed validation (PS-01). Radii in m, diffusivities in m^2/s,
/// concentration in mol/m^3, time in s.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedInput {
    /// `R_1 .. R_5`.
    pub radii: [f64; 5],
    /// `D_kernel .. D_OPyC`.
    pub d: [f64; 5],
    /// `n_kernel .. n_OPyC`, grid intervals per material.
    pub n_intervals: [usize; 5],
    /// `C_init`.
    pub c_init: f64,
    /// Effective `Delta t`.
    pub dt: f64,
    /// `J_steps`.
    pub steps: usize,
    /// Output stride in steps.
    pub output_every: usize,
}

/// Piecewise-uniform radial grid with a node on every interface.
#[derive(Clone, Debug)]
pub struct Grid {
    /// `N`, the index of the outer node (the grid has `N + 1` nodes).
    pub n: usize,
    /// `r_i`, length `N + 1`, in m.
    pub r: Vec<f64>,
    /// `N_kernel, N_buffer, N_IPyC, N_SiC` (F1).
    pub interface_nodes: [usize; 4],
    /// Material of interval `(r_i, r_{i+1})`, length `N` (F4).
    pub region_of_interval: Vec<Region>,
    /// Uniform spacing inside each material (F2), in m.
    pub dr_region: [f64; 5],
    /// `V_i`, length `N + 1`; `v[0] = 0` and `v[N]` unused (0.0), in m^3 (F7).
    pub v: Vec<f64>,
}

impl Grid {
    /// Builds the grid of a validated input. See [`Grid::from_radii`].
    ///
    /// # Errors
    ///
    /// As [`Grid::from_radii`].
    pub fn build(input: &ValidatedInput) -> Result<Grid> {
        Grid::from_radii(input.radii, input.n_intervals)
    }

    /// Grid generation, PS-02 and PS-03. Implements (F1) node indices, (F2) spacing per
    /// material, (F3) node radii (interface and outer radii assigned exactly, never
    /// accumulated), (F4) the material of each interval, (F5) spacings as differences of the
    /// stored radii, and (F7) the storage weights `V_i` with `V_0 = 0`.
    ///
    /// # Errors
    ///
    /// Radii not increasing from a positive first radius, a zero interval count, radii that
    /// collapse in floating point, or a non-finite or non-positive weight.
    pub fn from_radii(radii: [f64; 5], n_intervals: [usize; 5]) -> Result<Grid> {
        let mut previous = 0.0;
        for k in 0..5 {
            if !(radii[k].is_finite() && radii[k] > previous) {
                bail!(
                    "region {}: outer radius must be finite and exceed the previous radius",
                    Region::ALL[k].name()
                );
            }
            if n_intervals[k] == 0 {
                bail!(
                    "region {}: needs at least one interval",
                    Region::ALL[k].name()
                );
            }
            previous = radii[k];
        }

        // (F1): node index of each interface and of the outer node.
        let mut nodes_at = [0usize; 5];
        let mut acc = 0usize;
        for k in 0..5 {
            acc += n_intervals[k];
            nodes_at[k] = acc;
        }
        let n = acc;
        let interface_nodes = [nodes_at[0], nodes_at[1], nodes_at[2], nodes_at[3]];

        // (F2): uniform spacing inside each material.
        let mut dr_region = [0.0; 5];
        let mut inner_radius = 0.0;
        for k in 0..5 {
            dr_region[k] = (radii[k] - inner_radius) / n_intervals[k] as f64;
            inner_radius = radii[k];
        }

        // (F3): node radii; r_0 = 0 and the interface and outer radii are assigned exactly.
        let mut r = vec![0.0; n + 1];
        let mut first = 0usize;
        let mut r_prev = 0.0;
        for k in 0..5 {
            let last = nodes_at[k];
            for i in (first + 1)..last {
                r[i] = r_prev + (i - first) as f64 * dr_region[k];
            }
            r[last] = radii[k];
            first = last;
            r_prev = radii[k];
        }
        for i in 1..=n {
            if !(r[i].is_finite() && r[i] > r[i - 1]) {
                bail!("node radii are not strictly increasing at node {i}");
            }
        }

        // (F4): material of each interval.
        let mut region_of_interval = vec![Region::Kernel; n];
        let mut i = 0usize;
        for k in 0..5 {
            while i < nodes_at[k] {
                region_of_interval[i] = Region::ALL[k];
                i += 1;
            }
        }

        // (F7): trapezoidal storage weight; V_0 = 0 literally, V_N unused.
        let mut v = vec![0.0; n + 1];
        for i in 1..n {
            v[i] = r[i] * r[i] * ((r[i] - r[i - 1]) + (r[i + 1] - r[i])) / 2.0;
            if !(v[i].is_finite() && v[i] > 0.0) {
                bail!("storage weight at node {i} is not finite and positive");
            }
        }

        Ok(Grid {
            n,
            r,
            interface_nodes,
            region_of_interval,
            dr_region,
            v,
        })
    }

    /// `Delta r_{i,outer} = r_{i+1} - r_i`, `0 <= i <= N-1` (F5).
    pub fn dr_outer(&self, i: usize) -> f64 {
        self.r[i + 1] - self.r[i]
    }

    /// `Delta r_{i,inner} = r_i - r_{i-1}`, `1 <= i <= N` (F5).
    pub fn dr_inner(&self, i: usize) -> f64 {
        self.r[i] - self.r[i - 1]
    }

    /// Class of node `i` (Document 2 Section 3).
    pub fn node_class(&self, i: usize) -> NodeClass {
        if i == 0 {
            NodeClass::Centre
        } else if i == self.n {
            NodeClass::Outer
        } else if let Some(k) = self.interface_nodes.iter().position(|&x| x == i) {
            NodeClass::Interface {
                inner: Region::ALL[k],
                outer: Region::ALL[k + 1],
            }
        } else {
            NodeClass::Interior(self.region_of_interval[i])
        }
    }

    /// `D_{i,outer}` for `0 <= i <= N-1` (F4); `D_{i,inner} = D_{i-1,outer}`.
    pub fn d_of_interval(&self, d: &[f64; 5]) -> Vec<f64> {
        self.region_of_interval
            .iter()
            .map(|region| d[region.index()])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RADII: [f64; 5] = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4];

    #[test]
    fn interface_nodes_follow_f1() {
        for counts in [[1, 1, 1, 1, 1], [3, 2, 4, 5, 2], [7, 3, 11, 5, 13]] {
            let g = Grid::from_radii(RADII, counts).unwrap();
            let c = counts;
            let expect = [
                c[0],
                c[0] + c[1],
                c[0] + c[1] + c[2],
                c[0] + c[1] + c[2] + c[3],
            ];
            assert_eq!(g.interface_nodes, expect);
            assert_eq!(g.n, c.iter().sum::<usize>());
            assert_eq!(g.r.len(), g.n + 1);
        }
    }

    #[test]
    fn spacing_per_material_f2() {
        let g = Grid::from_radii(RADII, [4, 5, 2, 8, 10]).unwrap();
        let expect = [
            2.0e-4 / 4.0,
            (3.0e-4 - 2.0e-4) / 5.0,
            (3.5e-4 - 3.0e-4) / 2.0,
            (4.0e-4 - 3.5e-4) / 8.0,
            (4.5e-4 - 4.0e-4) / 10.0,
        ];
        assert_eq!(g.dr_region, expect);
    }

    #[test]
    fn interface_radii_exact_f3() {
        let g = Grid::from_radii(RADII, [37, 41, 23, 29, 31]).unwrap();
        assert_eq!(g.r[0].to_bits(), 0.0f64.to_bits());
        for k in 0..4 {
            assert_eq!(g.r[g.interface_nodes[k]].to_bits(), RADII[k].to_bits());
        }
        assert_eq!(g.r[g.n].to_bits(), RADII[4].to_bits());
        assert!(g.r.windows(2).all(|w| w[1] > w[0]));
        let fine = Grid::from_radii(RADII, [320; 5]).unwrap();
        assert!(fine.r.windows(2).all(|w| w[1] > w[0]));
    }

    #[test]
    fn material_of_interval_f4() {
        let g = Grid::from_radii(RADII, [3, 2, 4, 5, 2]).unwrap();
        let d = [1.0, 2.0, 3.0, 4.0, 5.0];
        let di = g.d_of_interval(&d);
        assert_eq!(di.len(), g.n);
        for i in 0..g.n {
            let expect = if i < 3 {
                0
            } else if i < 5 {
                1
            } else if i < 9 {
                2
            } else if i < 14 {
                3
            } else {
                4
            };
            assert_eq!(g.region_of_interval[i], Region::ALL[expect]);
            assert_eq!(di[i], d[expect]);
        }
        for (k, &node) in g.interface_nodes.iter().enumerate() {
            assert_eq!(di[node - 1], d[k]);
            assert_eq!(di[node], d[k + 1]);
            assert_ne!(di[node - 1], di[node]);
        }
    }

    #[test]
    fn spacings_are_differences_f5() {
        let g = Grid::from_radii(RADII, [4, 5, 2, 8, 10]).unwrap();
        for i in 1..g.n {
            assert_eq!(g.dr_inner(i), g.r[i] - g.r[i - 1]);
            assert_eq!(g.dr_outer(i), g.r[i + 1] - g.r[i]);
        }
        let k = g.interface_nodes[0];
        assert!(g.dr_inner(k) != g.dr_outer(k));
        assert!((g.dr_inner(2) - g.dr_region[0]).abs() < 1e-12 * g.dr_region[0]);
    }

    #[test]
    fn storage_weight_f7() {
        let g = Grid::from_radii(RADII, [3, 2, 4, 5, 2]).unwrap();
        assert_eq!(g.v[0].to_bits(), 0.0f64.to_bits());
        // Interior node 1 of the kernel: r_1 = R_1/3, equal spacings R_1/3.
        let r1 = RADII[0] / 3.0;
        let expect = r1 * r1 * (RADII[0] / 3.0);
        assert!((g.v[1] - expect).abs() <= 8.0 * f64::EPSILON * expect);
        // Interface node: weight is the sum of half of each adjacent interval.
        let k = g.interface_nodes[0];
        let half_in = g.r[k] * g.r[k] * g.dr_inner(k) / 2.0;
        let half_out = g.r[k] * g.r[k] * g.dr_outer(k) / 2.0;
        assert!((g.v[k] - (half_in + half_out)).abs() <= 8.0 * f64::EPSILON * g.v[k]);
    }

    #[test]
    fn node_classes() {
        let g = Grid::from_radii(RADII, [3, 2, 4, 5, 2]).unwrap();
        assert_eq!(g.node_class(0), NodeClass::Centre);
        assert_eq!(g.node_class(1), NodeClass::Interior(Region::Kernel));
        assert_eq!(
            g.node_class(3),
            NodeClass::Interface {
                inner: Region::Kernel,
                outer: Region::Buffer
            }
        );
        assert_eq!(g.node_class(g.n), NodeClass::Outer);
        assert_eq!(g.node_class(g.n - 1), NodeClass::Interior(Region::Opyc));
    }

    #[test]
    fn no_duplicate_interface_unknown_p8() {
        let g = Grid::from_radii(RADII, [3, 2, 4, 5, 2]).unwrap();
        let mut nodes = g.interface_nodes.to_vec();
        nodes.dedup();
        assert_eq!(nodes.len(), 4);
        assert_eq!(g.r.len(), g.n + 1);
    }

    #[test]
    fn rejects_bad_grids() {
        assert!(Grid::from_radii(RADII, [0, 1, 1, 1, 1]).is_err());
        let mut bad = RADII;
        bad[2] = bad[1];
        assert!(Grid::from_radii(bad, [1; 5]).is_err());
        bad = RADII;
        bad[0] = f64::NAN;
        assert!(Grid::from_radii(bad, [1; 5]).is_err());
    }
}
