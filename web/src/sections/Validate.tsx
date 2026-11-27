import { ExperimentalComparisons } from '../graph/Comparisons';
import { hrefDocumentation } from '../route';
import { useStrato } from '../state/store';
import { SectionHeading } from './common';

/**
 * The switch of the validation layer (prompt/DESIGN_SPECIFICATION.md Section 8.2, independent of the
 * comparison of Section 8.1). No prediction exists before the Phase 2 physics, so the switch is shown,
 * focusable and disabled; operating it changes nothing, and nothing in Compare depends on it.
 */
function ValidationLayer() {
  const on = useStrato((s) => s.view.showValidation);
  return (
    <div className="switch-row" role="group" aria-label="Validation layer">
      <button
        type="button"
        role="switch"
        className="btn btn--small"
        aria-checked={on}
        aria-disabled="true"
        aria-describedby="validation-layer-why"
        data-testid="toggle-validation"
        onClick={() => undefined}
      >
        Model predictions against measurements
      </button>
      <span id="validation-layer-why" className="muted" data-testid="validation-status">
        No comparison is shown yet: model predictions need generation, decay and temperature-dependent diffusion.{' '}
        <a href={hrefDocumentation('validate-limits')}>Why</a>
      </span>
    </div>
  );
}

export function Validate() {
  return (
    <article className="section" data-section="validate">
      <SectionHeading>Validation against measurements</SectionHeading>
      <ValidationLayer />

      <div data-mount="experimental-comparison-graph" data-testid="experimental-comparison-mount">
        <ExperimentalComparisons />
      </div>
    </article>
  );
}
