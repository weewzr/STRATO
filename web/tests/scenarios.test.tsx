// The operating-scenario row of the Explore page (owner directive of 26 Nov 2026, Sections 3, 5 and 6;
// decision 70(b); science design v3, Section 5): the five buttons, what each scenario changes in the
// equation (existing symbols only, never a term), the explanations, the coupling rule, the forbidden
// claims, the status and enabled state from the capability object (also for a fake object with scenarios
// implemented), keyboard operation and the address.
import { describe, expect, it } from 'vitest';
import {
  buildCapability,
  mechanismsOf,
  PHYSICS_IDS,
  SCENARIO_IDS,
  scenarioStatus,
  SCENARIO_NAME,
  selectionSupport,
  WRAPPER_SCENARIO_ID,
  type PhysicsId,
  type ScenarioId,
} from '../src/equation/capability';
import { ITEM_IDS, scenarioNote } from '../src/equation/numerics';
import { equationPieces, equationTex, D_SCENARIO, G_SCENARIO } from '../src/equation/physics';
import { renderTex } from '../src/equation/Tex';
import { paramsFor, SCENARIOS } from '../src/equation/scenarios';
import { type Catalogue } from '../src/research/catalogue';
import { fixtureCatalogue } from './catalogueFixture';

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

  it('says on the page, in two sentences, what the directive requires of each scenario', () => {
    const page = (s: ScenarioId) => SCENARIOS[s].explanation.join(' ');
    expect(page('steady')).toMatch(/does not impose that the time derivative of C is zero/);
    expect(page('steady')).toMatch(/constant conditions do not make the field stationary/);
    expect(page('shutdown')).toMatch(/radioactive decay continues/);
    expect(page('shutdown')).toMatch(/generation ends at shutdown and the temperature follows a prescribed cooldown/);
    expect(page('startup')).toMatch(/both rise from their starting values as power is raised/);
    expect(page('control-rod')).toMatch(/no control-rod term enters the equation/);
    expect(page('refuelling')).toMatch(/Mechanical refuelling is not simulated/);
    expect(page('refuelling')).toMatch(/future scenario/);
    for (const s of SCENARIO_IDS) expect(page(s).match(/[^.]+\./g)?.length, s).toBe(2);
  });

  // The full text of each scenario, as verified, is in Scientific Documentation (documentation/explore.tsx, tested in
  // tests/explore_documentation.test.tsx); its wording is asserted here on the data that module renders.
  it('keeps the full wording of each scenario, with what the directive requires of it', () => {
    const text = (s: ScenarioId) => SCENARIOS[s].full.join(' ');
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

  it('defines the symbols of the selection: the scenario symbols for the combined model, only those of the mechanisms for steady-state operation', () => {
    expect(paramsFor('startup', mechanismsOf('combined')).map((p) => p.symbol)).toEqual(['G(r,t)', 'T(t)', 'D(r,T(t))', String.raw`\lambda`]);
    expect(paramsFor('steady', mechanismsOf('generation')).map((p) => p.symbol)).toEqual(['G']);
    expect(paramsFor('steady', mechanismsOf('baseline'))).toEqual([]);
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
});

// A type check that the exported lists are the ones the tests assume.
const _ids: readonly PhysicsId[] = PHYSICS_IDS;
void _ids;
