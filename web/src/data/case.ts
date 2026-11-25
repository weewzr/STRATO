// Input cases available to the demo. The values come from the generated JSON files written by
// strato-webtools case from the TOML inputs in inputs/; none is typed here.
import syntheticCase from '../generated/synthetic_five_layer.case.json';

/** Mirrors strato::config::RegionConfig (src/config.rs at v1.0.1). */
export interface RegionConfig {
  name: string;
  outer_radius_m: number;
  diffusivity_m2_s: number;
  cells: number;
}

/** Mirrors strato::config::Config (src/config.rs at v1.0.1). Every field is required by serde. */
export interface V1Config {
  case_name: string;
  species: string;
  temperature_k: number;
  regions: RegionConfig[];
  initial: { kernel_concentration_mol_m3: number };
  time: { dt_s: number; end_time_s: number; output_every_steps: number };
  output: { directory: string };
}

export interface CaseProvenance {
  label: string;
  generator: string;
  strato_version: string;
  input_file: string;
  input_sha256: string;
  input_commit: string;
  command: string;
  notes: string[];
  units: Record<string, string>;
}

export interface CaseEntry {
  id: string;
  title: string;
  /** Path of the downloadable CSV copy, relative to the site base. */
  csvPath: string;
  provenance: CaseProvenance;
  config: V1Config;
}

function entry(id: string, title: string, doc: typeof syntheticCase): CaseEntry {
  const { data, ...header } = doc;
  return {
    id,
    title,
    csvPath: `data/${id}.case.csv`,
    provenance: {
      label: header.label,
      generator: header.generator,
      strato_version: header.strato_version,
      input_file: header.input_file,
      input_sha256: header.input_sha256,
      input_commit: header.input_commit,
      command: header.command,
      notes: header.notes,
      units: header.units,
    },
    config: data.config,
  };
}

export const CASES: readonly CaseEntry[] = [
  entry('synthetic_five_layer', 'Synthetic five-layer verification case', syntheticCase),
];

export const DEFAULT_CASE: CaseEntry = CASES[0] as CaseEntry;

export function findCase(id: string): CaseEntry | undefined {
  return CASES.find((c) => c.id === id);
}

/** Deep copy of a configuration (plain data, so structuredClone is exact). */
export function cloneConfig(cfg: V1Config): V1Config {
  return structuredClone(cfg);
}
