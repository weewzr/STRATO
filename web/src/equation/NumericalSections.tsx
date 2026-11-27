import { useId, type ReactNode } from 'react';
import { METHOD_1, METHOD_2, METHOD_2_DETAIL } from '../methods';
import { useStrato } from '../state/store';
import { mechanismsOf, type Mechanism, type PhysicsId, type ScenarioId } from './capability';
import { effectivePhysics, NOTATION, NUMERICAL_ITEMS, scenarioNote, type ItemId } from './numerics';
import { HOOKS, PSEUDOCODE } from './pseudocode';
import { SECTION_IDS, type SectionId } from './selectionIds';
import { conditionsFor, equationNumber, mainEquations, physicsAdditions, STEP_DEFS, type Stage } from './terms';
import { RichText, Tex } from './Tex';

// LOWER LEFT of Explore: NUMERICAL INTERPRETATION, six expandable sections, all collapsed by default
// (final directive, Sections 10 and decision 78(h)). The sections regroup the verified items and texts of
// the earlier Numerical Interpretation (equation/numerics.ts, equation/terms.ts): the equations, their
// numbers and their captions are moved, not rewritten. The pseudocode is the verbatim code of
// docs/mathematics/btcs/03_rust_pseudocode.md (equation/pseudocode.ts). What is shown follows the model
// equation (generation: the source in the right-hand side; decay: the diagonal; temperature: the
// coefficients at the mid-step temperature, with the matrix reassembled) and the operation (a note where
// it acts). Display only: nothing here runs a solver or evaluates a number.

export interface Applicability {
  applicable: boolean;
  extension: string | null;
  reason: string | null;
}

const SECTION_TITLE: Record<SectionId, string> = {
  'method-a': METHOD_1,
  'method-b': METHOD_2,
  discretisation: 'Mathematical discretisation',
  pseudocode: 'Pseudocode',
  assumptions: 'Assumptions',
  limitations: 'Limitations',
};

const withTag = (latex: string, tag: string | null): string => (tag ? `${latex} \\tag{${tag}}` : latex);

/** One derivation step in lean form: its name, its equations with their numbers and its caption, as terms.ts holds them. */
function StepBlock({ stage, physics, additions = true }: { stage: Stage; physics: PhysicsId; additions?: boolean }) {
  const def = STEP_DEFS[stage];
  const add = additions ? physicsAdditions(stage, physics) : { equations: [], note: null };
  return (
    <div className="nstep" data-stage={stage}>
      <h4 className="nstep__title">{def.name}</h4>
      {mainEquations(stage).map((e) => (
        <div className="nstep__eq" key={`${stage}-${e.id}`} data-eq={e.id}>
          <Tex display latex={withTag(e.latex, equationNumber(e))} />
        </div>
      ))}
      <p className="nstep__text">{def.summary}</p>
      {add.note !== null
        ? add.note.split('\n').map((p) => (
            <p className="nstep__text" key={p}>
              <RichText text={p} />
            </p>
          ))
        : null}
      {add.equations.map((e) => (
        <div className="nstep__eq" key={`${stage}-add-${e.id}`} data-eq={e.id}>
          <Tex display latex={withTag(e.latex, equationNumber(e))} />
        </div>
      ))}
    </div>
  );
}

function OperationNote({ item, physics, scenario }: { item: ItemId; physics: PhysicsId; scenario: ScenarioId | null }) {
  const row = NUMERICAL_ITEMS.find((i) => i.id === item);
  if (!row) return null;
  // A note belongs to the mechanism it acts on: it is shown only where that mechanism is part of the model equation.
  if (row.mechanism !== null && !mechanismsOf(effectivePhysics(physics)).includes(row.mechanism)) return null;
  const note = scenarioNote(scenario, item);
  if (note === null) return null;
  return (
    <p className="nstep__text nstep__text--operation" data-operation-note={item}>
      <RichText text={note} />
    </p>
  );
}

/** Mechanisms of the model equation, in the order of the steps 15 to 17. */
const MECHANISM_STAGE: Record<Mechanism, Stage> = { generation: 15, decay: 16, temperature: 17 };

function MethodA({ physics, scenario }: { physics: PhysicsId; scenario: ScenarioId | null }) {
  return (
    <>
      <StepBlock stage={11} physics={physics} />
      <OperationNote item={12} physics={physics} scenario={scenario} />
      <StepBlock stage={12} physics={physics} />
    </>
  );
}

function MethodB({ applicability }: { applicability: Applicability | null }) {
  return (
    <>
      <p className="nstep__text">
        {METHOD_2}: {METHOD_2_DETAIL}.
      </p>
      {applicability !== null ? (
        <p className="nstep__text" data-testid="method-b-applicability" data-applicable={applicability.applicable ? 'true' : 'false'}>
          {applicability.applicable ? `Applies here: ${applicability.extension ?? 'baseline'}.` : `Does not apply here: ${applicability.reason ?? ''}`}
        </p>
      ) : null}
    </>
  );
}

function Discretisation({ physics, scenario }: { physics: PhysicsId; scenario: ScenarioId | null }) {
  const effective = effectivePhysics(physics);
  const mechanisms = mechanismsOf(effective);
  return (
    <>
      <p className="nstep__text">
        <RichText text={NOTATION} />
      </p>
      {([5, 6, 7, 8, 9, 10] as const).map((s) => (
        <StepBlock key={s} stage={s} physics={effective} />
      ))}
      {mechanisms.map((m) => (
        <div key={m}>
          <StepBlock stage={MECHANISM_STAGE[m]} physics={effective} additions={false} />
          <OperationNote item={m === 'generation' ? 9 : m === 'decay' ? 10 : 11} physics={physics} scenario={scenario} />
        </div>
      ))}
    </>
  );
}

function Pseudocode({ physics }: { physics: PhysicsId }) {
  const mechanisms = mechanismsOf(effectivePhysics(physics));
  return (
    <>
      {PSEUDOCODE.map((b) => (
        <figure className="npseudo" key={b.id} data-block={b.id}>
          <figcaption className="npseudo__title">{b.title}</figcaption>
          <pre className="npseudo__code" tabIndex={0} aria-label={`Pseudocode of ${b.title}`}>
            <code>{b.code}</code>
          </pre>
        </figure>
      ))}
      {mechanisms.map((m) => {
        const hook = m === 'generation' ? HOOKS.G1 : m === 'decay' ? HOOKS.G2 : HOOKS.G3;
        return (
          <figure className="npseudo" key={m} data-hook={m}>
            <figcaption className="npseudo__title">{hook.mechanism}</figcaption>
            <pre className="npseudo__code" tabIndex={0} aria-label={`Hook of ${hook.mechanism}`}>
              <code>{`enters at: ${hook.entersAt}\n${hook.change}`}</code>
            </pre>
            {m === 'temperature' ? (
              <p className="nstep__text">
                <RichText text="The step diffusivity is taken at the temperature of the middle of the step, $\bar D_{\mathrm{material}}^j = D_{\mathrm{material}}\left(T\left(t^j + \tfrac{1}{2}\Delta t\right)\right)$, and the matrix is reassembled and its Thomas factors recomputed whenever $\bar D^j$ or $\Delta t$ differs from the previous step." />
              </p>
            ) : null}
          </figure>
        );
      })}
    </>
  );
}

function Assumptions() {
  const conditions = conditionsFor(4);
  return (
    <>
      {([1, 2, 3, 4] as const).map((s) => (
        <StepBlock key={s} stage={s} physics="baseline" additions={false} />
      ))}
      {conditions !== null ? (
        <div className="nstep">
          <p className="nstep__text">
            <RichText text={conditions.note} />
          </p>
          {conditions.equations.map((e) => (
            <div className="nstep__eq" key={e.id} data-eq={e.id}>
              <Tex display latex={withTag(e.latex, equationNumber(e))} />
            </div>
          ))}
        </div>
      ) : null}
    </>
  );
}

const LIMITATIONS: readonly string[] = [
  'The model has a constant diffusivity in each layer between changes of temperature, a source in the kernel only and an absorbing outer surface.',
  'An operation is a prescribed input history of the generation and the temperature; it adds no term to the equation.',
  'STRATO does not model neutron kinetics, control-rod motion, thermal hydraulics, power redistribution or mechanical refuelling.',
  'Method B applies where the problem is the one the Sturm–Liouville series solves; a temperature that changes in time is never drawn with it.',
  'A diffusivity correlation is evaluated outside its printed range and counted, never clamped.',
];

function Limitations() {
  return (
    <ul className="nlist">
      {LIMITATIONS.map((t) => (
        <li key={t}>{t}</li>
      ))}
    </ul>
  );
}

function Section({ id, children }: { id: SectionId; children: ReactNode }) {
  const open = useStrato((s) => s.selection.expanded.includes(id));
  const setExpanded = useStrato((s) => s.setExpanded);
  const ids = { head: useId(), panel: useId() };
  return (
    <li className="nsec" data-testid={`num-section-${id}`} data-open={open ? 'true' : 'false'}>
      <h3 className="nsec__head" id={ids.head}>
        <button
          type="button"
          className="nsec__toggle"
          aria-expanded={open}
          aria-controls={ids.panel}
          onClick={() => setExpanded(id, !open)}
          data-testid={`num-toggle-${id}`}
        >
          <span className="nsec__title">{SECTION_TITLE[id]}</span>
          <span className="nsec__chevron" aria-hidden="true" />
        </button>
      </h3>
      <div id={ids.panel} className="nsec__body" role="region" aria-labelledby={ids.head} hidden={!open}>
        {open ? children : null}
      </div>
    </li>
  );
}

/** The six sections. `applicability` is the wrapper's answer for the current request (null where none was asked). */
export function NumericalSections({ applicability }: { applicability: Applicability | null }) {
  const physics = useStrato((s) => s.selection.physics);
  const scenario = useStrato((s) => s.selection.scenario);
  const heading = useId();
  const body: Record<SectionId, ReactNode> = {
    'method-a': <MethodA physics={physics} scenario={scenario} />,
    'method-b': <MethodB applicability={applicability} />,
    discretisation: <Discretisation physics={physics} scenario={scenario} />,
    pseudocode: <Pseudocode physics={physics} />,
    assumptions: <Assumptions />,
    limitations: <Limitations />,
  };
  return (
    <section className="xarea xarea--numerical" aria-labelledby={heading} data-testid="numerical-interpretation" data-area="numerical">
      <h2 className="xarea__title" id={heading}>
        Numerical interpretation
      </h2>
      <ol className="nsecs">
        {SECTION_IDS.map((id) => (
          <Section key={id} id={id}>
            {body[id]}
          </Section>
        ))}
      </ol>
    </section>
  );
}
