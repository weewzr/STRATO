// Colours and faces of the plots, read from the design tokens (web/src/design/tokens.css) so that a
// figure follows the light and dark theme like the rest of the page. Display only.
import { useEffect, useState } from 'react';
import { LAYER_TOKEN_NAMES } from '../design/layers';

export interface PlotTheme {
  text: string;
  muted: string;
  grid: string;
  axis: string;
  surface: string;
  font: string;
  mono: string;
  seriesB: string;
  seriesC: string;
  seriesBase: string;
  seriesDiff: string;
  /** Layer colours, region index from the centre outwards (the particle's colours). */
  layers: string[];
  /** Changes whenever the theme changes, so a figure can be redrawn. */
  key: string;
}

/** Fallbacks for a document without the stylesheet (unit tests): the light theme's values. */
const FALLBACK: Record<string, string> = {
  '--text': '#15181e',
  '--text-muted': '#4f5663',
  '--plot-grid': '#e6e8ec',
  '--border-strong': '#767d8a',
  '--surface': '#ffffff',
  '--font-ui': 'IBM Plex Sans, Arial, sans-serif',
  '--font-mono': 'IBM Plex Mono, Consolas, monospace',
  '--series-b': '#dc2626',
  '--series-c': '#2563eb',
  '--series-base': '#4f5663',
  '--series-diff': '#00735a',
  '--layer-kernel': '#a36429',
  '--layer-buffer': '#42352e',
  '--layer-ipyc': '#1f285c',
  '--layer-sic': '#2c9688',
  '--layer-opyc': '#7c57c7',
};

function token(style: CSSStyleDeclaration | null, name: string): string {
  const v = style?.getPropertyValue(name).trim();
  return v ? v : (FALLBACK[name] ?? '#000000');
}

export function readPlotTheme(): PlotTheme {
  const style = typeof document !== 'undefined' && typeof getComputedStyle === 'function' ? getComputedStyle(document.documentElement) : null;
  const t = (n: string) => token(style, n);
  const theme = {
    text: t('--text'),
    muted: t('--text-muted'),
    grid: t('--plot-grid'),
    axis: t('--border-strong'),
    surface: t('--surface'),
    font: t('--font-ui'),
    mono: t('--font-mono'),
    seriesB: t('--series-b'),
    seriesC: t('--series-c'),
    seriesBase: t('--series-base'),
    seriesDiff: t('--series-diff'),
    layers: LAYER_TOKEN_NAMES.map((n) => t(n)),
  };
  return { ...theme, key: `${theme.text}|${theme.seriesB}|${theme.layers.join(',')}` };
}

/** The plot colours of the current theme; follows the theme toggle and the system setting. */
export function usePlotTheme(): PlotTheme {
  const [theme, setTheme] = useState<PlotTheme>(readPlotTheme);
  useEffect(() => {
    const update = () =>
      setTheme((prev) => {
        const next = readPlotTheme();
        return next.key === prev.key ? prev : next;
      });
    const obs = typeof MutationObserver === 'function' ? new MutationObserver(update) : null;
    obs?.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
    const mq = typeof window.matchMedia === 'function' ? window.matchMedia('(prefers-color-scheme: dark)') : null;
    mq?.addEventListener('change', update);
    update();
    return () => {
      obs?.disconnect();
      mq?.removeEventListener('change', update);
    };
  }, []);
  return theme;
}
