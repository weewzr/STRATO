// @vitest-environment node
// The mechanical guards of the minimal-text rules (scripts/minimal_text_rules.mjs, run by
// scripts/check_content.mjs): each rule must fail on a page that breaks it and pass on one that does not.
import { spawnSync } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import {
  CAPTION_MAX_CHARS,
  DOCUMENTATION_PAGE,
  evaluateSnapshots,
  HASH_PATTERN,
  PATH_PATTERN,
  PROSE_MAX_WORDS,
  TEMPORARY_EXCEPTIONS,
  type PageSnapshot,
} from '../scripts/minimal_text_rules.mjs';

const none: Record<string, string[]> = {};
const page = (over: Partial<PageSnapshot> = {}): PageSnapshot => ({
  id: 'compare',
  page: 'compare',
  text: 'Release fraction of the baseline and the current inputs.',
  shellText: 'STRATO Explore Compare Result COMPUTED v2.0',
  captions: ['Figure 1. Release fraction of the baseline and of the current inputs.'],
  proseWords: 12,
  ...over,
});
const failuresOf = (snaps: PageSnapshot[], rule: string, exceptions = none) => evaluateSnapshots(snaps, exceptions).results.find((r) => r.rule === rule)?.failures ?? [];
const words = (n: number) => Array.from({ length: n }, () => 'word').join(' ');

describe('build hash and path guards', () => {
  it('passes a clean page', () => {
    for (const rule of ['build-hash', 'path', 'caption', 'prose', 'disclaimer']) expect(failuresOf([page()], rule), rule).toEqual([]);
  });

  it('finds a build hash or digest, and not a number or a word', () => {
    expect(failuresOf([page({ text: 'build 08ff6bb of the solver' })], 'build-hash')).toHaveLength(1);
    expect(failuresOf([page({ text: 'SHA-256 257fe9933a4c' })], 'build-hash')).toHaveLength(1);
    expect(failuresOf([page({ text: 'difference 3.997129786790144e-12 at 0.0002795123671863 s, defaced and 1000000' })], 'build-hash')).toEqual([]);
    expect('08ff6bb'.match(HASH_PATTERN)).not.toBeNull();
  });

  it('finds a repository path, a source file name and a build command in visible text', () => {
    for (const t of ['inputs/synthetic/synthetic_five_layer.toml', 'see tests/btcs_verification.rs', 'btcs_run.json', 'run cargo test now', 'crates/strato-wasm', 'strato-webtools']) {
      expect(failuresOf([page({ text: t })], 'path'), t).toHaveLength(1);
    }
    expect(failuresOf([page({ text: 'input/output and metadata of the layers, doi:10.1016/j.jnucmat.2021.152840' })], 'path')).toEqual([]);
    expect(PATH_PATTERN.source.length).toBeGreaterThan(0);
  });

  it('does not apply to the documentation page, which holds this text', () => {
    const doc = page({ id: 'documentation', page: DOCUMENTATION_PAGE, text: 'build 08ff6bb, tests/btcs_verification.rs, cargo test', proseWords: 400 });
    for (const rule of ['build-hash', 'path', 'prose']) expect(failuresOf([doc], rule), rule).toEqual([]);
  });

  it('checks the header and footer once, as the shell', () => {
    const shell = page({ shellText: 'STRATO build 08ff6bb' });
    expect(failuresOf([shell], 'build-hash')).toHaveLength(1);
    expect(failuresOf([shell], 'build-hash', { shell: ['build-hash'] })).toEqual([]);
  });
});

describe('caption and prose guards', () => {
  it('limits a caption to the stated number of characters', () => {
    const ok = 'x'.repeat(CAPTION_MAX_CHARS);
    const long = 'x'.repeat(CAPTION_MAX_CHARS + 1);
    expect(failuresOf([page({ captions: [ok] })], 'caption')).toEqual([]);
    expect(failuresOf([page({ captions: [long] })], 'caption')).toHaveLength(1);
  });

  it('limits the prose before the first control, equation, graph or table', () => {
    expect(failuresOf([page({ proseWords: PROSE_MAX_WORDS })], 'prose')).toEqual([]);
    expect(failuresOf([page({ proseWords: PROSE_MAX_WORDS + 1 })], 'prose')).toHaveLength(1);
    // The About page may open with a short introduction of its own.
    expect(failuresOf([page({ id: 'about', page: 'about', proseWords: 100 })], 'prose')).toEqual([]);
    expect(failuresOf([page({ id: 'about', page: 'about', proseWords: 121 })], 'prose')).toHaveLength(1);
  });
});

describe('registered disclaimers', () => {
  const sentence = 'The case is a synthetic verification case, not physical data.';
  it('passes a sentence that appears once and fails one that appears twice', () => {
    expect(failuresOf([page({ text: sentence })], 'disclaimer')).toEqual([]);
    const two = [page({ id: 'compare', page: 'compare', text: sentence }), page({ id: 'data', page: 'data', text: `${sentence} Again.` })];
    expect(failuresOf(two, 'disclaimer')).toHaveLength(1);
    expect(failuresOf([page({ text: `${sentence} ${sentence}` })], 'disclaimer')).toHaveLength(1);
  });

  it('counts the documentation page with the others', () => {
    const snaps = [page({ id: 'documentation', page: DOCUMENTATION_PAGE, text: sentence }), page({ text: sentence })];
    expect(failuresOf(snaps, 'disclaimer')).toHaveLength(1);
  });

  it('does not count the pages that carry a temporary exception', () => {
    const snaps = [page({ text: sentence }), page({ id: 'data', page: 'data', text: sentence })];
    expect(failuresOf(snaps, 'disclaimer', { data: ['disclaimer'] })).toEqual([]);
  });
});

describe('temporary exceptions', () => {
  it('hide the failures of the named page and rule only, and a stale one is reported', () => {
    const bad = page({ text: 'inputs/synthetic/x.toml', captions: ['x'.repeat(300)] });
    const { results, notes } = evaluateSnapshots([bad], { compare: ['path', 'prose'] });
    expect(results.find((r) => r.rule === 'path')?.failures).toEqual([]);
    expect(results.find((r) => r.rule === 'caption')?.failures).toHaveLength(1);
    expect(notes.join(' ')).toContain('compare / prose');
  });

  it('names only the Explore page (removed by the integration task), and the documentation page is never among them', () => {
    expect(Object.keys(TEMPORARY_EXCEPTIONS).sort()).toEqual(['explore']);
    expect(TEMPORARY_EXCEPTIONS[DOCUMENTATION_PAGE]).toBeUndefined();
  });
});

describe('check_content.mjs with page snapshots', () => {
  const run = (snaps: PageSnapshot[]) => {
    const dir = mkdtempSync(join(tmpdir(), 'strato-content-'));
    writeFileSync(join(dir, 'index.html'), '<!doctype html><title>x</title>');
    writeFileSync(join(dir, 'build.json'), '{}');
    const file = join(dir, '..', `${dir.split(/[\\/]/).pop()}-snapshots.json`);
    writeFileSync(file, JSON.stringify(snaps));
    const script = fileURLToPath(new URL('../scripts/check_content.mjs', import.meta.url));
    return spawnSync(process.execPath, [script, dir, '--snapshots', file], { encoding: 'utf8' });
  };

  it('exits 0 for clean pages and 1 with the failing rule and page for a page that breaks a rule', () => {
    const clean = run([page({ id: 'documentation', page: DOCUMENTATION_PAGE })]);
    expect(clean.stdout).toContain('PASS');
    expect(clean.status).toBe(0);
    const bad = run([page({ id: 'documentation', page: DOCUMENTATION_PAGE, captions: [words(80)] })]);
    expect(bad.status).toBe(1);
    expect(bad.stdout).toContain('FAIL a figure or table caption longer than');
    expect(bad.stdout).toContain('[documentation]');
  });
});
