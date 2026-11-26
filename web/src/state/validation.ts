// Reading a number typed into a control. The rules that decide whether a configuration is valid are
// the wrapper's (validate): its own messages are shown, and there is no second copy of them here.

/** Parses a number typed by the user. Accepts plain and scientific notation; rejects anything else. */
export function parseNumber(text: string): number | null {
  const s = text.trim().replace(/\s+/g, '').replace(/[×x]10\^?/i, 'e').replace('−', '-');
  if (!/^[+-]?(\d+\.?\d*|\.\d+)(e[+-]?\d+)?$/i.test(s)) return null;
  const v = Number(s);
  return Number.isFinite(v) ? v : null;
}