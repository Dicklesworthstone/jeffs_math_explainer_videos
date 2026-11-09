# Native text and syntax-highlighted code

`Text` and `MarkupText` use Scribe's bundled fonts, style maps, shaping and
paragraph layout. There is no Pango, SVG intermediate, font discovery, external
LaTeX process or Python glyph renderer. `Text` treats its input literally;
`MarkupText` retains the supported native markup parser.

```python
heading = Text("Native typography", font="IBM Plex Sans", weight="BOLD")
body = Text(
    "Selective style\nwithout source rewriting",
    font="Computer Modern",
    t2w={"Selective": "BOLD"},
    t2s={"style": "ITALIC"},
    t2f={"source": "CM Typewriter"},
    t2g={"rewriting": [BLUE, GREEN]},
    alignment="LEFT",
    lsh=1.3,
)
```

## Font and style contract

The bundled text families are **Computer Modern**, **IBM Plex Sans**, and
**CM Typewriter**, including the native FontBook's documented aliases. Unknown
families fail by name; the system does not silently find a replacement in the
host's font directories. Supported weight/slant names are `NORMAL`/`BOLD` and
`NORMAL`/`ITALIC`, case-insensitively. Other weight/slant names currently refuse.

Whole-object font/weight/slant establish inherited character style. Inner
markup and local maps override them. Closing tags restore the inherited style;
`<tt>` selects the native typewriter face even inside an explicit base family.
No synthetic tags are inserted to do this: the stored string and UTF-8 glyph
spans still refer to the exact original input.

`t2f`, `t2s`, `t2w`, and `t2g` currently accept **literal source-string keys**.
Their long forms are `text2font`, `text2slant`, `text2weight`, and
`text2gradient`. As in the pinned constructor implementation, a nonempty long
form takes precedence over its short alias. The existing `t2c`/`text2color`
selector machinery continues to support its native Unicode/regex/span rules.
A whole-object `gradient` is applied after construction, and the `t2c` map is
applied last. Explicit VMobject style arguments keep their usual post-build
precedence. `global_config` and `local_configs` remain named refusals: this
surface does not claim to implement arbitrary Pango attributes.

`alignment` accepts `LEFT`, `CENTER` and `RIGHT`; an empty value preserves the
existing native left-aligned default. `lsh`/`line_spacing_height` is a positive
baseline-spacing multiplier, not an ink-height scale. `line_width` and `indent`
use the existing native layout's em units; `height` is a final scene-space
scale. Wrapping, justification and optional ligatures use the same native
layout as Rust `Text`. These are native typography semantics, not a claim of
pixel equivalence with Pango on arbitrary installed system fonts.

## Code uses positional native highlighting

```python
code = Code(
    "fn next(x: i32) { return x + 1; }",
    language="rust", code_style="monokai", font_size=32,
)
self.add(code)
code.get_part_by_text("next").set_color(YELLOW)
```

The existing native `franken_markdown` highlighter supplies token colors by
source position. A keyword's spelling inside an identifier, string or comment
does not acquire the keyword's color. The Code string remains the original
source, not generated markup; ordinary selectors, copies, reveals and string
matching keep those native spans.

The legacy Code-only default `font="Consolas"` explicitly selects **CM
Typewriter**, the sovereign native code face. `code.font` retains the authored
name and `code.native_font` records the selected family. This does not install
Consolas or alias that name for general `Text`. Pass a bundled family to select
it explicitly.

`default`, `friendly` and `colorful` map to the owned light palette; `monokai`,
`vim`, `one-dark`, `dracula` and `material` map to the owned dark palette. These
are native palette mappings, not byte-identical Pygments themes. Unknown themes
refuse. Language names pass to the native highlighter; its unknown-language
behavior is explicitly plain text. No Python lexer or subprocess is invoked.

Code shares the text layout's font, face, spacing, alignment and style options.
Native token fills take precedence over local native gradient maps on colored
tokens; a whole-object gradient or final `t2c` selectors can override those
fills. Code has one live glyph family and an empty parent point table, like
Text. The previous scaffold's copied parent outlines are no longer drawn over
its children: translucent ink and later selector edits therefore have one
source of rendered geometry. Use `get_all_points()` for complete Code geometry.

## Bounds and validation

Text/code source is limited to 262,144 UTF-8 bytes. Style maps and gradient stop
collections are bounded at 4,096 entries/stops; native literal style matching
is limited to 16,777,216 byte-work units. Coordinates, sizes and colors are
validated before the native glyph family is installed. Unsupported features
fail instead of emitting blank text or success-shaped placeholders.

`crates/fmn-python/tests/text_authoring.py` and `code_authoring.py` exercise the
installed native bridge, including original spans, fonts, inherited styles,
real pixel comparisons and one/four-thread replay. They do not establish full
cross-platform font parity or whole-project certification.
