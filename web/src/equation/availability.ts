// Mechanism availability (prompt/TECHNICAL_ARCHITECTURE.md Section 4): a lookup, not physics.
// A mechanism is live only when the Rust wrapper reports it in build_info().capabilities AND its
// research status in the generated catalogue is the live status. Switches, stage live updates and
// solver requests all ask this function; no component decides availability by itself.
import { CATALOGUE, statusOf, type Catalogue, type MechanismRow } from '../research/catalogue';
import { LIVE_STATUS } from '../research/status';
import { STAGE_DEFS, STAGES, type Stage } from './terms';

export function isAvailable(id: string, capabilities: readonly string[], catalogue: Catalogue = CATALOGUE): boolean {
  return capabilities.includes(id) && statusOf(catalogue, id) === LIVE_STATUS;
}

/**
 * Whether one catalogue row (a mechanism for one species and material domain) may drive the live
 * simulation: it names a solver capability, its own status is the live status, and isAvailable()
 * accepts that capability. A row without a solver capability is never live.
 */
export function isRowAvailable(row: MechanismRow, capabilities: readonly string[], catalogue: Catalogue = CATALOGUE): boolean {
  return row.solver_capability !== '' && row.status === LIVE_STATUS && isAvailable(row.solver_capability, capabilities, catalogue);
}

/**
 * The highest builder stage whose mechanisms are all available, or 0 when none is (before the wasm
 * module reports its capabilities). The simulation in block B solves this stage; a higher displayed
 * stage is an equation preview only.
 */
export function solvedStage(capabilities: readonly string[], catalogue: Catalogue = CATALOGUE): Stage | 0 {
  let solved: Stage | 0 = 0;
  for (const s of STAGES) {
    const m = STAGE_DEFS[s].mechanism;
    const ok = m === null ? solved === s - 1 && solved > 0 : isAvailable(m, capabilities, catalogue);
    if (!ok) break;
    solved = s;
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
