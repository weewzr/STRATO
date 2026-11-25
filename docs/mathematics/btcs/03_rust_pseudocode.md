# STRATO: Rust-Oriented Pseudocode for the Five-Layer BTCS Solver

**Status.** This is Document 3 of directive Section 9. It turns the final formulation ([Document 2](02_final_formulation.md), cited as "D2", equations (F1) to (F31) and properties P1 to P18) into routines that can be written in Rust without further mathematical decisions. Every routine names its inputs and outputs with units, its proposed Rust type and module, and the equations of D2 it implements. The derivations behind the equations are in [Document 1](01_from_ftcs_to_five_layer_btcs.md) ("D1"). The table that maps each equation to a planned Rust item and to the test that will check it is [Document 4](04_equation_to_code_traceability.md).

This document contains no solver results. It is pseudocode: it has not been compiled and no number in it comes from a run. Nothing here is experimental validation.

**What the pseudocode does not contain.** No finite-volume concept (no cell, no control volume, no face, no cell average) is used. Every unknown is a nodal value $C_i^j$ at a grid node $r_i$. The words "storage weight" and "interval conductance" are the names D2 gives to $V_i$ and $K_{i,\mathrm{outer}}$; they are coefficients of the nodal rows (F10), not cells of a mesh.

---

## 1. Conventions

### 1.1 Notation in the pseudocode

The binding notation of directive Section 5 applies to every equation in this document: $i$ is the global radial node index, $j$ the time-step index, $C_i^j$ the concentration at node $i$ at step $j$. A time superscript is an index, not a power. A power encloses the whole variable in parentheses.

In code, an array element replaces the superscript, and the loop counter that runs over time steps is called `step`, never `j` alone and never `i`, so that the spatial and the time index cannot be confused:

| Mathematics (D2) | Rust name | Unit |
|---|---|---|
| $i$ (node index, $0 \le i \le N$) | `i: usize` | dimensionless |
| $j$ (time-step index) | `step: usize` | dimensionless |
| $C_i^j$ | `c_old[i]` | $\mathrm{mol\ m^{-3}}$ |
| $C_i^{j+1}$ | `c_new[i]` | $\mathrm{mol\ m^{-3}}$ |
| $N$ | `n: usize` (index of the outer node; `n + 1` nodes) | dimensionless |
| $N_{\mathrm{kernel}}, N_{\mathrm{buffer}}, N_{\mathrm{IPyC}}, N_{\mathrm{SiC}}$ | `grid.interface_nodes[0..4]` | dimensionless |
| $r_i$ | `grid.r[i]` | $\mathrm{m}$ |
| $\Delta r_{\mathrm{material}}$ | `grid.dr_region[region.index()]` | $\mathrm{m}$ |
| $\Delta r_{i,\mathrm{outer}}$, $\Delta r_{i,\mathrm{inner}}$ | `grid.dr_outer(i)`, `grid.dr_inner(i)` (differences of `grid.r`) | $\mathrm{m}$ |
| $D_{\mathrm{material}}$ | `d[region.index()]` | $\mathrm{m^2\ s^{-1}}$ |
| $D_{i,\mathrm{outer}}$, $D_{i,\mathrm{inner}}$ | `d_of_interval[i]`, `d_of_interval[i-1]` | $\mathrm{m^2\ s^{-1}}$ |
| $\mathrm{Fo}_{\mathrm{material}}$ | `fo[region.index()]` | dimensionless |
| $K_{i,\mathrm{outer}}$ | `coeff.k_outer[i]` ($K_{i,\mathrm{inner}}$ is `coeff.k_outer[i-1]`) | $\mathrm{m^3\ s^{-1}}$ |
| $V_i$ | `grid.v[i]` | $\mathrm{m^3}$ |
| $L_i, B_i, U_i$ | `a.lower[i]`, `a.diag[i]`, `a.upper[i]` | dimensionless |
| $b_i^j$ | `rhs[i]` | $\mathrm{mol\ m^{-3}}$ |
| $\theta$ | `theta` | dimensionless |
| $\Delta t$ | `dt` | $\mathrm{s}$ |
| $M^j$, $M_{\mathrm{rel}}^j$, $M^0$ | `inventory`, `released`, `initial_inventory` | $\mathrm{mol}$ |
| $\Phi^j$ | `release_rate` | $\mathrm{mol\ s^{-1}}$ |
| $F^j$, $\varepsilon^j$ | `release_fraction`, `balance_residual` | dimensionless, $\mathrm{mol}$ |

The five materials are always indexed by the enum `Region` (below), never by $i$ or $j$. The symbol $C_{\mathrm{init}}$ of D2 is `initial.kernel_concentration_mol_m3` in the existing configuration; it is not called `c0`, because `c_new[0]` is the centre value $C_0^{j+1}$.

All quantities are in SI base units (metre, second, mole), all real numbers are `f64`, all counts are `usize`. No unit conversion happens inside the solver.

### 1.2 Arithmetic rules that the code must follow

These rules keep the code identical to the equations and make native and WebAssembly results comparable.

1. A square or cube of a variable is written as a product (`dr * dr`, `r * r`), not with `powi` or `powf`, so that no library routine with a platform-dependent last digit is called.
2. $L_i$ and $U_i$ are stored with their signs ($L_i \le 0$, $U_i < 0$, D2 Section 6.3). No absolute value is stored.
3. A coefficient that is exactly zero in D2 is assigned the literal `0.0`, never a computed product that happens to be zero: $K_{0,\mathrm{outer}}$ (after F6), $V_0$ (F7) and $L_1$ (F13).
4. $K_{N-1,\mathrm{outer}}$ is computed once, stored in `coeff.k_outer[n-1]`, and read from there by the matrix assembly (row $N-1$) and by the release (F26). It is never recomputed (D2 Section 7.2, normative rule).
5. The interface and boundary radii are assigned from the input, not accumulated by addition (F3).
6. No routine returns or stores a non-finite number. A routine that would produce one returns `Err`.
7. Rust `anyhow::Result` is used for errors, as in the existing crate. No new dependency is added.

### 1.3 Types and modules

The proposed layout adapts directive Section 10 to the existing crate. Files marked EXISTING are already in the repository and are extended or kept; files marked NEW do not exist yet. The old finite-volume module stays until the retirement step of the migration (it is deleted only after the BTCS solver has passed its verification); the pseudocode does not use it.

```text
src/
  config.rs                     EXISTING  Config, RegionConfig, validate   (extended, PS-01)
  analytical/sturm_liouville.rs EXISTING  independent reference, unchanged
  numerical/
    mod.rs                      EXISTING  gains the pub mod lines below
    grid.rs                     NEW  Region, NodeClass, Grid                       (PS-02, PS-03)
    coefficients.rs             NEW  Coefficients: fo, k_outer, v-checks          (PS-04)
    btcs.rs                     NEW  Assembly, assemble, Stepper, run             (PS-05, PS-11, PS-16)
    boundary_conditions.rs      NEW  centre_row, outer value, release_rate        (PS-06, PS-08, PS-13)
    interfaces.rs               NEW  interface_row_explicit, interior_row_explicit,
                                     interface_flux_both_sides                    (PS-07, PS-15)
    tridiagonal.rs              NEW  Tridiagonal, ThomasFactors                   (PS-10)
    inventory.rs                NEW  theta, initial_vector, inventory, Accumulator (PS-09, PS-12, PS-13, PS-14)
    diagnostics.rs              NEW  StepDiagnostics, TimeSeries, checks          (PS-17)
  physics/                      NEW  (Phase G, later; hooks only, PS-19)
    diffusion.rs                     DiffusivitySet (constant now, D(T) later)
    generation.rs, decay.rs, temperature.rs
  postprocessing/
    export.rs                   NEW  CSV export of series, profile, metadata      (PS-18)
    comparison.rs, plots.rs     EXISTING, re-pointed to BTCS
  validation/                   NEW  (experimental comparison; outside this document)
```

**Core types.**

```rust
/// The five materials, in radial order. Index 0..=4 is the position in every
/// per-material array. This enum is the only way a material is identified.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region { Kernel, Buffer, Ipyc, Sic, Opyc }
impl Region { pub fn index(self) -> usize; pub const ALL: [Region; 5]; }

/// What a node is, by position (D2 Section 3, "Node classes").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeClass {
    Centre,                                  // i = 0
    Interior(Region),                        // strictly inside one material
    Interface { inner: Region, outer: Region }, // i = N_kernel .. N_SiC
    Outer,                                   // i = N
}

pub struct Grid {
    pub n: usize,                    // N, index of the outer node
    pub r: Vec<f64>,                 // r_i, length N+1, [m]
    pub interface_nodes: [usize; 4], // N_kernel, N_buffer, N_IPyC, N_SiC
    pub region_of_interval: Vec<Region>, // length N, material of (r_i, r_{i+1})
    pub dr_region: [f64; 5],         // uniform spacing inside each material [m]
    pub v: Vec<f64>,                 // V_i, length N+1, v[0] = 0, v[N] unused (0.0) [m^3]
}

pub struct Coefficients {
    pub d: [f64; 5],                 // D_material [m^2/s]
    pub fo: [f64; 5],                // Fo_material (F8)
    pub k_outer: Vec<f64>,           // K_{i,outer}, length N, k_outer[0] = 0.0 [m^3/s]
}

pub struct Tridiagonal {             // order n_unknowns = N
    pub lower: Vec<f64>,             // lower[i] = L_i, lower[0] unused = 0.0
    pub diag:  Vec<f64>,             // B_i
    pub upper: Vec<f64>,             // U_i, upper[N-1] unused = 0.0
}

pub struct ThomasFactors { pub w: Vec<f64>, pub m: Vec<f64>, pub upper: Vec<f64> }

pub struct Assembly { pub matrix: Tridiagonal, pub coeff: Coefficients, pub dt: f64 }

pub struct StepDiagnostics {         // one output row
    pub inventory: f64,              // M^j            [mol]
    pub cumulative_release: f64,     // M_rel^j        [mol]
    pub release_fraction: f64,       // F^j
    pub release_rate: Option<f64>,   // Phi^j [mol/s]; None at j = 0 (D2 Section 7.2)
    pub balance_residual: f64,       // eps^j          [mol]
    pub min_concentration: f64,      // [mol/m^3]
    pub max_concentration: f64,      // [mol/m^3]
    // later, Phase G: generated, decayed [mol]
}
pub type TimeSeries = Vec<(f64, StepDiagnostics)>; // (t^j [s], row)
```

`StepDiagnostics` and `TimeSeries` already exist in the finite-volume module with six fields; they move to `diagnostics.rs`. The type of `release_rate` changes to `Option<f64>` because $\Phi^0$ is not defined (D2 Section 7.2); this is a deliberate change of the wrapper contract (see Section 4, item 4).

### 1.4 Order of the routines in one run

```text
PS-01 validate_input      -> ValidatedInput
PS-02 Grid::build         -> Grid                 (uses PS-03 inside)
PS-04 Coefficients::new   -> Coefficients
PS-05 assemble            -> Assembly             (uses PS-06, PS-07 checks, PS-08)
PS-10 ThomasFactors::new  -> factors
PS-09 initial_vector      -> c_old at j = 0, M^0
PS-16 run: for step in 1..=steps { PS-11 step; PS-12 inventory; PS-13 release; PS-14 residual;
                                    PS-17 diagnostics; PS-18 export rows }
```

---

## 2. Routines

Each routine has: purpose; inputs and outputs with units; type and module; equations implemented; pseudocode; notes. "Fails with" lists the `Err` cases. Section numbers in "D2 Section" refer to Document 2.

### PS-01 `validate_input` : input validation

**Purpose.** Reject every input for which the hypotheses (H) of D2 Section 8 do not hold, before any array is allocated.

**Inputs.**

| Name | Meaning | Unit |
|---|---|---|
| `cfg.regions[k].outer_radius_m` | $R_1, \dots, R_5$ | $\mathrm{m}$ |
| `cfg.regions[k].diffusivity_m2_s` | $D_{\mathrm{kernel}}, \dots, D_{\mathrm{OPyC}}$ | $\mathrm{m^2\ s^{-1}}$ |
| `cfg.regions[k].intervals` | $n_{\mathrm{kernel}}, \dots, n_{\mathrm{OPyC}}$ | dimensionless |
| `cfg.initial.kernel_concentration_mol_m3` | $C_{\mathrm{init}}$ | $\mathrm{mol\ m^{-3}}$ |
| `cfg.time.dt_s`, `cfg.time.end_time_s` | requested $\Delta t$ and end time | $\mathrm{s}$ |
| `cfg.time.output_every_steps` | output stride | dimensionless |
| `limits: Limits` | caller-supplied maxima on $N$ and on the number of steps | dimensionless |

**Outputs.** `ValidatedInput { radii: [f64; 5], d: [f64; 5], n_intervals: [usize; 5], c_init: f64, dt: f64, steps: usize, output_every: usize }` with $\Delta t$ in $\mathrm{s}$, or `Err`.

**Module and type.** `config.rs`, `impl Config { pub fn validate(&self) -> Result<()> }` (existing; extended) and `numerical/btcs.rs`, `pub fn validate_input(cfg: &Config, limits: &Limits) -> Result<ValidatedInput>`.

**Implements.** The conditions of the input table of D2 Section 2 and the hypotheses (H) of D2 Section 8; the step count rule is an implementation rule (review point RP-2).

**Existing configuration.** `RegionConfig.cells` of the existing configuration is reinterpreted as the number of grid *intervals* in that material, $n_{\mathrm{material}}$. The nodes are then the $N+1$ points of (F1) to (F3). The field is renamed `intervals` with `#[serde(alias = "cells")]` so that existing input files still load; the meaning is documented on the field. Exactly five regions in the fixed order kernel, buffer, IPyC, SiC, OPyC are required (as now).

```rust
fn validate_input(cfg, limits) -> Result<ValidatedInput> {
    require cfg.regions.len() == 5
    require species is not empty                       // existing rule
    require cfg.temperature_k finite and > 0           // existing rule; used by G3 only
    let mut previous = 0.0
    for k in 0..5 {
        let r = cfg.regions[k]
        require r.outer_radius_m finite and > previous     // 0 < R1 < R2 < ... < R5
        require r.diffusivity_m2_s finite and > 0.0         // each D > 0, finite
        require r.intervals >= 1                            // each n_material >= 1
        previous = r.outer_radius_m
    }
    let n = sum of r.intervals                               // N, D2 Section 2
    require n <= limits.max_intervals_total
    require cfg.initial.kernel_concentration_mol_m3 finite and > 0.0   // see RP-1
    require cfg.time.dt_s finite and > 0.0
    require cfg.time.end_time_s finite and > 0.0
    require cfg.time.output_every_steps >= 1
    // step count: the rule of the existing code, kept
    let steps_f = (end_time_s / dt_s).ceil()
    require steps_f finite and 1.0 <= steps_f <= limits.max_steps as f64
    let steps = steps_f as usize                             // J_steps >= 1
    let dt = end_time_s / steps as f64                       // effective Delta t [s]
    return ValidatedInput { ... }
}
```

**Fails with.** Any condition above; the message names the region and the quantity (for example "region IPyC: outer_radius_m must exceed the previous radius").

**Notes.** `limits` are arguments, not constants in this document: the numbers that guard the browser run are measured for the BTCS solver later and are not invented here. The effective step `dt = end / steps` equals the requested step when the end time is an integer multiple of it (the usual case), and is slightly smaller otherwise; the effective value is the one reported in the output metadata (PS-18). A $\Delta t$ that changes during a run is not supported here (D2 Section 5.8 describes the reassembly that a future time-dependent $D$ needs).

### PS-02 `Grid::build` : grid generation

**Purpose.** Build the nodes (F1 to F3), the spacing per material (F2), the interface node indices (F1) and the storage weights (F7).

**Inputs.** `ValidatedInput` (`radii`, `n_intervals`).

**Outputs.** `Grid` (Section 1.3): `n`, `r[0..=n]` in $\mathrm{m}$, `interface_nodes`, `region_of_interval`, `dr_region` in $\mathrm{m}$, `v[0..=n]` in $\mathrm{m^3}$.

**Module and type.** `numerical/grid.rs`, `impl Grid { pub fn build(input: &ValidatedInput) -> Result<Grid>; pub fn node_class(&self, i: usize) -> NodeClass; pub fn dr_outer(&self, i: usize) -> f64; pub fn dr_inner(&self, i: usize) -> f64; }`.

**Implements.** (F1), (F2), (F3), (F5), (F7); the material of each interval is assigned in PS-03.

```rust
fn build(input) -> Result<Grid> {
    // F1: node indices of the four interfaces and of the outer node
    let mut nodes_at = [0usize; 5];          // nodes_at[k] = index of the node at R_{k+1}
    let mut acc = 0
    for k in 0..5 { acc += input.n_intervals[k]; nodes_at[k] = acc }
    let n = acc                              // N = nodes_at[4]
    let interface_nodes = [nodes_at[0], nodes_at[1], nodes_at[2], nodes_at[3]]

    // F2: spacing per material
    let mut inner_radius = 0.0               // R_0 = 0
    let mut dr_region = [0.0; 5]
    for k in 0..5 {
        dr_region[k] = (input.radii[k] - inner_radius) / input.n_intervals[k] as f64
        inner_radius = input.radii[k]
    }

    // F3: node radii; interface and boundary radii are assigned exactly
    let mut r = vec![0.0; n + 1]             // r[0] = 0.0 exactly
    let mut first = 0                        // N_prev
    let mut r_prev = 0.0                     // R_prev
    for k in 0..5 {
        let last = nodes_at[k]               // N_next
        for i in (first + 1)..last {         // interior nodes of material k
            r[i] = r_prev + (i - first) as f64 * dr_region[k]
        }
        r[last] = input.radii[k]             // exact assignment, never accumulated
        first = last; r_prev = input.radii[k]
    }
    require r strictly increasing                 // guards against rounding collapse
    require every r[i] finite

    // PS-03: region_of_interval
    // F7: storage weight, trapezoidal, V_0 = 0 literally
    let mut v = vec![0.0; n + 1]
    for i in 1..n {                          // 1 <= i <= N-1
        v[i] = r[i] * r[i] * ((r[i] - r[i-1]) + (r[i+1] - r[i])) / 2.0
        require v[i] finite and > 0.0
    }
    // v[0] = 0.0 and v[n] = 0.0 (unused) by construction
    return Grid { n, r, interface_nodes, region_of_interval, dr_region, v }
}

fn dr_outer(&self, i) -> f64 { self.r[i + 1] - self.r[i] }         // F5, 0 <= i <= N-1
fn dr_inner(&self, i) -> f64 { self.r[i] - self.r[i - 1] }         // F5, 1 <= i <= N
fn node_class(&self, i) -> NodeClass {
    if i == 0 { Centre }
    else if i == self.n { Outer }
    else if let Some(k) = self.interface_nodes.position(i) {
        Interface { inner: Region::ALL[k], outer: Region::ALL[k + 1] }
    } else { Interior(self.region_of_interval[i]) }     // interval (r_i, r_{i+1}) lies in node i's material
}
```

**Fails with.** `r` not strictly increasing in floating point (for example $n_{\mathrm{material}}$ so large that two radii coincide); a non-finite or non-positive weight.

**Notes.**

- `dr_outer(i)` and `dr_inner(i)` are differences of the stored radii (F5). Inside a material they equal `dr_region` up to rounding; the *stored differences* are used everywhere a spacing enters a coefficient, so that all coefficients see the same number.
- The weight $V_i$ includes both adjacent spacings, so at an interface node it is the sum of half of each material's interval (D1 Eq. (43)), without any special case.
- `node_class` for an interior node uses the material of the interval on its outer side; for a node strictly inside a material both sides are the same material.

### PS-03 `assign_materials` : material assignment

**Purpose.** Give every interval $(r_i, r_{i+1})$ its material and diffusivity (F4).

**Inputs.** `Grid.interface_nodes`, `n` ; `d: [f64; 5]` in $\mathrm{m^2\ s^{-1}}$.

**Outputs.** `grid.region_of_interval: Vec<Region>` (length $N$) and `d_of_interval: Vec<f64>` in $\mathrm{m^2\ s^{-1}}$ (length $N$).

**Module and type.** `numerical/grid.rs`, `Grid::assign_regions(&mut self, nodes_at: &[usize; 5])` (called inside `build`) and `physics/diffusion.rs`, `DiffusivitySet::per_interval(&self, grid: &Grid) -> Vec<f64>`.

**Implements.** (F4).

```rust
fn assign_regions(&mut self, nodes_at) {
    self.region_of_interval = vec![Region::Kernel; self.n]
    let mut i = 0
    for k in 0..5 {
        while i < nodes_at[k] { self.region_of_interval[i] = Region::ALL[k]; i += 1 }   // interval i is (r_i, r_{i+1})
    }
}
// F4: D_{i,outer} for 0 <= i <= N-1, and D_{i,inner} = D_{i-1,outer}
fn per_interval(&self, grid) -> Vec<f64> {
    (0..grid.n).map(|i| self.d[grid.region_of_interval[i].index()]).collect()
}
```

**Notes.** The interface node $i = N_{\mathrm{kernel}}$ has $D_{i,\mathrm{inner}} = D_{\mathrm{kernel}}$ (interval $i-1$) and $D_{i,\mathrm{outer}} = D_{\mathrm{buffer}}$ (interval $i$), and similarly at the other three. No averaged diffusivity is ever formed (D2 Section 5.5). `DiffusivitySet` is where $D_{\mathrm{material}}(T)$ will be evaluated in Phase G3 (PS-19); for the baseline it holds the five constants of the input.

### PS-04 `Coefficients::new` : geometric coefficients

**Purpose.** Compute the interval conductances $K_{i,\mathrm{outer}}$ (F6) and the mesh Fourier numbers $\mathrm{Fo}_{\mathrm{material}}$ (F8).

**Inputs.** `grid: &Grid`; `d_of_interval` ($\mathrm{m^2\ s^{-1}}$); `d: [f64; 5]` ($\mathrm{m^2\ s^{-1}}$); `dt` ($\mathrm{s}$).

**Outputs.** `Coefficients { d, fo, k_outer }` with `fo` dimensionless and `k_outer[i]` in $\mathrm{m^3\ s^{-1}}$, $0 \le i \le N-1$.

**Module and type.** `numerical/coefficients.rs`, `impl Coefficients { pub fn new(grid: &Grid, d: [f64; 5], dt: f64) -> Result<Coefficients> }`.

**Implements.** (F6), (F8); and the unit-free form (F5) through `grid.dr_outer`.

```rust
fn new(grid, d, dt) -> Result<Coefficients> {
    // F8
    let mut fo = [0.0; 5]
    for k in 0..5 { fo[k] = d[k] * dt / (grid.dr_region[k] * grid.dr_region[k]) }
    // F6
    let mut k_outer = vec![0.0; grid.n]
    k_outer[0] = 0.0                                   // exact zero, because r_0 = 0 (rule 3 of Section 1.2)
    for i in 1..grid.n {
        let dr = grid.r[i + 1] - grid.r[i]             // dr_outer(i), F5
        k_outer[i] = d[grid.region_of_interval[i].index()] * grid.r[i] * grid.r[i + 1] / dr
        require k_outer[i] finite and > 0.0
    }
    return Coefficients { d, fo, k_outer }
}
```

**Notes.** `K_{i,inner}` is not stored: it is `k_outer[i - 1]` for $1 \le i \le N$ (F6). Row $N-1$ and the release use `k_outer[n - 1]`, the same stored number (rule 4 of Section 1.2). `fo` is used only by the centre row (PS-06) and by the explicit check forms (PS-07); the assembly rows themselves use `k_outer` and `grid.v` (F11).

### PS-05 `assemble` : matrix assembly

**Purpose.** Build the tridiagonal matrix $\mathbf{A}$ of (F19) from the rows (F11), (F12) and (F17).

**Inputs.** `grid: &Grid`; `coeff: &Coefficients`; `dt` ($\mathrm{s}$).

**Outputs.** `Assembly { matrix: Tridiagonal, coeff, dt }`; the matrix has order $N$ with `lower`, `diag`, `upper` dimensionless.

**Module and type.** `numerical/btcs.rs`, `pub fn assemble(grid: &Grid, coeff: Coefficients, dt: f64) -> Result<Assembly>`.

**Implements.** (F9), (F10), (F11), (F12), (F13), (F17), (F19). The rows for the centre, the interfaces and the outer boundary are PS-06, PS-07 and PS-08.

```rust
fn assemble(grid, coeff, dt) -> Result<Assembly> {
    let n = grid.n
    let mut lower = vec![0.0; n]; let mut diag = vec![0.0; n]; let mut upper = vec![0.0; n]

    // Row 0: centre, F12 (PS-06)
    let (b0, u0) = centre_row(coeff.fo[Region::Kernel.index()])
    diag[0] = b0; upper[0] = u0                        // lower[0] is absent

    // Rows 1 .. N-1: the one general row, F11, for every class of node (interior, interface, row N-1)
    for i in 1..n {                                    // 1 <= i <= N-1
        let k_in  = coeff.k_outer[i - 1]               // K_{i,inner}
        let k_out = coeff.k_outer[i]                   // K_{i,outer}
        let l = -dt * k_in  / grid.v[i]                // L_i  (F11)
        let u = -dt * k_out / grid.v[i]                // U_i  (F11)
        lower[i] = l
        diag[i]  = 1.0 - l - u                         // B_i = 1 + |L_i| + |U_i|  (F11)
        upper[i] = u
    }
    lower[1] = 0.0                                     // F13: exact zero (K_{1,inner} = K_{0,outer} = 0.0)
    // Row N-1: F17. The boundary value C_N = 0 (F18) removes the matrix entry, not the diagonal.
    upper[n - 1] = 0.0                                 // U_{N-1} stays inside diag[n-1], not a matrix entry

    for every entry: require finite
    return Assembly { matrix: Tridiagonal { lower, diag, upper }, coeff, dt }
}
```

**Notes.**

- **One row formula, not four.** Rows $1$ to $N-1$ are computed by the same two lines whatever the class of node $i$ (D2 Section 5.1, normative). Whether node $i$ is interior to a material, an interface node, or the node next to the outer boundary is decided only by which `k_outer` entries and which `grid.v[i]` enter, and those already carry the material and spacing information. The explicit forms (F14), (F15), (F16a to d) are the same numbers written by material; they are *checks* (PS-07) and are never used to fill the matrix.
- `upper[n - 1]` is set to zero *after* `diag[n - 1]` has been computed from the full $U_{N-1}$ of (F11). Dropping `u` before computing `diag` would be wrong (D2 Section 5.6).
- `lower[1] = 0.0` re-asserts (F13); with `k_outer[0] = 0.0` the product is $-\Delta t \cdot 0 / V_1 = -0.0$, which is replaced by the literal `0.0`.
- Each time $\Delta t$ or a diffusivity changes, `Coefficients::new` and `assemble` are called again (D2 Section 5.8). For the baseline they run once per simulation.
- The matrix is stored by three vectors of length $N$ (`lower[0]` and `upper[N-1]` unused), not as a dense matrix.

### PS-06 `centre_row` : centre-node treatment

**Purpose.** Return the coefficients of row 0.

**Inputs.** `fo_kernel` = $\mathrm{Fo}_{\mathrm{kernel}}$ (dimensionless).

**Outputs.** `(B_0, U_0)` dimensionless.

**Module and type.** `numerical/boundary_conditions.rs`, `pub fn centre_row(fo_kernel: f64) -> (f64, f64)`.

**Implements.** (F12). The row follows from the limit $3 D_{\mathrm{kernel}}\,\partial^2 C/\partial r^2$ at $r = 0$ and the symmetric ghost node $C_{-1} = C_1$ (D1 Eqs. (23) to (25)).

```rust
fn centre_row(fo_kernel) -> (f64, f64) {
    let six_fo = 6.0 * fo_kernel
    (1.0 + six_fo, -six_fo)           // B_0, U_0     right-hand side: b_0 = C_0^j (F20)
}
```

**Notes.**

- Row 0 is not a flux balance and carries no inventory: $V_0 = 0$ (F7). The centre value $C_0^{j+1}$ is reported but never enters the inventory (F25) or the release (F26); it is driven by node 1 and drives nothing (P7, F13).
- No centre weight such as a third of a half-cell cubed is used anywhere (D2 Section 4, after F7).
- If $n_{\mathrm{kernel}} = 1$, node 1 is the first interface node; row 0 is unchanged and `lower[1] = 0.0` still holds.

### PS-07 `interface_row_explicit` and `interior_row_explicit` : interface-node and interior-node check forms

**Purpose.** Evaluate the closed forms (F14) and (F16) so that the single formula (F11) of PS-05 can be checked against them in tests and, optionally, in debug builds. This is how the four interface nodes are treated: they are rows of (F11) with $D_{i,\mathrm{inner}} \ne D_{i,\mathrm{outer}}$ and unequal spacings, and PS-07 is their independent expression.

**Inputs.** For an interior node: `fo` ($\mathrm{Fo}_{\mathrm{material}}$), `dr` ($\Delta r_{\mathrm{material}}$, $\mathrm{m}$), `r_i` ($\mathrm{m}$). For an interface node: `fo_in`, `fo_out` (dimensionless), `dr_in`, `dr_out` ($\mathrm{m}$), `radius` = $R$ ($\mathrm{m}$).

**Outputs.** `(L_i, B_i, U_i)` dimensionless.

**Module and type.** `numerical/interfaces.rs`, `pub fn interior_row_explicit(fo: f64, dr: f64, r_i: f64) -> (f64, f64, f64)` and `pub fn interface_row_explicit(fo_in: f64, fo_out: f64, dr_in: f64, dr_out: f64, radius: f64) -> (f64, f64, f64)`.

**Implements.** (F14), (F15) (kernel special case), (F16), (F16a) to (F16d).

```rust
fn interior_row_explicit(fo, dr, r_i) -> (f64, f64, f64) {
    let l = -fo * (1.0 - dr / r_i)                 // F14
    let u = -fo * (1.0 + dr / r_i)
    (l, 1.0 + 2.0 * fo, u)
}
// kernel: dr / r_i = 1 / i  (F15), valid only for i in 1..=N_kernel-1 and only in the kernel

fn interface_row_explicit(fo_in, fo_out, dr_in, dr_out, radius) -> (f64, f64, f64) {
    let s = dr_in + dr_out
    let l = -fo_in  * (2.0 * dr_in  / s) * (1.0 - dr_in  / radius)    // F16
    let u = -fo_out * (2.0 * dr_out / s) * (1.0 + dr_out / radius)
    (l, 1.0 - l - u, u)
}
```

**Interface at each of the four nodes** (D2 Section 5.5 table):

| Interface node | $R$ | `fo_in`, `dr_in` | `fo_out`, `dr_out` | Equation |
|---|---|---|---|---|
| `interface_nodes[0]` ($N_{\mathrm{kernel}}$) | $R_1$ | kernel | buffer | (F16a) |
| `interface_nodes[1]` ($N_{\mathrm{buffer}}$) | $R_2$ | buffer | IPyC | (F16b) |
| `interface_nodes[2]` ($N_{\mathrm{IPyC}}$) | $R_3$ | IPyC | SiC | (F16c) |
| `interface_nodes[3]` ($N_{\mathrm{SiC}}$) | $R_4$ | SiC | OPyC | (F16d) |

**Interface conditions as they appear in the code.**

- *Continuity of concentration.* The interface node is one unknown, `c[i]`, shared by both materials. There is no second value to keep equal, so continuity is exact (P8). The code never stores an inner and an outer concentration at an interface.
- *Continuity of flux.* The flow from the inner side uses only `d_of_interval[i-1]` through `k_outer[i-1]`, the flow to the outer side uses only `d_of_interval[i]` through `k_outer[i]`, and both are balanced against the one storage weight `v[i]` (F10). No diffusivity is averaged and no constant-diffusivity stencil is applied across the jump (D2 Section 5.5).
- *Test hook.* In a test (and, behind `debug_assert!`, in a debug build) the row of PS-05 at each interface node is compared with `interface_row_explicit`; at every interior node of every material with `interior_row_explicit`. The comparison is a relative difference at the level of rounding: the multiple of the unit roundoff is fixed by the implementer from the operation count of the two forms and recorded in the test before it is run, not tuned afterwards (D2 Section 5.1, "assembly rule").

### PS-08 `outer_boundary` : outer boundary

**Purpose.** Apply the absorbing boundary $C_N^j = 0$ (F18) and the row next to it (F17).

**Inputs.** The solution vector `c_new` of length $N$ from PS-10; the matrix of PS-05.

**Outputs.** The full profile of length $N+1$ with `c[n] = 0.0`.

**Module and type.** `numerical/boundary_conditions.rs`, `pub fn outer_value() -> f64 { 0.0 }` and `pub fn extend_with_boundary(c_unknowns: &[f64]) -> Vec<f64>` (or an in-place write into a preallocated vector of length $N+1$).

**Implements.** (F17), (F18).

```rust
fn extend_with_boundary(c_unknowns: &[f64], full: &mut [f64]) {
    // full has length N + 1
    full[..n].copy_from_slice(c_unknowns)
    full[n] = 0.0                       // F18: node N has no row, no ghost node, and no stored unknown
}
```

**Notes.** The outer boundary appears in three places and nowhere else: (1) the absence of $U_{N-1}$ as a matrix entry with its presence in $B_{N-1}$ (PS-05); (2) the zero right-hand side contribution, because the boundary value is zero (PS-11 adds nothing to `rhs[n-1]`); (3) the release formula (PS-13), which is the flow through the last interval into the node where $C = 0$. The insulated row of the owner's original notes is not used.

### PS-09 `initial_vector` : initial condition

**Purpose.** Represent the discontinuous start (kernel at $C_{\mathrm{init}}$, coatings empty) on the nodes, with the inventory-matched kernel-surface value, and compute $M^0$.

**Inputs.** `grid: &Grid`; `c_init` ($\mathrm{mol\ m^{-3}}$); `n_kernel` ($= N_{\mathrm{kernel}}$).

**Outputs.** `c_old: Vec<f64>` of length $N+1$ ($\mathrm{mol\ m^{-3}}$) with `c_old[n] = 0.0`; `theta` (dimensionless); `initial_inventory` $M^0$ ($\mathrm{mol}$).

**Module and type.** `numerical/inventory.rs`, `pub fn kernel_surface_theta(dr_kernel: f64, dr_buffer: f64, n_kernel: usize) -> f64` and `pub fn initial_vector(grid: &Grid, c_init: f64) -> (Vec<f64>, f64, f64)`.

**Implements.** (F21), (F22), (F23) (as a check); $M^0$ through (F25).

```rust
fn kernel_surface_theta(dr_kernel, dr_buffer, n_kernel) -> f64 {
    dr_kernel / (dr_kernel + dr_buffer) * (1.0 - 1.0 / (3.0 * n_kernel as f64))      // F22
}

fn initial_vector(grid, c_init) -> (Vec<f64>, f64, f64) {
    let nk = grid.interface_nodes[0]                                // N_kernel
    let theta = kernel_surface_theta(grid.dr_region[0], grid.dr_region[1], nk)
    let mut c = vec![0.0; grid.n + 1]
    for i in 0..nk { c[i] = c_init }                                // F21: 0 <= i <= N_kernel - 1
    c[nk] = theta * c_init                                          // F21, F22
    // c[nk + 1 ..= n] stay 0.0
    let m0 = inventory(grid, &c)                                    // PS-12, F25
    require m0 finite and > 0.0
    (c, theta, m0)
}
```

**Notes.**

- $\theta \in (0, 1)$ always. The values $\tfrac{1}{2}$, $0$ and $1$ are not used (D2 Section 6.2).
- $M^0$ is computed by the discrete sum (F25), which for this start vector equals $\tfrac{4\pi}{3}\left(R_1\right)^3 C_{\mathrm{init}}$ (P10); the two values are compared in a test, not substituted for each other in the code. (F28) divides by the discrete $M^0$ (D2 Section 10).
- The check (F23): $\sum_{i=1}^{N_{\mathrm{kernel}}-1} V_i + \theta\,V_{N_{\mathrm{kernel}}} = \left(R_1\right)^3 / 3$ is evaluated in a test with the weights of PS-02.

### PS-10 `ThomasFactors` : tridiagonal solution

**Purpose.** Solve $\mathbf{A}\,\mathbf{x} = \mathbf{b}$ for the tridiagonal $\mathbf{A}$ of PS-05 by the algorithm (F24), in the form that D1 Section 9.2 (vi) proves non-negative.

**Inputs.** `Tridiagonal` (order $N$, dimensionless); a right-hand side `rhs` ($\mathrm{mol\ m^{-3}}$, length $N$).

**Outputs.** `x` ($\mathrm{mol\ m^{-3}}$, length $N$), or `Err`.

**Module and type.** `numerical/tridiagonal.rs`, `impl ThomasFactors { pub fn new(a: &Tridiagonal) -> Result<ThomasFactors>; pub fn solve(&self, rhs: &[f64], x: &mut [f64], work: &mut [f64]) }`.

**Implements.** (F24); properties P5 and P6.

The matrix does not change from step to step in the baseline (D2 Section 5.8), so the forward elimination of the matrix (pivots and multipliers) is done once in `new`, and each step only processes the right-hand side. The arithmetic is exactly that of (F24); the split into two functions changes no floating-point operation.

```rust
fn new(a: &Tridiagonal) -> Result<ThomasFactors> {
    let n = a.diag.len()
    let mut m = vec![0.0; n]; let mut w = vec![0.0; n]
    m[0] = a.diag[0]
    for i in 1..n {
        w[i] = a.lower[i] / m[i - 1]                       // w_i = L_i / m_{i-1}   (<= 0)
        m[i] = a.diag[i] - w[i] * a.upper[i - 1]           // m_i = B_i - w_i U_{i-1}
    }
    for i in 0..n { require m[i] finite and > 0.0 }        // the hypothesis of P6
    return ThomasFactors { w, m, upper: a.upper.clone() }
}

fn solve(&self, rhs, x, work) {                            // work = b'
    let n = self.m.len()
    work[0] = rhs[0]
    for i in 1..n { work[i] = rhs[i] - self.w[i] * work[i - 1] }        // b'_i = b_i - w_i b'_{i-1}
    x[n - 1] = work[n - 1] / self.m[n - 1]
    for i in (0..n - 1).rev() { x[i] = (work[i] - self.upper[i] * x[i + 1]) / self.m[i] }
}
```

**Fails with.** A non-finite or non-positive pivot `m[i]` (P5 shows that in exact arithmetic $m_i \ge 1 + |U_i| \ge 1$, so a failure here means the matrix was not assembled as D2 prescribes).

**Notes.**

- No pivoting. No division by `a.upper[i]` (the unstable "make $C_{i+1}$ the subject" marching of the owner's original notes is not used; D1 Section 9.3).
- Because $L_1 = 0$, `w[1] = 0.0` and the centre value is solved last, from node 1 (P7).
- Do not replace this routine by a textbook variant, by a subtraction-free elimination written for another matrix, or by a library solver without a new review: P6 (non-negativity in floating point) is proved only for the operations written above (D2 Section 8).
- The solver is generic in the matrix, and has no knowledge of the grid or the physics: `tridiagonal.rs` imports nothing from the other numerical modules.

### PS-11 `Stepper::step` : one time step

**Purpose.** Advance $C_i^j$ to $C_i^{j+1}$.

**Inputs.** `c_old` ($\mathrm{mol\ m^{-3}}$, length $N+1$, with `c_old[n] = 0.0`).

**Outputs.** `c_new` ($\mathrm{mol\ m^{-3}}$, length $N+1$, with `c_new[n] = 0.0`).

**Module and type.** `numerical/btcs.rs`, `pub struct Stepper { assembly: Assembly, factors: ThomasFactors, rhs: Vec<f64>, work: Vec<f64> }` with `pub fn new(grid: &Grid, d: [f64; 5], dt: f64) -> Result<Stepper>` and `pub fn step(&mut self, c_old: &[f64], c_new: &mut [f64])`.

**Implements.** (F9), (F20), (F24), (F18).

```rust
fn step(&mut self, c_old, c_new) {
    // F20: right-hand side b_i^j = C_i^j, 0 <= i <= N-1
    // (Phase G1 adds the source term here, PS-19; the boundary adds nothing, F18)
    self.rhs[..n].copy_from_slice(&c_old[..n])
    self.factors.solve(&self.rhs, &mut c_new[..n], &mut self.work)     // PS-10: C_0^{j+1} .. C_{N-1}^{j+1}
    c_new[n] = 0.0                                                    // F18 (PS-08)
}
```

**Notes.** One step costs a number of operations proportional to $N$ and allocates nothing. The step is a function of `c_old`, the stored matrix and nothing else, so it is deterministic and is the unit tested against a dense solve (Document 4). `Stepper::rebuild(&mut self, grid, d, dt)` runs PS-04, PS-05 and PS-10 again; it is the hook for $D(T)$ (PS-19).

### PS-12 `inventory` : inventory accounting

**Purpose.** Compute $M^j$.

**Inputs.** `grid: &Grid` (`v`, $\mathrm{m^3}$); `c: &[f64]` ($\mathrm{mol\ m^{-3}}$, length $N+1$).

**Outputs.** $M^j$ in $\mathrm{mol}$. Optionally the inventory per material (below).

**Module and type.** `numerical/inventory.rs`, `pub fn inventory(grid: &Grid, c: &[f64]) -> f64` and `pub fn inventory_by_region(grid: &Grid, c: &[f64]) -> [f64; 5]`.

**Implements.** (F25).

```rust
fn inventory(grid, c) -> f64 {
    let mut sum = 0.0
    for i in 1..grid.n { sum += grid.v[i] * c[i] }      // F25: 1 <= i <= N-1; V_0 = 0 and C_N = 0
    4.0 * PI * sum
}
```

**Notes.**

- The factor $4\pi$ is applied once, at the end. (F25) is the composite trapezoidal rule for $4\pi \int_0^{R_5} \left(r\right)^2 C\,dr$ on the nodes (D2 Section 7.1).
- `inventory_by_region` splits $V_i\,C_i$ between materials for the plots of the inventory in each layer. The weight of an interface node is split at the interface into its inner half-interval, $\left(R\right)^2\,\Delta r_{i,\mathrm{inner}}/2$, and its outer half-interval, $\left(R\right)^2\,\Delta r_{i,\mathrm{outer}}/2$ (the two terms of $V_i$ in (F7)). The five parts add up to $M^j$ to rounding; this sum rule is checked in a test. This routine is a presentation aid and is not used by the balance (F29).
- Summation is a plain loop in increasing $i$; the order is fixed so that native and WebAssembly agree (parity test, Document 4).

### PS-13 `release_rate` and `Accumulator` : release calculation

**Purpose.** Compute the release rate $\Phi^{j+1}$ (F26), the cumulative release $M_{\mathrm{rel}}^{j+1}$ (F27) and the release fraction $F^{j+1}$ (F28).

**Inputs.** `c_new[n - 1]` $= C_{N-1}^{j+1}$ ($\mathrm{mol\ m^{-3}}$); `coeff.k_outer[n - 1]` $= K_{N-1,\mathrm{outer}}$ ($\mathrm{m^3\ s^{-1}}$); `dt` ($\mathrm{s}$); `initial_inventory` $M^0$ ($\mathrm{mol}$).

**Outputs.** `release_rate` $\Phi^{j+1}$ ($\mathrm{mol\ s^{-1}}$); updated `released` $M_{\mathrm{rel}}^{j+1}$ ($\mathrm{mol}$); `release_fraction` $F^{j+1}$ (dimensionless).

**Module and type.** `numerical/boundary_conditions.rs`, `pub fn release_rate(k_last: f64, c_last: f64) -> f64`; `numerical/inventory.rs`, `pub struct Accumulator { sum: f64, compensation: f64 }` with `pub fn add(&mut self, x: f64)` and `pub fn value(&self) -> f64`, and `pub fn release_fraction(released: f64, initial_inventory: f64) -> f64`.

**Implements.** (F26), (F27), (F28).

```rust
fn release_rate(k_last, c_last) -> f64 { 4.0 * PI * k_last * c_last }       // F26

// in the run loop, after the step that produced c_new (level j + 1):
let phi = release_rate(coeff.k_outer[n - 1], c_new[n - 1])                  // the stored K, never recomputed
released.add(dt * phi)                                                      // F27, compensated
let f = released.value() / initial_inventory                               // F28

fn add(&mut self, x) {                      // compensated (Kahan) summation of the increments
    let y = x - self.compensation
    let t = self.sum + y
    self.compensation = (t - self.sum) - y
    self.sum = t
}
```

**Notes.**

- $\Phi^{j+1}$ uses the *new* level $C_{N-1}^{j+1}$ and the *same* $K_{N-1,\mathrm{outer}}$ that sits inside $B_{N-1}$. Any other formula (a one-sided gradient with a recomputed spacing, the old level $C_{N-1}^j$, or $\Phi^0$) breaks the balance (F29) (D2 Section 7.2). $\Phi^0$ is not computed; the output row at $j = 0$ has `release_rate: None`.
- The compensated summation is an implementation device for the sum in (F27); it changes the rounding of $M_{\mathrm{rel}}^j$ but not the identity. The finite-volume code used the same device; whether it should remain for BTCS is part of the rounding analysis that fixes the balance tolerance (RP-4).
- (F27) holds for the step length of the step that produced $\Phi^{k}$; with the constant effective `dt` of PS-01 this is `dt`. A variable step needs `dt_k` in place of `dt`.
- $F^j$ is the reported release fraction. The value $1 - M^j / M^0$ is not reported as a release; it is computed only inside PS-14 as a diagnostic.

### PS-14 `balance_residual` : conservation residual

**Purpose.** Report $\varepsilon^j$ at every output row.

**Inputs.** $M^j$, $M_{\mathrm{rel}}^j$, $M^0$ (all in $\mathrm{mol}$).

**Outputs.** `balance_residual` $\varepsilon^j$ in $\mathrm{mol}$, and `relative_balance_residual` $\varepsilon^j / M^0$ (dimensionless).

**Module and type.** `numerical/inventory.rs`, `pub fn balance_residual(inventory: f64, released: f64, initial_inventory: f64) -> f64`.

**Implements.** (F29) (the identity being tested), (F30) (the residual).

```rust
fn balance_residual(m_j, m_rel_j, m_0) -> f64 { m_j + m_rel_j - m_0 }       // F30
```

**Notes.**

- In exact arithmetic the residual is zero (P9); in floating point it is a rounding quantity (P18). **No tolerance is fixed in this document** (D2 Section 7.4). The tolerance that a test applies to $\left|\varepsilon^j\right| / M^0$ is derived from a rounding analysis of the Rust implementation (largest $\mathrm{Fo}$ and number of steps enter) and recorded before the first run (RP-4).
- The routine reports the residual; it never corrects the solution, the release or the inventory to make the residual vanish.
- Phase G extends the identity to $M^0 + M_{\mathrm{gen}}^j = M^j + M_{\mathrm{rel}}^j + M_{\mathrm{dec}}^j + \varepsilon^j$ (D2 Section 9) by adding two arguments. Until then they are absent, not zero-filled.

### PS-15 `interface_flux_both_sides` : interface and centre diagnostics

**Purpose.** Report, at an interface node, the outward flow computed from the inner side and from the outer side, and report the centre concentration.

**Inputs.** `grid`, `coeff`, `c_old`, `c_new` ($\mathrm{mol\ m^{-3}}$), `dt` ($\mathrm{s}$), the interface node index `i` $\in$ `grid.interface_nodes`.

**Outputs.** `(phi_from_inner, phi_from_outer)` in $\mathrm{mol\ s^{-1}}$ (the outward flow $\Phi_R^{j+1}$ of (F31), computed by each of its two expressions). Centre value: `c_new[0]` in $\mathrm{mol\ m^{-3}}$.

**Module and type.** `numerical/interfaces.rs`, `pub fn interface_flux_both_sides(grid: &Grid, coeff: &Coefficients, c_old: &[f64], c_new: &[f64], dt: f64, i: usize) -> (f64, f64)`.

**Implements.** (F31); property P8, P8a. The centre value follows from (F12).

```rust
fn interface_flux_both_sides(grid, coeff, c_old, c_new, dt, i) -> (f64, f64) {
    let radius = grid.r[i]
    let rate_of_change = (c_new[i] - c_old[i]) / dt
    let k_in  = coeff.k_outer[i - 1]
    let k_out = coeff.k_outer[i]
    let half_in  = radius * radius * grid.dr_inner(i) / 2.0     // inner half of V_i
    let half_out = radius * radius * grid.dr_outer(i) / 2.0     // outer half of V_i
    let from_inner = 4.0 * PI * (k_in * (c_new[i - 1] - c_new[i]) - half_in * rate_of_change)    // F31, first expression
    let from_outer = 4.0 * PI * (k_out * (c_new[i] - c_new[i + 1]) + half_out * rate_of_change)  // F31, second expression
    (from_inner, from_outer)
}
```

**Notes.** The two expressions are equal in exact arithmetic (P8). Their floating-point difference, scaled by the larger magnitude of the two, is a diagnostic and is skipped where both are below the rounding floor of the quantity that enters them. The routine checks the flux condition of the interface from *both* sides, as directive Section 8 A3 requires. D2 states that (F31) is consistent to second order (P8a) and that the second-order convergence of the computed value is observed only, not proved (P8b); the diagnostic does not claim more. The weights `half_in` and `half_out` are computed here from the same stored radii as `grid.v[i]` and add up to it; this is tested.

### PS-16 `run` : time stepping

**Purpose.** Run the whole simulation and collect the output rows.

**Inputs.** `cfg: &Config` (validated); `limits`.

**Outputs.** `RunResult { grid: Grid, series: TimeSeries, final_profile: Vec<f64>, snapshots: Vec<(f64, Vec<f64>)>, meta: RunMeta }`; times in $\mathrm{s}$, profiles in $\mathrm{mol\ m^{-3}}$ on the $N+1$ nodes; `RunMeta` carries the effective $\Delta t$, `steps`, `n`, $\theta$, $M^0$, the solver label.

**Module and type.** `numerical/btcs.rs`, `pub fn run(cfg: &Config, limits: &Limits, snapshot_times: &[f64]) -> Result<RunResult>`. The wrapper calls this function and contains no stepping loop of its own.

**Implements.** The time loop of (F9), (F20), (F24); the accounting (F25) to (F30).

```rust
fn run(cfg, limits, snapshot_times) -> Result<RunResult> {
    let input = validate_input(cfg, limits)?                              // PS-01
    let grid = Grid::build(&input)?                                       // PS-02, PS-03
    let mut stepper = Stepper::new(&grid, input.d, input.dt)?             // PS-04, PS-05, PS-10
    let (mut c_old, _theta, m0) = initial_vector(&grid, input.c_init)     // PS-09
    let mut c_new = vec![0.0; grid.n + 1]
    let mut released = Accumulator::new()
    let mut series = vec![ (0.0, row_at_start(&grid, &c_old, m0)) ]       // j = 0: release_rate = None, F = 0, residual = 0.0 by definition
    for step in 1..=input.steps {                                         // j = step - 1  ->  j + 1 = step
        stepper.step(&c_old, &mut c_new)                                  // PS-11
        let phi = release_rate(stepper.k_last(), c_new[grid.n - 1])       // PS-13, F26
        released.add(input.dt * phi)                                      // F27
        swap(&mut c_old, &mut c_new)                                      // c_old now holds C^{j+1}
        let t = step as f64 * input.dt
        if step % input.output_every == 0 || step == input.steps {
            let m = inventory(&grid, &c_old)                              // PS-12
            let row = StepDiagnostics {
                inventory: m,
                cumulative_release: released.value(),
                release_fraction: released.value() / m0,                  // F28
                release_rate: Some(phi),
                balance_residual: balance_residual(m, released.value(), m0),   // PS-14
                min_concentration: min(&c_old[..=grid.n]),
                max_concentration: max(&c_old[..=grid.n]),
            }
            require every field of row finite
            series.push((t, row))
        }
        keep a profile copy for every requested snapshot time that this step reaches (nearest output step, never interpolated)
    }
    return RunResult { ... , final_profile: c_old }
}
```

**Notes.**

- The row at $j = 0$ has the exact values $M^0$, $M_{\mathrm{rel}}^0 = 0$, $F^0 = 0$, $\varepsilon^0 = M^0 - M^0 = 0$, and no release rate (D2 Section 7.2, 7.3).
- `t = step * dt` is computed by one multiplication, not accumulated by repeated addition.
- Time is the only index of the outer loop; the inner loops of PS-05, PS-10, PS-12 run over $i$. The name `step` is $j+1$ after the step.
- The balance residual is evaluated from the stored `released` and `m` at every *output* row, but the accumulation of `released` runs at *every* step, so an output stride larger than one does not change the identity.
- A run that exceeds `limits` is refused in PS-01, never truncated.

### PS-17 `diagnostics` : verification diagnostics

**Purpose.** Check, during tests and optionally in a verification run, that the computed solution has the properties that D2 Section 8 proves or observes. These checks report; they never alter a result and never loosen a bound.

**Inputs.** Depending on the check: the matrix, the factors, `c_old`, `c_new`, the series.

**Outputs.** `DiagnosticsReport` with one named boolean and one named measured value per check, in the units of the checked quantity, or `Err` for a violated structural check.

**Module and type.** `numerical/diagnostics.rs`, `pub fn check_matrix_structure(a: &Tridiagonal) -> Result<MatrixReport>`, `pub fn check_step_properties(c_old: &[f64], c_new: &[f64]) -> StepReport`, `pub fn check_series(series: &TimeSeries) -> SeriesReport`.

**Implements.** Properties P1 to P10, P18; the quantities (F25), (F28), (F30), (F31).

| Check | Quantity and rule | Property | Exactness |
|---|---|---|---|
| Sign pattern | `lower[i] <= 0.0` for $i \ge 1$ with `lower[1] == 0.0`; `upper[i] < 0.0` for $0 \le i \le N-2$; `diag[i] > 0.0` | P1 | exact comparison on the assembled numbers |
| Row margin | $B_i - \lvert L_i \rvert - \lvert U_i \rvert$ over the matrix entries; it should equal 1 for interior rows, and be at least 1 for row $0$ and row $N-1$ | P1 | holds to rounding; report the smallest computed margin, do not assert equality |
| Diagonal of the last row | `diag[n-1]` equals $1 + \lvert L_{N-1} \rvert + \lvert U_{N-1} \rvert$ with $U_{N-1}$ recomputed from `coeff.k_outer[n-1]` and `grid.v[n-1]` | P1, D2 (F17) | rounding |
| Pivot bound | `m[i] >= 1.0 + abs(upper[i])` for $i \le N-2$ and `m[n-1] >= 1.0` | P5 | holds in exact arithmetic; in floating point the computed margin can lie slightly below 1 (D1 Section 9.4), so the report gives the smallest computed pivot margin and the check is an assertion only at rounding level |
| Non-negativity | `min(c_new) >= 0.0` for a non-negative `c_old` | P4, P6 | exact comparison with zero, because P6 proves it for (F24) whenever every pivot is positive |
| Maximum principle | `max(c_new) <= max(c_old)` | P3, P4 | exact in exact arithmetic; in floating point it may exceed by rounding, so the bound uses a slack fixed by the rounding analysis (RP-4) and is reported |
| Monotone release | `F` non-decreasing over the series, $0 \le F \le 1$ | P4 | exact comparison on the output rows (the first violation, if any, is reported with its step) |
| Remaining inventory | $M^j$ non-increasing for $G = \lambda = 0$ | P4, (F29) | as above |
| Balance residual | $\lvert \varepsilon^j \rvert / M^0$, worst over all steps, and the step where it occurs | P9, P18 | **measured and reported**; the tolerance is not set here (RP-4) |
| Initial inventory | $\lvert M^0 - \tfrac{4\pi}{3}\left(R_1\right)^3 C_{\mathrm{init}} \rvert / M^0$ | P10 | rounding |
| Interface flux | for each of the four interface nodes, $\lvert \Phi_{\mathrm{from\ inner}} - \Phi_{\mathrm{from\ outer}} \rvert$ divided by the larger of the two, at chosen output times (PS-15) | P8, P8a | rounding at the level of the equal-expression identity; P8b is observed only |
| Centre decoupling | rows $1$ to $N-1$ solved with `c_old[0]` replaced by another value give the same `c_new[1..]` | P7 | exact (bitwise): the algorithm never reads `c_old[0]` in rows $\ge 1$ |
| Weights | $V_0 = 0$; $V_i > 0$ for $1 \le i \le N-1$; $V_i$ equals the sum of its inner and outer half-interval parts (the two terms used in PS-15). The sum $\sum_{i=1}^{N-1} V_i$ is a trapezoidal approximation of $\left(R_5\right)^3/3$, not equal to it, and is reported only as a consistency number that must shrink as the grid is refined | F7 | rounding for the first three; the last is reported, not asserted |

**Notes.** A check that compares with a tolerance takes the tolerance as an argument; the checks in this document contain no default. Where a check says "rounding", the test fixes the bound from an operation count before running, in the way the balance tolerance is fixed (RP-4).

### PS-18 `export` : data export

**Purpose.** Write the series, the profiles and the run metadata in a reproducible text form, and expose the same arrays to the WebAssembly wrapper for download.

**Inputs.** `RunResult` (PS-16); an output path (native) or none (wrapper).

**Outputs.** Files `timeseries.csv`, `final_profile.csv`, `snapshots.csv`, `run_metadata.csv` (native), or the same arrays in the wrapper's JSON reply. Units are in the column names.

**Module and type.** `postprocessing/export.rs`, `pub fn write_timeseries(path: &Path, series: &TimeSeries) -> Result<()>`, `pub fn write_profile(path: &Path, grid: &Grid, c: &[f64]) -> Result<()>`, `pub fn write_metadata(path: &Path, meta: &RunMeta) -> Result<()>`; they use the existing `csv` dependency. JSON for the browser is produced in `crates/strato-wasm` from the same `RunResult`; the wrapper owns serialisation only, no numerics.

**Implements.** Presentation of (F1) (grid), (F18), (F21), (F25) to (F31).

Columns:

| File | Columns (unit in header) |
|---|---|
| `timeseries.csv` | `t_s`, `inventory_mol`, `cumulative_release_mol`, `release_fraction`, `release_rate_mol_s` (empty at `t_s = 0`), `balance_residual_mol`, `balance_residual_relative`, `min_concentration_mol_m3`, `max_concentration_mol_m3` |
| `final_profile.csv`, `snapshots.csv` | `t_s` (snapshots only), `node_index`, `r_m`, `region` (name of the material on the outer side; at an interface node, `interface_inner|outer`), `node_class` (`centre`, `interior`, `interface`, `outer`), `concentration_mol_m3` |
| `run_metadata.csv` | solver label and version, `n_total_intervals` (N), `n_intervals_<material>` (five rows), `dr_<material>_m`, `d_<material>_m2_s`, `dt_s` (effective), `steps`, `c_init_mol_m3`, `theta`, `initial_inventory_mol`, `fo_<material>` |

Rules:

1. Numbers are written in the shortest decimal form that reads back to the same `f64` (the Rust default for `{}` on `f64`), so a file re-read in Rust is bit-identical to the run that wrote it.
2. A non-finite value is never written; PS-16 has already refused it. The empty `release_rate_mol_s` at $t = 0$ is the only empty cell.
3. A file records what was computed. It carries the label "numerical solution of the stated model" and no label that suggests experimental validation (directive Section 11).
4. The export writes no absolute path, no user name and no host name into any file.
5. `final_profile.csv` has $N+1$ rows including the centre node ($C_0^j$, reported but not part of the inventory) and the outer node ($C_N^j = 0$ by (F18)).

### PS-19 Hooks for generation, decay and temperature-dependent diffusion (Phase G)

**Status.** These are *hooks*, not algorithms. D2 Section 9 does not derive the additional physics; its derivation, review and verification come first (directive Phase G). This section only fixes where each term would enter the routines above, so that the code is not reorganised later. No code is written for G1 to G3 before then, and the baseline run is bit-identical with the hooks present and inactive.

| Mechanism | Enters at | Change to the routines | Constraint from D2 Section 9 |
|---|---|---|---|
| G1 generation, source $G$ ($\mathrm{mol\ m^{-3}\ s^{-1}}$) | PS-11 right-hand side (F20); PS-14 balance; maybe PS-13 | `rhs[i] = c_old[i] + source_term(i)`; new accumulator `generated`; new `StepDiagnostics.generated` | the G1 derivation fixes the time level, the weighting at interface nodes, the centre node ($V_0 = 0$), and the release formula when the source is non-zero at $R_5$ |
| G2 decay, $\lambda$ ($\mathrm{s^{-1}}$) | PS-05 diagonals of (F11), (F12), (F17); PS-14 balance | `diag[i] += decay_term(i)`; new accumulator `decayed`; new `StepDiagnostics.decayed` | decay is booked separately from the physical release |
| G3 temperature, $D_{\mathrm{material}}(T)$ | PS-03 `DiffusivitySet`; PS-04, PS-05, PS-10 on every change | `Stepper::rebuild` is called whenever $T$ changes; the same $D$ values feed (F6), (F8), (F11), (F12), (F14), (F16) and (F26) | the G3 derivation fixes the time level at which $T$ is evaluated |

Until each derivation is reviewed, the corresponding term is the absent term: the pseudocode of this document is complete without it.

---

## 3. Coverage of Document 2

Every equation of D2 appears in a routine above. The reverse index, equation to routine, with the planned Rust item and the test, is Document 4.

| D2 section | Content | Routines |
|---|---|---|
| 2 | inputs, units | PS-01 |
| 3 | grid, material of each interval, node classes | PS-02, PS-03 |
| 4 | spacings, conductances, weights, Fourier numbers | PS-02, PS-04 |
| 5.1 | general row | PS-05 |
| 5.2, 5.3 | centre row, node 1 | PS-06, PS-05 |
| 5.4 | interior rows | PS-05 (formula), PS-07 (check) |
| 5.5 | interface rows | PS-05 (formula), PS-07 (check) |
| 5.6, 5.7 | row $N-1$, outer boundary | PS-05, PS-08 |
| 5.8 | assembled system | PS-05 |
| 6.1 | right-hand side | PS-11 |
| 6.2 | initial vector | PS-09 |
| 6.3 | tridiagonal solution | PS-10 |
| 7.1 to 7.4 | inventory, release, fraction, balance | PS-12, PS-13, PS-14 |
| 7.5 | interface and centre diagnostics | PS-15 |
| 8 | proven properties | PS-17 |
| 9 | additional physics placeholders | PS-19 |

---

## 4. Review points

These are the places where this pseudocode makes a choice that D2 leaves open, or where it needs a decision before code is written. They are listed so that the reviewer can accept or change each one. None of them changes a matrix coefficient of D2.

| # | Point | Choice made here | Alternative |
|---|---|---|---|
| RP-1 | $C_{\mathrm{init}} = 0$. D2 allows $C_{\mathrm{init}} \ge 0$, but (F28) divides by $M^0$, which is then zero. | PS-01 requires $C_{\mathrm{init}} > 0$ for a run. | accept $C_{\mathrm{init}} = 0$ and define $F = 0$; needed only once a source (G1) can start from an empty particle |
| RP-2 | Step count. D2 fixes $\Delta t$ and $J_{\mathrm{steps}}$ as inputs; the existing configuration gives an end time. | `steps = ceil(end / dt)`, `dt = end / steps` (the rule of the existing code). A floating-point quotient just above an integer would add one step. | require `end / dt` to be an integer within a stated tolerance and reject otherwise |
| RP-3 | `release_rate` at $j = 0$. | `Option<f64>`, `None` at $j = 0$, because $\Phi^0$ is not defined (D2 Section 7.2). The wrapper contract and the web types change accordingly. | a sentinel value; rejected, because a sentinel can be mistaken for data |
| RP-4 | Tolerances: balance residual, maximum principle slack, row margin, comparison of the explicit forms with (F11). | None is fixed here. Each is set from an operation count or a rounding analysis of the Rust code, and recorded before the first run (D2 Sections 5.1 and 7.4). | |
| RP-5 | Compensated summation of $M_{\mathrm{rel}}$. | Kept, as in the finite-volume code. It changes only rounding. | plain summation; D1 Section 9.4 shows both orders of magnitude of residual in scratch programs, with no clear ranking |
| RP-6 | $\Delta r_{i,\mathrm{outer}}$ in (F6) and (F26): D2 Section 7.2 notes that $r_N - r_{N-1}$ and $\left(R_5 - R_4\right)/n_{\mathrm{OPyC}}$ can differ in the last digit. | the stored difference `r[i+1] - r[i]` everywhere, as D2 prescribes. | none: D2 normative |
| RP-7 | Cross-reference in D2. In the introduction of D2 Section 5 the sentence "$C_N^{j+1}$ is not an unknown (F15)" cites (F15), which is the kernel interior row. The boundary condition is (F18). | This document and Document 4 use (F18). | D2 is final and reviewed; the owner or the coordinator may correct the cross-reference |
| RP-8 | The module `coefficients.rs` is an addition to the directive Section 10 list. | Separates $K_{i,\mathrm{outer}}$ and $\mathrm{Fo}_{\mathrm{material}}$ from the grid, so that a change of $D$ or $\Delta t$ (PS-19) does not touch the grid. | fold into `btcs.rs` |
