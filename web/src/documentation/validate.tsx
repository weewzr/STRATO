import { repoFile } from '../build';
import { hrefDocumentation } from '../route';
import type { DocContribution } from './types';

// The experimental-comparison statements of Scientific Documentation, updated for the four-page layout (v2.1.0): the
// comparison is drawn on Explore, and nothing on this page refers to a Validate page.

export const validate: DocContribution = {
  limitations: [
    {
      id: 'validate-limits',
      title: 'What a model curve is compared with measurements under',
      body: (
        <>
          <p>
            Heating tests change the temperature over time and the measured nuclides decay, so a model curve for a heating test needs
            temperature-dependent diffusivity over the registered temperature history. Every comparison file of this build holds its model
            curves computed so: the diffusivity of each layer follows the registered heating history (D(r, T(t))), the kernel is uniform
            at the start, and decay is not applied. Explore therefore draws the model curves of a comparison file under temperature-dependent
            diffusion only, and names that configuration beneath the graph.
          </p>
          <p>
            Under any other model equation Explore shows the measurements alone, never a model curve computed with a different equation. A
            constant-temperature result of Method A: BTCS is never compared with heating-test data. The measurements are the registered,
            digitised or tabulated release values, each with its caption, APA citation and link; the temperature of the dataset is the
            temperature of its model curve, and Explore lists the datasets of the temperature chosen.
          </p>
        </>
      ),
    },
  ],
  interpretation: [
    {
      id: 'validate-status',
      title: 'Experimental comparison and validation',
      body: (
        <>
          <p>
            Validation asks whether the model matches measured release from real particles. It is a different question from verification,
            which asks whether the code solves its equations correctly. The agreement of the two solution methods on the synthetic case is
            verification and appears under <a href={hrefDocumentation('evidence-verification')}>the verification evidence</a>.
          </p>
          <h4>What the experimental comparison shows</h4>
          <ul className="prose-list">
            <li>
              Measured release, drawn on Explore from the generated comparison files, which are built from the{' '}
              <a href={repoFile('data/experimental/release_measurements.csv')}>release measurements (CSV)</a>, with uncertainty or digitising
              uncertainty and a reference for each series.
            </li>
            <li>
              Model curves only from the registered runs with the matching nuclide, particle geometry and temperature history, computed in
              Rust and stored with their provenance.
            </li>
            <li>
              The agreement metric and where it is shown: see <a href={hrefDocumentation('r2-requirements')}>R² and its suitability requirements</a>.
            </li>
          </ul>
        </>
      ),
    },
  ],
};
