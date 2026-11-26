import { REPO_URL, repoFile } from '../build';
import { METHOD_1, METHOD_2 } from '../methods';
import { Badge } from '../panels/Badge';
import { hrefDocumentation } from '../route';
import type { ResultLabel } from '../solver/types';
import { SectionHeading } from './common';

// The four result labels in a few words each; their full definitions are in Scientific Documentation.
const LABELS: { label: ResultLabel; short: string }[] = [
  { label: 'COMPUTED v2.0', short: `solved here by ${METHOD_1}` },
  { label: 'PRECOMPUTED', short: 'native program output' },
  { label: 'ILLUSTRATIVE', short: 'schematic, not solver output' },
  { label: 'MEASURED', short: 'experimental value' },
];

/** A short professional introduction to the project; everything else is in Scientific Documentation. */
export function About() {
  return (
    <article className="section section--doc about" data-section="about">
      <SectionHeading>About STRATO</SectionHeading>

      <p>
        STRATO (Spherical TRISO Analysis and Transport Operations) models fission-product diffusion through the five layers of a TRISO coated
        fuel particle: kernel, buffer, inner pyrolytic carbon (IPyC), silicon carbide (SiC) and outer pyrolytic carbon (OPyC). Two methods
        solve it and check each other: {METHOD_1} (finite differences) and {METHOD_2} (eigenfunction series). Written by weewzr and released
        as open source.
      </p>
      <p>In each layer the concentration obeys spherical diffusion with a constant diffusivity D of that layer:</p>
      <p className="eq-plain" role="math" aria-label="partial c over partial t equals one over r squared times partial over partial r of r squared D i partial c over partial r">
        ∂<i>c</i>/∂<i>t</i> = (1/<i>r</i>
        <sup>2</sup>) ∂/∂<i>r</i> (<i>r</i>
        <sup>2</sup> <i>D</i>
        <sub>i</sub> ∂<i>c</i>/∂<i>r</i>)
      </p>

      <ul className="about__labels" aria-label="Result labels">
        {LABELS.map((l) => (
          <li key={l.label}>
            <Badge label={l.label} /> <span className="muted">{l.short}</span>
          </li>
        ))}
      </ul>

      <nav className="about__links" aria-label="Project links">
        <a href={hrefDocumentation()}>Scientific Documentation</a>
        <a href={REPO_URL}>Repository</a>
        <a href={repoFile('CITATION.cff')}>Citation metadata</a>
      </nav>

      <p className="about__fine">
        Licence: GNU Affero General Public License, version 3 (AGPL-3.0-only). Credits: the status line and the explicit labels follow
        Theodore&apos;s <a href="https://github.com/theodoreOnzGit/outram-park-backend">Outram Park</a> dispersion demonstration (AGPL-3.0-only;
        no file of it is copied here). Typefaces Literata, IBM Plex Sans and IBM Plex Mono, SIL Open Font License 1.1.
      </p>
    </article>
  );
}
