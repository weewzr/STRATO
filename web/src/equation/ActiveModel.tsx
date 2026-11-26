import { METHOD_1 } from '../methods';
import { useStrato } from '../state/store';
import { computedMechanisms, MECHANISM_NAME, phrase } from './capability';
import { useBuildCapability } from './useCapability';

/**
 * Which solver the live simulation and the results come from and which physics its computation
 * implements, read from the capability of the running build (what its wrapper reported); nothing is
 * typed per build. `short` is the line of the Results block.
 */
export function ActiveModel({ short = false }: { short?: boolean }) {
  const { cap, loading } = useBuildCapability();
  const version = useStrato((s) => s.solver.build?.btcs.solver.version ?? null);
  if (loading) return null;
  const { computed, notComputed } = computedMechanisms(cap);
  const tail = notComputed.length > 0 ? `${phrase(notComputed.map((m) => MECHANISM_NAME[m]))} ${notComputed.length > 1 ? 'are' : 'is'} not computed by this build, so no ${notComputed.length > 1 ? 'result of them' : 'result of it'} is shown.` : '';
  return (
    <p className="block__note active-model" data-testid={short ? 'active-model-results' : 'active-model'} data-computed={computed.join(',')}>
      {short ? (
        <>Shown: output of {METHOD_1}{version ? `, solver version ${version}` : ''}. </>
      ) : (
        <>
          Running computation: <strong>{METHOD_1}</strong>
          {version ? `, solver version ${version}` : ''}. It implements the baseline spherical diffusion in the five layers
          {computed.length > 0 ? ` with ${phrase(computed.map((m) => MECHANISM_NAME[m]))}` : ' (constant diffusivity in each layer, no generation and no decay)'}.{' '}
        </>
      )}
      {tail}
    </p>
  );
}
