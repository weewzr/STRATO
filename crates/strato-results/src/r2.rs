//! The whole-curve agreement metric R2 of `metric-r2.md` (decisions 69, 72 and 74): the formula
//! with its two-pass sums, the numerical budget, the zero-variance test, the decay gate, the
//! reading-uncertainty interval, the display rule, the label texts, the reason codes with their
//! fixed captions and the seventeen ordered checks.
//!
//! This module holds the arithmetic and the vocabulary only. The decisions about which points
//! enter a curve and which checks a curve fails are in `r2curve`; the register, the model runs and
//! the files are in `r2run`. Every number is binary64, no fused multiply-add, sums in increasing
//! index order.

/// The definition id of every R2 record.
pub const DEFINITION_ID: &str = "R2_LINEAR_PER_CURVE_V1";
/// The smallest number of matched points of a curve (Section 3.2, a labelled convention).
pub const N_MIN: usize = 5;
/// The largest change of R2 that the discretisation error may cause; also the threshold of the
/// decay gate (Sections 2.3 and 2.5).
pub const BUDGET_MAX: f64 = 5e-4;
/// The largest number of uncertain coordinates for which all corners are evaluated (Section 4.2).
pub const EXHAUSTIVE_CORNERS_MAX: usize = 16;
/// The minus sign of every displayed string (U+2212).
pub const MINUS: char = '\u{2212}';

/// The sixteen reason codes of Section 3.1, each with its fixed one-sentence caption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasonCode {
    /// No diffusivity row for the nuclide.
    ModelNotConfigurable,
    /// The published values are bounds.
    UpperBound,
    /// Coating failure known or suspected.
    CoatingFailure,
    /// Compact, sphere or batch values that cannot be related to one intact particle.
    CompactNotRelatable,
    /// Release definition or denominator differs.
    IncompatibleDefinition,
    /// No registered temperature history.
    MissingTemperatureHistory,
    /// Essential conditions not stated.
    MissingConditions,
    /// The decay correction cannot be matched.
    DecayNotComparable,
    /// Fewer than `N_MIN` points.
    TooFewPoints,
    /// No variance beyond the uncertainty.
    ZeroVariance,
    /// Diffusivity used outside its documented range.
    OutsideCorrelationValidity,
    /// Model values not resolved.
    ModelNotResolved,
    /// Method B does not apply.
    MethodBNotApplicable,
    /// Method B masked.
    MethodBMasked,
    /// Inputs not verified.
    InputsNotVerified,
    /// Calculation not independently verified.
    NotIndependentlyVerified,
}

impl ReasonCode {
    /// All sixteen codes.
    pub const ALL: [ReasonCode; 16] = [
        ReasonCode::ModelNotConfigurable,
        ReasonCode::UpperBound,
        ReasonCode::CoatingFailure,
        ReasonCode::CompactNotRelatable,
        ReasonCode::IncompatibleDefinition,
        ReasonCode::MissingTemperatureHistory,
        ReasonCode::MissingConditions,
        ReasonCode::DecayNotComparable,
        ReasonCode::TooFewPoints,
        ReasonCode::ZeroVariance,
        ReasonCode::OutsideCorrelationValidity,
        ReasonCode::ModelNotResolved,
        ReasonCode::MethodBNotApplicable,
        ReasonCode::MethodBMasked,
        ReasonCode::InputsNotVerified,
        ReasonCode::NotIndependentlyVerified,
    ];

    /// The code as written in the records.
    pub fn code(self) -> &'static str {
        match self {
            ReasonCode::ModelNotConfigurable => "model_not_configurable",
            ReasonCode::UpperBound => "upper_bound",
            ReasonCode::CoatingFailure => "coating_failure",
            ReasonCode::CompactNotRelatable => "compact_not_relatable",
            ReasonCode::IncompatibleDefinition => "incompatible_definition",
            ReasonCode::MissingTemperatureHistory => "missing_temperature_history",
            ReasonCode::MissingConditions => "missing_conditions",
            ReasonCode::DecayNotComparable => "decay_not_comparable",
            ReasonCode::TooFewPoints => "too_few_points",
            ReasonCode::ZeroVariance => "zero_variance",
            ReasonCode::OutsideCorrelationValidity => "outside_correlation_validity",
            ReasonCode::ModelNotResolved => "model_not_resolved",
            ReasonCode::MethodBNotApplicable => "method_b_not_applicable",
            ReasonCode::MethodBMasked => "method_b_masked",
            ReasonCode::InputsNotVerified => "inputs_not_verified",
            ReasonCode::NotIndependentlyVerified => "not_independently_verified",
        }
    }

    /// The fixed caption (Section 3.1). Every caption is one sentence. The wording of an unset
    /// field is neutral (decision 74, ruling 2).
    pub fn caption(self) -> &'static str {
        match self {
            ReasonCode::ModelNotConfigurable => "R\u{b2} unavailable because no diffusivity data are registered for this nuclide, so no model curve exists.",
            ReasonCode::UpperBound => "R\u{b2} unavailable because the published values are upper bounds, not exact measurements.",
            ReasonCode::CoatingFailure => "R\u{b2} unavailable because coating failure is known or suspected, and it is outside the approved STRATO physics.",
            ReasonCode::CompactNotRelatable => "R\u{b2} unavailable because the values describe a whole compact, sphere or batch and cannot be related to one intact particle.",
            ReasonCode::IncompatibleDefinition => "R\u{b2} unavailable because the published release definition or normalisation differs from the model's.",
            ReasonCode::MissingTemperatureHistory => "R\u{b2} unavailable because the experimental temperature history is not available.",
            ReasonCode::MissingConditions => "R\u{b2} unavailable because essential experimental conditions are not stated in the source.",
            ReasonCode::DecayNotComparable => "R\u{b2} unavailable because the decay correction of the published values cannot be matched to the model.",
            ReasonCode::TooFewPoints => "R\u{b2} unavailable because the curve has too few time points for a whole-curve comparison.",
            ReasonCode::ZeroVariance => "R\u{b2} unavailable because the values do not vary beyond their stated uncertainty.",
            ReasonCode::OutsideCorrelationValidity => "R\u{b2} unavailable because the diffusivity data used are not documented for these conditions.",
            ReasonCode::ModelNotResolved => "R\u{b2} unavailable because the model values at the experimental times are not numerically resolved.",
            ReasonCode::MethodBNotApplicable => "No R\u{b2} for Method B because the analytical method does not apply to these conditions.",
            ReasonCode::MethodBMasked => "No R\u{b2} for Method B because the analytical solution is not resolved at every experimental time.",
            ReasonCode::InputsNotVerified => "R\u{b2} unavailable because the data used have not yet been independently verified.",
            ReasonCode::NotIndependentlyVerified => "R\u{b2} not yet shown because its calculation has not been independently verified.",
        }
    }
}

/// The seventeen checks, in the order of Section 3.1 (the derived order is the declaration order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Check {
    /// A diffusivity row exists for the nuclide.
    M0,
    /// Upper bounds and non-measured points.
    C1,
    /// Coating failure.
    C2,
    /// Specimen level.
    C3,
    /// Release definition and denominator.
    C4,
    /// Temperature history.
    C5,
    /// Essential conditions.
    C6,
    /// Decay data.
    D1,
    /// Number of points.
    C7,
    /// Zero variance.
    C8,
    /// Validity of the diffusivity rows.
    M1,
    /// Numerical resolution.
    M2,
    /// Applicability of Method B.
    B1,
    /// The Method B drawing mask.
    B2,
    /// The decay gate.
    D2,
    /// Verified inputs.
    C0,
    /// Independent recomputation.
    V1,
}

impl Check {
    /// All seventeen in order.
    pub const ALL: [Check; 17] = [
        Check::M0,
        Check::C1,
        Check::C2,
        Check::C3,
        Check::C4,
        Check::C5,
        Check::C6,
        Check::D1,
        Check::C7,
        Check::C8,
        Check::M1,
        Check::M2,
        Check::B1,
        Check::B2,
        Check::D2,
        Check::C0,
        Check::V1,
    ];

    /// The check id of the records.
    pub fn id(self) -> &'static str {
        match self {
            Check::M0 => "M0",
            Check::C1 => "C1",
            Check::C2 => "C2",
            Check::C3 => "C3",
            Check::C4 => "C4",
            Check::C5 => "C5",
            Check::C6 => "C6",
            Check::D1 => "D1",
            Check::C7 => "C7",
            Check::C8 => "C8",
            Check::M1 => "M1",
            Check::M2 => "M2",
            Check::B1 => "B1",
            Check::B2 => "B2",
            Check::D2 => "D2",
            Check::C0 => "C0",
            Check::V1 => "V1",
        }
    }

    /// The reason code a failure of this check gives (D1 and D2 share one).
    pub fn reason(self) -> ReasonCode {
        match self {
            Check::M0 => ReasonCode::ModelNotConfigurable,
            Check::C1 => ReasonCode::UpperBound,
            Check::C2 => ReasonCode::CoatingFailure,
            Check::C3 => ReasonCode::CompactNotRelatable,
            Check::C4 => ReasonCode::IncompatibleDefinition,
            Check::C5 => ReasonCode::MissingTemperatureHistory,
            Check::C6 => ReasonCode::MissingConditions,
            Check::D1 | Check::D2 => ReasonCode::DecayNotComparable,
            Check::C7 => ReasonCode::TooFewPoints,
            Check::C8 => ReasonCode::ZeroVariance,
            Check::M1 => ReasonCode::OutsideCorrelationValidity,
            Check::M2 => ReasonCode::ModelNotResolved,
            Check::B1 => ReasonCode::MethodBNotApplicable,
            Check::B2 => ReasonCode::MethodBMasked,
            Check::C0 => ReasonCode::InputsNotVerified,
            Check::V1 => ReasonCode::NotIndependentlyVerified,
        }
    }

    /// True for the checks of scope "curve" (the check table of Section 3.1; C0 is a curve check
    /// except for the rows only Method B uses).
    pub fn is_curve_level(self) -> bool {
        !matches!(
            self,
            Check::M1 | Check::M2 | Check::B1 | Check::B2 | Check::D2 | Check::V1
        )
    }
}

/// One matched point with the quantities R2 and its interval need.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// Register row id.
    pub id: String,
    /// The experimental value used (never rescaled, smoothed or interpolated).
    pub f_exp: f64,
    /// The model value at the exact step boundary of the aligned time (a window difference for a
    /// window point).
    pub f_model: f64,
    /// Half-width of the value reading uncertainty, absolute.
    pub u_value: Option<f64>,
    /// Half-width of the time reading uncertainty, h.
    pub u_time_h: Option<f64>,
    /// The model value at the lowest shifted time (null where `u_time_h` is 0 or absent).
    pub f_model_t_minus: Option<f64>,
    /// The model value at the highest shifted time.
    pub f_model_t_plus: Option<f64>,
    /// The shifted times themselves, h, after clipping (for the slope `s_i`).
    pub t_minus_h: Option<f64>,
    /// See `t_minus_h`.
    pub t_plus_h: Option<f64>,
    /// The discretisation-error estimate `e_i` of the model value.
    pub e_num: f64,
    /// True if the 5 % rule of science design Section 3.5 holds at this point.
    pub resolved: bool,
    /// True if the value was read from a figure.
    pub digitised: bool,
}

/// The mean and the two sums of squares of a set of points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sums {
    /// N.
    pub n: usize,
    /// Mean of the experimental values.
    pub mean: f64,
    /// `SS_tot`.
    pub ss_tot: f64,
    /// `SS_res`.
    pub ss_res: f64,
}

impl Sums {
    /// `R2 = 1 - SS_res / SS_tot` (not finite for `SS_tot = 0`; the callers test C8 first).
    pub fn r2(&self) -> f64 {
        1.0 - self.ss_res / self.ss_tot
    }
}

/// The two-pass sums of Section 1.1: the mean first, then `SS_tot` from the deviations, never
/// `sum F^2 - N Fbar^2`. Sums in increasing index order.
pub fn sums(f_exp: &[f64], f_model: &[f64]) -> Sums {
    let n = f_exp.len();
    let mut total = 0.0;
    for x in f_exp {
        total += *x;
    }
    let mean = total / n as f64;
    let mut ss_tot = 0.0;
    for x in f_exp {
        let d = *x - mean;
        ss_tot += d * d;
    }
    let mut ss_res = 0.0;
    for (x, m) in f_exp.iter().zip(f_model) {
        let r = *x - *m;
        ss_res += r * r;
    }
    Sums {
        n,
        mean,
        ss_tot,
        ss_res,
    }
}

/// The sums of the points as they stand.
pub fn point_sums(points: &[Point]) -> Sums {
    let fe: Vec<f64> = points.iter().map(|p| p.f_exp).collect();
    let fm: Vec<f64> = points.iter().map(|p| p.f_model).collect();
    sums(&fe, &fm)
}

/// The numerical budget `B_M` of Section 2.3.
pub fn budget(points: &[Point], ss_tot: f64) -> f64 {
    let mut cross = 0.0;
    let mut squares = 0.0;
    for p in points {
        cross += (p.f_exp - p.f_model).abs() * p.e_num;
        squares += p.e_num * p.e_num;
    }
    (2.0 * cross + squares) / ss_tot
}

/// Check C8 (Section 3.3): zero variance. Item 1 is a test at floating-point level; item 2 applies
/// only when every point has a stated uncertainty `u_e`.
pub fn zero_variance(points: &[Point]) -> bool {
    let s = point_sums(points);
    let (mut lo, mut hi, mut big) = (f64::INFINITY, f64::NEG_INFINITY, 0.0_f64);
    for p in points {
        lo = lo.min(p.f_exp);
        hi = hi.max(p.f_exp);
        big = big.max(p.f_exp.abs());
    }
    if !s.ss_tot.is_finite() || s.ss_tot == 0.0 || hi - lo <= 1e-12 * big {
        return true;
    }
    if points.iter().all(|p| p.u_value.is_some_and(|u| u > 0.0)) {
        let mut sum_u2 = 0.0;
        for p in points {
            let u = p.u_value.unwrap_or(0.0);
            sum_u2 += u * u;
        }
        return s.ss_tot <= sum_u2;
    }
    false
}

/// The decay gate of check D2 (Section 2.5): the change of R2 when every experimental value is
/// multiplied by `exp(lambda t_d,i)` (`lambda` per hour, `t_d` in hours). The scaled values are a
/// test only.
pub fn decay_gate_delta(points: &[Point], lambda_per_h: f64, t_d_h: &[f64]) -> f64 {
    let base = point_sums(points).r2();
    let fe: Vec<f64> = points
        .iter()
        .zip(t_d_h)
        .map(|(p, t)| p.f_exp * (lambda_per_h * *t).exp())
        .collect();
    let fm: Vec<f64> = points.iter().map(|p| p.f_model).collect();
    (sums(&fe, &fm).r2() - base).abs()
}

/// How the corners of the interval were chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerMethod {
    /// All `2^k` corners.
    Exhaustive,
    /// The two gradient corners.
    Gradient,
}

impl CornerMethod {
    /// The word of the record.
    pub fn word(self) -> &'static str {
        match self {
            CornerMethod::Exhaustive => "exhaustive",
            CornerMethod::Gradient => "gradient",
        }
    }
}

/// The reading-uncertainty interval of Section 4.2.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    /// Smallest R2 over the corners.
    pub r2_low: f64,
    /// Largest R2 over the corners.
    pub r2_high: f64,
    /// `U = sum_i (|g_i| u_e,i + |h_i| u_t,i)`.
    pub first_order_half_width: f64,
    /// How the corners were chosen.
    pub corner_method: CornerMethod,
    /// The number k of uncertain coordinates.
    pub uncertain: usize,
}

/// Why no interval exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoInterval {
    /// No coordinate is uncertain and no point was read from a figure (tabulated values).
    NotApplicable,
    /// A point was read from a figure without a recorded reading uncertainty, or a time shift
    /// has no model values.
    NotAvailable,
}

#[derive(Clone, Copy)]
enum Coord {
    Value(usize),
    Time(usize),
}

fn sign(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// The interval of R2 for the reading uncertainties of the points (Section 4.2): every uncertain
/// coordinate is moved to an end of its range and R2 is evaluated exactly there, with the mean and
/// `SS_tot` recomputed and the model at the shifted times.
///
/// # Errors
///
/// [`NoInterval`] where no interval exists.
pub fn interval(points: &[Point]) -> Result<Interval, NoInterval> {
    let uv = |p: &Point| p.u_value.unwrap_or(0.0);
    let ut = |p: &Point| p.u_time_h.unwrap_or(0.0);
    if points
        .iter()
        .any(|p| p.digitised && p.u_value.is_none() && p.u_time_h.is_none())
    {
        return Err(NoInterval::NotAvailable);
    }
    let mut coords: Vec<Coord> = Vec::new();
    for (i, p) in points.iter().enumerate() {
        if uv(p) > 0.0 {
            coords.push(Coord::Value(i));
        }
        if ut(p) > 0.0 {
            if p.f_model_t_minus.is_none() || p.f_model_t_plus.is_none() {
                return Err(NoInterval::NotAvailable);
            }
            coords.push(Coord::Time(i));
        }
    }
    if coords.is_empty() {
        return Err(if points.iter().any(|p| p.digitised) {
            NoInterval::NotAvailable
        } else {
            NoInterval::NotApplicable
        });
    }
    let base = point_sums(points);
    let r2 = base.r2();
    let at = |signs: &dyn Fn(Coord) -> f64| -> f64 {
        let mut fe: Vec<f64> = points.iter().map(|p| p.f_exp).collect();
        let mut fm: Vec<f64> = points.iter().map(|p| p.f_model).collect();
        for c in &coords {
            let s = signs(*c);
            match *c {
                Coord::Value(i) => fe[i] += s * uv(&points[i]),
                Coord::Time(i) => {
                    let p = &points[i];
                    if s > 0.0 {
                        fm[i] = p.f_model_t_plus.unwrap_or(p.f_model);
                    } else if s < 0.0 {
                        fm[i] = p.f_model_t_minus.unwrap_or(p.f_model);
                    }
                }
            }
        }
        sums(&fe, &fm).r2()
    };
    // The first-order quantities (the cross-check, and the gradient corners).
    let mut g = vec![0.0; points.len()];
    let mut h = vec![0.0; points.len()];
    let mut half_width = 0.0;
    for (i, p) in points.iter().enumerate() {
        let r = p.f_exp - p.f_model;
        g[i] = 2.0 / base.ss_tot * ((1.0 - r2) * (p.f_exp - base.mean) - r);
        let slope = match (p.t_minus_h, p.t_plus_h, p.f_model_t_minus, p.f_model_t_plus) {
            (Some(a), Some(b), Some(fa), Some(fb)) if ut(p) > 0.0 && b > a => (fb - fa) / (b - a),
            _ => 0.0,
        };
        h[i] = 2.0 / base.ss_tot * r * slope;
        half_width += g[i].abs() * uv(p) + h[i].abs() * ut(p);
    }
    let k = coords.len();
    let (low, high, method) = if k <= EXHAUSTIVE_CORNERS_MAX {
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for mask in 0u32..(1u32 << k) {
            let idx = |c: Coord| -> usize {
                coords
                    .iter()
                    .position(|x| match (x, &c) {
                        (Coord::Value(a), Coord::Value(b)) | (Coord::Time(a), Coord::Time(b)) => {
                            a == b
                        }
                        _ => false,
                    })
                    .unwrap_or(0)
            };
            let v = at(&|c| {
                if mask >> idx(c) & 1 == 1 {
                    1.0
                } else {
                    -1.0
                }
            });
            lo = lo.min(v);
            hi = hi.max(v);
        }
        (lo, hi, CornerMethod::Exhaustive)
    } else {
        let plus = at(&|c| match c {
            Coord::Value(i) => sign(g[i]),
            Coord::Time(i) => sign(h[i]),
        });
        let minus = at(&|c| match c {
            Coord::Value(i) => -sign(g[i]),
            Coord::Time(i) => -sign(h[i]),
        });
        (plus.min(minus), plus.max(minus), CornerMethod::Gradient)
    };
    Ok(Interval {
        r2_low: low,
        r2_high: high,
        first_order_half_width: half_width,
        corner_method: method,
        uncertain: k,
    })
}

/// R2 with every experimental value multiplied by `1 - u_sys` and by `1 + u_sys` (Section 4.3).
pub fn systematic_interval(points: &[Point], u_sys: f64) -> (f64, f64) {
    let fm: Vec<f64> = points.iter().map(|p| p.f_model).collect();
    let r = |factor: f64| {
        let fe: Vec<f64> = points.iter().map(|p| p.f_exp * factor).collect();
        sums(&fe, &fm).r2()
    };
    let (a, b) = (r(1.0 - u_sys), r(1.0 + u_sys));
    (a.min(b), a.max(b))
}

fn signed(x: f64, text: String) -> String {
    if x < 0.0 {
        format!("{MINUS}{text}")
    } else {
        text
    }
}

/// The displayed value of R2 (Section 7.2): rounded toward minus infinity, with a guard against
/// the binary representation. `1.00` only for `SS_res = 0`.
pub fn display(r2: f64, ss_res: f64) -> String {
    if ss_res == 0.0 {
        return "1.00".to_string();
    }
    let k = ((100.0 * r2 + 1e-9).floor()).min(99.0);
    if k >= -9999.0 {
        let ki = k as i64;
        return signed(
            k,
            format!("{}.{:02}", ki.unsigned_abs() / 100, ki.unsigned_abs() % 100),
        );
    }
    let mut n = r2.abs().log10().floor() as i32;
    let mut m = (r2 / 10f64.powi(n - 1) + 1e-9).floor() / 10.0;
    if m.abs() >= 10.0 {
        m /= 10.0;
        n += 1;
    }
    signed(m, format!("{:.1} \u{d7} 10^{n}", m.abs()))
}

/// The display string in LaTeX (`-1.3 \times 10^{3}`).
pub fn display_latex(display: &str) -> String {
    let d = display.replace(MINUS, "-");
    match d.split_once(" \u{d7} 10^") {
        Some((m, e)) => format!("{m} \\times 10^{{{e}}}"),
        None => d,
    }
}

/// The label of a method (Section 7.3): LaTeX and plain text. `method` is `A` or `B`.
pub fn label(method: char, display: &str) -> (String, String) {
    (
        format!(
            "R^{{2}}_{{\\mathrm{{{method}}}}} = {}",
            display_latex(display)
        ),
        format!("R\u{b2}_{method} = {display}"),
    )
}

/// A number rounded toward minus infinity (`up = false`) or plus infinity (`up = true`) to four
/// decimals, with U+2212 for a negative sign.
pub fn outward_four_decimals(x: f64, up: bool) -> String {
    let scaled = x * 1e4;
    let r = if up { scaled.ceil() } else { scaled.floor() } / 1e4;
    let r = if r == 0.0 { 0.0 } else { r };
    signed(r, format!("{:.4}", r.abs()))
}

/// The `interval_display` string (Section 7.1). `scope` is the scope text of the record.
pub fn interval_display(iv: &Interval, scope: &str) -> String {
    let mut s = format!(
        "R\u{b2} reading-uncertainty interval [{}, {}] ({scope})",
        outward_four_decimals(iv.r2_low, false),
        outward_four_decimals(iv.r2_high, true)
    );
    if iv.corner_method == CornerMethod::Gradient {
        s.push_str(" (corners chosen by gradient; not guaranteed extremes)");
    }
    s
}

/// The `systematic_interval_display` string (Section 7.1).
pub fn systematic_interval_display(u_sys: f64, low: f64, high: f64) -> String {
    format!(
        "R\u{b2} with systematic uncertainty \u{b1}{} % [{}, {}]",
        100.0 * u_sys,
        outward_four_decimals(low, false),
        outward_four_decimals(high, true)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(f_exp: f64, f_model: f64, u: Option<f64>) -> Point {
        Point {
            id: String::new(),
            f_exp,
            f_model,
            u_value: u,
            u_time_h: None,
            f_model_t_minus: None,
            f_model_t_plus: None,
            t_minus_h: None,
            t_plus_h: None,
            e_num: 1e-5,
            resolved: true,
            digitised: u.is_some(),
        }
    }

    /// Worked example 1 of the definition: illustrative numbers injected through a fixture, never
    /// through production output.
    fn example_1() -> Vec<Point> {
        let fe = [0.10, 0.20, 0.30, 0.40, 0.50];
        let fm = [0.12, 0.18, 0.33, 0.38, 0.52];
        fe.iter()
            .zip(fm)
            .map(|(e, m)| pt(*e, m, Some(0.005)))
            .collect()
    }

    fn example_2() -> Vec<Point> {
        [0.010, 0.015, 0.018, 0.020, 0.021]
            .iter()
            .map(|e| pt(*e, 2.0 * e, None))
            .collect()
    }

    #[test]
    fn worked_example_1_gives_0_975_and_the_stated_interval() {
        let p = example_1();
        let s = point_sums(&p);
        assert_eq!(s.n, 5);
        assert!((s.mean - 0.30).abs() < 1e-15);
        assert!((s.ss_tot - 0.10).abs() < 1e-15);
        assert!((s.ss_res - 2.5e-3).abs() < 1e-15);
        assert!((s.r2() - 0.975).abs() < 1e-12, "{}", s.r2());
        let b = budget(&p, s.ss_tot);
        assert!((b - 2.2005e-5).abs() < 1e-12, "{b}");
        assert!(!zero_variance(&p));
        assert_eq!(display(s.r2(), s.ss_res), "0.97");
        let (latex, plain) = label('A', "0.97");
        assert_eq!(latex, "R^{2}_{\\mathrm{A}} = 0.97");
        assert_eq!(plain, "R\u{b2}_A = 0.97");
        let iv = interval(&p).expect("an interval");
        assert_eq!(iv.corner_method, CornerMethod::Exhaustive);
        assert_eq!(iv.uncertain, 5);
        assert!((iv.r2_high - 0.98476828).abs() < 1e-7, "{}", iv.r2_high);
        assert!((iv.r2_low - 0.96279465).abs() < 1e-7, "{}", iv.r2_low);
        assert!((iv.first_order_half_width - 0.011).abs() < 1e-12);
        assert_eq!(
            interval_display(&iv, "reading uncertainty only"),
            "R\u{b2} reading-uncertainty interval [0.9627, 0.9848] (reading uncertainty only)"
        );
        // The extremes are the gradient corners: the gradient method gives the same ends.
        let gradient_signs = [1.0, -1.0, 1.0, -1.0, 1.0];
        let fe_hi: Vec<f64> = p
            .iter()
            .zip(gradient_signs)
            .map(|(q, g)| q.f_exp + g * 0.005)
            .collect();
        let fm: Vec<f64> = p.iter().map(|q| q.f_model).collect();
        assert!((sums(&fe_hi, &fm).r2() - iv.r2_high).abs() < 1e-15);
    }

    #[test]
    fn worked_example_2_gives_a_negative_value_with_its_display() {
        let p = example_2();
        let s = point_sums(&p);
        assert!((s.mean - 0.0168).abs() < 1e-15);
        assert!((s.ss_tot - 7.880e-5).abs() < 1e-12);
        assert!((s.ss_res - 1.490e-3).abs() < 1e-12);
        assert!((s.r2() + 17.9086294416).abs() < 1e-9, "{}", s.r2());
        assert!(!zero_variance(&p));
        let d = display(s.r2(), s.ss_res);
        assert_eq!(d, "\u{2212}17.91");
        let (latex, plain) = label('A', &d);
        assert_eq!(latex, "R^{2}_{\\mathrm{A}} = -17.91");
        assert_eq!(plain, "R\u{b2}_A = \u{2212}17.91");
        // Tabulated values without a stated uncertainty: no interval applies.
        assert_eq!(interval(&example_2()), Err(NoInterval::NotApplicable));
    }

    #[test]
    fn the_two_pass_sum_survives_a_large_offset() {
        // A one-pass formula `sum F^2 - N mean^2` loses every digit here; the two-pass sum does not.
        let off = 1.0e8;
        let fe: Vec<f64> = (0..5).map(|i| off + i as f64 * 1.0e-3).collect();
        let s = sums(&fe, &fe);
        assert!((s.ss_tot - 1.0e-5).abs() < 1e-10, "{}", s.ss_tot);
    }

    #[test]
    fn the_zero_variance_case_is_refused_at_both_levels() {
        // Item 1: identical values.
        let flat: Vec<Point> = (0..5).map(|_| pt(0.3, 0.31, None)).collect();
        assert!(zero_variance(&flat));
        // Item 2: the spread is smaller than the stated uncertainty.
        let v = [0.300, 0.301, 0.300, 0.302, 0.301];
        let small: Vec<Point> = v.iter().map(|x| pt(*x, 0.3, Some(0.01 * x))).collect();
        let s = point_sums(&small);
        assert!((s.ss_tot - 2.8e-6).abs() < 1e-12);
        assert!(zero_variance(&small));
        // The same values without a stated uncertainty pass item 1 only.
        let bare: Vec<Point> = v.iter().map(|x| pt(*x, 0.3, None)).collect();
        assert!(!zero_variance(&bare));
    }

    #[test]
    fn the_display_rule_covers_every_example_of_section_7_2() {
        let cases: [(f64, f64, &str); 14] = [
            (0.975, 1.0, "0.97"),
            (0.29, 1.0, "0.29"),
            (0.99999, 1.0, "0.99"),
            (1.0 - 1e-12, 1.0, "0.99"),
            (1.0, 1e-20, "0.99"),
            (0.97 - 5e-12, 1.0, "0.97"),
            (-0.003, 1.0, "\u{2212}0.01"),
            (-17.9086, 1.0, "\u{2212}17.91"),
            (-99.99, 1.0, "\u{2212}99.99"),
            (-99.995, 1.0, "\u{2212}1.0 \u{d7} 10^2"),
            (-100.0, 1.0, "\u{2212}1.0 \u{d7} 10^2"),
            (-999.5, 1.0, "\u{2212}1.0 \u{d7} 10^3"),
            (-1234.5, 1.0, "\u{2212}1.3 \u{d7} 10^3"),
            (0.0, 1.0, "0.00"),
        ];
        for (r2, ss_res, want) in cases {
            assert_eq!(display(r2, ss_res), want, "R2 = {r2}");
        }
        // The one case that shows 1.00.
        assert_eq!(display(1.0, 0.0), "1.00");
        assert_eq!(
            display_latex("\u{2212}1.3 \u{d7} 10^3"),
            "-1.3 \\times 10^{3}"
        );
        assert!(!display(-250.0, 1.0).contains("100.00"));
    }

    #[test]
    fn the_decay_gate_is_zero_without_decay_and_positive_with_it() {
        let p = example_1();
        let t = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert!(decay_gate_delta(&p, 0.0, &t) < 1e-15);
        assert!(decay_gate_delta(&p, 1e-2, &t) > BUDGET_MAX);
    }

    #[test]
    fn a_time_shift_uses_the_model_at_the_shifted_times() {
        let mut p = example_1();
        for q in &mut p {
            q.u_value = None;
            q.u_time_h = Some(1.0);
            q.f_model_t_minus = Some(q.f_model - 0.01);
            q.f_model_t_plus = Some(q.f_model + 0.01);
            q.t_minus_h = Some(0.0);
            q.t_plus_h = Some(2.0);
        }
        let iv = interval(&p).expect("an interval");
        assert!(iv.r2_low < 0.975 && iv.r2_high > 0.975);
        // A shift without model values at the shifted times has no interval.
        p[0].f_model_t_minus = None;
        assert_eq!(interval(&p), Err(NoInterval::NotAvailable));
    }

    #[test]
    fn a_figure_point_without_a_recorded_reading_uncertainty_gives_no_interval() {
        let mut p = example_1();
        p[2].u_value = None;
        assert_eq!(interval(&p), Err(NoInterval::NotAvailable));
    }

    #[test]
    fn many_uncertain_coordinates_use_the_gradient_corners() {
        let p: Vec<Point> = (0..20)
            .map(|i| {
                pt(
                    0.01 * (i + 1) as f64,
                    0.01 * (i + 1) as f64 + 0.002,
                    Some(0.001),
                )
            })
            .collect();
        let iv = interval(&p).expect("an interval");
        assert_eq!(iv.corner_method, CornerMethod::Gradient);
        assert!(interval_display(&iv, "reading uncertainty only")
            .ends_with("(corners chosen by gradient; not guaranteed extremes)"));
        assert!(iv.r2_low <= point_sums(&p).r2() + 1e-12);
    }

    #[test]
    fn the_systematic_interval_brackets_the_value_apart_from_the_reading_interval() {
        let p = example_1();
        let (lo, hi) = systematic_interval(&p, 0.05);
        assert!(lo <= hi);
        assert!(systematic_interval_display(0.05, lo, hi)
            .starts_with("R\u{b2} with systematic uncertainty \u{b1}5 % ["));
    }

    #[test]
    fn the_sixteen_codes_have_one_sentence_captions_and_the_checks_run_in_order() {
        assert_eq!(ReasonCode::ALL.len(), 16);
        for c in ReasonCode::ALL {
            let s = c.caption();
            assert!(s.ends_with('.') && !s[..s.len() - 1].contains(". "), "{s}");
        }
        let mut sorted = Check::ALL;
        sorted.sort();
        assert_eq!(sorted, Check::ALL);
        assert_eq!(Check::D1.reason(), Check::D2.reason());
        assert_eq!(Check::ALL.map(Check::id)[0], "M0");
        assert_eq!(Check::ALL.map(Check::id)[16], "V1");
    }
}
