import { BTCS_CAPABILITY } from '../equation/availability';
import { METHOD_1 } from '../methods';
import { Badge } from '../panels/Badge';
import { BalancePanelVerify, CheckFigurePanel, MethodComparisonPanel, ObservedOrdersTable } from '../panels/Verification';
import { statusOf, useCatalogue } from '../research/catalogue';
import { StatusChip } from '../research/StatusChip';
import { hrefDocumentation } from '../route';
import { useStrato } from '../state/store';
import { SectionHeading } from './common';

/** One line: the method, its result label and its research status. Version, build and scope are in Scientific Documentation. */
function VerifyStatus() {
  const build = useStrato((s) => s.solver.build);
  const state = useStrato((s) => s.solver.state);
  const run = useStrato((s) => s.run);
  const catalogue = useCatalogue();
  if (state === 'failed') {
    return (
      <p className="solver-status solver-status--compact solver-status--failed" role="alert" data-testid="verify-status" data-state="failed">
        Solver: not available. {METHOD_1} did not load, so the page shows no live result.
      </p>
    );
  }
  if (state !== 'ready' || !build) {
    return (
      <p className="solver-status solver-status--compact" role="status" data-testid="verify-status" data-state="loading">
        Solver: loading {METHOD_1}.
      </p>
    );
  }
  const label = (run?.solver ?? build.btcs.solver).label;
  const status = statusOf(catalogue, BTCS_CAPABILITY);
  return (
    <p className="solver-status solver-status--compact" data-testid="verify-status" data-state="ready">
      <strong>{METHOD_1}</strong> <Badge label={label} /> {status ? <StatusChip status={status} mechanism={BTCS_CAPABILITY} compact /> : 'unknown'}{' '}
      <a href={hrefDocumentation('verify-scope')}>Scope and method</a>
    </p>
  );
}

export function Verify() {
  return (
    <article className="section" data-section="verify">
      <SectionHeading>Verification of the BTCS solver</SectionHeading>
      <VerifyStatus />
      <MethodComparisonPanel />
      <div className="plot-grid plot-grid--two">
        <CheckFigurePanel />
        <BalancePanelVerify />
      </div>
      <ObservedOrdersTable />
    </article>
  );
}
