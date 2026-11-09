# Transform image geometry and materials together

The Python portal's ordinary `Transform` now interpolates native images as well
as record fields and numeric uniforms. It uses the existing native raster
transition kernel and the existing animation frame order; there is no second
clock, renderer, or Python pixel-mixing implementation.

```python
from manimlib import ImageMobject, Transform, linear

start = ImageMobject(red_rgba, height=2).shift((-2, 0, 0))
end = ImageMobject(blue_rgba, height=3).shift((2, 0, 0))
self.add(start)
self.play(Transform(start, end, rate_func=linear), run_time=1)
```

The starting object's geometry and material change together. The original
target remains the endpoint, not a writable scratch image. The same material
axis applies to `ReplacementTransform`, `TransformFromCopy`, `MoveToTarget`,
`Restore`, and deferred method transforms such as `.animate.become(other)`.
Scene removal/replacement follows each animation's established contract.

The direct `Mobject.interpolate(mobject1, mobject2, alpha, path_func=None)` route
also interpolates images. Native resource identity is authoritative, including
for a base `Mobject` with an image-compatible record schema. Python class names,
filenames and cached pixel-dimension attributes do not supply the pixels.

## Family, timing and geometry rules

The existing family alignment and lag-ratio traversal determine the paired
objects and each member's alpha. Paths, rate functions and time spans are not
re-evaluated for the image axis. Native UV-grid alignment still resamples grid
records; indexed triangle meshes retain their existing alignment route. Images
can change raster dimensions without changing their independently interpolated
scene-space geometry. Light/dark materials publish as one image resource.

Each changed material gets an immutable native interpolation plan, prepared at
begin and refreshed when either actual endpoint's native resource changes.
Thus Succession captures the preceding result, and existing target-updater
semantics remain live. Cached plans are released on completion, failed begin,
failed interpolation and abort. Cancellation restores acquired transient flags
and locks, not authored geometry, scene time or arbitrary host side effects.

Interpolation is premultiplied linear light through Lumen's texture sampler.
Halfway opaque red/blue is `(188, 0, 188)`, not `(128, 0, 128)`. Endpoint transfer
metadata, alpha, hidden RGB and dimensions are exact at alpha zero/one. Eased
values outside zero/one clamp the image axis to endpoints, while geometry keeps
its ordinary path semantics. Nonfinite image alpha refuses before publication.
See `live_rasters.md` for PNG gamma, sampler and individual transition limits.

Unchanged materials do not get decoded or rewritten by a Transform. Ordinary
image placement retains the native animation route, and can run alongside an
explicit `RasterTransition` without overwriting its independently changing
pixels. A direct `Mobject.interpolate` call does copy even a constant endpoint
material into its destination, as it copies the other supplied state axes.

## Admission and limits

Both corresponding drawable endpoints must have compatible native image
primitives: image quad/image quad, triangle mesh/triangle mesh, or sampled
surface/sampled surface. Image-to-vector and textured-to-untextured primitive
conversion is not implemented; it refuses rather than silently losing pixels.
Existing geometry and family alignment limitations still apply.

A Transform family admits the sum of decoded endpoint texels before any family
material decode or interpolation, with a 256 MiB decoded limit. Each native plan
also enforces its own endpoint and intermediate image limits. Shared aliases
are deduplicated by the actual live/start/end tuple; distinct pairs are charged
conservatively even when their immutable resources share storage. This is not a
global memory limit across multiple simultaneous animations or scene history.

Authored paths, descriptors and updaters retain their ordinary host authority.
Native image preparation does not make arbitrary Python execution transactional.
A later authored failure can follow earlier successful family writes; frozen
frame/output ownership retains the existing no-partial-publication contract.

`demo/python/raster_transform.py` demonstrates simultaneous material and geometry
changes without input assets. `tests/raster_transform_kernel.py` verifies native
endpoint admission. `tests/raster_transform.py` checks the full portal lifecycle
and compares rendered frames with independently constructed colors and positions
at one, four and sixteen threads. These APIs are standard host authoring, not a
claim of a new certified asset closure or a material-aware Rust Transform API.
