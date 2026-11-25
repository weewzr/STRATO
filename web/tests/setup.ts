import { cleanup } from '@testing-library/react';
import { afterEach, beforeEach, vi } from 'vitest';
import { initialState, useStrato } from '../src/state/store';

// jsdom cannot lay out Plotly's SVG. A stand-in records the figure each plot hands to Plotly on its
// graph div (as Plotly does), so component tests can check what would be drawn.
vi.mock('plotly.js-basic-dist-min', () => {
  const record = (gd: HTMLElement & { data?: unknown; layout?: unknown }, data: unknown, layout: unknown) => {
    gd.data = data;
    gd.layout = layout;
    return Promise.resolve(gd);
  };
  return {
    default: {
      react: record,
      relayout: (gd: HTMLElement) => Promise.resolve(gd),
      purge: () => undefined,
      toImage: () => Promise.resolve('data:image/svg+xml,'),
      Plots: { resize: () => undefined },
    },
  };
});

// jsdom has no matchMedia; report "no match" for every query (phone layout, light theme, motion allowed).
if (typeof window !== 'undefined' && typeof window.matchMedia !== 'function') {
  window.matchMedia = (query: string) =>
    ({
      matches: false,
      media: query,
      onchange: null,
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
      addListener: () => undefined,
      removeListener: () => undefined,
      dispatchEvent: () => false,
    }) as MediaQueryList;
}

// jsdom has no canvas: getContext returns null (as a browser without the context would), so the
// particle drawing code takes its no-context path instead of jsdom printing "not implemented".
if (typeof HTMLCanvasElement !== 'undefined') {
  HTMLCanvasElement.prototype.getContext = (() => null) as unknown as HTMLCanvasElement['getContext'];
}

beforeEach(() => {
  useStrato.setState(initialState());
  if (typeof window !== 'undefined') window.location.hash = '';
});

afterEach(() => {
  cleanup();
});
