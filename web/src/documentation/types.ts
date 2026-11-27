import type { ReactNode } from 'react';

/**
 * The sections of Scientific Documentation, in page order. The id is the anchor other pages link to
 * (hrefDocumentation('assumptions')).
 */
export const DOC_SECTIONS = [
  { id: 'assumptions', title: 'Assumptions' },
  { id: 'limitations', title: 'Model limitations' },
  { id: 'derivations', title: 'Derivations' },
  { id: 'verification', title: 'Verification methodology' },
  { id: 'evidence', title: 'Numerical verification evidence' },
  { id: 'interpretation', title: 'Experimental interpretation' },
  { id: 'data', title: 'Parameters, references and downloads' },
  { id: 'statuses', title: 'Research and implementation statuses' },
  { id: 'advanced', title: 'Advanced solver configuration' },
  { id: 'implementation', title: 'Technical implementation and build' },
  { id: 'provenance', title: 'Data provenance and research catalogue' },
] as const;

export type DocSectionId = (typeof DOC_SECTIONS)[number]['id'];

/**
 * One entry of a section: a short heading and its content (short paragraphs, lists or tables). The id is
 * unique over the whole page and is the anchor of the entry: hrefDocumentation(item.id). Use the page it
 * came from as a prefix, for example "verify-balance".
 */
export interface DocItem {
  id: string;
  title: string;
  body: ReactNode;
  /** True for an entry that runs the solver or reads files: its body is built when the entry is opened. */
  lazy?: boolean;
}

/** What one source page contributes: entries per section. A page task fills its own module only. */
export type DocContribution = Partial<Record<DocSectionId, DocItem[]>>;
