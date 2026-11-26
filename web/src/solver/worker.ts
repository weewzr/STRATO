/// <reference lib="webworker" />
// Web Worker that loads the STRATO WebAssembly wrapper (built by strato-webtools build-wasm into
// ./pkg) and answers requests from client.ts. It passes JSON text to the BTCS functions of the
// wrapper and parses the reply; it does no arithmetic on the values. It imports no other function
// of the module: there is no path from the page to the retired finite-volume exports
// (web/tests/btcs_only.test.tsx). After a trap (a panic aborts on wasm32) the instance is unusable;
// the worker reports it and the client replaces the worker.
import init, { abi_version, build_info, compare, export_run, run, validate } from './pkg/strato_wasm.js';
import type { ErrorMessage, HelloMessage, LoadFailedMessage, ResponseMessage, WorkerRequest } from './protocol';
import type { BuildInfo } from './types';

const scope = self as unknown as DedicatedWorkerGlobalScope;
let ready = false;
let dead = false;
const queue: WorkerRequest[] = [];

function call(req: WorkerRequest): string {
  switch (req.kind) {
    case 'run':
      return run(req.payload);
    case 'validate':
      return validate(req.payload);
    case 'compare':
      return compare(req.payload);
    case 'export_run':
      return export_run(req.payload);
    case 'build_info':
      return build_info();
  }
}

function handle(req: WorkerRequest): void {
  if (dead) {
    const msg: ErrorMessage = { kind: 'response', id: req.id, ok: false, error: 'solver instance terminated by an earlier trap', trapped: true };
    scope.postMessage(msg);
    return;
  }
  const start = performance.now();
  try {
    const text = call(req);
    const elapsed = performance.now() - start;
    const msg: ResponseMessage = { kind: 'response', id: req.id, ok: true, body: JSON.parse(text) as unknown, elapsed_ms: elapsed };
    scope.postMessage(msg);
  } catch (e) {
    // A WebAssembly.RuntimeError is a trap: the instance cannot be used again.
    const trapped = e instanceof WebAssembly.RuntimeError;
    if (trapped) dead = true;
    const msg: ErrorMessage = { kind: 'response', id: req.id, ok: false, error: String(e), trapped };
    scope.postMessage(msg);
  }
}

// Listen first, so that nothing the page sends is lost; requests wait until the module is ready.
scope.onmessage = (ev: MessageEvent<WorkerRequest>) => {
  if (ready) handle(ev.data);
  else queue.push(ev.data);
};

init()
  .then(() => {
    const build = JSON.parse(build_info()) as BuildInfo;
    ready = true;
    const hello: HelloMessage = { kind: 'hello', abi: abi_version(), build };
    scope.postMessage(hello);
    for (const req of queue.splice(0)) handle(req);
  })
  .catch((e: unknown) => {
    const msg: LoadFailedMessage = { kind: 'load_failed', error: String(e) };
    scope.postMessage(msg);
  });