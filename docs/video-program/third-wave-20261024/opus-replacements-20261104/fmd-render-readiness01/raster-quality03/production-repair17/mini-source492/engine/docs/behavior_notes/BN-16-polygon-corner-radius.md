# BN-16 — Polygon corner rounding sees every edge

**Status:** Draft. Landed in W10 (fm-5wq.4.16); becomes Final when the
Python-portal gate passes.

## What changed

Classic manim's default `Polygon.round_corners()` radius is intended to be a
quarter of the shortest non-degenerate polygon edge. Its implementation zips
the public vertex list with `vertices[1:]`, however, so it never measures the
closing edge from the last vertex back to the first. The result depends on
which cyclic rotation of the same vertices the caller supplied.

FrankenManim measures the complete cyclic edge set. The default is therefore
one quarter of the actual shortest non-degenerate edge, independent of which
vertex happens to be listed first. An explicit radius follows the familiar
construction unchanged, including negative radii for concave corners.

If every edge is degenerate, FrankenManim keeps the polygon finite and leaves
its corner topology unrounded. The Reference instead lets `min()` raise after
filtering away every edge.

## Migration guidance

- Most polygons are unchanged: if the closing edge is not uniquely shortest,
  both engines choose the same default radius.
- Code relying on the omitted-edge accident can pass its former radius
  explicitly: `polygon.round_corners(radius=old_value)`.
- Prefer an explicit radius whenever the exact corner size is part of a scene's
  design. The no-argument form deliberately describes a geometry-derived
  default, not a stable value tied to vertex ordering.
- Code should not rely on all-coincident vertices raising from `min()`; inspect
  or reject degenerate input explicitly when that is an application error.

## Evidence

- `crates/fmn-library/src/poly.rs`: cyclic shortest-edge selection, finite
  all-coincident fallback, and explicit-radius corner construction.
- `crates/fmn-library/src/poly.rs` tests
  `round_corners_includes_the_closing_edge_in_its_default_radius` and
  `round_corners_keeps_an_all_coincident_polygon_finite`.
- `crates/fmn-python/tests/bridge.py`: the native construction and mutation are
  exercised through detached and scene-bound Python objects.
- Pinned Reference:
  `scripts/manim_ref/manimlib/mobject/geometry.py::Polygon.round_corners` at
  commit `6199a00d4c1b1127ebe45cb629c3f22538b10e13`.
