// Reading back the data files of a run (web phase 5; prompt/DESIGN_SPECIFICATION.md Section 8.5).
// The files are written by the library's export in Rust (the wrapper's export_run): a header of
// `# key: value` lines (solver version, commit, input digest, parameter source and class), the row of
// column names that carry the full physical name with the unit, and the rows. The page saves that text
// unchanged; it formats no number. This module only reads such a file back, for the tests and the
// end-to-end download check.

export interface ParsedCsv {
  meta: Record<string, string>;
  columns: Record<string, (number | null)[]>;
  names: string[];
}

/** Reads a CSV written by the library's export back (tests and the end-to-end download check). */
export function parseCsv(text: string): ParsedCsv {
  const meta: Record<string, string> = {};
  const lines = text.split('\n');
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
