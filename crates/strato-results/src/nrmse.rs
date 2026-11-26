//! The experimental comparison metric NRMSE of `metric-nrmse.md` (decisions 64 and 65).
//!
//! ```text
//! RMSE_M  = sqrt( (1/N) sum_i (F_M,i - F_exp,i)^2 )
//! NRMSE_M = 100 RMSE_M / F_scale,   F_scale = max_i F_exp,i   (FSCALE_MAX_EXP_V1)
//! ```
//!
//! This module holds the arithmetic, the propagation of the reading uncertainty (Section 4), the
//! display rule (Section 6) and the empty state. It forms no set: which rows are compatible
//! points is decided by the caller from the registers and the science design; here a point is
//! already a number pair. Nothing here reads a model: the model values are arguments, so the
//! tests inject illustrative numbers and production output is never touched.

/// The definition id of the scale.
pub const FSCALE_DEFINITION_ID: &str = "FSCALE_MAX_EXP_V1";

/// One compatible point of a comparison set.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// Register row id.
    pub id: String,
    /// Registered value `F_exp,i` (a cumulative release fraction).
    pub f_exp: f64,
    /// Model value `F_M,i` at the aligned time or window.
    pub f_model: f64,
    /// Reading uncertainty of the value, absolute (`p/100 F_exp` for a relative one); `None` if
    /// the register records none.
    pub u_value: Option<f64>,
    /// Reading uncertainty of the time in hours; `None` if none is recorded.
    pub u_time_h: Option<f64>,
    /// Slope `dF_M/dt` of the model at the point, per hour, from the model's own output.
    pub slope_per_h: f64,
}

/// The reason a record has no value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyReason {
    /// No compatible point after the exclusions.
    NoCompatiblePoints,
    /// `F_scale` is not positive.
    ScaleNotPositive,
    /// `RMSE` is not finite.
    RmseNotFinite,
    /// Some point of the set is not resolved by the model (science design Section 3.5).
    ModelNotResolved,
    /// Method B does not apply to the configuration (metric-nrmse Section 5).
    MethodBNotApplicable,
}

impl EmptyReason {
    /// The code written to the files.
    pub fn code(self) -> &'static str {
        match self {
            EmptyReason::NoCompatiblePoints => "no_compatible_points",
            EmptyReason::ScaleNotPositive => "f_scale_not_positive",
            EmptyReason::RmseNotFinite => "rmse_not_finite",
            EmptyReason::ModelNotResolved => "model_not_resolved",
            EmptyReason::MethodBNotApplicable => "method_b_not_applicable",
        }
    }
}

/// The value of one record.
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    /// `N`.
    pub n: usize,
    /// `F_scale`.
    pub f_scale: f64,
    /// The row that gives the scale (the latest of tied values).
    pub f_scale_row_id: String,
    /// `RMSE`.
    pub rmse: f64,
    /// `NRMSE` in percent.
    pub nrmse_percent: f64,
    /// Propagated reading uncertainty in percentage points, `None` if some point has none
    /// recorded.
    pub reading_uncertainty_pp: Option<f64>,
}

/// `F_scale`: the largest experimental value; with ties the latest point (the last in the given
/// order, which is time order) gives the row.
fn scale_index(points: &[Point]) -> usize {
    let mut k = 0;
    for (i, p) in points.iter().enumerate() {
        if p.f_exp >= points[k].f_exp {
            k = i;
        }
    }
    k
}

/// The NRMSE of a set of points, or the reason it is empty. The points are in time order.
///
/// # Errors
///
/// [`EmptyReason`] when `N = 0`, `F_scale` is not positive or `RMSE` is not finite.
pub fn compute(points: &[Point]) -> Result<Value, EmptyReason> {
    if points.is_empty() {
        return Err(EmptyReason::NoCompatiblePoints);
    }
    let n = points.len();
    let k = scale_index(points);
    let f_scale = points[k].f_exp;
    if f_scale.is_nan() || f_scale <= 0.0 {
        return Err(EmptyReason::ScaleNotPositive);
    }
    let sum: f64 = points.iter().map(|p| (p.f_model - p.f_exp).powi(2)).sum();
    let rmse = (sum / n as f64).sqrt();
    if !rmse.is_finite() {
        return Err(EmptyReason::RmseNotFinite);
    }
    let nrmse = 100.0 * rmse / f_scale;
    Ok(Value {
        n,
        f_scale,
        f_scale_row_id: points[k].id.clone(),
        rmse,
        nrmse_percent: nrmse,
        reading_uncertainty_pp: reading_uncertainty(points, k, rmse).map(|u_rel| nrmse * u_rel),
    })
}

/// The relative reading uncertainty of Section 4, first-order propagation with independent
/// points: `u_rel^2 = sum_i [(a_i + delta_ik / F_scale)^2 u_e,i^2 + a_i^2 s_i^2 u_t,i^2]` with
/// `a_i = d_i / (N RMSE^2)`. `None` if some point has no recorded uncertainty, or `RMSE = 0`.
fn reading_uncertainty(points: &[Point], k: usize, rmse: f64) -> Option<f64> {
    if rmse.is_nan() || rmse <= 0.0 {
        return None;
    }
    let n = points.len() as f64;
    let f_scale = points[k].f_exp;
    let mut sum = 0.0;
    for (i, p) in points.iter().enumerate() {
        let (ue, ut) = (p.u_value?, p.u_time_h?);
        let a = (p.f_model - p.f_exp) / (n * rmse * rmse);
        let scale_term = if i == k { 1.0 / f_scale } else { 0.0 };
        sum += (a + scale_term).powi(2) * ue * ue + (a * p.slope_per_h * ut).powi(2);
    }
    Some(sum.sqrt())
}

/// The display string of Section 6: two significant figures, formatted once here. A value with
/// `0.1 <= x < 100` (decided on the two-digit mantissa) is a plain decimal (`0.43`, `2.6`, `11`,
/// `87`); otherwise `m × 10^n` (`3.4 × 10^2`, `4.2 × 10^-3`); exactly 0 is `0`. The unit sign is
/// not part of the string; the labels add it.
pub fn display(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }
    let s = format!("{x:.1e}");
    let (mant, exp) = s.split_once('e').expect("scientific format");
    let n: i32 = exp.parse().expect("exponent");
    let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();
    match n {
        -1 => format!("0.{digits}"),
        0 => format!("{}.{}", &digits[..1], &digits[1..]),
        1 => digits,
        _ => format!("{mant} \u{d7} 10^{n}"),
    }
}

/// The LaTeX of the display string (`3.4 \times 10^{2}`).
pub fn display_latex(display: &str) -> String {
    match display.split_once(" \u{d7} 10^") {
        Some((m, e)) => format!("{m} \\times 10^{{{e}}}"),
        None => display.to_string(),
    }
}

/// The label of a method for the combined graph: LaTeX and plain text, with the display string
/// only. `method` is `A` or `B`.
pub fn label(method: char, display: &str) -> (String, String) {
    (
        format!(
            "\\mathrm{{NRMSE}}_{{\\mathrm{{{method}}}}} = {}\\,\\%",
            display_latex(display)
        ),
        format!("NRMSE_{method} = {display}%"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(id: &str, f_exp: f64, f_model: f64, u_value: f64, u_t: f64, slope: f64) -> Point {
        Point {
            id: id.into(),
            f_exp,
            f_model,
            u_value: Some(u_value),
            u_time_h: Some(u_t),
            slope_per_h: slope,
        }
    }

    /// Worked example 1 of `metric-nrmse.md` Section 8: a flat series, illustrative model value
    /// 0.330 at every point. The model numbers are invented to check the arithmetic and are not
    /// results.
    #[test]
    fn worked_example_1_flat_series() {
        let exp = [
            0.335, 0.338, 0.338, 0.338, 0.339, 0.339, 0.339, 0.340, 0.340, 0.340,
        ];
        let pts: Vec<Point> = exp
            .iter()
            .enumerate()
            .map(|(i, e)| pt(&format!("E-{}", 425 + i), *e, 0.330, 0.005 * e, 0.2, 0.0))
            .collect();
        let v = compute(&pts).unwrap();
        assert_eq!(v.n, 10);
        assert_eq!(v.f_scale, 0.340);
        assert_eq!(v.f_scale_row_id, "E-434", "the latest of three tied values");
        // sum d^2 = 25 + 3*64 + 3*81 + 3*100 (x 1e-6) = 7.600e-4.
        assert!((v.rmse - 8.7178e-3).abs() < 5e-8, "{}", v.rmse);
        assert!(
            (v.nrmse_percent - 2.5641).abs() < 5e-4,
            "{}",
            v.nrmse_percent
        );
        assert_eq!(display(v.nrmse_percent), "2.6");
        // u_rel = 0.059849 and u = 0.15 percentage points.
        let u = v.reading_uncertainty_pp.unwrap();
        assert!((u / v.nrmse_percent - 0.059849).abs() < 5e-6, "{u}");
        assert_eq!(format!("{u:.2}"), "0.15");
    }

    /// Worked example 2: a rising series with model `F_A(t) = 3.0e-8 t` (t in h), slope 3.0e-8
    /// per hour. Illustrative model values.
    #[test]
    fn worked_example_2_rising_series() {
        let t = [
            1.1, 25.1, 49.5, 73.2, 97.2, 145.3, 193.2, 255.1, 303.2, 351.1, 399.1,
        ];
        let f = [
            9.12e-7, 2.45e-6, 3.19e-6, 3.70e-6, 4.25e-6, 5.31e-6, 6.10e-6, 7.27e-6, 8.07e-6,
            9.11e-6, 1.10e-5,
        ];
        let pts: Vec<Point> = t
            .iter()
            .zip(f)
            .enumerate()
            .map(|(i, (t, f))| {
                pt(
                    &format!("E-{}", 472 + i),
                    f,
                    3.0e-8 * t,
                    0.01 * f,
                    0.2,
                    3.0e-8,
                )
            })
            .collect();
        let v = compute(&pts).unwrap();
        assert_eq!(v.f_scale, 1.10e-5);
        assert_eq!(v.f_scale_row_id, "E-482");
        assert!((v.rmse - 1.19730e-6).abs() < 5e-11, "{}", v.rmse);
        assert!(
            (v.nrmse_percent - 10.885).abs() < 5e-3,
            "{}",
            v.nrmse_percent
        );
        assert_eq!(display(v.nrmse_percent), "11");
        let u = v.reading_uncertainty_pp.unwrap();
        assert!((u / v.nrmse_percent - 0.020940).abs() < 5e-6, "{u}");
        assert_eq!(format!("{u:.2}"), "0.23");
    }

    #[test]
    fn one_point_gives_the_relative_error() {
        let v = compute(&[pt("E-1", 2.0e-3, 3.0e-3, 1e-5, 0.1, 0.0)]).unwrap();
        assert_eq!(v.n, 1);
        assert!((v.nrmse_percent - 50.0).abs() < 1e-12);
    }

    #[test]
    fn the_empty_state_has_a_reason_and_no_value() {
        assert_eq!(compute(&[]), Err(EmptyReason::NoCompatiblePoints));
        assert_eq!(
            compute(&[pt("E-1", 0.0, 1.0, 0.0, 0.0, 0.0)]),
            Err(EmptyReason::ScaleNotPositive)
        );
        assert_eq!(
            compute(&[pt("E-1", 1.0, f64::NAN, 0.0, 0.0, 0.0)]),
            Err(EmptyReason::RmseNotFinite)
        );
    }

    #[test]
    fn a_missing_reading_uncertainty_makes_the_term_unavailable_not_assumed() {
        let mut p = pt("E-1", 1.0, 1.5, 0.01, 0.1, 0.0);
        p.u_time_h = None;
        let v = compute(&[p]).unwrap();
        assert_eq!(v.reading_uncertainty_pp, None);
    }

    #[test]
    fn display_follows_section_6() {
        assert_eq!(display(0.43), "0.43");
        assert_eq!(display(2.5641), "2.6");
        assert_eq!(display(10.885), "11");
        assert_eq!(display(87.3), "87");
        assert_eq!(display(340.0), "3.4 \u{d7} 10^2");
        assert_eq!(display(9.0e4), "9.0 \u{d7} 10^4");
        assert_eq!(display(4.2e-3), "4.2 \u{d7} 10^-3");
        assert_eq!(display(0.0), "0");
        let (l, p) = label('A', &display(340.0));
        assert_eq!(
            l,
            "\\mathrm{NRMSE}_{\\mathrm{A}} = 3.4 \\times 10^{2}\\,\\%"
        );
        assert_eq!(p, "NRMSE_A = 3.4 \u{d7} 10^2%");
        let (l, _) = label('B', "2.6");
        assert_eq!(l, "\\mathrm{NRMSE}_{\\mathrm{B}} = 2.6\\,\\%");
    }

    #[test]
    fn the_scale_is_the_experiment_only() {
        // Changing the model values never changes F_scale.
        let a = compute(&[
            pt("E-1", 0.1, 0.0, 0.0, 0.0, 0.0),
            pt("E-2", 0.3, 0.0, 0.0, 0.0, 0.0),
        ])
        .unwrap();
        let b = compute(&[
            pt("E-1", 0.1, 9.0, 0.0, 0.0, 0.0),
            pt("E-2", 0.3, 9.0, 0.0, 0.0, 0.0),
        ])
        .unwrap();
        assert_eq!(a.f_scale, b.f_scale);
        assert_eq!(a.f_scale, 0.3);
    }
}
