// Lookups over the generated research catalogue for the Research section and block E (web phase
// 5.5). Selection and grouping only: every name, status, sentence and reference comes from
// web/src/generated/research_catalogue.json; nothing here is content.
import { hrefResearch } from '../route';
import type { Block, Catalogue, EntryInfo, MechanismRow, Span } from './catalogue';
import { STATUSES, type ResearchStatus } from './status';

export const ENTRY_PREFIX = 'docs/research/';
/** The two overview pages with a view of their own (prompt/TECHNICAL_ARCHITECTURE.md Section 5.1). */
export const GALLERY_ENTRY = `${ENTRY_PREFIX}materials_studio_gallery.md`;
export const ROADMAP_ENTRY = `${ENTRY_PREFIX}roadmap.md`;
/** The three transport regimes (prompt/DESIGN_SPECIFICATION.md Section 7.2), ids as in mechanisms.csv. */
export const REGIMES = ['A', 'B', 'C'] as const;
export type RegimeId = (typeof REGIMES)[number];

export function regimeEntry(regime: RegimeId): string {
  return `${ENTRY_PREFIX}regime-${regime.toLowerCase()}.md`;
}

export function entrySlug(path: string): string {
  return path.replace(/^docs\/research\//, '').replace(/\.md$/, '');
}

export function entryPathOfSlug(slug: string): string {
  return `${ENTRY_PREFIX}${slug}.md`;
}

/** The page of an entry file: the gallery and roadmap have their own views, every other file its slug. */
export function entryPage(path: string): string {
  if (path === GALLERY_ENTRY) return 'gallery';
  if (path === ROADMAP_ENTRY) return 'roadmap';
  return entrySlug(path);
}

export function entryHref(path: string, anchor = ''): string {
  return hrefResearch(entryPage(path), anchor);
}

export type EntryKind = 'species' | 'regime' | 'mechanism' | 'gallery' | 'roadmap' | 'other';

export function entryKind(cat: Catalogue, path: string): EntryKind {
  if (path === GALLERY_ENTRY) return 'gallery';
  if (path === ROADMAP_ENTRY) return 'roadmap';
  if (REGIMES.some((r) => regimeEntry(r) === path)) return 'regime';
  if (cat.species.some((s) => s.entry === path)) return 'species';
  if (cat.mechanisms.some((m) => m.entry === path)) return 'mechanism';
  return 'other';
}

export function rowKey(m: MechanismRow): string {
  return `${m.mechanism_id}|${m.species}|${m.material_domain}`;
}

export function rowsOfEntry(cat: Catalogue, path: string): MechanismRow[] {
  return cat.mechanisms.filter((m) => m.entry === path);
}

/** Rows of one species (its own rows; rows for "all" species are listed separately). */
export function rowsOfSpecies(cat: Catalogue, species: string): MechanismRow[] {
  return cat.mechanisms.filter((m) => m.species === species);
}

export function rowsForAllSpecies(cat: Catalogue): MechanismRow[] {
  return cat.mechanisms.filter((m) => m.species === 'all');
}

export function rowsOfRegime(cat: Catalogue, regime: RegimeId): MechanismRow[] {
  return cat.mechanisms.filter((m) => m.regime.includes(regime));
}

export function rowsOfBrick(cat: Catalogue, brick: number): MechanismRow[] {
  return cat.mechanisms.filter((m) => m.brick === brick);
}

/** The bricks that have rows, in order. */
export function bricks(cat: Catalogue): number[] {
  return [...new Set(cat.mechanisms.map((m) => m.brick).filter((b): b is number => b !== null))].sort((a, b) => a - b);
}

/** Number of rows at each status, in the order of the five levels; statuses without rows are left out. */
export function statusCounts(rows: readonly MechanismRow[]): { status: ResearchStatus; count: number }[] {
  return STATUSES.map((status) => ({ status, count: rows.filter((r) => r.status === status).length })).filter((c) => c.count > 0);
}

/** The distinct statuses of some rows, in the order of the five levels. */
export function distinctStatuses(rows: readonly MechanismRow[]): ResearchStatus[] {
  return statusCounts(rows).map((c) => c.status);
}

export function blocksOf(entry: EntryInfo | undefined): Block[] {
  return entry?.blocks ?? [];
}

/** The blocks after the page title. */
export function bodyBlocks(entry: EntryInfo | undefined): Block[] {
  const b = blocksOf(entry);
  return b[0]?.t === 'h' && b[0].level === 1 ? b.slice(1) : b;
}

/** The blocks between the title and the first second-level heading. */
export function introBlocks(entry: EntryInfo | undefined): Block[] {
  const body = bodyBlocks(entry);
  const end = body.findIndex((b) => b.t === 'h' && b.level <= 2);
  return end < 0 ? body : body.slice(0, end);
}

/** The blocks under one heading (by its id), up to the next heading of the same or a higher level. */
export function sectionBlocks(entry: EntryInfo | undefined, headingId: string): Block[] {
  const b = blocksOf(entry);
  const i = b.findIndex((x) => x.t === 'h' && x.id === headingId);
  if (i < 0) return [];
  const head = b[i] as Extract<Block, { t: 'h' }>;
  const rest = b.slice(i + 1);
  const end = rest.findIndex((x) => x.t === 'h' && x.level <= head.level);
  return end < 0 ? rest : rest.slice(0, end);
}

/** First column of the first table under a heading (for example a regime's "Mechanisms to investigate"). */
export function firstColumn(entry: EntryInfo | undefined, headingId: string): Span[][] {
  const table = sectionBlocks(entry, headingId).find((b) => b.t === 'table');
  return table && table.t === 'table' ? table.rows.map((r) => r[0] ?? []) : [];
}

function spansRefs(spans: readonly Span[], out: string[]): void {
  for (const s of spans) {
    if (s.t === 'ref') out.push(s.id);
    else if ('c' in s) spansRefs(s.c, out);
  }
}

/** Reference ids cited in some blocks, in the order of first citation. */
export function citedRefs(blocks: readonly Block[]): string[] {
  const out: string[] = [];
  for (const b of blocks) {
    if (b.t === 'h' || b.t === 'p') spansRefs(b.c, out);
    else if (b.t === 'ul' || b.t === 'ol') for (const it of b.items) spansRefs(it, out);
    else if (b.t === 'table') {
      for (const c of b.head) spansRefs(c, out);
      for (const r of b.rows) for (const c of r) spansRefs(c, out);
    }
  }
  return [...new Set(out)];
}

/** Every reference of a page: those cited in its text, then those named only in its catalogue rows. */
export function pageReferences(blocks: readonly Block[], rowRefs: readonly string[][]): string[] {
  return [...new Set([...citedRefs(blocks), ...rowRefs.flat()])];
}

/** Plain text of some spans (for titles and accessible names). */
export function spansText(spans: readonly Span[]): string {
  return spans.map((s) => (s.t === 'text' || s.t === 'code' || s.t === 'math' ? s.v : s.t === 'ref' ? '' : spansText(s.c))).join('');
}

/**
 * The roadmap's summary table by brick: what each brick adds and what delivers it (columns 2 and
 * 4 of the table under "Summary" in docs/research/roadmap.md). Its status column is not used: the
 * page shows the statuses of the catalogue rows.
 */
export function roadmapSummary(cat: Catalogue): Map<number, { adds: Span[]; deliveredBy: Span[]; statusText: string }> {
  const out = new Map<number, { adds: Span[]; deliveredBy: Span[]; statusText: string }>();
  const table = sectionBlocks(cat.entries[ROADMAP_ENTRY], 'summary').find((b) => b.t === 'table');
  if (!table || table.t !== 'table') return out;
  for (const r of table.rows) {
    const n = Number(spansText(r[0] ?? []).trim());
    if (Number.isInteger(n)) out.set(n, { adds: r[1] ?? [], deliveredBy: r[3] ?? [], statusText: spansText(r[2] ?? []) });
  }
  return out;
}

/** True when a link_checked cell holds the date on which the link was opened (otherwise it is not checked). */
export function linkCheckedDate(value: string): string | null {
  return /^\d{4}-\d{2}-\d{2}$/.test(value) ? value : null;
}

/** The species named in a gallery cell ("Ag", "Ag;Pd", "Kr;Xe;I" or "none"). */
export function gallerySpecies(cell: string): string[] {
  return cell
    .split(';')
    .map((s) => s.trim())
    .filter((s) => s && s !== 'none');
}
