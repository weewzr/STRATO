import { STATUS_INFO, type ResearchStatus } from './status';

/**
 * Research status of a mechanism: the word plus a shape (dotted, dashed or solid ring, filled disc,
 * filled disc with a check mark), never colour alone. Distinct from the result labels of numbers
 * (Badge): a different typeface and a leading shape instead of a box.
 */
export function StatusChip({ status, mechanism, compact = false }: { status: ResearchStatus; mechanism: string; compact?: boolean }) {
  const { shape, definition } = STATUS_INFO[status];
  return (
    <span
      className={`status-chip status-chip--${shape}${compact ? ' status-chip--compact' : ''}`}
      data-status={status}
      data-status-for={mechanism}
      title={definition}
    >
      <svg className="status-chip__shape" viewBox="0 0 12 12" width="12" height="12" aria-hidden="true" focusable="false">
        <circle cx="6" cy="6" r="4.75" />
        {shape === 'fill-check' ? <path d="M3.6 6.2 5.3 7.8 8.5 4.4" /> : null}
      </svg>
      <span className="visually-hidden">Research status: </span>
      <span className="status-chip__word">{status}</span>
    </span>
  );
}
