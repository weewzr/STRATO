import { Component, lazy, Suspense, useId, useState, type ReactNode } from 'react';
import type { CaseProvenance, CaseConfig } from '../data/case';
import { layerColour } from '../design/layers';
import { sciText } from '../format';
import { Badge, Sci } from '../panels/Badge';
import type { RunOk } from '../solver/types';
import { useShownRun } from '../solver/shown';
import { useStrato, type ColormapName } from '../state/store';
import { COLORMAP_LABEL, css, LOG_DECADES, legendTicks, scaleTop, table } from './colormap';
import { CrossSection2D } from './CrossSection2D';
import { ringsOf } from './geometry';
import { hasWebGL2 } from './webgl';

// The simulated particle in block B (web phase 4; prompt/DESIGN_SPECIFICATION.md Section 4.1): the
// 2D cross-section by default and the 3D cutaway as an option of the same size, the colour legend,
// layer inspection with the layer inventory and share the wrapper returned for the playback row,
// and the figure caption with source, model version and commit. Every number here is a field of
// the wrapper's RunResponse or an input; nothing is computed (owner decision M4).

const Particle3D = lazy(() => import('./Particle3D'));

class ThreeBoundary extends Component<{ onError: (message: string) => void; children: ReactNode }, { failed: boolean }> {
  override state = { failed: false };
  static getDerivedStateFromError() {
    return { failed: true };
  }
  override componentDidCatch(error: unknown) {
    this.props.onError(error instanceof Error ? error.message : String(error));
  }
  override render() {
    return this.state.failed ? null : this.props.children;
  }
}

/** Time of the playback row, from the wrapper; re-renders on playback without touching the drawing. */
function RowTime() {
  const { run } = useShownRun();
  const tIndex = useStrato((s) => s.playback.tIndex);
  const t = run?.profiles[tIndex]?.t_s;
  if (t === undefined) return null;
  return (
    <span className="figure__time" data-testid="particle-time" data-value={String(t)}>
      at time <Sci value={t} unit="s" digits={4} />
    </span>
  );
}

function Legend({ run }: { run: RunOk }) {
  const map = useStrato((s) => s.view.colormap);
  const log = useStrato((s) => s.view.logColour);
  const scale = log ? 'log' : 'linear';
  const entries = table(map);
  const stops = entries.filter((_, i) => i % 15 === 0 || i === entries.length - 1);
  const gradient = `linear-gradient(to right, ${stops.map((rgb, i) => `${css(rgb)} ${((i / (stops.length - 1)) * 100).toFixed(2)}%`).join(', ')})`;
  const top = scaleTop(run);
  return (
    <div className="legend" data-testid="colour-legend" data-colormap={map} data-scale={scale}>
      <p className="legend__title">
        Concentration (mol/m³), {scale === 'log' ? `log scale, ${LOG_DECADES} decades,` : 'linear scale,'} up to <Sci value={top} unit="mol/m³" />, the largest of the run.
      </p>
      <div className="legend__bar" style={{ backgroundImage: gradient }} aria-hidden="true" />
      <div className="legend__ticks" aria-hidden="true">
        {legendTicks(scale).map((t) => (
          <span key={t.label} style={{ left: `${t.position * 100}%` }}>
            {t.label}
          </span>
        ))}
      </div>
      <p className="visually-hidden">
        Colour map {COLORMAP_LABEL[map]}, {scale === 'log' ? `logarithmic over ${LOG_DECADES} decades below the largest concentration of the run` : 'linear from 0 to the largest concentration of the run'}.
      </p>
    </div>
  );
}

/** Display options of the colouring (scale and colour table); they change no input and no result. */
function ColourOptions() {
  const map = useStrato((s) => s.view.colormap);
  const log = useStrato((s) => s.view.logColour);
  const setView = useStrato((s) => s.setView);
  const scale = log ? 'log' : 'linear';
  return (
    <div className="legend__options particle__options" role="group" aria-label="Colour scale">
      <label>
        <span>Scale</span>
        <select value={scale} onChange={(e) => setView({ logColour: e.target.value === 'log' })} data-testid="colour-scale">
          <option value="linear">linear</option>
          <option value="log">log ({LOG_DECADES} decades)</option>
        </select>
      </label>
      <label>
        <span>Colour map</span>
        <select value={map} onChange={(e) => setView({ colormap: e.target.value as ColormapName })} data-testid="colour-map">
          <option value="cividis">cividis</option>
          <option value="viridis">viridis</option>
        </select>
      </label>
    </div>
  );
}

/** Inspector of one layer: inputs of the run, and the layer inventory and share at the playback row. */
function LayerDetail({ index, run, config }: { index: number; run: RunOk | null; config: CaseConfig }) {
  const tIndex = useStrato((s) => s.playback.tIndex);
  const regions: { name: string; outer_radius_m: number; diffusivity_m2_s: number; cells: number }[] = run
    ? run.inputs.regions.map((r) => ({ name: r.name, outer_radius_m: r.outer_radius_m, diffusivity_m2_s: r.diffusivity_m2_s, cells: r.intervals }))
    : config.regions;
  const sel = regions[index];
  if (!sel) return null;
  const profile = run?.profiles[tIndex];
  const share = profile?.layer_share?.[index];
  const inv = profile?.layer_inventory_mol[index];
  return (
    <dl className="inspector__detail" data-testid="inspector-detail" data-layer={index}>
      <dt>Layer</dt>
      <dd>
        <span className="layer-dot" style={{ ['--ring' as string]: layerColour(index) }} aria-hidden="true" />
        {sel.name}
      </dd>
      <dt>Inner and outer radius of the layer (m)</dt>
      <dd>
        {index === 0 ? <span className="num">0</span> : <Sci value={regions[index - 1]?.outer_radius_m ?? NaN} />} to{' '}
        <Sci value={sel.outer_radius_m} unit="m" />
      </dd>
      <dt>Diffusivity of the layer (m²/s)</dt>
      <dd>
        <Sci value={sel.diffusivity_m2_s} unit="m²/s" />
      </dd>
      <dt>Grid intervals in the layer (dimensionless)</dt>
      <dd className="num">{sel.cells}</dd>
      {run && profile ? (
        <>
          <dt>Inventory in the layer at the playback time of {sciText(profile.t_s, 4)} s (mol)</dt>
          <dd data-testid="inspector-inventory" data-value={inv === undefined ? '' : String(inv)} data-label={run.label}>
            {inv === undefined ? '–' : <Sci value={inv} unit="mol" digits={4} />}
          </dd>
          <dt>Share of the inventory in the particle (dimensionless)</dt>
          <dd data-testid="inspector-share" data-value={share === undefined ? '' : String(share)} data-label={run.label}>
            {share === undefined ? <span className="muted">not returned (inventory not positive)</span> : <Sci value={share} digits={4} />}{' '}
            <Badge label={run.label} />
          </dd>
        </>
      ) : null}
    </dl>
  );
}

export function ParticleView({ config }: { config: CaseConfig; provenance?: CaseProvenance }) {
  const { run, stale } = useShownRun();
  const selected = useStrato((s) => s.view.selectedLayer);
  const selectLayer = useStrato((s) => s.selectLayer);
  const view = useStrato((s) => s.view.particleView);
  const map = useStrato((s) => s.view.colormap);
  const logColour = useStrato((s) => s.view.logColour);
  const setView = useStrato((s) => s.setView);
  const [hovered, setHovered] = useState<number | null>(null);
  const [webgl, setWebgl] = useState<boolean | null>(null);
  const [failed3d, setFailed3d] = useState<string | null>(null);
  const captionId = useId();
  const listId = useId();


  // The drawing follows the run that coloured it (its echoed inputs); without a run, the inputs.
  const layers = run
    ? run.inputs.regions.map((r) => ({ name: r.name, outer_radius: r.outer_radius_m }))
    : config.regions.map((r) => ({ name: r.name, outer_radius: r.outer_radius_m }));
  const rings = ringsOf(layers);
  const names = layers.map((l) => l.name);
  const show3d = view === '3d' && run !== null && webgl === true && failed3d === null;
  const detail = hovered ?? selected;

  const label2d = run
    ? `Cross-section of the particle coloured by the concentration the solver returned at the playback time, ${COLORMAP_LABEL[map]} colour map; ${names.length} layers from ${names[0] ?? ''} at the centre to ${names[names.length - 1] ?? ''} at the surface, boundaries drawn in the layer colours. The layer inventories are listed in the inspector and in the inventory table.`
    : `Cross-section of the particle: ${names.length} concentric layers drawn to the outer radii of the active parameters. Schematic only, no concentration data.`;

  let note3d: string | null = null;
  if (view === '3d') {
    if (!run) note3d = 'The 3D view shows a solver result; it appears when the solver has run these inputs.';
    else if (webgl === false) note3d = 'This browser does not offer WebGL 2, so the 3D view is not available; the 2D cross-section is shown.';
    else if (failed3d) note3d = `The 3D view could not start (${failed3d}); the 2D cross-section is shown.`;
  }

  return (
    <figure className={`figure particle${stale ? ' is-stale' : ''}`} aria-labelledby={captionId} data-testid="particle-figure">
      <div className="particle__view" data-testid="particle" data-view={show3d ? '3d' : '2d'}>
        <div className="figure__head">
          <Badge label={run ? run.label : 'ILLUSTRATIVE'} />
          <span className="figure__kind" data-testid="particle-kind">
            {run ? (
              <>
                Concentration against radius <RowTime />
              </>
            ) : (
              'Particle cross-section to the input radii; no concentration data'
            )}
            {stale ? <span className="sim-note"> updating for the changed inputs</span> : null}
          </span>
          <div className="segmented" role="radiogroup" aria-label="Particle view" data-testid="view-switch">
            {(['2d', '3d'] as const).map((v) => (
              <button
                key={v}
                type="button"
                role="radio"
                aria-checked={view === v}
                className="segmented__item"
                data-testid={`view-${v}`}
                onClick={() => {
                  // WebGL 2 is tested only when the reader asks for the 3D view (no context is made on page load).
                  if (v === '3d' && webgl === null) setWebgl(hasWebGL2());
                  setView({ particleView: v });
                }}
              >
                {v === '2d' ? '2D section' : '3D cutaway'}
              </button>
            ))}
          </div>
        </div>
        {note3d ? (
          <p className="particle__note" role="status" data-testid="view-note">
            {note3d}
          </p>
        ) : null}
        <div className="particle__grid">
          <div className="particle__drawing">
            {show3d && run ? (
              <ThreeBoundary onError={setFailed3d}>
                <Suspense fallback={<p className="particle__loading">Loading the 3D view…</p>}>
                  <Particle3D run={run} label={label2d.replace('Cross-section', '3D cutaway')} onError={setFailed3d} />
                </Suspense>
              </ThreeBoundary>
            ) : rings ? (
              <CrossSection2D
                rings={rings}
                run={run}
                selected={selected}
                hovered={hovered}
                onHover={setHovered}
                onSelect={(i) => selectLayer(selected === i ? null : i)}
                label={label2d}
              />
            ) : (
              <p className="particle__none" role="status">
                The radii do not increase from the centre outwards, so the cross-section is not drawn. Correct the outer radii in the controls.
              </p>
            )}
          </div>
          <div className="particle__side">
            {run ? <Legend run={run} /> : null}
            <div className="inspector">
              <p className="inspector__title" id={listId}>
                Inspect a layer
              </p>
              <ul className="inspector__list" aria-labelledby={listId}>
                {names.map((name, i) => (
                  <li key={i}>
                    <button
                      type="button"
                      className="inspector__item"
                      aria-pressed={selected === i}
                      onClick={() => selectLayer(selected === i ? null : i)}
                      onFocus={() => setHovered(null)}
                      data-testid={`inspect-${i}`}
                      style={{ ['--ring' as string]: layerColour(i) }}
                    >
                      <span className="inspector__swatch" aria-hidden="true" />
                      {name}
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      </div>
      {run ? <ColourOptions /> : null}
      {detail !== null ? <LayerDetail index={detail} run={run} config={config} /> : null}
      <figcaption id={captionId} className="figure__caption">
        <strong>Figure 1.</strong>{' '}
        {run
          ? `Concentration at every node of the solver grid at the playback time, on a fixed ${logColour ? `logarithmic colour scale over ${LOG_DECADES} decades` : 'linear colour scale'}.`
          : 'The layers at the outer radii of the active parameters, drawn to scale; no concentration data.'}
      </figcaption>
    </figure>
  );
}
