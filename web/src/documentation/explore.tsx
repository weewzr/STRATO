import { repoFile } from '../build';
import { findCase } from '../data/case';
import { computedMechanisms, isSimulable, MECHANISM_NAME, phrase, SCENARIO_IDS } from '../equation/capability';
import { NUMERICAL_ITEMS } from '../equation/numerics';
import { NO_SCENARIO_TEXT_FULL, SCENARIO_HEADING, SCENARIO_KIND, SCENARIO_RULE_FULL, SCENARIOS } from '../equation/scenarios';
import { STEP_DEFS } from '../equation/terms';
import { useBuildCapability } from '../equation/useCapability';
import reference from '../generated/agr1_reference.json';
import { METHOD_1, METHOD_2 } from '../methods';
import { hrefDocumentation } from '../route';
import { useStrato } from '../state/store';
import type { DocContribution } from './types';

// Scientific material that left the Explore page in its minimal-text pass (decision 71), moved with its
// meaning unchanged: the case and the hints of the controls, the scope of the computation, the full text of
// the scenarios, the notes of the Numerical Interpretation, the details of the figures, the balance and
// diagnostics notes, the geometry note of the reference particle and the code traceability of the derivation.

function CaseEntry() {
  const caseId = useStrato((s) => s.caseId);
  const entry = findCase(caseId);
  return (
    <p>
      The manual baseline solver of this page runs the {entry?.title ?? 'synthetic five-layer verification case'}, which is not AGR-1 data. Its case file is{' '}
      {entry ? (
        <a href={repoFile(entry.provenance.input_file)}>
          <code>{entry.provenance.input_file}</code>
        </a>
      ) : (
        'the case file of the build'
      )}
      . The synthetic values are listed under{' '}
      <a href={hrefDocumentation('data-case-file')}>Parameters in use and their case file</a>; what no result from them is, is stated under{' '}
      <a href={hrefDocumentation('about-implemented')} data-testid="explore-case-implemented-link">What is implemented</a>.
    </p>
  );
}

/** What the computation of the running build implements, read from the capability of the build (what its wrapper reported). */
function ScopeEntry() {
  const { cap, loading } = useBuildCapability();
  if (loading) return <p>The capability of the build is loading.</p>;
  const { computed } = computedMechanisms(cap);
  const model = isSimulable(cap.baseline)
    ? `the baseline spherical diffusion in the five layers${
        computed.length > 0 ? ` with ${phrase(computed.map((m) => MECHANISM_NAME[m]))}` : ' (constant diffusivity in each layer, no generation and no decay)'
      }`
    : 'no model of the site';
  return (
    <p>
      {METHOD_1} implements {model}. Through the run_stage function of the wrapper this build computes the five model equations of Explore:
      baseline diffusion (stage 1), fission-product generation (stage 2), radioactive decay (stage 3), temperature-dependent diffusion
      (stage 4) and the combined model (stage 5), each with {METHOD_2} where it applies. Explore draws them for Steady-state operation; the
      other operations and Future species-specific physics show their equation only. A source starts from a fresh particle, and its graphs
      give the fraction of the generated inventory released. Which physics and which scenarios are implemented, verified or planned is
      read from what the WebAssembly wrapper of the build reports, never typed on the page.
    </p>
  );
}

function ScenariosEntry() {
  return (
    <>
      <p>{SCENARIO_RULE_FULL}</p>
      <p>{NO_SCENARIO_TEXT_FULL}</p>
      <p>Every scenario is labelled: {SCENARIO_KIND}</p>
      <dl className="prose-list" data-testid="documentation-scenarios">
        {SCENARIO_IDS.map((id) => (
          <div key={id}>
            <dt>{SCENARIO_HEADING[id]}</dt>
            <dd>{SCENARIOS[id].full.join(' ')}</dd>
          </div>
        ))}
      </dl>
    </>
  );
}

/** The notes that stood in the "Live values" disclosure of each derivation step, without the sentences that are registered elsewhere. They name the manual solver of this page, where the numbers of a run are shown. */
const LIVE_NOTES: { step: string; text: string }[] = Object.values(STEP_DEFS)
  .filter((d) => !d.live.startsWith('This step states mathematics') && !d.live.startsWith('The simulation below uses the synthetic'))
  .map((d) => ({ step: d.name, text: d.live.replace(' These checks verify the numerical method and are not experimental validation.', '') }));

function NumericalEntry() {
  return (
    <>
      <p>
        The Numerical Interpretation shows how the governing equation of Explore is discretised by the backward-time central-space (BTCS)
        scheme and solved in Rust. The graphs of Explore are the replies of the verified finite-difference BTCS Rust solver for the
        Model Equation chosen. Its fourteen items are those of the owner’s directive; each holds the derivation steps of the BTCS
        documents, and the equations are those of the documents.
      </p>
      <p>Notes on where the manual solver of this page shows the numbers of a step:</p>
      <ul className="prose-list" data-testid="documentation-live-notes">
        {LIVE_NOTES.map((n) => (
          <li key={n.step}>
            <strong>{n.step}.</strong> {n.text}
          </li>
        ))}
      </ul>
    </>
  );
}

function TraceabilityEntry() {
  return (
    <>
      <p>
        The code of the BTCS solver that implements each derivation step. The links of the Numerical Interpretation on Explore are named by
        the function; the file is given here. Code that exists on a development branch only is named, not linked.
      </p>
      <div className="table-box" role="region" aria-label="Code traceability of the derivation" tabIndex={0}>
        <table className="table" data-testid="documentation-traceability">
          <thead>
            <tr>
              <th scope="col">Item</th>
              <th scope="col">Step</th>
              <th scope="col">Function</th>
              <th scope="col">File</th>
            </tr>
          </thead>
          <tbody>
            {NUMERICAL_ITEMS.flatMap((it) =>
              it.stages.flatMap((st) => {
                const impl = STEP_DEFS[st].implementation;
                const rows = [
                  ...impl.code.map((c) => ({ label: c.label, path: c.path, href: repoFile(c.path) })),
                  ...(impl.devCode ?? []).map((c) => ({ label: c.label, path: c.path, href: '' })),
                ];
                return rows.map((r) => (
                  <tr key={`${st}:${r.path}:${r.label}`}>
                    <td className="num-col">{it.id}</td>
                    <td>{STEP_DEFS[st].name}</td>
                    <td>{r.href ? <a href={r.href}>{r.label}</a> : `${r.label} (development branch, not in this build)`}</td>
                    <td>
                      <code>{r.path}</code>
                    </td>
                  </tr>
                ));
              }),
            )}
          </tbody>
        </table>
      </div>
    </>
  );
}

function BalanceEntry() {
  const { cap, loading } = useBuildCapability();
  const { computed } = computedMechanisms(cap);
  // Fixed text only while the build reports that it computes neither generation nor decay (it was the wrapper's own note).
  const noSourceNoDecay = !loading && !computed.includes('generation') && !computed.includes('decay');
  return (
    <>
      <p>
        The inventory balance is taken at the end of the simulation; the residual is zero in exact arithmetic, and the solver reports the
        derived rounding bound beside it. The identity of the balance is: remaining + released + decayed − start − generated = residual
        (equation F30); the same identity written the other way round gives the residual the opposite sign.
        {noSourceNoDecay ? ' The solved model has no source and no decay, so both terms are exactly zero by construction; they are not measured.' : ''}
      </p>
      <p>
        The inventory by layer is the sum over the grid intervals of each layer, from the wrapper. The solver wall time is a measurement of
        the browser, not a result. {METHOD_1}, its solver version and its research status are stated in the status line of the
        simulation; the nature of the solution is that of <a href={hrefDocumentation('verify-scope')}>the scope of the solver</a>.
      </p>
      <p>
        The values of the particle inspector are inputs echoed by the solver, with the inventory and the share computed by the Rust
        wrapper; the concentration scale of the particle is the largest nodal concentration of the run, as returned by the solver.
      </p>
    </>
  );
}

function FiguresEntry() {
  return (
    <>
      <p>
        <strong>Figure 1</strong> shows the concentration at every node of the solver grid at the playback time, each node painted over its
        band of radii, on a fixed logarithmic colour scale over 6 decades up to the largest nodal concentration of the run (values at or
        below 10⁻⁶ of it take the lowest colour) or on a fixed linear scale from 0 to that concentration. The tick labels under the colour
        bar give the fraction of that largest concentration (dimensionless); the layers are drawn to scale with their boundaries in the
        layer colours. The playback shows the output times the solver recorded; nothing is interpolated. The colour tables cividis and
        viridis are those of matplotlib.
      </p>
      <p>
        <strong>Figure 2</strong> plots the concentration at each node of the solver grid at the playback time at the node radii, joined by
        straight lines, against the radius; the layers are the coloured bands, in the colours of the particle. The logarithmic axis spans
        six decades up to the largest concentration of the run, the scale of the particle colouring; values below it lie off the axis. The
        grey line is the start profile at time 0 s. The axis stays fixed during playback, so the frames compare.
      </p>
      <p>
        <strong>Figure 3</strong> shows the fraction of the initial inventory released through the outer surface, from {METHOD_1} (red
        line) and from {METHOD_2} (blue line), at the output times the solver recorded; the thin vertical line and the ring mark the
        playback time. {METHOD_2} is valid from a time that the plot marks (shaded before) and is accurate to the accuracy floor that the
        numerical diagnostics report (the thin grey horizontal line on the logarithmic axis); it uses the eigenmodes that the diagnostics
        count. Where {METHOD_2} was not computed for the inputs, the reason is listed under the numerical diagnostics.
      </p>
      <p>
        The pointwise difference of the two curves is not drawn on Explore: it is a difference of two nearly equal numbers, sensitive to
        their last digits, which differ between this browser build and the native build of the solver. The largest and the final
        difference that the solver returns are in the numerical diagnostics; the pointwise difference of the verified case, from the
        native program, is plotted under <a href={hrefDocumentation('evidence-verification')}>the verification evidence</a>.
      </p>
      <p>
        The values plotted are those returned by the solver; the CSV and JSON of the run are under <a href={hrefDocumentation('evidence-downloads')}>Data downloads</a>, and
        each figure can be saved as SVG or PNG with its source line, which carries the solver version and the build.
      </p>
    </>
  );
}

function ControlsEntry() {
  return (
    <ul className="prose-list" data-testid="documentation-controls">
      <li>
        The inputs of the solver are each checked by the solver itself; a change reruns {METHOD_1} after a short pause, and Run starts a
        run at once. {METHOD_1} is the verified STRATO solver compiled to WebAssembly and runs in the browser.
      </li>
      <li>Grid intervals: a whole number of at least 1 per layer; the grid has one more node than intervals.</li>
      <li>Initial kernel concentration: uniform in the kernel, zero in the coating layers.</li>
      <li>
        Temperature: required by the input format and recorded with the result. The solver does not use it: the diffusivities are
        constant.
      </li>
      <li>
        Time: the solver divides the duration into the whole number of equal steps that is at least the duration divided by the time
        step, and reports the step it used.
      </li>
    </ul>
  );
}

function ReferenceEntry() {
  return (
    <>
      <p>
        The reference fuel on Explore is geometry only (rule E2): the simulation stays the synthetic verification case, and AGR-1
        dimensions are never combined with its diffusivities. The AGR-1 comparisons of Explore use the registered temperature-dependent
        diffusivities and the registered heating histories, computed through run_stage.
      </p>
      <ul className="prose-list" data-testid="documentation-reference-sets">
        {reference.data.sets.map((s) => (
          <li key={s.set_id}>
            {s.set_id}:{' '}
            {s.layers.some((l) => l.outer_radius_source === 'generator')
              ? 'the outer radii are sums of the registered diameter and thicknesses, made by the data generator.'
              : 'the outer radii are as given in the register.'}
          </li>
        ))}
      </ul>
      <p>
        The source file is{' '}
        <a href={repoFile(reference.input_file)}>
          <code>{reference.input_file}</code>
        </a>
        , generated by <code>{reference.generator}</code>.
      </p>
    </>
  );
}

export const explore: DocContribution = {
  assumptions: [{ id: 'explore-case', title: 'Case of the manual solver', body: <CaseEntry /> }],
  limitations: [
    { id: 'explore-scope', title: 'What the build computes', body: <ScopeEntry /> },
    { id: 'explore-scenarios', title: 'Operating scenarios in full', body: <ScenariosEntry /> },
  ],
  derivations: [{ id: 'explore-numerical', title: 'Numerical Interpretation of Explore', body: <NumericalEntry /> }],
  verification: [{ id: 'explore-balance', title: 'Balance, diagnostics and inspector of the manual solver', body: <BalanceEntry /> }],
  interpretation: [
    {
      id: 'explore-results',
      title: 'Results of Explore',
      body: (
        <p>
          The graphs of Explore are model output, except the experimental comparison, which draws registered measurements (black markers;
          upper bounds as downward triangles) with a caption, an APA citation and a link. Its model curves are drawn only under
          temperature-dependent diffusion, because a comparison file is computed with temperature-dependent diffusion over the registered
          heating history; under another model equation the measurements stand alone. An R² is shown only where a record holds a value,
          otherwise a caption says why. See <a href={hrefDocumentation('interpretation')}>the experimental interpretation</a>.
        </p>
      ),
    },
  ],
  implementation: [
    { id: 'explore-figures', title: 'Figures of the manual solver', body: <FiguresEntry /> },
    { id: 'explore-controls', title: 'Numerical controls of the manual solver', body: <ControlsEntry /> },
    { id: 'explore-traceability', title: 'Code traceability of the derivation', body: <TraceabilityEntry /> },
  ],
  provenance: [{ id: 'explore-reference', title: 'Reference particle of Explore', body: <ReferenceEntry /> }],
};
