// The Plotly basic partial bundle ships no type declarations (docs/development/visualisation_library_comparison.md,
// Section 3.2). plots/plotly.ts types the few functions the site uses.
declare module 'plotly.js-basic-dist-min' {
  const Plotly: unknown;
  export default Plotly;
}
