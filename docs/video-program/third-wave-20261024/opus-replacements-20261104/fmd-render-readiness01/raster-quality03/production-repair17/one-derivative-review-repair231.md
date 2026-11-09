# one_derivative review repair 231: chapter 06, F1 and F2 of review224

Dispatch-0 (cc0), 2026-11-08T04:45Z. **Candidate source for root to integrate. Nothing has been applied, compiled, rendered or synthesised.** I wrote only this note and `one-derivative-review-repair231.rs`. Root owns integration, the build, the render, verification, TODO and Beads. I make no acceptance claim.

## Source

- **Base.** `production-source-preserved214/workspace/explainer_corpus/complex_analysis_visual/src/chapters/one_derivative.rs`, sha `4f0154ec82bb787cdbee4d11d308eea25ca6cc1164f7f4bdb1d908f8d54df29c`. This is the production19 file named in that folder's `receipt.json`. I re-hashed it after writing the candidate, and it is unchanged.
- **Candidate.** `one-derivative-review-repair231.rs`, sha `fee77db8ab0c1d76300dbc49138816f94a43f26a590cf1e0bd0f541710470a27`. It is a copy of the base with the changes below: 152 lines added and 8 removed.
- **Canonical differs.** The repository's `explainer_corpus/complex_analysis_visual/src/chapters/one_derivative.rs` has sha `b67231d9…` (last commit `2fb1c0c7`), not `4f0154ec…`. The canonical `chapters/mod.rs` (`57c46d67…`) also differs from production19's (`4f4b17c3…`). The candidate belongs on the production19 workspace; carrying it to canonical is root's decision.
- **Findings.** F1 and F2 of `cc0-final06-picture-review224.md` (sha `900a00fb…`). F3 (the heavy minus and times glyphs) is not touched.

## Changes

| Where (candidate line) | Change | Why |
|---|---|---|
| `planes()` (92–96) | The input window goes from (−0.2, 2.2)² to (−0.6, 2.6)². Both pictures are now 3.2 units across 4.6 scene units, 1.4375 scene units per unit. The output plane is unchanged. | **F1.** The input was drawn at 1.917 per unit and the output at 1.4375, so the drawn stretch was 0.75 × 2√2 = 2.12. The label says "stretch 2.83" and the voice "about two point eight". Now the drawn stretch is exactly 2√2. |
| `SAME_SCALE`, `scale_note` (62, 80, 272) | "both pictures drawn to the same scale", 18 pt dim, centred at (−0.6, −3.3) under the two pictures. It joins the frames' existing 0.4 s fade in 06_1. | **F1.** Makes the scale explicit from the first geometry. |
| `hand_over`, 06_4 zoom (196, 543) | `fade_in_all(stage, &zoomed, 0.3)` becomes `hand_over(stage, &[scale_note], &zoomed, 0.3)`. The note fades out while the zoom note fades in, in the same 0.3 s. `hand_over` is a local copy of the `draw::fade_in_all` / `draw::fade_out_all` pattern in one `play_prepared`. `draw.rs` is not changed. | From the zoom on, the right picture is no longer at the shared scale. The existing zoom note says so. |
| `UNLESS_ZERO`, `unless_note` (476) | "unless the derivative is 0", 18 pt dim, left-aligned at (0.25, 1.45) in the right picture's top-left corner. It joins `named`'s existing 0.3 s fade at 06_3 sentence 3, stays through 06_4, and leaves in 06_5's existing fade. | **F2.** Qualifies "multiplying by a complex number is a stretch and a turn" and "a tiny square … lands as a tiny square". |
| `BIGGER_HERE`, `bigger_note` (512) | "bigger here: stretch " plus `fact("deriv.scale")`, which reads "bigger here: stretch 2.83", at (0.25, 1.15). It joins the image's existing 0.5 s fade at 06_3 sentence 4 and leaves in 06_5's fade. | **F2.** The voice's "bigger" belongs to this example. The number comes from the same fact as 06_4's readout. |
| 06_7, `matrix_unless_note` (656, 686) | "unless the derivative is 0", 18 pt dim, centred at (0.2, −3.1) under the caption at (0.2, −2.7). It joins the caption's existing 0.4 s fade and leaves in 06_8's existing fade. | **F2.** Qualifies "the local map must be a stretch and a turn". |
| Tests (1087, 1096, 1122) | Three pure tests: `both_pictures_share_one_scale`, `the_drawn_stretch_is_the_derivatives_size` and `the_review224_notes_ride_existing_fades`. | See below. |

The script, narration, cues, timeline, facts, oracle and audio are unchanged. The Frederick takes stay bound as they are; no audio was heard or checked. The new labels add the condition the voice leaves out. This agrees with the later script: 07_1 and 07_2 treat critical points, and 15_2 says "away from critical points".

## Timing

The chapter keeps the same 95 time-bearing calls in the same order: fades, shifts, plays, sentence and phrase waits, holds and the predict. The one difference is `fade_in_all(… zoomed …)` becoming `hand_over(…)`, with the same 0.3 s. All 52 fade run times are equal. Each new label is created without play time and rides an existing fade in and an existing fade out. The scale note is faded out once only. The existing test notes that a second fade brings a faded mob back for a frame, so the note is left out of 06_5's list. I checked all of this with a Python mirror of the source, not by running the scene.

If integration keeps timing, the re-rendered 06 should still have 10,030 frames. Root can use that as a check.

## Layout arithmetic (scene units; not measured on a render)

- **Input picture at 0.75 of its old size.** The square of side 0.5 becomes 0.72; the 06_3 fan arrows are 0.43 long, down from 0.575; h becomes 0.65. The input labels are placed in plane coordinates and move with it. I found no new contact: the "z0 = 1 + i" label's bottom (about 0.11) stays above the square's top (−0.24) and above the 06_2 approach paths.
- **Top-left notes.** At 18 pt, CMR10 widths times 0.18 give 1.95 for "unless the derivative is 0" and 1.90 for "bigger here: stretch 2.83". Starting at x = 0.25, they end by x ≈ 2.20. The output's imaginary axis is at x = 2.4, the image's top at (2.4, 1.02), the legend starts at x = 2.75, and the panel's top edge is at y = 1.70. The 06_3 output fan tips stay below y = 0.51.
- **Scale note.** About 3.0 wide, so it spans x −2.1 to 0.9 at y −3.3, below both panels (whose bottoms are at −2.9). 06_4's gap line at (2.4, −3.35) shares that height, but it arrives after the hand-over has removed the note.
- **06_7 note.** It sits under the caption, and nothing else is drawn below it.

## Tests

- **`both_pictures_share_one_scale`.** Equal `unit()` and equal frame width for both planes.
- **`the_drawn_stretch_is_the_derivatives_size`.** On screen, the sides of the square's linear image and all four fan arrows come out |f′(z₀)| = 2√2 times their input lengths, to 1e−9. `fact("deriv.scale")` prints that size as "2.83".
- **`the_review224_notes_ride_existing_fades`.** Read from the source, in the style of the existing `the_reflections_numbers_leave_before_doublings_heading`:
  - each note is made once, faded in once and faded out once, inside the cue it belongs to;
  - the scale note is in the 06_1 frames' fade and in the only `hand_over`, and not in 06_5's list;
  - no bare zoom fade remains.

Existing tests that cover the change and should still pass: `the_windows_hold_the_pictures` (the larger input window holds the square), `the_fan_ends_as_the_derivative_times_each_step`, and `the_zoomed_right_plane_starts_true_and_closes_on_the_teal_square`.

## Checks run

- `rustfmt --check --edition 2024` with the repository `rustfmt.toml` (max_width 120) passes on the candidate. The base also passes.
- A Python mirror of `the_review224_notes_ride_existing_fades`, the timed-call comparison and the scale arithmetic all pass. The old drawn stretch is 2.1213 (review224 measured 2.11); the new one is 2.8284.
- **Not run:** the Rust build, Clippy, the tests, the glyph probe or any render. All Rust is uncompiled.

## For root

1. Put the candidate in place of production19's `chapters/one_derivative.rs` (base `4f0154ec…`).
2. Add `one_derivative::SAME_SCALE`, `one_derivative::UNLESS_ZERO` and `one_derivative::BIGGER_HERE` to the `TEXT` glyph supplements in production19's `chapters/mod.rs`, beside the other `one_derivative` entries. They are plain letters, digits, a colon and a full stop.
3. Run the guarded build: fmt, Clippy and tests (including the three new ones and the three existing ones above), facts-check, and the glyph probe for the new strings.
4. Render 06 under guard and check:
   - the 06_3 input fan at its smaller size;
   - the three dim notes' legibility and clearances;
   - the scale note leaving exactly at the zoom;
   - the frame count.

   If the input fan's heads crowd the short arrows, `FAN_LEN` 0.3 → 0.4 restores the old on-screen input length and keeps the true ratio. I have not applied it. With it, the output arrows are 1.13 units, inside the window, so `the_fan_ends_…` still holds, and the up-right tip lands near (3.06, 0.89), clear of `named` and the notes.
5. If the storyboard or description lists 06's on-screen strings, add the three notes there.
6. Have an independent picture review of the new 06 render. This candidate does not establish visual, audio, fine-sync or final AV acceptance.
