# ADR-0003 — The dev and fuzz allowlist tiers: policy for non-shipped dependencies

**Status:** Accepted
**Date:** 2026-07-23
**Bead:** fm-1o8 (ruling requested by fm-ntp)
**Amends:** none — executes the "separate policies" clause D1 (§3) already
reserves for the runtime / ffi / build / dev / fuzz classes.

## Context

fm-ntp (cargo-fuzz harnesses for the fmn-codec parsers, an R14 item) is
blocked on a ruling: `libfuzzer-sys` and the `cargo-fuzz` toolchain are
outside the FrankenSuite, and D1 forbids new unreviewed dependencies in
authoritative crates. `SUITE_ALLOWLIST.tsv` already *defines* `dev` and
`fuzz` classes in its schema, but no policy states what those classes admit
or what constraints bind them. Without the ruling, every future
dev-dependency question (fuzzing today; criterion-class benching or
property-testing helpers tomorrow) becomes an ad-hoc negotiation.

## Decision

D1's governed closure binds the **shipped** dependency graph. The classes
split as follows:

1. **`runtime` / `ffi` / `build`** — the shipped closure. Unchanged: suite
   crates plus the pre-authorized exceptions (PyO3, clap, wasm-bindgen),
   every transitive package allowlisted, CI failing on any unlisted
   package.
2. **`dev`** — dev-dependencies of workspace crates. Admitted only when
   (a) they cannot influence shipped artifacts (dev-deps never enter
   release builds by Cargo's own rules), (b) they are pinned by exact
   version + checksum in `SUITE_ALLOWLIST.tsv` like any other row, and
   (c) tests that gate merges do not silently depend on network or
   platform services through them. The bar stays high: the default answer
   to "may I add a dev-dep?" remains **no** — std-based test code first.
3. **`fuzz`** — the fuzzing toolchain (`libfuzzer-sys`, the `fuzz/`
   harness crate cargo-fuzz generates). Admitted under class=`fuzz` with
   these constraints: the `fuzz/` crate is **not a workspace member** (it
   never enters the workspace build graph or any shipped artifact); it may
   depend only on the crate under test plus `libfuzzer-sys`; its packages
   carry allowlist rows (pinned version, checksum, unsafe-audit status
   noted as expansion-tolerated — libfuzzer-sys wraps a C runtime, which
   is acceptable *only* because it is unreachable from any shipped
   artifact); and fuzz targets run as scheduled CI jobs, never as merge
   gates.

The `#![forbid(unsafe_code)]` posture (D3) is unaffected: authoritative
crate roots keep the forbid; the fuzz harness crate is tooling, not an
authoritative crate.

## Consequences

fm-ntp is unblocked: it may scaffold `fuzz/` with cargo-fuzz, add the
class=`fuzz` allowlist rows, and wire the scheduled CI job. The
`governed_closure.rs` check needs one refinement when fm-ntp lands: it
must continue to fail on unlisted packages in the **workspace's**
Cargo.lock while the fuzz crate maintains its own lockfile with its own
allowlist rows (the fuzz crate's lock is checked by the same test walking
`fuzz/Cargo.lock` when present). Future dev-dependency requests cite this
ADR and add rows — they do not reopen the policy.
