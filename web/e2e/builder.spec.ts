import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { expect, gotoSection, horizontalOverflow, solverRequests, test, waitForRun } from './fixtures';

// The progressive equation builder for the BTCS solver in the running build
// (prompt/TESTING_AND_VALIDATION.md Sections 3, 3.1, 3.2 and 3.3).

const catalogue = JSON.parse(readFileSync(join(process.cwd(), 'src', 'generated', 'research_catalogue.json'), 'utf8')) as {
  data: { mechanisms: { mechanism_id: string; status: string }[] };
};
const statusOf = (id: string) => catalogue.data.mechanisms.find((m) => m.mechanism_id === id)?.status;
const STAGES = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17] as const;
const PREVIEWS: readonly number[] = [15, 16, 17];

async function pickStage(page: Page, s: number) {
  await page.getByTestId(`stage-${s}`).click();
  await expect(page.getByTestId('equation-builder')).toHaveAttribute('data-stage', String(s));
}

async function simulationSnapshot(page: Page) {
  return {
    particle: await page.getByTestId('particle').innerHTML(),
    readout: await page.getByTestId('release-readout').innerText(),
    issues: await page.getByTestId('issue-count').innerText(),
    result: await page.getByTestId('status-result').innerText(),
    fields: await page.locator('[data-testid="controls"] input').evaluateAll((els) => els.map((e) => (e as HTMLInputElement).value)),
  };
}

test.describe('progressive equation builder at 1440 x 900', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('every step leaves the simulation unchanged and sends no request; no BTCS equation is called live before the switch', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    // The solver has run before the steps are changed; selecting a step must leave the inputs, the
    // result and the solver request count as they are.
    await waitForRun(page);
    const before = await simulationSnapshot(page);
    const sentBefore = await solverRequests(page);
    const keyBefore = await page.getByTestId('block-b').getAttribute('data-input-key');
    // Fonts and images the new equation needs may load; nothing else (no worker, data or solver request).
    const requests: string[] = [];
    page.on('request', (r) => {
      if (!['font', 'image'].includes(r.resourceType())) requests.push(`${r.resourceType()} ${r.url()}`);
    });
    // The wrapper of this build still runs the previous solver: the page says so for steps one to twelve.
    await expect(page.getByTestId('live-note')).toContainText('does not run the BTCS solver yet');
    for (const s of STAGES.filter((n) => n !== 1)) {
      await pickStage(page, s);
      if (s === 14 || PREVIEWS.includes(s)) await expect(page.getByTestId('preview-banner')).toBeVisible();
      else if (s === 13) await expect(page.getByTestId('live-note')).toContainText('not experimental validation');
      else await expect(page.getByTestId('live-note')).toContainText('does not run the BTCS solver yet');
      await expect(page.getByTestId('strip-stage')).toHaveText(`Step ${s} shown; simulation: previous solver, not BTCS`);
      expect(await simulationSnapshot(page)).toEqual(before);
    }
    await page.waitForTimeout(400);
    expect(await solverRequests(page)).toBe(sentBefore);
    await expect(page.getByTestId('block-b')).toHaveAttribute('data-input-key', keyBefore ?? '');
    expect(requests).toEqual([]);
  });

  test('every research status shown equals the generated catalogue', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    for (const s of STAGES) {
      await pickStage(page, s);
      const chips = await page.locator('[data-section="explore"] [data-status-for]').evaluateAll((els) =>
        els.map((e) => ({ id: (e as HTMLElement).dataset.statusFor ?? '', status: (e as HTMLElement).dataset.status ?? '', text: e.textContent ?? '' })),
      );
      expect(chips.length).toBeGreaterThan(0);
      for (const c of chips) {
        expect(c.status, c.id).toBe(statusOf(c.id));
        expect(c.text).toBe(`Research status: ${c.status}`);
      }
    }
  });

  test('KaTeX typesets every step without an error, each equation numbered with the number of its document', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    for (const s of STAGES) {
      await pickStage(page, s);
      await expect(page.locator('.katex-error')).toHaveCount(0);
      const eq = page.getByTestId('equation');
      await expect(eq.locator('.katex-display').first()).toBeVisible();
      const tag = (await eq.getAttribute('data-tag')) ?? '';
      const numbers = tag.split(' ').filter(Boolean);
      // Every equation of a tagged step carries its number; an untagged line carries none.
      expect(await eq.locator('.katex-tag').count(), `step ${s} numbers ${tag}`).toBe(numbers.length);
      const first = eq.locator('.katex-tag').first();
      if (numbers.length > 0) await expect(first).toBeVisible();
    }
  });

  test('the switches stay disabled and the switch of the preview is marked', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    for (const [s, id] of [
      [15, 'source'],
      [16, 'decay'],
      [17, 'arrhenius'],
    ] as const) {
      await pickStage(page, s);
      const sw = page.getByTestId(`mech-${id}`);
      await expect(sw).toHaveAttribute('data-stage-current', 'true');
      await expect(sw).toHaveAttribute('aria-disabled', 'true');
      await sw.click({ force: true });
      await expect(sw).toHaveAttribute('aria-checked', 'false');
    }
  });

  test('block A shows no number apart from equation numbers, references and step numbers', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    for (const s of STAGES) {
      await pickStage(page, s);
      const text = await page.getByTestId('block-a').evaluate((el) => {
        const c = el.cloneNode(true) as HTMLElement;
        c.querySelectorAll('.katex, [data-ref], .stage-num, .stage-select, [hidden]').forEach((n) => n.remove());
        return c.textContent ?? '';
      });
      expect(text, `step ${s}`).not.toMatch(/\d/);
    }
  });

  test('the part card opens on hover and the part is outlined in the equation', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await pickStage(page, 16);
    await page.getByTestId('term-decay').hover();
    await expect(page.getByTestId('term-card')).toContainText('Radioactive decay');
    await expect(page.getByTestId('equation')).toHaveAttribute('data-active-term', 'decay');
    const outline = await page.locator('[data-testid="equation"] .eq-term--decay').evaluate((e) => getComputedStyle(e).outlineStyle);
    expect(outline).toBe('dashed');
    // A part of a BTCS step is outlined the same way.
    await pickStage(page, 6);
    await page.getByTestId('term-second_new').hover();
    const outline6 = await page.locator('[data-testid="equation"] .eq-term--second_new').evaluate((e) => getComputedStyle(e).outlineStyle);
    expect(outline6).toBe('dashed');
  });

  test('the code links of the implementation disclosure point at the solver files, and the placeholder step links none', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await pickStage(page, 11);
    await page.getByTestId('more-implementation').click();
    const links = await page.getByTestId('more-panel-implementation').locator('a').evaluateAll((els) => els.map((e) => (e as HTMLAnchorElement).href));
    expect(links.some((h) => /src\/numerical\/tridiagonal\.rs#L\d+$/.test(h))).toBe(true);
    expect(links.some((h) => /03_rust_pseudocode\.md#ps-10-/.test(h))).toBe(true);
    await pickStage(page, 14);
    await page.getByTestId('more-implementation').click();
    expect(await page.getByTestId('more-panel-implementation').locator('a').evaluateAll((els) => els.map((e) => (e as HTMLAnchorElement).href).filter((h) => h.endsWith('.rs')))).toEqual([]);
  });
});

test.describe('keyboard', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('the step picker is one tab stop moved with the arrow keys; the disclosure panels open with Enter', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.getByTestId('stage-1').focus();
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('ArrowDown');
    await expect(page.getByTestId('stage-3')).toBeFocused();
    await expect(page.getByTestId('stage-3')).toHaveAttribute('aria-checked', 'true');
    // Tab leaves the radio group (one tab stop) and reaches the first part.
    await page.keyboard.press('Tab');
    await expect(page.getByTestId('term-rate')).toBeFocused();
    await expect(page.getByTestId('term-card')).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(page.getByTestId('term-card')).toHaveCount(0);
    await page.getByTestId('more-conditions').focus();
    await page.keyboard.press('Enter');
    await expect(page.getByTestId('more-panel-conditions')).toBeVisible();
    await expect(page.getByTestId('more-conditions')).toHaveAttribute('aria-expanded', 'true');
  });
});

for (const reducedMotion of ['no-preference', 'reduce'] as const) {
  test.describe(`preview highlight, reduced motion ${reducedMotion}`, () => {
    test.use({ viewport: { width: 1440, height: 900 }, reducedMotion });

    test('fades in only when motion is allowed', async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      await pickStage(page, 16);
      const dur = await page.locator('[data-testid="equation"] .eq-new').first().evaluate((e) => getComputedStyle(e).animationDuration);
      if (reducedMotion === 'reduce') expect(parseFloat(dur)).toBe(0);
      else expect(parseFloat(dur)).toBeGreaterThan(0);
    });
  });
}

/** The builder step whose typeset equation is the tallest (measured, not assumed). */
async function tallestStage(page: Page): Promise<number> {
  let best = 1;
  let height = 0;
  for (const s of STAGES) {
    await pickStage(page, s);
    const h = (await page.getByTestId('equation').boundingBox())?.height ?? 0;
    if (h > height + 0.5) {
      height = h;
      best = s;
    }
  }
  return best;
}

for (const colorScheme of ['light', 'dark'] as const) {
  for (const reducedMotion of ['no-preference', 'reduce'] as const) {
    test.describe(`fold with the builder, ${colorScheme}, reduced motion ${reducedMotion}`, () => {
      test.use({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1, colorScheme, reducedMotion });

      test('every step keeps A and the top of B in the first screen', async ({ page, problems }) => {
        void problems;
        await gotoSection(page, 'explore');
        await page.evaluate(() => document.fonts.ready);
        const tallest = await tallestStage(page);
        expect(tallest).toBeGreaterThanOrEqual(1);
        for (const s of STAGES) {
          await pickStage(page, s);
          await page.mouse.move(0, 0);
          await page.evaluate(() => window.scrollTo(0, 0));
          const footerTop = (await page.locator('footer.status').boundingBox())?.y ?? 900;
          const headerBottom = await page.locator('header.header').evaluate((el) => el.getBoundingClientRect().bottom);
          for (const id of ['block-a', 'stage-picker', 'equation', 'particle', 'release-readout']) {
            const box = await page.getByTestId(id).boundingBox();
            expect(box, `${id} at step ${s}`).not.toBeNull();
            if (!box) continue;
            expect(box.y, `${id} top at step ${s}`).toBeGreaterThanOrEqual(headerBottom - 0.5);
            expect(box.y + box.height, `${id} bottom at step ${s}`).toBeLessThanOrEqual(footerTop + 0.5);
          }
        }
      });
    });
  }
}

/** Pairs of an equation number and an equation glyph run that overlap inside block A's equation box. */
async function tagOverlaps(page: Page, testId = 'equation'): Promise<string[]> {
  return page.getByTestId(testId).evaluate((box) => {
    const hits: string[] = [];
    const tags = box.querySelectorAll('.katex-tag').length;
    // A step whose lines are all untagged has no number to cover anything.
    if (box.hasAttribute('data-tag') && tags === 0 && (box.getAttribute('data-tag') ?? '') !== '') hits.push('no katex-tag found: the selector no longer matches KaTeX');
    for (const html of Array.from(box.querySelectorAll('.katex-display > .katex > .katex-html'))) {
      // KaTeX 0.19 names the parts katex-tag and katex-base.
      const tag = html.querySelector(':scope > .katex-tag');
      if (!tag) continue;
      const t = tag.getBoundingClientRect();
      const bases = Array.from(html.querySelectorAll(':scope > .katex-base'));
      if (bases.length === 0) hits.push('no katex-base found: the selector no longer matches KaTeX');
      for (const base of bases) {
        const b = base.getBoundingClientRect();
        const overlapX = Math.min(t.right, b.right) - Math.max(t.left, b.left);
        const overlapY = Math.min(t.bottom, b.bottom) - Math.max(t.top, b.top);
        if (overlapX > 0.5 && overlapY > 0.5) hits.push(`${tag.textContent ?? ''} over ${(base.textContent ?? '').slice(0, 24)}`);
      }
    }
    return hits;
  });
}

for (const vp of [
  { width: 390, height: 844, isMobile: true, hasTouch: true },
  { width: 768, height: 1024, isMobile: false, hasTouch: false },
  { width: 1440, height: 900, isMobile: false, hasTouch: false },
]) {
  test.describe(`equation numbers at ${vp.width} px`, () => {
    test.use({ viewport: { width: vp.width, height: vp.height }, isMobile: vp.isMobile, hasTouch: vp.hasTouch });

    test('no equation number covers the equation at any step', async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      await page.evaluate(() => document.fonts.ready);
      for (const s of STAGES) {
        if (vp.width < 768) await page.getByTestId('stage-select').selectOption(String(s));
        else await pickStage(page, s);
        await expect(page.getByTestId('equation-builder')).toHaveAttribute('data-stage', String(s));
        expect(await tagOverlaps(page), `step ${s}`).toEqual([]);
        // The equations of the open disclosures are shown with their numbers too.
        for (const panel of ['implementation', 'conditions', 'balance']) {
          const toggle = page.getByTestId(`more-${panel}`);
          if ((await toggle.count()) === 0 || !(await toggle.isEnabled())) continue;
          await toggle.click();
          await expect(page.getByTestId(`more-panel-${panel}`)).toBeVisible();
          await page.evaluate(() => document.fonts.ready);
          // The builder places the numbers after the box is resized; wait two frames for it.
          await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
          expect(await tagOverlaps(page, `more-panel-${panel}`), `step ${s} ${panel}`).toEqual([]);
          await toggle.click();
        }
      }
    });
  });
}

test.describe('builder on a 390 px phone (device emulation)', () => {
  test.use({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true, deviceScaleFactor: 3 });

  test('step select, equation scrolling inside its own box, no sideways page scroll', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    for (const s of STAGES) {
      await page.getByTestId('stage-select').selectOption(String(s));
      await expect(page.getByTestId('equation-builder')).toHaveAttribute('data-stage', String(s));
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
      const overflowX = await page.getByTestId('equation').evaluate((e) => getComputedStyle(e).overflowX);
      expect(overflowX).toBe('auto');
    }
    // Tapping a part opens its card in the flow.
    await page.getByTestId('stage-select').selectOption('16');
    await page.getByTestId('term-decay').tap();
    await expect(page.getByTestId('term-card')).toBeVisible();
    expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
  });
});
