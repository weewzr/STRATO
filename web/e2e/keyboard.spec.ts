import type { Page } from '@playwright/test';
import { expect, gotoSection, test } from './fixtures';

async function focusInfo(page: Page) {
  return page.evaluate(() => {
    const el = document.activeElement as HTMLElement | null;
    if (!el || el === document.body) return null;
    const cs = getComputedStyle(el);
    return {
      key: el.getAttribute('data-testid') ?? el.id ?? '',
      tag: el.tagName.toLowerCase(),
      text: (el.getAttribute('aria-label') ?? el.textContent ?? '').trim().slice(0, 60),
      outlineStyle: cs.outlineStyle,
      outlineWidth: parseFloat(cs.outlineWidth),
      inControls: el.closest('[data-testid="block-c"] [data-testid="controls"]') !== null,
    };
  });
}

const CONTROLS = '[data-testid="block-c"] [data-testid="controls"]';

test.describe('keyboard at 1280 px', () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test('the skip link comes first and moves focus to the main content', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.keyboard.press('Tab');
    await expect(page.getByRole('link', { name: 'Skip to content' })).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(page.locator('main#main')).toBeFocused();
    await expect(page).toHaveURL(/#\/explore$/);
  });

  test('every control of block C is reached by Tab and shows a visible focus ring', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const expected = await page.evaluate(
      (sel) =>
        Array.from(document.querySelectorAll<HTMLElement>(`${sel} :is(input, select, button, summary, a[href])`)).filter(
          (el) => el.offsetParent !== null,
        ).length,
      CONTROLS,
    );
    expect(expected).toBeGreaterThan(30);
    await page.locator(`${CONTROLS} select`).focus();
    const seen = new Set<string>();
    let reached = 0;
    for (let i = 0; i < 200 && reached < expected; i++) {
      const f = await focusInfo(page);
      if (!f?.inControls) break;
      const id = `${f.tag}:${f.key}:${f.text}:${i}`;
      if (!seen.has(id)) {
        seen.add(id);
        reached++;
        expect(f.outlineStyle, `focus ring on ${f.tag} ${f.text}`).toBe('solid');
        expect(f.outlineWidth).toBeGreaterThanOrEqual(2);
      }
      await page.keyboard.press('Tab');
    }
    expect(reached).toBe(expected);
  });

  test('the diffusivity slider and fields work from the keyboard', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const slider = page.getByLabel('Kernel diffusivity (m²/s), logarithmic slider');
    await slider.focus();
    await page.keyboard.press('End');
    await expect(page.getByTestId('d-field-0')).toHaveValue('1e-8');
    await page.getByTestId('reset').focus();
    await page.keyboard.press('Enter');
    await expect(page.getByTestId('d-field-0')).toHaveValue('1e-12');

    const field = page.getByTestId('d-field-0');
    await field.focus();
    await page.keyboard.press('ControlOrMeta+a');
    await page.keyboard.type('2.5e-12');
    await expect(page.getByTestId('issue-count')).toHaveText('1 input differs from the baseline.');
    await page.getByTestId('set-baseline').focus();
    await page.keyboard.press('Space');
    await expect(page.getByTestId('issue-count')).toHaveText('Inputs equal the baseline.');
  });

  test('mechanism switches are focusable, stay off and say why', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    for (const id of ['source', 'decay', 'arrhenius']) {
      const sw = page.getByTestId(`mech-${id}`);
      await sw.focus();
      await expect(sw).toBeFocused();
      await page.keyboard.press('Space');
      await page.keyboard.press('Enter');
      await expect(sw).toHaveAttribute('aria-checked', 'false');
      await expect(sw).toHaveAttribute('aria-disabled', 'true');
      await expect(page.getByTestId('announce')).toContainText('not available');
    }
  });

  test('section links work with Enter and focus the new heading', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.getByRole('navigation', { name: 'Sections' }).getByRole('link', { name: 'Data' }).focus();
    await page.keyboard.press('Enter');
    await expect(page).toHaveURL(/#\/data$/);
    await expect(page.locator('[data-section="data"] h1')).toBeFocused();
  });

  test('the layer inspector works from the keyboard', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.getByTestId('inspect-3').focus();
    await page.keyboard.press('Enter');
    await expect(page.getByTestId('inspect-3')).toHaveAttribute('aria-pressed', 'true');
    await expect(page.getByTestId('inspector-detail')).toContainText('SiC');
  });
});

test.describe('keyboard at 375 px', () => {
  test.use({ viewport: { width: 375, height: 812 } });

  test('menu opens with Enter and closes with Escape, returning focus', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    const menu = page.getByRole('button', { name: 'Menu' });
    await menu.focus();
    await page.keyboard.press('Enter');
    await expect(page.getByRole('navigation', { name: 'Sections' })).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('navigation', { name: 'Sections' })).toBeHidden();
    await expect(menu).toBeFocused();
  });

  for (const opener of ['open-panel', 'strip-open-controls']) {
    test(`the controls sheet opens from ${opener} and closes with Escape`, async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      const open = page.getByTestId(opener);
      await open.focus();
      await page.keyboard.press('Enter');
      const sheet = page.getByTestId('parameter-sheet');
      await expect(sheet).toBeVisible();
      await expect(sheet.locator(':focus')).toHaveCount(1);
      await page.keyboard.press('Escape');
      await expect(sheet).toBeHidden();
      await expect(open).toBeFocused();
    });
  }
});
