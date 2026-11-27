import { mechanismRow, useCatalogue } from '../research/catalogue';
import { StatusChip } from '../research/StatusChip';
import { docHref, DOCS } from './links';
import { STEP_DEFS, type Stage } from './terms';
import { RichText, Tex } from './Tex';

/**
 * Every part of a step, with its meaning in plain words, the units of every symbol, the assumptions and,
 * for a mechanism, the evidence behind its research status. It is the "Symbols, units and assumptions"
 * disclosure of the step, so each explanation stands in one place.
 */
export function TermExplanations({ stage }: { stage: Stage }) {
  const catalogue = useCatalogue();
  const def = STEP_DEFS[stage];
  const parts = def.parts;
  return (
    <div className="terms-explained" data-testid="term-explanations">
      {parts.map((p, i) => {
        const row = p.mechanism ? mechanismRow(catalogue, p.mechanism) : undefined;
        // The evidence of a mechanism is written once, under its first part.
        const firstOfMechanism = parts.findIndex((u) => u.mechanism === p.mechanism) === i;
        return (
          <section key={p.id} id={`term-${p.id}`} className="term-x" aria-labelledby={`term-${p.id}-h`} data-term={p.id}>
            <h5 id={`term-${p.id}-h`} className="term-x__title">
              <Tex latex={p.symbol} className="term-x__tex" />
              <span>{p.name}</span>
            </h5>
            {row ? (
              <p className="term-x__status">
                <StatusChip status={row.status} mechanism={row.mechanism_id} />{' '}
                <span className="term-x__basis">{firstOfMechanism ? row.status_basis : `Same mechanism as the part above.`}</span>
              </p>
            ) : null}
            <p>
              <RichText text={p.meaning} />
            </p>
            {p.units.length > 0 ? (
              <table className="table table--units">
                <caption className="visually-hidden">Symbols and units of {p.name.toLowerCase()}</caption>
                <thead>
                  <tr>
                    <th scope="col">Symbol</th>
                    <th scope="col">Meaning</th>
                    <th scope="col">Unit</th>
                  </tr>
                </thead>
                <tbody>
                  {p.units.map((u) => (
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
            {p.assumptions.length > 0 ? (
              <>
                <p className="term-x__sub">Assumptions</p>
                <ul className="prose-list">
                  {p.assumptions.map((a) => (
                    <li key={a}>{a}</li>
                  ))}
                </ul>
              </>
            ) : null}
            {row && row.missing_work && firstOfMechanism ? (
              <p className="term-x__missing">
                <span className="term-x__sub">Still missing.</span> {row.missing_work}
              </p>
            ) : null}
            <p className="term-x__link">
              Derivation:{' '}
              <a href={docHref(p.derivation)}>
                {DOCS[p.derivation.doc].title}, {p.derivation.section}
              </a>
            </p>
          </section>
        );
      })}
    </div>
  );
}
