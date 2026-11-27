import { useEffect } from 'react';
import '../design/documentation.css';
import { DOC_SECTIONS, itemsOf } from '../documentation';
import { AdvancedSolver } from '../advanced/AdvancedSolver';
import { hrefDocumentation, useDocumentationAnchor } from '../route';
import { SectionHeading } from './common';

/**
 * Scientific Documentation, the last page of the navigation: the one place for assumptions, model
 * limitations, derivations, verification methodology, experimental interpretation, technical
 * implementation and build information, and data provenance. The entries come from one content module per
 * source page (src/documentation/); an entry has an anchor that other pages link to with
 * hrefDocumentation(id).
 */
export function Documentation() {
  const anchor = useDocumentationAnchor();
  // An anchor scrolls to its section or entry; a new visit starts at the top.
  useEffect(() => {
    if (anchor) document.getElementById(`doc-${anchor}`)?.scrollIntoView({ block: 'start' });
  }, [anchor]);

  const sections = DOC_SECTIONS.map((s) => ({ ...s, items: itemsOf(s.id) }));
  return (
    <article className="section section--doc doc" data-section="documentation" data-testid="documentation-page">
      <SectionHeading>Scientific Documentation</SectionHeading>
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
        <div className="doc__body">
          {sections.map((s) => (
            <section key={s.id} id={`doc-${s.id}`} className="doc__section" aria-labelledby={`doc-${s.id}-title`} data-testid={`doc-section-${s.id}`}>
              <h2 id={`doc-${s.id}-title`}>{s.title}</h2>
              {s.items.length === 0 ? (
                <p className="doc__empty" data-testid={`doc-empty-${s.id}`}>
                  No entries yet.
                </p>
              ) : (
                s.items.map((i) => (
                  <section key={i.id} id={`doc-${i.id}`} className="doc__item" aria-labelledby={`doc-${i.id}-title`} data-testid={`doc-item-${i.id}`}>
                    <h3 id={`doc-${i.id}-title`}>{i.title}</h3>
                    {i.body}
                  </section>
                ))
              )}
            </section>
          ))}
        </div>
      </div>
      {/* PREVIEW ONLY (not for merge): the page of the other agent mounts AdvancedSolver under Advanced solver configuration. */}
      <section aria-label="Advanced solver configuration" id="doc-advanced">
        <AdvancedSolver compact={false} />
      </section>
    </article>
  );
}
