// The governing equation of the Explore page (owner directive of 26 Nov 2026, Section 2; decision 70):
// the structure of the TeX, the highlight and reduced-motion behaviour, keyboard operation, and the
// status and the simulation derived from the capability object of the build.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
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
import { GoverningEquation } from '../src/equation/GoverningEquation';
import { equationPieces, equationTex, meaningsFor, PHYSICS_CHANGE, PHYSICS_LABEL } from '../src/equation/physics';
import { renderTex } from '../src/equation/Tex';
import { CatalogueContext, type Catalogue } from '../src/research/catalogue';
import { Explore } from '../src/sections/Explore';
import { useStrato } from '../src/state/store';
import { fixtureCatalogue } from './catalogueFixture';

// The owner's equations, as the directive writes them (r^2 for the power of r).
const DIRECTIVE_BASE = String.raw`\frac{\partial C}{\partial t}=\frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2D\frac{\partial C}{\partial r}\right)`;
const DIRECTIVE_FINAL = String.raw`\frac{\partial C}{\partial t}=\frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2D(r,T)\frac{\partial C}{\partial r}\right)+G-\lambda C`;
/** Whitespace, a thin space and the notation of a parenthesised power are the only differences allowed. */
const normal = (tex: string) => tex.replace(/\\,/g, '').replace(/\s+/g, '').replace(/\\left\(r\\right\)\^2/g, 'r^2');

const BASE = equationTex('baseline');
const withCatalogue = (cat: Catalogue, ui: React.ReactElement) => <CatalogueContext.Provider value={cat}>{ui}</CatalogueContext.Provider>;
const ALL_VERIFIED = fixtureCatalogue({ generation: 'VERIFIED', decay: 'VERIFIED', diffusivity_temperature: 'VERIFIED' });
const ALL_REPORTED = ['diffusion_btcs_layered', 'generation', 'decay', 'diffusivity_temperature'];

describe('the TeX of the governing equation', () => {
  it('shows the directive equations: the baseline first and the combined model last', () => {
    expect(normal(BASE)).toBe(normal(DIRECTIVE_BASE));
    expect(normal(equationTex('combined'))).toBe(normal(DIRECTIVE_FINAL));
  });

  it('keeps the baseline equation as the beginning of every variant', () => {
    for (const id of PHYSICS_IDS.filter((p) => p !== 'temperature' && p !== 'combined')) expect(equationTex(id).startsWith(BASE), id).toBe(true);
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
    expect(selectionSupport(cap, 'baseline', 'steady').supported).toBe(true);
    expect(selectionSupport(cap, 'generation', 'steady')).toEqual({ supported: false, missing: ['fission-product generation'] });
    expect(selectionSupport(cap, 'combined', 'steady').missing).toHaveLength(3);
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
    expect(selectionSupport(implemented, 'generation', 'steady').supported).toBe(false);
    // Reported and recorded as verified: simulable, and the others stay planned.
    const verified = real(['diffusion_btcs_layered', 'generation'], ALL_VERIFIED);
    expect(verified.mechanisms).toEqual({ generation: 'verified', decay: 'planned', temperature: 'planned' });
    expect(selectionSupport(verified, 'generation', 'steady').supported).toBe(true);
    expect(selectionSupport(verified, 'combined', 'steady').missing).toEqual(['radioactive decay', 'temperature-dependent diffusion']);
    // A catalogue entry alone, without the wrapper's report, never makes a mechanism available.
    expect(real(['diffusion_btcs_layered'], ALL_VERIFIED).mechanisms.generation).toBe('planned');
  });

  it('asks the wrapper for the baseline and the selected mechanisms only', () => {
    expect(runMechanisms('baseline')).toEqual(['diffusion_btcs_layered']);
    expect(runMechanisms('temperature')).toEqual(['diffusion_btcs_layered', 'diffusivity_temperature']);
    expect(runMechanisms('combined')).toEqual(['diffusion_btcs_layered', 'generation', 'decay', 'diffusivity_temperature']);
  });
});

describe('the governing-equation block', () => {
  const selected = () => screen.getByRole('radio', { checked: true });

  it('shows the five choices as one radio group with the baseline selected, each with its status', () => {
    act(() => useStrato.getState().setCapabilities(['diffusion_btcs_layered']));
    render(<GoverningEquation />);
    const group = screen.getByRole('radiogroup', { name: 'Physics of the governing equation' });
    const radios = within(group).getAllByRole('radio');
    expect(radios.map((r) => r.querySelector('.physics__name')?.textContent)).toEqual(PHYSICS_IDS.map((p) => PHYSICS_LABEL[p]));
    expect(selected().dataset.testid).toBe('physics-baseline');
    expect(radios.map((r) => r.getAttribute('tabindex'))).toEqual(['0', '-1', '-1', '-1', '-1']);
    expect(screen.getByTestId('physics-status-baseline').textContent).toBe('Verified');
    for (const id of ['generation', 'decay', 'temperature', 'combined']) expect(screen.getByTestId(`physics-status-${id}`).textContent).toBe('Planned');
  });

  it('keeps the base pieces mounted while a term enters, and highlights only what is new', async () => {
    const user = userEvent.setup();
    render(<GoverningEquation />);
    const lhs = document.querySelector('[data-piece="lhs"]');
    const op = document.querySelector('[data-piece="operator"]');
    expect(document.querySelectorAll('.geq-new')).toHaveLength(0);
    await user.click(screen.getByTestId('physics-generation'));
    expect(document.querySelector('[data-piece="lhs"]')).toBe(lhs);
    expect(document.querySelector('[data-piece="operator"]')).toBe(op);
    const g = document.querySelector('[data-piece="generation"]');
    expect(g?.classList.contains('geq-new')).toBe(true);
    expect(lhs?.classList.contains('geq-new')).toBe(false);
    expect(op?.classList.contains('geq-new')).toBe(false);
    await user.click(screen.getByTestId('physics-decay'));
    expect(document.querySelector('[data-piece="generation"]')).toBeNull();
    expect(document.querySelector('[data-piece="decay"]')?.classList.contains('geq-new')).toBe(true);
    expect(document.querySelector('[data-piece="lhs"]')).toBe(lhs);
    expect(document.querySelector('[data-piece="operator"]')).toBe(op);
  });

  it('marks the coefficient D(r,T) inside the operator for temperature dependence and adds no piece', async () => {
    const user = userEvent.setup();
    render(<GoverningEquation />);
    await user.click(screen.getByTestId('physics-temperature'));
    const pieces = Array.from(document.querySelectorAll('[data-piece]')).map((p) => (p as HTMLElement).dataset.piece);
    expect(pieces).toEqual(['lhs', 'operator']);
    const hi = document.querySelector('[data-piece="operator"] .geq-hi');
    expect(hi).not.toBeNull();
    expect(hi?.textContent).toContain('D(r,T)');
    // The base equation without the highlight is the baseline's piece, apart from the coefficient.
    expect(document.querySelectorAll('.geq-hi')).toHaveLength(1);
  });

  it('shows the whole final equation, all three terms highlighted, for the combined model', async () => {
    const user = userEvent.setup();
    render(<GoverningEquation />);
    await user.click(screen.getByTestId('physics-combined'));
    expect(Array.from(document.querySelectorAll('[data-piece]')).map((p) => (p as HTMLElement).dataset.piece)).toEqual(['lhs', 'operator', 'generation', 'decay']);
    expect(document.querySelectorAll('.geq-new, .geq-hi').length).toBeGreaterThanOrEqual(3);
    expect(screen.getByTestId('term-meanings').querySelectorAll('li')).toHaveLength(5);
    expect(screen.getByTestId('term-meanings').querySelectorAll('li.is-new')).toHaveLength(3);
  });

  it('announces what changed in a live region, but not on the first render', async () => {
    const user = userEvent.setup();
    render(<GoverningEquation />);
    const live = screen.getByTestId('physics-announce');
    expect(live.getAttribute('aria-live')).toBe('polite');
    expect(live.textContent).toBe('');
    await user.click(screen.getByTestId('physics-generation'));
    expect(live.textContent).toBe(PHYSICS_CHANGE.generation);
    expect(live.textContent).toMatch(/enters the right-hand side/);
    await user.click(screen.getByTestId('physics-temperature'));
    expect(live.textContent).toMatch(/inside the diffusion operator; no term is added/);
    // A change made through the address announces too.
    act(() => useStrato.getState().setSelection({ physics: 'decay' }));
    expect(live.textContent).toBe(PHYSICS_CHANGE.decay);
  });

  it('is operated by the keyboard: one tab stop, arrows select and move focus, Home and End jump', async () => {
    const user = userEvent.setup();
    render(<GoverningEquation />);
    screen.getByTestId('physics-baseline').focus();
    await user.keyboard('{ArrowRight}{ArrowRight}');
    expect(useStrato.getState().selection.physics).toBe('decay');
    expect(document.activeElement).toBe(screen.getByTestId('physics-decay'));
    await user.keyboard('{End}');
    expect(useStrato.getState().selection.physics).toBe('combined');
    await user.keyboard('{ArrowRight}');
    expect(useStrato.getState().selection.physics).toBe('baseline');
    await user.keyboard('{ArrowLeft}');
    expect(useStrato.getState().selection.physics).toBe('combined');
    await user.keyboard('{Home}');
    expect(useStrato.getState().selection.physics).toBe('baseline');
    expect(screen.getByTestId('physics-combined').getAttribute('tabindex')).toBe('-1');
    // The status legend is a disclosure button.
    const toggle = screen.getByTestId('status-legend-toggle');
    expect(toggle.getAttribute('aria-expanded')).toBe('false');
    await user.click(toggle);
    expect(toggle.getAttribute('aria-expanded')).toBe('true');
    expect(within(screen.getByTestId('status-legend')).getAllByRole('term')).toHaveLength(3);
  });

  it('shows the status of every choice from a fake wrapper report', () => {
    act(() => useStrato.getState().setCapabilities(['diffusion_btcs_layered', 'generation']));
    const { unmount } = render(withCatalogue(ALL_VERIFIED, <GoverningEquation />));
    expect(screen.getByTestId('physics-status-generation').textContent).toBe('Verified');
    expect(screen.getByTestId('physics-status-decay').textContent).toBe('Planned');
    expect(screen.getByTestId('physics-status-combined').textContent).toBe('Planned');
    unmount();
    act(() => useStrato.getState().setCapabilities(ALL_REPORTED));
    render(<GoverningEquation />);
    expect(screen.getByTestId('physics-status-generation').textContent).toBe('Implemented');
    expect(screen.getByTestId('physics-status-combined').textContent).toBe('Implemented');
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

describe('the simulation and results follow the selection and the capability of the build', () => {
  const renderExplore = (cat?: Catalogue) => {
    const ui = <Explore compact={false} onOpenControls={() => undefined} />;
    return render(cat ? withCatalogue(cat, ui) : ui);
  };

  it('with the baseline selected the simulation and the results are shown', () => {
    act(() => useStrato.getState().setCapabilities(['diffusion_btcs_layered']));
    renderExplore();
    expect(screen.getByTestId('block-d').dataset.supported).toBe('true');
    expect(screen.getByTestId('particle')).toBeTruthy();
    expect(screen.queryByTestId('unsupported-simulation')).toBeNull();
    expect(screen.queryByTestId('unsupported-results')).toBeNull();
  });

  it('for a mechanism that this build does not implement, D and E say so and show no substituted result', () => {
    act(() => {
      useStrato.getState().setCapabilities(['diffusion_btcs_layered']);
      useStrato.getState().setSelection({ physics: 'generation' });
    });
    renderExplore();
    expect(screen.getByTestId('block-d').dataset.supported).toBe('false');
    expect(screen.getByTestId('unsupported-simulation').textContent).toMatch(/does not implement fission-product generation/);
    expect(screen.getByTestId('unsupported-results').textContent).toMatch(/no result is shown for this selection/);
    expect(screen.queryByTestId('particle')).toBeNull();
    expect(screen.queryByTestId('release-readout')).toBeNull();
    expect(screen.queryByTestId('profile-figure')).toBeNull();
    // The equation and its explanations stay.
    expect(screen.getByTestId('equation-governing')).toBeTruthy();
    expect(screen.getByTestId('physics-status-generation').textContent).toBe('Planned');
  });

  it('turns the simulation on without an edit when the wrapper reports the mechanism as verified', () => {
    act(() => {
      useStrato.getState().setCapabilities(['diffusion_btcs_layered', 'generation']);
      useStrato.getState().setSelection({ physics: 'generation' });
    });
    const { unmount } = renderExplore();
    expect(screen.getByTestId('block-d').dataset.supported).toBe('false');
    unmount();
    renderExplore(ALL_VERIFIED);
    expect(screen.getByTestId('block-d').dataset.supported).toBe('true');
    expect(screen.getByTestId('particle')).toBeTruthy();
    expect(screen.queryByTestId('unsupported-simulation')).toBeNull();
  });

  it('names every missing part for the combined model, and an unsupported scenario', () => {
    act(() => {
      useStrato.getState().setCapabilities(['diffusion_btcs_layered']);
      useStrato.getState().setSelection({ physics: 'combined', scenario: 'startup' });
    });
    renderExplore();
    const text = screen.getByTestId('unsupported-simulation').textContent ?? '';
    for (const part of ['fission-product generation', 'radioactive decay', 'temperature-dependent diffusion', 'startup scenario']) expect(text).toContain(part);
    expect(screen.getByTestId('block-e').dataset.supported).toBe('false');
  });

  it('while the wrapper has not reported, the selection is not called unsupported', () => {
    act(() => useStrato.getState().setSelection({ physics: 'decay' }));
    renderExplore();
    expect(screen.getByTestId('block-d').dataset.supported).toBe('true');
  });
});

describe('the hierarchy of Explore', () => {
  it('shows A, B, C, D, E in this order, with the Numerical Interpretation collapsed by default', () => {
    render(<Explore compact={false} onOpenControls={() => undefined} />);
    const blocks = Array.from(document.querySelectorAll<HTMLElement>('[data-section="explore"] [data-block]'));
    expect(blocks.map((b) => b.dataset.block)).toEqual(['A', 'B', 'C', 'D', 'E']);
    expect(screen.getByRole('heading', { name: 'Numerical Interpretation' })).toBeTruthy();
    const toggle = screen.getByTestId('numerical-toggle');
    expect(toggle.getAttribute('aria-expanded')).toBe('false');
    expect(screen.queryByTestId('equation-builder')).toBeNull();
    // The derivation is in this one region, and opening it is part of the shared selection.
    act(() => toggle.click());
    expect(useStrato.getState().selection.expanded).toEqual(['numerical']);
    expect(within(screen.getByTestId('block-c')).getByTestId('equation-builder')).toBeTruthy();
  });

  it('is typed with the TeX of every physics id', () => {
    const ids: PhysicsId[] = [...PHYSICS_IDS];
    expect(ids.every((id) => equationTex(id).length > 0)).toBe(true);
  });
});
