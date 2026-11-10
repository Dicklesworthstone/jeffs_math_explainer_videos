# Point-cloud authoring and live glow

`PMobject`, `PGroup`, `DotCloud`, `TrueDot`, `GlowDots`, and `GlowDot` use the
shared initialization hooks. A cloud subclass can declare extra float32 record
fields and author points, uniforms, colors, children, and saved state without
being replaced by a stock native object. The native initializer assigns the
dot-cloud renderer identity before hooks, including for copies made by a hook.

```python
class Samples(DotCloud):
    data_dtype = [*DotCloud.data_dtype, ("temperature", np.float32, (1,))]

    def init_points(self):
        self.set_points([[-1, 0, 0], [0, 1, 0], [1, 0, 0]])
        self.data["temperature"][:] = [[10], [20], [30]]

samples = Samples(points=None, radius=0.2, color=BLUE)
samples.uniforms["glow_factor"] = 2.0
```

`points=None` retains hook-authored points. Omitting `points` uses the Reference's
one-point origin default. Explicit points replace hook-authored points after
initialization and radius assignment; an empty sequence produces an empty cloud.
`PGroup` retains hook-added children before its supplied point-cloud members.

## One native material, two authoring forms

A scalar write through `set_glow_factor`, `set_uniform`, `set_uniforms`, or the
live `uniforms` mapping broadcasts to the native `glow_factor` record lane.
When the cloud is empty, the write updates its retained style row, so later
appends and independent copies receive the material. Explicit scalar assignment
also deliberately replaces a heterogeneous per-point glow field.

Direct `data["glow_factor"]` edits remain per-point edits. The shared callback
record interpolator blends those values without broadcasting its scalar Python
uniform projection over them afterward. Data/uniform locks and authored paths
remain on the existing interpolation protocol. Ordinary native Transform
selection remains native; this adapter does not add a frame loop or renderer.

Scalar glow must be finite, non-negative, and float32-representable. Invalid
values and unusable fields refuse before accepting a new uniform value. Other
mobject types and unrelated custom uniforms retain their existing behavior.

## Validation boundary

`native_point_cloud_lifecycle.py` covers the new Rust initialization/validation
boundary and its public constructors. It requires a rebuilt native extension;
protocol probes against an older extension cannot prove primitive publication.
`native_point_cloud_materials.py` exercises actual records and Y4M output,
including retained-frame edits, independent constructor controls, and 1/4/16
worker comparisons. Both are registered in the embedded and installed gates.

The native catalog Transform's synchronization of Python uniform extras is a
separate remaining gap: its glow records can animate while the Python scalar
projection remains stale. This change fixes explicit scalar writes and the
shared callback interpolation path, not every native animation metadata path.
