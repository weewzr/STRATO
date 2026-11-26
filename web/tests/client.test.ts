// Solver client (prompt/TESTING_AND_VALIDATION.md Section 3, row 3: worker handshake, time-out and
// recovery; Section 3.2: the client never sends a mechanism for which isAvailable() is false).
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { SolverClient, SolverError, type WorkerLike } from '../src/solver/client';
import type { WorkerMessage, WorkerRequest } from '../src/solver/protocol';
import type { BuildInfo } from '../src/solver/types';

const BUILD_INFO = {
  abi: 3,
  crate: 'strato-wasm',
  wrapper_version: '2.0.0',
  strato_version: '2.0.0',
  commit: 'abc1234',
  dirty: false,
  wasm_bindgen: '0.2.130',
  btcs: {
    solver: {
      id: 'btcs_five_layer',
      method: 'BTCS five-layer spherical finite difference (nodal)',
      capability: 'diffusion_btcs_layered',
      version: '2.0.0',
      commit: 'abc1234',
      label: 'COMPUTED v2.0',
      formulation: 'docs/mathematics/btcs/02_final_formulation.md',
      scope: 'constant diffusivity in each layer',
      nature: 'numerical solution of the stated model; not experimental validation',
    },
    schema: 2,
    functions: ['validate', 'run', 'compare'],
    export_functions: ['export_run'],
    export_formats: ['timeseries_csv', 'profiles_csv', 'run_json'],
    capabilities: ['diffusion_btcs_layered'],
    label: 'COMPUTED v2.0',
    limits: { max_total_intervals: 1, min_intervals_per_layer: 1, max_steps: 1, max_profile_values: 1, max_profile_times: 1, max_series_modes: 1 },
  },
} satisfies BuildInfo;

class FakeWorker implements WorkerLike {
  onmessage: ((ev: MessageEvent<WorkerMessage>) => void) | null = null;
  onerror: ((ev: ErrorEvent) => void) | null = null;
  received: WorkerRequest[] = [];
  terminated = false;
  postMessage(m: WorkerRequest): void {
    this.received.push(m);
  }
  terminate(): void {
    this.terminated = true;
  }
  emit(msg: WorkerMessage): void {
    this.onmessage?.({ data: msg } as MessageEvent<WorkerMessage>);
  }
  hello(): void {
    this.emit({ kind: 'hello', abi: 3, build: BUILD_INFO });
  }
  reply(id: number, body: unknown): void {
    this.emit({ kind: 'response', id, ok: true, body, elapsed_ms: 1.5 });
  }
}

let workers: FakeWorker[];
const factory = () => {
  const w = new FakeWorker();
  workers.push(w);
  return w;
};

beforeEach(() => {
  workers = [];
  vi.useFakeTimers();
});
afterEach(() => {
  vi.useRealTimers();
});

describe('hello handshake', () => {
  it('queues requests until the worker reports that it is listening, then sends them in order', async () => {
    const onHello = vi.fn();
    const c = new SolverClient(factory, { onHello });
    const p1 = c.request('validate', { a: 1 });
    const p2 = c.request('build_info', '');
    const w = workers[0]!;
    expect(w.received).toHaveLength(0);
    w.hello();
    expect(onHello).toHaveBeenCalledWith(BUILD_INFO);
    expect(w.received.map((r) => r.kind)).toEqual(['validate', 'build_info']);
    expect(w.received[0]!.payload).toBe('{"a":1}');
    w.reply(w.received[1]!.id, { second: true });
    w.reply(w.received[0]!.id, { first: true });
    await expect(p1).resolves.toMatchObject({ body: { first: true }, elapsedMs: 1.5 });
    await expect(p2).resolves.toMatchObject({ body: { second: true } });
    expect(c.sent).toBe(2);
    expect(c.build).toEqual(BUILD_INFO);
  });

  it('reports a module that cannot be loaded and fails the waiting requests', async () => {
    const onLoadFailed = vi.fn();
    const c = new SolverClient(factory, { onLoadFailed });
    const p = c.request('validate', {});
    workers[0]!.emit({ kind: 'load_failed', error: 'CompileError' });
    await expect(p).rejects.toMatchObject({ reason: 'load_failed' });
    expect(onLoadFailed).toHaveBeenCalledWith('CompileError');
  });
});

describe('time-out and recovery', () => {
  it('rejects a request without a reply after the time-out and replaces the worker', async () => {
    const onRestart = vi.fn();
    const c = new SolverClient(factory, { onRestart }, 1000);
    workers[0]!.hello();
    const p = c.request('run', {});
    const caught = p.catch((e: unknown) => e);
    vi.advanceTimersByTime(1001);
    const err = await caught;
    expect(err).toBeInstanceOf(SolverError);
    expect((err as SolverError).reason).toBe('timeout');
    expect(workers[0]!.terminated).toBe(true);
    expect(workers).toHaveLength(2);
    expect(onRestart).toHaveBeenCalledWith('time-out');
    // The new worker serves requests after its own hello; a late reply of the old one is ignored.
    const q = c.request('validate', {});
    workers[0]!.reply(1, { late: true });
    workers[1]!.hello();
    workers[1]!.reply(workers[1]!.received[0]!.id, { ok: 1 });
    await expect(q).resolves.toMatchObject({ body: { ok: 1 } });
    expect(c.restarts).toBe(1);
  });

  it('replaces a worker whose instance trapped and never reuses it', async () => {
    const c = new SolverClient(factory, {});
    workers[0]!.hello();
    const p = c.request('run', {});
    workers[0]!.emit({ kind: 'response', id: workers[0]!.received[0]!.id, ok: false, error: 'RuntimeError: unreachable', trapped: true });
    await expect(p).rejects.toMatchObject({ reason: 'trap' });
    expect(workers[0]!.terminated).toBe(true);
    expect(workers).toHaveLength(2);
  });

  it('keeps the worker after an ordinary error reply', async () => {
    const c = new SolverClient(factory, {});
    workers[0]!.hello();
    const p = c.request('run', {});
    workers[0]!.emit({ kind: 'response', id: workers[0]!.received[0]!.id, ok: false, error: 'SyntaxError', trapped: false });
    await expect(p).rejects.toMatchObject({ reason: 'worker_error' });
    expect(workers).toHaveLength(1);
  });
});

describe('runs', () => {
  it('drops a reply that a newer run has overtaken (stale requests are cancelled by id)', async () => {
    const c = new SolverClient(factory, {});
    workers[0]!.hello();
    const first = c.run({ n: 1 }, ['diffusion_btcs_layered'], ['diffusion_btcs_layered']);
    const second = c.run({ n: 2 }, ['diffusion_btcs_layered'], ['diffusion_btcs_layered']);
    const [r1, r2] = workers[0]!.received;
    workers[0]!.reply(r1!.id, { status: 'ok', n: 1 });
    workers[0]!.reply(r2!.id, { status: 'ok', n: 2 });
    await expect(first).rejects.toMatchObject({ reason: 'stale' });
    await expect(second).resolves.toMatchObject({ body: { n: 2 } });
  });

  it('never sends a mechanism for which isAvailable() is false, and reports it', async () => {
    const c = new SolverClient(factory, {});
    workers[0]!.hello();
    // decay and generation are RESEARCHED in the catalogue: dropped even when a capability claims them.
    const p = c.run({}, ['diffusion_btcs_layered', 'decay', 'generation'], ['diffusion_btcs_layered', 'decay']);
    const sent = JSON.parse(workers[0]!.received[0]!.payload) as { schema: number; mechanisms: string[] };
    expect(sent.schema).toBe(2);
    expect(sent.mechanisms).toEqual(['diffusion_btcs_layered']);
    workers[0]!.reply(workers[0]!.received[0]!.id, { status: 'ok' });
    await expect(p).resolves.toMatchObject({ dropped: ['decay', 'generation'] });
  });

  it('sends nothing for diffusion when the wrapper has not reported its capabilities', async () => {
    const c = new SolverClient(factory, {});
    workers[0]!.hello();
    void c.run({}, ['diffusion_btcs_layered'], []).catch(() => undefined);
    const sent = JSON.parse(workers[0]!.received[0]!.payload) as { mechanisms: string[] };
    expect(sent.mechanisms).toEqual([]);
  });
});
