import type { RunOk } from './types';
import { BASELINE_RUN_KEY, inputKey, useStrato } from '../state/store';

/**
 * The run the page may show for the current selection (owner directive of 26 Nov 2026, Sections 6 and 7:
 * no silent substitution). A reply belongs to the physics and scenario that were selected when it was sent
 * (RunInfo.selectionKey); once the selection is another one, the earlier reply is not shown under it, so a
 * number never appears beside an equation or a label of a model that did not produce it. `stale` marks a
 * reply of the same selection whose inputs have changed or whose rerun is pending.
 */
export function useShownRun(): { run: RunOk | null; stale: boolean } {
  const run = useStrato((s) => s.run);
  const info = useStrato((s) => s.runInfo);
  const config = useStrato((s) => s.config);
  const status = useStrato((s) => s.status);
  if (run === null) return { run: null, stale: false };
  if (info?.selectionKey !== undefined && info.selectionKey !== BASELINE_RUN_KEY) return { run: null, stale: false };
  return { run, stale: status === 'running' || info?.inputKey !== inputKey(config) };
}
