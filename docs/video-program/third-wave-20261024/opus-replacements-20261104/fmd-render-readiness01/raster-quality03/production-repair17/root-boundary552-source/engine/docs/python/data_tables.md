# Native data tables

`fmn_python.table.TableMobject` connects Atlas's existing Data table builder to
Python scenes. CSV is parsed by the pinned frankenpandas implementation; table
metrics, glyphs, padding and rule placement come from the native Scribe builder.
There is no pandas installation, system font lookup, temporary file or external
parser involved. This is an enhanced-tier extension, not a new reference
`manimlib` symbol.

```python
from manimlib import *
from fmn_python.table import TableMobject

class Results(Scene):
    def construct(self):
        table = TableMobject.from_csv("method,error\nEuler,0.1\nRK4,0.001\n")
        self.add(table)
        table.get_cell(1, "error").set_color(GREEN)
        self.play(table.animate_data([["Euler", "0.05"], ["RK4", "0.0001"]]),
                  run_time=1)
        table.set_data([["Euler", "0.05"], ["RK4", "0.0001"], ["Exact", "0"]])
        self.wait()
```

## Inputs and live selection

The literal constructor takes headers and a rectangular body of **strings**.
It deliberately does not guess arbitrary Python object formatting. `from_csv`
and `set_csv` perform native column inference. Native scalar display uses empty
text for null, lowercase `true`/`false`, decimal integers and shortest round-trip
floats (negative zero normalizes to zero).

The currently governed `fp-frame` convenience reader is delimiter-only, not a
quote-aware CSV reader. **Quoted input and inconsistent field counts are refused**
instead of silently moving/dropping data. Use a literal string grid for fields
containing separators or quotes. The suite's quote-aware `fp-io` reader brings an
unadmitted Arrow/Excel/parser dependency closure and is deliberately not smuggled
into the engine. Full quoted CSV ingestion remains an explicit integration gap.

Tables require at least one header and one body row. Admission is bounded to
4,096 cells including headers and 262,144 UTF-8 input bytes. Literal iterators are
consumed with a bound, so an infinite row generator is refused rather than hung.
Unsupported characters raise the native missing-glyph error; no font downloader
or silent substitution is introduced.

`headers` and `values` are immutable tuples; `shape` excludes the header row.
`get_cell(row, column)` uses zero-based body rows and supports negative indices.
Columns can also be addressed by an unambiguous header string. `get_headers()`,
`get_rows()`, `get_columns()` and `get_rules()` return groups of **live objects**,
not copies. Normal color changes, cell animations, updaters and scene snapshots
therefore operate on the displayed native family.

## Replacing data

`set_cell`, `set_data(rows, headers=...)` and `set_csv` prepare and shape a new
native table before replacing live geometry. Surviving cells are matched by
ordinal `(row, column)`, not header text or flattened array index. Their cell
handles, callbacks, and attached annotations survive. Glyph families are replaced
when data changes; old glyph references are not promises of live text selection.
Surviving rules are matched by role: outline, horizontal boundary or vertical
boundary. Root-level annotations are untouched; cell annotations translate with
their cell center.

The new layout retains the old **upper-left** placement and grows right/down in
its existing affine frame. Rotation, nonuniform scale, shear, reflection and 3D
planar placement are retained. Degenerate frames are refused. Native rule geometry
is the layout authority; host-side invisible anchors merely carry that geometry
through normal mobject transforms and provide positions for empty cells. Nonlinear
warps of the table are not a persistent layout constraint: a later replacement
rebuilds native text and rules in the carried affine frame.

Cell styles are transferred to replacement content. New cells use the constructor's
body/header styles. Equal data is a no-op. A callback or input generator that edits
the live table while its replacement is being prepared causes a stale-update
refusal; the authored edit is not erased. Copying/checkpointing inside that input
callback does not copy an active-operation lock. The guard excludes reentry, not
arbitrary concurrent Scene mutation.

## Animating values

`table.animate_data(rows, headers=..., **animation_options)` returns a
`TableDataAnimation`, a normal `Transform` subclass. It uses the existing path,
rate-function, lag, updater, composition and frame-clock machinery. Targets are
laid out at `begin`, so a `Succession` sees earlier completed changes. Morphs
require an equal row/column count; `set_data` handles shape changes discretely.
The standard `.animate.set_data(...)` shorthand refuses and names `animate_data`
instead of silently losing table metadata in a generic method transform.

During a morph, `values` describes the **last committed dataset**, not an invented
intermediate string. A full target endpoint publishes exact target glyph families
(removing alignment padding); a full returning endpoint retains the original
dataset. Cancellation restores the prepared beginning dataset. An intentionally
partial final endpoint remains visual rather than being reported as completed
new data; reflow is refused until the animation reaches a full endpoint or a
saved scene checkpoint is restored. Geometry-only transforms of a table continue
to use ordinary `Transform`/`.animate` as before.

The native typesetter uses its fixed table font and layout metrics, scaled by
`font_size` (default 24). Per-cell text formatting callbacks, automatic key-based
row matching and date localization are not part of this API.
