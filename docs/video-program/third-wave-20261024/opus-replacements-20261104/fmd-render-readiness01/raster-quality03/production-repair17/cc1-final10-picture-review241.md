# Final10 picture review 241 (cc1): 10_rigidity, native 4K stills

**Verdict:** every mathematical claim on screen holds, and nothing contradicts the voice. There
is **no High** finding. There is **one Medium**: the colour plates never visibly fade. They pop
on and off, and the chapter ends on a fully visible plate. There are **three Lows** and some
observations that need no action. This is a stills review only; listening, fine sync, motion
review and independent final AV acceptance remain open.

## Inputs verified

| Item | Value |
|---|---|
| Movie `final10-terminal155/10_rigidity.mp4` | sha256 `ce9855429a89440c77b82681ac322b8a4f45388b5083c7fdf0cee4cedf59c8f7`, 7,442 frames, 60 fps (124.03 s) |
| Same movie hash recorded in | `render-summary.json`, `pictures-reviewed155.json`, `sampling.json` |
| Pictures `narration_lab/.../complex-final10-pictures155/pictures/` | 154, all 3840×2160, **all 154 sha256 match `sampling.json`** |
| Sheets `sheet-0001…0020.jpg` | 20 sheets of 1920×2260 (8 tiles each, sheet 20 has 2); **all 20 sha256 match** |
| `sampling.json` | `c0580f88…`; frames strictly increasing; seconds = frame/60 for all |
| `select.filter` | `d6abd276…`; its 154 `eq(n,…)` frames equal the sampled frames exactly |
| `resource-receipt.json` | `842be36c…`; exit 0; inputs `290a9432…` identical before and after (01:23:17–01:23:49 UTC) |
| `command.log` | **empty** (`e3b0c442…`). The extraction command is recorded only in the receipt's `command` list. This is a receipt gap, not a picture defect. |
| cues / timeline / cuelog / layout | `a58e14f5…` / `c02a7e7b…` / `1da8a65a…` / `e3b0c442…`. The layout lint is empty: zero rows. |
| Source read | `explainer_corpus/complex_analysis_visual/src/chapters/rigidity.rs` `c577e2c0…`, identical in canonical and in `production-source-preserved214`. Root's records give the render binary as build19 `f4836eee…`; I did not prove that binary-to-source link myself. |

Cues: 10_1 0.600 · 10_2 16.733 · 10_3 41.100 · 10_4 61.783 · 10_5 79.883 · 10_6 103.633 s.
The prediction mark is at 74.633 s (frame 4478) with a 5 s think window. The music breath is
at frame 7262 (121.03 s) for 2 s.

## Coverage

- **Sheets:** I read all 20 sheets, covering all 154 pictures in order, and logged every
  tile.
- **Native crops** (cut from the hash-checked pictures):
  - cross and 0 card at 81 and 84 s;
  - limit point q at 13 and 16 s;
  - value rows at 36 s;
  - smooth-function formula at 44 s;
  - dim "why" lines at 12 s;
  - red region note at 84 s;
  - plate foot at 87 and 88 s.
- **Pixel measurements:**
  - plate foot against the lower card at 87, 88 and 103 s;
  - contrast of the dim text, the red note and q against the fill;
  - mean plate brightness at 19 sampled times around every plate show and leave.
- **Not done:** no extraction, decode, render, build or listening.

## Findings

### M1 (Medium): colour plates pop on and off instead of fading; the chapter ends on a full plate

`Painter::show` and `Painter::leave` (`draw.rs:667`, `draw.rs:688`) fade the plate's
`RowStrips` group in or out with `fade_in_all` and `fade_out_all`. In these stills the plate
is always either absent or at full strength. I measured mean brightness over plate areas kept
clear of dots and labels:

| t (s) | frame | What the source is doing | Mean brightness |
|---|---|---|---|
| 61.783 / 62.000 | 3707 / 3720 | 10_3 fading out; no plate yet | 16.8 / 16.7 (background) |
| 62.283 | 3737 | first frame of `painter.show(1.0)` (rigidity.rs:428) | **129.46 (full)** |
| 62.783 / 63.000 / 64.000 | | show should be about half done / done | 129.46 / 129.46 / 129.51 |
| 104.000 | 6240 | 0.4 s overlay fade (rigidity.rs:506) just ending; leave not yet started | 129.46 (full, as expected) |
| 104.133 | 6248 | `painter.leave(0.5)` (rigidity.rs:507), about 20% done | **129.45 (full)** |
| 104.633 / 105.000 | 6278 / 6300 | `sine.show(1.0)` (rigidity.rs:533), about 10% / 47% | **130.69 / 130.69 (full)** |
| 123.000 | 7380 | breath, before the final fades | 129.38 |
| **124.000 / 124.017** | **7440 / 7441 (last frame)** | `sine.leave(0.5)` (rigidity.rs:619), about 93–97% done | **130.68 / 130.68 (full)** |

The vector overlays around the plates do fade normally: the 17.0, 62.0 and 123–124 s samples
show partial or complete fades. Their timing also confirms the schedule in the table:

- The sine axes are mid fade-in at 105.633 s, which is right after a 1.0 s show starting near
  104.53 s.
- The breath at frame 7262 lasts 120 frames. Add the 30-frame overlay fade and the 30-frame
  leave and you get 7442 frames, the chapter's exact length. So the leave occupies frames
  7412–7441.

Consequences:

- **62.283 s:** the sin(1/z) plate appears at full strength in a single step. Its axes and
  title fade in about 1.3 s later; they are present at 63.783 s.
- **Between 104.133 and 104.633 s:** the sin(1/z) plate is replaced by the sine plate as a
  hard swap in the same place.
- **Chapter end:** frames 7440 and 7441 still show the whole sine plate, with no overlays.
  In the joined master the plate therefore disappears in one frame at the join to
  chapter 11, unless the join adds its own transition (not checked).

I don't know the mechanism. It could be the `RowStrips` paint or how the engine handles
opacity on the strips. Either way it probably affects every Painter plate in the film; I did
not check other chapters. A motion check would confirm it: look at frames 3737–3800,
6242–6340 and 7412–7441, and at the chapter 10→11 join.

### L1 (Low): the lower "precisely" card covers the foot of the sin(1/z) plate

- **When:** 88.000–103.633 s (frames 5280–6218), in 16 sampled pictures. The card is absent
  at 87.000 s; it is created by `precisely_now` (rigidity.rs:495) and leaves in the 10_6 fade
  (rigidity.rs:506).
- **Measurement:** columns x = 700, 1164 and 1600 agree.
  - At 87 s, the plate fill ends at y ≈ 1832, its grey border runs 1834–1838, and the
    background starts at 1840.
  - At 88 s and 103 s, the plate fill ends at about 1826, the card edge sits at 1828–1832, and
    the card fill starts at 1834.
  - So the card hides the plate's whole bottom border and about 5 px of the picture, roughly
    8–10 px in all.
- **Intent:** the source comment says the plate is "centred, so that its foot stops above
  where a two-line lower third starts" (rigidity.rs:149). The empty layout lint did not catch
  this.
- **Possible fix:** raise or shrink `sin_recip_plane` by about 0.04 scene units.

### L2 (Low): heavier upright ≈, −, ≤ next to Computer Modern

This is the same glyph family as final08 review 219 L1, so probably a kit-level font mapping.
Crops:

- 36 s: the rows "sin²(3i) ≈ −100.36" and "cos²(3i) ≈ 101.36". The ≈ and the minus are
  heavier.
- 44 s: the formula "e^{−1/x} (x > 0), 0 (x ≤ 0)". The ≤ and the superscript minus are
  heavier.
- 12 s: the "f − g" minus in the dim "why" lines is short and heavy, close to a hyphen.

The same glyphs recur at 115–123 s ("between −1 and 1", "sin(3i) ≈ 10.0 i"). All are legible.
I judged this from pixels only, with no font inspection.

### L3 (Low): the limit point q is gold on the gold fill

- **When:** 13.000–16.733 s, the piling-up dots inside U (rigidity.rs:215–221).
- **Measurement:** q measures (251, 220, 110) against the nearby fill (206, 183, 88), a
  contrast ratio of **1.48:1**. The white dots that pile up are clearly visible, and the
  points with k ≥ 4 overlap q, which is drawn on top.
- **Why it matters:** q is the "point inside U" that the narration and the text "points
  piling up at a point inside U: enough" refer to. It is findable at the end of the trail,
  but it is the weakest element in that picture.
- **Possible fix:** give it the dark ground ring the 10_4 zero markers use.

### Observations (no action needed)

- **O1:** U and its label appear at 1.0 s, during "astonishingly rigid", before "connected
  region" is spoken.
- **O2:** from 28 to 41 s the spreading band is drawn over the "agree here" card and tints its
  dark backing olive. The text stays readable.
- **O3:** the right half of the screen is empty from about 42 to 52 s, while only the formula
  and graph are up.
- **O4:** the negative zeros −1/(kπ) are drawn but not listed. The list and the voice name
  only the positive ones; both are true.
- **O5:** in "|sin z| =  1.68" (119 s) there is a wide gap between the label and the live
  value.
- **O6:** the red cross at 0 (81–103.6 s) shows clearly at native resolution, about 64 px
  across, and just clears the innermost zero markers. At sheet scale it is too small to see.
- **O7:** the red note "0 is not in the region: sin(1/z) is undefined there" is small (18 pt)
  but measures 5.3:1. The dim "why" lines measure 6.4:1.

## Mathematical claims and conditions checked

| Where | Claim on screen | Check | Holds |
|---|---|---|---|
| 8–17 s | "f = g on a little curve ⇒ f = g on the whole connected region" | Curve drawn inside U. The card leaves out "holomorphic", but the voice ("two of them") and the 10_5 precise card supply it. | yes |
| 8–17 s | "on the curve, every derivative of f − g is 0, so f − g = 0 on a disk; step on, disk by disk" | Zeros accumulate at each curve point, so all derivatives vanish there. The power series is zero on the largest disk in U. Each next centre lies inside the previous disk (`chain()`), and all disks stay inside U in every frame. | yes |
| 13–17 s | "points piling up at a point inside U: enough" | Points q + t(1, 0.35) with t = 0.9·0.62ᵏ accumulate at q, and q is inside U. | yes (see L3) |
| 22–41 s | sin²x + cos²x = 1 on ℝ ⇒ on ℂ; sin²(3i) ≈ −100.36, cos²(3i) ≈ 101.36, sum = 1.000000 | −sinh²3 = −100.3578 and cosh²3 = 101.3578. Both sides are entire. The voice says "about minus one hundred" and "about one hundred and one". | yes |
| 42–62 s | e^{−1/x} (x > 0), 0 (x ≤ 0); zero on a half line yet not zero; every derivative at 0 is 0; series 0 + 0x + 0x² + ⋯ = 0 | The standard flat function. The red gaps at x = 0.5…3.5 have height f(x), the series' miss. The curve shape is right (about 0.78 at x = 4). | yes |
| 65–103 s | zeros of sin(1/z) at 1/π, 1/(2π), 1/(3π)… = 0.318, 0.159, 0.106, … → 0 | Values correct. The markers sit at ±1/(kπ) on the plate; the first three are checked in display px. The plate's pinwheels coincide with them. 0 itself is unmarked. | yes |
| 72–79.9 s | prediction "forced to be 0?" | The think ring runs from 74.633 s and is gone by 79.883 s. No answer shows until the cross at 80.883 s, which is in 10_5. | yes |
| 81–103 s | "0 is not in the region: sin(1/z) is undefined there"; card "agree on points piling up at a point inside the region ⇒ agree on the whole connected region" | The region is ℂ∖{0}, which is connected; the zeros accumulate only at 0. The condition is the one the theorem needs. | yes |
| 88–103.6 s | precisely: "if f and g are holomorphic on a connected open set U and agree on a set with a limit point in U, then f = g on U" | A correct statement of the identity theorem. | yes |
| 106–123 s | sine's zeros labelled −π, 0, π; "holomorphic on the whole plane and bounded ⇒ constant"; "on the real line: between −1 and 1" | Pinwheels sit at ±π, matching within 1 display px. The real-axis hues follow the sign of sin x. Liouville's theorem is stated correctly. | yes |
| 119–123 s | readout \|sin z\| = 1.68 while climbing; "sin(3i) ≈ 10.0 i" | At 119 s the dot is at Im ≈ 1.29 by pixels, and asinh(1.68) = 1.30. sin(3i) = i·sinh 3 = 10.018i. | yes |

## Limits

- **Stills only:** 154 sampled frames out of 7,442, mostly one per second plus cue and mark
  samples. Anything that happens between samples is unobserved: brief overlaps, the exact
  frames of the plate pops, and the 10→11 join.
- **No audio:** no listening, fine sync, loudness check or independent final AV acceptance.
  Every timing remark rests on the cue and timeline files and the source, not on audio.
- **M1's mechanism is unproven.** The table shows the plates' brightness, not why the fades do
  not appear.
- **Independence:** before my review I saw root's `pictures-reviewed155.json` summary, because
  it sits in the terminal folder I was asked to verify. Its "no concrete persistent clipping
  or overlap" differs from L1, and M1 is not mentioned there.
- I made no source, render, Beads, TODO or canonical change. This file is the only one I
  wrote.
