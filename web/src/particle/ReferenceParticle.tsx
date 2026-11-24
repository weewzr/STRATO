import { useId, useState } from 'react';
import { repoFile } from '../build';
import { layerColour } from '../design/layers';
import reference from '../generated/agr1_reference.json';
import { ringsOf } from './geometry';

// Reference fuel: AGR-1 UCO TRISO (rule E2; prompt/DESIGN_SPECIFICATION.md Sections 4.1 and 7.7).
// A second, geometry-only view: the particle drawn to scale from the registered dimensions in
// data/geometry/geometry.csv, through web/src/generated/agr1_reference.json (written by
// scripts/web/generate_data.py geometry), with the row id, register status and source of every
// value. It is not a simulation, carries no concentration and is never combined with the
// diffusivities of the synthetic case. It uses no result badge: the values are register entries.

type RefSet = (typeof reference.data.sets)[number];

const SETS: readonly RefSet[] = reference.data.sets;
const DEFAULT = SETS.find((s) => s.set_id === reference.data.default_set) ?? SETS[0];

const W = 200;
const H = 172;
const CX = 80;
const CY = 80;
const R = 74;

function Drawing({ set }: { set: RefSet }) {
  const rings = ringsOf(set.layers.map((l) => ({ name: l.layer, outer_radius: l.outer_radius_um })));
  if (!rings) return null;
  const k = R / 104; // ringsOf draws to a radius of 104 units
  return (
    <svg
      className="refp__svg"
      viewBox={`0 0 ${W} ${H}`}
      role="img"
      aria-label={`AGR-1 particle, set ${set.set_id}, drawn to scale from the registered kernel diameter and coating thicknesses: ${set.layers.map((l) => `${l.layer} to ${l.outer_radius_text} µm`).join(', ')}. Geometry only, no concentration.`}
    >
      {[...rings].reverse().map((ring) => (
        <circle key={ring.index} cx={CX} cy={CY} r={ring.outer * k} className="refp__ring" style={{ ['--ring' as string]: layerColour(ring.index) }} />
      ))}
      <line x1={CX} y1={CY} x2={CX + R} y2={CY} className="refp__radius" />
      <text x={CX} y={H - 4} className="refp__text" textAnchor="middle">
        outer radius {set.layers[set.layers.length - 1]?.outer_radius_text} µm
      </text>
    </svg>
  );
}

export function ReferenceParticle() {
  const [id, setId] = useState(DEFAULT?.set_id ?? '');
  const set = SETS.find((s) => s.set_id === id) ?? DEFAULT;
  const titleId = useId();
  const selectId = useId();
  if (!set) return null;
  const generated = set.layers.some((l) => l.outer_radius_source === 'generator');
  return (
    <figure className="figure refp" aria-labelledby={titleId} data-testid="reference-particle" data-set={set.set_id}>
      <div className="figure__head">
        <span className="tag tag--register" data-testid="reference-tag">
          Registered geometry
        </span>
        <span className="figure__kind">not a simulation; no concentration</span>
      </div>
      <h3 id={titleId} className="refp__title">
        Reference fuel: AGR-1 UCO TRISO
      </h3>
      <div className="refp__body">
        <Drawing set={set} />
        <div className="refp__meta">
          <label className="refp__select" htmlFor={selectId}>
            Register set
          </label>
          <select id={selectId} value={set.set_id} onChange={(e) => setId(e.target.value)} data-testid="reference-set">
            {SETS.map((s) => (
              <option key={s.set_id} value={s.set_id}>
                {s.set_id}: {s.design}
              </option>
            ))}
          </select>
          <p className="refp__source">
            {set.source}, {set.layers[1]?.location ?? set.layers[0]?.location}
            {set.link ? (
              <>
                {' '}
                <a href={set.link}>{set.link.replace('https://doi.org/', 'doi:')}</a>
              </>
            ) : null}
            ; evidence class {set.evidence_class}; register status {set.statuses.join(', ')}.
          </p>
        </div>
      </div>
      <div className="table-box refp__tablebox">
      <table className="table table--compact refp__table" data-testid="reference-table">
        <thead>
          <tr>
            <th scope="col">Layer</th>
            <th scope="col">Row</th>
            <th scope="col" className="num-col">
              Registered value
            </th>
            <th scope="col" className="num-col">
              Outer radius
            </th>
            <th scope="col">Status</th>
          </tr>
        </thead>
        <tbody>
          {set.layers.map((l, i) => (
            <tr key={l.row_id} data-row={l.row_id}>
              <th scope="row">
                <span className="layer-dot" style={{ ['--ring' as string]: layerColour(i) }} aria-hidden="true" />
                {l.layer} <span className="muted">({l.material})</span>
              </th>
              <td className="mono">{l.row_id}</td>
              <td className="num-col">
                <span className="num">
                  {l.quantity} {l.value_text} µm
                </span>
              </td>
              <td className="num-col">
                <span className="num" data-value={String(l.outer_radius_um)}>
                  {l.outer_radius_text} µm
                </span>
              </td>
              <td>{l.status}</td>
            </tr>
          ))}
        </tbody>
      </table>
      </div>
      <p className="refp__note">
        Geometry only (rule E2): the simulation above stays the synthetic verification case, and AGR-1 dimensions are never combined
        with its diffusivities. AGR-1 runs need temperature-dependent diffusivities, which are derived but not yet implemented.
        {generated ? ' Outer radii are sums of the registered diameter and thicknesses, made by the data generator.' : ' Outer radii as given in the register.'}{' '}
        Source file{' '}
        <a href={repoFile(reference.input_file)}>
          <code>{reference.input_file}</code>
        </a>
        , generated by <code>{reference.generator}</code>.
      </p>
    </figure>
  );
}
