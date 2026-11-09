# cc3 final07 picture review 229

Dispatch-3 (cc3), 2026-11-08 04:40 UTC. This is an independent review of the selected 4K pictures of chapter 07 (`07_one_point`), bound to `final07-terminal125`. It is the only file I wrote in the repository. I rendered, built and synthesized nothing, and I made no source, TODO, Beads, memory or commit changes. Scratch crops and measurements stayed in my session scratchpad, outside the repository.

I finished my own pass before opening root's record (`final07-terminal125/pictures-reviewed125.json`). The comparison is in the last section, added after the rest of the report was written.

Paths below are relative to this folder (`production-repair17/`), except the picture set, which is `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-final07-pictures125/` (called `P/`).

## Verdict

- The maths on screen is correct, including the one-point counterexample `|z|²` and its distinction from "holomorphic on an open region".
- The pictures match the bound script (`script.tsv`, `234f7257…`) line by line.
- The only native suspect from the sheets, green disks possibly touching the edge of U, is cleared at 4K: every disk sits inside U with visible margin.
- I found no High or Medium picture defect. There are five Low findings (L1–L5). L1 (empty frames in two cue openings) is the one root may want to weigh, against the narration onset that I did not measure.
- This is not acceptance of the master, and I did no listening, no fine sync and no final audiovisual review.

## Binding

| File | sha256 | Bytes | Checked |
|---|---|---|---|
| `final07-terminal125/07_one_point.mp4` | `a42d77f8dcd38bcba009a1f29e4ee482a7db610376675ba9d783f6a0b1f36767` | 10,116,476 | Equals `movie_sha256` in `P/sampling.json`. Decoded as x264 3840×2160, 60 fps, 6491 frames, 108.183 s, yuv420p bt709, with AAC audio. |
| `final07-terminal125/07_one_point.cues.tsv` | `ea5cf40807968d38f9e6d0faa8395afaa72aba292ae1589dbc6379ddff54a68a` | 112 | Cue starts: 07_1 0.600 s, 07_2 17.933, 07_3 35.033, 07_4 49.533, 07_5 64.383, 07_6 86.833. The predict mark is at frame 2657 (44.283 s) with a 5 s listen. The breath is at frame 6341. |
| `final07-terminal125/07_one_point.timeline.tsv` | `9b1ab5b31f03a640c2ab1b95912376d746ad35d95dd28d26e867b2879d1fb7b1` | 822 | Equals `timeline21-terminal01/07_one_point.timeline.tsv` (`c2e3c183…`) once narration paths are mapped. |
| `final07-terminal125/07_one_point.cuelog.tsv` | `05d1635ba7d1a946345327d5e31c1f69bd4cb6b8bb307cb4195a8944bfdede4d` | 1,856 | Compared with timeline21's cuelog, it adds only the music stem row (`d2692c6c…`, 48 kHz stereo). The narration WAV hashes are identical. |
| `final07-terminal125/render-summary.json` | `841346b01915e63c42a314c823bb005f1256bf6d9348f4927e158a12548774d7` | 2,458 | Binary `f4836eee…` (production19), x264 crf 16. |
| `script.tsv` | `234f725781bde0bdf35afbc21fd2d6ec0f73f99a95723294259cc1b75a813d3c` | 29,103 | The six 07_x lines were read in full. |
| `hz2-review07-84/final-render-permit84.json` | `4efe6e87c015ad3ed63cf01dfcc4dd50ad3a46d16875fb3951ac43002047ccbc` | 3,750 | Binds binary `f4836eee`, timeline `c2e3c183` and script `234f7257`. I read only its hash fields. |
| `P/sampling.json` | `daa03221e6f84f3a96ceb287f2e5e03a24a56f19ae69b0b20f903e9850748f88` | 58,553 | Its scope line says it is "not actual review, full master, listening, fine-sync or independentAV acceptance". |
| `P/select.filter` | `3c29345d3189e1fabbbc7afad34c5576e8d62a8fae66c352e067ad2836ae9aa6` | 1,773 | |
| `P/resource-receipt.json` | `952bd118b366c3a62931c450ded4c0ce5ee6a3833f91f68d8030d6053577d31d` | 3,106 | Exit 0; ran `final-chapter-pictures115.py`; inputs `929d6ae3…`. |

## Sampling count

The exact count is **138 frames on 18 sheets**.

- `select.filter` names 138 distinct frames, 0 to 6490. They equal the sorted frame list in `sampling.json`.
- Each sample has one reason:
  - 109 one-second samples;
  - 4 each for lines 07_1 to 07_6 (24 in all);
  - 4 for `mark:predict`;
  - 1 last frame.
- `P/pictures/` holds 138 JPEGs at 3840×2160.
- There are 18 sheets of 1920×2260: 17 with 8 tiles and the last with 2 (17 × 8 + 2 = 138).
- All 156 hashes in `sampling.json` (138 pictures and 18 sheets) match the files.

## What I inspected

1. I read all 18 sheets as images.
2. I took native 4K crops from 21 of the 138 pictures: frames 1080, 1380, 1440, 1620, 2132, 2400, 2460, 3780, 4260, 4800, 4920, 4980, 5210, 5220, 5240, 5520, 5940, 6000, 6300, 6360 and 6420. Small glyphs were also upscaled 3×.
3. I ran pixel measurements on the existing 4K pictures:
   - disk-to-edge gaps at frames 5460–6420;
   - red-pixel scans on all 19 samples from 5400 on, plus full-resolution scans of 5 of them.
4. I decoded all 6491 frames of the terminal mp4 to 64×36 gray, only to find empty frames. It ran once, before the account swap, and wrote nothing to the repository. Later steps reused it without decoding again.

## Maths

| Line | Claim on screen and in the script | Check | Verdict |
|---|---|---|---|
| 07_1 | `f′(z) = 2z, f′(0) = 0`. Under `z ↦ z²` the real line goes to `[0, ∞)` and the imaginary line to `(−∞, 0]`. The 90° arc becomes a 180° semicircle. | `x² ≥ 0` and `(iy)² = −y²`. The arc `e^{iθ}`, θ from 0 to π/2, maps to `e^{2iθ}`, θ from 0 to π. In the image panel both half-axes have the same extent (463 and 464 px at 4K), as `x²` and `y²` require when the domain extents are equal (422 and 421 px). | Correct |
| 07_2 | "derivative ≠ 0: angles kept (conformal)". A cross off the origin maps to two curves meeting at 90°. "precisely: if the first m − 1 derivatives vanish at a point and the m-th does not, angles there are multiplied by m". | The domain cross is in the second quadrant. Its image lies just right of the negative imaginary axis (angle ≈ 279°, so the preimage angle is ≈ 140°), with the curves crossing at about a right angle. Near such a point, `f(z) − f(z₀) ≈ c(z − z₀)^m` with `c ≠ 0`, so angles are multiplied by m. | Correct |
| 07_3–07_4 | `f(z) = |z|² = x² + y²`; the ratio is `|h|²/h = h̄`, its size `|h| → 0`, so `f′(0) = 0`. The ratio panel traces `h̄` and reads "ratio → 0.0000". | `|h|²/h = h h̄/h = h̄`. In the panel: real `h` traces the positive real axis, `h` going up traces downward, and the spiral comes out conjugated. All three are what `h ↦ h̄` does. | Correct |
| 07_5 | At 1: "right: → 2", "up: → 0", red ✗, "a complex derivative at 0 only". | `(|1+h|² − 1)/h = (2 Re h + |h|²)/h`. For `h = t`: `2 + t → 2`. For `h = it`: `t²/(it) = −it → 0`. At any `z₀ ≠ 0` the ratio tends to `z̄₀ + z₀·h̄/h`, which depends on direction, so the only point with a derivative is 0. | Correct |
| 07_5 | `|g(h)||h|²/|h| ≤ M|h| → 0`; caption "\|z\|² × jagged g, \|g\| ≤ 1 = M: derivative 0 at 0". The ratio scatter stays inside a circle labelled "inside radius 0.54" that shrinks to 0.06. | The product is 0 at 0, and `|(|h|² g(h))/h| = |g(h)||h| ≤ M|h|`. This needs only boundedness, so it holds for "smooth or jagged" g, discontinuous included. With M = 1 the ratios lie within radius `|h|`, which matches the circle. | Correct |
| 07_6 | "holomorphic on U: a complex derivative at every point of the open region U"; disks around points inside U; "\|z\|²: one point, no disk"; "from here on: holomorphic on an open region". | This is the standard definition. U is drawn with a dashed edge, showing the boundary is excluded. `{0}` contains no disk, so `|z|²` is holomorphic on no open set, which is the distinction the line draws. | Correct |

**Answer timing.** The "derivative at 0?" box appears at 41 s. The countdown ring runs 44.28–49.3 s. The answer (the `h̄` formula) first shows at 51.0 s, and the ratio panel at 51.5 s, both after 07_4 starts at 49.533 s. On the sampled frames (including the 4 `mark:predict` samples), the answer stays hidden until the think time ends.

## Pictures by section

- **0–18 s (07_1, sheets 1–3).**
  - Frame 0 is empty.
  - The two planes with `z ↦ z²` build up, and the green heading `f′(z) = 2z, f′(0) = 0` appears from about 5 s.
  - The real line is cyan and the imaginary line green. Their images land on the positive and negative real axes, and the yellow 90° arc becomes a labelled 180° semicircle.
  - Labels, the `Im`/`Re` tags and the arrow are clear at 4K.
- **18–35 s (07_2, sheets 4–6).**
  - Heading "derivative ≠ 0: angles kept (conformal)", with the cross and its image marked 90°.
  - The footer reads "names to search: conformal map · critical point".
  - Then come "derivative = 0: a critical point, angles multiplied" and the "precisely" box. At 4K the box text wraps onto two lines inside the frame.
- **37–51.5 s (07_3, sheets 7–9).**
  - `f(z) = |z|² = x² + y²` and "real outputs: one hue, bands for size".
  - The domain plot is one red hue in log-spaced bands. At native size its centre shows clean concentric bands without moiré, and the "0" tag sits below and right of the dot without covering it.
  - The question box and countdown follow, as described under answer timing.
- **51.5–65 s (07_4, sheets 10–11).** The `h̄` formula, the ratio dot tracing `h̄`, the spiral and "ratio → 0.0000". The point 1 appears at about 65 s.
- **65–86.8 s (07_5, sheets 12–14).**
  - At 1, the arrows right and up and the "0"/"1" tags are clear and don't collide.
  - "right: → 2", "up: → 0", the ✗, then "a complex derivative at 0 only".
  - The jagged disc's polar cells resolve cleanly down to the centre marker.
  - The bound formula, the caption and the shrinking ratio circle follow (0.54 down to 0.06, with all dots inside).
- **87–108.2 s (07_6, sheets 15–18).**
  - U with three points; their green disks grow from 91 s to 93 s, then hold.
  - The holomorphic box, "from here on: holomorphic on an open region", the legend, and the footer "names to search: holomorphic function · open set".
  - Then a fade-out. The last two frames are empty.

## Native suspect: disks against the edge of U, cleared

The sheets left it unclear whether the left and lower disks touch U's dashed edge. If they did, the picture would contradict "a small disk where it works everywhere". At 4K they don't.

I measured the distance from each ring's outer pixels to the nearest background pixel outside U:

| Time | Left disk (outer r 123 px) | Large disk (outer r 245 px) | Lower disk (outer r 112 px) |
|---|---|---|---|
| 91.0 s (growing) | 125 px | 257 px | 115 px |
| 92.0 s (growing) | 51 px | 107 px | 48 px |
| 93.0, 94.0, 100.0, 107.0 s (held) | 26 px | 57 px | 25–26 px |

The smallest gap, about 25 px at 4K (about 12 px at 1080p), includes the dashed stroke's inner half. In the crops at 100 s and 107 s, a band of U's fill is visible between each ring and the dashes. No disk crosses or touches the edge in any measured frame.

## Findings

**L1 (Low, pacing; root's judgment). Empty frames in two cue openings.** A frame counts as empty if its 64×36 gray decode varies by at most 2 levels around the background level 14.

| Span (frames) | Time | Length | Cue window |
|---|---|---|---|
| 0–37 | 0–0.633 s | 0.63 s | 07_1 starts at 0.600 s, so the overlap is only 0.03 s. Negligible. |
| 2129–2217 | 35.483–36.967 s | 1.48 s | 07_3 starts at 35.033 s. The 07_2 picture holds 0.45 s into the cue, then the frame is empty until `|z|²` appears 1.93 s after the cue start. The script opens "A subtler question. Take the function that sends z to the square of its length." |
| 5238–5379 | 87.300–89.667 s | 2.37 s | 07_6 starts at 86.833 s. The jagged-disc picture holds 0.47 s into the cue, then the frame is empty until U appears 2.83 s after the cue start. The script opens "The strong results ahead need more." |
| 6489–6490 | 108.150–108.183 s | 0.03 s | End of the fade-out. |

I did not measure where speech starts inside these cue windows: no listening, no WAV onset analysis. So the empty spans may fall partly in lead-in silence. If the narration does start at the cue time, the 07_6 opening plays about 2.4 s over an empty frame. Moving the next picture earlier, or holding the previous one, would close the gap. This is for root to decide against measured onsets.

**L2 (Low, typography). Heavier sans-style glyphs among Computer Modern text.** At 3× zoom of the 4K frames, these signs are drawn with visibly heavier, sans-style strokes than the CM letters around them:

- the `≠` in the 07_2 heading (frame 1440);
- the `²`, `×` and `≤` in the 07_5 caption (frames 4920, 5210, 5220);
- the `²` in the 07_6 legend "|z|²: one point, no disk" (frames 5940, 6360).

The `=` in the same caption has CM weight. The formula's `≤` (frame 4920) also looks somewhat heavier than CM `\leq`, but not clearly enough to call. The pattern fits a fallback face for these Unicode characters, but I proved it from pixels only, not from the font cmap. The same captions also set `z` and `g` upright, while the formulas above them use italic `z`, `h` and `g`. All of it is readable.

**L3 (Low). A red legend key with no red mark in the picture.**
- From 93 s to 107 s the legend "● |z|²: one point, no disk" shows a red dot (RGB about 237, 82, 86).
- No red mark appears in the U picture in any of the 19 samples from 90 s on. Five of them (92, 93, 94, 100 and 107 s) were scanned at full resolution over the left 2300 px, with zero reddish pixels.
- A viewer may look for the red point.
- The legend's words carry the meaning on their own. Fixes would be either to plot a red point (with no disk) or to drop the dot.
- Limit: samples are 1 s apart, so a mark shown only briefly between samples would be missed.

**L4 (Low). Two colour conventions in one panel (07_2).**
- The axes keep their identity colours across the map: real cyan, imaginary green.
- The cross is cyan in the domain and green in the image, which follows the panel-border convention instead (domain panel cyan, image panel green).
- So the green image cross can read as related to the imaginary line.

**L5 (Low, wording).** "|g| ≤ 1 = M" chains an inequality into a definition. "|g| ≤ M = 1" would read more directly. The meaning is clear either way.

## Not checked

- **Listening.** I did no listening. The narration is recorded as ElevenLabs Frederick Surrey in the production records; I did not verify the voice myself.
- **Fine sync.** Every timing statement comes from `cues.tsv` times and sampled frames. I did not measure where speech starts inside a cue.
- **Final AV.** This chapter render is not the master, and I made no loudness, A/V-offset or master checks.
- **Unsampled frames.** Between samples I saw only the 64×36 decode, which detects empty frames and nothing finer.
- **Source.** I read no source code.

## Comparison with root's record

(Added after the sections above were written.) Root's record is `final07-terminal125/pictures-reviewed125.json` (`ab51285fd0de6c28541e28ccfd6154abea0f9d02f79085bdb9faf850b10086f6`, written 2026-11-07 23:48 UTC).

**Where we agree.**
- Root binds the same movie (`a42d77f8…`) and the same `sampling.json` (`daa03221…`).
- Root records 138 selected pictures and sheets 1–18 viewed, which matches my exact count.
- I found no clipping or overlap either.
- The bound formula is intact at native size.
- The smooth `|z|²` field and the jagged bounded example are visually distinct in their roles.
- Root's limits match mine: selected pictures only; no continuous motion, listening, fine sync or independent final AV; master and platform checks still open.

**What this review adds.**
- Root's record lists native details at 5 s and 81 s only. This review measured U's disks against its edge at 91–107 s and found them inside, with margin.
- Root's record does not mention L1 (empty frames for 1.48 s and 2.37 s in the 07_3 and 07_6 cue openings), L2 (heavier glyphs), L3 (a red legend key with no red mark), L4 (mixed colour conventions) or L5 (the chained `≤ 1 = M`).
- None of these contradicts a claim root's record makes. All five are Low. L1 depends on narration onsets that neither record measured.
