import { useEffect, useId, useRef, useState, type KeyboardEvent, type ReactNode } from 'react';
import { statusOf, useCatalogue, type Catalogue, type MechanismRow } from '../research/catalogue';
import { StatusChip } from '../research/StatusChip';
import { useStrato } from '../state/store';
import { solvedStage } from './availability';
import { codeHref, docHref, DOCS, type DocRef } from './links';
import {
  assembleStage,
  balanceFor,
  conditionsFor,
  DOMAIN,
  implementationFor,
  STAGE_DEFS,
  STAGES,
  stageTerms,
  systemTerms,
  TERMS,
  type ShownEquation,
  type Stage,
  type TermId,
} from './terms';
import { Tex } from './Tex';

// Block A of the Explore page: the staged equation builder (prompt/DESIGN_SPECIFICATION.md Section 3).
// Display only. Selecting a stage never changes the inputs of the simulation and never sends a
// solver request; whether block B solves the stage is decided by isAvailable() alone.

/** Short symbol of each term for the term list (display only). */
const SYMBOL: Record<TermId, string> = {
  lhs: String.raw`\partial c/\partial t`,
  diffusion: 'D_i',
  source: 'S(r,t)',
  decay: String.raw`\lambda c`,
  diffusion_T: 'D_i(T(t))',
  // Two symbols, stacked so that both fit the symbol column.
  arrhenius_layers: String.raw`\begin{gathered}D_{0,i,m}\\ Q_{i,m}\end{gathered}`,
  partition: '',
};

/** A reference to an equation, section or web phase: exempt from the "no numbers" rule of block A. */
function Ref({ children }: { children: ReactNode }) {
  return <span data-ref="">{children}</span>;
}

function DocLink({ refTo, prefix }: { refTo: DocRef; prefix?: string }) {
  return (
    <a href={docHref(refTo)}>
      {prefix ? `${prefix} ` : ''}
      {DOCS[refTo.doc].title}, <Ref>{refTo.section}</Ref>
    </a>
  );
}

function withTag(latex: string, tag: string | null): string {
  return tag ? `${latex} \\tag{${tag}}` : latex;
}

function ShownEq({ eq }: { eq: ShownEquation }) {
  return (
    <figure className="shown-eq" data-eq={eq.id}>
      <figcaption className="shown-eq__label">{eq.label}</figcaption>
      <div className="eq-box">
        <Tex display latex={withTag(eq.latex, eq.source.tag)} />
      </div>
      {eq.note ? <p className="shown-eq__note">{eq.note}</p> : null}
      <p className="shown-eq__source">
        Source: <DocLink refTo={{ doc: eq.source.doc, section: eq.source.section }} />
        {eq.source.tag ? null : ' (untagged line)'}
      </p>
    </figure>
  );
}

function StagePicker({ stage, onPick, catalogue, asSelect }: { stage: Stage; onPick: (s: Stage) => void; catalogue: Catalogue; asSelect: boolean }) {
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const labelId = useId();

  if (asSelect) {
    return (
      <div className="stage-select">
        <label className="field__label" htmlFor="stage-select">
          Builder stage
        </label>
        <select id="stage-select" className="field__input" value={stage} onChange={(e) => onPick(Number(e.target.value) as Stage)} data-testid="stage-select">
          {STAGES.map((s) => (
            <option key={s} value={s}>
              {`${s}. ${STAGE_DEFS[s].name}`}
            </option>
          ))}
        </select>
      </div>
    );
  }

  const onKey = (e: KeyboardEvent<HTMLButtonElement>, s: Stage) => {
    const i = STAGES.indexOf(s);
    let next: number | null = null;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') next = (i + 1) % STAGES.length;
    if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') next = (i - 1 + STAGES.length) % STAGES.length;
    if (e.key === 'Home') next = 0;
    if (e.key === 'End') next = STAGES.length - 1;
    if (next === null) return;
    e.preventDefault();
    onPick(STAGES[next] as Stage);
    refs.current[next]?.focus();
  };

  return (
    <div className="stages" role="radiogroup" aria-labelledby={labelId} data-testid="stage-picker">
      <span id={labelId} className="visually-hidden">
        Builder stage
      </span>
      {STAGES.map((s, i) => {
        const def = STAGE_DEFS[s];
        const status = def.mechanism ? statusOf(catalogue, def.mechanism) : undefined;
        const checked = s === stage;
        return (
          <button
            key={s}
            ref={(el) => {
              refs.current[i] = el;
            }}
            type="button"
            role="radio"
            aria-checked={checked}
            tabIndex={checked ? 0 : -1}
            className={`stage${checked ? ' is-current' : ''}${s < stage ? ' is-earlier' : ''}`}
            onClick={() => onPick(s)}
            onKeyDown={(e) => onKey(e, s)}
            data-testid={`stage-${s}`}
            title={def.name}
          >
            <span className="stage__num stage-num" aria-hidden="true">
              {s}
            </span>
            <span className="stage__name">
              <span className="visually-hidden">
                Stage <span className="stage-num">{s}</span>: {def.name}
              </span>
              <span aria-hidden="true">{def.short}</span>
            </span>
            {status && def.mechanism ? <StatusChip status={status} mechanism={def.mechanism} compact /> : <span className="stage__none">catalogue</span>}
          </button>
        );
      })}
    </div>
  );
}

function TermCard({ term, catalogue, id, floating }: { term: TermId; catalogue: Catalogue; id: string; floating: boolean }) {
  const def = TERMS[term];
  const status = statusOf(catalogue, def.mechanism);
  return (
    <div className={`term-card${floating ? ' term-card--float' : ''}`} id={id} data-testid="term-card" data-term={term} role="region" aria-label={`About the term: ${def.name}`}>
      <p className="term-card__eyebrow">
        Term, from stage <span className="stage-num">{def.stage}</span>
      </p>
      <p className="term-card__title">{def.name}</p>
      {status ? <StatusChip status={status} mechanism={def.mechanism} /> : null}
      <p className="term-card__meaning">{def.meaning}</p>
      {def.formula ? (
        <div className="eq-box eq-box--small">
          <Tex display latex={withTag(def.formula.latex, def.formula.source.tag)} />
        </div>
      ) : null}
      <p className="term-card__links">
        <DocLink refTo={def.derivation} prefix="Derivation:" />
        <span aria-hidden="true"> · </span>
        <a href={`#term-${term}`}>Units and assumptions in the explanation</a>
      </p>
    </div>
  );
}

/**
 * The terms of the stage, each with its research status. Hovering, focusing or tapping a term opens
 * its meaning card and outlines the term in the equation; Escape closes it.
 */
function TermList({
  stage,
  catalogue,
  shown,
  onPreview,
  pinned,
  onPin,
  cardId,
}: {
  stage: Stage;
  catalogue: Catalogue;
  shown: TermId | null;
  onPreview: (t: TermId | null) => void;
  pinned: TermId | null;
  onPin: (t: TermId | null) => void;
  cardId: string;
}) {
  const fresh = STAGE_DEFS[stage].newTerms;
  const terms = stageTerms(stage);
  return (
    <ul
      className="eq-terms"
      role="list"
      aria-label="Terms of this stage"
      onMouseLeave={() => onPreview(null)}
      onKeyDown={(e) => {
        if (e.key === 'Escape') {
          onPreview(null);
          onPin(null);
        }
      }}
    >
      {terms.map((t) => {
        const def = TERMS[t];
        const status = statusOf(catalogue, def.mechanism);
        return (
          <li key={t} className={`eq-term-row${fresh.includes(t) ? ' is-new' : ''}`}>
            <button
              type="button"
              className="eq-term-btn"
              aria-expanded={shown === t}
              aria-controls={cardId}
              onClick={() => onPin(pinned === t ? null : t)}
              onFocus={() => onPreview(t)}
              onBlur={() => onPreview(null)}
              onMouseEnter={() => onPreview(t)}
              data-term={t}
              data-testid={`term-${t}`}
            >
              <span className="eq-term-btn__sym">
                {SYMBOL[t] ? <Tex latex={SYMBOL[t]} /> : <span className="eq-term-btn__none">not in the model</span>}
              </span>
              <span className="eq-term-btn__text">
                <span className="eq-term-btn__name">{def.name}</span>
                <span className="eq-term-btn__meta">
                  {status ? <StatusChip status={status} mechanism={def.mechanism} compact /> : null}
                  {fresh.includes(t) ? (
                    <span className="eq-term-btn__flag eq-term-btn__flag--new">
                      new<span className="visually-hidden"> in this stage</span>
                    </span>
                  ) : null}
                </span>
              </span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}

/** A catalogue mechanism that needs coupled equations: shown as a system beside the transport equation. */
export function CoupledSystem({ row, catalogue }: { row: MechanismRow; catalogue: Catalogue }) {
  const entry = row.entry ? catalogue.entries[row.entry] : undefined;
  const concepts = entry?.concept_equations ?? [];
  return (
    <section className="coupled" data-testid={`coupled-${row.mechanism_id}`} aria-label={`${row.name}: research concept`}>
      <p className="coupled__head">
        <span className="coupled__name">{row.name}</span>
        <StatusChip status={row.status} mechanism={row.mechanism_id} compact />
      </p>
      {concepts.length > 0 ? (
        <div className="coupled__system">
          <p className="coupled__label">Coupled system, research concept, not solved by STRATO</p>
          {concepts.map((c) => (
            <div className="eq-box eq-box--small" key={c}>
              <Tex display latex={c} />
            </div>
          ))}
        </div>
      ) : (
        <p className="coupled__label">No candidate equation in the catalogue entry yet.</p>
      )}
      <button
        type="button"
        role="switch"
        aria-checked="false"
        aria-disabled="true"
        className="mech__switch mech__switch--catalogue"
        data-testid={`catalogue-switch-${row.mechanism_id}`}
      >
        <span className="mech__name">Live simulation</span>
        <span className="mech__term">Not available: status {row.status}; not implemented in STRATO</span>
        <span className="mech__state">Off</span>
      </button>
    </section>
  );
}

function PreviewBanner({ stage, solved, catalogue }: { stage: Stage; solved: Stage | 0; catalogue: Catalogue }) {
  if (solved >= stage) {
    return (
      <p className="live-note" data-testid="live-note">
        Live: the simulation below solves this equation with the verified Rust solver.
      </p>
    );
  }
  if (stage === 1) {
    return (
      <p className="live-note live-note--pending" data-testid="live-note">
        The simulation below is of this equation; the Rust solver that runs it is loading.
      </p>
    );
  }
  const mech = STAGE_DEFS[stage].mechanism;
  const status = mech ? statusOf(catalogue, mech) : undefined;
  return (
    <div className="preview-banner" role="note" data-testid="preview-banner">
      <p>
        <strong className="preview-banner__lead">
          Equation preview only: the simulation below solves stage <span className="stage-num">{solved > 0 ? solved : 1}</span>.
        </strong>{' '}
        {mech && status ? (
          <>
            Not implemented in the solver, status{' '}
            <span className="nowrap">
              <StatusChip status={status} mechanism={mech} compact />;
            </span>{' '}
            selecting it changes no input and no result.
          </>
        ) : (
          <>
            No species-specific mechanism is implemented and no catalogue entry is assigned to this stage yet (catalogue:{' '}
            <Ref>web phase 5.5</Ref>).
          </>
        )}
        {solved === 0 ? ' The solver is loading.' : null}
      </p>
    </div>
  );
}

interface MoreItem {
  id: string;
  title: string;
  refs: string;
  body: ReactNode;
}

/**
 * Disclosure buttons in one row, each opening its panel at full width under the row; one panel at a
 * time, all closed by default so the equation and the top of the simulation share the first screen.
 */
function MoreRow({ items }: { items: MoreItem[] }) {
  const [open, setOpen] = useState<string | null>(null);
  const base = useId();
  return (
    <div className="more">
      <div className="more__row" role="group" aria-label="More about this stage">
        {items.map((it) => (
          <button
            key={it.id}
            type="button"
            className="more__toggle"
            aria-expanded={open === it.id}
            aria-controls={`${base}-${it.id}`}
            onClick={() => setOpen(open === it.id ? null : it.id)}
            data-testid={`more-${it.id}`}
          >
            <span className="more__title">{it.title}</span>
            {it.refs ? (
              <span className="more__refs">
                <Ref>{it.refs}</Ref>
              </span>
            ) : null}
          </button>
        ))}
      </div>
      {items.map((it) => (
        <div key={it.id} id={`${base}-${it.id}`} className="more__body" hidden={open !== it.id} data-testid={`more-panel-${it.id}`}>
          {it.body}
        </div>
      ))}
    </div>
  );
}

/** Equation numbers of a list, shortened to "first to last" for a run of more than two. */
function tagsOf(eqs: ShownEquation[]): string {
  const tags = eqs.map((e) => e.source.tag).filter((t): t is string => Boolean(t));
  if (tags.length > 2) return `(${tags[0]}) to (${tags[tags.length - 1]})`;
  return tags.map((t) => `(${t})`).join(', ');
}

function useWide(query: string): boolean {
  const [wide, setWide] = useState(() => typeof window !== 'undefined' && typeof window.matchMedia === 'function' && window.matchMedia(query).matches);
  useEffect(() => {
    if (typeof window.matchMedia !== 'function') return;
    const mq = window.matchMedia(query);
    const on = () => setWide(mq.matches);
    mq.addEventListener('change', on);
    return () => mq.removeEventListener('change', on);
  }, [query]);
  return wide;
}

export function EquationBuilder({ compact }: { compact: boolean }) {
  const stage = useStrato((s) => s.stage);
  const setStage = useStrato((s) => s.setStage);
  const capabilities = useStrato((s) => s.capabilities);
  const catalogue = useCatalogue();
  const def = STAGE_DEFS[stage];
  const asm = assembleStage(stage);
  const [preview, setPreview] = useState<TermId | null>(null);
  const [pinned, setPinned] = useState<TermId | null>(null);
  const [shownFor, setShownFor] = useState<Stage>(stage);
  if (shownFor !== stage) {
    setShownFor(stage);
    setPreview(null);
    setPinned(null);
  }
  const shown = preview ?? pinned;
  const outlined = shown ?? 'none';
  const cardId = useId();
  const floating = useWide('(min-width: 1024px)');
  const solved = solvedStage(capabilities, catalogue);
  const impl = implementationFor(stage);
  const conds = conditionsFor(stage);
  const balance = balanceFor(stage);
  const catalogueRows = catalogue.mechanisms.filter((m) => m.builder_stage === 6);

  return (
    <div className="builder" data-testid="equation-builder" data-stage={stage}>
      <StagePicker stage={stage} onPick={setStage} catalogue={catalogue} asSelect={compact} />
      <div className="builder__main">
        <div className="builder__eq" data-testid="equation-block">
          <div className="eq-box eq-box--main" data-testid="equation" data-active-term={outlined} data-tag={asm.tag}>
            <Tex display latex={withTag(asm.latex, asm.tag)} key={`main-${stage}`} />
            {systemTerms(stage).map((t) => (
              <div className="eq-system" key={`${stage}-${t}`} data-testid={`system-${t}`} role="group" aria-label="Coupled with one temperature law in each layer">
                <span className="eq-system__label" aria-hidden="true">
                  and in each layer
                </span>
                <Tex
                  display
                  latex={withTag(
                    `\\htmlClass{eq-term eq-term--${t}${def.newTerms.includes(t) ? ' eq-new eq-new--arrhenius' : ''}}{${TERMS[t].latex}}`,
                    TERMS[t].sources[0]?.tag ?? null,
                  )}
                />
              </div>
            ))}
          </div>
          <p className="eq-where" data-testid="equation-caption">
            <strong className="eq-where__stage">
              Stage <span className="stage-num">{stage}</span>, {def.name.toLowerCase()}.
            </strong>{' '}
            {def.summary} In each layer <Tex latex={DOMAIN.latex} className="nowrap" />; equation <Ref>({asm.tag})</Ref>
            {asm.reductions.length > 0 ? (
              <>
                {' '}
                with{' '}
                {asm.reductions.map((r, i) => (
                  <span key={r}>
                    {i > 0 ? ' and ' : null}
                    <Tex latex={r} />
                  </span>
                ))}
              </>
            ) : null}
            {stage === 1 ? (
              <>
                , written out per layer as <Ref>(2.7)</Ref>
              </>
            ) : null}
            .
          </p>
          {stage === 6 && catalogueRows.length > 0 ? (
            <div className="stage6" data-testid="stage6">
              {catalogueRows.map((r) => (
                <CoupledSystem key={`${r.mechanism_id}-${r.species}-${r.material_domain}`} row={r} catalogue={catalogue} />
              ))}
            </div>
          ) : null}
          <PreviewBanner stage={stage} solved={solved} catalogue={catalogue} />
        </div>
        <div className="builder__side">
          <TermList stage={stage} catalogue={catalogue} shown={shown} onPreview={setPreview} pinned={pinned} onPin={setPinned} cardId={cardId} />
          {shown ? <TermCard term={shown} catalogue={catalogue} id={cardId} floating={floating} /> : <div id={cardId} hidden />}
        </div>
      </div>
      <MoreRow
        items={[
          {
            id: 'implementation',
            title: 'Numerical implementation',
            refs: tagsOf(impl.equations),
            body: (
              <>
                <p className="more__note">{impl.note}</p>
                {impl.equations.map((e) => (
                  <ShownEq key={e.id} eq={e} />
                ))}
                {impl.code.length > 0 ? (
                  <ul className="more__links" aria-label="Code at this build">
                    {impl.code.map((c) => (
                      <li key={c.symbol}>
                        Code: <a href={codeHref(c)}>{c.label}</a> in <code>{c.path}</code>
                      </li>
                    ))}
                  </ul>
                ) : null}
                {impl.pseudocode.length > 0 ? (
                  <ul className="more__links" aria-label="Pseudocode">
                    {impl.pseudocode.map((p) => (
                      <li key={p.section}>
                        <DocLink refTo={p} prefix="Pseudocode:" />
                      </li>
                    ))}
                  </ul>
                ) : null}
                {impl.derivation.length > 0 ? (
                  <ul className="more__links" aria-label="Derivation">
                    {impl.derivation.map((p) => (
                      <li key={p.section}>
                        <DocLink refTo={p} prefix="Derivation:" />
                      </li>
                    ))}
                  </ul>
                ) : null}
              </>
            ),
          },
          {
            id: 'conditions',
            title: 'Conditions',
            refs: tagsOf(conds.equations),
            body: (
              <>
                <p className="more__note">{conds.note}</p>
                {conds.equations.map((e) => (
                  <ShownEq key={e.id} eq={e} />
                ))}
              </>
            ),
          },
          {
            id: 'balance',
            title: 'Inventory balance',
            refs: tagsOf(balance.equations),
            body: (
              <>
                {balance.equations.map((e) => (
                  <ShownEq key={e.id} eq={e} />
                ))}
                <p className="more__note">{balance.note}</p>
              </>
            ),
          },
          {
            id: 'compare',
            title: 'Compare with stage one',
            refs: '',
            body:
              stage === 1 ? (
                <p className="more__note">Stage one is the reference that later stages are compared with.</p>
              ) : (
                <p className="more__note">
                  Not offered: this stage is not solved. Once the solver implements it, this place shows the release curve of this stage
                  beside the stage-one curve and their difference, all computed by the Rust solver (<Ref>web phase 6</Ref>); nothing is
                  subtracted in the browser.
                </p>
              ),
          },
        ]}
      />
    </div>
  );
}
