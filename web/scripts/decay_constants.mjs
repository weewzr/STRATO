// Writes web/src/generated/decay_constants.json: the registered decay constant of every unstable nuclide of
// data/nuclear/decay.csv whose row status is "verified", copied as the register's own text (lambda_s-1 and its
// half-life), nothing computed. The page needs a decay constant for the species it offers; the generation table of
// data/results/parameters.json holds one only for nuclides that have a fission-yield source (Sr-90, Cs-137), but
// the register holds verified constants for the activation products Ag-110m and Cs-134 as well, and a decay
// curve needs only the constant. tests/explore_data.test.ts runs `build()` and requires the committed file to
// equal it, so a change of the register that is not regenerated fails the test.
// Usage (from web/): node scripts/decay_constants.mjs
import { readFileSync, writeFileSync } from 'node:fs';
import { parseCsv } from './arrhenius_heating.mjs';

// The script runs from web/, as does the unit test that calls build().
const root = `${process.cwd().split('\\').join('/')}/../`;

export function build() {
  const csv = parseCsv(readFileSync(`${root}data/nuclear/decay.csv`, 'utf8'));
  const nuclides = {};
  for (const r of csv) {
    if (r.status !== 'verified' || r.stable === 'yes') continue;
    const lambda = Number(r['lambda_s-1']);
    if (!(lambda > 0)) continue;
    nuclides[r.nuclide] = { decay_constant_per_s: lambda, half_life: Number(r.half_life), half_life_unit: r.unit, source: r.source };
  }
  return { what: 'verified decay constants of the nuclides of data/nuclear/decay.csv', generator: 'web/scripts/decay_constants.mjs', nuclides };
}

if ((process.argv[1] ?? '').split('\\').join('/').endsWith('scripts/decay_constants.mjs')) {
  writeFileSync(`${process.cwd()}/src/generated/decay_constants.json`, `${JSON.stringify(build(), null, 2)}\n`, 'utf8');
}
