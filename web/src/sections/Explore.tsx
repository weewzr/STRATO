import { useId } from 'react';
import { repoFile } from '../build';
import { findCase } from '../data/case';
import { EquationBuilder } from '../equation/EquationBuilder';
import { TermExplanations } from '../equation/TermExplanations';
import { ControlPanel } from '../panels/ControlPanel';
import { ParticleSchematic } from '../panels/ParticleSchematic';
import { Placeholder } from '../panels/Placeholder';
import { hrefFor } from '../route';
import { configDifferences, useStrato } from '../state/store';
import { validateInputs } from '../state/validation';
import { useHeadingFocus } from './common';

/**
 * The equation-first Explore page (owner decision M7; prompt/DESIGN_SPECIFICATION.md Section 2.1):
 * A governing equation, B live simulation, C controls, D explanation, E future research, in this
 * order at every width. Web phase 1 builds the frame; each placeholder names the phase that fills it.
 */
export function Explore({ compact, onOpenControls }: { compact: boolean; onOpenControls: () => void }) {
  const config = useStrato((s) => s.config);
  const baseline = useStrato((s) => s.baselineConfig);
  const caseId = useStrato((s) => s.caseId);
  const stage = useStrato((s) => s.stage);
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
      <section className="block block--b" data-block="B" aria-labelledby={ids.b} data-testid="block-b">
        <div className="block__head block__head--row">
          <h2 id={ids.b} className="block__title">
            Live simulation
          </h2>
          <p className="block__context" data-testid="simulation-case">
            Simulation case: {entry?.title ?? caseId}, not AGR-1 data.
          </p>
        </div>
        <div className="strip" role="group" aria-label="Simulation strip" data-testid="sim-strip">
          <button type="button" className="btn btn--primary btn--small" aria-disabled="true" data-testid="strip-run" aria-describedby={`${ids.b}-solver`}>
            Run
          </button>
          <span className="strip__item" id={`${ids.b}-solver`}>
            Solver: not connected in this build (web phase 3)
          </span>
          <span className="strip__item" data-testid="strip-stage">
            Equation shown: stage {stage}; simulation: stage 1
          </span>
          <span className="strip__item strip__item--muted">Playback: web phase 4</span>
          {compact ? (
            <button type="button" className="btn btn--small strip__controls" onClick={onOpenControls} data-testid="strip-open-controls">
              Controls
            </button>
          ) : null}
        </div>
        <div className="sim">
          <div className="sim__particle">{entry ? <ParticleSchematic config={config} provenance={entry.provenance} /> : null}</div>
          <div className="sim__side">
            <div className="readout readout--wide" data-testid="release-readout">
              <p className="readout__label">Release fraction F(t)</p>
              <p className="readout__value readout__value--none">No result</p>
              <p className="readout__meta">
                <span className="readout__nolabel">Label: none</span> The value and its result label appear when the solver runs (web phase 3).
              </p>
            </div>
            <div className="sim__plots">
              <Placeholder phase="5" title="Radial concentration profile" testId="profile-placeholder" className="placeholder--plot">
                <p>c(r) at the playback time, with the layer boundaries as labelled bands in the layer colours.</p>
              </Placeholder>
              <Placeholder phase="5" title="Release fraction over time" testId="release-placeholder" className="placeholder--plot">
                <p>F(t) from the finite-volume solution and the eigenfunction series on the same axes, with a playback cursor.</p>
              </Placeholder>
            </div>
            <div className="readouts">
              <Placeholder phase="3" title="Inventory balance" testId="balance-placeholder" className="placeholder--compact">
                <p>Inventory plus released minus start inventory, with its residual, as the Rust wrapper returns it.</p>
              </Placeholder>
              <Placeholder phase="3" title="Numerical diagnostics" testId="diagnostics-placeholder" className="placeholder--compact">
                <p>Cells, steps, the time step used, the largest balance residual and the solver time.</p>
              </Placeholder>
            </div>
            <Placeholder phase="4" title="Reference fuel: AGR-1 UCO TRISO" testId="reference-fuel-placeholder" className="placeholder--compact">
              <p>
                The AGR-1 particle drawn from its registered geometry in{' '}
                <a href={repoFile('data/geometry/geometry.csv')}>
                  <code>data/geometry/geometry.csv</code>
                </a>
                , with the row ids and register status of its values. Geometry only: it is never combined with the diffusivities of the
                synthetic case.
              </p>
            </Placeholder>
          </div>
        </div>
      </section>

      {/* C: controls */}
      <section className="block block--c" data-block="C" aria-labelledby={ids.c} data-testid="block-c">
        <div className="block__head block__head--row">
          <h2 id={ids.c} className="block__title">
            Controls
          </h2>
          <p className="block__context">Inputs of the STRATO v1.0 format; no solver request is sent in this build.</p>
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
        <div className="block__head">
          <h2 id={ids.e} className="block__title">
            Future research
          </h2>
        </div>
        <div className="prose">
          <Placeholder phase="5.5" title="Research catalogue summary" testId="catalogue-placeholder">
            <p>
              The transport regimes, the species entries and the brick-by-brick roadmap, each with its research status, read from the
              research catalogue files of the repository. Nothing in it is simulated. The full catalogue will be in the{' '}
              <a href={hrefFor('research')}>Research</a> section.
            </p>
          </Placeholder>
        </div>
      </section>
    </article>
  );
}
