# SVG and line authoring

`SVGMobject` now runs the ordinary `VMobject` initialization lifecycle before
calling its public `init_svg_mobject` and `mobjects_from_svg_string` factories.
Custom root record columns, geometry and hook-created decorations stay attached;
the returned shapes become the actual children. The default factory still uses
Chisel/Atlas's native XML, path and paint processing. No Python SVG parser or
source rewriting is involved.

```python
from manimlib import SVGMobject, StrokeArrow, UP

class WideAsset(SVGMobject):
    def mobjects_from_svg_string(self, source):
        return [part.stretch(1.5, 0)
                for part in super().mobjects_from_svg_string(source)]

class RaisedStrokeArrow(StrokeArrow):
    def init_points(self):
        super().init_points()
        self.shift(UP)
```

The SVG constructor keeps its existing source-resolution, y-flip, style and
centering/size finalization order. An explicit constructor style is applied
after authored assembly. The existing paint-role masks keep a stroke layer from
acquiring unintended fill during restyling. Calling `init_svg_mobject` again is
additive, including on scene-bound objects; it is not an implicit source reload
or replacement operation.

The default assembler freezes a complete factory iterable before adoption.
It rejects reused roots, references to its own existing family, foreign scene
geometry, invalid types and nonfinite points. Shared descendants remain shared.
The limits are 65,536 parts/family members and 1,048,576 points. Failed factories
leave the previous family installed; arbitrary effects inside Python callbacks
are not rolled back. The separate `svgelements` object-model/parser exclusions
on direct SVG-path helper APIs are unchanged.

`StrokeArrow` now resolves endpoint mobjects and constructs cooperatively through
`Line`/`VMobject`, so data, point, uniform and color hooks run in normal MRO order.
Its native in-place builder supplies the tapered point run and width profile
without replacing custom record schemas or children. As before, `set_stroke`
resets the taper from current endpoints without a further buffer; default
`init_points` rebuilds from the original endpoint/buffer recipe. Copies do not
rerun initialization. Native column views stay live on same-size point writes;
resizing and nursery-to-Scene adoption retain their existing detachment rules.

Ordinary `Line` endpoint and arc regeneration also uses the existing in-place
native writer while detached, rather than replacing the whole nursery. This
preserves custom fields, children, saved state, updaters and view generations,
including when a line is copied into an `.animate` target. The native curve
builder, style-hook dispatch, renderer and animation clock are unchanged.

`svg_lifecycle.py` and `line_stroke_lifecycle.py` are installed-native acceptance
suites registered in `scripts/check_portal_runtime.sh`. They exercise real record
storage, native curves/paint and changing PNG frames at one/four threads against
independent geometry. These targeted cases do not establish whole-portal or
cross-platform certification.
