# Logarithm preview02, chapters 05–07: independent picture review

Reviewer: BronzeJaguar. Assignment: Agent Mail 1418 from BlueSalmon, under root-owned bead `mx-video-program-program-87do.63`. Date: 2026-11-04.

Scope: selected still frames of a scratch-voice preview. This is not an Oliver review, a listening review, a fine-sync check, a continuous-motion check, or full AV acceptance. All times are scratch and will move.

## 1. Result

**I found no wrong number or formula in the 1,216 selected frames.** Every value I measured or recomputed holds (section 7). The pictures are not clean yet, though. Three defects can teach something false, and four more make key text hard to read. All 13 lint findings are accounted for: 12 are confirmed and one is negligible (section 5).

**Fix these seven first:**

| ID | Chapter | Selected frames (scratch time) | Defect | Lint |
|---|---|---|---|---|
| F-zero | 07 | 137–153, 195–233 (59.0–67.0, 84.0–101.0 s) | The red "removed zero" ring stays on the strip at w = 0, which is the image of z = 1 | no |
| E7 | 06 | 272–292 (116.0–126.0 s) | The arrow beside "πi → −πi" covers 0.92π of the axis, not 2π | no |
| E1 | 06 | 114–147 (48.3–60.0 s) | "one lap" reads as "one lap πi", though a lap is 2πi | partly |
| E10 | 06 | 387–408 (170.3–177.1 s) | The gold theorem line is crossed by "3πi", the axis and a tick, until the chapter ends | yes |
| F-edge | 07 | 196–233 (84.5–101.0 s) | "the bottom edge" runs into "rays at angle", reading "edgerays" | no |
| D2 | 05 | 130–161 (60.5–71.7 s) | "2i" and "zw" overprint each other | yes |
| F-bp | 07 | 293–388 (129.7–170.0 s) | "branch point" is crossed by the spiral, then by the circle, and is never faded | yes |

No narration change is needed for any repair below. All repairs are proposals: nothing was compiled or rendered, and no canonical file was edited.

## 2. Bindings

| Item | Evidence |
|---|---|
| build05 | exit 0, inputs unchanged, finished 08:27:36Z, "Native Rust and Python oracle agree: 93 facts" |
| preview02 render | `resource-receipt.json` exit 0, 08:28:02Z–09:12:33Z, `inputs_sha256` equals final (`600950d6641bd537…`), `command.log` complete |
| frames02 | exit 0, 09:12:56Z–09:19:11Z, `inputs_sha256` equals final (`c879e017cfff6cb6…`) |
| Frames to video | 12 frames re-decoded (4 per chapter), 12/12 byte-identical to the frame PNGs |
| Video to inputs | sha256 of all three videos equals the frames02 input list |
| Source to inputs | sha256 of the six files below equals the preview02 input list and the working tree when this report was written |

| Video | sha256 prefix | Video frames | Length |
|---|---|---|---|
| `05_complex_turns.mp4` | `e74bbdddabf19a1a` | 5427 | 180.9 s |
| `06_lifting_loops.mp4` | `4d788419310747d7` | 5321 | 177.366 s |
| `07_branch_cut.mp4` | `02586ff8c8692154` | 5107 | 170.233 s |

| Source (under `explainer_corpus/logarithm_mysteries/`) | sha256 prefix |
|---|---|
| `src/chapters/complex_turns.rs` | `d1f2d72555ee7f10` |
| `src/chapters/lifting_loops.rs` | `b409ace64e13fafb` |
| `src/chapters/branch_cut.rs` | `c3fd774e9172cdbe` |
| `src/chapters/draw.rs` | `59ebd09d2fa5c793` |
| `src/facts.rs` | `7bf886f9762f1904` |
| `script.tsv` | `3e4e34af7090e448` |

Images: `narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-opus-rebuild-frames02/`. Render: `logarithm-opus-rebuild-preview02/render/` beside it.

## 3. Denominator

| Chapter | Sheets viewed | Selected frames | Indices |
|---|---|---|---|
| 05 | 105 of 105 | 418 of 418 | sheets 000–104, frames 0–417 |
| 06 | 103 of 103 | 409 of 409 | sheets 000–102, frames 0–408 |
| 07 | 98 of 98 | 389 of 389 | sheets 000–097, frames 0–388 |
| **Total** | **306 of 306** | **1216 of 1216** | |

`sheet-ledger.tsv` lists every sheet with its frames, times and sha256. The selected frames are 7.67% of the 15,855 video frames.

**How the viewing happened.** I viewed every sheet as an image and did not delegate any of them. Images leave my working context after about 50, so older sheets rest on notes written right after each batch (`NOTES.md`). To compensate, I reopened the native evidence for every high and medium finding in one final pass. Low findings that name no frame rest on those notes.

**Native inspection.** I inspected 30 selected frames at native resolution in the final pass:

- **05:** 46, 133, 199, 248, 300, 360, 386, 410
- **06:** 118, 160, 165, 276, 292, 293, 296, 310, 400
- **07:** 30, 126, 127, 137, 150, 170, 200, 219, 270, 295, 300, 315, 388

The box for 07 frame 150 missed its target line, so the θ finding rests on frame 200 instead. I also decoded two short runs of consecutive video frames, at reduced size, for F-pop and E11. Neither run is part of the 1,216.

**Frame ranges.** The ranges in sections 1 and 4 come from `checks/persist.out.txt`, which compares each defect's box against a reference frame. The two colour tests at the end of that file cover E5 and E6, where the box was too sparse. Where viewing and the scan differ, the row says so.

**Root's chapter 05 sheets 000–007.** I viewed them independently and agree: no concrete collision in frames 0–31 (0–14 s). Mail 1418 told me root's conclusion in advance, so this was not blind, but I read root's JSON only after finishing all 306 sheets. The first concrete collision in chapter 05 starts at frame 45 (21.0 s, D3), on sheet 011.

## 4. Findings

Frame numbers are selected-frame indices in `frame-NNNN.png`; `n` means a decoded video frame. Line numbers refer to the source hashes in section 2.

### High

**F-zero. The removed point sits at the wrong place on the strip (accuracy).** `branch_cut.rs:204` creates `zero`, a red ring that means "z = 0 is removed". It is not faded until `:426`, so it stays through both morphs. On the strip it marks w = 0, which is the image of z = 1; z → 0 actually runs off to the left. From 95.5 s the dashed "crossing the cut" path passes tangent to the ring, because x = 0.8·ln 1.2 ≈ 0.146 and the ring radius is 0.13. The crossing therefore looks as if it skirts the removed point. Native: frames 137, 200, 219.
*Repair:* add `zero` to the fade at `:262` and re-show it after the roll-back at `:318`. Fade it again at `:330`, then re-show it after the morph back at `:393`, ready for `:426`. An optional small note, "z → 0 runs off to the left", needs no voice change.

**E7. The arrow's length contradicts its label (accuracy).** `lifting_loops.rs:459` draws the drop 1.1 scene units long. Ticks are about 1.19 units per π, so the arrow spans about 0.92π and stops just above the base tick, which is logarithm 0. The label says "πi → −πi", a drop of 2π. Native: frames 276, 292.
*Repair:* I recommend removing the arrow, because the label and the gauge already say this. A true-length arrow would end at native y ≈ 711, 5 px past the safe edge, and would trigger lint.

**E1. "One lap" reads as "one lap πi" (accuracy, partly lint).** At `lifting_loops.rs:261–278` the bracket sits at `base[0] − 0.45`, between the tick labels and the axis. "one lap" sits on the same row as the grey "πi" tick, separated only by a small gap, and the top stub runs through the "i" of "2πi". Lint reports only the stub. Native: frame 118.
*Repair:* move the bracket left of the tick labels (points at `base[0] − 0.85` and `− 0.75`) and put `lap_word` at about `base[0] − 1.35`. Saying "one lap = 2π" is also worth considering.

**E10. The theorem line is crossed (lint: hold @170.4 s, 79%).** `lifting_loops.rs:574` places it at `v(0.6, 0.8)`, on top of the tall axis. Grey "3πi" lies under "on", the axis runs through the "l" of "loop", and the tick crosses "a l". It stays this way until the chapter's last frame. Native: frame 400.
*Repair:* `v(1.2, -3.55)`. The measured width is 6.37 scene units, so it spans native x ≈ 461–1035 at y ≈ 680, clear of the formula above (which ends near y = 646) and of "the number z". There is no `precisely` line in this chapter. Verify on the next render.

**F-edge. "edgerays" (not in lint).** `TO_BOTTOM` (`branch_cut.rs:68`, placed at `:339–345`) overlaps the start of `rays` (`:306`) by a few pixels. Native: frame 200, and frame 219 with the leap.
The guard meant to prevent this passes wrongly. `the_question_and_answer_labels_fit_beside_the_strip` (`:621–632`) estimates width as chars × 0.55 × size / 144, which gives 2.14 scene units for this label; the picture measures 2.71.
*Repair:* drop "the" from `TO_TOP` and `TO_BOTTOM` (card text only; the voice is unchanged). Then 24 chars ≈ 2.32 units, ending near x = 0.57 against the column at 0.9. Raise the guard's constant to 0.70. That fails the current string and passes the new one.

**D2. "2i" on "zw" (lint: before 05_3 @60.6 s, 35%).** `complex_turns.rs:292–298` places `two_i_word` to the right of the arrow tip, where the "zw" label from `:131` still sits. Native: frame 133.
*Repair:* put it left of the axis with `ARROWS.p(two_i.add(C::new(-0.55, 0.0)))`, or fade zw and its label before the 2i arrow appears.

**F-bp. "branch point" is crossed for 40 seconds (lint: before 07_6 @129.8 s and 07_6 @134.0 s).** `branch_cut.rs:429` places it where the spiral's outer arc cuts "poi|nt" (frames 293–308). `:434` omits it from the fade, so it stays through both cautions, with the circle cutting "branc|h" from frame 311 until the last frame. The gap between the coils is about 95 px and the word is about 130 px, so it cannot fit at this size. Native: frames 295, 300, 315, 388.
*Repair:* use `kit.text_left(stage, "branch point", 26.0, gold, v(right, -3.05))`, whose colour ties it to the gold ring, and add `branch` to the `:434` list.

### Medium

| ID | Frames (time) | Source | Defect | Repair |
|---|---|---|---|---|
| D3 | 05: 45–160 (21.0–71.5 s), frame 46 | `complex_turns.rs:164` | Gold "i" is half hidden against the gold zw shaft for 50 s. Not in lint | `ARROWS.p(C::new(-0.2, 1.15))`, left of the axis; check against the purple arc |
| D4 | 05: 192–220 (86.5–97.5 s), frame 199; 399–417 (173.0–180.7 s), frame 410 | `complex_turns.rs:428`, `:667` | Both lines are centred on the output axis, which cuts "clos\|es" and "f" of "forgets". Not in lint | `text_left` at `OUT.centre[0] + 0.15` for both |
| D5 | 05: 385–417 (166.2–180.7 s), frame 386 | `complex_turns.rs:648` | Lint @166.0: −45° ray on "0:", ring on the "g" of "logarithm" | `v(OUT.centre[0] + 2.2, -2.6)`, clear of the outer ring, with a thin red leader; or dim the grid for this beat |
| D6 | 05: 359–417 (155.0–180.7 s), frame 360 | `complex_turns.rs:600` | Lint @154.8: r = e ring through "2," | `v(OUT.p(gz)[0] - 0.5, 2.4)`. This crowds `dial` unless D4's repair is also applied |
| D1 | 05: 245–268 (107.0–116.1 s), frame 248 | `complex_turns.rs:526` | The π gridline stub touches the "2", so it reads "−2π up". Not in lint | `bracket_a[0] + 0.35` |
| D9 | 05: 226–267 (99.5–116.0 s), frame 248 | `complex_turns.rs:497` | Lint @99.6: the horizontal link strikes through "0", which then looks like θ | Raise the label 0.2 for k = 0 |
| D7, D8 | 05: 204–267 (92.0–116.0 s), frame 248 | `complex_turns.rs:437`, `:448` | Lint @92.0: 45° ray on the "l" of "laps:", axis through "further" | Laps at `OUT.centre[0] + 2.35`, value at `+3.15`; aid at `v(OUT.centre[0] - 0.1, -3.5)` |
| D10 | 05: 292–353 (125.5–154.1 s), frame 300 | `complex_turns.rs:543`, `:551`, `:555` | Blue ring, then green segment, paint a stripe through the gold −1 dot. Not in lint | Draw a second gold dot after `half` |
| E5 | 06: 148–245 (60.3–102.5 s), frame 165 | `lifting_loops.rs:137`, `:301` | Gold z = 1 dot stays on the side-circle beat and looks like the start, though the walk starts at z = 2.5. Not in lint | Fade `one` with the side circle's arrival; re-show it for 06_6 |
| E6 | 06: 157–234 (64.5–97.9 s), frame 160 | `lifting_loops.rs:302` | Z axis through the "n" of "not". Not in lint | `Z.centre[0] + 0.15` |
| E11 | 06: 292→293, n3793→n3794 (126.433→126.467 s) | `lifting_loops.rs:488` | `set_tracker_value(turn, 0.0)` with `walk2` live: trail and dot jump in one frame, after the fade has ended | `play!(stage; tween(turn, 0.0, 0.6)?)` |
| F-sides | 07: 159–186 (69.2–80.6 s), frame 170 | `branch_cut.rs:320–321` | Lint @69.0 flags only "just below". Both words are crossed at "j" by the r = 2.3 ring and touch the r = 1.5 ring | `Z.p(C::new(-2.8, ±0.45))`, outside the outer ring |
| F-nocut | 07: 159–186, frame 170 | `branch_cut.rs:262`, `:318–322`, `:330` | Voice says "the two sides of the cut", but the cut has been faded, so gold and purple have nothing between them. Not in lint | Re-show `cut` after `:318`, fade it at `:330` |
| F-pop | 07: 126→127, n1612→n1613 (53.733→53.767 s) | `branch_cut.rs:260–261` | Grid added at full alpha in one frame, while every other element fades. Not in lint | Add an alpha parameter to `grid` (`:73`) driven by a tracker, as `predict` does (`draw.rs:352`) |
| S1 | 07: from about 105 s to frame 306 (132.7 s), frames 270, 300 | `branch_cut.rs:374`, `:434` | "remove (−∞, 0]: −π < Arg z < π", Log and d/dz stay beside the upward and spiral cuts and aren't marked "usual". Not in lint | Add `removed, log, deriv, where_` to the `:374` list, or dim them |
| V1 | 07: about 166–170 s, frame 388 | `branch_cut.rs:506`, `:517` | "That is the strip, with its top edge added" is spoken with no strip on screen; the strip starts rolling back at about 101 s. Not in lint | Crossfade the plane to a static strip for this sentence: solid top edge, dashed bottom edge |

### Low

| ID | Chapter, frames | Note |
|---|---|---|
| F-blocked | 07: 29–37 (12.7–15.0 s), frame 30 | Lint @12.8: unit circle touches "ed". Use `Z.p(C::new(-1.3, 0.75))` (`:178`) |
| F-fine | 07: from 165.5 s, frame 388 | 123-character fine print has 4.7 s, or 26 chars/s, before the chapter ends. Scratch timing; recheck against Oliver |
| F-theta | 07: frame 200 | "θ" in the rays card (`:306`) renders as ϑ. Typeset the card if the shape matters |
| F-gap | 07: 330–388, frame 388 | Wide gap between "principal angle, in π:" and its value. Use `right + 2.2` (`:466`) |
| F-stale | 07: from 147.5 s, frame 388 | Purple dot and live −0.850 stay through the second caution |
| S2 | 07: frame 270 | Cuts end visibly inside the frame while the card says "out to infinity" (spiral tip near scene x = −6.1) |
| F-leap | 07: frame 219 | Leap dots are 2π − 0.5 apart under the words "a jump of 2π" (`:358–361`). Using the marked-sides offset would put them on the edges |
| D11 | 05: 116.4–125 s | Blue x = 0 column stays lit through the think hold, hinting at the real part. Decision for root |
| D12 | 05, 06: frames 46, 276 | `arrow` heads are hollow, though `draw.rs:174–178` says "A solid arrow" |
| D13 | 05: from 150 s | `k` in e^{(π+2πk)i} is not defined on screen |
| D14 | 05: frames 164–166 (72.5–73.5 s) | Title alone during "Watch what that does to a grid". Plain carets in cards beside typeset formulas |
| E2 | 06: 110–128, frame 118 | Gold "2πi" sits tight against the red ring, with no overlap |
| E3 | 06: throughout | Base tick has no "0" label, though the voice says "the logarithm zero" |
| E8 | 06: 246–258, 293–320, frame 296 | Zero-length trails leave specks on the base tick |
| E9 | 06: 238–408, frame 292 | Tall axis has no label after 98 s; `w_word` is faded at `:293` and not restored at `:398` |
| E12 | 06: 310+ | ℓ is named only by colour; 2πi is roman in cards, italic in tex |
| E14 | 06: frame 165 | Teal ring overshoots the end of the Z x-axis |
| E15 | 06: 39.5–43.4 s | Think hold shows readout 1.86 and a dot 0.14π from the 2πi tick, which nearly gives the answer. Decision for root |
| Layout | 07 | Stack left edges are staggered (`text_left` against centred at 3.9); roman i in cards, italic in tex |

### Pacing, on scratch timing only

- **05:** nothing beyond D14.
- **06:** frozen 9–15 s; holds at 0.2π to 25.5 s; holds at 118–126 s and 144–151.4 s.
- **07:** plain plane 1.0–4.5 s; static 33–41 s, 42–52.4 s and 96–100.9 s; "a choice, not a crack" arrives about 4 s after 07_5 starts; static 135–142 s and 158–165 s.
- **Reveals:** all five predictions (05_1, 05_5, 06_2, 06_4, 07_3) hold still and don't leak a label. D11 and E15 are the two softer hints.

## 5. Lint reconciliation

All 13 findings in the three `layout.tsv` files are accounted for.

| Lint finding | Result |
|---|---|
| 05 "zw"/"2i" overlap @60.6 | Confirmed, D2 |
| 05 "laps:" @92.0 | Confirmed, D7 |
| 05 "each lap drawn…" @92.0 | Confirmed, D8 |
| 05 "0" @99.6 | Confirmed, D9 |
| 05 "length 2, angle 0.6π" @154.8 | Confirmed, D6 |
| 05 "0: no logarithm" @166.0 | Confirmed, D5 |
| 06 edge, x −7.00 @0.8 | Real and negligible: an axis line 0.04 past safe, no text |
| 06 "2πi" @48.4 | Confirmed, part of E1 |
| 06 "3πi" and theorem @170.4 | Confirmed, E10 |
| 07 "blocked" @12.8 | Confirmed, F-blocked |
| 07 "just below" @69.0 | Confirmed, F-sides, which also affects "just above" |
| 07 "branch point" before 07_6 @129.8 | Confirmed, F-bp |
| 07 "branch point" 07_6 @134.0 | Confirmed, F-bp |

**Lint did not report** D1, D3, D4, D10, E5, E6, E7, F-edge, F-zero or the "one lap πi" half of E1. F-edge and D4 are plain text collisions a viewer would notice. I did not investigate why lint missed them.

## 6. Repair notes

- **Shared edits.** F-zero and F-nocut both change `zero` and `cut` visibility across 07_3 to 07_5, so make them together. S1 and F-bp both change the `:434` list. D4 must go with D6.
- **Width guard.** The estimate at `branch_cut.rs:622` ran about 21% short here (2.14 against 2.71 scene units). Any label sized against it can pass and still collide.
- **Scripts.** No `script.tsv` change is proposed. F-edge changes card text only.

## 7. Checked and sound

**Chapter 05**

- zw at r ≈ 1.78, 80.4° (expected 1.8, 80°); iz at 1.2, 120°.
- Rings at e⁻¹, e^−½, 1, e^½, e.
- g at r ≈ 2, 107.8° (0.6π); the ln 2 column's x position.
- Dots at 0.6π, 2.6π, −1.4π; column πi, ±, 3πi; |e^{x+iy}| = e^x > 0.

**Chapter 06**

- 0.2π = 36°; 1.86π dot; 0.5 at the top; 1.0 at −1.
- Second-lap angles at seven checked points.
- Side loop centre 1.7, radius 0.8 (equals `loop.side` in `facts.rs:61` and the oracle).
- Log dot at frames 167, 168, 200, 201; closed oval returns at 85.0 s.
- Gauge k goes 0 → −1 just past −1 (`loop.principal_gap` = "0,-1"); red rung arrow sign; theorem wording matches the voice.

**Chapter 07**

- Walker stops at ±0.47 lap; Log i = πi/2 and Log(−i) = −πi/2 with ±90° arcs.
- Strip x offsets match ln r for r = .4, .7, 1, 1.5, 2.3 at `W_SCALE` 0.8; heights match π/4 steps.
- Gold side goes to the top edge, purple to the bottom.
- Window −3π/2 < angle < π/2; spiral radius monotone with no self-crossing.
- Arg(−1) = π; live angle +0.850, +0.857, +0.903, +0.988, −0.923, −0.865, −0.850 at 142.5–145.5 s, matching dot positions and the wrap.
- e^{Log z} = z for z ≠ 0; Log(e^w) = w ⟺ −π < Im w ≤ π; Log(e^{2πi}) = 0 ≠ 2πi; the fine-print statement.

## 8. Rejected suspects

| Suspect | Why rejected |
|---|---|
| 06 `drop_word` against the gauge | Clear at frame 276 |
| 06 `differ` tex width | About 5 px of clearance at frames 310 and 400: tight, not crossing |
| 06 "contradiction" against other text | No contact |
| 06 pop at `lifting_loops.rs:400` | `walk2` does not exist yet, so nothing visibly jumps |
| 06 `would_word` against `fixed` | No contact |
| 07 fine print against the y-axis | Axis ends well above it at frame 388 |
| 07 right-hand stack in 07_6 | No collisions through frame 388 |

## 9. Intentional, not flagged

- `swap` crossfades, including 06 frames 204 and 236, the 07 title at 309–310, upward to spiral at 261–262, and `cut_word` to `name` at 38.
- The drawing-aid lap spread, which is stated on screen.
- The exaggerated sides (0.07 rad, measured about 3.9°), which follow the storyboard.
- Prediction bars and the red cut itself.
- No fade at the end of chapters 05, 06 and 07, which is consistent across all three. Joins are out of scope.

## 10. Residual gaps

- **Samples only.** 1,216 of 15,855 video frames were reviewed, plus two short decoded runs. Motion between samples was not seen.
- **Scratch timing.** Nothing here is checked against Oliver, so holds, reveal timing, card reading time and the frame ranges will all move.
- **Not done.** No listening, fine sync, continuous-motion review, chapter joins, full AV acceptance, build or render.
- **Repairs untested.** Positions are computed from measured pixels, and each repaired layout needs its own render and lint pass.
- **Viewing method.** Older sheets rest on notes made at viewing time (section 3). Only the 30 listed frames were inspected at native resolution; other low findings rest on the 960×540 sheet panels.
- **Lint.** I did not investigate why lint missed the items in section 5.

## 11. Files

| Path | Contents |
|---|---|
| `REVIEW.md` | This report |
| `NOTES.md` | Progress notes in viewing order; IDs there match this report, and early entries are unconfirmed candidates |
| `sheet-ledger.tsv` | One row per sheet: frames, times, sha256 |
| `crops/`, `crops.sha256` | Native crops and the decoded-run montages, with hashes |
| `checks/` | `crop.py`, `persist.py`, `ledger.py`, `persist.out.txt` |

I made no canonical edits, Bead changes, builds, renders, synthesis, commits or memory changes, and used no extra agents.
