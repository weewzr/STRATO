// The data contract of the experimental-comparison graph (directive of 26 Nov 2026 on experimental
// data and R², Sections 5, 6 and 9; decisions 69 and 70(f)). THE one module that knows the shape of a
// comparison file: when the final record definition of the R² metric (stage-b/btcs/metric-r2.md) is
// verified, only this file changes. Until then the shape is the one the coordinator's task fixes:
// an R² record has the fields metric ("r2"), method ("A" or "B"), n, value (number or null), status,
// reason_code, caption_plain, label_latex, label_plain and source {citation_apa, link, location}.
//
// Every number in a comparison file, the curves, the observations and the R² values, is produced in
// Rust. Nothing in this folder computes a scientific quantity; the page selects, orders and displays.

export type MethodId = 'A' | 'B';

/** Where a value comes from: the full APA citation, the direct publication link and the exact place in the paper. */
export interface Source {
  citation_apa: string;
  link: string;
  location: string;
}

/** One whole-curve R² record of one method on one experimental curve (value null: R² unavailable). */
export interface R2Record {
  metric: 'r2';
  method: MethodId;
  /** Number of matched experimental observations. */
  n: number;
  value: number | null;
  status: string;
  reason_code: string | null;
  /** The reason in one sentence; shown below the graph where value is null. */
  caption_plain: string;
  label_latex: string;
  /** The label drawn in the method's colour where value is a number, exactly as Rust wrote it. */
  label_plain: string;
  source: Source;
}

/** A curve: time in the unit of the file's time_unit, cumulative release fraction (dimensionless). */
export interface Series {
  t: number[];
  f: number[];
}

/** Experimental observations of one kind. Upper bounds are kept apart from measurements. */
export interface Observations extends Series {
  kind: 'measurement' | 'upper_bound';
}

export interface ComparisonFile {
  schema: 1;
  /** Dataset identifier (the id of the experimental database). */
  id: string;
  nuclide: string;
  /** The experimental temperature, as written in the source and in the database. */
  temperature: { value: number; unit: string };
  /** Unit of the time axis, for example "h". The times of every series are in this unit. */
  time_unit: string;
  method_a: Series;
  /** Null where Method B does not apply to the experimental conditions. */
  method_b: Series | null;
  observations: Observations[];
  r2: R2Record[];
  /** The experimental publication that the observations come from. */
  source: Source;
}

const isObject = (x: unknown): x is Record<string, unknown> => typeof x === 'object' && x !== null && !Array.isArray(x);
const isText = (x: unknown): x is string => typeof x === 'string' && x.trim().length > 0;
const isNumber = (x: unknown): x is number => typeof x === 'number' && Number.isFinite(x);
/** Metrics the page never shows (directive: R² is the sole whole-curve agreement metric). */
const FORBIDDEN = /\b(RMSE|MAE|NRMSE|PASS|FAIL)\b/i;

function checkSource(x: unknown, where: string, errors: string[]): void {
  if (!isObject(x)) {
    errors.push(`${where}: missing`);
    return;
  }
  for (const k of ['citation_apa', 'link', 'location'] as const) if (!isText(x[k])) errors.push(`${where}.${k}: missing or empty`);
  if (isText(x.link) && !/^https?:\/\/\S+$/.test(x.link)) errors.push(`${where}.link: not an http(s) address`);
}

function checkSeries(x: unknown, where: string, errors: string[]): void {
  if (!isObject(x) || !Array.isArray(x.t) || !Array.isArray(x.f)) {
    errors.push(`${where}: needs the arrays t and f`);
    return;
  }
  if (x.t.length === 0 || x.t.length !== x.f.length) errors.push(`${where}: t and f must be non-empty and of equal length`);
  if (!x.t.every(isNumber) || !x.f.every(isNumber)) errors.push(`${where}: t and f must hold finite numbers`);
}

/** The reasons a value is not a comparison file; an empty list means it is one. */
export function comparisonErrors(x: unknown): string[] {
  const errors: string[] = [];
  if (!isObject(x)) return ['the file is not an object'];
  if (x.schema !== 1) errors.push('schema: must be 1');
  for (const k of ['id', 'nuclide', 'time_unit'] as const) if (!isText(x[k])) errors.push(`${k}: missing or empty`);
  const temp = x.temperature;
  if (!isObject(temp) || !isNumber(temp.value) || !isText(temp.unit)) errors.push('temperature: needs a number value and a text unit');
  checkSeries(x.method_a, 'method_a', errors);
  if (x.method_b !== null) checkSeries(x.method_b, 'method_b', errors);
  if (!Array.isArray(x.observations) || x.observations.length === 0) errors.push('observations: at least one series');
  else {
    x.observations.forEach((o: unknown, i: number) => {
      checkSeries(o, `observations[${i}]`, errors);
      if (!isObject(o) || (o.kind !== 'measurement' && o.kind !== 'upper_bound')) errors.push(`observations[${i}].kind: measurement or upper_bound`);
    });
  }
  checkSource(x.source, 'source', errors);
  if (!Array.isArray(x.r2)) errors.push('r2: must be an array of records');
  else {
    const seen = new Set<unknown>();
    x.r2.forEach((r: unknown, i: number) => {
      const w = `r2[${i}]`;
      if (!isObject(r)) {
        errors.push(`${w}: not an object`);
        return;
      }
      if (r.metric !== 'r2') errors.push(`${w}.metric: must be "r2" (no other metric is shown)`);
      if (r.method !== 'A' && r.method !== 'B') errors.push(`${w}.method: A or B`);
      if (seen.has(r.method)) errors.push(`${w}.method: two records for one method`);
      seen.add(r.method);
      if (r.method === 'B' && x.method_b === null) errors.push(`${w}: a Method B record where Method B does not apply`);
      if (!isNumber(r.n) || !Number.isInteger(r.n) || r.n < 0) errors.push(`${w}.n: a non-negative integer`);
      if (r.value !== null && !isNumber(r.value)) errors.push(`${w}.value: a number or null`);
      if (isNumber(r.value) && !isText(r.label_plain)) errors.push(`${w}.label_plain: needed where a value exists`);
      if (r.value === null && !isText(r.caption_plain)) errors.push(`${w}.caption_plain: needed where the value is null`);
      for (const k of ['status', 'label_latex'] as const) if (typeof r[k] !== 'string') errors.push(`${w}.${k}: text`);
      for (const k of ['caption_plain', 'label_plain'] as const) {
        if (typeof r[k] !== 'string') errors.push(`${w}.${k}: text`);
        else if (FORBIDDEN.test(r[k])) errors.push(`${w}.${k}: names a metric or verdict the page does not show`);
      }
      if (r.reason_code !== null && typeof r.reason_code !== 'string') errors.push(`${w}.reason_code: text or null`);
      checkSource(r.source, `${w}.source`, errors);
    });
  }
  return errors;
}

export function isComparisonFile(x: unknown): x is ComparisonFile {
  return comparisonErrors(x).length === 0;
}
