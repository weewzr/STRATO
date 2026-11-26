import { useMemo } from 'react';
import { useCatalogue } from '../research/catalogue';
import { useStrato } from '../state/store';
import { buildCapability, selectionSupport, type BuildCapability, type SelectionSupport } from './capability';

/** The capability of the running build, from what its wrapper reported; `loading` until the wrapper has reported. */
export function useBuildCapability(): { cap: BuildCapability; loading: boolean } {
  const reported = useStrato((s) => s.capabilities);
  const catalogue = useCatalogue();
  const cap = useMemo(() => buildCapability(reported, catalogue), [reported, catalogue]);
  return { cap, loading: reported.length === 0 };
}

/**
 * Whether the build can run the selection of the Explore page. While the wrapper has not reported
 * (loading, or the module failed to load) the selection counts as supported: the solver status line
 * and the run problem note say what is wrong there, and nothing is claimed about the build.
 */
export function useSelectionSupport(): SelectionSupport & { loading: boolean } {
  const { cap, loading } = useBuildCapability();
  const physics = useStrato((s) => s.selection.physics);
  const scenario = useStrato((s) => s.selection.scenario);
  return useMemo(() => (loading ? { supported: true, missing: [], loading } : { ...selectionSupport(cap, physics, scenario), loading }), [cap, loading, physics, scenario]);
}
