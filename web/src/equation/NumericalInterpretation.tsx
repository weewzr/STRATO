import { useId } from 'react';
import { useStrato } from '../state/store';
import { simulationMethod } from './availability';
import { scenarioStatus, STATUS_DEFINITION } from './capability';
import { StepBody } from './EquationBuilder';
import { effectivePhysics, itemExpandId, itemStatus, itemVisible, NOTATION, NUMERICAL_ITEMS, scenarioNote, type NumericalItem } from './numerics';
import { SCENARIO_LABEL } from './scenarios';
import { StatusTag } from './StatusTag';
import { RichText } from './Tex';
import { useBuildCapability, useSelectionSupport } from './useCapability';

/** Id of the region in the shared selection (state/store.ts: `expanded`); its items are 'n1' to 'n14'. */
export const NUMERICAL_ID = 'numerical';

/** One item of the sequence: a heading button that expands and collapses it, and, when open, its steps and the scenario note. */
function Item({ item }: { item: NumericalItem }) {
  const open = useStrato((s) => s.selection.expanded.includes(itemExpandId(item.id)));
  const setExpanded = useStrato((s) => s.setExpanded);
  const physics = useStrato((s) => effectivePhysics(s.selection.physics, s.selection.scenario));
  const scenario = useStrato((s) => s.selection.scenario);
  const { cap, loading } = useBuildCapability();
  const ids = { head: useId(), panel: useId() };
  const status = itemStatus(cap, item);
  const note = scenarioNote(scenario, item.id);
  return (
    <li className="num-item" data-testid={`num-item-${item.id}`} data-item={item.id} data-open={open ? 'true' : 'false'}>
      <h3 className="num-item__head" id={ids.head}>
        <button
          type="button"
          className="num-item__toggle"
          aria-expanded={open}
          aria-controls={ids.panel}
          onClick={() => setExpanded(itemExpandId(item.id), !open)}
          data-testid={`num-toggle-${item.id}`}
        >
          <span className="num-item__num stage-num" data-testid={`num-number-${item.id}`}>
            {item.id}
          </span>
          <span className="num-item__title">{item.title}</span>
          {item.mechanism !== null ? <StatusTag status={status} loading={loading} id={`num-status-${item.id}`} /> : null}
          <span className="num-item__chevron" aria-hidden="true" />
        </button>
      </h3>
      <div id={ids.panel} className="num-item__body" role="region" aria-labelledby={ids.head} hidden={!open}>
        {open ? (
          <>
            {item.mechanism !== null && !loading ? (
              <p className="num-item__status" data-testid={`num-status-text-${item.id}`}>
                {STATUS_DEFINITION[status]}
              </p>
            ) : null}
            {scenario !== null && note !== null ? (
              <p className="scenario-note" data-testid={`scenario-note-${item.id}`} data-scenario={scenario}>
                <strong>{SCENARIO_LABEL[scenario]}</strong> <StatusTag status={scenarioStatus(cap, scenario)} loading={loading} id={`scenario-note-status-${item.id}`} /> <RichText text={note} />
              </p>
            ) : null}
            {item.stages.map((s) => (
              <StepBody key={s} stage={s} physics={physics} showTitle={item.stages.length > 1} />
            ))}
          </>
        ) : null}
      </div>
    </li>
  );
}

/**
 * Block C of the Explore page: the numerical derivation of the BTCS scheme, in one collapsed region of
 * fourteen items (owner directive of 26 Nov 2026, Sections 4 and 5; equation/numerics.ts). Which items are
 * shown follows the selected physics, and the selected operating scenario writes its note inside the items
 * where it acts; the equations equal those of the BTCS documents. Each item expands and collapses on its own,
 * and which are open is part of the shared selection, so it survives navigation and is written to the address.
 * Nothing here runs a simulation.
 */
export function NumericalInterpretation() {
  const open = useStrato((s) => s.selection.expanded.includes(NUMERICAL_ID));
  const setExpanded = useStrato((s) => s.setExpanded);
  const physics = useStrato((s) => s.selection.physics);
  const scenario = useStrato((s) => s.selection.scenario);
  const capabilities = useStrato((s) => s.capabilities);
  const support = useSelectionSupport();
  const method = simulationMethod(capabilities);
  const ids = { title: useId(), panel: useId() };
  const items = NUMERICAL_ITEMS.filter((it) => itemVisible(it, physics, scenario));
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
            <p className="numerical__status" data-testid="numerical-status" data-supported={support.supported ? 'true' : 'false'}>
              {method === 'loading'
                ? 'The solver that runs the simulation is loading.'
                : method === 'unavailable'
                  ? 'The solver of this page does not offer the BTCS capability at the live status, so the simulation shows no result.'
                  : support.supported
                    ? 'The simulation solves these equations with the verified finite-difference BTCS Rust solver.'
                    : 'No simulation runs for this selection; Live simulation gives the reason. The equations and explanations remain.'}
            </p>
            <p className="numerical__notation" data-testid="numerical-notation">
              <RichText text={NOTATION} />
            </p>
            <ol className="num-items" aria-label="Numerical interpretation, in the order of the derivation" data-testid="num-items">
              {items.map((it) => (
                <Item key={it.id} item={it} />
              ))}
            </ol>
          </>
        ) : null}
      </div>
    </section>
  );
}
