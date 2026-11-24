// The five-level research status of a mechanism (owner decision M7; prompt/DESIGN_SPECIFICATION.md
// Section 7.1). This is the only module that names the five words (prompt/TESTING_AND_VALIDATION.md
// Section 3.1); the status of any mechanism is read from the generated catalogue, never typed here.
// A research status describes a mechanism; a result label (Badge) describes a number.

export const STATUSES = ['PENDING', 'RESEARCHED', 'IMPLEMENTED', 'VERIFIED', 'VALIDATED'] as const;
export type ResearchStatus = (typeof STATUSES)[number];

/** The one status at which a mechanism may drive the live simulation (with the solver capability). */
export const LIVE_STATUS: ResearchStatus = 'VERIFIED';

export type StatusShape = 'dotted' | 'dashed' | 'outline' | 'fill' | 'fill-check';

export const STATUS_INFO: Record<ResearchStatus, { shape: StatusShape; definition: string }> = {
  PENDING: { shape: 'dotted', definition: 'Research is proposed but has not been sufficiently established or implemented.' },
  RESEARCHED: { shape: 'dashed', definition: 'Relevant literature, candidate equations and parameter requirements have been documented.' },
  IMPLEMENTED: { shape: 'outline', definition: 'The numerical mechanism exists in STRATO but may not yet be fully verified.' },
  VERIFIED: { shape: 'fill', definition: 'Numerical implementation passes documented conservation, convergence and limiting-case tests.' },
  VALIDATED: {
    shape: 'fill-check',
    definition:
      'Predictions have been compared with relevant independent experimental evidence within a clearly defined applicability domain.',
  },
};

export function isStatus(value: unknown): value is ResearchStatus {
  return typeof value === 'string' && (STATUSES as readonly string[]).includes(value);
}
