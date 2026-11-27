import { expect, gotoSection, test, waitForRun } from './fixtures';

// The sequence of the independent check of the live site: run, edit the SiC diffusivity, run again, then select
// fission-product generation. No computed or verified status, no run line, no baseline line and no result number
// may stay on the page for a selection the build does not compute.
for (const vp of [
  { name: '1440 px', width: 1440, height: 900, phone: false },
  { name: '390 px', width: 390, height: 844, phone: true },
] as const) {
  test.describe(`computed status follows the selection at ${vp.name}`, () => {
    test.use(vp.phone ? { viewport: { width: vp.width, height: vp.height }, isMobile: true, hasTouch: true, deviceScaleFactor: 2 } : { viewport: { width: vp.width, height: vp.height } });

    test('after an edit and a second run, generation shows no badge, no run line, no baseline line and no number', async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      await waitForRun(page);
      const header = page.getByTestId('solver-status-line');
      await expect(header).toContainText('COMPUTED v2.0');
      await expect(page.getByTestId('release-value')).toBeVisible();

      // Edit the SiC diffusivity and press Run (in the bottom sheet on a phone).
      if (vp.phone) {
        await page.getByTestId('open-panel').click();
        await page.getByTestId('layer-3').locator('summary').click();
      }
      const field = page.getByTestId('d-field-3');
      await field.fill('1e-13');
      await field.press('Tab');
      await page.getByTestId('run').click();
      await expect(page.getByTestId('announce')).toHaveText('Running the current inputs.');
      if (vp.phone) await page.getByTestId('close-sheet').click();
      await waitForRun(page);
      await expect(header).toContainText('COMPUTED v2.0');

      // Fission-product generation.
      await page.getByTestId('physics-generation').click();
      await expect(page.getByTestId('block-d')).toHaveAttribute('data-supported', 'false');
      await expect(page.getByTestId('unsupported-simulation')).toContainText('No simulation for this selection');
      await expect(header).toContainText('Method A: BTCS');
      await expect(header).not.toContainText(/COMPUTED|VERIFIED/i);
      await expect(page.locator('.badge')).toHaveCount(0);
      await expect(page.getByTestId('release-value')).toHaveCount(0);
      await expect(page.getByTestId('status-result')).not.toContainText('COMPUTED');
      const text = (await page.locator('body').innerText()) + (await page.locator('body').textContent());
      expect(text).not.toContain('Running the current inputs.');
      expect(text).not.toContain('Inputs equal the baseline.');
      expect(text).not.toContain('inputs differ from the baseline');
      expect(text).not.toContain('input differs from the baseline');
    });
  });
}
