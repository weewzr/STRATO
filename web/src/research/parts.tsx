import { useId } from 'react';
import { isRowAvailable } from '../equation/availability';
import { hrefDocumentation, hrefFor } from '../route';
import { useStrato } from '../state/store';
import { TableBox } from '../sections/common';
import { Cite } from './Blocks';
import { referenceRow, type Catalogue, type GalleryRow, type MechanismRow } from './catalogue';
import { entryHref, gallerySpecies, linkCheckedDate, rowKey } from './entries';
import { NoteText } from './PathLink';
import { STATUSES, isImplementedStatus } from './status';
import { StatusChip } from './StatusChip';

// Building blocks of the research catalogue interface (web phase 5.5). Every value shown comes
// from the generated catalogue; these components only arrange it.

/**
 * The five statuses in one row, each a word and a shape (the definition is the hover title of the chip). The
 * definitions are printed once, in Scientific Documentation, which the link leads to.
 */
export function StatusLegend({ id, title = 'Research status' }: { id?: string; title?: string }) {
  const own = useId();
  const head = id ?? own;
  return (
    <section className="status-legend" aria-labelledby={head} data-testid="status-legend">
      <p className="status-legend__title" id={head}>
        {title}
      </p>
      <ul className="status-legend__list" aria-label="The five research statuses">
        {STATUSES.map((s) => (
          <li key={s}>
            <StatusChip status={s} legend />
          </li>
        ))}
      </ul>
      <a className="status-legend__more" href={hrefDocumentation('research-status-scale')}>
        Definitions
      </a>
    </section>
  );
}

/** A row's chip: the status of that catalogue row (mechanism, species and material domain). */
export function RowChip({ row, compact = false }: { row: MechanismRow; compact?: boolean }) {
  return <StatusChip status={row.status} mechanism={row.mechanism_id} row={rowKey(row)} compact={compact} />;
}

/**
 * The live-simulation switch of a catalogue row. Present, focusable and disabled while the row is
 * not available (isRowAvailable: a solver capability reported by the Rust wrapper and the live
 * status); it says why and what is missing. Operating it changes nothing and sends no request. A
 * row that is available is not switched here: the page points to the controls that run it.
 */
export function CatalogueSwitch({ row, compact = false }: { row: MechanismRow; compact?: boolean }) {
  const capabilities = useStrato((s) => s.capabilities);
  const why = useId();
  const key = rowKey(row);
  if (isRowAvailable(row, capabilities)) {
    return (
      <p className="cat-switch cat-switch--live" data-testid={`row-live-${key}`}>
        In the live simulation: switched under Controls on <a href={hrefFor('explore')}>Explore</a>.
      </p>
    );
  }
  // A research-only mechanism needs no reason beside its status word: the statement that STRATO does not
  // solve it is made once, in Scientific Documentation.
  const reason = isImplementedStatus(row.status) ? 'the solver does not report this mechanism as verified and available' : '';
  return (
    <div className={`cat-switch${compact ? ' cat-switch--compact' : ''}`}>
      <button
        type="button"
        role="switch"
        aria-checked="false"
        aria-disabled="true"
        aria-describedby={why}
        className="mech__switch mech__switch--catalogue"
        data-testid={`row-switch-${key}`}
        data-row={key}
      >
        <span className="mech__swatch mech__swatch--dotted" aria-hidden="true" />
        <span className="mech__name">Live simulation{row.species !== 'all' ? `, ${row.species}` : ''}</span>
        <span className="mech__term">
          Not available: status {row.status}
          {reason ? `; ${reason}` : ''}
        </span>
        <span className="mech__state">Off</span>
      </button>
      <p className="cat-switch__why" id={why}>
        {row.missing_work ? (
          <>
            <span className="cat-switch__lead">Missing: </span>
            <NoteText text={row.missing_work} />
          </>
        ) : (
          <>No missing work is recorded in the catalogue row.</>
        )}
      </p>
    </div>
  );
}

/** Required parameters, each marked as registered (with its register row) or missing. */
export function Parameters({ items }: { items: readonly string[] }) {
  if (items.length === 0) return <span className="muted">none listed</span>;
  return (
    <ul className="params">
      {items.map((p) => {
        const m = /^(.*?)\s*\((registered|missing)([^)]*)\)\s*$/.exec(p);
        const name = m ? m[1] : p;
        const state = m ? m[2] : '';
        const detail = m ? (m[3] ?? '').replace(/^[\s,;:]+/, '') : '';
        return (
          <li key={p} className={`params__item${state ? ` params__item--${state}` : ''}`}>
            <span className="params__name">{name}</span>
            {state ? (
              <span className="params__state">
                {state === 'registered' ? 'registered' : 'missing'}
                {detail ? (
                  <>
                    : <NoteText text={detail} />
                  </>
                ) : null}
              </span>
            ) : null}
          </li>
        );
      })}
    </ul>
  );
}

/** The catalogue record of some rows: one card per row with its status, basis, parameters and switch. */
export function RowRecords({ rows, catalogue, showEntryLink = false }: { rows: readonly MechanismRow[]; catalogue: Catalogue; showEntryLink?: boolean }) {
  return (
    <ul className="records" aria-label="Catalogue rows">
      {rows.map((r) => (
        <li key={rowKey(r)} className={`record${isImplementedStatus(r.status) ? ' record--implemented' : ' record--pending'}`} data-testid="row-record" data-row={rowKey(r)}>
          <div className="record__head">
            <p className="record__name">
              {showEntryLink && r.entry ? <a href={entryHref(r.entry)}>{r.name}</a> : r.name}
            </p>
            <RowChip row={r} />
          </div>
          <CatalogueSwitch row={r} />
          <dl className="record__meta">
            <div>
              <dt>Species</dt>
              <dd>{r.species === 'all' ? 'any species' : r.species}</dd>
            </div>
            <div>
              <dt>Material domain</dt>
              <dd>{r.material_domain.replace(/_/g, ' ')}</dd>
            </div>
            <div>
              <dt>Regime</dt>
              <dd>{r.regime.join(', ')}</dd>
            </div>
            <div>
              <dt>Brick</dt>
              <dd>{r.brick ?? 'none'}</dd>
            </div>
            <div>
              <dt>Status since</dt>
              <dd>{r.status_date || 'not recorded'}</dd>
            </div>
          </dl>
          <p className="record__basis">
            <span className="record__lead">Basis of the status: </span>
            <NoteText text={r.status_basis} />
          </p>
          <div className="record__params">
            <p className="record__lead">Required parameters</p>
            <Parameters items={r.required_parameters} />
          </div>
          {r.reference_ids.length > 0 ? (
            <p className="record__refs">
              <span className="record__lead">Sources: </span>
              {r.reference_ids.map((id, i) => (
                <span key={id}>
                  {i > 0 ? ' ' : null}
                  <Cite id={id} catalogue={catalogue} />
                </span>
              ))}
            </p>
          ) : null}
        </li>
      ))}
    </ul>
  );
}

/** Full reference list of some ids, with link and the date on which the link was checked. */
export function ReferenceList({ ids, catalogue, label = 'References' }: { ids: readonly string[]; catalogue: Catalogue; label?: string }) {
  return (
    <ol className="ref-list" aria-label={label} data-testid="reference-list">
      {ids.map((id) => {
        const r = referenceRow(catalogue, id);
        if (!r) {
          return (
            <li key={id} className="ref-list__item ref-list__item--missing" data-ref-id={id}>
              {id}: not in the reference list
            </li>
          );
        }
        const checked = linkCheckedDate(r.link_checked);
        return (
          <li key={id} className="ref-list__item" data-ref-id={id}>
            <span className="ref-list__label">{catalogue.referenceLabels[id] ?? id}</span>
            <span className="ref-list__citation">{r.citation}</span>{' '}
            <a className="ref-list__link" href={r.doi_or_url} rel="noopener noreferrer">
              {r.doi_or_url}
            </a>
            <span className={`ref-list__checked${checked ? '' : ' ref-list__checked--no'}`} data-link-checked={checked ? 'date' : 'no'}>
              {checked ? `Link opened on ${checked}.` : `Link: ${r.link_checked}.`}
            </span>
            {r.notes ? (
              <span className="ref-list__notes">
                <NoteText text={r.notes} />
              </span>
            ) : null}
          </li>
        );
      })}
    </ol>
  );
}

const KIND_LABEL: Record<GalleryRow['kind'], string> = {
  screenshot: 'Materials Studio screenshot',
  reconstruction: 'Interactive reconstruction of an exported structure',
  placeholder: 'Pending placeholder: calculation not performed, no image',
};

/** Where a gallery file is served (vite.config.ts copies docs/research/gallery/ to research/gallery/). */
export function galleryFileUrl(file: string): string {
  return `${import.meta.env.BASE_URL}${file.replace(/^docs\//, '')}`;
}

/**
 * One item of the Materials Studio gallery (prompt/DESIGN_SPECIFICATION.md Section 7.5, rule E5).
 * A placeholder is a plain panel with its status and the planned calculation, and no picture. A
 * screenshot shows its image only when the row names a file and a provenance; a reconstruction is
 * framed and labelled differently and never mixed with screenshots.
 */
export function GalleryItem({ item, catalogue }: { item: GalleryRow; catalogue: Catalogue }) {
  const species = gallerySpecies(item.species);
  const showsImage = (item.kind === 'screenshot' || item.kind === 'reconstruction') && item.file !== '' && item.provenance !== '';
  const planned = item.kind === 'placeholder';
  return (
    <article className={`gallery-item gallery-item--${item.kind}`} data-testid="gallery-item" data-kind={item.kind} data-item={item.item_id} aria-labelledby={`g-${item.item_id}`}>
      <p className="gallery-item__kind">{KIND_LABEL[item.kind]}</p>
      <div className="gallery-item__head">
        <h3 className="gallery-item__title" id={`g-${item.item_id}`}>
          {item.title}
        </h3>
        <StatusChip status={item.status} compact />
      </div>
      {showsImage && item.kind === 'screenshot' ? (
        <figure className="gallery-item__figure">
          <img src={galleryFileUrl(item.file)} alt={item.alt_text} loading="lazy" />
        </figure>
      ) : null}
      {showsImage && item.kind === 'reconstruction' ? (
        <p className="gallery-item__file">
          Structure file: <a href={galleryFileUrl(item.file)}>{item.file}</a> (no atomistic viewer is installed on this site)
        </p>
      ) : null}
      <p className="gallery-item__caption">{item.caption}</p>
      <p className="gallery-item__alt">
        <span className="record__lead">In plain words: </span>
        {item.alt_text}
      </p>
      <dl className="gallery-item__meta">
        <div>
          <dt>Material</dt>
          <dd>{item.material}</dd>
        </div>
        <div>
          <dt>Species</dt>
          <dd>{species.length > 0 ? species.join(', ') : 'none'}</dd>
        </div>
        <div>
          <dt>{planned ? 'Planned method' : 'Method'}</dt>
          <dd>{item.method}</dd>
        </div>
        <div>
          <dt>{planned ? 'Planned conditions' : 'Conditions'}</dt>
          <dd>{item.conditions}</dd>
        </div>
        <div>
          <dt>Provenance</dt>
          <dd>{item.provenance || 'none: nothing has been produced'}</dd>
        </div>
      </dl>
      {item.mechanism_ids.length > 0 ? (
        <p className="gallery-item__links">
          <span className="record__lead">Catalogue entries: </span>
          {item.mechanism_ids.map((id, i) => {
            const row = catalogue.mechanisms.find((m) => m.mechanism_id === id);
            return (
              <span key={id}>
                {i > 0 ? '; ' : ''}
                {row?.entry ? <a href={entryHref(row.entry)}>{row.name}</a> : id}
              </span>
            );
          })}
        </p>
      ) : null}
      {item.reference_ids.length > 0 ? (
        <p className="gallery-item__links">
          <span className="record__lead">Sources for the planned subject: </span>
          {item.reference_ids.map((id, i) => (
            <span key={id}>
              {i > 0 ? ' ' : null}
              <Cite id={id} catalogue={catalogue} />
            </span>
          ))}
        </p>
      ) : null}
    </article>
  );
}

/** The sentence every planned calculation ends with; it is stated once, in Scientific Documentation. */
const NOT_PERFORMED = /\s*This calculation has not been performed\. It is planned for a future phase of the project\./;

/** A planned calculation's text without the sentence that all planned calculations share. */
export function withoutNotPerformed(text: string): string {
  return text.replace(NOT_PERFORMED, '');
}

/** One planned calculation in full, once (Scientific Documentation): the texts of its gallery row, nothing shortened. */
export function GalleryPlannedDetail({ item, catalogue }: { item: GalleryRow; catalogue: Catalogue }) {
  const species = gallerySpecies(item.species);
  return (
    <article className="gallery-detail" data-testid="gallery-detail" data-item={item.item_id}>
      <h4 className="gallery-detail__title">
        {item.title} <StatusChip status={item.status} compact />
      </h4>
      <p>{withoutNotPerformed(item.caption)}</p>
      <p>
        <span className="record__lead">In plain words: </span>
        {withoutNotPerformed(item.alt_text)}
      </p>
      <ul className="prose-list">
        <li>Material: {item.material}</li>
        <li>Species: {species.length > 0 ? species.join(', ') : 'none'}</li>
        <li>Planned method: {item.method}</li>
        <li>Planned conditions: {item.conditions}</li>
        {item.mechanism_ids.length > 0 ? (
          <li>
            Catalogue entries:{' '}
            {item.mechanism_ids.map((id, i) => {
              const row = catalogue.mechanisms.find((m) => m.mechanism_id === id);
              return (
                <span key={id}>
                  {i > 0 ? '; ' : ''}
                  {row?.entry ? <a href={entryHref(row.entry)}>{row.name}</a> : id}
                </span>
              );
            })}
          </li>
        ) : null}
        {item.reference_ids.length > 0 ? (
          <li>
            Sources for the planned subject:{' '}
            {item.reference_ids.map((id, i) => (
              <span key={id}>
                {i > 0 ? ' ' : null}
                <Cite id={id} catalogue={catalogue} />
              </span>
            ))}
          </li>
        ) : null}
      </ul>
    </article>
  );
}

/** A short list of rows: status, then the name (linked to its entry) and the species or domain. */
export function RowList({ rows, label, testId }: { rows: readonly MechanismRow[]; label: string; testId?: string }) {
  return (
    <ul className="row-list" aria-label={label} data-testid={testId}>
      {rows.map((r) => (
        <li key={rowKey(r)} className="row-list__item">
          <RowChip row={r} compact />
          <span className="row-list__name">{r.entry ? <a href={entryHref(r.entry)}>{r.name}</a> : r.name}</span>
          <span className="row-list__meta">
            {r.species === 'all' ? 'any species' : r.species}, {r.material_domain.replace(/_/g, ' ')}
            {r.brick !== null ? `, brick ${r.brick}` : ''}
          </span>
        </li>
      ))}
    </ul>
  );
}

/** A compact table of rows: name (linked to its entry), species, status. */
export function RowTable({ rows, label }: { rows: readonly MechanismRow[]; label: string }) {
  return (
    <TableBox label={label}>
      <table className="table row-table">
        <thead>
          <tr>
            <th scope="col">Mechanism</th>
            <th scope="col">Species</th>
            <th scope="col">Brick</th>
            <th scope="col">Research status</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={rowKey(r)}>
              <th scope="row">{r.entry ? <a href={entryHref(r.entry)}>{r.name}</a> : r.name}</th>
              <td>{r.species === 'all' ? 'any' : r.species}</td>
              <td className="num-col">{r.brick ?? ''}</td>
              <td>
                <RowChip row={r} compact />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </TableBox>
  );
}
