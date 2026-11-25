import { useEffect, useRef, useState } from 'react';
import { loadPlotly, PLOT_CONFIG, type GraphDiv, type Layout, type PlotlyApi, type Trace } from './plotly';

// Thin React wrapper around Plotly (prompt/TECHNICAL_ARCHITECTURE.md Section 4, "Plots"). It draws the
// figure it is given and nothing else. Redraws are coalesced to one per animation frame, so playback
// at 8x asks Plotly for at most one frame per display refresh. The graph div carries the traces Plotly
// drew (`data`), which the end-to-end tests compare with the wrapper's values.

export interface Figure {
  data: Trace[];
  layout: Layout;
}

export interface ExportInfo {
  /** File name stem of the image, without extension. */
  name: string;
  /** Source line written into the exported image: model, version, commit, label. */
  source: string;
}

type DrawState = 'loading' | 'ready' | 'failed';

/** On a touch screen a drag scrolls the page instead of zooming the plot. */
function coarsePointer(): boolean {
  return typeof window.matchMedia === 'function' && window.matchMedia('(pointer: coarse)').matches;
}

export function Plot({
  figure,
  label,
  testId,
  height,
  exportInfo,
  minIntervalMs = 0,
}: {
  figure: Figure;
  /** Smallest time between two redraws (display only); 0 redraws on every animation frame. */
  minIntervalMs?: number;
  /** Accessible name: what the plot shows, in words. */
  label: string;
  testId: string;
  height: number;
  exportInfo?: ExportInfo;
}) {
  const div = useRef<HTMLDivElement>(null);
  const plotly = useRef<PlotlyApi | null>(null);
  const latest = useRef<Figure>(figure);
  const frame = useRef<number | null>(null);
  const busy = useRef(false);
  const again = useRef(false);
  const draws = useRef(0);
  const wait = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastDraw = useRef(0);
  const [state, setState] = useState<DrawState>('loading');
  const [error, setError] = useState<string>('');

  // Draws the latest figure; while a draw is in flight, the next one waits for it.
  const draw = useRef<() => void>(() => undefined);
  useEffect(() => {
    draw.current = () => {
      const gd = div.current;
      const api = plotly.current;
      if (!gd || !api) return;
      if (busy.current) {
        again.current = true;
        return;
      }
      busy.current = true;
      const fig = latest.current;
      void api
        .react(gd, fig.data, { ...fig.layout, height, ...(coarsePointer() ? { dragmode: false } : {}) }, PLOT_CONFIG)
        .then(() => {
          draws.current += 1;
          gd.dataset.draws = String(draws.current);
          gd.dataset.ready = 'true';
        })
        .catch((e: unknown) => {
          setError(e instanceof Error ? e.message : String(e));
          setState('failed');
        })
        .finally(() => {
          busy.current = false;
          if (again.current) {
            again.current = false;
            draw.current();
          }
        });
    };
  }, [height]);

  useEffect(() => {
    let alive = true;
    loadPlotly()
      .then((api) => {
        if (!alive) return;
        plotly.current = api;
        setState('ready');
        draw.current();
      })
      .catch((e: unknown) => {
        if (!alive) return;
        setError(e instanceof Error ? e.message : String(e));
        setState('failed');
      });
    const gd = div.current;
    return () => {
      alive = false;
      if (frame.current !== null) cancelAnimationFrame(frame.current);
      if (gd && plotly.current) plotly.current.purge(gd);
    };
  }, []);

  useEffect(() => {
    latest.current = figure;
    if (!plotly.current) return;
    if (frame.current !== null || wait.current !== null) return;
    const schedule = () => {
      frame.current = requestAnimationFrame(() => {
        frame.current = null;
        lastDraw.current = performance.now();
        draw.current();
      });
    };
    // During playback the plot redraws at most once per minIntervalMs, so the particle keeps its
    // frame rate; the last row is always drawn.
    const since = performance.now() - lastDraw.current;
    if (minIntervalMs > 0 && since < minIntervalMs) {
      wait.current = setTimeout(() => {
        wait.current = null;
        schedule();
      }, minIntervalMs - since);
    } else schedule();
  }, [figure, height, minIntervalMs]);

  useEffect(
    () => () => {
      if (wait.current !== null) clearTimeout(wait.current);
    },
    [],
  );

  // Follow the width of the container (the grid can change it without a window resize).
  useEffect(() => {
    const gd = div.current;
    if (!gd || typeof ResizeObserver !== 'function') return;
    const obs = new ResizeObserver(() => {
      if (plotly.current && gd.dataset.ready === 'true') void plotly.current.Plots.resize(gd);
    });
    obs.observe(gd);
    return () => obs.disconnect();
  }, []);

  const exportImage = async (format: 'png' | 'svg') => {
    const gd = div.current as GraphDiv | null;
    const api = plotly.current;
    if (!gd || !api || !exportInfo) return;
    const layout = (gd.layout ?? latest.current.layout) as Layout & { annotations?: unknown[]; margin?: Record<string, number> };
    const width = 960;
    const exportHeight = 600;
    const url = await api.toImage(
      {
        data: gd.data ?? latest.current.data,
        layout: {
          ...layout,
          width,
          height: exportHeight,
          margin: { ...(layout.margin ?? {}), b: 110 },
          annotations: [
            ...(layout.annotations ?? []),
            {
              text: exportInfo.source,
              xref: 'paper',
              yref: 'paper',
              x: 0,
              y: 0,
              yshift: -96,
              xanchor: 'left',
              yanchor: 'bottom',
              showarrow: false,
              align: 'left',
              font: { size: 10 },
            },
          ],
        },
      },
      { format, width, height: exportHeight },
    );
    const a = document.createElement('a');
    a.href = url;
    a.download = `${exportInfo.name}.${format}`;
    document.body.appendChild(a);
    a.click();
    a.remove();
  };

  return (
    <div className="plot" data-testid={testId} data-state={state}>
      <div ref={div} className="plot__graph" role="img" aria-label={label} style={{ height }} data-testid={`${testId}-graph`} />
      {state === 'loading' ? (
        <p className="plot__status" role="status">
          Loading the plotting library…
        </p>
      ) : null}
      {state === 'failed' ? (
        <p className="plot__status" role="alert" data-testid={`${testId}-failed`}>
          The plot could not be drawn ({error}). The values are in the downloads of the Data section.
        </p>
      ) : null}
      {exportInfo && state === 'ready' ? (
        <div className="plot__export" role="group" aria-label="Save the figure as an image">
          <span className="plot__export-label">Save figure</span>
          <button type="button" className="btn btn--small" onClick={() => void exportImage('svg')} data-testid={`${testId}-svg`}>
            SVG
          </button>
          <button type="button" className="btn btn--small" onClick={() => void exportImage('png')} data-testid={`${testId}-png`}>
            PNG
          </button>
        </div>
      ) : null}
    </div>
  );
}

/** The traces a plot drew (for tests): reads the figure Plotly attached to the graph div. */
export function drawnTraces(el: Element | null): Trace[] {
  return ((el as GraphDiv | null)?.data ?? []) as Trace[];
}
