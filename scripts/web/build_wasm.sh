#!/usr/bin/env sh
# Build and check the STRATO WebAssembly wrapper (prompt/TECHNICAL_ARCHITECTURE.md Section 3).
# The work is done by build_wasm.py, which also runs on Windows; this wrapper keeps the name
# the architecture gives the step. Run from the repository root.
set -eu
exec python3 -I -X utf8 scripts/web/build_wasm.py "$@"
