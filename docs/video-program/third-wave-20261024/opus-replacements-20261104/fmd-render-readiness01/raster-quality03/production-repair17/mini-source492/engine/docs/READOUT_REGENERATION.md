# Numeric and copied-text regeneration

`DecimalNumber` and `Integer` construct through the existing native-backed
`VMobject` initialization protocol. Subclass recipes are available in `init_data`;
`init_points` and `init_uniforms` follow. As in the pinned Reference, `init_colors`
runs once during base initialization and again after the numeric glyphs exist.

```python
from manimlib import DecimalNumber, GREEN

class Readout(DecimalNumber):
    def init_colors(self):
        super().init_colors()
        self.set_color(GREEN)
```

Scribe/Atlas still build numeric glyphs and layout. Value updates publish those
results without replacing the root's declared record schema or discarding
subclass-created geometry and decorations. Native background geometry stays on
the ordinary readout root; when a subclass already draws there, the background
is a separately owned child. Custom record columns survive same-size writes;
resized views retain their existing native detachment behavior. Formatting and
glyph-construction errors occur before publication, preserving the prior value
and generated family. Arbitrary authored publication hooks are not transactions.

Generated-child ownership is represented using the shared copier's remapped
object-array protocol, not an ordinary shallow tuple of the original's children.
This applies to numeric readouts and to `Text`, `MarkupText`, `Code`, `Tex`, and
`TexText`. Consequently a copied object's `init_points()` replaces its own glyphs
instead of retaining them as decorations and appending a second set. Copying a
copy, deep copying, pickling, nested TeX groups, and `.animate.init_points()` use
the same rule. Decorations stay attached, native UTF-8 source spans are rebased
to the new glyphs, and the original object is unchanged. No general shallow-copy
policy, native typesetter, renderer, or animation clock is replaced.

The installed-native regression suites `decimal_lifecycle.py` and
`copied_string_regeneration.py` exercise real glyphs, custom record columns,
selectors, copies, failures, and changing PNG frames. Their pixel comparisons
include independent literal readouts/direct text and one/four-thread renders.
They are registered in `scripts/check_portal_runtime.sh`; they do not establish
whole-portal parity, a full-workspace pass, or cross-platform certification.

## Real infinities

With a positive `num_decimal_places` (including the default of 2),
`DecimalNumber(float("inf"))` displays `inf`; negative infinity uses the usual
en-dash minus. The public formatter still owns signs, field width and padding.
Infinity is a numeric display token, not a coordinate: only finite native Text
glyph records enter the scene. The existing authored glyph-composition path
builds those letters; the finite numeric shelf and its fast path are unchanged.

`set_value` and updater callbacks can switch between finite values and real
infinities while retaining the fixed edge, live style, font size, custom records
and decorations. Copies own their own glyphs. A formatter or glyph failure
occurs before the new value and generated family are published, as for finite
readouts. No new interpolation rule for arithmetic involving infinity is implied.

NaN and nonfinite complex components remain refused. Integer formatting
(`num_decimal_places=0`, including `Integer`) still requires conversion to an
actual integer, so infinity raises `OverflowError`, as in the pinned Reference.
Nonfinite geometry remains invalid; this feature does not adopt a NaN-curve
skipping policy or relax native record validation.

`infinite_readouts.py` covers positive/negative infinity, transitions, custom
formatters and glyphs, failure recovery, copies, finite record checks and
changing PNG sequences against independent literal glyphs at one/four threads.
