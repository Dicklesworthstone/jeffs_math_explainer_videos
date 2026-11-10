# BN-13 — Cubic curves keep a measured error bound

**Status:** Final. Landed in W2 (fm-6cf). G1 (fm-o3j) passed on 2026-08-20; the
migration guidance below was checked against the code on 2026-10-04: the
0.1 px / `0.1 / 135` scene-unit default (`fmn-geom/src/cubic.rs`), the no-op
`set_use_simple_quadratic_approx`, `add_cubic_bezier_curve_to_with_tolerance`,
and the fallible `CubicBezier` conversions with no infallible `From`.

## What changed

Classic manim reduces an API cubic to two quadratics with no quality bound.
When `use_simple_quadratic_approx` is enabled, a shallow cubic may instead
take a one-quadratic shortcut. The selected mechanism changes point counts
and can move the curve by several output pixels.

FrankenManim has one cubic-to-quadratic converter for API cubics, smoothing,
and future SVG ingestion. It emits a C1 quadratic spline and proves the
parameter-matched deviation of every piece from the source cubic using a
Bernstein convex-hull bound. The default is **0.1 output pixel**, expressed
as `0.1 / 135` scene units at the canonical 1920-pixel frame. Callers that
know another render scale can supply an explicit scene-unit tolerance.
TrueType outlines are already quadratic and pass through without loss.

The `use_simple_quadratic_approx` compatibility setting is accepted, but it
cannot bypass the bound. The audited converter emits one quadratic for
effectively quadratic input when that preserves both the bound and
shared-anchor encoding; otherwise fidelity wins over the old shortcut.

## Migration guidance

- Cubic point counts are adaptive. Code should treat the first and last
  points as the API anchors and should not assume that every cubic has five
  stored points.
- A scene that enabled `use_simple_quadratic_approx` may gain points and a
  more faithful curve. Remove point-count assumptions rather than attempting
  to restore the unbounded shortcut.
- Rust importers can call
  `QuadPath::add_cubic_bezier_curve_to_with_tolerance` when their output scale
  differs from the canonical frame.
- Rust `CubicBezier::build` and its `TryFrom<CubicBezier>` conversions are
  fallible. Handle `GeomError` explicitly; there is no infallible
  `From<CubicBezier> for Mobject`, because a rejected conversion must not
  publish a point-like partial object.
- The Reference's old two-quad values remain available only as conformance
  observations; they are not production output goldens.

## Why

G0-2 measured the Reference splitter at up to 7.71 pixels of deviation on its
look-study corpus. The 0.1-pixel default is 77 times tighter and sits well
inside the analytic antialiasing transition. One bound across every cubic
source makes curve quality a system property instead of a call-site accident.

## Evidence

- `crates/fmn-geom/src/cubic.rs`: the construction, proof, deterministic
  subdivision, degenerate handling, property tests, and bit-locked fixture
  corpus.
- `crates/fmn-geom/tests/invariant_fixtures.rs`: QuadPath routing, stationary
  endpoint encoding, compatibility-knob behavior, and smoothing invariants.
- `crates/fmn-library/src/poly.rs`: the fallible `CubicBezier` builder,
  `TryFrom` surface, and non-finite/resource-guard refusal regressions.
- `docs/g0/G0-2-look-study-ratification.md`, finding L8 and decision (f).
- `crates/fmn-geom/benches/cubic_converter.rs`: ingestion-path microbenchmark.
