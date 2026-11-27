import { useId } from 'react';
import { simulationMethod } from '../equation/availability';
import { unsupportedMessage } from '../equation/capability';
import { ActiveModel } from '../equation/ActiveModel';
import { useSelectionSupport } from '../equation/useCapability';
import { findCase } from '../data/case';
import { METHOD_1 } from '../methods';
import { ControlPanel } from '../panels/ControlPanel';
import { Playback } from '../panels/Playback';
import { BalancePanel, DiagnosticsPanel, ReleaseReadout, RunProblemNote, SolverStatus } from '../panels/SimResults';
import { ParticleView } from '../particle/ParticleView';
import { ReferenceParticle } from '../particle/ReferenceParticle';
import { SimPlots } from '../plots/SimPlots';
import { hrefDocumentation } from '../route';
import { configDifferences, inputKey, useStrato } from '../state/store';

/**
 * The manual baseline solver of the earlier Explore page, moved here unchanged in behaviour: the Run button
 * and the playback, the particle view, the numerical controls (diffusivities, layer radii, grid, time step,
 * initial concentration, duration, output step), the results and the numerical diagnostics. It runs the
 * baseline problem with the solver of the page (Method A) whatever model equation and operation Explore has
 * selected. Explore no longer mounts it; Scientific Documentation mounts it under "Advanced solver
 * configuration" (`compact` and `onOpenControls` are the bottom-sheet entry of the phone layout; without
 * `onOpenControls` the controls are shown inline at every width).
 */
export function AdvancedSolver({ compact = false, onOpenControls }: { compact?: boolean; onOpenControls?: () => void }) {
  const config = useStrato((s) => s.config);
  const baseline = useStrato((s) => s.baselineConfig);
  const caseId = useStrato((s) => s.caseId);
  const solver = useStrato((s) => s.solver);
  const status = useStrato((s) => s.status);
  const requestRun = useStrato((s) => s.requestRun);
  const runInfo = useStrato((s) => s.runInfo);
  const capabilities = useStrato((s) => s.capabilities);
  const validation = useStrato((s) => s.validation);
  const method = simulationMethod(capabilities);
  const support = useSelectionSupport();
  const entry = findCase(caseId);
  const ids = { d: useId(), e: useId(), c: useId() };
  const issues = validation.state === 'rejected' || validation.state === 'error' ? validation.errors.length : 0;
  const changed = configDifferences(baseline, config).length;
  const note = unsupportedMessage(support);
  const sheet = compact && onOpenControls !== undefined;

  return (
    <div className="advanced-solver" data-testid="advanced-solver">
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
          <h3 id={ids.d} className="block__title">
            Live simulation
          </h3>
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
                Simulation: {method === 'btcs' ? METHOD_1 : method === 'loading' ? 'solver loading' : 'solver not available'}
              </span>
              <Playback />
              {sheet ? (
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
          <h4 id={ids.c} className="controls-block__title">
            Numerical controls
          </h4>
          {sheet ? (
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

      {support.supported ? (
        <section className="block block--e" data-block="E" aria-labelledby={ids.e} data-testid="block-e" data-supported="true">
          <div className="block__head block__head--row">
            <h3 id={ids.e} className="block__title">
              Results
            </h3>
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
    </div>
  );
}
