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
 * the parity tolerance; with a scale it is the rule of tests/phase1_regression.rs.
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
