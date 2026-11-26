//! Re-estimate of the accuracy floor of Method B for the extension formulas (B2) to (B5)
//! (decision 68(c)); tests and the helper that prints the estimate table. Test code only.
//!
//! The error of the extension evaluation ([`KernelRun`]) is measured against references over
//! a designed set of geometries, mode counts, decay constants, source histories and times.
//! References are of two kinds and every row says which:
//!
//! * `closed form`: independent of the evaluation under test (the uniform sphere of the
//!   physics module, with its own series, and a quadrature of the superposition integral);
//! * `self reference`: the same evaluation with a much larger mode count (it measures the
//!   mode-count error, and not the error of the formulas or of the shooting solution).
//!
//! The derivation of the rule that uses the table is in
//! `docs/mathematics/btcs/06_analytical_extensions.md`, Section 8.

use super::extensions::{
    Decay, KernelRun, ModalState, PiecewiseSource, EARLY_RELEASE_FLOOR_FACTOR,
};
use super::homogeneous_sphere::HomogeneousSphere as ClosedCore;
use super::sturm_liouville::{Layer, ModalRelease};
use crate::physics::reference::HomogeneousSphere as PhysicsSphere;
use std::f64::consts::PI;

/// Outer radii of the reference particle (m): the registered AGR-1 baseline geometry
/// (kernel 175 um, then the cumulative radii of buffer, IPyC, SiC and OPyC).
pub const RADII: [f64; 5] = [175.0e-6, 278.5e-6, 317.9e-6, 353.2e-6, 394.2e-6];

/// The size of a run of the designed set: mode counts under test and reference sizes.
#[derive(Debug, Clone)]
pub struct SetConfig {
    /// Mode counts under test.
    pub mode_counts: Vec<usize>,
    /// Mode count of the self reference of the layered geometries (at least 8 times the
    /// largest mode count under test).
    pub reference_modes: usize,
    /// Mode count of the closed-form reference of the homogeneous particle.
    pub closed_modes: usize,
    /// Time points per decade.
    pub per_decade: f64,
    /// Last time of the set as a multiple of `1 / alpha_1` of the geometry.
    pub t_end: f64,
    /// Registered nuclides whose decay constants enter (besides no decay).
    pub nuclides: Vec<&'static str>,
    /// Run only the geometries whose name contains this text (all when `None`).
    pub only: Option<String>,
}

impl SetConfig {
    /// The set the committed test asserts (cheap enough for a debug build).
    pub fn test_set() -> Self {
        Self {
            mode_counts: vec![8, 16, 32, 64],
            reference_modes: 1024,
            closed_modes: 8000,
            per_decade: 1.5,
            t_end: 300.0,
            nuclides: vec!["Cs-134", "Kr-85m", "Ag-110"],
            only: None,
        }
    }

    /// The larger set printed by the example (release build).
    pub fn full_set() -> Self {
        Self {
            mode_counts: vec![16, 64, 256, 1024, 2048],
            reference_modes: 16_384,
            closed_modes: 40_000,
            per_decade: 3.0,
            t_end: 1000.0,
            nuclides: vec!["Cs-137", "Cs-134", "I-131", "Kr-85m", "Ag-110"],
            only: None,
        }
    }
}

/// Registered decay constants (s^-1) of the unstable nuclides of `data/nuclear/decay.csv`.
pub fn registered_decay_constants() -> Vec<(String, f64)> {
    let text = include_str!("../../data/nuclear/decay.csv");
    let mut out = Vec::new();
    for line in text.lines().skip(1) {
        let f: Vec<&str> = line.split(',').take(12).collect();
        if f.len() == 12 && f[5] == "no" {
            if let Ok(l) = f[11].parse::<f64>() {
                if l > 0.0 {
                    out.push((f[0].to_string(), l));
                }
            }
        }
    }
    out
}

/// Layers with the given diffusivities (m^2 s^-1) on the registered radii.
pub fn layers_of(d: [f64; 5]) -> Vec<Layer> {
    RADII
        .iter()
        .zip(d)
        .map(|(&r, d)| Layer {
            outer_radius: r,
            diffusivity: d,
        })
        .collect()
}

/// The series of the first `n` modes of the layered particle with diffusivities `d`. The
/// eigenvalue limit handed to the module is `D_max (2 pi n / R_5)^2`, at which the Sturm
/// count is at least `2n` (a limit as large as 1e15 s^-1 makes the count of the module
/// unreliable: rounding of phases of the order of 1e10 rad, and the cap on the number of
/// modes then does not hold).
pub fn layered_series(d: [f64; 5], n: usize) -> ModalRelease {
    layered_series_on(RADII, d, n)
}

/// As [`layered_series`] on the given outer radii (m).
pub fn layered_series_on(radii: [f64; 5], d: [f64; 5], n: usize) -> ModalRelease {
    let d_max = d.iter().cloned().fold(0.0, f64::max);
    let lambda_max = d_max * (2.0 * PI * n as f64 / radii[4]).powi(2);
    let layers: Vec<Layer> = radii
        .iter()
        .zip(d)
        .map(|(&r, d)| Layer {
            outer_radius: r,
            diffusivity: d,
        })
        .collect();
    ModalRelease::new(&layers, lambda_max, n).unwrap()
}

/// Kernel volume (m^3) of the reference particle.
pub fn kernel_volume() -> f64 {
    4.0 / 3.0 * PI * RADII[0].powi(3)
}

/// Gauss-Legendre nodes and weights on [-1, 1] by Newton iteration on `P_n`.
fn gauss_legendre(n: usize) -> (Vec<f64>, Vec<f64>) {
    let (mut x, mut w) = (vec![0.0; n], vec![0.0; n]);
    for i in 0..n {
        let mut z = (PI * (i as f64 + 0.75) / (n as f64 + 0.5)).cos();
        let mut dp = 1.0;
        for _ in 0..100 {
            let (mut p0, mut p1) = (1.0, z);
            for k in 2..=n {
                let p2 = ((2 * k - 1) as f64 * z * p1 - (k - 1) as f64 * p0) / k as f64;
                p0 = p1;
                p1 = p2;
            }
            dp = n as f64 * (z * p1 - p0) / (z * z - 1.0);
            let dz = p1 / dp;
            z -= dz;
            if dz.abs() < 1e-15 {
                break;
            }
        }
        x[i] = z;
        w[i] = 2.0 / ((1.0 - z * z) * dp * dp);
    }
    (x, w)
}

/// Integral of a vector function over [lo, hi] (s) by 16-point Gauss-Legendre rules in the
/// variable `u = sqrt(theta)`, `int f(theta) dtheta = int f(u^2) 2 u du`. The release of the
/// uniform sphere grows as `sqrt(theta)` from zero, so the integrand is smooth in `u`, also at
/// `lo = 0`. `rate` (s^-1) is the largest rate of the integrand (`lambda + 100 a_1`): the
/// panel width in `u` is at most `u_cap = 0.3 / sqrt(rate)` (s^0.5) while `u < sqrt(60 /
/// rate)` (where `exp(-rate u^2)` is not yet below 1e-26), and otherwise the panel doubles its
/// end point (the integrands are then smooth in the logarithm of the time). The first panel
/// is `[0, min(0.01 sqrt(hi), u_cap)]` when `lo = 0`.
fn integrate3(lo: f64, hi: f64, rate: f64, f: &dyn Fn(f64) -> [f64; 3]) -> [f64; 3] {
    let mut out = [0.0; 3];
    if hi <= lo {
        return out;
    }
    let (u_lo, u_hi) = (lo.sqrt(), hi.sqrt());
    let (u_cap, u_cut) = (0.3 / rate.sqrt(), (60.0 / rate).sqrt());
    let (x, w) = gauss_legendre(16);
    let mut a = u_lo;
    let mut b = if lo > 0.0 {
        2.0 * a
    } else {
        (0.01 * u_hi).min(u_cap)
    };
    while a < u_hi {
        b = b.min(u_hi);
        let (mid, half) = (0.5 * (a + b), 0.5 * (b - a));
        for (xi, wi) in x.iter().zip(&w) {
            let u = mid + half * xi;
            let v = f(u * u);
            for k in 0..3 {
                out[k] += half * wi * 2.0 * u * v[k];
            }
        }
        a = b;
        b = if a < u_cut { a + a.min(u_cap) } else { 2.0 * a };
    }
    out
}

/// `Phi(x) = sum_{n >= 1} 1 / (n^2 (n^2 + x^2))`, in closed form:
/// `[pi^2 / 6 - (pi x coth(pi x) - 1) / (2 x^2)] / x^2`, and for `x < 0.5` (where the closed
/// form cancels) the series `sum_k (-1)^k x^(2k) zeta(2k + 4)`.
pub fn phi_sum(x: f64) -> f64 {
    if x < 0.5 {
        let mut term = 1.0;
        let mut sum = 0.0;
        for k in 0..60 {
            sum += term * zeta(2.0 * k as f64 + 4.0);
            term *= -x * x;
        }
        sum
    } else {
        let px = PI * x;
        let coth = 1.0 + 2.0 / (2.0 * px).exp_m1();
        (PI * PI / 6.0 - (px * coth - 1.0) / (2.0 * x * x)) / (x * x)
    }
}

/// Riemann zeta function for `s >= 4`: 30 terms and the Euler-Maclaurin tail.
fn zeta(s: f64) -> f64 {
    let m = 30.0_f64;
    let head: f64 = (1..30).map(|n| (n as f64).powf(-s)).sum();
    head + m.powf(1.0 - s) / (s - 1.0) + 0.5 * m.powf(-s) + s * m.powf(-s - 1.0) / 12.0
        - s * (s + 1.0) * (s + 2.0) * m.powf(-s - 3.0) / 720.0
}

/// The truth values (mol) a reference supplies for one case at one time.
#[derive(Debug, Clone, Copy)]
pub struct Truth {
    pub inventory: f64,
    pub released: f64,
    pub decayed: f64,
    /// Release rate (mol s^-1), where the reference has it.
    pub flux: Option<f64>,
    /// Computed bound on the rounding error of the reference itself, for the inventory, the
    /// released inventory (both mol) and the flux (mol s^-1): the running bound of a self
    /// reference. The closed-form reference of the physics module gives zeros: its agreement
    /// with direct sums to 1e-15 is tested separately.
    pub rounding: [f64; 3],
}

/// One case: initial inventory, decay constant and source history.
#[derive(Debug, Clone)]
pub struct Case {
    pub m0: f64,
    pub lambda: f64,
    pub source: PiecewiseSource,
    pub v_k: f64,
    /// Kernel radius and outer radius (m) of the particle.
    pub geometry: (f64, f64),
}

impl Case {
    pub fn decay(&self) -> Decay {
        if self.lambda == 0.0 {
            Decay::None
        } else {
            Decay::Uniform(self.lambda)
        }
    }
}

/// A reference: closed form or self reference.
pub trait Reference {
    fn label(&self) -> String;
    fn truth(&self, case: &Case, t: f64) -> Truth;
}

/// Self reference: the same evaluation with a much larger mode count.
pub struct SelfReference {
    pub series: ModalRelease,
    pub kind: &'static str,
}

impl Reference for SelfReference {
    fn label(&self) -> String {
        format!(
            "self reference: {} ({} modes)",
            self.kind,
            self.series.eigenvalues.len()
        )
    }
    fn truth(&self, case: &Case, t: f64) -> Truth {
        let st = KernelRun::new(
            &self.series,
            case.m0,
            case.v_k,
            case.source.clone(),
            &case.decay(),
        )
        .unwrap()
        .state(t)
        .unwrap();
        Truth {
            inventory: st.inventory,
            released: st.released,
            decayed: st.decayed,
            flux: Some(st.flux),
            rounding: [
                st.inventory_rounding_bound,
                st.release_rounding_bound,
                st.flux_rounding_bound,
            ],
        }
    }
}

/// Closed form: the uniform sphere (radius `r`, diffusivity `d`, absorbing surface) of the
/// physics module (`physics::reference::HomogeneousSphere`, series of retained and decayed
/// fraction, many terms), the source by superposition of unit kernel-step responses
/// (Duhamel) with a quadrature. Independent of the evaluation under test.
pub struct PhysicsReference {
    pub r: f64,
    pub d: f64,
}

impl PhysicsReference {
    fn a1(&self) -> f64 {
        PI * PI * self.d / (self.r * self.r)
    }

    /// (retained, decayed, released) fractions of a unit initial inventory after `theta` (s).
    fn parts(&self, lambda: f64, theta: f64) -> [f64; 3] {
        if theta <= 0.0 {
            return [1.0, 0.0, 0.0];
        }
        if self.a1() * theta < 6.0e-11 {
            // Series of more than 1e6 terms are not summed. For `a_1 theta < 6e-11` the
            // small-time expansion of the release of a uniform sphere,
            // `F_0 = 6 sqrt(a_1 theta) / pi^1.5 - 3 a_1 theta / pi^2` (next term of the order
            // `(a_1 theta)^1.5`, below 1e-15), the decay `lambda theta` (error below
            // `lambda theta F_0`) and the retained rest are used.
            let f0 = 6.0 * (self.a1() * theta).sqrt() / PI.powf(1.5)
                - 3.0 * self.a1() * theta / (PI * PI);
            let dec = lambda * theta;
            let ret = (-lambda * theta).exp() * (1.0 - f0);
            return [ret, dec, 1.0 - ret - dec];
        }
        let sphere = PhysicsSphere::new(self.r, self.d, lambda).unwrap();
        // every term beyond n has exp(-n^2 a_1 theta) below exp(-60)
        let n = (60.0 / (self.a1() * theta)).sqrt().ceil() as usize + 2;
        let ret = sphere.retained_fraction(theta, n).unwrap();
        let mut dec = sphere.decayed_fraction(theta, n).unwrap();
        if lambda > 0.0 {
            // the decayed series beyond n, where the exponential is negligible:
            // (6 / pi^2) lambda sum_{k > n} 1 / (k^2 (k^2 a_1 + lambda)); the sum over all k is
            // Phi(x) / a_1 with x^2 = lambda / a_1, the first n terms are summed here.
            let a1 = self.a1();
            let head: f64 = (1..=n)
                .map(|k| 1.0 / ((k * k) as f64 * ((k * k) as f64 * a1 + lambda)))
                .sum();
            dec += 6.0 / (PI * PI) * lambda * (phi_sum((lambda / a1).sqrt()) / a1 - head);
        }
        [ret, dec, 1.0 - ret - dec]
    }
}

impl Reference for PhysicsReference {
    fn label(&self) -> String {
        "closed form: uniform sphere of the physics module".to_string()
    }
    fn truth(&self, case: &Case, t: f64) -> Truth {
        let own = self.parts(case.lambda, t);
        let (mut inv, mut dec, mut rel) = (case.m0 * own[0], case.m0 * own[1], case.m0 * own[2]);
        let mut flux = 0.0;
        let b = case.source.breaks();
        for (k, &g) in case.source.rates().iter().enumerate() {
            if b[k] >= t || g == 0.0 {
                continue;
            }
            let e = t.min(b[k + 1]);
            let (hi, lo) = (t - b[k], t - e);
            let rate = case.lambda + 100.0 * self.a1();
            let parts = integrate3(lo, hi, rate, &|th| self.parts(case.lambda, th));
            inv += case.v_k * g * parts[0];
            dec += case.v_k * g * parts[1];
            rel += case.v_k * g * parts[2];
            flux +=
                case.v_k * g * (self.parts(case.lambda, hi)[2] - self.parts(case.lambda, lo)[2]);
        }
        Truth {
            inventory: inv,
            released: rel,
            decayed: dec,
            flux: (case.m0 == 0.0).then_some(flux),
            rounding: [0.0; 3],
        }
    }
}

/// Closed-form modes of the kernel step in a homogeneous particle (kernel `r_k`, outer
/// radius `r`, diffusivity `d`) as a series with `n` modes.
pub fn closed_form_series(r: f64, r_k: f64, d: f64, n: usize) -> ModalRelease {
    let sphere = ClosedCore::core(r, r_k, d, 1.0).unwrap();
    let eigenvalues: Vec<f64> = (1..=n).map(|k| sphere.eigenvalue(k)).collect();
    let weights: Vec<f64> = (1..=n).map(|k| sphere.weight(k)).collect();
    ModalRelease {
        lambda_covered: eigenvalues[n - 1],
        eigenvalues,
        weights,
        min_flux_ratio: f64::INFINITY,
    }
}

/// The uniform sphere as a mode series with `n` closed-form modes.
pub fn uniform_series(r: f64, d: f64, n: usize) -> ModalRelease {
    closed_form_series(r, r, d, n)
}

/// A source history shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceShape {
    None,
    /// Constant on for the whole span.
    On,
    /// On, then switched off at `3 t_scale`.
    OnOff,
    /// Off, then switched on at `3 t_scale`.
    OffOn,
    /// On, off from `t_scale`, on again from `3 t_scale`.
    Pulse,
    /// Thirteen pieces: twelve of length `2 t_scale` with rates `g` times a pattern (including
    /// zeros), then `g` to the end (a many-cycle history).
    Many,
}

impl SourceShape {
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::On => "on",
            Self::OnOff => "on-off",
            Self::OffOn => "off-on",
            Self::Pulse => "pulse",
            Self::Many => "many",
        }
    }

    /// History for the time scale `ts` (s) and rate `g` (mol m^-3 s^-1).
    pub fn history(self, ts: f64, g: f64) -> PiecewiseSource {
        let end = 1.0e4 * ts;
        match self {
            Self::None => PiecewiseSource::none(),
            Self::On => PiecewiseSource::new(vec![0.0, end], vec![g]).unwrap(),
            Self::OnOff => PiecewiseSource::new(vec![0.0, 3.0 * ts, end], vec![g, 0.0]).unwrap(),
            Self::OffOn => PiecewiseSource::new(vec![0.0, 3.0 * ts, end], vec![0.0, g]).unwrap(),
            Self::Pulse => {
                PiecewiseSource::new(vec![0.0, ts, 3.0 * ts, end], vec![g, 0.0, g]).unwrap()
            }
            Self::Many => {
                let pattern = [
                    1.0, 0.0, 0.5, 2.0, 0.0, 1.0, 0.1, 3.0, 0.0, 1.0, 0.5, 0.0, 1.0,
                ];
                let mut breaks: Vec<f64> = (0..=12).map(|k| 2.0 * ts * k as f64).collect();
                breaks.push(end);
                let rates = pattern.iter().map(|p| p * g).collect();
                PiecewiseSource::new(breaks, rates).unwrap()
            }
        }
    }
}

/// Log-spaced times from `lo` to `hi` (s), `per_decade` per decade.
pub fn log_times(lo: f64, hi: f64, per_decade: f64) -> Vec<f64> {
    let n = ((hi / lo).log10() * per_decade).ceil() as usize;
    (0..=n)
        .map(|i| lo * (hi / lo).powf(i as f64 / n as f64))
        .collect()
}

/// The quantities of the table (the drawn ones of `Quantity`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Q {
    Inventory,
    Release,
    Flux,
}

/// One measured point.
#[derive(Debug, Clone)]
pub struct Point {
    pub n: usize,
    pub q: Q,
    pub t: f64,
    /// `t / t_v`.
    pub t_over_tv: f64,
    pub lambda: f64,
    pub source: SourceShape,
    pub value: f64,
    pub truth: f64,
    /// `|value - truth|` (mol or mol s^-1).
    pub error: f64,
    /// The computed bound (52) of the same quantity without the floor term (same unit).
    pub bound_no_floor: f64,
    /// `accuracy_floor()` of the series (fraction), and the scale of the quantity so that
    /// `floor * scale` is the floor in the unit of `error`.
    pub floor: f64,
    pub scale: f64,
    /// The estimated error `e_ext` of the production rule (`ModalState::estimated_errors`),
    /// same unit as `error`.
    pub estimated: f64,
    /// The computed rounding bound of the evaluation under test (same unit as `error`), part
    /// of `estimated`.
    pub rounding: f64,
    /// The computed rounding bound of the reference (same unit as `error`).
    pub truth_rounding: f64,
}

impl Point {
    /// `(error - bound_no_floor - rounding) / (floor * scale)`: the excess of the error over
    /// the computed model bound (52) or (53) and the computed rounding bound, in units of the
    /// floor term of the rule, which is what the factors `EXT_FACTOR_*` have to cover (the
    /// error of the series). The reference's own rounding is removed from the error first.
    pub fn excess(&self) -> f64 {
        (self.error - self.truth_rounding - self.bound_no_floor - self.rounding)
            / (self.floor * self.scale)
    }

    /// The estimate holds at this point: the error does not exceed the estimate by more than
    /// the rounding error of the reference.
    pub fn within_estimate(&self) -> bool {
        self.error <= self.estimated + self.truth_rounding
    }
}

/// Measure the quantities of a series against the truth at one time (`f_now` of the code under
/// measurement is [`EARLY_RELEASE_FLOOR_FACTOR`]; its floor term is removed from the bound
/// `bound_no_floor`).
pub fn measure(
    series: &ModalRelease,
    case: &Case,
    shape: SourceShape,
    truth: &Truth,
    t: f64,
) -> Vec<Point> {
    let f_now = EARLY_RELEASE_FLOOR_FACTOR;
    let run = KernelRun::new(
        series,
        case.m0,
        case.v_k,
        case.source.clone(),
        &case.decay(),
    )
    .unwrap();
    let st: ModalState = run.state(t).unwrap();
    let floor = series.accuracy_floor();
    let delta0 = (st.early_release - f_now * floor).max(0.0);
    let k = delta0 / st.early_release;
    let est = st.estimated_errors(floor);
    let mut pts = Vec::new();
    let mk = |q: Q, value: f64, tr: f64, b: f64, scale: f64, estimated: f64, own: f64| Point {
        n: series.eigenvalues.len(),
        q,
        t,
        t_over_tv: t / st.relax_time,
        lambda: case.lambda,
        source: shape,
        value,
        truth: tr,
        error: (value - tr).abs(),
        bound_no_floor: b,
        floor,
        scale,
        estimated,
        rounding: own,
        truth_rounding: 0.0,
    };
    let [ri, rr, rf] = truth.rounding;
    let mut inv = mk(
        Q::Inventory,
        st.inventory,
        truth.inventory,
        k * st.inventory_error_bound,
        st.age_input,
        est.inventory,
        st.inventory_rounding_bound,
    );
    inv.truth_rounding = ri;
    pts.push(inv);
    let mut rel = mk(
        Q::Release,
        st.released,
        truth.released,
        k * st.release_error_bound,
        st.age_input,
        est.released,
        st.release_rounding_bound,
    );
    rel.truth_rounding = rr;
    pts.push(rel);
    if let (Some(fl), Some(fb), Some(ef)) = (truth.flux, st.flux_error_bound, est.flux) {
        if st.flux_scale > 0.0 {
            let mut p = mk(
                Q::Flux,
                st.flux,
                fl,
                k * fb,
                st.flux_scale,
                ef,
                st.flux_rounding_bound,
            );
            p.truth_rounding = rf;
            pts.push(p);
        }
    }
    pts
}

/// A geometry of the designed set.
pub struct Geometry {
    pub name: String,
    /// Series of `n` modes of the geometry under test.
    pub build: Box<dyn Fn(usize) -> ModalRelease>,
    /// Builds the reference (lazily: a self reference with many modes is costly).
    pub reference: Box<dyn Fn() -> Box<dyn Reference>>,
    /// Slowest decay rate `alpha_1` (s^-1) of the geometry.
    pub alpha_1: f64,
    pub v_k: f64,
    /// Kernel radius and outer radius (m).
    pub radii: (f64, f64),
    /// Earliest time of the set as a multiple of 1 / alpha_1 (0: from the cover).
    pub earliest: f64,
    /// The diffusivities (m^2 s^-1) of the five layers, as text for the table.
    pub diffusivities: String,
}

/// Diffusivity (m^2 s^-1) of the registered Arrhenius law `d0 exp(-q / (R T))`.
pub fn arrhenius(d0: f64, q_j_mol: f64, t_celsius: f64) -> f64 {
    d0 * (-q_j_mol / (8.314 * (t_celsius + 273.15))).exp()
}

/// The five designed geometries and diffusivity sets.
pub fn geometries(cfg: &SetConfig) -> Vec<Geometry> {
    let mut out: Vec<Geometry> = Vec::new();
    let vk = kernel_volume();
    // 1. uniform sphere (physics module closed form), closed-form modes under test
    {
        let (r, d) = (RADII[0], 1.0e-14);
        out.push(Geometry {
            name: "uniform sphere".to_string(),
            build: Box::new(move |n| uniform_series(r, d, n)),
            reference: Box::new(move || Box::new(PhysicsReference { r, d })),
            alpha_1: PI * PI * d / (r * r),
            v_k: vk,
            radii: (r, r),
            earliest: 1.0e-2,
            diffusivities: format!("one sphere of radius 175 um, D = {d:.1e}"),
        });
    }
    // 2. kernel step in a homogeneous five-layer particle: shooting modes under test,
    //    closed-form modes (many) as the reference
    {
        let d = 1.0e-13;
        let closed_modes = cfg.closed_modes;
        out.push(Geometry {
            name: "homogeneous particle".to_string(),
            build: Box::new(move |n| layered_series([d; 5], n)),
            reference: Box::new(move || {
                Box::new(SelfReference {
                    series: closed_form_series(RADII[4], RADII[0], d, closed_modes),
                    kind: "closed-form modes",
                })
            }),
            alpha_1: ClosedCore::core(RADII[4], RADII[0], d, 1.0)
                .unwrap()
                .eigenvalue(1),
            v_k: vk,
            radii: (RADII[0], RADII[4]),
            earliest: 0.0,
            diffusivities: format!("all five layers {d:.1e}"),
        });
    }
    // 3, 4. reference particle at two temperatures: registered Ag-in-UO2, LTI PyC and SiC
    //    correlations (data/diffusivity/arrhenius.csv, T978 rows A1-04, A2-03, A3-05); the
    //    buffer has no registered Ag law and takes the kernel value (a test choice)
    for t_c in [1000.0, 1600.0] {
        let d_k = arrhenius(6.7e-9, 165.0e3, t_c);
        let d_py = arrhenius(5.3e-9, 154.0e3, t_c);
        let d_sic = arrhenius(3.6e-9, 215.0e3, t_c);
        let d = [d_k, d_k, d_py, d_sic, d_py];
        out.push(layered(
            &format!("reference particle {t_c:.0} C"),
            RADII,
            d,
            cfg.reference_modes,
        ));
    }
    // 5. strongly contrasted set (test values, not a correlation): a ratio of 1e7
    out.push(layered(
        "strongly contrasted",
        RADII,
        [1.0e-11, 1.0e-9, 1.0e-12, 1.0e-16, 1.0e-12],
        cfg.reference_modes,
    ));
    // 6. other radii (the geometries of the second verification of the estimate): five equal
    //    layers on radii 200 to 450 um, shooting modes under test, closed-form modes as the
    //    reference
    {
        let radii = [200.0e-6, 260.0e-6, 300.0e-6, 380.0e-6, 450.0e-6];
        let d = 2.0e-13;
        let closed_modes = cfg.closed_modes;
        out.push(Geometry {
            name: "equal layers 200 to 450 um".to_string(),
            build: Box::new(move |n| layered_series_on(radii, [d; 5], n)),
            reference: Box::new(move || {
                Box::new(SelfReference {
                    series: closed_form_series(radii[4], radii[0], d, closed_modes),
                    kind: "closed-form modes",
                })
            }),
            alpha_1: ClosedCore::core(radii[4], radii[0], d, 1.0)
                .unwrap()
                .eigenvalue(1),
            v_k: 4.0 / 3.0 * PI * radii[0].powi(3),
            radii: (radii[0], radii[4]),
            earliest: 0.0,
            diffusivities: format!("all five layers {d:.1e}"),
        });
    }
    // 7. a contrasted layered particle on radii 250 to 465 um (kernel fraction 0.54)
    out.push(layered(
        "layered 250 to 465 um",
        [250.0e-6, 350.0e-6, 390.0e-6, 425.0e-6, 465.0e-6],
        [2.0e-13, 5.0e-12, 1.0e-13, 1.0e-15, 1.0e-13],
        cfg.reference_modes,
    ));
    // 8. kernel step with closed-form modes (series floor at rounding level) in a particle of
    //    kernel 160 um and outer radius 400 um, D = 1e-12
    {
        let (r_k, r, d) = (160.0e-6, 400.0e-6, 1.0e-12);
        let closed_modes = cfg.closed_modes;
        out.push(Geometry {
            name: "kernel step 160 to 400 um".to_string(),
            build: Box::new(move |n| closed_form_series(r, r_k, d, n)),
            reference: Box::new(move || {
                Box::new(SelfReference {
                    series: closed_form_series(r, r_k, d, closed_modes),
                    kind: "closed-form modes",
                })
            }),
            alpha_1: ClosedCore::core(r, r_k, d, 1.0).unwrap().eigenvalue(1),
            v_k: 4.0 / 3.0 * PI * r_k.powi(3),
            radii: (r_k, r),
            earliest: 0.0,
            diffusivities: format!("one material {d:.1e}"),
        });
    }
    // 9. a uniform sphere of 300 um with closed-form modes (floor at rounding level)
    {
        let (r, d) = (300.0e-6, 5.0e-15);
        let closed_modes = cfg.closed_modes;
        out.push(Geometry {
            name: "uniform sphere 300 um".to_string(),
            build: Box::new(move |n| uniform_series(r, d, n)),
            reference: Box::new(move || {
                Box::new(SelfReference {
                    series: uniform_series(r, d, closed_modes),
                    kind: "closed-form modes",
                })
            }),
            alpha_1: PI * PI * d / (r * r),
            v_k: 4.0 / 3.0 * PI * r.powi(3),
            radii: (r, r),
            earliest: 0.0,
            diffusivities: format!("one sphere of radius 300 um, D = {d:.1e}"),
        });
    }
    out
}

fn layered(name: &str, radii: [f64; 5], d: [f64; 5], reference_modes: usize) -> Geometry {
    let alpha_1 = layered_series_on(radii, d, 1).eigenvalues[0];
    Geometry {
        name: name.to_string(),
        build: Box::new(move |n| layered_series_on(radii, d, n)),
        reference: Box::new(move || {
            Box::new(SelfReference {
                series: layered_series_on(radii, d, reference_modes),
                kind: "shooting modes",
            })
        }),
        alpha_1,
        v_k: 4.0 / 3.0 * PI * radii[0].powi(3),
        radii: (radii[0], radii[4]),
        earliest: 0.0,
        diffusivities: format!(
            "D_kernel {:.2e}, D_buffer {:.2e}, D_IPyC {:.2e}, D_SiC {:.2e}, D_OPyC {:.2e}",
            d[0], d[1], d[2], d[3], d[4]
        ),
    }
}

/// The cases of one geometry: (source shape, initial inventory), and the decay constants.
pub fn case_shapes() -> Vec<(SourceShape, f64)> {
    vec![
        (SourceShape::None, 1.0),
        (SourceShape::On, 0.0),
        (SourceShape::On, 1.0),
        (SourceShape::OnOff, 0.0),
        (SourceShape::OffOn, 1.0),
        (SourceShape::Pulse, 0.0),
        (SourceShape::Many, 1.0),
    ]
}

/// Decay constants of the designed set (s^-1): none, and the registered nuclides of the
/// configuration (the full set spans the longest, Cs-137, to the shortest, Ag-110, half-life).
pub fn decay_set(cfg: &SetConfig) -> Vec<f64> {
    let all = registered_decay_constants();
    let pick = |name: &str| all.iter().find(|(n, _)| n == name).unwrap().1;
    let mut out = vec![0.0];
    out.extend(cfg.nuclides.iter().map(|n| pick(n)));
    out
}

/// The points of one geometry (name with reference label) of a run.
pub type GeometryPoints = (String, Vec<Point>);

/// Visit every (geometry, case, time) of the designed set. The visitor gets the geometry
/// name with the reference label, the series of all mode counts, the case, its source shape,
/// the time and the reference truth at that time.
pub fn for_each_case(
    cfg: &SetConfig,
    mut visit: impl FnMut(&str, &[ModalRelease], &Case, SourceShape, f64, &Truth, &dyn Reference),
) {
    for geo in geometries(cfg) {
        if let Some(f) = &cfg.only {
            if !geo.name.contains(f.as_str()) {
                continue;
            }
        }
        let reference = (geo.reference)();
        let name = format!("{} [{}]", geo.name, reference.label());
        let series: Vec<ModalRelease> = cfg.mode_counts.iter().map(|&n| (geo.build)(n)).collect();
        let t_scale = 1.0 / geo.alpha_1;
        let t_lo = (0.3 * series[series.len() - 1].min_valid_time()).max(geo.earliest * t_scale);
        let times = log_times(t_lo, cfg.t_end * t_scale, cfg.per_decade);
        for lambda in decay_set(cfg) {
            for (shape, m0) in case_shapes() {
                if lambda == 0.0 && shape == SourceShape::None {
                    continue;
                }
                let case = Case {
                    m0,
                    lambda,
                    source: shape.history(t_scale, 1.0 / (geo.v_k * t_scale)),
                    v_k: geo.v_k,
                    geometry: geo.radii,
                };
                for &t in &times {
                    let truth = reference.truth(&case, t);
                    visit(&name, &series, &case, shape, t, &truth, reference.as_ref());
                }
            }
        }
    }
}

/// Run the whole designed set. Returns, per geometry (name with reference label), the
/// measured points.
pub fn run_set(cfg: &SetConfig) -> Vec<GeometryPoints> {
    let mut all: Vec<GeometryPoints> = Vec::new();
    for_each_case(cfg, |name, series, case, shape, t, truth, _| {
        if all.last().map(|(n, _)| n.as_str()) != Some(name) {
            all.push((name.to_string(), Vec::new()));
        }
        let points = &mut all.last_mut().expect("pushed above").1;
        for s in series {
            points.extend(measure(s, case, shape, truth, t));
        }
    });
    all
}

/// Result of [`run_mask_set`].
#[derive(Debug, Clone, Default)]
pub struct MaskReport {
    /// Values the mask was asked about (quantity, mode count, case, time).
    pub asked: usize,
    /// Of these, values drawn.
    pub drawn: usize,
    /// Of these, values withheld before `t_v`, withheld by the regime, withheld by the
    /// estimated error.
    pub before_valid: usize,
    pub outside_regime: usize,
    pub error_too_large: usize,
    /// Largest relative error against the reference over the drawn values.
    pub max_drawn_error: f64,
    /// Values the mask drew although their relative error exceeds the threshold.
    pub wrong_drawn: Vec<String>,
    /// Raw values (unmasked) whose relative error exceeds the threshold, and of these how many
    /// the mask withheld.
    pub raw_wrong: usize,
    pub raw_wrong_withheld: usize,
    /// Of the values asked and drawn, those that are `F_post`.
    pub post_event_asked: usize,
    pub post_event_drawn: usize,
}

/// Ask the production mask ([`KernelRun::drawn`]) for the release fraction, the inventory
/// fraction (both need `m0 > 0`), the generated release fraction and `beta` over the designed
/// set, and compare every drawn value with the reference.
pub fn run_mask_set(cfg: &SetConfig, threshold: f64) -> MaskReport {
    use super::extensions::{Drawn, Quantity, Withheld};
    let mut rep = MaskReport::default();
    for_each_case(cfg, |name, series, case, shape, t, truth, reference| {
        for s in series {
            let run = KernelRun::new(s, case.m0, case.v_k, case.source.clone(), &case.decay())
                .expect("valid run")
                .with_geometry(case.geometry.0, case.geometry.1);
            let total = case.m0 + case.v_k * case.source.generated_per_volume(t);
            let g_now = case.source.rate_at(t);
            let mut asks: Vec<(Quantity, f64)> = Vec::new();
            if case.m0 > 0.0 {
                asks.push((Quantity::InventoryFraction, truth.inventory / case.m0));
                asks.push((Quantity::ReleaseFraction, truth.released / case.m0));
            }
            if total > 0.0 {
                asks.push((Quantity::ReleaseFractionGenerated, truth.released / total));
            }
            if let (Some(fl), true) = (truth.flux, g_now > 0.0) {
                asks.push((Quantity::Beta, fl / (case.v_k * g_now)));
            }
            // F_post against the reference: the event at twice the relaxation time of this
            // mode count, the gained release over the inventory at the event
            let t_e = 2.0 * s.min_valid_time();
            if case.m0 > 0.0 && t >= t_e {
                let at_event = reference.truth(case, t_e);
                if at_event.inventory > 0.0 {
                    asks.push((
                        Quantity::PostEventRelease { t_event: t_e },
                        (truth.released - at_event.released) / at_event.inventory,
                    ));
                }
            }
            for (q, reference) in asks {
                rep.asked += 1;
                let is_post = matches!(q, Quantity::PostEventRelease { .. });
                rep.post_event_asked += usize::from(is_post);
                let raw = run.value(q, t).expect("valid").expect("defined");
                let rel = if reference != 0.0 {
                    (raw - reference).abs() / reference.abs()
                } else {
                    f64::INFINITY
                };
                if rel > threshold {
                    rep.raw_wrong += 1;
                }
                // the gate is open here: the table measures the rule itself
                let d = run.drawn_gated(q, t, true).expect("valid");
                match &d {
                    Drawn::Value(v) => {
                        rep.drawn += 1;
                        rep.post_event_drawn += usize::from(is_post);
                        let r = if reference != 0.0 {
                            (v - reference).abs() / reference.abs()
                        } else {
                            f64::INFINITY
                        };
                        rep.max_drawn_error = rep.max_drawn_error.max(r);
                        if r > threshold {
                            rep.wrong_drawn.push(format!(
                                "{name} N {} {q:?} lambda {:.2e} {} t {t:.3e}: relative error {r:.3e}",
                                s.eigenvalues.len(),
                                case.lambda,
                                shape.name()
                            ));
                        }
                    }
                    Drawn::Withheld(w) => {
                        if rel > threshold {
                            rep.raw_wrong_withheld += 1;
                        }
                        match w {
                            Withheld::BeforeMinValidTime => rep.before_valid += 1,
                            Withheld::OutsideEstimatedRegime(_) => rep.outside_regime += 1,
                            Withheld::ExtensionErrorTooLarge { .. } => rep.error_too_large += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
    });
    rep
}

/// Summary of one (geometry, mode count, quantity) cell of the table.
#[derive(Debug, Clone)]
pub struct Cell {
    pub geometry: String,
    pub n: usize,
    pub q: Q,
    pub points: usize,
    /// Largest `excess` (error over computed bound, in units of `floor * scale`).
    pub max_excess: f64,
    /// Largest `error / estimated` (must stay below 1).
    pub max_error_over_estimate: f64,
    /// `accuracy_floor()` of the series of this cell.
    pub floor: f64,
}

/// Table of [`Cell`]s of a run.
pub fn summarise(cfg: &SetConfig, runs: &[(String, Vec<Point>)]) -> Vec<Cell> {
    let mut cells = Vec::new();
    for (name, points) in runs {
        for &n in &cfg.mode_counts {
            for q in [Q::Inventory, Q::Release, Q::Flux] {
                let sel: Vec<&Point> = points.iter().filter(|p| p.n == n && p.q == q).collect();
                if sel.is_empty() {
                    continue;
                }
                cells.push(Cell {
                    geometry: name.clone(),
                    n,
                    q,
                    points: sel.len(),
                    max_excess: sel.iter().map(|p| p.excess()).fold(f64::MIN, f64::max),
                    max_error_over_estimate: sel
                        .iter()
                        .map(|p| p.error / (p.estimated + p.truth_rounding))
                        .fold(0.0, f64::max),
                    floor: sel[0].floor,
                });
            }
        }
    }
    cells
}

/// Errors of the shooting weights of the homogeneous particle (D = 1e-13 m^2 s^-1, five layers
/// of one material) against the closed-form weights: the sum and the maximum of
/// `|w_shooting - w_closed|` over the first `n` modes, and the `accuracy_floor()` of the series.
pub fn homogeneous_weight_errors(n: usize) -> (f64, f64, f64) {
    let d = 1.0e-13;
    let s = layered_series([d; 5], n);
    let c = closed_form_series(RADII[4], RADII[0], d, n);
    let diffs: Vec<f64> = s
        .weights
        .iter()
        .zip(&c.weights)
        .map(|(a, b)| (a - b).abs())
        .collect();
    (
        diffs.iter().sum(),
        diffs.iter().cloned().fold(0.0, f64::max),
        s.accuracy_floor(),
    )
}

/// The range of the parameters the set covers (what `REGIME_*` of `extensions` may claim).
#[derive(Debug, Clone, Copy)]
pub struct Coverage {
    pub modes: (usize, usize),
    pub lambda_over_alpha1_max: f64,
    pub alpha1_range: (f64, f64),
    pub alpha1_t_max: f64,
    pub pieces_max: usize,
    pub floor_max: f64,
}

/// Coverage of a configuration, from its parameters (no evaluation of the series is needed,
/// except `accuracy_floor()` of the series of the geometries).
pub fn coverage(cfg: &SetConfig) -> Coverage {
    let lambdas = decay_set(cfg);
    let lambda_max = lambdas.iter().cloned().fold(0.0, f64::max);
    let mut cov = Coverage {
        modes: (
            *cfg.mode_counts.iter().min().expect("mode counts"),
            *cfg.mode_counts.iter().max().expect("mode counts"),
        ),
        lambda_over_alpha1_max: 0.0,
        alpha1_range: (f64::INFINITY, 0.0),
        alpha1_t_max: cfg.t_end,
        pieces_max: SourceShape::Many.history(1.0, 1.0).rates().len(),
        floor_max: 0.0,
    };
    for geo in geometries(cfg) {
        cov.lambda_over_alpha1_max = cov.lambda_over_alpha1_max.max(lambda_max / geo.alpha_1);
        cov.alpha1_range.0 = cov.alpha1_range.0.min(geo.alpha_1);
        cov.alpha1_range.1 = cov.alpha1_range.1.max(geo.alpha_1);
        for &n in &cfg.mode_counts {
            cov.floor_max = cov.floor_max.max((geo.build)(n).accuracy_floor());
        }
    }
    cov
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::analytical::extensions::{
        EXT_FACTOR_FLUX, EXT_FACTOR_INVENTORY, EXT_FACTOR_RELEASE, MASK_RELATIVE_TOLERANCE,
        REGIME_ALPHA1_T_MAX, REGIME_KERNEL_FRACTION, REGIME_LAMBDA_OVER_ALPHA1_MAX, REGIME_MODES,
        REGIME_OUTER_RADIUS, REGIME_PIECES_MAX,
    };
    use std::sync::OnceLock;

    fn factor(q: Q) -> f64 {
        match q {
            Q::Inventory => EXT_FACTOR_INVENTORY,
            Q::Release => EXT_FACTOR_RELEASE,
            Q::Flux => EXT_FACTOR_FLUX,
        }
    }

    /// The test set, run once for all tests of this module.
    fn runs() -> &'static (SetConfig, Vec<GeometryPoints>) {
        static RUNS: OnceLock<(SetConfig, Vec<GeometryPoints>)> = OnceLock::new();
        RUNS.get_or_init(|| {
            let cfg = SetConfig::test_set();
            let r = run_set(&cfg);
            (cfg, r)
        })
    }

    fn all_points() -> Vec<&'static Point> {
        runs().1.iter().flat_map(|(_, p)| p.iter()).collect()
    }

    /// The points of the geometries whose reference carries a computed rounding bound (the self
    /// references). The reference of the physics module (the uniform sphere of 175 um) has none:
    /// its own rounding, a few floors of the largest scale, differs between builds (0.9 and 3.1
    /// floors of `Release` at 16 modes in a test build and in a release example), so its excess
    /// is the noise of the reference and says nothing about the series.
    fn bounded_reference_points() -> Vec<&'static Point> {
        runs()
            .1
            .iter()
            .filter(|(name, _)| !name.contains("physics module"))
            .flat_map(|(_, p)| p.iter())
            .collect()
    }

    #[test]
    fn the_set_is_as_designed() {
        let (cfg, runs) = runs();
        assert!(runs.len() >= 4, "geometries: {}", runs.len());
        assert!(cfg.mode_counts.len() >= 4);
        let lambdas = decay_set(cfg);
        assert!(lambdas.iter().filter(|&&l| l > 0.0).count() >= 3);
        let pts = all_points();
        assert!(pts.len() > 5000, "{}", pts.len());
        for q in [Q::Inventory, Q::Release, Q::Flux] {
            assert!(pts.iter().any(|p| p.q == q));
        }
        // sources off and on, switched, and many pieces
        for s in [
            SourceShape::None,
            SourceShape::On,
            SourceShape::OnOff,
            SourceShape::OffOn,
            SourceShape::Pulse,
            SourceShape::Many,
        ] {
            assert!(pts.iter().any(|p| p.source == s), "{s:?}");
        }
        // from below t_v to long times
        let (lo, hi) = pts.iter().fold((f64::INFINITY, 0.0f64), |(l, h), p| {
            (l.min(p.t_over_tv), h.max(p.t_over_tv))
        });
        assert!(lo < 0.5 && hi > 1.0e3, "t/t_v from {lo} to {hi}");
        // both kinds of reference
        assert!(runs.iter().any(|(n, _)| n.contains("closed form")));
        assert!(runs.iter().any(|(n, _)| n.contains("self reference")));
    }

    /// The estimate `e_ext` of the rule holds at every sampled point of the table.
    #[test]
    fn the_estimated_error_bounds_the_error_at_every_sampled_point() {
        for p in all_points() {
            assert!(
                p.within_estimate(),
                "{:?} N {} lambda {:.2e} {:?} t/t_v {:.3e}: error {:.3e} above estimate {:.3e}",
                p.q,
                p.n,
                p.lambda,
                p.source,
                p.t_over_tv,
                p.error,
                p.estimated
            );
        }
    }

    /// The factors of the floor terms carry the margin of 2 stated in Document 6 over the excess
    /// at the points whose reference has a computed rounding bound, and they are not far above
    /// what is observed at any point (a factor divided by 8 would be violated).
    #[test]
    fn the_factors_have_the_stated_margin_and_are_needed() {
        for q in [Q::Inventory, Q::Release, Q::Flux] {
            let excess_of = |pts: Vec<&Point>| {
                pts.iter()
                    .filter(|p| p.q == q)
                    .map(|p| p.excess())
                    .fold(f64::MIN, f64::max)
            };
            let bounded = excess_of(bounded_reference_points());
            assert!(
                2.0 * bounded <= factor(q),
                "{q:?}: observed excess {bounded}, factor {}",
                factor(q)
            );
            let any = excess_of(all_points());
            assert!(
                any > factor(q) / 8.0,
                "{q:?}: observed excess {any} does not need factor {}",
                factor(q)
            );
        }
    }

    /// Mutants of the bound fail: without the floor term, without the computed bound (52),
    /// (53), and with the floor factor divided by 8 the error exceeds the estimate somewhere.
    #[test]
    fn mutants_of_the_bound_are_caught() {
        let pts = all_points();
        for q in [Q::Inventory, Q::Release, Q::Flux] {
            let sel: Vec<&&Point> = pts.iter().filter(|p| p.q == q).collect();
            // floor term removed
            assert!(sel.iter().any(|p| p.error > p.bound_no_floor), "{q:?}");
            // computed bound removed
            assert!(
                sel.iter().any(|p| p.error > factor(q) * p.floor * p.scale),
                "{q:?}"
            );
            // floor factor divided by 8
            assert!(
                sel.iter()
                    .any(|p| p.error > p.bound_no_floor + factor(q) / 8.0 * p.floor * p.scale),
                "{q:?}"
            );
            // computed bound halved (valid only where the bound is not rounding): the error is
            // within a factor 2 of the computed bound somewhere
            assert!(
                sel.iter().any(|p| {
                    p.bound_no_floor > 100.0 * p.floor * p.scale
                        && p.error > 0.5 * p.bound_no_floor + factor(q) * p.floor * p.scale
                }),
                "{q:?}"
            );
        }
    }

    /// The production mask draws a value only where its relative error against the reference is
    /// at most 5 %, and withholds every unmasked value whose error is larger.
    #[test]
    fn the_mask_draws_only_values_within_the_threshold() {
        let rep = run_mask_set(&SetConfig::test_set(), MASK_RELATIVE_TOLERANCE);
        assert!(rep.wrong_drawn.is_empty(), "{:?}", rep.wrong_drawn);
        assert!(rep.max_drawn_error <= MASK_RELATIVE_TOLERANCE);
        // not vacuous: many values are drawn, early ones are withheld, and the unmasked values
        // that are wrong exist and are all withheld
        assert!(rep.drawn > 1000, "{rep:?}");
        assert!(rep.before_valid > 100, "{rep:?}");
        assert!(rep.error_too_large > 100, "{rep:?}");
        assert!(rep.raw_wrong > 100, "{rep:?}");
        assert_eq!(rep.raw_wrong, rep.raw_wrong_withheld, "{rep:?}");
        // modes below the measured range (8 of the test set) are outside the regime
        assert!(rep.outside_regime > 0, "{rep:?}");
        // F_post is in the comparison against the reference and is drawn where it is accurate
        assert!(rep.post_event_asked > 100, "{rep:?}");
        assert!(rep.post_event_drawn > 20, "{rep:?}");
    }

    /// Major 1 of the third verification round: the points (inside the regime, closed-form
    /// series whose floor is the rounding unit) at which the earlier estimate with an observed
    /// rounding factor was exceeded by plain summation. The estimate now carries a computed
    /// rounding bound and holds at each of them, at mode counts of 1500 and 2048.
    #[test]
    fn points_where_an_observed_rounding_factor_failed_are_inside_the_estimate() {
        let cfg = SetConfig::test_set();
        let lambda = |name: &str| {
            registered_decay_constants()
                .into_iter()
                .find(|(n, _)| n == name)
                .unwrap()
                .1
        };
        let geo_of = |name: &str| {
            geometries(&cfg)
                .into_iter()
                .find(|g| g.name.contains(name))
                .unwrap_or_else(|| panic!("{name}"))
        };
        // (geometry, N, nuclide, source, initial inventory, time as (multiple, unit))
        // unit 0: t_v of the series, unit 1: 1 / alpha_1
        let rows: [(&str, usize, &str, SourceShape, f64, f64, usize); 5] = [
            (
                "kernel step 160",
                1500,
                "Sr-90",
                SourceShape::None,
                1.0,
                1.0,
                0,
            ),
            (
                "uniform sphere 300",
                1500,
                "Ba-137m",
                SourceShape::None,
                1.0,
                1.003,
                0,
            ),
            (
                "kernel step 160",
                1500,
                "Ag-110",
                SourceShape::Pulse,
                0.0,
                999.0,
                1,
            ),
            (
                "kernel step 160",
                2048,
                "Sr-90",
                SourceShape::On,
                0.0,
                1.02,
                0,
            ),
            (
                "uniform sphere 300",
                2048,
                "Ag-110",
                SourceShape::OffOn,
                1.0,
                999.0,
                1,
            ),
        ];
        for (name, n, nuclide, shape, m0, mult, unit) in rows {
            let geo = geo_of(name);
            let series = (geo.build)(n);
            let t_scale = 1.0 / geo.alpha_1;
            let t = mult
                * if unit == 0 {
                    series.min_valid_time()
                } else {
                    t_scale
                };
            let case = Case {
                m0,
                lambda: lambda(nuclide),
                source: shape.history(t_scale, 1.0 / (geo.v_k * t_scale)),
                v_k: geo.v_k,
                geometry: geo.radii,
            };
            let reference = (geo.reference)();
            let truth = reference.truth(&case, t);
            let pts = measure(&series, &case, shape, &truth, t);
            assert!(!pts.is_empty());
            for p in &pts {
                assert!(
                    p.within_estimate(),
                    "{name} N {n} {nuclide} {shape:?} t {t:.3e}: {:?} error {:.3e} estimate {:.3e} + reference {:.3e}",
                    p.q, p.error, p.estimated, p.truth_rounding
                );
                // the rounding bound is a real part of the estimate here, not zero
                assert!(p.rounding > 0.0, "{:?}", p.q);
            }
        }
    }

    /// The regime constants of the extension mask do not claim more than the full set covers.
    #[test]
    fn the_regime_lies_inside_the_coverage_of_the_full_set() {
        let full = SetConfig::full_set();
        let lambdas = decay_set(&full);
        let lambda_max = lambdas.iter().cloned().fold(0.0, f64::max);
        assert!(REGIME_MODES.0 >= *full.mode_counts.iter().min().unwrap());
        assert!(REGIME_MODES.1 <= *full.mode_counts.iter().max().unwrap());
        assert!(REGIME_ALPHA1_T_MAX <= full.t_end);
        assert!(REGIME_PIECES_MAX <= SourceShape::Many.history(1.0, 1.0).rates().len());
        // the slowest mode rate of every geometry of the set
        let mut worst_ratio = 0.0f64;
        let mut outer = (f64::INFINITY, 0.0f64);
        let mut fraction = (f64::INFINITY, 0.0f64);
        for geo in geometries(&full) {
            worst_ratio = worst_ratio.max(lambda_max / geo.alpha_1);
            outer = (outer.0.min(geo.radii.1), outer.1.max(geo.radii.1));
            let f = geo.radii.0 / geo.radii.1;
            fraction = (fraction.0.min(f), fraction.1.max(f));
        }
        // the geometry the mask accepts is the hull of the measured geometries
        let tol = 1.0e-9;
        assert!(REGIME_OUTER_RADIUS.0 >= outer.0 * (1.0 - tol));
        assert!(REGIME_OUTER_RADIUS.1 <= outer.1 * (1.0 + tol));
        assert!(REGIME_KERNEL_FRACTION.0 >= fraction.0 * (1.0 - tol));
        assert!(REGIME_KERNEL_FRACTION.1 <= fraction.1 * (1.0 + tol));
        assert!(
            REGIME_LAMBDA_OVER_ALPHA1_MAX <= worst_ratio,
            "regime {REGIME_LAMBDA_OVER_ALPHA1_MAX:e}, covered {worst_ratio:e}"
        );
    }

    /// The weight errors of the shooting solution of a homogeneous particle stay within a few
    /// floors in sum (the basis of the factor of `Delta`).
    #[test]
    fn weight_errors_of_the_homogeneous_particle_are_a_few_floors() {
        for n in [16, 64, 256] {
            let (sum, max, floor) = homogeneous_weight_errors(n);
            assert!(
                max <= 1.5 * floor,
                "N {n}: max {max:.2e}, floor {floor:.2e}"
            );
            assert!(
                sum <= EARLY_RELEASE_FLOOR_FACTOR * floor,
                "N {n}: sum {sum:.2e}, floor {floor:.2e}"
            );
        }
    }

    /// The Gauss-Legendre rule, the closed-form sum of the reference and the small-time branch
    /// of the closed form agree with their definitions.
    #[test]
    fn helpers_of_the_closed_form_reference() {
        // Phi(x) against a direct sum, both branches of the closed form
        for x in [0.01, 0.3, 0.49, 0.5, 1.0, 3.6, 12.0] {
            // summed from the small terms to the large ones (less rounding)
            let direct: f64 = (1..2_000_000)
                .rev()
                .map(|n| {
                    let n2 = (n as f64) * (n as f64);
                    1.0 / (n2 * (n2 + x * x))
                })
                .sum();
            let tail = 1.0 / (3.0 * 2.0e6_f64.powi(3));
            assert!(
                (phi_sum(x) - direct - tail).abs() < 3.0e-14 * phi_sum(x).max(1.0),
                "x {x}: {} vs {direct}",
                phi_sum(x)
            );
        }
        // the small-time branch against the series summed directly at the boundary
        let r = PhysicsReference {
            r: RADII[0],
            d: 1.0e-14,
        };
        let theta = 5.9e-11 / r.a1();
        let small = r.parts(1.0e-8, theta);
        let direct = direct_parts(r.a1(), 1.0e-8, theta, 2_000_000);
        assert!(
            (small[0] - direct[0]).abs() < 1.0e-15,
            "{small:?} vs {direct:?}"
        );
        assert!(
            (small[1] - direct[1]).abs() < 1.0e-15,
            "{small:?} vs {direct:?}"
        );
        // the quadrature in u = sqrt(theta) integrates sqrt(theta) exactly enough
        let v = integrate3(0.0, 4.0, 1.0, &|th| [th.sqrt(), 1.0, th]);
        assert!((v[0] - 16.0 / 3.0).abs() < 1e-12 && (v[1] - 4.0).abs() < 1e-12);
        assert!((v[2] - 8.0).abs() < 1e-12);
    }

    /// The retained and decayed fractions of the uniform sphere by direct summation of the
    /// series of the physics module's formulas, from the small terms to the large ones, with
    /// `n` terms and the analytical tail of the decayed series.
    fn direct_parts(a1: f64, lambda: f64, theta: f64, n: usize) -> [f64; 2] {
        let (mut ret, mut dec) = (0.0, 0.0);
        for k in (1..=n).rev() {
            let k2 = (k as f64) * (k as f64);
            let a = k2 * a1 + lambda;
            ret += (-a * theta).exp() / k2;
            dec += lambda / a * (-(-a * theta).exp_m1()) / k2;
        }
        let tail = lambda / (3.0 * a1 * (n as f64).powi(3));
        let c = 6.0 / (PI * PI);
        [c * ret, c * (dec + tail)]
    }

    /// The uniform-sphere reference agrees with the series summed directly to 1e-15 (rounding
    /// of the sum of 1e6 terms), and with the physics module's own sums to 1e-12 (the rounding
    /// of its naive forward sum of 4e5 terms).
    #[test]
    fn the_uniform_reference_agrees_with_the_physics_module() {
        let r = PhysicsReference {
            r: RADII[0],
            d: 1.0e-14,
        };
        let a1 = r.a1();
        for lambda in [0.0, 1.0e-8, 1.0e-6, 4.3e-5] {
            for x in [0.01, 0.2, 3.0, 40.0] {
                let theta = x / a1;
                let p = r.parts(lambda, theta);
                let direct = direct_parts(a1, lambda, theta, 1_000_000);
                // `p[0]` is the module's retained fraction (with exp(-lambda theta) outside)
                assert!(
                    (p[0] - direct[0]).abs() < 1.0e-15,
                    "ret {} vs {}",
                    p[0],
                    direct[0]
                );
                assert!(
                    (p[1] - direct[1]).abs() < 1.0e-15,
                    "dec {} vs {}",
                    p[1],
                    direct[1]
                );
                let sphere = PhysicsSphere::new(r.r, r.d, lambda).unwrap();
                let n = 400_000;
                let ret = sphere.retained_fraction(theta, n).unwrap();
                let dec = sphere.decayed_fraction(theta, n).unwrap();
                assert!((p[0] - ret).abs() < 1.0e-12, "ret {} vs module {ret}", p[0]);
                assert!((p[1] - dec).abs() < 1.0e-12, "dec {} vs module {dec}", p[1]);
            }
        }
    }

    /// The full designed set (a few minutes in a release build): the same assertions with
    /// five mode counts up to 2048, five nuclides and times to 1000 / alpha_1. Run it with
    /// `cargo test --release --lib -- --ignored full_set`.
    #[test]
    #[ignore]
    fn full_set_the_estimate_holds_and_has_the_stated_margin() {
        let cfg = SetConfig::full_set();
        let runs = run_set(&cfg);
        for (name, points) in &runs {
            for p in points {
                assert!(
                    p.within_estimate(),
                    "{name} {:?} N {} lambda {:.2e} {:?} t/t_v {:.3e}: error {:.3e} above {:.3e}",
                    p.q,
                    p.n,
                    p.lambda,
                    p.source,
                    p.t_over_tv,
                    p.error,
                    p.estimated
                );
            }
        }
        for q in [Q::Inventory, Q::Release, Q::Flux] {
            let m = runs
                .iter()
                .flat_map(|(_, p)| p.iter())
                .filter(|p| p.q == q)
                .map(|p| p.excess())
                .fold(f64::MIN, f64::max);
            assert!(
                2.0 * m <= factor(q),
                "{q:?}: excess {m}, factor {}",
                factor(q)
            );
        }
        let cov = coverage(&cfg);
        assert!(REGIME_LAMBDA_OVER_ALPHA1_MAX <= cov.lambda_over_alpha1_max);
        assert!(crate::analytical::extensions::REGIME_FLOOR_MAX >= cov.floor_max);
    }
}
