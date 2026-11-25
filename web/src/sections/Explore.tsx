import { useId } from 'react';
import { repoFile } from '../build';
import { findCase } from '../data/case';
import { solvedStage } from '../equation/availability';
import { EquationBuilder } from '../equation/EquationBuilder';
import { BalancePanel, DiagnosticsPanel, ReleaseReadout, RunProblemNote } from '../panels/SimResults';
import { TermExplanations } from '../equation/TermExplanations';
import { ControlPanel } from '../panels/ControlPanel';
import { Playback } from '../panels/Playback';
import { ParticleView } from '../particle/ParticleView';
import { ReferenceParticle } from '../particle/ReferenceParticle';
import { FutureResearch } from '../research/FutureResearch';
import { SimPlots } from '../plots/SimPlots';
import { hrefFor } from '../route';
import { configDifferences, inputKey, useStrato } from '../state/store';
import { validateInputs } from '../state/validation';
import { useHeadingFocus } from './common';

/**
 * The equation-first Explore page (owner decision M7; prompt/DESIGN_SPECIFICATION.md Section 2.1):
 * A governing equation, B live simulation, C controls, D explanation, E future research, in this
 * order at every width. Web phase 1 built the frame; web phase 3 connects the Rust solver in block B
 * (release readout, inventory balance, diagnostics); web phase 4 colours the particle from the
 * solver's profiles, adds playback, the 3D cutaway and the AGR-1 reference particle (geometry only);
 * web phase 5 adds the radial profile and the release curves beside the particle; web phase 5.5
 * fills block E with the summary of the research catalogue.
 */
export function Explore({ compact, onOpenControls }: { compact: boolean; onOpenControls: () => void }) {
  const config = useStrato((s) => s.config);
  const baseline = useStrato((s) => s.baselineConfig);
  const caseId = useStrato((s) => s.caseId);
  const stage = useStrato((s) => s.stage);
  const solver = useStrato((s) => s.solver);
  const status = useStrato((s) => s.status);
  const requestRun = useStrato((s) => s.requestRun);
  const runInfo = useStrato((s) => s.runInfo);
  const capabilities = useStrato((s) => s.capabilities);
  const solved = solvedStage(capabilities);
  const entry = findCase(caseId);
  const h1 = useHeadingFocus();
  const ids = { b: useId(), c: useId(), d: useId(), e: useId() };
  const issues = validateInputs(config).length;
  const changed = configDifferences(baseline, config).length;

  return (
    <article className="section explore" data-section="explore">
      {/* A: governing equation and staged builder */}
      <section className="block block--a" data-block="A" aria-labelledby="explore-title" data-testid="block-a">
        <div className="block__head block__head--a">
          <p className="block__eyebrow">Governing equation</p>
          <h1 id="explore-title" ref={h1} tabIndex={-1} className="block__title block__title--page">
            Fission-product diffusion through the five layers of a TRISO particle
          </h1>
        </div>
        <EquationBuilder compact={compact} />
      </section>

      {/* B: live simulation of the equation shown in A */}
      <section
        className="block block--b"
        data-block="B"
        aria-labelledby={ids.b}
        data-testid="block-b"
        data-input-key={inputKey(config)}
        data-result-key={runInfo?.inputKey ?? ''}
        data-run-status={status}
      >
        <div className="block__head block__head--row">
          <h2 id={ids.b} className="block__title">
            Live simulation
          </h2>
          <p className="block__context" data-testid="simulation-case">
            Simulation case: {entry?.title ?? caseId}, not AGR-1 data.
          </p>
        </div>
        <div className="strip" role="group" aria-label="Simulation strip" data-testid="sim-strip">
          <button
            type="button"
            className="btn btn--primary btn--small"
            aria-disabled={solver.state !== 'ready'}
            data-testid="strip-run"
            aria-describedby={`${ids.b}-solver`}
            onClick={() => {
              if (solver.state === 'ready') requestRun();
            }}
          >
            Run
          </button>
          <span className="strip__item" id={`${ids.b}-solver`} data-testid="strip-solver" data-state={solver.state}>
            {solver.state === 'ready' && solver.build
              ? `Solver: WebAssembly, ${status === 'running' ? 'running' : 'ready'}`
              : solver.state === 'failed'
                ? 'Solver: not available'
                : 'Solver: loading'}
          </span>
          <span className="strip__item" data-testid="strip-stage">
            Equation shown: stage {stage}; simulation: stage {solved > 0 ? solved : 1}
          </span>
          <Playback />
          {compact ? (
            <button type="button" className="btn btn--small strip__controls" onClick={onOpenControls} data-testid="strip-open-controls">
              Controls
            </button>
          ) : null}
        </div>
        <div className="sim">
          <div className="sim__particle">{entry ? <ParticleView config={config} provenance={entry.provenance} /> : null}</div>
          <div className="sim__side">
            <ReleaseReadout />
            <RunProblemNote />
            <SimPlots compact={compact} />
            <ReferenceParticle />
          </div>
        </div>
        <div className="results" role="group" aria-label="Inventory balance and numerical diagnostics of the run">
          <BalancePanel />
          <DiagnosticsPanel />
        </div>
      </section>

      {/* C: controls */}
      <section className="block block--c" data-block="C" aria-labelledby={ids.c} data-testid="block-c">
        <div className="block__head block__head--row">
          <h2 id={ids.c} className="block__title">
            Controls
          </h2>
          <p className="block__context">Inputs of the STRATO v1.0 format; a change reruns the solver after a short pause.</p>
        </div>
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
      </section>

      {/* D: scientific explanation */}
      <section className="block block--d" data-block="D" aria-labelledby={ids.d} data-testid="block-d">
        <div className="block__head">
          <h2 id={ids.d} className="block__title">
            Explanation
          </h2>
        </div>
        <div className="prose">
          <TermExplanations />
          <h3>The case on this page</h3>
          <p>
            The simulation uses the synthetic five-layer verification case,{' '}
            <a href={repoFile('inputs/synthetic/synthetic_five_layer.toml')}>
              <code>inputs/synthetic/synthetic_five_layer.toml</code>
            </a>
            . Its radii and diffusivities were chosen to test the solver. They are not measured TRISO properties, and no result from this case
            is a prediction of real fission-product release.
          </p>
          <h3>Boundary, interface and initial conditions</h3>
          <p>
            The centre of the particle is a point of symmetry, so no material crosses it. Neighbouring layers are in perfect contact: the
            concentration and the diffusive flux are continuous at every interface. The outer surface is a perfect sink, held at zero
            concentration. At the start the species is spread uniformly through the kernel and absent from the coating layers.
          </p>
          <h3>Verification is not validation</h3>
          <p>
            Verification shows that STRATO solves its own equations correctly; it is reported under <a href={hrefFor('verify')}>Verify</a>.
            Validation asks whether the model matches measured release from real particles; it needs the Phase 2 physics and is described
            under <a href={hrefFor('validate')}>Validate</a>.
          </p>
        </div>
      </section>

      {/* E: future research mechanisms */}
      <section className="block block--e" data-block="E" aria-labelledby={ids.e} data-testid="block-e">
        <div className="block__head block__head--row">
          <h2 id={ids.e} className="block__title">
            Future research
          </h2>
          <p className="block__context">From the research catalogue of the repository. Nothing in this block is simulated.</p>
        </div>
        <FutureResearch />
      </section>
    </article>
  );
}
