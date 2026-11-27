// The experimental-comparison graph component is mounted at the mount points of Compare and Validate, behind
// its data check: with no comparison file in the build the mount points are empty and the pages are as they
// were; with a comparison file the graph appears inside the mount point.
import { render } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import both from './fixtures/fixture_r2_both_methods.json';

const state = vi.hoisted(() => ({ files: [] as unknown[] }));
vi.mock('../src/graph/files', () => ({
  get COMPARISON_FILES() {
    return state.files;
  },
  COMPARISON_FILE_CHECKS: [],
}));

import { ExperimentalEvidence } from '../src/documentation/evidence';
import { initialState, useStrato } from '../src/state/store';

const MOUNT = '[data-mount="experimental-comparison-graph"]';

beforeEach(() => {
  useStrato.setState({ ...initialState() });
  state.files = [];
});

describe('the mount point of the experimental graphs in Scientific Documentation', () => {
  for (const [name, Page] of [
    ['Experimental comparison graphs', ExperimentalEvidence],
  ] as const) {
    it(`${name}: empty with no comparison file in the build`, () => {
      const { container } = render(<Page />);
      const mount = container.querySelector(MOUNT);
      expect(mount).not.toBeNull();
      expect(mount?.childElementCount).toBe(0);
      expect(container.querySelector('[data-testid="exp-comparisons"]')).toBeNull();
    });

    it(`${name}: holds the graph of a comparison file inside the mount point`, () => {
      state.files = [both];
      const { container } = render(<Page />);
      const mount = container.querySelector(MOUNT);
      expect(mount?.querySelector('[data-testid="exp-comparisons"]')).not.toBeNull();
    });
  }
});