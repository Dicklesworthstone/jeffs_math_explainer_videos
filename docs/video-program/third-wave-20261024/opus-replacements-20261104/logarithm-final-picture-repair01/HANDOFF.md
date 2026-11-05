# Logarithm chapters 11 and 12: picture repair

From RubyRaven to root (BlueSalmon), thread `mx-video-program-program-87do.63`. Findings F1, F2, F3 and M1 are
from review `logarithm-final-review-pane169-01`.

**Status: source only.** Nothing here was compiled, run under Cargo, rendered or heard. `rustfmt --check` passed on
both candidates, which is a format check and not a build. I wrote this repair and my own review found the defects, so
this is not independent work. It is not publication acceptance.

## What to apply

`repair.patch` applies with `patch -p1` from the root of a copy of `logarithm-stage09-workspace01`. It touches two
files. A real `patch -p1` on scratch copies of the baselines gave both candidates byte for byte
(`checks/outputs/patch_apply.out`). The workspace itself was not changed.

| File | Baseline sha256 | Candidate sha256 |
|---|---|---|
| `unreachable.rs` | `c822e8b46796e79e2f8461872fe538de711a4fde755913ec57696710d0cc28a7` | `ba25565067fd68fb3f51f8eee3a1a381df05985e9ef5f35d3cb1d6bb40da0e53` |
| `what_it_forgets.rs` | `6951d3b2f157387683efaa3ec7c9d9d12a23c8448df247531cd5f0081bab1e28` | `5d04fefaef89cc7b2f2f98b006a54ca9181226904b6c9eb509466246dd649982` |

`draw.rs`, `facts.rs`, `script.tsv`, takes and audio are not touched. The patch calls `fade_in_all`, `swap`,
`matrix_card` and `position_c` as they stand. If Bronze's `draw.rs` work changes any of those signatures, this patch
needs a matching change. `explainer_kit::fade_all` and `Timed` already exist in the kit.

## For Lavender's rebind

Only each file's `sha256` changes in these map rows. Line, marker and interval hash stay as stored in
`narration-direction-map-oliver16.json`. Nothing above any of these lines moved.

| Hold | File | Say line | Hold line | Marker | `interval_sha256` |
|---|---|---|---|---|---|
| 11_3 | `unreachable.rs` | 963 | 1070 | `predict(&kit, stage, v(right + 1.6, -1.4), 3.0)?;` | `e34e65ad69a71865c956f2cf6e3d5270433370e8b6e79e5f8defc221555771db` |
| 11_20 | `unreachable.rs` | 1501 | 1590 | `kit.music_breath(stage, rule_read + row_reads.iter().sum::<f64>())?;` | `4a4e6c72d1f92e5e88e7430c03e0f0f062e3e7972184736bfc911a4313fae5e0` |
| 12_5 | `what_it_forgets.rs` | 1099 | 1123 | `predict(&kit, stage, v(2.6, -2.1), 3.4)?;` | `b238a92968284627ce972479bb1c5a93ac5f5df33efee0b17ddea5d362c8387f` |

Lines after 1619 in `unreachable.rs` move up 7, and the film's helpers and tests grow. No hold is bound below line 1590.

## What changes on screen

| Finding | Cue | Before | After |
|---|---|---|---|
| F1 | 11_1 | `the average ½(I + mirror) has det 0:` | no colon; new dim line under it, `(det = determinant, the signed area factor)`; `it squashes the plane flat onto a line` moves down to y −0.8 |
| F1 | 11_2 | `… det = exp(∫ trace dt) > 0 (Liouville)` | `… det (signed area) = exp(∫ trace dt) > 0, trace = diagonal sum (Liouville)` |
| F1 | 11_8 | `e^(tr X)` | `e^(trace X)` |
| F2 | 11_8 | `log A = diag(ln 2 + iπ, −ln 2 + iπ)` | `a complex logarithm: X = diag(ln 2 + iπ, −ln 2 + iπ),  e^X = A` |
| M1 | 11_5 | routes appear with no numbers | with the routes, in the gap left of the column: `here X is a new sample:`, then `X =` and a card of 0.4, 0.6, −0.5, −0.3; they fade over the same 2.6 s as the off-diagonals shrink |
| F3 | end of 12_21 | film stops 0.78 s after the last word, no fade | finished card held 5.5 s after that same frame, then everything fades over 1.5 s |

The card's four numbers come from `base = fact_matrix("matrix.probe_candidate")`, the value that already drives the
arrows and the live gap. None is typed. 11_4's `(here X is the sample flow from before)` and `matrix.flow` stay as they
were. The column keeps the general letters p, b, c, q.

## Timing consequences

Computed from source and the verification record, not measured.

| | Before | After |
|---|---|---|
| Chapter 11 | 16,550 frames | 16,550 frames, same fades and swaps |
| Chapter 12 | 9,837 frames, 163.95 s | 10,257 frames, 170.95 s |
| Hold | none | local frames 9,837 to 10,167 (163.95 to 169.45 s) |
| Fade | none | local frames 10,167 to 10,257 (169.45 to 170.95 s) |
| Film | 116,245 frames, 1937.42 s | 116,665 frames, 1944.42 s, before your 04_7 change |

Every earlier chapter 12 frame keeps its place. The hold begins on the same frame `finish` used, because the window's
own `hold_breath` waits out the same 0.6 s. `finish` then waits 0.

Consequences to carry through:

- **Chapter 12's music stem is refused until re-scored.** `Kit::finish` requires an exact sample count.
- **Chapter 12's score will change.** By reading `duck::gaps`, the closing resolution now falls in the 12_5 prediction
  hold, the last speech gap of at least 1.6 s. The 7.0 s tail moves it to the true end, which also changes that 12_5
  gap's phrase and may rotate the chapter's harmony. By my arithmetic a closing phrase fits, with about 7.65 s against
  about 7.23 s needed. That is reading, not a run. It needs listening.
- **Other stems may differ.** The film is scored as one render with whole-film filters, so `cmp` stems 00 to 11 after
  the new score. Your 04_7 change forces a new timeline and score anyway.
- **Master, sheets and fine sync all need redoing.** Captions are unchanged, because no line moved.
- Not produced: padding the old stem with silence. It would keep the music bytes but is a hand edit outside the score
  pipeline. I recommend re-scoring.

## Tests in the patch

New in `unreachable.rs`:

- `the_longer_formula_and_gloss_fit_on_real_glyphs`: the native width check you asked for. It builds the film's own
  `complex_log_tex` and `DET_GLOSS` at the film's size and place and measures real bounding boxes.
- `the_sample_x_prints_the_candidate_and_stands_in_the_gap`: sample bound to its fact; all three parts measured clear
  of the panel and column; word above card; name clear of card.

Extended or updated:

- `the_complex_logarithm_of_the_flip_pair_exponentiates_back` now also bars `\log A` from the film.
- `the_footers_are_in_ink_and_clear_of_the_edge_and_the_film` already measures real glyphs, now on the longer footer.
- `the_panels_and_text_column_fit` takes the new strings.

New in `what_it_forgets.rs`: `the_closing_card_is_held_after_the_last_line_and_then_fades`.

None of these has run. `checks/check_repair.py` only mirrors their string and order assertions, plus width estimates
from the crate's em table.

## Source checks that did run

`checks/outputs/check_repair.out`: 42 checks, 0 failed, including 8 negative controls that must fail on mutated input.
They show that baselines equal the workspace; the patch rebuilds both candidates; all three holds reproduce their stored
intervals on the baseline and keep them on the candidate; cue order is unchanged; only the listed film lines changed;
the fade 0.5, swap 0.4 and tween 2.6 keep their run times; sample entries equal `facts.rs` and `oracle.py`.

Run it from the bundle with `--ws` and `--verification`. My first full run failed one row because of an off-by-one in
the checker's own slice, not in the patch. I fixed the checker and reran.

## Order for integration

1. Apply the patch to a copy alongside the other repairs.
2. `cargo fmt --check`, Clippy, `cargo test`, facts-check. A compile failure here is expected to be possible: no line of
   this has met a compiler.
3. Read the glyph probe for `\operatorname{trace}`, `\text{a complex logarithm: }`, `\quad` and the new strings.
4. Timeline-only, score, `cmp` stems, render 11 and 12, lint 0.
5. Read sheets at each row in "Open until rendered", then Lavender's map, then the master.

## Open until rendered

- **Formula width at 11_8.** The em table says about 8.9 wide against 13.7, but it ignores math spacing and `\quad`.
  If the test fails, shorten the words first. Do not shrink the type below the row's 28 pt neighbour.
- **Footer width at 11_2.** Estimated 11.76 of 13.6, one line at 22 pt. Character share gives it about 18 s on screen
  against an 8.3 s reading floor at 15 characters a second. Both are estimates.
- **Gloss at 11_1.** It arrives at sentence 4 of 5 and stands to the start of 11_2. That time is unmeasured.
- **Sample at 11_5.** Its reading time runs from "send it along both routes" to sentence 4 and is unmeasured. Motion is unseen, in
  particular whether a 2.6 s fade reads as "these entries stop being true" and not as a mistake. Dim 18 pt matches
  11_4's sample line, but contrast in this spot is unmeasured.
- **The 7 s ending.** Whether 5.5 + 1.5 s feels right is a judgement I made from source, and it will not be known
  until it is watched with its music. The two constants sit together for easy change, but any change re-scores.
- **Fade to background.** `fade_all` ends on the plain background, and the lint should treat that as empty. Confirm at
  lint.

Storyboard docs are not patched. Preserved predecessors: `before-unreachable.rs`, `before-what_it_forgets.rs`.
