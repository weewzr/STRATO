import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { compareArrays, compareReplies } from '../src/solver/compare';
import { PARITY_RTOL } from '../src/solver/tolerances';
import type { RunOk } from '../src/solver/types';
import { expect, gotoSection, test, waitForRun } from './fixtures';

// The BTCS solver in the browser (prompt/TESTING_AND_VALIDATION.md Section 3, row 3, and Section 2;
// test H of the directive): the WebAssembly build against the native wrapper, per value, at the rule
// of the wrapper's parity example (relative 1e-12, identical array lengths and keys; the independent
// series adds its own accuracy floor), for the default run and for the small cases of the battery
// (validate, run, the data export, build info); rejected inputs with the wrapper's own messages; and
// every value of block B equal to the wrapper's value.

const read = (...p: string[]) => JSON.parse(readFileSync(join(process.cwd(), ...p), 'utf8')) as Record<string, unknown>;
const CASE = read('src', 'generated', 'synthetic_five_layer.case.json') as { data: { config: unknown } };
const NATIVE = (read('public', 'data', 'wrapper_native_synthetic.json') as { data: RunOk }).data;
const BATTERY = (read('public', 'data', 'wrapper_native_battery.json') as { data: Record<string, { function: string; input: string; reply: unknown }> }).data;

/** The request kind of the worker for a function of the wrapper. */
const KIND: Record<string, string> = { validate: 'validate', run: 'run', compare: 'compare', export_run: 'export_run', build_info: 'build_info' };

/** URL of the solver worker that the page started. */
async function workerUrl(page: Page): Promise<string> {
  const existing = page.workers()[0];
  if (existing) return existing.url();
  const w = await page.waitForEvent('worker', { timeout: 20_000 });
  return w.url();
}

/**
 * Starts a second instance of the page's own worker and asks it for one wrapper call. The request
 * is posted at once, before the worker's hello, so this also exercises the worker's queue. `payload`
 * is JSON text for the function, or an object that is turned into JSON text.
 */
async function callWorker(page: Page, kind: string, payload: unknown): Promise<{ body: unknown; elapsed_ms: number; hello: unknown }> {
  const url = await workerUrl(page);
  const text = typeof payload === 'string' ? payload : JSON.stringify(payload);
  return page.evaluate(
    ({ url, kind, text }) =>
      new Promise<{ body: unknown; elapsed_ms: number; hello: unknown }>((resolve, reject) => {
        const w = new Worker(url, { type: 'module' });
        let hello: unknown = null;
        w.onmessage = (ev: MessageEvent<{ kind: string; ok?: boolean; body?: unknown; elapsed_ms?: number; error?: string }>) => {
          const m = ev.data;
          if (m.kind === 'hello') hello = m;
          else if (m.kind === 'response') {
            w.terminate();
            if (m.ok) resolve({ body: m.body, elapsed_ms: m.elapsed_ms ?? NaN, hello });
            else reject(new Error(m.error));
          } else reject(new Error(JSON.stringify(m)));
        };
        w.postMessage({ id: 1, kind, payload: text });
      }),
    { url, kind, text },
  );
}

function describeFailures(c: ReturnType<typeof compareReplies>): string {
  return c.failures.map((f) => `${f.pattern}: ${f.count} value(s), first ${JSON.stringify(f.first)}`).join('\n');
}

test.describe('browser parity', () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test('the WebAssembly run equals the native wrapper leaf by leaf at the parity rule', async ({ page, problems }, info) => {
    void problems;
    await gotoSection(page, 'explore');
    const { body, elapsed_ms, hello } = await callWorker(page, 'run', { schema: 2, config: CASE.data.config });
    const run = body as RunOk;
    expect(run.status).toBe('ok');
    expect(run.label).toBe('COMPUTED v2.0');
    expect(run.solver.capability).toBe('diffusion_btcs_layered');
    expect((hello as { abi: number }).abi).toBe(2);
    const cmp = compareReplies(NATIVE, run, PARITY_RTOL);
    const record = { values: cmp.compared, identical: cmp.identical, failing_patterns: cmp.failures.map((f) => f.pattern), elapsed_ms };
    await info.attach('parity.json', { body: JSON.stringify(record, null, 2), contentType: 'application/json' });
    if (process.env.PARITY_OUT) writeFileSync(process.env.PARITY_OUT, JSON.stringify(record, null, 2));
    expect(cmp.compared).toBeGreaterThan(20_000);
    expect(describeFailures(cmp), 'values outside the parity rule').toBe('');
    // The BTCS values are bit-identical between the two builds (only IEEE arithmetic); the series is not.
    expect(cmp.identical).toBeGreaterThan(cmp.compared * 0.99);
    // The grid and the release by array, for a readable report when one fails.
    expect(compareArrays('node_radii_m', run.grid.node_radii_m, NATIVE.grid.node_radii_m, PARITY_RTOL).ok).toBe(true);
    expect(compareArrays('release_fraction', run.release_fraction, NATIVE.release_fraction, PARITY_RTOL).ok).toBe(true);
  });

  for (const name of ['validate_toml', 'validate_rejected_dt', 'run_single_step', 'export_timeseries_csv', 'build_info']) {
    test(`the battery case ${name} equals the native reply at the parity rule`, async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      const c = BATTERY[name];
      expect(c, name).toBeDefined();
      const kind = KIND[c!.function];
      expect(kind, c!.function).toBeDefined();
      const { body } = await callWorker(page, kind!, c!.input);
      const cmp = compareReplies(c!.reply, body, PARITY_RTOL);
      expect(cmp.compared).toBeGreaterThan(5);
      expect(describeFailures(cmp), name).toBe('');
    });
  }

  test('a data file written by the WebAssembly module carries the provenance header', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const c = BATTERY.export_timeseries_csv!;
    const { body } = await callWorker(page, 'export_run', c.input);
    const reply = body as { status: string; text: string; media_type: string; label: string };
    expect(reply.status).toBe('ok');
    expect(reply.media_type).toBe('text/csv');
    expect(reply.label).toBe('COMPUTED v2.0');
    for (const line of ['# solver_package_version: ', '# commit: ', '# input_sha256: ', '# parameter_source: synthetic benchmark, not literature data', '# parameter_class: synthetic benchmark']) {
      expect(reply.text, line).toContain(line);
    }
    expect(reply.text).toContain('Release fraction (dimensionless)');
  });

  test('invalid and placeholder requests are rejected with the wrapper messages, and the worker keeps answering', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const cfg = CASE.data.config as { time: Record<string, number>; regions: Record<string, unknown>[] };
    const cases: [string, unknown, RegExp][] = [
      ['run', { schema: 2, config: { ...cfg, time: { ...cfg.time, dt_s: 0 } } }, /invalid time controls/],
      ['run', { schema: 2, config: { ...cfg, regions: cfg.regions.slice(0, 4) } }, /exactly five physical regions/],
      ['run', { schema: 2, config: { ...cfg, time: { ...cfg.time, dt_s: 0.001 } } }, /MAX_STEPS/],
      ['run', { schema: 2, config: { ...cfg, regions: cfg.regions.map((r, i) => (i === 0 ? { ...r, diffusivity_m2_s: 'INSERT_VALUE_HERE' } : r)) } }, /INSERT_VALUE_HERE/],
      ['run', { schema: 2, config: cfg, mechanisms: ['decay'] }, /not supported by this solver build/],
      ['run', { schema: 1, config: cfg }, /unsupported request schema/],
      ['export_run', { schema: 2, config: cfg, format: 'xlsx' }, /unknown export format/],
    ];
    for (const [kind, req, msg] of cases) {
      const body = (await callWorker(page, kind, req)).body as { status: string; errors: string[] };
      expect(body.status).toBe('rejected');
      expect(body.errors.join(' ')).toMatch(msg);
    }
    const v = (await callWorker(page, 'validate', { schema: 2, config: { ...cfg, time: { ...cfg.time, dt_s: 0 } } })).body as { ok: boolean; errors: string[] };
    expect(v.ok).toBe(false);
    expect(v.errors.length).toBeGreaterThan(0);
  });
});

test.describe('block B shows the wrapper values', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('the readout, balance, layer shares and diagnostics equal the native wrapper values', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    const v = async (id: string) => Number(await page.getByTestId(id).getAttribute('data-value'));
    const same = (got: number, want: number) => expect(compareArrays('v', [got], [want], PARITY_RTOL).ok, `${got} vs ${want}`).toBe(true);
    same(await v('release-value'), NATIVE.diagnostics.final_release_fraction);
    same(await v('bal-start'), NATIVE.balance.start_inventory_mol);
    same(await v('bal-inventory'), NATIVE.balance.final.remaining_mol);
    same(await v('bal-released'), NATIVE.balance.final.released_mol);
    same(await v('bal-residual'), NATIVE.balance.final.residual_mol);
    same(await v('bal-bound'), NATIVE.balance.final.residual_bound_mol);
    same(await v('bal-max-rel'), NATIVE.balance.max_abs_residual_relative);
    const last = NATIVE.profiles[NATIVE.profiles.length - 1]!;
    for (let i = 0; i < last.layer_inventory_mol.length; i++) {
      same(await v(`layer-inv-${i}`), last.layer_inventory_mol[i]!);
      same(await v(`layer-share-${i}`), last.layer_share![i]!);
    }
    same(await v('diag-intervals'), NATIVE.diagnostics.intervals_total);
    same(await v('diag-steps'), NATIVE.diagnostics.steps);
    same(await v('diag-dt'), NATIVE.diagnostics.dt_used_s);
    const series = NATIVE.diagnostics.analytical_comparison;
    expect(series).not.toBeNull();
    // The series-derived difference is a subtraction of two numbers near each other; allow the series floor.
    expect(Math.abs((await v('diag-bc-max')) - series!.max_abs_difference)).toBeLessThanOrEqual(PARITY_RTOL * series!.max_abs_difference + series!.accuracy_floor);
    for (const id of ['release-readout', 'balance-panel', 'diagnostics-panel']) {
      await expect(page.getByTestId(id).locator('.badge').first()).toHaveText('COMPUTED v2.0');
    }
    // The solver and its verification status are on the page.
    await expect(page.getByTestId('solver-status')).toContainText('Method 1: Finite Difference BTCS');
    await expect(page.getByTestId('solver-version')).toHaveText(NATIVE.solver.version);
    await expect(page.getByTestId('solver-status').locator('[data-status-for]')).toHaveAttribute('data-status', 'VERIFIED');
  });

  test('a rejected input shows the wrapper message and no result; a valid input brings the result back', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    const field = page.getByTestId('d-field-0');
    await field.fill('0');
    await waitForRun(page, 'rejected');
    await expect(page.getByTestId('run-rejected')).toContainText('invalid region Kernel');
    // The wrapper's own validation message is also listed beside the controls.
    await expect(page.getByTestId('validation-errors')).toContainText('invalid region Kernel');
    await expect(page.getByTestId('release-readout').locator('.badge')).toHaveCount(0);
    await expect(page.getByTestId('status-result')).toHaveText('Result: none yet');
    await field.fill('2e-12');
    await waitForRun(page);
    await expect(page.getByTestId('run-rejected')).toHaveCount(0);
    await expect(page.getByTestId('validation-errors')).toHaveCount(0);
    const f = Number(await page.getByTestId('release-value').getAttribute('data-value'));
    expect(f).toBeGreaterThan(0);
    expect(f).not.toBe(NATIVE.diagnostics.final_release_fraction);
  });
});
