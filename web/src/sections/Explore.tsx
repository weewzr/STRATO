import { useId } from 'react';
import { simulationMethod } from '../equation/availability';
import { PHYSICS_LABEL } from '../equation/physics';
import { unsupportedMessage } from '../equation/capability';
import { ActiveModel } from '../equation/ActiveModel';
import { GoverningEquation } from '../equation/GoverningEquation';
import { NumericalInterpretation } from '../equation/NumericalInterpretation';
import { ScenarioPicker } from '../equation/ScenarioPicker';
import { SCENARIO_LABEL } from '../equation/scenarios';
import { useSelectionSupport } from '../equation/useCapability';
import { findCase } from '../data/case';
import { BalancePanel, DiagnosticsPanel, ReleaseReadout, RunProblemNote, SolverStatus } from '../panels/SimResults';
import { METHOD_1 } from '../methods';
import { ControlPanel } from '../panels/ControlPanel';
import { Playback } from '../panels/Playback';
import { ParticleView } from '../particle/ParticleView';
import { ReferenceParticle } from '../particle/ReferenceParticle';
import { SimPlots } from '../plots/SimPlots';
import { hrefDocumentation } from '../route';
import { configDifferences, inputKey, useStrato } from '../state/store';
import { useHeadingFocus } from './common';

/**
 * The Explore page in the hierarchy of the owner's directive of 26 Nov 2026 (Section 9, decision 70):
 * A governing equation with its physics controls, B operating scenarios (the row of buttons with the
 * explanation of the selected one), C Numerical Interpretation (one collapsed region), D live simulation (the
 * particle and the numerical controls), E results (model output only). One shared selection (state/store.ts)
 * drives them. Content that belongs to none of the five blocks lives on the technical pages: the
 * research catalogue summary under Research, the boundary conditions and the note on verification
 * under Verify.
 */
export function Explore({ compact, onOpenControls }: { compact: boolean; onOpenControls: () => void }) {
  const config = useStrato((s) => s.config);
  const baseline = useStrato((s) => s.baselineConfig);
  const caseId = useStrato((s) => s.caseId);
  const solver = useStrato((s) => s.solver);
  const status = useStrato((s) => s.status);
  const requestRun = useStrato((s) => s.requestRun);
  const runInfo = useStrato((s) => s.runInfo);
  const capabilities = useStrato((s) => s.capabilities);
  const physics = useStrato((s) => s.selection.physics);
  const scenario = useStrato((s) => s.selection.scenario);
  const method = simulationMethod(capabilities);
  const support = useSelectionSupport();
  const entry = findCase(caseId);
  const h1 = useHeadingFocus();
  const ids = { d: useId(), e: useId(), c: useId() };
  const validation = useStrato((s) => s.validation);
  const issues = validation.state === 'rejected' || validation.state === 'error' ? validation.errors.length : 0;
  const changed = configDifferences(baseline, config).length;
  const note = unsupportedMessage(support);

  return (
    <article className="section explore" data-section="explore">
      {/* A: governing equation and its physics controls */}
      <section className="block block--a" data-block="A" aria-labelledby="explore-title" data-testid="block-a">
        <div className="block__head block__head--a">
          <p className="block__eyebrow">Governing equation</p>
          <h1 id="explore-title" ref={h1} tabIndex={-1} className="block__title block__title--page">
            Fission-product diffusion through the five layers of a TRISO particle
          </h1>
        </div>
        <GoverningEquation />
      </section>

      {/* B: operating scenarios, directly beneath the physics controls */}
      <ScenarioPicker />

      {/* C: Numerical Interpretation, one collapsed region */}
      <NumericalInterpretation />

      {/* D: live simulation of the selection above */}
      <section
        className="block block--d"
        data-block="D"
        aria-labelledby={ids.d}
        data-testid="block-d"
        data-input-key={inputKey(config)}
        data-result-key={support.supported ? (runInfo?.inputKey ?? '') : ''}
        data-run-status={support.supported ? status : 'none'}
        data-supported={support.supported ? 'true' : 'false'}
      >
        <div className="block__head block__head--row">
          <h2 id={ids.d} className="block__title">
            Live simulation
          </h2>
          <p className="block__context" data-testid="simulation-case">
            Case: {entry?.title ?? caseId} <a href={hrefDocumentation('explore-case')} className="link-target">
              Case and scope
            </a>
          </p>
          <SolverStatus />
        </div>
        {support.supported ? <ActiveModel /> : null}
        {support.supported ? (
          <>
            <div className="strip" role="group" aria-label="Simulation strip" data-testid="sim-strip">
              <button
                type="button"
                className="btn btn--primary btn--small"
                aria-disabled={solver.state !== 'ready'}
                data-testid="strip-run"
                aria-describedby={`${ids.d}-solver`}
                onClick={() => {
                  if (solver.state === 'ready') requestRun();
                }}
              >
                Run
              </button>
              <span className="strip__item" id={`${ids.d}-solver`} data-testid="strip-solver" data-state={solver.state}>
                {solver.state === 'ready' && solver.build
                  ? `Solver: WebAssembly, ${status === 'running' ? 'running' : 'ready'}`
                  : solver.state === 'failed'
                    ? 'Solver: not available'
                    : 'Solver: loading'}
              </span>
              <span className="strip__item" data-testid="strip-stage">
                Physics: {PHYSICS_LABEL[physics]}; scenario: {scenario === null ? 'none' : SCENARIO_LABEL[scenario]}; simulation: {method === 'btcs' ? METHOD_1 : method === 'loading' ? 'solver loading' : 'solver not available'}
              </span>
              <Playback />
              {compact ? (
                <button type="button" className="btn btn--small strip__controls" onClick={onOpenControls} data-testid="strip-open-controls">
                  Controls
                </button>
              ) : null}
            </div>
            <RunProblemNote />
            <div className="sim">
              <div className="sim__particle">{entry ? <ParticleView config={config} /> : null}</div>
              <div className="sim__side">
                <ReferenceParticle />
              </div>
            </div>
          </>
        ) : (
          <p className="sim-alert" role="status" data-testid="unsupported-simulation">
            <strong className="sim-alert__title">No simulation for this selection:</strong> {note.charAt(0).toLowerCase() + note.slice(1)}
          </p>
        )}
        <div className="controls-block" role="group" aria-label="Numerical controls" data-testid="controls-block">
          <h3 id={ids.c} className="controls-block__title">
            Numerical controls
          </h3>
          {compact ? (
            <div className="controls-summary" data-testid="controls-summary">
              <p className="controls-summary__text" role="status">
                {issues > 0
                  ? `${issues === 1 ? '1 input needs' : `${issues} inputs need`} attention.`
                  : changed === 0
                    ? 'Inputs equal the baseline.'
                    : `${changed === 1 ? '1 input differs' : `${changed} inputs differ`} from the baseline.`}
              </p>
              <button type="button" className="btn" onClick={onOpenControls} data-testid="open-panel">
                Open controls
              </button>
            </div>
          ) : (
            <ControlPanel layout="inline" />
          )}
        </div>
      </section>

      {/* E: results of the simulation above (model output only) */}
      {support.supported ? (
        <section className="block block--e" data-block="E" aria-labelledby={ids.e} data-testid="block-e" data-supported="true">
          <div className="block__head block__head--row">
            <h2 id={ids.e} className="block__title">
              Results
            </h2>
            <p className="block__context">
              <a href={hrefDocumentation('explore-figures')} className="link-target">
                Figure details
              </a>
            </p>
          </div>
          <ReleaseReadout />
          <div className="results-plots">
            <SimPlots compact={compact} />
          </div>
          <div className="results" role="group" aria-label="Inventory balance and numerical diagnostics of the run">
            <BalancePanel />
            <DiagnosticsPanel />
          </div>
        </section>
      ) : null}
    </article>
  );
}
