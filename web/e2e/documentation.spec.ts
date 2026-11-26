import { expect, gotoSection, horizontalOverflow, test, VIEWPORTS } from './fixtures';

test.describe('Scientific Documentation', () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test('is the last entry of the navigation and lists its sections', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const links = page.getByRole('navigation', { name: 'Sections' }).getByRole('link');
    await expect(links.last()).toHaveText('Scientific Documentation');
    await links.last().click();
    await expect(page.locator('[data-section="documentation"] h1')).toHaveText('Scientific Documentation');
    await expect(links.last()).toHaveAttribute('aria-current', 'page');
    await expect(page.locator('main h2')).toHaveCount(7);
  });

  test('the contents links move to their section, and a deep link opens it', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'documentation');
    const toc = page.getByRole('navigation', { name: 'Contents of Scientific Documentation' });
    await toc.getByRole('link', { name: 'Technical implementation and build' }).click();
    await expect(page).toHaveURL(/#\/documentation\/implementation$/);
    await expect(page.locator('#doc-implementation')).toBeInViewport();
    await page.goto('./#/documentation/provenance');
    await expect(page.locator('#doc-provenance')).toBeInViewport();
  });

  test('the build table names the commit of this build', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'documentation');
    const build = (await (await page.request.get('/STRATO/build.json')).json()) as { commit: string };
    await expect(page.getByTestId('documentation-build')).toContainText(build.commit);
  });

  test('every contents link can be reached and operated with the keyboard', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'documentation');
    const first = page.getByRole('navigation', { name: 'Contents of Scientific Documentation' }).getByRole('link').first();
    await first.focus();
    await expect(first).toBeFocused();
    await page.keyboard.press('Tab');
    await page.keyboard.press('Shift+Tab');
    await page.keyboard.press('Enter');
    await expect(page).toHaveURL(/#\/documentation\/assumptions$/);
  });
});

test.describe('Scientific Documentation at phone width', () => {
  const phone = VIEWPORTS[0];
  test.use({ viewport: { width: phone.width, height: phone.height } });

  test('has no sideways scroll and 44 px targets in the contents', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'documentation');
    expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
    const heights = await page
      .getByRole('navigation', { name: 'Contents of Scientific Documentation' })
      .getByRole('link')
      .evaluateAll((els) => els.filter((e) => (e as HTMLElement).offsetParent !== null).map((e) => e.getBoundingClientRect().height));
    expect(heights.length).toBeGreaterThanOrEqual(7);
    for (const h of heights) expect(h).toBeGreaterThanOrEqual(44);
  });
});
