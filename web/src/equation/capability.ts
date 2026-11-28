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
export type PhysicsId = 'baseline' | Mechanism | 'combined' | 'future';
/** The six legacy buttons of MODEL EQUATION, in order. 'future' shows the combined equation and is never a simulation. */
export const PHYSICS_IDS: readonly PhysicsId[] = ['baseline', 'generation', 'decay', 'temperature', 'combined', 'future'];

/** Option B independent physics toggles (P01–P08), species-specific physics selector, and 3 graph overlays. */
export type PhysicsConfigId = 'P01' | 'P02' | 'P03' | 'P04' | 'P05' | 'P06' | 'P07' | 'P08';
export const PHYSICS_CONFIG_IDS: readonly PhysicsConfigId[] = ['P01', 'P02', 'P03', 'P04', 'P05', 'P06', 'P07', 'P08'];
export interface PhysicsFlags {
  generation: boolean;
  decay: boolean;
  temperature: boolean;
}
export type PhysicsSelection = PhysicsId | PhysicsFlags;
export type SpeciesPhysicsId = 'none';
export interface OverlayFlags {
  methodA: boolean;
  methodB: boolean;
  experiment: boolean;
}
export const DEFAULT_MECHANISMS: PhysicsFlags = { generation: false, decay: false, temperature: false };
export const DEFAULT_OVERLAYS: OverlayFlags = { methodA: true, methodB: true, experiment: true };

export const CONFIG_LABELS: Record<PhysicsConfigId, { stageAlias: string; summary: string }> = {
  P01: { stageAlias: 'Stage A', summary: 'Baseline diffusion' },
  P02: { stageAlias: 'Stage B', summary: 'Baseline + generation' },
  P03: { stageAlias: 'Stage C (decay-only; G=0)', summary: 'Baseline + decay' },
  P04: { stageAlias: 'Stage D (heating ramp; G=0)', summary: 'Baseline + temperature-dependent D(T)' },
  P05: { stageAlias: 'Stage C', summary: 'Baseline + generation + decay' },
  P06: { stageAlias: 'P06 (+G, T(t))', summary: 'Baseline + generation + temperature-dependent D(T)' },
  P07: { stageAlias: 'Stage D (+decay, T(t))', summary: 'Baseline + decay + temperature-dependent D(T)' },
  P08: { stageAlias: 'Stage D (combined)', summary: 'Baseline + generation + decay + temperature-dependent D(T)' },
};

/** Operating scenarios (directive Section 3). Steady-state operation is the default. */
export type ScenarioId = 'steady' | 'shutdown' | 'startup' | 'control-rod' | 'refuelling';
export const SCENARIO_IDS: readonly ScenarioId[] = ['steady', 'shutdown', 'startup', 'control-rod', 'refuelling'];
export const DEFAULT_PHYSICS: PhysicsId = 'baseline';
/** Steady-state operation is the operation selected by default; the Model Equation and Operation groups are independent. */
export const DEFAULT_SCENARIO: ScenarioId | null = 'steady';

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

/** Convert either a legacy PhysicsId or a PhysicsFlags object into canonical boolean flags. */
export function flagsOf(physics: PhysicsSelection): PhysicsFlags {
  if (typeof physics !== 'string') {
    return { generation: Boolean(physics.generation), decay: Boolean(physics.decay), temperature: Boolean(physics.temperature) };
  }
  if (physics === 'baseline') return { generation: false, decay: false, temperature: false };
  if (physics === 'combined' || physics === 'future') return { generation: true, decay: true, temperature: true };
  return {
    generation: physics === 'generation',
    decay: physics === 'decay',
    temperature: physics === 'temperature',
  };
}

/** Map active mechanism flags to the canonical P01–P08 configuration id. */
export function flagsToConfigId(flags: PhysicsFlags): PhysicsConfigId {
  const { generation: g, decay: d, temperature: t } = flags;
  if (!g && !d && !t) return 'P01';
  if (g && !d && !t) return 'P02';
  if (!g && d && !t) return 'P03';
  if (!g && !d && t) return 'P04';
  if (g && d && !t) return 'P05';
  if (g && !d && t) return 'P06';
  if (!g && d && t) return 'P07';
  return 'P08';
}

export function configIdToFlags(id: PhysicsConfigId): PhysicsFlags {
  switch (id) {
    case 'P01':
      return { generation: false, decay: false, temperature: false };
    case 'P02':
      return { generation: true, decay: false, temperature: false };
    case 'P03':
      return { generation: false, decay: true, temperature: false };
    case 'P04':
      return { generation: false, decay: false, temperature: true };
    case 'P05':
      return { generation: true, decay: true, temperature: false };
    case 'P06':
      return { generation: true, decay: false, temperature: true };
    case 'P07':
      return { generation: false, decay: true, temperature: true };
    case 'P08':
      return { generation: true, decay: true, temperature: true };
  }
}

export function flagsToPhysicsId(flags: PhysicsFlags): PhysicsId {
  const { generation: g, decay: d, temperature: t } = flags;
  const count = Number(g) + Number(d) + Number(t);
  if (count === 0) return 'baseline';
  if (count === 1) return g ? 'generation' : d ? 'decay' : 'temperature';
  return 'combined';
}

/** The mechanisms that a physics selection switches on. */
export function mechanismsOf(physics: PhysicsSelection): Mechanism[] {
  const f = flagsOf(physics);
  return MECHANISMS.filter((m) => f[m]);
}

/** The status of a physics selection: the weakest of the baseline and the mechanisms it contains. */
export function physicsStatus(cap: BuildCapability, physics: PhysicsSelection): BuildStatus {
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
export function selectionSupport(cap: BuildCapability, physics: PhysicsSelection, scenario: ScenarioId | null): SelectionSupport {
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
export function runMechanisms(physics: PhysicsSelection): string[] {
  return [BTCS_CAPABILITY, ...mechanismsOf(physics).map((m) => WRAPPER_ID[m])];
}

/** What the build does not implement, as a phrase ("a, b and c"). */
export function missingList(support: SelectionSupport): string {
  return phrase(support.missing);
}

/** One sentence for the simulation and results blocks when the selection cannot be run. */
export function unsupportedMessage(support: SelectionSupport): string {
  const list = missingList(support);
  return `The solver in this build does not implement ${list}.`;
}
