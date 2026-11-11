# counting root repair02: CC0 independent source review (review01)

CC0 (LavenderPike), 2026-11-10T23:15Z. This review covers root's files in final-review995/counting-root-repair02:

| File | sha256 |
|---|---|
| counting.rs | `21c5043a3339cfadb10024a2e8227a62a0b06674139eace1a0d9029a4e1aa0db` |
| candidate.patch | `39fe90eaef2cc609f8f4215959575dc285cde7c0b2876206fb1d4dc16b5dc7cd` |
| handoff.json | `9ef6f99dad20586ad896069e4dbeb4c1151e1ca04e8bc7b7398f70e570d1e5ca` |

**Scope of the work:**
- Only source reads and cheap local checks were made: no build, Rust test, Clippy, render or SSH.
- Nothing was written outside this review folder.
- Root's queued build02 files were only read, not run or duplicated.
- No audio heard or checked. The Frederick Surrey narration is unchanged and was not assessed.
- This is not native, picture or AV acceptance.

## Verdict: no blocker; bounded source acceptance

21c5043a is fit for root's queued guarded build02 as staged. Two Low items follow; no change is asked.

## What was checked

**Lineage.**
- `patch -p1` of candidate.patch on ecff48f7, the 760 counting.rs, gives exactly 21c5043a.
- Against CC3's 5cf11416, only root's five hunks differ. CC3's caption, its constants and its two tests are byte-identical.
- build02's csd-counting02-counting.rs is 21c5043a.

**The exact Möbius-circle walker (11_3).**

*Formula.* model.rs has f(z) = (z − 0.45)/(z − 1.35). The image of |z| = r is the circle through f(r) and f(−r), so:
- centre (r² − ab)/(r² − b²);
- radius r·|b − a|/|r² − b²|.

`moebius_image_circle` is that formula. My prototype samples 721 points directly and agrees to 1e-12; the sampled points sit on the circle within 1.3e-15.

*Scale.* gain = OUT_R/(|c| + ρ) puts the farthest point exactly at 1.35. A positive real scale keeps every direction, zero, and every turn. The live counter still reads `turns_so_far` on the raw f, so the count shown equals the turns drawn.

*Drawn values* (the handoff's `numeric_check` values reproduce; they are these drawn, scaled values):

| r | gain | drawn centre, radius | zero | turns | on screen |
|---|---|---|---|---|---|
| 0.8 | 2.1214 | −0.0583, 1.2917 | enclosed | 1 | x 1.000–5.306, y −2.503–1.803 |
| 1.7 | 0.3780 | 0.8082, 0.5418 | missed by 0.2665 (0.444 scene, about 120 px at 4K) | 0 | x 3.694–5.500, y −1.253–0.553 |

- Both images lie inside the output frame, x 0.75–5.75 and y −2.85–2.15.
- The r = 1.7 image stays below the 11_3 inset card, whose bottom edge is at y 1.275.
- Its swing is asin(1.4333/2.1382) = 42.1°, which is the storyboard's "a circle off zero, swinging about ±42°".

*Code.*
- `moebius_circle_walker` is `draw::walker` line for line. The only changes are that `squash(w, s, OUT_R)` becomes `w.scale(gain)` and the centre is fixed at zero.
- The updater, the styles and the mob order `[trail, arrow, in_dot, out_dot]` are the same.
- Walker's fields are `pub`, and every name used is already imported.
- The dispatch in `walk_on` reaches only 11_3's two calls, at radii 0.8 and 1.7. These keep the squash walker:
  - the 11_1 walks (Identity, Square, Recip);
  - the 11_2 walk (ZeroDoublePole);
  - the 11_6 quintic walk.

  The 11_4 pole approach uses its own code.
- The `unwrap()` at z0 = r could fail only at r = 1.35, which no call reaches.

*New test.* `the_moebius_images_stay_circles_with_their_true_winding`'s assertions hold in my prototype. Its closest margin is (|c| − ρ)·gain = 0.26646 against 0.26.

**The early compression note (11_6).**
- *Text and placement.* The note is "output size compressed", the first clause of SQUASHED. Its glyphs are a subset of the registered SQUASHED. It is set at 18 pt in dim at (3.25, 2.55); 2.55 is `ROW_OVER_OUTPUT`, the row between the frame top (2.15) and the counter row (3.15). At 1.82 units wide it spans x 2.34–4.16.
- *Nothing else in its row.* The ASK card fades out at 11_6's start, before the note arrives. The −3z label sits at (4.83, 1.40), inside the window. Nothing else occupies that row in 11_6.
- *It is true while shown.* The quintic walk and the reference curve are drawn with `squash`. The note goes (`remove_from_scene`) at the moment the full SQUASHED card arrives.
- *The reading-window test still holds.* mod.rs `each_card_outlasts_its_reading_window` holds for 11_6:
  - the removal comes before `let squashed = `;
  - the span from that card to its window contains no `fade_out` or `remove_from_scene`;
  - `wait_until(stage, line.end())` still comes before the card.

**Timing.** The source says timing-neutral:
- no play, wait or pause is added;
- the one changed fade keeps its 0.4 s;
- the walker's show and walk are unchanged;
- `kit.text` and `remove_from_scene` take no time.

Native timelines are what confirm it.

**CC3's half (the input-loop caption).**
- *Methods exist.* `Frame2::left()` and `right()` exist (explainer_kit plot.rs), so the output frame spans x 0.75–5.75. `carded` draws its card as the text width + 0.5.
- *Card margins.* The new first card is 4.780 wide, leaving 0.110 on each side against the test's 0.08. The other two cards leave 0.427 and 0.359.
- *Width model.* The model gives 3.651 for the old card, against 991/270 = 3.670 measured, a difference of 0.019 within the test's 0.03.
- *Winding.* The input loop |z| = 1.6 winds +1 around both the zero at 1 and the double pole at −1. The output winds −1, which is the fact `argument.several.turns`.
- *Wording.* 11_2's narration says "a loop that winds once counterclockwise"; the card names that loop as the input loop, with the same meaning.

**Gates in source.**
- *rustfmt.* rustfmt 1.10.0-nightly (9085017724 2026-08-30), with the 760 rustfmt.toml and edition 2024: `--check` exits 0 on both the candidate and the base.
- *Fade gate.* draw.rs's fade gate passes: counting's film code has no `fade_in(`, `fade_out(` or `fade_all(` line.
- *Self-quoting tests.* The other counting.rs tests that quote their own source are untouched: the Quintic count of 3, the 11_7 lines and the coarse loop. The new tests add no `Some(Func::Quintic)`.
- *Supplements.* The new strings are unique, so there is no supplement duplicate.
- *Test count.* There are 3 new tests, so 247 + 3 = 250. build02 sets `TESTS_PASSED = 250` and names all three.
- *Clippy.* The walker's patterns (`t == last`, `let _ = s.set_points(…)`) are `draw::walker`'s own, which passed Clippy `-D warnings` in build844. The rest is for the native run.

## Low items (no change asked)

- **L1, relative size in 11_3.** Each walk has its own gain. So the r = 1.7 image, truly 2.35× the r = 0.8 image (radius 1.433 vs 0.609), is drawn at 0.42× its size (0.542 vs 1.292), and no note in 11_3 says so.
  - The storyboard's "rescaled to fit" allows this, and nothing spoken compares sizes. Judge it at pictures02.
  - If root prefers one scale for both walks, a gain of 0.378 draws the r = 0.8 image with radius 0.230 about −0.010, about 104 px at 4K.
- **L2, the note leaves in one frame.** It is removed with `remove_from_scene` as the full card comes. This keeps the timing exact and satisfies the reading-window test.

## Limits

- No Rust compile, test or Clippy run was made here; the native run decides those.
- No pixels were viewed. All positions come from source arithmetic: cmr10 widths, scene units, and 270 px a unit at 4K.
- The 11_1 and 11_2 walks and the 11_4 approach keep the squash. That is pre-existing and outside this change.

Outputs in this folder:
- numerics01.py / numerics01.out (the independent prototype);
- apply/counting.rs (the patch round trip);
- fmt/ (the rustfmt inputs);
- inputs/ (byte-exact copies of root's three files).
