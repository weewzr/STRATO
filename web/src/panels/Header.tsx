import { useEffect, useRef, useState } from 'react';
import '../design/nav.css';
import { REPO_URL } from '../build';
import { hrefFor, SECTIONS, type SectionId } from '../route';
import type { Theme } from '../theme';

interface Props {
  section: SectionId;
  theme: Theme;
  onToggleTheme: () => void;
  compact: boolean;
}

export function Header({ section, theme, onToggleTheme, compact }: Props) {
  const [menuOpen, setMenuOpen] = useState(false);
  const menuButton = useRef<HTMLButtonElement>(null);

  // Close the phone menu on navigation, and on Escape (focus returns to the menu button).
  useEffect(() => {
    if (!menuOpen) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setMenuOpen(false);
        menuButton.current?.focus();
      }
    };
    const onHash = () => setMenuOpen(false);
    window.addEventListener('keydown', onKey);
    window.addEventListener('hashchange', onHash);
    return () => {
      window.removeEventListener('keydown', onKey);
      window.removeEventListener('hashchange', onHash);
    };
  }, [menuOpen]);

  const showNav = !compact || menuOpen;

  return (
    <header className="header">
      <div className="header__brand">
        <a className="header__wordmark" href={hrefFor('explore')}>
          STRATO
        </a>
        <span className="header__subtitle">Fission-product diffusion in a TRISO particle</span>
      </div>
      {compact ? (
        <button
          ref={menuButton}
          type="button"
          className="btn btn--quiet header__menu"
          aria-expanded={menuOpen}
          aria-controls="site-nav"
          onClick={() => setMenuOpen((o) => !o)}
        >
          {menuOpen ? 'Close menu' : 'Menu'}
        </button>
      ) : null}
      <nav id="site-nav" className={`nav${compact ? ' nav--compact' : ''}`} aria-label="Sections" hidden={!showNav}>
        <ul className="nav__list" role="list">
          {SECTIONS.map((s) => (
            <li key={s.id}>
              <a className="nav__link" href={hrefFor(s.id)} aria-current={s.id === section ? 'page' : undefined}>
                {s.label}
              </a>
            </li>
          ))}
        </ul>
        <div className="nav__tools">
          <button type="button" className="btn btn--quiet" aria-pressed={theme === 'dark'} onClick={onToggleTheme} data-testid="theme-toggle">
            Dark theme
          </button>
          <a className="nav__link nav__link--ext" href={REPO_URL}>
            Repository
          </a>
        </div>
      </nav>
    </header>
  );
}
