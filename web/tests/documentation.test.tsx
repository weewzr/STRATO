// Scientific Documentation: the last page of the navigation, its sections, contents and anchors, and the
// derivation links (against the documents that crates/strato-docsite publishes).
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { act, fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { allItemIds, DOC_SECTIONS } from '../src/documentation';
import { documentationAnchorFromHash, hrefDocumentation, sectionFromHash, SECTIONS } from '../src/route';
import { Documentation } from '../src/sections/Documentation';

describe('Scientific Documentation in the navigation', () => {
  it('has exactly four entries, in the decided order', () => {
    expect(SECTIONS.map((s) => s.label)).toEqual(['Explore', 'Basis', 'Research Catalogue', 'Scientific Documentation']);
  });

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
  it('has its sections with headings, contents links and anchors', () => {
    expect(DOC_SECTIONS.map((s) => s.title)).toEqual([
      'Assumptions',
      'Model limitations',
      'Derivations',
      'Verification methodology',
      'Numerical verification evidence',
      'Experimental interpretation',
      'Parameters, references and downloads',
      'Research and implementation statuses',
      'Advanced solver configuration',
      'Technical implementation and build',
      'Data provenance and research catalogue',
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

  it('holds the statuses, the references, the downloads and the mount point of the advanced solver configuration', () => {
    render(<Documentation />);
    expect(within(screen.getByTestId('status-table')).getAllByRole('row').length).toBeGreaterThan(1);
    expect(screen.getByTestId('downloads-evidence')).toBeTruthy();
    expect(screen.getByTestId('references-evidence')).toBeTruthy();
    expect(screen.getByTestId('r2-reasons').textContent).toContain('missing_conditions');
    const mount = screen.getByTestId('advanced-solver-mount');
    expect(mount.getAttribute('data-mount')).toBe('advanced-solver-configuration');
    expect(mount.childElementCount).toBe(0);
  });

  it('builds an entry that runs the solver only when it is opened', async () => {
    render(<Documentation />);
    const item = screen.getByTestId('doc-item-evidence-comparison') as HTMLDetailsElement;
    expect(item.open).toBe(false);
    expect(screen.queryByTestId('comparison-evidence')).toBeNull();
    await act(async () => {
      item.open = true;
      fireEvent(item, new Event('toggle'));
    });
    expect(screen.getByTestId('comparison-evidence')).toBeTruthy();
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

describe('Scientific Documentation filter', () => {
  it('filters the entries by typed text, by keyboard, and Escape shows them all again', async () => {
    const user = userEvent.setup();
    render(<Documentation />);
    const total = allItemIds().length;
    const filter = screen.getByTestId('doc-filter');
    await user.tab();
    await user.click(filter);
    expect(document.activeElement).toBe(filter);
    await user.keyboard('Sturm');
    const shown = Array.from(document.querySelectorAll<HTMLElement>('[data-doc-item]')).filter((e) => !e.hidden);
    expect(shown.length).toBeGreaterThan(0);
    expect(shown.length).toBeLessThan(total);
    expect(screen.getByTestId('doc-filter-count').textContent).toBe(`${shown.length} of ${total} entries`);
    await user.keyboard('qqqzzzxx');
    expect(screen.getByTestId('doc-filter-count').textContent).toBe('No entry contains this text.');
    expect(screen.getByTestId('doc-section-assumptions').hidden).toBe(true);
    await user.keyboard('{Escape}');
    expect(Array.from(document.querySelectorAll<HTMLElement>('[data-doc-item]')).every((e) => !e.hidden)).toBe(true);
    expect((filter as HTMLInputElement).value).toBe('');
  });

});
