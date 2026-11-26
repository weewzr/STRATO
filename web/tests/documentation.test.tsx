// Scientific Documentation: the last page of the navigation, its sections, contents and anchors, and the
// derivation links (against the documents that crates/strato-docsite publishes).
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { allItemIds, DOC_SECTIONS } from '../src/documentation';
import { documentationAnchorFromHash, hrefDocumentation, sectionFromHash, SECTIONS } from '../src/route';
import { Documentation } from '../src/sections/Documentation';

describe('Scientific Documentation in the navigation', () => {
  it('is the last entry, with its own route', () => {
    expect(SECTIONS[SECTIONS.length - 1]).toEqual({ id: 'documentation', label: 'Scientific Documentation' });
    expect(sectionFromHash('#/documentation')).toBe('documentation');
    expect(sectionFromHash('#/documentation/assumptions')).toBe('documentation');
  });

  it('builds and reads anchors', () => {
    expect(hrefDocumentation()).toBe('#/documentation');
    expect(hrefDocumentation('verify-balance')).toBe('#/documentation/verify-balance');
    expect(documentationAnchorFromHash('#/documentation/verify-balance')).toBe('verify-balance');
    expect(documentationAnchorFromHash('#/research/gallery')).toBe('');
  });
});

describe('Scientific Documentation page', () => {
  it('has the seven sections with headings, contents links and anchors', () => {
    expect(DOC_SECTIONS.map((s) => s.title)).toEqual([
      'Assumptions',
      'Model limitations',
      'Derivations',
      'Verification methodology',
      'Experimental interpretation',
      'Technical implementation and build',
      'Data provenance',
    ]);
    render(<Documentation />);
    expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
    const toc = screen.getByRole('navigation', { name: 'Contents of Scientific Documentation' });
    for (const s of DOC_SECTIONS) {
      expect(screen.getByRole('heading', { level: 2, name: s.title })).toBeTruthy();
      expect(within(toc).getByRole('link', { name: s.title }).getAttribute('href')).toBe(`#/documentation/${s.id}`);
      expect(document.getElementById(`doc-${s.id}`), s.id).not.toBeNull();
    }
  });

  it('gives every entry a unique id, a heading, an anchor and a contents link', () => {
    const ids = allItemIds();
    expect(new Set(ids).size).toBe(ids.length);
    for (const s of DOC_SECTIONS) expect(ids, 'an entry id may not be a section id').not.toContain(s.id);
    render(<Documentation />);
    const toc = screen.getByRole('navigation', { name: 'Contents of Scientific Documentation' });
    for (const id of ids) {
      const el = document.getElementById(`doc-${id}`);
      expect(el, id).not.toBeNull();
      expect(within(el as HTMLElement).getAllByRole('heading', { level: 3 }).length, id).toBeGreaterThan(0);
      expect(
        within(toc)
          .getAllByRole('link')
          .some((a) => a.getAttribute('href') === `#/documentation/${id}`),
        id,
      ).toBe(true);
    }
  });

  it('holds the build, solver version and source paths, which no other page shows', () => {
    render(<Documentation />);
    const build = within(screen.getByTestId('documentation-build'));
    expect(build.getByText('Site build')).toBeTruthy();
    expect(build.getByText('Solver version')).toBeTruthy();
    expect(within(screen.getByTestId('documentation-source')).getByText('src/')).toBeTruthy();
  });

  it('links only derivation pages that crates/strato-docsite publishes, each beside its source', () => {
    const site = readFileSync(join(process.cwd(), '..', 'crates', 'strato-docsite', 'src', 'site.rs'), 'utf8');
    const published = [...site.matchAll(/src: "([^"]+\.md)"/g)].map((m) => m[1]);
    expect(published.length).toBeGreaterThan(0);
    render(<Documentation />);
    const hrefs = within(screen.getByTestId('documentation-derivations'))
      .getAllByRole('link')
      .map((a) => a.getAttribute('href') ?? '');
    const pages = hrefs.filter((h) => h.endsWith('.html'));
    expect(pages.length).toBeGreaterThan(0);
    for (const h of pages) {
      expect(h.startsWith(import.meta.env.BASE_URL), h).toBe(true);
      const src = h.slice(import.meta.env.BASE_URL.length).replace(/\.html$/, '.md');
      expect(published, h).toContain(src);
      expect(hrefs.some((x) => x.endsWith(`/${src}`)), `${src} is linked on GitHub too`).toBe(true);
    }
  });
});
