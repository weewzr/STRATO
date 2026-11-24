import { useId, useMemo, useState, type ReactNode } from 'react';
import { repoFile } from '../build';
import { CASES, type RegionConfig } from '../data/case';
import { layerColour } from '../design/layers';
import { isAvailable } from '../equation/availability';
import { SWITCH_MECHANISM, SWITCHES, switchIntroducedBy, TERMS } from '../equation/terms';
import { Tex } from '../equation/Tex';
import { clamp, sciText } from '../format';
import { statusOf, useCatalogue } from '../research/catalogue';
import { StatusChip } from '../research/StatusChip';
import { configDifferences, useStrato } from '../state/store';
import { validateInputs } from '../state/validation';
import { Sci } from './Badge';
import { NumberField } from './NumberField';

// Slider span for the diffusivity, in decades. A display range only: the number field accepts
// any positive finite value, which is the only rule Config::validate applies to D_i.
const LOG_D_MIN = -18;
const LOG_D_MAX = -8;

export type ControlLayout = 'inline' | 'sheet';

function LayerFields({ r, i, issueAt }: { r: RegionConfig; i: number; issueAt: (path: string) => string | undefined }) {
  const setRegionField = useStrato((s) => s.setRegionField);
  const logD = Math.log10(r.diffusivity_m2_s);
  const sliderValue = Number.isFinite(logD) ? clamp(logD, LOG_D_MIN, LOG_D_MAX) : LOG_D_MIN;
  return (
    <>
      <div className="field">
        <label className="field__label" htmlFor={`d-slider-${i}`}>
          D<sub>{i + 1}</sub>, logarithmic slider
        </label>
        <input
          id={`d-slider-${i}`}
          className="slider"
          type="range"
          min={LOG_D_MIN}
          max={LOG_D_MAX}
          step={0.01}
          value={sliderValue}
          aria-valuetext={`D${i + 1} = ${sciText(r.diffusivity_m2_s)} square metres per second`}
          onChange={(e) => setRegionField(i, 'diffusivity_m2_s', Number((10 ** Number(e.target.value)).toPrecision(3)))}
        />
        <div className="slider__ticks" aria-hidden="true">
          <span>
            10<sup>{String(LOG_D_MIN).replace('-', '−')}</sup>
          </span>
          <span>
            10<sup>{String(LOG_D_MAX).replace('-', '−')}</sup>
          </span>
        </div>
      </div>
      <NumberField
        label={
          <>
            Diffusivity D<sub>{i + 1}</sub>
          </>
        }
        unit="m² s⁻¹"
        value={r.diffusivity_m2_s}
        issue={issueAt(`regions.${i}.diffusivity_m2_s`)}
        onCommit={(v) => setRegionField(i, 'diffusivity_m2_s', v)}
        testId={`d-field-${i}`}
      />
      <NumberField
        label={
          <>
            Outer radius R<sub>{i + 1}</sub>
          </>
        }
        unit="m"
        value={r.outer_radius_m}
        issue={issueAt(`regions.${i}.outer_radius_m`)}
        onCommit={(v) => setRegionField(i, 'outer_radius_m', v)}
        testId={`r-field-${i}`}
      />
      <NumberField
        label="Cells"
        value={r.cells}
        integer
        issue={issueAt(`regions.${i}.cells`)}
        onCommit={(v) => setRegionField(i, 'cells', v)}
        testId={`n-field-${i}`}
      />
    </>
  );
}

/**
 * Block C: the parameter controls, bound to the store. Inline (tablet and desktop) the layers sit
 * side by side as one group per layer; in the phone bottom sheet each layer is a collapsible group.
 * Nothing here runs the solver: Run stays unavailable until web phase 3.
 */
export function ControlPanel({ layout = 'inline', onClose }: { layout?: ControlLayout; onClose?: (() => void) | undefined }) {
  const config = useStrato((s) => s.config);
  const baseline = useStrato((s) => s.baselineConfig);
  const caseId = useStrato((s) => s.caseId);
  const mechanisms = useStrato((s) => s.mechanisms);
  const capabilities = useStrato((s) => s.capabilities);
  const stage = useStrato((s) => s.stage);
  const catalogue = useCatalogue();
  const introduced = switchIntroducedBy(stage);
  const { loadCase, setTimeField, setKernelConcentration, setTemperature, resetToBaseline, setCurrentAsBaseline, setMechanism } =
    useStrato.getState();

  const issues = useMemo(() => validateInputs(config), [config]);
  const issueAt = (path: string) => issues.find((i) => i.path === path)?.message;
  const changed = useMemo(() => configDifferences(baseline, config).length, [baseline, config]);
  const [announce, setAnnounce] = useState('');
  const ids = { mech: useId(), run: useId(), step: useId(), title: useId(), caseHint: useId() };
  const sheet = layout === 'sheet';

  const caseField = (
    <div className="field controls__case">
      <label className="field__label" htmlFor={`case-select-${layout}`}>
        Case
      </label>
      <select
        id={`case-select-${layout}`}
        className="field__input"
        value={caseId}
        onChange={(e) => loadCase(e.target.value)}
        aria-describedby={ids.caseHint}
        data-testid="case-select"
      >
        {CASES.map((c) => (
          <option key={c.id} value={c.id}>
            {c.title}
          </option>
        ))}
      </select>
      <p className="field__hint" id={ids.caseHint}>
        Synthetic values chosen for verification, not measured TRISO properties.
      </p>
    </div>
  );

  const layers = config.regions.map((r, i) => {
    const head: ReactNode = (
      <>
        <span className="layer__swatch" aria-hidden="true" />
        <span className="layer__name">{r.name}</span>
        <span className="layer__d">
          D = <Sci value={r.diffusivity_m2_s} unit="m² s⁻¹" />
        </span>
      </>
    );
    const style = { ['--ring' as string]: layerColour(i) };
    return sheet ? (
      <details key={i} className="layer layer--collapsible" open={i === 0} data-testid={`layer-${i}`} style={style}>
        <summary className="layer__summary">{head}</summary>
        <div className="layer__body">
          <LayerFields r={r} i={i} issueAt={issueAt} />
        </div>
      </details>
    ) : (
      <fieldset key={i} className="layer layer--column" data-testid={`layer-${i}`} style={style}>
        <legend className="layer__legend">{head}</legend>
        <LayerFields r={r} i={i} issueAt={issueAt} />
      </fieldset>
    );
  });

  const startAndTime = (
    <>
      <fieldset className="group">
        <legend className="group__legend">Start and temperature</legend>
        <NumberField
          label={
            <>
              Kernel start concentration c<sub>0</sub>
            </>
          }
          unit="mol m⁻³"
          value={config.initial.kernel_concentration_mol_m3}
          issue={issueAt('initial.kernel_concentration_mol_m3')}
          onCommit={setKernelConcentration}
          hint="Uniform in the kernel, zero in the coating layers."
        />
        <NumberField
          label="Temperature T"
          unit="K"
          value={config.temperature_k}
          issue={issueAt('temperature_k')}
          onCommit={setTemperature}
          hint="Required by the input format. v1.0 records it and does not use it."
        />
      </fieldset>
      <fieldset className="group">
        <legend className="group__legend">Time</legend>
        <NumberField label="Time step Δt" unit="s" value={config.time.dt_s} issue={issueAt('time.dt_s')} onCommit={(v) => setTimeField('dt_s', v)} />
        <NumberField
          label={
            <>
              End time t<sub>end</sub>
            </>
          }
          unit="s"
          value={config.time.end_time_s}
          issue={issueAt('time.end_time_s')}
          onCommit={(v) => setTimeField('end_time_s', v)}
        />
        <NumberField
          label="Output every"
          unit="steps"
          value={config.time.output_every_steps}
          integer
          issue={issueAt('time.output_every_steps')}
          onCommit={(v) => setTimeField('output_every_steps', v)}
        />
        <p className="field__hint" id={ids.step}>
          Step rule of the solver: N = ⌈t<sub>end</sub> / Δt⌉ steps of length t<sub>end</sub> / N. The solver reports the step it used.
        </p>
      </fieldset>
    </>
  );

  // Labels come from the term table (equation/terms.ts) and every status from the generated
  // catalogue; availability only from isAvailable(). The switch that the selected builder stage
  // introduces is marked, so the stage picker and the switches stay in step.
  const mechanismSwitches = (
    <fieldset className="group group--mechs">
      <legend className="group__legend">Mechanism switches</legend>
      <ul className="mechs" role="list">
        {SWITCHES.map((sw) => {
          const term = TERMS[sw.term];
          const mechId = SWITCH_MECHANISM[sw.id];
          const status = statusOf(catalogue, mechId);
          const live = isAvailable(mechId, capabilities, catalogue) && mechanisms[sw.id].available;
          const current = introduced === sw.id;
          const why = `${ids.mech}-${sw.id}`;
          return (
            <li key={sw.id} className={`mech${current ? ' is-stage' : ''}`}>
              <button
                type="button"
                role="switch"
                className={`mech__switch mech__switch--${sw.id}`}
                aria-checked={live && mechanisms[sw.id].enabled}
                aria-disabled={!live}
                aria-describedby={why}
                data-testid={`mech-${sw.id}`}
                data-stage-current={current ? 'true' : undefined}
                onClick={() => {
                  if (!live) setAnnounce(`${term.name}: not available; status ${status ?? 'unknown'}; awaiting reviewed implementation.`);
                  else setMechanism(sw.id, !mechanisms[sw.id].enabled);
                }}
              >
                <span className={`mech__swatch mech__swatch--${sw.line}`} aria-hidden="true" />
                <span className="mech__name">{term.name}</span>
                <span className="mech__term">
                  <Tex latex={sw.latex} />
                </span>
                <span className="mech__state">{live && mechanisms[sw.id].enabled ? 'On' : 'Off'}</span>
              </button>
              <p id={why} className="mech__why">
                {live ? (
                  <>
                    {status ? <StatusChip status={status} mechanism={mechId} compact /> : null} Available: implemented in the solver at the live
                    status.
                  </>
                ) : (
                  <>
                    Not available: status {status ?? 'unknown'}; awaiting reviewed implementation (see the{' '}
                    <a href={repoFile('docs/development/phase2_review_package.md')}>Phase 2 review package</a>).
                  </>
                )}
                {current ? <span className="mech__stage"> Introduced by stage {stage}.</span> : null}
              </p>
            </li>
          );
        })}
      </ul>
      <p id={ids.mech} className="field__hint">
        Operating a switch that is not available changes no result and sends no solver request. Layer-specific temperature laws come with
        the temperature switch; species-specific mechanisms are research catalogue entries.
      </p>
    </fieldset>
  );

  const actions = (
    <div className="controls__actions">
      {issues.length > 0 ? (
        <p className="panel__issues" role="status" data-testid="issue-count">
          {issues.length === 1 ? '1 input needs attention.' : `${issues.length} inputs need attention.`}
        </p>
      ) : (
        <p className="panel__issues panel__issues--ok" role="status" data-testid="issue-count">
          {changed === 0 ? 'Inputs equal the baseline.' : changed === 1 ? '1 input differs from the baseline.' : `${changed} inputs differ from the baseline.`}
        </p>
      )}
      <div className="panel__buttons">
        <button
          type="button"
          className="btn btn--primary"
          aria-disabled="true"
          aria-describedby={ids.run}
          data-testid="run"
          onClick={() => setAnnounce('Run is not available yet: the Rust solver is connected in web phase 3.')}
        >
          Run
        </button>
        <button
          type="button"
          className="btn"
          data-testid="reset"
          onClick={() => {
            resetToBaseline();
            setAnnounce('Parameters reset to the baseline.');
          }}
        >
          Reset to baseline
        </button>
        <button
          type="button"
          className="btn"
          data-testid="set-baseline"
          onClick={() => {
            setCurrentAsBaseline();
            setAnnounce('Current parameters set as the baseline.');
          }}
        >
          Set current as baseline
        </button>
      </div>
      <p id={ids.run} className="field__hint">
        Run needs the Rust solver compiled to WebAssembly, which this build does not include yet (web phase 3).
      </p>
      <p className="visually-hidden" role="status" aria-live="polite" data-testid="announce">
        {announce}
      </p>
    </div>
  );

  if (sheet) {
    return (
      <section className="panel" aria-labelledby={ids.title} data-testid="controls" data-layout="sheet">
        <div className="panel__head">
          <h2 id={ids.title} className="panel__title" tabIndex={-1}>
            Controls
          </h2>
          {onClose ? (
            <button type="button" className="btn btn--quiet" onClick={onClose} data-testid="close-sheet">
              Close
            </button>
          ) : null}
        </div>
        <div className="panel__body">
          {caseField}
          <fieldset className="group">
            <legend className="group__legend">Layers</legend>
            {layers}
          </fieldset>
          {startAndTime}
          {mechanismSwitches}
        </div>
        <div className="panel__actions">{actions}</div>
      </section>
    );
  }

  return (
    <div className="controls" data-testid="controls" data-layout="inline">
      <div className="controls__top">
        {caseField}
        {actions}
      </div>
      <div className="controls__layers" role="group" aria-label="Layer parameters">
        {layers}
      </div>
      <div className="controls__rest">
        {startAndTime}
        {mechanismSwitches}
      </div>
    </div>
  );
}
