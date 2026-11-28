import { useId, useMemo, useRef, type KeyboardEvent } from 'react';
import { CONFIG_LABELS, flagsToConfigId } from '../equation/capability';
import { GRAPH_IDS, type GraphId } from '../equation/selectionIds';
import { METHOD_1, METHOD_2 } from '../methods';
import { Plot } from '../plots/Plot';
import { usePlotTheme } from '../plots/theme';
import { useStrato } from '../state/store';
import {
  datasetLabel,
  datasetsAt,
  datasetsOf,
  offeredTemperatures,
  speciesFromOptions,
  stageRequest,
  type ComparisonRecord,
  type Species,
  type StageRequest,
} from './exploreData';
import {
  experimentFigure,
  methodsFigure,
  profileFigure,
  releaseFigure,
  scenarioCurveFigure,
  STUDY39,
  study39CurveFigure,
} from './exploreFigures';
import { useOptions, useStageResult, type StageResult } from './useStageResult';

// LOWER RIGHT of Explore: INTERACTIVE GRAPHS (Phase 3 Option B).
// Provides three independent overlay checkboxes (Method A — BTCS, Method B — Sturm–Liouville,
// Experimental data), canonical Phase 3 Cs-137 progressive physics matrix P01–P08 across
// 1200–1800 °C with Graph B styling and immutable single Δ endpoint annotations, authentic
// Stage 5 operating-scenario curves for Cs-137 and Sr-90, and live WASM curves for all other cases.

const GRAPH_LABEL: Record<GraphId, string> = {
  release: 'Cumulative release fraction against time',
  profile: 'Radial concentration profile',
  methods: 'Method A against Method B',
  experiment: 'Experimental comparison',
};

const CANONICAL_FALLBACK_SPECIES: readonly Species[] = [
  { id: 'cs137', name: 'Cs-137', element: 'Cs' },
  { id: 'sr90', name: 'Sr-90', element: 'Sr' },
];

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

/** Resolve the species, temperature, dataset, and WASM stage request for the active selection. */
export function useGraphChoices(): GraphChoices {
  const sel = useStrato((s) => s.selection);
  const options = useOptions();
  const species = useMemo(() => {
    const wasmList = speciesFromOptions(options);
    if (wasmList.length === 0) return [...CANONICAL_FALLBACK_SPECIES];
    const seen = new Set(wasmList.map((s) => s.id));
    const extras = CANONICAL_FALLBACK_SPECIES.filter((c) => !seen.has(c.id));
    return [...wasmList, ...extras];
  }, [options]);

  // Empty selection defaults to canonical Cs-137 when steady or WASM's first entry otherwise.
  const current =
    (sel.species ? species.find((s) => s.id === sel.species) : species.find((s) => s.id === 'cs137') ?? species[0]) ??
    null;

  const isCanonicalCsSteady = current?.id === 'cs137' && sel.scenario === 'steady' && sel.graph !== 'profile';
  const temperatures = useMemo(() => {
    if (isCanonicalCsSteady) return [...STUDY39.temperatures];
    return current ? offeredTemperatures(current.element) : [];
  }, [isCanonicalCsSteady, current]);

  const defaultTemp = isCanonicalCsSteady ? 1700 : (temperatures[0] ?? null);
  const temperature =
    sel.temperature !== null && temperatures.includes(sel.temperature) ? sel.temperature : defaultTemp;

  const request = useMemo(
    () =>
      sel.scenario === 'steady' && current !== null && temperature !== null && !isCanonicalCsSteady
        ? stageRequest(sel.physics, current, temperature)
        : sel.scenario === 'steady' && current !== null && temperature !== null && sel.graph === 'profile'
          ? stageRequest(sel.physics, current, temperature)
          : null,
    [sel.scenario, sel.physics, sel.graph, current, temperature, isCanonicalCsSteady],
  );
  const datasets = useMemo(
    () => (request !== null ? datasetsAt(datasetsOf(current), temperature) : []),
    [request, current, temperature],
  );
  const dataset = datasets.find((d) => d.curve_id === sel.dataset) ?? datasets[0] ?? null;
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
  const mechanisms = useStrato((s) => s.selection.mechanisms);
  const scenario = useStrato((s) => s.selection.scenario);
  const overlays = useStrato((s) => s.selection.overlays);
  const controlledCompare = useStrato((s) => s.selection.controlledCompare);
  const setSelection = useStrato((s) => s.setSelection);
  const theme = usePlotTheme();
  const ids = { species: useId(), temperature: useId(), dataset: useId(), title: useId() };
  const { species, current, temperatures, temperature } = choices;

  const configId = flagsToConfigId(mechanisms);
  const hasAnyOverlay = overlays.methodA || overlays.methodB || overlays.experiment;
  const isCanonicalCsSteady = current?.id === 'cs137' && scenario === 'steady' && graph !== 'profile' && physics !== 'future';
  const isScenarioCurve =
    scenario !== null && scenario !== 'steady' && (current?.id === 'cs137' || current?.id === 'sr90') && physics !== 'future' && graph !== 'profile';

  const drawable = scenario === 'steady' && physics !== 'future';
  const expected = useMemo(
    () => (drawable && !isCanonicalCsSteady && current !== null && temperature !== null ? stageRequest(physics, current, temperature) : null),
    [drawable, isCanonicalCsSteady, physics, current, temperature],
  );
  const result = expected !== null && choices.result?.key === expected.key ? choices.result : null;
  const datasets = useMemo(
    () => (expected !== null ? datasetsAt(datasetsOf(current, choices.datasets), temperature) : []),
    [expected, current, temperature, choices.datasets],
  );
  const dataset = datasets.find((d) => d.curve_id === choices.dataset?.curve_id) ?? datasets[0] ?? null;

  const canControlOverlay = isCanonicalCsSteady && (configId === 'P02' || configId === 'P05');

  const studyView = useMemo(() => {
    if (!isCanonicalCsSteady || temperature === null) return null;
    return study39CurveFigure(temperature, configId, overlays, theme, {
      controlled: controlledCompare,
      includeRef: configId !== 'P01',
    });
  }, [isCanonicalCsSteady, temperature, configId, overlays, theme, controlledCompare]);

  const scenarioView = useMemo(() => {
    if (!isScenarioCurve || !scenario || !current) return null;
    return scenarioCurveFigure(current.id as 'cs137' | 'sr90', scenario, temperature, overlays, theme);
  }, [isScenarioCurve, scenario, current, temperature, overlays, theme]);

  const wasmFigure = useMemo(() => {
    if (isCanonicalCsSteady || isScenarioCurve || !drawable) return null;
    if (graph === 'experiment') return dataset !== null ? experimentFigure(dataset, theme, physics === 'temperature', overlays) : null;
    if (result === null) return null;
    if (graph === 'release') return releaseFigure(result.reply, theme, overlays, result.analytical);
    if (graph === 'profile') return profileFigure(result.reply, theme);
    return methodsFigure(result.reply, result.analytical, theme, overlays);
  }, [isCanonicalCsSteady, isScenarioCurve, drawable, graph, dataset, result, theme, physics, overlays]);

  const figure = studyView?.figure ?? scenarioView?.figure ?? wasmFigure;
  const showExperiment = !isCanonicalCsSteady && !isScenarioCurve && graph === 'experiment' && figure !== null && dataset !== null;
  const name = current?.name ?? '';

  const methodBNotice = useMemo(() => {
    if (!overlays.methodB) return null;
    if (isScenarioCurve) {
      return `${METHOD_2} applies only to constant-temperature cases; it is omitted for time-varying operating histories.`;
    }
    if (studyView && !studyView.methodBdrawn && studyView.methodBReason) {
      return `${METHOD_2} is omitted here: ${studyView.methodBReason}`;
    }
    return null;
  }, [overlays.methodB, isScenarioCurve, studyView]);

  return (
    <section className="xarea xarea--graphs" aria-labelledby={ids.title} data-testid="interactive-graphs" data-area="graphs">
      <h2 className="xarea__title" id={ids.title}>
        Interactive graphs
      </h2>
      <Toggle value={graph} onChange={(g) => setSelection({ graph: g })} />
      <div className="xselects">
        <label className="xselect" htmlFor={ids.species}>
          <span className="xselect__label">Species</span>
          <select
            id={ids.species}
            value={current?.id ?? ''}
            disabled={species.length === 0}
            onChange={(e) => setSelection({ species: e.target.value, dataset: '' })}
            data-testid="species-select"
          >
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
        {!isCanonicalCsSteady && graph === 'experiment' && datasets.length > 0 ? (
          <label className="xselect xselect--wide" htmlFor={ids.dataset}>
            <span className="xselect__label">Dataset</span>
            <select
              id={ids.dataset}
              value={dataset?.curve_id ?? ''}
              disabled={datasets.length === 0}
              onChange={(e) => setSelection({ dataset: e.target.value })}
              data-testid="dataset-select"
            >
              {datasets.map((d) => (
                <option key={d.curve_id} value={d.curve_id}>
                  {datasetLabel(d)}
                </option>
              ))}
            </select>
          </label>
        ) : null}
      </div>

      <div className="gchecks" role="group" aria-label="Graph overlays" data-testid="graph-overlays">
        <label className="gcheck">
          <input
            type="checkbox"
            checked={overlays.methodA}
            data-testid="overlay-method-a"
            onChange={(e) => setSelection({ overlays: { ...overlays, methodA: e.target.checked } })}
          />
          <span>{METHOD_1}</span>
        </label>
        <label className="gcheck">
          <input
            type="checkbox"
            checked={overlays.methodB}
            data-testid="overlay-method-b"
            onChange={(e) => setSelection({ overlays: { ...overlays, methodB: e.target.checked } })}
          />
          <span>{METHOD_2}</span>
        </label>
        <label className="gcheck">
          <input
            type="checkbox"
            checked={overlays.experiment}
            data-testid="overlay-experiment"
            onChange={(e) => setSelection({ overlays: { ...overlays, experiment: e.target.checked } })}
          />
          <span>Experimental data</span>
        </label>
        {canControlOverlay ? (
          <label className="gcheck">
            <input
              type="checkbox"
              checked={controlledCompare}
              data-testid="overlay-controlled"
              onChange={(e) => setSelection({ controlledCompare: e.target.checked })}
            />
            <span>Controlled matched-inventory</span>
          </label>
        ) : null}
      </div>

      <div className="gpanel" data-testid="graph-panel" data-graph={graph} data-empty={figure === null ? 'true' : 'false'}>
        {figure === null ? (
          <div className="gpanel__frame" data-testid="graph-empty">
            {!hasAnyOverlay ? (
              <p className="gnote" data-testid="overlay-empty-note" style={{ padding: '1rem' }}>
                Select at least one overlay (Method A, Method B, or Experimental data) to display curves.
              </p>
            ) : null}
          </div>
        ) : (
          <Plot
            figure={figure}
            testId={`explore-plot-${graph}`}
            height={250}
            quiet
            label={`${GRAPH_LABEL[graph]}${name ? `, ${name}` : ''}${
              isCanonicalCsSteady && temperature !== null
                ? `, ${String(temperature)} °C (${configId} ${CONFIG_LABELS[configId].stageAlias})`
                : temperature !== null
                  ? `, ${String(temperature)} °C`
                  : ''
            }`}
          />
        )}
      </div>

      {methodBNotice ? (
        <p className="gnote" data-testid="method-b-note">
          {methodBNotice}
        </p>
      ) : null}

      {scenarioView?.series ? (
        <p className="gnote" data-testid="scenario-curve-note">
          BTCS operating-history release for {scenarioView.series.nuclide} from 1028 °C reference irradiation (shown with steady-state baseline).
        </p>
      ) : null}

      {showExperiment ? (
        <div className="gcite" data-testid="experiment-citation">
          <p className="gcite__caption" data-testid="experiment-caption">
            {dataset.caption.caption_text}
          </p>
          {physics === 'temperature' ? (
            <p className="gcite__caption" data-testid="experiment-model-note">
              Model curves: temperature-dependent diffusion over the registered heating history of this dataset, {dataset.caption.temperature_label}.
            </p>
          ) : null}
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
