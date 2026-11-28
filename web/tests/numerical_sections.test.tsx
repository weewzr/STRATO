// NUMERICAL INTERPRETATION of the final Explore layout: six expandable sections, collapsed by default, filled
// from the existing verified items and synchronised with the model equation and the operation; the pseudocode
// is the verbatim code of docs/mathematics/btcs/03_rust_pseudocode.md. Replaces the tests of the fourteen items
// of the earlier region (their pure parts stay in tests/numerics.test.ts).
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';
import { NumericalSections } from '../src/equation/NumericalSections';
import { PHYSICS_IDS, SCENARIO_IDS } from '../src/equation/capability';
import { scenarioNote } from '../src/equation/numerics';
import { HOOKS, PSEUDOCODE } from '../src/equation/pseudocode';
import { SECTION_IDS } from '../src/equation/selectionIds';
import { initialState, useStrato } from '../src/state/store';

beforeEach(() => {
  useStrato.setState({ ...initialState() });
});

const open = (...ids: string[]) => act(() => ids.forEach((id) => useStrato.getState().setExpanded(id, true)));
const body = (id: string) => within(screen.getByTestId(`num-section-${id}`)).getByRole('region', { hidden: true });

describe('the six sections', () => {
  it('are in the order of the directive and collapsed by default, each a button that expands its region', async () => {
    const user = userEvent.setup();
    render(<NumericalSections applicability={null} />);
    const titles = screen.getAllByRole('heading', { level: 3 }).map((h) => h.textContent);
    expect(titles).toEqual(['Method A: BTCS', 'Method B: Sturm–Liouville', 'Mathematical discretisation', 'Pseudocode', 'Assumptions', 'Limitations']);
    for (const id of SECTION_IDS) {
      const toggle = screen.getByTestId(`num-toggle-${id}`);
      expect(toggle.getAttribute('aria-expanded')).toBe('false');
      expect(body(id).hidden).toBe(true);
      expect(body(id).textContent).toBe('');
    }
    await user.click(screen.getByTestId('num-toggle-discretisation'));
    expect(screen.getByTestId('num-toggle-discretisation').getAttribute('aria-expanded')).toBe('true');
    expect(useStrato.getState().selection.expanded).toEqual(['discretisation']);
    expect(body('discretisation').hidden).toBe(false);
    await user.click(screen.getByTestId('num-toggle-discretisation'));
    expect(useStrato.getState().selection.expanded).toEqual([]);
  });

  it('are operated by the keyboard: Tab reaches each heading, Enter and Space toggle it, focus stays on it', async () => {
    const user = userEvent.setup();
    render(<NumericalSections applicability={null} />);
    await user.tab();
    expect(document.activeElement).toBe(screen.getByTestId('num-toggle-method-a'));
    await user.keyboard('{Enter}');
    expect(useStrato.getState().selection.expanded).toEqual(['method-a']);
    await user.keyboard(' ');
    expect(useStrato.getState().selection.expanded).toEqual([]);
    expect(document.activeElement).toBe(screen.getByTestId('num-toggle-method-a'));
    await user.tab();
    expect(document.activeElement).toBe(screen.getByTestId('num-toggle-method-b'));
  });

  it('keeps what is open across a change of the model equation or the operation', () => {
    render(<NumericalSections applicability={null} />);
    open('discretisation', 'pseudocode');
    act(() => useStrato.getState().setSelection({ physics: 'combined', scenario: 'shutdown' }));
    expect(useStrato.getState().selection.expanded).toEqual(['discretisation', 'pseudocode']);
  });
});

describe('what the model equation shows', () => {
  const equationsOf = (id: string) => Array.from(body(id).querySelectorAll<HTMLElement>('.nstep__eq')).map((e) => e.dataset.eq);

  it('adds the source in the right-hand side with generation, the diagonal with decay, the coefficients with temperature, and all three for the combined model', () => {
    render(<NumericalSections applicability={null} />);
    open('discretisation');
    const stages = () => Array.from(body('discretisation').querySelectorAll<HTMLElement>('.nstep')).map((s) => Number(s.dataset.stage));
    const BASE = [5, 6, 7, 8, 9, 10];
    act(() => useStrato.getState().setSelection({ physics: 'baseline' }));
    expect(stages()).toEqual(BASE);
    act(() => useStrato.getState().setSelection({ physics: 'generation' }));
    expect(stages()).toEqual([...BASE, 15]);
    act(() => useStrato.getState().setSelection({ physics: 'decay' }));
    expect(stages()).toEqual([...BASE, 16]);
    act(() => useStrato.getState().setSelection({ physics: 'temperature' }));
    expect(stages()).toEqual([...BASE, 17]);
    act(() => useStrato.getState().setSelection({ physics: 'combined' }));
    expect(stages()).toEqual([...BASE, 15, 16, 17]);
    act(() => useStrato.getState().setSelection({ physics: 'future' }));
    expect(stages()).toEqual([...BASE, 15, 16, 17]);
    expect(equationsOf('discretisation').length).toBeGreaterThan(8);
  });

  it('places every equation number in its own non-overlapping .nstep__tag column outside .nstep__math instead of inline KaTeX .katex-tag', () => {
    render(<NumericalSections applicability={null} />);
    open('method-a', 'discretisation', 'assumptions');
    act(() => useStrato.getState().setSelection({ physics: 'combined' }));
    const tagged = Array.from(document.querySelectorAll<HTMLElement>('.nstep__eq[data-has-tag="true"]'));
    expect(tagged.length).toBeGreaterThan(5);
    for (const eq of tagged) {
      expect(eq.querySelector('.nstep__math')).not.toBeNull();
      expect(eq.querySelector('.nstep__tag')?.textContent).toMatch(/^\(.+\)$/);
      expect(eq.querySelector('.katex-tag')).toBeNull();
    }
  });

  it('writes the operation as a note inside the item where it acts, only where its mechanism is part of the model equation', () => {
    render(<NumericalSections applicability={null} />);
    open('discretisation', 'method-a');
    const notes = () => Array.from(document.querySelectorAll<HTMLElement>('[data-operation-note]')).map((n) => Number(n.dataset.operationNote));
    act(() => useStrato.getState().setSelection({ physics: 'baseline', scenario: 'shutdown' }));
    expect(notes()).toEqual([]);
    act(() => useStrato.getState().setSelection({ physics: 'combined', scenario: 'shutdown' }));
    expect(notes()).toEqual([9, 10, 11]);
    act(() => useStrato.getState().setSelection({ physics: 'decay', scenario: 'shutdown' }));
    expect(notes()).toEqual([10]);
    act(() => useStrato.getState().setSelection({ physics: 'baseline', scenario: 'steady' }));
    expect(notes()).toEqual([12]);
    // The text of a note is the existing one, moved.
    act(() => useStrato.getState().setSelection({ physics: 'temperature', scenario: 'shutdown' }));
    expect(document.querySelector('[data-operation-note="11"]')?.textContent).toContain('reassembled');
    expect(scenarioNote('shutdown', 11)).toMatch(/temperature of the middle of the step/);
  });

  it('puts the equations of the step of the Rust solution algorithm and of the accounting under Method A, and none under Method B', () => {
    render(<NumericalSections applicability={null} />);
    open('method-a', 'method-b');
    expect(Array.from(body('method-a').querySelectorAll<HTMLElement>('.nstep')).map((s) => Number(s.dataset.stage))).toEqual([11, 12]);
    expect(body('method-b').querySelectorAll('.nstep__eq')).toHaveLength(0);
  });

  it('says where Method B applies from the wrapper answer, and nothing where none was asked', () => {
    const { rerender } = render(<NumericalSections applicability={null} />);
    open('method-b');
    expect(screen.queryByTestId('method-b-applicability')).toBeNull();
    rerender(<NumericalSections applicability={{ applicable: true, extension: 'decay at a constant temperature', reason: null }} />);
    expect(screen.getByTestId('method-b-applicability').textContent).toBe('Applies here: decay at a constant temperature.');
    rerender(<NumericalSections applicability={{ applicable: false, extension: null, reason: 'the temperature changes in time' }} />);
    expect(screen.getByTestId('method-b-applicability').textContent).toBe('Does not apply here: the temperature changes in time');
  });
});

describe('the pseudocode', () => {
  it('shows the verbatim routines of the document for the baseline and the hook of each selected mechanism', () => {
    render(<NumericalSections applicability={null} />);
    open('pseudocode');
    const blocks = () => Array.from(body('pseudocode').querySelectorAll<HTMLElement>('[data-block]')).map((b) => b.dataset.block);
    expect(blocks()).toEqual(PSEUDOCODE.map((b) => b.id));
    for (const b of PSEUDOCODE) expect(body('pseudocode').querySelector(`[data-block="${b.id}"] code`)?.textContent).toBe(b.code);
    expect(body('pseudocode').querySelectorAll('[data-hook]')).toHaveLength(0);
    act(() => useStrato.getState().setSelection({ physics: 'generation' }));
    expect(Array.from(body('pseudocode').querySelectorAll<HTMLElement>('[data-hook]')).map((h) => h.dataset.hook)).toEqual(['generation']);
    expect(body('pseudocode').querySelector('[data-hook="generation"] code')?.textContent).toContain(HOOKS.G1.change);
    act(() => useStrato.getState().setSelection({ physics: 'combined' }));
    expect(Array.from(body('pseudocode').querySelectorAll<HTMLElement>('[data-hook]')).map((h) => h.dataset.hook)).toEqual(['generation', 'decay', 'temperature']);
    // Temperature: the diffusivity is taken at the mid-step temperature, as Document 5 states.
    expect(body('pseudocode').querySelector('[data-hook="temperature"]')?.textContent).toMatch(/temperature of the middle of the step/);
    expect(body('pseudocode').textContent).not.toMatch(/new time level/);
  });

  it('equals the code blocks of docs/mathematics/btcs/03_rust_pseudocode.md character for character', () => {
    const doc = readFileSync(join(process.cwd(), '..', 'docs', 'mathematics', 'btcs', '03_rust_pseudocode.md'), 'utf8').replace(/\r\n/g, '\n');
    const lines = doc.split('\n');
    for (const b of PSEUDOCODE) {
      const h = lines.findIndex((l) => l.startsWith(`### ${b.id} `));
      expect(h, b.id).toBeGreaterThan(-1);
      expect(lines[h]).toBe(`### ${b.title}`);
      let s = h;
      while (!lines[s]?.startsWith('```rust')) s++;
      let e = s + 1;
      while (!lines[e]?.startsWith('```')) e++;
      expect(b.code, b.id).toBe(lines.slice(s + 1, e).join('\n'));
    }
    for (const [g, hook] of Object.entries(HOOKS)) {
      const row = lines.find((l) => l.startsWith(`| ${g} `));
      expect(row, g).toBe(`| ${hook.mechanism} | ${hook.entersAt} | ${hook.change} | ${row?.split('|')[4]?.trim() ?? ''} |`);
    }
  });
});

describe('assumptions and limitations', () => {
  it('hold the existing texts: the continuous model with its conditions, and the limits of the model', () => {
    render(<NumericalSections applicability={null} />);
    open('assumptions', 'limitations');
    expect(Array.from(body('assumptions').querySelectorAll<HTMLElement>('.nstep[data-stage]')).map((s) => Number(s.dataset.stage))).toEqual([1, 2, 3, 4]);
    expect(body('limitations').textContent).toMatch(/prescribed input history/);
    expect(body('limitations').textContent).toMatch(/neutron kinetics/);
    expect(body('limitations').querySelectorAll('li').length).toBeGreaterThanOrEqual(4);
  });
});

describe('the whole region', () => {
  it('carries no status word with every section open, for every model equation and every operation', { timeout: 120_000 }, () => {
    render(<NumericalSections applicability={null} />);
    open(...SECTION_IDS);
    for (const physics of PHYSICS_IDS) {
      for (const scenario of SCENARIO_IDS) {
        act(() => useStrato.getState().setSelection({ physics, scenario }));
        const copy = document.body.cloneNode(true) as HTMLElement;
        copy.querySelectorAll('.katex, .npseudo__code').forEach((n) => n.remove());
        const words = (copy.textContent ?? '').match(/(.{0,40})\b(planned|verified|computed|implemented|precomputed|validated|pending|researched)\b(.{0,20})/gi);
        expect(words ?? [], `${physics}/${scenario}`).toEqual([]);
      }
    }
  });
});
