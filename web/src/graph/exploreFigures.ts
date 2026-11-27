// The Plotly figures of the INTERACTIVE GRAPHS of Explore. Every x and y array of a trace is a field of the
// wrapper's reply (or of a comparison file), the same array object, unchanged: nothing is added,
// scaled, interpolated or connected here (a value that is null stays a gap). Method A is blue and solid,
// Method B red and solid, the experimental markers black, upper bounds downward triangles.
import { METHOD_1, METHOD_2 } from '../methods';
import { axis, baseLayout } from '../plots/figures';
import type { Figure } from '../plots/Plot';
import type { Trace } from '../plots/plotly';
import type { PlotTheme } from '../plots/theme';
import type { AnalyticalReply, StageOk } from '../solver/types';
import type { ComparisonRecord } from './exploreData';
import { COLOURS } from './plotModel';

const LINE_WIDTH = 2;
const TIME_TITLE = 'Time (h)';
const RELEASE_TITLE = 'Cumulative release fraction (dimensionless)';

const hoverRelease = (unit: string, name: string): string => `Time (${unit}) = %{x:.6~g}<br>Cumulative release fraction (dimensionless) = %{y:.6~g}<extra>${name}</extra>`;

function methodA(t: readonly number[], f: readonly number[], id: string): Trace {
  return {
    type: 'scatter',
    name: METHOD_1,
    x: t,
    y: f,
    mode: 'lines',
    line: { color: COLOURS.A, width: LINE_WIDTH, dash: 'solid' },
    hovertemplate: hoverRelease('h', METHOD_1),
    meta: { id, label: METHOD_1, unit: '1' },
  };
}

function methodB(t: readonly number[], f: readonly (number | null)[], id: string): Trace {
  return {
    type: 'scatter',
    name: METHOD_2,
    x: t,
    y: f,
    mode: 'lines',
    line: { color: COLOURS.B, width: LINE_WIDTH, dash: 'solid' },
    hovertemplate: hoverRelease('h', METHOD_2),
    meta: { id, label: METHOD_2, unit: '1' },
  };
}

const releaseLayout = (theme: PlotTheme, timeUnit: string, extra: Record<string, unknown> = {}) =>
  baseLayout(theme, {
    xaxis: axis(theme, timeUnit === 'h' ? TIME_TITLE : `Time (${timeUnit})`, { rangemode: 'tozero', hoverformat: '.6~g' }),
    yaxis: axis(theme, RELEASE_TITLE, { rangemode: 'tozero' }),
    ...extra,
  });

/** Method B of the reply of analytical as a trace, or null where the wrapper gave no drawable value. */
export function methodBTrace(a: AnalyticalReply | null): Trace | null {
  if (a === null || !a.applicable || a.times_h === null || a.release_fraction === null) return null;
  if (!a.release_fraction.some((v) => v !== null)) return null;
  return methodB(a.times_h, a.release_fraction, 'stage-method-b');
}

/** Cumulative release fraction against time: Method A of the stage reply. */
export function releaseFigure(stage: StageOk, theme: PlotTheme): Figure {
  return { data: [methodA(stage.series.times_h, stage.series.release_fraction, 'stage-method-a')], layout: releaseLayout(theme, 'h', { showlegend: false }) };
}

/** Method A against Method B: both curves where Method B answered; null otherwise (no figure, the panel stays empty). */
export function methodsFigure(stage: StageOk, analytical: AnalyticalReply | null, theme: PlotTheme): Figure | null {
  const b = methodBTrace(analytical);
  if (b === null) return null;
  return { data: [methodA(stage.series.times_h, stage.series.release_fraction, 'stage-method-a'), b], layout: releaseLayout(theme, 'h') };
}

/** Radial concentration profile of Method A at the times the reply gives, node by node. */
export function profileFigure(stage: StageOk, theme: PlotTheme): Figure | null {
  if (stage.profiles.length === 0) return null;
  const last = stage.profiles.length - 1;
  const data: Trace[] = stage.profiles.map((p, i) => ({
    type: 'scatter',
    name: `${String(p.t_h)} h`,
    x: stage.grid.node_radii_m,
    y: p.c_mol_per_m3,
    mode: 'lines',
    // The profiles of earlier times are the same blue, lighter in width only: one method, one colour.
    line: { color: COLOURS.A, width: i === last ? LINE_WIDTH : 1, dash: 'solid' },
    hovertemplate: `Radius (m) = %{x:.6~g}<br>Concentration (mol/m³) = %{y:.6~g}<extra>${String(p.t_h)} h</extra>`,
    meta: { id: `stage-profile-${String(i)}`, label: `${String(p.t_h)} h`, unit: 'mol/m3' },
  }));
  const shapes = stage.grid.interface_radii_m.map((r) => ({
    type: 'line',
    xref: 'x',
    yref: 'paper',
    x0: r,
    x1: r,
    y0: 0,
    y1: 1,
    line: { color: theme.axis, width: 1, dash: 'dot' },
  }));
  return {
    data,
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Radius (m)', { rangemode: 'tozero', exponentformat: 'power' }),
      yaxis: axis(theme, 'Concentration (mol/m³)', { rangemode: 'tozero', exponentformat: 'power' }),
      shapes,
    }),
  };
}

/** True where a point of a comparison file is an upper bound (kept apart from the measurements). */
export const isBound = (pointClass: string): boolean => /bound/i.test(pointClass);

/** The experimental comparison of a registered curve: markers, the model curve(s) of the file, the R² label only where it holds a value. */
export function experimentFigure(file: ComparisonRecord, theme: PlotTheme): Figure {
  const measured = file.experiment.points.filter((p) => !isBound(p.point_class));
  const bounds = file.experiment.points.filter((p) => isBound(p.point_class));
  const data: Trace[] = [methodA(file.model.method_a.time_h, file.model.method_a.release_fraction, 'exp-method-a')];
  const b = file.model.method_b;
  if (b.status === 'available' && b.time_h && b.release_fraction && b.release_fraction.some((v) => v !== null)) data.push(methodB(b.time_h, b.release_fraction, 'exp-method-b'));
  const markers = (list: typeof measured, name: string, id: string, symbol: string): Trace => ({
    type: 'scatter',
    name,
    x: list.map((p) => p.time_h),
    y: list.map((p) => p.value),
    mode: 'markers',
    marker: { color: COLOURS.observations, size: 8, symbol, line: { color: '#ffffff', width: 1 } },
    hovertemplate: hoverRelease('h', name),
    meta: { id, label: name, unit: '1' },
  });
  if (measured.length > 0) data.push(markers(measured, 'Experimental measurements', 'exp-measurements', 'circle'));
  if (bounds.length > 0) data.push(markers(bounds, 'Experimental upper bounds', 'exp-upper-bounds', 'triangle-down'));
  const labels = file.r2_records.filter((r) => r.status === 'available' && r.label_plain);
  const annotations = labels.map((r, i) => ({
    xref: 'paper',
    yref: 'paper',
    x: 0.02,
    y: 0.98,
    yshift: -18 * i,
    xanchor: 'left',
    yanchor: 'top',
    text: r.label_plain ?? '',
    showarrow: false,
    align: 'left',
    font: { size: 13, color: r.method === 'A' ? COLOURS.A : COLOURS.B },
  }));
  return { data, layout: releaseLayout(theme, 'h', { annotations, margin: { l: 64, r: 16, t: 40, b: 48, pad: 0 } }) };
}
