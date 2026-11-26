// Browser self-test (pattern of NUS-H2 frontend/selftest.js at b4817b3; prompt/TESTING_AND_VALIDATION.md Section 4).
// Open the built site with ?selftest=1. The verdict is written as JSON to #selftest-result and to
// document.body.dataset.selftest ('pass' or 'fail'). The self-test compares values only; it computes no physics.
import { BUILD } from '../build';
import { DEFAULT_CASE } from '../data/case';
import { isAvailable } from '../equation/availability';
import { buildCapability, MECHANISMS, WRAPPER_ID } from '../equation/capability';
import { SWITCH_MECHANISM, type SwitchId } from '../equation/terms';
import { CATALOGUE, statusOf } from '../research/catalogue';
import { entryPage } from '../research/entries';
import { hrefResearch, SECTIONS } from '../route';
import { compareArrays } from '../solver/compare';
import { solverClient } from '../solver/session';
import { PARITY_RTOL } from '../solver/tolerances';
import type { RunReply } from '../solver/types';
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

async function fetchJson(name: string): Promise<{ data: Record<string, unknown> } | null> {
  try {
    const res = await fetch(`${import.meta.env.BASE_URL}data/${name}?v=${encodeURIComponent(BUILD.commit)}`, { cache: 'no-store' });
    return res.ok ? ((await res.json()) as { data: Record<string, unknown> }) : null;
  } catch {
    return null;
  }
}

type Col = (number | null)[];

async function solverChecks(add: (id: string, ok: boolean, detail: string) => void): Promise<void> {
  const ready = await waitFor(() => useStrato.getState().solver.state !== 'loading', 20000);
  const s = useStrato.getState();
  const build = s.solver.build;
  add(
    'wasm_loads',
    ready && s.solver.state === 'ready' && build !== null && build.commit === BUILD.commit,
    build ? `abi=${build.abi} strato=${build.strato_version} wasm commit=${build.commit} page commit=${BUILD.commit} capabilities=[${build.btcs.capabilities.join(', ')}]` : `state=${s.solver.state} ${s.solver.error ?? ''}`,
  );
  const client = solverClient();
  if (!client || s.solver.state !== 'ready') {
    add('native_parity', false, 'no solver');
    return;
  }
  let run: RunReply;
  try {
    // A request of its own (not run()), so that the page's current run is not overtaken.
    run = (await client.request<RunReply>('run', { schema: 2, config: DEFAULT_CASE.config, mechanisms: ['diffusion_btcs_layered'] })).body;
  } catch (e) {
    add('native_parity', false, `run failed: ${String(e)}`);
    return;
  }
  if (run.status !== 'ok') {
    add('native_parity', false, `rejected: ${run.errors.join('; ')}`);
    return;
  }

  const native = await fetchJson('wrapper_native_synthetic.json');
  const n = native?.data as unknown as RunReply | undefined;
  if (!n || n.status !== 'ok') {
    add('native_parity', false, 'wrapper_native_synthetic.json not loaded');
    return;
  }
  const pairs: [string, Col, Col][] = [
    ['times_s', run.times_s, n.times_s],
    ['release_fraction', run.release_fraction, n.release_fraction],
    ['inventory_mol', run.inventory_mol, n.inventory_mol],
    ['balance_residual_mol', run.balance_residual_mol, n.balance_residual_mol],
    ['final_profile', run.profiles[run.profiles.length - 1]?.c_mol_per_m3 ?? [], n.profiles[n.profiles.length - 1]?.c_mol_per_m3 ?? []],
    ['node_radii_m', run.grid.node_radii_m, n.grid.node_radii_m],
    ['release_rate_mol_s', run.release_rate_mol_s, n.release_rate_mol_s],
    ['analytical', run.analytical?.release_fraction ?? [], n.analytical?.release_fraction ?? []],
  ];
  const results = pairs.map(([name, a, b]) => compareArrays(name, a, b, PARITY_RTOL));
  const bad = results.filter((r) => !r.ok);
  add('native_parity', bad.length === 0, bad.length === 0 ? `${results.length} arrays equal per value within ${PARITY_RTOL}` : bad.map((r) => `${r.name} ${JSON.stringify(r.first)} failures ${r.failures}`).join('; '));
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

  // 3. The status of each mechanism follows the capabilities the wrapper reported (equation/capability.ts), and
  //    a mechanism that is not verified in this build is not simulable.
  const cap = buildCapability(state.capabilities);
  const mechEntries = MECHANISMS.map((m) => [m, cap.mechanisms[m]] as const);
  const mechOk = mechEntries.every(([m, s]) => (s === 'verified') === isAvailable(WRAPPER_ID[m], state.capabilities) && (s !== 'planned') === state.capabilities.includes(WRAPPER_ID[m]));
  add('mechanisms_status', mechOk, `capabilities=[${state.capabilities.join(', ')}]; ${mechEntries.map(([m, s]) => `${m}:${s}`).join(' ')}`);

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

  // 4c. Every research status shown equals the generated catalogue (value comparison only).
  //     A chip of one catalogue row (data-status-row) is compared with that row.
  const chips = Array.from(document.querySelectorAll<HTMLElement>('[data-status-for]'));
  const wrong = chips.filter((c) => {
    const row = c.dataset.statusRow ? CATALOGUE.mechanisms.find((m) => `${m.mechanism_id}|${m.species}|${m.material_domain}` === c.dataset.statusRow) : undefined;
    return c.dataset.status !== (c.dataset.statusRow ? row?.status : statusOf(CATALOGUE, c.dataset.statusFor ?? ''));
  });
  add(
    'research_statuses',
    chips.length > 0 && wrong.length === 0,
    wrong.length === 0 ? `${chips.length} status chips equal the catalogue` : `differ: ${wrong.map((c) => c.dataset.statusFor).join(', ')}`,
  );

  // 4d. No mechanism switch is active outside isAvailable() (capabilities and catalogue status).
  const caps = useStrato.getState().capabilities;
  const switches = Array.from(document.querySelectorAll<HTMLElement>('[role="switch"]'));
  const active = switches.filter((s) => {
    const id = (s.dataset.testid ?? '').replace(/^mech-/, '') as SwitchId;
    const allowed = id in SWITCH_MECHANISM && isAvailable(SWITCH_MECHANISM[id], caps);
    return s.getAttribute('aria-disabled') !== 'true' && !allowed;
  });
  add(
    'no_active_switch',
    active.length === 0,
    active.length === 0 ? `${switches.length} switches, none active outside the capabilities [${caps.join(', ')}]` : `active: ${active.map((s) => s.dataset.testid).join(', ')}`,
  );

  // 4e. KaTeX typeset the governing equation without an error.
  const katexCount = document.querySelectorAll('[data-section="explore"] .katex').length;
  const katexErrors = document.querySelectorAll('.katex-error').length;
  add('katex_renders', katexCount > 0 && katexErrors === 0, `katex elements=${katexCount} errors=${katexErrors}`);

  // 4f. Research catalogue (web phase 5.5): every page renders, its row statuses equal the catalogue,
  //     and no switch on it is active.
  const pages = ['', 'gallery', 'roadmap', 'references', ...Object.keys(CATALOGUE.entries).map(entryPage).filter((p) => p !== 'gallery' && p !== 'roadmap')];
  const pageProblems: string[] = [];
  let pageChips = 0;
  for (const p of pages) {
    window.location.hash = hrefResearch(p);
    const ok = await waitFor(() => document.querySelector(`[data-section="research"][data-research-page="${p || 'overview'}"] h1`) !== null);
    if (!ok) {
      pageProblems.push(`${p || 'overview'}: no heading`);
      continue;
    }
    for (const c of Array.from(document.querySelectorAll<HTMLElement>('[data-section="research"] [data-status-row]'))) {
      pageChips += 1;
      const row = CATALOGUE.mechanisms.find((m) => `${m.mechanism_id}|${m.species}|${m.material_domain}` === c.dataset.statusRow);
      if (!row || row.status !== c.dataset.status) pageProblems.push(`${p}: ${c.dataset.statusRow ?? ''}`);
    }
    for (const s of Array.from(document.querySelectorAll<HTMLElement>('[data-section="research"] [role="switch"]'))) {
      if (s.getAttribute('aria-disabled') !== 'true') pageProblems.push(`${p}: active switch ${s.dataset.testid ?? ''}`);
    }
    if (document.querySelector('.katex-error')) pageProblems.push(`${p}: KaTeX error`);
  }
  add(
    'research_pages',
    pageProblems.length === 0 && pageChips > 0,
    pageProblems.length === 0 ? `${pages.length} pages, ${pageChips} row chips equal the catalogue, no active switch` : pageProblems.join('; '),
  );
  window.location.hash = startHash || '#/explore';
  await nextFrame();

  // 5. Reduced motion, when the browser asks for it.
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
    const btn = document.querySelector('.btn');
    const dur = btn ? getComputedStyle(btn).transitionDuration : '';
    add('reduced_motion', btn !== null && dur.split(',').every((d) => parseFloat(d) === 0), `button transition-duration=${dur}`);
  }

  // 6. Solver: the module loads, reports its build and capabilities, and a run of the default
  //    case equals the native wrapper (parity rule).
  await solverChecks(add);

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
