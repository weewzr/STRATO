import { useId } from 'react';
import { BUILD } from '../build';
import type { CaseProvenance, V1Config } from '../data/case';
import { layerColour } from '../design/layers';
import { useStrato } from '../state/store';
import { Badge, Sci } from './Badge';

// Drawing geometry in SVG user units (display scaling only).
const W = 320;
const H = 240;
const CX = 116;
const CY = 120;
const MAX_R = 104;
const LABEL_X = 244;
const LABEL_TOP = 30;
const LABEL_BOTTOM = 206;

interface Ring {
  index: number;
  name: string;
  inner: number;
  outer: number;
}

/** Leader-line end points for the layer labels: outermost layer at the top, kernel at the bottom. */
function leaders(rings: Ring[]) {
  const n = rings.length;
  return rings.map((ring) => {
    const slot = n - 1 - ring.index;
    const y = n === 1 ? CY : LABEL_TOP + (slot * (LABEL_BOTTOM - LABEL_TOP)) / (n - 1);
    const rm = (ring.inner + ring.outer) / 2;
    const dy = Math.max(-rm, Math.min(rm, y - CY));
    const x = CX + Math.sqrt(Math.max(0, rm * rm - dy * dy));
    return { ...ring, px: x, py: CY + dy, ly: y };
  });
}

/**
 * Cross-section of the particle with the layer radii of the active configuration, drawn to scale,
 * one colour per layer and every layer boundary labelled. It carries no concentration data and is
 * labelled ILLUSTRATIVE; from web phase 4 the solver's profile colours the cells.
 */
export function ParticleSchematic({ config, provenance }: { config: V1Config; provenance: CaseProvenance }) {
  const selected = useStrato((s) => s.view.selectedLayer);
  const selectLayer = useStrato((s) => s.selectLayer);
  const captionId = useId();
  const listId = useId();
  const regions = config.regions;
  const outer = regions[regions.length - 1]?.outer_radius_m ?? 0;
  const increasing = regions.every((r, i) => Number.isFinite(r.outer_radius_m) && r.outer_radius_m > (regions[i - 1]?.outer_radius_m ?? 0));
  const drawable = increasing && outer > 0;
  const sel = selected === null ? null : regions[selected];

  const rings: Ring[] = drawable
    ? regions.map((r, i) => ({
        index: i,
        name: r.name,
        inner: i === 0 ? 0 : ((regions[i - 1]?.outer_radius_m ?? 0) / outer) * MAX_R,
        outer: (r.outer_radius_m / outer) * MAX_R,
      }))
    : [];

  return (
    <figure className="figure particle" aria-labelledby={captionId} data-testid="particle-figure">
      <div className="particle__view" data-testid="particle">
        <div className="figure__head">
          <Badge label="ILLUSTRATIVE" />
          <span className="figure__kind">Particle cross-section to the input radii; no concentration data</span>
        </div>
        <div className="particle__grid">
          {drawable ? (
            <svg
              className="particle__svg"
              viewBox={`0 0 ${W} ${H}`}
              role="img"
              aria-label={`Cross-section of the particle: ${regions.length} concentric layers drawn to the outer radii of the active parameters, from ${regions[0]?.name ?? ''} at the centre to ${regions[regions.length - 1]?.name ?? ''} at the surface. Schematic only, no concentration data.`}
            >
              <defs>
                <pattern id={`${captionId}-hatch`} width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
                  <line x1="0" y1="0" x2="0" y2="6" className="particle__hatch" />
                </pattern>
              </defs>
              {[...rings].reverse().map((ring) => (
                <circle
                  key={ring.index}
                  cx={CX}
                  cy={CY}
                  r={ring.outer}
                  className={`particle__ring${selected === ring.index ? ' is-selected' : ''}`}
                  style={{ ['--ring' as string]: layerColour(ring.index) }}
                />
              ))}
              <circle cx={CX} cy={CY} r={MAX_R} fill={`url(#${captionId}-hatch)`} className="particle__overlay" />
              {leaders(rings).map((l) => (
                <g key={l.index} className="particle__label" style={{ ['--ring' as string]: layerColour(l.index) }}>
                  <circle cx={l.px} cy={l.py} r={2.25} className="particle__dot" />
                  <polyline points={`${l.px},${l.py} ${LABEL_X - 20},${l.ly} ${LABEL_X - 6},${l.ly}`} className="particle__leader" />
                  <text x={LABEL_X} y={l.ly} dominantBaseline="middle" className="particle__text">
                    {l.name}
                  </text>
                </g>
              ))}
            </svg>
          ) : (
            <p className="particle__none" role="status">
              The radii do not increase from the centre outwards, so the cross-section is not drawn. Correct the outer radii in the controls.
            </p>
          )}
          <div className="inspector">
            <p className="inspector__title" id={listId}>
              Inspect a layer
            </p>
            <ul className="inspector__list" aria-labelledby={listId}>
              {regions.map((r, i) => (
                <li key={i}>
                  <button
                    type="button"
                    className="inspector__item"
                    aria-pressed={selected === i}
                    onClick={() => selectLayer(selected === i ? null : i)}
                    data-testid={`inspect-${i}`}
                    style={{ ['--ring' as string]: layerColour(i) }}
                  >
                    <span className="inspector__swatch" aria-hidden="true" />
                    {r.name}
                  </button>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </div>
      {sel && selected !== null ? (
        <dl className="inspector__detail" data-testid="inspector-detail">
          <dt>Layer</dt>
          <dd>{sel.name}</dd>
          <dt>
            Inner radius R<sub>{selected}</sub>
          </dt>
          <dd>{selected === 0 ? <span className="num">0 (centre)</span> : <Sci value={regions[selected - 1]?.outer_radius_m ?? NaN} unit="m" />}</dd>
          <dt>
            Outer radius R<sub>{selected + 1}</sub>
          </dt>
          <dd>
            <Sci value={sel.outer_radius_m} unit="m" />
          </dd>
          <dt>
            Diffusivity D<sub>{selected + 1}</sub>
          </dt>
          <dd>
            <Sci value={sel.diffusivity_m2_s} unit="m² s⁻¹" />
          </dd>
          <dt>Cells</dt>
          <dd className="num">{sel.cells}</dd>
          <dt>Inventory share</dt>
          <dd className="muted">from the solver, web phase 4</dd>
        </dl>
      ) : null}
      <figcaption id={captionId} className="figure__caption">
        <strong>Figure 1.</strong> The {regions.length} layers of the particle at the outer radii of the active parameters, drawn to scale, one
        colour per layer. Schematic only: the fill carries no concentration data.
        <span className="figure__source">
          Source: active parameters, starting from <code>{provenance.input_file}</code> (synthetic verification case, not physical data).
          STRATO v{BUILD.strato_version}, build <code>{BUILD.commit}</code>.
        </span>
      </figcaption>
    </figure>
  );
}
