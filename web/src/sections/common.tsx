import { createContext, useContext, useEffect, useRef, type ReactNode, type RefObject } from 'react';

/** True once the reader has moved between sections in this page view. */
export const NavigatedContext = createContext(false);

/** After in-page navigation the section's h1 takes focus, so screen readers announce the new section. */
export function useHeadingFocus(): RefObject<HTMLHeadingElement | null> {
  const ref = useRef<HTMLHeadingElement>(null);
  const navigated = useContext(NavigatedContext);
  useEffect(() => {
    if (navigated) ref.current?.focus();
  }, [navigated]);
  return ref;
}

/** The page heading of a section. */
export function SectionHeading({ children, lede }: { children: ReactNode; lede?: ReactNode }) {
  const ref = useHeadingFocus();
  return (
    <div className="section-head">
      <h1 ref={ref} tabIndex={-1} className="section-head__title">
        {children}
      </h1>
      {lede ? <p className="section-head__lede">{lede}</p> : null}
    </div>
  );
}

/** A table that scrolls sideways inside its own labelled, focusable box (shared core A9). */
export function TableBox({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="table-box" role="region" aria-label={label} tabIndex={0}>
      {children}
    </div>
  );
}

export function Note({ children, tone = 'info' }: { children: ReactNode; tone?: 'info' | 'warn' }) {
  return <div className={`note note--${tone}`}>{children}</div>;
}
