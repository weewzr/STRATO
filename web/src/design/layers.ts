// One colour token per TRISO layer (tokens.css), by region index from the centre outwards.
// v1.0 always has the five regions kernel, buffer, IPyC, SiC and OPyC (Config::validate); the
// layer names shown on screen come from the active configuration, not from this file.
const LAYER_TOKENS = ['--layer-kernel', '--layer-buffer', '--layer-ipyc', '--layer-sic', '--layer-opyc'] as const;

export type LayerToken = (typeof LAYER_TOKENS)[number];

/** The colour token of region `index`, or null when the index has no layer colour. */
export function layerToken(index: number): LayerToken | null {
  return LAYER_TOKENS[index] ?? null;
}

/** CSS colour value for region `index`; regions without a token fall back to the strong border colour. */
export function layerColour(index: number): string {
  const t = layerToken(index);
  return t ? `var(${t})` : 'var(--border-strong)';
}

export const LAYER_TOKEN_NAMES: readonly string[] = LAYER_TOKENS;
