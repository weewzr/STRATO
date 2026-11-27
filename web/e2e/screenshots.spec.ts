import { join } from 'node:path';
import { expect, gotoSection, SECTIONS, test } from './fixtures';

// Review screenshots, written only when SHOTS_DIR is set (they are not test evidence and are not committed).
const dir = process.env.SHOTS_DIR ?? '';

for (const vp of [
  { width: 1440, height: 900, mobile: false },
  { width: 390, height: 844, mobile: true },
]) {
  for (const scheme of ['light', 'dark'] as const) {
    test.describe(`screenshots ${vp.width} ${scheme}`, () => {
      test.use({ viewport: { width: vp.width, height: vp.height }, isMobile: vp.mobile, hasTouch: vp.mobile, colorScheme: scheme });

      test('every section, the first screen and an interaction', async ({ page, problems }) => {
        void problems;
        test.skip(!dir, 'SHOTS_DIR not set');
        for (const id of SECTIONS) {
          await gotoSection(page, id);
          await page.evaluate(() => document.fonts.ready);
          await page.screenshot({ path: join(dir, `${id}-${vp.width}-${scheme}.png`), fullPage: true });
        }
        await gotoSection(page, 'explore');
        await expect(page.getByTestId('graph-panel')).toBeVisible();
        await expect(page.getByTestId('explore-plot-release-graph')).toBeVisible({ timeout: 30_000 });
        await page.evaluate(() => document.fonts.ready);
        await page.screenshot({ path: join(dir, `explore-fold-${vp.width}-${scheme}.png`), fullPage: false });
        // An interaction: temperature-dependent diffusion (which draws the model curve of a dataset), the numerical sections open, the experimental comparison.
        await page.goto('./#/explore?physics=temperature&open=method-a,discretisation,pseudocode&graph=experiment');
        await expect(page.getByTestId('explore-plot-experiment-graph')).toBeVisible({ timeout: 30_000 });
        await page.evaluate(() => document.fonts.ready);
        await page.screenshot({ path: join(dir, `explore-interaction-${vp.width}-${scheme}.png`), fullPage: true });
        // The empty panel of an operation without a result.
        await page.getByTestId('operation-shutdown').click();
        await expect(page.getByTestId('graph-empty')).toBeVisible();
        await page.getByTestId('interactive-graphs').screenshot({ path: join(dir, `explore-empty-${vp.width}-${scheme}.png`) });
      });
    });
  }
}
