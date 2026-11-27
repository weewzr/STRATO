import { expect, gotoAdvanced, test, waitForRun } from './fixtures';

// The sequence of the independent check of the live site, rewritten for the final layout: the manual solver
// (Scientific Documentation, "Advanced solver configuration") runs the baseline problem whatever Explore has
// selected. Run, edit the SiC diffusivity (the computed status goes while the stored run is of other inputs),
// run again (it returns), then select fission-product generation on Explore: the manual solver is unchanged.
for (const vp of [
  { name: '1440 px', width: 1440, height: 900, phone: false },
  { name: '390 px', width: 390, height: 844, phone: true },
] as const) {
  test.describe(`computed status of the manual solver follows its inputs at ${vp.name}`, () => {
    test.use(vp.phone ? { viewport: { width: vp.width, height: vp.height }, isMobile: true, hasTouch: true, deviceScaleFactor: 2 } : { viewport: { width: vp.width, height: vp.height } });

    test('after an edit and a second run the status returns, and the Explore selection does not change it', async ({ page, problems }) => {
      void problems;
      await gotoAdvanced(page);
      await waitForRun(page);
      const header = page.getByTestId('solver-status-line');
      await expect(header).toContainText('COMPUTED v2.0');
      await expect(page.getByTestId('release-value')).toBeVisible();

      // Edit the SiC diffusivity and press Run (in the bottom sheet where the page offers one).
      const open = page.getByTestId('open-panel');
      if ((await open.count()) > 0) {
        await open.click();
        await page.getByTestId('layer-3').locator('summary').click();
      }
      const field = page.getByTestId('d-field-3');
      await field.fill('1e-13');
      await field.press('Tab');
      await page.getByTestId('run').click();
      await expect(page.getByTestId('announce')).toHaveText('Running the current inputs.');
      if ((await page.getByTestId('close-sheet').count()) > 0) await page.getByTestId('close-sheet').click();
      await waitForRun(page);
      await expect(header).toContainText('COMPUTED v2.0');

      // Fission-product generation selected on Explore: nothing changes in the manual solver, and nothing is run.
      await page.goto('./#/explore?physics=generation');
      await expect(page.getByTestId('physics-generation')).toHaveAttribute('aria-checked', 'true');
      await gotoAdvanced(page);
      await expect(page.getByTestId('block-d')).toHaveAttribute('data-supported', 'true');
      await expect(page.getByTestId('release-value')).toBeVisible();
    });
  });
}