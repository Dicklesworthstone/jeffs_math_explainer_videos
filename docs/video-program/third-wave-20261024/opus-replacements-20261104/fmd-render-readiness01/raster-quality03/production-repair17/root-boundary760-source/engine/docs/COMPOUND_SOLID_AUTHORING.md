# Authoring compound solids

`Cube`, `Prism`, `VCube`, `VPrism`, `Dodecahedron` and `Prismify` construct through
their existing native-backed group classes. Root `init_data`, `init_points`,
`init_uniforms` and `init_colors` hooks run in the normal inheritance order;
custom record columns, root geometry and hook-created decorations are retained.
No independent native root replaces that authored group.

Cube faces come from the public `Square3D` and `square_to_cube_faces` symbols;
vector cube faces come from `Square` and that same helper. Lookup uses the
class's defining `manimlib.mobject.three_dimensions` module, with the existing
native namespace as fallback. A replaced helper may return a bounded generator.
The resulting faces, including custom subclasses and paints, become the actual
children rather than being reconstructed from the original size recipe.

`Prism` and `VPrism` forward their options through their cube parent, then call
public `rescale_to_fit` for width, height and depth. An override sees those
operations on the initialized, populated group. Cube's legacy `resolution`
metadata still describes the faces; its empty root is not itself a sampled face.
Only generated surface faces are indexed, so separately authored decorations
retain their own grid topology. Generated-face references follow the copier's
existing object-array remapping protocol.

```python
from manimlib import VCube, Prismify, Square, Dot, UP, RED

class MarkedCube(VCube):
    data_dtype = VCube.data_dtype + [("mass", 1)]

    def init_points(self):
        super().init_points()
        self.marker = Dot().shift(2 * UP)
        self.add(self.marker)

    def init_colors(self):
        super().init_colors()
        self.marker.set_fill(RED, opacity=1, border_width=0).set_stroke(width=0)

cube = MarkedCube(side_length=1)
extruded = Prismify(Square(side_length=1), depth=0.4)
```

Dodecahedron faces and Prismify's extrusion geometry continue to come from Atlas.
They are prepared off-object, then supplied to the ordinary group constructor.
An extrusion reads the live source geometry, including a source already in a
scene, without modifying it. Family extrusions retain each point-bearing
source's own paint. Explicit Prismify paint options apply to its root rather
than flattening all child colors. VGroup3D's normal post-initialization depth,
shading and joint configuration still runs on the completed family. Native
z ordering is published through `set_z_index`, not merely a Python attribute.

Face factories must return compatible, detached objects. Duplicate roots,
cycles, a reference to the target itself, nonfinite geometry and over-budget
families refuse before adoption; shared descendants retain their identity.
The limits are 65,536 family members and 16,777,216 records. Surface cube sampling
also applies the existing aggregate six-face grid budget. A failed factory or
initialization hook is not retried, and invocation guards are released on failure.
Arbitrary authored callback effects are not sandboxed or rolled back. Re-running
a constructor on a scene-bound object is refused; ordinary transforms and copies
remain supported.

## Validation boundary

`solid_group_lifecycle.py` and `vector_solid_group_lifecycle.py` exercise real
native records, face factories, copying, source preservation and animated PNGs.
Independent native builders supply the geometry controls; one/four-thread output
is compared byte-for-byte. Rotation controls use a common explicit pivot rather
than separately rounded family bounding boxes. Public face rotations may differ
from monolithic f64-before-narrowing builders by near-zero rounding, so numeric
control comparisons use an explicit absolute tolerance rather than claiming
identical geometry bits for every construction history. These targeted tests do
not establish whole-portal, fresh-wheel or cross-platform certification.
