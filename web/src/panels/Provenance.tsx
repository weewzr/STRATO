import { useId, useState } from 'react';
import { repoFile } from '../build';
import type { CaseEntry, CaseConfig } from '../data/case';
import { plain } from '../format';
import { useCatalogue } from '../research/catalogue';
import { linkCheckedDate } from '../research/entries';
import { hrefDocumentation } from '../route';
import { useStrato } from '../state/store';
import { TableBox } from '../sections/common';

// Provenance of every parameter in use and the reference list (web phase 5;
// prompt/DESIGN_SPECIFICATION.md Section 8.4). The values are the case file's (through the generated
// case JSON) and the inputs now set; the search filters the rows for display only.

export interface ParameterRow {
  group: string;
  parameter: string;
  key: string;
  unit: string;
  file: number | string;
  current: number | string;
  /** Source row of the value: the case file, or "recorded only" for an input the solver does not use. */
  source: string;
  /** Class of the value (coordinator decisions 16 and 31). */
  status: string;
}

/** One row per input of the solver, from the case file and the current inputs (selection only). */
export function parameterRows(file: CaseConfig, current: CaseConfig, input: string): ParameterRow[] {
  // Every input of this case is a synthetic verification value: its class says so, and the case file
  // is its source row. A numerical setting (grid, time step) is a choice of the run, not a material value.
  const synthetic = 'synthetic benchmark, not literature TRISO data';
  const setting = 'numerical setting of the run';
  const out: ParameterRow[] = [
    { group: 'Case', parameter: 'Species', key: 'species', unit: '', file: file.species, current: current.species, source: input, status: 'label only' },
    {
      group: 'Case',
      parameter: 'Temperature',
      key: 'temperature_k',
      unit: 'K',
      file: file.temperature_k,
      current: current.temperature_k,
      source: input,
      status: 'recorded only; the solver does not use it',
    },
  ];
  file.regions.forEach((r, i) => {
    const c = current.regions[i];
    if (!c) return;
    out.push({ group: r.name, parameter: `${r.name} outer radius`, key: `regions[${i}].outer_radius_m`, unit: 'm', file: r.outer_radius_m, current: c.outer_radius_m, source: input, status: synthetic });
    out.push({ group: r.name, parameter: `${r.name} diffusivity`, key: `regions[${i}].diffusivity_m2_s`, unit: 'm²/s', file: r.diffusivity_m2_s, current: c.diffusivity_m2_s, source: input, status: synthetic });
    out.push({ group: r.name, parameter: `Grid intervals in the ${r.name} layer`, key: `regions[${i}].cells`, unit: 'dimensionless', file: r.cells, current: c.cells, source: input, status: setting });
  });
  out.push({
    group: 'Start',
    parameter: 'Initial kernel concentration',
    key: 'initial.kernel_concentration_mol_m3',
    unit: 'mol/m³',
    file: file.initial.kernel_concentration_mol_m3,
    current: current.initial.kernel_concentration_mol_m3,
    source: input,
    status: synthetic,
  });
  out.push({ group: 'Time', parameter: 'Time step', key: 'time.dt_s', unit: 's', file: file.time.dt_s, current: current.time.dt_s, source: input, status: setting });
  out.push({ group: 'Time', parameter: 'Simulation duration', key: 'time.end_time_s', unit: 's', file: file.time.end_time_s, current: current.time.end_time_s, source: input, status: setting });
  out.push({
    group: 'Time',
    parameter: 'Time steps between output rows',
    key: 'time.output_every_steps',
    unit: 'dimensionless',
    file: file.time.output_every_steps,
    current: current.time.output_every_steps,
    source: input,
    status: setting,
  });
  return out;
}
/** Rows whose text contains every word of the query (case-insensitive); display filter only. */
export function filterRows(rows: ParameterRow[], query: string): ParameterRow[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return rows;
  return rows.filter((r) => {
    const text = [r.group, r.parameter, r.key, r.unit, String(r.file), String(r.current), r.source, r.status].join(' ').toLowerCase();
    return words.every((w) => text.includes(w));
  });
}

const show = (v: number | string) => (typeof v === 'number' ? plain(v) : v);

/** The class of a value as one short word (the full class stays in the row, so the search finds it). */
export function shortClass(status: string): string {
  if (status.startsWith('synthetic')) return 'synthetic';
  if (status.startsWith('numerical setting')) return 'setting';
  if (status.startsWith('recorded only')) return 'recorded only';
  return status;
}

export function ParameterProvenance({ entry }: { entry: CaseEntry }) {
  const config = useStrato((s) => s.config);
  const [query, setQuery] = useState('');
  const searchId = useId();
  const p = entry.provenance;
  const all = parameterRows(entry.config, config, p.input_file);
  const rows = filterRows(all, query);
  return (
    <>
      <div className="provenance-search">
        <label htmlFor={searchId}>Search the parameters</label>
        <input
          id={searchId}
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="for example SiC, diffusivity or time"
          data-testid="provenance-search"
        />
        <span className="muted" role="status" data-testid="provenance-count">
          {rows.length} of {all.length} rows
        </span>
      </div>
      <TableBox label="Parameters in use and their provenance">
        <table className="table" data-testid="input-table">
          <caption className="table__caption">
            <strong>Table 1.</strong> {entry.title}: case-file and current value of every input.{' '}
            <a href={hrefDocumentation('data-case-file')}>Provenance</a>
          </caption>
          <thead>
            <tr>
              <th scope="col">Parameter</th>
              <th scope="col" className="num-col">
                Case file
              </th>
              <th scope="col" className="num-col">
                Current
              </th>
              <th scope="col">Unit</th>
              <th scope="col">Class</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => {
              const differs = !Object.is(r.file, r.current);
              return (
                <tr key={r.key} className={differs ? 'is-changed' : undefined} data-key={r.key}>
                  <th scope="row">{r.parameter}</th>
                  <td className="num-col num">{show(r.file)}</td>
                  <td className="num-col num">
                    {show(r.current)}
                    {differs ? <span className="changed-mark"> changed</span> : null}
                  </td>
                  <td>{r.unit}</td>
                  <td>{shortClass(r.status)}</td>
                </tr>
              );
            })}
            {rows.length === 0 ? (
              <tr>
                <td colSpan={5}className="muted">
                  No parameter matches the search.
                </td>
              </tr>
            ) : null}
          </tbody>
        </table>
      </TableBox>
    </>
  );
}

/**
 * One reference list: the key references of the README that the research catalogue does not hold (Gelbard 2003
 * and IAEA 1997 are catalogue entries, with their links, and are shown once there), then every reference of
 * the research catalogue.
 */
export function References() {
  const cat = useCatalogue();
  return (
    <>
      <ul className="refs" data-testid="catalogue-refs">
        <li data-testid="readme-ref">
          Hales, J. D., Jiang, W., Toptan, A., &amp; Gamble, K. A. (2021). Modeling fission product diffusion in TRISO fuel particles with
          BISON. <em>Journal of Nuclear Materials, 548</em>, 152840. <a href="https://doi.org/10.1016/j.jnucmat.2021.152840">doi:10.1016/j.jnucmat.2021.152840</a>
        </li>
        {cat.references.map((r) => (
          <li key={r.reference_id} data-ref={r.reference_id}>
            {r.citation}{' '}
            {/^https?:\/\//.test(r.doi_or_url) ? <a href={r.doi_or_url}>{r.doi_or_url}</a> : <span className="muted">{r.doi_or_url || 'no link'}</span>}
            {linkCheckedDate(r.link_checked) ? (
              <span className="muted"> (link checked {r.link_checked})</span>
            ) : (
              <span className="muted"> (link: {r.link_checked || 'not checked'})</span>
            )}
          </li>
        ))}
      </ul>
      <p>
        <a href={repoFile('data/reference/STRATO_Parameter_Register.md')}>Parameter register</a>
      </p>
    </>
  );
}
