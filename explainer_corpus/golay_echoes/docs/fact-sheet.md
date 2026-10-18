# Fact Sheet — Two Bad Echoes Can Make One Perfect Measurement

Every number shown or spoken. Each row's value is computed in `src/facts.rs`
(from `src/model.rs`) and recomputed independently by `oracle.py`; `./x
golay_echoes facts-check` compares the two outputs key by key. Pinned rows
also carry an independently verified literal. Scores are listed from shift
−3 to 3 unless a first shift is given.

| id | quantity | shown / said | computed in | oracle key | value | source |
|---|---|---|---|---|---|---|
| F1 | pulses per pattern | "four pulses" (01_1) | `CODE_A.len()` | `G-F1.pulses` | `4` | pinned |
| F1 | pattern A | "Up, up, up, down" (01_1) | `model::CODE_A` | `G-F1.code_a` | `1,1,1,-1` | pinned: Wikipedia, *Complementary sequences* |
| F1 | pattern B | "Up, up, down, up" (01_7) | `model::CODE_B` | `G-F1.code_b` | `1,1,-1,1` | pinned: same |
| F1 | A and B are complementary | the whole chapter | `model::is_complementary` | `G-F1.complementary` | `true` | pinned: Chi, Pezeshki and Calderbank, definition 1 |
| F1 | the key's products | "+1", "−1" on screen; "add a block", "take one away" (01_3) | `facts::key_pairs` | `G-F1.key.agree`, `G-F1.key.disagree` | `1`, `-1` | computed |
| F2 | A's score row | the first row of blocks (01_4 to 01_6) | `model::autocorrelation(CODE_A)` | `G-F2.a.scores` | `-1,0,1,4,1,0,-1` | computed |
| F2 | A's score at its echo | "all four agree", "A score of four" (01_4) | same, shift 0 | `G-F2.a.peak` | `4` | pinned |
| F2 | the products one step off | three blocks, two green and one red (01_5) | `model::products(A, A, 1)` | `G-F2.a.one_off.products` | `1,1,-1` | computed |
| F2 | pairs, agreements, disagreements, score one step off | "three pairs line up. Two agree, one disagrees. A score of one" (01_5) | same | `G-F2.a.one_off.pairs`, `.agree`, `.disagree`, `.score` | `3`, `2`, `1`, `1` | computed |
| F2 | ghost size against the echo | "a quarter the echo's size" (01_6) | largest ghost ÷ peak | `G-F2.ghost_fraction` | `1/4` | computed |
| F2 | where A's ghosts are | "ghost" labels (01_6) | `Scores::ghost_shifts` | `G-F2.a.ghost_shifts` | `-3,-1,1,3` | computed |
| F2 | A's far-shift score | the far columns flash (01_7) | `model::far_shift_score` | `G-F2.a.far_shift_score` | `-1` | computed |
| F2 | B's score row, ghosts and their size | the second row (01_8); "the same size, at the same shifts" | `model::autocorrelation(CODE_B)` | `G-F2.b.scores`, `G-F2.b.ghost_shifts`, `G-F2.b.largest_ghost` | `1,0,-1,4,-1,0,1`; `-3,-1,1,3`; `1` | computed |
| F3 | the two rows added | the merged row (01_9) | `model::two_measurements(A, still, B, still)` | `G-F3.sum.scores` | `0,0,0,8,0,0,0` | pinned: Wikipedia's sum (8, 0, 0, 0) at lags 0 to 3, mirrored |
| F3 | the added echo, the largest ghost left | "8"; "not a single ghost" (01_9) | same | `G-F3.sum.peak`, `G-F3.sum.largest_ghost` | `8`, `0` | pinned: definition 1, 2N δ(k) with N = 4 |
| F3 | added echo against one row's | "twice as tall" (01_9) | 8 ÷ 4 | `G-F3.peak_ratio` | `2` | computed |
| F4 | the echo moved between the sends | "one step" (01_10) | `facts::echo_step` | `G-F4.moved.step` | `1` | computed |
| F4 | the two rows added, the echo moved | the last row (01_10), from shift −3 | `model::two_measurements(A, still, B, moved)` | `G-F4.moved.first_shift`, `G-F4.moved.scores` | `-3`; `-1,1,1,3,5,-1,-1,1` | computed |
| F4 | ghosts that return, and where | "ghosts return"; the flashing columns; "3" and "5" over the split echo | same | `G-F4.moved.largest_ghost`, `G-F4.moved.ghost_shifts` | `1`; `-3,-2,-1,2,3,4` | computed |
| X1 | far-shift score sizes, every pattern of 2 to 10 pulses | "With more than one pulse, every pattern leaves ghosts" (01_7) and its lower-third | `model::far_shift_score` over all 2,046 patterns | `G-X1.far_shift_sizes` | `1` | computed (edge test) |

## Claims (non-numeric)

| id | claim as narrated or shown | source | verified how |
|---|---|---|---|
| C1 | Sliding a copy along the recording and adding the lined-up products finds the echo (the matched filter) | Chi, Pezeshki and Calderbank eqs. 5–6 (the zero-Doppler ambiguity function is the autocorrelation); Purdue notes (matched filters) | `model::correlate`; `model::tests::the_matched_filter_is_convolution_with_the_reversed_code` |
| C2 | Every pattern of more than one up/down pulse leaves a ghost; one pulse has no other shift | the far shift overlaps only the two end pulses, so it scores ±1 | `every_code_longer_than_one_pulse_has_a_ghost` (lengths 2 to 10, all codes); `G-X1` |
| C3 | B's ghosts are A's with the other sign, at every shift | definition 1 (the sum is zero off shift 0) | `a_pairs_ghosts_are_always_opposite`: every complementary pair of 1 to 8 pulses (4, 8, 0, 32, 0, 0, 0, 192 ordered pairs) |
| C4 | Added, the two measurements of the same scene leave the scene itself, 2N times, with no ghosts (exact version: same still scene, no noise) | definition 1 and the chapter's eq. 9 | `the_same_scene_twice_returns_the_scene_magnified`: every pair up to 8 pulses × all 125 scenes of three steps with strengths −2 to 2; oracle repeats it independently |
| C5 | The guarantee needs the same scene for both sends; a one-step move brings ghosts back | the chapter's eqs. 7–8 and section 2.2 (Doppler between the two transmissions spoils the sum); Purdue notes ("where there is not significant motion between measurements") | `a_changed_scene_leaves_ghosts` (moved, upside down, stronger) and the identity A·sa + B·sb = A·(sa − sb) + (A + B)·sb; the chapter claims the assumption is needed for the guarantee, not that every change breaks it |
| C6 | Noise picked up by the receiver is scored too, and is not guaranteed to cancel; the pair removes the patterns' own ghosts | the chapter's measurement model eq. 33 (an added noise term) | `receiver_noise_is_scored_not_removed` (one noise sample leaves ghosts at four shifts); the lower-third promises nothing more |

## Conventions chosen

- Exact integer arithmetic throughout (`i64` in Rust, `int` in Python); the
  score at shift k is Σₙ recording[n]·copy[n − k], shifts from −(N − 1).
- A drawn column's height is the model's score by construction
  (`chapters::draw::settle`, tested against every row the chapter draws) and
  is checked again at render time (`Chart::check`).
- Typographic minus (U+2212) on screen; ASCII hyphen in fact values.
