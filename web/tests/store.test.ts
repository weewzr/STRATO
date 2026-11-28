import { describe, expect, it } from 'vitest';
import caseJson from '../src/generated/synthetic_five_layer.case.json';
import { DEFAULT_CASE } from '../src/data/case';
import { configDifferences, useStrato } from '../src/state/store';

describe('store defaults', () => {
  it('start from the generated case JSON, not from typed values', () => {
    const s = useStrato.getState();
    expect(s.config).toEqual(caseJson.data.config);
    expect(s.baselineConfig).toEqual(caseJson.data.config);
    expect(s.caseId).toBe('synthetic_five_layer');
  });

  it('hold copies, so editing the store never changes the case data', () => {
    useStrato.getState().setRegionField(3, 'diffusivity_m2_s', 5e-14);
    expect(DEFAULT_CASE.config.regions[3]?.diffusivity_m2_s).toBe(caseJson.data.config.regions[3]?.diffusivity_m2_s);
    expect(useStrato.getState().baselineConfig.regions[3]?.diffusivity_m2_s).toBe(caseJson.data.config.regions[3]?.diffusivity_m2_s);
  });

  it('start with no solver capabilities (nothing is live before the wasm module loads)', () => {
    const s = useStrato.getState();
    expect(s.capabilities).toEqual([]);
  });

  it('have no run result and an idle status before the solver exists', () => {
    const s = useStrato.getState();
    expect(s.run).toBeNull();
    expect(s.baselineRun).toBeNull();
    expect(s.status).toBe('idle');
  });
});

describe('store actions', () => {
  it('set region, time, start and temperature fields', () => {
    const a = useStrato.getState();
    a.setRegionField(0, 'outer_radius_m', 2.5e-4);
    a.setRegionField(1, 'cells', 12);
    a.setTimeField('dt_s', 100);
    a.setTimeField('output_every_steps', 5);
    a.setKernelConcentration(2);
    a.setTemperature(1200);
    const c = useStrato.getState().config;
    expect(c.regions[0]?.outer_radius_m).toBe(2.5e-4);
    expect(c.regions[1]?.cells).toBe(12);
    expect(c.time.dt_s).toBe(100);
    expect(c.time.output_every_steps).toBe(5);
    expect(c.initial.kernel_concentration_mol_m3).toBe(2);
    expect(c.temperature_k).toBe(1200);
  });

  it('reset to the baseline', () => {
    const a = useStrato.getState();
    a.setRegionField(2, 'diffusivity_m2_s', 1e-11);
    a.resetToBaseline();
    expect(useStrato.getState().config).toEqual(caseJson.data.config);
  });

  it('set the current parameters as the baseline', () => {
    const a = useStrato.getState();
    a.setRegionField(2, 'diffusivity_m2_s', 1e-11);
    a.setCurrentAsBaseline();
    expect(useStrato.getState().baselineConfig.regions[2]?.diffusivity_m2_s).toBe(1e-11);
    a.setRegionField(2, 'diffusivity_m2_s', 3e-11);
    a.resetToBaseline();
    expect(useStrato.getState().config.regions[2]?.diffusivity_m2_s).toBe(1e-11);
  });

  it('reload a case, which also resets the baseline', () => {
    const a = useStrato.getState();
    a.setRegionField(2, 'diffusivity_m2_s', 1e-11);
    a.setCurrentAsBaseline();
    a.loadCase('synthetic_five_layer');
    expect(useStrato.getState().config).toEqual(caseJson.data.config);
    expect(useStrato.getState().baselineConfig).toEqual(caseJson.data.config);
  });

  it('ignore an unknown case id', () => {
    useStrato.getState().loadCase('no-such-case');
    expect(useStrato.getState().caseId).toBe('synthetic_five_layer');
  });

  it('start with the default selection: baseline physics, steady-state operation, nothing expanded, the release graph', () => {
    expect(useStrato.getState().selection).toEqual({
      physics: 'baseline',
      mechanisms: { generation: false, decay: false, temperature: false },
      speciesPhysics: 'none',
      scenario: 'steady',
      expanded: [],
      graph: 'release',
      species: '',
      temperature: null,
      dataset: '',
      overlays: { methodA: true, methodB: true, experiment: true },
      controlledCompare: false,
    });
  });

  it('select and clear a layer', () => {
    useStrato.getState().selectLayer(3);
    expect(useStrato.getState().view.selectedLayer).toBe(3);
    useStrato.getState().selectLayer(null);
    expect(useStrato.getState().view.selectedLayer).toBeNull();
  });
});

describe('configDifferences', () => {
  it('is empty for equal configurations', () => {
    const s = useStrato.getState();
    expect(configDifferences(s.baselineConfig, s.config)).toEqual([]);
  });

  it('lists exactly the changed fields with both values', () => {
    const a = useStrato.getState();
    a.setRegionField(3, 'diffusivity_m2_s', 4e-14);
    a.setTimeField('end_time_s', 1e5);
    const s = useStrato.getState();
    const d = configDifferences(s.baselineConfig, s.config);
    expect(d.map((x) => x.path)).toEqual(['regions.3.diffusivity_m2_s', 'time.end_time_s']);
    expect(d[0]).toMatchObject({ baseline: caseJson.data.config.regions[3]?.diffusivity_m2_s, current: 4e-14 });
  });
});
