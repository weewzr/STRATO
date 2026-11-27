import { lazy, Suspense } from 'react';
import { isRowAvailable } from '../equation/availability';
import { ExperimentalComparisons } from '../graph/Comparisons';
import { METHOD_1 } from '../methods';
import { useCatalogue } from '../research/catalogue';
import { rowKey } from '../research/entries';
import { NoteText, PathStyle } from '../research/PathLink';
import { StatusChip } from '../research/StatusChip';
import { TableBox } from '../sections/common';
import { useStrato } from '../state/store';
import { ComparisonEvidence } from './ComparisonEvidence';
import { DownloadsEvidence, ParametersEvidence, ReferencesEvidence } from './DataEvidence';
import type { DocContribution } from './types';
import { VerificationEvidence } from './VerificationEvidence';

// The evidence that left the standalone Compare, Validate, Verify, Data and About pages and the status
// records that left the Research Catalogue: the live figures and tables stay live, exactly as they were, and
// each sits here once. Items marked `lazy` are built only when opened (they run the solver or read files).

/** The experimental comparison graphs of the Validate page, with the mount point they always had. */
export function ExperimentalEvidence() {
  return (
    <div data-mount="experimental-comparison-graph" data-testid="experimental-comparison-mount">
      <ExperimentalComparisons />
    </div>
  );
}

/**
 * Research status of every catalogue row, generated from the same catalogue data as the Research Catalogue,
 * with whether the solver of this build reports the mechanism (the wrapper's own capability list).
 */
export function ResearchStatusTable() {
  const cat = useCatalogue();
  const capabilities = useStrato((s) => s.capabilities);
  return (
    <TableBox label="Research status of every catalogue row">
      <PathStyle.Provider value="files">
        <table className="table table--compact" data-testid="status-table">
          <caption className="visually-hidden">Research status of every catalogue row</caption>
          <thead>
            <tr>
              <th scope="col">Mechanism</th>
              <th scope="col">Species</th>
              <th scope="col">Material domain</th>
              <th scope="col">Regime</th>
              <th scope="col" className="num-col">
                Brick
              </th>
              <th scope="col">Research status</th>
              <th scope="col">Since</th>
              <th scope="col">Basis of the status</th>
              <th scope="col">Solved in this build</th>
            </tr>
          </thead>
          <tbody>
            {cat.mechanisms.map((r) => (
              <tr key={rowKey(r)} data-row={rowKey(r)}>
                <th scope="row">{r.name}</th>
                <td>{r.species === 'all' ? 'any' : r.species}</td>
                <td>{r.material_domain.replace(/_/g, ' ')}</td>
                <td>{r.regime.join(', ')}</td>
                <td className="num-col">{r.brick ?? ''}</td>
                <td>
                  <StatusChip status={r.status} mechanism={r.mechanism_id} row={rowKey(r)} compact />
                </td>
                <td>{r.status_date || 'not recorded'}</td>
                <td>
                  <NoteText text={r.status_basis} />
                </td>
                <td>{isRowAvailable(r, capabilities) ? 'yes' : 'no'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </PathStyle.Provider>
    </TableBox>
  );
}

const AdvancedSolver = lazy(() => import('../advanced/AdvancedSolver').then((m) => ({ default: m.AdvancedSolver })));

/** The manual baseline solver and its 3D view; the code loads only when the entry is opened. */
export function AdvancedSolverMount() {
  return (
    <div data-mount="advanced-solver-configuration" data-testid="advanced-solver-mount">
      <Suspense fallback={null}>
        <AdvancedSolver />
      </Suspense>
    </div>
  );
}

/** What the solver of this build implements, as the WebAssembly wrapper reports it. */
function ImplementedByBuild() {
  const solver = useStrato((s) => s.solver);
  const capabilities = useStrato((s) => s.capabilities);
  const scope = solver.build?.btcs.solver.scope;
  return (
    <div data-testid="statuses-implemented">
      <p>
        {METHOD_1}: {scope ?? (solver.state === 'failed' ? 'the solver is not available' : 'loading')}.
      </p>
      <p>
        Capabilities reported by the wrapper:{' '}
        {capabilities.length > 0 ? capabilities.map((c, i) => (
          <span key={c}>
            {i > 0 ? ', ' : ''}
            <code>{c}</code>
          </span>
        )) : solver.state === 'failed' ? 'none (solver not available)' : 'loading'}
        .
      </p>
    </div>
  );
}

/** The R² suitability requirements in a few sentences; the authority is the metric note of the repository (decision 74). */
function R2Requirements() {
  return (
    <>
      <p>
        R² is the only displayed agreement metric. It is computed in Rust, per release-versus-time curve, and shown only where the comparison
        is scientifically defensible; otherwise a one-sentence caption below the graph says why. No registered dataset supports one today.
      </p>
      <p>
        Seventeen ordered checks decide, and the first check that a curve fails gives its reason code. Of the fifteen registered curves, four
        (AGR-1 loose particles) fail on <code>coating_failure</code>, nine (compact or sphere totals) on <code>upper_bound</code> and two
        (JAERI) on <code>incompatible_definition</code>.
      </p>
      <p data-testid="r2-reasons">
        The other reason codes are <code>compact_not_relatable</code>, <code>decay_not_comparable</code> and{' '}
        <code>model_not_configurable</code>; an assessment field that is not set gives <code>missing_conditions</code>. Each record carries one
        sentence of caption, with its citation and link in separate fields.
      </p>    </>
  );
}

export const evidence: DocContribution = {
  interpretation: [
    {
      id: 'r2-requirements',
      title: 'R² and its suitability requirements',
      body: <R2Requirements />,
    },
    {
      id: 'evidence-experimental',
      title: 'Experimental comparison graphs',
      lazy: true,
      body: <ExperimentalEvidence />,
    },
  ],
  evidence: [
    {
      id: 'evidence-comparison',
      title: 'Comparison with the baseline',
      lazy: true,
      body: <ComparisonEvidence />,
    },
    {
      id: 'evidence-verification',
      title: 'Verification of the BTCS solver',
      lazy: true,
      body: <VerificationEvidence />,
    },
  ],
  data: [
    { id: 'evidence-parameters', title: 'Scientific parameters in use', body: <ParametersEvidence /> },
    { id: 'evidence-references', title: 'References (APA) and links', body: <ReferencesEvidence /> },
    { id: 'evidence-downloads', title: 'Data downloads', body: <DownloadsEvidence /> },
  ],
  statuses: [
    { id: 'statuses-research', title: 'Research status of every catalogue row', body: <ResearchStatusTable /> },
    { id: 'statuses-implemented', title: 'What the solver of this build implements', body: <ImplementedByBuild /> },
  ],
  advanced: [
    {
      id: 'advanced-solver',
      title: 'Manual baseline solver and numerical controls',
      lazy: true,
      body: <AdvancedSolverMount />,
    },
  ],
};
