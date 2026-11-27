import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { within } from '../src/solver/compare';
import { PARITY_RTOL } from '../src/solver/tolerances';
import type { RunOk } from '../src/solver/types';
import { expect, gotoAdvanced, test, waitForRun } from './fixtures';

// Web phase 4 (prompt/TESTING_AND_VALIDATION.md Section 3, row 4; prompt/IMPLEMENTATION_ROADMAP.md
// Phase 4): the particle colours are the colour map applied to the solver's values, the displayed
// layer shares are the wrapper's layer_share values, playback frame rate, keyboard playback, the
// 3D view (keyboard rotation) and its WebGL fallback, and the AGR-1 reference particle (geometry only).

const NATIVE = (JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'wrapper_native_synthetic.json'), 'utf8')) as { data: RunOk }).data;
const dir = process.env.SHOTS_DIR ?? '';
const json = (...p: string[]) => JSON.parse(readFileSync(join(process.cwd(), ...p), 'utf8')) as unknown;
const colormaps = json('src', 'generated', 'colormaps.json') as { data: Record<'cividis' | 'viridis', { rgb8: number[][] }> };
const reference = json('src', 'generated', 'agr1_reference.json') as {
  data: { default_set: string; sets: { set_id: string; link: string | null; layers: { row_id: string; value_text: string; outer_radius_text: string; status: string }[] }[] };
};

/**
 * The expected colour, worked out here independently of the page code: the position on the scale
 * (linear c / top, or log10(c / top) / 6 + 1, clipped to [0, 1], top being the largest nodal concentration of the run) picks the nearest of the 256 entries.
 */
function expectedColour(map: 'cividis' | 'viridis', c: number, top: number, scale: 'log' | 'linear'): number[] {
  let t = c <= 0 ? 0 : scale === 'linear' ? c / top : Math.log10(c / top) / 6 + 1;
  t = Math.min(1, Math.max(0, t));
  return colormaps.data[map].rgb8[Math.round(t * 255)]!;
}

async function row(page: Page): Promise<number> {
  return Number(await page.getByTestId('playback').getAttribute('data-row'));
}

// Frame-rate measurement (prompt/TESTING_AND_VALIDATION.md Section 3, row 4: playback at 30 fps or
// more, measured in e2e). The tests tagged @perf run in the Playwright project "performance" with
// one worker, and continuous integration runs that project in its own step after the other tests,
// so no other browser shares the machine while a frame rate is measured.
const PERF_WINDOW_MS = 1000;

interface FrameRate {
  fps: number;
  frames: number;
  ms: number;
  medianIntervalMs: number;
  maxIntervalMs: number;
  idleRafHz: number;
  renderer: string;
}

/** The WebGL renderer of this browser (software or hardware), recorded with every measurement. */
async function webglRenderer(page: Page): Promise<string> {
  return page.evaluate(() => {
    const gl = document.createElement('canvas').getContext('webgl2');
    if (!gl) return 'no WebGL 2';
    const ext = gl.getExtension('WEBGL_debug_renderer_info');
    const name = String(gl.getParameter(ext ? ext.UNMASKED_RENDERER_WEBGL : gl.RENDERER));
    gl.getExtension('WEBGL_lose_context')?.loseContext();
    return name;
  });
}

/** Animation-frame callbacks per second of the idle page (the browser's own frame cadence). */
async function idleRafRate(page: Page): Promise<number> {
  return page.evaluate(
    () =>
      new Promise<number>((resolve) => {
        let n = 0;
        let start = 0;
        const step = (t: number) => {
          if (n === 0) start = t;
          n += 1;
          if (n === 61) resolve(60_000 / (t - start));
          else requestAnimationFrame(step);
        };
        requestAnimationFrame(step);
      }),
  );
}

/**
 * Plays from row 0 at 8x and measures the rate at which the view draws, from the page's own clock:
 * every change of the view's frame counter (`attr` on `testId`) is time-stamped in the page, and the
 * rate is the counter's increase over the time between the first change after the click (playback
 * has begun) and the last change inside the following PERF_WINDOW_MS.
 */
async function measurePlayback(page: Page, testId: string, attr: string): Promise<FrameRate> {
  const renderer = await webglRenderer(page);
  await page.getByTestId('pb-slider').fill('0');
  await page.getByTestId('pb-speed').selectOption('8');
  const idleRafHz = await idleRafRate(page);
  await page.getByTestId(testId).evaluate((el, a) => {
    const w = window as unknown as { __frameSamples: { t: number; n: number }[] };
    w.__frameSamples = [];
    new MutationObserver((records) => {
      const t = performance.now();
      for (let i = 0; i < records.length; i++) w.__frameSamples.push({ t, n: Number(el.getAttribute(a) ?? '0') });
    }).observe(el, { attributes: true, attributeFilter: [a] });
  }, attr);
  await page.getByTestId('pb-play').click();
  await expect.poll(() => page.evaluate(() => (window as unknown as { __frameSamples: unknown[] }).__frameSamples.length), { timeout: 10_000 }).toBeGreaterThan(0);
  await page.waitForTimeout(PERF_WINDOW_MS + 300);
  const samples = await page.evaluate(() => (window as unknown as { __frameSamples: { t: number; n: number }[] }).__frameSamples);
  const first = samples[0]!;
  const inside = samples.filter((s) => s.t <= first.t + PERF_WINDOW_MS);
  const last = inside[inside.length - 1]!;
  const intervals = inside.slice(1).map((s, i) => s.t - inside[i]!.t).sort((a, b) => a - b);
  const ms = last.t - first.t;
  return {
    fps: ms > 0 ? ((last.n - first.n) / ms) * 1000 : 0,
    frames: last.n - first.n,
    ms,
    medianIntervalMs: intervals.length ? intervals[Math.floor(intervals.length / 2)]! : NaN,
    maxIntervalMs: intervals.length ? intervals[intervals.length - 1]! : NaN,
    idleRafHz,
    renderer,
  };
}

function recordFrameRate(info: { annotations: { type: string; description?: string }[] }, name: string, r: FrameRate, extra: Record<string, unknown> = {}): void {
  const line = `${name}: ${r.fps.toFixed(1)} fps (${r.frames} frames in ${r.ms.toFixed(0)} ms; median interval ${r.medianIntervalMs.toFixed(1)} ms, longest ${r.maxIntervalMs.toFixed(1)} ms; idle animation frames ${r.idleRafHz.toFixed(1)} per s; renderer ${r.renderer}${Object.entries(extra).map(([k, v]) => `; ${k} ${String(v)}`).join('')})`;
  info.annotations.push({ type: name, description: line });
  // Printed so that the workflow log of continuous integration carries the measurement.
  process.stdout.write(`${line}\n`);
  if (dir) writeFileSync(join(dir, `${name}.json`), JSON.stringify({ ...r, ...extra }, null, 2));
}

test.describe('particle colouring and layer shares', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('the centre pixel and a buffer pixel carry the colour map applied to the solver values', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
    await waitForRun(page);
    const last = NATIVE.profiles.length - 1;
    expect(await row(page)).toBe(last);
    // Pixels read back from the canvas (the canvas is CANVAS_PX square, the mesh outer radius is half of it).
    const read = (x: number, y: number) =>
      page.getByTestId('xsec-canvas').evaluate(
        (el, p) => Array.from((el as HTMLCanvasElement).getContext('2d')!.getImageData(p.x, p.y, 1, 1).data).slice(0, 3),
        { x, y },
      );
    const size = Number(await page.getByTestId('xsec-canvas').getAttribute('width'));
    const edges = NATIVE.grid.ring_edges_m;
    const outer = edges[edges.length - 1]!;
    const top = Math.max(...NATIVE.max_concentration_mol_m3);
    for (const scale of ['log', 'linear'] as const) {
      await page.getByTestId('colour-scale').selectOption(scale);
      for (const map of ['cividis', 'viridis'] as const) {
        await page.getByTestId('colour-map').selectOption(map);
        // A pixel in the middle of a node band, away from its edges: node 0 at the centre, and node 60 (buffer).
        for (const cell of [0, 60]) {
          const mid = cell === 0 ? 0 : ((edges[cell]! + edges[cell + 1]!) / 2 / outer) * (size / 2);
          const px = await read(Math.floor(size / 2 + mid), Math.floor(size / 2));
          const c = NATIVE.profiles[last]!.c_mol_per_m3[cell]!;
          const want = expectedColour(map, c, top, scale);
          expect(px, `${scale} ${map} cell ${cell}`).toEqual(want);
        }
      }
    }
  });

  test('the displayed layer inventory and share equal the wrapper values at the playback time', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
    await waitForRun(page);
    for (const target of [NATIVE.profiles.length - 1, 30, 1]) {
      await page.getByTestId('pb-slider').fill(String(target));
      expect(await row(page)).toBe(target);
      for (let layer = 0; layer < 5; layer++) {
        await page.getByTestId(`inspect-${layer}`).click();
        const share = Number(await page.getByTestId('inspector-share').getAttribute('data-value'));
        const inv = Number(await page.getByTestId('inspector-inventory').getAttribute('data-value'));
        const p = NATIVE.profiles[target]!;
        expect(within(share, p.layer_share![layer]!, PARITY_RTOL), `share ${target} ${layer}`).toBe(true);
        expect(within(inv, p.layer_inventory_mol[layer]!, PARITY_RTOL), `inventory ${target} ${layer}`).toBe(true);
        await expect(page.getByTestId('inspector-share').locator('.badge')).toHaveText('COMPUTED v2.0');
        await page.getByTestId(`inspect-${layer}`).click();
      }
      const t = Number(await page.getByTestId('pb-time').getAttribute('data-value'));
      expect(t).toBe(NATIVE.profiles[target]!.t_s);
    }
  });
});

test.describe('playback', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('keyboard: Space plays and pauses, arrows step; nothing starts by itself', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
    await waitForRun(page);
    await page.waitForTimeout(500);
    await expect(page.getByTestId('playback')).toHaveAttribute('data-playing', 'false');
    await page.getByTestId('pb-slider').fill('10');
    await page.getByTestId('pb-play').focus();
    await page.keyboard.press('ArrowRight');
    expect(await row(page)).toBe(11);
    await page.keyboard.press('ArrowLeft');
    await page.keyboard.press('ArrowLeft');
    expect(await row(page)).toBe(9);
    await page.keyboard.press('Space');
    await expect(page.getByTestId('playback')).toHaveAttribute('data-playing', 'true');
    await page.waitForTimeout(400);
    await page.keyboard.press('Space');
    await expect(page.getByTestId('playback')).toHaveAttribute('data-playing', 'false');
    expect(await row(page)).toBeGreaterThan(9);
    // On the slider the arrow keys step through the rows natively.
    const at = await row(page);
    await page.getByTestId('pb-slider').focus();
    await page.keyboard.press('ArrowLeft');
    expect(await row(page)).toBe(at - 1);
  });

  test('frame rate of the 2D playback at 8x over 1001 output rows', { tag: '@perf' }, async ({ page, problems }, info) => {
    void problems;
    await gotoAdvanced(page);
    await waitForRun(page);
    await page.getByTestId('output-every-field').fill('1');
    await page.getByTestId('output-every-field').press('Enter');
    await expect.poll(async () => Number(await page.getByTestId('pb-slider').getAttribute('max')), { timeout: 20_000 }).toBe(1000);
    await waitForRun(page);
    const r = await measurePlayback(page, 'xsec-canvas', 'data-draws');
    recordFrameRate(info, 'fps-2d', r, { rows: 1001 });
    expect(r.ms, 'measured over most of the window').toBeGreaterThanOrEqual(0.8 * PERF_WINDOW_MS);
    expect(r.fps).toBeGreaterThanOrEqual(30);
  });
});

test.describe('reduced motion', () => {
  test.use({ viewport: { width: 1440, height: 900 }, reducedMotion: 'reduce' });

  test('playback does not start by itself and still steps', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
    await waitForRun(page);
    await page.waitForTimeout(800);
    await expect(page.getByTestId('playback')).toHaveAttribute('data-playing', 'false');
    expect(await row(page)).toBe(NATIVE.profiles.length - 1);
    await page.getByTestId('pb-back').click();
    expect(await row(page)).toBe(NATIVE.profiles.length - 2);
  });
});

test.describe('3D cutaway', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  // Rewritten for the hierarchy of 26 Nov 2026: the particle is in block D below the equation, so the fold is
  // measured with the particle box scrolled to the top of the page area; the readout and the profile are in block E.
  test('renders, rotates with the arrow keys, zooms and resets, and keeps the particle box with its controls and key help inside one screen', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
    await waitForRun(page);
    await page.getByTestId('view-3d').click();
    const c = page.getByTestId('p3d-canvas');
    await expect(c).toBeVisible({ timeout: 20_000 });
    await expect.poll(async () => Number((await c.getAttribute('data-frames')) ?? '0'), { timeout: 20_000 }).toBeGreaterThan(0);
    await expect(page.getByTestId('particle')).toHaveAttribute('data-view', '3d');
    // In the reading column of Scientific Documentation the page scrolls; the particle box and its 3D controls are measured on one screen.
    // Fold, measured on the first screen before any click can scroll the page: the 3D view takes
    // the place of the 2D drawing at no greater height, so the equation, the whole particle box
    // with its 3D controls and key help, the release readout and the top of the profile figure stay
    // above the status line.
    const scrollY0 = await page.evaluate(() => {
      const el = document.querySelector('[data-testid="particle"]');
      window.scrollTo(0, (el?.getBoundingClientRect().top ?? 0) + window.scrollY - 120);
      return window.scrollY;
    });
    const footerTop = (await page.locator('footer.status').boundingBox())?.y ?? 900;
    for (const id of ['particle', 'p3d-canvas', 'p3d-controls', 'p3d-zoom-in', 'p3d-zoom-out', 'p3d-reset', 'p3d-keys']) {
      const box = await page.getByTestId(id).boundingBox();
      expect(box, id).not.toBeNull();
      expect(box?.y ?? -1, `${id} top`).toBeGreaterThanOrEqual(0);
      expect((box?.y ?? 0) + (box?.height ?? 0), id).toBeLessThanOrEqual(footerTop + 0.5);
    }
    // The buttons lie inside the scene's box, clear of the particle at the default view (about 92 %
    // of the stage height, centred), and the 3D drawing is no taller than the 2D one.
    const canvasBox = (await c.boundingBox())!;
    for (const id of ['p3d-zoom-in', 'p3d-zoom-out', 'p3d-reset']) {
      const b = (await page.getByTestId(id).boundingBox())!;
      expect(b.x + b.width, id).toBeLessThanOrEqual(canvasBox.x + canvasBox.width + 0.5);
      expect(b.y + b.height, id).toBeLessThanOrEqual(canvasBox.y + canvasBox.height + 0.5);
      expect(b.x, id).toBeGreaterThanOrEqual(canvasBox.x + canvasBox.width / 2 + 0.46 * canvasBox.height);
    }
    const drawing3d = (await page.locator('.particle__drawing').boundingBox())!.height;
    await page.getByTestId('view-2d').click();
    await expect(page.getByTestId('xsec')).toBeVisible();
    const drawing2d = (await page.locator('.particle__drawing').boundingBox())!.height;
    expect(drawing3d).toBeLessThanOrEqual(drawing2d + 0.5);
    await page.getByTestId('view-3d').click();
    await expect(c).toBeVisible({ timeout: 20_000 });
    await expect.poll(async () => Number((await c.getAttribute('data-frames')) ?? '0'), { timeout: 20_000 }).toBeGreaterThan(0);
    expect(await page.evaluate(() => window.scrollY)).toBe(scrollY0);
    const az0 = Number(await c.getAttribute('data-azimuth'));
    await c.focus();
    await page.keyboard.press('ArrowRight');
    const az1 = Number(await c.getAttribute('data-azimuth'));
    expect(az1).toBeGreaterThan(az0);
    await page.keyboard.press('ArrowUp');
    await page.keyboard.press('-');
    await page.getByTestId('p3d-reset').click();
    expect(Number(await c.getAttribute('data-azimuth'))).toBe(az0);
    // The buttons were on screen, so using them did not scroll the page.
    expect(await page.evaluate(() => window.scrollY)).toBe(scrollY0);
    if (dir) await page.screenshot({ path: join(dir, 'explore-3d-1440.png') });
    await page.getByTestId('view-2d').click();
    await expect(page.getByTestId('xsec')).toBeVisible();
  });

  test('frame rate of the 3D playback at 8x', { tag: '@perf' }, async ({ page, problems }, info) => {
    void problems;
    await gotoAdvanced(page);
    await waitForRun(page);
    await page.getByTestId('view-3d').click();
    const c = page.getByTestId('p3d-canvas');
    await expect(c).toBeVisible({ timeout: 20_000 });
    await expect.poll(async () => Number((await c.getAttribute('data-frames')) ?? '0'), { timeout: 20_000 }).toBeGreaterThan(0);
    // One rendered frame per new row: 101 rows in 1.25 s at 8x, so at most about 80 new rows per second arrive.
    const r = await measurePlayback(page, 'p3d-canvas', 'data-frames');
    // The view draws without multisampling on a software WebGL renderer (Particle3D.tsx, softwareWebGL); recorded with the rate.
    recordFrameRate(info, 'fps-3d', r, { rows: NATIVE.profiles.length, antialias: await c.getAttribute('data-antialias') });
    expect(r.ms, 'measured over most of the window').toBeGreaterThanOrEqual(0.8 * PERF_WINDOW_MS);
    expect(r.fps).toBeGreaterThanOrEqual(30);
  });

  test('falls back to the 2D cross-section where WebGL 2 is not available', async ({ page, problems }) => {
    void problems;
    await page.addInitScript(() => {
      const orig = HTMLCanvasElement.prototype.getContext;
      HTMLCanvasElement.prototype.getContext = function (this: HTMLCanvasElement, type: string, ...rest: unknown[]) {
        if (type === 'webgl2' || type === 'webgl') return null;
        return (orig as (...a: unknown[]) => unknown).call(this, type, ...rest);
      } as typeof orig;
    });
    await gotoAdvanced(page);
    await waitForRun(page);
    await page.getByTestId('view-3d').click();
    await expect(page.getByTestId('view-note')).toContainText('WebGL 2');
    await expect(page.getByTestId('xsec')).toBeVisible();
    await expect(page.getByTestId('particle')).toHaveAttribute('data-view', '2d');
    await expect(page.getByTestId('p3d-canvas')).toHaveCount(0);
  });
});

test.describe('AGR-1 reference particle', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('shows the registered geometry with row ids, status and source, labelled as not a simulation', async ({ page, problems }) => {
    void problems;
    await gotoAdvanced(page);
    const ref = page.getByTestId('reference-particle');
    await expect(ref.getByTestId('reference-tag')).toHaveText('Registered geometry');
    await expect(ref).toContainText('not a simulation');
    await expect(ref.locator('.badge')).toHaveCount(0);
    for (const set of reference.data.sets) {
      await ref.getByTestId('reference-set').selectOption(set.set_id);
      await expect(ref).toHaveAttribute('data-set', set.set_id);
      // Every link the reader can follow is a public https URL; no register-local file note is shown.
      const hrefs = await ref.locator('a').evaluateAll((as) => as.map((a) => a.getAttribute('href') ?? ''));
      if (set.link) expect(hrefs.length, set.set_id).toBeGreaterThan(0);
      for (const h of hrefs) expect(h, `${set.set_id} href`).toMatch(/^https:\/\//);
      await expect(ref).not.toContainText('file:');
      await expect(ref).not.toContainText('.pdf');
      await expect(ref).not.toContainText('idx');
      if (set.link) await expect(ref.getByTestId('reference-link')).toHaveAttribute('href', set.link);
      else await expect(ref.getByTestId('reference-no-link')).toContainText('no verified public link in the register');
      for (const l of set.layers) {
        const tr = ref.locator(`tr[data-row="${l.row_id}"]`);
        await expect(tr).toContainText(l.value_text);
        await expect(tr).toContainText(`${l.outer_radius_text} µm`);
        await expect(tr).toContainText(l.status);
      }
    }
    await ref.getByTestId('reference-set').selectOption(reference.data.default_set);
    await expect(ref).toContainText('Morris et al. (2016)');
    await expect(page.getByTestId('simulation-case')).toContainText('Synthetic five-layer verification case');
  });
});

for (const vp of [
  { name: 'phone', width: 375, height: 812 },
  { name: 'tablet', width: 768, height: 1024 },
  { name: 'desktop', width: 1440, height: 900 },
  { name: 'wide', width: 1920, height: 1080 },
]) {
  test.describe(`particle screenshot at ${vp.width}`, () => {
    test.use({ viewport: { width: vp.width, height: vp.height } });
    test('block B renders the coloured particle without sideways scroll', async ({ page, problems }) => {
      void problems;
      await gotoAdvanced(page);
      await waitForRun(page);
      await expect(page.getByTestId('xsec')).toHaveAttribute('data-mode', 'computed');
      expect(await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth)).toBeLessThanOrEqual(0);
      if (dir) await page.getByTestId('particle-figure').screenshot({ path: join(dir, `particle-${vp.width}.png`) });
    });
  });
}
