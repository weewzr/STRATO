import { STATUS_INFO, type ResearchStatus } from './status';

/**
 * Research status of a mechanism: the word plus a shape (dotted, dashed or solid ring, filled disc,
 * filled disc with a check mark), never colour alone. Distinct from the result labels of numbers
 * (Badge): a different typeface and a leading shape instead of a box.
 *
 * `mechanism` names the mechanism whose status this is (data-status-for); `row` narrows it to one
 * catalogue row ("mechanism_id|species|material_domain", data-status-row) where a mechanism has
 * several rows. A chip with `count` stands for that many rows at the status (a summary), and a
 * chip with `legend` explains the status itself; neither names a mechanism.
 */
export function StatusChip({
  status,
  mechanism,
  row,
  count,
  legend = false,
  compact = false,
}: {
  status: ResearchStatus;
  mechanism?: string;
  row?: string;
  count?: number;
  legend?: boolean;
  compact?: boolean;
}) {
  const { shape, definition } = STATUS_INFO[status];
  const named = !legend && count === undefined && mechanism !== undefined;
  return (
    <span
      className={`status-chip status-chip--${shape}${compact ? ' status-chip--compact' : ''}${count !== undefined ? ' status-chip--count' : ''}`}
      data-status={status}
      data-status-for={named ? mechanism : undefined}
      data-status-row={named ? row : undefined}
      data-status-count={count}
      data-status-legend={legend ? '' : undefined}
      title={definition}
    >
      <svg className="status-chip__shape" viewBox="0 0 12 12" width="12" height="12" aria-hidden="true" focusable="false">
        <circle cx="6" cy="6" r="4.75" />
        {shape === 'fill-check' ? <path d="M3.6 6.2 5.3 7.8 8.5 4.4" /> : null}
      </svg>
      {count !== undefined ? <span className="status-chip__count">{count}</span> : null}
      <span className="visually-hidden">{count !== undefined ? (count === 1 ? ' row at research status: ' : ' rows at research status: ') : 'Research status: '}</span>
      <span className="status-chip__word">{status}</span>
    </span>
  );
}
