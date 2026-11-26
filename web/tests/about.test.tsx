// The "Contents of this build" table (Table 2; it moved from the About page to Scientific Documentation, section
// "Technical implementation and build") against the record of the web phases in prompt/PROGRESS.md: a phase
// recorded as done shows "Yes", any other phase shows "Not yet". The table had fallen behind twice (phases 4,
// 5 and 5.5 still read "Not yet" after they were delivered). The About page itself is checked at the end.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { allItemIds } from '../src/documentation';
import { BuildContentsTable } from '../src/documentation/about';
import { About } from '../src/sections/About';
import { Documentation } from '../src/sections/Documentation';

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

describe('Scientific Documentation, contents of this build', () => {
  const phases = recordedPhases();

  it('reads the web phases from PROGRESS.md', () => {
    for (const id of ['1', '2', '3', '4', '5', '5.5', '6.1', '6.2', '7', '8']) expect(phases.has(id), `phase ${id}`).toBe(true);
  });

  it('shows "Yes" exactly for the phases recorded as done', () => {
    render(<BuildContentsTable />);
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

// The document links of the former About page ("Documents") are the derivation links of Scientific
// Documentation (same six documents, same index link); documentation.test.tsx asserts them against the
// published set of crates/strato-docsite.

describe('About page', () => {
  it('is a short introduction with the links to Scientific Documentation, the repository and the citation metadata', () => {
    render(<About />);
    const page = screen.getByRole('article');
    expect(within(page).getAllByRole('heading', { level: 1 })).toHaveLength(1);
    expect(within(page).getByRole('link', { name: 'Scientific Documentation' }).getAttribute('href')).toBe('#/documentation');
    expect(within(page).getByRole('link', { name: 'Repository' }).getAttribute('href')).toMatch(/github\.com\/weewzr\/STRATO$/);
    expect(within(page).getByRole('link', { name: 'Citation metadata' }).getAttribute('href')).toMatch(/CITATION\.cff$/);
    // The four result labels stay as a legend; the page has no table and no build table.
    for (const label of ['COMPUTED v2.0', 'PRECOMPUTED', 'ILLUSTRATIVE', 'MEASURED']) expect(within(page).getByText(label)).toBeTruthy();
    expect(within(page).queryByRole('table')).toBeNull();
    expect((page.textContent ?? '').split(/\s+/).length).toBeLessThan(220);
  });

  it('has its scientific statements in Scientific Documentation, once each', () => {
    for (const id of ['about-model', 'about-implemented', 'about-method-b-scope', 'about-methods', 'about-labels', 'about-build-contents']) {
      expect(allItemIds(), id).toContain(id);
    }
    render(<Documentation />);
    const text = document.body.textContent ?? '';
    const once = (s: string) => expect(text.split(s).length - 1, s).toBe(1);
    once('the outer surface is a perfect sink and the initial inventory is uniform in the kernel');
    once('The release fraction is the share of the initial inventory that has left through the outer surface.');
    once('Produced earlier by the native STRATO binary');
    once('multilayer Sturm-Liouville eigenfunction series');
    expect(screen.getByTestId('build-table')).toBeTruthy();
  });
});
