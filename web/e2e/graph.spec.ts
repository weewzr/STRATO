import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import type { Locator, Page } from '@playwright/test';
import type { ComparisonFile } from '../src/graph/contract';
import { expect, gotoSection, horizontalOverflow, test } from './fixtures';

// The experimental-comparison graph in a real browser: styles read from the rendered SVG, the plotted
// arrays against the file, the R² label rule, the caption, citation and link below the graph, the
// keyboard and the phone layout. The graph is mounted through the test harness (tests/harness, the
// development server on 127.0.0.1:4433) with the fixtures; the production build has neither.

const fixture = (name: string) => JSON.parse(readFileSync(join(process.cwd(), 'tests', 'fixtures', `fixture_r2_${name}.json`), 'utf8')) as ComparisonFile;
const BOTH = fixture('both_methods');
const NEG = fixture('negative');
const BOUNDS = fixture('upper_bounds');
const ABSENT = fixture('method_b_absent');
const HARNESS = 'http://127.0.0.1:4433/STRATO/harness.html';
const SHOTS = process.env.SHOTS_DIR;

async function open(page: Page, f: string, theme = 'light'): Promise<void> {
  await page.goto(`${HARNESS}?f=${f}&theme=${theme}`);
  for (const key of f.split(',')) {
    const id = { both: BOTH, negative: NEG, bounds: BOUNDS, absent: ABSENT }[key as 'both']!.id;
    await expect(page.getByTestId(`exp-plot-${id}-graph`)).toHaveAttribute('data-ready', 'true', { timeout: 30_000 });
  }
}

const graph = (page: Page, f: ComparisonFile): Locator => page.getByTestId(`exp-plot-${f.id}-graph`);

interface Painted {
  stroke: string;
  width: string;
  dash: string;
}

/** The stroke of every drawn line and the fill of every drawn marker, as the browser painted them. */
async function paint(page: Page, f: ComparisonFile) {
  return graph(page, f).evaluate((el) => {
    const lines = Array.from(el.querySelectorAll('.scatterlayer .trace .lines path')).map((p) => {
      const s = getComputedStyle(p);
      return { stroke: s.stroke, width: s.strokeWidth, dash: s.strokeDasharray } as Painted;
    });
    const markers = Array.from(el.querySelectorAll('.scatterlayer .trace .points path')).map((p) => ({ fill: getComputedStyle(p).fill, d: p.getAttribute('d') ?? '' }));
    return { lines, markers };
  });
}

test.describe('graph in the harness, desktop', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('Method A is a blue and Method B a red solid line of equal width; observations are black markers', async ({ page, problems }) => {
    void problems;
    await open(page, 'both');
    const p = await paint(page, BOTH);
    expect(p.lines).toHaveLength(2);
    expect(p.lines[0]?.stroke).toBe('rgb(37, 99, 235)');
    expect(p.lines[1]?.stroke).toBe('rgb(220, 38, 38)');
    expect(p.lines[0]?.width).toBe(p.lines[1]?.width);
    expect(p.lines[0]?.width).toBe('2px');
    for (const l of p.lines) expect(['none', '']).toContain(l.dash);
    expect(p.markers).toHaveLength(BOTH.observations[0]!.t.length);
    for (const m of p.markers) expect(m.fill).toBe('rgb(17, 17, 17)');
  });

  test('upper bounds are downward triangles, measurements circles', async ({ page, problems }) => {
    void problems;
    await open(page, 'bounds,both');
    const tri = (await paint(page, BOUNDS)).markers;
    const dot = (await paint(page, BOTH)).markers;
    expect(tri).toHaveLength(BOUNDS.observations[0]!.t.length);
    for (const m of tri) {
      expect(m.fill).toBe('rgb(17, 17, 17)');
      expect(m.d).not.toMatch(/a/i);
    }
    for (const m of dot) expect(m.d).toMatch(/a/i);
    // The first point of a downward triangle lies above its other two corners (SVG y grows downwards).
    const ys = Array.from(tri[0]!.d.matchAll(/(-?[\d.]+),(-?[\d.]+)/g)).map((m) => Number(m[2]));
    expect(ys.length).toBeGreaterThanOrEqual(2);
    expect(ys[ys.length - 1]).toBeGreaterThan(ys[0]!);
  });

  test('every drawn trace equals the arrays of the file, element by element', async ({ page, problems }) => {
    void problems;
    await open(page, 'both,negative,bounds,absent');
    for (const f of [BOTH, NEG, BOUNDS, ABSENT]) {
      const traces = await graph(page, f).evaluate((el) => ((el as unknown as { data: { x: number[]; y: number[]; name: string }[] }).data ?? []).map((t) => ({ x: Array.from(t.x), y: Array.from(t.y), name: t.name })));
      const expected = [f.method_a, ...(f.method_b ? [f.method_b] : []), ...f.observations];
      expect(traces).toHaveLength(expected.length);
      expected.forEach((s, i) => {
        expect(traces[i]?.x).toEqual(s.t);
        expect(traces[i]?.y).toEqual(s.f);
      });
      expect(traces.map((t) => t.name).slice(0, 1)).toEqual(['Method A: BTCS']);
      if (f.method_b) expect(traces[1]?.name).toBe('Method B: Sturm–Liouville');
    }
  });

  test('axes, title line and legend name their quantities and units', async ({ page, problems }) => {
    void problems;
    await open(page, 'both');
    const fig = page.getByTestId(`exp-graph-${BOTH.id}`);
    await expect(page.getByTestId(`exp-title-${BOTH.id}`)).toHaveText('Ag-110m, 1500 °C');
    await expect(fig.locator('.xtitle')).toHaveText('Time (h)');
    await expect(fig.locator('.ytitle')).toHaveText('Cumulative release fraction (dimensionless)');
    const legend = fig.locator('.legend');
    await expect(legend).toContainText('Method A: BTCS');
    await expect(legend).toContainText('Method B: Sturm–Liouville');
    await expect(legend).toContainText('Experimental observations');
  });

  test('R² is drawn verbatim in each method colour, including a negative value; nothing of it where it is unavailable', async ({ page, problems }) => {
    void problems;
    await open(page, 'both,negative,bounds,absent');
    const notes = async (f: ComparisonFile) =>
      graph(page, f).evaluate((el) => Array.from(el.querySelectorAll('.annotation-text')).map((a) => ({ text: a.textContent ?? '', fill: getComputedStyle(a).fill })));
    expect(await notes(BOTH)).toEqual([
      { text: BOTH.r2[0]!.label_plain, fill: 'rgb(37, 99, 235)' },
      { text: BOTH.r2[1]!.label_plain, fill: 'rgb(220, 38, 38)' },
    ]);
    expect(await notes(NEG)).toEqual([{ text: 'R² (Method A: BTCS) = -0.42', fill: 'rgb(37, 99, 235)' }]);
    expect(await notes(ABSENT)).toEqual([{ text: ABSENT.r2[0]!.label_plain, fill: 'rgb(37, 99, 235)' }]);
    expect(await notes(BOUNDS)).toEqual([]);
    const plotText = await page.getByTestId(`exp-plot-${BOUNDS.id}`).innerText();
    expect(plotText).not.toMatch(/R²|pending|unavailable|upper bound\b(?!s)/i);
    // No other metric and no verdict anywhere on the page.
    expect(await page.locator('body').innerText()).not.toMatch(/\b(RMSE|MAE|NRMSE|PASS|FAIL)\b/i);
  });

  test('the caption, then the citation and the link, stand below the whole graph', async ({ page, problems }) => {
    void problems;
    await open(page, 'bounds,both');
    const plot = await page.getByTestId(`exp-plot-${BOUNDS.id}`).boundingBox();
    const caption = await page.getByTestId(`exp-caption-${BOUNDS.id}`).boundingBox();
    const source = await page.getByTestId(`exp-source-${BOUNDS.id}`).boundingBox();
    expect(plot && caption && source).toBeTruthy();
    expect(caption!.y).toBeGreaterThanOrEqual(plot!.y + plot!.height - 1);
    expect(source!.y).toBeGreaterThanOrEqual(caption!.y + caption!.height - 1);
    await expect(page.getByTestId(`exp-caption-${BOUNDS.id}`)).toHaveText(BOUNDS.r2[0]!.caption_plain);
    await expect(page.getByTestId(`exp-citation-${BOUNDS.id}`)).toHaveText(BOUNDS.source.citation_apa);
    // Where both methods have a value there is no caption.
    await expect(page.getByTestId(`exp-caption-${BOTH.id}`)).toHaveCount(0);
  });

  test('the link is a real anchor with the address of the file and takes keyboard focus', async ({ page, problems }) => {
    void problems;
    await open(page, 'bounds');
    const link = page.getByTestId(`exp-link-${BOUNDS.id}`);
    await expect(link).toHaveAttribute('href', BOUNDS.source.link);
    expect(await link.evaluate((a) => a.tagName)).toBe('A');
    await page.keyboard.press('Tab');
    await expect(link).toBeFocused();
    const outline = await link.evaluate((a) => getComputedStyle(a).outlineStyle);
    expect(outline).not.toBe('none');
  });

  test('screenshots at 1440 px', async ({ page, problems }) => {
    void problems;
    test.skip(!SHOTS, 'screenshots only when SHOTS_DIR is set');
    for (const theme of ['light', 'dark']) {
      await open(page, 'both,negative,bounds,absent', theme);
      await page.screenshot({ path: join(SHOTS!, `graph-1440-${theme}.png`), fullPage: true });
    }
  });
});

for (const width of [768, 390]) {
  test.describe(`graph in the harness, ${width} px`, () => {
    test.use({ viewport: { width, height: 844 }, ...(width === 390 ? { isMobile: true, hasTouch: true, deviceScaleFactor: 2 } : {}) });

    test('fits without sideways scroll; legend, plot, caption and link stay inside the page; the link is a 44 px target', async ({ page, problems }) => {
      void problems;
      await open(page, 'both,negative,bounds,absent');
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
      for (const f of [BOTH, NEG, BOUNDS, ABSENT]) {
        const g = await graph(page, f).boundingBox();
        expect(g!.x).toBeGreaterThanOrEqual(0);
        expect(g!.x + g!.width).toBeLessThanOrEqual(width);
        const legend = await page.getByTestId(`exp-graph-${f.id}`).locator('.legend').boundingBox();
        expect(legend!.x).toBeGreaterThanOrEqual(0);
        expect(legend!.x + legend!.width).toBeLessThanOrEqual(width);
        // The legend sits above the plot area, not over the curves.
        const area = await page.getByTestId(`exp-graph-${f.id}`).locator('.draglayer .nsewdrag').first().boundingBox();
        expect(legend!.y + legend!.height).toBeLessThanOrEqual(area!.y + 1);
        const link = await page.getByTestId(`exp-link-${f.id}`).boundingBox();
        expect(link!.height).toBeGreaterThanOrEqual(44);
        expect(link!.x + link!.width).toBeLessThanOrEqual(width + 1);
      }
      if (SHOTS) {
        await page.screenshot({ path: join(SHOTS, `graph-${width}-light.png`), fullPage: true });
        await open(page, 'both,negative,bounds,absent', 'dark');
        await page.screenshot({ path: join(SHOTS, `graph-${width}-dark.png`), fullPage: true });
      }
    });
  });
}

test.describe('pages and build', () => {
  test('Compare and Validate show no comparison graph while the build holds no comparison file', async ({ page, problems }) => {
    void problems;
    for (const id of ['compare', 'validate']) {
      await gotoSection(page, id);
      await expect(page.getByTestId('exp-comparisons')).toHaveCount(0);
      await expect(page.locator('[data-testid^="exp-"]')).toHaveCount(0);
    }
  });

  test('no fixture and no harness is in the build output', () => {
    const dist = join(process.cwd(), 'dist');
    const walk = (d: string): string[] => readdirSync(d).flatMap((n) => (statSync(join(d, n)).isDirectory() ? walk(join(d, n)) : [join(d, n)]));
    const files = walk(dist);
    expect(files.length).toBeGreaterThan(0);
    expect(files.filter((f) => /harness|fixture/i.test(f))).toEqual([]);
    const hits = files.filter((f) => /\.(html|js|css|json|csv|txt|svg|md)$/i.test(f) && readFileSync(f, 'utf8').includes('FIXTURE'));
    expect(hits).toEqual([]);
  });
});
