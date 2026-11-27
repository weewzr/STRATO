// A pure mapping from a comparison file to the model of its plot. It selects and orders; it does no
// arithmetic on a scientific value. The arrays of a trace are the file's arrays themselves (the same
// objects), the R² text is the file's label_plain exactly, the caption is the file's caption_plain.
import { METHOD_1, METHOD_2 } from '../methods';
import type { ComparisonFile, MethodId, Source } from './contract';

/** Fixed colours of the graph (decision 53): Method A blue, Method B red, observations black. */
export const COLOURS = { A: '#DC2626', B: '#2563EB', observations: '#111111' } as const;

export type TraceRole = 'method_a' | 'method_b' | 'measurement' | 'upper_bound';

export interface TraceModel {
  id: string;
  role: TraceRole;
  name: string;
  t: readonly number[];
  f: readonly number[];
}

export interface R2Label {
  method: MethodId;
  /** The record's label_plain, verbatim. */
  text: string;
  colour: string;
}

export interface PlotModel {
  /** Nuclide and experimental temperature. */
  title: string;
  timeUnit: string;
  traces: TraceModel[];
  /** One label per applicable method whose record holds a value. */
  r2Labels: R2Label[];
  /** The reason R² is unavailable, from the record(s); null where every applicable record has a value. */
  caption: string | null;
  source: Source;
}

export function toPlotModel(file: ComparisonFile): PlotModel {
  const traces: TraceModel[] = [{ id: 'exp-method-a', role: 'method_a', name: METHOD_1, t: file.method_a.t, f: file.method_a.f }];
  if (file.method_b) traces.push({ id: 'exp-method-b', role: 'method_b', name: METHOD_2, t: file.method_b.t, f: file.method_b.f });
  file.observations.forEach((o, i) =>
    traces.push({
      id: `exp-${o.kind === 'upper_bound' ? 'upper-bound' : 'observations'}-${i}`,
      role: o.kind === 'upper_bound' ? 'upper_bound' : 'measurement',
      name: o.kind === 'upper_bound' ? 'Experimental upper bounds' : 'Experimental observations',
      t: o.t,
      f: o.f,
    }),
  );
  const r2Labels: R2Label[] = [];
  const reasons: string[] = [];
  for (const method of ['A', 'B'] as const) {
    const r = file.r2.find((x) => x.method === method);
    if (!r) continue;
    if (r.value !== null) r2Labels.push({ method, text: r.label_plain, colour: COLOURS[method] });
    else if (!reasons.includes(r.caption_plain)) reasons.push(r.caption_plain);
  }
  return {
    title: `${file.nuclide}, ${String(file.temperature.value)} ${file.temperature.unit}`,
    timeUnit: file.time_unit,
    traces,
    r2Labels,
    caption: reasons.length > 0 ? reasons.join(' ') : null,
    source: file.source,
  };
}
