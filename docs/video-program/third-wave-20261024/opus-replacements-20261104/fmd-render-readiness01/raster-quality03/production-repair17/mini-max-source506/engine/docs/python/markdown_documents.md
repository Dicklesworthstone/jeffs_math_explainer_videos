# Native Markdown documents

```python
from manimlib import *
from fmn_python.markdown import MarkdownMobject

class DocumentScene(Scene):
    def construct(self):
        document = MarkdownMobject(
            '# A native slide\n\n**Bold** text and `Vec<T>`.\n\n- first\n- second',
            font_size=28,
            theme='monokai',
            block_gap=0.45,
        )
        self.add(document)
        self.play(document.get_block(0).animate.set_color(BLUE))
        self.play(document.animate_source(
            '# An edited slide\n\n**New** text and `Vec<T>`.\n\n- first\n- third',
        ))
```

`MarkdownMobject` is an enhanced `VGroup`, imported explicitly from
`fmn_python.markdown`; it does not replace a compatibility-namespace class.
Atlas owns the fmd Markdown parser and Scribe owns glyph shaping, code themes
and native table layout. The host retains source catalogs and ordinary native
object families. There is no browser, external typesetter or host Markdown
parser, and no source is executed.

## Content and provenance

Headings, paragraphs, emphasis, strong text, strikethrough and inline code use
the native markup/glyph pipeline. Literal `<`, `>` and `&` remain literal;
inline code cannot become styling markup. Fenced code uses native syntax
highlighting. Top-level tables with body rows use the native ruled-table
layouter; cell text is retained but inline cell styling and column alignment
hints are not applied. Without a `body_color`, table rules use `GREY_B` so they
remain visible on the scene's default #333333 background (and on black); cell
fills, native stroke widths and syntax colors are otherwise retained. The
mathematical layout of lists, quotations and rules is in
`mathematical_markdown.md`. In the `math_mode=False` text-only mode, lists and
block quotes flatten with explicit prefixes, retaining nested non-paragraph
content; nested tables and header-only tables use a textual presentation; and
thematic breaks are a textual rule.

The source is bounded to 32,768 UTF-8 bytes, 256 top-level blocks and 24 levels
of block/inline nesting; native completed geometry is bounded to 1,048,576
records, and each table to 4,096 cells. These are document admission limits,
not hard process-wide peak-memory limits. The parser runs only after source
admission; AST depth is checked before recursive presentation and font work.

`source`, `block_ranges` and `block_kinds` describe the last committed source.
Ranges are half-open **UTF-8 byte** intervals returned by fmd, not Python
character indices. `get_block_source(i)` decodes the exact interval.
`get_blocks()` and `get_block(i)` return live native objects. Negative block
indices are accepted. `select_source(start, end)` accepts half-open Python
**character** offsets and selects every intersecting whole block.
`select_text(text, occurrence=0)` selects one non-overlapping literal occurrence
in the original Markdown, including its delimiters. Neither selection implies
inline glyph-to-source mapping. Empty intervals select nothing; absent literal
occurrences raise rather than selecting an unrelated object.

`get_blocks()` returns the document's owned block container, so animating it
does not promote a temporary selection group into a scene root. Partial source
selections are ordinary `VGroup` views: animate their individual blocks (for
example `document.select_text('term')[0].animate.shift(RIGHT)`) to retain the
document root for subsequent insertions. Animating a new selection group itself
has the normal native scene-grouping semantics and can ungroup its old parent;
explicitly re-add the document before further live edits in that workflow.

## Discrete edits and placement

`set_source(text)` prepares a complete native layout before publishing new
content. Exact unchanged blocks are matched in source occurrence order and
retain glyph identity, local styling, updaters and attached views. Insertions
cannot steal those identities. Changed same-kind blocks may retain their
positional wrappers, annotations and updaters, but receive freshly shaped native
content. Deleted blocks leave the document catalog; external references may
still own them. Root annotations are retained and are not treated as Markdown.

New content reflows in the document's retained upper-left affine frame. Whole
object scaling, rotation, reflection, shear and 3D plane placement are retained.
The three invisible frame anchors lie at the native layout bounds; an empty
document uses an invisible unit frame so it can later grow. Independently edited
unchanged blocks keep their local shape/style and shift to their new layout
station; this is not automatic collision avoidance for annotations or local edits.

Source parsing/shaping failures leave existing geometry and source metadata
unpublished. The invocation guard is outside copied object state, so copies,
pickles and saved states made during preparation are not permanently locked.
Reentrant source edits, record locks, damaged owned families, singular placement,
active/partial document animation, and detected authored changes during preparation
refuse. Arbitrary side effects from host callbacks or custom publication methods
are not transactionally rolled back.

`save_state()` / `restore()` restore both source metadata and the complete saved
family, including after a change in block/glyph counts. The document root and
its scene membership survive, but direct document restoration installs copied
saved blocks; reacquire block selections afterward. Scene checkpoints instead
use the existing native identity-preserving scene restore and retain original
block references. Neither operation changes the saved source's geometry.
Calling the constructor again on an existing document refuses; use `set_source`.

## Animation and scope

`animate_source(text, **animation_config)` uses the existing `Transform`
lifecycle. Targets are prepared at `begin`, so `Succession` observes preceding
edits. This tranche requires equal top-level block counts; use discrete
`set_source` for inserting/removing blocks. Source metadata commits only after a
complete final endpoint. Cancellation restores the previous source/geometry;
a fractional final alpha remains a visual intermediate and must be restored
from a checkpoint before another source edit. Easing, duration, sub-alpha and
frame order remain the shared animation contract.

The `.animate.set_source(...)` spelling refuses explicitly. Generic `Transform`
of documents remains a geometry operation; use `animate_source` to also commit
source metadata. Normal native block animations, copies and scene checkpoints
remain available. Changed block content uses the new source's native styles;
unchanged content retains authored styling.

Documents typeset `$...$`, `$$...$$` and `math` fences natively by default;
that layout and its wrapping controls are described in
`mathematical_markdown.md`, and edits and restore retain those settings too.
There is no embedded-image loading or network access: an image is a named
error that asks for an explicit scene asset. Links show their text. The
`math_mode=False` text-only mode keeps HTML and math syntax literal, shows
image alt text, and has no automatic wrapping.
Code themes are the names accepted
by the native `CodeTheme` catalog. `font_size` must be finite in `(0, 10000]`;
`block_gap` must be finite in `[0, 1000]` and scales with body font size.

The native/raw tests are `markdown::tests` and `native_markdown.py`; the public
scene/ownership/rendering suite is `live_markdown.py`. The no-assets example is
`demo/python/live_markdown.py`.
