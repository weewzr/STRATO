import { test as base, expect, type Page } from '@playwright/test';

export const SECTIONS = ['explore', 'compare', 'validate', 'verify', 'research', 'data', 'about'] as const;

/** The widths of prompt/TESTING_AND_VALIDATION.md Section 3, row 1. */
export const VIEWPORTS = [
  { name: 'phone', width: 375, height: 812 },
  { name: 'tablet', width: 768, height: 1024 },
  { name: 'laptop', width: 1280, height: 800 },
  { name: 'desktop', width: 1440, height: 900 },
  { name: 'wide', width: 1920, height: 1080 },
] as const;

export const BLOCKS = ['A', 'B', 'C', 'D', 'E'] as const;

/** Every page in these tests records console errors, page errors and failed requests; each test ends by asserting none. */
export const test = base.extend<{ problems: string[] }>({
  problems: async ({ page }, provide) => {
    const problems: string[] = [];
    page.on('console', (m) => {
      if (m.type() === 'error') problems.push(`console: ${m.text()}`);
    });
    page.on('pageerror', (e) => problems.push(`pageerror: ${e.message}`));
    page.on('requestfailed', (r) => problems.push(`requestfailed: ${r.url()} ${r.failure()?.errorText ?? ''}`));
    page.on('response', (r) => {
      if (r.status() >= 400) problems.push(`HTTP ${r.status()}: ${r.url()}`);
    });
    await provide(problems);
    expect(problems, 'console errors or failed requests').toEqual([]);
  },
});

export { expect };

export async function gotoSection(page: Page, id: string): Promise<void> {
  await page.goto(`./#/${id}`);
  await expect(page.locator(`[data-section="${id}"] h1`)).toBeVisible();
}

/** Waits until the solver has answered for the current inputs (web phase 3). */
export async function waitForRun(page: Page, status: 'ok' | 'rejected' = 'ok'): Promise<void> {
  const b = page.getByTestId('block-b');
  await expect(b).toHaveAttribute('data-run-status', status, { timeout: 20_000 });
  const key = await b.getAttribute('data-input-key');
  if (status === 'ok') await expect(b).toHaveAttribute('data-result-key', key ?? '');
}

/** Requests posted to the solver worker so far (stamped by solver/session.ts). */
export async function solverRequests(page: Page): Promise<number> {
  return Number((await page.locator('html').getAttribute('data-solver-requests')) ?? 'NaN');
}

export async function horizontalOverflow(page: Page): Promise<number> {
  return page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
}

/** Letters and top edges of the Explore blocks, in document order. */
export async function blockTops(page: Page): Promise<{ letter: string; top: number }[]> {
  return page.locator('[data-section="explore"] [data-block]').evaluateAll((els) =>
    els.map((el) => ({ letter: (el as HTMLElement).dataset.block ?? '', top: el.getBoundingClientRect().top + window.scrollY })),
  );
}
