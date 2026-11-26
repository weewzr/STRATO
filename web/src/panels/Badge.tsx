import type { ReactNode } from 'react';
import { sciParts } from '../format';
import type { ResultLabel } from '../solver/types';

const CLASS: Record<ResultLabel, string> = {
  'COMPUTED v2.0': 'badge badge--computed',
  PRECOMPUTED: 'badge badge--precomputed',
  MEASURED: 'badge badge--measured',
  ILLUSTRATIVE: 'badge badge--illustrative',
};

/**
 * Result label of a number, plot or particle colouring: the word plus a shape (solid, dashed,
 * hatched, dot), never colour alone. Research statuses of mechanisms are a separate system with
 * their own shapes (web phase 2).
 */
export function Badge({ label }: { label: ResultLabel }) {
  return (
    <span className={CLASS[label]} data-label={label}>
      {label}
    </span>
  );
}

/** A number in scientific notation with an optional unit, typeset with a superscript exponent. */
export function Sci({ value, unit, digits }: { value: number; unit?: ReactNode; digits?: number }) {
  const { mantissa, exponent } = sciParts(value, digits);
  return (
    <span className="num">
      {exponent === null ? (
        mantissa
      ) : (
        <>
          {mantissa} × 10<sup>{exponent}</sup>
        </>
      )}
      {unit ? <span className="unit"> {unit}</span> : null}
    </span>
  );
}
