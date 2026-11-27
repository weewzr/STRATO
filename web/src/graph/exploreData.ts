// What the graphs of Explore are drawn from, and the request that asks the wrapper for it. Two sources
// only: the reply of the wrapper (run_stage, analytical, options) and the generated results files of
// data/results (parameters.json, comparison/*.json), imported at build time. This module selects and
// arranges; it computes no scientific quantity. Inputs of a request (the registered diffusivity at a
// temperature, the decay constant, the yield, the radii) are the register values of parameters.json,
// verbatim.
import parametersFile from '../../../data/results/parameters.json';
import type { PhysicsId } from '../equation/capability';
import type { OptionsNuclide, OptionsReply } from '../solver/types';
import { arrheniusLaws, gasConstant, LAYER_REGIONS } from './arrhenius';

// ---- the generated files, typed for what the page reads

export interface DiffusivityAt {
  temperature_c: number;
  diffusivity_m2_s: number;
}
export interface DiffusivityRow {
  layer: string;
  register_rows: string[];
  published_range_c: { min: number; max: number } | [number, number] | null;
  diffusivity_m2_s_at_c: DiffusivityAt[];
}
export interface GenerationNuclide {
  nuclide: string;
  decay_constant_per_s: number;
  cumulative_yield_per_fission: number;
}
export interface ParametersFile {
  geometry: { design: string; outer_radii_m: number[]; layers: string[] };
  diffusivity: { element: string; heating_test_rows: DiffusivityRow[] }[];
  generation: { fission_rate_density_target: string; fission_rate_density_per_m3_s: number; label: string; nuclides: GenerationNuclide[] }[];
}
export const PARAMETERS = parametersFile as unknown as ParametersFile;

export interface ExperimentPoint {
  id: string;
  time_h: number;
  value: number;
  qualifier: string;
  point_class: string;
}
export interface ComparisonR2 {
  method: 'A' | 'B';
  status: string;
  label_plain: string | null;
}
export interface ComparisonRecord {
  curve_id: string;
  nuclide: string;
  experiment: { marker_colour: string; points: ExperimentPoint[] };
  model: {
    method_a: { time_h: number[]; release_fraction: number[] };
    method_b: { status: string; time_h?: number[]; release_fraction?: (number | null)[] };
  };
  r2_records: ComparisonR2[];
  caption: {
    caption_text: string;
    source_citation_apa: string;
    source_link: string;
    source_figure_table: string;
    temperature_label: string;
    specimen_label: string;
  };
}

const comparisonModules = import.meta.glob<unknown>('../../../data/results/comparison/*.json', { eager: true, import: 'default' });
export const COMPARISONS: readonly ComparisonRecord[] = Object.values(comparisonModules) as ComparisonRecord[];

// ---- species

/** The key under which a nuclide name, a wrapper id and a comparison file's nuclide agree: lower case without punctuation. */
export const nuclideKey = (name: string): string => name.toLowerCase().replace(/[^a-z0-9]/g, '');

export interface Species {
  id: string;
  name: string;
  /** The element symbol of the diffusivity register. */
  element: string;
}

/** The nuclides the wrapper lists (options: experiment and scenario nuclides, each once). Empty until the wrapper has answered. */
export function speciesFromOptions(options: OptionsReply | null): Species[] {
  if (!options) return [];
  const seen = new Set<string>();
  const out: Species[] = [];
  const add = (n: OptionsNuclide) => {
    const key = nuclideKey(n.name);
    if (seen.has(key)) return;
    seen.add(key);
    out.push({ id: n.id, name: n.name, element: n.element ?? n.name.split('-')[0] ?? n.name });
  };
  for (const n of options.nuclides.experiment) add(n);
  for (const n of options.nuclides.scenario) add(n);
  return out;
}

// ---- the temperatures

/** The multiples of 100 degC inside [min, max], ascending. Nothing else is ever offered. */
export function temperatureSteps(range: { min: number; max: number } | null): number[] {
  if (range === null || !(range.max >= range.min)) return [];
  const first = Math.ceil(range.min / 100) * 100;
  const out: number[] = [];
  for (let t = first; t <= range.max; t += 100) out.push(t);
  return out;
}

const rangeOf = (r: DiffusivityRow['published_range_c']): { min: number; max: number } | null => {
  if (r === null) return null;
  return Array.isArray(r) ? { min: r[0], max: r[1] } : r;
};

/** The heating-test diffusivity rows of an element (five layers), or null. */
export function heatingRows(element: string, params: ParametersFile = PARAMETERS): DiffusivityRow[] | null {
  return params.diffusivity.find((d) => d.element === element)?.heating_test_rows ?? null;
}

/**
 * The range of temperatures that the registered diffusivity rows of an element support, in degC: the
 * intersection over the five layers of the printed range of the row; where a row prints no range, the
 * span of the temperatures at which the register lists its diffusivity. Null when the element has no rows.
 */
export function registeredRange(element: string, params: ParametersFile = PARAMETERS): { min: number; max: number } | null {
  const rows = heatingRows(element, params);
  if (!rows || rows.length === 0) return null;
  let min = -Infinity;
  let max = Infinity;
  for (const row of rows) {
    const printed = rangeOf(row.published_range_c);
    const listed = row.diffusivity_m2_s_at_c.map((d) => d.temperature_c);
    if (printed === null && listed.length === 0) return null;
    const lo = printed ? printed.min : Math.min(...listed);
    const hi = printed ? printed.max : Math.max(...listed);
    min = Math.max(min, lo);
    max = Math.min(max, hi);
  }
  return { min, max };
}

/** The temperatures offered for an element: the 100 degC steps inside its registered range at which every layer has a registered diffusivity. */
export function offeredTemperatures(element: string, params: ParametersFile = PARAMETERS): number[] {
  const rows = heatingRows(element, params);
  if (!rows) return [];
  return temperatureSteps(registeredRange(element, params)).filter((t) => rows.every((r) => r.diffusivity_m2_s_at_c.some((d) => d.temperature_c === t)));
}

// ---- the request of run_stage

/** The numerical settings of the graphs: one grid and one time span for every request (fixed, not controls). */
export const GRID_INTERVALS = [160, 20, 20, 160, 20] as const;
export const END_TIME_H = 300;
export const END_TIME_S = END_TIME_H * 3600;
export const TIME_STEP_S = 1000;
export const OUTPUT_EVERY_STEPS = 10;
/** The times of the profiles asked for: a tenth, a half and the whole of the end time (an input of the request). */
export const PROFILE_TIMES_S = [END_TIME_S / 10, END_TIME_S / 2, END_TIME_S] as const;
/** The initial kernel concentration of a request without a source, mol/m^3: the stage runs report a fraction of the initial inventory, so only the ratio matters. */
export const KERNEL_CONCENTRATION_MOL_M3 = 1;
/** The initial kernel concentration of a request with a source: a fresh particle (C = 0), as in every operating scenario. */
export const FRESH_PARTICLE_CONCENTRATION_MOL_M3 = 0;

/** True where the model equation has a fission-product source: its graphs show the fraction of the generated inventory released (the fresh particle has no initial inventory). */
export const hasSource = (physics: PhysicsId): boolean => physics === 'generation' || physics === 'combined';
const KELVIN_OFFSET = 273.15;

/** The equation stage of run_stage that a model equation maps to (null: no stage, nothing is computed). */
export function stageOf(physics: PhysicsId): 1 | 2 | 3 | 4 | 5 | null {
  switch (physics) {
    case 'baseline':
      return 1;
    case 'generation':
      return 2;
    case 'decay':
      return 3;
    case 'temperature':
      return 4;
    case 'combined':
      return 5;
    default:
      return null;
  }
}

export interface StageRequest {
  stage: 1 | 2 | 3 | 4 | 5;
  /** The JSON of the request, as it is sent. */
  body: Record<string, unknown>;
  /** A stable key of the request, to drop a reply that arrives after the selection has changed. */
  key: string;
}

/**
 * The request of run_stage for a model equation, a species and a temperature in degC, or null when the
 * registers hold no input that the model equation needs for this species (then no curve exists and
 * nothing is substituted). Which model equation switches on which mechanism is the stage table of the
 * wrapper (wasm-interface.md section 10); a mechanism that the model equation leaves out is left out
 * of the physics table.
 */
export function stageRequest(physics: PhysicsId, species: Species, temperatureC: number, params: ParametersFile = PARAMETERS): StageRequest | null {
  const stage = stageOf(physics);
  if (stage === null) return null;
  const rows = heatingRows(species.element, params);
  if (!rows || rows.length !== LAYER_REGIONS.length) return null;
  const regions = [];
  for (let k = 0; k < rows.length; k++) {
    const d = rows[k]?.diffusivity_m2_s_at_c.find((x) => x.temperature_c === temperatureC);
    const radius = params.geometry.outer_radii_m[k];
    if (!d || radius === undefined) return null;
    regions.push({ name: LAYER_REGIONS[k], outer_radius_m: radius, diffusivity_m2_s: d.diffusivity_m2_s, intervals: GRID_INTERVALS[k] });
  }
  const generation = params.generation[0];
  const nuclide = generation?.nuclides.find((n) => nuclideKey(n.nuclide) === nuclideKey(species.name));
  const physicsBody: Record<string, unknown> = {};
  const withSource = hasSource(physics);
  const withDecay = physics === 'decay' || physics === 'combined';
  const withTemperature = physics === 'temperature' || physics === 'combined';
  if (withSource || withDecay) {
    if (!generation || !nuclide) return null;
  }
  if (withSource && generation && nuclide) {
    physicsBody.source = {
      mode: 'kernel_fission_yield',
      product: 'direct_fission_product',
      yield_kind: 'cumulative',
      yield_atoms_per_fission: nuclide.cumulative_yield_per_fission,
      fission_rate_origin: generation.label,
      fission_rate_density: { value: generation.fission_rate_density_per_m3_s },
    };
  }
  if (withDecay && nuclide) physicsBody.decay = { decay_constant_per_s: nuclide.decay_constant_per_s };
  if (withTemperature) {
    const laws = arrheniusLaws(species.element);
    if (!laws) return null;
    // The gas constant belongs to the temperature section of the physics table (the schema rejects it elsewhere).
    const gas = gasConstant(species.element);
    physicsBody.temperature = gas === null ? { unit: 'degC', value: temperatureC } : { unit: 'degC', value: temperatureC, gas_constant_j_mol_k: gas };
    physicsBody.arrhenius = laws;
  }
  const body: Record<string, unknown> = {
    schema: 3,
    config: {
      case_name: `${species.name}, ${String(temperatureC)} degC, stage ${String(stage)}`,
      species: species.name,
      temperature_k: temperatureC + KELVIN_OFFSET,
      regions,
      initial: { kernel_concentration_mol_m3: withSource ? FRESH_PARTICLE_CONCENTRATION_MOL_M3 : KERNEL_CONCENTRATION_MOL_M3 },
      time: { dt_s: TIME_STEP_S, end_time_s: END_TIME_S, output_every_steps: OUTPUT_EVERY_STEPS },
    },
    stage,
    profile_times_s: [...PROFILE_TIMES_S],
  };
  if (Object.keys(physicsBody).length > 0) body.physics = physicsBody;
  return { stage, body, key: JSON.stringify(body) };
}

// ---- experimental datasets

/** The comparison files of a species, in the order of the files. */
export function datasetsOf(species: Species | null, files: readonly ComparisonRecord[] = COMPARISONS): ComparisonRecord[] {
  if (!species) return [];
  return files.filter((f) => nuclideKey(f.nuclide) === nuclideKey(species.name));
}

/** The temperature in degC that a dataset's label states as one number, or null for a heating history that steps between temperatures. */
export const datasetTemperatureC = (f: ComparisonRecord): number | null => {
  const m = /^\s*(\d+(?:\.\d+)?)\s*\S?C\s*$/.exec(f.caption.temperature_label.replace(/[^\x20-\x7e]/g, ''));
  return m ? Number(m[1]) : null;
};

/** The datasets that belong to the selected temperature: those held at it, and those whose heating history steps through temperatures (their label says so). */
export function datasetsAt(list: readonly ComparisonRecord[], temperatureC: number | null): ComparisonRecord[] {
  return list.filter((f) => {
    const t = datasetTemperatureC(f);
    return t === null || temperatureC === null || t === temperatureC;
  });
}

/** The short name of a dataset in the select: the specimen and the temperature of the record's caption. */
export const datasetLabel = (f: ComparisonRecord): string => `${f.caption.specimen_label}, ${f.caption.temperature_label}`;
