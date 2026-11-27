// The governing equation of the Explore page and its progression (owner directive of 26 Nov 2026,
// Section 2). Display only: nothing here is evaluated. The equations are those of the directive; the
// typesetting is the directive's own (r^2); the repository documents write the same power as (r)^2 and
// nothing else differs. The combined equation is also
// equation (1) of Document 5 (docs/mathematics/btcs/05_generation_decay_temperature.md).
import { MECHANISMS, type Mechanism, type PhysicsId, type ScenarioId } from './capability';

export const D_CONSTANT = 'D';
export const D_TEMPERATURE = 'D(r,T)';
/** The symbols an operating scenario changes (directive Section 3): G becomes G(r,t), D(r,T) becomes D(r,T(t)). A scenario adds no term. */
export const G_SCENARIO = 'G(r,t)';
export const D_SCENARIO = 'D(r,T(t))';

const LHS = String.raw`\frac{\partial C}{\partial t} =`;

/** The diffusion operator with the coefficient `d` inside it (the temperature law enters here, never as a term). */
export function operatorTex(d: string): string {
  // The thin space before the gradient is spacing only (Document 5 writes it too); it changes no mathematics.
  return String.raw`\frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 ${d}\,\frac{\partial C}{\partial r}\right)`;
}

export type PieceId = 'lhs' | 'operator' | 'generation' | 'decay';

export interface Piece {
  id: PieceId;
  /** Clean TeX of the piece; the full equation is the pieces joined by one space. */
  tex: string;
  /** The mechanism that this piece or its coefficient introduces; null for the baseline pieces. */
  mechanism: Mechanism | null;
  /** The TeX shown on the page: the same as `tex`, with the introduced coefficient marked for the highlight. */
  shown: string;
  /** True when the operating scenario changed a symbol of this piece (G or D); the changed symbol is marked in `shown`. */
  scenario?: boolean;
}

const mark = (cls: string, tex: string) => String.raw`\htmlClass{${cls}}{${tex}}`;

/** True when the scenario changes G and D(r,T) in time; no scenario and steady-state operation hold them constant. */
export const isTimed = (scenario: ScenarioId | null): boolean => scenario !== null && scenario !== 'steady';

/**
 * The pieces of the equation for a physics selection and an operating scenario, left to right; the baseline
 * pieces come first and are always present. Steady-state operation (constant prescribed conditions) changes
 * nothing: G and T are constants. Every other scenario changes existing symbols only, G to G(r,t) and D(r,T)
 * to D(r,T(t)), and only where the mechanism is part of the selection; it never adds a term.
 */
export function equationPieces(physics: PhysicsId, scenario: ScenarioId | null = null): Piece[] {
  const on = (m: Mechanism) => physics === m || physics === 'combined' || physics === 'future';
  const timed = isTimed(scenario);
  const dSymbol = !on('temperature') ? D_CONSTANT : timed ? D_SCENARIO : D_TEMPERATURE;
  const dShown = !on('temperature') ? D_CONSTANT : mark(timed ? 'geq-hi geq-hi--scenario' : 'geq-hi geq-hi--temperature', dSymbol);
  const gSymbol = timed ? G_SCENARIO : 'G';
  const pieces: Piece[] = [
    { id: 'lhs', tex: LHS, mechanism: null, shown: LHS },
    {
      id: 'operator',
      tex: operatorTex(dSymbol),
      mechanism: on('temperature') ? 'temperature' : null,
      shown: operatorTex(dShown),
      ...(timed && on('temperature') ? { scenario: true } : {}),
    },
  ];
  if (on('generation'))
    pieces.push({
      id: 'generation',
      tex: `+${gSymbol}`,
      mechanism: 'generation',
      shown: timed ? `+${mark('geq-hi geq-hi--scenario', gSymbol)}` : '+G',
      ...(timed ? { scenario: true } : {}),
    });
  if (on('decay')) pieces.push({ id: 'decay', tex: String.raw`-\lambda C`, mechanism: 'decay', shown: String.raw`-\lambda C` });
  return pieces;
}

/** The whole equation as one TeX string (structural tests and the accessible name). */
export function equationTex(physics: PhysicsId, scenario: ScenarioId | null = null): string {
  return equationPieces(physics, scenario)
    .map((p) => p.tex)
    .join(' ');
}

/** Heading of each choice; the full physical names of the directive. */
export const PHYSICS_LABEL: Record<PhysicsId, string> = {
  baseline: 'Baseline diffusion',
  generation: 'Fission-product generation',
  decay: 'Radioactive decay',
  temperature: 'Temperature-dependent diffusion',
  combined: 'Combined model',
  future: 'Future species-specific physics',
};

/** What the page announces when the selection changes (the live-region sentence). */
export const PHYSICS_CHANGE: Record<PhysicsId, string> = {
  baseline: 'Baseline diffusion: the spherical diffusion equation with a constant diffusivity in each layer.',
  generation: 'Fission-product generation: the source term plus G enters the right-hand side of the baseline equation.',
  decay: 'Radioactive decay: the term minus lambda C enters the right-hand side of the baseline equation.',
  temperature: 'Temperature-dependent diffusion: the diffusivity D becomes D of r and T inside the diffusion operator; no term is added.',
  combined: 'Combined model: the diffusivity is D of r and T inside the operator, and the terms plus G and minus lambda C are added on the right.',
  future: 'Future species-specific physics: the combined equation, unchanged.',
};

/** The same sentences under an operating scenario that changes the symbols in time: D of r and T of t, and G of r and t. */
export const PHYSICS_CHANGE_SCENARIO: Partial<Record<PhysicsId, string>> = {
  temperature: 'Temperature-dependent diffusion: under the scenario the diffusivity is D of r and T of t inside the diffusion operator; no term is added.',
  combined:
    'Combined model: under the scenario the diffusivity is D of r and T of t inside the operator, the source is G of r and t, and the term minus lambda C is added on the right.',
};

export interface TermMeaning {
  id: string;
  /** TeX of the symbol. */
  symbol: string;
  name: string;
  /** One or two sentences, from the repository documents (Document 1, Section 1; Document 5, Sections 0 to 3). */
  meaning: string;
  /** Quantity with its unit (TeX unit). */
  unit: string;
  /** The mechanism that introduces the term; null for the baseline. */
  mechanism: Mechanism | null;
}

export const TERM_MEANINGS: readonly TermMeaning[] = [
  {
    id: 'rate',
    symbol: String.raw`\frac{\partial C}{\partial t}`,
    name: 'Rate of change of the concentration',
    meaning: 'How fast the concentration C of the diffusing species at radius r changes with time t.',
    unit: String.raw`\mathrm{mol\ m^{-3}\ s^{-1}}`,
    mechanism: null,
  },
  {
    id: 'diffusion',
    symbol: operatorTex(D_CONSTANT),
    name: 'Spherical diffusion',
    meaning:
      'Net transport of the species by diffusion in spherical geometry: Fick\u2019s first law gives the flux, and the divergence of the flux in a thin shell changes the concentration. D is the diffusivity of the material, uniform in space inside each of the five layers (with temperature dependence it also depends on T).',
    unit: String.raw`D:\ \mathrm{m^2\ s^{-1}}`,
    mechanism: null,
  },
  {
    id: 'temperature',
    symbol: 'D(r,T)',
    name: 'Temperature-dependent diffusivity',
    meaning:
      'The diffusivity of each material depends on the temperature T. It stays inside the operator because D multiplies the concentration gradient in the flux; it changes a coefficient and adds no source or loss term.',
    unit: String.raw`D:\ \mathrm{m^2\ s^{-1}},\quad T:\ \mathrm{K}`,
    mechanism: 'temperature',
  },
  {
    id: 'generation',
    symbol: 'G',
    name: 'Fission-product generation',
    meaning:
      'The rate at which the species is born by fission, per unit volume; in the particle it is a source in the kernel only. Its value is a reference derived from published AGR-1 burnup (calculated), or a user input; it is not a measured quantity.',
    unit: String.raw`\mathrm{mol\ m^{-3}\ s^{-1}}`,
    mechanism: 'generation',
  },
  {
    id: 'decay',
    symbol: String.raw`-\lambda C`,
    name: 'Radioactive decay',
    meaning: 'Loss of the species by radioactive decay, in proportion to its concentration, with the decay constant \u03bb of the nuclide (the same in every material).',
    unit: String.raw`\lambda:\ \mathrm{s^{-1}}`,
    mechanism: 'decay',
  },
];

/** The meanings that apply to a selection: the two baseline terms, then each selected mechanism in the order of the equation. */
export function meaningsFor(physics: PhysicsId): TermMeaning[] {
  const sel = new Set<Mechanism>(physics === 'baseline' ? [] : physics === 'combined' || physics === 'future' ? MECHANISMS : [physics]);
  return TERM_MEANINGS.filter((t) => t.mechanism === null || sel.has(t.mechanism));
}
