import { repoFile } from '../build';
import { ExperimentalComparisons } from '../graph/Comparisons';
import { hrefFor } from '../route';
import { useStrato } from '../state/store';
import { Note, SectionHeading } from './common';

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
      <span id="validation-layer-why" className="muted">
        Not available: no model prediction exists until generation, decay and temperature-dependent diffusion are implemented and verified,
        and the measurements are plotted when the comparison step is built. This is a labelled placeholder: no comparison is shown.
      </span>
    </div>
  );
}

export function Validate() {
  return (
    <article className="section" data-section="validate">
      <SectionHeading lede="Validation asks whether the model matches measured release from real particles. It is a different question from verification, which asks whether the code solves its equations correctly.">
        Validation against measurements
      </SectionHeading>
      <Note tone="warn">
        <p>
          <strong>Model predictions require generation, decay and temperature-dependent diffusion.</strong> Heating tests change the
          temperature over time and the measured nuclides decay, so a comparison needs temperature-dependent diffusivity, temperature histories
          and radioactive decay. These are shown in the equation builder as labelled previews and are not solved. A constant-temperature result
          of Method A: BTCS is never compared with heating-test data.
        </p>
      </Note>
      <ValidationLayer />
      <h2>What this section will show</h2>
      <ul className="prose-list">
        <li>
          Measured release from{' '}
          <a href={repoFile('data/experimental/release_measurements.csv')}>
            <code>data/experimental/release_measurements.csv</code>
          </a>
          , with uncertainty or digitising uncertainty and a reference for each series.
        </li>
        <li>Model predictions only from reviewed runs with the matching nuclide, particle geometry and temperature history.</li>
        <li>Error metrics computed in Rust, never typed in.</li>
      </ul>
      <p>
        The agreement of the two solution methods on the synthetic case is verification and appears under{' '}
        <a href={hrefFor('verify')}>Verify</a>.
      </p>
      <ExperimentalComparisons />
    </article>
  );
}
