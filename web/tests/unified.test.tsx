// The unified selection of the Explore page (owner directive of 26 Nov 2026, Sections 6 and 7): a reply is shown
// only under the physics and scenario it was computed for, the running computation is described from the
// capability of the build, and a selection the build does not compute shows no number of another model.
import { act, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeEach, describe, expect, it } from 'vitest';
import { ActiveModel } from '../src/equation/ActiveModel';
import { buildCapability, computedMechanisms, phrase, type BuildCapability } from '../src/equation/capability';
import { LiveValues } from '../src/equation/LiveValues';
import { BalancePanel } from '../src/panels/SimResults';
import { BTCS_CAPABILITY } from '../src/equation/availability';
import { useShownRun } from '../src/solver/shown';
import type { RunOk } from '../src/solver/types';
import { initialState, inputKey, selectionKey, useStrato } from '../src/state/store';

const RUN = (JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'wrapper_native_synthetic.json'), 'utf8')) as { data: RunOk }).data;

function Probe() {
  const { run, stale } = useShownRun();
  return <p data-testid="probe">{run === null ? 'none' : stale ? 'stale' : 'shown'}</p>;
}

function runFor(selection: { physics: Parameters<typeof selectionKey>[0]['physics']; scenario: Parameters<typeof selectionKey>[0]['scenario'] }) {
  const s = useStrato.getState();
  act(() => {
    s.setSolver({ state: 'ready' });
    s.setCapabilities([BTCS_CAPABILITY]);
    s.setRunOutcome({ run: RUN, info: { elapsedMs: 1, requestId: 1, inputKey: inputKey(s.config), dropped: [], selectionKey: selectionKey(selection) } });
  });
}

beforeEach(() => {
  useStrato.setState({ ...initialState() });
});

describe('the manual solver shows its reply for the baseline problem, whatever the Explore selection is', () => {
  it('shows the baseline reply', () => {
    runFor({ physics: 'baseline', scenario: null });
    render(<Probe />);
    expect(screen.getByTestId('probe').textContent).toBe('shown');
  });

  it('is not changed by the model equation or the operation selected on Explore (the two groups of buttons do not start or hide it)', () => {
    runFor({ physics: 'baseline', scenario: null });
    render(<Probe />);
    for (const patch of [{ physics: 'generation' as const }, { physics: 'decay' as const }, { physics: 'temperature' as const }, { physics: 'future' as const, scenario: 'startup' as const }]) {
      act(() => useStrato.getState().setSelection(patch));
      expect(screen.getByTestId('probe').textContent, JSON.stringify(patch)).toBe('shown');
    }
  });

  it('hides, with no substitute, a reply that was computed for a problem other than the baseline', () => {
    runFor({ physics: 'decay', scenario: null });
    render(<Probe />);
    expect(screen.getByTestId('probe').textContent).toBe('none');
  });

  it('keeps a reply without a selection key shown (older callers)', () => {
    const s = useStrato.getState();
    act(() => s.setRunOutcome({ run: RUN, info: { elapsedMs: 1, requestId: 1, inputKey: inputKey(s.config), dropped: [] } }));
    render(<Probe />);
    expect(screen.getByTestId('probe').textContent).toBe('shown');
  });

  it('gives the result panels the numbers of the baseline reply under any Explore selection', () => {
    runFor({ physics: 'baseline', scenario: null });
    render(<BalancePanel />);
    expect(screen.getByTestId('bal-inventory').dataset.value).toBe(String(RUN.balance.final.remaining_mol));
    act(() => useStrato.getState().setSelection({ physics: 'decay' }));
    expect(screen.getByTestId('bal-inventory').dataset.value).toBe(String(RUN.balance.final.remaining_mol));
  });

  it('gives the live values of a numerical step no number of a reply computed for another problem', () => {
    runFor({ physics: 'baseline', scenario: null });
    render(<LiveValues stage={12} />);
    expect(screen.getByTestId('live-numbers')).toBeTruthy();
    runFor({ physics: 'generation', scenario: null });
    expect(screen.queryByTestId('live-numbers')).toBeNull();
    expect(screen.getByTestId('live-numbers-none').textContent).toContain('No numbers yet');
  });
});
describe('the running computation is described from the capability of the build', () => {
  it('names the baseline and lists what the build does not compute (today)', () => {
    act(() => {
      useStrato.getState().setSolver({ state: 'ready' });
      useStrato.getState().setCapabilities([BTCS_CAPABILITY]);
    });
    render(<ActiveModel />);
    const text = screen.getByTestId('active-model').textContent ?? '';
    // One line; the solver is named by the status line beside it, the scope sentence is in Scientific Documentation.
    expect(text).toBe('Computes: Baseline diffusion. Not computed: fission-product generation, radioactive decay and temperature-dependent diffusion.');
    expect(screen.getByTestId('active-model').dataset.computed).toBe('');
  });

  it('does not say that it computes the selection, and says what the build computes, when the selection is not computed', () => {
    act(() => {
      useStrato.getState().setSolver({ state: 'ready' });
      useStrato.getState().setCapabilities([BTCS_CAPABILITY]);
    });
    render(<ActiveModel supported={false} />);
    const text = screen.getByTestId('active-model').textContent ?? '';
    expect(text).toContain('This build computes: Baseline diffusion');
    expect(text).not.toMatch(/^Computes/);
  });

  it('derives the baseline phrase from the capability object', () => {
    act(() => {
      useStrato.getState().setSolver({ state: 'ready' });
      useStrato.getState().setCapabilities(['some_other_capability']);
    });
    render(<ActiveModel />);
    expect(screen.getByTestId('active-model').textContent).toContain('No model of this page');
  });

  it('says nothing before the wrapper has reported', () => {
    render(<ActiveModel />);
    expect(screen.queryByTestId('active-model')).toBeNull();
  });

  it('derives the computed mechanisms from the capability object, so a later wrapper needs no edit', () => {
    const cap: BuildCapability = buildCapability([BTCS_CAPABILITY]);
    expect(computedMechanisms(cap)).toEqual({ computed: [], notComputed: ['generation', 'decay', 'temperature'] });
    const later: BuildCapability = { ...cap, mechanisms: { generation: 'verified', decay: 'verified', temperature: 'implemented' } };
    expect(computedMechanisms(later)).toEqual({ computed: ['generation', 'decay'], notComputed: ['temperature'] });
    expect(phrase(['a', 'b', 'c'])).toBe('a, b and c');
  });
});
