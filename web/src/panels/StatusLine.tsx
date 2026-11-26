import { BUILD, REPO_URL } from '../build';
import { useSelectionSupport } from '../equation/useCapability';
import { useStrato } from '../state/store';
import { Badge } from './Badge';

/**
 * Footer status line: label of the current main result, solver state and time, model version and
 * commit. On phones it keeps one line: the result and the build commit.
 */
export function StatusLine({ compact = false }: { compact?: boolean }) {
  const run = useStrato((s) => s.run);
  const solver = useStrato((s) => s.solver);
  const info = useStrato((s) => s.runInfo);
  const status = useStrato((s) => s.status);
  const support = useSelectionSupport();
  const solverText =
    solver.state === 'ready'
      ? status === 'running'
        ? 'Solver: running'
        : info && run
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
        ) : run ? (
          <>
            Result <Badge label={run.label} />
          </>
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
          {solver.build ? `Solver version ${solver.build.btcs.solver.version}` : `STRATO v${BUILD.strato_version}`}
        </span>
      )}
      <span className="status__item" data-testid="status-commit">
        build{' '}
        <a href={`${REPO_URL}/commit/${BUILD.commit}`} className="status__link">
          {BUILD.commit}
        </a>
        {BUILD.dirty ? (compact ? ' (uncommitted)' : ' (uncommitted changes)') : ''}
      </span>
    </footer>
  );
}
