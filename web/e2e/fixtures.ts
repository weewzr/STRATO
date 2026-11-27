import { test as base, expect, type Page } from '@playwright/test';

export const SECTIONS = ['explore', 'compare', 'validate', 'verify', 'research', 'data', 'about', 'documentation'] as const;

/** The widths of prompt/TESTING_AND_VALIDATION.md Section 3, row 1. */
export const VIEWPORTS = [
  { name: 'phone', width: 375, height: 812 },
  { name: 'tablet', width: 768, height: 1024 },
  { name: 'laptop', width: 1280, height: 800 },
  { name: 'desktop', width: 1440, height: 900 },
  { name: 'wide', width: 1920, height: 1080 },
] as const;

/** The four areas of Explore, in the order of the page (final layout of 27 Nov 2026). */
export const AREAS = ['controls', 'equation', 'numerical', 'graphs'] as const;

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

/**
 * Opens Explore with the given sections of the Numerical Interpretation expanded ('method-a', 'method-b',
 * 'discretisation', 'pseudocode', 'assumptions', 'limitations'; none by default: they start collapsed). Which
 * sections are open is part of the shared selection, so it is written to the address and the address opens them.
 */
export async function gotoNumerical(page: Page, sections: readonly string[] = [], extra = ''): Promise<void> {
  const parts = [extra, sections.length > 0 ? `open=${sections.join(',')}` : ''].filter(Boolean);
  await page.goto(`./#/explore${parts.length > 0 ? `?${parts.join('&')}` : ''}`);
  await expect(page.locator('[data-section="explore"] h1')).toBeAttached();
  await expect(page.getByTestId('numerical-interpretation')).toBeVisible();
}

/**
 * Opens the manual baseline solver (advanced/AdvancedSolver.tsx), which Scientific Documentation mounts under
 * "Advanced solver configuration" since the final layout of Explore (decision 77(f)): Explore no longer holds it.
 */
export async function gotoAdvanced(page: Page): Promise<void> {
  await page.goto('./#/documentation');
  const solver = page.getByTestId('advanced-solver');
  await expect(solver).toBeAttached({ timeout: 20_000 });
  await solver.scrollIntoViewIfNeeded();
}
/** Waits until the solver has answered for the current inputs (web phase 3); the simulation is block D of the manual solver (gotoAdvanced). */
export async function waitForRun(page: Page, status: 'ok' | 'rejected' = 'ok'): Promise<void> {
  const b = page.getByTestId('block-d');
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

/** Letters and top edges of the four areas of Explore, in document order. */
export async function areaTops(page: Page): Promise<{ area: string; top: number; left: number; bottom: number }[]> {
  return page.locator('[data-section="explore"] [data-area]').evaluateAll((els) =>
    els.map((el) => {
      const r = el.getBoundingClientRect();
      return { area: (el as HTMLElement).dataset.area ?? '', top: r.top + window.scrollY, left: r.left, bottom: r.bottom + window.scrollY };
    }),
  );
}