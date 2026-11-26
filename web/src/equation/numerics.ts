// The fourteen items of the Numerical Interpretation and what decides which of them the page shows (owner
// directive of 26 Nov 2026, Sections 4 and 5; decision 70(c)). Display logic only: nothing here is evaluated.
//
// The seventeen derivation steps of the earlier builder (equation/terms.ts) are grouped into the directive's
// fourteen items, each step in exactly one item. Mapping, step to item (the test in tests/numerics.test.ts
// holds this table):
//
//   step  existing name                          item  directive item
//    1    Conservation of mass                     1    Spherical diffusion equation and modelling assumptions
//    2    Fick's law                               1
//    3    Spherical diffusion equation             1
//    4    Five-layer material model                2    Five-layer TRISO geometry and material properties
//    5    Central spatial differences              3    Central finite-difference spatial discretisation
//    6    Backward Euler discretisation            4    Backward Euler time discretisation
//    7    Centre-node equation                     5    Centre-node treatment
//    8    Interface-node equations                 6    Material-interface conditions
//    9    Outer-boundary equation                  7    Absorbing outer-boundary condition
//   10    Tridiagonal matrix                       8    Tridiagonal matrix assembly
//   15    Generation (a preview before)            9    Generation source-term contribution
//   16    Decay (a preview before)                10    Radioactive decay contribution
//   17    Temperature (a preview before)          11    Temperature-dependent diffusion coefficients
//   11    Rust solution algorithm                 12    Rust BTCS solution algorithm
//   12    Inventory and release                   13    Inventory and release accounting
//   14    Experimental comparisons (placeholder)  13    (its content, the release fraction, belongs to the accounting;
//                                                         the comparison itself is under Compare and Validate)
//   13    Numerical verification                  14    Numerical verification and convergence
//
// Items nine to eleven had no derivation before: they are written from Document 5.
import { mechanismsOf, type BuildCapability, type BuildStatus, type Mechanism, type PhysicsId, type ScenarioId } from './capability';
import { requiresCombined } from './scenarios';
import type { Stage } from './terms';

export type ItemId = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14;
export const ITEM_IDS: readonly ItemId[] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];

export interface NumericalItem {
  id: ItemId;
  title: string;
  /** The derivation steps (terms.ts) that this item holds, in the order they are shown. */
  stages: readonly Stage[];
  /** The mechanism whose selection reveals the item; null for the items of the baseline model. */
  mechanism: Mechanism | null;
}

/** The fourteen items in the directive's order. */
export const NUMERICAL_ITEMS: readonly NumericalItem[] = [
  { id: 1, title: 'Spherical diffusion equation and modelling assumptions', stages: [1, 2, 3], mechanism: null },
  { id: 2, title: 'Five-layer TRISO geometry and material properties', stages: [4], mechanism: null },
  { id: 3, title: 'Central finite-difference spatial discretisation', stages: [5], mechanism: null },
  { id: 4, title: 'Backward Euler time discretisation', stages: [6], mechanism: null },
  { id: 5, title: 'Centre-node treatment', stages: [7], mechanism: null },
  { id: 6, title: 'Material-interface conditions', stages: [8], mechanism: null },
  { id: 7, title: 'Absorbing outer-boundary condition', stages: [9], mechanism: null },
  { id: 8, title: 'Tridiagonal matrix assembly', stages: [10], mechanism: null },
  { id: 9, title: 'Generation source-term contribution', stages: [15], mechanism: 'generation' },
  { id: 10, title: 'Radioactive decay contribution', stages: [16], mechanism: 'decay' },
  { id: 11, title: 'Temperature-dependent diffusion coefficients', stages: [17], mechanism: 'temperature' },
  { id: 12, title: 'Rust BTCS solution algorithm', stages: [11], mechanism: null },
  { id: 13, title: 'Inventory and release accounting', stages: [12, 14], mechanism: null },
  { id: 14, title: 'Numerical verification and convergence', stages: [13], mechanism: null },
];

/**
 * The correspondence between the symbols of the governing equation (the directive's G, lambda and D(r,T)) and
 * those of the documents, stated once on the page (decision 70(a)); the mathematics is unchanged. Inline
 * mathematics between dollar signs; tests/numerics.test.ts checks every piece against the documents.
 */
export const NOTATION =
  'Notation of the documents: $i$ is the radial node index, $j$ the time-step index and $C_i^j$ the concentration. The diffusivity $D(r,T)$ of the governing equation is $D_{\\mathrm{material}}$ of the layer, evaluated for a step as $\\bar D_{\\mathrm{material}}^j$; the source $G$ enters as the step average $\\bar G_{\\mathrm{material}}^j$ and the nodal density $g_i^j$; $\\lambda$ enters as the decay number $\\kappa = \\lambda\\,\\Delta t$; the molar gas constant is $R_{\\mathrm{gas}}$, because $R_1, \\dots, R_5$ are radii.';

/** The item that holds a derivation step. */
export const STEP_TO_ITEM: Record<Stage, ItemId> = Object.fromEntries(NUMERICAL_ITEMS.flatMap((it) => it.stages.map((s) => [s, it.id] as const))) as Record<Stage, ItemId>;

/** The ids under which the shared selection keeps whether an item is expanded (state/store.ts: `expanded`). */
export const itemExpandId = (id: ItemId): string => `n${id}`;
export const ITEM_EXPAND_IDS: readonly string[] = ITEM_IDS.map(itemExpandId);

/**
 * The physics that the numerical items follow: the selected one, except that a scenario always carries the
 * combined model (equation/scenarios.ts, SCENARIO_RULE), so the two cannot disagree.
 */
export const effectivePhysics = (physics: PhysicsId, scenario: ScenarioId | null): PhysicsId => (requiresCombined(scenario) ? 'combined' : physics);

/** Whether an item is shown for a physics selection and an operating scenario. A scenario never hides an item: it adds notes inside them. */
export function itemVisible(item: NumericalItem, physics: PhysicsId, scenario: ScenarioId | null): boolean {
  if (item.mechanism === null) return true;
  return mechanismsOf(effectivePhysics(physics, scenario)).includes(item.mechanism);
}

/** The ids of the items shown for a selection, in order. */
export function visibleItems(physics: PhysicsId, scenario: ScenarioId | null): ItemId[] {
  return NUMERICAL_ITEMS.filter((it) => itemVisible(it, physics, scenario)).map((it) => it.id);
}

/** The status of the thing an item describes in the running build: the mechanism's status, or the baseline's. */
export function itemStatus(cap: BuildCapability, item: NumericalItem): BuildStatus {
  return item.mechanism === null ? cap.baseline : cap.mechanisms[item.mechanism];
}

/**
 * How the selected operating scenario enters the numerical scheme, written inside the item where it acts and
 * nowhere else (directive Section 5). A scenario adds no term: it changes the source G to G(r,t) and the
 * temperature to T(t), which enter through the step average of the source (item 9), the decay that continues
 * (item 10), the diffusivity at the middle of the step and the reassembly of the matrix (item 11), and, for
 * steady-state operation, the difference between constant conditions and a stationary field (item 12).
 * Prose without numerals: the numbers of a history are registered data and are not written here.
 */
const SCENARIO_ITEM_NOTES: Record<ScenarioId, Partial<Record<ItemId, string>>> = {
  steady: {
    9: 'The generation is constant, so its step average equals the prescribed value at every step.',
    11: 'The temperature is constant, so the diffusivity of each material is the same at every step: with one uniform time step the matrix is assembled and factorised once, and only the right-hand side changes from step to step.',
    12: 'A stationary field would repeat from step to step. The run does not assume it: it starts from the initial state and advances through every step.',
  },
  shutdown: {
    9: 'From the shutdown time the step-averaged generation is zero, so the right-hand side of each step is the concentration of the previous step.',
    10: 'Decay continues after shutdown: its contribution stays on the diagonal of the matrix.',
    11: 'The diffusivity at the temperature of the middle of the step changes with every cooldown step, so the matrix is reassembled at each of those steps, held once the temperature is held, and reassembled again when the time step changes.',
  },
  startup: {
    9: 'While the power rises, the step average of the generation changes the right-hand side at every step; it is constant once the prescribed rise has ended.',
    11: 'The prescribed temperature rise changes the diffusivity at the middle of the step, and with it the matrix, at every step of the rise; both are constant once the rise has ended.',
  },
  'control-rod': {
    9: 'The prescribed change of the fission-rate history enters only through the step-averaged generation on the right-hand side; no term is assembled for the control rod.',
    11: 'The prescribed change of the temperature history enters only through the diffusivity at the middle of the step in the matrix; it changes during the transition and is held afterwards.',
  },
  refuelling: {
    9: 'After removal the step-averaged generation is zero, as in shutdown, so the right-hand side of each step is the concentration of the previous step.',
    10: 'Decay continues in storage: its contribution stays on the diagonal of the matrix.',
    11: 'The matrix follows the prescribed storage temperature history.',
  },
};

/** The note of a scenario inside an item, or null when the scenario changes nothing there. */
export function scenarioNote(scenario: ScenarioId | null, item: ItemId): string | null {
  if (scenario === null) return null;
  return SCENARIO_ITEM_NOTES[scenario][item] ?? null;
}

/** The items in which a scenario writes a note (for the tests and the visibility table). */
export function scenarioItems(scenario: ScenarioId): ItemId[] {
  return ITEM_IDS.filter((id) => SCENARIO_ITEM_NOTES[scenario][id] !== undefined);
}
