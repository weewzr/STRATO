// Connects the solver client to the store (web phase 3): records build_info().capabilities,
// runs the current inputs once the worker is ready and again after every change (debounced),
// and stores the wrapper's reply as it is. Nothing here computes a value.
import { inputKey, useStrato } from '../state/store';
import { SolverClient, SolverError, type WorkerFactory } from './client';
import { RUN_DEBOUNCE_MS } from './tolerances';

let client: SolverClient | null = null;
let timer: ReturnType<typeof setTimeout> | null = null;

/** The mechanism ids a v1.0 run asks for: the layered diffusion of builder stage 1. */
const RUN_MECHANISMS = ['diffusion_layered'];

export function solverClient(): SolverClient | null {
  return client;
}

function stampRequests(): void {
  if (typeof document !== 'undefined' && client) document.documentElement.dataset.solverRequests = String(client.sent);
}

async function runNow(): Promise<void> {
  if (!client) return;
  const s = useStrato.getState();
  const config = s.config;
  const key = inputKey(config);
  s.setRunning();
  try {
    const pending = client.runV1(config, RUN_MECHANISMS, s.capabilities);
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
      s.setCapabilities(build.capabilities);
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
  useStrato.setState({ requestRun: () => schedule(0) });
  useStrato.subscribe((state, prev) => {
    if (state.config !== prev.config && state.solver.state === 'ready') schedule(RUN_DEBOUNCE_MS);
  });
  return client;
}
