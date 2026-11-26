import { join } from 'node:path';
import { claimsIn } from '../tests/forbiddenClaims';
import { expect, gotoSection, horizontalOverflow, solverRequests, test, waitForRun } from './fixtures';

// The operating-scenario row of the Explore page (owner directive of 26 Nov 2026, Sections 3 and 6): five
// buttons directly beneath the physics controls, the symbols they change in the equation, the explanation,
// the status of the build on each button, keyboard operation, the address, no forbidden claim on the built
// page, and the layout at 1440, 768 and 390 px in both themes. Screenshots only when SHOTS_DIR is set.

const dir = process.env.SHOTS_DIR ?? '';
const SCENARIOS = ['steady', 'shutdown', 'startup', 'control-rod', 'refuelling'] as const;
const LABELS = ['Steady-state operation', 'Shutdown and cooldown', 'Startup', 'Control-rod insertion', 'Refuelling'];

test.describe('scenario row at 1440 x 900', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('five buttons in order, beneath the physics controls, with the status of this build', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const group = page.getByRole('radiogroup', { name: 'Operating scenario' });
    await expect(group.getByRole('radio')).toHaveCount(5);
    for (let i = 0; i < 5; i++) await expect(group.getByRole('radio').nth(i).locator('.physics__name')).toHaveText(LABELS[i] ?? '');
    const physics = await page.getByTestId('physics-picker').boundingBox();
    const scen = await group.boundingBox();
    expect(scen?.y ?? 0).toBeGreaterThan((physics?.y ?? 0) + (physics?.height ?? 0));
    // Baseline only today: no scenario is selected, and all five (steady-state operation too) are planned, because the baseline run is not a scenario.
    for (const s of SCENARIOS) await expect(page.getByTestId(`scenario-status-${s}`)).toHaveText('Planned');
    await expect(group.locator('[aria-checked="true"]')).toHaveCount(0);
    await expect(page.getByTestId('scenario-rule')).toContainText('choosing one selects the combined model');
  });

  test('each scenario changes only G and D in the equation, selects the combined model and says it is not simulated here', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    await page.getByTestId('numerical-toggle').click();
    const before = await solverRequests(page);
    for (const s of SCENARIOS) {
      await page.getByTestId(`scenario-${s}`).click();
      await expect(page.getByTestId('physics-combined')).toHaveAttribute('aria-checked', 'true');
      await expect(page.getByTestId('governing-equation')).toHaveAttribute('data-scenario', s);
      // The Numerical Interpretation says that no simulation runs, not that one does.
      await expect(page.getByTestId('live-note')).toContainText('No simulation runs for the selected physics or scenario');
      await expect(page.getByTestId('live-note')).not.toContainText('Live: the simulation below solves');
      // The same four pieces as the combined model, with the two symbols changed and highlighted.
      expect(await page.locator('[data-testid="equation-governing"] [data-piece]').evaluateAll((e) => e.map((p) => (p as HTMLElement).dataset.piece))).toEqual([
        'lhs',
        'operator',
        'generation',
        'decay',
      ]);
      const eq = page.getByTestId('equation-governing');
      // Steady-state operation holds G and T constant: nothing changes. The others change G and D(r,T) only.
      await expect(eq.locator('.geq-hi--scenario')).toHaveCount(s === 'steady' ? 0 : 2);
      const annotations = await eq.locator('annotation[encoding="application/x-tex"]').allTextContents();
      if (s === 'steady') {
        expect(annotations.join(' ')).not.toContain('G(r,t)');
      } else {
        expect(annotations.join(' ')).toContain('G(r,t)');
        expect(annotations.join(' ')).toContain('D(r,T(t))');
      }
      expect(annotations.join(' ')).not.toMatch(/rod|refuel|\\rho/i);
      await expect(page.locator('.katex-error')).toHaveCount(0);
      await expect(page.getByTestId('scenario-explanation')).toContainText(`${LABELS[SCENARIOS.indexOf(s)]}`);
      await expect(page.getByTestId('unsupported-simulation')).toContainText('does not implement');
      await expect(page.getByTestId('unsupported-results')).toContainText('no result is shown for this selection');
      await expect(page.getByTestId('particle')).toHaveCount(0);
    }
    await page.waitForTimeout(300);
    expect(await solverRequests(page)).toBe(before);
    // The baseline physics clears the scenario and runs the baseline, labelled as such: nothing else is substituted.
    await page.getByTestId('physics-baseline').click();
    await expect(page.getByTestId('scenario-picker').locator('[aria-checked="true"]')).toHaveCount(0);
    await waitForRun(page);
    await expect(page.getByTestId('particle')).toBeVisible();
    await expect(page.getByTestId('strip-stage')).toContainText('Physics: Baseline diffusion; scenario: none');
    await expect(page.getByTestId('live-note')).toContainText('Live: the simulation below solves these equations');
  });

  test('steady-state operation says that the concentration field is not assumed stationary, and why', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.getByTestId('scenario-steady').click();
    const text = page.getByTestId('scenario-explanation');
    await expect(text).toContainText('does not impose that the time derivative of C is zero');
    await expect(text).toContainText('the run starts from its initial state (a fresh particle holds no inventory)');
    await expect(text).toContainText('only over times longer than the irradiation');
    await expect(text).not.toContainText('still being generated');
  });

  test('the focused scenario button stays inside the viewport after a keyboard selection', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const first = page.getByTestId('scenario-steady');
    await first.scrollIntoViewIfNeeded();
    await first.focus();
    await page.keyboard.press('ArrowRight');
    await expect(page.getByTestId('scenario-shutdown')).toBeFocused();
    await page.waitForTimeout(400);
    const vp = page.viewportSize();
    const box = await page.getByTestId('scenario-shutdown').boundingBox();
    const status = await page.locator('.status').boundingBox();
    expect(box?.y ?? -1).toBeGreaterThanOrEqual(0);
    expect((box?.y ?? 0) + (box?.height ?? 0)).toBeLessThanOrEqual((status?.y ?? vp?.height ?? 0) + 1);
  });

  test('the scenario is written to the address, survives navigation, and a copied address reproduces it', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.getByTestId('scenario-control-rod').click();
    await expect(page).toHaveURL(/#\/explore\?physics=combined&scenario=control-rod$/);
    const nav = page.getByRole('navigation', { name: 'Sections' });
    await nav.getByRole('link', { name: 'Compare' }).click();
    await expect(page.locator('[data-section="compare"] h1')).toBeVisible();
    await nav.getByRole('link', { name: 'Explore' }).click();
    await expect(page.getByTestId('scenario-control-rod')).toHaveAttribute('aria-checked', 'true');
    await page.goto('./#/explore?scenario=refuelling');
    await expect(page.getByTestId('scenario-refuelling')).toHaveAttribute('aria-checked', 'true');
    await page.waitForLoadState('networkidle');
    await page.reload();
    await expect(page.getByTestId('scenario-refuelling')).toHaveAttribute('aria-checked', 'true');
    await expect(page.getByTestId('physics-combined')).toHaveAttribute('aria-checked', 'true');
    // Another physics clears the scenario, and the address follows.
    await page.getByTestId('physics-decay').click();
    await expect(page.getByTestId('scenario-picker').locator('[aria-checked="true"]')).toHaveCount(0);
    await expect(page).toHaveURL(/#\/explore\?physics=decay$/);
  });

  test('is operated by the keyboard, with a visible focus ring', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.getByTestId('scenario-steady').focus();
    await page.keyboard.press('ArrowRight');
    await expect(page.getByTestId('scenario-shutdown')).toBeFocused();
    await expect(page.getByTestId('scenario-shutdown')).toHaveAttribute('aria-checked', 'true');
    await page.keyboard.press('End');
    await expect(page.getByTestId('scenario-refuelling')).toBeFocused();
    await page.keyboard.press('Home');
    await expect(page.getByTestId('scenario-steady')).toBeFocused();
    await page.keyboard.press('ArrowLeft');
    await expect(page.getByTestId('scenario-refuelling')).toHaveAttribute('aria-checked', 'true');
    const ring = await page.getByTestId('scenario-refuelling').evaluate((e) => ({ style: getComputedStyle(e).outlineStyle, width: parseFloat(getComputedStyle(e).outlineWidth) }));
    expect(ring.style).toBe('solid');
    expect(ring.width).toBeGreaterThanOrEqual(2);
    // One tab stop in the group: Tab leaves it for the next control.
    await page.keyboard.press('Tab');
    expect(await page.evaluate(() => (document.activeElement as HTMLElement | null)?.closest('[data-testid="scenario-picker"]') === null)).toBe(true);
    await expect(page.getByTestId('scenario-announce')).toContainText('Refuelling selected');
  });

  test('the built page carries no forbidden claim, in any scenario, and defines the symbols with units', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.getByTestId('numerical-toggle').click();
    for (const s of SCENARIOS) {
      await page.getByTestId(`scenario-${s}`).click();
      const text = await page.locator('[data-section="explore"]').innerText();
      expect(claimsIn(text), s).toEqual([]);
      await expect(page.getByTestId('scenario-numerics')).toContainText(LABELS[SCENARIOS.indexOf(s)] ?? '');
      await expect(page.getByTestId('scenario-params').locator('li')).toHaveCount(s === 'steady' ? 2 : 4);
      await expect(page.getByTestId('scenario-params')).toContainText('Unit:');
    }
    await expect(page.getByTestId('scenario-explanation')).toContainText('future scenario');
    await expect(page.getByTestId('scenario-explanation')).toContainText('Mechanical refuelling is not simulated');
    await expect(page.getByTestId('scenario-explanation')).toContainText('Fuel removal');
  });
});

for (const vp of [
  { width: 1440, height: 900, mobile: false },
  { width: 768, height: 1024, mobile: false },
  { width: 390, height: 844, mobile: true },
]) {
  for (const scheme of ['light', 'dark'] as const) {
    test.describe(`scenario row at ${vp.width} px, ${scheme}`, () => {
      test.use({ viewport: { width: vp.width, height: vp.height }, isMobile: vp.mobile, hasTouch: vp.mobile, colorScheme: scheme });

      test('every scenario fits the width, no sideways page scroll, 44 px targets', async ({ page, problems }) => {
        void problems;
        await gotoSection(page, 'explore');
        await page.evaluate(() => document.fonts.ready);
        for (const s of SCENARIOS) {
          await page.getByTestId(`scenario-${s}`).click();
          await expect(page.getByTestId('governing-equation')).toHaveAttribute('data-scenario', s);
          expect(await horizontalOverflow(page), s).toBeLessThanOrEqual(0);
          const eq = await page.getByTestId('equation-governing').boundingBox();
          expect((eq?.x ?? 0) + (eq?.width ?? 0), s).toBeLessThanOrEqual(vp.width + 0.5);
          const row = await page.getByTestId('scenario-picker').boundingBox();
          expect((row?.x ?? 0) + (row?.width ?? 0), s).toBeLessThanOrEqual(vp.width + 0.5);
          const box = await page.getByTestId('block-b').boundingBox();
          expect((box?.x ?? 0) + (box?.width ?? 0), s).toBeLessThanOrEqual(vp.width + 0.5);
          if (dir && (s === 'startup' || s === 'steady')) await page.getByTestId('block-b').screenshot({ path: join(dir, `scenario-${s}-${vp.width}-${scheme}.png`) });
        }
        for (const s of SCENARIOS) {
          const b = await page.getByTestId(`scenario-${s}`).boundingBox();
          expect(b?.height ?? 0, s).toBeGreaterThanOrEqual(44);
          expect(b?.width ?? 0, s).toBeGreaterThanOrEqual(44);
        }
        // After a keyboard selection the focused button is still inside the viewport and clear of the status line.
        await page.getByTestId('scenario-steady').focus();
        await page.keyboard.press('ArrowRight');
        await expect(page.getByTestId('scenario-shutdown')).toBeFocused();
        await page.waitForTimeout(400);
        const kb = await page.getByTestId('scenario-shutdown').boundingBox();
        const bar = await page.locator('.status').boundingBox();
        expect(kb?.y ?? -1).toBeGreaterThanOrEqual(0);
        expect((kb?.y ?? 0) + (kb?.height ?? 0)).toBeLessThanOrEqual(Math.min(vp.height, bar?.y ?? vp.height) + 1);
        if (dir) {
          await page.getByTestId('scenario-startup').click();
          await page.getByTestId('numerical-toggle').click();
          await page.screenshot({ path: join(dir, `explore-scenario-${vp.width}-${scheme}.png`), fullPage: true });
        }
      });
    });
  }
}
