import { useId, useState } from 'react';
import { repoFile } from '../build';
import type { CaseEntry, V1Config } from '../data/case';
import { plain } from '../format';
import { useCatalogue } from '../research/catalogue';
import { linkCheckedDate } from '../research/entries';
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
  /** How the value is classed: the case file's own note, or "recorded only" for inputs v1.0 does not use. */
  status: string;
}

/** One row per input of the v1.0 format, from the case file and the current inputs (selection only). */
export function parameterRows(file: V1Config, current: V1Config, note: string): ParameterRow[] {
  // The case file's own note (in capitals in the file) is shown once under the table; each row says it briefly.
  const synthetic = note ? 'synthetic verification value, not literature TRISO data (case-file note)' : 'synthetic verification value';
  const out: ParameterRow[] = [
    { group: 'Case', parameter: 'Species', key: 'species', unit: '', file: file.species, current: current.species, status: 'label only' },
    {
      group: 'Case',
      parameter: 'Temperature',
      key: 'temperature_k',
      unit: 'K',
      file: file.temperature_k,
      current: current.temperature_k,
      status: 'recorded only; v1.0 physics does not use it',
    },
  ];
  file.regions.forEach((r, i) => {
    const c = current.regions[i];
    if (!c) return;
    out.push({ group: r.name, parameter: 'Outer radius', key: `regions[${i}].outer_radius_m`, unit: 'm', file: r.outer_radius_m, current: c.outer_radius_m, status: synthetic });
    out.push({ group: r.name, parameter: 'Diffusivity', key: `regions[${i}].diffusivity_m2_s`, unit: 'm² s⁻¹', file: r.diffusivity_m2_s, current: c.diffusivity_m2_s, status: synthetic });
    out.push({ group: r.name, parameter: 'Cells', key: `regions[${i}].cells`, unit: '', file: r.cells, current: c.cells, status: 'numerical setting' });
  });
  out.push({
    group: 'Start',
    parameter: 'Kernel concentration',
    key: 'initial.kernel_concentration_mol_m3',
    unit: 'mol m⁻³',
    file: file.initial.kernel_concentration_mol_m3,
    current: current.initial.kernel_concentration_mol_m3,
    status: synthetic,
  });
  out.push({ group: 'Time', parameter: 'Time step', key: 'time.dt_s', unit: 's', file: file.time.dt_s, current: current.time.dt_s, status: 'numerical setting' });
  out.push({ group: 'Time', parameter: 'End time', key: 'time.end_time_s', unit: 's', file: file.time.end_time_s, current: current.time.end_time_s, status: 'numerical setting' });
  out.push({
    group: 'Time',
    parameter: 'Output every',
    key: 'time.output_every_steps',
    unit: 'steps',
    file: file.time.output_every_steps,
    current: current.time.output_every_steps,
    status: 'numerical setting',
  });
  return out;
}

/** Rows whose text contains every word of the query (case-insensitive); display filter only. */
export function filterRows(rows: ParameterRow[], query: string): ParameterRow[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return rows;
  return rows.filter((r) => {
    const text = [r.group, r.parameter, r.key, r.unit, String(r.file), String(r.current), r.status].join(' ').toLowerCase();
    return words.every((w) => text.includes(w));
  });
}

const show = (v: number | string) => (typeof v === 'number' ? plain(v) : v);

export function ParameterProvenance({ entry }: { entry: CaseEntry }) {
  const config = useStrato((s) => s.config);
  const [query, setQuery] = useState('');
  const searchId = useId();
  const p = entry.provenance;
  const all = parameterRows(entry.config, config, p.notes.join(' '));
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
            <strong>Table 1.</strong> {entry.title}: every input of the run, the value in the case file and the value now set in the controls,
            with its status and source. Source of every row: <code>{p.input_file}</code> (SHA-256 <code className="hash">{p.input_sha256.slice(0, 12)}</code>,
            last changed in commit <code>{p.input_commit.slice(0, 7)}</code>). None of these values comes from the parameter register, so no row
            has a register id; the synthetic values were chosen for verification and are not measured TRISO properties.
          </caption>
          <thead>
            <tr>
              <th scope="col">Group</th>
              <th scope="col">Parameter</th>
              <th scope="col" className="num-col">
                Case file
              </th>
              <th scope="col" className="num-col">
                Current
              </th>
              <th scope="col">Unit</th>
              <th scope="col">Status</th>
              <th scope="col">Key in the case file</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => {
              const differs = !Object.is(r.file, r.current);
              return (
                <tr key={r.key} className={differs ? 'is-changed' : undefined} data-key={r.key}>
                  <td>{r.group}</td>
                  <th scope="row">{r.parameter}</th>
                  <td className="num-col num">{show(r.file)}</td>
                  <td className="num-col num">
                    {show(r.current)}
                    {differs ? <span className="changed-mark"> changed</span> : null}
                  </td>
                  <td>{r.unit}</td>
                  <td>{r.status}</td>
                  <td>
                    <code>{r.key}</code>
                  </td>
                </tr>
              );
            })}
            {rows.length === 0 ? (
              <tr>
                <td colSpan={7} className="muted">
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

/** The reference list: the key references of the README, and every reference of the research catalogue. */
export function References() {
  const cat = useCatalogue();
  return (
    <>
      <p>Key references of the STRATO model, as listed in the repository README.</p>
      <ul className="refs" data-testid="readme-refs">
        <li>
          Gelbard, F. (2003). <em>Analytical modeling of fission product releases by diffusion from multicoated fuel particles</em>{' '}
          (SAND2002-3966). Sandia National Laboratories.
        </li>
        <li>
          International Atomic Energy Agency. (1997). <em>Fuel performance and fission product behaviour in gas cooled reactors</em>{' '}
          (IAEA-TECDOC-978). IAEA.
        </li>
        <li>
          Hales, J. D., Jiang, W., Toptan, A., &amp; Gamble, K. A. (2021). Modeling fission product diffusion in TRISO fuel particles with
          BISON. <em>Journal of Nuclear Materials, 548</em>, 152840. <a href="https://doi.org/10.1016/j.jnucmat.2021.152840">doi:10.1016/j.jnucmat.2021.152840</a>
        </li>
      </ul>
      <p>
        References of the research catalogue (<code>data/research/references.csv</code>), each with the date its link was last opened:
      </p>
      <ul className="refs" data-testid="catalogue-refs">
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
        Literature parameters for Phase 2, each with its register status (verified, digitised or not verified) and source, are in the{' '}
        <a href={repoFile('data/reference/STRATO_Parameter_Register.md')}>parameter register</a>. v1.0 does not read them.
      </p>
    </>
  );
}
