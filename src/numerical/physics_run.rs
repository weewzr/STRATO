//! Generation, radioactive decay and temperature-dependent diffusion in the five-layer BTCS
//! solver (directive section 17, G1 to G4).
//!
//! Implements Document 2 Section 11, equations (F32) to (F46), derived in Document 5:
//!
//! * G1, the source: step averages of the per-material source (F32), the node weights (F33)
//!   with the kernel closure factor `omega`, the nodal densities (F34) and the right-hand side
//!   (F38);
//! * G2, implicit decay: the decay number `kappa = lambda dt` on every diagonal entry (F35),
//!   decay booked separately from release (F40);
//! * G3, temperature-dependent diffusion: the diffusivity of each material evaluated at the
//!   middle of the step (F36) from an Arrhenius law supplied as data (one term or the published
//!   sum of terms; nothing is chosen in code) and the matrix reassembled whenever it changes;
//! * G4, the combined scheme: release with the outer-node source (F39), the full balance of
//!   initial, generated, remaining, released and decayed amounts (F41), the three release
//!   fractions (F42) and the rounding bound (F46) as the tolerance of the residual.
//!
//! The baseline solver of [`super::btcs`] is not changed in behaviour: with no source, no decay
//! and a constant diffusivity this module performs the same floating-point operations in the
//! same order, so its output equals [`super::btcs::run_input`] bit for bit (P19, P29), and the
//! baseline rounding bound of [`super::diagnostics::BalanceBound`] is then used (Document 2
//! Section 11.7). A run with a source, decay or a changing temperature uses (F46).
//!
//! The fission-rate density is an explicit input with no default (decisions 14 and 42). A
//! direct-yield source for an activation product is refused (decision 15). The result is a
//! numerical solution of the stated model; it is not experimental validation.
//!
//! Like the rest of the solver, the module uses only `std` without files, clocks or threads, so
//! it builds for `wasm32-unknown-unknown`.

use super::{
    btcs::Stepper,
    diagnostics::{check_matrix_structure, BalanceBound, MatrixReport, StepDiagnostics},
    grid::{Grid, ValidatedInput},
    inventory::{initial_vector, inventory, Accumulator},
};
use crate::physics::{
    arrhenius::{ArrheniusLaw, GasConstant, RangeStatus, Temperature},
    source::{source_mol_per_m3_s, FissionYield, ProductKind},
};
use anyhow::{bail, Result};
use std::f64::consts::PI;

// ---------------------------------------------------------------------------------------
// Histories
// ---------------------------------------------------------------------------------------

/// How a table of values is read between its time points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interpolation {
    /// The value of a point holds from its time until the next point; the last value holds
    /// for ever.
    Step,
    /// Straight lines between the points; the table must reach the end of the run.
    Linear,
}

/// A quantity that depends on time only: constant, piecewise constant or piecewise linear.
/// Step averages over a step are exact for these forms (F32).
#[derive(Clone, Debug, PartialEq)]
pub enum TimeFunction {
    /// The same value at all times.
    Constant(f64),
    /// Points `(times[k], values[k])`; `times[0] = 0`, strictly increasing.
    Table {
        /// Time points in s.
        times: Vec<f64>,
        /// Values at the points, in the unit of the quantity.
        values: Vec<f64>,
        /// Reading rule between the points.
        mode: Interpolation,
    },
}

impl TimeFunction {
    /// A constant history.
    ///
    /// # Errors
    ///
    /// A value that is not finite.
    pub fn constant(value: f64) -> Result<Self> {
        if !value.is_finite() {
            bail!("a history value must be finite");
        }
        Ok(TimeFunction::Constant(value))
    }

    /// A tabulated history. The first time must be exactly 0, the times strictly increasing,
    /// all numbers finite.
    ///
    /// # Errors
    ///
    /// A table that violates one of those rules.
    pub fn table(times: Vec<f64>, values: Vec<f64>, mode: Interpolation) -> Result<Self> {
        if times.is_empty() || times.len() != values.len() {
            bail!("a history needs at least one point and equally many times and values");
        }
        if times[0] != 0.0 {
            bail!("a history must start at time 0 s");
        }
        if !times.iter().all(|t| t.is_finite()) || !values.iter().all(|v| v.is_finite()) {
            bail!("history times and values must be finite");
        }
        if times.windows(2).any(|w| w[1] <= w[0]) {
            bail!("history times must be strictly increasing");
        }
        Ok(TimeFunction::Table {
            times,
            values,
            mode,
        })
    }

    /// True if the value never changes.
    pub fn is_constant(&self) -> bool {
        match self {
            TimeFunction::Constant(_) => true,
            TimeFunction::Table { values, .. } => values.windows(2).all(|w| w[0] == w[1]),
        }
    }

    /// True if the value is zero at every time.
    pub fn is_zero(&self) -> bool {
        match self {
            TimeFunction::Constant(v) => *v == 0.0,
            TimeFunction::Table { values, .. } => values.iter().all(|v| *v == 0.0),
        }
    }

    /// Smallest value over all points.
    pub fn min_value(&self) -> f64 {
        match self {
            TimeFunction::Constant(v) => *v,
            TimeFunction::Table { values, .. } => {
                values.iter().copied().fold(f64::INFINITY, f64::min)
            }
        }
    }

    /// True if the history is defined on `0 ..= end_s`.
    pub fn covers(&self, end_s: f64) -> bool {
        match self {
            TimeFunction::Table {
                times,
                mode: Interpolation::Linear,
                ..
            } => times[times.len() - 1] >= end_s,
            _ => true,
        }
    }

    /// The same history with every value mapped through `f` (a unit conversion or the yield
    /// factor of the source).
    ///
    /// # Errors
    ///
    /// An error of `f`.
    pub fn map_values(&self, f: impl Fn(f64) -> Result<f64>) -> Result<Self> {
        match self {
            TimeFunction::Constant(v) => TimeFunction::constant(f(*v)?),
            TimeFunction::Table {
                times,
                values,
                mode,
            } => {
                let mut out = Vec::with_capacity(values.len());
                for v in values {
                    out.push(f(*v)?);
                }
                TimeFunction::table(times.clone(), out, *mode)
            }
        }
    }

    fn piece(times: &[f64], t: f64) -> usize {
        times.partition_point(|&x| x <= t).saturating_sub(1)
    }

    fn lerp(times: &[f64], values: &[f64], k: usize, x: f64) -> f64 {
        values[k] + (values[k + 1] - values[k]) * ((x - times[k]) / (times[k + 1] - times[k]))
    }

    /// Value at time `t` (s). For a linear table a time past the last point reads the last
    /// value (the difference is a rounding of the end time of the run, see
    /// [`PhysicsInput::validate`]).
    pub fn value_at(&self, t: f64) -> f64 {
        match self {
            TimeFunction::Constant(v) => *v,
            TimeFunction::Table {
                times,
                values,
                mode,
            } => {
                let k = Self::piece(times, t);
                match mode {
                    Interpolation::Step => values[k],
                    Interpolation::Linear => {
                        if k + 1 >= times.len() {
                            values[k]
                        } else {
                            Self::lerp(times, values, k, t)
                        }
                    }
                }
            }
        }
    }

    /// Mean value over `a < t < b`, `(1/(b - a)) int f dt`, exact for the three forms (F32).
    /// A step that lies inside one piece returns that piece's value with no rounding (a
    /// constant source over a step is `G` itself); a linear piece returns the value at the
    /// middle of the step.
    pub fn mean_over(&self, a: f64, b: f64) -> f64 {
        match self {
            TimeFunction::Constant(v) => *v,
            TimeFunction::Table {
                times,
                values,
                mode,
            } => {
                let k0 = Self::piece(times, a);
                let last = times.len() - 1;
                let single = k0 >= last || times[k0 + 1] >= b;
                match (mode, single) {
                    (Interpolation::Step, true) => values[k0],
                    (Interpolation::Linear, true) => {
                        if k0 >= last || values[k0] == values[k0 + 1] {
                            values[k0]
                        } else {
                            Self::lerp(times, values, k0, 0.5 * (a + b))
                        }
                    }
                    _ => {
                        let mut acc = 0.0;
                        let mut t = a;
                        let mut k = k0;
                        while t < b {
                            let seg_end = if k < last { times[k + 1].min(b) } else { b };
                            let (va, vb) = match mode {
                                Interpolation::Step => (values[k], values[k]),
                                Interpolation::Linear if k < last => (
                                    Self::lerp(times, values, k, t),
                                    Self::lerp(times, values, k, seg_end),
                                ),
                                Interpolation::Linear => (values[k], values[k]),
                            };
                            acc += (seg_end - t) * (va + vb) / 2.0;
                            t = seg_end;
                            k += 1;
                        }
                        acc / (b - a)
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// Source, decay and diffusivity models
// ---------------------------------------------------------------------------------------

/// The volumetric source of each material, `G_material(t)` in mol m^-3 s^-1 (assumption 1 of
/// Document 5: constant in space inside a material, any non-negative history in time).
#[derive(Clone, Debug, PartialEq)]
pub struct SourceModel {
    /// Kernel, buffer, IPyC, SiC, OPyC.
    pub per_material: [TimeFunction; 5],
}

impl SourceModel {
    /// No source in any material.
    pub fn none() -> Self {
        SourceModel {
            per_material: std::array::from_fn(|_| TimeFunction::Constant(0.0)),
        }
    }

    /// The direct fission-product source of the kernel, `G_kernel = y F_dot / N_A` (Document
    /// 5, Eq. (2)); the coatings have none. `rate_density` is the fission-rate density history
    /// in fissions m^-3 s^-1; it is an explicit input with no default (decisions 14 and 42).
    ///
    /// # Errors
    ///
    /// A direct-yield source for an activation product (Ag-110m, Cs-134, Eu-154 and similar,
    /// decision 15), or a rate or yield that is negative or not finite.
    pub fn kernel_fission(
        rate_density: &TimeFunction,
        yield_: FissionYield,
        product: ProductKind,
    ) -> Result<Self> {
        let kernel = rate_density.map_values(|x| Ok(source_mol_per_m3_s(x, yield_, product)?))?;
        let mut s = SourceModel::none();
        s.per_material[0] = kernel;
        Ok(s)
    }

    /// True if every history is zero (no generation).
    pub fn is_zero(&self) -> bool {
        self.per_material.iter().all(TimeFunction::is_zero)
    }
}

/// How the diffusivity of each material is obtained.
#[derive(Clone, Debug)]
pub enum DiffusivityModel {
    /// The diffusivities of the validated input, constant in time.
    Constant([f64; 5]),
    /// One Arrhenius law per material, evaluated at the temperature at the middle of each step
    /// (F36). Each law is one term or a documented sum of terms: the caller supplies it, no
    /// form is chosen here (decisions 13 and 39).
    Arrhenius {
        /// Law of the kernel, buffer, IPyC, SiC, OPyC.
        laws: Box<[ArrheniusLaw; 5]>,
        /// Gas constant used with the laws.
        gas: GasConstant,
        /// Particle temperature in K.
        temperature_k: TimeFunction,
    },
}

/// Per material: in how many steps the law was evaluated inside, below or above its stated
/// validity range, or had no range stated. A temperature outside the range is reported, never
/// clamped.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RangeReport {
    /// Steps inside the stated range of every term.
    pub within: usize,
    /// Steps with some term below its stated minimum.
    pub below: usize,
    /// Steps with some term above its stated maximum (and none below).
    pub above: usize,
    /// Steps of a law whose terms state no range.
    pub not_stated: usize,
}

/// Everything beyond the baseline input.
#[derive(Clone, Debug)]
pub struct PhysicsInput {
    /// Grid, `C_init`, step and step count (as validated for the baseline solver).
    pub base: ValidatedInput,
    /// Requested end time in s (histories must cover it).
    pub end_time_s: f64,
    /// Decay constant `lambda` in s^-1 (0 for a stable species).
    pub decay_constant_per_s: f64,
    /// The source.
    pub source: SourceModel,
    /// The diffusivity.
    pub diffusivity: DiffusivityModel,
}

impl PhysicsInput {
    /// A baseline input: no source, no decay, the constant diffusivities of `base`.
    pub fn baseline(base: ValidatedInput, end_time_s: f64) -> Self {
        let d = base.d;
        PhysicsInput {
            base,
            end_time_s,
            decay_constant_per_s: 0.0,
            source: SourceModel::none(),
            diffusivity: DiffusivityModel::Constant(d),
        }
    }

    /// Checks the hypotheses of Document 5 that the data must meet: `lambda` finite and not
    /// negative, every source history non-negative, every history defined on the whole run, a
    /// positive temperature.
    ///
    /// # Errors
    ///
    /// The first violated condition.
    pub fn validate(&self) -> Result<()> {
        let lambda = self.decay_constant_per_s;
        if !(lambda.is_finite() && lambda >= 0.0) {
            bail!("the decay constant must be finite and not negative");
        }
        if !(self.end_time_s.is_finite() && self.end_time_s > 0.0) {
            bail!("the end time must be finite and positive");
        }
        for (k, g) in self.source.per_material.iter().enumerate() {
            if g.min_value() < 0.0 {
                bail!("the source of material {k} must not be negative (G >= 0)");
            }
            if !g.covers(self.end_time_s) {
                bail!("the source history of material {k} ends before the end of the run");
            }
        }
        match &self.diffusivity {
            DiffusivityModel::Constant(d) => {
                if !d.iter().all(|x| x.is_finite() && *x > 0.0) {
                    bail!("every diffusivity must be finite and positive");
                }
            }
            DiffusivityModel::Arrhenius { temperature_k, .. } => {
                if !temperature_k.covers(self.end_time_s) {
                    bail!("the temperature history ends before the end of the run");
                }
                if temperature_k.min_value() <= 0.0 {
                    bail!("the temperature must be positive (kelvin)");
                }
            }
        }
        Ok(())
    }

    /// True if the run is the baseline problem: no source, no decay, a diffusivity that does
    /// not change in time. Such a run uses the baseline rounding bound.
    pub fn is_baseline(&self) -> bool {
        self.source.is_zero()
            && self.decay_constant_per_s == 0.0
            && match &self.diffusivity {
                DiffusivityModel::Constant(_) => true,
                DiffusivityModel::Arrhenius { temperature_k, .. } => temperature_k.is_constant(),
            }
    }
}

// ---------------------------------------------------------------------------------------
// Source weights, (F33) and (F34)
// ---------------------------------------------------------------------------------------

/// Node weights and nodal densities of the source for one step.
#[derive(Clone, Debug)]
pub struct SourceNodes {
    /// `Q_i` of (F33), length `N + 1`, mol/s; `q[0] = 0` (the centre has no weight).
    pub q: Vec<f64>,
    /// `g_i` of (F34), length `N`, mol m^-3 s^-1.
    pub g: Vec<f64>,
    /// `sum_{i=1}^{N} Q_i`, mol/s (node `N` included), accumulated in increasing `i`.
    pub sum_q: f64,
    /// True if some `g_i` is positive (the flag `chi` of (F46)).
    pub active: bool,
}

/// Source weights (F33) and nodal densities (F34) from the step-averaged source of each
/// material `gbar` (F32). The operation order is the one the rounding bound (F46) is derived
/// for: `Q_i = fl(fl(r_i r_i / 2) fl(dr_in G_in + dr_out G_out))` for an ordinary node, the
/// bracket of (F33) with `omega = 1 - 1/(3 n_kernel)` at node `N_kernel`, and
/// `Q_N = fl(fl(R_5 R_5 / 2) dr) G_OPyC` at the outer node; `g_i = fl(Q_i / V_i)` and
/// `g_0 = G_kernel`.
pub fn source_nodes(grid: &Grid, gbar: [f64; 5]) -> SourceNodes {
    let n = grid.n;
    let nk = grid.interface_nodes[0];
    let omega = 1.0 - 1.0 / (3.0 * nk as f64);
    let mut q = vec![0.0; n + 1];
    for i in 1..n {
        let half_r2 = grid.r[i] * grid.r[i] / 2.0;
        let dr_in = grid.dr_inner(i);
        let dr_out = grid.dr_outer(i);
        let g_in = gbar[grid.region_of_interval[i - 1].index()];
        let g_out = gbar[grid.region_of_interval[i].index()];
        q[i] = if i == nk {
            half_r2 * (dr_in * (omega * g_in + (1.0 - omega) * g_out) + dr_out * g_out)
        } else {
            half_r2 * (dr_in * g_in + dr_out * g_out)
        };
    }
    q[n] = grid.r[n] * grid.r[n] / 2.0 * grid.dr_inner(n) * gbar[4];
    let mut g = vec![0.0; n];
    g[0] = gbar[0];
    for ((gi, qi), vi) in g.iter_mut().zip(q.iter()).zip(grid.v.iter()).skip(1) {
        *gi = qi / vi;
    }
    let mut sum_q = 0.0;
    for qi in q.iter().skip(1) {
        sum_q += qi;
    }
    let active = g.iter().any(|x| *x > 0.0);
    SourceNodes {
        q,
        g,
        sum_q,
        active,
    }
}

// ---------------------------------------------------------------------------------------
// The rounding bound (F46)
// ---------------------------------------------------------------------------------------

fn gamma(k: f64) -> f64 {
    k * BalanceBound::U / (1.0 - k * BalanceBound::U)
}

/// Quantities that enter the bound (F46) at the end of step `j`.
#[derive(Clone, Copy, Debug)]
pub struct BoundTerms {
    /// `N`, the index of the outer node.
    pub n: usize,
    /// `J`, the number of steps taken.
    pub steps_taken: usize,
    /// `sum_{k=1}^{J} S^k` of (F43), mol per `4 pi`.
    pub sum_s: f64,
    /// `sum_{k=0}^{J-1} chi^k M^k`, mol.
    pub chi_inventory: f64,
    /// Reported `M^J`.
    pub inventory: f64,
    /// Reported `M^0`.
    pub initial_inventory: f64,
    /// Reported `M_rel^J`.
    pub released: f64,
    /// Reported `M_gen^J`.
    pub generated: f64,
    /// Reported `M_dec^J`.
    pub decayed: f64,
}

/// The right side of (F46), in mol: a rigorous bound on `|eps^J|` for the combined scheme
/// (Document 5, Section 5.8, Theorem R-comb), valid under its hypotheses (IEEE double with
/// rounding to nearest, no overflow and no subnormal products, positive computed pivots, the
/// operation orders of the code, `eta <= 1`). Document 5 evaluates the bound in floating
/// point and shows that the factor `1 + eta` covers its own rounding.
pub fn combined_balance_bound(t: &BoundTerms) -> f64 {
    let u = BalanceBound::U;
    let nn = t.n as f64;
    let jj = t.steps_taken as f64;
    let p_hat = 4.0 * PI;
    let eta = 4.0 * (3.0 * nn + jj + 10.0) * u;
    let c_a = gamma(4.0) / (1.0 - gamma(4.0)) + gamma(4.0) / (1.0 - gamma(1.0));
    let s_j = jj;
    let bracket = p_hat * c_a * t.sum_s
        + u * t.chi_inventory
        + gamma(nn) * (t.inventory + t.initial_inventory)
        + (gamma(5.0) + s_j * u) * t.released
        + (gamma(nn + 1.0) + gamma(3.0) + s_j * u) * t.generated
        + (gamma(nn + 1.0) + s_j * u) * t.decayed
        + gamma(2.0) * (t.inventory + t.released + t.initial_inventory + t.generated);
    (1.0 + eta) * bracket
}

// ---------------------------------------------------------------------------------------
// Results
// ---------------------------------------------------------------------------------------

/// Which rounding bound belongs to a run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundKind {
    /// The baseline problem: the derived bound of [`BalanceBound`] (Document 2 Section 7.4).
    Baseline,
    /// A source, decay or a changing temperature: the bound (F46).
    Combined,
}

/// The output quantities of a row that the baseline row [`StepDiagnostics`] does not have.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsRow {
    /// `M_gen^j` (F40), mol.
    pub generated: f64,
    /// `M_dec^j` (F40), mol; never part of the release.
    pub decayed: f64,
    /// `F_gen^j = M_rel^j / (M^0 + M_gen^j)` (F42), dimensionless, in `[0, 1]` up to rounding.
    pub release_fraction_of_total: f64,
    /// `beta^j`, release rate over generation rate (F42); `None` at `j = 0` and where the
    /// generation rate is zero.
    pub release_to_generation_ratio: Option<f64>,
    /// Temperature in K at the output time, `None` for a constant diffusivity.
    pub temperature_k: Option<f64>,
    /// `eps^j / (M^0 + M_gen^j)`, dimensionless.
    pub balance_residual_relative: f64,
}

/// Metadata of a run with the additional mechanisms.
#[derive(Clone, Debug)]
pub struct PhysicsMeta {
    /// Effective `dt` in s.
    pub dt: f64,
    /// Number of steps.
    pub steps: usize,
    /// `N`.
    pub n: usize,
    /// `theta` of (F22).
    pub theta: f64,
    /// `M^0` in mol.
    pub initial_inventory: f64,
    /// Decay number `kappa = lambda dt` (F35), the one stored number.
    pub decay_number: f64,
    /// Which bound is used for the residual.
    pub bound_kind: BoundKind,
    /// How often the matrix was assembled and factored.
    pub assemblies: usize,
    /// Smallest row margin over all assembled matrices (P1, P24: at least `1 + kappa`).
    pub matrix_min_row_margin: f64,
    /// Largest `|B_i - |L_i| - |U_i| - 1 - kappa|` over rows `0 ..= N-2` of all matrices.
    pub matrix_max_interior_margin_deviation: f64,
    /// Largest row sum of absolute values over all matrices.
    pub matrix_norm_inf: f64,
    /// Smallest `m_i - |U_i|` over all factorisations.
    pub min_pivot_margin: f64,
    /// Largest `|U_i| / m_i` over all factorisations.
    pub max_multiplier: f64,
    /// Range report of the diffusivity laws per material (empty counts for a constant `D`).
    pub ranges: [RangeReport; 5],
    /// `sum_i Q_i` of the first step in mol/s (0 if there is no source).
    pub first_step_generation_weight_sum: f64,
}

/// Result of [`run_physics`].
#[derive(Clone, Debug)]
pub struct PhysicsRun {
    /// The grid.
    pub grid: Grid,
    /// Output rows including `j = 0`: baseline row and the additional quantities.
    pub series: Vec<(f64, StepDiagnostics, PhysicsRow)>,
    /// Profile `C_i^J` at the end.
    pub final_profile: Vec<f64>,
    /// Profiles at the steps nearest the requested times.
    pub snapshots: Vec<(f64, Vec<f64>)>,
    /// Metadata.
    pub meta: PhysicsMeta,
}

// ---------------------------------------------------------------------------------------
// The time loop
// ---------------------------------------------------------------------------------------

fn min_max(c: &[f64]) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &x in c {
        lo = lo.min(x);
        hi = hi.max(x);
    }
    (lo, hi)
}

/// What one output row is made of.
struct RowState {
    inventory: f64,
    released: f64,
    generated: f64,
    decayed: f64,
    min_c: f64,
    max_c: f64,
    bound: f64,
    temperature_k: Option<f64>,
}

/// The output row from the running totals. The residual is evaluated in the order of (F46):
/// `fl(fl(fl(M + M_rel) - fl(M^0 + M_gen)) + M_dec)`; with `M_gen = M_dec = 0` it is the
/// baseline residual `fl(fl(M + M_rel) - M^0)` bit for bit.
fn make_row(
    m0: f64,
    s: &RowState,
    phi: Option<f64>,
    generation_rate: f64,
) -> (StepDiagnostics, PhysicsRow) {
    let total = m0 + s.generated;
    let eps = ((s.inventory + s.released) - total) + s.decayed;
    (
        StepDiagnostics {
            inventory: s.inventory,
            cumulative_release: s.released,
            release_fraction: s.released / m0,
            release_rate: phi,
            balance_residual: eps,
            balance_residual_bound: s.bound,
            min_concentration: s.min_c,
            max_concentration: s.max_c,
        },
        PhysicsRow {
            generated: s.generated,
            decayed: s.decayed,
            release_fraction_of_total: s.released / total,
            release_to_generation_ratio: match phi {
                Some(p) if generation_rate > 0.0 => Some(p / generation_rate),
                _ => None,
            },
            temperature_k: s.temperature_k,
            balance_residual_relative: eps / total,
        },
    )
}

fn classify(statuses: &[RangeStatus], report: &mut RangeReport) {
    if statuses.contains(&RangeStatus::BelowMin) {
        report.below += 1;
    } else if statuses.contains(&RangeStatus::AboveMax) {
        report.above += 1;
    } else if statuses.iter().all(|s| *s == RangeStatus::NotStated) {
        report.not_stated += 1;
    } else {
        report.within += 1;
    }
}

/// Runs the combined scheme (PS-16 with the hooks of PS-19): for each step the diffusivities
/// of the step (F36), the step-averaged source (F32) with its weights (F33), (F34), the matrix
/// of (F37) (reassembled when the diffusivities change), the right-hand side (F38), the solve
/// (F24), the release (F39), the generated and decayed amounts (F40), and at output steps the
/// inventory, the three release fractions (F42), the residual of (F41) and its rounding bound.
///
/// # Errors
///
/// Invalid input (see [`PhysicsInput::validate`]), a diffusivity that is not finite and
/// positive, a failed solve, a non-finite output row, or a run so long that the bound (F46)
/// does not apply (`eta > 1`).
pub fn run_physics(input: &PhysicsInput, snapshot_times: &[f64]) -> Result<PhysicsRun> {
    input.validate()?;
    let base = &input.base;
    let grid = Grid::build(base)?;
    let n = grid.n;
    let dt = base.dt;
    let steps = base.steps;
    let (mut c_old, theta, m0) = initial_vector(&grid, base.c_init);
    if !(m0.is_finite() && m0 > 0.0) {
        bail!("initial inventory is not finite and positive");
    }
    let kappa = input.decay_constant_per_s * dt;
    if !kappa.is_finite() {
        bail!("the decay number lambda dt is not finite");
    }
    let kind = if input.is_baseline() {
        BoundKind::Baseline
    } else {
        BoundKind::Combined
    };
    let eta_last = 4.0 * (3.0 * n as f64 + steps as f64 + 10.0) * BalanceBound::U;
    if kind == BoundKind::Combined && eta_last > 1.0 {
        bail!("the run has too many steps for the rounding bound (eta > 1)");
    }
    let p_hat = 4.0 * PI;
    let temperature_of = |t: f64| -> Option<f64> {
        match &input.diffusivity {
            DiffusivityModel::Constant(_) => None,
            DiffusivityModel::Arrhenius { temperature_k, .. } => Some(temperature_k.value_at(t)),
        }
    };

    let mut baseline_bound = BalanceBound::new(n, steps, m0);
    let mut sum_s = 0.0;
    let mut chi_inventory = 0.0;

    let mut c_new = vec![0.0; n + 1];
    let mut released = Accumulator::new();
    let mut generated = Accumulator::new();
    let mut decayed = Accumulator::new();
    let mut m_current = m0;

    let mut stepper: Option<Stepper> = None;
    let mut last_d: [u64; 5] = [0; 5];
    let mut ranges = [RangeReport::default(); 5];
    let mut meta_margin = f64::INFINITY;
    let mut meta_dev: f64 = 0.0;
    let mut meta_norm: f64 = 0.0;
    let mut meta_pivot = f64::INFINITY;
    let mut meta_mult: f64 = 0.0;
    let mut assemblies = 0usize;

    let mut last_gbar: Option<[u64; 5]> = None;
    let mut nodes = source_nodes(&grid, [0.0; 5]);
    let mut first_sum_q = 0.0;

    let (cmin0, cmax0) = min_max(&c_old);
    let bound0 = match kind {
        BoundKind::Baseline => baseline_bound.first_row(m0) * m0,
        BoundKind::Combined => combined_balance_bound(&BoundTerms {
            n,
            steps_taken: 0,
            sum_s: 0.0,
            chi_inventory: 0.0,
            inventory: m0,
            initial_inventory: m0,
            released: 0.0,
            generated: 0.0,
            decayed: 0.0,
        }),
    };
    let (d0, p0) = make_row(
        m0,
        &RowState {
            inventory: m0,
            released: 0.0,
            generated: 0.0,
            decayed: 0.0,
            min_c: cmin0,
            max_c: cmax0,
            bound: bound0,
            temperature_k: temperature_of(0.0),
        },
        None,
        0.0,
    );
    let mut series = vec![(0.0, d0, p0)];

    let mut wanted: Vec<usize> = snapshot_times
        .iter()
        .map(|&t| ((t / dt).round().max(0.0) as usize).min(steps))
        .collect();
    wanted.sort_unstable();
    wanted.dedup();
    let mut snapshots: Vec<(f64, Vec<f64>)> = Vec::new();
    if wanted.first() == Some(&0) {
        snapshots.push((0.0, c_old.clone()));
    }

    for step in 1..=steps {
        let j = step - 1;
        let t_a = j as f64 * dt;
        let t_b = step as f64 * dt;

        // (F36): the diffusivities of the step, from the temperature at its middle.
        let d_step = match &input.diffusivity {
            DiffusivityModel::Constant(d) => *d,
            DiffusivityModel::Arrhenius {
                laws,
                gas,
                temperature_k,
            } => {
                let t_mid = temperature_k.value_at((j as f64 + 0.5) * dt);
                let mut d = [0.0; 5];
                for (k, law) in laws.iter().enumerate() {
                    let ev = law.evaluate(Temperature::Kelvin(t_mid), *gas)?;
                    if !(ev.d_m2_s.is_finite() && ev.d_m2_s > 0.0) {
                        bail!(
                            "the diffusivity of material {k} at {t_mid} K is not finite and positive"
                        );
                    }
                    classify(&ev.term_ranges, &mut ranges[k]);
                    d[k] = ev.d_m2_s;
                }
                d
            }
        };
        let d_bits: [u64; 5] = std::array::from_fn(|k| d_step[k].to_bits());
        if stepper.is_none() || d_bits != last_d {
            let s = Stepper::new_with_decay(&grid, d_step, dt, kappa)?;
            let rep: MatrixReport = check_matrix_structure(&s.assembly().matrix)?;
            meta_margin = meta_margin.min(rep.min_row_margin);
            meta_dev = meta_dev.max(rep.max_interior_margin_deviation);
            meta_norm = meta_norm.max(rep.norm_inf);
            let pivots = s.factors().pivots();
            let upper = &s.assembly().matrix.upper;
            for (m_i, u_i) in pivots.iter().zip(upper.iter()).take(n - 1) {
                meta_pivot = meta_pivot.min(m_i - u_i.abs());
                meta_mult = meta_mult.max(u_i.abs() / m_i);
            }
            assemblies += 1;
            last_d = d_bits;
            stepper = Some(s);
        }
        let st = stepper.as_mut().expect("assembled above");

        // (F32) to (F34): the source of the step.
        let gbar: [f64; 5] =
            std::array::from_fn(|k| input.source.per_material[k].mean_over(t_a, t_b));
        let gbar_bits: [u64; 5] = std::array::from_fn(|k| gbar[k].to_bits());
        if last_gbar != Some(gbar_bits) {
            nodes = source_nodes(&grid, gbar);
            last_gbar = Some(gbar_bits);
        }
        if step == 1 {
            first_sum_q = nodes.sum_q;
        }

        // (F38) and (F24).
        st.step_with_source(&c_old, &nodes.g, &mut c_new)?;

        // (F39): release with the outer-node source; same stored K as row N-1.
        let phi = p_hat * st.k_last() * c_new[n - 1] + p_hat * nodes.q[n];
        let release_increment = dt * phi;
        released.add(release_increment);
        // (F40): generated and decayed amounts.
        let gen_rate = p_hat * nodes.sum_q;
        generated.add(dt * gen_rate);
        let m_old = m_current;
        let need_inventory = kind == BoundKind::Combined || kappa != 0.0;
        if need_inventory {
            m_current = inventory(&grid, &c_new);
        }
        if kappa != 0.0 {
            decayed.add(kappa * m_current);
        }
        // Bound bookkeeping.
        match kind {
            BoundKind::Baseline => {
                baseline_bound.add_step(&st.assembly().matrix, &grid.v, &c_new, release_increment);
            }
            BoundKind::Combined => {
                let a = &st.assembly().matrix;
                let mut s = 0.0;
                for i in 1..n {
                    let left = a.lower[i].abs() * c_new[i - 1].abs();
                    let right = a.upper[i].abs() * c_new[i + 1].abs();
                    s += grid.v[i] * (left + a.diag[i].abs() * c_new[i].abs() + right);
                }
                sum_s += s;
                if nodes.active {
                    chi_inventory += m_old;
                }
            }
        }
        std::mem::swap(&mut c_old, &mut c_new);
        let t = step as f64 * dt;
        if step % base.output_every == 0 || step == steps {
            let m = if need_inventory {
                m_current
            } else {
                inventory(&grid, &c_old)
            };
            let (cmin, cmax) = min_max(&c_old);
            let bound = match kind {
                BoundKind::Baseline => baseline_bound.row(m, released.value()) * m0,
                BoundKind::Combined => combined_balance_bound(&BoundTerms {
                    n,
                    steps_taken: step,
                    sum_s,
                    chi_inventory,
                    inventory: m,
                    initial_inventory: m0,
                    released: released.value(),
                    generated: generated.value(),
                    decayed: decayed.value(),
                }),
            };
            let (d, p) = make_row(
                m0,
                &RowState {
                    inventory: m,
                    released: released.value(),
                    generated: generated.value(),
                    decayed: decayed.value(),
                    min_c: cmin,
                    max_c: cmax,
                    bound,
                    temperature_k: temperature_of(t),
                },
                Some(phi),
                gen_rate,
            );
            let finite = d.inventory.is_finite()
                && d.cumulative_release.is_finite()
                && d.release_fraction.is_finite()
                && phi.is_finite()
                && d.balance_residual.is_finite()
                && p.generated.is_finite()
                && p.decayed.is_finite()
                && bound.is_finite()
                && cmin.is_finite()
                && cmax.is_finite();
            if !finite {
                bail!("non-finite output at step {step}");
            }
            series.push((t, d, p));
        }
        if wanted.binary_search(&step).is_ok() {
            snapshots.push((t, c_old.clone()));
        }
    }

    Ok(PhysicsRun {
        grid,
        series,
        final_profile: c_old,
        snapshots,
        meta: PhysicsMeta {
            dt,
            steps,
            n,
            theta,
            initial_inventory: m0,
            decay_number: kappa,
            bound_kind: kind,
            assemblies,
            matrix_min_row_margin: meta_margin,
            matrix_max_interior_margin_deviation: meta_dev,
            matrix_norm_inf: meta_norm,
            min_pivot_margin: meta_pivot,
            max_multiplier: meta_mult,
            ranges,
            first_step_generation_weight_sum: first_sum_q,
        },
    })
}
