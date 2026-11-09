#!/usr/bin/env bash
# PG-5 thread-count determinism harness (fm-sol; §16.7 declares thread count
# outside the input closure — "proven inert under §10.5" — and this is the
# standing proof).
#
# What runs:
#   1. The scene-corpus thread-invariance gate
#      (fmn-conformance --test scene_goldens) at FMN_PG5_THREAD_COUNTS,
#      comma-separated and always starting at the 1-thread baseline every
#      other count is compared against. Default: 1,4,16 — the per-commit
#      PG-5 sweep.
#   2. The certified-engine corpus sweep (fmn-conformance --test
#      certified_engine::every_locked_frame_is_thread_count_invariant),
#      which holds its locked frames byte-exact at its own fixed {1,4,16}.
#
# Cadence (R22 budget, docs/dist/ci_coverage.md):
#   - per commit:  FMN_PG5_THREAD_COUNTS=1,4,16  (the default)
#   - weekly:      FMN_PG5_THREAD_COUNTS=1,32,96 — the high-core cadence,
#                  env-gated through the variable alone; no code path
#                  changes between the two profiles.
#
# The tests are pure render-and-compare: no wall-clock assertions, so the
# lane is deterministic on any runner size.
set -euo pipefail
cd "$(dirname "$0")/.."

: "${FMN_PG5_THREAD_COUNTS:=1,4,16}"
export FMN_PG5_THREAD_COUNTS

echo "==> PG-5 scene-corpus thread invariance at {${FMN_PG5_THREAD_COUNTS}} threads"
cargo test -p fmn-conformance --test scene_goldens every_certified_frame_is_thread_count_invariant

echo "==> PG-5 certified-engine corpus sweep (fixed {1,4,16})"
cargo test -p fmn-conformance --test certified_engine every_locked_frame_is_thread_count_invariant

echo "OK: PG-5 thread-count determinism holds at {${FMN_PG5_THREAD_COUNTS}}"
