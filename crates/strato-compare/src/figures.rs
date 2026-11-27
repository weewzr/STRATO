//! Static figures A to H as SVG, drawn with `plotters` (pure Rust SVG backend, no fonts or image
//! libraries needed).
//!
//! Drawing rules: a measurement is drawn only if the register holds it; its marker says what kind
//! of comparison it supports (filled triangle pointing down: upper bound; hollow diamond or circle:
//! DATA INCOMPLETE, shown for scale; grey cross: NOT DIRECTLY COMPARABLE context). Error bars are
//! drawn only where the source reports a measurement uncertainty, and the register holds none.
//! Every figure names its sources, the solver version and the commit, and says that decay and
//! generation are not in the solver.

use std::path::Path;

use plotters::element::DashedPathElement;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use strato_data::release::{Measured, ReadingUncertainty, ReleaseValue, TestTemperature};
use strato_data::table::Vocabulary;
use strato_data::Register;

use crate::compare::{Eval, RowRecord};
use crate::data::Species;
use crate::model::Curve;
use crate::outputs::Prov;
use crate::status::{end_time, measured_text, Status};
use crate::studies::{AnalyticSeries, ConvergenceRow, ProfileRow, SensitivityRow};

/// Residual and bound series of one case in figure E.
type ResidualSeries = (Species, f64, Vec<(f64, f64)>, Vec<(f64, f64)>);

fn pe<E: std::fmt::Display>(e: E) -> String {
    format!("plot error: {e}")
}

// Colour-blind-safe palette (Okabe and Ito).
const BLUE: RGBColor = RGBColor(0, 114, 178);
const ORANGE: RGBColor = RGBColor(230, 159, 0);
const PURPLE: RGBColor = RGBColor(204, 121, 167);
const GREEN: RGBColor = RGBColor(0, 158, 115);
const SKY: RGBColor = RGBColor(86, 180, 233);
const VERMILION: RGBColor = RGBColor(213, 94, 0);
const GREY: RGBColor = RGBColor(120, 120, 120);
const INK: RGBColor = RGBColor(25, 25, 25);

/// The three test temperatures drawn.
pub const TEMPERATURES: [f64; 3] = [1600.0, 1700.0, 1800.0];

fn temp_colour(t_c: f64) -> RGBColor {
    if (t_c - 1600.0).abs() < 1.0 {
        BLUE
    } else if (t_c - 1700.0).abs() < 1.0 {
        ORANGE
    } else {
        PURPLE
    }
}

fn temp_dash(t_c: f64) -> Option<(u32, u32)> {
    if (t_c - 1600.0).abs() < 1.0 {
        None
    } else if (t_c - 1700.0).abs() < 1.0 {
        Some((10, 6))
    } else {
        Some((2, 5))
    }
}

/// A tick label of a logarithmic axis: ten with a superscript exponent, for example 10⁻³.
fn sci(v: &f64) -> String {
    let e = v.log10().round() as i32;
    let sup: String = e
        .to_string()
        .chars()
        .map(|c| match c {
            '-' => '⁻',
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            other => other,
        })
        .collect();
    format!("10{sup}")
}

fn ts(size: u32) -> TextStyle<'static> {
    ("sans-serif", size).into_font().color(&INK)
}

fn ts_pos(size: u32, h: HPos, v: VPos) -> TextStyle<'static> {
    ts(size).pos(Pos::new(h, v))
}

fn shape(c: RGBColor, filled: bool, w: u32) -> ShapeStyle {
    ShapeStyle {
        color: c.to_rgba(),
        filled,
        stroke_width: w,
    }
}

/// The kinds of marker.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Glyph {
    /// Triangle pointing down: an upper bound.
    BoundDown {
        /// Colour.
        c: RGBColor,
        /// Filled when the model satisfies the bound; the outline of an exceeded bound is heavier.
        filled: bool,
    },
    /// Hollow diamond: DATA INCOMPLETE, shown for scale.
    Diamond {
        /// Colour.
        c: RGBColor,
    },
    /// Hollow circle (second nuclide of a provisional pair).
    Ring {
        /// Colour.
        c: RGBColor,
    },
    /// Hollow square.
    Square {
        /// Colour.
        c: RGBColor,
    },
    /// Grey cross: context that is not directly comparable.
    Cross,
    /// Plus sign: the independent analytical series.
    Plus {
        /// Colour.
        c: RGBColor,
    },
    /// Filled circle: a point comparison or a computed point.
    Dot {
        /// Colour.
        c: RGBColor,
    },
    /// A line sample for legends.
    Line {
        /// Colour.
        c: RGBColor,
        /// Dash pattern.
        dash: Option<(u32, u32)>,
    },
}

macro_rules! mark {
    ($chart:expr, $glyph:expr, $pts:expr) => {{
        let pts: Vec<(f64, f64)> = $pts;
        match $glyph {
            Glyph::BoundDown { c, filled } => {
                if filled {
                    $chart
                        .draw_series(pts.iter().map(|p| {
                            EmptyElement::at(*p)
                                + Polygon::new(vec![(-6, -6), (6, -6), (0, 7)], shape(c, true, 1))
                        }))
                        .map_err(pe)?;
                } else {
                    $chart
                        .draw_series(pts.iter().map(|p| {
                            EmptyElement::at(*p)
                                + PathElement::new(
                                    vec![(-6, -6), (6, -6), (0, 7), (-6, -6)],
                                    shape(c, false, 3),
                                )
                        }))
                        .map_err(pe)?;
                }
            }
            Glyph::Diamond { c } => {
                $chart
                    .draw_series(pts.iter().map(|p| {
                        EmptyElement::at(*p)
                            + PathElement::new(
                                vec![(0, -7), (7, 0), (0, 7), (-7, 0), (0, -7)],
                                shape(c, false, 2),
                            )
                    }))
                    .map_err(pe)?;
            }
            Glyph::Ring { c } => {
                $chart
                    .draw_series(pts.iter().map(|p| Circle::new(*p, 6, shape(c, false, 2))))
                    .map_err(pe)?;
            }
            Glyph::Square { c } => {
                $chart
                    .draw_series(pts.iter().map(|p| {
                        EmptyElement::at(*p)
                            + Rectangle::new([(-5, -5), (5, 5)], shape(c, false, 2))
                    }))
                    .map_err(pe)?;
            }
            Glyph::Cross => {
                $chart
                    .draw_series(pts.iter().map(|p| Cross::new(*p, 4, shape(GREY, false, 2))))
                    .map_err(pe)?;
            }
            Glyph::Plus { c } => {
                $chart
                    .draw_series(pts.iter().map(|p| {
                        EmptyElement::at(*p)
                            + PathElement::new(vec![(-4, 0), (4, 0)], shape(c, false, 1))
                            + PathElement::new(vec![(0, -4), (0, 4)], shape(c, false, 1))
                    }))
                    .map_err(pe)?;
            }
            Glyph::Dot { c } => {
                $chart
                    .draw_series(pts.iter().map(|p| Circle::new(*p, 4, shape(c, true, 1))))
                    .map_err(pe)?;
            }
            Glyph::Line { .. } => {}
        }
    }};
}

macro_rules! line {
    ($chart:expr, $pts:expr, $c:expr, $dash:expr, $w:expr) => {{
        let pts: Vec<(f64, f64)> = $pts;
        if pts.len() >= 2 {
            match $dash {
                None => {
                    $chart
                        .draw_series(LineSeries::new(pts, shape($c, false, $w)))
                        .map_err(pe)?;
                }
                Some((on, off)) => {
                    $chart
                        .draw_series(std::iter::once(DashedPathElement::new(
                            pts,
                            on,
                            off,
                            shape($c, false, $w),
                        )))
                        .map_err(pe)?;
                }
            }
        }
    }};
}

/// One legend entry.
pub struct Entry {
    /// The label.
    pub label: String,
    /// The glyph.
    pub glyph: Glyph,
}

fn entry(label: impl Into<String>, glyph: Glyph) -> Entry {
    Entry {
        label: label.into(),
        glyph,
    }
}

type Root<'a> = DrawingArea<SVGBackend<'a>, plotters::coord::Shift>;

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for w in text.split_whitespace() {
        if !cur.is_empty() && cur.len() + 1 + w.len() > width {
            lines.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(w);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

fn title(root: &Root<'_>, text: &str, sub: &str) -> Result<(), String> {
    root.draw(&Text::new(text.to_string(), (24, 14), ts(26)))
        .map_err(pe)?;
    root.draw(&Text::new(sub.to_string(), (24, 48), ts(16)))
        .map_err(pe)?;
    Ok(())
}

/// Legend in columns, laid out in pixels from `(x0, y0)`.
fn legend(
    root: &Root<'_>,
    x0: i32,
    y0: i32,
    col_width: i32,
    cols: usize,
    items: &[Entry],
) -> Result<i32, String> {
    let rows = items.len().div_ceil(cols.max(1));
    for (k, e) in items.iter().enumerate() {
        let (col, row) = (k % cols, k / cols);
        let x = x0 + col as i32 * col_width;
        let y = y0 + row as i32 * 26;
        let (cx, cy) = (x + 16, y + 9);
        match e.glyph {
            Glyph::Line { c, dash } => {
                let style = shape(c, false, 3);
                match dash {
                    None => root
                        .draw(&PathElement::new(vec![(x, cy), (x + 34, cy)], style))
                        .map_err(pe)?,
                    Some((on, off)) => root
                        .draw(&DashedPathElement::new(
                            vec![(x, cy), (x + 34, cy)],
                            on,
                            off,
                            style,
                        ))
                        .map_err(pe)?,
                }
            }
            Glyph::BoundDown { c, filled } => {
                if filled {
                    root.draw(
                        &(EmptyElement::at((cx, cy))
                            + Polygon::new(vec![(-6, -6), (6, -6), (0, 7)], shape(c, true, 1))),
                    )
                    .map_err(pe)?
                } else {
                    root.draw(
                        &(EmptyElement::at((cx, cy))
                            + PathElement::new(
                                vec![(-6, -6), (6, -6), (0, 7), (-6, -6)],
                                shape(c, false, 3),
                            )),
                    )
                    .map_err(pe)?
                }
            }
            Glyph::Diamond { c } => root
                .draw(
                    &(EmptyElement::at((cx, cy))
                        + PathElement::new(
                            vec![(0, -7), (7, 0), (0, 7), (-7, 0), (0, -7)],
                            shape(c, false, 2),
                        )),
                )
                .map_err(pe)?,
            Glyph::Ring { c } => root
                .draw(&Circle::new((cx, cy), 6, shape(c, false, 2)))
                .map_err(pe)?,
            Glyph::Square { c } => root
                .draw(
                    &(EmptyElement::at((cx, cy))
                        + Rectangle::new([(-5, -5), (5, 5)], shape(c, false, 2))),
                )
                .map_err(pe)?,
            Glyph::Cross => root
                .draw(&Cross::new((cx, cy), 4, shape(GREY, false, 2)))
                .map_err(pe)?,
            Glyph::Plus { c } => root
                .draw(
                    &(EmptyElement::at((cx, cy))
                        + PathElement::new(vec![(-5, 0), (5, 0)], shape(c, false, 1))
                        + PathElement::new(vec![(0, -5), (0, 5)], shape(c, false, 1))),
                )
                .map_err(pe)?,
            Glyph::Dot { c } => root
                .draw(&Circle::new((cx, cy), 4, shape(c, true, 1)))
                .map_err(pe)?,
        }
        root.draw(&Text::new(e.label.clone(), (x + 44, y), ts(15)))
            .map_err(pe)?;
    }
    Ok(y0 + rows as i32 * 26)
}

fn footer(root: &Root<'_>, y: i32, width_chars: usize, text: &str) -> Result<(), String> {
    for (k, l) in wrap(text, width_chars).iter().enumerate() {
        root.draw(&Text::new(l.clone(), (24, y + 18 * k as i32), ts(13)))
            .map_err(pe)?;
    }
    Ok(())
}

fn provenance_line(prov: &Prov) -> String {
    format!(
        "Model: STRATO BTCS five-layer spherical finite difference, package version {}, commit {}; registered AGR1-B-M16 geometry; published heating-test diffusion coefficients chosen by the strato-data selection function (register sets C16-T2 and C16-BUF), evaluated at the test temperature; kernel-only start; constant temperature; decay and generation are not in the solver. This is a numerical comparison, not an experimental validation.",
        prov.solver_version, prov.commit
    )
}

/// Footer text of the verification figures F and G: a synthetic homogeneous sphere, not AGR-1.
fn verification_line(prov: &Prov) -> String {
    format!(
        "Solver: STRATO BTCS five-layer spherical finite difference, package version {}, commit {}. The case is a numerical verification case on the registered AGR1-B-M16 radii with the same synthetic diffusion coefficient in all five regions, kernel-only start, absorbing outer surface; the reference is the exact series of a homogeneous sphere. It uses no AGR-1 diffusion coefficient and no measurement, and it is not an experimental validation.",
        prov.solver_version, prov.commit
    )
}

fn range_of(values: impl Iterator<Item = f64>, floor: f64) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = 0.0_f64;
    for v in values.filter(|v| *v > 0.0 && v.is_finite()) {
        lo = lo.min(v.max(floor));
        hi = hi.max(v);
    }
    if !lo.is_finite() || hi <= 0.0 {
        return (floor, 1.0);
    }
    let lo_dec = lo.log10().floor();
    let hi_dec = hi.log10().ceil();
    (10f64.powf(lo_dec), 10f64.powf(hi_dec.max(lo_dec + 1.0)))
}

fn panel_title(species: Species) -> String {
    match species {
        Species::Ag => "Silver (Ag)",
        Species::Cs => "Caesium (Cs)",
        Species::Sr => "Strontium (Sr)",
    }
    .to_string()
}

fn finish(root: Root<'_>) -> Result<(), String> {
    root.present().map_err(pe)
}

/// A measurement to place in figure A, B or C.
#[derive(Debug, Clone)]
pub struct Placed {
    /// Row id.
    pub id: String,
    /// Element panel.
    pub species: Species,
    /// Test temperature, °C.
    pub t_c: f64,
    /// Time at temperature, hours.
    pub t_h: f64,
    /// The value drawn (measured value or bound).
    pub y: f64,
    /// Status after the model.
    pub status: Status,
    /// True for a bound drawn as a bound.
    pub is_bound: bool,
    /// Source citation.
    pub source: String,
    /// Programme.
    pub program: String,
    /// The nuclide or element as the register names it (the model has no decay, so it computes
    /// the element).
    pub nuclide: String,
    /// The model release over the window of the measurement, when the measurement covers a window
    /// that does not start at the beginning of the test (matrix-corrected values).
    pub model_window: Option<f64>,
    /// True if the value was read from a published figure (class digitised), false if it is a
    /// tabulated or printed value that is exact as printed.
    pub digitised: bool,
    /// The interval of the value that the reading uncertainty of the register allows (lowest and
    /// highest value), for a digitised row whose record states a reading uncertainty on the value.
    pub reading_interval: Option<(f64, f64)>,
    /// The rule of the comparison status (why the row is or is not comparable).
    pub rule: String,
}

/// The interval a stated reading uncertainty allows around a value.
pub fn reading_interval(
    u: &strato_data::release::ReadingUncertainty,
    y: f64,
) -> Option<(f64, f64)> {
    use strato_data::release::{ReadingUncertainty as U, ValueReading as V};
    let U::Stated { value: Some(v), .. } = u else {
        return None;
    };
    let (lo, hi) = match v {
        V::RelativePercent(p) => (y * (1.0 - p.value / 100.0), y * (1.0 + p.value / 100.0)),
        V::RelativePercentRange { hi, .. } => {
            (y * (1.0 - hi.value / 100.0), y * (1.0 + hi.value / 100.0))
        }
        V::Absolute(p) => (y - p.value, y + p.value),
        V::Factor(p) if p.value > 0.0 => (y / p.value, y * p.value),
        V::Factor(_) => return None,
    };
    Some((lo, hi))
}

/// Rows that can be placed on the release-versus-time panels: a fraction (unit 1), a point,
/// digitised or bound value, one of the three test temperatures, an element with a curve.
pub fn placed_rows(reg: &Register, records: &[RowRecord]) -> Vec<Placed> {
    let mut out = Vec::new();
    for r in records {
        if r.row.unit.as_str() != "1" {
            continue;
        }
        let TestTemperature::Celsius(t) = &r.row.temperature else {
            continue;
        };
        if !TEMPERATURES.iter().any(|x| (x - t.value).abs() < 1.0) {
            continue;
        }
        let element = match &r.row.measured {
            strato_data::release::Measured::Nuclide(n) => n.element.clone(),
            strato_data::release::Measured::Element(e) => e.clone(),
            strato_data::release::Measured::NotApplicable => continue,
        };
        // The panel is the element: every nuclide of the element (Cs-134, Cs-137) and an element-only
        // label are drawn in the panel of the element; the nuclide of each row is listed in the
        // panel (`Placed::nuclide`). The model has no decay, so it computes the element.
        let Some(species) = Species::of_element(&element) else {
            continue;
        };
        let (y, is_bound) = match &r.row.value {
            ReleaseValue::Point(p) | ReleaseValue::Digitised(p) => (p.value, false),
            ReleaseValue::UpperBound(p) => (p.value, true),
            _ => continue,
        };
        let is_bound = is_bound || matches!(r.eval, Eval::Bound { .. });
        // Measurements of a window (matrix-corrected) are placed at the end of the window: the end
        // of the 300 h plateau or of the shorter test (decision 68), not the as-run end of a
        // longer test.
        let t_h = match (&r.row.time_h, &r.model) {
            (Some(p), _) => p.value,
            (None, Some(m)) if r.is_window_bound() => m.spec.t_end_h,
            _ => match end_time(reg, &r.row, t.value) {
                Some((h, _)) => h,
                None => continue,
            },
        };
        out.push(Placed {
            id: r.row.id.clone(),
            species,
            t_c: t.value,
            t_h,
            y,
            status: r.status,
            is_bound,
            source: r.row.source.clone(),
            program: r.row.program.as_str().to_string(),
            nuclide: measured_text(&r.row),
            model_window: r
                .model
                .as_ref()
                .filter(|m| m.spec.window_start_h > 0.0)
                .map(|m| m.f_window),
            digitised: matches!(r.row.value, ReleaseValue::Digitised(_))
                || !matches!(
                    r.row.reading_uncertainty,
                    strato_data::release::ReadingUncertainty::NotDigitised
                ),
            reading_interval: reading_interval(&r.row.reading_uncertainty, y),
            rule: r.verdict.category.to_string(),
        });
    }
    out
}

/// Drawing order of a marker: 0 for context that is not comparable (muted grey), 1 for rows whose
/// denominator is not established (hollow diamonds), 2 for upper bounds and point comparisons,
/// which are drawn on top.
pub fn marker_pass(p: &Placed) -> u8 {
    match p.status {
        Status::NotDirectlyComparable | Status::NotAssessed => 0,
        Status::DataIncomplete => 1,
        _ => 2,
    }
}

/// The marker of a measurement. The status of the row decides it, never the way the value was
/// obtained: an upper bound is always a bound glyph (or a muted cross when it is context), never a
/// dot or a ring that would read as an exact value.
pub fn glyph_of(p: &Placed) -> Glyph {
    let c = temp_colour(p.t_c);
    match p.status {
        Status::WithinReportedUpperBound => Glyph::BoundDown { c, filled: true },
        Status::ExceedsReportedUpperBound => Glyph::BoundDown {
            c: VERMILION,
            filled: false,
        },
        Status::DataIncomplete => Glyph::Diamond { c },
        Status::ComparisonAvailable
        | Status::WithinReportedUncertainty
        | Status::OutsideReportedUncertainty => Glyph::Dot { c },
        Status::NotDirectlyComparable | Status::NotAssessed => Glyph::Cross,
    }
}

/// The legend entries of the markers that are actually drawn: an entry exists only if at least one
/// drawn measurement has the status it describes.
fn status_legend(placed: &[Placed]) -> Vec<Entry> {
    let has = |f: &dyn Fn(Status) -> bool| placed.iter().any(|p| drawable(p) && f(p.status));
    let mut v = Vec::new();
    if has(&|s| s == Status::WithinReportedUpperBound) {
        v.push(entry(
            "Upper bound, model within it (WITHIN REPORTED UPPER BOUND)",
            Glyph::BoundDown {
                c: INK,
                filled: true,
            },
        ));
    }
    if has(&|s| s == Status::ExceedsReportedUpperBound) {
        v.push(entry(
            "Upper bound exceeded by the model (EXCEEDS REPORTED UPPER BOUND)",
            Glyph::BoundDown {
                c: VERMILION,
                filled: false,
            },
        ));
    }
    if has(&|s| s == Status::DataIncomplete) {
        v.push(entry(
            "Hollow diamond: DATA INCOMPLETE, shown for scale, not a comparison",
            Glyph::Diamond { c: INK },
        ));
    }
    if has(&|s| matches!(s, Status::NotDirectlyComparable | Status::NotAssessed)) {
        v.push(entry(
            "Grey cross: NOT DIRECTLY COMPARABLE context (muted)",
            Glyph::Cross,
        ));
    }
    v
}

fn temperature_legend() -> Vec<Entry> {
    TEMPERATURES
        .iter()
        .map(|t| {
            entry(
                format!("BTCS model, {t:.0} °C (marker colour is the test temperature)"),
                Glyph::Line {
                    c: temp_colour(*t),
                    dash: temp_dash(*t),
                },
            )
        })
        .collect()
}

/// Inputs of the figures.
pub struct Inputs<'a> {
    /// The registers.
    pub reg: &'a Register,
    /// All rows with their status.
    pub records: &'a [RowRecord],
    /// The default curves.
    pub curves: &'a [(Species, f64, &'a Curve)],
    /// The independent analytical series at sample times, per species and temperature.
    pub analytic: &'a AnalyticSeries,
    /// Radial profiles.
    pub profiles: &'a [ProfileRow],
    /// Temperature sensitivity.
    pub sensitivity: &'a [SensitivityRow],
    /// Spatial convergence.
    pub spatial: &'a [ConvergenceRow],
    /// Temporal convergence.
    pub temporal: &'a [ConvergenceRow],
    /// Spatial error floor of the mesh of the temporal study.
    pub temporal_spatial_floor: f64,
    /// Description of the verification case.
    pub homogeneous_note: String,
    /// Provenance.
    pub prov: &'a Prov,
    /// Region outer radii in micrometre.
    pub radii_um: [f64; 5],
}

/// The caveat on the early part of the BTCS curves, printed in the footers of the figures A, I and J.
const EARLY_TIME_NOTE: &str = "The early part of every BTCS curve is not converged in time: where the release is still rising steeply from zero, the base run and the run with half the step differ by a large factor, and the report states for each curve the time after which they agree within 10 percent (a reading aid, not a tolerance).";

/// Upper end of the time axis of the figures A, I and J, hours at temperature.
pub const T_AXIS_H: f64 = 420.0;

/// True if a measurement lies inside the axes of the release-versus-time panels. A value beyond
/// the time axis is never moved onto its edge: it is not drawn and the footer lists it.
pub fn drawable(p: &Placed) -> bool {
    p.y > 1e-10 && p.t_h > 0.0 && p.t_h <= T_AXIS_H
}

/// The sentence of a footer that lists the measurements that are not drawn because their time lies
/// beyond the time axis (empty if there is none).
fn beyond_axis_note(rows: impl Iterator<Item = (String, f64)>) -> String {
    let mut items: Vec<String> = rows
        .filter(|(_, t)| *t > T_AXIS_H)
        .map(|(id, t)| format!("{id} ({t} h)"))
        .collect();
    items.sort();
    items.dedup();
    if items.is_empty() {
        String::new()
    } else {
        format!(
            "Not drawn because the time at temperature lies beyond the {T_AXIS_H:.0} h axis (the value is never moved onto the edge): {}.",
            items.join(", ")
        )
    }
}

fn curve_points(c: &Curve) -> Vec<(f64, f64)> {
    c.t_h
        .iter()
        .zip(&c.f)
        .filter(|(t, f)| **t > 0.0 && **f > 1.0e-10)
        .map(|(t, f)| (*t, *f))
        .collect()
}

fn sources_of(placed: &[Placed]) -> String {
    let mut s: Vec<String> = placed.iter().map(|p| p.source.clone()).collect();
    s.sort();
    s.dedup();
    s.join("; ")
}

/// Figure A: release fraction versus time.
pub fn figure_a(path: &Path, inp: &Inputs<'_>) -> Result<(), String> {
    let root = SVGBackend::new(path, (1560, 960)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    title(
        &root,
        "A. Fraction released versus time at temperature",
        "BTCS model of one intact TRISO particle against the register's measurements; log scale; hours counted from reaching the test temperature",
    )?;
    let placed = placed_rows(inp.reg, inp.records);
    let panels = root.margin(70, 460, 10, 10);
    let areas = panels.split_evenly((1, 3));
    for (k, species) in Species::ALL.iter().enumerate() {
        let mut chart = ChartBuilder::on(&areas[k])
            .caption(panel_title(*species), ts(20))
            .margin(8)
            .x_label_area_size(44)
            .y_label_area_size(76)
            .build_cartesian_2d(0.0f64..T_AXIS_H, (1e-10f64..2.0f64).log_scale())
            .map_err(pe)?;
        chart
            .configure_mesh()
            .x_desc("Time at temperature (h)")
            .y_desc("Fraction released (dimensionless)")
            .x_labels(8)
            .x_label_formatter(&|v| format!("{v:.0}"))
            .y_label_formatter(&sci)
            .light_line_style(RGBColor(235, 235, 235))
            .label_style(ts(14))
            .axis_desc_style(ts(16))
            .draw()
            .map_err(pe)?;
        for (s, t_c, c) in inp.curves.iter().filter(|(s, _, _)| s == species) {
            let _ = s;
            line!(
                chart,
                curve_points(c),
                temp_colour(*t_c),
                temp_dash(*t_c),
                3
            );
            if let Some(an) = inp.analytic.get(&(*species, (*t_c * 10.0).round() as i64)) {
                let pts: Vec<(f64, f64)> = an
                    .iter()
                    .filter_map(|(t, f)| f.filter(|f| *f > 1e-10).map(|f| (*t, f)))
                    .collect();
                mark!(
                    chart,
                    Glyph::Plus {
                        c: temp_colour(*t_c)
                    },
                    pts
                );
            }
        }
        // Measurements: context first, then data incomplete, then bounds on top.
        for pass in 0..3 {
            for p in placed.iter().filter(|p| p.species == *species) {
                if marker_pass(p) == pass && drawable(p) {
                    mark!(chart, glyph_of(p), vec![(p.t_h, p.y)]);
                }
            }
        }
        // The model's own value over the window of a matrix-corrected measurement, beside it.
        for p in placed
            .iter()
            .filter(|p| p.species == *species && drawable(p))
        {
            if let Some(w) = p.model_window.filter(|w| *w > 1e-10) {
                mark!(
                    chart,
                    Glyph::Square {
                        c: temp_colour(p.t_c)
                    },
                    vec![(p.t_h, w)]
                );
            }
        }
        let mut nuclides: Vec<String> = placed
            .iter()
            .filter(|p| p.species == *species)
            .map(|p| p.nuclide.clone())
            .collect();
        nuclides.sort();
        nuclides.dedup();
        chart
            .draw_series(std::iter::once(Text::new(
                format!("measured: {}", nuclides.join(", ")),
                (4.0, 1.4),
                ts_pos(13, HPos::Left, VPos::Top),
            )))
            .map_err(pe)?;
        let n_here = placed
            .iter()
            .filter(|p| p.species == *species && drawable(p))
            .count();
        let count_of = |f: &dyn Fn(Status) -> bool| {
            placed
                .iter()
                .filter(|p| p.species == *species && drawable(p) && f(p.status))
                .count()
        };
        let n_bounds = count_of(&|s| {
            matches!(
                s,
                Status::WithinReportedUpperBound | Status::ExceedsReportedUpperBound
            )
        });
        let n_points = count_of(&|s| s == Status::ComparisonAvailable);
        let note = match (n_bounds, n_points) {
            (0, 0) => format!(
                "{n_here} registered values drawn; none supports a comparison (context only)"
            ),
            (b, 0) => format!("{n_here} registered values drawn; {b} upper-bound comparisons"),
            (0, p) => format!("{n_here} registered values drawn; {p} point comparisons"),
            (b, p) => format!(
                "{n_here} registered values drawn; {b} upper-bound comparisons, {p} point comparisons"
            ),
        };
        chart
            .draw_series(std::iter::once(Text::new(
                note,
                (416.0, 1.6e-10),
                ts_pos(13, HPos::Right, VPos::Bottom),
            )))
            .map_err(pe)?;
    }
    let mut items = temperature_legend();
    items.push(entry(
        "Plus: independent analytical series, only where valid (criterion in the footnote)",
        Glyph::Plus { c: INK },
    ));
    items.push(entry(
        "Hollow square: model over the same window (24 h to the plateau end) as a window upper bound",
        Glyph::Square { c: INK },
    ));
    items.extend(status_legend(&placed));
    let y = legend(&root, 40, 520, 760, 2, &items)?;
    let mut text = format!(
        "Experimental values: {}. Digitised values carry the reading uncertainty of the register; the sources report no per-value measurement uncertainty for these rows (the one general statement, a 5 percent systematic gamma-measurement error plus counting statistics for the Stempien 2023 values, is not a per-value uncertainty and changes no status), so no error bars are drawn. Compact values are not single-particle values. Rows with several times per compact are drawn at their own times; values without a time are drawn at the registered end of test. The matrix-corrected values of intact compacts are window upper bounds under premise BP1 (the source's assumption, not an established fact): fractions of the as-run calculated compact inventory over a window from 24 h at temperature to the end of the 300 h plateau (or the end of the test if shorter), drawn as bounds at the window end beside the model's value over the same window, not against the cumulative curve. The model has no decay: every panel is the element, and the nuclides measured are listed in the panel. Analytical-series values are drawn only where the criterion holds: {}. {} ",
        sources_of(&placed),
        crate::studies::ANALYTIC_CRITERION,
        EARLY_TIME_NOTE
    );
    let beyond = beyond_axis_note(
        placed
            .iter()
            .filter(|p| p.y > 1e-10)
            .map(|p| (p.id.clone(), p.t_h)),
    );
    if !beyond.is_empty() {
        text.push_str(&beyond);
        text.push(' ');
    }
    text.push_str(&provenance_line(inp.prov));
    footer(&root, y + 14, 215, &text)?;
    finish(root)
}

/// Figures B and C share a layout: a left panel of point deviations and a right panel of bound
/// margins.
fn deviation_figure(
    path: &Path,
    inp: &Inputs<'_>,
    name: &str,
    sub: &str,
    relative: bool,
) -> Result<(), String> {
    let root = SVGBackend::new(path, (1560, 700)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    title(&root, name, sub)?;
    let areas = root.margin(70, 240, 10, 10).split_evenly((1, 2));

    // Left: point deviations of the rows with a point comparison only. A row whose denominator
    // is not established (DATA INCOMPLETE) has no deviation and is not drawn here.
    let mut pts: Vec<(&RowRecord, f64, f64)> = Vec::new();
    for r in inp.records {
        if let (Eval::Point { dev, .. }, Some(m)) = (&r.eval, &r.model) {
            let y = if relative {
                dev.relative_percent
            } else {
                Some(dev.absolute)
            };
            if let Some(y) = y {
                if y > 0.0 {
                    pts.push((r, m.spec.t_end_h, y));
                }
            }
        }
    }
    let n_comparisons = inp
        .records
        .iter()
        .filter(|r| matches!(r.eval, Eval::Point { .. }))
        .count();
    let (ylo, yhi) = range_of(pts.iter().map(|p| p.2), 1e-12);
    let (ylo, yhi) = if relative {
        (ylo.min(1.0), yhi.max(1e4))
    } else {
        (ylo.min(1e-8), yhi.max(1.0))
    };
    {
        let mut chart = ChartBuilder::on(&areas[0])
            .caption(
                if relative {
                    "Relative percentage deviation of point values"
                } else {
                    "Absolute deviation of point values"
                },
                ts(19),
            )
            .margin(8)
            .x_label_area_size(44)
            .y_label_area_size(80)
            .build_cartesian_2d(0.0f64..420.0f64, (ylo..yhi).log_scale())
            .map_err(pe)?;
        chart
            .configure_mesh()
            .x_desc("Time at temperature (h)")
            .y_desc(if relative {
                "Relative deviation (percent)"
            } else {
                "Absolute deviation (dimensionless)"
            })
            .x_labels(8)
            .x_label_formatter(&|v| format!("{v:.0}"))
            .y_label_formatter(&sci)
            .light_line_style(RGBColor(235, 235, 235))
            .label_style(ts(14))
            .axis_desc_style(ts(16))
            .draw()
            .map_err(pe)?;
        for (r, t, y) in &pts {
            let c = temp_colour(r.model.as_ref().map_or(1600.0, |m| m.spec.t_c));
            mark!(chart, Glyph::Dot { c }, vec![(*t, *y)]);
        }
        let msg = if n_comparisons == 0 {
            "NO ROW HAS A POINT COMPARISON AVAILABLE".to_string()
        } else {
            format!("{n_comparisons} point comparisons")
        };
        chart
            .draw_series(std::iter::once(Text::new(
                msg,
                (210.0, ylo * 3.0),
                ts_pos(16, HPos::Center, VPos::Bottom),
            )))
            .map_err(pe)?;
        if pts.is_empty() {
            chart
                .draw_series(std::iter::once(Text::new(
                    "upper bounds and rows without a point comparison have no deviation (decisions 22 and 67)",
                    (210.0, (ylo * yhi).sqrt()),
                    ts_pos(16, HPos::Center, VPos::Center),
                )))
                .map_err(pe)?;
        }
    }

    // Right: bound margins (relative figure: signed log10 ratio instead of the ratio axis).
    let mut bounds: Vec<(&RowRecord, f64, f64)> = Vec::new();
    for r in inp.records {
        if let (Eval::Bound { margin, .. }, Some(m)) = (&r.eval, &r.model) {
            if margin.ratio > 0.0 {
                bounds.push((r, m.spec.t_end_h, margin.ratio));
            }
        }
    }
    let (blo, bhi) = range_of(bounds.iter().map(|b| b.2), 1e-12);
    let (blo, bhi) = (blo.min(1e-4), bhi.max(10.0));
    {
        let mut chart = ChartBuilder::on(&areas[1])
            .caption(
                "Model value over upper bound (inequality, not a deviation)",
                ts(19),
            )
            .margin(8)
            .x_label_area_size(44)
            .y_label_area_size(80)
            .build_cartesian_2d(0.0f64..420.0f64, (blo..bhi).log_scale())
            .map_err(pe)?;
        chart
            .configure_mesh()
            .x_desc("Time at temperature (h)")
            .y_desc("Model over reported upper bound (dimensionless)")
            .x_labels(8)
            .x_label_formatter(&|v| format!("{v:.0}"))
            .y_label_formatter(&sci)
            .light_line_style(RGBColor(235, 235, 235))
            .label_style(ts(14))
            .axis_desc_style(ts(16))
            .draw()
            .map_err(pe)?;
        chart
            .draw_series(LineSeries::new(
                vec![(0.0, 1.0), (420.0, 1.0)],
                shape(VERMILION, false, 2),
            ))
            .map_err(pe)?;
        chart
            .draw_series(std::iter::once(Text::new(
                "bound reached (ratio 1): above this line the model exceeds the bound",
                (4.0, 1.25),
                ts_pos(14, HPos::Left, VPos::Bottom),
            )))
            .map_err(pe)?;
        for (r, t, y) in &bounds {
            let c = temp_colour(r.model.as_ref().map_or(1600.0, |m| m.spec.t_c));
            let ok = matches!(r.status, Status::WithinReportedUpperBound);
            mark!(
                chart,
                Glyph::BoundDown {
                    c: if ok { c } else { VERMILION },
                    filled: ok
                },
                vec![(*t, *y)]
            );
        }
    }
    let mut items: Vec<Entry> = Vec::new();
    if !pts.is_empty() {
        items.push(entry(
            "Filled circle: a point comparison (COMPARISON AVAILABLE)",
            Glyph::Dot { c: INK },
        ));
    }
    let satisfied: Vec<&RowRecord> = bounds
        .iter()
        .map(|b| b.0)
        .filter(|r| matches!(r.status, Status::WithinReportedUpperBound))
        .collect();
    if !satisfied.is_empty() {
        items.push(entry(
            "Filled triangle pointing down: upper bound satisfied by the model",
            Glyph::BoundDown {
                c: INK,
                filled: true,
            },
        ));
    }
    if bounds
        .iter()
        .any(|b| matches!(b.0.status, Status::ExceedsReportedUpperBound))
    {
        items.push(entry(
            "Hollow vermilion triangle pointing down: upper bound exceeded by the model",
            Glyph::BoundDown {
                c: VERMILION,
                filled: false,
            },
        ));
    } // Only the temperatures of marks actually drawn get a legend entry.
    let drawn: Vec<f64> = pts
        .iter()
        .map(|p| p.0)
        .chain(satisfied.iter().copied())
        .filter_map(|r| r.model.as_ref().map(|m| m.spec.t_c))
        .collect();
    for t in TEMPERATURES {
        if drawn.iter().any(|d| (d - t).abs() < 1.0) {
            items.push(entry(
                format!("Colour: {t:.0} °C test temperature"),
                Glyph::Dot { c: temp_colour(t) },
            ));
        }
    }
    let y = legend(&root, 40, 480, 760, 2, &items)?;
    let mut text = String::from(
        "Relative deviation is undefined for a measured value that is not positive and no near-zero floor is applied, because no row states a detection floor (decision 17); no accuracy threshold is used. A row whose denominator is not established (DATA INCOMPLETE, decision 22) has no deviation and is not drawn; its measured and model values are in figure A. Upper bounds are inequalities and never enter a deviation; an exceeded bound is drawn in vermilion whatever the test temperature. ",
    );
    text.push_str(&provenance_line(inp.prov));
    footer(&root, y + 14, 215, &text)?;
    finish(root)
}

/// Figure B: absolute deviation.
pub fn figure_b(path: &Path, inp: &Inputs<'_>) -> Result<(), String> {
    deviation_figure(
        path,
        inp,
        "B. Absolute deviation versus time",
        "Absolute difference between model and measurement where a model value was computed; bounds are shown as a margin, not as a deviation",
        false,
    )
}

/// Figure C: relative deviation.
pub fn figure_c(path: &Path, inp: &Inputs<'_>) -> Result<(), String> {
    deviation_figure(
        path,
        inp,
        "C. Relative percentage deviation versus time",
        "Relative deviation is shown only where the measured value is positive; the same rows as figure B",
        true,
    )
}

/// Figure D: radial profiles.
pub fn figure_d(path: &Path, inp: &Inputs<'_>) -> Result<(), String> {
    let root = SVGBackend::new(path, (1560, 650)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    title(
        &root,
        "D. Radial concentration profiles through the five layers",
        "BTCS, 1600 °C, kernel-only start; concentration over the initial kernel concentration; shaded bands are the layers",
    )?;
    let areas = root.margin(70, 190, 10, 10).split_evenly((1, 3));
    let times = [1.0, 10.0, 24.0, 100.0, 300.0];
    let tcols = [SKY, BLUE, GREEN, ORANGE, VERMILION];
    let names = ["Kernel", "Buffer", "IPyC", "SiC", "OPyC"];
    let r_max = inp.radii_um[4];
    for (k, species) in Species::ALL.iter().enumerate() {
        let mut chart = ChartBuilder::on(&areas[k])
            .caption(panel_title(*species), ts(20))
            .margin(8)
            .x_label_area_size(44)
            .y_label_area_size(76)
            .build_cartesian_2d(0.0f64..r_max, (1e-12f64..3.0f64).log_scale())
            .map_err(pe)?;
        chart
            .configure_mesh()
            .x_desc("Radius (µm)")
            .y_desc("Concentration over initial kernel concentration (dimensionless)")
            .x_labels(8)
            .y_labels(14)
            .x_label_formatter(&|v| format!("{v:.0}"))
            .y_label_formatter(&sci)
            .light_line_style(TRANSPARENT)
            .label_style(ts(14))
            .axis_desc_style(ts(15))
            .draw()
            .map_err(pe)?;
        // layer bands
        let mut lo = 0.0;
        for (i, name) in names.iter().enumerate() {
            let hi = inp.radii_um[i];
            let fill = if i % 2 == 0 {
                RGBColor(238, 238, 238)
            } else {
                RGBColor(250, 250, 250)
            };
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(lo, 1e-12), (hi, 3.0)],
                    shape(fill, true, 1),
                )))
                .map_err(pe)?;
            chart
                .draw_series(std::iter::once(Text::new(
                    name.to_string(),
                    ((lo + hi) / 2.0, 2.2),
                    ts_pos(12, HPos::Center, VPos::Top),
                )))
                .map_err(pe)?;
            lo = hi;
        }
        for (j, t) in times.iter().enumerate() {
            let pts: Vec<(f64, f64)> = inp
                .profiles
                .iter()
                .filter(|p| {
                    p.species == *species
                        && (p.t_h - t).abs() < 0.2 * (t / 10.0).max(0.05)
                        && p.c_over_c0 > 1e-12
                })
                .map(|p| (p.r_um, p.c_over_c0))
                .collect();
            line!(chart, pts, tcols[j], None::<(u32, u32)>, 3);
        }
    }
    let items: Vec<Entry> = times
        .iter()
        .enumerate()
        .map(|(j, t)| {
            entry(
                format!("{t:.0} h at temperature"),
                Glyph::Line {
                    c: tcols[j],
                    dash: None,
                },
            )
        })
        .collect();
    let y = legend(&root, 40, 480, 300, 5, &items)?;
    let text = format!(
        "Concentrations at or below 1e-12 of the initial value are not drawn (log scale; the outer node is zero by the absorbing boundary). The profile at the node closest to each requested time is shown; values are the numerical solution of the stated model. {}",
        provenance_line(inp.prov)
    );
    footer(&root, y + 14, 215, &text)?;
    finish(root)
}

/// Figure E: inventory balance.
pub fn figure_e(path: &Path, inp: &Inputs<'_>) -> Result<(), String> {
    let root = SVGBackend::new(path, (1560, 690)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    title(
        &root,
        "E. Inventory balance",
        "Remaining inventory plus cumulative release, and the mass-balance residual with its derived rounding bound",
    )?;
    let areas = root.margin(70, 230, 10, 10).split_evenly((1, 2));
    let sample = inp
        .curves
        .iter()
        .find(|(s, t, _)| *s == Species::Ag && (*t - 1800.0).abs() < 1.0)
        .map(|(_, _, c)| *c);
    if let Some(c) = sample {
        let mut chart = ChartBuilder::on(&areas[0])
            .caption("Ag-110m, 1800 °C: remaining and released fractions", ts(19))
            .margin(8)
            .x_label_area_size(44)
            .y_label_area_size(70)
            .build_cartesian_2d(0.0f64..420.0f64, 0.0f64..1.05f64)
            .map_err(pe)?;
        chart
            .configure_mesh()
            .x_desc("Time at temperature (h)")
            .y_desc("Fraction of the initial inventory (dimensionless)")
            .x_labels(8)
            .x_label_formatter(&|v| format!("{v:.0}"))
            .light_line_style(RGBColor(235, 235, 235))
            .label_style(ts(14))
            .axis_desc_style(ts(16))
            .draw()
            .map_err(pe)?;
        line!(
            chart,
            c.t_h
                .iter()
                .zip(&c.remaining)
                .map(|(t, v)| (*t, *v))
                .collect(),
            BLUE,
            None::<(u32, u32)>,
            3
        );
        line!(
            chart,
            c.t_h.iter().zip(&c.f).map(|(t, v)| (*t, *v)).collect(),
            VERMILION,
            Some((10, 6)),
            3
        );
        line!(
            chart,
            c.t_h
                .iter()
                .zip(c.remaining.iter().zip(&c.f))
                .map(|(t, (a, b))| (*t, a + b))
                .collect(),
            INK,
            Some((2, 5)),
            2
        );
        chart
            .draw_series(std::iter::once(Text::new(
                "Generated inventory: not applicable (no source term in the solver yet)",
                (200.0, 0.62),
                ts_pos(13, HPos::Center, VPos::Top),
            )))
            .map_err(pe)?;
        chart
            .draw_series(std::iter::once(Text::new(
                "Decay loss: not applicable (no decay in the solver yet)",
                (200.0, 0.55),
                ts_pos(13, HPos::Center, VPos::Top),
            )))
            .map_err(pe)?;
    }
    {
        let mut series: Vec<ResidualSeries> = Vec::new();
        for (s, t, c) in inp
            .curves
            .iter()
            .filter(|(_, t, _)| (*t - 1600.0).abs() < 1.0)
        {
            let res: Vec<(f64, f64)> = c
                .t_h
                .iter()
                .zip(&c.residual_rel)
                .filter(|(t, r)| **t > 0.0 && r.abs() > 0.0)
                .map(|(t, r)| (*t, r.abs()))
                .collect();
            let bnd: Vec<(f64, f64)> = c
                .t_h
                .iter()
                .zip(&c.bound_rel)
                .filter(|(t, b)| **t > 0.0 && **b > 0.0)
                .map(|(t, b)| (*t, *b))
                .collect();
            series.push((*s, *t, res, bnd));
        }
        let all = series
            .iter()
            .flat_map(|s| s.2.iter().chain(s.3.iter()))
            .map(|p| p.1);
        let (lo, hi) = range_of(all, 1e-20);
        let mut chart = ChartBuilder::on(&areas[1])
            .caption(
                "Relative balance residual (solid) and its derived bound (dashed), 1600 °C",
                ts(19),
            )
            .margin(8)
            .x_label_area_size(44)
            .y_label_area_size(80)
            .build_cartesian_2d(0.0f64..420.0f64, (lo..hi).log_scale())
            .map_err(pe)?;
        chart
            .configure_mesh()
            .x_desc("Time at temperature (h)")
            .y_desc("Relative balance residual of the inventory (dimensionless)")
            .x_labels(8)
            .x_label_formatter(&|v| format!("{v:.0}"))
            .y_label_formatter(&sci)
            .light_line_style(RGBColor(235, 235, 235))
            .label_style(ts(14))
            .axis_desc_style(ts(16))
            .draw()
            .map_err(pe)?;
        let cols = [BLUE, GREEN, VERMILION];
        for (k, (_, _, res, bnd)) in series.iter().enumerate() {
            line!(chart, res.clone(), cols[k % 3], None::<(u32, u32)>, 2);
            line!(chart, bnd.clone(), cols[k % 3], Some((8, 5)), 2);
        }
    }
    let mut items = vec![
        entry(
            "Remaining inventory over initial inventory (dimensionless)",
            Glyph::Line {
                c: BLUE,
                dash: None,
            },
        ),
        entry(
            "Cumulative release over initial inventory (dimensionless)",
            Glyph::Line {
                c: VERMILION,
                dash: Some((10, 6)),
            },
        ),
        entry(
            "Sum of both (equals one up to the residual)",
            Glyph::Line {
                c: INK,
                dash: Some((2, 5)),
            },
        ),
    ];
    let cols = [BLUE, GREEN, VERMILION];
    for (k, s) in Species::ALL.iter().enumerate() {
        items.push(entry(
            format!("{} residual (solid) and bound (dashed)", s.nuclide()),
            Glyph::Line {
                c: cols[k],
                dash: None,
            },
        ));
    }
    let y = legend(&root, 40, 480, 760, 2, &items)?;
    let text = format!(
        "The residual is the observed value; the bound is the derived a-posteriori rounding bound of the solver (decision 6), shown beside it and never tightened. {}",
        provenance_line(inp.prov)
    );
    footer(&root, y + 14, 215, &text)?;
    finish(root)
}

fn convergence_figure(
    path: &Path,
    inp: &Inputs<'_>,
    heading: (&str, &str),
    rows: &[ConvergenceRow],
    x_desc: &str,
    ref_order: f64,
    floor: Option<(f64, &str)>,
) -> Result<(), String> {
    let root = SVGBackend::new(path, (1200, 690)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    title(&root, heading.0, heading.1)?;
    let area = root.margin(70, 220, 10, 10);
    let pts: Vec<(f64, f64)> = rows
        .iter()
        .filter(|r| r.error > 0.0)
        .map(|r| (r.step, r.error))
        .collect();
    let (xlo, xhi) = range_of(pts.iter().map(|p| p.0), 1e-30);
    let (ylo, yhi) = range_of(
        pts.iter()
            .map(|p| p.1)
            .chain(floor.map(|f| f.0).into_iter().filter(|f| *f > 0.0)),
        1e-30,
    );
    let mut chart = ChartBuilder::on(&area)
        .margin(8)
        .x_label_area_size(48)
        .y_label_area_size(86)
        .build_cartesian_2d((xlo..xhi).log_scale(), (ylo..yhi).log_scale())
        .map_err(pe)?;
    chart
        .configure_mesh()
        .x_desc(x_desc)
        .y_desc("Absolute error of the release fraction at 100 h (dimensionless)")
        .y_label_formatter(&sci)
        .x_labels(5)
        .x_label_formatter(&|v| format!("{v:.1e}"))
        .light_line_style(RGBColor(235, 235, 235))
        .label_style(ts(14))
        .axis_desc_style(ts(16))
        .draw()
        .map_err(pe)?;
    line!(chart, pts.clone(), BLUE, None::<(u32, u32)>, 3);
    mark!(chart, Glyph::Dot { c: BLUE }, pts.clone());
    // reference slope through the finest point
    if let Some(last) = pts.last() {
        let slope: Vec<(f64, f64)> = [xlo, xhi]
            .iter()
            .map(|x| (*x, last.1 * (x / last.0).powf(ref_order)))
            .collect();
        line!(chart, slope, GREY, Some((8, 6)), 2);
    }
    if let Some((f, _)) = floor {
        if f > 0.0 {
            line!(chart, vec![(xlo, f), (xhi, f)], VERMILION, Some((3, 5)), 2);
        }
    }
    for r in rows {
        if let Some(o) = r.order {
            chart
                .draw_series(std::iter::once(Text::new(
                    format!("order {o:.2}"),
                    (r.step, r.error * 1.8),
                    ts_pos(12, HPos::Center, VPos::Bottom),
                )))
                .map_err(pe)?;
        }
    }
    let mut items = vec![
        entry("BTCS error against the analytical series", Glyph::Line { c: BLUE, dash: None }),
        entry(
            format!("Reference slope of order {ref_order} (grey, dashed), drawn through the finest point"),
            Glyph::Line { c: GREY, dash: Some((8, 6)) },
        ),
    ];
    if let Some((f, label)) = floor {
        if f > 0.0 {
            items.push(entry(
                label.to_string(),
                Glyph::Line {
                    c: VERMILION,
                    dash: Some((3, 5)),
                },
            ));
        }
    }
    let y = legend(&root, 40, 490, 1100, 1, &items)?;
    let text = format!(
        "Verification case: {} It is a numerical verification case, not physical data. The orders printed are observed between neighbouring levels; they are reported, not required to meet a target (decision 7). {}",
        inp.homogeneous_note,
        verification_line(inp.prov)
    );
    footer(&root, y + 12, 170, &text)?;
    finish(root)
}

/// Figure F: spatial convergence.
pub fn figure_f(path: &Path, inp: &Inputs<'_>) -> Result<(), String> {
    convergence_figure(
        path,
        inp,
        (
            "F. Spatial convergence",
            "Error of the release fraction against mesh width; the time-step error is removed by Richardson extrapolation of two step sizes",
        ),
        inp.spatial,
        "Kernel mesh width (m)",
        2.0,
        None,
    )
}

/// Figure G: temporal convergence.
pub fn figure_g(path: &Path, inp: &Inputs<'_>) -> Result<(), String> {
    convergence_figure(
        path,
        inp,
        (
            "G. Temporal convergence",
            "Error of the release fraction against the time step on a fixed fine mesh; the dotted line is the spatial error of that mesh, the level the curve approaches",
        ),
        inp.temporal,
        "Time step (s)",
        1.0,
        Some((inp.temporal_spatial_floor, "Spatial error of the fixed mesh (from the spatial study)")),
    )
}

/// Figure H: temperature sensitivity.
pub fn figure_h(path: &Path, inp: &Inputs<'_>) -> Result<(), String> {
    let root = SVGBackend::new(path, (1560, 860)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    title(
        &root,
        "H. Temperature sensitivity of the registered diffusion correlations",
        "Release fraction at 300 h at temperature against test temperature, one curve per registered coefficient variant; log scale",
    )?;
    let areas = root.margin(70, 390, 10, 10).split_evenly((1, 3));
    // One style per variant, unique over the whole figure: eight colour and dash pairs, none
    // repeated, so no two variants of any panel share a style.
    let styles: [(RGBColor, Option<(u32, u32)>); 8] = [
        (BLUE, None),
        (VERMILION, Some((10, 6))),
        (GREEN, Some((2, 5))),
        (ORANGE, Some((14, 4))),
        (PURPLE, None),
        (SKY, Some((6, 3))),
        (INK, Some((10, 6))),
        (GREY, None),
    ];
    let mut variant_ids: Vec<String> = Vec::new();
    for r in inp.sensitivity {
        if !variant_ids.contains(&r.variant_id) {
            variant_ids.push(r.variant_id.clone());
        }
    }
    let mut legend_entries: Vec<Entry> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for (k, species) in Species::ALL.iter().enumerate() {
        let rows: Vec<&SensitivityRow> = inp
            .sensitivity
            .iter()
            .filter(|r| r.species == *species)
            .collect();
        let (lo, hi) = range_of(rows.iter().map(|r| r.f_300), 1e-14);
        let mut chart = ChartBuilder::on(&areas[k])
            .caption(panel_title(*species), ts(20))
            .margin(8)
            .x_label_area_size(44)
            .y_label_area_size(80)
            .build_cartesian_2d(
                1590.0f64..1810.0f64,
                (lo.min(1e-6)..hi.max(1.0)).log_scale(),
            )
            .map_err(pe)?;
        chart
            .configure_mesh()
            .x_desc("Test temperature (°C)")
            .y_desc("Fraction released after 300 h (dimensionless)")
            .x_labels(6)
            .x_label_formatter(&|v| format!("{v:.0}"))
            .y_label_formatter(&sci)
            .light_line_style(RGBColor(235, 235, 235))
            .label_style(ts(14))
            .axis_desc_style(ts(16))
            .draw()
            .map_err(pe)?;
        let mut ids: Vec<String> = Vec::new();
        for r in &rows {
            if !ids.contains(&r.variant_id) {
                ids.push(r.variant_id.clone());
            }
        }
        for (j, id) in ids.iter().enumerate() {
            let pts: Vec<(f64, f64)> = rows
                .iter()
                .filter(|r| &r.variant_id == id && r.f_300 > 0.0)
                .map(|r| (r.t_c, r.f_300))
                .collect();
            let _ = j;
            let g = variant_ids.iter().position(|v| v == id).unwrap_or(0);
            let (c, dash) = styles[g % styles.len()];
            line!(chart, pts.clone(), c, dash, 3);
            mark!(chart, Glyph::Dot { c }, pts);
            if !seen.contains(id) {
                seen.push(id.clone());
                let label = rows
                    .iter()
                    .find(|r| &r.variant_id == id)
                    .map(|r| format!("{}: {}", r.variant_id, r.variant_label))
                    .unwrap_or_default();
                legend_entries.push(entry(label, Glyph::Line { c, dash }));
            }
        }
    }
    let y = legend(&root, 40, 490, 1500, 1, &legend_entries)?;
    let has_s20 = inp
        .sensitivity
        .iter()
        .any(|r| r.variant_id.starts_with("S20-fitted"));
    let outside = inp
        .sensitivity
        .iter()
        .filter(|r| r.outside_printed_range)
        .count();
    let text = format!(
        "The default correlation prints no validity range, so the sweep covers the registered heating-test temperatures 1600 to 1800 °C; of {} sweep points, {} lie outside a printed range of a row they use. Variants other than the published default are labelled alternatives, never defaults (decisions 40 and 41); the class of each coefficient is in the sensitivity file. {}{}",
        inp.sensitivity.len(),
        outside,
        if has_s20 {
            "The AGR-1 fitted silicon-carbide rows of Schappel (2020, S20-fitted-SiC) are fitted to AGR-1 heating-test data and disagree with the correction factors of Collin (2016) by 4 to 220 times (decision 40). "
        } else {
            ""
        },
        provenance_line(inp.prov)
    );
    footer(&root, y + 12, 215, &text)?;
    finish(root)
}

/// A number to three significant figures in scientific notation, for the text of the figures.
fn three(x: f64) -> String {
    format!("{x:.3e}")
}

/// The element a register row measures, if it is one of the three the model has coefficients for.
fn row_species(r: &RowRecord) -> Option<Species> {
    match &r.row.measured {
        Measured::Nuclide(n) => Species::of_element(&n.element),
        Measured::Element(e) => Species::of_element(e),
        Measured::NotApplicable => None,
    }
}

/// The lines of text printed under a nuclide figure: what the register holds for the nuclide, why
/// rows are or are not comparable, and every numerical deviation that is defined.
pub fn deviation_lines(records: &[RowRecord], species: Species) -> Vec<String> {
    let nuc = species.nuclide();
    let rows: Vec<&RowRecord> = records
        .iter()
        .filter(|r| row_species(r) == Some(species))
        .collect();
    let digitised = rows
        .iter()
        .filter(|r| !matches!(r.row.reading_uncertainty, ReadingUncertainty::NotDigitised))
        .count();
    let mut out = vec![format!(
        "Register rows of {nuc} (including rows labelled with the element only): {}; {} digitised from published figures (labelled digitised, with their reading uncertainty), {} tabulated or printed and exact as printed.",
        rows.len(),
        digitised,
        rows.len() - digitised
    )];
    let counts: Vec<String> = Status::ALL
        .iter()
        .filter_map(|s| {
            let n = rows.iter().filter(|r| r.status == *s).count();
            (n > 0).then(|| format!("{} {n}", s.label()))
        })
        .collect();
    out.push(format!(
        "Comparison status of these rows: {}.",
        counts.join("; ")
    ));
    let mut reasons: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for r in rows
        .iter()
        .filter(|r| r.status == Status::NotDirectlyComparable)
    {
        *reasons.entry(r.verdict.category).or_insert(0) += 1;
    }
    if !reasons.is_empty() {
        let text: Vec<String> = reasons
            .iter()
            .map(|(k, n)| {
                let meaning = crate::status::CATEGORIES
                    .iter()
                    .find(|(c, _)| c == k)
                    .map_or("", |(_, m)| *m);
                format!("{n} x {k} ({meaning})")
            })
            .collect();
        out.push(format!(
            "NOT DIRECTLY COMPARABLE because the measured quantity is not the model quantity: {}.",
            text.join("; ")
        ));
    }
    let mut pairs: Vec<(f64, f64)> = Vec::new();
    let mut any_scale = false;
    for r in &rows {
        let Some(m) = &r.model else { continue };
        let at = format!(
            "{} | {} \u{b0}C | {} h at temperature",
            r.row.id, m.spec.t_c, m.spec.t_end_h
        );
        match &r.eval {
            Eval::Point { value, dev } => {
                pairs.push((m.f_window, *value));
                out.push(format!(
                    "{at} | measured {} | model {} | absolute deviation {} | relative deviation {} % | signed log10 ratio {}",
                    three(*value),
                    three(m.f_window),
                    three(dev.absolute),
                    dev.relative_percent
                        .map_or("undefined".to_string(), three),
                    dev.log10_ratio
                        .map_or("undefined".to_string(), three),
                ));
            }
            Eval::Bound {
                limit,
                margin,
                source: crate::status::BoundSource::WindowUnderBp1,
            } => {
                out.push(format!(
                    "{} | {} \u{b0}C | window {}-{} h at temperature | bound {} (digitised, matrix-corrected fraction of the as-run calculated compact inventory) | model over the same window {} | model over bound {} | {}: not a point comparison, so no deviation, no relative deviation and no whole-curve metric is formed",
                    r.row.id,
                    m.spec.t_c,
                    m.spec.window_start_h,
                    m.spec.t_end_h,
                    three(*limit),
                    three(m.f_window),
                    three(margin.ratio),
                    crate::status::WINDOW_BOUND_CLASS,
                ));
            }
            Eval::Provisional { value, log10_ratio } => {
                any_scale = true;
                out.push(format!(
                    "{at} | measured {} (digitised) | model {} | signed log10 ratio {} for scale only: the denominator is not established (decision 22), so no absolute or relative deviation is formed",
                    three(*value),
                    three(m.f_window),
                    log10_ratio.map_or("undefined".to_string(), three),
                ));
            }
            _ => {}
        }
    }
    match crate::metrics::point_aggregate(&pairs) {
        Some(a) => out.push(format!(
            "Over {} point comparisons: largest absolute deviation {}; no aggregate deviation is formed (the whole-curve metric of the pipeline is R\u{b2}, in the results files).",
            a.n,
            three(a.max_absolute)
        )),
        None => out.push(format!(
            "No {nuc} row supports a point comparison, so the absolute deviation and the relative deviation in percent are not defined for {nuc}{}; no number is substituted.",
            if any_scale {
                " (the log10 ratios above are for scale only and enter no aggregate)"
            } else {
                ""
            }
        )),
    }
    out
}

/// Figures I and J: one nuclide, measurements with their reading uncertainty, the BTCS prediction,
/// the analytical prediction where it is valid, and the numerical deviations beside the curves.
fn nuclide_figure(
    path: &Path,
    inp: &Inputs<'_>,
    species: Species,
    heading: &str,
) -> Result<(), String> {
    let nuc = species.nuclide();
    let placed: Vec<Placed> = placed_rows(inp.reg, inp.records)
        .into_iter()
        .filter(|p| p.species == species)
        .collect();
    let mut text_lines: Vec<String> = Vec::new();
    for l in deviation_lines(inp.records, species) {
        text_lines.extend(wrap(&l, 190));
    }
    // The points of the right panel: the log10 ratio of the model to the measured value at the
    // end of the window, for every row that has a model value and a value to set against it.
    let pts: Vec<(String, f64, f64, bool)> = inp
        .records
        .iter()
        .filter(|r| row_species(r) == Some(species))
        .filter_map(|r| {
            let m = r.model.as_ref()?;
            match &r.eval {
                Eval::Point { dev, .. } => dev
                    .log10_ratio
                    .map(|l| (r.row.id.clone(), m.spec.t_end_h, l, true)),
                Eval::Provisional { log10_ratio, .. } => {
                    log10_ratio.map(|l| (r.row.id.clone(), m.spec.t_end_h, l, false))
                }
                _ => None,
            }
        })
        .collect();
    let has_point = pts.iter().any(|p| p.3);
    let beyond = beyond_axis_note(
        placed
            .iter()
            .filter(|p| p.y > 1e-10)
            .map(|p| (p.id.clone(), p.t_h))
            .chain(pts.iter().map(|p| (p.0.clone(), p.1))),
    );
    let provenance = format!(
        "Experimental values: {}. Only the rows whose status supports a comparison are drawn in colour; rows that are NOT DIRECTLY COMPARABLE are muted grey context, because they measure another quantity than the model computes (failed particles, totals that include release in the reactor, another fuel, fuel spheres). A bar on a marker is the reading uncertainty of the register (the error of reading the point off the published figure); it is not a measurement uncertainty, and the sources report no per-value measurement uncertainty for these rows (a general 5 percent systematic gamma-measurement error stated for the Stempien 2023 values is not a per-value uncertainty and changes no status), so no other error bar is drawn. Compact values are not single-particle values; rows at a temperature outside 1600 to 1800 \u{b0}C or whose value is not a fraction are not drawn. {beyond} Analytical-series values are drawn only where the criterion holds: {}. {} {}",
        sources_of(&placed),
        crate::studies::ANALYTIC_CRITERION,
        EARLY_TIME_NOTE,
        provenance_line(inp.prov)
    );
    let foot_lines = wrap(&provenance, 215).len() as i32;
    // 640 px of charts and titles, 6 legend rows of 26 px, the heading of the deviation lines, the
    // lines themselves and the footer.
    let height =
        640 + 10 + 26 * 6 + 12 + 26 + 19 * text_lines.len() as i32 + 14 + 18 * foot_lines + 30;
    let root = SVGBackend::new(path, (1560, height as u32)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let (heading_tail, subtitle_tail) = if has_point {
        (
            "and deviations",
            "and the deviations of the model from the measurements",
        )
    } else if pts.is_empty() {
        (
            "and why no deviation is defined",
            "and the reason why no deviation of the model from the measurements is defined",
        )
    } else {
        (
            "and the ratios of the model to the measurements, for scale only",
            "and the ratios of the model to the measurements, which are for scale only because no deviation is defined",
        )
    };
    title(
        &root,
        &format!("{heading}: measurements, BTCS prediction, analytical prediction {heading_tail}"),
        &format!(
            "Measurements of {nuc}, BTCS prediction, analytical prediction where valid, {subtitle_tail}; hours counted from reaching the test temperature"
        ),
    )?;
    let areas = root.margin(70, height - 640, 10, 10).split_evenly((1, 2));

    // Left: release fraction versus time.
    {
        let mut chart = ChartBuilder::on(&areas[0])
            .caption(
                format!("Fraction of {nuc} released versus time at temperature"),
                ts(19),
            )
            .margin(8)
            .x_label_area_size(44)
            .y_label_area_size(80)
            .build_cartesian_2d(0.0f64..420.0f64, (1e-10f64..2.0f64).log_scale())
            .map_err(pe)?;
        chart
            .configure_mesh()
            .x_desc("Time at temperature (h)")
            .y_desc(format!("Fraction of {nuc} released (dimensionless)"))
            .x_labels(8)
            .x_label_formatter(&|v| format!("{v:.0}"))
            .y_label_formatter(&sci)
            .light_line_style(RGBColor(235, 235, 235))
            .label_style(ts(14))
            .axis_desc_style(ts(16))
            .draw()
            .map_err(pe)?;
        for (_, t_c, c) in inp.curves.iter().filter(|(s, _, _)| *s == species) {
            line!(
                chart,
                curve_points(c),
                temp_colour(*t_c),
                temp_dash(*t_c),
                3
            );
            if let Some(an) = inp.analytic.get(&(species, (*t_c * 10.0).round() as i64)) {
                let pts: Vec<(f64, f64)> = an
                    .iter()
                    .filter_map(|(t, f)| f.filter(|f| *f > 1e-10).map(|f| (*t, f)))
                    .collect();
                mark!(
                    chart,
                    Glyph::Plus {
                        c: temp_colour(*t_c)
                    },
                    pts
                );
            }
        }
        // Reading bars first (a secondary attribute of a digitised value), then the markers: the
        // status decides the marker, never the way the value was obtained. Context first, then
        // the rows that are not comparable data, then the bounds and point comparisons on top.
        for pass in 0..3 {
            for p in placed.iter().filter(|p| drawable(p)) {
                if marker_pass(p) != pass {
                    continue;
                }
                if let Some((lo, hi)) = p.reading_interval {
                    let lo = lo.max(1e-10);
                    if hi > lo {
                        let c = if pass == 0 {
                            RGBColor(170, 170, 170)
                        } else {
                            temp_colour(p.t_c)
                        };
                        chart
                            .draw_series(LineSeries::new(
                                vec![(p.t_h, lo), (p.t_h, hi)],
                                shape(c, false, 1),
                            ))
                            .map_err(pe)?;
                    }
                }
                mark!(chart, glyph_of(p), vec![(p.t_h, p.y)]);
            }
        }
        // The model's own value over the window of a matrix-corrected measurement, beside it: that
        // measurement is a release over a window, not cumulative from t = 0.
        for p in placed.iter().filter(|p| drawable(p)) {
            if let Some(w) = p.model_window.filter(|w| *w > 1e-10) {
                mark!(
                    chart,
                    Glyph::Square {
                        c: temp_colour(p.t_c)
                    },
                    vec![(p.t_h, w)]
                );
            }
        }
        let drawn: Vec<&Placed> = placed.iter().filter(|p| drawable(p)).collect();
        let count = |f: &dyn Fn(&Placed) -> bool| drawn.iter().filter(|p| f(p)).count();
        let n_ctx = count(&|p| marker_pass(p) == 0);
        let n_inc = count(&|p| p.status == Status::DataIncomplete);
        let n_bound = count(&|p| p.is_bound && marker_pass(p) == 2);
        let n_point = count(&|p| marker_pass(p) == 2 && !p.is_bound);
        let n_dig = drawn.iter().filter(|p| p.digitised).count();
        let line1 = format!(
            "{} registered values drawn: {n_ctx} NOT DIRECTLY COMPARABLE (grey context), {n_inc} DATA INCOMPLETE",
            drawn.len()
        );
        let line2 = format!(
            "{n_bound} upper bounds, {n_point} point comparisons; {n_dig} of the drawn values digitised, {} tabulated or printed",
            drawn.len() - n_dig
        );
        for (txt, y) in [(line1, 5e-10), (line2, 1.6e-10)] {
            chart
                .draw_series(std::iter::once(Text::new(
                    txt,
                    (T_AXIS_H - 4.0, y),
                    ts_pos(13, HPos::Right, VPos::Bottom),
                )))
                .map_err(pe)?;
        }
    }

    // Right: the signed log10 ratio of the model to the measured value. Only a point comparison
    // is a deviation; the rows without an established denominator are shown for scale only.
    {
        let pts: Vec<(f64, f64, bool)> = pts
            .iter()
            .filter(|p| p.1 > 0.0 && p.1 <= T_AXIS_H)
            .map(|p| (p.1, p.2, p.3))
            .collect();
        let top = pts.iter().map(|p| p.1).fold(1.0f64, f64::max).ceil() + 0.5;
        let bottom = pts.iter().map(|p| p.1).fold(-1.0f64, f64::min).floor() - 0.5;
        let title_right = if has_point {
            format!("Signed log10 ratio, model over measured value, {nuc}")
        } else if pts.is_empty() {
            format!("Deviation of the model from the measurements, {nuc}: not defined")
        } else {
            format!("Log10 ratio, model over measured value, for scale only, {nuc}")
        };
        let mut chart = ChartBuilder::on(&areas[1])
            .caption(title_right, ts(19))
            .margin(8)
            .x_label_area_size(44)
            .y_label_area_size(80)
            .build_cartesian_2d(0.0f64..T_AXIS_H, bottom..top)
            .map_err(pe)?;
        chart
            .configure_mesh()
            .x_desc("Time at temperature (h)")
            .y_desc("Signed log10 of (model fraction / measured fraction) (dimensionless)")
            .x_labels(8)
            .x_label_formatter(&|v| format!("{v:.0}"))
            .light_line_style(RGBColor(235, 235, 235))
            .label_style(ts(14))
            .axis_desc_style(ts(16))
            .draw()
            .map_err(pe)?;
        chart
            .draw_series(LineSeries::new(
                vec![(0.0, 0.0), (T_AXIS_H, 0.0)],
                shape(GREY, false, 1),
            ))
            .map_err(pe)?;
        if !pts.is_empty() && !has_point {
            chart
                .draw_series(std::iter::once(Text::new(
                    "no deviation is defined for these rows: the denominator of the fraction is not established",
                    (T_AXIS_H / 2.0, top - 0.1),
                    ts_pos(13, HPos::Center, VPos::Top),
                )))
                .map_err(pe)?;
        }
        for (t, l, is_point) in &pts {
            if *is_point {
                mark!(chart, Glyph::Dot { c: INK }, vec![(*t, *l)]);
            } else {
                mark!(chart, Glyph::Diamond { c: INK }, vec![(*t, *l)]);
            }
        }
        if pts.is_empty() {
            chart
                .draw_series(std::iter::once(Text::new(
                    format!("NO DEVIATION IS DEFINED FOR {nuc}"),
                    (210.0, (top + bottom) / 2.0 + 0.4),
                    ts_pos(18, HPos::Center, VPos::Center),
                )))
                .map_err(pe)?;
            chart
                .draw_series(std::iter::once(Text::new(
                    "no row is a point comparison; no whole-curve metric is formed (see the lines below)",
                    (210.0, (top + bottom) / 2.0 - 0.4),
                    ts_pos(14, HPos::Center, VPos::Center),
                )))
                .map_err(pe)?;
        }
    }

    let mut items = temperature_legend();
    items.push(entry(
        "Plus: analytical prediction (independent series), only where valid",
        Glyph::Plus { c: INK },
    ));
    items.extend(status_legend(&placed));
    if placed
        .iter()
        .any(|p| drawable(p) && p.model_window.is_some_and(|w| w > 1e-10))
    {
        items.push(entry(
            "Hollow square: model over the same window (24 h to the plateau end) as a window upper bound",
            Glyph::Square { c: INK },
        ));
    }
    items.push(entry(
        "Vertical bar: reading uncertainty of a digitised value; no bar: printed value, exact",
        Glyph::Line {
            c: GREY,
            dash: None,
        },
    ));
    if has_point {
        items.push(entry(
            "Right panel, filled circle: a point comparison",
            Glyph::Dot { c: INK },
        ));
    }
    items.push(entry(
        "Marker colour: test temperature, as the line of the same colour",
        Glyph::Line {
            c: GREY,
            dash: None,
        },
    ));
    let y = legend(&root, 40, 650, 760, 2, &items)?;
    let mut ty = y + 12;
    root.draw(&Text::new(
        format!("Numerical deviations of the model from the measurements of {nuc}"),
        (24, ty),
        ts(17),
    ))
    .map_err(pe)?;
    ty += 26;
    for l in &text_lines {
        root.draw(&Text::new(l.clone(), (24, ty), ts(14)))
            .map_err(pe)?;
        ty += 19;
    }
    footer(&root, ty + 14, 215, &provenance)?;
    finish(root)
}

/// Figure I: silver, Ag-110m.
pub fn figure_i(path: &Path, inp: &Inputs<'_>) -> Result<(), String> {
    nuclide_figure(path, inp, Species::Ag, "I. Silver-110m (Ag-110m)")
}

/// Figure J: strontium, Sr-90.
pub fn figure_j(path: &Path, inp: &Inputs<'_>) -> Result<(), String> {
    nuclide_figure(path, inp, Species::Sr, "J. Strontium-90 (Sr-90)")
}

/// Figure K: caesium-137, for which AGR-1 and AGR-2 report no release (decision 43).
pub fn figure_k(path: &Path, inp: &Inputs<'_>) -> Result<(), String> {
    let rows: Vec<&RowRecord> = inp
        .records
        .iter()
        .filter(|r| matches!(&r.row.measured, Measured::Nuclide(n) if n.name() == "Cs-137"))
        .collect();
    let agr1 = rows
        .iter()
        .filter(|r| r.row.program.as_str() == "AGR-1" || r.row.program.as_str() == "AGR-2")
        .count();
    let mut lines = vec![format!(
        "AGR-1 and AGR-2 report no release of Cs-137 (decision 43): the register holds {agr1} Cs-137 rows of those programmes."
    )];
    let mut by: std::collections::BTreeMap<(String, String, &str), usize> =
        std::collections::BTreeMap::new();
    for r in &rows {
        *by.entry((
            r.row.program.as_str().to_string(),
            r.row.source.clone(),
            r.status.label(),
        ))
        .or_insert(0) += 1;
    }
    for ((program, source, status), n) in &by {
        let why = if *status == Status::DataIncomplete.label() {
            "; they are not a release measurement of the AGR-1 particles, their denominator is not established, and no deviation is formed from them"
        } else {
            ""
        };
        lines.push(format!(
            "The register holds {n} Cs-137 rows of the programme {program} ({source}) with the status {status}{why}."
        ));
    }
    let n_point = rows
        .iter()
        .filter(|r| matches!(r.eval, Eval::Point { .. }))
        .count();
    lines.push(format!(
        "The model computes the element caesium without decay (decay is not in the solver yet): it gives the release of the element, not of a nuclide. Figure A draws the registered Cs-137 rows of the other programme against the caesium curve with the marker of their status; of the {} Cs-137 rows, {n_point} support a point comparison.",
        rows.len()
    ));
    let mut text_lines: Vec<String> = Vec::new();
    for l in &lines {
        text_lines.extend(wrap(l, 190));
    }
    let provenance = provenance_line(inp.prov);
    let foot = wrap(&provenance, 215).len() as i32;
    let height = 255 + 19 * text_lines.len() as i32 + 14 + 18 * foot + 30;
    let root = SVGBackend::new(path, (1560, height as u32)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    title(
        &root,
        "K. Caesium-137 (Cs-137): DATA UNAVAILABLE",
        "No release measurement of Cs-137 exists for the AGR-1 particles in the sources of the register; no curve, no deviation and no metric is drawn",
    )?;
    root.draw(&Rectangle::new(
        [(24, 80), (1536, 230)],
        shape(GREY, false, 2),
    ))
    .map_err(pe)?;
    root.draw(&Text::new(
        "DATA UNAVAILABLE",
        (780, 140),
        ts_pos(48, HPos::Center, VPos::Center),
    ))
    .map_err(pe)?;
    root.draw(&Text::new(
        "Fraction of Cs-137 released versus time at temperature: no measurement, no comparison",
        (780, 195),
        ts_pos(18, HPos::Center, VPos::Center),
    ))
    .map_err(pe)?;
    let mut ty = 255;
    for l in &text_lines {
        root.draw(&Text::new(l.clone(), (24, ty), ts(14)))
            .map_err(pe)?;
        ty += 19;
    }
    footer(&root, ty + 14, 215, &provenance)?;
    finish(root)
}
