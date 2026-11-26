import { hrefFor, hrefResearch } from '../route';
import { Blocks, Spans } from './Blocks';
import { useCatalogue, type MechanismRow } from './catalogue';
import {
  bricks,
  entryHref,
  firstColumn,
  regimeEntry,
  REGIMES,
  roadmapSummary,
  ROADMAP_ENTRY,
  rowsOfBrick,
  rowsOfRegime,
  rowsOfSpecies,
  rowKey,
  sectionBlocks,
  statusCounts,
} from './entries';
import { RowChip, StatusLegend } from './parts';
import { isImplementedStatus } from './status';
import { StatusChip } from './StatusChip';

// Block E of the Explore page (prompt/DESIGN_SPECIFICATION.md Sections 2.1 and 7): a summary of the
// research catalogue. What the solver implements is set apart from what is only research; the three
// regimes, the nine species and the bricks follow, each with its research status, and link to the
// Research section. Every name, sentence and status is read from the generated catalogue.

export function Counts({ rows }: { rows: readonly MechanismRow[] }) {
  return (
    <span className="counts">
      {statusCounts(rows).map((c) => (
        <StatusChip key={c.status} status={c.status} count={c.count} compact />
      ))}
    </span>
  );
}

export function FutureResearch() {
  const cat = useCatalogue();
  const implemented = cat.mechanisms.filter((m) => isImplementedStatus(m.status));
  const research = cat.mechanisms.filter((m) => !isImplementedStatus(m.status));
  const summary = roadmapSummary(cat);
  const futureHead = (cat.entries[ROADMAP_ENTRY]?.blocks ?? []).filter((b) => b.t === 'h' && b.id === 'two-items-among-the-future-phases');
  const future = [...futureHead, ...sectionBlocks(cat.entries[ROADMAP_ENTRY], 'two-items-among-the-future-phases')];
  return (
    <div className="future" data-testid="future-research">
      <div className="split" data-testid="future-split">
        <section className="split__side split__side--implemented" aria-labelledby="future-implemented" data-testid="future-implemented">
          <h3 id="future-implemented" className="split__title">
            Implemented in the solver
          </h3>
          <ul className="split__list">
            {implemented.map((r) => (
              <li key={rowKey(r)}>
                <span className="split__name">{r.name}</span>
                <span className="split__domain">{r.material_domain.replace(/_/g, ' ')} case</span>
                <RowChip row={r} compact />
              </li>
            ))}
          </ul>
        </section>
        <section className="split__side split__side--research" aria-labelledby="future-research-only" data-testid="future-pending">
          <h3 id="future-research-only" className="split__title">
            Research only, not solved
          </h3>
          <p className="split__summary">
            <span className="split__count">{research.length}</span> catalogue rows <Counts rows={research} />
          </p>
        </section>
      </div>

      <h3 className="future__h">Three transport regimes</h3>
      <div className="regimes" data-testid="future-regimes">
        {REGIMES.map((g) => {
          const entry = cat.entries[regimeEntry(g)];
          const rows = rowsOfRegime(cat, g);
          const items = firstColumn(entry, 'mechanisms-to-investigate');
          return (
            <section className="regime" key={g} aria-labelledby={`future-regime-${g}`} data-testid={`future-regime-${g}`}>
              <h4 className="regime__title" id={`future-regime-${g}`}>
                <a href={entryHref(regimeEntry(g))}>{entry?.title ?? g}</a>
              </h4>
              {items.length > 0 ? (
                <ul className="regime__items" aria-label="Mechanisms to investigate">
                  {items.map((it, i) => (
                    <li key={i}>
                      <Spans spans={it} catalogue={cat} />
                    </li>
                  ))}
                </ul>
              ) : null}
              <p className="regime__counts">
                <span className="regime__n">{rows.length} catalogue rows</span> <Counts rows={rows} />
              </p>
            </section>
          );
        })}
      </div>

      <h3 className="future__h">Nine species</h3>
      <ul className="species-strip" data-testid="future-species">
        {cat.species.map((s) => (
          <li key={s.species}>
            <a className="species-strip__link" href={entryHref(s.entry)} data-species={s.species}>
              <span className="species-strip__sym">{s.species}</span>
              <span className="species-strip__name">{s.element}</span>
            </a>
            <Counts rows={rowsOfSpecies(cat, s.species)} />
          </li>
        ))}
      </ul>

      <h3 className="future__h">Brick by brick</h3>
      <ol className="brick-strip" data-testid="future-bricks">
        {bricks(cat).map((b) => {
          const s = summary.get(b);
          return (
            <li key={b} className="brick-strip__item" data-brick={b}>
              <span className="brick-strip__num" aria-hidden="true">
                {b}
              </span>
              <span className="brick-strip__name">
                <span className="visually-hidden">Brick {b}: </span>
                {s ? <Spans spans={s.adds} catalogue={cat} /> : null}
              </span>
              <Counts rows={rowsOfBrick(cat, b)} />
            </li>
          );
        })}
      </ol>

      {future.length > 0 ? (
        <div className="future__planned" data-testid="future-planned">
          <Blocks blocks={future} catalogue={cat} shift={2} />
        </div>
      ) : null}

      <StatusLegend />

      <p className="future__links">
        Full catalogue in <a href={hrefFor('research')}>Research</a>: <a href={hrefResearch('roadmap')}>brick roadmap</a>,{' '}
        <a href={hrefResearch('gallery')}>Materials Studio gallery</a>, <a href={hrefResearch('references')}>references</a>.
      </p>
    </div>
  );
}
