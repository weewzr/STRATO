import { useEffect, useState } from 'react';
import { SolverError } from '../solver/client';
import { solverClient } from '../solver/session';
import type { AnalyticalReply, OptionsReply, StageOk } from '../solver/types';
import { useStrato } from '../state/store';
import type { StageRequest } from './exploreData';

// The wrapper's answers for the graphs of Explore, kept apart from the store: the reply of run_stage for
// the request of the selection, Method B only where the stage reply says the analytical method applies,
// and the options reply (nuclides and temperatures). A reply is shown only for the request that asked for
// it: when the selection changes, the earlier reply is dropped (it is never left on screen), and a reply
// that a newer request has overtaken is ignored. Nothing is computed here.

export interface StageResult {
  /** The key of the request this result answers. */
  key: string;
  reply: StageOk;
  /** Method B at the output times, only where the stage reply says it applies and the wrapper answered. */
  analytical: AnalyticalReply | null;
}

/** The result for a request, or null while it is pending, when none was asked, or when the wrapper did not answer with a result. */
export function useStageResult(request: StageRequest | null): StageResult | null {
  const [held, setHeld] = useState<StageResult | null>(null);
  const ready = useStrato((s) => s.solver.state === 'ready');
  const key = request?.key ?? null;

  useEffect(() => {
    const client = solverClient();
    if (request === null || client === null || !ready) return;
    let live = true;
    void (async () => {
      try {
        const stage = await client.runStage(request.body);
        if (!live || stage.body.status !== 'ok') return;
        let analytical: AnalyticalReply | null = null;
        if (stage.body.analytical.applicable) {
          const { profile_times_s: _omitted, ...withoutProfiles } = request.body;
          void _omitted;
          try {
            const b = await client.analytical(withoutProfiles);
            if (b.body.applicable && b.body.release_fraction !== null) analytical = b.body;
          } catch (e) {
            if (e instanceof SolverError && e.reason === 'stale') return;
          }
        }
        if (live) setHeld({ key: request.key, reply: stage.body, analytical });
      } catch {
        // A rejected, stale or failed request leaves no result: the panel stays empty.
      }
    })();
    return () => {
      live = false;
    };
    // The request is identified by its key.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, ready]);

  return held !== null && held.key === key ? held : null;
}

let optionsCache: OptionsReply | null = null;

/** The wrapper's options reply (read once; the reply is the same text for every call). Null until it has answered. */
export function useOptions(): OptionsReply | null {
  const [options, setOptions] = useState<OptionsReply | null>(optionsCache);
  const ready = useStrato((s) => s.solver.state === 'ready');
  useEffect(() => {
    const client = solverClient();
    if (optionsCache !== null || client === null || !ready) return;
    let live = true;
    void client
      .options()
      .then((r) => {
        optionsCache = r.body;
        if (live) setOptions(r.body);
      })
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [ready]);
  return options;
}
