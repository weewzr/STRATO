import { repoFile } from '../build';
import { METHOD_1, METHOD_2 } from '../methods';
import { BalancePanelVerify, ConvergencePanel, MethodComparisonPanel } from '../panels/Verification';
import { SolverStatus } from '../panels/SimResults';
import { SectionHeading, TableBox } from './common';

const CHECKS: { check: string; shows: string; evidence: { path: string; text: string }[] }[] = [
  {
    check: `${METHOD_1} against ${METHOD_2}`,
    shows: `The finite-difference solution and the multilayer Sturm-Liouville series agree on the release fraction, within the discretisation error of ${METHOD_1}, which the independent check below measures.`,
    evidence: [
      { path: 'tests/btcs_verification.rs', text: 'tests/btcs_verification.rs' },
      { path: 'tests/golden/btcs/README.md', text: 'tests/golden/btcs/' },
    ],
  },
  {
    check: 'Observed orders of accuracy',
    shows:
      'Refinement of the time step and of the grid: the observed orders round to 1 in time and to 2 in space, and the numerical limit estimated from the refined runs is compared with the independent reference under a stated convention.',
    evidence: [
      { path: 'tests/btcs_regression.rs', text: 'tests/btcs_regression.rs' },
      { path: 'tests/golden/btcs/s5_n40_dt200/independent_check.csv', text: 'independent_check.csv' },
    ],
  },
  {
    check: 'Inventory balance and positivity',
    shows:
      'The remaining inventory plus the cumulative release equals the initial inventory in exact arithmetic; in floating point the residual stays within a derived a-posteriori rounding bound, and the concentration is never negative.',
    evidence: [
      { path: 'tests/btcs_verification.rs', text: 'tests/btcs_verification.rs' },
      { path: 'tests/btcs_properties.rs', text: 'tests/btcs_properties.rs' },
    ],
  },
  {
    check: 'Regression files',
    shows: 'The time series and the final profile of three synthetic cases equal the stored regression references to a relative tolerance of 10⁻¹²; the files are regression references, not truth.',
    evidence: [{ path: 'tests/btcs_regression.rs', text: 'tests/btcs_regression.rs' }],
  },
  {
    check: 'Wrapper against the library',
    shows:
      'The WebAssembly wrapper returns the library solver’s numbers: its time series and profiles are bit-for-bit those of the library run on the same machine; invalid and placeholder inputs are rejected with a message, and the data export carries its provenance header.',
    evidence: [{ path: 'crates/strato-wasm/tests/btcs_api.rs', text: 'crates/strato-wasm/tests/btcs_api.rs' }],
  },
  {
    check: 'Difference curve of a comparison',
    shows:
      'The difference curve on Compare is the series that compare returns, the current minus the baseline computed in Rust on the shared output times; a wrapper test checks it point by point. Runs with different output times are rejected, never interpolated.',
    evidence: [
      { path: 'crates/strato-wasm/tests/btcs_api.rs', text: 'crates/strato-wasm/tests/btcs_api.rs' },
      { path: 'web/e2e/plots.spec.ts', text: 'web/e2e/plots.spec.ts' },
    ],
  },
  {
    check: 'Native and browser parity',
    shows:
      'The WebAssembly build in the browser gives the same numbers as the native wrapper, to a relative tolerance of 10⁻¹² per value with identical array lengths; the values of the independent series carry its own accuracy floor as an additional allowance. Run by the browser tests in continuous integration and by the self-test of this page (?selftest=1).',
    evidence: [
      { path: 'web/e2e/solver.spec.ts', text: 'web/e2e/solver.spec.ts' },
      { path: 'web/public/data/wrapper_native_synthetic.json', text: 'wrapper_native_synthetic.json' },
    ],
  },
];

export function Verify() {
  return (
    <article className="section" data-section="verify">
      <SectionHeading lede="Verification shows that STRATO solves its own equations correctly. It does not show that the equations or the inputs describe a real particle.">
        Verification of the BTCS solver
      </SectionHeading>
      <SolverStatus />
      <p>
        The checks below run in the repository's continuous integration and can be repeated with <code>cargo test --all-targets</code>. The
        output of the native STRATO programs for the synthetic case is published with this site as precomputed files (label PRECOMPUTED, with
        commit, input hash, solver version and command; see Data); the figures below plot them as they are. The live simulation on the Explore
        page reports its own balance residual and its largest and final difference from {METHOD_2} (label COMPUTED v2.0). All of this is
        verification on the synthetic case; none of it compares the model with a measurement.
      </p>
      <h2>
        {METHOD_1} against {METHOD_2}
      </h2>
      <MethodComparisonPanel />
      <h2>Observed orders and the independent check</h2>
      <ConvergencePanel />
      <h2>Inventory balance</h2>
      <BalancePanelVerify />
      <h2>Checks and their evidence</h2>
      <TableBox label="Verification checks">
        <table className="table">
          <caption className="visually-hidden">Verification checks of the BTCS solver</caption>
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
                  {c.evidence.map((e, i) => (
                    <span key={e.path}>
                      {i > 0 ? ', ' : ''}
                      <a href={repoFile(e.path)}>
                        <code>{e.text}</code>
                      </a>
                    </span>
                  ))}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </TableBox>
    </article>
  );
}
