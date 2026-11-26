// Test harness of the experimental-comparison graph: a page served by the development server
// (npm run harness, port 4433) for the browser tests only. It mounts the component with the fixtures
// of tests/fixtures. It is not an entry of the production build (vite builds index.html only), and a
// test fails if a fixture appears in the build output.
import '@fontsource/ibm-plex-sans/latin-400.css';
import '@fontsource/ibm-plex-mono/latin-400.css';
import '../../src/design/tokens.css';
import '../../src/design/app.css';
import { createRoot } from 'react-dom/client';
import { ExperimentalComparisons } from '../../src/graph/Comparisons';
import { isComparisonFile, type ComparisonFile } from '../../src/graph/contract';
import both from '../fixtures/fixture_r2_both_methods.json';
import absent from '../fixtures/fixture_r2_method_b_absent.json';
import negative from '../fixtures/fixture_r2_negative.json';
import bounds from '../fixtures/fixture_r2_upper_bounds.json';

const ALL: Record<string, unknown> = { both, negative, bounds, absent };
const params = new URLSearchParams(window.location.search);
const theme = params.get('theme');
if (theme === 'light' || theme === 'dark') document.documentElement.setAttribute('data-theme', theme);
const chosen = (params.get('f') ?? 'both,negative,bounds,absent').split(',');
const files = chosen.map((k) => ALL[k]).filter(isComparisonFile) as ComparisonFile[];

const root = document.getElementById('root');
if (!root) throw new Error('missing #root');
createRoot(root).render(
  <div className="app app--document">
    <main id="main" className="app__main">
      <article className="section" data-section="harness">
        <ExperimentalComparisons files={files} />
      </article>
    </main>
  </div>,
);
