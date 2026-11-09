# Complex-valued live readouts

The Python portal's `DecimalNumber` accepts finite real and complex values,
including NumPy complex scalars. Stock real/imaginary components use the existing
Rust number shelf; authored formatters and fonts use native Scribe glyph mobjects.
No additional font, TeX process, renderer, or animation clock is involved.

```python
from manimlib import *

class ComplexReadout(Scene):
    def construct(self):
        value = DecimalNumber(1 + 2j, color=BLUE, unit="V")
        self.add(value)
        self.play(ChangeDecimalToValue(value, -3j))
        self.play(ChangeDecimalToValue(value, 4))
```

`hide_zero_components_on_complex=True` hides exactly-zero components. Set it to
`False` to retain the full `0.00+2.00i` form. The imaginary component following a
real component always includes its sign; `include_sign` controls the first
component. Precision, comma grouping, zero padding and negative-zero suppression
follow the real-number contract independently for each component. Zero decimal
places truncate each displayed component toward zero, as real readouts do.

The `i` glyph is part of the numeric string. An optional ellipsis and user unit
follow it. Units remain native plain text: a leading `^` requests raised
alignment, not TeX interpretation. Use `unit="^°"`, not `unit=r"^\circ"`.
Background rectangles remain root records, with numeric glyphs as children.

`set_value`, `increment_value`, `ChangingDecimal`, `ChangeDecimalToValue`, and
`CountInFrom` can cross between real, imaginary and full-complex displays. Updates
retain the object identity, fixed edge, scaled font size and glyph style, and
replace the exact glyph list when a number becomes shorter. Nested animated
readouts remain descendants rather than acquiring duplicate scene draw roots.

Replacement geometry and style are prepared before updating the live receiver.
Non-finite components, invalid geometry settings, missing glyphs and exhausted
layout budgets therefore do not publish a new value alongside old glyphs. The
4,096-character budget covers the **combined** numeric string, ellipsis and unit,
not 4,096 characters independently for each component. Authored Python override
side effects are not rolled back.

This extends the portal, not the Rust scalar type: Rust's `DecimalNumber` still
accepts `f64`; Rust callers compose scalar readouts for complex values. The native
real-number builder remains unchanged.

Acceptance coverage: `crates/fmn-python/tests/decimal_authoring.py`.

## Complex matrix cells and live equations

`Matrix`, `DecimalMatrix`, `IntegerMatrix` and `TexMatrix` accept complex
entries from lists, NumPy arrays or row iterators. Numeric cells are live
`DecimalNumber` objects, not a typeset Python representation of a complex
number. `IntegerMatrix` truncates each displayed component while retaining the
original complex value. Existing real-only native constructor paths are unchanged.

The base `Matrix` also accepts already-built `VMobject` cells without requiring
an `element_config` workaround: `Matrix([[Circle(), Square()], [Tex("x"), 1.25]])`
preserves each supplied mobject and its row/column aliases. Geometry is arranged
through the existing native composition path, not stringified or rebuilt as
scalar text. Cross-scene entries are still rejected before layout moves them.

```python
class ComplexCells(Scene):
    def construct(self):
        matrix = DecimalMatrix([[1 + 2j, -3j], [4, 5 - 6j]])
        self.add(matrix)
        self.play(ChangeDecimalToValue(matrix.get_entries()[0], -2j))
```

A readout returned by `Tex.make_number_changeable` supports the same complex
updates and remains addressable through its live TeX span. Numeric animations
keep matrix entries, row/column aliases, brackets and top-level scene roots
intact. Matrix layout is established at construction; changing a cell does not
automatically reflow other cells or resize brackets.

The native PNG integration test is
`crates/fmn-python/tests/complex_readouts_render.py`. It checks visible changes
in both a live formula and a matrix across six frames, thread-count equality,
and cancellation of a generation whose readout update fails.

## Authored formatters, glyphs, and fonts

`DecimalNumber`, `Integer`, and numeric matrix entries support native typography
through `text_config`, including bundled font families, bold/italic faces and
the native Text style maps. Pass the overall `font_size` to the readout itself,
not inside `text_config`. Unsupported font names and Pango-only settings remain
explicit errors; there is no system-font or external-typesetter fallback.

```python
value = DecimalNumber(
    1 + 2j, font_size=36,
    text_config={"font": "IBM Plex Sans", "weight": "BOLD"},
)

class ScientificReadout(DecimalNumber):
    def get_formatter(self, **kwargs):
        return "{:.2e}"
```

The actual glyphs follow `get_num_string`, `get_formatter`, and
`get_complex_formatter` overrides, rather than drawing an unrelated scalar
while reporting the custom string. Subclass methods, instance overrides and
later class replacements all remain live. A `char_to_mob` override receives
each character and must return a `VMobject`; templates are copied before layout,
so sharing a cached template or returning another scene's mobject does not move
the original. A template with no `font_size` attribute uses 48 as its nominal
size. The native number builder remains the unchanged path for stock readouts.

The readout's default `char_to_mob` uses native **literal** Text, including for
the imaginary unit and backslashes. BN-08's raised-unit marker and one-child
U+2026 ellipsis are preserved. The separately exported `char_to_cahced_mob`
utility retains its existing Reference-compatible cached Text/Tex dispatch.

Public precision, minimum width, sign, comma, spacing, ellipsis, unit and
background attributes share the native constructor's parameter storage. Edits
take effect at the next `set_value`/rebuild. Scaled font size, fixed-edge
placement and live glyph paint survive that rebuild, including when a glyph is
a nested Text/VGroup rather than one record-bearing leaf.

Custom strings use the same combined 4,096-character limit. Formatting,
conversion and styling finish on detached geometry before replacing the live
readout. A failed glyph callback leaves its prior value and geometry intact;
arbitrary side effects inside authored callbacks are not transactional.

## Runnable native scene

`demo/python/native_numeric_authoring.py` combines a live TeX readout, a bold
complex coefficient, a scientific-notation subclass and already-live geometric
matrix cells. It uses only the production portal, Scribe typography and native
scene/render/output pipeline:

```sh
fmn-python demo/python/native_numeric_authoring.py NativeNumericAuthoring \
    --format png_sequence --video_dir out/native-numeric-authoring
```

Use a fresh output directory: publication remains no-clobber. Native acceptance
covers frame-by-frame motion, installed CLI output, bound-cell adoption and
unchanged cell aliases. The example is a standard render, not a claim of
certified output from arbitrary Python callbacks.
