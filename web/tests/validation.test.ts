import { describe, expect, it } from 'vitest';
import { cloneConfig, DEFAULT_CASE, type V1Config } from '../src/data/case';
import { parseNumber, validateInputs } from '../src/state/validation';

const base = () => cloneConfig(DEFAULT_CASE.config);
const paths = (c: V1Config) => validateInputs(c).map((i) => i.path);

describe('validateInputs mirrors Config::validate', () => {
  it('accepts the bundled case', () => {
    expect(validateInputs(base())).toEqual([]);
  });

  it('rejects a region count other than five', () => {
    const c = base();
    c.regions.pop();
    expect(paths(c)).toContain('regions');
  });

  it('rejects a blank species', () => {
    const c = base();
    c.species = '   ';
    expect(paths(c)).toEqual(['species']);
  });

  it.each([0, -1, Number.NaN, Number.POSITIVE_INFINITY])('rejects temperature %s', (t) => {
    const c = base();
    c.temperature_k = t;
    expect(paths(c)).toEqual(['temperature_k']);
  });

  it('rejects radii that do not increase', () => {
    const c = base();
    const r2 = c.regions[2];
    const r1 = c.regions[1];
    if (!r1 || !r2) throw new Error('fixture');
    r2.outer_radius_m = r1.outer_radius_m;
    expect(paths(c)).toEqual(['regions.2.outer_radius_m']);
  });

  it('rejects a non-positive first radius', () => {
    const c = base();
    const r0 = c.regions[0];
    if (!r0) throw new Error('fixture');
    r0.outer_radius_m = 0;
    expect(paths(c)).toEqual(['regions.0.outer_radius_m']);
  });

  it.each([0, -1e-12, Number.NaN])('rejects diffusivity %s', (d) => {
    const c = base();
    const r = c.regions[4];
    if (!r) throw new Error('fixture');
    r.diffusivity_m2_s = d;
    expect(paths(c)).toEqual(['regions.4.diffusivity_m2_s']);
  });

  it.each([0, 1.5, -2])('rejects cell count %s', (n) => {
    const c = base();
    const r = c.regions[1];
    if (!r) throw new Error('fixture');
    r.cells = n;
    expect(paths(c)).toEqual(['regions.1.cells']);
  });

  it('rejects invalid time controls', () => {
    const c = base();
    c.time.dt_s = 0;
    c.time.end_time_s = -1;
    c.time.output_every_steps = 0;
    expect(paths(c)).toEqual(['time.dt_s', 'time.end_time_s', 'time.output_every_steps']);
  });

  it('requires a positive start concentration (the solver needs a positive start inventory)', () => {
    const c = base();
    c.initial.kernel_concentration_mol_m3 = 0;
    expect(paths(c)).toEqual(['initial.kernel_concentration_mol_m3']);
  });
});

describe('parseNumber', () => {
  it.each([
    ['2e-4', 2e-4],
    ['2.0e-4', 2e-4],
    ['0.0002', 2e-4],
    [' 1E-12 ', 1e-12],
    ['200', 200],
    ['-3.5', -3.5],
    ['.5', 0.5],
    ['2 × 10^-4', 2e-4],
  ])('reads %s', (text, v) => {
    expect(parseNumber(text)).toBe(v);
  });

  it.each(['', 'abc', '1e', '1..2', '0x10', 'Infinity', '1e999'])('refuses %s', (text) => {
    expect(parseNumber(text)).toBeNull();
  });
});
