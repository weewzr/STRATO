// Application state (zustand), shaped as in prompt/TECHNICAL_ARCHITECTURE.md Section 4.
// Defaults come from the generated case JSON (strato-webtools case), never from literals.
import { create } from 'zustand';
import { cloneConfig, DEFAULT_CASE, findCase, type RegionConfig, type CaseConfig } from '../data/case';
import { DEFAULT_PHYSICS, DEFAULT_SCENARIO, PHYSICS_IDS, SCENARIO_IDS, type PhysicsId, type ScenarioId } from '../equation/capability';
import { requiresCombined } from '../equation/scenarios';
import type { Stage } from '../equation/terms';
import type { BtcsLimits, BuildInfo, CompareOk, Grid, RunResponse, ValidateReply } from '../solver/types';

/** Builder step shown in the Numerical Interpretation region (prompt/DESIGN_SPECIFICATION.md Section 3.1). */
export type { Stage };

/**
 * The one selection state of the Explore page (owner directive of 26 Nov 2026, Section 6): the physics
 * of the governing equation, the operating scenario and which numerical regions are expanded. It lives
 * in this store, so it survives navigation between the sections, and state/address.ts writes it to and
 * reads it from the address, so a copied address reproduces the view. It is never stored in the browser.
 */
export interface Selection {
  physics: PhysicsId;
  /** The selected operating scenario; null while none is selected (the default). */
  scenario: ScenarioId | null;
  /** Ids of the expanded numerical regions (today only 'numerical', the Numerical Interpretation region). */
  expanded: string[];
}
export const DEFAULT_SELECTION: Selection = { physics: DEFAULT_PHYSICS, scenario: DEFAULT_SCENARIO, expanded: [] };
/** The ids that `expanded` may hold. */
export const EXPANDABLE_IDS = ['numerical'] as const;
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
  /** Displayed builder stage. Selecting a stage changes the display only, never the inputs or a run. */
  stage: Stage;
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
  setStage: (stage: Stage) => void;
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
export function normaliseSelection(sel: { physics?: string | undefined; scenario?: string | null | undefined; expanded?: readonly string[] | undefined }): Selection {
  const physics = PHYSICS_IDS.includes(sel.physics as PhysicsId) ? (sel.physics as PhysicsId) : DEFAULT_SELECTION.physics;
  const scenario = SCENARIO_IDS.includes(sel.scenario as ScenarioId) ? (sel.scenario as ScenarioId) : DEFAULT_SELECTION.scenario;
  const expanded = EXPANDABLE_IDS.filter((id) => (sel.expanded ?? []).includes(id));
  // The one rule that couples the controls (equation/scenarios.ts, SCENARIO_RULE): every scenario
  // requires the combined model, so the scenario wins over the physics it is held with.
  return { physics: requiresCombined(scenario) ? 'combined' : physics, scenario, expanded };
}

export const sameSelection = (a: Selection, b: Selection): boolean =>
  a.physics === b.physics && a.scenario === b.scenario && a.expanded.length === b.expanded.length && a.expanded.every((id, i) => id === b.expanded[i]);

export function initialState() {
  return {
    caseId: DEFAULT_CASE.id,
    config: cloneConfig(DEFAULT_CASE.config),
    baselineConfig: cloneConfig(DEFAULT_CASE.config),
    stage: 1 as Stage,
    capabilities: [] as string[],
    selection: { ...DEFAULT_SELECTION, expanded: [] as string[] },
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

  setStage: (stage) => set({ stage }),

  setSelection: (patch) => {
    // Choosing a physics that is not the combined model (without naming a scenario) clears the scenario.
    const merged = { ...get().selection, ...patch };
    if (patch.physics !== undefined && patch.scenario === undefined && patch.physics !== 'combined') merged.scenario = DEFAULT_SCENARIO;
    const next = normaliseSelection(merged);
    if (!sameSelection(next, get().selection)) set({ selection: next });
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