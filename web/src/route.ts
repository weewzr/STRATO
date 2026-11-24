import { useEffect, useState } from 'react';

// Sections of prompt/DESIGN_SPECIFICATION.md Section 2.2, in its order. Research holds the full
// research catalogue from web phase 5.5 and shows a placeholder until then.
export const SECTIONS = [
  { id: 'explore', label: 'Explore' },
  { id: 'compare', label: 'Compare' },
  { id: 'validate', label: 'Validate' },
  { id: 'verify', label: 'Verify' },
  { id: 'research', label: 'Research' },
  { id: 'data', label: 'Data' },
  { id: 'about', label: 'About' },
] as const;

export type SectionId = (typeof SECTIONS)[number]['id'];

export function sectionFromHash(hash: string): SectionId {
  const id = hash.replace(/^#\/?/, '').split(/[/?]/)[0] ?? '';
  return (SECTIONS.find((s) => s.id === id)?.id ?? 'explore') as SectionId;
}

export function hrefFor(id: SectionId): string {
  return `#/${id}`;
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
