# Authored braces and legacy single-string math

`Brace` and `LineBrace` now run the normal VMobject data, points, uniforms and
colors initialization hooks. Atlas still supplies their analytic outline and
native tip index (BN-08), not an external LaTeX glyph. The exported classes and
Tex inheritance are unchanged. `tex_string` remains compatibility metadata.

```python
from manimlib import Brace, UP, RED

class RaisedBrace(Brace):
    def init_points(self):
        super().init_points()
        self.shift(0.25 * UP)

    def init_colors(self):
        super().init_colors()
        self.set_color(RED)
```

Default `init_points()` rebuilds against the current source, direction and
padding. LineBrace obtains each public endpoint once; Atlas owns orientation
and placement. The candidate's points are written through the existing record
interface, preserving custom columns, children, live styles, saved state and
same-size views. A replacement point hook can set `tip_point_index` explicitly;
otherwise construction selects the outward extreme of its root points. Tip
and label helpers continue to follow subsequent native transformations.

## Legacy TeX

`SingleStringTex` remains a qualified-only class:

```python
from manimlib.mobject.svg.old_tex_mobject import SingleStringTex

class ShiftedMath(SingleStringTex):
    def init_points(self):
        super().init_points()
        self.shift(UP)
```

It now initializes through the ordinary VMobject lifecycle and builds glyphs
inside its public point hook. `get_modified_expression` runs once for each
actual build, including explicit live regeneration. Scribe still owns native
math/text layout, glyph outlines and UTF-8 spans. The inherited expression
normalization helpers and the named external `latex_to_svg` refusal remain
unchanged; no SVG round-trip or alternate typesetter is introduced.

`init_colors` follows glyph construction, allowing authored overrides to style
the actual family. The native white stroke and half-pixel glyph fill border are
retained rather than replaced with generic VMobject defaults. Optional height
and public left-to-right organization run after color initialization. A default
live rebuild preserves the receiver's whole-object style, not an arbitrary
per-glyph edit history. It rebuilds at the current font size; it does not replay
the constructor's height/organization epilogue or retain an old placement.

The shared string publisher replaces only its generated glyph children while
retaining other decorations and the root record schema. Its FamilyRefs ownership
marker uses the existing copier's remapping protocol, so copied, deep-copied and
pickled objects regenerate their own glyphs without duplicate translucent ink.
An authored point hook can add root geometry after `super().init_points()`.

## Failure and execution boundaries

Inputs are checked before initialization. Legacy source and normalized text
are bounded to 262,144 UTF-8 bytes, with the native typesetter's additional
limits still in force. Invalid numeric controls, locked animation data and
reentrant calls are refused. Constructors require detached receivers; explicit
regeneration can operate on a scene-owned object.

Sampling/normalization or typesetting errors preserve the prior generated
geometry. Changes to the tracked receiver records, family roots, ownership or
recipe during authored sampling are detected before publication. Arbitrary
callback effects and custom publication methods are not transactions and are
not rolled back. No second frame clock, renderer or copy protocol is added.

`brace_lifecycle.py` and `legacy_tex_lifecycle.py` are installed-native tests,
registered without removing existing runtime checks. They include actual moving
and rebuilding PNG sequences against independent native-builder controls at one
and four threads, plus copy, error, hook and record-identity cases. Targeted
native tests do not establish a fresh final-tree wheel, full-workspace pass or
cross-platform certification.
