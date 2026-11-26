import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Locator, Page } from '@playwright/test';
import { expect, gotoNumerical, horizontalOverflow, solverRequests, test, waitForRun } from './fixtures';

// The Numerical Interpretation of Explore (owner directive of 26 Nov 2026, Sections 4 and 5; decision 70(c)):
// fourteen items under the physics and scenario controls, each expanded and collapsed on its own, the state
// kept in the shared selection (and so in the address). The derivation steps of the earlier builder sit inside
// the items (prompt/TESTING_AND_VALIDATION.md Sections 3, 3.1, 3.2 and 3.3). Rewritten for that layout: the
// one-step-at-a-time picker, its select and its banner no longer exist.

const catalogue = JSON.parse(readFileSync(join(process.cwd(), 'src', 'generated', 'research_catalogue.json'), 'utf8')) as {
  data: { mechanisms: { mechanism_id: string; status: string }[] };
};
const statusOf = (id: string) => catalogue.data.mechanisms.find((m) => m.mechanism_id === id)?.status;
const ALL_ITEMS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14] as const;
const BASELINE_ITEMS = [1, 2, 3, 4, 5, 6, 7, 8, 12, 13, 14] as const;
const STEPS_IN_ITEM: Record<number, number[]> = { 1: [1, 2, 3], 2: [4], 3: [5], 4: [6], 5: [7], 6: [8], 7: [9], 8: [10], 9: [15], 10: [16], 11: [17], 12: [11], 13: [12, 14], 14: [13] };
const COMBINED = 'physics=combined';

async function simulationSnapshot(page: Page) {
  return {
    particle: await page.getByTestId('particle').innerHTML(),
    readout: await page.getByTestId('release-readout').innerText(),
    issues: await page.getByTestId('issue-count').innerText(),
    result: await page.getByTestId('status-result').innerText(),
    fields: await page.locator('[data-testid="controls"] input').evaluateAll((els) => els.map((e) => (e as HTMLInputElement).value)),
  };
}

test.describe('Numerical Interpretation at 1440 x 900', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('opens collapsed: fourteen items for the combined model, eleven for the baseline, none open, no step in the page', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page);
    await expect(page.getByTestId('num-items').locator('> li')).toHaveCount(BASELINE_ITEMS.length);
    for (const id of BASELINE_ITEMS) await expect(page.getByTestId(`num-toggle-${id}`)).toHaveAttribute('aria-expanded', 'false');
    await expect(page.getByTestId('equation-builder')).toHaveCount(0);
    await gotoNumerical(page, [], COMBINED);
    await expect(page.getByTestId('num-items').locator('> li')).toHaveCount(ALL_ITEMS.length);
  });

  test('every item opens and closes on its own, and opening one leaves the simulation, the inputs and the solver untouched', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page);
    await waitForRun(page);
    const before = await simulationSnapshot(page);
    const sentBefore = await solverRequests(page);
    const keyBefore = await page.getByTestId('block-d').getAttribute('data-input-key');
    // Fonts and images the new equation needs may load, and the browser asks again for the tab icon when the address
    // is rewritten; nothing else (no worker, data or solver request).
    const requests: string[] = [];
    page.on('request', (r) => {
      if (!['font', 'image'].includes(r.resourceType()) && !r.url().endsWith('/favicon.svg')) requests.push(`${r.resourceType()} ${r.url()}`);
    });
    // A selection that the build solves needs no status line in the interpretation (the sentence is in Scientific Documentation).
    await expect(page.getByTestId('numerical-status')).toHaveCount(0);
    for (const id of BASELINE_ITEMS) {
      await page.getByTestId(`num-toggle-${id}`).click();
      await expect(page.getByTestId(`num-toggle-${id}`)).toHaveAttribute('aria-expanded', 'true');
      const steps = await page.getByTestId(`num-item-${id}`).getByTestId('equation-builder').evaluateAll((els) => els.map((e) => Number((e as HTMLElement).dataset.stage)));
      expect(steps, `item ${id}`).toEqual(STEPS_IN_ITEM[id]);
      await expect(page.getByTestId('strip-stage')).toHaveText('Physics: Baseline diffusion; scenario: none; simulation: Method A: BTCS');
      expect(await simulationSnapshot(page)).toEqual(before);
    }
    // All eleven are open at once now; each can be closed without closing another.
    await page.getByTestId('num-toggle-5').click();
    await expect(page.getByTestId('num-toggle-5')).toHaveAttribute('aria-expanded', 'false');
    await expect(page.getByTestId('num-toggle-6')).toHaveAttribute('aria-expanded', 'true');
    await expect(page.getByTestId('num-item-5').getByTestId('equation-builder')).toHaveCount(0);
    await page.waitForTimeout(400);
    expect(await solverRequests(page)).toBe(sentBefore);
    await expect(page.getByTestId('block-d')).toHaveAttribute('data-input-key', keyBefore ?? '');
    expect(requests).toEqual([]);
  });

  test('the selected physics decides the items, labels the mechanisms with the status of this build and substitutes nothing', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page, [8]);
    await expect(page.getByTestId('num-item-9')).toHaveCount(0);
    for (const [physics, item] of [['generation', 9], ['decay', 10], ['temperature', 11]] as const) {
      await page.getByTestId(`physics-${physics}`).click();
      await expect(page.getByTestId(`num-item-${item}`)).toBeVisible();
      await expect(page.getByTestId('num-items').locator('> li')).toHaveCount(BASELINE_ITEMS.length + 1);
      // The wrapper of this build reports the baseline only: every mechanism is labelled planned, and the page says nothing runs.
      await expect(page.getByTestId(`num-status-${item}`)).toHaveText('Planned');
      await expect(page.getByTestId('numerical-status')).toContainText('No simulation runs for this selection');
      await expect(page.getByTestId('unsupported-simulation')).toBeVisible();
      await expect(page.getByTestId('particle')).toHaveCount(0);
      // The assembled system (item 8) shows what this mechanism adds, with the same status word.
      await expect(page.getByTestId('num-item-8').getByTestId('physics-additions')).toBeVisible();
    }
    await page.getByTestId('physics-combined').click();
    await expect(page.getByTestId('num-items').locator('> li')).toHaveCount(ALL_ITEMS.length);
    await expect(page.getByTestId('num-item-8').locator('[data-eq="d5_combined_balance"]')).toBeVisible();
    await expect(page.getByTestId('num-item-8').locator('[data-eq="d5_row"]')).toBeVisible();
    await page.getByTestId('physics-baseline').click();
    await expect(page.getByTestId('num-items').locator('> li')).toHaveCount(BASELINE_ITEMS.length);
    await expect(page.getByTestId('num-item-8').getByTestId('physics-additions')).toHaveCount(0);
    await waitForRun(page);
  });

  test('with a source the accounting reports F_gen and beta, not the release fraction, and item 1 says it is the baseline derivation', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page, [1, 13]);
    await expect(page.getByTestId('num-item-1').getByTestId('physics-additions')).toHaveCount(0);
    await expect(page.getByTestId('num-item-13').locator('[data-eq="d5_release_fraction_generated"]')).toHaveCount(0);
    await page.getByTestId('physics-generation').click();
    await expect(page.getByTestId('num-item-1').getByTestId('physics-additions')).toContainText('baseline derivation');
    await expect(page.getByTestId('num-item-13').locator('[data-eq="d5_release_fraction_generated"]')).toBeVisible();
    await expect(page.getByTestId('num-item-13').getByTestId('physics-additions')).toContainText('not reported');
    await page.getByTestId('physics-decay').click();
    await expect(page.getByTestId('num-item-13').locator('[data-eq="d5_release_fraction_generated"]')).toHaveCount(0);
    await page.getByTestId('physics-baseline').click();
    await waitForRun(page);
  });

  test('a scenario writes its numerical treatment inside the items where it acts, and nowhere else', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page, [9, 10, 11, 12]);
    await page.getByTestId('scenario-shutdown').click();
    await expect(page.locator('.scenario-note')).toHaveCount(3);
    await expect(page.getByTestId('num-item-9').getByTestId('scenario-note-9')).toContainText('generation is zero');
    await expect(page.getByTestId('num-item-10').getByTestId('scenario-note-10')).toContainText('diagonal');
    await expect(page.getByTestId('num-item-11').getByTestId('scenario-note-11')).toContainText('reassembled');
    await expect(page.getByTestId('scenario-note-status-11')).toHaveText('Planned');
    await page.getByTestId('scenario-steady').click();
    await expect(page.getByTestId('num-item-12').getByTestId('scenario-note-12')).toContainText('stationary field would repeat from step to step');
    await page.getByTestId('scenario-startup').click();
    await expect(page.getByTestId('scenario-note-9')).toContainText('step average of the generation');
    await expect(page.getByTestId('scenario-note-12')).toHaveCount(0);
    await page.getByTestId('scenario-control-rod').click();
    await expect(page.getByTestId('scenario-note-9')).toContainText('no term is assembled for the control rod');
    // Choosing the baseline physics clears the scenario and its notes.
    await page.getByTestId('physics-baseline').click();
    await expect(page.locator('.scenario-note')).toHaveCount(0);
  });

  test('which items are open is kept in the address and across navigation to the other sections and back', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page, [3, 8]);
    await page.getByTestId('num-toggle-2').click();
    await expect(page).toHaveURL(/#\/explore\?open=numerical,n2,n3,n8$/);
    const nav = page.getByRole('navigation', { name: 'Sections' });
    for (const name of ['Compare', 'Validate']) {
      await nav.getByRole('link', { name }).click();
      await expect(page.locator(`[data-section="${name.toLowerCase()}"] h1`)).toBeVisible();
    }
    await nav.getByRole('link', { name: 'Explore' }).click();
    await expect(page.locator('[data-section="explore"] h1')).toBeVisible();
    for (const id of [2, 3, 8]) await expect(page.getByTestId(`num-toggle-${id}`)).toHaveAttribute('aria-expanded', 'true');
    await expect(page.getByTestId('num-toggle-4')).toHaveAttribute('aria-expanded', 'false');
    await expect(page).toHaveURL(/#\/explore\?open=numerical,n2,n3,n8$/);
  });

  test('every research status shown equals the generated catalogue', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page, ALL_ITEMS, COMBINED);
    const chips = await page.locator('[data-section="explore"] [data-status-for]').evaluateAll((els) =>
      els.map((e) => ({ id: (e as HTMLElement).dataset.statusFor ?? '', status: (e as HTMLElement).dataset.status ?? '', text: e.textContent ?? '' })),
    );
    expect(chips.length).toBeGreaterThan(0);
    for (const c of chips) {
      expect(c.status, c.id).toBe(statusOf(c.id));
      expect(c.text).toBe(`Research status: ${c.status}`);
    }
  });

  test('KaTeX typesets every step without an error, each equation numbered with the number of its document, and no step or equation appears twice', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page, ALL_ITEMS, COMBINED);
    await expect(page.locator('.katex-error')).toHaveCount(0);
    const builders = page.getByTestId('equation-builder');
    await expect(builders).toHaveCount(17);
    const stages = await builders.evaluateAll((els) => els.map((e) => Number((e as HTMLElement).dataset.stage)));
    expect([...stages].sort((a, b) => a - b)).toEqual(Array.from({ length: 17 }, (_, i) => i + 1));
    for (let i = 0; i < 17; i++) {
      const eq = builders.nth(i).getByTestId('equation');
      await expect(eq.locator('.katex-display').first()).toBeAttached();
      const tag = (await eq.getAttribute('data-tag')) ?? '';
      const numbers = tag.split(' ').filter(Boolean);
      // Every equation of a tagged step carries its number; an untagged line carries none.
      expect(await eq.locator('.katex-tag').count(), `step ${stages[i]} numbers ${tag}`).toBe(numbers.length);
    }
    const ids = await page.locator('.eq-box--main [data-eq]').evaluateAll((els) => els.map((e) => (e as HTMLElement).dataset.eq));
    expect(new Set(ids).size).toBe(ids.length);
    await expect(page.getByTestId('num-items').locator('> li')).toHaveCount(14);
    // The items are in the directive's order.
    const order = await page.getByTestId('num-items').locator('> li').evaluateAll((els) => els.map((e) => Number((e as HTMLElement).dataset.item)));
    expect(order).toEqual([...ALL_ITEMS]);
  });

  test('the controls hold no second mechanism switch: the physics is chosen in the governing equation', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page);
    await expect(page.getByTestId('controls').locator('[role="switch"]')).toHaveCount(0);
    await expect(page.getByTestId('physics-picker').getByRole('radio')).toHaveCount(5);
  });

  test('the Numerical Interpretation shows no number apart from equation numbers, references and item numbers', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page, ALL_ITEMS, COMBINED);
    const text = await page.getByTestId('numerical-interpretation').evaluate((el) => {
      const c = el.cloneNode(true) as HTMLElement;
      // The numbers of the current run ("Live values") come from the wrapper; they are not text of the page.
      c.querySelectorAll('.katex, [data-ref], .stage-num, [hidden], [data-testid="live-numbers"]').forEach((n) => n.remove());
      return c.textContent ?? '';
    });
    expect(text).not.toMatch(/\d/);
  });

  test('the part card opens on hover and the part is outlined in the equation', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page, [10, 4], 'physics=decay');
    const decay = page.getByTestId('num-item-10');
    await decay.getByTestId('term-decay').hover();
    await expect(page.getByTestId('term-card')).toContainText('Decay number of a step');
    await expect(decay.getByTestId('equation')).toHaveAttribute('data-active-term', 'decay');
    const outline = await decay.locator('[data-testid="equation"] .eq-term--decay').evaluate((e) => getComputedStyle(e).outlineStyle);
    expect(outline).toBe('dashed');
    // A part of a baseline step is outlined the same way.
    await page.getByTestId('num-item-4').getByTestId('term-second_new').hover();
    const outline6 = await page.locator('[data-testid="equation"] .eq-term--second_new').evaluate((e) => getComputedStyle(e).outlineStyle);
    expect(outline6).toBe('dashed');
  });

  test('the code links of the implementation disclosure point at the solver files; a mechanism step names its development code and links none', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page, [12, 11], 'physics=temperature');
    await page.getByTestId('num-item-12').getByTestId('more-implementation').click();
    const links = await page.getByTestId('num-item-12').getByTestId('more-panel-implementation').locator('a').evaluateAll((els) => els.map((e) => (e as HTMLAnchorElement).href));
    expect(links.some((h) => /src\/numerical\/tridiagonal\.rs#L\d+$/.test(h))).toBe(true);
    expect(links.some((h) => /03_rust_pseudocode\.md#ps-10-/.test(h))).toBe(true);
    await page.getByTestId('num-item-11').getByTestId('more-implementation').click();
    const panel = page.getByTestId('num-item-11').getByTestId('more-panel-implementation');
    await expect(panel.getByTestId('dev-code')).toContainText('development branch, not in this build');
    expect(await panel.getByTestId('dev-code').locator('a').count()).toBe(0);
    const rs = await panel.locator('a').evaluateAll((els) => els.map((e) => (e as HTMLAnchorElement).href).filter((h) => h.endsWith('.rs') || /\.rs#/.test(h)));
    expect(rs).toEqual([]);
  });
});

test.describe('keyboard', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('Tab reaches each item heading, Enter and Space open and close it, focus stays on it', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page);
    await page.getByTestId('num-toggle-1').focus();
    await page.keyboard.press('Enter');
    await expect(page.getByTestId('num-toggle-1')).toHaveAttribute('aria-expanded', 'true');
    await expect(page.getByTestId('num-toggle-1')).toBeFocused();
    const ring = await page.getByTestId('num-toggle-1').evaluate((e) => ({ style: getComputedStyle(e).outlineStyle, width: parseFloat(getComputedStyle(e).outlineWidth) }));
    expect(ring.style).not.toBe('none');
    expect(ring.width).toBeGreaterThanOrEqual(2);
    await page.keyboard.press('Space');
    await expect(page.getByTestId('num-toggle-1')).toHaveAttribute('aria-expanded', 'false');
    await page.keyboard.press('Tab');
    await expect(page.getByTestId('num-toggle-2')).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(page.getByTestId('num-toggle-2')).toHaveAttribute('aria-expanded', 'true');
    // Inside an open item: Tab reaches the first part, whose card opens, and Escape closes it.
    await page.keyboard.press('Tab');
    await expect(page.getByTestId('term-materials')).toBeFocused();
    await expect(page.getByTestId('term-card')).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(page.getByTestId('term-card')).toHaveCount(0);
    await page.getByTestId('num-item-2').getByTestId('more-conditions').focus();
    await page.keyboard.press('Enter');
    await expect(page.getByTestId('num-item-2').getByTestId('more-panel-conditions')).toBeVisible();
    await expect(page.getByTestId('num-item-2').getByTestId('more-conditions')).toHaveAttribute('aria-expanded', 'true');
  });

  test('Tab from the last part of a step moves into the card of that part, and keyboard focus is never lost', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page, [3]);
    const buttons = page.getByTestId('num-item-3').locator('.eq-term-btn');
    const n = await buttons.count();
    expect(n).toBeGreaterThan(1);
    await buttons.nth(n - 1).focus();
    const where = () => page.evaluate(() => ({ tag: document.activeElement?.tagName ?? '', inCard: document.activeElement?.closest('[data-testid="term-card"]') !== null }));
    await page.keyboard.press('Tab');
    const first = await where();
    expect(first.tag).not.toBe('BODY');
    expect(first.inCard).toBe(true);
    // Through the links of the card and out of it: focus is on an element every time, never on the page body.
    for (let i = 0; i < 4; i++) {
      await page.keyboard.press('Tab');
      expect((await where()).tag, `Tab press ${i + 2}`).not.toBe('BODY');
    }
    await page.keyboard.press('Shift+Tab');
    expect((await where()).tag).not.toBe('BODY');
  });
});

for (const reducedMotion of ['no-preference', 'reduce'] as const) {
  test.describe(`highlight of a new term, reduced motion ${reducedMotion}`, () => {
    test.use({ viewport: { width: 1440, height: 900 }, reducedMotion });

    test('fades in only when motion is allowed', async ({ page, problems }) => {
      void problems;
      await gotoNumerical(page, [10], 'physics=decay');
      const dur = await page.locator('[data-testid="num-item-10"] [data-testid="equation"] .eq-new').first().evaluate((e) => getComputedStyle(e).animationDuration);
      if (reducedMotion === 'reduce') expect(parseFloat(dur)).toBe(0);
      else expect(parseFloat(dur)).toBeGreaterThan(0);
    });
  });
}

for (const colorScheme of ['light', 'dark'] as const) {
  for (const reducedMotion of ['no-preference', 'reduce'] as const) {
    test.describe(`fold with the Numerical Interpretation open, ${colorScheme}, reduced motion ${reducedMotion}`, () => {
      test.use({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1, colorScheme, reducedMotion });

      // The equation and the physics controls (block A) stay in the first screen however many items of block C
      // below them are open: block C is under them and moves nothing above it.
      test('the governing equation and the physics controls stay in the first screen with every item open', async ({ page, problems }) => {
        void problems;
        await gotoNumerical(page, ALL_ITEMS, COMBINED);
        await page.evaluate(() => document.fonts.ready);
        await page.mouse.move(0, 0);
        await page.evaluate(() => window.scrollTo(0, 0));
        const footerTop = (await page.locator('footer.status').boundingBox())?.y ?? 900;
        const headerBottom = await page.locator('header.header').evaluate((el) => el.getBoundingClientRect().bottom);
        for (const id of ['equation-governing', 'physics-picker']) {
          const box = await page.getByTestId(id).boundingBox();
          expect(box, id).not.toBeNull();
          if (!box) continue;
          expect(box.y, `${id} top`).toBeGreaterThanOrEqual(headerBottom - 0.5);
          expect(box.y + box.height, `${id} bottom`).toBeLessThanOrEqual(footerTop + 0.5);
        }
      });
    });
  }
}

/** Pairs of an equation number and an equation glyph run that overlap inside one equation box. */
async function tagOverlaps(box: Locator): Promise<string[]> {
  return box.evaluate((el) => {
    const hits: string[] = [];
    for (const html of Array.from(el.querySelectorAll('.katex-display > .katex > .katex-html'))) {
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

    test('no equation number covers the equation of any step or of an open implementation disclosure', async ({ page, problems }) => {
      void problems;
      await gotoNumerical(page, ALL_ITEMS, COMBINED);
      await page.evaluate(() => document.fonts.ready);
      const builders = page.getByTestId('equation-builder');
      const count = await builders.count();
      expect(count).toBe(17);
      for (let i = 0; i < count; i++) {
        const b = builders.nth(i);
        const stage = await b.getAttribute('data-stage');
        expect(await tagOverlaps(b.getByTestId('equation')), `step ${stage}`).toEqual([]);
        // The equations of the open disclosures are shown with their numbers too.
        const toggle = b.getByTestId('more-implementation');
        await toggle.click();
        await expect(b.getByTestId('more-panel-implementation')).toBeVisible();
        await page.evaluate(() => document.fonts.ready);
        // The page places the numbers after the box is resized; wait two frames for it.
        await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
        expect(await tagOverlaps(b.getByTestId('more-panel-implementation')), `step ${stage} implementation`).toEqual([]);
        await toggle.click();
      }
    });
  });
}

test.describe('Numerical Interpretation on a 390 px phone (device emulation)', () => {
  test.use({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true, deviceScaleFactor: 3 });

  test('44 px headings, long equations scroll inside their own box with a visible hint, no sideways page scroll', async ({ page, problems }) => {
    void problems;
    await gotoNumerical(page, ALL_ITEMS, COMBINED);
    for (const id of ALL_ITEMS) {
      const box = await page.getByTestId(`num-toggle-${id}`).boundingBox();
      expect(box?.height ?? 0, `item ${id} heading`).toBeGreaterThanOrEqual(44);
    }
    expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
    // Open one implementation disclosure with long lines (the system of item 8 and the Thomas algorithm of item 12).
    for (const id of [8, 12, 9]) await page.getByTestId(`num-item-${id}`).getByTestId('more-implementation').click();
    expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
    const builders = page.getByTestId('equation-builder');
    const n = await builders.count();
    let scrolling = 0;
    for (let i = 0; i < n; i++) {
      const eq = builders.nth(i).getByTestId('equation');
      expect(await eq.evaluate((e) => getComputedStyle(e).overflowX), `step ${await builders.nth(i).getAttribute('data-stage')}`).toBe('auto');
    }
    // Every box that holds more than it shows says so, in words a screen reader also gets, and can be reached by keyboard.
    const boxes = page.locator('[data-testid="numerical-interpretation"] .eq-box');
    const total = await boxes.count();
    for (let i = 0; i < total; i++) {
      const el = boxes.nth(i);
      if (!(await el.isVisible())) continue;
      const scrolls = await el.evaluate((e) => e.scrollWidth > e.clientWidth + 1);
      if (!scrolls) {
        expect(await el.getAttribute('aria-describedby')).toBeNull();
        continue;
      }
      scrolling++;
      await expect(el).toHaveAttribute('aria-label', /scrolls sideways/);
      await expect(el).toHaveAttribute('tabindex', '0');
      const hintId = (await el.getAttribute('aria-describedby')) ?? '';
      expect(hintId).not.toBe('');
      const hint = page.locator(`[id="${hintId}"]`);
      await expect(hint).toBeVisible();
      await expect(hint).toContainText('Scroll sideways');
    }
    // The matrix (item 8) is wider than a phone: its box scrolls and says so.
    expect(scrolling).toBeGreaterThan(0);
    await expect(page.getByTestId('num-item-8').getByTestId('equation-scroll-hint').first()).toBeVisible();
    // Tapping a part opens its card in the flow.
    await page.getByTestId('num-item-10').getByTestId('term-decay').tap();
    await expect(page.getByTestId('term-card')).toBeVisible();
    expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
  });
});
