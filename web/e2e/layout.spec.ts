import { expect, gotoSection, horizontalOverflow, SECTIONS, test, VIEWPORTS } from './fixtures';

for (const vp of VIEWPORTS) {
  test.describe(`layout at ${vp.width}x${vp.height}`, () => {
    test.use({ viewport: { width: vp.width, height: vp.height } });

    for (const id of SECTIONS) {
      test(`${id}: one h1, no sideways scroll, status line visible`, async ({ page, problems }) => {
        void problems;
        await gotoSection(page, id);
        expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
        await expect(page.locator('h1')).toHaveCount(1);
        await expect(page.locator('footer.status')).toBeVisible();
        await expect(page.locator('header.header')).toBeVisible();
      });
    }

    test('controls: inline in block C from 768 px, a bottom sheet below', async ({ page, problems }, info) => {
      void problems;
      await gotoSection(page, 'explore');
      const blockC = page.getByTestId('block-c');
      // No control column beside the page at any width (owner decision M7).
      await expect(page.locator('aside')).toHaveCount(0);
      if (vp.width >= 768) {
        await expect(blockC.getByTestId('controls')).toHaveAttribute('data-layout', 'inline');
        await expect(page.getByTestId('open-panel')).toHaveCount(0);
        await expect(page.getByTestId('parameter-sheet')).toHaveCount(0);
        // The five layer groups sit side by side on desktop widths.
        const tops = await blockC.locator('[data-testid^="layer-"]').evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().top)));
        expect(tops).toHaveLength(5);
        if (vp.width >= 1200) expect(new Set(tops).size).toBe(1);
      } else {
        await expect(blockC.getByTestId('controls')).toHaveCount(0);
        await page.getByTestId('open-panel').click();
        const sheet = page.getByTestId('parameter-sheet');
        await expect(sheet).toBeVisible();
        const box = await sheet.boundingBox();
        if (!box) throw new Error('no sheet box');
        expect(box.width).toBe(vp.width);
        expect(Math.round(box.y + box.height)).toBe(vp.height);
        expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
      }
      await info.attach(`explore-${vp.width}`, { body: await page.screenshot({ fullPage: true }), contentType: 'image/png' });
    });

    test('navigation: tabs on wide screens, a menu on phones', async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      const nav = page.getByRole('navigation', { name: 'Sections' });
      if (vp.width < 768) {
        await expect(nav).toBeHidden();
        await page.getByRole('button', { name: 'Menu' }).click();
        await expect(nav).toBeVisible();
        await nav.getByRole('link', { name: 'Research' }).click();
        await expect(page.locator('[data-section="research"] h1')).toBeVisible();
        await expect(nav).toBeHidden();
      } else {
        await expect(nav).toBeVisible();
        await nav.getByRole('link', { name: 'Verify' }).click();
        await expect(page.locator('[data-section="verify"] h1')).toBeVisible();
        await expect(nav.getByRole('link', { name: 'Verify' })).toHaveAttribute('aria-current', 'page');
      }
    });
  });
}
