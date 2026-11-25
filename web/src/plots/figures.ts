// Figure builders (web phase 5). Each function turns a reply of the Rust wrapper, or a precomputed
// native file, into Plotly traces and a layout. The x and y arrays of every trace are the returned
// arrays themselves (the same objects, unchanged), or a selection of one returned element for the
// playback cursor; nothing is added, subtracted, scaled or interpolated here (owner decision M4).
// What is computed is display only and named as such: axis ranges (the logarithm of the kernel start
// concentration c0 for a fixed six-decade log axis, as on the particle's colour scale) and positions
// of labels and shapes.
import { sciText } from '../format';
import { LOG_DECADES } from '../particle/colormap';
import type { CompareOk, RunOk } from '../solver/types';
import type { Figure } from './Plot';
import type { Layout, Trace } from './plotly';
import type { PlotTheme } from './theme';

export type { Figure };

const SUP_MINUS = '⁻';

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

/** Fixed y range of a concentration axis: [0, c0] (linear) or six decades up to c0 (log). Display only. */
function concentrationAxis(theme: PlotTheme, c0: number, log: boolean): Record<string, unknown> {
  const title = `Concentration c (mol m${SUP_MINUS}³)`;
  if (log) {
    const top = Math.log10(c0);
    return axis(theme, title, { type: 'log', range: [top - LOG_DECADES, top + 0.1], autorange: false, exponentformat: 'power', dtick: 1 });
  }
  return axis(theme, title, { type: 'linear', range: [0, c0 * 1.04], autorange: false, exponentformat: 'power', rangemode: 'tozero' });
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

/**
 * Radial concentration profile c(r) at the playback row (prompt/DESIGN_SPECIFICATION.md Section 4.3):
 * the returned cell concentrations at the returned cell centroids, the layer boundaries as bands in
 * the layer colours, and optionally the start profile (row 0).
 */
export function profileFigure(run: RunOk, row: number, opts: ProfileOptions, theme: PlotTheme): Figure {
  const profile = run.profiles[row] ?? run.profiles[run.profiles.length - 1];
  const start = run.profiles[0];
  const x = run.mesh.cell_centroids_m;
  const c0 = run.inputs.initial.kernel_concentration_mol_m3;
  const data: Trace[] = [];
  if (opts.showStart && start && profile !== start) {
    data.push({
      type: 'scatter',
      name: 'Start, t = 0 s',
      x,
      y: start.c_mol_per_m3,
      mode: 'lines',
      line: { color: theme.seriesBase, width: 1.5, dash: 'dot' },
      hovertemplate: 'r = %{x:.4~s}m<br>c = %{y:.4~e} mol m⁻³<extra>start</extra>',
      meta: { id: 'profile-start', label: 'start profile', unit: 'mol m^-3' },
    });
  }
  if (profile) {
    data.push({
      type: 'scatter',
      name: `t = ${sciText(profile.t_s, 4).replace(' × 10^', 'e')} s`,
      x,
      y: profile.c_mol_per_m3,
      mode: 'lines',
      line: { color: theme.seriesB, width: 2 },
      hovertemplate: 'r = %{x:.4~s}m<br>c = %{y:.4~e} mol m⁻³<extra>Code B</extra>',
      meta: { id: 'profile', label: 'finite volume (Code B)', unit: 'mol m^-3' },
    });
  }
  const bands = layerBands(run);
  const outer = bands[bands.length - 1]?.outer_m ?? 0;
  const { shapes, annotations } = bandShapes(bands, theme);
  return {
    data,
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Radius r (m)', { range: [0, outer], autorange: false, tickformat: '~s', hoverformat: '.4~s' }),
      yaxis: concentrationAxis(theme, c0, opts.log),
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
}

/**
 * Release fraction F(t) from Code B (finite volume) and Code C (eigenfunction series) on the same axes
 * (prompt/DESIGN_SPECIFICATION.md Section 4.4), with the series' validity start and accuracy floor and
 * a playback cursor at the output row shown in the particle.
 */
export function releaseFigure(run: RunOk, row: number, opts: ReleaseOptions, theme: PlotTheme): Figure {
  const a = run.analytical;
  const data: Trace[] = [
    {
      type: 'scatter',
      name: 'Finite volume (Code B)',
      x: run.times_s,
      y: run.release_fraction,
      mode: 'lines',
      line: { color: theme.seriesB, width: 2 },
      hovertemplate: 't = %{x:.4~s}s<br>F = %{y:.6~g}<extra>Code B</extra>',
      meta: { id: 'release-b', label: 'finite volume (Code B)', unit: '1' },
    },
  ];
  if (a) {
    data.push({
      type: 'scatter',
      name: 'Eigenfunction series (Code C)',
      x: a.times_s,
      y: a.release_fraction,
      mode: 'lines',
      line: { color: theme.seriesC, width: 2, dash: 'dash' },
      hovertemplate: 't = %{x:.4~s}s<br>F = %{y:.6~g}<extra>Code C</extra>',
      meta: { id: 'release-c', label: 'eigenfunction series (Code C)', unit: '1' },
    });
  }
  const tRow = run.times_s[row];
  const fRow = run.release_fraction[row];
  if (tRow !== undefined && fRow !== undefined) {
    data.push({
      type: 'scatter',
      name: 'Playback time',
      x: [tRow],
      y: [fRow],
      mode: 'markers',
      marker: { color: theme.surface, size: 9, symbol: 'circle', line: { color: theme.text, width: 2 } },
      showlegend: false,
      hovertemplate: 'playback: t = %{x:.4~s}s<br>F = %{y:.6~g}<extra>Code B</extra>',
      meta: { id: 'release-cursor', label: 'playback row', unit: '1' },
    });
  }
  const shapes: unknown[] = [];
  const annotations: unknown[] = [];
  if (a) {
    shapes.push({ type: 'rect', xref: 'x', yref: 'paper', x0: 0, x1: a.valid_from_s, y0: 0, y1: 1, fillcolor: theme.muted, opacity: 0.12, line: { width: 0 }, layer: 'below' });
    annotations.push({
      xref: 'x',
      yref: 'paper',
      x: a.valid_from_s,
      y: 1,
      xanchor: 'left',
      yanchor: 'top',
      xshift: 3,
      text: `series valid from ${sciText(a.valid_from_s, 4).replace(' × 10^', 'e')} s`,
      showarrow: false,
      font: { size: 10, color: theme.muted },
    });
    if (opts.log) {
      shapes.push({ type: 'line', xref: 'paper', yref: 'y', x0: 0, x1: 1, y0: a.accuracy_floor, y1: a.accuracy_floor, line: { color: theme.seriesC, width: 1, dash: 'dot' } });
      annotations.push({
        xref: 'paper',
        yref: 'y',
        x: 1,
        y: Math.log10(a.accuracy_floor),
        xanchor: 'right',
        yanchor: 'bottom',
        text: `series accuracy floor ${sciText(a.accuracy_floor, 4).replace(' × 10^', 'e')}`,
        showarrow: false,
        font: { size: 10, color: theme.seriesC },
      });
    }
  }
  if (tRow !== undefined) {
    shapes.push({ type: 'line', xref: 'x', yref: 'paper', x0: tRow, x1: tRow, y0: 0, y1: 1, line: { color: theme.text, width: 1, dash: 'dot' } });
  }
  return {
    data,
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Time t (s)', { tickformat: '~s', hoverformat: '.4~s', rangemode: 'tozero' }),
      yaxis: opts.log
        ? axis(theme, 'Release fraction F (of the start inventory)', { type: 'log', exponentformat: 'power', dtick: 2 })
        : axis(theme, 'Release fraction F (of the start inventory)', { type: 'linear', rangemode: 'tozero' }),
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
 * curves, and optionally the difference curve that compare_v1 returns (current minus baseline on the
 * shared output times, computed in Rust) on a secondary axis.
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
      line: { color: theme.seriesBase, width: 2, dash: 'dash' },
      hovertemplate: 't = %{x:.4~s}s<br>F = %{y:.6~g}<extra>baseline</extra>',
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
    hovertemplate: 't = %{x:.4~s}s<br>F = %{y:.6~g}<extra>current</extra>',
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
      line: { color: theme.seriesDiff, width: 2, dash: 'dot' },
      hovertemplate: 't = %{x:.4~s}s<br>ΔF = %{y:.4~e}<extra>difference</extra>',
      meta: { id: 'compare-difference', label: 'difference (compare_v1)', unit: '1' },
    });
  }
  return {
    data,
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Time t (s)', { tickformat: '~s', hoverformat: '.4~s', rangemode: 'tozero' }),
      yaxis: axis(theme, 'Release fraction F', { rangemode: 'tozero' }),
      ...(opts.showDifference
        ? { yaxis2: axis(theme, 'Difference ΔF', { overlaying: 'y', side: 'right', exponentformat: 'power', zeroline: true, showgrid: false }) }
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
  generator: string;
  strato_version: string;
  commit: string;
  input_file?: string;
  input_sha256?: string;
  command: string;
  rustc?: string;
  created_utc: string;
  note: string;
}

export interface ComparisonDoc extends PrecomputedHeader {
  data: { time_s: number[]; release_fv: number[]; release_analytical: (number | null)[]; abs_error: (number | null)[] };
}

export interface ConvergenceDoc extends PrecomputedHeader {
  data: {
    points: { study: string[]; total_cells: number[]; dt_s: number[]; fv_release: number[]; abs_error_vs_exact: number[] };
    exact_final_release_fraction: string;
    spatial_observed_orders: string;
    temporal_observed_orders: string;
  };
}

export interface GuidesDoc extends PrecomputedHeader {
  data: Record<'spatial' | 'temporal', { x_name: string; x: number[]; slope: number; guide_abs_error: number[]; anchor_abs_error: number }>;
}

export interface SyntheticRunDoc extends PrecomputedHeader {
  data: {
    timeseries: { time_s: number[]; balance_residual_mol: number[]; release_fraction: number[] };
    summary: Record<string, string>;
  };
}

/**
 * The pointwise difference |F_B − F_C| of the verified synthetic case, as the native binary wrote it
 * (comparison_fv_vs_analytical.csv, column abs_error). Shown only from this precomputed file: the live
 * page does not return it (coordinator decision of 2026-11-25; prompt/PROGRESS.md, "Phase 3 record").
 */
export function codeBCDifferenceFigure(doc: ComparisonDoc, floor: number | null, theme: PlotTheme): Figure {
  const shapes: unknown[] = [];
  const annotations: unknown[] = [];
  if (floor !== null && Number.isFinite(floor) && floor > 0) {
    shapes.push({ type: 'line', xref: 'paper', yref: 'y', x0: 0, x1: 1, y0: floor, y1: floor, line: { color: theme.seriesC, width: 1, dash: 'dot' } });
    annotations.push({
      xref: 'paper',
      yref: 'y',
      x: 1,
      y: Math.log10(floor),
      xanchor: 'right',
      yanchor: 'bottom',
      text: `series accuracy floor ${sciText(floor, 4).replace(' × 10^', 'e')}`,
      showarrow: false,
      font: { size: 10, color: theme.seriesC },
    });
  }
  return {
    data: [
      {
        type: 'scatter',
        name: '|F_B − F_C|, native binary',
        x: doc.data.time_s,
        y: doc.data.abs_error,
        mode: 'lines+markers',
        line: { color: theme.seriesB, width: 1.5 },
        marker: { color: theme.seriesB, size: 4 },
        hovertemplate: 't = %{x:.4~s}s<br>|F_B − F_C| = %{y:.4~e}<extra></extra>',
        meta: { id: 'bc-difference', label: 'abs_error of comparison_fv_vs_analytical.csv', unit: '1' },
      },
    ],
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Time t (s)', { tickformat: '~s', hoverformat: '.4~s', rangemode: 'tozero' }),
      yaxis: axis(theme, 'Difference |F_B − F_C|', { type: 'log', exponentformat: 'power', dtick: 1 }),
      shapes,
      annotations,
      showlegend: false,
    }),
  };
}

/** Code B and Code C release fractions of the verified case, as the native binary wrote them. */
export function codeBCReleaseFigure(doc: ComparisonDoc, theme: PlotTheme): Figure {
  return {
    data: [
      {
        type: 'scatter',
        name: 'Finite volume (Code B)',
        x: doc.data.time_s,
        y: doc.data.release_fv,
        mode: 'lines',
        line: { color: theme.seriesB, width: 2 },
        hovertemplate: 't = %{x:.4~s}s<br>F = %{y:.6~g}<extra>Code B</extra>',
        meta: { id: 'bc-release-b', label: 'release_fv', unit: '1' },
      },
      {
        type: 'scatter',
        name: 'Eigenfunction series (Code C)',
        x: doc.data.time_s,
        y: doc.data.release_analytical,
        mode: 'lines',
        line: { color: theme.seriesC, width: 2, dash: 'dash' },
        hovertemplate: 't = %{x:.4~s}s<br>F = %{y:.6~g}<extra>Code C</extra>',
        meta: { id: 'bc-release-c', label: 'release_analytical', unit: '1' },
      },
    ],
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Time t (s)', { tickformat: '~s', hoverformat: '.4~s', rangemode: 'tozero' }),
      yaxis: axis(theme, 'Release fraction F', { rangemode: 'tozero' }),
    }),
  };
}

/** Indices of one refinement study in the convergence points, in file order (selection only). */
export function studyIndices(doc: ConvergenceDoc, study: 'spatial' | 'temporal'): number[] {
  return doc.data.points.study.flatMap((s, i) => (s === study ? [i] : []));
}

/**
 * Convergence of one refinement study on log-log axes (prompt/DESIGN_SPECIFICATION.md Section 8.3):
 * the errors against the exact series that the native binary printed, and the guide line of the
 * reference slope (written by scripts/web/generate_data.py figures, label DISPLAY).
 */
export function convergenceFigure(doc: ConvergenceDoc, guides: GuidesDoc | null, study: 'spatial' | 'temporal', theme: PlotTheme): Figure {
  const idx = studyIndices(doc, study);
  const p = doc.data.points;
  const x = idx.map((i) => (study === 'spatial' ? p.total_cells[i] : p.dt_s[i]) ?? NaN);
  const y = idx.map((i) => p.abs_error_vs_exact[i] ?? NaN);
  const g = guides?.data[study];
  const data: Trace[] = [
    {
      type: 'scatter',
      name:
        study === 'spatial'
          ? `Error at t_end, cells refined (Δt = ${String(p.dt_s[idx[0] ?? 0])} s)`
          : `Error at t_end, step refined (${String(p.total_cells[idx[0] ?? 0])} cells)`,
      x,
      y,
      mode: 'lines+markers',
      line: { color: theme.seriesB, width: 2 },
      marker: { color: theme.seriesB, size: 7 },
      hovertemplate: study === 'spatial' ? 'cells = %{x}<br>error = %{y:.4~e}<extra></extra>' : 'Δt = %{x} s<br>error = %{y:.4~e}<extra></extra>',
      meta: { id: `convergence-${study}`, label: 'abs_error_vs_exact', unit: '1' },
    },
  ];
  if (g) {
    data.push({
      type: 'scatter',
      name: `Guide line, slope ${g.slope} (not a result)`,
      x: g.x,
      y: g.guide_abs_error,
      mode: 'lines',
      line: { color: theme.muted, width: 1.5, dash: 'dash' },
      hoverinfo: 'skip',
      meta: { id: `guide-${study}`, label: 'guide line', unit: '1' },
    });
  }
  return {
    data,
    layout: baseLayout(theme, {
      xaxis: axis(theme, study === 'spatial' ? 'Total cells N' : 'Time step Δt (s)', { type: 'log', exponentformat: 'power', dtick: 'D2' }),
      yaxis: axis(theme, 'Error |F − F_exact| at t_end', { type: 'log', exponentformat: 'power', dtick: 'D2' }),
    }),
  };
}

/** The balance residual of the native run at every output time (PRECOMPUTED). */
export function residualFigure(doc: SyntheticRunDoc, theme: PlotTheme): Figure {
  return {
    data: [
      {
        type: 'scatter',
        name: 'Residual M + M_rel − M⁰',
        x: doc.data.timeseries.time_s,
        y: doc.data.timeseries.balance_residual_mol,
        mode: 'lines+markers',
        line: { color: theme.seriesB, width: 1.5 },
        marker: { color: theme.seriesB, size: 4 },
        hovertemplate: 't = %{x:.4~s}s<br>residual = %{y:.3~e} mol<extra></extra>',
        meta: { id: 'balance-residual', label: 'balance_residual_mol', unit: 'mol' },
      },
    ],
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Time t (s)', { tickformat: '~s', hoverformat: '.4~s', rangemode: 'tozero' }),
      yaxis: axis(theme, 'Balance residual (mol)', { exponentformat: 'power', zeroline: true }),
      showlegend: false,
    }),
  };
}
