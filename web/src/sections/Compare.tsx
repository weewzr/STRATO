import { plain } from '../format';
import { hrefFor } from '../route';
import { configDifferences, useStrato } from '../state/store';
import { Note, SectionHeading, TableBox } from './common';

export function Compare() {
  const config = useStrato((s) => s.config);
  const baseline = useStrato((s) => s.baselineConfig);
  const diffs = configDifferences(baseline, config);
  return (
    <article className="section" data-section="compare">
      <SectionHeading lede="Compare a modified parameter set with a baseline. Both runs will use the verified v1.0 solver, so the comparison shows the effect of the inputs, not of new physics.">
        Compare with the baseline
      </SectionHeading>

      <h2>Inputs that differ from the baseline</h2>
      {diffs.length === 0 ? (
        <p data-testid="no-diffs">
          The current inputs equal the baseline. Change a value in the controls on <a href={hrefFor('explore')}>Explore</a>, or use{' '}
          <em>Set current as baseline</em> there to keep the present values as the reference.
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
      <Note>
        <p>
          The baseline curve, the modified curve and their difference need the solver (web phases 3 and 5). The difference curve will be the
          series that the Rust wrapper returns; the browser does not subtract one curve from the other.
        </p>
      </Note>
    </article>
  );
}
