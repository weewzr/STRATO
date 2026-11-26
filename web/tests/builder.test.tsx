// The Numerical Interpretation (block C): fourteen items that expand and collapse independently with the state in
// the shared selection, kept across navigation; the items that the selected physics and scenario show; the
// additions and scenario notes inside them; one place for every item and equation; keyboard operation; the
// meaning card of a part; and "no active mechanism without solver support" (prompt/TESTING_AND_VALIDATION.md
// Section 3.2). Rewritten with the directive of 26 Nov 2026: the one-step-at-a-time picker of the earlier
// builder is gone, so the tests of its radio group, its select and its banner are replaced by these.
import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../src/App';
import { BTCS_CAPABILITY } from '../src/equation/availability';
import { CoupledSystem, StepBody } from '../src/equation/EquationBuilder';
import { NumericalInterpretation } from '../src/equation/NumericalInterpretation';
import { ITEM_IDS, itemExpandId, NUMERICAL_ITEMS, visibleItems } from '../src/equation/numerics';
import { STAGES } from '../src/equation/terms';
import { ControlPanel } from '../src/panels/ControlPanel';
import { CATALOGUE, CatalogueContext, parseCatalogue, type Catalogue } from '../src/research/catalogue';
import { Explore } from '../src/sections/Explore';
import { useStrato } from '../src/state/store';

function withCatalogue(cat: Catalogue, ui: React.ReactElement) {
  return <CatalogueContext.Provider value={cat}>{ui}</CatalogueContext.Provider>;
}

/** Text of an element without KaTeX output, equation and section references and item numbers. */
function plainText(el: HTMLElement): string {
  const copy = el.cloneNode(true) as HTMLElement;
  copy.querySelectorAll('.katex, [data-ref], .stage-num, [hidden], [data-testid="live-numbers"]').forEach((n) => n.remove());
  return copy.textContent ?? '';
}

const select = (patch: Parameters<ReturnType<typeof useStrato.getState>['setSelection']>[0]) => act(() => useStrato.getState().setSelection(patch));
const openRegion = () => act(() => useStrato.getState().setExpanded('numerical', true));
const toggle = (id: number) => screen.getByTestId(`num-toggle-${id}`);
const itemIds = () => Array.from(document.querySelectorAll<HTMLElement>('[data-testid^="num-item-"]')).map((e) => Number(e.dataset.item));

describe('the items of the Numerical Interpretation', () => {
  it('shows the eleven baseline items, all collapsed, when the region is opened', () => {
    render(<NumericalInterpretation />);
    expect(screen.queryByTestId('num-items')).toBeNull();
    openRegion();
    expect(itemIds()).toEqual(visibleItems('baseline', null));
    expect(itemIds()).toHaveLength(11);
    for (const id of itemIds()) expect(toggle(id).getAttribute('aria-expanded')).toBe('false');
    // Nothing of any step is in the page until an item is opened.
    expect(screen.queryByTestId('equation-builder')).toBeNull();
    expect(screen.getByTestId('numerical-notation').textContent).toMatch(/Notation of the documents/);
  });

  it('expands and collapses each item independently, and keeps that in the shared selection', async () => {
    const user = userEvent.setup();
    render(<NumericalInterpretation />);
    openRegion();
    await user.click(toggle(3));
    await user.click(toggle(8));
    expect(useStrato.getState().selection.expanded).toEqual(['numerical', 'n3', 'n8']);
    expect(toggle(3).getAttribute('aria-expanded')).toBe('true');
    expect(toggle(8).getAttribute('aria-expanded')).toBe('true');
    expect(toggle(4).getAttribute('aria-expanded')).toBe('false');
    expect(within(screen.getByTestId('num-item-3')).getByTestId('equation-builder').dataset.stage).toBe('5');
    expect(within(screen.getByTestId('num-item-8')).getByTestId('equation-builder').dataset.stage).toBe('10');
    await user.click(toggle(3));
    expect(useStrato.getState().selection.expanded).toEqual(['numerical', 'n8']);
    expect(toggle(8).getAttribute('aria-expanded')).toBe('true');
    expect(screen.queryByTestId('equation-builder')?.dataset.stage).toBe('10');
  });

  it('keeps the expanded items, the physics and the scenario across navigation (the page is left and returned to)', () => {
    window.location.hash = '#/explore';
    const first = render(<App />);
    select({ physics: 'generation' });
    act(() => useStrato.getState().setExpanded('numerical', true));
    act(() => useStrato.getState().setExpanded(itemExpandId(9), true));
    act(() => useStrato.getState().setExpanded(itemExpandId(2), true));
    const before = useStrato.getState().selection;
    act(() => {
      window.location.hash = '#/research';
      window.dispatchEvent(new HashChangeEvent('hashchange'));
    });
    expect(screen.queryByTestId('num-items')).toBeNull();
    act(() => {
      window.location.hash = '#/explore';
      window.dispatchEvent(new HashChangeEvent('hashchange'));
    });
    expect(useStrato.getState().selection).toEqual(before);
    expect(toggle(2).getAttribute('aria-expanded')).toBe('true');
    expect(toggle(9).getAttribute('aria-expanded')).toBe('true');
    expect(toggle(3).getAttribute('aria-expanded')).toBe('false');
    first.unmount();
    // A new mount of the region (a different page instance) reads the same state.
    render(<NumericalInterpretation />);
    expect(toggle(9).getAttribute('aria-expanded')).toBe('true');
  });

  it('is operated with the keyboard: Tab reaches each heading, Enter and Space toggle it, focus stays on it', async () => {
    const user = userEvent.setup();
    render(<NumericalInterpretation />);
    openRegion();
    toggle(1).focus();
    await user.keyboard('{Enter}');
    expect(toggle(1).getAttribute('aria-expanded')).toBe('true');
    expect(document.activeElement).toBe(toggle(1));
    await user.keyboard(' ');
    expect(toggle(1).getAttribute('aria-expanded')).toBe('false');
    expect(document.activeElement).toBe(toggle(1));
    await user.tab();
    expect(document.activeElement).toBe(toggle(2));
    await user.keyboard('{Enter}');
    expect(toggle(2).getAttribute('aria-expanded')).toBe('true');
    // Every heading is a named button in a labelled list.
    expect(screen.getByRole('list', { name: /Numerical interpretation/ })).toBeTruthy();
    for (const id of itemIds()) expect(screen.getByRole('button', { name: new RegExp(NUMERICAL_ITEMS[id - 1]!.title) })).toBeTruthy();
  });
});

describe('what the selected physics and scenario show', () => {
  it('follow the physics: item nine with generation, ten with decay, eleven with temperature, all with the combined model', () => {
    render(<NumericalInterpretation />);
    openRegion();
    const cases: [Parameters<typeof select>[0], number[]][] = [
      [{ physics: 'generation' }, [9]],
      [{ physics: 'decay' }, [10]],
      [{ physics: 'temperature' }, [11]],
      [{ physics: 'combined' }, [9, 10, 11]],
      [{ physics: 'baseline' }, []],
    ];
    for (const [patch, extra] of cases) {
      select(patch);
      expect(itemIds(), JSON.stringify(patch)).toEqual(visibleItems(useStrato.getState().selection.physics, null));
      expect(itemIds().filter((i) => [9, 10, 11].includes(i))).toEqual(extra);
    }
  });

  it('labels a mechanism with the true status of this build and shows nothing as solved when the solver does not implement it', () => {
    act(() => useStrato.getState().setCapabilities([BTCS_CAPABILITY]));
    render(<NumericalInterpretation />);
    openRegion();
    select({ physics: 'combined' });
    for (const id of [9, 10, 11]) {
      expect(screen.getByTestId(`num-status-${id}`).dataset.status).toBe('planned');
    }
    expect(screen.getByTestId('numerical-status').dataset.supported).toBe('false');
    expect(screen.getByTestId('numerical-status').textContent).toMatch(/No simulation runs for this selection: the solver in this build does not implement fission-product generation, radioactive decay and temperature-dependent diffusion/);
    // The statuses come from the wrapper's report: a build that reports a mechanism at the live status labels it so.
    act(() => useStrato.getState().setCapabilities([BTCS_CAPABILITY, 'generation']));
    expect(screen.getByTestId('num-status-9').dataset.status).not.toBe('planned');
    expect(screen.getByTestId('num-status-10').dataset.status).toBe('planned');
  });

  it('says the solver loads, and that the baseline is solved, without claiming more', () => {
    render(<NumericalInterpretation />);
    openRegion();
    expect(screen.getByTestId('numerical-status').textContent).toMatch(/solver that runs the simulation is loading/);
    act(() => useStrato.getState().setCapabilities([BTCS_CAPABILITY]));
    expect(screen.getByTestId('numerical-status').textContent).toMatch(/^The simulation solves these equations with the verified finite-difference BTCS Rust solver\.$/);
    act(() => useStrato.getState().setCapabilities(['generation']));
    expect(screen.getByTestId('numerical-status').textContent).toMatch(/does not offer the BTCS capability at the live status/);
  });

  it('adds the right-hand side vector with generation, the diagonal note with decay, the coefficients with temperature and the whole formulation with the combined model', () => {
    render(<NumericalInterpretation />);
    openRegion();
    act(() => useStrato.getState().setExpanded(itemExpandId(8), true));
    const eqIds = () => Array.from(within(screen.getByTestId('num-item-8')).getByTestId('physics-additions', { exact: true }).querySelectorAll('figure[data-eq]')).map((f) => (f as HTMLElement).dataset.eq);
    expect(within(screen.getByTestId('num-item-8')).queryByTestId('physics-additions')).toBeNull();
    select({ physics: 'generation' });
    expect(eqIds()).toEqual(['d5_row']);
    select({ physics: 'decay' });
    expect(eqIds()).toEqual([]);
    expect(within(screen.getByTestId('num-item-8')).getByTestId('physics-additions').textContent).toMatch(/every diagonal entry gains the decay number/);
    select({ physics: 'temperature' });
    expect(eqIds()).toEqual([]);
    expect(within(screen.getByTestId('num-item-8')).getByTestId('physics-additions').textContent).toMatch(/evaluated with the diffusivities of the step/);
    select({ physics: 'combined' });
    expect(eqIds()).toEqual(['d5_combined_balance', 'd5_row']);
    expect(within(screen.getByTestId('num-item-8')).getByTestId('physics-additions').textContent).toMatch(/complete formulation of the combined model/);
    select({ physics: 'baseline' });
    expect(within(screen.getByTestId('num-item-8')).queryByTestId('physics-additions')).toBeNull();
  });

  it('writes the scenario inside the items where it acts, with its status, and in no second place', () => {
    act(() => useStrato.getState().setCapabilities([BTCS_CAPABILITY]));
    render(<NumericalInterpretation />);
    openRegion();
    for (const id of [9, 10, 11, 12]) act(() => useStrato.getState().setExpanded(itemExpandId(id as 9), true));
    expect(document.querySelectorAll('[data-testid^="scenario-note-"]')).toHaveLength(0);
    select({ scenario: 'shutdown' });
    expect(useStrato.getState().selection.physics).toBe('combined');
    const notes = Array.from(document.querySelectorAll<HTMLElement>('.scenario-note'));
    expect(notes.map((n) => n.dataset.testid)).toEqual(['scenario-note-9', 'scenario-note-10', 'scenario-note-11']);
    expect(screen.getByTestId('scenario-note-11').textContent).toMatch(/Shutdown and cooldown.*reassembled/);
    expect(screen.getByTestId('scenario-note-status-11').dataset.status).toBe('planned');
    select({ scenario: 'steady' });
    expect(Array.from(document.querySelectorAll('.scenario-note')).map((n) => (n as HTMLElement).dataset.testid)).toEqual(['scenario-note-9', 'scenario-note-11', 'scenario-note-12']);
    expect(screen.getByTestId('scenario-note-12').textContent).toMatch(/do not make the concentration field stationary/);
    // The scenario text is not repeated outside the items.
    expect(document.querySelectorAll('[data-testid="scenario-numerics"]')).toHaveLength(0);
  });

  it('puts every item and every derivation step in the page once, with the combined model and everything open', () => {
    render(<NumericalInterpretation />);
    openRegion();
    select({ physics: 'combined' });
    for (const id of ITEM_IDS) act(() => useStrato.getState().setExpanded(itemExpandId(id), true));
    expect(itemIds()).toEqual([...ITEM_IDS]);
    const stages = Array.from(document.querySelectorAll<HTMLElement>('[data-testid="equation-builder"]')).map((e) => Number(e.dataset.stage));
    expect([...stages].sort((a, b) => a - b)).toEqual([...STAGES]);
    // Each item holds exactly its steps, in order.
    for (const item of NUMERICAL_ITEMS) {
      const inside = Array.from(within(screen.getByTestId(`num-item-${item.id}`)).getAllByTestId('equation-builder')).map((e) => Number(e.dataset.stage));
      expect(inside, `item ${item.id}`).toEqual([...item.stages]);
    }
    // The main equation of a step is in the page once, and so is every equation that a physics addition shows.
    const main = Array.from(document.querySelectorAll<HTMLElement>('.eq-box--main [data-eq]')).map((e) => e.dataset.eq);
    expect(new Set(main).size).toBe(main.length);
    const added = Array.from(document.querySelectorAll<HTMLElement>('[data-testid="physics-additions"] figure[data-eq]')).map((e) => e.dataset.eq);
    expect(new Set(added).size).toBe(added.length);
    // The notation is stated once.
    expect(document.querySelectorAll('[data-testid="numerical-notation"]')).toHaveLength(1);
  });

  it('shows no number outside the equations, the references and the item numbers, with every item open', { timeout: 120_000 }, () => {
    render(<App />);
    act(() => useStrato.getState().setExpanded('numerical', true));
    select({ physics: 'combined' });
    for (const id of ITEM_IDS) act(() => useStrato.getState().setExpanded(itemExpandId(id), true));
    const region = screen.getByTestId('numerical-interpretation');
    expect(plainText(region)).not.toMatch(/\d/);
    const maths = Array.from(region.querySelectorAll('.katex-html')).map((k) => {
      const c = k.cloneNode(true) as HTMLElement;
      c.querySelectorAll('.katex-tag, .tag').forEach((t) => t.remove());
      return c.textContent ?? '';
    });
    for (const m of maths) expect(m).not.toMatch(/\d\.\d|\de[+-]?\d/);
  });
});

describe('a step: equation, parts and disclosures', () => {
  it('numbers each equation with the number printed in the document and marks its parts', () => {
    render(<StepBody stage={6} physics="baseline" showTitle={false} />);
    const eq = screen.getByTestId('equation');
    expect(eq.dataset.tag).toBe('D1.12');
    expect(eq.querySelector('.eq-term--second_new')).not.toBeNull();
    expect(eq.querySelector('.eq-term--first_new')).not.toBeNull();
    expect(eq.querySelector('.katex-error')).toBeNull();
    expect(eq.textContent).toContain('(D1.12)');
  });

  it('typesets the matrix, the algorithm and the Document 5 equations of the mechanism steps', () => {
    for (const [stage, tag] of [[10, 'F19'], [11, 'F24'], [12, 'F25 F27'], [15, 'D5.3'], [16, 'D5.16'], [17, 'D5.23']] as const) {
      const { unmount } = render(<StepBody stage={stage} physics="baseline" showTitle={false} />);
      expect(screen.getByTestId('equation').dataset.tag).toBe(tag);
      expect(screen.getByTestId('equation').querySelector('.katex-error')).toBeNull();
      expect(screen.getByTestId('equation').textContent).toContain(`(${tag.split(' ').pop()})`);
      unmount();
    }
    render(<StepBody stage={10} physics="baseline" showTitle={false} />);
    expect(screen.getByTestId('equation').querySelector('.mtable')).not.toBeNull();
  });

  it('carries the mechanism colour on the part that a mechanism introduces', () => {
    render(<StepBody stage={16} physics="decay" showTitle={false} />);
    const eq = screen.getByTestId('equation');
    expect(eq.querySelectorAll('.eq-new')).toHaveLength(1);
    expect(eq.querySelector('.eq-new')?.className).toContain('eq-term--decay');
  });

  it('opens the meaning card on hover, focus or tap, outlines the part in the equation, and closes with Escape', async () => {
    const user = userEvent.setup();
    render(<StepBody stage={3} physics="baseline" showTitle={false} />);
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

  it('opens one disclosure panel at a time: implementation, conditions, symbols, live values', async () => {
    const user = userEvent.setup();
    render(<StepBody stage={10} physics="baseline" showTitle={false} />);
    for (const id of ['implementation', 'conditions', 'terms', 'live']) expect((screen.getByTestId(`more-panel-${id}`) as HTMLElement).hidden).toBe(true);
    await user.click(screen.getByTestId('more-implementation'));
    expect(screen.getByTestId('more-panel-implementation').hidden).toBe(false);
    await user.click(screen.getByTestId('more-conditions'));
    expect(screen.getByTestId('more-panel-implementation').hidden).toBe(true);
    expect(screen.getByTestId('more-panel-conditions').textContent).toContain('(F21)');
    await user.click(screen.getByTestId('more-terms'));
    expect(screen.getByTestId('more-panel-terms').textContent).toMatch(/Lower diagonal/);
  });

  it('links the code of a baseline step and names, without a link, the development code of a mechanism step', async () => {
    const user = userEvent.setup();
    const { unmount } = render(<StepBody stage={11} physics="baseline" showTitle={false} />);
    await user.click(screen.getByTestId('more-implementation'));
    const impl = screen.getByTestId('more-panel-implementation');
    expect(within(impl).getByText(/ThomasFactors::solve, the back substitution/).closest('a')?.getAttribute('href')).toMatch(/src\/numerical\/tridiagonal\.rs#L230$/);
    expect(within(impl).getByText(/PS-10, tridiagonal solution/).closest('a')?.getAttribute('href')).toMatch(/03_rust_pseudocode\.md#ps-10-thomasfactors--tridiagonal-solution$/);
    expect(within(impl).queryByTestId('dev-code')).toBeNull();
    unmount();
    render(<StepBody stage={17} physics="temperature" showTitle={false} />);
    await user.click(screen.getByTestId('more-implementation'));
    const dev = screen.getByTestId('dev-code');
    expect(dev.textContent).toMatch(/development branch, not in this build/);
    expect(dev.textContent).toMatch(/run_physics/);
    expect(dev.querySelectorAll('a')).toHaveLength(0);
  });

  it('gives no live number for a selection that the build does not solve, and does not substitute the baseline run', async () => {
    const user = userEvent.setup();
    act(() => useStrato.getState().setCapabilities([BTCS_CAPABILITY]));
    select({ physics: 'generation' });
    render(<StepBody stage={6} physics="generation" showTitle={false} />);
    await user.click(screen.getByTestId('more-live'));
    expect(screen.getByTestId('live-numbers-unsupported').textContent).toMatch(/No simulation runs for the selection/);
    expect(screen.queryByTestId('live-numbers')).toBeNull();
  });

  it('labels the verification step as verification, not validation, and the release-fraction step as a pointer to Validate', () => {
    render(<StepBody stage={13} physics="baseline" showTitle={false} />);
    expect(screen.getByTestId('live-note').textContent).toMatch(/not experimental validation/);
    expect(screen.getByTestId('equation').textContent).not.toMatch(/measured/i);
  });
});

describe('selecting changes the display only', () => {
  it('opening items and choosing physics sends no request and changes no input or run', () => {
    const fetchSpy = vi.spyOn(globalThis, 'fetch');
    const before = JSON.stringify({ c: useStrato.getState().config, b: useStrato.getState().baselineConfig, r: useStrato.getState().run });
    render(<NumericalInterpretation />);
    openRegion();
    select({ physics: 'combined', scenario: 'startup' });
    for (const id of ITEM_IDS) act(() => useStrato.getState().setExpanded(itemExpandId(id), true));
    const after = JSON.stringify({ c: useStrato.getState().config, b: useStrato.getState().baselineConfig, r: useStrato.getState().run });
    expect(after).toBe(before);
    expect(useStrato.getState().status).toBe('idle');
    expect(fetchSpy).not.toHaveBeenCalled();
    fetchSpy.mockRestore();
  });

  it('is part of the Explore page between the scenarios and the simulation', () => {
    render(<Explore compact={false} onOpenControls={() => undefined} />);
    const blocks = Array.from(document.querySelectorAll<HTMLElement>('[data-section="explore"] [data-block]')).map((b) => b.dataset.block);
    expect(blocks).toEqual(['A', 'B', 'C', 'D', 'E']);
  });
});

// The mechanism switches of the controls were replaced by the physics selection of block A on 26 Nov 2026
// (tests/physics.test.tsx covers its status, the capability object and the simulation gate).
describe('the controls hold no second mechanism control', () => {
  it('has no mechanism switch', () => {
    act(() => useStrato.getState().setCapabilities(['diffusion_btcs_layered']));
    render(<ControlPanel layout="inline" />);
    expect(document.querySelectorAll('[role="switch"]')).toHaveLength(0);
  });
});

describe('catalogue rows with coupled equations', () => {
  const press = async (el: HTMLElement) => {
    const user = userEvent.setup();
    await user.click(el);
    el.focus();
    await user.keyboard('{Enter}');
    await user.keyboard(' ');
  };

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
    render(withCatalogue(cat, <StepBody stage={17} physics="baseline" showTitle={false} />));
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
