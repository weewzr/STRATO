import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { parseCsv } from '../src/downloads/serialise';
import { compareArrays } from '../src/solver/compare';
import { PARITY_RTOL } from '../src/solver/tolerances';
import type { CompareOk, RunOk } from '../src/solver/types';
import { expect, gotoAdvanced, gotoSection, horizontalOverflow, solverRequests, test, waitForRun } from './fixtures';

// Web phase 5: the scientific graphs (prompt/TESTING_AND_VALIDATION.md Section 3, row 5). The plotted
// arrays equal the wrapper's values; the plotted difference curve is the series compare returned
// (nothing is subtracted in the browser); downloads are the files the wrapper's export wrote and
// reproduce the plotted values; the comparison and validation toggles are independent; every plot
// carries its label and names its axes in full with their units.

const read = (...p: string[]) => JSON.parse(readFileSync(join(process.cwd(), ...p), 'utf8')) as Record<string, unknown>;
const CASE = read('src', 'generated', 'synthetic_five_layer.case.json') as { data: { config: { regions: { diffusivity_m2_s: number }[] } } };
const NATIVE = (read('public', 'data', 'wrapper_native_synthetic.json') as { data: RunOk }).data;
const COMPARISON = read('public', 'data', 'btcs_vs_analytical.json') as { data: { t_s: number[]; abs_difference: (number | null)[] } };
const CHECK = read('public', 'data', 'btcs_independent_check.json') as { data: { time_order: number[]; space_order: number[] } };

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

/** The pages that hold what the old Compare, Validate, Verify and Data sections held: entries of Scientific Documentation. */
const ENTRY: Record<string, string> = { Compare: 'evidence-comparison', Validate: 'validate-status', Verify: 'evidence-verification', Data: 'evidence-downloads' };

async function nav(page: Page, name: string): Promise<void> {
  await page.goto(`./#/documentation/${ENTRY[name] ?? ''}`);
  await expect(page.locator('[data-section="documentation"] h1')).toBeVisible();
}

test.describe('block B plots', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('the profile and release plots draw the wrapper values and follow the playback row', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
    await waitForRun(page);
    const last = NATIVE.profiles.length - 1;
    const profile = await drawn(page, 'profile-plot');
    // WebAssembly against the native wrapper: Method A, the grid and the profiles are bit-identical.
    expect(identical(trace(profile, 'profile').x, NATIVE.grid.node_radii_m)).toBe(true);
    expect(identical(trace(profile, 'profile').y, NATIVE.profiles[last]!.c_mol_per_m3)).toBe(true);
    expect(identical(trace(profile, 'profile-start').y, NATIVE.profiles[0]!.c_mol_per_m3)).toBe(true);
    const release = await drawn(page, 'release-plot');
    expect(identical(trace(release, 'release-b').x, NATIVE.times_s)).toBe(true);
    expect(identical(trace(release, 'release-b').y, NATIVE.release_fraction)).toBe(true);
    const c = trace(release, 'release-c');
    const ok = [compareArrays('series t', c.x, NATIVE.analytical!.times_s, PARITY_RTOL), compareArrays('series F', c.y, NATIVE.analytical!.release_fraction, PARITY_RTOL)];
    expect(ok.every((r) => r.ok && r.sameLength), JSON.stringify(ok)).toBe(true);
    // No pointwise difference of the two methods is drawn for the live run.
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

  test('every plot shows its result label, axes named in full with units and a caption with source and commits', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
    await waitForRun(page);
    for (const id of ['profile-figure', 'release-figure']) {
      const fig = page.getByTestId(id);
      await expect(fig.locator('.figure__head .badge')).toHaveText('COMPUTED v2.0');
      // One sentence under the figure; the source line (solver version and build) is in the exported file and in Scientific Documentation.
      await expect(fig.locator('figcaption')).not.toContainText('Page build');
      expect(((await fig.locator('figcaption').innerText()) ?? '').length).toBeLessThanOrEqual(220);
    }
    await expect(page.getByTestId('profile-plot').locator('.xtitle')).toHaveText('Radius (m)');
    await expect(page.getByTestId('profile-plot').locator('.ytitle')).toHaveText('Concentration (mol/m³)');
    await expect(page.getByTestId('release-plot').locator('.xtitle')).toHaveText('Time (s)');
    await expect(page.getByTestId('release-plot').locator('.ytitle')).toHaveText('Release fraction (dimensionless)');
    // The legend of the release plot names the two methods.
    await expect(page.getByTestId('release-plot').locator('.legend')).toContainText('Method A: BTCS');
    await expect(page.getByTestId('release-plot').locator('.legend')).toContainText('Method B: Sturm–Liouville');
  });

  test('display options change the plot only: no input, no run', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
    await waitForRun(page);
    const key = await page.getByTestId('block-d').getAttribute('data-input-key');
    const before = await solverRequests(page);
    await page.getByTestId('profile-log').uncheck();
    await page.getByTestId('profile-start').uncheck();
    await page.getByTestId('release-log').check();
    await expect.poll(async () => (await drawn(page, 'profile-plot')).map((t) => t.id)).toEqual(['profile']);
    await expect
      .poll(() => page.getByTestId('release-plot-graph').evaluate((el) => (el as unknown as { layout: { yaxis: { type: string } } }).layout.yaxis.type))
      .toBe('log');
    // Switching Method B off removes its curve only.
    await page.getByTestId('release-method-2').uncheck();
    await expect.poll(async () => (await drawn(page, 'release-plot')).map((t) => t.id).sort()).toEqual(['release-b', 'release-cursor']);
    expect(await page.getByTestId('block-d').getAttribute('data-input-key')).toBe(key);
    expect(await solverRequests(page)).toBe(before);
  });

  test('a figure saved as SVG carries the source line with the method, version and commits', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
    await waitForRun(page);
    await drawn(page, 'release-plot');
    const { name, text } = await download(page, 'release-plot-svg');
    expect(name).toMatch(/^strato-release-[0-9a-f]{7}\.svg$/);
    expect(text).toContain('<svg');
    expect(text).toContain('Method A: BTCS');
    expect(text).toContain('COMPUTED v2.0');
    expect(text).toContain('page build');
  });
});

test.describe('phone layout of the plots', () => {
  test.use({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true, deviceScaleFactor: 2 });

  test('one plot at a time with a switcher, and no sideways scroll', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
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

  test('the plotted difference is the series compare returned; the toggles keep their state', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
    await waitForRun(page);
    // A faster SiC: the same change the wrapper's compare test uses.
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
      schema: 2,
      baseline: { schema: 2, config: baseline, mechanisms: ['diffusion_btcs_layered'] },
      modified: { schema: 2, config: modified, mechanisms: ['diffusion_btcs_layered'] },
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
    await expect(page.getByTestId('toggle-validation')).toHaveCount(0);
    await nav(page, 'Compare');
    await expect(page.getByTestId('toggle-comparison')).not.toBeChecked();
    await expect(page.getByTestId('toggle-difference')).toBeChecked();
  });

  test('a comparison of runs with different output times is rejected, never interpolated', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
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

  test('the run downloads are written by the wrapper, carry their provenance header and reproduce the plotted values', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
    await waitForRun(page);
    const release = await drawn(page, 'release-plot');
    const profile = await drawn(page, 'profile-plot');
    const shownBound = Number(await page.getByTestId('bal-bound').getAttribute('data-value'));
    await nav(page, 'Data');
    const ts = await download(page, 'dl-run-time-series-csv');
    expect(ts.name).toMatch(/^strato-run-timeseries-[0-9a-f]{7}\.csv$/);
    const parsed = parseCsv(ts.text);
    // The header is the library's provenance header: solver, version, commit, input digest, parameter source and class.
    expect(parsed.meta.label).toBe('numerical solution of the stated model; not experimental validation');
    expect(parsed.meta.solver_package_version).toBe('2.1.0');
    expect(parsed.meta.commit).toMatch(/^[0-9a-f]{7}/);
    expect(parsed.meta.input_file).toBe('synthetic_five_layer.toml');
    expect(parsed.meta.input_sha256).toMatch(/^[0-9a-f]{64}$/);
    expect(parsed.meta.parameter_source).toBe('synthetic benchmark, not literature data');
    expect(parsed.meta.parameter_class).toBe('synthetic benchmark');
    // The columns carry the full physical name with the unit.
    expect(parsed.names).toContain('Time (s)');
    expect(parsed.names).toContain('Release fraction (dimensionless)');
    expect(identical(parsed.columns['Time (s)']!, trace(release, 'release-b').x)).toBe(true);
    expect(identical(parsed.columns['Release fraction (dimensionless)']!, trace(release, 'release-b').y)).toBe(true);
    // The rounding bound of the mass-balance residual on the page is the value in the file: one quantity, one value.
    const bounds = parsed.columns['Derived a-posteriori rounding bound of the mass-balance residual (mol)']!;
    expect(bounds[bounds.length - 1]).toBe(shownBound);

    const pr = parseCsv((await download(page, 'dl-run-profiles-csv')).text);
    const times = pr.columns['Time (s)']!;
    const lastTime = times[times.length - 1];
    const rows = times.map((t, i) => (t === lastTime ? i : -1)).filter((i) => i >= 0);
    expect(identical(rows.map((i) => pr.columns['Concentration (mol/m^3)']![i]!), trace(profile, 'profile').y)).toBe(true);
    expect(identical(rows.map((i) => pr.columns['Radius (m)']![i]!), trace(profile, 'profile').x)).toBe(true);

    const js = JSON.parse((await download(page, 'dl-run-run-json')).text) as { provenance: { solver_package_version?: string; package_version?: string; parameter_class: string }; label: string; series: { release_fraction: number[] } };
    expect(js.provenance.parameter_class).toBe('synthetic benchmark');
    expect(js.label).toContain('not experimental validation');
    expect(identical(js.series.release_fraction, trace(release, 'release-b').y)).toBe(true);

    // The comparison: offered only after the Compare section has run it.
    await expect(page.getByTestId('dl-comparison')).toHaveAttribute('data-available', 'false');
    await nav(page, 'Compare');
    await expect(page.getByTestId('compare-figure')).toHaveAttribute('data-state', 'ok', { timeout: 20_000 });
    await page.getByTestId('toggle-difference').check();
    await expect.poll(async () => (await drawn(page, 'compare-plot')).map((t) => t.id)).toContain('compare-difference');
    const traces = await drawn(page, 'compare-plot');
    await nav(page, 'Data');
    const cmp = await download(page, 'dl-comparison-comparison-csv');
    expect(cmp.name).toMatch(/^strato-comparison-[0-9a-f]{7}\.csv$/);
    const pc = parseCsv(cmp.text);
    // The file starts with the library's provenance header and carries the full names with the units.
    expect(pc.meta.label).toBe('numerical solution of the stated model; not experimental validation');
    expect(pc.meta.input_sha256).toMatch(/^[0-9a-f]{64}$/);
    expect(pc.meta.baseline_input_sha256).toMatch(/^[0-9a-f]{64}$/);
    expect(pc.meta.parameter_source).toBeTruthy();
    expect(identical(pc.columns['Release fraction of the baseline run (dimensionless)']!, trace(traces, 'compare-baseline').y)).toBe(true);
    expect(identical(pc.columns['Release fraction of the run with the current inputs (dimensionless)']!, trace(traces, 'compare-modified').y)).toBe(true);
    expect(identical(pc.columns['Release fraction of the current inputs minus release fraction of the baseline (dimensionless)']!, trace(traces, 'compare-difference').y)).toBe(true);
  });

  test('the parameter table can be searched', async ({ page, problems }) => {
    void problems;
    await page.goto('./#/documentation/evidence-parameters');
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
    await nav(page, 'Verify');
    const diff = await drawn(page, 'verify-bc-difference-plot');
    expect(identical(diff[0]!.y, COMPARISON.data.abs_difference)).toBe(true);
    expect(identical(diff[0]!.x, COMPARISON.data.t_s)).toBe(true);
    for (const id of ['verify-bc-release', 'verify-bc-difference', 'verify-check', 'verify-balance']) {
      await expect(page.getByTestId(id).locator('.figure__head .badge')).toHaveText('PRECOMPUTED');
      await expect(page.getByTestId(id).locator('figcaption')).toContainText('Figure');
      await expect(page.getByTestId(id).getByRole('link', { name: 'Data file' })).toHaveAttribute('href', /\/data\/btcs_[a-z_]+\.json$/);
    }
    // The provenance of the precomputed files moved to Scientific Documentation (commit, solver version, command).
    for (const file of ['btcs_vs_analytical.json', 'btcs_independent_check.json', 'btcs_run.json']) {
      await expect(page.getByTestId(`prov-${file}`)).toHaveCount(0);
    }
    await drawn(page, 'verify-check-plot');
    await drawn(page, 'verify-balance-plot');
    for (const [i, o] of CHECK.data.time_order.entries()) await expect(page.getByTestId(`orders-temporal-${i}`)).toHaveAttribute('data-value', String(o));
    for (const [i, o] of CHECK.data.space_order.entries()) await expect(page.getByTestId(`orders-spatial-${i}`)).toHaveAttribute('data-value', String(o));
    await expect(page.getByTestId('observed-orders')).toContainText('Observed order in time (dimensionless)');
    await expect(page.getByTestId('observed-orders')).toContainText('Observed order in space (dimensionless)');
    await expect(page.getByTestId('observed-orders')).toContainText('Release fraction by Method A: BTCS (dimensionless)');
    await expect(page.getByTestId('observed-orders')).not.toContainText('BTCSagainst');
    // Every column must be visible without sideways scrolling (the table sits in a box that scrolls).
    const fit = await page.getByTestId('observed-orders').evaluate((t) => {
      const box = t.closest('.table-box') as HTMLElement;
      return { table: t.scrollWidth, box: box.clientWidth };
    });
    expect(fit.table).toBeLessThanOrEqual(fit.box);
    await expect(page.getByTestId('verify-status')).toContainText('VERIFIED');
    await expect(page.getByTestId('verify-status')).toContainText('Method A: BTCS');
  });

  test('the provenance, scope and numbers behind the verification figures are in Scientific Documentation', async ({ page, problems }) => {
    void problems;
    await page.goto('./#/documentation/verify-files');
    for (const file of ['btcs_vs_analytical.json', 'btcs_independent_check.json', 'btcs_run.json']) {
      const row = page.getByTestId(`prov-${file}`);
      await expect(row).toContainText('2.1.0');
      await expect(row.locator('code').first()).toBeVisible();
    }
    await expect(page.getByTestId('documentation-accuracy-floor')).toContainText('accuracy floor');
    await expect(page.getByTestId('documentation-balance')).toContainText('Largest relative residual');
    await expect(page.getByTestId('documentation-scope')).toContainText('constant diffusivity');
    await expect(page.getByTestId('documentation-checks').locator('tbody tr')).toHaveCount(7);
  });
});
