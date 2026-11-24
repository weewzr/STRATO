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

beforeEach(() => {
  useStrato.setState(initialState());
  if (typeof window !== 'undefined') window.location.hash = '';
});

afterEach(() => {
  cleanup();
});
