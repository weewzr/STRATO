import type { Page } from '@playwright/test';
import { join } from 'node:path';
import { expect, gotoSection, horizontalOverflow, solverRequests, test, waitForRun } from './fixtures';

// The learning sequence of the owner's directive of 26 Nov 2026 (Section 12, items 1 to 11), walked through the
// shared selection: base equation, each mechanism, each scenario, the numerical items that appear, and the
// messages of the live simulation and the results. The build under test reports the baseline only, so every
// other selection must show its equation and explanations and, in D and E, the reason why nothing is run
// or shown (no substituted model). The walk runs at 1440, 768 and 390 px in both colour schemes and both
// motion settings; one more pass uses the keyboard only. Screenshots are written only when SHOTS_DIR is set.

const dir = process.env.SHOTS_DIR ?? '';
const MECHANISMS = [
  { id: 'generation', name: 'fission-product generation', piece: 'generation', item: 9 },
  { id: 'decay', name: 'radioactive decay', piece: 'decay', item: 10 },
  { id: 'temperature', name: 'temperature-dependent diffusion', piece: 'operator', item: 11 },
] as const;
const SCENARIOS = [
  { id: 'steady', name: 'steady-state operation' },
  { id: 'shutdown', name: 'shutdown and cooldown' },
  { id: 'startup', name: 'startup' },
  { id: 'control-rod', name: 'control-rod insertion' },
  { id: 'refuelling', name: 'fuel-removal' },
] as const;

async function expectFits(page: Page, width: number, label: string) {
  expect(await horizontalOverflow(page), label).toBeLessThanOrEqual(0);
  for (const id of ['equation-governing', 'scenario-picker', 'block-d', 'block-e']) {
    // The results block is absent for a selection that the build does not simulate.
    if (id === 'block-e' && (await page.getByTestId('block-e').count()) === 0) continue;
    const box = await page.getByTestId(id).boundingBox();
    expect((box?.x ?? 0) + (box?.width ?? 0), `${label}: ${id}`).toBeLessThanOrEqual(width + 0.5);
  }
}

/** D and E of a selection that the build does not compute: the reason, the solver identity, and nothing of a model. */
async function expectNothingSubstituted(page: Page, name: string) {
  await expect(page.getByTestId('block-d')).toHaveAttribute('data-supported', 'false');
  await expect(page.getByTestId('block-d')).toHaveAttribute('data-run-status', 'none');
  await expect(page.getByTestId('unsupported-simulation')).toContainText('does not implement');
  await expect(page.getByTestId('unsupported-simulation')).toContainText(name);
  await expect(page.getByTestId('unsupported-simulation')).toContainText('no result is shown for this selection');
  await expect(page.getByTestId('block-e')).toHaveCount(0);
  // The solver stays identified in D by its status line, and the physics of the build by one line.
  await expect(page.getByTestId('solver-status-line').first()).toContainText('Method A: BTCS');
  // The condition is stated once (the bold sentence in D): no second line on what the build computes, no note under Run, no result block.
  await expect(page.getByTestId('active-model')).toHaveCount(0);
  await expect(page.getByTestId('run-unsupported')).toHaveCount(0);
  await expect(page.getByTestId('block-d')).not.toContainText('reruns');
  for (const id of ['particle', 'release-readout', 'profile-figure', 'release-figure', 'balance-panel', 'diagnostics-panel']) {
    await expect(page.getByTestId(id), id).toHaveCount(0);
  }
}

for (const vp of [
  { width: 1440, height: 900, mobile: false },
  { width: 768, height: 1024, mobile: false },
  { width: 390, height: 844, mobile: true },
]) {
  for (const scheme of ['light', 'dark'] as const) {
    for (const motion of ['no-preference', 'reduce'] as const) {
      test.describe(`learning sequence at ${vp.width} px, ${scheme}, motion ${motion}`, () => {
        test.use({ viewport: { width: vp.width, height: vp.height }, isMobile: vp.mobile, hasTouch: vp.mobile, colorScheme: scheme, reducedMotion: motion });

        test('base equation, each mechanism, each scenario, then the baseline again', async ({ page, problems }) => {
          void problems;
          await gotoSection(page, 'explore');
          await page.evaluate(() => document.fonts.ready);
          await page.getByTestId('numerical-toggle').click();
          const tag = `${vp.width}-${scheme}-${motion}`;

          // 1 and 6: the base equation and the simulation of the baseline, with the solver and its physics identified.
          await waitForRun(page);
          await expect(page.getByTestId('governing-equation')).toHaveAttribute('data-physics', 'baseline');
          expect(await page.locator('[data-testid="equation-governing"] [data-piece]').evaluateAll((e) => e.map((p) => (p as HTMLElement).dataset.piece))).toEqual(['lhs', 'operator']);
          await expect(page.getByTestId('solver-status-line')).toContainText('Method A: BTCS');
          await expect(page.getByTestId('active-model')).toContainText('Computes: Baseline diffusion');
          expect(await page.getByTestId('active-model').innerText()).not.toMatch(/[.!?]\s+[a-z]/);
          expect(await page.getByTestId('block-e').innerText()).not.toMatch(/Compare and Validate/);
          await expect(page.getByTestId('num-number-1')).toBeVisible();
          await expect(page.getByTestId('num-item-1')).not.toContainText('Item 1');
          await expect(page.getByTestId('particle')).toBeVisible();
          await expect(page.getByTestId('release-readout')).toBeVisible();
          await expect(page.getByTestId('balance-panel')).toBeVisible();
          await expect(page.getByTestId('diagnostics-panel')).toBeVisible();
          for (const id of [9, 10, 11]) await expect(page.getByTestId(`num-item-${id}`), `item ${id} at baseline`).toHaveCount(0);
          await expectFits(page, vp.width, `baseline ${tag}`);
          if (dir) await page.screenshot({ path: join(dir, `sequence-baseline-${tag}.png`), fullPage: true });

          // 2, 4 and 7: each mechanism changes the equation and the numerical items, and is not presented as simulated.
          const before = await solverRequests(page);
          for (const m of MECHANISMS) {
            await page.getByTestId(`physics-${m.id}`).click();
            await expect(page.getByTestId('governing-equation')).toHaveAttribute('data-physics', m.id);
            await expect(page.locator(`[data-piece="${m.piece}"]`)).toBeVisible();
            await expect(page.getByTestId('physics-status-' + m.id)).toHaveText('Planned');
            await expect(page.getByTestId(`num-item-${m.item}`)).toBeVisible();
            for (const other of MECHANISMS.filter((o) => o.id !== m.id)) await expect(page.getByTestId(`num-item-${other.item}`)).toHaveCount(0);
            await page.getByTestId(`num-toggle-${m.item}`).click();
            // The item says in one word that the build does not report the mechanism (the sentence that explained it is the legend of the physics controls).
            await expect(page.getByTestId(`num-status-${m.item}`)).toHaveAttribute('data-status', 'planned');
            // Its live values say that the build has not run it, and show no number of the baseline run.
            await expect(page.getByTestId(`num-item-${m.item}`).getByTestId('live-numbers-unsupported')).toContainText('No simulation runs for the selection');
            await expect(page.getByTestId(`num-item-${m.item}`).getByTestId('live-numbers')).toHaveCount(0);
            await page.getByTestId(`num-toggle-${m.item}`).click();
            await expectNothingSubstituted(page, m.name);
            await expectFits(page, vp.width, `${m.id} ${tag}`);
          }
          await page.getByTestId('physics-combined').click();
          for (const m of MECHANISMS) await expect(page.getByTestId(`num-item-${m.item}`)).toBeVisible();
          await expectNothingSubstituted(page, 'radioactive decay');

          // 3, 4 and 7: each scenario selects the combined model, writes its note inside the numerical items, and shows no run.
          for (const s of SCENARIOS) {
            await page.getByTestId(`scenario-${s.id}`).click();
            await expect(page.getByTestId('physics-combined')).toHaveAttribute('aria-checked', 'true');
            await expect(page.getByTestId('governing-equation')).toHaveAttribute('data-scenario', s.id);
            await expect(page.getByTestId('scenario-explanation')).toBeVisible();
            await expect(page.getByTestId('scenario-status-' + s.id)).toHaveText('Planned');
            await expectNothingSubstituted(page, s.name);
            await expectFits(page, vp.width, `${s.id} ${tag}`);
          }
          // A scenario writes a note into the item where it acts, and the item is opened without losing the selection.
          await page.getByTestId('scenario-shutdown').click();
          await page.getByTestId('num-toggle-9').click();
          await expect(page.getByTestId('scenario-note-9')).toHaveAttribute('data-scenario', 'shutdown');
          await expect(page.getByTestId('num-item-9').getByTestId('live-numbers-unsupported')).toContainText('No simulation runs for the selection');
          if (dir) await page.screenshot({ path: join(dir, `sequence-scenario-${tag}.png`), fullPage: true });

          // The simulation was never asked for a model of another selection.
          await page.waitForTimeout(300);
          expect(await solverRequests(page)).toBe(before);

          // 1 and 6 again: the baseline clears the scenario and the run returns, with nothing left of the other selection.
          await page.getByTestId('physics-baseline').click();
          await waitForRun(page);
          await expect(page.getByTestId('scenario-picker').locator('[aria-checked="true"]')).toHaveCount(0);
          await expect(page.getByTestId('particle')).toBeVisible();
          await expect(page.getByTestId('release-readout').locator('.badge')).toHaveText('COMPUTED v2.0');
          await expect(page.getByTestId('unsupported-simulation')).toHaveCount(0);
          await expectFits(page, vp.width, `baseline again ${tag}`);
        });
      });
    }
  }
}

test.describe('learning sequence with the keyboard only', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  /** Presses Tab until the focused element satisfies the test; fails if it is not reached. */
  async function tabTo(page: Page, matches: (el: HTMLElement | null) => boolean, label: string) {
    for (let i = 0; i < 120; i++) {
      await page.keyboard.press('Tab');
      if (await page.evaluate(`(${matches.toString()})(document.activeElement)`)) return;
    }
    throw new Error(`the keyboard did not reach ${label}`);
  }

  test('selects physics, a scenario and the numerical items without a pointer', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await waitForRun(page);
    await tabTo(page, (el) => (el?.dataset.testid ?? '').startsWith('physics-') && el?.getAttribute('role') === 'radio', 'the physics controls');
    await expect(page.getByTestId('physics-baseline')).toBeFocused();
    await page.keyboard.press('ArrowRight');
    await expect(page.getByTestId('physics-generation')).toBeFocused();
    await expect(page.getByTestId('governing-equation')).toHaveAttribute('data-physics', 'generation');
    await expect(page.getByTestId('unsupported-simulation')).toContainText('does not implement fission-product generation');
    await page.keyboard.press('ArrowRight');
    await page.keyboard.press('ArrowRight');
    await expect(page.getByTestId('governing-equation')).toHaveAttribute('data-physics', 'temperature');
    await page.keyboard.press('ArrowRight');
    await expect(page.getByTestId('governing-equation')).toHaveAttribute('data-physics', 'combined');
    // Next to the scenario row, then a scenario.
    await tabTo(page, (el) => el?.closest('[data-testid="scenario-picker"]') !== null && el?.getAttribute('role') === 'radio', 'the scenario controls');
    // With no scenario selected the tab stop is the first button; Space selects it.
    await expect(page.getByTestId('scenario-steady')).toBeFocused();
    await page.keyboard.press('Space');
    await expect(page.getByTestId('governing-equation')).toHaveAttribute('data-scenario', 'steady');
    await page.keyboard.press('ArrowRight');
    await expect(page.getByTestId('governing-equation')).toHaveAttribute('data-scenario', 'shutdown');
    await expect(page.getByTestId('unsupported-simulation')).toContainText('shutdown and cooldown');
    // The Numerical Interpretation, then one of its items.
    await tabTo(page, (el) => el?.dataset.testid === 'numerical-toggle', 'the Numerical Interpretation');
    await page.keyboard.press('Enter');
    await expect(page.getByTestId('num-items')).toBeVisible();
    await tabTo(page, (el) => (el?.dataset.testid ?? '').startsWith('num-toggle-'), 'an item of the Numerical Interpretation');
    await page.keyboard.press('Enter');
    await expect(page.locator('[data-testid^="num-toggle-"][aria-expanded="true"]')).toHaveCount(1);
    // The selection is the same everywhere: back to the baseline with the keyboard, the run returns.
    await page.getByTestId('physics-combined').focus();
    await page.keyboard.press('Home');
    await expect(page.getByTestId('physics-baseline')).toBeFocused();
    await waitForRun(page);
    await expect(page.getByTestId('particle')).toBeVisible();
    await expect(page.getByTestId('scenario-picker').locator('[aria-checked="true"]')).toHaveCount(0);
  });
});
