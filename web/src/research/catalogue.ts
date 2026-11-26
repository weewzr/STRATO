// Typed access to the generated research catalogue (web/src/generated/research_catalogue.json,
// written by strato-webtools catalogue from data/research/*.csv). Every research status
// the page shows comes from here.
import { createContext, useContext } from 'react';
import catalogueJson from '../generated/research_catalogue.json';
import { isStatus, type ResearchStatus } from './status';

export interface MechanismRow {
  mechanism_id: string;
  name: string;
  regime: string[];
  brick: number | null;
  builder_stage: number | null;
  species: string;
  material_domain: string;
  status: ResearchStatus;
  status_basis: string;
  evidence: string[];
  validation_ids: string[];
  equation_ids: string[];
  coupled: boolean;
  solver_capability: string;
  required_parameters: string[];
  missing_work: string;
  reference_ids: string[];
  entry: string;
  status_date: string;
}

export interface ReferenceRow {
  reference_id: string;
  citation: string;
  doi_or_url: string;
  link_checked: string;
  register_link: string;
  notes: string;
}

export interface SpeciesRow {
  species: string;
  element: string;
  nuclides: string[];
  layers: string[];
  regimes: string[];
  summary: string;
  reference_ids: string[];
  entry: string;
}

export type GalleryKind = 'screenshot' | 'reconstruction' | 'placeholder';

export interface GalleryRow {
  item_id: string;
  kind: GalleryKind;
  title: string;
  caption: string;
  alt_text: string;
  material: string;
  species: string;
  method: string;
  conditions: string;
  provenance: string;
  file: string;
  status: ResearchStatus;
  mechanism_ids: string[];
  reference_ids: string[];
}

/** Inline spans of an entry text, as the generator writes them (display data; nothing to parse). */
export type Span =
  | { t: 'text'; v: string }
  | { t: 'code'; v: string }
  | { t: 'math'; v: string }
  | { t: 'ref'; id: string }
  | { t: 'strong'; c: Span[] }
  | { t: 'em'; c: Span[] }
  | { t: 'link'; href?: string; entry?: string; anchor?: string; c: Span[] };

export type Block =
  | { t: 'h'; level: number; id: string; c: Span[] }
  | { t: 'p'; c: Span[] }
  | { t: 'ul'; items: Span[][] }
  | { t: 'ol'; items: Span[][]; start: number }
  | { t: 'table'; head: Span[][]; rows: Span[][][] }
  | { t: 'math'; tex: string };

export interface EntryInfo {
  path: string;
  title: string;
  concept_equations: string[];
  /** The entry text as blocks; absent only in older test fixtures. */
  blocks?: Block[];
}

export interface Catalogue {
  commit: string;
  generator: string;
  inputs: { path: string; sha256: string }[];
  tables_absent: string[];
  mechanisms: MechanismRow[];
  references: ReferenceRow[];
  entries: Record<string, EntryInfo>;
  species: SpeciesRow[];
  gallery: GalleryRow[];
  referenceLabels: Record<string, string>;
}

const GALLERY_KINDS: readonly string[] = ['screenshot', 'reconstruction', 'placeholder'];

/** Builds a typed catalogue from the generated JSON (or a test fixture of the same shape). */
export function parseCatalogue(raw: unknown): Catalogue {
  const doc = raw as {
    commit: string;
    generator: string;
    inputs: { path: string; sha256: string }[];
    tables_absent: string[];
    data: {
      mechanisms: Record<string, unknown>[];
      references: ReferenceRow[];
      entries: Record<string, EntryInfo>;
      species?: SpeciesRow[];
      gallery?: Record<string, unknown>[];
      reference_labels?: Record<string, string>;
    };
  };
  const mechanisms = doc.data.mechanisms.map((m) => {
    if (!isStatus(m.status)) throw new Error(`research catalogue: unknown status for ${String(m.mechanism_id)}`);
    return m as unknown as MechanismRow;
  });
  const gallery = (doc.data.gallery ?? []).map((g) => {
    if (!isStatus(g.status)) throw new Error(`research catalogue: unknown status for gallery item ${String(g.item_id)}`);
    if (typeof g.kind !== 'string' || !GALLERY_KINDS.includes(g.kind)) throw new Error(`research catalogue: unknown kind for gallery item ${String(g.item_id)}`);
    return g as unknown as GalleryRow;
  });
  return {
    commit: doc.commit,
    generator: doc.generator,
    inputs: doc.inputs,
    tables_absent: doc.tables_absent,
    mechanisms,
    references: doc.data.references,
    entries: doc.data.entries,
    species: doc.data.species ?? [],
    gallery,
    referenceLabels: doc.data.reference_labels ?? {},
  };
}

export const CATALOGUE: Catalogue = parseCatalogue(catalogueJson);

/** The catalogue row of a mechanism; for a mechanism with several species or domains, the first row. */
export function mechanismRow(cat: Catalogue, id: string): MechanismRow | undefined {
  return cat.mechanisms.find((m) => m.mechanism_id === id);
}

export function statusOf(cat: Catalogue, id: string): ResearchStatus | undefined {
  return mechanismRow(cat, id)?.status;
}

export function referenceRow(cat: Catalogue, id: string): ReferenceRow | undefined {
  return cat.references.find((r) => r.reference_id === id);
}

export const CatalogueContext = createContext<Catalogue>(CATALOGUE);

export function useCatalogue(): Catalogue {
  return useContext(CatalogueContext);
}
