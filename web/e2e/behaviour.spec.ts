import { expect, gotoSection, SECTIONS, test } from './fixtures';

test.describe('reduced motion', () => {
  test.use({ viewport: { width: 1280, height: 800 }, reducedMotion: 'reduce' });

  test('transitions are 0 s and every section renders', async ({ page, problems }) => {
    void problems;
    for (const id of SECTIONS) {
      await gotoSection(page, id);
      const durations = await page.evaluate(() =>
        Array.from(document.querySelectorAll('.btn, .nav__link, .inspector__item')).map((el) => getComputedStyle(el).transitionDuration),
      );
      expect(durations.length).toBeGreaterThan(0);
      for (const d of durations) for (const part of d.split(',')) expect(parseFloat(part)).toBe(0);
    }
  });
});

test.describe('themes', () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test('follow the system setting', async ({ page, problems }) => {
    void problems;
    await page.emulateMedia({ colorScheme: 'light' });
    await gotoSection(page, 'explore');
    expect(await page.evaluate(() => getComputedStyle(document.body).backgroundColor)).toBe('rgb(251, 251, 250)');
    await page.emulateMedia({ colorScheme: 'dark' });
    expect(await page.evaluate(() => getComputedStyle(document.body).backgroundColor)).toBe('rgb(15, 18, 23)');
  });

  test('the manual toggle overrides the system and is remembered', async ({ page, problems }) => {
    void problems;
    await page.emulateMedia({ colorScheme: 'light' });
    await gotoSection(page, 'explore');
    const toggle = page.getByTestId('theme-toggle');
    await expect(toggle).toHaveAttribute('aria-pressed', 'false');
    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-pressed', 'true');
    expect(await page.evaluate(() => getComputedStyle(document.body).backgroundColor)).toBe('rgb(15, 18, 23)');
    await page.reload();
    await expect(page.getByTestId('theme-toggle')).toHaveAttribute('aria-pressed', 'true');
    expect(await page.evaluate(() => document.documentElement.getAttribute('data-theme'))).toBe('dark');
  });
});

test.describe('content and data', () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test('no console error and no failed request in any section', async ({ page, problems }) => {
    void problems;
    for (const id of SECTIONS) await gotoSection(page, id);
  });

  test('the case CSV download carries its provenance header', async ({ page, request, problems }) => {
    void problems;
    await gotoSection(page, 'data');
    const href = await page.getByTestId('download-case').getAttribute('href');
    expect(href).toBe('/STRATO/data/synthetic_five_layer.case.csv');
    const res = await request.get(href as string);
    expect(res.status()).toBe(200);
    const text = await res.text();
    expect(text.split('\n')[0]).toBe('# label: INPUT');
    expect(text).toMatch(/# input_sha256: [0-9a-f]{64}/);
    expect(text).toContain('# strato_version: ');
  });

  test('the parameter table shows the case-file values and marks changed inputs', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const field = page.getByTestId('d-field-0');
    await field.fill('3e-12');
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('link', { name: 'Data' }).click();
    const table = page.getByTestId('input-table');
    await expect(table.locator('tr.is-changed')).toHaveCount(1);
    await expect(table.locator('tr.is-changed')).toContainText('3e-12');
    await expect(table.locator('tr.is-changed')).toContainText('1e-12');
  });

  test('the status line names the build commit and states that no result exists yet', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const build = (await (await page.request.get('/STRATO/build.json')).json()) as { commit: string };
    await expect(page.getByTestId('status-commit')).toContainText(build.commit);
    await expect(page.getByTestId('status-result')).toHaveText('Result: none in this build');
    expect(await page.locator('meta[name="strato-build"]').getAttribute('content')).toBe(build.commit);
  });

  test('the site does not link the prompt folder', async ({ page, problems }) => {
    void problems;
    for (const id of SECTIONS) {
      await gotoSection(page, id);
      const hrefs = await page.locator('a[href]').evaluateAll((as) => as.map((a) => a.getAttribute('href') ?? ''));
      for (const h of hrefs) expect(h).not.toMatch(/(^|\/)prompt\//);
    }
  });
});
