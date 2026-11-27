// Nothing scientific was lost when the Compare, Validate, Verify, Data and About sections left the application:
// every sentence, table, figure, citation and download below stood on one of those pages (or on a Research page
// as a status) and now stands, once, in Scientific Documentation. The table lists the texts that moved.
import { act, fireEvent, render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { allItemIds } from '../src/documentation';
import { Documentation } from '../src/sections/Documentation';

/** Where each text stood, the text, and whether it is plain text (T) or the test id of a component (C). */
const MOVED: { from: string; kind: 'T' | 'C'; what: string }[] = [
  // About
  { from: 'About', kind: 'T', what: 'the outer surface is a perfect sink and the initial inventory is uniform in the kernel' },
  { from: 'About', kind: 'T', what: 'The release fraction is the share of the initial inventory that has left through the outer surface.' },
  { from: 'About', kind: 'T', what: 'multilayer Sturm-Liouville eigenfunction series' },
  { from: 'About', kind: 'T', what: 'Produced earlier by the native STRATO binary' },
  { from: 'About', kind: 'T', what: 'GNU Affero General Public License, version 3' },
  { from: 'About', kind: 'C', what: 'result-labels' },
  { from: 'About', kind: 'C', what: 'build-table' },
  // Compare
  { from: 'Compare', kind: 'T', what: 'the comparison shows the effect of the inputs, not of new physics' },
  { from: 'Compare', kind: 'T', what: 'Runs with different output times are not compared' },
  { from: 'Compare', kind: 'C', what: 'comparison-evidence' },
  { from: 'Compare', kind: 'T', what: 'Fraction of the initial inventory released: baseline (grey) and current inputs (blue)' },
  { from: 'Compare', kind: 'C', what: 'no-diffs' },
  // Validate
  { from: 'Validate', kind: 'T', what: 'A constant-temperature result of Method A: BTCS is never compared with heating-test data' },
  { from: 'Validate', kind: 'T', what: 'It is a different question from verification' },
  { from: 'Validate', kind: 'T', what: 'R² is the only displayed agreement metric' },
  { from: 'Validate', kind: 'T', what: 'a one-sentence caption below the graph says why' },
  { from: 'Validate', kind: 'C', what: 'experimental-comparison-mount' },
  // Verify
  { from: 'Verify', kind: 'C', what: 'verification-evidence' },
  { from: 'Verify', kind: 'C', what: 'verify-status' },
  { from: 'Verify', kind: 'C', what: 'documentation-scope' },
  // Data
  { from: 'Data', kind: 'C', what: 'provenance-search' },
  { from: 'Data', kind: 'C', what: 'provenance' },
  { from: 'Data', kind: 'C', what: 'references-evidence' },
  { from: 'Data', kind: 'C', what: 'download-case' },
  { from: 'Data', kind: 'C', what: 'download-agr1-geometry' },
  { from: 'Data', kind: 'C', what: 'download-btcs_run.timeseries.csv' },
  { from: 'Data', kind: 'C', what: 'download-btcs_run.profiles.csv' },
  { from: 'Data', kind: 'C', what: 'download-btcs_vs_analytical.csv' },
  { from: 'Data', kind: 'C', what: 'download-btcs_independent_check.csv' },
  { from: 'Data', kind: 'C', what: 'dl-run' },
  { from: 'Data', kind: 'C', what: 'dl-comparison' },
  { from: 'Data', kind: 'T', what: 'Literature parameters for Phase 2' },
  // Research Catalogue (the status of each entry)
  { from: 'Research Catalogue', kind: 'C', what: 'status-table' },
  { from: 'Research Catalogue', kind: 'C', what: 'research-status-scale' },
  { from: 'Research Catalogue', kind: 'T', what: 'Nothing in the research catalogue is simulated.' },
];

async function openAll() {
  for (const d of Array.from(document.querySelectorAll<HTMLDetailsElement>('details.doc__item'))) {
    if (d.open) continue;
    await act(async () => {
      d.open = true;
      fireEvent(d, new Event('toggle'));
    });
  }
}

describe('what left the Compare, Validate, Verify, Data and About sections is in Scientific Documentation', () => {
  it('holds every moved text and component once', async () => {
    render(<Documentation />);
    await openAll();
    const body = document.body.textContent ?? '';
    for (const m of MOVED) {
      if (m.kind === 'T') {
        const n = body.split(m.what).length - 1;
        expect(n, `${m.from}: ${m.what}`).toBe(1);
      } else {
        expect(document.querySelectorAll(`[data-testid="${m.what}"]`).length, `${m.from}: ${m.what}`).toBe(1);
      }
    }
  });

  it('gives every moved block an anchor that the old addresses and the links use', () => {
    for (const id of ['evidence-comparison', 'validate-status', 'evidence-verification', 'evidence-parameters', 'evidence-references', 'evidence-downloads', 'statuses-research', 'about-credits', 'r2-requirements']) {
      expect(allItemIds(), id).toContain(id);
    }
  });
});
