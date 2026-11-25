// Typesets the mathematics of a rendered STRATO document with the KaTeX copy served beside it
// (crates/strato-docsite). The page holds each formula as TeX in a span with the class
// math-inline or math-display; this script only hands that text to katex.render. A formula KaTeX
// cannot read stays visible as TeX, is marked, and is reported on the console.
(function () {
  'use strict';
  function run() {
    var nodes = document.querySelectorAll('.math-inline, .math-display');
    var failed = 0;
    for (var i = 0; i < nodes.length; i++) {
      var el = nodes[i];
      var tex = el.textContent || '';
      try {
        window.katex.render(tex, el, {
          displayMode: el.classList.contains('math-display'),
          throwOnError: true,
          strict: 'ignore',
          output: 'htmlAndMathml',
        });
      } catch (e) {
        failed++;
        el.classList.add('math-error');
        el.setAttribute('title', String((e && e.message) || e));
        console.error('KaTeX could not typeset a formula: ' + ((e && e.message) || e));
      }
    }
    document.documentElement.setAttribute('data-math-total', String(nodes.length));
    document.documentElement.setAttribute('data-math-failed', String(failed));
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', run);
  else run();
})();
