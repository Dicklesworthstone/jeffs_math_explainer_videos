# Storyboard — Two Bad Echoes Can Make One Perfect Measurement

The hand-edited source of this video's words and pictures.
`python3 -I tools/storyboard.py golay_echoes` checks it (gate G2) and writes
`script.tsv` from the words column. Every number below is a fact in
`docs/fact-sheet.md`, computed in `src/facts.rs` and recomputed by `oracle.py`.

Words are spoken exactly as written, one thought at a time: no colons,
semicolons, dashes or parentheses in them. `*(…)*` is an annotation, never
spoken. `⟨predict⟩` opens a prediction question; the beat ends with the
question and the picture holds (a visible four-second countdown, "pause for
more time") before the next beat answers it.

## The arc

One tiny example runs through the whole video: pattern A is up, up, up, down;
pattern B is up, up, down, up. Every chapter keeps those two rows of four
pulses on screen.

| Chapter | Spine | Status |
|---|---|---|
| `01_two_bad_echoes` | send a pattern, score it against its echo at every shift; every single pattern leaves ghosts; a second pattern's ghosts are the same size with the opposite sign; added, only the echo is left, twice as tall; the guarantee's assumption: the same scene for both sends | silent scratch draft accepted centrally at a201b4a (native tests, facts, glyphs, cues, frames); narration and the full video not yet accepted |
| `02_what_the_ghosts_hide` | two things in the scene, one loud and one quiet, three steps apart: their echoes overlap and add; pattern A's row is each echo's own row added, and the loud one's ghost erases the quiet one's peak (a zero score where something is); pattern B's ghost doubles it; added, the scene itself, eight times over; sending A twice keeps the quiet one hidden (`model::two_measurements` with the scene 4, 0, 0, 1) | silent scratch draft accepted centrally at a201b4a (80.5 s; native tests, facts, glyphs, cues, frames); narration and the full video not yet accepted |
| `03_why_they_cancel` | one pulse paired with itself, doubled into the length-2 pair, doubled again into A and B (`model::double`); at one step, every lined-up pair sits inside a half or reaches across the middle: inside products match in both rows, crossing products flip sign; added, the crossing ones cancel and the rest is twice the old pair's rows, at every shift and every doubling; leave out the flip and the ghosts come back | silent scratch draft accepted centrally at b7665b6 (76 s; native tests, facts, cues, frames); narration and the full video not yet accepted |
| `04_when_it_breaks` | the clean control first and kept beside every failure; the echo moves one step between the sends (it splits three and five, ghosts at six shifts); the same echo comes back upside down (not a new object, not a negative distance; in radar a half turn of phase, Chi, Pezeshki and Calderbank eq. 8): the true echo cancels and every ghost doubles; in general, with no added noise, our four-pulse pair sums to eight times the second scene plus A's row of the change (2N at length N); a noise blip is scored, not guaranteed to cancel, and the peak survives; the ending: two bad echoes make one perfect measurement only in the ideal case, the same echoes in both sends and no added noise | scene and tests written, awaiting central compile, tests and draft; not yet rendered or accepted |

## Vocabulary (one word per idea)

**pulse** (one up or down blip; never "chip", "bit" or "symbol"); **pattern**
(a row of pulses; never "code" or "sequence" in the voice; "pattern A",
"pattern B" on screen); **send** (never "transmit"); **echo** (what comes
back, and the place it really comes from); **copy** (the receiver's own
pattern, the one that slides); **line up** (two pulses at the same place);
**agree** and **disagree** (same or opposite directions); **block** (one
agreement; a disagreement takes one away); **score** (the blocks left at one
shift; never "correlation" in the voice); **shift** (where the copy sits) and
**step** (one place along); **ghost** (a score where there is no echo; the
lower-third names "sidelobe" in a later chapter); **row** (the scores at
every shift, side by side).

## Color roles

| Idea | Color |
|---|---|
| pattern A's pulses | blue `#58C4DD` |
| pattern B's pulses | violet `#C792EA` |
| an agreement, and a block above the line | green `#83C167` |
| a disagreement, and a block below the line | red `#FC6255` |
| the echo: where the thing really is; "echo" and its column marker | `hero` gold `#FFD866`, strokes and words only, never a block |
| a ghost's word | `dim` |
| a pair across the middle of a doubled pattern (chapter 03): its arc, and a ring on its block | teal `#5CD0B3`, strokes only |
| baselines, shift numbers, the sender and the thing | `faint` and `dim` |

## 01_two_bad_echoes: Two bad echoes (~82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Send out four pulses. Up, up, up, down. Something bounces them back. | A sender on the left. Pattern A's four blue pulses appear at it one by one on "Up, up, up, down"; a dot runs right to a dim block on the far right and back, and the pattern returns as the echo row | pulses built from the pattern in facts, motion | |
| 01_2 | Slide a copy along what came back, and multiply the pulses that line up. | The sent pattern is the copy: it comes down under the echo, three steps to the left, and slides right one step at a time until it sits exactly under it. A "for programmers" lower-third gives the score as code | slide, lower-third (code) | |
| 01_3 | Pulses pointing the same way agree, and add a block. Opposite ways disagree, and take one away. | A key on the right: two up pulses over a green block, an up over a down over a red block, each with its product (+1, −1) | legend built from the products | |
| 01_4 | Lined up exactly, all four agree. A score of four, at the echo. | At shift 0 the four lined-up pairs light, four green blocks appear between the rows and drop into the score column under the copy's start; the gold word "echo" above it, the score 4 beside it | blocks from the products, highlight, chart | |
| 01_5 | One step off, three pairs line up. Two agree, one disagrees. A score of one, where nothing is. | The copy steps right by one; three pairs light: two green blocks and one red. In the column the red block lands on a green one and both vanish; one green block is left, and the word "ghost" | blocks cancel, highlight | |
| 01_6 | Slide all the way, and ghosts appear on both sides, a quarter the echo's size. | The copy steps quickly through every other shift, scoring each; the full row stands: one block up at one step either side, one block hanging down at three steps either side, nothing at two; "ghost" by each | fast slide, chart, lagged labels | |
| 01_7 | With more than one pulse, every pattern leaves ghosts. So send a second. Up, up, down, up. | The two far columns flash: only the end pulses meet there. Pattern A's row shrinks to a shelf at the upper right; the echo and copy become pattern B's violet pulses (up, up, down, up). A "precisely" lower-third states the far-shift rule | highlight, morph rows, lower-third | |
| 01_8 | Its ghosts are the same size, at the same shifts. What if we add the two rows? ⟨predict⟩ | B's row is scored quickly: the same echo of four, ghosts of one at the same shifts but on the other side of the line. A's row comes down from the shelf and hovers over B's, column above column | fast slide, chart | |
| 01_9 | Every ghost meets its opposite and cancels. Only the echo is left, twice as tall. Two bad echoes, and not a single ghost. | A's row drops onto B's: at every ghost a green and a red block meet and vanish together; at the echo A's four blocks stack on B's four. The score 8, the gold "echo" above it. On "not a single ghost" a two-line "precisely" lower-third keeps the exact claim (same still scene for both sends; receiver noise is scored too and is not guaranteed to cancel) | blocks cancel, stack, highlight, lower-third | |
| 01_10 | But the guarantee needs the same scene for both sends. Move the echo one step between them, and ghosts return. | The added row clears; A's row returns over B's, as the two sends were. On "Move the echo" B's row steps one shift right and A's drops onto it: this time ghosts are left at six shifts and the echo is split three and five | chart, blocks cancel, highlight | |

### Notes for 01

- The copy's first pulse sits over the column its score lands in, so the
  score row is drawn where the copy was.
- Prediction (01_8): the question is on screen by its last word; four seconds
  of silence with the countdown; the answer starts with 01_9.
- Exact versions kept on screen: 01_7 (every pattern of two or more pulses
  scores one up or one down at its last shift; a single pulse has no other
  shift) and 01_9 (exactly zero away from the echo when both sends meet the
  same still scene; noise picked up by the receiver is scored too and is not
  guaranteed to cancel). Both cards are two lines, so they clear the shift
  numbers.
- 01_10 states the guarantee's assumption and one tested way to break it (the
  echo moved one step); it does not claim that every change breaks the
  cancellation (`model::tests::a_changed_scene_leaves_ghosts` keeps the
  general identity: what is left is the change, scored by pattern A).
- Budget (2026-10-18 scratch draft at 8f154b5: 92.2 s): the kit's scratch
  voice reads 2.6 words a second plus 0.4 s a line, thought gaps measured
  0.81 to 0.83 s, the think window 5.2 s, the tail 2.8 s. This revision cuts
  15 words, one line and the 2 s hold, and ends its last animation before
  the last line does: about 82 s by the same measures.

## 02_what_the_ghosts_hide: What the ghosts hide (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Now two things echo back. A loud one, and a quiet one three steps farther. | "the scene" at the top: two gold outlines on one line, four units tall at step 0 ("loud") and one unit tall at step 3 ("quiet"), the true answer the receiver never sees directly | scene built from the facts | |
| 02_2 | Pattern A's two echoes overlap, and the receiver records their sum. | Pattern A comes back twice: the loud echo, four times as tall, from step 0, and the quiet echo, normal size, from step 3, on a row of its own. The quiet row rises into the loud one; where they overlap (step 3) four down and one up become three down: the recording, 4, 4, 4, −3, 1, 1, −1 | motion, rows that add | |
| 02_3 | Score it as before. The loud one stands tall at zero. At three steps, where the quiet one is, the score is zero. | A copy of A sweeps along the recording and A's row stands in thin slabs, one per unit: tall at 0, nothing at 3. A gold dashed line drops from the quiet outline to shift 3, where the score 0 appears | fast slide, chart, highlight | |
| 02_4 | Zero is not empty. Each echo brings its own row of scores, and at three steps they collide. | A's row splits into each echo's own row: the loud echo's row (A's row four times over, centred on 0) held above, the quiet echo's row (A's row as it is, centred on 3) below; both columns at 3 flash. A "for programmers" lower-third: the filter is linear | chart, highlight, lower-third (code) | |
| 02_5 | The loud one's ghost there is minus four. The quiet one's peak is plus four. They cancel. | "−4" under the loud row's column 3 (four red slabs), "+4" over the quiet row's (four green); on "They cancel" the loud row drops onto the quiet one, every slab meets its match or stacks, and column 3 empties: A's row again | blocks cancel, stack | |
| 02_6 | One step either side, the loud ghosts stand four tall, as tall as the quiet echo. | Columns −1 and 1 flash with "4" over each; at 3 a gold outline four units tall shows what the quiet echo alone would score | highlight | |
| 02_7 | Pattern B's ghosts have the other sign. What will B score at three steps? ⟨predict⟩ | A's row rises to wait above; a "?" over shift 3 on the empty line below; the countdown | chart | |
| 02_8 | Eight. Its ghost lands on the quiet echo instead of erasing it, and doubles it. | B's row stands below: at 3, eight green slabs, "8", twice the gold outline | chart, highlight | |
| 02_9 | Add the two rows. Thirty-two at zero, eight at three, zero everywhere else. The scene, eight times over. | A's row drops onto B's: every ghost meets its opposite and vanishes; 32 stands at 0 and 8 at 3, four to one like the scene's outlines, which flash with them. A "precisely" lower-third: the same still scene for both sends; receiver noise is scored too and is not guaranteed to cancel | blocks cancel, stack, highlight, lower-third | |
| 02_10 | Sending A twice would not help. Its ghosts never cancel themselves, so the quiet one stays hidden. | The sum clears; A's row stands above and again below, and drops: every column doubles, 0 stays 0 at three steps, the ghosts beside the loud one stand eight tall. A "precisely" lower-third: of the 256 ordered pairs of four-pulse patterns, exactly the 32 whose ghosts cancel give this scene back | chart, stack, highlight, lower-third | |

### Notes for 02

- Every row is computed: A's, B's and the sum from `model::measure` and
  `model::two_measurements` on the scene 4, 0, 0, 1; each echo's own row from
  `model::contributions`. A column's height is its score, one slab per unit,
  and every row is checked against the model when it is drawn.
- Zero is not empty: A scores 0 at the quiet echo's step (an echo there) and
  scores at six shifts where nothing is (`G-F6.a.zero_at_echo`,
  `G-F6.a.score_without_echo`). Only the pair's sum is zero exactly where the
  scene is empty.
- Assumptions, kept on screen at 02_9: the same still (linear, unchanging)
  scene for both sends; finite rows, nothing wrapping around; receiver noise
  is scored too and is not guaranteed to cancel.
- The wrong pair (02_10) is a theorem, not a sample: for any ordered pair of
  four-pulse patterns and any scene with an echo, the two rows add to 8 times
  the scene exactly when the pair is complementary, because the difference
  is the scene convolved with the pair's leftover ghosts and the last nonzero
  score of that convolution is the product of the two rows' last nonzero
  scores (`model::tests::only_a_pair_whose_ghosts_cancel_gives_the_scene_back`).
- Prediction (02_7): the question is on screen by its last word; four
  seconds of silence with the countdown; B's row appears only with 02_8.
- Budget, by chapter 01's measured draft: 160 words at 2.6 a second plus
  0.4 s a line, thought gaps 0.82 s, the think window 5.0 s, a 1.8 s tail:
  about 79.5 s.

## 03_why_they_cancel: Why they cancel (~77 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Where do pairs like these come from? Start with one pulse, paired with itself. | One up pulse on the top line ("first"), one on the bottom line ("other"): the pair of length one | rows built from `model::double`'s seed | |
| 03_2 | Now double it. Write the first, then the other. Then the first again, and the other flipped. | A copy of the other rises to the right of the first; the other slides right and a copy of the first drops in on its left; the other flips over: up, up on top, up, down below | motion, morph rows | |
| 03_3 | Double again, and out come patterns A and B. | The same moves on the two-pulse pair: up, up, up, down on top, up, up, down, up below, renamed pattern A and pattern B; a thin tick marks the middle of each | motion, morph rows | |
| 03_4 | Why do their ghosts cancel? Slide each copy one step, and look at the pairs that line up. | Faint copies slide one step under each row; then arcs over each row join the pulses that lined up, a product block on each arc. Arcs that cross the middle tick are teal and their blocks wear a teal ring | slide, arcs, blocks from the products | |
| 03_5 | Inside a half, B matches A, or has both pulses flipped. Those products don't change. | The inside arcs flash in both rows, block by block the same colour; B's second half and A's second half flash: every pulse flipped | highlight | |
| 03_6 | A pair across the halves has exactly one flipped pulse, so its product changes sign. | The teal arcs flash: green over A, red over B | highlight | |
| 03_7 | So what's left when we add A's and B's products here? ⟨predict⟩ | The blocks wait; the score line and its shift numbers below; the countdown | chart | |
| 03_8 | The crossing products cancel in pairs. The rest is twice the old pair's rows, whose ghosts already cancel. | The two ringed blocks drop into the column at shift 1 and vanish together; then the four inside blocks drop, two green and two red, and vanish in pairs: 0. Beside it, from the old pair at one step: 2(1 + (−1)) = 0 | blocks cancel, TeX numbers, chart | |
| 03_9 | This split works at every shift, for every doubling. Each doubling of a cancelling pair gives a new one, twice as long. | Shifts 2 and 3 in quick succession: only teal arcs, each pair of blocks meets between the rows and vanishes. X and Y over A's halves, X and −Y over B's; the identity S over X,Y plus S over X,−Y equals 2(S of X plus S of Y). A "precisely" lower-third: for halves of the same length at every shift; doubling from one pulse gives lengths 1, 2, 4, 8 and on, but one pair of length 10 cancels too | blocks cancel, TeX derivation, lower-third | |
| 03_10 | Leave out the flip, and the crossing products add instead. The ghosts come back. | The identity and the X, Y labels leave as B's second half flips back (they belong to the flipped pair): both rows are pattern A. At the far shift both teal arcs carry a red block; they stack, two below the line; the rows clear and the rest of A-twice stands, ghosts on both sides. A "precisely" lower-third: the same still scene for both sends; receiver noise is scored too and is not guaranteed to cancel | arcs, blocks stack, chart, lower-third | |

### Notes for 03

- Every pattern shown is `model::double`'s output from one pulse; every
  product is `facts::split` of the shown halves; the column is checked
  against the model when it settles.
- The argument is a proof, not a sample: at shift k each lined-up pair of
  (X then −Y) is the matching pair of (X then Y) with its product unchanged
  inside a half and negated across the middle, so the rows add to twice X's
  and Y's rows (old rows extended by zero) for any equal-length halves and
  every shift. Tested pair by pair in
  `model::tests::the_doubling_identity_holds_pair_by_pair`; the oracle
  checks the identity on every pair of halves of 1 to 5 pulses.
- Boundaries: shift 0 has no crossing pair; shift k from 1 to 2n − 1 has
  min(k, 2n − k); from shift n on every pair crosses (`G-X3`,
  `crossing_pairs_begin_at_one_step_and_fill_the_far_shifts`).
- The planted negative (03_10): without the flip the crossing products add,
  and at the far shift that is 2·X[0]·Y[n − 1], never zero, for every pair.
  Flipping the first half instead also works; flipping both does not.
- Not every pair whose ghosts cancel is a doubling: Golay's pair of length
  10 (`model::GOLAY_10`). The chapter does not claim otherwise.
- Budget: 153 words at 2.6 a second plus 0.4 s a line, thought gaps 0.82 s,
  the think window 5.0 s, a 1.8 s tail: about 76.8 s.

## 04_when_it_breaks: When it breaks (~82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Back to the clean case. Both sends meet the same scene, and every ghost cancels. | At the top, each send's scene as a gold outline: "first send" and "second send", one echo at step 0 in both. A's row stands above, B's below; A's drops onto B's: 8 at 0 and nothing else, its name now "both rows added". That control shrinks to a shelf at the upper right, "same scene, A + B", and stays there | scene outlines, chart, blocks cancel | |
| 04_2 | Now let the echo move one step between the sends. | The second send's outline slides one step right | motion | |
| 04_3 | B's row slides with it, and its ghosts miss their opposites. | A's row stands above, B's below and slides one step right with the echo; A's drops onto it: green and red no longer meet everywhere | chart, blocks cancel | |
| 04_4 | The echo splits into three and five, with ghosts at six shifts. | 3 over shift 0 and 5 over shift 1; the six ghost columns flash, next to the clean control on its shelf | highlight | |
| 04_5 | Now keep it in place, and let the next echo come back upside down. | The row clears; the second send's outline returns to step 0 and turns over: it hangs below the line | morph | |
| 04_6 | Not a new object, and not a negative distance. The same echo, flipped. In radar, half a turn of phase does this. | The flipped outline flashes in the same column as the first send's, one outline, one place. A "precisely" lower-third: half a turn of phase between the sends multiplies the second echo by −1 (Chi, Pezeshki and Calderbank, eq. 8); the same reflector at the same distance | highlight, lower-third | |
| 04_7 | What happens to the true echo when we add? ⟨predict⟩ | A's row above; below, B's row for the flipped echo, every column B's own turned over; the countdown | chart | |
| 04_8 | It cancels, and every ghost doubles instead. | A's row drops onto it: at 0 four green meet four red and vanish, 0; the ghosts stack two high, flashing; the control still on its shelf | blocks cancel, stack, highlight | |
| 04_9 | In general, with no added noise, our four-pulse pair sums to eight times the second scene, plus pattern A's row of whatever changed. | h₁ and h₂ beside the two sends; the identity S_A(h₁) + S_B(h₂) = 8h₂ + S_A(h₁ − h₂). A "precisely" lower-third: S_A(h) is A's row for scene h; rows and scenes are zero past their ends; the identity is for no added noise; 8 = 2N for our N = 4 pulses, and any complementary pair of length N gives 2N | TeX derivation, lower-third | |
| 04_10 | Noise is scored too. A stray blip gets its own little row, and nothing guarantees it cancels. | The flipped outline turns back; the clean total stands again below, still "both rows added". The first recording, with one stray dim blip after the echo; the blip's own row stands above (A read backwards from the blip) and drops: 8 stays 8, the blip's scores remain, and the total is renamed "both rows + blip" | chart, highlight | |
| 04_11 | So two bad echoes make one perfect measurement, but only in the ideal case, when both sends meet the same echoes and no noise is added. | The noisy row clears; the clean control comes back from the shelf to full size under the two still scenes, "both rows added"; the title, "Two bad echoes, one perfect measurement". A "precisely" lower-third: perfect means exactly 8 times the scene with no ghosts, in the ideal case: the same echo response in both sends and no added noise; added noise is scored, not guaranteed to cancel | chart, title, lower-third | |

### Notes for 04

- Every row is the model's: the control and the moved row from
  `two_measurements`, the flipped row with `model::FLIPPED` (one echo at
  step 0, strength −1), the blip from `model::NOISE_BLIP`. Each chart is
  checked against the model as it is drawn.
- The flipped echo is a theorem, not a sample: each pattern's own peak is N,
  so the echo scores N − N = 0, and B's ghosts are minus A's, so every ghost
  doubles, for every complementary pair
  (`a_flipped_echo_cancels_itself_and_doubles_the_ghosts`). The moved echo
  still scores at both its places: a change need not erase the echo.
- The identity (04_9) follows from linearity:
  S_A(h₁) + S_B(h₂) = S_A(h₂) + S_B(h₂) + S_A(h₁ − h₂) = 8h₂ + S_A(h₁ − h₂),
  scenes and rows extended by zero past their ends
  (`a_changed_scene_leaves_pattern_as_row_of_the_change`; the oracle checks
  all 125 × 125 pairs of three-step scenes).
- Noise: the blip is scored as A read backwards from where it sits; the
  peak is untouched, and nothing in the pair's identity cancels the blip.
  No claim that noise or motion always destroys the echo.
- The eq. 8 claim stays within the chapter's support: a phase factor on the
  second send's response, a half turn giving −1.
- The identity's 8 is this four-pulse pair's: two for every pulse
  (`G-F16.pulses`, `G-F16.per_pulse`); a complementary pair of length N
  gives 2N (the oracle checks every pair of 1 to 8 pulses). The identity
  is for no added noise; 04_10 shows what noise adds.
- The ending names its conditions in the sentence that makes the promise:
  the ideal case, the same echoes in both sends and no added noise, so the
  last line keeps the limitation 04_10 has just shown.
- Every label names the row beside it as it stands: "pattern A" and
  "pattern B" only while their rows stand, "both rows added" for a total,
  "both rows + blip" once the blip's row has dropped, and the shelf's
  "same scene, A + B".
- Budget: 166 words at 2.6 a second plus 0.4 s a line, thought gaps 0.82 s,
  the think window 5.0 s, a 1.8 s tail: about 82 s.
