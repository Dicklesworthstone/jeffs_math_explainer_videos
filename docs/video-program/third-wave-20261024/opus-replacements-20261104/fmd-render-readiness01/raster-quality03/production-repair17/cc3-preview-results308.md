# Preview274 results audit 308 (cc3)

Independent, read-only audit of the finished preview274 job: its receipt, summary, extraction, fits and sample manifests, plus the chapter sources and narration timings they depend on. It answers root's follow-ups on 07, 14 I5, 11 G2/G3, the 11 programmer section, 09/12 coverage and the 09 guide crossing. It ends with the exact list of extra frames and motion intervals.

I did not render, extract, edit or launch anything. I wrote no source, permit, TODO or Bead. CC0's repair310 is theirs; I have not read or changed it. The only picture I viewed is listed in section 9. Nothing here is listening, motion, fine-sync, final-AV or picture acceptance.

## Inputs

| File (Q-relative) | sha256 |
|---|---|
| `cc3-preview-fade274.py` (sampling helper) | `d80cf7b6ffba56968f7c6dfd7a2cca50e65769dce1c97deb5d34683ed3fc9669` |
| `cc3-preview-fade274-notes.md` | `a14dfe9702414e1b3e3945b7b0e1fc0a9b4b8a152b4b33b16eba478afbacd01a` |
| `preview274-review305/summary.json` | `b4e2cb792155655efe4744b8b2a7fe08589fbb92487f793a769309b0547e2d38` |
| `preview274-review305/resource-receipt.json` | `505b12ce94b1396b63969ad14e17767a553f7da89717adde086815ba6756f638` |
| `preview274-review305/05_gallery/sampling.json` | `b8011d9025f5128b3582990cff2cfc59337f3d939ec94c226accd7fccf23c1b9` |
| `preview274-review305/07_one_point/sampling.json` | `efa82174af0134e4a82fbf77a3006927340b4430fd94ea7174b9515c8b77f0c4` |
| `preview274-review305/09_boundary/sampling.json` | `1af25e86abce5f1b8a45253820e1e48053895ab1daf0acf3e83493ee69301da4` |
| `preview274-review305/11_counting/sampling.json` | `eb67bf33a9ad2f7c91c5601026816d7ebd5d1fc3dafb5a68ae3c5004e8307846` |
| `preview274-review305/14_continuation/sampling.json` | `5faa729974a76a216bfcc5d2a36975df67fabd0bd79a07a1912fc784db5feb2b` |
| `preview274-review305/preview/09_boundary.mp4` | `dba964d2b0730b0e327eb32b9b9bb81496ddb4ec3422f043d356a390f1de3fb4` |

The other inputs are listed by their own hashes where they are used:
- Sources: build268's `source-copy.json` and its local copies.
  - boundary.rs `dc80c541…` = `boundary-label-repair52.rs`.
  - counting.rs `ecff48f7…`, one_point.rs `76a8bf1d…`, continuing.rs `9cb4fc0b…` and residues.rs `7d06f69b…` are equal to the files in `production-source-preserved214`.
  - gallery.rs `d57e003a…` = `gallery-review-repair249.rs`.
  - draw.rs `9bb07b3f…` = `cc2-plate-fade-candidate264-draw.rs`.
- Narration timings: rebind269's `report.json` (measured sentence starts) and `phrase-timings.json` (09_1 and 05_9 marks), read read-only on the render host.

## 1. Integrity of the finished job

- **Receipt:** admitted, exit 0, started 2026-11-08T07:10:35Z, finished 10:50:17Z; inputs_sha256 equals final_inputs_sha256; no resource stop. `native.stdout` reports 81,032 frames in 12,927.4 s.
- **Local copy:** all 1,211 job files equal the render host's copies by full sha256.
- **Per chapter:** samples = pictures = summary count, and sheets = ⌈samples/8⌉.

  | Chapter | 01 | 04 | 05 | 07 | 09 | 10 | 11 | 12 | 14 |
  |---|---|---|---|---|---|---|---|---|---|
  | Samples | 74 | 54 | 222 | 59 | 100 | 113 | 193 | 101 | 80 |
  | Sheets | 10 | 7 | 28 | 8 | 13 | 15 | 25 | 13 | 10 |

- **Every chapter:**
  - every schedule row is equal;
  - chroma values are finite and their count equals the frame count;
  - `select.filter` equals the sample list, and picture order equals sample order;
  - every picture and sheet hash matches its record;
  - every sample has a reason.
- **Required frames for placed fades:** 0 missing. The rule is s−1, s, middle, last, first hold and +0.5 s; out-fades add s+n−3…s+n+3, and non-exact plays add ±3 brackets.
- **Gallery249 anchors:** all present except 9909, which lies past 05's last frame (9878).
- **Rigidity247 windows:** all 59 frames present.
- **Lint:**
  - 01 has one finding: "01_1 tight `\frac{1}{1+x^2}` 0.22 from top edge, title-safe 0.25". I did not compare it with an earlier 01 lint.
  - 09 has two findings: 09_3 "behind: unlabeled stroke x 2.80..2.82 under 'opposite samples share a value'" and 09_7 "tight 'x' 0.18 from right edge". Both match lines 3 and 4 of preview21's `09_boundary.layout.tsv` word for word, so they are pre-existing. Preview21's other two 09 findings (crossed ρ, behind p) are gone.
  - The lint cannot see the 09 guide crossing (section 6).

All of these checks passed. They cover only the samples the job chose, not the chapters.

## 2. Fit dispositions

"Authored" is the scene's own schedule:
- the cue's line frame from the timeline,
- plus the `wait_sentence`/`wait_for` offset from the measured sentence start or phrase mark (draw.rs `wait_until` with `frames_covering`),
- plus the frames of the plays before the fade.

Chroma is the per-frame SATAVG from the job's `chroma.txt`.

| Site | Authored source | Fit start | Authored start | Fit − authored | Chroma | Disposition |
|---|---|---|---|---|---|---|
| A1 (01) | 01_6@5063 + sentence 2 (7.24 s) | 5496 | 5498 | −2 | first rise 5499 | right event, small offset |
| B1 (04) | 04_1@36 + sentence 1 (3.34 s) | 240 | 237 | +3 | first rise 239 | right event, small offset |
| C10 (05) | 05_7@6158 + sentence 5 (18.96 s) + 18 | 6875 | 7314 | −439 | strip ramp 7318–7360 | **wrong event** |
| C14 (05) | 05_9@8692 + "in little tiles" (7.66 s) + 18 | 9170 | 9170 | 0 | | right |
| C15 (05, after C14) | | 9188 | 9188 | 0 | | right |
| D1 (07) | 07_3@2102 + sentence 1 (1.925 s) | 2221 | 2218 | +3 | first rise 2223 | right event, small offset |
| D2 (07) | | 4746 | 4746 | 0 | | right |
| G3 (11) | 11_7@7448 + sentence 3 (14.12 s) | refused | 8296 | (best candidate 8296) | fall 8296–8313 | **unplaced**, frames missing |
| H1a (12) | 12_3@2279 + sentence 1 (2.88 s) + 25 | 2474 | 2477 | −3 | first rise 2478 | right event, small offset |
| H1b (12, after H1a) | | 2535 | 2538 | −3 | | follows H1a |
| H1c (12) | | 2767 | 2765 | +2 | rise at 2766, one frame before the fit start | right event; chroma contradicts the fit |

Chroma never contradicts the authored starts. At ±3 frames it cannot rank fit against authored, except in H1c. For the five offset sites, the job sampled the authored start or first hold within its brackets. The authored middle and +0.5 s frames are 2–3 frames from the sampled ones (list A-7 below).

**C10 is a wrong-event fit that passed every acceptance rule.**
- Fit: score 332.4, runner-up 176.1 (0.53 of best), amplitude 4.70.
- The fit sits on the `fade_out_all(&[zcover], 0.8)` at sentence 2 of 05_7 (`gallery-review-repair249.rs:767`; 11.96 s → 6876). Its chroma rise runs 6878–6920 (6.30 → 10.92).
- The authored C10 is `strip.show(stage, 0.8)` at `:807`. It comes after sentence 5 (→ 7296) and the 0.3 s fade-out at `:792` (18 frames), so it starts at **7314** and runs 49 frames to 7362; first hold 7363. Chroma rises 10.96 → 15.50 over 7318–7360.
- Preview274 has **no sample between 7200 and 7500**. The pictures labelled C10 (6872–6954) are real frames of the zcover reveal. The job itself never sampled the strip plate fade. Root has since viewed the authored frames (A-1 in section 8).

**G3 was refused only by the ambiguity rule** (runner-up 218.1 against 221.8), although its best start equals the authored 8296.
- Chroma falls 13.33 → 2.05 over 8296–8313.
- Its search-window samples run every 15 frames from 7491 to 9606. That is why root saw later teaching in the G3-labelled pictures.
- Two of them happen to land in the fade: 8301 (ramp frame 6 of 18) and 8316 (two frames after the first hold). The job missed the exact frames; root has since viewed most of them (A-2 in section 8).

**Lesson for the helper (no change made).** The scene's wait arithmetic is computable from rebind269's report, so a chroma fit was unnecessary. Fitting added one wrong-event acceptance (C10) and one refusal (G3). Every extraction for these sites should use the authored frames given here.

## 3. 07_6: an authored 2.30 s transition, not a 20 s gap

Source: `one_point.rs` (76a8bf1d) lines 669–759. Line 07_6 starts at frame 5210 (86.833 s).

| Frames | Time (s) | What happens |
|---|---|---|
| 5210–5239 | | `fade_out_all(gone, 0.5)`; chroma reaches the 2.00 floor at 5238 |
| 5240–5377 | 87.333–89.617 | Empty screen, 138 frames = 2.30 s. This is the end of sentence 1, "The strong results ahead need more.", which runs 0–2.8 s into the take. |
| 5378–5413 | from 89.633 | `wait_sentence(1)` releases at 5378; blob fill, rim and U `fade_in_all` 0.6 s; chroma first moves at 5380 |
| 5414–5564 | | Centres 0.3 s, disks 0.2 s, grow 1.6 s (`tween_linear`), lone 0.4 s |
| 5900 | 11.5 s into the take | Sentence 3: definition card 0.5 s; chroma rises 5903–5949 |
| 6120 | 15.155 s into the take | Sentence 4: from_here 0.4 s; chroma step at 6124 |
| 6341 | 105.683 s | Breath row in the timeline |
| 6461–6490 | | Closing `fade_out_all(all, 0.5)`; chroma 5.343 → 2.000 at 6490 |

- The take is 18.240 s long, so the voice ends at 105.073 s.
- Blank samples 5270 and 5330 lie inside the authored empty interval. The blank last frame (6490, 108.167 s) is the authored end fade.
- No source repair is indicated by this evidence. Whether 2.30 s of empty screen under that sentence reads as a deliberate beat is an editorial call for motion review (M2).

## 4. 14_8 I5: an authored progressive reveal; the flag is a fit artifact

Source: `continuing.rs` (9cb4fc0b) 14_8, lines 728–800.

| Frames | What happens |
|---|---|
| 10206–10230 | I4 `lac.leave(0.4)` |
| 10231–10237 | `Painter::build(gamma)` comes up blank ("the plane comes up blank and the cover goes on before any colour can show"). I5 is `gamma.show(0.1)`: 7 frames, chroma flat at 2.00. |
| 10238–10279 | Cover added at reveal 0; `gamma.to(Some(Func::Gamma), 0.7)` blends 42 frames; chroma rises from 10247 to 6.31 at 10276–10279 |
| 10280–10297 | `gborder` fade 0.3 s |
| from 10987 | Sentence 3 (13.42 s): five poles (0, −1, −2, −3, −4 by `gallery.gamma.poles`). Each pole gets hop 0.2 s, reveal tween 0.7 s, marks 0.2 s and hop out 0.2 s, 78 frames per pole. |
| 11377–11401 | `pole_says` fades in |

- Frame 10261 is blend frame 24 of 42 (smooth ≈ 0.63). The field shows only on the right because the cover masks the left.
- The reveal tweens end at 11040, 11118, 11196, 11274 and 11352. The full field arrives strip by strip over these.
- The I5 flag comes from the fit window: t at anchor 3.74 is below T_MIN, and the best shift is −4 at t 22.4. That window takes in I4's fade-out tail, so the flag does not mean the timing disagrees with the source.
- `Painter::to` is a blend tween, not one of the 264 plate fades.

## 5. 11_7 G2 and G3

**G2 is an authored seamless swap.** Source: `counting.rs` (ecff48f7) lines 504–519.
- Frames 7448–7465 run `fade_out_all(gone, 0.3)`.
- `Turning::build` then adds the field that will turn, without a fade. The comment at `:512` reads "the painter gives way to a field that can turn, in the same colours, so nothing shows". The loop and root marks are re-added on top.
- `painter.leave(0.4)` (G2) runs 7466–7490.
- The full field in samples 7465–7494 and 7521 is this turning field, as authored.
- G2's cross-check is weak (t 2.08, amplitude 0.027) because nothing visibly changes.
- The stills cannot rule out a one-frame flicker in the unsampled ramp frames 7467–7477 and 7479–7487 (optional frames A-6; motion M4).

**After G2:**
- The field turns at "complex polynomial" and turns back at "which leaves".
- At sentence 3, `fade_out_all(&acted, 0.3)` (G3, `:567`, includes `turning.field`) runs 8296–8313.
- The wide plane fades in over 8314–8343, and the dog walk starts at sentence 4 (18.66 s, near 8568).

## 6. 09_1 guide crossing (root's finding at 69.67 s and 90.28 s)

**Source:** `boundary.rs` dc80c541 (`boundary-label-repair52.rs`).
- `zoom_links` (`:271–279`) draws two lines from the "a little beyond" circle on the picture (polar 0.8 at ±0.7 rad) to the close-up's left corners.
- They are built at `:542–551` as `faded_line(Palette::dim(), 1.5, 0.45)`, faded in at `:552` at "It is undefined at a itself", and removed only in `proof_gone` (`:811–816`).
- The labels in the column between picture and close-up:
  - |g| ≤ M at (1.15, 0.75), 25 pt (`:576–578`);
  - at GAP_LOW −2.0: the cancellation two-liner (19 pt), "both walked / counterclockwise" (18 pt), the wobble two-liner (16 pt) and `EVERY_TEX` (scaled 2.1 wide).

**Geometry** (scene units; text extents estimated from CMR widths, not measured):

| Line | y at x 0.10 / 0.35 / 0.44 / 0.74 / 1.15 / 1.56 | Against |
|---|---|---|
| Upper | 0.741 / 0.775 / 0.787 / 0.827 / 0.882 / 0.937 | label centred at 0.75: crosses its upper left (\|g\|) and grazes its top |
| Lower | −1.953 / −2.062 / −2.101 / −2.231 / −2.408 / −2.586 | second lines centred −2.123 to −2.143: crosses the start of each GAP_LOW second line and the left end of "for every small ρ" |

**Windows** (09_1 line at frame 36; rebind269 phrase marks):

| Interval | Frames | Seconds |
|---|---|---|
| Guides visible | 1703–7996 | 28.38–133.28 |
| Upper crossing (\|g\| ≤ M up) | 2326–7996 | 38.77–133.28 (≈94.5 s) |
| Lower crossing, cancellation | 3742–4606 | 62.37–76.78 (observed by root at 69.67 s) |
| Lower crossing, both walked / counterclockwise | 4643–4956 | 77.38–82.62 (predicted) |
| Lower crossing, wobble | 5698–6523 | 94.97–108.73 (predicted) |
| Lower crossing, for every small ρ | 6886–7065 | 114.77–117.77 (predicted) |

At 90.28 s (frame 5417), no GAP_LOW text is up: "both" left at 82.6 s and the wobble lines arrive at 94.97 s. This matches root seeing only the upper crossing there.

**Why the lint missed it.** `crates/explainer_kit/src/layout.rs` ignores strokes whose contrast is below `FAINT_INK` = 0.35. `Palette::dim` has contrast 0.51 at full alpha; at opacity 0.45 that is ≈ 0.23. So the guides count as background texture. Every earlier 09 lint had the same blind spot, and the lint cannot confirm a repair; pictures must.

**Prior review.** Preview58 rendered the same boundary.rs (`source-comparison158.json` 'new'), and the 09 timeline, cue and cuelog tables are equal after narration-path mapping (`timeline69-comparison.json`; `timeline269-comparison305.json`, old timeline ae7a13f4…). So preview58's 1 s frames had the same crossings at the same times. A text search of `pictures-reviewed146.json` for link, connector, zoom, cross, finding and defect finds nothing. I did not view those sheets. Current pixels govern.

**Correction options** (source reading only; independent of CC0's 310):

- **(a) Smallest.** Retire the guides in the existing `hole_name` fade-out at "But as z approaches a" (`:556`): fade `hole_name` and both links in that one 0.2 s play. Also delete `proof_gone.extend(links);` at `:814`, because fading mobjects that are already off the scene would re-root them for the fade.
  - The play count and run times stay the same, so no frame or cue should move. The qualifying timeline must confirm this.
  - The guides would then show over 1703–2099 (28.38–35.00 s), before any column label exists.
- **(b)** Keep the guides through the z approach: move them into the z fade-out at `:579`, and play that before the dashes/bound fade-in at `:578`. The guides are then gone by about frame 2337, and the dot leaves 0.2 s before the dashes appear.
- **(c)** Re-anchor `zoom_links` at the beyond circle's top and bottom (±π/2) to keep the guides for the whole proof. My estimates give about 0.07 clearance above |g| ≤ M and 0.04 below the GAP_LOW second lines. That is smaller than the error of estimated glyph boxes, so I do not recommend (c) without measured boxes.

**Frames to check on any repaired preview:** 1712, 1730, 2092, 2105, 2343, 3760, 4180, 4660, 5417, 5715, 6903, 7980, 7997.

## 7. Coverage and earlier reviews

Preview274 samples fade windows, cue starts and lint persistence; it was never a chapter review. Every chapter except 01 has unsampled stretches longer than 6 s:

| Chapter | Unsampled stretches > 6 s (s) |
|---|---|
| 04 | 5.70–21.85, 23.85–37.73, 39.73–49.30, 51.30–72.20, 74.20–80.40, 82.40–91.48, 98.73–112.38 |
| 05 | 2.60–17.60, 19.60–32.77, 35.00–47.85, 50.08–64.78, 66.78–79.15, 81.38–102.63, 104.87–114.53, 115.90–126.37, 128.37–144.87 |
| 07 | 2.60–17.93, 19.93–35.03, 51.53–64.38, 66.38–79.05, 80.10–86.83, 88.83–108.17 |
| 09 | 2.60–138.88 (the proof), 140.88–149.97 |
| 10 | 2.60–12.00, 18.73–41.10, 43.10–61.78, 106.05–123.52 |
| 11 | 2.60–19.62, 21.62–42.30, 44.30–71.47, 73.47–84.63, 86.63–94.52, 102.77–124.13, 162.18–198.85 (programmer section 11_8) |
| 12 | 2.60–20.33, 22.33–37.98, 53.02–65.20 |
| 14 | 2.60–20.00, 22.00–42.05, 44.05–83.28, 97.07–127.35, 129.35–147.55, 149.55–169.68, 171.68–195.42 |

01 has 18 shorter gaps, all between 2 and 6 s.

Earlier one-second stills of matching timing exist for every chapter: combined with preview274's samples, each leaves no gap over 1.00 s. In the eight earlier finals, every `line:` sample frame equals preview274's line frame. For 09, the timeline, cue and cuelog tables are equal after narration-path mapping.

| Chapter | Earlier stills (movie) | Scene file in build268 vs production214 | Review record |
|---|---|---|---|
| 01 | final01 pictures142 (0cea4f76), 158 | unchanged | not located by me |
| 04 | final04 pictures198 (6ba1ebd0), 146 | unchanged | `cc2-final04-picture-review207.md` |
| 05 | final05 pictures153 (0c7400e2), 202 | **gallery.rs changed** | `cc0-final05-picture-review240.md` |
| 07 | final07 pictures125 (a42d77f8), 138 | unchanged | `cc3-final07-picture-review229.md` |
| 09 | preview58 (a277716f), 383 | boundary.rs equal to the reviewed candidate68 | `pictures-reviewed146.json` (crossing not recorded) |
| 10 | final10 pictures155 (ce985542), 154 | **rigidity.rs changed** | `cc1-final10-picture-review241.md` |
| 11 | final11 pictures144 (06901e97), 236 | unchanged | root's TODO 2026-11-08T00:53 (all 236 viewed); preview21-11 review72 (programmer panel); root's three current paused frames at 169.76, 184.51 and 194.29 s (`counting-programmer-samples305.json`) |
| 12 | final12 pictures94 (ac19aeee), 82 | unchanged | `cc2-final12-picture-review234.md` |
| 14 | final14 pictures217 (c00d0c83), 230 | unchanged | `cc2-final14-picture-review221.md` |

What differs between those renders and build268, as far as I checked:
- **draw.rs:** the production214/candidate68 → 268 diff only adds lines (358 added, none removed). Outside tests, it adds `image_records` and `plate_fade`, and inside `fade_in_all`/`fade_out_all` it adds a plate fade for families that hold raster images. So pixels can differ only inside those fade windows; holds before and after match.
- **mod.rs:** only the `TEXT` glyph-supplement list changed.
- **Not checked:** I did not bind each earlier final's own source to production214. "Unchanged" means build268 against production214.

**Disposition.** The earlier stills give useful context, but they do not stand in for current frames. 09 shows why: a defect sat in frames of the same source and timing, and the earlier review never recorded it. Covering the stretches above needs current frames (Tier B), and continuous motion needs playback.

## 8. Exact extra frames and motion intervals (existing preview274 MP4s; no render)

### Tier A: targeted frames

Root has viewed 18 authored frames since my first draft of this list (`authored-fade-samples305`, `authored-fade-samples305b`, `authored-fade-review305.json`):
- Both manifests name the current preview274 MP4s: 05 `f5123300…` and 11 `0b6244a0…`.
- The six 305 pictures of each chapter match their recorded hashes. The 305b manifest records the movie hash but no picture hashes.
- I did not view these frames. The status column below quotes root's record.

| Item | Frames | Status |
|---|---|---|
| A-1. 05 C10 `strip.show(0.8)`, gallery249:807 (authored 7314–7362) | Core: 7313, 7314, 7338, 7362, 7363, 7393. Brackets: 7311, 7312, 7315–7317, 7359, 7360, 7364–7366 | **Core done by root**, plus 7337, 7361 and 7392. Root reports the strip absent at the start, partial at 7337/7338 and full at 7362/7363, with no clipping. The 10 bracket frames are optional: I listed them against uncertainty in the start, and the authored start is source arithmetic that chroma confirms. |
| A-2. 11 G3 `fade_out_all(&acted, 0.3)`, counting:567 (authored 8296–8313) | Core: 8295, 8296, 8304, 8313, 8314, 8344. End bracket 8311–8317 | **Core done by root**, plus 8331, 8332 and 8362. Root reports progressive removal, complete by 8313, then the next circle. With preview274's 8316, the out-fade bracket that every other out-fade got still lacks **8311, 8312, 8315 and 8317**. |
| A-3. 09 guide extent, current pixels before repair | 1730, 2343, 3760, 4660, 5715, 6903, 7971, 7997 | full extent of the crossing; root already has 4180 and 5417 |
| A-4. 07_6 interior | 5377, 5378, 5396, 5414, 5566, 5930, 6145, 6340, 6460 | bounds of the empty interval and the later builds |
| A-5. 14_8 reveal | 10279, 10298, 10986, 11041, 11119, 11197, 11275, 11353, 11402 | progressive reveal pole by pole |
| A-6. 11 G2 ramp (optional) | 7472, 7484 | no flicker mid-swap |
| A-7. Fitted-site authored frames (optional; 2–3 frames from sampled ones) | A1 5512, 5558; B1 236, 272, 308, 339; D1 2217, 2247, 2277, 2308; H1a 2477, 2494, 2543; H1b 2538, 2555, 2604; H1c 2782, 2831 | exact authored middles and holds |

Still required: 30 frames (4 in A-2, 8 in A-3, 9 in A-4 and 9 in A-5). Optional: 30 frames (10 bracket frames in A-1, 2 in A-6 and 18 in A-7).

### Tier B: the one-second grid over unsampled stretches

Rule: take every frame number that is a multiple of 60 and lies strictly inside a gap of more than 60 frames between consecutive preview274 samples. Count frame 0 and the last frame as bounds, and skip frames already sampled. These are the same frame numbers the earlier finals' one-second samples used, so each current frame can sit beside its earlier counterpart.

| Chapter | 01 | 04 | 05 | 07 | 09 | 10 | 11 | 12 | 14 | Total |
|---|---|---|---|---|---|---|---|---|---|---|
| Frames | 88 | 94 | 129 | 92 | 248 | 85 | 146 | 53 | 176 | 1,111 (139 sheets of 8) |

If staged, I would start with:
1. 09 (all 248; the proof stretch is frames 180–8280);
2. the 11_8 programmer section, frames 9780–11880 (36 frames);
3. all 53 frames of 12;
4. 14_3, frames 2700–4980 (39 frames).

### Motion intervals

None of these has been watched continuously. Stills cannot give motion acceptance.

- **M1. The 44 plate-fade windows.** 264 changes a ramp over time, so each window needs playback from 0.5 s before its start to 0.5 s after its first hold. Authored starts are used where the fit was off.
  - 01: A1 5498–5528, A2 6275–6311
  - 04: B1 237–309, B2 6744–6774
  - 05: C1 36–96, C2 1991–2021, C3 2021–2070, C4 2896–2926, C5 2926–2975, C6 4774–4804, C7 4804–4853, C8 6183–6213, C9 6213–6262, C10 7314–7363, C11 8717–8742, C12 8742–8767, C13 8767–8857, C14 9170–9188, C15 9188–9218, C16 9819–9849, C17 9849–9879
  - 07: D1 2218–2278, D2 4746–4776
  - 09: E1 36–96, E2 16474–16504
  - 10: F1 3737–3797, F2 6243–6273, F3 6273–6333, F4 7412–7442
  - 11: G1 36–85, G2 7466–7491, G3 8296–8314
  - 12: H1a 2477–2513, H1b 2538–2574, H1c 2765–2801, H2 3027–3102 (three plays), H3 3102–3151, H4 3913–3943
  - 14: I1 36–96, I2 1225–1255, I3 8878–8938, I4 10206–10231, I5 10231–10238, I6 11726–11756
- **M2. 07, frames 5210–5414:** fade-out, the 2.30 s empty screen and the blob arriving.
- **M3. 14, frames 10181–10298 and 10987–11402.**
- **M4. 11, frames 7448–7491 (painter-to-turning swap) and 8296–8344.**
- **M5. 05, frames 6876–6924:** the zcover fade-out that the C10 fit latched onto.

Listening, fine sync and final AV remain open for all nine chapters. Nothing in this audit addresses them.

## 9. What I viewed

I viewed only `preview274-review305/07_one_point/sheet-0007.jpg`: frames 4775–4779, 4806, 5210 and 5240.
- D2's plate leaves cleanly while the vector mosaic fades in, with no reappearance in those tiles.
- 5210 shows the populated mosaic, bound and ratio inset; 5240 is already empty.

I viewed no other sheet, picture or video frame, and I extracted no frames. Every other picture statement here comes from root's records or from source and chroma.

## 10. Open items and limits

- **C10 strip fade:** root has viewed the six core authored frames and three more (A-1). Only stills: the fade has not been watched in motion (M1).
- **G3:** root has viewed the core authored frames. The out-fade bracket still lacks 8311, 8312, 8315 and 8317 (A-2).
- **09 guide crossing:** HOLD stands on root's current pixels. The repair (CC0's 310) needs the frames in section 6 on a new preview. The lint will not catch a relapse.
- **Coverage:** the unsampled stretches above need Tier B or an explicit root decision. No chapter is reviewed in full from preview274's fade samples.
- **Motion, listening, fine sync, 4K finals and final AV:** open.
- **Stray folder:** `Q/.ee/hook-state` exists. It is gitignored by `**/.ee/` and dated 2026-11-08 01:16 local. I did not establish its origin and left it in place.
- **Checks are read-only scripts in my scratch space.**
  - They covered integrity, coverage, fit against authored, the 09 windows and geometry, and grid counts.
  - The 09 glyph extents are estimates.
  - Windows marked "predicted" in section 6 come from source and phrase marks; I did not observe them on frames.
