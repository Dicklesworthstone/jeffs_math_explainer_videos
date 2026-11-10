#!/usr/bin/env bash
# W5 wasm tier 1 (fm-l97): build the smoke probe for wasm32 and drive it
# headlessly in node (bun-as-node is fine). This is the bead's recorded
# headless gate: it proves the foundation crates execute in a real wasm VM
# and that the single-thread render is deterministic there.
set -euo pipefail
cd "$(dirname "$0")"

cargo build --locked --release --target wasm32-unknown-unknown

# CARGO_TARGET_DIR may redirect the artifact; honor it, else the crate-local
# target dir (this crate is a non-member, so its default target is its own).
BASE="${CARGO_TARGET_DIR:-target}"
WASM="$BASE/wasm32-unknown-unknown/release/fmn_wasm_smoke.wasm"
[[ -f "$WASM" ]] || { echo "wasm-smoke: artifact missing at $WASM" >&2; exit 1; }

node run.mjs "$WASM"
