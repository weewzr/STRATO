#!/usr/bin/env sh
# Precomputed results of the web demo (prompt/TECHNICAL_ARCHITECTURE.md Section 5): runs the
# native STRATO release binary and the native wasm wrapper on the synthetic case and writes
# web/public/data/*.json and *.csv with a PRECOMPUTED provenance header. Pass --check to
# regenerate in memory and compare with the committed files (CI). Run from the repository root.
set -eu
exec python3 -I -X utf8 scripts/web/generate_data.py precomputed "$@"
