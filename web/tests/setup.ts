import { cleanup } from '@testing-library/react';
import { afterEach, beforeEach } from 'vitest';
import { initialState, useStrato } from '../src/state/store';

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
