import { useEffect, useState } from 'react';

// Sections of prompt/DESIGN_SPECIFICATION.md Section 2.2, in its order. Research holds the full
// research catalogue (web phase 5.5), with its own pages under #/research/.
export const SECTIONS = [
  { id: 'explore', label: 'Explore' },
  { id: 'compare', label: 'Compare' },
  { id: 'validate', label: 'Validate' },
  { id: 'verify', label: 'Verify' },
  { id: 'research', label: 'Research' },
  { id: 'data', label: 'Data' },
  { id: 'about', label: 'About' },
  // Always the last entry: the one page for assumptions, limitations, derivations, verification
  // methodology, experimental interpretation and technical implementation (minimal-text rules).
  { id: 'documentation', label: 'Scientific Documentation' },
] as const;

export type SectionId = (typeof SECTIONS)[number]['id'];

export function sectionFromHash(hash: string): SectionId {
  const id = hash.replace(/^#\/?/, '').split(/[/?]/)[0] ?? '';
  return (SECTIONS.find((s) => s.id === id)?.id ?? 'explore') as SectionId;
}

export function hrefFor(id: SectionId): string {
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
    const on = () => setSection(sectionFromHash(window.location.hash));
    window.addEventListener('hashchange', on);
    return () => window.removeEventListener('hashchange', on);
  }, []);
  return section;
}
