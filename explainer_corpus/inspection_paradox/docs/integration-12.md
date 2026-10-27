# Chapter 12: native source and integration

2026-10-24, GoldGate, for root (assignment ROOT1232-I12, Mail 6927). This
note covers the native scene source for `12_one_more_timetable`, the film's
last chapter. It sits in one new file that nothing registers yet, so the
build does not compile it.

None of it has been compiled, rendered, narrated or listened to. Every check
below is a Python model, a rustfmt parse or a source reading, not native
execution.

## Files

| file | sha256 | bytes |
|---|---|---|
| `src/chapters/one_more_timetable.rs` | `e067a6d02a7f4206b4e370c821a7e94ef7ceb8607ba10f7982ca398a153ad4c2` | 76303 |

The file is 1820 lines and holds `OneMoreTimetable` and 12 tests. No shared
helper was added or edited. It uses:

- `draw`, `timetable` and `two_averages::written`;
- `no_bus_late`'s public `curve`, `minute_up`, `plot_at`, `red_x`,
  `triangles_at` and `wait_of`;
- `back_to_the_stop::{stop, slide_at}` and `twelve_minutes::grow_lagged`.
  `back_to_the_stop.rs` is the pending chapter 11, so 12 compiles only
  after 10/11 are registered;
- the kit's `live_bar` (also called by 11 and by `lie_robotics`),
  `receipts_card` (called by `lie_robotics` and `bakery_algorithm`),
  `names_to_search` and `precisely`.

Three private drawings are copied word for word, because they are private
in their chapters: 01's `street`, and 07's `plot_frame`, `mark_at` and
`spot`, with their constants. `the_street_is_chapter_01s` and
`the_plot_is_chapter_07s` compare the copies byte for byte with
`the_stop.rs` and `no_bus_late.rs`.

### The base

The patch is against the **pending 10/11 integration**: the live files
with the Mail 6926 patch applied (`docs/integration-10-11.md`, patch
sha256 `c42705490f51d03cefe3a79b950b759118e885d68ff0932c5f33cdba5f102305`).
It does not apply to the live tree until 10/11 lands. On the live tree,
`git apply --check` fails on the five files 10/11 also changes; that is
the negative control.

Read at 13:47Z, the live files still equal the 10/11 base:

| file | live sha256 | after the 10/11 patch (this patch's base) |
|---|---|---|
| `src/chapters/mod.rs` | `5db91597369ddd1edc11f47ba7477642397b88b533fa630185de03c92c6710dc` | `db225ada9c4330afe2db81e7a6f05b749430445b647134d0fea9b694835c11a1` |
| `src/main.rs` | `6d93ba3251912bff33d9a72822cf3590795c531787088577732e4ee966d3b84e` | `4eb50f22511c07a6823c9ee2b2f642bdc49549d89d9079645022e07786874bc7` |
| `docs/storyboard.md` | `b310188dc6252294a3528631f9e24cb72928b266d8c401f74a337228fb6a6f8f` | `493f2ff4643c169315fde4b7fcb3a08568067d2fd6ce9081cd8e33e1bcfef351` |
| `docs/storyboard-draft.md` | `de5591dd804c427ab8a817390822b7f0468150ab85f3a4e334c6f1202fd8005c` | `c819fa96ca178fefc746a1f92c5cbff86b9112cae59bdfcffd85fb2dc4db8be0` |
| `docs/pause-overrides.json` | `0d425e2456bbd83e2581086c5018422b9e5331e770c139f195fdf7722b3cd914` | `4f762187b723f0aa60c5f6f5a433a0792f75081c37e23d6a222d403ac0f02468` |
| `src/facts.rs` | `a3cd5b3e035beb4760c587bd3d096b9b3fa62e3abd4643afc9303d4fe5f43a92` | unchanged by 10/11 |
| `oracle.py` | `47a2c058700d3224a817129428a8b68bd8157c37bdd269ba75a5b7716907dcef` | unchanged by 10/11 |
| `script.tsv` | `95cdcab9d87ba04eaa7631b1b091e6a3d621dbc25f153572e57726ce175eda29` | regenerated, see step 3 |

The "after" column is the 10/11 note's own "after" table, so root's 08
square-minute and pause-map repairs are in the base. The 12 source reads,
but does not change:

| file | sha256 |
|---|---|
| `draw.rs` | `e51741b39e65e31a61f9c8b490b8dc99976197abbdc687ee75e3939833ba7f30` |
| `timetable.rs` | `66c43d843f42b762f208ca7b523dc8470b1e7b9339ec001c8fe059016d64facf` |
| `no_bus_late.rs` | `5ebf1bb67b955f8a3bfd5f567ec18a12938136c9e3741ce2453ef3dbf588e42b` |
| `the_stop.rs` | `1432b76358c2dfeb685bc8fc407e1ecd9aeeae0cb74f1db6bb35f7eabdf1b841` |
| `back_to_the_stop.rs` | `3c98fc169d961cbda78e80df5f1b3e3b53b3c73193f3ecb957391f733dcf6ee1` |
| `twelve_minutes.rs` | `61206a6f6ae1ff08c78e4c00cbf1df8ce8d25ceac516dc65ec56c59cb7721df3` |
| `two_averages.rs` | `78325de488aa368adf1c5cd9b0481ae6af839f297ef179948ccd03372a570756` |
| `model.rs` | `bc3be4ad39edaef314d7ddb8edd431bf60286a9922e11a67483d2c327ef3011b` |
| `palette.rs` | `aeb18f555f303cd45dde4cc6ee1e5cc30a8d9bcb1e091fdf94394c4b53624a7a` |

If root changes the 10/11 result before applying this, the hunks anchor on:
- the 11 registry entry and the `one_a_minute` / `same_sum` module lines
  (`mod.rs`);
- the doc line "Chapters 01 through 11 have scene source" and the last 11
  SPOKEN row (`main.rs`);
- the end of 11's pause notes (`storyboard.md`);
- the draft's 12 heading, which runs to the end of the file;
- the `_comment` and the last 11 override (`pause-overrides.json`);
- the `F-Q1.gaps` lines and the 3:20 pin (`facts.rs`), the `F-Q1.gaps`
  line and the d = 4 self-test check (`oracle.py`).

`git apply --3way` or the steps below by hand would then do.

## Integration, in order (root's)

1. **The 10/11 integration first**, as `docs/integration-10-11.md` gives
   it, including its `script.tsv` step. Its handoff items stay as they
   were: 10_3 → 10_4 must never be a coherent seam, and the proposed
   `F-P1.long_calls` fact is still only a proposal.

2. **Apply the patch** at the end of this note from the repo root,
   `git apply --check` then `git apply`. It edits seven files:
   - **`src/chapters/mod.rs`:** adds `pub mod one_more_timetable;` in
     alphabetical order, registers `12_one_more_timetable`
     (`one_more_timetable::OneMoreTimetable`, intensity 0.5, a proposal),
     and rewrites the module comment: all twelve chapters now have scene
     source.
   - **`src/main.rs`:** appends the 19 SPOKEN rows of step 4 after 11's,
     and the doc line becomes "All twelve chapters have scene source".
   - **`src/facts.rs`:** adds `F-Q1.short` and `F-Q1.long` after
     `F-Q1.gaps` (step 5), and pins them, 4 and 8, in
     `the_facts_compute_and_their_pins_hold`.
   - **`oracle.py`:** adds the same two keys from its own `hour_gaps`, and
     one self-test check.
   - **`docs/storyboard.md`:** appends the authored section, notes
     included.
   - **`docs/storyboard-draft.md`:** replaces the draft's 12 section (from
     its heading to the end of the file) with the same text, so
     `the_full_draft_holds_the_storyboard_verbatim` holds. The draft's
     chapter table is root's and unchanged.
   - **`docs/pause-overrides.json`:** appends the four entries of step 6
     in the file's compact layout. `_comment` becomes "(chapters 01 to
     12)" and drops its sentence about chapter 12 being in the draft.

   | section | sha256 | bytes |
   |---|---|---|
   | `## 12_one_more_timetable: One more timetable (~106 s)` | `16da59fd72df8042c9f8c1397f27b1096401ea1faa48970532db5b6698179028` | 10430 |

   That hash covers `"\n## "` plus the section with trailing whitespace
   stripped, which is what the verbatim test compares. `~106 s` is
   storyboard.py's estimate, not a limit.

   Patched files:

   | file | before (the 10/11 result) | after | bytes after |
   |---|---|---|---|
   | `src/chapters/mod.rs` | `db225ada…` | `0b6dec9835610354694766abd2f7bd00f917c53063ab02871075645194a19f4d` | 3253 |
   | `src/main.rs` | `4eb50f22…` | `2d6c843b5dc3ef279c1cc035d1b44ccec8c79c3a5d1998878e12c3d95af729cc` | 23328 |
   | `src/facts.rs` | `a3cd5b3e…` | `024169e5a3d85ebe36c5d024fd8ba975490e71519f3aa32a076adfedda7f761f` | 21519 |
   | `oracle.py` | `47a2c058…` | `4984013fa3ebcd0f45ab41e19086c9a32345efb3f0b42598c05f36057c8a01f7` | 31549 |
   | `docs/storyboard.md` | `493f2ff4…` | `85e3f9c96d614b2f87b0829e182128b173d76552add20353837be834b61cb677` | 69433 |
   | `docs/storyboard-draft.md` | `c819fa96…` | `57cde1720132ec9ac7be9846d654e3c3730a156e22a9ec983d6d6102c1c0c9b3` | 75996 |
   | `docs/pause-overrides.json` | `4f762187…` | `49fcd5fd90caa93605f781442e46197cb9367daa4d58bd59ef923dfe94946eb7` | 9931 |

3. **`script.tsv`:** `python3 -I tools/storyboard.py inspection_paradox`
   (this slug only), then `python3 -I tools/storyboard.py --check
   inspection_paradox`. That appends 12_1 to 12_9.
   - I did not run storyboard.py's main. I called its own `parse`,
     `check` and `script_text` on scratch copies, with the working tree's
     `tools/storyboard.py`.
   - The 10/11 result's script emulates to
     `355d3ce4db895c262b2e94c62658abddbcc1ff10c8ec2f19c6842a7e686cddee`
     (12305 bytes), the 10/11 note's expectation.
   - After this patch: the nine appended lines hash to
     `5d520bc28d4bda82696e3fd042997d88c9150bfb3e4b790e9e9edd7cb4d78a0d`
     (1398 bytes), and the whole file to
     `8a65ec15cfafd499006ac2590013562744eb81d478fd50e1b67ea8968920ad50`
     (13703 bytes, 92 lines plus the header).
   - These are expectations, not receipts.

4. **Numeric bindings (`SPOKEN`, in the patch).** Checked as main.rs's
   `every_spoken_number_is_the_fact_it_speaks_for` checks them, on the
   emulated script:

   | line | phrase | key | value |
   |---|---|---|---|
   | 12_1 | one a minute | F-W2.per_minute | 1 |
   | 12_2 | four minutes after the blue | F-Q1.offset | 4 |
   | 12_2 | the gaps are four and eight | F-Q1.gaps | 4,8 |
   | 12_3 | The gap of four | F-Q1.short | 4 |
   | 12_3 | half of four squared | F-F1.area_factor | 1/2 |
   | 12_3 | eight square minutes | F-Q1.short_area | 8 |
   | 12_3 | The gap of eight | F-Q1.long | 8 |
   | 12_3 | half of eight squared | F-F1.area_factor | 1/2 |
   | 12_3 | thirty-two square minutes | F-Q1.long_area | 32 |
   | 12_4 | forty square minutes | F-Q1.total_area | 40 |
   | 12_4 | the twelve minutes they cover | F-T1.period | 12 |
   | 12_4 | Forty over twelve | F-Q1.total_area | 40 |
   | 12_4 | over twelve is | F-T1.period | 12 |
   | 12_4 | three and a third minutes | F-Q1.mean_wait | 10/3 |
   | 12_4 | Three minutes and twenty seconds | F-Q1.mean_wait_clock | 3:20 |
   | 12_5 | closer the wait comes to three minutes | F-O1.min_wait | 3 |
   | 12_6 | four minutes twenty | F-W1.mean_wait_clock | 4:20 |
   | 12_7 | allow the full ten minutes | F-W1.allowance | 10 |
   | 12_8 | Ten minutes is the worst case | F-W1.allowance | 10 |

   - Every phrase is in its line and every value is the patched oracle's.
   - Over the whole patched SPOKEN (205 rows) on the 92 emulated lines,
     the test's checks pass. 12_9 says no number word, so it has no row.
   - A stricter check finds every 12 number word inside one of its line's
     row phrases.
   - "Forty" and "twelve" are said twice in 12_4, once in the sum and once
     in the division, so each has a row per phrase.
   - Every key is added literally by the patched `facts.rs` and printed
     by the patched oracle. No line uses a `PRONOUN_ONES` phrase, and no
     12 line says a pronoun "one".

5. **Two new facts (in the patch).** `F-Q1.short` = 4 and `F-Q1.long` = 8,
   the puzzle's shortest and longest gap, so the spoken "four" and
   "eight" of "The gap of four" and "The gap of eight" bind to the gaps,
   not to the offset or an area.
   - `facts.rs` takes them as the min and max of
     `model::hour_gaps(PUZZLE_OFFSET)`; the oracle takes the min and max
     of its own `hour_gaps`. `facts-check` then compares two
     derivations.
   - The oracle's new self-test check asks for 4 and 8, and for
     `F-Q1.gaps` to be the two joined. With `long` taken as the min in a
     scratch copy, that check fails (negative control).
   - The patched oracle prints 168 facts, the 166 before plus these two,
     every other line unchanged (output sha256
     `3fccba331bce7dabadb8e0827f13edd5ea4fefc2928d464f7663fc85459cb787`).
     `--self-test` reports 0 failures.
   - `facts.rs` is root's file. The edit is three lines and a pin.

6. **Pause overrides (in the patch).** Editorial hypotheses from the
   section's notes, not measurements:

   | cue | after | default | authored |
   |---|---|---|---|
   | 12_4 | "three and a third minutes" | thought 0.35–1.1 | thought 0.5–1.0 |
   | 12_6 | "And your train" | question 0.65–1.65 | question 0.35–0.7 |
   | 12_7 | "not an average" | thought 0.35–1.1 | thought 0.4–0.8 |
   | 12_8 | "exactly on time" | thought 0.35–1.1 | thought 0.5–1.0 |

   - I ran them through `tools/narration_director.py`'s own `directions`
     on the patched draft's 92 lines, together with the 32 earlier
     overrides. Each lands on exactly one boundary, not a cue end, and
     replaces the punctuation default there. No earlier chapter's row
     changes.
   - 12_6 keeps the role `question`: the next sentence answers it, so
     only the range is shorter.
   - Refused, as they should be: "minutes", which occurs many times in
     12_4, and the old wording "forty minutes of waiting", no longer in
     12_4. An override at 12_9's end ("during a long gap") is accepted by
     the director, so my check's cue-end rule is what catches it.
   - The holds at the ends of 12_2, 12_4, 12_8 and 12_9 are scene waits,
     not narration gaps, so they have no entries.

7. **Glyphs.** The characters 12 prints beyond ASCII are ² · ½ × ÷ – ⅓ →.
   - Registered chapters already print ² ½ × ÷ ⅓ → (08 and 05 among them)
     and · (06, 09). The – of "renewal–reward theorem" is first printed by
     10 (`same_sum.rs`, pending), so it is on screen before 12 either way.
   - Lockfile fonts (fmd-font at `126a5304`), read with a small
     format-4/12 cmap reader. That is a cmap reading, not a probe run:
     IBM Plex Sans Regular maps all eight. Computer Modern maps · ÷ –.
     Noto Sans Math Symbols maps · × ÷ →.
   - So a row like "4 min → ½ × (4 min)² = 8 min²" mixes fonts, as 08's
     rows already do. That is a frame check for weight and baseline, not
     a missing glyph.

Then root's usual order: `./x inspection_paradox facts-check`, `cargo fmt
--check`, Clippy with `-D warnings`, the crate's tests,
`inspection_paradox glyphs`, a headless run and a preview.

### Cue IDs and what each waits for

Every phrase occurs once in its line, after the previous one. The 23
phrases are parsed from the construct body, and every `At "…"` in the
section's picture column is one of them or its line's opening words.

- **12_1:** "Now forget the planners"; "one a minute"; "spread evenly over
  the hour".
- **12_2:** "Move the red bus"; "What's the average wait".
- **12_3:** "The gap of four"; "The gap of eight".
- **12_4:** "over the twelve minutes they cover"; "Dividing square
  minutes by minutes"; "Forty over twelve"; "Three minutes and twenty
  seconds". The sum comes in as the line starts, "Together they make
  forty square minutes".
- **12_5:** "That's a point on the curve"; "between our timetable and the
  even timetable"; "The more even the gaps".
- **12_6:** "Back on our timetable"; "four minutes twenty".
- **12_7:** "your worst morning"; "Turn up just as a red bus leaves";
  "allow the full ten minutes".
- **12_8:** "only because every bus runs exactly on time"; "It's a bound
  in this model".
- **12_9:** "Your bus isn't late"; "You're just more likely".

### Authored holds

- **Prediction (12_2):** after "What's the average wait?" the kit's
  `think` countdown, 4 s with "pause for more time". Until it ends, the
  screen holds the red bus at four minutes, the brackets "4" and "8" and
  the question: no triangle, no area, no wait. The triangles are made in
  12_3, after `think` returns.
- **Inspection (after 12_4):** the whole step, "average wait = 40 min² ÷
  12 min", "min² ÷ min = min", "= 3⅓ min" and the boxed "= 3 min 20 s",
  stands with no voice until both hold: its reading time (4.7 s at 15
  characters a second, from the moment it is complete) and 2 s past the
  line and its breath (`INSPECT_S`).
- **Visual (before 12_5):** chapter 07's U and its three marks come back,
  0.8 s, still with no voice (`CALLBACK_RT`). 12_5 then starts on the
  curve already in place.
- **Reading (12_8):** K10 stands for its reading time, 10.5 s, from the
  moment it is up. The line and its breath are inside that time. Then K10,
  the bar and its label clear, 1.2 s, while you go back to the stop,
  before 12_9 speaks.
- **Reading (12_9):** after the last line and its breath, the names band
  stands for its reading time, 8.3 s. Then the picture fades, and the
  receipts card comes up alone, with the repository link, for its own
  reading time, 12.9 s. Then the film fades. The test asserts the
  timeline runs past the last line by more than both together.

## How the source meets the assignment

- **The same objects.** The hour line, its blue and red markers and the
  sixty passengers are rebuilt where 11_7 left them, in their stacks a
  minute before the buses. The triangles are 05's and 11's. The U and
  its three points are 07's, at 07's place and scale. The street strip
  and the gold wait bar are 01's. One tracker, the red bus's offset,
  moves the red markers and repaints the passengers in 12_2 and 12_6, and
  the triangles follow it back to our timetable in 12_6.
- **Back to random arrivals first.** "Now forget the planners." Each
  planner slides back along the line, never forward, to the moment it was
  ready. That is the middle of its minute, so the row is one a minute,
  evenly spread, as the line says and the dim caption prints. The dot at
  :01 runs off the start and comes in at :59½.
  `the_planners_go_back_to_where_they_were_ready` checks every slide.
  `last` refuses passengers that are not evenly one a minute
  (`F-W2.per_minute`).
- **The question does real work.** The puzzle uses only the picture the
  viewer has: move one bus, read two gaps, predict. The answer is
  withheld (see the holds). Then the triangles are raised, and each
  area is shown as half its gap squared, in square minutes. The two are
  added, divided by the twelve minutes they cover with the units
  divided too, and the result lands on the U from 07 at d = 4.
- **The 12_4 repair.** The area is forty square minutes and is never
  called minutes of waiting. "Dividing square minutes by minutes leaves
  minutes": 40 min² ÷ 12 min = 3⅓ min, three minutes twenty. That is 08's
  "Any gap holds half its length squared" and 11's "Fifty-two square
  minutes over twelve minutes". 05_5's passenger-minute reading, one
  passenger a minute turning square minutes into minutes of someone's
  waiting, is a different counting measure and is not used.
  `the_area_is_square_minutes_over_minutes` makes the old draft wording
  its negative control.
- **Cost against margin.** 12_6 names the average, four minutes twenty,
  as "what the stop costs you on an average morning". 12_7 says a
  deadline needs the worst morning. You turn up just as the :02 red bus
  leaves, and 01's bar grows from the :12 blue bus back to you, the whole
  long gap: "allow 10 min". `last` checks that wait against the model, and
  that no moment in the middle of any second of the hour waits longer.
- **The condition and its limit.** 12_8: "Ten minutes is the worst case
  only because every bus runs exactly on time. It's a bound in this
  model, not a guarantee against real delays, or for the whole journey."
  The green on-time marks of 01 and 07 land on "every bus runs exactly on
  time". K10, the exact version, says the same: "with every bus exactly
  on time, the longest wait is the longest gap, 10 min, whenever you turn
  up; it says nothing about late buses or the rest of the journey".
- **The ending.** 12_9 restates the title's claim in the honest
  direction: "Your bus isn't late. You're just more likely to turn up
  during a long gap." "every bus on time" is ringed on the first
  sentence. The names and the receipts then each get their own reading
  time.

## Departures from the draft

- The draft's 12_1 (even gaps, three minutes) repeated 07_3 and 07_4. It
  is replaced by the return to random arrivals.
- The draft's 12_3 gave the answer before the triangles: "Three minutes
  and twenty seconds" was its first sentence. Now the triangles, the
  areas, the sum and the division come first, and the answer comes last
  in 12_4.
- The draft's 12_4 called the forty square minutes "forty minutes of
  waiting". Repaired as above.
- The draft's 12_5 is split into the cost (12_6), the margin (12_7) and
  the margin's condition and limit (12_8, K10). The draft said "you might
  turn up just as a red bus leaves". Now the bar shows that morning.
- Six cues became nine. storyboard.py estimates ~106 s, against the
  draft's ~64 s. Nothing was compressed, and no runtime cap applies.

## Tests (in the new file; none has run)

1. `the_chapter_runs_and_says_its_lines_in_order`: the whole chapter
   headless at 10 fps with the scratch voice. The nine lines are said in
   order. 12_5 starts at least `INSPECT_S` past 12_4's end. 12_9 starts
   more than K10's reading time after 12_8. The timeline runs past the
   last line by more than the names' and receipts' reading times. Needs
   the integrated `script.tsv`.
2. `the_numbers_are_the_models_and_a_wrong_fact_is_refused`: every label,
   asserted as text. Then 16 wrong facts, each refused by `last` with its
   key named: offset 2, gaps 2,10, short 2, long 10, areas 16 / 64 / 52,
   the wait as 40 or as the even guess 3, the clock 4:20, `F-O1.wait.d2`
   3:20, `F-O1.min_wait` 10/3, two passengers a minute, the allowance
   clock 4:20, the allowance 8 (the puzzle's long gap) and the average
   as 10:00.
3. `the_area_is_square_minutes_over_minutes`: the square-minute words, the
   printed units, and 40/12 = 10/3. The old draft sentence is the
   negative control.
4. `the_u_is_nearer_three_where_the_gaps_are_more_even`: whole minutes and
   tenths, with a swapped U and a wrong even offset refused, and the d = 4
   point on 07's curve.
5. `the_longest_wait_is_the_long_gap`: turning up as each red bus leaves
   waits the allowance. At the middle of every second of the hour the wait
   is shorter, though over nine minutes at worst. On the puzzle's
   timetable the longest wait would be eight.
6. `the_planners_go_back_to_where_they_were_ready`: each slide starts in
   11's stacks, ends at its ready moment, and never moves forward.
7. `the_colors_and_triangles_are_the_gaps`: `after_red` gives each
   passenger its gap at d = 2 and d = 4. Every triangle's shoelace area
   is half its gap squared.
8. `the_street_is_chapter_01s` and 9. `the_plot_is_chapter_07s`: the
   copies equal the originals, byte for byte.
10. `the_printed_words_are_the_drafts`: every printed string is quoted in
    the draft. Needs the integrated draft.
11. `the_fixed_words_carry_no_numbers`: no digit in a fixed word.
12. `the_picture_stays_apart_and_on_screen`: eight moments of estimated
    text boxes. Everything stays on screen and apart, rows stay above the
    d = 4 triangles, the strip stays above the d = 2 triangles, no label
    sits on the U, and you stay clear of the bar label, clocks, markers
    and cost. The bands keep to three lines, and the receipt lines fit.

## Checks run (no Cargo)

All from the repo root, on scratch copies, by `python3 -I -B` or rustfmt.

- **rustfmt.** `--check` is clean on the file in place. On a scratch copy
  of the crate with the 10/11 patch and this patch applied, `rustfmt
  --check` on `main.rs` (every module) is clean. With one semicolon
  removed from the 12 file in a further copy, it reports the parse error
  (negative control).
- **The patch.** It applies to that copy after the 10/11 patch, and the
  seven results equal my patched files byte for byte. On the live tree
  `git apply --check` fails, as above.
- **Names.** All 62 imported names are used (`Timed` through `.rt(`), as
  are the 6 test imports. Every binding in the construct is read again.
  Every method and free function it calls is called by a root-compiled
  chapter, or is defined here, with these exceptions: `live_bar` and
  `slide_at` are called by the pending 11; `receipts_card` by
  `lie_robotics` and `bakery_algorithm`; and six std methods. These are
  names, not signatures. The signatures of every model, facts and
  helper function `last` calls were read against `model.rs`, `facts.rs`,
  `draw.rs`, `timetable.rs`, `two_averages.rs` and `back_to_the_stop.rs`.
- **A Python port of the tests (114 checks, 0 problems).**
  - The numbers and labels as asserted.
  - All 16 refusals, each naming its key.
  - The U's order with its two controls, the longest wait, the
    planners' slides, the colours and the shoelace areas.
  - 23 printed strings and 4 plot marks quoted in the section; K10 equal
    to the section's exact version.
  - The layout's eight moments, with a lowered-rows negative control.
  - The phrases once each and in order, with a reworded-line control;
    the 21 fact keys present in the patched oracle.
  - The timing emulation below, whose transcribed beats follow the
    construct's 32 says and waits.
- **Copies.** A Python emulation of the two copy tests finds `street`,
  `plot_frame`, `mark_at`, `spot` and every copied constant identical,
  with a negative control.
- **Integration, emulated (0 problems).** storyboard.py's `check` has no
  problems (total 17.0 min over 12 chapters). The scripts are as in step
  3. `check_video` passes, with the registry, the new type and its
  `name()`. The verbatim draft test passes. SPOKEN passes (step 4), and
  every key is added. Six negative controls are caught: a stale script, a
  missing registry key, a changed section, a dropped 12_5 row, the old
  12_4 wording and an invented key.
- **Oracle.** `--self-test` on the patched oracle: 0 failures, and the
  min-for-max copy fails the new check.
- **Overrides.** As in step 6.

## Timing

storyboard.py estimates 106.3 s (words at 2.68 a second plus its hold
allowances). The scratch-voice emulation of the transcribed beats (2.6
words a second plus 0.4 s a line) gives 139.6 s, because it counts the
real holds:

- silence after a line and its breath: 3.8 s after 12_4 (the inspection
  hold and the U's return), 4.7 s after 12_8 (K10's reading, then the
  clearing) and 23.4 s after 12_9 (the names, the fade, the receipts and
  the last fade);
- and the 12_2 countdown after its question.

Two phrases land after their words: "Now forget the planners" by 0.8 s
(the chapter's opening fade, as 10_1 has) and "Back on our timetable" by
0.12 s. Both are within the sentences they name. Real narration lengths
will move all of this.

## Controls that failed, and what changed

These were caught by my own checks before handoff:

- The timing emulation found three pictures landing late:
  - 12_1's caption, 0.56 s, after a 1.6 s flash;
  - 12_5's knob, 1.8 s, because the U faded in after the line began;
  - 12_9's ring, 1.7 s, behind the K10 clearing.
  Fixed: the flash is 1.0 s, the U returns before 12_5 speaks, and K10
  clears before 12_9 speaks. The section's picture column says so.
- Cross-checking the picture column against the code found 12_8's green
  marks written at the line's start, while the code waits for "only
  because every bus runs exactly on time". The section now says the
  latter.
- Earlier drafts of the layout test had three flaws, each fixed before
  handoff:
  - the plot frame's box contained its own labels, so the axes are now
    two line boxes checked only against texts;
  - a fudge on the strip, now per item against the d = 2 triangles;
  - you and the bar tested as one moment, though they touch by design,
    now separate.
- 07's `mark_point` returns only the label, which would have left the
  dots behind in 12_6. 12 has its own `mark`, returning both.
- The name check's first run flagged a trait import, local closures, std
  methods and a string-stripping artifact. Those were checker faults, not
  source faults; the check was refined, and no source changed.

## Pronunciation risks (nothing probed or listened to)

- "half of four squared", "half of eight squared": a break before
  "squared" invites (½ × 4)². The next words, "eight square minutes",
  settle it, and the row on screen shows ½ × (4 min)². Listen for no
  break.
- "thirty-two" is new to the narration. "three and a third minutes" is
  already in the lexicon's risks.
- "Three minutes and twenty seconds" is a duration. "four minutes twenty"
  (12_6) drops "seconds", as 06's "four twenty" does, though here with
  "minutes".
- "square minutes", five times, a unit; "Forty over twelve", "over" as
  division.
- "minutes" is the time noun throughout (the lexicon's MIN-it).
- Contractions "Here's", "What's", "That's", "It's", "isn't", "You're".
- Never spoken: "min²", "÷", "renewal–reward", "github.com",
  "Dicklesworthstone".

## Visual risks (for root's frames)

- The sixty dots sliding back in 12_1, and the :01 dot coming in again at
  the hour's end.
- Teal and gold for the 4 and 8 gaps. `timetable::gap_color` knows only 2
  and 10, so 12 paints by `after_red`, the rule 07 uses.
- The see-through triangles behind the markers: z is set before adding.
- Mixed fonts in the rows (Glyphs).
- The layout test's boxes are estimates (0.0045 units a point a
  character), not measured glyph boxes.

## Gates not run (all of them native)

- Cargo build, `cargo fmt --check` in the repo, Clippy `-D warnings`.
- The crate's tests: these 12, main.rs's, and facts.rs's pins.
- `./x inspection_paradox facts-check` with the two new facts.
- The glyph probe, the headless run, previews and frames.
- TTS, contextual takes, seam checks, measured pauses and listening.
- Beads, the catalog and every promotion decision. Those are root's.

A written plan and passing arithmetic do not prove rendered clarity,
natural pacing or English-accent acceptance.

## The patch

363 lines, sha256
`7d84592dbe539f5fc73c960c6810ee99a31d8c8c2c437fd87f734f482f287cb9`,
against the 10/11 result. Between the fences, byte for byte:

````diff
diff --git a/explainer_corpus/inspection_paradox/docs/pause-overrides.json b/explainer_corpus/inspection_paradox/docs/pause-overrides.json
index bcd9e95..84fb863 100644
--- a/explainer_corpus/inspection_paradox/docs/pause-overrides.json
+++ b/explainer_corpus/inspection_paradox/docs/pause-overrides.json
@@ -1,5 +1,5 @@
 {
-  "_comment": "Authored pause intentions for the active script (chapters 01 to 11), in tools/narration_director.py's override format, to splice into a map's \"overrides\" after `plan`. Each 'after' phrase occurs once in its cue. Ranges are editorial hypotheses from the storyboard's notes, not measurements or acceptance; nothing has been narrated or listened to. The chapter's prediction countdown and title hold are picture holds in the scene, not narration gaps. Chapter 12 has its pause intentions in docs/storyboard-draft.md and joins here with its scene.",
+  "_comment": "Authored pause intentions for the active script (chapters 01 to 12), in tools/narration_director.py's override format, to splice into a map's \"overrides\" after `plan`. Each 'after' phrase occurs once in its cue. Ranges are editorial hypotheses from the storyboard's notes, not measurements or acceptance; nothing has been narrated or listened to. The chapter's prediction countdown and title hold are picture holds in the scene, not narration gaps.",
   "overrides": [
     {
       "video": "inspection_paradox",
@@ -286,6 +286,26 @@
       "video": "inspection_paradox", "cue": "11_7", "after": "every wait would be one minute",
       "role": "thought", "range_s": [0.5, 1.0],
       "reason": "The wait bar falls to a minute while the five-in-six share stays; the fall lands before the reason."
+    },
+    {
+      "video": "inspection_paradox", "cue": "12_4", "after": "three and a third minutes",
+      "role": "thought", "range_s": [0.5, 1.0],
+      "reason": "The division's answer as a fraction lands before the same wait is read as a clock; the step stays on screen."
+    },
+    {
+      "video": "inspection_paradox", "cue": "12_6", "after": "And your train",
+      "role": "question", "range_s": [0.35, 0.7],
+      "reason": "The question the film opened with, answered by the next sentence at once: shorter than a question's default."
+    },
+    {
+      "video": "inspection_paradox", "cue": "12_7", "after": "not an average",
+      "role": "thought", "range_s": [0.4, 0.8],
+      "reason": "The turn from the average cost to the worst case, before you are placed just after the red bus."
+    },
+    {
+      "video": "inspection_paradox", "cue": "12_8", "after": "exactly on time",
+      "role": "thought", "range_s": [0.5, 1.0],
+      "reason": "The green on-time marks land before the bound is limited to this model."
     }
   ]
 }
diff --git a/explainer_corpus/inspection_paradox/docs/storyboard-draft.md b/explainer_corpus/inspection_paradox/docs/storyboard-draft.md
index 282c7f8..f4e5464 100644
--- a/explainer_corpus/inspection_paradox/docs/storyboard-draft.md
+++ b/explainer_corpus/inspection_paradox/docs/storyboard-draft.md
@@ -626,37 +626,81 @@ who plan.
   - the verdict after 11_3 and K09 after 11_7 are reading holds, picture
     with no voice until each can be read.
 
-## 12_one_more_timetable: One more timetable (~64 s)
+## 12_one_more_timetable: One more timetable (~106 s)
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 12_1 | And if you plan the buses, even out the gaps. The same ten buses, six minutes apart, bring the average wait down to three. | The dots spread back to one a minute and you return. The red markers slide to +6, the triangles become ten 6 by 6, and "average wait: 3 min" | moves, shapes | |
-| 12_2 | Here's one last timetable to solve with the picture. Put the red bus four minutes after the blue, so the gaps are four and eight. What's the average wait? ⟨predict⟩ | The red markers slide back to +4. Brackets "4" teal and "8" gold, one twelve-minute stretch bright. No triangles. Question card "average wait?"; countdown | moves, brackets, think countdown | |
-| 12_3 | Three minutes and twenty seconds. The triangles hold half of four times four, which is eight, and half of eight times eight, which is thirty-two. | The two triangles rise, 4 by 4 teal and 8 by 8 gold, with "½ × 4 × 4 = 8" and "½ × 8 × 8 = 32" | shapes from the model, TeX from facts | |
-| 12_4 | That's forty minutes of waiting every twelve minutes, and forty over twelve is three and a third. Closer to even, closer to three. | "8 + 32 = 40", "40 ÷ 12 = 3⅓ min = 3 min 20 s" in green, boxed. The U from 07 returns small, with its point at d = 4 marked "3:20" | TeX derivation, plot callback | |
-| 12_5 | And your train? Four minutes twenty is what the stop costs you on an average morning. But you might turn up just as a red bus leaves, so allow the full ten minutes. | The red markers slide back to +2 and the opening picture returns: the street strip with you at the stop and the train at the station. On "Four minutes twenty" a green "average cost: 4 min 20 s" (F-W1.mean_wait_clock). On "just as a red bus leaves" your unlucky ghost from 01_4 glows just after the :02 red marker, and a gold bar reaches from there to the :12 blue bus, the whole long gap, labelled in your color "allow 10 min" (F-W1.allowance_clock) | callback, bar from the model | |
-| 12_6 | Your bus isn't late. You're just more likely to turn up during a long gap. | The opening picture with its gold gaps and you at the stop. Lower-third "names to search: inspection paradox · random incidence · length-biased sampling · renewal–reward theorem · class size paradox". Then the receipts card "Every number in this video was computed by code", with the repository link | lower-third, receipts card | hold |
+| 12_1 | Now forget the planners. Go back to passengers who never look at the timetable, one a minute, spread evenly over the hour. | Chapter 11's end rebuilt without its panel, its labels, its triangles or K09: the hour line, the blue and red markers of our timetable, and the sixty dots in their stacks a minute before the buses, each in its gap's color. At "Now forget the planners" the dots slide back along the line, never forward, each to the moment it was ready, the middle of its minute, and leave their stacks; the dot at :01 runs off the start and comes in again at :59½. Each takes the color of the gap it moves through. At "one a minute" the dots flash in time order, and at "spread evenly over the hour" the dim caption "at random: no look at the timetable, spread evenly over the hour" stands under them | rebuild of 11, moves, colors from the model | |
+| 12_2 | Here's a last timetable to solve with the picture. Move the red bus to four minutes after the blue, so the gaps are four and eight. What's the average wait? ⟨predict⟩ | At "Move the red bus" the red markers slide from two minutes after each blue marker to four, and every dot after a red marker and before the next blue turns gold, the rest teal. Then brackets under every gap, "4" in teal and "8" in gold. At "What's the average wait" the question "average wait?" under the caption; countdown, "pause for more time". No triangle and no wait on screen until the countdown ends | moves, brackets, think countdown | |
+| 12_3 | Raise the triangles again, a minute up for every minute across. The gap of four holds half of four squared, eight square minutes. The gap of eight holds half of eight squared, thirty-two square minutes. | The question and the ring go, and so do the brackets after the first twelve minutes. The wait triangles rise over the whole hour, a minute up as long as a minute across: five 4 by 4 in teal and five 8 by 8 in gold, see-through, behind the markers. At "The gap of four" the first short triangle brightens, the triangles after the first twelve minutes dim, and "4 min → ½ × (4 min)² = 8 min²" comes in at the top in teal; at "The gap of eight" the first long triangle brightens and "8 min → ½ × (8 min)² = 32 min²" comes in under it in gold | shapes from the model, rows from facts | |
+| 12_4 | Together they make forty square minutes, over the twelve minutes they cover. Dividing square minutes by minutes leaves minutes. Forty over twelve is three and a third minutes. Three minutes and twenty seconds. | At "Together they make forty square minutes" "8 min² + 32 min² = 40 min²" under the two rows; at "over the twelve minutes they cover" the two brackets give way to one under the whole stretch, "12 min". At "Dividing square minutes by minutes" "average wait = 40 min² ÷ 12 min" lower left, and "min² ÷ min = min" dim under it; at "Forty over twelve" "= 3⅓ min"; at "Three minutes and twenty seconds" "= 3 min 20 s" in green, boxed and ringed. The whole step stands with no voice until it can be read | unit division, verdict box | hold |
+| 12_5 | That's a point on the curve from before, between our timetable and the even timetable. The more even the gaps, the closer the wait comes to three minutes. | Before the voice, chapter 07's plot returns lower right at its place and scale: the offset axis "red bus after blue (min)" with its ticks, the wait axis, the whole U in green and its three points as 07_6 left them, d = 2 (4:20), d = 6 (3:00) and d = 0 (6:00). At "That's a point on the curve" the knob stands at four minutes on the offset axis with a guide up from it, and the U gains a green point at d = 4 (3:20). At "between our timetable and the even timetable" the "4:20" and "3:00" are ringed; at "The more even the gaps" the "3:00" flashes | plot callback from 07 | |
+| 12_6 | And your train? Back on our timetable, four minutes twenty is what the stop costs you on an average morning. | The rows, the steps, the plot and the stretch bracket go, the dots fade and the triangles return to one brightness. Chapter 01's street strip comes back at the top, you at the stop and the train at the station. At "Back on our timetable" the red markers slide back to two minutes after the blue and the triangles reshape with them, five 2 by 2 in teal and five 10 by 10 in gold. At "four minutes twenty" "average cost: 4 min 20 s" in green, lower right | callback to 01, shapes from the model | |
+| 12_7 | But a deadline needs your worst morning, not an average. Turn up just as a red bus leaves, and you wait the whole long gap, so allow the full ten minutes. | At "your worst morning" the gold triangle after the red bus at :02 brightens and the other triangles dim. At "Turn up just as a red bus leaves" you leave the strip and grow back to full size just under the line at :02, and a gold bar, chapter 01's wait as you arrive, grows from the blue bus at :12 back to you: the whole long gap, the triangle's tallest edge. At "allow the full ten minutes" "allow 10 min" under the bar in your color | bar from the model, callback to 01 | |
+| 12_8 | Ten minutes is the worst case only because every bus runs exactly on time. It's a bound in this model, not a guarantee against real delays, or for the whole journey. | At "only because every bus runs exactly on time" a green dot over every marker and "every bus on time", as in 01 and 07. At "It's a bound in this model" a "precisely" card (K10). It stands until it can be read | lower-third | hold |
+| 12_9 | Your bus isn't late. You're just more likely to turn up during a long gap. | Before the voice, K10, the bar and its label go, and you go back to the stop in the strip. At "Your bus isn't late" "every bus on time" is ringed in green. At "You're just more likely" the sixty dots return in their gaps' colors, and the five long gaps shade gold. When the line ends, the lower third "names to search:" with "inspection paradox · random incidence · length-biased sampling · renewal–reward theorem · class size paradox", held until it can be read. Then the picture fades and the receipts card comes up alone, "Every number in this video comes from the code", "and is checked against an independent oracle", "github.com/Dicklesworthstone/jeffs_math_explainer_videos" and "explainer_corpus/inspection_paradox". It stands until it can be read, then the film fades | lower-third, receipts card | hold |
 
 ### Notes for 12
 
+- The chapter keeps the film's objects: the hour line and markers of 01,
+  the sixty passengers of 02, the triangles of 05 and 11, the U of 07, the
+  street strip and wait bar of 01. One tracker, the red bus's offset,
+  moves the red markers and repaints the dots in 12_2 and 12_6, and the
+  triangles follow it in 12_6.
+- 12_1 undoes 11's planners on screen. Each dot slides back the way it
+  came, to the moment it was ready. The film is back to arrival moments
+  independent of the timetable and spread evenly in time, K01's condition,
+  for the puzzle and the train (F-W2.per_minute).
 - The transfer question (12_2) is new to the viewer but uses only the
-  picture: two triangles, their areas, and division by the time.
-  - F-Q1 computes it second by second.
-  - The oracle checks it against the offset sweep at d = 4.
-  - The passengers' share of the long gap there is 2/3 (F-Q1), shown only
-    as the gold dots.
-- 12_5 pays off the train of 01_1 and 01_4. A deadline needs the longest
-  wait, not the average. Turning up just as a red bus leaves, you wait the
-  whole long gap, ten minutes (F-W1.allowance, `model::longest_wait`; the
-  oracle takes the worst of the hour's 3,600 seconds and rounds up). 4:20
-  is the average cost of a morning, not a safe margin. The line says
-  "might", and the K01 conditions still hold: you turn up at random.
+  picture. Nothing about the answer is on screen until the countdown ends:
+  no triangle, no area, no wait. The triangles come in 12_3, the wait at
+  the end of 12_4. F-Q1 computes the puzzle second by second in the
+  oracle, which checks it against the offset sweep at d = 4.
+- 12_3 and 12_4 count area, as 08 and 11 do. Both axes are minutes, so a
+  triangle's area is in square minutes; the two make forty square minutes
+  (F-Q1.total_area), and dividing by the twelve minutes they cover leaves
+  minutes, 3⅓ min (F-Q1.mean_wait), 3:20 (F-Q1.mean_wait_clock). The words
+  never call the area minutes of waiting. 05_5's bridge, one passenger a
+  minute turning square minutes into minutes of someone's waiting, is a
+  different counting measure and is not used here.
+- New facts F-Q1.short (4) and F-Q1.long (8), the puzzle's two gaps, the
+  shortest and longest of `model::hour_gaps(PUZZLE_OFFSET)`, so the spoken
+  "four" and "eight" of the gaps are bound to the gaps, not to the offset
+  or an area.
+- 12_5's point is the U's at four minutes (`no_bus_late::wait_of`); the
+  scene refuses a U that is not nearer three wherever the gaps are more
+  even, whole minute by whole minute.
+- 12_6 to 12_8 separate the two numbers a traveller needs. The average
+  wait, 4:20 (F-W1.mean_wait_clock), is the cost of an average morning. A
+  deadline needs the longest wait. Turning up just as a red bus leaves you
+  wait the whole long gap, ten minutes (F-W1.allowance,
+  `model::longest_wait`; the oracle takes the worst of the hour's 3,600
+  seconds and rounds up). That bound holds only with every bus exactly on
+  time, and covers the wait at the stop alone.
+- Exact version K10 (12_8): "with every bus exactly on time, the longest
+  wait is the longest gap, 10 min, whenever you turn up; it says nothing
+  about late buses or the rest of the journey".
 - The last line restates the title's claim, "longer than you expect", in
   the honest direction. The wait is longer than the guess from the sign
   because of where random moments fall, not because a bus is late.
+- The names and the receipts each get their own reading time: the names
+  over the last picture, the receipts on a card alone after the picture
+  fades.
+- Departures from the earlier draft: its 12_1 (even gaps, three minutes)
+  is 07_3 and 07_4 again and is dropped for the return to random arrivals;
+  its 12_4 called the forty square minutes "forty minutes of waiting"; its
+  12_5 is split into the cost (12_6), the margin (12_7) and the margin's
+  condition (12_8).
 - Pause intentions:
-  - "Three minutes and twenty seconds." 0.5 to 1.0 s;
+  - after "What's the average wait?" the countdown, a prediction hold;
+  - after "three and a third minutes." a thought pause, 0.5 to 1.0 s, before
+    the clock reading;
+  - the step after 12_4 is an inspection hold, picture with no voice until
+    it can be read, and the U then returns before 12_5 speaks;
   - after "And your train?" 0.35 to 0.7 s, the question the film opened
     with;
-  - before "Your bus isn't late." a turn, 0.9 to 1.6 s;
-  - after the last line, the chapter's hold.
+  - after "not an average." 0.4 to 0.8 s, the turn from the cost to the
+    margin;
+  - after "exactly on time." 0.5 to 1.0 s, while the green marks land;
+  - K10 after 12_8, then the names and the receipts after 12_9, are
+    reading holds, and the picture clears K10 before 12_9 speaks.
diff --git a/explainer_corpus/inspection_paradox/docs/storyboard.md b/explainer_corpus/inspection_paradox/docs/storyboard.md
index 367afaf..9cca91f 100644
--- a/explainer_corpus/inspection_paradox/docs/storyboard.md
+++ b/explainer_corpus/inspection_paradox/docs/storyboard.md
@@ -534,3 +534,82 @@ who plan.
     while the bar falls;
   - the verdict after 11_3 and K09 after 11_7 are reading holds, picture
     with no voice until each can be read.
+
+## 12_one_more_timetable: One more timetable (~106 s)
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 12_1 | Now forget the planners. Go back to passengers who never look at the timetable, one a minute, spread evenly over the hour. | Chapter 11's end rebuilt without its panel, its labels, its triangles or K09: the hour line, the blue and red markers of our timetable, and the sixty dots in their stacks a minute before the buses, each in its gap's color. At "Now forget the planners" the dots slide back along the line, never forward, each to the moment it was ready, the middle of its minute, and leave their stacks; the dot at :01 runs off the start and comes in again at :59½. Each takes the color of the gap it moves through. At "one a minute" the dots flash in time order, and at "spread evenly over the hour" the dim caption "at random: no look at the timetable, spread evenly over the hour" stands under them | rebuild of 11, moves, colors from the model | |
+| 12_2 | Here's a last timetable to solve with the picture. Move the red bus to four minutes after the blue, so the gaps are four and eight. What's the average wait? ⟨predict⟩ | At "Move the red bus" the red markers slide from two minutes after each blue marker to four, and every dot after a red marker and before the next blue turns gold, the rest teal. Then brackets under every gap, "4" in teal and "8" in gold. At "What's the average wait" the question "average wait?" under the caption; countdown, "pause for more time". No triangle and no wait on screen until the countdown ends | moves, brackets, think countdown | |
+| 12_3 | Raise the triangles again, a minute up for every minute across. The gap of four holds half of four squared, eight square minutes. The gap of eight holds half of eight squared, thirty-two square minutes. | The question and the ring go, and so do the brackets after the first twelve minutes. The wait triangles rise over the whole hour, a minute up as long as a minute across: five 4 by 4 in teal and five 8 by 8 in gold, see-through, behind the markers. At "The gap of four" the first short triangle brightens, the triangles after the first twelve minutes dim, and "4 min → ½ × (4 min)² = 8 min²" comes in at the top in teal; at "The gap of eight" the first long triangle brightens and "8 min → ½ × (8 min)² = 32 min²" comes in under it in gold | shapes from the model, rows from facts | |
+| 12_4 | Together they make forty square minutes, over the twelve minutes they cover. Dividing square minutes by minutes leaves minutes. Forty over twelve is three and a third minutes. Three minutes and twenty seconds. | At "Together they make forty square minutes" "8 min² + 32 min² = 40 min²" under the two rows; at "over the twelve minutes they cover" the two brackets give way to one under the whole stretch, "12 min". At "Dividing square minutes by minutes" "average wait = 40 min² ÷ 12 min" lower left, and "min² ÷ min = min" dim under it; at "Forty over twelve" "= 3⅓ min"; at "Three minutes and twenty seconds" "= 3 min 20 s" in green, boxed and ringed. The whole step stands with no voice until it can be read | unit division, verdict box | hold |
+| 12_5 | That's a point on the curve from before, between our timetable and the even timetable. The more even the gaps, the closer the wait comes to three minutes. | Before the voice, chapter 07's plot returns lower right at its place and scale: the offset axis "red bus after blue (min)" with its ticks, the wait axis, the whole U in green and its three points as 07_6 left them, d = 2 (4:20), d = 6 (3:00) and d = 0 (6:00). At "That's a point on the curve" the knob stands at four minutes on the offset axis with a guide up from it, and the U gains a green point at d = 4 (3:20). At "between our timetable and the even timetable" the "4:20" and "3:00" are ringed; at "The more even the gaps" the "3:00" flashes | plot callback from 07 | |
+| 12_6 | And your train? Back on our timetable, four minutes twenty is what the stop costs you on an average morning. | The rows, the steps, the plot and the stretch bracket go, the dots fade and the triangles return to one brightness. Chapter 01's street strip comes back at the top, you at the stop and the train at the station. At "Back on our timetable" the red markers slide back to two minutes after the blue and the triangles reshape with them, five 2 by 2 in teal and five 10 by 10 in gold. At "four minutes twenty" "average cost: 4 min 20 s" in green, lower right | callback to 01, shapes from the model | |
+| 12_7 | But a deadline needs your worst morning, not an average. Turn up just as a red bus leaves, and you wait the whole long gap, so allow the full ten minutes. | At "your worst morning" the gold triangle after the red bus at :02 brightens and the other triangles dim. At "Turn up just as a red bus leaves" you leave the strip and grow back to full size just under the line at :02, and a gold bar, chapter 01's wait as you arrive, grows from the blue bus at :12 back to you: the whole long gap, the triangle's tallest edge. At "allow the full ten minutes" "allow 10 min" under the bar in your color | bar from the model, callback to 01 | |
+| 12_8 | Ten minutes is the worst case only because every bus runs exactly on time. It's a bound in this model, not a guarantee against real delays, or for the whole journey. | At "only because every bus runs exactly on time" a green dot over every marker and "every bus on time", as in 01 and 07. At "It's a bound in this model" a "precisely" card (K10). It stands until it can be read | lower-third | hold |
+| 12_9 | Your bus isn't late. You're just more likely to turn up during a long gap. | Before the voice, K10, the bar and its label go, and you go back to the stop in the strip. At "Your bus isn't late" "every bus on time" is ringed in green. At "You're just more likely" the sixty dots return in their gaps' colors, and the five long gaps shade gold. When the line ends, the lower third "names to search:" with "inspection paradox · random incidence · length-biased sampling · renewal–reward theorem · class size paradox", held until it can be read. Then the picture fades and the receipts card comes up alone, "Every number in this video comes from the code", "and is checked against an independent oracle", "github.com/Dicklesworthstone/jeffs_math_explainer_videos" and "explainer_corpus/inspection_paradox". It stands until it can be read, then the film fades | lower-third, receipts card | hold |
+
+### Notes for 12
+
+- The chapter keeps the film's objects: the hour line and markers of 01,
+  the sixty passengers of 02, the triangles of 05 and 11, the U of 07, the
+  street strip and wait bar of 01. One tracker, the red bus's offset,
+  moves the red markers and repaints the dots in 12_2 and 12_6, and the
+  triangles follow it in 12_6.
+- 12_1 undoes 11's planners on screen. Each dot slides back the way it
+  came, to the moment it was ready. The film is back to arrival moments
+  independent of the timetable and spread evenly in time, K01's condition,
+  for the puzzle and the train (F-W2.per_minute).
+- The transfer question (12_2) is new to the viewer but uses only the
+  picture. Nothing about the answer is on screen until the countdown ends:
+  no triangle, no area, no wait. The triangles come in 12_3, the wait at
+  the end of 12_4. F-Q1 computes the puzzle second by second in the
+  oracle, which checks it against the offset sweep at d = 4.
+- 12_3 and 12_4 count area, as 08 and 11 do. Both axes are minutes, so a
+  triangle's area is in square minutes; the two make forty square minutes
+  (F-Q1.total_area), and dividing by the twelve minutes they cover leaves
+  minutes, 3⅓ min (F-Q1.mean_wait), 3:20 (F-Q1.mean_wait_clock). The words
+  never call the area minutes of waiting. 05_5's bridge, one passenger a
+  minute turning square minutes into minutes of someone's waiting, is a
+  different counting measure and is not used here.
+- New facts F-Q1.short (4) and F-Q1.long (8), the puzzle's two gaps, the
+  shortest and longest of `model::hour_gaps(PUZZLE_OFFSET)`, so the spoken
+  "four" and "eight" of the gaps are bound to the gaps, not to the offset
+  or an area.
+- 12_5's point is the U's at four minutes (`no_bus_late::wait_of`); the
+  scene refuses a U that is not nearer three wherever the gaps are more
+  even, whole minute by whole minute.
+- 12_6 to 12_8 separate the two numbers a traveller needs. The average
+  wait, 4:20 (F-W1.mean_wait_clock), is the cost of an average morning. A
+  deadline needs the longest wait. Turning up just as a red bus leaves you
+  wait the whole long gap, ten minutes (F-W1.allowance,
+  `model::longest_wait`; the oracle takes the worst of the hour's 3,600
+  seconds and rounds up). That bound holds only with every bus exactly on
+  time, and covers the wait at the stop alone.
+- Exact version K10 (12_8): "with every bus exactly on time, the longest
+  wait is the longest gap, 10 min, whenever you turn up; it says nothing
+  about late buses or the rest of the journey".
+- The last line restates the title's claim, "longer than you expect", in
+  the honest direction. The wait is longer than the guess from the sign
+  because of where random moments fall, not because a bus is late.
+- The names and the receipts each get their own reading time: the names
+  over the last picture, the receipts on a card alone after the picture
+  fades.
+- Departures from the earlier draft: its 12_1 (even gaps, three minutes)
+  is 07_3 and 07_4 again and is dropped for the return to random arrivals;
+  its 12_4 called the forty square minutes "forty minutes of waiting"; its
+  12_5 is split into the cost (12_6), the margin (12_7) and the margin's
+  condition (12_8).
+- Pause intentions:
+  - after "What's the average wait?" the countdown, a prediction hold;
+  - after "three and a third minutes." a thought pause, 0.5 to 1.0 s, before
+    the clock reading;
+  - the step after 12_4 is an inspection hold, picture with no voice until
+    it can be read, and the U then returns before 12_5 speaks;
+  - after "And your train?" 0.35 to 0.7 s, the question the film opened
+    with;
+  - after "not an average." 0.4 to 0.8 s, the turn from the cost to the
+    margin;
+  - after "exactly on time." 0.5 to 1.0 s, while the green marks land;
+  - K10 after 12_8, then the names and the receipts after 12_9, are
+    reading holds, and the picture clears K10 before 12_9 speaks.
diff --git a/explainer_corpus/inspection_paradox/oracle.py b/explainer_corpus/inspection_paradox/oracle.py
index 7417c1c..49fe618 100644
--- a/explainer_corpus/inspection_paradox/oracle.py
+++ b/explainer_corpus/inspection_paradox/oracle.py
@@ -517,6 +517,8 @@ def facts():
     p_wait = time_average(wait_at, PUZZLE_OFFSET)
     add("F-Q1.offset", PUZZLE_OFFSET)
     add("F-Q1.gaps", csv(p_gaps[:2]))
+    add("F-Q1.short", min(p_gaps))
+    add("F-Q1.long", max(p_gaps))
     add("F-Q1.short_area", fmt(p_short))
     add("F-Q1.long_area", fmt(p_long))
     add("F-Q1.total_area", fmt(p_short + p_long))
@@ -646,6 +648,8 @@ def self_test():
     # the transfer puzzle against the closed form and the offset sweep
     check("puzzle: 8 + 32 = 40 per 12 minutes, 3:20", (f["F-Q1.total_area"], f["F-Q1.mean_wait_clock"]) == ("40", "3:20"))
     check("puzzle: agrees with the offset sweep at d = 4", f["F-Q1.mean_wait_clock"] == f["F-O1.wait_clocks"].split(",")[PUZZLE_OFFSET])
+    check("puzzle: the gaps are the shortest 4 and the longest 8, the stretch's two",
+          (f["F-Q1.short"], f["F-Q1.long"]) == (4, 8) and f["F-Q1.gaps"] == f"{f['F-Q1.short']},{f['F-Q1.long']}")
 
     # the profiler: as many samples in one 10 ms call as in five 2 ms calls
     check("profiler shares", f["F-P1.long_sample_share"] == "1/2" and f["F-P1.long_call_share"] == "1/6")
diff --git a/explainer_corpus/inspection_paradox/src/chapters/mod.rs b/explainer_corpus/inspection_paradox/src/chapters/mod.rs
index eb3f0aa..bdfc332 100644
--- a/explainer_corpus/inspection_paradox/src/chapters/mod.rs
+++ b/explainer_corpus/inspection_paradox/src/chapters/mod.rs
@@ -3,11 +3,10 @@
 //! the chapter with the same `NN`.
 //!
 //! Only chapters with native scene source are registered, and they are
-//! exactly the chapters of `docs/storyboard.md` and `script.tsv`: chapters
-//! 01 through 11. Chapter 12 is written as words and pictures in
-//! `docs/storyboard-draft.md`, the full draft. It moves into the storyboard
-//! and this registry when root assigns it and its scene source exists, never
-//! as a stand-in.
+//! exactly the chapters of `docs/storyboard.md` and `script.tsv`: all twelve
+//! chapters of the full draft, `docs/storyboard-draft.md`, which holds the
+//! storyboard verbatim. A new chapter moves into the storyboard and this
+//! registry only when its scene source exists, never as a stand-in.
 
 pub mod at_random;
 pub mod back_to_the_stop;
@@ -15,6 +14,7 @@ pub mod draw;
 pub mod glyphs;
 pub mod no_bus_late;
 pub mod one_a_minute;
+pub mod one_more_timetable;
 pub mod same_sum;
 pub mod stretch_one_gap;
 pub mod the_formula;
@@ -84,6 +84,11 @@ pub fn registry() -> Vec<Chapter> {
             make: |kit| Box::new(back_to_the_stop::BackToTheStop { kit }),
             intensity: 0.5,
         },
+        Chapter {
+            key: "12_one_more_timetable",
+            make: |kit| Box::new(one_more_timetable::OneMoreTimetable { kit }),
+            intensity: 0.5,
+        },
     ]
 }
 
diff --git a/explainer_corpus/inspection_paradox/src/facts.rs b/explainer_corpus/inspection_paradox/src/facts.rs
index e441179..22915c3 100644
--- a/explainer_corpus/inspection_paradox/src/facts.rs
+++ b/explainer_corpus/inspection_paradox/src/facts.rs
@@ -443,6 +443,9 @@ pub fn facts() -> Result<Facts, String> {
     let p_wait = model::offset_wait(PUZZLE_OFFSET);
     f.add("F-Q1.offset", PUZZLE_OFFSET)?;
     f.add("F-Q1.gaps", csv(&p_gaps[..2]))?;
+    // 12_3: "The gap of four", "The gap of eight"
+    f.add("F-Q1.short", p_gaps.iter().min().ok_or("the puzzle has no gaps")?)?;
+    f.add("F-Q1.long", p_gaps.iter().max().ok_or("the puzzle has no gaps")?)?;
     f.add("F-Q1.short_area", p_short)?;
     f.add("F-Q1.long_area", p_long)?;
     f.add("F-Q1.total_area", p_short + p_long)?;
@@ -471,6 +474,7 @@ mod tests {
         assert_eq!(f.get("F-W1.allowance_clock"), Some("10:00"));
         assert_eq!(f.get("F-O1.between_readout"), Some("4:01"));
         assert_eq!(f.get("F-Q1.mean_wait_clock"), Some("3:20"));
+        assert_eq!((f.get("F-Q1.short"), f.get("F-Q1.long")), (Some("4"), Some("8")));
     }
 
     /// The facts print one line per key, no key twice (the oracle's format).
diff --git a/explainer_corpus/inspection_paradox/src/main.rs b/explainer_corpus/inspection_paradox/src/main.rs
index 1337b7f..77c3546 100644
--- a/explainer_corpus/inspection_paradox/src/main.rs
+++ b/explainer_corpus/inspection_paradox/src/main.rs
@@ -11,7 +11,7 @@
 //! inspection_paradox help             # the full option list
 //! ```
 //!
-//! Chapters 01 through 11 have scene source; the whole film is
+//! All twelve chapters have scene source; the whole film is
 //! `docs/storyboard-draft.md` (see `chapters/mod.rs`).
 
 mod chapters;
@@ -328,6 +328,33 @@ mod tests {
         ("11_6", "two for each red", "F-K2.per_red", "2"),
         ("11_7", "Five in six would still stand", "F-K2.in_long_share", "5/6"),
         ("11_7", "every wait would be one minute", "F-K2.waits", "1"),
+        // 12: the last timetable, its area over its time, the cost and the
+        // margin. "Forty" and "twelve" are spoken twice in 12_4, once in the
+        // sum and once in the division, so each has a row per phrase
+        ("12_1", "one a minute", "F-W2.per_minute", "1"),
+        ("12_2", "four minutes after the blue", "F-Q1.offset", "4"),
+        ("12_2", "the gaps are four and eight", "F-Q1.gaps", "4,8"),
+        ("12_3", "The gap of four", "F-Q1.short", "4"),
+        ("12_3", "half of four squared", "F-F1.area_factor", "1/2"),
+        ("12_3", "eight square minutes", "F-Q1.short_area", "8"),
+        ("12_3", "The gap of eight", "F-Q1.long", "8"),
+        ("12_3", "half of eight squared", "F-F1.area_factor", "1/2"),
+        ("12_3", "thirty-two square minutes", "F-Q1.long_area", "32"),
+        ("12_4", "forty square minutes", "F-Q1.total_area", "40"),
+        ("12_4", "the twelve minutes they cover", "F-T1.period", "12"),
+        ("12_4", "Forty over twelve", "F-Q1.total_area", "40"),
+        ("12_4", "over twelve is", "F-T1.period", "12"),
+        ("12_4", "three and a third minutes", "F-Q1.mean_wait", "10/3"),
+        (
+            "12_4",
+            "Three minutes and twenty seconds",
+            "F-Q1.mean_wait_clock",
+            "3:20",
+        ),
+        ("12_5", "closer the wait comes to three minutes", "F-O1.min_wait", "3"),
+        ("12_6", "four minutes twenty", "F-W1.mean_wait_clock", "4:20"),
+        ("12_7", "allow the full ten minutes", "F-W1.allowance", "10"),
+        ("12_8", "Ten minutes is the worst case", "F-W1.allowance", "10"),
     ];
 
     /// The words the guard below listens for: every number word the full
````
