# Logarithm preview02, chapters 08–10: independent picture review

WhiteSwan, 2026-11-04, answering Mail 1419 under root-owned Bead `mx-video-program-program-87do.63`, which stays open.

**Result.** I viewed all 779 selected frames (196 sheets) and found 17 concrete defects: 4 high, 7 medium and 6 low, plus 4 nits.

- **Accuracy:** two findings put something wrong or misleading on screen (F4, F8). The rest of the mathematics holds.
- **Worst readability:** the two decimal matrix cards in chapter 09 cannot be read (F1).
- **Worst layout:** one label runs off the frame (F2) and two labels collide for 31 s (F3).
- **Worst continuity:** a clipping bug removes or breaks the grid border at the moment the film says everything is home (F5).
- **Narration:** no repair needs a new take, except one optional route for F10.

Every repair below is a proposal. Nothing was compiled, rendered or synthesized.

This reviews pictures from a scratch-voice render. It is not listening, Oliver acceptance, fine sync, continuous-motion review or independent full AV acceptance. All times are scratch timings and will move under Oliver. Beats bound to `until_sentence` keep their order.

## 1. Bindings

Both receipts report exit 0 with stable input hashes. Every hash in both input lists matched at about 09:53Z, and no crate source was newer than the render start. The three MP4s and five source files below were re-hashed at 10:23Z and still match.

| Item | Value |
|---|---|
| Render | `narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-opus-rebuild-preview02`: `render 00…12 --res 1280x720 --fps 30 --scratch-voice --lint --crf 26 --preset ultrafast`, 08:28:02–09:12:33Z |
| Render inputs | `600950d6641bd537bfb13976ee545812c7cc275a80fc47689b42f2d4e3e085cc` (174 files) |
| Frames | `…/logarithm-opus-rebuild-frames02`, `frames02.py`, 09:12:56–09:19:11Z |
| Frame inputs | `c879e017cfff6cb68787bd5bec9cbb28e1be9d7528709645c0469c5bcc3e3dd4` (216 files) |
| MP4 sha256 prefixes | 08 `a60d6bdd65c70d8b`, 09 `39fef25bed9a48f3`, 10 `02daf1a4a52d26ca` |
| Source sha256 prefixes | `product_rule.rs` `a8b4e31d98b23bd7`, `matrix_logs.rs` `4250a5f0294f58a3`, `doing_nothing.rs` `bf0008113e804186`, `draw.rs` `59ebd09d2fa5c793`, `script.tsv` `3e4e34af7090e448` |

## 2. Review denominator

| Chapter | Sheets viewed | Frames viewed | Rendered frames | Duration | Lint findings |
|---|---|---|---|---|---|
| `08_product_rule` | 82 (000–081) | 326 (0–325) | 4453 | 148.43 s | 0 |
| `09_matrix_logs` | 58 (000–057) | 232 (0–231) | 2841 | 94.70 s | 8 |
| `10_doing_nothing` | 56 (000–055; 055 holds only frame 220) | 221 (0–220) | 2967 | 98.90 s | 0 |
| **Total** | **196** | **779** | **10261** | 342.03 s | 8 |

The selected frames are about 7.6% of the rendered frames: half-second samples plus cue boundaries and lint anchors. `PROGRESS.md` records the sheets in order. I confirmed each finding on a native 1280×720 frame or a native crop, except F17, which rests on source and samples.

**Lint.** All 8 findings in chapter 09 are accounted for:

| Lint finding | Where it goes |
|---|---|
| 12.4 s crossed | F1 |
| 44.5 s crossed ×4 | F1 |
| 15.0 s outside | F14 |
| 42.2 s outside | F14 |
| hold 86.9 s outside | F2 |

An empty lint report does not mean a clean chapter. Lint flags none of F3, F6 or F7, although each one puts ink on ink.

## 3. Findings

| ID | Ch | Severity | Frames | Scratch time | Defect |
|---|---|---|---|---|---|
| F1 | 09 | High | 28–34, 109–169 | 12.3–13.5, 44.2–67.6 | Decimal matrix cards run together and their brackets cross the digits |
| F2 | 09 | High | 207–231 | 86.5–94.7 | Dictionary label is cut off at the right frame edge |
| F3 | 10 | High | 153–220 | 67.3–98.9 | "keep −π < turning < π" prints onto "ellipse flows" |
| F4 | 10 | High, accuracy | 94–108 | 39.5–46.5 | A number's window is drawn open at +π |
| F5 | 10 | Medium | 25–35, 48–58, 66–87, 120–134 | 11.0–13.7, 19.5–24.5, 28.5–36.5, 52.5–59.5 | Grid border missing or broken when the flow is home |
| F6 | 08 | Medium | 36–~200 | 16.5–91.0 | Vertical axis runs through the `z²` label |
| F7 | 08 | Medium | 262–325 | 118.0–148.4 | Vertical axis runs through the √z definition |
| F8 | 10 | Medium, accuracy | 114–220 | 49.5–98.9 | "turns 0.872" has no unit and reads as laps |
| F9 | 09 | Medium, reveal | 67–79 | 25.5–31.5 | "Here is the picture that matters" over an empty left half |
| F10 | 10 | Medium, reveal | 116–121 | 50.5–53.0 | Screen says "does not turn" while the voice describes turning |
| F11 | 09, 10 | Medium, clarity | 09: 203–231; 10: 18–87 | see below | Symbol J is never defined |
| F12 | 08 | Low | 199–201 | 90.6–91.0 | A removed label flashes back during a fade |
| F13 | 08 | Low | 277–325 | 125.5–148.4 | Green product line crowds step 4 |
| F14 | 09 | Low | 39–42, 101–169 | 15.0–16.0, 42.1–67.6 | Grid passes the left frame edge |
| F15 | 10 | Low | 200–220 | 90.0–98.9 | Literal carets in `e^(tX)` and `e^X` |
| F16 | 10 | Low | 200–220 | 90.0–98.9 | Reading load at the end of the chapter |
| F17 | 09, 10 | Low | see below | see below | Instant grid jumps and pops |

### F1. Chapter 09 matrix cards are illegible (high)

- **On screen:** the rotation card (frames 28–34, native frame 30) reads "0.5−0.866", and its brackets cut through a 0.866 on each side. `e_card` (frames 109–169, about 23 s, native frame 120) reads "0.905−0.97" over "0.7760.517", with brackets through the leading 0s and final digits. The right bracket of `x_card` touches the 1 of −0.1 (frames 80–169).
- **Source:** `draw.rs:494` sets `dx = size*0.011` and `draw.rs:503` sets `w = dx + size*0.0095`, whatever the entries are. The comment at `draw.rs:492` uses the short em. `docs/storyboard.md:201–214` already lists this geometry as never seen.
- **Evidence:** checks section B gives negative clearances: rotation brackets −0.064 and −0.197; `e_card` row gaps −0.065 and −0.023, brackets −0.057 and −0.098; `x_card` right bracket −0.023. Its rotation bracket position, 3.70, matches the lint crossing at 3.70.
- **Not the numbers:** the card values are right. Section C gives e^X = [[0.90522, −0.97041], [0.77633, 0.51706]]. −0.97 is −0.970 with the trailing zero dropped, which makes the card's decimals uneven.
- **Repair:** measure width from the entries in `matrix_card`: `dx = max(size*0.011, (widest + 0.25)/2)` and `w = dx + max(size*0.0095, widest/2 + 0.10)`. Section H shows every chapter 09 card clear on these estimates, with `e_card` at 3.61..5.59 (`e_word` ends near scene 3.11) and `pair` at 4.34..5.46 (the ↔ starts near 5.63). Rounding to two places does not fix the cards on its own (section H), though it could be added as a choice.

### F2. Dictionary label is cut off (high)

- **On screen:** "turn-and-scale matrix ↔ complex numb", frames 207–231. Native frame 220 has lit columns up to 1279, the last column of the frame (section I).
- **Source and lint:** `matrix_logs.rs:354–360`, centred at x = 5.4. Lint hold @86.9 gives x up to 7.26.
- **Repair:** centre at x = 5.0, size 20. The span becomes about 3.14..6.86, inside the 6.96 safe edge and 0.57 clear of panel 3 at rest (right edge near 2.57).

### F3. Labels collide for 31.6 s (high)

- **On screen:** the π of "keep −π < turning < π" prints onto the first letter of "ellipse flows: ±2π", frames 153–220, native frames 170 and 204. The text reads "…< πllipse flows".
- **Evidence:** in frame 170 the green rule ends at column 1099 and the purple label starts at column 1094, so at least 6 columns are shared (section I).
- **Source:** `doing_nothing.rs:379` and `:368–374`, both at `TURN_Y − 0.7`. Spans are about 3.10..5.14 and 5.03..6.56.
- **Repair:** centre the rule at x = 3.7 (span 2.68..4.72, gap 0.31). Or fade `lap_word` when the red crosses land, since the crosses already reject ±2π.

### F4. A number's window is open at +π (high, accuracy)

- **On screen:** native frame 97 shows red open rings at both −π and +π under "a number's principal angle: within a half turn" (`doing_nothing.rs:292–301`).
- **Why it is wrong:** the principal angle of a negative number is +π. Chapter 08 draws this correctly, open at −π and closed at +π (`product_rule.rs:191–192`, native frame 130). 08_5 says "Each minus one has principal angle pi", the 07 precisely line gives (−π, π], and chapter 12 uses Log(−1) = iπ. Only flows are "strictly" inside, and the narration does not say so until 10_4.
- **Repair:** at 10_3 sentence 1, draw a closed dot at +π and an open ring at −π for numbers. On the 10_4 "strictly", swap the closed dot for an open ring. That turns the change into a visible point about matrices.

### F5. Grid border lost when everything is home (medium)

- **On screen:**
  - Frames 25–35 and 48–58: no outer box.
  - Frames 66–87: left outer line missing above y = 1.83, right outer line missing below y = −1.83, short gaps at top-left and bottom-right.
  - Frames 120–134: the columns x = ±2 of the stretch flow vanish as they land on ±3, although frame 119 still has them.
  - The start grid (frame 8) has the full box, and the box snaps back at frames 36 and 59. Native crops of frames 8, 28, 80 and 125 show this.
- **Why it matters:** it happens while 10_2 says the flow "brings the whole plane home exactly", so home looks different from the start.
- **Source and cause:** `draw.rs:569`, `if q[0].abs() <= clip && q[1].abs() <= clip`, called with clip 3.0 at `doing_nothing.rs:126` and `:313`. The lines k = ±3 sit exactly on the clip, and e^X = I + about 1e−15 pushes their points just past it. The stretch flow lands x = 2 at 3.0000000000000018.
- **Evidence:** section A's Python mirror keeps 0/73 samples on all four lines for 2πJ and reproduces every gap above. It does not run the crate.
- **Repair:** one line, `<= clip + 1e-9`. Section A shows 73/73 samples kept on every line. `clipped_grid` also serves chapter 11 (section 8).

### F6. Axis through `z²` (medium)

Frames 36 to about 200, native frames 130 and 183. `product_rule.rs:153` places the label at `Z.p(zz.scale(1.3))`, and for this angle that point is on the vertical axis. **Repair:** shift the label about 0.35 to the right so it sits beside the arrow tip.

### F7. Axis through the √z definition (medium)

Frames 262–325, native frame 300: the axis passes through the `e` of √z = e^{½ Log z}. `product_rule.rs:343–350` centres it on the axis. **Repair:** centre it about 0.9 to the right of the axis. That clears the axis and the right column.

### F8. "turns 0.872" has no unit (medium, accuracy)

- **On screen:** "spiral flow: turns 0.872 per unit time, about 0.28π", frames 114–220 (`doing_nothing.rs:329–339`).
- **Why it misleads:** 0.872 is radians, which is 0.2775π or 0.1387 laps (section C). Chapter 09's labels "0.25 turn per unit time" and "1.25 turns per unit time" (`matrix_logs.rs:316–317`) count laps, so this one reads as 0.872 laps. The dot on the line is placed correctly at 0.28π.
- **Repair:** "turns 0.872 radians per unit time, about 0.28π". Right edge goes from scene 5.84 to about 6.62, inside the safe edge (section I). Or drop 0.872 and keep 0.28π. `model.rs:445` and `facts.rs:237` carry the same wording in comments only.

### F9. "The picture that matters" over nothing (medium, reveal)

Native frame 75 shows the title and the series alone, with the left half empty, across frames 67–79. Sentence 1 of 09_2 is "Here is the picture that matters." The field arrives at sentence 2 (`matrix_logs.rs:225`, frame 80). This survives re-voicing because it is bound to sentences. **Repair:** add the resting grid and F at sentence 1 by creating `flowing` with time 0 there. Keep the arrows and the X card at sentence 2 and the motion at sentence 3.

### F10. Stretch flow arrives during the description of turning (medium, reveal)

10_3 sentence 3 is "Every steady flow of the plane either turns round at a steady rate, sweeping round stretched circles, perhaps spiralling in or out, or does not turn round at all."

- **On screen:** the picture switches to the straight-line stretch flow at frame 116, just after the sentence starts (about 49.5 s; native frame 118). "stretch flow: does not turn round at all" appears at about 52.7 s. By word proportion that is about 4–5 s before those words. This is an estimate from a scratch take, with no claim about Oliver.
- **Source:** `doing_nothing.rs:327–357`.
- **Repair:** replay or hold the spiral through the first clauses and run the stretch flow at the end. The kit has only sentence anchors (`kit.rs:290`, `:325`), so either use a wait tuned once Oliver is bound (no new take) or split the sentence in the script (new take, root's call).

### F11. J is never defined (medium, clarity)

"X = (π/2 + 2πk) J" appears in chapter 09 frames 203–231 (`matrix_logs.rs:346`). "X = 2πJ" and "2πPJP⁻¹" appear in chapter 10 frames 18–87 (`doing_nothing.rs:153`, `:253`). The narration never says J. The only definition is in `matrix_log.rs:70`, which is not registered (`mod.rs:14` registers `matrix_logs`). **Repair:** change the label at `matrix_logs.rs:326` to "J: quarter turn". J is already in the glyph list (`mod.rs:29`).

### F12. Removed label flashes back (low)

"principal window" fades out at 54.0 s (`product_rule.rs:258`). It is absent at frame 198 (90.5 s), back at full strength at frames 199–200 and faint at frame 201 (91.0 s). The frame difference is exactly that label's box, (527, 525, 663, 542) (section F). It stays in `line` (`:194`), which `away = line.clone()` re-roots for the fade (`:308`). **Repair:** take it out of `line` once it has faded.

### F13. Product line crowds step 4 (low)

The green line sits 8–9 native px under "= √1 = 1" in frames 277–325 with no ink contact, so it reads as attached to that step (native frame 300, `product_rule.rs:385–395`). **Repair:** move it to y of about −1.15 to −1.2.

### F14. Grid past the left edge (low)

- **Shear:** frames 39–42, lint x −7.88, with lines visibly leaving the frame (native frame 40).
- **Flow:** frames 101–169, lint x −7.20, a small overrun (native frame 120).
- **Source:** `matrix_logs.rs:35–36`.
- **Repair:** use `clipped_grid` with clip about 4.4 in chapter 09 (bound −6.63). Or move `CENTRE` right 0.2 and use n = 2 for the shear. Storyboard line 173 would then need 09 added to its list of clipped grids.

### F15. Literal carets (low)

"e^(tX)" and "e^X" appear in plain text in the textbook and precisely lines, frames 200–220, native frame 204 (`doing_nothing.rs:430`, `:438`). **Repair:** use `exp(tX)` and `exp X = A`, or typeset with tex.

### F16. Reading load at the end of chapter 10 (low)

The final state holds 12 text items, and the spiral, stretch and zero lines stay through 10_4. The dim lines hold 218 characters on screen from 90.0 to 98.9 s, or 24.5 characters per second on scratch timing (section G). **Repair:** fade those three lines at 10_4 sentence 1, shorten the textbook line, hold before the wipe, and measure again under Oliver.

### F17. Instant jumps and pops (low)

This comes from the source and the half-second samples. Motion between samples was not viewed.

- **Grid jumps by `set_tracker_value`:** frames 115→116 (`doing_nothing.rs:343–344`) and 134→135 (`:360–361`).
- **`add_to_scene` with no fade:**
  - Chapter 09 panels, frames 172→173, 183→184 and 193→194 (`matrix_logs.rs:323`, `:329`, `:332`).
  - Chapter 10 grid, frames 107→108 (`doing_nothing.rs:315`).
  - Chapter 10 orbit, frames 35→36 (`doing_nothing.rs:194`). The narration says "Squash the circle into an ellipse" there, but circles fade and ellipses appear with no squash.
- **Repair:** add a short fade around each reset, or tween back quickly, and fade the panels in with their labels.

## 4. Nits

- Matrix `a`, `b` are upright beside italic `a + bi` (chapter 09 frames 207 onward).
- "of I" has an upright I beside math-italic I (chapter 10 frames 78–87).
- The stretch ring (r 0.16) and spiral dot (r 0.1) overlap by about 2 native px at 0 on the turning line (chapter 10 frames 121 onward). A ring of r 0.13 clears it.
- `docs/storyboard.md:173` says drawing aids are "each stated on screen", but no chapter 10 frame states the grid clip. Add a caption or correct the document.

## 5. Pacing notes (scratch timing, not counted as defects)

- **Chapter 09:**
  - 09_3 has no motion from 44.7 to 67.3 s apart from reveals at 49.0, 56.5 and 62.5. An option is to replay the flow on sentence 0.
  - In 09_4 the panels are static from 69.5 s until the 4 s spin at 80.0 s, while the narration says each one "reaches" the target.
- **Chapter 10:**
  - The number line stands alone from 39.5 to 46.5 s.
  - The left half is a static identity grid from 60.0 s to the end, 39 s.
  - The 10_4 condition has no picture of a reversed direction, though chapter 11 treats it.
- **Chapter 08:** it has long sentence-bound holds. Judge them under Oliver.

## 6. Checked and found clean

- **08:**
  - Fades and reveals from 0 to 90 s.
  - Predict bars at about 31.0–35.0 and 112–116 s, with no answer shown before the cue (answer box at 116.267 s).
  - 08_4 steps at 94.5, 96.5, 99.5 and 105.0 s, and the absurd result at 108.0 s.
  - Roots sweep at 122–124.4 s, and the product at 125.5 s after the absurd line fades.
  - The k line (k = −1, −½π = 3/2π + 2πk).
  - `halved` clears the axis by about 5 native px. The precisely line gives k ∈ {−1, 0, 1}.
- **09:**
  - All four 09_1 examples are geometrically right, including the dashed chords.
  - Field directions were hand-checked at corners and stay fixed in space under the moving grid.
  - The counter eases 0 → 1.00 over 40–43.4 s. The tips match the e^X columns at t = 1.
  - The quarter-turn card is clean.
  - Rates 0.25, 1.25 and −0.75 equal π/2 + 2πk (section D).
  - Spin directions and final F orientation are right.
  - `def` stays clear of `all_three` and `column`.
- **10:**
  - Identity, X = 0 and 2πJ traces close at home.
  - The ellipse widths are 2:1, and the tilt is about 29–30° on frame 65.
  - Dots return to their start (frame 57 against frame 59).
  - Gold family through (r, 0), stretch 1 → 2.2 → 0.6, tilt 15°.
  - 2πPJP⁻¹ is valid; the stretch scales are 1.5 and 0.6; the spiral mark is at 0.28π.
  - The 10_4 statement holds at its edges (I, positive eigenvalues, rotations, −I, diag(−2, −½), singular) and matches Higham and Gallier as filed.
  - `cond_a` right edge is at about scene 6.71, inside the safe area.

## 7. Seen and deliberately not counted

- **Crossfades and swaps:** 08 `ask` → `same` at 39.5 s; 09 series → motion at 56.5 s, cards at 13.5 and 14.9–15.0 s, and title swaps; 10 `numbers_word` → `flows_word` at frame 108.
- **Red strikes** at ±2π from frame 155.
- **Clipped grids during motion** in chapter 10. The storyboard calls for them.
- **Blue traces off the grey ellipses.** Each point has its own ellipse.
- **Faint residue at chapter 10 frames 88–89**, at most 4/255 and gone by frame 90 (section E). I read it as preview encoding. Recheck at final quality.

## 8. Repair notes

- **Shared helpers:** `matrix_card` also draws the chapter 10 identity card and five chapter 11 cards (`unreachable.rs:274`, `:505`, `:639`, `:696`, `:801`). `:505` has computed entries, so its brackets may move. `clipped_grid` also serves chapter 11 with clip 3.2 (`unreachable.rs:131`, `:278`, `:497`). Chapter 11 needs a fresh look if F1 or F5 is adopted.
- **Tests:** a grep found no test or `include_str!` pinning the label strings in F2, F3, F4, F8 or F11. `script.tsv` is the only included text. Tests were not run.
- **Suggested order:**
  1. F1–F5, which are the highest-value changes and mostly small edits.
  2. F6, F7, F8, F11 and F13, which are label placement and wording.
  3. F9, F10 and the rest.
- **Narration:** only the script-split route for F10 changes any spoken text.

## 9. Residual gaps

- **Sampling:** half-second samples only, 779 of 10261 rendered frames. No continuous motion was viewed, so fades, pops and short transients between samples may be missed.
- **Audio:** no audio was heard. The render uses the scratch voice, and Oliver is not bound. There is no listening, fine-sync or independent full AV acceptance.
- **Repairs unproven:** nothing was compiled, rendered or synthesized. The checks are Python mirrors and width estimates from the CMR10 table.
- **Resolution:** frames are 1280×720 from a CRF 26 ultrafast encode. Fine detail and residue need rechecking at final quality.
- **Joins:** the joins to chapters 07 and 11 were not compared frame to frame. Chapter 05 was not reviewed: the operator limited this job to 08–10, and BronzeJaguar holds 05–07.
- **Scope:** no canonical edits, Beads, commits, extra agents or memory changes.

## Files

- `PROGRESS.md`: sheet-by-sheet record.
- `checks/review_checks.py`, `checks/review_checks.out.txt`: the checks and their output (exit 0).
