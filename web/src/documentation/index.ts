import { about } from './about';
import { compare } from './compare';
import { data } from './data';
import { explore } from './explore';
import { research } from './research';
import { site } from './site';
import type { DocContribution, DocItem, DocSectionId } from './types';
import { validate } from './validate';
import { verify } from './verify';

export { DOC_SECTIONS } from './types';
export type { DocContribution, DocItem, DocSectionId } from './types';

/**
 * One content module per source page. Each page task puts the scientific material that leaves its page
 * into its own module (and edits no other); the order here is the order of the entries in a section.
 */
const CONTRIBUTIONS: DocContribution[] = [site, explore, compare, validate, verify, research, data, about];

/** The entries of a section, in the order of the modules. */
export function itemsOf(section: DocSectionId): DocItem[] {
  return CONTRIBUTIONS.flatMap((c) => c[section] ?? []);
}

/** Every entry id of the page (for the anchors and for the tests). */
export function allItemIds(): string[] {
  return CONTRIBUTIONS.flatMap((c) => Object.values(c).flatMap((items) => (items ?? []).map((i) => i.id)));
}
