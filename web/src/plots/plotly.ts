// Lazy loader and minimal types of the Plotly basic partial bundle (plotly.js-basic-dist-min 4.3.0,
// selected in docs/development/visualisation_library_comparison.md; prompt/TECHNICAL_ARCHITECTURE.md
// Section 4). The bundle is fetched the first time a plot is drawn, so the first screen does not wait
// for it. Plotly draws what it is given: every trace below carries arrays exactly as the Rust wrapper
// or a precomputed file returned them; axis ranges, ticks and log scaling are display operations.

/** One line or marker series. `x` and `y` are the returned arrays themselves, never derived copies. */
export interface Trace {
  type: 'scatter';
  name: string;
  x: readonly number[];
  y: readonly (number | null)[];
  mode: 'lines' | 'markers' | 'lines+markers';
  line?: { color?: string; width?: number; dash?: 'solid' | 'dash' | 'dot' | 'dashdot' | 'longdash'; shape?: 'linear' | 'hv' | 'vh' };
  marker?: { color?: string; size?: number; symbol?: string; line?: { color?: string; width?: number } };
  yaxis?: 'y' | 'y2';
  xaxis?: 'x';
  showlegend?: boolean;
  hovertemplate?: string;
  hoverinfo?: 'skip' | 'all' | 'none';
  /** Identifies the series for tests and for the CSV export (not drawn). */
  meta?: { id: string; label: string; unit?: string };
}

export type Layout = Record<string, unknown>;
export type PlotConfig = Record<string, unknown>;

/** The graph div after Plotly has drawn into it (Plotly attaches the figure it drew). */
export interface GraphDiv extends HTMLDivElement {
  data?: Trace[];
  layout?: Layout;
}

export interface PlotlyApi {
  react(gd: HTMLElement, data: readonly Trace[], layout: Layout, config?: PlotConfig): Promise<unknown>;
  relayout(gd: HTMLElement, update: Layout): Promise<unknown>;
  purge(gd: HTMLElement): void;
  toImage(fig: { data: readonly Trace[]; layout: Layout; config?: PlotConfig }, opts: { format: 'png' | 'svg'; width: number; height: number; scale?: number }): Promise<string>;
  Plots: { resize(gd: HTMLElement): Promise<unknown> | void };
}

let loading: Promise<PlotlyApi> | null = null;

/** Loads the Plotly bundle once (its own chunk); later calls share the same promise. */
export function loadPlotly(): Promise<PlotlyApi> {
  if (!loading) {
    loading = import('plotly.js-basic-dist-min').then((m) => {
      const mod = m as unknown as { default?: unknown };
      return (mod.default ?? mod) as PlotlyApi;
    });
    // A failed load can be retried on the next plot.
    loading.catch(() => {
      loading = null;
    });
  }
  return loading;
}

/** Plotly options of every figure on the site: no Plotly logo or link, no editing, responsive. */
export const PLOT_CONFIG: PlotConfig = {
  displaylogo: false,
  responsive: true,
  scrollZoom: false,
  showTips: false,
  // No floating tool bar: on a phone it covered the legend. Drag zooms, a double click resets the axes.
  displayModeBar: false,
  doubleClick: 'reset',
};
