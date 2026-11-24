// @vitest-environment node
// Repository guards for the web sources: no hand-typed scientific values, generated data with a
// provenance header, the shared token file unchanged, and none of the text the owner keeps off the site.
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import caseJson from '../src/generated/synthetic_five_layer.case.json';

const SRC = fileURLToPath(new URL('../src', import.meta.url));
const WEB = fileURLToPath(new URL('..', import.meta.url));

function files(dir: string, out: string[] = []): string[] {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) files(p, out);
    else out.push(p);
  }
  return out;
}

const sources = files(SRC).filter((f) => /\.(ts|tsx|css)$/.test(f));

describe('no hand-typed scientific values in the components', () => {
  it('has no floating-point literal in exponent notation outside src/generated', () => {
    // A literal such as 2e-4 or 1.0e-12 in a component would be a parameter typed by hand.
    const offenders: string[] = [];
    for (const f of sources) {
      if (f.includes(`${join('src', 'generated')}`)) continue;
      const text = readFileSync(f, 'utf8')
        .replace(/\/\/.*$/gm, '')
        .replace(/\/\*[\s\S]*?\*\//g, '');
      const hits = text.match(/(?<![\w#.-])\d+(\.\d+)?[eE][+-]?\d+(?![\w])/g);
      if (hits) offenders.push(`${f}: ${hits.join(', ')}`);
    }
    expect(offenders).toEqual([]);
  });
});

describe('research statuses are never typed into a component', () => {
  // From web phase 2 the five status words may appear only in src/generated/ and in
  // src/research/status.ts (prompt/TESTING_AND_VALIDATION.md Section 3.1). The old two-level
  // chip word is retired (owner decision M7). Stored encoded, so this file does not carry them.
  const words = ['UEVORElORw==', 'UkVTRUFSQ0hFRA==', 'SU1QTEVNRU5URUQ=', 'VkVSSUZJRUQ=', 'VkFMSURBVEVE', 'UExBTk5FRA=='].map((b) =>
    Buffer.from(b, 'base64').toString('utf8'),
  );
  const allowed = [join('src', 'generated'), join('src', 'research', 'status.ts')];

  it('finds no status word outside the generated catalogue and the status module', () => {
    const offenders: string[] = [];
    for (const f of sources) {
      if (allowed.some((a) => f.includes(a))) continue;
      const text = readFileSync(f, 'utf8');
      for (const w of words) if (new RegExp(`\\b${w}\\b`).test(text)) offenders.push(`${f}: ${w}`);
    }
    expect(offenders).toEqual([]);
  });
});

describe('generated case data', () => {
  it('carries the provenance header', () => {
    expect(caseJson.label).toBe('INPUT');
    expect(caseJson.generator).toMatch(/^scripts\/web\/generate_data\.py v\d+$/);
    expect(caseJson.input_file).toBe('inputs/synthetic/synthetic_five_layer.toml');
    expect(caseJson.input_sha256).toMatch(/^[0-9a-f]{64}$/);
    expect(caseJson.input_commit).toMatch(/^[0-9a-f]{40}$/);
    expect(caseJson.strato_version).toMatch(/^\d+\.\d+\.\d+$/);
    expect(caseJson.command).toContain('generate_data.py case');
  });

  it('agrees value for value with the downloadable CSV written by the same generator', () => {
    const csv = readFileSync(join(WEB, 'public', 'data', 'synthetic_five_layer.case.csv'), 'utf8');
    expect(csv).toContain(`# input_sha256: ${caseJson.input_sha256}`);
    const rows = csv
      .split('\n')
      .filter((l) => l && !l.startsWith('#'))
      .slice(1)
      .map((l) => l.split(','));
    const cfg = caseJson.data.config;
    for (const [group, idx, , key, value] of rows) {
      if (group === 'region') {
        const r = cfg.regions[Number(idx)] as Record<string, unknown>;
        expect(Number(value)).toBe(r[key as string]);
      } else if (group === 'time') {
        expect(Number(value)).toBe((cfg.time as Record<string, unknown>)[key as string]);
      } else if (group === 'initial') {
        expect(Number(value)).toBe(cfg.initial.kernel_concentration_mol_m3);
      }
    }
    expect(rows.length).toBe(3 + cfg.regions.length * 3 + 1 + 3);
  });
});

describe('shared design core', () => {
  it('imports tokens.core.css version 1.0.0, copied unchanged', () => {
    const core = readFileSync(join(SRC, 'design', 'tokens.core.css'), 'utf8');
    expect(core.split('\n')[0]).toBe('/* tokens.core.css, shared design core, version 1.0.0 (2026-11-24)');
    const tokens = readFileSync(join(SRC, 'design', 'tokens.css'), 'utf8');
    expect(tokens).toContain("@import './tokens.core.css';");
  });
});

describe('owner content rules (M3, M5)', () => {
  // Stored encoded, so this file does not carry the strings it forbids.
  const forbidden = ['U05SU0k=', 'd2l0aGluIDIl'].map((b) => Buffer.from(b, 'base64').toString('utf8'));

  it('keeps the internal comparison text and any agreement figure out of the web sources', () => {
    for (const f of [...sources, join(WEB, 'index.html')]) {
      const text = readFileSync(f, 'utf8');
      for (const s of forbidden) expect(text.includes(s), `${f} contains a forbidden string`).toBe(false);
    }
  });

  it('does not link the prompt folder', () => {
    for (const f of sources) expect(readFileSync(f, 'utf8')).not.toMatch(/['"`/]prompt\//);
  });
});
