import { useEffect, useRef, useState } from 'react';
import { ControlPanel } from './panels/ControlPanel';
import { Header } from './panels/Header';
import { StatusLine } from './panels/StatusLine';
import { useSection, type SectionId } from './route';
import { Basis } from './sections/Basis';
import { NavigatedContext } from './sections/common';
import { Documentation } from './sections/Documentation';
import { Explore } from './sections/Explore';
import { Research } from './sections/Research';
import { useMediaQuery, useTheme } from './theme';

// STRATO's binding breakpoints (prompt/DESIGN_SPECIFICATION.md Section 2.1): desktop from 1200 px,
// tablet from 768 px, phone below. On a phone the controls of block C open in a bottom sheet; the
// block order A to E is the same at every width.
const TABLET_UP = '(min-width: 768px)';

export function App() {
  const section = useSection();
  const [theme, toggleTheme] = useTheme();
  const tabletUp = useMediaQuery(TABLET_UP);
  const [sheetOpen, setSheetOpen] = useState(false);
  const [navigated, setNavigated] = useState(false);
  const opener = useRef<HTMLElement | null>(null);
  const sheet = useRef<HTMLDivElement>(null);
  const mainRef = useRef<HTMLElement>(null);

  const sheetMode = section === 'explore' && !tabletUp;

  // Close the sheet on section change and when the layout leaves the phone width.
  const [lastSection, setLastSection] = useState(section);
  if (lastSection !== section) {
    setLastSection(section);
    if (!navigated) setNavigated(true);
    if (sheetOpen) setSheetOpen(false);
  }
  if (sheetOpen && !sheetMode) setSheetOpen(false);

  useEffect(() => {
    if (!sheetOpen) return;
    sheet.current?.querySelector<HTMLElement>('h2')?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setSheetOpen(false);
        opener.current?.focus();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [sheetOpen]);

  const openSheet = () => {
    opener.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    setSheetOpen(true);
  };
  const closeSheet = () => {
    setSheetOpen(false);
    opener.current?.focus();
  };

  const view = (id: SectionId) => {
    switch (id) {
      case 'explore':
        return <Explore compact={!tabletUp} onOpenControls={openSheet} />;
      case 'basis':
        return <Basis />;
      case 'research':
        return <Research />;
      case 'documentation':
        return <Documentation />;
    }
  };

  return (
    <NavigatedContext.Provider value={navigated}>
      <div className={`app app--${section === 'explore' ? 'instrument' : 'document'}`}>
        <a
          className="skip-link"
          href="#main"
          onClick={(e) => {
            e.preventDefault();
            mainRef.current?.focus();
          }}
        >
          Skip to content
        </a>
        <Header section={section} theme={theme} onToggleTheme={toggleTheme} compact={!tabletUp} />
        <main id="main" ref={mainRef} tabIndex={-1} className="app__main">
          {view(section)}
        </main>
        {sheetMode ? (
          <div
            id="controls-sheet"
            ref={sheet}
            className="sheet sheet--bottom"
            role="dialog"
            aria-modal="false"
            aria-label="Controls"
            hidden={!sheetOpen}
            data-testid="parameter-sheet"
          >
            {sheetOpen ? <ControlPanel layout="sheet" onClose={closeSheet} /> : null}
          </div>
        ) : null}
        <StatusLine compact={!tabletUp} />
      </div>
    </NavigatedContext.Provider>
  );
}
