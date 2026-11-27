// The Explore page under the minimal-text rules (decision 71; prompt/DESIGN_SPECIFICATION.md Section 12), in the
// states the content check of the built site does not render: every section of the Numerical Interpretation open,
// each model equation and each operation selected. No hash, no path, no registered disclaimer; one-sentence
// captions. The manual baseline solver (AdvancedSolver) keeps the checks it had on the earlier Explore page
// (a run in the results, the one-line solver status and active model). Rewritten for the four-area layout.
import { act, render } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeEach, describe, expect, it } from 'vitest';
import { AdvancedSolver } from '../src/advanced/AdvancedSolver';
import { BTCS_CAPABILITY } from '../src/equation/availability';
import { PHYSICS_IDS, SCENARIO_IDS } from '../src/equation/capability';
import { SECTION_IDS } from '../src/equation/selectionIds';
import { Explore } from '../src/sections/Explore';
import type { RunResponse } from '../src/solver/types';
import { initialState, inputKey, useStrato } from '../src/state/store';
import { CAPTION_MAX_CHARS, DISCLAIMERS, HASH_PATTERN, PATH_PATTERN } from '../scripts/minimal_text_rules.mjs';

const native = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'wrapper_native_synthetic.json'), 'utf8')) as { data: RunResponse };

/** The text of the page without the typeset formulas and the pseudocode (KaTeX writes every symbol twice, once as MathML). */
function prose(root: HTMLElement): string {
  const copy = root.cloneNode(true) as HTMLElement;
  copy.querySelectorAll('.katex').forEach((k) => k.replaceWith(' '));
  return (copy.textContent ?? '').replace(/\s+/g, ' ');
}

function openEverything() {
  act(() => {
    const s = useStrato.getState();
    s.setCapabilities([BTCS_CAPABILITY]);
    for (const id of SECTION_IDS) s.setExpanded(id, true);
  });
}

function check(page: HTMLElement) {
  const copy = page.cloneNode(true) as HTMLElement;
  // The verbatim pseudocode of the document is code, not prose: the rules on prose do not apply to it.
  copy.querySelectorAll('.npseudo__code').forEach((n) => n.remove());
  const text = prose(copy);
  expect(text.match(HASH_PATTERN) ?? [], 'build hash').toEqual([]);
  expect(text.match(PATH_PATTERN) ?? [], 'path or file name').toEqual([]);
  for (const d of DISCLAIMERS) expect(text.match(d.pattern) ?? [], `disclaimer ${d.id}`).toEqual([]);
  for (const c of Array.from(page.querySelectorAll('figcaption'))) {
    if (c.classList.contains('npseudo__title')) continue;
    const t = prose(c as HTMLElement).trim();
    expect(t.length, t).toBeLessThanOrEqual(CAPTION_MAX_CHARS);
  }
}

beforeEach(() => {
  useStrato.setState({ ...initialState() });
});

describe('Explore under the minimal-text rules', () => {
  it('with every section of the Numerical Interpretation open, for every model equation and every operation: no hash, path or registered disclaimer, short captions', { timeout: 120_000 }, () => {
    openEverything();
    const { container } = render(<Explore />);
    for (const physics of PHYSICS_IDS) {
      for (const scenario of SCENARIO_IDS) {
        act(() => useStrato.getState().setSelection({ physics, scenario }));
        check(container);
      }
    }
  });

  it('shows no commentary beside the controls: the first prose is the legend, the sections are collapsed by default', () => {
    const { container } = render(<Explore />);
    expect(container.querySelectorAll('[data-testid^="num-section-"][data-open="true"]')).toHaveLength(0);
    expect(container.querySelectorAll('p').length).toBeLessThanOrEqual(2);
  });
});

describe('the manual baseline solver under the minimal-text rules', () => {
  it('with a run: no hash, path or registered disclaimer, one-sentence captions', () => {
    act(() => {
      const s = useStrato.getState();
      s.setCapabilities([BTCS_CAPABILITY]);
      s.setRunOutcome({ run: native.data, info: { elapsedMs: 3.2, requestId: 1, inputKey: inputKey(useStrato.getState().config), dropped: [] } });
    });
    const { container } = render(<AdvancedSolver compact={false} />);
    expect(container.querySelector('[data-testid="release-figure"] figcaption')).not.toBeNull();
    expect(container.querySelector('[data-testid="particle-figure"] figcaption')).not.toBeNull();
    check(container);
  });

  it('names the active solver and the physics it computes in one line each, whatever the Explore selection is', () => {
    act(() => {
      const s = useStrato.getState();
      s.setCapabilities([BTCS_CAPABILITY]);
      s.setSelection({ physics: 'combined', scenario: 'shutdown' });
      s.setSolver({ state: 'ready' });
      s.setRunOutcome({ run: native.data, info: { elapsedMs: 3.2, requestId: 1, inputKey: inputKey(useStrato.getState().config), dropped: [] } });
    });
    const { container } = render(<AdvancedSolver compact={false} />);
    expect(container.querySelector('[data-testid="active-model"]')?.textContent).toBe(
      'Computes: Baseline diffusion. Not computed: fission-product generation, radioactive decay and temperature-dependent diffusion.',
    );
    expect(container.querySelector('[data-testid="block-d"]')?.getAttribute('data-supported')).toBe('true');
  });
});
