import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { parseCsv } from '../src/downloads/serialise';
import { compareArrays } from '../src/solver/compare';
import { PARITY_RTOL } from '../src/solver/tolerances';
import type { CompareOk, RunOk } from '../src/solver/types';
import { expect, gotoSection, horizontalOverflow, solverRequests, test, waitForRun } from './fixtures';

// Web phase 5: the scientific graphs (prompt/TESTING_AND_VALIDATION.md Section 3, row 5). The plotted
// arrays equal the wrapper's values; the plotted difference curve is the series compare_v1 returned
// (nothing is subtracted in the browser); downloads reproduce the plotted values; the comparison and
// validation toggles are independent; every plot carries its label.

const read = (...p: string[]) => JSON.parse(readFileSync(join(process.cwd(), ...p), 'utf8')) as Record<string, unknown>;
const CASE = read('src', 'generated', 'synthetic_five_layer.case.json') as { data: { config: { regions: { diffusivity_m2_s: number }[] } } };
const NATIVE = (read('public', 'data', 'wrapper_native_synthetic.json') as { data: RunOk }).data;
const COMPARISON = read('public', 'data', 'comparison_fv_vs_analytical.json') as { data: { time_s: number[]; abs_error: (number | null)[] } };
const CONVERGENCE = read('public', 'data', 'convergence_study.json') as { data: { spatial_observed_orders: string; temporal_observed_orders: string } };

interface Drawn {
  id: string;
  x: number[];
  y: (number | null)[];
  yaxis: string;
}

/** The traces Plotly drew into a plot, read from its graph div. */
async function drawn(page: Page, testId: string): Promise<Drawn[]> {
  const graph = page.getByTestId(`${testId}-graph`);
  await expect(graph).toHaveAttribute('data-ready', 'true', { timeout: 20_000 });
  return graph.evaluate((el) =>
    ((el as unknown as { data: { meta?: { id: string }; x: number[]; y: (number | null)[]; yaxis?: string }[] }).data ?? []).map((t) => ({
      id: t.meta?.id ?? '',
      x: Array.from(t.x),
      y: Array.from(t.y),
      yaxis: t.yaxis ?? 'y',
    })),
  );
}

function trace(traces: Drawn[], id: string): Drawn {
  const t = traces.find((x) => x.id === id);
  if (!t) throw new Error(`trace ${id} not drawn (${traces.map((x) => x.id).join(', ')})`);
  return t;
}

/** Bit-for-bit equality of two arrays (Object.is per value, same length). */
function identical(a: readonly (number | null)[], b: readonly (number | null)[]): boolean {
  return a.length === b.length && a.every((v, i) => Object.is(v, b[i]));
}

/** One call of the page's own solver worker, in a second instance (as in solver.spec.ts). */
async function callWorker(page: Page, kind: string, payload: unknown): Promise<unknown> {
  const url = page.workers()[0]?.url() ?? (await page.waitForEvent('worker', { timeout: 20_000 })).url();
  return page.evaluate(
    ({ url, kind, payload }) =>
      new Promise<unknown>((resolve, reject) => {
        const w = new Worker(url, { type: 'module' });
        w.onmessage = (ev: MessageEvent<{ kind: string; ok?: boolean; body?: unknown; error?: string }>) => {
          if (ev.data.kind !== 'response') return;
          w.terminate();
          if (ev.data.ok) resolve(ev.data.body);
          else reject(new Error(ev.data.error));
        };
        w.postMessage({ id: 1, kind, payload: JSON.stringify(payload) });
      }),
    { url, kind, payload },
  );
}

async function download(page: Page, testId: string): Promise<{ name: string; text: string }> {
  const [dl] = await Promise.all([page.waitForEvent('download'), page.getByTestId(testId).click()]);
  const path = await dl.path();
  return { name: dl.suggestedFilename(), text: readFileSync(path, 'utf8') };
}

async function nav(page: Page, name: string): Promise<void> {
  await page.getByRole('navigation', { name: 'Sections' }).getByRole('link', { name }).click();
}

test.describe('block B plots', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('the profile and release plots draw the wrapper values and follow the playback row', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    const last = NATIVE.profiles.length - 1;
    const profile = await drawn(page, 'profile-plot');
    // WebAssembly against the native wrapper: Code B, the mesh and the profiles are bit-identical (Phase 3 record).
    expect(identical(trace(profile, 'profile').x, NATIVE.mesh.cell_centroids_m)).toBe(true);
    expect(identical(trace(profile, 'profile').y, NATIVE.profiles[last]!.c_mol_per_m3)).toBe(true);
    expect(identical(trace(profile, 'profile-start').y, NATIVE.profiles[0]!.c_mol_per_m3)).toBe(true);
    const release = await drawn(page, 'release-plot');
    expect(identical(trace(release, 'release-b').x, NATIVE.times_s)).toBe(true);
    expect(identical(trace(release, 'release-b').y, NATIVE.release_fraction)).toBe(true);
    const c = trace(release, 'release-c');
    const ok = [compareArrays('series t', c.x, NATIVE.analytical!.times_s, PARITY_RTOL), compareArrays('series F', c.y, NATIVE.analytical!.release_fraction, PARITY_RTOL)];
    expect(ok.every((r) => r.ok && r.sameLength), JSON.stringify(ok)).toBe(true);
    // No pointwise difference of Code B and Code C is drawn for the live run.
    expect(release.map((t) => t.id).sort()).toEqual(['release-b', 'release-c', 'release-cursor']);

    // Step back ten rows: both plots follow the playback row.
    const before = await solverRequests(page);
    await page.getByTestId('pb-slider').focus();
    for (let i = 0; i < 10; i++) await page.keyboard.press('ArrowLeft');
    const row = last - 10;
    await expect(page.getByTestId('playback')).toHaveAttribute('data-row', String(row));
    await expect.poll(async () => identical(trace(await drawn(page, 'profile-plot'), 'profile').y, NATIVE.profiles[row]!.c_mol_per_m3)).toBe(true);
    await expect.poll(async () => trace(await drawn(page, 'release-plot'), 'release-cursor').x[0]).toBe(NATIVE.times_s[row]);
    await expect(page.getByTestId('release-cursor-value')).toHaveAttribute('data-value', String(NATIVE.release_fraction[row]));
    await expect(page.getByTestId('profile-time')).toHaveAttribute('data-value', String(NATIVE.profiles[row]!.t_s));
    expect(await solverRequests(page)).toBe(before);
  });

  test('every plot shows its result label, axes with units and a caption with source and commits', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    for (const id of ['profile-figure', 'release-figure']) {
      const fig = page.getByTestId(id);
      await expect(fig.locator('.figure__head .badge')).toHaveText('COMPUTED v1.0');
      await expect(fig.locator('figcaption')).toContainText('Source: STRATO v1.0.1 Rust solver in WebAssembly, build');
      await expect(fig.locator('figcaption')).toContainText('Page build');
    }
    await expect(page.getByTestId('profile-plot').locator('.xtitle')).toHaveText('Radius r (m)');
    await expect(page.getByTestId('profile-plot').locator('.ytitle')).toContainText('mol m');
    await expect(page.getByTestId('release-plot').locator('.xtitle')).toHaveText('Time t (s)');
  });

  test('display options change the plot only: no input, no run', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    const key = await page.getByTestId('block-b').getAttribute('data-input-key');
    const before = await solverRequests(page);
    await page.getByTestId('profile-log').uncheck();
    await page.getByTestId('profile-start').uncheck();
    await page.getByTestId('release-log').check();
    await expect.poll(async () => (await drawn(page, 'profile-plot')).map((t) => t.id)).toEqual(['profile']);
    const layout = await page.getByTestId('release-plot-graph').evaluate((el) => (el as unknown as { layout: { yaxis: { type: string } } }).layout.yaxis.type);
    expect(layout).toBe('log');
    expect(await page.getByTestId('block-b').getAttribute('data-input-key')).toBe(key);
    expect(await solverRequests(page)).toBe(before);
  });

  test('a figure saved as SVG carries the source line with version and commits', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    await drawn(page, 'release-plot');
    const { name, text } = await download(page, 'release-plot-svg');
    expect(name).toMatch(/^strato-release-[0-9a-f]{7}\.svg$/);
    expect(text).toContain('<svg');
    expect(text).toContain('STRATO v1.0.1');
    expect(text).toContain('COMPUTED v1.0');
    expect(text).toContain('page build');
  });
});

test.describe('phone layout of the plots', () => {
  test.use({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true, deviceScaleFactor: 2 });

  test('one plot at a time with a switcher, and no sideways scroll', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    await expect(page.getByTestId('profile-figure')).toBeVisible();
    await expect(page.getByTestId('release-figure')).toHaveCount(0);
    await page.getByTestId('plot-tab-release').click();
    await expect(page.getByTestId('release-figure')).toBeVisible();
    await expect(page.getByTestId('profile-figure')).toHaveCount(0);
    await drawn(page, 'release-plot');
    expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
    const box = await page.getByTestId('release-plot-graph').boundingBox();
    expect(box?.width ?? 0).toBeGreaterThan(250);
    expect((box?.x ?? 0) + (box?.width ?? 0)).toBeLessThanOrEqual(390);
  });
});

test.describe('comparison', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('the plotted difference is the series compare_v1 returned; toggles are independent of validation', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    // A faster SiC: the same change the wrapper test uses (crates/strato-wasm/tests/parity.rs).
    const field = page.getByTestId('d-field-3');
    await field.fill('4e-14');
    await field.press('Tab');
    await waitForRun(page);
    await nav(page, 'Compare');
    await expect(page.getByTestId('compare-figure')).toHaveAttribute('data-state', 'ok', { timeout: 20_000 });
    await page.getByTestId('toggle-difference').check();
    await expect.poll(async () => (await drawn(page, 'compare-plot')).map((t) => t.id)).toEqual(['compare-baseline', 'compare-modified', 'compare-difference']);
    const traces = await drawn(page, 'compare-plot');

    const baseline = CASE.data.config;
    const modified = structuredClone(baseline);
    modified.regions[3]!.diffusivity_m2_s = 4e-14;
    const reply = (await callWorker(page, 'compare', {
      schema: 1,
      baseline: { schema: 1, config: baseline, mechanisms: ['diffusion_layered'] },
      modified: { schema: 1, config: modified, mechanisms: ['diffusion_layered'] },
    })) as CompareOk;
    expect(reply.status).toBe('ok');
    const d = trace(traces, 'compare-difference');
    expect(d.yaxis).toBe('y2');
    expect(identical(d.y, reply.difference.release_fraction)).toBe(true);
    expect(identical(d.x, reply.difference.times_s)).toBe(true);
    expect(identical(trace(traces, 'compare-baseline').y, reply.baseline.release_fraction)).toBe(true);
    expect(identical(trace(traces, 'compare-modified').y, reply.modified.release_fraction)).toBe(true);
    // The baseline of an unchanged case equals the native wrapper run.
    expect(identical(reply.baseline.release_fraction, NATIVE.release_fraction)).toBe(true);
    await expect(page.getByTestId('compare-final-difference')).toHaveAttribute('data-value', String(reply.difference.release_fraction.at(-1)));

    // The comparison toggles change only this figure; the validation switch stays off and disabled.
    await page.getByTestId('toggle-comparison').uncheck();
    await expect.poll(async () => (await drawn(page, 'compare-plot')).map((t) => t.id)).toEqual(['compare-modified', 'compare-difference']);
    await nav(page, 'Validate');
    const v = page.getByTestId('toggle-validation');
    await expect(v).toHaveAttribute('aria-disabled', 'true');
    await expect(v).toHaveAttribute('aria-checked', 'false');
    await v.click({ force: true });
    await v.press('Enter');
    await expect(v).toHaveAttribute('aria-checked', 'false');
    await nav(page, 'Compare');
    await expect(page.getByTestId('toggle-comparison')).not.toBeChecked();
    await expect(page.getByTestId('toggle-difference')).toBeChecked();
  });

  test('a comparison of runs with different output times is rejected, never interpolated', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    await page.getByTestId('set-baseline').click();
    const every = page.getByTestId('output-every-field');
    await every.fill('20');
    await every.press('Tab');
    await waitForRun(page);
    await nav(page, 'Compare');
    await expect(page.getByTestId('compare-problem')).toContainText('different output times', { timeout: 20_000 });
    await expect(page.getByTestId('compare-plot')).toHaveCount(0);
  });
});

test.describe('downloads', () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test('the run and comparison downloads reproduce the plotted values and carry their header', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    const release = await drawn(page, 'release-plot');
    const profile = await drawn(page, 'profile-plot');
    await nav(page, 'Data');
    const ts = await download(page, 'dl-run-time-series-csv');
    expect(ts.name).toMatch(/^strato-run-timeseries-[0-9a-f]{7}\.csv$/);
    const parsed = parseCsv(ts.text);
    expect(parsed.meta.label).toBe('COMPUTED v1.0');
    expect(parsed.meta.strato_version).toBe('1.0.1');
    expect(parsed.meta.solver_commit).toMatch(/^[0-9a-f]{7}/);
    expect(parsed.meta.page_commit).toMatch(/^[0-9a-f]{7}/);
    expect(parsed.meta.case_file).toBe('inputs/synthetic/synthetic_five_layer.toml');
    expect(parsed.meta.created_utc).toMatch(/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ$/);
    expect(JSON.parse(parsed.meta.inputs ?? 'null')).toHaveProperty('regions');
    expect(parsed.meta.units).toContain('time_s=s');
    expect(identical(parsed.columns.time_s!, trace(release, 'release-b').x)).toBe(true);
    expect(identical(parsed.columns.release_fraction_fv!, trace(release, 'release-b').y)).toBe(true);
    const series = parsed.columns.release_fraction_series!.filter((v) => v !== null);
    expect(identical(series, trace(release, 'release-c').y)).toBe(true);

    const pr = parseCsv((await download(page, 'dl-run-profiles-csv')).text);
    expect(identical(pr.columns.centroid_m!, trace(profile, 'profile').x)).toBe(true);
    const lastName = pr.names[pr.names.length - 1]!;
    expect(identical(pr.columns[lastName]!, trace(profile, 'profile').y)).toBe(true);

    const js = JSON.parse((await download(page, 'dl-run-full-reply-json')).text) as { label: string; solver_commit: string; data: RunOk; units: Record<string, string> };
    expect(js.label).toBe('COMPUTED v1.0');
    expect(identical(js.data.release_fraction, trace(release, 'release-b').y)).toBe(true);
    expect(js.units.time_s).toBe('s');

    // The comparison: offered only after the Compare section has run it.
    await expect(page.getByTestId('dl-comparison')).toHaveAttribute('data-available', 'false');
    await nav(page, 'Compare');
    await expect(page.getByTestId('compare-figure')).toHaveAttribute('data-state', 'ok', { timeout: 20_000 });
    await page.getByTestId('toggle-difference').check();
    await expect.poll(async () => (await drawn(page, 'compare-plot')).map((t) => t.id)).toContain('compare-difference');
    const traces = await drawn(page, 'compare-plot');
    await nav(page, 'Data');
    const cmp = parseCsv((await download(page, 'dl-comparison-curves-csv')).text);
    expect(cmp.meta.kind).toBe('comparison');
    expect(identical(cmp.columns.difference_current_minus_baseline!, trace(traces, 'compare-difference').y)).toBe(true);
    expect(identical(cmp.columns.release_fraction_baseline!, trace(traces, 'compare-baseline').y)).toBe(true);
    expect(identical(cmp.columns.release_fraction_current!, trace(traces, 'compare-modified').y)).toBe(true);
  });

  test('the parameter table can be searched', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'data');
    await page.getByTestId('provenance-search').fill('sic diffusivity');
    await expect(page.getByTestId('input-table').locator('tbody tr')).toHaveCount(1);
    await expect(page.getByTestId('input-table').locator('tbody tr')).toHaveAttribute('data-key', 'regions[3].diffusivity_m2_s');
    await expect(page.getByTestId('catalogue-refs').locator('li')).not.toHaveCount(0);
  });
});

test.describe('verification panel', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('plots the precomputed native output with its label and provenance, and the observed orders from the file', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'verify');
    const diff = await drawn(page, 'verify-bc-difference-plot');
    expect(identical(diff[0]!.y, COMPARISON.data.abs_error)).toBe(true);
    expect(identical(diff[0]!.x, COMPARISON.data.time_s)).toBe(true);
    for (const id of ['verify-bc-release', 'verify-bc-difference', 'verify-spatial', 'verify-temporal', 'verify-balance']) {
      await expect(page.getByTestId(id).locator('.figure__head .badge')).toHaveText('PRECOMPUTED');
      await expect(page.getByTestId(id).locator('figcaption')).toContainText('commit');
    }
    await drawn(page, 'verify-spatial-plot');
    await drawn(page, 'verify-temporal-plot');
    await expect(page.getByTestId('orders-spatial')).toHaveAttribute('data-value', CONVERGENCE.data.spatial_observed_orders);
    await expect(page.getByTestId('orders-temporal')).toHaveAttribute('data-value', CONVERGENCE.data.temporal_observed_orders);
    await expect(page.getByTestId('verify-temporal')).toContainText('Section 6.7');
    await expect(page.getByTestId('verify-temporal').locator('a', { hasText: 'Section 6.7' })).toHaveAttribute('href', /#67-observed-order-of-accuracy-and-convergence$/);
  });
});
