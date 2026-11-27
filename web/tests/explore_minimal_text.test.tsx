// The Explore page under the minimal-text rules (decision 71; prompt/DESIGN_SPECIFICATION.md Section 12), in the
// states the content check of the built site does not render: the Numerical Interpretation with every item open,
// a run in the results, a scenario selected. No hash, no path, no registered disclaimer; one-sentence captions;
// every link into Scientific Documentation reaches an entry.
import { act, render } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeEach, describe, expect, it } from 'vitest';
import { allItemIds } from '../src/documentation';
import { BTCS_CAPABILITY } from '../src/equation/availability';
import { ITEM_EXPAND_IDS } from '../src/equation/numerics';
import { Explore } from '../src/sections/Explore';
import { documentationAnchorFromHash } from '../src/route';
import type { RunResponse } from '../src/solver/types';
import { initialState, inputKey, useStrato } from '../src/state/store';
import { CAPTION_MAX_CHARS, DISCLAIMERS, HASH_PATTERN, PATH_PATTERN } from '../scripts/minimal_text_rules.mjs';

const native = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'wrapper_native_synthetic.json'), 'utf8')) as { data: RunResponse };

/** The text of the page without the typeset formulas (KaTeX writes every symbol twice, once as MathML). */
function prose(root: HTMLElement): string {
  const copy = root.cloneNode(true) as HTMLElement;
  copy.querySelectorAll('.katex').forEach((k) => k.replaceWith(' '));
  return (copy.textContent ?? '').replace(/\s+/g, ' ');
}

function prepare(withRun: boolean) {
  act(() => {
    const s = useStrato.getState();
    s.setSolver({ state: 'ready', build: { abi: 3, crate: 'strato-wasm', wrapper_version: '0', strato_version: '2.0.0', commit: 'abc1234', dirty: false, wasm_bindgen: '0', btcs: { solver: native.data.solver, schema: 2, functions: [], export_functions: [], export_formats: [], capabilities: [BTCS_CAPABILITY], label: 'COMPUTED v2.0', limits: { max_total_intervals: 1, min_intervals_per_layer: 1, max_steps: 1, max_profile_values: 1, max_profile_times: 1, max_series_modes: 1 } } } });
    s.setCapabilities([BTCS_CAPABILITY]);
    s.setExpanded('numerical', true);
    for (const id of ITEM_EXPAND_IDS) s.setExpanded(id, true);
    if (withRun) s.setRunOutcome({ run: native.data, info: { elapsedMs: 3.2, requestId: 1, inputKey: inputKey(useStrato.getState().config), dropped: [] } });
  });
}

function check(page: HTMLElement) {
  const text = prose(page);
  expect(text.match(HASH_PATTERN) ?? [], 'build hash').toEqual([]);
  expect(text.match(PATH_PATTERN) ?? [], 'path or file name').toEqual([]);
  for (const d of DISCLAIMERS) expect(text.match(d.pattern) ?? [], `disclaimer ${d.id}`).toEqual([]);
  for (const c of Array.from(page.querySelectorAll('figcaption'))) {
    const t = prose(c as HTMLElement).trim();
    expect(t.length, t).toBeLessThanOrEqual(CAPTION_MAX_CHARS);
  }
  const anchors = Array.from(page.querySelectorAll<HTMLAnchorElement>('a[href^="#/documentation/"]')).map((a) => documentationAnchorFromHash(a.getAttribute('href') ?? ''));
  expect(anchors.length).toBeGreaterThan(0);
  for (const a of anchors) expect(allItemIds(), a).toContain(a);
}

beforeEach(() => {
  useStrato.setState({ ...initialState() });
});

describe('Explore under the minimal-text rules', () => {
  it('with a run and every item of the Numerical Interpretation open: no hash, path or registered disclaimer, one-sentence captions, working links', () => {
    prepare(true);
    const { container } = render(<Explore compact={false} onOpenControls={() => undefined} />);
    expect(container.querySelector('[data-testid="release-figure"] figcaption')).not.toBeNull();
    expect(container.querySelector('[data-testid="particle-figure"] figcaption')).not.toBeNull();
    check(container);
  });

  it('with a scenario selected (the combined model, not computed by this build): the same', () => {
    prepare(false);
    act(() => useStrato.getState().setSelection({ scenario: 'shutdown' }));
    const { container } = render(<Explore compact={false} onOpenControls={() => undefined} />);
    expect(container.querySelector('[data-testid="unsupported-simulation"]')).not.toBeNull();
    check(container);
  });

  it('names the active solver and the physics it computes in one line each, and keeps the explanation of each term', () => {
    prepare(true);
    const { container } = render(<Explore compact={false} onOpenControls={() => undefined} />);
    expect(container.querySelector('[data-testid="solver-status-line"]')?.textContent).toMatch(/^Method A: BTCS v2\.1\.0/);
    expect(container.querySelector('[data-testid="active-model"]')?.textContent).toBe(
      'Computes: Baseline diffusion. Not computed: fission-product generation, radioactive decay and temperature-dependent diffusion.',
    );
    const terms = container.querySelectorAll('[data-testid="term-meanings"] .geq__meaning');
    expect(terms.length).toBeGreaterThan(0);
    for (const t of Array.from(terms)) expect((t.textContent ?? '').match(/[.!?](\s|$)/g)?.length ?? 0, t.textContent ?? '').toBeLessThanOrEqual(2);
  });
});
