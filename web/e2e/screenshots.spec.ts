import { join } from 'node:path';
import { expect, gotoSection, SECTIONS, test, waitForRun } from './fixtures';

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
        await waitForRun(page);
        await page.evaluate(() => document.fonts.ready);
        await page.screenshot({ path: join(dir, `explore-fold-${vp.width}-${scheme}.png`), fullPage: false });
        // Web phase 3: the solver results of block B.
        await page.getByTestId('block-e').screenshot({ path: join(dir, `explore-results-${vp.width}-${scheme}.png`) });
        await page.getByTestId('inspect-3').click();
        if (vp.width < 768) {
          await page.getByTestId('open-panel').click();
          await expect(page.getByTestId('parameter-sheet')).toBeVisible();
          await page.getByTestId('layer-2').locator('summary').click();
        }
        await page.getByTestId('r-field-2').fill('1e-4');
        await page.screenshot({ path: join(dir, `explore-interaction-${vp.width}-${scheme}.png`), fullPage: false });
        // The radius is now below the one inside it: the solver rejects the inputs and says why.
        if (vp.width < 768) await page.getByTestId('close-sheet').click();
        await waitForRun(page, 'rejected');
        await page.getByTestId('block-d').screenshot({ path: join(dir, `explore-rejected-${vp.width}-${scheme}.png`) });
      });
    });
  }
}
