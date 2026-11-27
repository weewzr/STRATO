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
import { Basis } from '../src/sections/Basis';
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

describe('Basis page', () => {
  it('is concise and visual: the particle, the equation, the assumptions, the two methods and the benchmark, with no badge', () => {
    render(<Basis />);
    const page = screen.getByRole('article');
    expect(within(page).getAllByRole('heading', { level: 1 })).toHaveLength(1);
    expect(within(page).getByTestId('basis-particle').querySelector('svg')).toBeTruthy();
    const layers = within(screen.getByTestId('basis-layers')).getAllByRole('row').slice(1);
    expect(layers.map((r) => r.querySelector('th')?.textContent?.trim())).toEqual(['kernel', 'buffer', 'IPyC', 'SiC', 'OPyC']);
    expect(page.querySelector('[role="math"]')).toBeTruthy();
    expect(within(screen.getByTestId('basis-assumptions')).getAllByRole('row').length).toBeGreaterThan(3);
    expect(screen.getByTestId('basis-method-a').textContent).toContain('Method A: BTCS');
    expect(screen.getByTestId('basis-method-b').textContent).toContain('Method B: Sturm');
    expect(within(screen.getByTestId('basis-benchmark')).getAllByRole('row')).toHaveLength(6);
    expect(page.querySelector('.badge, .status-chip, [data-label], [data-status]')).toBeNull();
    expect((page.textContent ?? '').split(/\s+/).length).toBeLessThan(450);
  });
  it('no longer says that generation, decay, temperature-dependent diffusion or the comparison are missing', () => {
    render(<Documentation />);
    const text = document.body.textContent ?? '';
    expect(text).not.toContain('are not computed by this build');
    expect(text).not.toContain('Shown as previews with a label, not solved');
    expect(text).not.toContain('derived, not implemented');
    expect(text).not.toContain('which are derived but not yet implemented');
    expect(text).not.toContain('will appear under Validate');
    expect(text).not.toContain('No comparison is shown yet');
  });

  it('has its scientific statements in Scientific Documentation, once each', () => {
    for (const id of ['about-model', 'about-implemented', 'about-method-b-scope', 'about-methods', 'about-labels', 'about-build-contents', 'about-credits']) {
      expect(allItemIds(), id).toContain(id);
    }
    render(<Documentation />);
    const text = document.body.textContent ?? '';
    const once = (s: string) => expect(text.split(s).length - 1, s).toBe(1);
    once('the outer surface is a perfect sink and the initial inventory is uniform in the kernel');
    once('The release fraction is the share of the initial inventory that has left through the outer surface.');
    once('Produced earlier by the native STRATO binary');
    once('multilayer Sturm-Liouville eigenfunction series');
    once('SIL Open Font License 1.1');
    expect(screen.getByTestId('build-table')).toBeTruthy();
  });
});
