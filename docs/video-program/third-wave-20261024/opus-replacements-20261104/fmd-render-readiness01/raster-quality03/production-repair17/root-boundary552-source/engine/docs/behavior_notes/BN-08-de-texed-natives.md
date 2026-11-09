# BN-08 — The de-TeX'd natives

**Status:** Draft. Landed in two waves. W7 (fm-y69) landed the geometry half —
`Brace`/`BraceLabel`/`LineBrace`, `SurroundingRectangle`,
`BackgroundRectangle`, `Cross`, `Underline`, `Checkmark`, `Exmark`. W7
(fm-ebl, on the fm-p5d Scribe bridge) landed the text-backed half —
`DecimalNumber`/`Integer`, the `Matrix` family, `BulletedList`, `Title`,
and the `interactive.py` control compositions.

> **Numbering.** §16.8 of the plan assigns BN-08 to "de-TeX'd classes
> (§11.6)", which is this note. A file named `BN-08-animation-contract.md`
> already occupies the number on disk, and `BN-07` is duplicated across two
> files. The plan's register is normative, so this note takes BN-08;
> reconciling the strays is filed separately rather than done here, because
> renumbering someone else's published note is a documentation act with its
> own review.

## What changed

The Reference routes a set of **non-mathematical** classes through LaTeX,
because LaTeX was already a hard dependency and therefore free at the margin.
It is not free here — it is the dependency this whole program exists to
delete — and several of those classes were never well served by it anyway:

| Class | Reference mechanism | FrankenManim |
|---|---|---|
| `Brace`, `BraceLabel`, `BraceText`, `LineBrace` | render `\underbrace{\qquad}`, then stretch the glyph | a **parametric path family** |
| `Checkmark`, `Exmark` | `\ding{51}` / `\ding{55}` via the `pifont` package | drawn paths |
| `Cross`, `Underline`, `SurroundingRectangle`, `BackgroundRectangle` | already native, but built on Tex-derived metrics in places | pure geometry over the target's box |
| `DecimalNumber`, `Integer`, the `Matrix` family, `BulletedList`, `Title` | `Tex`/`TexText` | native text through Scribe |
| `Matrix` brackets | `\left[\begin{array}…\right]` through LaTeX | the same source through **fmd-math's extensible-delimiter engine** (ADR-0005) |
| `interactive.py` controls | `Tex` labels + Pango text | Scribe text + drawn marks; event wiring lives in Proscenium (W9) |

Nothing about the mathematics changes. These classes were never doing
mathematics; they were borrowing a typesetter to draw a curly bracket.

## Why the brace is the interesting one

`Brace` is not a de-TeXing of convenience — the Reference's implementation is
**incorrect at the edges**, and in a way that is intrinsic to the approach.

It renders one fixed drawing and then resizes it. Widening is handled by a
special case over six hard-coded submobject indices of that render (stretch
the two straight runs, shift the tips outward). Narrowing has no special case
at all and falls through to `set_width(width, stretch=True)`, which squashes
the curl horizontally along with everything else. A brace over a narrow column
in the Reference is a compressed caricature of a brace over a wide one.

FrankenManim generates the brace instead. The curl — the two end hooks and the
centre point — has its own size; the straight runs absorb all the width. Three
clamps make the family well-formed at *every* positive width:

```text
cap   ≤ 0.20 · w  and  waist ≤ 0.20 · w   ⇒  cap + waist ≤ 0.40 w < 0.50 w
thickness ≤ 0.10 · w                       ⇒  thickness < cap, and the two
                                              inner hooks cannot cross
height = thickness + 2·reach > 1.2·thickness
```

The first says an end hook can never reach the centre point, so the runs never
invert. The second says the inner edge stays inside the span — without it a
brace narrower than its own stroke is inside out, which is not a hypothetical:
it was caught by the property test at width 0.01 during development. The third
keeps the centre point's inner edge from punching through its own runs.

These are proofs about the family, not tunings. `Brace` is correct at any
width by construction, which is a claim the Reference cannot make.

The tip is likewise exact. The Reference finds it by scanning the rendered
glyph for its minimum-`y` point (`tip_point_index = np.argmin(...)`); here it
is closed-form, because the shape is known rather than discovered.

## Why Checkmark and Exmark are drawn, not set from a bundled glyph

The intent was to take both from bundled faces. The faces were checked rather
than assumed:

| Codepoint | Character | Computer Modern | IBM Plex Sans | Noto Sans Math |
|---|---|---|---|---|
| U+2713 | ✓ CHECK MARK | no | **yes** | **yes** |
| U+2717 | ✗ BALLOT X | no | no | no |
| U+2714 / U+2718 / U+2715 | heavy variants | no | no | no |

A glyph exists for the check; **none exists for the cross**, in any bundled
face. Three ways out were available: substitute U+00D7 (present, but a thin
multiplication sign — wrong weight, wrong shape for `\ding{55}`), bundle a
dingbat face for two glyphs, or draw both.

Both are drawn. Drawing only the cross would leave a matched pair mismatched:
the Reference's two dingbats come from one font and read as siblings, and an
IBM Plex check beside a hand-drawn cross would not. This also follows the
precedent ADR-0005 already set for extensible delimiters, where the drawn path
is the *mainline* rather than a fallback, for exactly the same reason — the
authored glyph is not there to be had.

Both marks are built in the same unit box, so they scale and align
identically.

## Why the Matrix brackets are not a glyph stretch either

The Reference typesets its brackets the same way it typesets everything
else — a LaTeX render of `\left[\begin{array}{c}…\end{array}\right]` —
and then stretches the two halves to the grid's height. FrankenManim feeds
the *same source* to fmd-math, whose delimiter engine (ADR-0005) is
three-stage by construction: a natural glyph when it covers the target,
uniform scaling up to the 1.25× ceiling, and the parametric drawn-path
construction beyond it. Past the ceiling the bracket is *generated* — the
same precedent the drawn `Brace` set — so a 12-row matrix's brackets keep
their proportions instead of thickening into rails. The scaling stages are
load-bearing and tested at row counts 1 through 8, glyph stage to drawn
stage, with bracket height equal to grid height plus `v_buff` to 1e-9.

## Why DecimalNumber is a cache and not a typesetter

The Reference's ticking counter re-typesets its string every update and
then *becomes* the new submobjects in place when the digit count is
unchanged. FrankenManim makes the reuse explicit: each character is laid
out **once** (digits never kern in the Reference — it arranges per-glyph
submobjects with `digit_buff`), cached by character, and `set_value`
rebuilds the row from the cache, typesetting only characters it has never
seen. A ticking counter therefore typesets eleven glyphs total, ever.
Formatting is ported case-for-case, including the ones that are easy to
get subtly wrong: negative-zero suppression, the `-` → U+2013 en-dash
substitution with its vertical reseat, comma grouping with the `,` glyph
dropped half its height, and `min_total_width` zero-padding with CPython's
re-grouping overshoot (`format(123456, '08,d')` is nine wide, not eight —
verified against live Python for the whole corpus). `Integer.value()`
rounds half-to-even (`np.round` semantics); its *display* truncates, as
the Reference's does.

The native Rust scalar readout accepts `f64`. The Python portal composes
those same native readouts for general complex values, including
`hide_zero_components_on_complex`, signed imaginary components, and live
transitions between real, imaginary and full-complex displays. Matrix
constructors route complex cells through that composition rather than
float-coercing or stringifying them. No Python glyph engine is involved.
`unit` remains plain text — a leading `^` is an alignment marker, never a
TeX command; write the glyph itself (`°`), which the bundled faces map.
See [complex-valued live readouts](../python/complex-readouts.md).

Native layout work is explicitly bounded. `DecimalNumber` admits 4,096
display characters by default, covering numeric glyphs, ellipsis, and unit;
the `Matrix` family admits 4,096 entries before cloning, typesetting, or
bracket construction. `character_limit` and `entry_limit` are explicit
Rust-side opt-ins for larger scenes. Requests above the declared limit,
unrepresentable dimension arithmetic, and failed validated reservations are
typed errors rather than allocation panics. Within the admitted range,
formatting and layout retain the Reference behavior described above,
including comma-boundary width overshoot.

## The small compositions

`BulletedList` is a native bullet glyph (U+2022, which the bundled faces
*do* map — checked, like the dingbats were) beside Scribe text, arranged
with the Reference's buff and alignment; `fade_all_but` ports the
Reference's scaling formula verbatim over the family-recursive style
surface. `Title` joins its parts into one layout and re-groups the glyph
children by source-byte span through the native span map, so part *i*
stays addressable without the Reference's render-twice isolation. The
`interactive.py` controls are compositions plus their value surface; the
mouse/keyboard wiring is Proscenium's (W9), not the library's.

## Migration guidance

- **Metrics differ.** These constructions are ours, so their exact
  dimensions differ from the LaTeX-derived originals. Scenes that hard-code
  offsets measured against a Reference brace's height, or against a
  `\ding{51}`'s width, need re-measuring. Scenes that position with
  `next_to`, `get_tip`, `put_at_tip`, or the positional API — which is nearly
  all of them — need no change.
- **Dimensions are bounded, not free.** A native keeps the box of the glyph
  it replaces. Brace, Checkmark/Exmark, Matrix, DecimalNumber and Integer
  roots stay within 10% of the Reference's extent and 0.05 units of its
  centre; the structural differential's envelopes check this (fm-5wq.45).
  Two natives are outside that bound today, each tracked as a bug, not
  covered by this note:
  - the drawn marks are about 3x the glyph's size (`fm-5wq.52`);
  - the parametric brace is 39% deeper than `\underbrace` (`fm-5wq.53`).
- **Scaling behaviour is better defined, and therefore different.** A brace
  much narrower or much wider than its natural size now keeps its curl's
  proportions, and a tall matrix's brackets keep their weight. If a scene
  relied on the Reference's distortion for a visual effect, it will look
  different — better, but different.
- **Stroke tapers are reproduced.** `Cross` and `Underline` keep their
  variable stroke widths (`[0, 6, 0]` and `[0, 3, 3, 0]`), resized onto the
  path by linear interpolation the way `set_stroke(width=[...])` does. The
  taper is the visual character of those classes, not a detail.
- **No LaTeX escape hatch.** There is deliberately no way to ask for the old
  rendering. `pifont` is gone; so is every other TeX package.
- **`DecimalNumber.unit` is text.** A scene passing `unit="^\\circ"` gets
  the alignment marker and then literal characters — write `unit="°"`
  instead (or `unit="^°"` for raised alignment). Python callers can pass a
  complex value directly; native Rust callers compose scalar readouts.
- **Rust-side typing is explicit.** The Reference's runtime dispatch in
  `Matrix` (float → `DecimalNumber`, str → `Tex`) is a constructor choice
  here: `DecimalMatrix`/`IntegerMatrix`/`TexMatrix`/`MobjectMatrix` own the
  branches, and the base `Matrix` takes built entries. Out-of-range row and
  column access returns `Option` rather than raising `IndexError`.
- **Native dimensions are bounded.** Ordinary labels and matrices need no
  change. Extremely large generated surfaces must opt into a larger
  `character_limit` or `entry_limit` and handle the typed resource error;
  host-size overflow remains a refusal at every limit.

## Evidence

- `crates/fmn-library/src/brace.rs` — the parametric family, with
  `braces_never_self_intersect_at_any_width` checking all three invariants
  across widths from 0.001 to 200, and `the_curl_keeps_its_size_as_the_brace_widens`
  pinning the property the Reference lacks.
- `crates/fmn-library/src/matchers.rs` — the matchers and marks, with the
  cmap survey recorded in the module documentation and
  `both_marks_are_closed_shapes_in_the_same_unit_box` holding the pair to a
  matched size.
- `crates/fmn-library/src/vmobject.rs` — `with_stroke_profile` and
  `resize_with_interpolation`, the taper mechanism; `map_style_deep`, the
  family-recursive style surface `fade_all_but` rides.
- `crates/fmn-library/src/numbers.rs` — the glyph cache (`typesets()` is
  the recycling witness: an update inside the cached alphabet typesets
  nothing, test-proven), the formatting corpus, and exact-limit,
  just-over-limit, and `usize` boundary tests.
- `crates/fmn-library/src/matrix.rs` — the delimiter construction, with
  bracket height held to grid height plus `v_buff` across the glyph and
  drawn-path stages at row counts 1–8, plus checked entry, partition,
  accessor, and child-capacity bounds.
- `crates/fmn-library/src/special_tex.rs` — the bullet and title
  compositions; `fade_all_but`'s formula port.
- `crates/fmn-library/src/controls.rs` — the control compositions and the
  tracker binding (`add_scalar_control`), with the W9-deferred event hooks
  named in the module documentation.

- `crates/fmn-python/tests/decimal_authoring.py` and `complex_matrix.py` —
  complex formatting, mode changes, anchors/styles, copy identity, combined
  resource limits and failure-atomic native cell construction.
- `crates/fmn-python/tests/complex_readouts_render.py` — live TeX readouts and
  matrix cells in six native PNG frames, byte-equal at 1/4/16 threads, with
  failed-generation publication cancelled.
