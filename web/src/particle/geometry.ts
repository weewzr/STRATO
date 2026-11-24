// Drawing geometry of the 2D cross-section in SVG user units (display scaling only, no physics).
// The particle is drawn to scale: every radius is divided by the outer radius of the drawing.

export const VIEW_W = 320;
export const VIEW_H = 240;
export const CX = 116;
export const CY = 120;
export const MAX_R = 104;
export const LABEL_X = 244;
const LABEL_TOP = 30;
const LABEL_BOTTOM = 206;

/** Pixel size of the concentration canvas (square); CSS scales it to the drawing. */
export const CANVAS_PX = 640;

export interface Ring {
  index: number;
  name: string;
  /** Inner and outer radius in drawing units. */
  inner: number;
  outer: number;
}

/** Rings of the layers, drawn to scale, from outer radii in metres (or any one unit). Null if the radii do not increase. */
export function ringsOf(layers: readonly { name: string; outer_radius: number }[]): Ring[] | null {
  const outer = layers[layers.length - 1]?.outer_radius ?? 0;
  const increasing = layers.every((r, i) => Number.isFinite(r.outer_radius) && r.outer_radius > (layers[i - 1]?.outer_radius ?? 0));
  if (!increasing || !(outer > 0)) return null;
  return layers.map((r, i) => ({
    index: i,
    name: r.name,
    inner: i === 0 ? 0 : ((layers[i - 1]?.outer_radius ?? 0) / outer) * MAX_R,
    outer: (r.outer_radius / outer) * MAX_R,
  }));
}

/** Leader-line end points for the layer labels: outermost layer at the top, kernel at the bottom. */
export function leaders(rings: Ring[], cx = CX, cy = CY, labelX = LABEL_X) {
  const n = rings.length;
  return rings.map((ring) => {
    const slot = n - 1 - ring.index;
    const y = n === 1 ? cy : LABEL_TOP + (slot * (LABEL_BOTTOM - LABEL_TOP)) / (n - 1);
    const rm = (ring.inner + ring.outer) / 2;
    const dy = Math.max(-rm, Math.min(rm, y - cy));
    const x = cx + Math.sqrt(Math.max(0, rm * rm - dy * dy));
    return { ...ring, px: x, py: cy + dy, ly: y, lx: labelX };
  });
}

/** SVG path of an annulus (or a disc when inner is 0), for pointer hit areas. */
export function annulusPath(inner: number, outer: number, cx = CX, cy = CY): string {
  const circle = (r: number) => `M ${cx - r} ${cy} a ${r} ${r} 0 1 0 ${2 * r} 0 a ${r} ${r} 0 1 0 ${-2 * r} 0 Z`;
  return inner > 0 ? `${circle(outer)} ${circle(inner)}` : circle(outer);
}
