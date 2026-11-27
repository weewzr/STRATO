import type { Page } from '@playwright/test';
import { areaTops, AREAS, expect, gotoNumerical, gotoSection, horizontalOverflow, test } from './fixtures';

// Explore in four areas (final layout of 27 Nov 2026): model equation and operation upper left, the governing
// equation with its legend upper centre and right, numerical interpretation lower left, interactive graphs
// lower right; stacked in that order at 390 px. The earlier specs of the blocks A to E, of the numerical items,
// of the scenario row and of the learning sequence encoded the layout that this one replaces.

const WIDTHS = [
  { width: 1440, height: 900, mobile: false },
  { width: 768, height: 1024, mobile: false },
  { width: 390, height: 844, mobile: true },
] as const;
const STATUS_WORDS = /\b(planned|pending|researched|implemented|verified|validated|computed|precomputed|not verified|unavailable|not available)\b/i;

/** The text of the Explore page without the typeset formulas and the code of the pseudocode. */
async function prose(page: Page): Promise<string> {
  return page.locator('[data-section="explore"]').evaluate((root) => {
    const copy = root.cloneNode(true) as HTMLElement;
    copy.querySelectorAll('.katex, .npseudo__code, option').forEach((n) => n.remove());
    return copy.textContent ?? '';
  });
}

for (const vp of WIDTHS) {
  test.describe(`Explore at ${vp.width}x${vp.height}`, () => {
    test.use({ viewport: { width: vp.width, height: vp.height }, isMobile: vp.mobile, hasTouch: vp.mobile, deviceScaleFactor: vp.mobile ? 2 : 1 });

    test('shows the four areas in order, without sideways page scroll, with 44 px targets for every control', async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      await expect(page.getByTestId('graph-panel')).toBeVisible();
      const tops = await areaTops(page);
      expect(tops.map((t) => t.area)).toEqual([...AREAS]);
      const [controls, equation, numerical, graphs] = tops;
      if (!controls || !equation || !numerical || !graphs) throw new Error('missing area');
      if (vp.width >= 1024) {
        // Upper left, upper right, lower left, lower right.
        expect(equation.left).toBeGreaterThan(controls.left + 100);
        expect(Math.abs(equation.top - controls.top)).toBeLessThan(4);
        expect(numerical.top).toBeGreaterThan(controls.bottom - 1);
        expect(Math.abs(graphs.left - equation.left)).toBeLessThan(4);
        expect(Math.abs(graphs.top - numerical.top)).toBeLessThan(4);
      } else {
        for (let i = 1; i < tops.length; i++) expect(tops[i]?.top ?? 0).toBeGreaterThan((tops[i - 1]?.top ?? 0) + 20);
      }
      // MODEL EQUATION above OPERATION.
      const model = await page.getByTestId('model-equation').boundingBox();
      const operation = await page.getByTestId('operation').boundingBox();
      expect((operation?.y ?? 0) > (model?.y ?? 0)).toBe(true);
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
      // Every button, toggle and select is at least 44 px high.
      const small = await page.locator('[data-section="explore"] button, [data-section="explore"] select').evaluateAll((els) =>
        els.filter((e) => e.getBoundingClientRect().height < 43.5 && e.getBoundingClientRect().width > 0).map((e) => `${e.tagName} ${(e.textContent ?? '').slice(0, 30)} ${Math.round(e.getBoundingClientRect().height)}`),
      );
      expect(small).toEqual([]);
    });

    test('has six model-equation buttons and five operation buttons, a legend of the displayed variables and no status word', async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      await expect(page.getByTestId('physics-picker').getByRole('radio')).toHaveCount(6);
      await expect(page.getByTestId('operation-picker').getByRole('radio')).toHaveCount(5);
      await expect(page.getByTestId('legend').locator('tbody tr')).toHaveCount(4);
      expect(await prose(page)).not.toMatch(STATUS_WORDS);
      await page.getByTestId('physics-combined').click();
      await expect(page.getByTestId('legend').locator('tbody tr')).toHaveCount(7);
      await page.getByTestId('physics-future').click();
      await expect(page.getByTestId('future-links').getByRole('link')).toHaveCount(4);
      expect(await prose(page)).not.toMatch(STATUS_WORDS);
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
    });
  });
}

for (const colorScheme of ['light', 'dark'] as const) {
  for (const reducedMotion of ['no-preference', 'reduce'] as const) {
    test.describe(`fold at 1440 x 900, ${colorScheme}, reduced motion ${reducedMotion}`, () => {
      test.use({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1, colorScheme, reducedMotion });

      test('the four areas are visible together, the graphs reaching past the fold only slightly', async ({ page, problems }) => {
        void problems;
        await gotoSection(page, 'explore');
        await expect(page.getByTestId('graph-panel')).toBeVisible();
        const tops = await areaTops(page);
        for (const t of tops) expect(t.top, t.area).toBeLessThan(900 - 80);
        const graphs = tops.find((t) => t.area === 'graphs');
        expect(graphs?.bottom ?? 0).toBeLessThan(900 + 200);
        expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
      });
    });
  }
}

test.describe('selection, address and keyboard', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('the model equation and the operation are independent, and both are written to the address', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.getByTestId('operation-shutdown').click();
    await expect(page.getByTestId('physics-baseline')).toHaveAttribute('aria-checked', 'true');
    await expect(page).toHaveURL(/#\/explore\?scenario=shutdown$/);
    await page.getByTestId('physics-decay').click();
    await expect(page.getByTestId('operation-shutdown')).toHaveAttribute('aria-checked', 'true');
    await expect(page).toHaveURL(/#\/explore\?physics=decay&scenario=shutdown$/);
    await page.getByTestId('operation-steady').click();
    await expect(page).toHaveURL(/#\/explore\?physics=decay$/);
    // A copied address reproduces the view, and the selection survives navigation to another section and back.
    await page.goto('./#/explore?physics=future&scenario=startup&open=pseudocode&graph=profile');
    await expect(page.getByTestId('physics-future')).toHaveAttribute('aria-checked', 'true');
    await expect(page.getByTestId('operation-startup')).toHaveAttribute('aria-checked', 'true');
    await expect(page.getByTestId('graph-profile')).toHaveAttribute('aria-checked', 'true');
    await expect(page.getByTestId('num-toggle-pseudocode')).toHaveAttribute('aria-expanded', 'true');
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('link', { name: 'Research' }).click();
    await expect(page.locator('[data-section="research"] h1')).toBeVisible();
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('link', { name: 'Explore' }).click();
    await expect(page.getByTestId('physics-future')).toHaveAttribute('aria-checked', 'true');
    await expect(page).toHaveURL(/physics=future&scenario=startup/);
  });

  test('each group is one tab stop, moves with the arrow keys, shows a visible focus ring and announces the change', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const baseline = page.getByTestId('physics-baseline');
    await baseline.focus();
    await page.keyboard.press('ArrowRight');
    await expect(page.getByTestId('physics-generation')).toBeFocused();
    await expect(page.getByTestId('physics-generation')).toHaveAttribute('aria-checked', 'true');
    await expect(page.getByTestId('physics-announce')).toContainText('enters the right-hand side');
    const ring = await page.evaluate(() => {
      const cs = getComputedStyle(document.activeElement as Element);
      return { style: cs.outlineStyle, width: parseFloat(cs.outlineWidth) };
    });
    expect(ring.style).toBe('solid');
    expect(ring.width).toBeGreaterThanOrEqual(2);
    await page.keyboard.press('End');
    await expect(page.getByTestId('physics-future')).toHaveAttribute('aria-checked', 'true');
    await page.keyboard.press('Tab');
    expect(await page.evaluate(() => document.activeElement?.closest('[data-testid="physics-picker"]') === null)).toBe(true);
    await page.getByTestId('operation-steady').focus();
    await page.keyboard.press('ArrowRight');
    await expect(page.getByTestId('operation-shutdown')).toBeFocused();
    await expect(page.getByTestId('physics-future')).toHaveAttribute('aria-checked', 'true');
  });

  test('the six sections of the Numerical Interpretation are collapsed, open with Enter and Space, and keep the focus', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    for (const id of ['method-a', 'method-b', 'discretisation', 'pseudocode', 'assumptions', 'limitations']) {
      await expect(page.getByTestId(`num-toggle-${id}`)).toHaveAttribute('aria-expanded', 'false');
    }
    const toggle = page.getByTestId('num-toggle-discretisation');
    await toggle.focus();
    await page.keyboard.press('Enter');
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    await expect(toggle).toBeFocused();
    await expect(page).toHaveURL(/open=discretisation/);
    await expect(page.getByTestId('num-section-discretisation').locator('.katex').first()).toBeVisible();
    await expect(page.getByTestId('num-section-discretisation').locator('.katex-error')).toHaveCount(0);
    await page.keyboard.press('Space');
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
  });

  test('every section opened for the combined model typesets without an error, and the pseudocode is the document code', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page, ['method-a', 'method-b', 'discretisation', 'pseudocode', 'assumptions', 'limitations'], 'physics=combined&scenario=shutdown');
    await expect(page.locator('[data-section="explore"] .katex-error')).toHaveCount(0);
    await expect(page.getByTestId('num-section-pseudocode').locator('[data-block]')).toHaveCount(6);
    await expect(page.getByTestId('num-section-pseudocode').locator('[data-hook]')).toHaveCount(3);
    await expect(page.locator('[data-operation-note]')).toHaveCount(3);
    expect(await prose(page)).not.toMatch(STATUS_WORDS);
    expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
  });
});

test.describe('interactive graphs', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('draw the wrapper curves for steady-state operation: Method A blue and solid, the temperature list in 100 degC steps', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const temp = page.getByTestId('temperature-select');
    await expect(temp.locator('option')).toHaveText(['1600 °C', '1700 °C', '1800 °C'], { timeout: 30_000 });
    await expect(page.getByTestId('species-select').locator('option').first()).toBeAttached();
    const plot = page.getByTestId('explore-plot-release-graph');
    await expect(plot).toBeVisible({ timeout: 30_000 });
    await expect.poll(() => plot.evaluate((el) => ((el as unknown as { data?: unknown[] }).data ?? []).length), { timeout: 30_000 }).toBe(1);
    const trace = await plot.evaluate((el) => {
      const t = (el as unknown as { data: { x: number[]; y: number[]; line: { color: string; dash: string } }[] }).data[0];
      return { n: t?.x.length ?? 0, ny: t?.y.length ?? 0, colour: t?.line.color, dash: t?.line.dash, last: t?.y[(t?.y.length ?? 1) - 1] ?? NaN };
    });
    expect(trace.n).toBeGreaterThan(10);
    expect(trace.ny).toBe(trace.n);
    expect(trace.colour).toBe('#2563EB');
    expect(trace.dash).toBe('solid');
    // The release fraction of one particle lies between 0 and 1.
    expect(trace.last).toBeGreaterThanOrEqual(0);
    expect(trace.last).toBeLessThanOrEqual(1);
    await expect(page.getByTestId('graph-empty')).toHaveCount(0);
  });

  test('are an empty frame, with no text, for every other operation and for future species-specific physics', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await expect(page.getByTestId('explore-plot-release-graph')).toBeVisible({ timeout: 30_000 });
    for (const graph of ['release', 'profile', 'methods', 'experiment']) {
      await page.getByTestId(`graph-${graph}`).click();
      for (const operation of ['shutdown', 'startup', 'control-rod', 'refuelling']) {
        await page.getByTestId(`operation-${operation}`).click();
        const panel = page.getByTestId('graph-panel');
        await expect(panel).toHaveAttribute('data-empty', 'true');
        expect(((await panel.textContent()) ?? '').trim()).toBe('');
        await expect(panel.locator('svg, canvas, .js-plotly-plot')).toHaveCount(0);
      }
      await page.getByTestId('operation-steady').click();
      await page.getByTestId('physics-future').click();
      await expect(page.getByTestId('graph-panel')).toHaveAttribute('data-empty', 'true');
      expect(((await page.getByTestId('graph-panel').textContent()) ?? '').trim()).toBe('');
      await page.getByTestId('physics-baseline').click();
    }
  });

  test('draw the registered experimental data with its caption, citation and link beneath', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.getByTestId('graph-experiment').click();
    await expect(page.getByTestId('dataset-select')).toBeVisible({ timeout: 30_000 });
    const plot = page.getByTestId('explore-plot-experiment-graph');
    await expect(plot).toBeVisible();
    await expect.poll(() => plot.evaluate((el) => ((el as unknown as { data?: unknown[] }).data ?? []).length), { timeout: 30_000 }).toBeGreaterThan(1);
    const markers = await plot.evaluate((el) =>
      (el as unknown as { data: { mode: string; marker?: { color: string; symbol: string } }[] }).data.filter((t) => t.mode === 'markers').map((t) => `${t.marker?.color}/${t.marker?.symbol}`),
    );
    expect(markers.length).toBeGreaterThan(0);
    for (const m of markers) expect(m).toMatch(/^#111111\/(circle|triangle-down)$/);
    await expect(page.getByTestId('experiment-caption')).not.toBeEmpty();
    await expect(page.getByTestId('experiment-apa')).toContainText(/\(\d{4}\)/);
    await expect(page.getByTestId('experiment-link')).toHaveAttribute('href', /^https?:\/\//);
    expect(await prose(page)).not.toMatch(STATUS_WORDS);
  });
});
