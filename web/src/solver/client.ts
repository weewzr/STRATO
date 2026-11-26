// Request/response wrapper around the solver worker (prompt/TECHNICAL_ARCHITECTURE.md Section 4),
// for the BTCS interface of the wrapper only:
// hello handshake (requests wait until the worker reports that it is listening), one id per
// request, a time-out per request, replacement of a worker that trapped or timed out, and
// "latest wins" for runs so that a stale reply is dropped by its id. It passes JSON text and
// returns the wrapper's parsed reply; it computes nothing.
import { filterRequestedMechanisms } from '../equation/availability';
import type { ErrorMessage, HelloMessage, RequestKind, ResponseMessage, WorkerMessage, WorkerRequest } from './protocol';
import { REQUEST_TIMEOUT_MS } from './tolerances';
import type { BuildInfo, CompareReply, ExportFormat, ExportReply, RunReply, ValidateReply } from './types';

/** The part of the Worker interface the client uses (a fake in the unit tests). */
export interface WorkerLike {
  postMessage(message: WorkerRequest): void;
  terminate(): void;
  onmessage: ((ev: MessageEvent<WorkerMessage>) => void) | null;
  onerror: ((ev: ErrorEvent) => void) | null;
}

export type WorkerFactory = () => WorkerLike;

export class SolverError extends Error {
  readonly reason: 'timeout' | 'trap' | 'load_failed' | 'worker_error' | 'stale' | 'closed';
  constructor(reason: SolverError['reason'], message: string) {
    super(message);
    this.name = 'SolverError';
    this.reason = reason;
  }
}

export interface Reply<T> {
  body: T;
  /** Wall time of the wrapper call in the worker, measured with performance.now(). */
  elapsedMs: number;
  requestId: number;
}

interface Pending {
  kind: RequestKind;
  resolve: (r: Reply<unknown>) => void;
  reject: (e: Error) => void;
  timer: ReturnType<typeof setTimeout>;
}

export interface ClientEvents {
  onHello?: (build: BuildInfo) => void;
  onLoadFailed?: (error: string) => void;
  /** A worker was replaced after a trap or a time-out. */
  onRestart?: (reason: string) => void;
}

export class SolverClient {
  private worker: WorkerLike | null = null;
  private nextId = 1;
  private hello: HelloMessage | null = null;
  private queue: WorkerRequest[] = [];
  private pending = new Map<number, Pending>();
  private latestRun = 0;
  private latestCompare = 0;
  private closed = false;
  /** Requests posted to a worker, for the tests that require that no request is sent. */
  sent = 0;
  restarts = 0;

  constructor(
    private readonly factory: WorkerFactory,
    private readonly events: ClientEvents = {},
    private readonly timeoutMs = REQUEST_TIMEOUT_MS,
  ) {
    this.start();
  }

  get build(): BuildInfo | null {
    return this.hello?.build ?? null;
  }

  private start(): void {
    const w = this.factory();
    this.worker = w;
    this.hello = null;
    w.onmessage = (ev) => this.onMessage(w, ev.data);
    w.onerror = (ev) => {
      if (w !== this.worker) return;
      const text = typeof ev.message === 'string' && ev.message ? ev.message : 'worker error';
      this.failAll(new SolverError('worker_error', text));
      this.replace(`worker error: ${text}`);
    };
  }

  private onMessage(w: WorkerLike, msg: WorkerMessage): void {
    if (w !== this.worker) return; // a message from a worker that was replaced
    if (msg.kind === 'hello') {
      this.hello = msg;
      this.events.onHello?.(msg.build);
      for (const req of this.queue.splice(0)) this.post(req);
      return;
    }
    if (msg.kind === 'load_failed') {
      this.failAll(new SolverError('load_failed', msg.error));
      this.events.onLoadFailed?.(msg.error);
      return;
    }
    this.settle(msg);
  }

  private settle(msg: ResponseMessage | ErrorMessage): void {
    const p = this.pending.get(msg.id);
    if (!p) return;
    clearTimeout(p.timer);
    this.pending.delete(msg.id);
    if (msg.ok) {
      p.resolve({ body: msg.body, elapsedMs: msg.elapsed_ms, requestId: msg.id });
      return;
    }
    p.reject(new SolverError(msg.trapped ? 'trap' : 'worker_error', msg.error));
    if (msg.trapped) {
      this.failAll(new SolverError('trap', `solver instance replaced after a trap: ${msg.error}`));
      this.replace(`trap: ${msg.error}`);
    }
  }

  private failAll(err: Error): void {
    for (const [id, p] of this.pending) {
      clearTimeout(p.timer);
      p.reject(err);
      this.pending.delete(id);
    }
    this.queue = [];
  }

  /** Terminates the current worker and starts a fresh one (a trapped instance is never reused). */
  private replace(reason: string): void {
    if (this.closed) return;
    this.worker?.terminate();
    this.restarts++;
    this.events.onRestart?.(reason);
    this.start();
  }

  private post(req: WorkerRequest): void {
    this.sent++;
    this.worker?.postMessage(req);
  }

  request<T>(kind: RequestKind, payload: unknown): Promise<Reply<T>> {
    if (this.closed) return Promise.reject(new SolverError('closed', 'client closed'));
    const id = this.nextId++;
    const req: WorkerRequest = { id, kind, payload: typeof payload === 'string' ? payload : JSON.stringify(payload) };
    return new Promise<Reply<T>>((resolve, reject) => {
      const timer = setTimeout(() => {
        if (!this.pending.has(id)) return;
        this.pending.delete(id);
        reject(new SolverError('timeout', `no reply within ${this.timeoutMs} ms`));
        // The call may still be running inside the instance; replace the worker.
        this.failAll(new SolverError('timeout', 'solver replaced after a time-out'));
        this.replace('time-out');
      }, this.timeoutMs);
      this.pending.set(id, { kind, resolve: resolve as (r: Reply<unknown>) => void, reject, timer });
      if (this.hello) this.post(req);
      else this.queue.push(req);
    });
  }

  /**
   * Runs a configuration with the BTCS solver. Only mechanisms for which isAvailable() is true are
   * sent; the others are dropped and reported. A reply that a newer run has overtaken rejects with
   * reason 'stale'. A rejected configuration comes back as the wrapper's own reply, with its own messages.
   */
  async run(config: unknown, mechanisms: readonly string[], capabilities: readonly string[]): Promise<Reply<RunReply> & { dropped: string[] }> {
    const { send, dropped } = filterRequestedMechanisms(mechanisms, capabilities);
    const ticket = ++this.latestRun;
    const reply = await this.request<RunReply>('run', { schema: 2, config, mechanisms: send });
    if (ticket !== this.latestRun) throw new SolverError('stale', 'overtaken by a newer run');
    return { ...reply, dropped };
  }

  /**
   * Runs the baseline and the current configuration through compare, which returns both runs and
   * their difference computed in Rust. Mechanisms are filtered as for run, for both halves. Latest
   * wins, separately from run: a reply that a newer comparison has overtaken rejects with 'stale'.
   */
  async compare(
    baseline: unknown,
    modified: unknown,
    mechanisms: readonly string[],
    capabilities: readonly string[],
  ): Promise<Reply<CompareReply> & { dropped: string[] }> {
    const { send, dropped } = filterRequestedMechanisms(mechanisms, capabilities);
    const ticket = ++this.latestCompare;
    const reply = await this.request<CompareReply>('compare', {
      schema: 2,
      baseline: { schema: 2, config: baseline, mechanisms: send },
      modified: { schema: 2, config: modified, mechanisms: send },
    });
    if (ticket !== this.latestCompare) throw new SolverError('stale', 'overtaken by a newer comparison');
    return { ...reply, dropped };
  }

  /** The wrapper's check of a configuration: the same rules as run, nothing is solved. */
  validate(config: unknown): Promise<Reply<ValidateReply>> {
    return this.request<ValidateReply>('validate', { schema: 2, config });
  }

  /**
   * A data file of a run, written by the library's export with its provenance header. The text is
   * saved unchanged. `source` and `parameterClass` are the source row and class of the parameters.
   */
  exportRun(
    config: unknown,
    format: ExportFormat,
    provenance: { inputFile: string; parameterSource: string; parameterClass: string },
  ): Promise<Reply<ExportReply>> {
    return this.request<ExportReply>('export_run', {
      schema: 2,
      config,
      format,
      input_file: provenance.inputFile,
      parameter_source: provenance.parameterSource,
      parameter_class: provenance.parameterClass,
    });
  }
  close(): void {
    this.closed = true;
    this.failAll(new SolverError('closed', 'client closed'));
    this.worker?.terminate();
    this.worker = null;
  }
}
