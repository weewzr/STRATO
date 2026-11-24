// TypeScript mirror of the wrapper's JSON replies (prompt/TECHNICAL_ARCHITECTURE.md Section 3).
// Types only: the wrapper crate crates/strato-wasm and its worker arrive in web phase 3, and
// nothing in this file computes a value.

export type ResultLabel = 'COMPUTED v1.0' | 'COMPUTED v2.0' | 'PRECOMPUTED' | 'MEASURED' | 'ILLUSTRATIVE';

export interface RunProfile {
  t_s: number;
  c_mol_per_m3: number[];
  inventory_mol: number;
  layer_inventory_mol: number[];
  layer_share: number[] | null;
}

export interface RunResponse {
  schema: 1;
  status: 'ok' | 'rejected';
  label: ResultLabel;
  engine: { strato_version: string; commit: string; abi: number };
  inputs: unknown;
  mesh: { cell_centroids_m: number[]; cell_faces_m: number[]; region_of_cell: number[] };
  times_s: number[];
  release_fraction: number[];
  balance_residual_mol: number[];
  profiles: RunProfile[];
  analytical: { times_s: number[]; release_fraction: number[]; accuracy_floor: number; valid_from_s: number } | null;
  messages: string[];
  errors?: string[];
}
