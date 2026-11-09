# UV-aware surface morphing

`Transform` can align sampled native surfaces with different UV resolutions.
This is distinct from `Surface.init_points()`, which still regenerates a fixed
UV topology from the current authored recipe. Both native Rust and the optional
Python portal use Marionette's one UV-record alignment owner.

```python
from manimlib import *

class SurfaceMorph(Scene):
    def construct(self):
        source = ParametricSurface(lambda u, v: [u, v, u * v],
                                   u_range=(-1, 1), v_range=(-1, 1),
                                   resolution=(5, 9), color=BLUE)
        target = ParametricSurface(lambda u, v: [u, v, 0.5 * (u*u - v*v)],
                                   u_range=(-1, 1), v_range=(-1, 1),
                                   resolution=(11, 7), color=RED)
        self.add(source)
        self.play(Transform(source, target), run_time=2)
        # Source now has an 11-by-9 native grid and matching public indices.
        # The authored target keeps its original 11-by-7 grid.
```

## Alignment and ownership

The common resolution is the component-wise maximum of the two UV shapes.
Equal flat point counts do **not** imply equal topology: a 2-by-3 surface and a
3-by-2 surface align to 3-by-3. Every numeric record lane is resampled on the
normalized UV chart, using bilinear interpolation in fixed order. This includes
positions, normal-control points, colors, texture coordinates, opacity and
custom schema fields. Native placement, materials, handles, family membership,
updaters and saved states keep their existing owners.

Alignment uses the stored sampled geometry. It never calls either UV function,
reopens a texture, or regenerates analytic geometry. Bilinear resampling is an
approximation of the stored UV interpolant: changing the grid does not promise
pixel-identical endpoint triangulation for arbitrary curved surfaces. Existing
sampling knots that coincide with new stations and boundary endpoints are kept
exactly. Native temporal interpolation remains the normal Transform mechanism.

A resized record generation detaches old live views under the existing view
protocol. New views follow the aligned geometry and subsequent animation. A
same-topology alignment does not replace the native record generation.

`surface.align_points(other)` intentionally aligns both operands. `Transform`
aligns a protected target copy whenever shapes or schemas differ; it does not
resize the caller's authored target. Replacement transforms install the original
authored target after playback, retaining that target's own topology. Python
surface leaves use the existing ordered Transform lifecycle, including nested
Groups and Succession, so public `resolution`, triangle indices and UV queries
agree with the native buffer before scene updaters or frame capture run.

## Boundaries and failures

Both dimensions must be at least two, the grid must match its record count, and
the common grid may not exceed 65,536 points. Both operands must have matching
record schemas and finite fields. Invalid pairs fail before either pair member's
records are changed. This pair-level guarantee does not roll back arbitrary
family edits or authored callbacks performed earlier by a higher-level animation.

This operation is for rectangular sampled UV surfaces, not arbitrary indexed
meshes, point clouds or surface-to-vector-path morphs. Schema conversion and
texture-image crossfades are not introduced: textured surfaces retain their
existing decoded material owner while numeric texture coordinates interpolate.
The source's authored UV domain and function remain its recipe after a morph;
call `become` for topology/recipe replacement or construct a new surface for a
new recipe. Nonlinear camera/chart mapping is not part of this resampler.

`crates/fmn-mobject/tests/surface_grid.rs` and
`crates/fmn-anim/tests/surface_grid_alignment.rs` exercise native alignment and
Transform. `crates/fmn-python/tests/surface_alignment.py` adds installed/embedded
portal tests and real threaded Y4M output; no replacement renderer is used.
