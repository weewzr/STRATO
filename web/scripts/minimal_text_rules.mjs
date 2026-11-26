// Mechanical guards of the minimal-text rules (prompt/DESIGN_SPECIFICATION.md, section "Minimal-text rules").
// Pure functions over page snapshots, so each rule can be tested with a fixture. The snapshots come
// from scripts/render_pages.mjs (the built site rendered in a browser) or from a file given to
// check_content.mjs with --snapshots.
//
// A snapshot is { id, page, text, shellText, captions, proseWords }:
//   id         a unique id of the rendered view ("compare", "research-roadmap", ...)
//   page       the page it belongs to: explore, compare, validate, verify, research, data, about,
//              documentation. The temporary exceptions below are named by page.
//   text       the visible text of the main interface of that view
//   shellText  the visible text of the header and footer (the same on every page, checked once as "shell")
//   captions   the visible text of every figure caption and table caption of the view
//   proseWords the number of words of explanatory prose before the first control, equation, graph or table
//   longestBlockWords, longestBlockText (optional) the longest paragraph or list item outside tables, and its first words

/** A figure or table caption is about one sentence: at most this many characters (with its "Figure n." prefix). */
export const CAPTION_MAX_CHARS = 220;

/** No more than this many words of prose before the first control, equation, graph or table of a page. */
export const PROSE_MAX_WORDS = 60;

/**
 * Pages whose first content is a short introduction and has no control, equation, graph or table of its
 * own may carry a longer opening; the allowance is part of the rule, not an exception.
 */
export const PROSE_MAX_WORDS_BY_PAGE = { about: 120 };

/** The one page that holds detailed, technical and implementation text; the hash and path rules do not apply to it. */
export const DOCUMENTATION_PAGE = 'documentation';

/** No single paragraph or list item of a page (outside Scientific Documentation) is longer than this many words. */
export const BLOCK_MAX_WORDS = 100;

export const RULES = ['build-hash', 'path', 'caption', 'prose', 'block', 'disclaimer'];

export const RULE_MESSAGES = {
  'build-hash': 'a build hash or digest in the main interface (it belongs in Scientific Documentation)',
  path: 'a repository file path, source file name or build command in visible text (it belongs in Scientific Documentation; a link address may hold a path, its text may not)',
  caption: `a figure or table caption longer than ${CAPTION_MAX_CHARS} characters (a caption is about one sentence)`,
  prose: `more than ${PROSE_MAX_WORDS} words of prose before the first control, equation, graph or table`,
  block: `a paragraph or list item longer than ${BLOCK_MAX_WORDS} words anywhere on a page (long explanations belong in Scientific Documentation)`,
  disclaimer: 'a registered disclaimer sentence that appears more than once in the site (it belongs once in Scientific Documentation)',
};

// Hexadecimal word of 7 to 64 characters with at least one digit and one letter a-f: a commit hash or digest.
// A number such as 3.99e-12 is not one (the run starts after a dot or inside a longer word).
export const HASH_PATTERN = /(?<![\w.-])(?=[0-9a-f]*\d)(?=[0-9a-f]*[a-f])[0-9a-f]{7,64}(?![\w-]|\.\w)/g;

// A path into the repository, a source or data file name, or a build command.
export const PATH_PATTERN = new RegExp(
  [
    String.raw`\b(?:src|crates|tests|docs|data|inputs|web|prompt|examples|scripts)\/[\w.\/-]*\w`,
    String.raw`\b\.github\/[\w.\/-]*\w`,
    String.raw`\b[\w-]+\.(?:rs|tsx?|mjs|toml|md|cff|lock|csv|json|ya?ml)\b`,
    String.raw`\bcargo (?:test|run|build)\b`,
    String.raw`\bstrato-(?:webtools|docsite|wasm|data)\b`,
    String.raw`\bwasm-bindgen\b`,
    String.raw`\bnpm run\b`,
  ].join('|'),
  'g',
);

/**
 * Registered disclaimer sentences: a qualification that appears once in the whole site, in Scientific
 * Documentation. Add a sentence here when it is found repeated. (The label "Research concept, not solved by
 * STRATO" of each research equation is a label of the equation, not a disclaimer, and is not registered.)
 */
export const DISCLAIMERS = [
  { id: 'synthetic-case-not-physical', pattern: /synthetic (?:five-layer )?verification case,? not physical data/gi },
  { id: 'not-measured-triso', pattern: /not measured TRISO properties/gi },
  { id: 'not-experimental-validation', pattern: /not experimental validation|verification is not validation/gi },
  { id: 'labelled-previews', pattern: /labelled previews?/gi },
  { id: 'nothing-simulated', pattern: /nothing in (?:this section|the research catalogue) is simulated/gi },
];

/**
 * No page is excepted from any rule. The parameter exceptions of the evaluation functions remains for the
 * tests of the guards themselves (a fixture can name an exception to prove that it is reported).
 */
export const TEMPORARY_EXCEPTIONS = {};

const norm = (s) => s.replace(/[‘’]/g, "'").replace(/\s+/g, ' ').trim();

const excepted = (exceptions, page, rule) => (exceptions[page] ?? []).includes(rule);

function matches(text, pattern) {
  const out = [];
  for (const m of norm(text).matchAll(new RegExp(pattern.source, pattern.flags.includes('g') ? pattern.flags : `${pattern.flags}g`))) out.push(m[0]);
  return out;
}

const short = (s) => (s.length > 60 ? `${s.slice(0, 57)}...` : s);

/**
 * Applies every rule. Returns { results, notes }: results is [{ rule, name, failures, excepted }] with the
 * failures of excepted pages left out (and counted in `excepted`), notes lists exceptions that are stale.
 */
export function evaluateSnapshots(snapshots, exceptions = TEMPORARY_EXCEPTIONS) {
  const found = new Map(); // `${page}|${rule}` -> failures (before exceptions)
  const add = (page, rule, message) => {
    const key = `${page}|${rule}`;
    if (!found.has(key)) found.set(key, []);
    found.get(key).push(message);
  };

  // The shell (header and footer) is the same on every view; check the first snapshot's copy.
  const shell = snapshots.find((s) => typeof s.shellText === 'string');
  if (shell) {
    for (const m of matches(shell.shellText, HASH_PATTERN)) add('shell', 'build-hash', `header or footer: "${m}"`);
    for (const m of matches(shell.shellText, PATH_PATTERN)) add('shell', 'path', `header or footer: "${m}"`);
  }

  for (const s of snapshots) {
    const technical = s.page === DOCUMENTATION_PAGE;
    if (!technical) {
      for (const m of matches(s.text, HASH_PATTERN)) add(s.page, 'build-hash', `${s.id}: "${m}"`);
      for (const m of matches(s.text, PATH_PATTERN)) add(s.page, 'path', `${s.id}: "${m}"`);
      const limit = PROSE_MAX_WORDS_BY_PAGE[s.page] ?? PROSE_MAX_WORDS;
      if ((s.longestBlockWords ?? 0) > BLOCK_MAX_WORDS) add(s.page, 'block', `${s.id}: a block of ${s.longestBlockWords} words ("${short(s.longestBlockText ?? '')}"; limit ${BLOCK_MAX_WORDS})`);
      if (s.proseWords > limit) add(s.page, 'prose', `${s.id}: ${s.proseWords} words of prose before the first control, equation, graph or table (limit ${limit})`);
    }
    for (const c of s.captions) {
      const n = norm(c).length;
      if (n > CAPTION_MAX_CHARS) add(s.page, 'caption', `${s.id}: caption of ${n} characters ("${short(norm(c))}")`);
    }
  }

  // Disclaimers: the occurrences of the whole site; occurrences on a page with a disclaimer exception are not counted.
  const totals = new Map();
  const where = new Map();
  for (const d of DISCLAIMERS) {
    for (const s of snapshots) {
      if (excepted(exceptions, s.page, 'disclaimer')) continue;
      const n = matches(s.text, d.pattern).length;
      if (n === 0) continue;
      totals.set(d.id, (totals.get(d.id) ?? 0) + n);
      where.set(d.id, [...(where.get(d.id) ?? []), `${s.id} (${n})`]);
    }
  }
  const disclaimerFailures = [];
  for (const [id, n] of totals) if (n > 1) disclaimerFailures.push(`disclaimer "${id}" appears ${n} times: ${where.get(id).join(', ')}`);
  // Also report the occurrences hidden by exceptions, so that a stale exception is visible.
  const hiddenDisclaimerPages = new Set();
  for (const d of DISCLAIMERS) {
    const perPage = new Map();
    for (const s of snapshots) {
      const n = matches(s.text, d.pattern).length;
      if (n > 0) perPage.set(s.page, (perPage.get(s.page) ?? 0) + n);
    }
    const all = [...perPage.values()].reduce((a, b) => a + b, 0);
    if (all > 1) for (const p of perPage.keys()) hiddenDisclaimerPages.add(p);
  }

  const results = [];
  const notes = [];
  const pages = [...new Set([...Object.keys(exceptions), ...snapshots.map((s) => s.page), 'shell'])];
  for (const rule of RULES) {
    if (rule === 'disclaimer') {
      results.push({ rule, name: RULE_MESSAGES.disclaimer, failures: disclaimerFailures, excepted: pages.filter((p) => excepted(exceptions, p, rule)) });
      continue;
    }
    const failures = [];
    const hidden = [];
    for (const page of pages) {
      const list = found.get(`${page}|${rule}`) ?? [];
      if (excepted(exceptions, page, rule)) {
        if (list.length > 0) hidden.push(page);
      } else {
        failures.push(...list.map((m) => `[${page}] ${m}`));
      }
    }
    results.push({ rule, name: RULE_MESSAGES[rule], failures, excepted: hidden });
  }
  for (const page of Object.keys(exceptions)) {
    if (!snapshots.some((s) => s.page === page) && page !== 'shell') continue;
    for (const rule of exceptions[page]) {
      const still = rule === 'disclaimer' ? hiddenDisclaimerPages.has(page) : (found.get(`${page}|${rule}`) ?? []).length > 0;
      if (!still) notes.push(`temporary exception ${page} / ${rule} is no longer needed: remove it from TEMPORARY_EXCEPTIONS`);
    }
  }
  return { results, notes };
}
