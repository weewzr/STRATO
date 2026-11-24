// Browser self-test (pattern of NUS-H2 frontend/selftest.js at b4817b3; prompt/TESTING_AND_VALIDATION.md Section 4).
// Open the built site with ?selftest=1. The verdict is written as JSON to #selftest-result and to
// document.body.dataset.selftest ('pass' or 'fail'). The self-test compares values only; it computes no physics.
import { BUILD } from '../build';
import { DEFAULT_CASE } from '../data/case';
import { SECTIONS } from '../route';
import { useStrato } from '../state/store';

type Status = 'pass' | 'fail' | 'not_run';
interface Check {
  id: string;
  status: Status;
  detail: string;
}

const captured: string[] = [];

/** Call before the app renders, so every console error and uncaught error is counted. */
export function installCapture(): void {
  const original = console.error.bind(console);
  console.error = (...args: unknown[]) => {
    captured.push(args.map(String).join(' '));
    original(...args);
  };
  window.addEventListener('error', (e) => captured.push(`error: ${e.message}`));
  window.addEventListener('unhandledrejection', (e) => captured.push(`unhandledrejection: ${String(e.reason)}`));
}

const nextFrame = () => new Promise<void>((r) => requestAnimationFrame(() => setTimeout(r, 30)));

async function waitFor(pred: () => boolean, ms = 3000): Promise<boolean> {
  const start = performance.now();
  while (performance.now() - start < ms) {
    if (pred()) return true;
    await nextFrame();
  }
  return pred();
}

export async function runSelftest(): Promise<void> {
  const checks: Check[] = [];
  const add = (id: string, ok: boolean, detail: string) => checks.push({ id, status: ok ? 'pass' : 'fail', detail });
  const params = new URLSearchParams(window.location.search);

  // 1. Build stamp: meta tag, bundle constant and build.json agree.
  const meta = document.querySelector<HTMLMetaElement>('meta[name="strato-build"]')?.content ?? '';
  let fileCommit: string;
  try {
    const res = await fetch(`${import.meta.env.BASE_URL}build.json?v=${encodeURIComponent(BUILD.commit)}`, { cache: 'no-store' });
    fileCommit = res.ok ? String(((await res.json()) as { commit?: string }).commit ?? '') : `HTTP ${res.status}`;
  } catch (e) {
    fileCommit = `fetch failed: ${String(e)}`;
  }
  add('build_stamp', meta === BUILD.commit && fileCommit === BUILD.commit, `meta=${meta} bundle=${BUILD.commit} build.json=${fileCommit}`);
  const expected = params.get('expect');
  if (expected) {
    add('expected_commit', expected.startsWith(BUILD.commit) || BUILD.commit.startsWith(expected), `expected=${expected} bundle=${BUILD.commit}`);
  }

  // 2. Inputs: the store holds the generated case values (value comparison only).
  const state = useStrato.getState();
  const sameConfig = JSON.stringify(state.baselineConfig) === JSON.stringify(DEFAULT_CASE.config);
  const hashOk = /^[0-9a-f]{64}$/.test(DEFAULT_CASE.provenance.input_sha256);
  add('case_inputs', sameConfig && hashOk, `baseline equals generated case: ${sameConfig}; input SHA-256 present: ${hashOk}`);

  // 3. Planned mechanisms stay unavailable.
  const mechs = Object.values(state.mechanisms);
  add('mechanisms_disabled', mechs.every((m) => !m.available && !m.enabled), 'decay, source and arrhenius unavailable before web phase 6');

  // 4. Every section renders its heading.
  const startHash = window.location.hash;
  const missing: string[] = [];
  for (const s of SECTIONS) {
    window.location.hash = `#/${s.id}`;
    const ok = await waitFor(() => document.querySelector(`[data-section="${s.id}"] h1`) !== null);
    if (!ok) missing.push(s.id);
  }
  window.location.hash = '#/explore';
  await waitFor(() => document.querySelector('[data-section="explore"] h1') !== null);
  add('sections_render', missing.length === 0, missing.length === 0 ? `${SECTIONS.length} sections` : `missing: ${missing.join(', ')}`);

  // 4b. Explore shows blocks A to E in this order (owner decision M7): document order and top edges.
  const blocks = Array.from(document.querySelectorAll<HTMLElement>('[data-section="explore"] [data-block]'));
  const letters = blocks.map((b) => b.dataset.block ?? '').join('');
  const tops = blocks.map((b) => b.getBoundingClientRect().top);
  const increasing = tops.every((t, i) => i === 0 || t > (tops[i - 1] ?? Number.NEGATIVE_INFINITY));
  add('explore_block_order', letters === 'ABCDE' && increasing, `order=${letters} tops_increasing=${increasing}`);
  window.location.hash = startHash || '#/explore';
  await nextFrame();

  // 5. Reduced motion, when the browser asks for it.
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
    const btn = document.querySelector('.btn');
    const dur = btn ? getComputedStyle(btn).transitionDuration : '';
    add('reduced_motion', btn !== null && dur.split(',').every((d) => parseFloat(d) === 0), `button transition-duration=${dur}`);
  }

  // 6. Solver checks arrive with the wasm wrapper.
  checks.push({ id: 'wasm_loads', status: 'not_run', detail: 'no WebAssembly module before web phase 3' });
  checks.push({ id: 'golden_parity', status: 'not_run', detail: 'no solver before web phase 3' });

  // 7. Failed requests (resource timing) and console errors.
  const failed = performance
    .getEntriesByType('resource')
    .filter((e) => {
      const status = (e as PerformanceResourceTiming & { responseStatus?: number }).responseStatus;
      return typeof status === 'number' && status >= 400;
    })
    .map((e) => e.name);
  add('no_failed_requests', failed.length === 0, failed.length === 0 ? 'none' : failed.join(' '));
  add('no_console_errors', captured.length === 0, captured.length === 0 ? 'none' : captured.join(' | '));

  const verdict = {
    selftest: 1,
    commit: BUILD.commit,
    strato_version: BUILD.strato_version,
    reduced_motion: window.matchMedia('(prefers-reduced-motion: reduce)').matches,
    result: checks.some((c) => c.status === 'fail') ? 'fail' : 'pass',
    checks,
  };
  let out = document.getElementById('selftest-result');
  if (!out) {
    out = document.createElement('pre');
    out.id = 'selftest-result';
    out.hidden = true;
    document.body.appendChild(out);
  }
  out.textContent = JSON.stringify(verdict, null, 2);
  document.body.dataset.selftest = verdict.result;
}
