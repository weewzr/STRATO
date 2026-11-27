import { useEffect, useId, useRef, useState } from 'react';
import '../design/documentation.css';
import { DOC_SECTIONS, itemsOf, type DocItem } from '../documentation';
import { hrefDocumentation, useDocumentationAnchor } from '../route';
import { SectionHeading } from './common';

/**
 * Scientific Documentation, the last page of the navigation: the one place for assumptions, model
 * limitations, derivations, numerical methods, verification evidence, experimental interpretation, parameters,
 * references and downloads, research and implementation statuses, the advanced solver configuration and the
 * build information. The entries come from one content module per source page (src/documentation/); an entry
 * has an anchor that other pages link to with hrefDocumentation(id). Sections and entries expand and collapse;
 * the text filter hides the entries that do not contain the typed text.
 */

function Item({ item, anchor }: { item: DocItem; anchor: string }) {
  const [open, setOpen] = useState(!item.lazy);
  // An entry that the address names opens (a lazy one builds its content then).
  const [seen, setSeen] = useState('');
  if (anchor !== seen) {
    setSeen(anchor);
    if (anchor === item.id) setOpen(true);
  }
  return (
    <details
      id={`doc-${item.id}`}
      className="doc__item"
      open={open}
      onToggle={(e) => setOpen(e.currentTarget.open)}
      data-doc-item=""
      data-testid={`doc-item-${item.id}`}
    >
      <summary className="doc__summary">
        <h3 id={`doc-${item.id}-title`}>{item.title}</h3>
      </summary>
      {open ? item.body : null}
    </details>
  );
}

export function Documentation() {
  const anchor = useDocumentationAnchor();
  const [query, setQuery] = useState('');
  const [shown, setShown] = useState<number | null>(null);
  const filterId = useId();
  const body = useRef<HTMLDivElement>(null);
  const sections = DOC_SECTIONS.map((s) => ({ ...s, items: itemsOf(s.id) }));
  const total = sections.reduce((n, s) => n + s.items.length, 0);

  // An anchor scrolls to its section or entry; a new visit starts at the top.
  useEffect(() => {
    if (!anchor) return;
    const el = document.getElementById(`doc-${anchor}`);
    if (el instanceof HTMLDetailsElement) el.open = true;
    el?.closest('details.doc__section')?.setAttribute('open', '');
    if (typeof el?.scrollIntoView === 'function') el.scrollIntoView({ block: 'start' });
  }, [anchor]);

  // The filter: an entry stays when its text holds every typed word; a section with no entry left is hidden.
  useEffect(() => {
    const root = body.current;
    if (!root) return;
    const words = query.toLowerCase().split(/\s+/).filter(Boolean);
    let count = 0;
    root.querySelectorAll<HTMLElement>('[data-doc-item]').forEach((el) => {
      const text = (el.textContent ?? '').toLowerCase();
      const match = words.every((w) => text.includes(w));
      el.hidden = !match;
      if (match) count++;
    });
    root.querySelectorAll<HTMLDetailsElement>('details.doc__section').forEach((sec) => {
      const any = Array.from(sec.querySelectorAll<HTMLElement>('[data-doc-item]')).some((i) => !i.hidden);
      sec.hidden = !any;
      if (words.length > 0 && any) sec.open = true;
    });
    setShown(words.length > 0 ? count : null);
  }, [query]);

  return (
    <article className="section section--doc doc" data-section="documentation" data-testid="documentation-page">
      <SectionHeading>Scientific Documentation</SectionHeading>
      <div className="doc__filter" role="search">
        <label htmlFor={filterId} className="doc__filter-label">
          Filter the entries
        </label>
        <input
          id={filterId}
          type="search"
          className="doc__filter-input"
          value={query}
          autoComplete="off"
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Escape') setQuery('');
          }}
          data-testid="doc-filter"
        />
        <p className="doc__filter-count" role="status" data-testid="doc-filter-count">
          {shown === null ? '' : shown === 0 ? 'No entry contains this text.' : `${shown} of ${total} entries`}
        </p>
      </div>
      <div className="doc__layout">
        <nav className="doc__toc" aria-label="Contents of Scientific Documentation">
          <ol className="doc__toc-list">
            {sections.map((s) => (
              <li key={s.id}>
                <a className="doc__toc-link" href={hrefDocumentation(s.id)}>
                  {s.title}
                </a>
                {s.items.length > 0 ? (
                  <ul className="doc__toc-items">
                    {s.items.map((i) => (
                      <li key={i.id}>
                        <a className="doc__toc-link doc__toc-link--item" href={hrefDocumentation(i.id)}>
                          {i.title}
                        </a>
                      </li>
                    ))}
                  </ul>
                ) : null}
              </li>
            ))}
          </ol>
        </nav>
        <div className="doc__body" ref={body}>
          {sections.map((s) => (
            <details key={s.id} id={`doc-${s.id}`} className="doc__section" open data-testid={`doc-section-${s.id}`}>
              <summary className="doc__summary doc__summary--section">
                <h2 id={`doc-${s.id}-title`}>{s.title}</h2>
              </summary>
              {s.items.length === 0 ? (
                <p className="doc__empty" data-testid={`doc-empty-${s.id}`}>
                  No entries yet.
                </p>
              ) : (
                s.items.map((i) => <Item key={i.id} item={i} anchor={anchor} />)
              )}
            </details>
          ))}
        </div>
      </div>
    </article>
  );
}
