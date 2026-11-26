// @vitest-environment node
// Progressive equation builder for the BTCS solver: every equation against the BTCS documents,
// KaTeX rendering, the notation rules of directive Section 5, and the documentation and code anchors
// (prompt/TESTING_AND_VALIDATION.md Section 3 row 2).
import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { DOCS, headingSlug, type DocId, type DocRef } from '../src/equation/links';
import { NOTATION, NUMERICAL_ITEMS } from '../src/equation/numerics';
import {
  ADDITION_PHYSICS,
  allEquations,
  BASELINE_STAGES,
  conditionsFor,
  equationNumber,
  LAST_SOLVED_STEP,
  mainEquations,
  markParts,
  MECHANISM_STAGES,
  physicsAdditions,
  STAGES,
  STEP_DEFS,
  SWITCH_MECHANISM,
  SWITCHES,
  switchIntroducedBy,
  TERMS,
  type EqSource,
} from '../src/equation/terms';
import { renderTex } from '../src/equation/Tex';

const REPO = fileURLToPath(new URL('../..', import.meta.url));
const docText = (doc: DocId) => readFileSync(join(REPO, DOCS[doc].path), 'utf8');
const TEXT: Record<DocId, string> = { d1: docText('d1'), d2: docText('d2'), d3: docText('d3'), d4: docText('d4'), d5: docText('d5') };
/*
 * The patterns are written as \u escapes so that this file is pure ASCII and cannot be damaged by a re-encoding.
 * They are the forms that occurred: the right single quotation mark (U+2019, bytes E2 80 99), the Greek letters pi
 * (U+03C0, CF 80) and Delta (U+0394, CE 94) and the middle dot (U+00B7, C2 B7), each read as Windows-1252; the lead
 * characters of such pairs; and the replacement character.
 */
const MOJIBAKE = ['\u00e2\u20ac', '\u00cf\u20ac', '\u00ce\u201d', '\u00c2', '\u00c3', '\ufffd'];
/** Returns the first damaged-text pattern the text contains, or null. */
export function findMojibake(text: string): string | null {
  return MOJIBAKE.find((bad) => text.includes(bad)) ?? null;
}
const squash = (s: string) => s.replace(/\s+/g, '');
const MATHS = squash(TEXT.d1 + TEXT.d2 + TEXT.d5);

/** Removes \htmlClass{classes}{body} wrappers, keeping the body (brace matching). */
export function stripHtmlClass(latex: string): string {
  const key = '\\htmlClass{';
  let out = latex;
  for (let at = out.indexOf(key); at >= 0; at = out.indexOf(key)) {
    let i = at + key.length;
    let depth = 1;
    while (depth > 0 && i < out.length) {
      if (out[i] === '{') depth++;
      else if (out[i] === '}') depth--;
      i++;
    }
    if (out[i] !== '{') throw new Error('malformed \\htmlClass');
    const start = i + 1;
    depth = 1;
    i = start;
    while (depth > 0 && i < out.length) {
      if (out[i] === '{') depth++;
      else if (out[i] === '}') depth--;
      i++;
    }
    out = out.slice(0, at) + out.slice(start, i - 1) + out.slice(i);
  }
  return out;
}

/** The $$ ... $$ block that carries \tag{tag}. */
function taggedBlock(doc: DocId, tag: string): string {
  const text = TEXT[doc];
  const at = text.indexOf(`\\tag{${tag}}`);
  if (at < 0) throw new Error(`no \\tag{${tag}} in ${DOCS[doc].path}`);
  const open = text.lastIndexOf('$$', at);
  const close = text.indexOf('$$', at);
  return text.slice(open + 2, close);
}

interface Heading {
  level: number;
  text: string;
  start: number;
}
function headings(doc: DocId): Heading[] {
  const out: Heading[] = [];
  const re = /^(#{1,6})\s+(.*?)\s*$/gm;
  for (let m = re.exec(TEXT[doc]); m; m = re.exec(TEXT[doc])) out.push({ level: m[1]!.length, text: m[2]!, start: m.index });
  return out;
}

/** Text of a section, from its heading to the next heading of the same or a higher level. */
function sectionText(doc: DocId, heading: string): string {
  const hs = headings(doc);
  const i = hs.findIndex((h) => h.text === heading);
  if (i < 0) throw new Error(`no heading "${heading}" in ${DOCS[doc].path}`);
  const h = hs[i]!;
  const next = hs.slice(i + 1).find((n) => n.level <= h.level);
  return TEXT[doc].slice(h.start, next ? next.start : undefined);
}

/** The block without \tag{}, a trailing full stop or comma, and an outer \boxed{}. */
export function wholeEquation(block: string): string {
  let b = block.replace(/\\tag\{[^}]*\}/g, '').trim();
  b = b.replace(/[.,]\s*$/, '').trim();
  if (b.startsWith('\\boxed{') && b.endsWith('}')) {
    let depth = 0;
    let closesAtEnd = true;
    for (let i = '\\boxed'.length; i < b.length; i++) {
      if (b[i] === '{') depth++;
      else if (b[i] === '}') {
        depth--;
        if (depth === 0 && i < b.length - 1) closesAtEnd = false;
      }
    }
    if (closesAtEnd) b = b.slice('\\boxed{'.length, -1).trim();
  }
  return b.replace(/[.,]\s*$/, '').trim();
}

/** The content of a leading \\boxed{...} group, or null. */
export function boxedInner(whole: string): string | null {
  if (!whole.startsWith('\\boxed{')) return null;
  let depth = 0;
  for (let i = '\\boxed'.length; i < whole.length; i++) {
    if (whole[i] === '{') depth++;
    else if (whole[i] === '}' && --depth === 0) return whole.slice('\\boxed{'.length, i);
  }
  return null;
}

/** True when position `at` of the (whitespace-free) text lies at brace depth 0 next to =, a comma, a semicolon or an end. */
export function atTopLevelSeparator(text: string, at: number, start: boolean): boolean {
  let depth = 0;
  for (let i = 0; i < at; i++) {
    if (text[i] === '{') depth++;
    else if (text[i] === '}') depth--;
  }
  if (depth !== 0) return false;
  if (at === 0 || at === text.length) return true;
  const ch = start ? text[at - 1]! : text[at]!;
  return ch === '=' || ch === ',' || ch === ';';
}
/** Every math span of a section ($$...$$ first, then $...$), as whole equations without tag, boxed wrapper or final punctuation. */
export function mathSpans(section: string): string[] {
  const out: string[] = [];
  const rest = section.replace(/\$\$([\s\S]*?)\$\$/g, (_m, inner: string) => {
    out.push(inner);
    return ' ';
  });
  for (const m of rest.matchAll(/\$([^$\n]+?)\$/g)) out.push(m[1]!);
  return out.map((b) => squash(wholeEquation(b)));
}

/** The text around each occurrence of `name` in a section: its own line, extended through the next display block when that line holds no math. */
export function namedWindows(section: string, name: string): string {
  const lines = section.split(/\r?\n/);
  const out: string[] = [];
  lines.forEach((line, i) => {
    if (!line.includes(name)) return;
    if (line.includes('$')) {
      out.push(line);
      return;
    }
    const rest = lines.slice(i).join('\n');
    const open = rest.indexOf('$$');
    const close = open < 0 ? -1 : rest.indexOf('$$', open + 2);
    out.push(close < 0 ? line : rest.slice(0, close + 2));
  });
  return out.join('\n');
}
/** Entries that may show a declared piece of a longer tagged equation (each is checked against the rules below). */
export const PIECE_ALLOWED = ['d1:5'];

function checkSource(src: EqSource, displayed: string, what: string, numberAs?: string) {
  const v = squash(src.verbatim);
  const key = `${src.doc}:${src.tag ?? src.section}`;
  if (src.piece) expect(PIECE_ALLOWED, `${what}: ${key} is not an allowed piece`).toContain(key);
  const block = src.tag ? taggedBlock(src.doc, src.tag) : sectionText(src.doc, src.section);
  if (src.tag) {
    // A tagged equation is the whole block, so a copy with terms or qualifiers cut off fails; a piece needs an explicit declaration.
    const whole = squash(wholeEquation(block));
    if (src.piece) {
      const inner = boxedInner(whole);
      const isBoxedPiece = inner !== null && v === squash(inner);
      if (!isBoxedPiece) {
        const at = whole.indexOf(v);
        expect(at, `${what}: declared piece not found in (${src.tag})`).toBeGreaterThanOrEqual(0);
        expect(v, `${what}: a piece must differ from the whole equation`).not.toBe(whole);
        expect(atTopLevelSeparator(whole, at, true), `${what}: the piece does not start at a top-level separator`).toBe(true);
        expect(atTopLevelSeparator(whole, at + v.length, false), `${what}: the piece does not end at a top-level separator`).toBe(true);
        // A piece never drops the right-hand side: it does not end just before a top-level "=".
        expect(whole[at + v.length], `${what}: the piece ends before an equals sign`).not.toBe('=');
      }
    } else {
      expect(v, `${what}: copy differs from the whole block of (${src.tag})`).toBe(whole);
    }
  } else {
    // An untagged equation is one whole math span of the cited section, not a part of it.
    expect(src.piece, `${what}: an untagged line is never a piece`).toBeFalsy();
    // With a name from the text (P14, F16a), the span must lie at the place of that name: its table row, or the line and the display that follows it.
    const scope = numberAs ? namedWindows(block, numberAs) : block;
    expect(mathSpans(scope), `${what}: no whole math span ${numberAs ? `at ${numberAs} ` : ''}of ${src.section} equals the copy`).toContain(v);
  }
  // The tagged equation also lies in the cited section.
  if (src.tag) expect(sectionText(src.doc, src.section), `${what}: (${src.tag}) is not in section "${src.section}"`).toContain(`\\tag{${src.tag}}`);
  // An untagged line that the page numbers by a name from the text: the section must use that name.
  if (!src.tag && numberAs) expect(sectionText(src.doc, src.section), `${what}: the name ${numberAs} is not in the section`).toContain(numberAs);
  // Nothing is rewritten: the displayed LaTeX is the copy.
  expect(squash(stripHtmlClass(displayed)), `${what}: displayed LaTeX differs from the document text`).toBe(v);
}
describe('the fourteen baseline steps and the three mechanism steps', () => {
  // Rewritten with the directive of 26 Nov 2026: steps fifteen to seventeen are no longer previews with
  // placeholder equations; they are written from Document 5 and carry their mechanism (their status is the build's).
  it('has the fourteen baseline steps in order, then the three mechanism steps', () => {
    expect(BASELINE_STAGES.map((s) => STEP_DEFS[s].name)).toEqual([
      'Conservation of mass',
      "Fick's law",
      'Spherical diffusion equation',
      'Five-layer material model',
      'Central spatial differences',
      'Backward Euler discretisation',
      'Centre-node equation',
      'Interface-node equations',
      'Outer-boundary equation',
      'Tridiagonal matrix',
      'Rust solution algorithm',
      'Inventory and release',
      'Numerical verification',
      'Release fraction for comparison',
    ]);
    expect(MECHANISM_STAGES.map((s) => STEP_DEFS[s].name)).toEqual([
      'Generation source-term contribution',
      'Radioactive decay contribution',
      'Temperature-dependent diffusion coefficients',
    ]);
    expect(STAGES.length).toBe(17);
    expect(BASELINE_STAGES.every((s) => STEP_DEFS[s].kind === 'step' && STEP_DEFS[s].mechanism === null)).toBe(true);
    expect(MECHANISM_STAGES.every((s) => STEP_DEFS[s].kind === 'mechanism' && STEP_DEFS[s].mechanism !== null)).toBe(true);
    expect(LAST_SOLVED_STEP).toBe(12);
  });

  it('every step has a main equation and parts, and part ids are unique over all steps', () => {
    const ids = STAGES.flatMap((s) => STEP_DEFS[s].parts.map((p) => p.id));
    expect(new Set(ids).size).toBe(ids.length);
    for (const s of STAGES) {
      expect(STEP_DEFS[s].main.length, `step ${s} main`).toBeGreaterThan(0);
      expect(STEP_DEFS[s].parts.length, `step ${s} parts`).toBeGreaterThan(0);
      expect(STEP_DEFS[s].stage).toBe(s);
    }
    expect(Object.keys(TERMS).sort()).toEqual([...ids].sort());
  });

  it('keeps the mechanism steps, the switches and the mechanism ids in step', () => {
    expect(SWITCHES.map((s) => s.id)).toEqual(['source', 'decay', 'arrhenius']);
    expect(SWITCH_MECHANISM).toEqual({ source: 'generation', decay: 'decay', arrhenius: 'diffusivity_temperature' });
    for (const sw of SWITCHES) {
      expect(switchIntroducedBy(sw.stage)).toBe(sw.id);
      expect(STEP_DEFS[sw.stage].kind).toBe('mechanism');
      expect(STEP_DEFS[sw.stage].mechanism).toBe(SWITCH_MECHANISM[sw.id]);
      expect(TERMS[sw.term]?.stage).toBe(sw.stage);
    }
    for (const s of BASELINE_STAGES) expect(switchIntroducedBy(s)).toBeNull();
  });
});

describe('every equation is the text of Document 1 or Document 2', () => {
  it('main, implementation, condition and balance equations of every step', () => {
    for (const [what, e] of allEquations()) {
      expect(e.latex, `${what}: no rewriting of the copied line`).toBe(e.source.verbatim);
      checkSource(e.source, e.latex, what, e.numberAs);
    }
  });

  it('a copy with terms or qualifiers cut off is refused', () => {
    const find = (tag: string) => allEquations().map(([, e]) => e).find((e) => e.source.tag === tag && !e.source.piece)!;
    const cut = (tag: string, from: string) => {
      const e = find(tag);
      const at = e.source.verbatim.lastIndexOf(from);
      expect(at, `${tag}: marker present`).toBeGreaterThan(0);
      const short = e.source.verbatim.slice(0, at).trimEnd();
      expect(() => checkSource({ ...e.source, verbatim: short }, short, tag), `${tag} shortened`).toThrow();
    };
    cut('F17', ' + \\left|U_{N-1}\\right|'); // the last term of the diagonal
    cut('F11', ' = 1 + \\left|L_i\\right|'); // the second form of B_i
    cut('15', ', \\qquad U_i'); // the upper coefficient
    cut('F29', ' \\quad \\text{for every }'); // the qualifier
    // The untouched copies pass.
    for (const tag of ['F17', 'F11', '15', 'F29']) {
      const e = find(tag);
      expect(() => checkSource(e.source, e.latex, tag)).not.toThrow();
    }
  });
  it('untagged equations are whole spans: a truncated or swapped copy is refused', () => {
    const find = (id: string) => allEquations().map(([, e]) => e).find((e) => e.id === id)!;
    const refuse = (id: string, latex: string) => {
      const e = find(id);
      expect(() => checkSource({ ...e.source, verbatim: latex }, latex, id, e.numberAs), `${id} mutated`).toThrow();
    };
    const f16a = find('interface_row_a').source.verbatim;
    refuse('interface_row_a', f16a.slice(0, f16a.indexOf(', \\qquad U_')));
    refuse('order_five_layers', String.raw`\max_j\left\lVert \mathbf{e}^j\right\rVert _\infty = O\left(\Delta t + \Delta r\right)`);
    refuse('order_five_layers', String.raw`\max_j\left\lVert \mathbf{e}^j\right\rVert _V`);
    for (const id of ['interface_row_a', 'order_five_layers', 'order_one_material']) {
      const e = find(id);
      expect(() => checkSource(e.source, e.latex, id, e.numberAs)).not.toThrow();
    }
  });

  it('a piece that drops the right-hand side, or an undeclared piece, is refused', () => {
    const e = allEquations().map(([, x]) => x).find((x) => x.source.tag === 'F11')!;
    const cutAtEquals = e.source.verbatim.slice(0, e.source.verbatim.lastIndexOf(' = 1 - L_i - U_i')) + ' = 1 - L_i - U_i';
    expect(() => checkSource({ ...e.source, verbatim: cutAtEquals, piece: true }, cutAtEquals, 'F11')).toThrow();
    const dropsRhs = e.source.verbatim.slice(0, e.source.verbatim.indexOf(', \\qquad B_i = 1 - L_i - U_i') + ', \\qquad B_i'.length);
    expect(() => checkSource({ ...e.source, verbatim: dropsRhs, piece: true }, dropsRhs, 'F11')).toThrow();
    expect(PIECE_ALLOWED).toEqual(['d1:5']);
  });
  it('the equation number shown is the number printed in the document', () => {
    for (const [what, e] of allEquations()) {
      const n = equationNumber(e);
      if (e.source.tag) expect(n, what).toBe(e.source.doc === 'd1' ? `D1.${e.source.tag}` : e.source.doc === 'd5' ? `D5.${e.source.tag}` : e.source.tag);
      else expect(n, what).toBe(e.numberAs ?? null);
      // Every number of Document 2 shown as an F-number is an F-number of that document.
      if (e.source.doc === 'd2' && e.source.tag) expect(e.source.tag, what).toMatch(/^F\d+[a-z]?$/);
      if (e.source.doc === 'd1' && e.source.tag) expect(e.source.tag, what).toMatch(/^\d+[a-z]?$/);
      // A Document 5 number is shown as D5.n.
      if (e.source.doc === 'd5' && e.source.tag) {
        expect(e.source.tag, what).toMatch(/^\d+$/);
        expect(n, what).toBe(`D5.${e.source.tag}`);
      }
    }
  });

  it('only the BTCS documents are cited as the source of an equation', () => {
    for (const [, e] of allEquations()) expect(['d1', 'd2', 'd5']).toContain(e.source.doc);
  });

  it('every equation of the mechanism steps and every physics addition is the text of Document 5', () => {
    const fromD5 = allEquations().filter(([what]) => /^step (15|16|17) (main|implementation)|addition/.test(what));
    expect(fromD5.length).toBeGreaterThan(8);
    for (const [what, e] of fromD5) expect(e.source.doc, what).toBe('d5');
    // The generation, decay, temperature, combined additions exist where the assembled system, the algorithm, the accounting and the verification are shown.
    for (const p of ADDITION_PHYSICS) {
      for (const s of [10, 11, 12, 13] as const) expect(physicsAdditions(s, p).note, `${p} at step ${s}`).not.toBeNull();
    }
    expect(physicsAdditions(10, 'baseline')).toEqual({ equations: [], note: null });
    expect(physicsAdditions(5, 'combined')).toEqual({ equations: [], note: null });
  });

  it('a Document 5 equation that is altered, cut or taken from the wrong place is refused (negative test)', () => {
    const find = (id: string) => allEquations().map(([, e]) => e).find((e) => e.id === id)!;
    const refuse = (id: string, latex: string) => {
      const e = find(id);
      expect(() => checkSource({ ...e.source, verbatim: latex }, latex, id, e.numberAs), `${id} mutated`).toThrow();
    };
    const d16 = find('d5_decay_diagonal');
    refuse('d5_decay_diagonal', d16.source.verbatim.replace(' + \\kappa \\quad', ' \\quad')); // the decay number dropped from B_i
    refuse('d5_step_diffusivity', find('d5_step_diffusivity').source.verbatim.replace('\\tfrac{1}{2}\\Delta t', '\\Delta t')); // end of the step instead of the middle
    refuse('d5_combined_balance_sum', find('d5_combined_balance_sum').source.verbatim.replace(' - \\left(M_{\\mathrm{dec}}^{j+1}', ' + \\left(M_{\\mathrm{dec}}^{j+1}')); // a sign changed
    refuse('d5_row', find('d5_row').source.verbatim.replace('\\Delta t\\,g_i^j', 'g_i^j')); // the time step dropped from the right-hand side
    // A tag of another equation, or a tag the document does not have, is refused.
    const e = find('d5_arrhenius');
    expect(() => checkSource({ ...e.source, tag: '23' }, e.latex, 'wrong tag')).toThrow();
    expect(() => checkSource({ ...e.source, tag: '9999' }, e.latex, 'absent tag')).toThrow();
    // The untouched copies pass.
    for (const id of ['d5_decay_diagonal', 'd5_step_diffusivity', 'd5_combined_balance_sum', 'd5_row', 'd5_arrhenius']) {
      const x = find(id);
      expect(() => checkSource(x.source, x.latex, id, x.numberAs)).not.toThrow();
    }
  });

  it('the highlighted equation of a step is the copy with wrappers added', () => {
    for (const s of STAGES) {
      const shown = mainEquations(s);
      expect(shown.length).toBe(STEP_DEFS[s].main.length);
      shown.forEach((e, i) => {
        const src = STEP_DEFS[s].main[i]!;
        expect(squash(stripHtmlClass(e.latex)), `step ${s} ${e.id}`).toBe(squash(src.latex));
        checkSource(src.source, stripHtmlClass(e.latex), `step ${s} ${e.id}`, src.numberAs);
      });
    }
  });

  it('every part marks a piece of a main equation of its step, and only the parts that a mechanism introduces carry its colour', () => {
    for (const s of STAGES) {
      const def = STEP_DEFS[s];
      for (const p of def.parts) {
        if (p.latex !== '') {
          expect(
            def.main.some((e) => e.latex.includes(p.latex)),
            `step ${s} part ${p.id}: "${p.latex}" is not in a main equation`,
          ).toBe(true);
        }
        // A colour needs a mechanism step and a piece of the equation to colour; a baseline part has neither.
        if (p.highlight !== undefined) {
          expect(def.kind, `part ${p.id}`).toBe('mechanism');
          expect(p.latex, `part ${p.id}`).not.toBe('');
        }
        expect(p.mechanism !== null, `part ${p.id}`).toBe(def.kind === 'mechanism');
      }
      if (def.kind === 'mechanism') expect(def.parts.some((p) => p.highlight !== undefined), `step ${s} introduces something`).toBe(true);
      // The highlight wrappers do not overlap (markParts throws if they do).
      for (const e of def.main) expect(() => markParts(e.latex, def.parts)).not.toThrow();
    }
  });

  it('every symbol, piece and unit of the parts occurs in the maths of the documents', () => {
    for (const s of STAGES) {
      for (const p of STEP_DEFS[s].parts) {
        expect(MATHS.includes(squash(p.symbol)), `part ${p.id} symbol ${p.symbol}`).toBe(true);
        if (p.latex) expect(MATHS.includes(squash(p.latex)), `part ${p.id} latex`).toBe(true);
        for (const u of p.units) {
          expect(MATHS.includes(squash(u.symbol)), `unit symbol ${u.symbol} of ${p.id}`).toBe(true);
          if (!u.unit.startsWith('\\text{')) expect(MATHS.includes(squash(u.unit)), `unit ${u.unit} of ${p.id}`).toBe(true);
        }
      }
    }
    for (const sw of SWITCHES) expect(MATHS.includes(squash(sw.latex)), `switch ${sw.id}`).toBe(true);
  });

  // Rewritten with the directive of 26 Nov 2026: the old test checked that the three previews showed only the
  // placeholder lines of Document 2, Section 9; the mechanism steps now show the derived equations of Document 5.
  it('the mechanism steps show tagged equations of Document 5 as their main equation', () => {
    for (const s of MECHANISM_STAGES) {
      for (const e of STEP_DEFS[s].main) {
        expect(e.source.doc).toBe('d5');
        expect(e.source.tag).not.toBeNull();
        expect(squash(sectionText('d5', e.source.section)), `step ${s} ${e.id}`).toContain(`\\tag{${e.source.tag}}`);
      }
    }
  });
});

describe('notation of directive Section 5', () => {
  const shown = () => allEquations().map(([what, e]) => [what, e.latex] as [string, string]);

  it('powers enclose the whole variable in parentheses; the time index j is a superscript index', () => {
    for (const [what, latex] of shown()) {
      for (const m of latex.matchAll(/\^\{?([2-9])\}?/g)) {
        const before = latex.slice(0, m.index);
        expect(/(\\right\)|\\partial|\\partial r)$/.test(before), `${what}: bare power "${before.slice(-14)}^${m[1]}"`).toBe(true);
      }
      expect(latex, what).not.toMatch(/C_i\[j\]|C\[/);
    }
  });

  it('no equation uses j as a spatial index or i as a material index', () => {
    for (const [what, latex] of shown()) {
      expect(latex, what).not.toMatch(/\b[A-Za-z]_j\b|_\{j[,}]/);
      expect(latex, what).not.toMatch(/D_i\b(?!,)/);
    }
  });

  it('text shown in block A has no numeral outside an equation, a reference or a step number', () => {
    for (const s of STAGES) {
      const def = STEP_DEFS[s];
      const texts = [def.name, def.short, def.summary, ...def.main.flatMap((e) => [e.label, e.note ?? '']), ...def.parts.flatMap((p) => [p.name, p.meaning])];
      // Inline mathematics (between dollar signs) is an equation, taken from the documents and checked below.
      for (const t of texts) expect(t.replace(/\$[^$]*\$/g, ''), `step ${s}: "${t}"`).not.toMatch(/\d/);
    }
  });
});

describe('KaTeX renders every LaTeX string without error', () => {
  it('equations with their numbers, highlights, symbols, units and switches', () => {
    const strings: [string, boolean][] = [];
    for (const s of STAGES) {
      for (const e of mainEquations(s)) strings.push([`${e.latex} \\tag{${equationNumber(e) ?? ''}}`, true]);
      for (const p of STEP_DEFS[s].parts) {
        strings.push([p.symbol, false]);
        for (const u of p.units) strings.push([u.symbol, false], [u.unit, false]);
      }
    }
    for (const [, e] of allEquations()) strings.push([`${e.latex} \\tag{${equationNumber(e) ?? ''}}`, true]);
    for (const sw of SWITCHES) strings.push([sw.latex, false]);
    for (const [latex, display] of strings) expect(() => renderTex(latex, display), latex).not.toThrow();
  });

  it('rejects an unknown command (errors are not hidden)', () => {
    expect(() => renderTex('\\notacommand{x}', true)).toThrow();
  });
});

describe('documentation and code anchors resolve', () => {
  type Ref = DocRef & { label?: string };
  const docRefs = (): Ref[] => [
    ...STAGES.flatMap((s) => [
      ...STEP_DEFS[s].parts.map((p) => p.derivation),
      ...STEP_DEFS[s].implementation.pseudocode,
      ...STEP_DEFS[s].implementation.derivation,
    ]),
    ...allEquations().map(([, e]) => ({ doc: e.source.doc, section: e.source.section })),
  ];

  it('every cited section is a heading of its document, with a unique GitHub anchor', () => {
    for (const ref of docRefs()) {
      const hs = headings(ref.doc);
      const matches = hs.filter((h) => h.text === ref.section);
      expect(matches.length, `${ref.doc}: "${ref.section}"`).toBe(1);
      const slug = headingSlug(ref.section);
      expect(hs.filter((h) => headingSlug(h.text) === slug).length, `anchor ${slug} unique`).toBe(1);
    }
  });

  it('the anchor function follows GitHub slugs', () => {
    expect(headingSlug('5. The matrix rows')).toBe('5-the-matrix-rows');
    expect(headingSlug('PS-10 `ThomasFactors` : tridiagonal solution')).toBe('ps-10-thomasfactors--tridiagonal-solution');
    expect(headingSlug("1. From conservation of mass and Fick's law to the spherical diffusion equation")).toBe(
      '1-from-conservation-of-mass-and-ficks-law-to-the-spherical-diffusion-equation',
    );
  });

  it('every document path exists', () => {
    for (const d of Object.values(DOCS)) expect(readFileSync(join(REPO, d.path), 'utf8').length).toBeGreaterThan(0);
  });

  /** The lines of a source file of the BTCS solver: the working tree, else the tip of the solver branch. */
  const cache = new Map<string, string[] | null>();
  function sourceLines(path: string): string[] | null {
    if (!cache.has(path)) cache.set(path, readSourceLines(path));
    return cache.get(path) ?? null;
  }
  function readSourceLines(path: string): string[] | null {
    if (existsSync(join(REPO, path))) return readFileSync(join(REPO, path), 'utf8').split('\n');
    try {
      return execFileSync('git', ['show', `origin/feat/btcs-solver:${path}`], { cwd: REPO, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'], maxBuffer: 16 * 1024 * 1024 }).split('\n');
    } catch {
      return null;
    }
  }

  const refs = STAGES.flatMap((s) => STEP_DEFS[s].implementation.code);
  const available = refs.every((c) => sourceLines(c.path) !== null);

  it('has code links for the steps that have code, and none for the steps that do not', () => {
    for (const s of [1, 2, 3, 4, 14] as const) expect(STEP_DEFS[s].implementation.code, `step ${s}`).toEqual([]);
    for (const s of [5, 6, 7, 8, 9, 10, 11, 12, 13] as const) expect(STEP_DEFS[s].implementation.code.length, `step ${s}`).toBeGreaterThan(0);
    for (const c of refs) expect(c.path).toMatch(/^(src|tests)\/.+\.rs$/);
    expect(STEP_DEFS[11].implementation.code.map((c) => c.symbol)).toContain('pub struct ThomasFactors');
  });

  // The files of the solver are on branch feat/btcs-solver. In a checkout that holds neither the files
  // nor that branch the check cannot run; it then says so instead of passing silently.
  it.skipIf(!available)('every code link points at the line that defines the named item', () => {
    for (const c of refs) {
      const lines = sourceLines(c.path) ?? [];
      expect(lines[c.line - 1], `${c.path}#L${c.line}`).toContain(c.symbol);
    }
  });

  // The mechanism steps cite Rust that is on a development branch, not in this build: named by function and
  // equation, never linked and never given a line number (the file is not on the published branch).
  const devRefs = MECHANISM_STAGES.flatMap((s) => STEP_DEFS[s].implementation.devCode ?? []);
  function devSource(path: string): string | null {
    try {
      return execFileSync('git', ['show', `origin/feat/results-pipeline:${path}`], { cwd: REPO, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'], maxBuffer: 16 * 1024 * 1024 });
    } catch {
      return null;
    }
  }
  const devAvailable = devRefs.every((c) => devSource(c.path) !== null);

  it('a mechanism step links no code of this build and names the development code it cites', () => {
    for (const s of MECHANISM_STAGES) {
      expect(STEP_DEFS[s].implementation.code, `step ${s}`).toEqual([]);
      expect((STEP_DEFS[s].implementation.devCode ?? []).length, `step ${s}`).toBeGreaterThan(0);
    }
    for (const c of devRefs) {
      expect(c.path).toMatch(/^src\/.+\.rs$/);
      expect(Object.keys(c).sort(), 'no line number').toEqual(['label', 'path', 'symbol']);
    }
  });

  it.skipIf(!devAvailable)('every development-branch function named by a mechanism step is defined on origin/feat/results-pipeline', () => {
    for (const c of devRefs) expect(devSource(c.path) ?? '', `${c.path}: ${c.symbol}`).toContain(c.symbol);
  });

  it('the release-fraction step is no longer a placeholder', () => {
    expect(STEP_DEFS[14].implementation.note).not.toMatch(/placeholder/i);
    expect(STEP_DEFS[14].summary).not.toMatch(/placeholder|does not exist/i);
  });

  // Rewritten with the directive of 26 Nov 2026: the conditions are stated once (steps four and ten), the
  // balance disclosure is gone (its equations are the related equations of the accounting step), instead of
  // both being repeated under every step.
  it('the conditions are stated once, with the material model and with the assembly of the system', () => {
    expect(STAGES.filter((s) => conditionsFor(s) !== null)).toEqual([4, 10]);
    for (const s of [4, 10] as const) expect(conditionsFor(s)?.note.length, `step ${s}`).toBeGreaterThan(0);
    expect(conditionsFor(10)?.equations.map((e) => e.source.tag)).toEqual(['F1', 'F2', 'F18', 'F21', 'F22']);
    // The flux condition (27) is the main equation of step four and is not repeated; the outer surface has no tag in the document.
    expect(conditionsFor(4)?.equations.map((e) => e.source.tag)).toEqual(['26', null, '38']);
  });
});

describe('text encoding of the builder', () => {
  const SRC = join(REPO, 'web');
  const walk = (dir: string): string[] =>
    readdirSync(dir, { withFileTypes: true }).flatMap((d) => (d.isDirectory() ? (d.name === 'node_modules' ? [] : walk(join(dir, d.name))) : [join(dir, d.name)]));

  it('no builder string contains a mojibake sequence', () => {
    const strings: string[] = [];
    for (const s of STAGES) {
      const d = STEP_DEFS[s];
      strings.push(d.name, d.short, d.summary, d.implementation.note ?? '', d.live ?? '');
      const shown = [...d.main, ...d.implementation.equations, ...(conditionsFor(s)?.equations ?? [])];
      for (const e of shown) strings.push(e.label, e.note ?? '', e.latex);
      for (const p of d.parts) strings.push(p.name, p.meaning, p.symbol, ...p.units.flatMap((u) => [u.symbol, u.unit, u.meaning]));
      strings.push(conditionsFor(s)?.note ?? '');
      for (const p of ADDITION_PHYSICS) {
        const a = physicsAdditions(s, p);
        strings.push(a.note ?? '', ...a.equations.flatMap((e) => [e.label, e.note ?? '', e.latex]));
      }
    }
    strings.push(NOTATION, ...NUMERICAL_ITEMS.map((i) => i.title));
    for (const t of strings) expect(findMojibake(t), `"${t.slice(0, 60)}" holds damaged text`).toBeNull();
  });

  it('the damaged-text check fails on every damaged form that occurred and passes the correct text', () => {
    // Fixtures as escapes: the real damaged strings of the earlier builds, and their correct forms.
    const damaged = [
      'Fick\u00e2\u20ac\u2122s law put into the local balance',
      'divided by 4\u00cf\u20ac',
      'inventory \u00c2\u00b7 release',
      'error of order (\u00ce\u201dr) squared',
      'broken \ufffd character',
    ];
    for (const text of damaged) expect(findMojibake(text), text).not.toBeNull();
    const correct = ['Fick\u2019s law put into the local balance', 'divided by 4\u03c0', 'inventory \u00b7 release', 'order (\u0394r) squared'];
    for (const text of correct) expect(findMojibake(text), text).toBeNull();
  });

  it('inline mathematics in the prose is taken from Documents 1, 2 and 5, and no prose spells it out as plain text', () => {
    const prose: string[] = [NOTATION];
    for (const s of STAGES) {
      const d = STEP_DEFS[s];
      prose.push(d.summary, d.implementation.note ?? '', d.live ?? '', conditionsFor(s)?.note ?? '');
      for (const e of [...d.main, ...d.implementation.equations, ...(conditionsFor(s)?.equations ?? [])]) prose.push(e.note ?? '');
      for (const p of d.parts) prose.push(p.meaning);
    }
    let spans = 0;
    for (const text of prose) {
      const pieces = text.split('$');
      expect(pieces.length % 2, `unbalanced dollar signs in "${text.slice(0, 60)}"`).toBe(1);
      pieces.forEach((piece, i) => {
        if (i % 2 === 1) {
          spans++;
          renderTex(piece, false);
          expect(MATHS.includes(squash(piece)), `inline mathematics "${piece}" is not in Document 1 or 2`).toBe(true);
        } else {
          expect(/_\*|\(Delta r\)|squared/.test(piece), `plain-text mathematics in "${piece.slice(0, 60)}"`).toBe(false);
        }
      });
    }
    expect(spans).toBeGreaterThan(0);
  });

  it('this test file is pure ASCII, so that its patterns cannot be re-encoded unnoticed', () => {
    const own = readFileSync(fileURLToPath(import.meta.url), 'utf8');
    expect(Array.from(own).every((c) => c.charCodeAt(0) < 128)).toBe(true);
  });

  it('no source file of the page starts with a byte-order mark or holds a mojibake sequence', () => {
    for (const dir of ['src', 'tests', 'e2e']) {
      for (const f of walk(join(SRC, dir)).filter((p) => /\.(ts|tsx|css)$/.test(p))) {
        const b = readFileSync(f);
        expect(b[0] === 0xef && b[1] === 0xbb && b[2] === 0xbf, `${f} starts with a byte-order mark`).toBe(false);
        expect(findMojibake(b.toString('utf8')), `${f} holds damaged text`).toBeNull();
      }
    }
  });
});