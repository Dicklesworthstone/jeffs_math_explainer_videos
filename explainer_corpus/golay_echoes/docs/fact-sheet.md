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
| F2 | A's score row | the first row of blocks (01_4 to 01_6); the thumbnail's row A | `model::autocorrelation(CODE_A)` | `G-F2.a.scores` | `-1,0,1,4,1,0,-1` | computed |
| F2 | A's score at its echo | "all four agree", "A score of four" (01_4) | same, shift 0 | `G-F2.a.peak` | `4` | pinned |
| F2 | the products one step off | three blocks, two green and one red (01_5) | `model::products(A, A, 1)` | `G-F2.a.one_off.products` | `1,1,-1` | computed |
| F2 | pairs, agreements, disagreements, score one step off | "three pairs line up. Two agree, one disagrees. A score of one" (01_5) | same | `G-F2.a.one_off.pairs`, `.agree`, `.disagree`, `.score` | `3`, `2`, `1`, `1` | computed |
| F2 | ghost size against the echo | "a quarter the echo's size" (01_6) | largest ghost ÷ peak | `G-F2.ghost_fraction` | `1/4` | computed |
| F2 | where A's ghosts are | "ghost" labels (01_6) | `Scores::ghost_shifts` | `G-F2.a.ghost_shifts` | `-3,-1,1,3` | computed |
| F2 | A's far-shift score | the far columns flash (01_7) | `model::far_shift_score` | `G-F2.a.far_shift_score` | `-1` | computed |
| F2 | B's score row, ghosts and their size | the second row (01_8); "the same size, at the same shifts"; the thumbnail's row B | `model::autocorrelation(CODE_B)` | `G-F2.b.scores`, `G-F2.b.ghost_shifts`, `G-F2.b.largest_ghost` | `1,0,-1,4,-1,0,1`; `-3,-1,1,3`; `1` | computed |
| F3 | the two rows added | the merged row (01_9); the thumbnail's sum | `model::two_measurements(A, still, B, still)` | `G-F3.sum.scores` | `0,0,0,8,0,0,0` | pinned: Wikipedia's sum (8, 0, 0, 0) at lags 0 to 3, mirrored |
| F3 | the added echo, the largest ghost left | "8"; "not a single ghost" (01_9) | same | `G-F3.sum.peak`, `G-F3.sum.largest_ghost` | `8`, `0` | pinned: definition 1, 2N δ(k) with N = 4 |
| F3 | added echo against one row's | "twice as tall" (01_9) | 8 ÷ 4 | `G-F3.peak_ratio` | `2` | computed |
| F4 | the echo moved between the sends | "one step" (01_10) | `facts::echo_step` | `G-F4.moved.step` | `1` | computed |
| F4 | the two rows added, the echo moved | the last row (01_10), from shift −3 | `model::two_measurements(A, still, B, moved)` | `G-F4.moved.first_shift`, `G-F4.moved.scores` | `-3`; `-1,1,1,3,5,-1,-1,1` | computed |
| F4 | ghosts that return, and where | "ghosts return"; the flashing columns; "3" and "5" over the split echo | same | `G-F4.moved.largest_ghost`, `G-F4.moved.ghost_shifts` | `1`; `-3,-2,-1,2,3,4` | computed |
| X1 | far-shift score sizes, every pattern of 2 to 10 pulses | "With more than one pulse, every pattern leaves ghosts" (01_7) and its lower-third | `model::far_shift_score` over all 2,046 patterns | `G-X1.far_shift_sizes` | `1` | computed (edge test) |

### Chapter 02 (scene 4, 0, 0, 1; rows from shift −3 to 6)

| id | quantity | shown / said | computed in | oracle key | value | source |
|---|---|---|---|---|---|---|
| F5 | the scene; its echoes; loud, quiet; the quiet one's step | outlines (02_1); "two things", "three steps farther" | `model::LOUD_AND_QUIET`, `facts::loud_and_quiet` | `G-F5.scene`, `.echoes`, `.loud`, `.quiet`, `.quiet_step` | `4,0,0,1`; `2`; `4`; `1`; `3` | pinned: the task's fixed scene |
| F5 | pattern A's recording | the merged row (02_2) | `model::echo(A, scene)` | `G-F5.recording_a` | `4,4,4,-3,1,1,-1` | computed |
| F6 | A's row; its score at the quiet echo | the row of slabs; "the score is zero" (02_3) | `facts::hidden_a` | `G-F6.a.scores`, `G-F6.a.at_quiet` | `-4,0,4,15,4,1,0,1,0,-1`; `0` | computed |
| F6 | each echo's part at shift 3 | "minus four", "plus four" (02_5) | `model::contributions` | `G-F6.a.loud_part_at_quiet`, `G-F6.a.quiet_part_at_quiet` | `-4`; `4` | computed; the oracle measures each echo alone |
| F6 | the loud ghosts one step either side | "four tall", "as tall as the quiet echo" (02_6) | same | `G-F6.a.loud_ghosts_one_step` | `4,4` | computed |
| F6 | zero at an echo; scores with no echo | "Zero is not empty" (02_4) | `facts::chapter_02` | `G-F6.a.zero_at_echo`, `G-F6.a.score_without_echo` | `3`; `-3,-1,1,2,4,6` | computed (boundary test) |
| F7 | B's row; its score at the quiet echo; its loud part there; against the quiet echo | "Eight", "doubles it" (02_8) | `facts::hidden_b` | `G-F7.b.scores`, `.at_quiet`, `.loud_part_at_quiet`, `.over_quiet` | `4,0,-4,17,-4,-1,8,-1,0,1`; `8`; `4`; `2` | computed |
| F8 | the two rows added; at each echo; largest ghost; magnification | "Thirty-two at zero, eight at three, and zero everywhere else. The scene, eight times over." (02_9) | `facts::hidden_sum` | `G-F8.sum.scores`, `.at_loud`, `.at_quiet`, `.largest_ghost`, `G-F8.magnification` | `0,0,0,32,0,0,8,0,0,0`; `32`; `8`; `0`; `8` | pinned: 8 times the scene (definition 1, N = 4) |
| F9 | A sent twice: sends, row, at the quiet echo, one step either side, each of A's ghosts against A's own | "Sending pattern A twice", "just doubles its ghosts", "stays hidden" (02_10) | `facts::a_twice` | `G-F9.a_twice.sends`, `.scores`, `.at_quiet`, `.at_one_step`, `.ghost_ratio` | `2`; `-8,0,8,30,8,2,0,2,0,-2`; `0`; `8,8`; `2` | computed |
| X2 | ordered pairs of 4-pulse patterns; how many give this scene back; of those, complementary; complementary in all | the 02_10 lower-third | `facts::pairs_giving_back` | `G-X2.pairs`, `.giving_back`, `.giving_back_complementary`, `.complementary` | `256`; `32`; `32`; `32` | computed (exhaustive; the theorem is `only_a_pair_whose_ghosts_cancel_gives_the_scene_back`) |

### Chapter 03 (the doubling; halves X = 1, 1 and Y = 1, −1)

| id | quantity | shown / said | computed in | oracle key | value | source |
|---|---|---|---|---|---|---|
| F10 | the seed, its length; the pair of length 2; of length 4 | rows (03_1 to 03_3); "one pulse" | `facts::doublings` (`model::double`) | `G-F10.seed`, `.seed_length`, `.length_two`, `.length_four` | `1;1`; `1`; `1,1;1,-1`; `1,1,1,-1;1,1,-1,1` | pinned: the seed; Wikipedia's length-4 pair |
| F10 | the shift; A's and B's products there, inside and across | arcs and blocks (03_4 to 03_6); "one step" | `facts::split` | `G-F10.shift`, `.a.inside`, `.a.across`, `.b.inside`, `.b.across` | `1`; `1,-1`; `1`; `1,-1`; `-1` | computed; the oracle classifies by index |
| F10 | pulses B flips in a crossing pair | "exactly one flipped pulse" (03_6) | `facts::chapter_03` | `G-F10.flipped_in_crossing` | `1` | computed |
| F10 | the old halves' scores at one step; inside total; total | "2(1 + (−1)) = 0" (03_8) | `facts::doubling_numbers` | `G-F10.old_rows_at_shift`, `.inside_total`, `.total` | `1,-1`; `0`; `0` | computed |
| F10 | the identity's factor; the length factor | "twice the old pair's rows", "twice as long"; the identity (03_9) | same | `G-F10.factor`, `G-F10.length_factor` | `2`; `2` | computed |
| F11 | lengths doubling reaches from one pulse, each checked; Golay's pair of length 10 | the 03_9 lower-third | `facts::doubling_lengths`, `model::GOLAY_10` | `G-F11.lengths`, `.length10`, `.length10_complementary`, `.length10_power_of_two` | `1,2,4,8,16,32,64`; `10`; `true`; `false` | computed; the length-10 pair from Wikipedia, its cancelling checked |
| F12 | A paired with itself: row, far products, far score | 03_10 | `model::autocorrelation(A)` times 2 | `G-F12.no_flip.scores`, `.far_products`, `.far` | `-2,0,2,8,2,0,-2`; `-1,-1`; `-2` | computed |
| X3 | crossing pairs at shifts 0 to 3 (halves of 2) | storyboard notes | `facts::split` | `G-X3.crossing_counts` | `0,1,2,1` | computed (boundary test, all halves to 6) |

### Chapter 04 (what breaks the pair; the blip's rows from shift −3 to 4)

| id | quantity | shown / said | computed in | oracle key | value | source |
|---|---|---|---|---|---|---|
| F13 | the control: both rows added, the same scene; its largest ghost | "both rows added", then the shelf "same scene, A + B" (04_1); "every ghost cancels" | `facts::sum` | `G-F13.control.scores`, `.largest_ghost` | `0,0,0,8,0,0,0`; `0` | computed (the row F3 pins) |
| F14 | the step moved; the total at the two echo steps; ghost count | "one step" (04_2); "three and five", "six shifts" (04_4) | `facts::moved` | `G-F14.moved.step`, `.at_echoes`, `.ghost_count` | `1`; `3,5`; `6` | computed |
| F15 | the flipped scene; the flip factor; the total; at the echo; ghosts against A's | the hanging outline (04_5); "half a turn" and the card's −1 (04_6); "Cancelled out", "every ghost doubles" (04_8) | `facts::flipped`, `model::FLIPPED` | `G-F15.phase.scene`, `.flip_factor`, `.scores`, `.at_echo`, `.ghost_ratio` | `-1`; `-1`; `-2,0,2,0,2,0,-2`; `0`; `2` | computed; the −1 is Chi, Pezeshki and Calderbank eq. 8's half turn of phase |
| F16 | the identity's factor; pulses per pattern; factor per pulse | "our four-pulse pair", "eight times the second scene" (04_9); the identity and its card, "8 = 2N for our N = 4 pulses" | `facts::chapter_04` | `G-F16.factor`, `.pulses`, `.per_pulse` | `8`; `4`; `2` | computed; 2N at shift 0, definition 1 |
| F17 | the blip; its own row; the total with it; the peak | the dim blip, "the blip's own row" (04_10); "both rows + blip", "8" | `facts::blip_row`, `facts::noisy` | `G-F17.noise.blip`, `.blip_row`, `.scores`, `.peak` | `0,0,0,0,1`; `0,0,0,0,-1,1,1,1`; `0,0,0,8,-1,1,1,1`; `8` | computed; the oracle builds the blip's row from where the blip meets each copy pulse |
| F18 | sends | "two bad echoes" (04_11) | `facts::chapter_04` | `G-F18.sends` | `2` | computed |

## Claims (non-numeric)

| id | claim as narrated or shown | source | verified how |
|---|---|---|---|
| C10 | Doubling (X then Y, X then −Y) keeps a pair cancelling, at every shift, for any equal-length halves: inside products repeat, crossing products flip sign (03_5 to 03_9) | the argument in `model::double`'s documentation; the identity S(X then Y) + S(X then −Y) = 2(S(X) + S(Y)), old rows extended by zero | `the_doubling_identity_holds_pair_by_pair` (pair by pair, all up/down halves of 1 to 4 and all halves of 1 or 2 values in −2 to 2); the oracle checks the identity on every pair of halves of 1 to 5 |
| C11 | Leave out the flip and the ghosts come back (03_10) | at the far shift the one crossing pair gives 2·X[0]·Y[n − 1], never zero | `leaving_out_the_flip_brings_the_ghosts_back` (every complementary pair of 1 to 4 pulses; flipping the first half also works, flipping both does not) |
| C12 | Doubling does not build every cancelling pair | Golay's length-10 pair (Wikipedia) | `some_pairs_are_not_doublings` |
| C7 | Two echoes overlap and add; a scene's row is each echo's own row added (02_2, 02_4) | the measurement model (chapter eq. 33) is linear in the scene | `a_scenes_row_is_the_sum_of_its_echoes_rows`: every code of 1 to 4 pulses, every scene of 3 or 4 steps with strengths −2 to 2; the oracle measures each echo alone |
| C8 | A zero score is not an empty step; a nonzero score is not an echo; only the pair's sum is zero exactly where the scene is empty (02_3, 02_4) | the rows above | `a_zero_score_is_not_an_absent_echo` |
| C9 | Sending A twice does not help; only a pair whose ghosts cancel gives the scene back, and any other pair gets every scene with an echo wrong (02_10) | definition 1; the residual is the scene convolved with the pair's leftover ghosts, whose last nonzero score is the product of the two last scores | `only_a_pair_whose_ghosts_cancel_gives_the_scene_back` (all 256 ordered pairs × 125 scenes, the lemma checked as stated); the oracle repeats it by its own convolution |
| C1 | Sliding a copy along the recording and adding the lined-up products finds the echo (the matched filter) | Chi, Pezeshki and Calderbank eqs. 5–6 (the zero-Doppler ambiguity function is the autocorrelation); Purdue notes (matched filters) | `model::correlate`; `model::tests::the_matched_filter_is_convolution_with_the_reversed_code` |
| C2 | Every pattern of more than one up/down pulse leaves a ghost; one pulse has no other shift | the far shift overlaps only the two end pulses, so it scores ±1 | `every_code_longer_than_one_pulse_has_a_ghost` (lengths 2 to 10, all codes); `G-X1` |
| C3 | B's ghosts are A's with the other sign, at every shift | definition 1 (the sum is zero off shift 0) | `a_pairs_ghosts_are_always_opposite`: every complementary pair of 1 to 8 pulses (4, 8, 0, 32, 0, 0, 0, 192 ordered pairs) |
| C4 | Added, the two measurements of the same scene leave the scene itself, 2N times, with no ghosts (exact version: same still scene, no noise) | definition 1 and the chapter's eq. 9 | `the_same_scene_twice_returns_the_scene_magnified`: every pair up to 8 pulses × all 125 scenes of three steps with strengths −2 to 2; oracle repeats it independently |
| C5 | The guarantee needs the same scene for both sends; a one-step move brings ghosts back | the chapter's eqs. 7–8 and section 2.2 (Doppler between the two transmissions spoils the sum); Purdue notes ("where there is not significant motion between measurements") | `a_changed_scene_leaves_ghosts` (moved, upside down, stronger) and the identity A·sa + B·sb = A·(sa − sb) + (A + B)·sb; the chapter claims the assumption is needed for the guarantee, not that every change breaks it |
| C6 | Noise picked up by the receiver is scored too, and is not guaranteed to cancel; the pair removes the patterns' own ghosts | the chapter's measurement model eq. 33 (an added noise term) | `receiver_noise_is_scored_not_removed` (one noise sample leaves ghosts at four shifts); the lower-third promises nothing more |
| C13 | The echo upside down in the second send is the same reflector at the same distance, not a new object or a negative distance; half a turn of phase multiplies the second response by −1 (04_6) | Chi, Pezeshki and Calderbank eq. 8 (a phase factor on the second transmission's response), used for nothing more | `model::FLIPPED` is one echo at step 0 with strength −1; `the_broken_cases_are_the_ones_on_screen` |
| C14 | Flipped, the true echo cancels and every ghost doubles (04_8) | each pattern's own peak is N, so N − N = 0; B's ghosts are minus A's, so A's row minus B's is twice A's off the echo | `a_flipped_echo_cancels_itself_and_doubles_the_ghosts` (every complementary pair of 1 to 8 pulses); a moved echo still scores at both its steps, so no claim that every change erases the echo |
| C15 | With no added noise, S_A(h₁) + S_B(h₂) = 2N·h₂ + S_A(h₁ − h₂): 8 for this four-pulse pair (04_9) | linearity and definition 1, scenes and rows zero past their ends | `a_changed_scene_leaves_pattern_as_row_of_the_change` (every complementary pair of 1 to 4 pulses × two-step scenes in −1 to 1, and A, B × every three-step scene in −2 to 2); the oracle checks 125 × 125 scene pairs |
| C16 | A noise blip is scored as its own row and is not guaranteed to cancel; here the peak survives (04_10) | the measurement model eq. 33 (an added noise term) | `a_noise_blip_is_scored_and_the_peak_survives`; the oracle checks a blip at every position 0 to 7 |
| C17 | Two bad echoes make one perfect measurement, exactly 2N times the scene with no ghosts, only in the ideal case: the same echoes in both sends and no added noise (04_11) | C4, C5, C15, C16 | the spoken line names both conditions; the 04_11 card repeats them |

## Conventions chosen

- Exact integer arithmetic throughout (`i64` in Rust, `int` in Python); the
  score at shift k is Σₙ recording[n]·copy[n − k], shifts from −(N − 1).
- A drawn column's height is the model's score by construction
  (`chapters::draw::settle`, tested against every row the chapter draws) and
  is checked again at render time (`Chart::check`).
- Typographic minus (U+2212) on screen; ASCII hyphen in fact values.
