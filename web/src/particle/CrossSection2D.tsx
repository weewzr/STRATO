import { useEffect, useId, useRef } from 'react';
import { layerColour } from '../design/layers';
import type { RunOk } from '../solver/types';
import { useStrato, type ColormapName } from '../state/store';
import { cellColours, css, type ColourScale } from './colormap';
import { annulusPath, CANVAS_PX, CX, CY, leaders, MAX_R, VIEW_H, VIEW_W, type Ring } from './geometry';

// 2D cross-section of the particle (prompt/DESIGN_SPECIFICATION.md Section 4.1; the default view,
// directly under the equation). With a run, a canvas paints every cell of the solver's mesh in the
// colour of the concentration the wrapper returned for the output row being played; the layer
// boundaries and their labels are drawn over it in the layer colours. The canvas is redrawn from a
// store subscription, so playback does not re-render React. Without a run the drawing is the
// schematic of the input radii (ILLUSTRATIVE), hatched and without concentration data.

/** Paints the concentration of output row `index` of `run`, one filled circle per cell, outside in. */
export function paintCells(canvas: HTMLCanvasElement, run: RunOk, index: number, map: ColormapName, scale: ColourScale): boolean {
  const ctx = canvas.getContext('2d');
  if (!ctx) return false;
  const faces = run.mesh.cell_faces_m;
  const outer = faces[faces.length - 1] ?? 0;
  const colours = cellColours(run, index, map, scale);
  const c = CANVAS_PX / 2;
  ctx.clearRect(0, 0, CANVAS_PX, CANVAS_PX);
  if (!(outer > 0)) return false;
  for (let i = colours.length - 1; i >= 0; i--) {
    const rgb = colours[i];
    const face = faces[i + 1];
    if (!rgb || face === undefined) continue;
    ctx.beginPath();
    ctx.arc(c, c, (face / outer) * c, 0, 2 * Math.PI);
    ctx.fillStyle = css(rgb);
    ctx.fill();
  }
  canvas.dataset.row = String(index);
  canvas.dataset.draws = String(Number(canvas.dataset.draws ?? '0') + 1);
  return true;
}

function ConcentrationCanvas({ run }: { run: RunOk }) {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const draw = () => {
      const s = useStrato.getState();
      paintCells(canvas, run, s.playback.tIndex, s.view.colormap, s.view.logColour ? 'log' : 'linear');
    };
    draw();
    return useStrato.subscribe((s, p) => {
      if (
        s.playback.tIndex !== p.playback.tIndex ||
        s.view.colormap !== p.view.colormap ||
        s.view.logColour !== p.view.logColour
      )
        draw();
    });
  }, [run]);
  const left = ((CX - MAX_R) / VIEW_W) * 100;
  const top = ((CY - MAX_R) / VIEW_H) * 100;
  return (
    <canvas
      ref={ref}
      className="xsec__canvas"
      width={CANVAS_PX}
      height={CANVAS_PX}
      data-testid="xsec-canvas"
      aria-hidden="true"
      style={{ left: `${left}%`, top: `${top}%`, width: `${((2 * MAX_R) / VIEW_W) * 100}%`, height: `${((2 * MAX_R) / VIEW_H) * 100}%` }}
    />
  );
}

export interface CrossSectionProps {
  rings: Ring[];
  run: RunOk | null;
  selected: number | null;
  hovered: number | null;
  onHover: (index: number | null) => void;
  onSelect: (index: number) => void;
  label: string;
}

export function CrossSection2D({ rings, run, selected, hovered, onHover, onSelect, label }: CrossSectionProps) {
  const hatchId = useId();
  const schematic = run === null;
  const focus = hovered ?? selected;
  return (
    <div className="xsec" data-testid="xsec" data-mode={schematic ? 'schematic' : 'computed'}>
      {run ? <ConcentrationCanvas run={run} /> : null}
      <svg className="particle__svg" viewBox={`0 0 ${VIEW_W} ${VIEW_H}`} role="img" aria-label={label} onPointerLeave={() => onHover(null)}>
        <defs>
          <pattern id={`${hatchId}-hatch`} width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
            <line x1="0" y1="0" x2="0" y2="6" className="particle__hatch" />
          </pattern>
        </defs>
        {schematic
          ? [...rings].reverse().map((ring) => (
              <circle
                key={`fill-${ring.index}`}
                cx={CX}
                cy={CY}
                r={ring.outer}
                className={`particle__ring${selected === ring.index ? ' is-selected' : ''}`}
                style={{ ['--ring' as string]: layerColour(ring.index) }}
              />
            ))
          : null}
        {schematic ? <circle cx={CX} cy={CY} r={MAX_R} fill={`url(#${hatchId}-hatch)`} className="particle__overlay" /> : null}
        {/* Layer boundaries in the layer colours, over a thin halo so they read on every fill. */}
        {schematic
          ? null
          : rings.map((ring) => (
              <g key={`b-${ring.index}`} className="particle__boundary" style={{ ['--ring' as string]: layerColour(ring.index) }}>
                <circle cx={CX} cy={CY} r={ring.outer} className="particle__halo" />
                <circle cx={CX} cy={CY} r={ring.outer} className="particle__edge" data-testid={`boundary-${ring.index}`} />
              </g>
            ))}
        {focus !== null && rings[focus] ? (
          <g className="particle__focus" style={{ ['--ring' as string]: layerColour(focus) }} data-testid="layer-focus">
            <circle cx={CX} cy={CY} r={rings[focus].outer} />
            {rings[focus].inner > 0 ? <circle cx={CX} cy={CY} r={rings[focus].inner} /> : null}
          </g>
        ) : null}
        {/* Pointer hit areas: hovering shows a layer in the inspector, a click selects it. */}
        {rings.map((ring) => (
          <path
            key={`hit-${ring.index}`}
            d={annulusPath(ring.inner, ring.outer)}
            fillRule="evenodd"
            className="particle__hit"
            onPointerEnter={() => onHover(ring.index)}
            onClick={() => onSelect(ring.index)}
          />
        ))}
        {leaders(rings).map((l) => (
          <g key={`l-${l.index}`} className="particle__label" style={{ ['--ring' as string]: layerColour(l.index) }}>
            <circle cx={l.px} cy={l.py} r={2.25} className="particle__dot" />
            <polyline points={`${l.px},${l.py} ${l.lx - 20},${l.ly} ${l.lx - 6},${l.ly}`} className="particle__leader" />
            <text x={l.lx} y={l.ly} dominantBaseline="middle" className="particle__text">
              {l.name}
            </text>
          </g>
        ))}
      </svg>
    </div>
  );
}
