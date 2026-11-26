// Block B results (web phase 3): every value shown is the field of the wrapper's RunResponse, with
// its result label; nothing is computed in the browser (owner decision M4). The fixture is the
// native wrapper response (the BTCS run) written by strato-webtools precomputed.
import { act, render, screen, within } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeEach, describe, expect, it } from 'vitest';
import { METHOD_1, METHOD_2 } from '../src/methods';
import { BalancePanel, DiagnosticsPanel, ReleaseReadout, RunProblemNote, SolverStatus } from '../src/panels/SimResults';
import type { RunResponse } from '../src/solver/types';
import { initialState, inputKey, useStrato } from '../src/state/store';

// Vitest runs from web/ (jsdom gives import.meta.url no file scheme here).
const native = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'wrapper_native_synthetic.json'), 'utf8')) as {
  label: string;
  data: RunResponse;
};
const RUN = native.data;

function withRun() {
  const s = useStrato.getState();
  act(() => {
    s.setSolver({ state: 'ready' });
    s.setRunOutcome({ run: RUN, info: { elapsedMs: 3.25, requestId: 1, inputKey: inputKey(s.config), dropped: [] } });
  });
}

function value(id: string): number {
  const el = screen.getByTestId(id);
  return Number(el.dataset.value);
}

beforeEach(() => {
  useStrato.setState({ ...initialState() });
});

describe('the fixture is a precomputed native wrapper response', () => {
  it('carries the provenance header and an ok status', () => {
    expect(native.label).toBe('PRECOMPUTED');
    expect(RUN.status).toBe('ok');
    expect(RUN.label).toBe('COMPUTED v2.0');
    expect(RUN.solver.capability).toBe('diffusion_btcs_layered');
  });
});

describe('release readout', () => {
  it('shows the wrapper final release fraction and the series value at the end time, with the label', () => {
    withRun();
    render(<ReleaseReadout />);
    const r = screen.getByTestId('release-readout');
    expect(value('release-value')).toBe(RUN.diagnostics.final_release_fraction);
    expect(value('release-value')).toBe(RUN.release_fraction[RUN.release_fraction.length - 1]);
    const a = RUN.analytical;
    if (!a) throw new Error('no series in the fixture');
    expect(value('release-series')).toBe(a.release_fraction[a.release_fraction.length - 1]);
    expect(r.querySelector('.badge')?.textContent).toBe('COMPUTED v2.0');
    expect(r.textContent).toContain(METHOD_1);
    expect(r.textContent).toContain(METHOD_2);
    expect(r.textContent).not.toMatch(/Code [BC]|finite volume/i);
  });

  it('says no result and shows no label before a reply', () => {
    render(<ReleaseReadout />);
    expect(screen.getByTestId('release-readout').querySelector('.badge')).toBeNull();
  });

  it('marks the result as updating while the inputs differ from those of the result', () => {
    withRun();
    act(() => useStrato.getState().setTimeField('end_time_s', 1000));
    render(<ReleaseReadout />);
    expect(screen.getByTestId('result-stale')).toBeTruthy();
  });
});

describe('inventory balance', () => {
  it('shows the start inventory, inventory, released amount and residual exactly as returned', () => {
    withRun();
    render(<BalancePanel />);
    const b = RUN.balance;
    expect(value('bal-start')).toBe(b.start_inventory_mol);
    expect(value('bal-inventory')).toBe(b.final.remaining_mol);
    expect(value('bal-bound')).toBe(b.final.residual_bound_mol);
    expect(value('bal-released')).toBe(b.final.released_mol);
    expect(value('bal-residual')).toBe(b.final.residual_mol);
    expect(value('bal-max-rel')).toBe(b.max_abs_residual_relative);
    expect(screen.getByTestId('balance-panel').querySelector('.badge')?.textContent).toBe('COMPUTED v2.0');
    // the wrapper's own statement of the identity and its sign convention is printed, not re-signed
    expect(screen.getByTestId('bal-statement').textContent).toContain(b.statement);
  });

  it('shows the layer inventories and shares of the last profile as returned (no sums in TypeScript)', () => {
    withRun();
    render(<BalancePanel />);
    const last = RUN.profiles[RUN.profiles.length - 1]!;
    const table = screen.getByTestId('layer-table');
    expect(within(table).getAllByRole('row')).toHaveLength(1 + RUN.grid.layer_names.length);
    last.layer_inventory_mol.forEach((v, i) => {
      expect(value(`layer-inv-${i}`)).toBe(v);
      expect(value(`layer-share-${i}`)).toBe(last.layer_share?.[i]);
      expect(within(table).getByText(RUN.grid.layer_names[i]!)).toBeTruthy();
    });
  });
});

describe('numerical diagnostics', () => {
  it('shows the returned diagnostics and the measured solver time, which carries no result label', () => {
    withRun();
    render(<DiagnosticsPanel />);
    const d = RUN.diagnostics;
    expect(value('diag-intervals')).toBe(d.intervals_total);
    expect(value('diag-nodes')).toBe(d.nodes);
    expect(value('diag-steps')).toBe(d.steps);
    expect(value('diag-dt')).toBe(d.dt_used_s);
    expect(value('diag-residual')).toBe(d.max_abs_balance_residual_relative);
    expect(value('diag-bc-max')).toBe(d.analytical_comparison?.max_abs_difference);
    expect(value('diag-bc-final')).toBe(d.analytical_comparison?.final_abs_difference);
    expect(value('diag-floor')).toBe(d.analytical_comparison?.accuracy_floor);
    expect(value('diag-modes')).toBe(d.analytical_comparison?.modes);
    d.fourier_numbers.forEach((f, i) => expect(value('diag-fo-' + String(i))).toBe(f));
    const time = screen.getByTestId('diag-time');
    expect(time.textContent).toContain('3.3 ms');
    expect(time.textContent).toContain('not a result');
    expect(time.dataset.label).toBeUndefined();
  });
});

describe('rejected inputs and failures', () => {
  it('lists the wrapper messages and shows no result', () => {
    act(() => {
      useStrato.getState().setSolver({ state: 'ready' });
      useStrato.getState().setRunOutcome({ problem: { kind: 'rejected', messages: ['invalid time controls'] }, info: null });
    });
    render(
      <>
        <ReleaseReadout />
        <RunProblemNote />
        <BalancePanel />
      </>,
    );
    expect(screen.getByTestId('run-rejected').textContent).toContain('invalid time controls');
    expect(screen.getByTestId('release-readout').textContent).toContain('Inputs rejected');
    expect(screen.getByTestId('balance-panel').querySelector('.badge')).toBeNull();
    expect(useStrato.getState().run).toBeNull();
  });

  it('offers a retry after a solver failure', () => {
    let asked = 0;
    act(() => {
      useStrato.setState({ requestRun: () => (asked += 1) });
      useStrato.getState().setSolver({ state: 'ready' });
      useStrato.getState().setRunOutcome({ problem: { kind: 'error', messages: ['no reply within 10000 ms'] }, info: null });
    });
    render(<RunProblemNote />);
    act(() => screen.getByTestId('retry').click());
    expect(asked).toBe(1);
  });
});

describe('solver version and verification status', () => {
  it('shows the method, the solver version and commit, the label and the catalogue status of the solver', () => {
    act(() => {
      const s = useStrato.getState();
      s.setSolver({ state: 'ready', build: { abi: 2, crate: 'strato-wasm', wrapper_version: '0', strato_version: '1.0.1', commit: 'abc1234', dirty: false, wasm_bindgen: '0', btcs: { solver: RUN.solver, schema: 2, functions: [], export_functions: [], export_formats: [], capabilities: ['diffusion_btcs_layered'], label: 'COMPUTED v2.0', limits: { max_total_intervals: 1, min_intervals_per_layer: 1, max_steps: 1, max_profile_values: 1, max_profile_times: 1, max_series_modes: 1 } } } });
    });
    render(<SolverStatus />);
    const el = screen.getByTestId('solver-status');
    expect(el.dataset.state).toBe('ready');
    expect(el.textContent).toContain(METHOD_1);
    expect(screen.getByTestId('solver-version').textContent).toBe(RUN.solver.version);
    expect(screen.getByTestId('solver-commit').textContent).toBe(RUN.solver.commit);
    expect(el.querySelector('.badge')?.textContent).toBe('COMPUTED v2.0');
    // the status is read from the generated research catalogue, and the page says what it is not
    expect(el.querySelector('[data-status-for]')?.getAttribute('data-status')).toBe('VERIFIED');
    expect(el.textContent).toContain('not experimental validation');
  });
});