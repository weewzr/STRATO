import { expect, gotoAdvanced, gotoSection, horizontalOverflow, SECTIONS, test, VIEWPORTS } from './fixtures';

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

    test('the sticky header is compact: at most 100 px high from 768 px, and as high as --header-height', async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      const h = await page.locator('header.header').evaluate((e) => e.getBoundingClientRect().height);
      if (vp.width >= 768) expect(h).toBeLessThanOrEqual(100);
      const declared = await page.evaluate(() => parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--header-height')));
      if (vp.width >= 768) expect(Math.abs(h - declared)).toBeLessThanOrEqual(2);
    });

    test('everything of the header lies inside the header box and the title lies below it', async ({ page, problems }) => {
      void problems;
      test.skip(vp.width !== 768, 'one run: it sets its own widths');
      await gotoSection(page, 'explore');
      for (const width of [768, 900, 1024, 1099, 1280]) {
      await page.setViewportSize({ width, height: 900 });
      const m = await page.evaluate(() => {
        const r = (e: Element) => e.getBoundingClientRect();
        const header = r(document.querySelector('header.header')!);
        const items = Array.from(document.querySelectorAll('header.header .nav, header.header .nav__link, header.header .nav__tools, header.header .nav__tools > *, header.header .header__brand'))
          .map((e) => ({ cls: e.className, top: r(e).top, bottom: r(e).bottom, width: r(e).width }));
        const h1 = r(document.querySelector('h1')!);
        return { header: { top: header.top, bottom: header.bottom }, items, h1top: h1.top };
      });
      for (const it of m.items) {
        expect(it.top, it.cls).toBeGreaterThanOrEqual(m.header.top - 0.5);
        expect(it.bottom, it.cls).toBeLessThanOrEqual(m.header.bottom + 0.5);
      }
      expect(m.h1top).toBeGreaterThanOrEqual(m.header.bottom);
      const links = await page.locator('header.header .nav__link').evaluateAll((els) => els.map((e) => [e.textContent, e.getBoundingClientRect().width, e.getBoundingClientRect().height] as const));
      for (const [name, w, h] of links) {
        expect(w, String(name)).toBeGreaterThanOrEqual(43.5);
        expect(h, String(name)).toBeGreaterThanOrEqual(43.5);
      }
      }
    });
    test('phone targets: the footer link, the figure links and the switches are 44 px', async ({ page, problems }) => {
      void problems;
      test.skip(vp.width >= 768, 'phone width only');
      await gotoSection(page, 'compare');
      const height = (sel: string) => page.locator(sel).first().evaluate((e) => e.getBoundingClientRect().height);
      expect(await height('.status__link')).toBeGreaterThanOrEqual(43.5);
      expect(await height('a.link-target')).toBeGreaterThanOrEqual(43.5);
      expect(await height('.switch-row .plot-option')).toBeGreaterThanOrEqual(43.5);
      await gotoSection(page, 'research');
      expect(await height('.species-card__head')).toBeGreaterThanOrEqual(43.5);
      await gotoSection(page, 'data');
      expect(await height('caption a.link-target')).toBeGreaterThanOrEqual(43.5);
    });

    test('Explore holds no control column and no numerical control; the manual solver keeps its controls inline from 768 px', async ({ page, problems }, info) => {
      void problems;
      await gotoSection(page, 'explore');
      // No control column beside the page at any width, and none of the numerical controls on Explore (decision 77(f)).
      await expect(page.locator('aside')).toHaveCount(0);
      await expect(page.getByTestId('controls')).toHaveCount(0);
      await expect(page.getByTestId('open-panel')).toHaveCount(0);
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
      await info.attach(`explore-${vp.width}`, { body: await page.screenshot({ fullPage: true }), contentType: 'image/png' });
      await page.waitForLoadState('networkidle');
      await gotoAdvanced(page);
      const controls = page.getByTestId('advanced-solver').getByTestId('controls');
      if (vp.width >= 768) {
        await expect(controls).toHaveAttribute('data-layout', 'inline');
        // The five layer groups sit side by side on desktop widths.
        const tops = await page.getByTestId('block-d').locator('[data-testid^="layer-"]').evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().top)));
        expect(tops).toHaveLength(5);
        if (vp.width >= 1200) expect(new Set(tops).size).toBe(1);
      }
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
