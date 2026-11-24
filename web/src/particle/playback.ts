// Playback clock (prompt/TECHNICAL_ARCHITECTURE.md Section 4: kept outside React re-render paths).
// It walks through the output rows the solver recorded, never between them: the row shown is the
// one the wrapper returned, so no value is interpolated. At speed 1x the whole run plays in
// PLAYBACK_BASE_SECONDS of wall time; the row index follows from the elapsed wall time.
import { useStrato } from '../state/store';

export const PLAYBACK_BASE_SECONDS = 10;

let raf = 0;
let last = 0;
/** Fractional row position while playing. */
let position = 0;

function frame(now: number): void {
  const s = useStrato.getState();
  const rows = s.run?.profiles.length ?? 0;
  if (!s.playback.playing || rows < 2) {
    raf = 0;
    return;
  }
  const dt = last === 0 ? 0 : (now - last) / 1000;
  last = now;
  position += (dt * s.playback.speed * (rows - 1)) / PLAYBACK_BASE_SECONDS;
  const index = Math.min(rows - 1, Math.floor(position));
  if (index >= rows - 1) {
    s.setPlayback({ tIndex: rows - 1, playing: false });
    raf = 0;
    return;
  }
  if (index !== s.playback.tIndex) s.setPlayback({ tIndex: index });
  raf = requestAnimationFrame(frame);
}

function start(): void {
  const s = useStrato.getState();
  position = s.playback.tIndex;
  last = 0;
  if (!raf) raf = requestAnimationFrame(frame);
}

function stop(): void {
  if (raf) cancelAnimationFrame(raf);
  raf = 0;
}

/** Play from the shown row; at the last row, play starts again from the first. */
export function togglePlay(): void {
  const s = useStrato.getState();
  const rows = s.run?.profiles.length ?? 0;
  if (rows < 2) return;
  if (s.playback.playing) {
    s.setPlayback({ playing: false });
    return;
  }
  s.setPlayback({ tIndex: s.playback.tIndex >= rows - 1 ? 0 : s.playback.tIndex, playing: true });
}

let installed = false;

/** Starts and stops the animation loop with the store's `playing` flag. Called once, from main.tsx. */
export function installPlaybackClock(): void {
  if (installed || typeof window === 'undefined') return;
  installed = true;
  useStrato.subscribe((s, p) => {
    if (s.playback.playing && !p.playback.playing) start();
    else if (!s.playback.playing && p.playback.playing) stop();
    else if (s.playback.playing && s.playback.speed !== p.playback.speed) position = s.playback.tIndex;
  });
}
