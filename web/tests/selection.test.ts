// The shared selection of the Explore page (final directive of 27 Nov 2026, decisions 77 and 78): the model
// equation and the operation are INDEPENDENT (every combination, no coupling), the choices of the graphs, the
// address round trip, survival across navigation, no browser storage. Rewritten from the version of 26 Nov
// 2026, which encoded the rule "a scenario selects the combined model".
import { act } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { configIdToFlags, flagsOf, PHYSICS_CONFIG_IDS, PHYSICS_IDS, SCENARIO_IDS } from '../src/equation/capability';
import { GRAPH_IDS, SECTION_IDS } from '../src/equation/selectionIds';
import { exploreHash, formatSelection, hasSelectionKeys, parseSelection, startAddressSync } from '../src/state/address';
import { DEFAULT_SELECTION, normaliseSelection, useStrato, type Selection } from '../src/state/store';

const goTo = (hash: string) => {
  window.location.hash = hash;
  window.dispatchEvent(new HashChangeEvent('hashchange'));
};

describe('selection state', () => {
  it('starts as baseline diffusion with steady-state operation, nothing expanded, the release graph', () => {
    expect(useStrato.getState().selection).toEqual(DEFAULT_SELECTION);
    expect(DEFAULT_SELECTION).toEqual({
      physics: 'baseline',
      mechanisms: { generation: false, decay: false, temperature: false },
      speciesPhysics: 'none',
      scenario: 'steady',
      expanded: [],
      graph: 'release',
      species: '',
      temperature: null,
      dataset: '',
      overlays: { methodA: true, methodB: true, experiment: true },
      controlledCompare: false,
    });
  });

  for (const physics of PHYSICS_IDS) {
    for (const scenario of SCENARIO_IDS) {
      for (const expanded of [[], ['method-a'], ['method-b', 'pseudocode']]) {
        it(`holds ${physics} / ${scenario} / ${expanded.join('+') || 'collapsed'} exactly as chosen and survives a round trip through the address`, () => {
          useStrato.getState().setSelection({ physics, scenario, expanded });
          const sel = useStrato.getState().selection;
          // The two groups are independent: nothing is changed to suit the other.
          expect(sel).toEqual({ ...DEFAULT_SELECTION, physics, mechanisms: flagsOf(physics), scenario, expanded });
          expect(parseSelection(formatSelection(sel))).toEqual(sel);
          expect(parseSelection(exploreHash(sel).split('?')[1] ?? '')).toEqual(sel);
        });
      }
    }
  }

  for (const cid of PHYSICS_CONFIG_IDS) {
    it(`round-trips Option B configuration ${cid} cleanly through the store and address`, () => {
      const mechanisms = configIdToFlags(cid);
      useStrato.getState().setSelection({ mechanisms, overlays: { methodA: true, methodB: false, experiment: true } });
      const sel = useStrato.getState().selection;
      expect(sel.mechanisms).toEqual(mechanisms);
      expect(parseSelection(formatSelection(sel))).toEqual(sel);
    });
  }

  it('changes one part and keeps the others', () => {
    const s = useStrato.getState();
    s.setSelection({ physics: 'decay' });
    s.setSelection({ scenario: 'startup' });
    s.setExpanded('method-a', true);
    expect(useStrato.getState().selection).toEqual({ ...DEFAULT_SELECTION, physics: 'decay', mechanisms: flagsOf('decay'), scenario: 'startup', expanded: ['method-a'] });
    s.setExpanded('method-a', false);
    expect(useStrato.getState().selection.expanded).toEqual([]);
  });

  it('never lets one group change the other: an operation does not select a model equation, and a model equation does not clear the operation', () => {
    const s = useStrato.getState();
    s.setSelection({ physics: 'baseline' });
    s.setSelection({ scenario: 'shutdown' });
    expect(useStrato.getState().selection).toMatchObject({ physics: 'baseline', scenario: 'shutdown' });
    s.setSelection({ physics: 'decay' });
    expect(useStrato.getState().selection).toMatchObject({ physics: 'decay', scenario: 'shutdown' });
    s.setSelection({ scenario: 'refuelling' });
    expect(useStrato.getState().selection).toMatchObject({ physics: 'decay', scenario: 'refuelling' });
    s.setSelection({ physics: 'future' });
    expect(useStrato.getState().selection).toMatchObject({ physics: 'future', scenario: 'refuelling' });
    // An address names the two parts independently as well.
    expect(parseSelection('physics=decay&scenario=control-rod')).toEqual({ ...DEFAULT_SELECTION, physics: 'decay', mechanisms: flagsOf('decay'), scenario: 'control-rod' });
    expect(normaliseSelection({ physics: 'temperature', scenario: 'refuelling' })).toMatchObject({ physics: 'temperature', scenario: 'refuelling' });
  });

  it('keeps the choices of the graphs and writes them to the address', () => {
    const sel = normaliseSelection({ graph: 'experiment', species: 'sr90', temperature: 1700, dataset: 'IAEA97-K31-SR90' });
    expect(sel).toMatchObject({ graph: 'experiment', species: 'sr90', temperature: 1700, dataset: 'IAEA97-K31-SR90' });
    expect(formatSelection(sel)).toBe('graph=experiment&species=sr90&temperature=1700&dataset=IAEA97-K31-SR90');
    expect(parseSelection(formatSelection(sel))).toEqual(sel);
    for (const graph of GRAPH_IDS) expect(parseSelection(`graph=${graph}`).graph).toBe(graph);
    // A value that is not one of the choices is the default; a temperature must be a number.
    expect(normaliseSelection({ graph: 'bogus', species: 'a b', temperature: Number.NaN, dataset: '../x' })).toEqual(DEFAULT_SELECTION);
    expect(parseSelection('temperature=abc').temperature).toBeNull();
  });

  it('replaces an invalid part by its default and an equal selection changes nothing', () => {
    const before = useStrato.getState().selection;
    useStrato.getState().setSelection({ physics: 'baseline' });
    expect(useStrato.getState().selection).toBe(before);
    expect(normaliseSelection({ physics: 'nonsense', scenario: 'later', expanded: ['x', 'method-a', 'method-a', 'numerical'] })).toEqual({ ...DEFAULT_SELECTION, expanded: ['method-a'] });
  });

  it('is not written to the browser storage', () => {
    const set = vi.spyOn(Storage.prototype, 'setItem');
    useStrato.getState().setSelection({ physics: 'combined', scenario: 'shutdown', expanded: ['method-a'], graph: 'profile' });
    expect(set).not.toHaveBeenCalled();
    set.mockRestore();
  });
});

describe('selection in the address', () => {
  let stop: (() => void) | null = null;
  afterEach(() => {
    stop?.();
    stop = null;
  });

  it('leaves out every default part: the default view is a bare #/explore', () => {
    expect(formatSelection(DEFAULT_SELECTION)).toBe('');
    expect(exploreHash(DEFAULT_SELECTION)).toBe('#/explore');
    expect(exploreHash({ ...DEFAULT_SELECTION, physics: 'generation' })).toBe('#/explore?physics=generation');
    expect(exploreHash({ ...DEFAULT_SELECTION, physics: 'combined', scenario: 'startup' })).toBe('#/explore?physics=combined&scenario=startup');
    expect(exploreHash({ ...DEFAULT_SELECTION, scenario: 'startup', expanded: ['method-a'] })).toBe('#/explore?scenario=startup&open=method-a');
    expect(hasSelectionKeys('physics=decay')).toBe(true);
    expect(hasSelectionKeys('temperature=1600')).toBe(true);
    expect(hasSelectionKeys('x=1')).toBe(false);
  });

  it('writes a changed selection to the address and reads a pasted address into the selection', () => {
    window.location.hash = '#/explore';
    stop = startAddressSync();
    act(() => useStrato.getState().setSelection({ physics: 'temperature', expanded: ['discretisation'] }));
    expect(window.location.hash).toBe('#/explore?physics=temperature&open=discretisation');
    // A copied address reproduces the view: a fresh state, then the address is followed.
    act(() => useStrato.setState({ selection: { ...DEFAULT_SELECTION, expanded: [] } }));
    act(() => goTo('#/explore?physics=future&scenario=shutdown&open=pseudocode&graph=methods&species=cs137&temperature=1800'));
    expect(useStrato.getState().selection).toEqual({
      ...DEFAULT_SELECTION,
      physics: 'future',
      mechanisms: flagsOf('future'),
      scenario: 'shutdown',
      expanded: ['pseudocode'],
      graph: 'methods',
      species: 'cs137',
      temperature: 1800,
      dataset: '',
    });
  });

  it('rewrites an address with invalid values to the selection actually held', () => {
    act(() => useStrato.setState({ selection: { ...DEFAULT_SELECTION, expanded: [] } }));
    window.location.hash = '#/explore?physics=bogus&scenario=zzz';
    stop = startAddressSync();
    expect(useStrato.getState().selection.physics).toBe('baseline');
    expect(window.location.hash).toBe(exploreHash(useStrato.getState().selection));
    expect(window.location.hash).not.toContain('bogus');
  });

  it('survives navigation between the sections', () => {
    window.location.hash = '#/explore?physics=decay';
    stop = startAddressSync();
    expect(useStrato.getState().selection.physics).toBe('decay');
    for (const id of ['compare', 'validate', 'verify', 'research', 'data', 'about']) {
      act(() => goTo(`#/${id}`));
      expect(useStrato.getState().selection.physics).toBe('decay');
      expect(window.location.hash).toBe(`#/${id}`);
    }
    // The navigation link back to Explore carries no query: the held selection returns to the address.
    act(() => goTo('#/explore'));
    expect(useStrato.getState().selection.physics).toBe('decay');
    expect(window.location.hash).toBe('#/explore?physics=decay');
  });

  it('does not rewrite the address of another section when the selection changes there', () => {
    stop = startAddressSync();
    act(() => goTo('#/compare'));
    act(() => useStrato.getState().setSelection({ physics: 'generation' }));
    expect(window.location.hash).toBe('#/compare');
  });

  it('does not leave the physics when the address is edited to an unknown value', () => {
    stop = startAddressSync();
    act(() => goTo('#/explore?physics=bogus&scenario=never'));
    expect(useStrato.getState().selection).toEqual(DEFAULT_SELECTION);
  });
});

describe('selection type', () => {
  it('lists six model equations, five operations, six sections and four graphs', () => {
    const sel: Selection = DEFAULT_SELECTION;
    expect(PHYSICS_IDS).toEqual(['baseline', 'generation', 'decay', 'temperature', 'combined', 'future']);
    expect(SCENARIO_IDS).toEqual(['steady', 'shutdown', 'startup', 'control-rod', 'refuelling']);
    expect(SECTION_IDS).toEqual(['method-a', 'method-b', 'discretisation', 'pseudocode', 'assumptions', 'limitations']);
    expect(GRAPH_IDS).toEqual(['release', 'profile', 'methods', 'experiment']);
    expect(sel.scenario).toBe('steady');
  });
});
