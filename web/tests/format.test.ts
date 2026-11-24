import { describe, expect, it } from 'vitest';
import { plain, sciParts, sciText } from '../src/format';
import { hrefFor, sectionFromHash } from '../src/route';

describe('format (display only)', () => {
  it.each([2e-4, 1e-12, 4e-10, 3.5e-4, 200, 200000, 1, 0.5, 1.23456789e-15])('plain(%s) reads back to the same double', (v) => {
    expect(Number(plain(v))).toBe(v);
  });

  it('writes scientific notation for small values', () => {
    expect(plain(2e-4)).toBe('2e-4');
    expect(plain(200000)).toBe('200000');
    expect(sciParts(7e-13)).toEqual({ mantissa: '7', exponent: '−13' });
    expect(sciParts(200)).toEqual({ mantissa: '200', exponent: null });
    expect(sciText(1e-12)).toBe('1 × 10^-12');
  });
});

describe('route', () => {
  it('maps hashes to sections, defaulting to Explore', () => {
    expect(sectionFromHash('')).toBe('explore');
    expect(sectionFromHash('#/verify')).toBe('verify');
    expect(sectionFromHash('#data')).toBe('data');
    expect(sectionFromHash('#/nowhere')).toBe('explore');
    expect(hrefFor('about')).toBe('#/about');
  });
});
