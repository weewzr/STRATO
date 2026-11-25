import katex from 'katex';
import 'katex/dist/katex.min.css';
import { useMemo } from 'react';

// KaTeX, self-hosted through the bundle (fonts under the SIL Open Font License, see prompt/PROGRESS.md).
// Synchronous rendering; HTML for the eye and MathML for screen readers (KaTeX's default output).
// Only \htmlClass is trusted, for the highlight of the new term (prompt/TECHNICAL_ARCHITECTURE.md Section 4).

const cache = new Map<string, string>();

export const KATEX_OPTIONS = {
  throwOnError: true,
  // Every non-standard input is an error, except the HTML extension used for \htmlClass.
  strict: (code: string) => (code === 'htmlExtension' ? ('ignore' as const) : ('error' as const)),
  output: 'htmlAndMathml' as const,
  trust: (ctx: { command: string }) => ctx.command === '\\htmlClass',
};

/** Renders LaTeX to an HTML string; throws on any KaTeX error (the unit tests render every string). */
export function renderTex(latex: string, display: boolean): string {
  const key = `${display ? 'D' : 'I'}${latex}`;
  let html = cache.get(key);
  if (html === undefined) {
    // A matrix row is a little taller than the default so that subscripts and superscripts of neighbouring rows do not touch (rendering option only; the LaTeX is unchanged).
    html = katex.renderToString(latex, { ...KATEX_OPTIONS, displayMode: display, macros: { '\\arraystretch': '1.3' } });
    cache.set(key, html);
  }
  return html;
}

export function Tex({ latex, display = false, className, testId }: { latex: string; display?: boolean; className?: string; testId?: string }) {
  const html = useMemo(() => renderTex(latex, display), [latex, display]);
  const cls = `tex${display ? ' tex--display' : ''}${className ? ` ${className}` : ''}`;
  return display ? (
    <div className={cls} data-testid={testId} dangerouslySetInnerHTML={{ __html: html }} />
  ) : (
    <span className={cls} data-testid={testId} dangerouslySetInnerHTML={{ __html: html }} />
  );
}
