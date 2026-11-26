import { act, fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import caseJson from '../src/generated/synthetic_five_layer.case.json';
import { App } from '../src/App';
import { ControlPanel } from '../src/panels/ControlPanel';
import { useStrato } from '../src/state/store';

describe('ControlPanel', () => {
  it('shows one group per layer, named from the case data', () => {
    render(<ControlPanel />);
    for (const [i, r] of caseJson.data.config.regions.entries()) {
      const group = screen.getByTestId(`layer-${i}`);
      expect(within(group).getByText(r.name)).toBeTruthy();
    }
  });

  it('binds the diffusivity field to the store', async () => {
    const user = userEvent.setup();
    render(<ControlPanel />);
    const field = screen.getByTestId('d-field-0') as HTMLInputElement;
    await user.clear(field);
    await user.type(field, '3e-12');
    expect(useStrato.getState().config.regions[0]?.diffusivity_m2_s).toBe(3e-12);
    expect(screen.getByTestId('issue-count').textContent).toContain('1 input differs from the baseline');
  });

  it('marks text that is not a number as invalid and does not send it', async () => {
    const user = userEvent.setup();
    render(<ControlPanel />);
    const field = screen.getByTestId('r-field-1') as HTMLInputElement;
    await user.clear(field);
    await user.type(field, 'abc');
    expect(field.getAttribute('aria-invalid')).toBe('true');
    expect(useStrato.getState().config.regions[1]?.outer_radius_m).toBe(caseJson.data.config.regions[1]?.outer_radius_m);
  });

  it('shows the wrapper\'s own message when it rejects the inputs, and no rule of its own', async () => {
    const user = userEvent.setup();
    render(<ControlPanel />);
    const field = screen.getByTestId('r-field-2') as HTMLInputElement;
    await user.clear(field);
    await user.type(field, '1e-4');
    // The page applies no radius rule itself: the field is valid until the wrapper answers.
    expect(field.getAttribute('aria-invalid')).toBeNull();
    expect(screen.getByTestId('issue-count').textContent).toContain('from the baseline');
    // The wrapper's answer to validate for these inputs (the text is the wrapper's, shown as it is).
    act(() => {
      useStrato.getState().setValidation({ key: 'k', state: 'rejected', errors: ['invalid region IPyC'], warnings: [], plan: null, grid: null, limits: null });
    });
    expect(screen.getByTestId('validation-errors').textContent).toContain('invalid region IPyC');
    expect(screen.getByTestId('issue-count').textContent).toContain('1 message');
  });

  it('shows a warning of the wrapper beside the time controls', () => {
    render(<ControlPanel />);
    act(() => {
      useStrato.getState().setValidation({ key: 'k', state: 'ok', errors: [], warnings: ['the solver will use dt = 2.5e2 s'], plan: null, grid: null, limits: null });
    });
    expect(screen.getByTestId('validation-warning').textContent).toBe('the solver will use dt = 2.5e2 s');
  });

  it('names every control with its full physical name and its unit', () => {
    render(<ControlPanel />);
    for (const label of [
      'Kernel diffusivity (m²/s)',
      'SiC diffusivity (m²/s)',
      'Kernel outer radius (m)',
      'Grid intervals in the Kernel layer (dimensionless)',
      'Initial kernel concentration (mol/m³)',
      'Temperature (K)',
      'Time step (s)',
      'Simulation duration (s)',
      'Time steps between output rows (dimensionless)',
    ]) {
      expect(screen.getByLabelText(label), label).toBeTruthy();
    }
  });
  it('moves the diffusivity with the log slider', () => {
    render(<ControlPanel />);
    const slider = screen.getByLabelText(/Kernel diffusivity \(m²\/s\), logarithmic slider/) as HTMLInputElement;
    fireEvent.change(slider, { target: { value: '-11' } });
    expect(useStrato.getState().config.regions[0]?.diffusivity_m2_s).toBe(1e-11);
  });

  it('resets to the baseline and shows the restored value', async () => {
    const user = userEvent.setup();
    render(<ControlPanel />);
    const field = screen.getByTestId('d-field-0') as HTMLInputElement;
    await user.clear(field);
    await user.type(field, '5e-12');
    await user.click(screen.getByTestId('reset'));
    expect((screen.getByTestId('d-field-0') as HTMLInputElement).value).toBe('1e-12');
    expect(screen.getByTestId('announce').textContent).toBe('Parameters reset to the baseline.');
  });

  it('reset also clears an unsent invalid draft whose stored value did not change', async () => {
    const user = userEvent.setup();
    render(<ControlPanel />);
    const stored = useStrato.getState().config.regions[0]?.cells;
    const field = screen.getByTestId('n-field-0') as HTMLInputElement;
    // Appending a decimal part to the stored whole number leaves text the field refuses to send.
    await user.type(field, '.5');
    expect(screen.getByText('Must be a whole number.')).toBeTruthy();
    expect(useStrato.getState().config.regions[0]?.cells).toBe(stored);
    await user.click(screen.getByTestId('reset'));
    expect((screen.getByTestId('n-field-0') as HTMLInputElement).value).toBe(String(stored));
    expect(screen.queryByText('Must be a whole number.')).toBeNull();
  });

  it('holds no mechanism switch: the physics is selected in the governing equation (block A)', () => {
    render(<ControlPanel />);
    expect(document.querySelectorAll('[role="switch"]')).toHaveLength(0);
  });

  it('keeps Run unavailable before the solver exists', async () => {
    const user = userEvent.setup();
    render(<ControlPanel />);
    const run = screen.getByTestId('run');
    expect(run.getAttribute('aria-disabled')).toBe('true');
    await user.click(run);
    expect(useStrato.getState().status).toBe('idle');
    expect(useStrato.getState().run).toBeNull();
  });
});

describe('ControlPanel layouts', () => {
  it('inline: one fieldset per layer, each with the layer colour token', () => {
    render(<ControlPanel layout="inline" />);
    const root = screen.getByTestId('controls');
    expect(root.getAttribute('data-layout')).toBe('inline');
    const tokens = ['--layer-kernel', '--layer-buffer', '--layer-ipyc', '--layer-sic', '--layer-opyc'];
    for (const [i, token] of tokens.entries()) {
      const group = screen.getByTestId(`layer-${i}`);
      expect(group.tagName).toBe('FIELDSET');
      expect(group.style.getPropertyValue('--ring')).toBe(`var(${token})`);
    }
  });

  it('sheet: collapsible layer groups, a heading and a Close button', async () => {
    const user = userEvent.setup();
    let closed = 0;
    render(<ControlPanel layout="sheet" onClose={() => (closed += 1)} />);
    expect(screen.getByTestId('controls').getAttribute('data-layout')).toBe('sheet');
    expect(screen.getByTestId('layer-0').tagName).toBe('DETAILS');
    expect(screen.getByRole('heading', { name: 'Controls' })).toBeTruthy();
    await user.click(screen.getByTestId('close-sheet'));
    expect(closed).toBe(1);
  });
});

describe('Explore page (owner decision M7)', () => {
  it('renders the blocks A to E in this order, each labelled', () => {
    render(<App />);
    const blocks = Array.from(document.querySelectorAll<HTMLElement>('[data-section="explore"] [data-block]'));
    expect(blocks.map((b) => b.dataset.block)).toEqual(['A', 'B', 'C', 'D', 'E']);
    // Block B is the empty slot of the scenario row, hidden from assistive technology until it holds the row.
    for (const b of blocks.filter((x) => x.dataset.block !== 'B')) expect(b.getAttribute('aria-labelledby')).toBeTruthy();
    expect(blocks[1]?.getAttribute('aria-hidden')).toBe('true');
    expect(document.querySelector('[data-block="A"] h1')).toBeTruthy();
    expect(document.querySelectorAll('h1')).toHaveLength(1);
  });

  it('puts the particle cross-section in block D and the release readout in block E', () => {
    render(<App />);
    const d = screen.getByTestId('block-d');
    const e = screen.getByTestId('block-e');
    expect(within(d).getByTestId('particle')).toBeTruthy();
    // Without a solver reply (no worker in this test) the readout says so and carries no label.
    expect(within(e).getByTestId('release-readout').textContent).toContain('Waiting for the solver');
    expect(within(e).getByTestId('release-readout').querySelector('.badge')).toBeNull();
    expect(within(d).getByTestId('simulation-case').textContent).toContain('not AGR-1 data');
    // The scenario slot (block B) holds nothing visible.
    expect(screen.getByTestId('block-b').childElementCount).toBe(0);
    expect(screen.getByTestId('block-b').textContent).toBe('');
  });

  it('marks every later-phase area as a placeholder with its phase and no result label', () => {
    render(<App />);
    const ph = Array.from(document.querySelectorAll<HTMLElement>('[data-placeholder-phase]'));
    // Web phase 2 filled the two phase-2 placeholders (equation builder in A, terms in D); web
    // phase 3 filled the two phase-3 placeholders (inventory balance and diagnostics in B); web
    // phase 4 filled the phase-4 placeholder (the AGR-1 reference particle); web phase 5 filled the
    // two phase-5 placeholders (radial profile and release curves); web phase 5.5 filled the last
    // one, so none is left. The research catalogue summary lives under Research since 26 Nov 2026.
    expect(ph.map((p) => p.dataset.placeholderPhase)).toEqual([]);
    expect(screen.queryByTestId('future-research')).toBeNull();
    for (const p of ph) {
      expect(p.textContent).toMatch(/^Placeholder/);
      expect(p.querySelector('.badge')).toBeNull();
    }
  });

  it('draws one ring per layer with the layer colours and labels every layer on the cross-section', () => {
    render(<App />);
    const svg = screen.getByTestId('particle').querySelector('svg');
    if (!svg) throw new Error('no svg');
    const rings = Array.from(svg.querySelectorAll<SVGCircleElement>('circle.particle__ring'));
    expect(rings).toHaveLength(caseJson.data.config.regions.length);
    const labels = Array.from(svg.querySelectorAll('text.particle__text')).map((t) => t.textContent);
    expect(labels.sort()).toEqual(caseJson.data.config.regions.map((r) => r.name).sort());
  });
});

describe('App', () => {
  it('routes between the seven sections by hash', async () => {
    render(<App />);
    expect(document.querySelector('[data-section="explore"] h1')).toBeTruthy();
    for (const id of ['compare', 'validate', 'verify', 'research', 'data', 'about', 'explore']) {
      await act(async () => {
        window.location.hash = `#/${id}`;
        window.dispatchEvent(new HashChangeEvent('hashchange'));
      });
      expect(document.querySelector(`[data-section="${id}"] h1`)).toBeTruthy();
      expect(document.querySelectorAll('h1')).toHaveLength(1);
    }
  });

  it('lists the inputs that differ from the baseline on Compare', async () => {
    useStrato.getState().setRegionField(3, 'diffusivity_m2_s', 4e-14);
    window.location.hash = '#/compare';
    render(<App />);
    const table = screen.getByTestId('diff-table');
    expect(within(table).getByText('Diffusivity of the SiC layer (m²/s)')).toBeTruthy();
    expect(within(table).getByText('4e-14')).toBeTruthy();
  });

  it('shows the case provenance on Data', () => {
    window.location.hash = '#/data';
    render(<App />);
    const prov = screen.getByTestId('provenance');
    expect(within(prov).getByText(caseJson.input_sha256)).toBeTruthy();
    expect(within(prov).getByText(caseJson.input_file)).toBeTruthy();
  });

  it('opens the parameter sheet on narrow layouts and closes it with Escape', async () => {
    const user = userEvent.setup();
    render(<App />);
    const open = screen.getByTestId('open-panel');
    const sheet = screen.getByTestId('parameter-sheet');
    expect(sheet.hidden).toBe(true);
    await user.click(open);
    expect(sheet.hidden).toBe(false);
    await user.keyboard('{Escape}');
    expect(sheet.hidden).toBe(true);
    expect(document.activeElement).toBe(open);
  });
});
