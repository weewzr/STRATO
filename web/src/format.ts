// Display formatting only (unit and notation). No arithmetic on scientific values.

/** Shortest text that reads back to the same double, in plain or scientific notation. */
export function plain(v: number): string {
  if (!Number.isFinite(v)) return String(v);
  const a = Math.abs(v);
  if (a !== 0 && (a < 0.001 || a >= 1000000)) return v.toExponential();
  return String(v);
}

/** Splits the shortest scientific form of v into mantissa and exponent text, for typesetting. */
export function sciParts(v: number): { mantissa: string; exponent: string | null } {
  if (!Number.isFinite(v)) return { mantissa: String(v), exponent: null };
  const a = Math.abs(v);
  if (a === 0 || (a >= 0.001 && a < 1000000)) return { mantissa: String(v), exponent: null };
  const [m = '', e = '0'] = v.toExponential().split('e');
  return { mantissa: m, exponent: e.replace('+', '').replace('-', '−') };
}

/** Plain-text form for accessible names: "1 × 10^-12". */
export function sciText(v: number): string {
  const { mantissa, exponent } = sciParts(v);
  return exponent === null ? mantissa : `${mantissa} × 10^${exponent.replace('−', '-')}`;
}

/** Clamp for a slider position (display only). */
export function clamp(v: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, v));
}
