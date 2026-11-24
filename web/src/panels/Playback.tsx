import { useId, type KeyboardEvent } from 'react';
import { readableDuration, sciText } from '../format';
import { togglePlay } from '../particle/playback';
import { PLAYBACK_SPEEDS, useStrato } from '../state/store';
import { Sci } from './Badge';

// Playback of the solver's output rows (prompt/DESIGN_SPECIFICATION.md Section 4.2): play, pause,
// step back, step forward, speed 0.25x to 8x, a time slider and the current time in seconds and in
// a readable unit. Space toggles play and the arrow keys step while focus is in the group. Nothing
// starts by itself, so reduced motion needs no special case. The time shown is the t_s the wrapper
// returned for the row.

function speedText(s: number): string {
  return `${s}×`;
}

export function Playback() {
  const run = useStrato((s) => s.run);
  const tIndex = useStrato((s) => s.playback.tIndex);
  const playing = useStrato((s) => s.playback.playing);
  const speed = useStrato((s) => s.playback.speed);
  const setPlayback = useStrato((s) => s.setPlayback);
  const step = useStrato((s) => s.stepPlayback);
  const ids = { label: useId(), time: useId() };
  const rows = run?.profiles.length ?? 0;
  const usable = rows > 1;
  const t = run?.profiles[tIndex]?.t_s;

  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    const target = e.target as HTMLElement;
    const tag = target.tagName;
    if (!usable) return;
    if (e.key === ' ' && tag !== 'BUTTON' && tag !== 'SELECT') {
      e.preventDefault();
      togglePlay();
    } else if ((e.key === 'ArrowLeft' || e.key === 'ArrowRight') && tag !== 'INPUT' && tag !== 'SELECT') {
      e.preventDefault();
      step(e.key === 'ArrowLeft' ? -1 : 1);
    }
  };

  return (
    <div
      className="playback"
      role="group"
      aria-labelledby={ids.label}
      aria-describedby={ids.time}
      data-testid="playback"
      data-row={usable ? tIndex : ''}
      data-playing={playing ? 'true' : 'false'}
      onKeyDown={onKeyDown}
    >
      <span id={ids.label} className="visually-hidden">
        Playback
      </span>
      <button
        type="button"
        className="btn btn--small btn--icon"
        aria-label="Step back one output time"
        aria-disabled={!usable || tIndex === 0}
        data-testid="pb-back"
        onClick={() => usable && step(-1)}
      >
        <span aria-hidden="true">◀︎|</span>
      </button>
      <button
        type="button"
        className="btn btn--small btn--icon playback__play"
        aria-label={playing ? 'Pause' : 'Play'}
        aria-disabled={!usable}
        data-testid="pb-play"
        onClick={() => usable && togglePlay()}
      >
        <span aria-hidden="true">{playing ? '❚❚' : '▶︎'}</span>
      </button>
      <button
        type="button"
        className="btn btn--small btn--icon"
        aria-label="Step forward one output time"
        aria-disabled={!usable || tIndex >= rows - 1}
        data-testid="pb-forward"
        onClick={() => usable && step(1)}
      >
        <span aria-hidden="true">|▶︎</span>
      </button>
      <label className="playback__speed">
        <span className="visually-hidden">Speed</span>
        <select
          value={String(speed)}
          onChange={(e) => setPlayback({ speed: Number(e.target.value) })}
          data-testid="pb-speed"
          aria-label="Playback speed (1× plays the whole run in 10 s)"
        >
          {PLAYBACK_SPEEDS.map((s) => (
            <option key={s} value={String(s)}>
              {speedText(s)}
            </option>
          ))}
        </select>
      </label>
      <input
        type="range"
        className="playback__slider"
        min={0}
        max={Math.max(0, rows - 1)}
        step={1}
        value={usable ? tIndex : 0}
        disabled={!usable}
        aria-label="Output time"
        aria-valuetext={t === undefined ? 'no run' : `t = ${sciText(t, 4)} s (${readableDuration(t)}), output ${tIndex + 1} of ${rows}`}
        data-testid="pb-slider"
        onChange={(e) => setPlayback({ tIndex: Number(e.target.value), playing: false })}
      />
      <span className="playback__time" id={ids.time} data-testid="pb-time" data-value={t === undefined ? '' : String(t)}>
        {t === undefined ? (
          'no run'
        ) : (
          <>
            <i>t</i> = <Sci value={t} unit="s" digits={4} /> <span className="muted">({readableDuration(t)})</span>
          </>
        )}
      </span>
    </div>
  );
}
