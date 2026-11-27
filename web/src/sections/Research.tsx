import { useEffect, type ReactNode } from 'react';
import { Blocks, headingDomId, Spans } from '../research/Blocks';
import { useCatalogue, type Catalogue, type EntryInfo } from '../research/catalogue';
import {
  bodyBlocks,
  bricks,
  entryHref,
  entryKind,
  entryPathOfSlug,
  firstColumn,
  GALLERY_ENTRY,
  pageReferences,
  regimeEntry,
  REGIMES,
  roadmapSummary,
  ROADMAP_ENTRY,
  rowsForAllSpecies,
  rowsOfBrick,
  rowsOfEntry,
  rowsOfRegime,
  rowsOfSpecies,
  rowKey,
} from '../research/entries';
import { GalleryItem, ReferenceList, RowList, RowRecords, RowTable } from '../research/parts';
import { hrefDocumentation, hrefFor, hrefResearch, useResearchPath } from '../route';
import { SectionHeading, TableBox, useHeadingFocus } from './common';

/**
 * Research section (prompt/DESIGN_SPECIFICATION.md Section 7; web phase 5.5): the research
 * catalogue, read only from the generated catalogue JSON. Pages: the overview (three regimes, nine
 * species, bricks), one page per entry file
 * (species, regime, mechanism), the Materials Studio gallery, the brick roadmap and the reference
 * list. Every live-simulation switch in it is disabled until the Rust solver implements and
 * verifies its mechanism; the statements of limits live in Scientific Documentation.
 */
export function Research() {
  const { page, anchor } = useResearchPath();
  const cat = useCatalogue();
  // A new page starts at its top; an anchor scrolls to its heading.
  useEffect(() => {
    if (anchor) document.getElementById(headingDomId(anchor))?.scrollIntoView({ block: 'start' });
    else if (typeof window.scrollTo === 'function') window.scrollTo(0, 0);
  }, [page, anchor]);

  let view: ReactNode;
  if (page === '') view = <Overview cat={cat} />;
  else if (page === 'gallery') view = <GalleryPage cat={cat} />;
  else if (page === 'roadmap') view = <RoadmapPage cat={cat} />;
  else if (page === 'references') view = <ReferencesPage cat={cat} />;
  else {
    const path = entryPathOfSlug(page);
    const entry = cat.entries[path];
    view = entry ? <EntryPage cat={cat} path={path} entry={entry} key={path} /> : <NotFound page={page} />;
  }
  return (
    <article className="section section--doc research" data-section="research" data-research-page={page || 'overview'}>
      {view}
    </article>
  );
}

function Crumbs({ items }: { items: { label: string; href?: string }[] }) {
  return (
    <nav className="crumbs" aria-label="Research catalogue">
      <ol>
        <li>
          <a href={hrefFor('research')}>Research Catalogue</a>
        </li>
        {items.map((it) => (
          <li key={it.label}>{it.href ? <a href={it.href}>{it.label}</a> : <span aria-current="page">{it.label}</span>}</li>
        ))}
      </ol>
    </nav>
  );
}

function PageTitle({ children }: { children: ReactNode }) {
  const ref = useHeadingFocus();
  return (
    <h1 ref={ref} tabIndex={-1} className="section-head__title research__title">
      {children}
    </h1>
  );
}

function Overview({ cat }: { cat: Catalogue }) {
  const summary = roadmapSummary(cat);
  return (
    <>
      <SectionHeading>Research Catalogue</SectionHeading>

      <div className="overview">
        <div className="overview__main">
          <h2 id="regimes">Three transport regimes</h2>
          {REGIMES.map((g) => {
            const path = regimeEntry(g);
            const entry = cat.entries[path];
            const rows = rowsOfRegime(cat, g);
            const items = firstColumn(entry, 'mechanisms-to-investigate');
            return (
              <section key={g} className="regime regime--wide" aria-labelledby={`ov-regime-${g}`} data-testid={`overview-regime-${g}`}>
                <h3 className="regime__title" id={`ov-regime-${g}`}>
                  <a href={entryHref(path)}>{entry?.title ?? g}</a>
                </h3>
                {items.length > 0 ? (
                  <p className="regime__items-inline">
                    <span className="record__lead">To investigate: </span>
                    {items.map((it, i) => (
                      <span key={i}>
                        {i > 0 ? '; ' : ''}
                        <Spans spans={it} catalogue={cat} />
                      </span>
                    ))}
                  </p>
                ) : null}
                <RowTable rows={rows} label={`Catalogue rows of ${entry?.title ?? g}`} />
              </section>
            );
          })}

          <h2 id="species">Nine species</h2>
          <ul className="species-grid" data-testid="overview-species">
            {cat.species.map((s) => (
              <li key={s.species} className="species-card" data-species={s.species}>
                <a className="species-card__head" href={entryHref(s.entry)}>
                  <span className="species-card__sym">{s.species}</span>
                  <span className="species-card__name">{s.element}</span>
                </a>
                <p className="species-card__summary">{s.summary}</p>
                <p className="species-card__meta">
                  <span className="record__lead">Regimes </span>
                  {s.regimes.join(', ')}
                </p>
              </li>
            ))}
          </ul>

          <h2 id="bricks">Bricks</h2>
          <TableBox label="Bricks of the development sequence">
            <table className="table brick-table" data-testid="overview-bricks">
              <thead>
                <tr>
                  <th scope="col">Brick</th>
                  <th scope="col">Adds</th>
                </tr>
              </thead>
              <tbody>
                {bricks(cat).map((b) => {
                  const s = summary.get(b);
                  return (
                    <tr key={b}>
                      <td className="num-col">{b}</td>
                      <td>{s ? <Spans spans={s.adds} catalogue={cat} /> : null}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </TableBox>
          <nav className="research__links" aria-label="Further catalogue pages">
            <a href={hrefResearch('roadmap')}>Brick roadmap</a>
            <a href={hrefResearch('gallery')}>Materials Studio gallery</a>
            <a href={hrefResearch('references')}>References</a>
          </nav>
        </div>
      </div>
    </>
  );
}

function EntryPage({ cat, path, entry }: { cat: Catalogue; path: string; entry: EntryInfo }) {
  const kind = entryKind(cat, path);
  const body = bodyBlocks(entry);
  const species = cat.species.find((s) => s.entry === path);
  const regime = REGIMES.find((g) => regimeEntry(g) === path);
  const rows =
    kind === 'mechanism' ? rowsOfEntry(cat, path) : kind === 'species' && species ? rowsOfSpecies(cat, species.species) : regime ? rowsOfRegime(cat, regime) : [];
  const general = kind === 'species' ? rowsForAllSpecies(cat) : [];
  // The references of the text, then those of the rows whose records (with their sources) are shown.
  const refs = pageReferences(body, [...(kind === 'mechanism' ? rows.map((r) => r.reference_ids) : []), ...(species ? [species.reference_ids] : [])]);
  const kindLabel = kind === 'species' ? 'Species' : kind === 'regime' ? 'Transport regime' : kind === 'mechanism' ? 'Mechanism' : 'Entry';
  return (
    <div className={`entry entry--${kind}`} data-testid="entry-page" data-entry={path} data-entry-kind={kind}>
      <Crumbs items={[{ label: kindLabel }, { label: entry.title }]} />
      <PageTitle>{entry.title}</PageTitle>
      <div className="entry__layout">
        <aside className="entry__record" aria-label="Catalogue record">
          {species ? (
            <section className="facts" aria-label="Species record" data-testid="species-record">
              <p className="facts__title">Species record</p>
              <dl className="facts__list">
                <div>
                  <dt>Nuclides of interest</dt>
                  <dd>
                    <ul className="nuclides">
                      {species.nuclides.map((n) => {
                        const m = /^(.*?)\s*\(not in register\)$/.exec(n);
                        return (
                          <li key={n} className={m ? 'nuclides__item nuclides__item--absent' : 'nuclides__item'}>
                            {m ? m[1] : n}
                            {m ? <span className="nuclides__note">not in the nuclear-data register</span> : null}
                          </li>
                        );
                      })}
                    </ul>
                  </dd>
                </div>
                <div>
                  <dt>TRISO layers</dt>
                  <dd>{species.layers.join(', ')}</dd>
                </div>
                <div>
                  <dt>Regimes</dt>
                  <dd>{species.regimes.join(', ')}</dd>
                </div>
              </dl>
            </section>
          ) : null}
          <section className="facts" aria-labelledby="record-rows" data-testid="entry-rows">
            <p className="facts__title" id="record-rows">
              {kind === 'species' ? 'Catalogue rows of its mechanisms' : kind === 'regime' ? 'Catalogue rows in this regime' : 'Catalogue rows of this entry'}
            </p>
            {kind === 'regime' || kind === 'species' ? (
              <RowList rows={rows} label={kind === 'regime' ? 'Catalogue rows in this regime' : 'Catalogue rows of this species'} testId="entry-row-list" />
            ) : rows.length > 0 ? (
              <RowRecords rows={rows} catalogue={cat} />
            ) : (
              <p className="facts__note">No catalogue row names this entry.</p>
            )}
            {general.length > 0 ? (
              <>
                <p className="facts__sub">Rows for any species</p>
                <ul className="mini-rows">
                  {general.map((r) => (
                    <li key={rowKey(r)}>
                      {r.entry ? <a href={entryHref(r.entry)}>{r.name}</a> : r.name}
                    </li>
                  ))}
                </ul>
              </>
            ) : null}
          </section>
        </aside>
        <div className="entry__text prose">
          <Blocks blocks={body} catalogue={cat} />
          <h2 className="entry-h entry-h--2" id="r-references">
            References cited on this page
          </h2>
          <ReferenceList ids={refs} catalogue={cat} label="References cited on this page" />
        </div>
      </div>
    </div>
  );
}

const GALLERY_TITLES = {
  screenshot: 'Materials Studio screenshots',
  reconstruction: 'Interactive reconstructions of exported structures',
  placeholder: 'Pending placeholders: planned calculations',
} as const;

function GalleryPage({ cat }: { cat: Catalogue }) {
  const entry = cat.entries[GALLERY_ENTRY];
  const groups = (['screenshot', 'reconstruction', 'placeholder'] as const).map((k) => ({ kind: k, items: cat.gallery.filter((g) => g.kind === k) }));
  const planned = groups.find((g) => g.kind === 'placeholder')?.items.length ?? 0;
  return (
    <div className="gallery-page" data-testid="gallery-page">
      <Crumbs items={[{ label: 'Materials Studio gallery' }]} />
      <PageTitle>{entry?.title ?? 'Materials Studio gallery'}</PageTitle>
      <p className="gallery-page__status" data-testid="gallery-status">
        {planned} planned calculations; no image exists yet. <a href={hrefDocumentation('research-gallery')}>Rules</a>
        {' · '}
        <a href={hrefDocumentation('research-gallery-planned')}>The planned calculations</a>
      </p>
      {groups
        .filter((g) => g.kind !== 'placeholder' && g.items.length > 0)
        .map((g) => (
          <section key={g.kind} className={`gallery-group gallery-group--${g.kind}`} aria-labelledby={`gallery-${g.kind}`} data-testid={`gallery-group-${g.kind}`}>
            <h2 id={`gallery-${g.kind}`}>{GALLERY_TITLES[g.kind]}</h2>
            <div className="gallery-grid">
              {g.items.map((it) => (
                <GalleryItem key={it.item_id} item={it} catalogue={cat} />
              ))}
            </div>
          </section>
        ))}
    </div>
  );
}

function RoadmapPage({ cat }: { cat: Catalogue }) {
  const entry = cat.entries[ROADMAP_ENTRY];
  const summary = roadmapSummary(cat);
  return (
    <div className="roadmap-page" data-testid="roadmap-page">
      <Crumbs items={[{ label: 'Brick roadmap' }]} />
      <PageTitle>{entry?.title ?? 'Brick roadmap'}</PageTitle>
      <section className="bricks" aria-labelledby="bricks-now" data-testid="roadmap-bricks">
        <h2 id="bricks-now">Bricks and their catalogue rows</h2>
        <TableBox label="Bricks of the development sequence and their catalogue rows">
          <table className="table brick-table" data-testid="roadmap-table">
            <thead>
              <tr>
                <th scope="col">Brick</th>
                <th scope="col">Adds</th>
                <th scope="col">Catalogue rows</th>
              </tr>
            </thead>
            <tbody>
              {bricks(cat).map((b) => {
                const rows = rowsOfBrick(cat, b);
                const s = summary.get(b);
                return (
                  <tr key={b} className="brick" data-brick={b}>
                    <th scope="row" className="num-col">
                      {b}
                    </th>
                    <td>{s ? <Spans spans={s.adds} catalogue={cat} /> : null}</td>
                    <td>
                      <ul className="brick__rows">
                        {rows.map((r) => (
                          <li key={rowKey(r)}>
                            {r.entry ? <a href={entryHref(r.entry)}>{r.name}</a> : r.name}
                            {r.species !== 'all' ? ` (${r.species})` : ''}
                          </li>
                        ))}
                      </ul>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </TableBox>      </section>
      <p className="roadmap-page__more" data-testid="roadmap-more">
        <a href={hrefDocumentation('research-roadmap-body')}>Brick narratives</a>
      </p>
    </div>
  );
}

function ReferencesPage({ cat }: { cat: Catalogue }) {
  return (
    <div data-testid="references-page">
      <Crumbs items={[{ label: 'References' }]} />
      <PageTitle>References of the research catalogue</PageTitle>
      <div className="prose">
        <ReferenceList ids={cat.references.map((r) => r.reference_id)} catalogue={cat} label="All references of the research catalogue" />
      </div>
    </div>
  );
}

function NotFound({ page }: { page: string }) {
  return (
    <div data-testid="research-not-found">
      <PageTitle>No catalogue page named {page}</PageTitle>
      <p>
        Go to the <a href={hrefFor('research')}>research catalogue</a>.
      </p>
    </div>
  );
}
