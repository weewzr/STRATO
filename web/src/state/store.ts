// Application state (zustand), shaped as in prompt/TECHNICAL_ARCHITECTURE.md Section 4.
// Defaults come from the generated case JSON (scripts/web/generate_data.py), never from literals.
import { create } from 'zustand';
import { cloneConfig, DEFAULT_CASE, findCase, type RegionConfig, type V1Config } from '../data/case';
import { isAvailable } from '../equation/availability';
import { SWITCH_MECHANISM, type Stage } from '../equation/terms';
import type { RunResponse } from '../solver/types';

// Mechanism switches; the ids map to mechanism_id rows of data/research/mechanisms.csv through
// SWITCH_MECHANISM (equation/terms.ts): decay -> decay, source -> generation, arrhenius -> diffusivity_temperature.
export type Mechanism = 'decay' | 'source' | 'arrhenius';
/** Builder stage shown in block A (prompt/DESIGN_SPECIFICATION.md Section 3.1). */
export type { Stage };
export type RunStatus = 'idle' | 'running' | 'ok' | 'rejected' | 'error';
export type RegionField = 'outer_radius_m' | 'diffusivity_m2_s' | 'cells';
export type TimeField = 'dt_s' | 'end_time_s' | 'output_every_steps';

export interface MechanismState {
  enabled: boolean;
  /** Set only from isAvailable() (equation/availability.ts): reported by the wrapper and at the live status. */
  available: boolean;
}

export interface StratoState {
  caseId: string;
  config: V1Config;
  baselineConfig: V1Config;
  /** Displayed builder stage. Selecting a stage changes the display only, never the inputs or a run. */
  stage: Stage;
  /** Mechanism ids the wasm wrapper reports from build_info(); empty until the module has loaded (web phase 3). */
  capabilities: string[];
  mechanisms: Record<Mechanism, MechanismState>;
  run: RunResponse | null;
  baselineRun: RunResponse | null;
  status: RunStatus;
  playback: { tIndex: number; playing: boolean; speed: number };
  view: {
    showComparison: boolean;
    showDifference: boolean;
    showValidation: boolean;
    logColour: boolean;
    selectedLayer: number | null;
  };

  loadCase: (id: string) => void;
  setRegionField: (index: number, field: RegionField, value: number) => void;
  setTimeField: (field: TimeField, value: number) => void;
  setKernelConcentration: (value: number) => void;
  setTemperature: (value: number) => void;
  resetToBaseline: () => void;
  setCurrentAsBaseline: () => void;
  setMechanism: (m: Mechanism, enabled: boolean) => void;
  setStage: (stage: Stage) => void;
  /** Records build_info().capabilities of the wasm wrapper and recomputes availability (web phase 3 calls it). */
  setCapabilities: (capabilities: string[]) => void;
  selectLayer: (index: number | null) => void;
}

const MECHANISM_KEYS: Mechanism[] = ['decay', 'source', 'arrhenius'];

function mechanismsFor(capabilities: readonly string[], prev?: Record<Mechanism, MechanismState>): Record<Mechanism, MechanismState> {
  const out = {} as Record<Mechanism, MechanismState>;
  for (const m of MECHANISM_KEYS) {
    const available = isAvailable(SWITCH_MECHANISM[m], capabilities);
    out[m] = { enabled: available && (prev?.[m].enabled ?? false), available };
  }
  return out;
}

const initialMechanisms = (): Record<Mechanism, MechanismState> => mechanismsFor([]);

export function initialState() {
  return {
    caseId: DEFAULT_CASE.id,
    config: cloneConfig(DEFAULT_CASE.config),
    baselineConfig: cloneConfig(DEFAULT_CASE.config),
    stage: 1 as Stage,
    capabilities: [] as string[],
    mechanisms: initialMechanisms(),
    run: null,
    baselineRun: null,
    status: 'idle' as RunStatus,
    playback: { tIndex: 0, playing: false, speed: 1 },
    view: { showComparison: false, showDifference: false, showValidation: false, logColour: false, selectedLayer: null },
  };
}

function withRegion(cfg: V1Config, index: number, patch: Partial<RegionConfig>): V1Config {
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

  resetToBaseline: () => set({ config: cloneConfig(get().baselineConfig), run: get().baselineRun }),

  setCurrentAsBaseline: () => set({ baselineConfig: cloneConfig(get().config), baselineRun: get().run }),

  setMechanism: (m, enabled) => {
    const cur = get().mechanisms[m];
    // Only a mechanism that the wrapper reports and whose catalogue status is the live one can be switched on.
    if (!cur.available || !isAvailable(SWITCH_MECHANISM[m], get().capabilities)) return;
    set({ mechanisms: { ...get().mechanisms, [m]: { ...cur, enabled } } });
  },

  setStage: (stage) => set({ stage }),

  setCapabilities: (capabilities) => set({ capabilities: [...capabilities], mechanisms: mechanismsFor(capabilities, get().mechanisms) }),

  selectLayer: (index) => set({ view: { ...get().view, selectedLayer: index } }),
}));

export interface ConfigDifference {
  path: string;
  label: string;
  unit: string;
  baseline: number | string;
  current: number | string;
}

/** Lists the input fields whose values differ between two configurations (equality test only). */
export function configDifferences(baseline: V1Config, current: V1Config): ConfigDifference[] {
  const out: ConfigDifference[] = [];
  const cmp = (path: string, label: string, unit: string, a: number | string, b: number | string) => {
    if (!Object.is(a, b)) out.push({ path, label, unit, baseline: a, current: b });
  };
  cmp('temperature_k', 'Temperature (recorded only in v1.0)', 'K', baseline.temperature_k, current.temperature_k);
  current.regions.forEach((r, i) => {
    const b = baseline.regions[i];
    if (!b) return;
    cmp(`regions.${i}.outer_radius_m`, `${r.name} outer radius`, 'm', b.outer_radius_m, r.outer_radius_m);
    cmp(`regions.${i}.diffusivity_m2_s`, `${r.name} diffusivity`, 'm² s⁻¹', b.diffusivity_m2_s, r.diffusivity_m2_s);
    cmp(`regions.${i}.cells`, `${r.name} cells`, '', b.cells, r.cells);
  });
  cmp(
    'initial.kernel_concentration_mol_m3',
    'Kernel start concentration',
    'mol m⁻³',
    baseline.initial.kernel_concentration_mol_m3,
    current.initial.kernel_concentration_mol_m3,
  );
  cmp('time.dt_s', 'Time step', 's', baseline.time.dt_s, current.time.dt_s);
  cmp('time.end_time_s', 'End time', 's', baseline.time.end_time_s, current.time.end_time_s);
  cmp('time.output_every_steps', 'Output every', 'steps', baseline.time.output_every_steps, current.time.output_every_steps);
  return out;
}
