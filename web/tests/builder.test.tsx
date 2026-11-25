// The progressive equation builder (block A) and the mechanism switches of block C:
// step picking, the banner, the part card, no numbers in block A, and
// "no active toggle without solver support" (prompt/TESTING_AND_VALIDATION.md Section 3.2).
import { act, fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../src/App';
import { BTCS_CAPABILITY } from '../src/equation/availability';
import { CoupledSystem, EquationBuilder } from '../src/equation/EquationBuilder';
import { CORE_STAGES, PREVIEW_STAGES, STAGES, STEP_DEFS, type Stage } from '../src/equation/terms';
import { ControlPanel } from '../src/panels/ControlPanel';
import { CATALOGUE, CatalogueContext, parseCatalogue, statusOf, type Catalogue } from '../src/research/catalogue';
import { useStrato } from '../src/state/store';
import { fixtureCatalogue } from './catalogueFixture';

function withCatalogue(cat: Catalogue, ui: React.ReactElement) {
  return <CatalogueContext.Provider value={cat}>{ui}</CatalogueContext.Provider>;
}

/** Text of an element without KaTeX output, equation and section references and step numbers. */
function plainText(el: HTMLElement): string {
  const copy = el.cloneNode(true) as HTMLElement;
  copy.querySelectorAll('.katex, [data-ref], .stage-num, .stage-select, [hidden]').forEach((n) => n.remove());
  return copy.textContent ?? '';
}

describe('step picker', () => {
  it('offers the fourteen steps and three previews as a radio group, one line each', () => {
    render(<EquationBuilder compact={false} />);
    const radios = screen.getAllByRole('radio');
    expect(radios).toHaveLength(17);
    expect(radios.map((r) => r.getAttribute('aria-checked'))).toEqual(['true', ...Array<string>(16).fill('false')]);
    // The research status of a preview is shown in its banner and its part list, from the catalogue.
    for (const s of STAGES) expect(screen.getByTestId(`stage-${s}`).querySelector('[data-status-for]')).toBeNull();
    act(() => screen.getByTestId('stage-16').click());
    expect(screen.getByTestId('preview-banner').querySelector<HTMLElement>('[data-status-for]')?.dataset.status).toBe(statusOf(CATALOGUE, 'decay'));
    expect(CORE_STAGES.every((s) => STEP_DEFS[s].mechanism === null)).toBe(true);
    expect(PREVIEW_STAGES.every((s) => STEP_DEFS[s].mechanism !== null)).toBe(true);
  });

  it('moves with the arrow keys and selects on the way (roving focus)', async () => {
    const user = userEvent.setup();
    render(<EquationBuilder compact={false} />);
    screen.getByTestId('stage-1').focus();
    await user.keyboard('{ArrowRight}{ArrowRight}');
    expect(useStrato.getState().stage).toBe(3);
    expect(document.activeElement).toBe(screen.getByTestId('stage-3'));
    await user.keyboard('{End}');
    expect(useStrato.getState().stage).toBe(17);
    await user.keyboard('{Home}{ArrowLeft}');
    expect(useStrato.getState().stage).toBe(17);
    expect(screen.getByTestId('stage-17').getAttribute('tabindex')).toBe('0');
    expect(screen.getByTestId('stage-1').getAttribute('tabindex')).toBe('-1');
  });

  it('is a select on phones, with the previews in their own group', () => {
    render(<EquationBuilder compact />);
    const select = screen.getByTestId('stage-select') as HTMLSelectElement;
    expect(select.querySelectorAll('optgroup')).toHaveLength(2);
    expect(select.querySelectorAll('option')).toHaveLength(17);
    fireEvent.change(select, { target: { value: '8' } });
    expect(useStrato.getState().stage).toBe(8);
    expect(screen.getByTestId('equation').dataset.tag).toBe('F10');
  });
});

describe('selecting a step changes the display only', () => {
  for (const s of [2, 5, 9, 12, 13, 14, 15, 16, 17] as Stage[]) {
    it(`step ${s}: unchanged inputs and run, no request`, () => {
      const fetchSpy = vi.spyOn(globalThis, 'fetch');
      const before = JSON.stringify({ c: useStrato.getState().config, b: useStrato.getState().baselineConfig, r: useStrato.getState().run });
      render(<EquationBuilder compact={false} />);
      act(() => screen.getByTestId(`stage-${s}`).click());
      expect(useStrato.getState().stage).toBe(s);
      const after = JSON.stringify({ c: useStrato.getState().config, b: useStrato.getState().baselineConfig, r: useStrato.getState().run });
      expect(after).toBe(before);
      expect(useStrato.getState().status).toBe('idle');
      expect(fetchSpy).not.toHaveBeenCalled();
      fetchSpy.mockRestore();
    });
  }

  it('says that the solver is loading before it reports its capabilities', () => {
    render(<EquationBuilder compact={false} />);
    expect(screen.queryByTestId('preview-banner')).toBeNull();
    expect(screen.getByTestId('live-note').textContent).toMatch(/solver that runs the simulation below is loading/);
  });

  it('does not call a BTCS equation live while the wrapper still runs the previous solver', () => {
    act(() => useStrato.getState().setCapabilities(['diffusion_layered']));
    render(<EquationBuilder compact={false} />);
    for (const s of [1, 6, 12]) {
      act(() => screen.getByTestId(`stage-${s}`).click());
      expect(screen.getByTestId('live-note').textContent).toMatch(/does not run the BTCS solver yet; the website switch is pending/);
      expect(screen.getByTestId('live-note').textContent).not.toMatch(/^Live:/);
    }
  });

  it('with the BTCS capability, steps one to twelve are live; the previews are not', () => {
    act(() => useStrato.getState().setCapabilities([BTCS_CAPABILITY]));
    render(<EquationBuilder compact={false} />);
    expect(screen.getByTestId('live-note').textContent).toBe('Live: the simulation below solves these equations with the verified BTCS Rust solver.');
    act(() => screen.getByTestId('stage-15').click());
    expect(screen.getByTestId('preview-banner').textContent).toMatch(/^Preview only: this mechanism is not solved\./);
  });

  it('labels step thirteen as verification, not validation, and step fourteen as a placeholder', () => {
    render(<EquationBuilder compact={false} />);
    act(() => screen.getByTestId('stage-13').click());
    expect(screen.getByTestId('live-note').textContent).toMatch(/not experimental validation/);
    act(() => screen.getByTestId('stage-14').click());
    expect(screen.getByTestId('preview-banner').textContent).toMatch(/^Placeholder: no experimental comparison is shown\./);
    expect(screen.getByTestId('equation').textContent).not.toMatch(/measured/i);
  });
});

describe('equation and parts', () => {
  it('numbers each equation with the number printed in the document and marks its parts', () => {
    render(<EquationBuilder compact={false} />);
    act(() => screen.getByTestId('stage-6').click());
    const eq = screen.getByTestId('equation');
    expect(eq.dataset.tag).toBe('D1.12');
    expect(eq.querySelector('.eq-term--second_new')).not.toBeNull();
    expect(eq.querySelector('.eq-term--first_new')).not.toBeNull();
    expect(eq.querySelector('.katex-error')).toBeNull();
    expect(eq.textContent).toContain('(D1.12)');
    act(() => screen.getByTestId('stage-12').click());
    expect(screen.getByTestId('equation').dataset.tag).toBe('F25 F27');
    expect(screen.getByTestId('equation').textContent).toContain('(F27)');
  });

  it('typesets the matrix of step ten and the algorithm of step eleven', () => {
    render(<EquationBuilder compact={false} />);
    act(() => screen.getByTestId('stage-10').click());
    expect(screen.getByTestId('equation').dataset.tag).toBe('F19');
    expect(screen.getByTestId('equation').querySelector('.katex-error')).toBeNull();
    expect(screen.getByTestId('equation').querySelector('.mtable')).not.toBeNull();
    act(() => screen.getByTestId('stage-11').click());
    expect(screen.getByTestId('equation').dataset.tag).toBe('F24');
    expect(screen.getByTestId('equation').querySelector('.katex-error')).toBeNull();
  });

  it('the labelled previews carry the mechanism colour on the placeholder', () => {
    render(<EquationBuilder compact={false} />);
    act(() => screen.getByTestId('stage-16').click());
    const eq = screen.getByTestId('equation');
    expect(eq.querySelectorAll('.eq-new')).toHaveLength(1);
    expect(eq.querySelector('.eq-new')?.className).toContain('eq-term--decay');
    expect(eq.textContent).toContain('G2 decay term');
  });

  it('opens the meaning card on hover, focus or tap, outlines the part in the equation, and closes with Escape', async () => {
    const user = userEvent.setup();
    render(<EquationBuilder compact={false} />);
    act(() => screen.getByTestId('stage-3').click());
    expect(screen.queryByTestId('term-card')).toBeNull();
    await user.hover(screen.getByTestId('term-diffusion'));
    expect(screen.getByTestId('term-card').dataset.term).toBe('diffusion');
    expect(screen.getByTestId('equation').dataset.activeTerm).toBe('diffusion');
    expect(screen.getByTestId('equation').querySelector('.eq-term--diffusion')?.classList.contains('is-active')).toBe(true);
    await user.unhover(screen.getByTestId('term-diffusion'));
    await user.click(screen.getByTestId('term-rate'));
    expect(screen.getByTestId('term-card').dataset.term).toBe('rate');
    expect(screen.getByTestId('term-rate').getAttribute('aria-expanded')).toBe('true');
    await user.keyboard('{Escape}');
    expect(screen.queryByTestId('term-card')).toBeNull();
    expect(screen.getByTestId('equation').querySelector('.is-active')).toBeNull();
  });

  it('opens one disclosure panel at a time: implementation, conditions, balance, live values', async () => {
    const user = userEvent.setup();
    render(<EquationBuilder compact={false} />);
    act(() => screen.getByTestId('stage-11').click());
    for (const id of ['implementation', 'conditions', 'balance', 'live']) expect((screen.getByTestId(`more-panel-${id}`) as HTMLElement).hidden).toBe(true);
    await user.click(screen.getByTestId('more-implementation'));
    const impl = screen.getByTestId('more-panel-implementation');
    expect(impl.hidden).toBe(false);
    expect(impl.textContent).toContain('solve_in_place');
    expect(within(impl).getByText(/solve_in_place, the tridiagonal solve/).closest('a')?.getAttribute('href')).toMatch(/src\/numerical\/tridiagonal\.rs#L79$/);
    expect(within(impl).getByText(/PS-10, tridiagonal solution/).closest('a')?.getAttribute('href')).toMatch(/03_rust_pseudocode\.md#ps-10-thomasfactors--tridiagonal-solution$/);
    await user.click(screen.getByTestId('more-conditions'));
    expect(impl.hidden).toBe(true);
    expect(screen.getByTestId('more-panel-conditions').textContent).toContain('(F21)');
    act(() => useStrato.getState().setStage(12));
    await user.click(screen.getByTestId('more-live'));
    expect(screen.getByTestId('more-panel-live').textContent).toMatch(/under Inventory balance in the live simulation below/);
    act(() => useStrato.getState().setStage(14));
    await user.click(screen.getByTestId('more-implementation'));
    expect(screen.getByTestId('more-panel-implementation').textContent).toMatch(/^Placeholder/);
  });

  it('block A shows no number apart from equation numbers, references and step numbers, at every step', { timeout: 120_000 }, () => {
    render(<App />);
    for (const s of STAGES) {
      act(() => useStrato.getState().setStage(s));
      const a = screen.getByTestId('block-a');
      expect(plainText(a), `step ${s}`).not.toMatch(/\d/);
      // Inside the typeset maths: no decimal number outside the equation tags.
      const maths = Array.from(a.querySelectorAll('.katex-html')).map((k) => {
        const c = k.cloneNode(true) as HTMLElement;
        c.querySelectorAll('.katex-tag, .tag').forEach((t) => t.remove());
        return c.textContent ?? '';
      });
      for (const m of maths) expect(m, `step ${s}`).not.toMatch(/\d\.\d|\de[+-]?\d/);
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

  it('marks the switch that the selected preview stands for', () => {
    render(<ControlPanel layout="inline" />);
    act(() => useStrato.getState().setStage(16));
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
    act(() => useStrato.getState().setStage(17));
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
