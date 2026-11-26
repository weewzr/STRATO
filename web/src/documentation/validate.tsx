import { repoFile } from '../build';
import { hrefFor } from '../route';
import type { DocContribution } from './types';

// Scientific material that left the Validate page, moved with its meaning unchanged.

export const validate: DocContribution = {
  limitations: [
    {
      id: 'validate-limits',
      title: 'Why no model prediction is compared with measurements yet',
      body: (
        <>
          <p>
            Model predictions require generation, decay and temperature-dependent diffusion. Heating tests change the temperature over time
            and the measured nuclides decay, so a comparison needs temperature-dependent diffusivity, temperature histories and radioactive
            decay. These are shown in the equation builder as labelled previews and are not solved. A constant-temperature result of Method A:
            BTCS is never compared with heating-test data.
          </p>
          <p>
            No model prediction exists until generation, decay and temperature-dependent diffusion are implemented and verified, and the
            measurements are plotted when the comparison step is built. The switch on the Validate page is a labelled placeholder: no
            comparison is shown.
          </p>
        </>
      ),
    },
  ],
  interpretation: [
    {
      id: 'validate-status',
      title: 'Validation against measurements',
      body: (
        <>
          <p>
            Validation asks whether the model matches measured release from real particles. It is a different question from verification,
            which asks whether the code solves its equations correctly. The agreement of the two solution methods on the synthetic case is
            verification and appears under <a href={hrefFor('verify')}>Verify</a>.
          </p>
          <ul className="prose-list">
            <li>
              Measured release from the{' '}
              <a href={repoFile('data/experimental/release_measurements.csv')}>release measurements (CSV)</a>, with uncertainty or digitising
              uncertainty and a reference for each series.
            </li>
            <li>Model predictions only from reviewed runs with the matching nuclide, particle geometry and temperature history.</li>
            <li>
              R² (coefficient of determination) is the only displayed whole-curve agreement metric; it is computed in Rust, never typed in.
              Where R² is unavailable, a short caption below the graph says why.
            </li>
          </ul>
        </>
      ),
    },
  ],
};
