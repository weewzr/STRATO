// Web phase 5: the scientific graphs. Every plotted array is the array the Rust wrapper (or a
// precomputed native file) returned, the same object and unchanged; the difference curve is the series
// compare returned, never subtracted here (owner decision M4); every axis and legend entry names its
// quantity in full with its unit (decisions 30 and 52); the data files of a run are written by Rust and
// saved unchanged; the comparison and validation layers are independent. The run fixture is the native
// wrapper response (the BTCS run) written by strato-webtools precomputed.
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { parseCsv } from '../src/downloads/serialise';
import { METHOD_1, METHOD_2 } from '../src/methods';
import { LOG_DECADES } from '../src/particle/colormap';
import { LiveDownloads, parameterProvenance } from '../src/panels/Downloads';
import { filterRows, parameterRows } from '../src/panels/Provenance';
import {
  checkFigure,
  compareFigure,
  layerBands,
  methodDifferenceFigure,
  methodReleaseFigure,
  profileFigure,
  releaseFigure,
  residualFigure,
  type CheckDoc,
  type ComparisonDoc,
  type RunDoc,
} from '../src/plots/figures';
import type { GraphDiv } from '../src/plots/plotly';
import { RadialProfile, ReleaseCurves, SimPlots } from '../src/plots/SimPlots';
import { readPlotTheme } from '../src/plots/theme';
import { DOC_SECTIONS, itemsOf } from '../src/documentation';
import { cloneConfig, DEFAULT_CASE } from '../src/data/case';
import * as session from '../src/solver/session';
import type { CompareOk, RunOk } from '../src/solver/types';
import { comparisonKey, initialState, inputKey, useStrato } from '../src/state/store';

const data = (f: string) => JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', f), 'utf8')) as unknown;
const text = (f: string) => readFileSync(join(process.cwd(), 'public', 'data', f), 'utf8');
const RUN = (data('wrapper_native_synthetic.json') as { data: RunOk }).data;
const COMPARISON = data('btcs_vs_analytical.json') as ComparisonDoc;
const CHECK = data('btcs_independent_check.json') as CheckDoc;
const NATIVE_RUN = data('btcs_run.json') as RunDoc;
const THEME = readPlotTheme();

/** A compare reply shaped as the wrapper returns it; the difference array is a stand-in object whose identity is checked. */
function fakeCompare(): CompareOk {
  const modified = structuredClone(RUN);
  return {
    schema: 2,
    status: 'ok',
    label: 'COMPUTED v2.0',
    engine: RUN.engine,
    solver: RUN.solver,
    baseline: RUN,
    modified,
    difference: {
      times_s: RUN.times_s,
      release_fraction: RUN.times_s.map((_, i) => (i % 2 === 0 ? 0 : -0)),
      release_rate_mol_s: RUN.release_rate_mol_s,
      inventory_mol: RUN.inventory_mol,
      cumulative_release_mol: RUN.cumulative_release_mol,
    },
    difference_definition: 'modified minus baseline',
    summary: { max_abs_release_fraction_difference: 0, final_release_fraction_difference: 0 },
    changes: [],
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
  it('plots the returned nodal concentrations at the returned node radii, the same arrays', () => {
    const row = 37;
    const fig = profileFigure(RUN, row, { log: true, showStart: true }, THEME);
    const now = fig.data.find((t) => t.meta?.id === 'profile');
    const start = fig.data.find((t) => t.meta?.id === 'profile-start');
    expect(now?.x).toBe(RUN.grid.node_radii_m);
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
    expect(labels).toEqual(RUN.grid.layer_names);
  });

  it('fixes the concentration axis: six decades up to the largest nodal concentration (log) or 0 to it (linear)', () => {
    const top = Math.max(...RUN.max_concentration_mol_m3);
    const log = profileFigure(RUN, 5, { log: true, showStart: false }, THEME).layout.yaxis as { type: string; range: number[] };
    expect(log.type).toBe('log');
    expect(log.range[0]).toBe(Math.log10(top) - LOG_DECADES);
    const lin = profileFigure(RUN, 5, { log: false, showStart: false }, THEME).layout.yaxis as { type: string; range: number[] };
    expect(lin.type).toBe('linear');
    expect(lin.range[0]).toBe(0);
    // The same range at every playback row, so frames compare.
    expect((profileFigure(RUN, 90, { log: true, showStart: false }, THEME).layout.yaxis as { range: number[] }).range).toEqual(log.range);
  });
});

describe('release figure', () => {
  const both = { log: false, showMethod1: true, showMethod2: true };

  it('plots Method A and Method B as returned, with a cursor at the playback row', () => {
    const row = 42;
    const fig = releaseFigure(RUN, row, both, THEME);
    const b = fig.data.find((t) => t.meta?.id === 'release-b');
    const c = fig.data.find((t) => t.meta?.id === 'release-c');
    const cursor = fig.data.find((t) => t.meta?.id === 'release-cursor');
    expect(b?.name).toBe(METHOD_1);
    expect(c?.name).toBe(METHOD_2);
    expect(b?.x).toBe(RUN.times_s);
    expect(b?.y).toBe(RUN.release_fraction);
    expect(c?.x).toBe(RUN.analytical?.times_s);
    expect(c?.y).toBe(RUN.analytical?.release_fraction);
    expect(cursor?.x).toEqual([RUN.times_s[row]]);
    expect(cursor?.y).toEqual([RUN.release_fraction[row]]);
    // Decision 53: both methods are solid lines of equal width, Method A blue and Method B red.
    expect(b?.line).toEqual({ color: '#2563eb', width: 2 });
    expect(c?.line).toEqual({ color: '#dc2626', width: 2 });
  });

  it('draws no dashed or dotted line in any figure, and writes every hover line as name (unit) = value (decisions 30, 52, 53)', () => {
    const figs = [
      profileFigure(RUN, 5, { log: true, showStart: true }, THEME),
      releaseFigure(RUN, 5, { ...both, log: true }, THEME),
      compareFigure(fakeCompare(), { showBaseline: true, showDifference: true }, THEME),
      methodReleaseFigure(COMPARISON, THEME),
      methodDifferenceFigure(COMPARISON, COMPARISON.data.analytical.accuracy_floor, THEME),
      checkFigure(CHECK, THEME),
      residualFigure(NATIVE_RUN, THEME),
    ];
    for (const f of figs) {
      for (const t of f.data) {
        expect(t.line?.dash ?? 'solid', String(t.name)).toBe('solid');
        // Every line of a hover text names its quantity, then its unit in brackets, then the value.
        const lines = String(t.hovertemplate ?? '')
          .replace(/<extra>.*<\/extra>/, '')
          .split('<br>');
        for (const l of lines) expect(l, `${String(t.name)}: ${l}`).toMatch(/^[A-Z][A-Za-z -]+ \((?:s|m|mol|mol\/m³|dimensionless)\) = %\{[xy]:[^}]+\}$/);
      }
      for (const s of (f.layout.shapes ?? []) as { line?: { dash?: string } }[]) expect(s.line?.dash ?? 'solid').toBe('solid');
      for (const a of (f.layout.annotations ?? []) as { text?: string }[]) {
        if (/accuracy floor/.test(a.text ?? '')) expect(a.text).toContain('(dimensionless)');
      }
    }
  });

  it('draws only the methods that are switched on', () => {
    const ids = (m1: boolean, m2: boolean) => releaseFigure(RUN, 3, { log: false, showMethod1: m1, showMethod2: m2 }, THEME).data.map((t) => t.meta?.id);
    expect(ids(true, false)).toEqual(['release-b', 'release-cursor']);
    expect(ids(false, true)).toEqual(['release-c']);
    expect(ids(false, false)).toEqual([]);
  });

  it('names every axis in full with its unit', () => {
    const fig = releaseFigure(RUN, 3, both, THEME);
    const title = (a: unknown) => ((a as { title: { text: string } }).title.text);
    expect(title(fig.layout.xaxis)).toBe('Time (s)');
    expect(title(fig.layout.yaxis)).toBe('Release fraction (dimensionless)');
    expect(title(profileFigure(RUN, 3, { log: true, showStart: false }, THEME).layout.xaxis)).toBe('Radius (m)');
    expect(title(profileFigure(RUN, 3, { log: true, showStart: false }, THEME).layout.yaxis)).toBe('Concentration (mol/m³)');
  });

  it('marks the validity start of Method B, and its accuracy floor on the log axis', () => {
    const a = RUN.analytical!;
    const lin = releaseFigure(RUN, 3, both, THEME);
    const shapes = lin.layout.shapes as { type: string; x1?: number; y0?: number }[];
    expect(shapes.some((s) => s.type === 'rect' && s.x1 === a.valid_from_s)).toBe(true);
    expect(shapes.some((s) => s.type === 'line' && s.y0 === a.accuracy_floor)).toBe(false);
    const log = releaseFigure(RUN, 3, { ...both, log: true }, THEME);
    expect((log.layout.shapes as { y0?: number }[]).some((s) => s.y0 === a.accuracy_floor)).toBe(true);
  });

  it('returns no pointwise difference of the two methods for a live run', () => {
    const fig = releaseFigure(RUN, 10, { ...both, log: true }, THEME);
    expect(fig.data.map((t) => t.meta?.id)).toEqual(['release-b', 'release-c', 'release-cursor']);
  });
});

describe('comparison figure', () => {
  it('plots the difference series compare returned, on the secondary axis', () => {
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
    const fig = methodDifferenceFigure(COMPARISON, COMPARISON.data.analytical.accuracy_floor, THEME);
    expect(COMPARISON.label).toBe('PRECOMPUTED');
    expect(COMPARISON.solver).toBe('STRATO BTCS five-layer spherical finite difference');
    expect(fig.data[0]?.y).toBe(COMPARISON.data.abs_difference);
    expect(fig.data[0]?.x).toBe(COMPARISON.data.t_s);
    const rel = methodReleaseFigure(COMPARISON, THEME);
    expect(rel.data.map((t) => t.name)).toEqual([METHOD_1, METHOD_2]);
    expect(rel.data[0]?.y).toBe(COMPARISON.data.release_fraction_btcs);
  });

  it('carries the solver version in every precomputed header and the full names in the column headers', () => {
    for (const doc of [COMPARISON, CHECK, NATIVE_RUN]) {
      expect(doc.strato_version).toMatch(/^\d+\.\d+\.\d+$/);
      expect(doc.solver).toContain('BTCS');
      expect(doc.commit).toMatch(/^[0-9a-f]{40}$/);
    }
    for (const h of Object.values(COMPARISON.data.column_headers)) expect(h).toMatch(/\((s|dimensionless)\)$/);
    for (const h of Object.values(CHECK.data.column_headers)) expect(h).toMatch(/\((s|dimensionless|true or false)\)$/);
  });

  it('plots the independent check as written, with the observed orders about 1 in time and 2 in space', () => {
    const fig = checkFigure(CHECK, THEME);
    expect(fig.data[0]?.x).toBe(CHECK.data.t_s);
    expect(fig.data[0]?.y).toBe(CHECK.data.abs_difference);
    expect(fig.data[1]?.y).toBe(CHECK.data.limit_difference);
    for (const o of CHECK.data.time_order) expect(Math.abs(o - 1)).toBeLessThan(0.5);
    for (const o of CHECK.data.space_order) expect(Math.abs(o - 2)).toBeLessThan(0.5);
    expect(CHECK.data.passed.every(Boolean)).toBe(true);
  });

  it('plots the native balance residual and its derived bound as written', () => {
    const fig = residualFigure(NATIVE_RUN, THEME);
    expect(fig.data[0]?.y).toBe(NATIVE_RUN.data.run.series.balance_residual_mol);
    expect(fig.data[1]?.y).toBe(NATIVE_RUN.data.run.series.balance_residual_bound_mol);
  });
});

describe('data files of a run are written by Rust and saved unchanged', () => {
  const blobs: Blob[] = [];
  const requests: unknown[] = [];

  beforeEach(() => {
    blobs.length = 0;
    requests.length = 0;
    URL.createObjectURL = (b: Blob | MediaSource) => {
      blobs.push(b as Blob);
      return 'blob:test';
    };
    URL.revokeObjectURL = () => undefined;
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => undefined);
    const csv = text('btcs_run.timeseries.csv');
    vi.spyOn(session, 'solverClient').mockReturnValue({
      exportRun: (config: unknown, format: string, provenance: unknown) => {
        requests.push({ config, format, provenance });
        return Promise.resolve({ body: { status: 'ok', media_type: 'text/csv', text: csv }, elapsedMs: 1, requestId: 1 });
      },
    } as unknown as ReturnType<typeof session.solverClient>);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('asks the wrapper for the file with the source row and class of the parameters, and saves its text as it is', async () => {
    withRun();
    render(<LiveDownloads />);
    fireEvent.click(screen.getByTestId('dl-run-time-series-csv'));
    await waitFor(() => expect(blobs).toHaveLength(1));
    expect(await blobs[0]!.text()).toBe(text('btcs_run.timeseries.csv'));
    expect(requests[0]).toMatchObject({
      format: 'timeseries_csv',
      provenance: { inputFile: DEFAULT_CASE.provenance.input_file, parameterSource: 'synthetic benchmark, not literature data', parameterClass: 'synthetic benchmark' },
    });
  });

  it('says that changed values are user-supplied and carry no source class', () => {
    const changed = cloneConfig(DEFAULT_CASE.config);
    changed.regions[3]!.diffusivity_m2_s *= 2;
    expect(parameterProvenance(DEFAULT_CASE.config, DEFAULT_CASE.config).parameterClass).toBe('synthetic benchmark');
    const p = parameterProvenance(DEFAULT_CASE.config, changed);
    expect(p.parameterClass).toBe('user-supplied value, no source class recorded');
    expect(p.source).toContain('changed by the user');
  });

  it('shows a failure of the export as a message and writes no file', async () => {
    vi.spyOn(session, 'solverClient').mockReturnValue({
      exportRun: () => Promise.resolve({ body: { status: 'rejected', errors: ['the export refused this input'] }, elapsedMs: 1, requestId: 1 }),
    } as unknown as ReturnType<typeof session.solverClient>);
    withRun();
    render(<LiveDownloads />);
    fireEvent.click(screen.getByTestId('dl-run-run-json'));
    await waitFor(() => expect(screen.getByTestId('download-error').textContent).toContain('the export refused this input'));
    expect(blobs).toHaveLength(0);
  });

  it('reads back a file of the library export: provenance header, full column names, gaps', () => {
    const parsed = parseCsv(text('btcs_run.timeseries.csv'));
    expect(parsed.meta.solver_package_version).toBe(NATIVE_RUN.strato_version);
    expect(parsed.meta.parameter_class).toBe('synthetic benchmark');
    expect(parsed.names).toContain('Release fraction (dimensionless)');
    expect(parsed.names).toContain('Release rate (mol/s)');
    expect(parsed.columns['Release rate (mol/s)']![0]).toBeNull();
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

  it('show the release fraction at the playback time as returned, with its label and the two method switches', async () => {
    withRun();
    render(<ReleaseCurves />);
    act(() => useStrato.getState().setPlayback({ tIndex: 20 }));
    const v = screen.getByTestId('release-cursor-value');
    expect(v.dataset.value).toBe(String(RUN.release_fraction[20]));
    expect(v.dataset.label).toBe('COMPUTED v2.0');
    // The reason why the pointwise difference is not drawn is in Scientific Documentation (explore-figures), not under the figure.
    expect(screen.getByTestId('release-figure').textContent).not.toContain('not drawn here');
    expect(screen.getByLabelText(METHOD_1)).toBeTruthy();
    expect(screen.getByLabelText(METHOD_2)).toBeTruthy();
    fireEvent.click(screen.getByTestId('release-method-2'));
    expect(useStrato.getState().view.showMethod2).toBe(false);
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

  it('has no validation switch any more: the statement of why no prediction is compared is one entry of Scientific Documentation', () => {
    // The disabled placeholder switch of the Validate page left with that page; its reason stays, once.
    expect(itemsOf('limitations').some((i) => i.id === 'validate-limits')).toBe(true);
    expect(DOC_SECTIONS.some((s) => s.id === 'limitations')).toBe(true);
  });
});

describe('downloads panel', () => {
  it('offers the current run once there is one, and the comparison only after a comparison', () => {
    render(<LiveDownloads />);
    expect(screen.getByTestId('dl-run').dataset.available).toBe('false');
    expect(screen.getByTestId('dl-comparison').dataset.available).toBe('false');
    withRun();
    expect(screen.getByTestId('dl-run').dataset.available).toBe('true');
    expect(screen.getByTestId('dl-run').textContent).toContain('COMPUTED v2.0');
    act(() => useStrato.getState().setComparison({ status: 'ok', reply: fakeCompare(), key: 'x' }));
    expect(screen.getByTestId('dl-comparison').dataset.available).toBe('true');
    expect(screen.getByTestId('dl-baseline').dataset.available).toBe('true');
  });

  it('writes the comparison as a CSV through the wrapper export, with the baseline configuration', async () => {
    const requests: { format: string; baseline: unknown }[] = [];
    vi.spyOn(session, 'solverClient').mockReturnValue({
      exportRun: (_config: unknown, format: string, _provenance: unknown, baseline?: unknown) => {
        requests.push({ format, baseline });
        return Promise.resolve({ body: { status: 'ok', text: '# label: x\n', media_type: 'text/csv' }, elapsedMs: 1, requestId: 1 });
      },
    } as unknown as ReturnType<typeof session.solverClient>);
    vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:x');
    vi.spyOn(URL, 'revokeObjectURL').mockReturnValue(undefined);
    withRun();
    const s = useStrato.getState();
    act(() => s.setComparison({ status: 'ok', reply: fakeCompare(), key: comparisonKey(s.baselineConfig, s.config) }));
    render(<LiveDownloads />);
    // The page offers no file that JavaScript wrote: the comparison has one button, the CSV.
    expect(screen.queryByTestId('dl-comparison-comparison-reply-json')).toBeNull();
    fireEvent.click(screen.getByTestId('dl-comparison-comparison-csv'));
    await waitFor(() => expect(requests).toHaveLength(1));
    expect(requests[0]?.format).toBe('comparison_csv');
    expect(requests[0]?.baseline).toBe(useStrato.getState().baselineConfig);
  });

  it('writes no file of the inputs of a later edit while the shown results are stale', () => {
    withRun();
    render(<LiveDownloads />);
    const button = screen.getByTestId('dl-run-time-series-csv') as HTMLButtonElement;
    expect(button.disabled).toBe(false);
    const changed = cloneConfig(DEFAULT_CASE.config);
    changed.regions[3]!.diffusivity_m2_s *= 2;
    act(() => useStrato.setState({ config: changed }));
    expect((screen.getByTestId('dl-run-time-series-csv') as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByTestId('dl-run-stale').textContent).toContain('the inputs have changed since');
  });
});

describe('parameter provenance', () => {
  it('lists every input with status and filters by text', () => {
    const rows = parameterRows(DEFAULT_CASE.config, DEFAULT_CASE.config, DEFAULT_CASE.provenance.input_file);
    expect(rows).toHaveLength(2 + DEFAULT_CASE.config.regions.length * 3 + 1 + 3);
    expect(rows.find((r) => r.key === 'regions[3].diffusivity_m2_s')?.file).toBe(DEFAULT_CASE.config.regions[3]?.diffusivity_m2_s);
    expect(rows.find((r) => r.key === 'regions[3].diffusivity_m2_s')?.status).toContain('not literature TRISO data');
    expect(rows.find((r) => r.key === 'regions[3].diffusivity_m2_s')?.source).toBe(DEFAULT_CASE.provenance.input_file);
    // every parameter row has a full name and a unit: no bare symbol
    for (const r of rows) expect(r.parameter, r.key).toMatch(/[a-z]{4,}/);
    expect(filterRows(rows, 'sic diffusivity').map((r) => r.key)).toEqual(['regions[3].diffusivity_m2_s']);
    expect(filterRows(rows, '')).toHaveLength(rows.length);
  });
});
