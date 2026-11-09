# cc0 final06 picture review 224: chapter 06, one derivative

Dispatch-0 (cc0), 2026-11-08T04:35Z. **Independent review of rendered stills only.** I rendered, extracted, built and synthesised nothing. This is the only file I wrote in the repository. For viewing and measuring I made crops of the existing pictures, kept outside the repository. I wrote my own pass before I opened root's receipt.

## Subject

- **Film.** `final06-terminal169/06_one_derivative.mp4`, sha `fe9ab36f4000500e35d71a232b6163f65adb58167aba82634e010b2d0b97e875`, recomputed at 04:27Z. It has 10,030 frames and is 11,701,301 bytes.
- **Render summary.** `render-summary.json`, sha `100fdab5…cc8cb8`: binary `f4836eee…` (production19), job `ts2-production19-final-06-one-derivative123`, x264 CRF 16.
- **Tables.** The timeline (`a31f6c1a…`), cues (`ee8eb35a…`), cue log (`67c75f38…`) and layout (`e3b0c442…`, empty) match the render summary.
- **Extraction.** `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-final06-pictures169/`. Its `sampling.json` (sha `0150623d…`) lists 209 native 3840×2160 pictures and 27 sheets.
  - I checked all 236 file hashes against it: 0 mismatches.
  - Its `movie_sha256` equals the film's.
  - The pictures on disk are exactly the listed ones.
  - The extraction receipt (`resource-receipt.json`, sha `4a86f75a…`) shows exit 0, final inputs equal to the admitted inputs, run 01:56:54Z–01:57:45Z.
- **Script.** Lines 06_1–06_9 of the exact bound production script: `production-source-preserved214/workspace/explainer_corpus/complex_analysis_visual/script.tsv`, sha `234f7257…813d3c`. I checked what is shown against these words and their cue times. I did not hear the audio.

## Result

**No blocking defect.** In every sampled picture, every number, label and formula agrees with direct calculation and with the bound script. The quiz answer stays hidden until the think time ends. Every label is legible at native size, and nothing is clipped. I found two Low–Medium items and one Low item:
- **F1:** the output panel is drawn at three quarters of the input's scale.
- **F2:** two general sentences in 06_3 and 06_7 leave out critical points.
- **F3:** the heavy minus and times glyphs.

None of them needs a re-render for release. F1 is the one I would fix if chapter 06 is reopened.

## Findings

**F1, Low–Medium: the drawn stretch is about 2.1×, while the label says 2.83 (10–75.9 s, lines 06_1–06_4).**

The output panel uses a smaller scale than the input panel. Nothing on screen says so. I measured picture 74 (60 s) at native size:
- **Input scale.** The input axes sit at x = 430 and y = 1759, and the dot at z0 = 1 + i is at (947.5, 1241.5). That gives 517.5 px per unit both ways. The input square has side 259 px, which is 0.500 units, so its half-width is s = 0.25.
- **Output scale.** The teal "exact stretch and turn" diamond has half-diagonal 387 px. For s = 0.25, its exact half-diagonal is |f′|·s·√2 = 1.000 unit. The output scale is therefore 387 px per unit, 0.748 of the input's. The green image's vertices agree with this scale.
- **What the viewer sees.** The 259 px square becomes a diamond with 547 px sides, so it is drawn 2.11 times larger. The label "stretch 2.83, turn 45°" (from 60 s) and the voice ("stretches by about two point eight") give the true 2√2. The area looks about 4.5 times larger instead of 8.
- **The arrows.** In picture 64 (51 s), the turned output arrows are about 2.1 times the input arrows.
- **The change arrow.** In picture 18 (17 s), the output change arrow is about 2.5 times h, where the true ratio |2 + 2i + h| is about 3.3.

Neither panel has ticks, and both are the same size, so a viewer will read them as one scale. The grey note "zoomed in about 2i to keep the image's size" appears at 71 s. It describes the later tenfold zoom during the shrink, not this factor.

The turn (45°) and the convergence readout are unaffected. At the input's scale, the first output change arrow (about 1.47 units at 10 s) would leave the panel, which is probably why the scale was reduced. If 06 is reopened, any one of these would repair it:
- add a note such as "output drawn at ¾ scale", from 10 s;
- add matching ticks to both panels;
- start with a smaller h and draw both panels at one scale. The "the image / exact stretch and turn" legend would then need to move, because the image reaches 2i ± 1.125.

Each fix is a picture change in 06 only. The narration stays as it is.

**F2, Low–Medium (wording): 06_3 and 06_7 state the stretch-and-turn picture without the condition f′(z) ≠ 0.** This is the critical-point check.
- **06_3.** "So a tiny square around z lands as a tiny square, bigger and turned, but still square." This fails at a critical point. There f′(z) = 0, and the local map collapses squares instead of stretching and turning them. "Bigger" also holds only where |f′(z)| > 1.
- **06_7.** "The local map must be a stretch and a turn" has the same gap. At a critical point the local map is zero.
- **The same function in this film.** Chapter 03 of this film shows z² keeping right angles "except at 0", and 0 is a critical point of z².
- **What chapter 06 shows.** No picture in 06 shows a critical point. The only point examined is 1 + i, where f′ = 2 + 2i, and there both sentences are true.

So no picture is wrong; the general sentences are. If 06 is reopened, a small on-screen qualifier "(where f′(z) ≠ 0)" under the 06_3 square, and on the 06_7 caption, would fix it without new narration. Changing the words would need a new Frederick take.

**F3, Low: the minus and times glyphs do not match the Computer Modern "+" and "=".** This is the same pattern as review212's F2 for "≠". The "−" is heavier and shorter, and the "×" is heavy and sans-like. They appear here:
- the difference quotient (10–75.9 s; native crop of picture 64);
- "× f′(z0)" (from 46 s; picture 64);
- "v_y − i u_y" (87–129 s) and the boxed "v_x = −u_y" (104–129 s; picture 141 half scale);
- the "−2" in the matrix (118–129 s);
- "= ×(2 + 2i)" (125–129 s; native crop of picture 157).

All of them are legible, and the meaning is clear. A glyph probe would settle whether a fallback face is in use. The text-mode dash in "up: −1" matches its serif text.

**Notes (not defects).**
- **N1.** The red "no complex derivative" from the reflection stays on screen through the switch to the squash (159–161 s, pictures 200–202). The squash ratios appear at 162 s and the red X at 163 s. The voice says "Squashing fails too" in this span, so it reads as the new verdict. I did not check word timing.
- **N2.** The moving ratio readouts (22–33 s) use fixed slots with wide gaps ("1.64   + 2.18   i"), two decimals and an upright i. The settled readout has three decimals: "→ 2.000 + 2.000 i".
- **N3.** At sheet scale, the "change in the output" label keeps its height while the arrow shortens (16–19 s).
- **N4.** The "the ratio" inset ends 82 native px (2.1%) from the right edge, and its readout ends 104 px from it. Both are inside the frame.
- **N5.** The squash note "image drawn moved down by i" fades in at 162 s, one sample after the rectangle starts to grow (161 s). The note is correct (see 06_9 below).

## Mathematics checked

Measurements are from native pictures where stated, otherwise from the sheets.

| Line (cue) | Checked | Result |
|---|---|---|
| 06_1 (0.6–21.0 s) | Labels "z0 = 1 + i" and "z0² = 2i" are on the correct points. h ≈ 0.37 + 0.255i (native, picture 18). The output change arrow reads 0.38 + 1.44i against the exact (2 + 2i)h + h² = 0.31 + 1.44i; the tip was read at half scale, which explains the difference. "→ f′(z0)" appears from 17 s. | agrees; F1 |
| 06_2 (21.6–36.3 s) | The quotient for z² is exactly 2z0 + h = 2 + 2i + h. From the right: 2.45, 2.30, 2.10 + 2.00i. From above: 2.00 + 2.45i, 2.25i, 2.04i. Along a spiral: 1.64 + 2.18i, with measured h = −0.36 + 0.185i (native, picture 40), then 1.99 + 1.81i, then 2.00 + 2.00i. Each value equals 2 + 2i + h. "→ 2.000 + 2.000 i" from 34 s. | agrees; N2 |
| 06_3 (36.9–59.1 s) | Four small arrows at about 22°, 112°, 202° and 292° map to 66°, 154°, 245° and 337°. That is +45° = arg(2 + 2i). "× f′(z0)" (46 s) and "a stretch and a turn" (49 s) match the words. | turn agrees; F1 length; F2 wording |
| 06_4 (59.7–74.8 s) | "stretch 2.83, turn 45°" = 2√2 and an eighth of a turn. The green image's vertices sit at 2i + 1.125i, 2i − 0.875i and 2i ± 1 − 0.125i (measured 1.13, 0.876 and 0.124 output units). These are 2i + (2 + 2i)δ + δ² at the corners of the s = 0.25 square. The teal diamond is the linear part. The shrink is to s = 0.025. "largest gap ÷ image side: 0.088 → 0.0088" = 2s² / (4√2·s) = s / (2√2) = 0.0884 and 0.00884. The zoom note is at 71 s. | agrees; F1 |
| 06_5 (75.4–96.3 s) | "z = x + i y", "f = u + i v", "step right: u_x + i v_x", and "u_x is how fast u changes as x changes". "step up: (u_y + i v_y)/i = v_y − i u_y": dividing by i multiplies by −i, a quarter turn back. The "must agree" bracket appears at 95 s. | agrees; F3 |
| 06_6 (96.9–110.3 s) | Equating the two lines gives the boxed u_x = v_y (98.9 s) and v_x = −u_y (104 s), in the order spoken. The banner "names to search: Cauchy–Riemann equations" shows at 108–111 s, with 106–108 px margins. | agrees; F3 |
| 06_7 (110.9–128.2 s) | Matrix [[u_x, u_y], [v_x, v_y]], captioned "a stretch and a turn: no squash, no reflection" (111.9 s). "= [[2, −2], [2, 2]]" (118 s): u = x² − y² and v = 2xy give u_x = 2, u_y = −2, v_x = 2 and v_y = 2 at 1 + i, matching the spoken "two, minus two, two, and two". "= ×(2 + 2i)" (125 s): multiplication by a + bi has matrix [[a, −b], [b, a]]. | agrees; F2, F3 |
| 06_8 (128.85–140.45 s) | Above the axis, a cyan square with an anticlockwise arrow. The gold mirror axis appears at 131 s. Below, a green copy at the same distance with a clockwise arrow, so orientation is reversed. "x + iy ↦ x − iy" at 135 s. "a derivative anywhere?" at 139 s. | agrees |
| Quiz | The ring appears at the predict mark, frame 8463 (141.05 s, picture 175, full). It is nearly empty at 146.0 s (picture 183, only its end dot left) and gone at 146.3 s (picture 184), with the box still up. The think-time row is 5 s from frame 8463, ending at 146.05 s. No sample before then shows any answer. "right: 1" first appears at 148 s. | answer hidden until think time ends |
| 06_9 (146.3–164.06 s) | "right: 1" (148 s) and "up: −1" (151 s): conj(h)/h is 1 for real h and −1 for h = it. The red X and "no complex derivative" appear at 154 s. Squash "x + iy ↦ x + 2iy" (159 s): measured on picture 205 at half scale, the square centred at 1 + i has side 137 px (228 px per unit), and the image is 274 px tall, twice the side. The image is drawn centred on 1 + i, while its true centre is 1 + 2i, so "image drawn moved down by i" is exact. "right: 1 up: 2" (162 s), then the X (163 s). | agrees; N1, N5 |
| Ends | Picture 1 (frame 0) and picture 209 (frame 10029, 167.15 s) are uniform RGB (12, 17, 20). The fade runs at 167.0 s. | agrees |

**Square deformation against true differentiability.** The film contrasts three local maps on one small square:
- **Squaring at 1 + i** keeps the square square, turns it by 45° and converges to the linear diamond as s shrinks.
- **The reflection** keeps the square's shape but reverses its orientation, shown by the arrow.
- **The squash** changes the shape to 1:2.

These match the 06_7 caption "no squash, no reflection" and the words of 06_9. Apart from F1's scale factor, the drawn shapes are the true images. The film draws a single square, not a grid.

## Readability and layout

- **Text.** All titles, panel headers, captions, matrix entries, readouts, the banner, the quiz box and the verdicts are legible at native size. The smallest text is grey: "u_x is how fast u changes as x changes" (82–129 s), the zoom note (71–75.9 s) and the squash note (162–167 s). All three are legible at half scale (pictures 88, 141 and 205). I did not test phone size.
- **Clipping and overlap.** I found no clipping and no unintended overlap. Only N4 is close to an edge.
- **Transitions.** The sampled mid-motion pictures show the fades at 75.9, 129.0–129.35, 159 and 167 s and the mirror flip at 132 s. Each resolves to the correct settled picture in the next sample.

## Coverage

- **Sheets.** I read all 27 sheets, which together show all 209 pictures at 960×540. I logged them sheet by sheet as I went. The sampled line starts (06_6 picture 118, 06_7 picture 136, 06_8 picture 158, 06_9 picture 184), the predict mark (picture 175) and the last frame (picture 209) are all among them.
- **Native views.**
  - Whole pictures at half scale: 18, 40, 64, 74, 88, 92, 133, 141 and 205.
  - Crops: the formula in picture 64, the matrix in 157, the ratios in 195, and the quiz area in 175, 183 and 184.
  - Pixel measurements: rows and columns of pictures 18, 64 and 74 for the panel scales, plus pictures 1 and 209.

## Agreement with root

After writing the sections above, I read root's receipt: `final06-terminal169/pictures-reviewed169.json`, sha `9333f711…4e4b93`, written at 2026-11-08T01:58:41Z, for the same film sha, covering all 209 pictures and all 27 sheets. I agree with each of its four observations:
- the difference quotient and the paths are legible;
- the Cauchy–Riemann derivation and the matrix are clear;
- both counterexamples are readable, and the answer is withheld through the pause;
- there is no persistent clipping or overlap.

The receipt does not mention F1–F3. Its phrase "local square and approximation clear through zoom" is true of the shapes. F1 concerns the scale between the two panels, which the receipt does not address.

## Limits

- **Sampled stills only.** I checked 209 of 10,030 frames, about 2%. I make no claim about motion between samples or about how long any state lasts.
- **No audio heard or checked.** Per root, the narration is ElevenLabs Frederick Surrey. I did not verify the soundtrack (`9b2afe3f…`, not present here), its loudness or its sync.
- **No captions or fine sync.** I did not check the captions, and no fine sync was checked. Word timing within cues (N1, quiz reveal) is unverified.
- **Not final acceptance.** This is not independent final AV acceptance of chapter 06 or of the master, and it is not publication acceptance.
