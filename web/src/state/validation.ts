// Input checks shown next to the controls. They mirror the rejection rules of
// strato::config::Config::validate (src/config.rs lines 79 to 113 at v1.0.1) and the serde types
// (cells and output_every_steps are usize), plus the run-time rule of finite_volume::run that the
// start inventory must be positive (src/numerical/finite_volume.rs line 187), which the UI applies
// as c0 > 0 (prompt/TECHNICAL_ARCHITECTURE.md Section 3, wrapper facts table).
// These are input checks, not physics. The Rust wrapper repeats every rule (web phase 3) and its
// answer is the one that counts.
import type { V1Config } from '../data/case';

export interface InputIssue {
  /** Field path, for example "regions.3.diffusivity_m2_s". */
  path: string;
  message: string;
}

const isPositiveFinite = (v: number) => Number.isFinite(v) && v > 0;
const isCount = (v: number) => Number.isSafeInteger(v) && v >= 1;

export function validateInputs(cfg: V1Config): InputIssue[] {
  const issues: InputIssue[] = [];
  if (cfg.regions.length !== 5) {
    issues.push({ path: 'regions', message: 'Exactly five regions are required in v1.0.' });
  }
  if (cfg.species.trim() === '') {
    issues.push({ path: 'species', message: 'The species name must not be empty.' });
  }
  if (!isPositiveFinite(cfg.temperature_k)) {
    issues.push({ path: 'temperature_k', message: 'Temperature must be a positive finite number of kelvin.' });
  }
  let last = 0;
  cfg.regions.forEach((r, i) => {
    if (!Number.isFinite(r.outer_radius_m) || r.outer_radius_m <= last) {
      issues.push({
        path: `regions.${i}.outer_radius_m`,
        message:
          i === 0
            ? `${r.name}: the outer radius must be a positive finite number.`
            : `${r.name}: the outer radius must be larger than the radius of the layer inside it.`,
      });
    }
    if (!isPositiveFinite(r.diffusivity_m2_s)) {
      issues.push({ path: `regions.${i}.diffusivity_m2_s`, message: `${r.name}: the diffusivity must be a positive finite number.` });
    }
    if (!isCount(r.cells)) {
      issues.push({ path: `regions.${i}.cells`, message: `${r.name}: the cell count must be a whole number of at least 1.` });
    }
    if (Number.isFinite(r.outer_radius_m)) last = r.outer_radius_m;
  });
  if (!isPositiveFinite(cfg.time.dt_s)) {
    issues.push({ path: 'time.dt_s', message: 'The time step must be a positive finite number of seconds.' });
  }
  if (!isPositiveFinite(cfg.time.end_time_s)) {
    issues.push({ path: 'time.end_time_s', message: 'The end time must be a positive finite number of seconds.' });
  }
  if (!isCount(cfg.time.output_every_steps)) {
    issues.push({ path: 'time.output_every_steps', message: 'The output interval must be a whole number of steps, at least 1.' });
  }
  if (!isPositiveFinite(cfg.initial.kernel_concentration_mol_m3)) {
    issues.push({
      path: 'initial.kernel_concentration_mol_m3',
      message: 'The kernel start concentration must be a positive finite number (the solver needs a positive start inventory).',
    });
  }
  return issues;
}

/** Parses a number typed by the user. Accepts plain and scientific notation; rejects anything else. */
export function parseNumber(text: string): number | null {
  const s = text.trim().replace(/\s+/g, '').replace(/[×x]10\^?/i, 'e').replace('−', '-');
  if (!/^[+-]?(\d+\.?\d*|\.\d+)(e[+-]?\d+)?$/i.test(s)) return null;
  const v = Number(s);
  return Number.isFinite(v) ? v : null;
}
