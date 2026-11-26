import { useId } from 'react';
import { useStrato } from '../state/store';
import { EquationBuilder } from './EquationBuilder';
import { SCENARIO_LABEL, SCENARIOS } from './scenarios';
import { TermExplanations } from './TermExplanations';

/** How the selected operating scenario enters the numerical scheme (directive Section 5); it follows the shared selection. */
function ScenarioNumerics() {
  const scenario = useStrato((s) => s.selection.scenario);
  if (scenario === null) return null;
  return (
    <div className="scenario-numerics" data-testid="scenario-numerics" data-scenario={scenario}>
      <h3 className="scenario-numerics__title">Selected scenario: {SCENARIO_LABEL[scenario]}</h3>
      <p>{SCENARIOS[scenario].numerics.join(' ')}</p>
    </div>
  );
}

/** Id of this region in the shared selection (state/store.ts: `expanded`). */
export const NUMERICAL_ID = 'numerical';

/**
 * Block C of the Explore page: the numerical derivation of the BTCS scheme, in one collapsed region
 * (owner directive of 26 Nov 2026, Sections 4 and 9). For now it holds the existing derivation steps
 * unchanged; their reorganisation into the directive's sequence and their synchronisation with the
 * selected physics and scenario are the next tasks. Whether it is open is part of the shared selection,
 * so it survives navigation and is written to the address.
 */
export function NumericalInterpretation({ compact }: { compact: boolean }) {
  const open = useStrato((s) => s.selection.expanded.includes(NUMERICAL_ID));
  const setExpanded = useStrato((s) => s.setExpanded);
  const ids = { title: useId(), panel: useId() };
  return (
    <section className="block block--c" data-block="C" aria-labelledby={ids.title} data-testid="block-c">
      <div className="block__head block__head--row">
        <h2 id={ids.title} className="block__title">
          Numerical Interpretation
        </h2>
        <p className="block__context">How the equation above is discretised by the backward-time central-space (BTCS) scheme and solved in Rust.</p>
      </div>
      <button
        type="button"
        className="btn disclosure"
        aria-expanded={open}
        aria-controls={ids.panel}
        onClick={() => setExpanded(NUMERICAL_ID, !open)}
        data-testid="numerical-toggle"
      >
        {open ? 'Hide the derivation steps' : 'Show the derivation steps'}
      </button>
      <div id={ids.panel} className="numerical" data-testid="numerical-interpretation" hidden={!open}>
        {open ? (
          <>
            <ScenarioNumerics />
            <EquationBuilder compact={compact} />
            <div className="prose">
              <TermExplanations />
            </div>
          </>
        ) : null}
      </div>
    </section>
  );
}
