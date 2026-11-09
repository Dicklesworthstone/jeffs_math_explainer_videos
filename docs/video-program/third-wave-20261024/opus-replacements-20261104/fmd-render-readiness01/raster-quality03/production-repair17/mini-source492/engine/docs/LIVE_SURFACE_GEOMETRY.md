# Live native surface geometry

`Surface.init_points()` now resamples the current fixed-topology UV recipe
through Atlas. It updates both vertex positions and normal-control points in
the existing native record buffer. The Surface proxy, bound Scene, children,
updaters, material, textures and already-exported record views keep their owners.

```python
from manimlib import ParametricSurface, Scene, ValueTracker, linear

class LiveSurface(Scene):
    def construct(self):
        amplitude = ValueTracker(0)
        surface = ParametricSurface(
            lambda u, v: (u, v, amplitude.get_value() * u * v),
            u_range=(-2, 2), v_range=(-2, 2), resolution=(21, 21),
        )
        surface.add_updater(lambda current: current.init_points(), call=False)
        self.add(surface)
        self.play(amplitude.animate.set_value(.8), run_time=2, rate_func=linear)
```

A complete example is `demo/python/live_surface_geometry.py`. Programmatic
export uses the same native publication owner as any other scene:

```python
from fmn_python import render_scene
result = render_scene(LiveSurface, "surface.y4m", format="y4m",
                      resolution=(640, 360), fps=30)
```

## Shape-specific reconstruction

Unmodified Sphere, Torus, Cylinder, Cone, Disk3D and Square3D UV methods use their
specialized native constructors for the prepared geometry. Changing radius,
height, axis or side length therefore does not accidentally regenerate a unit
solid. Sphere retains its analytic-normal and clockwise controls. A subclass's
UV-method override instead executes through the normal native parametric sampler.
Cylinder-derived Line3D uses its stored cylinder dimensions and axis; this is not
an endpoint-editing operation.

For TexturedSurface, refresh its source Surface first, then call the textured
surface's `init_points()`. Geometry is copied from the source's current world
coordinates. Its existing UV coordinates, per-vertex opacity, light/dark image
resources and material are not replaced or decoded again.

## Ownership and failure rules

The native UV dimensions must stay unchanged (both axes at least two, at most
65,536 samples total). Invalid bounds, non-finite samples, inconsistent grids,
recursive regeneration and rebuilding a surface held by its own active animation
are refused. A failed callback publishes no prepared geometry. Callback effects
on external objects are real effects, not transactions; callback mutations to the
destination are detected and preserved instead of being overwritten.

Like an initializer, procedural regeneration evaluates the recipe in its own
coordinates. Previous shifts/rotations/scales of that surface are not reapplied.
Put a desired placement into the UV function or apply it after regeneration.
An initializer returns `None`; updater callbacks can call it directly.

## Topology replacement and interpolation

Use `surface.become(new_surface)` for an immediate topology replacement. The
existing native replacement now also synchronizes Python UV dimensions, domains
and triangle indices, including Surface children inside groups. A subsequent
`uv_to_point()` no longer reshapes new records using stale dimensions. Other
arbitrary Python attributes and procedural recipes retain normal `become`
semantics; this operation does not replace the receiver's authored callback.

Animated transforms between unequal UV grids still require topology-aware
alignment and are not implemented by this change. Fixed-topology native
interpolation now keeps every declared vec3 pointlike field in the same space,
including locked normal-control fields when a live view requires world-space
records. Reading a retained surface must not change its lighting or compound a
later animation sample.

## Acceptance

`crates/fmn-mobject/tests/pointlike_transforms.rs` and
`crates/fmn-anim/tests/surface_transforms.rs` exercise native placement, field
views, normal seeds, custom anchors and copy-on-write snapshots.
`crates/fmn-python/tests/surface_pointlikes.py` checks real native normals and
pixel-stable readback. `crates/fmn-python/tests/surface_geometry.py` exercises
actual sampling, failures, materials, topology projection, specialized solids,
and tracker-driven Y4M frames. The existing texture Gauntlet witness invokes
live geometry acceptance as well. Passing these bounded tests is not a claim
of full surface compatibility, all-platform certification or performance gates.
