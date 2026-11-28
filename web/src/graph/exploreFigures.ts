// The Plotly figures of INTERACTIVE GRAPHS on Explore:
// - Live WASM BTCS & Sturm-Liouville stage runs + comparison datasets
// - Canonical Phase 3 Cs-137 progressive physics matrix (P01–P08 x 1200–1800 °C) in Graph B style
// - Authentic Stage 5 operating-scenario curves (shutdown, startup, control-rod, refuelling)
import study39Raw from '../data/study39.json';
import { CONFIG_LABELS, type OverlayFlags, type PhysicsConfigId, type ScenarioId } from '../equation/capability';
import { SCENARIO_LABEL } from '../equation/scenarios';
import { METHOD_1, METHOD_2 } from '../methods';
import { axis, baseLayout } from '../plots/figures';
import type { Figure } from '../plots/Plot';
import type { Trace } from '../plots/plotly';
import type { PlotTheme } from '../plots/theme';
import type { AnalyticalReply, StageOk } from '../solver/types';
import type { ComparisonRecord } from './exploreData';
import { COLOURS } from './plotModel';

export interface Study39CurveEntry {
  time_h: number[];
  f_a: number[];
  f_b: (number | null)[];
  method_b_meta?: {
    available: boolean;
    reason_code: string | null;
    reason: string | null;
    valid_from_h: number | null;
    accuracy_floor: number | null;
  };
}

export interface Study39EndpointRow {
  temperature_c: number;
  config_id: string;
  stage_alias: string;
  curve_key: string;
  method: 'Method A' | 'Method B';
  t_final_h: number;
  exp_dataset: string;
  exp_kind: string;
  f_exp_or_u_pct: number;
  f_model_pct: number;
  delta_pp: number;
  violation_pp: number;
  ratio_model_over_exp: number;
  label_text: string;
  comparison_class: string;
}

export interface Study39ScenarioCurve {
  label: string;
  scenario: string;
  variant: string;
  context: string;
  nuclide: string;
  normal_temperature_c: number;
  event_time_h: number | null;
  plotted_quantity: string;
  final_rfg: number | null;
  final_post_event: number | null;
  time_h: number[];
  rf_gen: (number | null)[];
  post_event: (number | null)[];
}

export interface Study39Data {
  species: string;
  temperatures: number[];
  configs: {
    config_id: PhysicsConfigId;
    stage_alias: string;
    label: string;
    curve_suffix: string;
    generation: boolean;
    decay: boolean;
    temperature: boolean;
    controlled_id: 'CTRL_G' | 'CTRL_GL' | null;
  }[];
  control_configs: {
    config_id: 'CTRL_G' | 'CTRL_GL';
    stage_alias: string;
    label: string;
    curve_suffix: string;
    base_config: PhysicsConfigId;
  }[];
  config_ids: PhysicsConfigId[];
  verification: {
    spatial_order: number;
    temporal_order: number;
    max_residual_over_bound: number;
    run_count: number;
    max_ab_diff_pp: number;
  };
  curves: Record<string, Record<string, Study39CurveEntry>>;
  experimental: Record<string, { series: string; kind: string; time_h: number[]; f: number[] }[]>;
  endpoint_deviations: Study39EndpointRow[];
  web81: {
    counts: {
      method_a: number;
      method_b_potentially_supported: number;
      method_b_available_drawn: number;
      experimental: number;
      verification: number;
      total: number;
    };
    stage_to_p_map: {
      stage: string;
      config_id: PhysicsConfigId;
      generation: boolean;
      decay: boolean;
      temperature: boolean;
      curve_suffix: string;
    }[];
    entries: Record<string, unknown>[];
  };
  exact_summary: {
    case_id: string;
    stage_key: string;
    mae_pp: number;
    rmse_pp: number;
    iae_time_avg_pp: number;
    final_delta_pp: number;
  }[];
  stage_improvement: {
    case_id: string;
    stage_key: string;
    mae_pp: number;
    rmse_pp: number;
    iae_time_avg_pp: number;
    final_delta_pp: number;
    rmse_improvement_vs_A_pct: number;
  }[];
  scenarios: Record<'cs137' | 'sr90', Record<string, Study39ScenarioCurve>>;
  graph_count: number;
}

export const STUDY39 = study39Raw as unknown as Study39Data;

const LINE_WIDTH = 2;
const TIME_TITLE = 'Time (h)';
const RELEASE_TITLE = 'Cumulative release fraction (dimensionless)';
const ALL_OVERLAYS: OverlayFlags = { methodA: true, methodB: true, experiment: true };

const hoverRelease = (unit: string, name: string): string =>
  `Time (${unit}) = %{x:.6~g}<br>Cumulative release fraction (dimensionless) = %{y:.6~g}<extra>${name}</extra>`;

function methodA(t: readonly number[], f: readonly number[], id: string): Trace {
  return {
    type: 'scatter',
    name: METHOD_1,
    x: t,
    y: f,
    mode: 'lines',
    line: { color: COLOURS.A, width: LINE_WIDTH, dash: 'solid' },
    hovertemplate: hoverRelease('h', METHOD_1),
    meta: { id, label: METHOD_1, unit: '1' },
  };
}

function methodB(t: readonly number[], f: readonly (number | null)[], id: string): Trace {
  return {
    type: 'scatter',
    name: METHOD_2,
    x: t,
    y: f,
    mode: 'lines',
    line: { color: COLOURS.B, width: LINE_WIDTH, dash: 'solid' },
    hovertemplate: hoverRelease('h', METHOD_2),
    meta: { id, label: METHOD_2, unit: '1' },
  };
}

const releaseLayout = (theme: PlotTheme, timeUnit: string, extra: Record<string, unknown> = {}) =>
  baseLayout(theme, {
    xaxis: axis(theme, timeUnit === 'h' ? TIME_TITLE : `Time (${timeUnit})`, { rangemode: 'tozero', hoverformat: '.6~g' }),
    yaxis: axis(theme, RELEASE_TITLE, { rangemode: 'tozero' }),
    ...extra,
  });

/** Method B of the reply of analytical as a trace, or null where the wrapper gave no drawable value. */
export function methodBTrace(a: AnalyticalReply | null): Trace | null {
  if (a === null || !a.applicable || a.times_h === null || a.release_fraction === null) return null;
  if (!a.release_fraction.some((v) => v !== null)) return null;
  return methodB(a.times_h, a.release_fraction, 'stage-method-b');
}

/** Cumulative release fraction against time: respects independent Method A and Method B overlay flags. */
export function releaseFigure(stage: StageOk, theme: PlotTheme, overlays: OverlayFlags = ALL_OVERLAYS, analytical: AnalyticalReply | null = null): Figure | null {
  const data: Trace[] = [];
  if (overlays.methodA) data.push(methodA(stage.series.times_h, stage.series.release_fraction, 'stage-method-a'));
  if (overlays.methodB) {
    const b = methodBTrace(analytical);
    if (b !== null) data.push(b);
  }
  if (data.length === 0) return null;
  return { data, layout: releaseLayout(theme, 'h', { showlegend: data.length > 1 }) };
}

/** Method A against Method B: draws whichever selected curves exist when Method B answered. */
export function methodsFigure(stage: StageOk, analytical: AnalyticalReply | null, theme: PlotTheme, overlays: OverlayFlags = ALL_OVERLAYS): Figure | null {
  const b = methodBTrace(analytical);
  if (b === null) return null;
  const data: Trace[] = [];
  if (overlays.methodA) data.push(methodA(stage.series.times_h, stage.series.release_fraction, 'stage-method-a'));
  if (overlays.methodB) data.push(b);
  if (data.length === 0) return null;
  return { data, layout: releaseLayout(theme, 'h') };
}

/** Radial concentration profile of Method A at the times the reply gives, node by node. */
export function profileFigure(stage: StageOk, theme: PlotTheme): Figure | null {
  if (stage.profiles.length === 0) return null;
  const last = stage.profiles.length - 1;
  const data: Trace[] = stage.profiles.map((p, i) => ({
    type: 'scatter',
    name: `${String(p.t_h)} h`,
    x: stage.grid.node_radii_m,
    y: p.c_mol_per_m3,
    mode: 'lines',
    line: { color: COLOURS.A, width: i === last ? LINE_WIDTH : 1, dash: 'solid' },
    hovertemplate: `Radius (m) = %{x:.6~g}<br>Concentration (mol/m³) = %{y:.6~g}<extra>${String(p.t_h)} h</extra>`,
    meta: { id: `stage-profile-${String(i)}`, label: `${String(p.t_h)} h`, unit: 'mol/m3' },
  }));
  const shapes = stage.grid.interface_radii_m.map((r) => ({
    type: 'line',
    xref: 'x',
    yref: 'paper',
    x0: r,
    x1: r,
    y0: 0,
    y1: 1,
    line: { color: theme.axis, width: 1, dash: 'dot' },
  }));
  return {
    data,
    layout: baseLayout(theme, {
      xaxis: axis(theme, 'Radius (m)', { rangemode: 'tozero', exponentformat: 'power' }),
      yaxis: axis(theme, 'Concentration (mol/m³)', { rangemode: 'tozero', exponentformat: 'power' }),
      shapes,
    }),
  };
}

/** True where a point of a comparison file is an upper bound (kept apart from the measurements). */
export const isBound = (pointClass: string): boolean => /bound/i.test(pointClass);

/** The experimental comparison of a registered curve: respects independent Method A, Method B, and Experimental overlay checkboxes. */
export function experimentFigure(file: ComparisonRecord, theme: PlotTheme, withModel = true, overlays: OverlayFlags = ALL_OVERLAYS): Figure | null {
  const measured = file.experiment.points.filter((p) => !isBound(p.point_class));
  const bounds = file.experiment.points.filter((p) => isBound(p.point_class));
  const data: Trace[] = [];
  if (withModel && overlays.methodA) {
    data.push(methodA(file.model.method_a.time_h, file.model.method_a.release_fraction, 'exp-method-a'));
  }
  const b = file.model.method_b;
  if (withModel && overlays.methodB && b.status === 'available' && b.time_h && b.release_fraction && b.release_fraction.some((v) => v !== null)) {
    data.push(methodB(b.time_h, b.release_fraction, 'exp-method-b'));
  }
  const markers = (list: typeof measured, name: string, id: string, symbol: string): Trace => ({
    type: 'scatter',
    name,
    x: list.map((p) => p.time_h),
    y: list.map((p) => p.value),
    mode: 'markers',
    marker: { color: COLOURS.observations, size: 8, symbol, line: { color: '#ffffff', width: 1 } },
    hovertemplate: hoverRelease('h', name),
    meta: { id, label: name, unit: '1' },
  });
  if (overlays.experiment && measured.length > 0) data.push(markers(measured, 'Experimental measurements', 'exp-measurements', 'circle'));
  if (overlays.experiment && bounds.length > 0) data.push(markers(bounds, 'Experimental upper bounds', 'exp-upper-bounds', 'triangle-down'));
  if (data.length === 0) return null;

  const showMetricA = withModel && overlays.methodA;
  const showMetricB = withModel && overlays.methodB;
  const labels = file.r2_records.filter(
    (r) => r.status === 'available' && r.label_plain && ((r.method === 'A' && showMetricA) || (r.method === 'B' && showMetricB)),
  );
  const annotations = labels.map((r, i) => ({
    xref: 'paper',
    yref: 'paper',
    x: 0.02,
    y: 0.98,
    yshift: -18 * i,
    xanchor: 'left',
    yanchor: 'top',
    text: r.label_plain ?? '',
    showarrow: false,
    align: 'left',
    font: { size: 13, color: r.method === 'A' ? COLOURS.A : COLOURS.B },
  }));
  return { data, layout: releaseLayout(theme, 'h', { annotations, margin: { l: 64, r: 16, t: 40, b: 48, pad: 0 } }) };
}

/**
 * Look up the single immutable endpoint deviation row from data/study39/endpoint_deviations.csv
 * (never recomputed in browser JS).
 */
export function findEndpointRow(temp: number, configId: string, method: 'Method A' | 'Method B'): Study39EndpointRow | null {
  return (
    STUDY39.endpoint_deviations.find(
      (r) => r.temperature_c === temp && r.config_id === configId && r.method === method,
    ) ?? null
  );
}

/**
 * Canonical Phase 3 Cs-137 figure (P01–P08 across 1200–1800 °C) in Graph B style:
 * supports every non-empty overlay subset of (Method A, Method B, Experimental data), optional
 * controlled matched-inventory run (CTRL_G / CTRL_GL), and exactly one immutable Δ annotation per active model curve.
 */
export function study39CurveFigure(
  temp: number,
  configId: PhysicsConfigId,
  overlays: OverlayFlags,
  theme: PlotTheme,
  opts: { controlled?: boolean; includeRef?: boolean } = {},
): { figure: Figure | null; methodBReason: string | null; methodBdrawn: boolean } {
  const tKey = String(temp);
  const curve = STUDY39.curves[tKey]?.[configId];
  const refCurve = STUDY39.curves[tKey]?.P01;
  const expList = STUDY39.experimental[tKey] ?? [];
  if (!curve) return { figure: null, methodBReason: null, methodBdrawn: false };

  const hasMethodB = Boolean(curve.method_b_meta?.available) && curve.f_b.some((v) => v !== null);
  const methodBReason = hasMethodB ? null : (curve.method_b_meta?.reason ?? 'Method B applies only to constant-temperature cases with eigenfunctions uniform in time.');

  const traces: Trace[] = [];
  const annotations: Record<string, unknown>[] = [];

  if (opts.includeRef !== false && configId !== 'P01' && overlays.methodA && refCurve) {
    traces.push({
      type: 'scatter',
      mode: 'lines',
      name: 'P01 (Stage A) reference',
      x: refCurve.time_h,
      y: refCurve.f_a.map((v) => v * 100),
      line: { color: '#9CA3AF', dash: 'dot', width: 1.5 },
      hovertemplate: 'Time (hours) = %{x:.6~g}<br>Release (%) = %{y:.4~g}%<extra>P01 reference</extra>',
    });
  }

  const labelShort = `${configId} (${CONFIG_LABELS[configId].stageAlias})`;

  if (overlays.methodA) {
    traces.push({
      type: 'scatter',
      mode: 'lines',
      name: `${METHOD_1} — ${labelShort}`,
      x: curve.time_h,
      y: curve.f_a.map((v) => v * 100),
      line: { color: '#DC2626', dash: 'solid', width: 2.3 },
      hovertemplate: `Time (hours) = %{x:.6~g}<br>Cumulative Cs-137 release (%) = %{y:.4~g}%<extra>${METHOD_1} (${configId})</extra>`,
      meta: { id: `study39-a-${configId}`, label: `${METHOD_1} (${configId})`, unit: '%' },
    });
    const epA = findEndpointRow(temp, configId, 'Method A');
    if (epA) {
      annotations.push({
        xref: 'paper',
        yref: 'paper',
        x: 0.02,
        y: 0.98,
        xanchor: 'left',
        yanchor: 'top',
        text: `Method A (${configId}): ${epA.label_text}`,
        showarrow: false,
        align: 'left',
        font: { size: 12, color: '#DC2626' },
      });
    }
  }

  const methodBdrawn = overlays.methodB && hasMethodB;
  if (methodBdrawn) {
    traces.push({
      type: 'scatter',
      mode: 'lines',
      name: `${METHOD_2} — ${labelShort}`,
      x: curve.time_h,
      y: curve.f_b.map((v) => (v === null ? null : v * 100)),
      line: { color: '#2563EB', dash: 'solid', width: 2.0 },
      hovertemplate: `Time (hours) = %{x:.6~g}<br>Cumulative Cs-137 release (%) = %{y:.4~g}%<extra>${METHOD_2} (${configId})</extra>`,
      meta: { id: `study39-b-${configId}`, label: `${METHOD_2} (${configId})`, unit: '%' },
    });
    const epB = findEndpointRow(temp, configId, 'Method B');
    if (epB) {
      annotations.push({
        xref: 'paper',
        yref: 'paper',
        x: 0.02,
        y: 0.98,
        yshift: -18 * annotations.length,
        xanchor: 'left',
        yanchor: 'top',
        text: `Method B (${configId}): ${epB.label_text}`,
        showarrow: false,
        align: 'left',
        font: { size: 12, color: '#2563EB' },
      });
    }
  }

  const ctrlId = configId === 'P02' ? 'CTRL_G' : configId === 'P05' ? 'CTRL_GL' : null;
  if (opts.controlled && ctrlId && overlays.methodA) {
    const cCurve = STUDY39.curves[tKey]?.[ctrlId];
    if (cCurve) {
      traces.push({
        type: 'scatter',
        mode: 'lines',
        name: `Controlled matched-inventory (${ctrlId})`,
        x: cCurve.time_h,
        y: cCurve.f_a.map((v) => v * 100),
        line: { color: '#991B1B', dash: 'dash', width: 2.0 },
        hovertemplate: `Time (hours) = %{x:.6~g}<br>Cumulative Cs-137 release (%) = %{y:.4~g}%<extra>${ctrlId}</extra>`,
      });
      const epC = findEndpointRow(temp, ctrlId, 'Method A');
      if (epC) {
        annotations.push({
          xref: 'paper',
          yref: 'paper',
          x: 0.02,
          y: 0.98,
          yshift: -18 * annotations.length,
          xanchor: 'left',
          yanchor: 'top',
          text: `${ctrlId}: ${epC.label_text}`,
          showarrow: false,
          align: 'left',
          font: { size: 12, color: '#991B1B' },
        });
      }
    }
  }

  if (overlays.experiment) {
    for (const s of expList) {
      const isUb = s.kind === 'upper_bound';
      const isNc = s.kind === 'not_comparable_plotted';
      traces.push({
        type: 'scatter',
        mode: 'markers',
        name: s.series,
        x: s.time_h,
        y: s.f.map((v) => v * 100),
        marker: isNc
          ? { color: '#6B7280', symbol: 'circle-open', size: 6 }
          : { color: '#111111', symbol: isUb ? 'triangle-down' : 'circle', size: isUb ? 9 : 7 },
        hovertemplate: `Time (hours) = %{x:.6~g}<br>Measured Cs-137 release (%) = %{y:.4~g}%<extra>${s.series}</extra>`,
        meta: { id: `study39-exp-${s.series}`, label: s.series, unit: '%' },
      });
    }
  }

  if (traces.length === 0) return { figure: null, methodBReason, methodBdrawn };

  const layout = baseLayout(theme, {
    title: { text: `Cs-137 — ${temp} °C (${labelShort})`, font: { size: 13 } },
    xaxis: axis(theme, 'Time (hours)', { rangemode: 'tozero', hoverformat: '.6~g' }),
    yaxis: axis(theme, 'Cumulative Cs-137 release (%)', { rangemode: 'tozero' }),
    annotations,
    margin: { l: 64, r: 16, t: 40, b: 48, pad: 0 },
  });
  return { figure: { data: traces, layout }, methodBReason, methodBdrawn };
}

/**
 * Authentic Stage 5 operating-scenario curve for Cs-137 or Sr-90 alongside the steady-state reference
 * at normal irradiation conditions (1028 °C), plus post-refuelling heating variant where applicable.
 */
export function scenarioCurveFigure(
  speciesId: 'cs137' | 'sr90',
  scenario: Exclude<ScenarioId, 'steady'>,
  temp: number | null,
  overlays: OverlayFlags,
  theme: PlotTheme,
): { figure: Figure | null; series: Study39ScenarioCurve | null } {
  const spMap = STUDY39.scenarios[speciesId];
  if (!spMap || !overlays.methodA) return { figure: null, series: null };

  const ref = spMap.steady;
  const variantKey =
    scenario === 'refuelling' && temp && [1600, 1700, 1800].includes(temp) ? `refuelling-${temp}` : scenario;
  const cur = spMap[variantKey] ?? spMap[scenario];
  if (!cur) return { figure: null, series: null };

  const traces: Trace[] = [];
  if (ref) {
    traces.push({
      type: 'scatter',
      mode: 'lines',
      name: 'Method A — Steady-state (1028 °C reference)',
      x: ref.time_h,
      y: ref.rf_gen.map((v) => (v === null ? null : v * 100)),
      line: { color: '#9CA3AF', dash: 'dot', width: 1.6 },
      hovertemplate: 'Time (hours) = %{x:.6~g}<br>Release of generated (%) = %{y:.4~e}%<extra>Steady reference</extra>',
    });
  }
  traces.push({
    type: 'scatter',
    mode: 'lines',
    name: `Method A — ${SCENARIO_LABEL[scenario]}${variantKey.includes('refuelling-') ? ` (+${temp} °C heating)` : ''}`,
    x: cur.time_h,
    y: cur.rf_gen.map((v) => (v === null ? null : v * 100)),
    line: { color: '#DC2626', dash: 'solid', width: 2.3 },
    hovertemplate: `Time (hours) = %{x:.6~g}<br>Release of generated (%) = %{y:.4~e}%<extra>${cur.nuclide} ${SCENARIO_LABEL[scenario]}</extra>`,
  });

  const layout = baseLayout(theme, {
    title: { text: `${cur.nuclide} — ${SCENARIO_LABEL[scenario]} (BTCS operating history)`, font: { size: 13 } },
    xaxis: axis(theme, 'Time (hours)', { rangemode: 'tozero', hoverformat: '.6~g' }),
    yaxis: axis(theme, 'Cumulative release of generated inventory (%)', { rangemode: 'tozero' }),
    margin: { l: 68, r: 16, t: 40, b: 48, pad: 0 },
  });
  return { figure: { data: traces, layout }, series: cur };
}
