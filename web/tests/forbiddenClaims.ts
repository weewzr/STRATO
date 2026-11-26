// Claims that the page must never make about the operating scenarios (science design v3, Section 5.1;
// owner directive of 26 Nov 2026, Section 3). A scenario is a prescribed-input simulation: STRATO does not
// model neutron kinetics, control-rod motion, thermal hydraulics, power redistribution or mechanical
// refuelling, and no scenario is a validated reactor prediction. A sentence that denies the claim ("does not
// model rod motion", "not a validated reactor prediction") is allowed; these patterns match only the
// affirmative forms. The unit tests scan the page text; the browser tests scan the built page.
export const FORBIDDEN_CLAIMS: readonly { name: string; pattern: RegExp }[] = [
  { name: 'models a reactor mechanism', pattern: /\b(models|simulates|predicts|solves|includes|computes)\s+(the\s+)?(neutron kinetics|control[- ]rod (motion|worth|reactivity)|thermal[- ]hydraulics|power redistribution|mechanical refuelling)/i },
  { name: 'validated reactor prediction', pattern: /(?<!not a )(?<!not )validated (reactor )?(prediction|simulation|scenario)/i },
  { name: 'reactor-validated', pattern: /reactor[- ]validated|validated against (a |the )?reactor/i },
  { name: 'measured scenario history', pattern: /\b(measured|experimental|observed) (power|temperature|fission[- ]rate|scenario) history/i },
  { name: 'control-rod term in the equation', pattern: /control[- ]rod term (is|enters|appears) (in|into) the (governing )?equation/i },
  { name: 'real-time reactor prediction', pattern: /(real[- ]time|plant) (reactor )?prediction/i },
];

/** The names of the forbidden claims that a text makes (empty when it makes none). */
export function claimsIn(text: string): string[] {
  return FORBIDDEN_CLAIMS.filter((c) => c.pattern.test(text)).map((c) => c.name);
}
