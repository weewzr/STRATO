// MODEL EQUATION, OPERATION and the GOVERNING EQUATION of the final Explore layout (decisions 77 and 78): two
// independent groups of buttons, the progressive equation for each of the six buttons and each operation
// (structural TeX test: nothing is added by an operation), the legend of only the displayed variables, and
// the links of Future species-specific physics. Replaces the component tests of the earlier
// GoverningEquation and ScenarioPicker (tests/governing.test.tsx and tests/scenarios.test.tsx keep the pure
// tests of the TeX, the capability object and the scenario texts).
import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';
import { PHYSICS_IDS, SCENARIO_IDS } from '../src/equation/capability';
import { ExploreControls, ExploreEquation, FUTURE_LINKS } from '../src/equation/ExploreEquation';
import { legendRows } from '../src/equation/legend';
import { D_SCENARIO, equationPieces, equationTex, G_SCENARIO, physicsConfigAnnouncement } from '../src/equation/physics';
import { SCENARIO_LABEL } from '../src/equation/scenarios';
import { CATALOGUE } from '../src/research/catalogue';
import { initialState, useStrato } from '../src/state/store';

beforeEach(() => {
  useStrato.setState({ ...initialState() });
});

const radios = (name: string) => within(screen.getByRole('radiogroup', { name })).getAllByRole('radio');

describe('Select a Modelling Stage and Operation (Option B)', () => {
  it('renders four stage checkboxes (Baseline locked checked), Species-Specific Physics dropdown, and five Operation radio buttons', () => {
    render(<ExploreControls />);
    const baseline = screen.getByTestId('mech-baseline') as HTMLInputElement;
    const gen = screen.getByTestId('mech-generation') as HTMLInputElement;
    const decay = screen.getByTestId('mech-decay') as HTMLInputElement;
    const temp = screen.getByTestId('mech-temperature') as HTMLInputElement;
    expect(baseline.checked).toBe(true);
    expect(baseline.disabled).toBe(true);
    expect([gen.checked, decay.checked, temp.checked]).toEqual([false, false, false]);
    expect(screen.getByTestId('config-id-label').textContent).toContain('P01');

    const spSelect = screen.getByTestId('species-physics-select') as HTMLSelectElement;
    expect(Array.from(spSelect.options).map((o) => o.textContent)).toEqual(['None — Future development']);

    const operation = radios('Operation');
    expect(operation.map((r) => r.textContent)).toEqual(['Steady-state operation', 'Shutdown and cooldown', 'Startup', 'Control-rod insertion', 'Refuelling']);
    expect(operation.map((r) => r.textContent)).toEqual(SCENARIO_IDS.map((s) => SCENARIO_LABEL[s]));
    const a = screen.getByTestId('model-equation');
    const b = screen.getByTestId('operation');
    expect(a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(operation.map((r) => r.getAttribute('tabindex'))).toEqual(['0', '-1', '-1', '-1', '-1']);
    expect(radios('Operation')[0]?.getAttribute('aria-checked')).toBe('true');
  });

  it('toggles all P01–P08 mechanism checkboxes independently of the operation', async () => {
    const user = userEvent.setup();
    render(<ExploreControls />);
    for (const s of SCENARIO_IDS) {
      await user.click(screen.getByTestId(`operation-${s}`));
      expect(useStrato.getState().selection).toMatchObject({ physics: 'baseline', scenario: s });
    }
    await user.click(screen.getByTestId('mech-generation'));
    expect(screen.getByTestId('config-id-label').textContent).toContain('P02');
    await user.click(screen.getByTestId('mech-decay'));
    expect(screen.getByTestId('config-id-label').textContent).toContain('P05');
    await user.click(screen.getByTestId('mech-temperature'));
    expect(screen.getByTestId('config-id-label').textContent).toContain('P08');
    await user.click(screen.getByTestId('operation-startup'));
    expect(useStrato.getState().selection.scenario).toBe('startup');
    expect(useStrato.getState().selection.mechanisms).toEqual({ generation: true, decay: true, temperature: true });
    expect(document.body.textContent).not.toMatch(/requires it|is selected, because/);
  });

  it('is operated by the keyboard: Tab checks boxes, arrows and Home/End navigate Operation radios', async () => {
    const user = userEvent.setup();
    render(<ExploreControls />);
    screen.getByTestId('mech-generation').focus();
    await user.keyboard(' ');
    expect(useStrato.getState().selection.mechanisms.generation).toBe(true);
    screen.getByTestId('operation-steady').focus();
    await user.keyboard('{End}');
    expect(useStrato.getState().selection.scenario).toBe('refuelling');
    await user.keyboard('{Home}');
    expect(useStrato.getState().selection.scenario).toBe('steady');
    expect(useStrato.getState().selection.mechanisms.generation).toBe(true);
  });

  it('carries no status word or status badge on the controls', () => {
    render(<ExploreControls />);
    expect(document.body.textContent).not.toMatch(/\b(planned|verified|computed|implemented|precomputed|validated)\b/i);
    expect(document.querySelectorAll('[data-status], .bstatus, .badge')).toHaveLength(0);
  });
});

describe('the governing equation', () => {
  const pieceIds = () => Array.from(document.querySelectorAll('[data-piece]')).map((p) => (p as HTMLElement).dataset.piece);

  it('shows, for each of the six buttons, the equation of the model: base kept, +G and the decay term at the right, D(r,T) inside the operator', () => {
    render(<ExploreEquation />);
    const expected: Record<string, string[]> = {
      baseline: ['lhs', 'operator'],
      generation: ['lhs', 'operator', 'generation'],
      decay: ['lhs', 'operator', 'decay'],
      temperature: ['lhs', 'operator'],
      combined: ['lhs', 'operator', 'generation', 'decay'],
      future: ['lhs', 'operator', 'generation', 'decay'],
    };
    for (const p of PHYSICS_IDS) {
      act(() => useStrato.getState().setSelection({ physics: p }));
      expect(pieceIds(), p).toEqual(expected[p]);
      expect(screen.getByTestId('governing-equation').dataset.physics).toBe(p);
    }
    // Future species-specific physics shows the combined equation, unchanged.
    expect(equationTex('future')).toBe(equationTex('combined'));
  });

  it('shows an operation as G(r,t) and D(r,T(t)) where it prescribes a history, and adds no term', () => {
    render(<ExploreEquation />);
    for (const physics of PHYSICS_IDS) {
      for (const scenario of SCENARIO_IDS) {
        act(() => useStrato.getState().setSelection({ physics, scenario }));
        const own = equationPieces(physics, null).map((p) => p.id);
        expect(pieceIds(), `${physics}/${scenario}`).toEqual(own);
        const tex = equationTex(physics, scenario);
        const timed = scenario !== 'steady';
        const hasG = own.includes('generation');
        const hasD = physics === 'temperature' || physics === 'combined' || physics === 'future';
        expect(tex.includes(G_SCENARIO), `${physics}/${scenario} G`).toBe(timed && hasG);
        expect(tex.includes(D_SCENARIO), `${physics}/${scenario} D`).toBe(timed && hasD);
        // One plus and one minus at most: the number of terms of the model equation, never one more.
        expect((tex.match(/\+/g) ?? []).length).toBe(hasG ? 1 : 0);
        expect((tex.match(/-/g) ?? []).length).toBe(own.includes('decay') ? 1 : 0);
        expect(tex).not.toMatch(/rod|refuel|startup|shutdown/i);
      }
    }
  });

  it('highlights the new symbols subtly: the entering piece, the coefficient inside the operator, the symbols an operation changes', () => {
    render(<ExploreEquation />);
    expect(document.querySelectorAll('.geq-new, .geq-hi')).toHaveLength(0);
    act(() => useStrato.getState().setSelection({ physics: 'generation' }));
    expect(document.querySelector('[data-piece="generation"]')?.classList.contains('geq-new')).toBe(true);
    expect(document.querySelector('[data-piece="lhs"]')?.classList.contains('geq-new')).toBe(false);
    act(() => useStrato.getState().setSelection({ physics: 'temperature' }));
    expect(document.querySelector('[data-piece="operator"] .geq-hi')?.textContent).toContain('D(r,T)');
    act(() => useStrato.getState().setSelection({ physics: 'combined', scenario: 'startup' }));
    const hi = Array.from(document.querySelectorAll('.geq-hi--scenario')).map((e) => e.textContent ?? '');
    expect(hi.some((t) => t.includes('G(r,t)'))).toBe(true);
    expect(hi.some((t) => t.includes('D(r,T(t))'))).toBe(true);
  });

  it('announces a change of the configuration or operation in a polite live region, but not on the first render', () => {
    render(<ExploreEquation />);
    const live = screen.getByTestId('physics-announce');
    expect(live.getAttribute('aria-live')).toBe('polite');
    expect(live.textContent).toBe('');
    act(() => useStrato.getState().setSelection({ physics: 'generation' }));
    expect(live.textContent).toBe(physicsConfigAnnouncement({ generation: true, decay: false, temperature: false }, 'steady'));
    act(() => useStrato.getState().setSelection({ scenario: 'startup' }));
    expect(live.textContent).toBe(physicsConfigAnnouncement({ generation: true, decay: false, temperature: false }, 'startup'));
  });
});

describe('the legend under the equation', () => {
  const symbols = (physics: Parameters<typeof legendRows>[0], scenario: Parameters<typeof legendRows>[1] = null) => legendRows(physics, scenario).map((r) => r.id);

  it('lists only the variables of the displayed equation, with meaning and SI unit', () => {
    expect(symbols('baseline')).toEqual(['C', 't', 'r', 'D']);
    expect(symbols('generation')).toEqual(['C', 't', 'r', 'D', 'G']);
    expect(symbols('decay')).toEqual(['C', 't', 'r', 'D', 'lambda']);
    expect(symbols('temperature')).toEqual(['C', 't', 'r', 'D', 'T']);
    expect(symbols('combined')).toEqual(['C', 't', 'r', 'D', 'G', 'lambda', 'T']);
    expect(symbols('future')).toEqual(symbols('combined'));
    const byId = Object.fromEntries(legendRows('combined', null).map((r) => [r.id, r]));
    expect(byId.C?.meaning).toBe('Concentration');
    expect(byId.G?.meaning).toBe('Generation rate');
    expect(byId.lambda?.meaning).toBe('Radioactive decay constant');
    expect(byId.T?.meaning).toBe('Temperature');
    expect(byId.D?.unit).toContain('m^{2}/s');
    expect(byId.T?.unit).toBe(String.raw`\mathrm{K}`);
  });

  it('shows the symbols an operation changes, and no row the equation does not show', () => {
    expect(legendRows('baseline', 'startup').map((r) => r.symbol)).toEqual(['C', 't', 'r', 'D']);
    expect(legendRows('combined', 'startup').map((r) => r.symbol)).toEqual(['C', 't', 'r', 'D(r,T(t))', 'G(r,t)', String.raw`\lambda`, 'T(t)']);
    expect(legendRows('generation', 'shutdown').map((r) => r.symbol)).toEqual(['C', 't', 'r', 'D', 'G(r,t)']);
  });

  it('is a table under the equation with a row for every variable, for every selection', () => {
    render(<ExploreEquation />);
    for (const physics of PHYSICS_IDS) {
      act(() => useStrato.getState().setSelection({ physics }));
      const rows = within(screen.getByTestId('legend')).getAllByRole('row').slice(1);
      expect(rows.map((r) => (r as HTMLElement).dataset.legend)).toEqual(symbols(physics));
      expect(rows.every((r) => within(r as HTMLElement).getAllByRole('cell').length === 2)).toBe(true);
    }
    const eq = screen.getByTestId('equation-governing');
    expect(eq.compareDocumentPosition(screen.getByTestId('legend')) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });
});

describe('Future species-specific physics', () => {
  it('links to the four documented research mechanisms of the Research Catalogue, and only then', () => {
    render(<ExploreEquation />);
    expect(screen.queryByTestId('future-links')).toBeNull();
    act(() => useStrato.getState().setSelection({ physics: 'future' }));
    const links = within(screen.getByTestId('future-links')).getAllByRole('link');
    expect(links.map((a) => a.textContent)).toEqual(['Silver trapping and detrapping', 'Grain-boundary transport', 'Palladium-assisted SiC degradation', 'Barrier failure']);
    for (const a of links) expect(a.getAttribute('href')).toMatch(/^#\/research\//);
    // Every link names an entry that the catalogue documents.
    const entries = new Set(CATALOGUE.mechanisms.map((m) => m.entry));
    for (const l of FUTURE_LINKS) expect(entries.has(l.entry), l.entry).toBe(true);
  });
});
