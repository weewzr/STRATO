// The leaf-by-leaf comparison of two replies, the browser counterpart of `btcs_parity compare`
// (crates/strato-wasm/examples/btcs_parity.rs): the same rule, with the series' accuracy floor as an
// allowance for the series-derived leaves only.
import { describe, expect, it } from 'vitest';
import { compareReplies } from '../src/solver/compare';
import { PARITY_RTOL } from '../src/solver/tolerances';

const base = () => ({
  status: 'ok',
  engine: { commit: 'aaaaaaa' },
  release_fraction: [0, 0.25, 0.5],
  analytical: { accuracy_floor: 1e-12, release_fraction: [0, 0.2500000000005, 0.5] },
  diagnostics: { analytical_comparison: { max_abs_difference: 1e-4, min_flux_ratio: 1e-5 } },
});

describe('compareReplies', () => {
  it('accepts equal replies and ignores the commit that each build stamps', () => {
    const a = base();
    const b = base();
    b.engine.commit = 'bbbbbbb';
    const c = compareReplies(a, b, PARITY_RTOL);
    expect(c.ok).toBe(true);
    expect(c.identical).toBe(c.compared);
  });

  it('fails a BTCS value that differs by more than the relative rule, with its path', () => {
    const b = base();
    b.release_fraction[1] = 0.25 * (1 + 1e-9);
    const c = compareReplies(base(), b, PARITY_RTOL);
    expect(c.ok).toBe(false);
    expect(c.failures.map((f) => f.pattern)).toEqual(['.release_fraction[]']);
    expect(c.failures[0]?.count).toBe(1);
  });

  it('allows the series floor for series values only, and not for the numbers that define the floor', () => {
    const b = base();
    b.analytical.release_fraction[1] = 0.2500000000005 + 5e-13;
    expect(compareReplies(base(), b, PARITY_RTOL).ok).toBe(true);
    b.analytical.release_fraction[1] = 0.2500000000005 + 5e-11;
    expect(compareReplies(base(), b, PARITY_RTOL).ok).toBe(false);
    const f = base();
    f.analytical.accuracy_floor = 1.001e-12;
    expect(compareReplies(base(), f, PARITY_RTOL).ok).toBe(false);
    const m = base();
    m.diagnostics.analytical_comparison.min_flux_ratio = 1.1e-5;
    expect(compareReplies(base(), m, PARITY_RTOL).ok).toBe(false);
  });

  it('fails a different array length, a missing key and an unequal string', () => {
    const b = base() as Record<string, unknown>;
    b.release_fraction = [0, 0.25];
    const c = compareReplies(base(), b, PARITY_RTOL);
    expect(c.failures.map((f) => f.pattern)).toContain('.release_fraction.length');
    const d = base() as Record<string, unknown>;
    delete d.status;
    expect(compareReplies(base(), d, PARITY_RTOL).failures.map((f) => f.pattern)).toContain('.status (key)');
    const e = base();
    e.status = 'rejected';
    expect(compareReplies(base(), e, PARITY_RTOL).ok).toBe(false);
  });

  it('compares an exported file without its commit line', () => {
    const a = { text: '# commit: abc1234\n# label: x\nTime (s)\n0e0\n' };
    const b = { text: '# commit: def5678\n# label: x\nTime (s)\n0e0\n' };
    expect(compareReplies(a, b, PARITY_RTOL).ok).toBe(true);
    expect(compareReplies(a, { text: '# commit: def5678\n# label: y\nTime (s)\n0e0\n' }, PARITY_RTOL).ok).toBe(false);
  });
});
