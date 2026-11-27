// The registered Arrhenius laws of the heating-test set, as the request of run_stage needs them:
// D = sum over the terms of a layer of d0 exp(-Q/(R T)), computed by the wrapper. The numbers are read from
// src/generated/arrhenius_heating.json, which web/scripts/arrhenius_heating.mjs writes from the register
// (data/diffusivity/arrhenius.csv) for the register rows that the results files name; nothing is typed here.
import lawsFile from '../generated/arrhenius_heating.json';

export const LAYER_REGIONS = ['Kernel', 'Buffer', 'IPyC', 'SiC', 'OPyC'] as const;
export type LayerRegion = (typeof LAYER_REGIONS)[number];

/** The keys of the register's layers, in the order of LAYER_REGIONS. */
const LAYER_KEYS = ['kernel', 'buffer', 'IPyC', 'SiC', 'OPyC'] as const;

export interface RegisterTerm {
  id: string;
  d0_m2_s: number;
  activation_energy_kj_per_mol: number;
  gas_constant_j_mol_k: number;
  class: string;
  source: string;
}

type LawsFile = { elements: Record<string, Record<(typeof LAYER_KEYS)[number], RegisterTerm[]>> };
const LAWS = lawsFile as unknown as LawsFile;

/** The gas constant of the register rows of an element (the same in every row used), or null without rows. */
export function gasConstant(element: string): number | null {
  return LAWS.elements[element]?.kernel[0]?.gas_constant_j_mol_k ?? null;
}

/** The `physics.arrhenius` list of a run_stage request for an element, or null when the register has no heating-test rows for it. */
export function arrheniusLaws(element: string): unknown[] | null {
  const laws = LAWS.elements[element];
  if (!laws) return null;
  return LAYER_REGIONS.map((region, k) => {
    const terms = laws[LAYER_KEYS[k] ?? 'kernel'];
    return {
      region,
      source: terms.map((t) => t.id).join(', '),
      class: terms[0]?.class ?? '',
      terms: terms.map((t) => ({ d0_m2_s: t.d0_m2_s, activation_energy_kj_per_mol: t.activation_energy_kj_per_mol })),
    };
  });
}
