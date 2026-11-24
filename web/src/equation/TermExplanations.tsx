import { mechanismRow, useCatalogue } from '../research/catalogue';
import { StatusChip } from '../research/StatusChip';
import { useStrato } from '../state/store';
import { docHref, DOCS } from './links';
import { STAGE_DEFS, stageTerms, TERMS } from './terms';
import { Tex } from './Tex';

/**
 * Block D, first part: every term of the stage shown in block A, with its meaning in plain words,
 * the units of every symbol, the assumptions and the evidence behind its research status.
 */
export function TermExplanations() {
  const stage = useStrato((s) => s.stage);
  const catalogue = useCatalogue();
  const terms = stageTerms(stage);
  return (
    <div className="terms-explained" data-testid="term-explanations">
      <h3>
        The terms of stage {stage}, {STAGE_DEFS[stage].name.toLowerCase()}
      </h3>
      {terms.map((t, i) => {
        const def = TERMS[t];
        const row = mechanismRow(catalogue, def.mechanism);
        // The evidence of a mechanism is written once, under its first term.
        const firstOfMechanism = terms.findIndex((u) => TERMS[u].mechanism === def.mechanism) === i;
        return (
          <section key={t} id={`term-${t}`} className="term-x" aria-labelledby={`term-${t}-h`} data-term={t}>
            <h4 id={`term-${t}-h`} className="term-x__title">
              {def.latex ? <Tex latex={def.latex} className="term-x__tex" /> : null}
              <span>{def.name}</span>
            </h4>
            {row ? (
              <p className="term-x__status">
                <StatusChip status={row.status} mechanism={row.mechanism_id} />{' '}
                <span className="term-x__basis">{firstOfMechanism ? row.status_basis : `Same mechanism as the term above.`}</span>
              </p>
            ) : null}
            <p>{def.meaning}</p>
            {def.formula ? <Tex display latex={`${def.formula.latex} \\tag{${def.formula.source.tag ?? ''}}`} /> : null}
            {def.units.length > 0 ? (
              <table className="table table--units">
                <caption className="visually-hidden">Symbols and units of {def.name.toLowerCase()}</caption>
                <thead>
                  <tr>
                    <th scope="col">Symbol</th>
                    <th scope="col">Meaning</th>
                    <th scope="col">Unit</th>
                  </tr>
                </thead>
                <tbody>
                  {def.units.map((u) => (
                    <tr key={u.symbol}>
                      <th scope="row">
                        <Tex latex={u.symbol} />
                      </th>
                      <td>{u.meaning}</td>
                      <td>
                        <Tex latex={u.unit} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            ) : null}
            <p className="term-x__sub">Assumptions</p>
            <ul className="prose-list">
              {def.assumptions.map((a) => (
                <li key={a}>{a}</li>
              ))}
            </ul>
            {row && row.missing_work && firstOfMechanism ? (
              <p className="term-x__missing">
                <span className="term-x__sub">Still missing.</span> {row.missing_work}
              </p>
            ) : null}
            <p className="term-x__link">
              Derivation:{' '}
              <a href={docHref(def.derivation)}>
                {DOCS[def.derivation.doc].title}, {def.derivation.section}
              </a>
            </p>
          </section>
        );
      })}
    </div>
  );
}
