import { join } from 'node:path';
import { blockTops, BLOCKS, expect, gotoSection, horizontalOverflow, solverRequests, test, waitForRun } from './fixtures';

// The Explore page in the hierarchy of the owner's directive of 26 Nov 2026 (Sections 2, 6 and 9):
// A governing equation, B operating scenarios, C Numerical Interpretation (collapsed), D live simulation,
// E results; the progressive equation; the shared selection in the address; a selection that the build
// does not implement is said so and shows no substituted result. Review screenshots are written only
// when SHOTS_DIR is set.

const dir = process.env.SHOTS_DIR ?? '';
const PHYSICS = ['baseline', 'generation', 'decay', 'temperature', 'combined'] as const;

test.describe('hierarchy and selection at 1440 x 900', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('blocks A to E in order; the Numerical Interpretation is collapsed; block B holds the five scenario buttons', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const tops = await blockTops(page);
    expect(tops.map((t) => t.letter)).toEqual([...BLOCKS]);
    for (let i = 1; i < tops.length; i++) expect(tops[i]?.top ?? 0).toBeGreaterThan(tops[i - 1]?.top ?? 0);
    await expect(page.getByTestId('block-b').getByRole('radio')).toHaveCount(5);
    await expect(page.getByTestId('numerical-toggle')).toHaveAttribute('aria-expanded', 'false');
    await expect(page.getByTestId('equation-builder')).toHaveCount(0);
    await expect(page.getByRole('heading', { name: 'Numerical Interpretation' })).toBeVisible();
    // The equation is the first element of the page and carries the baseline.
    await expect(page.getByTestId('governing-equation')).toHaveAttribute('data-physics', 'baseline');
    expect(await page.locator('[data-testid="equation-governing"] .katex').count()).toBeGreaterThan(0);
    await expect(page.locator('.katex-error')).toHaveCount(0);
  });

  test('the Numerical Interpretation opens, keeps the selected physics, and is written to the address', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.getByTestId('physics-decay').click();
    await page.getByTestId('numerical-toggle').click();
    // The region opens with its items collapsed; decay reveals its own item.
    await expect(page.getByTestId('num-items')).toBeVisible();
    await expect(page.getByTestId('equation-builder')).toHaveCount(0);
    await expect(page.getByTestId('num-item-10')).toBeVisible();
    await expect(page.getByTestId('num-item-9')).toHaveCount(0);
    await expect(page).toHaveURL(/#\/explore\?physics=decay&open=numerical$/);
    await page.getByTestId('num-toggle-3').click();
    await expect(page.getByTestId('num-item-3').getByTestId('equation-builder')).toHaveAttribute('data-stage', '5');
    await expect(page.getByTestId('governing-equation')).toHaveAttribute('data-physics', 'decay');
    await expect(page).toHaveURL(/#\/explore\?physics=decay&open=numerical,n3$/);
    await page.getByTestId('numerical-toggle').click();
    await expect(page.getByTestId('equation-builder')).toHaveCount(0);
    await expect(page).toHaveURL(/#\/explore\?physics=decay&open=n3$/);
  });

  test('a copied address reproduces the view, and the selection survives navigation between the sections', async ({ page, problems }) => {
    void problems;
    await page.goto('./#/explore?physics=combined&scenario=startup&open=numerical,n9,n11');
    await expect(page.locator('[data-section="explore"] h1')).toBeVisible();
    await expect(page.getByTestId('physics-combined')).toHaveAttribute('aria-checked', 'true');
    await expect(page.getByTestId('num-item-9').getByTestId('equation-builder')).toBeVisible();
    await expect(page.getByTestId('num-toggle-10')).toHaveAttribute('aria-expanded', 'false');
    // Through the navigation links and back.
    const nav = page.getByRole('navigation', { name: 'Sections' });
    for (const name of ['Compare', 'Validate', 'Verify', 'Research', 'Data', 'About']) {
      await nav.getByRole('link', { name }).click();
      await expect(page.locator(`[data-section="${name.toLowerCase()}"] h1`)).toBeVisible();
    }
    await nav.getByRole('link', { name: 'Explore' }).click();
    await expect(page.locator('[data-section="explore"] h1')).toBeVisible();
    await expect(page.getByTestId('physics-combined')).toHaveAttribute('aria-checked', 'true');
    // Which items are open is kept with the physics and the scenario.
    await expect(page.getByTestId('num-toggle-9')).toHaveAttribute('aria-expanded', 'true');
    await expect(page.getByTestId('num-toggle-11')).toHaveAttribute('aria-expanded', 'true');
    await expect(page.getByTestId('num-toggle-10')).toHaveAttribute('aria-expanded', 'false');
    await expect(page).toHaveURL(/#\/explore\?physics=combined&scenario=startup&open=numerical,n9,n11$/);
    // A reload of that address gives the same view; nothing is kept in the browser's storage.
    await page.reload();
    await expect(page.getByTestId('physics-combined')).toHaveAttribute('aria-checked', 'true');
    expect(await page.evaluate(() => window.localStorage.length + window.sessionStorage.length)).toBe(0);
  });

  test('a selection that this build does not implement is said so in D and E, and nothing is run or shown in its place', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    const before = await solverRequests(page);
    // Every choice of the build's capability shows its status in a few words: baseline only today.
    await expect(page.getByTestId('physics-status-baseline')).toHaveText('Verified');
    for (const id of ['generation', 'decay', 'temperature', 'combined']) await expect(page.getByTestId(`physics-status-${id}`)).toHaveText('Planned');
    await page.getByTestId('physics-generation').click();
    await expect(page.getByTestId('unsupported-simulation')).toContainText('does not implement fission-product generation');
    await expect(page.getByTestId('unsupported-results')).toContainText('no result is shown for this selection');
    await expect(page.getByTestId('particle')).toHaveCount(0);
    await expect(page.getByTestId('release-readout')).toHaveCount(0);
    await expect(page.getByTestId('profile-figure')).toHaveCount(0);
    await expect(page.getByTestId('status-result')).toHaveText('Result: none for the selection on Explore');
    // The equation and its explanation stay, with the new term highlighted.
    await expect(page.locator('[data-piece="generation"]')).toBeVisible();
    await expect(page.getByTestId('term-meanings')).toContainText('Fission-product generation');
    await page.waitForTimeout(300);
    expect(await solverRequests(page)).toBe(before);
    if (dir) await page.screenshot({ path: join(dir, 'explore-unsupported-1440-full.png'), fullPage: true });
    // Back to the baseline: the simulation and the results return.
    await page.getByTestId('physics-baseline').click();
    await waitForRun(page);
    await expect(page.getByTestId('particle')).toBeVisible();
    await expect(page.getByTestId('release-readout').locator('.badge')).toHaveText('COMPUTED v2.0');
    await expect(page.getByTestId('unsupported-simulation')).toHaveCount(0);
  });

  test('the live region says what changed', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const live = page.getByTestId('physics-announce');
    await expect(live).toHaveText('');
    await page.getByTestId('physics-generation').click();
    await expect(live).toContainText('enters the right-hand side');
    await page.getByTestId('physics-temperature').click();
    await expect(live).toContainText('no term is added');
  });
});

for (const reducedMotion of ['no-preference', 'reduce'] as const) {
  test.describe(`highlight of the new term, reduced motion ${reducedMotion}`, () => {
    test.use({ viewport: { width: 1440, height: 900 }, reducedMotion });

    test('is outlined in both cases and moves only when motion is allowed', async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      for (const [id, selector] of [
        ['generation', '[data-piece="generation"]'],
        ['decay', '[data-piece="decay"]'],
        ['temperature', '[data-piece="operator"] .geq-hi'],
      ] as const) {
        await page.getByTestId(`physics-${id}`).click();
        const style = await page.locator(selector).first().evaluate((e) => {
          const cs = getComputedStyle(e);
          return { outline: cs.outlineStyle, width: parseFloat(cs.outlineWidth), animation: cs.animationName };
        });
        expect(style.outline, id).toBe('solid');
        expect(style.width, id).toBeGreaterThanOrEqual(2);
        if (reducedMotion === 'reduce') expect(style.animation, id).toBe('none');
        else expect(style.animation, id).not.toBe('none');
      }
      // The base pieces carry no highlight and no animation.
      const lhs = await page.locator('[data-piece="lhs"]').evaluate((e) => ({ outline: getComputedStyle(e).outlineStyle, animation: getComputedStyle(e).animationName }));
      expect(lhs).toEqual({ outline: 'none', animation: 'none' });
    });
  });
}

for (const vp of [
  { width: 1440, height: 900, mobile: false },
  { width: 768, height: 1024, mobile: false },
  { width: 390, height: 844, mobile: true },
]) {
  for (const scheme of ['light', 'dark'] as const) {
    test.describe(`governing equation at ${vp.width} px, ${scheme}`, () => {
      test.use({ viewport: { width: vp.width, height: vp.height }, isMobile: vp.mobile, hasTouch: vp.mobile, colorScheme: scheme });

      test('every physics choice fits the width, the controls are 44 px high and no equation scrolls the page', async ({ page, problems }) => {
        void problems;
        await gotoSection(page, 'explore');
        await page.evaluate(() => document.fonts.ready);
        for (const id of PHYSICS) {
          await page.getByTestId(`physics-${id}`).click();
          await expect(page.getByTestId('governing-equation')).toHaveAttribute('data-physics', id);
          expect(await horizontalOverflow(page), id).toBeLessThanOrEqual(0);
          const eq = await page.getByTestId('equation-governing').boundingBox();
          expect(eq?.x ?? -1).toBeGreaterThanOrEqual(0);
          expect((eq?.x ?? 0) + (eq?.width ?? 0), id).toBeLessThanOrEqual(vp.width + 0.5);
          if (dir) await page.getByTestId('block-a').screenshot({ path: join(dir, `equation-${id}-${vp.width}-${scheme}.png`) });
        }
        for (const id of PHYSICS) {
          const b = await page.getByTestId(`physics-${id}`).boundingBox();
          expect(b?.height ?? 0, id).toBeGreaterThanOrEqual(44);
          expect(b?.width ?? 0, id).toBeGreaterThanOrEqual(44);
        }
        // Focus is visible on a choice.
        await page.getByTestId('physics-combined').focus();
        await page.keyboard.press('ArrowLeft');
        await expect(page.getByTestId('physics-temperature')).toBeFocused();
        const ring = await page.getByTestId('physics-temperature').evaluate((e) => ({ style: getComputedStyle(e).outlineStyle, width: parseFloat(getComputedStyle(e).outlineWidth) }));
        expect(ring.style).toBe('solid');
        expect(ring.width).toBeGreaterThanOrEqual(2);
        if (dir) {
          await page.screenshot({ path: join(dir, `explore-first-${vp.width}-${scheme}.png`) });
          await page.getByTestId('numerical-toggle').click();
          await page.screenshot({ path: join(dir, `explore-numerical-${vp.width}-${scheme}.png`), fullPage: true });
        }
      });
    });
  }
}
