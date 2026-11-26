import { METHOD_1 } from '../methods';
import { useStrato } from '../state/store';
import { computedMechanisms, isSimulable, MECHANISM_NAME, phrase } from './capability';
import { useBuildCapability } from './useCapability';

const capitalise = (s: string): string => (s === '' ? s : s.charAt(0).toUpperCase() + s.slice(1));

/**
 * Which solver the live simulation and the results come from and which physics its computation
 * implements, read from the capability of the running build (what its wrapper reported); nothing is
 * typed per build. `short` is the line of the Results block. `supported` says whether the selection is run:
 * when it is not, nothing is running, and the line names the solver of the build instead of a running one.
 */
export function ActiveModel({ short = false, supported = true }: { short?: boolean; supported?: boolean }) {
  const { cap, loading } = useBuildCapability();
  const version = useStrato((s) => s.solver.build?.btcs.solver.version ?? null);
  if (loading) return null;
  const { computed, notComputed } = computedMechanisms(cap);
  const tail =
    notComputed.length > 0
      ? `${capitalise(phrase(notComputed.map((m) => MECHANISM_NAME[m])))} ${notComputed.length > 1 ? 'are' : 'is'} not computed by this build, so no ${notComputed.length > 1 ? 'result of them' : 'result of it'} is shown.`
      : '';
  const model = isSimulable(cap.baseline)
    ? `the baseline spherical diffusion in the five layers${
        computed.length > 0 ? ` with ${phrase(computed.map((m) => MECHANISM_NAME[m]))}` : ' (constant diffusivity in each layer, no generation and no decay)'
      }`
    : 'no model of this page';
  return (
    <p className="block__note active-model" data-testid={short ? 'active-model-results' : 'active-model'} data-computed={computed.join(',')}>
      {short ? (
        <>Shown: output of {METHOD_1}{version ? `, solver version ${version}` : ''}. </>
      ) : (
        <>
          {supported ? 'Running computation' : 'Solver of this build'}: <strong>{METHOD_1}</strong>
          {version ? `, solver version ${version}` : ''}. It implements {model}.{' '}
        </>
      )}
      {tail}
    </p>
  );
}
