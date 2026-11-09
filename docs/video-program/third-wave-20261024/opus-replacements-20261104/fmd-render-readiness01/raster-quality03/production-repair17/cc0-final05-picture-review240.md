# complex final05 picture review 240: chapter 05 (gallery), terminal 153

Dispatch-0 (cc0), 2026-11-08T05:00Z. This is an independent review of stills only. I wrote only this file and made no other repository writes. I ran no extraction, render, build or synthesis, and made no edits to source, TODO or Beads.

Narration is ElevenLabs Frederick. No audio was heard or checked. Stills do not establish motion, fine sync, audio acceptance or final AV acceptance.

## Result

I found no mathematical error in what 05 draws, labels or reads out. Every number on screen matches an independent computation, and every zero, pole and turn count sits where the mathematics puts it. There are two findings for root:

- **F1 (Medium, motion polish).** In this render, the fades of the colour pictures (`Painter::show` / `Painter::leave`) never show partial opacity. Each picture change becomes a cut. The chapter's last frame still shows the zoomed tile at full brightness, so the zoom disappears at the 05/06 join instead of fading.
- **F2 (Low to Medium, teaching precision).** 05_9 says the tiles between zeros and poles are "nearly square". The z³ − 1 picture on screen has a critical point at z = 0, where the tile is one large non-square cell. This is the same omission as review224 F2 in 06, which root decided to repair.

## Bound inputs

| Item | sha256 / value |
|---|---|
| `final05-terminal153/05_gallery.mp4` | `0c7400e2703b1688d770dad77af97bebae9c1c897ec7b20177a5cd516e29e67b` (20,519,363 bytes) |
| `render-summary.json` | `561354da…`. 9,879 frames, x264 crf 16; binary `f4836eee…`, permit `1e758a93…` |
| timeline / cues / cuelog / layout | `0537552d…` / `19f9713f…` / `ecae0505…` / `e3b0c442…` (empty layout) |
| `copy-evidence.json` / terminal `resource-receipt.json` | `bebf8374…` / `3334bd66…` |
| `complex-final05-pictures153/sampling.json` | `d69e6b13cbf2da9954f37ae32b032573d3274f62984d7bf9101df6c4d6d07fff`; its `movie_sha256` equals the movie |
| extraction `resource-receipt.json` | `893bc14a…`, 2026-11-08T01:13:10Z to 01:13:51Z, exit 0 |
| pictures and sheets | I re-hashed all 202 pictures and 26 sheets against `sampling.json`: 228 of 228 match. The pictures on disk are exactly the listed set. |
| production19 source (`production-source-preserved214/workspace`) | `gallery.rs` `7d1cf20f…`, `draw.rs` `cd394e5a…`, `model.rs` `118172af…`, `facts.rs` `12f2fef0…`, `script.tsv` `234f7257…`. All equal the entries in `receipt.json`. |
| narration | 05_1 to 05_9 from `delivery101`, starting at frames 36, 1056, 1966, 2871, 3887, 4749, 6158, 7582 and 8692 (cuelog). The music stem is `cec99f95…`. |

## Coverage

- **Sheets.** I viewed all 26 sheets (`sheet-0001.jpg` to `sheet-0026.jpg`), in order, before reading root's record. They hold all 202 selected pictures:
  - 165 one-second samples (frames 0 to 9840);
  - 36 line samples (four per line: the start, then +0.5, +1.0 and +2.0 s);
  - the last frame, 9878.
- **Native details**, cropped at 4K from existing pictures, with crops and measurements kept outside the repository:
  - the traced tile (frames 9420 and 9840);
  - the e^z square (2700);
  - the centre of z³ − 1 (180);
  - the edge marks on the sin panel (9000);
  - the sin(3i) card (3780);
  - the right-edge labels (6720, 7140, 7380, 8160 and 8400);
  - the text and TeX glyphs (frames 120, 1020, 1176, 1860, 3300, 4500, 4980, 6278, 7140, 7380, 8940 and 9420).
- **Pixel measurements.** These are portrait-region means over 27 frames around six picture changes (F1), and edge profiles of the traced tile.
- **Join.** I compared against the first frame of 06 (`complex-final06-pictures169`, frame 0) for the join.

## Findings

### F1 (Medium): the colour pictures' fades render as cuts, and 05 ends on a full-brightness zoom

`Painter::show` fades the picture plate in over `rt`, and `Painter::leave` fades it out (`draw.rs` 1007–1032). In this render, the plate's colour is the same at every sampled point inside a fade as when it is steady. The table below gives the times from the source and cuelog, and the mean RGB over the picture region:

| Change | Frame (s) | Point in the fade | Mean RGB | Steady value |
|---|---|---|---|---|
| z³ − 1 shows over 0–1.0 s | 36 (0.60) | 60% in | 111.2, 141.3, 144.5 | 111.2, 141.3, 144.5 (frame 60) |
| (z−1)/(z+1)² leaves, then e^z shows over 33.667–34.467 | 2026 (33.767) | 12.5% in | 139.6, 115.8, 133.2 | 139.5, 115.8, 133.1 (2100) |
| e^z leaves, then sin shows over 48.75–49.55 | 2931 (48.85) | 12.5% in | 126.0, 127.6, 135.3 | identical (3000) |
| tan leaves over 79.55–80.05 | 4800 (80.00) | 90% out | 143.7, 130.2, 134.0 | identical (4779) |
| Γ shows over 80.05–80.85 | 4809 (80.15) | 12.5% in | 152.1, 113.9, 116.4 | identical (4860) |
| ζ shows over 103.533–104.333 | 6218 (103.633) | 12.5% in | 146.7, 117.9, 120.9 | identical (6300) |
| zoom leaves over 164.15–164.65 | 9878 (164.633, last frame) | 96.7% out | 75.6, 159.4, 165.1 | identical (9840) |

Other polygons do fade. The dark cover over Re z < 0 in 05_6 is part-way out at frame 5460, between the dark 5400 and the clear 5520. So this behaviour is specific to the picture plate. The consequences in this cut:

1. Each change of picture (05_3, 05_4, 05_6, 05_7) is a one-frame swap, not the authored fade.
2. In 05_9, the five panels leaving at frame 9180 are at full colour while their names and marks are already dim.
3. The chapter ends on the zoomed tile at full brightness (frame 9878). Frame 0 of 06 is plain background (mean 12, 17, 20), so the zoom vanishes on the join. The 0.5 s fade the source asks for never happens.

`Painter` is shared through `draw.rs`, so other chapters' picture fades probably behave the same way. I did not verify that here.

**Recommendation (root's call).** First, confirm this in motion at one change and at the 05/06 join. Then either:

- make the plate honour opacity in the kit or engine path (root's call, or an upstream bead with the owner's go-ahead); or
- use a native workaround: inside the existing `leave` / `show` run time, fade a ground-coloured polygon over the plate, as `cover_left` does, and remove or add the plate behind it. This adds no time.

A cheap pure check is to compare a mid-fade frame's mean with the steady frame's, as in the table above.

### F2 (Low to Medium): "little tiles that are nearly square" against the critical point of z³ − 1

05_9 says: "Zeros and poles are isolated points. Between them, the colours flow smoothly in little tiles that are nearly square. That is not a coincidence. It comes from what a complex derivative demands."

- **The counterexample on screen.** For f(z) = z³ − 1, f′(0) = 0 while f(0) = −1. With `BAND_OFFSET` 0.5, the value −1 sits at the centre of a tile. The preimage of that tile near 0 is one large cell with curved sides and twelve corners, because z³ triples angles.
  - **05_1:** the native crop of frame 180 shows the whole inside of the unit circle as one nearly uniform cyan cell.
  - **05_9:** the same picture is on screen in the mosaic (frames 8880–9120) and beside the zoom (9240–9780).

  So at a point that is neither a zero nor a pole, the example picture contradicts "between them … nearly square".
- **Γ as well.** Γ also has a critical point, at x ≈ 1.4616, which lies inside both Γ windows.
- **What is otherwise right.** The claim is true away from points where f′ = 0, and the chapter's chosen tile is honestly picked away from there (`TILE_SEED`). Chapter 07 (07_1, 07_2) treats critical points later.
- **Recommendation.** Keep the treatment consistent with root's decision on 06 F2. Add a native label only, with no audio or timing change. One dim line could join the existing fade of `tile_says` or `next` in 05_9, for example "not where the derivative is 0, as at z = 0 here (chapter 7)". Alternatively, add a small marker at z = 0 on the cubic panel, riding the same fade.

## Notes (no change required)

- **N1.** The heavy math minus, the review224 F3 class, appears here too:
  - "f(z) = z³ − 1";
  - "(z − 1)/(z + 1)²", in the title and the mosaic names;
  - "tan(0.4 − 12i)";
  - "2⁻ˢ" and "3⁻ˢ" in the ζ title;
  - the text label "one tile of z³ − 1".

  The "≤" in "−1 ≤ sin x ≤ 1" comes from a heavier face than the letters. Everything stays legible.
- **N2.** Right-hand labels in 05_7 and 05_8 end 130 px from the right edge (3.4%). The "Riemann hypothesis: open" card ends 86 px (2.2%) from the edge. Both are inside the frame, but tighter than a 5% title-safe margin. Moving the card left (`zcol + 1.2`) is an easy option.
- **N3.** In the mosaic, the sin panel's marks at ±2π overlap the panel border by a few pixels at 4K (frame 9000).
- **N4.** "Re s" is set roman in the text labels but italic in the ζ title.
- **N5.** The traced tile outline sits on the visible tile to within about 10–30 px at 4K. That is about 0.01 to 0.03 of a unit, inside the soft edge of the shading. The e^z square sits exactly on one tile. Neither needs a change.
- **N6.** By the source's arithmetic, 05_9's mosaic completes about 3.3 s after the line starts (fade 0.4, two leaves at 0.4 each, six shows at 0.25 each, frames 0.3, names 0.3). At frame 8812, 2.0 s in, four of six panels are up. Whether "These pictures share a pattern" lands on a complete mosaic is a sync question I cannot settle from stills. In 05_7, the strip is up alone at frame 7320, about 0.4 s before its border, gold band and box arrive.

## Mathematics checked

| Line | On screen | Independent check |
|---|---|---|
| 05_1 | Zeros at 1 and −½ ± (√3/2)i, on the unit circle. The loop has r = 1.8, and the counter reads "turns: 3 = zeros inside". | The winding number of z³ − 1 on \|z\| = 1.8 is 3.000. The partial counts agree: +0.5 when the walker is at about 60°, +1.0 at about 115°. |
| 05_2 | A simple zero at 1. A pole of order 2 at −1, with "turns: −2 (backwards twice)" on a loop of r = 0.3. | The winding number is −2.000. The output arc runs clockwise; the counter reads −0.6 when the walker is at about 108°, matching −2 × 108/360. |
| 05_3 | No zeros or poles. Direction set by height, size by how far right. One exact square. The bracket reads "2π ≈ 6.28". | e^(x+iy) = eˣe^(iy). The bands are a step of 2π/12 apart in both x and y on an isotropic plane, so the tiles are exact squares; the outlined square sits on one tile. The bracket spans y = 0 to 2π, and 2π = 6.2832. |
| 05_4 | Zeros at 0, ±π, ±2π (inside a window to ±6.6). "−1 ≤ sin x ≤ 1". sin(i) ≈ 1.18 i and sin(3i) ≈ 10.0 i. | sin(i) = 1.1752i and sin(3i) = 10.0179i. The voice says "about ten i", which is the same value. |
| 05_5 | Zeros and poles alternate (5 zeros, 4 poles). "tan z → ±i". tan(0.4 ± 12i) ≈ ±1.0000 i. | tan(0.4 ± 12i) = 5.4×10⁻¹¹ ± 0.99999999995i. The flat colours above and below are the colours of ±i. |
| 05_6 | Γ(n+1) = n!, Γ(4) = 3! = 6, Γ(5) = 4! = 24. A cover over Re z < 0. Poles at 0 to −4. "no zeros anywhere". | Γ(4) = 6 and Γ(5) = 24. Euler's integral needs Re z > 0. The poles are at the non-positive integers (−4.5 is the window's edge), and Γ has no zeros. |
| 05_7 | ζ(s) = 1 + 2⁻ˢ + 3⁻ˢ + ⋯ for Re s > 1. A single pole at 1. Zeros at −2, −4, −6. The strip runs to Im s = 32, with zeros at ½ + 14.13i, 21.02i, 25.01i and 30.42i. | The Dirichlet series converges only for Re s > 1. The marks sit at the computed positions in both pictures. The heights match Odlyzko's 14.134725…, 21.022040…, 25.010858… and 30.424876…. The gold box [−1, 2] × [0, 3] equals the gold band at the strip's foot, and 3/32 of the strip's height matches. |
| 05_8 | Rings on the four zeros. "Riemann hypothesis: open". "precisely: this strip shows four zeros, located by Newton's method". | "No zero found so far sits off that line" is true. The precisely line states the strip's scope honestly. |
| 05_9 | Six panels with their zeros and poles. One tile of z³ − 1 traced and zoomed. | The marks in every panel sit at the computed positions (ζ's panel spans only \|Im\| ≤ 3.1, so it shows no nontrivial zeros, correctly). The tile is nearly square, but see F2. |

## Agreement with root's record

I read `final05-terminal153/pictures-reviewed153.json` (`e34fc3a8…`) only after the pass above.

- **Agreement.** I agree with its observations: detailed colour fields, readable Γ and ζ panels and labels, the zero and pole turn counts, the open Riemann hypothesis, the four-zero numerical scope, and the final local zoom. I also agree that the pictures show no persistent overlap or clipping. N2's margins are tight but inside the frame.
- **What the record does not mention.** The fades rendering as cuts, including the full-brightness last frame at the join (F1), and the critical-point counterexample to "nearly square" (F2).
- **Acceptance flags.** Root's flags for audio, fine sync and independent AV acceptance are all false, and this review leaves them so.

## Limits

- **Stills only.** I did not see the motion. F1 rests on pixel means of sampled stills against the source and cuelog timing; the one-frame swaps in motion are inferred. F2's description of the z = 0 tile comes from the 05_1 crop and the same picture as it appears in 05_9.
- **No audio.** I heard and checked no audio, and claim no fine sync, audio acceptance or final AV acceptance. The stated narration is ElevenLabs Frederick, but I did not confirm voice identity by ear.
- **Scope.** All crops and measurements come from existing pictures. I ran no new extraction, render, build or synthesis. Earlier I reviewed 06 (review224) and prepared candidate 231. Of root's 05 notes, I read only `pictures-reviewed153.json`.
