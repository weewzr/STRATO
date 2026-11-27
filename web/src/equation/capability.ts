// What the running build can solve: the ONE object the Explore page asks (owner directive of 26 Nov
// 2026, Section 2 and decision 70). It is derived from what the WebAssembly wrapper of the running build
// reports about itself (build_info().btcs.capabilities, recorded in the store) and from the generated
// research catalogue; no flag is set by hand, so a later wrapper that reports more mechanisms turns the
// controls on without an edit to the page logic. Nothing here computes a scientific value.
import { CATALOGUE, type Catalogue } from '../research/catalogue';
import { BTCS_CAPABILITY, isAvailable } from './availability';

/** The three approved additional mechanisms (directive Section 2). */
export type Mechanism = 'generation' | 'decay' | 'temperature';
export const MECHANISMS: readonly Mechanism[] = ['generation', 'decay', 'temperature'];

/** The physics selection of the Explore page: the baseline, one mechanism on its own, or all of them. */
export type PhysicsId = 'baseline' | Mechanism | 'combined';
export const PHYSICS_IDS: readonly PhysicsId[] = ['baseline', 'generation', 'decay', 'temperature', 'combined'];

/** Operating scenarios (directive Section 3). Steady-state operation is the default. */
export type ScenarioId = 'steady' | 'shutdown' | 'startup' | 'control-rod' | 'refuelling';
export const SCENARIO_IDS: readonly ScenarioId[] = ['steady', 'shutdown', 'startup', 'control-rod', 'refuelling'];
export const DEFAULT_PHYSICS: PhysicsId = 'baseline';
/** No scenario is selected by default: the page then runs the physics selected with constant inputs (the baseline run is "Baseline diffusion", never a scenario). */
export const DEFAULT_SCENARIO: ScenarioId | null = null;

export type BuildStatus = 'implemented' | 'verified' | 'planned';
export const BUILD_STATUSES: readonly BuildStatus[] = ['implemented', 'verified', 'planned'];

/** The three statuses, each defined in one sentence; the page shows the label and offers the sentence. */
export const STATUS_LABEL: Record<BuildStatus, string> = { implemented: 'Implemented', verified: 'Verified', planned: 'Planned' };
export const STATUS_DEFINITION: Record<BuildStatus, string> = {
  implemented: 'The solver in this build reports the mechanism, but its verification is not yet recorded in the research catalogue, so the simulation does not use it.',
  verified: 'The solver in this build reports the mechanism and the research catalogue records it as verified, so the simulation uses it.',
  planned: 'The solver in this build does not report the mechanism: only its equation and explanation are shown, and the simulation shows no result for it.',
};

/** The identifier under which the wrapper reports each mechanism (it is also the mechanism id of the research catalogue). */
export const WRAPPER_ID: Record<Mechanism, string> = { generation: 'generation', decay: 'decay', temperature: 'diffusivity_temperature' };

/**
 * The identifier under which the wrapper reports each scenario of the scenario engine (it is also the id of
 * the scenario's row in the research catalogue). Steady-state operation is Scenario 1 of the science design
 * (constant generation at the reference conditions, from a fresh particle): the baseline run is not that
 * scenario, so it needs its own report from the wrapper like every other scenario.
 */
export const WRAPPER_SCENARIO_ID: Record<ScenarioId, string> = {
  steady: 'scenario_normal_operation',
  shutdown: 'scenario_shutdown_cooldown',
  startup: 'scenario_startup',
  'control-rod': 'scenario_control_rod_insertion',
  refuelling: 'scenario_fuel_removal',
};

export interface BuildCapability {
  /** The constant-diffusivity BTCS model of the build. */
  baseline: BuildStatus;
  mechanisms: Record<Mechanism, BuildStatus>;
  /**
   * Scenario support: what the wrapper reports under WRAPPER_SCENARIO_ID for each of the five scenarios
   * (planned while it reports nothing, as the wrapper of this build does).
   */
  scenarios: Record<ScenarioId, BuildStatus>;
}

function statusOfReport(reported: boolean, verified: boolean): BuildStatus {
  if (!reported) return 'planned';
  return verified ? 'verified' : 'implemented';
}

/** The capability of the build whose wrapper reported `reported` (build_info().btcs.capabilities). */
export function buildCapability(reported: readonly string[], catalogue: Catalogue = CATALOGUE): BuildCapability {
  const baseline = statusOfReport(reported.includes(BTCS_CAPABILITY), isAvailable(BTCS_CAPABILITY, reported, catalogue));
  const mechanisms = {} as Record<Mechanism, BuildStatus>;
  for (const m of MECHANISMS) mechanisms[m] = statusOfReport(reported.includes(WRAPPER_ID[m]), isAvailable(WRAPPER_ID[m], reported, catalogue));
  const scenario = (id: ScenarioId): BuildStatus => statusOfReport(reported.includes(WRAPPER_SCENARIO_ID[id]), isAvailable(WRAPPER_SCENARIO_ID[id], reported, catalogue));
  return {
    baseline,
    mechanisms,
    scenarios: { steady: scenario('steady'), shutdown: scenario('shutdown'), startup: scenario('startup'), 'control-rod': scenario('control-rod'), refuelling: scenario('refuelling') },
  };
}

/** The weakest of several statuses. */
function weakest(all: BuildStatus[]): BuildStatus {
  if (all.includes('planned')) return 'planned';
  if (all.includes('implemented')) return 'implemented';
  return 'verified';
}

/** The simulation may use a mechanism, the baseline or a scenario only at the verified status. */
export const isSimulable = (s: BuildStatus): boolean => s === 'verified';

/** The mechanisms that a physics selection switches on. */
export function mechanismsOf(physics: PhysicsId): Mechanism[] {
  if (physics === 'baseline') return [];
  if (physics === 'combined') return [...MECHANISMS];
  return [physics];
}

/** The status of a physics selection: the weakest of the baseline and the mechanisms it contains. */
export function physicsStatus(cap: BuildCapability, physics: PhysicsId): BuildStatus {
  return weakest([cap.baseline, ...mechanismsOf(physics).map((m) => cap.mechanisms[m])]);
}

/**
 * The status shown on a scenario button: the weakest of the scenario's own status and that of the physics
 * selected. Every scenario runs with the combined model (see state/store.ts), so its status includes the
 * three mechanisms.
 */
export function scenarioStatus(cap: BuildCapability, scenario: ScenarioId): BuildStatus {
  return weakest([cap.scenarios[scenario], physicsStatus(cap, 'combined')]);
}

export interface SelectionSupport {
  /** True when the solver of this build can run the selection exactly as selected. */
  supported: boolean;
  /** What the build does not implement, by name (empty when supported). */
  missing: string[];
}

export const MECHANISM_NAME: Record<Mechanism, string> = {
  generation: 'fission-product generation',
  decay: 'radioactive decay',
  temperature: 'temperature-dependent diffusion',
};
export const SCENARIO_NAME: Record<ScenarioId, string> = {
  steady: 'steady-state operation',
  shutdown: 'shutdown and cooldown',
  startup: 'startup',
  'control-rod': 'control-rod insertion',
  refuelling: 'fuel-removal',
};

/** Whether the build can run the selection; never substitutes another model (the page shows nothing instead). */
export function selectionSupport(cap: BuildCapability, physics: PhysicsId, scenario: ScenarioId | null): SelectionSupport {
  const missing: string[] = [];
  if (!isSimulable(cap.baseline)) missing.push('the baseline BTCS model');
  for (const m of mechanismsOf(physics)) if (!isSimulable(cap.mechanisms[m])) missing.push(MECHANISM_NAME[m]);
  if (scenario !== null && !isSimulable(cap.scenarios[scenario])) missing.push(`the ${SCENARIO_NAME[scenario]} scenario`);
  return { supported: missing.length === 0, missing };
}

/**
 * What the running computation implements, from the capability of the build alone: the mechanisms its
 * wrapper reports at the verified status (the simulation uses those) and those it does not compute.
 */
export function computedMechanisms(cap: BuildCapability): { computed: Mechanism[]; notComputed: Mechanism[] } {
  return { computed: MECHANISMS.filter((m) => isSimulable(cap.mechanisms[m])), notComputed: MECHANISMS.filter((m) => !isSimulable(cap.mechanisms[m])) };
}

/** A list of names as a phrase ("a, b and c"). */
export function phrase(names: readonly string[]): string {
  return names.length > 1 ? `${names.slice(0, -1).join(', ')} and ${names[names.length - 1]}` : (names[0] ?? '');
}

/** The mechanism ids a run asks the wrapper for: the baseline capability and the selected mechanisms (the client drops what is not available). */
export function runMechanisms(physics: PhysicsId): string[] {
  return [BTCS_CAPABILITY, ...mechanismsOf(physics).map((m) => WRAPPER_ID[m])];
}

/** What the build does not implement, as a phrase ("a, b and c"). */
export function missingList(support: SelectionSupport): string {
  return support.missing.length > 1 ? `${support.missing.slice(0, -1).join(', ')} and ${support.missing[support.missing.length - 1]}` : (support.missing[0] ?? '');
}

/** One sentence for the simulation and results blocks when the selection cannot be run. */
export function unsupportedMessage(support: SelectionSupport): string {
  const list = missingList(support);
  return `The solver in this build does not implement ${list}.`;
}
