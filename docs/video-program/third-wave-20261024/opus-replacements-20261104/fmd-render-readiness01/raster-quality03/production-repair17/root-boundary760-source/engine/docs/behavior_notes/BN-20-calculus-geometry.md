# BN-20 — Calculus helpers operate in the axes' coordinate system

**Status:** Draft. **Owner:** W7/W10. **Related work:** fm-5wq.4.143.

## Riemann rectangles

`CoordinateSystem.get_riemann_rectangles` now constructs native Rectangle paths
from data-coordinate corners. Rotating, stretching, translating, or tilting an
Axes no longer collapses the rectangles or changes which samples are negative.
Rectangle identity, native records, styles, family ownership and rendering are
preserved; there is no alternative path or rasterization engine.

The integration interval is authoritative. A final partial bin ends at the
requested upper bound; right and center sampling use that bin's actual bounds.
A three-value `x_range` supplies its own step; otherwise `dx` (or the axes'
default step) supplies it. Caller-owned ranges, including NumPy arrays, are not
modified. A zero-length interval yields an empty VGroup.

`show_signed_area=False` leaves the ordinary gradient on negative rectangles;
it does not reflect them above the baseline. With the default `True`, negative
samples receive `negative_color`. Zero-height rectangles are nonnegative.

The Reference at `6199a00d4c1b1127ebe45cb629c3f22538b10e13` instead measures
width and sign using world-space components, extends the sampling range, and
does not act on `show_signed_area`. Its sign is
`i2gp(sample, graph)[1] > c2p(sample, 0)[1]`, a curve-proportion lookup. Near a
zero crossing that lookup can land on the other side of the axis:
`_2022/borwein/main.py` `Graph15Case` gets `negative_color` on two rectangles
that the data-coordinate sample makes positive. The corpus differential
excludes them by the `bn20-riemann-sign` row. These are intentional corrections under
the plan's correctness-first contract, not pixel-compatibility promises.

## Migration and bounds

Remove workarounds that unrotate axes, extend the upper bound manually, or
recolor negative bins when signed coloring is disabled. Code that intentionally
needs an overshooting last bin should request that larger interval explicitly.

Ranges must be finite and nondecreasing, with a positive finite step. Requests
are limited to 65,536 rectangles and reject steps that cannot advance the
floating-point coordinate. Sampling mode and style inputs are validated before
evaluating authored graph callbacks. A sample in a geometric graph's missing
interval raises a ValueError instead of inventing a bridge across the gap.
No rollback of effects performed inside authored callbacks is promised.

## Integral regions follow the drawn graph

`CoordinateSystem.get_area_under_graph` clips the current quadratic geometry at
its actual data-coordinate bounds. Nonuniform sampling and nonlinear curve
parameters no longer shift an integration boundary. Reversed paths and plain
VMobjects work without `x_range` or `underlying_function` metadata. Each
continuous component closes to its own baseline; discontinuity gaps remain
unfilled. Requests outside the drawn domain are intersected with that domain,
not extrapolated. Empty intersections and zero-width intervals are empty paths.

The result is a detached static VMobject, with fill and projection styles but
without the source's callbacks, analytic metadata or child objects. Source live
views, records and scene membership are not changed. An analytic function may
have changed since the last scene tick: the *drawn geometry*, not a fresh call
to that function, defines this area. Use `always_redraw` or an updater to obtain
a live region, as shown in `demo/python/integral_regions.py`.

This replaces the Reference's assumption that a data-coordinate fraction is a
curve-index fraction and its single baseline closure across all components.
Clipping delegates to the native partial-reveal geometry owner, preserving
quadratic handles; it is not a second sampling or rendering implementation.

The supported region is a graph with x-monotone quadratic subpaths in the axes'
affine chart. Foldbacks are rejected explicitly rather than choosing one
branch. Bounds must contain exactly two finite, nondecreasing endpoints.
Input and output paths share the live-graph record budget. General multi-valued
parametric regions and nonlinear coordinate charts are not promised by this
helper.

## Evidence

`crates/fmn-python/tests/live_graphing.py` exercises native transformed geometry,
negative and zero samples, clipped bins, all sampling modes, styles, invalid
requests, disconnected paths and a rendered reference-polygon comparison.
`crates/fmn-python/tests/calculus_area.py` adds analytic quadratic endpoint and
handle checks, reversed/nonuniform paths, declared poles, source-view ownership,
projection styles and tracker-driven region playback. Independently constructed
native polygons verify that disconnected fills have no invented bridge.
Y4M comparisons run at one and four threads. These witnesses do not close the
wider compatibility or certified-rendering gates.

The native Rust `Axes::get_riemann_rectangles` follows the same rules — the same
bin formula as the portal's `_riemann_grid`, chart corners, `show_signed_area` —
witnessed by `riemann_bins_follow_bn20` in `crates/fmn-library/src/coords.rs`.
Both front doors default to the pinned Reference's `colors=(BLUE, GREEN)`, and
the portal's `get_area_under_graph` to its `fill_color=BLUE`
(`test_default_colors_are_the_references` in `live_graphing.py`). The native
`get_area_under_graph` still cuts by curve index; it has not taken this note's
data-coordinate clipping yet.
