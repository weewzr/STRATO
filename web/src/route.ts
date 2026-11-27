import { useEffect, useState } from 'react';

// The four pages of the site (prompt/DESIGN_SPECIFICATION.md Section 12), in navigation order. Research
// Catalogue holds the research catalogue with its own pages under #/research/; Scientific Documentation is
// always the last entry.
export const SECTIONS = [
  { id: 'explore', label: 'Explore' },
  { id: 'basis', label: 'Basis' },
  { id: 'research', label: 'Research Catalogue' },
  { id: 'documentation', label: 'Scientific Documentation' },
] as const;

export type SectionId = (typeof SECTIONS)[number]['id'];

/**
 * Addresses of the earlier layout and the place that now holds their content (an address with a sub-path,
 * such as #/verify/balance, goes to the same place).
 */
export const LEGACY_ADDRESSES: Record<string, string> = {
  compare: '#/documentation/evidence-comparison',
  validate: '#/documentation/validate-status',
  verify: '#/documentation/evidence-verification',
  data: '#/documentation/evidence-parameters',
  about: '#/basis',
};

/** The address that replaces an old one, or null when the hash is current. */
export function redirectFromHash(hash: string): string | null {
  const id = hash.replace(/^#\/?/, '').split(/[/?]/)[0] ?? '';
  return Object.prototype.hasOwnProperty.call(LEGACY_ADDRESSES, id) ? (LEGACY_ADDRESSES[id] as string) : null;
}

export function sectionFromHash(hash: string): SectionId {
  const target = redirectFromHash(hash);
  const id = (target ?? hash).replace(/^#\/?/, '').split(/[/?]/)[0] ?? '';
  return (SECTIONS.find((s) => s.id === id)?.id ?? 'explore') as SectionId;
}
/** The address of a page. An address of the earlier layout is accepted and redirects. */
export function hrefFor(id: SectionId | keyof typeof LEGACY_ADDRESSES): string {
  return `#/${id}`;
}

/**
 * Page inside the Research section: `#/research/<page>/<anchor>`. The page is "" (overview),
 * "gallery", "roadmap", "references" or the slug of an entry file of docs/research/ (its name
 * without ".md"); the anchor names a heading of that entry.
 */
export function researchPathFromHash(hash: string): { page: string; anchor: string } {
  const parts = hash.replace(/^#\/?/, '').split('?')[0]?.split('/') ?? [];
  if (parts[0] !== 'research') return { page: '', anchor: '' };
  return { page: decodeURIComponent(parts[1] ?? ''), anchor: decodeURIComponent(parts[2] ?? '') };
}

export function hrefResearch(page = '', anchor = ''): string {
  return `#/research${page ? `/${encodeURIComponent(page)}` : ''}${anchor ? `/${encodeURIComponent(anchor)}` : ''}`;
}

/**
 * Anchor inside Scientific Documentation: `#/documentation/<anchor>`. The anchor is the id of a section
 * (for example "assumptions") or of an entry of one (src/documentation/); other pages link to it with
 * hrefDocumentation.
 */
export function documentationAnchorFromHash(hash: string): string {
  const parts = hash.replace(/^#\/?/, '').split('?')[0]?.split('/') ?? [];
  return parts[0] === 'documentation' ? decodeURIComponent(parts[1] ?? '') : '';
}

export function hrefDocumentation(anchor = ''): string {
  return `#/documentation${anchor ? `/${encodeURIComponent(anchor)}` : ''}`;
}

/** Current Scientific Documentation anchor from location.hash; updates on hashchange. */
export function useDocumentationAnchor(): string {
  const [anchor, setAnchor] = useState(() => documentationAnchorFromHash(window.location.hash));
  useEffect(() => {
    const on = () => setAnchor(documentationAnchorFromHash(window.location.hash));
    window.addEventListener('hashchange', on);
    return () => window.removeEventListener('hashchange', on);
  }, []);
  return anchor;
}

/** Current Research page from location.hash; updates on hashchange. */
export function useResearchPath(): { page: string; anchor: string } {
  const [path, setPath] = useState(() => researchPathFromHash(window.location.hash));
  useEffect(() => {
    const on = () => setPath(researchPathFromHash(window.location.hash));
    window.addEventListener('hashchange', on);
    return () => window.removeEventListener('hashchange', on);
  }, []);
  return path;
}

/** Current section from location.hash; updates on hashchange. */
export function useSection(): SectionId {
  const [section, setSection] = useState<SectionId>(() => sectionFromHash(window.location.hash));
  useEffect(() => {
    // An address of the earlier layout is replaced by the address that now holds its content.
    const on = () => {
      const target = redirectFromHash(window.location.hash);
      if (target) window.location.replace(target);
      setSection(sectionFromHash(window.location.hash));
    };
    on();
    window.addEventListener('hashchange', on);
    return () => window.removeEventListener('hashchange', on);
  }, []);
  return section;
}
