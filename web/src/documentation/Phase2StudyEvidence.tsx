import { useMemo, useState } from 'react';
import study39 from '../data/study39.json';
import { METHOD_1, METHOD_2 } from '../methods';
import { Plot, type Figure } from '../plots/Plot';
import type { Layout, Trace } from '../plots/plotly';
import { usePlotTheme } from '../plots/theme';
import { TableBox } from '../sections/common';

type StageKey = 'stage_a' | 'stage_b' | 'stage_c' | 'stage_c_decay_only' | 'stage_d_ramp_decay_off';

const STAGE_LABELS: Record<StageKey, string> = {
  stage_a: 'Stage A — Baseline (const D, G=0, λ=0)',
  stage_b: 'Stage B — +Fission generation G (Fresh, F_gen)',
  stage_c: 'Stage C — +Generation G & Decay λ (F_gen)',
  stage_c_decay_only: 'Stage C decay-only — G=0, +λ (F_init)',
  stage_d_ramp_decay_off: 'Stage D — Heating ramp D(r, T(t))',
};

export function Phase2StudyEvidence() {
  const [species] = useState<string>(study39.species);
  const [temp, setTemp] = useState<number>(1700);
  const [stage, setStage] = useState<StageKey>('stage_d_ramp_decay_off');
  const theme = usePlotTheme();

  const figure: Figure | null = useMemo(() => {
    const tKey = String(temp) as keyof typeof study39.curves;
    const curve = study39.curves[tKey]?.[stage];
    const stageA = study39.curves[tKey]?.stage_a;
    const expList = study39.experimental[tKey] ?? [];
    if (!curve) return null;

    const traces: Trace[] = [];
    if (stage !== 'stage_a' && stageA) {
      traces.push({
        type: 'scatter',
        mode: 'lines',
        name: 'Stage A baseline ref',
        x: stageA.time_h,
        y: stageA.f_a.map((v) => v * 100),
        line: { color: '#9CA3AF', dash: 'dot', width: 1.5 },
      });
    }
    traces.push({
      type: 'scatter',
      mode: 'lines',
      name: `${METHOD_1} (${STAGE_LABELS[stage].split('—')[0]?.trim() ?? stage})`,
      x: curve.time_h,
      y: curve.f_a.map((v) => v * 100),
      line: { color: '#DC2626', dash: 'solid', width: 2.3 },
    });
    if (curve.f_b.some((v) => v !== null)) {
      traces.push({
        type: 'scatter',
        mode: 'lines',
        name: METHOD_2,
        x: curve.time_h,
        y: curve.f_b.map((v) => (v === null ? null : v * 100)),
        line: { color: '#2563EB', dash: 'dash', width: 1.8 },
      });
    }
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
      });
    }

    const layout: Layout = {
      margin: { t: 36, r: 18, b: 48, l: 58 },
      title: { text: `Cs-137 — ${temp} °C (${STAGE_LABELS[stage]})`, font: { size: 13 } },
      xaxis: { title: { text: 'Time (hours)' }, gridcolor: theme.grid },
      yaxis: { title: { text: 'Cumulative release fraction (%)' }, gridcolor: theme.grid },
      legend: { orientation: 'h', y: -0.22 },
      height: 330,
      paper_bgcolor: 'transparent',
      plot_bgcolor: 'transparent',
      font: { color: theme.text },
    };
    return { data: traces, layout };
  }, [temp, stage, theme]);

  return (
    <div data-testid="phase2-study39">
      <p>
        Canonical {study39.graph_count}-graph Phase 2 investigation for <strong>{species}</strong> on AGR1-B-M16 across{' '}
        {study39.temperatures.join(', ')} °C ({study39.verification.run_count} runs: spatial convergence order{' '}
        {study39.verification.spatial_order.toFixed(3)}, temporal order {study39.verification.temporal_order.toFixed(3)}, max balance
        residual/bound {study39.verification.max_residual_over_bound.toExponential(2)}, Method A vs B diff &le;{' '}
        {study39.verification.max_ab_diff_pp.toExponential(2)} pp).
      </p>

      <div className="xselects" style={{ marginBottom: '0.75rem' }}>
        <label className="xselect">
          <span className="xselect__label">Species</span>
          <select value={species} disabled aria-label="Phase 2 species">
            <option value="Cs-137">Cs-137</option>
          </select>
        </label>
        <label className="xselect">
          <span className="xselect__label">Temperature</span>
          <select value={temp} onChange={(e) => setTemp(Number(e.target.value))} data-testid="study39-temp">
            {study39.temperatures.map((t) => (
              <option key={t} value={t}>
                {t} °C
              </option>
            ))}
          </select>
        </label>
        <label className="xselect xselect--wide">
          <span className="xselect__label">Stage</span>
          <select value={stage} onChange={(e) => setStage(e.target.value as StageKey)} data-testid="study39-stage">
            {(Object.keys(STAGE_LABELS) as StageKey[]).map((k) => (
              <option key={k} value={k}>
                {STAGE_LABELS[k]}
              </option>
            ))}
          </select>
        </label>
      </div>

      {figure && <Plot figure={figure} height={330} testId="study39-plot" label={`Phase 2 Cs-137 at ${temp} °C`} />}

      <TableBox label="Stage A to D exact deviation and improvement summary (ACT3 1700 °C and ACT4 1800 °C)">
        <table className="table table--compact" data-testid="study39-exact-table">
          <caption className="table__caption">
            <strong>Table P2-1.</strong> Exact time-series deviation (pp) and Stage A→D RMSE change for comparable Minato et al. (2000) HRB-22 tests (ACT3 N=8, ACT4 N=11).
          </caption>
          <thead>
            <tr>
              <th scope="col">Dataset</th>
              <th scope="col">Stage</th>
              <th scope="col" className="num-col">MAE</th>
              <th scope="col" className="num-col">RMSE</th>
              <th scope="col" className="num-col">IAE</th>
              <th scope="col" className="num-col">End Δ</th>
              <th scope="col" className="num-col">vs A</th>
            </tr>
          </thead>
          <tbody>
            {study39.stage_improvement.map((r) => {
              const stageShort =
                STAGE_LABELS[r.stage_key as StageKey]?.split('—')[0]?.trim().replace('Stage ', '') ?? r.stage_key;
              return (
                <tr key={`${r.case_id}-${r.stage_key}`}>
                  <th scope="row">{r.case_id.includes('ACT3') ? 'ACT3 (1700 °C)' : 'ACT4 (1800 °C)'}</th>
                  <td>{stageShort}</td>
                  <td className="num-col num">{r.mae_pp.toFixed(3)}</td>
                  <td className="num-col num">{r.rmse_pp.toFixed(3)}</td>
                  <td className="num-col num">{r.iae_time_avg_pp.toFixed(3)}</td>
                  <td className="num-col num">{r.final_delta_pp >= 0 ? `+${r.final_delta_pp.toFixed(3)}` : r.final_delta_pp.toFixed(3)}</td>
                  <td className="num-col num">{r.rmse_improvement_vs_A_pct >= 0 ? `+${r.rmse_improvement_vs_A_pct.toFixed(2)}%` : `${r.rmse_improvement_vs_A_pct.toFixed(2)}%`}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </TableBox>
    </div>
  );
}
