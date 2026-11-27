import { useMemo } from 'react';
import { useCatalogue } from '../research/catalogue';
import { runIsCurrent, useStrato } from '../state/store';
import { buildCapability, selectionSupport, type BuildCapability, type SelectionSupport } from './capability';

/** The capability of the running build, from what its wrapper reported; `loading` until the wrapper has reported. */
export function useBuildCapability(): { cap: BuildCapability; loading: boolean } {
  const reported = useStrato((s) => s.capabilities);
  const catalogue = useCatalogue();
  const cap = useMemo(() => buildCapability(reported, catalogue), [reported, catalogue]);
  return { cap, loading: reported.length === 0 };
}

/**
 * Whether the build can run the manual baseline solver (the advanced solver configuration), which runs the
 * baseline problem whatever model equation and operation Explore has selected: the two groups of buttons do
 * not start it and do not change it. While the wrapper has not reported (loading, or the module failed to
 * load) the run counts as supported: the solver status line and the run problem note say what is wrong
 * there, and nothing is claimed about the build.
 */
export function useSelectionSupport(): SelectionSupport & { loading: boolean } {
  const { cap, loading } = useBuildCapability();
  return useMemo(() => (loading ? { supported: true, missing: [], loading } : { ...selectionSupport(cap, 'baseline', null), loading }), [cap, loading]);
}

/**
 * Whether a computed status may be shown now: the stored run belongs to the current selection and input
 * key, and the build supports the selection (state/store.ts runIsCurrent). Every computed badge, status
 * chip of a result and run-derived line reads this one condition.
 */
export function useRunIsCurrent(): boolean {
  const { supported } = useSelectionSupport();
  const run = useStrato((s) => s.run);
  const info = useStrato((s) => s.runInfo);
  const config = useStrato((s) => s.config);
  return runIsCurrent({ run, runInfo: info, config }, supported);
}
