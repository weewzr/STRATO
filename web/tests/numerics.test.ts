// @vitest-environment node
// The fourteen items of the Numerical Interpretation: the mapping of the seventeen derivation steps (every step
// appears exactly once), the visibility table (which items for which physics and scenario), the notes that a
// scenario writes inside the items, and the expanded state in the shared selection and the address.
import { describe, expect, it } from 'vitest';
import { PHYSICS_IDS, SCENARIO_IDS, type PhysicsId, type ScenarioId } from '../src/equation/capability';
import {
  effectivePhysics,
  ITEM_EXPAND_IDS,
  ITEM_IDS,
  itemExpandId,
  itemVisible,
  NUMERICAL_ITEMS,
  scenarioItems,
  scenarioNote,
  STEP_TO_ITEM,
  visibleItems,
  type ItemId,
} from '../src/equation/numerics';
import { STAGES, STEP_DEFS, type Stage } from '../src/equation/terms';
import { formatSelection, parseSelection } from '../src/state/address';
import { EXPANDABLE_IDS, normaliseSelection } from '../src/state/store';

describe('the fourteen items and the mapping of the seventeen steps', () => {
  it('are the directive items in the directive order', () => {
    expect(NUMERICAL_ITEMS.map((i) => i.title)).toEqual([
      'Spherical diffusion equation and modelling assumptions',
      'Five-layer TRISO geometry and material properties',
      'Central finite-difference spatial discretisation',
      'Backward Euler time discretisation',
      'Centre-node treatment',
      'Material-interface conditions',
      'Absorbing outer-boundary condition',
      'Tridiagonal matrix assembly',
      'Generation source-term contribution',
      'Radioactive decay contribution',
      'Temperature-dependent diffusion coefficients',
      'Rust BTCS solution algorithm',
      'Inventory and release accounting',
      'Numerical verification and convergence',
    ]);
    expect(NUMERICAL_ITEMS.map((i) => i.id)).toEqual([...ITEM_IDS]);
  });

  // The mapping table of the report, step to item (also the code comment of equation/numerics.ts).
  const MAPPING: [Stage, ItemId][] = [
    [1, 1], [2, 1], [3, 1], [4, 2], [5, 3], [6, 4], [7, 5], [8, 6], [9, 7], [10, 8],
    [15, 9], [16, 10], [17, 11], [11, 12], [12, 13], [14, 13], [13, 14],
  ];

  it('holds every existing step in exactly one item, as the mapping table says', () => {
    const placed = NUMERICAL_ITEMS.flatMap((i) => i.stages);
    expect([...placed].sort((a, b) => a - b)).toEqual([...STAGES]);
    expect(new Set(placed).size).toBe(placed.length);
    expect(Object.fromEntries(MAPPING)).toEqual(STEP_TO_ITEM);
    for (const [s, item] of MAPPING) expect(NUMERICAL_ITEMS.find((i) => i.id === item)?.stages, `step ${s}`).toContain(s);
  });

  it('puts the mechanism steps, and only them, in the items that a mechanism reveals', () => {
    for (const item of NUMERICAL_ITEMS) {
      const mechanisms = item.stages.map((s) => STEP_DEFS[s].mechanism);
      if (item.mechanism === null) expect(mechanisms.every((m) => m === null), `item ${item.id}`).toBe(true);
      else expect(mechanisms, `item ${item.id}`).toEqual([{ generation: 'generation', decay: 'decay', temperature: 'diffusivity_temperature' }[item.mechanism]]);
    }
    expect(NUMERICAL_ITEMS.filter((i) => i.mechanism !== null).map((i) => [i.id, i.mechanism])).toEqual([
      [9, 'generation'],
      [10, 'decay'],
      [11, 'temperature'],
    ]);
  });
});

describe('visibility table: which items for which physics and scenario', () => {
  const BASE: ItemId[] = [1, 2, 3, 4, 5, 6, 7, 8, 12, 13, 14];
  const ALL: ItemId[] = [...ITEM_IDS];
  const table: [PhysicsId, ScenarioId | null, ItemId[]][] = [
    ['baseline', null, BASE],
    ['generation', null, [1, 2, 3, 4, 5, 6, 7, 8, 9, 12, 13, 14]],
    ['decay', null, [1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 13, 14]],
    ['temperature', null, [1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 14]],
    ['combined', null, ALL],
    // Every scenario carries the combined model (SCENARIO_RULE), so the physics held with it does not matter.
    ...SCENARIO_IDS.flatMap((s) => PHYSICS_IDS.map((p): [PhysicsId, ScenarioId | null, ItemId[]] => [p, s, ALL])),
  ];
  for (const [physics, scenario, expected] of table) {
    it(`${physics} / ${scenario ?? 'no scenario'} shows items ${expected.join(', ')}`, () => {
      expect(visibleItems(physics, scenario)).toEqual(expected);
    });
  }

  it('shows the baseline items for every selection and a mechanism item only with its mechanism', () => {
    for (const p of PHYSICS_IDS) {
      for (const s of [null, ...SCENARIO_IDS]) {
        const shown = visibleItems(p, s);
        for (const id of BASE) expect(shown, `${p}/${s} item ${id}`).toContain(id);
        const eff = effectivePhysics(p, s);
        expect(shown.includes(9)).toBe(eff === 'generation' || eff === 'combined');
        expect(shown.includes(10)).toBe(eff === 'decay' || eff === 'combined');
        expect(shown.includes(11)).toBe(eff === 'temperature' || eff === 'combined');
        for (const it of NUMERICAL_ITEMS) expect(itemVisible(it, p, s), `${p}/${s} item ${it.id}`).toBe(shown.includes(it.id));
      }
    }
  });
});

describe('what a scenario writes inside the items', () => {
  const where: Record<ScenarioId, ItemId[]> = {
    steady: [9, 11, 12],
    shutdown: [9, 10, 11],
    startup: [9, 11],
    'control-rod': [9, 11],
    refuelling: [9, 10, 11],
  };
  it('is a note in the items where the scenario acts, and nowhere else', () => {
    for (const s of SCENARIO_IDS) {
      expect(scenarioItems(s), s).toEqual(where[s]);
      for (const id of ITEM_IDS) expect(scenarioNote(s, id) !== null, `${s} item ${id}`).toBe(where[s].includes(id));
      expect(scenarioNote(null, 9)).toBeNull();
    }
  });
  it('says each thing once, without numerals, and never invents a control-rod term', () => {
    const notes = SCENARIO_IDS.flatMap((s) => scenarioItems(s).map((i) => scenarioNote(s, i) ?? ''));
    expect(new Set(notes).size).toBe(notes.length);
    for (const n of notes) expect(n, n).not.toMatch(/\d/);
    expect(scenarioNote('control-rod', 9)).toMatch(/no term is assembled for the control rod/);
    // Shutdown: the source changes, decay stays on the diagonal, the diffusivity follows the temperature and the matrix is reassembled.
    expect(scenarioNote('shutdown', 9)).toMatch(/generation is zero/);
    expect(scenarioNote('shutdown', 10)).toMatch(/diagonal/);
    expect(scenarioNote('shutdown', 11)).toMatch(/reassembled/);
    // Steady-state operation: constant conditions are not a stationary field.
    expect(scenarioNote('steady', 12)).toMatch(/do not make the concentration field stationary/);
    // Startup: time-dependent generation and temperature enter through the step average and the middle of the step.
    expect(scenarioNote('startup', 9)).toMatch(/step average of the generation/);
    expect(scenarioNote('startup', 11)).toMatch(/middle of the step/);
  });
});

describe('the expanded state of the items in the shared selection', () => {
  it('knows the region and the fourteen items, in a fixed order', () => {
    expect(EXPANDABLE_IDS).toEqual(['numerical', ...ITEM_IDS.map((i) => `n${i}`)]);
    expect(ITEM_EXPAND_IDS).toEqual(ITEM_IDS.map(itemExpandId));
  });
  it('keeps valid ids, drops the others and orders them', () => {
    const sel = normaliseSelection({ physics: 'generation', expanded: ['n9', 'x', 'numerical', 'n3', 'n9', 'n15'] });
    expect(sel.expanded).toEqual(['numerical', 'n3', 'n9']);
  });
  it('survives a round trip through the address, with the physics and the scenario', () => {
    const sel = normaliseSelection({ physics: 'combined', scenario: 'shutdown', expanded: ['n11', 'numerical', 'n8'] });
    const query = formatSelection(sel);
    expect(parseSelection(query)).toEqual(sel);
    expect(query).toContain('open=numerical,n8,n11');
  });
});
