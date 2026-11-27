// The data layer of the graphs of Explore: the temperatures offered (the 100 degC steps inside the registered
// range, nothing else), the species the wrapper lists, the request of run_stage for each model equation, the
// registered Arrhenius laws (read from the register, not typed), and the experimental datasets.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { PHYSICS_IDS } from '../src/equation/capability';
import { arrheniusLaws, gasConstant } from '../src/graph/arrhenius';
import {
  COMPARISONS,
  datasetLabel,
  datasetsOf,
  heatingRows,
  nuclideKey,
  offeredTemperatures,
  PARAMETERS,
  registeredRange,
  speciesFromOptions,
  stageOf,
  stageRequest,
  temperatureSteps,
  type ParametersFile,
  type Species,
} from '../src/graph/exploreData';
import lawsFile from '../src/generated/arrhenius_heating.json';
import type { OptionsReply } from '../src/solver/types';
import { build } from '../scripts/arrhenius_heating.mjs';

const SR: Species = { id: 'sr90', name: 'Sr-90', element: 'Sr' };
const CS: Species = { id: 'cs137', name: 'Cs-137', element: 'Cs' };
const AG: Species = { id: 'ag110m', name: 'Ag-110m', element: 'Ag' };

const options = (experiment: string[], scenario: string[]): OptionsReply => ({
  nuclides: {
    experiment: experiment.map((name) => ({ id: nuclideKey(name), name })),
    scenario: scenario.map((name) => ({ id: nuclideKey(name), name })),
    no_generation_source: [],
  },
  temperatures: { main_comparison_c: {}, exploration: [] },
  mechanisms: [],
});

describe('the temperatures offered', () => {
  it('are the multiples of 100 degC inside the range, ascending, and nothing else', () => {
    expect(temperatureSteps({ min: 1600, max: 1800 })).toEqual([1600, 1700, 1800]);
    expect(temperatureSteps({ min: 1650, max: 1850 })).toEqual([1700, 1800]);
    expect(temperatureSteps({ min: 1601, max: 1699 })).toEqual([]);
    expect(temperatureSteps({ min: 955, max: 1197 })).toEqual([1000, 1100]);
    expect(temperatureSteps({ min: 1700, max: 1700 })).toEqual([1700]);
    expect(temperatureSteps(null)).toEqual([]);
    expect(temperatureSteps({ min: 1800, max: 1600 })).toEqual([]);
    for (const t of temperatureSteps({ min: 400, max: 1800 })) expect(t % 100).toBe(0);
  });

  it('follow the registered diffusivity rows of the species: the intersection of the layers, at the temperatures the register lists', () => {
    for (const element of ['Ag', 'Cs', 'Sr']) {
      const rows = heatingRows(element);
      expect(rows, element).not.toBeNull();
      expect(rows).toHaveLength(5);
      const listed = rows?.map((r) => r.diffusivity_m2_s_at_c.map((d) => d.temperature_c)) ?? [];
      const range = registeredRange(element);
      expect(range, element).toEqual({ min: Math.max(...listed.map((l) => Math.min(...l))), max: Math.min(...listed.map((l) => Math.max(...l))) });
      expect(offeredTemperatures(element), element).toEqual(temperatureSteps(range));
      expect(offeredTemperatures(element), element).toEqual([1600, 1700, 1800]);
    }
    expect(offeredTemperatures('Eu')).toEqual([]);
    expect(registeredRange('Eu')).toBeNull();
  });

  it('use a printed range where the register prints one, and offer only temperatures at which every layer has a registered diffusivity', () => {
    const rows = (range: { min: number; max: number } | null, temps: number[]) =>
      ['kernel', 'buffer', 'IPyC', 'SiC', 'OPyC'].map((layer) => ({ layer, register_rows: [], published_range_c: range, diffusivity_m2_s_at_c: temps.map((t) => ({ temperature_c: t, diffusivity_m2_s: 1e-12 })) }));
    const params = (r: ReturnType<typeof rows>): ParametersFile => ({ ...PARAMETERS, diffusivity: [{ element: 'X', heating_test_rows: r }] });
    expect(registeredRange('X', params(rows({ min: 1000, max: 1500 }, [1000, 1100, 1500])))).toEqual({ min: 1000, max: 1500 });
    expect(offeredTemperatures('X', params(rows({ min: 1000, max: 1500 }, [1000, 1100, 1500])))).toEqual([1000, 1100, 1500]);
    // 1200, 1300 and 1400 lie inside the range, but the register lists no diffusivity there: they are not offered.
    expect(offeredTemperatures('X', params(rows({ min: 1000, max: 1500 }, [1000, 1100, 1450, 1500])))).toEqual([1000, 1100, 1500]);
  });
});

describe('the species', () => {
  it('are the nuclides the wrapper lists, each once, and none before the wrapper has answered', () => {
    expect(speciesFromOptions(null)).toEqual([]);
    const s = speciesFromOptions(options(['Ag-110m', 'Cs-134', 'Sr-90'], ['Sr-90', 'Cs-137']));
    expect(s.map((x) => x.name)).toEqual(['Ag-110m', 'Cs-134', 'Sr-90', 'Cs-137']);
    expect(s.map((x) => x.element)).toEqual(['Ag', 'Cs', 'Sr', 'Cs']);
  });
});

describe('the registered laws', () => {
  it('equal the register: the committed file is what scripts/arrhenius_heating.mjs writes from data/diffusivity/arrhenius.csv', () => {
    expect(lawsFile).toEqual(build());
    const csv = readFileSync(join(process.cwd(), '..', 'data', 'diffusivity', 'arrhenius.csv'), 'utf8');
    expect(csv).toContain('C16-01,C16-T2,Ag,kernel');
  });

  it('give five laws, one per layer in the order of the particle, with their register rows as the source', () => {
    for (const element of ['Ag', 'Cs', 'Sr']) {
      const laws = arrheniusLaws(element) as { region: string; source: string; class: string; terms: unknown[] }[];
      expect(laws.map((l) => l.region)).toEqual(['Kernel', 'Buffer', 'IPyC', 'SiC', 'OPyC']);
      for (const l of laws) {
        expect(l.source).toMatch(/^C16-\d\d(, C16-\d\d)?$/);
        expect(l.terms.length).toBeGreaterThan(0);
        expect(l.class).toBe('published PARFUME correlation');
      }
      expect(gasConstant(element)).toBe(laws.length > 0 ? gasConstant('Ag') : null);
    }
    expect(arrheniusLaws('Eu')).toBeNull();
    // The two-term laws of the register are two terms of one law.
    const cs = arrheniusLaws('Cs') as { terms: unknown[] }[];
    expect(cs.map((l) => l.terms.length)).toEqual([2, 1, 1, 2, 1]);
  });
});

describe('the request of run_stage', () => {
  const body = (p: (typeof PHYSICS_IDS)[number], sp: Species, t = 1600) => stageRequest(p, sp, t)?.body;

  it('maps each model equation to a stage of the wrapper and Future species-specific physics to none', () => {
    expect(PHYSICS_IDS.map((p) => stageOf(p))).toEqual([1, 2, 3, 4, 5, null]);
    expect(stageRequest('future', SR, 1600)).toBeNull();
  });

  it('switches on exactly the mechanisms of the model equation, and gives each the registered inputs', () => {
    const phys = (p: (typeof PHYSICS_IDS)[number]) => (body(p, SR)?.physics ?? {}) as Record<string, unknown>;
    expect(body('baseline', SR)?.physics).toBeUndefined();
    expect(Object.keys(phys('generation'))).toEqual(['source']);
    expect(Object.keys(phys('decay'))).toEqual(['decay']);
    expect(Object.keys(phys('temperature')).sort()).toEqual(['arrhenius', 'gas_constant_j_mol_k', 'temperature']);
    expect(Object.keys(phys('combined')).sort()).toEqual(['arrhenius', 'decay', 'gas_constant_j_mol_k', 'source', 'temperature']);
    const nuclide = PARAMETERS.generation[0]?.nuclides.find((n) => n.nuclide === 'Sr-90');
    expect((phys('decay').decay as { decay_constant_per_s: number }).decay_constant_per_s).toBe(nuclide?.decay_constant_per_s);
    expect((phys('generation').source as { yield_atoms_per_fission: number }).yield_atoms_per_fission).toBe(nuclide?.cumulative_yield_per_fission);
    expect((phys('temperature').temperature as { unit: string; value: number })).toEqual({ unit: 'degC', value: 1600 });
    expect(body('combined', SR)?.stage).toBe(5);
    expect(body('baseline', SR)?.stage).toBe(1);
    expect(body('baseline', SR)?.schema).toBe(3);
  });

  it('takes the radii and the diffusivities of the five layers from the results file, at the selected temperature', () => {
    for (const t of [1600, 1700, 1800]) {
      const cfg = body('baseline', CS, t)?.config as { regions: { outer_radius_m: number; diffusivity_m2_s: number; intervals: number }[]; temperature_k: number };
      expect(cfg.regions.map((r) => r.outer_radius_m)).toEqual(PARAMETERS.geometry.outer_radii_m);
      const rows = heatingRows('Cs') ?? [];
      expect(cfg.regions.map((r) => r.diffusivity_m2_s)).toEqual(rows.map((r) => r.diffusivity_m2_s_at_c.find((d) => d.temperature_c === t)?.diffusivity_m2_s));
      expect(cfg.regions).toHaveLength(5);
      expect(cfg.temperature_k).toBeCloseTo(t + 273.15, 9);
    }
  });

  it('gives no request where the registers hold no input for the selection: no temperature outside the listed ones, no source for an activation product', () => {
    expect(stageRequest('baseline', SR, 1650)).toBeNull();
    expect(stageRequest('baseline', { id: 'eu154', name: 'Eu-154', element: 'Eu' }, 1600)).toBeNull();
    // Ag-110m and Cs-134 have no fission-yield source and no decay constant in the results file.
    expect(stageRequest('generation', AG, 1600)).toBeNull();
    expect(stageRequest('decay', AG, 1600)).toBeNull();
    expect(stageRequest('combined', AG, 1600)).toBeNull();
    expect(stageRequest('baseline', AG, 1600)).not.toBeNull();
    expect(stageRequest('temperature', AG, 1600)).not.toBeNull();
  });

  it('has a key that changes with the selection, so that a reply for another selection can be told apart', () => {
    const keys = new Set([stageRequest('baseline', SR, 1600)?.key, stageRequest('baseline', SR, 1700)?.key, stageRequest('baseline', CS, 1600)?.key, stageRequest('decay', SR, 1600)?.key]);
    expect(keys.size).toBe(4);
  });
});

describe('the experimental datasets', () => {
  it('are the files of data/results/comparison, each with its caption, its citation and its link', () => {
    expect(COMPARISONS).toHaveLength(15);
    for (const f of COMPARISONS) {
      expect(f.caption.caption_text.length, f.curve_id).toBeGreaterThan(10);
      expect(f.caption.source_citation_apa, f.curve_id).toMatch(/\(\d{4}\)/);
      expect(f.caption.source_link, f.curve_id).toMatch(/^https?:\/\//);
      expect(f.experiment.points.length, f.curve_id).toBeGreaterThan(0);
      expect(f.model.method_a.time_h.length, f.curve_id).toBe(f.model.method_a.release_fraction.length);
    }
  });

  it('are listed for the species they belong to, and for none other', () => {
    const names = (sp: Species) => datasetsOf(sp).map((d) => d.nuclide);
    expect(new Set(names(SR))).toEqual(new Set(['Sr-90']));
    expect(datasetsOf(SR).length).toBeGreaterThan(2);
    expect(datasetsOf(null)).toEqual([]);
    expect(datasetsOf({ id: 'xe133', name: 'Xe-133', element: 'Xe' })).toEqual([]);
    expect(datasetLabel(datasetsOf(CS)[0] ?? COMPARISONS[0]!)).toMatch(/°C/);
  });
});
