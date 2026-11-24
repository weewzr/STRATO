import type { ReactNode } from 'react';

/**
 * A reserved place on the page for content that a later web phase delivers. It is drawn as a plain
 * dashed panel with the word "Placeholder" and the phase, so it can never be mistaken for a result,
 * a figure or a research status (prompt/DESIGN_SPECIFICATION.md Section 9: pending content sits in
 * its own outlined panels and never uses the chart and particle styles of computed results).
 */
export function Placeholder({
  phase,
  title,
  children,
  testId,
  className,
}: {
  phase: string;
  title: string;
  children: ReactNode;
  testId?: string;
  className?: string;
}) {
  return (
    <div className={`placeholder${className ? ` ${className}` : ''}`} data-testid={testId} data-placeholder-phase={phase}>
      <p className="placeholder__tag">
        <span className="placeholder__word">Placeholder</span>
        <span className="placeholder__phase">web phase {phase}</span>
      </p>
      <p className="placeholder__title">{title}</p>
      <div className="placeholder__body">{children}</div>
    </div>
  );
}
