// The operating-scenario row of the Explore page (owner directive of 26 Nov 2026, Sections 3, 5 and 6;
// decision 70(b); science design v3, Section 5): the five buttons, what each scenario changes in the
// equation (existing symbols only, never a term), the explanations, the coupling rule, the forbidden
// claims, the status and enabled state from the capability object (also for a fake object with scenarios
// implemented), keyboard operation and the address.
import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import {
  buildCapability,
  mechanismsOf,
  PHYSICS_IDS,
  SCENARIO_IDS,
  scenarioStatus,
  SCENARIO_NAME,
  selectionSupport,
  unsupportedMessage,
  WRAPPER_SCENARIO_ID,
  type PhysicsId,
  type ScenarioId,
} from '../src/equation/capability';
import { GoverningEquation } from '../src/equation/GoverningEquation';
import { ITEM_IDS, scenarioNote } from '../src/equation/numerics';
import { equationPieces, equationTex, D_SCENARIO, G_SCENARIO } from '../src/equation/physics';
import { renderTex } from '../src/equation/Tex';
import { ScenarioPicker } from '../src/equation/ScenarioPicker';
import { NO_SCENARIO_TEXT, paramsFor, SCENARIO_KIND, SCENARIO_LABEL, SCENARIO_RULE, SCENARIOS } from '../src/equation/scenarios';
import { CatalogueContext, type Catalogue } from '../src/research/catalogue';
import { Explore } from '../src/sections/Explore';
import { exploreHash, startAddressSync } from '../src/state/address';
import { useStrato } from '../src/state/store';
import { fixtureCatalogue } from './catalogueFixture';
import { claimsIn } from './forbiddenClaims';

const NON_STEADY = SCENARIO_IDS.filter((s) => s !== 'steady');
const MECH = { generation: 'generation', decay: 'decay', temperature: 'diffusivity_temperature' };
const REPORTED_MECHANISMS = ['diffusion_btcs_layered', ...Object.values(MECH)];
const VERIFIED_MECHANISMS = fixtureCatalogue({ generation: 'VERIFIED', decay: 'VERIFIED', diffusivity_temperature: 'VERIFIED' });

/** A catalogue in which the named scenario rows exist at the given status (the later integration adds such rows). */
function withScenarios(base: Catalogue, status: 'VERIFIED' | 'IMPLEMENTED', ids: ScenarioId[]): Catalogue {
  const template = base.mechanisms[0];
  if (!template) throw new Error('no mechanism row to copy');
  const rows = ids.map((s) => ({ ...template, mechanism_id: WRAPPER_SCENARIO_ID[s], status }));
  return { ...base, mechanisms: [...base.mechanisms, ...rows] };
}
const withCatalogue = (cat: Catalogue, ui: React.ReactElement) => <CatalogueContext.Provider value={cat}>{ui}</CatalogueContext.Provider>;

describe('the five scenario buttons', () => {
  it('are in the order and with the labels of the directive, in one radio group', () => {
    act(() => useStrato.getState().setCapabilities(['diffusion_btcs_layered']));
    render(<ScenarioPicker />);
    const group = screen.getByRole('radiogroup', { name: 'Operating scenario' });
    const radios = within(group).getAllByRole('radio');
    expect(radios.map((r) => r.querySelector('.physics__name')?.textContent)).toEqual([
      'Steady-state operation',
      'Shutdown and cooldown',
      'Startup',
      'Control-rod insertion',
      'Refuelling',
    ]);
    expect(radios.map((r) => r.dataset.testid)).toEqual(['scenario-steady', 'scenario-shutdown', 'scenario-startup', 'scenario-control-rod', 'scenario-refuelling']);
    expect(radios.map((r) => r.getAttribute('tabindex'))).toEqual(['0', '-1', '-1', '-1', '-1']);
    // No scenario is selected by default: the first button is the one tab stop, and none is checked.
    expect(radios.map((r) => r.getAttribute('aria-checked'))).toEqual(['false', 'false', 'false', 'false', 'false']);
  });

  it('sit directly beneath the physics controls: block A, then block B, in the page', () => {
    render(<Explore compact={false} onOpenControls={() => undefined} />);
    const blocks = Array.from(document.querySelectorAll<HTMLElement>('[data-section="explore"] [data-block]')).map((b) => b.dataset.block);
    expect(blocks.slice(0, 2)).toEqual(['A', 'B']);
  });
});

describe('what a scenario changes in the equation', () => {
  const COMBINED = equationTex('combined');

  it('changes G to G(r,t) and D(r,T) to D(r,T(t)) for every scenario except steady-state operation, and adds no term', () => {
    expect(equationTex('combined', 'steady')).toBe(COMBINED);
    for (const s of NON_STEADY) {
      const tex = equationTex('combined', s);
      expect(tex, s).toBe(COMBINED.replace('D(r,T)', D_SCENARIO).replace('+G', `+${G_SCENARIO}`));
      expect(equationPieces('combined', s).map((p) => p.id), s).toEqual(['lhs', 'operator', 'generation', 'decay']);
      // The signs and operators of the equation are those of the combined model: still one plus and one minus on the right.
      expect(tex.match(/\+/g), s).toHaveLength(1);
      expect(tex.match(/-/g), s).toHaveLength(1);
    }
  });

  it('keeps the baseline pieces of the equation: the operator and the left-hand side are those of the combined model with the symbol changed', () => {
    const base = equationPieces('combined');
    for (const s of NON_STEADY) {
      const timed = equationPieces('combined', s);
      expect(timed[0]).toEqual(base[0]);
      expect(timed[1]?.tex).toBe(base[1]?.tex.replace('D(r,T)', D_SCENARIO));
      expect(timed[3]).toEqual(base[3]);
    }
  });

  it('leaves the equation of a selection without the mechanism unchanged by the scenario function (no term is added)', () => {
    for (const p of PHYSICS_IDS.filter((x) => x !== 'combined')) for (const s of NON_STEADY) expect(equationPieces(p, s).map((x) => x.id), `${p}/${s}`).toEqual(equationPieces(p).map((x) => x.id));
    expect(equationTex('baseline', 'startup')).toBe(equationTex('baseline'));
  });

  it('marks the changed symbols (and only those) with the scenario highlight, and typesets every variant', () => {
    for (const s of NON_STEADY) {
      const changed = equationPieces('combined', s).filter((p) => p.scenario);
      expect(changed.map((p) => p.id), s).toEqual(['operator', 'generation']);
      for (const p of changed) expect(p.shown).toContain('geq-hi--scenario');
      for (const p of equationPieces('combined', s)) expect(() => renderTex(String.raw`\displaystyle ${p.shown}`, false), `${s}/${p.id}`).not.toThrow();
    }
    expect(equationPieces('combined', 'steady').some((p) => p.scenario)).toBe(false);
  });

  it('contains no control-rod or refuelling symbol in any equation string', () => {
    const strings: string[] = [];
    for (const p of PHYSICS_IDS) for (const s of SCENARIO_IDS) for (const piece of equationPieces(p, s)) strings.push(piece.tex, piece.shown);
    for (const s of SCENARIO_IDS) for (const p of SCENARIOS[s].params) strings.push(p.symbol, p.unit);
    expect(strings.length).toBeGreaterThan(50);
    for (const str of strings) expect(str).not.toMatch(/rod|refuel|\\rho|\\varrho|\\Sigma|\\sigma|\\phi|\\Phi|\\beta|reactivity|ρ|Σ/i);
    // The symbols of the whole set are the five of the directive and the three of the model, nothing else.
    const symbols = new Set(SCENARIO_IDS.flatMap((s) => SCENARIOS[s].params.map((p) => p.symbol)));
    expect([...symbols].sort()).toEqual(['D(r,T(t))', 'G', 'G(r,t)', String.raw`\lambda`, 'T', 'T(t)'].sort());
  });

  // Rewritten with the directive of 26 Nov 2026 (Section 5): the numerical treatment of a scenario is no longer a
  // field of the scenario but a note inside the items of the Numerical Interpretation (equation/numerics.ts).
  it('describes the diffusivity at the middle of the step, never as a step average', () => {
    const numerics = SCENARIO_IDS.flatMap((s) => ITEM_IDS.map((i) => scenarioNote(s, i) ?? '')).join(' ');
    expect(numerics).not.toMatch(/step-averaged diffusivit/i);
    expect(scenarioNote('shutdown', 11)).toMatch(/temperature of the middle of the step/);
    expect(scenarioNote('shutdown', 11)).toMatch(/reassembled again when the time step changes/);
    expect(SCENARIOS.shutdown.explanation.join(' ')).not.toMatch(/reduced or ends/);
    expect(SCENARIO_NAME.refuelling).toBe('fuel-removal');
  });

  it('shows the changed symbols on the page when a scenario is selected', async () => {
    const user = userEvent.setup();
    render(
      <>
        <GoverningEquation />
        <ScenarioPicker />
      </>,
    );
    await user.click(screen.getByTestId('scenario-startup'));
    const hi = Array.from(document.querySelectorAll('.geq-hi--scenario')).map((e) => e.textContent);
    expect(hi.some((t) => t?.includes('G(r,t)'))).toBe(true);
    expect(hi.some((t) => t?.includes('D(r,T(t))'))).toBe(true);
    expect(document.querySelectorAll('.geq-new')).toHaveLength(0);
    expect(screen.getByTestId('governing-equation').dataset.scenario).toBe('startup');
    // Back to steady-state operation: the equation is the combined model of the physics controls again.
    await user.click(screen.getByTestId('scenario-steady'));
    expect(document.querySelectorAll('.geq-hi--scenario')).toHaveLength(0);
    expect(screen.getByTestId('governing-equation').dataset.physics).toBe('combined');
  });
});

describe('the explanation of the selected scenario', () => {
  const sentences = (t: string) => t.match(/[^.]+\./g)?.length ?? 0;

  it('is two to four sentences for every scenario, with the parameters defined and their units', () => {
    for (const s of SCENARIO_IDS) {
      const n = SCENARIOS[s].explanation.reduce((a, t) => a + sentences(t), 0);
      expect(n, s).toBeGreaterThanOrEqual(2);
      expect(n, s).toBeLessThanOrEqual(4);
      for (const p of paramsFor(s, ['generation', 'decay', 'temperature'])) {
        expect(p.meaning.length).toBeGreaterThan(20);
        expect(p.unit).not.toBe('');
      }
    }
  });

  it('says what the directive requires of each scenario', () => {
    const text = (s: ScenarioId) => SCENARIOS[s].explanation.join(' ');
    // Steady-state operation: constant conditions, and explicitly not a stationary concentration field.
    expect(text('steady')).toMatch(/constant prescribed operating conditions/i);
    expect(text('steady')).toMatch(/does not impose that the time derivative of C is zero/);
    // The reason: the run starts from its initial state and reaches the balance only over times longer than the irradiation
    // (not "because the species is still being generated, diffusing and decaying": a stationary field is exactly such a balance).
    expect(text('steady')).toMatch(/Constant conditions do not make the field stationary: the run starts from its initial state \(a fresh particle holds no inventory\)/);
    expect(text('steady')).toMatch(/approaches a balance of generation, release and decay only over times longer than the irradiation, so C\(r,t\) keeps changing/);
    expect(text('steady')).not.toMatch(/still being generated/);
    expect(text('shutdown')).toMatch(/Radioactive decay continues after shutdown/);
    expect(text('shutdown')).toMatch(/temperature follows a prescribed cooldown/);
    expect(text('startup')).toMatch(/prescribed generation and temperature histories both rise from their starting values as power is raised/);
    expect(text('control-rod')).toMatch(/prescribed changes of the fission-rate history and of the temperature history/);
    expect(text('control-rod')).toMatch(/No control-rod term enters the equation/);
    // The initial states: shutdown and fuel removal start from the end of normal operation, startup from a fresh particle.
    expect(text('shutdown')).toMatch(/Starting from the state at the end of normal operation/);
    expect(text('startup')).toMatch(/Starting from a fresh particle/);
    // Refuelling: shown as fuel removal and storage; the rest is a future scenario.
    expect(text('refuelling')).toMatch(/^Fuel removal, starting from the state at the end of normal operation: the particle leaves the reactor and is stored/);
    expect(text('refuelling')).toMatch(/future scenario/);
    expect(text('refuelling')).toMatch(/Mechanical refuelling is not simulated/);
  });

  it('is shown with the selected scenario, with the prescribed-input label, and follows the selection', async () => {
    const user = userEvent.setup();
    render(<ScenarioPicker />);
    expect(screen.getByTestId('scenario-explanation').textContent).toBe(NO_SCENARIO_TEXT);
    for (const s of SCENARIO_IDS) {
      await user.click(screen.getByTestId(`scenario-${s}`));
      const box = screen.getByTestId('scenario-explanation');
      expect(box.dataset.scenario).toBe(s);
      expect(box.textContent).toContain(SCENARIO_LABEL[s]);
      expect(box.textContent).toContain(SCENARIO_KIND);
      expect(box.textContent).toContain(SCENARIOS[s].explanation[0]);
    }
  });

  it('defines the symbols of the selection: the scenario symbols for the combined model, only those of the mechanisms for steady-state operation', () => {
    expect(paramsFor('startup', mechanismsOf('combined')).map((p) => p.symbol)).toEqual(['G(r,t)', 'T(t)', 'D(r,T(t))', String.raw`\lambda`]);
    expect(paramsFor('steady', mechanismsOf('generation')).map((p) => p.symbol)).toEqual(['G']);
    expect(paramsFor('steady', mechanismsOf('baseline'))).toEqual([]);
  });

  it('keeps the page free of the forbidden claims, in every scenario', async () => {
    const user = userEvent.setup();
    render(<Explore compact={false} onOpenControls={() => undefined} />);
    for (const s of SCENARIO_IDS) {
      await user.click(screen.getByTestId(`scenario-${s}`));
      expect(claimsIn(screen.getByTestId('block-b').textContent ?? ''), s).toEqual([]);
      expect(claimsIn(document.body.textContent ?? ''), s).toEqual([]);
    }
    // The detector itself finds a forbidden claim (so an empty result is meaningful).
    expect(claimsIn('STRATO models neutron kinetics and this is a validated reactor prediction.')).toEqual(['models a reactor mechanism', 'validated reactor prediction']);
    expect(claimsIn('STRATO does not model neutron kinetics. It is not a validated reactor prediction.')).toEqual([]);
  });
});

describe('the rule that couples a scenario to the physics', () => {
  it('is stated once on the page in one sentence', () => {
    render(<ScenarioPicker />);
    expect(screen.getByTestId('scenario-rule').textContent).toBe(SCENARIO_RULE);
    expect(SCENARIO_RULE.match(/\./g)).toHaveLength(1);
  });

  it('selects the combined model with every scenario (steady-state operation too), and announces it', async () => {
    const user = userEvent.setup();
    render(
      <>
        <GoverningEquation />
        <ScenarioPicker />
      </>,
    );
    expect(useStrato.getState().selection).toMatchObject({ physics: 'baseline', scenario: null });
    await user.click(screen.getByTestId('scenario-shutdown'));
    expect(useStrato.getState().selection).toMatchObject({ physics: 'combined', scenario: 'shutdown' });
    expect(screen.getByTestId('physics-combined').getAttribute('aria-checked')).toBe('true');
    expect(screen.getByTestId('scenario-announce').textContent).toMatch(/Shutdown and cooldown selected\..*The combined model is selected, because this scenario requires it/);
    // Choosing another physics clears the scenario, also announced.
    await user.click(screen.getByTestId('physics-decay'));
    expect(useStrato.getState().selection).toMatchObject({ physics: 'decay', scenario: null });
    expect(screen.getByTestId('scenario-picker').querySelector('[aria-checked="true"]')).toBeNull();
    expect(screen.getByTestId('scenario-announce').textContent).toMatch(/No operating scenario selected, because the physics chosen is not the combined model/);
    // Steady-state operation follows the same rule: it selects the combined model.
    await user.click(screen.getByTestId('physics-generation'));
    await user.click(screen.getByTestId('scenario-steady'));
    expect(useStrato.getState().selection).toMatchObject({ physics: 'combined', scenario: 'steady' });
  });
});

describe('status and enabled state come from the capability object', () => {
  it('for the wrapper of this build (baseline only): all five scenarios are planned, steady-state operation too', () => {
    const cap = buildCapability(['diffusion_btcs_layered']);
    // The baseline run is not Scenario 1 (constant generation from a fresh particle): it never makes a scenario verified.
    expect(cap.scenarios).toEqual({ steady: 'planned', shutdown: 'planned', startup: 'planned', 'control-rod': 'planned', refuelling: 'planned' });
    for (const s of SCENARIO_IDS) expect(scenarioStatus(cap, s), s).toBe('planned');
    expect(selectionSupport(cap, 'baseline', null).supported).toBe(true);
    for (const s of SCENARIO_IDS) expect(selectionSupport(cap, 'combined', s).missing, s).toContain(`the ${SCENARIO_NAME[s]} scenario`);
  });

  it('shows these statuses on the buttons', () => {
    act(() => useStrato.getState().setCapabilities(['diffusion_btcs_layered']));
    render(<ScenarioPicker />);
    for (const s of SCENARIO_IDS) expect(screen.getByTestId(`scenario-status-${s}`).textContent, s).toBe('Planned');
  });

  it('turns steady-state operation on only when the wrapper reports its scenario with the mechanisms it needs', () => {
    const only = buildCapability([...REPORTED_MECHANISMS, WRAPPER_SCENARIO_ID.steady], withScenarios(VERIFIED_MECHANISMS, 'VERIFIED', ['steady']));
    expect(only.scenarios.steady).toBe('verified');
    expect(selectionSupport(only, 'combined', 'steady')).toEqual({ supported: true, missing: [] });
    // Without the generation mechanism the scenario is not simulable, and the page names what is missing.
    const noGeneration = buildCapability(['diffusion_btcs_layered', WRAPPER_SCENARIO_ID.steady], withScenarios(VERIFIED_MECHANISMS, 'VERIFIED', ['steady']));
    expect(selectionSupport(noGeneration, 'combined', 'steady').missing).toContain('fission-product generation');
  });

  it('follows a fake wrapper that reports scenarios: implemented, then verified, without an edit to the page logic', () => {
    // Reported by the wrapper but not recorded as verified: implemented, not simulable.
    const implemented = buildCapability([...REPORTED_MECHANISMS, WRAPPER_SCENARIO_ID.startup]);
    expect(implemented.scenarios.startup).toBe('implemented');
    expect(selectionSupport(implemented, 'combined', 'startup').supported).toBe(false);
    // Reported and recorded as verified, together with the three mechanisms: simulable; the others stay planned.
    const cat = withScenarios(VERIFIED_MECHANISMS, 'VERIFIED', ['startup', 'shutdown']);
    const verified = buildCapability([...REPORTED_MECHANISMS, WRAPPER_SCENARIO_ID.startup, WRAPPER_SCENARIO_ID.shutdown], cat);
    expect(verified.scenarios).toEqual({ steady: 'planned', shutdown: 'verified', startup: 'verified', 'control-rod': 'planned', refuelling: 'planned' });
    expect(selectionSupport(verified, 'combined', 'startup')).toEqual({ supported: true, missing: [] });
    expect(selectionSupport(verified, 'combined', 'control-rod').missing).toEqual(['the control-rod insertion scenario']);
    // A catalogue row alone, without the wrapper's report, never makes a scenario available.
    expect(buildCapability(REPORTED_MECHANISMS, cat).scenarios.startup).toBe('planned');
  });

  it('shows the statuses of a fake wrapper on the buttons and turns the simulation on for a verified scenario', () => {
    const cat = withScenarios(VERIFIED_MECHANISMS, 'VERIFIED', ['startup']);
    act(() => {
      useStrato.getState().setCapabilities([...REPORTED_MECHANISMS, WRAPPER_SCENARIO_ID.startup]);
      useStrato.getState().setSelection({ scenario: 'startup' });
    });
    render(withCatalogue(cat, <Explore compact={false} onOpenControls={() => undefined} />));
    expect(screen.getByTestId('scenario-status-startup').textContent).toBe('Verified');
    expect(screen.getByTestId('scenario-status-shutdown').textContent).toBe('Planned');
    expect(screen.getByTestId('block-d').dataset.supported).toBe('true');
    expect(screen.queryByTestId('unsupported-simulation')).toBeNull();
    // A scenario that the wrapper does not report is still explained and labelled, and shows no result.
    act(() => useStrato.getState().setSelection({ scenario: 'shutdown' }));
    expect(screen.getByTestId('block-d').dataset.supported).toBe('false');
    expect(screen.getByTestId('unsupported-simulation').textContent).toMatch(/the shutdown and cooldown scenario/);
    expect(screen.getByTestId('scenario-explanation').dataset.scenario).toBe('shutdown');
  });

  it('for a scenario the build cannot solve, the simulation and the results say so concisely and show nothing', () => {
    for (const s of SCENARIO_IDS) {
      act(() => {
        useStrato.getState().setCapabilities(['diffusion_btcs_layered']);
        useStrato.getState().setSelection({ scenario: s });
      });
      const { unmount } = render(<Explore compact={false} onOpenControls={() => undefined} />);
      const sim = screen.getByTestId('unsupported-simulation').textContent ?? '';
      expect(sim, s).toContain(`the ${SCENARIO_NAME[s]} scenario`);
      expect(sim, s).toMatch(/does not implement/);
      expect(screen.getByTestId('unsupported-results').textContent, s).toMatch(/no result is shown for this selection/);
      expect(screen.queryByTestId('particle'), s).toBeNull();
      expect(screen.queryByTestId('release-readout'), s).toBeNull();
      expect(screen.getByTestId('block-d').dataset.runStatus, s).toBe('none');
      unmount();
    }
    const msg = unsupportedMessage({ supported: false, missing: ['a', 'b', 'c'] });
    expect(msg).toContain('a, b and c');
  });

  it('runs the baseline, labelled as baseline diffusion with no scenario, and substitutes nothing for any scenario', () => {
    act(() => useStrato.getState().setCapabilities(['diffusion_btcs_layered']));
    const { unmount } = render(<Explore compact={false} onOpenControls={() => undefined} />);
    expect(screen.getByTestId('block-d').dataset.supported).toBe('true');
    const stage = screen.getByTestId('strip-stage').textContent ?? '';
    expect(stage).toContain('Physics: Baseline diffusion; scenario: none');
    expect(stage).not.toContain('Steady-state operation');
    unmount();
  });
});

describe('keyboard operation and the address', () => {
  it('is operated by the keyboard: one tab stop, arrows select and move focus, Home and End jump', async () => {
    const user = userEvent.setup();
    render(<ScenarioPicker />);
    screen.getByTestId('scenario-steady').focus();
    await user.keyboard('{ArrowRight}{ArrowRight}');
    expect(useStrato.getState().selection.scenario).toBe('startup');
    expect(document.activeElement).toBe(screen.getByTestId('scenario-startup'));
    await user.keyboard('{End}');
    expect(useStrato.getState().selection.scenario).toBe('refuelling');
    await user.keyboard('{ArrowRight}');
    expect(useStrato.getState().selection.scenario).toBe('steady');
    await user.keyboard('{ArrowLeft}');
    // (wraps round the five buttons)
    expect(useStrato.getState().selection.scenario).toBe('refuelling');
    await user.keyboard('{Home}');
    expect(useStrato.getState().selection.scenario).toBe('steady');
    expect(screen.getByTestId('scenario-refuelling').getAttribute('tabindex')).toBe('-1');
    // Space and Enter on a focused button select it too.
    screen.getByTestId('scenario-shutdown').focus();
    await user.keyboard('{Enter}');
    expect(useStrato.getState().selection.scenario).toBe('shutdown');
  });

  it('brings the checked button back into view after a selection made with the keyboard or the pointer', async () => {
    const user = userEvent.setup();
    const calls: (string | undefined)[] = [];
    const original = Element.prototype.scrollIntoView;
    Element.prototype.scrollIntoView = function (this: HTMLElement) {
      calls.push(this.dataset.testid);
    };
    try {
      render(<ScenarioPicker />);
      // Rendering alone (or an address change) does not scroll.
      expect(calls).toEqual([]);
      screen.getByTestId('scenario-steady').focus();
      await user.keyboard('{ArrowRight}');
      expect(calls).toContain('scenario-shutdown');
      await user.click(screen.getByTestId('scenario-startup'));
      expect(calls).toContain('scenario-startup');
    } finally {
      Element.prototype.scrollIntoView = original;
    }
  });

  it('writes the selection to the address and reads it back (round trip)', async () => {
    const user = userEvent.setup();
    window.location.hash = '#/explore';
    const stop = startAddressSync();
    render(<ScenarioPicker />);
    await user.click(screen.getByTestId('scenario-control-rod'));
    expect(window.location.hash).toBe('#/explore?physics=combined&scenario=control-rod');
    expect(window.location.hash).toBe(exploreHash(useStrato.getState().selection));
    // The address names the scenario: following it sets the selection.
    act(() => {
      useStrato.getState().setSelection({ scenario: null, physics: 'baseline' });
    });
    expect(window.location.hash).toBe('#/explore');
    act(() => {
      window.location.hash = '#/explore?scenario=refuelling';
      window.dispatchEvent(new HashChangeEvent('hashchange'));
    });
    expect(useStrato.getState().selection).toMatchObject({ physics: 'combined', scenario: 'refuelling' });
    expect(screen.getByTestId('scenario-refuelling').getAttribute('aria-checked')).toBe('true');
    stop();
    window.location.hash = '';
  });
});

describe('the style of the scenario row', () => {
  it('has a 44 px target and wraps at phone width (the physics buttons rule applies to it)', async () => {
    const { readFileSync } = await import('node:fs');
    const { join } = await import('node:path');
    const css = readFileSync(join(process.cwd(), 'src', 'design', 'app.css'), 'utf8');
    expect(css).toMatch(/\.physics__btn \{[^}]*min-height: var\(--touch-min\)/);
    expect(css).toMatch(/\.physics \{[^}]*flex-wrap: wrap/);
    // A scenario row is a .physics group, so it takes the same rules.
    render(<ScenarioPicker />);
    expect(screen.getByTestId('scenario-picker').classList.contains('physics')).toBe(true);
  });
});

// A type check that the exported lists are the ones the tests assume.
const _ids: readonly PhysicsId[] = PHYSICS_IDS;
void _ids;
