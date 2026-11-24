// The web build needs the WebAssembly wrapper that scripts/web/build_wasm.py writes into
// src/solver/pkg (git-ignored: it is built from crates/strato-wasm, never committed). Fail early,
// with the command to run, when it is missing.
import { existsSync } from 'node:fs';

const need = ['src/solver/pkg/strato_wasm.js', 'src/solver/pkg/strato_wasm_bg.wasm', 'src/solver/pkg/strato_wasm.d.ts'];
const missing = need.filter((p) => !existsSync(new URL(`../${p}`, import.meta.url)));
if (missing.length > 0) {
  console.error(`FAIL the WebAssembly wrapper is not built (missing ${missing.join(', ')}).`);
  console.error('     Run from the repository root: python -I -X utf8 scripts/web/build_wasm.py');
  process.exit(1);
}
console.log('PASS the WebAssembly wrapper is built (src/solver/pkg)');
