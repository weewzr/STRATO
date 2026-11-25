// The "This build" table of the About page (Table 2) against the record of the web phases in
// prompt/PROGRESS.md: a phase recorded as done shows "Yes", any other phase shows "Not yet". The table
// had fallen behind twice (phases 4, 5 and 5.5 still read "Not yet" after they were delivered).
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { About } from '../src/sections/About';

/** Phase id (such as "4", "5.5", "6.1") to its status cell, read from the "Web demo phases" table. */
function recordedPhases(): Map<string, string> {
  // The tests run from web/ (npm test); import.meta.url is not a file URL under jsdom.
  const text = readFileSync(join(process.cwd(), '..', 'prompt', 'PROGRESS.md'), 'utf8').replace(/\r\n/g, '\n');
  const start = text.indexOf('\n## Web demo phases\n');
  expect(start, 'PROGRESS.md has a "Web demo phases" section').toBeGreaterThan(-1);
  const end = text.indexOf('\n## ', start + 1);
  const section = text.slice(start, end === -1 ? undefined : end);
  const phases = new Map<string, string>();
  for (const line of section.split('\n')) {
    const m = /^\| (\d+(?:\.\d+)?) [^|]*\| ([^|]*)\|/.exec(line);
    if (m?.[1] && m[2] !== undefined) phases.set(m[1], m[2].trim());
  }
  return phases;
}

/** Done when every recorded phase with this id, or with this id as its prefix ("6" for 6.1 and 6.2), is done. */
function isDone(phases: Map<string, string>, id: string): boolean | undefined {
  const matching = [...phases].filter(([p]) => p === id || p.startsWith(`${id}.`));
  if (matching.length === 0) return undefined;
  return matching.every(([, status]) => status.startsWith('done'));
}

describe('About page, contents of this build', () => {
  const phases = recordedPhases();

  it('reads the web phases from PROGRESS.md', () => {
    for (const id of ['1', '2', '3', '4', '5', '5.5', '6.1', '6.2', '7', '8']) expect(phases.has(id), `phase ${id}`).toBe(true);
  });

  it('shows "Yes" exactly for the phases recorded as done', () => {
    render(<About />);
    const rows = within(screen.getByTestId('build-table')).getAllByRole('row').slice(1);
    expect(rows.length).toBeGreaterThan(0);
    for (const row of rows) {
      const cells = row.querySelectorAll('th, td');
      const part = cells[0]?.textContent ?? '';
      const shown = cells[1]?.textContent ?? '';
      const phase = cells[2]?.textContent ?? '';
      const done = isDone(phases, phase);
      expect(done, `web phase ${phase} of "${part}" is recorded in PROGRESS.md`).not.toBeUndefined();
      expect(shown, `web phase ${phase}: "${part}"`).toBe(done ? 'Yes' : 'Not yet');
    }
  });
});
