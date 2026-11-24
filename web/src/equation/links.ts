// Links from the builder to the derivation documents and to the code, at the build commit.
import { repoFile } from '../build';

export type DocId = 'v1' | 'p2' | 'pseudo';

export const DOCS: Record<DocId, { path: string; title: string }> = {
  v1: { path: 'docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md', title: 'v1.0 full mathematical derivation' },
  p2: { path: 'docs/mathematics/STRATO_Phase2_Derivations.md', title: 'Phase 2 derivations' },
  pseudo: { path: 'docs/mathematics/STRATO_Phase2_Pseudocode.md', title: 'Phase 2 pseudocode' },
};

/** A section of a derivation document, named by its heading text exactly as written there. */
export interface DocRef {
  doc: DocId;
  section: string;
}

/** A line of the Rust source at tag v1.0.1 (frozen); the link test checks that the line holds the symbol. */
export interface CodeRef {
  path: string;
  line: number;
  symbol: string;
}

/** GitHub's heading anchor: lower case, punctuation removed, spaces to hyphens. */
export function headingSlug(heading: string): string {
  return heading
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s_-]/gu, '')
    .replace(/\s/g, '-');
}

export function docHref(ref: DocRef): string {
  return `${repoFile(DOCS[ref.doc].path)}#${headingSlug(ref.section)}`;
}

export function codeHref(ref: CodeRef): string {
  return `${repoFile(ref.path)}#L${ref.line}`;
}
