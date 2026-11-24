// TypeScript mirror of the wrapper's JSON replies (crates/strato-wasm/src/lib.rs;
// prompt/TECHNICAL_ARCHITECTURE.md Section 3). Types only: nothing in this file computes a value.

export type ResultLabel = 'COMPUTED v1.0' | 'COMPUTED v2.0' | 'PRECOMPUTED' | 'MEASURED' | 'ILLUSTRATIVE';

export interface EngineStamp {
  strato_version: string;
  commit: string;
  abi: number;
}

export interface WrapperLimits {
  max_total_cells: number;
  max_steps: number;
  max_profile_values: number;
  max_series_modes: number;
  library_max_modes: number;
}

/** build_info(): versions, the stamped commit and the mechanism ids this build solves. */
export interface BuildInfo {
  schema: 1;
  abi: number;
  crate: string;
  wrapper_version: string;
  strato_version: string;
  commit: string;
  dirty: boolean;
  wasm_bindgen: string;
  label: ResultLabel;
  capabilities: string[];
  limits: WrapperLimits;
}

export interface RunProfile {
  t_s: number;
  c_mol_per_m3: number[];
  inventory_mol: number;
  layer_inventory_mol: number[];
  layer_share: number[] | null;
}

export interface Analytical {
  times_s: number[];
  release_fraction: number[];
  accuracy_floor: number;
  valid_from_s: number;
  modes: number;
}

/** The v1.0 inventory balance: inventory + released - start inventory = residual ((3.9)). */
export interface BalanceV1 {
  form: 'v1.0';
  statement: string;
  source: string;
  start_inventory_mol: number;
  final: { t_s: number; inventory_mol: number; released_mol: number; residual_mol: number };
  max_abs_residual_mol: number;
  max_abs_residual_relative: number;
}

export interface CodeCDiagnostics {
  modes: number;
  max_abs_difference: number;
  final_abs_difference: number;
  accuracy_floor: number;
  valid_from_s: number;
  min_flux_ratio: number;
  resolved: boolean;
}

export interface Diagnostics {
  cells: number;
  regions: number;
  steps: number;
  dt_requested_s: number;
  dt_used_s: number;
  end_time_s: number;
  output_rows: number;
  explicit_dt_bound_s: number;
  final_release_fraction: number;
  max_abs_balance_residual_mol: number;
  max_abs_balance_residual_relative: number;
  min_concentration_mol_m3: number;
  layer_sum_max_relative_difference: number;
  code_c: CodeCDiagnostics | null;
}

export interface RunOk {
  schema: 1;
  status: 'ok';
  label: ResultLabel;
  engine: EngineStamp;
  mechanisms: string[];
  inputs: unknown;
  mesh: { cell_centroids_m: number[]; cell_faces_m: number[]; region_of_cell: number[]; region_names: string[] };
  times_s: number[];
  release_fraction: number[];
  inventory_mol: number[];
  cumulative_release_mol: number[];
  release_rate_mol_s: number[];
  balance_residual_mol: number[];
  min_concentration_mol_m3: number[];
  profiles: RunProfile[];
  balance: BalanceV1;
  diagnostics: Diagnostics;
  analytical: Analytical | null;
  messages: string[];
}

export interface Rejected {
  schema: 1;
  status: 'rejected';
  errors: string[];
  engine?: EngineStamp;
}

export type RunResponse = RunOk;
export type RunReply = RunOk | Rejected;

export interface ValidateReply {
  schema: 1;
  ok: boolean;
  errors: string[];
  warnings: string[];
  plan: { cells: number; steps: number; dt_used_s: number; output_rows: number } | null;
  limits: WrapperLimits;
}
