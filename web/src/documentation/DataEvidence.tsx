import { findCase } from '../data/case';
import { METHOD_1, METHOD_2 } from '../methods';
import { Badge } from '../panels/Badge';
import { LiveDownloads } from '../panels/Downloads';
import { ParameterProvenance, References } from '../panels/Provenance';
import { useStrato } from '../state/store';

/** Precomputed native output of the synthetic case (web/public/data, written by strato-webtools precomputed). */
const PRECOMPUTED: { file: string; title: string }[] = [
  { file: 'btcs_run.timeseries.csv', title: 'Time series' },
  { file: 'btcs_run.profiles.csv', title: 'Profiles' },
  { file: 'btcs_vs_analytical.csv', title: `${METHOD_1} against ${METHOD_2}` },
  { file: 'btcs_independent_check.csv', title: 'Independent check' },
];

/** The parameters in use with their class and source (moved from the Data page). */
export function ParametersEvidence() {
  const caseId = useStrato((s) => s.caseId);
  const entry = findCase(caseId);
  if (!entry) return null;
  return (
    <div data-testid="parameters-evidence">
      <ParameterProvenance entry={entry} />
    </div>
  );
}

/** The reference list of the Data page. */
export function ReferencesEvidence() {
  return (
    <div data-testid="references-evidence">
      <References />
    </div>
  );
}

/** The downloads of the live results and of the precomputed native output (moved from the Data page). */
export function DownloadsEvidence() {
  const caseId = useStrato((s) => s.caseId);
  const entry = findCase(caseId);
  if (!entry) return null;
  const base = import.meta.env.BASE_URL;
  return (
    <div data-testid="downloads-evidence">
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
    </div>
  );
}