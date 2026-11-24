// Term and stage table of the staged equation builder (prompt/TECHNICAL_ARCHITECTURE.md Section 4,
// prompt/DESIGN_SPECIFICATION.md Section 3). Display only: nothing here is evaluated. No research
// status is written here; the builder reads every status from the generated catalogue.
//
// Every LaTeX string that stands for an equation of the repository carries its source: the document,
// the equation tag (or the section, for an untagged line), the text copied verbatim from that document
// and the notation changes that turn the copy into what the page shows. web/tests/terms.test.ts reads
// the documents and checks every copy and every change. Notation changes used:
//   - the species index k of Part 2.1 is dropped for one species, as the Phase 2 derivations do
//     themselves (Section 3.0: "index dropped for one species");
//   - the layer index l of the v1.0 derivation is written i, as in the Phase 2 derivations;
//   - in (2.6) the piecewise-constant D is written D_i, its value in layer i (Section 2.6);
//   - in (P22.11) D(r,T(t)) is written D_i(T(t)), its value in layer i (definition in (P22.11)).
import type { CodeRef, DocRef } from './links';

export type Stage = 1 | 2 | 3 | 4 | 5 | 6;
export const STAGES: readonly Stage[] = [1, 2, 3, 4, 5, 6];

export type SwitchId = 'source' | 'decay' | 'arrhenius';
export interface SwitchSet {
  source: boolean;
  decay: boolean;
  arrhenius: boolean;
}

/** Mechanism switch to catalogue mechanism id (data/research/mechanisms.csv). */
export const SWITCH_MECHANISM: Record<SwitchId, string> = {
  source: 'generation',
  decay: 'decay',
  arrhenius: 'diffusivity_temperature',
};

export interface EqSource {
  doc: 'v1' | 'p2' | 'pseudo';
  /** Equation tag as printed, for example 'P21.58'; null for an untagged line, found in `section`. */
  tag: string | null;
  section: string;
  /** Copied character for character from the document. */
  verbatim: string;
  /** Text replacements, applied in order to `verbatim`, that give the displayed LaTeX. */
  notation?: readonly (readonly [string, string])[];
}

export interface UnitRow {
  symbol: string; // LaTeX
  meaning: string;
  unit: string; // LaTeX
}

export type TermId = 'lhs' | 'diffusion' | 'source' | 'decay' | 'diffusion_T' | 'arrhenius_layers' | 'partition';

export interface TermDef {
  id: TermId;
  name: string;
  mechanism: string;
  stage: Stage;
  /** Displayed LaTeX; empty for a term that is listed but not in the model. */
  latex: string;
  sources: EqSource[];
  switchId?: SwitchId;
  /** Plain-language meaning first, then the symbols. Written without numerals (block A shows it). */
  meaning: string;
  /** A defining relation shown under the meaning, with its source. */
  formula?: { latex: string; source: EqSource };
  units: UnitRow[];
  assumptions: string[];
  derivation: DocRef;
}

const NO_SPECIES = [
  ['c_k', 'c'],
  ['D_{i,k}', 'D_i'],
  ['\\lambda_k', '\\lambda'],
  ['S_{i,k}', 'S'],
] as const;

export const TERMS: Record<TermId, TermDef> = {
  lhs: {
    id: 'lhs',
    name: 'Rate of change of concentration',
    mechanism: 'diffusion_layered',
    stage: 1,
    latex: String.raw`\frac{\partial c}{\partial t}`,
    sources: [
      { doc: 'v1', tag: '2.6', section: '2.5 Spherical diffusion equation', verbatim: String.raw`\frac{\partial c}{\partial t}` },
      { doc: 'p2', tag: 'P21.1', section: '3.1 (a) Extended transport equation', verbatim: String.raw`\frac{\partial c_k}{\partial t}`, notation: NO_SPECIES },
    ],
    meaning:
      'How fast the amount of the fission product per unit volume changes at one radius. Every term on the right-hand side is a rate in the same unit, so the equation is a balance: what changes equals what flows in, minus what decays, plus what is born.',
    units: [
      { symbol: 'c', meaning: 'concentration of the fission product', unit: String.raw`\mathrm{mol\ m^{-3}}` },
      { symbol: 't', meaning: 'time', unit: String.raw`\mathrm{s}` },
    ],
    assumptions: ['Spherical symmetry: the concentration depends on the radius and time only (v1.0 derivation, Section 1.6).'],
    derivation: { doc: 'v1', section: '2.6 Layer equation of v1.0.0' },
  },
  diffusion: {
    id: 'diffusion',
    name: 'Diffusion through the five layers',
    mechanism: 'diffusion_layered',
    stage: 1,
    latex: String.raw`\frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 D_i\frac{\partial c}{\partial r}\right)`,
    sources: [
      { doc: 'v1', tag: '2.6', section: '2.5 Spherical diffusion equation', verbatim: String.raw`\frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 D\,\frac{\partial c}{\partial r}\right)`, notation: [['D\\,', 'D_i']] },
      {
        doc: 'p2',
        tag: 'P21.1',
        section: '3.1 (a) Extended transport equation',
        verbatim: String.raw`\frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 D_{i,k}\frac{\partial c_k}{\partial r}\right)`,
        notation: NO_SPECIES,
      },
    ],
    meaning:
      'The net inflow by diffusion into a thin spherical shell, per unit volume (Fick’s law). The factor r² follows the area of the shell, which grows with the radius. Each of the five layers (kernel, buffer, inner pyrocarbon, silicon carbide, outer pyrocarbon) has its own constant diffusivity Dᵢ, so the layering is already part of this first stage.',
    units: [
      { symbol: 'r', meaning: 'radius from the centre of the particle', unit: String.raw`\mathrm{m}` },
      { symbol: 'D_i', meaning: 'diffusivity in layer i, constant in time and within the layer', unit: String.raw`\mathrm{m^2\ s^{-1}}` },
      { symbol: 'R_i', meaning: 'outer radius of layer i', unit: String.raw`\mathrm{m}` },
    ],
    assumptions: [
      'Fickian diffusion with a constant diffusivity in each layer (v1.0 derivation, Section 2.6).',
      'Perfect contact between layers: concentration and flux are continuous, with no partition coefficient (Section 3.2).',
      'The temperature is read from the input and enters no formula in v1.0.',
    ],
    derivation: { doc: 'v1', section: '2.6 Layer equation of v1.0.0' },
  },
  source: {
    id: 'source',
    name: 'Fission-product generation',
    mechanism: 'generation',
    stage: 2,
    switchId: 'source',
    latex: String.raw`+ S(r,t)`,
    sources: [{ doc: 'p2', tag: 'P21.1', section: '3.1 (a) Extended transport equation', verbatim: String.raw`+ S_{i,k}(r,t)`, notation: NO_SPECIES }],
    meaning:
      'Atoms of the fission product born by fission, per unit volume and time. In the kernel the source is the fission rate density times the yield, converted from atoms to moles; outside the kernel it is zero.',
    formula: {
      latex: String.raw`S(r,t) = \frac{\dot F(r,t)\,y}{N_A}\,\chi_1(r)`,
      source: {
        doc: 'p2',
        tag: 'P21.8',
        section: '3.3.1 Direct fission products',
        verbatim: String.raw`S_k(r,t) = \frac{\dot F(r,t)\,y_k}{N_A}\,\chi_1(r)`,
        notation: [
          ['S_k', 'S'],
          ['y_k', 'y'],
        ],
      },
    },
    units: [
      { symbol: 'S', meaning: 'volumetric source', unit: String.raw`\mathrm{mol\ m^{-3}\ s^{-1}}` },
      { symbol: String.raw`\dot F`, meaning: 'fission rate density, the only source input', unit: String.raw`\mathrm{fissions\ m^{-3}\ s^{-1}}` },
      { symbol: 'y', meaning: 'cumulative fission yield', unit: String.raw`\text{atoms per fission}` },
      { symbol: String.raw`\chi_1`, meaning: 'indicator of the kernel: one inside it, zero outside', unit: String.raw`\text{none}` },
      { symbol: 'N_A', meaning: 'Avogadro constant', unit: String.raw`\mathrm{mol^{-1}}` },
    ],
    assumptions: [
      'The source is confined to the kernel and uniform there; fission-fragment recoil into the buffer is neglected (Phase 2 derivations, Section 3.0, A5).',
      'The fission rate density is the only source input (owner decision D10); cumulative yields only (D2).',
      'Kernel-only source by default, with a configurable source region (owner decision D9); exact step average of the source in time (D11).',
      'The fission rate density is not yet in the parameter register (register gap GAP-28).',
    ],
    derivation: { doc: 'p2', section: '3.3.1 Direct fission products' },
  },
  decay: {
    id: 'decay',
    name: 'Radioactive decay',
    mechanism: 'decay',
    stage: 3,
    switchId: 'decay',
    latex: String.raw`- \lambda c`,
    sources: [{ doc: 'p2', tag: 'P21.1', section: '3.1 (a) Extended transport equation', verbatim: String.raw`- \lambda_k c_k`, notation: NO_SPECIES }],
    meaning:
      'Loss by radioactive decay: each atom decays with the same constant probability per unit time, wherever it is. The decay constant follows from the half-life. A stable isotope has no decay term.',
    formula: {
      latex: String.raw`\lambda = \frac{\ln 2}{t_{1/2}}`,
      source: { doc: 'p2', tag: 'P21.7', section: '3.2 (b) Radioactive decay law', verbatim: String.raw`\lambda = \frac{\ln 2}{t_{1/2}}` },
    },
    units: [
      { symbol: String.raw`\lambda`, meaning: 'decay constant', unit: String.raw`\mathrm{s^{-1}}` },
      { symbol: String.raw`t_{1/2}`, meaning: 'half-life, from data/nuclear/decay.csv', unit: String.raw`\mathrm{s}` },
    ],
    assumptions: [
      'First-order decay with the same decay constant in every layer: a nuclear property, independent of the host material (Phase 2 derivations, Section 3.0, A4).',
      'Released atoms never return; outside the particle they decay with the same constant (A6).',
      'Time integration: unsplit Backward Euler with implicit decay (owner decision D6).',
    ],
    derivation: { doc: 'p2', section: '3.2 (b) Radioactive decay law' },
  },
  diffusion_T: {
    id: 'diffusion_T',
    name: 'Temperature-dependent diffusivity',
    mechanism: 'diffusivity_temperature',
    stage: 4,
    switchId: 'arrhenius',
    latex: String.raw`\frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 D_i(T(t))\frac{\partial c}{\partial r}\right)`,
    sources: [
      {
        doc: 'p2',
        tag: 'P22.11',
        section: '5.2.1 Continuous problem',
        verbatim: String.raw`\frac{1}{r^2}\frac{\partial}{\partial r}\!\left(r^2 D(r,T(t))\frac{\partial c}{\partial r}\right)`,
        notation: [
          ['\\!', ''],
          ['D(r,T(t))', 'D_i(T(t))'],
        ],
      },
    ],
    meaning:
      'The diffusivity of each layer now follows the particle temperature, which changes with time during a heating test. Only the diffusivity changes; the form of the diffusion term and the layer conditions stay as in stage one.',
    units: [
      { symbol: 'T(t)', meaning: 'particle temperature, uniform across the particle', unit: String.raw`\mathrm{K}` },
      { symbol: 'D_i(T)', meaning: 'diffusivity of layer i at temperature T', unit: String.raw`\mathrm{m^2\ s^{-1}}` },
    ],
    assumptions: [
      'The temperature is uniform across the particle at each instant, which holds approximately in post-irradiation heating tests (Phase 2 derivations, Section 5.0, A1).',
      'The diffusivity depends on time only through the temperature (Section 5.0, A5).',
      'Conductances use the step-average diffusivity, option A, with option E selectable (owner decision D8).',
    ],
    derivation: { doc: 'p2', section: '5.2.1 Continuous problem' },
  },
  arrhenius_layers: {
    id: 'arrhenius_layers',
    name: 'Layer-specific temperature laws',
    mechanism: 'diffusivity_layer_laws',
    stage: 5,
    switchId: 'arrhenius',
    latex: String.raw`D_i(T) = \sum_{m=1}^{m_i} D_{0,i,m}\,\exp\!\left(-\frac{Q_{i,m}}{R_g T}\right), \qquad m_i \in \{1, 2\}`,
    sources: [
      {
        doc: 'p2',
        tag: 'P22.1',
        section: '5.1.1 Statement',
        verbatim: String.raw`D_i(T) = \sum_{m=1}^{m_i} D_{0,i,m}\,\exp\!\left(-\frac{Q_{i,m}}{R_g T}\right), \qquad m_i \in \{1, 2\}`,
      },
    ],
    meaning:
      'Each layer follows its own Arrhenius law, a sum of one or two branches with their own pre-exponential factor and activation energy. Because the layers respond differently to temperature, the ratios between their diffusivities change during a heating test. The layering itself is already in stage one; what is new here is a separate temperature law for each layer.',
    units: [
      { symbol: 'D_{0,i,m}', meaning: 'pre-exponential factor of branch m in layer i', unit: String.raw`\mathrm{m^2\ s^{-1}}` },
      { symbol: 'Q_{i,m}', meaning: 'activation energy of branch m in layer i', unit: String.raw`\mathrm{J\ mol^{-1}}` },
      { symbol: 'R_g', meaning: 'molar gas constant', unit: String.raw`\mathrm{J\ mol^{-1}\ K^{-1}}` },
      { symbol: 'T', meaning: 'temperature', unit: String.raw`\mathrm{K}` },
    ],
    assumptions: [
      'Diffusivity rows: the TECDOC-978 FRG two-branch rows (owner decision D4); AGR-1 correction factors are never defaults.',
      'Gas constant: the exact SI value by default, with the TECDOC-978 value selectable (owner decision D4).',
      'The registered kernel rows are for UO₂; an AGR-1 UCO run that uses them carries a label naming this assumption (rule E3).',
      'Interface partition coefficients are not in the model: concentration stays continuous at every interface.',
    ],
    derivation: { doc: 'p2', section: '5.4 (d) General multilayer case: layer-specific activation energies' },
  },
  partition: {
    id: 'partition',
    name: 'Interface partition coefficients',
    mechanism: 'interface_partition',
    stage: 5,
    latex: '',
    sources: [],
    meaning:
      'A partition coefficient would let the concentration jump at an interface between two layers. STRATO assumes perfect contact with no partition coefficient, so this term is listed for completeness and not inserted into any equation.',
    units: [],
    assumptions: ['Not in the model (v1.0 derivation, Section 3.2; Phase 2 derivations, Section 5.0, A2).'],
    derivation: { doc: 'v1', section: '3.2 Interfaces' },
  },
};

/**
 * The mechanism switches of block C, in the order of the builder stages (generation, decay,
 * temperature; rule E8). Layer-specific laws come with the temperature switch.
 */
export const SWITCHES: { id: SwitchId; term: TermId; stage: Stage; latex: string; line: 'dashed' | 'solid' | 'dotted' }[] = [
  { id: 'source', term: 'source', stage: 2, latex: String.raw`+ S(r,t)`, line: 'dashed' },
  { id: 'decay', term: 'decay', stage: 3, latex: String.raw`- \lambda c`, line: 'solid' },
  { id: 'arrhenius', term: 'diffusion_T', stage: 4, latex: String.raw`D_i \to D_i(T(t))`, line: 'dotted' },
];

/** The radial domain of the transport equation, copied from (2.7). */
export const DOMAIN: { latex: string; source: EqSource } = {
  latex: String.raw`R_{i-1} < r < R_i,\quad i = 1,\dots,5`,
  source: {
    doc: 'v1',
    tag: '2.7',
    section: '2.6 Layer equation of v1.0.0',
    verbatim: String.raw`R_{\ell-1} < r < R_\ell,\quad \ell = 1,\dots,5`,
    notation: [['\\ell', 'i']],
  },
};

export interface StageDef {
  stage: Stage;
  name: string;
  /** Short name for the stage picker. */
  short: string;
  /** The catalogue mechanism whose status the stage picker shows; null for the catalogue stage. */
  mechanism: string | null;
  newTerms: TermId[];
  switches: SwitchSet;
  summary: string;
}

export const STAGE_DEFS: Record<Stage, StageDef> = {
  1: {
    stage: 1,
    name: 'Base diffusion',
    short: 'Base diffusion',
    mechanism: 'diffusion_layered',
    newTerms: ['lhs', 'diffusion'],
    switches: { source: false, decay: false, arrhenius: false },
    summary: 'The equation the verified STRATO solver solves today: diffusion through five spherical layers, each with its own constant diffusivity.',
  },
  2: {
    stage: 2,
    name: 'Generation',
    short: 'Generation',
    mechanism: 'generation',
    newTerms: ['source'],
    switches: { source: true, decay: false, arrhenius: false },
    summary: 'Adds the fission-product source in the kernel.',
  },
  3: {
    stage: 3,
    name: 'Decay',
    short: 'Decay',
    mechanism: 'decay',
    newTerms: ['decay'],
    switches: { source: true, decay: true, arrhenius: false },
    summary: 'Adds radioactive decay in every layer.',
  },
  4: {
    stage: 4,
    name: 'Temperature-dependent diffusivity',
    short: 'Temperature',
    mechanism: 'diffusivity_temperature',
    newTerms: ['diffusion_T'],
    switches: { source: true, decay: true, arrhenius: true },
    summary: 'Lets each diffusivity follow the particle temperature over time.',
  },
  5: {
    stage: 5,
    name: 'Layer-specific temperature laws',
    short: 'Layer laws',
    mechanism: 'diffusivity_layer_laws',
    newTerms: ['arrhenius_layers', 'partition'],
    switches: { source: true, decay: true, arrhenius: true },
    summary: 'Gives each layer its own Arrhenius law; the layering itself is already in stage one.',
  },
  6: {
    stage: 6,
    name: 'Species-specific mechanisms',
    short: 'Species',
    mechanism: null,
    newTerms: [],
    switches: { source: true, decay: true, arrhenius: true },
    summary: 'Only mechanisms that STRATO implements belong here; none does yet.',
  },
};

export function stageSwitches(stage: Stage): SwitchSet {
  return STAGE_DEFS[stage].switches;
}

/** Terms of a stage that are listed but not part of any equation. */
export function listedOnly(stage: Stage): TermId[] {
  return (STAGES.filter((s) => s <= stage) as Stage[]).flatMap((s) => STAGE_DEFS[s].newTerms).filter((t) => TERMS[t].latex === '');
}

/** The system equation (one law per layer) shown beside the transport equation from stage 5 on. */
export function systemTerms(stage: Stage): TermId[] {
  return stage >= 5 ? ['arrhenius_layers'] : [];
}

// ---------------------------------------------------------------------------
// Conditions, numerical implementation and balance of each stage
// ---------------------------------------------------------------------------

export interface ShownEquation {
  id: string;
  label: string;
  latex: string; // displayed, may contain \htmlClass highlights
  source: EqSource;
  note?: string;
}

const L = (s: TemplateStringsArray) => String.raw(s);

const ELL = [['\\ell', 'i']] as const;

const CONDITIONS_V1: ShownEquation[] = [
  {
    id: 'centre',
    label: 'Centre: symmetry, nothing crosses it',
    latex: L`\frac{\partial c_1}{\partial r}(0, t) = 0`,
    source: { doc: 'v1', tag: '3.1', section: '3.1.4 Result', verbatim: L`\frac{\partial c_1}{\partial r}(0, t) = 0` },
  },
  {
    id: 'interface_c',
    label: 'Interfaces: concentration continuous',
    latex: L`c_i(R_i^-, t) = c_{i+1}(R_i^+, t), \qquad i = 1, \dots, 4,\quad t > 0`,
    source: {
      doc: 'v1',
      tag: '3.2',
      section: '3.2.1 Concentration continuity (an assumption about contact)',
      verbatim: L`c_\ell(R_\ell^-, t) = c_{\ell+1}(R_\ell^+, t), \qquad \ell = 1, \dots, 4,\quad t > 0`,
      notation: ELL,
    },
  },
  {
    id: 'interface_flux',
    label: 'Interfaces: flux continuous',
    latex: L`D_i\,\frac{\partial c_i}{\partial r}(R_i^-, t) = D_{i+1}\,\frac{\partial c_{i+1}}{\partial r}(R_i^+, t), \qquad i = 1, \dots, 4,\quad t > 0`,
    source: {
      doc: 'v1',
      tag: '3.3',
      section: '3.2.2 Flux continuity (derived from conservation)',
      verbatim: L`D_\ell\,\frac{\partial c_\ell}{\partial r}(R_\ell^-, t) = D_{\ell+1}\,\frac{\partial c_{\ell+1}}{\partial r}(R_\ell^+, t), \qquad \ell = 1, \dots, 4,\quad t > 0`,
      notation: ELL,
    },
  },
  {
    id: 'surface',
    label: 'Outer surface: a perfect sink',
    latex: L`c_5(R, t) = 0, \qquad t > 0`,
    source: { doc: 'v1', tag: '3.4', section: '3.3 Outer surface: a perfect sink', verbatim: L`c_5(R, t) = 0, \qquad t > 0` },
  },
  {
    id: 'initial',
    label: 'Start: uniform in the kernel, none in the coatings',
    latex: L`c(r, 0) = \begin{cases} c_0, & 0 \le r < R_1, \\ 0, & R_1 < r < R. \end{cases}`,
    source: { doc: 'v1', tag: '3.5', section: '3.4 Initial state', verbatim: L`c(r, 0) = \begin{cases} c_0, & 0 \le r < R_1, \\ 0, & R_1 < r < R. \end{cases}` },
  },
];

const CONDITIONS_P22: ShownEquation[] = [
  {
    id: 'conditions_T',
    label: 'Interfaces, centre, surface and start, with temperature-dependent diffusivities',
    latex: L`c\ \text{and}\ D_i\,\partial_r c\ \text{continuous at}\ r = R_i,\quad \partial_r c(0,t) = 0,\quad c(R,t) = 0,\quad c(r,0) = c_{\mathrm{init}}(r)`,
    source: {
      doc: 'p2',
      tag: 'P22.12',
      section: '5.2.1 Continuous problem',
      verbatim: L`c\ \text{and}\ D_i\,\partial_r c\ \text{continuous at}\ r = R_i,\quad \partial_r c(0,t) = 0,\quad c(R,t) = 0,\quad c(r,0) = c_{\mathrm{init}}(r)`,
    },
    note: 'Here Dᵢ stands for Dᵢ(T(t)); only the diffusivity changes with time.',
  },
];

export function conditionsFor(stage: Stage): { equations: ShownEquation[]; note: string } {
  if (stage === 1) {
    return {
      equations: CONDITIONS_V1,
      note: 'cᵢ is the concentration restricted to layer i. These are the conditions that STRATO v1.0 solves.',
    };
  }
  if (stage <= 3) {
    return {
      equations: CONDITIONS_V1,
      note: 'Decay and the source change no interface or boundary condition (Phase 2 derivations, Section 3.1); the start profile stays uniform in the kernel (owner decision D13).',
    };
  }
  return { equations: CONDITIONS_P22, note: 'From stage four on the conditions are written with temperature-dependent diffusivities (Phase 2 derivations, Section 5.2.1).' };
}

export interface Implementation {
  equations: ShownEquation[];
  /** Code that implements the stage at the build commit; empty while no code exists. */
  code: (CodeRef & { label: string })[];
  /** Pseudocode blocks to read while no code exists. */
  pseudocode: DocRef[];
  derivation: DocRef[];
  note: string;
}

const P21_58 = L`\Big(\frac{V_j}{\Delta t}+\lambda V_j+G_W+G_E\Big)C_j^{n+1}-G_WC_{j-1}^{n+1}-G_EC_{j+1}^{n+1}=\frac{V_j}{\Delta t}C_j^n+V_j\bar S_j^{n+1}`;
const P21_57A = L`\bar S_j^{n+1}=\frac{1}{\Delta t}\int_{t^n}^{t^{n+1}}\bar S_j(t)\,dt`;
const P22_15 = L`V_j\frac{C_j^{n+1} - C_j^n}{\Delta t_n} = G^{(n)}_{j-1/2}(C^{n+1}_{j-1} - C^{n+1}_j) - G^{(n)}_{j+1/2}(C^{n+1}_j - C^{n+1}_{j+1}) - \lambda V_jC_j^{n+1} + V_jS_j^{(n)}`;
const P22_14 = L`G_f(t) = \frac{A_f}{\delta_f^-/D_{i(f^-)}(T(t)) + \delta_f^+/D_{i(f^+)}(T(t))},\qquad G_{N+1/2}(t) = \frac{4\pi R^2 D_5(T(t))}{R - \bar r_N}`;
const OPTION_A = L`\bar D_i^{(n)} = \frac{1}{\Delta t_n}\int_{t_n}^{t_{n+1}} D_i(T(t))\,dt`;

/** Wraps `part` of `latex` (its first occurrence) in a highlight for mechanism `kind`. */
export function highlight(latex: string, part: string, kind: string): string {
  const at = latex.indexOf(part);
  if (at < 0) throw new Error(`highlight: '${part}' not found`);
  return `${latex.slice(0, at)}\\htmlClass{eq-new eq-new--${kind}}{${part}}${latex.slice(at + part.length)}`;
}

const BE_DECAY: DocRef = { doc: 'p2', section: '4.3 (b) Backward Euler with implicit decay' };
const BLOCK_H: DocRef = { doc: 'pseudo', section: '4.9 Block H: one time step' };

export function implementationFor(stage: Stage): Implementation {
  switch (stage) {
    case 1:
      return {
        equations: [
          {
            id: 'be_cell',
            label: 'Backward Euler balance of cell j over one step',
            latex: L`-g^W_j\,C_{j-1}^{n+1} + \left(v_j + g^W_j + g^E_j\right)C_j^{n+1} - g^E_j\,C_{j+1}^{n+1} = v_j\,C_j^n = \mathrm{rhs}_j`,
            source: {
              doc: 'v1',
              tag: '5.14',
              section: '5.8.2 Backward Euler for every cell',
              verbatim: L`-g^W_j\,C_{j-1}^{n+1} + \left(v_j + g^W_j + g^E_j\right)C_j^{n+1} - g^E_j\,C_{j+1}^{n+1} = v_j\,C_j^n = \mathrm{rhs}_j`,
            },
            note: 'vⱼ = Vⱼ/Δt; gᵂⱼ and gᴱⱼ are the conductances of the inner and outer face of cell j, from (5.9).',
          },
          {
            id: 'conductance',
            label: 'Face conductance: two half-cell resistances in series',
            latex: L`A_{j+1/2}J_{j+1/2} = G_{j+1/2}\,(C_j - C_{j+1}), \qquad G_{j+1/2} = \frac{A_{j+1/2}}{\delta^L_{j+1/2}/D_j + \delta^R_{j+1/2}/D_{j+1}}`,
            source: {
              doc: 'v1',
              tag: '5.9',
              section: '5.5 Face flux and the series conductance',
              verbatim: L`A_{j+1/2}J_{j+1/2} = G_{j+1/2}\,(C_j - C_{j+1}), \qquad G_{j+1/2} = \frac{A_{j+1/2}}{\delta^L_{j+1/2}/D_j + \delta^R_{j+1/2}/D_{j+1}}`,
            },
          },
        ],
        code: [
          { label: 'backward_euler_step, one implicit step', path: 'src/numerical/finite_volume.rs', line: 162, symbol: 'pub fn backward_euler_step' },
          { label: 'build_mesh, cell volumes and face conductances', path: 'src/numerical/finite_volume.rs', line: 50, symbol: 'pub fn build_mesh' },
        ],
        pseudocode: [],
        derivation: [
          { doc: 'v1', section: '5.8.2 Backward Euler for every cell' },
          { doc: 'v1', section: '5.5 Face flux and the series conductance' },
        ],
        note: 'Implemented and verified in STRATO v1.0: the finite-volume cell balance with exact shell volumes, solved by a subtraction-free tridiagonal elimination.',
      };
    case 2:
      return {
        equations: [
          {
            id: 'be_decay_source',
            label: 'Backward Euler balance of cell j with the source',
            latex: highlight(P21_58, L`V_j\bar S_j^{n+1}`, 'source'),
            source: { doc: 'p2', tag: 'P21.58', section: '4.3 (b) Backward Euler with implicit decay', verbatim: P21_58 },
            note: 'At this stage the decay constant is zero; stage three adds it.',
          },
          {
            id: 'step_average',
            label: 'Exact step average of the source (owner decision D11)',
            latex: P21_57A,
            source: { doc: 'p2', tag: 'P21.57', section: '4.3 (b) Backward Euler with implicit decay', verbatim: P21_57A },
          },
        ],
        code: [],
        pseudocode: [BLOCK_H, { doc: 'pseudo', section: '4.6 Block E: yield and source history' }],
        derivation: [BE_DECAY],
        note: 'Derived, not implemented: no Rust code exists for this stage.',
      };
    case 3:
      return {
        equations: [
          {
            id: 'be_decay_source',
            label: 'Backward Euler balance of cell j with implicit decay and the source',
            latex: highlight(P21_58, L`\lambda V_j`, 'decay'),
            source: { doc: 'p2', tag: 'P21.58', section: '4.3 (b) Backward Euler with implicit decay', verbatim: P21_58 },
            note: 'Decay is taken at the new time level, unsplit (owner decision D6).',
          },
        ],
        code: [],
        pseudocode: [BLOCK_H, { doc: 'pseudo', section: '4.3 Block B: decay constant from half-life' }],
        derivation: [BE_DECAY],
        note: 'Derived, not implemented: no Rust code exists for this stage.',
      };
    case 4:
      return {
        equations: [
          {
            id: 'be_recomputed',
            label: 'Backward Euler with conductances recomputed every step',
            latex: highlight(highlight(P22_15, L`G^{(n)}_{j-1/2}`, 'arrhenius'), L`G^{(n)}_{j+1/2}`, 'arrhenius'),
            source: { doc: 'p2', tag: 'P22.15', section: '5.2.3 Backward Euler with recomputed conductances', verbatim: P22_15 },
          },
          {
            id: 'option_a',
            label: 'Step-average diffusivity, option A (owner decision D8)',
            latex: OPTION_A,
            source: { doc: 'p2', tag: null, section: '5.2.3 Backward Euler with recomputed conductances', verbatim: OPTION_A },
          },
        ],
        code: [],
        pseudocode: [{ doc: 'pseudo', section: '4.4 Block C: Arrhenius diffusivity of a layer' }, { doc: 'pseudo', section: '4.5 Block D: temperature history' }, BLOCK_H],
        derivation: [{ doc: 'p2', section: '5.2.3 Backward Euler with recomputed conductances' }],
        note: 'Derived, not implemented: no Rust code exists for this stage.',
      };
    case 5:
      return {
        equations: [
          {
            id: 'conductance_T',
            label: 'Face conductance with the law of each neighbouring layer',
            latex: highlight(highlight(P22_14, L`D_{i(f^-)}(T(t))`, 'arrhenius'), L`D_{i(f^+)}(T(t))`, 'arrhenius'),
            source: { doc: 'p2', tag: 'P22.14', section: '5.2.2 Semi-discrete equations', verbatim: P22_14 },
            note: 'Each face takes the diffusivities of the two layers it joins, each from that layer’s own law.',
          },
        ],
        code: [],
        pseudocode: [{ doc: 'pseudo', section: '4.4 Block C: Arrhenius diffusivity of a layer' }],
        derivation: [
          { doc: 'p2', section: '5.2.2 Semi-discrete equations' },
          { doc: 'p2', section: '5.4 (d) General multilayer case: layer-specific activation energies' },
        ],
        note: 'Derived, not implemented: no Rust code exists for this stage.',
      };
    case 6:
      return {
        equations: [],
        code: [],
        pseudocode: [],
        derivation: [],
        note: 'No species-specific mechanism has a discrete form in STRATO.',
      };
  }
}

export function balanceFor(stage: Stage): { equations: ShownEquation[]; note: string } {
  if (stage === 1) {
    return {
      equations: [
        {
          id: 'balance_v1',
          label: 'Inventory plus released amount equals the start inventory',
          latex: L`M(t) + M_{\mathrm{rel}}(t) = M^0`,
          source: { doc: 'v1', tag: null, section: '3.5.4 Cumulative release and release fraction', verbatim: L`M(t) + M_{\mathrm{rel}}(t) = M^0` },
        },
        {
          id: 'release_fraction',
          label: 'Release fraction',
          latex: L`F(t) = \frac{M_{\mathrm{rel}}(t)}{M^0} = 1 - \frac{M(t)}{M^0}`,
          source: { doc: 'v1', tag: '3.9', section: '3.5.4 Cumulative release and release fraction', verbatim: L`F(t) = \frac{M_{\mathrm{rel}}(t)}{M^0} = 1 - \frac{M(t)}{M^0}` },
        },
      ],
      note: 'The values of each term and the residual for the current run are under “Inventory balance” in the live simulation below, as the Rust wrapper returns them; nothing is computed in the browser.',
    };
  }
  return {
    equations: [
      {
        id: 'balance_p21',
        label: 'Start inventory plus generated equals inventory plus decayed plus released',
        latex: L`M_0 + M_\mathrm{gen}(t) = M(t) + M_\mathrm{dec}(t) + M_\mathrm{rel}(t)`,
        source: { doc: 'p2', tag: 'P21.42', section: '3.6 (f) Inventory closure', verbatim: L`M_0 + M_\mathrm{gen}(t) = M(t) + M_\mathrm{dec}(t) + M_\mathrm{rel}(t)` },
      },
    ],
    note: 'Each term and the residual will come from the Rust wrapper once the solver implements this stage (web phase 6.1); nothing is computed in the browser.',
  };
}

// ---------------------------------------------------------------------------
// Assembly of the transport equation from the switches (display only)
// ---------------------------------------------------------------------------

export interface Assembled {
  terms: TermId[];
  latex: string;
  tag: string;
  /** LaTeX conditions under which the cited equation reduces to the one shown. */
  reductions: string[];
}

function wrap(term: TermId, latex: string, highlighted: readonly TermId[]): string {
  const kind = term === 'diffusion_T' ? 'arrhenius' : term;
  const cls = highlighted.includes(term) ? `eq-term eq-term--${term} eq-new eq-new--${kind}` : `eq-term eq-term--${term}`;
  return `\\htmlClass{${cls}}{${latex}}`;
}

/**
 * The transport equation for a set of switches, in the term order of (P21.1) and (P22.11). The
 * equation number is the repository equation it reduces from; `reductions` names what is set to zero.
 */
export function assemble(sw: SwitchSet, highlighted: readonly TermId[] = []): Assembled {
  const terms: TermId[] = ['lhs', sw.arrhenius ? 'diffusion_T' : 'diffusion'];
  if (sw.decay) terms.push('decay');
  if (sw.source) terms.push('source');
  const [lhs, ...rhs] = terms;
  const latex = `${wrap(lhs as TermId, TERMS.lhs.latex, highlighted)} = ${rhs.map((t) => wrap(t, TERMS[t].latex, highlighted)).join(' ')}`;
  const reductions: string[] = [];
  let tag: string;
  if (sw.arrhenius || sw.decay || sw.source) {
    tag = sw.arrhenius ? 'P22.11' : 'P21.1';
    if (!sw.decay) reductions.push(String.raw`\lambda = 0`);
    if (!sw.source) reductions.push(String.raw`S = 0`);
  } else {
    tag = '2.6';
    reductions.push(String.raw`S = 0`, String.raw`D = D_i \text{ in layer } i`);
  }
  return { terms, latex, tag, reductions };
}

export function assembleStage(stage: Stage): Assembled {
  return assemble(stageSwitches(stage), STAGE_DEFS[stage].newTerms);
}

/** Every term shown at a stage: the transport equation's terms, the system equations and the listed ones. */
export function stageTerms(stage: Stage): TermId[] {
  return [...assembleStage(stage).terms, ...systemTerms(stage), ...listedOnly(stage)];
}

/** The switch the stage introduces, if any (keeps the stage picker and the switches in step). */
export function switchIntroducedBy(stage: Stage): SwitchId | null {
  const term = STAGE_DEFS[stage].newTerms.map((t) => TERMS[t]).find((t) => t.switchId);
  return term?.switchId ?? null;
}
