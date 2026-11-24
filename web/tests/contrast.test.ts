// @vitest-environment node
// WCAG 2.2 contrast of every token pair the site uses, in the light and the dark theme.
// Design check only (relative luminance of sRGB colours); no scientific quantity is involved.
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const css = readFileSync(new URL('../src/design/tokens.css', import.meta.url), 'utf8');

function block(selector: RegExp): Record<string, string> {
  const m = selector.exec(css);
  if (!m) throw new Error(`block not found: ${selector}`);
  const body = m[1] ?? '';
  const out: Record<string, string> = {};
  for (const d of body.matchAll(/--([a-z0-9-]+):\s*(#[0-9a-fA-F]{6})\s*;/g)) out[d[1] as string] = (d[2] as string).toLowerCase();
  return out;
}

const light = block(/^:root \{([\s\S]*?)\n\}/m);
const dark = block(/:root\[data-theme='dark'\] \{([\s\S]*?)\n\}/);
const darkMedia = block(/:root:not\(\[data-theme='light'\]\) \{([\s\S]*?)\n {2}\}/);

function luminance(hex: string): number {
  const ch = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  const lin = ch.map((c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
  return 0.2126 * (lin[0] ?? 0) + 0.7152 * (lin[1] ?? 0) + 0.0722 * (lin[2] ?? 0);
}

export function ratio(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x) as [number, number];
  return (hi + 0.05) / (lo + 0.05);
}

const TEXT = 4.5;
const NON_TEXT = 3;

// [foreground, background, minimum]
const PAIRS: [string, string, number][] = [
  ...['bg', 'surface', 'bg-subtle', 'accent-bg', 'warn-bg'].map((b): [string, string, number] => ['text', b, TEXT]),
  ...['bg', 'surface', 'bg-subtle'].map((b): [string, string, number] => ['text-muted', b, TEXT]),
  ...['bg', 'surface', 'bg-subtle', 'accent-bg'].map((b): [string, string, number] => ['accent', b, TEXT]),
  ['on-accent', 'accent', TEXT],
  ['on-accent', 'accent-hover', TEXT],
  ...['bg', 'surface', 'bg-subtle'].map((b): [string, string, number] => ['accent-hover', b, TEXT]),
  ...['bg', 'surface', 'bg-subtle'].map((b): [string, string, number] => ['danger', b, TEXT]),
  ...['label-computed', 'label-measured', 'label-illustrative', 'placeholder-text'].flatMap((l) =>
    ['bg', 'surface', 'bg-subtle'].map((b): [string, string, number] => [l, b, TEXT]),
  ),
  ...['bg', 'surface', 'bg-subtle'].map((b): [string, string, number] => ['focus', b, NON_TEXT]),
  ...['bg', 'surface', 'bg-subtle'].map((b): [string, string, number] => ['border-strong', b, NON_TEXT]),
  ['warn', 'warn-bg', NON_TEXT],
  ...['mech-decay', 'mech-source', 'mech-arrhenius'].flatMap((m) =>
    ['surface', 'bg-subtle'].map((b): [string, string, number] => [m, b, NON_TEXT]),
  ),
  // Layer colours are drawn as ring outlines, leader lines and swatches on the page and figure surfaces.
  ...['layer-kernel', 'layer-buffer', 'layer-ipyc', 'layer-sic', 'layer-opyc'].flatMap((l) =>
    ['bg', 'surface', 'bg-subtle'].map((b): [string, string, number] => [l, b, NON_TEXT]),
  ),
];

describe.each([
  ['light', light],
  ['dark', dark],
])('%s theme', (_name, theme) => {
  it.each(PAIRS)('%s on %s meets %s:1', (fg, bg, min) => {
    const f = theme[fg];
    const b = theme[bg];
    expect(f, `token --${fg}`).toBeDefined();
    expect(b, `token --${bg}`).toBeDefined();
    expect(ratio(f as string, b as string)).toBeGreaterThanOrEqual(min);
  });
});

describe('theme blocks', () => {
  it('define the same dark values for the system setting and the manual toggle', () => {
    expect(darkMedia).toEqual(dark);
  });

  it('give every light colour token a dark value', () => {
    const colourTokens = Object.keys(light);
    expect(colourTokens.length).toBeGreaterThan(20);
    for (const k of colourTokens) expect(dark[k], `--${k}`).toBeDefined();
  });

  it('reproduce known WCAG ratios', () => {
    expect(ratio('#000000', '#ffffff')).toBeCloseTo(21, 6);
    expect(ratio('#767676', '#ffffff')).toBeCloseTo(4.54, 2);
  });
});
