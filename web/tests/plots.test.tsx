// Web phase 5: the scientific graphs. Every plotted array is the array the Rust wrapper (or a
// precomputed native file) returned, the same object and unchanged; the difference curve is the series
// compare_v1 returned, never subtracted here (owner decision M4); downloads reproduce the plotted values
// exactly; the comparison and validation layers are independent. The run fixture is the native wrapper
// response written by scripts/web/generate_data.py precomputed.
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeEach, describe, expect, it } from 'vitest';
import {
  comparisonColumns,
  csvText,
  exportMeta,
  numberText,
  parseCsv,
  runProfileColumns,
  runTimeseriesColumns,
  seriesAtRunTimes,
} from '../src/downloads/serialise';
import { LOG_DECADES } from '../src/particle/colormap';
import { LiveDownloads } from '../src/panels/Downloads';
import { filterRows, parameterRows } from '../src/panels/Provenance';
import {
  codeBCDifferenceFigure,
  compareFigure,
  convergenceFigure,
  layerBands,
  profileFigure,
  releaseFigure,
  residualFigure,
  type ComparisonDoc,
  type ConvergenceDoc,
  type GuidesDoc,
  type SyntheticRunDoc,
} from '../src/plots/figures';
import type { GraphDiv } from '../src/plots/plotly';
import { RadialProfile, ReleaseCurves, SimPlots } from '../src/plots/SimPlots';
import { readPlotTheme } from '../src/plots/theme';
import { Validate } from '../src/sections/Validate';
import { DEFAULT_CASE } from '../src/data/case';
import type { CompareOk, RunOk } from '../src/solver/types';
import { initialState, inputKey, useStrato } from '../src/state/store';

const data = (f: string) => JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', f), 'utf8')) as unknown;
const RUN = (data('wrapper_native_synthetic.json') as { data: RunOk }).data;
const COMPARISON = data('comparison_fv_vs_analytical.json') as ComparisonDoc;
const CONVERGENCE = data('convergence_study.json') as ConvergenceDoc;
const GUIDES = data('convergence_guides.json') as GuidesDoc;
const SYNTHETIC = data('synthetic_run.json') as SyntheticRunDoc;
const THEME = readPlotTheme();

/** A compare_v1 reply shaped as the wrapper returns it; the difference array is a stand-in object whose identity is checked. */
function fakeCompare(): CompareOk {
  const modified = structuredClone(RUN);
  return {
    schema: 1,
    status: 'ok',
    label: 'COMPUTED v1.0',
    engine: RUN.engine,
    baseline: RUN,
    modified,
    difference: { times_s: RUN.times_s, release_fraction: RUN.times_s.map((_, i) => (i % 2 === 0 ? 0 : -0)) },
  };
}

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

describe('radial profile figure', () => {
  it('plots the returned cell concentrations at the returned centroids, the same arrays', () => {
    const row = 37;
    const fig = profileFigure(RUN, row, { log: true, showStart: true }, THEME);
    const now = fig.data.find((t) => t.meta?.id === 'profile');
    const start = fig.data.find((t) => t.meta?.id === 'profile-start');
    expect(now?.x).toBe(RUN.mesh.cell_centroids_m);
    expect(now?.y).toBe(RUN.profiles[row]?.c_mol_per_m3);
    expect(start?.y).toBe(RUN.profiles[0]?.c_mol_per_m3);
    expect(fig.data).toHaveLength(2);
    expect(profileFigure(RUN, row, { log: true, showStart: false }, THEME).data).toHaveLength(1);
  });

  it('draws the layer bands at the radii the solver echoed, in the layer colours', () => {
    const bands = layerBands(RUN);
    expect(bands.map((b) => b.outer_m)).toEqual(RUN.inputs.regions.map((r) => r.outer_radius_m));
    expect(bands[0]?.inner_m).toBe(0);
    bands.slice(1).forEach((b, i) => expect(b.inner_m).toBe(RUN.inputs.regions[i]?.outer_radius_m));
    const fig = profileFigure(RUN, 0, { log: false, showStart: false }, THEME);
    const rects = (fig.layout.shapes as { type: string; fillcolor?: string; x0: number; x1: number }[]).filter((s) => s.type === 'rect');
    expect(rects.map((r) => r.fillcolor)).toEqual(THEME.layers);
    expect(rects.map((r) => r.x1)).toEqual(RUN.inputs.regions.map((r) => r.outer_radius_m));
    const labels = (fig.layout.annotations as { text: string }[]).map((a) => a.text);
    expect(labels).toEqual(RUN.mesh.region_names);
  });

  it('fixes the concentration axis: six decades up to c0 (log) or 0 to c0 (linear), the particle scale', () => {
    const c0 = RUN.inputs.initial.kernel_concentration_mol_m3;
    const log = profileFigure(RUN, 5, { log: true, showStart: false }, THEME).layout.yaxis as { type: string; range: number[] };
    expect(log.type).toBe('log');
    expect(log.range[0]).toBe(Math.log10(c0) - LOG_DECADES);
    const lin = profileFigure(RUN, 5, { log: false, showStart: false }, THEME).layout.yaxis as { type: string; range: number[] };
    expect(lin.type).toBe('linear');
    expect(lin.range[0]).toBe(0);
    // The same range at every playback row, so frames compare.
    expect((profileFigure(RUN, 90, { log: true, showStart: false }, THEME).layout.yaxis as { range: number[] }).range).toEqual(log.range);
  });
});

describe('release figure', () => {
  it('plots Code B and Code C as returned, with a cursor at the playback row', () => {
    const row = 42;
    const fig = releaseFigure(RUN, row, { log: false }, THEME);
    const b = fig.data.find((t) => t.meta?.id === 'release-b');
    const c = fig.data.find((t) => t.meta?.id === 'release-c');
    const cursor = fig.data.find((t) => t.meta?.id === 'release-cursor');
    expect(b?.x).toBe(RUN.times_s);
    expect(b?.y).toBe(RUN.release_fraction);
    expect(c?.x).toBe(RUN.analytical?.times_s);
    expect(c?.y).toBe(RUN.analytical?.release_fraction);
    expect(cursor?.x).toEqual([RUN.times_s[row]]);
    expect(cursor?.y).toEqual([RUN.release_fraction[row]]);
    expect(b?.line?.dash ?? 'solid').not.toBe(c?.line?.dash);
  });

  it('marks the series validity start, and the accuracy floor on the log axis', () => {
    const a = RUN.analytical!;
    const lin = releaseFigure(RUN, 3, { log: false }, THEME);
    const shapes = lin.layout.shapes as { type: string; x1?: number; y0?: number }[];
    expect(shapes.some((s) => s.type === 'rect' && s.x1 === a.valid_from_s)).toBe(true);
    expect(shapes.some((s) => s.type === 'line' && s.y0 === a.accuracy_floor)).toBe(false);
    const log = releaseFigure(RUN, 3, { log: true }, THEME);
    expect((log.layout.shapes as { y0?: number }[]).some((s) => s.y0 === a.accuracy_floor)).toBe(true);
  });

  it('returns no pointwise difference of Code B and Code C for a live run', () => {
    const fig = releaseFigure(RUN, 10, { log: true }, THEME);
    expect(fig.data.map((t) => t.meta?.id)).toEqual(['release-b', 'release-c', 'release-cursor']);
  });
});

describe('comparison figure', () => {
  it('plots the difference series compare_v1 returned, on the secondary axis', () => {
    const reply = fakeCompare();
    const fig = compareFigure(reply, { showBaseline: true, showDifference: true }, THEME);
    const d = fig.data.find((t) => t.meta?.id === 'compare-difference');
    expect(d?.y).toBe(reply.difference.release_fraction);
    expect(d?.x).toBe(reply.difference.times_s);
    expect(d?.yaxis).toBe('y2');
    expect(fig.data.find((t) => t.meta?.id === 'compare-baseline')?.y).toBe(reply.baseline.release_fraction);
    expect(fig.data.find((t) => t.meta?.id === 'compare-modified')?.y).toBe(reply.modified.release_fraction);
    expect(fig.layout.yaxis2).toBeDefined();
  });

  it('shows the baseline and the difference only when their switches are on', () => {
    const reply = fakeCompare();
    const ids = (b: boolean, d: boolean) => compareFigure(reply, { showBaseline: b, showDifference: d }, THEME).data.map((t) => t.meta?.id);
    expect(ids(false, false)).toEqual(['compare-modified']);
    expect(ids(true, false)).toEqual(['compare-baseline', 'compare-modified']);
    expect(ids(false, true)).toEqual(['compare-modified', 'compare-difference']);
  });
});

describe('verification figures from precomputed native output', () => {
  it('plots the pointwise difference only from the native file, as written', () => {
    const fig = codeBCDifferenceFigure(COMPARISON, Number(SYNTHETIC.data.summary.analytical_accuracy_floor_abs), THEME);
    expect(COMPARISON.label).toBe('PRECOMPUTED');
    expect(fig.data[0]?.y).toBe(COMPARISON.data.abs_error);
    expect(fig.data[0]?.x).toBe(COMPARISON.data.time_s);
  });

  it('plots each refinement study and its guide line', () => {
    const p = CONVERGENCE.data.points;
    for (const study of ['spatial', 'temporal'] as const) {
      const fig = convergenceFigure(CONVERGENCE, GUIDES, study, THEME);
      const idx = p.study.flatMap((s, i) => (s === study ? [i] : []));
      expect(fig.data[0]?.y).toEqual(idx.map((i) => p.abs_error_vs_exact[i]));
      expect(fig.data[0]?.x).toEqual(idx.map((i) => (study === 'spatial' ? p.total_cells[i] : p.dt_s[i])));
      expect(fig.data[1]?.y).toBe(GUIDES.data[study].guide_abs_error);
      expect(GUIDES.data[study].slope).toBe(study === 'spatial' ? 2 : 1);
    }
    expect(CONVERGENCE.data.spatial_observed_orders).toBe('2.00,2.00,2.00,2.00');
  });

  it('keeps the guide-line file in step with the convergence study it was written from', () => {
    for (const study of ['spatial', 'temporal'] as const) {
      const g = GUIDES.data[study];
      const first = CONVERGENCE.data.points.study.indexOf(study);
      expect(g.anchor_abs_error).toBe(CONVERGENCE.data.points.abs_error_vs_exact[first]);
      expect(g.guide_abs_error[0]).toBe(g.anchor_abs_error);
    }
    expect(GUIDES.label).toBe('DISPLAY');
  });

  it('plots the native balance residual as written', () => {
    const fig = residualFigure(SYNTHETIC, THEME);
    expect(fig.data[0]?.y).toBe(SYNTHETIC.data.timeseries.balance_residual_mol);
  });
});

describe('download serialisation', () => {
  const meta = exportMeta('run_timeseries', 'COMPUTED v1.0', RUN.engine, 'abc1234', 'inputs/synthetic/synthetic_five_layer.toml', RUN.inputs, 'test', new Date('2026-11-25T00:00:00Z'));

  it('writes every number so that it reads back to the same double, negative zero and gaps included', () => {
    const values = [0, -0, 1e-300, 5e-324, -2.5e-14, RUN.release_fraction[50] ?? NaN, 1 / 3, null, 2e5];
    const text = csvText(meta, [{ name: 'v', unit: '1', values }]);
    const back = parseCsv(text).columns.v!;
    expect(back).toHaveLength(values.length);
    back.forEach((v, i) => {
      if (values[i] === null) expect(v).toBeNull();
      else expect(Object.is(v, values[i])).toBe(true);
    });
    expect(numberText(-0)).toBe('-0');
  });

  it('carries the metadata header: label, version, both commits, case file, time, inputs and units', () => {
    const cols = runTimeseriesColumns(RUN);
    const parsed = parseCsv(csvText(meta, cols));
    expect(parsed.meta.label).toBe('COMPUTED v1.0');
    expect(parsed.meta.strato_version).toBe(RUN.engine.strato_version);
    expect(parsed.meta.solver_commit).toBe(RUN.engine.commit);
    expect(parsed.meta.page_commit).toBe('abc1234');
    expect(parsed.meta.case_file).toBe('inputs/synthetic/synthetic_five_layer.toml');
    expect(parsed.meta.created_utc).toBe('2026-11-25T00:00:00Z');
    expect(JSON.parse(parsed.meta.inputs ?? 'null')).toEqual(RUN.inputs);
    expect(parsed.meta.units).toContain('release_fraction_fv=1');
    expect(parsed.meta.units).toContain('inventory_mol=mol');
  });

  it('reproduces the plotted release curves of the run exactly', () => {
    const parsed = parseCsv(csvText(meta, runTimeseriesColumns(RUN)));
    const fig = releaseFigure(RUN, 0, { log: false }, THEME);
    const b = fig.data.find((t) => t.meta?.id === 'release-b')!;
    expect(parsed.columns.time_s).toEqual([...b.x]);
    expect(parsed.columns.release_fraction_fv).toEqual([...b.y]);
    // The series column holds Code C at the times it returned, empty elsewhere.
    const a = RUN.analytical!;
    const series = parsed.columns.release_fraction_series!;
    a.times_s.forEach((t, i) => expect(series[RUN.times_s.indexOf(t)]).toBe(a.release_fraction[i]));
    expect(series.filter((v) => v !== null)).toHaveLength(a.times_s.length);
    expect(seriesAtRunTimes(RUN)[0]).toBeNull();
  });

  it('reproduces the plotted profile at every output time', () => {
    const parsed = parseCsv(csvText(meta, runProfileColumns(RUN)));
    expect(parsed.columns.centroid_m).toEqual(RUN.mesh.cell_centroids_m);
    RUN.profiles.forEach((p) => expect(parsed.columns[`c_mol_m3_at_t_s=${numberText(p.t_s)}`]).toEqual(p.c_mol_per_m3));
  });

  it('writes the comparison with the difference compare_v1 returned', () => {
    const reply = fakeCompare();
    const parsed = parseCsv(csvText(meta, comparisonColumns(reply)));
    const back = parsed.columns.difference_current_minus_baseline!;
    back.forEach((v, i) => expect(Object.is(v, reply.difference.release_fraction[i])).toBe(true));
    expect(parsed.columns.release_fraction_baseline).toEqual(reply.baseline.release_fraction);
  });
});

describe('plots in block B', () => {
  it('hand the returned arrays to Plotly and follow the playback row', async () => {
    withRun();
    render(<RadialProfile />);
    const graph = screen.getByTestId('profile-plot-graph') as GraphDiv;
    await waitFor(() => expect(graph.data).toBeDefined());
    const last = RUN.profiles.length - 1;
    expect(graph.data?.find((t) => t.meta?.id === 'profile')?.y).toBe(RUN.profiles[last]?.c_mol_per_m3);
    act(() => useStrato.getState().setPlayback({ tIndex: 12 }));
    await waitFor(() => expect(graph.data?.find((t) => t.meta?.id === 'profile')?.y).toBe(RUN.profiles[12]?.c_mol_per_m3));
    expect(screen.getByTestId('profile-time').dataset.value).toBe(String(RUN.profiles[12]?.t_s));
  });

  it('show the release fraction at the playback time as returned, with its label', async () => {
    withRun();
    render(<ReleaseCurves />);
    act(() => useStrato.getState().setPlayback({ tIndex: 20 }));
    const v = screen.getByTestId('release-cursor-value');
    expect(v.dataset.value).toBe(String(RUN.release_fraction[20]));
    expect(v.dataset.label).toBe('COMPUTED v1.0');
    expect(screen.getByTestId('release-figure').textContent).toContain('not drawn here');
  });

  it('show no plot and no label before a run', () => {
    render(<SimPlots compact={false} />);
    expect(screen.getByTestId('profile-figure').dataset.state).toBe('empty');
    expect(screen.getByTestId('release-figure').dataset.state).toBe('empty');
    expect(document.querySelector('.badge')).toBeNull();
  });

  it('show one plot at a time with a switcher on a phone', () => {
    withRun();
    render(<SimPlots compact={true} />);
    expect(screen.queryByTestId('profile-figure')).not.toBeNull();
    expect(screen.queryByTestId('release-figure')).toBeNull();
    fireEvent.click(screen.getByTestId('plot-tab-release'));
    expect(screen.queryByTestId('profile-figure')).toBeNull();
    expect(screen.queryByTestId('release-figure')).not.toBeNull();
    expect(screen.getByTestId('plot-tab-release').getAttribute('aria-selected')).toBe('true');
  });

  it('change no input and send nothing when a display option changes', () => {
    withRun();
    render(<SimPlots compact={false} />);
    const before = useStrato.getState().config;
    fireEvent.click(screen.getByTestId('profile-log'));
    fireEvent.click(screen.getByTestId('release-log'));
    expect(useStrato.getState().config).toBe(before);
    expect(useStrato.getState().view.profileLog).toBe(false);
    expect(useStrato.getState().view.releaseLog).toBe(true);
  });
});

describe('comparison and validation layers are independent', () => {
  it('changes neither layer when the other changes', () => {
    const s = useStrato.getState();
    act(() => s.setView({ showComparison: false, showDifference: true }));
    expect(useStrato.getState().view.showValidation).toBe(false);
    act(() => useStrato.getState().setView({ showValidation: true }));
    expect(useStrato.getState().view.showComparison).toBe(false);
    expect(useStrato.getState().view.showDifference).toBe(true);
  });

  it('keeps the validation switch disabled while no prediction exists, and operating it changes nothing', () => {
    render(<Validate />);
    const sw = screen.getByTestId('toggle-validation');
    expect(sw.getAttribute('aria-disabled')).toBe('true');
    expect(sw.getAttribute('aria-checked')).toBe('false');
    const before = useStrato.getState().view;
    fireEvent.click(sw);
    fireEvent.keyDown(sw, { key: 'Enter' });
    expect(useStrato.getState().view).toEqual(before);
  });
});

describe('downloads panel', () => {
  it('offers the current run once there is one, and the comparison only after a comparison', () => {
    render(<LiveDownloads />);
    expect(screen.getByTestId('dl-run').dataset.available).toBe('false');
    expect(screen.getByTestId('dl-comparison').dataset.available).toBe('false');
    withRun();
    expect(screen.getByTestId('dl-run').dataset.available).toBe('true');
    expect(screen.getByTestId('dl-run').textContent).toContain('COMPUTED v1.0');
    act(() => useStrato.getState().setComparison({ status: 'ok', reply: fakeCompare(), key: 'x' }));
    expect(screen.getByTestId('dl-comparison').dataset.available).toBe('true');
    expect(screen.getByTestId('dl-baseline').dataset.available).toBe('true');
  });
});

describe('parameter provenance', () => {
  it('lists every input with status and filters by text', () => {
    const rows = parameterRows(DEFAULT_CASE.config, DEFAULT_CASE.config, DEFAULT_CASE.provenance.notes.join(' '));
    expect(rows).toHaveLength(2 + DEFAULT_CASE.config.regions.length * 3 + 1 + 3);
    expect(rows.find((r) => r.key === 'regions[3].diffusivity_m2_s')?.file).toBe(DEFAULT_CASE.config.regions[3]?.diffusivity_m2_s);
    expect(rows.find((r) => r.key === 'regions[3].diffusivity_m2_s')?.status).toContain('not literature TRISO data');
    expect(filterRows(rows, 'sic diffusivity').map((r) => r.key)).toEqual(['regions[3].diffusivity_m2_s']);
    expect(filterRows(rows, '')).toHaveLength(rows.length);
  });
});
