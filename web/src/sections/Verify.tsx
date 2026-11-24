import { repoFile } from '../build';
import { SectionHeading, TableBox } from './common';

const CHECKS: { check: string; shows: string; evidence: { path: string; text: string }[] }[] = [
  {
    check: 'Finite volume against the series solution',
    shows: 'Code B (finite volume) and Code C (Sturm-Liouville series) agree on the release fraction, within the discretisation error of Code B.',
    evidence: [
      { path: 'tests/finite_volume_vs_analytical.rs', text: 'tests/finite_volume_vs_analytical.rs' },
      { path: 'tests/golden/v1.0/comparison_fv_vs_analytical.csv', text: 'comparison_fv_vs_analytical.csv' },
    ],
  },
  {
    check: 'Convergence orders',
    shows: 'Spatial refinement against the exact series, and temporal refinement of Backward Euler. A page that shows a temporal order states its reference.',
    evidence: [
      { path: 'examples/verify.rs', text: 'examples/verify.rs' },
      { path: 'tests/golden/v1.0/convergence_study.csv', text: 'convergence_study.csv' },
    ],
  },
  {
    check: 'Inventory balance',
    shows: 'Inventory plus cumulative release equals the start inventory at every step, to round-off.',
    evidence: [{ path: 'tests/golden/v1.0/verify_stdout.txt', text: 'verify_stdout.txt' }],
  },
  {
    check: 'Golden regression',
    shows: 'The time series, final profile and analytical series equal the delivered v1.0 outputs to a relative tolerance of 10⁻¹².',
    evidence: [{ path: 'tests/phase1_regression.rs', text: 'tests/phase1_regression.rs' }],
  },
  {
    check: 'Native and browser parity',
    shows: 'Required from web phase 3: the WebAssembly build gives the same numbers as the native build, to a relative tolerance of 10⁻¹² per value. Not yet run.',
    evidence: [],
  },
];

export function Verify() {
  return (
    <article className="section" data-section="verify">
      <SectionHeading lede="Verification shows that STRATO solves its own equations correctly. It does not show that the equations or the inputs describe a real particle.">
        Verification of the v1.0 solver
      </SectionHeading>
      <p>
        The checks below run in the repository's continuous integration and can be repeated with <code>cargo test --all-targets</code> and{' '}
        <code>cargo run --release --example verify</code>. Their numbers are shown here from web phase 3 onwards, read from output that the
        native STRATO binary produced (label PRECOMPUTED, with commit, input hash and command) and, for parity, from the live WebAssembly
        run. No number is shown before that file exists.
      </p>
      <TableBox label="Verification checks">
        <table className="table">
          <caption className="visually-hidden">Verification checks of STRATO v1.0</caption>
          <thead>
            <tr>
              <th scope="col">Check</th>
              <th scope="col">What it shows</th>
              <th scope="col">Evidence in the repository</th>
            </tr>
          </thead>
          <tbody>
            {CHECKS.map((c) => (
              <tr key={c.check}>
                <th scope="row">{c.check}</th>
                <td>{c.shows}</td>
                <td>
                  {c.evidence.length === 0 ? (
                    <span className="muted">added in web phase 3</span>
                  ) : (
                    c.evidence.map((e, i) => (
                      <span key={e.path}>
                        {i > 0 ? ', ' : ''}
                        <a href={repoFile(e.path)}>
                          <code>{e.text}</code>
                        </a>
                      </span>
                    ))
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </TableBox>
    </article>
  );
}
