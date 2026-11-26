import type { ReactNode } from 'react';
import { BUILD } from '../build';
import { BTCS_CAPABILITY } from '../equation/availability';
import { layerColour } from '../design/layers';
import { count, sciText } from '../format';
import { METHOD_1, METHOD_2 } from '../methods';
import { statusOf, useCatalogue } from '../research/catalogue';
import { StatusChip } from '../research/StatusChip';
import type { RunResponse } from '../solver/types';
import { inputKey, useStrato } from '../state/store';
import { TableBox } from '../sections/common';
import { Badge, Sci } from './Badge';

// Block B results of the BTCS solver: the release readout, the inventory balance with its residual
// and the layer inventories, the numerical diagnostics, and the solver version with its verification
// status. Every number is a field of the wrapper's reply, shown as returned (owner decision M4); the
// only operations here are choosing a field and formatting it. Each value carries its result label
// and, for the tests, the exact returned number in data-value. Quantities are named in full with
// their unit (coordinator decisions 30 and 52).

/** A returned number, formatted to `digits` significant digits, with the exact value attached. */
function V({ v, unit, digits = 6, id }: { v: number | null | undefined; unit?: ReactNode; digits?: number; id: string }) {
  if (v === null || v === undefined || !Number.isFinite(v)) {
    return (
      <span className="num muted" data-testid={id} data-value="">
        not returned
      </span>
    );
  }
  return (
    <span data-testid={id} data-value={String(v)} data-label="COMPUTED v2.0" title={`${sciText(v)} (value as returned)`}>
      <Sci value={v} unit={unit} digits={digits} />
    </span>
  );
}

function useCurrentRun(): { run: RunResponse | null; stale: boolean } {
  const run = useStrato((s) => s.run);
  const info = useStrato((s) => s.runInfo);
  const config = useStrato((s) => s.config);
  const status = useStrato((s) => s.status);
  const stale = run !== null && (status === 'running' || info?.inputKey !== inputKey(config));
  return { run, stale };
}

function Updating({ stale }: { stale: boolean }) {
  return stale ? (
    <span className="sim-note" data-testid="result-stale">
      updating for the changed inputs
    </span>
  ) : null;
}

/**
 * The solver that runs the simulation: its method name, version and build, the result label and the
 * verification status read from the research catalogue (never typed here). Shown whenever the
 * module has loaded, with or without a run, so that no result appears without it.
 */
export function SolverStatus({ compact = false }: { compact?: boolean }) {
  const build = useStrato((s) => s.solver.build);
  const state = useStrato((s) => s.solver.state);
  const error = useStrato((s) => s.solver.error);
  const run = useStrato((s) => s.run);
  const catalogue = useCatalogue();
  const testId = compact ? 'solver-status-line' : 'solver-status';
  const cls = `solver-status${compact ? ' solver-status--compact' : ''}`;
  if (state === 'failed') {
    return (
      <p className={`${cls} solver-status--failed`} role="alert" data-testid={testId} data-state="failed">
        Solver: not available. {METHOD_1} did not load, so the page shows no result{compact ? '' : ` (${error ?? 'no reason given'})`}.
      </p>
    );
  }
  if (state !== 'ready' || !build) {
    return (
      <p className={cls} role="status" data-testid={testId} data-state="loading">
        Solver: loading {METHOD_1}.
      </p>
    );
  }
  const solver = run?.solver ?? build.btcs.solver;
  const status = statusOf(catalogue, BTCS_CAPABILITY);
  const chip = status ? <StatusChip status={status} mechanism={BTCS_CAPABILITY} compact /> : 'unknown';
  if (compact) {
    return (
      <p className={cls} data-testid={testId} data-state="ready">
        <strong>{METHOD_1}</strong>, solver version <span data-testid="solver-version-line">{solver.version}</span>, build{' '}
        <code>{solver.commit}</code>, <Badge label={solver.label} /> {chip}
      </p>
    );
  }
  return (
    <p className={cls} data-testid={testId} data-state="ready">
      <strong>{METHOD_1}</strong>: solver version <span data-testid="solver-version">{solver.version}</span>, built from commit{' '}
      <code data-testid="solver-commit">{solver.commit}</code>, results labelled <Badge label={solver.label} />. Verification status in the
      research catalogue: {chip}, numerical verification of the stated model, not experimental validation. Scope: {solver.scope}.
    </p>
  );
}
/** Release fraction at the end time: Method 1, with Method 2 at the same time when the series resolves it. */
export function ReleaseReadout() {
  const { run, stale } = useCurrentRun();
  const status = useStrato((s) => s.status);
  const solver = useStrato((s) => s.solver.state);
  const problem = useStrato((s) => s.problem);

  if (!run) {
    let text = 'Waiting for the solver';
    if (solver === 'failed') text = 'Solver not available';
    else if (status === 'running') text = 'Running';
    else if (problem?.kind === 'rejected') text = 'Inputs rejected';
    else if (problem?.kind === 'error') text = 'Run failed';
    return (
      <div className="readout readout--wide" data-testid="release-readout" data-state={problem?.kind ?? solver}>
        <p className="readout__label">Release fraction at the end of the simulation (dimensionless)</p>
        <p className="readout__value readout__value--none">{text}</p>
        <p className="readout__meta">
          <span className="readout__nolabel">Label: none</span>
          {problem ? 'No result for these inputs; the reason is shown below.' : 'The value and its result label appear when the Rust solver has run.'}
        </p>
      </div>
    );
  }
  const d = run.diagnostics;
  const a = run.analytical;
  const tEnd = run.balance.final.t_s;
  const last = a ? a.times_s.length - 1 : -1;
  // The series value at the last output time, when the series resolves that time.
  const seriesAtEnd = a && a.applicable && last >= 0 && a.times_s[last] === tEnd ? (a.release_fraction[last] ?? null) : null;
  return (
    <div className={`readout readout--wide${stale ? ' is-stale' : ''}`} data-testid="release-readout" data-state="ok">
      <p className="readout__label">Release fraction at the end of the simulation (dimensionless)</p>
      <p className="readout__value" data-testid="release-value" data-value={String(d.final_release_fraction)}>
        {d.final_release_fraction.toPrecision(6)}
      </p>
      <Badge label={run.label} />
      <p className="readout__meta">
        {METHOD_1} at the time <Sci value={tEnd} unit="s" digits={4} />.{' '}
        {seriesAtEnd !== null ? (
          <>
            {METHOD_2} at the same time:{' '}
            <span data-testid="release-series" data-value={String(seriesAtEnd)}>
              {seriesAtEnd.toPrecision(6)}
            </span>
            .
          </>
        ) : (
          `${METHOD_2} is not available for this run.`
        )}{' '}
        <Updating stale={stale} />
      </p>
    </div>
  );
}

/** Rejected inputs or a solver failure, with the wrapper's own messages and a retry. */
export function RunProblemNote() {
  const problem = useStrato((s) => s.problem);
  const solver = useStrato((s) => s.solver);
  const requestRun = useStrato((s) => s.requestRun);
  if (solver.state === 'failed') {
    return (
      <div className="sim-alert" role="alert" data-testid="solver-failed">
        <p className="sim-alert__title">The WebAssembly solver could not be loaded, so there is no result.</p>
        <p className="sim-alert__text">{solver.error}</p>
      </div>
    );
  }
  if (!problem) return null;
  return (
    <div className="sim-alert" role="alert" data-testid={problem.kind === 'rejected' ? 'run-rejected' : 'run-error'}>
      <p className="sim-alert__title">
        {problem.kind === 'rejected' ? 'The solver rejected these inputs' : 'The run did not finish'}
        {problem.kind === 'rejected' ? <span className="muted"> (no result is shown)</span> : null}
      </p>
      <ul className="sim-alert__list">
        {problem.messages.map((m) => (
          <li key={m}>{m}</li>
        ))}
      </ul>
      {problem.kind === 'error' ? (
        <button type="button" className="btn btn--small" onClick={requestRun} data-testid="retry">
          Retry
        </button>
      ) : null}
    </div>
  );
}

/** The inventory balance of the run: start, remaining, released and the residual with its bound. */
export function BalancePanel() {
  const { run, stale } = useCurrentRun();
  if (!run) return <EmptyPanel title="Inventory balance" testId="balance-panel" />;
  const b = run.balance;
  const last = run.profiles[run.profiles.length - 1];
  return (
    <section className={`result-panel${stale ? ' is-stale' : ''}`} data-testid="balance-panel" aria-labelledby="balance-title">
      <header className="result-panel__head">
        <h3 id="balance-title" className="result-panel__title">
          Inventory balance
        </h3>
        <Badge label={run.label} />
      </header>
      <p className="result-panel__note">
        At the end of the simulation; the residual is zero in exact arithmetic, and the solver reports the derived rounding bound beside
        it. <Updating stale={stale} />
      </p>
      <table className="table table--compact">
        <tbody>
          <tr>
            <th scope="row">Initial inventory (mol)</th>
            <td className="num-col">
              <V id="bal-start" v={b.start_inventory_mol} unit="mol" />
            </td>
          </tr>
          <tr>
            <th scope="row">Remaining inventory in the particle (mol)</th>
            <td className="num-col">
              <V id="bal-inventory" v={b.final.remaining_mol} unit="mol" />
            </td>
          </tr>
          <tr>
            <th scope="row">Cumulative release through the outer surface (mol)</th>
            <td className="num-col">
              <V id="bal-released" v={b.final.released_mol} unit="mol" />
            </td>
          </tr>
          <tr className="is-residual">
            <th scope="row">Mass-balance residual (mol)</th>
            <td className="num-col">
              <V id="bal-residual" v={b.final.residual_mol} unit="mol" digits={3} />
            </td>
          </tr>
          <tr>
            <th scope="row">Derived rounding bound of the residual (mol)</th>
            <td className="num-col">
              <V id="bal-bound" v={b.final.residual_bound_mol} unit="mol" digits={3} />
            </td>
          </tr>
          <tr>
            <th scope="row">Largest relative mass-balance residual over the run (dimensionless)</th>
            <td className="num-col">
              <V id="bal-max-rel" v={b.max_abs_residual_relative} digits={3} />
            </td>
          </tr>
        </tbody>
      </table>
      <p className="result-panel__note" data-testid="bal-statement">
        {b.statement} {b.generated_and_decayed_note}
      </p>
      {last ? <LayerTable profile={last} names={run.grid.layer_names} /> : null}
    </section>
  );
}

function LayerTable({ profile, names }: { profile: RunResponse['profiles'][number]; names: string[] }) {
  return (
    <TableBox label="Inventory by layer">
    <table className="table table--compact layer-table" data-testid="layer-table">
      <caption className="table__caption">By layer at the end of the simulation (sums over the grid intervals of each layer, from the wrapper)</caption>
      <thead>
        <tr>
          <th scope="col">Layer</th>
          <th scope="col" className="num-col">
            Inventory (mol)
          </th>
          <th scope="col" className="num-col">
            Share of the inventory in the particle (dimensionless)
          </th>
        </tr>
      </thead>
      <tbody>
        {profile.layer_inventory_mol.map((inv, i) => (
          <tr key={names[i] ?? i}>
            <th scope="row">
              <span className="layer-dot" style={{ ['--ring' as string]: layerColour(i) }} aria-hidden="true" />
              {names[i] ?? `Region ${i + 1}`}
            </th>
            <td className="num-col">
              <V id={`layer-inv-${i}`} v={inv} unit="mol" digits={4} />
            </td>
            <td className="num-col">
              <V id={`layer-share-${i}`} v={profile.layer_share ? profile.layer_share[i] : null} digits={4} />
            </td>
          </tr>
        ))}
      </tbody>
    </table>
    </TableBox>
  );
}

function EmptyPanel({ title, testId }: { title: string; testId: string }) {
  return (
    <section className="result-panel result-panel--empty" data-testid={testId}>
      <header className="result-panel__head">
        <h3 className="result-panel__title">{title}</h3>
        <span className="readout__nolabel">Label: none</span>
      </header>
      <p className="result-panel__note">Appears when the Rust solver has run the current inputs.</p>
    </section>
  );
}

/** Numerical diagnostics the wrapper returns, and the measured solver time. */
export function DiagnosticsPanel() {
  const { run, stale } = useCurrentRun();
  const info = useStrato((s) => s.runInfo);
  if (!run) return <EmptyPanel title="Numerical diagnostics" testId="diagnostics-panel" />;
  const d = run.diagnostics;
  const c = d.analytical_comparison;
  return (
    <section className={`result-panel${stale ? ' is-stale' : ''}`} data-testid="diagnostics-panel" aria-labelledby="diag-title">
      <header className="result-panel__head">
        <h3 id="diag-title" className="result-panel__title">
          Numerical diagnostics
        </h3>
        <Badge label={run.label} />
      </header>
      <dl className="diag">
        <div className="diag__row">
          <dt>Grid intervals (dimensionless)</dt>
          <dd data-testid="diag-intervals" data-value={String(d.intervals_total)}>
            {count(d.intervals_total)}
          </dd>
        </div>
        <div className="diag__row">
          <dt>Grid nodes (dimensionless)</dt>
          <dd data-testid="diag-nodes" data-value={String(d.nodes)}>
            {count(d.nodes)}
          </dd>
        </div>
        <div className="diag__row">
          <dt>Backward Euler time steps (dimensionless)</dt>
          <dd data-testid="diag-steps" data-value={String(d.steps)}>
            {count(d.steps)}
          </dd>
        </div>
        <div className="diag__row">
          <dt>Time step used (s)</dt>
          <dd>
            <V id="diag-dt" v={d.dt_used_s} unit="s" />
            {d.dt_used_s !== d.dt_requested_s ? (
              <span className="muted">
                {' '}
                (requested <Sci value={d.dt_requested_s} unit="s" />)
              </span>
            ) : null}
          </dd>
        </div>
        {d.fourier_numbers.map((f, i) => (
          <div className="diag__row" key={`fo-${i}`}>
            <dt>Mesh Fourier number of the {run.grid.layer_names[i] ?? `layer ${i + 1}`} layer (dimensionless)</dt>
            <dd>
              <V id={`diag-fo-${i}`} v={f} digits={4} />
            </dd>
          </div>
        ))}
        <div className="diag__row">
          <dt>Largest relative mass-balance residual (dimensionless)</dt>
          <dd>
            <V id="diag-residual" v={d.max_abs_balance_residual_relative} digits={3} />
          </dd>
        </div>
        <div className="diag__row">
          <dt>Residual within its derived rounding bound at every output time</dt>
          <dd data-testid="diag-within-bound" data-value={String(run.balance.tolerance.within_bound)}>
            {run.balance.tolerance.within_bound ? 'yes' : 'no'}
          </dd>
        </div>
        <div className="diag__row">
          <dt>Smallest concentration over all nodes and times (mol/m³)</dt>
          <dd>
            <V id="diag-min" v={d.min_concentration_mol_m3} unit="mol/m³" digits={3} />
          </dd>
        </div>
        <div className="diag__row">
          <dt>Largest absolute difference between {METHOD_1} and {METHOD_2} in the release fraction (dimensionless)</dt>
          <dd>{c ? <V id="diag-bc-max" v={c.max_abs_difference} digits={4} /> : <span className="muted">{METHOD_2} not available</span>}</dd>
        </div>
        <div className="diag__row">
          <dt>Absolute difference between {METHOD_1} and {METHOD_2} at the end of the simulation (dimensionless)</dt>
          <dd>{c ? <V id="diag-bc-final" v={c.final_abs_difference} digits={4} /> : <span className="muted">not available</span>}</dd>
        </div>
        {c ? (
          <>
            <div className="diag__row">
              <dt>Eigenmodes of {METHOD_2} (dimensionless)</dt>
              <dd data-testid="diag-modes" data-value={String(c.modes)}>
                {count(c.modes)}
              </dd>
            </div>
            <div className="diag__row">
              <dt>Accuracy floor of {METHOD_2} (dimensionless)</dt>
              <dd>
                <V id="diag-floor" v={c.accuracy_floor} digits={4} />
              </dd>
            </div>
            <div className="diag__row">
              <dt>{METHOD_2} valid from the time (s)</dt>
              <dd>
                <V id="diag-valid" v={c.valid_from_s} unit="s" digits={4} />
              </dd>
            </div>
          </>
        ) : null}
        <div className="diag__row diag__row--timing">
          <dt>Solver time</dt>
          <dd data-testid="diag-time">
            {info ? `${info.elapsedMs.toFixed(1)} ms` : '–'} <span className="muted">wall time in this browser, not a result</span>
          </dd>
        </div>
      </dl>
      {c && !c.resolved ? (
        <p className="result-panel__note" data-testid="diag-unresolved">
          {METHOD_1} and {METHOD_2} differ by less than three times the accuracy floor of {METHOD_2}; the difference is not resolved.
        </p>
      ) : null}
      {run.messages.length > 0 ? (
        <ul className="result-panel__messages" data-testid="run-messages">
          {run.messages.map((m) => (
            <li key={m}>{m}</li>
          ))}
        </ul>
      ) : null}
      <p className="result-panel__note">
        <Updating stale={stale} />
        {METHOD_1}, solver version {run.solver.version}, WebAssembly build <code>{run.engine.commit}</code>
        {run.engine.commit !== BUILD.commit ? <strong> (differs from the page build {BUILD.commit})</strong> : null}. {run.solver.nature}.
      </p>
    </section>
  );
}
