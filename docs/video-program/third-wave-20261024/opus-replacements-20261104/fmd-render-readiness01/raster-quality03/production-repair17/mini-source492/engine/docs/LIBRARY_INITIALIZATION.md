# Native-backed library initialization

`NumberLine` / `UnitInterval` and `Text` / `MarkupText` / `Code` / `Tex` /
`TexText` now run the ordinary `VMobject` initialization hooks rather than
installing a replacement native tree around an already allocated subclass.
The exported classes and their inheritance retain their identities.

```python
from manimlib import Text, NumberLine, UP, GREEN

class TallText(Text):
    def init_points(self):
        super().init_points()
        self.stretch(1.5, 1)

    def init_colors(self):
        super().init_colors()
        self.set_color(GREEN)

class RaisedTicks(NumberLine):
    def get_tick(self, x, size=None):
        return super().get_tick(x, size).shift(0.3 * UP)
```

## Lifecycle and factory ownership

Recipe fields are available before `init_data`; `init_points`, `init_uniforms`
and `init_colors` follow in that order. Each hook runs once through native
initialization. Subclasses can call `super()` or supply their own implementation.
Exceptions propagate without retrying the hook. Root `data_dtype` extensions,
custom uniforms, and children added by initialization hooks stay attached.
Arbitrary authored hook effects are not sandboxed or rolled back.

Number lines build through `Line`, apply width or unit scale, center, and then
call the public tip, tick and label factories. Tick size is therefore not
scaled with the axis length. An asymmetric range is centered, as in the pinned
Reference; `n2p` still locates each coordinate from the live endpoints. An
explicit `stroke_color` takes precedence over the default `color`. The default
`get_tick_range` reads current range and `include_tip` fields, so regeneration
honors edits. `add_ticks` adds a new group and updates `ticks`, without removing older
groups implicitly.

The default tick assembler stages a complete group before adding it. A failed
`get_tick` leaves the previous group intact. Returned lines must be independent,
detached, bounded, and finite; already-owned scene geometry is not rotated or
adopted as a tick. Tick counts are bounded before allocation.

## Native typography and source spans

Scribe still creates all glyph outlines, markup paints, syntax highlighting and
UTF-8 span maps. Those native results are published through the initialized
object's point and family interfaces; they do not replace its custom record
schema. Span paths are rebased around hook-created decorations, including nested
TeX groups, so selectors and matching transforms still address actual glyphs.

Default string `init_colors` applies explicit caller styles without replacing
native per-glyph paints with a uniform default. An authored override can then
restyle the glyphs. Constructor-level color maps, gradients, centering and height
adjustments retain their existing finalization order after the hooks. Use
`should_center=False` when an absolute placement from a hook should remain.

Calling `init_points` again builds a detached typesetting candidate first, then
replaces its prior generated glyph children while preserving other children.
A typesetter failure leaves the old family and span map intact. This is point
regeneration, not a new automatic source reload or a transaction around custom
`set_points` / `set_submobjects` overrides. `Code` retains its public legacy font
name and uses its recorded native font for regeneration.

## Validation boundary

`number_line_lifecycle.py` and `string_lifecycle.py` exercise the actual native
portal, custom record fields and callbacks, span selection, matching and PNG
output against independently assembled geometry and one/four-thread controls.
They are registered in `scripts/check_portal_runtime.sh`. These features do not
by themselves establish whole-library parity, performance qualification, or
cross-platform certification.
