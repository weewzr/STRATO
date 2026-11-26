// The shared selection of the Explore page (owner directive of 26 Nov 2026, Section 6; decision 70):
// every combination, the address round trip, survival across navigation, no browser storage.
import { act } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { PHYSICS_IDS, SCENARIO_IDS } from '../src/equation/capability';
import { exploreHash, formatSelection, hasSelectionKeys, parseSelection, startAddressSync } from '../src/state/address';
import { DEFAULT_SELECTION, normaliseSelection, useStrato, type Selection } from '../src/state/store';

const goTo = (hash: string) => {
  window.location.hash = hash;
  window.dispatchEvent(new HashChangeEvent('hashchange'));
};

describe('selection state', () => {
  it('starts as baseline physics, steady-state operation, nothing expanded', () => {
    expect(useStrato.getState().selection).toEqual(DEFAULT_SELECTION);
  });

  for (const physics of PHYSICS_IDS) {
    for (const scenario of SCENARIO_IDS) {
      for (const expanded of [[], ['numerical']]) {
        it(`holds ${physics} / ${scenario} / ${expanded.length ? 'expanded' : 'collapsed'} and survives a round trip through the address`, () => {
          useStrato.getState().setSelection({ physics, scenario, expanded });
          const sel = useStrato.getState().selection;
          // The one coupling rule: a scenario other than steady-state operation requires the combined model.
          expect(sel).toEqual({ physics: scenario === 'steady' ? physics : 'combined', scenario, expanded });
          expect(parseSelection(formatSelection(sel))).toEqual(sel);
          expect(parseSelection(exploreHash(sel).split('?')[1] ?? '')).toEqual(sel);
        });
      }
    }
  }

  it('changes one part and keeps the others', () => {
    const s = useStrato.getState();
    s.setSelection({ physics: 'decay' });
    s.setSelection({ scenario: 'startup' });
    s.setExpanded('numerical', true);
    // Choosing a scenario selects the combined model (the rule), and expanding keeps both.
    expect(useStrato.getState().selection).toEqual({ physics: 'combined', scenario: 'startup', expanded: ['numerical'] });
    s.setExpanded('numerical', false);
    expect(useStrato.getState().selection.expanded).toEqual([]);
  });

  it('applies the coupling rule in both directions and for an address', () => {
    const s = useStrato.getState();
    s.setSelection({ scenario: 'shutdown' });
    expect(useStrato.getState().selection).toMatchObject({ physics: 'combined', scenario: 'shutdown' });
    // Another physics than the combined model returns to steady-state operation.
    s.setSelection({ physics: 'decay' });
    expect(useStrato.getState().selection).toMatchObject({ physics: 'decay', scenario: 'steady' });
    // The combined model keeps the scenario; a scenario named with a different physics wins.
    s.setSelection({ scenario: 'startup' });
    s.setSelection({ physics: 'combined' });
    expect(useStrato.getState().selection).toMatchObject({ physics: 'combined', scenario: 'startup' });
    expect(parseSelection('physics=decay&scenario=control-rod')).toEqual({ physics: 'combined', scenario: 'control-rod', expanded: [] });
    expect(normaliseSelection({ physics: 'temperature', scenario: 'refuelling' }).physics).toBe('combined');
  });

  it('replaces an invalid part by its default and an equal selection changes nothing', () => {
    const before = useStrato.getState().selection;
    useStrato.getState().setSelection({ physics: 'baseline' });
    expect(useStrato.getState().selection).toBe(before);
    expect(normaliseSelection({ physics: 'nonsense', scenario: 'later', expanded: ['x', 'numerical', 'numerical'] })).toEqual({
      physics: 'baseline',
      scenario: 'steady',
      expanded: ['numerical'],
    });
  });

  it('is not written to the browser storage', () => {
    const set = vi.spyOn(Storage.prototype, 'setItem');
    useStrato.getState().setSelection({ physics: 'combined', scenario: 'shutdown', expanded: ['numerical'] });
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
    expect(exploreHash({ physics: 'generation', scenario: 'steady', expanded: [] })).toBe('#/explore?physics=generation');
    expect(exploreHash({ physics: 'combined', scenario: 'startup', expanded: ['numerical'] })).toBe('#/explore?physics=combined&scenario=startup&open=numerical');
    expect(hasSelectionKeys('physics=decay')).toBe(true);
    expect(hasSelectionKeys('x=1')).toBe(false);
  });

  it('writes a changed selection to the address and reads a pasted address into the selection', () => {
    window.location.hash = '#/explore';
    stop = startAddressSync();
    act(() => useStrato.getState().setSelection({ physics: 'temperature', expanded: ['numerical'] }));
    expect(window.location.hash).toBe('#/explore?physics=temperature&open=numerical');
    // A copied address reproduces the view: a fresh state, then the address is followed.
    act(() => useStrato.setState({ selection: { ...DEFAULT_SELECTION, expanded: [] } }));
    act(() => goTo('#/explore?physics=combined&scenario=shutdown&open=numerical'));
    expect(useStrato.getState().selection).toEqual({ physics: 'combined', scenario: 'shutdown', expanded: ['numerical'] });
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
  it('lists five physics choices and five scenarios', () => {
    const sel: Selection = DEFAULT_SELECTION;
    expect(PHYSICS_IDS).toEqual(['baseline', 'generation', 'decay', 'temperature', 'combined']);
    expect(SCENARIO_IDS).toEqual(['steady', 'shutdown', 'startup', 'control-rod', 'refuelling']);
    expect(sel.scenario).toBe('steady');
  });
});
