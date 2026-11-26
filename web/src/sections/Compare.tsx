import { useEffect, useMemo } from 'react';
import { BUILD } from '../build';
import { findCase } from '../data/case';
import { plain } from '../format';
import { METHOD_1 } from '../methods';
import { Badge, Sci } from '../panels/Badge';
import { compareFigure } from '../plots/figures';
import { Plot } from '../plots/Plot';
import { usePlotTheme } from '../plots/theme';
import { hrefFor } from '../route';
import { comparisonKey, configDifferences, useStrato } from '../state/store';
import { Note, SectionHeading, TableBox } from './common';

// The Compare section (web phase 5; prompt/DESIGN_SPECIFICATION.md Section 8.1): the baseline and the
// current release curves from compare, and the optional difference curve that compare computes
// in Rust (current minus baseline on the shared output times). The browser subtracts nothing. The
// comparison runs only while this section is open.

function ComparisonFigure() {
  const comparison = useStrato((s) => s.comparison);
  const config = useStrato((s) => s.config);
  const baseline = useStrato((s) => s.baselineConfig);
  const showBaseline = useStrato((s) => s.view.showComparison);
  const showDifference = useStrato((s) => s.view.showDifference);
  const setView = useStrato((s) => s.setView);
  const caseId = useStrato((s) => s.caseId);
  const theme = usePlotTheme();
  const reply = comparison.reply;
  const stale = reply !== null && (comparison.status === 'running' || comparison.key !== comparisonKey(baseline, config));
  const figure = useMemo(() => (reply ? compareFigure(reply, { showBaseline, showDifference }, theme) : null), [reply, showBaseline, showDifference, theme]);
  const caseFile = findCase(caseId)?.provenance.input_file ?? '';

  return (
    <figure className={`figure plotfig${stale ? ' is-stale' : ''}`} data-testid="compare-figure" data-state={reply ? 'ok' : comparison.status}>
      <div className="figure__head">
        {reply ? <Badge label={reply.label} /> : <span className="readout__nolabel">Label: none</span>}
        <span className="figure__kind">
          Release fraction (dimensionless) of the baseline and of the current inputs
          {stale ? <span className="sim-note"> updating for the changed inputs</span> : null}
        </span>
      </div>
      <div className="switch-row" role="group" aria-label="Comparison layers">
        <label className="plot-option">
          <input type="checkbox" checked={showBaseline} onChange={(e) => setView({ showComparison: e.target.checked })} data-testid="toggle-comparison" />
          <span>Baseline curve</span>
        </label>
        <label className="plot-option">
          <input type="checkbox" checked={showDifference} onChange={(e) => setView({ showDifference: e.target.checked })} data-testid="toggle-difference" />
          <span>Difference curve, current minus baseline (right axis)</span>
        </label>
      </div>
      {reply && figure ? (
        <Plot
          figure={figure}
          testId="compare-plot"
          height={320}
          label={`Release fraction against time for the baseline${showBaseline ? ' (grey line)' : ' (hidden)'} and the current inputs (blue line)${showDifference ? ', with the difference current minus baseline on the right axis (green line), as compare returned it' : ''}.`}
          exportInfo={{
            name: `strato-comparison-${reply.engine.commit}`,
            source: `STRATO ${METHOD_1}, solver version ${reply.solver.version}, Rust solver in WebAssembly (build ${reply.engine.commit}), compare; ${reply.label}; synthetic five-layer verification case, not physical data; page build ${BUILD.commit}.`,
          }}
        />
      ) : comparison.status === 'rejected' || comparison.status === 'error' ? (
        <div className="sim-alert" role="alert" data-testid="compare-problem">
          <p className="sim-alert__title">{comparison.status === 'rejected' ? 'The solver rejected the comparison' : 'The comparison did not finish'}</p>
          <ul className="sim-alert__list">
            {(comparison.problem ?? []).map((m) => (
              <li key={m}>{m}</li>
            ))}
          </ul>
        </div>
      ) : (
        <p className="plotfig__empty" role="status">
          {comparison.status === 'running' ? 'Running the comparison…' : 'Waiting for the solver.'}
        </p>
      )}
      {reply ? (
        <p className="figure__kind" data-testid="compare-summary">
          Final release fraction: baseline{' '}
          <span data-testid="compare-final-baseline" data-value={String(reply.baseline.diagnostics.final_release_fraction)}>
            {reply.baseline.diagnostics.final_release_fraction.toPrecision(6)}
          </span>
          , current{' '}
          <span data-testid="compare-final-current" data-value={String(reply.modified.diagnostics.final_release_fraction)}>
            {reply.modified.diagnostics.final_release_fraction.toPrecision(6)}
          </span>
          ; difference at the end of the simulation{' '}
          {(() => {
            const d = reply.difference.release_fraction[reply.difference.release_fraction.length - 1];
            return d === undefined ? '–' : (
              <span data-testid="compare-final-difference" data-value={String(d)}>
                <Sci value={d} digits={4} />
              </span>
            );
          })()}{' '}
          (from compare). Solver wall time in this browser (ms): {comparison.elapsedMs !== null ? comparison.elapsedMs.toFixed(1) : '–'}, not a result.
        </p>
      ) : null}
      <figcaption className="figure__caption">
        <strong>Figure 1.</strong> Fraction of the initial inventory released, for the baseline (grey line) and the current inputs (blue line), both
        run by {METHOD_1}; until generation, decay and temperature dependence are solved, a comparison shows the effect of changed inputs, not of new physics.
        The green difference curve is the series that <code>compare</code> returns: the current minus the baseline release fraction,
        computed in Rust at each output time both runs share. Runs with different output times are not compared, because the difference is
        never interpolated. The parameters that differ are listed above.
        {reply ? (
          <span className="figure__source">
            Source: {METHOD_1}, solver version {reply.solver.version}, Rust solver in WebAssembly, build <code>{reply.engine.commit}</code>, both runs
            starting from <code>{caseFile}</code> (synthetic verification case, not physical data). Page build <code>{BUILD.commit}</code>.
            The CSV of the comparison, written by the solver in Rust with its provenance header, is under <a href={hrefFor('data')}>Data</a>.
          </span>
        ) : null}
      </figcaption>
    </figure>
  );
}

export function Compare() {
  const config = useStrato((s) => s.config);
  const baseline = useStrato((s) => s.baselineConfig);
  const ready = useStrato((s) => s.solver.state === 'ready');
  const comparisonKeyNow = useStrato((s) => s.comparison.key);
  const requestCompare = useStrato((s) => s.requestCompare);
  const diffs = configDifferences(baseline, config);
  const key = comparisonKey(baseline, config);

  // Run the comparison while this section is open, once per (baseline, current) pair.
  useEffect(() => {
    if (ready && comparisonKeyNow !== key) requestCompare();
  }, [ready, key, comparisonKeyNow, requestCompare]);

  return (
    <article className="section" data-section="compare">
      <SectionHeading lede="Compare the current inputs with a baseline. Both runs use the verified solver of Method A: BTCS, so the comparison shows the effect of the inputs, not of new physics.">
        Compare with the baseline
      </SectionHeading>

      <h2>Inputs that differ from the baseline</h2>
      {diffs.length === 0 ? (
        <p data-testid="no-diffs">
          The current inputs equal the baseline, so the two curves coincide and the difference returned by the solver is zero. Change a value
          in the controls on <a href={hrefFor('explore')}>Explore</a>, or use <em>Set current as baseline</em> there to keep the present values
          as the reference.
        </p>
      ) : (
        <TableBox label="Inputs that differ from the baseline">
          <table className="table" data-testid="diff-table">
            <caption className="visually-hidden">Inputs that differ from the baseline</caption>
            <thead>
              <tr>
                <th scope="col">Input</th>
                <th scope="col" className="num-col">
                  Baseline
                </th>
                <th scope="col" className="num-col">
                  Current
                </th>
                <th scope="col">Unit</th>
              </tr>
            </thead>
            <tbody>
              {diffs.map((d) => (
                <tr key={d.path}>
                  <th scope="row">{d.label}</th>
                  <td className="num-col num">{typeof d.baseline === 'number' ? plain(d.baseline) : d.baseline}</td>
                  <td className="num-col num">{typeof d.current === 'number' ? plain(d.current) : d.current}</td>
                  <td>{d.unit}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </TableBox>
      )}

      <h2>Release curves</h2>
      <ComparisonFigure />
      <Note>
        <p>
          The comparison and the validation layers are independent: the switches above change only this figure, and the measurements and
          predictions of the <a href={hrefFor('validate')}>Validate</a> section are not drawn here.
        </p>
      </Note>
    </article>
  );
}
