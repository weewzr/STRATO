// Renders the built site in a browser and returns one snapshot per view for the minimal-text guards
// (scripts/minimal_text_rules.mjs). Serves the build directory on 127.0.0.1 under the Pages base path;
// with STRATO_CHECK_URL set (for example the published site) it renders that address instead.
// Set PW_CHANNEL=msedge to use the installed Microsoft Edge.
/* global document */
import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { extname, join, normalize } from 'node:path';

const BASE = '/STRATO/';
const PORT = 4441;

/** Views to render: the id, the page it belongs to, the hash and what to wait for. */
export const VIEWS = [
  { id: 'explore', page: 'explore', hash: '#/explore', settle: '[data-block]' },
  { id: 'compare', page: 'compare', hash: '#/compare', settle: '[data-testid="compare-figure"][data-state="ok"]' },
  { id: 'validate', page: 'validate', hash: '#/validate' },
  { id: 'verify', page: 'verify', hash: '#/verify', settle: '[data-testid="verify-balance-plot"]' },
  { id: 'research', page: 'research', hash: '#/research' },
  { id: 'research-roadmap', page: 'research', hash: '#/research/roadmap' },
  { id: 'research-gallery', page: 'research', hash: '#/research/gallery' },
  { id: 'research-references', page: 'research', hash: '#/research/references' },
  { id: 'data', page: 'data', hash: '#/data' },
  { id: 'about', page: 'about', hash: '#/about' },
  { id: 'documentation', page: 'documentation', hash: '#/documentation' },
];

const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.csv': 'text/csv; charset=utf-8',
  '.wasm': 'application/wasm',
  '.svg': 'image/svg+xml',
  '.woff2': 'font/woff2',
  '.woff': 'font/woff',
  '.png': 'image/png',
  '.txt': 'text/plain; charset=utf-8',
};

function serve(root) {
  const server = createServer(async (req, res) => {
    try {
      let path = decodeURIComponent((req.url ?? '/').split('?')[0]);
      if (!path.startsWith(BASE)) path = BASE;
      let file = normalize(join(root, path.slice(BASE.length)));
      if (!file.startsWith(normalize(root))) throw new Error('outside');
      if ((await stat(file).catch(() => null))?.isDirectory()) file = join(file, 'index.html');
      const body = await readFile(file);
      res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' });
      res.end(body);
    } catch {
      res.writeHead(404);
      res.end('not found');
    }
  });
  return new Promise((resolve) => server.listen(PORT, '127.0.0.1', () => resolve(server)));
}

/** Runs in the page: the text, captions and the prose that comes before the first control, equation, graph or table. */
function snapshotInPage() {
  document.querySelectorAll('details').forEach((d) => {
    d.open = true;
  });
  const main = document.querySelector('main');
  const text = main ? main.innerText : '';
  const shellText = [document.querySelector('header'), document.querySelector('footer')].map((e) => (e ? e.innerText : '')).join('\n');
  const captions = Array.from(document.querySelectorAll('main figcaption, main caption.table__caption')).map((c) => c.innerText);

  const PRIMARY = 'input, select, textarea, button, [role="switch"], table, svg, canvas, .katex, [role="math"], .eq-plain, .js-plotly-plot, figure, .ref-list, .refs';
  const PROSE = 'p, li, dd, blockquote';
  let words = 0;
  if (main) {
    const seen = [];
    for (const el of main.querySelectorAll(`${PROSE}, ${PRIMARY}`)) {
      if (el.matches(PRIMARY) || el.querySelector(PRIMARY)) break;
      if (el.closest('nav, .crumbs, [hidden]')) continue;
      if (seen.some((p) => p.contains(el))) continue;
      const t = el.innerText.trim();
      if (t === '') continue;
      seen.push(el);
      words += t.split(/\s+/).length;
    }
  }
  // The longest single paragraph or list item outside tables (an item that holds other items is not counted).
  let longestBlockWords = 0;
  let longestBlockText = '';
  if (main) {
    for (const el of main.querySelectorAll('p, li, dd, blockquote')) {
      // A bibliographic reference is a citation, not prose; it is counted by no rule.
      if (el.closest('table, nav, [hidden], .ref-list, .refs') || el.querySelector('p, li, dd')) continue;
      // A typeset formula is not prose: its text (the KaTeX markup and the MathML copy, one symbol per line) is left out of the count.
      const copy = el.cloneNode(true);
      copy.querySelectorAll('.katex').forEach((k) => k.replaceWith(' '));
      const t = (copy.textContent ?? '').replace(/\s+/g, ' ').trim();
      const n = t === '' ? 0 : t.split(/\s+/).length;
      if (n > longestBlockWords) {
        longestBlockWords = n;
        longestBlockText = t;
      }
    }
  }
  return { text, shellText, captions, proseWords: words, longestBlockWords, longestBlockText };
}

export async function collectSnapshots(dist) {
  const { chromium } = await import('@playwright/test');
  const external = process.env.STRATO_CHECK_URL;
  const server = external ? null : await serve(dist);
  const base = external ?? `http://127.0.0.1:${PORT}${BASE}`;
  const channel = process.env.PW_CHANNEL;
  const browser = await chromium.launch(channel ? { channel } : {});
  const snapshots = [];
  try {
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
    const page = await context.newPage();
    for (const v of VIEWS) {
      await page.goto(`${base}${v.hash}`);
      await page.waitForSelector(`main h1`, { timeout: 30_000 });
      // The solver status in the footer says "loading" until the WebAssembly module has answered or failed.
      await page.waitForFunction(() => !/Solver: loading/.test(document.body.innerText), undefined, { timeout: 30_000 }).catch(() => undefined);
      if (v.settle) await page.waitForSelector(v.settle, { timeout: 30_000 }).catch(() => undefined);
      await page.waitForTimeout(1_500);
      snapshots.push({ id: v.id, page: v.page, ...(await page.evaluate(snapshotInPage)) });
    }
  } finally {
    await browser.close();
    if (server) await new Promise((r) => server.close(r));
  }
  return snapshots;
}
