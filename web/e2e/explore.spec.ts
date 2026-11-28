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

    test('has Select a Modelling Stage checkboxes, Species-Specific Physics dropdown, five Operation buttons, legend and no status word', async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      await expect(page.getByTestId('mech-baseline')).toBeChecked();
      await expect(page.getByTestId('mech-baseline')).toBeDisabled();
      await expect(page.getByTestId('species-physics-select')).toHaveValue('none');
      await expect(page.getByTestId('operation-picker').getByRole('radio')).toHaveCount(5);
      await expect(page.getByTestId('legend').locator('tbody tr')).toHaveCount(4);
      expect(await prose(page)).not.toMatch(STATUS_WORDS);
      await page.getByTestId('mech-generation').check();
      await page.getByTestId('mech-decay').check();
      await page.getByTestId('mech-temperature').check();
      await expect(page.getByTestId('legend').locator('tbody tr')).toHaveCount(7);
      await expect(page.getByTestId('config-id-label')).toContainText('P08');
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

  test('the model checkboxes and the operation are independent, and both are written to the address', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.getByTestId('operation-shutdown').click();
    await expect(page.getByTestId('config-id-label')).toContainText('P01');
    await expect(page).toHaveURL(/#\/explore\?scenario=shutdown$/);
    await page.getByTestId('mech-decay').check();
    await expect(page.getByTestId('operation-shutdown')).toHaveAttribute('aria-checked', 'true');
    await expect(page).toHaveURL(/#\/explore\?physics=decay&scenario=shutdown$/);
    await page.getByTestId('mech-generation').check();
    await expect(page).toHaveURL(/#\/explore\?config=P05&scenario=shutdown$/);
    await page.getByTestId('operation-steady').click();
    await expect(page).toHaveURL(/#\/explore\?config=P05$/);
    // A copied address reproduces the view, and the selection survives navigation to another section and back.
    await page.goto('./#/explore?config=P05&scenario=startup&open=pseudocode');
    await expect(page.getByTestId('mech-generation')).toBeChecked();
    await expect(page.getByTestId('mech-decay')).toBeChecked();
    await expect(page.getByTestId('mech-temperature')).not.toBeChecked();
    await expect(page.getByTestId('operation-startup')).toHaveAttribute('aria-checked', 'true');
    await expect(page.getByTestId('graph-toggles')).toHaveCount(0);
    await expect(page.getByTestId('num-toggle-pseudocode')).toHaveAttribute('aria-expanded', 'true');
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('link', { name: 'Research' }).click();
    await expect(page.locator('[data-section="research"] h1')).toBeVisible();
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('link', { name: 'Explore' }).click();
    await expect(page.getByTestId('config-id-label')).toContainText('P05');
    await expect(page).toHaveURL(/config=P05&scenario=startup/);
  });

  test('each control group moves cleanly by keyboard, shows a visible focus ring and announces changes', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const gen = page.getByTestId('mech-generation');
    await gen.focus();
    await page.keyboard.press('Space');
    await expect(gen).toBeChecked();
    await expect(page.getByTestId('physics-announce')).toContainText('P02');
    const ring = await page.evaluate(() => {
      const cs = getComputedStyle(document.activeElement as Element);
      return { style: cs.outlineStyle, width: parseFloat(cs.outlineWidth) };
    });
    expect(ring.style).toBe('solid');
    expect(ring.width).toBeGreaterThanOrEqual(2);
    await page.getByTestId('operation-steady').focus();
    await page.keyboard.press('ArrowRight');
    await expect(page.getByTestId('operation-shutdown')).toBeFocused();
    await expect(page.getByTestId('operation-shutdown')).toHaveAttribute('aria-checked', 'true');
    await expect(gen).toBeChecked();
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

  for (const width of [1440, 1024, 768, 390] as const) {
    test(`mathematical equation numbers never overlap expressions at ${width} px across all accordions`, async ({ page, problems }) => {
      void problems;
      await page.setViewportSize({ width, height: 900 });
      await gotoNumerical(page, ['method-a', 'method-b', 'discretisation', 'pseudocode', 'assumptions', 'limitations'], 'physics=combined&scenario=shutdown');
      await expect(page.locator('[data-section="explore"] .katex-error')).toHaveCount(0);
      const collisions = await page.evaluate(() => {
        const problems: string[] = [];
        const sectionBox = document.querySelector('.xarea--numerical')?.getBoundingClientRect();
        for (const eq of Array.from(document.querySelectorAll<HTMLElement>('.nstep__eq[data-has-tag="true"]'))) {
          const math = eq.querySelector<HTMLElement>('.nstep__math');
          const tag = eq.querySelector<HTMLElement>('.nstep__tag');
          if (!math || !tag) {
            problems.push(`missing child in ${eq.dataset.eq ?? 'unknown'}`);
            continue;
          }
          if (eq.querySelector('.katex-tag')) problems.push(`inline katex-tag still present in ${eq.dataset.eq ?? ''}`);
          const mRect = math.getBoundingClientRect();
          const tRect = tag.getBoundingClientRect();
          if (mRect.right > tRect.left + 1) {
            problems.push(`overlap ${eq.dataset.eq ?? ''}: math.right=${mRect.right.toFixed(1)} > tag.left=${tRect.left.toFixed(1)}`);
          }
          if (sectionBox && tRect.right > sectionBox.right + 2) {
            problems.push(`clipped tag ${eq.dataset.eq ?? ''}: tag.right=${tRect.right.toFixed(1)} > section.right=${sectionBox.right.toFixed(1)}`);
          }
        }
        return problems;
      });
      expect(collisions, `collisions at ${width}px`).toEqual([]);
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
    });
  }
});

test.describe('interactive graphs', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('locks Species to Cs-137 only, removes graph-type buttons and title, and keeps overlay toggles interactive', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await expect(page.getByTestId('graph-toggles')).toHaveCount(0);
    await expect(page.getByText(/Solver: \d/).first()).toBeVisible({ timeout: 30_000 });
    const species = page.getByTestId('species-select');
    await expect(species.locator('option')).toHaveText(['Cs-137']);
    await expect(species).toHaveValue('cs137');
    const temp = page.getByTestId('temperature-select');
    await expect(temp.locator('option')).toHaveText(['1200 °C', '1350 °C', '1500 °C', '1600 °C', '1700 °C', '1800 °C'], { timeout: 30_000 });
    const plot = page.getByTestId('explore-plot-release-graph');
    await expect(plot).toBeVisible({ timeout: 30_000 });
    await expect.poll(() => plot.evaluate((el) => ((el as unknown as { data?: unknown[] }).data ?? []).length), { timeout: 30_000 }).toBeGreaterThanOrEqual(3);
    const layoutTitle = await plot.evaluate((el) => (el as unknown as { layout?: { title?: { text?: string } } }).layout?.title?.text ?? '');
    expect(layoutTitle).toBe('');
    // Uncheck Method B -> Experimental + Method A remain
    await page.getByTestId('overlay-method-b').uncheck();
    await expect.poll(() => plot.evaluate((el) => ((el as unknown as { data?: unknown[] }).data ?? []).length)).toBe(2);
    // Uncheck Method A and Experimental -> empty overlay prompt; re-enable Experiment -> 1 trace
    await page.getByTestId('overlay-method-a').uncheck();
    await page.getByTestId('overlay-experiment').uncheck();
    await expect(page.getByTestId('overlay-empty-note')).toBeVisible();
    await page.getByTestId('overlay-experiment').check();
    await expect.poll(() => plot.evaluate((el) => ((el as unknown as { data?: unknown[] }).data ?? []).length)).toBe(1);
    await page.getByTestId('overlay-method-a').check();
    await page.getByTestId('overlay-method-b').check();
  });

  test('keeps all six temperatures available across every operation and physics selection, and updates Cs-137 plots cleanly', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const expectedTemps = ['1200 °C', '1350 °C', '1500 °C', '1600 °C', '1700 °C', '1800 °C'];
    const species = page.getByTestId('species-select');
    const temp = page.getByTestId('temperature-select');
    const plot = page.getByTestId('explore-plot-release-graph');
    await expect(plot).toBeVisible({ timeout: 30_000 });

    // Steady-state across all 6 temperatures updates Cs-137 graph without plot titles
    for (const tc of ['1200', '1350', '1500', '1600', '1700', '1800']) {
      await temp.selectOption(tc);
      await expect(temp.locator('option')).toHaveText(expectedTemps);
      await expect(species.locator('option')).toHaveText(['Cs-137']);
      await expect(page.getByTestId('graph-panel')).toHaveAttribute('data-empty', 'false');
      const title = await plot.evaluate((el) => (el as unknown as { layout?: { title?: { text?: string } } }).layout?.title?.text ?? '');
      expect(title).toBe('');
    }

    // All 5 operations preserve Cs-137 only + all 6 temperatures
    for (const operation of ['steady', 'shutdown', 'startup', 'control-rod', 'refuelling']) {
      await page.getByTestId(`operation-${operation}`).click();
      await expect(species.locator('option')).toHaveText(['Cs-137']);
      await expect(temp.locator('option')).toHaveText(expectedTemps);
      await expect(page.getByTestId('graph-panel')).toHaveAttribute('data-empty', 'false');
      if (operation !== 'steady') {
        await expect(page.getByTestId('scenario-curve-note')).toBeVisible();
      }
    }

    // Return to Steady-state and switch across P01, P02 (with controlled overlay), P04, P08
    await page.getByTestId('operation-steady').click();
    await page.getByTestId('mech-generation').check();
    await expect(page.getByTestId('overlay-controlled')).toBeVisible();
    await expect(temp.locator('option')).toHaveText(expectedTemps);
    await page.getByTestId('mech-generation').uncheck();
    await page.getByTestId('mech-temperature').check();
    await expect(temp.locator('option')).toHaveText(expectedTemps);
    await page.getByTestId('mech-generation').check();
    await page.getByTestId('mech-decay').check();
    await expect(temp.locator('option')).toHaveText(expectedTemps);
    await expect(page.getByTestId('graph-panel')).toHaveAttribute('data-empty', 'false');

    // Future physics still keeps Species = Cs-137 and all 6 temperatures while leaving the plot frame empty
    await page.goto('./#/explore?physics=future');
    await expect(species.locator('option')).toHaveText(['Cs-137']);
    await expect(temp.locator('option')).toHaveText(expectedTemps);
    await expect(page.getByTestId('graph-panel')).toHaveAttribute('data-empty', 'true');
  });
});
