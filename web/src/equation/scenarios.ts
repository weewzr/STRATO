// The operating scenarios of the Explore page (owner directive of 26 Nov 2026, Sections 3 and 5; science
// design v3, Section 5). Display text only: nothing here is evaluated, and no number of a scenario history
// appears (the histories arrive with the scenario definition files in the later integration; this page
// describes the kind of history in words). Every scenario is a prescribed-input simulation: it changes the
// existing symbols G and D(r,T) (to G(r,t) and D(r,T(t))) and adds no term, and no control-rod or refuelling
// term exists anywhere. STRATO does not model neutron kinetics, control-rod motion, thermal hydraulics,
// power redistribution or mechanical refuelling. Which scenario the build can simulate comes from the
// capability object (equation/capability.ts), never from this file.
import { SCENARIO_IDS, type Mechanism, type ScenarioId } from './capability';

export const SCENARIO_LABEL: Record<ScenarioId, string> = {
  steady: 'Steady-state operation',
  shutdown: 'Shutdown and cooldown',
  startup: 'Startup',
  'control-rod': 'Control-rod insertion',
  refuelling: 'Refuelling',
};

/** The heading of the explanation of the selected scenario: the button label, and for refuelling what it is (fuel removal). */
export const SCENARIO_HEADING: Record<ScenarioId, string> = { ...SCENARIO_LABEL, refuelling: 'Refuelling (fuel removal)' };

/** What every scenario is, in a few words (shown with the selected scenario). */
export const SCENARIO_KIND = 'Prescribed-input simulation, not a validated reactor prediction.';

/**
 * The one rule that couples the two controls (never silent; stated on the page in this sentence): every scenario
 * is a prescribed history of the generation G and of the temperature (steady-state operation: constant ones, from
 * a fresh particle) with decay continuing, so it requires all three mechanisms. Choosing a scenario selects the
 * combined model; choosing another physics clears the scenario.
 */
export const SCENARIO_RULE =
  'Rule: every scenario shown here (a history from a fresh particle) needs generation, decay and temperature-dependent diffusion, so choosing one selects the combined model, and choosing another physics clears the scenario.';

/** Every scenario requires the combined model; no scenario at all requires nothing. */
export const requiresCombined = (scenario: ScenarioId | null): boolean => scenario !== null;

/** A symbol that the page defines with the scenario: its name, meaning (from Document 5, Sections 0 to 3) and unit. */
export interface ScenarioParam {
  id: string;
  /** TeX of the symbol. */
  symbol: string;
  name: string;
  meaning: string;
  /** Quantity unit (TeX). */
  unit: string;
  /** Shown only when this mechanism is part of the physics selected (steady-state operation); null: always shown. */
  mechanism: Mechanism | null;
}

const G_UNIT = String.raw`\mathrm{mol\ m^{-3}\ s^{-1}}`;
const D_UNIT = String.raw`\mathrm{m^2\ s^{-1}}`;

export interface ScenarioInfo {
  /** Two to four sentences: what the scenario is and which existing terms it changes. */
  explanation: string[];
  /** The symbols it changes or holds, with their definitions. */
  params: ScenarioParam[];
  /** The numerical treatment in the Numerical Interpretation (two sentences; the equations themselves are in the numerical steps). */
  numerics: string[];
}

const G_TIMED: ScenarioParam = {
  id: 'G',
  symbol: 'G(r,t)',
  name: 'Fission-product generation, prescribed in time',
  meaning: 'The generation rate density of the species; in the particle a source in the kernel only (zero in the coatings), with its history in time prescribed by the scenario.',
  unit: G_UNIT,
  mechanism: 'generation',
};
const T_TIMED: ScenarioParam = {
  id: 'T',
  symbol: 'T(t)',
  name: 'Particle temperature, prescribed in time',
  meaning: 'The temperature of the particle, taken uniform in the particle at each time, with its history prescribed by the scenario.',
  unit: String.raw`\mathrm{K}`,
  mechanism: 'temperature',
};
const D_TIMED: ScenarioParam = {
  id: 'D',
  symbol: 'D(r,T(t))',
  name: 'Diffusivity at the prescribed temperature',
  meaning: 'The diffusivity of each material evaluated at T(t): it depends on time only through the temperature, and stays inside the diffusion operator.',
  unit: D_UNIT,
  mechanism: 'temperature',
};
const LAMBDA: ScenarioParam = {
  id: 'lambda',
  symbol: String.raw`\lambda`,
  name: 'Decay constant',
  meaning: 'The decay constant of the nuclide, the same in every material; no scenario changes it, and decay continues in every scenario.',
  unit: String.raw`\mathrm{s^{-1}}`,
  mechanism: 'decay',
};

export const SCENARIOS: Record<ScenarioId, ScenarioInfo> = {
  steady: {
    explanation: [
      'Constant prescribed operating conditions: the generation G and the particle temperature T are held constant in time.',
      'This does not impose that the time derivative of C is zero.',
      'Constant conditions do not make the field stationary: the run starts from its initial state (a fresh particle holds no inventory), and the concentration approaches a balance of generation, release and decay only over times longer than the irradiation, so C(r,t) keeps changing.',
    ],
    params: [
      { ...G_TIMED, id: 'G', symbol: 'G', name: 'Fission-product generation, constant in time', meaning: 'The generation rate density in the kernel, held at one prescribed value.' },
      { ...T_TIMED, id: 'T', symbol: 'T', name: 'Particle temperature, constant in time', meaning: 'The particle temperature, held at one prescribed value.' },
    ],
    numerics: [
      'With constant inputs and one uniform time step, the diffusivities and the step do not change, so the matrix is assembled and factorised once; only the right-hand side changes from step to step, because it holds the concentration of the previous step.',
      'A stationary field would need the concentration to repeat from step to step; constant inputs do not impose that, and the run does not assume it.',
    ],
  },
  shutdown: {
    explanation: [
      'Starting from the state at the end of normal operation, generation ends at shutdown, and the temperature follows a prescribed cooldown.',
      'Radioactive decay continues after shutdown, so the inventory keeps falling, while diffusion slows as the particle cools.',
      'G becomes G(r,t), falling from its operating value, and D(r,T) becomes D(r,T(t)); the equation gains no term.',
    ],
    params: [G_TIMED, T_TIMED, D_TIMED, LAMBDA],
    numerics: [
      'From the shutdown time the step-averaged generation is zero, so the right-hand side of each step is the concentration of the previous step; the decay contribution stays on the diagonal of the matrix.',
      'The diffusivity evaluated at the temperature of the middle of the step (F36) changes with every cooldown step, so the matrix is reassembled at each of those steps, held once the temperature is held, and reassembled again when the time step changes.',
    ],
  },
  startup: {
    explanation: [
      'Starting from a fresh particle, the prescribed generation and temperature histories both rise from their starting values as power is raised.',
      'The rising temperature raises the diffusivities, while the generation feeds the kernel.',
      'G becomes G(r,t) and D(r,T) becomes D(r,T(t)); the equation gains no term.',
    ],
    params: [G_TIMED, T_TIMED, D_TIMED, LAMBDA],
    numerics: [
      'While the power rises, the step average of the generation changes the right-hand side at every step, and the diffusivity evaluated at the temperature of the middle of the step changes the matrix at every step.',
      'Both stay constant once the prescribed rise has ended.',
    ],
  },
  'control-rod': {
    explanation: [
      'Control-rod insertion is represented only by its consequence for the particle: prescribed changes of the fission-rate history and of the temperature history.',
      'No control-rod term enters the equation, and STRATO does not model rod motion, neutron kinetics or the core.',
      'G becomes G(r,t) and D(r,T) becomes D(r,T(t)), both driven by the prescribed histories.',
    ],
    params: [
      { ...G_TIMED, name: 'Fission-product generation, from the prescribed fission-rate history', meaning: 'The generation rate density in the kernel; it follows the prescribed fission-rate history and is zero in the coatings.' },
      T_TIMED,
      D_TIMED,
      LAMBDA,
    ],
    numerics: [
      'The prescribed change enters only through the step-averaged generation on the right-hand side and the diffusivity evaluated at the temperature of the middle of the step in the matrix; no new term is assembled.',
      'Both change during the transition and are held afterwards.',
    ],
  },
  refuelling: {
    explanation: [
      'Fuel removal, starting from the state at the end of normal operation: the particle leaves the reactor and is stored, so generation ends and the temperature falls to a storage value, while diffusion and decay continue.',
      'Mechanical refuelling is not simulated, and reinsertion or irradiation in several passes is a future scenario, because no registered input history supports it.',
      'G becomes G(r,t) and D(r,T) becomes D(r,T(t)), as for shutdown; the equation gains no term.',
    ],
    params: [G_TIMED, T_TIMED, D_TIMED, LAMBDA],
    numerics: [
      'After removal the step-averaged generation is zero, as in shutdown, so the right-hand side of each step is the concentration of the previous step.',
      'The matrix follows the storage temperature history and keeps the decay contribution on its diagonal.',
    ],
  },
};

/** The parameters to define for a selection: those of the scenario that belong to a mechanism of the physics selected (all of them for the combined model). */
export function paramsFor(scenario: ScenarioId, active: readonly Mechanism[]): ScenarioParam[] {
  return SCENARIOS[scenario].params.filter((p) => p.mechanism === null || active.includes(p.mechanism));
}

/** What the page announces when the scenario changes (the live-region sentence). */
export function scenarioChange(scenario: ScenarioId, physicsChanged: boolean): string {  const tail = physicsChanged ? ' The combined model is selected, because this scenario requires it.' : '';
  return `${SCENARIO_LABEL[scenario]} selected. ${SCENARIO_KIND}${tail}`;
}

/** What the page announces when choosing a physics cleared the scenario. */
export const SCENARIO_RESET = 'No operating scenario selected, because the physics chosen is not the combined model.';

/** Shown in block B while no scenario is selected. */
export const NO_SCENARIO_TEXT =
  'No operating scenario is selected: the physics chosen above is taken with constant inputs, and the simulation block says whether this build runs it; it is labelled by that physics, not by a scenario. Choose a scenario to see its prescribed histories.';

export { SCENARIO_IDS };
