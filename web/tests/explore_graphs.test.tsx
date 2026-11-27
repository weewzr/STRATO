// INTERACTIVE GRAPHS of the final Explore layout: every plotted array is a field of the wrapper's reply, element
// by element and the same object; the colours and markers of the decisions 53, 69 and 74; the temperature
// and species selectors; and the clean EMPTY panel (no text node, no curve) for every operation other than
// steady-state operation, for Future species-specific physics and wherever no result exists.
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';
import { PHYSICS_IDS, SCENARIO_IDS } from '../src/equation/capability';
import { GRAPH_IDS } from '../src/equation/selectionIds';
import { ExploreGraphs, type GraphChoices } from '../src/graph/ExploreGraphs';
import { COMPARISONS, datasetsOf, offeredTemperatures, stageRequest, type Species } from '../src/graph/exploreData';
import { experimentFigure, isBound, methodsFigure, profileFigure, releaseFigure } from '../src/graph/exploreFigures';
import { COLOURS } from '../src/graph/plotModel';
import { readPlotTheme } from '../src/plots/theme';
import type { AnalyticalReply, StageOk } from '../src/solver/types';
import { initialState, useStrato } from '../src/state/store';

const theme = readPlotTheme();
const SR: Species = { id: 'sr90', name: 'Sr-90', element: 'Sr' };
const CS: Species = { id: 'cs137', name: 'Cs-137', element: 'Cs' };

/** A reply with distinct arrays: the tests compare them with the traces by identity and by element. */
const stage: StageOk = {
  schema: 3,
  status: 'ok',
  label: 'COMPUTED v2.0',
  stage: { stage: 1, id: 'baseline', name: 'Baseline', adds: '', mechanisms: [] },
  grid: { node_radii_m: [0, 1, 2, 3, 4, 5].map((i) => i * 0.5), interface_radii_m: [0.5, 1.5, 2.5, 3.5], layer_names: ['Kernel', 'Buffer', 'IPyC', 'SiC', 'OPyC'] },
  series: { times_s: [0, 36, 72], times_h: [0, 0.01, 0.02], release_fraction: [0, 0.25, 0.5] },
  profiles: [
    { t_s: 36, t_h: 0.01, c_mol_per_m3: [5, 4, 3, 2, 1, 0], inventory_mol: 1 },
    { t_s: 72, t_h: 0.02, c_mol_per_m3: [4, 3, 2, 1, 0.5, 0], inventory_mol: 0.5 },
  ],
  analytical: { applicable: true, extension: 'baseline', reason_code: null, reason: null },
  units: {},
};
const analytical: AnalyticalReply = { applicable: true, extension: 'baseline', reason_code: null, reason: null, times_s: [0, 36, 72], times_h: [0, 0.01, 0.02], release_fraction: [0, null, 0.5] };

function choices(extra: Partial<GraphChoices> = {}): GraphChoices {
  const request = stageRequest('baseline', SR, 1600);
  return {
    species: [SR, CS],
    current: SR,
    temperatures: offeredTemperatures('Sr'),
    temperature: 1600,
    datasets: datasetsOf(SR),
    dataset: datasetsOf(SR)[0] ?? null,
    request,
    result: request ? { key: request.key, reply: stage, analytical } : null,
    ...extra,
  };
}

beforeEach(() => {
  useStrato.setState({ ...initialState() });
});

describe('the traces are the replies, element by element', () => {
  it('draws Method A of the stage reply as a solid blue line with the very arrays of the reply', () => {
    const fig = releaseFigure(stage, theme);
    expect(fig.data).toHaveLength(1);
    const t = fig.data[0];
    expect(t?.x).toBe(stage.series.times_h);
    expect(t?.y).toBe(stage.series.release_fraction);
    expect(t?.line).toMatchObject({ color: '#2563EB', dash: 'solid' });
    expect(t?.mode).toBe('lines');
    for (let i = 0; i < stage.series.times_h.length; i++) {
      expect(t?.x[i]).toBe(stage.series.times_h[i]);
      expect(t?.y[i]).toBe(stage.series.release_fraction[i]);
    }
  });

  it('draws Method B red and solid from the reply of analytical, with nulls kept as gaps, and none where the wrapper gave no value', () => {
    const fig = methodsFigure(stage, analytical, theme);
    expect(fig?.data.map((d) => d.name)).toEqual(['Method A: BTCS', 'Method B: Sturm–Liouville']);
    const b = fig?.data[1];
    expect(b?.x).toBe(analytical.times_h);
    expect(b?.y).toBe(analytical.release_fraction);
    expect(b?.y[1]).toBeNull();
    expect(b?.line).toMatchObject({ color: '#DC2626', dash: 'solid' });
    expect(methodsFigure(stage, null, theme)).toBeNull();
    expect(methodsFigure(stage, { ...analytical, applicable: false }, theme)).toBeNull();
    expect(methodsFigure(stage, { ...analytical, release_fraction: [null, null, null] }, theme)).toBeNull();
    expect(methodsFigure(stage, { ...analytical, release_fraction: null }, theme)).toBeNull();
  });

  it('draws the radial profile of Method A node by node at the times the reply gives, in blue', () => {
    const fig = profileFigure(stage, theme);
    expect(fig?.data).toHaveLength(stage.profiles.length);
    fig?.data.forEach((d, i) => {
      expect(d.x).toBe(stage.grid.node_radii_m);
      expect(d.y).toBe(stage.profiles[i]?.c_mol_per_m3);
      expect(d.line?.color).toBe(COLOURS.A);
      expect(d.name).toBe(`${String(stage.profiles[i]?.t_h)} h`);
    });
    expect(profileFigure({ ...stage, profiles: [] }, theme)).toBeNull();
  });

  it('draws the registered experimental curves: black markers, upper bounds as downward triangles, the model curve of the file', () => {
    expect(COMPARISONS.length).toBeGreaterThan(0);
    for (const file of COMPARISONS) {
      const fig = experimentFigure(file, theme);
      const a = fig.data[0];
      expect(a?.x, file.curve_id).toBe(file.model.method_a.time_h);
      expect(a?.y, file.curve_id).toBe(file.model.method_a.release_fraction);
      expect(a?.line?.color).toBe('#2563EB');
      const measured = file.experiment.points.filter((p) => !isBound(p.point_class));
      const bounds = file.experiment.points.filter((p) => isBound(p.point_class));
      const m = fig.data.find((d) => d.meta?.id === 'exp-measurements');
      const u = fig.data.find((d) => d.meta?.id === 'exp-upper-bounds');
      expect(m?.x ?? [], file.curve_id).toEqual(measured.map((p) => p.time_h));
      expect(m?.y ?? [], file.curve_id).toEqual(measured.map((p) => p.value));
      expect(u?.x ?? [], file.curve_id).toEqual(bounds.map((p) => p.time_h));
      expect(u?.y ?? [], file.curve_id).toEqual(bounds.map((p) => p.value));
      if (m) expect(m.marker).toMatchObject({ color: '#111111', symbol: 'circle' });
      if (u) expect(u.marker).toMatchObject({ color: '#111111', symbol: 'triangle-down' });
      expect(measured.length + bounds.length).toBe(file.experiment.points.length);
      // Method B only where the file holds it, with the file's own array.
      const b = fig.data.find((d) => d.meta?.id === 'exp-method-b');
      expect(Boolean(b), file.curve_id).toBe(file.model.method_b.status === 'available');
      if (b) expect(b.y).toBe(file.model.method_b.release_fraction);
      // No R squared label where the record holds no value: today none does.
      expect((fig.layout.annotations as unknown[]).length, file.curve_id).toBe(file.r2_records.filter((r) => r.status === 'available' && r.label_plain).length);
    }
  });

  it('writes an R squared label only where a record holds a value, in the colour of its method', () => {
    const file = JSON.parse(JSON.stringify(COMPARISONS[0])) as (typeof COMPARISONS)[number];
    file.r2_records = [{ method: 'A', status: 'available', label_plain: 'R² = 0.95' }, { method: 'B', status: 'unavailable', label_plain: null }];
    const ann = experimentFigure(file, theme).layout.annotations as { text: string; font: { color: string } }[];
    expect(ann).toHaveLength(1);
    expect(ann[0]).toMatchObject({ text: 'R² = 0.95', font: { color: '#2563EB' } });
  });
});

describe('the panel of the graphs', () => {
  const text = (el: Element) => {
    const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
    const out: string[] = [];
    while (walker.nextNode()) out.push(walker.currentNode.textContent ?? '');
    return out.join('');
  };

  it('is an empty frame, with no text node and no curve, for every operation except steady-state operation, whatever the model equation and the result', () => {
    render(<ExploreGraphs choices={choices()} />);
    for (const graph of GRAPH_IDS) {
      for (const scenario of SCENARIO_IDS.filter((s) => s !== 'steady')) {
        for (const physics of PHYSICS_IDS) {
          act(() => useStrato.getState().setSelection({ graph, scenario, physics }));
          const panel = screen.getByTestId('graph-panel');
          expect(panel.dataset.empty, `${graph}/${scenario}/${physics}`).toBe('true');
          expect(text(panel), `${graph}/${scenario}/${physics}`).toBe('');
          expect(panel.querySelector('svg, canvas, .js-plotly-plot')).toBeNull();
          expect(screen.queryByTestId('experiment-citation')).toBeNull();
        }
      }
    }
  });

  it('is an empty frame for Future species-specific physics, also for steady-state operation', () => {
    render(<ExploreGraphs choices={choices()} />);
    for (const graph of GRAPH_IDS) {
      act(() => useStrato.getState().setSelection({ graph, scenario: 'steady', physics: 'future' }));
      const panel = screen.getByTestId('graph-panel');
      expect(panel.dataset.empty, graph).toBe('true');
      expect(text(panel)).toBe('');
    }
  });

  it('is empty, and shows nothing of an earlier selection, where there is no result for the selection', () => {
    render(<ExploreGraphs choices={choices({ result: null })} />);
    for (const graph of ['release', 'profile', 'methods'] as const) {
      act(() => useStrato.getState().setSelection({ graph, scenario: 'steady', physics: 'baseline' }));
      expect(screen.getByTestId('graph-panel').dataset.empty).toBe('true');
      expect(text(screen.getByTestId('graph-panel'))).toBe('');
    }
    // Method A against Method B has nothing to draw where Method B gave no value.
    render(<ExploreGraphs choices={choices({ result: { key: 'k', reply: stage, analytical: null } })} />);
    act(() => useStrato.getState().setSelection({ graph: 'methods' }));
    expect(screen.getAllByTestId('graph-panel').at(-1)?.dataset.empty).toBe('true');
  });

  it('draws for steady-state operation where a result exists, and the frame is gone', () => {
    render(<ExploreGraphs choices={choices()} />);
    for (const graph of ['release', 'profile', 'methods', 'experiment'] as const) {
      act(() => useStrato.getState().setSelection({ graph, scenario: 'steady', physics: 'baseline' }));
      expect(screen.getByTestId('graph-panel').dataset.empty, graph).toBe('false');
      expect(screen.queryByTestId('graph-empty'), graph).toBeNull();
    }
  });

  it('shows the caption, the APA citation and the link beneath the experimental graph only', () => {
    render(<ExploreGraphs choices={choices()} />);
    expect(screen.queryByTestId('experiment-citation')).toBeNull();
    act(() => useStrato.getState().setSelection({ graph: 'experiment' }));
    const file = datasetsOf(SR)[0];
    expect(screen.getByTestId('experiment-caption').textContent).toBe(file?.caption.caption_text);
    expect(screen.getByTestId('experiment-apa').textContent).toBe(file?.caption.source_citation_apa);
    expect(screen.getByTestId('experiment-link').getAttribute('href')).toBe(file?.caption.source_link);
    // The experimental comparison is empty again as soon as the operation is not steady-state operation.
    act(() => useStrato.getState().setSelection({ scenario: 'startup' }));
    expect(screen.queryByTestId('experiment-citation')).toBeNull();
  });

  it('has four compact toggles, one species select, one temperature select in 100 degC steps shown in degC, and a dataset select for the experiment', async () => {
    const user = userEvent.setup();
    render(<ExploreGraphs choices={choices()} />);
    const toggles = screen.getByRole('radiogroup', { name: 'Graph' });
    expect(Array.from(toggles.querySelectorAll('[role="radio"]')).map((r) => r.textContent)).toEqual([
      'Cumulative release fraction against time',
      'Radial concentration profile',
      'Method A against Method B',
      'Experimental comparison',
    ]);
    const temp = screen.getByTestId('temperature-select') as HTMLSelectElement;
    expect(Array.from(temp.options).map((o) => o.textContent)).toEqual(['1600 °C', '1700 °C', '1800 °C']);
    expect(Array.from(temp.options).map((o) => Number(o.value) % 100)).toEqual([0, 0, 0]);
    const species = screen.getByTestId('species-select') as HTMLSelectElement;
    expect(Array.from(species.options).map((o) => o.textContent)).toEqual(['Sr-90', 'Cs-137']);
    expect(screen.queryByTestId('dataset-select')).toBeNull();
    await user.click(screen.getByTestId('graph-experiment'));
    expect(useStrato.getState().selection.graph).toBe('experiment');
    expect((screen.getByTestId('dataset-select') as HTMLSelectElement).options.length).toBe(datasetsOf(SR).length);
    await user.selectOptions(temp, '1700');
    expect(useStrato.getState().selection.temperature).toBe(1700);
    await user.selectOptions(species, 'cs137');
    expect(useStrato.getState().selection).toMatchObject({ species: 'cs137', dataset: '' });
  });
});

describe('no status word on the graphs', () => {
  it('has no badge, status or label word on the toggles, the selectors or the panel', () => {
    render(<ExploreGraphs choices={choices()} />);
    for (const graph of GRAPH_IDS) {
      act(() => useStrato.getState().setSelection({ graph }));
      expect(document.body.textContent ?? '').not.toMatch(/\b(planned|verified|computed|implemented|precomputed|validated|unavailable|not available)\b/i);
      expect(document.querySelectorAll('.badge, [data-status]')).toHaveLength(0);
    }
  });
});
