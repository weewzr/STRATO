// Serialisation of the downloads of a live run, the comparison and the baseline run (web phase 5;
// prompt/DESIGN_SPECIFICATION.md Section 8.5). Each file starts with a metadata header: label, STRATO
// version, the solver build commit, the page build commit, the full inputs, the time of the download
// and the units of every column. The values are the wrapper's numbers written with the shortest text
// that reads back to the same double (String(v), and "-0" for negative zero), so a download
// reproduces the plotted values exactly. Nothing is computed here.
import type { CompareOk, EngineStamp, ResultLabel, RunInputs, RunOk } from '../solver/types';

export interface ExportMeta {
  kind: string;
  label: ResultLabel;
  source: string;
  strato_version: string;
  solver_commit: string;
  page_commit: string;
  case_file: string;
  created_utc: string;
  note: string;
  inputs: unknown;
}

export interface Column {
  name: string;
  unit: string;
  values: readonly (number | null)[];
}

/** Exact text of a double: String(v) is the shortest round-trip form; negative zero keeps its sign. */
export function numberText(v: number | null | undefined): string {
  if (v === null || v === undefined) return '';
  if (Object.is(v, -0)) return '-0';
  return String(v);
}

export function exportMeta(
  kind: string,
  label: ResultLabel,
  engine: EngineStamp,
  pageCommit: string,
  caseFile: string,
  inputs: unknown,
  note: string,
  now: Date = new Date(),
): ExportMeta {
  return {
    kind,
    label,
    source: `STRATO v${engine.strato_version} Rust solver (crates/strato-wasm) in WebAssembly, run in the browser`,
    strato_version: engine.strato_version,
    solver_commit: engine.commit,
    page_commit: pageCommit,
    case_file: caseFile,
    created_utc: now.toISOString().replace(/\.\d{3}Z$/, 'Z'),
    note,
    inputs,
  };
}

const META_KEYS: (keyof ExportMeta)[] = ['kind', 'label', 'source', 'strato_version', 'solver_commit', 'page_commit', 'case_file', 'created_utc', 'note'];

/** A CSV with '# key: value' header lines (the inputs as one line of JSON), a units line and the columns. */
export function csvText(meta: ExportMeta, columns: readonly Column[]): string {
  const lines: string[] = META_KEYS.map((k) => `# ${k}: ${String(meta[k])}`);
  lines.push(`# inputs: ${JSON.stringify(meta.inputs)}`);
  lines.push(`# units: ${columns.map((c) => `${c.name}=${c.unit}`).join('; ')}`);
  lines.push(columns.map((c) => c.name).join(','));
  const rows = Math.max(0, ...columns.map((c) => c.values.length));
  for (let i = 0; i < rows; i++) lines.push(columns.map((c) => numberText(c.values[i])).join(','));
  return `${lines.join('\n')}\n`;
}

/** A JSON document: the metadata header and the data, numbers written by JSON.stringify (round-trip exact). */
export function jsonText(meta: ExportMeta, units: Record<string, string>, data: unknown): string {
  return `${JSON.stringify({ ...meta, units, data }, null, 1)}\n`;
}

export interface ParsedCsv {
  meta: Record<string, string>;
  columns: Record<string, (number | null)[]>;
  names: string[];
}

/** Reads a CSV written by csvText back (tests and the end-to-end download check). */
export function parseCsv(text: string): ParsedCsv {
  const meta: Record<string, string> = {};
  const lines = text.split('\n');
  // csvText ends every line with a newline; an empty line before the end is a row of empty cells.
  if (lines[lines.length - 1] === '') lines.pop();
  let i = 0;
  for (; i < lines.length && lines[i]!.startsWith('# '); i++) {
    const line = lines[i]!.slice(2);
    const at = line.indexOf(': ');
    if (at > 0) meta[line.slice(0, at)] = line.slice(at + 2);
  }
  const names = (lines[i] ?? '').split(',');
  const columns: Record<string, (number | null)[]> = Object.fromEntries(names.map((n) => [n, []]));
  for (const line of lines.slice(i + 1)) {
    line.split(',').forEach((cell, j) => {
      const n = names[j];
      if (n !== undefined) columns[n]!.push(cell === '' ? null : Number(cell));
    });
  }
  return { meta, columns, names };
}

/** The series value at each output time of the run, or null where the series omits that time (matching by equal time only). */
export function seriesAtRunTimes(run: RunOk): (number | null)[] {
  const a = run.analytical;
  if (!a) return run.times_s.map(() => null);
  const at = new Map<number, number>();
  a.times_s.forEach((t, i) => {
    const v = a.release_fraction[i];
    if (v !== undefined) at.set(t, v);
  });
  return run.times_s.map((t) => (at.has(t) ? (at.get(t) as number) : null));
}

export const TIMESERIES_UNITS: Record<string, string> = {
  time_s: 's',
  release_fraction_fv: '1',
  release_fraction_series: '1',
  inventory_mol: 'mol',
  cumulative_release_mol: 'mol',
  release_rate_mol_s: 'mol s^-1',
  balance_residual_mol: 'mol',
  min_concentration_mol_m3: 'mol m^-3',
};

/** Time series of a run: every column of finite_volume::run as returned, and the series (Code C) where it exists. */
export function runTimeseriesColumns(run: RunOk): Column[] {
  const u = TIMESERIES_UNITS;
  return [
    { name: 'time_s', unit: u.time_s!, values: run.times_s },
    { name: 'release_fraction_fv', unit: u.release_fraction_fv!, values: run.release_fraction },
    { name: 'release_fraction_series', unit: u.release_fraction_series!, values: seriesAtRunTimes(run) },
    { name: 'inventory_mol', unit: u.inventory_mol!, values: run.inventory_mol },
    { name: 'cumulative_release_mol', unit: u.cumulative_release_mol!, values: run.cumulative_release_mol },
    { name: 'release_rate_mol_s', unit: u.release_rate_mol_s!, values: run.release_rate_mol_s },
    { name: 'balance_residual_mol', unit: u.balance_residual_mol!, values: run.balance_residual_mol },
    { name: 'min_concentration_mol_m3', unit: u.min_concentration_mol_m3!, values: run.min_concentration_mol_m3 },
  ];
}

/** Concentration profiles of a run: one row per cell (index, region, centroid), one column per output time. */
export function runProfileColumns(run: RunOk): Column[] {
  const cols: Column[] = [
    { name: 'cell', unit: '1', values: run.mesh.cell_centroids_m.map((_, i) => i) },
    { name: 'region', unit: '1', values: run.mesh.region_of_cell },
    { name: 'centroid_m', unit: 'm', values: run.mesh.cell_centroids_m },
  ];
  for (const p of run.profiles) cols.push({ name: `c_mol_m3_at_t_s=${numberText(p.t_s)}`, unit: 'mol m^-3', values: p.c_mol_per_m3 });
  return cols;
}

/** Release curves of a comparison and the difference compare_v1 returned (modified minus baseline). */
export function comparisonColumns(reply: CompareOk): Column[] {
  return [
    { name: 'time_s', unit: 's', values: reply.difference.times_s },
    { name: 'release_fraction_baseline', unit: '1', values: reply.baseline.release_fraction },
    { name: 'release_fraction_current', unit: '1', values: reply.modified.release_fraction },
    { name: 'difference_current_minus_baseline', unit: '1', values: reply.difference.release_fraction },
  ];
}

/** The inputs of a run as the solver echoed them (output.directory is not part of a run). */
export function runInputs(run: RunOk): RunInputs {
  return run.inputs;
}
