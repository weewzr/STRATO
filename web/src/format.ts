// Display formatting only (unit and notation). No arithmetic on scientific values: rounding to a
// number of significant digits is done by the JavaScript number-to-text conversion.

/** Shortest text that reads back to the same double, in plain or scientific notation. */
export function plain(v: number): string {
  if (!Number.isFinite(v)) return String(v);
  const a = Math.abs(v);
  if (a !== 0 && (a < 0.001 || a >= 1000000)) return v.toExponential();
  return String(v);
}

/**
 * Splits v into mantissa and exponent text, for typesetting. Without `digits` the shortest form
 * that reads back to the same double; with `digits` that many significant digits.
 */
export function sciParts(v: number, digits?: number): { mantissa: string; exponent: string | null } {
  if (!Number.isFinite(v)) return { mantissa: String(v), exponent: null };
  const a = Math.abs(v);
  if (a === 0 || (a >= 0.001 && a < 1000000)) {
    if (digits === undefined || a === 0) return { mantissa: String(v), exponent: null };
    // toPrecision keeps trailing zeros (so a column of values lines up) but switches to exponent
    // form when the exponent reaches the digit count; then the rounded text is read back in plain form.
    const p = v.toPrecision(digits);
    return { mantissa: p.includes('e') ? String(Number(p)) : p, exponent: null };
  }
  const text = digits === undefined ? v.toExponential() : v.toExponential(Math.max(0, digits - 1));
  const [m = '', e = '0'] = text.split('e');
  return { mantissa: m, exponent: e.replace('+', '').replace('-', '−') };
}

/** Plain-text form for accessible names: "1 × 10^-12". */
export function sciText(v: number, digits?: number): string {
  const { mantissa, exponent } = sciParts(v, digits);
  return exponent === null ? mantissa : `${mantissa} × 10^${exponent.replace('−', '-')}`;
}

/** A count with thin-space digit groups, for readability (display only). */
export function count(n: number): string {
  return Number.isInteger(n) ? n.toLocaleString('en-GB').replace(/,/g, ' ') : String(n);
}

/** Clamp for a slider position (display only). */
export function clamp(v: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, v));
}
