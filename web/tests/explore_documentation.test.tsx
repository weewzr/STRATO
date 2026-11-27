// The scientific material that left the Explore page in its minimal-text pass (decision 71) is in
// Scientific Documentation, once: one entry per kind of material, each asserted at its new place.
import { act, render } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeEach, describe, expect, it } from 'vitest';
import { allItemIds } from '../src/documentation';
import { BTCS_CAPABILITY } from '../src/equation/availability';
import { SCENARIO_IDS } from '../src/equation/capability';
import { NO_SCENARIO_TEXT_FULL, SCENARIO_RULE_FULL, SCENARIOS } from '../src/equation/scenarios';
import { NUMERICAL_ITEMS } from '../src/equation/numerics';
import { STEP_DEFS } from '../src/equation/terms';
import reference from '../src/generated/agr1_reference.json';
import { Documentation } from '../src/sections/Documentation';
import type { RunResponse } from '../src/solver/types';
import { initialState, useStrato } from '../src/state/store';

const native = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'wrapper_native_synthetic.json'), 'utf8')) as { data: RunResponse };
const norm = (s: string) => s.replace(/[−‐]/g, '-').replace(/\s+/g, ' ').toLowerCase();

const IDS = ['explore-case', 'explore-scope', 'explore-scenarios', 'explore-numerical', 'explore-balance', 'explore-results', 'explore-figures', 'explore-controls', 'explore-traceability', 'explore-reference'];

function entry(id: string): string {
  const el = document.getElementById(`doc-${id}`);
  expect(el, id).not.toBeNull();
  return (el as HTMLElement).textContent ?? '';
}

beforeEach(() => {
  useStrato.setState({ ...initialState() });
});

describe('Scientific Documentation holds what left Explore', () => {
  it('has one entry per kind of material, each with an anchor', () => {
    const ids = allItemIds();
    for (const id of IDS) expect(ids, id).toContain(id);
    render(<Documentation />);
    for (const id of IDS) expect(entry(id).length, id).toBeGreaterThan(40);
  });

  it('states the case, the scope, the scenarios in full and the results scope', () => {
    act(() => {
      useStrato.getState().setSolver({ state: 'ready' });
      useStrato.getState().setCapabilities([BTCS_CAPABILITY]);
    });
    render(<Documentation />);
    expect(entry('explore-case')).toContain('not AGR-1 data');
    expect(entry('explore-case')).toContain('no result from this case is a prediction of real fission-product release');
    expect(entry('explore-scope')).toContain('Method A: BTCS implements the baseline spherical diffusion in the five layers');
    expect(entry('explore-results')).toContain('model output only; no experimental data are shown there');
    expect(entry('explore-results')).toContain('none is available in this build');
    const scenarios = entry('explore-scenarios');
    expect(scenarios).toContain(SCENARIO_RULE_FULL);
    expect(scenarios).toContain(NO_SCENARIO_TEXT_FULL);
    for (const id of SCENARIO_IDS) expect(scenarios, id).toContain(SCENARIOS[id].full.join(' '));
  });

  it('holds the identity and the notes of the balance as the wrapper returns them', () => {
    act(() => {
      useStrato.getState().setSolver({ state: 'ready' });
      useStrato.getState().setCapabilities([BTCS_CAPABILITY]);
    });
    render(<Documentation />);
    const text = norm(entry('explore-balance'));
    expect(text).toContain(norm(native.data.balance.statement).replace(/\.$/, ''));
    expect(text).toContain(norm(native.data.balance.generated_and_decayed_note));
    expect(text).toContain('a measurement of the browser, not a result');
  });

  it('does not state the no-source, no-decay balance before the build has reported what it computes', () => {
    render(<Documentation />);
    expect(norm(entry('explore-balance'))).not.toContain('no source and no decay');
  });

  it('names the live simulation on Explore, never "below", in the notes of the derivation', () => {
    render(<Documentation />);
    const text = entry('explore-numerical');
    expect(text).not.toMatch(/\bbelow\b/);
    expect(text).toContain('in the live simulation on Explore');
  });

  it('holds the details of the three figures and of the controls', () => {
    render(<Documentation />);
    const figures = entry('explore-figures');
    expect(figures).toContain('values at or below 10⁻⁶ of it take the lowest colour');
    expect(figures).toContain('The axis stays fixed during playback, so the frames compare');
    expect(figures).toContain('The pointwise difference of the two curves is not drawn on Explore');
    const controls = entry('explore-controls');
    expect(controls).toContain('the grid has one more node than intervals');
    expect(controls).toContain('The solver does not use it: the diffusivities are constant');
    expect(controls).toContain('uniform in the kernel, zero in the coating layers');
  });

  it('holds the geometry note of the reference particle for every register set, with the source file', () => {
    render(<Documentation />);
    const text = entry('explore-reference');
    expect(text).toContain('AGR-1 dimensions are never combined with its diffusivities');
    for (const s of reference.data.sets) expect(text, s.set_id).toContain(s.set_id);
    expect(text).toContain(reference.input_file);
  });

  it('lists the code of every derivation step with its file (the page links name the function only)', () => {
    render(<Documentation />);
    const text = entry('explore-traceability');
    for (const it of NUMERICAL_ITEMS) {
      for (const st of it.stages) {
        for (const c of STEP_DEFS[st].implementation.code) {
          expect(text, `${c.label} ${c.path}`).toContain(c.label);
          expect(text, c.path).toContain(c.path);
        }
      }
    }
  });
});
