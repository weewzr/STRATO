// The one condition under which a computed status is shown (state/store.ts runIsCurrent): the stored run belongs to
// the current selection and input key, and the build supports the selection. Unit tests of the condition, and the
// sequence of the independent check of the live site: run the baseline, edit an input, run again, select
// fission-product generation: no badge, no "Running" line, no "Inputs equal the baseline" line, no result number.
import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeEach, describe, expect, it } from 'vitest';
import { BTCS_CAPABILITY } from '../src/equation/availability';
import { missingList, phrase, unsupportedMessage } from '../src/equation/capability';
import { StatusLine } from '../src/panels/StatusLine';
import { Explore } from '../src/sections/Explore';
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

describe('runIsCurrent: the stored run belongs to the current selection and input key, and the selection is supported', () => {
  it('holds for a run of the current selection and inputs', () => {
    expect(runIsCurrent(stateWith({}), true)).toBe(true);
  });

  it('does not hold without a run or without the information on the run', () => {
    expect(runIsCurrent(stateWith({}, BASELINE, null), true)).toBe(false);
    expect(runIsCurrent(stateWith(null), true)).toBe(false);
  });

  it('does not hold for a run of another selection', () => {
    expect(runIsCurrent(stateWith({}, GENERATION), true)).toBe(false);
  });

  it('does not hold for a run of other inputs', () => {
    expect(runIsCurrent(stateWith({ inputKey: 'not-the-current-key' }), true)).toBe(false);
  });

  it('does not hold when the build does not support the selection, even for a matching run', () => {
    expect(runIsCurrent(stateWith({ selectionKey: selectionKey(GENERATION) }, GENERATION), false)).toBe(false);
    expect(runIsCurrent(stateWith({}), false)).toBe(false);
  });

  it('counts a reply without a selection key as belonging to the current selection', () => {
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
    s.setRunOutcome({ run: RUN, info: { elapsedMs: 2, requestId: s.runInfo ? s.runInfo.requestId + 1 : 1, inputKey: inputKey(s.config), dropped: [], selectionKey: selectionKey(s.selection) } });
  });
}

describe('Explore after a run of the baseline, an edit, a second run and a switch to fission-product generation', () => {
  it('shows no computed status, no run line, no baseline line and no result number for the unsupported selection', async () => {
    const user = userEvent.setup();
    ready();
    render(
      <>
        <Explore compact={false} onOpenControls={() => undefined} />
        <StatusLine />
      </>,
    );
    // The baseline, run: the badge, the result and the baseline line are there.
    storeRun();
    const header = () => screen.getByTestId('solver-status-line');
    expect(header().querySelector('[data-label="COMPUTED v2.0"]')).not.toBeNull();
    expect(screen.getByTestId('release-value')).toBeTruthy();
    expect(screen.getByTestId('issue-count').textContent).toBe('Inputs equal the baseline.');

    // An input edited (the SiC diffusivity), the same selection run again.
    act(() => useStrato.getState().setRegionField(3, 'diffusivity_m2_s', 1e-13));
    await user.click(screen.getAllByTestId('run')[0] as HTMLElement);
    expect(screen.getByTestId('announce').textContent).toBe('Running the current inputs.');
    storeRun();
    expect(header().querySelector('[data-label="COMPUTED v2.0"]')).not.toBeNull();
    expect(screen.getByTestId('issue-count').textContent).toBe('1 input differs from the baseline.');

    // Fission-product generation selected.
    act(() => useStrato.getState().setSelection({ physics: 'generation' }));
    expect(screen.getByTestId('unsupported-simulation').textContent).toContain('No simulation for this selection');
    expect(header().textContent).toMatch(/^Method A: BTCS v2\.0\.0/);
    expect(header().querySelector('[data-label]')).toBeNull();
    expect(header().textContent).not.toMatch(/COMPUTED|VERIFIED/i);
    expect(document.querySelector('.badge')).toBeNull();
    expect(screen.queryByTestId('release-value')).toBeNull();
    expect(screen.queryByTestId('release-readout')).toBeNull();
    expect(screen.getByTestId('announce').textContent).toBe('');
    expect(document.body.textContent).not.toContain('Running the current inputs.');
    expect(document.body.textContent).not.toContain('Inputs equal the baseline.');
    expect(screen.queryByTestId('issue-count')).toBeNull();
    expect(within(screen.getByTestId('status-result')).queryByText(/COMPUTED/)).toBeNull();
    expect(screen.getByTestId('status-result').textContent).toBe('Result: none for the selection on Explore');
  });

  it('shows the status again on the baseline once a run of the current inputs is stored', () => {
    ready();
    render(<Explore compact={false} onOpenControls={() => undefined} />);
    storeRun();
    act(() => useStrato.getState().setSelection({ physics: 'generation' }));
    expect(document.querySelector('.badge')).toBeNull();
    act(() => useStrato.getState().setSelection({ physics: 'baseline' }));
    expect(screen.getByTestId('solver-status-line').querySelector('[data-label="COMPUTED v2.0"]')).not.toBeNull();
  });

  it('shows no computed status on the baseline while the stored run is of other inputs', () => {
    ready();
    render(<Explore compact={false} onOpenControls={() => undefined} />);
    storeRun();
    act(() => useStrato.getState().setRegionField(3, 'diffusivity_m2_s', 1e-13));
    expect(screen.getByTestId('solver-status-line').querySelector('[data-label]')).toBeNull();
  });
});
