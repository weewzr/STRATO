// @vitest-environment node
// Staged equation builder: term assembly, LaTeX against the derivation documents, KaTeX rendering,
// and the documentation and code anchors (prompt/IMPLEMENTATION_ROADMAP.md Phase 2,
// prompt/TESTING_AND_VALIDATION.md Section 3 row 2).
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { DOCS, headingSlug, type DocId } from '../src/equation/links';
import {
  assemble,
  assembleStage,
  balanceFor,
  conditionsFor,
  DOMAIN,
  implementationFor,
  listedOnly,
  STAGE_DEFS,
  STAGES,
  stageTerms,
  SWITCH_MECHANISM,
  SWITCHES,
  switchIntroducedBy,
  systemTerms,
  TERMS,
  type EqSource,
  type ShownEquation,
  type SwitchSet,
  type TermId,
} from '../src/equation/terms';
import { renderTex } from '../src/equation/Tex';

const REPO = fileURLToPath(new URL('../..', import.meta.url));
const docText = (doc: DocId) => readFileSync(join(REPO, DOCS[doc].path), 'utf8');
const TEXT: Record<DocId, string> = { v1: docText('v1'), p2: docText('p2'), pseudo: docText('pseudo') };
const squash = (s: string) => s.replace(/\s+/g, '');

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

function applyNotation(src: EqSource): string {
  let s = src.verbatim;
  for (const [from, to] of src.notation ?? []) s = s.split(from).join(to);
  return s;
}

function checkSource(src: EqSource, displayed: string, what: string) {
  const block = src.tag ? taggedBlock(src.doc, src.tag) : sectionText(src.doc, src.section);
  expect(squash(block), `${what}: verbatim copy not found in ${src.tag ? `(${src.tag})` : src.section}`).toContain(squash(src.verbatim));
  // The tagged equation also lies in the cited section.
  if (src.tag) expect(sectionText(src.doc, src.section), `${what}: (${src.tag}) is not in section "${src.section}"`).toContain(`\\tag{${src.tag}}`);
  expect(squash(applyNotation(src)), `${what}: displayed LaTeX differs from the copy after the stated notation changes`).toBe(
    squash(stripHtmlClass(displayed)),
  );
}

const allShown = (): [string, ShownEquation][] =>
  STAGES.flatMap((s) => [
    ...conditionsFor(s).equations.map((e) => [`stage ${s} condition ${e.id}`, e] as [string, ShownEquation]),
    ...implementationFor(s).equations.map((e) => [`stage ${s} implementation ${e.id}`, e] as [string, ShownEquation]),
    ...balanceFor(s).equations.map((e) => [`stage ${s} balance ${e.id}`, e] as [string, ShownEquation]),
  ]);

describe('LaTeX of every term matches the cited equation of the derivation documents', () => {
  it('terms of the transport equation and the system', () => {
    for (const t of Object.values(TERMS)) {
      if (t.latex === '') {
        expect(t.sources, `${t.id} is listed only and cites no equation`).toEqual([]);
        continue;
      }
      expect(t.sources.length, `${t.id} has a source`).toBeGreaterThan(0);
      for (const s of t.sources) checkSource(s, t.latex, `term ${t.id}`);
      if (t.formula) checkSource(t.formula.source, t.formula.latex, `formula of ${t.id}`);
    }
  });

  it('domain of the layer equation', () => {
    checkSource(DOMAIN.source, DOMAIN.latex, 'domain');
  });

  it('conditions, discrete updates and balances of every stage', () => {
    for (const [what, e] of allShown()) checkSource(e.source, e.latex, what);
  });

  it('highlights only parts of the copied equation', () => {
    for (const [what, e] of allShown()) {
      if (e.latex.includes('\\htmlClass')) expect(squash(stripHtmlClass(e.latex)), what).toBe(squash(applyNotation(e.source)));
    }
  });

  it('the stripping helper removes nested wrappers and keeps their content', () => {
    expect(stripHtmlClass('a \\htmlClass{x y}{b + \\frac{c}{d}} e')).toBe('a b + \\frac{c}{d} e');
    expect(stripHtmlClass('\\htmlClass{x}{\\htmlClass{y}{z}}')).toBe('z');
  });
});

describe('term assembly for the six builder stages', () => {
  const expected: Record<number, { terms: TermId[]; tag: string; system: TermId[]; listed: TermId[] }> = {
    1: { terms: ['lhs', 'diffusion'], tag: '2.6', system: [], listed: [] },
    2: { terms: ['lhs', 'diffusion', 'source'], tag: 'P21.1', system: [], listed: [] },
    3: { terms: ['lhs', 'diffusion', 'decay', 'source'], tag: 'P21.1', system: [], listed: [] },
    4: { terms: ['lhs', 'diffusion_T', 'decay', 'source'], tag: 'P22.11', system: [], listed: [] },
    5: { terms: ['lhs', 'diffusion_T', 'decay', 'source'], tag: 'P22.11', system: ['arrhenius_layers'], listed: ['partition'] },
    6: { terms: ['lhs', 'diffusion_T', 'decay', 'source'], tag: 'P22.11', system: ['arrhenius_layers'], listed: ['partition'] },
  };

  for (const s of STAGES) {
    it(`stage ${s} shows exactly its terms and highlights exactly its new term`, () => {
      const a = assembleStage(s);
      const want = expected[s]!;
      expect(a.terms).toEqual(want.terms);
      expect(a.tag).toBe(want.tag);
      expect(systemTerms(s)).toEqual(want.system);
      expect(listedOnly(s)).toEqual(want.listed);
      expect(stageTerms(s)).toEqual([...want.terms, ...want.system, ...want.listed]);
      // Each term appears once, and only the stage's new terms carry the highlight.
      for (const t of a.terms) expect(a.latex.split(`eq-term--${t}`).length - 1, `${t} once`).toBe(1);
      const highlighted = a.terms.filter((t) => new RegExp(`eq-term--${t} eq-new`).test(a.latex));
      expect(highlighted).toEqual(STAGE_DEFS[s].newTerms.filter((t) => a.terms.includes(t)));
      // Stage 1 is always visible as the base: its two terms are in every stage.
      expect(a.terms[0]).toBe('lhs');
      expect(['diffusion', 'diffusion_T']).toContain(a.terms[1]);
    });
  }

  it('stage 5 introduces the layer laws and lists the partition coefficients as not in the model', () => {
    expect(STAGE_DEFS[5].newTerms).toEqual(['arrhenius_layers', 'partition']);
    expect(TERMS.partition.latex).toBe('');
    expect(TERMS.arrhenius_layers.meaning).toMatch(/layering itself is already in stage one/);
    expect(STAGE_DEFS[5].summary).toMatch(/layering itself is already in stage one/);
  });

  it('keeps the stage picker and the switches in step', () => {
    expect(switchIntroducedBy(1)).toBeNull();
    expect(switchIntroducedBy(2)).toBe('source');
    expect(switchIntroducedBy(3)).toBe('decay');
    expect(switchIntroducedBy(4)).toBe('arrhenius');
    expect(switchIntroducedBy(5)).toBe('arrhenius');
    expect(switchIntroducedBy(6)).toBeNull();
    expect(SWITCHES.map((s) => s.id)).toEqual(['source', 'decay', 'arrhenius']);
    expect(SWITCH_MECHANISM).toEqual({ source: 'generation', decay: 'decay', arrhenius: 'diffusivity_temperature' });
  });
});

describe('term assembly for all eight combinations of the three Phase 6 switches', () => {
  const combos: SwitchSet[] = [];
  for (const source of [false, true]) for (const decay of [false, true]) for (const arrhenius of [false, true]) combos.push({ source, decay, arrhenius });

  it('has eight distinct combinations', () => {
    expect(new Set(combos.map((c) => JSON.stringify(c))).size).toBe(8);
  });

  for (const sw of combos) {
    it(`source=${sw.source} decay=${sw.decay} arrhenius=${sw.arrhenius}`, () => {
      const a = assemble(sw);
      const want: TermId[] = ['lhs', sw.arrhenius ? 'diffusion_T' : 'diffusion'];
      if (sw.decay) want.push('decay');
      if (sw.source) want.push('source');
      expect(a.terms).toEqual(want);
      const any = sw.source || sw.decay || sw.arrhenius;
      expect(a.tag).toBe(sw.arrhenius ? 'P22.11' : any ? 'P21.1' : '2.6');
      if (any) {
        expect(a.reductions.includes(String.raw`\lambda = 0`)).toBe(!sw.decay);
        expect(a.reductions.includes('S = 0')).toBe(!sw.source);
      } else {
        expect(a.reductions[0]).toBe('S = 0');
      }
      // No highlight unless asked for, and the equation typesets.
      expect(a.latex).not.toContain('eq-new');
      expect(() => renderTex(`${a.latex} \\tag{${a.tag}}`, true)).not.toThrow();
      // The displayed equation is the cited one after removing the switched-off terms.
      const shown = squash(stripHtmlClass(a.latex));
      for (const t of a.terms) expect(shown).toContain(squash(TERMS[t].latex));
    });
  }
});

describe('KaTeX renders every LaTeX string without error', () => {
  it('equations, formulas, units and domain', () => {
    const strings: [string, boolean][] = [[DOMAIN.latex, false]];
    for (const s of STAGES) {
      const a = assembleStage(s);
      strings.push([`${a.latex} \\tag{${a.tag}}`, true]);
      for (const r of a.reductions) strings.push([r, false]);
    }
    for (const t of Object.values(TERMS)) {
      if (t.latex) strings.push([t.latex, true]);
      if (t.formula) strings.push([`${t.formula.latex} \\tag{${t.formula.source.tag ?? ''}}`, true]);
      for (const u of t.units) strings.push([u.symbol, false], [u.unit, false]);
    }
    for (const [, e] of allShown()) strings.push([e.source.tag ? `${e.latex} \\tag{${e.source.tag}}` : e.latex, true]);
    for (const sw of SWITCHES) strings.push([sw.latex, false]);
    for (const [latex, display] of strings) expect(() => renderTex(latex, display), latex).not.toThrow();
  });

  it('rejects an unknown command (errors are not hidden)', () => {
    expect(() => renderTex('\\notacommand{x}', true)).toThrow();
  });
});

describe('documentation and code anchors resolve', () => {
  const docRefs = () => [
    ...Object.values(TERMS).map((t) => t.derivation),
    ...STAGES.flatMap((s) => [...implementationFor(s).pseudocode, ...implementationFor(s).derivation]),
    ...allShown().map(([, e]) => ({ doc: e.source.doc, section: e.source.section })),
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
    expect(headingSlug('2.6 Layer equation of v1.0.0')).toBe('26-layer-equation-of-v100');
    expect(headingSlug('4.3 (b) Backward Euler with implicit decay')).toBe('43-b-backward-euler-with-implicit-decay');
    expect(headingSlug('5.4 (d) General multilayer case: layer-specific activation energies')).toBe(
      '54-d-general-multilayer-case-layer-specific-activation-energies',
    );
  });

  it('every code link points at the line that defines the named function', () => {
    const refs = STAGES.flatMap((s) => implementationFor(s).code);
    expect(refs.length).toBeGreaterThan(0);
    for (const c of refs) {
      const lines = readFileSync(join(REPO, c.path), 'utf8').split('\n');
      expect(lines[c.line - 1], `${c.path}#L${c.line}`).toContain(c.symbol);
    }
  });

  it('every document path exists', () => {
    for (const d of Object.values(DOCS)) expect(readFileSync(join(REPO, d.path), 'utf8').length).toBeGreaterThan(0);
  });

  it('a stage without code says so and links the pseudocode', () => {
    for (const s of [2, 3, 4, 5] as const) {
      const impl = implementationFor(s);
      expect(impl.code).toEqual([]);
      expect(impl.note).toMatch(/^Derived, not implemented/);
      expect(impl.pseudocode.length).toBeGreaterThan(0);
    }
    expect(implementationFor(1).code.map((c) => c.symbol)).toContain('pub fn backward_euler_step');
  });
});
