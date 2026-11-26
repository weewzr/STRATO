import { findCase } from '../data/case';
import { METHOD_1, METHOD_2 } from '../methods';
import { Badge } from '../panels/Badge';
import { LiveDownloads } from '../panels/Downloads';
import { ParameterProvenance, References } from '../panels/Provenance';
import { hrefDocumentation } from '../route';
import { useStrato } from '../state/store';
import { SectionHeading } from './common';

/** Precomputed native output of the synthetic case (web/public/data, written by strato-webtools precomputed). */
const PRECOMPUTED: { file: string; title: string }[] = [
  { file: 'btcs_run.timeseries.csv', title: 'Time series' },
  { file: 'btcs_run.profiles.csv', title: 'Profiles' },
  { file: 'btcs_vs_analytical.csv', title: `${METHOD_1} against ${METHOD_2}` },
  { file: 'btcs_independent_check.csv', title: 'Independent check' },
];

/**
 * The Data section (prompt/DESIGN_SPECIFICATION.md Sections 8.4 and 8.5): the parameters in use with
 * their class and source, the downloads of the live results and of the precomputed native output, and
 * the reference list. Descriptions of the files and the provenance of the case file are in Scientific
 * Documentation (documentation/data.tsx).
 */
export function Data() {
  const caseId = useStrato((s) => s.caseId);
  const entry = findCase(caseId);
  if (!entry) return null;
  const base = import.meta.env.BASE_URL;
  return (
    <article className="section" data-section="data">
      <SectionHeading>Data</SectionHeading>

      <ParameterProvenance entry={entry} />

      <h2>Downloads</h2>
      <LiveDownloads />
      <ul className="downloads">
        <li className="downloads__item" data-testid="dl-precomputed">
          <p className="downloads__title">
            Native output <Badge label="PRECOMPUTED" />
          </p>
          <div className="downloads__actions">
            {PRECOMPUTED.map((f) => (
              <a key={f.file} className="btn btn--small" href={`${base}data/${f.file}`} download data-testid={`download-${f.file}`}>
                {f.title} (CSV)
              </a>
            ))}
          </div>
        </li>
        <li className="downloads__item" data-testid="dl-inputs">
          <p className="downloads__title">Inputs and geometry</p>
          <div className="downloads__actions">
            <a className="btn btn--small" href={`${base}${entry.csvPath}`} download data-testid="download-case">
              Case inputs (CSV)
            </a>
            <a className="btn btn--small" href={`${base}data/agr1_reference_geometry.csv`} download data-testid="download-agr1-geometry">
              AGR-1 reference-particle geometry (CSV)
            </a>
          </div>
        </li>
      </ul>
      <p className="muted">
        <a href={hrefDocumentation('data-files')}>File contents and provenance</a>
      </p>

      <h2>References</h2>
      <References />
    </article>
  );
}
