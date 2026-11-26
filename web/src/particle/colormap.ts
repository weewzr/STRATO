// Colour mapping of the particle (prompt/TECHNICAL_ARCHITECTURE.md Section 4, "Particle rendering").
// A concentration returned by the Rust wrapper is turned into a colour: its position on the colour
// scale (linear, or logarithmic over a fixed number of decades), then the nearest of the 256
// entries of the colour table. This is display mapping only, like an axis of a plot; it computes
// no physical quantity. The tables come from web/src/generated/colormaps.json, written by
// strato-webtools colormaps from data/colormaps/ (copied from matplotlib, cividis by default).
import maps from '../generated/colormaps.json';
import type { ColormapName } from '../state/store';
import type { RunOk } from '../solver/types';

export type Rgb = readonly [number, number, number];
export type ColourScale = 'linear' | 'log';

/** Decades shown by the logarithmic scale, below the top of the scale. */
export const LOG_DECADES = 6;

const TABLES: Record<ColormapName, readonly Rgb[]> = {
  cividis: maps.data.cividis.rgb8 as unknown as Rgb[],
  viridis: maps.data.viridis.rgb8 as unknown as Rgb[],
};

export const COLORMAP_LABEL: Record<ColormapName, string> = { cividis: 'cividis', viridis: 'viridis' };

export function table(name: ColormapName): readonly Rgb[] {
  return TABLES[name];
}

/**
 * Position of `value` on the colour scale [0, 1] whose top is `top`. Linear: value / top. Log:
 * log10(value / top) / LOG_DECADES + 1. Values below the scale (including zero) give 0, values above
 * give 1. Returns null for a value that is not a finite number.
 */
export function scalePosition(value: number, top: number, scale: ColourScale): number | null {
  if (!Number.isFinite(value) || !(top > 0)) return null;
  if (value <= 0) return 0;
  const t = scale === 'linear' ? value / top : Math.log10(value / top) / LOG_DECADES + 1;
  return Math.min(1, Math.max(0, t));
}

/** Index of the colour-table entry for a scale position (nearest of the 256 entries). */
export function tableIndex(position: number): number {
  return Math.round(position * 255);
}

/** The colour of one value; values that are not finite get the neutral colour `none`. */
export function colourOf(name: ColormapName, value: number, top: number, scale: ColourScale, none: Rgb = [128, 128, 128]): Rgb {
  const t = scalePosition(value, top, scale);
  if (t === null) return none;
  return TABLES[name][tableIndex(t)] ?? none;
}

export function css(rgb: Rgb): string {
  return `rgb(${rgb[0]}, ${rgb[1]}, ${rgb[2]})`;
}

/**
 * The top of the colour scale: the largest nodal concentration over the whole run, which the wrapper
 * returns for every output row (max_concentration_mol_m3). It is fixed for the whole run so that
 * frames can be compared. A selection of returned numbers, not a computed quantity.
 */
export function scaleTop(run: RunOk): number {
  let top = 0;
  for (const v of run.max_concentration_mol_m3) if (v > top) top = v;
  return top;
}

/** Colours of every node of the run's output row `index`, in node order (centre outwards). */
export function nodeColours(run: RunOk, index: number, name: ColormapName, scale: ColourScale): Rgb[] {
  const profile = run.profiles[index];
  if (!profile) return [];
  const top = scaleTop(run);
  return profile.c_mol_per_m3.map((c) => colourOf(name, c, top, scale));
}

/**
 * The colour band that contains each of `bins` equally spaced radii (bin centres) from the centre to
 * the outer radius: a lookup in the ring edges the wrapper returned (the band of node i is
 * [edges[i], edges[i+1]]). Used to paint the cut faces of the 3D view.
 */
export function bandOfBins(edges: readonly number[], bins: number): number[] {
  const out: number[] = [];
  const outer = edges[edges.length - 1] ?? 0;
  let band = 0;
  for (let k = 0; k < bins; k++) {
    const r = ((k + 0.5) / bins) * outer;
    while (band < edges.length - 2 && r > (edges[band + 1] ?? Infinity)) band++;
    out.push(band);
  }
  return out;
}

/**
 * Tick marks of the legend, labelled by the full name of the position on the scale (the top of the
 * scale is the largest concentration of the run): 0, half and the top on the linear scale; every third
 * decade below the top on the logarithmic scale.
 */
export function legendTicks(scale: ColourScale): { position: number; label: string }[] {
  if (scale === 'linear')
    return [
      { position: 0, label: '0' },
      { position: 0.5, label: 'half of the top' },
      { position: 1, label: 'top' },
    ];
  const ticks: { position: number; label: string }[] = [];
  const sup = (n: number) => String(n).replace(/\d/g, (k) => '⁰¹²³⁴⁵⁶⁷⁸⁹'[Number(k)] ?? k);
  for (let d = LOG_DECADES; d > 0; d -= 3) ticks.push({ position: 1 - d / LOG_DECADES, label: `10⁻${sup(d)} of the top` });
  ticks.push({ position: 1, label: 'top' });
  return ticks;
}