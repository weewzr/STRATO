import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { compareArrays, type ArrayComparison } from '../src/solver/compare';
import { GOLDEN_RTOL, PARITY_RTOL } from '../src/solver/tolerances';
import type { RunOk } from '../src/solver/types';
import { expect, gotoSection, test, waitForRun } from './fixtures';

// Web phase 3: the verified v1.0 solver in the browser (prompt/TESTING_AND_VALIDATION.md Section 3,
// row 3, and Section 2): browser wasm against the native wrapper (relative 1e-12 per value,
// identical array lengths), browser against tests/golden/v1.0/ (regression rule), rejected inputs,
// and every value of block B equal to the wrapper's value.

const read = (...p: string[]) => JSON.parse(readFileSync(join(process.cwd(), ...p), 'utf8')) as Record<string, unknown>;
const CASE = read('src', 'generated', 'synthetic_five_layer.case.json') as { data: { config: unknown } };
const NATIVE = (read('public', 'data', 'wrapper_native_synthetic.json') as { data: RunOk }).data;
const GOLDEN = (read('public', 'data', 'golden_v1.json') as { data: Record<string, Record<string, (number | null)[]>> }).data;

/** URL of the solver worker that the page started. */
async function workerUrl(page: Page): Promise<string> {
  const existing = page.workers()[0];
  if (existing) return existing.url();
  const w = await page.waitForEvent('worker', { timeout: 20_000 });
  return w.url();
}

/**
 * Starts a second instance of the page's own worker and asks it for one wrapper call. The request
 * is posted at once, before the worker's hello, so this also exercises the worker's queue.
 */
async function callWorker(page: Page, kind: string, payload: unknown): Promise<{ body: unknown; elapsed_ms: number; hello: unknown }> {
  const url = await workerUrl(page);
  return page.evaluate(
    ({ url, kind, payload }) =>
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
        w.postMessage({ id: 1, kind, payload: JSON.stringify(payload) });
      }),
    { url, kind, payload },
  );
}

function report(results: ArrayComparison[]): string {
  return results
    .filter((r) => !r.ok)
    .map((r) => `${r.name}: ${r.failures} of ${r.count} outside, same length ${r.sameLength}, first ${JSON.stringify(r.first)}`)
    .join('\n');
}

test.describe('browser parity', () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test('the WebAssembly run equals the native wrapper per value within 1e-12, with identical lengths', async ({ page, problems }, info) => {
    void problems;
    await gotoSection(page, 'explore');
    const { body, elapsed_ms, hello } = await callWorker(page, 'run_v1', { schema: 1, config: CASE.data.config });
    const run = body as RunOk;
    expect(run.status).toBe('ok');
    expect((hello as { abi: number }).abi).toBe(2);
    const results: ArrayComparison[] = [];
    const arrays = ['times_s', 'release_fraction', 'inventory_mol', 'cumulative_release_mol', 'release_rate_mol_s', 'balance_residual_mol', 'min_concentration_mol_m3'] as const;
    for (const k of arrays) results.push(compareArrays(k, run[k], NATIVE[k], PARITY_RTOL));
    for (const k of ['cell_centroids_m', 'cell_faces_m', 'region_of_cell'] as const) results.push(compareArrays(`mesh.${k}`, run.mesh[k], NATIVE.mesh[k], PARITY_RTOL));
    expect(run.profiles.length).toBe(NATIVE.profiles.length);
    run.profiles.forEach((p, i) => {
      const n = NATIVE.profiles[i]!;
      results.push(compareArrays(`profiles[${i}].t_s`, [p.t_s], [n.t_s], PARITY_RTOL));
      results.push(compareArrays(`profiles[${i}].c`, p.c_mol_per_m3, n.c_mol_per_m3, PARITY_RTOL));
      results.push(compareArrays(`profiles[${i}].inventory`, [p.inventory_mol], [n.inventory_mol], PARITY_RTOL));
      results.push(compareArrays(`profiles[${i}].layers`, p.layer_inventory_mol, n.layer_inventory_mol, PARITY_RTOL));
      results.push(compareArrays(`profiles[${i}].share`, p.layer_share ?? [], n.layer_share ?? [], PARITY_RTOL));
    });
    const a = run.analytical;
    const na = NATIVE.analytical;
    expect(a).not.toBeNull();
    if (a && na) {
      results.push(compareArrays('analytical.times_s', a.times_s, na.times_s, PARITY_RTOL));
      results.push(compareArrays('analytical.release_fraction', a.release_fraction, na.release_fraction, PARITY_RTOL));
      results.push(compareArrays('analytical.scalars', [a.accuracy_floor, a.valid_from_s, a.modes], [na.accuracy_floor, na.valid_from_s, na.modes], PARITY_RTOL));
    }
    const b = run.balance;
    const nb = NATIVE.balance;
    results.push(
      compareArrays(
        'balance',
        [b.start_inventory_mol, b.final.t_s, b.final.inventory_mol, b.final.released_mol, b.final.residual_mol, b.max_abs_residual_mol, b.max_abs_residual_relative],
        [nb.start_inventory_mol, nb.final.t_s, nb.final.inventory_mol, nb.final.released_mol, nb.final.residual_mol, nb.max_abs_residual_mol, nb.max_abs_residual_relative],
        PARITY_RTOL,
      ),
    );
    const d = run.diagnostics;
    const nd = NATIVE.diagnostics;
    results.push(
      compareArrays(
        'diagnostics',
        [d.cells, d.steps, d.dt_used_s, d.output_rows, d.explicit_dt_bound_s, d.final_release_fraction, d.max_abs_balance_residual_mol, d.min_concentration_mol_m3, d.code_c?.max_abs_difference ?? null, d.code_c?.final_abs_difference ?? null, d.code_c?.min_flux_ratio ?? null],
        [nd.cells, nd.steps, nd.dt_used_s, nd.output_rows, nd.explicit_dt_bound_s, nd.final_release_fraction, nd.max_abs_balance_residual_mol, nd.min_concentration_mol_m3, nd.code_c?.max_abs_difference ?? null, nd.code_c?.final_abs_difference ?? null, nd.code_c?.min_flux_ratio ?? null],
        PARITY_RTOL,
      ),
    );
    // How many values are bit-identical, for the record (the requirement is the tolerance above).
    const total = results.reduce((s, r) => s + r.count, 0);
    const identical = results.reduce((s, r) => s + r.identical, 0);
    const worst = results.reduce((m, r) => (r.worstRelative > m.v ? { v: r.worstRelative, name: r.name } : m), { v: 0, name: '' });
    const record = { arrays: results.length, values: total, identical, worst_relative: worst, failed: results.filter((r) => !r.ok).map((r) => r.name), elapsed_ms };
    await info.attach('parity.json', { body: JSON.stringify(record, null, 2), contentType: 'application/json' });
    if (process.env.PARITY_OUT) writeFileSync(process.env.PARITY_OUT, JSON.stringify(record, null, 2));
    expect(report(results), 'values outside the parity tolerance').toBe('');
    expect(results.every((r) => r.ok)).toBe(true);
  });

  test('the WebAssembly run equals tests/golden/v1.0/ to the regression tolerance', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const run = (await callWorker(page, 'run_v1', { schema: 1, config: CASE.data.config })).body as RunOk;
    const ts = GOLDEN.timeseries!;
    const fp = GOLDEN.final_profile!;
    const cmp = GOLDEN.comparison_fv_vs_analytical!;
    const m0 = ts.inventory_mol![0]!;
    const conc = fp.concentration_mol_m3! as number[];
    const cmax = Math.max(...conc);
    const kept = (cmp.release_analytical ?? []).filter((v) => v !== null).map((v) => ({ v }));
    const results = [
      compareArrays('time_s', run.times_s, ts.time_s!, GOLDEN_RTOL, 1),
      compareArrays('inventory_mol', run.inventory_mol, ts.inventory_mol!, GOLDEN_RTOL, m0),
      compareArrays('cumulative_release_mol', run.cumulative_release_mol, ts.cumulative_release_mol!, GOLDEN_RTOL, m0),
      compareArrays('release_fraction', run.release_fraction, ts.release_fraction!, GOLDEN_RTOL, 1),
      compareArrays('final_profile', run.profiles[run.profiles.length - 1]!.c_mol_per_m3, conc, GOLDEN_RTOL, cmax),
      compareArrays('series', run.analytical?.release_fraction ?? [], kept.map((x) => x.v), GOLDEN_RTOL, 1),
    ];
    expect(report(results)).toBe('');
  });

  test('invalid and placeholder requests are rejected with messages, and the worker keeps answering', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const cfg = CASE.data.config as { time: Record<string, number>; regions: Record<string, unknown>[] };
    const cases: [unknown, RegExp][] = [
      [{ schema: 1, config: { ...cfg, time: { ...cfg.time, dt_s: 0 } } }, /invalid time controls/],
      [{ schema: 1, config: { ...cfg, regions: cfg.regions.slice(0, 4) } }, /exactly five physical regions/],
      [{ schema: 1, config: { ...cfg, time: { ...cfg.time, dt_s: 0.001 } } }, /MAX_STEPS/],
      [{ schema: 1, config: { ...cfg, regions: cfg.regions.map((r, i) => (i === 0 ? { ...r, diffusivity_m2_s: 'INSERT_VALUE_HERE' } : r)) } }, /INSERT_VALUE_HERE/],
      [{ schema: 1, config: cfg, mechanisms: ['decay'] }, /not supported by this solver build/],
      [{ schema: 2, config: cfg }, /unsupported request schema/],
    ];
    for (const [req, msg] of cases) {
      const body = (await callWorker(page, 'run_v1', req)).body as { status: string; errors: string[] };
      expect(body.status).toBe('rejected');
      expect(body.errors.join(' ')).toMatch(msg);
    }
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
    same(await v('bal-inventory'), NATIVE.balance.final.inventory_mol);
    same(await v('bal-released'), NATIVE.balance.final.released_mol);
    same(await v('bal-residual'), NATIVE.balance.final.residual_mol);
    same(await v('bal-max-rel'), NATIVE.balance.max_abs_residual_relative);
    const last = NATIVE.profiles[NATIVE.profiles.length - 1]!;
    for (let i = 0; i < last.layer_inventory_mol.length; i++) {
      same(await v(`layer-inv-${i}`), last.layer_inventory_mol[i]!);
      same(await v(`layer-share-${i}`), last.layer_share![i]!);
    }
    same(await v('diag-cells'), NATIVE.diagnostics.cells);
    same(await v('diag-steps'), NATIVE.diagnostics.steps);
    same(await v('diag-dt'), NATIVE.diagnostics.dt_used_s);
    same(await v('diag-bc-max'), NATIVE.diagnostics.code_c!.max_abs_difference);
    for (const id of ['release-readout', 'balance-panel', 'diagnostics-panel']) {
      await expect(page.getByTestId(id).locator('.badge').first()).toHaveText('COMPUTED v1.0');
    }
  });

  test('a rejected input shows the wrapper message and no result; a valid input brings the result back', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    const field = page.getByTestId('d-field-0');
    await field.fill('0');
    await waitForRun(page, 'rejected');
    await expect(page.getByTestId('run-rejected')).toContainText('invalid region Kernel');
    await expect(page.getByTestId('release-readout').locator('.badge')).toHaveCount(0);
    await expect(page.getByTestId('status-result')).toHaveText('Result: none yet');
    await field.fill('2e-12');
    await waitForRun(page);
    await expect(page.getByTestId('run-rejected')).toHaveCount(0);
    const f = Number(await page.getByTestId('release-value').getAttribute('data-value'));
    expect(f).toBeGreaterThan(0);
    expect(f).not.toBe(NATIVE.diagnostics.final_release_fraction);
  });
});
