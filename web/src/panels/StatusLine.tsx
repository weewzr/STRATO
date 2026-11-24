import { BUILD, REPO_URL } from '../build';
import { useStrato } from '../state/store';
import { Badge } from './Badge';

/**
 * Footer status line: label of the current main result, solver state, model version and commit.
 * On phones it keeps one line: the result and the build commit.
 */
export function StatusLine({ compact = false }: { compact?: boolean }) {
  const run = useStrato((s) => s.run);
  return (
    <footer className={`status${compact ? ' status--compact' : ''}`} aria-label="Status">
      <span className="status__item" data-testid="status-result">
        {run ? (
          <>
            Result <Badge label={run.label} />
          </>
        ) : (
          'Result: none in this build'
        )}
      </span>
      {compact ? null : <span className="status__item">Solver: not connected (web phase 3)</span>}
      {compact ? null : <span className="status__item">STRATO v{BUILD.strato_version}</span>}
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
