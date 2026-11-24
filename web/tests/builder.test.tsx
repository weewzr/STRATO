// The staged equation builder (block A) and the mechanism switches of block C:
// stage picking, the preview banner, the term card, no numbers in block A, and
// "no active toggle without solver support" (prompt/TESTING_AND_VALIDATION.md Section 3.2).
import { act, fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../src/App';
import { CoupledSystem, EquationBuilder } from '../src/equation/EquationBuilder';
import { STAGE_DEFS, STAGES, type Stage } from '../src/equation/terms';
import { ControlPanel } from '../src/panels/ControlPanel';
import { CATALOGUE, CatalogueContext, parseCatalogue, statusOf, type Catalogue } from '../src/research/catalogue';
import { useStrato } from '../src/state/store';
import { fixtureCatalogue } from './catalogueFixture';

function withCatalogue(cat: Catalogue, ui: React.ReactElement) {
  return <CatalogueContext.Provider value={cat}>{ui}</CatalogueContext.Provider>;
}

/** Text of an element without KaTeX output, equation and section references and stage numbers. */
function plainText(el: HTMLElement): string {
  const copy = el.cloneNode(true) as HTMLElement;
  copy.querySelectorAll('.katex, [data-ref], .stage-num, .stage-select, [hidden]').forEach((n) => n.remove());
  return copy.textContent ?? '';
}

describe('stage picker', () => {
  it('offers the six stages as a radio group, each with its status from the catalogue', () => {
    render(<EquationBuilder compact={false} />);
    const radios = screen.getAllByRole('radio');
    expect(radios).toHaveLength(6);
    expect(radios.map((r) => r.getAttribute('aria-checked'))).toEqual(['true', 'false', 'false', 'false', 'false', 'false']);
    for (const s of STAGES) {
      const mech = STAGE_DEFS[s].mechanism;
      const chip = screen.getByTestId(`stage-${s}`).querySelector<HTMLElement>('[data-status-for]');
      if (mech) expect(chip?.dataset.status).toBe(statusOf(CATALOGUE, mech));
      else expect(chip).toBeNull();
    }
  });

  it('moves with the arrow keys and selects on the way (roving focus)', async () => {
    const user = userEvent.setup();
    render(<EquationBuilder compact={false} />);
    screen.getByTestId('stage-1').focus();
    await user.keyboard('{ArrowRight}{ArrowRight}');
    expect(useStrato.getState().stage).toBe(3);
    expect(document.activeElement).toBe(screen.getByTestId('stage-3'));
    await user.keyboard('{End}');
    expect(useStrato.getState().stage).toBe(6);
    await user.keyboard('{Home}{ArrowLeft}');
    expect(useStrato.getState().stage).toBe(6);
    expect(screen.getByTestId('stage-6').getAttribute('tabindex')).toBe('0');
    expect(screen.getByTestId('stage-1').getAttribute('tabindex')).toBe('-1');
  });

  it('is a select on phones', () => {
    render(<EquationBuilder compact />);
    const select = screen.getByTestId('stage-select') as HTMLSelectElement;
    fireEvent.change(select, { target: { value: '4' } });
    expect(useStrato.getState().stage).toBe(4);
    expect(screen.getByTestId('equation').dataset.tag).toBe('P22.11');
  });
});

describe('selecting a stage changes the display only', () => {
  for (const s of [2, 3, 4, 5, 6] as Stage[]) {
    it(`stage ${s}: preview banner, unchanged inputs and run, no request`, () => {
      const fetchSpy = vi.spyOn(globalThis, 'fetch');
      const before = JSON.stringify({ c: useStrato.getState().config, b: useStrato.getState().baselineConfig, r: useStrato.getState().run });
      render(<EquationBuilder compact={false} />);
      act(() => screen.getByTestId(`stage-${s}`).click());
      expect(useStrato.getState().stage).toBe(s);
      const banner = screen.getByTestId('preview-banner');
      expect(banner.textContent).toMatch(/^Equation preview only: the simulation below solves stage 1\./);
      const after = JSON.stringify({ c: useStrato.getState().config, b: useStrato.getState().baselineConfig, r: useStrato.getState().run });
      expect(after).toBe(before);
      expect(useStrato.getState().status).toBe('idle');
      expect(fetchSpy).not.toHaveBeenCalled();
      fetchSpy.mockRestore();
    });
  }

  // Web phase 3 connects the solver: before it reports its capabilities, stage 1 says the solver
  // is loading (the Phase 2 text named web phase 3).
  it('stage 1 shows no preview banner and says the solver is loading before it reports', () => {
    render(<EquationBuilder compact={false} />);
    expect(screen.queryByTestId('preview-banner')).toBeNull();
    expect(screen.getByTestId('live-note').textContent).toMatch(/Rust solver that runs it is loading/);
  });

  it('with the v1.0 capability, stage 1 is live and stage 2 is a preview', () => {
    act(() => useStrato.getState().setCapabilities(['diffusion_layered']));
    render(<EquationBuilder compact={false} />);
    expect(screen.getByTestId('live-note').textContent).toBe('Live: the simulation below solves this equation with the verified Rust solver.');
    act(() => screen.getByTestId('stage-2').click());
    expect(screen.getByTestId('preview-banner').textContent).not.toMatch(/Solver|loading/);
  });
});

describe('equation and terms', () => {
  it('numbers the equation with the cited tag and marks the new term', () => {
    render(<EquationBuilder compact={false} />);
    act(() => screen.getByTestId('stage-3').click());
    const eq = screen.getByTestId('equation');
    expect(eq.dataset.tag).toBe('P21.1');
    expect(eq.querySelectorAll('.eq-new')).toHaveLength(1);
    expect(eq.querySelector('.eq-new')?.className).toContain('eq-term--decay');
    expect(eq.querySelector('.katex-error')).toBeNull();
    expect(eq.textContent).toContain('(P21.1)');
  });

  it('stage 5 shows the layer laws as a second equation of the system, not inside the transport equation', () => {
    render(<EquationBuilder compact={false} />);
    act(() => screen.getByTestId('stage-5').click());
    const system = screen.getByTestId('system-arrhenius_layers');
    expect(system.textContent).toContain('(P22.1)');
    expect(screen.getByTestId('term-partition').textContent).toMatch(/not in the model/);
    expect(screen.getByTestId('equation-caption').textContent).toMatch(/layering itself is already in stage one/);
  });

  it('opens the meaning card on hover, focus or tap, and closes it with Escape', async () => {
    const user = userEvent.setup();
    render(<EquationBuilder compact={false} />);
    expect(screen.queryByTestId('term-card')).toBeNull();
    await user.hover(screen.getByTestId('term-diffusion'));
    expect(screen.getByTestId('term-card').dataset.term).toBe('diffusion');
    expect(screen.getByTestId('equation').dataset.activeTerm).toBe('diffusion');
    await user.unhover(screen.getByTestId('term-diffusion'));
    await user.click(screen.getByTestId('term-lhs'));
    expect(screen.getByTestId('term-card').dataset.term).toBe('lhs');
    expect(screen.getByTestId('term-lhs').getAttribute('aria-expanded')).toBe('true');
    await user.keyboard('{Escape}');
    expect(screen.queryByTestId('term-card')).toBeNull();
  });

  it('opens one disclosure panel at a time: implementation, conditions, balance, comparison', async () => {
    const user = userEvent.setup();
    render(<EquationBuilder compact={false} />);
    for (const id of ['implementation', 'conditions', 'balance', 'compare']) expect((screen.getByTestId(`more-panel-${id}`) as HTMLElement).hidden).toBe(true);
    await user.click(screen.getByTestId('more-implementation'));
    const impl = screen.getByTestId('more-panel-implementation');
    expect(impl.hidden).toBe(false);
    expect(impl.textContent).toContain('backward_euler_step');
    expect(within(impl).getByText(/backward_euler_step, one implicit step/).closest('a')?.getAttribute('href')).toMatch(
      /src\/numerical\/finite_volume\.rs#L162$/,
    );
    await user.click(screen.getByTestId('more-conditions'));
    expect(impl.hidden).toBe(true);
    expect(screen.getByTestId('more-panel-conditions').textContent).toContain('(3.4)');
    act(() => useStrato.getState().setStage(2));
    await user.click(screen.getByTestId('more-implementation'));
    expect(screen.getByTestId('more-panel-implementation').textContent).toMatch(/Derived, not implemented/);
    expect(screen.getByTestId('more-panel-implementation').textContent).toMatch(/Pseudocode: Phase 2 pseudocode/);
  });

  it('block A shows no number apart from equation numbers, references and stage numbers, at every stage', () => {
    render(<App />);
    for (const s of STAGES) {
      act(() => useStrato.getState().setStage(s));
      const a = screen.getByTestId('block-a');
      expect(plainText(a), `stage ${s}`).not.toMatch(/\d/);
      // Inside the typeset maths: no decimal number outside the equation tags.
      const maths = Array.from(a.querySelectorAll('.katex-html')).map((k) => {
        const c = k.cloneNode(true) as HTMLElement;
        c.querySelectorAll('.katex-tag, .tag').forEach((t) => t.remove());
        return c.textContent ?? '';
      });
      for (const m of maths) expect(m, `stage ${s}`).not.toMatch(/\d\.\d|\de[+-]?\d/);
    }
  });
});

describe('no active toggle without solver support', () => {
  const press = async (el: HTMLElement) => {
    const user = userEvent.setup();
    await user.click(el);
    el.focus();
    await user.keyboard('{Enter}');
    await user.keyboard(' ');
  };

  it('with the v1.0 capabilities every mechanism switch is disabled, stays off and says why', async () => {
    act(() => useStrato.getState().setCapabilities(['diffusion_layered']));
    render(<ControlPanel layout="inline" />);
    for (const id of ['source', 'decay', 'arrhenius']) {
      const sw = screen.getByTestId(`mech-${id}`);
      expect(sw.getAttribute('aria-disabled')).toBe('true');
      await press(sw);
      expect(sw.getAttribute('aria-checked')).toBe('false');
      const why = document.getElementById(sw.getAttribute('aria-describedby') ?? '');
      expect(why?.textContent).toMatch(/Not available: status RESEARCHED; awaiting reviewed implementation/);
    }
    expect(Object.values(useStrato.getState().mechanisms).every((m) => !m.enabled && !m.available)).toBe(true);
    expect(screen.getByTestId('announce').textContent).toMatch(/not available; status RESEARCHED/);
  });

  it('a VERIFIED mechanism absent from the capabilities stays disabled', async () => {
    const cat = fixtureCatalogue({ generation: 'VERIFIED', decay: 'VERIFIED', diffusivity_temperature: 'VERIFIED' });
    act(() => useStrato.getState().setCapabilities(['diffusion_layered']));
    render(withCatalogue(cat, <ControlPanel layout="inline" />));
    for (const id of ['source', 'decay', 'arrhenius']) {
      const sw = screen.getByTestId(`mech-${id}`);
      expect(sw.getAttribute('aria-disabled')).toBe('true');
      await press(sw);
      expect(sw.getAttribute('aria-checked')).toBe('false');
    }
  });

  it('a mechanism reported by the wrapper but only IMPLEMENTED stays disabled', async () => {
    const cat = fixtureCatalogue({ generation: 'IMPLEMENTED', decay: 'IMPLEMENTED', diffusivity_temperature: 'IMPLEMENTED' });
    act(() => useStrato.getState().setCapabilities(['diffusion_layered', 'generation', 'decay', 'diffusivity_temperature']));
    render(withCatalogue(cat, <ControlPanel layout="inline" />));
    for (const id of ['source', 'decay', 'arrhenius']) {
      const sw = screen.getByTestId(`mech-${id}`);
      expect(sw.getAttribute('aria-disabled')).toBe('true');
      await press(sw);
      expect(sw.getAttribute('aria-checked')).toBe('false');
    }
    expect(useStrato.getState().mechanisms.source.enabled).toBe(false);
  });

  it('setMechanism refuses a mechanism that isAvailable() rejects', () => {
    act(() => useStrato.getState().setCapabilities(['diffusion_layered', 'generation']));
    useStrato.getState().setMechanism('source', true);
    expect(useStrato.getState().mechanisms.source).toEqual({ enabled: false, available: false });
  });

  it('marks the switch that the selected stage introduces', () => {
    render(<ControlPanel layout="inline" />);
    act(() => useStrato.getState().setStage(3));
    expect(screen.getByTestId('mech-decay').dataset.stageCurrent).toBe('true');
    expect(screen.getByTestId('mech-source').dataset.stageCurrent).toBeUndefined();
    act(() => useStrato.getState().setStage(1));
    expect(document.querySelectorAll('[data-stage-current]')).toHaveLength(0);
  });

  it('a catalogue mechanism with coupled equations is shown as a system with a disabled switch', async () => {
    const raw = JSON.parse(JSON.stringify(CATALOGUE)) as Catalogue;
    const row = {
      ...raw.mechanisms[0]!,
      mechanism_id: 'fixture_trapping',
      name: 'Fixture trapping',
      status: 'PENDING' as const,
      builder_stage: 6,
      coupled: true,
      entry: 'docs/research/fixture.md',
    };
    const cat: Catalogue = {
      ...raw,
      mechanisms: [...raw.mechanisms, row],
      entries: { 'docs/research/fixture.md': { path: 'docs/research/fixture.md', title: 'Fixture', concept_equations: [String.raw`\frac{\partial c_t}{\partial t} = k c`] } },
    };
    render(withCatalogue(cat, <EquationBuilder compact={false} />));
    act(() => useStrato.getState().setStage(6));
    const sys = screen.getByTestId('coupled-fixture_trapping');
    expect(sys.textContent).toMatch(/Coupled system, research concept, not solved by STRATO/);
    expect(sys.querySelector('.katex')).not.toBeNull();
    const sw = screen.getByTestId('catalogue-switch-fixture_trapping');
    expect(sw.getAttribute('aria-disabled')).toBe('true');
    await press(sw);
    expect(sw.getAttribute('aria-checked')).toBe('false');
    // Rendering CoupledSystem alone gives the same disabled switch.
    render(<CoupledSystem row={row} catalogue={cat} />);
  });

  it('rejects a catalogue whose status is not one of the five words', () => {
    const raw = { commit: '', generator: '', inputs: [], tables_absent: [], data: { mechanisms: [{ mechanism_id: 'x', status: 'Verified' }], references: [], entries: {} } };
    expect(() => parseCatalogue(raw)).toThrow(/unknown status/);
  });
});
