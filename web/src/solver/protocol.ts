// Messages between the page and the solver worker (prompt/TECHNICAL_ARCHITECTURE.md Section 4,
// "Worker protocol"). Types only.
import type { BuildInfo } from './types';

export type RequestKind = 'run_v1' | 'analytical' | 'validate' | 'compare' | 'build_info';

export interface WorkerRequest {
  id: number;
  kind: RequestKind;
  /** JSON text handed unchanged to the wrapper function. */
  payload: string;
}

/** Posted once, when the module is instantiated and the worker is listening. */
export interface HelloMessage {
  kind: 'hello';
  abi: number;
  build: BuildInfo;
}

/** Posted instead of hello when the module cannot be loaded. */
export interface LoadFailedMessage {
  kind: 'load_failed';
  error: string;
}

export interface ResponseMessage {
  kind: 'response';
  id: number;
  ok: true;
  /** The wrapper's reply, parsed. */
  body: unknown;
  /** Wall time of the wrapper call inside the worker, in milliseconds (performance.now()). */
  elapsed_ms: number;
}

export interface ErrorMessage {
  kind: 'response';
  id: number;
  ok: false;
  error: string;
  /** True when the call trapped the instance (a panic aborts on wasm32): the worker is unusable. */
  trapped: boolean;
}

export type WorkerMessage = HelloMessage | LoadFailedMessage | ResponseMessage | ErrorMessage;
