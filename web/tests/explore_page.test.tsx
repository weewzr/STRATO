// The Explore page as a whole (final layout of 27 Nov 2026): no status word anywhere in its DOM, nothing of the
// removed controls, and a reply of the wrapper that arrives after the selection has changed is dropped.
import { act, render, renderHook, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { PHYSICS_IDS, SCENARIO_IDS } from '../src/equation/capability';
import { GRAPH_IDS, SECTION_IDS } from '../src/equation/selectionIds';
import { stageRequest, type Species } from '../src/graph/exploreData';
import { useStageResult } from '../src/graph/useStageResult';
import { Explore } from '../src/sections/Explore';
import type { AnalyticalReply, StageOk } from '../src/solver/types';
import { initialState, useStrato } from '../src/state/store';

const pending = vi.hoisted(() => ({ stage: [] as ((r: unknown) => void)[], analytical: [] as ((r: unknown) => void)[] }));

vi.mock('../src/solver/session', () => ({
  solverClient: () => ({
    runStage: () => new Promise((resolve) => pending.stage.push(resolve)),
    analytical: () => new Promise((resolve) => pending.analytical.push(resolve)),
    options: () => new Promise(() => undefined),
  }),
}));

const SR: Species = { id: 'sr90', name: 'Sr-90', element: 'Sr' };
const reply = (tag: number): StageOk => ({
  schema: 3,
  status: 'ok',
  label: 'COMPUTED v2.0',
  stage: { stage: 1, id: 'baseline', name: 'Baseline', adds: '', mechanisms: [] },
  grid: { node_radii_m: [0, 1], interface_radii_m: [], layer_names: [] },
  series: { times_s: [0, tag], times_h: [0, tag], release_fraction: [0, 0.5] },
  profiles: [],
  analytical: { applicable: false, extension: null, reason_code: 'X', reason: 'not applicable' },
  units: {},
});

beforeEach(() => {
  useStrato.setState({ ...initialState() });
  pending.stage.length = 0;
  pending.analytical.length = 0;
});

describe('Explore, whole page', () => {
  it('has no status word and no status badge for any model equation, operation, graph or section', { timeout: 120_000 }, () => {
    const { container } = render(<Explore />);
    act(() => SECTION_IDS.forEach((id) => useStrato.getState().setExpanded(id, true)));
    const check = (label: string) => {
      const copy = container.cloneNode(true) as HTMLElement;
      copy.querySelectorAll('.katex, .npseudo__code').forEach((n) => n.remove());
      const words = (copy.textContent ?? '').match(/(.{0,40})\b(planned|pending|researched|implemented|verified|validated|computed|precomputed|not verified|unavailable|not available|loading)\b(.{0,20})/gi);
      expect(words ?? [], label).toEqual([]);
      expect(container.querySelectorAll('.badge, .bstatus, .status-chip, [data-status]'), label).toHaveLength(0);
    };
    for (const physics of PHYSICS_IDS) {
      for (const scenario of SCENARIO_IDS) {
        for (const graph of GRAPH_IDS) {
          act(() => useStrato.getState().setSelection({ physics, scenario, graph }));
          check(`${physics}/${scenario}/${graph}`);
        }
      }
    }
  });

  it('holds none of the removed controls: no live simulation, sliders, layer, grid, time-step, concentration, duration or output-step input, particle view, solver status or hash', () => {
    const { container } = render(<Explore />);
    act(() => SECTION_IDS.forEach((id) => useStrato.getState().setExpanded(id, true)));
    for (const id of ['block-d', 'block-e', 'controls', 'controls-block', 'sim-strip', 'particle', 'solver-status-line', 'strip-run', 'open-panel', 'unsupported-simulation']) expect(screen.queryByTestId(id), id).toBeNull();
    expect(container.querySelectorAll('input[type="range"], input[type="number"], input[type="text"], [role="slider"], [role="switch"]')).toHaveLength(0);
    expect(container.textContent ?? '').not.toMatch(/Live simulation|Numerical controls|Solver:|commit|\b[0-9a-f]{7,40}\b/);
  });

  it('shows the four areas, and the legend directly under the equation', () => {
    const { container } = render(<Explore />);
    expect(Array.from(container.querySelectorAll('[data-area]')).map((a) => (a as HTMLElement).dataset.area)).toEqual(['controls', 'equation', 'numerical', 'graphs']);
    expect(container.querySelectorAll('h1')).toHaveLength(1);
    expect(screen.getByTestId('graph-panel').dataset.empty).toBe('true');
  });
});

describe('a reply of the wrapper for another selection', () => {
  it('is dropped: only the reply for the request of the selection is shown', async () => {
    const a = stageRequest('baseline', SR, 1600);
    const b = stageRequest('baseline', SR, 1700);
    if (!a || !b) throw new Error('no request');
    act(() => useStrato.getState().setSolver({ state: 'ready' }));
    const { result, rerender } = renderHook(({ req }) => useStageResult(req), { initialProps: { req: a } });
    await waitFor(() => expect(pending.stage).toHaveLength(1));
    rerender({ req: b });
    await waitFor(() => expect(pending.stage).toHaveLength(2));
    // The reply for the first request arrives late, then the reply for the second.
    await act(async () => {
      pending.stage[0]?.({ body: reply(1) });
    });
    expect(result.current).toBeNull();
    await act(async () => {
      pending.stage[1]?.({ body: reply(2) });
    });
    await waitFor(() => expect(result.current?.key).toBe(b.key));
    expect(result.current?.reply.series.times_h).toEqual([0, 2]);
    expect(result.current?.analytical).toBeNull();
    // A change of the selection removes the result at once: nothing of the earlier selection stays on screen.
    rerender({ req: a });
    expect(result.current).toBeNull();
  });

  it('asks for Method B only where the stage reply says it applies, and shows its values as the reply gives them', async () => {
    const a = stageRequest('baseline', SR, 1600);
    if (!a) throw new Error('no request');
    act(() => useStrato.getState().setSolver({ state: 'ready' }));
    const { result } = renderHook(() => useStageResult(a));
    await waitFor(() => expect(pending.stage).toHaveLength(1));
    const applicable = { ...reply(3), analytical: { applicable: true, extension: 'baseline', reason_code: null, reason: null } };
    await act(async () => {
      pending.stage[0]?.({ body: applicable });
    });
    await waitFor(() => expect(pending.analytical).toHaveLength(1));
    const b: AnalyticalReply = { applicable: true, extension: 'baseline', reason_code: null, reason: null, times_s: [0, 3], times_h: [0, 3], release_fraction: [0, null] };
    await act(async () => {
      pending.analytical[0]?.({ body: b });
    });
    await waitFor(() => expect(result.current).not.toBeNull());
    expect(result.current?.analytical).toBe(b);
    expect(result.current?.analytical?.release_fraction?.[1]).toBeNull();
  });
});
