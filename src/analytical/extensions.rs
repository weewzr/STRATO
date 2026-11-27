//! Method B extensions: radioactive decay and a piecewise-constant kernel source, with the
//! same Sturm-Liouville eigenfunctions as the existing series (`sturm_liouville`), and the
//! a-posteriori drawing mask.
//!
//! Everything is a pure function of its arguments in `f64`: no file access, no global state,
//! no clock, so the module compiles unchanged for `wasm32-unknown-unknown`. The derivations,
//! with every step and the status of every property, are in
//! `docs/mathematics/btcs/06_analytical_extensions.md` (equation numbers (42) onwards; the
//! science design calls its formulas (B1) to (B5)).
//!
//! Notation. Mode `mu` has decay rate `alpha_mu` (s^-1, the series' `eigenvalues`) and weight
//! `w_mu` (the series' `weights`). The decay constant `lambda` (s^-1) is the same in every
//! layer, `a_mu = alpha_mu + lambda`. The kernel volume is `v_k` (m^3), the initial kernel
//! inventory `m0` (mol), the source `G_k` (mol m^-3 s^-1) on `[s_k, s_{k+1})`. Mode
//! inventories obey `dm/dt = -a m + w v_k G(t)`, `m(0) = w m0`.
//!
//! Evaluation. The modal sums (B2) to (B5) run over all modes, and a truncated sum misses
//! the slowly convergent tail `sum_{mu > mu_max} w_mu alpha_mu / a_mu`, of size
//! `W_miss = 1 - sum_{mu <= mu_max} w_mu` (the weights change sign, so the missing weight is
//! only a diagnostic). The evaluation here therefore never sums the tail. It uses
//! `sum_mu w_mu exp(-a_mu theta) = exp(-lambda theta) R(theta)`, with
//! `R(theta) = 1 - F(theta)` the remaining fraction of the existing series (the complete set
//! has `sum_mu w_mu = 1`), and replaces `R(theta)` by `1` for `theta < t_v = 40 / lambda_covered`
//! and by the held modes for `theta >= t_v`. Then every quantity is a closed-form sum over
//! the held modes with the factors `exp(-a_mu t_v)`, and the only approximation is
//! `0 <= 1 - R(theta) <= Delta = 1 - sum_{mu <= mu_max} w_mu exp(-alpha_mu t_v)` for
//! `theta < t_v` (the release is not decreasing), plus the `exp(-40)` content of the
//! omitted modes after `t_v` (the module's convention). `Delta` gives a computed error bound
//! for each quantity (`ModalState::*_error_bound`). The derivation and the status of each
//! property are in the document named above.

use super::sturm_liouville::ModalRelease;
use std::fmt;

/// Number of e-folds after which an omitted mode counts as relaxed (the module's own
/// convention: `min_valid_time = 40 / lambda_covered`, i.e. `exp(-40)`).
const RELAX_EFOLDS: f64 = 40.0;

/// Fraction of the plotted value by which the cover-reduced series may differ (the labelled
/// 5 % convention of science design Section 3.5 and decision 49; not a derived bound).
pub const MASK_RELATIVE_TOLERANCE: f64 = 0.05;

/// Factor of `accuracy_floor()` in item 3 of the mask for the existing series (no source, no
/// decay), and in no other place. Verifier check (v) of the science design (open point O10)
/// compared the series with closed-form modes at values of 1 to 100 times `accuracy_floor()`
/// and observed differences up to 3.34 times the floor in one homogeneous layer (Document 6,
/// Section 5). 4 is that ratio rounded up: a labelled convention (observed, not derived). The
/// extension quantities do not use it: they use the rule of Document 6, Section 8.
pub const SERIES_FLOOR_FACTOR: f64 = 4.0;

/// Factor of `accuracy_floor()` inside `Delta` (49), the computed bound on the release fraction
/// of the kernel step at `t_v`: the shooting error of the weights enters `1 - sum w exp(-alpha
/// t_v)` with at most the observed ratio of check (v) (3.34, rounded up to 4). Document 6,
/// Section 2.3 and Section 8 (the observed sum of the weight errors of the homogeneous
/// particle is 2.9 floors at 64 modes and 4.3 floors at 1024 modes).
pub const EARLY_RELEASE_FLOOR_FACTOR: f64 = 4.0;

/// Whether the accuracy floor of the series has been re-estimated for (B2) to (B5) (science
/// design Section 4.2, item 3, decision 68(c)): the estimate of Document 6, Section 8 exists
/// AND the independent verification of it passed (decision 68(c) allows `true` only then).
/// With `false`, [`KernelRun::drawn`] returns [`Withheld::FloorNotReestimated`] for every
/// quantity of a run with decay or a non-zero source. The independent verification of the
/// estimate of Document 6, Section 8 passed (log `analytical-extensions-verify`, round 4, 0
/// blockers, 0 majors) and decision 68(c) authorises setting this constant once it has, so it
/// is `true`; the mask then draws an extension value only where the estimated relative error is
/// at most 5 % inside the measured regime.
pub const EXTENSION_FLOOR_REESTIMATED: bool = true;

/// Factor of `floor * A(t)` (mol) added to the computed bounds of the inventory `M` for the
/// error of the series (Document 6, Equation (54)). It covers the error of the series only:
/// the rounding of the evaluation is a computed bound (`ModalState::inventory_rounding_bound`).
/// The largest observed excess of the error over the computed model and rounding bounds in the
/// designed sets is 1.86 times `floor * A` (Table 8.2); 6 is a margin of 3.2.
pub const EXT_FACTOR_INVENTORY: f64 = 6.0;
/// As [`EXT_FACTOR_INVENTORY`] for the released inventory `M_rel`: the largest observed excess
/// is 1.86 (full set, against references with a computed rounding bound), so 6 is a margin of
/// 3.2. The uniform sphere of the physics module, whose reference has no rounding bound, shows
/// 0.9 to 3.1 in different builds: noise of that reference (Document 6, Section 8.4).
pub const EXT_FACTOR_RELEASE: f64 = 6.0;
/// Factor of `floor * S_Phi(t)` (mol s^-1) added to the computed bounds of the release rate
/// `Phi`: the largest observed excess is 1.86, so 6 is a margin of 3.2.
pub const EXT_FACTOR_FLUX: f64 = 6.0;

/// The regime in which the error estimate of Document 6, Section 8 was measured. A run outside
/// it is withheld ([`Withheld::OutsideEstimatedRegime`]): nothing is claimed there. Smallest
/// and largest number of modes of the series.
pub const REGIME_MODES: (usize, usize) = (16, 2048);
/// Largest ratio `lambda / alpha_1` of the designed set (`alpha_1` the slowest mode rate of the
/// series). The set covers the registered decay constants of `data/nuclear/decay.csv` from
/// Cs-137 (7.3e-10 s^-1) to Ag-110 (2.8e-2 s^-1) on geometries whose `alpha_1` lies between
/// 1.4e-9 and 6.4e-6 s^-1 (Table 8.1), so the ratio reaches 1.96e7.
pub const REGIME_LAMBDA_OVER_ALPHA1_MAX: f64 = 1.9e7;
/// Largest time in units of `1 / alpha_1`.
pub const REGIME_ALPHA1_T_MAX: f64 = 1000.0;
/// Largest number of pieces of the source history.
pub const REGIME_PIECES_MAX: usize = 13;
/// Largest `accuracy_floor()` of the designed set (the strongly contrasted set, 1.62e-9).
pub const REGIME_FLOOR_MAX: f64 = 2.0e-9;
/// Smallest and largest outer radius (m) of the particles of the designed set (Table 8.1: a
/// uniform sphere of 175 um to a layered particle of 465 um). A run must declare its geometry
/// ([`KernelRun::with_geometry`]); outside this range it is withheld.
pub const REGIME_OUTER_RADIUS: (f64, f64) = (175.0e-6, 465.0e-6);
/// Smallest and largest ratio kernel radius over outer radius of the designed set (0.4 for a
/// kernel of 160 um in 400 um, 1 for the uniform spheres).
pub const REGIME_KERNEL_FRACTION: (f64, f64) = (0.4, 1.0);

/// Number of layers of the particle (kernel, buffer, IPyC, SiC, OPyC), hence the number of
/// entries of [`Decay::PerLayer`].
pub const LAYER_COUNT: usize = 5;

/// Reasons an extension refuses its input. A refusal never produces a number.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtensionError {
    /// [`Decay::PerLayer`] must hold exactly one decay constant per layer ([`LAYER_COUNT`]).
    WrongLayerCount(usize),
    /// The named argument is NaN or infinite.
    NotFinite(&'static str),
    /// The named argument must not be negative.
    Negative(&'static str),
    /// The named argument must be strictly positive.
    NotPositive(&'static str),
    /// The decay constants of the layers are not all equal. The extension needs one decay
    /// constant (D5 assumption 3, true for radioactive decay): with different constants in
    /// different layers the eigenfunctions of the operator change.
    UnequalDecayConstants,
    /// The source history is not a valid piecewise-constant history.
    BadSourceHistory(&'static str),
    /// The series holds no mode (or no mode below the requested cover).
    NoModes,
    /// The series has eigenvalues and weights of different lengths.
    InconsistentSeries,
}

impl fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLayerCount(n) => write!(
                f,
                "{n} decay constants given; exactly one per layer ({LAYER_COUNT}) is required"
            ),
            Self::NotFinite(n) => write!(f, "{n} must be finite"),
            Self::Negative(n) => write!(f, "{n} must not be negative"),
            Self::NotPositive(n) => write!(f, "{n} must be strictly positive"),
            Self::UnequalDecayConstants => write!(
                f,
                "the decay constants of the layers differ; the extension needs the same \
                 decay constant in every layer"
            ),
            Self::BadSourceHistory(why) => write!(f, "invalid source history: {why}"),
            Self::NoModes => write!(f, "the series holds no mode"),
            Self::InconsistentSeries => {
                write!(
                    f,
                    "the series has eigenvalues and weights of different lengths"
                )
            }
        }
    }
}

impl std::error::Error for ExtensionError {}

fn finite(name: &'static str, x: f64) -> Result<f64, ExtensionError> {
    if x.is_finite() {
        Ok(x)
    } else {
        Err(ExtensionError::NotFinite(name))
    }
}

fn non_negative(name: &'static str, x: f64) -> Result<f64, ExtensionError> {
    finite(name, x)?;
    if x >= 0.0 {
        Ok(x)
    } else {
        Err(ExtensionError::Negative(name))
    }
}

fn positive(name: &'static str, x: f64) -> Result<f64, ExtensionError> {
    finite(name, x)?;
    if x > 0.0 {
        Ok(x)
    } else {
        Err(ExtensionError::NotPositive(name))
    }
}

/// Radioactive decay constant(s) in s^-1.
#[derive(Debug, Clone, PartialEq)]
pub enum Decay {
    /// No decay (stable or very long-lived product treated as stable).
    None,
    /// The same decay constant in every layer.
    Uniform(f64),
    /// Exactly one decay constant per layer ([`LAYER_COUNT`] entries). Accepted only when all
    /// are exactly equal; otherwise the extension refuses with
    /// [`ExtensionError::UnequalDecayConstants`] (or [`ExtensionError::WrongLayerCount`]).
    PerLayer(Vec<f64>),
}

impl Decay {
    /// The single decay constant (s^-1), or an error when the conditions of the extension
    /// do not hold (not finite, negative, or different in different layers).
    pub fn constant(&self) -> Result<f64, ExtensionError> {
        match self {
            Self::None => Ok(0.0),
            Self::Uniform(l) => non_negative("decay constant", *l),
            Self::PerLayer(v) => {
                let first = *v
                    .first()
                    .ok_or(ExtensionError::NotFinite("decay constants (empty)"))?;
                if v.len() != LAYER_COUNT {
                    return Err(ExtensionError::WrongLayerCount(v.len()));
                }
                non_negative("decay constant", first)?;
                for &l in v {
                    non_negative("decay constant", l)?;
                    if l != first {
                        return Err(ExtensionError::UnequalDecayConstants);
                    }
                }
                Ok(first)
            }
        }
    }
}

/// A piecewise-constant kernel source: `G_k` (mol m^-3 s^-1) on `[s_k, s_{k+1})`, with
/// `s_0 = 0`. After the last breakpoint the source is zero. An empty history (a single
/// breakpoint 0) is "no source".
#[derive(Debug, Clone, PartialEq)]
pub struct PiecewiseSource {
    breaks: Vec<f64>,
    rates: Vec<f64>,
}

impl PiecewiseSource {
    /// `breaks` = `[s_0 = 0, s_1, ..., s_K]` strictly increasing (s), `rates` = `[G_0, ...,
    /// G_{K-1}]` finite and not negative (mol m^-3 s^-1).
    pub fn new(breaks: Vec<f64>, rates: Vec<f64>) -> Result<Self, ExtensionError> {
        if breaks.is_empty() || breaks[0] != 0.0 {
            return Err(ExtensionError::BadSourceHistory(
                "the first breakpoint must be 0",
            ));
        }
        if rates.len() + 1 != breaks.len() {
            return Err(ExtensionError::BadSourceHistory(
                "need one more breakpoint than rates",
            ));
        }
        for &b in &breaks {
            finite("source breakpoint", b)?;
        }
        if breaks.windows(2).any(|w| w[1] <= w[0]) {
            return Err(ExtensionError::BadSourceHistory(
                "breakpoints must be strictly increasing",
            ));
        }
        for &g in &rates {
            non_negative("source rate", g)?;
        }
        Ok(Self { breaks, rates })
    }

    /// No source.
    pub fn none() -> Self {
        Self {
            breaks: vec![0.0],
            rates: Vec::new(),
        }
    }

    /// One constant rate `rate` (mol m^-3 s^-1) on `[0, duration)`.
    pub fn constant(rate: f64, duration: f64) -> Result<Self, ExtensionError> {
        positive("source duration", duration)?;
        Self::new(vec![0.0, duration], vec![rate])
    }

    /// Breakpoints (s).
    pub fn breaks(&self) -> &[f64] {
        &self.breaks
    }

    /// Rates (mol m^-3 s^-1).
    pub fn rates(&self) -> &[f64] {
        &self.rates
    }

    /// True when every rate is exactly zero (or there is no piece).
    pub fn is_zero(&self) -> bool {
        self.rates.iter().all(|&g| g == 0.0)
    }

    /// The rate at time `t` (mol m^-3 s^-1): the rate of the piece containing `t`, zero after
    /// the last breakpoint.
    pub fn rate_at(&self, t: f64) -> f64 {
        for k in 0..self.rates.len() {
            if t >= self.breaks[k] && t < self.breaks[k + 1] {
                return self.rates[k];
            }
        }
        0.0
    }

    /// `integral_0^t G ds` in mol m^-3: `sum_k G_k (min(t, s_{k+1}) - s_k)` over `s_k < t`.
    pub fn generated_per_volume(&self, t: f64) -> f64 {
        let mut total = 0.0;
        for k in 0..self.rates.len() {
            if self.breaks[k] < t {
                total += self.rates[k] * (t.min(self.breaks[k + 1]) - self.breaks[k]);
            }
        }
        total
    }
}

/// The quantity a drawn value belongs to (science design Section 4.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quantity {
    /// `M(t) / M^0` (needs `M^0 > 0`).
    InventoryFraction,
    /// `F(t) = M_rel(t) / M^0` (needs `M^0 > 0`); the existing series when there is neither
    /// source nor decay.
    ReleaseFraction,
    /// `F_gen(t) = M_rel(t) / (M^0 + M_gen(t))`; absent while `M^0 + M_gen = 0`.
    ReleaseFractionGenerated,
    /// `F_post(t) = (M_rel(t) - M_rel(t_e)) / M(t_e)` for `t >= t_e`.
    PostEventRelease {
        /// Event time `t_e` (s).
        t_event: f64,
    },
    /// `beta(t) = Phi(t) / (V_K G(t))`; absent while `G(t) = 0`.
    Beta,
}

/// The inventories, the flux and their computed error bounds at one time, for one set of
/// modes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModalState {
    /// Time (s).
    pub time: f64,
    /// `M(t)` (mol): inventory inside the particle.
    pub inventory: f64,
    /// `M_rel(t)` (mol): released through the surface.
    pub released: f64,
    /// `M_dec(t)` (mol): lost by decay inside the particle.
    pub decayed: f64,
    /// `M_gen(t) = v_k int_0^t G ds` (mol): generated so far.
    pub generated: f64,
    /// `M^0` (mol).
    pub initial_inventory: f64,
    /// `Phi(t)` (mol s^-1): release rate through the surface.
    pub flux: f64,
    /// The source rate `G(t)` (mol m^-3 s^-1).
    pub source_rate: f64,
    /// `sum_mu w_mu` over the modes held (diagnostic: `1 - weight_sum` is the missing weight).
    pub weight_sum: f64,
    /// Number of modes summed.
    pub modes: usize,
    /// `t_v = 40 / (largest eigenvalue covered)` (s).
    pub relax_time: f64,
    /// `Delta`: the computed bound (plus the series' `accuracy_floor()`) on the release
    /// fraction of the kernel step at `t_v`, hence on `1 - R(theta)` for `theta < t_v`.
    pub early_release: f64,
    /// Bound on `|M - M(exact)|` (mol), from `Delta`.
    pub inventory_error_bound: f64,
    /// Bound on `|M_rel - M_rel(exact)|` (mol), from `Delta`.
    pub release_error_bound: f64,
    /// Bound on `|Phi - Phi(exact)|` (mol s^-1), equation (53) of Document 6; valid from
    /// `t >= t_v` only, `None` before.
    pub flux_error_bound: Option<f64>,
    /// A-posteriori bound (mol) on the rounding error of `M`, from a running error analysis of
    /// the evaluation (compensated sums; Document 6, Section 8.2). It does not contain the
    /// error of the series (the accuracy floor). Zero for the existing series (no source, no
    /// decay), which the floor test of item 3 judges.
    pub inventory_rounding_bound: f64,
    /// As [`ModalState::inventory_rounding_bound`] for `M_rel` (mol).
    pub release_rounding_bound: f64,
    /// As [`ModalState::inventory_rounding_bound`] for `Phi` (mol s^-1).
    pub flux_rounding_bound: f64,
    /// `A(t) = M^0 + V_K sum_{k: s_k < t} G_k (t - s_k)` (mol): the input weighted by its
    /// age, the scale of the cancellation in the sums of (51) (Document 6, Section 8).
    pub age_input: f64,
    /// `V_K sum_{k: s_k < t} G_k + M^0 / t` (mol s^-1): the scale of the flux. The second term
    /// bounds the initial-inventory part of the flux, `M^0 sum_mu w_mu alpha_mu e^{-alpha_mu t}`,
    /// in which every term is at most `alpha_mu e^{-alpha_mu t} <= 1 / (e t)` (Document 6,
    /// Section 8). Zero at `t = 0` without a source.
    pub flux_scale: f64,
}

impl ModalState {
    /// `M^0 + M_gen(t)` (mol).
    pub fn total_input(&self) -> f64 {
        self.initial_inventory + self.generated
    }

    /// `M + M_rel + M_dec - (M^0 + M_gen)` (mol): zero up to rounding, because the three
    /// terms are built from one remaining fraction (an identity of the evaluation, derived in
    /// the document).
    pub fn balance_residual(&self) -> f64 {
        self.inventory + self.released + self.decayed - self.total_input()
    }

    /// The error estimate of Document 6, Section 8 for a series whose `accuracy_floor()` is
    /// `floor`: the computed model bounds (52), (53), plus the computed rounding bound of the
    /// evaluation, plus the floor terms (54) to (56) for the error of the series.
    pub fn estimated_errors(&self, floor: f64) -> EstimatedErrors {
        EstimatedErrors {
            inventory: self.inventory_error_bound
                + self.inventory_rounding_bound
                + EXT_FACTOR_INVENTORY * floor * self.age_input,
            released: self.release_error_bound
                + self.release_rounding_bound
                + EXT_FACTOR_RELEASE * floor * self.age_input,
            flux: self
                .flux_error_bound
                .map(|b| b + self.flux_rounding_bound + EXT_FACTOR_FLUX * floor * self.flux_scale),
        }
    }
}

/// The estimated absolute errors of [`ModalState::estimated_errors`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EstimatedErrors {
    /// `e_ext` of the inventory `M` (mol).
    pub inventory: f64,
    /// `e_ext` of the released inventory `M_rel` (mol).
    pub released: f64,
    /// `e_ext` of the release rate `Phi` (mol s^-1); `None` before `t_v`.
    pub flux: Option<f64>,
}

/// Why a value is not drawn (the drawing mask of science design Section 4.2).
#[derive(Debug, Clone, PartialEq)]
pub enum Withheld {
    /// Item 1: `t < min_valid_time()` (for `F_post` also `t_e`).
    BeforeMinValidTime,
    /// Item 2: the series with the eigenvalue cover reduced by a factor 4 differs by more
    /// than 5 % of the value.
    CoverSensitivity {
        /// Value with the full cover.
        full: f64,
        /// Value with the cover reduced by a factor 4.
        reduced: f64,
    },
    /// Item 3 for the existing series (no source, no decay): the floor exceeds 5 % of the value
    /// (as a fraction of the input).
    BelowAccuracyFloor {
        /// The value expressed as a fraction (of `M^0`).
        value: f64,
        /// `SERIES_FLOOR_FACTOR * accuracy_floor()` of the series.
        floor: f64,
    },
    /// Item 3 for a run with decay or a source (Document 6, Section 8): the estimated relative
    /// error of the value exceeds 5 %.
    ExtensionErrorTooLarge {
        /// Estimated relative error (the bound of Section 8 over the value).
        relative_error: f64,
    },
    /// The run lies outside the regime in which the error estimate was measured
    /// (`REGIME_*`); the text says which limit.
    OutsideEstimatedRegime(&'static str),
    /// The quantity is not defined here (for example `F` with `M^0 = 0`).
    Undefined(&'static str),
    /// The run has decay or a source, and the accuracy floor has not been re-estimated for
    /// (B2) to (B5). [`EXTENSION_FLOOR_REESTIMATED`] is `true`, so [`KernelRun::drawn`] never
    /// returns this; the gate is kept and tested through `drawn_gated` with the flag closed.
    FloorNotReestimated,
}

/// A value that is either drawn or withheld with its reason.
#[derive(Debug, Clone, PartialEq)]
pub enum Drawn {
    /// The value passes the mask.
    Value(f64),
    /// No value is returned.
    Withheld(Withheld),
}

impl Drawn {
    /// The value if it is drawn.
    pub fn value(&self) -> Option<f64> {
        match self {
            Self::Value(v) => Some(*v),
            Self::Withheld(_) => None,
        }
    }
}

/// Unit roundoff `u = 2^-53` of `f64` (round to nearest): every elementary operation
/// `fl(a op b)` is `(a op b)(1 + d)` with `|d| <= u`.
const UNIT_ROUNDOFF: f64 = f64::EPSILON / 2.0;

/// ASSUMED relative error of `exp` and `exp_m1` of the platform's math library: one unit in
/// the last place, `2 u` (the standard accuracy of a good library; not proved here).
const LIBM_REL_ERROR: f64 = 2.0 * UNIT_ROUNDOFF;

/// A value with a running bound on the rounding error accumulated in computing it (running
/// error analysis, Document 6 Section 8.2): `|computed - exact| <= e`, for exact inputs, the
/// standard model of floating-point arithmetic and the assumed accuracy of `exp`. The error of
/// the series itself (eigenvalues and weights, the accuracy floor) is NOT in `e`.
#[derive(Debug, Clone, Copy)]
struct Ev {
    v: f64,
    e: f64,
}

/// An exact input.
fn ev(v: f64) -> Ev {
    Ev { v, e: 0.0 }
}

impl Ev {
    /// A difference of two exact numbers computed in floating point: error `u |v|`.
    fn rounded(v: f64) -> Self {
        Self {
            v,
            e: UNIT_ROUNDOFF * v.abs(),
        }
    }

    fn exp(self) -> Self {
        let v = self.v.exp();
        Self {
            v,
            e: v * (self.e.exp_m1() * (1.0 + LIBM_REL_ERROR) + LIBM_REL_ERROR),
        }
    }

    fn exp_m1(self) -> Self {
        let v = self.v.exp_m1();
        let propagated = self.e * (self.v + self.e).exp();
        Self {
            v,
            e: propagated * (1.0 + LIBM_REL_ERROR) + LIBM_REL_ERROR * v.abs(),
        }
    }
}

impl std::ops::Neg for Ev {
    type Output = Ev;
    fn neg(self) -> Ev {
        Ev {
            v: -self.v,
            e: self.e,
        }
    }
}

impl std::ops::Add for Ev {
    type Output = Ev;
    fn add(self, o: Ev) -> Ev {
        let v = self.v + o.v;
        Ev {
            v,
            e: self.e + o.e + UNIT_ROUNDOFF * v.abs(),
        }
    }
}

impl std::ops::Sub for Ev {
    type Output = Ev;
    fn sub(self, o: Ev) -> Ev {
        let v = self.v - o.v;
        Ev {
            v,
            e: self.e + o.e + UNIT_ROUNDOFF * v.abs(),
        }
    }
}

impl std::ops::Mul for Ev {
    type Output = Ev;
    fn mul(self, o: Ev) -> Ev {
        let v = self.v * o.v;
        Ev {
            v,
            e: self.v.abs() * o.e + o.v.abs() * self.e + self.e * o.e + UNIT_ROUNDOFF * v.abs(),
        }
    }
}

impl std::ops::Div for Ev {
    type Output = Ev;
    fn div(self, o: Ev) -> Ev {
        let v = self.v / o.v;
        let den = o.v.abs() - o.e;
        let e = if den > 0.0 {
            (self.e + v.abs() * o.e) / den + UNIT_ROUNDOFF * v.abs()
        } else {
            f64::INFINITY
        };
        Ev { v, e }
    }
}

/// Error-free transformation of a sum (Knuth): `a + b = s + q` exactly.
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let z = s - a;
    (s, (a - (s - z)) + (b - z))
}

/// The sum of `terms` by compensated summation (Ogita, Rump and Oishi, "Sum2"): the computed
/// sum `r` of the computed terms obeys `|r - sum| <= u |sum| + gamma_{n-1}^2 sum |term|` with
/// `gamma_k = k u / (1 - k u)`, a bound that holds for any term order and carries no factor
/// `n` in front of `u`. The error of each term (its own `e`) is added.
fn sum_terms(terms: impl Iterator<Item = Ev>) -> Ev {
    let (mut s, mut c, mut e, mut abs, mut n) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0usize);
    for t in terms {
        let (ns, q) = two_sum(s, t.v);
        s = ns;
        c += q;
        e += t.e;
        abs += t.v.abs();
        n += 1;
    }
    let r = s + c;
    let g = n.saturating_sub(1) as f64 * UNIT_ROUNDOFF;
    let gamma = g / (1.0 - g);
    Ev {
        v: r,
        e: e + UNIT_ROUNDOFF * r.abs() * (1.0 + 2.0 * UNIT_ROUNDOFF) + gamma * gamma * abs,
    }
}

/// `int_0^x exp(-lambda s) ds = (1 - exp(-lambda x)) / lambda`, `x` for `lambda = 0`.
fn len_exp(lambda: f64, x: Ev) -> Ev {
    if lambda == 0.0 {
        x
    } else {
        -((ev(-lambda) * x).exp_m1()) / ev(lambda)
    }
}

/// `int_0^x len_exp(lambda, s) ds = (lambda x + expm1(-lambda x)) / lambda^2`; a series for
/// small `lambda x` (cancellation), `x^2 / 2` for `lambda = 0`.
fn len_exp_integral(lambda: f64, x: Ev) -> Ev {
    let z = ev(lambda) * x;
    if z.v < 0.1 {
        // x^2 sum_n (-z)^n / (n + 2)!, nine terms: the omitted term is below 3e-17 x^2, which
        // is added to the error bound
        let (mut term, mut sum) = (ev(0.5), ev(0.5));
        for n in 1..9 {
            term = term * (-z) / ev(n as f64 + 2.0);
            sum = sum + term;
        }
        let mut r = x * x * sum;
        r.e += 3.0e-17 * x.v * x.v;
        r
    } else {
        (z + (-z).exp_m1()) / (ev(lambda) * ev(lambda))
    }
}

/// The held modes with the factors `exp(-a_mu t_v)`: the building blocks of the evaluation.
/// Every function takes a time `th` with its rounding error and returns the value with a
/// running bound on its rounding error.
struct Kernel<'a> {
    alpha: &'a [f64],
    weight: &'a [f64],
    /// `a_mu = alpha_mu + lambda`.
    a: Vec<Ev>,
    /// `c_mu = w_mu exp(-a_mu t_v)`.
    c: Vec<Ev>,
    lambda: f64,
    t_v: f64,
    /// `int_0^{t_v} exp(-lambda s) ds`.
    e_v: Ev,
    /// `int_0^{t_v} E(s) ds`.
    q_v: Ev,
}

impl<'a> Kernel<'a> {
    fn new(alpha: &'a [f64], weight: &'a [f64], lambda: f64, t_v: f64) -> Self {
        let a: Vec<Ev> = alpha.iter().map(|&x| ev(x) + ev(lambda)).collect();
        let c: Vec<Ev> = weight
            .iter()
            .zip(&a)
            .map(|(&w, &a)| ev(w) * (-(a * ev(t_v))).exp())
            .collect();
        Self {
            alpha,
            weight,
            a,
            c,
            lambda,
            t_v,
            e_v: len_exp(lambda, ev(t_v)),
            q_v: len_exp_integral(lambda, ev(t_v)),
        }
    }

    /// `exp(-lambda theta) R^(theta)`: the model remaining fraction with decay.
    fn remaining(&self, th: Ev) -> Ev {
        if th.v < self.t_v {
            (ev(-self.lambda) * th).exp()
        } else {
            sum_terms(
                self.weight
                    .iter()
                    .zip(&self.a)
                    .map(|(&w, &a)| ev(w) * (-(a * th)).exp()),
            )
        }
    }

    /// `E(theta) = int_0^theta exp(-lambda s) R^(s) ds`.
    fn e0(&self, th: Ev) -> Ev {
        if th.v < self.t_v {
            len_exp(self.lambda, th)
        } else {
            let l = th - ev(self.t_v);
            self.e_v
                + sum_terms(
                    self.c
                        .iter()
                        .zip(&self.a)
                        .map(|(&c, &a)| c * (-((-(a * l)).exp_m1())) / a),
                )
        }
    }

    /// `Q(theta) = int_0^theta E(s) ds`.
    fn q0(&self, th: Ev) -> Ev {
        if th.v < self.t_v {
            len_exp_integral(self.lambda, th)
        } else {
            let l = th - ev(self.t_v);
            self.q_v
                + self.e_v * l
                + sum_terms(
                    self.c
                        .iter()
                        .zip(&self.a)
                        .map(|(&c, &a)| c / a * (l + (-(a * l)).exp_m1() / a)),
                )
        }
    }

    /// `F^_lambda(theta) = 1 - exp(-lambda theta) R^(theta) - lambda E(theta)`: the released
    /// fraction of a unit kernel-step inventory (0 before `t_v`).
    fn release(&self, th: Ev) -> Ev {
        if th.v < self.t_v {
            ev(0.0)
        } else {
            let s = sum_terms(
                self.weight
                    .iter()
                    .zip(self.alpha)
                    .map(|(&w, &al)| ev(w) * (-(ev(al) * th)).exp()),
            );
            // 1 - exp(-lambda theta) s = (1 - exp(-lambda theta)) + exp(-lambda theta) (1 - s)
            let lt = ev(-self.lambda) * th;
            -(lt.exp_m1()) + lt.exp() * (ev(1.0) - s) - ev(self.lambda) * self.e0(th)
        }
    }

    /// `int_0^theta F^_lambda(s) ds`.
    fn release_integral(&self, th: Ev) -> Ev {
        if th.v < self.t_v {
            ev(0.0)
        } else {
            (th - ev(self.t_v))
                - (self.e0(th) - self.e_v)
                - ev(self.lambda) * (self.q0(th) - self.q_v)
        }
    }

    /// `dF^_lambda/dtheta = exp(-lambda theta) sum_mu w_mu alpha_mu exp(-alpha_mu theta)`
    /// (0 before `t_v`).
    fn release_rate(&self, th: Ev) -> Ev {
        if th.v < self.t_v {
            ev(0.0)
        } else {
            sum_terms(
                self.weight
                    .iter()
                    .zip(self.alpha)
                    .zip(&self.a)
                    .map(|((&w, &al), &a)| ev(w) * ev(al) * (-(a * th)).exp()),
            )
        }
    }
}

/// A kernel-step initial inventory with an optional piecewise-constant kernel source and
/// radioactive decay, evaluated on the modes of a [`ModalRelease`].
#[derive(Debug, Clone)]
pub struct KernelRun<'a> {
    series: &'a ModalRelease,
    m0: f64,
    v_k: f64,
    source: PiecewiseSource,
    lambda: f64,
    /// Kernel radius and outer radius (m), when declared.
    geometry: Option<(f64, f64)>,
}

impl<'a> KernelRun<'a> {
    /// `m0` (mol) is the initial kernel inventory, `v_k` (m^3) the kernel volume. Refuses an
    /// invalid series, input or decay (see [`Decay::constant`]).
    pub fn new(
        series: &'a ModalRelease,
        m0: f64,
        v_k: f64,
        source: PiecewiseSource,
        decay: &Decay,
    ) -> Result<Self, ExtensionError> {
        if series.eigenvalues.is_empty() {
            return Err(ExtensionError::NoModes);
        }
        if series.eigenvalues.len() != series.weights.len() {
            return Err(ExtensionError::InconsistentSeries);
        }
        non_negative("initial inventory", m0)?;
        positive("kernel volume", v_k)?;
        let lambda = decay.constant()?;
        Ok(Self {
            series,
            m0,
            v_k,
            source,
            lambda,
            geometry: None,
        })
    }

    /// Declare the geometry of the series: the kernel radius and the outer radius (m). The
    /// extension mask draws a value only for a declared geometry inside the measured range
    /// ([`REGIME_OUTER_RADIUS`], [`REGIME_KERNEL_FRACTION`]); the error estimate was measured
    /// on those geometries only (Document 6, Section 8.6).
    pub fn with_geometry(mut self, kernel_radius: f64, outer_radius: f64) -> Self {
        self.geometry = Some((kernel_radius, outer_radius));
        self
    }

    /// The decay constant in use (s^-1).
    pub fn lambda(&self) -> f64 {
        self.lambda
    }

    fn no_source_no_decay(&self) -> bool {
        self.lambda == 0.0 && self.source.is_zero()
    }

    fn mode_count(&self, cap: Option<f64>) -> Result<usize, ExtensionError> {
        let n = match cap {
            None => self.series.eigenvalues.len(),
            Some(c) => self.series.eigenvalues.partition_point(|&l| l <= c),
        };
        if n == 0 {
            Err(ExtensionError::NoModes)
        } else {
            Ok(n)
        }
    }

    /// The state at time `t` (s) with all modes of the series. UNMASKED, NOT FOR DISPLAY: this
    /// is the raw evaluation of (42) to (49). Before `min_valid_time()` it holds the convention
    /// `R^ = 1`, and near it the values with a source or decay can be wrong by up to 100 %.
    /// Use [`KernelRun::drawn`] for anything that is displayed.
    pub fn state(&self, t: f64) -> Result<ModalState, ExtensionError> {
        self.state_cap(t, None)
    }

    /// As [`KernelRun::state`] with only the modes `alpha_mu <= cap` (s^-1), for the cover
    /// reduced by a factor 4 of the mask; `None` takes all modes.
    pub fn state_cap(&self, t: f64, cap: Option<f64>) -> Result<ModalState, ExtensionError> {
        non_negative("time", t)?;
        let n = self.mode_count(cap)?;
        let s = self.series;
        let cover = cap.map_or(s.lambda_covered, |c| c.min(s.lambda_covered));
        let t_v = RELAX_EFOLDS / cover;
        let lam = self.lambda;
        let alpha = &s.eigenvalues[..n];
        let weight = &s.weights[..n];
        let k = Kernel::new(alpha, weight, lam, t_v);

        let weight_sum: f64 = weight.iter().sum();
        let remaining_at_tv: f64 = alpha
            .iter()
            .zip(weight)
            .map(|(al, w)| w * (-al * t_v).exp())
            .sum();
        let delta =
            (1.0 - remaining_at_tv).max(0.0) + EARLY_RELEASE_FLOOR_FACTOR * s.accuracy_floor();

        let m0 = self.m0;
        let v = self.v_k;
        let t_ev = ev(t);
        let mut inventory = ev(m0) * k.remaining(t_ev);
        let mut decay_integral = ev(m0) * k.e0(t_ev);
        let mut released = ev(m0) * k.release(t_ev);
        let mut flux = ev(m0) * k.release_rate(t_ev);
        let mut sum_g = 0.0; // sum of G_k over the pieces started before t
        let mut age_input = m0;
        let mut flux_b = 0.0;
        // `a_hat(theta) Delta` bounds `|F_hat_lambda - F_lambda|` at `theta` (Document 6, (53)),
        // which is exactly 0 at `theta = 0`.
        let a_hat = |theta: f64| {
            if theta <= 0.0 {
                0.0
            } else {
                (if theta < t_v { 1.0 } else { 0.0 }) + lam * theta.min(t_v)
            }
        };
        let mut inv_b = if t < t_v { m0 * delta } else { 0.0 };
        let mut rel_b = m0 * delta * (if t < t_v { 1.0 } else { 0.0 } + lam * t.min(t_v));
        for j in 0..self.source.rates.len() {
            let g = self.source.rates[j];
            let sj = self.source.breaks[j];
            if sj >= t {
                continue;
            }
            let ej = t.min(self.source.breaks[j + 1]);
            let (th_hi, th_lo) = (t - sj, t - ej);
            // the times since the start and the end of the piece, rounded differences
            let (eh, el) = (Ev::rounded(th_hi), Ev::rounded(th_lo));
            let vg = ev(v) * ev(g);
            inventory = inventory + vg * (k.e0(eh) - k.e0(el));
            decay_integral = decay_integral + vg * (k.q0(eh) - k.q0(el));
            released = released + vg * (k.release_integral(eh) - k.release_integral(el));
            flux = flux + vg * (k.release(eh) - k.release(el));
            if g > 0.0 {
                sum_g += g;
                age_input += v * g * (t - sj);
                flux_b += delta * v * g * a_hat(th_hi).max(a_hat(th_lo));
                let len = ej - sj;
                let overlap = ((th_hi).min(t_v) - th_lo).max(0.0);
                inv_b += delta * v * g * overlap;
                rel_b += delta * v * g * (overlap + lam * t_v * len);
            }
        }
        let generated = v * self.source.generated_per_volume(t);
        // The decay loss: lambda int_0^t M ds, from the same remaining fraction.
        let mut decayed = lam * decay_integral.v;
        let mut released_out = released.v;
        let mut inventory_out = inventory.v;
        // The rounding bounds belong to the evaluation of the extension formulas; the existing
        // series (below) is judged by the floor test of item 3 and carries none.
        let mut rounding = [inventory.e, released.e, flux.e];
        if self.no_source_no_decay() {
            // The existing series term by term: F = 1 - sum w exp(-alpha t).
            let remaining: f64 = alpha
                .iter()
                .zip(weight)
                .map(|(l, w)| w * (-l * t).exp())
                .sum();
            released_out = m0 * (1.0 - remaining);
            // The same remaining fraction in the inventory, so that the balance closes also
            // for `t < t_v`, where `R^` of the evaluation is replaced by 1.
            inventory_out = m0 * remaining;
            decayed = 0.0;
            rounding = [0.0; 3];
        }
        Ok(ModalState {
            time: t,
            inventory: inventory_out,
            released: released_out,
            decayed,
            generated,
            initial_inventory: m0,
            flux: flux.v,
            source_rate: self.source.rate_at(t),
            weight_sum,
            modes: n,
            relax_time: t_v,
            early_release: delta,
            inventory_error_bound: inv_b,
            release_error_bound: rel_b,
            flux_error_bound: (t >= t_v).then_some(flux_b),
            inventory_rounding_bound: rounding[0],
            release_rounding_bound: rounding[1],
            flux_rounding_bound: rounding[2],
            age_input,
            flux_scale: v * sum_g + if t > 0.0 { m0 / t } else { 0.0 },
        })
    }

    /// The value of `q` at `t` with all modes, or `None` where the quantity is not defined.
    /// UNMASKED, NOT FOR DISPLAY (see [`KernelRun::state`]); display only [`KernelRun::drawn`].
    pub fn value(&self, q: Quantity, t: f64) -> Result<Option<f64>, ExtensionError> {
        self.value_cap(q, t, None)
    }

    /// The value of `q` at `t` with the modes `alpha_mu <= cap` only.
    pub fn value_cap(
        &self,
        q: Quantity,
        t: f64,
        cap: Option<f64>,
    ) -> Result<Option<f64>, ExtensionError> {
        let st = self.state_cap(t, cap)?;
        Ok(match q {
            Quantity::InventoryFraction => (self.m0 > 0.0).then(|| st.inventory / self.m0),
            Quantity::ReleaseFraction => {
                if self.m0 > 0.0 {
                    if self.no_source_no_decay() && cap.is_none() {
                        // bit for bit the existing `release_fraction`
                        Some(self.series.release_fraction(t))
                    } else {
                        Some(st.released / self.m0)
                    }
                } else {
                    None
                }
            }
            Quantity::ReleaseFractionGenerated => {
                let total = st.total_input();
                (total > 0.0).then(|| st.released / total)
            }
            Quantity::PostEventRelease { t_event } => {
                non_negative("event time", t_event)?;
                if t < t_event {
                    return Ok(None);
                }
                let ev = self.state_cap(t_event, cap)?;
                (ev.inventory > 0.0).then(|| (st.released - ev.released) / ev.inventory)
            }
            Quantity::Beta => (st.source_rate > 0.0).then(|| st.flux / (self.v_k * st.source_rate)),
        })
    }

    /// The value of `q` at `t`, or the reason it is not drawn: the drawing mask of science
    /// design Section 4.2 (items 1 to 3), applied a posteriori. No value is returned where
    /// the series is not trustworthy. Item 3 for a run with decay or a non-zero source is the
    /// rule of Document 6, Section 8 (estimated relative error at most 5 %, inside the
    /// measured regime); for the existing series it is the floor test of check (v).
    pub fn drawn(&self, q: Quantity, t: f64) -> Result<Drawn, ExtensionError> {
        self.drawn_gated(q, t, EXTENSION_FLOOR_REESTIMATED)
    }

    /// [`KernelRun::drawn`] with the floor re-estimation gate as an argument: with
    /// `reestimated = false` a run with decay or a non-zero source returns
    /// [`Withheld::FloorNotReestimated`]. Kept so that the gate stays tested.
    pub(crate) fn drawn_gated(
        &self,
        q: Quantity,
        t: f64,
        reestimated: bool,
    ) -> Result<Drawn, ExtensionError> {
        non_negative("time", t)?;
        if !self.no_source_no_decay() && !reestimated {
            return Ok(Drawn::Withheld(Withheld::FloorNotReestimated));
        }
        self.mask(q, t)
    }

    /// The first limit of the measured regime that the run exceeds, or `None`.
    fn outside_regime(&self, t: f64) -> Option<&'static str> {
        let s = self.series;
        let n = s.eigenvalues.len();
        if n < REGIME_MODES.0 || n > REGIME_MODES.1 {
            return Some("number of modes outside the measured range");
        }
        if s.accuracy_floor() > REGIME_FLOOR_MAX {
            return Some("accuracy floor above the largest measured");
        }
        let alpha_1 = s.eigenvalues[0];
        if self.lambda > REGIME_LAMBDA_OVER_ALPHA1_MAX * alpha_1 {
            return Some("decay constant above the largest measured multiple of the slowest mode");
        }
        if alpha_1 * t > REGIME_ALPHA1_T_MAX {
            return Some("time above the largest measured multiple of the slowest relaxation time");
        }
        if self.source.rates().len() > REGIME_PIECES_MAX {
            return Some("more source pieces than measured");
        }
        match self.geometry {
            None => return Some("geometry not declared"),
            Some((r_k, r_o)) => {
                // a relative slack of 1e-9 so that the end points themselves (0.4 = 160 / 400)
                // are inside whatever the rounding of the division
                let in_range = |x: f64, (lo, hi): (f64, f64)| {
                    x >= lo * (1.0 - 1.0e-9) && x <= hi * (1.0 + 1.0e-9)
                };
                if !(r_k > 0.0 && r_o.is_finite() && in_range(r_o, REGIME_OUTER_RADIUS)) {
                    return Some("outer radius outside the measured range");
                }
                if !in_range(r_k / r_o, REGIME_KERNEL_FRACTION) {
                    return Some("kernel radius fraction outside the measured range");
                }
            }
        }
        None
    }

    /// Estimated relative error of the value of `q` at `t` (Document 6, Section 8): the sum of
    /// the estimated absolute errors (`ModalState::estimated_errors`) of the quantities that
    /// enter the value over the value itself. `None` where it is not defined (a zero or
    /// negative denominator).
    fn estimated_relative_error(&self, q: Quantity, t: f64) -> Result<Option<f64>, ExtensionError> {
        let floor = self.series.accuracy_floor();
        let st = self.state(t)?;
        let e = st.estimated_errors(floor);
        Ok(match q {
            Quantity::InventoryFraction => (st.inventory > 0.0).then(|| e.inventory / st.inventory),
            Quantity::ReleaseFraction | Quantity::ReleaseFractionGenerated => {
                (st.released > 0.0).then(|| e.released / st.released)
            }
            Quantity::PostEventRelease { t_event } => {
                let ev = self.state(t_event)?;
                let ee = ev.estimated_errors(floor);
                let gained = st.released - ev.released;
                (gained > 0.0 && ev.inventory > 0.0)
                    .then(|| (e.released + ee.released) / gained + ee.inventory / ev.inventory)
            }
            Quantity::Beta => match e.flux {
                Some(f) if st.flux > 0.0 => Some(f / st.flux),
                _ => None,
            },
        })
    }

    /// Items 1 to 3 of the drawing mask, without the floor re-estimation gate of
    /// [`KernelRun::drawn`].
    pub(crate) fn mask(&self, q: Quantity, t: f64) -> Result<Drawn, ExtensionError> {
        let s = self.series;
        non_negative("time", t)?;
        let valid_from = s.min_valid_time();
        // Item 1 (for F_post the event time too).
        let event_ok = match q {
            Quantity::PostEventRelease { t_event } => t_event >= valid_from,
            _ => true,
        };
        if t < valid_from || !event_ok {
            return Ok(Drawn::Withheld(Withheld::BeforeMinValidTime));
        }
        let full = match self.value(q, t)? {
            Some(v) => v,
            None => {
                return Ok(Drawn::Withheld(Withheld::Undefined(
                    "the quantity is not defined for this state",
                )))
            }
        };
        // Item 2: cover reduced by a factor 4.
        let reduced = match self.value_cap(q, t, Some(s.lambda_covered / 4.0)) {
            Ok(Some(v)) => v,
            Ok(None) | Err(ExtensionError::NoModes) => {
                return Ok(Drawn::Withheld(Withheld::Undefined(
                    "no mode remains in the reduced cover",
                )))
            }
            Err(e) => return Err(e),
        };
        if (full - reduced).abs() > MASK_RELATIVE_TOLERANCE * full.abs() {
            return Ok(Drawn::Withheld(Withheld::CoverSensitivity {
                full,
                reduced,
            }));
        }
        if !self.no_source_no_decay() {
            // Item 3 for a run with decay or a source: the estimated relative error.
            if let Some(why) = self.outside_regime(t) {
                return Ok(Drawn::Withheld(Withheld::OutsideEstimatedRegime(why)));
            }
            // For `F_post` the event time `t_e <= t` needs no regime check of its own: the
            // limits depend on the time only through `alpha_1 t`, which is largest at `t`.
            return match self.estimated_relative_error(q, t)? {
                Some(r) if r <= MASK_RELATIVE_TOLERANCE => Ok(Drawn::Value(full)),
                Some(r) => Ok(Drawn::Withheld(Withheld::ExtensionErrorTooLarge {
                    relative_error: r,
                })),
                None => Ok(Drawn::Withheld(Withheld::Undefined(
                    "the relative error is not defined for this value",
                ))),
            };
        }
        // Item 3 for the existing series, the value expressed as a fraction of the input.
        let fraction = match q {
            Quantity::InventoryFraction => None,
            Quantity::ReleaseFraction => {
                let st = self.state(t)?;
                Some(full * self.m0 / st.total_input())
            }
            Quantity::ReleaseFractionGenerated
            | Quantity::PostEventRelease { .. }
            | Quantity::Beta => Some(full),
        };
        if let Some(value) = fraction {
            let floor = SERIES_FLOOR_FACTOR * s.accuracy_floor();
            if value.is_nan() || floor > MASK_RELATIVE_TOLERANCE * value {
                return Ok(Drawn::Withheld(Withheld::BelowAccuracyFloor {
                    value,
                    floor,
                }));
            }
        }
        Ok(Drawn::Value(full))
    }
}

impl ModalRelease {
    /// State with decay and a piecewise-constant kernel source, formulas (42) to (49): the
    /// extension of the science design Section 4.2. `m0` (mol) initial kernel inventory,
    /// `v_k` (m^3) kernel volume, `source` the piecewise-constant rate (mol m^-3 s^-1),
    /// `decay` the decay constant (s^-1) which must be the same in every layer, `t` (s).
    /// Refuses with an error where a condition does not hold. UNMASKED, NOT FOR DISPLAY: the
    /// drawing mask is [`KernelRun::drawn`].
    pub fn with_source_decay(
        &self,
        m0: f64,
        v_k: f64,
        source: &PiecewiseSource,
        decay: &Decay,
        t: f64,
    ) -> Result<ModalState, ExtensionError> {
        KernelRun::new(self, m0, v_k, source.clone(), decay)?.state(t)
    }

    /// Decay extension, formula (B2): no source, kernel-step start, the same decay constant
    /// in every layer, constant diffusivities (those of the series). UNMASKED, NOT FOR
    /// DISPLAY (see [`KernelRun::state`]).
    pub fn with_decay(&self, m0: f64, decay: &Decay, t: f64) -> Result<ModalState, ExtensionError> {
        // v_k is irrelevant without a source; 1 m^3 keeps the units consistent.
        KernelRun::new(self, m0, 1.0, PiecewiseSource::none(), decay)?.state(t)
    }
}

#[cfg(test)]
mod tests {
    use super::super::homogeneous_sphere::HomogeneousSphere;
    use super::super::sturm_liouville::Layer;
    use super::*;
    use std::f64::consts::PI;

    const R1: f64 = 2.0e-4;
    const RADII: [f64; 5] = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4];
    const VK: f64 = 4.0 / 3.0 * PI * R1 * R1 * R1;

    fn layers(d: [f64; 5]) -> Vec<Layer> {
        RADII
            .iter()
            .zip(d)
            .map(|(&r, d)| Layer {
                outer_radius: r,
                diffusivity: d,
            })
            .collect()
    }

    fn equal_layers(d: f64) -> Vec<Layer> {
        layers([d; 5])
    }

    /// Five different diffusivities (m^2 s^-1), as in the existing tests.
    const D_FIVE: [f64; 5] = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13];

    fn rel(a: f64, b: f64) -> f64 {
        ((a - b) / b).abs()
    }

    /// Independent mode data of the homogeneous kernel step (closed form).
    fn analytic_series(d: f64, n_modes: usize, lambda_covered: f64) -> ModalRelease {
        let sphere = HomogeneousSphere::core(RADII[4], R1, d, 1.0).unwrap();
        let eig: Vec<f64> = (1..=n_modes).map(|n| sphere.eigenvalue(n)).collect();
        let w: Vec<f64> = (1..=n_modes).map(|n| sphere.weight(n)).collect();
        ModalRelease {
            lambda_covered,
            eigenvalues: eig,
            weights: w,
            min_flux_ratio: f64::INFINITY,
        }
    }

    /// Composite Simpson rule with an even number of intervals.
    fn simpson(a: f64, b: f64, n: usize, f: impl Fn(f64) -> f64) -> f64 {
        let h = (b - a) / n as f64;
        let mut s = f(a) + f(b);
        for i in 1..n {
            s += f(a + i as f64 * h) * if i % 2 == 1 { 4.0 } else { 2.0 };
        }
        s * h / 3.0
    }

    /// Decay-free release fraction of the kernel step, rigorous closed-form series.
    fn f0(sphere: &HomogeneousSphere, t: f64) -> f64 {
        if t <= 0.0 {
            return 0.0;
        }
        sphere.release_fraction(t, 1.0e-16).unwrap().value
    }

    /// Release fraction with decay from the decay-free closed form:
    /// F_lambda(theta) = exp(-lambda theta) F0(theta) + lambda int_0^theta exp(-lambda s) F0(s) ds
    /// (integration by parts of the released flux exp(-lambda s) dF0/ds, which follows from
    /// C = exp(-lambda t) C_decay-free); it uses no formula of the extension.
    fn f_lambda(sphere: &HomogeneousSphere, lambda: f64, theta: f64) -> f64 {
        if theta <= 0.0 {
            return 0.0;
        }
        let integral = simpson(0.0, theta, 200, |s| (-lambda * s).exp() * f0(sphere, s));
        (-lambda * theta).exp() * f0(sphere, theta) + lambda * integral
    }

    fn five_equal_series(d: f64, lambda_max: f64) -> ModalRelease {
        ModalRelease::new(&equal_layers(d), lambda_max, 20_000).unwrap()
    }

    #[test]
    fn check_i_zero_source_and_zero_decay_is_the_existing_series_bit_for_bit() {
        let s = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 20_000).unwrap();
        let none = KernelRun::new(&s, 2.5, VK, PiecewiseSource::none(), &Decay::None).unwrap();
        let zero_piece = KernelRun::new(
            &s,
            2.5,
            VK,
            PiecewiseSource::constant(0.0, 1.0e9).unwrap(),
            &Decay::Uniform(0.0),
        )
        .unwrap();
        for t in [5.0e3, 2.0e4, 1.0e5, 1.0e6] {
            let expected = s.release_fraction(t);
            assert_eq!(
                none.value(Quantity::ReleaseFraction, t).unwrap(),
                Some(expected)
            );
            assert_eq!(
                zero_piece.value(Quantity::ReleaseFraction, t).unwrap(),
                Some(expected)
            );
            assert_eq!(none.state(t).unwrap().released, 2.5 * expected);
            // the remaining inventory is the existing one to rounding
            let m = none.state(t).unwrap().inventory / 2.5;
            assert!((m - s.remaining_fraction(t)).abs() < 1e-14);
        }
    }

    #[test]
    fn limits_lambda_and_source_to_zero_reproduce_the_existing_series_to_rounding() {
        let s = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 20_000).unwrap();
        let floor = s.accuracy_floor();
        for lambda in [1.0e-30, 1.0e-24, 1.0e-20] {
            let run = KernelRun::new(
                &s,
                1.0,
                VK,
                PiecewiseSource::none(),
                &Decay::Uniform(lambda),
            )
            .unwrap();
            let src = KernelRun::new(
                &s,
                1.0,
                VK,
                PiecewiseSource::constant(lambda, 1.0e9).unwrap(),
                &Decay::None,
            )
            .unwrap();
            for t in [5.0e3, 2.0e4, 1.0e5, 1.0e6] {
                let f = s.release_fraction(t);
                for r in [&run, &src] {
                    let x = r.value(Quantity::ReleaseFraction, t).unwrap().unwrap();
                    assert!(
                        (x - f).abs() <= floor.max(1e-14),
                        "lambda {lambda} t {t}: {x} vs {f}"
                    );
                }
            }
        }
    }

    #[test]
    fn decay_extension_matches_an_independent_closed_form() {
        // Homogeneous kernel step (five equal diffusivities): the layered shooting series with
        // decay against the closed-form decay-free release F0 of `homogeneous_sphere`, with
        // the decay entering only through F_lambda (see `f_lambda`). The difference must lie
        // within the computed error bound plus the accuracy of the reference quadrature.
        let d = 1.0e-10;
        let s = five_equal_series(d, 45.0 / 20.0);
        let sphere = HomogeneousSphere::core(RADII[4], R1, d, 1.0).unwrap();
        let lambda = 5.0e-3;
        let run = KernelRun::new(
            &s,
            1.0,
            VK,
            PiecewiseSource::none(),
            &Decay::Uniform(lambda),
        )
        .unwrap();
        for t in [5.0, 20.0, 60.0, 150.0, 400.0] {
            let st = run.state(t).unwrap();
            let f_ref = f_lambda(&sphere, lambda, t);
            let err = (st.released - f_ref).abs();
            eprintln!(
                "B2 t {t}: {:.6e} ref {f_ref:.6e} err {err:.2e} bound {:.2e}",
                st.released, st.release_error_bound
            );
            assert!(err <= st.release_error_bound + 1e-12, "t {t}");
            // remaining inventory exp(-lambda t)(1 - F0), and the decay loss from the balance
            let m_ref = (-lambda * t).exp() * (1.0 - f0(&sphere, t));
            assert!(
                (st.inventory - m_ref).abs() <= st.inventory_error_bound + 1e-12,
                "t {t}"
            );
            let dec_ref = 1.0 - m_ref - f_ref;
            // M_dec = lambda int M ds carries the errors of M and M_rel, summed
            assert!(
                (st.decayed - dec_ref).abs()
                    <= st.release_error_bound + st.inventory_error_bound + 1e-11,
                "decayed t {t}"
            );
            assert!(st.balance_residual().abs() < 1e-14, "balance t {t}: {st:?}");
        }
    }

    #[test]
    fn decay_extension_matches_the_homogeneous_sphere_of_the_physics_module() {
        // Uniform start (kernel = whole sphere): weights 6 / (n pi)^2, rates n^2 pi^2 D / R^2.
        // The physics reference truncates its series at N terms with the tail bounded by
        // 6 / (pi^2 N); the difference must stay within that bound.
        use crate::physics::reference::HomogeneousSphere as Reference;
        let (r, d, lambda) = (1.0, 1.0, 0.7);
        let n = 100_000usize;
        let eig: Vec<f64> = (1..=n)
            .map(|k| (k as f64 * PI).powi(2) * d / (r * r))
            .collect();
        let w: Vec<f64> = (1..=n).map(|k| 6.0 / (k as f64 * PI).powi(2)).collect();
        let series = ModalRelease {
            lambda_covered: eig[n - 1],
            eigenvalues: eig,
            weights: w,
            min_flux_ratio: f64::INFINITY,
        };
        let reference = Reference::new(r, d, lambda).unwrap();
        let bound = 6.0 / (PI * PI * n as f64);
        for t in [0.01, 0.05, 0.2, 0.5] {
            let st = series.with_decay(1.0, &Decay::Uniform(lambda), t).unwrap();
            let rel_ref = reference.released_fraction(t, n).unwrap();
            let dec_ref = reference.decayed_fraction(t, n).unwrap();
            let ret_ref = reference.retained_fraction(t, n).unwrap();
            assert!((st.released - rel_ref).abs() <= bound, "released t {t}");
            assert!((st.decayed - dec_ref).abs() <= bound, "decayed t {t}");
            assert!((st.inventory - ret_ref).abs() <= bound, "retained t {t}");
        }
    }

    #[test]
    fn source_extension_matches_the_superposition_of_the_independent_closed_form() {
        // M_rel(t) = v_k sum_k G_k int_{s_k}^{min(t, s_{k+1})} F_lambda(t - tau) dtau
        // (each source increment released as a kernel step), with F_lambda from the
        // decay-free closed form; M(t) likewise from exp(-lambda theta)(1 - F0).
        let d = 1.0e-10;
        let s = five_equal_series(d, 45.0 / 20.0);
        let sphere = HomogeneousSphere::core(RADII[4], R1, d, 1.0).unwrap();
        let lambda = 2.0e-3;
        let (g0, g1) = (3.0e-3, 1.0e-3);
        let (s1, s2) = (100.0, 250.0);
        let source = PiecewiseSource::new(vec![0.0, s1, s2], vec![g0, g1]).unwrap();
        let run = KernelRun::new(&s, 0.0, VK, source, &Decay::Uniform(lambda)).unwrap();
        for t in [30.0, 150.0, 300.0, 500.0] {
            let st = run.state(t).unwrap();
            let pieces = [(0.0, s1, g0), (s1, s2, g1)];
            let (mut rel_ref, mut inv_ref) = (0.0, 0.0);
            for (a, b, g) in pieces {
                let e = t.min(b);
                if a >= t {
                    continue;
                }
                rel_ref += VK * g * simpson(a, e, 40, |tau| f_lambda(&sphere, lambda, t - tau));
                inv_ref += VK
                    * g
                    * simpson(a, e, 40, |tau| {
                        (-lambda * (t - tau)).exp() * (1.0 - f0(&sphere, t - tau))
                    });
            }
            let scale = st.generated;
            let quad = 1e-8 * scale; // accuracy of the nested reference quadrature
            assert!(
                (st.released - rel_ref).abs() <= st.release_error_bound + quad,
                "released t {t}: {} vs {rel_ref}",
                st.released
            );
            assert!(
                (st.inventory - inv_ref).abs() <= st.inventory_error_bound + quad,
                "inventory t {t}"
            );
            assert!(
                st.balance_residual().abs() <= 1e-13 * scale,
                "balance t {t}: {st:?}"
            );
        }
    }

    #[test]
    fn superposition_over_two_source_segments_equals_one_run() {
        let s = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 20_000).unwrap();
        let (m0, g0, g1) = (1.5e-9, 4.0e-6, 9.0e-6);
        let (s1, s2) = (3.0e4, 8.0e4);
        let lam = Decay::Uniform(7.0e-6);
        let mk = |m, rates: Vec<f64>| {
            KernelRun::new(
                &s,
                m,
                VK,
                PiecewiseSource::new(vec![0.0, s1, s2], rates).unwrap(),
                &lam,
            )
            .unwrap()
        };
        let both = mk(m0, vec![g0, g1]);
        let first = mk(m0, vec![g0, 0.0]);
        let second = mk(0.0, vec![0.0, g1]);
        for t in [2.0e4, 5.0e4, 8.0e4, 3.0e5] {
            let (a, b, c) = (
                both.state(t).unwrap(),
                first.state(t).unwrap(),
                second.state(t).unwrap(),
            );
            for (x, y, name) in [
                (a.inventory, b.inventory + c.inventory, "inventory"),
                (a.released, b.released + c.released, "released"),
                (a.decayed, b.decayed + c.decayed, "decayed"),
                (a.generated, b.generated + c.generated, "generated"),
                (a.flux, b.flux + c.flux, "flux"),
            ] {
                assert!(
                    (x - y).abs() <= 1e-12 * x.abs().max(1e-30),
                    "{name} t {t}: {x} vs {y}"
                );
            }
        }
        // two equal adjacent segments equal one segment
        let split = mk(m0, vec![g0, g0]);
        let single = KernelRun::new(
            &s,
            m0,
            VK,
            PiecewiseSource::new(vec![0.0, s2], vec![g0]).unwrap(),
            &lam,
        )
        .unwrap();
        for t in [2.0e4, 5.0e4, 3.0e5] {
            let (a, b) = (split.state(t).unwrap(), single.state(t).unwrap());
            assert!((a.inventory - b.inventory).abs() <= 1e-12 * a.inventory);
            assert!((a.released - b.released).abs() <= 1e-12 * a.released);
        }
    }

    #[test]
    fn generated_release_fraction_matches_an_independent_m_rel_over_the_input() {
        // F_gen = M_rel / (M^0 + M_gen) with M_rel from the Duhamel superposition of the
        // independent closed form (decay-free, lambda = 0), for M^0 = 0 and for M^0 > 0.
        let d = 1.0e-10;
        let s = five_equal_series(d, 45.0 / 20.0);
        let sphere = HomogeneousSphere::core(RADII[4], R1, d, 1.0).unwrap();
        let (g, stop) = (2.0e-3, 300.0);
        for m0 in [0.0, 0.7 * VK] {
            let source = PiecewiseSource::constant(g, stop).unwrap();
            let run = KernelRun::new(&s, m0, VK, source, &Decay::None).unwrap();
            for t in [200.0, 400.0, 900.0] {
                let st = run.state(t).unwrap();
                let rel_ref = m0 * f0(&sphere, t)
                    + VK * g * simpson(0.0, t.min(stop), 60, |tau| f0(&sphere, t - tau));
                let total = m0 + VK * g * t.min(stop);
                let fgen = run
                    .value(Quantity::ReleaseFractionGenerated, t)
                    .unwrap()
                    .unwrap();
                let expected = rel_ref / total;
                assert!(
                    (fgen - expected).abs() <= st.release_error_bound / total + 1e-8,
                    "m0 {m0} t {t}: {fgen} vs {expected}"
                );
                assert!(
                    rel(fgen, expected) < 1e-6,
                    "m0 {m0} t {t}: {fgen} vs {expected}"
                );
            }
        }
    }

    #[test]
    fn flux_matches_a_finite_difference_inside_and_after_the_second_of_two_pieces() {
        // Two pieces, so that the flux of the finished first piece (its F^ at both limits) is
        // part of the value: Phi = d M_rel / dt by a central difference of the raw M_rel.
        let s = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 20_000).unwrap();
        let src = PiecewiseSource::new(vec![0.0, 2.0e5, 6.0e5], vec![3.0e-6, 8.0e-6]).unwrap();
        let run = KernelRun::new(&s, 2.0e-9, VK, src, &Decay::Uniform(5.0e-7)).unwrap();
        let h = 20.0;
        for t in [3.0e5, 5.0e5, 8.0e5, 1.5e6] {
            let phi = run.state(t).unwrap().flux;
            let fd = (run.state(t + h).unwrap().released - run.state(t - h).unwrap().released)
                / (2.0 * h);
            assert!(rel(phi, fd) < 1e-6, "t {t}: {phi} vs {fd}");
        }
    }

    #[test]
    fn inventory_and_release_are_one_evaluation_below_the_relaxation_time_without_source() {
        // For t < t_v without source and decay the inventory and the release are both built
        // from the existing series, so the balance closes.
        let s = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 20_000).unwrap();
        let run = KernelRun::new(&s, 2.0, VK, PiecewiseSource::none(), &Decay::None).unwrap();
        for t in [
            0.0,
            0.2 * s.min_valid_time(),
            0.9 * s.min_valid_time(),
            1.0e5,
        ] {
            let st = run.state(t).unwrap();
            assert!(st.balance_residual().abs() <= 1e-15 * 2.0, "t {t}: {st:?}");
            assert!(
                (st.inventory - 2.0 * (1.0 - s.release_fraction(t))).abs() <= 1e-14,
                "t {t}: {st:?}"
            );
        }
    }

    #[test]
    fn balance_identity_of_the_evaluation() {
        let s = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 20_000).unwrap();
        let run = KernelRun::new(
            &s,
            1.0e-9,
            VK,
            PiecewiseSource::new(vec![0.0, 4.0e4, 9.0e4], vec![8.0e-6, 2.0e-6]).unwrap(),
            &Decay::Uniform(7.3e-7),
        )
        .unwrap();
        for t in [1.0e4, 4.0e4, 7.0e4, 2.0e5, 1.0e6] {
            let st = run.state(t).unwrap();
            assert!(
                st.balance_residual().abs() < 1e-12 * st.total_input(),
                "t {t}"
            );
        }
    }

    /// Closed-form steady state of check (ii): D5 (14) integrated over the sphere,
    /// 4 pi int C_ss r^2 dr, with G_kernel only in the kernel. Units: mol.
    fn steady_inventory_d5_14(d: [f64; 5], g: f64) -> f64 {
        let mut c_outer = 0.0; // C at the outer radius of the current layer
        let mut total = 0.0;
        for i in (1..5).rev() {
            let (a, b) = (RADII[i - 1], RADII[i]);
            let gm = g * R1.powi(3) / (3.0 * d[i]);
            let c_b = c_outer;
            // int_a^b r^2 [c_b + gm (1/r - 1/b)] dr
            total += (c_b - gm / b) * (b.powi(3) - a.powi(3)) / 3.0 + gm * (b * b - a * a) / 2.0;
            c_outer = c_b + gm * (1.0 / a - 1.0 / b);
        }
        // kernel: C(r) = C(R1) + g (R1^2 - r^2) / (6 D_kernel)
        total += c_outer * R1.powi(3) / 3.0 + g * R1.powi(5) / (45.0 * d[0]);
        4.0 * PI * total
    }

    #[test]
    fn check_ii_steady_state_matches_d5_14() {
        let d = D_FIVE;
        let s = ModalRelease::new(&layers(d), 45.0 / 5.0e2, 3000).unwrap();
        let g = 6.0e-6;
        // long time: the slowest mode has decayed by exp(-80)
        let t = 80.0 / s.eigenvalues[0];
        let run = KernelRun::new(
            &s,
            0.0,
            VK,
            PiecewiseSource::constant(g, 2.0 * t).unwrap(),
            &Decay::None,
        )
        .unwrap();
        let st = run.state(t).unwrap();
        let reference = steady_inventory_d5_14(d, g);
        let sum_w_over_alpha: f64 = s
            .eigenvalues
            .iter()
            .zip(&s.weights)
            .map(|(a, w)| w / a)
            .sum();
        eprintln!(
            "steady state: modes {}, M {:.9e}, v_k G sum w/alpha {:.9e}, M_D5(14) {:.9e}, rel {:.3e}, missing weight {:.3e}, bound {:.2e}",
            s.eigenvalues.len(),
            st.inventory,
            VK * g * sum_w_over_alpha,
            reference,
            rel(st.inventory, reference),
            1.0 - st.weight_sum,
            st.inventory_error_bound
        );
        // The truncated modal sum v_k G sum w / alpha has a tail of order N^-3 (observed, not
        // proved); the evaluation here replaces it by the early-time closed part and agrees
        // with D5 (14) to the computed bound.
        assert!(rel(st.inventory, VK * g * sum_w_over_alpha) < 1e-6);
        assert!((st.inventory - reference).abs() <= st.inventory_error_bound + 1e-9 * reference);
        // Phi -> v_k G
        assert!(rel(st.flux, VK * g) <= 1e-9);
        assert!(rel(st.flux, VK * g) <= st.flux_error_bound.unwrap() / (VK * g) + 1e-12);
        // beta -> 1
        let beta = run.value(Quantity::Beta, t).unwrap().unwrap();
        assert!((beta - 1.0).abs() < 1e-9);
        // the held weights sum to nearly 1 (diagnostic only)
        assert!((1.0 - st.weight_sum).abs() < 5.0e-3);
    }

    /// Steady state with decay, one homogeneous material, kernel source only: solves
    /// D (C'' + 2 C'/r) - lambda C + G chi = 0, regular at 0, C(R_5) = 0 (u = r C):
    /// u = r G / lambda + A sinh(b r) inside, B sinh(b (R_5 - r)) outside, b = sqrt(lambda/D).
    fn steady_inventory_with_decay(d: f64, lambda: f64, g: f64) -> f64 {
        let b = (lambda / d).sqrt();
        let r5 = RADII[4];
        let l = r5 - R1;
        let (sh1, ch1) = ((b * R1).sinh(), (b * R1).cosh());
        let (shl, chl) = ((b * l).sinh(), (b * l).cosh());
        let p = g / lambda;
        let det = sh1 * b * chl + shl * b * ch1;
        // Cramer: [sh1, -shl; b ch1, b chl] [A; B] = [-R1 p; -p]
        let a_sol = ((-R1 * p) * (b * chl) - (-shl) * (-p)) / det;
        let b_sol = (sh1 * (-p) - (b * ch1) * (-R1 * p)) / det;
        let inside = p * R1.powi(3) / 3.0 + a_sol * (R1 * ch1 / b - sh1 / (b * b));
        let outside = b_sol * (r5 * (chl - 1.0) / b - (l * chl / b - shl / (b * b)));
        4.0 * PI * (inside + outside)
    }

    #[test]
    fn steady_state_with_decay_matches_the_closed_form() {
        let d = 1.0e-10;
        let lambda = 4.0e-3;
        let s = five_equal_series(d, 45.0 / 5.0e-1);
        let g = 5.0e-6;
        let t = 60.0 / lambda;
        let run = KernelRun::new(
            &s,
            0.0,
            VK,
            PiecewiseSource::constant(g, 2.0 * t).unwrap(),
            &Decay::Uniform(lambda),
        )
        .unwrap();
        let st = run.state(t).unwrap();
        let reference = steady_inventory_with_decay(d, lambda, g);
        eprintln!(
            "decay steady state: modes {}, M {:.9e}, closed {:.9e}, rel {:.3e}, bound {:.2e}",
            st.modes,
            st.inventory,
            reference,
            rel(st.inventory, reference),
            st.inventory_error_bound
        );
        assert!((st.inventory - reference).abs() <= st.inventory_error_bound + 1e-9 * reference);
        // generation = loss rate: Phi + lambda M = v_k G in the steady state
        let lhs = st.flux + lambda * st.inventory;
        assert!(rel(lhs, VK * g) < 1e-6);
    }

    #[test]
    fn check_iii_homogeneous_layers_reproduce_the_duhamel_row_and_the_decay_series() {
        // Shooting modes of five equal layers against the closed-form modes of
        // `homogeneous_sphere`, same formulas (D5 (21) and the Duhamel row of D5 4.5).
        let d = 1.0e-10;
        let s = five_equal_series(d, 45.0 / 20.0);
        let a = analytic_series(d, s.eigenvalues.len(), s.lambda_covered);
        for (e, r) in s.eigenvalues.iter().zip(&a.eigenvalues).take(40) {
            assert!(rel(*e, *r) < 1e-10);
        }
        let lambda = 3.0e-3;
        let source = PiecewiseSource::new(vec![0.0, 200.0, 700.0], vec![2.0e-3, 5.0e-3]).unwrap();
        for t in [30.0, 120.0, 400.0, 900.0] {
            let x = KernelRun::new(&s, 1.0, VK, source.clone(), &Decay::Uniform(lambda))
                .unwrap()
                .state(t)
                .unwrap();
            let y = KernelRun::new(&a, 1.0, VK, source.clone(), &Decay::Uniform(lambda))
                .unwrap()
                .state(t)
                .unwrap();
            assert!((x.inventory - y.inventory).abs() < 1e-9, "inventory t {t}");
            assert!((x.released - y.released).abs() < 1e-9, "released t {t}");
            assert!((x.decayed - y.decayed).abs() < 1e-9, "decayed t {t}");
        }
        // Duhamel row: M(t) = v_k G sum w (1 - exp(-alpha t)) / alpha for lambda = 0, m0 = 0,
        // summed over the closed-form modes (the tail beyond the held modes is the omitted
        // early part, below the computed bound)
        let g = 4.0e-3;
        let run = KernelRun::new(
            &s,
            0.0,
            VK,
            PiecewiseSource::constant(g, 1.0e5).unwrap(),
            &Decay::None,
        )
        .unwrap();
        // many closed-form modes: the truncated Duhamel sum has a tail of order N^-3 (observed)
        let a = analytic_series(d, 20_000, s.lambda_covered);
        for t in [50.0, 300.0, 1000.0] {
            let duhamel: f64 = a
                .eigenvalues
                .iter()
                .zip(&a.weights)
                .map(|(al, w)| w * (-(-al * t).exp_m1()) / al)
                .sum::<f64>()
                * VK
                * g;
            let st = run.state(t).unwrap();
            assert!(
                (st.inventory - duhamel).abs() <= st.inventory_error_bound + 1e-9 * duhamel,
                "t {t}"
            );
        }
    }

    #[test]
    fn check_v_floor_test_one_homogeneous_layer() {
        // Early times where the decay-free closed form lies between 1 and 100 times
        // accuracy_floor(): B2 (lambda > 0) and B5 (lambda > 0, constant G) against the
        // closed-form modes of check (iii); the absolute difference must not exceed the floor.
        let d = 1.0e-10;
        let s = five_equal_series(d, 45.0 / 2.0);
        let floor = s.accuracy_floor();
        let a = analytic_series(d, s.eigenvalues.len(), s.lambda_covered);
        let sphere = HomogeneousSphere::core(RADII[4], R1, d, 1.0).unwrap();
        let lambda = 2.0e-3;
        let g = 1.0e-3;
        eprintln!(
            "floor test: modes {}, accuracy_floor {floor:.3e}, min_valid_time {:.3e} s",
            s.eigenvalues.len(),
            s.min_valid_time()
        );
        let decay = Decay::Uniform(lambda);
        let src = || PiecewiseSource::constant(g, 1.0e4).unwrap();
        let run = KernelRun::new(&s, 1.0, VK, src(), &decay).unwrap();
        let run_a = KernelRun::new(&a, 1.0, VK, src(), &decay).unwrap();
        let b2 = KernelRun::new(&s, 1.0, VK, PiecewiseSource::none(), &decay).unwrap();
        let b2_a = KernelRun::new(&a, 1.0, VK, PiecewiseSource::none(), &decay).unwrap();
        let mut tested = 0;
        let mut worst = 0.0f64;
        for target in [1.0, 3.0, 10.0, 30.0, 100.0] {
            // time at which the decay-free closed form crosses target * floor (bisection)
            let (mut lo, mut hi) = (0.1, 100.0);
            for _ in 0..80 {
                let mid = 0.5 * (lo + hi);
                if f0(&sphere, mid) < target * floor {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let t = hi;
            if t < s.min_valid_time() {
                eprintln!("target {target}: t {t:.3} s is before min_valid_time, skipped");
                continue;
            }
            let (x, y) = (
                b2.state(t).unwrap().released,
                b2_a.state(t).unwrap().released,
            );
            let (sx, sy) = (run.state(t).unwrap(), run_a.state(t).unwrap());
            let b5_diff = (sx.released - sy.released).abs() / sx.total_input();
            eprintln!(
                "  decay-free: shooting {:.3e} analytic modes {:.3e} diff {:.3e}; max weight diff {:.2e}; sum w: {:.17} {:.17}",
                s.release_fraction(t),
                a.release_fraction(t),
                (s.release_fraction(t) - a.release_fraction(t)).abs(),
                s.weights.iter().zip(&a.weights).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max),
                s.weights.iter().sum::<f64>(),
                a.weights.iter().sum::<f64>()
            );
            eprintln!(
                "target {target}: t {t:.3} s, B2 value {x:.3e} diff {:.3e}, B5 value/input {:.3e} diff {b5_diff:.3e}",
                (x - y).abs(),
                sx.released / sx.total_input()
            );
            worst = f64::max(worst, f64::max((x - y).abs(), b5_diff) / floor);
            tested += 1;
        }
        eprintln!("check (v): worst difference / accuracy_floor = {worst:.2}");
        // The literal criterion (difference <= floor) is not met: the observed ratio is the
        // basis of SERIES_FLOOR_FACTOR, which item 3 of the mask uses for the existing series
        // instead (the extension quantities use the rule of Document 6, Section 8).
        assert!(worst <= SERIES_FLOOR_FACTOR, "observed ratio {worst}");
        assert!(
            tested >= 3,
            "the floor test needs times at or after min_valid_time"
        );
    }

    /// Test-local backward-time centred-space scheme for the homogeneous sphere, `u = r C`:
    /// `(u_i^{j+1} - u_i^j) / dt = D (u_{i+1} - 2 u_i + u_i-1) / h^2 - lambda u_i + r_i G chi_i`
    /// with `u_0 = u_N = 0`, `chi` the kernel indicator (one half at the kernel surface node),
    /// uniform `h`, `9 k` intervals so that the kernel surface is node `4 k`. Returns the
    /// inventory `4 pi h sum_i r_i u_i` at `t_end` (mol) for an initial concentration 1 mol/m^3
    /// in the kernel. It is NOT the production BTCS solver (finite volume, five layers): it is
    /// an independent scalar scheme, an additional check besides verifier check (iv).
    fn scalar_btcs_inventory(
        k: usize,
        steps: usize,
        t_end: f64,
        d: f64,
        lambda: f64,
        g: f64,
    ) -> f64 {
        let n = 9 * k;
        let i1 = 4 * k;
        let h = RADII[4] / n as f64;
        let dt = t_end / steps as f64;
        let mu = d * dt / (h * h);
        let chi = |i: usize| match i.cmp(&i1) {
            std::cmp::Ordering::Less => 1.0,
            std::cmp::Ordering::Equal => 0.5,
            std::cmp::Ordering::Greater => 0.0,
        };
        let r = |i: usize| i as f64 * h;
        let mut u: Vec<f64> = (0..=n).map(|i| r(i) * chi(i)).collect();
        u[0] = 0.0;
        u[n] = 0.0;
        let diag = 1.0 + 2.0 * mu + lambda * dt;
        let mut c = vec![0.0; n + 1];
        let mut dd = vec![0.0; n + 1];
        for _ in 0..steps {
            // Thomas algorithm on rows 1..n-1
            let mut prev_c = 0.0;
            let mut prev_d = 0.0;
            for i in 1..n {
                let rhs = u[i] + dt * r(i) * g * chi(i);
                let den = diag + mu * prev_c;
                c[i] = -mu / den;
                dd[i] = (rhs + mu * prev_d) / den;
                prev_c = c[i];
                prev_d = dd[i];
            }
            let mut next = 0.0;
            for i in (1..n).rev() {
                u[i] = dd[i] - c[i] * next;
                next = u[i];
            }
        }
        4.0 * PI * h * (1..n).map(|i| r(i) * u[i]).sum::<f64>()
    }

    #[test]
    fn check_iv_surrogate_scalar_btcs_converges_to_the_extension() {
        // An additional independent check, besides verifier check (iv) proper (the production
        // BTCS with the D5 rows, `tests/analytical_extensions_btcs.rs`): a scalar
        // backward-Euler / centred-difference scheme. Self-convergence orders (2 in space,
        // 1 in time), then the extrapolated limit against B3/B5.
        let (d, lambda, g, t_end) = (1.0e-10, 2.0e-3, 1.0e-3, 3000.0);
        let m = |k, steps| scalar_btcs_inventory(k, steps, t_end, d, lambda, g);
        let space = (m(8, 4000) - m(16, 4000)) / (m(16, 4000) - m(32, 4000));
        let time = (m(32, 500) - m(32, 1000)) / (m(32, 1000) - m(32, 2000));
        eprintln!("surrogate BTCS: space ratio {space:.3} (order {:.3}), time ratio {time:.3} (order {:.3})", space.log2(), time.log2());
        assert!((3.0..5.0).contains(&space), "space ratio {space}");
        assert!((1.7..2.6).contains(&time), "time ratio {time}");
        let finest = m(32, 2000);
        let limit = finest + (finest - m(16, 2000)) / 3.0 + (finest - m(32, 1000));
        let s = five_equal_series(d, 45.0 / 0.2);
        let run = KernelRun::new(
            &s,
            VK,
            VK,
            PiecewiseSource::constant(g, 1.0e5).unwrap(),
            &Decay::Uniform(lambda),
        )
        .unwrap();
        let exact = run.state(t_end).unwrap().inventory;
        eprintln!(
            "surrogate BTCS: finest {finest:.9e}, extrapolated {limit:.9e}, B3 {exact:.9e}, rel diff of extrapolation {:.2e}, of finest {:.2e}",
            rel(limit, exact),
            rel(finest, exact)
        );
        assert!(rel(limit, exact) < 1e-5);
        assert!(
            rel(finest, exact) > rel(limit, exact),
            "the extrapolation must improve"
        );
    }

    #[test]
    fn flux_is_the_derivative_of_the_released_inventory() {
        let s = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 20_000).unwrap();
        let run = KernelRun::new(
            &s,
            1.0e-9,
            VK,
            PiecewiseSource::constant(5.0e-6, 1.0e9).unwrap(),
            &Decay::Uniform(7.3e-7),
        )
        .unwrap();
        let t = 2.0e5;
        let h = 50.0;
        let dm =
            (run.state(t + h).unwrap().released - run.state(t - h).unwrap().released) / (2.0 * h);
        let phi = run.state(t).unwrap().flux;
        assert!(rel(phi, dm) < 1e-6, "{phi} vs {dm}");
    }

    #[test]
    fn mask_withholds_early_values_and_draws_late_ones() {
        let d = 1.0e-10;
        let s = five_equal_series(d, 45.0 / 20.0);
        let run = KernelRun::new(
            &s,
            1.0,
            VK,
            PiecewiseSource::none(),
            &Decay::Uniform(1.0e-3),
        )
        .unwrap()
        .with_geometry(R1, RADII[4]);
        // with the gate closed (`EXTENSION_FLOOR_REESTIMATED = false`) this run is withheld
        // altogether; the mask logic of items 1 to 3 is tested through `mask`.
        assert_eq!(
            run.drawn_gated(Quantity::ReleaseFraction, 500.0, false)
                .unwrap(),
            Drawn::Withheld(Withheld::FloorNotReestimated)
        );
        // item 1
        assert_eq!(
            run.mask(Quantity::ReleaseFraction, 0.5 * s.min_valid_time())
                .unwrap(),
            Drawn::Withheld(Withheld::BeforeMinValidTime)
        );
        // a value far below the floor or sensitive to the cover is not drawn
        let early = run
            .mask(Quantity::ReleaseFraction, s.min_valid_time())
            .unwrap();
        assert!(early.value().is_none(), "{early:?}");
        // a late value passes the mask and equals the raw value
        let late = run.mask(Quantity::ReleaseFraction, 500.0).unwrap();
        let raw = run
            .value(Quantity::ReleaseFraction, 500.0)
            .unwrap()
            .unwrap();
        assert_eq!(late, Drawn::Value(raw));
        assert!(run
            .mask(Quantity::InventoryFraction, 500.0)
            .unwrap()
            .value()
            .is_some());
        // F_gen is undefined with no inventory and no generation
        let empty = KernelRun::new(&s, 0.0, VK, PiecewiseSource::none(), &Decay::None).unwrap();
        assert!(matches!(
            empty
                .drawn(Quantity::ReleaseFractionGenerated, 500.0)
                .unwrap(),
            Drawn::Withheld(Withheld::Undefined(_))
        ));
    }

    #[test]
    fn mask_item_3_withholds_a_value_below_the_floor() {
        // A series whose floor is large: any release below 20 times the floor is not drawn.
        let s = ModalRelease {
            eigenvalues: vec![1.0e-3, 1.0],
            weights: vec![0.9, 0.1],
            lambda_covered: 1.0,
            min_flux_ratio: 1.0e-3,
        };
        assert!((s.accuracy_floor() - 2.2e-13).abs() < 1e-20);
        let run = KernelRun::new(&s, 1.0, 1.0, PiecewiseSource::none(), &Decay::None).unwrap();
        // F(40) = 1 - 0.9 exp(-1e-3 * 40) - 0.1 exp(-40) is about 0.14: drawn
        let v = run.drawn(Quantity::ReleaseFraction, 40.0).unwrap();
        assert!(v.value().is_some(), "{v:?}");
        // a slow mode so that F(40) is about 5e-13, below 20 times the floor
        let s2 = ModalRelease {
            eigenvalues: vec![1.0e-14, 1.0],
            weights: vec![1.0 - 1.0e-13, 1.0e-13],
            lambda_covered: 1.0,
            min_flux_ratio: 1.0e-3,
        };
        let run2 = KernelRun::new(&s2, 1.0, 1.0, PiecewiseSource::none(), &Decay::None).unwrap();
        let w = run2.drawn(Quantity::ReleaseFraction, 40.0).unwrap();
        assert!(
            matches!(w, Drawn::Withheld(Withheld::BelowAccuracyFloor { .. })),
            "{w:?}"
        );
    }

    /// Minor 1 of the second verification round: the floor term of `Delta` (49) is tested. A
    /// series whose only weight sits on a very slow mode has `F_0(t_v)` far below 1e-15, so the
    /// floor term is all of `Delta`.
    #[test]
    fn early_release_bound_carries_the_floor_term() {
        let s = ModalRelease {
            eigenvalues: vec![1.0e-20, 1.0],
            weights: vec![1.0, 0.0],
            lambda_covered: 1.0,
            min_flux_ratio: 0.1,
        };
        let run = KernelRun::new(
            &s,
            1.0,
            1.0,
            PiecewiseSource::none(),
            &Decay::Uniform(1.0e-3),
        )
        .unwrap();
        let st = run.state(100.0).unwrap();
        let t_v = s.min_valid_time();
        let kept: f64 = s
            .weights
            .iter()
            .zip(&s.eigenvalues)
            .map(|(w, a)| w * (-a * t_v).exp())
            .sum();
        let released_by_tv = (1.0 - kept).max(0.0);
        assert!(released_by_tv < 1.0e-15, "{released_by_tv}");
        let floor = s.accuracy_floor();
        assert_eq!(
            st.early_release,
            released_by_tv + EARLY_RELEASE_FLOOR_FACTOR * floor
        );
        // not the floor alone, and not twice it
        assert!(st.early_release > 3.9 * floor && st.early_release < 4.1 * floor);
    }

    /// The tightened flux bound (53): zero for a source on since the start without decay,
    /// `Delta lambda t_v V_K G` with decay, and `Delta (1 + lambda theta) V_K G` shortly after
    /// a switch-on.
    #[test]
    fn flux_bound_53_follows_the_age_of_the_source_pieces() {
        let s = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 20_000).unwrap();
        let t_v = s.min_valid_time();
        let g = 1.0e-3;
        let on = PiecewiseSource::constant(g, 1.0e12).unwrap();
        let run = KernelRun::new(&s, 0.0, VK, on, &Decay::None).unwrap();
        let st = run.state(1.0e6).unwrap();
        assert_eq!(st.flux_error_bound, Some(0.0));
        let lam = 2.0e-6;
        let run = KernelRun::new(
            &s,
            0.0,
            VK,
            PiecewiseSource::constant(g, 1.0e12).unwrap(),
            &Decay::Uniform(lam),
        )
        .unwrap();
        let st = run.state(1.0e6).unwrap();
        let expected = st.early_release * VK * g * lam * t_v;
        assert!(rel(st.flux_error_bound.unwrap(), expected) < 1e-12);
        let late_on = PiecewiseSource::new(vec![0.0, 2.0e5, 1.0e12], vec![0.0, g]).unwrap();
        let run = KernelRun::new(&s, 0.0, VK, late_on, &Decay::Uniform(lam)).unwrap();
        let theta = 0.5 * t_v;
        let st = run.state(2.0e5 + theta).unwrap();
        let expected = st.early_release * VK * g * (1.0 + lam * theta);
        assert!(rel(st.flux_error_bound.unwrap(), expected) < 1e-12);
    }

    /// Item 3 for a run with decay or a source is the estimated relative error at most 5 %,
    /// inside the measured regime (Document 6, Section 8).
    #[test]
    fn extension_mask_item_3_uses_the_estimated_error_and_the_regime() {
        let s = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 64).unwrap();
        assert!(s.eigenvalues.len() >= REGIME_MODES.0 && s.eigenvalues.len() <= REGIME_MODES.1);
        let src = PiecewiseSource::constant(1.0e-3, 1.0e9).unwrap();
        let run = KernelRun::new(&s, 1.0, VK, src.clone(), &Decay::Uniform(1.0e-6))
            .unwrap()
            .with_geometry(R1, RADII[4]);
        let t = 1.0e6;
        // late: drawn, and the estimated relative error is far below 5 %
        let drawn = run
            .drawn_gated(Quantity::ReleaseFractionGenerated, t, true)
            .unwrap();
        let raw = run
            .value(Quantity::ReleaseFractionGenerated, t)
            .unwrap()
            .unwrap();
        assert_eq!(drawn, Drawn::Value(raw));
        let r = run
            .estimated_relative_error(Quantity::ReleaseFractionGenerated, t)
            .unwrap()
            .unwrap();
        assert!(r > 0.0 && r < MASK_RELATIVE_TOLERANCE, "{r}");
        // a time above the measured multiple of the slowest relaxation time
        let far = REGIME_ALPHA1_T_MAX * 1.01 / s.eigenvalues[0];
        assert!(matches!(
            run.drawn_gated(Quantity::ReleaseFraction, far, true)
                .unwrap(),
            Drawn::Withheld(Withheld::OutsideEstimatedRegime(_))
        ));
        // a decay constant above the measured multiple of the slowest mode rate
        let fast = KernelRun::new(
            &s,
            1.0,
            VK,
            src.clone(),
            &Decay::Uniform(2.0 * REGIME_LAMBDA_OVER_ALPHA1_MAX * s.eigenvalues[0]),
        )
        .unwrap()
        .with_geometry(R1, RADII[4]);
        assert!(matches!(
            fast.drawn_gated(Quantity::ReleaseFraction, t, true)
                .unwrap(),
            Drawn::Withheld(Withheld::OutsideEstimatedRegime(_))
        ));
        // fewer modes than measured
        let few = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 8).unwrap();
        let run_few = KernelRun::new(&few, 1.0, VK, src.clone(), &Decay::Uniform(1.0e-6))
            .unwrap()
            .with_geometry(R1, RADII[4]);
        assert!(matches!(
            run_few
                .drawn_gated(Quantity::ReleaseFraction, 10.0 * few.min_valid_time(), true)
                .unwrap(),
            Drawn::Withheld(Withheld::OutsideEstimatedRegime(_))
        ));
        // the geometry: undeclared, outer radius out of range, kernel fraction out of range
        // (Document 6, Section 8.6: geometries outside the measured set are withheld)
        let why = |r: KernelRun| match r.drawn_gated(Quantity::ReleaseFraction, t, true).unwrap() {
            Drawn::Withheld(Withheld::OutsideEstimatedRegime(w)) => w,
            other => panic!("{other:?}"),
        };
        let fresh = || KernelRun::new(&s, 1.0, VK, src.clone(), &Decay::Uniform(1.0e-6)).unwrap();
        assert_eq!(why(fresh()), "geometry not declared");
        assert!(
            why(fresh().with_geometry(R1, 1.2 * REGIME_OUTER_RADIUS.1)).contains("outer radius")
        );
        assert!(why(
            fresh().with_geometry(0.5 * REGIME_OUTER_RADIUS.0, 0.5 * REGIME_OUTER_RADIUS.0)
        )
        .contains("outer radius"));
        assert!(why(fresh().with_geometry(0.3 * RADII[4], RADII[4])).contains("kernel radius"));
        // the end points themselves are inside
        assert!(fresh()
            .with_geometry(R1, RADII[4])
            .outside_regime(t)
            .is_none());
        assert!(fresh()
            .with_geometry(0.4 * REGIME_OUTER_RADIUS.1, REGIME_OUTER_RADIUS.1)
            .outside_regime(t)
            .is_none());
        // a value below its own error estimate: the floor term (16 floors times the input)
        // exceeds 5 % of a release of 1e-12 (synthetic series with 16 modes)
        let mut eigenvalues = vec![1.0e-14];
        eigenvalues.extend((0..15).map(|k| 1.0 + 0.1 * k as f64));
        let mut weights = vec![1.0 - 15.0e-20];
        weights.extend(std::iter::repeat_n(1.0e-20, 15));
        let synthetic = ModalRelease {
            eigenvalues,
            weights,
            lambda_covered: 2.4,
            min_flux_ratio: 1.0e-3,
        };
        let run_s = KernelRun::new(
            &synthetic,
            1.0,
            1.0,
            PiecewiseSource::none(),
            &Decay::Uniform(1.0e-20),
        )
        .unwrap()
        .with_geometry(R1, RADII[4]);
        let w = run_s
            .drawn_gated(Quantity::ReleaseFraction, 100.0, true)
            .unwrap();
        assert!(
            matches!(w, Drawn::Withheld(Withheld::ExtensionErrorTooLarge { .. })),
            "{w:?}"
        );
    }

    /// A series of 16 modes (one slow mode with almost all the weight, fifteen fast modes with
    /// weights of 1e-20) whose accuracy floor is `2.2e-16 / ratio` (`ratio <= 1`).
    fn floor_series(ratio: f64) -> ModalRelease {
        let mut eigenvalues = vec![1.0e-14];
        eigenvalues.extend((0..15).map(|k| 1.0 + 0.1 * k as f64));
        let mut weights = vec![1.0 - 15.0e-20];
        weights.extend(std::iter::repeat_n(1.0e-20, 15));
        ModalRelease {
            eigenvalues,
            weights,
            lambda_covered: 2.4,
            min_flux_ratio: ratio,
        }
    }

    /// Minor 1 of the third verification round: the drawing threshold of item 3 for a run with
    /// decay or a source is 5 % of the value. A value whose estimated relative error is just
    /// below 5 % is drawn; one just above is withheld with `ExtensionErrorTooLarge`.
    #[test]
    fn extension_threshold_is_five_percent_of_the_value() {
        let estimated = |ratio: f64| {
            let s = floor_series(ratio);
            KernelRun::new(
                &s,
                1.0,
                1.0,
                PiecewiseSource::none(),
                &Decay::Uniform(1.0e-20),
            )
            .unwrap()
            .with_geometry(R1, RADII[4])
            .estimated_relative_error(Quantity::ReleaseFraction, 1.0e4)
            .unwrap()
            .unwrap()
        };
        // the estimated relative error grows with the floor, that is falls with the ratio:
        // find the ratio at which it crosses 5 % by bisection on a logarithmic scale
        let (mut lo, mut hi) = (1.0e-4_f64, 1.0_f64);
        assert!(estimated(lo) > 0.1 && estimated(hi) < 0.01);
        for _ in 0..80 {
            let mid = (lo * hi).sqrt();
            if estimated(mid) > MASK_RELATIVE_TOLERANCE {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let verdict = |ratio: f64| {
            let s = floor_series(ratio);
            KernelRun::new(
                &s,
                1.0,
                1.0,
                PiecewiseSource::none(),
                &Decay::Uniform(1.0e-20),
            )
            .unwrap()
            .with_geometry(R1, RADII[4])
            .drawn_gated(Quantity::ReleaseFraction, 1.0e4, true)
            .unwrap()
        };
        let below = verdict(hi * 1.02); // relative error about 4.9 %
        let above = verdict(lo * 0.98); // relative error about 5.1 %
        assert!(
            matches!(below, Drawn::Value(_)),
            "{below:?}, r = {}",
            estimated(hi * 1.02)
        );
        match above {
            Drawn::Withheld(Withheld::ExtensionErrorTooLarge { relative_error }) => {
                assert!(relative_error > MASK_RELATIVE_TOLERANCE && relative_error < 0.06);
            }
            other => panic!("{other:?}"),
        }
        let r_below = estimated(hi * 1.02);
        assert!(
            r_below > 0.045 && r_below <= MASK_RELATIVE_TOLERANCE,
            "{r_below}"
        );
    }

    /// Major 1 of the third verification round: the estimate (54) is the sum of the computed
    /// model bound, the computed rounding bound of the evaluation and the floor term, for each
    /// of the three quantities, and the rounding bound is positive and grows with the number of
    /// terms summed (a mutant that drops a part changes the sum exactly).
    #[test]
    fn estimate_is_model_bound_plus_rounding_bound_plus_floor_term() {
        let src = PiecewiseSource::constant(1.0e-3, 1.0e9).unwrap();
        let state = |n: usize| {
            // the number of modes below a bound grows with the square root of the bound
            let bound = 45.0 / 5.0e3 * (n as f64 / 64.0).powi(2);
            let s = ModalRelease::new(&layers(D_FIVE), bound, n).unwrap();
            let run = KernelRun::new(&s, 1.0, VK, src.clone(), &Decay::Uniform(1.0e-6)).unwrap();
            (run.state(1.0e6).unwrap(), s.accuracy_floor())
        };
        let (st, floor) = state(64);
        let e = st.estimated_errors(floor);
        assert!(st.inventory_rounding_bound > 0.0);
        assert!(st.release_rounding_bound > 0.0);
        assert!(st.flux_rounding_bound > 0.0);
        assert_eq!(
            e.inventory,
            st.inventory_error_bound
                + st.inventory_rounding_bound
                + EXT_FACTOR_INVENTORY * floor * st.age_input
        );
        assert_eq!(
            e.released,
            st.release_error_bound
                + st.release_rounding_bound
                + EXT_FACTOR_RELEASE * floor * st.age_input
        );
        assert_eq!(
            e.flux.unwrap(),
            st.flux_error_bound.unwrap()
                + st.flux_rounding_bound
                + EXT_FACTOR_FLUX * floor * st.flux_scale
        );
        // more modes, more terms in each sum: a larger rounding bound
        let (more, _) = state(256);
        assert!(more.inventory_rounding_bound > st.inventory_rounding_bound);
        // the existing series (no source, no decay) carries no rounding bound
        let s = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 64).unwrap();
        let plain = KernelRun::new(&s, 1.0, VK, PiecewiseSource::none(), &Decay::None)
            .unwrap()
            .state(1.0e6)
            .unwrap();
        assert_eq!(plain.inventory_rounding_bound, 0.0);
    }

    /// Minor 2 of the third verification round: the estimate of `F_post` is the sum of the
    /// release terms at `t` and at `t_e` over the gained release, plus the relative error of
    /// the inventory at `t_e`; each of the three terms is positive, and the regime limits of the
    /// floor, of the number of pieces and of the time withhold the value.
    #[test]
    fn post_event_estimate_has_its_three_terms_and_the_regime_limits() {
        let s = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 64).unwrap();
        let src = PiecewiseSource::constant(1.0e-3, 1.0e9).unwrap();
        let run = KernelRun::new(&s, 1.0, VK, src.clone(), &Decay::Uniform(1.0e-6))
            .unwrap()
            .with_geometry(R1, RADII[4]);
        let (t_e, t) = (2.0e5, 1.0e6);
        let q = Quantity::PostEventRelease { t_event: t_e };
        let floor = s.accuracy_floor();
        let (st, ev_) = (run.state(t).unwrap(), run.state(t_e).unwrap());
        let (e, ee) = (st.estimated_errors(floor), ev_.estimated_errors(floor));
        let gained = st.released - ev_.released;
        let term_release = (e.released + ee.released) / gained;
        let term_inventory = ee.inventory / ev_.inventory;
        assert!(term_release > 0.0 && term_inventory > 0.0);
        // the two release errors enter, and the inventory error at the event enters
        assert!(e.released > 0.0 && ee.released > 0.0);
        let got = run.estimated_relative_error(q, t).unwrap().unwrap();
        assert_eq!(got, term_release + term_inventory);
        // not the release terms of one time only, and not without the inventory term
        assert!(got > (e.released) / gained + term_inventory);
        assert!(got > term_release);

        // the floor limit
        let high_floor = floor_series(1.0e-8);
        assert!(high_floor.accuracy_floor() > REGIME_FLOOR_MAX);
        let run_f = KernelRun::new(
            &high_floor,
            1.0,
            1.0,
            PiecewiseSource::none(),
            &Decay::Uniform(1.0e-20),
        )
        .unwrap()
        .with_geometry(R1, RADII[4]);
        match run_f
            .drawn_gated(Quantity::ReleaseFraction, 100.0, true)
            .unwrap()
        {
            Drawn::Withheld(Withheld::OutsideEstimatedRegime(w)) => assert!(w.contains("floor")),
            other => panic!("{other:?}"),
        }
        // the limit on the number of source pieces
        let breaks: Vec<f64> = (0..=REGIME_PIECES_MAX + 1)
            .map(|k| 1.0e4 * k as f64)
            .collect();
        let rates = vec![1.0e-3; REGIME_PIECES_MAX + 1];
        let many = PiecewiseSource::new(breaks, rates).unwrap();
        let run_p = KernelRun::new(&s, 1.0, VK, many, &Decay::Uniform(1.0e-6))
            .unwrap()
            .with_geometry(R1, RADII[4]);
        match run_p
            .drawn_gated(Quantity::ReleaseFraction, 1.0e6, true)
            .unwrap()
        {
            Drawn::Withheld(Withheld::OutsideEstimatedRegime(w)) => assert!(w.contains("pieces")),
            other => panic!("{other:?}"),
        }
        // F_post is withheld when its time lies beyond the measured multiple of 1 / alpha_1
        // (the time of the event, never later than t, needs no check of its own)
        let far = REGIME_ALPHA1_T_MAX * 1.01 / s.eigenvalues[0];
        assert!(matches!(
            run.drawn_gated(q, far, true).unwrap(),
            Drawn::Withheld(Withheld::OutsideEstimatedRegime(_))
        ));
        // and F_post is drawn where the estimate allows it
        assert!(run.drawn_gated(q, t, true).unwrap().value().is_some());
    }

    /// The compensated sum with its a-posteriori bound: against the exact sum of terms that
    /// are all integer multiples of 2^-93 (so that the exact sum is an integer of `i128`), for
    /// 2048 terms with alternating signs and a large cancellation. The plain sum is off by
    /// several times more, and the bound holds for the compensated one.
    #[test]
    fn compensated_sum_stays_within_its_bound_where_the_plain_sum_does_not() {
        let scale = 2.0_f64.powi(93);
        let n = 2048;
        let terms: Vec<f64> = (0..n)
            .map(|k| {
                let x = 0.5 + 0.4 * ((k as f64) * 0.7123).sin() + 1.0 / (3.0 + k as f64);
                (if k % 2 == 0 { x } else { -x }) * (1.0 + 1.0e-9 * k as f64)
            })
            .collect();
        let exact: i128 = terms.iter().map(|&t| (t * scale) as i128).sum();
        // every term is an exact multiple of 2^-93 (it has at most 53 significant bits and a
        // magnitude of at least 2^-41)
        assert!(terms
            .iter()
            .all(|&t| ((t * scale) as i128) as f64 == t * scale));
        let err = |r: f64| ((r * scale) as i128 - exact).abs() as f64 / scale;
        let comp = sum_terms(terms.iter().map(|&t| ev(t)));
        let plain: f64 = terms.iter().sum();
        assert!(err(comp.v) <= comp.e, "{} vs bound {}", err(comp.v), comp.e);
        // far below the worst case n u sum|x| of the plain sum
        let abs_sum: f64 = terms.iter().map(|t| t.abs()).sum();
        assert!(
            comp.e < 1.0e-3 * n as f64 * UNIT_ROUNDOFF * abs_sum,
            "{} {}",
            comp.e,
            err(plain)
        );
    }

    #[test]
    fn mask_item_2_withholds_a_value_that_depends_on_the_cover() {
        // Synthetic modes (not a physical series) with a heavy weight in the band between the
        // reduced and the full cover: the value changes by more than 5 % when the cover is
        // reduced, so it must be withheld.
        let s = ModalRelease {
            eigenvalues: vec![1.0e-3, 30.0, 100.0],
            weights: vec![0.5, 1.0e4, -1.0e4 + 0.5],
            lambda_covered: 100.0,
            min_flux_ratio: f64::INFINITY,
        };
        let run = KernelRun::new(&s, 1.0, 1.0, PiecewiseSource::none(), &Decay::None).unwrap();
        let t = s.min_valid_time();
        assert!(matches!(
            run.drawn(Quantity::ReleaseFraction, t).unwrap(),
            Drawn::Withheld(Withheld::CoverSensitivity { .. })
        ));
    }

    #[test]
    fn beta_is_withheld_early_and_tends_to_one_late() {
        let d = 1.0e-10;
        let s = five_equal_series(d, 45.0 / 20.0);
        let src = PiecewiseSource::constant(1.0e-3, 1.0e7).unwrap();
        let run = KernelRun::new(&s, 0.0, VK, src, &Decay::None)
            .unwrap()
            .with_geometry(R1, RADII[4]);
        assert_eq!(
            run.drawn_gated(Quantity::Beta, 1.0e5, false).unwrap(),
            Drawn::Withheld(Withheld::FloorNotReestimated)
        );
        assert_eq!(
            run.mask(Quantity::Beta, 0.5 * s.min_valid_time()).unwrap(),
            Drawn::Withheld(Withheld::BeforeMinValidTime)
        );
        let late = run.mask(Quantity::Beta, 1.0e5).unwrap().value().unwrap();
        assert!((late - 1.0).abs() < 1e-9, "{late}");
        // beta is absent where G(t) = 0 (after the last breakpoint)
        assert_eq!(run.value(Quantity::Beta, 2.0e7).unwrap(), None);
    }

    #[test]
    fn post_event_release_identities() {
        let s = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 20_000).unwrap();
        let run = KernelRun::new(&s, 1.0, VK, PiecewiseSource::none(), &Decay::None).unwrap();
        let te = 1.0e5;
        let at_event = run
            .value(Quantity::PostEventRelease { t_event: te }, te)
            .unwrap()
            .unwrap();
        assert_eq!(at_event, 0.0);
        let t = 4.0e5;
        let post = run
            .value(Quantity::PostEventRelease { t_event: te }, t)
            .unwrap()
            .unwrap();
        let (f, fe) = (s.release_fraction(t), s.release_fraction(te));
        assert!(((f - fe) / (1.0 - fe) - post).abs() < 1e-12);
        // before the event the quantity is undefined
        assert_eq!(
            run.value(Quantity::PostEventRelease { t_event: te }, 0.5 * te)
                .unwrap(),
            None
        );
    }

    #[test]
    fn refusals() {
        let s = ModalRelease::new(&layers(D_FIVE), 45.0 / 5.0e3, 2000).unwrap();
        let none = PiecewiseSource::none;
        // unequal decay constants in different layers: never a silent result
        assert_eq!(
            s.with_decay(
                1.0,
                &Decay::PerLayer(vec![1e-9, 1e-9, 2e-9, 1e-9, 1e-9]),
                1e5
            ),
            Err(ExtensionError::UnequalDecayConstants)
        );
        assert!(s
            .with_decay(1.0, &Decay::PerLayer(vec![1e-9; 5]), 1e5)
            .is_ok());
        assert!(matches!(
            s.with_decay(1.0, &Decay::PerLayer(vec![]), 1e5),
            Err(ExtensionError::NotFinite(_))
        ));
        assert_eq!(
            s.with_decay(1.0, &Decay::Uniform(-1e-9), 1e5),
            Err(ExtensionError::Negative("decay constant"))
        );
        assert_eq!(
            s.with_decay(1.0, &Decay::Uniform(f64::NAN), 1e5),
            Err(ExtensionError::NotFinite("decay constant"))
        );
        assert_eq!(
            s.with_decay(-1.0, &Decay::None, 1e5),
            Err(ExtensionError::Negative("initial inventory"))
        );
        assert_eq!(
            s.with_decay(1.0, &Decay::None, -1.0),
            Err(ExtensionError::Negative("time"))
        );
        assert_eq!(
            s.with_decay(1.0, &Decay::None, f64::INFINITY),
            Err(ExtensionError::NotFinite("time"))
        );
        assert_eq!(
            s.with_source_decay(1.0, 0.0, &none(), &Decay::None, 1.0),
            Err(ExtensionError::NotPositive("kernel volume"))
        );
        // source histories
        assert!(PiecewiseSource::new(vec![1.0, 2.0], vec![1.0]).is_err());
        assert!(PiecewiseSource::new(vec![0.0, 2.0, 1.0], vec![1.0, 1.0]).is_err());
        assert!(PiecewiseSource::new(vec![0.0, 1.0], vec![1.0, 1.0]).is_err());
        // equal breakpoints (s_1 = s_0) are refused: the breakpoints are strictly increasing
        assert!(PiecewiseSource::new(vec![0.0, 0.0], vec![1.0]).is_err());
        assert!(PiecewiseSource::new(vec![0.0, 5.0, 5.0], vec![1.0, 2.0]).is_err());
        // exactly one decay constant per layer
        for n in [1usize, 4, 6] {
            assert_eq!(
                s.with_decay(1.0, &Decay::PerLayer(vec![1e-9; n]), 1e5),
                Err(ExtensionError::WrongLayerCount(n))
            );
        }
        assert!(PiecewiseSource::new(vec![0.0, 1.0], vec![-1.0]).is_err());
        assert!(PiecewiseSource::new(vec![0.0, f64::NAN], vec![1.0]).is_err());
        assert!(PiecewiseSource::new(vec![0.0, 1.0], vec![f64::INFINITY]).is_err());
        assert!(PiecewiseSource::new(vec![], vec![]).is_err());
        assert!(PiecewiseSource::constant(1.0, 0.0).is_err());
        // an empty or inconsistent series
        let empty = ModalRelease {
            eigenvalues: vec![],
            weights: vec![],
            lambda_covered: 1.0,
            min_flux_ratio: 1.0,
        };
        assert_eq!(
            KernelRun::new(&empty, 1.0, 1.0, none(), &Decay::None).unwrap_err(),
            ExtensionError::NoModes
        );
        let ragged = ModalRelease {
            eigenvalues: vec![1.0, 2.0],
            weights: vec![1.0],
            lambda_covered: 2.0,
            min_flux_ratio: 1.0,
        };
        assert_eq!(
            KernelRun::new(&ragged, 1.0, 1.0, none(), &Decay::None).unwrap_err(),
            ExtensionError::InconsistentSeries
        );
        // F needs an initial inventory
        let run = KernelRun::new(&s, 0.0, VK, none(), &Decay::None).unwrap();
        assert_eq!(run.value(Quantity::ReleaseFraction, 1e5).unwrap(), None);
        assert_eq!(run.value(Quantity::InventoryFraction, 1e5).unwrap(), None);
        // a negative event time is refused
        let run = KernelRun::new(&s, 1.0, VK, none(), &Decay::None).unwrap();
        assert!(run
            .value(Quantity::PostEventRelease { t_event: -1.0 }, 1e5)
            .is_err());
    }

    #[test]
    fn source_helpers_and_kernel_functions() {
        let src = PiecewiseSource::new(vec![0.0, 10.0, 30.0], vec![2.0, 5.0]).unwrap();
        assert_eq!(src.rate_at(0.0), 2.0);
        assert_eq!(src.rate_at(10.0), 5.0);
        assert_eq!(src.rate_at(30.0), 0.0);
        assert_eq!(src.generated_per_volume(5.0), 10.0);
        assert_eq!(src.generated_per_volume(20.0), 20.0 + 50.0);
        assert_eq!(src.generated_per_volume(100.0), 20.0 + 100.0);
        assert!(!src.is_zero());
        assert!(PiecewiseSource::none().is_zero());
        // the stable helpers against their definitions
        for (lam, x) in [
            (0.0, 3.0),
            (1.0e-9, 5.0),
            (0.2, 1.0),
            (2.0, 7.0),
            (1.0e-3, 40.0),
        ] {
            let le = len_exp(lam, ev(x)).v;
            let numeric = simpson(0.0, x, 2000, |s| (-lam * s).exp());
            // the reference is a Simpson rule: its own error is about 1e-10 here
            assert!(rel(le, numeric) < 1e-9, "len_exp {lam} {x}");
            let lei = len_exp_integral(lam, ev(x)).v;
            let numeric = simpson(0.0, x, 2000, |s| len_exp(lam, ev(s)).v);
            assert!(rel(lei, numeric) < 1e-9, "len_exp_integral {lam} {x}");
        }
        // continuity of the series branch of len_exp_integral
        let (lam, x) = (1.0, 0.0499999999);
        let a = len_exp_integral(lam, ev(x)).v;
        let b = len_exp_integral(lam, ev(x * (1.0 + 1e-6))).v;
        assert!(rel(a, b) < 1e-5);
    }
}
