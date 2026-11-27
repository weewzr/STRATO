// The derivation steps (the step body of equation/EquationBuilder.tsx, whose equations the six sections of the page regroup), the
// controls and the catalogue rows. The tests of the fourteen items of the old Numerical Interpretation region were rewritten for the six
// sections as tests/numerical_sections.test.tsx (decision 77 and the final Explore layout).
import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { BTCS_CAPABILITY } from '../src/equation/availability';
import { StepBody } from '../src/equation/EquationBuilder';
import { ControlPanel } from '../src/panels/ControlPanel';
import { parseCatalogue } from '../src/research/catalogue';
import { hrefFor } from '../src/route';
import { useStrato } from '../src/state/store';

const select = (patch: Parameters<ReturnType<typeof useStrato.getState>['setSelection']>[0]) => act(() => useStrato.getState().setSelection(patch));

describe('a step: equation, parts and disclosures', () => {
  it('numbers each equation with the number printed in the document and marks its parts', () => {
    render(<StepBody stage={6} physics="baseline" showTitle={false} />);
    const eq = screen.getByTestId('equation');
    expect(eq.dataset.tag).toBe('D1.12');
    expect(eq.querySelector('.eq-term--second_new')).not.toBeNull();
    expect(eq.querySelector('.eq-term--first_new')).not.toBeNull();
    expect(eq.querySelector('.katex-error')).toBeNull();
    expect(eq.textContent).toContain('(D1.12)');
  });

  it('typesets the matrix, the algorithm and the Document 5 equations of the mechanism steps', () => {
    for (const [stage, tag] of [[10, 'F19'], [11, 'F24'], [12, 'F25 F27'], [15, 'D5.3'], [16, 'D5.16'], [17, 'D5.23']] as const) {
      const { unmount } = render(<StepBody stage={stage} physics="baseline" showTitle={false} />);
      expect(screen.getByTestId('equation').dataset.tag).toBe(tag);
      expect(screen.getByTestId('equation').querySelector('.katex-error')).toBeNull();
      expect(screen.getByTestId('equation').textContent).toContain(`(${tag.split(' ').pop()})`);
      unmount();
    }
    render(<StepBody stage={10} physics="baseline" showTitle={false} />);
    expect(screen.getByTestId('equation').querySelector('.mtable')).not.toBeNull();
  });

  it('carries the mechanism colour on the part that a mechanism introduces', () => {
    render(<StepBody stage={16} physics="decay" showTitle={false} />);
    const eq = screen.getByTestId('equation');
    expect(eq.querySelectorAll('.eq-new')).toHaveLength(1);
    expect(eq.querySelector('.eq-new')?.className).toContain('eq-term--decay');
  });

  it('opens the meaning card on hover, focus or tap, outlines the part in the equation, and closes with Escape', async () => {
    const user = userEvent.setup();
    render(<StepBody stage={3} physics="baseline" showTitle={false} />);
    expect(screen.queryByTestId('term-card')).toBeNull();
    await user.hover(screen.getByTestId('term-diffusion'));
    expect(screen.getByTestId('term-card').dataset.term).toBe('diffusion');
    expect(screen.getByTestId('equation').dataset.activeTerm).toBe('diffusion');
    expect(screen.getByTestId('equation').querySelector('.eq-term--diffusion')?.classList.contains('is-active')).toBe(true);
    await user.unhover(screen.getByTestId('term-diffusion'));
    await user.click(screen.getByTestId('term-rate'));
    expect(screen.getByTestId('term-card').dataset.term).toBe('rate');
    expect(screen.getByTestId('term-rate').getAttribute('aria-expanded')).toBe('true');
    await user.keyboard('{Escape}');
    expect(screen.queryByTestId('term-card')).toBeNull();
    expect(screen.getByTestId('equation').querySelector('.is-active')).toBeNull();
  });

  it('opens one disclosure panel at a time: implementation, conditions, symbols, live values', async () => {
    const user = userEvent.setup();
    render(<StepBody stage={10} physics="baseline" showTitle={false} />);
    for (const id of ['implementation', 'conditions', 'terms', 'live']) expect((screen.getByTestId(`more-panel-${id}`) as HTMLElement).hidden).toBe(true);
    await user.click(screen.getByTestId('more-implementation'));
    expect(screen.getByTestId('more-panel-implementation').hidden).toBe(false);
    await user.click(screen.getByTestId('more-conditions'));
    expect(screen.getByTestId('more-panel-implementation').hidden).toBe(true);
    expect(screen.getByTestId('more-panel-conditions').textContent).toContain('(F2)');
    await user.click(screen.getByTestId('more-terms'));
    expect(screen.getByTestId('more-panel-terms').textContent).toMatch(/Lower diagonal/);
  });

  it('links the code of a baseline step and names, without a link, the development code of a mechanism step', async () => {
    const user = userEvent.setup();
    const { unmount } = render(<StepBody stage={11} physics="baseline" showTitle={false} />);
    await user.click(screen.getByTestId('more-implementation'));
    const impl = screen.getByTestId('more-panel-implementation');
    expect(within(impl).getByText(/ThomasFactors::solve, the back substitution/).closest('a')?.getAttribute('href')).toMatch(/src\/numerical\/tridiagonal\.rs#L230$/);
    expect(within(impl).getByText(/PS-10, tridiagonal solution/).closest('a')?.getAttribute('href')).toMatch(/03_rust_pseudocode\.md#ps-10-thomasfactors--tridiagonal-solution$/);
    expect(within(impl).queryByTestId('dev-code')).toBeNull();
    unmount();
    render(<StepBody stage={17} physics="temperature" showTitle={false} />);
    await user.click(screen.getByTestId('more-implementation'));
    const dev = screen.getByTestId('dev-code');
    expect(dev.textContent).toMatch(/development branch, not in this build/);
    expect(dev.textContent).toMatch(/run_physics/);
    expect(dev.querySelectorAll('a')).toHaveLength(0);
  });

  it('gives no live number for a selection that the build does not solve, and does not substitute the baseline run', async () => {
    const user = userEvent.setup();
    act(() => useStrato.getState().setCapabilities([BTCS_CAPABILITY]));
    select({ physics: 'generation' });
    render(<StepBody stage={6} physics="generation" showTitle={false} />);
    await user.click(screen.getByTestId('more-live'));
    expect(screen.getByTestId('live-numbers-unsupported').textContent).toMatch(/No simulation runs for the selection/);
    expect(screen.queryByTestId('live-numbers')).toBeNull();
  });

  it('keeps the verification step free of measurements; its statement that it is not validation is in Scientific Documentation', () => {
    render(<StepBody stage={13} physics="baseline" showTitle={false} />);
    expect(screen.queryByTestId('live-note')).toBeNull();
    expect(screen.getByTestId('equation').textContent).not.toMatch(/measured/i);
  });
});

// The mechanism switches of the controls were replaced by the physics selection of block A on 26 Nov 2026
// (tests/physics.test.tsx covers its status, the capability object and the simulation gate).
describe('the controls hold no second mechanism control', () => {
  it('has no mechanism switch', () => {
    act(() => useStrato.getState().setCapabilities(['diffusion_btcs_layered']));
    render(<ControlPanel layout="inline" />);
    expect(document.querySelectorAll('[role="switch"]')).toHaveLength(0);
  });
});

describe('catalogue rows with coupled equations', () => {
  // The coupled systems and disabled switches of the species-specific rows are shown on the Research page
  // (e2e/research.spec.ts); the last derivation step only links there (decision 71), so no switch is rendered in the step.
  it('the last step links to the research catalogue and renders no catalogue switch', () => {
    render(<StepBody stage={17} physics="baseline" showTitle={false} />);
    const note = screen.getByTestId('stage6');
    expect(note.querySelector('a')?.getAttribute('href')).toBe(hrefFor('research'));
    expect(document.querySelectorAll('[role="switch"]')).toHaveLength(0);
  });

  it('rejects a catalogue whose status is not one of the five words', () => {
    const raw = { commit: '', generator: '', inputs: [], tables_absent: [], data: { mechanisms: [{ mechanism_id: 'x', status: 'Verified' }], references: [], entries: {} } };
    expect(() => parseCatalogue(raw)).toThrow(/unknown status/);
  });
});
