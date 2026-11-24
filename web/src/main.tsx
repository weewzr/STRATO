import '@fontsource/literata/latin-400.css';
import '@fontsource/literata/latin-400-italic.css';
import '@fontsource/literata/latin-600.css';
import '@fontsource/ibm-plex-sans/latin-400.css';
import '@fontsource/ibm-plex-sans/latin-500.css';
import '@fontsource/ibm-plex-sans/latin-600.css';
import '@fontsource/ibm-plex-mono/latin-400.css';
import './design/tokens.css';
import './design/app.css';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { installCapture, runSelftest } from './selftest/selftest';
import { startSolver } from './solver/session';

const selftest = new URLSearchParams(window.location.search).get('selftest') === '1';
if (selftest) installCapture();

// The Rust solver in a Web Worker (web phase 3); the page runs the inputs once it reports ready.
startSolver();

const root = document.getElementById('root');
if (!root) throw new Error('missing #root');
createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);

if (selftest) {
  // Let the first render and the fonts settle, then run the checks.
  window.addEventListener('load', () => {
    void document.fonts.ready.then(() => runSelftest());
  });
}
