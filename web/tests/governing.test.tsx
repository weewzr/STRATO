// The governing equation of the Explore page (owner directive of 26 Nov 2026, Section 2; decision 70):
// the structure of the TeX, the highlight and reduced-motion behaviour, keyboard operation, and the
// status and the simulation derived from the capability object of the build.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import {
  buildCapability,
  isSimulable,
  MECHANISMS,
  PHYSICS_IDS,
  runMechanisms,
  selectionSupport,
  STATUS_DEFINITION,
  STATUS_LABEL,
  type PhysicsId,
} from '../src/equation/capability';
import { equationPieces, equationTex, meaningsFor } from '../src/equation/physics';
import { renderTex } from '../src/equation/Tex';
import { type Catalogue } from '../src/research/catalogue';
import { fixtureCatalogue } from './catalogueFixture';

// The owner's equations, as the directive writes them (r^2 for the power of r).
const DIRECTIVE_BASE = String.raw`\frac{\partial C}{\partial t}=\frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2D\frac{\partial C}{\partial r}\right)`;
const DIRECTIVE_FINAL = String.raw`\frac{\partial C}{\partial t}=\frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2D(r,T)\frac{\partial C}{\partial r}\right)+G-\lambda C`;
/** Whitespace, a thin space and the notation of a parenthesised power are the only differences allowed. */
const normal = (tex: string) => tex.replace(/\\,/g, '').replace(/\s+/g, '').replace(/\\left\(r\\right\)\^2/g, 'r^2');

const BASE = equationTex('baseline');
const ALL_VERIFIED = fixtureCatalogue({ generation: 'VERIFIED', decay: 'VERIFIED', diffusivity_temperature: 'VERIFIED' });
const ALL_REPORTED = ['diffusion_btcs_layered', 'generation', 'decay', 'diffusivity_temperature'];

describe('the TeX of the governing equation', () => {
  it('typesets the directive power r^2 without parentheses', () => {
    expect(equationTex('combined')).not.toContain(String.raw`\left(r\right)`);
    expect(equationTex('combined')).toContain('r^2');
  });

  it('shows the directive equations: the baseline first and the combined model last', () => {
    expect(normal(BASE)).toBe(normal(DIRECTIVE_BASE));
    expect(normal(equationTex('combined'))).toBe(normal(DIRECTIVE_FINAL));
  });

  it('keeps the baseline equation as the beginning of every variant', () => {
    for (const id of PHYSICS_IDS.filter((p) => p !== 'temperature' && p !== 'combined' && p !== 'future')) expect(equationTex(id).startsWith(BASE), id).toBe(true);
    // Future species-specific physics shows the combined equation, unchanged.
    expect(equationTex('future')).toBe(equationTex('combined'));
  });

  it('puts +G and the decay term last on the right-hand side, after the whole diffusion operator', () => {
    expect(equationTex('generation')).toBe(`${BASE} +G`);
    expect(equationTex('decay')).toBe(`${BASE} -\\lambda C`);
    expect(equationTex('combined').endsWith('+G -\\lambda C')).toBe(true);
    expect(equationPieces('combined').map((p) => p.id)).toEqual(['lhs', 'operator', 'generation', 'decay']);
  });

  it('changes only the coefficient inside the operator for temperature dependence, and adds no term', () => {
    const t = equationTex('temperature');
    expect(t.replace('D(r,T)', 'D')).toBe(BASE);
    expect(t).not.toMatch(/[+-]/);
    expect(t.indexOf('D(r,T)')).toBeGreaterThan(t.indexOf('\\left('));
    expect(t.indexOf('D(r,T)')).toBeLessThan(t.lastIndexOf('\\right)'));
    expect(equationPieces('temperature').map((p) => p.id)).toEqual(['lhs', 'operator']);
  });

  it('puts the temperature coefficient inside the operator also in the combined model', () => {
    expect(equationTex('combined').replace('D(r,T)', 'D')).toBe(`${BASE} +G -\\lambda C`);
  });

  it('shows exactly the TeX of the structure: the shown pieces without the highlight marks equal the tex pieces', () => {
    const strip = (s: string) => s.replace(/\\htmlClass\{[^}]*\}\{((?:[^{}]|\{[^{}]*\})*)\}/g, '$1');
    for (const id of PHYSICS_IDS) for (const p of equationPieces(id)) expect(strip(p.shown), `${id}/${p.id}`).toBe(p.tex);
  });

  it('typesets every variant, with and without the highlight marks, without a KaTeX error', () => {
    for (const id of PHYSICS_IDS) for (const p of equationPieces(id)) {
      expect(() => renderTex(String.raw`\displaystyle ${p.shown}`, false), `${id}/${p.id}`).not.toThrow();
      expect(() => renderTex(p.tex, false)).not.toThrow();
    }
  });

  it('gives each selection its terms in meaning: two baseline terms plus the selected mechanisms, each with a unit', () => {
    expect(meaningsFor('baseline').map((t) => t.id)).toEqual(['rate', 'diffusion']);
    expect(meaningsFor('decay').map((t) => t.id)).toEqual(['rate', 'diffusion', 'decay']);
    expect(meaningsFor('combined').map((t) => t.id)).toEqual(['rate', 'diffusion', 'temperature', 'generation', 'decay']);
    for (const t of meaningsFor('combined')) {
      expect(t.meaning.length).toBeGreaterThan(20);
      expect(t.unit).not.toBe('');
    }
  });
});

describe('the capability object', () => {
  it('is baseline only for the wrapper of this build, and everything else planned', () => {
    const cap = buildCapability(['diffusion_btcs_layered']);
    expect(cap.baseline).toBe('verified');
    for (const m of MECHANISMS) expect(cap.mechanisms[m]).toBe('planned');
    expect(selectionSupport(cap, 'baseline', null).supported).toBe(true);
    expect(selectionSupport(cap, 'generation', null)).toEqual({ supported: false, missing: ['fission-product generation'] });
    expect(selectionSupport(cap, 'combined', null).missing).toHaveLength(3);
    expect(selectionSupport(cap, 'baseline', 'startup').missing).toEqual(['the startup scenario']);
  });

  it('defines each of the three statuses in one sentence', () => {
    for (const s of ['implemented', 'verified', 'planned'] as const) {
      expect(STATUS_LABEL[s]).toMatch(/^[A-Z][a-z]+$/);
      expect(STATUS_DEFINITION[s].match(/\./g)).toHaveLength(1);
    }
  });

  it('follows a fake wrapper report: planned, then implemented, then verified', () => {
    const real = (reported: string[], cat?: Catalogue) => buildCapability(reported, cat);
    expect(real(['diffusion_btcs_layered']).mechanisms.generation).toBe('planned');
    // Reported by the wrapper, but the catalogue does not record the verification: implemented, not simulable.
    const implemented = real(ALL_REPORTED);
    expect(implemented.mechanisms.generation).toBe('implemented');
    expect(isSimulable(implemented.mechanisms.generation)).toBe(false);
    expect(selectionSupport(implemented, 'generation', null).supported).toBe(false);
    // Reported and recorded as verified: simulable, and the others stay planned.
    const verified = real(['diffusion_btcs_layered', 'generation'], ALL_VERIFIED);
    expect(verified.mechanisms).toEqual({ generation: 'verified', decay: 'planned', temperature: 'planned' });
    expect(selectionSupport(verified, 'generation', null).supported).toBe(true);
    expect(selectionSupport(verified, 'combined', null).missing).toEqual(['radioactive decay', 'temperature-dependent diffusion']);
    // A catalogue entry alone, without the wrapper's report, never makes a mechanism available.
    expect(real(['diffusion_btcs_layered'], ALL_VERIFIED).mechanisms.generation).toBe('planned');
  });

  it('asks the wrapper for the baseline and the selected mechanisms only', () => {
    expect(runMechanisms('baseline')).toEqual(['diffusion_btcs_layered']);
    expect(runMechanisms('temperature')).toEqual(['diffusion_btcs_layered', 'diffusivity_temperature']);
    expect(runMechanisms('combined')).toEqual(['diffusion_btcs_layered', 'generation', 'decay', 'diffusivity_temperature']);
  });
});

describe('the highlight and the reduced-motion rule in the style sheet', () => {
  const css = readFileSync(join(process.cwd(), 'src', 'design', 'app.css'), 'utf8');
  const block = (re: RegExp) => css.match(re)?.[0] ?? '';

  it('marks a new term with an outline that needs no motion', () => {
    const rule = (selector: string) => (css.match(new RegExp(`(?:^|\\n)${selector.replace(/\./g, '\\.')} \\{[^}]*\\}`, 'g')) ?? []).join('\n');
    expect(rule('.geq-new')).toMatch(/outline: 2px solid/);
    expect(rule('.geq-hi')).toMatch(/outline: 2px solid/);
  });

  it('moves and fades a new term only when motion is allowed', () => {
    const allowed = block(/@media \(prefers-reduced-motion: no-preference\) \{[\s\S]*?\n\}\n/);
    expect(allowed).toMatch(/animation: geq-in/);
    expect(allowed).toMatch(/animation: geq-fade/);
    // No animation of the equation is declared outside that media query.
    const outside = css.replace(allowed, '');
    expect(outside).not.toMatch(/animation: geq-/);
    expect(css).toMatch(/@keyframes geq-in \{[^}]*\{[^}]*translateX/);
  });
});

describe('the hierarchy of Explore', () => {
  it('is typed with the TeX of every physics id', () => {
    const ids: PhysicsId[] = [...PHYSICS_IDS];
    expect(ids.every((id) => equationTex(id).length > 0)).toBe(true);
  });
});
