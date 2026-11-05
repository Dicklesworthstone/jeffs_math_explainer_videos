# Logarithm stage06 pictures, chapters 05 to 08

Reviewer: LavenderPike, for BlueSalmon's Mail 3079. Evidence bundle only; no source was edited.

## Answer

I opened all 392 selected-picture sheets, which hold all 1,557 pictures. The stills show no blocker:

- The mathematics on screen was right everywhere I checked it against the drawn geometry.
- All seven think gaps keep the answer hidden.
- None of the 14 lint anchors is a visible defect at its point.
- No text is clipped, and no text touches a stroke or other text in a way that hurts reading.

The main weakness is timing. In many beats the picture states a result several seconds before the voice does, then holds still while the voice catches up. Three findings deserve repair before takes are fine-synced:

1. **C1, chapter 05:** the worked answer `2i` is on screen about 9 to 10 s before "so we land on two i".
2. **L1, chapter 06:** the gap jumps and the leap is captioned about 3.6 to 5.7 s before the sentence reaches them. This is the chapter's key moment.
3. **R10, chapter 08:** the ring never changes after 125.0 s, so the half turn that explains the puzzle is never drawn.

This is not acceptance of the film. The limits are stated below.

## Coverage

Every sheet was opened as an image, in order, in the batches recorded in `viewing-log.tsv`. `manifest/sheets.tsv` gives each sheet's SHA-256, byte size, frames and times; the seal script checked every sheet against it.

| Chapter | Sheets viewed | Pictures | Length | Cues | Lint anchors | Sheet 000 SHA-256 starts | Last sheet SHA-256 starts |
|---|---|---|---|---|---|---|---|
| 05_complex_turns | 000–106 (107) | 425 | 181.900 s | 8 | 6 | `81d19482a09378ac` | 106 `7c3bb5e5b76e1f58` |
| 06_lifting_loops | 000–104 (105) | 417 | 178.400 s | 8 | 4 | `c75930f1700b6709` | 104 `cb96b193e4abef96` |
| 07_branch_cut | 000–097 (98) | 389 | 170.233 s | 6 | 4 | `a0829d3c05daa895` | 097 `352b4c9bef126e72` |
| 08_product_rule | 000–081 (82) | 326 | 148.433 s | 5 | 0 | `4d8466b5f87b0732` | 081 `51b10068b153b5c6` |
| **Total** | **392** | **1,557** | | | **14** | | |

The last sheet of chapters 05, 06, 07 and 08 holds 1, 1, 1 and 2 pictures; the blank panels are expected from those picture counts.

The source files under `explainer_corpus/logarithm_mysteries/` match what the preview's `inputs.json` hashed, so line pointers below describe the code that made these pictures:

`script.tsv` `f201d2dc57fa`, `docs/storyboard.md` `7db2b8edeed4`, `src/model.rs` `8c9e3c6abfcc`, `src/chapters/draw.rs` `2a4e84504bab`, `complex_turns.rs` `2cc2ba7723e2`, `lifting_loops.rs` `ca34aef0523c`, `branch_cut.rs` `193c4733de9f`, `product_rule.rs` `e9019c828710`.

`checks/build_manifest.py` rebuilds the manifest and prints that comparison.

## How to read the numbers

- **f** is the picture number, as in `frame-NNNN.png`. Times are the preview's, from each panel's label.
- **Sheet coordinates:** x runs 0 to 960 across a panel and y counts down from the top of the panel's 30 px label bar. To get a pixel in the 1280×720 frame, take (x, y − 30) × 4/3. Figures marked "at 1280" are already in frame pixels.
- **Phrase times are estimates**, not word alignment. `checks/word_share.py` spreads each cue's length over its words, then over its letters, and `checks/word_share.out` prints both. Where the two differ I quote a range. They can differ by 2 s (07_2). A larger gap between picture and phrase is therefore firmer than a small one.
- **Timings belong to this preview's voice.** They will move with final takes, but a reveal placed at a sentence start stays ahead of a phrase late in that sentence.
- **Fix directions are suggestions.** None was compiled or tested.

## Findings

### Medium

| ID | Cue | Frames, time | What the pictures show | Source | Fix direction |
|---|---|---|---|---|---|
| **C1** | 05_20 | f134–f136, 60.0–60.6 | The worked example arrives whole, with `= 2i`, its arrow and the `2i` label together, as "For instance" starts (59.4–60.3). "the length is two" is 66.1–66.3 and "so we land on two i" is 69.6–70.2, so the answer leads by about 9 to 10 s. Nothing moves from 61 to 72.7. | `complex_turns.rs:349-366` | Stage x = ln 2 and y = π/2 on their words, then the arrow and `2i` last, on "land on two i". |
| **L1** | 06_6 | f268–f271, 114.0–115.5 | The dot crosses −1 and the gap drops at about 114.4, during "The gap sits at zero until" (113.3–115.7). "passes minus one" is 116.4–117.8. "jumps down a lap" is 118.0–118.1, about 3.6 s after the drop. The arrow and caption are up by 115.5; "leaps from pi to minus pi" starts 121.1–121.5, about 5.7 s later. Static 118 to 126. | `lifting_loops.rs:585-594` | Hold at turn 0.45 through "sits at zero until the number", cross on "passes minus one", bring the inset on "because" or "leaps". |
| **R10** | 08_5 | f276–f325, 125.0–148.183 | The ring stops changing at 125.0: green dot on −1, two green arcs. The voice then gives the product's angle zero (130.3 on) and "a half turn is a factor of minus one" (138.1–141.8). The ring has no mark at +1, and no half turn is drawn between the two answers. Five rows of text carry the whole explanation. This is a teaching judgement rather than an error. | `product_rule.rs:394-449` | At `:428` add a gold dot at +1 with the gold row. At `:438` draw a purple half-turn arc from +1 to −1 with `halved`, so the sign has a picture. |

### Medium-low

| ID | Cue | Frames, time | What the pictures show | Source | Fix direction |
|---|---|---|---|---|---|
| **B1** | 07_2 | f96, 41.5 | Both worked examples arrive together: gold dot, `Log i = πi/2` x 263–345 y 235 and gold arc; purple dot, `Log(−i) = −πi/2` x 263–369 y 403 and purple arc. "Log of i" starts 44.2–46.2 and "Log of minus i" 48.1–49.0, so they lead by about 3 to 5 s and 7 s. Static 41.5 to 53.03. Values and arc directions are right. | `branch_cut.rs:338-362` | One at a time, on their words. |
| **R1** | 08_3 | f139–f140, 62.5–63.0 | `slide rule: keeps the digits, drops a factor of 10` x 520–827 y 137 and `Log: keeps the direction, drops a lap of 2πi` x 520–800 y 168 arrive together. "drops a lap" is 67.0–67.6 and "dropped a factor of ten" 70.5–71.1. The screen puts the slide rule first and the voice puts Log first. No motion 63.0–72.5, and the sum is never shown running past the window. | `product_rule.rs:274-289` | Log row on "drops a lap", slide rule row on "factor of ten", or swap the rows. A sweep of the sum past the window would fill the hold. |
| **R2** | 08_2 | f96–f97, 43.5–44.0 | `Log z + Log z = 3/2 πi,  Log(z²) = −½ πi` x 545–901 y 259 arrives whole. The product half leads "while the logarithm of the product" (48.9–49.2) by about 5 s. Nothing moves 44.0–53.5. | `product_rule.rs:238-251` | Split into two mobjects, the second on "while". |
| **R3** | 08_3 | f160–f161, 73.0–73.5 | `Log(zw) = Log z + Log w + 2πi k` x 558–888 y 218 arrives whole. The purple last term leads "plus a whole number of laps" (79.1–79.8) by about 6 s. No motion 74.0–83.5. | `product_rule.rs:290-299` | Bring the purple term on its words. |
| **R7** | 08_5 | f271–f276, 122.5–125.0 | The dot sweeps 0 to 180° under "Each minus one has principal angle pi". It ends 2.2 to 2.7 s before "turn by pi" (127.2–127.7) and about 3.8 s before "point at minus one" (128.7–128.9). The two green arcs (r ≈ 34 for 0–90°, r ≈ 46 for 90–180°) are unlabelled, so nothing ties one arc to one root. | `product_rule.rs:362-394` | Start the sweep on "so the two roots together" and mark each arc π/2. |
| **C2** | 05_3 | f215–f227, 94.5–98.5 | The input dot waits at the strip top on the labelled 3π tick while the output winds on: laps 2 at f215–f216, laps 3 from f217, output resting at 90° f218–f227. At a true 3π the output would be at 180° and 1.5 laps, so the still pair can read as "3 laps at 3π". The source comment makes the wait deliberate, and an arrow says the line goes on. | `complex_turns.rs:65`, `:471-473`, `:521` | Move the dot along the arrow, or fade it, once it leaves the labelled ticks. |
| **B8** | 07_5 | f262 on, 114.5 on | Zoom `07-f0264-spiral.png` (×6): the two innermost turns merge into a solid red knot about 10 px wide at 1280, and the red zero ring is lost in the same colour. The voice says "without crossing itself" (119.3–121.2). Turn radii are about 0.03, 0.09, 0.28, 0.88 and 2.7 at stroke width 4.0. | `branch_cut.rs:227-241`, `:496`, `:70` | Start the spiral at the ring, or thin the inner turns; recolour the ring. The test near `:696` requires start radius under 0.05 and would need updating. |
| **B11** | 07_6 | f362 to end, 158.5–169.6 | Grey footnote of about 110 characters, x 62–898 y 543, 27 px above the frame bottom at 960, for about 11.5 s, beside three formulas and the voice. Readable at 1280 (`07-f0365-foot.png`). The comment at `:595-596` makes it deliberate. | `branch_cut.rs:597-602` | Shorten it, or move the DLMF detail to the description. |
| **B3** | 07_3–07_4 | f218, 95.5 | 15 separate text items, including 07_2's cards held from earlier. All legible, none touching. | `branch_cut.rs:367-368`, `:470-473` | Dim or remove the 07_2 cards when the strip arrives. |
| **R4** | 08_3 | f188, 87.0 | About 16 text items. None touching. | `product_rule.rs:273`, `:318-323` | Clear the `compare` line or the two rule rows before `keeps`. |

### Low

| ID | Cue | Frames, time | What the pictures show | Source | Fix direction |
|---|---|---|---|---|---|
| C3 | 05_3 | f185 on, 80.5 on | Eight rays at π/4 steps but strip horizontals only at π/2 steps; one teal π/4 line accounts for one diagonal. | `complex_turns.rs:381-391`, `:412-414`, `:449-460` | Faint strip lines at π/4 steps. |
| C4 | 05_6 | f332, 143.0 | `2π apart` caption about 7 s before "two pi apart" (150.0–150.2). | `complex_turns.rs:661-664` | Show it on the phrase. |
| C5 | 05_6 | f348–f362, 151.0–155.1 | The formula sits on the output axis's row. No contact (`05-f0349-each-card.png`), but "whole" ends about 9 px at 1280 before the axis's left end, so the axis reads like a dash after the text. | `complex_turns.rs:85-88` | Move the card off that row, for example y −0.6. |
| C7 | 05_7 | f374–f376, 159.0–160.0 | `angles a lap apart` and dots about 3 to 4 s before "whole column of angles" (163.3–163.8). | `complex_turns.rs:710-713` | Move after sentence 1's start. |
| C8 | 05_7 | f405–f406, 173.5–174.0 | `a dial that forgets laps` x 672–847 y 123 about 3 s before "a dial" (176.8–177.0). The folded slide rule (179.5–181.3) is spoken with no picture of it. | `complex_turns.rs:733-735` | Show it on the phrase; consider a small slide-rule callback. |
| C9 | 05_2, 05_3 | f67–f168, 29.0–72.7; f172–f173, 74.0–74.5 | No label at the −1 dot through 05_2 and 05_20. The input strip has no 0 label or x values. Title-only frames at 74.0–74.5 under "Watch what that does to a grid". | `complex_turns.rs:255-266`, `:380` | Label the −1 dot. |
| L2 | 06_20 | f336, 145.0 | `fixed` about 4.4 s before "stays fixed" (149.4–149.5), though its comment says it lands on the sentence's last words. Static 145.0–151.8. | `lifting_loops.rs:646-649` | Wait for the phrase. |
| L3 | 06_7 | f375 on | Ring radius 0.2 around ours takes in the thin L trace, offset only 0.16, though labelled "ours" (`06-f0375-ours-ring.png`). | `lifting_loops.rs:619`, `:666` | Smaller ring or larger offset. |
| L4 | 06_7 | f362 on | The thin L trace stays at 2π height beside the L(1) dot at its foot. That is the contradiction drawn correctly, but no caption names it until "k would have to jump". | `lifting_loops.rs:611-626`, `:656` | Note only. |
| L5 | 06_1–06_2 | 9.0–15.0, 19.5 | No motion 9.0–15.0. The walk waits for sentence 2 (`:331-332`), so it starts about 15.0, after "walk the number slowly around the circle" (11.2–14.4) has ended. `length stays 1: real part stays 0` at 19.5 is about 4 s before "only the angle changes" (23.2–24.0). | `lifting_loops.rs:331-346` | Start the walk on the phrase. |
| B2 | 07_4 | f196 on, 84.5 on | `just below → bottom edge` x 361–521 y 462 ends 19 px before `rays at angle θ →…` x 540 y 465, 3 px off the same row, so they read as a run-on line, though colour separates them (`07-f0196-runon.png`). The test at `:735-751` guards a 0.15 gap only. | `branch_cut.rs:168-173`, `:406` | Move one row, or remove the rays card first. |
| B4 | 07_4 | f218, 95.5 | `crossing the cut: a jump of 2π` about 3.3 s before "jump by two pi" (98.7–98.9). The dots fit "So a path that crosses" (94.8–95.2). | `branch_cut.rs:435-449` | Caption on the phrase. |
| B5 | 07_5 | f244, 105.5 | `cut upward: −3π/2 < angle < π/2` about 4.5 s before "from minus three halves of pi" (109.8–110.4). No arc draws the window. Static 105.5–113.5. | `branch_cut.rs:483-494` | Caption on the phrase; add a window arc. |
| B6 | 07_5 | f242, 104.5 | `a choice, not a crack` arrives just after its sentence (100.9–104.0), because the fade, morph and swap come first. | `branch_cut.rs:453-482` | Show the words first. |
| B7 | 07_5 | f265–f285, 116.0–126.0 | About 10 s static under "no loop can wind around zero" (123.9–126.2). No attempted loop drawn. | `branch_cut.rs:498-507` | Consider one blocked loop. |
| B9 | 07_6 | f330–f336, 142.5–145.5 | The dot crosses at about 144.2, 1.8 to 3.4 s after "But it jumps" (140.8–142.4). Static 135.0–142.0 before it. | `branch_cut.rs:528-554` | Start the slide earlier. |
| B10 | 07_6 | f360–f361, 157.5–158.0 | `Log(e^w) = w ⟺ −π < Im w ≤ π` x 600–886 y 397 leads "above minus pi and at most pi" (162.2–162.8) by about 4.5 s. Red `Log(e^{2πi}) = 0 ≠ 2πi` x 669–818 y 448 is true but never spoken. | `branch_cut.rs:576-594` | Condition on its words; drop the unspoken line, or give it words. |
| R5 | 08_3 | f162 to wipe, 74.0–90.8 | `precisely: for nonzero z, w on the principal branch, k is −1, 0 or 1` x 266–695 y 543, 27 px from the bottom at 960. Correct. Letters are upright here and italic in the formula. | `product_rule.rs:300-301` | Note only, or typeset the letters. |
| R6 | 08_2 | f117–f118, 54.0–54.5 | `principal window` leaves while `outside` stays, so "outside" no longer says of what. The gold bar remains. | `product_rule.rs:268` | Keep the word, if it stays clear of the arrow. |
| R8 | 08_4 | f213 on, 96.5 on | Row 1's `=` centre is at x 746 at 1280 and rows 2–3 at x 774, 28 px or 0.32 scene units apart (`08-f0240-equals.png`), against the source comment that they line up. | `product_rule.rs:334-336` | `right + 0.46` in place of `right + 0.78`; confirm in a render. |
| R9 | 08_5 | f257 on, 116.5 on | Red box x 514–825 y 265–308 around row 3 at x 572–695: margins 58 px left and 130 px right. | `product_rule.rs:350` | Centre near scene x 2.29, width about 2.4 to 2.6. |
| R11 | 08_5 | f298 on, 136.0 on | The green row writes `e^{iπ}` while the purple row writes `e^{πi}` (`08-f0300-stack.png`, `08-f0300-halved.png`). | `product_rule.rs:398`, `:432` | One order. |
| R12 | 08_1 | f44 on, 20.5 on | The gold bar runs to the centre of the open ring at −π and the grey tick stands in it, so the "open" end is not empty, though it still differs from the solid dot at +π (`08-f0044-window-ends.png`). | `product_rule.rs:186`, `:198`, `:454-456` | Stop the bar at the ring's edge. |
| R13 | 08_5 | f277 on, 125.5 on | Evidence rows are centred near x 723 and chain rows left-aligned at x 572. The green row is about row 2 but sits under row 4. | `product_rule.rs:336`, `:395-427` | Align the rows or tie the green row to row 2. |
| R14 | 08_1, 08_5 | various | `z: ¾π` at 9.0 leads "three quarters of pi" (12.3) by about 3.3 s. The gold dot covers half the green arrowhead at f31 14.0 (`08-f0031-tip.png`). `e^{πi} = −1` at 135.5 leads "a factor of minus one" (139.6–140.0) by about 4 s. `±` at 142.0 leads "only up to that sign" (145.9–146.1) by about 4 s, within its own sentence. | `product_rule.rs:127-136`, `:149`, `:430-448` | Note only. |
| N1 | all four | headers | Module comments still call the code uncompiled and unrendered, though these frames were rendered from it. | `complex_turns.rs:15`, `lifting_loops.rs:15`, `branch_cut.rs:18`, `product_rule.rs:9-10` | Update when next edited. |

**Long holds** not already listed, for root's pacing judgement. These stills show no change across them; narration continues over them.

- **Chapter 05:** 29.5–38.0, 160–166.5, 174–181.9.
- **Chapter 06:** 16.0–20.0 and 22.0–25.5 (the walk stops on no landmark at 0.20π and 0.40π), 171.5–178.4 ("Something has to give" at 176.1–177.8 changes nothing).
- **Chapter 07:** 31.5–41.
- **Chapter 08:** 9.5–13.5, 17–20, 21.5–26, 27–30, 40–43, 54.5–58.2, 92–116 for the ring and cut, which go unused until 122.5, and 136.0–141.0 for the whole frame.

**Smaller notes:**

- Chapter 05: the timer bar at 122–125.5 crosses the vertical axis tail, touching no label.
- Chapter 06: the rewind 1.94π→0 takes about 1 s under "Now suppose" (`lifting_loops.rs:601-602`, deliberate). The teal circle runs past the z-axis end.
- Chapter 07: `no crossing` at 5.0 leads "forbid crossing" (7.9–8.5) by about 3 s. `d/dz Log z = 1/z` at 31.5 leads its words (33.7–35.6) by 2 to 4 s. The `i` is upright in the Log i labels (`branch_cut.rs:349`, `:356`) and italic in `Log z = ln|z| + i Arg z` (`:320`).
- All four chapters end on the full picture, with no wipe.
- `07-f0150-theta.png` and `07-f0365-logew.png` show an open-form ϑ and some heavier symbols. Glyph questions belong to `logarithm-stage06-glyph-review01`, and I did not redo them.

**Withdrawn on a closer look:**

- 06 walk ending at f169 70.5: it fits "Walk the number once around it" (68.4–71.3).
- 06 `contradiction` at f391 169.5: it fits its words (169.3–170.4).
- 07 fraction legibility at f244: readable at 1280.
- 08 faint marks in f203 under ×6 brightness: f204 under the same boost shows only the title, axes, ring and cut. That is consistent with the last of the fade, not a kept mobject. Neither is visible at normal brightness.
- 08 sheet 081 looking more saturated: the empty corner measures (12, 17, 20) in frames 300, 318 and 320 to 325, so it is not in the pictures.
- 05 "−1" near ring r = e^0.5 at f284: about 3.5 px clear at 1280 (`05-f0284-minus-one-ring.png`). Worth one look at 4K, no action.

## What looked right

**Checked against the drawn geometry:**

- **05:** 1.2×1.5 = 1.8, 30+50 = 80, i×i = −1, e^{ln 2 + iπ/2} = 2i, ring ratios about e^0.5, the 05_7 arm at 108° with radius twice the unit ring, column dots at 2.6π, 0.6π and −1.4π, the ln 2 = 0.693 line at the right x.
- **06:** readout, arm and column height agree at 0.20, 0.40, 0.50, 1.00, 1.86, 2.00 and 4.00. The log map, the loop extents ln 0.89 to ln 2.49 and arg ±0.49, and the bracket of one lap 0 to 2π all agree. The blue trace follows the dot at 0.90π, 0.95π, 1.09π, 1.94π, 1.79π, 0.78π, 0.92π and 1.67π. The dial reads 0 until f268 and −1 from f269. `L(z) − ℓ = 2πik` and "k would have to jump" 0 to −1 are right.
- **07:** strip 339 px for 2π, seven green horizontals 42.5 px apart, verticals at ln r × 54 px matching circle radii. The gold wedge goes to the top edge and the purple to the bottom. The readout agrees with the dot within 1 px at six points, with the sign flip right. Window −3π/2..π/2 is right for the upward cut. Log i, Log(−i), Arg(−1) = π, e^{Log z} = z, Log(e^{2πi}) = 0 and the solid top and dashed bottom edges against −π < Im w ≤ π all hold.
- **08:** number line 104.5 px per π, with green, red and gold dots at 3/4π, 3/2π and −π/2. Open at −π and closed at π matches the convention. Arrow length 210 px = 2π, pointing one lap down. −½πi = 3/2πi + 2πik gives k = −1. Also right: the chain as a false proof, e^{iπ/2}e^{iπ/2} = e^{iπ} = −1, √1 = e^{½ Log 1} = e^0 = 1, Log 1 = 0, ½(2πi) = πi, and k ∈ {−1, 0, 1} for principal Log.

**Beats that land on their words:**

- **06_7:** L(1) dot 155.0, ring 161.5, arrow 164.0–164.5, theorem 171.0–171.4, each within about half a second of its sentence's start.
- **07_3:** morph 56.0–59.0 on "unrolls", strip labels 59.5, circles 62.5, rollback, wedges 69.2.
- **07_5:** spiral 114.0–114.5, `any curve` 116.0, gold ring 126.5, `branch point` 129.5.
- **08_1, 08_2:** cut 7.0–7.5, the sweep 14.0–16.0 on "Square it", `Log(z²) = ?` 26.5–27.0, `outside` 35.3–35.5, `same direction` 39.5, purple lap arrow 54.5.
- **08_4:** chain rows each on their sentence (94.5, 96.5, 99.5–100.0, 105.0), `so −1 = 1 ?` 108.0.
- **08_5:** red box 116.267, `√z` 118.0–118.5, gold and purple rows 130.5, `½(2πi)` 135.0–136.0.
- **Wipes:** clean in all four chapters on the sampled frames.

## Think gaps

Each gap runs from the question cue's end to the answer cue's start. All seven keep the answer off screen across the sampled frames.

| Chapter | Gap | Frames | Result |
|---|---|---|---|
| 05 | 22.538–27.433 | f50–f61 | Hidden |
| 05 | 121.113–126.000 | f286–f297 | Hidden |
| 06 | 38.546–43.433 | f88–f102 (38.446–43.500) | Hidden; readout stays 1.86; first move f103 43.633 |
| 06 | 77.156–82.033 | f184–f196 | Hidden; loop stopped at about 125°; first move f197 82.233 |
| 07 | 75.741–80.633 | f174–f186 | Hidden; bar f177–f185; first change f187 80.833; grid opens f189 81.133 |
| 08 | 30.231–35.100 | f64–f77 | Hidden; first reveal f78 35.3. The red dot has sat outside the gold bar since 21.0, which hints the first half of the answer only; −π/2 stays hidden. |
| 08 | 111.385–116.267 | f244–f255 | Hidden; first reveal is the red box at f256 116.267, exactly at the cue |

## Lint anchors

Stage06's `.layout.tsv` files are empty for all four chapters. The anchors in `sampling.json` come from older previews, so a label may no longer describe what stage06 shows at that time. I looked at each anchor and at the same text wherever it does appear. None is a visible defect.

| Anchor | Finding as listed | What the pixels show |
|---|---|---|
| 05 @60.6 | `zw` and `2i` overlap 35% | Gap: 2i x ≈ 210, zw x ≈ 279, f136–f159 |
| 05 @92.0 | stroke under `laps:` (4.63, 1.78) | laps: starts x 820, ray dot ends 797, outer ring x ≈ 754 |
| 05 @92.0 | stroke under `each lap drawn…` (2.70, −2.81) | Caption y 530–542, axis ends y 513 |
| 05 @99.6 | stroke over `0` (−4.05, −0.15) | Glyph bottom y ≈ 302, connector y 310 |
| 05 @154.8 | stroke under `length 2, angle 0.6π` (1.58, 1.87) | Caption not up at 154.8. When up, ends x 651 against axis x 662, bottom y 144 against ring top y 153 |
| 05 @166.0 | stroke under `0: no logarithm` (4.01, −0.65) | Not up at 166.0. When up, top y 468 under leader end y 466, ring y ≈ 431 there, ray end (797, 445) |
| 06 @0.8 | shape spans x −7.00..−1.20 | Source half_w 2.8 puts the axis end at −6.90 (`lifting_loops.rs:35-41`), inside the ±6.96 safe area. Pixels agree at x 13–14 of 960. Nothing clipped |
| 06 @48.4 | stroke over `2πi` (0.45, −0.34) | Tick label ends x 511, red ring starts 527, gold label starts 560, bracket tick ends 488 |
| 06 @170.4 | `3πi` and `no continuous logarithm on a loo…` overlap 79% | No such caption at f394 170.4. When up at f397–f398 it is at y 538, x 346–776, with the 3πi tick at (503, 240) |
| 06 @0.8 | the same report again, for a second shape | Same reading as the row above |
| 07 @12.8 | stroke under `blocked` (−4.10, 0.41) | blocked ends x 185, ring at that height x ≈ 204, blocked bottom y ≈ 276, no crossing top y ≈ 289 |
| 07 @69.0 | stroke under `just below` (−5.66, −0.72) | No label yet at f158 69.0. When up, top y ≈ 344, purple wedge end y 333, outer ring x ≈ 98 there, label ends x 82 |
| 07 @129.8 | stroke under `branch point` (−2.03, −0.80) | About pixel (343, 354). At f294 the label is at x 541–638 y 505, nothing near it |
| 07 @134.0 | stroke over `branch point` (−2.50, −0.73) | Label gone by f309 133.167; f312 134.0 has none |

Chapter 08 has no anchors.

## Corrections to the log

`viewing-log.tsv` was written as I read and is kept as written, with correction rows appended:

- **06, row 088-095:** said "fixed" stays beside "k would have to jump". f379 has it; f380 does not (`lifting_loops.rs:684`).
- **07, row 024-031:** word-share figures were loose. `checks/word_share.out` supersedes them.
- **08, row 072-079:** said about 2 s for `e^{πi} = −1`. It is about 4 s.
- **05, row 080-087:** says "zoom frame 350". The zoom made was f349.
- **All phrase times in the log** are reading-time estimates. Where they differ from `checks/word_share.out`, the table wins and this review uses it.

## Limits

- **Sampled stills only.** Half-second samples, plus cue boundaries and lint anchors. `sampling.json` says: "Half-second samples, cue boundaries and exact lint anchors plus nearby looks; not continuous-motion or listening proof." Anything shorter than the sampling can be missed, and motion quality between samples was not seen.
- **No audio was reviewed.** Nothing here is evidence about narration, pronunciation, cadence or sync as heard. Phrase times are estimates from cue lengths.
- **Scratch preview, not final.** Panels are 960×540 from 1280×720 frames. I inspected 18 spots at frame resolution. A 1 px defect elsewhere could pass, and contact gaps need a look at 4K. These previews are not the ElevenLabs Oliver final and not final 4K.
- **Not read for this review:** the citation behind `DLMF 4.2` was not opened. Chapters 00 to 04 are out of scope.
- **Mathematics checks** compare the pictures with the formulas and the geometry they draw. They do not replace the oracle or the film's tests, and I ran neither.
- **Not independent.** I share a model and auto-memory with earlier reviewers and repairers of these chapters (BronzeJaguar on 05–07, WhiteSwan on 08–10), and their summaries are in the memory I started from. Treat this as one more careful read, not independent final acceptance.
- **Nothing was changed outside this folder.** No source edits, synthesis, builds, renders, Beads changes or deletions. The sealed geometry review is untouched. Findings are for root to route.

## Files

| Path | What it holds |
|---|---|
| `REVIEW.md` | This report |
| `viewing-log.tsv` | Chapter, sheets read, what the pixels showed, with correction rows |
| `manifest/sheets.tsv` | All 392 sheets: SHA-256, bytes, frames, times |
| `manifest/cues.tsv` | Cue times and text |
| `manifest/zooms.tsv` | Each zoom's frame, frame hash, crop, treatment and hash |
| `zooms/` | The 18 zooms cited |
| `checks/build_manifest.py` | Rebuilds the manifest; compares source hashes |
| `checks/word_share.py`, `.out` | Phrase-time estimates, by words and by letters |
| `checks/frame_times.py`, `.out` | Cited picture numbers against `sampling.json`; the lint anchors as listed |
| `SHA256SUMS` | Seal over all of the above |
