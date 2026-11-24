// Typed access to the generated research catalogue (web/src/generated/research_catalogue.json,
// written by scripts/web/generate_data.py catalogue from data/research/*.csv). Every research status
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

export interface EntryInfo {
  path: string;
  title: string;
  concept_equations: string[];
}

export interface Catalogue {
  commit: string;
  generator: string;
  inputs: { path: string; sha256: string }[];
  tables_absent: string[];
  mechanisms: MechanismRow[];
  references: ReferenceRow[];
  entries: Record<string, EntryInfo>;
}

/** Builds a typed catalogue from the generated JSON (or a test fixture of the same shape). */
export function parseCatalogue(raw: unknown): Catalogue {
  const doc = raw as {
    commit: string;
    generator: string;
    inputs: { path: string; sha256: string }[];
    tables_absent: string[];
    data: { mechanisms: Record<string, unknown>[]; references: ReferenceRow[]; entries: Record<string, EntryInfo> };
  };
  const mechanisms = doc.data.mechanisms.map((m) => {
    if (!isStatus(m.status)) throw new Error(`research catalogue: unknown status for ${String(m.mechanism_id)}`);
    return m as unknown as MechanismRow;
  });
  return {
    commit: doc.commit,
    generator: doc.generator,
    inputs: doc.inputs,
    tables_absent: doc.tables_absent,
    mechanisms,
    references: doc.data.references,
    entries: doc.data.entries,
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
