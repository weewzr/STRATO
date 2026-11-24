// @vitest-environment node
// The five TRISO layer colours stay distinguishable for common colour-vision deficiencies
// (prompt/DESIGN_SPECIFICATION.md Section 9). Design check only: colour science of sRGB swatches,
// no scientific quantity of the model is involved.
//
// Method: each colour is passed through the Machado, Oliveira and Fernandes (2009) simulation
// matrices at severity 1 for protanopia, deuteranopia and tritanopia (applied to linear RGB), and
// the CIEDE2000 colour difference of every pair is computed in CIELAB (D65). The palette was chosen
// by a max-min search of that difference; the test keeps every pair at 20 or more.
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const css = readFileSync(new URL('../src/design/tokens.css', import.meta.url), 'utf8');
const LAYERS = ['layer-kernel', 'layer-buffer', 'layer-ipyc', 'layer-sic', 'layer-opyc'];
const MIN_DE00 = 20;

function block(selector: RegExp): Record<string, string> {
  const m = selector.exec(css);
  if (!m) throw new Error(`block not found: ${selector}`);
  const out: Record<string, string> = {};
  for (const d of (m[1] ?? '').matchAll(/--([a-z0-9-]+):\s*(#[0-9a-fA-F]{6})\s*;/g)) out[d[1] as string] = (d[2] as string).toLowerCase();
  return out;
}

type V3 = [number, number, number];
const MATRICES: Record<string, V3[]> = {
  normal: [
    [1, 0, 0],
    [0, 1, 0],
    [0, 0, 1],
  ],
  protan: [
    [0.152286, 1.052583, -0.204868],
    [0.114503, 0.786281, 0.099216],
    [-0.003882, -0.048116, 1.051998],
  ],
  deutan: [
    [0.367322, 0.860646, -0.227968],
    [0.280085, 0.672501, 0.047413],
    [-0.01182, 0.04294, 0.968881],
  ],
  tritan: [
    [1.255528, -0.076749, -0.178779],
    [-0.078411, 0.930809, 0.147602],
    [0.004733, 0.691367, 0.3039],
  ],
};

const toLin = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);

function linRgb(hex: string): V3 {
  return [1, 3, 5].map((i) => toLin(parseInt(hex.slice(i, i + 2), 16) / 255)) as V3;
}

function simulate(rgb: V3, m: V3[]): V3 {
  return m.map((row) => Math.min(1, Math.max(0, row[0] * rgb[0] + row[1] * rgb[1] + row[2] * rgb[2]))) as V3;
}

function lab([r, g, b]: V3): V3 {
  const x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047;
  const y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  const z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883;
  const f = (t: number) => (t > 0.008856 ? Math.cbrt(t) : 7.787 * t + 16 / 116);
  return [116 * f(y) - 16, 500 * (f(x) - f(y)), 200 * (f(y) - f(z))];
}

const rad = (d: number) => (d * Math.PI) / 180;
const deg = (r: number) => (r * 180) / Math.PI;

/** CIEDE2000 (Sharma, Wu and Dalal 2005). */
export function de2000([L1, a1, b1]: V3, [L2, a2, b2]: V3): number {
  const C1 = Math.hypot(a1, b1);
  const C2 = Math.hypot(a2, b2);
  const Cb = (C1 + C2) / 2;
  const G = 0.5 * (1 - Math.sqrt(Cb ** 7 / (Cb ** 7 + 25 ** 7)));
  const a1p = (1 + G) * a1;
  const a2p = (1 + G) * a2;
  const C1p = Math.hypot(a1p, b1);
  const C2p = Math.hypot(a2p, b2);
  const h1p = (deg(Math.atan2(b1, a1p)) + 360) % 360;
  const h2p = (deg(Math.atan2(b2, a2p)) + 360) % 360;
  const dLp = L2 - L1;
  const dCp = C2p - C1p;
  let dh = h2p - h1p;
  if (C1p * C2p === 0) dh = 0;
  else if (dh > 180) dh -= 360;
  else if (dh < -180) dh += 360;
  const dHp = 2 * Math.sqrt(C1p * C2p) * Math.sin(rad(dh / 2));
  const Lbp = (L1 + L2) / 2;
  const Cbp = (C1p + C2p) / 2;
  let hbp: number;
  if (C1p * C2p === 0) hbp = h1p + h2p;
  else if (Math.abs(h1p - h2p) > 180) hbp = h1p + h2p < 360 ? (h1p + h2p + 360) / 2 : (h1p + h2p - 360) / 2;
  else hbp = (h1p + h2p) / 2;
  const T = 1 - 0.17 * Math.cos(rad(hbp - 30)) + 0.24 * Math.cos(rad(2 * hbp)) + 0.32 * Math.cos(rad(3 * hbp + 6)) - 0.2 * Math.cos(rad(4 * hbp - 63));
  const dth = 30 * Math.exp(-(((hbp - 275) / 25) ** 2));
  const Rc = 2 * Math.sqrt(Cbp ** 7 / (Cbp ** 7 + 25 ** 7));
  const Sl = 1 + (0.015 * (Lbp - 50) ** 2) / Math.sqrt(20 + (Lbp - 50) ** 2);
  const Sc = 1 + 0.045 * Cbp;
  const Sh = 1 + 0.015 * Cbp * T;
  const Rt = -Math.sin(rad(2 * dth)) * Rc;
  return Math.sqrt((dLp / Sl) ** 2 + (dCp / Sc) ** 2 + (dHp / Sh) ** 2 + Rt * (dCp / Sc) * (dHp / Sh));
}

describe('CIEDE2000 implementation', () => {
  // Pairs 1, 7 and 17 of the test data published with Sharma, Wu and Dalal (2005), Table 1.
  it.each([
    [[50, 2.6772, -79.7751], [50, 0, -82.7485], 2.0425],
    [[50, 0, 0], [50, -1, 2], 2.3669],
    [[50, 2.5, 0], [73, 25, -18], 27.1492],
  ] as [V3, V3, number][])('reproduces the published value %#', (p, q, expected) => {
    expect(de2000(p, q)).toBeCloseTo(expected, 4);
  });
});

describe.each([
  ['light', block(/^:root \{([\s\S]*?)\n\}/m)],
  ['dark', block(/:root\[data-theme='dark'\] \{([\s\S]*?)\n\}/)],
])('%s layer colours', (_name, theme) => {
  it('are five different tokens', () => {
    const values = LAYERS.map((l) => theme[l]);
    for (const v of values) expect(v).toMatch(/^#[0-9a-f]{6}$/);
    expect(new Set(values).size).toBe(5);
  });

  it.each(Object.keys(MATRICES))(`differ pairwise by at least ${MIN_DE00} CIEDE2000 under %s vision`, (kind) => {
    const m = MATRICES[kind] as V3[];
    const labs = LAYERS.map((l) => lab(simulate(linRgb(theme[l] as string), m)));
    for (let i = 0; i < labs.length; i++) {
      for (let j = i + 1; j < labs.length; j++) {
        expect(de2000(labs[i] as V3, labs[j] as V3), `${LAYERS[i]} vs ${LAYERS[j]}`).toBeGreaterThanOrEqual(MIN_DE00);
      }
    }
  });
});
