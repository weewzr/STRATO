// Web phase 4: colour mapping, playback state, the particle view and the AGR-1 reference view.
// The fixture is the native wrapper response written by strato-webtools precomputed (the BTCS run).
import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeEach, describe, expect, it } from 'vitest';
import { DEFAULT_CASE } from '../src/data/case';
import { readableDuration } from '../src/format';
import colormaps from '../src/generated/colormaps.json';
import reference from '../src/generated/agr1_reference.json';
import { bandOfBins, colourOf, LOG_DECADES, nodeColours, scalePosition, scaleTop, table, tableIndex } from '../src/particle/colormap';
import { ParticleView } from '../src/particle/ParticleView';
import { ReferenceParticle } from '../src/particle/ReferenceParticle';
import type { RunOk } from '../src/solver/types';
import { initialState, inputKey, useStrato } from '../src/state/store';

const RUN = (JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'wrapper_native_synthetic.json'), 'utf8')) as { data: RunOk }).data;

function withRun() {
  const s = useStrato.getState();
  act(() => {
    s.setSolver({ state: 'ready' });
    s.setRunOutcome({ run: RUN, info: { elapsedMs: 1, requestId: 1, inputKey: inputKey(s.config), dropped: [] } });
  });
}

beforeEach(() => {
  useStrato.setState({ ...initialState() });
});

describe('colour tables', () => {
  it('are the 256-entry cividis and viridis tables of the generated file, with matplotlib end points', () => {
    expect(colormaps.label).toBe('DISPLAY');
    expect(colormaps.inputs.map((i) => i.path)).toEqual(['data/colormaps/cividis.csv', 'data/colormaps/viridis.csv', 'data/colormaps/README.md']);
    for (const name of ['cividis', 'viridis'] as const) {
      expect(table(name)).toHaveLength(256);
      // rgb8 is int(255 * value) of the copied table
      colormaps.data[name].rgb.forEach((rgb, i) => expect(table(name)[i]).toEqual(rgb.map((v) => Math.trunc(255 * v))));
    }
    // first and last entries as printed in matplotlib's _cm_listed.py (cividis 0.000000, 0.135112, 0.304751)
    expect(colormaps.data.cividis.rgb[0]).toEqual([0, 0.135112, 0.304751]);
  });
});

describe('colour mapping of a known profile', () => {
  const c0 = 2;
  const profile = [2, 1, 0.5, 0, 2e-3, 2e-6, 2e-9, 3, -1, Number.NaN];

  it('linear: position c / c0, clipped, nearest table entry', () => {
    const pos = profile.map((c) => scalePosition(c, c0, 'linear'));
    expect(pos).toEqual([1, 0.5, 0.25, 0, 0.001, 0.000001, 1e-9, 1, 0, null]);
    expect(profile.slice(0, 3).map((c) => colourOf('cividis', c, c0, 'linear'))).toEqual([table('cividis')[255], table('cividis')[128], table('cividis')[64]]);
    expect(colourOf('cividis', 0, c0, 'linear')).toEqual(table('cividis')[0]);
    expect(colourOf('viridis', 3, c0, 'linear')).toEqual(table('viridis')[255]);
  });

  it(`log: ${LOG_DECADES} decades below c0`, () => {
    expect(scalePosition(2, c0, 'log')).toBe(1);
    expect(scalePosition(2e-3, c0, 'log')).toBeCloseTo(0.5, 12);
    expect(scalePosition(2e-6, c0, 'log')).toBeCloseTo(0, 12);
    expect(scalePosition(2e-9, c0, 'log')).toBe(0);
    expect(colourOf('cividis', 2e-3, c0, 'log')).toEqual(table('cividis')[tableIndex(0.5)]);
    expect(tableIndex(0.5)).toBe(128);
  });

  it('nodeColours applies the map to every node of a returned profile, on a scale topped by the returned maximum', () => {
    const row = 50;
    const colours = nodeColours(RUN, row, 'cividis', 'log');
    const values = RUN.profiles[row]!.c_mol_per_m3;
    expect(colours).toHaveLength(values.length);
    expect(colours).toHaveLength(RUN.grid.nodes);
    // the top of the scale is a number the wrapper returned (the largest of its per-row maxima), not a computed one
    expect(RUN.max_concentration_mol_m3).toContain(scaleTop(RUN));
    values.forEach((c, i) => expect(colours[i]).toEqual(colourOf('cividis', c, scaleTop(RUN), 'log')));
  });

  it('bandOfBins finds the node band that contains each radius bin, from the ring edges the wrapper returned', () => {
    expect(bandOfBins([0, 1, 3, 4], 8)).toEqual([0, 0, 1, 1, 1, 1, 2, 2]);
    const edges = RUN.grid.ring_edges_m;
    expect(edges).toHaveLength(RUN.grid.nodes + 1);
    const bins = bandOfBins(edges, 1024);
    const outer = edges[edges.length - 1]!;
    bins.forEach((band, k) => {
      const r = ((k + 0.5) / 1024) * outer;
      expect(r).toBeGreaterThanOrEqual(edges[band]!);
      expect(r).toBeLessThanOrEqual(edges[band + 1]!);
    });
  });
});

describe('playback state', () => {
  it('shows a new run at its last output row and clamps steps to the recorded rows', () => {
    withRun();
    const s = useStrato.getState();
    expect(s.playback.tIndex).toBe(RUN.profiles.length - 1);
    act(() => s.stepPlayback(1));
    expect(useStrato.getState().playback.tIndex).toBe(RUN.profiles.length - 1);
    act(() => s.setPlayback({ tIndex: -5 }));
    expect(useStrato.getState().playback.tIndex).toBe(0);
    act(() => s.setPlayback({ tIndex: 2.7, playing: true }));
    expect(useStrato.getState().playback).toMatchObject({ tIndex: 2, playing: true });
    act(() => s.stepPlayback(1));
    expect(useStrato.getState().playback).toMatchObject({ tIndex: 3, playing: false });
  });

  it('never plays without a run', () => {
    act(() => useStrato.getState().setPlayback({ playing: true }));
    expect(useStrato.getState().playback.playing).toBe(false);
  });
});

describe('particle view', () => {
  it('is the ILLUSTRATIVE schematic of the inputs before a run', () => {
    render(<ParticleView config={DEFAULT_CASE.config} provenance={DEFAULT_CASE.provenance} />);
    expect(screen.getByTestId('particle').querySelector('.badge')?.textContent).toBe('ILLUSTRATIVE');
    expect(screen.getByTestId('xsec').dataset.mode).toBe('schematic');
  });

  it('with a run is labelled COMPUTED v2.0, shows the row time and the wrapper share of the selected layer', () => {
    withRun();
    render(<ParticleView config={DEFAULT_CASE.config} provenance={DEFAULT_CASE.provenance} />);
    expect(screen.getByTestId('particle').querySelector('.badge')?.textContent).toBe('COMPUTED v2.0');
    expect(screen.getByTestId('xsec').dataset.mode).toBe('computed');
    const last = RUN.profiles.length - 1;
    expect(Number(screen.getByTestId('particle-time').dataset.value)).toBe(RUN.profiles[last]!.t_s);
    fireEvent.click(screen.getByTestId('inspect-3'));
    expect(Number(screen.getByTestId('inspector-share').dataset.value)).toBe(RUN.profiles[last]!.layer_share![3]);
    act(() => useStrato.getState().setPlayback({ tIndex: 20 }));
    expect(Number(screen.getByTestId('inspector-share').dataset.value)).toBe(RUN.profiles[20]!.layer_share![3]);
    expect(Number(screen.getByTestId('inspector-inventory').dataset.value)).toBe(RUN.profiles[20]!.layer_inventory_mol[3]);
    // one boundary per layer, in the layer colours
    expect(screen.getByTestId('xsec').querySelectorAll('[data-testid^="boundary-"]')).toHaveLength(RUN.inputs.regions.length);
  });

  it('says when WebGL 2 is missing and keeps the 2D view (jsdom has no WebGL)', () => {
    withRun();
    render(<ParticleView config={DEFAULT_CASE.config} provenance={DEFAULT_CASE.provenance} />);
    fireEvent.click(screen.getByTestId('view-3d'));
    expect(screen.getByTestId('view-note').textContent).toContain('WebGL 2');
    expect(screen.getByTestId('particle').dataset.view).toBe('2d');
  });

  it('describes the colour scale in the caption as it is set (log by default, or linear)', () => {
    withRun();
    render(<ParticleView config={DEFAULT_CASE.config} provenance={DEFAULT_CASE.provenance} />);
    const caption = () => screen.getByTestId('particle-figure').querySelector('figcaption')?.textContent ?? '';
    expect(useStrato.getState().view.logColour).toBe(true);
    expect(caption()).toContain('logarithmic colour scale over 6 decades up to the largest nodal concentration of the run');
    expect(caption()).toContain('at or below 10⁻⁶ of it take the lowest colour');
    expect(caption()).not.toContain('from 0 to');
    fireEvent.change(screen.getByTestId('colour-scale'), { target: { value: 'linear' } });
    expect(caption()).toContain('linear colour scale from 0 to the largest nodal concentration of the run');
  });
});

describe('AGR-1 reference particle (rule E2)', () => {
  it('comes from the register through the generator, with the Morris (2016) set as default', () => {
    expect(reference.label).toBe('REGISTER');
    expect(reference.input_file).toBe('data/geometry/geometry.csv');
    expect(reference.data.default_set).toBe('AGR1-B-M16');
    const m16 = reference.data.sets.find((s) => s.set_id === 'AGR1-B-M16')!;
    expect(m16.layers.map((l) => l.row_id)).toEqual(['G-01', 'G-02', 'G-03', 'G-04', 'G-05']);
    expect(m16.layers.every((l) => l.outer_radius_source === 'register')).toBe(true);
  });

  it('renders as registered geometry with row ids and status, no result badge, and switches sets', () => {
    render(<ReferenceParticle />);
    const fig = screen.getByTestId('reference-particle');
    expect(within(fig).getByTestId('reference-tag').textContent).toBe('Registered geometry');
    expect(fig.textContent).toContain('not a simulation');
    expect(fig.querySelector('.badge')).toBeNull();
    for (const l of reference.data.sets[0]!.layers) expect(fig.querySelector(`tr[data-row="${l.row_id}"]`)?.textContent).toContain(l.status);
    fireEvent.change(within(fig).getByTestId('reference-set'), { target: { value: 'AGR1-T714-V1' } });
    expect(fig.dataset.set).toBe('AGR1-T714-V1');
    expect(fig.querySelector('tr[data-row="G-20"]')).toBeTruthy();
  });

  it('ships only public https links and the cleaned source location', () => {
    for (const s of reference.data.sets) {
      expect(s.link === null || /^https:\/\//.test(s.link), s.set_id).toBe(true);
      expect(s.link_status).toBe(s.link ? 'public link in the register' : 'no verified public link in the register');
      for (const l of s.layers) {
        expect(l.link === null || /^https:\/\//.test(l.link), l.row_id).toBe(true);
        expect(l.location, l.row_id).not.toMatch(/\.pdf|idx/);
        expect('source_location' in l, l.row_id).toBe(false);
      }
    }
    // The IAEA (2012) sets have no verified public URL in the register; the view says so in words.
    render(<ReferenceParticle />);
    const fig = screen.getByTestId('reference-particle');
    fireEvent.change(within(fig).getByTestId('reference-set'), { target: { value: 'AGR1-T714-B' } });
    expect(within(fig).queryByTestId('reference-link')).toBeNull();
    expect(within(fig).getByTestId('reference-no-link').textContent).toContain('no verified public link in the register');
    for (const a of Array.from(fig.querySelectorAll('a'))) expect(a.getAttribute('href') ?? '').toMatch(/^https:\/\//);
  });
});

describe('readable duration', () => {
  it('restates seconds in a readable unit', () => {
    expect(readableDuration(0)).toBe('0 s');
    expect(readableDuration(30)).toBe('30 s');
    expect(readableDuration(600)).toBe('10.0 min');
    expect(readableDuration(7200)).toBe('2.00 h');
    expect(readableDuration(864000)).toBe('10.0 d');
  });
});
