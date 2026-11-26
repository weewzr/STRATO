// Mechanism availability (prompt/TECHNICAL_ARCHITECTURE.md Section 4): a lookup, not physics.
// A mechanism is live only when the Rust wrapper reports it in build_info().capabilities AND its
// research status in the generated catalogue is the live status. Switches, stage live updates and
// solver requests all ask this function; no component decides availability by itself.
import { CATALOGUE, statusOf, type Catalogue, type MechanismRow } from '../research/catalogue';
import { LIVE_STATUS } from '../research/status';
import { LAST_SOLVED_STEP, PREVIEW_STAGES, STEP_DEFS, type Stage } from './terms';

/**
 * Capability id of the BTCS solver (coordinator decision 10: diffusion_btcs_layered). The builder's
 * equations are those of this solver, so the page calls them live only when the wrapper reports it.
 * Until the website switch, the wrapper reports the previous capability only.
 */
export const BTCS_CAPABILITY = 'diffusion_btcs_layered';
/** The catalogue row whose research status must be the live status; the website switch may rename it. */
export const BTCS_CATALOGUE_ROW = 'diffusion_layered';

export function isAvailable(id: string, capabilities: readonly string[], catalogue: Catalogue = CATALOGUE, statusRow: string = id): boolean {
  return capabilities.includes(id) && statusOf(catalogue, statusRow) === LIVE_STATUS;
}

/**
 * Whether one catalogue row (a mechanism for one species and material domain) may drive the live
 * simulation: it names a solver capability, its own status is the live status, and isAvailable()
 * accepts that capability. A row without a solver capability is never live.
 */
export function isRowAvailable(row: MechanismRow, capabilities: readonly string[], catalogue: Catalogue = CATALOGUE): boolean {
  return row.solver_capability !== '' && row.status === LIVE_STATUS && isAvailable(row.solver_capability, capabilities, catalogue);
}

export type SimulationMethod = 'loading' | 'previous' | 'btcs';

/** Which solver the simulation in block B runs, as far as the wrapper's capabilities say. */
export function simulationMethod(capabilities: readonly string[], catalogue: Catalogue = CATALOGUE): SimulationMethod {
  if (capabilities.length === 0) return 'loading';
  return isAvailable(BTCS_CAPABILITY, capabilities, catalogue, BTCS_CATALOGUE_ROW) ? 'btcs' : 'previous';
}

/**
 * The highest builder step whose equations the simulation in block B solves, or 0 when none is:
 * the BTCS steps one to twelve when the wrapper runs the BTCS solver, then each preview whose
 * mechanism is available, in order.
 */
export function solvedStage(capabilities: readonly string[], catalogue: Catalogue = CATALOGUE): Stage | 0 {
  if (simulationMethod(capabilities, catalogue) !== 'btcs') return 0;
  let solved: Stage = LAST_SOLVED_STEP;
  for (const s of PREVIEW_STAGES) {
    const m = STEP_DEFS[s].mechanism;
    if (m !== null && isAvailable(m, capabilities, catalogue)) solved = s;
    else break;
  }
  return solved;
}

/** The mechanism ids that the solver client may send; it drops (and reports) every other id. */
export function filterRequestedMechanisms(
  ids: readonly string[],
  capabilities: readonly string[],
  catalogue: Catalogue = CATALOGUE,
): { send: string[]; dropped: string[] } {
  const send: string[] = [];
  const dropped: string[] = [];
  for (const id of ids) (isAvailable(id, capabilities, catalogue) ? send : dropped).push(id);
  return { send, dropped };
}
