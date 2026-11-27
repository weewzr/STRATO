// The one condition under which a computed status is shown (state/store.ts runIsCurrent): the stored run is the manual
// solver's run of the baseline problem, it belongs to the current input key, and the build supports it. The model
// equation and the operation selected on Explore do not enter (they neither start nor change that run). Unit tests of
// the condition, and the sequence of the independent check of the live site, rewritten for the final layout: run the
// baseline, edit an input (the status goes), run again (it returns), select fission-product generation on Explore
// (nothing changes in the manual solver).
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeEach, describe, expect, it } from 'vitest';
import { BTCS_CAPABILITY } from '../src/equation/availability';
import { missingList, phrase, unsupportedMessage } from '../src/equation/capability';
import { StatusLine } from '../src/panels/StatusLine';
import { AdvancedSolver } from '../src/advanced/AdvancedSolver';
import type { RunOk, RunResponse } from '../src/solver/types';
import { initialState, inputKey, runIsCurrent, selectionKey, useStrato, type RunInfo } from '../src/state/store';

const native = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'wrapper_native_synthetic.json'), 'utf8')) as { data: RunResponse };
const RUN = native.data as RunOk;

const BASELINE = { physics: 'baseline', scenario: null } as const;
const GENERATION = { physics: 'generation', scenario: null } as const;

function stateWith(info: Partial<RunInfo> | null, selection: { physics: 'baseline' | 'generation'; scenario: null } = BASELINE, run: unknown = RUN) {
  const config = useStrato.getState().config;
  return {
    run,
    runInfo: info === null ? null : ({ elapsedMs: 1, requestId: 1, inputKey: inputKey(config), dropped: [], selectionKey: selectionKey(BASELINE), ...info } as RunInfo),
    config,
    selection,
  };
}

beforeEach(() => {
  useStrato.setState({ ...initialState() });
});

describe('runIsCurrent: the stored run is the baseline run of the current input key, and the build supports it', () => {
  it('holds for a run of the baseline problem and the current inputs', () => {
    expect(runIsCurrent(stateWith({}), true)).toBe(true);
  });

  it('does not hold without a run or without the information on the run', () => {
    expect(runIsCurrent(stateWith({}, BASELINE, null), true)).toBe(false);
    expect(runIsCurrent(stateWith(null), true)).toBe(false);
  });

  it('does not hold for a run of another problem than the baseline', () => {
    expect(runIsCurrent(stateWith({ selectionKey: selectionKey(GENERATION) }), true)).toBe(false);
  });

  it('does not depend on the model equation or the operation selected on Explore', () => {
    expect(runIsCurrent(stateWith({}, GENERATION), true)).toBe(true);
  });

  it('does not hold for a run of other inputs', () => {
    expect(runIsCurrent(stateWith({ inputKey: 'not-the-current-key' }), true)).toBe(false);
  });

  it('does not hold when the build does not support the selection, even for a matching run', () => {
    expect(runIsCurrent(stateWith({ selectionKey: selectionKey(GENERATION) }, GENERATION), false)).toBe(false);
    expect(runIsCurrent(stateWith({}), false)).toBe(false);
  });

  it('counts a reply without a selection key as the baseline run', () => {
    const st = stateWith({});
    delete (st.runInfo as { selectionKey?: string }).selectionKey;
    expect(runIsCurrent(st, true)).toBe(true);
  });
});

describe('the unsupported message joins the list with commas and one final "and"', () => {
  it('names one, two and three items', () => {
    expect(phrase(['a'])).toBe('a');
    expect(phrase(['a', 'b'])).toBe('a and b');
    expect(missingList({ supported: false, missing: ['a', 'b', 'c'] })).toBe('a, b and c');
    expect(unsupportedMessage({ supported: false, missing: ['a', 'b', 'c'] })).toBe('The solver in this build does not implement a, b and c.');
  });
});

function ready() {
  act(() => {
    const s = useStrato.getState();
    s.setSolver({
      state: 'ready',
      build: { abi: 3, crate: 'strato-wasm', wrapper_version: '0', strato_version: '2.0.0', commit: 'abc1234', dirty: false, wasm_bindgen: '0', btcs: { solver: native.data.solver, schema: 2, functions: [], export_functions: [], export_formats: [], capabilities: [BTCS_CAPABILITY], label: 'COMPUTED v2.0', limits: { max_total_intervals: 1, min_intervals_per_layer: 1, max_steps: 1, max_profile_values: 1, max_profile_times: 1, max_series_modes: 1 } } },
    });
    s.setCapabilities([BTCS_CAPABILITY]);
  });
}

function storeRun() {
  act(() => {
    const s = useStrato.getState();
    s.setRunOutcome({ run: RUN, info: { elapsedMs: 2, requestId: s.runInfo ? s.runInfo.requestId + 1 : 1, inputKey: inputKey(s.config), dropped: [], selectionKey: selectionKey(BASELINE) } });
  });
}

describe('the manual solver after a run of the baseline, an edit, a second run and a switch to fission-product generation on Explore', () => {
  it('shows the computed status of the baseline run, loses it while the stored run is of other inputs, and keeps it when Explore selects another model equation', async () => {
    const user = userEvent.setup();
    ready();
    render(
      <>
        <AdvancedSolver compact={false} />
        <StatusLine />
      </>,
    );
    // The baseline, run: the badge, the result and the baseline line are there.
    storeRun();
    const header = () => screen.getByTestId('solver-status-line');
    expect(header().querySelector('[data-label="COMPUTED v2.0"]')).not.toBeNull();
    expect(screen.getByTestId('release-value')).toBeTruthy();
    expect(screen.getByTestId('issue-count').textContent).toBe('Inputs equal the baseline.');

    // An input edited (the SiC diffusivity): the stored run is of other inputs, so no computed status stays.
    act(() => useStrato.getState().setRegionField(3, 'diffusivity_m2_s', 1e-13));
    expect(header().querySelector('[data-label]')).toBeNull();
    // The same problem run again.
    await user.click(screen.getAllByTestId('run')[0] as HTMLElement);
    expect(screen.getByTestId('announce').textContent).toBe('Running the current inputs.');
    storeRun();
    expect(header().querySelector('[data-label="COMPUTED v2.0"]')).not.toBeNull();
    expect(screen.getByTestId('issue-count').textContent).toBe('1 input differs from the baseline.');

    // Fission-product generation selected on Explore: the manual solver is the baseline solver and does not change.
    act(() => useStrato.getState().setSelection({ physics: 'generation' }));
    expect(screen.queryByTestId('unsupported-simulation')).toBeNull();
    expect(header().querySelector('[data-label="COMPUTED v2.0"]')).not.toBeNull();
    expect(screen.getByTestId('release-value')).toBeTruthy();
    expect(screen.getByTestId('issue-count').textContent).toBe('1 input differs from the baseline.');
    expect(screen.getByTestId('status-result').textContent).toContain('COMPUTED');
  });

  it('shows no computed status while the stored run is of other inputs', () => {
    ready();
    render(<AdvancedSolver compact={false} />);
    storeRun();
    act(() => useStrato.getState().setRegionField(3, 'diffusivity_m2_s', 1e-13));
    expect(screen.getByTestId('solver-status-line').querySelector('[data-label]')).toBeNull();
  });
});
