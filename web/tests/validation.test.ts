import { describe, expect, it } from 'vitest';
import { parseNumber } from '../src/state/validation';

describe('parseNumber', () => {
  it.each([
    ['2e-4', 2e-4],
    ['2.0e-4', 2e-4],
    ['0.0002', 2e-4],
    [' 1E-12 ', 1e-12],
    ['200', 200],
    ['-3.5', -3.5],
    ['.5', 0.5],
    ['2 × 10^-4', 2e-4],
  ])('reads %s', (text, v) => {
    expect(parseNumber(text)).toBe(v);
  });

  it.each(['', 'abc', '1e', '1..2', '0x10', 'Infinity', '1e999'])('refuses %s', (text) => {
    expect(parseNumber(text)).toBeNull();
  });
});
