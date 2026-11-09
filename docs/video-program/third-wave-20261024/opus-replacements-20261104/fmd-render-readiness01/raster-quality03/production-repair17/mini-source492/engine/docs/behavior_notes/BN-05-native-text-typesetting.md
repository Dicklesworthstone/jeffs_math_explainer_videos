# BN-05 — Native text typesetting: bundled faces, owned shaping, no Pango

**Status:** Draft.

**Subsystem:** Scribe I (fmn-text) · **Plan:** §11.2, D-08 · **Bead:** fm-u1u

## What the Reference does

`Text`/`MarkupText` render through Pango against **host fonts** (the
Reference's default resolves to whatever the platform maps its default
family to — commonly Consolas on Windows-authored scenes), with Pango's
shaping, ligatures, and line metrics. Output therefore differs between
machines with different font installations and Pango versions, and a
missing font silently substitutes.

## What FrankenManim does

- **The default text face is bundled Computer Modern** (CM Unicode), the
  3b1b typographic identity; CM Typewriter serves `<tt>`, IBM Plex Sans is
  the bundled sans. A bare install renders every built-in scene
  identically on every machine.
- **Shaping is owned**: cmap→gids, kerning from the legacy `kern` table
  plus the GPOS `kern` feature (CM Unicode kerns through GPOS), and the
  bundled faces' **ligature sets off by default** — matching the familiar
  manim look — with an explicit opt-in flag.
- **A missing font family is a named error**
  (`font family 'X' is not available; bundled families: …`), never a
  silent substitution. User TTFs load from bytes under the capability
  doctrine; the family-name lookup tolerates the bundled aliases only.
- **Per-character coverage falls back through a fixed, bundled chain.** A
  character the selected face has no glyph for (U+2212 MINUS SIGN, `×`,
  `→`, `½` in the Computer Modern subset) is drawn from Computer Modern,
  then the Noto Sans Math symbol face, then IBM Plex Sans — the first
  that covers it — and the glyph records the face it came from. Like
  Pango's fallback, but compiled in, so identical on every machine. Only
  a character no bundled face covers is a named `UnmappedChar` error.
- **Line metrics are fixed constants** (1.2 em baseline skip ×
  `line_spacing`, greedy breaking with manim's width semantics;
  least-badness is an explicit option), not Pango's font-derived leading.
- **Markup** is the manim tag set with precise line:column diagnostics —
  unknown tags/attributes are errors, not passthrough.
- **Empty input constructs an empty mobject.** At the pin, `TexText()`,
  `Title()`, `BulletedList()` and `TexTextFromPresetString()` with no text
  raise `LatexError` ("There's no line here to end", "perhaps a missing
  \item"), because the Reference compiles an empty LaTeX body. Native
  typesetting returns a point-less mobject with no glyph children instead
  (`Title` keeps its underline), which can be filled later with `become`
  or animated in. Found by the
  Appendix-A class sweep (fm-5wq.26).

## Why

Determinism and sovereignty: text output must be a pure function of the
input closure. Host-font resolution, Pango versioning, and silent
substitution all break that; the bundled set plus owned shaping closes
the pipeline (§4). Correctness: `Text[3:7]`/`isolate=` keep the
Reference's structural conventions (non-whitespace glyphs in order, a
ligature is one submobject), so scene code indexes identically.

## Migration

- Scenes that relied on a specific host font must load it explicitly
  (`FontBook::add_family`) — the name error tells you at once.
- Exact glyph metrics (advances, kerns, line heights) differ from any
  Pango render; positions shift at the sub-em scale. Layout-relative code
  (`next_to`, `align_to`) is unaffected; pixel-locked expectations are
  not honored anywhere in FrankenManim by design.
- **The magnitude is bounded** and checked by the structural differential's
  envelopes (fm-5wq.45). Text is a different outline, not a different size
  or place:
  - `Tex`-family boxes stay within 10% of the Reference's extent and
    0.05 units of its centre, since both engines set Computer Modern.
  - `Text`, `MarkupText` and `Code` boxes stay within 25% and 0.05 units.
    The Reference's own extent depends on its host's fonts: `Text('Hello')`
    measured 1.28 and 1.049 wide on two hosts.
  - A glyph regrouped out of its text stays within 25% and 0.1 units.
  - A larger difference is a bug with its own bead. For example,
    `fm-tex-metrics-glyphs-ru72` covers fallback glyphs that widen formulas
    by up to 35%.
- Pango-only markup (arbitrary attributes, `<gravity>`, etc.) is out of
  the compatibility claim; the supported set is documented in
  `fmn_text::markup`.
- `\!`, `\:` and `\;` in mathematics are TeX's −3, 4 and 5 mu spaces. The
  Reference's default template loads `tipa`, which turns each into a
  one-token box with no space (Appendix C-27): there `x\;y` sets as `xy`,
  `x\;=y` loses the relation spacing around `=`, and `a\!\!b` does not
  compile. A formula that asks for a space gets it.
