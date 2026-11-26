import { useId, useLayoutEffect, useRef, useState, type KeyboardEvent } from 'react';
import { useStrato } from '../state/store';
import { SCENARIO_IDS, mechanismsOf, scenarioStatus, type ScenarioId } from './capability';
import { NO_SCENARIO_TEXT, paramsFor, SCENARIO_KIND, SCENARIO_LABEL, SCENARIO_RESET, SCENARIO_RULE, SCENARIOS, scenarioChange } from './scenarios';
import { StatusTag } from './StatusTag';
import { Tex } from './Tex';
import { useBuildCapability } from './useCapability';

/**
 * Block B of the Explore page: the row of operating-scenario buttons directly beneath the physics controls
 * (owner directive of 26 Nov 2026, Section 3). It behaves like the physics controls: one choice, arrow keys,
 * a status word on each button that comes from the capability object of the build. The selected scenario
 * changes the symbols of the equation above (G to G(r,t), D(r,T) to D(r,T(t))), and below the row stand its
 * short explanation and the definitions of the symbols it changes. Nothing here runs a simulation: a scenario
 * the build cannot solve is explained and labelled, and the simulation and results blocks say so.
 */
export function ScenarioPicker() {
  const scenario = useStrato((s) => s.selection.scenario);
  const physics = useStrato((s) => s.selection.physics);
  const setSelection = useStrato((s) => s.setSelection);
  const { cap, loading } = useBuildCapability();
  const ids = { title: useId(), group: useId() };
  const refs = useRef<(HTMLButtonElement | null)[]>([]);

  // The live-region sentence: set while rendering when the selection differs from the one last rendered.
  const [seen, setSeen] = useState<{ scenario: ScenarioId | null; physics: string } | null>(null);
  const [message, setMessage] = useState('');
  if (seen === null || seen.scenario !== scenario || seen.physics !== physics) {
    setSeen({ scenario, physics });
    if (seen !== null && seen.scenario !== scenario) {
      // No scenario any more: the physics chosen cleared it.
      setMessage(scenario === null ? SCENARIO_RESET : scenarioChange(scenario, seen.physics !== physics));
    }
  }

  // A selection can move the row (the term list above it changes with the physics). After the render that
  // follows a selection made here, the checked button is brought back into view, clear of the sticky bars.
  const moved = useRef(false);
  useLayoutEffect(() => {
    if (!moved.current) return;
    moved.current = false;
    const bring = () => refs.current[scenario === null ? 0 : SCENARIO_IDS.indexOf(scenario)]?.scrollIntoView?.({ block: 'nearest' });
    bring();
    const frame = requestAnimationFrame(bring);
    return () => cancelAnimationFrame(frame);
  }, [scenario, physics]);
  const choose = (id: ScenarioId) => {
    if (id !== scenario) moved.current = true;
    setSelection({ scenario: id });
  };

  const onKey = (e: KeyboardEvent<HTMLButtonElement>, id: ScenarioId) => {
    const i = SCENARIO_IDS.indexOf(id);
    let next: number | null = null;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') next = (i + 1) % SCENARIO_IDS.length;
    if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') next = (i - 1 + SCENARIO_IDS.length) % SCENARIO_IDS.length;
    if (e.key === 'Home') next = 0;
    if (e.key === 'End') next = SCENARIO_IDS.length - 1;
    if (next === null) return;
    e.preventDefault();
    choose(SCENARIO_IDS[next] as ScenarioId);
    refs.current[next]?.focus({ preventScroll: true });
  };

  const info = scenario === null ? null : SCENARIOS[scenario];
  const params = scenario === null ? [] : paramsFor(scenario, mechanismsOf(physics));

  return (
    <section className="block block--b" data-block="B" aria-labelledby={ids.title} data-testid="block-b" data-scenario={scenario ?? 'none'}>
      <div className="block__head block__head--row">
        <h2 id={ids.title} className="block__title">
          Operating scenarios
        </h2>
      </div>
      <div className="physics scenarios" role="radiogroup" aria-label="Operating scenario" data-testid="scenario-picker">
        {SCENARIO_IDS.map((id, i) => {
          const checked = id === scenario;
          const status = scenarioStatus(cap, id);
          return (
            <button
              key={id}
              ref={(el) => {
                refs.current[i] = el;
              }}
              type="button"
              role="radio"
              aria-checked={checked}
              tabIndex={checked || (scenario === null && i === 0) ? 0 : -1}
              className={`physics__btn${checked ? ' is-current' : ''}`}
              data-testid={`scenario-${id}`}
              data-status={loading ? 'loading' : status}
              onClick={() => choose(id)}
              onKeyDown={(e) => onKey(e, id)}
            >
              <span className="physics__name">{SCENARIO_LABEL[id]}</span>
              <StatusTag status={status} loading={loading} id={`scenario-status-${id}`} />
            </button>
          );
        })}
      </div>
      <p className="scenario__rule" data-testid="scenario-rule">
        {SCENARIO_RULE}
      </p>
      <p className="visually-hidden" role="status" aria-live="polite" data-testid="scenario-announce">
        {message}
      </p>
      <div className="scenario" data-testid="scenario-explanation" data-scenario={scenario ?? 'none'}>
        {scenario === null || info === null ? (
          <p className="scenario__text" data-testid="no-scenario">
            {NO_SCENARIO_TEXT}
          </p>
        ) : (
          <>
            <h3 className="scenario__title">
              {SCENARIO_LABEL[scenario]} <span className="scenario__kind">{SCENARIO_KIND}</span>
            </h3>
            <p className="scenario__text">{info.explanation.join(' ')}</p>
          </>
        )}
        {params.length > 0 ? (
          <ul className="geq__terms scenario__params" aria-label="Symbols of the selected scenario" data-testid="scenario-params">
            {params.map((p) => (
              <li key={p.id} className="geq__term" data-param={p.id}>
                <span className="geq__sym" aria-hidden="true">
                  <Tex latex={p.symbol} />
                </span>
                <span className="geq__text">
                  <span className="geq__name">{p.name}</span>
                  <span className="geq__meaning">{p.meaning}</span>
                  <span className="geq__unit">
                    Unit: <Tex latex={p.unit} />
                  </span>
                </span>
              </li>
            ))}
          </ul>
        ) : null}
      </div>
    </section>
  );
}
