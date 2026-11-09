# ADR-0004 — OQ-3 resolved: fmd workspace conversion, fmd-font versioning, and the consumption route

**Status:** Accepted
**Date:** 2026-07-23
**Bead:** fm-ydw
**Amends:** resolves OQ-3 (fmd workspace-conversion mechanics and
release/versioning for the new crates; owner W6)

## Context

fmd-font must exist as a crate franken_manim can pin and consume, but
franken_markdown was a single-package repo whose engine philosophy (zero
third-party dependencies, WASM-clean core, forbid-unsafe) predates the
factoring. OQ-3 asked how the workspace conversion happens without
breaking fmd's build, how the new crates are versioned/released, and how
FrankenManim consumes them.

## Decision

1. **Conversion mechanics: root package + member, not a restructure.** The
   root `Cargo.toml` gains a `[workspace]` with
   `members = [".", "fmd-font"]` and matching `default-members`, so plain
   `cargo test`/`check` in the repo root — fmd's own AGENTS.md gate —
   covers both crates. `src/text.rs` moves wholesale (git mv) to
   `fmd-font/src/lib.rs`; fmd re-exports it as `pub use fmd_font as text;`,
   which keeps `crate::text::…` (all five internal consumers) and the
   public `franken_markdown::text::…` surface compiling unchanged. The
   glyf outline decoder lands as `fmd-font/src/outline.rs` — new
   functionality, new module.
2. **Versioning: fmd-font starts at 0.1.0** and rides franken_markdown's
   repo and release cadence (same tree, same tag). It inherits the engine
   constraints: zero dependencies, `#![forbid(unsafe_code)]`, the deny
   set (`unwrap`/`expect`/`panic`/`todo`), edition 2024, and the
   `--no-default-features` WASM-core gate. fmd-math will follow the same
   pattern as a sibling member when its bead lands.
3. **Consumption route: git dependency at the SUITE.lock rev.** Following
   the established suite precedent (frankenmermaid → franken_networkx),
   franken_manim consumes
   `fmd-font = { git = "…/franken_markdown", rev = <SUITE.lock pin> }`.
   The lock's `franken_markdown` row is the single authority; the
   governed-closure check verifies the Cargo dependency's rev equals the
   pinned commit, and upgrades ride the SUITE.lock ritual
   (`docs/GOVERNANCE.md` §6). No sibling-checkout path dependencies — they
   would break remote/CI builds that only materialize this repo.

## Consequences

fmd stays green through the conversion (its whole test suite and WASM
gate run unchanged); fmd-font is independently testable and fuzzable;
franken_manim's first foundation dependency enters through the same
pinned-git mechanism every later suite crate (fnp, fsci, fmd-math) will
use, so the governed_closure admission logic gets its git-dependency
handling now rather than under G2 pressure. The plan's §23 OQ-3 entry is
trued up to point here in the same commit.
