// Coordinator decisions 29, 30, 31 and 52: the page offers exactly two methods, "Method 1: Finite
// Difference BTCS" and "Method 2: Analytical Solution"; the retired finite-volume method is not named
// as a method anywhere; the labels read COMPUTED v2.0 and "BTCS finite differences"; every control,
// table header and axis names its quantity in full with its unit. The whole page is rendered, every
// section, with a run and the precomputed files, and its text is searched.
import { act, render, screen, waitFor } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from '../src/App';
import { METHOD_1, METHOD_2, METHODS, SOLVER_WORDING } from '../src/methods';
import type { RunOk } from '../src/solver/types';
import { initialState, inputKey, useStrato } from '../src/state/store';
import { SECTIONS } from '../src/route';

const publicData = (f: string) => readFileSync(join(process.cwd(), 'public', 'data', f), 'utf8');
const RUN = (JSON.parse(publicData('wrapper_native_synthetic.json')) as { data: RunOk }).data;

beforeAll(() => {
  // The precomputed files are read from web/public/data by the fetch the page uses.
  vi.stubGlobal(
    'fetch',
    vi.fn((url: string) => {
      const name = String(url).split('/data/')[1]?.split('?')[0] ?? '';
      try {
        const body = publicData(name);
        return Promise.resolve({ ok: true, status: 200, json: () => Promise.resolve(JSON.parse(body) as unknown) });
      } catch {
        return Promise.resolve({ ok: false, status: 404, json: () => Promise.reject(new Error('not found')) });
      }
    }),
  );
});

beforeEach(() => {
  useStrato.setState({ ...initialState() });
  const s = useStrato.getState();
  act(() => {
    s.setSolver({ state: 'ready' });
    s.setCapabilities(['diffusion_btcs_layered']);
    s.setRunOutcome({ run: RUN, info: { elapsedMs: 1, requestId: 1, inputKey: inputKey(s.config), dropped: [] } });
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
});

/** Every text of the page that a reader sees: the text content, the labels and the plot titles. */
function pageText(): string {
  // Every text node on its own line, so that two neighbouring elements never run together.
  const parts: string[] = [];
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  for (let n = walker.nextNode(); n; n = walker.nextNode()) parts.push(n.textContent ?? '');
  document.querySelectorAll('[aria-label],[title]').forEach((e) => parts.push(e.getAttribute('aria-label') ?? '', e.getAttribute('title') ?? ''));
  document.querySelectorAll<HTMLElement & { data?: { name?: string }[]; layout?: Record<string, unknown> }>('[data-testid$="-plot-graph"]').forEach((g) => {
    for (const t of g.data ?? []) parts.push(t.name ?? '');
    for (const k of ['xaxis', 'yaxis', 'yaxis2']) {
      const a = g.layout?.[k] as { title?: { text?: string } } | undefined;
      parts.push(a?.title?.text ?? '');
    }
  });
  return parts.join('\n');
}

describe('the page offers exactly two methods', () => {
  it('names the two methods, and no other "Method n:" name, in every section', async () => {
    const seen = new Set<string>();
    for (const sec of SECTIONS) {
      window.location.hash = `#/${sec.id}`;
      const { unmount } = render(<App />);
      await waitFor(() => expect(document.querySelector(`[data-section="${sec.id}"] h1`)).not.toBeNull());
      await act(async () => {
        await new Promise((r) => setTimeout(r, 30));
      });
      const text = pageText();
      for (const m of text.matchAll(/Method \d+: [A-Z][A-Za-z]*(?: [A-Z][A-Za-z]*)*/g)) seen.add(m[0]);
      unmount();
    }
    expect([...seen].sort()).toEqual([...METHODS].sort());
    expect(METHODS).toEqual(['Method 1: Finite Difference BTCS', 'Method 2: Analytical Solution']);
  });

  it('shows both methods on Explore, beside the run, and none of the retired wording', async () => {
    window.location.hash = '#/explore';
    render(<App />);
    await waitFor(() => expect(screen.getByTestId('release-value')).toBeTruthy());
    const text = pageText();
    expect(text).toContain(METHOD_1);
    expect(text).toContain(METHOD_2);
    expect(text).toContain('COMPUTED v2.0');
    expect(text).not.toMatch(/finite[- ]volume/i);
    expect(text).not.toMatch(/\bCode [ABC]\b/);
    expect(text).not.toContain('COMPUTED v1.0');
    expect(SOLVER_WORDING).toBe('BTCS finite differences');
  });

  it('names no retired method and no old label in any section', async () => {
    for (const sec of SECTIONS) {
      window.location.hash = `#/${sec.id}`;
      const { unmount } = render(<App />);
      await waitFor(() => expect(document.querySelector(`[data-section="${sec.id}"] h1`)).not.toBeNull());
      await act(async () => {
        await new Promise((r) => setTimeout(r, 30));
      });
      const text = pageText();
      expect(text, sec.id).not.toMatch(/finite[- ]volume/i);
      expect(text, sec.id).not.toMatch(/\bCode [ABC]\b/);
      expect(text, sec.id).not.toContain('COMPUTED v1.0');
      expect(text, sec.id).not.toMatch(/\b(?:run|compare)_v1\b/);
      unmount();
    }
  });
});

describe('quantities are named in full with their units', () => {
  it('uses no bare symbol as the name of a control, a table header or a result on Explore and Verify', async () => {
    for (const id of ['explore', 'verify']) {
      window.location.hash = `#/${id}`;
      const { unmount } = render(<App />);
      await waitFor(() => expect(document.querySelector(`[data-section="${id}"] h1`)).not.toBeNull());
      await act(async () => {
        await new Promise((r) => setTimeout(r, 30));
      });
      // Labels of controls, headers of tables and the terms of the definition lists, outside equations.
      const named: string[] = [];
      document.querySelectorAll(`[data-section="${id}"] label, [data-section="${id}"] th, [data-section="${id}"] dt`).forEach((e) => {
        if (e.closest('.katex')) return;
        named.push((e.textContent ?? '').trim());
      });
      expect(named.length, id).toBeGreaterThan(5);
      for (const n of named) {
        if (n === '') continue;
        // A bare symbol is a name of one or two characters, possibly with an index, such as "F", "M0", "t_end".
        expect(n, `${id}: "${n}"`).not.toMatch(/^[A-Za-zΔ][A-Za-z]?(?:[_0-9₀-₉]\w*)?$/);
        expect(n, `${id}: "${n}"`).not.toMatch(/\b(?:c0|t_end|M0|D\d|R\d|F_B|F_C|Δt)\b/);
      }
      unmount();
    }
  });

  it('gives every column of the exported precomputed files a header with its unit', () => {
    for (const f of ['btcs_run.timeseries.csv', 'btcs_run.profiles.csv', 'btcs_vs_analytical.csv', 'btcs_independent_check.csv']) {
      const header = publicData(f)
        .split('\n')
        .find((l) => l !== '' && !l.startsWith('#'))!;
      for (const h of header.split(',')) {
        // A name with a unit in brackets, or a name of a class or a region (no number in it).
        expect(h, `${f}: ${h}`).toMatch(/\((?:s|mol|mol\/s|mol\/m\^3|m|dimensionless|true or false)\)$|^(?:Material region|Node class)$/);
      }
    }
  });
});
