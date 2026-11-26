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

/// Factor by which item 3 of the drawing mask multiplies `accuracy_floor()`. Verifier check (v)
/// of the science design (open point O10) compared the series with closed-form modes at values
/// of 1 to 100 times `accuracy_floor()` and observed differences up to 3.3 times the floor, a
/// constant offset that the existing decay-free series has as well (the weights of the shooting
/// solution, not the extension). The science design then asks for item 3 to use the observed
/// error instead of the floor. 4 is that observed ratio rounded up: a labelled convention
/// (observed in one geometry, not derived), applied to every quantity because the offset
/// belongs to the series. The floor itself is to be re-estimated.
pub const FLOOR_FACTOR_FROM_CHECK_V: f64 = 4.0;

/// Reasons an extension refuses its input. A refusal never produces a number.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtensionError {
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
    /// One decay constant per layer. Accepted only when all are exactly equal; otherwise the
    /// extension refuses with [`ExtensionError::UnequalDecayConstants`].
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
    /// Bound on `|Phi - Phi(exact)|` (mol s^-1); valid from `t >= t_v` only, `None` before.
    pub flux_error_bound: Option<f64>,
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
    /// Item 3: the floor exceeds 5 % of the value (as a fraction of the input).
    BelowAccuracyFloor {
        /// The value expressed as a fraction (of `M^0 + M_gen`, of `M(t_e)`, or `beta`).
        value: f64,
        /// `FLOOR_FACTOR_FROM_CHECK_V * accuracy_floor()` of the series.
        floor: f64,
    },
    /// The quantity is not defined here (for example `F` with `M^0 = 0`).
    Undefined(&'static str),
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

/// `int_0^x exp(-lambda s) ds = (1 - exp(-lambda x)) / lambda`, `x` for `lambda = 0`.
fn len_exp(lambda: f64, x: f64) -> f64 {
    if lambda == 0.0 {
        x
    } else {
        -(-lambda * x).exp_m1() / lambda
    }
}

/// `int_0^x len_exp(lambda, s) ds = (lambda x + expm1(-lambda x)) / lambda^2`; a series for
/// small `lambda x` (cancellation), `x^2 / 2` for `lambda = 0`.
fn len_exp_integral(lambda: f64, x: f64) -> f64 {
    let z = lambda * x;
    if z < 0.1 {
        // x^2 sum_n (-z)^n / (n + 2)!, nine terms: the omitted term is below 1e-17 of the sum
        let (mut term, mut sum) = (0.5, 0.5);
        for n in 1..9 {
            term *= -z / (n as f64 + 2.0);
            sum += term;
        }
        x * x * sum
    } else {
        (z + (-z).exp_m1()) / (lambda * lambda)
    }
}

/// The held modes with the factors `exp(-a_mu t_v)`: the building blocks of the evaluation.
struct Kernel<'a> {
    alpha: &'a [f64],
    weight: &'a [f64],
    /// `a_mu = alpha_mu + lambda`.
    a: Vec<f64>,
    /// `c_mu = w_mu exp(-a_mu t_v)`.
    c: Vec<f64>,
    lambda: f64,
    t_v: f64,
    /// `int_0^{t_v} exp(-lambda s) ds`.
    e_v: f64,
    /// `int_0^{t_v} E(s) ds`.
    q_v: f64,
}

impl<'a> Kernel<'a> {
    fn new(alpha: &'a [f64], weight: &'a [f64], lambda: f64, t_v: f64) -> Self {
        let a: Vec<f64> = alpha.iter().map(|x| x + lambda).collect();
        let c: Vec<f64> = weight
            .iter()
            .zip(&a)
            .map(|(w, a)| w * (-a * t_v).exp())
            .collect();
        Self {
            alpha,
            weight,
            a,
            c,
            lambda,
            t_v,
            e_v: len_exp(lambda, t_v),
            q_v: len_exp_integral(lambda, t_v),
        }
    }

    /// `exp(-lambda theta) R^(theta)`: the model remaining fraction with decay.
    fn remaining(&self, theta: f64) -> f64 {
        if theta < self.t_v {
            (-self.lambda * theta).exp()
        } else {
            self.weight
                .iter()
                .zip(self.alpha)
                .map(|(w, al)| w * (-(al + self.lambda) * theta).exp())
                .sum()
        }
    }

    /// `E(theta) = int_0^theta exp(-lambda s) R^(s) ds`.
    fn e0(&self, theta: f64) -> f64 {
        if theta < self.t_v {
            len_exp(self.lambda, theta)
        } else {
            let l = theta - self.t_v;
            self.e_v
                + self
                    .c
                    .iter()
                    .zip(&self.a)
                    .map(|(c, a)| c * (-(-a * l).exp_m1()) / a)
                    .sum::<f64>()
        }
    }

    /// `Q(theta) = int_0^theta E(s) ds`.
    fn q0(&self, theta: f64) -> f64 {
        if theta < self.t_v {
            len_exp_integral(self.lambda, theta)
        } else {
            let l = theta - self.t_v;
            self.q_v
                + self.e_v * l
                + self
                    .c
                    .iter()
                    .zip(&self.a)
                    .map(|(c, a)| c / a * (l + (-a * l).exp_m1() / a))
                    .sum::<f64>()
        }
    }

    /// `F^_lambda(theta) = 1 - exp(-lambda theta) R^(theta) - lambda E(theta)`: the released
    /// fraction of a unit kernel-step inventory (0 before `t_v`).
    fn release(&self, theta: f64) -> f64 {
        if theta < self.t_v {
            0.0
        } else {
            let s: f64 = self
                .weight
                .iter()
                .zip(self.alpha)
                .map(|(w, al)| w * (-al * theta).exp())
                .sum();
            // 1 - exp(-lambda theta) s = (1 - exp(-lambda theta)) + exp(-lambda theta) (1 - s)
            -(-self.lambda * theta).exp_m1() + (-self.lambda * theta).exp() * (1.0 - s)
                - self.lambda * self.e0(theta)
        }
    }

    /// `int_0^theta F^_lambda(s) ds`.
    fn release_integral(&self, theta: f64) -> f64 {
        if theta < self.t_v {
            0.0
        } else {
            (theta - self.t_v)
                - (self.e0(theta) - self.e_v)
                - self.lambda * (self.q0(theta) - self.q_v)
        }
    }

    /// `dF^_lambda/dtheta = exp(-lambda theta) sum_mu w_mu alpha_mu exp(-alpha_mu theta)`
    /// (0 before `t_v`).
    fn release_rate(&self, theta: f64) -> f64 {
        if theta < self.t_v {
            0.0
        } else {
            self.weight
                .iter()
                .zip(self.alpha)
                .map(|(w, al)| w * al * (-(al + self.lambda) * theta).exp())
                .sum()
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
        })
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

    /// The state at time `t` (s) with all modes of the series. This is the raw evaluation of
    /// (42) to (49) without the drawing mask: use [`KernelRun::drawn`] for anything that is
    /// displayed.
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
        let delta = (1.0 - remaining_at_tv).max(0.0) + s.accuracy_floor();

        let m0 = self.m0;
        let v = self.v_k;
        let mut inventory = m0 * k.remaining(t);
        let mut decay_integral = m0 * k.e0(t);
        let mut released = m0 * k.release(t);
        let mut flux = m0 * k.release_rate(t);
        let mut sum_g = 0.0; // sum of G_k over the pieces started before t
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
            inventory += v * g * (k.e0(th_hi) - k.e0(th_lo));
            decay_integral += v * g * (k.q0(th_hi) - k.q0(th_lo));
            released += v * g * (k.release_integral(th_hi) - k.release_integral(th_lo));
            flux += v * g * (k.release(th_hi) - k.release(th_lo));
            if g > 0.0 {
                sum_g += g;
                let len = ej - sj;
                let overlap = ((th_hi).min(t_v) - th_lo).max(0.0);
                inv_b += delta * v * g * overlap;
                rel_b += delta * v * g * (overlap + lam * t_v * len);
            }
        }
        let generated = v * self.source.generated_per_volume(t);
        // The decay loss: lambda int_0^t M ds, from the same remaining fraction.
        let mut decayed = lam * decay_integral;
        let mut released_out = released;
        if self.no_source_no_decay() {
            // The existing series term by term: F = 1 - sum w exp(-alpha t).
            let remaining: f64 = alpha
                .iter()
                .zip(weight)
                .map(|(l, w)| w * (-l * t).exp())
                .sum();
            released_out = m0 * (1.0 - remaining);
            decayed = 0.0;
        }
        Ok(ModalState {
            time: t,
            inventory,
            released: released_out,
            decayed,
            generated,
            initial_inventory: m0,
            flux,
            source_rate: self.source.rate_at(t),
            weight_sum,
            modes: n,
            relax_time: t_v,
            early_release: delta,
            inventory_error_bound: inv_b,
            release_error_bound: rel_b,
            flux_error_bound: (t >= t_v).then_some(2.0 * delta * v * sum_g * (1.0 + lam * t_v)),
        })
    }

    /// The value of `q` at `t` with all modes, or `None` where the quantity is not defined.
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
    /// the series is not trustworthy.
    pub fn drawn(&self, q: Quantity, t: f64) -> Result<Drawn, ExtensionError> {
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
        // Item 3 for release quantities, the value expressed as a fraction of the input.
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
            let floor = FLOOR_FACTOR_FROM_CHECK_V * s.accuracy_floor();
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
    /// Refuses with an error where a condition does not hold. Raw evaluation: the drawing
    /// mask is [`KernelRun::drawn`].
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
    /// in every layer, constant diffusivities (those of the series).
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
        // basis of FLOOR_FACTOR_FROM_CHECK_V, which item 3 of the mask uses instead.
        assert!(worst <= FLOOR_FACTOR_FROM_CHECK_V, "observed ratio {worst}");
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
    /// an independent scalar scheme used as a surrogate for verifier check (iv).
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
        // Surrogate for verifier check (iv): the production BTCS with the D5 rows is not on this
        // branch (it is on feat/btcs-physics, not yet verified), so an independent scalar
        // backward-Euler / centred-difference scheme is run in its place. Self-convergence
        // orders (2 in space, 1 in time), then the extrapolated limit against B3/B5.
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
        .unwrap();
        // item 1
        assert_eq!(
            run.drawn(Quantity::ReleaseFraction, 0.5 * s.min_valid_time())
                .unwrap(),
            Drawn::Withheld(Withheld::BeforeMinValidTime)
        );
        // a value far below the floor or sensitive to the cover is not drawn
        let early = run
            .drawn(Quantity::ReleaseFraction, s.min_valid_time())
            .unwrap();
        assert!(early.value().is_none(), "{early:?}");
        // a late value is drawn and equals the raw value
        let late = run.drawn(Quantity::ReleaseFraction, 500.0).unwrap();
        let raw = run
            .value(Quantity::ReleaseFraction, 500.0)
            .unwrap()
            .unwrap();
        assert_eq!(late, Drawn::Value(raw));
        assert!(run
            .drawn(Quantity::InventoryFraction, 500.0)
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
        let run = KernelRun::new(&s, 0.0, VK, src, &Decay::None).unwrap();
        assert_eq!(
            run.drawn(Quantity::Beta, 0.5 * s.min_valid_time()).unwrap(),
            Drawn::Withheld(Withheld::BeforeMinValidTime)
        );
        let late = run.drawn(Quantity::Beta, 1.0e5).unwrap().value().unwrap();
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
            let le = len_exp(lam, x);
            let numeric = simpson(0.0, x, 2000, |s| (-lam * s).exp());
            // the reference is a Simpson rule: its own error is about 1e-10 here
            assert!(rel(le, numeric) < 1e-9, "len_exp {lam} {x}");
            let lei = len_exp_integral(lam, x);
            let numeric = simpson(0.0, x, 2000, |s| len_exp(lam, s));
            assert!(rel(lei, numeric) < 1e-9, "len_exp_integral {lam} {x}");
        }
        // continuity of the series branch of len_exp_integral
        let (lam, x) = (1.0, 0.0499999999);
        let a = len_exp_integral(lam, x);
        let b = len_exp_integral(lam, x * (1.0 + 1e-6));
        assert!(rel(a, b) < 1e-5);
    }
}
