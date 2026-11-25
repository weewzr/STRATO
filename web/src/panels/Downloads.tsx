import type { ReactNode } from 'react';
import { BUILD } from '../build';
import { findCase } from '../data/case';
import {
  comparisonColumns,
  csvText,
  exportMeta,
  jsonText,
  runProfileColumns,
  runTimeseriesColumns,
  TIMESERIES_UNITS,
  type Column,
} from '../downloads/serialise';
import { hrefFor } from '../route';
import type { CompareOk, RunOk } from '../solver/types';
import { comparisonKey, inputKey, useStrato } from '../state/store';
import { Badge } from './Badge';

// Downloads of the live results (web phase 5; prompt/DESIGN_SPECIFICATION.md Section 8.5): the current
// run, the baseline run and the comparison, each as CSV and JSON with a metadata header. The files are
// written in the browser from the wrapper's reply as it is (downloads/serialise.ts).

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

function columnUnits(cols: Column[]): Record<string, string> {
  return Object.fromEntries(cols.map((c) => [c.name, c.unit]));
}

interface Item {
  id: string;
  title: string;
  text: ReactNode;
  run: RunOk | null;
  files: { label: string; name: string; build: () => { text: string; type: string } }[];
  missing: ReactNode;
  stale?: boolean;
}

function DownloadItem({ item }: { item: Item }) {
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
              data-testid={`dl-${item.id}-${f.label.toLowerCase().replace(/[^a-z]+/g, '-')}`}
              onClick={() => {
                const { text, type } = f.build();
                save(f.name, text, type);
              }}
            >
              {f.label}
            </button>
          ))}
        </div>
      ) : null}
      {item.run && item.stale ? (
        <p className="downloads__text" data-testid={`dl-${item.id}-stale`}>
          These are the results of the last finished run; the inputs have changed since.
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
  const caseFile = findCase(caseId)?.provenance.input_file ?? '';
  const reply: CompareOk | null = comparison.reply;
  const compareStale = reply !== null && comparison.key !== comparisonKey(baselineConfig, config);
  const commit = (r: RunOk) => r.engine.commit;

  const runFiles = (r: RunOk, kind: string, stem: string, note: string) => [
    {
      label: 'Time series CSV',
      name: `${stem}-timeseries-${commit(r)}.csv`,
      build: () => {
        const cols = runTimeseriesColumns(r);
        return { text: csvText(exportMeta(`${kind}_timeseries`, r.label, r.engine, BUILD.commit, caseFile, r.inputs, note), cols), type: 'text/csv' };
      },
    },
    {
      label: 'Profiles CSV',
      name: `${stem}-profiles-${commit(r)}.csv`,
      build: () => {
        const cols = runProfileColumns(r);
        return { text: csvText(exportMeta(`${kind}_profiles`, r.label, r.engine, BUILD.commit, caseFile, r.inputs, note), cols), type: 'text/csv' };
      },
    },
    {
      label: 'Full reply JSON',
      name: `${stem}-${commit(r)}.json`,
      build: () => ({
        text: jsonText(exportMeta(kind, r.label, r.engine, BUILD.commit, caseFile, r.inputs, note), { ...TIMESERIES_UNITS, c_mol_per_m3: 'mol m^-3', cell_centroids_m: 'm', cell_faces_m: 'm', layer_inventory_mol: 'mol', layer_share: '1' }, r),
        type: 'application/json',
      }),
    },
  ];

  const items: Item[] = [
    {
      id: 'run',
      title: 'Current run',
      run,
      stale: run !== null && info?.inputKey !== inputKey(config),
      text: 'The run shown on Explore: time series of the finite-volume solution with the series (Code C) where it exists, the concentration in every cell at every output time, and the full reply of the solver.',
      missing: (
        <>
          No run yet. The solver runs the inputs on <a href={hrefFor('explore')}>Explore</a> as soon as it has loaded.
        </>
      ),
      files: run ? runFiles(run, 'run', 'strato-run', 'Current run of the synthetic verification case; not physical data.') : [],
    },
    {
      id: 'baseline',
      title: 'Baseline run',
      run: reply?.baseline ?? null,
      stale: compareStale,
      text: 'The baseline run of the last comparison, as compare_v1 returned it.',
      missing: (
        <>
          Appears when a comparison has run: open <a href={hrefFor('compare')}>Compare</a>.
        </>
      ),
      files: reply ? runFiles(reply.baseline, 'baseline_run', 'strato-baseline', 'Baseline run of the last comparison (compare_v1).') : [],
    },
    {
      id: 'comparison',
      title: 'Comparison',
      run: reply?.modified ?? null,
      stale: compareStale,
      text: 'Release fraction of the baseline and of the current inputs on their shared output times, and the difference current minus baseline as compare_v1 computed it in Rust.',
      missing: (
        <>
          Appears when a comparison has run: open <a href={hrefFor('compare')}>Compare</a>.
        </>
      ),
      files: reply
        ? [
            {
              label: 'Curves CSV',
              name: `strato-comparison-${reply.engine.commit}.csv`,
              build: () => {
                const cols = comparisonColumns(reply);
                return {
                  text: csvText(
                    exportMeta('comparison', reply.label, reply.engine, BUILD.commit, caseFile, { baseline: reply.baseline.inputs, current: reply.modified.inputs }, 'difference = current minus baseline, computed by compare_v1 in Rust.'),
                    cols,
                  ),
                  type: 'text/csv',
                };
              },
            },
            {
              label: 'Full reply JSON',
              name: `strato-comparison-${reply.engine.commit}.json`,
              build: () => ({
                text: jsonText(
                  exportMeta('comparison', reply.label, reply.engine, BUILD.commit, caseFile, { baseline: reply.baseline.inputs, current: reply.modified.inputs }, 'difference = current minus baseline, computed by compare_v1 in Rust.'),
                  columnUnits(comparisonColumns(reply)),
                  reply,
                ),
                type: 'application/json',
              }),
            },
          ]
        : [],
    },
  ];

  return (
    <ul className="downloads" data-testid="live-downloads">
      {items.map((it) => (
        <DownloadItem key={it.id} item={it} />
      ))}
    </ul>
  );
}
