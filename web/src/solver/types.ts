// TypeScript mirror of the replies of the BTCS interface of the wrapper (schema 2;
// crates/strato-wasm/src/btcs_api.rs; the field list of the wrapper's interface description).
// Types only: nothing in this file computes a value.

export type ResultLabel = 'COMPUTED v2.0' | 'PRECOMPUTED' | 'MEASURED' | 'ILLUSTRATIVE';

export interface EngineStamp {
  strato_version: string;
  commit: string;
  abi: number;
  solver: string;
}

/** The solver block of a reply and of build_info().btcs: what it is, its version, its scope. */
export interface SolverInfo {
  id: string;
  method: string;
  capability: string;
  version: string;
  commit: string;
  label: ResultLabel;
  formulation: string;
  scope: string;
  nature: string;
}

export interface BtcsLimits {
  max_total_intervals: number;
  min_intervals_per_layer: number;
  max_steps: number;
  max_profile_values: number;
  max_profile_times: number;
  max_series_modes: number;
}

/** build_info().btcs: the interface the page uses. */
export interface BtcsBuildInfo {
  solver: SolverInfo;
  schema: 2;
  functions: string[];
  export_functions: string[];
  export_formats: string[];
  capabilities: string[];
  label: ResultLabel;
  limits: BtcsLimits;
}

/** build_info(): the build stamp and the BTCS block. The page reads only `btcs` for capabilities. */
export interface BuildInfo {
  abi: number;
  crate: string;
  wrapper_version: string;
  strato_version: string;
  commit: string;
  dirty: boolean;
  wasm_bindgen: string;
  btcs: BtcsBuildInfo;
}

/** Echo of the validated configuration in a run reply. */
export interface RunInputs {
  case_name: string;
  species: string;
  temperature_k: number;
  temperature_used_by_solver: boolean;
  regions: { name: string; outer_radius_m: number; diffusivity_m2_s: number; intervals: number }[];
  initial: { kernel_concentration_mol_m3: number };
  time: { dt_s: number; end_time_s: number; output_every_steps: number };
}

/** The nodal grid. Unknowns are nodal values at node_radii_m; there are no cells. */
export interface Grid {
  intervals_total: number;
  nodes: number;
  intervals_per_layer: number[];
  node_radii_m: number[];
  node_class: ('centre' | 'interior' | 'interface' | 'outer')[];
  node_layer_inner: number[];
  node_layer_outer: number[];
  layer_of_interval: number[];
  interface_nodes: number[];
  interface_radii_m: number[];
  spacing_per_layer_m: number[];
  layer_names: string[];
  /** Presentation geometry: the colour band of node i is [ring_edges_m[i], ring_edges_m[i+1]]. */
  ring_edges_m: number[];
}

export interface InterfaceFlux {
  interface_nodes: number[];
  radii_m: number[];
  outward_from_inner_expression_mol_s: number[];
  outward_from_outer_expression_mol_s: number[];
  difference_mol_s: number[];
  difference_bound_mol_s: number[];
  within_bound: boolean;
}

export interface RunProfile {
  step: number;
  t_s: number;
  requested_times_s: number[];
  c_mol_per_m3: number[];
  inventory_mol: number;
  layer_inventory_mol: number[];
  layer_share: number[] | null;
  interface_flux: InterfaceFlux | null;
}

/** The independent solution: the Sturm-Liouville series, Method B. */
export interface Analytical {
  applicable: boolean;
  name: string;
  assumptions: string;
  times_s: number[];
  release_fraction: number[];
  accuracy_floor: number;
  valid_from_s: number;
  modes: number;
}

export interface Balance {
  form: string;
  statement: string;
  source: string;
  start_inventory_mol: number;
  generated_mol: number;
  decayed_mol: number;
  generated_and_decayed_note: string;
  final: { t_s: number; remaining_mol: number; released_mol: number; residual_mol: number; residual_bound_mol: number };
  max_abs_residual_mol: number;
  max_abs_residual_relative: number;
  tolerance: { kind: string; final_bound_relative: number; max_ratio_residual_to_bound: number; within_bound: boolean };
}

export interface AnalyticalComparison {
  modes: number;
  max_abs_difference: number;
  final_abs_difference: number;
  accuracy_floor: number;
  valid_from_s: number;
  min_flux_ratio: number;
  resolved: boolean;
}

export interface Diagnostics {
  intervals_total: number;
  nodes: number;
  intervals_per_layer: number[];
  steps: number;
  dt_requested_s: number;
  dt_used_s: number;
  end_time_s: number;
  output_rows: number;
  profiles_returned: number;
  kernel_surface_start_fraction: number;
  fourier_numbers: number[];
  matrix: { min_row_margin: number; max_interior_margin_deviation: number; norm_inf: number } | null;
  final_step: {
    step: number;
    min_new_mol_m3: number;
    max_new_mol_m3: number;
    max_old_mol_m3: number;
    non_negative: boolean;
    max_principle_excess_mol_m3: number;
  } | null;
  series_properties: {
    release_monotone: boolean;
    release_in_range: boolean;
    inventory_non_increasing: boolean;
    max_inventory_increase_relative: number;
  };
  final_release_fraction: number;
  max_abs_balance_residual_mol: number;
  max_abs_balance_residual_relative: number;
  min_concentration_mol_m3: number;
  max_concentration_mol_m3: number;
  non_negative: boolean;
  layer_sum_max_relative_difference: number;
  interface_flux: { profiles_with_flux: number; max_relative_difference: number; all_within_bound: boolean };
  analytical_comparison: AnalyticalComparison | null;
}

export interface RunOk {
  schema: 2;
  status: 'ok';
  label: ResultLabel;
  engine: EngineStamp;
  solver: SolverInfo;
  mechanisms: string[];
  /** The validated configuration the wrapper ran, echoed back. */
  inputs: RunInputs;
  grid: Grid;
  times_s: number[];
  steps_of_rows: number[];
  release_fraction: number[];
  /** Null at the first row: the release rate is not defined at time zero. */
  release_rate_mol_s: (number | null)[];
  inventory_mol: number[];
  cumulative_release_mol: number[];
  balance_residual_mol: number[];
  balance_residual_bound_mol: number[];
  min_concentration_mol_m3: number[];
  max_concentration_mol_m3: number[];
  profiles: RunProfile[];
  profile_times_requested: boolean;
  balance: Balance;
  diagnostics: Diagnostics;
  analytical: Analytical | null;
  messages: string[];
}

export interface Rejected {
  schema: 2;
  status: 'rejected';
  errors: string[];
  engine?: EngineStamp;
}

export type RunResponse = RunOk;
export type RunReply = RunOk | Rejected;

/** compare: both runs and the difference series, modified minus baseline, computed in Rust. */
export interface CompareOk {
  schema: 2;
  status: 'ok';
  label: ResultLabel;
  engine: EngineStamp;
  solver: SolverInfo;
  baseline: RunOk;
  modified: RunOk;
  difference: {
    times_s: number[];
    release_fraction: number[];
    release_rate_mol_s: (number | null)[];
    inventory_mol: number[];
    cumulative_release_mol: number[];
  };
  difference_definition: string;
  summary: { max_abs_release_fraction_difference: number; final_release_fraction_difference: number };
  changes: { path: string; baseline: unknown; modified: unknown }[];
}

export type CompareReply = CompareOk | Rejected;

/** validate: the same rules as run, nothing is solved; the grid and the plan preview the run. */
export interface ValidateReply {
  schema: 2;
  ok: boolean;
  errors: string[];
  warnings: string[];
  plan: { intervals_total: number; nodes: number; steps: number; dt_used_s: number; output_rows: number; profile_values_at_output_rows: number } | null;
  grid: Grid | null;
  solver: SolverInfo;
  limits: BtcsLimits;
}

/** export_run: the text of a data file written by the library's export, with its provenance header. */
export interface ExportOk {
  schema: 2;
  status: 'ok';
  label: ResultLabel;
  engine: EngineStamp;
  format: ExportFormat;
  media_type: string;
  text: string;
}

export type ExportFormat = 'timeseries_csv' | 'profiles_csv' | 'run_json' | 'comparison_csv';
export type ExportReply = ExportOk | Rejected;

// ---- ABI 4 (schema 3): the functions run_stage, analytical and options of the wrapper
// (stage-b/btcs/wasm-interface.md Part B, sections 11 and 15). Only the fields the page reads are typed.

/** The reply of run_stage: the series and the profiles of a stage, verbatim from the wrapper. */
export interface StageSeries {
  times_s: number[];
  times_h: number[];
  release_fraction: number[];
  /** Released over (M0 + generated); the fraction that a fresh particle with a source is plotted by. */
  release_fraction_of_total?: number[];
}
export interface StageProfile {
  t_s: number;
  t_h: number;
  c_mol_per_m3: number[];
  inventory_mol: number;
}
export interface StageOk {
  schema: 3;
  status: 'ok';
  label: ResultLabel;
  stage: { stage: number; id: string; name: string; adds: string; mechanisms: string[] };
  grid: { node_radii_m: number[]; interface_radii_m: number[]; layer_names: string[] };
  series: StageSeries;
  profiles: StageProfile[];
  analytical: { applicable: boolean; extension: string | null; reason_code: string | null; reason: string | null };
  units: Record<string, string>;
}
export interface StageRejected {
  schema: 3;
  status: 'rejected';
  errors: string[];
}
export type StageReply = StageOk | StageRejected;

/** The reply of analytical: Method B at the requested times, null where the mask withholds a value. */
export interface AnalyticalReply {
  schema?: number;
  applicable: boolean;
  extension: string | null;
  reason_code: string | null;
  reason: string | null;
  times_s: number[] | null;
  times_h: number[] | null;
  release_fraction: (number | null)[] | null;
  /** Method B released over (M0 + generated), present where the configuration has a source. */
  release_fraction_of_total?: (number | null)[] | null;
  drawn_count?: number;
}

export interface OptionsNuclide {
  id: string;
  name: string;
  label?: string;
  element?: string;
  main_comparison_temperatures_c?: number[];
}
export interface OptionsPhase {
  phase: string;
  temperatures_c: number[];
  validity_intersection_c: { min: number; max: number } | null;
}
/** The reply of options: the nuclides and temperatures the wrapper lists. */
export interface OptionsReply {
  schema?: number;
  nuclides: { scenario: OptionsNuclide[]; no_generation_source: OptionsNuclide[]; experiment: OptionsNuclide[] };
  temperatures: { main_comparison_c: Record<string, number[]>; exploration: { nuclide: string; name?: string; phases: OptionsPhase[] }[] };
  mechanisms: { id: string; term?: string; implemented: boolean; first_stage?: number }[];
}