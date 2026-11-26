// The two solution methods of STRATO (coordinator decisions 29 and 52): the only names under which a
// method appears on the page, in the product and in the exports. The retired finite-volume method is
// not offered and not named anywhere. Method 2 applies where the problem is the one the
// Sturm-Liouville series solves (constant diffusivity per layer, no source, no decay, constant
// temperature); the wrapper says where it does not.

export const METHOD_1 = 'Method 1: Finite Difference BTCS';
export const METHOD_2 = 'Method 2: Analytical Solution';
export const METHODS = [METHOD_1, METHOD_2] as const;

/** The solver's own description of Method 2, in words that name no symbol. */
export const METHOD_2_DETAIL = 'the Sturm-Liouville eigenfunction series of the five-layer sphere';
/** The label of the numerical finite-difference results of the page. */
export const SOLVER_WORDING = 'BTCS finite differences';