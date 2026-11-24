import { repoFile } from '../build';
import { hrefFor } from '../route';
import { Note, SectionHeading } from './common';

export function Validate() {
  return (
    <article className="section" data-section="validate">
      <SectionHeading lede="Validation asks whether the model matches measured release from real particles. It is a different question from verification, which asks whether the code solves its equations correctly.">
        Validation against measurements
      </SectionHeading>
      <Note tone="warn">
        <p>
          <strong>Model predictions require Phase 2 physics.</strong> Heating tests change the temperature over time and the measured
          nuclides decay, so a comparison needs temperature-dependent diffusivity, temperature histories and radioactive decay. These are
          derived in the Phase 2 documents and not implemented. A v1.0 constant-temperature result is never compared with heating-test data.
        </p>
      </Note>
      <h2>What this section will show</h2>
      <ul className="prose-list">
        <li>
          Measured release from{' '}
          <a href={repoFile('data/experimental/release_measurements.csv')}>
            <code>data/experimental/release_measurements.csv</code>
          </a>
          , with uncertainty or digitising uncertainty and a reference for each series (web phase 7).
        </li>
        <li>Model predictions only from reviewed Phase 2 runs with the matching nuclide, particle geometry and temperature history.</li>
        <li>Error metrics computed by a documented script or in Rust, never typed in.</li>
      </ul>
      <p>
        The agreement of the two v1.0 solution methods on the synthetic case is verification and appears under{' '}
        <a href={hrefFor('verify')}>Verify</a>.
      </p>
    </article>
  );
}
