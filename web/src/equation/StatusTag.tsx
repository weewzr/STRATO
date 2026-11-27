import { STATUS_LABEL, type BuildStatus } from './capability';

/** The build status of a mechanism, a scenario or a physics choice, as one word with a border shape (never colour alone). */
export function StatusTag({ status, loading, id }: { status: BuildStatus; loading: boolean; id?: string }) {
  return (
    <span className={`bstatus bstatus--${loading ? 'loading' : status}`} data-status={loading ? 'loading' : status} data-testid={id}>
      {loading ? 'Checking the solver' : STATUS_LABEL[status]}
    </span>
  );
}
