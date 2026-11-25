import { test, expect, gotoSection, horizontalOverflow, waitForRun } from './fixtures';

// The derivation and register documents rendered into the build by crates/strato-docsite (web
// phase 8). Continuous integration renders them into web/dist right after `npm run build`; to run
// this file locally, run the same command first (web/README.md, "Documents").

const DOCUMENTS = [
  'docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.html',
  'docs/mathematics/v1.0.0/STRATO_Final_Deliverable.html',
  'docs/mathematics/STRATO_Phase2_Derivations.html',
  'docs/mathematics/STRATO_Phase2_Pseudocode.html',
  'docs/mathematics/phase1/index.html',
  'docs/mathematics/phase1/TRISO_Finite_Volume_Derivation.html',
  'docs/mathematics/phase1/TRISO_Finite_Volume_Pseudocode.html',
  'data/reference/index.html',
  'data/reference/STRATO_Parameter_Register.html',
  'data/reference/B6_audit_log.html',
];

const LISTS = ['docs/index.html', 'docs/mathematics/index.html', 'docs/mathematics/v1.0.0/index.html', 'tests/golden/v1.0/index.html'];

const expectedCommit = (process.env.EXPECT_COMMIT ?? '').slice(0, 7);

test.describe('rendered documents', () => {
  test.setTimeout(180_000);

  for (const path of DOCUMENTS) {
    test(`${path}: every formula typeset, labelled with its source and commit`, async ({ page, problems }) => {
      void problems;
      await page.goto(`./${path}`);
      await expect(page.locator('main h1').first()).toBeVisible();
      await expect(page.locator('.doc-meta')).toContainText('Status');
      await expect(page.locator('.doc-meta')).toContainText('Source');
      if (expectedCommit) await expect(page.locator('meta[name="strato-build"]')).toHaveAttribute('content', expectedCommit);
      const formulas = await page.locator('.math-inline, .math-display').count();
      if (formulas > 0) {
        await expect(page.locator('html')).toHaveAttribute('data-math-failed', '0', { timeout: 120_000 });
        expect(Number(await page.locator('html').getAttribute('data-math-total'))).toBe(formulas);
        // Every formula element now holds KaTeX output, and KaTeX reported no error.
        expect(await page.locator('.math-inline:not(:has(.katex)), .math-display:not(:has(.katex))').count()).toBe(0);
        await expect(page.locator('.katex-error, .math-error')).toHaveCount(0);
      }
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
    });
  }

  for (const path of LISTS) {
    test(`${path}: list page opens`, async ({ page, problems }) => {
      void problems;
      await page.goto(`./${path}`);
      await expect(page.locator('main h1')).toBeVisible();
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
    });
  }

  test('the About page leads to the document list, which links every document', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'about');
    // Leave only after the first page view has finished loading, so no request is cut off.
    await page.waitForLoadState('networkidle');
    await page.getByRole('link', { name: 'all documents rendered on this site' }).click();
    await expect(page).toHaveURL(/\/STRATO\/docs\/index\.html$/);
    const hrefs = await page.locator('a.doc-list__title').evaluateAll((els) => els.map((e) => (e as HTMLAnchorElement).href));
    for (const path of DOCUMENTS) expect(hrefs, path).toContain(new URL(`/STRATO/${path}`, page.url()).href);
    await page.getByRole('link', { name: 'Web demonstration' }).click();
    await expect(page.locator('[data-section="explore"] h1')).toBeVisible();
    await waitForRun(page);
  });

  test('old addresses of the Jekyll site resolve and prompt/ is not published', async ({ page }) => {
    for (const path of ['docs/mathematics/v1.0.0/', 'tests/golden/v1.0/', 'docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.html']) {
      const res = await page.request.get(`./${path}`);
      expect(res.status(), path).toBe(200);
      expect(await res.text(), path).toContain('<main id="main"');
    }
    const prompt = await page.request.get('./prompt/PROGRESS.md');
    expect(await prompt.text()).not.toContain('# Progress');
  });
});

test.describe('rendered documents at phone width', () => {
  test.use({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true, deviceScaleFactor: 2 });
  test.setTimeout(180_000);

  for (const path of ['docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.html', 'data/reference/STRATO_Parameter_Register.html', 'docs/index.html']) {
    test(`${path} at 390 px: no sideways scroll`, async ({ page, problems }) => {
      void problems;
      await page.goto(`./${path}`);
      if ((await page.locator('.math-inline, .math-display').count()) > 0) {
        await expect(page.locator('html')).toHaveAttribute('data-math-failed', '0', { timeout: 120_000 });
      }
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
    });
  }
});
