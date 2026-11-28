import { useMemo, useState } from 'react';
import { CONFIG_LABELS, PHYSICS_CONFIG_IDS, type PhysicsConfigId } from '../equation/capability';
import { study39CurveFigure, STUDY39 } from '../graph/exploreFigures';
import { Plot } from '../plots/Plot';
import { usePlotTheme } from '../plots/theme';
import { TableBox } from '../sections/common';

export function Phase2StudyEvidence() {
  const [temp, setTemp] = useState<number>(1700);
  const [configId, setConfigId] = useState<PhysicsConfigId>('P01');
  const [showA, setShowA] = useState(true);
  const [showB, setShowB] = useState(true);
  const [showExp, setShowExp] = useState(true);
  const [controlled, setControlled] = useState(false);
  const theme = usePlotTheme();

  const { figure, methodBReason, methodBdrawn } = useMemo(
    () =>
      study39CurveFigure(
        temp,
        configId,
        { methodA: showA, methodB: showB, experiment: showExp },
        theme,
        { controlled, includeRef: configId !== 'P01' },
      ),
    [temp, configId, showA, showB, showExp, controlled, theme],
  );

  const act3And4 = useMemo(() => {
    const out: {
      temp: number;
      dataset: string;
      configId: string;
      stageAlias: string;
      fModelPct: number;
      fExpPct: number;
      deltaPp: number;
      labelText: string;
    }[] = [];
    for (const cid of [...PHYSICS_CONFIG_IDS, 'CTRL_G', 'CTRL_GL']) {
      for (const tc of [1700, 1800]) {
        const r = STUDY39.endpoint_deviations.find((e) => e.temperature_c === tc && e.config_id === cid && e.method === 'Method A');
        if (r) {
          out.push({
            temp: tc,
            dataset: tc === 1700 ? 'ACT3 (5.456%)' : 'ACT4 (14.710%)',
            configId: cid,
            stageAlias: r.stage_alias,
            fModelPct: r.f_model_pct,
            fExpPct: r.f_exp_or_u_pct,
            deltaPp: r.delta_pp,
            labelText: r.label_text,
          });
        }
      }
    }
    return out;
  }, []);

  return (
    <div data-testid="phase2-study39">
      <p>
        Canonical {STUDY39.graph_count}-graph Phase 3 investigation for <strong>{STUDY39.species}</strong> on AGR1-B-M16 across{' '}
        {STUDY39.temperatures.join(', ')} °C ({STUDY39.verification.run_count} audited runs across P01–P08 and matched-inventory controls:
        spatial convergence order {STUDY39.verification.spatial_order.toFixed(3)}, temporal order{' '}
        {STUDY39.verification.temporal_order.toFixed(3)}, max balance residual/bound{' '}
        {STUDY39.verification.max_residual_over_bound.toExponential(2)}, Stage A Method A vs B diff &le;{' '}
        {STUDY39.verification.max_ab_diff_pp.toExponential(2)} pp). Website catalog holds{' '}
        <strong>{STUDY39.web81.counts.total} entries</strong> ({STUDY39.web81.counts.method_a} Method A +{' '}
        {STUDY39.web81.counts.method_b_potentially_supported} Method B potential [{STUDY39.web81.counts.method_b_available_drawn} drawn] +{' '}
        {STUDY39.web81.counts.experimental} Experimental + {STUDY39.web81.counts.verification} Verification).
      </p>

      <div className="xselects" style={{ marginBottom: '0.5rem' }}>
        <label className="xselect">
          <span className="xselect__label">Species</span>
          <select value={STUDY39.species} disabled aria-label="Study 39 species">
            <option value="Cs-137">Cs-137</option>
          </select>
        </label>
        <label className="xselect">
          <span className="xselect__label">Temperature</span>
          <select value={temp} onChange={(e) => setTemp(Number(e.target.value))} data-testid="study39-temp">
            {STUDY39.temperatures.map((t) => (
              <option key={t} value={t}>
                {t} °C
              </option>
            ))}
          </select>
        </label>
        <label className="xselect xselect--wide">
          <span className="xselect__label">Physics configuration</span>
          <select value={configId} onChange={(e) => setConfigId(e.target.value as PhysicsConfigId)} data-testid="study39-stage">
            {PHYSICS_CONFIG_IDS.map((cid) => (
              <option key={cid} value={cid}>
                {cid} — {CONFIG_LABELS[cid].summary} ({CONFIG_LABELS[cid].stageAlias})
              </option>
            ))}
          </select>
        </label>
      </div>

      <div className="gchecks" role="group" aria-label="Documentation plot overlays">
        <label className="gcheck">
          <input type="checkbox" checked={showA} onChange={(e) => setShowA(e.target.checked)} />
          <span>Method A</span>
        </label>
        <label className="gcheck">
          <input type="checkbox" checked={showB} onChange={(e) => setShowB(e.target.checked)} />
          <span>Method B</span>
        </label>
        <label className="gcheck">
          <input type="checkbox" checked={showExp} onChange={(e) => setShowExp(e.target.checked)} />
          <span>Experimental data</span>
        </label>
        {(configId === 'P02' || configId === 'P05') && (
          <label className="gcheck">
            <input type="checkbox" checked={controlled} onChange={(e) => setControlled(e.target.checked)} />
            <span>Controlled matched-inventory</span>
          </label>
        )}
      </div>

      {figure && <Plot figure={figure} height={330} testId="study39-plot" label={`Phase 3 Cs-137 at ${temp} °C (${configId})`} />}
      {showB && !methodBdrawn && methodBReason && (
        <p style={{ fontSize: '0.85rem', marginTop: '0.35rem' }}>Method B omitted: {methodBReason}</p>
      )}

      <TableBox label="Website 81-entry verification inventory">
        <table className="table table--compact" data-testid="study39-web81-table">
          <caption className="table__caption">
            <strong>Table P3-1.</strong> 81-entry canonical Cs-137 simulation, experimental, and verification inventory.
          </caption>
          <thead>
            <tr>
              <th scope="col">Category</th>
              <th scope="col" className="num-col">Count</th>
              <th scope="col">Scope</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <th scope="row">Method A (BTCS)</th>
              <td className="num-col num">{STUDY39.web81.counts.method_a}</td>
              <td>P01–P08 × 6 temperatures (1200–1800 °C)</td>
            </tr>
            <tr>
              <th scope="row">Method B (Sturm–Liouville)</th>
              <td className="num-col num">{STUDY39.web81.counts.method_b_potentially_supported}</td>
              <td>Constant-T configs P01–P03, P05 ({STUDY39.web81.counts.method_b_available_drawn} drawn; 15 withheld/mask)</td>
            </tr>
            <tr>
              <th scope="row">Experimental datasets</th>
              <td className="num-col num">{STUDY39.web81.counts.experimental}</td>
              <td>D01 (1200) to D05 (1800 °C) + D06 normal-operation bound</td>
            </tr>
            <tr>
              <th scope="row">Numerical verification</th>
              <td className="num-col num">{STUDY39.web81.counts.verification}</td>
              <td>C01 convergence/mass balance, C02 A-vs-B, C03 deviations</td>
            </tr>
            <tr>
              <th scope="row">Total inventory</th>
              <td className="num-col num"><strong>{STUDY39.web81.counts.total}</strong></td>
              <td>Manifest verified against rust/study39 outputs</td>
            </tr>
          </tbody>
        </table>
      </TableBox>

      <TableBox label="Mapping between Stage A–D labels and P01–P08 checkbox configurations">
        <table className="table table--compact" data-testid="study39-mapping-table">
          <caption className="table__caption">
            <strong>Table P3-2.</strong> Equivalence between historical Stages A–D and independent Option B checkboxes P01–P08.
          </caption>
          <thead>
            <tr>
              <th scope="col">Config</th>
              <th scope="col">Report alias</th>
              <th scope="col">Gen G</th>
              <th scope="col">Decay λ</th>
              <th scope="col">T(t)</th>
            </tr>
          </thead>
          <tbody>
            {STUDY39.web81.stage_to_p_map.map((m) => (
              <tr key={m.config_id}>
                <th scope="row">{m.config_id}</th>
                <td>{m.stage}</td>
                <td>{m.generation ? 'On' : 'Off'}</td>
                <td>{m.decay ? 'On' : 'Off'}</td>
                <td>{m.temperature ? 'On' : 'Off'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </TableBox>

      <TableBox label="Comparable endpoint deviations at 1700 °C ACT3 and 1800 °C ACT4">
        <table className="table table--compact" data-testid="study39-exact-table">
          <caption className="table__caption">
            <strong>Table P3-3.</strong> Canonical final-endpoint comparison (Δ = Model − Experiment) at 1700 °C (ACT3) and 1800 °C (ACT4).
          </caption>
          <thead>
            <tr>
              <th scope="col">Config</th>
              <th scope="col">Test</th>
              <th scope="col" className="num-col">Model (%)</th>
              <th scope="col" className="num-col">Exp (%)</th>
              <th scope="col" className="num-col">End Δ (pp)</th>
            </tr>
          </thead>
          <tbody>
            {act3And4.map((r) => (
              <tr key={`${r.configId}-${r.temp}`}>
                <th scope="row">{r.configId}</th>
                <td>{r.dataset}</td>
                <td className="num-col num">{r.fModelPct.toFixed(3)}</td>
                <td className="num-col num">{r.fExpPct.toFixed(3)}</td>
                <td className="num-col num">{r.deltaPp >= 0 ? `+${r.deltaPp.toFixed(3)}` : r.deltaPp.toFixed(3)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </TableBox>
    </div>
  );
}
