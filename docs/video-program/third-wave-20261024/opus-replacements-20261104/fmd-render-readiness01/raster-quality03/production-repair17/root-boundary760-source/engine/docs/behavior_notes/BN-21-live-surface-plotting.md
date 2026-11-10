# BN-21 — Live surfaces retain the axes chart; wireframes refresh in place

**Status:** Draft · **Workstream:** W7/W10 · **Related:** fm-5wq.4.143

## Visible difference

`ThreeDAxes.get_parametric_surface(f)` maps every authored coordinate triple
through the axes' actual `c2p` chart. Rotation, reflection, nonuniform scaling,
and shearing therefore affect the plot just as they affect the axes. The old
world-axis stretches retained only axis lengths and silently lost orientation.
`get_graph(f)` uses the same route for the coordinate triple `(u, v, f(u, v))`.
Its omitted domains come from the axes' x/y ranges, and explicit NumPy ranges
are accepted without ambiguous array truth tests.

A plot retains this chart as part of its recipe. Calling `init_points()` samples
the current function and chart, rather than rebuilding an untransformed surface
at the world origin. Atlas calculates finite-difference normal-control points
from the mapped samples, so they describe the transformed surface. This differs
from stretching a precomputed normal as though every affine transform preserved
angles. The existing native sampler, record buffer and renderer remain the
owners of all geometry and pixels.

Ordinary affine axes are frozen for each rebuild using their origin and three
basis vectors. Callback changes to those axes or their mapping methods refuse
publication of the inconsistent candidate. A custom `c2p` or `coords_to_point`
override is dispatched at the actual sampled coordinates, not approximated by
an affine basis; this permits genuinely nonlinear charts. Undetectable changes
to external state inside a custom chart are still the author's responsibility.
Arbitrary callback side effects are not rolled back.

## Authoring live plots

```python
axes = ThreeDAxes().rotate(PI / 6, axis=RIGHT)
height = ValueTracker(0)
surface = axes.get_graph(
    lambda u, v: height.get_value() + 0.2 * u * v,
    u_range=(-2, 2), v_range=(-2, 2), resolution=(13, 11),
)
surface.add_updater(lambda obj: obj.init_points(), call=False)
mesh = SurfaceMesh(surface, resolution=(7, 5), stroke_width=2)
mesh.add_updater(lambda obj: obj.init_points(), call=False)
self.add(axes, surface, mesh)
self.play(height.animate.set_value(1), run_time=2)
```

Keep the surface before its wireframe in scene order. Neither object updates
implicitly: explicit rebuilds or normal scene updaters choose when to publish.
An `init_points()` pass follows the recipe, not additional manual placement
applied only to the old plot or mesh. Move the axes when subsequent plot
rebuilds should follow that placement. Copy and deepcopy retain the external
axes binding like a Python closure; pickle follows normal memo semantics when
the authored function itself is picklable.

A surface's UV topology remains fixed during regeneration. Use the native
UV-aware Transform or a deliberate `become` for topology changes. Changing an
axes range does not implicitly overwrite the plot's stored `u_range`/`v_range`.
Set those domains explicitly when the desired sampling domain changes.

## Refreshing wireframes

`SurfaceMesh.init_points()` refreshes generated curves from the source's current
native records and public normal accessor; it never re-evaluates the UV recipe.
The earlier portal no-op left overlays frozen, while the Reference's repeated
initialization appended another set of lines. FrankenManim updates its generated
wire block without accumulating duplicates. Surviving per-axis wire identities,
record styles, live views, updaters, annotations and scene ownership follow the
existing live-mesh contract. See `docs/LIVE_SURFACE_MESH.md` for the exact
ownership rules when wire density changes. The plotting implementation reuses
that mesher unchanged.

## Bounded input and scope

The public `ParametricSurface` constructor and 3D plotting helpers now preflight
integer grid dimensions and the 65,536-point authoring budget **before** the
native sampler allocates its arrays. Excessive requests previously reached a
process-aborting allocation rather than a Python exception. Fractional dimensions
are refused instead of silently truncated. Range endpoints must be finite;
`epsilon` must be positive and finite, and `normal_nudge` nonnegative and finite.
A graph function returns a finite real scalar; a chart's parametric function
returns exactly three real coordinates whose mapped world values fit float32.

This budget enforcement is at the public host entry points. Direct Rust surface
builders and calls to private native allocation seams are not newly bounded by
this change. Empty/singleton construction remains supported, while live
regeneration still requires an actual grid with both dimensions at least two.
This work does not add arbitrary indexed-mesh remeshing, texture-image crossfades,
or automatic callback certification.

Plotting acceptance uses `tests/surface_plotting.py`; mesh coverage remains in
`tests/live_surface_mesh.py` and `render_matrix.python_surface_mesh.v1`. Independent analytic
coordinate and PNG/Y4M oracles exercise the real installed native engine.
