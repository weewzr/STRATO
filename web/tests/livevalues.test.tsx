// The hooks of the equation builder for numbers of the current run ("Live values" of a step): every
// number is a field of the wrapper's reply, named in full with its unit, and a step whose quantity the
// run does not return shows none.
import { act, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeEach, describe, expect, it } from 'vitest';
import { LiveValues, liveRows } from '../src/equation/LiveValues';
import type { Stage } from '../src/equation/terms';
import type { RunOk } from '../src/solver/types';
import { initialState, inputKey, useStrato } from '../src/state/store';

const RUN = (JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'wrapper_native_synthetic.json'), 'utf8')) as { data: RunOk }).data;

function withRun() {
  const s = useStrato.getState();
  act(() => {
    s.setSolver({ state: 'ready' });
    s.setRunOutcome({ run: RUN, info: { elapsedMs: 1, requestId: 1, inputKey: inputKey(s.config), dropped: [] } });
  });
}

beforeEach(() => {
  useStrato.setState({ ...initialState() });
});

describe('numbers of the run for each builder step', () => {
  it('are the wrapper values, step by step', () => {
    const last = RUN.profiles.length - 1;
    const byId = (stage: Stage) => Object.fromEntries(liveRows(stage, RUN, last).map((r) => [r.id, r.value]));
    expect(byId(5)['live-nodes']).toBe(RUN.grid.nodes);
    RUN.grid.spacing_per_layer_m.forEach((h, i) => expect(byId(5)[`live-h-${i}`]).toBe(h));
    expect(byId(6)['live-dt']).toBe(RUN.diagnostics.dt_used_s);
    RUN.diagnostics.fourier_numbers.forEach((f, i) => expect(byId(6)[`live-fo-${i}`]).toBe(f));
    expect(byId(7)['live-centre']).toBe(RUN.profiles[last]!.c_mol_per_m3[0]);
    expect(byId(9)['live-release']).toBe(RUN.diagnostics.final_release_fraction);
    expect(byId(10)['live-norm']).toBe(RUN.diagnostics.matrix!.norm_inf);
    expect(byId(11)['live-version']).toBe(RUN.solver.version);
    expect(byId(12)['live-residual']).toBe(RUN.balance.final.residual_mol);
    expect(byId(12)['live-bound']).toBe(RUN.balance.final.residual_bound_mol);
    expect(byId(13)['live-an-max']).toBe(RUN.diagnostics.analytical_comparison!.max_abs_difference);
  });

  it('are absent for the steps that state mathematics, the previews and the comparison placeholder', () => {
    for (const s of [1, 2, 3, 14, 15, 16, 17] as Stage[]) expect(liveRows(s, RUN, 0), `step ${s}`).toEqual([]);
  });

  it('are shown with the full name of the quantity, and not at all before a run', () => {
    render(<LiveValues stage={12} />);
    expect(screen.getByTestId('live-numbers-none').textContent).toContain('No numbers yet');
    expect(screen.queryByTestId('live-numbers')).toBeNull();
    withRun();
    const box = screen.getByTestId('live-numbers');
    expect(box.textContent).toContain('Remaining inventory at the end of the run');
    expect(box.textContent).toContain('Mass-balance residual at the end of the run');
    expect(screen.getByTestId('live-residual').dataset.value).toBe(String(RUN.balance.final.residual_mol));
    for (const dt of Array.from(box.querySelectorAll('dt'))) expect((dt.textContent ?? '').length, dt.textContent ?? '').toBeGreaterThan(8);
  });

  it('show the research status of the solver at the verification step, read from the catalogue', () => {
    withRun();
    render(<LiveValues stage={13} />);
    expect(screen.getByTestId('live-status').textContent).toBe('VERIFIED');
  });
});
