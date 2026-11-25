import type { Page } from '@playwright/test';
import { BLOCKS, blockTops, expect, gotoSection, horizontalOverflow, solverRequests, test, VIEWPORTS, waitForRun } from './fixtures';

// Equation-first page structure (owner decision M7): blocks A to E in this order at every width,
// and the fold requirement at 1440 x 900 (prompt/TESTING_AND_VALIDATION.md Section 3.3).

async function expectOrder(page: Page) {
  const tops = await blockTops(page);
  expect(tops.map((t) => t.letter)).toEqual([...BLOCKS]);
  for (let i = 1; i < tops.length; i++) {
    expect(tops[i]?.top ?? 0, `top of ${tops[i]?.letter} below top of ${tops[i - 1]?.letter}`).toBeGreaterThan(tops[i - 1]?.top ?? 0);
  }
}

for (const vp of VIEWPORTS) {
  test.describe(`blocks at ${vp.width}x${vp.height}`, () => {
    test.use({ viewport: { width: vp.width, height: vp.height } });

    test('appear in the order A to E, without sideways page scroll', async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      await expectOrder(page);
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
    });
  });
}

test.describe('blocks on a 390 px phone (device emulation)', () => {
  test.use({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true, deviceScaleFactor: 3 });

  test('appear in the order A to E, controls open in a bottom sheet', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await expectOrder(page);
    expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
    // Block C holds the entry to the controls; the controls themselves open in the bottom sheet.
    const blockC = page.getByTestId('block-c');
    await expect(blockC.getByTestId('open-panel')).toBeVisible();
    await expect(blockC.getByTestId('controls')).toHaveCount(0);
    await blockC.getByTestId('open-panel').click();
    const sheet = page.getByTestId('parameter-sheet');
    await expect(sheet).toBeVisible();
    await expect(sheet.getByTestId('controls')).toHaveAttribute('data-layout', 'sheet');
    const box = await sheet.boundingBox();
    expect(box?.width).toBe(390);
    expect(Math.round((box?.y ?? 0) + (box?.height ?? 0))).toBe(844);
  });
});

for (const colorScheme of ['light', 'dark'] as const) {
  for (const reducedMotion of ['no-preference', 'reduce'] as const) {
    test.describe(`fold at 1440 x 900, ${colorScheme}, reduced motion ${reducedMotion}`, () => {
      test.use({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1, colorScheme, reducedMotion });

      test('the equation block and the top of the simulation are visible without scrolling', async ({ page, problems }) => {
        void problems;
        await gotoSection(page, 'explore');
        // Web phase 3: measure with the solver's result in place (the readout then carries its label).
        await waitForRun(page);
        await page.evaluate(() => document.fonts.ready);
        expect(await page.evaluate(() => window.scrollY)).toBe(0);
        const footerTop = (await page.locator('footer.status').boundingBox())?.y ?? 900;
        const headerBottom = await page.locator('header.header').evaluate((el) => el.getBoundingClientRect().bottom);
        for (const id of ['block-a', 'particle', 'release-readout']) {
          const box = await page.getByTestId(id).boundingBox();
          expect(box, id).not.toBeNull();
          if (!box) continue;
          expect(box.y, `${id} top`).toBeGreaterThanOrEqual(headerBottom - 0.5);
          expect(box.x, `${id} left`).toBeGreaterThanOrEqual(0);
          expect(box.x + box.width, `${id} right`).toBeLessThanOrEqual(1440);
          expect(box.y + box.height, `${id} bottom above the status line`).toBeLessThanOrEqual(footerTop + 0.5);
        }
        // Each of the two simulation elements carries its result label, inside the measured box.
        // Web phase 4: with the run in place the particle is coloured by the solver's concentration.
        await expect(page.getByTestId('particle').locator('.badge')).toHaveText('COMPUTED v1.0');
        await expect(page.getByTestId('release-readout').locator('.badge')).toHaveText('COMPUTED v1.0');
        await expectOrder(page);
      });
    });
  }
}

test.describe('placeholders', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('name the web phase that fills them and contain no result label', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const phases = await page.locator('[data-placeholder-phase]').evaluateAll((els) => els.map((e) => e.getAttribute('data-placeholder-phase')));
    // Web phase 2 replaced the two phase-2 placeholders with the equation builder and the term
    // explanations; web phase 3 replaced the two phase-3 placeholders with the solver's inventory
    // balance and diagnostics; web phase 4 replaced the AGR-1 reference placeholder with the
    // registered-geometry view; web phase 5 replaced the profile and release placeholders with the
    // plots; web phase 5.5 replaced the last one with the research catalogue summary in block E.
    expect(phases).toEqual([]);
    await expect(page.getByTestId('block-e').getByTestId('future-research')).toBeVisible();
    expect(phases).not.toContain('2');
    expect(phases).not.toContain('3');
    expect(phases).not.toContain('4');
    expect(phases).not.toContain('5');
    for (const p of phases) expect(p).toMatch(/^\d(\.\d)?$/);
    await expect(page.locator('[data-placeholder-phase] .badge')).toHaveCount(0);
    await expect(page.getByTestId('simulation-case')).toContainText('Synthetic five-layer verification case, not AGR-1 data');
  });

  // Web phase 3: Run now runs the solver; the mechanism switches still send nothing (Section 3.2).
  test('the mechanism switches send nothing and stay off; Run sends exactly one solver request', async ({ page, problems }) => {
    void problems;
    const requests: string[] = [];
    await gotoSection(page, 'explore');
    await waitForRun(page);
    const before = await solverRequests(page);
    expect(before).toBeGreaterThanOrEqual(1);
    page.on('request', (r) => requests.push(r.url()));
    // aria-disabled controls stay focusable and clickable; force skips Playwright's enabled check.
    for (const id of ['source', 'decay', 'arrhenius']) {
      await page.getByTestId(`mech-${id}`).click({ force: true });
      await expect(page.getByTestId(`mech-${id}`)).toHaveAttribute('aria-checked', 'false');
    }
    await page.waitForTimeout(400);
    expect(await solverRequests(page)).toBe(before);
    expect(requests).toEqual([]);
    await page.getByTestId('strip-run').click();
    await expect.poll(() => solverRequests(page)).toBe(before + 1);
    await waitForRun(page);
    await expect(page.getByTestId('status-result')).toHaveText('Result COMPUTED v1.0');
    expect(requests).toEqual([]);
  });
});
