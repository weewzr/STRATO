// Links from the builder to the BTCS mathematics documents and to the code of the BTCS solver.
import { repoFile } from '../build';

/** The five BTCS documents (docs/mathematics/btcs): derivation, final formulation, pseudocode, traceability, generation, decay and temperature. */
export type DocId = 'd1' | 'd2' | 'd3' | 'd4' | 'd5';

export const DOCS: Record<DocId, { path: string; title: string; short: string }> = {
  d1: { path: 'docs/mathematics/btcs/01_from_ftcs_to_five_layer_btcs.md', title: 'BTCS Document 1, from the explicit scheme to the five-layer BTCS derivation', short: 'D1' },
  d2: { path: 'docs/mathematics/btcs/02_final_formulation.md', title: 'BTCS Document 2, final five-layer formulation', short: 'D2' },
  d3: { path: 'docs/mathematics/btcs/03_rust_pseudocode.md', title: 'BTCS Document 3, Rust-oriented pseudocode', short: 'D3' },
  d4: { path: 'docs/mathematics/btcs/04_equation_to_code_traceability.md', title: 'BTCS Document 4, equation-to-code traceability', short: 'D4' },
  d5: { path: 'docs/mathematics/btcs/05_generation_decay_temperature.md', title: 'BTCS Document 5, generation, decay and temperature-dependent diffusivity', short: 'D5' },
};

/** A section of a BTCS document, named by its heading text exactly as written there. */
export interface DocRef {
  doc: DocId;
  section: string;
}

/**
 * A line of a Rust source file of the BTCS solver (branch feat/btcs-solver). The link test checks
 * that the line holds the symbol, in the working tree when the file is there and otherwise in the
 * branch tip, so a moved definition is found before it is published.
 */
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

/**
 * The number printed beside an equation: a Document 1 number is written D1.n, a Document 5 number D5.n,
 * a Document 2 number as it is (F10). Document 5 numbers its equations (1), (2), ... as Document 1 does.
 */
export function displayTag(doc: DocId, tag: string): string {
  return doc === 'd1' ? `D1.${tag}` : doc === 'd5' ? `D5.${tag}` : tag;
}

/**
 * Code that exists only on a development branch of the solver and is not in the running build (generation,
 * decay and temperature-dependent diffusion, branch feat/results-pipeline). Named by function and the
 * equation it implements, never by line number, and shown as text, not as a link: the file is not on the
 * published branch.
 */
export interface DevCodeRef {
  label: string;
  path: string;
  /** The text that the definition line of the item contains (checked against the branch in the tests). */
  symbol: string;
}
