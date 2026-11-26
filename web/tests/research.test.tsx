// Research catalogue interface (web phase 5.5; prompt/TESTING_AND_VALIDATION.md Section 3, row 5.5,
// and Sections 3.1 and 3.2): every page of the Research section and block E render from the
// generated catalogue; every status is one of the five words and equals its row; no pending
// mechanism has an enabled switch; no entry lacks its references; gallery placeholders show no image.
import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import catalogueJson from '../src/generated/research_catalogue.json';
import { App } from '../src/App';
import { isRowAvailable } from '../src/equation/availability';
import { CATALOGUE, CatalogueContext, parseCatalogue, type Catalogue, type GalleryRow, type MechanismRow } from '../src/research/catalogue';
import {
  bodyBlocks,
  citedRefs,
  entryPage,
  GALLERY_ENTRY,
  regimeEntry,
  REGIMES,
  roadmapSummary,
  ROADMAP_ENTRY,
  rowKey,
  rowsOfBrick,
  sectionBlocks,
} from '../src/research/entries';
import { FutureResearch } from '../src/research/FutureResearch';
import { GalleryItem } from '../src/research/parts';
import { pathLabel } from '../src/research/PathLink';
import { STATUS_INFO, STATUSES, type ResearchStatus } from '../src/research/status';
import { Documentation } from '../src/sections/Documentation';
import { Research } from '../src/sections/Research';
import { useStrato } from '../src/state/store';

const SPECIES_HEADINGS = [
  'Why it matters',
  'Relevant TRISO layers',
  'Candidate release mechanisms',
  'Current evidence',
  'Proposed governing-equation extensions',
  'Potential Materials Studio calculations',
  'Experimental comparison opportunities',
  'Research limitations',
  'Implementation status',
];
const MECHANISM_HEADINGS = [
  'Proposed physics',
  'Candidate governing equations (research concepts)',
  'Required parameters',
  'Supporting literature',
  'Missing calculations or experiments',
  'Interactions with other regimes',
];

const ALL_PAGES = ['', 'gallery', 'roadmap', 'references', ...Object.keys(CATALOGUE.entries).map(entryPage).filter((p) => p !== 'gallery' && p !== 'roadmap')];

function renderPage(page: string, cat: Catalogue = CATALOGUE) {
  window.location.hash = `#/research${page ? `/${page}` : ''}`;
  return render(
    <CatalogueContext.Provider value={cat}>
      <Research />
    </CatalogueContext.Provider>,
  );
}

function rowOf(key: string): MechanismRow | undefined {
  return CATALOGUE.mechanisms.find((m) => rowKey(m) === key);
}

function cloneCatalogue(): { data: Record<string, unknown> } & Record<string, unknown> {
  return JSON.parse(JSON.stringify(catalogueJson)) as { data: Record<string, unknown> } & Record<string, unknown>;
}

describe('the generated catalogue holds the research content', () => {
  it('has the nine species, the three regime pages, the gallery and the roadmap', () => {
    expect(CATALOGUE.species.map((s) => s.species)).toEqual(['Ag', 'Pd', 'Cs', 'Sr', 'Eu', 'I', 'Ru', 'Xe', 'Kr']);
    for (const g of REGIMES) expect(CATALOGUE.entries[regimeEntry(g)]?.blocks?.length ?? 0).toBeGreaterThan(0);
    expect(CATALOGUE.entries[GALLERY_ENTRY]).toBeTruthy();
    expect(CATALOGUE.entries[ROADMAP_ENTRY]).toBeTruthy();
    expect(CATALOGUE.gallery.length).toBeGreaterThan(0);
  });

  it('holds only the five status words, in the mechanism rows and the gallery rows', () => {
    for (const m of CATALOGUE.mechanisms) expect(STATUSES).toContain(m.status);
    for (const g of CATALOGUE.gallery) expect(STATUSES).toContain(g.status);
  });

  it('rejects a catalogue whose gallery holds a status outside the five words or an unknown kind', () => {
    for (const bad of ['Verified', 'PLANNED', 'pending', '']) {
      const raw = cloneCatalogue();
      (raw.data.gallery as { status: string }[])[0]!.status = bad;
      expect(() => parseCatalogue(raw), bad).toThrow(/unknown status/);
    }
    const raw = cloneCatalogue();
    (raw.data.gallery as { kind: string }[])[0]!.kind = 'mockup';
    expect(() => parseCatalogue(raw)).toThrow(/unknown kind/);
  });

  it('marks nothing as validated and only the five-layer diffusion row as verified (the evidence of 2026-11-25)', () => {
    expect(CATALOGUE.mechanisms.filter((m) => m.status === 'VALIDATED')).toEqual([]);
    expect(CATALOGUE.mechanisms.filter((m) => m.status === 'VERIFIED').map((m) => m.mechanism_id)).toEqual(['diffusion_btcs_layered']);
    expect(CATALOGUE.mechanisms.filter((m) => m.brick !== null && m.brick >= 4).every((m) => m.status === 'PENDING')).toBe(true);
  });
});

describe('every catalogue entry carries its references', () => {
  const refIds = new Set(CATALOGUE.references.map((r) => r.reference_id));

  it('names at least one known reference in every row that has an entry, every species row and every gallery row', () => {
    const lacking: string[] = [];
    for (const m of CATALOGUE.mechanisms.filter((x) => x.entry)) if (m.reference_ids.length === 0) lacking.push(rowKey(m));
    for (const s of CATALOGUE.species) if (s.reference_ids.length === 0) lacking.push(s.species);
    for (const g of CATALOGUE.gallery) if (g.reference_ids.length === 0) lacking.push(g.item_id);
    expect(lacking).toEqual([]);
    const unknown = [...CATALOGUE.mechanisms.flatMap((m) => m.reference_ids), ...CATALOGUE.species.flatMap((s) => s.reference_ids), ...CATALOGUE.gallery.flatMap((g) => g.reference_ids)].filter(
      (id) => !refIds.has(id),
    );
    expect(unknown).toEqual([]);
  });

  it('cites at least one reference in the text of every entry file', () => {
    for (const [path, e] of Object.entries(CATALOGUE.entries)) expect(citedRefs(e.blocks ?? []).length, path).toBeGreaterThan(0);
  });

  it('lists on each entry page every reference its text and rows name, and links every citation to its source', () => {
    for (const [path, entry] of Object.entries(CATALOGUE.entries)) {
      const page = entryPage(path);
      if (page === 'gallery' || page === 'roadmap') continue;
      const { unmount } = renderPage(page);
      const listed = Array.from(screen.getByTestId('reference-list').querySelectorAll<HTMLElement>('[data-ref-id]')).map((l) => l.dataset.refId);
      const species = CATALOGUE.species.find((s) => s.entry === path);
      // Rows whose records (with their sources) are shown: those of a mechanism entry.
      const rows = CATALOGUE.mechanisms.filter((m) => m.entry === path);
      const expected = new Set([...citedRefs(bodyBlocks(entry)), ...rows.flatMap((r) => r.reference_ids), ...(species?.reference_ids ?? [])]);
      expect(new Set(listed), path).toEqual(expected);
      for (const a of Array.from(document.querySelectorAll<HTMLAnchorElement>('[data-testid="cite-link"]'))) {
        const id = a.closest<HTMLElement>('[data-ref]')?.dataset.ref ?? '';
        expect(a.getAttribute('href'), `${path} ${id}`).toBe(CATALOGUE.references.find((r) => r.reference_id === id)?.doi_or_url);
      }
      expect(document.querySelectorAll('.cite--missing').length, path).toBe(0);
      unmount();
    }
  });

  it('renders every [ref:...] of an entry as a link', () => {
    const path = 'docs/research/species-ag.md';
    renderPage(entryPage(path));
    const source = JSON.stringify(CATALOGUE.entries[path]?.blocks);
    const inText = (source.match(/"t":"ref"/g) ?? []).length;
    const text = screen.getByTestId('entry-page').querySelector('.entry__text') as HTMLElement;
    expect(text.querySelectorAll('[data-testid="cite-link"]').length).toBe(inText);
  });

  it('marks a link that was not opened, and dates the ones that were', () => {
    renderPage('references');
    const items = Array.from(document.querySelectorAll<HTMLElement>('[data-ref-id]'));
    expect(items).toHaveLength(CATALOGUE.references.length);
    for (const li of items) {
      const r = CATALOGUE.references.find((x) => x.reference_id === li.dataset.refId)!;
      const mark = li.querySelector<HTMLElement>('[data-link-checked]')!;
      expect(mark.dataset.linkChecked, r.reference_id).toBe(/^\d{4}-\d{2}-\d{2}$/.test(r.link_checked) ? 'date' : 'no');
      expect(mark.textContent).toContain(r.link_checked);
    }
  });

  it('fails when an entry text cites a reference that is not in the list', () => {
    const raw = cloneCatalogue();
    const refs = raw.data.references as { reference_id: string }[];
    raw.data.references = refs.filter((r) => r.reference_id !== 'demkowicz2016');
    renderPage('species-ag', parseCatalogue(raw));
    expect(document.querySelectorAll('.cite--missing').length).toBeGreaterThan(0);
  });
});

describe('species, regime and mechanism pages', () => {
  it('render the nine species entries with every heading of the format, in order', () => {
    for (const s of CATALOGUE.species) {
      const { unmount } = renderPage(entryPage(s.entry));
      const heads = Array.from(document.querySelectorAll('.entry__text h2')).map((h) => h.textContent);
      expect(heads.filter((h) => SPECIES_HEADINGS.includes(h ?? '')), s.species).toEqual(SPECIES_HEADINGS);
      expect(screen.getByTestId('species-record')).toBeTruthy();
      unmount();
    }
  });

  it('render the three regimes and every mechanism entry with the headings of its format', () => {
    for (const g of REGIMES) {
      const { unmount } = renderPage(entryPage(regimeEntry(g)));
      expect(screen.getByRole('heading', { level: 1 }).textContent).toBe(CATALOGUE.entries[regimeEntry(g)]?.title);
      unmount();
    }
    const mechEntries = [...new Set(CATALOGUE.mechanisms.map((m) => m.entry).filter(Boolean))];
    expect(mechEntries.length).toBeGreaterThan(0);
    for (const path of mechEntries) {
      const { unmount } = renderPage(entryPage(path));
      const heads = Array.from(document.querySelectorAll('.entry__text h2')).map((h) => h.textContent ?? '');
      expect(heads.filter((h) => MECHANISM_HEADINGS.includes(h)), path).toEqual(MECHANISM_HEADINGS);
      unmount();
    }
  });

  it('typesets every candidate equation with KaTeX and labels it as a research concept, numbered', () => {
    let total = 0;
    for (const page of ALL_PAGES) {
      const { unmount } = renderPage(page);
      expect(document.querySelectorAll('[data-testid="tex-failed"]').length, page).toBe(0);
      const eqs = Array.from(document.querySelectorAll<HTMLElement>('[data-testid="concept-equation"]'));
      eqs.forEach((e, i) => {
        expect(e.querySelector('.katex-display'), page).not.toBeNull();
        expect(e.querySelector('figcaption')?.textContent).toMatch(/^Research concept, not solved by STRATO/);
        expect(e.dataset.conceptNumber).toBe(String(i + 1));
      });
      total += eqs.length;
      unmount();
    }
    const inCatalogue = Object.values(CATALOGUE.entries)
      .filter((e) => e.path !== GALLERY_ENTRY)
      .reduce((n, e) => n + (e.blocks ?? []).filter((b) => b.t === 'math').length, 0);
    expect(total).toBe(inCatalogue);
  });

  it('every link to another entry opens a page of the Research section', () => {
    for (const page of ALL_PAGES) {
      const { unmount } = renderPage(page);
      for (const a of Array.from(document.querySelectorAll<HTMLAnchorElement>('a[data-entry-link]'))) {
        expect(a.getAttribute('href'), page).toMatch(/^#\/research\/[\w-]+(\/[\w-]+)?$/);
      }
      unmount();
    }
  });
});

describe('statuses shown equal the catalogue, as word and shape', () => {
  it('shows only the five words, each chip of a row equal to that row, on every Research page and in block E', () => {
    const check = (where: string) => {
      const chips = Array.from(document.querySelectorAll<HTMLElement>('.status-chip'));
      if (where !== 'references') expect(chips.length, where).toBeGreaterThan(0);
      for (const c of chips) {
        const word = c.querySelector('.status-chip__word')?.textContent ?? '';
        expect(STATUSES as readonly string[], `${where}: ${word}`).toContain(word);
        expect(c.dataset.status).toBe(word);
        expect(c.querySelector('svg.status-chip__shape'), where).not.toBeNull();
        if (c.dataset.statusRow) expect(rowOf(c.dataset.statusRow)?.status, `${where}: ${c.dataset.statusRow}`).toBe(word);
      }
    };
    for (const page of ALL_PAGES) {
      const { unmount } = renderPage(page);
      check(page || 'overview');
      unmount();
    }
    render(<FutureResearch />);
    check('block E');
  });

  it('counts the rows of each status correctly in the summaries', () => {
    render(<FutureResearch />);
    const bricks = screen.getByTestId('future-bricks');
    for (const li of Array.from(bricks.querySelectorAll<HTMLElement>('[data-brick]'))) {
      const rows = rowsOfBrick(CATALOGUE, Number(li.dataset.brick));
      const shown = Array.from(li.querySelectorAll<HTMLElement>('[data-status-count]')).map((c) => [c.dataset.status, Number(c.dataset.statusCount)]);
      const expected = STATUSES.map((s) => [s, rows.filter((r) => r.status === s).length]).filter(([, n]) => (n as number) > 0);
      expect(shown, `brick ${li.dataset.brick}`).toEqual(expected);
    }
  });

  // The definitions of the five statuses moved from the legend to Scientific Documentation (minimal-text
  // rules): the legend is one row of the five words, the definitions are printed once, in the documentation.
  it('shows a legend of the five statuses as one row, with a link to their definitions', () => {
    renderPage('');
    const legend = screen.getByTestId('status-legend');
    const words = Array.from(legend.querySelectorAll('.status-chip__word')).map((w) => w.textContent);
    expect(words).toEqual([...STATUSES]);
    expect(legend.querySelectorAll('dd')).toHaveLength(0);
    expect(within(legend).getByRole('link', { name: 'Definitions' }).getAttribute('href')).toBe('#/documentation/research-status-scale');
  });

  it('defines all five statuses once, in Scientific Documentation', () => {
    render(<Documentation />);
    const rows = Array.from(screen.getByTestId('research-status-scale').querySelectorAll('tbody tr'));
    expect(rows.map((r) => r.querySelector('.status-chip__word')?.textContent)).toEqual([...STATUSES]);
    for (const [i, s] of STATUSES.entries()) {
      expect(rows[i]?.querySelectorAll('td')[1]?.textContent).toBe(STATUS_INFO[s].definition);
      expect((document.body.textContent ?? '').split(STATUS_INFO[s].definition).length - 1, s).toBe(1);
    }
  });

  it('keeps the statements of what the catalogue does not do in Scientific Documentation, and off the pages', () => {
    const doc = render(<Documentation />);
    const text = doc.container.textContent ?? '';
    const statements = ['Nothing in the research catalogue is simulated.', 'a research concept that STRATO does not solve', 'No mechanism is switched on for a species automatically'];
    for (const s of statements) expect(text, s).toContain(s);
    doc.unmount();
    for (const page of ['', 'gallery', 'references', 'roadmap', 'species-ag', 'ag_trapping_detrapping']) {
      const view = renderPage(page);
      for (const s of statements) expect(view.container.textContent ?? '', `${page}: ${s}`).not.toContain(s);
      view.unmount();
    }
  });

  it('shows no file name or path on any research page, and lists the entry files with links in Scientific Documentation', () => {
    const FILE = /\b[\w-]+\.(?:rs|toml|md|csv|json|ya?ml)\b|\b(?:src|crates|tests|docs|data|inputs)\//;
    for (const page of ALL_PAGES) {
      const view = renderPage(page);
      expect(view.container.textContent ?? '', `research page "${page}"`).not.toMatch(FILE);
      view.unmount();
    }
    const doc = render(<Documentation />);
    const list = doc.getByTestId('research-entry-files');
    const hrefs = [...list.querySelectorAll('a')].map((a) => a.getAttribute('href') ?? '');
    for (const path of Object.keys(CATALOGUE.entries)) {
      expect(hrefs.some((h) => h.endsWith(`/${path}`)), path).toBe(true);
      expect(list.textContent ?? '', path).toContain(path);
    }
    expect(doc.getByTestId('doc-item-research-roadmap').textContent ?? '').toContain('The brick roadmap lists the nine bricks');
    doc.unmount();
  });

  it('names a folder by what it holds, so two folders with the same last part differ', () => {
    expect(pathLabel('tests/golden/btcs')).not.toBe(pathLabel('docs/mathematics/btcs'));
    expect(pathLabel('data/diffusivity/arrhenius.csv#A3-05')).toBe('diffusivity register A3-05');
    expect(pathLabel('some/new/file_name.json')).toBe('file name');
  });

  it('keeps the roadmap text in step with the catalogue rows of each brick', () => {
    const summary = roadmapSummary(CATALOGUE);
    expect(summary.size).toBe(9);
    for (const [brick, s] of summary) {
      const word = (/\b[A-Z]{7,}\b/.exec(s.statusText)?.[0] ?? '') as ResearchStatus;
      expect(STATUSES, `brick ${brick}: ${s.statusText}`).toContain(word);
      expect(rowsOfBrick(CATALOGUE, brick).map((r) => r.status), `brick ${brick}`).toContain(word);
    }
  });
});

describe('no pending mechanism has an enabled switch', () => {
  const press = async (el: HTMLElement) => {
    const user = userEvent.setup();
    await user.click(el);
    el.focus();
    await user.keyboard('{Enter}');
    await user.keyboard(' ');
  };

  it('gives every row of an entry a switch that is present, focusable, disabled and says what is missing', { timeout: 120_000 }, async () => {
    act(() => useStrato.getState().setCapabilities(['diffusion_btcs_layered']));
    const withEntry = CATALOGUE.mechanisms.filter((m) => m.entry);
    for (const path of [...new Set(withEntry.map((m) => m.entry))]) {
      const { unmount } = renderPage(entryPage(path));
      for (const row of withEntry.filter((m) => m.entry === path)) {
        const sw = screen.getByTestId(`row-switch-${rowKey(row)}`);
        expect(sw.getAttribute('role')).toBe('switch');
        expect(sw.getAttribute('aria-disabled')).toBe('true');
        expect(sw.getAttribute('aria-checked')).toBe('false');
        expect(sw.hasAttribute('disabled')).toBe(false);
        sw.focus();
        expect(document.activeElement).toBe(sw);
        expect(sw.textContent).toContain(`status ${row.status}`);
        const why = document.getElementById(sw.getAttribute('aria-describedby') ?? '');
        expect(why?.textContent, rowKey(row)).toContain(row.missing_work || 'No missing work');
        await press(sw);
        expect(sw.getAttribute('aria-checked')).toBe('false');
      }
      unmount();
    }
  });

  it('has no switch without aria-disabled anywhere in the Research section or block E, and sends no solver request', { timeout: 120_000 }, async () => {
    let asked = 0;
    act(() => {
      useStrato.getState().setCapabilities(['diffusion_btcs_layered']);
      useStrato.setState({ requestRun: () => (asked += 1) });
    });
    const config = JSON.stringify(useStrato.getState().config);
    const mechanisms = JSON.stringify(useStrato.getState().mechanisms);
    for (const page of ALL_PAGES) {
      const { unmount } = renderPage(page);
      const switches = Array.from(document.querySelectorAll<HTMLElement>('[role="switch"]'));
      for (const s of switches) expect(s.getAttribute('aria-disabled'), `${page}: ${s.dataset.testid ?? ''}`).toBe('true');
      for (const s of switches.slice(0, 2)) await press(s);
      unmount();
    }
    render(<FutureResearch />);
    for (const s of Array.from(document.querySelectorAll<HTMLElement>('[role="switch"]'))) expect(s.getAttribute('aria-disabled')).toBe('true');
    expect(asked).toBe(0);
    expect(JSON.stringify(useStrato.getState().config)).toBe(config);
    expect(JSON.stringify(useStrato.getState().mechanisms)).toBe(mechanisms);
  });

  it('stays disabled for a verified row without the capability, and for an implemented row with it', () => {
    const base = CATALOGUE.mechanisms.find((m) => m.mechanism_id === 'ag_trapping_detrapping')!;
    const verifiedNoCap: MechanismRow = { ...base, status: 'VERIFIED', solver_capability: 'ag_trapping_detrapping' };
    const implementedCap: MechanismRow = { ...base, status: 'IMPLEMENTED', solver_capability: 'ag_trapping_detrapping' };
    const cat = (row: MechanismRow): Catalogue => ({ ...CATALOGUE, mechanisms: [row, ...CATALOGUE.mechanisms.filter((m) => m.mechanism_id !== row.mechanism_id)] });
    expect(isRowAvailable(verifiedNoCap, ['diffusion_btcs_layered'], cat(verifiedNoCap))).toBe(false);
    expect(isRowAvailable(implementedCap, ['diffusion_btcs_layered', 'ag_trapping_detrapping'], cat(implementedCap))).toBe(false);
    // Only both together make a row live; a pending row never is.
    expect(isRowAvailable(verifiedNoCap, ['ag_trapping_detrapping'], cat(verifiedNoCap))).toBe(true);
    expect(isRowAvailable(base, ['ag_trapping_detrapping'], CATALOGUE)).toBe(false);
    for (const row of CATALOGUE.mechanisms.filter((m) => m.status !== 'VERIFIED')) {
      expect(isRowAvailable(row, [row.mechanism_id, row.solver_capability, 'diffusion_btcs_layered'].filter(Boolean)), rowKey(row)).toBe(false);
    }
  });
});

describe('Materials Studio gallery', () => {
  it('lists every placeholder compactly as PENDING with its title and species, and no image', () => {
    renderPage('gallery');
    const items = screen.getAllByTestId('gallery-item');
    expect(items).toHaveLength(CATALOGUE.gallery.length);
    for (const it of items) {
      const row = CATALOGUE.gallery.find((g) => g.item_id === it.dataset.item)!;
      expect(it.dataset.kind).toBe(row.kind);
      if (row.kind !== 'placeholder') continue;
      expect(it.querySelector('img, canvas, svg:not(.status-chip__shape), picture, video')).toBeNull();
      expect(it.querySelector('.status-chip__word')?.textContent).toBe('PENDING');
      expect(it.textContent).toContain(row.title);
    }
    // The sentence that all planned calculations share is not repeated on the page.
    expect(document.body.textContent ?? '').not.toContain('planned for a future phase of the project');
    expect(document.querySelector('[data-section="research"] img')).toBeNull();
  });

  it('holds the full text of every planned calculation once in Scientific Documentation, with the future-phase wording', () => {
    render(<Documentation />);
    const details = screen.getAllByTestId('gallery-detail');
    const planned = CATALOGUE.gallery.filter((g) => g.kind === 'placeholder');
    expect(details).toHaveLength(planned.length);
    for (const row of planned) {
      const it = details.find((d) => d.dataset.item === row.item_id)!;
      expect(it.querySelector('img, canvas, picture, video')).toBeNull();
      expect(it.querySelector('.status-chip__word')?.textContent).toBe('PENDING');
      for (const field of [row.method, row.conditions, row.material]) expect(it.textContent).toContain(field);
      expect(it.textContent).toContain(row.caption.replace(/\s*This calculation has not been performed\. It is planned for a future phase of the project\./, ''));
    }
    const doc = document.getElementById('doc-research-gallery-planned');
    expect((doc?.textContent ?? '').match(/planned for a future phase of the project/g)).toHaveLength(1);
  });

  it('shows a genuine screenshot only with its file and provenance, and labels reconstructions differently', () => {
    const base = CATALOGUE.gallery[0]!;
    const shot: GalleryRow = { ...base, item_id: 'fx-shot', kind: 'screenshot', file: 'docs/research/gallery/fx.png', provenance: 'fixture provenance', status: 'PENDING' };
    const noProv: GalleryRow = { ...shot, item_id: 'fx-noprov', provenance: '' };
    const recon: GalleryRow = { ...shot, item_id: 'fx-recon', kind: 'reconstruction', file: 'docs/research/gallery/fx.cif' };
    const { unmount } = render(<GalleryItem item={shot} catalogue={CATALOGUE} />);
    const img = document.querySelector('img');
    expect(img?.getAttribute('alt')).toBe(shot.alt_text);
    expect(img?.getAttribute('src')).toMatch(/research\/gallery\/fx\.png$/);
    const shotLabel = document.querySelector('.gallery-item__kind')?.textContent;
    expect(shotLabel).toMatch(/Materials Studio screenshot/);
    unmount();
    const r2 = render(<GalleryItem item={noProv} catalogue={CATALOGUE} />);
    expect(document.querySelector('img')).toBeNull();
    r2.unmount();
    render(<GalleryItem item={recon} catalogue={CATALOGUE} />);
    expect(document.querySelector('img')).toBeNull();
    const reconLabel = document.querySelector('.gallery-item__kind')?.textContent;
    expect(reconLabel).toMatch(/reconstruction/i);
    expect(reconLabel).not.toBe(shotLabel);
    expect(document.querySelector('.gallery-item')?.className).toContain('gallery-item--reconstruction');
  });
});

describe('roadmap, future phases and block E', () => {
  it('names both future items on the roadmap page and in block E, planned for a future phase', () => {
    const planned = sectionBlocks(CATALOGUE.entries[ROADMAP_ENTRY], 'two-items-among-the-future-phases');
    expect(planned.length).toBeGreaterThan(0);
    renderPage('roadmap');
    const roadmap = screen.getByTestId('roadmap-page');
    expect(roadmap.textContent).toMatch(/Materials Studio calculations/);
    expect(roadmap.textContent).toMatch(/FLiBe and cover-gas transport/);
    expect(roadmap.textContent).toMatch(/planned for a future phase/);
    expect(within(roadmap).getByTestId('roadmap-bricks').querySelectorAll('[data-brick]')).toHaveLength(9);
  });

  it('block E separates what the solver implements from the research, and links the Research section', () => {
    render(<App />);
    const e = screen.getByTestId('block-e');
    const impl = within(e).getByTestId('future-implemented');
    const pending = within(e).getByTestId('future-pending');
    for (const c of Array.from(impl.querySelectorAll<HTMLElement>('[data-status-row]'))) expect(['IMPLEMENTED', 'VERIFIED', 'VALIDATED']).toContain(c.dataset.status);
    expect(pending.querySelectorAll('[data-status-row]').length).toBe(0);
    expect(within(e).getByTestId('future-species').querySelectorAll('[data-species]')).toHaveLength(9);
    expect(within(e).getByTestId('future-regimes').children).toHaveLength(3);
    expect(within(e).getByTestId('future-planned').textContent).toMatch(/planned for a future phase/);
    expect(within(e).getByTestId('status-legend')).toBeTruthy();
    expect(e.querySelector('a[href="#/research"]')).not.toBeNull();
  });

  it('says on the downstream entry that FLiBe and cover-gas modelling does not exist and is planned, apart from the gas-cooled AGR data', () => {
    renderPage('additional_species_downstream_transport');
    const text = screen.getByTestId('entry-page').textContent ?? '';
    expect(text).toMatch(/planned for a future phase/);
    expect(text).toMatch(/gas-cooled/);
  });
});
