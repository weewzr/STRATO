//! SVG figures written with `plotters` (no system fonts or image libraries needed).

use super::comparison::{ConvergencePoint, ConvergenceStudy, ReleaseComparison};
use anyhow::{anyhow, Result};
use plotters::prelude::*;
use std::path::Path;

fn pe<E: std::fmt::Display>(e: E) -> anyhow::Error {
    anyhow!("plot error: {e}")
}

fn sci(v: &f64) -> String {
    format!("{v:.0e}")
}

fn positive_range(values: impl Iterator<Item = f64>) -> (f64, f64) {
    let (mut lo, mut hi) = (f64::INFINITY, 0.0f64);
    for v in values.filter(|v| *v > 0.0 && v.is_finite()) {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    if !lo.is_finite() || hi <= lo {
        let base = if lo.is_finite() { lo } else { 1.0 };
        return (base * 0.5, base * 2.0);
    }
    (lo / 1.5, hi * 1.5)
}

/// Release fraction F(t): FV against the exact series, with the signed error below.
pub fn release_comparison(path: &Path, case: &str, cmp: &ReleaseComparison) -> Result<()> {
    let root = SVGBackend::new(path, (900, 760)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let (upper, lower) = root.split_vertically(470);

    let (t_lo, t_hi) = positive_range(cmp.rows.iter().map(|r| r.time_s));
    let mut chart = ChartBuilder::on(&upper)
        .caption(
            format!("Release fraction, FV vs exact series ({case})"),
            ("sans-serif", 20),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d((t_lo..t_hi).log_scale(), 0.0f64..1.05f64)
        .map_err(pe)?;
    chart
        .configure_mesh()
        .x_desc("time (s)")
        .y_desc("release fraction F")
        .x_labels(6)
        .x_label_formatter(&sci)
        .light_line_style(TRANSPARENT)
        .draw()
        .map_err(pe)?;
    chart
        .draw_series(LineSeries::new(
            cmp.rows.iter().map(|r| (r.time_s, r.fv)),
            BLUE.stroke_width(2),
        ))
        .map_err(pe)?
        .label("FV, Backward Euler")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.stroke_width(2)));
    chart
        .draw_series(cmp.rows.iter().filter_map(|r| {
            r.exact
                .map(|e| Circle::new((r.time_s, e), 3, RED.stroke_width(1)))
        }))
        .map_err(pe)?
        .label("exact eigenfunction series")
        .legend(|(x, y)| Circle::new((x + 10, y), 3, RED.stroke_width(1)));
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .background_style(WHITE.mix(0.85))
        .border_style(BLACK)
        .draw()
        .map_err(pe)?;

    let errs: Vec<(f64, f64)> = cmp
        .rows
        .iter()
        .filter_map(|r| r.exact.map(|e| (r.time_s, r.fv - e)))
        .collect();
    let bound = errs
        .iter()
        .map(|(_, e)| e.abs())
        .fold(0.0f64, f64::max)
        .max(1e-12)
        * 1.2;
    let mut err_chart = ChartBuilder::on(&lower)
        .caption("Signed error FV - exact", ("sans-serif", 16))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d((t_lo..t_hi).log_scale(), -bound..bound)
        .map_err(pe)?;
    err_chart
        .configure_mesh()
        .x_desc("time (s)")
        .y_desc("F_FV - F_exact")
        .x_labels(6)
        .x_label_formatter(&sci)
        .y_label_formatter(&|v| format!("{v:.1e}"))
        .light_line_style(TRANSPARENT)
        .draw()
        .map_err(pe)?;
    err_chart
        .draw_series(LineSeries::new(errs, RED.stroke_width(2)))
        .map_err(pe)?;
    root.present().map_err(pe)?;
    Ok(())
}

fn loglog_panel<DB: DrawingBackend>(
    area: &DrawingArea<DB, plotters::coord::Shift>,
    title: &str,
    x_desc: &str,
    points: &[(f64, f64)],
    slope: f64,
    floor: f64,
) -> Result<()> {
    let (x_lo, x_hi) = positive_range(points.iter().map(|p| p.0));
    let ys: Vec<f64> = points.iter().map(|p| p.1.max(1e-16)).collect();
    let (y_lo, y_hi) = positive_range(ys.iter().copied().chain(std::iter::once(floor)));
    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", 16))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d((x_lo..x_hi).log_scale(), (y_lo..y_hi).log_scale())
        .map_err(pe)?;
    chart
        .configure_mesh()
        .x_desc(x_desc)
        .y_desc("|F_FV - F_exact| at end time")
        .x_labels(6)
        .y_labels(6)
        .x_label_formatter(&sci)
        .y_label_formatter(&sci)
        .light_line_style(TRANSPARENT)
        .draw()
        .map_err(pe)?;
    chart
        .draw_series(LineSeries::new(
            points.iter().zip(&ys).map(|(p, y)| (p.0, *y)),
            BLUE.stroke_width(2),
        ))
        .map_err(pe)?
        .label("FV error")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.stroke_width(2)));
    chart
        .draw_series(
            points
                .iter()
                .zip(&ys)
                .map(|(p, y)| Circle::new((p.0, *y), 4, BLUE.filled())),
        )
        .map_err(pe)?;
    // Reference slope anchored at the first point.
    let (x0, y0) = (points[0].0, ys[0]);
    let x1 = points[points.len() - 1].0;
    let guide = vec![(x0, y0), (x1, y0 * (x1 / x0).powf(slope))];
    chart
        .draw_series(LineSeries::new(
            guide.into_iter().filter(|(_, y)| *y >= y_lo && *y <= y_hi),
            BLACK.stroke_width(1),
        ))
        .map_err(pe)?
        .label(format!("slope {slope:+.0} guide"))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK));
    chart
        .draw_series(LineSeries::new(
            vec![(x_lo, floor), (x_hi, floor)],
            RED.mix(0.7).stroke_width(1),
        ))
        .map_err(pe)?
        .label("analytical accuracy floor")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.mix(0.7)));
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerLeft)
        .background_style(WHITE.mix(0.85))
        .border_style(BLACK)
        .draw()
        .map_err(pe)?;
    Ok(())
}

/// Spatial (left) and temporal (right) error against the exact series.
pub fn convergence(path: &Path, case: &str, study: &ConvergenceStudy) -> Result<()> {
    let root = SVGBackend::new(path, (1100, 480)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let (left, right) = root.split_horizontally(550);
    let pts = |v: &[ConvergencePoint], x: fn(&ConvergencePoint) -> f64| -> Vec<(f64, f64)> {
        v.iter().map(|p| (x(p), p.abs_error)).collect()
    };
    loglog_panel(
        &left,
        &format!("Spatial refinement ({case})"),
        "total FV cells",
        &pts(&study.spatial, |p| p.cells as f64),
        -2.0,
        study.accuracy_floor,
    )?;
    loglog_panel(
        &right,
        &format!("Timestep refinement ({case})"),
        "timestep dt (s)",
        &pts(&study.temporal, |p| p.dt_s),
        1.0,
        study.accuracy_floor,
    )?;
    root.present().map_err(pe)?;
    Ok(())
}

/// Final concentration profile with the four material interfaces marked.
pub fn profile(
    path: &Path,
    case: &str,
    radius: &[f64],
    conc: &[f64],
    layers: &[(String, f64)],
) -> Result<()> {
    let root = SVGBackend::new(path, (900, 480)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let r_max = radius.last().copied().unwrap_or(1.0) * 1.02;
    let c_max = conc.iter().copied().fold(0.0f64, f64::max).max(1e-300) * 1.1;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Final concentration profile ({case})"),
            ("sans-serif", 20),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d(0.0f64..r_max, 0.0f64..c_max)
        .map_err(pe)?;
    chart
        .configure_mesh()
        .x_desc("radius (m)")
        .y_desc("concentration (mol/m^3)")
        .x_label_formatter(&|v| format!("{v:.1e}"))
        .y_label_formatter(&|v| format!("{v:.2e}"))
        .light_line_style(TRANSPARENT)
        .draw()
        .map_err(pe)?;
    chart
        .draw_series(LineSeries::new(
            radius.iter().copied().zip(conc.iter().copied()),
            BLUE.stroke_width(2),
        ))
        .map_err(pe)?;
    let mut inner = 0.0;
    for (name, outer) in layers {
        chart
            .draw_series(LineSeries::new(
                vec![(*outer, 0.0), (*outer, c_max)],
                RED.mix(0.6),
            ))
            .map_err(pe)?;
        chart
            .draw_series(std::iter::once(Text::new(
                name.clone(),
                (0.5 * (inner + outer), c_max * 0.97),
                ("sans-serif", 13).into_font().color(&RED),
            )))
            .map_err(pe)?;
        inner = *outer;
    }
    root.present().map_err(pe)?;
    Ok(())
}
