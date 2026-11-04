# Logarithm chapters 04 (remaining), 05, 06: picture review

Reviewer: RubyRaven, for BlueSalmon, Mail 1923, bead `mx-video-program-program-87do.63`.
Reviewed: root's `logarithm-combined-repair-frames01` sheets, render inputs `bf0169836cdd3708…`, against canonical source and `script.tsv`.
Narration stays ElevenLabs Oliver. No audio was generated or listened to.

**This is a scratch-voice picture review. It is not listening, fine-sync, continuous-motion or final AV acceptance.**

## Verdict

**I found no wrong number, sign or formula in any picture I viewed. Chapters 05 and 06 do have concept-presentation defects: the pictures say less than, or something other than, the voice. Narrow patches for them are staged here.**

| Chapter | Viewed | Verdict |
|---|---|---|
| 04 `undoing_growth`, 98.0 to 190.15 s | 206 of 430 pictures; root viewed the rest | No concept defect. One low labelling patch (S1). |
| 05 `complex_turns`, whole chapter | 425 of 425 | Two medium winding and sync defects (C1, C2), one low-medium collision (C7), one low order fix (S3). |
| 06 `lifting_loops`, whole chapter | 417 of 417 | One medium notation defect at the crux of the proof (C4), one medium-low label defect (C6b), four low fixes (C3, S4, S5, S6). |

**On what this unlocks.** The call is root's. My view is that nothing here is a reason to hold a real-voice preview, because no patch changes a cue, a word or a sentence index, so recordings are unaffected. I would still take C1, C2 and C4 before calling these chapters finished. The patches themselves have not been compiled or rendered, so **no one has seen a patched picture yet.**

## Disclosure

**This review is not independent.**

- As RubyRaven I wrote the original rebuild of `logarithm_mysteries`, including chapters 04 to 06 (round 1 Mail 785, round 2 Mail 981).
- Later hands changed them: LavenderPike reviewed and repaired 01 to 04 (Mails 1448, 1502). An agent sharing my memory store reviewed 05 to 07 (Mails 1418, 1479). BronzeJaguar repaired them (Mails 1484, 1531). Root integrated them and made the compile fixes.
- I also wrote the final80 bound plan and the cue-checker repair.
- I wrote every patch here, so treat them as an author's patches, not as reviewed code.

**I logged sheets I had not seen, and retracted those rows.** During this assignment I logged `05_complex_turns` sheets 010 to 055 as viewed, with detailed notes including a radii measurement, while their Read results in my context read `[media removed: request limit]`. My notes for sheets 056 to 059 were one sheet off.

- I told the owner when I found it.
- I moved the old log to `coverage/retracted/` unchanged and retracted those 50 rows.
- I built a logger that refuses a row unless the header time I read from the image matches `sampling.json` within 0.0015 s.
- I then viewed all 50 sheets again and logged them afresh.

No finding below rests on a retracted note.

**Older rows carry less protection.** 92 of the 264 rows came from the earlier logger, which had no image-time check: 04 sheets 056 to 107, and 05 sheets 000 to 009 and 060 to 089.

- I displayed 11 of those sheets again (`coverage/spot-check.tsv`). Ten notes matched. The note on 04 `sheet-080` held a clause that belongs to a later sheet ("line at t=50 reads about 12"). The fact is true, and I measured it again on `sheet-094`.
- The other 81 were checked only by a scan for time offsets (67 time-like values, 9 out of range, all of them quantities rather than times, no offset found).
- Seven small note corrections, with old and new text, are in `coverage/log-corrections.tsv`.

## Coverage

| Chapter | Sheets | Pictures | Time, s | Folder total |
|---|---|---|---|---|
| `04_undoing_growth` | 056 to 107 = 52 | 224 to 429 = 206 | 98.0 to 190.15 | 108 sheets, 430 pictures |
| `05_complex_turns` | 000 to 106 = 107 | 0 to 424 = 425 | 0 to 181.65 | 107 sheets, 425 pictures |
| `06_lifting_loops` | 000 to 104 = 105 | 0 to 416 = 417 | 0 to 178.15 | 105 sheets, 417 pictures |
| **Total** | **264** | **1,048** | | |

A script compared `coverage/sheets-viewed.tsv` with the folder: every assigned sheet is present, its SHA-256 and first and last times match, and none is marked undisplayed. A log row is my record of a view. It cannot prove on its own that an image reached me, which is why the retraction above matters.

I opened seven frames at full size to settle disputed anchors (`coverage/viewed-frames.tsv`).

## Evidence and limits

| Source | What it supports | What it cannot show |
|---|---|---|
| Sheets: half-second samples, cue boundaries, lint anchors, 1280×720 scratch render | Layout, labels, order, values at sampled instants | Motion between samples, real-voice sync, anything audible |
| Canonical source and `script.tsv` | Intended geometry, what the voice says | How long Oliver's phrases really take; sentence starts in the scratch render are character shares |
| `checks/build_candidates.py` | Each patch applies exactly and round-trips; rustfmt parses and accepts the candidates | Types, borrows, Clippy: **nothing was compiled** |
| `checks/desk_checks.py`, TEXT and MATHS | Cue calls unchanged; `check_cues.py` passes on both copies; geometry and budgets, with baseline controls | Real text widths in places: widths are measured from today's frames or estimated |
| `checks/desk_checks.py`, PIXELS | Boxes that patched labels will occupy are empty in **today's baseline frames** at the sampled times | That the patched label looks right |

`sampling.json`'s `layout_findings` merges this render's lint with the old `logarithm-opus-rebuild-preview02` lint. This render's `layout.tsv` has one row in 04 (before 04_3, outside my range), none in 05, and one in 06 (C3). The old-only rows were not seen in pixels: 05 "zw"/"2i" at 60.6, "laps:" at 92.0, "0" at 99.6, "length 2" at 154.8, "0: no logarithm" at 166.0; 06 "2πi" at 48.4, "3πi" at 170.4.

## Defects and corrections

Lines are in `baseline/`. Times are from the scratch render. Every correction is staged in `patches/`.

### C1, medium, winding: "again and again" is not shown (`complex_turns.rs:500`, 93.0 to 98.5 s, frames 212 to 227)

05_3 ends: "But a vertical line is infinitely long, and a circle closes after one lap. So each vertical line wraps around its circle again and again." The picture climbs only from 2π to 3π, half a lap more. The counter appears at "laps: 1" and **never changes** (frames 212, 216, 223). The on-screen note beside it says the circle closes after one lap, so the page reads as one lap, which is the opposite of the sentence.

**Correction.** The climb runs on to 6.5π, so the count moves 1, 2, 3 and stops a quarter past the third lap. The input dot waits at the strip top (y = 3π) under a new arrow and "the line goes on", while the output keeps winding. `LAP_SPREAD` goes from 0.045 to 0.06, so the laps stay separate strokes.

The picture budget for that sentence grows from 3.8 s to 4.2 s (scratch share about 4.8 s). Under an assumed smooth rate each count stands 1.62, 0.71 and 1.87 s. **Whether 0.71 s reads at real speed has not been seen.** The green dot rides the true unit circle, so it ends about 15 px inside the trace's tip. The on-screen aid already says laps are drawn further out; check it in the first patched picture.

### C2, medium, concept and sync: the output sits at −1 while the voice says one (`complex_turns.rs:510`, 98.7 to 117.07 s, frames 228 to 275)

05_4: "Zero and two pi times i are different inputs, and both land on one." For the whole cue the climb's green dot stays at −1, its blue dot at 3π, and "laps: 1" and the aid stay up. At frame 255 the green dot lies on the purple 0→1 link. The "4πi above" arrow head also lands on the blue dot (C2b, frames 245 to 275).

**Correction.** The climb's objects, including the new arrow and note, leave in the existing 0.3 s fade at the start of 05_4. The stage keeps `v_line`. They are removed from the 05_5 `tidy` list (`:579`) because fading an already removed mobject re-roots it as a ghost at this pin.

### C4, medium, concept notation: the formula calls ours a function of the point (`lifting_loops.rs:76`, 133.0 to 178.15 s)

The card reads `e^{L(z)} = z = e^{ℓ(z)} ⇒ L(z) − ℓ(z) = 2πik`. The whole proof turns on ours **not** being a function of the point: at z = 1 it has the value 0 and, one lap later, 2πi. 06_7 says so aloud ("Theirs is a function of the point… Ours finishes two pi times i higher"), and the card still contradicts it for 45 s. The letter ℓ is also never tied to "ours: carried along" except by colour.

**Correction.** `DIFFER` becomes `e^{L(z)} = z = e^{ℓ} ⇒ L(z) − ℓ = 2πik, k whole`. The colour needle becomes `\ell`, and "(ℓ)" appears beside "ours: carried along" with the formula.

Source read: `find_occurrences` is a plain substring match, so `\ell` selects both occurrences, and `\ell` does not match any other command in that string. The colours themselves have not been seen in pixels.

### C6b, medium-low, labels: 0 is unmarked, and its note sits under the wrong circle (`lifting_loops.rs:138`, `:418`, about 64 s to the end; full size frame 170)

06_4: "a circle off to one side, which zero is not inside". The chapter is about loops around 0, yet 0 is never marked in the z plane. "0 is not inside" stands directly under the grey unit circle, **which does hold 0**, so read by position it is false.

**Correction.** The note moves under the side circle. A small dot and "0" appear with the side circle and stay to the end, because the closing theorem is about loops that wind around 0. The name's box is empty in 35 sampled baseline frames from 60 s on.

### C7, low-medium, collision (`complex_turns.rs:628`, about 151 to 155.3 s, full size frame 350)

`e^{(π+2πk)i} = −1, k whole` spans px 294 to 572. The dim output axis starts at px 532 and runs under "whole". This render's lint is empty for 05, so it does not catch this.

**Correction.** The card moves left, to span about −4.39 to −1.31, clear of the strip (−4.54) and the axis (−1.20). A test now guards the axis.

### C3, low, left-edge axis, the only lint row in 06 (`lifting_loops.rs:38`)

The z axis's left end is at −7.00, past the safe edge −6.96. It is a line and the frame does not cut it. **Correction:** `half_w` goes from 2.9 to 2.8, left end −6.90. No text or dot depends on the removed length. The right end still stops inside the side circle, as today (cosmetic).

### Low fixes

| Id | Where | Defect | Correction |
|---|---|---|---|
| S1 | `undoing_growth.rs:473`, 04_4 to early 04_5, frames up to 243 | The reflection's mirror is never named, and the voice never says "y equals x" | "y = x" just right of the line's top, leaving with the graph |
| S3 | `complex_turns.rs:617`, 141.5 to 142.5 s | Column dots rise −3π, −π, π, 3π while the voice names π, 3π, −π | Sort to 1, 3, −1, −3 before drawing. The positions are unchanged |
| S4 | `lifting_loops.rs:330`, 16.0 to 25.5 s | The walk stands still under "As the number turns" | A 2.4 s tween from turn 0.1 to 0.2, short of the quarter. Budget 2.8 s against a scratch share of about 6.4 s |
| S5 | `lifting_loops.rs:626`, 141.0 s | "fixed" appears about 9 s before "so it stays fixed" | Moved after the walk that shows it. Same total time in that sentence |
| S6 | `lifting_loops.rs:364`, 47.0 s, full size frame 111 | Red ring about 2 px from "2πi" at 720p | Label 0.05 further right |

## Checked, no defect

**04, 98 s to the end.** y = ln x as the reflection; red wall at x = 0 and "no real logarithm" on negatives; tangent y = x − 1 and ln 1.01 ≈ 0.00995; ruler positions (1 under e, 2 at e² left of 8, log₂ ticks under 1, 2, 4, 8); ln(ab) with a, b > 0; 2 × 4 = 8 bars; change of base; ratio-axis line (t = 50 reads about 12); 13.9, 69.3, 6.91/0.05 ≈ 138, 138.6, 2¹⁰ = 1024; ten doubling dots; no answer during the think bar from 151.5.

**05.** Arrow lengths and angles for z, w, zw and iz; i × i = ? held through its think bar; ruler proportions; `e^{x+iy}` card and e^{ln 2 + iπ/2} = 2i; ray at 45°; eight rays; the circle closes at 2π; ring radii in ratio e^{1/2}; 0 and 2πi links; "2π up = one lap round"; −1 question with no answer during its think bar, 122 to 125.5; column at x = 0, angle π, dots at odd π; generic number at length 2, 0.6π with its purple dots; "0: no logarithm"; |e^{x+iy}| = e^x > 0; "a dial that forgets laps".

**06.** Column ticks; walk values against turn at every sampled anchor, including 2.00 at 44.5, 4.00 at 56.0 and reverse to 2.00 at 58.5; no answer during the think bar 39.5 to 43.4; side circle spans 0.9 to 2.5; z = (1.268, 0.676) at 70.0; closed logarithm oval x −0.10 to 0.92, y ±0.49 = arcsin(0.8/1.7); "the logarithm returns: change 0"; winding table.

The candidate's gauge jumps at the crossing of −1, in the right direction (candidate − ours = −2πi, frames at 114.0 and 114.5). 06_7's teal dot, ring and "k would have to jump" are consistent. The final theorem is true: a logarithm on the image would make the lift close, which forces winding 0.

## Noted, not patched: root's choice

- C5: the 05_2 arrow axes carry no 0, 1 or i labels, and −1 has no point label. Labelling risks collisions.
- C6: the 06 column shows only 0 to 4πi, and −πi appears only in the 06_6 strip.
- The gauge starts at k = 0 as one instance, with no note that any rung would serve.
- W2: the close-up axes carry no scale.
- The candidate card says "between −π and π" without the half-open end. The voice matches and the strip is consistent.
- Under the voice, a 1 s rewind at 127 to 128 s.
- Long holds at scratch timing: 05_3 85.5 to 93, 05_20 about 11 s from 61.6, 06_20 about 7 s from 145. Recheck on Oliver timing.
- Swap overprints in one sampled frame each: 05 at 29.0, 47.5, 60.0; 06 at 85.0, 98.167. Title and step cross-fades in 04 at 138.5 and 155.733, under 0.4 s.
- Teal dot to formula, 4 px at 156.0 (frame 364): tight, not a defect.

## Patches

| Patch | Findings | Size | SHA-256 |
|---|---|---|---|
| `patches/04-undoing_growth.patch` | S1 | +18 −3 | `28452f65bfcc8792…` |
| `patches/05-complex_turns.patch` | C1, C2, C7, S3 | +85 −12 | `1e13c30a27349394…` |
| `patches/06-lifting_loops.patch` | C3, C4, C6b, S4, S5, S6 | +63 −10 | `a93717e1eb36ad2d…` |

Each is against the canonical file's hash in `base.sha256`, and they apply independently. `baseline/` holds byte copies and `candidate/` the results.

- `patch -p1` round-tripped all three in a temporary directory, and `rustfmt --check` passes on each candidate.
- `kit.say` and `kit.until_sentence` calls are identical in order and arguments, baseline against candidate (35, 33, 34 calls).
- `script.tsv` is unchanged. `docs/check_cues.py` passes on both. Its listing changes only by the new `"0"` label and the reworded formula.

**New tests, none yet run.** They may fail or not compile:

- `the_count_moves_on_screen_and_the_wound_trace_stays_inside_the_next_circle`
- `the_note_past_the_strip_clears_the_strip_and_the_title`
- `zero_is_marked_inside_the_unit_circle_clear_of_both_axes`
- `the_formula_gives_ours_no_argument_because_ours_is_not_a_function_of_the_point`
- `the_numbers_axis_stays_inside_the_safe_area`

Two existing tests gain assertions (`the_typeset_cards_fit_their_places`, `the_side_circle_note_clears_the_axis_and_both_circles`), and one gains const asserts (`the_flip_labels_keep_clear_of_each_other_the_names_and_the_mirror`).

**Integrating.** Root's unintegrated `combined-repair-stage05/bound-plan/` carries line numbers in these files, which the patches shift (its 04_2 anchor at `undoing_growth.rs:112` moves by +3, and `complex_turns.rs:359` and `:509` move too). If canonical moves first, use `git merge-file` with `baseline/` as the base, not `git apply`. I did not examine the plan's dispositions for 06_2 and 06_4 against S4 and C6b, so check those two when merging.

## Not verified

- Compilation, types, borrows, Clippy. Text checks cannot see them.
- Any patched picture. All pixel checks are on the baseline.
- That `\ell` colours both occurrences in pixels, and how "(ℓ)" aligns beside "ours: carried along".
- Whether the count's 0.71 s step reads at real speed.
- Real Oliver sentence timing. `say` waits for the picture, so an overrun would delay the next cue. 05_3's last sentence and 06_2's first sentence are the two budgets to recheck.
- Listening, fine-sync, continuous motion, final AV.
- `tween`'s rate. I assumed the default is smooth (peak 1.875× average) for the count times.

## Open for root

1. Decide on the patches. Build with tests and Clippy.
2. Render, then look at: 05 about 93 to 99 s and the start of 05_4; the 05_6 rule; 06_4's note and 0; 06 at 133 s for the formula and "(ℓ)"; 04_4's "y = x"; and confirm 06's lint row has cleared.
3. Decide the "noted, not patched" list.
4. Run real-voice preview and independent review as usual. This bundle replaces neither.
