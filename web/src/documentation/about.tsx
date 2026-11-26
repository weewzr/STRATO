import { BUILD } from '../build';
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
    part: 'Equation-first page: governing equation, live simulation, controls, explanation and future research, in this order, with placeholders for later phases',
    here: true,
    phase: '1',
  },
  { part: 'Controls bound to the inputs, input checks, light and dark themes', here: true, phase: '1' },
  { part: 'Particle cross-section drawn to the input radii, one colour per layer (ILLUSTRATIVE)', here: true, phase: '1' },
  { part: 'Staged equation builder (KaTeX, numbered equations) and the research status of each mechanism', here: true, phase: '2' },
  {
    part: `${METHOD_1} in the browser (Rust compiled to WebAssembly, in a worker): release fraction, inventory balance with its residual, layer inventories and numerical diagnostics of every run, with ${METHOD_2} beside it; precomputed native output with provenance`,
    here: true,
    phase: '3',
  },
  { part: 'Particle view with concentration colouring, layer inventory shares, playback and the AGR-1 reference particle', here: true, phase: '4' },
  { part: 'Radial profile, release curves, model comparison, verification plots, downloads of results', here: true, phase: '5' },
  { part: 'Research catalogue: transport regimes, species entries, Materials Studio gallery, brick roadmap', here: true, phase: '5.5' },
  { part: 'Decay, generation and temperature-dependent diffusivity (derived, not implemented)', here: false, phase: '6' },
  { part: 'Measured release data and, after phase 6, model predictions', here: false, phase: '7' },
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
              Solved by {METHOD_1}: diffusion through the five layers at constant temperature, with a constant diffusivity per layer and the
              initial inventory in the kernel. Verified on a synthetic case.
            </li>
            <li>
              Shown as previews with a label, not solved: fission-product generation, radioactive decay and temperature-dependent diffusivity.
              The comparison with experimental measurements is a placeholder.
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
          {METHOD_2} applies where the problem is the one it solves: a constant diffusivity per layer, no source, no decay and a constant
          temperature. Each checks the other.
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
  provenance: [{ id: 'about-labels', title: 'Result labels', body: <ResultLabels /> }],
  implementation: [{ id: 'about-build-contents', title: 'Contents of this build', body: <BuildContentsTable /> }],
};
