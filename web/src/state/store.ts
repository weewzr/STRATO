// Application state (zustand), shaped as in prompt/TECHNICAL_ARCHITECTURE.md Section 4.
// Defaults come from the generated case JSON (strato-webtools case), never from literals.
import { create } from 'zustand';
import { cloneConfig, DEFAULT_CASE, findCase, type RegionConfig, type CaseConfig } from '../data/case';
import {
  DEFAULT_MECHANISMS,
  DEFAULT_OVERLAYS,
  DEFAULT_PHYSICS,
  DEFAULT_SCENARIO,
  flagsOf,
  flagsToPhysicsId,
  PHYSICS_IDS,
  SCENARIO_IDS,
  type OverlayFlags,
  type PhysicsFlags,
  type PhysicsId,
  type ScenarioId,
  type SpeciesPhysicsId,
} from '../equation/capability';
import { DEFAULT_GRAPH, GRAPH_IDS, SECTION_IDS, type GraphId } from '../equation/selectionIds';
import type { BtcsLimits, BuildInfo, CompareOk, Grid, RunResponse, ValidateReply } from '../solver/types';

/**
 * The one selection state of the Explore page (final directive of 27 Nov 2026): the model equation, the
 * operation (two independent choices: neither changes the other and nothing is activated silently), the
 * expanded sections of the Numerical Interpretation, and the choices of the graphs (which graph, which
 * species, which temperature, which experimental dataset). It lives in this store, so it survives
 * navigation between the sections, and state/address.ts writes it to and reads it from the address, so a
 * copied address reproduces the view. It is never stored in the browser. A species, temperature or dataset
 * that is empty (or null) stands for the first one that the wrapper lists; the page resolves it.
 */
export interface Selection {
  physics: PhysicsId;
  /** Independent mechanism flags for Option B stages P01–P08. */
  mechanisms: PhysicsFlags;
  /** Future species-specific physics selector. */
  speciesPhysics: SpeciesPhysicsId;
  /** The selected operation (steady-state operation by default). */
  scenario: ScenarioId | null;
  /** Ids of the expanded sections of the Numerical Interpretation (equation/selectionIds.ts: SECTION_IDS). */
  expanded: string[];
  /** The graph shown in INTERACTIVE GRAPHS. */
  graph: GraphId;
  /** Id of the species (the wrapper's nuclide id, for example sr90); empty for the first listed. */
  species: string;
  /** The temperature in degC; null for the first one offered. */
  temperature: number | null;
  /** Id of the experimental dataset (the comparison file); empty for the first one of the species. */
  dataset: string;
  /** Independent graph curve overlay toggles. */
  overlays: OverlayFlags;
  /** Optional controlled matched-inventory curve overlay (when applicable). */
  controlledCompare: boolean;
}
export const DEFAULT_SELECTION: Selection = {
  physics: DEFAULT_PHYSICS,
  mechanisms: { ...DEFAULT_MECHANISMS },
  speciesPhysics: 'none',
  scenario: DEFAULT_SCENARIO,
  expanded: [],
  graph: DEFAULT_GRAPH,
  species: '',
  temperature: null,
  dataset: '',
  overlays: { ...DEFAULT_OVERLAYS },
  controlledCompare: false,
};
/** The ids that `expanded` may hold, in the fixed order in which they are kept. */
export const EXPANDABLE_IDS: readonly string[] = SECTION_IDS;
export type RunStatus = 'idle' | 'running' | 'ok' | 'rejected' | 'error';
export type RegionField = 'outer_radius_m' | 'diffusivity_m2_s' | 'cells';
export type TimeField = 'dt_s' | 'end_time_s' | 'output_every_steps';
/** Colour map of the particle colouring (prompt/DESIGN_SPECIFICATION.md Section 9: cividis default, viridis option). */
export type ColormapName = 'cividis' | 'viridis';
/** View of the simulated particle in block B: the 2D cross-section (default) or the 3D cutaway. */
export type ParticleView = '2d' | '3d';
/** Playback speeds offered (prompt/DESIGN_SPECIFICATION.md Section 4.2: 0.25x to 8x). */
export const PLAYBACK_SPEEDS = [0.25, 0.5, 1, 2, 4, 8] as const;
/** The plot shown in block B on a phone, where one plot is shown at a time (prompt/DESIGN_SPECIFICATION.md Section 2.1). */
export type SimPlot = 'profile' | 'release';

export interface PlaybackState {
  /** Index of the output row shown (run.profiles[tIndex]); the solver's recorded rows, never interpolated. */
  tIndex: number;
  playing: boolean;
  speed: number;
}

/** The WebAssembly solver as the page sees it (web phase 3). */
export interface SolverState {
  state: 'loading' | 'ready' | 'failed';
  build: BuildInfo | null;
  error: string | null;
  /** Workers replaced after a trap or a time-out. */
  restarts: number;
}

/** The last run reply that is not a result: rejected inputs or a solver failure. */
export interface RunProblem {
  kind: 'rejected' | 'error';
  messages: string[];
}

export interface RunInfo {
  /** Wall time of the wrapper call in the worker (performance.now()), not a result. */
  elapsedMs: number;
  requestId: number;
  /** Key of the inputs the result belongs to (inputKey of the config sent). */
  inputKey: string;
  /** Mechanism ids that isAvailable() removed from the request. */
  dropped: string[];
  /** selectionKey of the physics and scenario selected when the request was sent; the reply is shown only under that selection (solver/shown.ts). */
  selectionKey?: string;
}

/** A key of the physics and the scenario of a selection (the expanded regions are display state and are not part of it). */
export const selectionKey = (sel: { physics: PhysicsId; scenario: ScenarioId | null }): string => `${sel.physics}|${sel.scenario ?? ''}`;

/** The key under which the manual solver's reply is recorded: it always runs the baseline problem, whatever the Explore selection is. */
export const BASELINE_RUN_KEY = selectionKey({ physics: 'baseline', scenario: null });

/**
 * The one condition under which a computed status may be shown (badge, research-status chip of a result,
 * result label in the footer, run-derived text): the stored run is the manual solver's run of the baseline
 * problem, it belongs to the current inputs, and the build supports the baseline run. The Explore selection of
 * the model equation and the operation does not enter: it neither starts nor changes this run. Everything that
 * presents a result as computed or verified reads this, so that a run of other inputs is never described as the
 * result of the current ones.
 */
export function runIsCurrent(
  s: { run: unknown; runInfo: RunInfo | null; config: CaseConfig },
  supported: boolean,
): boolean {
  if (!supported || s.run === null || s.run === undefined || s.runInfo === null) return false;
  // A reply sent without a selection key (older callers) counts as the baseline run, as in solver/shown.ts.
  const sameSelection = s.runInfo.selectionKey === undefined || s.runInfo.selectionKey === BASELINE_RUN_KEY;
  return sameSelection && s.runInfo.inputKey === inputKey(s.config);
}

/** The comparison of the current inputs with the baseline (compare), run while the Compare section is open (web phase 5). */
export interface ComparisonState {
  status: 'idle' | 'running' | 'ok' | 'rejected' | 'error';
  reply: CompareOk | null;
  problem: string[] | null;
  /** comparisonKey of the baseline and current inputs that the reply (or the problem) belongs to. */
  key: string;
  /** Wall time of the wrapper call in the worker, not a result. */
  elapsedMs: number | null;
}

/** The wrapper's check of the current inputs (validate): its own messages, the plan and the grid of the run. */
export interface ValidationState {
  /** inputKey of the configuration the answer belongs to; empty before the first answer. */
  key: string;
  state: 'idle' | 'ok' | 'rejected' | 'error';
  errors: string[];
  warnings: string[];
  plan: ValidateReply['plan'];
  grid: Grid | null;
  limits: BtcsLimits | null;
}

export interface StratoState {
  caseId: string;
  config: CaseConfig;
  baselineConfig: CaseConfig;
  /** Mechanism ids the wasm wrapper reports from build_info(); empty until the module has loaded (web phase 3). */
  capabilities: string[];
  /** The shared selection of the Explore page (physics, scenario, expanded regions); mirrored in the address. */
  selection: Selection;
  run: RunResponse | null;
  baselineRun: RunResponse | null;
  status: RunStatus;
  solver: SolverState;
  runInfo: RunInfo | null;
  problem: RunProblem | null;
  playback: PlaybackState;
  comparison: ComparisonState;
  validation: ValidationState;
  view: {
    showComparison: boolean;
    showDifference: boolean;
    showValidation: boolean;
    logColour: boolean;
    colormap: ColormapName;
    particleView: ParticleView;
    selectedLayer: number | null;
    /** Display options of the plots (web phase 5); they change no input and no result. */
    profileLog: boolean;
    profileStart: boolean;
    releaseLog: boolean;
    /** Which solution methods the release plot draws: Method A (finite-difference BTCS) and Method B (analytical solution). */
    showMethod1: boolean;
    showMethod2: boolean;
    simPlot: SimPlot;
  };

  loadCase: (id: string) => void;
  setRegionField: (index: number, field: RegionField, value: number) => void;
  setTimeField: (field: TimeField, value: number) => void;
  setKernelConcentration: (value: number) => void;
  setTemperature: (value: number) => void;
  resetToBaseline: () => void;
  setCurrentAsBaseline: () => void;
  /** Replaces the selection (invalid parts fall back to the default); an equal selection changes nothing. */
  setSelection: (patch: Partial<Selection>) => void;
  /** Expands or collapses one numerical region; the rest of the selection is kept. */
  setExpanded: (id: string, open: boolean) => void;
  /** Records build_info().capabilities of the wasm wrapper (web phase 3 calls it). */
  setCapabilities: (capabilities: string[]) => void;
  selectLayer: (index: number | null) => void;
  /** Display options of the particle (colour scale, colour map, 2D or 3D); they change no input and no result. */
  /**
   * Display options: the particle (colour scale, colour map, 2D or 3D), the plots (log axes, start
   * profile, the plot shown on a phone) and the comparison and validation layers (Compare and Validate
   * sections, independent of each other). They change no input and no result.
   */
  setView: (patch: Partial<Omit<StratoState['view'], 'selectedLayer'>>) => void;
  /** Records the wrapper's answer to validate (called by solver/session.ts). */
  setValidation: (v: ValidationState) => void;
  /** Records the state of the comparison (called by solver/session.ts). */
  setComparison: (patch: Partial<ComparisonState>) => void;
  /** Asks the solver session for a comparison now; a no-op until solver/session.ts installs it. */
  requestCompare: () => void;
  /** Playback over the recorded output rows of the current run (web phase 4). */
  setPlayback: (patch: Partial<PlaybackState>) => void;
  stepPlayback: (delta: number) => void;
  setSolver: (patch: Partial<SolverState>) => void;
  /** Records a solver reply: a result, rejected inputs or a failure (called by solver/session.ts). */
  setRunOutcome: (outcome: { run: RunResponse; info: RunInfo } | { problem: RunProblem; info: RunInfo | null }) => void;
  setRunning: () => void;
  /** Asks the solver session for a run now; a no-op until solver/session.ts installs it. */
  requestRun: () => void;
}

/** A key of the inputs (display and tests only): equal configs give equal keys. */
export function inputKey(cfg: CaseConfig): string {
  const text = JSON.stringify(cfg);
  let h = 0x811c9dc5;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h.toString(16).padStart(8, '0');
}

/** A selection that holds only valid parts: anything else becomes the default; the expanded ids are unique and in a fixed order. */
export function normaliseSelection(sel: {
  physics?: string | undefined;
  mechanisms?: Partial<PhysicsFlags> | undefined;
  speciesPhysics?: string | undefined;
  scenario?: string | null | undefined;
  expanded?: readonly string[] | undefined;
  graph?: string | undefined;
  species?: string | undefined;
  temperature?: number | null | undefined;
  dataset?: string | undefined;
  overlays?: Partial<OverlayFlags> | undefined;
  controlledCompare?: boolean | undefined;
}): Selection {
  const hasExplicitPhysics = sel.physics !== undefined && PHYSICS_IDS.includes(sel.physics as PhysicsId);
  const rawMech = sel.mechanisms;
  const mechanisms: PhysicsFlags = rawMech
    ? {
        generation: Boolean(rawMech.generation),
        decay: Boolean(rawMech.decay),
        temperature: Boolean(rawMech.temperature),
      }
    : hasExplicitPhysics
      ? flagsOf(sel.physics as PhysicsId)
      : { ...DEFAULT_MECHANISMS };
  const physics: PhysicsId = hasExplicitPhysics ? (sel.physics as PhysicsId) : flagsToPhysicsId(mechanisms);
  const speciesPhysics: SpeciesPhysicsId = 'none';
  const scenario = SCENARIO_IDS.includes(sel.scenario as ScenarioId) ? (sel.scenario as ScenarioId) : DEFAULT_SELECTION.scenario;
  const expanded = EXPANDABLE_IDS.filter((id) => (sel.expanded ?? []).includes(id));
  const graph = GRAPH_IDS.includes(sel.graph as GraphId) ? (sel.graph as GraphId) : DEFAULT_SELECTION.graph;
  const text = (x: string | undefined): string => (typeof x === 'string' && /^[A-Za-z0-9._-]{1,80}$/.test(x) ? x : '');
  const temperature = typeof sel.temperature === 'number' && Number.isFinite(sel.temperature) ? sel.temperature : null;
  const overlays: OverlayFlags = sel.overlays
    ? {
        methodA: sel.overlays.methodA ?? DEFAULT_OVERLAYS.methodA,
        methodB: sel.overlays.methodB ?? DEFAULT_OVERLAYS.methodB,
        experiment: sel.overlays.experiment ?? DEFAULT_OVERLAYS.experiment,
      }
    : { ...DEFAULT_OVERLAYS };
  const controlledCompare = Boolean(sel.controlledCompare);
  // The model equation and the operation are independent: neither is changed to suit the other.
  return { physics, mechanisms, speciesPhysics, scenario, expanded, graph, species: text(sel.species), temperature, dataset: text(sel.dataset), overlays, controlledCompare };
}

export const sameSelection = (a: Selection, b: Selection): boolean =>
  a.physics === b.physics &&
  a.mechanisms.generation === b.mechanisms.generation &&
  a.mechanisms.decay === b.mechanisms.decay &&
  a.mechanisms.temperature === b.mechanisms.temperature &&
  a.speciesPhysics === b.speciesPhysics &&
  a.scenario === b.scenario &&
  a.graph === b.graph &&
  a.species === b.species &&
  a.temperature === b.temperature &&
  a.dataset === b.dataset &&
  a.overlays.methodA === b.overlays.methodA &&
  a.overlays.methodB === b.overlays.methodB &&
  a.overlays.experiment === b.overlays.experiment &&
  a.controlledCompare === b.controlledCompare &&
  a.expanded.length === b.expanded.length &&
  a.expanded.every((id, i) => id === b.expanded[i]);

export function initialState() {
  return {
    caseId: DEFAULT_CASE.id,
    config: cloneConfig(DEFAULT_CASE.config),
    baselineConfig: cloneConfig(DEFAULT_CASE.config),
    capabilities: [] as string[],
    selection: {
      ...DEFAULT_SELECTION,
      mechanisms: { ...DEFAULT_MECHANISMS },
      overlays: { ...DEFAULT_OVERLAYS },
      expanded: [] as string[],
    } as Selection,
    run: null,
    baselineRun: null,
    status: 'idle' as RunStatus,
    solver: { state: 'loading', build: null, error: null, restarts: 0 } as SolverState,
    runInfo: null as RunInfo | null,
    problem: null as RunProblem | null,
    playback: { tIndex: 0, playing: false, speed: 1 },
    comparison: { status: 'idle', reply: null, problem: null, key: '', elapsedMs: null } as ComparisonState,
    validation: { key: '', state: 'idle', errors: [], warnings: [], plan: null, grid: null, limits: null } as ValidationState,
    view: {
      // Compare section: the baseline curve is shown by default, the difference curve is optional
      // (prompt/DESIGN_SPECIFICATION.md Section 8.1). Validate section: no prediction layer exists yet.
      showComparison: true,
      showDifference: false,
      showValidation: false,
      // Logarithmic by default: at most output times most of the particle is far below c0, and a
      // linear scale would show it as one colour.
      logColour: true,
      colormap: 'cividis' as ColormapName,
      particleView: '2d' as ParticleView,
      selectedLayer: null as number | null,
      // The profile's concentration axis has the particle's default scale: six decades up to c0.
      profileLog: true,
      profileStart: true,
      releaseLog: false,
      showMethod1: true,
      showMethod2: true,
      simPlot: 'profile' as SimPlot,
    },
  };
}

function withRegion(cfg: CaseConfig, index: number, patch: Partial<RegionConfig>): CaseConfig {
  const next = cloneConfig(cfg);
  const r = next.regions[index];
  if (r) Object.assign(r, patch);
  return next;
}

export const useStrato = create<StratoState>()((set, get) => ({
  ...initialState(),

  loadCase: (id) => {
    const c = findCase(id);
    if (!c) return;
    set({
      caseId: c.id,
      config: cloneConfig(c.config),
      baselineConfig: cloneConfig(c.config),
      run: null,
      baselineRun: null,
      status: 'idle',
      validation: { key: '', state: 'idle', errors: [], warnings: [], plan: null, grid: null, limits: null },
    });
  },

  setRegionField: (index, field, value) => set({ config: withRegion(get().config, index, { [field]: value }) }),

  setTimeField: (field, value) => {
    const next = cloneConfig(get().config);
    next.time[field] = value;
    set({ config: next });
  },

  setKernelConcentration: (value) => {
    const next = cloneConfig(get().config);
    next.initial.kernel_concentration_mol_m3 = value;
    set({ config: next });
  },

  setTemperature: (value) => {
    const next = cloneConfig(get().config);
    next.temperature_k = value;
    set({ config: next });
  },

  resetToBaseline: () => {
    const run = get().baselineRun;
    set({
      config: cloneConfig(get().baselineConfig),
      run,
      playback: { tIndex: Math.max(0, (run?.profiles.length ?? 1) - 1), playing: false, speed: get().playback.speed },
    });
  },

  setCurrentAsBaseline: () => set({ baselineConfig: cloneConfig(get().config), baselineRun: get().run }),


  setSelection: (patch) => {
    // The model equation mechanisms and operation groups are independent: changing one never alters the other.
    const prev = get().selection;
    const merged: Parameters<typeof normaliseSelection>[0] = { ...prev, ...patch };
    if (patch.mechanisms !== undefined && patch.physics === undefined) {
      merged.physics = flagsToPhysicsId(patch.mechanisms);
    } else if (patch.physics !== undefined && patch.mechanisms === undefined) {
      merged.mechanisms = flagsOf(patch.physics);
    }
    const next = normaliseSelection(merged);
    if (!sameSelection(next, prev)) set({ selection: next });
  },

  setExpanded: (id, open) => {
    const cur = get().selection.expanded;
    get().setSelection({ expanded: open ? [...cur, id] : cur.filter((e) => e !== id) });
  },

  setCapabilities: (capabilities) => set({ capabilities: [...capabilities] }),

  selectLayer: (index) => set({ view: { ...get().view, selectedLayer: index } }),

  setView: (patch) => set({ view: { ...get().view, ...patch } }),

  setPlayback: (patch) => {
    const rows = get().run?.profiles.length ?? 0;
    const next = { ...get().playback, ...patch };
    next.tIndex = rows > 0 ? Math.min(rows - 1, Math.max(0, Math.trunc(next.tIndex))) : 0;
    if (rows === 0) next.playing = false;
    set({ playback: next });
  },

  stepPlayback: (delta) => {
    const p = get().playback;
    get().setPlayback({ tIndex: p.tIndex + delta, playing: false });
  },

  setSolver: (patch) => set({ solver: { ...get().solver, ...patch } }),

  setValidation: (validation) => set({ validation }),

  setComparison: (patch) => set({ comparison: { ...get().comparison, ...patch } }),

  requestCompare: () => undefined,

  setRunOutcome: (outcome) => {
    // A new result is shown at its last output row (t_end, the time of the release readout); playback stops.
    const speed = get().playback.speed;
    if ('run' in outcome)
      set({
        run: outcome.run,
        runInfo: outcome.info,
        problem: null,
        status: 'ok',
        playback: { tIndex: Math.max(0, outcome.run.profiles.length - 1), playing: false, speed },
      });
    else set({ run: null, runInfo: outcome.info, problem: outcome.problem, status: outcome.problem.kind, playback: { tIndex: 0, playing: false, speed } });
  },

  setRunning: () => set({ status: 'running' }),

  requestRun: () => undefined,
}));

export interface ConfigDifference {
  path: string;
  label: string;
  unit: string;
  baseline: number | string;
  current: number | string;
}

/** Key of a (baseline, current) pair of configurations (display and tests only). */
export function comparisonKey(baseline: CaseConfig, current: CaseConfig): string {
  return `${inputKey(baseline)}|${inputKey(current)}`;
}

/** Lists the input fields whose values differ between two configurations (equality test only). */
export function configDifferences(baseline: CaseConfig, current: CaseConfig): ConfigDifference[] {
  const out: ConfigDifference[] = [];
  const cmp = (path: string, label: string, unit: string, a: number | string, b: number | string) => {
    if (!Object.is(a, b)) out.push({ path, label, unit, baseline: a, current: b });
  };
  cmp('temperature_k', 'Temperature (K), recorded, not used by the solver', 'K', baseline.temperature_k, current.temperature_k);
  current.regions.forEach((r, i) => {
    const b = baseline.regions[i];
    if (!b) return;
    cmp(`regions.${i}.outer_radius_m`, `Outer radius of the ${r.name} layer (m)`, 'm', b.outer_radius_m, r.outer_radius_m);
    cmp(`regions.${i}.diffusivity_m2_s`, `Diffusivity of the ${r.name} layer (m²/s)`, 'm²/s', b.diffusivity_m2_s, r.diffusivity_m2_s);
    cmp(`regions.${i}.cells`, `Grid intervals in the ${r.name} layer (dimensionless)`, 'dimensionless', b.cells, r.cells);
  });
  cmp(
    'initial.kernel_concentration_mol_m3',
    'Initial kernel concentration (mol/m³)',
    'mol/m³',
    baseline.initial.kernel_concentration_mol_m3,
    current.initial.kernel_concentration_mol_m3,
  );
  cmp('time.dt_s', 'Time step (s)', 's', baseline.time.dt_s, current.time.dt_s);
  cmp('time.end_time_s', 'Simulation duration (s)', 's', baseline.time.end_time_s, current.time.end_time_s);
  cmp('time.output_every_steps', 'Time steps between output rows (dimensionless)', 'dimensionless', baseline.time.output_every_steps, current.time.output_every_steps);
  return out;
}