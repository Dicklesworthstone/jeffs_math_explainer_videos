# Logarithm preview02, chapters 01 to 04: independent picture review

LavenderPike for BlueSalmon, Mail 1390, 2026-11-04.

## Verdict

Every number I checked on screen is right. This review does not accept any of the four chapters.

Ten defects should be fixed before the next picture render, because the picture says something false or hides text the viewer needs:

| # | Chapter | Frames (time) | Defect |
|---|---|---|---|
| 1.1 | 01 | 281–303 (122.0–131.5 s) | The blue bar under "now one step = ×10" does not run from 1 to 10 |
| 1.2 | 01 | 284–303 (123.5–131.5 s) | The labels "8" and "10" collide and read as "810" |
| 1.3 | 01 | 281–303 (122.0–131.5 s) | `pos(x) = log₂ x` and `pos(3) = 1.585` stay up under a ruler relabelled to base 10 |
| 2.1 | 02 | 70–127, 143–172, 186–237 | A reading line strikes the digit being read: the "8" for 21 s, then the "6" twice |
| 2.2 | 02 | 143–237 | The red end tick runs into "10", and it and "D stops at 10" are never removed |
| 3.1 | 03 | 47–100 (19.4–42.5 s) | "one lap = ×10" loses its arc and reads as the label of the gold tick for 23 s |
| 3.2 | 03 | 145–187 (60.3–77.5 s) | The helix runs through "a spiral, and its shadow" until the chapter ends |
| 4.1 | 04 | 182–238 (79.0–104.8 s) | The red "no logarithm" marks survive the mirror and now sit on outputs ln can reach |
| 4.2 | 04 | 133–175 (57.5–76.2 s) | "ln 2 ≈ 0.693" is printed over the tick labels "1" and "2" |
| 4.4 | 04 | 408–424 (183.5–189.9 s) | "2¹⁰ = 1024" starts on the end of the dashed ×2 line and reads as its label |

The biggest teaching gaps are 4.3 (the product law gets no picture, though the ruler already shows 2, 4 and 8) and 1.4 to 1.6 (equal steps cannot be told apart once they touch).

The native lint flagged 2 of these 10 (2.1 and 3.2). Its 9 lines are ruled on below.

**Boundaries.** This is selected still review only: half-second samples plus cue and lint anchors. I watched no motion and heard no audio, and I make no claim about the full film or about any chapter after 04. Current narration is ElevenLabs Oliver. This render used the scratch voice, so every time here is provisional and the timing findings need re-checking on Oliver's cues. Sentence times come from character shares, counted by hand. All fixes are uncompiled proposals. I edited no source and ran no render, test, Clippy or oracle.

Frame numbers refer to `frame-NNNN.png` in the chapter's folder. Pixel positions are in the 1280×720 frame.

## Chapter 01: `01_building_ruler` (131.7 s)

### Fix before the next render

**1.1 The ×10 bar does not span one to ten.** Frames 281–303; opened alone: 288. The bar runs from x 250 to 545, but the 1 tick is at 316 and the 10 tick at 496. It sticks out on both sides, under the words "now one step = ×10". The squeeze (`building_ruler.rs:297–303`) moves the centres of the three blue lengths, which keep their 2.0 width. Fix: fade those lengths before the squeeze and draw one new bar from 1 to 10 under `ten_step` (`:320`).

**1.2 "810".** Frames 284–303; opened alone: 288. "8" and "10" touch across x 472–508. Frame 283 is already nearly touching. "1 √2 2" and "3 4" are cramped too, and the right two thirds of the spine is empty. The lint is silent. Fix: fade the "8", "√2" and "½" labels before the squeeze. A larger scale also works, but the test `every_mark_from_a_half_to_ten_fits_the_ruler` pins ten on two's old place and would have to change.

**1.3 Base-2 formulas under a base-10 ruler.** Frames 281–303. `pos(x) = log₂ x`, `pos(3) = 1.585`, `log₂ 8 = 3` and `log₂ 3 ≈ 1.585` all stay up after the relabel. On that ruler the position of 3 is log₁₀ 3 ≈ 0.477. Fix: clear them before `:297`. If 0.477 is shown instead, it needs a new key in `facts.rs` and the oracle.

### Teaching

**1.4 Equal doublings merge into one bar.** Frames 29 onward (13.0 s); opened alone: 35. The lengths butt into a solid bar from x 316 to 856, so "every doubling adds the same length" has nothing to count. "one step" sits under the first third and appears to label the whole bar until about 124 s. Fix in `length()` (`:44`): inset each end by about 0.03, or alternate two tones.

**1.5 Two half steps are never visible as two.** Frames 77–98 (34.0–42.1 s); opened alone: 85. `back`, `h1` and `h2` merge into one bar from x 135 to 498, which also touches the top of "one step". The lint is silent. Only the √2 caption carries the claim. Fix: same as 1.4, and move the row down clear of "one step".

**1.6 Twelve threes against nineteen doublings cannot be counted.** Frames 172–243 (75.5–103.6 s); opened alone: 175. The separators are faint on both colours, so the rows read as nearly solid lines. Fix: alternate tones within each row.

**1.7 "19 doublings" margin (lint).** Frames 173–243. The text ends at about x 1266, 0.16 from the edge, and abuts the end of the bar. "12 threes" is centred and does not line up with it. Fix: bring it at least 0.25 inside the edge and align the two labels.

**1.8 The hunter gives away the answer.** Frames 112–135 (48.5–57.0 s). Its last stop at 1.55 almost covers the final position, about 6 px away, during "Where, exactly?" and the think bar. Fix: rest it mid-gap, or at a question mark with no commitment.

### Smaller

- **1.9 Clutter.** Frames 224–240: title, rule, dim caption, ruler, "one step", pos(3), a three-part power row, two labelled bars and the 440 Hz line.
- **1.10 Timing.** Frames 258–279 (110–121 s): the picture is static through "Choose a different length for the first step and you get a different base". There is no `until_sentence` for that sentence.
- **1.11 Rounded equals.** `pos(3) = 1.585` and `19/12 = 1.583` use "=" while `log₂ 3 ≈ 1.585` uses "≈".
- **1.12 Early title.** "Where does three go?" is up from 0.6 s, about 41 s before the voice asks, and stays until 131 s, long after it is answered.
- **1.13 Word against picture.** The voice and caption say "stretched" while the picture compresses.
- **1.14 Missing pictures.** The keyboard callback is one line of text with no keyboard, and "Closer matches between powers" has no picture.
- **1.15 Fade-out nodes.** Frames 97–98 show small bright nodes at the joins of the fading bars.

## Chapter 02: `02_slide_rule` (93.4 s)

### Fix before the next render

**2.1 The reading line strikes the digit (lint, 3 lines).** The question is "Which number sits directly below it?", and the line runs into the answer. Crops opened: 88, 150, 190.

| Frames | Time | What is struck | Position |
|---|---|---|---|
| 70–127 | 30.5–51.5 s | D's "8", the line ends in its lower bowl | (750, 384) |
| 143–172 | 56.7–66.5 s | C's "6", red | (1093, 253) |
| 186–237 | 70.7–93.4 s | C's "6", gold | (374, 253) |

`hairline()` (`slide_rule.rs:54`) runs from `D_Y - 0.62` to `C_Y + 0.62`. Measured on the crops, D's ticks end at `D_Y - 0.33` and its label starts at `D_Y - 0.5`; C's label ends at about `C_Y + 0.47`. Fix: run it from `D_Y - 0.42` to `C_Y + 0.42`, which stops short of both labels.

**2.2 Red end tick and stale caption.** Frames 143–237; crop opened: 150. The `end` tick (`:227`) reaches down to `D_Y - 0.5`, into the top of D's "10", and "D stops at 10" (`:247`) hugs the same label. Neither is in the fade at `:253`, so both stay about 35 s after the idea is over. The lint is silent. Fix: end the tick at `D_Y - 0.42`, drop the text lower, and add `end, stops` to `:253`.

### Teaching and timing (provisional)

**2.3 The slide runs ahead of the voice.** Frames 37–44. The shift plays about 15.6–17.2 s and the ring lands at 17.5 s, but "slide" is spoken at about 18.5 s. Fix: `kit.until_phrase(stage, "02_2", "slide the moving")` before `:112`.

**2.4 0.398 has no unit.** Frames 37–40 onward. It appears at 15.1 s, after the "one full length" caption has gone at about 11.5 s (frames 26–27). `pos(8) = 0.903` also follows chapter 01's on-screen `pos(x) = log₂ x`. Fix: caption at `:84`: "one full length = 1: from 1 to 10, ×10".

### Smaller

- **2.5 Stale card.** Frames 127–132: the voice says "Now try four times six" at 51.4 s while the card shows "2.5 × 3.2 = 8" until about 53 s. Fix: swap the card (`:217`) before the shift home (`:208`).
- **2.6 No slide on "slides".** Frames 7–36: the voice says "one that slides along it" and nothing moves until 15.6 s.
- **2.7 C's name off frame (lint).** Frames from 68.4 s to the end: C's "1" and "C" are off frame. The test says C's 1 is deliberate, but the scale loses its name for 25 s. Fix: re-place the name near C's 10.
- **2.8 C's "10" margin (lint).** Frames 137–175: 0.24 against a wanted 0.25. Harmless; accept it.
- **2.9 "one mark".** Frames 208–237: this dim label sits beside "24" only, with no bracket over the three rows it describes.

## Chapter 03: `03_folded_decade` (77.7 s)

### Fix before the next render

**3.1 Orphaned "one lap = ×10".** Frames 47–100; crop opened: 88. `swap` at `folded_decade.rs:218` removes the arc and leaves the words, which stay until 42.5 s as the only text beside the gold tick and ring. The lint is silent. Fix: fade `lap_word` with the arc and take it out of `clear` (`:321`), or move it beside the lap counter so it serves "The extra lap is the factor of ten".

**3.2 Helix through the caption (lint).** Frames 145–187; crop opened: 170. The top lap crosses "and its shadow" at (345, 136). Fix: put the caption (`:391`) at about y −3.55 under the floor, or to the right of the spiral. The test `the_spiral_readings_share_one_shadow_and_fit_the_frame` covers the spiral, so run it.

### Teaching and timing (provisional)

**3.3 The dial never says 2.4.** The gold tick has no number from 13.0 s, and "2.4" appears only inside "4 × 6 = 10 × 2.4 = 24" from frame 80 (35.5 s), though the voice says "stop on two point four's mark" at about 31 s. After frame 101 (43.0 s) every dial numeral is gone, so "one mark on the dial" names no number. The tick also shows the landing place before the turn. Fix: place a small "2.4" at the ring on landing, and caption the floor "the 2.4 mark: one place on the dial".

**3.4 Stale title.** Frames 100–143 (42.5–60.0 s): "Multiplying is turning" stays over all of 03_3 and the spiral. Fix: move the swap at `:396` to the start of 03_3, whose first sentence says "forgets the laps".

**3.5 One-second caption.** Frames 96–100: "the dial shows only the mark" (`:315`) fades in at 41.9 s and is going by 42.5 s, nearly touching "one lap = ×10". Fix: hold it through the first sentence, or drop it.

**3.6 Tick in the caption (lint).** Frames 117–187; crop opened: 126. The tilted tick ends inside the "o" of "one mark on the dial" at (547, 631). Fix: move the text right by 0.25 at `:352`.

**3.7 Unlabelled right panel.** Frames 155–176 (from 63 s): the column, ring and dots have no label for about 11 s during "complex numbers". The formula and caption arrive at frame 177 (74 s), 3.7 s before the chapter ends. Fix: bring the caption at `:422` forward to the second sentence.

### Smaller

- **3.8 Unvoiced row.** Frame 133 onward: "240 / 2 laps" appears on a sentence that names only 0.24.
- **3.9 Tight readings.** Crop 170: "24", "2.4" and "0.24" tuck under the loops within 2 px, with no ink crossing. Fix: move the numbers right by 0.2.
- **3.10 Irregular dashes.** Crop 170: four overlaid dashed drops of different lengths read as a broken line in the upper half. Fix: one dashed line from the top point.
- **3.11 End state.** Frames 180–187: 13 texts and two pictures split the eye.
- **3.12 Pointer pop.** Frames 47–48: the pointer appears without a fade, and "0" arrives before its label.
- **3.13 Detached value.** The gap between "laps past the top:" and its value looks detached.
- **3.14 Sparse frames.** Frames 104–107 (44.5–46 s): flat ellipse and title only.

## Chapter 04: `04_undoing_growth` (190.1 s)

### Fix before the next render

**4.1 The red marks survive the mirror.** Frames 182–238; opened alone: 210, plus a crop. `floor` (`undoing_growth.rs:216`) and `nowhere` (`:326`) are not in the 04_4 fade (`:332`). They were true of e^x:

- The solid bar on the negative y-axis meant "outputs e^x never reaches".
- After the flip that axis holds ln's outputs, and every value on it is reached. The curve's tail runs beside the bar to −2.6 (x 352 against 358).
- The dashed floor now runs through the curve and under the gold dot at x = 1.
- The inputs that really have no logarithm, the negative x-axis, are unmarked.
- The caption that explained the red ("0 and below: no real logarithm") faded at 76.2 s.

Fix: move `floor, nowhere` to the fade at `:332` and take them out of `graph` at `:407` so they are not faded twice. A richer option is to reflect them with the mirror, so the bar lands on the negative x-axis and the floor becomes ln's wall on the y-axis.

**4.2 "ln 2 ≈ 0.693" on the tick labels.** Frames 133–175; crop opened: 140. The text spans x 425–549, y 427–446. The tick "1" sits on "ln" and the tick "2" crosses the "9". The lint is silent. Fix: move `ln2_word` (`:301`) to `p(1.5, -0.75)`.

**4.4 "2¹⁰ = 1024" reads as the dashed line's label.** Frames 408–424; opened alone: 412, plus a crop. The text starts at x 750, directly above the end of the ×2 line at x 758, and overhangs the axis, which ends at 767. A line meaning ×2 now seems to be labelled 1024. The lint is silent. Fix: two rows at `:617`, "2¹⁰ = 1024" and "10 doublings ≈ 138.6 years", centred near x 4.4.

### Teaching

**4.3 The product law has no picture.** Frames 258–280 (114.5–125.5 s). "The position of a product is still the sum of the positions" gets a formula and then 11 still seconds, on a ruler that already carries 2, 4 and 8. Fix at `:442`: lay a bar from 1 to 2 end to end with a bar from 1 to 4 so it lands on 8, and label it 2 × 4 = 8. This needs no new number.

**4.5 "Only ever climbs" is frozen.** Frames 86–105 (39.0–48.5 s). After the caption "slope = height > 0: always climbing" arrives, nothing moves for about 10 s through "it only ever climbs, from near zero on the left to as large as you like on the right". Fix: run a dot or tangent up the curve during that sentence (after `:233`).

**4.6 The triangle never shows the trade.** Frames 188–238. "rise and run trade places" and "slope one over x" have an unlabelled triangle, static for about 6 s from 82 s. Fix: label the legs "rise" and "run" before the flip and swap the labels after it.

**4.7 Hollow arrows, all three chapters that use them.** `draw::arrow` (`draw.rs:175`) strokes the arrow's outline with no fill, so it renders as a double shaft with an outlined head. The dial pointer, built with `arrow_parts` (`draw.rs:182`), is solid.

- Chapter 04, crop 140: the down arrow, frames 132–175, call at `:300`.
- Chapter 03, crop 17: the ×10 arrows, call at `folded_decade.rs:167`.
- Chapter 02, crop 59: the `carried` arrow, shaft seen, head outside the crop, call at `slide_rule.rs:117`.

Fix: make `draw::arrow` build the solid shape.

### Smaller

- **4.8 Name left behind.** Frames 184–186: "y = e^x" stays after the curve has left. Fix: fade it as the flip starts.
- **4.9 No mark for ln 1 = 0.** From 63.5 s: the gold dot at (0, 1) lost its label at 19.8 s and is not tied to this line. Fix: pulse the dot.
- **4.10 Stale law.** `e^{a+b} = e^a e^b` stays from 9.5 s to about 76 s. The end of 04_3 has 8 text items.
- **4.11 ln is never named.** The words "natural logarithm" never meet "ln" on screen, and the `back` formula appears at 55.5 s, before the voice names it at 56.3 s.
- **4.12 Blank stretches.** See the table below.
- **4.13 Ratio ruler.** Frames 316 onward: "growth factor, on a ratio ruler" is the only sign the y-axis doubles. The voice never says so, and the line fades in whole rather than growing.
- **4.14 Unnamed results.** "rule of seventy" is spoken but not on the card. The thousandfold dot and card never say ×1000, and the dot sits on the tick labelled 1024.
- **4.15 Far-off think bar.** Frames 340–348: the bar is about 380 px below "×2 when?". Fix: place it under the question (`:539`).
- **4.16 Tangent over triangle.** Frames 201–238: the gold tangent crosses the blue triangle after the triangle's job is done.
- **4.17 Rounded equals.** "ln 1.01 = 0.00995" (true 0.0099503) and "100 × ln 2 = 69.3" use "=".
- **4.18 Small layout items.** The right-column rows have four different left edges (04_3). The green dots at x = ±1 (frames 53–123) are never tied to e^x e^−x. e_down and the gold dot crowd the "y" of "y = e^x". The ln curve stays green under a blue name. "log₂:" (`:460`) uses a small on-line 2 (crop 285), while the formula has a true subscript; use `kit.tex`. The drop at 13.9 has no label at its foot. The right third of the frame is empty in 04_5, and "the same ruler stretched" shows the same positions relabelled.

## Native lint: all 9 lines

| Chapter | Line | Looks after | Ruling |
|---|---|---|---|
| 01 | 01_4 +19.2 s, tight, "19 doublings" 0.16 from edge | 138 | Real, 1.7 |
| 02 | hold @30.7 s, crossed, over "8" | 107 | Real, 2.1 |
| 02 | 02_4 +9.5 s, tight, "10" 0.24 | 58 | Harmless, 2.8 |
| 02 | 02_4 +11.5 s, crossed, over "6" | 46 | Real, 2.1 |
| 02 | 02_5 +9.7 s, outside, group holding "1" | 121 | Intended by test, name lost, 2.7 |
| 02 | 02_5 +12.1 s, crossed, over "6" | 109 | Real, 2.1 |
| 03 | 03_3 +14.3 s, behind, "one mark on the dial" | 134 | Real, 3.6 |
| 03 | 03_4 +3.7 s, behind, "a spiral, and its shadow" | 82 | Real, 3.2 |
| 04 | before 04_3 @54.6 s, crossed, over "2" | 1 | Confirmed, harmless: the probe at height 2 crosses the axis label, frames 120–122, and fades |

The lint found none of 1.2, 1.5, 2.2, 3.1, 4.2 or 4.4, and cannot find 1.1, 1.3 or 4.1, which are about meaning. A clean lint on these chapters would not have been a clean picture.

## Blank transitions

No frame is fully blank after a chapter's first two frames. Each chapter opens with frames 0–1 empty and frame 2 faint, and each ends on held content with no blank tail.

| Chapter | Frames | Time | On screen | Voice |
|---|---|---|---|---|
| 04 | 239–241 | 105.0–106.0 s | Title only, about 1.5 s | "And this is the ruler again" |
| 04 | 311–314 | 138.1–139.5 s | Title only, about 1.9 s | "Here is the inverse at work" |

The first should be fixed by showing the ruler's spine on that sentence. The second is acceptable.

Every `swap` overlaps old and new text for about 0.5 s in stills (for example chapter 02 frames 132 and 218, chapter 03 frames 45–46 and 144, chapter 04 frames 350–351). Stills cannot judge how that plays in motion, so I have not called it a defect.

## What holds

These read well in stills and should be left alone:

- Chapter 03: the bend from 13.0 to 15.6 s, the dial angles, the lap counter passing the top at frame 68, and the spiral drawing.
- Chapter 04: the live tangent (12–16.5 s), the probe with its count (frames 106–122), and the mirror itself. Sync on the sampled times is good from 04_3 onward, for example ln 1 = 0 at 63.5 s, the ×2 line at 148.5 s, the hit at 164.5 s and the card at 183.5 s.
- Chapter 04's think bar: the answer stays hidden until it drains.
- The doubling panel: its line reads about 12 at 50 years (e^2.5 = 12.18), and exactly 10 gold dots land on it.

**Numbers checked by arithmetic against what is on screen:**

- 3¹² = 531,441; 2¹⁹ = 524,288; 1.36%
- log₂ 3 = 1.585; 19/12 = 1.583; 440 × 2^(19/12) = 1318.51
- log₁₀ of 2.5, 3.2, 8, 4, 6, 24 = 0.398, 0.505, 0.903, 0.602, 0.778, 1.380
- ln 1.01 = 0.00995; ln 2 = 0.693; ln 1000 = 6.908; e = 2.718
- t = 13.86 years; 138.16 years; ten doublings 138.63 years; 100 ln 2 = 69.3

## Evidence

| Chapter | Sheets read | Frames carried | Sheet set digest | Frame set digest |
|---|---|---|---|---|
| `01_building_ruler` | 76 of 76, 000–075 | 304 | `4287a119d9100ddf349ee856f73295741010592383ea834c98299be15a120911` | `e77fab06f925fc866ae335c26bba5b3dac7b6c03c35eb64a93dcea1946e6b69a` |
| `02_slide_rule` | 60 of 60, 000–059 | 238 | `5b0baab87609364aefd132a7113eea4833d634caf72fbbcd8ed68c2010a28c4e` | `36c6f1732ab501c4d27c238c1f5d8282ef0cec3f27d63d13eccb93b3fbdb1e20` |
| `03_folded_decade` | 47 of 47, 000–046 | 188 | `e14c9459625b49c652b0c0242bda016952211b37c47251ffc96f2677d66a0fa2` | `d3681efd7eb0fceb090c40286abda4ee590947a27d46949e916ccf2bc0948c9d` |
| `04_undoing_growth` | 107 of 107, 000–106 | 425 | `549457cf1e92e21d780cc6186042d31b1dbf942e14ac3f904c33d230a2792526` | `f8fe94607f8cc49bd37ca92eb0b3c57adccede21ba0d8d4162e31ad27c187a56` |

All 290 sheets were opened as images, in order. Each carries 4 frames, except that the last sheet of chapter 02 carries 2 and the last of chapter 04 carries 1. Frame folder: `narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-opus-rebuild-frames02/`.

Each set digest is the SHA-256 of the lines `<sha256>  <file name>\n` in file-name order. That is also the SHA-256 of the matching file in `lists/`, which gives every file's own hash.

**Opened alone at 1280×720:**

| Chapter | Whole frames | Enlarged crops |
|---|---|---|
| 01 | 35, 85, 175, 288 | 288 |
| 02 | none | 59, 88, 150, 190 |
| 03 | none | 17, 88, 126, 170 (two crops) |
| 04 | 210, 412 | 140 (two crops), 210, 285, 412 |

`lists/bindings.sha256` holds the hashes of the script, chapter sources, `draw.rs`, `facts.rs`, `model.rs`, `oracle.py`, and each chapter's `sampling.json`, mp4, cues and layout file.

**Binding.** The nine script and source files in `bindings.sha256` matched `logarithm-opus-rebuild-preview02-inputs.json` at 09:39Z, and their hashes were unchanged when I re-hashed them at about 10:07Z. So the source line numbers above apply to these frames.
