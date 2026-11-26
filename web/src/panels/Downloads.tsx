import { useState, type ReactNode } from 'react';
import { findCase, type CaseConfig } from '../data/case';
import { METHOD_1 } from '../methods';
import { hrefFor } from '../route';
import { solverClient } from '../solver/session';
import type { CompareOk, ExportFormat, RunOk } from '../solver/types';
import { comparisonKey, configDifferences, inputKey, useStrato } from '../state/store';
import { Badge } from './Badge';

// Downloads of the live results (web phase 5; prompt/DESIGN_SPECIFICATION.md Section 8.5): the current
// run and the baseline run as CSV and JSON, and the comparison. The data files of a run are written by
// the library's export in Rust (the wrapper's export_run): each starts with a provenance header with
// the solver version and commit, the digest of the input, and the source and class of the parameters,
// and its columns carry the full physical name with the unit. The page saves the text unchanged.

function save(name: string, text: string, type: string): void {
  const blob = new Blob([text], { type });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = name;
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
}

const FORMATS: { format: Exclude<ExportFormat, 'comparison_csv'>; label: string; stem: string; ext: string }[] = [
  { format: 'timeseries_csv', label: 'Time series CSV', stem: 'timeseries', ext: 'csv' },
  { format: 'profiles_csv', label: 'Profiles CSV', stem: 'profiles', ext: 'csv' },
  { format: 'run_json', label: 'Run JSON', stem: 'run', ext: 'json' },
];

/** Source row and class of the parameters of an export (decision 31): the synthetic benchmark, or the user's changes to it. */
export function parameterProvenance(caseConfig: CaseConfig | undefined, config: CaseConfig): { source: string; parameterClass: string } {
  const same = caseConfig !== undefined && configDifferences(caseConfig, config).length === 0;
  return same
    ? { source: 'synthetic benchmark, not literature data', parameterClass: 'synthetic benchmark' }
    : { source: 'synthetic benchmark input with values changed by the user in the web controls', parameterClass: 'user-supplied value, no source class recorded' };
}

interface FileSpec {
  label: string;
  name: string;
  build: () => Promise<{ text: string; type: string }>;
}

interface Item {
  id: string;
  title: string;
  text: ReactNode;
  run: RunOk | null;
  files: FileSpec[];
  missing: ReactNode;
  stale?: boolean;
}

function DownloadItem({ item, onError }: { item: Item; onError: (message: string | null) => void }) {
  return (
    <li className="downloads__item" data-testid={`dl-${item.id}`} data-available={item.run ? 'true' : 'false'}>
      <p className="downloads__title">
        {item.title} {item.run ? <Badge label={item.run.label} /> : null}
      </p>
      <p className="downloads__text">{item.run ? item.text : item.missing}</p>
      {item.run ? (
        <div className="downloads__actions">
          {item.files.map((f) => (
            <button
              key={f.name}
              type="button"
              className="btn btn--small"
              disabled={item.stale === true}
              data-testid={`dl-${item.id}-${f.label.toLowerCase().replace(/[^a-z]+/g, '-')}`}
              onClick={() => {
                onError(null);
                f.build().then(
                  ({ text, type }) => save(f.name, text, type),
                  (e: unknown) => onError(e instanceof Error ? e.message : String(e)),
                );
              }}
            >
              {f.label}
            </button>
          ))}
        </div>
      ) : null}
      {item.run && item.stale ? (
        <p className="downloads__text" data-testid={`dl-${item.id}-stale`}>
          These are the results of the last finished run; the inputs have changed since, so the files can be written again when the new run has finished.
        </p>
      ) : null}
    </li>
  );
}

export function LiveDownloads() {
  const run = useStrato((s) => s.run);
  const info = useStrato((s) => s.runInfo);
  const config = useStrato((s) => s.config);
  const baselineConfig = useStrato((s) => s.baselineConfig);
  const comparison = useStrato((s) => s.comparison);
  const caseId = useStrato((s) => s.caseId);
  const entry = findCase(caseId);
  const caseFile = entry?.provenance.input_file ?? '';
  const [error, setError] = useState<string | null>(null);
  const reply: CompareOk | null = comparison.reply;
  const compareStale = reply !== null && comparison.key !== comparisonKey(baselineConfig, config);

  /** One data file of a run, written by the library's export in Rust and saved unchanged. */
  const exportFile = async (cfg: CaseConfig, format: ExportFormat, baseline?: CaseConfig): Promise<{ text: string; type: string }> => {
    const client = solverClient();
    if (!client) throw new Error('The solver is not available, so no file can be written.');
    const prov = parameterProvenance(entry?.config, cfg);
    const baseProv = baseline === undefined ? undefined : parameterProvenance(entry?.config, baseline);
    const out = await client.exportRun(
      cfg,
      format,
      { inputFile: caseFile, parameterSource: prov.source, parameterClass: prov.parameterClass },
      baseline,
      baseProv === undefined ? undefined : { parameterSource: baseProv.source, parameterClass: baseProv.parameterClass },
    );
    if (out.body.status !== 'ok') throw new Error(out.body.errors.join('; '));
    return { text: out.body.text, type: out.body.media_type };
  };

  const runFiles = (r: RunOk, cfg: CaseConfig, stem: string): FileSpec[] =>
    FORMATS.map((f) => ({
      label: f.label,
      name: `${stem}-${f.stem}-${r.engine.commit}.${f.ext}`,
      build: () => exportFile(cfg, f.format),
    }));

  const items: Item[] = [
    {
      id: 'run',
      title: 'Current run',
      run,
      stale: run !== null && info?.inputKey !== inputKey(config),
      text: `The run shown on Explore, written by the solver in Rust: the time series (inventory, release, mass-balance residual with its derived bound), the concentration at every node at every output time, and the JSON of the whole run, each file with the provenance header of ${METHOD_1}.`,
      missing: (
        <>
          No run yet. The solver runs the inputs on <a href={hrefFor('explore')}>Explore</a> as soon as it has loaded.
        </>
      ),
      files: run ? runFiles(run, config, 'strato-run') : [],
    },
    {
      id: 'baseline',
      title: 'Baseline run',
      run: reply?.baseline ?? null,
      stale: compareStale,
      text: 'The baseline run of the last comparison, written by the solver in the same three files.',
      missing: (
        <>
          Appears when a comparison has run: open <a href={hrefFor('compare')}>Compare</a>.
        </>
      ),
      files: reply ? runFiles(reply.baseline, baselineConfig, 'strato-baseline') : [],
    },
    {
      id: 'comparison',
      title: 'Comparison',
      run: reply?.modified ?? null,
      stale: compareStale,
      text: 'Both release fractions and their difference, the release fraction of the current inputs minus that of the baseline, at the output times both runs share, written by the solver in Rust as a CSV with the provenance header.',
      missing: (
        <>
          Appears when a comparison has run: open <a href={hrefFor('compare')}>Compare</a>.
        </>
      ),
      files: reply
        ? [
            {
              label: 'Comparison CSV',
              name: `strato-comparison-${reply.engine.commit}.csv`,
              build: () => exportFile(config, 'comparison_csv', baselineConfig),
            },
          ]
        : [],
    },
  ];

  return (
    <>
      <ul className="downloads" data-testid="live-downloads">
        {items.map((it) => (
          <DownloadItem key={it.id} item={it} onError={setError} />
        ))}
      </ul>
      {error ? (
        <p className="sim-alert" role="alert" data-testid="download-error">
          The file could not be written: {error}
        </p>
      ) : null}
    </>
  );
}
