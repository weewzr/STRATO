// Loads the precomputed native output in web/public/data/ (written by strato-webtools precomputed,
// each file with a provenance header; prompt/TECHNICAL_ARCHITECTURE.md Section 5) with a plain fetch,
// once per file. The values are shown as they are in the file.
import { useEffect, useState } from 'react';

const cache = new Map<string, Promise<unknown>>();

export function precomputedUrl(file: string): string {
  return `${import.meta.env.BASE_URL}data/${file}`;
}

export function loadPrecomputed<T>(file: string): Promise<T> {
  let p = cache.get(file);
  if (!p) {
    p = fetch(precomputedUrl(file)).then((r) => {
      if (!r.ok) throw new Error(`${file}: HTTP ${r.status}`);
      return r.json() as Promise<unknown>;
    });
    p.catch(() => cache.delete(file));
    cache.set(file, p);
  }
  return p as Promise<T>;
}

export interface Loaded<T> {
  doc: T | null;
  error: string | null;
}

export function usePrecomputed<T>(file: string): Loaded<T> {
  const [state, setState] = useState<Loaded<T>>({ doc: null, error: null });
  useEffect(() => {
    let alive = true;
    loadPrecomputed<T>(file)
      .then((doc) => {
        if (alive) setState({ doc, error: null });
      })
      .catch((e: unknown) => {
        if (alive) setState({ doc: null, error: e instanceof Error ? e.message : String(e) });
      });
    return () => {
      alive = false;
    };
  }, [file]);
  return state;
}
