import { useMemo, type ReactNode } from 'react';
import { METHOD_1, METHOD_2 } from '../methods';
import {
  checkFigure,
  methodDifferenceFigure,
  methodReleaseFigure,
  residualFigure,
  type CheckDoc,
  type ComparisonDoc,
  type PrecomputedHeader,
  type RunDoc,
} from '../plots/figures';
import { Plot } from '../plots/Plot';
import { precomputedUrl, usePrecomputed } from '../plots/precomputed';
import { usePlotTheme } from '../plots/theme';
import { useStrato } from '../state/store';
import { Badge, Sci } from './Badge';
import { TableBox } from '../sections/common';

// The verification panels of the Verify section (web phase 5; prompt/DESIGN_SPECIFICATION.md Section
// 8.3): Method 1 against Method 2, the independent check with its observed orders in time and in
// space, and the balance residual with its derived rounding bound, from the precomputed output of the
// native program (label PRECOMPUTED, with its provenance header), and the live run's own diagnostics
// from the wrapper (COMPUTED v2.0). The pointwise difference between the two methods is shown only
// here, from the native file (coordinator decision of 2026-11-25). Numerical verification only.

/** The provenance header of a precomputed file, as it is in the file. */
export function ProvenanceNote({ doc, file }: { doc: PrecomputedHeader; file: string }) {
  return (
    <span className="figure__source" data-testid={`prov-${file}`}>
      Source: <code>{file}</code> ({doc.label}), written by {doc.converter} from the native program {doc.solver}, solver version{' '}
      {doc.strato_version}, at commit <code>{doc.commit.slice(0, 7)}</code>
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
      {doc.rustc ? <>, {doc.rustc}</> : null}; written {doc.created_utc}. <a href={precomputedUrl(file)}>Open the file</a>.
    </span>
  );
}

function Loading({ error, what }: { error: string | null; what: string }) {
  return (
    <p className="plotfig__empty" role={error ? 'alert' : 'status'}>
      {error ? `The precomputed ${what} could not be loaded (${error}). No other source of numbers is used.` : `Loading the precomputed ${what}…`}
    </p>
  );
}

function Fig({
  number,
  testId,
  children,
  caption,
  label,
}: {
  number: number;
  testId: string;
  children: ReactNode;
  caption: ReactNode;
  label: 'PRECOMPUTED' | 'COMPUTED v2.0';
}) {
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
  return `${file} (${doc.label}), native ${doc.solver}, solver version ${doc.strato_version} at commit ${doc.commit.slice(0, 7)}, ${doc.command}; synthetic five-layer verification case, not physical data.`;
}

/** Method 1 against Method 2: the two release fractions and their pointwise difference. */
export function MethodComparisonPanel() {
  const { doc, error } = usePrecomputed<ComparisonDoc>('btcs_vs_analytical.json');
  const theme = usePlotTheme();
  const floor = doc ? doc.data.analytical.accuracy_floor : null;
  const release = useMemo(() => (doc ? methodReleaseFigure(doc, theme) : null), [doc, theme]);
  const diff = useMemo(() => (doc ? methodDifferenceFigure(doc, floor, theme) : null), [doc, floor, theme]);
  if (!doc || !release || !diff) return <Loading error={error} what={`comparison of ${METHOD_1} and ${METHOD_2}`} />;
  const a = doc.data.analytical;
  return (
    <div className="plot-grid plot-grid--two">
      <Fig
        number={1}
        testId="verify-bc-release"
        label="PRECOMPUTED"
        caption={
          <>
            Release fraction of the verified synthetic case from {METHOD_1} (solid) and {METHOD_2} (dashed), the {a.name}, as the native
            program wrote them (<code>btcs_vs_analytical.csv</code>). <ProvenanceNote doc={doc} file="btcs_vs_analytical.json" />
          </>
        }
      >
        <Plot
          figure={release}
          testId="verify-bc-release-plot"
          height={280}
          label={`Release fraction against time of the verified case from ${METHOD_1} and ${METHOD_2}, precomputed by the native program.`}
          exportInfo={{ name: `strato-verify-release-${doc.commit.slice(0, 7)}`, source: exportSource(doc, 'btcs_vs_analytical.json') }}
        />
      </Fig>
      <Fig
        number={2}
        testId="verify-bc-difference"
        label="PRECOMPUTED"
        caption={
          <>
            Absolute difference of the release fractions of {METHOD_1} and {METHOD_2} of the verified case at every output time, as the Rust
            tool computed it from the native reply (column <code>Absolute difference of the release fractions (dimensionless)</code>); a
            discretisation error of {METHOD_1}, not a defect. The dotted line is the accuracy floor of {METHOD_2} ({a.accuracy_floor},
            valid from the time {a.valid_from_s} s, {a.modes} eigenmodes; largest difference {a.max_abs_difference}, final difference{' '}
            {a.final_abs_difference}). This plot is shown only from the native file: for inputs changed on Explore, the page shows the two
            curves and the largest and final difference that the solver returns, because the pointwise difference of two nearly equal
            numbers depends on their last digits, which differ between the browser build and the native build.{' '}
            <ProvenanceNote doc={doc} file="btcs_vs_analytical.json" />
          </>
        }
      >
        <Plot
          figure={diff}
          testId="verify-bc-difference-plot"
          height={280}
          label={`Absolute difference between ${METHOD_1} and ${METHOD_2} against time, logarithmic axis, precomputed by the native program.`}
          exportInfo={{ name: `strato-verify-difference-${doc.commit.slice(0, 7)}`, source: exportSource(doc, 'btcs_vs_analytical.json') }}
        />
      </Fig>
    </div>
  );
}

/** The independent check: observed orders in time and in space, and the differences from the reference. */
export function ConvergencePanel() {
  const { doc, error } = usePrecomputed<CheckDoc>('btcs_independent_check.json');
  const theme = usePlotTheme();
  const fig = useMemo(() => (doc ? checkFigure(doc, theme) : null), [doc, theme]);
  if (!doc || !fig) return <Loading error={error} what="independent check" />;
  const d = doc.data;
  const h = d.column_headers;
  const columns: { key: 't_s' | 'f_btcs' | 'f_reference' | 'abs_difference' | 'time_order' | 'space_order'; testId?: string }[] = [
    { key: 't_s' },
    { key: 'f_btcs' },
    { key: 'f_reference' },
    { key: 'abs_difference' },
    { key: 'time_order', testId: 'orders-temporal' },
    { key: 'space_order', testId: 'orders-spatial' },
  ];
  return (
    <>
      <TableBox label="Independent check of the release fraction">
        <table className="table" data-testid="observed-orders">
          <caption className="table__caption">
            <strong>Table 1.</strong> The independent check of the regression suite for the verified case: the release fraction computed by {METHOD_1}
            against a reference that contains no BTCS code, and the orders of accuracy observed from refined runs. Expected: about 1 in time and
            about 2 in space (<Badge label="PRECOMPUTED" />, copied from <code>tests/golden/btcs/s5_n40_dt200/independent_check.csv</code>). The
            file calls itself a regression reference, not truth.
          </caption>
          <thead>
            <tr>
              {columns.map((c) => (
                <th key={c.key} scope="col" className="num-col">
                  {h[c.key]}
                </th>
              ))}
              <th scope="col">{h.passed}</th>
            </tr>
          </thead>
          <tbody>
            {d.t_s.map((t, i) => (
              <tr key={t}>
                {columns.map((c) => {
                  const v = d[c.key][i];
                  return (
                    <td key={c.key} className="num-col num" data-testid={c.testId ? `${c.testId}-${i}` : undefined} data-value={String(v)}>
                      {v === undefined ? '–' : <Sci value={v} digits={c.key.endsWith('order') ? 4 : 5} />}
                    </td>
                  );
                })}
                <td data-value={String(d.passed[i])}>{d.passed[i] ? 'yes' : 'no'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </TableBox>
      <div className="plot-grid plot-grid--two">
        <Fig
          number={3}
          testId="verify-check"
          label="PRECOMPUTED"
          caption={
            <>
              Absolute difference of the release fraction computed by {METHOD_1} from the independent reference, and of the reference from the
              numerical limit estimated by refining the time step and the grid, at the four checkpoint times, on log-log axes. The reference is{' '}
              {METHOD_2} (the multilayer Sturm-Liouville series). The check passes when the reference lies within the stated convention of the
              estimated limit; that convention (5 percent of the difference plus the accuracy floor of the reference) is a labelled convention, not a
              derived bound. <ProvenanceNote doc={doc} file="btcs_independent_check.json" />
            </>
          }
        >
          <Plot
            figure={fig}
            testId="verify-check-plot"
            height={300}
            label="Absolute differences of the BTCS release fraction and of the estimated numerical limit from the independent reference at four checkpoint times, log-log."
            exportInfo={{ name: `strato-verify-check-${doc.commit.slice(0, 7)}`, source: exportSource(doc, 'btcs_independent_check.json') }}
          />
        </Fig>
      </div>
    </>
  );
}

export function BalancePanelVerify() {
  const { doc, error } = usePrecomputed<RunDoc>('btcs_run.json');
  const live = useStrato((s) => s.run);
  const theme = usePlotTheme();
  const fig = useMemo(() => (doc ? residualFigure(doc, theme) : null), [doc, theme]);
  if (!doc || !fig) return <Loading error={error} what="native run" />;
  const s = doc.data.summary;
  return (
    <Fig
      number={4}
      testId="verify-balance"
      label="PRECOMPUTED"
      caption={
        <>
          Mass-balance residual of the native run at every output time with its derived a-posteriori rounding bound, as the program wrote
          them; the residual is zero in exact arithmetic, so what remains is rounding. Largest relative residual as printed:{' '}
          {s.worst_balance_residual_relative ?? '–'}, against a final relative bound of {s.final_balance_residual_bound_relative ?? '–'}.{' '}
          {live ? (
            <span data-testid="verify-live-residual">
              The run now on Explore reports a largest relative residual of{' '}
              <span data-value={String(live.balance.max_abs_residual_relative)}>
                <Sci value={live.balance.max_abs_residual_relative} digits={3} />
              </span>{' '}
              (<Badge label={live.label} />, from the WebAssembly solver build <code>{live.engine.commit}</code>).
            </span>
          ) : null}{' '}
          <ProvenanceNote doc={doc} file="btcs_run.json" />
        </>
      }
    >
      <Plot
        figure={fig}
        testId="verify-balance-plot"
        height={260}
        label="Mass-balance residual of the native run and its derived rounding bound against time."
        exportInfo={{ name: `strato-verify-balance-${doc.commit.slice(0, 7)}`, source: exportSource(doc, 'btcs_run.json') }}
      />
    </Fig>
  );
}
