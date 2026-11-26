import { useId, useMemo, useState, type ReactNode } from 'react';
import { repoFile } from '../build';
import { CASES, type RegionConfig } from '../data/case';
import { layerColour } from '../design/layers';
import { isAvailable } from '../equation/availability';
import { SWITCH_MECHANISM, SWITCHES, switchIntroducedBy, termOf } from '../equation/terms';
import { Tex } from '../equation/Tex';
import { clamp, sciText } from '../format';
import { METHOD_1 } from '../methods';
import { statusOf, useCatalogue } from '../research/catalogue';
import { StatusChip } from '../research/StatusChip';
import { configDifferences, useStrato } from '../state/store';
import { Sci } from './Badge';
import { NumberField } from './NumberField';

// Slider span for the diffusivity, in decades. A display range only: the number field accepts any
// value, and the wrapper decides whether it is valid (it must be positive and finite).
const LOG_D_MIN = -18;
const LOG_D_MAX = -8;

export type ControlLayout = 'inline' | 'sheet';

function LayerFields({ r, i }: { r: RegionConfig; i: number }) {
  const setRegionField = useStrato((s) => s.setRegionField);
  const logD = Math.log10(r.diffusivity_m2_s);
  const sliderValue = Number.isFinite(logD) ? clamp(logD, LOG_D_MIN, LOG_D_MAX) : LOG_D_MIN;
  return (
    <>
      <div className="field">
        <label className="field__label" htmlFor={`d-slider-${i}`}>
          {r.name} diffusivity (m²/s), logarithmic slider
        </label>
        <input
          id={`d-slider-${i}`}
          className="slider"
          type="range"
          min={LOG_D_MIN}
          max={LOG_D_MAX}
          step={0.01}
          value={sliderValue}
          aria-valuetext={`${r.name} diffusivity = ${sciText(r.diffusivity_m2_s)} square metres per second`}
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
        label={`${r.name} diffusivity`}
        unit="m²/s"
        value={r.diffusivity_m2_s}
        onCommit={(v) => setRegionField(i, 'diffusivity_m2_s', v)}
        testId={`d-field-${i}`}
      />
      <NumberField
        label={`${r.name} outer radius`}
        unit="m"
        value={r.outer_radius_m}
        onCommit={(v) => setRegionField(i, 'outer_radius_m', v)}
        testId={`r-field-${i}`}
      />
      <NumberField
        label={`Grid intervals in the ${r.name} layer`}
        unit="dimensionless"
        value={r.cells}
        integer
        onCommit={(v) => setRegionField(i, 'cells', v)}
        testId={`n-field-${i}`}
        hint={i === 0 ? 'Whole number of at least 1 per layer; the grid has one more node than intervals.' : undefined}
      />
    </>
  );
}

/**
 * Block C: the parameter controls, bound to the store. Inline (tablet and desktop) the layers sit
 * side by side as one group per layer; in the phone bottom sheet each layer is a collapsible group.
 * Run asks the solver session (solver/session.ts) for a run; every input change also reruns it. The
 * wrapper checks every input: its own messages are listed under the controls when it rejects one.
 */
export function ControlPanel({ layout = 'inline', onClose }: { layout?: ControlLayout; onClose?: (() => void) | undefined }) {
  const config = useStrato((s) => s.config);
  const baseline = useStrato((s) => s.baselineConfig);
  const caseId = useStrato((s) => s.caseId);
  const mechanisms = useStrato((s) => s.mechanisms);
  const capabilities = useStrato((s) => s.capabilities);
  const stage = useStrato((s) => s.stage);
  const solverState = useStrato((s) => s.solver.state);
  const validation = useStrato((s) => s.validation);
  const requestRun = useStrato((s) => s.requestRun);
  const catalogue = useCatalogue();
  const introduced = switchIntroducedBy(stage);
  const { loadCase, setTimeField, setKernelConcentration, setTemperature, resetToBaseline, setCurrentAsBaseline, setMechanism } =
    useStrato.getState();

  const changed = useMemo(() => configDifferences(baseline, config).length, [baseline, config]);
  const [announce, setAnnounce] = useState('');
  const ids = { mech: useId(), run: useId(), step: useId(), title: useId(), caseHint: useId() };
  const sheet = layout === 'sheet';
  const errors = validation.state === 'rejected' || validation.state === 'error' ? validation.errors : [];

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
          Diffusivity <Sci value={r.diffusivity_m2_s} unit="m²/s" />
        </span>
      </>
    );
    const style = { ['--ring' as string]: layerColour(i) };
    return sheet ? (
      <details key={i} className="layer layer--collapsible" open={i === 0} data-testid={`layer-${i}`} style={style}>
        <summary className="layer__summary">{head}</summary>
        <div className="layer__body">
          <LayerFields r={r} i={i} />
        </div>
      </details>
    ) : (
      <fieldset key={i} className="layer layer--column" data-testid={`layer-${i}`} style={style}>
        <legend className="layer__legend">{head}</legend>
        <LayerFields r={r} i={i} />
      </fieldset>
    );
  });

  const startAndTime = (
    <>
      <fieldset className="group">
        <legend className="group__legend">Start and temperature</legend>
        <NumberField
          label="Initial kernel concentration"
          unit="mol/m³"
          value={config.initial.kernel_concentration_mol_m3}
          onCommit={setKernelConcentration}
          hint="Uniform in the kernel, zero in the coating layers."
        />
        <NumberField
          label="Temperature"
          unit="K"
          value={config.temperature_k}
          onCommit={setTemperature}
          hint="Required by the input format and recorded with the result. The solver does not use it: the diffusivities are constant."
        />
      </fieldset>
      <fieldset className="group">
        <legend className="group__legend">Time</legend>
        <NumberField label="Time step" unit="s" value={config.time.dt_s} onCommit={(v) => setTimeField('dt_s', v)} />
        <NumberField label="Simulation duration" unit="s" value={config.time.end_time_s} onCommit={(v) => setTimeField('end_time_s', v)} />
        <NumberField
          label="Time steps between output rows"
          unit="dimensionless"
          testId="output-every-field"
          value={config.time.output_every_steps}
          integer
          onCommit={(v) => setTimeField('output_every_steps', v)}
        />
        <p className="field__hint" id={ids.step}>
          The solver divides the duration into the whole number of equal steps that is at least the duration divided by the time step, and
          reports the step it used.
        </p>
        {validation.warnings.map((m) => (
          <p className="field__msg" key={m} role="status" data-testid="validation-warning">
            {m}
          </p>
        ))}
      </fieldset>
    </>
  );

  // Labels come from the term table (equation/terms.ts) and every status from the generated
  // catalogue; availability only from isAvailable(). The switch that the selected builder preview
  // stands for is marked, so the step picker and the switches stay in step.
  const mechanismSwitches = (
    <fieldset className="group group--mechs">
      <legend className="group__legend">Mechanism switches</legend>
      <ul className="mechs" role="list">
        {SWITCHES.map((sw) => {
          const term = termOf(sw.term);
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
                    <a href={repoFile('docs/development/phase2_review_package.md')}>Phase 2 review package</a>). A labelled preview of its equation
                    is shown in the equation builder.
                  </>
                )}
                {current ? <span className="mech__stage"> Previewed at step {stage}.</span> : null}
              </p>
            </li>
          );
        })}
      </ul>
      <p id={ids.mech} className="field__hint">
        Operating a switch that is not available changes no result and sends no solver request. Generation, decay and temperature
        dependence are previews until the solver implements and verifies them; species-specific mechanisms are research catalogue entries.
      </p>
    </fieldset>
  );

  const actions = (
    <div className="controls__actions">
      {errors.length > 0 ? (
        <div className="panel__issues" role="alert" data-testid="validation-errors">
          <p data-testid="issue-count">{errors.length === 1 ? 'The solver rejects these inputs, with 1 message:' : `The solver rejects these inputs, with ${errors.length} messages:`}</p>
          <ul className="panel__issue-list">
            {errors.map((m) => (
              <li key={m}>{m}</li>
            ))}
          </ul>
        </div>
      ) : (
        <p className="panel__issues panel__issues--ok" role="status" data-testid="issue-count">
          {changed === 0 ? 'Inputs equal the baseline.' : changed === 1 ? '1 input differs from the baseline.' : `${changed} inputs differ from the baseline.`}
        </p>
      )}
      <div className="panel__buttons">
        <button
          type="button"
          className="btn btn--primary"
          aria-disabled={solverState !== 'ready'}
          aria-describedby={ids.run}
          data-testid="run"
          onClick={() => {
            if (solverState !== 'ready') {
              setAnnounce(solverState === 'failed' ? 'Run is not available: the solver could not be loaded.' : 'The solver is still loading.');
              return;
            }
            requestRun();
            setAnnounce('Running the current inputs.');
          }}
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
        {METHOD_1}, the verified STRATO solver compiled to WebAssembly, runs in your browser. A change of input reruns it after a short
        pause; Run starts a run at once. The solver itself checks every input.
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
