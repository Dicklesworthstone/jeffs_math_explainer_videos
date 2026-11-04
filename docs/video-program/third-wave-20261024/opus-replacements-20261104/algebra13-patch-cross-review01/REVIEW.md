# 13_arithmetic patches A and B: cross-review

Reviewer: RubyRaven, for BlueSalmon, Mail 1858, bead `mx-video-program-program-87do.64`.
Reviewed: BronzeJaguar's `algebra-picture-review-13_arithmetic/` (REVIEW.md, patches A and B, candidates, baseline, desk checks).
Narration stays ElevenLabs Oliver. No audio was generated or listened to. **This is not listening, narrated sync or final AV acceptance.**

## Verdict

**A+B is mathematically correct and repairs its main defects. It should be built with the corrections in patch C.**

- I found no mathematical defect in poses, trail boxes, `arithmetic.closure.product` or composition order.
- The source replay found no hide or re-show fault in 13_1 to 13_8 or 13_12. A hand walk of 13_9 to 13_11 found none either.
- Four timing or naming defects remain. Two were introduced by the patches themselves (F1, F2).
- **Layer A should not ship alone**, despite the author's "A can ship alone" (A1 below).

Patch C is staged against candidate-AB. It is **uncompiled and unrendered, and nobody else has reviewed it.** No candidate has pixels yet.

## Disclosure

- I did not write `arithmetic.rs`, patch A or patch B.
- This is not independent of lineage. BronzeJaguar and I are the same kind of agent and share a memory store. A note in that store already named the author's D1 to D3 before I began, and the assignment had me read their review first.
- My checks import nothing from their bundle: `checks/replay.py` and `checks/group_and_boxes.py` carry their own tile model and read the Rust as text.
- I wrote patch C myself, so treat it as an author's patch, not as reviewed code.

## Evidence and limits

| Source | What it supports | What it cannot show |
|---|---|---|
| Candidate source, read in full | Logic, order, lifecycle | Types, borrows, Clippy: nothing was compiled |
| `checks/replay.py` | Sums of literal run times, pose against name, fades against stage state | Real motion; no pixels exist for any candidate |
| Oliver take01 starts from root's `arithmetic-sentence-audit02.json` | Windows on the voice | These are ASR estimates; phrase times are character shares. Margins under about 0.2 s are not proven |
| Baseline frames (scratch voice, 720p) | The author's defect claims | Anything about the candidates; narrated sync |

I opened 14 frames at full size and one sheet (`viewed-frames.tsv`), chosen to test disputed claims. This is not a full-film picture pass.
Times below are seconds from the start of the cue, on Oliver take01 estimates.

## Defects and corrections

Line numbers are in `candidate-AB/.../src/arithmetic.rs` (AB) or `candidate-A/...` (A). Each correction is one hunk group in `patches/C-cross-review-corrections.patch`.

### F1, medium, introduced by B: `r³ r = e` is unreadable (AB:283 to 291)

The name is fully shown for **0.04 s**: it finishes fading in at 13.78, and its fade-out starts at 13.82 for "A half-turn". The floor at 15 characters per second is 0.33 s. A and baseline hold it for 3.42 s. B's budget table shows window 3.54 against ask 3.50 and counts no hold. Expect zero hold on a scratch-voice preview too.

Correction X4 lets the rack leave on "an undo move", so the tile is ready before its sentence:

```rust
until_phrase(stage, &kit, "13_8", 1, UNDO_PHRASE)?;   // was until_sentence(.., 2) here
d4g::hide(stage, &moves.slots, 0.4)?;
let undo = ...; let undo_objects = ...;
kit.until_sentence(stage, "13_8", 2)?;                // moved below the new tile
```

Result: rack gone by about 9.7, hold 0.44 s. The rack still stays through "stays inside the rack".

### F2, medium, introduced by A: `r²` and `r³` leave before the voice names them (AB:193 to 196, A:170 to 173)

`step` clears the old name before the next turn, on a fixed 4 s rhythm.

| Name | Fully shown | Fade-out ends | Voice says it, about | Picture at that word |
|---|---|---|---|---|
| `r²` | 7.00 to 8.00 | 8.40 | 9.2 ("We write r squared" starts 8.18) | No name; third turn running, 8.40 to 10.10 |
| `r³` | 11.00 to 12.00 | 12.40 | 12.5 ("give r cubed") | No name; fourth turn starting |

The first turn also starts at 0.40, under "Try a calculation". The author repaired that same fault on 13_4 and 13_5 (D11, D13).

Correction X2 replaces the loop:

```rust
kit.until_sentence(stage, "13_3", 1)?;
fade(stage, &[left_label], false)?;
perform(stage, left, 1., LONE_RT, &[])?;
perform(stage, left, 2., LONE_RT, &[])?;
kit.until_sentence(stage, "13_3", 3)?;
left_label = caption(stage, &kit, "r²", LEFT_LABEL.0, LEFT_LABEL.1)?;
kit.until_sentence(stage, "13_3", 4)?;
left_label = step(stage, &kit, left, 3., left_label, "r³", LEFT_LABEL)?;
stage.wait(0.6)?;
```

Result: both turns finish by 5.34, inside "Do a quarter-turn, and then another" (window 3.58, ask 3.50). `r²` holds 1.84 s across its sentence. `r³` is whole from 13.42 and holds 0.60 s, about 0.9 s after "r cubed" begins. The fourth turn ends at 16.12 and `r⁴ = e` still lands on 16.62, with zero slack on these estimates. `r` is no longer captioned here; the voice does not name it in 13_3.

### F3, medium-low, in B: the reversed turn is shown before the voice reverses it (AB:253 to 256)

`s r = r⁻¹ s` lands at 3.42 to 3.82 and the rewrite finishes at 7.02. "if we reverse" comes at about 8.69. The picture then sits idle for 3.20 s. This is the author's own D3 fault in a new place.

Correction X3 draws the bracket on sentence 1, then waits:

```rust
let bracket = grouping(stage, -2.4, -0.6, 0.6)?;
until_phrase(stage, &kit, "13_7", 1, REVERSE_PHRASE)?;
let rule = caption(stage, &kit, "s r = r⁻¹ s", 0., -0.7)?;
fade(stage, &[bracket, factors[0], factors[1]], false)?;   // stage.wait(1.6) removed
```

Result: the rewrite ends at 10.29, and the "pairs of flips" box is 0.07 s late. `const REVERSE_PHRASE: &str = "if we reverse";`.

### F4, medium-low, kept by A and B: `s` sits under a tile that has not flipped (AB:175 to 176, A:152 to 153)

For 2.10 s (3.54 to 5.64) `s` names an unflipped or moving tile. It is the only pose/name interval left in AB. The author's D2 table does not list it. Pixels: `sheet-014` frame 57 shows `s` under the unflipped tile, and frames 58 and 59 show it during the flip.

Correction X1 swaps the two lines, so `advance(stage, demo_flip, 1.)` runs before the caption:

```rust
advance(stage, demo_flip, 1.)?;
let mut demo_s = caption(stage, &kit, "s", RIGHT_LABEL.0, RIGHT_LABEL.1)?;
```

`s` then arrives at 5.74 to 6.14, before sentence 2 at 6.76.
**This one is a judgement call for root:** the sentence says "Let s mean the horizontal flip", name first, so the existing order also has a case. I recommend the swap to keep D2's rule without exception.

### Low

- **F5 (AB:563):** the comment says `r times s`, but the pair is now `(S, R)`. X5 changes it to `s times r`.
- **F6 (AB:1138):** nothing tests the poses the labels name. After B, the film stops at `left` t=8 and `right` t=2, while `conjugation_and_reduction_agree_on_every_corner` still uses t=9 and t=5, which are no longer performed. X6 adds `the_tiles_stop_under_their_names`. I left the old test in place. Root may also trim `LEFT` and `RIGHT`.
- **F7 (AB:71):** the `"e"` caption arm is unused after B. It is harmless. I left it for root, in line with the author's N1.

### Layer A alone

- **A1:** `step` follows `step` with no hold. `r` and `s` (13_5) and `r` and `e` (13_6) are never fully held: each fades in and straight out.
- **A2:** `r` in 13_8 appears before its turn and stays through it, 2.10 s (0.40 to 2.50), still at y −2. So L5's near-touch from frame 0358 can recur, and "Layer A clears the moving labels" is not true at this site.
- **A3:** D9's overrun remains: 13_8's sentence 4 is 2.90 s late.

B removes all three.

## What you asked me to verify

| Item | Result | Evidence |
|---|---|---|
| Pose and name | AB has 1 interval (F4); ABC has 0. Baseline has 19, A has 2 | `replay-*.out`. All final poses against matrices: `group_and_boxes-AB.out` §3. Frames 0081, 0125, 0151, 0270 |
| Composed function boxes | **Correct.** Row 1 holds 1, 3, 9 and g∘h(1)=9. Row 2 holds 3, 9, 8 and f∘g(3)=8. Each box stops short of the next arrow. Row labels and gold packages agree | §5, computed from the candidate's constants. Frame 0480 confirms D1, including 15 and 26 |
| `closure.product` `r^3s` | **Correct.** `CLOSURE_PAIR = (S, R)`; `main.rs:339` emits `.tex()`, so ASCII `r^3s`. Oracle `TEX[compose(s_, r_)]` gives the same. Of 240 oracle rows, exactly this key changes. On-screen `s r = r³s` frames slots 4, 1, 7. `s r` is not a rack name and `r³s` is | §2, §4. The oracle ran; the Rust fact did not |
| Map composition | **Correct throughout.** Left acts last in `compose`, `Elem::mul`, oracle and `f∘g`. 13_2, 13_5, 13_6 and both 13_9 packagings pose as spoken. `s(rs)` pairs with `f∘(g∘h)` as "first two" | §1 (all 64 products), §3, §5 |
| Hide and re-show | **No fault found.** No double fade, so the `fade_out` ghost cannot fire at these sites. No cleared handle is re-shown: restarted tiles use new trackers. `relight` clears updaters first and fades in `lit` from zero. `shown` is reassigned after each `relight`. `perform(.., &[])` is safe because its tween is always present | Replay LIFECYCLE 0 for baseline, A, AB, ABC on 13_1 to 13_8 and 13_12. 13_9 to 13_11 walked by hand only |
| Waits | All `until_sentence` indices in range and in order. All 15 phrases found in their sentences; ABC has 17. All 12 sentence counts match. Only overrun in AB: 13_8 sentence 4, −0.40 s, as the author disclosed | `group_and_boxes-*.out` §6, `replay-AB.out` |
| Reading holds | F1 and F2 above. D5's repair holds: Closure 2.74, Associativity 1.79, Identity 2.13, Inverse 1.80 s, all above 15 characters per second, including every staged line | `replay-AB.out`. Frames 0555, 0566, 0579 |

## The author's other claims

**Reproduced:** D1, D2, D3, D5, D7, D10, D14 and L5 against pixels or source. The 13_1, 13_5, 13_7, 13_8 and 13_12 budget sums. Both patches byte-equal to their candidates. rustfmt clean. Oracle diff of one line. D3's "about 12.4 s" (the replay has the caption at 12.00 to 12.40). The 1.81 reach comment (about 1.82 on frame 0358).

**Corrections to their numbers, none harmful:**

- 13_7's rack beat asks 1.62 s, not 2.01. `lagged` takes a ratio, so the rack's `appear(.., 0.3, 0.08)` runs 0.47 s, not 0.86.
- 13_6's window of 3.46 s is not reproducible from the bundle. The large ASR model merges the two sentences; the small model puts "Flip" at 3.18, which gives 3.32. On that estimate `s r s` lands 0.13 s into the next sentence.
- D2's table has 10 rows and the replay finds 19 intervals on baseline, for example `r²` under a turning tile in frame 0125. Patch A covers all but F4 and the 13_8 site, which B removes.

**Checked and not a defect:**

- `RS` was already absent from baseline's imports, and `recolor` is still used.
- The right tile's `r⁴ = e` in 13_4 is true of its pose though that tile never moved. It serves "finish in the same arrangement", so I would keep it.
- Row 2's box starts under machine h because the 3 there is h's output, the package's input.
- `row_frame` clears the rack by about 8 px on frame 0510's geometry.

## Root's part

1. `cmp` baseline against canonical. At this review they still match: arithmetic `78ed9b54`, oracle `80378269`, script `83619d48`.
2. Apply A, then B, then C. C touches `arithmetic.rs` only. `checks/outputs/patch_check.out` shows each applied with `patch -p1` to a scratch copy, byte-equal to its candidate.
3. Run the guarded build, tests, Clippy, oracle and facts check. Patch C adds 1 test and 2 phrases (`PHRASES` becomes 17); facts are unchanged from AB.
4. On the re-render, look at:
   - The author's list: `normal` gliding over the rack, the row frames, the 13_1 thumbnails, and tiles popping in after fades.
   - **Fast motions:** `smooth` peaks at 1.875 times the average rate. 13_1's long route peaks at 2.8 moves/s, about 0.36 s per flip. 13_8's three quarter-turns peak near 360°/s and its half-turns near 480°/s. These are source figures; I cannot judge them without motion.
   - The 13_8 half-turns' 0.40 s overrun, which I left in place.
5. Decide F4 (X1). Any narration plan that hashes `arithmetic.rs` needs rebinding after these patches.

## Not verified

- No compile, test, Clippy or render of A, B or C. rustfmt `--check` and text mirrors only.
- No listening, narrated sync, fine sync or final AV. No motion between samples.
- Oliver times are estimates on take01, not on the take root finally selects.
- 13_9 to 13_11 lifecycle: by hand, not replayed. Their code differs from baseline only in positions, trail boxes and one caption.
- Teaching judgement beyond what was asked. The author's open items for root stand: L1, L3, L4, L8, L9, the 13_3 and 13_5 rack, the hinge line, the direction marker, N2. One addition: 13_7 says "groups of four quarter-turns cancel" while the picture combines `r⁻¹ r²`, with no group of four shown.

## Files

| Path | What |
|---|---|
| `patches/C-cross-review-corrections.patch` | X1 to X6 against candidate-AB, +32 −11 |
| `candidate-ABC/` | Full result; its `oracle.py` is byte-equal to candidate-AB's |
| `checks/replay.py`, `checks/group_and_boxes.py` | Independent checks |
| `checks/build_correction.py`, `checks/run_all.py` | Exact recipe; rerun from repo root with `/usr/bin/python3 -I -B` |
| `checks/outputs/` | Saved outputs. AB's single `group_and_boxes` failure is F5; ABC has 0 failures |
| `viewed-frames.tsv` | Images opened, with time, SHA-256 and what was seen |
| `base.sha256`, `bundle.sha256` | Inputs and this bundle, repo-relative |
