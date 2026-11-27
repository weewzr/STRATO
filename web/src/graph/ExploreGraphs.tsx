import { useId, useMemo, useRef, type KeyboardEvent } from 'react';
import { Plot } from '../plots/Plot';
import { usePlotTheme } from '../plots/theme';
import { GRAPH_IDS, type GraphId } from '../equation/selectionIds';
import { useStrato } from '../state/store';
import { datasetLabel, datasetsOf, offeredTemperatures, speciesFromOptions, stageRequest, type ComparisonRecord, type Species, type StageRequest } from './exploreData';
import { experimentFigure, methodsFigure, profileFigure, releaseFigure } from './exploreFigures';
import { useOptions, useStageResult, type StageResult } from './useStageResult';

// LOWER RIGHT of Explore: INTERACTIVE GRAPHS (final directive, Section 11 and decision 78(b)). Four compact
// toggles, one species select (the nuclides the wrapper lists), one temperature select (the 100 degC steps
// the registered diffusivity rows support) and, for the experimental comparison, one dataset select.
// Where no result exists for the selection (every operation except steady-state operation, future
// species-specific physics, a species or temperature without registered inputs, a reply not yet received)
// the panel is an empty frame: no text, no placeholder curve, nothing from another selection.

const GRAPH_LABEL: Record<GraphId, string> = {
  release: 'Cumulative release fraction against time',
  profile: 'Radial concentration profile',
  methods: 'Method A against Method B',
  experiment: 'Experimental comparison',
};

export interface GraphChoices {
  species: Species[];
  current: Species | null;
  temperatures: number[];
  temperature: number | null;
  datasets: ComparisonRecord[];
  dataset: ComparisonRecord | null;
  request: StageRequest | null;
  result: StageResult | null;
}

/** The species, temperature and dataset the selection resolves to, the request they give, and the wrapper's result for it. */
export function useGraphChoices(): GraphChoices {
  const sel = useStrato((s) => s.selection);
  const options = useOptions();
  const species = useMemo(() => speciesFromOptions(options), [options]);
  const current = species.find((s) => s.id === sel.species) ?? species[0] ?? null;
  const temperatures = useMemo(() => (current ? offeredTemperatures(current.element) : []), [current]);
  const temperature = sel.temperature !== null && temperatures.includes(sel.temperature) ? sel.temperature : (temperatures[0] ?? null);
  const datasets = useMemo(() => datasetsOf(current), [current]);
  const dataset = datasets.find((d) => d.curve_id === sel.dataset) ?? datasets[0] ?? null;
  // Only steady-state operation has curves; the other operations and future physics have none (their request is null).
  const request = useMemo(
    () => (sel.scenario === 'steady' && current !== null && temperature !== null ? stageRequest(sel.physics, current, temperature) : null),
    [sel.scenario, sel.physics, current, temperature],
  );
  const result = useStageResult(request);
  return { species, current, temperatures, temperature, datasets, dataset, request, result };
}

function Toggle({ value, onChange }: { value: GraphId; onChange: (g: GraphId) => void }) {
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const onKey = (e: KeyboardEvent<HTMLButtonElement>, i: number) => {
    let next: number | null = null;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') next = (i + 1) % GRAPH_IDS.length;
    if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') next = (i - 1 + GRAPH_IDS.length) % GRAPH_IDS.length;
    if (e.key === 'Home') next = 0;
    if (e.key === 'End') next = GRAPH_IDS.length - 1;
    if (next === null) return;
    e.preventDefault();
    const id = GRAPH_IDS[next];
    if (id) {
      onChange(id);
      refs.current[next]?.focus();
    }
  };
  return (
    <div className="xbtns xbtns--graphs" role="radiogroup" aria-label="Graph" data-testid="graph-toggles">
      {GRAPH_IDS.map((id, i) => (
        <button
          key={id}
          ref={(el) => {
            refs.current[i] = el;
          }}
          type="button"
          role="radio"
          aria-checked={id === value}
          tabIndex={id === value ? 0 : -1}
          className={`xbtn${id === value ? ' is-current' : ''}`}
          data-testid={`graph-${id}`}
          onClick={() => onChange(id)}
          onKeyDown={(e) => onKey(e, i)}
        >
          {GRAPH_LABEL[id]}
        </button>
      ))}
    </div>
  );
}

export function ExploreGraphs({ choices }: { choices: GraphChoices }) {
  const graph = useStrato((s) => s.selection.graph);
  const physics = useStrato((s) => s.selection.physics);
  const scenario = useStrato((s) => s.selection.scenario);
  const setSelection = useStrato((s) => s.setSelection);
  const theme = usePlotTheme();
  const ids = { species: useId(), temperature: useId(), dataset: useId(), title: useId() };
  const { species, current, temperatures, temperature, datasets, dataset } = choices;
  // Only steady-state operation has curves, and never Future species-specific physics. A result is drawn only
  // for the request of the selection shown: a result held for another selection is never drawn.
  const drawable = scenario === 'steady' && physics !== 'future';
  const expected = useMemo(() => (drawable && current !== null && temperature !== null ? stageRequest(physics, current, temperature) : null), [drawable, physics, current, temperature]);
  const result = expected !== null && choices.result?.key === expected.key ? choices.result : null;

  const figure = useMemo(() => {
    if (!drawable) return null;
    if (graph === 'experiment') return dataset !== null ? experimentFigure(dataset, theme) : null;
    if (result === null) return null;
    if (graph === 'release') return releaseFigure(result.reply, theme);
    if (graph === 'profile') return profileFigure(result.reply, theme);
    return methodsFigure(result.reply, result.analytical, theme);
  }, [drawable, graph, dataset, result, theme]);

  const showExperiment = graph === 'experiment' && figure !== null && dataset !== null;
  const name = current?.name ?? '';

  return (
    <section className="xarea xarea--graphs" aria-labelledby={ids.title} data-testid="interactive-graphs" data-area="graphs">
      <h2 className="xarea__title" id={ids.title}>
        Interactive graphs
      </h2>
      <Toggle value={graph} onChange={(g) => setSelection({ graph: g })} />
      <div className="xselects">
        <label className="xselect" htmlFor={ids.species}>
          <span className="xselect__label">Species</span>
          <select id={ids.species} value={current?.id ?? ''} disabled={species.length === 0} onChange={(e) => setSelection({ species: e.target.value, dataset: '' })} data-testid="species-select">
            {species.map((s) => (
              <option key={s.id} value={s.id}>
                {s.name}
              </option>
            ))}
          </select>
        </label>
        <label className="xselect" htmlFor={ids.temperature}>
          <span className="xselect__label">Temperature</span>
          <select
            id={ids.temperature}
            value={temperature === null ? '' : String(temperature)}
            disabled={temperatures.length === 0}
            onChange={(e) => setSelection({ temperature: Number(e.target.value) })}
            data-testid="temperature-select"
          >
            {temperatures.map((t) => (
              <option key={t} value={String(t)}>
                {`${String(t)} °C`}
              </option>
            ))}
          </select>
        </label>
        {graph === 'experiment' ? (
          <label className="xselect xselect--wide" htmlFor={ids.dataset}>
            <span className="xselect__label">Dataset</span>
            <select id={ids.dataset} value={dataset?.curve_id ?? ''} disabled={datasets.length === 0} onChange={(e) => setSelection({ dataset: e.target.value })} data-testid="dataset-select">
              {datasets.map((d) => (
                <option key={d.curve_id} value={d.curve_id}>
                  {datasetLabel(d)}
                </option>
              ))}
            </select>
          </label>
        ) : null}
      </div>
      <div className="gpanel" data-testid="graph-panel" data-graph={graph} data-empty={figure === null ? 'true' : 'false'}>
        {figure === null ? (
          <div className="gpanel__frame" data-testid="graph-empty" />
        ) : (
          <Plot
            figure={figure}
            testId={`explore-plot-${graph}`}
            height={320}
            quiet
            label={`${GRAPH_LABEL[graph]}${name ? `, ${name}` : ''}${graph !== 'experiment' && temperature !== null ? `, ${String(temperature)} °C` : ''}`}
          />
        )}
      </div>
      {showExperiment ? (
        <div className="gcite" data-testid="experiment-citation">
          <p className="gcite__caption" data-testid="experiment-caption">
            {dataset.caption.caption_text}
          </p>
          <p className="gcite__source">
            <span data-testid="experiment-apa">{dataset.caption.source_citation_apa}</span>{' '}
            <a href={dataset.caption.source_link} target="_blank" rel="noopener noreferrer" data-testid="experiment-link">
              {dataset.caption.source_link}
            </a>
          </p>
        </div>
      ) : null}
    </section>
  );
}
