import { act, fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import caseJson from '../src/generated/synthetic_five_layer.case.json';
import { AdvancedSolver } from '../src/advanced/AdvancedSolver';
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

describe('Explore page (final layout of 27 Nov 2026)', () => {
  it('renders the four areas in this order: model equation, operation, governing equation, numerical interpretation, graphs', () => {
    render(<App />);
    const order = ['model-equation', 'operation', 'governing-equation', 'numerical-interpretation', 'interactive-graphs'];
    const nodes = order.map((id) => screen.getByTestId(id));
    for (let i = 1; i < nodes.length; i++) {
      const before = nodes[i - 1];
      const after = nodes[i];
      expect(Boolean((before?.compareDocumentPosition(after as Node) ?? 0) & Node.DOCUMENT_POSITION_FOLLOWING), order[i]).toBe(true);
    }
    expect(document.querySelectorAll('h1')).toHaveLength(1);
    // The large live simulation, the numerical controls and the particle view are not part of Explore.
    for (const id of ['block-d', 'block-e', 'controls-block', 'particle', 'sim-strip']) expect(screen.queryByTestId(id), id).toBeNull();
  });

  it('holds the manual baseline solver in AdvancedSolver: the particle cross-section in block D and the release readout in block E', () => {
    render(<AdvancedSolver compact={false} />);
    const d = screen.getByTestId('block-d');
    const e = screen.getByTestId('block-e');
    expect(within(d).getByTestId('particle')).toBeTruthy();
    // Without a solver reply (no worker in this test) the readout says so and carries no label.
    expect(within(e).getByTestId('release-readout').textContent).toContain('Waiting for the solver');
    expect(within(e).getByTestId('release-readout').querySelector('.badge')).toBeNull();
    expect(within(d).getByTestId('simulation-case').textContent).toContain('Synthetic five-layer verification case');
  });

  it('draws one ring per layer with the layer colours and labels every layer on the cross-section (AdvancedSolver)', () => {
    render(<AdvancedSolver compact={false} />);
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

  it('shows the case provenance in Scientific Documentation and the searchable parameters on Data', () => {
    // The case file, its digest and its commit moved from Data to Scientific Documentation (documentation/data.tsx).
    window.location.hash = '#/documentation';
    const { unmount } = render(<App />);
    const prov = screen.getByTestId('provenance');
    expect(within(prov).getByText(caseJson.input_sha256)).toBeTruthy();
    expect(within(prov).getByText(caseJson.input_file)).toBeTruthy();
    unmount();
    window.location.hash = '#/data';
    render(<App />);
    expect(screen.getByTestId('provenance-search')).toBeTruthy();
    expect(screen.queryByTestId('provenance')).toBeNull();
  });

  it('opens the numerical controls from the manual solver on a narrow layout (AdvancedSolver with its entry to the controls)', async () => {
    const user = userEvent.setup();
    const opened: string[] = [];
    render(<AdvancedSolver compact onOpenControls={() => opened.push('open')} />);
    expect(screen.queryByTestId('controls')).toBeNull();
    await user.click(screen.getByTestId('open-panel'));
    expect(opened).toEqual(['open']);
  });

});
