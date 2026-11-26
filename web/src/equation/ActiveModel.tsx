import { useStrato } from '../state/store';
import { computedMechanisms, isSimulable, MECHANISM_NAME, phrase } from './capability';
import { useBuildCapability } from './useCapability';

const capitalise = (s: string): string => (s === '' ? s : s.charAt(0).toUpperCase() + s.slice(1));

/**
 * Which physics the computation of the build implements, in one line, read from the capability of the
 * running build (what its wrapper reported); nothing is typed per build. The solver itself is named by the
 * status line beside it. `supported` says whether the selection is run: when it is not, nothing is running.
 */
export function ActiveModel({ supported = true }: { supported?: boolean }) {
  const { cap, loading } = useBuildCapability();
  const solverState = useStrato((s) => s.solver.state);
  if (loading || solverState !== 'ready') return null;
  const { computed, notComputed } = computedMechanisms(cap);
  const model = isSimulable(cap.baseline)
    ? `baseline diffusion${computed.length > 0 ? ` with ${phrase(computed.map((m) => MECHANISM_NAME[m]))}` : ''}`
    : 'no model of this page';
  const tail = notComputed.length > 0 ? ` Not computed: ${phrase(notComputed.map((m) => MECHANISM_NAME[m]))}.` : '';
  return (
    <p className="block__note active-model" data-testid="active-model" data-computed={computed.join(',')}>
      {supported ? 'Computes' : 'This build computes'}: {capitalise(model)}.{tail}
    </p>
  );
}