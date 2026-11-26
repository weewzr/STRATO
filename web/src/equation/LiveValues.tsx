import type { ReactNode } from 'react';
import { METHOD_2 } from '../methods';
import { Sci } from '../panels/Badge';
import { statusOf, useCatalogue } from '../research/catalogue';
import type { RunOk } from '../solver/types';
import { useStrato } from '../state/store';
import { BTCS_CAPABILITY } from './availability';
import type { Stage } from './terms';

// The hooks of the equation builder for numbers of the current run (the "Live values" disclosure of
// a step). Every number is a field of the wrapper's reply, shown as returned with the full physical
// name of the quantity and its unit; this file selects and formats, it computes nothing. A step
// whose quantity the run does not return shows no number.

interface Row {
  id: string;
  label: string;
  value: number | string | boolean;
  unit?: string;
  digits?: number;
}

/** The rows of one step for one run and playback row; empty for a step without a live number. */
export function liveRows(stage: Stage, run: RunOk, row: number): Row[] {
  const d = run.diagnostics;
  const g = run.grid;
  const names = g.layer_names;
  const profile = run.profiles[row] ?? run.profiles[run.profiles.length - 1];
  switch (stage) {
    case 4:
      return run.inputs.regions.map((r, i) => ({
        id: `live-d-${i}`,
        label: `Diffusivity of the ${r.name} layer`,
        value: r.diffusivity_m2_s,
        unit: 'm²/s',
      }));
    case 5:
      return [
        { id: 'live-nodes', label: 'Grid nodes (dimensionless)', value: g.nodes },
        ...g.spacing_per_layer_m.map((h, i) => ({ id: `live-h-${i}`, label: `Grid spacing in the ${names[i] ?? `layer ${i + 1}`} layer`, value: h, unit: 'm' })),
      ];
    case 6:
      return [
        { id: 'live-dt', label: 'Time step used', value: d.dt_used_s, unit: 's' },
        { id: 'live-steps', label: 'Number of time steps (dimensionless)', value: d.steps },
        ...d.fourier_numbers.map((f, i) => ({
          id: `live-fo-${i}`,
          label: `Mesh Fourier number of the ${names[i] ?? `layer ${i + 1}`} layer (dimensionless)`,
          value: f,
        })),
      ];
    case 7:
      return profile
        ? [
            { id: 'live-centre', label: 'Concentration at the centre node at the playback time', value: profile.c_mol_per_m3[0] ?? Number.NaN, unit: 'mol/m³' },
            { id: 'live-theta', label: 'Start value of the kernel-surface node as a fraction of the initial kernel concentration (dimensionless)', value: d.kernel_surface_start_fraction },
          ]
        : [];
    case 8: {
      const flux = profile?.interface_flux ?? null;
      if (!flux) return [{ id: 'live-flux-none', label: 'Flow through the interfaces', value: 'not defined at time zero' }];
      return [
        ...flux.difference_mol_s.map((x, i) => ({
          id: `live-flux-${i}`,
          label: `Difference between the two flow expressions at the interface at radius ${String(flux.radii_m[i])} m`,
          value: x,
          unit: 'mol/s',
        })),
        { id: 'live-flux-ok', label: 'All differences within their rounding bound', value: flux.within_bound },
      ];
    }
    case 9: {
      const last = run.release_rate_mol_s[run.release_rate_mol_s.length - 1];
      return [
        { id: 'live-release', label: 'Release fraction at the end of the run (dimensionless)', value: d.final_release_fraction },
        ...(last === null || last === undefined ? [] : [{ id: 'live-rate', label: 'Release rate at the end of the run', value: last, unit: 'mol/s' }]),
      ];
    }
    case 10:
      return d.matrix
        ? [
            { id: 'live-margin', label: 'Smallest row margin of the matrix (dimensionless)', value: d.matrix.min_row_margin },
            { id: 'live-norm', label: 'Infinity norm of the matrix (dimensionless)', value: d.matrix.norm_inf },
          ]
        : [];
    case 11:
      return [
        { id: 'live-solver', label: 'Solver', value: run.solver.method },
        { id: 'live-version', label: 'Solver version', value: run.solver.version },
        { id: 'live-commit', label: 'Solver build commit', value: run.solver.commit },
        { id: 'live-label', label: 'Result label', value: run.label },
      ];
    case 12:
      return [
        { id: 'live-remaining', label: 'Remaining inventory at the end of the run', value: run.balance.final.remaining_mol, unit: 'mol' },
        { id: 'live-released', label: 'Cumulative release at the end of the run', value: run.balance.final.released_mol, unit: 'mol' },
        { id: 'live-residual', label: 'Mass-balance residual at the end of the run', value: run.balance.final.residual_mol, unit: 'mol', digits: 3 },
        { id: 'live-bound', label: 'Derived rounding bound of the residual at the end of the run', value: run.balance.final.residual_bound_mol, unit: 'mol', digits: 3 },
      ];
    case 13:
      return [
        { id: 'live-within', label: 'Every residual within its rounding bound', value: run.balance.tolerance.within_bound },
        ...(d.analytical_comparison
          ? [
              { id: 'live-an-max', label: `Largest absolute difference of the release fraction from ${METHOD_2} (dimensionless)`, value: d.analytical_comparison.max_abs_difference, digits: 4 },
              { id: 'live-an-final', label: `Absolute difference of the release fraction from ${METHOD_2} at the end of the run (dimensionless)`, value: d.analytical_comparison.final_abs_difference, digits: 4 },
            ]
          : []),
      ];
    default:
      return [];
  }
}

function show(v: Row['value'], unit: string | undefined, digits: number | undefined): ReactNode {
  if (typeof v === 'number') return Number.isFinite(v) ? <Sci value={v} unit={unit} {...(digits === undefined ? {} : { digits })} /> : 'not returned';
  if (typeof v === 'boolean') return v ? 'yes' : 'no';
  return v;
}

/** The numbers of the current run for one builder step, or a note that there is no run yet. */
export function LiveValues({ stage }: { stage: Stage }) {
  const run = useStrato((s) => s.run);
  const row = useStrato((s) => s.playback.tIndex);
  const catalogue = useCatalogue();
  if (!run) {
    return (
      <p className="more__note" data-testid="live-numbers-none">
        No numbers yet: they appear when the Rust solver has run the current inputs.
      </p>
    );
  }
  const rows = liveRows(stage, run, row);
  const status = stage === 13 ? statusOf(catalogue, BTCS_CAPABILITY) : undefined;
  if (rows.length === 0 && !status) return null;
  return (
    <dl className="diag" data-testid="live-numbers">
      {rows.map((r) => (
        <div className="diag__row" key={r.id}>
          <dt>{r.unit ? `${r.label} (${r.unit})` : r.label}</dt>
          <dd data-testid={r.id} data-value={String(r.value)}>
            {show(r.value, r.unit, r.digits)}
          </dd>
        </div>
      ))}
      {status ? (
        <div className="diag__row">
          <dt>Verification status of the solver in the research catalogue</dt>
          <dd data-testid="live-status">{status}</dd>
        </div>
      ) : null}
    </dl>
  );
}
