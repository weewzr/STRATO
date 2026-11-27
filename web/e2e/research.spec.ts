import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import { expect, gotoAdvanced, gotoSection, horizontalOverflow, solverRequests, test, waitForRun } from './fixtures';

const catalogueJson = JSON.parse(readFileSync(fileURLToPath(new URL('../src/generated/research_catalogue.json', import.meta.url)), 'utf8')) as {
  data: { mechanisms: { mechanism_id: string; species: string; material_domain: string; status: string }[] };
};

// Research catalogue interface (web phase 5.5; prompt/TESTING_AND_VALIDATION.md Section 3, row 5.5):
// every status shown equals the generated catalogue, every catalogue switch is disabled and sends
// nothing, the gallery shows no image, no page scrolls sideways; review screenshots at 1440 x 900 and
// 390 px when SHOTS_DIR is set.

const dir = process.env.SHOTS_DIR ?? '';
const rows = catalogueJson.data.mechanisms;
const statusOfRow = new Map(rows.map((r) => [`${r.mechanism_id}|${r.species}|${r.material_domain}`, r.status]));
const FIVE = ['PENDING', 'RESEARCHED', 'IMPLEMENTED', 'VERIFIED', 'VALIDATED'];

const PAGES = [
  { page: '', name: 'overview' },
  { page: 'species-ag', name: 'species-ag' },
  { page: 'species-kr', name: 'species-kr' },
  { page: 'ag_trapping_detrapping', name: 'mechanism-ag-trapping' },
  { page: 'additional_species_downstream_transport', name: 'mechanism-downstream' },
  { page: 'regime-c', name: 'regime-c' },
  { page: 'gallery', name: 'gallery' },
  { page: 'roadmap', name: 'roadmap' },
  { page: 'references', name: 'references' },
];

async function gotoResearch(page: Page, sub: string) {
  await page.goto(`./#/research${sub ? `/${sub}` : ''}`);
  await expect(page.locator(`[data-section="research"][data-research-page="${sub || 'overview'}"] h1`)).toBeVisible();
  await page.evaluate(() => document.fonts.ready);
}

async function checkStatuses(page: Page, scope: string) {
  const chips = await page.locator(`${scope} .status-chip`).evaluateAll((els) =>
    els.map((e) => ({ word: e.querySelector('.status-chip__word')?.textContent ?? '', status: (e as HTMLElement).dataset.status ?? '', row: (e as HTMLElement).dataset.statusRow ?? '', shape: e.querySelector('svg') !== null })),
  );
  for (const c of chips) {
    expect(FIVE).toContain(c.word);
    expect(c.status).toBe(c.word);
    expect(c.shape).toBe(true);
    if (c.row) expect(statusOfRow.get(c.row), c.row).toBe(c.word);
  }
  return chips.length;
}

for (const vp of [
  { width: 1440, height: 900, mobile: false, label: '1440' },
  { width: 390, height: 844, mobile: true, label: '390' },
]) {
  test.describe(`research catalogue at ${vp.label}`, () => {
    test.use({ viewport: { width: vp.width, height: vp.height }, isMobile: vp.mobile, hasTouch: vp.mobile, deviceScaleFactor: vp.mobile ? 3 : 1 });

    test('every page: no status badge, switches disabled, no image, no sideways scroll', async ({ page, problems }) => {
      void problems;
      for (const p of PAGES) {
        await gotoResearch(page, p.page);
        // No status chip, badge or legend on any page of the Research Catalogue: the statuses are in Scientific Documentation.
        expect(await checkStatuses(page, '[data-section="research"]'), p.name).toBe(0);
        await expect(page.locator('[data-section="research"] .badge, [data-section="research"] [data-testid="status-legend"]')).toHaveCount(0);
        const switches = page.locator('[data-section="research"] [role="switch"]');
        for (const s of await switches.all()) await expect(s).toHaveAttribute('aria-disabled', 'true');
        await expect(page.locator('[data-section="research"] img')).toHaveCount(0);
        await expect(page.locator('.katex-error, [data-testid="tex-failed"]')).toHaveCount(0);
        expect(await horizontalOverflow(page), p.name).toBeLessThanOrEqual(0);
        if (dir) {
          await page.screenshot({ path: join(dir, `research-${p.name}-${vp.label}-first.png`), fullPage: false });
          await page.screenshot({ path: join(dir, `research-${p.name}-${vp.label}-full.png`), fullPage: true });
        }
      }
    });

    // The summary of the catalogue left the Explore page with the hierarchy of 26 Nov 2026 (block E is now the
    // results); the Research overview carries the same content: implemented and research only, the three
    // regimes, the nine species, the five statuses.
    test('the Research overview carries the regimes and the species with no status, and Explore carries no catalogue summary', async ({ page, problems }) => {
      void problems;
      await gotoResearch(page, '');
      await expect(page.getByTestId('overview-implemented')).toHaveCount(0);
      await expect(page.getByTestId('overview-builder-research')).toHaveCount(0);
      await expect(page.getByTestId('overview-species').locator('[data-species]')).toHaveCount(9);
      await expect(page.locator('[data-testid^="overview-regime-"]')).toHaveCount(3);
      await expect(page.getByTestId('status-legend')).toHaveCount(0);
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
      await gotoSection(page, 'explore');
     await expect(page.getByTestId('future-research')).toHaveCount(0);
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
    });
  });
}

test.describe('catalogue switches by keyboard', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('a pending switch takes focus, stays off under Space and Enter, and sends no solver request', async ({ page, problems }) => {
    void problems;
    // The initial run request is sent once the solver is ready: wait for its answer before counting requests.
    await gotoAdvanced(page);
    await waitForRun(page);
    await gotoResearch(page, 'ag_trapping_detrapping');
    // The module of the wrapper is larger since ABI 4 and loads later: take the count once the page's own first requests are posted.
    await expect.poll(() => solverRequests(page), { timeout: 20_000 }).toBeGreaterThan(0);
    const before = await solverRequests(page);
    const sw = page.locator('[data-section="research"] [role="switch"]').first();
    await sw.focus();
    await expect(sw).toBeFocused();
    await page.keyboard.press('Space');
    await page.keyboard.press('Enter');
    await sw.click({ force: true });
    await expect(sw).toHaveAttribute('aria-checked', 'false');
    await expect(sw).toContainText('Not available');
    expect(await solverRequests(page)).toBe(before);
  });

  test('Future species-specific physics links to the research catalogue and holds no switch', async ({ page, problems }) => {
    void problems;
    await page.goto('./#/explore?physics=future');
    const links = page.getByTestId('future-links');
    await expect(links.getByRole('link')).toHaveCount(4);
    for (const a of await links.getByRole('link').all()) await expect(a).toHaveAttribute('href', /#\/research\//);
    await expect(page.locator('[data-section="explore"] [role="switch"]')).toHaveCount(0);
    if (dir) await page.getByTestId('governing-equation').screenshot({ path: join(dir, 'explore-future-1440.png') });
  });

  test('a link to a heading of another entry opens that entry at the heading', async ({ page, problems }) => {
    void problems;
    await gotoResearch(page, 'regime-c');
    const link = page.locator('a[href="#/research/species-xe/candidate-release-mechanisms"]').first();
    await link.click();
    await expect(page.locator('[data-research-page="species-xe"] h1')).toBeVisible();
    await expect(page.locator('#r-candidate-release-mechanisms')).toBeInViewport();
  });
});
