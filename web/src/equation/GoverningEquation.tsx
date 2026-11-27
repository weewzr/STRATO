import { useId, useRef, useState, type KeyboardEvent } from 'react';
import { useStrato } from '../state/store';
import { BUILD_STATUSES, PHYSICS_IDS, STATUS_DEFINITION, STATUS_LABEL, mechanismsOf, physicsStatus, type PhysicsId } from './capability';
import { StatusTag } from './StatusTag';
import { useSideScroll } from './EquationBuilder';
import { equationPieces, isTimed, meaningsFor, PHYSICS_CHANGE, PHYSICS_CHANGE_SCENARIO, PHYSICS_LABEL, type Piece } from './physics';
import { Tex } from './Tex';
import { useBuildCapability } from './useCapability';

// Block A of the Explore page: the governing equation, the first and largest element (owner directive
// of 26 Nov 2026, Section 2). The baseline equation is shown first and kept; the controls below it add
// a term on the right (generation, decay), change a coefficient inside the operator (temperature) or
// show them all (combined). The base pieces stay mounted; only the introduced piece enters, highlighted.
// The status of every choice comes from the capability object (equation/capability.ts), never from here.

/** Class suffix of the mechanism colour tokens (tokens.css: --mech-source, --mech-decay, --mech-arrhenius). */
const COLOUR = { generation: 'source', decay: 'decay', temperature: 'arrhenius' } as const;

function EquationPiece({ piece, timed }: { piece: Piece; timed: boolean }) {
  // "{}" gives the leading plus or minus the spacing of a binary operator; \displaystyle keeps the fractions full size.
  const lead = piece.id === 'generation' || piece.id === 'decay' ? '{}' : '';
  // While an operating scenario changes symbols, the changed symbols carry the highlight (inside the TeX), not the whole pieces.
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

export function GoverningEquation() {
  const physics = useStrato((s) => s.selection.physics);
  const scenario = useStrato((s) => s.selection.scenario);
  const setSelection = useStrato((s) => s.setSelection);
  const { cap, loading } = useBuildCapability();
  const pieces = equationPieces(physics, scenario);
  const meanings = meaningsFor(physics);
  const introduced = new Set(mechanismsOf(physics));
  const ids = { group: useId(), legend: useId(), hint: useId() };
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const box = useRef<HTMLDivElement>(null);
  const side = useSideScroll(box, `${physics}/${scenario ?? 'none'}`);
  const [legendOpen, setLegendOpen] = useState(false);

  // The live-region sentence: set while rendering when the selection differs from the one last rendered
  // (not on the first render, and also when the address changes the selection).
  const [seen, setSeen] = useState<PhysicsId | null>(null);
  const [message, setMessage] = useState('');
  if (seen !== physics) {
    setSeen(physics);
    if (seen !== null) setMessage((isTimed(scenario) ? PHYSICS_CHANGE_SCENARIO[physics] : undefined) ?? PHYSICS_CHANGE[physics]);
  }

  const onKey = (e: KeyboardEvent<HTMLButtonElement>, id: PhysicsId) => {
    const i = PHYSICS_IDS.indexOf(id);
    let next: number | null = null;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') next = (i + 1) % PHYSICS_IDS.length;
    if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') next = (i - 1 + PHYSICS_IDS.length) % PHYSICS_IDS.length;
    if (e.key === 'Home') next = 0;
    if (e.key === 'End') next = PHYSICS_IDS.length - 1;
    if (next === null) return;
    e.preventDefault();
    setSelection({ physics: PHYSICS_IDS[next] as PhysicsId });
    refs.current[next]?.focus();
  };

  return (
    <div className="geq" data-testid="governing-equation" data-physics={physics} data-scenario={scenario ?? 'none'}>
      <div className={`eq-frame${side.more ? ' eq-frame--more' : ''}`}>
        <div
          className="geq__eq"
          data-testid="equation-governing"
          ref={box}
          {...(side.scrolls ? { tabIndex: 0, role: 'region', 'aria-label': 'Governing equation, scrolls sideways', 'aria-describedby': ids.hint } : { role: 'group', 'aria-label': 'Governing equation' })}
        >
          {pieces.map((p) => (
            <EquationPiece key={p.id} piece={p} timed={isTimed(scenario)} />
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

      <div className="geq__controls">
        <div className="physics" role="radiogroup" aria-label="Physics of the governing equation" data-testid="physics-picker">
          {PHYSICS_IDS.map((id, i) => {
            const checked = id === physics;
            const status = id === 'baseline' ? cap.baseline : physicsStatus(cap, id);
            return (
              <button
                key={id}
                ref={(el) => {
                  refs.current[i] = el;
                }}
                type="button"
                role="radio"
                aria-checked={checked}
                tabIndex={checked ? 0 : -1}
                className={`physics__btn${checked ? ' is-current' : ''}`}
                data-testid={`physics-${id}`}
                data-status={loading ? 'loading' : status}
                onClick={() => setSelection({ physics: id })}
                onKeyDown={(e) => onKey(e, id)}
              >
                <span className="physics__name">{PHYSICS_LABEL[id]}</span>
                <StatusTag status={status} loading={loading} id={`physics-status-${id}`} />
              </button>
            );
          })}
        </div>
        <p className="physics__legend">
          <button type="button" className="linklike" aria-expanded={legendOpen} aria-controls={ids.legend} onClick={() => setLegendOpen(!legendOpen)} data-testid="status-legend-toggle">
            What the statuses mean
          </button>
        </p>
        <dl className="physics__defs" id={ids.legend} hidden={!legendOpen} data-testid="status-legend">
          {BUILD_STATUSES.map((s) => (
            <div key={s}>
              <dt>{STATUS_LABEL[s]}</dt>
              <dd>{STATUS_DEFINITION[s]}</dd>
            </div>
          ))}
        </dl>
      </div>

      <p className="visually-hidden" role="status" aria-live="polite" data-testid="physics-announce">
        {message}
      </p>

      <ul className="geq__terms" aria-label="Meaning of each term of the equation" data-testid="term-meanings">
        {meanings.map((t) => {
          const isNew = t.mechanism !== null && introduced.has(t.mechanism);
          const status = t.mechanism ? cap.mechanisms[t.mechanism] : cap.baseline;
          return (
            <li key={t.id} className={`geq__term${isNew ? ' is-new' : ''}`} data-term={t.id} data-new={isNew ? 'true' : undefined}>
              <span className="geq__sym" aria-hidden="true">
                <Tex latex={t.symbol} />
              </span>
              <span className="geq__text">
                <span className="geq__name">
                  {t.name} <StatusTag status={status} loading={loading} />
                </span>
                <span className="geq__meaning">{t.meaning}</span>
                <span className="geq__unit">
                  Unit: <Tex latex={t.unit} />
                </span>
              </span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
