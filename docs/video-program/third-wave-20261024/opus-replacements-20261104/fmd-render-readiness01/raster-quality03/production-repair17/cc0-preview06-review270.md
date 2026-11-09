# cc0 preview06 review 270: chapter 06, one derivative (native 540p preview243)

## Subject and gate

- **Gate.** I waited until `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-preview243-copy265/resource-receipt.json` showed: finished 2026-11-08T05:36:17.721810Z, exit 0, `inputs_sha256` = `final_inputs_sha256` = `aa5d96a7b0e05c69d47624fdedf9a877d5cd670c69c9fa2d0cc491d1b3bf1ac9`, `resource_stop` null. Paths below are relative to this file's folder. I opened nothing in `preview243-review265/` before that. I copied nothing.
- **Film.** `preview243-review265/preview/06_one_derivative.mp4`, sha `5e45d72da91701c9c7c346b98ded8491c13ba1befed6be1b6301c1c6d33ed62d`, re-taken after the checks. It is 960×540 at 60 fps: 10,030 frames, 167.17 s. Its job receipt shows exit 0, inputs equal to final, and no resource stop.
- **Tables.** The cue table `ee8eb35a…` is identical to final06's. The layout table `e3b0c442…` is empty: 0 lint findings for 06.
- **Source.** Binary `review-repair238`, built from `one-derivative-review-repair238.rs` (`7a4cc01a…`) and `mod-review-repair236.rs`. Narration binding rebind242, as root recorded it. I did not re-derive the binding.
- **Extraction.** `preview243-review265/06_one_derivative/sampling.json`, sha `34ee8c6ae24ccd33e234ac2b1b27cf5441fae1087af8f53f4b4436942efd22ba`. It lists 376 pictures (frames 0–10029, at most 30 frames apart, 0 hash mismatches) and 47 sheets.
- **Independence.** I wrote review224 and the 06 repair note and candidate 231, which 238 corrects. This review is independent of root's build, render and extraction, but not of the repair's design. cc3's review235 is the independent source review.

## What I viewed

- **Sheets.** All 47 sheets, `sheet-0001.jpg` to `sheet-0047.jpg`, each read as an image. Together they hold all 376 sampled pictures; sheet k holds samples 8k−7 to 8k.
- **Native crops.** I read enlarged crops (1–4×, nearest neighbour) of 12 pictures:
  - frame 600 (`frame-000025.jpg`)
  - 1680 (`…065`)
  - 2940 (`…111`)
  - 3000 (`…113`)
  - 3060 (`…115`)
  - 3240 (`…121`)
  - 3600 (`…134`)
  - 4320 (`…161`)
  - 4440 (`…165`)
  - 6720 (`…252`)
  - 7470 (`…278`)
  - 8778 (`…330`)
- **Pixel measurements.** I measured these frames by script, without reading them as images:
  - 600, 900, 1200, 1290;
  - every sample from 1320 to 1890 (22.0–31.5 s);
  - 2910, 2940, 2970, 3000, 3030, 3060, 3090, 3120, 3240, 3600, 4320, 6720.
- **Where the crops went.** The crops were temporary files outside the repo. I did not keep them.

## Result

**No blocker and no Medium.**
- **Review224.** F1 (shared scale) is fixed on pixels. F2 is handled on screen as designed, but the spoken words are unchanged. F3 remains.
- **Review235.** Measurement clears L1. L2 and L3 appear as predicted and stay Low.
- **New.** Two cosmetic edge Lows, R1 and R2.
- **Notes.** Review224's N1 remains.

| Item | Severity | Status in preview243 |
|---|---|---|
| review224 F1, output at 0.748 of input scale | was Low–Medium | fixed: both planes at 97.0 px/unit |
| review224 F2, no f′ ≠ 0 condition | was Low–Medium | on screen at 06_3 and 06_7; voice words unchanged |
| review224 F3, heavy − and × | Low | present |
| review235 L1, z₀ label near "from above" | Low | cleared: at least 12 px at 540p |
| review235 L2, notes pair with legends | Low | present |
| review235 L3, "both pictures" with the inset | Low | present |
| R1, change label 3 px from the frame | Low, cosmetic | new to this review |
| R2, z₀ label 2 px from the Im axis | Low, cosmetic | new |
| review224 N1, red verdict into the squash | note | present |

## Checks

### Shared plane scale (review224 F1): fixed

At 540p the two frames sit at x 80–82/391–393 and x 486–487/796–798, with y 154–155/464–466. Each is 310.5 px wide for 3.2 units, so both are 97.0 px per unit. The axes sit where the plane ranges put them: input Im at x 139–140, input Re at y 406–408, output Im at x 641–642.

**Frame 3600 (60.0 s).**
- **Input square.** The edges are at x 212.5/260.5 and y 285.5/334.5, so the side is 0.50 units.
- **Teal "exact stretch and turn" diamond.**
  - Vertices at x 546/738 (row 311) and y 214/406 (column 642).
  - Half-diagonal 0.99 units; the exact value is 1.000.
- **Green image, against the exact z² image of the square.**

  | Point | Measured | Exact |
  |---|---|---|
  | Side vertices, row 323 | x 547/737 | ±1 at Im 1.875 |
  | Top vertex | y 201.5, Im 3.13 | 3.125 |
  | Bottom vertex | y 393.5, Im 1.15 | 1.125 |

- **Drawn ratio.** The square is drawn 2.83 times larger, matching "stretch 2.83".

**Frame 600 (10.0 s).**
- The h tip is 0.44 units at 35.5°; the exact value is 0.450 at 34.4°.
- The change-arrow tip is 1.471 units at 78.4°; the exact value is 1.471 at 78.0°.
- The drawn ratio is 3.34 against the exact 3.27. The difference is h's short antialiased tip.

**Scale note.** "both pictures drawn to the same scale" is on screen from 0.5 s to 70.5 s (sheets 1–20). At 71.0 s (frame 4260) the zoom note has replaced it. The 0.3 s hand-over itself falls between samples.

### Nonzero-derivative qualifiers (review224 F2)

- **06_3.**
  - "unless the derivative is 0" appears at 48.5 s (sheet 14), at the top left of the output frame, 10 px inside it. "a stretch and a turn" sits right of the Im axis.
  - From 52.5 s the claim fades with the fan. From 53.0 s the note stands above "bigger here: stretch 2.83".
  - Both notes leave in the 06_5 fade, 75.5–75.93 s (sheets 21–22).
- **06_7.**
  - The note sits under "a stretch and a turn: no squash, no reflection" from 111.88 s to about 128.85 s (sheets 32–36).
  - It stays through the matrix values (118.0 s) and "= ×(2+2i)" (124.5 s).
  - Its ink ends about 56 px above the bottom edge (frame 6720).
- **Legibility.** All the grey notes are legible at native size. Measured contrast against the background:
  - 06_3 qualifier: 5.3:1
  - "bigger here": 4.7:1
  - zoom note: 5.1:1
  - 06_7 qualifier: 4.6:1, the smallest and dimmest.

  I did not test phone size.
- **Spoken words.** The cue table is identical to final06's, so the words of 06_3 ("still square") and 06_7 ("must be a stretch and a turn") are unchanged. The condition is on screen only. No audio heard or checked.

### Fan (06_3)

Output tips, measured about the centre 2i. The exact values are 65°, 155°, 245° and 335° at length 0.849.

| Time | Frame | Angles | Length (units) |
|---|---|---|---|
| 48.5 s | 2910 | 22.6°, 111.8°, 197.0°, 289.5° | 0.33–0.36, copies of the input |
| 49.0 s | 2940 | about 21° | 0.64–0.66, stretching |
| 49.5 s | 2970 | about 31° | 0.81–0.84, turning |
| 50.0 s | 3000 | about 56° | 0.82–0.84 |
| 50.5–52.0 s | 3030–3120 | 64.9°, 154.3°, 244.6°, 336.1° | 0.82–0.83 |

- **Settled lengths.** The settled tips fall short of 0.849 because of antialiasing at the point.
- **Input fan (frame 3060).** 23.5°, 111.7°, 197.4° and 289.2° at length 0.28. The exact values are 20°, 110°, 200° and 290° at 0.3.
- **Label.** The "×f′(z₀)" label is up from 46.0 s.
- **Cosmetic.** The input pinwheel is small (29 px arms at 540p) and its heads nearly touch at the hub. It is readable at native size.

### Zoom labels (06_4)

- **Readout (frame 3600).** "stretch 2.83, turn 45°" is up from 60.0 s. Its ink ends at x 785 and the ratio inset begins at x 810, so it is 25 px clear.
- **Zoom note (frame 4320, 72.0 s).**
  - "zoomed in about 2i / to keep the image's size" is up from 71.0 s.
  - It starts 10 px right of the Im axis and ends about 18 px inside the frame.
  - Its top is 17 px below the diamond's bottom vertex.
  - The shrink runs from 71.0 s to 74.0 s.
- **Gap readout (frame 4440, 74.0 s).** "largest gap ÷ image side: 0.088 → 0.0088" sits 30 px under the output frame. I did not recompute its values in this review.

### Review235 Lows

- **L1, cleared.**
  - The "z₀ = 1 + i" ink spans x 143–223 and y 244–260 in every sample from 22.0 s to 31.5 s.
  - The from-above line (x 235–238) first appears at 28.0 s with its top at y 265, and falls to y 294 by 29.5 s.
  - The closest gap is at least 12 px horizontally, or 13 px diagonally, at 540p. Scaled to 4K that is about 50 px, not measured. This is wider than review235's 16–30 px estimate.
- **L2, present (Low).**
  - At 49.0 s the qualifier (y ≈ 172) and "a stretch and a turn" (y ≈ 179) share a row on either side of the axis.
  - From 53.0 s to 75.5 s the rows read "unless the derivative is 0 | the image" (both at y ≈ 171) and "bigger here: stretch 2.83 | exact stretch and turn" (both at y ≈ 192).
  - The grey against green/teal and the Im axis separate them. Even so, the second row can read as one phrase at a glance.
  - A half-row vertical offset between notes and legends would end the pairing.
- **L3, present (Low, wording only).** The "the ratio" inset is up from 22.0 s, while "both pictures drawn to the same scale" stays until 70.5 s.
- **L4.** This is chapter 08, outside this review.

### New items

- **R1, Low (cosmetic).**
  - The "change in the output" ink ends at x 792 and the output frame stroke starts at x 796. That leaves 3 px clear at 540p in frames 600, 900, 1200 and 1290 (9.5–21.6 s).
  - It is not clipped.
  - The output scale is unchanged from final06 (review224: 387 px/unit at 4K, 96.75 px/unit at 540p). The margin was probably already there, and review224 did not flag it.
- **R2, Low (cosmetic).**
  - The "z₀ = 1 + i" ink starts at x 143, and the input Im axis line is at x 139–140. That leaves 2 px clear at 540p in every sample from 22.0 s to 31.5 s.
  - The 238 rescale moved the input plane, so this proximity is new. They do not touch.

### Still present from review224

- **F3 (Low).**
  - Heavy "−" and "×": in frame 7470, "v_x = −u_y", "−2" and "= ×(2+2i)".
  - In frame 8778, "x + iy ↦ x − iy".
  - This was outside the repair's scope.
- **N1.** The red "no complex derivative" stays when the squash title appears at 159.0 s. The X returns at 162.5 s.
- **N2–N5.** At sheet scale they look as they did in final06. I did not re-measure them.

### Other sheet checks

- **06_2.** Every ratio readout agrees with 2 + 2i + h. The settled "→ 2.000 + 2.000 i" is up from 33.5 s.
- **06_5 and 06_6.** These build in voice order. The Cauchy–Riemann banner runs from 108.0 s to 111.0 s.
- **06_8.** The cyan square's arrow is anticlockwise. After the flip, the green square below has a clockwise arrow (frame 8778).
- **Quiz.**
  - The ring is full at 141.05 s and gone at 146.3 s; the box is still up then and no answer shows (frame 8778).
  - The box fades at 146.5 s.
  - "right: 1" is faint at 147.5 s and full at 148.0 s.
  - The answer stays hidden through the think time.
- **End.** The end fade is at 167.0 s. The last picture, frame 10029 at 167.15 s, is blank.

## Not established

- **Motion between samples.** Samples are at most 30 frames apart, so these are unsampled:
  - the scale-note hand-over at 70.5–71.0 s;
  - the fan's stretch and turn between half-second samples;
  - the middle of every fade, the mirror flip and the zoom shrink.
- **4K.** None of these figures is a 4K measurement; the 4K figures are ×4 estimates. This preview is CRF 18 veryfast at 540p, not the final encode.
- **Audio.** No audio heard or checked. I did not check word timing or fine sync, and I did not verify the narration voice (ElevenLabs Frederick Surrey, per the binding).
- **Acceptance.** This is not AV acceptance, publication approval or picture acceptance of a 4K final.
