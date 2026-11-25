import { useId, useMemo, type KeyboardEvent, type ReactNode } from 'react';
import { BUILD } from '../build';
import { findCase } from '../data/case';
import { hrefFor } from '../route';
import { Badge, Sci } from '../panels/Badge';
import type { RunOk } from '../solver/types';
import { inputKey, useStrato, type SimPlot } from '../state/store';
import { profileFigure, releaseFigure } from './figures';
import { Plot } from './Plot';
import { usePlotTheme } from './theme';

// The plots of block B (web phase 5; prompt/DESIGN_SPECIFICATION.md Sections 4.3 and 4.4), directly
// under the equation beside the particle: the radial concentration profile at the playback time and
// the release fraction of Code B and Code C with a playback cursor. Every plotted value is an array
// of the wrapper's RunResponse as returned (owner decision M4). On a phone one plot is shown at a time,
// with a switcher.

function useShownRun(): { run: RunOk | null; stale: boolean } {
  const run = useStrato((s) => s.run);
  const info = useStrato((s) => s.runInfo);
  const config = useStrato((s) => s.config);
  const status = useStrato((s) => s.status);
  return { run, stale: run !== null && (status === 'running' || info?.inputKey !== inputKey(config)) };
}

/** Source line of an exported image: model, version, solver and page commits, label, case. */
export function runSource(run: RunOk): string {
  return `STRATO v${run.engine.strato_version}, Rust solver in WebAssembly (build ${run.engine.commit}); ${run.label}; synthetic five-layer verification case, not physical data; page build ${BUILD.commit}.`;
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
      <p className="plotfig__empty">Appears when the Rust solver has run the current inputs.</p>
      <figcaption className="figure__caption">
        <strong>Figure {number}.</strong> {title}: drawn from the solver's reply; no plot is shown without one.
      </figcaption>
    </figure>
  );
}

function FigureSource({ run }: { run: RunOk }) {
  const caseId = useStrato((s) => s.caseId);
  const entry = findCase(caseId);
  return (
    <span className="figure__source">
      Source: STRATO v{run.engine.strato_version} Rust solver in WebAssembly, build <code>{run.engine.commit}</code>, run on the active
      parameters starting from <code>{entry?.provenance.input_file ?? 'the case file'}</code> (synthetic verification case, not physical data).
      Page build <code>{BUILD.commit}</code>. Values as returned by the solver; the CSV and JSON of this run are under{' '}
      <a href={hrefFor('data')}>Data</a>.
    </span>
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
  if (!run || !figure) return <EmptyFigure title="Radial concentration profile c(r)" testId="profile-figure" number={2} />;
  const row = run.profiles[tIndex] ?? run.profiles[run.profiles.length - 1];
  return (
    <figure className={`figure plotfig${stale ? ' is-stale' : ''}`} data-testid="profile-figure" data-row={tIndex}>
      <div className="figure__head">
        <Badge label={run.label} />
        <span className="figure__kind">
          <i>c</i>(<i>r</i>) at{' '}
          <span className="figure__time" data-testid="profile-time" data-value={row ? String(row.t_s) : ''}>
            <i>t</i> = {row ? <Sci value={row.t_s} unit="s" digits={4} /> : '–'}
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
        label={`Radial concentration profile at the playback time: the concentration the solver returned in each of its ${run.mesh.cell_centroids_m.length} cells against the radius, ${log ? 'on a logarithmic axis over six decades up to the kernel start concentration' : 'on a linear axis from zero to the kernel start concentration'}, with the five layers as coloured bands.`}
        exportInfo={{ name: `strato-profile-${run.engine.commit}`, source: runSource(run) }}
      />
      <figcaption className="figure__caption">
        <strong>Figure 2.</strong> Concentration in each cell of the solver mesh at the playback time, plotted at the cell centroids and joined
        by straight lines, against the radius; the layers are the coloured bands, in the colours of the particle.{' '}
        {log ? 'Logarithmic axis over six decades up to the kernel start concentration c₀ (an input), the scale of the particle colouring; values below it lie off the axis.' : 'Linear axis from zero to the kernel start concentration c₀ (an input).'}
        {showStart ? ' Dotted: the start profile at t = 0.' : null} The axis stays fixed during playback, so the frames compare.
        <FigureSource run={run} />
      </figcaption>
    </figure>
  );
}

export function ReleaseCurves() {
  const { run, stale } = useShownRun();
  const tIndex = useStrato((s) => s.playback.tIndex);
  const playing = useStrato((s) => s.playback.playing);
  const log = useStrato((s) => s.view.releaseLog);
  const setView = useStrato((s) => s.setView);
  const theme = usePlotTheme();
  const figure = useMemo(() => (run ? releaseFigure(run, tIndex, { log }, theme) : null), [run, tIndex, log, theme]);
  if (!run || !figure) return <EmptyFigure title="Release fraction F(t)" testId="release-figure" number={3} />;
  const t = run.times_s[tIndex];
  const f = run.release_fraction[tIndex];
  const a = run.analytical;
  return (
    <figure className={`figure plotfig${stale ? ' is-stale' : ''}`} data-testid="release-figure" data-row={tIndex}>
      <div className="figure__head">
        <Badge label={run.label} />
        <span className="figure__kind">
          <i>F</i>
          <sub>B</sub>(
          <span className="figure__time" data-testid="release-cursor-time" data-value={t === undefined ? '' : String(t)}>
            {t === undefined ? '–' : <Sci value={t} unit="s" digits={4} />}
          </span>
          ) ={' '}
          <span className="figure__time" data-testid="release-cursor-value" data-value={f === undefined ? '' : String(f)} data-label={run.label}>
            {f === undefined ? '–' : f.toPrecision(6)}
          </span>
          <span className="muted"> at the playback time</span>
          {stale ? <span className="sim-note"> updating for the changed inputs</span> : null}
        </span>
        <div className="plot-options" role="group" aria-label="Release plot display options">
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
        label={`Release fraction against time: the finite-volume solution (Code B, solid) and the eigenfunction series (Code C, dashed) as the solver returned them, ${log ? 'on a logarithmic axis' : 'on a linear axis'}, with the playback time marked.`}
        exportInfo={{ name: `strato-release-${run.engine.commit}`, source: runSource(run) }}
      />
      <figcaption className="figure__caption">
        <strong>Figure 3.</strong> Fraction of the start inventory released through the outer surface, from the finite-volume solution
        (Code B, solid) and the exact eigenfunction series (Code C, dashed), at the output times the solver recorded; the dotted line and the
        ring mark the playback time.{' '}
        {a ? (
          <>
            The series is valid from <i>t</i> = <Sci value={a.valid_from_s} unit="s" digits={4} /> (shaded before) and accurate to{' '}
            <Sci value={a.accuracy_floor} digits={4} />
            {log ? ' (the dotted horizontal line)' : ''}; it uses {a.modes} eigenmodes.{' '}
          </>
        ) : (
          'The series was not computed for these inputs; the reason is listed under the numerical diagnostics. '
        )}
        The pointwise difference of the two curves is not drawn here: it is a difference of two nearly equal numbers, sensitive to their
        last digits, which differ between this browser build and the native build of the solver. The largest and the final difference that
        the solver returns are in the numerical diagnostics; the pointwise difference of the verified case, from the native program, is
        plotted under <a href={hrefFor('verify')}>Verify</a>.
        <FigureSource run={run} />
      </figcaption>
    </figure>
  );
}

const PLOTS: { id: SimPlot; label: string }[] = [
  { id: 'profile', label: 'Profile c(r)' },
  { id: 'release', label: 'Release F(t)' },
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
