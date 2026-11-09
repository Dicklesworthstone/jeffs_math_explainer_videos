#!/usr/bin/env bash
# G0-5 spike driver (fm-87q): build the prototype bridge, expose it as an
# importable module, run the extensibility suite and the crossing-cost
# bench. Repeatable: everything is pinned by this crate's own Cargo.lock
# and the workspace's rust-toolchain.toml.
#
# The build runs locally by design (RCH_CARGO_WRAPPER_BYPASS): the cdylib
# links the host CPython, which remote build workers do not carry.
set -euo pipefail
cd "$(dirname "$0")"

echo "==> cargo build --release (local extension: host CPython ABI, no libpython link)"
PYO3_BUILD_EXTENSION_MODULE=1 RCH_CARGO_WRAPPER_BYPASS=1 \
  cargo build --release --target-dir target

echo "==> exposing fmn_spike_bridge on sys.path"
ln -sf ../target/release/libfmn_spike_bridge.so py/fmn_spike_bridge.so

echo "==> the extensibility suite"
(cd py && python3 test_extensibility.py)

echo "==> crossing-cost bench (PG-8 seed numbers)"
(cd py && python3 bench_crossing.py)

echo "OK: G0-5 spike green"
