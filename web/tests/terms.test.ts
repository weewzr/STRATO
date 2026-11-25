// @vitest-environment node
// Progressive equation builder for the BTCS solver: every equation against the BTCS documents,
// KaTeX rendering, the notation rules of directive Section 5, and the documentation and code anchors
// (prompt/TESTING_AND_VALIDATION.md Section 3 row 2).
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { DOCS, headingSlug, type DocId, type DocRef } from '../src/equation/links';
import {
  allEquations,
  balanceFor,
  conditionsFor,
  CORE_STAGES,
  equationNumber,
  LAST_SOLVED_STEP,
  mainEquations,
  markParts,
  PREVIEW_STAGES,
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
const TEXT: Record<DocId, string> = { d1: docText('d1'), d2: docText('d2'), d3: docText('d3'), d4: docText('d4') };
const squash = (s: string) => s.replace(/\s+/g, '');
const MATHS = squash(TEXT.d1 + TEXT.d2);

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

function checkSource(src: EqSource, displayed: string, what: string, numberAs?: string) {
  const block = src.tag ? taggedBlock(src.doc, src.tag) : sectionText(src.doc, src.section);
  expect(squash(block), `${what}: verbatim copy not found in ${src.tag ? `(${src.tag})` : src.section}`).toContain(squash(src.verbatim));
  // The tagged equation also lies in the cited section.
  if (src.tag) expect(sectionText(src.doc, src.section), `${what}: (${src.tag}) is not in section "${src.section}"`).toContain(`\\tag{${src.tag}}`);
  // An untagged line that the page numbers by a name from the text: the section must use that name.
  if (!src.tag && numberAs) expect(sectionText(src.doc, src.section), `${what}: the name ${numberAs} is not in the section`).toContain(numberAs);
  // Nothing is rewritten: the displayed LaTeX is the copy.
  expect(squash(stripHtmlClass(displayed)), `${what}: displayed LaTeX differs from the document text`).toBe(squash(src.verbatim));
}

describe('the fourteen steps and the three previews', () => {
  it('has the fourteen steps of directive Section 16 in order, then three previews', () => {
    expect(CORE_STAGES.map((s) => STEP_DEFS[s].name)).toEqual([
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
      'Experimental comparisons',
    ]);
    expect(PREVIEW_STAGES.map((s) => STEP_DEFS[s].name)).toEqual([
      'Fission-product generation (preview)',
      'Radioactive decay (preview)',
      'Temperature-dependent diffusivity (preview)',
    ]);
    expect(STAGES.length).toBe(17);
    expect(CORE_STAGES.every((s) => STEP_DEFS[s].kind === 'step')).toBe(true);
    expect(PREVIEW_STAGES.every((s) => STEP_DEFS[s].kind === 'preview' && STEP_DEFS[s].mechanism !== null)).toBe(true);
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

  it('keeps the previews, the switches and the mechanism ids in step', () => {
    expect(SWITCHES.map((s) => s.id)).toEqual(['source', 'decay', 'arrhenius']);
    expect(SWITCH_MECHANISM).toEqual({ source: 'generation', decay: 'decay', arrhenius: 'diffusivity_temperature' });
    for (const sw of SWITCHES) {
      expect(switchIntroducedBy(sw.stage)).toBe(sw.id);
      expect(STEP_DEFS[sw.stage].kind).toBe('preview');
      expect(STEP_DEFS[sw.stage].mechanism).toBe(SWITCH_MECHANISM[sw.id]);
      expect(TERMS[sw.term]?.stage).toBe(sw.stage);
    }
    for (const s of CORE_STAGES) expect(switchIntroducedBy(s)).toBeNull();
  });
});

describe('every equation is the text of Document 1 or Document 2', () => {
  it('main, implementation, condition and balance equations of every step', () => {
    for (const [what, e] of allEquations()) {
      expect(e.latex, `${what}: no rewriting of the copied line`).toBe(e.source.verbatim);
      checkSource(e.source, e.latex, what, e.numberAs);
    }
  });

  it('the equation number shown is the number printed in the document', () => {
    for (const [what, e] of allEquations()) {
      const n = equationNumber(e);
      if (e.source.tag) expect(n, what).toBe(e.source.doc === 'd1' ? `D1.${e.source.tag}` : e.source.tag);
      else expect(n, what).toBe(e.numberAs ?? null);
      // Every number of Document 2 shown as an F-number is an F-number of that document.
      if (e.source.doc === 'd2' && e.source.tag) expect(e.source.tag, what).toMatch(/^F\d+[a-z]?$/);
      if (e.source.doc === 'd1' && e.source.tag) expect(e.source.tag, what).toMatch(/^\d+[a-z]?$/);
    }
  });

  it('only the BTCS documents are cited as the source of an equation', () => {
    for (const [, e] of allEquations()) expect(['d1', 'd2']).toContain(e.source.doc);
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

  it('every part marks a piece of a main equation of its step, and only the previews carry a mechanism colour', () => {
    for (const s of STAGES) {
      const def = STEP_DEFS[s];
      for (const p of def.parts) {
        if (p.latex !== '') {
          expect(
            def.main.some((e) => e.latex.includes(p.latex)),
            `step ${s} part ${p.id}: "${p.latex}" is not in a main equation`,
          ).toBe(true);
        }
        expect(p.highlight !== undefined, `part ${p.id}`).toBe(def.kind === 'preview');
        expect(p.mechanism !== null, `part ${p.id}`).toBe(def.kind === 'preview');
      }
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
    for (const sw of SWITCHES) expect(squash(TEXT.d2).includes(squash(sw.latex)), `switch ${sw.id}`).toBe(true);
  });

  it('the previews show only the places that Document 2, Section 9 marks', () => {
    const section9 = squash(sectionText('d2', '9. Placeholders for the approved additional physics (directive Section 17)'));
    for (const s of PREVIEW_STAGES) {
      for (const e of STEP_DEFS[s].main) {
        expect(e.source.doc).toBe('d2');
        expect(e.source.tag).toBeNull();
        expect(section9, `step ${s} ${e.id}`).toContain(squash(e.latex));
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
      for (const t of texts) expect(t, `step ${s}: "${t}"`).not.toMatch(/\d/);
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
    expect(STEP_DEFS[11].implementation.code.map((c) => c.symbol)).toContain('pub fn solve_in_place');
  });

  // The files of the solver are on branch feat/btcs-solver. In a checkout that holds neither the files
  // nor that branch the check cannot run; it then says so instead of passing silently.
  it.skipIf(!available)('every code link points at the line that defines the named item', () => {
    for (const c of refs) {
      const lines = sourceLines(c.path) ?? [];
      expect(lines[c.line - 1], `${c.path}#L${c.line}`).toContain(c.symbol);
    }
  });

  it('a preview or a placeholder says so and links no code', () => {
    for (const s of PREVIEW_STAGES) {
      expect(STEP_DEFS[s].implementation.code).toEqual([]);
      expect(STEP_DEFS[s].implementation.note).toMatch(/^Preview, not implemented/);
    }
    expect(STEP_DEFS[14].implementation.note).toMatch(/^Placeholder/);
  });

  it('conditions and balance disclosures of every step are defined', () => {
    for (const s of STAGES) {
      expect(conditionsFor(s).note.length).toBeGreaterThan(0);
      expect(balanceFor(s).note.length).toBeGreaterThan(0);
    }
  });
});
