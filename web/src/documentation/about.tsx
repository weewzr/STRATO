import { BUILD, REPO_URL, repoFile } from '../build';
import { METHOD_1, METHOD_2 } from '../methods';
import { Badge } from '../panels/Badge';
import { hrefDocumentation } from '../route';
import { TableBox } from '../sections/common';
import type { ResultLabel } from '../solver/types';
import type { DocContribution } from './types';

// Scientific material that left the About page, moved with its meaning unchanged: the model assumptions, the
// two methods, what is implemented, the definitions of the result labels and the contents of the build by
// web phase.

const LABELS: { label: ResultLabel; meaning: string }[] = [
  {
    label: 'COMPUTED v2.0',
    meaning: `Produced in this page view by ${METHOD_1} (BTCS finite differences), the verified solver, from the inputs shown. The value is a numerical solution of the stated model, not an experimental measurement.`,
  },
  { label: 'PRECOMPUTED', meaning: 'Produced earlier by the native STRATO binary; the file records commit, input hash and command.' },
  { label: 'ILLUSTRATIVE', meaning: 'A schematic or animation that is not derived from solver output.' },
  { label: 'MEASURED', meaning: 'An experimental value from the repository data, with its source and uncertainty.' },
];

const BUILD_CONTENTS: { part: string; here: boolean; phase: string }[] = [
  {
    part: 'Equation-first page of the first web phase (later replaced by the four-page layout: Explore, Basis, Research Catalogue, Scientific Documentation)',
    here: true,
    phase: '1',
  },
  { part: 'Controls bound to the inputs of the manual solver, input checks, light and dark themes', here: true, phase: '1' },
  { part: 'Particle cross-section drawn to the input radii, one colour per layer (ILLUSTRATIVE)', here: true, phase: '1' },
  { part: 'Staged equation builder (KaTeX, numbered equations) and the research status of each mechanism (Explore: independent Model Equation and Operation)', here: true, phase: '2' },
  {
    part: `${METHOD_1} in the browser (Rust compiled to WebAssembly, in a worker): release fraction, inventory balance with its residual, layer inventories and numerical diagnostics of every run, with ${METHOD_2} beside it; precomputed native output with provenance`,
    here: true,
    phase: '3',
  },
  { part: 'Particle view with concentration colouring, layer inventory shares, playback and the AGR-1 reference particle (in the manual solver of Scientific Documentation)', here: true, phase: '4' },
  { part: 'Radial profile, release curves, model comparison, verification plots, downloads of results (verification plots and downloads in Scientific Documentation)', here: true, phase: '5' },
  { part: 'Research catalogue: transport regimes, species entries, Materials Studio gallery, brick roadmap', here: true, phase: '5.5' },
  { part: 'Radioactive decay, fission-product generation, temperature-dependent diffusivity and their combination (stages 2 to 5 of the wrapper; radioactive decay and temperature-dependent diffusion are drawn on Explore, generation and the combined model are computed but not drawn)', here: true, phase: '6' },
  { part: 'Measured release data with model curves, captions and citations on Explore (R² only where a record holds a value)', here: true, phase: '7' },
];

/** What the build contains, by web phase (checked against prompt/PROGRESS.md by the unit test). */
export function BuildContentsTable() {
  return (
    <TableBox label="Contents of this build">
      <table className="table" data-testid="build-table">
        <caption className="table__caption">
          <strong>Table 2.</strong> What this build of the web demonstration contains, by web phase. Build <code>{BUILD.commit}</code>, STRATO v
          {BUILD.strato_version}.
        </caption>
        <thead>
          <tr>
            <th scope="col">Part</th>
            <th scope="col">In this build</th>
            <th scope="col" className="num-col">
              Web phase
            </th>
          </tr>
        </thead>
        <tbody>
          {BUILD_CONTENTS.map((b) => (
            <tr key={b.part}>
              <th scope="row">{b.part}</th>
              <td>{b.here ? 'Yes' : 'Not yet'}</td>
              <td className="num-col num">{b.phase}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </TableBox>
  );
}

function ResultLabels() {
  return (
    <>
      <p>Every number, plot and particle colouring carries one of these labels. Each label is a word with its own outline or pattern.</p>
      <TableBox label="Result labels">
        <table className="table" data-testid="result-labels">
          <caption className="visually-hidden">Result labels</caption>
          <thead>
            <tr>
              <th scope="col">Label</th>
              <th scope="col">Meaning</th>
            </tr>
          </thead>
          <tbody>
            {LABELS.map((l) => (
              <tr key={l.label}>
                <td>
                  <Badge label={l.label} />
                </td>
                <td>{l.meaning}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </TableBox>
    </>
  );
}

export const about: DocContribution = {
  assumptions: [
    {
      id: 'about-model',
      title: 'The model and its boundary conditions',
      body: (
        <>
          <p>
            The centre is symmetric, interfaces are in perfect contact, the outer surface is a perfect sink and the initial inventory is uniform
            in the kernel.
          </p>
          <p>The release fraction is the share of the initial inventory that has left through the outer surface.</p>
        </>
      ),
    },
  ],
  limitations: [
    {
      id: 'about-implemented',
      title: 'What is implemented',
      body: (
        <>
          <ul className="prose-list">
            <li>
              The manual solver of this page runs {METHOD_1}: diffusion through the five layers at constant temperature, with a constant
              diffusivity per layer and the initial inventory in the kernel. Verified on a synthetic case.
            </li>
            <li>
              Solved through the run_stage function of the wrapper (stages 1 to 5) and computed for Steady-state operation. Drawn on Explore are baseline
              diffusion, radioactive decay (every species with a registered decay constant) and temperature-dependent diffusion; generation and the combined model are computed by
              run_stage but not drawn (fresh particle). Experimental
              comparisons are drawn on Explore with captions and citations; an R² is shown only where a record holds a value.
            </li>
          </ul>
          <p>
            The bundled synthetic case uses radii and diffusivities chosen for verification (see <a href={hrefDocumentation('data-case-file')}>the case file</a>);
            no result from this case is a prediction of real fission-product release.
          </p>
        </>
      ),
    },
    {
      id: 'about-method-b-scope',
      title: 'Where Method B applies',
      body: (
        <p>
          {METHOD_2} applies at constant temperature with a constant diffusivity per layer, including the documented extensions for
          radioactive decay (B2) and piecewise-constant kernel sources (B3–B5) within its convergence mask; it never applies when
          temperature varies in time.
        </p>
      ),
    },
  ],
  derivations: [
    {
      id: 'about-methods',
      title: 'The two methods',
      body: (
        <p>
          STRATO solves the problem in two ways, and the page offers exactly these two. {METHOD_1} is a finite-difference method: spherical
          central differences in space on a grid with nodes on the layer interfaces, and Backward Euler time stepping. {METHOD_2} is the
          multilayer Sturm-Liouville eigenfunction series.
        </p>
      ),
    },
  ],
  // The definitions of the result labels belong with the provenance of results, not with the experimental interpretation.
  provenance: [
    { id: 'about-labels', title: 'Result labels', body: <ResultLabels /> },
    {
      id: 'about-credits',
      title: 'Licence, credits and citation',
      body: (
        <>
          <p>
            <a href={REPO_URL}>Repository</a> and <a href={repoFile('CITATION.cff')}>citation metadata</a>. Licence: GNU Affero General Public
            License, version 3 (AGPL-3.0-only).
          </p>
          <p>
            Credits: the status line and the explicit labels follow Theodore&apos;s{' '}
            <a href="https://github.com/theodoreOnzGit/outram-park-backend">Outram Park</a> dispersion demonstration (AGPL-3.0-only; no file of
            it is copied here). Typefaces Literata, IBM Plex Sans and IBM Plex Mono, SIL Open Font License 1.1.
          </p>
        </>
      ),
    },
  ],
  implementation: [{ id: 'about-build-contents', title: 'Contents of this build', body: <BuildContentsTable /> }],
};
