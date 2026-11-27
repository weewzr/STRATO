import { BUILD } from '../build';
import { useRunIsCurrent, useSelectionSupport } from '../equation/useCapability';
import { hrefDocumentation } from '../route';
import { useStrato } from '../state/store';

/**
 * Footer status line: label of the current main result, solver state and time, the solver version, and a
 * link to the build details. The build hash, the commit link and the other build facts live in Scientific
 * Documentation (section "Technical implementation and build"). On phones it keeps one line: the result
 * and the link.
 */
export function StatusLine({ compact = false }: { compact?: boolean }) {
  const run = useStrato((s) => s.run);
  const solver = useStrato((s) => s.solver);
  const info = useStrato((s) => s.runInfo);
  const status = useStrato((s) => s.status);
  const support = useSelectionSupport();
  const current = useRunIsCurrent();
  const solverText =
    solver.state === 'ready'
      ? status === 'running'
        ? 'Solver: running'
        : info && run && current
          ? `Solver: ${info.elapsedMs.toFixed(1)} ms in this browser`
          : 'Solver: ready'
      : solver.state === 'failed'
        ? 'Solver: not available'
        : 'Solver: loading';
  return (
    <footer className={`status${compact ? ' status--compact' : ''}`} aria-label="Status">
      <span className="status__item" data-testid="status-result">
        {!support.supported ? (
          'Result: none for the selection on Explore'
        ) : run && current ? (
          `Result: ${run.label}`
        ) : (
          'Result: none yet'
        )}
      </span>
      {compact ? null : (
        <span className="status__item" data-testid="status-solver">
          {solverText}
        </span>
      )}
      {compact ? null : (
        <span className="status__item" data-testid="status-version">
          {solver.build ? `v${solver.build.btcs.solver.version}` : `v${BUILD.strato_version}`}
        </span>
      )}
      <span className="status__item" data-testid="status-build">
        <a href={hrefDocumentation('build')} className="status__link">
          Build details
        </a>
      </span>
    </footer>
  );
}
