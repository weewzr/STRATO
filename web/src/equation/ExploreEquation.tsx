import { useId, useRef, useState, type KeyboardEvent } from 'react';
import { entryHref } from '../research/entries';
import { useStrato } from '../state/store';
import {
  CONFIG_LABELS,
  flagsToConfigId,
  SCENARIO_IDS,
  type PhysicsConfigId,
  type ScenarioId,
  type SpeciesPhysicsId,
} from './capability';
import { useSideScroll } from './EquationBuilder';
import { legendRows } from './legend';
import { equationPieces, isTimed, meaningsFor, physicsConfigAnnouncement, type Piece } from './physics';
import { SCENARIO_LABEL, SCENARIOS } from './scenarios';
import { Tex } from './Tex';

// UPPER LEFT, UPPER CENTRE AND RIGHT of Explore (Phase 3 Option B).
// "Select a Modelling Stage" uses four independent checkboxes (Baseline locked checked, plus Generation,
// Decay, and Temperature-dependent diffusion across P01–P08) and a "Species-Specific Physics" selector.
// "Operation" remains an independent group beneath: changing one never changes the other.

/** The documented research mechanisms that Species-specific physics links to (data/research/mechanisms.csv). */
export const FUTURE_LINKS: readonly { label: string; entry: string }[] = [
  { label: 'Silver trapping and detrapping', entry: 'docs/research/ag_trapping_detrapping.md' },
  { label: 'Grain-boundary transport', entry: 'docs/research/ag_grain_boundary_transport.md' },
  { label: 'Palladium-assisted SiC degradation', entry: 'docs/research/pd_assisted_degradation.md' },
  { label: 'Barrier failure', entry: 'docs/research/coating_degradation_failure.md' },
];

/** Class suffix of the mechanism colour tokens (tokens.css: --mech-source, --mech-decay, --mech-arrhenius). */
const COLOUR = { generation: 'source', decay: 'decay', temperature: 'arrhenius' } as const;

function RadioButtons<T extends string>({
  ids,
  value,
  label,
  name,
  testPrefix,
  onChange,
}: {
  ids: readonly T[];
  value: T | null;
  label: string;
  name: (id: T) => string;
  testPrefix: string;
  onChange: (id: T) => void;
}) {
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const onKey = (e: KeyboardEvent<HTMLButtonElement>, i: number) => {
    let next: number | null = null;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') next = (i + 1) % ids.length;
    if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') next = (i - 1 + ids.length) % ids.length;
    if (e.key === 'Home') next = 0;
    if (e.key === 'End') next = ids.length - 1;
    if (next === null) return;
    e.preventDefault();
    const id = ids[next];
    if (id !== undefined) {
      onChange(id);
      refs.current[next]?.focus();
    }
  };
  return (
    <div className="xbtns" role="radiogroup" aria-label={label} data-testid={`${testPrefix}-picker`}>
      {ids.map((id, i) => {
        const checked = id === value;
        return (
          <button
            key={id}
            ref={(el) => {
              refs.current[i] = el;
            }}
            type="button"
            role="radio"
            aria-checked={checked}
            tabIndex={checked || (value === null && i === 0) ? 0 : -1}
            className={`xbtn${checked ? ' is-current' : ''}`}
            data-testid={`${testPrefix}-${id}`}
            onClick={() => onChange(id)}
            onKeyDown={(e) => onKey(e, i)}
          >
            {name(id)}
          </button>
        );
      })}
    </div>
  );
}

/** UPPER LEFT: Select a Modelling Stage (checkboxes + Species-Specific Physics) and, beneath it, Operation. */
export function ExploreControls() {
  const mechanisms = useStrato((s) => s.selection.mechanisms);
  const speciesPhysics = useStrato((s) => s.selection.speciesPhysics);
  const scenario = useStrato((s) => s.selection.scenario);
  const setSelection = useStrato((s) => s.setSelection);
  const ids = { model: useId(), speciesPhys: useId(), operation: useId() };
  const configId: PhysicsConfigId = flagsToConfigId(mechanisms);
  const meta = CONFIG_LABELS[configId];

  return (
    <div className="xcontrols" data-area="controls">
      <section className="xarea xarea--model" aria-labelledby={ids.model} data-testid="model-equation">
        <div className="xarea__head">
          <h2 className="xarea__title" id={ids.model}>
            Select a Modelling Stage
          </h2>
          <span className="xconfig-tag" data-testid="config-id-label">
            {configId} — {meta.stageAlias}
          </span>
        </div>
        <div className="xchecks" role="group" aria-labelledby={ids.model} data-testid="physics-checkboxes">
          <label className="xcheck xcheck--locked">
            <input type="checkbox" checked disabled data-testid="mech-baseline" />
            <span>Baseline diffusion</span>
          </label>
          <label className="xcheck">
            <input
              type="checkbox"
              checked={mechanisms.generation}
              data-testid="mech-generation"
              onChange={(e) => setSelection({ mechanisms: { ...mechanisms, generation: e.target.checked } })}
            />
            <span>Fission-product generation</span>
          </label>
          <label className="xcheck">
            <input
              type="checkbox"
              checked={mechanisms.decay}
              data-testid="mech-decay"
              onChange={(e) => setSelection({ mechanisms: { ...mechanisms, decay: e.target.checked } })}
            />
            <span>Radioactive decay</span>
          </label>
          <label className="xcheck">
            <input
              type="checkbox"
              checked={mechanisms.temperature}
              data-testid="mech-temperature"
              onChange={(e) => setSelection({ mechanisms: { ...mechanisms, temperature: e.target.checked } })}
            />
            <span>Temperature-dependent diffusion</span>
          </label>
        </div>
        <label className="xselect xselect--species-phys" htmlFor={ids.speciesPhys}>
          <span className="xselect__label">Species-Specific Physics</span>
          <select
            id={ids.speciesPhys}
            value={speciesPhysics}
            data-testid="species-physics-select"
            onChange={(e) => setSelection({ speciesPhysics: e.target.value as SpeciesPhysicsId })}
          >
            <option value="none">None — Future development</option>
          </select>
        </label>
      </section>
      <section className="xarea xarea--operation" aria-labelledby={ids.operation} data-testid="operation">
        <h2 className="xarea__title" id={ids.operation}>
          Operation
        </h2>
        <RadioButtons<ScenarioId>
          ids={SCENARIO_IDS}
          value={scenario}
          label="Operation"
          name={(id) => SCENARIO_LABEL[id]}
          testPrefix="operation"
          onChange={(id) => setSelection({ scenario: id })}
        />
      </section>
    </div>
  );
}

function EquationPiece({ piece, timed }: { piece: Piece; timed: boolean }) {
  // "{}" gives the leading plus or minus the spacing of a binary operator; \displaystyle keeps the fractions full size.
  const lead = piece.id === 'generation' || piece.id === 'decay' ? '{}' : '';
  // While an operation changes symbols, the changed symbols carry the highlight (inside the TeX), not the whole pieces.
  const colour = piece.mechanism && !timed ? COLOUR[piece.mechanism] : null;
  return (
    <span
      className={`geq__piece geq__piece--${piece.id}${colour ? ` geq-new geq-new--${colour}` : ''}`}
      data-piece={piece.id}
      data-mechanism={piece.mechanism ?? undefined}
      data-scenario-changed={piece.scenario ? 'true' : undefined}
    >
      <Tex latex={String.raw`\displaystyle ${lead}${piece.shown}`} />
    </span>
  );
}

/** UPPER CENTRE AND RIGHT: the governing equation, large, with the legend and active mechanism/operation explanations directly beneath. */
export function ExploreEquation() {
  const physics = useStrato((s) => s.selection.physics);
  const mechanisms = useStrato((s) => s.selection.mechanisms);
  const scenario = useStrato((s) => s.selection.scenario);
  const configId = flagsToConfigId(mechanisms);
  const pieces = equationPieces(mechanisms, scenario);
  const timed = isTimed(scenario);
  const rows = legendRows(mechanisms, scenario);
  const meanings = meaningsFor(mechanisms);
  const box = useRef<HTMLDivElement>(null);
  const side = useSideScroll(box, `${configId}/${scenario ?? 'none'}`);
  const ids = { title: useId(), hint: useId() };

  // Live-region sentence: updated whenever config or scenario changes after initial render.
  const selKey = `${configId}/${scenario ?? 'none'}`;
  const [seen, setSeen] = useState<string | null>(null);
  const [message, setMessage] = useState('');
  if (seen !== selKey) {
    setSeen(selKey);
    if (seen !== null) setMessage(physicsConfigAnnouncement(mechanisms, scenario));
  }

  const opExplanation = scenario ? SCENARIOS[scenario].explanation : [];

  return (
    <section
      className="xarea xarea--equation geq"
      aria-labelledby={ids.title}
      data-testid="governing-equation"
      data-area="equation"
      data-physics={physics}
      data-config={configId}
      data-scenario={scenario ?? 'none'}
    >
      <h2 className="xarea__title" id={ids.title}>
        Governing equation
      </h2>
      <div className={`eq-frame${side.more ? ' eq-frame--more' : ''}`}>
        <div
          className="geq__eq"
          data-testid="equation-governing"
          ref={box}
          {...(side.scrolls
            ? { tabIndex: 0, role: 'region', 'aria-label': 'Governing equation, scrolls sideways', 'aria-describedby': ids.hint }
            : { role: 'group', 'aria-label': 'Governing equation' })}
        >
          {pieces.map((p) => (
            <EquationPiece key={p.id} piece={p} timed={timed} />
          ))}
        </div>
        <span className="eq-fade" aria-hidden="true" />
      </div>
      {side.scrolls ? (
        <p className="eq-scroll-hint" id={ids.hint}>
          <span aria-hidden="true">{side.more ? '→ ' : '← '}</span>
          Scroll sideways to see the whole equation.
        </p>
      ) : null}
      <p className="visually-hidden" role="status" aria-live="polite" data-testid="physics-announce">
        {message}
      </p>
      <table className="xlegend" data-testid="legend">
        <caption className="visually-hidden">Variables of the governing equation</caption>
        <thead>
          <tr>
            <th scope="col">Symbol</th>
            <th scope="col">Meaning</th>
            <th scope="col">Unit</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.id} data-legend={r.id}>
              <th scope="row">
                <Tex latex={r.symbol} />
              </th>
              <td>{r.meaning}</td>
              <td>
                <Tex latex={r.unit} />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <div className="geq-explain" data-testid="equation-explanations">
        <ul className="geq-explain__list">
          {meanings.map((m) => (
            <li key={m.id}>
              <strong>{m.name}:</strong> {m.meaning}
            </li>
          ))}
        </ul>
        {scenario ? (
          <div className="geq-explain__op" data-testid="operation-explanation">
            <strong>{SCENARIO_LABEL[scenario]} operation:</strong> {opExplanation.join(' ')}
          </div>
        ) : null}
      </div>
      {physics === 'future' ? (
        <ul className="xfuture" aria-label="Documented research mechanisms" data-testid="future-links">
          {FUTURE_LINKS.map((l) => (
            <li key={l.entry}>
              <a href={entryHref(l.entry)}>{l.label}</a>
            </li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}
