import { useEffect, useId, useRef, useState, type KeyboardEvent, type ReactNode } from 'react';
import { statusOf, useCatalogue, type Catalogue, type MechanismRow } from '../research/catalogue';
import { distinctStatuses, entryHref, rowKey } from '../research/entries';
import { CatalogueSwitch, RowChip } from '../research/parts';
import { StatusChip } from '../research/StatusChip';
import { hrefFor } from '../route';
import { useStrato } from '../state/store';
import { simulationMethod, type SimulationMethod } from './availability';
import { codeHref, docHref, DOCS, type DocRef } from './links';
import { LiveValues } from './LiveValues';
import {
  balanceFor,
  conditionsFor,
  CORE_STAGES,
  equationNumber,
  mainEquations,
  PREVIEW_STAGES,
  STAGES,
  STEP_DEFS,
  TERMS,
  type ShownEquation,
  type Stage,
  type TermId,
} from './terms';
import { RichText, Tex } from './Tex';
import { useSelectionSupport } from './useCapability';

// Block A of the Explore page: the progressive equation builder for the BTCS solver
// (prompt/DESIGN_SPECIFICATION.md Section 3). Display only. Selecting a step never changes the
// inputs of the simulation and never sends a solver request; whether block B solves the equations
// shown is decided by simulationMethod() alone.

/** A reference to an equation, section or web phase: exempt from the "no numbers" rule of block A. */
function Ref({ children }: { children: ReactNode }) {
  return <span data-ref="">{children}</span>;
}

function DocLink({ refTo, prefix }: { refTo: DocRef & { label?: string }; prefix?: string }) {
  return (
    <a href={docHref(refTo)}>
      <Ref>
        {prefix ? `${prefix} ` : ''}
        {DOCS[refTo.doc].title}, {refTo.label ?? refTo.section}
      </Ref>
    </a>
  );
}

function withTag(latex: string, tag: string | null): string {
  return tag ? `${latex} \\tag{${tag}}` : latex;
}

/** An equation that does not fit beside its number gets the number on its own line, now and when the box is resized or opened. */
function useTagPlacement(ref: { current: HTMLElement | null }, dep: unknown) {
  useEffect(() => {
    const box = ref.current;
    if (!box) return;
    const place = () => {
      for (const display of Array.from(box.querySelectorAll<HTMLElement>('.katex-display'))) {
        display.classList.remove('eq-tag-below');
        const html = display.querySelector<HTMLElement>(':scope > .katex > .katex-html');
        const tag = html?.querySelector<HTMLElement>(':scope > .katex-tag');
        const bases = html ? Array.from(html.querySelectorAll<HTMLElement>(':scope > .katex-base')) : [];
        if (!html || !tag || bases.length === 0) continue;
        const width = bases.reduce((sum, b) => sum + b.getBoundingClientRect().width, 0);
        // The equation is centred, so it needs room for the number on both sides.
        if (width + 2 * tag.getBoundingClientRect().width + 16 > display.clientWidth) display.classList.add('eq-tag-below');
      }
    };
    place();
    if (typeof ResizeObserver === 'undefined') return;
    const ro = new ResizeObserver(place);
    ro.observe(box);
    return () => ro.disconnect();
  }, [ref, dep]);
}

/**
 * Whether the box holds more than it shows sideways, and whether more lies to the right of what is shown.
 * Used for the fading edge and the "scroll sideways" label, so that a cut-off equation is not mistaken for a complete one.
 */
export function useSideScroll(ref: { current: HTMLElement | null }, dep: unknown): { scrolls: boolean; more: boolean } {
  const [state, setState] = useState({ scrolls: false, more: false });
  useEffect(() => {
    const box = ref.current;
    if (!box) return;
    const measure = () => {
      const scrolls = box.scrollWidth > box.clientWidth + 1;
      const more = scrolls && box.scrollLeft + box.clientWidth < box.scrollWidth - 1;
      setState((s) => (s.scrolls === scrolls && s.more === more ? s : { scrolls, more }));
    };
    measure();
    box.addEventListener('scroll', measure, { passive: true });
    if (typeof ResizeObserver === 'undefined') return () => box.removeEventListener('scroll', measure);
    const ro = new ResizeObserver(measure);
    ro.observe(box);
    for (const child of Array.from(box.children)) ro.observe(child);
    return () => {
      box.removeEventListener('scroll', measure);
      ro.disconnect();
    };
  }, [ref, dep]);
  return state;
}

function ShownEq({ eq }: { eq: ShownEquation }) {
  const box = useRef<HTMLDivElement>(null);
  useTagPlacement(box, eq.id);
  return (
    <figure className="shown-eq" data-eq={eq.id}>
      <figcaption className="shown-eq__label">{eq.label}</figcaption>
      <div className="eq-box" ref={box}>
        <Tex display latex={withTag(eq.latex, equationNumber(eq))} />
      </div>
      {eq.note ? (
        <p className="shown-eq__note">
          <RichText text={eq.note} />
        </p>
      ) : null}
      <p className="shown-eq__source">
        Source: <DocLink refTo={{ doc: eq.source.doc, section: eq.source.section }} />
        {eq.source.tag ? null : ' (untagged line)'}
      </p>
    </figure>
  );
}

function StagePicker({ stage, onPick, asSelect }: { stage: Stage; onPick: (s: Stage) => void; asSelect: boolean }) {
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const labelId = useId();

  if (asSelect) {
    return (
      <div className="stage-select">
        <label className="field__label" htmlFor="stage-select">
          Builder step
        </label>
        <select id="stage-select" className="field__input" value={stage} onChange={(e) => onPick(Number(e.target.value) as Stage)} data-testid="stage-select">
          <optgroup label="BTCS solver">
            {CORE_STAGES.map((s) => (
              <option key={s} value={s}>
                {`${s}. ${STEP_DEFS[s].name}`}
              </option>
            ))}
          </optgroup>
          <optgroup label="Previews, not solved">
            {PREVIEW_STAGES.map((s) => (
              <option key={s} value={s}>
                {`${s}. ${STEP_DEFS[s].name}`}
              </option>
            ))}
          </optgroup>
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
        Builder step
      </span>
      {STAGES.map((s, i) => {
        const def = STEP_DEFS[s];
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
            className={`stage${checked ? ' is-current' : ''}${s < stage ? ' is-earlier' : ''}${def.kind === 'preview' ? ' is-preview' : ''}${s === PREVIEW_STAGES[0] ? ' is-group-start' : ''}`}
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
                Step <span className="stage-num">{s}</span>: {def.name}
              </span>
              <span aria-hidden="true">{def.short}</span>
            </span>
          </button>
        );
      })}
    </div>
  );
}

function TermCard({ term, catalogue, id, floating, onLeave }: { term: TermId; catalogue: Catalogue; id: string; floating: boolean; onLeave: () => void }) {
  const def = TERMS[term];
  if (!def) return null;
  const status = def.mechanism ? statusOf(catalogue, def.mechanism) : undefined;
  return (
    <div className={`term-card${floating ? ' term-card--float' : ''}`} id={id} data-testid="term-card" data-term={term} role="region" aria-label={`About the part: ${def.name}`} onBlur={(e) => {
        // Keyboard focus leaving the card for anything outside it closes the preview.
        if (!e.currentTarget.contains(e.relatedTarget as Node | null)) onLeave();
      }}>
      <p className="term-card__eyebrow">
        Part of step <span className="stage-num">{def.stage}</span>
      </p>
      <p className="term-card__title">{def.name}</p>
      {status && def.mechanism ? <StatusChip status={status} mechanism={def.mechanism} /> : null}
      <p className="term-card__meaning">
        <RichText text={def.meaning} />
      </p>
      <p className="term-card__links">
        <DocLink refTo={def.derivation} prefix="Derivation:" />
        <span aria-hidden="true"> · </span>
        <a href={`#term-${term}`}>Units and assumptions in the explanation</a>
      </p>
    </div>
  );
}

/**
 * The parts of the step, each with its symbol. Hovering, focusing or tapping a part opens its
 * meaning card and outlines it in the equation; Escape closes it.
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
  const parts = STEP_DEFS[stage].parts;
  return (
    <ul
      className="eq-terms"
      role="list"
      aria-label="Parts of this step"
      onMouseLeave={() => onPreview(null)}
      onKeyDown={(e) => {
        if (e.key === 'Escape') {
          onPreview(null);
          onPin(null);
        }
      }}
    >
      {parts.map((def) => {
        const status = def.mechanism ? statusOf(catalogue, def.mechanism) : undefined;
        return (
          <li key={def.id} className="eq-term-row">
            <button
              type="button"
              className="eq-term-btn"
              aria-expanded={shown === def.id}
              aria-controls={cardId}
              onClick={() => onPin(pinned === def.id ? null : def.id)}
              onFocus={() => onPreview(def.id)}
              onBlur={(e) => {
                // Tab from the last part moves focus into the card that this part opened (its links):
                // the card must stay, or the focused link would be removed and focus would fall to the page.
                if (document.getElementById(cardId)?.contains(e.relatedTarget as Node | null)) return;
                onPreview(null);
              }}
              onMouseEnter={() => onPreview(def.id)}
              data-term={def.id}
              data-testid={`term-${def.id}`}
            >
              <span className="eq-term-btn__sym">
                <Tex latex={def.symbol} />
              </span>
              <span className="eq-term-btn__text">
                <span className="eq-term-btn__name">{def.name}</span>
                {status && def.mechanism ? (
                  <span className="eq-term-btn__meta">
                    <StatusChip status={status} mechanism={def.mechanism} compact />
                  </span>
                ) : null}
              </span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}

/** A decimal or exponent number in TeX (the "no numbers in block A" rule of the builder tests). */
const NUMERIC_CONSTANT = /\d\.\d|\de[+-]?\d/;

/** A catalogue mechanism that needs coupled equations: shown as a system beside the transport equation. */
export function CoupledSystem({ row, catalogue }: { row: MechanismRow; catalogue: Catalogue }) {
  const entry = row.entry ? catalogue.entries[row.entry] : undefined;
  // Block A shows no number apart from equation numbers and units (Section 3.3): a research-concept
  // equation that carries numerical constants is named here and shown on its entry page instead.
  const all = entry?.concept_equations ?? [];
  const concepts = all.filter((c) => !NUMERIC_CONSTANT.test(c));
  const withConstants = all.length - concepts.length;
  return (
    <section className="coupled" data-testid={`coupled-${row.mechanism_id}`} aria-label={`${row.name}: research concept`}>
      <p className="coupled__head">
        <span className="coupled__name">{row.entry ? <a href={entryHref(row.entry)}>{row.name}</a> : row.name}</span>
        <StatusChip status={row.status} mechanism={row.mechanism_id} row={rowKey(row)} compact />
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
      ) : all.length === 0 ? (
        <p className="coupled__label">No candidate equation in the catalogue entry yet.</p>
      ) : null}
      {withConstants > 0 && row.entry ? (
        <p className="coupled__label" data-testid={`coupled-more-${row.mechanism_id}`}>
          Further candidate equations with numerical constants are on the{' '}
          <a href={entryHref(row.entry, 'candidate-governing-equations-research-concepts')}>entry page</a>.
        </p>
      ) : null}
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

/**
 * Stage 6 lists the species-specific catalogue rows (prompt/DESIGN_SPECIFICATION.md Section 3.1):
 * none is implemented, so the list sits behind one disclosure, closed by default, under the
 * equation. A mechanism with coupled equations is shown once as its coupled system; every other
 * row as a line with its status and its disabled switch.
 */
function Stage6Catalogue({ rows, catalogue }: { rows: MechanismRow[]; catalogue: Catalogue }) {
  const [open, setOpen] = useState(false);
  const panel = useId();
  const coupledShown = new Set<string>();
  return (
    <div className="stage6" data-testid="stage6">
      <button type="button" className="stage6__toggle" aria-expanded={open} aria-controls={panel} onClick={() => setOpen(!open)} data-testid="stage6-toggle">
        <span className="stage6__title">Species-specific catalogue entries (research list, not part of this step)</span>
        <span className="stage6__counts">
          <span className="stage6__n">{distinctStatuses(rows).length === 1 ? 'every row at' : 'rows at'}</span>
          {distinctStatuses(rows).map((s) => (
            <StatusChip key={s} status={s} legend compact />
          ))}
        </span>
      </button>
      <div id={panel} className="stage6__panel" hidden={!open}>
        {rows.map((r) => {
          if (r.coupled) {
            if (coupledShown.has(r.mechanism_id)) return null;
            coupledShown.add(r.mechanism_id);
            return <CoupledSystem key={rowKey(r)} row={r} catalogue={catalogue} />;
          }
          return (
            <div className="stage6__row" key={rowKey(r)}>
              <p className="stage6__name">
                {r.entry ? <a href={entryHref(r.entry)}>{r.name}</a> : r.name} <RowChip row={r} compact />
              </p>
              <CatalogueSwitch row={r} compact />
            </div>
          );
        })}
        <p className="stage6__more">
          Full entries in the <a href={hrefFor('research')}>research catalogue</a>.
        </p>
      </div>
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

/**
 * What the simulation below has to do with the equations of the step. A step of the BTCS solver is
 * live only when the wrapper reports the BTCS capability at the live status; otherwise the page says
 * so instead of calling the equation live.
 */
function Banner({ stage, method, catalogue }: { stage: Stage; method: SimulationMethod; catalogue: Catalogue }) {
  const def = STEP_DEFS[stage];
  const support = useSelectionSupport();
  if (def.kind === 'preview') {
    const status = def.mechanism ? statusOf(catalogue, def.mechanism) : undefined;
    return (
      <div className="preview-banner" role="note" data-testid="preview-banner">
        <p>
          <strong className="preview-banner__lead">Preview only: this mechanism is not solved.</strong>{' '}
          {def.mechanism && status ? (
            <>
              Research status{' '}
              <span className="nowrap">
                <StatusChip status={status} mechanism={def.mechanism} compact />;
              </span>{' '}
              the equation is a placeholder; nothing changes in the inputs or results.
            </>
          ) : (
            <>Nothing changes in the inputs or results.</>
          )}
        </p>
      </div>
    );
  }
  if (stage === 14) {
    return (
      <div className="preview-banner" role="note" data-testid="preview-banner">
        <p>
          <strong className="preview-banner__lead">Placeholder: no experimental comparison is shown.</strong> The comparison data and their metrics are not part of this build.
        </p>
      </div>
    );
  }
  if (stage === 13) {
    return (
      <p className="live-note" data-testid="live-note">
        Numerical verification checks that the code solves its own equations. It is not experimental validation. The checks of the solver and their results are under Verify; the tests are in tests/btcs_verification.rs.
      </p>
    );
  }
  if (method === 'btcs' && !support.supported) {
    return (
      <p className="live-note live-note--pending" data-testid="live-note">
        No simulation runs for the selected physics or scenario: the solver in this build does not implement it. These are the BTCS equations of the selection.
      </p>
    );
  }
  if (method === 'btcs') {
    return (
      <p className="live-note" data-testid="live-note">
        Live: the simulation below solves these equations with the verified finite-difference BTCS Rust solver.
      </p>
    );
  }
  return (
    <p className="live-note live-note--pending" data-testid="live-note">
      {method === 'loading'
        ? 'These are BTCS equations. The solver that runs the simulation below is loading.'
        : 'These are the BTCS equations. The solver reported by this page does not offer the BTCS capability at the live status, so the simulation below shows no result.'}
    </p>
  );
}

/** Equation numbers of a list, shortened to "first to last" for a run of more than two. */
function tagsOf(eqs: ShownEquation[]): string {
  const tags = eqs.map(equationNumber).filter((t): t is string => Boolean(t));
  if (tags.length > 2) return `(${tags[0]}) to (${tags[tags.length - 1]})`;
  return tags.map((t) => `(${t})`).join(', ');
}

export function EquationBuilder({ compact }: { compact: boolean }) {
  const stage = useStrato((s) => s.stage);
  const setStage = useStrato((s) => s.setStage);
  const capabilities = useStrato((s) => s.capabilities);
  const catalogue = useCatalogue();
  const def = STEP_DEFS[stage];
  const equations = mainEquations(stage);
  const numbers = equations.map(equationNumber).filter((t): t is string => Boolean(t));
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
  const method = simulationMethod(capabilities, catalogue);
  const impl = def.implementation;
  const conds = conditionsFor(stage);
  const balance = balanceFor(stage);
  const catalogueRows = catalogue.mechanisms.filter((m) => m.builder_stage === 6);
  const eqBox = useRef<HTMLDivElement>(null);

  // The part picked in the list is outlined in the equation (its class is set after KaTeX has rendered).
  useEffect(() => {
    const box = eqBox.current;
    if (!box) return;
    for (const el of Array.from(box.querySelectorAll<HTMLElement>('.eq-term'))) {
      el.classList.toggle('is-active', shown !== null && el.classList.contains(`eq-term--${shown}`));
    }
  }, [shown, stage]);

  // An equation that does not fit beside its number gets the number on its own line.
  useTagPlacement(eqBox, stage);
  const side = useSideScroll(eqBox, stage);
  const hintId = useId();

  return (
    <div className="builder" data-testid="equation-builder" data-stage={stage}>
      <StagePicker stage={stage} onPick={setStage} asSelect={compact} />
      <div className="builder__main">
        <div className="builder__eq" data-testid="equation-block">
          <div className={`eq-frame${side.more ? ' eq-frame--more' : ''}`}>
            <div
              className="eq-box eq-box--main"
              data-testid="equation"
              data-active-term={outlined}
              data-tag={numbers.join(' ')}
              ref={eqBox}
              {...(side.scrolls ? { tabIndex: 0, role: 'region', 'aria-label': `Equation of step ${stage}, scrolls sideways`, 'aria-describedby': hintId } : {})}
            >
              {equations.map((e, i) => (
                <div className={i === 0 ? 'eq-first' : 'eq-system'} key={`${stage}-${e.id}`} data-eq={e.id}>
                  <span className="eq-label">{e.label}</span>
                  <Tex display latex={withTag(e.latex, equationNumber(e))} />
                </div>
              ))}
            </div>
            <span className="eq-fade" aria-hidden="true" />
          </div>
          {side.scrolls ? (
            <p className="eq-scroll-hint" id={hintId} data-testid="equation-scroll-hint">
              <span aria-hidden="true">{side.more ? '→ ' : '← '}</span>
              Scroll sideways to see the whole equation.
            </p>
          ) : null}
          <p className="eq-where" data-testid="equation-caption">
            <strong className="eq-where__stage">
              Step <span className="stage-num">{stage}</span>, {def.name}.
            </strong>{' '}
            {def.summary}
          </p>
          {stage === STAGES[STAGES.length - 1] && catalogueRows.length > 0 ? <Stage6Catalogue rows={catalogueRows} catalogue={catalogue} /> : null}
          <Banner stage={stage} method={method} catalogue={catalogue} />
        </div>
        <div className="builder__side">
          <TermList stage={stage} catalogue={catalogue} shown={shown} onPreview={setPreview} pinned={pinned} onPin={setPinned} cardId={cardId} />
          {shown ? <TermCard term={shown} catalogue={catalogue} id={cardId} floating={floating} onLeave={() => setPreview(null)} /> : <div id={cardId} hidden />}
        </div>
      </div>
      <MoreRow
        items={[
          {
            id: 'implementation',
            title: 'Related equations and code',
            refs: tagsOf(impl.equations),
            body: (
              <>
                <p className="more__note">
                  <RichText text={impl.note ?? ''} />
                </p>
                {impl.equations.map((e) => (
                  <ShownEq key={e.id} eq={e} />
                ))}
                {impl.code.length > 0 ? (
                  <ul className="more__links" aria-label="Code of the BTCS solver">
                    {impl.code.map((c) => (
                      <li key={`${c.path}:${c.symbol}`}>
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
                <p className="more__note">
                  <RichText text={conds.note} />
                </p>
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
                <p className="more__note">
                  <RichText text={balance.note} />
                </p>
              </>
            ),
          },
          {
            id: 'live',
            title: 'Live values',
            refs: '',
            body: (
              <>
                <p className="more__note" data-testid="live-values">
                  {def.live}
                </p>
                <LiveValues stage={stage} />
                {stage === 13 ? (
                  <p className="more__note">
                    <a href={hrefFor('verify')}>Verification results</a>
                  </p>
                ) : null}
                {stage === 14 ? (
                  <p className="more__note">
                    <a href={hrefFor('validate')}>Comparison with experiments</a>
                  </p>
                ) : null}
              </>
            ),
          },
        ]}
      />
    </div>
  );
}
