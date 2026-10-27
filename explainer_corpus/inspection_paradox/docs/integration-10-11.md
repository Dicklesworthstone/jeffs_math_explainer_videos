# Chapters 10 and 11: native source and integration

2026-10-24, GoldGate, for root. This note covers the native scene source for
`10_same_sum` and `11_back_to_the_stop`, which sits in two new files.
Nothing registers them yet, so the build does not compile them.

None of it has been compiled, rendered, narrated or listened to. Every check
below is a Python model, a rustfmt parse or a source reading, not native
execution.

## Files

| file | sha256 (first 16) | bytes | what it is |
|---|---|---|---|
| `src/chapters/same_sum.rs` | `6a6cf394f33a5205` | 66106 | chapter 10, `SameSum` |
| `src/chapters/back_to_the_stop.rs` | `3c98fc169d961cbd` | 68150 | chapter 11, `BackToTheStop` |

No shared helper was added or edited.

- **10** uses:
  - chapter 04's stretch from `twelve_minutes.rs`: `stretch`, `gaps`,
    `in_gap`, `Passenger`, `fade_in_flashing` and `pick`;
  - `two_averages::{level, written}`, `timetable` and `draw`.
- **11** uses `draw`, `timetable`, `triangles`, `twelve_minutes::grow_lagged`
  and `two_averages::written`. Its test `the_street_is_chapter_01s` reads
  `the_stop.rs`.
- Neither uses 08's or 09's code.

### The base

The patch is against the live production files after root's 08/09
integration, read at 12:46Z. Root's `tend-1232` was still running then.

| file | sha256 (first 8) |
|---|---|
| `main.rs` | 6d93ba32 |
| `chapters/mod.rs` | 5db91597 |
| `script.tsv` | 95cdcab9 |
| `docs/storyboard.md` | b310188d |
| `docs/storyboard-draft.md` | de5591dd |
| `docs/pause-overrides.json` | 0d425e24 |
| `draw.rs` | e51741b3 |
| `timetable.rs` | 66c43d84 |
| `glyphs.rs` | 7aacf404 |
| `model.rs` | bc3be4ad |
| `facts.rs` | a3cd5b3e |
| `palette.rs` | aeb18f55 |
| `oracle.py` | 47a2c058 |
| `the_stop.rs` | 1432b763 |
| `twelve_minutes.rs` | 61206a6f |
| `triangles.rs` | dc74a296 |
| `two_averages.rs` | 78325de4 |

- I first built the patch on production plus my own 08/09 patch. Root's
  integration then changed:
  - 08_1's words, its SPOKEN row and the 08 picture text;
  - the overrides file, which gained six entries in a compact layout.
- So I rebuilt against the live files. The overrides now go in as text in
  that compact layout, not re-dumped JSON, and every earlier entry is
  checked unchanged.
- `the_formula.rs` (55876aff) and `at_random.rs` (f43764c1) also changed
  during root's integration. Nothing here depends on them.

If root changes these five files again before applying, the hunks anchor
on:
- the 09 registry entry and the module list (`mod.rs`);
- the module doc line and the last 09 SPOKEN row (`main.rs`);
- the end of 09's pause notes (`storyboard.md`);
- the draft's 10/11 sections, between 09's notes and the 12 heading;
- the `_comment` and the last override (`pause-overrides.json`).

`git apply --3way` or the steps below by hand would then do.

## Integration, in order (root's)

1. **Apply the patch** at the end of this note, from the repo root:
   `git apply --check`, then `git apply`. It makes five edits:
   - **`src/chapters/mod.rs`:**
     - adds `pub mod back_to_the_stop;` and `pub mod same_sum;` in
       alphabetical order;
     - registers `10_same_sum` (`same_sum::SameSum`, intensity 0.45) and
       `11_back_to_the_stop` (`back_to_the_stop::BackToTheStop`, intensity
       0.5) after 09;
     - rewraps the module comment to "01 through 11. Chapter 12 is
       written as words and pictures in …".
     - The intensities are proposals. 10 is a detour with reading holds.
       11 holds the film's turn.
   - **`src/main.rs`:**
     - appends the 28 SPOKEN rows of step 3 after 09's;
     - changes the crate doc's "Chapters 01 through 05 have scene source"
       to "01 through 11". That line has been stale since 06 registered;
       root may prefer other words.
   - **`docs/storyboard.md`:** appends the two authored sections, notes
     included:

     | section | sha256 (first 16) | bytes |
     |---|---|---|
     | `## 10_same_sum: The same sum elsewhere (~98 s)` | `5e45aa3305c75ef3` | 8594 |
     | `## 11_back_to_the_stop: Back to the stop (~81 s)` | `53d31448dfdeacea` | 7774 |

     Each hash covers `"\n## "` plus the section, with trailing
     whitespace stripped. That is what
     `the_full_draft_holds_the_storyboard_verbatim` compares. The `~98 s`
     and `~81 s` are storyboard.py's estimates, not limits (see Timing).
   - **`docs/storyboard-draft.md`:** replaces the draft's 10 and 11
     sections with the same authored text, so the verbatim test holds.
     - Chapter 12 is unchanged.
     - The draft's chapter table is unchanged. Its statuses for 02–09
       were already stale, and the table is root's.
   - **`docs/pause-overrides.json`:**
     - appends the five entries of step 4 in the file's compact layout;
     - changes `_comment` to chapters 01 to 11, with chapter 12 still in
       the draft.

   Patched files, if applied to the hashes above:

   | file | before | after | bytes after |
   |---|---|---|---|
   | `src/chapters/mod.rs` | 5db91597 | db225ada | 3065 |
   | `src/main.rs` | 6d93ba32 | 4eb50f22 | 21743 |
   | `docs/storyboard.md` | b310188d | 493f2ff4 | 59002 |
   | `docs/storyboard-draft.md` | de5591dd | c819fa96 | 69318 |
   | `docs/pause-overrides.json` | 0d425e24 | 4f762187 | 8994 |

2. **`script.tsv`:** run `python3 -I tools/storyboard.py inspection_paradox`
   (this slug only), then
   `python3 -I tools/storyboard.py --check inspection_paradox`. This adds
   the sixteen lines 10_1 to 11_7.
   - I did not run storyboard.py's main. I called its own `parse`,
     `check` and `script_text` on scratch copies.
   - On the base storyboard, that reproduces the live `script.tsv` byte
     for byte (95cdcab9…, 9915 bytes).
   - On the patched storyboard, the sixteen appended lines hash to
     `343f11db1c144aa8`. The whole file hashes to `355d3ce4db895c26`
     (12305 bytes, 83 lines plus the header).
   - Treat these as expectations, not receipts.

3. **Numeric bindings (`SPOKEN`, in the patch).** The rows are rustfmt's
   layout for this repo. Checked as main.rs's
   `every_spoken_number_is_the_fact_it_speaks_for` checks them:
   - Each phrase is in its line, and each value is the oracle's value for
     its key.
   - The guard passes for all 16 lines. 10_1, 10_3, 10_4 and 10_8 say no
     number word, so they have no row.
   - A stricter check finds every number word inside some row's phrase.
   - All keys are added literally in `facts.rs`.
   - No line uses a `PRONOUN_ONES` phrase, so that list needs no change.
   - Over the whole patched SPOKEN (186 rows) on the emulated script,
     the test's three checks pass.
   - Four negative controls are caught: a wrong value, a missing phrase,
     a dropped line and an invented key.

   Bindings worth a second look:
   - The "one" of "one ten millisecond call" (10_7) is the loop's single
     long call. `facts.rs` writes it as the 1 in
     `F-P1.long_call_share = 1/(SHORT_CALLS + 1)`, and no fact names it.
     The row binds the ten (F-P1.long_ms). A fact `F-P1.long_calls` would
     make it explicit, but `facts.rs` is root's.
   - In "one of two students and one of ten" (10_2), "one" means one
     class. The row binds both sizes (F-C1.sizes, "2,10").
   - "Ten of the twelve students" (10_5) has two rows:
     F-C1.big_class_students for the ten, F-C1.students for the twelve.
   - "over a century ago" (10_9) is bound to F-H1.over_a_century, as the
     draft's oracle intends.
   - 11_4 and 11_6 bind their "one minute before …" to F-K1.early. 11_5's
     "One minute" is F-K1.wait, and 11_7's is F-K2.waits.

4. **Pause overrides (in the patch).** The ranges are the sections'
   pause notes, carried over from the draft. They are editorial
   hypotheses, not measurements.

   | cue | after | default | authored |
   |---|---|---|---|
   | 10_5 | "Eight and two thirds" | thought 0.35–1.1 | thought 0.4–0.9 |
   | 10_7 | "For programmers" | thought 0.35–1.1 | thought 0.35–0.7 |
   | 11_5 | "One minute" | thought 0.35–1.1 | thought 0.4–0.8 |
   | 11_6 | "And if every passenger planned" | question 0.65–1.65 | question 0.35–0.7 |
   | 11_7 | "every wait would be one minute" | thought 0.35–1.1 | thought 0.5–1.0 |

   - I ran each entry through `tools/narration_director.py`'s own
     `directions` on the patched draft's 89 lines. Each names exactly one
     boundary, not a cue end, and replaces the punctuation default there.
   - 11_6 keeps the role `question`: the next sentence answers it at once,
     so only the range is shorter.
   - Two negative controls were refused: "minute", which occurs twice in
     11_7, and a phrase not in 10_5.
   - The picture holds below sit at cue ends. They are scene waits, not
     narration gaps, so no entry was added for them.

5. **Glyphs.** Every character the two files' literals print is already
   printed by a registered chapter: ² × ÷ −, and the – of the
   Feld & Grofman pages.
   - Fractions come from `two_averages::written`.
   - The ⅔ of "8⅔" may be its first use on screen. It is not in
     `glyphs::DIGITS`, but the probe collects what registered scenes
     typeset, so no `glyphs.rs` edit is needed for it to be probed.
   - At pin `a399d626` the fallback chain is Computer Modern → Noto Sans
     Math Symbols → IBM Plex Sans. Only IBM Plex Sans Regular maps ⅔, as
     it does ½ ⅓ ⅙ ⅚ and ², which registered chapters already print.
     Computer Modern maps ÷ and –, not ⅔, ² or ×.
   - I read the cmaps of the lockfile's fmd-font copies with a small
     format-4/12 reader. That is a cmap reading, not a probe run.

Then root's usual order: `cargo fmt --check`, Clippy with `-D warnings`,
the crate's tests, `inspection_paradox glyphs`, and a preview.

### Cue IDs and what each waits for

Every phrase occurs once in its line, after the previous one. The 25 + 23
phrases are parsed from the construct bodies.

- **10_1:** "Whenever you pick a person or a moment"; "instead of a group";
  "the big groups get picked more often".
- **10_2:** "seat those twelve passengers"; "two classes"; "one of two
  students"; "and one of ten"; "The school says"; "six students".
- **10_3:** "Ask every student in turn".
- **10_4:** "What's the average answer".
- **10_5:** "It's the bus sum again"; "students in place of minutes"; "Ten
  of the twelve students".
- **10_6:** "the class size paradox".
- **10_7:** "checks what your program is doing"; "one ten millisecond
  call"; "five two millisecond calls"; "half the samples"; "a sixth of
  the calls".
- **10_8:** "aren't in step"; "Checks that tick exactly with its loop";
  "land in the same function every time"; "the shares come out completely
  wrong".
- **10_9:** "In nineteen twenty-two".
- **11_1:** "Ten buses an hour"; "all on time"; "a bus every six minutes on
  average"; "All of that is true".
- **11_2:** "you don't sample buses"; "You sample a moment"; "Five times in
  six"; "where the waits are long".
- **11_3:** "The area of the triangles"; "divided by the time they cover";
  "Fifty-two square minutes"; "Four minutes and twenty seconds".
- **11_4:** "turn up one minute before your bus"; "How long do you wait".
- **11_5:** "the triangles' average doesn't describe you".
- **11_6:** "with a minute to spare"; "and stand at the stop"; "Ten would
  wait for each blue bus"; "two for each red".
- **11_7:** "every wait would be one minute"; "They'd all stand in the last
  minute of a gap"; "the triangles' average wouldn't describe them";
  "Change how passengers arrive".

### Authored semantic holds

These are silent windows the scenes author. The durations come from the
code, and the reading times are `read_s` (characters ÷ 15).
`narration_director`'s `visual_holds` needs an evidence file and its hash,
so none is proposed as JSON until a preview exists. The candidates, by
adjacent cues:

| between | hold | why |
|---|---|---|
| 10_3 → 10_4 | the asking: 3 students at 0.9 s, then 9 at 0.5 s (7.2 s), then the caption's 0.4 s; it runs on past the line (about 2.8 s of picture after the breath, on the scratch voice) | twelve individual answers before the question. **Never a coherent seam:** the kit refuses one with animation overrun |
| 10_4 → 10_5 | `think`, 4 s ring | prediction |
| 10_6 → 10_7 | until the source card is read (148 characters, 9.9 s from its fade-in), then the 1.0 s flight back to the corner | readable attribution |
| 10_8 → 10_9 | until K08 is read (140 characters, 9.3 s) | K08's two conditions |
| 10_9 → end | until the Terada card is read (126 characters, 8.4 s; at least `STAND_UP`), then the fade | readable attribution |
| 11_3 → 11_4 | until the verdict is read (43 characters, 2.9 s from its ring) | the boxed 4 min 20 s |
| 11_4 → 11_5 | `think`, 4 s ring | prediction |
| 11_7 → end | until K09 is read (207 characters, 13.8 s), then the fade | K09's conditions |

## How the source meets the assignment

**The same people and the same timetable as the bridge (10).**

- 10_1 opens on chapter 04's twelve-minute stretch exactly as it was: its
  twelve passengers on the line, two teal in the short gap and ten gold in
  the long one, under the two brackets.
  - The dots flash at "a person or a moment" and the brackets at "instead
    of a group". At "the big groups" the ten gold dots and the long
    bracket flash.
  - The picture then shrinks into the top-left corner, every dot with it.
- In 10_2 the same twelve dots fly out of the corner one by one and grow
  into students: the two teal into the small room, the ten gold into the
  big one.
- In 10_6 they fly back into the corner as passengers. In 10_9 the corner
  picture grows back to its place.
- `school` derives the class sizes from the stretch's gaps' passenger
  counts (`twelve_minutes::in_gap`). It refuses any of F-C1.classes,
  F-C1.sizes, F-C1.students, F-C1.big_class_students, F-C1.school_mean,
  F-C1.student_mean or F-S1.person_gap when it stops matching.
- So each student's answer is the gap its passenger stood in. The
  students' average is 06's passengers' gap (F-S1.person_gap, 8:40), which
  appears under the corner picture in 10_5 while its brackets flash.
- Test: `the_students_are_the_stretchs_passengers`.

**Twelve individually sampled students before the equation; the answer
protected.**

- 10_2's one-bar-per-class row ("classes of 2 and 10", level 6, "school's
  average: (2 + 10) ÷ 2 = 6") comes first.
- In 10_3 each student is asked in turn, the first three slowly.
  - A tag under the student says 2 or 10.
  - A bar as tall as the answer is born at the tag and joins the lower
    row, captioned "one bar per student" and "2 say 2, 10 say 10".
- 10_4 asks "What's the average answer?" with the twelve bars up and the
  countdown running.
- The level, the 8⅔ and "students' average: (2 × 2 + 10 × 10) ÷ 12 =
  104 ÷ 12 = 8⅔" are first built in 10_5, after `think` returns.
  - That order is in the construct body, a source reading. No test
    asserts their absence.
  - `the_picture_stays_apart_and_on_screen` lays out the question moment
    with the school's sum and level only.
- The question is its own cue because the twelve answers outlast "Ask
  every student in turn how big their class is." (see Departures).

**Time share kept apart from call share; the sampling assumptions
qualified (10_7, 10_8).**

- The program is an example, not a measurement: a loop of one 10 ms call
  and five 2 ms calls (F-P1), three loops drawn.
- Each tally says both shares:
  - "10 ms call: ½ of the time, ⅙ of the calls", in gold;
  - "2 ms calls: ½ of the time, ⅚ of the calls", in teal;
  - then "checks landing in the 10 ms calls: ½".
- The voice says "it measures where the time goes, not how often each
  function is called" and "half the samples, but only a sixth of the
  calls".
- "On average" is said because only some clocks give the time share:
  - The drawn even checks, every 3 ms from 1.5 ms, share no factor with
    the 20 ms loop. Over whole loops exactly half of them land in the long
    call.
  - Checks at random moments land there half the time only on average.
- 10_8 shows the wrong clock. Checks in step with the loop, one a loop, all
  land in the 10 ms calls. Started a long call later, none do.
- K08 says the conditions: "on average and over a long run, samples taken
  often and not in step with the program fall in each function in
  proportion to its running time".
- No sampling clock is promised the time share exactly. Tests:
  - `the_profiler_measures_time_not_calls`;
  - `a_clock_in_step_with_the_loop_gets_the_shares_wrong`: all, none, and
    a 4 ms clock giving 3/5 or 2/5 by where it starts. `all_or_none`
    refuses 3/5;
  - `random_checks_measure_time_on_average`: 20,000 fixed-seed xorshift
    moments, within 0.02 of ½ and more than 0.2 from ⅙.

**The original stop, and the computed `planned_arrivals` (11).**

- 11_1 rebuilds chapter 01's picture as 01_3 left it, from chapter 01's
  own `street`: the strip, halved at the top, with you at the stop, the
  hour line, both routes' markers and the sign.
  - `the_street_is_chapter_01s` compares the function body and the
    street constants with `the_stop.rs`, byte for byte.
- 11_2 brings back 02's tags and sixty passengers, then the wait
  triangles over the whole hour, to scale
  (`the_triangles_are_the_models_to_scale`).
- In 11_4 you leave the strip for :25, one minute before the red bus at
  :26. That bus is computed (`your_bus`), the red bus nearest the middle
  of the hour.
- 11_5's wait is the triangle's own height at your moment
  (`the_timetable_reader_waits_one_minute`).
- The planners in 11_6 are `model::planned_arrivals(OFFSET)`.
  - Each dot slides forward, never back, to one minute before the first
    bus it can make with a minute to spare.
  - The two dots in every twelve minutes that are ready less than a
    minute before a bus flash first (`too_late`), and they slide on to the
    next bus.
  - The stacks form from the dots and the timetable alone: ten before
    each blue bus and two before each red. The dot with least far to
    slide goes on top.
  - `the_planners_stand_a_minute_before_the_first_bus_they_can_make`
    checks every planner by brute force over −1 to 2 hours, with stacks
    per F-K2.per_blue and F-K2.per_red. `the_slide_never_goes_back`
    checks the slide.
  - The dot at :59½ runs off the end at :60 and comes in again at :00, to
    stand at :01.

**The counterexample kept (11_7).**

- The panel's "share of passengers in long gaps" bar stays at ⅚ and is
  ringed. F-K2.in_long_share is checked equal to F-T1.long_time_share.
- The "average wait" bar falls steadily from 4 min 20 s to a minute.
  "1 min" and every stack's "wait 1 min" come in during the fall.
- Then the last minute of every gap is drawn over the line, and the gold
  triangles all but vanish.
- `the_same_share_of_long_gaps_can_wait_differently` keeps the draft's
  rejected alternative as a control. Six to a bus also waits a minute,
  but with only half in long gaps. So the share alone never gives the
  wait.

**Conditions visible; transitions open to inspection.**

- K08 and K09 are held until read.
- Every dot keeps its identity through the slide and takes the color of
  the gap it moves through.
- The bar falls linearly (`steady`).
- 11_3's stretch brightens while the other four dim, labelled "the same ×
  5". The area labels and the 12 min bracket come before the box.

**No compression.**

- storyboard.py estimates 98.2 s and 81.1 s. The scratch-voice emulation
  gives 116.5 s and 96.7 s.
- Neither was cut toward the draft's 72 s and 76 s.

**Attribution cards readable.**

- Feld & Grofman: 148 characters, 2 lines.
- Terada: 126 characters, 2 lines.
- Each is held for `read_s` from its fade-in (9.9 s and 8.4 s). The
  layout tests keep each in its lower-third band.

**Source-derived numbers and wrong-model controls.**

- Every number on screen is read from the facts or counted from what is
  drawn.
- `school`, `profile` and `stop` refuse a fact that stops matching.
- `the_numbers_are_the_models_and_a_wrong_fact_is_refused` tests eleven
  refusals in 11. Most are a wrong model's value:
  - six a bus (F-K2.per_blue and F-K2.per_red "6");
  - half in long gaps (F-K2.in_long_share "1/2");
  - the random arrivals' 13/3 for the planners and for you (F-K2.waits,
    F-K1.wait);
  - a planner early (F-K2.never_early "false");
  - standing at the bus (F-K2.stand_at "0,10");
  - half the gap ahead (F-K2.wait_share_of_gap "1/2");
  - the triangles' 50 and 6:00 (F-A1.total_area, F-W1.mean_wait_clock);
  - the planners' sixth for random arrivals (F-S1.wait_share_of_gap
    "1/6").
- K09's "2" comes from F-S1.wait_share_of_gap's denominator, and `stop`
  refuses a numerator other than 1.

## Departures from the draft

Words (all checked against SPOKEN and the phrase lists):

- **10_2** "Take a school …" became "Now seat those twelve passengers in a
  school …": the same people as the bridge.
- **10_3** became "Ask every student in turn how big their class is." The
  question "What's the average answer?" is its own cue, 10_4, because the
  twelve answers outlast the line. So 10 has nine cues, and the draft's
  10_4 to 10_7 are now 10_5, 10_6, 10_7 and 10_9.
- **10_7** (the draft's 10_6) says "checks what your program is doing" and
  "how often each function is called". It adds "On average," and "half
  the samples, but only a sixth of the calls".
- **10_8** is new. It is the aliasing line, with K08 moved there.
- **11_3** "Fifty-two over twelve" became "Fifty-two square minutes over
  twelve minutes", carrying root's 04/05 unit bridge.
- **11_5** "the triangles don't describe you" became "the triangles'
  average doesn't describe you".
- **11_7** "Nobody would be spread through a gap any more, so the
  triangles wouldn't describe them" became "They'd all stand in the last
  minute of a gap, so the triangles' average wouldn't describe them". It
  says what the picture shows.

Pictures:

- **10_1** starts from chapter 04's stretch, not a generic bus picture.
- 10's two bar rows, one bar per class and one per student, are new.
- The passengers' gap appears under the corner picture, not beside the
  sum.
- **10_7**'s tallies say "of the time", not "of samples": they describe
  the program. A third line counts the checks.
- **K08** gained "on average and over a long run".
- **11_3**'s box is "average wait: 52 min² ÷ 12 min = 4 min 20 s", area
  over time in their units. 08's letter formula is not repeated.
- **11_6** brings the share/wait panel in. The draft first shows bars in
  11_7.
- **11_7** draws the last minute of every gap. The draft left the empty
  part of each long gap bare. The gold triangles all but vanish (opacity
  0.05) instead of dimming.

## Tests

- **`same_sum.rs`** (10):
  - `the_chapter_runs_and_says_its_lines_in_order` (10_1 … 10_9)
  - `the_students_are_the_stretchs_passengers`
  - `the_two_averages_are_the_buses_and_the_passengers`
  - `the_profiler_measures_time_not_calls`
  - `a_clock_in_step_with_the_loop_gets_the_shares_wrong`
  - `random_checks_measure_time_on_average`
  - `the_printed_words_are_the_drafts`
  - `the_citations_are_the_drafts`
  - `the_fixed_words_carry_no_numbers`
  - `the_picture_stays_apart_and_on_screen`
- **`back_to_the_stop.rs`** (11):
  - `the_chapter_runs_and_says_its_lines_in_order` (11_1 … 11_7)
  - `the_street_is_chapter_01s`
  - `the_numbers_are_the_models_and_a_wrong_fact_is_refused`
  - `the_timetable_reader_waits_one_minute`
  - `the_planners_stand_a_minute_before_the_first_bus_they_can_make`
  - `the_same_share_of_long_gaps_can_wait_differently`
  - `the_slide_never_goes_back`
  - `the_triangles_are_the_models_to_scale`
  - `the_printed_words_are_the_drafts`: K09, three lines
  - `the_fixed_words_carry_no_numbers`
  - `the_picture_stays_apart_and_on_screen`

Only the two `the_chapter_runs_and_says_its_lines_in_order` tests need
integration, because they read `script.tsv`. Each plays its whole chapter
headless with the scratch voice.

## Checks run (no Cargo)

- **rustfmt:**
  - `rustfmt --edition 2024 --check` with the repo's `rustfmt.toml` is
    clean on both files.
  - It is also clean on a scratch copy of the whole crate with the patch
    applied. That run parses every module from `main.rs`, the two new ones
    included, as registered.
  - Negative control: a broken `back_to_the_stop.rs` in a second copy
    fails it.
- **The patch:**
  - `git apply --check` passes against the live tree, which it does not
    modify.
  - Applied to a scratch copy of the crate, it gives exactly my built
    copies of the five files.
  - The patched `pause-overrides.json` parses.
- **Integration emulation:** the crate's tests emulated in Python on the
  patched copies, with storyboard.py's own functions:
  - `check_video`: storyboard keys = registry keys, script lines =
    storyboard lines, and every line owned by a registered chapter;
  - the verbatim draft (14 sections);
  - SPOKEN over all 186 rows.
  - Three negative controls are caught: a stale script, a missing
    registry key and a changed section.
- **Static scans:**
  - no unused imports (`Timed` supplies `.rt()`, as in `two_averages.rs`);
  - no unused bindings: the scan's eight hits were inline format
    arguments, `..` ranges and a shadowing `let`'s right-hand side.
- **API, read at pin `a399d626`:**
  - `Stage::scale` scales about the box centre. 11 halves chapter 01's
    street group with it, then moves it to the top.
  - `set_z_index` does not re-sort a mobject already in the scene. So
    11's triangles (detached when made) and your segment get their z
    before they are added.
  - 10's corner moves are `draw::shrink_to`: AnimBuilder `.scale` then
    `.move_to`. They apply to the stretch picture and to each dot
    separately, so the dots can leave for the school.
  - `clear_updaters(mob, true)` stops `live_bar`'s `always_redraw`; 11
    clears it once the bar has fallen.
  - Every fmn, kit and topic-helper method or function the two files call
    is also called by a registered chapter that root compiled. The only
    other names are the files' own methods and `std`'s `div_euclid` and
    `min_by_key`.
  - That matches names, not signatures.
- **Phrases, facts and cards:**
  - 9 + 7 lines are said in the sections' order, and the 25 + 23 waits
    hold as listed.
  - All 41 fact keys the files name are oracle keys.
  - K08 (140 characters) and K09 (207, with the 2 from a fact) are the
    sections' exact versions.
  - Four negative controls are caught.
- **SPOKEN rows and pause overrides:** as in steps 3 and 4.
- **Python port** of the tests' arithmetic, text and geometry: 13,243
  checks pass. Constants are parsed from the Rust sources, and arithmetic
  is exact where the Rust's is. It covers:
  - the school, the profiler's shares and all three clocks;
  - the planners by brute force, and the six-to-a-bus control;
  - the slide, sampled at 200 steps a dot;
  - the printed words against the sections;
  - 10's six moments and 11's seven moments, apart and on screen;
  - 10's panel holding its contents;
  - 11's labels and panel clear of the triangles' tops;
  - "50 min²" inside its triangle;
  - your guide line clear of the dots, the clock labels and your words.

  Six port negative controls are caught: an overlap, a changed K09, the
  six-to-a-bus share, an in-step clock, triangles 0.2 taller, and "of
  samples" in a tally.
- **Oracle:** `oracle.py --self-test` reports 0 failures. Its 166 facts are
  the ones the checks used.

## Timing

These figures are the scratch voice at 2.6 words a second plus 0.4 s a
line, with a 0.6 s breath. The beats are transcribed by hand from the code.
They are emulations, not measurements.

| | storyboard.py's rule | scratch-voice emulation | of which silent picture holds after a line's breath |
|---|---|---|---|
| 10 | 98.2 s | 116.5 s | 10.3 s (10_3 2.8, 10_6 3.4, 10_8 7.9) |
| 11 | 81.1 s | 96.7 s | 10.1 s (11_3 0.8, 11_7 9.3) |

The `hold` in 10_3's, 10_9's and 11_7's hold/listen cells marks those
picture holds for the storyboard estimate. The kit uses the cell only in
the animatic.

These pictures land after their phrase:

| line | phrase | late by | why |
|---|---|---|---|
| 11_2 | "you don't sample buses" | 0.09 s | "every bus on time" and its dots fade first |
| 11_6 | "Ten would wait for each blue bus" | 0.09 s | it follows the 3.5 s slide |

## Unresolved

1. **Uncompiled.** These are the calls most worth watching in the first
   build:
   - 10's corner shrink and regrowth, with the stretch picture and each
     dot scaled about their own centres to computed spots;
   - the students' `shrink_to` from born bars;
   - 11's slide, whose :59½ dot wraps from :60 to :00;
   - 11's `live_bar` tracker and `clear_updaters` after the fall;
   - `fade_in_with` mixing fades with `steady`'s builder.
2. **Sounding seams.** 10_3 → 10_4 runs about 2.8 s of picture past the
   line, so it must never be contracted as a coherent seam (see the
   holds). The reading holds after 10_6, 10_8 and 11_3 also make a pair
   non-contiguous.
3. **Facts.** The loop's single long call has no fact of its own (step
   3). `facts.rs` is root's.
4. **Text boxes.** Box sizes in the layout tests are estimates (0.0045
   units a point a character), not measured glyph boxes.
5. **Glyphs.** ⅔ is unprobed until 10 is registered.
6. **Pronunciation.** Feld, Grofman, Torahiko Terada and "millisecond" are
   still to probe, as the sections' notes say.
7. **Intensities.** 0.45 and 0.5 are proposals.
8. **No listening claims.** Nothing here proves rendered clarity, natural
   pacing or English-accent consistency.

## The patch

The patch has 413 lines, sha256 `c42705490f51d03c` (first 16), and covers
the text between the fences. It was made with `git diff --no-index`
against copies of the five live files at the hashes above.

````diff
diff --git a/explainer_corpus/inspection_paradox/docs/pause-overrides.json b/explainer_corpus/inspection_paradox/docs/pause-overrides.json
index 13fbfb5..bcd9e95 100644
--- a/explainer_corpus/inspection_paradox/docs/pause-overrides.json
+++ b/explainer_corpus/inspection_paradox/docs/pause-overrides.json
@@ -1,5 +1,5 @@
 {
-  "_comment": "Authored pause intentions for the active script (chapters 01 to 09), in tools/narration_director.py's override format, to splice into a map's \"overrides\" after `plan`. Each 'after' phrase occurs once in its cue. Ranges are editorial hypotheses from the storyboard's notes, not measurements or acceptance; nothing has been narrated or listened to. The chapter's prediction countdown and title hold are picture holds in the scene, not narration gaps. Chapters 10 to 12 have their pause intentions in docs/storyboard-draft.md and join here with their scenes.",
+  "_comment": "Authored pause intentions for the active script (chapters 01 to 11), in tools/narration_director.py's override format, to splice into a map's \"overrides\" after `plan`. Each 'after' phrase occurs once in its cue. Ranges are editorial hypotheses from the storyboard's notes, not measurements or acceptance; nothing has been narrated or listened to. The chapter's prediction countdown and title hold are picture holds in the scene, not narration gaps. Chapter 12 has its pause intentions in docs/storyboard-draft.md and joins here with its scene.",
   "overrides": [
     {
       "video": "inspection_paradox",
@@ -261,6 +261,31 @@
       "video": "inspection_paradox", "cue": "09_8", "after": "four twenty",
       "role": "thought", "range_s": [0.5, 0.9],
       "reason": "Let the three named cases settle before extending the scale past six."
+    },
+    {
+      "video": "inspection_paradox", "cue": "10_5", "after": "Eight and two thirds",
+      "role": "thought", "range_s": [0.4, 0.9],
+      "reason": "The prediction's answer: the students' level settles at 8⅔ before the bus sum is named."
+    },
+    {
+      "video": "inspection_paradox", "cue": "10_7", "after": "For programmers",
+      "role": "thought", "range_s": [0.35, 0.7],
+      "reason": "A label for the turn from the school to the program, not a sentence the voice leans on."
+    },
+    {
+      "video": "inspection_paradox", "cue": "11_5", "after": "One minute",
+      "role": "thought", "range_s": [0.4, 0.8],
+      "reason": "The prediction's answer: your one-minute wait lands before the reason the triangles miss it."
+    },
+    {
+      "video": "inspection_paradox", "cue": "11_6", "after": "And if every passenger planned",
+      "role": "question", "range_s": [0.35, 0.7],
+      "reason": "A question the next sentence answers at once: shorter than a question's default, before the dots move."
+    },
+    {
+      "video": "inspection_paradox", "cue": "11_7", "after": "every wait would be one minute",
+      "role": "thought", "range_s": [0.5, 1.0],
+      "reason": "The wait bar falls to a minute while the five-in-six share stays; the fall lands before the reason."
     }
   ]
 }
diff --git a/explainer_corpus/inspection_paradox/docs/storyboard-draft.md b/explainer_corpus/inspection_paradox/docs/storyboard-draft.md
index 8aada26..282c7f8 100644
--- a/explainer_corpus/inspection_paradox/docs/storyboard-draft.md
+++ b/explainer_corpus/inspection_paradox/docs/storyboard-draft.md
@@ -504,21 +504,33 @@ who plan.
   - "Six minutes." 0.6 to 1.2 s, the chapter's surprise;
   - after "The buses have no memory." 0.45 to 0.9 s.
 
-## 10_same_sum: The same sum elsewhere (~72 s)
+## 10_same_sum: The same sum elsewhere (~98 s)
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 10_1 | The same sum turns up far from bus stops. Whenever you pick a person or a moment instead of a group, the big groups get picked more often. | The bus picture shrinks into a corner, and two empty rooms appear | morph | |
-| 10_2 | Take a school with two classes, one of two students and one of ten. The school says the average class has six students. | Two rooms, 2 teal dots and 10 gold dots; "school's average: (2 + 10) ÷ 2 = 6" | dots, equation card | |
-| 10_3 | Ask every student how big their class is. What's the average answer? ⟨predict⟩ | Each dot gets a tiny tag, 2 or 10. Question card "students' average?"; countdown | think countdown | |
-| 10_4 | Eight and two thirds. It's the bus sum again, with students in place of minutes. Ten of the twelve students sit in the big class. | "(2 × 2 + 10 × 10) ÷ 12 = 104 ÷ 12 = 8⅔". Beside it, ghosted from 06, "passengers' gap: 8 min 40 s" | equation card, callback | |
-| 10_5 | Scott Feld and Bernard Grofman wrote about this in nineteen seventy-seven and called it the class size paradox. | Lower-third: "Feld & Grofman, 'Variation in class size, the class size paradox, and some consequences for students', Research in Higher Education 6 (1977) 215–222" | lower-third | |
-| 10_6 | For programmers. A sampling profiler checks your program many times a second, so it measures where the time goes, not how often each function runs. One ten millisecond call shows up as often as five two millisecond calls put together. | A "for programmers" tag. A code panel slides in from the right: a timeline with one 10 ms bar and five 2 ms bars, sample ticks falling evenly; tallies "10 ms call: ½ of samples, ⅙ of calls" and "2 ms calls: ½ of samples, ⅚ of calls". A "precisely" card (K08) | code panel, tallies from facts, lower-third | |
-| 10_7 | And people noticed it over a century ago. In nineteen twenty-two the physicist Torahiko Terada wrote an essay about waiting for Tokyo's trams, with his own observations of the trams. | Lower-third: "Torahiko Terada, physicist, essay of 1922; English translation and discussion: N. Masuda & T. Hiraoka, arXiv:2004.14081 (2020)". It stands, then the chapter fades | lower-third | |
+| 10_1 | The same sum turns up far from bus stops. Whenever you pick a person or a moment instead of a group, the big groups get picked more often. | Chapter 04's stretch returns as it was: its twelve passengers on the line, two teal in the short gap and ten gold in the long one, under the two gaps' brackets. At "a person or a moment" the dots flash one by one, at "instead of a group" the brackets flash, and at "the big groups" the ten gold dots and the long bracket flash. The picture shrinks into the top-left corner, every dot with it, and two empty rooms appear | morph, callback | |
+| 10_2 | Now seat those twelve passengers in a school with two classes, one of two students and one of ten. The school says the average class has six students. | The same twelve dots fly out of the corner one by one and grow into students: the two teal into the small room, the ten gold into the big one, five by two. The rooms flash at "two classes", the small class at "one of two students", the big one at "and one of ten". At "The school says" a teal and a gold bar, each as tall as its class, are born in the rooms and settle side by side in the upper row, captioned "one bar per class" and "classes of 2 and 10". At "six students" the row levels off at 6 and "school's average: (2 + 10) ÷ 2 = 6" appears at the top | dots, bars from facts, equation card | |
+| 10_3 | Ask every student in turn how big their class is. | The students are asked one at a time, the first three slowly. A tiny tag under each says its class's size, 2 or 10, and a bar as tall as the answer is born at the tag and joins the lower row, captioned "one bar per student" and "2 say 2, 10 say 10". The asking runs on past the line, one student at a time, until all twelve have answered | individual answers, visual hold | hold |
+| 10_4 | What's the average answer? ⟨predict⟩ | The question card "students' average?" and the countdown, "pause for more time". The twelve bars are up; their level and their sum are not | think countdown | |
+| 10_5 | Eight and two thirds. It's the bus sum again, with students in place of minutes. Ten of the twelve students sit in the big class. | The lower row levels off at "8⅔" in gold, and "students' average: (2 × 2 + 10 × 10) ÷ 12 = 104 ÷ 12 = 8⅔" appears under the school's sum. At "It's the bus sum again" 06's "passengers' gap: 8 min 40 s" appears under the corner picture while its brackets flash. At "students in place of minutes" the students flash; at "Ten of the twelve students" the ten gold students and their bars flash, and the 8⅔ is ringed | equation card, callback | |
+| 10_6 | Scott Feld and Bernard Grofman wrote about this in nineteen seventy-seven and called it the class size paradox. | Lower-third "source:" with "Feld & Grofman, 'Variation in class size, the class size paradox, and some consequences for students', Research in Higher Education 6 (1977) 215–222", held until it can be read. The students fly back into the corner as passengers while the school fades | lower-third | |
+| 10_7 | For programmers. A sampling profiler checks what your program is doing many times a second, so it measures where the time goes, not how often each function is called. On average, one ten millisecond call shows up as often as five two millisecond calls put together: half the samples, but only a sixth of the calls. | A framed panel slides in from the right under the corner picture, headed "for programmers": three loops of an example program along a line, each one long gold call and five short teal calls, named "10 ms" and "2 ms" under the first loop. At "checks what your program is doing" evenly spaced checks drop onto the calls in their calls' colors. At "one ten millisecond call" the tally "10 ms call: ½ of the time, ⅙ of the calls" in gold while the long calls flash; at "five two millisecond calls" "2 ms calls: ½ of the time, ⅚ of the calls" in teal while the short calls flash; at "half the samples" "checks landing in the 10 ms calls: ½" while the gold checks flash. At "a sixth of the calls" the gold tally is ringed | code panel, tallies from facts | |
+| 10_8 | That needs checks that aren't in step with the program. Checks that tick exactly with its loop land in the same function every time, and the shares come out completely wrong. | At "aren't in step" the even checks and their line go. At "Checks that tick exactly with its loop" a bracket marks one loop, "the loop", and one check a loop drops into the middle of each long call. At "land in the same function every time" the line "checks in step with the loop: all in the 10 ms calls". At "the shares come out completely wrong" the same checks slide a long call later and turn teal, and the line becomes "started later: none in the 10 ms calls". A "precisely" card (K08), held until it can be read | aliasing control, lower-third | |
+| 10_9 | And people noticed it over a century ago. In nineteen twenty-two the physicist Torahiko Terada wrote an essay about waiting for Tokyo's trams, with his own observations of the trams. | The panel goes, and the corner picture grows back to its place, the twelve passengers on their line again. At "In nineteen twenty-two" lower-third "source:" with "Torahiko Terada, physicist, essay of 1922; English translation and discussion: N. Masuda & T. Hiraoka, arXiv:2004.14081 (2020)". It stands until it can be read, then the chapter fades | callback, lower-third | hold |
 
 ### Notes for 10
 
-- Prediction (10_3): the 104 ÷ 12 is not on screen until 10_4.
+- Prediction (10_4): the twelve bars are up before the question, each from
+  its own student's answer; their level, the 8⅔ and the 104 ÷ 12 are not on
+  screen until 10_5. The question is a line of its own because the twelve
+  answers take longer than "Ask every student in turn how big their class
+  is." The asking runs on in silence and the question waits for it.
+- The students are chapter 04's twelve passengers, the same dots. The class
+  sizes are the stretch's gaps' passenger counts (F-W2.people,
+  F-W2.long_people), so each student's answer is the gap its passenger
+  stood in, and the students' average is 06's passengers' gap
+  (F-S1.person_gap, 8:40). The dots go back to the corner as passengers in
+  10_6 and home to their line in 10_9.
 - The tags on the students repeat the tags on the passengers in 06_2:
   same shape, same place below the dot.
 - Feld and Grofman: names, title, journal, volume, pages and year are as
@@ -529,33 +541,57 @@ who plan.
 - Terada: the voice claims only what the translators' abstract says: "a
   Japanese physicist, Torahiko Terada", an essay "in 1922", who "observed
   and analyzed data on Tokyo City trams". The essay itself was not read.
-- Exact version K08 (10_6): "samples taken often and not in step with the
-  program fall in each function in proportion to its running time".
+- The profiler is an example program, not a measurement: a loop of one
+  10 ms call and five 2 ms calls (F-P1), three loops drawn. Each tally keeps
+  the time share and the call share apart, "of the time" beside "of the
+  calls".
+- The even checks, every 3 ms from 1.5 ms, share no factor with the 20 ms
+  loop and run over whole loops, so exactly half land in the long call.
+  Over the first loop alone they do not, and checks at random moments land
+  there half the time only on average; hence "On average" in 10_7. Checks
+  in step with the loop, one a loop, land all in the long call or, started
+  10 ms later, none, and a check every 4 ms gets three fifths or two fifths
+  by where it starts. No sampling clock is promised the time share exactly.
+- Exact version K08 (10_7, 10_8): "on average and over a long run, samples
+  taken often and not in step with the program fall in each function in
+  proportion to its running time".
 - Pronunciation to probe: Feld, Grofman ("GROFF-man"), Torahiko Terada
   ("toh-RAH-hee-koh teh-RAH-dah", to be checked against a Japanese source
   and the owner), "millisecond".
 - Pause intentions:
   - "Eight and two thirds." 0.4 to 0.9 s;
   - "For programmers." 0.35 to 0.7 s, a label, not a sentence the voice
-    leans on.
+    leans on;
+  - the end of 10_3 is a visual hold: the twelve answers run on with no
+    voice until the last bar is in, about three seconds of picture, and
+    the question waits for it. 10_3 and 10_4 are never one sounding unit;
+  - the source card after 10_6, K08 after 10_8 and the Terada card after
+    10_9 are reading holds, picture with no voice until each card can be
+    read.
 
-## 11_back_to_the_stop: Back to the stop (~76 s)
+## 11_back_to_the_stop: Back to the stop (~81 s)
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 11_1 | Back to the stop. Ten buses an hour, all on time, a bus every six minutes on average. All of that is true. | The opening picture rebuilt exactly: the street strip with you at the stop, the sign, the hour line, the blue and red markers, and the sign's two lines | rebuild of 01 | |
-| 11_2 | But you don't sample buses when you turn up. You sample a moment. Five times in six, that moment falls in a long gap, where the waits are long. | The tags appear above and the sixty dots below, as in 02. The long gaps shade gold, and the wait triangles rise above the whole hour, five small teal and five large gold | shapes from the model, callback | |
-| 11_3 | The area of the triangles, divided by the time they cover. Fifty-two over twelve. Four minutes and twenty seconds. | One twelve-minute stretch of the hour brightens (the other four dim, "the same × 5"). On the opening picture, boxed: "average wait = average of gap² ÷ (2 × average gap) = 52 ÷ 12 min = 4 min 20 s" in green | TeX from facts, verdict box | |
-| 11_4 | Now suppose you do check the timetable, and turn up one minute before your bus. How long do you wait? ⟨predict⟩ | You, placed one minute before a red marker; question card "your wait?"; countdown | think countdown | |
-| 11_5 | One minute. You're no longer turning up at random, so the triangles don't describe you. | "wait: 1 min" beside you; the triangles dim | highlight | |
-| 11_6 | And if every passenger planned? Each would aim for the first bus they could make with a minute to spare, and stand at the stop one minute before it. Ten would wait for each blue bus, and two for each red. | The sixty dots, each at the moment its passenger is ready, slide forward along the line, never back, to one minute before the first bus they can make with a minute to spare (`model::planned_arrivals`). The two dots of each twelve minutes that are ready less than a minute before a bus slide on to the next one. Ten stack before each blue marker and two before each red (F-K2.per_blue, F-K2.per_red), every stack in the last minute of its gap (F-K2.stand_at). Each dot keeps the color of the gap it now stands in: the stacks of ten gold, the stacks of two teal | moves, stacking from the model | |
-| 11_7 | Five in six would still stand in a long gap, but every wait would be one minute. Nobody would be spread through a gap any more, so the triangles wouldn't describe them. Change how passengers arrive, and you change the model. | The bar "share of passengers in long gaps" stays at 5/6 (F-K2.in_long_share, equal to F-T1.long_time_share) while beside it "average wait" falls from 4 min 20 s to 1 min (F-K2.mean_wait_clock). Each stack shows "wait 1 min" (F-K2.waits). The gold triangles dim, with the empty part of each long gap left bare. A "precisely" card (K09). It stands, then the chapter fades | bars and labels from facts, lower-third | |
+| 11_1 | Back to the stop. Ten buses an hour, all on time, a bus every six minutes on average. All of that is true. | The opening picture rebuilt as 01_3 left it: chapter 01's street strip with you at the stop, halved at the top; the hour line with the blue and red markers; the sign. At "Ten buses an hour" the markers pulse in time order; at "all on time" a green dot over every marker and "every bus on time"; at "a bus every six minutes on average" the sign's last line is ringed; at "All of that is true" the sign's two lines with numbers are ringed in green | rebuild of 01 | |
+| 11_2 | But you don't sample buses when you turn up. You sample a moment. Five times in six, that moment falls in a long gap, where the waits are long. | "every bus on time" and its dots go. At "you don't sample buses" the tags appear over the markers, as in 02; at "You sample a moment" the sixty dots appear under the line, one in the middle of every minute. At "Five times in six" the long gaps shade gold and every dot takes its gap's color. At "where the waits are long" the tags and the shading go, and the wait triangles rise over the whole hour, a minute up as long as a minute across, five small teal and five large gold, see-through, behind the markers | shapes from the model, callback | |
+| 11_3 | The area of the triangles, divided by the time they cover. Fifty-two square minutes over twelve minutes. Four minutes and twenty seconds. | At "The area of the triangles" the first twelve minutes brighten and the other four stretches dim, labelled "the same × 5"; the short triangle is labelled "2 min²" left of the line and the long one "50 min²" inside it. At "divided by the time they cover" a bracket under the stretch, "12 min". At "Fifty-two square minutes" the green box "average wait: 52 min² ÷ 12 min = 4 min 20 s"; at "Four minutes and twenty seconds" it is ringed, and it stands until it can be read | areas from facts, verdict box | |
+| 11_4 | Now suppose you do check the timetable, and turn up one minute before your bus. How long do you wait? ⟨predict⟩ | The labels go and the triangles return to one brightness. At "turn up one minute before your bus" you leave the strip and grow back to full size under the passengers' row at :25, one minute before the red marker at :26, with a thin line up to your moment on the hour line, and that marker flashes. At "How long do you wait" the question "your wait?" beside you; countdown, "pause for more time" | think countdown | |
+| 11_5 | One minute. You're no longer turning up at random, so the triangles' average doesn't describe you. | A green segment grows up from your moment to the triangle's slant, the triangle's own height there, and "wait: 1 min" stands beside you. At "the triangles' average doesn't describe you" the triangles and the box dim | highlight | |
+| 11_6 | And if every passenger planned? Each would aim for the first bus they could make with a minute to spare, and stand at the stop one minute before it. Ten would wait for each blue bus, and two for each red. | You, your line, the box, the strip and the sign go. At the top, a panel: "share of passengers in long gaps", a bar filled five sixths in gold, "⅚"; "average wait", a bar that grows to 4 min 20 s, "4 min 20 s". At "with a minute to spare" the dots that are ready less than a minute before a bus flash, two in every twelve minutes. At "and stand at the stop" the sixty dots slide forward along the line, never back, each to one minute before the first bus it can make with a minute to spare (`model::planned_arrivals`); the dot at :59½ runs off the end at :60 and comes in again at :00, to stand at :01. Each dot takes the color of the gap it moves through and stacks down from the passengers' row, the dot with least far to slide on top: ten before each blue marker, gold, and two before each red, teal (F-K2.per_blue, F-K2.per_red), every stack in the last minute of its gap (F-K2.stand_at). At "Ten would wait for each blue bus" the blue stacks and markers flash, at "two for each red" the red ones | moves, stacking from the model, bars from facts | |
+| 11_7 | Five in six would still stand in a long gap, but every wait would be one minute. They'd all stand in the last minute of a gap, so the triangles' average wouldn't describe them. Change how passengers arrive, and you change the model. | The "⅚" is ringed: the share stays (F-K2.in_long_share, equal to F-T1.long_time_share). At "every wait would be one minute" "4 min 20 s" goes, and while the wait bar falls steadily to a minute, "1 min" comes in beside it (F-K2.mean_wait_clock) and "wait 1 min" under every stack (F-K2.waits). At "They'd all stand in the last minute of a gap" the last minute of every gap is drawn over the line in its gap's color; at "the triangles' average wouldn't describe them" the gold triangles all but vanish. At "Change how passengers arrive" a "precisely" card (K09). It stands until it can be read, then the chapter fades | bars and labels from facts, lower-third | hold |
 
 ### Notes for 11
 
+- 11_1 rebuilds chapter 01's picture from chapter 01's own street (the
+  scene's copy is checked against `the_stop.rs`), with the hour, the
+  markers and the sign as 01_3 left them.
+- 11_3's box divides the area by the time in their units, "52 min² ÷ 12
+  min". The letter formula is 08's and is not repeated here.
 - Prediction (11_4): a negative control for the assumption. Arriving by the
   timetable is not arriving at random (oracle "timetable reader",
-  F-K1.wait).
+  F-K1.wait). Your wait is the triangle's own height at your moment, and
+  "wait: 1 min" is not on screen until 11_5.
 - 11_6 and 11_7 test the model's limit with the same dots. Each dot is
   the moment its passenger is ready. A planner stands at the stop one
   minute before the first bus at or after that moment plus one minute, so
@@ -574,7 +610,8 @@ who plan.
   wait. Oracle negative control "planners".
 - Not used: spreading the planners evenly, six to a bus. That would put
   half of them in a short gap only by making some pass up a bus they could
-  make, which the film's planners don't do.
+  make, which the film's planners don't do. The scene's tests keep it as a
+  control: it also waits a minute, with only half in long gaps.
 - Exact version K09 (11_7): "the average wait E[G²]/(2E[G]) is for arrival
   moments independent of the timetable and spread evenly in time;
   passengers who plan, or who arrive with other passengers' timing, need a
@@ -584,7 +621,10 @@ who plan.
   - "One minute." 0.4 to 0.8 s;
   - after "And if every passenger planned?" 0.35 to 0.7 s before the dots
     move;
-  - after "every wait would be one minute." a thought pause, 0.5 to 1.0 s.
+  - after "every wait would be one minute." a thought pause, 0.5 to 1.0 s,
+    while the bar falls;
+  - the verdict after 11_3 and K09 after 11_7 are reading holds, picture
+    with no voice until each can be read.
 
 ## 12_one_more_timetable: One more timetable (~64 s)
 
diff --git a/explainer_corpus/inspection_paradox/docs/storyboard.md b/explainer_corpus/inspection_paradox/docs/storyboard.md
index c8e8448..367afaf 100644
--- a/explainer_corpus/inspection_paradox/docs/storyboard.md
+++ b/explainer_corpus/inspection_paradox/docs/storyboard.md
@@ -412,3 +412,125 @@ who plan.
 - Pause intentions:
   - "Six minutes." 0.6 to 1.2 s, the chapter's surprise;
   - after "The buses have no memory." 0.45 to 0.9 s.
+
+## 10_same_sum: The same sum elsewhere (~98 s)
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 10_1 | The same sum turns up far from bus stops. Whenever you pick a person or a moment instead of a group, the big groups get picked more often. | Chapter 04's stretch returns as it was: its twelve passengers on the line, two teal in the short gap and ten gold in the long one, under the two gaps' brackets. At "a person or a moment" the dots flash one by one, at "instead of a group" the brackets flash, and at "the big groups" the ten gold dots and the long bracket flash. The picture shrinks into the top-left corner, every dot with it, and two empty rooms appear | morph, callback | |
+| 10_2 | Now seat those twelve passengers in a school with two classes, one of two students and one of ten. The school says the average class has six students. | The same twelve dots fly out of the corner one by one and grow into students: the two teal into the small room, the ten gold into the big one, five by two. The rooms flash at "two classes", the small class at "one of two students", the big one at "and one of ten". At "The school says" a teal and a gold bar, each as tall as its class, are born in the rooms and settle side by side in the upper row, captioned "one bar per class" and "classes of 2 and 10". At "six students" the row levels off at 6 and "school's average: (2 + 10) ÷ 2 = 6" appears at the top | dots, bars from facts, equation card | |
+| 10_3 | Ask every student in turn how big their class is. | The students are asked one at a time, the first three slowly. A tiny tag under each says its class's size, 2 or 10, and a bar as tall as the answer is born at the tag and joins the lower row, captioned "one bar per student" and "2 say 2, 10 say 10". The asking runs on past the line, one student at a time, until all twelve have answered | individual answers, visual hold | hold |
+| 10_4 | What's the average answer? ⟨predict⟩ | The question card "students' average?" and the countdown, "pause for more time". The twelve bars are up; their level and their sum are not | think countdown | |
+| 10_5 | Eight and two thirds. It's the bus sum again, with students in place of minutes. Ten of the twelve students sit in the big class. | The lower row levels off at "8⅔" in gold, and "students' average: (2 × 2 + 10 × 10) ÷ 12 = 104 ÷ 12 = 8⅔" appears under the school's sum. At "It's the bus sum again" 06's "passengers' gap: 8 min 40 s" appears under the corner picture while its brackets flash. At "students in place of minutes" the students flash; at "Ten of the twelve students" the ten gold students and their bars flash, and the 8⅔ is ringed | equation card, callback | |
+| 10_6 | Scott Feld and Bernard Grofman wrote about this in nineteen seventy-seven and called it the class size paradox. | Lower-third "source:" with "Feld & Grofman, 'Variation in class size, the class size paradox, and some consequences for students', Research in Higher Education 6 (1977) 215–222", held until it can be read. The students fly back into the corner as passengers while the school fades | lower-third | |
+| 10_7 | For programmers. A sampling profiler checks what your program is doing many times a second, so it measures where the time goes, not how often each function is called. On average, one ten millisecond call shows up as often as five two millisecond calls put together: half the samples, but only a sixth of the calls. | A framed panel slides in from the right under the corner picture, headed "for programmers": three loops of an example program along a line, each one long gold call and five short teal calls, named "10 ms" and "2 ms" under the first loop. At "checks what your program is doing" evenly spaced checks drop onto the calls in their calls' colors. At "one ten millisecond call" the tally "10 ms call: ½ of the time, ⅙ of the calls" in gold while the long calls flash; at "five two millisecond calls" "2 ms calls: ½ of the time, ⅚ of the calls" in teal while the short calls flash; at "half the samples" "checks landing in the 10 ms calls: ½" while the gold checks flash. At "a sixth of the calls" the gold tally is ringed | code panel, tallies from facts | |
+| 10_8 | That needs checks that aren't in step with the program. Checks that tick exactly with its loop land in the same function every time, and the shares come out completely wrong. | At "aren't in step" the even checks and their line go. At "Checks that tick exactly with its loop" a bracket marks one loop, "the loop", and one check a loop drops into the middle of each long call. At "land in the same function every time" the line "checks in step with the loop: all in the 10 ms calls". At "the shares come out completely wrong" the same checks slide a long call later and turn teal, and the line becomes "started later: none in the 10 ms calls". A "precisely" card (K08), held until it can be read | aliasing control, lower-third | |
+| 10_9 | And people noticed it over a century ago. In nineteen twenty-two the physicist Torahiko Terada wrote an essay about waiting for Tokyo's trams, with his own observations of the trams. | The panel goes, and the corner picture grows back to its place, the twelve passengers on their line again. At "In nineteen twenty-two" lower-third "source:" with "Torahiko Terada, physicist, essay of 1922; English translation and discussion: N. Masuda & T. Hiraoka, arXiv:2004.14081 (2020)". It stands until it can be read, then the chapter fades | callback, lower-third | hold |
+
+### Notes for 10
+
+- Prediction (10_4): the twelve bars are up before the question, each from
+  its own student's answer; their level, the 8⅔ and the 104 ÷ 12 are not on
+  screen until 10_5. The question is a line of its own because the twelve
+  answers take longer than "Ask every student in turn how big their class
+  is." The asking runs on in silence and the question waits for it.
+- The students are chapter 04's twelve passengers, the same dots. The class
+  sizes are the stretch's gaps' passenger counts (F-W2.people,
+  F-W2.long_people), so each student's answer is the gap its passenger
+  stood in, and the students' average is 06's passengers' gap
+  (F-S1.person_gap, 8:40). The dots go back to the corner as passengers in
+  10_6 and home to their line in 10_9.
+- The tags on the students repeat the tags on the passengers in 06_2:
+  same shape, same place below the dot.
+- Feld and Grofman: names, title, journal, volume, pages and year are as
+  printed on the article's first page (scan hosted by UC Irvine, pages 215
+  and 216 read 2026-10-24; brief.md). Page 216: "We call this the class
+  size paradox." The voice claims only that they wrote about it in 1977 and
+  named it.
+- Terada: the voice claims only what the translators' abstract says: "a
+  Japanese physicist, Torahiko Terada", an essay "in 1922", who "observed
+  and analyzed data on Tokyo City trams". The essay itself was not read.
+- The profiler is an example program, not a measurement: a loop of one
+  10 ms call and five 2 ms calls (F-P1), three loops drawn. Each tally keeps
+  the time share and the call share apart, "of the time" beside "of the
+  calls".
+- The even checks, every 3 ms from 1.5 ms, share no factor with the 20 ms
+  loop and run over whole loops, so exactly half land in the long call.
+  Over the first loop alone they do not, and checks at random moments land
+  there half the time only on average; hence "On average" in 10_7. Checks
+  in step with the loop, one a loop, land all in the long call or, started
+  10 ms later, none, and a check every 4 ms gets three fifths or two fifths
+  by where it starts. No sampling clock is promised the time share exactly.
+- Exact version K08 (10_7, 10_8): "on average and over a long run, samples
+  taken often and not in step with the program fall in each function in
+  proportion to its running time".
+- Pronunciation to probe: Feld, Grofman ("GROFF-man"), Torahiko Terada
+  ("toh-RAH-hee-koh teh-RAH-dah", to be checked against a Japanese source
+  and the owner), "millisecond".
+- Pause intentions:
+  - "Eight and two thirds." 0.4 to 0.9 s;
+  - "For programmers." 0.35 to 0.7 s, a label, not a sentence the voice
+    leans on;
+  - the end of 10_3 is a visual hold: the twelve answers run on with no
+    voice until the last bar is in, about three seconds of picture, and
+    the question waits for it. 10_3 and 10_4 are never one sounding unit;
+  - the source card after 10_6, K08 after 10_8 and the Terada card after
+    10_9 are reading holds, picture with no voice until each card can be
+    read.
+
+## 11_back_to_the_stop: Back to the stop (~81 s)
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 11_1 | Back to the stop. Ten buses an hour, all on time, a bus every six minutes on average. All of that is true. | The opening picture rebuilt as 01_3 left it: chapter 01's street strip with you at the stop, halved at the top; the hour line with the blue and red markers; the sign. At "Ten buses an hour" the markers pulse in time order; at "all on time" a green dot over every marker and "every bus on time"; at "a bus every six minutes on average" the sign's last line is ringed; at "All of that is true" the sign's two lines with numbers are ringed in green | rebuild of 01 | |
+| 11_2 | But you don't sample buses when you turn up. You sample a moment. Five times in six, that moment falls in a long gap, where the waits are long. | "every bus on time" and its dots go. At "you don't sample buses" the tags appear over the markers, as in 02; at "You sample a moment" the sixty dots appear under the line, one in the middle of every minute. At "Five times in six" the long gaps shade gold and every dot takes its gap's color. At "where the waits are long" the tags and the shading go, and the wait triangles rise over the whole hour, a minute up as long as a minute across, five small teal and five large gold, see-through, behind the markers | shapes from the model, callback | |
+| 11_3 | The area of the triangles, divided by the time they cover. Fifty-two square minutes over twelve minutes. Four minutes and twenty seconds. | At "The area of the triangles" the first twelve minutes brighten and the other four stretches dim, labelled "the same × 5"; the short triangle is labelled "2 min²" left of the line and the long one "50 min²" inside it. At "divided by the time they cover" a bracket under the stretch, "12 min". At "Fifty-two square minutes" the green box "average wait: 52 min² ÷ 12 min = 4 min 20 s"; at "Four minutes and twenty seconds" it is ringed, and it stands until it can be read | areas from facts, verdict box | |
+| 11_4 | Now suppose you do check the timetable, and turn up one minute before your bus. How long do you wait? ⟨predict⟩ | The labels go and the triangles return to one brightness. At "turn up one minute before your bus" you leave the strip and grow back to full size under the passengers' row at :25, one minute before the red marker at :26, with a thin line up to your moment on the hour line, and that marker flashes. At "How long do you wait" the question "your wait?" beside you; countdown, "pause for more time" | think countdown | |
+| 11_5 | One minute. You're no longer turning up at random, so the triangles' average doesn't describe you. | A green segment grows up from your moment to the triangle's slant, the triangle's own height there, and "wait: 1 min" stands beside you. At "the triangles' average doesn't describe you" the triangles and the box dim | highlight | |
+| 11_6 | And if every passenger planned? Each would aim for the first bus they could make with a minute to spare, and stand at the stop one minute before it. Ten would wait for each blue bus, and two for each red. | You, your line, the box, the strip and the sign go. At the top, a panel: "share of passengers in long gaps", a bar filled five sixths in gold, "⅚"; "average wait", a bar that grows to 4 min 20 s, "4 min 20 s". At "with a minute to spare" the dots that are ready less than a minute before a bus flash, two in every twelve minutes. At "and stand at the stop" the sixty dots slide forward along the line, never back, each to one minute before the first bus it can make with a minute to spare (`model::planned_arrivals`); the dot at :59½ runs off the end at :60 and comes in again at :00, to stand at :01. Each dot takes the color of the gap it moves through and stacks down from the passengers' row, the dot with least far to slide on top: ten before each blue marker, gold, and two before each red, teal (F-K2.per_blue, F-K2.per_red), every stack in the last minute of its gap (F-K2.stand_at). At "Ten would wait for each blue bus" the blue stacks and markers flash, at "two for each red" the red ones | moves, stacking from the model, bars from facts | |
+| 11_7 | Five in six would still stand in a long gap, but every wait would be one minute. They'd all stand in the last minute of a gap, so the triangles' average wouldn't describe them. Change how passengers arrive, and you change the model. | The "⅚" is ringed: the share stays (F-K2.in_long_share, equal to F-T1.long_time_share). At "every wait would be one minute" "4 min 20 s" goes, and while the wait bar falls steadily to a minute, "1 min" comes in beside it (F-K2.mean_wait_clock) and "wait 1 min" under every stack (F-K2.waits). At "They'd all stand in the last minute of a gap" the last minute of every gap is drawn over the line in its gap's color; at "the triangles' average wouldn't describe them" the gold triangles all but vanish. At "Change how passengers arrive" a "precisely" card (K09). It stands until it can be read, then the chapter fades | bars and labels from facts, lower-third | hold |
+
+### Notes for 11
+
+- 11_1 rebuilds chapter 01's picture from chapter 01's own street (the
+  scene's copy is checked against `the_stop.rs`), with the hour, the
+  markers and the sign as 01_3 left them.
+- 11_3's box divides the area by the time in their units, "52 min² ÷ 12
+  min". The letter formula is 08's and is not repeated here.
+- Prediction (11_4): a negative control for the assumption. Arriving by the
+  timetable is not arriving at random (oracle "timetable reader",
+  F-K1.wait). Your wait is the triangle's own height at your moment, and
+  "wait: 1 min" is not on screen until 11_5.
+- 11_6 and 11_7 test the model's limit with the same dots. Each dot is
+  the moment its passenger is ready. A planner stands at the stop one
+  minute before the first bus at or after that moment plus one minute, so
+  every dot moves only forward and nobody turns up before they're ready
+  (F-K2.never_early). Nobody is shared out over the buses: the stacks of
+  ten and two come from the dots and the timetable alone.
+- Planning doesn't move passengers out of long gaps. Five in six still
+  stand in one, exactly the time share (F-K2.in_long_share against
+  F-T1.long_time_share). What changes is where in the gap they stand.
+  Random arrivals are spread evenly through their gap, so on average half
+  of it is still ahead of them (F-S1.wait_share_of_gap, 1/2). Every planner
+  stands in the last minute of a gap, so on average only a sixth of it is
+  ahead (F-K2.wait_share_of_gap, 1/6). The triangles assume the even
+  spread, so they give 4:20 where the planners wait 1:00
+  (F-K2.mean_wait_clock). The share of long gaps alone never gave the
+  wait. Oracle negative control "planners".
+- Not used: spreading the planners evenly, six to a bus. That would put
+  half of them in a short gap only by making some pass up a bus they could
+  make, which the film's planners don't do. The scene's tests keep it as a
+  control: it also waits a minute, with only half in long gaps.
+- Exact version K09 (11_7): "the average wait E[G²]/(2E[G]) is for arrival
+  moments independent of the timetable and spread evenly in time;
+  passengers who plan, or who arrive with other passengers' timing, need a
+  model of how they arrive". The video makes no claim about how many real
+  passengers plan.
+- Pause intentions:
+  - "One minute." 0.4 to 0.8 s;
+  - after "And if every passenger planned?" 0.35 to 0.7 s before the dots
+    move;
+  - after "every wait would be one minute." a thought pause, 0.5 to 1.0 s,
+    while the bar falls;
+  - the verdict after 11_3 and K09 after 11_7 are reading holds, picture
+    with no voice until each can be read.
diff --git a/explainer_corpus/inspection_paradox/src/chapters/mod.rs b/explainer_corpus/inspection_paradox/src/chapters/mod.rs
index 7f505d8..eb3f0aa 100644
--- a/explainer_corpus/inspection_paradox/src/chapters/mod.rs
+++ b/explainer_corpus/inspection_paradox/src/chapters/mod.rs
@@ -4,16 +4,18 @@
 //!
 //! Only chapters with native scene source are registered, and they are
 //! exactly the chapters of `docs/storyboard.md` and `script.tsv`: chapters
-//! 01 through 09. Chapters 10 to
-//! 12 are written as words and pictures in `docs/storyboard-draft.md`, the
-//! full draft. Each moves into the storyboard and this registry when root
-//! assigns it and its scene source exists, never as a stand-in.
+//! 01 through 11. Chapter 12 is written as words and pictures in
+//! `docs/storyboard-draft.md`, the full draft. It moves into the storyboard
+//! and this registry when root assigns it and its scene source exists, never
+//! as a stand-in.
 
 pub mod at_random;
+pub mod back_to_the_stop;
 pub mod draw;
 pub mod glyphs;
 pub mod no_bus_late;
 pub mod one_a_minute;
+pub mod same_sum;
 pub mod stretch_one_gap;
 pub mod the_formula;
 pub mod the_stop;
@@ -72,6 +74,16 @@ pub fn registry() -> Vec<Chapter> {
             make: |kit| Box::new(at_random::AtRandom { kit }),
             intensity: 0.5,
         },
+        Chapter {
+            key: "10_same_sum",
+            make: |kit| Box::new(same_sum::SameSum { kit }),
+            intensity: 0.45,
+        },
+        Chapter {
+            key: "11_back_to_the_stop",
+            make: |kit| Box::new(back_to_the_stop::BackToTheStop { kit }),
+            intensity: 0.5,
+        },
     ]
 }
 
diff --git a/explainer_corpus/inspection_paradox/src/main.rs b/explainer_corpus/inspection_paradox/src/main.rs
index 02c1724..1337b7f 100644
--- a/explainer_corpus/inspection_paradox/src/main.rs
+++ b/explainer_corpus/inspection_paradox/src/main.rs
@@ -11,7 +11,7 @@
 //! inspection_paradox help             # the full option list
 //! ```
 //!
-//! Chapters 01 through 05 have scene source; the whole film is
+//! Chapters 01 through 11 have scene source; the whole film is
 //! `docs/storyboard-draft.md` (see `chapters/mod.rs`).
 
 mod chapters;
@@ -291,6 +291,43 @@ mod tests {
         ("09_8", "random buses wait six", "F-R1.mean_wait", "6"),
         ("09_8", "sits at four twenty", "F-W1.mean_wait_clock", "4:20"),
         ("09_8", "even past six", "F-R1.mean_wait", "6"),
+        // 10: the same sum elsewhere; the class size paradox; the profiler;
+        // Terada. The "one" of "one ten millisecond call" is the loop's single
+        // long call, the 1 in F-P1.long_call_share = 1/(SHORT_CALLS + 1)
+        ("10_2", "those twelve passengers", "F-W2.people", "12"),
+        ("10_2", "two classes", "F-C1.classes", "2"),
+        ("10_2", "one of two students and one of ten", "F-C1.sizes", "2,10"),
+        ("10_2", "six students", "F-C1.school_mean", "6"),
+        ("10_5", "Eight and two thirds", "F-C1.student_mean", "26/3"),
+        ("10_5", "Ten of the twelve students", "F-C1.big_class_students", "10"),
+        ("10_5", "of the twelve students", "F-C1.students", "12"),
+        ("10_6", "in nineteen seventy-seven", "F-C1.year", "1977"),
+        ("10_7", "one ten millisecond call", "F-P1.long_ms", "10"),
+        ("10_7", "five two millisecond calls", "F-P1.short_calls", "5"),
+        ("10_7", "two millisecond calls", "F-P1.short_ms", "2"),
+        ("10_7", "half the samples", "F-P1.long_sample_share", "1/2"),
+        ("10_7", "a sixth of the calls", "F-P1.long_call_share", "1/6"),
+        ("10_9", "over a century ago", "F-H1.over_a_century", "true"),
+        ("10_9", "In nineteen twenty-two", "F-H1.year", "1922"),
+        // 11: back to the stop; the triangles' average; the planners
+        ("11_1", "Ten buses an hour", "F-T1.buses_per_hour", "10"),
+        ("11_1", "every six minutes on average", "F-T1.mean_gap", "6"),
+        ("11_2", "Five times in six", "F-T1.long_time_share", "5/6"),
+        ("11_3", "Fifty-two square minutes", "F-A1.total_area", "52"),
+        ("11_3", "over twelve minutes", "F-T1.period", "12"),
+        (
+            "11_3",
+            "Four minutes and twenty seconds",
+            "F-W1.mean_wait_clock",
+            "4:20",
+        ),
+        ("11_4", "one minute before your bus", "F-K1.early", "1"),
+        ("11_5", "One minute", "F-K1.wait", "1"),
+        ("11_6", "one minute before it", "F-K1.early", "1"),
+        ("11_6", "Ten would wait for each blue bus", "F-K2.per_blue", "10"),
+        ("11_6", "two for each red", "F-K2.per_red", "2"),
+        ("11_7", "Five in six would still stand", "F-K2.in_long_share", "5/6"),
+        ("11_7", "every wait would be one minute", "F-K2.waits", "1"),
     ];
 
     /// The words the guard below listens for: every number word the full
````
