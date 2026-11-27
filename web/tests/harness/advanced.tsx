// Test harness of the manual baseline solver (advanced/AdvancedSolver.tsx): a page served by the development
// server (npm run harness, port 4433) for the browser tests only. The solver is mounted in place of Explore,
// inside the same header, status line and sections as the site, so the manual-solver tests do not depend on
// which page of the site mounts the component. It is not an entry of the production build (vite builds
// index.html only).
import '@fontsource/literata/latin-400.css';
import '@fontsource/ibm-plex-sans/latin-400.css';
import '@fontsource/ibm-plex-sans/latin-500.css';
import '@fontsource/ibm-plex-sans/latin-600.css';
import '@fontsource/ibm-plex-mono/latin-400.css';
import '../../src/design/tokens.css';
import '../../src/design/app.css';
import '../../src/design/research.css';
import { createRoot } from 'react-dom/client';
import { AdvancedSolver } from '../../src/advanced/AdvancedSolver';
import { Header } from '../../src/panels/Header';
import { StatusLine } from '../../src/panels/StatusLine';
import { installPlaybackClock } from '../../src/particle/playback';
import { useSection } from '../../src/route';
import { About } from '../../src/sections/About';
import { Compare } from '../../src/sections/Compare';
import { Data } from '../../src/sections/Data';
import { Documentation } from '../../src/sections/Documentation';
import { Research } from '../../src/sections/Research';
import { Validate } from '../../src/sections/Validate';
import { Verify } from '../../src/sections/Verify';
import { startSolver } from '../../src/solver/session';
import { startAddressSync } from '../../src/state/address';
import { useMediaQuery, useTheme } from '../../src/theme';

startAddressSync();
startSolver();
installPlaybackClock();

function Harness() {
  const section = useSection();
  const [theme, toggleTheme] = useTheme();
  const tabletUp = useMediaQuery('(min-width: 768px)');
  const view = () => {
    switch (section) {
      case 'compare':
        return <Compare />;
      case 'validate':
        return <Validate />;
      case 'verify':
        return <Verify />;
      case 'research':
        return <Research />;
      case 'data':
        return <Data />;
      case 'about':
        return <About />;
      case 'documentation':
        return <Documentation />;
      default:
        return (
          <article className="section" data-section="advanced">
            <h1 className="visually-hidden">Advanced solver</h1>
            <AdvancedSolver />
          </article>
        );
    }
  };
  return (
    <div className="app app--document">
      <Header section={section} theme={theme} onToggleTheme={toggleTheme} compact={!tabletUp} />
      <main id="main" tabIndex={-1} className="app__main">
        {view()}
      </main>
      <StatusLine compact={!tabletUp} />
    </div>
  );
}

const root = document.getElementById('root');
if (!root) throw new Error('missing #root');
createRoot(root).render(<Harness />);
