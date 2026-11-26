// The Plotly figure of an experimental-comparison graph, from the plot model. Display only: the x and
// y arrays of every trace are the model's (the file's) arrays, unchanged. Both methods are solid lines
// of equal width; the experimental observations are black markers, upper bounds downward triangles.
// Nothing about an unavailable R² is drawn in the plot area (the caption below the graph says why).
import { axis, baseLayout } from '../plots/figures';
import type { Figure } from '../plots/Plot';
import type { Trace } from '../plots/plotly';
import type { PlotTheme } from '../plots/theme';
import { COLOURS, type PlotModel, type TraceModel } from './plotModel';

const LINE_WIDTH = 2;

function trace(m: TraceModel, unit: string): Trace {
  const hover = `Time (${unit}) = %{x:.6~g}<br>Cumulative release fraction (dimensionless) = %{y:.6~g}<extra>${m.name}</extra>`;
  const base = { type: 'scatter' as const, name: m.name, x: m.t, y: m.f, hovertemplate: hover, meta: { id: m.id, label: m.name, unit: '1' } };
  if (m.role === 'method_a' || m.role === 'method_b') {
    return { ...base, mode: 'lines', line: { color: m.role === 'method_a' ? COLOURS.A : COLOURS.B, width: LINE_WIDTH, dash: 'solid' } };
  }
  return {
    ...base,
    mode: 'markers',
    marker: { color: COLOURS.observations, size: 8, symbol: m.role === 'upper_bound' ? 'triangle-down' : 'circle', line: { color: '#ffffff', width: 1 } },
  };
}

export function comparisonFigure(model: PlotModel, theme: PlotTheme): Figure {
  const annotations = model.r2Labels.map((l, i) => ({
    xref: 'paper',
    yref: 'paper',
    x: 0.02,
    y: 0.98,
    yshift: -18 * i,
    xanchor: 'left',
    yanchor: 'top',
    text: l.text,
    showarrow: false,
    align: 'left',
    font: { size: 13, color: l.colour },
  }));
  return {
    data: model.traces.map((m) => trace(m, model.timeUnit)),
    layout: baseLayout(theme, {
      xaxis: axis(theme, `Time (${model.timeUnit})`, { rangemode: 'tozero', hoverformat: '.6~g' }),
      yaxis: axis(theme, 'Cumulative release fraction (dimensionless)', { rangemode: 'tozero' }),
      annotations,
      margin: { l: 64, r: 16, t: 72, b: 48, pad: 0 },
      legend: { orientation: 'h', x: 0, xanchor: 'left', yref: 'container', y: 1, yanchor: 'top', font: { size: 11, color: theme.text }, bgcolor: 'rgba(0,0,0,0)' },
    }),
  };
}
