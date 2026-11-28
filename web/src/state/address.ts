// The Explore selection in the address (owner directive of 26 Nov 2026, Section 6: "preserve
// selections"; a copied address reproduces the view). Written as a query after the route, for example
//   #/explore?physics=generation&scenario=startup&open=numerical
// A part that has its default value is left out; a bare #/explore is the default view. Nothing is
// stored in the browser: the address and the in-memory store (state/store.ts) are the only places.
import { configIdToFlags, flagsToConfigId, flagsToPhysicsId, PHYSICS_CONFIG_IDS, type OverlayFlags, type PhysicsConfigId } from '../equation/capability';
import { sectionFromHash } from '../route';
import { DEFAULT_SELECTION, normaliseSelection, sameSelection, useStrato, type Selection } from './store';

const KEYS = ['physics', 'config', 'scenario', 'open', 'graph', 'species', 'temperature', 'dataset', 'overlays', 'controlled'] as const;

function formatOverlays(o: OverlayFlags): string | null {
  if (o.methodA && o.methodB && o.experiment) return null;
  const parts = [o.methodA ? 'A' : null, o.methodB ? 'B' : null, o.experiment ? 'Exp' : null].filter(Boolean);
  return parts.length > 0 ? parts.join(',') : 'none';
}

function parseOverlays(raw: string | null): Partial<OverlayFlags> | undefined {
  if (raw === null) return undefined;
  const trimmed = raw.trim();
  if (trimmed === '' || trimmed === 'none') return { methodA: false, methodB: false, experiment: false };
  const tokens = new Set(trimmed.split(',').map((s) => s.trim().toLowerCase()));
  return {
    methodA: tokens.has('a') || tokens.has('methoda'),
    methodB: tokens.has('b') || tokens.has('methodb'),
    experiment: tokens.has('exp') || tokens.has('experiment'),
  };
}

/** The query (without the question mark) that stands for a selection; empty for the default selection. */
export function formatSelection(sel: Selection): string {
  const q = new URLSearchParams();
  const cid = flagsToConfigId(sel.mechanisms);
  if (cid === 'P05' || cid === 'P06' || cid === 'P07') {
    q.set('config', cid);
  } else if (sel.physics !== DEFAULT_SELECTION.physics) {
    q.set('physics', sel.physics);
  }
  if (sel.scenario !== DEFAULT_SELECTION.scenario) q.set('scenario', sel.scenario ?? 'none');
  if (sel.expanded.length > 0) q.set('open', sel.expanded.join(','));
  if (sel.graph !== DEFAULT_SELECTION.graph) q.set('graph', sel.graph);
  if (sel.species !== '') q.set('species', sel.species);
  if (sel.temperature !== null) q.set('temperature', String(sel.temperature));
  if (sel.dataset !== '') q.set('dataset', sel.dataset);
  const ov = formatOverlays(sel.overlays);
  if (ov !== null) q.set('overlays', ov);
  if (sel.controlledCompare) q.set('controlled', '1');
  return q.toString().replace(/%2C/g, ',');
}

/** The selection that a query stands for; an unknown or missing part is the default. */
export function parseSelection(query: string): Selection {
  const q = new URLSearchParams(query);
  const t = q.get('temperature');
  const rawCfg = q.get('config');
  const hasValidCfg = rawCfg !== null && PHYSICS_CONFIG_IDS.includes(rawCfg as PhysicsConfigId);
  const mechanisms = hasValidCfg ? configIdToFlags(rawCfg as PhysicsConfigId) : undefined;
  const rawPhysics = q.get('physics') ?? undefined;
  const physics = hasValidCfg && !rawPhysics ? flagsToPhysicsId(mechanisms!) : rawPhysics;
  return normaliseSelection({
    physics,
    mechanisms,
    scenario: q.get('scenario') ?? undefined,
    expanded: (q.get('open') ?? '').split(',').filter(Boolean),
    graph: q.get('graph') ?? undefined,
    species: q.get('species') ?? undefined,
    temperature: t !== null && t.trim() !== '' ? Number(t) : null,
    dataset: q.get('dataset') ?? undefined,
    overlays: parseOverlays(q.get('overlays')),
    controlledCompare: q.get('controlled') === '1' || q.get('controlled') === 'true',
  });
}

/** True when the query names at least one part of the selection. */
export function hasSelectionKeys(query: string): boolean {
  const q = new URLSearchParams(query);
  return KEYS.some((k) => q.has(k));
}

/** The hash of the Explore page for a selection. */
export function exploreHash(sel: Selection): string {
  const q = formatSelection(sel);
  return q ? `#/explore?${q}` : '#/explore';
}

const queryOf = (hash: string): string => {
  const i = hash.indexOf('?');
  return i < 0 ? '' : hash.slice(i + 1);
};

/**
 * Keeps the selection and the address in step, in both directions, while the Explore page is the
 * current section: a query in the address sets the selection (also when the address is edited or a link
 * is followed), and a change of the selection rewrites the address (history.replaceState, so the back
 * button is not filled with every click). An Explore address with no query keeps the selection already
 * held, so the navigation link back to Explore returns to the view that was left. Returns a function that stops it.
 */
export function startAddressSync(): () => void {
  const onExplore = () => sectionFromHash(window.location.hash) === 'explore';

  const write = () => {
    if (!onExplore()) return;
    const sel = useStrato.getState().selection;
    const query = formatSelection(sel);
    const have = queryOf(window.location.hash);
    if (query === '' && have === '') return;
    const want = exploreHash(sel);
    if (window.location.hash !== want) window.history.replaceState(window.history.state, '', want);
  };

  const read = () => {
    if (!onExplore()) return;
    const query = queryOf(window.location.hash);
    if (hasSelectionKeys(query)) {
      const next = parseSelection(query);
      if (!sameSelection(next, useStrato.getState().selection)) useStrato.setState({ selection: next });
      // Normalise the address to the selection actually held (an invalid value falls back to the default).
      const want = exploreHash(useStrato.getState().selection);
      if (window.location.hash !== want) window.history.replaceState(window.history.state, '', want);
    } else write();
  };

  read();
  window.addEventListener('hashchange', read);
  const unsubscribe = useStrato.subscribe((state, prev) => {
    if (state.selection !== prev.selection) write();
  });
  return () => {
    window.removeEventListener('hashchange', read);
    unsubscribe();
  };
}
