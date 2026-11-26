import { useId, useMemo, type KeyboardEvent, type ReactNode } from 'react';
import { BUILD } from '../build';
import { METHOD_1, METHOD_2 } from '../methods';
import { Badge, Sci } from '../panels/Badge';
import type { RunOk } from '../solver/types';
import { useShownRun } from '../solver/shown';
import { useStrato, type SimPlot } from '../state/store';
import { profileFigure, releaseFigure } from './figures';
import { Plot } from './Plot';
import { usePlotTheme } from './theme';

// The plots of block B (web phase 5; prompt/DESIGN_SPECIFICATION.md Sections 4.3 and 4.4), directly
// under the equation beside the particle: the radial concentration profile at the playback time and
// the release fraction of Method A and Method B with a playback cursor. Every plotted value is an array
// of the wrapper's RunResponse as returned (owner decision M4). On a phone one plot is shown at a time,
// with a switcher.

/** Source line of an exported image: model, version, solver and page commits, label, case. */
export function runSource(run: RunOk): string {
  return `STRATO ${METHOD_1}, solver version ${run.solver.version}, Rust solver in WebAssembly (build ${run.engine.commit}); ${run.label}; synthetic five-layer verification case, not physical data; page build ${BUILD.commit}.`;
}

function Toggle({ checked, onChange, children, testId }: { checked: boolean; onChange: (v: boolean) => void; children: ReactNode; testId: string }) {
  return (
    <label className="plot-option">
      <input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} data-testid={testId} />
      <span>{children}</span>
    </label>
  );
}

function EmptyFigure({ title, testId, number }: { title: ReactNode; testId: string; number: number }) {
  return (
    <figure className="figure plotfig plotfig--empty" data-testid={testId} data-state="empty">
      <div className="figure__head">
        <span className="readout__nolabel">Label: none</span>
        <span className="figure__kind">{title}</span>
      </div>
      <p className="plotfig__empty">No result yet.</p>
      <figcaption className="figure__caption">
        <strong>Figure {number}.</strong> {title}.
      </figcaption>
    </figure>
  );
}

const PLOT_HEIGHT = 250;
/** While playback runs, each plot redraws at most this often, so the particle keeps its frame rate (display only). */
const PLAYING_REDRAW_MS = 100;

export function RadialProfile() {
  const { run, stale } = useShownRun();
  const tIndex = useStrato((s) => s.playback.tIndex);
  const playing = useStrato((s) => s.playback.playing);
  const log = useStrato((s) => s.view.profileLog);
  const showStart = useStrato((s) => s.view.profileStart);
  const setView = useStrato((s) => s.setView);
  const theme = usePlotTheme();
  const figure = useMemo(() => (run ? profileFigure(run, tIndex, { log, showStart }, theme) : null), [run, tIndex, log, showStart, theme]);
  if (!run || !figure) return <EmptyFigure title="Radial concentration profile" testId="profile-figure" number={2} />;
  const row = run.profiles[tIndex] ?? run.profiles[run.profiles.length - 1];
  return (
    <figure className={`figure plotfig${stale ? ' is-stale' : ''}`} data-testid="profile-figure" data-row={tIndex}>
      <div className="figure__head">
        <Badge label={run.label} />
        <span className="figure__kind">
          Concentration against radius at the time{' '}
          <span className="figure__time" data-testid="profile-time" data-value={row ? String(row.t_s) : ''}>
            {row ? <Sci value={row.t_s} unit="s" digits={4} /> : '–'}
          </span>
          {stale ? <span className="sim-note"> updating for the changed inputs</span> : null}
        </span>
        <div className="plot-options" role="group" aria-label="Profile display options">
          <Toggle checked={log} onChange={(v) => setView({ profileLog: v })} testId="profile-log">
            Log scale
          </Toggle>
          <Toggle checked={showStart} onChange={(v) => setView({ profileStart: v })} testId="profile-start">
            Start profile
          </Toggle>
        </div>
      </div>
      <Plot
        figure={figure}
        testId="profile-plot"
        height={PLOT_HEIGHT}
        minIntervalMs={playing ? PLAYING_REDRAW_MS : 0}
        label={`Radial concentration profile at the playback time: the concentration the solver returned at each of its ${run.grid.node_radii_m.length} grid nodes against the radius, ${log ? 'on a logarithmic axis over six decades up to the largest concentration of the run' : 'on a linear axis from zero to the largest concentration of the run'}, with the five layers as coloured bands.`}
        exportInfo={{ name: `strato-profile-${run.engine.commit}`, source: runSource(run) }}
      />
      <figcaption className="figure__caption">
        <strong>Figure 2.</strong> Concentration at each solver node against radius at the playback time{showStart ? '; the grey line is the start profile' : ''}.
      </figcaption>
    </figure>
  );
}

export function ReleaseCurves() {
  const { run, stale } = useShownRun();
  const tIndex = useStrato((s) => s.playback.tIndex);
  const playing = useStrato((s) => s.playback.playing);
  const log = useStrato((s) => s.view.releaseLog);
  const showMethod1 = useStrato((s) => s.view.showMethod1);
  const showMethod2 = useStrato((s) => s.view.showMethod2);
  const setView = useStrato((s) => s.setView);
  const theme = usePlotTheme();
  const figure = useMemo(
    () => (run ? releaseFigure(run, tIndex, { log, showMethod1, showMethod2 }, theme) : null),
    [run, tIndex, log, showMethod1, showMethod2, theme],
  );
  if (!run || !figure) return <EmptyFigure title="Release fraction against time" testId="release-figure" number={3} />;
  const t = run.times_s[tIndex];
  const f = run.release_fraction[tIndex];
  const a = run.analytical && run.analytical.applicable ? run.analytical : null;
  return (
    <figure className={`figure plotfig${stale ? ' is-stale' : ''}`} data-testid="release-figure" data-row={tIndex}>
      <div className="figure__head">
        <Badge label={run.label} />
        <span className="figure__kind">
          Release fraction (dimensionless) at the time{' '}
          <span className="figure__time" data-testid="release-cursor-time" data-value={t === undefined ? '' : String(t)}>
            {t === undefined ? '–' : <Sci value={t} unit="s" digits={4} />}
          </span>
          :{' '}
          <span className="figure__time" data-testid="release-cursor-value" data-value={f === undefined ? '' : String(f)} data-label={run.label}>
            {f === undefined ? '–' : f.toPrecision(6)}
          </span>
          <span className="muted"> ({METHOD_1}, at the playback time)</span>
          {stale ? <span className="sim-note"> updating for the changed inputs</span> : null}
        </span>
        <div className="plot-options" role="group" aria-label="Release plot display options">
          <Toggle checked={showMethod1} onChange={(v) => setView({ showMethod1: v })} testId="release-method-1">
            {METHOD_1}
          </Toggle>
          <Toggle checked={showMethod2} onChange={(v) => setView({ showMethod2: v })} testId="release-method-2">
            {METHOD_2}
          </Toggle>
          <Toggle checked={log} onChange={(v) => setView({ releaseLog: v })} testId="release-log">
            Log scale
          </Toggle>
        </div>
      </div>
      <Plot
        figure={figure}
        testId="release-plot"
        height={PLOT_HEIGHT}
        minIntervalMs={playing ? PLAYING_REDRAW_MS : 0}
        label={`Release fraction against time: ${METHOD_1} (blue line) and ${METHOD_2} (red line) as the solver returned them, ${log ? 'on a logarithmic axis' : 'on a linear axis'}, with the playback time marked.`}
        exportInfo={{ name: `strato-release-${run.engine.commit}`, source: runSource(run) }}
      />
      <figcaption className="figure__caption">
        <strong>Figure 3.</strong> Fraction of the initial inventory released through the outer surface; the vertical line marks the playback time.{a ? null : ` ${METHOD_2} was not computed for these inputs.`}
      </figcaption>
    </figure>
  );
}
const PLOTS: { id: SimPlot; label: string }[] = [
  { id: 'profile', label: 'Concentration profile' },
  { id: 'release', label: 'Release fraction' },
];

/** Block B plots: both on wider screens; on a phone one at a time with a switcher. */
export function SimPlots({ compact }: { compact: boolean }) {
  const shown = useStrato((s) => s.view.simPlot);
  const setView = useStrato((s) => s.setView);
  const id = useId();
  if (!compact) {
    return (
      <div className="sim__plots" data-testid="sim-plots">
        <RadialProfile />
        <ReleaseCurves />
      </div>
    );
  }
  const onKey = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight') return;
    e.preventDefault();
    const next: SimPlot = shown === 'profile' ? 'release' : 'profile';
    setView({ simPlot: next });
    document.getElementById(`${id}-tab-${next}`)?.focus();
  };
  return (
    <div className="sim__plots sim__plots--switch" data-testid="sim-plots">
      <div className="segmented plot-switch" role="tablist" aria-label="Plot shown" onKeyDown={onKey} data-testid="plot-switch">
        {PLOTS.map((p) => (
          <button
            key={p.id}
            id={`${id}-tab-${p.id}`}
            type="button"
            role="tab"
            className="segmented__item"
            aria-selected={shown === p.id}
            aria-controls={`${id}-panel`}
            tabIndex={shown === p.id ? 0 : -1}
            onClick={() => setView({ simPlot: p.id })}
            data-testid={`plot-tab-${p.id}`}
          >
            {p.label}
          </button>
        ))}
      </div>
      <div id={`${id}-panel`} role="tabpanel" aria-labelledby={`${id}-tab-${shown}`}>
        {shown === 'profile' ? <RadialProfile /> : <ReleaseCurves />}
      </div>
    </div>
  );
}
