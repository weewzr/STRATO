import { Fragment, useMemo, type ReactNode } from 'react';
import { renderTex } from '../equation/Tex';
import { TableBox } from '../sections/common';
import { referenceRow, type Block, type Catalogue, type Span } from './catalogue';
import { entryHref, spansText } from './entries';

// Renders the entry text of the research catalogue from the blocks that
// scripts/web/generate_data.py writes (headings, paragraphs, lists, tables, display maths and
// inline spans). No Markdown is parsed here and no text is added to the entry: only the label of
// each display equation ("Research concept, not solved by STRATO", prompt/DESIGN_SPECIFICATION.md
// Section 7.3) and its number within the page.

/** KaTeX for catalogue text. A string KaTeX cannot typeset is shown as its source, marked as such. */
export function ResearchTex({ tex, display }: { tex: string; display: boolean }) {
  const html = useMemo(() => {
    try {
      return renderTex(tex, display);
    } catch {
      return null;
    }
  }, [tex, display]);
  if (html === null) {
    return (
      <code className="tex-source" data-testid="tex-failed" title="This equation could not be typeset; its TeX source is shown">
        {tex}
      </code>
    );
  }
  return display ? (
    <div className="tex tex--display" dangerouslySetInnerHTML={{ __html: html }} />
  ) : (
    <span className="tex" dangerouslySetInnerHTML={{ __html: html }} />
  );
}

/** A citation: the short label, linked to the source, with the full citation as its title. */
export function Cite({ id, catalogue }: { id: string; catalogue: Catalogue }) {
  const row = referenceRow(catalogue, id);
  const label = catalogue.referenceLabels[id] ?? id;
  if (!row) {
    return (
      <span className="cite cite--missing" data-ref={id}>
        ({id}: not in the reference list)
      </span>
    );
  }
  return (
    <span className="cite" data-ref={id}>
      (
      <a href={row.doi_or_url} title={row.citation} data-testid="cite-link" rel="noopener noreferrer">
        {label}
      </a>
      )
    </span>
  );
}

export function Spans({ spans, catalogue }: { spans: readonly Span[]; catalogue: Catalogue }) {
  return (
    <>
      {spans.map((s, i) => {
        switch (s.t) {
          case 'text':
            return <Fragment key={i}>{s.v}</Fragment>;
          case 'code':
            return <code key={i}>{s.v}</code>;
          case 'math':
            return <ResearchTex key={i} tex={s.v} display={false} />;
          case 'ref':
            return <Cite key={i} id={s.id} catalogue={catalogue} />;
          case 'strong':
            return (
              <strong key={i}>
                <Spans spans={s.c} catalogue={catalogue} />
              </strong>
            );
          case 'em':
            return (
              <em key={i}>
                <Spans spans={s.c} catalogue={catalogue} />
              </em>
            );
          case 'link':
            return s.entry ? (
              <a key={i} href={entryHref(s.entry, s.anchor ?? '')} data-entry-link={s.entry}>
                <Spans spans={s.c} catalogue={catalogue} />
              </a>
            ) : (
              <a key={i} href={s.href} rel="noopener noreferrer">
                <Spans spans={s.c} catalogue={catalogue} />
              </a>
            );
        }
      })}
    </>
  );
}

/** Id of a heading of an entry on the page (prefixed, so it cannot meet another id of the app). */
export function headingDomId(id: string): string {
  return `r-${id}`;
}

/**
 * The blocks of an entry text. Headings move down by `shift` levels so that the page keeps one h1;
 * display equations are numbered from `firstEquation` and labelled as research concepts.
 */
export function Blocks({
  blocks,
  catalogue,
  shift = 0,
  firstEquation = 1,
}: {
  blocks: readonly Block[];
  catalogue: Catalogue;
  shift?: number;
  firstEquation?: number;
}) {
  let eq = firstEquation - 1;
  const out: ReactNode[] = [];
  blocks.forEach((b, i) => {
    switch (b.t) {
      case 'h': {
        const level = Math.min(6, Math.max(2, b.level + shift));
        const Tag = `h${level}` as 'h2' | 'h3' | 'h4' | 'h5' | 'h6';
        out.push(
          <Tag key={i} id={headingDomId(b.id)} className={`entry-h entry-h--${level}`}>
            <Spans spans={b.c} catalogue={catalogue} />
          </Tag>,
        );
        break;
      }
      case 'p':
        out.push(
          <p key={i}>
            <Spans spans={b.c} catalogue={catalogue} />
          </p>,
        );
        break;
      case 'ul':
        out.push(
          <ul key={i} className="entry-list">
            {b.items.map((it, k) => (
              <li key={k}>
                <Spans spans={it} catalogue={catalogue} />
              </li>
            ))}
          </ul>,
        );
        break;
      case 'ol':
        out.push(
          <ol key={i} className="entry-list" start={b.start}>
            {b.items.map((it, k) => (
              <li key={k}>
                <Spans spans={it} catalogue={catalogue} />
              </li>
            ))}
          </ol>,
        );
        break;
      case 'table':
        out.push(
          <TableBox key={i} label={`Table: ${b.head.map((h) => spansText(h)).join(', ')}`}>
            <table className="table entry-table">
              <thead>
                <tr>
                  {b.head.map((h, k) => (
                    <th key={k} scope="col">
                      <Spans spans={h} catalogue={catalogue} />
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {b.rows.map((r, k) => (
                  <tr key={k}>
                    {r.map((c, j) => (
                      <td key={j}>
                        <Spans spans={c} catalogue={catalogue} />
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </TableBox>,
        );
        break;
      case 'math': {
        eq += 1;
        out.push(
          <figure key={i} className="concept-eq" data-testid="concept-equation" data-concept-number={eq}>
            <figcaption className="concept-eq__label">
              Research concept, not solved by STRATO<span className="visually-hidden">, equation {eq} of this page</span>
            </figcaption>
            <div className="concept-eq__row">
              <div className="eq-box concept-eq__box">
                <ResearchTex tex={b.tex} display />
              </div>
              <span className="concept-eq__num" aria-hidden="true">
                ({eq})
              </span>
            </div>
          </figure>,
        );
        break;
      }
    }
  });
  return <>{out}</>;
}

/** Number of display equations in some blocks (to continue the numbering across parts of a page). */
export function equationCount(blocks: readonly Block[]): number {
  return blocks.filter((b) => b.t === 'math').length;
}
