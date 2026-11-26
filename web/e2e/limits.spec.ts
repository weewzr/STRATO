import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { REQUEST_TIMEOUT_MS } from '../src/solver/tolerances';
import { expect, gotoSection, test, waitForRun } from './fixtures';

// Measured browser run times of the BTCS solver at the wrapper limits MAX_TOTAL_INTERVALS, MAX_STEPS,
// MAX_PROFILE_VALUES and MAX_SERIES_MODES (prompt/TECHNICAL_ARCHITECTURE.md Section 3, defence 2,
// and Section 6). The configurations are those of crates/strato-wasm/examples/limits_timing.rs.
// Each run must finish inside the worker time-out; the times are recorded (LIMITS_OUT, if set) for
// prompt/PROGRESS.md, never typed into the page.

type Cfg = { time: Record<string, number>; regions: Record<string, unknown>[] };
const CASE = (JSON.parse(readFileSync(join(process.cwd(), 'src', 'generated', 'synthetic_five_layer.case.json'), 'utf8')) as { data: { config: Cfg } }).data.config;

const cells = (c: Cfg, n: number): Cfg => ({ ...c, regions: c.regions.map((r) => ({ ...r, cells: n })) });
const time = (c: Cfg, t: Record<string, number>): Cfg => ({ ...c, time: { ...c.time, ...t } });

const CASES: [string, Cfg][] = [
  ['default', CASE],
  ['max cells, default steps', cells(CASE, 400)],
  ['max steps, default cells', time(CASE, { dt_s: 10 })],
  ['max cells and steps', time(cells(CASE, 400), { dt_s: 10, output_every_steps: 50 })],
  ['series near its mode limit', time(cells(CASE, 20), { dt_s: 0.5, end_time_s: 10000, output_every_steps: 5 })],
  ['max profile values', time(cells(CASE, 198), { dt_s: 10, output_every_steps: 20 })],
];

async function timeRun(page: Page, url: string, config: Cfg) {
  return page.evaluate(
    ({ url, config }) =>
      new Promise<{ status: string; elapsed_ms: number; wall_ms: number; modes: number | null; rows: number | null; bytes: number }>((resolve, reject) => {
        const w = new Worker(url, { type: 'module' });
        let t0 = 0;
        w.onmessage = (ev: MessageEvent<{ kind: string; ok?: boolean; body?: Record<string, unknown>; elapsed_ms?: number; error?: string }>) => {
          const m = ev.data;
          if (m.kind === 'hello') {
            t0 = performance.now();
            w.postMessage({ id: 1, kind: 'run', payload: JSON.stringify({ schema: 2, config }) });
          } else if (m.kind === 'response') {
            const wall = performance.now() - t0;
            w.terminate();
            if (!m.ok || !m.body) return reject(new Error(m.error));
            const b = m.body as { status: string; diagnostics?: { output_rows: number }; analytical?: { modes: number } | null };
            resolve({
              status: b.status,
              elapsed_ms: m.elapsed_ms ?? NaN,
              wall_ms: wall,
              modes: b.analytical?.modes ?? null,
              rows: b.diagnostics?.output_rows ?? null,
              bytes: JSON.stringify(m.body).length,
            });
          }
        };
      }),
    { url, config },
  );
}

test.describe('wrapper limits in the browser', () => {
  test.setTimeout(180_000);

  test('every configuration at a limit finishes inside the worker time-out', async ({ page, problems }, info) => {
    void problems;
    await gotoSection(page, 'explore');
    // First interactive solve after load (Section 6 budget: under 1 s), measured from navigation.
    await waitForRun(page);
    const firstSolveMs = await page.evaluate(() => performance.now());
    const url = page.workers()[0]?.url() ?? (await page.waitForEvent('worker')).url();
    const out: Record<string, unknown>[] = [];
    for (const [name, cfg] of CASES) {
      const r = await timeRun(page, url, cfg);
      out.push({ name, ...r });
      expect(r.status, name).toBe('ok');
      expect(r.wall_ms, `${name} within the time-out`).toBeLessThan(REQUEST_TIMEOUT_MS);
    }
    const record = { measured: new Date().toISOString(), user_agent: await page.evaluate(() => navigator.userAgent), first_solve_after_navigation_ms: firstSolveMs, runs: out };
    await info.attach('limits.json', { body: JSON.stringify(record, null, 2), contentType: 'application/json' });
    const target = process.env.LIMITS_OUT;
    if (target) {
      mkdirSync(join(target, '..'), { recursive: true });
      writeFileSync(target, JSON.stringify(record, null, 2));
    }
  });
});
