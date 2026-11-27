import { repoFile } from '../build';
import { BTCS_CAPABILITY } from '../equation/availability';
import { METHOD_1, METHOD_2 } from '../methods';
import { Badge } from '../panels/Badge';
import type { CheckDoc, ComparisonDoc, RunDoc } from '../plots/figures';
import { precomputedUrl, usePrecomputed } from '../plots/precomputed';
import { statusOf, useCatalogue } from '../research/catalogue';
import { StatusChip } from '../research/StatusChip';
import { TableBox } from '../sections/common';
import { hrefDocumentation } from '../route';
import { useStrato } from '../state/store';
import type { DocContribution } from './types';

// Scientific material that left the Verify page, moved with its meaning unchanged. The numbers of the
// accuracy floor, the balance residual and the provenance of each precomputed file are read from the
// files, never typed.

/** The scope of the solver, as the WebAssembly wrapper reports it. */
function SolverScope() {
  const state = useStrato((s) => s.solver.state);
  const build = useStrato((s) => s.solver.build);
  const scope = build?.btcs.solver.scope;
  return (
    <p data-testid="documentation-scope">
      Scope of {METHOD_1}: {scope ?? (state === 'failed' ? 'the solver is not available' : 'loading')}.
    </p>
  );
}

/** Verification status of Method A in the research catalogue, with the qualification that goes with it. */
function VerificationStatus() {
  const catalogue = useCatalogue();
  const status = statusOf(catalogue, BTCS_CAPABILITY);
  return (
    <p>
      {METHOD_1}: results labelled <Badge label="COMPUTED v2.0" />. Verification status in the research catalogue:{' '}
      {status ? <StatusChip status={status} mechanism={BTCS_CAPABILITY} compact /> : 'unknown'}, numerical verification of the stated model,
      not experimental validation.
    </p>
  );
}

function AccuracyFloor() {
  const { doc } = usePrecomputed<ComparisonDoc>('btcs_vs_analytical.json');
  if (!doc) return null;
  const a = doc.data.analytical;
  return (
    <p data-testid="documentation-accuracy-floor">
      The accuracy floor of {METHOD_2} is {a.accuracy_floor}, valid from the time {a.valid_from_s} s, with {a.modes} eigenmodes; the largest
      difference between the two methods is {a.max_abs_difference} and the final difference is {a.final_abs_difference}.
    </p>
  );
}

function BalanceNumbers() {
  const { doc } = usePrecomputed<RunDoc>('btcs_run.json');
  if (!doc) return null;
  const s = doc.data.summary;
  return (
    <TableBox label="Mass-balance residual of the native run">
      <table className="table" data-testid="documentation-balance">
        <caption className="visually-hidden">Mass-balance residual of the native run</caption>
        <tbody>
          <tr>
            <th scope="row">Largest relative residual as printed</th>
            <td className="num">{s.worst_balance_residual_relative ?? '–'}</td>
          </tr>
          <tr>
            <th scope="row">Final relative bound</th>
            <td className="num">{s.final_balance_residual_bound_relative ?? '–'}</td>
          </tr>
        </tbody>
      </table>
    </TableBox>
  );
}

const CHECKS: { check: string; shows: string; evidence: { path: string; text: string }[] }[] = [
  {
    check: `${METHOD_1} against ${METHOD_2}`,
    shows: `The finite-difference solution and the multilayer Sturm-Liouville series agree on the release fraction, within the discretisation error of ${METHOD_1}, which the independent check measures.`,
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
      'A wrapper test checks the difference curve of Compare point by point. What the curve is, and the rule on output times, are stated once, in the entry "Difference curve of a comparison".',
    evidence: [
      { path: 'crates/strato-wasm/tests/btcs_api.rs', text: 'crates/strato-wasm/tests/btcs_api.rs' },
      { path: 'web/e2e/plots.spec.ts', text: 'web/e2e/plots.spec.ts' },
    ],
  },
  {
    check: 'Native and browser parity',
    shows:
      'The WebAssembly build in the browser gives the same numbers as the native wrapper, to a relative tolerance of 10⁻¹² per value with identical array lengths; the values of the independent series carry its own accuracy floor as an additional allowance. Run by the browser tests in continuous integration and by the self-test of the site (?selftest=1).',
    evidence: [
      { path: 'web/e2e/solver.spec.ts', text: 'web/e2e/solver.spec.ts' },
      { path: 'web/public/data/wrapper_native_synthetic.json', text: 'wrapper_native_synthetic.json' },
    ],
  },
];

function ChecksTable() {
  return (
    <TableBox label="Verification checks">
      <table className="table" data-testid="documentation-checks">
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
  );
}

const FILES = ['btcs_vs_analytical.json', 'btcs_independent_check.json', 'btcs_run.json'] as const;

/** The provenance header of a precomputed file, as it is in the file. */
function ProvenanceRow({ file }: { file: (typeof FILES)[number] }) {
  const { doc, error } = usePrecomputed<ComparisonDoc | CheckDoc | RunDoc>(file);
  if (!doc) {
    return (
      <tr data-testid={`prov-${file}`}>
        <th scope="row">
          <code>{file}</code>
        </th>
        <td colSpan={6}>{error ? `could not be loaded (${error})` : 'loading'}</td>
      </tr>
    );
  }
  return (
    <tr data-testid={`prov-${file}`}>
      <th scope="row">
        <a href={precomputedUrl(file)}>
          <code>{file}</code>
        </a>{' '}
        ({doc.label})
      </th>
      <td>
        {doc.converter}, from the native program {doc.solver}
      </td>
      <td>{doc.strato_version}</td>
      <td>
        <code>{doc.commit.slice(0, 7)}</code>
        {doc.input_file ? (
          <>
            {' '}
            on <code>{doc.input_file}</code>
            {doc.input_file_sha256 ? (
              <>
                {' '}
                (SHA-256 <code className="hash">{doc.input_file_sha256.slice(0, 12)}</code>)
              </>
            ) : null}
          </>
        ) : null}
      </td>
      <td>
        <code>{doc.command}</code>
      </td>
      <td>{doc.rustc ?? '–'}</td>
      <td>{doc.created_utc}</td>
    </tr>
  );
}

function PrecomputedProvenance() {
  return (
    <TableBox label="Provenance of the precomputed verification files">
      <table className="table" data-testid="documentation-precomputed">
        <caption className="visually-hidden">Provenance headers of the precomputed verification files</caption>
        <thead>
          <tr>
            <th scope="col">File</th>
            <th scope="col">Written by</th>
            <th scope="col">Solver version</th>
            <th scope="col">Commit and input</th>
            <th scope="col">Command</th>
            <th scope="col">Toolchain</th>
            <th scope="col">Written (UTC)</th>
          </tr>
        </thead>
        <tbody>
          {FILES.map((f) => (
            <ProvenanceRow key={f} file={f} />
          ))}
        </tbody>
      </table>
    </TableBox>
  );
}

export const verify: DocContribution = {
  assumptions: [{ id: 'verify-scope', title: 'Scope of the verified solver', body: <SolverScope /> }],
  verification: [
    {
      id: 'verify-definition',
      title: 'What verification shows',
      body: (
        <>
          <p>
            Verification shows that STRATO solves its own equations correctly. It does not show that the equations or the inputs describe a
            real particle.
          </p>
          <VerificationStatus />
        </>
      ),
    },
    {
      id: 'verify-evidence',
      title: 'Where the verification results come from',
      body: (
        <>
          <p>
            The checks run in the repository’s continuous integration. The output of the native STRATO programs for the synthetic case is
            published with this site as precomputed files (label PRECOMPUTED, with commit, input hash, solver version and command; see{' '}
            <a href={hrefDocumentation('evidence-downloads')}>Data downloads</a>); the figures on <a href={hrefDocumentation('evidence-verification')}>the verification evidence</a> plot them as they are. The live
            simulation on the Explore page reports its own balance residual and its largest and final difference from {METHOD_2} (label
            COMPUTED v2.0). All of this is verification on the synthetic case; none of it compares the model with a measurement.
          </p>
          <p>
            The provenance of each precomputed file is in the table under{' '}
            <a href={hrefDocumentation('verify-files')}>Technical implementation and build</a>.
          </p>
        </>
      ),
    },
    {
      id: 'verify-methods',
      title: `${METHOD_1} against ${METHOD_2}`,
      body: (
        <p>
          The release fractions of {METHOD_1} (blue line) and {METHOD_2} (red line), the multilayer Sturm-Liouville series, are plotted as
          the native program wrote them.
        </p>
      ),
    },
    {
      id: 'verify-difference',
      title: 'Difference between the two methods',
      body: (
        <>
          <p>
            The absolute difference of the release fractions of {METHOD_1} and {METHOD_2} of the verified case at every output time, as the
            Rust tool computed it from the native reply, is a discretisation error of {METHOD_1}, not a defect. The thin grey horizontal line
            is the accuracy floor of {METHOD_2}.
          </p>
          <AccuracyFloor />
          <p>
            This plot is shown only from the native file: for inputs changed on Explore, the page shows the two curves and the largest and
            final difference that the solver returns, because the pointwise difference of two nearly equal numbers depends on their last
            digits, which differ between the browser build and the native build.
          </p>
        </>
      ),
    },
    {
      id: 'verify-check',
      title: 'Independent check and observed orders',
      body: (
        <>
          <p>
            The independent check of the regression suite for the verified case: the release fraction computed by {METHOD_1} against a
            reference that contains no BTCS code, and the orders of accuracy observed from refined runs. Expected: about 1 in time and about
            2 in space (<Badge label="PRECOMPUTED" />, copied from <code>tests/golden/btcs/s5_n40_dt200/independent_check.csv</code>). The
            file calls itself a regression reference, not truth.
          </p>
          <p>
            Figure 3 plots the absolute difference of the release fraction computed by {METHOD_1} from the independent reference, and of the
            reference from the numerical limit estimated by refining the time step and the grid, at the four checkpoint times, on log-log
            axes. The reference is {METHOD_2} (the multilayer Sturm-Liouville series). The check passes when the reference lies within the
            stated convention of the estimated limit; that convention (5 percent of the difference plus the accuracy floor of the reference)
            is a labelled convention, not a derived bound.
          </p>
        </>
      ),
    },
    {
      id: 'verify-balance',
      title: 'Inventory balance',
      body: (
        <>
          <p>
            The mass-balance residual of the native run at every output time is plotted with its derived a-posteriori rounding bound, as the
            program wrote them; the residual is zero in exact arithmetic, so what remains is rounding.
          </p>
          <BalanceNumbers />
        </>
      ),
    },
    { id: 'verify-checks', title: 'Checks and their evidence', body: <ChecksTable /> },
  ],
  implementation: [
    {
      id: 'verify-files',
      title: 'Precomputed verification files',
      body: (
        <>
          <p>
            The checks can be repeated with <code>cargo test --all-targets</code>. Each precomputed file starts with the provenance header
            below.
          </p>
          <PrecomputedProvenance />
        </>
      ),
    },
  ],
};
