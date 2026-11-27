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
import { hrefDocumentation } from '../src/route';
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
    // The sentence on what no result of the case is stays once, in the About entry of what is implemented; this entry links to it.
    expect(entry('explore-case')).not.toContain('prediction of real fission-product release');
    expect(entry('about-implemented')).toContain('no result from this case is a prediction of real fission-product release');
    const link = document.querySelector<HTMLAnchorElement>('#doc-explore-case a[data-testid="explore-case-implemented-link"]');
    expect(link?.getAttribute('href')).toBe(hrefDocumentation('about-implemented'));
    expect(document.querySelector(`#doc-explore-case a[href="${hrefDocumentation('compare-limits')}"]`)).toBeNull();
    expect(entry('explore-scope')).toContain('Method A: BTCS implements the baseline spherical diffusion in the five layers');
    // Decision 78: the graphs are model output except the experimental comparison, whose model curves belong to temperature-dependent diffusion only.
    expect(entry('explore-results')).toContain('the experimental comparison, which draws registered measurements');
    expect(entry('explore-results')).toContain('under another model equation the measurements stand alone');
    expect(entry('explore-results')).not.toContain('none is available in this build');
    expect(entry('explore-scope')).toContain('stage 5');
    expect(entry('explore-scope')).not.toContain('not computed by this build');
    expect(entry('explore-scope')).toContain('never plotted in its place');
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

  it('names the manual solver of Scientific Documentation, never "below" or Explore, in the notes of the derivation', () => {
    render(<Documentation />);
    const text = entry('explore-numerical');
    expect(text).not.toMatch(/\bbelow\b/);
    expect(text).not.toContain('live simulation');
    expect(text).toContain('the manual solver of Scientific Documentation');
  });

  it('points only to pages that exist: the manual solver for the numbers, the comparison graph of Explore for measurements, no Validate page', () => {
    render(<Documentation />);
    const text = entry('explore-numerical');
    expect(text).not.toContain('version, commit and verification status');
    expect(text).toContain('its version and verification status are shown in the status line of the manual solver in Scientific Documentation');
    expect(text).toContain('the build is under Technical implementation and build');
    expect(text).toContain('the comparison with measured release is the experimental comparison graph of Explore.');
    expect(text).not.toContain('under Validate');
    expect(text).not.toContain('none is available in this build');
  });

  it('states the independent Model Equation and Operation, never that a scenario selects the combined model', () => {
    render(<Documentation />);
    const text = entry('explore-scenarios');
    expect(text).toContain('chosen independently');
    expect(text).not.toContain('selects the combined model');
    expect(text).not.toContain('clears the scenario');
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
