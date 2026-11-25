// Step and part table of the progressive equation builder for the BTCS solver (directive Section 16:
// the fourteen steps, then three labelled previews). Display only: nothing here is evaluated. No
// research status is written here; the builder reads every status from the generated catalogue.
//
// Every equation shown is copied character for character from one of the BTCS documents
// (docs/mathematics/btcs: Document 1 = d1, Document 2 = d2) and carries the equation number printed
// there: a Document 1 number is shown as D1.n, a Document 2 number as written, for example F10.
// web/tests/terms.test.ts reads the documents and checks every copy, so the page cannot drift from
// them. An untagged line is found by its section and says so. Where a line is only part of a
// documented line, the shown text is a piece of it; nothing is rewritten.
//
// The notation is that of directive Section 5: i the radial node index, j the time-step index,
// C_i^j the concentration, the powers parenthesised. Generation, decay and temperature dependence
// are labelled previews (coordinator decision 11): they show only the places that Document 2,
// Section 9 marks for them.
import type { CodeRef, DocId, DocRef } from './links';

/** Steps one to fourteen are the BTCS steps; fifteen to seventeen are the labelled previews. */
export type Stage = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17;
export const STAGES: readonly Stage[] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
export const CORE_STAGES: readonly Stage[] = STAGES.filter((s) => s <= 14);
export const PREVIEW_STAGES: readonly Stage[] = STAGES.filter((s) => s >= 15);
/** The last step whose equations the BTCS solver solves; steps thirteen and fourteen are content, not equations of a run. */
export const LAST_SOLVED_STEP: Stage = 12;

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
  doc: DocId;
  /** Equation tag as printed in the document, for example 'F10' or '4'; null for an untagged line, found in `section`. */
  tag: string | null;
  section: string;
  /** Copied character for character from the document (the shown LaTeX is exactly this). */
  verbatim: string;
  /** True only when the shown LaTeX is a declared piece of a longer tagged equation; it must start and end at a top-level separator. */
  piece?: boolean;
}

export interface UnitRow {
  symbol: string; // LaTeX, as written in the symbol tables of the documents
  meaning: string;
  unit: string; // LaTeX
}

export type TermId = string;

export interface PartDef {
  id: TermId;
  name: string;
  /** Short symbol for the part list (display only). */
  symbol: string;
  /** The piece of a shown equation that this part stands for, copied from it; empty when the part is listed but not marked. */
  latex: string;
  /** Mechanism colour of the highlight, for the labelled previews. */
  highlight?: 'source' | 'decay' | 'arrhenius';
  /** Catalogue mechanism whose research status the part shows; null for the BTCS steps (their evidence is in the documents). */
  mechanism: string | null;
  /** Plain-language meaning first, then the symbols. Written without numerals (block A shows it). */
  meaning: string;
  units: UnitRow[];
  assumptions: string[];
  derivation: DocRef;
}

export interface ShownEquation {
  id: string;
  label: string;
  latex: string; // displayed; may contain \htmlClass highlights
  source: EqSource;
  /** Number shown beside an untagged line that the document names in its text, for example F16a. */
  numberAs?: string;
  note?: string;
}

export interface Implementation {
  equations: ShownEquation[];
  /** Code of the BTCS solver (branch feat/btcs-solver) that implements the step. */
  code: (CodeRef & { label: string })[];
  /** Pseudocode routines of Document 3 to read beside the code. */
  pseudocode: (DocRef & { label?: string })[];
  derivation: DocRef[];
  note: string;
}

export interface StepDef {
  stage: Stage;
  name: string;
  /** Short name for the step picker. */
  short: string;
  kind: 'step' | 'preview';
  /** Catalogue mechanism whose research status the picker shows; null for the BTCS steps. */
  mechanism: string | null;
  summary: string;
  main: ShownEquation[];
  parts: PartDef[];
  implementation: Implementation;
  /** What the live simulation shows for this step, and where. */
  live: string;
}

// ---------------------------------------------------------------------------
// Sources
// ---------------------------------------------------------------------------

const D1_1 = "1. From conservation of mass and Fick's law to the spherical diffusion equation";
const D1_2 = '2. The constant-diffusivity form inside one layer';
const D1_32 = "3.2 Central differences in space, from Taylor's theorem";
const D1_33 = "3.3 Backward Euler in time, from Taylor's theorem";
const D1_34 = '3.4 The BTCS interior equation and its tridiagonal coefficients';
const D1_4 = '4. A2: the centre node';
const D1_42 = '4.2 Ghost node and the BTCS centre row';
const D1_5 = '5. A3: the four material interfaces';
const D1_6 = '6. A4: the absorbing outer boundary and the release rate';
const D1_7 = '7. A5: the initial condition on the grid';
const D1_9 = "9. Matrix properties used by the scheme (coordinator's hypothesis 4)";
const D1_10 = '10. The assembled tridiagonal system, row by row';
const D2_1 = '1. Scope and model';
const D2_3 = '3. Grid';
const D2_4 = '4. Geometric coefficients';
const D2_5 = '5. The matrix rows';
const D2_6 = '6. Right-hand side, initial vector and solution';
const D2_7 = '7. Inventory, release and balance';
const D2_8 = '8. Proven properties and their hypotheses';
const D2_9 = '9. Placeholders for the approved additional physics (directive Section 17)';

const L = (s: TemplateStringsArray) => String.raw(s);

const eq = (id: string, label: string, doc: DocId, tag: string | null, section: string, verbatim: string, extra: { numberAs?: string; note?: string; piece?: boolean } = {}): ShownEquation => {
  const { piece, ...rest } = extra;
  return { id, label, latex: verbatim, source: { doc, tag, section, verbatim, ...(piece ? { piece: true } : {}) }, ...rest };
};

// Document 1
const D1_EQ1 = eq('mass_shell', 'Balance on a thin shell', 'd1', '1', D1_1, L`\frac{d}{dt}\int_r^{r+\delta r} C(s,t)\,4\pi\left(s\right)^2\,ds = \Phi(r,t) - \Phi(r+\delta r,t)`);
const D1_EQ2 = eq('mass_local', 'The shell shrunk to a radius', 'd1', '2', D1_1, L`4\pi\left(r\right)^2\,\frac{\partial C}{\partial t} = -\frac{\partial \Phi}{\partial r} = -\frac{\partial}{\partial r}\left[4\pi\left(r\right)^2 J\right]`);
const D1_EQ3 = eq('fick', "Fick's first law", 'd1', '3', D1_1, L`J = -D(r)\,\frac{\partial C}{\partial r}`);
const D1_EQ4 = eq('spherical', 'Spherical diffusion equation', 'd1', '4', D1_1, L`\frac{\partial C}{\partial t} = \frac{1}{\left(r\right)^2}\frac{\partial}{\partial r}\left[\left(r\right)^2 D(r)\,\frac{\partial C}{\partial r}\right]`);
const D1_EQ5 = eq('spherical_layer', 'Inside one material, the diffusivity is a constant', 'd1', '5', D1_2, L`\frac{\partial C}{\partial t} = D\left[\frac{\partial^2 C}{\partial r^2} + \frac{2}{r}\frac{\partial C}{\partial r}\right]`, { piece: true });
const D1_EQ9 = eq('second_difference', 'Central second difference, with its error', 'd1', '9', D1_32, L`\frac{\partial^2 C}{\partial r^2}(r_i) = \frac{C_{i+1} - 2C_i + C_{i-1}}{\left(\Delta r\right)^2} - \frac{\left(\Delta r\right)^2}{12}C_i'''' + O\left(\left(\Delta r\right)^4\right)`);
const D1_EQ10 = eq('first_difference', 'Central first difference, with its error', 'd1', '10', D1_32, L`\frac{\partial C}{\partial r}(r_i) = \frac{C_{i+1} - C_{i-1}}{2\,\Delta r} - \frac{\left(\Delta r\right)^2}{6}C_i''' + O\left(\left(\Delta r\right)^4\right)`);
const D1_EQ11 = eq('time_quotient', 'Backward Euler: the time quotient belongs to the new time level', 'd1', '11', D1_33, L`\frac{\partial C}{\partial t}(r_i, t^{j+1}) = \frac{C_i^{j+1} - C_i^j}{\Delta t} + \frac{\Delta t}{2}\ddot C(r_i, t^{j+1}) + O\left(\left(\Delta t\right)^2\right)`);
const D1_EQ12 = eq('btcs_interior', 'The equation at an interior node, all values at the new time level', 'd1', '12', D1_34, L`\frac{C_i^{j+1} - C_i^j}{\Delta t} = D\left[\frac{C_{i+1}^{j+1} - 2C_i^{j+1} + C_{i-1}^{j+1}}{\left(\Delta r\right)^2} + \frac{2}{r_i}\,\frac{C_{i+1}^{j+1} - C_{i-1}^{j+1}}{2\,\Delta r}\right]`);
const D1_EQ14 = eq('btcs_row', 'Three unknowns on the left, the known old value on the right', 'd1', '14', D1_34, L`L_i\,C_{i-1}^{j+1} + B_i\,C_i^{j+1} + U_i\,C_{i+1}^{j+1} = C_i^j`);
const D1_EQ15 = eq('btcs_coefficients', 'The three coefficients of an interior node', 'd1', '15', D1_34, L`L_i = -\mathrm{Fo}\left(1 - \frac{\Delta r}{r_i}\right), \qquad B_i = 1 + 2\,\mathrm{Fo}, \qquad U_i = -\mathrm{Fo}\left(1 + \frac{\Delta r}{r_i}\right)`);
const D1_EQ22 = eq('even_series', 'Near the centre the concentration is an even function of the radius', 'd1', '22', D1_4, L`C(r,t) = C(0,t) + \frac{\left(r\right)^2}{2}C''(0,t) + \frac{\left(r\right)^4}{24}C''''(0,t) + O\left(\left(r\right)^6\right)`);
const D1_EQ23 = eq('centre_limit', 'At the centre the operator is three times the second derivative', 'd1', '23', D1_4, L`\frac{\partial^2 C}{\partial r^2} + \frac{2}{r}\frac{\partial C}{\partial r} = 3\,C''(0,t) + \frac{5}{6}\left(r\right)^2 C''''(0,t) + \dots \xrightarrow{\ r\to0\ } 3\left.\frac{\partial^2 C}{\partial r^2}\right|_{r=0}`);
const D1_EQ24 = eq('ghost_node', 'The ghost node equals the first node', 'd1', '24', D1_42, L`\left.\frac{\partial^2 C}{\partial r^2}\right|_{r=0} \approx \frac{C_1 - 2C_0 + C_{-1}}{\left(\Delta r_{0,\mathrm{outer}}\right)^2} = \frac{2\left(C_1 - C_0\right)}{\left(\Delta r_{0,\mathrm{outer}}\right)^2}`);
const D1_EQ26 = eq('interface_c', 'Interfaces: the concentration is continuous', 'd1', '26', D1_5, L`C(R_1^-, t) = C(R_1^+, t), \quad C(R_2^-, t) = C(R_2^+, t), \quad C(R_3^-, t) = C(R_3^+, t), \quad C(R_4^-, t) = C(R_4^+, t)`);
const D1_EQ27 = eq('interface_flux', 'Interface at the kernel surface: the flux is continuous', 'd1', '27', D1_5, L`D_{\mathrm{kernel}}\left.\frac{\partial C}{\partial r}\right|_{R_1^-} = D_{\mathrm{buffer}}\left.\frac{\partial C}{\partial r}\right|_{R_1^+}`);
const D1_EQ38 = eq('initial', 'Start: uniform in the kernel, none in the coatings', 'd1', '38', D1_7, L`C(r,0) = \begin{cases} C_{\mathrm{init}}, & 0 \le r < R_1,\\ 0, & R_1 < r < R_5. \end{cases}`);

// Document 2
const F1 = eq('grid_nodes', 'Node index of each interface and of the outer node', 'd2', 'F1', D2_3, L`N_{\mathrm{kernel}} = n_{\mathrm{kernel}}, \quad N_{\mathrm{buffer}} = N_{\mathrm{kernel}} + n_{\mathrm{buffer}}, \quad N_{\mathrm{IPyC}} = N_{\mathrm{buffer}} + n_{\mathrm{IPyC}}, \quad N_{\mathrm{SiC}} = N_{\mathrm{IPyC}} + n_{\mathrm{SiC}}, \quad N = N_{\mathrm{SiC}} + n_{\mathrm{OPyC}}`);
const F2 = eq('grid_spacing', 'Uniform spacing inside each material', 'd2', 'F2', D2_3, L`\Delta r_{\mathrm{kernel}} = \frac{R_1}{n_{\mathrm{kernel}}}, \quad \Delta r_{\mathrm{buffer}} = \frac{R_2 - R_1}{n_{\mathrm{buffer}}}, \quad \Delta r_{\mathrm{IPyC}} = \frac{R_3 - R_2}{n_{\mathrm{IPyC}}}, \quad \Delta r_{\mathrm{SiC}} = \frac{R_4 - R_3}{n_{\mathrm{SiC}}}, \quad \Delta r_{\mathrm{OPyC}} = \frac{R_5 - R_4}{n_{\mathrm{OPyC}}}`);
const F6 = eq('conductance', 'Conductance of the interval between two nodes (exact for a spherical shell)', 'd2', 'F6', D2_4, L`K_{i,\mathrm{outer}} = \frac{D_{i,\mathrm{outer}}\,r_i\,r_{i+1}}{\Delta r_{i,\mathrm{outer}}} \quad (0 \le i \le N-1), \qquad K_{i,\mathrm{inner}} = K_{i-1,\mathrm{outer}} \quad (1 \le i \le N)`);
const F7 = eq('storage_weight', 'Storage weight of a node: half of each adjacent interval', 'd2', 'F7', D2_4, L`V_i = \left(r_i\right)^2\,\frac{\Delta r_{i,\mathrm{inner}} + \Delta r_{i,\mathrm{outer}}}{2} \quad (1 \le i \le N-1), \qquad V_0 = 0`);
const F8 = eq('fourier', 'Mesh Fourier number of each material', 'd2', 'F8', D2_4, L`\mathrm{Fo}_{\mathrm{material}} = \frac{D_{\mathrm{material}}\,\Delta t}{\left(\Delta r_{\mathrm{material}}\right)^2}, \qquad \text{material} \in \left\{\mathrm{kernel}, \mathrm{buffer}, \mathrm{IPyC}, \mathrm{SiC}, \mathrm{OPyC}\right\}`);
const F9 = eq('row_form', 'Row i of the system', 'd2', 'F9', D2_5, L`L_i\,C_{i-1}^{j+1} + B_i\,C_i^{j+1} + U_i\,C_{i+1}^{j+1} = b_i^j`);
const F10 = eq('flux_balance', 'Backward Euler flux balance of a node', 'd2', 'F10', D2_5, L`V_i\,\frac{C_i^{j+1} - C_i^j}{\Delta t} = K_{i,\mathrm{outer}}\left(C_{i+1}^{j+1} - C_i^{j+1}\right) - K_{i,\mathrm{inner}}\left(C_i^{j+1} - C_{i-1}^{j+1}\right)`);
const F11 = eq('row_coefficients', 'Coefficients of the general row', 'd2', 'F11', D2_5, L`L_i = -\frac{\Delta t\,K_{i,\mathrm{inner}}}{V_i}, \qquad U_i = -\frac{\Delta t\,K_{i,\mathrm{outer}}}{V_i}, \qquad B_i = 1 - L_i - U_i = 1 + \left|L_i\right| + \left|U_i\right|`);
const F12 = eq('centre_row', 'Centre row', 'd2', 'F12', D2_5, L`\left(1 + 6\,\mathrm{Fo}_{\mathrm{kernel}}\right)C_0^{j+1} - 6\,\mathrm{Fo}_{\mathrm{kernel}}\,C_1^{j+1} = b_0^j, \qquad B_0 = 1 + 6\,\mathrm{Fo}_{\mathrm{kernel}}, \quad U_0 = -6\,\mathrm{Fo}_{\mathrm{kernel}}`);
const F13 = eq('one_way', 'The centre value drives nothing: the lower entry of row one is zero', 'd2', 'F13', D2_5, L`L_1 = 0`);
const F16 = eq('interface_row', 'Interface-node coefficients (the four interfaces differ in the inner and outer material)', 'd2', 'F16', D2_5, L`L_i = -\mathrm{Fo}_{\mathrm{in}}\,\frac{2\,\Delta r_{\mathrm{in}}}{\Delta r_{\mathrm{in}} + \Delta r_{\mathrm{out}}}\left(1 - \frac{\Delta r_{\mathrm{in}}}{R}\right), \qquad U_i = -\mathrm{Fo}_{\mathrm{out}}\,\frac{2\,\Delta r_{\mathrm{out}}}{\Delta r_{\mathrm{in}} + \Delta r_{\mathrm{out}}}\left(1 + \frac{\Delta r_{\mathrm{out}}}{R}\right), \qquad B_i = 1 - L_i - U_i`);
const F16A = eq(
  'interface_row_a',
  'The kernel-buffer interface written out',
  'd2',
  null,
  D2_5,
  L`L_{N_{\mathrm{kernel}}} = -\frac{2D_{\mathrm{kernel}}\,\Delta t\left(R_1 - \Delta r_{\mathrm{kernel}}\right)}{R_1\,\Delta r_{\mathrm{kernel}}\left(\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}\right)}, \qquad U_{N_{\mathrm{kernel}}} = -\frac{2D_{\mathrm{buffer}}\,\Delta t\left(R_1 + \Delta r_{\mathrm{buffer}}\right)}{R_1\,\Delta r_{\mathrm{buffer}}\left(\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}\right)}`,
  { numberAs: 'F16a', note: 'The other three interfaces follow by the substitutions of the table in the document.' },
);
const F17 = eq('last_row', 'Row next to the outer boundary: the boundary entry leaves the matrix, not the diagonal', 'd2', 'F17', D2_5, L`L_{N-1}\,C_{N-2}^{j+1} + B_{N-1}\,C_{N-1}^{j+1} = b_{N-1}^j, \qquad B_{N-1} = 1 + \left|L_{N-1}\right| + \left|U_{N-1}\right|`);
const F18 = eq('outer_value', 'Outer boundary: absorbing', 'd2', 'F18', D2_5, L`C_N^j = 0 \qquad \text{for every } j \ge 0`);
const F19 = eq(
  'matrix',
  'The assembled tridiagonal system',
  'd2',
  'F19',
  D2_5,
  L`\mathbf{A}\,\mathbf{C}^{j+1} = \mathbf{b}^j, \qquad
\mathbf{A} =
\begin{pmatrix}
B_0 & U_0 & & & & \\
0 & B_1 & U_1 & & & \\
 & L_2 & B_2 & U_2 & & \\
 & & \ddots & \ddots & \ddots & \\
 & & & L_{N-2} & B_{N-2} & U_{N-2} \\
 & & & & L_{N-1} & B_{N-1}
\end{pmatrix},
\qquad
\mathbf{C}^{j+1} = \begin{pmatrix} C_0^{j+1} \\ C_1^{j+1} \\ \vdots \\ C_{N-1}^{j+1} \end{pmatrix}`,
);
const F20 = eq('rhs', 'Right-hand side: the old profile', 'd2', 'F20', D2_6, L`b_i^j = C_i^j, \qquad 0 \le i \le N-1`);
const F21 = eq('initial_vector', 'Initial vector', 'd2', 'F21', D2_6, L`C_i^0 = C_{\mathrm{init}} \quad \left(0 \le i \le N_{\mathrm{kernel}} - 1\right), \qquad C_{N_{\mathrm{kernel}}}^0 = \theta\,C_{\mathrm{init}}, \qquad C_i^0 = 0 \quad \left(N_{\mathrm{kernel}} + 1 \le i \le N\right)`);
const F22 = eq('theta', 'Start value at the kernel surface, matched to the exact inventory', 'd2', 'F22', D2_6, L`\theta = \frac{\Delta r_{\mathrm{kernel}}}{\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}}\left(1 - \frac{1}{3\,n_{\mathrm{kernel}}}\right)`);
const F24 = eq(
  'thomas',
  'Thomas algorithm, in the form that the code uses',
  'd2',
  'F24',
  D2_6,
  L`\begin{aligned}
&m_0 = B_0, \qquad b'_0 = b_0^j,\\
&w_i = L_i/m_{i-1}, \quad m_i = B_i - w_i\,U_{i-1}, \quad b'_i = b_i^j - w_i\,b'_{i-1}, \qquad i = 1, \dots, N-1,\\
&C_{N-1}^{j+1} = b'_{N-1}/m_{N-1}, \qquad C_i^{j+1} = \left(b'_i - U_i\,C_{i+1}^{j+1}\right)/m_i, \qquad i = N-2, \dots, 0.
\end{aligned}`,
);
const F25 = eq('inventory', 'Discrete inventory', 'd2', 'F25', D2_7, L`M^j = 4\pi\sum_{i=1}^{N-1}V_i\,C_i^j`);
const F26 = eq('release_rate', 'Release rate through the outer surface, at the new time level', 'd2', 'F26', D2_7, L`\Phi^{j+1} = 4\pi\,K_{N-1,\mathrm{outer}}\,C_{N-1}^{j+1}, \qquad j \ge 0`);
const F27 = eq('cumulative', 'Cumulative release', 'd2', 'F27', D2_7, L`M_{\mathrm{rel}}^0 = 0, \qquad M_{\mathrm{rel}}^{j+1} = M_{\mathrm{rel}}^j + \Delta t\,\Phi^{j+1}, \qquad\text{so}\qquad M_{\mathrm{rel}}^j = \sum_{k=1}^{j}\Delta t\,\Phi^k`);
const F28 = eq('release_fraction', 'Release fraction', 'd2', 'F28', D2_7, L`F^j = \frac{M_{\mathrm{rel}}^j}{M^0}`);
const F29 = eq('balance', 'Balance identity: what leaves the particle is what the inventory lost', 'd2', 'F29', D2_7, L`M^{j+1} - M^j = -\Delta t\,\Phi^{j+1}, \qquad M^j + M_{\mathrm{rel}}^j = M^0 \quad \text{for every } j \ge 0 \ \text{(exact arithmetic)}`);
const F30 = eq('residual', 'Balance residual, reported at every output step', 'd2', 'F30', D2_7, L`\varepsilon^j = M^j + M_{\mathrm{rel}}^j - M^0, \qquad \frac{\varepsilon^j}{M^0} = F^j - \left(1 - \frac{M^j}{M^0}\right)`);
const F31 = eq('interface_flow', 'Outward flow at an interface node, from the inner and from the outer side', 'd2', 'F31', D2_7, L`\Phi_{R}^{j+1} = 4\pi K_{i,\mathrm{inner}}\left(C_{i-1}^{j+1} - C_i^{j+1}\right) - 4\pi\left(R\right)^2\frac{\Delta r_{i,\mathrm{inner}}}{2}\,\frac{C_i^{j+1} - C_i^j}{\Delta t} = 4\pi K_{i,\mathrm{outer}}\left(C_i^{j+1} - C_{i+1}^{j+1}\right) + 4\pi\left(R\right)^2\frac{\Delta r_{i,\mathrm{outer}}}{2}\,\frac{C_i^{j+1} - C_i^j}{\Delta t}`);
const P13 = eq('order_one_material', 'Convergence for one material, proved', 'd2', null, D2_8, L`\max_j\left\lVert \mathbf{e}^j\right\rVert _\infty = O\left(\Delta t + \left(\Delta r\right)^2\right)`, { numberAs: 'P13', note: 'Hypotheses (Document 2, Section 8): one material, a uniform grid, and a smooth solution on the whole time interval from the start to T. Proved.' });
const P14 = eq('order_five_layers', 'Convergence for five layers, a proof outline: the constants are not tracked', 'd2', null, D2_8, L`\max_j\left\lVert \mathbf{e}^j\right\rVert _V = O\left(\left(\Delta r\right)^2 + \Delta t\right)`, { numberAs: 'P14', note: 'Valid on [t_*, T], not from the first step: for a solution four times continuously differentiable in r on each layer and twice in t on [t_*, T], with the error at t_* of order (Δr) squared. The discontinuous start profile does not satisfy the smoothness hypothesis near the start. A proof outline.' });
const OUTER_CONT = eq('outer_continuous', 'Outer surface: absorbing', 'd2', null, D2_1, L`C(R_5, t) = 0`);
const G1_RHS = eq('g1_rhs', 'Right-hand side with the place of a source term', 'd2', null, D2_9, L`b_i^j = C_i^j + \left[\text{G1 source term}\right]`, { note: 'Not derived: the term is a placeholder that is zero until the derivation, review and verification of this mechanism exist.' });
const G1_BAL = eq('g1_balance', 'Balance with a generated amount', 'd2', null, D2_9, L`M^0 + M_{\mathrm{gen}}^j = M^j + M_{\mathrm{rel}}^j + \left[\text{G2 decayed amount}\right]`);
const G2_DIAG = eq('g2_diag', 'Diagonal with the place of a decay term', 'd2', null, D2_9, L`B_i = 1 + \left|L_i\right| + \left|U_i\right| + \left[\text{G2 decay term}\right]`, { note: 'Not derived: the term is a placeholder that is zero until the derivation, review and verification of this mechanism exist.' });
const G3_D = eq('g3_d', 'The diffusivity of each material becomes a function of temperature', 'd2', null, D2_9, L`D_{\mathrm{material}}(T)`, { note: 'Not derived in the documents of this build: only the place where the temperature enters is fixed.' });

// ---------------------------------------------------------------------------
// Units
// ---------------------------------------------------------------------------

const CONC = String.raw`\mathrm{mol\ m^{-3}}`;
const NONE = String.raw`\text{dimensionless}`;
const u = (symbol: string, meaning: string, unit: string): UnitRow => ({ symbol, meaning, unit });
const U_C = u('C(r,t)', 'concentration of the diffusing species', CONC);
const U_R = u('r', 'radial coordinate', String.raw`\mathrm{m}`);
const U_T = u('t', 'time', String.raw`\mathrm{s}`);
const U_D = u('D(r)', 'diffusivity at radius r', String.raw`\mathrm{m^2\ s^{-1}}`);
const U_DM = u(String.raw`D_{\mathrm{kernel}}, D_{\mathrm{buffer}}, D_{\mathrm{IPyC}}, D_{\mathrm{SiC}}, D_{\mathrm{OPyC}}`, 'diffusivity of each material, constant within the material', String.raw`\mathrm{m^2\ s^{-1}}`);
const U_I = u('i = 0, 1, \\dots, N', 'global radial node index; node zero at the centre, node N at the outer surface', NONE);
const U_J = u('j = 0, 1, 2, \\dots', 'time-step index', NONE);
const U_RI = u('r_i', 'radius of node i', String.raw`\mathrm{m}`);
const U_DT = u(String.raw`\Delta t`, 'time step', String.raw`\mathrm{s}`);
const U_CIJ = u('C_i^j', 'concentration at node i and time step j', CONC);
const U_FO = u(String.raw`\mathrm{Fo} = D\,\Delta t/\left(\Delta r\right)^2`, 'mesh Fourier number', NONE);
const U_K = u(String.raw`K_{i,\mathrm{outer}}`, 'conductance of the interval between node i and node i+1, divided by 4π', String.raw`\mathrm{m^3\ s^{-1}}`);
const U_V = u('V_i', 'storage weight of node i (nodal volume divided by 4π)', String.raw`\mathrm{m^3}`);
const U_LBU = u('L_i, B_i, U_i', 'lower, diagonal and upper coefficient of matrix row i', NONE);
const U_M = u(String.raw`M^j`, 'discrete inventory', String.raw`\mathrm{mol}`);
const U_PHI = u(String.raw`\Phi^{j}`, 'discrete release rate through the outer surface', String.raw`\mathrm{mol\ s^{-1}}`);
const U_MREL = u(String.raw`M_{\mathrm{rel}}^j`, 'cumulative release', String.raw`\mathrm{mol}`);
const U_F = u('F^j', 'release fraction', NONE);
const U_EPS = u(String.raw`\varepsilon^j`, 'balance residual', String.raw`\mathrm{mol}`);

const doc1 = (section: string): DocRef => ({ doc: 'd1', section });
const doc2 = (section: string): DocRef => ({ doc: 'd2', section });

const part = (p: Omit<PartDef, 'mechanism' | 'units' | 'assumptions'> & { units?: UnitRow[]; assumptions?: string[]; mechanism?: string | null }): PartDef => ({
  mechanism: null,
  units: [],
  assumptions: [],
  ...p,
});

// ---------------------------------------------------------------------------
// Code and pseudocode links (branch feat/btcs-solver; src/numerical, the lines are checked by the link test)
// ---------------------------------------------------------------------------

const code = (label: string, path: string, line: number, symbol: string) => ({ label, path, line, symbol });
const CODE = {
  fromRadii: code('Grid::from_radii, nodes, radii and weights', 'src/numerical/grid.rs', 136, 'pub fn from_radii'),
  coefficients: code('Coefficients::new, conductances and Fourier numbers', 'src/numerical/coefficients.rs', 31, 'pub fn new'),
  assemble: code('assemble, the rows of the matrix', 'src/numerical/btcs.rs', 151, 'pub fn assemble'),
  tridiagonal: code('Tridiagonal, the three diagonals', 'src/numerical/btcs.rs', 123, 'pub struct Tridiagonal'),
  step: code('Stepper::step, one implicit step', 'src/numerical/btcs.rs', 231, 'pub fn step'),
  run: code('run, the time loop', 'src/numerical/btcs.rs', 278, 'pub fn run'),
  thomas: code('ThomasFactors, the F24 factors', 'src/numerical/tridiagonal.rs', 152, 'pub struct ThomasFactors'),
  thomasNew: code('ThomasFactors::new, the forward sweep', 'src/numerical/tridiagonal.rs', 169, 'pub fn new'),
  thomasSolve: code('ThomasFactors::solve, the back substitution', 'src/numerical/tridiagonal.rs', 230, 'pub fn solve'),
  crossCheck: code('solve_in_place, the general Thomas solve kept as a cross-check', 'src/numerical/tridiagonal.rs', 79, 'pub fn solve_in_place'),
  centre: code('centre_row, the centre-node row', 'src/numerical/boundary_conditions.rs', 11, 'pub fn centre_row'),
  outer: code('outer_value, the absorbing boundary', 'src/numerical/boundary_conditions.rs', 17, 'pub fn outer_value'),
  release: code('release_rate, the outward flow', 'src/numerical/boundary_conditions.rs', 32, 'pub fn release_rate'),
  interior: code('interior_row_explicit, check form of an interior row', 'src/numerical/interfaces.rs', 13, 'pub fn interior_row_explicit'),
  interfaceRow: code('interface_row_explicit, check form of an interface row', 'src/numerical/interfaces.rs', 24, 'pub fn interface_row_explicit'),
  interfaceFlux: code('interface_flux_both_sides, the flow seen from both sides', 'src/numerical/interfaces.rs', 40, 'pub fn interface_flux_both_sides'),
  theta: code('kernel_surface_theta, the start value at the kernel surface', 'src/numerical/inventory.rs', 12, 'pub fn kernel_surface_theta'),
  initial: code('initial_vector, the start profile', 'src/numerical/inventory.rs', 18, 'pub fn initial_vector'),
  inventory: code('inventory, the weighted sum', 'src/numerical/inventory.rs', 32, 'pub fn inventory'),
  accumulator: code('Accumulator, the cumulative release', 'src/numerical/inventory.rs', 72, 'pub struct Accumulator'),
  fraction: code('release_fraction', 'src/numerical/inventory.rs', 98, 'pub fn release_fraction'),
  residual: code('balance_residual', 'src/numerical/inventory.rs', 104, 'pub fn balance_residual'),
  series: code('check_series, the checks on a whole run', 'src/numerical/diagnostics.rs', 140, 'pub fn check_series'),
  balanceTest: code('balance identity, every step, to rounding', 'tests/btcs_verification.rs', 856, 'fn balance_identity_to_rounding_every_step'),
};
const PS = (n: string, label: string): DocRef & { label: string } => ({ doc: 'd3', section: n, label });
const PSEUDO = {
  grid: PS('PS-02 `Grid::build` : grid generation', 'PS-02, grid generation'),
  coefficients: PS('PS-04 `Coefficients::new` : geometric coefficients', 'PS-04, geometric coefficients'),
  assemble: PS('PS-05 `assemble` : matrix assembly', 'PS-05, matrix assembly'),
  centre: PS('PS-06 `centre_row` : centre-node treatment', 'PS-06, centre-node treatment'),
  interface: PS('PS-07 `interface_row_explicit` and `interior_row_explicit` : interface-node and interior-node check forms', 'PS-07, interface-node and interior-node rows'),
  outer: PS('PS-08 `outer_boundary` : outer boundary', 'PS-08, outer boundary'),
  initial: PS('PS-09 `initial_vector` : initial condition', 'PS-09, initial condition'),
  thomas: PS('PS-10 `ThomasFactors` : tridiagonal solution', 'PS-10, tridiagonal solution'),
  step: PS('PS-11 `Stepper::step` : one time step', 'PS-11, one time step'),
  inventory: PS('PS-12 `inventory` : inventory accounting', 'PS-12, inventory accounting'),
  release: PS('PS-13 `release_rate` and `Accumulator` : release calculation', 'PS-13, release calculation'),
  residual: PS('PS-14 `balance_residual` : conservation residual', 'PS-14, conservation residual'),
  flux: PS('PS-15 `interface_flux_both_sides` : interface and centre diagnostics', 'PS-15, interface and centre diagnostics'),
  run: PS('PS-16 `run` : time stepping', 'PS-16, time stepping'),
  diagnostics: PS('PS-17 `diagnostics` : verification diagnostics', 'PS-17, verification diagnostics'),
  hooks: PS('PS-19 Hooks for generation, decay and temperature-dependent diffusion (Phase G)', 'PS-19, hooks for generation, decay and temperature'),
};
const TRACE: DocRef = { doc: 'd4', section: '2. Matrix of equations (D2 numbering)' };

const NO_CODE = 'This step states the continuous mathematics that every later step discretises; no solver code belongs to it.';
const NO_LIVE_MATH = 'This step states mathematics; it has no live value. The numbers of the current run are in the live simulation below.';

// ---------------------------------------------------------------------------
// The steps
// ---------------------------------------------------------------------------

export const STEP_DEFS: Record<Stage, StepDef> = {
  1: {
    stage: 1,
    name: 'Conservation of mass',
    short: 'Mass balance',
    kind: 'step',
    mechanism: null,
    summary: 'The amount of the species in a thin spherical shell changes only by what flows through its two surfaces. Shrinking the shell gives the local balance.',
    main: [D1_EQ1],
    parts: [
      part({
        id: 'shell_amount',
        name: 'Amount in the shell',
        symbol: 'C(s,t)',
        latex: String.raw`\frac{d}{dt}\int_r^{r+\delta r} C(s,t)\,4\pi\left(s\right)^2\,ds`,
        meaning: 'How fast the total amount of the species inside the shell between two radii changes: the concentration times the volume element of a sphere, added over the shell.',
        units: [U_C, U_R, U_T],
        assumptions: ['No source and no sink inside the shell: the amount changes only by flow through the two surfaces.', 'Spherical symmetry: the concentration depends on the radius and the time only.'],
        derivation: doc1(D1_1),
      }),
      part({
        id: 'flow_in',
        name: 'Flow in through the inner surface',
        symbol: String.raw`\Phi(r,t)`,
        latex: String.raw`\Phi(r,t)`,
        meaning: 'The outward flow through the sphere of radius r. Entering the shell through its inner surface, it adds to the amount in the shell.',
        units: [u(String.raw`\Phi(r,t) = 4\pi\left(r\right)^2 J(r,t)`, 'outward flow through the sphere of radius r', String.raw`\mathrm{mol\ s^{-1}}`)],
        derivation: doc1(D1_1),
      }),
      part({
        id: 'flow_out',
        name: 'Flow out through the outer surface',
        symbol: String.raw`\Phi(r+\delta r,t)`,
        latex: String.raw`\Phi(r+\delta r,t)`,
        meaning: 'The outward flow through the outer surface of the shell. Leaving the shell, it takes from the amount in the shell.',
        derivation: doc1(D1_1),
      }),
      part({
        id: 'local_balance',
        name: 'Local balance',
        symbol: String.raw`\Phi`,
        latex: '',
        meaning: 'When the shell shrinks to a radius, the difference of the two flows divided by the thickness becomes a derivative: the rate of change of the concentration, weighted by the sphere area, equals minus the change of the flow with the radius.',
        units: [u('J(r,t)', 'radial diffusive flux, outward positive', String.raw`\mathrm{mol\ m^{-2}\ s^{-1}}`)],
        derivation: doc1(D1_1),
      }),
    ],
    implementation: { equations: [D1_EQ2], code: [], pseudocode: [], derivation: [doc1(D1_1)], note: NO_CODE },
    live: NO_LIVE_MATH,
  },
  2: {
    stage: 2,
    name: "Fick's law",
    short: "Fick's law",
    kind: 'step',
    mechanism: null,
    summary: 'The flux runs down the concentration gradient, at a rate set by the diffusivity. The diffusivity stays inside the derivative later, so that it may jump between materials.',
    main: [D1_EQ3],
    parts: [
      part({
        id: 'flux',
        name: 'Diffusive flux',
        symbol: 'J',
        latex: 'J',
        meaning: 'The amount of the species crossing a unit area of a sphere per unit time, positive when it flows outward.',
        units: [u('J(r,t)', 'radial diffusive flux, outward positive', String.raw`\mathrm{mol\ m^{-2}\ s^{-1}}`)],
        derivation: doc1(D1_1),
      }),
      part({
        id: 'diffusivity',
        name: 'Diffusivity',
        symbol: 'D(r)',
        latex: 'D(r)',
        meaning: 'How easily the species moves through the material at radius r. It is constant inside one material and can differ between materials by orders of magnitude.',
        units: [U_D, U_DM],
        assumptions: ['Fickian diffusion: the flux is proportional to the gradient.', 'One constant diffusivity in each material.'],
        derivation: doc1(D1_1),
      }),
      part({
        id: 'gradient',
        name: 'Concentration gradient',
        symbol: String.raw`\partial C/\partial r`,
        latex: String.raw`\frac{\partial C}{\partial r}`,
        meaning: 'How steeply the concentration changes with the radius. The minus sign makes the flux run from high to low concentration.',
        units: [U_C, U_R],
        derivation: doc1(D1_1),
      }),
    ],
    implementation: { equations: [], code: [], pseudocode: [], derivation: [doc1(D1_1)], note: NO_CODE },
    live: NO_LIVE_MATH,
  },
  3: {
    stage: 3,
    name: 'Spherical diffusion equation',
    short: 'Spherical eq.',
    kind: 'step',
    mechanism: null,
    summary: 'Fick’s law put into the local balance gives the equation that the solver discretises. Inside one material, where the diffusivity is constant, it takes the familiar second-derivative form.',
    main: [D1_EQ4],
    parts: [
      part({
        id: 'rate',
        name: 'Rate of change of concentration',
        symbol: String.raw`\partial C/\partial t`,
        latex: String.raw`\frac{\partial C}{\partial t}`,
        meaning: 'How fast the concentration at one radius changes with time. Every term on the right is a rate in the same unit, so the equation is a balance.',
        units: [U_C, U_T],
        derivation: doc1(D1_1),
      }),
      part({
        id: 'diffusion',
        name: 'Diffusion in a sphere',
        symbol: 'D(r)',
        latex: String.raw`\frac{1}{\left(r\right)^2}\frac{\partial}{\partial r}\left[\left(r\right)^2 D(r)\,\frac{\partial C}{\partial r}\right]`,
        meaning: 'The net inflow by diffusion into a thin shell, per unit volume. The factor that grows with the square of the radius follows the area of the shell; the diffusivity stays inside the derivative so that it may jump at an interface.',
        units: [U_R, U_D],
        assumptions: ['Spherical symmetry.', 'Valid where the diffusivity is smooth. At an interface it is not applied as written: step eight builds the interface rows from fluxes.'],
        derivation: doc1(D1_1),
      }),
      part({
        id: 'curvature',
        name: 'Curvature term',
        symbol: String.raw`\frac{\partial^2 C}{\partial r^2}`,
        latex: '',
        meaning: 'Inside one material the diffusion term splits into a curvature part, which smooths the profile as in a flat slab.',
        units: [U_R],
        derivation: doc1(D1_2),
      }),
      part({
        id: 'geometry',
        name: 'Geometry term',
        symbol: String.raw`\frac{2}{r}`,
        latex: '',
        meaning: 'The part that exists only because the shells grow with the radius. It is the reason that the matrix coefficients depend on the radius of the node.',
        units: [U_R],
        derivation: doc1(D1_2),
      }),
    ],
    implementation: { equations: [D1_EQ5], code: [], pseudocode: [], derivation: [doc1(D1_1), doc1(D1_2)], note: NO_CODE },
    live: NO_LIVE_MATH,
  },
  4: {
    stage: 4,
    name: 'Five-layer material model',
    short: 'Five layers',
    kind: 'step',
    mechanism: null,
    summary: 'The particle has five concentric materials, each with its own constant diffusivity. In perfect contact the concentration and the flux are continuous at the four interfaces.',
    main: [D1_EQ27],
    parts: [
      part({
        id: 'materials',
        name: 'Diffusivity of each material',
        symbol: String.raw`D_{\mathrm{kernel}}`,
        latex: String.raw`D_{\mathrm{kernel}}`,
        meaning: 'Kernel, buffer, inner pyrocarbon, silicon carbide and outer pyrocarbon each have one constant diffusivity. Materials are named, never numbered by the node index.',
        units: [U_DM, u(String.raw`R_1, R_2, R_3, R_4, R_5`, 'outer radii of kernel, buffer, IPyC, SiC and OPyC; R_0 = 0', String.raw`\mathrm{m}`)],
        assumptions: ['The diffusivity is constant in each material.', 'The temperature is constant; temperature dependence is a labelled preview.'],
        derivation: doc1('0. Scope, physical model and notation'),
      }),
      part({
        id: 'continuity',
        name: 'Concentration continuous',
        symbol: String.raw`C(R_1^-, t)`,
        latex: '',
        meaning: 'Perfect contact: the concentration has the same value on both sides of each interface. This is a modelling assumption with a partition coefficient of one.',
        units: [U_C],
        assumptions: ['No partition coefficient: an assumption about contact, not a consequence of conservation.'],
        derivation: doc1('5.1 Continuity of concentration (assumption)'),
      }),
      part({
        id: 'flux_continuity',
        name: 'Flux continuous',
        symbol: String.raw`D_{\mathrm{buffer}}`,
        latex: String.raw`D_{\mathrm{buffer}}\left.\frac{\partial C}{\partial r}\right|_{R_1^+}`,
        meaning: 'What flows out of one material flows into the next. Because the diffusivities differ, the slope of the concentration jumps at the interface in the inverse ratio.',
        units: [U_D],
        assumptions: ['Derived from conservation applied to a thin shell that straddles the interface.'],
        derivation: doc1('5.2 Continuity of diffusive flux (derived)'),
      }),
    ],
    implementation: { equations: [], code: [], pseudocode: [], derivation: [doc1('0. Scope, physical model and notation'), doc1(D1_5)], note: NO_CODE },
    live: 'The simulation below uses the synthetic five-layer verification case; its radii and diffusivities are chosen to test the solver and are not measured TRISO properties.',
  },
  5: {
    stage: 5,
    name: 'Central spatial differences',
    short: 'Differences',
    kind: 'step',
    mechanism: null,
    summary: 'On the grid of nodes, the first and second derivative are replaced by central differences from Taylor’s theorem. Both are second-order accurate.',
    main: [D1_EQ9],
    parts: [
      part({
        id: 'second_diff',
        name: 'Second difference',
        symbol: String.raw`\left(\Delta r\right)^2`,
        latex: String.raw`\frac{C_{i+1} - 2C_i + C_{i-1}}{\left(\Delta r\right)^2}`,
        meaning: 'The curvature of the profile at node i, from the node itself and its two neighbours.',
        units: [U_I, U_RI, u(String.raw`\Delta r`, 'uniform spacing, where the grid is uniform', String.raw`\mathrm{m}`)],
        derivation: doc1(D1_32),
      }),
      part({
        id: 'first_diff',
        name: 'First difference',
        symbol: String.raw`2\,\Delta r`,
        latex: '',
        meaning: 'The slope of the profile at node i, from its two neighbours.',
        units: [U_I],
        derivation: doc1(D1_32),
      }),
      part({
        id: 'truncation',
        name: 'Error of the differences',
        symbol: String.raw`O\left(\left(\Delta r\right)^2\right)`,
        latex: String.raw`O\left(\left(\Delta r\right)^4\right)`,
        meaning: 'What the differences leave out. After the division the leading error is proportional to the square of the spacing: halving the spacing divides it by four.',
        derivation: doc1(D1_32),
      }),
    ],
    implementation: {
      equations: [D1_EQ10, F6, F7],
      code: [CODE.fromRadii, CODE.coefficients],
      pseudocode: [PSEUDO.grid, PSEUDO.coefficients],
      derivation: [doc1(D1_32), doc2(D2_3), doc2(D2_4)],
      note: 'Inside a material the central differences equal a difference of flows through the intervals between nodes. The solver uses this flux form at every node, so that it conserves the amount exactly and needs no special stencil at an interface.',
    },
    live: 'The grid of the current run is set by the number of intervals per material in the controls; the solver reports the nodes it used.',
  },
  6: {
    stage: 6,
    name: 'Backward Euler discretisation',
    short: 'Backward Euler',
    kind: 'step',
    mechanism: null,
    summary: 'Every spatial value on the right-hand side is taken at the new time level. Each new value is coupled to its two new neighbours, so all rows are solved together, and no limit on the time step is needed.',
    main: [D1_EQ12],
    parts: [
      part({
        id: 'time_quotient',
        name: 'Time difference',
        symbol: String.raw`\Delta t`,
        latex: String.raw`\frac{C_i^{j+1} - C_i^j}{\Delta t}`,
        meaning: 'The change of the concentration at node i over one time step, divided by the step. Time superscripts are indices, not powers.',
        units: [U_CIJ, U_DT, U_J],
        derivation: doc1(D1_33),
      }),
      part({
        id: 'second_new',
        name: 'Second difference, new level',
        symbol: String.raw`\left(\Delta r\right)^2`,
        latex: String.raw`\frac{C_{i+1}^{j+1} - 2C_i^{j+1} + C_{i-1}^{j+1}}{\left(\Delta r\right)^2}`,
        meaning: 'The curvature of the profile at node i, from the unknown new values of the node and its two neighbours. Because they are unknown, every row is coupled to its neighbours.',
        units: [U_CIJ, U_I],
        derivation: doc1(D1_34),
      }),
      part({
        id: 'first_new',
        name: 'First difference, new level',
        symbol: String.raw`\frac{2}{r_i}`,
        latex: String.raw`\frac{2}{r_i}\,\frac{C_{i+1}^{j+1} - C_{i-1}^{j+1}}{2\,\Delta r}`,
        meaning: 'The slope of the profile at node i, weighted by the geometry term, also at the new level.',
        units: [U_CIJ, U_RI],
        derivation: doc1(D1_34),
      }),
    ],
    implementation: {
      equations: [D1_EQ11, D1_EQ14, D1_EQ15, F10, F11, F8],
      code: [CODE.assemble, CODE.coefficients],
      pseudocode: [PSEUDO.coefficients, PSEUDO.assemble],
      derivation: [doc1(D1_33), doc1(D1_34), doc2(D2_5)],
      note: 'The general row is the flux balance of the node over the new time step, written once for every interior and interface node. The explicit coefficient forms are checks: they equal it to rounding.',
    },
    live: 'The time step and the number of steps of the current run are in the controls; the solver reports the step it used.',
  },
  7: {
    stage: 7,
    name: 'Centre-node equation',
    short: 'Centre node',
    kind: 'step',
    mechanism: null,
    summary: 'At the centre the radius is zero, so the operator takes its limit: three times the second derivative. A ghost node equal to the first node gives the centre row.',
    main: [F12, F13],
    parts: [
      part({
        id: 'centre_limit',
        name: 'Limit at the centre',
        symbol: String.raw`C''(0,t)`,
        latex: '',
        meaning: 'As the radius goes to zero, the curvature term and the geometry term together become three times the second derivative.',
        units: [U_C, U_R],
        assumptions: ['The concentration is smooth and an even function of the radius near the centre.'],
        derivation: doc1(D1_4),
      }),
      part({
        id: 'centre_row',
        name: 'Centre row',
        symbol: String.raw`\mathrm{Fo}_{\mathrm{kernel}}`,
        latex: String.raw`\left(1 + 6\,\mathrm{Fo}_{\mathrm{kernel}}\right)C_0^{j+1} - 6\,\mathrm{Fo}_{\mathrm{kernel}}\,C_1^{j+1}`,
        meaning: 'The implicit row at node zero. The factor is three from the limit times two from the ghost node, with the kernel diffusivity.',
        units: [U_FO],
        assumptions: ['Only the kernel diffusivity enters: the centre lies inside the kernel.'],
        derivation: doc1(D1_42),
      }),
      part({
        id: 'one_way',
        name: 'One-way coupling',
        symbol: 'L_1 = 0',
        latex: 'L_1 = 0',
        meaning: 'The row of node one has no term for node zero. The centre value is driven by node one and sends nothing back, so the centre does not enter the inventory or the release.',
        units: [U_LBU],
        derivation: doc1('4.3 The centre value does not feed back (coordinator\'s hypothesis 2)'),
      }),
    ],
    implementation: {
      equations: [D1_EQ23, D1_EQ24, D1_EQ22],
      code: [CODE.centre, CODE.assemble],
      pseudocode: [PSEUDO.centre, PSEUDO.assemble],
      derivation: [doc1(D1_4), doc1(D1_42), doc2(D2_5)],
      note: 'The centre value is reported from this row. It influences no other row and no sum, because the weight of the centre node is zero.',
    },
    live: 'The concentration at the centre in the particle view is the value of this row.',
  },
  8: {
    stage: 8,
    name: 'Interface-node equations',
    short: 'Interface nodes',
    kind: 'step',
    mechanism: null,
    summary: 'A node sits on each of the four interfaces and carries one value shared by both materials. Its row balances the flow through its inner half-interval against the flow through its outer half-interval, each with its own diffusivity.',
    main: [F10],
    parts: [
      part({
        id: 'storage',
        name: 'Storage at the node',
        symbol: String.raw`V_i`,
        latex: String.raw`V_i\,\frac{C_i^{j+1} - C_i^j}{\Delta t}`,
        meaning: 'The change of the amount held by the node: half of each adjacent interval, so the inner and the outer material contribute their own half.',
        units: [U_V, U_CIJ],
        derivation: doc1('5.3 The interface row: a node on the interface, with a flux balance over its two half-intervals'),
      }),
      part({
        id: 'flow_out_interval',
        name: 'Flow through the outer interval',
        symbol: String.raw`K_{i,\mathrm{outer}}`,
        latex: String.raw`K_{i,\mathrm{outer}}\left(C_{i+1}^{j+1} - C_i^{j+1}\right)`,
        meaning: 'The flow between the node and its outer neighbour, with the diffusivity of the outer material only.',
        units: [U_K],
        assumptions: ['No diffusivity is averaged, and no constant-diffusivity stencil is applied across the jump.'],
        derivation: doc1('5.3 The interface row: a node on the interface, with a flux balance over its two half-intervals'),
      }),
      part({
        id: 'flow_in_interval',
        name: 'Flow through the inner interval',
        symbol: String.raw`K_{i,\mathrm{inner}}`,
        latex: String.raw`K_{i,\mathrm{inner}}\left(C_i^{j+1} - C_{i-1}^{j+1}\right)`,
        meaning: 'The flow between the inner neighbour and the node, with the diffusivity of the inner material only.',
        units: [U_K],
        derivation: doc1('5.3 The interface row: a node on the interface, with a flux balance over its two half-intervals'),
      }),
    ],
    implementation: {
      equations: [F16, F16A, F31],
      code: [CODE.assemble, CODE.interfaceRow, CODE.interfaceFlux],
      pseudocode: [PSEUDO.assemble, PSEUDO.interface, PSEUDO.flux],
      derivation: [doc1(D1_5), doc2(D2_5)],
      note: 'The four interface rows are the general row at the four interface nodes. The two expressions for the flow through an interface are equal in exact arithmetic; their difference on a computed solution is a diagnostic.',
    },
    live: 'The interface concentration and the flow through each interface are diagnostics of a run; they are listed with the diagnostics under the live simulation once the solver reports them.',
  },
  9: {
    stage: 9,
    name: 'Outer-boundary equation',
    short: 'Outer boundary',
    kind: 'step',
    mechanism: null,
    summary: 'The outer surface is absorbing: the concentration there is zero, so the last node needs no row. The release rate is the flow through the last interval.',
    main: [F18, F26],
    parts: [
      part({
        id: 'absorbing',
        name: 'Absorbing surface',
        symbol: String.raw`C_N^j = 0`,
        latex: String.raw`C_N^j = 0`,
        meaning: 'The concentration at the outer node is fixed at zero at every time step. The node has no row and no ghost node.',
        units: [U_I, U_J],
        assumptions: ['A perfect sink: released atoms never return. The insulated outer row of the original derivation is not used.'],
        derivation: doc1(D1_6),
      }),
      part({
        id: 'last_row',
        name: 'Row next to the surface',
        symbol: String.raw`B_{N-1}`,
        latex: '',
        meaning: 'The boundary term leaves the matrix because its value is zero, but the diagonal keeps its full value: the coupling to the surface stays in the diagonal.',
        units: [U_LBU],
        derivation: doc2(D2_5),
      }),
      part({
        id: 'release_rate',
        name: 'Release rate',
        symbol: String.raw`\Phi^{j+1}`,
        latex: String.raw`\Phi^{j+1} = 4\pi\,K_{N-1,\mathrm{outer}}\,C_{N-1}^{j+1}`,
        meaning: 'The flow through the last interval, at the new time level, with the stored conductance of that interval: the same number that sits in the last row. It is never negative.',
        units: [U_PHI, U_K],
        derivation: doc1(D1_6),
      }),
    ],
    implementation: {
      equations: [F17],
      code: [CODE.outer, CODE.release, CODE.assemble],
      pseudocode: [PSEUDO.outer, PSEUDO.release],
      derivation: [doc1(D1_6), doc2(D2_5), doc2(D2_7)],
      note: 'The release rate must use the stored conductance of the last interval, not a recomputed one; otherwise the balance of step twelve acquires a residual of discretisation size.',
    },
    live: 'The release rate of the current run appears in the release readout and the release curve of the live simulation.',
  },
  10: {
    stage: 10,
    name: 'Tridiagonal matrix',
    short: 'Matrix',
    kind: 'step',
    mechanism: null,
    summary: 'The rows of all nodes form one tridiagonal system for the new concentrations. With a constant time step and constant diffusivities the matrix is the same at every step and is assembled once.',
    main: [F19],
    parts: [
      part({
        id: 'row_lower',
        name: 'Lower diagonal',
        symbol: 'L_i',
        latex: '',
        meaning: 'The coupling to the inner neighbour. The first entry of the second row is zero: the centre is one-way.',
        units: [U_LBU],
        derivation: doc2(D2_5),
      }),
      part({
        id: 'row_diag',
        name: 'Diagonal',
        symbol: 'B_i',
        latex: '',
        meaning: 'The diagonal entry. Every diagonal entry exceeds the sum of the sizes of the entries beside it in the matrix by at least one.',
        units: [U_LBU],
        derivation: doc2(D2_8),
      }),
      part({
        id: 'row_upper',
        name: 'Upper diagonal',
        symbol: 'U_i',
        latex: '',
        meaning: 'The coupling to the outer neighbour. The last row has none: its neighbour is the zero boundary value.',
        units: [U_LBU],
        derivation: doc2(D2_5),
      }),
      part({
        id: 'unknowns',
        name: 'Unknown vector',
        symbol: String.raw`\mathbf{C}^{j+1}`,
        latex: String.raw`\mathbf{C}^{j+1} = \begin{pmatrix} C_0^{j+1} \\ C_1^{j+1} \\ \vdots \\ C_{N-1}^{j+1} \end{pmatrix}`,
        meaning: 'The concentrations at the new time level for the nodes from the centre to the node next to the surface. The surface node is not an unknown.',
        units: [U_CIJ],
        derivation: doc2(D2_5),
      }),
    ],
    implementation: {
      equations: [F9, F20, F21, F22],
      code: [CODE.tridiagonal, CODE.assemble, CODE.initial],
      pseudocode: [PSEUDO.assemble, PSEUDO.initial],
      derivation: [doc1(D1_10), doc2(D2_5), doc2(D2_6)],
      note: 'The matrix is assembled again whenever the time step or a diffusivity changes. The right-hand side is the old profile; the start profile is uniform in the kernel with a value at the kernel surface that makes the initial inventory exact.',
    },
    live: 'The size of the system is the number of nodes of the current run; the matrix itself is not shown in the simulation.',
  },
  11: {
    stage: 11,
    name: 'Rust solution algorithm',
    short: 'Rust algorithm',
    kind: 'step',
    mechanism: null,
    summary: 'One forward sweep and one back substitution solve the tridiagonal system at a cost proportional to the number of nodes, with no pivoting. The code follows the form written here.',
    main: [F24],
    parts: [
      part({
        id: 'pivot',
        name: 'Multiplier and pivot',
        symbol: 'm_i',
        latex: String.raw`m_i = B_i - w_i\,U_{i-1}`,
        meaning: 'The forward sweep removes the lower entry of each row. Every pivot is at least one plus the size of the upper entry of its row (the matrix properties of the final formulation), so none is zero, and the back-substitution factors, the size of the upper entry divided by the pivot, are below one.',
        units: [U_LBU],
        assumptions: ['Positive pivots follow from the sign pattern of the matrix; no pivoting is needed.'],
        derivation: doc1(D1_9),
      }),
      part({
        id: 'rhs_update',
        name: 'Right-hand side update',
        symbol: String.raw`b'_i`,
        latex: String.raw`b'_i = b_i^j - w_i\,b'_{i-1}`,
        meaning: 'The same elimination applied to the known old profile.',
        derivation: doc2('6.3 Solution of the linear system'),
      }),
      part({
        id: 'back_substitution',
        name: 'Back substitution',
        symbol: String.raw`C_i^{j+1}`,
        latex: String.raw`C_i^{j+1} = \left(b'_i - U_i\,C_{i+1}^{j+1}\right)/m_i`,
        meaning: 'The new concentrations from the outside inward. With a non-negative old profile every result is non-negative, because the code is written in this form.',
        units: [U_CIJ],
        assumptions: ['Non-negativity in floating point holds for this form of the algorithm, as coded; another form would need its own proof.'],
        derivation: doc2('6.3 Solution of the linear system'),
      }),
    ],
    implementation: {
      equations: [F20],
      code: [CODE.thomas, CODE.thomasNew, CODE.thomasSolve, CODE.step, CODE.run, CODE.crossCheck],
      pseudocode: [PSEUDO.thomas, PSEUDO.step, PSEUDO.run],
      derivation: [doc2('6.3 Solution of the linear system'), TRACE],
      note: 'The links point at the solver branch. The routine that solves each step is ThomasFactors, coded in the form of the equation above. solve_in_place is the general Thomas solve, kept only as a cross-check utility.',
    },
    live: 'The solver of the current run is this algorithm once the website runs the BTCS solver; the engine and version are listed in the diagnostics under the live simulation.',
  },
  12: {
    stage: 12,
    name: 'Inventory and release',
    short: 'Inventory',
    kind: 'step',
    mechanism: null,
    summary: 'The inventory is a weighted sum over the nodes; the release adds the outer flow step by step and is divided by the start inventory.',
    main: [F25, F27],
    parts: [
      part({
        id: 'inventory',
        name: 'Inventory',
        symbol: String.raw`M^j`,
        latex: String.raw`M^j = 4\pi\sum_{i=1}^{N-1}V_i\,C_i^j`,
        meaning: 'The amount of the species still inside the particle: the concentration of each node times its storage weight, added over the nodes.',
        units: [U_M, U_V, U_CIJ],
        derivation: doc2('7.1 Discrete inventory'),
      }),
      part({
        id: 'release_rate_12',
        name: 'Release rate',
        symbol: String.raw`\Phi^{j+1}`,
        latex: '',
        meaning: 'How fast the species leaves through the outer surface during the step.',
        units: [U_PHI, U_K],
        derivation: doc2('7.2 Release rate'),
      }),
      part({
        id: 'released',
        name: 'Cumulative release',
        symbol: String.raw`M_{\mathrm{rel}}^j`,
        latex: String.raw`M_{\mathrm{rel}}^{j+1} = M_{\mathrm{rel}}^j + \Delta t\,\Phi^{j+1}`,
        meaning: 'The total released so far: the rate times the length of the step, added over the steps.',
        units: [U_MREL, U_DT],
        derivation: doc2('7.3 Cumulative release and release fraction'),
      }),
    ],
    implementation: {
      equations: [F26, F28, F29, F30],
      code: [CODE.inventory, CODE.accumulator, CODE.release, CODE.fraction, CODE.residual],
      pseudocode: [PSEUDO.inventory, PSEUDO.release, PSEUDO.residual],
      derivation: [doc2(D2_7), TRACE],
      note: 'The reported release fraction is the released amount over the exact start inventory. The complementary value, one minus the inventory over the start inventory, is a diagnostic that differs from it by the balance residual.',
    },
    live: 'The values of the inventory, the released amount and the balance residual for the current run are under Inventory balance in the live simulation below, as the Rust wrapper returns them; nothing is computed in the browser.',
  },
  13: {
    stage: 13,
    name: 'Numerical verification',
    short: 'Verification',
    kind: 'step',
    mechanism: null,
    summary: 'Does the code solve its own equations? The balance holds to rounding and the error falls at the proved order. No measurement is involved.',
    main: [F29],
    parts: [
      part({
        id: 'identity',
        name: 'Balance identity',
        symbol: String.raw`M^0`,
        latex: String.raw`M^j + M_{\mathrm{rel}}^j = M^0`,
        meaning: 'In exact arithmetic the inventory and the released amount always add up to the start inventory. In floating point it holds to rounding.',
        units: [U_M, U_MREL],
        derivation: doc2(D2_7),
      }),
      part({
        id: 'residual',
        name: 'Balance residual',
        symbol: String.raw`\varepsilon^j`,
        latex: '',
        meaning: 'The amount by which the identity fails in a run. Its tolerance is not fixed by this page: it must come from a rounding analysis of the code, and the observed residual is always reported beside it.',
        units: [U_EPS],
        derivation: doc2(D2_7),
      }),
      part({
        id: 'order_one',
        name: 'Order for one material',
        symbol: String.raw`\left\lVert \mathbf{e}^j\right\rVert _\infty`,
        latex: '',
        meaning: 'First order in time and second order in space, proved for one material with a uniform grid and a smooth solution on the whole time interval from the start to T.',
        derivation: doc2(D2_8),
      }),
      part({
        id: 'order_five',
        name: 'Order for five layers',
        symbol: String.raw`\left\lVert \mathbf{e}^j\right\rVert _V`,
        latex: '',
        meaning: 'The same orders for five layers in a weighted norm, with the constants not tracked (a proof outline), on [t_*, T] only, for a solution four times continuously differentiable in r on each layer and twice in t on [t_*, T], with the error at t_* of order (Δr) squared. The second order of the computed release is observed, not proved.',
        derivation: doc2(D2_8),
      }),
    ],
    implementation: {
      equations: [F30, P13, P14],
      code: [CODE.balanceTest, CODE.series],
      pseudocode: [PSEUDO.diagnostics],
      derivation: [doc2(D2_8), TRACE],
      note: 'Document 2, Section 8 lists each property with its hypotheses and its label: proved, proof outline, observed or not proven. The tests of the solver branch check them. Nothing on this page claims an accuracy against experimental data.',
    },
    live: 'Until the website switch, the Verify section of the site still reports the verification of the previous finite-volume solver, not of the BTCS solver. The checks of the BTCS solver are the tests in tests/btcs_verification.rs on the solver branch. Both verify the numerical method and are not experimental validation.',
  },
  14: {
    stage: 14,
    name: 'Experimental comparisons',
    short: 'Comparison',
    kind: 'step',
    mechanism: null,
    summary: 'Placeholder. The comparison of the computed release fraction with measured release is not part of this build: the comparison data and their metrics do not exist on this site yet.',
    main: [F28],
    parts: [
      part({
        id: 'model_fraction',
        name: 'Computed release fraction',
        symbol: String.raw`F^j`,
        latex: String.raw`F^j = \frac{M_{\mathrm{rel}}^j}{M^0}`,
        meaning: 'The quantity of the model that a comparison will set beside a measured release fraction. A measured value is fixed data; the computed value changes with the inputs.',
        units: [U_F, U_MREL, U_M],
        assumptions: ['Placeholder: no measurement is shown on this page and no agreement is claimed.', 'A comparison needs the data, the diffusivity rows and the assumptions stated beside every result; none of them is shown yet.'],
        derivation: doc2(D2_7),
      }),
    ],
    implementation: {
      equations: [],
      code: [],
      pseudocode: [],
      derivation: [doc2(D2_7)],
      note: 'Placeholder: no code of the comparison is linked until the comparison data and their metrics exist.',
    },
    live: 'Placeholder: the comparison with measured release is described under Validate; no measured data are shown here.',
  },
  15: {
    stage: 15,
    name: 'Fission-product generation (preview)',
    short: 'Generation',
    kind: 'preview',
    mechanism: 'generation',
    summary: 'Preview only. A volumetric source would enter the right-hand side and the balance. The term is a placeholder: it is not derived, and the solver does not implement it.',
    main: [G1_RHS],
    parts: [
      part({
        id: 'source',
        name: 'Fission-product generation',
        symbol: 'G',
        latex: String.raw`\left[\text{G1 source term}\right]`,
        highlight: 'source',
        mechanism: 'generation',
        meaning: 'Atoms of the species born inside the particle, per unit volume and time. The documents fix only where it would enter: the right-hand side and the balance. Its derivation, its weighting at interface and centre nodes and its effect on the release formula are still to be done.',
        units: [u('G', 'volumetric source', String.raw`\mathrm{mol\ m^{-3}\ s^{-1}}`), u(String.raw`M_{\mathrm{gen}}^j`, 'generated amount', String.raw`\mathrm{mol}`)],
        assumptions: ['No source value is given by this page. The fission-rate density is not in the parameter register (register gap GAP-28); until it is, a source would be a plain user input with no default presented as a measurement.'],
        derivation: doc2(D2_9),
      }),
    ],
    implementation: { equations: [], code: [], pseudocode: [PSEUDO.hooks], derivation: [doc2(D2_9)], note: 'Preview, not implemented: no solver code exists for this mechanism.' },
    live: 'Not offered: this mechanism is not solved. The switch in the controls stays off.',
  },
  16: {
    stage: 16,
    name: 'Radioactive decay (preview)',
    short: 'Decay',
    kind: 'preview',
    mechanism: 'decay',
    summary: 'Preview only. A decay constant would add to the diagonal of every row and the lost amount would be booked apart from the release. The term is a placeholder: it is not derived, and the solver does not implement it.',
    main: [G2_DIAG],
    parts: [
      part({
        id: 'decay',
        name: 'Radioactive decay',
        symbol: String.raw`\lambda`,
        latex: String.raw`\left[\text{G2 decay term}\right]`,
        highlight: 'decay',
        mechanism: 'decay',
        meaning: 'Loss of the species by radioactive decay, with a decay constant. The documents fix only where it would enter: the diagonal of the matrix and the balance, with a decayed amount booked apart from the physical release.',
        units: [u(String.raw`\lambda`, 'decay constant', String.raw`\mathrm{s^{-1}}`), u(String.raw`M_{\mathrm{dec}}^j`, 'decayed amount', String.raw`\mathrm{mol}`)],
        assumptions: ['Not derived in the documents of this build.'],
        derivation: doc2(D2_9),
      }),
    ],
    implementation: { equations: [], code: [], pseudocode: [PSEUDO.hooks], derivation: [doc2(D2_9)], note: 'Preview, not implemented: no solver code exists for this mechanism.' },
    live: 'Not offered: this mechanism is not solved. The switch in the controls stays off.',
  },
  17: {
    stage: 17,
    name: 'Temperature-dependent diffusivity (preview)',
    short: 'Temperature',
    kind: 'preview',
    mechanism: 'diffusivity_temperature',
    summary: 'Preview only. Each diffusivity would depend on the temperature. The law is not derived, and the solver does not implement it.',
    main: [G3_D],
    parts: [
      part({
        id: 'diffusion_T',
        name: 'Temperature-dependent diffusivity',
        symbol: String.raw`D_{\mathrm{material}}(T)`,
        latex: String.raw`D_{\mathrm{material}}(T)`,
        highlight: 'arrhenius',
        mechanism: 'diffusivity_temperature',
        meaning: 'The diffusivity of each material would follow the temperature. The documents fix only that the conductances, the Fourier numbers, the rows and the release use the same values and that the matrix is assembled again when the temperature changes.',
        units: [u(String.raw`D_{\mathrm{material}}(T)`, 'diffusivity of the material at temperature T', String.raw`\mathrm{m^2\ s^{-1}}`)],
        assumptions: ['The time level at which the temperature is evaluated in a step is still to be fixed.'],
        derivation: doc2(D2_9),
      }),
    ],
    implementation: { equations: [], code: [], pseudocode: [PSEUDO.hooks], derivation: [doc2(D2_9)], note: 'Preview, not implemented: no solver code exists for this mechanism.' },
    live: 'Not offered: this mechanism is not solved. The switch in the controls stays off.',
  },
};

// ---------------------------------------------------------------------------
// Registry and helpers
// ---------------------------------------------------------------------------

/** Every part by id (the ids are unique over all steps). */
export const TERMS: Record<TermId, PartDef & { stage: Stage }> = Object.fromEntries(
  STAGES.flatMap((s) => STEP_DEFS[s].parts.map((p) => [p.id, { ...p, stage: s }] as const)),
);

/** One part by id; throws for an unknown id (a programming error, caught by the tests). */
export function termOf(id: TermId): PartDef & { stage: Stage } {
  const p = TERMS[id];
  if (!p) throw new Error('unknown part ' + id);
  return p;
}

/** The mechanism switches of block C, in the order of the previews (generation, decay, temperature). */
export const SWITCHES: { id: SwitchId; term: TermId; stage: Stage; latex: string; line: 'dashed' | 'solid' | 'dotted' }[] = [
  { id: 'source', term: 'source', stage: 15, latex: 'G', line: 'dashed' },
  { id: 'decay', term: 'decay', stage: 16, latex: String.raw`\lambda`, line: 'solid' },
  { id: 'arrhenius', term: 'diffusion_T', stage: 17, latex: String.raw`D_{\mathrm{material}}(T)`, line: 'dotted' },
];

export function stageDef(stage: Stage): StepDef {
  return STEP_DEFS[stage];
}

/** The parts of a step, as ids. */
export function stageTerms(stage: Stage): TermId[] {
  return STEP_DEFS[stage].parts.map((p) => p.id);
}

/** The switch that a preview step stands for, if any (keeps the step picker and the switches in step). */
export function switchIntroducedBy(stage: Stage): SwitchId | null {
  return SWITCHES.find((s) => s.stage === stage)?.id ?? null;
}

const key = (e: ShownEquation) => `${e.source.doc}:${e.source.tag ?? e.source.verbatim}`;

/** Wraps each part's piece of `latex` (its first occurrence) in a highlight; the pieces must not overlap. */
export function markParts(latex: string, parts: readonly PartDef[]): string {
  const spans = parts
    .filter((p) => p.latex !== '')
    .map((p) => ({ p, at: latex.indexOf(p.latex) }))
    .filter((s) => s.at >= 0)
    .sort((a, b) => a.at - b.at);
  let out = '';
  let pos = 0;
  for (const { p, at } of spans) {
    if (at < pos) throw new Error(`markParts: the piece of part ${p.id} overlaps another piece`);
    const cls = `eq-term eq-term--${p.id}${p.highlight ? ` eq-new eq-new--${p.highlight}` : ''}`;
    out += `${latex.slice(pos, at)}\\htmlClass{${cls}}{${p.latex}}`;
    pos = at + p.latex.length;
  }
  return out + latex.slice(pos);
}

/** The equations of the step shown in the equation box, with the highlights of its parts. */
export function mainEquations(stage: Stage): ShownEquation[] {
  const def = STEP_DEFS[stage];
  return def.main.map((e) => ({ ...e, latex: markParts(e.latex, def.parts) }));
}

/** The number printed beside an equation (a line with no tag in the document may carry a name from its text). */
export function equationNumber(e: ShownEquation): string | null {
  if (e.source.tag) return e.source.doc === 'd1' ? `D1.${e.source.tag}` : e.source.tag;
  return e.numberAs ?? null;
}

const CONTINUOUS_CONDITIONS = [D1_EQ26, D1_EQ27, OUTER_CONT, D1_EQ38];
const DISCRETE_CONDITIONS = [F1, F2, F18, F21, F22];

export function conditionsFor(stage: Stage): { equations: ShownEquation[]; note: string } {
  const def = STEP_DEFS[stage];
  const shown = new Set(def.main.map(key));
  if (def.kind === 'preview') {
    return { equations: [], note: 'A preview changes no condition: the conditions of the base problem stay as stated in the documents.' };
  }
  const base = stage <= 4 ? CONTINUOUS_CONDITIONS : DISCRETE_CONDITIONS;
  const equations = base.filter((e) => !shown.has(key(e)));
  return {
    equations,
    note:
      stage <= 4
        ? 'The conditions of the problem: perfect contact at the four interfaces, an absorbing outer surface and a start that is uniform in the kernel and empty in the coatings.'
        : 'The conditions on the grid. Document 2, Section 8 states the standing hypotheses: every number of intervals is at least one, every diffusivity is positive and finite, and the time step is positive. No condition relates the time step to the spacing.',
  };
}

export function balanceFor(stage: Stage): { equations: ShownEquation[]; note: string } {
  const def = STEP_DEFS[stage];
  const shown = new Set(def.main.map(key));
  if (def.kind === 'preview') {
    return { equations: [G1_BAL].filter((e) => !shown.has(key(e))), note: 'The balance would gain the generated and the decayed amount, each booked apart from the physical release.' };
  }
  if (stage <= 4) {
    return { equations: [], note: 'The balance of the discrete scheme is stated from step five on; at this step the continuous balance is the equation itself.' };
  }
  const equations = [F29, F30].filter((e) => !shown.has(key(e)));
  return {
    equations,
    note: 'The values of each term and the residual for the current run are under “Inventory balance” in the live simulation below, as the Rust wrapper returns them; nothing is computed in the browser.',
  };
}

/** The mechanism-free list of every shown equation of every step, for the tests. */
export function allEquations(): [string, ShownEquation][] {
  return STAGES.flatMap((s) => {
    const def = STEP_DEFS[s];
    return [
      ...def.main.map((e) => [`step ${s} main ${e.id}`, e] as [string, ShownEquation]),
      ...def.implementation.equations.map((e) => [`step ${s} implementation ${e.id}`, e] as [string, ShownEquation]),
      ...conditionsFor(s).equations.map((e) => [`step ${s} condition ${e.id}`, e] as [string, ShownEquation]),
      ...balanceFor(s).equations.map((e) => [`step ${s} balance ${e.id}`, e] as [string, ShownEquation]),
    ];
  });
}
