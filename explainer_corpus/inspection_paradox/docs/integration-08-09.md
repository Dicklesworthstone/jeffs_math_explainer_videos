# Chapters 08 and 09: native source and integration

2026-10-24, GoldGate, for root. This note covers the native scene source for
`08_the_formula` and `09_at_random`, which sits in two new files. Nothing
registers them yet, so the build does not compile them. None of it has been
compiled, rendered, narrated or listened to. Every check below is a Python
model, a rustfmt parse or a source reading, not native execution.

## Files

| file | sha256 (first 16) | bytes | what it is |
|---|---|---|---|
| `src/chapters/the_formula.rs` | `658ecf773468bdf2` | 85788 | chapter 08, `TheFormula` |
| `src/chapters/at_random.rs` | `00d407d0074f4212` | 58941 | chapter 09, `AtRandom` |

No shared helper was added or edited.

- 08 uses chapter 05's drawing from `twelve_minutes.rs`: `stretch`, `gaps`,
  `numbers`, `passengers`, `at`, `frame`, `mixed`, `MIN`, `AXIS_Y` and
  `ORIGIN_X`. Its test `the_squares_tile_the_long_square` compares its
  triangle with `triangles::triangle`.
- 09 uses 08's public helpers: `Row`, `Role`, `show_row`, `halo`, `pulse`,
  `frame_around`, `text_box`, `play_all`, `spread_color` and `formula`.
  So 09 needs 08, and both need 04/05 as registered now.

Both files are built on the production state after root's 06/07
integration. I did not edit any of these files:

| file | sha256 (first 8) |
|---|---|
| `main.rs` | d739241b |
| `chapters/mod.rs` | fd975fbd |
| `script.tsv` | 14548118 |
| `docs/storyboard.md` | 989f6417 |
| `docs/storyboard-draft.md` | 94375fd0 |
| `docs/pause-overrides.json` | d9e0faea |
| `draw.rs` | e51741b3 |
| `timetable.rs` | 66c43d84 |
| `glyphs.rs` | 7aacf404 |
| `model.rs` | bc3be4ad |
| `facts.rs` | a3cd5b3e |
| `palette.rs` | aeb18f55 |
| `oracle.py` | 47a2c058 |
| `the_stop.rs` | 1432b763 |
| `two_ways_to_count.rs` | 8e5d0386 |
| `stretch_one_gap.rs` | 9087c041 |
| `twelve_minutes.rs` | 61206a6f |
| `one_a_minute.rs` | 747ac26e |
| `triangles.rs` | dc74a296 |
| `two_averages.rs` | 78325de4 |
| `no_bus_late.rs` | 5ebf1bb6 |

The draft's 08 and 09 sections did not change in root's 06/07 integration
(only 06_7's picture text did). 08's wait-axis name follows root's repaired
05 label: the axis runs 0.05 past the frame, and the name sits 0.2 above it.

## Integration, in order (root's)

1. **Apply the patch** at the end of this note, from the repo root:
   `git apply --check`, then `git apply`. It makes four edits, all to
   files I left untouched:
   - **`src/chapters/mod.rs`:**
     - adds `pub mod at_random;` and `pub mod the_formula;` in alphabetical
       order;
     - registers `08_the_formula` (`the_formula::TheFormula`, intensity 0.4)
       and `09_at_random` (`at_random::AtRandom`, intensity 0.5) after 07;
     - changes the module comment to "01 through 09. Chapters 10 to 12".
     - The intensities are proposals. 08 is a long derivation with silent
       reading holds, so its score sits lower. 09 holds the surprise.
   - **`docs/storyboard.md`:** appends the two draft sections byte for
     byte, notes included:

     | section | sha256 (first 16) | bytes |
     |---|---|---|
     | `## 08_the_formula: The formula, from the triangles (~85 s)` | `8779839a3095f473` | 3951 |
     | `## 09_at_random: No timetable at all (~77 s)` | `dafdbc406c9ee2c6` | 4030 |

     Each hash covers `"\n## "` plus the section, with trailing whitespace
     stripped (the unit `the_full_draft_holds_the_storyboard_verbatim`
     compares). The `~85 s` and `~77 s` are the draft's estimates, not
     limits (see Timing).
   - **`src/main.rs`, `SPOKEN`:** appends the 38 rows of step 3.
   - **`docs/pause-overrides.json`:** appends the three entries of step 4,
     and changes `_comment` to say chapters 01 to 09 (10 to 12 still in the
     draft).

   Patched files, if applied to the hashes above:

   | file | before | after |
   |---|---|---|
   | `src/chapters/mod.rs` | fd975fbd | 5db91597 |
   | `src/main.rs` | d739241b | 4fd8ad3f |
   | `docs/storyboard.md` | 989f6417 | 265f0926 |
   | `docs/pause-overrides.json` | d9e0faea | 3069d9ad |

2. **`script.tsv`:** run `python3 -I tools/storyboard.py inspection_paradox`
   (this slug only), then
   `python3 -I tools/storyboard.py --check inspection_paradox`. This adds
   the sixteen lines 08_1 to 09_8.
   - I did not run storyboard.py. I emulated its spoken text, and the
     emulation reproduces all 51 current lines exactly.
   - On that emulation the sixteen appended lines hash to `2b98a51028c7ac65`,
     and the whole file to `799fd063ce7e9fae` (sha256, first 16).
   - Treat these as expectations, not receipts.

3. **Numeric bindings (`SPOKEN`, in the patch).** The rows are rustfmt's
   layout for this repo (max_width 120). I ran them through rustfmt in a
   copy of main.rs's context. Checked as main.rs's
   `every_spoken_number_is_the_fact_it_speaks_for` checks them:
   - Each phrase is in its line, and each value is the oracle's value for
     its key.
   - The guard passes for all 16 lines. 08_2, 09_1 and 09_3 say no number
     word, so they have no row.
   - A stricter check finds every number word inside some row's phrase.
   - All keys are added literally in `facts.rs`.
   - No line uses a pronoun "one", so `PRONOUN_ONES` needs no change.
   - Four negative controls are caught: a wrong value, a missing phrase, a
     dropped line and an invented key.

   Bindings worth a second look:
   - "twice" in 08_5's "over twice six" sits inside the row bound to the
     six (F-T1.mean_gap). The 2 is the reciprocal of F-F1.area_factor, which
     `formula` derives and checks. No separate fact names it.
   - "not half of it" (09_4) is bound to F-X1.bound_factor, the ½ of 08's
     bound that random buses exceed.
   - 08_6's minute and twenty seconds is F-W1.extra_clock, the value that
     flies to you. 08_5's is F-F1.penalty_clock. `formula` checks that they
     are equal.

4. **Pause overrides (in the patch).** The ranges come from the draft's
   pause notes. They are editorial hypotheses, not measurements.

   | cue | after | default | authored |
   |---|---|---|---|
   | 08_8 | "No" | thought 0.35–1.1 | thought 0.4–0.8 |
   | 09_4 | "Six minutes" | thought 0.35–1.1 | thought 0.6–1.2 |
   | 09_5 | "The buses have no memory" | thought 0.35–1.1 | thought 0.45–0.9 |

   - I ran each entry through `tools/narration_director.py`'s own
     `directions` on the draft's 87 lines. Each names exactly one boundary,
     not a cue end, and replaces the punctuation default there.
   - Two negative controls were refused: "gap", which occurs twice in
     09_5, and a phrase not in its cue.
   - The draft's other two intentions sit at cue ends, so no entry was
     added for them:
     - After "There's the extra time." (08_5) the draft asks for 0.6–1.1 s.
       The default `cue_transition` (0.45–1.4 s) contains it.
     - After "not mornings." (08_6) the draft asks for the cue transition
       itself.

5. **Glyphs.** 08/09 print two characters that no registered chapter
   prints yet. The probe collects whatever registered scenes typeset, so
   no `glyphs.rs` edit is needed for them to be probed.
   - § (U+00A7, in K05) is in the bundled Computer Modern (`cmunrm.ttf`).
   - ≥ (U+2265, "average wait ≥ ½ × average gap") is not in Computer
     Modern. At pin `a399d626` the fallback chain is Computer Modern → Noto
     Sans Math → IBM Plex Sans. Noto Sans Math Symbols maps it, and so does
     IBM Plex Sans Regular.
   - Everything else non-ASCII is already printed by 01–07: ² → × ÷ − ½ ⅓
     · and the min² unit.
   - I read the cmaps from the fmd-font copies the lockfile pins. That is
     a cmap reading, not a probe run.

Then root's usual order: `cargo fmt --check`, Clippy with `-D warnings`,
the crate's tests, `inspection_paradox glyphs`, and a preview.

### Cue IDs and what each waits for

Every phrase occurs once in its line, after the previous one. The 28 + 27
phrases are parsed from the construct bodies, loop arrays included.

- **08_1:** "The gap of two"; "the gap of ten holds fifty"; "Any gap holds
  half its length squared".
- **08_2:** "Add up the triangles"; "divide by the time they cover";
  "That's the average wait".
- **08_3:** "Divide the top and the bottom"; "The top becomes"; "The bottom
  becomes".
- **08_4:** "two squared is four"; "ten squared is a hundred"; "the average
  square is fifty-two"; "Six squared is only thirty-six"; "The extra
  sixteen"; "how uneven the gaps are".
- **08_5:** "three minutes"; "plus sixteen over twice six"; "a minute and
  twenty seconds"; "There's the extra time".
- **08_6:** "that minute and twenty seconds"; "The sign can't show it"; "it
  counts buses"; "not mornings".
- **08_7:** "Can the average wait ever be less".
- **08_8:** "The extra piece is never negative"; "who turn up at random";
  "at least half the average gap"; "exactly half only when every gap is the
  same".
- **09_1:** "Suppose buses come completely at random"; "At every moment";
  "just as likely to come"; "however long it's been".
- **09_2:** "a bus every six minutes"; "some gaps are tiny"; "the long ones
  hold huge triangles".
- **09_3:** "How long is the average wait now".
- **09_4:** "The whole average gap"; "not half of it".
- **09_5:** "So from the moment you turn up"; "the wait ahead of you";
  "just like a brand new gap"; "a gap averages six minutes".
- **09_6:** "The formula agrees"; "the spread is the average gap squared";
  "Three minutes, plus thirty-six over twelve"; "Six".
- **09_7:** "Look back too"; "So the gap you land in"; "averages twelve
  minutes".
- **09_8:** "Even gaps wait three minutes"; "random buses wait six"; "our
  timetable sits at four twenty"; "More spread means a longer wait"; "even
  past six"; "The formula needs only".

### Authored semantic holds

These are silent windows the scenes author. The durations come from the
code, and the reading times are `read_s` (characters ÷ 15).
`narration_director`'s `visual_holds` needs an evidence file and its hash,
so none is proposed as JSON until a preview exists. These are the
candidates, by adjacent cues:

| between | hold | why |
|---|---|---|
| 08_2 → 08_3 | `music_breath` 2.0 s, then 1.2 s of cleanup | the sum in letters is read before it is divided |
| 08_3 → 08_4 | `music_breath` 2.0 s, then 1.2 s of cleanup | the formula in letters, our numbers under it |
| 08_4 → 08_5 | about 18.4 s (strips 0.6 s; rows read 3.9 + 2.7 + 4.5 s, each after a 0.5 s fade; then max(3.3, 4.0) s) | the square argument, one step at a time, each held long enough to read |
| 08_5 → 08_6 | `music_breath` 2.0 s | the formula in letters, after our numbers |
| 08_7 → 08_8 | `think`, 4 s ring | prediction |
| 08_8 → end | until card + K05 are read (10.1 + 16.3 s from the card), then `stand` | the finite check and K05 |
| 09_1 → 09_2 | the 5 s sweep, then `hold` | the since-last clock restarts at each bus while the chance stays level |
| 09_3 → 09_4 | `think`, 4 s ring | prediction |
| 09_4 → 09_5 | until K06 is read (11.3 s from its fade-in) | K06's model and conditions |
| 09_6 → 09_7 | `music_breath` 1.5 s, then 0.4 s of cleanup | the verdict box |
| 09_7 → 09_8 | until the names are read (2.3 s), then about 1.5 s of cleanup and the empty scale | names to search; the scale is up before "Even gaps" |
| 09_8 → end | until K07 is read (12.3 s), then `stand` | K07's conditions |

## How the source meets the assignment

**Numbers become letters through the same triangles (08).**

- 08_1 opens on chapter 05's picture: the stretch, its two triangles, and
  the wait axis, all in minutes. At "The gap of two", the row
  "2 min → ½ × (2 min)² = 2 min²" appears. A copy of the bracket's "2"
  flies to the row's measured first slot, and the row comes in as the copy
  goes. A faint mirror completes the square, and the triangle pulses. Ten
  follows the same way ("10 min → ½ × (10 min)² = 50 min²").
- Then "g → ½ × g² (g: a gap's length, min)": the 10 and the ½ are carried
  from the ten row into the letters, and "g" labels the long triangle's leg
  and base.
- 08_2 adds the areas carried from the rows and the minutes carried from
  the brackets: "average wait = (2 + 50) min² ÷ (2 + 10) min", then
  "= 52 min² ÷ 12 min = 4⅓ min". The wait gets a green halo before the same
  sum appears in letters.
- 08_3 divides top and bottom by 2, the number of gaps, carries each part
  into "average wait = average of g² ÷ (2 × average gap)", and puts our
  numbers under it.
- Sequential replacement: no number crossfades with another. A carried
  copy flies to its slot, measured from the row's own text (`Shown::piece`),
  and goes as the row comes in. Old rows leave before new ones take their
  place.

**Units are explicit (the assignment's minute-squared point).**

- Every area is in min², every gap and wait in min, and the division is
  written "52 min² ÷ 12 min = 4⅓ min".
- The bridge stays on screen from the first area row until 08_6: "at one
  passenger a minute, each min² of area is a minute of someone's waiting".
  It is 05_5's conversion again, so "two minutes of waiting" in the voice
  is never silent about the axes.
- **Proposed prose correction (optional; the draft's bytes are
  untouched).**
  - In 08_1, "The gap of two holds two minutes of waiting, and the gap of
    ten holds fifty." would become "The gap of two holds two square
    minutes, and the gap of ten holds fifty."
  - The voice would then say what the rows print, with the bridge left to
    the screen and to 05_5.
  - All three 08_1 waits survive the change.
  - Its SPOKEN row would become
    `("08_1", "holds two square minutes", "F-A1.short_area", "2")`.
  - If root keeps the draft's words, nothing else changes. The bridge note
    covers them.

**The spread is defined through computed geometry before its name (08_4).**

- Each triangle and its filled mirror make the square on its gap ("2² = 4
  min²", "10² = 100 min²"), and their average is carried into "average of
  g²: (4 + 100) ÷ 2 = 52 min²".
- The average gap's dim 6 × 6 square sits inside the long square
  ("(average gap)²: 6² = 36 min²"), and the violet 4 × 4 corner is
  "52 − 36 = 16 min²".
- Violet segments mark each gap's distance from the level 6, with "4"
  beside each, and "each gap is 4 from 6: (4² + 4²) ÷ 2 = 16 min²".
- The 16 is nonnegative because it is an average of squares, drawn.

**The square argument gets time, and is not the finite check.**

- After 08_4's words, the two strips (6 × 4) appear on the long square,
  and five rows come in one at a time, each held for its reading time:
  - "10² = 6² + 6 × 4 + 6 × 4 + 4²" and "2² = 6² − 6 × 4 − 6 × 4 + 4²";
  - "average: 52 = 36 + 16 (the strips cancel)";
  - only now the name: "spread: the average squared distance from the
    average gap = 16 min²";
  - "any gaps: average of g² = (average gap)² + spread".
- In all, about 18 s of silent inspection.
- 08_5 gives our numbers first: "½ × 6 min = 3 min", "16 min² ÷ (2 × 6 min)
  = 1⅓ min = 1 min 20 s", and the boxed "3:00 + 1:20 = 4:20" with a violet
  halo on 1:20. The letters follow: "average wait = ½ × average gap +
  spread ÷ (2 × average gap)".
- 08_6: the plot gives way to the sign and you. The violet 1:20 flies to
  "+1 min 20 s a morning" under you, the sign's "every 6 minutes" dims,
  and halos land on "buses" and on your morning.
- 08_8 states the uniform-time assumption in ink before the bound: "for
  passengers who turn up evenly in time, independently of the buses:",
  then "average wait ≥ ½ × average gap" and "exactly half only when every
  gap is the same". The draft's card is headed "a finite check, not the
  proof:", and K05 follows.

**The prediction is protected (08_7).**

- Before 08_7 nothing on screen holds "≥", "never", "at least", "half
  only" or "negative" (a test).
- The question sits over the sign with the think ring. The answer's row,
  "spread: an average of squares, never negative", comes only at "The
  extra piece is never negative", with the halo on the spread term.

**The seeded picture is not the model (09).**

- The illustration is a fixed seed (358), labelled "illustration", with
  no numbers on it. `fits` refuses a picture with:
  - touching markers;
  - a triangle off screen;
  - no tiny or no long gaps;
  - your moment within 1.2 min of a bus;
  - your gap shorter than the model's average. Passengers usually land in
    a longer gap (09_7), and seed 271 fails only this rule.
- The sample's own average gap is 6.53 min. Nothing prints it, and every
  number on screen is the model's ("(model)" on the box).

**The memoryless assumption is taught before the formula confirms it.**

- 09_1: a cursor sweeps the line. The bar "time since the last bus" grows
  and restarts at every bus, while the bar "chance a bus comes this
  moment" stays level and gets a halo at "just as likely to come".
- 09_4: "average wait: 6 min" lands, green, beside "average gap: 6 min
  (model)", then K06.
- 09_5: you, at your moment (chapters 02/03's). Everything behind you
  dims, and two rows are drawn from your moment:
  - "wait ahead of you: 6 min on average", green;
  - under it, from a bus mark at the same moment, "a new gap: 6 min on
    average".
  - They are the same passenger and a fresh gap, compared on screen before
    09_6's formula.
- 09_6: chapter 08's formula in letters, dim, then "spread = 6² = 36 min²
  (model)" and "3 min + 36 min² ÷ 12 min = 6 min", boxed green at "Six".

**Nothing is revealed before its question is answered.**

- Before 09_4: no "average wait:". The only "6 min" is the given average
  gap in the model box (a test).
- Before your gap: no "your gap" and no "12 min on average" (a test).
- In 09_7 the gold bracket marks your gap at "So the gap you land in". Its
  label "your gap: 12 min on average" waits for "averages twelve minutes".
- 09_6's "12 min" is twice the average gap, as the voice says it there.
  It is not the encountered gap.

**3 / 4:20 / 6, and past six (09_8).**

- The comparison leaves, the picture dims, and an empty scale comes in
  before the line: "average wait, a bus every 6 min on average".
- Marks land as said: "even gaps: 3 min", "completely random: 6 min", "our
  timetable: 4 min 20 s".
- At "More spread means a longer wait" a violet arrow "more spread" runs
  from the 3 onward. At "even past six" the explicit example lands: "gaps
  0, 0, 18 (average 6): 9 min".
- The scale runs on past it, unlabelled. Nothing calls any end a maximum
  (a test).
- K07 keeps both arrival conditions and the finite-moment condition, word
  for word. K06 keeps independence and long-run averages.

**Emphasis and fades.**

- Text emphasis uses separate stroke-only halo outlines (`halo`, around a
  measured text box or a box frame), never a flash on text.
- Open shapes are dimmed by stroke only. `fade_to`, which sets every alpha,
  is used only on filled shapes: the triangles' fills and the markers.
- Changing digits never crossfade.
- End positions are measured from the placed text (`Shown::piece`,
  `text_box`) before anything flies there.

## Departures from the draft's picture column

1. **08_1:** units on every row, the faint mirror squares, the bridge note,
   and "g" on the long triangle. The draft's "g → ½ g²" is written
   "g → ½ × g² (g: a gap's length, min)".
2. **08_2:** the sum uses the carried areas, "(2 + 50) min² ÷ (2 + 10) min
   = 52 min² ÷ 12 min = 4⅓ min", rather than re-expanding ½ × 2².
3. **08_3:** the division is written out as rows ("top and bottom ÷ 2, the
   number of gaps", "top: …", "bottom: …") before the formula.
4. **08_4:** besides the draft's squares and "each gap is 4 from 6", the
   silent square argument (above). The spread is named in it, after the
   corner and the distances.
5. **08_5:** numbers first, then letters. The draft shows the letters
   first, but the assignment and the storyboard's own rule say numbers
   before letters. The draft's single row is split into three, with units.
6. **08_6:** the draft's "street strip from 01" is the sign plus you at the
   stop (`timetable::sign`, `you`), and the 1:20 lands as "+1 min 20 s a
   morning".
7. **08_8:** the draft's flash text is three rows, the conditions on
   turning up are added, and the card gets its head line.
8. **09_1:** the two gauges replace "a dim clock face that does not reset".
   They show what resets (the time since the last bus) and what does not
   (the chance).
9. **09_2:** each triangle is teal under the model's average gap and gold
   from it on. The huge ones (≥ 12 min) pulse. Each piece's colour comes
   from its part on screen.
10. **09_5:** the wait ahead gets its own row beside the draft's "a new gap:
    6 min on average".
11. **09_6:** units, and the formula in letters shown dim above.
12. **09_7:** the forward arrow is 09_5's "wait ahead of you" row, kept;
    the back arrow is "since the last bus: 6 min on average"; the gold
    bracket joins them.
13. **09_8:** the explicit 0, 0, 18 mark at 9 min (the draft has it only in
    its notes). The arrow "more spread" starts at the 3.

## Tests

Not compiled. I ported the arithmetic and geometry to Python and checked
them there (below).

- **`the_formula.rs`** (10):
  - `the_chapter_runs_and_says_its_lines_in_order`
  - `the_cards_are_the_drafts`: K05, the check card, the question and the
    name of g, against the draft; K05 fits three lower-third lines
  - `the_rows_read_as_their_numbers`: the 28 rows with their units, each
    accent unique in its row
  - `the_formula_is_the_models_and_a_wrong_fact_is_refused`: negative
    controls on F-F1.spread, F-F1.penalty_clock, F-X1.cases,
    F-F1.area_factor, F-F1.squares and F-F1.distance
  - `the_square_argument_holds_for_every_choice_checked`: the identity, the
    strips cancelling, the spread ≥ 0 and zero only for even gaps, and the
    formula, exactly, on each of the 2,375 choices; an uneven pair exceeds
    half and an even pair equals it
  - `the_finite_check_counts_its_choices`: 2,375 from 0 min, 1,819 from 1
    min
  - `the_squares_tile_the_long_square`: the triangle is 05's; triangle plus
    mirror is the square; the four blocks tile the long square
  - `nothing_answers_the_question_before_it_is_asked`
  - `the_picture_stays_apart_and_on_screen`
  - `the_fixed_words_carry_no_numbers`
- **`at_random.rs`** (11):
  - `the_chapter_runs_and_says_its_lines_in_order`
  - `the_printed_words_are_the_drafts`: K06 and K07, two lines each
  - `the_numbers_are_the_models_and_a_wrong_fact_is_refused`: negative
    controls on F-R1.spread, F-R1.mean_wait, F-R1.person_gap,
    F-R2.mean_wait and F-R2.gaps
  - `the_illustration_is_fixed_and_fits`: nine buses, the first at 4.39
    min, the next after the hour at 60.62 min; seeds 1 to 357 fail; 271
    fails on your gap
  - `the_pieces_draw_the_wait`: the triangles' top edges are the wait, and
    their area equals a 600,000-step integral
  - `no_memory_and_the_formula_agree`
  - `the_scale_runs_on_past_six`
  - `nothing_is_revealed_before_its_line`
  - `the_time_since_the_last_bus_starts_again_at_each_bus`
  - `the_picture_stays_apart_and_on_screen`
  - `the_fixed_words_carry_no_numbers`

Only the two `the_chapter_runs_and_says_its_lines_in_order` tests need
integration, because they read `script.tsv`. Each plays its whole chapter
headless at 10 fps with the scratch voice. That covers every phrase the
chapter waits for, every fact it reads and every check its scene makes.

## Checks run (no Cargo)

- **rustfmt:** `rustfmt --edition 2024 --check` with the repo's
  `rustfmt.toml` is clean on both files. It is also clean on a scratch copy
  of the whole crate with the patch applied. That run parses every module,
  the two new ones included, as registered.
- **The patch:**
  - `git apply --check` passes against the live tree, which it does not
    modify.
  - Applied to scratch copies of the four files, it gives exactly my
    edited copies.
- **Static scans:**
  - no unused imports;
  - no dead items: the test-only helpers live in the test modules;
  - every `Rc`, `Narrator` and `Script` import in the test modules is
    used.
- **API, read at pin `a399d626`:**
  - `Stage::set_x` and `move_to` return `&mut Self`, with no `must_use`;
    `clear_updaters` returns `()`; `add_updater` returns a `Result`, which
    is handled.
  - `Arrow::color` sets fill and stroke.
  - The updater's `set_points(…, polyline(…).build().points())` and
    `Polygon::new(…).build()` match `no_bus_late.rs`, which root compiled.
  - The integer types in `formula`, `choices`, `random` and the tests were
    read for the 07 inference error root fixed (`0_i32..=120`). None is
    used both as i32 and i64.
- **Phrases, facts and cards:**
  - 8 + 8 lines are said in draft order, and the 28 + 27 waits hold as
    listed.
  - All 37 fact keys the files name are oracle keys.
  - K05 (244 characters), K06 (170, built from fixed pieces and two facts)
    and K07 (184) are the draft's exact versions, and so are the other 12
    quoted texts the chapters print.
  - Four negative controls are caught.
- **SPOKEN rows and pause overrides:** as in steps 3 and 4.
- **Python port** of the tests' arithmetic and geometry: 10,677 checks
  pass, with constants parsed from the Rust sources. They cover:
  - the 28 row texts equal to the Rust test's strings, and the 6 refused
    facts;
  - the square argument on all 2,375 choices (9,503 assertions);
  - the blocks tiling the square;
  - 08's wait-axis name top at y 3.738 (the frame's top is 4.0);
  - 08's panel and late moments apart (tightest pairs 0.173 and 0.100
    units), and K05 in three lines;
  - 09's seed rule, its 11 pieces with area 270.3879 min² equal to the
    integral, and the tallest top at y 3.144;
  - 09's six moments apart and on screen (tightest 0.082 units);
  - the marks, rows, cards and reveal checks.

  Three port negative controls are caught: an overlap, a failing seed and
  a three-line card.
- **Found and fixed by the port:** 09's layout test had treated the 09_7
  back arrow and the ahead row as separate boxes overlapping at you. As
  written, the test would have failed. They are now one box,
  `both_ways`.
- **Oracle:** `oracle.py --self-test` reports 0 failures. Its 166 facts are
  identical to the copy the checks used.

## Timing

These figures are the scratch voice at 2.6 words a second plus 0.4 s a
line, with a 0.6 s breath. The beats are transcribed by hand from the code.
They are emulations, not measurements.

| | storyboard.py's rule | scratch-voice emulation | of which authored silent holds |
|---|---|---|---|
| 08 | 84.8 s | 148.1 s | 32.4 s |
| 09 | 76.9 s | 111.0 s | 21.6 s |

Neither chapter was compressed toward the draft's estimate. The extra time
is the derivation's carries and holds, the silent square argument, the two
countdowns, the 09_1 sweep, and the reading holds for the check card, K05,
K06, K07 and the names.

These pictures land after their phrase:

| line | phrase | late by | why |
|---|---|---|---|
| 08_1 | "Any gap holds half its length squared" | 0.39 s | it follows the ten row's carry and pulse |
| 08_2 | "divide by the time they cover" | 0.26 s | it follows the first sum's carry and pulse |
| 08_8 | "The extra piece is never negative" | 0.07 s | the question's fade |

Authored gaps between lines:

- 08_3 and 08_4 each start 1.2 s after the previous breath, while the old
  rows leave and the letters move up before the words that act on them.
- The 09_1 sweep and its hold run 1.1 s past that line's breath.
- 09_6 → 09_7 has a 0.4 s cleanup, and 09_7 → 09_8 a 1.5 s scale build.

## Unresolved

1. **Uncompiled.** These are the calls most worth watching in the first
   build:
   - 08's `Shown::piece`/`source`/`moved` measuring a placed row's pieces
     (re-typeset text widths at the row's size);
   - `carry`'s `add_to_scene` copies;
   - `kit.text_t2c` accents within a row;
   - in 09_1, the `follow` updater on a point-free anchor;
   - in 09_1, `set_x` on markers made at 0;
   - `Arrow` with `buff(0.0)`;
   - in 08_8, `rect` added through `arena_mut` and then faded in.
2. **Prose correction** for 08_1 (above): optional, root's call.
3. **The silent square argument**, about 18 s, has no voice. If the
   preview reads it as too long without words, a spoken line could carry
   it. I drafted none, because the draft has none.
4. **Facts.** The formula's 2 (the reciprocal of F-F1.area_factor) has no
   fact of its own. `facts.rs` is root's.
5. **Text boxes.** Box sizes in the layout tests are estimates (0.0045
   units a point a character), not measured glyph boxes.
6. **Glyphs.** ≥ and § are unprobed until the chapters are registered.
7. **Intensities.** 0.4 and 0.5 are proposals.
8. **No listening claims.** Nothing here proves rendered clarity, natural
   pacing or English-accent consistency.

## The patch

The patch has 233 lines, sha256 `7797357c9ee02cb0` (first 16), and covers
the text between the fences. It was made with `git diff --no-index`
against copies of the files at the hashes above.

````diff
diff --git a/explainer_corpus/inspection_paradox/docs/pause-overrides.json b/explainer_corpus/inspection_paradox/docs/pause-overrides.json
index 821bcd8..d1b8a48 100644
--- a/explainer_corpus/inspection_paradox/docs/pause-overrides.json
+++ b/explainer_corpus/inspection_paradox/docs/pause-overrides.json
@@ -1,5 +1,5 @@
 {
-  "_comment": "Authored pause intentions for the active script (chapters 01 to 07), in tools/narration_director.py's override format, to splice into a map's \"overrides\" after `plan`. Each 'after' phrase occurs once in its cue. Ranges are editorial hypotheses from the storyboard's notes, not measurements or acceptance; nothing has been narrated or listened to. The chapter's prediction countdown and title hold are picture holds in the scene, not narration gaps. Chapters 08 to 12 have their pause intentions in docs/storyboard-draft.md and join here with their scenes.",
+  "_comment": "Authored pause intentions for the active script (chapters 01 to 09), in tools/narration_director.py's override format, to splice into a map's \"overrides\" after `plan`. Each 'after' phrase occurs once in its cue. Ranges are editorial hypotheses from the storyboard's notes, not measurements or acceptance; nothing has been narrated or listened to. The chapter's prediction countdown and title hold are picture holds in the scene, not narration gaps. Chapters 10 to 12 have their pause intentions in docs/storyboard-draft.md and join here with their scenes.",
   "overrides": [
     {
       "video": "inspection_paradox",
@@ -198,6 +198,39 @@
         0.9
       ],
       "reason": "The prediction's answer, while the ten equal triangles land."
+    },
+    {
+      "video": "inspection_paradox",
+      "cue": "08_8",
+      "after": "No",
+      "role": "thought",
+      "range_s": [
+        0.4,
+        0.8
+      ],
+      "reason": "The prediction's answer, while the violet spread term flashes, before the reason."
+    },
+    {
+      "video": "inspection_paradox",
+      "cue": "09_4",
+      "after": "Six minutes",
+      "role": "thought",
+      "range_s": [
+        0.6,
+        1.2
+      ],
+      "reason": "The chapter's surprise: the green answer box lands beside the model's average gap."
+    },
+    {
+      "video": "inspection_paradox",
+      "cue": "09_5",
+      "after": "The buses have no memory",
+      "role": "thought",
+      "range_s": [
+        0.45,
+        0.9
+      ],
+      "reason": "The assumption settles before you land on the line and the wait ahead is drawn."
     }
   ]
 }
diff --git a/explainer_corpus/inspection_paradox/docs/storyboard.md b/explainer_corpus/inspection_paradox/docs/storyboard.md
index 1e81d6c..bd79247 100644
--- a/explainer_corpus/inspection_paradox/docs/storyboard.md
+++ b/explainer_corpus/inspection_paradox/docs/storyboard.md
@@ -340,3 +340,75 @@ who plan.
   - "Three minutes." 0.4 to 0.9 s;
   - the listen window in 07_5 is a visual hold (the sweep), five seconds of
     picture with no voice.
+
+## 08_the_formula: The formula, from the triangles (~85 s)
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 08_1 | Now turn the triangles into a formula for any timetable. The gap of two holds two minutes of waiting, and the gap of ten holds fifty. Any gap holds half its length squared. | The two triangles from 05 at left. A formula panel builds: "2 → ½ × 2² = 2", "10 → ½ × 10² = 50", then "g → ½ g²" (g labelled on screen "a gap's length") | TeX derivation, `TransformMatchingTex` numbers → letters | |
+| 08_2 | Add up the triangles and divide by the time they cover. That's the average wait. | "average wait = (½ × 2² + ½ × 10²) ÷ (2 + 10) = 52 ÷ 12", then letters below: "(sum of ½ g²) ÷ (sum of g)" | TeX derivation | |
+| 08_3 | Divide the top and the bottom by the number of gaps. The top becomes half the average of the squared gaps. The bottom becomes the average gap. | The letters morph: "average wait = average of g² ÷ (2 × average gap)". Under it the numbers "52 ÷ (2 × 6)" | `TransformMatchingTex` | |
+| 08_4 | For our gaps, two squared is four and ten squared is a hundred, so the average square is fifty-two. Six squared is only thirty-six. The extra sixteen measures how uneven the gaps are. | Squares drawn to scale: 2 × 2 teal and 10 × 10 gold, "average area 52"; a dim 6 × 6 square, "36"; the difference "16" in violet, named "spread"; "each gap is 4 from 6, and 4² = 16" | squares from facts, highlight | |
+| 08_5 | So the average wait is half the average gap, three minutes, plus sixteen over twice six, which is a minute and twenty seconds. There's the extra time. | "average wait = ½ × average gap + spread ÷ (2 × average gap)". Under it "3 min + 16 ÷ 12 min = 3:00 + 1:20 = 4:20", the 1:20 in violet, boxed in green | TeX derivation, verdict box | |
+| 08_6 | For you at the stop, that minute and twenty seconds is the cost of uneven gaps. The sign can't show it, because it counts buses, not mornings. | The street strip from 01 returns at the top, with you at the stop. The violet "1:20" flies from the formula to you; the sign's "every 6 minutes" dims beside it | callback, move | |
+| 08_7 | Can the average wait ever be less than half the average gap? ⟨predict⟩ | Question card "ever less than half the average gap?"; countdown | think countdown | |
+| 08_8 | No. The extra piece is never negative, so for passengers who turn up at random, the average wait is at least half the average gap. It's exactly half only when every gap is the same. | The violet term flashes: "spread ≥ 0, and spread = 0 only for even gaps". A small dim check card: "every choice of up to 4 gaps (repeats allowed, order ignored), each 0 to 12 min: 2,375 choices, 0 below half; equal only when all the gaps are the same". That is F-X1, a finite check; the square argument is the proof. A "precisely" card (K05). It stands, then the chapter fades | flash, card, lower-third | |
+
+### Notes for 08
+
+- Numbers before letters. Every step is shown with 2 and 10 before g.
+- "spread" is defined by its picture (the violet 16) before it is named.
+  "Variance" appears once, in K05.
+- The commuter anchor (08_6) restates the variance term as your 1:20 a
+  morning (F-W1.extra_clock equals F-F1.penalty_clock).
+- Exact version K05 (08_8): "for passengers turning up uniformly in time,
+  independently of the buses: average wait = E[G²] ÷ (2 E[G]) = E[G]/2 +
+  Var(G) ÷ (2 E[G]); equal to E[G]/2 exactly when Var(G) = 0. Larson &
+  Odoni, Urban Operations Research, §2.13, 'random incidence'".
+- The always-sentence (08_8) is tested at its edges:
+  - even gaps give equality (oracle "even gaps" and F-X1.equal_iff_even);
+  - zero gaps are included in the sweep;
+  - a strictly uneven pair exceeds half.
+- Pause intentions:
+  - "No." 0.4 to 0.8 s;
+  - after "There's the extra time." a thought pause, 0.6 to 1.1 s;
+  - after "not mornings." the cue transition before the prediction.
+
+## 09_at_random: No timetable at all (~77 s)
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 09_1 | What if there's no timetable at all? Suppose buses come completely at random. At every moment a bus is just as likely to come, however long it's been since the last bus. | A fresh hour line. Markers appear at irregular moments (a seeded illustration, labelled "illustration", with no numbers on it). Over each new marker a dim clock face that does not reset | moves, seeded picture | |
+| 09_2 | Keep the same average, a bus every six minutes. Now some gaps are tiny and some are long, and the long ones hold huge triangles. | The wait triangles rise above the irregular line, a few tiny and a few huge; "average gap 6 min (model)" | shapes, label | |
+| 09_3 | How long is the average wait now? ⟨predict⟩ | Question card "average wait now?"; countdown | think countdown | |
+| 09_4 | Six minutes. The whole average gap, not half of it. | "average wait: 6 min" in green beside "average gap: 6 min (model)", and a "precisely" card (K06) | equation card, lower-third | |
+| 09_5 | The buses have no memory. So from the moment you turn up, the wait ahead of you is just like a brand new gap, and a gap averages six minutes. | You land on the line. Everything behind you dims, and the wait ahead is drawn as a fresh gap labelled "a new gap: 6 min on average" | highlight | |
+| 09_6 | The formula agrees. For completely random gaps the spread is the average gap squared, thirty-six. Three minutes, plus thirty-six over twelve. Six. | "spread = 6² = 36 (model)"; "3 + 36 ÷ 12 = 6" with the 3 dim and the 36 ÷ 12 violet, boxed in green | TeX derivation | |
+| 09_7 | Look back too, and the last bus left six minutes before you, on average. So the gap you land in averages twelve minutes, twice the average gap. | Arrows back and forward from you, "6 min" each, joined into "your gap: 12 min on average". Lower-third "names to search: waiting-time paradox, bus paradox" | arrows, lower-third | |
+| 09_8 | Even gaps wait three minutes, random buses wait six, and our timetable sits at four twenty. More spread means a longer wait, even past six. The formula needs only the average gap and the spread. | A dim scale of average wait for a bus every 6 minutes on average: "even gaps: 3 min", "our timetable: 4 min 20 s", "completely random: 6 min", the scale running on past 6 with an arrow "more spread". A "precisely" card (K07). It stands, then the chapter fades | number line, lower-third | |
+
+### Notes for 09
+
+- The irregular markers are a seeded picture, not data. Nothing measured is
+  shown, and every number on screen is a model number labelled "model".
+- Exact version K06 (09_4): "model: buses as a Poisson process, one every 6
+  minutes on average; gaps exponential, mean 6, variance 36; passengers
+  turn up independently of the buses; long-run averages". The oracle uses
+  exact moments k! μ^k. Its self-test adds quadrature and a seeded
+  simulation of 200,000 arrivals (5.969 min); neither is shown.
+- Exact version K07 (09_8): "the formula holds for passengers who turn up
+  independently of the buses and spread evenly in time, when the gaps have
+  a long-run average and spread (finite mean and finite mean square)".
+  Both conditions on turning up are needed. Someone who turns up at :02:30
+  every day is independent of a fixed timetable, yet waits 9:30 every day,
+  not 4:20.
+- The 09_7 search names come from the one source read on them: Masuda and
+  Hiraoka's abstract says Terada "described the waiting-time paradox, also
+  called the bus paradox". No book section is cited on the card, because
+  none was read (brief, "Located, not read"). The line makes no claim
+  about where real services sit on the scale. "Even past six" is checked:
+  gaps 0, 0, 18 keep the average gap at 6 and wait 9 (F-R2). The scale's
+  right end is never labelled a worst case.
+- Pause intentions:
+  - "Six minutes." 0.6 to 1.2 s, the chapter's surprise;
+  - after "The buses have no memory." 0.45 to 0.9 s.
diff --git a/explainer_corpus/inspection_paradox/src/chapters/mod.rs b/explainer_corpus/inspection_paradox/src/chapters/mod.rs
index 58451b2..7f505d8 100644
--- a/explainer_corpus/inspection_paradox/src/chapters/mod.rs
+++ b/explainer_corpus/inspection_paradox/src/chapters/mod.rs
@@ -4,16 +4,18 @@
 //!
 //! Only chapters with native scene source are registered, and they are
 //! exactly the chapters of `docs/storyboard.md` and `script.tsv`: chapters
-//! 01 through 07. Chapters 08 to
+//! 01 through 09. Chapters 10 to
 //! 12 are written as words and pictures in `docs/storyboard-draft.md`, the
 //! full draft. Each moves into the storyboard and this registry when root
 //! assigns it and its scene source exists, never as a stand-in.
 
+pub mod at_random;
 pub mod draw;
 pub mod glyphs;
 pub mod no_bus_late;
 pub mod one_a_minute;
 pub mod stretch_one_gap;
+pub mod the_formula;
 pub mod the_stop;
 pub mod timetable;
 pub mod triangles;
@@ -60,6 +62,16 @@ pub fn registry() -> Vec<Chapter> {
             make: |kit| Box::new(no_bus_late::NoBusLate { kit }),
             intensity: 0.45,
         },
+        Chapter {
+            key: "08_the_formula",
+            make: |kit| Box::new(the_formula::TheFormula { kit }),
+            intensity: 0.4,
+        },
+        Chapter {
+            key: "09_at_random",
+            make: |kit| Box::new(at_random::AtRandom { kit }),
+            intensity: 0.5,
+        },
     ]
 }
 
diff --git a/explainer_corpus/inspection_paradox/src/main.rs b/explainer_corpus/inspection_paradox/src/main.rs
index 1cd8a98..18801a6 100644
--- a/explainer_corpus/inspection_paradox/src/main.rs
+++ b/explainer_corpus/inspection_paradox/src/main.rs
@@ -241,6 +241,56 @@ mod tests {
         ("07_6", "the average is six", "F-O1.max_wait", "6"),
         ("07_7", "from three minutes to six", "F-O1.min_wait", "3"),
         ("07_7", "from three minutes to six", "F-O1.max_wait", "6"),
+        // 08: the formula from the triangles; the spread; never below half
+        ("08_1", "The gap of two", "F-T1.short", "2"),
+        ("08_1", "holds two minutes of waiting", "F-A1.short_area", "2"),
+        ("08_1", "the gap of ten", "F-T1.long", "10"),
+        ("08_1", "holds fifty", "F-A1.long_area", "50"),
+        ("08_1", "half its length squared", "F-F1.area_factor", "1/2"),
+        (
+            "08_3",
+            "half the average of the squared gaps",
+            "F-F1.area_factor",
+            "1/2",
+        ),
+        (
+            "08_4",
+            "two squared is four and ten squared is a hundred",
+            "F-F1.squares",
+            "4,100",
+        ),
+        ("08_4", "two squared", "F-T1.short", "2"),
+        ("08_4", "ten squared", "F-T1.long", "10"),
+        ("08_4", "the average square is fifty-two", "F-F1.mean_square", "52"),
+        ("08_4", "Six squared", "F-T1.mean_gap", "6"),
+        ("08_4", "Six squared is only thirty-six", "F-F1.mean_gap_squared", "36"),
+        ("08_4", "The extra sixteen", "F-F1.spread", "16"),
+        ("08_5", "half the average gap, three minutes", "F-F1.area_factor", "1/2"),
+        ("08_5", "half the average gap, three minutes", "F-T1.half_mean_gap", "3"),
+        ("08_5", "plus sixteen", "F-F1.spread", "16"),
+        ("08_5", "over twice six", "F-T1.mean_gap", "6"),
+        ("08_5", "a minute and twenty seconds", "F-F1.penalty_clock", "1:20"),
+        ("08_6", "that minute and twenty seconds", "F-W1.extra_clock", "1:20"),
+        ("08_7", "less than half the average gap", "F-X1.bound_factor", "1/2"),
+        ("08_8", "at least half the average gap", "F-X1.bound_factor", "1/2"),
+        ("08_8", "exactly half", "F-X1.bound_factor", "1/2"),
+        // 09: buses at random; no memory; the gap you land in; the scale
+        ("09_2", "a bus every six minutes", "F-R1.mean_gap", "6"),
+        ("09_4", "Six minutes.", "F-R1.mean_wait", "6"),
+        ("09_4", "not half of it", "F-X1.bound_factor", "1/2"),
+        ("09_5", "a gap averages six minutes", "F-R1.mean_gap", "6"),
+        ("09_6", "the average gap squared, thirty-six", "F-R1.spread", "36"),
+        ("09_6", "Three minutes", "F-R1.half_mean", "3"),
+        ("09_6", "plus thirty-six", "F-R1.spread", "36"),
+        ("09_6", "over twelve", "F-R1.twice_mean", "12"),
+        ("09_6", "Six.", "F-R1.mean_wait", "6"),
+        ("09_7", "six minutes before you", "F-R1.mean_age", "6"),
+        ("09_7", "averages twelve minutes", "F-R1.person_gap", "12"),
+        ("09_7", "twice the average gap", "F-R1.person_gap_ratio", "2"),
+        ("09_8", "Even gaps wait three minutes", "F-O1.even_wait", "3"),
+        ("09_8", "random buses wait six", "F-R1.mean_wait", "6"),
+        ("09_8", "sits at four twenty", "F-W1.mean_wait_clock", "4:20"),
+        ("09_8", "even past six", "F-R1.mean_wait", "6"),
     ];
 
     /// The words the guard below listens for: every number word the full
````
