// Writes web/src/generated/arrhenius_heating.json: the registered Arrhenius laws of the heating-test set,
// read from data/diffusivity/arrhenius.csv for the register rows that data/results/parameters.json names for
// each element and layer (the rows behind the diffusivities that the results files use). Nothing is typed by
// hand: the numbers are the register's text. tests/explore_data.test.ts runs `build()` and requires the
// committed file to equal it, so a change of the register that is not regenerated fails the test.
// Usage (from web/): node scripts/arrhenius_heating.mjs
import { readFileSync, writeFileSync } from 'node:fs';

// The script runs from web/, as does the unit test that calls build().
const root = `${process.cwd().split('\\').join('/')}/../`;

/** Minimal CSV reader with quoted fields (the register has commas inside quoted notes). */
export function parseCsv(text) {
  const rows = [];
  let row = [];
  let cell = '';
  let quoted = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (quoted) {
      if (c === '"' && text[i + 1] === '"') {
        cell += '"';
        i++;
      } else if (c === '"') quoted = false;
      else cell += c;
    } else if (c === '"') quoted = true;
    else if (c === ',') {
      row.push(cell);
      cell = '';
    } else if (c === '\n') {
      row.push(cell.replace(/\r$/, ''));
      rows.push(row);
      row = [];
      cell = '';
    } else cell += c;
  }
  if (cell !== '' || row.length > 0) {
    row.push(cell);
    rows.push(row);
  }
  const [head, ...body] = rows;
  return body.filter((r) => r.length === head.length).map((r) => Object.fromEntries(head.map((h, i) => [h, r[i]])));
}

const LAYERS = ['kernel', 'buffer', 'IPyC', 'SiC', 'OPyC'];

export function build() {
  const csv = parseCsv(readFileSync(`${root}data/diffusivity/arrhenius.csv`, 'utf8'));
  const params = JSON.parse(readFileSync(`${root}data/results/parameters.json`, 'utf8'));
  const elements = {};
  for (const d of params.diffusivity) {
    const layers = {};
    for (const layer of LAYERS) {
      const row = d.heating_test_rows.find((r) => r.layer === layer);
      if (!row) throw new Error(`no heating-test row for ${d.element} ${layer}`);
      layers[layer] = row.register_rows.map((id) => {
        const r = csv.find((x) => x.id === id);
        if (!r) throw new Error(`register row ${id} is not in arrhenius.csv`);
        return {
          id,
          d0_m2_s: Number(r.D0_m2_s),
          activation_energy_kj_per_mol: Number(r.Q_kJ_mol),
          gas_constant_j_mol_k: Number(r.R_J_mol_K),
          class: r.coefficient_class,
          source: r.source,
          phase_of_use: r.phase_of_use,
        };
      });
    }
    elements[d.element] = layers;
  }
  return { what: 'registered Arrhenius laws of the heating-test set, from data/diffusivity/arrhenius.csv', generator: 'web/scripts/arrhenius_heating.mjs', elements };
}

if ((process.argv[1] ?? '').split('\\').join('/').endsWith('scripts/arrhenius_heating.mjs')) {
  writeFileSync(`${process.cwd()}/src/generated/arrhenius_heating.json`, `${JSON.stringify(build(), null, 2)}\n`, 'utf8');
}
