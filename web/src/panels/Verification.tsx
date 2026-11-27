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
import { hrefDocumentation } from '../route';
import { useRunIsCurrent } from '../equation/useCapability';
import { useStrato } from '../state/store';
import { Badge, Sci } from './Badge';
import { TableBox } from '../sections/common';

// The verification panels of the Verify section (prompt/DESIGN_SPECIFICATION.md Section 8.3): Method A
// against Method B, the independent check with its observed orders in time and in space, and the
// balance residual with its derived rounding bound, from the precomputed output of the native program
// (label PRECOMPUTED), and the live run's own residual (COMPUTED v2.0). The pointwise difference between
// the two methods is shown only here, from the native file (coordinator decision of 2026-11-25). The
// provenance header of each file, the methodology and the numbers behind the captions are in Scientific
// Documentation (documentation/verify.tsx). Numerical verification only.

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
  file,
  anchor,
}: {
  number: number;
  testId: string;
  children: ReactNode;
  caption: ReactNode;
  file: string;
  anchor: string;
}) {
  return (
    <figure className="figure plotfig" data-testid={testId}>
      <div className="figure__head">
        <Badge label="PRECOMPUTED" />
        <a className="figure__kind figure__kind--fit" href={precomputedUrl(file)}>
          Data file
        </a>
        <a className="figure__kind figure__kind--fit" href={hrefDocumentation(anchor)}>
          Method
        </a>
      </div>
      {children}
      <figcaption className="figure__caption">
        <strong>Figure {number}.</strong> {caption}
      </figcaption>
    </figure>
  );
}

/** Breaks a legend name into lines of at most `width` characters at word boundaries (Plotly reads <br>). */
function wrapLegendName(name: string | undefined, width = 44): string {
  if (!name) return '';
  const lines: string[] = [];
  let line = '';
  for (const word of name.split(' ')) {
    if (line !== '' && line.length + 1 + word.length > width) {
      lines.push(line);
      line = word;
    } else {
      line = line === '' ? word : `${line} ${word}`;
    }
  }
  lines.push(line);
  return lines.join('<br>');
}

function exportSource(doc: PrecomputedHeader, file: string): string {
  return `${file} (${doc.label}), native ${doc.solver}, solver version ${doc.strato_version} at commit ${doc.commit.slice(0, 7)}, ${doc.command}; synthetic five-layer verification case, not physical data.`;
}

/** Method A against Method B: the two release fractions and their pointwise difference. */
export function MethodComparisonPanel() {
  const { doc, error } = usePrecomputed<ComparisonDoc>('btcs_vs_analytical.json');
  const theme = usePlotTheme();
  const floor = doc ? doc.data.analytical.accuracy_floor : null;
  const release = useMemo(() => (doc ? methodReleaseFigure(doc, theme) : null), [doc, theme]);
  const diff = useMemo(() => (doc ? methodDifferenceFigure(doc, floor, theme) : null), [doc, floor, theme]);
  if (!doc || !release || !diff) return <Loading error={error} what={`comparison of ${METHOD_1} and ${METHOD_2}`} />;
  return (
    <div className="plot-grid plot-grid--two">
      <Fig
        number={1}
        testId="verify-bc-release"
        file="btcs_vs_analytical.json"
        anchor="verify-methods"
        caption={
          <>
            Release fraction from {METHOD_1} (red) and {METHOD_2} (blue).
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
        file="btcs_vs_analytical.json"
        anchor="verify-difference"
        caption={<>Absolute difference of the two release fractions; the grey line is the accuracy floor of {METHOD_2}.</>}
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

/** Figure 3: the differences from the independent reference. */
export function CheckFigurePanel() {
  const { doc, error } = usePrecomputed<CheckDoc>('btcs_independent_check.json');
  const theme = usePlotTheme();
  const fig = useMemo(() => {
    if (!doc) return null;
    const f = checkFigure(doc, theme);
    // Long legend names are broken into lines so that no entry is cut off by the plot edge.
    return { ...f, data: f.data.map((t) => ({ ...t, name: wrapLegendName(t.name) })) };
  }, [doc, theme]);
  if (!doc || !fig) return <Loading error={error} what="independent check" />;
  return (
    <Fig
      number={3}
      testId="verify-check"
      file="btcs_independent_check.json"
      anchor="verify-check"
      caption={<>Differences of {METHOD_1} and of the estimated numerical limit from the independent reference (log-log).</>}
    >
      <Plot
        figure={fig}
        testId="verify-check-plot"
        height={300}
        label="Absolute differences of the BTCS release fraction and of the estimated numerical limit from the independent reference at four checkpoint times, log-log."
        exportInfo={{ name: `strato-verify-check-${doc.commit.slice(0, 7)}`, source: exportSource(doc, 'btcs_independent_check.json') }}
      />
    </Fig>
  );
}

/** Table 1: the independent check with the observed orders in time and in space. */
export function ObservedOrdersTable() {
  const { doc, error } = usePrecomputed<CheckDoc>('btcs_independent_check.json');
  if (!doc) return <Loading error={error} what="independent check" />;
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
    <TableBox label="Independent check of the release fraction">
      <table className="table check-table" data-testid="observed-orders">
        <caption className="table__caption">
          <Badge label="PRECOMPUTED" /> <strong>Table 1.</strong> Independent check: release fraction against a reference without BTCS
          code, and observed orders of accuracy (expected about 1 in time, about 2 in space).
        </caption>
        <thead>
          <tr>
            {columns.map((c) => (
              <th key={c.key} scope="col" className="num-col">
                {c.key === 'f_btcs' ? `Release fraction by ${METHOD_1} (dimensionless)` : h[c.key]}
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
  );
}

/** Figure 4: the mass-balance residual of the native run, with the residual of the live run beside it. */
export function BalancePanelVerify() {
  const { doc, error } = usePrecomputed<RunDoc>('btcs_run.json');
  const stored = useStrato((s) => s.run);
  const current = useRunIsCurrent();
  const live = current ? stored : null;
  const theme = usePlotTheme();
  const fig = useMemo(() => (doc ? residualFigure(doc, theme) : null), [doc, theme]);
  if (!doc || !fig) return <Loading error={error} what="native run" />;
  return (
    <Fig
      number={4}
      testId="verify-balance"
      file="btcs_run.json"
      anchor="verify-balance"
      caption={<>Mass-balance residual of the native run and its derived rounding bound.</>}
    >
      <Plot
        figure={fig}
        testId="verify-balance-plot"
        height={260}
        label="Mass-balance residual of the native run and its derived rounding bound against time."
        exportInfo={{ name: `strato-verify-balance-${doc.commit.slice(0, 7)}`, source: exportSource(doc, 'btcs_run.json') }}
      />
      {live ? (
        <p className="figure__kind" data-testid="verify-live-residual">
          Live run on Explore, largest relative residual:{' '}
          <span data-value={String(live.balance.max_abs_residual_relative)}>
            <Sci value={live.balance.max_abs_residual_relative} digits={3} />
          </span>{' '}
          <Badge label={live.label} />
        </p>
      ) : null}
    </Fig>
  );
}
