import { useMemo, type ReactNode } from 'react';
import { repoFile } from '../build';
import {
  codeBCDifferenceFigure,
  codeBCReleaseFigure,
  convergenceFigure,
  residualFigure,
  studyIndices,
  type ComparisonDoc,
  type ConvergenceDoc,
  type GuidesDoc,
  type PrecomputedHeader,
  type SyntheticRunDoc,
} from '../plots/figures';
import { Plot } from '../plots/Plot';
import { precomputedUrl, usePrecomputed } from '../plots/precomputed';
import { usePlotTheme } from '../plots/theme';
import { useStrato } from '../state/store';
import { Badge, Sci } from './Badge';

// The verification panel of the Verify section (web phase 5; prompt/DESIGN_SPECIFICATION.md Section
// 8.3): Code B against Code C, the convergence study with its observed orders, and the balance
// residual, from the precomputed output of the native program (label PRECOMPUTED, with its provenance
// header), and the live run's own diagnostics from the wrapper (COMPUTED v1.0). The pointwise difference
// |F_B − F_C| is shown only here, from the native file (coordinator decision of 2026-11-25).

const DERIVATION = 'docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md';

/** The provenance header of a precomputed file, as it is in the file. */
export function ProvenanceNote({ doc, file }: { doc: PrecomputedHeader; file: string }) {
  return (
    <span className="figure__source" data-testid={`prov-${file}`}>
      Source: <code>{file}</code> ({doc.label}), written by {doc.generator} from the native program{' '}
      {doc.strato_version ? `STRATO v${doc.strato_version}` : ''} at commit <code>{doc.commit.slice(0, 7)}</code>
      {doc.input_file ? (
        <>
          {' '}
          on <code>{doc.input_file}</code>
          {doc.input_sha256 ? (
            <>
              {' '}
              (SHA-256 <code className="hash">{doc.input_sha256.slice(0, 12)}</code>)
            </>
          ) : null}
        </>
      ) : null}
      ; command <code>{doc.command}</code>
      {doc.rustc ? (
        <>
          , {doc.rustc}
        </>
      ) : null}
      ; written {doc.created_utc}. <a href={precomputedUrl(file)}>Open the file</a>.
    </span>
  );
}

function Loading({ error, what }: { error: string | null; what: string }) {
  return (
    <p className="plotfig__empty" role={error ? 'alert' : 'status'}>
      {error ? `The precomputed ${what} could not be loaded (${error}).` : `Loading the precomputed ${what}…`}
    </p>
  );
}

function Fig({ number, testId, children, caption, label }: { number: number; testId: string; children: ReactNode; caption: ReactNode; label: 'PRECOMPUTED' | 'COMPUTED v1.0' }) {
  return (
    <figure className="figure plotfig" data-testid={testId}>
      <div className="figure__head">
        <Badge label={label} />
      </div>
      {children}
      <figcaption className="figure__caption">
        <strong>Figure {number}.</strong> {caption}
      </figcaption>
    </figure>
  );
}

function exportSource(doc: PrecomputedHeader, file: string): string {
  return `${file} (${doc.label}), native STRATO v${doc.strato_version} at commit ${doc.commit.slice(0, 7)}, ${doc.command}; synthetic five-layer verification case, not physical data.`;
}

export function CodeBCPanel() {
  const { doc, error } = usePrecomputed<ComparisonDoc>('comparison_fv_vs_analytical.json');
  const run = usePrecomputed<SyntheticRunDoc>('synthetic_run.json');
  const theme = usePlotTheme();
  const summary = run.doc?.data.summary ?? null;
  const floor = summary ? Number(summary.analytical_accuracy_floor_abs) : null;
  const release = useMemo(() => (doc ? codeBCReleaseFigure(doc, theme) : null), [doc, theme]);
  const diff = useMemo(() => (doc ? codeBCDifferenceFigure(doc, floor, theme) : null), [doc, floor, theme]);
  if (!doc || !release || !diff) return <Loading error={error} what="comparison of Code B and Code C" />;
  return (
    <div className="plot-grid plot-grid--two">
      <Fig
        number={1}
        testId="verify-bc-release"
        label="PRECOMPUTED"
        caption={
          <>
            Release fraction of the verified synthetic case from the finite-volume solution (Code B, solid) and the exact eigenfunction series
            (Code C, dashed), as the native program wrote them (<code>comparison_fv_vs_analytical.csv</code>, equal to{' '}
            <code>tests/golden/v1.0/</code>). <ProvenanceNote doc={doc} file="comparison_fv_vs_analytical.json" />
          </>
        }
      >
        <Plot figure={release} testId="verify-bc-release-plot" height={280} label="Release fraction against time of the verified case from Code B and Code C, precomputed by the native program." exportInfo={{ name: `strato-verify-release-${doc.commit.slice(0, 7)}`, source: exportSource(doc, 'comparison_fv_vs_analytical.json') }} />
      </Fig>
      <Fig
        number={2}
        testId="verify-bc-difference"
        label="PRECOMPUTED"
        caption={
          <>
            Pointwise difference |<i>F</i>
            <sub>B</sub> − <i>F</i>
            <sub>C</sub>| of the verified case at every output time, as the native program computed and wrote it (column{' '}
            <code>abs_error</code>); a discretisation error of Code B, not a defect. The dotted line is the series' accuracy floor
            {summary ? (
              <>
                {' '}
                ({summary.analytical_accuracy_floor_abs}, valid from <i>t</i> = {summary.analytical_valid_from_s} s, {summary.analytical_modes}{' '}
                modes; largest difference {summary.max_abs_error_vs_analytical}, final {summary.final_abs_error_vs_analytical}, as printed by
                the program)
              </>
            ) : null}
            . This plot is shown only from the native file: for inputs changed on Explore, the page shows the two curves and the largest and
            final difference that the solver returns, because the pointwise difference of two nearly equal numbers depends on their last
            digits, which differ between the browser build and the native build. <ProvenanceNote doc={doc} file="comparison_fv_vs_analytical.json" />
          </>
        }
      >
        <Plot figure={diff} testId="verify-bc-difference-plot" height={280} label="Pointwise difference between Code B and Code C against time, logarithmic axis, precomputed by the native program." exportInfo={{ name: `strato-verify-difference-${doc.commit.slice(0, 7)}`, source: exportSource(doc, 'comparison_fv_vs_analytical.json') }} />
      </Fig>
    </div>
  );
}

export function ConvergencePanel() {
  const { doc, error } = usePrecomputed<ConvergenceDoc>('convergence_study.json');
  const guides = usePrecomputed<GuidesDoc>('convergence_guides.json');
  const theme = usePlotTheme();
  const spatial = useMemo(() => (doc ? convergenceFigure(doc, guides.doc, 'spatial', theme) : null), [doc, guides.doc, theme]);
  const temporal = useMemo(() => (doc ? convergenceFigure(doc, guides.doc, 'temporal', theme) : null), [doc, guides.doc, theme]);
  if (!doc || !spatial || !temporal) return <Loading error={error} what="convergence study" />;
  // First and last level of each study, selected from the file (no value is typed here).
  const p = doc.data.points;
  const si = studyIndices(doc, 'spatial');
  const ti = studyIndices(doc, 'temporal');
  const first = (ix: number[]) => ix[0] ?? 0;
  const last = (ix: number[]) => ix[ix.length - 1] ?? 0;
  const spatialOrders = doc.data.spatial_observed_orders.split(',').join(', ');
  const temporalOrders = doc.data.temporal_observed_orders.split(',').join(', ');
  const guideNote = guides.doc ? (
    <>
      {' '}
      The dashed guide line (<code>convergence_guides.json</code>, label {guides.doc.label}, written by <code>generate_data.py figures</code>) has
      the reference slope and passes through the coarsest point; it is a reading aid, not a result.
    </>
  ) : null;
  return (
    <>
      <dl className="orders" data-testid="observed-orders">
        <dt>Observed spatial orders, cells doubled</dt>
        <dd data-testid="orders-spatial" data-value={doc.data.spatial_observed_orders}>
          {spatialOrders}
        </dd>
        <dt>Observed temporal orders, step halved</dt>
        <dd data-testid="orders-temporal" data-value={doc.data.temporal_observed_orders}>
          {temporalOrders}
        </dd>
        <dt>Exact final release fraction (series)</dt>
        <dd>{doc.data.exact_final_release_fraction}</dd>
      </dl>
      <p className="muted">
        As the native program printed them (<Badge label="PRECOMPUTED" />
        ). Expected: about 2 in space and about 1 in time.
      </p>
      <div className="plot-grid plot-grid--two">
        <Fig
          number={3}
          testId="verify-spatial"
          label="PRECOMPUTED"
          caption={
            <>
              Spatial refinement: error of the final release fraction against the exact series for {p.total_cells[first(si)]} to{' '}
              {p.total_cells[last(si)]} cells at a step of {p.dt_s[first(si)]} s, on log-log axes; observed orders {spatialOrders}.
              {guideNote} <ProvenanceNote doc={doc} file="convergence_study.json" />
            </>
          }
        >
          <Plot figure={spatial} testId="verify-spatial-plot" height={280} label="Error of the final release fraction against the total cell count, log-log, with a guide line of slope 2." exportInfo={{ name: `strato-verify-spatial-${doc.commit.slice(0, 7)}`, source: exportSource(doc, 'convergence_study.json') }} />
        </Fig>
        <Fig
          number={4}
          testId="verify-temporal"
          label="PRECOMPUTED"
          caption={
            <>
              Temporal refinement: error of the final release fraction against the exact series for steps of {p.dt_s[first(ti)]} to{' '}
              {p.dt_s[last(ti)]} s on {p.total_cells[first(ti)]} cells, on log-log axes; observed orders {temporalOrders}. They fall below 1
              because each error is measured against the exact series and so also contains the fixed spatial error of the{' '}
              {p.total_cells[first(ti)]}-cell mesh, which matters more as the time error shrinks; against a reference on the same mesh the
              observed order is first order, as the derivation shows (
              <a href={`${repoFile(DERIVATION)}#67-observed-order-of-accuracy-and-convergence`}>derivation, Section 6.7</a>).{guideNote}{' '}
              <ProvenanceNote doc={doc} file="convergence_study.json" />
            </>
          }
        >
          <Plot figure={temporal} testId="verify-temporal-plot" height={280} label="Error of the final release fraction against the time step, log-log, with a guide line of slope 1." exportInfo={{ name: `strato-verify-temporal-${doc.commit.slice(0, 7)}`, source: exportSource(doc, 'convergence_study.json') }} />
        </Fig>
      </div>
    </>
  );
}

export function BalancePanelVerify() {
  const { doc, error } = usePrecomputed<SyntheticRunDoc>('synthetic_run.json');
  const live = useStrato((s) => s.run);
  const theme = usePlotTheme();
  const fig = useMemo(() => (doc ? residualFigure(doc, theme) : null), [doc, theme]);
  if (!doc || !fig) return <Loading error={error} what="native run" />;
  return (
    <Fig
      number={5}
      testId="verify-balance"
      label="PRECOMPUTED"
      caption={
        <>
          Inventory balance residual <i>M</i> + <i>M</i>
          <sub>rel</sub> − <i>M</i>
          <sup>0</sup> of the native run at every output time, as the program wrote it; zero in exact arithmetic, so what remains is
          round-off. Largest at the end of the run as printed: {doc.data.summary.max_final_balance_residual_mol ?? '–'} mol.{' '}
          {live ? (
            <span data-testid="verify-live-residual">
              The run now on Explore reports a largest |residual| of{' '}
              <span data-value={String(live.balance.max_abs_residual_relative)}>
                <Sci value={live.balance.max_abs_residual_relative} digits={3} />
              </span>{' '}
              of the start inventory (<Badge label={live.label} />, from the WebAssembly solver build <code>{live.engine.commit}</code>).
            </span>
          ) : null}{' '}
          <ProvenanceNote doc={doc} file="synthetic_run.json" />
        </>
      }
    >
      <Plot figure={fig} testId="verify-balance-plot" height={260} label="Inventory balance residual of the native run against time." exportInfo={{ name: `strato-verify-balance-${doc.commit.slice(0, 7)}`, source: exportSource(doc, 'synthetic_run.json') }} />
    </Fig>
  );
}
