import type { RunOk } from './types';
import { useSelectionSupport } from '../equation/useCapability';
import { BASELINE_RUN_KEY, inputKey, useStrato } from '../state/store';

/**
 * The run the manual solver may show (owner directive of 26 Nov 2026, Sections 6 and 7: no silent
 * substitution). The manual solver runs the baseline problem whatever model equation and operation Explore has
 * selected, so a reply belongs to it when it was sent as the baseline run (RunInfo.selectionKey); a reply of
 * any other problem is not shown, so a number never appears beside a label of a model that did not produce it.
 * `stale` marks a reply whose inputs have changed or whose rerun is pending.
 */
export function useShownRun(): { run: RunOk | null; stale: boolean } {
  const run = useStrato((s) => s.run);
  const info = useStrato((s) => s.runInfo);
  const config = useStrato((s) => s.config);
  const status = useStrato((s) => s.status);
  const { supported } = useSelectionSupport();
  if (run === null || !supported) return { run: null, stale: false };
  if (info?.selectionKey !== undefined && info.selectionKey !== BASELINE_RUN_KEY) return { run: null, stale: false };
  return { run, stale: status === 'running' || info?.inputKey !== inputKey(config) };
}