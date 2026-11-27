import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import { expect, gotoSection, horizontalOverflow, solverRequests, test } from './fixtures';

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

    test('every page: statuses equal the catalogue, switches disabled, no image, no sideways scroll', async ({ page, problems }) => {
      void problems;
      for (const p of PAGES) {
        await gotoResearch(page, p.page);
        const n = await checkStatuses(page, '[data-section="research"]');
        if (p.page !== 'references') expect(n, p.name).toBeGreaterThan(0);
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

    test('block E summarises the catalogue under the live simulation', async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      const e = page.getByTestId('block-e');
      await e.scrollIntoViewIfNeeded();
      await expect(e.getByTestId('future-implemented')).toBeVisible();
      await expect(e.getByTestId('future-pending')).toBeVisible();
      await expect(e.getByTestId('future-species').locator('[data-species]')).toHaveCount(9);
      await expect(e.getByTestId('future-regimes').locator('> section')).toHaveCount(3);
      // The definitions moved to Scientific Documentation: the legend is the five statuses in one row.
      await expect(e.getByTestId('status-legend').locator('.status-chip')).toHaveCount(5);
      expect(await checkStatuses(page, '[data-testid="block-e"]')).toBeGreaterThan(0);
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
      if (dir) {
        await page.evaluate(() => document.fonts.ready);
        await e.screenshot({ path: join(dir, `explore-block-e-${vp.label}.png`) });
      }
    });
  });
}

test.describe('catalogue switches by keyboard', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('a pending switch takes focus, stays off under Space and Enter, and sends no solver request', async ({ page, problems }) => {
    void problems;
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
    await expect(sw).toContainText('Not available: status PENDING');
    expect(await solverRequests(page)).toBe(before);
  });

  test('the last step of the builder lists the catalogue rows behind one disclosure, with disabled switches', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.getByTestId('stage-17').click();
    const toggle = page.getByTestId('stage6-toggle');
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    const before = await solverRequests(page);
    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    const switches = page.getByTestId('stage6').locator('[role="switch"]');
    expect(await switches.count()).toBeGreaterThan(0);
    for (const s of await switches.all()) await expect(s).toHaveAttribute('aria-disabled', 'true');
    await switches.first().click({ force: true });
    await expect(switches.first()).toHaveAttribute('aria-checked', 'false');
    expect(await checkStatuses(page, '[data-testid="stage6"]')).toBeGreaterThan(0);
    expect(await solverRequests(page)).toBe(before);
    if (dir) await page.getByTestId('block-a').screenshot({ path: join(dir, 'explore-last-step-1440.png') });
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
