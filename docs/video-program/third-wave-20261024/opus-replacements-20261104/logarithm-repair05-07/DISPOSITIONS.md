# Logarithm 05–07: defect dispositions

BronzeJaguar, Agent Mail 1484 under `mx-video-program-program-87do.63`, 2026-11-04. Findings are the IDs in `logarithm/preview02-independent-review-05-07/REVIEW.md` (sha256 `c7ae8802126f7352…`, unchanged).

**Status: uncompiled, tests not run, nothing rendered, nothing heard.** No Cargo, render, synthesis, Beads change, commit or deletion. No canonical or shared file was edited. Root owns the guarded integration, build and picture review. Oliver listening, fine sync and the final independent AV review remain open.

## Result

- Implemented 7 of 7 high, 17 of 17 medium and 10 of the 19 low findings. Two more low findings are part done, and one (D12) is a separate patch for root's decision. The other six are left, with reasons below.
- Narration preserved: `script.tsv` is byte-identical, and every `say`, `until_sentence` and `hold` anchor is unchanged and in order. All five `predict` calls keep their count and order. No cue words or Oliver takes change.
- 21 new `#[test]`s, a net +20. They build the real text through `Kit` and read real boxes through `fmn::run_scene`; there is no stand-in framework.

## Apply

From the repo root, against the hashes in the next table:

```
patch -p1 -i docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-repair05-07/patches/logarithm-repair05-07.patch
```

`patches/draw-arrow-solid-head.patch` is separate and optional (see D12). `SHA256SUMS` has every file's hash.

| Baseline file, under `explainer_corpus/logarithm_mysteries/` | sha256 prefix |
|---|---|
| `src/chapters/complex_turns.rs` | `d1f2d72555ee7f10` |
| `src/chapters/lifting_loops.rs` | `b409ace64e13fafb` |
| `src/chapters/branch_cut.rs` | `c3fd774e9172cdbe` |
| `src/chapters/draw.rs` (helper patch only) | `59ebd09d2fa5c793` |
| `script.tsv` (no change) | `3e4e34af7090e448` |

## High

| ID | Disposition | Test |
|---|---|---|
| F-zero | `grid()` now draws the zero ring and the cut itself, at strength `plane_marks(m)`, which is 0 from `m = 0.08`. They cannot stand on the strip, whatever the fade lists do. This replaces the review's fade-list repair. On the strip, a note says "z → 0: off to the left" | `the_removed_zero_and_the_cut_stand_on_the_plane_and_never_on_the_strip` |
| E7 | The short arrow is gone. A shifted inset has its own axis marked πi, 0, −πi, at the tall axis's scale, with a filled red arrow from πi to −πi. That is a true 2π, and the bottom stays on screen at y −2.49 | `the_candidates_jump_is_drawn_two_pi_long_at_the_tall_axis_scale` |
| E1 | The bracket and "one lap" move left of the tick labels | `the_lap_word_stands_outside_a_bracket_that_clears_every_tick_label` |
| E10 | The theorem moves to `v(1.2, -3.55)`, under the formula and clear of the tall axis | `the_theorem_stands_clear_of_the_tall_axis_and_the_formula_above_it` |
| F-edge | Cards become "just above → top edge" and "just below → bottom edge". The width guard that passed wrongly is replaced with real boxes | `the_labels_beside_the_strip_end_before_the_cards_in_their_rows` |
| D2 | "2i" moves left of the axis | `the_names_of_i_and_two_i_stand_left_of_the_axis_clear_of_zw` |
| F-bp | "branch point" moves to `text_left` at `v(right, -3.05)` and now fades at 07_6 | `the_branch_point_word_clears_the_spiral_and_the_card_above_it` |

## Medium

| ID | Disposition | Test |
|---|---|---|
| D3 | "i" moves left of the axis | names test above |
| D4 | `closes` and `dial` are `text_left`, just right of the output axis | `the_lines_over_the_output_plane_clear_its_axis_and_outer_ring` |
| D5 | "0: no logarithm" moves outside the rings, with a thin red leader 10° off the nearest ray (predicted) | `the_no_logarithm_word_clears_every_ring_and_its_leader_misses_the_rays` |
| D6 | `g_word` moves above the outer ring, left of the axis | lines test above |
| D1, D9 | Bracket words start past the gridlines; the "0" label stands above its level link | `the_strip_labels_clear_its_lines_and_the_link_to_one` |
| D7, D8 | Laps at `+2.35` and `+3.15`. Aid at `v(OUT.centre[0] - 0.4, -3.5)`, further left than the review's `-0.1`, so it also clears "output" | `the_lap_count_and_the_aid_clear_the_rays_the_axis_and_the_output_word` |
| D10 | `target` is added to the scene again after each stroke through it | `a_dot_added_again_after_the_strokes_through_it_is_drawn_over_them` |
| E5 | The gold dot swaps out as the side circle arrives and returns for 06_6 | none (timing of a fade) |
| E6 | "0 is not inside" starts right of the number's axis | `the_side_circle_note_clears_the_axis_and_both_circles` |
| E11 | The walk tweens back over 0.6 s instead of jumping in one frame | none (motion) |
| F-sides | Words move to `Z.p(C::new(-2.95, ±0.45))`, further left than the review's `-2.8`, for a clear gap from the outer circle | `the_words_for_the_two_sides_clear_every_circle_and_the_marked_lines` |
| F-nocut | Same mechanism as F-zero. While the sides are marked, the cut thins from 6 to 2.5 so gold and purple stay visible on either side of it | zero test above |
| F-pop | The grid fades in through `grid_alpha`; the sides fade in through `sides`, which also removes a second one-frame pop the review did not list | none (motion) |
| S1 | `removed`, `log`, `deriv` and `where_` now leave at the start of 07_5 | none (timing of a fade) |
| V1 | At "That is the strip…", `closing_strip` crossfades in: grid, solid gold top edge, dashed bottom edge, "top edge: included", "bottom edge: left out" | `the_closing_strip_shows_the_two_edges_two_pi_apart` |

## Low

| ID | Disposition |
|---|---|
| F-blocked | Done, moved to `Z.p(C::new(-1.3, 0.75))`. Test: `the_blocked_word_clears_the_unit_circle` |
| F-fine | Done. The fine print now arrives right after `example`, an estimated 10–11 s before the end on scratch timing rather than 4.7 s. Recheck on Oliver |
| F-gap | Done. Test: `the_live_angle_starts_just_past_the_words_that_name_it` |
| F-stale | Done. The dot, words and live angle fade at 07_6 sentence 3 |
| F-leap | Done. The leap's ends stand on the marked lines. Test: `the_leap_runs_between_the_two_marked_lines` |
| D11 | Done, **your decision.** The blue x = 0 column leaves through the 05_5 hold and returns with 05_6's first sentence. To reverse it, drop `v_line` from both lists |
| D12 | **Separate patch, your decision; see below** |
| D13 | Done. The rule now ends ", k whole". Test: `the_typeset_cards_fit_their_places` |
| D14 | Part done. The two cards with carets are now TeX, and so is "y sets the angle", to match them. The title-alone beat at the start of 05_3 is left: it is about 1 s of scratch timing |
| E3 | Done, "0" label on the base tick |
| E8 | Done. Trails under `MIN_TRAIL` are not drawn. Test: `a_trail_too_short_to_draw_is_shorter_than_its_own_stroke` |
| E9 | Done. "its logarithm" returns near the axis top from 06_6. Test: `the_tall_axis_keeps_its_name_clear_of_labels_and_panels` |
| S2 | Part done. The misleading comment is corrected. The picture is left, because the card carries "to infinity" and a change would need a design choice |
| F-theta | Left. The text font's θ is a legitimate variant, and typesetting one card would mix fonts with the card above it |
| E2, E12, E14, Layout | Left. No overlap or wrong statement, and fixing them means wider restyling |
| E15 | Left, **your decision**: the 06_2 hold shows 1.86 and a dot near the 2πi tick. Changing it alters the prediction's picture |

The review's pacing notes are also left, because they are on scratch timing.

## D12: the shared arrow helper

**Cause.** At this pin `Arrow` is one closed polygon, so `line_style` (fill opacity 0) only outlines it and the head looks hollow.

**Patch.** `patches/draw-arrow-solid-head.patch` changes only the body of `draw::arrow`, to `arrow_parts(stage.arena_mut(), a, b, c, w, 0.22)`. The signature is the same and the chapter patch does not depend on it. It affects every remaining `arrow` call in the film.

**Checked.** Every canonical caller only fades or lists the handle, which now names a group. The patch applies with no fuzz to canonical `draw.rs` and to `logarithm-repair08-10/candidate`'s `draw.rs` (sha256 `82c7d086462eafd3`), which edits a different part of the file. WhiteSwan, RubyRaven and LavenderPike were told in Mail 1531.

## Added scene time

The voice does not move within a line. `until_sentence` does nothing once its moment has passed, so each addition first uses up the wait after it. Only a cue whose pictures outlast its voice would delay the next cue, as added silence.

| Where | Change |
|---|---|
| 07_3, before sentence 1 | +0.5 s |
| 07_3, sentence 4 | +0.4 s |
| 07_5, start | −0.4 s (a swap replaces two steps) |
| 07_6, sentence 3 | +0.3 s |
| 06_6, sentence 5 | +0.6 s |
| 05 | none |

**New card text, for the glyph probe:** "z → 0: off to the left", "top edge: included", "bottom edge: left out", the shortened edge cards, a "0" tick, and three new TeX cards plus the extended rule in 05.

**Lines moved.** If a binder hashes these sources, it must rebind. `predict` is now at `complex_turns.rs:235` and `:588`, `lifting_loops.rs:358` and `:458`, and `branch_cut.rs:419`.

## Evidence

Output of these is in `checks/`, or quoted here.

| Check | Result | What it does not show |
|---|---|---|
| `rustfmt --check` on a scratch copy of the whole crate, both patches applied | No parse errors. Only `matrix_logs.rs` and `score.rs` have diffs, and neither is in these patches | That it compiles |
| `patch -p1` on a scratch copy of canonical | Exit 0, no fuzz, equals `candidate/` byte for byte, other chapters unchanged; reverses to canonical | |
| `git apply --check`, both patches, working tree | Exit 0 | |
| `checks/invariants.py` | PASS: script hash, anchors, predicts, no home paths | How anything looks |
| `checks/predicted_margins.py` | 83 inequalities, none predicted to fail, tightest +0.008. Its width rule is within 3% of 11 measured widths. TeX boxes are rougher | **A prediction, not a test run** |
| `docs/check_cues.py` on a scratch copy | Fails, with a failure list **identical** to canonical's: 14 lines, including older chapter files on disk such as `bookends.rs` that say the same cues. Not caused here; worth your look | |

## For root's picture review

- **07:** the ring and cut leaving as the morph starts and returning at the end of the roll-back; the crossfade to the closing strip; the note beside the strip.
- **06:** whether the inset reads as the same axis's scale rather than a second plane. It is new design and nobody has seen it.
- **05:** the leader across the rings; the TeX cards beside plain text cards, whose sizes may not match.

## Stale bindings seen, untouched

- `narration_units.json` and `docs/narration-direction-map-oliver11.json` bind script hash `15eb9ae9…`, not the current `3e4e34af…`.
- The map's one `branch_cut.rs` row binds source hash `55fcd1ac…`, line 52.
- `docs/storyboard.md` is not contradicted, though it could mention the closing strip, and its "text widths are estimated" risk note no longer holds for these three chapters.

## Files

`baseline/`, `candidate/`, `patches/`, `checks/`, `SHA256SUMS`, this file.
