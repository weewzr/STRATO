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

  it('shows the Config::validate rule when radii stop increasing', async () => {
    const user = userEvent.setup();
    render(<ControlPanel />);
    const field = screen.getByTestId('r-field-2') as HTMLInputElement;
    await user.clear(field);
    await user.type(field, '1e-4');
    expect(field.getAttribute('aria-invalid')).toBe('true');
    expect(screen.getByText(/must be larger than the radius of the layer inside it/)).toBeTruthy();
    expect(screen.getByTestId('issue-count').textContent).toContain('1 input needs attention');
  });

  it('moves the diffusivity with the log slider', () => {
    render(<ControlPanel />);
    const slider = screen.getByLabelText(/D1, logarithmic slider/) as HTMLInputElement;
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
    expect(field.value).toBe('1e-12');
    expect(screen.getByTestId('announce').textContent).toBe('Parameters reset to the baseline.');
  });

  it('keeps the mechanism switches focusable, disabled and off', async () => {
    const user = userEvent.setup();
    render(<ControlPanel />);
    for (const id of ['source', 'decay', 'arrhenius']) {
      const sw = screen.getByTestId(`mech-${id}`);
      expect(sw.getAttribute('role')).toBe('switch');
      expect(sw.getAttribute('aria-disabled')).toBe('true');
      expect(sw.getAttribute('aria-checked')).toBe('false');
      expect(sw.hasAttribute('disabled')).toBe(false);
      await user.click(sw);
      expect(sw.getAttribute('aria-checked')).toBe('false');
    }
    expect(screen.getByText(/Not available: awaiting reviewed implementation/)).toBeTruthy();
    expect(useStrato.getState().mechanisms).toEqual({
      decay: { enabled: false, available: false },
      source: { enabled: false, available: false },
      arrhenius: { enabled: false, available: false },
    });
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
    for (const b of blocks) expect(b.getAttribute('aria-labelledby')).toBeTruthy();
    expect(document.querySelector('[data-block="A"] h1')).toBeTruthy();
    expect(document.querySelectorAll('h1')).toHaveLength(1);
  });

  it('puts the particle cross-section and the release readout in block B', () => {
    render(<App />);
    const b = screen.getByTestId('block-b');
    expect(within(b).getByTestId('particle')).toBeTruthy();
    expect(within(b).getByTestId('release-readout').textContent).toContain('No result');
    expect(within(b).getByTestId('simulation-case').textContent).toContain('not AGR-1 data');
  });

  it('marks every later-phase area as a placeholder with its phase and no result label', () => {
    render(<App />);
    const ph = Array.from(document.querySelectorAll<HTMLElement>('[data-placeholder-phase]'));
    expect(ph.map((p) => p.dataset.placeholderPhase)).toEqual(['2', '5', '5', '3', '3', '4', '2', '5.5']);
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
    expect(within(table).getByText('SiC diffusivity')).toBeTruthy();
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
