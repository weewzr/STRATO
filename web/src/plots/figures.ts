// Figure builders (web phase 5). Each function turns a reply of the Rust wrapper, or a precomputed
// native file, into Plotly traces and a layout. Every axis, legend entry and hover text uses the full
// physical name of the quantity with its unit (coordinator decisions 30 and 52). The x and y arrays of every trace are the returned
// arrays themselves (the same objects, unchanged), or a selection of one returned element for the
// playback cursor; nothing is added, subtracted, scaled or interpolated here (owner decision M4).
// What is computed is display only and named as such: axis ranges (the logarithm of the largest nodal
// concentration of the run for a fixed six-decade log axis, as on the particle's colour scale) and
// positions of labels and shapes.
import { sciText } from '../format';
import { LOG_DECADES } from '../particle/colormap';
import { METHOD_1, METHOD_2 } from '../methods';
import type { CompareOk, RunOk } from '../solver/types';
import type { Figure } from './Plot';
import type { Layout, Trace } from './plotly';
import type { PlotTheme } from './theme';

export type { Figure };

function axis(theme: PlotTheme, title: string, extra: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    title: { text: title, font: { size: 12, color: theme.text }, standoff: 8 },
    color: theme.muted,
    gridcolor: theme.grid,
    linecolor: theme.axis,
    zerolinecolor: theme.axis,
    tickfont: { family: theme.mono, size: 11, color: theme.muted },
    ticks: 'outside',
    ticklen: 4,
    tickcolor: theme.axis,
    showline: true,
    mirror: false,
    automargin: true,
    ...extra,
  };
}

function baseLayout(theme: PlotTheme, extra: Layout): Layout {
  return {
    font: { family: theme.font, size: 12, color: theme.text },
    paper_bgcolor: 'rgba(0,0,0,0)',
    plot_bgcolor: 'rgba(0,0,0,0)',
    margin: { l: 56, r: 16, t: 8, b: 44, pad: 0 },
    showlegend: true,
    legend: { orientation: 'h', x: 0, xanchor: 'left', y: 1.0, yanchor: 'bottom', font: { size: 11, color: theme.text }, bgcolor: 'rgba(0,0,0,0)' },
    hoverlabel: { font: { family: theme.mono, size: 12 } },
    hovermode: 'closest',
    dragmode: 'zoom',
    ...extra,
  };
}

/** Fixed y range of a concentration axis: [0, top] (linear) or six decades up to top (log). Display only. */
function concentrationAxis(theme: PlotTheme, top: number, log: boolean): Record<string, unknown> {
  const title = 'Concentration (mol/m³)';
  if (log) {
    const upper = Math.log10(top);
    return axis(theme, title, { type: 'log', range: [upper - LOG_DECADES, upper + 0.1], autorange: false, exponentformat: 'power', dtick: 1 });
  }
  return axis(theme, title, { type: 'linear', range: [0, top * 1.04], autorange: false, exponentformat: 'power', rangemode: 'tozero' });
}

export interface LayerBand {
  name: string;
  inner_m: number;
  outer_m: number;
}

/** Layer bands from the radii the solver echoed (inputs.regions[].outer_radius_m): selection only. */
export function layerBands(run: RunOk): LayerBand[] {
  return run.inputs.regions.map((r, i) => ({ name: r.name, inner_m: i === 0 ? 0 : (run.inputs.regions[i - 1]?.outer_radius_m ?? 0), outer_m: r.outer_radius_m }));
}

function bandShapes(bands: LayerBand[], theme: PlotTheme): { shapes: unknown[]; annotations: unknown[] } {
  const shapes: unknown[] = [];
  const annotations: unknown[] = [];
  bands.forEach((b, i) => {
    const colour = theme.layers[i] ?? theme.axis;
    shapes.push({ type: 'rect', xref: 'x', yref: 'paper', x0: b.inner_m, x1: b.outer_m, y0: 0, y1: 1, fillcolor: colour, opacity: 0.1, line: { width: 0 }, layer: 'below' });
    shapes.push({ type: 'line', xref: 'x', yref: 'paper', x0: b.outer_m, x1: b.outer_m, y0: 0, y1: 1, line: { color: colour, width: 1.5 }, layer: 'below' });
    annotations.push({
      xref: 'x',
      yref: 'paper',
      x: b.inner_m,
      y: 1,
      xanchor: 'left',
      yanchor: 'top',
      xshift: 2,
      text: b.name,
      showarrow: false,
      font: { size: 11, color: colour },
      bgcolor: 'rgba(0,0,0,0)',
    });
  });
  return { shapes, annotations };
}

export interface ProfileOptions {
  log: boolean;
  showStart: boolean;
}

/** The largest nodal concentration over the run, as the wrapper returned it per row (selection only). */
function concentrationTop(run: RunOk): number {
  let top = 0;
  for (const v of run.max_concentration_mol_m3) if (v > top) top = v;
  return top;
}

/**
 * Radial concentration profile at the playback row (prompt/DESIGN_SPECIFICATION.md Section 4.3): the
 * returned nodal concentrations at the returned node radii, the layer boundaries as bands in the
 * layer colours, and optionally the start profile (row 0).
 */
export function profileFigure(run: RunOk, row: number, opts: ProfileOptions, theme: PlotTheme): Figure {
  const profile = run.profiles[row] ?? run.profiles[run.profiles.length - 1];
  const start = run.profiles[0];
  const x = run.grid.node_radii_m;
  const top = concentrationTop(run);
  const data: Trace[] = [];
  if (opts.showStart && start && profile !== start) {
    data.push({
      type: 'scatter',
      name: 'Start, time 0 s',
      x,
      y: start.c_mol_per_m3,
      mode: 'lines',
      line: { color: theme.seriesBase, width: 1.5 },
      hovertemplate: 'Radius (m) = %{x:.4~s}<br>Concentration (mol/m³) = %{y:.4~e}<extra>start</extra>',
      meta: { id: 'profile-start', label: 'start profile', unit: 'mol/m^3' },
    });
  }
  if (profile) {
    data.push({
      type: 'scatter',
      name: `Time ${sciText(profile.t_s, 4).replace(' × 10^', 'e')} s`,
      x,
      y: profile.c_mol_per_m3,
      mode: 'lines',
      line: { color: theme.seriesB, width: 2 },
      hovertemplate: `Radius (m) = %{x:.4~s}<br>Concentration (mol/m³) = %{y:.4~e}<extra>${METHOD_1}</extra>`,
      meta: { id: 'profile', label: METHOD_1, unit: 'mol/m^3' },
    });
  }
  const bands = layerBands(run);
  const outer = bands[bands.length - 1]?.outer_m ?? 0;
  const { shapes, annotations } = bandShapes(bands, theme);
  return {
    data,
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Radius (m)', { range: [0, outer], autorange: false, tickformat: '~s', hoverformat: '.4~s' }),
      yaxis: concentrationAxis(theme, top, opts.log),
      shapes,
      annotations,
      margin: { l: 60, r: 12, t: 22, b: 44, pad: 0 },
      showlegend: data.length > 1,
      legend: { orientation: 'h', x: 1, xanchor: 'right', y: 1.0, yanchor: 'bottom', font: { size: 11, color: theme.text } },
    }),
  };
}

export interface ReleaseOptions {
  log: boolean;
  /** Draw the curve of Method A (finite-difference BTCS). */
  showMethod1: boolean;
  /** Draw the curve of Method B (the analytical solution), where it applies. */
  showMethod2: boolean;
}

/**
 * Release fraction against time from Method A (finite-difference BTCS) and Method B (the analytical
 * solution) on the same axes (prompt/DESIGN_SPECIFICATION.md Section 4.4), with the validity start
 * and accuracy floor of Method B and a playback cursor at the output row shown in the particle.
 */
export function releaseFigure(run: RunOk, row: number, opts: ReleaseOptions, theme: PlotTheme): Figure {
  const a = run.analytical && run.analytical.applicable ? run.analytical : null;
  const data: Trace[] = [];
  if (opts.showMethod1) {
    data.push({
      type: 'scatter',
      name: METHOD_1,
      x: run.times_s,
      y: run.release_fraction,
      mode: 'lines',
      line: { color: theme.seriesB, width: 2 },
      hovertemplate: `Time (s) = %{x:.4~s}<br>Release fraction (dimensionless) = %{y:.6~g}<extra>${METHOD_1}</extra>`,
      meta: { id: 'release-b', label: METHOD_1, unit: '1' },
    });
  }
  if (a && opts.showMethod2) {
    data.push({
      type: 'scatter',
      name: METHOD_2,
      x: a.times_s,
      y: a.release_fraction,
      mode: 'lines',
      line: { color: theme.seriesC, width: 2 },
      hovertemplate: `Time (s) = %{x:.4~s}<br>Release fraction (dimensionless) = %{y:.6~g}<extra>${METHOD_2}</extra>`,
      meta: { id: 'release-c', label: METHOD_2, unit: '1' },
    });
  }
  const tRow = run.times_s[row];
  const fRow = run.release_fraction[row];
  if (tRow !== undefined && fRow !== undefined && opts.showMethod1) {
    data.push({
      type: 'scatter',
      name: 'Playback time',
      x: [tRow],
      y: [fRow],
      mode: 'markers',
      marker: { color: theme.surface, size: 9, symbol: 'circle', line: { color: theme.text, width: 2 } },
      showlegend: false,
      hovertemplate: `Playback time (s) = %{x:.4~s}<br>Release fraction (dimensionless) = %{y:.6~g}<extra>${METHOD_1}</extra>`,
      meta: { id: 'release-cursor', label: 'playback row', unit: '1' },
    });
  }
  const shapes: unknown[] = [];
  const annotations: unknown[] = [];
  if (a && opts.showMethod2) {
    shapes.push({ type: 'rect', xref: 'x', yref: 'paper', x0: 0, x1: a.valid_from_s, y0: 0, y1: 1, fillcolor: theme.muted, opacity: 0.12, line: { width: 0 }, layer: 'below' });
    annotations.push({
      xref: 'x',
      yref: 'paper',
      x: a.valid_from_s,
      y: 1,
      xanchor: 'left',
      yanchor: 'top',
      xshift: 3,
      text: `${METHOD_2} valid from time ${sciText(a.valid_from_s, 4).replace(' × 10^', 'e')} s`,
      showarrow: false,
      font: { size: 10, color: theme.muted },
    });
    if (opts.log) {
      shapes.push({ type: 'line', xref: 'paper', yref: 'y', x0: 0, x1: 1, y0: a.accuracy_floor, y1: a.accuracy_floor, line: { color: theme.muted, width: 1 } });
      annotations.push({
        xref: 'paper',
        yref: 'y',
        x: 1,
        y: Math.log10(a.accuracy_floor),
        xanchor: 'right',
        yanchor: 'bottom',
        text: `${METHOD_2} accuracy floor (dimensionless) ${sciText(a.accuracy_floor, 4).replace(' × 10^', 'e')}`,
        showarrow: false,
        font: { size: 10, color: theme.text },
      });
    }
  }
  if (tRow !== undefined) {
    shapes.push({ type: 'line', xref: 'x', yref: 'paper', x0: tRow, x1: tRow, y0: 0, y1: 1, line: { color: theme.text, width: 1 } });
  }
  return {
    data,
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Time (s)', { tickformat: '~s', hoverformat: '.4~s', rangemode: 'tozero' }),
      yaxis: opts.log
        ? axis(theme, 'Release fraction (dimensionless)', { type: 'log', exponentformat: 'power', dtick: 2 })
        : axis(theme, 'Release fraction (dimensionless)', { type: 'linear', rangemode: 'tozero' }),
      shapes,
      annotations,
    }),
  };
}

export interface CompareOptions {
  showBaseline: boolean;
  showDifference: boolean;
}

/**
 * Model comparison (prompt/DESIGN_SPECIFICATION.md Section 8.1): the baseline and the current release
 * curves of Method A, and optionally the difference curve that compare returns (current minus
 * baseline on the shared output times, computed in Rust) on a secondary axis.
 */
export function compareFigure(reply: CompareOk, opts: CompareOptions, theme: PlotTheme): Figure {
  const data: Trace[] = [];
  if (opts.showBaseline) {
    data.push({
      type: 'scatter',
      name: 'Baseline',
      x: reply.baseline.times_s,
      y: reply.baseline.release_fraction,
      mode: 'lines',
      line: { color: theme.seriesBase, width: 2 },
      hovertemplate: 'Time (s) = %{x:.4~s}<br>Release fraction (dimensionless) = %{y:.6~g}<extra>baseline</extra>',
      meta: { id: 'compare-baseline', label: 'baseline', unit: '1' },
    });
  }
  data.push({
    type: 'scatter',
    name: 'Current inputs',
    x: reply.modified.times_s,
    y: reply.modified.release_fraction,
    mode: 'lines',
    line: { color: theme.seriesB, width: 2 },
    hovertemplate: 'Time (s) = %{x:.4~s}<br>Release fraction (dimensionless) = %{y:.6~g}<extra>current</extra>',
    meta: { id: 'compare-modified', label: 'current inputs', unit: '1' },
  });
  if (opts.showDifference) {
    data.push({
      type: 'scatter',
      name: 'Difference, current minus baseline (right axis)',
      x: reply.difference.times_s,
      y: reply.difference.release_fraction,
      mode: 'lines',
      yaxis: 'y2',
      line: { color: theme.seriesDiff, width: 2 },
      hovertemplate: 'Time (s) = %{x:.4~s}<br>Difference of the release fraction (dimensionless) = %{y:.4~e}<extra>difference</extra>',
      meta: { id: 'compare-difference', label: 'difference (compare)', unit: '1' },
    });
  }
  return {
    data,
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Time (s)', { tickformat: '~s', hoverformat: '.4~s', rangemode: 'tozero' }),
      yaxis: axis(theme, 'Release fraction (dimensionless)', { rangemode: 'tozero' }),
      ...(opts.showDifference
        ? { yaxis2: axis(theme, 'Difference of the release fraction (dimensionless)', { overlaying: 'y', side: 'right', exponentformat: 'power', zeroline: true, showgrid: false }) }
        : {}),
      margin: { l: 56, r: opts.showDifference ? 64 : 16, t: 8, b: 44, pad: 0 },
    }),
  };
}

// ---------------------------------------------------------------------------------------------
// Verification figures from precomputed native output (PRECOMPUTED)
// ---------------------------------------------------------------------------------------------

export interface PrecomputedHeader {
  schema: number;
  label: 'PRECOMPUTED' | 'DISPLAY';
  kind: string;
  solver: string;
  generator: string;
  converter: string;
  strato_version: string;
  commit: string;
  input_file?: string;
  input_file_sha256?: string;
  command: string;
  rustc?: string;
  created_utc: string;
  note: string;
}

/** btcs_vs_analytical.json: the two release fractions at every output time, and the absolute difference. */
export interface ComparisonDoc extends PrecomputedHeader {
  data: {
    t_s: number[];
    release_fraction_btcs: number[];
    release_fraction_analytical: (number | null)[];
    abs_difference: (number | null)[];
    analytical: {
      name: string;
      modes: number;
      accuracy_floor: number;
      valid_from_s: number;
      max_abs_difference: number;
      final_abs_difference: number;
      resolved: boolean;
    };
    column_headers: Record<string, string>;
  };
}

/** btcs_independent_check.json: the independent check of the regression suite, with the observed orders. */
export interface CheckDoc extends PrecomputedHeader {
  data: {
    t_s: number[];
    f_btcs: number[];
    f_reference: number[];
    abs_difference: number[];
    time_error: number[];
    time_order: number[];
    time_uncertainty: number[];
    space_error: number[];
    space_order: number[];
    space_uncertainty: number[];
    cross_error: number[];
    f_limit: number[];
    limit_difference: number[];
    reference_floor: number[];
    tolerance: number[];
    limit_difference_over_tolerance: number[];
    passed: boolean[];
    column_headers: Record<string, string>;
    notes: string[];
  };
}

/** btcs_run.json: the run of the native command line (series and diagnostics as the solver wrote them). */
export interface RunDoc extends PrecomputedHeader {
  data: {
    run: {
      series: {
        t_s: number[];
        balance_residual_mol: number[];
        balance_residual_bound_mol: number[];
        release_fraction: number[];
      };
      diagnostics: { worst_balance_residual_relative: number; final_balance_residual_bound_relative: number };
    };
    summary: Record<string, string>;
  };
}

function floorLine(floor: number | null, theme: PlotTheme): { shapes: unknown[]; annotations: unknown[] } {
  const shapes: unknown[] = [];
  const annotations: unknown[] = [];
  if (floor !== null && Number.isFinite(floor) && floor > 0) {
    shapes.push({ type: 'line', xref: 'paper', yref: 'y', x0: 0, x1: 1, y0: floor, y1: floor, line: { color: theme.muted, width: 1 } });
    annotations.push({
      xref: 'paper',
      yref: 'y',
      x: 1,
      y: Math.log10(floor),
      xanchor: 'right',
      yanchor: 'bottom',
      text: `${METHOD_2} accuracy floor (dimensionless) ${sciText(floor, 4).replace(' × 10^', 'e')}`,
      showarrow: false,
      font: { size: 10, color: theme.text },
    });
  }
  return { shapes, annotations };
}

/**
 * The pointwise absolute difference between Method A and Method B at every output time, as the Rust
 * tool wrote it from the native reply (btcs_vs_analytical.csv). Shown only from this precomputed file:
 * the live page returns the largest and the final difference, not the pointwise one.
 */
export function methodDifferenceFigure(doc: ComparisonDoc, floor: number | null, theme: PlotTheme): Figure {
  const { shapes, annotations } = floorLine(floor, theme);
  return {
    data: [
      {
        type: 'scatter',
        name: `Absolute difference between ${METHOD_1} and ${METHOD_2}`,
        x: doc.data.t_s,
        y: doc.data.abs_difference,
        mode: 'lines+markers',
        line: { color: theme.seriesB, width: 1.5 },
        marker: { color: theme.seriesB, size: 4 },
        hovertemplate: 'Time (s) = %{x:.4~s}<br>Absolute difference of the release fraction (dimensionless) = %{y:.4~e}<extra></extra>',
        meta: { id: 'bc-difference', label: 'abs_difference of btcs_vs_analytical.csv', unit: '1' },
      },
    ],
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Time (s)', { tickformat: '~s', hoverformat: '.4~s', rangemode: 'tozero' }),
      yaxis: axis(theme, 'Absolute difference of the<br>release fraction (dimensionless)', { type: 'log', exponentformat: 'power', dtick: 1 }),
      shapes,
      annotations,
      showlegend: false,
    }),
  };
}

/** The release fractions of Method A and Method B of the verified case, as the native wrapper wrote them. */
export function methodReleaseFigure(doc: ComparisonDoc, theme: PlotTheme): Figure {
  return {
    data: [
      {
        type: 'scatter',
        name: METHOD_1,
        x: doc.data.t_s,
        y: doc.data.release_fraction_btcs,
        mode: 'lines',
        line: { color: theme.seriesB, width: 2 },
        hovertemplate: `Time (s) = %{x:.4~s}<br>Release fraction (dimensionless) = %{y:.6~g}<extra>${METHOD_1}</extra>`,
        meta: { id: 'bc-release-b', label: 'release_fraction_btcs', unit: '1' },
      },
      {
        type: 'scatter',
        name: METHOD_2,
        x: doc.data.t_s,
        y: doc.data.release_fraction_analytical,
        mode: 'lines',
        line: { color: theme.seriesC, width: 2 },
        hovertemplate: `Time (s) = %{x:.4~s}<br>Release fraction (dimensionless) = %{y:.6~g}<extra>${METHOD_2}</extra>`,
        meta: { id: 'bc-release-c', label: 'release_fraction_analytical', unit: '1' },
      },
    ],
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Time (s)', { tickformat: '~s', hoverformat: '.4~s', rangemode: 'tozero' }),
      yaxis: axis(theme, 'Release fraction (dimensionless)', { rangemode: 'tozero' }),
    }),
  };
}

/**
 * The independent check of the regression suite at its checkpoint times: the absolute difference of
 * the BTCS release fraction from a reference that contains no BTCS code, and the absolute difference
 * of that reference from the numerical limit estimated from refined runs of the solver itself. Both
 * traces are columns of the file as it is; the observed orders are in the table beside the figure.
 */
export function checkFigure(doc: CheckDoc, theme: PlotTheme): Figure {
  const d = doc.data;
  const trace = (name: string, y: number[], colour: string, id: string): Trace => ({
    type: 'scatter',
    name,
    x: d.t_s,
    y,
    mode: 'lines+markers',
    line: { color: colour, width: 2 },
    marker: { color: colour, size: 6 },
    hovertemplate: `Time (s) = %{x:.4~s}<br>${name} = %{y:.4~e}<extra></extra>`,
    meta: { id, label: name, unit: '1' },
  });
  return {
    data: [
      trace('Absolute difference of the BTCS release fraction from the independent reference (dimensionless)', d.abs_difference, theme.seriesB, 'check-difference'),
      trace('Absolute difference of the reference from the estimated numerical limit (dimensionless)', d.limit_difference, theme.seriesDiff, 'check-limit'),
    ],
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Time (s)', { type: 'log', tickformat: '~s', hoverformat: '.4~s', exponentformat: 'power' }),
      yaxis: axis(theme, 'Absolute difference of the<br>release fraction (dimensionless)', { type: 'log', exponentformat: 'power', dtick: 1 }),
      legend: { orientation: 'h', x: 0, xanchor: 'left', y: -0.35, yanchor: 'top', font: { size: 11, color: theme.text }, bgcolor: 'rgba(0,0,0,0)' },
      margin: { l: 56, r: 16, t: 8, b: 110, pad: 0 },
    }),
  };
}

/**
 * The balance residual of the native run at every output time with the derived rounding bound
 * (PRECOMPUTED): the residual is zero in exact arithmetic, so what remains is rounding.
 */
export function residualFigure(doc: RunDoc, theme: PlotTheme): Figure {
  const s = doc.data.run.series;
  return {
    data: [
      {
        type: 'scatter',
        name: 'Mass-balance residual (mol)',
        x: s.t_s,
        y: s.balance_residual_mol,
        mode: 'lines+markers',
        line: { color: theme.seriesB, width: 1.5 },
        marker: { color: theme.seriesB, size: 4 },
        hovertemplate: 'Time (s) = %{x:.4~s}<br>Mass-balance residual (mol) = %{y:.3~e}<extra></extra>',
        meta: { id: 'balance-residual', label: 'balance_residual_mol', unit: 'mol' },
      },
      {
        type: 'scatter',
        name: 'Derived rounding bound of the residual (mol)',
        x: s.t_s,
        y: s.balance_residual_bound_mol,
        mode: 'lines',
        line: { color: theme.muted, width: 1.5 },
        hovertemplate: 'Time (s) = %{x:.4~s}<br>Derived rounding bound of the residual (mol) = %{y:.3~e}<extra></extra>',
        meta: { id: 'balance-bound', label: 'balance_residual_bound_mol', unit: 'mol' },
      },
    ],
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Time (s)', { tickformat: '~s', hoverformat: '.4~s', rangemode: 'tozero' }),
      yaxis: axis(theme, 'Mass-balance residual and its bound (mol)', { exponentformat: 'power', zeroline: true }),
      showlegend: true,
    }),
  };
}