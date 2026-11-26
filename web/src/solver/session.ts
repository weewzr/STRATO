// Connects the solver client to the store: records build_info().btcs.capabilities, validates the
// current inputs with the wrapper and runs them once the worker is ready and again after every
// change (debounced), and stores the wrapper's reply as it is; it also runs compare when the
// Compare section asks for it. Every request goes to a BTCS function of the wrapper; a failure to
// load the solver is stored as a failure, and no other source of numbers exists. Nothing here
// computes a value.
import { comparisonKey, inputKey, useStrato } from '../state/store';
import { SolverClient, SolverError, type WorkerFactory } from './client';
import { RUN_DEBOUNCE_MS } from './tolerances';

let client: SolverClient | null = null;
let timer: ReturnType<typeof setTimeout> | null = null;

/** The mechanism ids a run asks for: the layered diffusion of the BTCS solver (builder steps 1 to 12). */
const RUN_MECHANISMS = ['diffusion_btcs_layered'];

export function solverClient(): SolverClient | null {
  return client;
}

function stampRequests(): void {
  if (typeof document !== 'undefined' && client) document.documentElement.dataset.solverRequests = String(client.sent);
}

/** Asks the wrapper whether the current inputs are valid; its own messages are stored and shown. */
async function validateNow(): Promise<void> {
  if (!client) return;
  const s = useStrato.getState();
  const key = inputKey(s.config);
  try {
    const reply = await client.validate(s.config);
    const body = reply.body;
    useStrato.getState().setValidation({
      key,
      state: body.ok ? 'ok' : 'rejected',
      errors: body.errors,
      warnings: body.warnings,
      plan: body.plan,
      grid: body.grid,
      limits: body.limits,
    });
  } catch (e) {
    if (e instanceof SolverError && (e.reason === 'closed' || e.reason === 'stale')) return;
    useStrato.getState().setValidation({
      key,
      state: 'error',
      errors: [e instanceof Error ? e.message : String(e)],
      warnings: [],
      plan: null,
      grid: null,
      limits: null,
    });
  }
}

async function runNow(): Promise<void> {
  if (!client) return;
  const s = useStrato.getState();
  const config = s.config;
  const key = inputKey(config);
  s.setRunning();
  void validateNow();
  try {
    const pending = client.run(config, RUN_MECHANISMS, s.capabilities);
    stampRequests();
    const reply = await pending;
    const info = { elapsedMs: reply.elapsedMs, requestId: reply.requestId, inputKey: key, dropped: reply.dropped };
    if (reply.body.status === 'ok') useStrato.getState().setRunOutcome({ run: reply.body, info });
    else useStrato.getState().setRunOutcome({ problem: { kind: 'rejected', messages: reply.body.errors }, info });
  } catch (e) {
    stampRequests();
    if (e instanceof SolverError && e.reason === 'stale') return;
    const text = e instanceof Error ? e.message : String(e);
    useStrato.getState().setRunOutcome({ problem: { kind: 'error', messages: [text] }, info: null });
  }
}

function schedule(delay: number): void {
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => {
    timer = null;
    void runNow();
  }, delay);
}

let compareTimer: ReturnType<typeof setTimeout> | null = null;

/**
 * Compares the current inputs with the baseline through compare. Asked for only by the Compare
 * section, so the Explore page sends no comparison request. The reply is stored as it is.
 */
async function compareNow(): Promise<void> {
  if (!client) return;
  const s = useStrato.getState();
  if (s.solver.state !== 'ready') return;
  const baseline = s.baselineConfig;
  const current = s.config;
  const key = comparisonKey(baseline, current);
  s.setComparison({ status: 'running' });
  try {
    const pending = client.compare(baseline, current, RUN_MECHANISMS, s.capabilities);
    stampRequests();
    const reply = await pending;
    const set = useStrato.getState().setComparison;
    if (reply.body.status === 'ok') set({ status: 'ok', reply: reply.body, problem: null, key, elapsedMs: reply.elapsedMs });
    else set({ status: 'rejected', reply: null, problem: reply.body.errors, key, elapsedMs: null });
  } catch (e) {
    stampRequests();
    if (e instanceof SolverError && e.reason === 'stale') return;
    const text = e instanceof Error ? e.message : String(e);
    useStrato.getState().setComparison({ status: 'error', reply: null, problem: [text], key, elapsedMs: null });
  }
}

function scheduleCompare(delay: number): void {
  if (compareTimer) clearTimeout(compareTimer);
  compareTimer = setTimeout(() => {
    compareTimer = null;
    void compareNow();
  }, delay);
}

const defaultFactory: WorkerFactory = () => new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });

/** Starts the worker and keeps the run in step with the inputs. Called once, from main.tsx. */
export function startSolver(factory: WorkerFactory = defaultFactory): SolverClient {
  if (client) return client;
  // After a trap or a time-out the replacement worker does not rerun the inputs that caused it;
  // the reader starts the next run with Run.
  let runOnHello = true;
  client = new SolverClient(factory, {
    onHello: (build) => {
      const s = useStrato.getState();
      s.setSolver({ state: 'ready', build, error: null });
      // The BTCS capabilities only: there is no other list to read.
      s.setCapabilities(build.btcs.capabilities);
      if (runOnHello) schedule(0);
    },
    onLoadFailed: (error) => useStrato.getState().setSolver({ state: 'failed', error }),
    onRestart: () => {
      runOnHello = false;
      const s = useStrato.getState();
      s.setSolver({ state: 'loading', restarts: s.solver.restarts + 1 });
    },
  });
  stampRequests();
  useStrato.setState({ requestRun: () => schedule(0), requestCompare: () => scheduleCompare(RUN_DEBOUNCE_MS) });
  useStrato.subscribe((state, prev) => {
    if (state.config !== prev.config && state.solver.state === 'ready') schedule(RUN_DEBOUNCE_MS);
  });
  return client;
}