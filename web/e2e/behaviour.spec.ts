import { expect, gotoSection, SECTIONS, test, waitForRun } from './fixtures';

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
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    await expect.poll(() => page.evaluate(() => getComputedStyle(document.body).backgroundColor)).toBe('rgb(15, 18, 23)');
    await expect.poll(() => page.evaluate(() => localStorage.getItem('strato-theme'))).toBe('dark');
    // A reload while the page is still fetching (the solver's WebAssembly module, the lazily loaded
    // plot chunk, fonts) aborts those requests, and the fixture rightly records each abort as a failed
    // request. Reload only once the first page view has finished loading: the solver has answered,
    // every plot of block B has been drawn (so the plot chunk is in), the fonts are in and no request
    // is in flight.
    await waitForRun(page);
    const graphs = page.locator('[data-testid$="-graph"]');
    await expect.poll(() => graphs.count()).toBeGreaterThan(0);
    await expect
      .poll(() => graphs.evaluateAll((els) => els.every((el) => (el as HTMLElement).dataset.ready === 'true')), { timeout: 20_000 })
      .toBe(true);
    await page.evaluate(() => document.fonts.ready.then(() => undefined));
    await page.waitForLoadState('networkidle');
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

  // Web phase 3: once the solver has run, the status line carries the result label of the run and
  // the solver time; the WebAssembly module reports the same commit as the page.
  // The build hash left the footer (minimal-text rules): the footer links to the build details, and the commit
  // is asserted where it now lives, in the build table of Scientific Documentation.
  test('the status line shows the result label and the solver time, and links to the build commit in Scientific Documentation', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const build = (await (await page.request.get('/STRATO/build.json')).json()) as { commit: string };
    await expect(page.getByTestId('status-build')).not.toContainText(build.commit);
    await waitForRun(page);
    await expect(page.getByTestId('status-result')).toHaveText('Result COMPUTED v2.0');
    await expect(page.getByTestId('status-solver')).toContainText(/ms in this browser/);
    await expect(page.getByTestId('diagnostics-panel')).toContainText(`WebAssembly build ${build.commit}`);
    expect(await page.locator('meta[name="strato-build"]').getAttribute('content')).toBe(build.commit);
    await page.getByTestId('status-build').getByRole('link', { name: 'Build details' }).click();
    await expect(page).toHaveURL(/#\/documentation\/build$/);
    await expect(page.getByTestId('documentation-build')).toContainText(build.commit);
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
