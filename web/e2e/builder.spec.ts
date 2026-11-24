import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { expect, gotoSection, horizontalOverflow, solverRequests, test, waitForRun } from './fixtures';

// Web phase 2: the staged equation builder in the running build (prompt/TESTING_AND_VALIDATION.md
// Sections 3, 3.1, 3.2 and 3.3).

const catalogue = JSON.parse(readFileSync(join(process.cwd(), 'src', 'generated', 'research_catalogue.json'), 'utf8')) as {
  data: { mechanisms: { mechanism_id: string; status: string }[] };
};
const statusOf = (id: string) => catalogue.data.mechanisms.find((m) => m.mechanism_id === id)?.status;
const STAGES = [1, 2, 3, 4, 5, 6] as const;

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

test.describe('staged equation builder at 1440 x 900', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('stages 2 to 6 show the preview banner, leave the simulation unchanged and send no request', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    // Web phase 3: the solver has run stage 1 before the stages are changed; selecting a stage
    // must leave the inputs, the result and the solver request count as they are.
    await waitForRun(page);
    const before = await simulationSnapshot(page);
    const sentBefore = await solverRequests(page);
    const keyBefore = await page.getByTestId('block-b').getAttribute('data-input-key');
    // Fonts and images the new equation needs may load; nothing else (no worker, data or solver request).
    const requests: string[] = [];
    page.on('request', (r) => {
      if (!['font', 'image'].includes(r.resourceType())) requests.push(`${r.resourceType()} ${r.url()}`);
    });
    await expect(page.getByTestId('preview-banner')).toHaveCount(0);
    for (const s of [2, 3, 4, 5, 6]) {
      await pickStage(page, s);
      await expect(page.getByTestId('preview-banner')).toContainText('Equation preview only: the simulation below solves stage 1.');
      await expect(page.getByTestId('strip-stage')).toHaveText(`Equation shown: stage ${s}; simulation: stage 1`);
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

  test('KaTeX typesets every stage without an error, numbered on the right', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    for (const s of STAGES) {
      await pickStage(page, s);
      await expect(page.locator('.katex-error')).toHaveCount(0);
      const eq = page.getByTestId('equation');
      await expect(eq.locator('.katex-display').first()).toBeVisible();
      const tag = eq.locator('.katex-tag').first();
      await expect(tag).toBeVisible();
      const eqBox = await eq.boundingBox();
      const tagBox = await tag.boundingBox();
      if (!eqBox || !tagBox) throw new Error('no box');
      expect(tagBox.x + tagBox.width).toBeGreaterThan(eqBox.x + eqBox.width * 0.75);
    }
  });

  test('the switches stay disabled and the switch of the stage is marked', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    for (const [s, id] of [
      [2, 'source'],
      [3, 'decay'],
      [4, 'arrhenius'],
    ] as const) {
      await pickStage(page, s);
      const sw = page.getByTestId(`mech-${id}`);
      await expect(sw).toHaveAttribute('data-stage-current', 'true');
      await expect(sw).toHaveAttribute('aria-disabled', 'true');
      await sw.click({ force: true });
      await expect(sw).toHaveAttribute('aria-checked', 'false');
    }
  });

  test('block A shows no number apart from equation numbers, references and stage numbers', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    for (const s of STAGES) {
      await pickStage(page, s);
      const text = await page.getByTestId('block-a').evaluate((el) => {
        const c = el.cloneNode(true) as HTMLElement;
        c.querySelectorAll('.katex, [data-ref], .stage-num, .stage-select, [hidden]').forEach((n) => n.remove());
        return c.textContent ?? '';
      });
      expect(text, `stage ${s}`).not.toMatch(/\d/);
    }
  });

  test('the term card opens on hover and the term is outlined in the equation', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await pickStage(page, 3);
    await page.getByTestId('term-decay').hover();
    await expect(page.getByTestId('term-card')).toContainText('Radioactive decay');
    await expect(page.getByTestId('equation')).toHaveAttribute('data-active-term', 'decay');
    const outline = await page.locator('[data-testid="equation"] .eq-term--decay').evaluate((e) => getComputedStyle(e).outlineStyle);
    expect(outline).toBe('dashed');
  });
});

test.describe('keyboard', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('the stage picker is one tab stop moved with the arrow keys; the disclosure panels open with Enter', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    await page.getByTestId('stage-1').focus();
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('ArrowDown');
    await expect(page.getByTestId('stage-3')).toBeFocused();
    await expect(page.getByTestId('stage-3')).toHaveAttribute('aria-checked', 'true');
    // Tab leaves the radio group (one tab stop) and reaches the first term.
    await page.keyboard.press('Tab');
    await expect(page.getByTestId('term-lhs')).toBeFocused();
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
  test.describe(`new-term highlight, reduced motion ${reducedMotion}`, () => {
    test.use({ viewport: { width: 1440, height: 900 }, reducedMotion });

    test('fades in only when motion is allowed', async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      await pickStage(page, 2);
      const dur = await page.locator('[data-testid="equation"] .eq-new').first().evaluate((e) => getComputedStyle(e).animationDuration);
      if (reducedMotion === 'reduce') expect(parseFloat(dur)).toBe(0);
      else expect(parseFloat(dur)).toBeGreaterThan(0);
    });
  });
}

/** The builder stage whose typeset equation is the tallest (measured, not assumed). */
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

      test('stage 1 and the stage with the tallest equation keep A and the top of B in the first screen', async ({ page, problems }) => {
        void problems;
        await gotoSection(page, 'explore');
        await page.evaluate(() => document.fonts.ready);
        const tallest = await tallestStage(page);
        expect(tallest).toBeGreaterThanOrEqual(5);
        for (const s of [1, tallest]) {
          await pickStage(page, s);
          await page.mouse.move(0, 0);
          await page.evaluate(() => window.scrollTo(0, 0));
          const footerTop = (await page.locator('footer.status').boundingBox())?.y ?? 900;
          const headerBottom = await page.locator('header.header').evaluate((el) => el.getBoundingClientRect().bottom);
          for (const id of ['block-a', 'stage-picker', 'equation', 'particle', 'release-readout']) {
            const box = await page.getByTestId(id).boundingBox();
            expect(box, `${id} at stage ${s}`).not.toBeNull();
            if (!box) continue;
            expect(box.y, `${id} top at stage ${s}`).toBeGreaterThanOrEqual(headerBottom - 0.5);
            expect(box.y + box.height, `${id} bottom at stage ${s}`).toBeLessThanOrEqual(footerTop + 0.5);
          }
        }
      });
    });
  }
}

/** Pairs of an equation number and an equation glyph run that overlap inside block A's equation box. */
async function tagOverlaps(page: Page): Promise<string[]> {
  return page.getByTestId('equation').evaluate((box) => {
    const hits: string[] = [];
    if (box.querySelectorAll('.katex-tag').length === 0) hits.push('no katex-tag found: the selector no longer matches KaTeX');
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

    test('no equation number covers the equation at any stage', async ({ page, problems }) => {
      void problems;
      await gotoSection(page, 'explore');
      await page.evaluate(() => document.fonts.ready);
      for (const s of STAGES) {
        if (vp.width < 768) await page.getByTestId('stage-select').selectOption(String(s));
        else await pickStage(page, s);
        await expect(page.getByTestId('equation-builder')).toHaveAttribute('data-stage', String(s));
        expect(await tagOverlaps(page), `stage ${s}`).toEqual([]);
      }
    });
  });
}

test.describe('builder on a 390 px phone (device emulation)', () => {
  test.use({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true, deviceScaleFactor: 3 });

  test('stage select, equation scrolling inside its own box, no sideways page scroll', async ({ page, problems }) => {
    void problems;
    await gotoSection(page, 'explore');
    for (const s of STAGES) {
      await page.getByTestId('stage-select').selectOption(String(s));
      await expect(page.getByTestId('equation-builder')).toHaveAttribute('data-stage', String(s));
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
      const overflowX = await page.getByTestId('equation').evaluate((e) => getComputedStyle(e).overflowX);
      expect(overflowX).toBe('auto');
    }
    // Tapping a term opens its card in the flow.
    await page.getByTestId('term-decay').tap();
    await expect(page.getByTestId('term-card')).toBeVisible();
    expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
  });
});
