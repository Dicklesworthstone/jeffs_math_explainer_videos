# Mathematical Markdown scenes

`MarkdownMobject(source)` composes native Scribe prose with the existing TeX
engine (plan §11.7: fmd's parser, Scribe layout, fmd-math mathematics). It
extends the source-addressable document API; it does not replace its native
parser, scene objects, source edit machinery, or frame clock. Pass
`math_mode=False` to keep every dollar literal.

```python
from manimlib import *
from fmn_python.markdown import MarkdownMobject

class Derivation(Scene):
    def construct(self):
        source = "# A useful identity\n\nFor $x>0$, let $f(x)=x^2$.\n\n```math\nf'(x)=2x\n```"
        document = MarkdownMobject(source, line_width=7, font_size=28)
        self.add(document)
        self.play(document.select_text("f'(x)")[0].animate.set_color(YELLOW))
        self.play(document.animate_source(source.replace("x^2", "x^3").replace("2x", "3x^2")))
```

## Native layout and controls

Dollar islands `$...$` use native inline mathematics. Delimiters follow fmd's
own rule: an opening dollar must precede non-whitespace and a closing dollar
must follow it, so `$5 and $10` stays prose, as it does in fmd's HTML and PDF.
Double-dollar islands occupy their own lines; they and `math`/`tex` fences
use native display style. A `textext` fence uses the native
text-mainland TeX API. Underscores and backslashes inside formulas are protected
from Markdown interpretation. Reference links retain document-wide definitions even in mathematical paragraphs.
Inline code and ordinary fenced code remain literal; an escaped or unmatched dollar remains text. Formulas have the same
native glyphs, rules, calibration and error messages as ordinary `Tex` objects.

`line_width` is a paragraph measure in initial scene units. Words, code spans,
and formulas are indivisible: wrapping occurs at whitespace, and an oversized
atom can exceed the measure. Lines use actual native ascenders, descenders and
advances so fractions and radicals do not collide with adjacent lines. Text and
math are seated on shared baselines, not center-aligned boxes. Headings, nested
lists, task markers, quotations and native ruled tables retain their structure.
Tables and code blocks are not automatically scaled or wrapped to the measure.

`body_color` sets native text, mathematical and rule colors without replacing
fenced-code syntax colors; without it, table rules take `TableMobject`'s
`GREY_B`. `line_width` and `body_color` are refused with `math_mode=False`.
The existing `theme`, `font_size` and `block_gap` options remain available;
block gaps scale with font size as in text-only documents. Ordinary inherited
whole-family color changes still affect every selected native member.

## Live editing and animation

Math mode, paragraph measure and body color survive copies, pickle, scene
checkpoints, `set_source`, and `animate_source`. Unchanged source blocks preserve
their identities and authored styles. Changed source is prepared entirely through
the native engine before live geometry is replaced. The same source is a no-op.
A malformed formula or unsupported asset fails without replacing a live document.

The existing `animate_source` operation resolves its target at `begin`, uses
normal Transform easing/path/lag and composition, and commits source metadata
only at complete endpoints. Equal block counts are required for morphs;
`set_source` handles structural insertion/removal discretely. Cancellation restores
the beginning document. A partial endpoint remains visual and must be finished or
restored from a checkpoint before further edits. The ordinary `.animate.set_source`
spelling refuses rather than lose source metadata in a generic method transform.

`get_block_source`, `select_source` and `select_text` still address original source.
`block_ranges` use UTF-8 bytes; `select_source` accepts Python character offsets.
Selections return live **whole blocks**, not invented per-glyph Markdown spans.
Source highlighting across individual Markdown tokens remains outside this API.

## Boundaries

No network fetches, browser, external TeX, or system-font lookup occurs. Math-mode
images are refused by name rather than substituted with an unrelated shape.
Links retain their labels without fetching destinations. Raw HTML is literal
text. Native table cells currently flatten inline styling and refuse mathematical
islands; use native `TexMatrix` for a mathematical array. Unsupported TeX constructs
or characters return their native diagnostics.

Admission retains the existing 32,768-byte, 256-block and 24-level document limits,
plus bounded layout atoms, shaped characters and native points. One block admits
at most 256 dollar islands and each island at most 8,192 bytes. For multiline or
structurally complex display mathematics, use a `math` fence: dollar islands are
scoped to original parser blocks, not a second Markdown block grammar. This is
not a browser layout or full CommonMark/LaTeX implementation.

## Saved-state recovery

`save_state()` followed by `restore()` recovers the saved native document even
when intervening edits inserted blocks, removed blocks or changed the glyph
family shape. Restoration copies saved native geometry rather than reparsing or
retypesetting the source. Saved paint, placement, document settings, catalogs and
annotations are recovered without advancing the clock or replacing the scene's
root object, visible-root list or root callbacks. Repeated restoration does not
mutate the saved snapshot.

Restored descendants are independent native copies, not aliases to the snapshot
or promises to retain old glyph/block handles. Use `Scene.get_state()` and
`Scene.restore_state()` when restoring the original block identities matters.
Restoration refuses while the document is actively animating. A saved document
bound to another Scene is copied without changing its original owner. A finished
partial source morph can be recovered with `restore()` and then edited again.
The `.animate.restore()` shorthand refuses; use the explicit source animation API
for document morphs. Generic `Restore` is not an additional document-source
animation protocol.

Native Rust callers keep the existing `Markdown::build` literal behavior and
select mathematics through `build_with_math(book, engine)` or
`build_with_math_options(book, engine, MarkdownMathOptions { line_width, color })`.
