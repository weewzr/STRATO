// Value comparison for the parity checks (browser self-test, end-to-end tests). It compares
// numbers that the solver returned with numbers from a reference file; it computes no physics
// and nothing it returns is shown as a result.

export interface ArrayComparison {
  name: string;
  /** Values compared (the shorter length when the lengths differ). */
  count: number;
  sameLength: boolean;
  /** Values outside the tolerance. */
  failures: number;
  /** First value outside the tolerance, for the report. */
  first: { index: number; got: number | null; want: number | null } | null;
  /** Values that are bit-for-bit equal. */
  identical: number;
  /** Largest |a - b| / max(|a|, |b|) over the compared values (0 when all are identical). */
  worstRelative: number;
  ok: boolean;
}

/**
 * |a - b| <= rtol * max(scale, |a|, |b|). With scale 0 this is the per-value relative rule of
 * the parity tolerance; with a scale it is the rule of tests/analytical_series_regression.rs.
 */
export function within(a: number | null, b: number | null, rtol: number, scale = 0): boolean {
  if (a === null || b === null) return a === b;
  if (Object.is(a, b) || a === b) return true;
  return Math.abs(a - b) <= rtol * Math.max(scale, Math.abs(a), Math.abs(b));
}

export function compareArrays(name: string, got: readonly (number | null)[], want: readonly (number | null)[], rtol: number, scale = 0): ArrayComparison {
  const n = Math.min(got.length, want.length);
  let failures = 0;
  let identical = 0;
  let worstRelative = 0;
  let first: ArrayComparison['first'] = null;
  for (let i = 0; i < n; i++) {
    const g = got[i] ?? null;
    const w = want[i] ?? null;
    if (Object.is(g, w)) identical++;
    else if (g !== null && w !== null) worstRelative = Math.max(worstRelative, Math.abs(g - w) / Math.max(Math.abs(g), Math.abs(w)));
    if (!within(g, w, rtol, scale)) {
      failures++;
      first ??= { index: i, got: g, want: w };
    }
  }
  const sameLength = got.length === want.length;
  return { name, count: n, sameLength, failures, first, identical, worstRelative, ok: sameLength && failures === 0 };
}

/** One leaf that the reply comparison found outside the rule, with its path (array positions as `[]`). */
export interface LeafFailure {
  pattern: string;
  count: number;
  first: { native: unknown; wasm: unknown };
}

export interface ReplyComparison {
  compared: number;
  identical: number;
  failures: LeafFailure[];
  ok: boolean;
}

/** Leaves of the independent series (and its differences), where the series' own accuracy floor adds an allowance. */
function seriesDerived(pattern: string): boolean {
  // A compare reply holds two runs, .baseline and .modified, each with its own series.
  const p = pattern.replace(/^\.(?:baseline|modified)/, '');
  return p.startsWith('.analytical.') || p.startsWith('.diagnostics.analytical_comparison');
}

/** The series accuracy floor of the native reply that a leaf belongs to: that of the run itself, or of the half of a compare reply the leaf lies in. */
function floorAt(native: unknown, path: string): number {
  const root = native as Record<string, unknown> | null;
  const half = path.startsWith('.baseline') ? root?.baseline : path.startsWith('.modified') ? root?.modified : root;
  const floor = (half as { analytical?: { accuracy_floor?: unknown } | null } | null | undefined)?.analytical?.accuracy_floor;
  return typeof floor === 'number' ? floor : 0;
}

/** A text without the provenance line that records the commit of the build: the two builds stamp their own. */
function withoutCommitLine(s: string): string {
  return s.includes('# commit:') ? s.split('\n').filter((l) => !l.startsWith('# commit:')).join('\n') : s;
}

/**
 * Compares two replies of the BTCS interface leaf by leaf, by the rule of the Rust example
 * `btcs_parity compare` (crates/strato-wasm/examples/btcs_parity.rs): |a - b| <= rtol max(|a|, |b|) for
 * every number, identical array lengths and object keys, equal strings, booleans and nulls. One stated
 * addition, for the values that come from the independent series: the accuracy floor of that case (read
 * from the native reply, for each run of a compare reply) is added as an absolute allowance, because the native and WebAssembly builds
 * use different exp, cos and sin routines (tolerances.rs, SERIES_FLOOR_RULE); `accuracy_floor` and
 * `min_flux_ratio` themselves and every BTCS value keep the strict rule. The `commit` fields, and the
 * `# commit:` line of an exported file, record the build that wrote them and are not compared.
 */
export function compareReplies(native: unknown, wasm: unknown, rtol: number): ReplyComparison {
  const out: ReplyComparison = { compared: 0, identical: 0, failures: [], ok: true };
  const byPattern = new Map<string, LeafFailure>();
  const fail = (path: string, a: unknown, b: unknown) => {
    const pattern = path.replace(/\[\d+\]/g, '[]');
    const f = byPattern.get(pattern) ?? { pattern, count: 0, first: { native: a, wasm: b } };
    f.count++;
    byPattern.set(pattern, f);
  };
  const walk = (path: string, a: unknown, b: unknown): void => {
    if (typeof a === 'number' && typeof b === 'number') {
      out.compared++;
      if (Object.is(a, b) || a === b) {
        out.identical++;
        return;
      }
      const pattern = path.replace(/\[\d+\]/g, '[]');
      const extra = seriesDerived(pattern) && !pattern.endsWith('.accuracy_floor') && !pattern.endsWith('.min_flux_ratio') ? floorAt(native, path) : 0;
      if (Math.abs(a - b) > rtol * Math.max(Math.abs(a), Math.abs(b)) + extra) fail(path, a, b);
      return;
    }
    if (Array.isArray(a) && Array.isArray(b)) {
      if (a.length !== b.length) fail(`${path}.length`, a.length, b.length);
      for (let i = 0; i < Math.min(a.length, b.length); i++) walk(`${path}[${i}]`, a[i], b[i]);
      return;
    }
    if (a !== null && b !== null && typeof a === 'object' && typeof b === 'object' && !Array.isArray(a) && !Array.isArray(b)) {
      const x = a as Record<string, unknown>;
      const y = b as Record<string, unknown>;
      for (const k of new Set([...Object.keys(x), ...Object.keys(y)])) {
        if (k === 'commit') continue;
        if (k in x && k in y) walk(`${path}.${k}`, x[k], y[k]);
        else fail(`${path}.${k} (key)`, k in x, k in y);
      }
      return;
    }
    out.compared++;
    const same = typeof a === 'string' && typeof b === 'string' ? withoutCommitLine(a) === withoutCommitLine(b) : a === b;
    if (same) out.identical++;
    else fail(path, a, b);
  };
  walk('', native, wasm);
  out.failures = [...byPattern.values()];
  out.ok = out.failures.length === 0;
  return out;
}