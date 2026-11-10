# CC2 review 747: final chapter 09 (boundary) pictures, independent

CC2 (RubyRaven), 2026-11-10. This is an independent picture review of the selected stills from the final 4K movie. I
read root's records only for their anchors, then judged each anchor from the pixels. No source edit, SSH, build,
render, synthesis, Beads change, commit or deletion took place. This review makes no listening, motion, fine-sync, AV or
release claim.

| Input | sha256 |
|---|---|
| Final movie (per `pictures587.json` and root's 746) | `f7fe8f2620daab4814075e06a56210c713761803da7ba772473fdacb8687a88a` |
| `pictures587.json` in root's local review folder 746 | `1d7751b08cb62e316fc55ff32ed898ad45bbf5637117cc12a19fd97c9647917d` |
| `root-boundary-final-review746.json` | `c05d90297c3ca0fea7840053c068eada3fbacd2af8e845a0e39ca0a5b47e05b9` |
| `root-boundary-review-followup705.json` | `25ce602848d30b1a08d945a1a1c5307c27e75609702bf8954f3aca465dbae84c` |
| `cc3-native-boundary-preview-review694.md` | `1fde30cc0ff97a6e219d11c0d8b4eeac43d017b87bcb2a270d0bead03fe89503` |
| `root-boundary607-gate.py` (lines 82-96 read) | `9293391ff356226af4b7689a50a97308fa6c933b5d17870afafcdf7d53c8b2a1` |
| Bound `boundary.rs` under `root-boundary552-source/` | `bda7d931498b23bd3203cd0c6106de69dc0a8c5e160e738d698376322ff39e53` |

## Result

**One concrete picture defect, B1, which I count as blocking under 607. Everything else is acceptable.**

B1 covers about 71 to 85 s. The voice and the formula say the inner polygon is walked clockwise and then
counterclockwise. In the picture, the inner polygon's direction cannot be read, and its name `p` sits beside the outer
polygon. Nothing false is shown, and the formula text carries the step, so I rate it Medium, not High. It is still a
specific defect visible in the pixels at the anchors, not a matter of taste. I would not publish this step as it is.

Root's other findings:
- F1, F4, F5 and F6 are acceptable cosmetic limitations.
- F3 is no defect.
- O3 is resolved; I agree with root.
- F2 splits in two. Its `p` half is part of B1. Its rho half is acceptable on its own, though only narrowly.

So, on my evidence, `concrete_picture_defects_observed` cannot honestly be `[]` for this movie. 607 line 94 requires
exactly that, and I do not propose weakening it. Root can repair the source and re-render, which then needs a new
picture pass. Or root can dispute B1 with evidence. Either way, root's 746 sentence "no high or medium picture blocker
identified" does not match my reading.

A related fact for whoever writes 607's record: 746 does not have the keys 607 reads. 607 reads
`extraction_manifest_sha256`, `concrete_picture_defects_observed` and `observations` (lines 92 and 94). 746 has
`sampling_sha256` and `findings` instead.

## What I viewed

All evidence is native, from the final movie as extracted by root.
- I re-hashed all 313 pictures and 40 sheets against `pictures587.json`: 0 mismatches.
- Every picture is 3840x2160 and every sheet is 1280x1536.
- I read all 40 sheets as images.
- I cut native 4K crops with nearest-neighbour scaling, at 1:1 or enlarged 2 to 5 times. They covered these pictures:
  12, 55, 129, 137, 139, 142, 144, 145, 147, 148, 150, 151, 153, 155, 166, 186, 191, 193, 209, 212, 213, 216, 233, 261,
  271, 277, 300 and 307.
- Every detail anchor root listed in 705 and 746 is among them or on a sheet I read.

Picture times come from `selected_frames` at 60 fps. I read the bound `boundary.rs` to name what is drawn: lines 1 to
354 and 466 to 885.

I also recomputed every number in the closing section independently, in plain Python. All match what the screen
prints:

| Shown | Computed |
|---|---|
| average of 8 | 1.0039216 |
| average of 16 | 1.0000153 |
| r = 2, average of 16 | -0.0000153 |
| limit 0, and 1 - 0.5 - 0.5 = 0 | residues 1, -1/2 and -1/2 |
| c0 to c4 | 1, 0, -1, 0, 1 |
| 60 terms about 1, at 2 | 0.2000000002 |
| 60 terms about 1, at 2.6 | 212.18 |
| interval (-0.41, 2.41), radius √2 ≈ 1.414 | 1 ± √2 |

## B1, Medium, blocking: the inner polygon step shows neither the inner direction nor the inner name

**Where:** pictures 146 to 161, from 71.0 to 85.0 s.

**The heads.** The inner heads are drawn by `side_heads(stage, &cp, &polygon(SMALL_R, SIDES), false, output())`. They
use `output()` green, the same green as the inner polygon and the small circle beneath it. Each head is 0.12 scene,
about 32 native px. It merges into the green band and reads as a notch.
- At native 1:1 (picture 148, 72 s), the inner ring looks like a jagged green ring, and no direction can be read.
- Enlarged 3 times, the notches in picture 148 (72 s, clockwise) and picture 155 (79 s, counterclockwise) are visibly
  mirrored. That is the only scale at which the flip shows.
- At 1080p a head is about 16 px. I did not measure phone or 1080p viewing; the 1080p figure is inferred from native
  size.

The outer polygon's yellow heads beside them are clear at 1:1. So the viewer sees one directed polygon and one
undirected ring.

**What carries the direction instead.**
- The inner direction is legible in the picture only through the yellow boundary walks. In the samples these run from
  69.467 to 70.000 s (pictures 141 to 144) and are gone at 70.667 s (picture 145).
- From 71 s, `∮_P g dz + ∮_{p clockwise} g dz = 0` and the green heads are what remain.
- From about 77 to 82 s, `∮_P = ∮_p` and the caption "both walked counterclockwise" appear. At this point the inner
  heads have reversed, which no one can see.
- The 09_1 reference row has no phrase marks. So I cannot say which picture is on screen when the voice says "the inner
  polygon, clockwise" or "both walked counterclockwise".

**The name.** `p` names the inner polygon. It is drawn in ink at `POLYGON_NAMES_AT` (-0.445, -0.445), which is |z| ≈
0.63.
- That is about 0.13 z units, roughly 80 native px, beyond the outer polygon's lower-left vertex (radius 0.5).
- It is about 0.45 z units, roughly 270 native px, from the inner polygon (radius 0.18).
- `P` is placed symmetrically at the upper right. On screen, the two names sit at opposite corners of the same yellow
  octagon (crop of picture 148).
- `p` is not green, and the formula's `p` is not coloured either.

The tie to the inner polygon rests on three things: lowercase for the small polygon, term order, and the voice saying
"outer" and "inner".

**Why I count it as blocking.**
- The step's statement is about the inner polygon's direction, and for about 14 s the inner polygon has neither a
  legible direction nor a nearby name.
- A viewer who reads only the picture sees two names on one polygon and a ring with no direction. They must take the
  flip from the formula text.
- This is the step the chapter's proof turns on: the polygons' cancellation becomes equality.

Either half alone would be Low to Medium. Together, at this step, I would not pass them as cosmetic.

**Repair sketch, for root to decide. Not done by me.**
- Draw the inner heads in a contrasting colour, for example the yellow the inner boundary walks already use, or ink.
  Or enlarge them and set them just inside the green band.
- Alternatively, keep the inner boundary walks on screen through the "clockwise" phrase.
- Put `p` beside the inner polygon, for example in the annulus between radius 0.18 and the dashed 0.3 circle, or at
  least colour it green.

Any repair means new source, a new render and a new picture pass.

## Root's findings and the other anchors

| Item | Pictures and times | What I saw at native scale | Verdict |
|---|---|---|---|
| F2, rho | 43 to 119 s; crops 148, 193, 209 | Green ϱ sits at about native (3476, 1480), outside the yellow loop and about 370 px from the small green circle. From 110 s, `∮_{\|z-a\|=r} = ∮_{\|z-a\|=ϱ}` and "for every small ϱ" (sampled 114.8 to 117.2 s) appear while ϱ is still there. r is never labelled in the close-up, and the formula's ϱ is white. ϱ moves beside the small circle from about 119 to 120 s. | Acceptable Low, narrowly. The label's green matches the small circle and the green "small circle fixed" caption, and the yellow loop is the obvious r. Still a placement weakness. If B1 is repaired, starting ϱ where the shrink updater already puts it, (r + 0.06, -0.12), costs one position. The ϱ glyph form is used consistently in the label and every formula, so that is not a defect. |
| F1 | 166 (90 s), 186 (104 s) | "inside the dashes" sits about 8 native px above the top border of the "same endpoints" card. It is readable, with no contact. It belongs to `\|g\| ≤ M` about 100 px above, but proximity groups it with the card. | Acceptable Low. |
| F3 | 186 (104 s), 191 (108 s) | "→ 0" ends about 70 native px (0.26 scene) from the right edge. Fully visible. | No defect. |
| F4 | 233 (157 s), 261 (203.15 s) | The "r = 1/2" pill's right border touches or nearly touches the yellow circle. The text is untouched. | Acceptable cosmetic. |
| F5 | 212 to 217 (123 to 128 s) | With the circle at `MID_R`, its top stroke touches the top of `a` from about 120 to 127 s. During the 1 s shrink near 127 s, the stroke crosses the middle of `a`. From 128 s, `a` is clear. The red hole ring stays visible. | Acceptable cosmetic. |
| F6 | 12 (28.5 s), 55 (34.9 s) | The leaders are present at 28.5 s, before the close-up fades in (from about 28.75 s). They end in blank space at native (2568, 797) and (2568, 1877), about 240 px left of the close-up circle and beyond its top and bottom. `zoom_links` aims at the close-up frame's left corners, and that frame is never drawn. They read as a zoom cone and point at nothing false. | Acceptable cosmetic. |
| O3 | 129 (58 s), 137 (66 s), 139 (68 s) | At 1:1 every triangle walk is distinct and correct. Outer sides run counterclockwise and inner sides clockwise (yellow). The red pair on the spoke above `a` is plainly opposite: up on the left, down on the right. It is busy but legible. | Resolved. I agree with root. |
| O1 (694) | 186, 213 | The closing `\Big\|` overlaps the top right of the italic z in "dz\|". This happens in the arc bound (95 to 103 s), the sum bound (103.8 to 108 s) and the shrink bound (121 to 128 s). The z stays readable. | Acceptable cosmetic. |
| Focus | 233 (157 s), 256 (201.15 s), 271 (223.4 s), 307 (273.2 s) | The "opposite samples share a value" pill is opaque and readable but dim, and sits on the plot's bottom border. The "precisely:" card is readable. The graph's x is whole, about 0.2 scene from the edge. The √2 circle about 1 stays inside the plate. | Acceptable. |
| Late contacts (not in 746) | 277 (228.2 s), 300 (260.6 s) | The small yellow circle is tangent to the upper left of `a`. Then, while the √2 circle grows, its stroke crosses the left of `a`; the large circle is clear by 261.6 s. | Acceptable cosmetic, same class as F5. |

Sheets 001 to 040 show nothing beyond the above.

## Not established

- Motion between samples. These are 313 of 16,504 frames, and stretches such as 2.6 to 28.0 s are unsampled.
- Audio, narration and sync, including which words coincide with B1's pictures.
- Legibility at 1080p or on a phone; only the native size was seen, and smaller sizes are inferred.
- Colour on real displays.
- Anything about other chapters or the whole film.

This review does not accept the picture for 607, does not accept AV, and does not accept release.
