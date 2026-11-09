//! The Gauntlet: Parity Ledger, correctness oracles, self-goldens, Look Gallery tooling, perf gates (§16).
//!
//! Landed so far:
//! - the governed-closure audit (D1, fm-g2c) — see [`closure`] and
//!   `tests/governed_closure.rs`, the CI teeth of SUITE.lock +
//!   SUITE_ALLOWLIST.tsv;
//! - the Gauntlet bootstrap (fm-xb3): the self-golden rig ([`golden`],
//!   D-16 — bit-locked artifacts, per-platform lock files, the
//!   `UPDATE_GOLDENS=1` bless flow, `.actual` drift sidecars), the tolerance
//!   doctrine as reusable checks ([`tolerance`], §16.4), and the `.npy`
//!   fixture-interchange subset ([`npy`], §16.3) the Reference-fixture
//!   scripts emit.
//!
//! - the public coverage ratchet (§11.5, fm-mol) — [`ratchet`] and
//!   `tests/coverage_ratchet.rs`: the four public numbers against G0-4's
//!   frozen denominator, monotone by CI, pin-coupled so a SUITE.lock bump
//!   of franken_markdown without a ratchet re-run fails.
//!
//! - the one API schema (§16.2, fm-vn6) — [`schema`] and
//!   `tests/api_schema.rs`: the extracted Reference surface
//!   (`API_SCHEMA.tsv`) merged with the authored rulings
//!   (`API_OVERLAY.tsv`), and the generators that turn the merge into
//!   `fmn-config`'s typed extraction, the Parity Ledger rows, the CLI flag
//!   table, and the docs. Drift between a generated artifact and the schema
//!   fails the build.
//!
//! - the performance-gate decision core (§17.2, fm-inr) — [`perf`]:
//!   comparable-run identities, raw valid/invalid repetitions, robust
//!   median/MAD summaries, versioned observed baselines, and explicit
//!   pass/alert/block/inconclusive verdicts. It never turns an unpinned or
//!   unobserved run into passing PG evidence.
//! - the pinned-host authority (§17.2, fm-inr.1) — [`perf_host`]: strict host
//!   profiles, compiled compiler/suite identity, live Linux bare-metal and
//!   isolation checks, and an opaque token that current producers require
//!   before recording a qualified measurement.
//! - the canonical PG-2 producer (fm-inr.2.1) — [`perf_pg2`]: fixed fill and
//!   stroke fixtures, exact benchmark/configuration digests, bounded raw
//!   samples, and a content-addressed stage trace over the real Lumen path.
//! - the canonical PG-5 per-commit producer (fm-inr.3.2) — [`perf_pg5`]: the
//!   complete certified corpus at `{1,4,16}` requested threads plus real
//!   frame-parallel and ordered-emitter schedule lanes, expressed as three
//!   exact mismatch-count samples.
//! - the canonical PG-6 allocation producer (fm-inr.3.1) — [`perf_pg6`]: one
//!   warm frame plus one arena/output-buffer reuse frame for every committed
//!   scene-golden case, with exact engine-owned allocation samples and retained
//!   frame identities.
//! - the canonical PG-6 peak-residency producer (fm-0q0g) —
//!   [`perf_pg6_peak`]: eleven real passes over a certified three-view UHD 3D
//!   gallery through the public library/renderer path, with concurrent
//!   resident-set sampling and no duplicated pseudo-samples.
//! - the canonical PG-7 producer (fm-inr.2.2) — [`perf_pg7`]: cold formula,
//!   cache-proven formula-hit, and exact 10,000-glyph native-text fixtures
//!   with result self-goldens and bounded integer latency samples.
//! - the canonical PG-8 producer (fm-zoi) — [`perf_pg8`]: the Python
//!   boundary's four scene classes — native built-ins against a pure-Rust
//!   twin, per-frame and point-transform callbacks, dynamic subclasses —
//!   with bridge-driven workload identity and locked state self-goldens.
//!
//! - self-goldens at scale (§16.3 plane 2, fm-t1v) — [`scene_goldens`] and
//!   `tests/scene_goldens.rs`: the ~25-scene primitive-and-feature corpus
//!   over the landed class families, bit-locked at the post-construct and
//!   post-transform lifecycle points through the D-16 rig.
//! - the engine-equivalence suite (§16.3 plane 4, fm-t1v) — [`equivalence`]
//!   and `tests/engine_equivalence.rs`: certified CPU vs fast CPU under the
//!   versioned v1 visual budget (max/RMS channel error + luma SSIM), the
//!   merge blocker for engine arithmetic changes.
//! - the correctness-oracle corpus (§16.3 plane 1, fm-t1v) — [`oracles`]
//!   and `tests/oracles.rs`: analytic ground truths, metamorphic laws with
//!   documented domains, and the sha256-verified structural-fixture corpus.
//! - the Look Gallery (§16.3 plane 3, fm-t1v) — [`gallery`] and
//!   `tests/look_gallery.rs`: SSIM, symmetric chamfer edge-distance, and
//!   local-error percentiles feeding the human verdict workflow — smoke
//!   alarms, never hard gates.
//! - the deterministic fuzzing campaign (§16.5 plane 4, fm-t1v) — [`fuzz`]
//!   and `tests/fuzz_campaign.rs`: seeded structure-aware targets with
//!   resource budgets, corpus persistence, and the CI/scheduled two-count
//!   gate.
//! - the scripted end-to-end scenario harness (§16.6, W10, fm-fjq) —
//!   [`e2e`] and `tests/e2e_harness.rs`: checked-in Rust scenarios across
//!   the render-matrix/determinism/failure-path/lifecycle/parity classes,
//!   driven through live surfaces (Rust API, in-process CLI), asserting
//!   through the D-16 golden rig and the `LogExpect` log DSL, capturing a
//!   deterministic NDJSON run log per scenario, auto-bundling an FMNA
//!   repro on any failure, and proving the failure machinery itself with
//!   regression drills. Scenarios register as data — no harness edits.
//! - the distribution bundle gates (W11, fm-aef) — [`font_bundle`] and
//!   `tests/font_bundle.rs`: the committed `dist/FONT_BUNDLE.json`
//!   regenerated byte-for-byte from the actual bundled faces (a font
//!   change without regeneration is a CI block) and the license
//!   completeness check (every face's OFL text shipped; the engine's
//!   MIT+rider license in the bundle; nothing unlicensed); [`corpus_leak`]
//!   and `tests/corpus_leak.rs`: the no-corpus-leak release gate (§15.3).
//!
//! The rest of the Gauntlet lands with its owning workstreams; see
//! COMPREHENSIVE_PLAN §19 for the crate map.
// corpus_leak's Windows file identity (volume serial + file index) is the
// pinned nightly's `windows_by_handle` API, exactly as fmn-output uses it.
#![cfg_attr(windows, feature(windows_by_handle))]
#![forbid(unsafe_code)]

pub mod closure;
pub mod corpus_leak;
pub mod e2e;
pub mod equivalence;
pub mod font_bundle;
pub mod fuzz;
pub mod gallery;
pub mod golden;
pub mod npy;
pub mod oracles;
pub mod perf;
pub mod perf_frontdoor;
pub mod perf_host;
pub mod perf_pg2;
pub mod perf_pg5;
pub mod perf_pg6;
pub mod perf_pg6_peak;
pub mod perf_pg7;
pub mod perf_pg8;
pub mod ratchet;
pub mod scene_goldens;
pub mod schema;
pub mod semantic;
pub mod tolerance;
