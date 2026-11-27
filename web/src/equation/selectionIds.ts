// Identifiers of the parts of the Explore selection that are neither the model equation nor the
// operation: the six expandable sections of the Numerical Interpretation and the four graph toggles.
// No dependencies, so that the store and the address can import it.

/** The six sections of the Numerical Interpretation, in the order of the page; all collapsed by default. */
export const SECTION_IDS = ['method-a', 'method-b', 'discretisation', 'pseudocode', 'assumptions', 'limitations'] as const;
export type SectionId = (typeof SECTION_IDS)[number];

/** The four graph toggles of INTERACTIVE GRAPHS, in the order of the page. */
export const GRAPH_IDS = ['release', 'profile', 'methods', 'experiment'] as const;
export type GraphId = (typeof GRAPH_IDS)[number];
export const DEFAULT_GRAPH: GraphId = 'release';
