import { BUILD, REPO_URL, repoFile } from '../build';
import { Badge } from '../panels/Badge';
import type { ResultLabel } from '../solver/types';
import { SectionHeading, TableBox } from './common';

const LABELS: { label: ResultLabel; meaning: string }[] = [
  { label: 'COMPUTED v1.0', meaning: 'Produced in this page view by the verified v1.0 solver from the inputs shown.' },
  { label: 'COMPUTED v2.0', meaning: 'Produced by the reviewed Phase 2 implementation. Not available: Phase 2 is not implemented.' },
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
    part: 'STRATO v1.0 solver in the browser (Rust compiled to WebAssembly, in a worker): release fraction, inventory balance with its residual, layer inventories and numerical diagnostics of every run; precomputed native output with provenance',
    here: true,
    phase: '3',
  },
  { part: 'Particle view with concentration colouring, layer inventory shares, playback and the AGR-1 reference particle', here: true, phase: '4' },
  { part: 'Radial profile, release curves, model comparison, verification plots, downloads of results', here: true, phase: '5' },
  { part: 'Research catalogue: transport regimes, species entries, Materials Studio gallery, brick roadmap', here: true, phase: '5.5' },
  { part: 'Decay, generation and temperature-dependent diffusivity (derived, not implemented)', here: false, phase: '6' },
  { part: 'Measured release data and, after phase 6, model predictions', here: false, phase: '7' },
];

// `page`: the page that crates/strato-docsite renders from the same file into this site (web phase 8),
// with the mathematics typeset; documents without one are linked on GitHub only.
const docs: { path: string; title: string; page?: string }[] = [
  {
    path: 'docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md',
    title: 'Full Mathematical Derivation (v1.0)',
    page: 'docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.html',
  },
  {
    path: 'docs/mathematics/v1.0.0/STRATO_Final_Deliverable.md',
    title: 'Final Deliverable: equations, pseudocode and Rust code (v1.0)',
    page: 'docs/mathematics/v1.0.0/STRATO_Final_Deliverable.html',
  },
  {
    path: 'docs/mathematics/STRATO_Phase2_Derivations.md',
    title: 'Phase 2 derivations (design only, not implemented)',
    page: 'docs/mathematics/STRATO_Phase2_Derivations.html',
  },
  {
    path: 'docs/mathematics/STRATO_Phase2_Pseudocode.md',
    title: 'Phase 2 pseudocode (design only, not implemented)',
    page: 'docs/mathematics/STRATO_Phase2_Pseudocode.html',
  },
  { path: 'data/reference/STRATO_Parameter_Register.md', title: 'Parameter register', page: 'data/reference/STRATO_Parameter_Register.html' },
  { path: 'docs/development/provenance.md', title: 'Provenance and verification evidence' },
];

export function About() {
  return (
    <article className="section section--doc" data-section="about">
      <SectionHeading lede="STRATO (Spherical TRISO Analysis and Transport Operations) models fission-product diffusion through the five layers of a TRISO coated fuel particle.">
        About STRATO and this demonstration
      </SectionHeading>

      <h2>The v1.0 model</h2>
      <p>
        The particle is five concentric spherical layers: kernel, buffer, inner pyrolytic carbon (IPyC), silicon carbide (SiC) and outer
        pyrolytic carbon (OPyC). In layer <i>i</i> the concentration <i>c</i>(<i>r</i>, <i>t</i>) obeys spherical diffusion with a constant
        diffusivity <i>D</i>
        <sub>i</sub>:
      </p>
      <p className="eq-plain" role="math" aria-label="partial c over partial t equals one over r squared times partial over partial r of r squared D i partial c over partial r">
        ∂<i>c</i>/∂<i>t</i> = (1/<i>r</i>
        <sup>2</sup>) ∂/∂<i>r</i> (<i>r</i>
        <sup>2</sup> <i>D</i>
        <sub>i</sub> ∂<i>c</i>/∂<i>r</i>)
      </p>
      <p>
        The centre is symmetric, interfaces are in perfect contact, the outer surface is a perfect sink and the start inventory is uniform in
        the kernel. The release fraction <i>F</i>(<i>t</i>) is the share of the start inventory that has left through the outer surface. The
        staged equation builder on the Explore page typesets and numbers these equations, and the simulation under it solves them with the
        verified Rust solver compiled to WebAssembly.
      </p>
      <p>
        STRATO solves the problem two independent ways. Code B is a conservative finite-volume method with exact shell volumes and Backward
        Euler time stepping. Code C is a multilayer Sturm-Liouville eigenfunction series. Each checks the other.
      </p>

      <h2>What is implemented</h2>
      <ul className="prose-list">
        <li>
          Solved by v1.0: diffusion through the five layers at constant temperature, with a constant diffusivity per layer and the start
          inventory in the kernel. Released and verified on a synthetic case.
        </li>
        <li>
          Derived and specified, not implemented: radioactive decay, fission-product generation, inventory accounting,
          temperature-dependent diffusivity and temperature histories (Phase 2, v2.0).
        </li>
      </ul>
      <p>
        From web phase 2 each mechanism shows its research status on a five-level scale, assigned from the evidence in the repository and
        read from the research catalogue files.
      </p>
      <p>
        The bundled synthetic case uses radii and diffusivities chosen for verification. They are not measured TRISO properties, and no
        result from this case is a prediction of real fission-product release.
      </p>

      <h2 id="this-build">This build</h2>
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

      <h2>Labels used on this site</h2>
      <p>Every number, plot and particle colouring carries one of these labels. Each label is a word with its own outline or pattern.</p>
      <TableBox label="Result labels">
        <table className="table">
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

      <h2>Documents</h2>
      <p>
        The derivation and register documents are rendered on this site with the mathematics typeset; the Markdown files on GitHub are their
        source. See <a href={`${import.meta.env.BASE_URL}docs/index.html`}>all documents rendered on this site</a>.
      </p>
      <ul className="prose-list" data-testid="about-documents">
        {docs.map((d) => (
          <li key={d.path}>
            {d.page ? (
              <>
                <a href={`${import.meta.env.BASE_URL}${d.page}`}>{d.title}</a> (<a href={repoFile(d.path)}>Markdown source on GitHub</a>)
              </>
            ) : (
              <>
                <a href={repoFile(d.path)}>{d.title}</a> (on GitHub)
              </>
            )}
          </li>
        ))}
      </ul>

      <h2>Credits</h2>
      <p>
        The honesty devices of this demonstration (a status line that names what was computed, explicit labels for recorded results)
        follow Theodore&apos;s Outram Park dispersion demonstration (
        <a href="https://github.com/theodoreOnzGit/outram-park-backend">outram-park-backend</a>, AGPL-3.0-only). Its control column beside
        the view is not used here: this page puts the equation first and the simulation under it. No file of Outram Park is copied into
        this build.
      </p>
      <p>
        Typefaces: Literata, IBM Plex Sans and IBM Plex Mono, under the SIL Open Font License 1.1, served from this site through the
        Fontsource packages.
      </p>

      <h2>Licence and citation</h2>
      <p>
        STRATO is licensed under the GNU Affero General Public License, version 3 (AGPL-3.0-only). To cite it, use the metadata in{' '}
        <a href={repoFile('CITATION.cff')}>
          <code>CITATION.cff</code>
        </a>
        . Source code: <a href={REPO_URL}>github.com/weewzr/STRATO</a>.
      </p>
    </article>
  );
}
