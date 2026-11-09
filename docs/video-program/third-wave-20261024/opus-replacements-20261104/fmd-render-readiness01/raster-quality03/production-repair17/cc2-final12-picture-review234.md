# Final 12 picture review 234

RubyRaven, for root, under `mx-video-program-program-87do.39`. This is an independent review of the selected final 12 (`12_residues`) stills, checked against the preserved production19 source and script.

This review covers stills and source/technical evidence only:
- I viewed every contact sheet as an image.
- I viewed full frames at half native size, and native zooms of every formula, marker and loop.
- I ran read-only Python measurements at full native resolution on geometry, margins, stroke weights and the colour fields.

I did no listening and make no claim about motion between samples, sync, the master join or the final audiovisual result. The narration is ElevenLabs Frederick Surrey according to root and the production19 script header; I did not hear it or check it. The soundtrack, cues and cuelog are not in the local folder, so this review has no audio measurement at all. I ran no job, extraction, render, synthesis or build. I changed no file except this one.

## Job

**Folder:** `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-final12-pictures94`.
- The assignment named `complex-final12-pictures93`. No folder of that name exists. The only final 12 picture job in `five-film-release-20261027` is `complex-final12-pictures94`, and its receipt binds the `final12-terminal93` inputs. I reviewed that job and created no extraction.
- Its `sampling.json` names movie `ac19aeeeb555…`, the sha256 of `final12-terminal93/12_residues.mp4`. Its scope reads: "Completed scored 4K chapter12 only. Contact sheets downsampled; native selected frames retained. No full master, motion, listening or independent AV acceptance."

**Receipt:**
- The command is `python3 -I -B` on `final12-pictures94.py` (repo-relative path). The helper's sha256 is `deffb1078f88f2a17ae7b81c6f49e86c2ccfa93078170dc5de1feab2406ec64d`, which matches the inputs manifest and the file in `production-repair17`.
- The guard admitted the job at 22:30:01 UTC on 2026-11-07, in phase `native` with `preflight_only` false. It finished at 22:30:26 UTC with exit 0. The receipt has no stop entry.
- The inputs hash was `6f92faead3a4b69c55e3ea2ff8a483263c572da4b2d4b9c40fd606c0ea3850ef` at both the start and the end. This equals the sha256 of `final12-pictures94-inputs.json`.
- That manifest lists six files: the helper, plus the timeline, mp4, render receipt, render summary and layout from `final12-terminal93`. Each one matches its file.

| Job file | sha256 |
|---|---|
| `resource-receipt.json` | `e8e4e94b0922d21408c14bd32f7734bc0da6ed2fd48bd7377b17c4a0564baabd` |
| `sampling.json` | `8199cc31470ccba9a764173348614d5aa8a7d67cdaccee9ff8a046cc44d74572` |
| `select.filter` (1,045 bytes) | `7087dcce5cf3b763189493f3cb33ca8dcad1fb9f8838cff21eba5a1ade351310` |
| `command.log` (0 bytes) | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

## Inputs (`final12-terminal93`)

| File | sha256 |
|---|---|
| `12_residues.mp4` (3,943 frames, 65.717 s at 60 fps) | `ac19aeeeb555bb7700d1ff366c5d07effbd44a8037f25bb67239c46e603ead28` |
| `12_residues.timeline.tsv` | `4be8f9372fdb4c636416ac2052081025b387a3c4117fcd53c9b366908b9715a0` |
| `12_residues.layout.tsv` (empty: clean lint) | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `render-summary.json` | `33a30608e95e9fd24aefd73e859c94c90fe7dc9f892891c63c3d9b3c7eef65e5` |
| `resource-receipt.json` (render) | `22e85a0c9a29bedffaf8cbddc9dab596c07cefd4368c494bc08e9ecceaa87126` |

- `render-summary.json` records chapter `12_residues`, 3,943 frames, binary `f4836eeeb4f2…`, permit `94727519fd01…` and x264 CRF 16.
- The summary also names the render host's cuelog (`77aa8b22…`), cues (`be53a37b…`) and soundtrack (`82d9e227…`). None of the three is in the local folder, so I could not check them.
- The folder also holds root's `pictures-review94.json` (`4af26604b86e…`), `bead94.stdout` and `bead94.stderr`.
  - I read `pictures-review94.json` only after my own pass. Nothing below depends on it.
  - Root viewed all 11 sheets and the 82 pictures, plus full source frames 2100 and 3600. It reports no persistent label overlap or clipping, which agrees with my measurements.
  - It does not mention L1 or L2 below.
  - I did not read `bead94.*`.

**Timeline:**

| Row | Frame | Time |
|---|---|---|
| line 12_1 | 36 | 0.600 s |
| line 12_2 | 1220 | 20.333 s |
| line 12_3 | 2279 | 37.983 s |
| line 12_4 | 3002 | 50.033 s |
| breath (2 s) | 3763 | 62.717 s |

The timeline has no `mark` or `listen` rows, so this chapter has no quiz or think window.

## Source and script (preserved production19)

- **Folder:** `production-source-preserved214/`. Its receipt is dated 2026-11-08T03:48:11Z, from the production-repair19 workspace. All 65 `complex_analysis_visual` files in it match `trj-production19-transfer-manifest01.json`.
- **Files read**, all under `workspace/explainer_corpus/complex_analysis_visual/`:

  | File | sha256 |
  |---|---|
  | `src/chapters/residues.rs` (18,370 bytes) | `7d06f69b2aff86ac1bee103c7aaefc58891d0b335312e98df3610769f45af7c1` |
  | `src/facts.rs` | `12f2fef0d6807540d3642972a1125c89e683d5f9615689f4100e6aea522530bc` |
  | `src/chapters/draw.rs` | `cd394e5a451bf1114bead7a086660d8f22cc282e8bb66de4b17b833a98bd9af1` |
  | `script.tsv` | `234f725781bde0bdf35afbc21fd2d6ec0f73f99a95723294259cc1b75a813d3c` |
  | `docs/storyboard.md` | `e45d2bb52e74557aa95d40c22dded1eebc6211812b87acd0d7a870538768b153` |
  | `narration_units.json` | `995e25c7788943deff65c4d84050d8a17edc3c2c3498be0d665d7ac5b661357a` |

- **Script header:** it names ElevenLabs Frederick Surrey.
- **Lines 12_1 to 12_4** are identical in `script.tsv`, `narration_units.json` and the storyboard.
- **Facts:**
  - `argument.local` = "1,2,-1": the residues of f′/f for z, z² and 1/z.
  - `residue.bump.at_i` = "0.0,-0.5": 1/(2i) as (re, im).
  - Source tests pin both values and the 24-step loop sums.

## Coverage

- **Samples:** 82 native JPEGs at 3840×2160, frames 0 to 3900, all unique, with no gap longer than 1.0 s:
  - 66 one-second samples (frames 0–3900);
  - 16 cue samples (4 lines × 4, at +0, +30, +60 and +120 frames).
  - The selection does not include the chapter's last frame (3942). The last sample is frame 3900 (65.000 s), 42 frames before the end.
- **Hashes:** I re-hashed all 82 pictures and all 11 sheets against `sampling.json`. There were 0 mismatches.
- **Sheets:** I viewed all 11 sheets as images: 10 sheets of 8 tiles and 2 tiles on the last sheet, which is all 82 samples.
- **Full frames at half native size (1920×1080), viewed as images:** frames 1140, 2040, 2940 and 3780.
- **Native zooms at 1:1, viewed as images:**
  - frame 1140: the Laurent formula and the 1/(z − p) arrow column; "residue = a₋₁" and the names panel (at half size);
  - frame 2040: the residue theorem, the near-pole and simple-pole formulas, the 12_2 plane with all four loops, and the big loop's arrow;
  - frame 1440: the big loop's arrow before it dims;
  - frame 1500: "each small loop: 2πi × its residue";
  - frame 2940: both f′/f formulas, the "turns" text, and the f′/f lines and residue labels under the plates;
  - frame 3780: the pole at i with its loop, the pole at −i with its label, and the three 12_4 formulas.
- **Measurements at full native resolution, on 18 frames:**
  - frames 0, 1140, 1440, 1500, 2040, 2940 and 3780;
  - names-panel presence on frames 1020, 1080, 1140, 1200, 1220, 1250, 1260, 1500, 1560, 1620, 2279, 2280 and 2309 (1140 and 1500 are in both lists).
  - The methods were:
    - plane-border scans;
    - hue against the argument of the drawn function, with control functions;
    - marker centres from ring centroids, dark centres and white cores;
    - circle fits to loop strokes;
    - ink extents for margins;
    - stroke thickness of bars.

| Sheet | Tiles | Frames | Seconds | sha256 |
|---|---|---|---|---|
| sheet-0001.jpg | 8 | 0–180 | 0.000–3.000 | `a734ff1cace7c2540dad3a9134f46e402b96f8aa14ec7ad4f13fa72f745e4596` |
| sheet-0002.jpg | 8 | 240–660 | 4.000–11.000 | `4ca5ed59b5993520be309b6bef00d0e506e7e30d8c56162f8792008c14c47dd2` |
| sheet-0003.jpg | 8 | 720–1140 | 12.000–19.000 | `97b0f62829e0a1f96a782edf5fd48c2824ade2915739d10db3599485efe65f4d` |
| sheet-0004.jpg | 8 | 1200–1380 | 20.000–23.000 | `1bb8a9ffef61950dbe81782ac0766b6f2a99a2f0b7d40d567976c2dbd21b0204` |
| sheet-0005.jpg | 8 | 1440–1860 | 24.000–31.000 | `b5cf3be11aa01395ed68b50a091e2abe818f9d2d617bc015d61c5ed8603dc5b6` |
| sheet-0006.jpg | 8 | 1920–2280 | 32.000–38.000 | `4067e4ce07bf41043e46302023d90b7cd6a02ddd6b5b176eea0642028800c4fd` |
| sheet-0007.jpg | 8 | 2309–2580 | 38.483–43.000 | `bf4bc829fb72eb530d587bc20c8ad967bf6c752b4e5e647d90928051e68eb2fb` |
| sheet-0008.jpg | 8 | 2640–3002 | 44.000–50.033 | `9c64058592e48a2319c80ae226a4385afccc9f9db26ade65121e640343fd51db` |
| sheet-0009.jpg | 8 | 3032–3300 | 50.533–55.000 | `3fb9fbfcb3bf17b20ee63789e60a38c7567980f5578fa500af627e66dad2c846` |
| sheet-0010.jpg | 8 | 3360–3780 | 56.000–63.000 | `8fbdc84b94cbf5a6d840b06903700aa0578fd6749c891a200900a06da43b8923` |
| sheet-0011.jpg | 2 | 3840–3900 | 64.000–65.000 | `0e0f0d90b117d7aba742479d4ceae7a3aac4f060fe79e1ea74afbbb130ac7e0c` |

## Verdict

**Blockers: none.** I found two Low cosmetic findings, one of them the carried type-style class. Nothing is High or Medium. Neither finding changes what the chapter teaches, and neither justifies a re-render on its own; that call is root's.

The residue mathematics holds on every sampled frame and agrees with the script, the storyboard and `facts.rs`:
- **Laurent terms (12_1):** each term's tip-to-tail chain does what the line says. The 1/(z − p)² and holomorphic chains close; the 1/(z − p) chain is a straight column labelled 2πi; and the residue is named a₋₁.
- **Residue theorem (12_2):** the theorem, the deformation picture and the simple-pole rule Res g/h = g(p)/h′(p) are correct. The loops sit at the source radii and centres within about 1 px, and all four direction marks run counterclockwise.
- **Turns as residues (12_3):** f′/f ≈ m/(z − z₀) at a zero and −n/(z − p) at a pole. The three plates show residues 1, 2 and −1, which equals `argument.local`. Each colour field matches arg f, against controls.
- **The bump (12_4):** Res at i of 1/(1 + z²) = 1/(2i) = −0.5 i, which equals `residue.bump.at_i` and line 12_4. The field matches arg 1/(1 + z²), and the loop encloses i only.

## Findings (both Low, cosmetic)

**L1. The "at i" dot gives the pole marker at i a white centre, the look of a zero mark.** Seen from 56.000 s to 65.000 s, in samples 3360–3900 (sheets 10–11).
- **Source:** `residues.rs` line 340 puts `dot(stage, bp.p(I), 0.05, Palette::ink())` on top of the bump's pole mark at i. There it is drawn in white, 13.3 px in radius (the source gives 13.5 px), centred at (1001.5, 823.0). The pole sits at (1002.0, 823.5).
- **The other pole:** the pole at −i keeps its dark centre, measured at (1001.9, 1498.0).
- **Why it matters:** in 12_3, from 38 s to 50 s, the chapter shows zeros as white dots (radius 18.8 px on a dark backing) and poles as pink rings with dark centres. So for about the last 9 s, the two simple poles of the same function are drawn differently, and the one at i carries the zero colour.
- **Why it is Low:** the pink ring is still there. The gold loop circles it, the "i" label sits beside it, and the line says "At i, that is two i". A viewer can still tell what is meant.
- **Possible fix (root's call):** a non-white dot, or no dot, would keep the two pole marks alike.

**L2. Type-style differences.** This is the same class as final13 L4, final04 L4 and final14 L3, which root accepted in disposition 206.
- **Minus sign:** inside typeset formulas it is short and heavy.
  - In "(z − p)" at frame 1140, the minus bar is 7 px thick. The plus bar is 3 px, each equals bar is 3 px, and the fraction rule is 4 px. The minus is 44 px wide; the plus is 62 px.
  - The same minus appears in every "z − p" and "z − z₀", in "−n", "−1/z" and "−0.5 i", and in the text "zeros − poles" and "residue −1".
- **Other heavy glyphs:**
  - By eye at 1:1, `≈` and `≠` are heavier than `=` (frames 2040 and 2940).
  - By eye at 1:1, the `×` in "each small loop: 2πi × its residue" (frame 1500) looks like a heavy sans glyph in a serif line.
- **Upright against italic:**
  - "2πi" under the column and in the "each small loop" caption has an upright i; the theorem's 2πi is italic.
  - The pole labels "i" and "−i" are upright, while "h′(i) = 2i" is italic.
  - h, p, m and n are upright in "h holomorphic near p", "near a zero of order m" and "near a pole of order n".
- **Prime:**
  - "the sum of the residues of f'/f" uses a straight apostrophe, while the formulas use f′.
  - There is a visible gap between the prime and the parenthesis in "h′ (p)", "h′ (z)" and "h′ (i)" (frames 2040 and 3780).
- All are readable, and none changes meaning.

## Carried open item (not checked here)

The same open item as final13 L3 and review221 applies. Several elements sit in the lowest 15% of the frame, where uploaded captions and player controls appear:
- **The "names to search" panel:** rows 1926–2107, 106 px from each side and 52 px from the bottom.
  - "Laurent series · residue" shows from 18.0 s (fading in) until line 12_2 starts, and is gone by 20.833 s.
  - "residue theorem" shows from 26.0 s to 38.0 s, and is gone by 38.483 s.
- **The 12_3 lines under the plates:**
  - "f′/f = 1/z", "2/z" and "−1/z" sit in rows 1794–1958.
  - "residue 1", "residue 2" and "residue −1" end at row 2086, 73 px from the bottom.
  - They show from 42 s to about 50.4 s.
- **"residue = a₋₁"** sits in rows 1788–1856. It shows from 18 s, is still there at 20.333 s, and is gone by 20.833 s.

Whether captions or controls cover these on the platform cannot be seen in source stills. It stays with root's open platform caption-overlay check.

## Near misses and observations

- **Opening:** frames 0 and 36 are blank background. The 12_1 plane fades in from 0.6 s, the first cue frame. This was not seen as a defect.
- **Line starts:** the samples at the starts of 12_2 (1220), 12_3 (2279 and 2280) and 12_4 (3002) still show the previous scene. Each clears within 0.5 s (by 1250, 2309 and 3032). This is the same hold-to-cue pattern as final04 and final14.
- **12_4 build-up:**
  - After the 12_3 labels clear (3032), the three bare plates leave one at a time: the z plate is gone at 3060, and two plates remain at 3062.
  - The bump field shows at 52.0 s (3120), with its marks and ±i labels at 53.0 s (3180).
  - "g = 1, h = 1 + z², h′(z) = 2z" arrives between the 53 s and 54 s samples. From the source fades, it starts at about 53.03 s and is complete at 53.43 s. That is 3.0–3.4 s after line 12_4 starts, and the line's first sentence names "one plus z squared" and "two z".
  - Whether the formula lands before, with or after those words is not established. There is no soundtrack here and I did no listening.
- **Big loop's arrow (12_2):**
  - The direction mark is a straight chord from angle 1.75 to 2.1 rad laid on the circle.
  - At full strength (frame 1440) it merges with the loop, except for a small step where the shaft starts.
  - After the loop dims at 25 s, the overlap of chord and arc shows brighter than the rest of the dimmed loop at 1:1 (frame 2040). This is invisible at sheet scale.
- **Bottom small loop (12_2):** its corridor runs along the negative imaginary axis, so the axis line runs between the two corridor lines. That loop's direction mark sits on the axis. It is readable.
- **Right margin (12_3):** "the sum of the residues of f'/f" ends at column 3679, 160 px (4.2%) from the right edge. It is clear of the edge, and it is the tightest side margin in the chapter.
- **The 2πi column (12_1):** all 24 arrowheads are visible at 1:1. The column is the 24-step sum, 6.2653i, against the label's 2πi (6.2832i); the source test pins it within 0.02. The 0.3% shortfall is invisible.
- **End of chapter:** the last sample (3900, 65.000 s) is mid-fade. The formulas, loop and markers are dimming, and the bump field is still full. By the source, `fade_out_all` (0.5 s) and then `bump.leave` (0.5 s) end on the last frame, 3942. That frame was not sampled, and the 12→13 join was not checked.

## What holds

- **Geometry:** 270 px per scene unit, with the origin at (1920, 1080). The plane borders' ink edges fall within 3 px of the source boxes, which is the stroke width:
  - 12_1/12_2 plane: x 324–1571, y 618–1865, against 327–1569 and 621–1863;
  - 12_3 z plate: x 216–977, against 219–975;
  - 12_3 1/z plate: x 1944–2705, against 1947–2703;
  - bump plate: x 324–1679, y 483–1838, against 327–1677 and 486–1836.
- **12_1, Laurent terms.**
  - The formula "f(z) = a₋₂/(z − p)² + a₋₁/(z − p) + h(z)" has a₋₁ in gold, and "h holomorphic near p" sits under it. Its first ink is at row 174.
  - The p marker's centre is (947.7, 1241.5), against (948.0, 1242.0).
  - The small loop fits to centre (947.6, 1241.6) and radius 349.3 px, which is the source's 0.9 units (349.3 px). Its arrow, at about 45°, runs counterclockwise.
  - **The 1/(z − p)² chain** is a closed ring that turns clockwise, as the products (i/r)e^{−iθ}dθ do.
  - **The holomorphic chain** h = 1 + (z − p)/2 is a closed ring that turns counterclockwise. Its long arrows are at θ = 0 and its short ones at θ = π, as |1 + z/2| runs from 1.45 to 0.55.
  - **The 1/(z − p) chain** is a straight upward column.
  - "closes: 0", "2πi" and "closes: 0" follow in that order, then "residue = a₋₁" (18 s) with the names panel. This is the order of line 12_1, and the source test holds the other two totals under 1e-9.
- **12_2, the residue theorem.**
  - **Pole markers:** (675.9, 1047.4), (1180.6, 1008.6) and (947.7, 1513.2), against (676.3, 1047.9), (1180.9, 1009.1) and (948.0, 1513.7).
  - **Big loop:** it fits to centre (948.0, 1241.8) and radius 523.8 px, against 1.35 units (524.0 px).
  - **Small loops:** their radii are 97.1–97.2 px, against 0.25 units (97.0 px), and their centres are within 1.0 px.
  - **Direction marks:** all four run counterclockwise:
    - the big loop's, at top left, points left;
    - each small loop's sits on the side away from its corridor: the upper-left loop's at its lower right points up-right, the upper-right loop's at its lower left points down-right, and the bottom loop's at its top points left.
  - **Deformation picture:** each corridor is a pair of lines running radially out to the big loop. The small loops, marks and corridors are laid down one at a time (22–24 s), and the big loop dims from 25 s, as the storyboard says.
  - **Statements:** these are all correct:
    - "∮ f(z) dz = 2πi Σ_k Res_{p_k} f";
    - "sum over the poles inside the loop";
    - "the big loop = the small loops (Cauchy's theorem between them)";
    - "each small loop: 2πi × its residue".
  - **Simple-pole formulas:** "g/h ≈ g(p)/(h′(p)(z − p))" shows from 28 s, and "h(p) = 0, h′(p) ≠ 0: Res_p g/h = g(p)/h′(p)" from 33 s. The source cues the second on "the residue is". Both are correct for g holomorphic at p, and the second also holds when g(p) = 0 (residue 0). The names panel "residue theorem" shows from 26 s.
- **12_3, turns as residues.**
  - **Order:** "the turns we counted: / zeros − poles = / the sum of the residues of f'/f" comes first (38.5 s). Then the zero formula (40–41 s), the z plate (42 s), the z² plate (43 s), and the pole formula with the 1/z plate (46–47 s). This is the order of line 12_3.
  - **Formulas:** "f′/f ≈ m/(z − z₀)" ("near a zero of order m") and "f′/f ≈ −n/(z − p)" ("near a pole of order n") are the correct leading terms. The plates' "f′/f = 1/z, 2/z, −1/z" and "residue 1, 2, −1" are correct and equal `argument.local`.
  - **Hue** (frame 2940, 508 grid points per plate, |z| ≥ 0.25):

    | Plate | Median deviation | 90th percentile | Offset | Sign | Other sign (median) | Control (median) |
    |---|---|---|---|---|---|---|
    | z | 7.5° | 13.9° | 357.9° | +1 | 81.8° | 82.7° (z²) |
    | z² | 7.0° | 14.5° | 357.1° | +1 | 79.8° | 89.4° (z) |
    | 1/z | 7.2° | 13.9° | 357.9° | +1 | 81.7° | 82.4° (z²) |

    On the 1/z plate, the opposite-sign score is the score that arg z would get. So that plate shows 1/z, not z.
  - **Markers:** the zero dots are at (596.5, 1349.5) and (1460.5, 1349.5), with a radius of 18.8 px (the source gives 18.9). The 1/z pole's dark centre is at (2324.9, 1349.7).
- **12_4, the bump.**
  - **Hue** (frame 3780, 1,389 points away from the poles, loop and labels): median deviation 6.6°, 90th percentile 13.9°, offset 355.5°, sign +1. The opposite sign gives 77.4° and the control 1/(1 − z²) gives 40.3°.
  - **Spot pixels:**

    | Point | Hue | arg f |
    |---|---|---|
    | right of i | 279.6° | 277.1° |
    | above i | 182.8° | 180° |
    | below i | 351.1° | 0° |
    | left of −i | 279.6° | 277.1° |
    | below −i | 182.9° | 180° |
    | real axis at ±1.5 | 350–351° | 0° |

  - **Poles:** i's white core is at (1001.5, 823.0) and −i's dark centre at (1001.9, 1498.0), against (1002.0, 823.5) and (1002.0, 1498.5).
  - **Loop about i:** it fits to centre (1001.8, 823.1) and radius 118.4 px, against 0.35 units (118.1 px). It encloses i only; −i is 2 units away. The "i" and "−i" labels sit 0.62 units right of the poles, clear of the loop.
  - **Steps:** "g = 1, h = 1 + z², h′(z) = 2z", then "h′(i) = 2i" (from 56 s), then "Res_{z=i} 1/(1 + z²) = 1/(2i) = −0.5 i" (from 59 s). These are correct, in the order of line 12_4, and "−0.5" is the `residue.bump.at_i` imaginary part.
- **Layout:**
  - The layout log is empty.
  - The top ink is at row 174 for the Laurent formula, 156 for the theorem and 195 for the 12_3 formulas. The 12_4 formulas occupy rows 412–1171 and end 377 px from the right edge.
  - No sample shows clipping or text touching a frame edge, which agrees with root's observation.

## Not done

- No listening to the ElevenLabs Frederick Surrey narration or the soundtrack mix. The soundtrack is not local, and I made no audio measurement.
- No check of cues or cuelog; they are on the render host only.
- No claim about motion between samples or fine sync, including the 12_4 formula onset above.
- No check of the last frame (3942), the 12→13 join, the master, captions or the platform overlay.
- No independent final audiovisual acceptance and no publication acceptance.
- No job, extraction, render, synthesis or build.
- No edit to source, helpers, shared files, TODO, memory or Beads. No commit.
