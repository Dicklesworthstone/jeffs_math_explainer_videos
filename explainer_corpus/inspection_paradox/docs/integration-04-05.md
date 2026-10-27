# Chapters 04 and 05: native source and integration

2026-10-24, GoldGate, for root (the assignment after Mail 6862). Native scene
source for `04_one_a_minute` and `05_triangles` sits in three new files. The
build does not compile them yet, because nothing registers them. None of it
has been compiled, rendered, narrated or listened to.

## Files

| file | sha256 (first 16) | bytes | what it is |
|---|---|---|---|
| `src/chapters/twelve_minutes.rs` | `61206a6f6ae1ff08` | 29823 | shared helper for 04 and 05: the stretch, its passengers, the numbers, the picture |
| `src/chapters/one_a_minute.rs` | `747ac26e1ed363d3` | 33437 | chapter 04, `OneAMinute` |
| `src/chapters/triangles.rs` | `44df1f81e273cf67` | 44523 | chapter 05, `Triangles` |

`twelve_minutes.rs` covers four things:

- **The stretch:** the first twelve minutes of chapter 03's timetable, drawn
  as a data window in which a minute up is as long as a minute across.
- **Its passengers:** one at the middle of each minute.
- **The numbers:** every number either chapter prints. Each is read from the
  facts and checked against the stretch.
- **The picture:** the stretch picture and the bars.

I built on the following files and did not edit them. They match the state
root integrated for 02 and 03:

| file | sha256 (first 8) |
|---|---|
| `main.rs` | 3ab2f991 |
| `chapters/mod.rs` | e587dd9e |
| `draw.rs` | e51741b3 |
| `timetable.rs` | 66c43d84 |
| `the_stop.rs` | 1432b763 |
| `two_ways_to_count.rs` | 8e5d0386 |
| `stretch_one_gap.rs` | 9087c041 |
| `glyphs.rs` | 7aacf404 |
| `model.rs` | bc3be4ad |
| `facts.rs` | a3cd5b3e |
| `palette.rs` | aeb18f55 |
| `script.tsv` | 8d544850 |
| `docs/storyboard.md` | b9befb21 |
| `docs/storyboard-draft.md` | b2eb19fc |
| `docs/pause-overrides.json` | 0ab89237 |
| `oracle.py` | 47a2c058 |

## Integration edits, in order (root's)

1. **`src/chapters/mod.rs`:** declare the three modules and register both
   chapters after 03.

   ```rust
   pub mod draw;
   pub mod glyphs;
   pub mod one_a_minute;
   pub mod stretch_one_gap;
   pub mod the_stop;
   pub mod timetable;
   pub mod triangles;
   pub mod twelve_minutes;
   pub mod two_ways_to_count;
   ```

   ```rust
   Chapter {
       key: "04_one_a_minute",
       make: |kit| Box::new(one_a_minute::OneAMinute { kit }),
       intensity: 0.5,
   },
   Chapter {
       key: "05_triangles",
       make: |kit| Box::new(triangles::Triangles { kit }),
       intensity: 0.5,
   },
   ```

   The intensities are proposals. Then update the module comment:
   "chapters 01 through 05", and "Chapters 06 to 12" are in the draft.

2. **`docs/storyboard.md`:** after the 03 section, append these two sections
   byte for byte from `docs/storyboard-draft.md`, notes included:
   - `## 04_one_a_minute: Twelve passengers (~85 s)`
   - `## 05_triangles: The waiting is an area (~79 s)`

   Keep them in the draft too. main.rs's
   `the_full_draft_holds_the_storyboard_verbatim` needs both copies. Each
   hash below covers `"\n## "` plus the section, with its trailing newlines
   stripped:

   | section | sha256 (first 16) | bytes |
   |---|---|---|
   | 04 | `2ac7466d809ff00f` | 4250 |
   | 05 | `b5e72bf863217b05` | 3365 |

3. **`script.tsv`:** run `python3 -I tools/storyboard.py inspection_paradox`
   (this slug only), then
   `python3 -I tools/storyboard.py --check inspection_paradox`. This adds the
   sixteen lines 04_1 to 05_8. My checks derived each line from the draft as
   storyboard.py does. That derivation gives back script.tsv's text for all
   21 integrated lines.

4. **`src/main.rs`, `SPOKEN`:** append the rows below. They are already in
   rustfmt's layout for this repo (max_width 120).
   - Each phrase is in its draft line, and each value equals the oracle's
     value for its key.
   - All 43 keys are written literally in `facts.rs`.
   - 05_2 and 05_3 say no number word, so they have no row.

   Five lines use "one" as a pronoun, not a count:
   - "zoom in on one stretch" (04_1)
   - "how long each one waits" (04_2)
   - "your lucky one" (04_4)
   - "The long one" (05_5)
   - "the short one" (05_6)

   Each of those lines has rows of its own, so `PRONOUN_ONES` needs no change.

   ```rust
           // 04: twelve minutes, twelve passengers, the waits
           ("04_1", "the same twelve minutes", "F-T1.period", "12"),
           ("04_1", "five times over", "F-T1.cycles_per_hour", "5"),
           ("04_1", "two minutes later", "F-T1.offset", "2"),
           ("04_1", "ten minutes after that", "F-T1.long", "10"),
           ("04_2", "twelve of our passengers", "F-W2.people", "12"),
           ("04_2", "one in the middle of each minute", "F-W2.per_minute", "1"),
           ("04_3", "The two passengers in the short gap", "F-W2.short_people", "2"),
           (
               "04_3",
               "a minute and a half, and half a minute",
               "F-W2.short_waits",
               "3/2,1/2",
           ),
           ("04_3", "One minute each", "F-W1.short_mean_wait", "1"),
           ("04_4", "The ten passengers in the long gap", "F-W2.long_people", "10"),
           (
               "04_4",
               "nine and a half minutes down to half a minute",
               "F-W2.long_waits_range",
               "19/2,1/2",
           ),
           ("04_4", "Five minutes each", "F-W1.long_mean_wait", "5"),
           ("04_5", "all twelve passengers", "F-W2.people", "12"),
           ("04_6", "Not three", "F-T1.half_mean_gap", "3"),
           ("04_6", "The two passengers in the short gap", "F-W2.short_people", "2"),
           ("04_6", "waited two minutes between them", "F-W2.short_total", "2"),
           ("04_6", "The ten in the long gap", "F-W2.long_people", "10"),
           ("04_6", "waited fifty", "F-W2.long_total", "50"),
           ("04_7", "fifty-two minutes of waiting", "F-W2.total", "52"),
           ("04_7", "shared by twelve passengers", "F-W2.people", "12"),
           ("04_7", "Four and a third minutes each", "F-W2.mean_wait", "13/3"),
           (
               "04_7",
               "four minutes and twenty seconds",
               "F-W1.mean_wait_clock",
               "4:20",
           ),
           ("04_8", "Half of six", "F-T1.mean_gap", "6"),
           ("04_8", "the average of one and five", "F-W1.short_mean_wait", "1"),
           ("04_8", "the average of one and five", "F-W1.long_mean_wait", "5"),
           ("04_8", "five times as many", "F-W2.people_ratio", "5"),
           // 05: the waiting is an area
           ("05_1", "those twelve minutes", "F-T1.period", "12"),
           ("05_4", "our twelve passengers", "F-W2.people", "12"),
           ("05_4", "a bar one minute wide", "F-A1.bar_width", "1"),
           (
               "05_4",
               "exactly as much as the triangles",
               "F-A1.bars_equal_triangles",
               "true",
           ),
           ("05_5", "half of two times two", "F-T1.short", "2"),
           ("05_5", "which is two", "F-A1.short_area", "2"),
           ("05_5", "half of ten times ten", "F-T1.long", "10"),
           ("05_5", "which is fifty", "F-A1.long_area", "50"),
           ("05_5", "the fifty-two minutes", "F-A1.total_area", "52"),
           ("05_6", "five times as wide", "F-A1.width_ratio", "5"),
           ("05_7", "Twenty-five times", "F-A1.area_ratio", "25"),
           ("05_7", "five times as wide", "F-A1.width_ratio", "5"),
           ("05_7", "five times as tall", "F-A1.height_ratio", "5"),
           ("05_7", "Five times as many passengers", "F-W2.people_ratio", "5"),
           ("05_7", "five times as long on average", "F-A1.mean_wait_ratio", "5"),
           ("05_8", "Double a gap", "F-A1.double", "2"),
           ("05_8", "four times the waiting", "F-A1.double_area_ratio", "4"),
   ```

5. **`docs/pause-overrides.json`:** append these five entries to
   `overrides`. Then change `_comment` to say chapters 01 to 05, with 06 to
   12 still in the draft. The draft's pause notes for 04 and 05 give the
   ranges, which are editorial hypotheses, not measurements.

   I ran each entry through `tools/narration_director.py`'s own
   `directions` on the sixteen lines. Each entry names exactly one boundary,
   and each replaces the punctuation default there. The default for 04_3's
   comma is `clause`; the override makes it `connected`. Two negative
   controls were refused: "minute", which occurs three times in 04_3, and a
   phrase that is not in its cue.

   ```json
   {"video": "inspection_paradox", "cue": "04_3", "after": "a minute and a half", "role": "connected", "range_s": [0.0, 0.28], "reason": "The two waits are one listing: no clause break inside it."},
   {"video": "inspection_paradox", "cue": "04_4", "after": "Five minutes each, on average", "role": "thought", "range_s": [0.35, 0.7], "reason": "The long gap's average settles before the callback to your two mornings."},
   {"video": "inspection_paradox", "cue": "04_6", "after": "Not three", "role": "thought", "range_s": [0.4, 0.8], "reason": "The guess is struck out before the two sums are read."},
   {"video": "inspection_paradox", "cue": "05_4", "after": "as tall as their wait", "role": "thought", "range_s": [0.6, 1.2], "reason": "The bars finish widening before \"The bars hold\"."},
   {"video": "inspection_paradox", "cue": "05_7", "after": "Twenty-five times", "role": "thought", "range_s": [0.5, 1.0], "reason": "The answer to the prediction, while the first tiles land."}
   ```

   The draft also asks that the cue break after 04_6's "waited fifty." stay
   at the default. That needs no entry.

6. **Glyphs:** the scenes set two characters that `glyphs::DIGITS` does not
   list:
   - "…" in 04_6's long sum;
   - "→" in 05_8's card.

   The probe collects whatever registered scenes typeset, so registering the
   chapters is enough to get them probed. No `glyphs.rs` edit is needed.
   "×", "÷", "½" and "⅓" are already in `DIGITS`.

Then root's usual order: `cargo fmt --check`, Clippy with `-D warnings`,
the crate's tests, `inspection_paradox glyphs`, and a preview.

## How the source meets the assignment

**The same timetable and passengers.**

- 04_1 starts from chapter 03's hour: `hour_line`, every bus marker, the
  `timetable::bracket` pair and the sixty passengers.
- It zooms into the first twelve minutes with `replacement_transform` of the
  same objects:
  - the line's first twelve minutes → the stretch's axis;
  - the three markers;
  - both brackets' lines and labels;
  - the twelve dots.

  Everything else fades. "× 5 in an hour" stays beside the stretch.
- 05 opens on that stretch with its bars, built by the same helper.
- Positions are the model's:
  - markers at `model::bus_times(OFFSET)`;
  - dots at `model::passengers()` (a minute's middle);
  - bar heights from `model::wait_at`, checked equal to
    `midpoint_waits`.

**Waiting against the whole gap.**

- 04_3 and 04_4 stand each passenger's wait, as a bar, beside its gap's
  bracket, so the bracket's length and the bars' heights can be compared.
- In 05_2 you ride the line, and a point above you traces your wait. The
  wait is the whole gap only just after a bus ("wait the whole gap"), with
  the bracket label flashing. It shrinks to nothing at the next bus.
- The short ride stops a millionth of a minute before the red bus, so the
  jump back up comes with "then jumps back up". The test
  `the_trace_is_the_sawtooth` pins this.
- 05_3 closes the trace into the triangles: as wide as the gap (the bracket
  labels flash) and as tall (the wait axis labels and a level line).

**Computed counts and checked numbers.**

- `twelve_minutes::numbers` reads every printed number from the facts and
  refuses a fact that disagrees with the stretch:
  - the waits against `midpoint_waits`;
  - each gap's total and average;
  - the mornings (F-M1);
  - W2.total and the mean wait, against `model::mean_wait` and W1.mean_wait;
  - the guess (half the mean gap, equal to the naive mean of the gaps'
    averages);
  - each area as `triangle_area`, equal to ½·g·g and to its bars' total;
  - the width, height and area ratios;
  - the tiling counts (`model::tiling`), summing to the area ratio;
  - the doubling.
- Before 05 starts, it counts its tiles and checks them against
  F-A1.tiles.*. It checks the four half-size triangles against
  F-A1.double_area_ratio.
- 04 checks F-X2.midpoint_exact before it starts.

**Each quantity defined before it is used.**

- A gap's average comes after its waits are shown and said.
- The sums are built before "52 ÷ 12".
- K02, the "precisely" card for minute-middle passengers, appears with the
  verdict.
- The wait axis is named in 05_1 before the ride.
- 05_4 shows that the bars hold the triangles before any area is computed in
  05_5.
- "× 5" (05_6) comes before "× 25" (05_7).

**Observation and prediction time.** There is no runtime ceiling, as root
set.

- 04_5 and 05_6 each have a `think` countdown, with no sum or tile on
  screen.
- 04_7 holds K02 until `read_s(K02)` has passed.
- 05_2's long ride runs about 2.9 s past its line on the scratch voice. That
  is silent watching of the long slant.
- 05_7 holds, then breathes 1.5 s over the finished tiling.
- Each chapter ends with `stand` before its fade.

## Departures from the draft's picture column

1. 04_1: "× 5 in an hour" appears right after the zoom, beside the stretch.
2. 04_2: the two ghosts are rings only, without chapter 01's dot. This is
   the ghost-ring lesson: an undimmed fill would hide the dots.
3. 04_6: the short sum reads "1½ + ½ = 2" in the bars' order. The draft
   writes "½ + 1½". The long sum is the draft's "9½ + 8½ + … + ½ = 50".
4. 05_2: the "wait: m:ss" readout sits left-aligned in the panel's top row,
   rounded to whole seconds (`model::clock_rounded`). It does not ride
   beside you.
5. 05_3:
   - There are no new braces. The brackets' labels carry the widths, and
     the wait axis labels 2 and 10 with a level line carry the heights.
   - You, the traced point, the trace, the readout and the bus markers leave
     as the triangles fill in. Each triangle's vertical edge stands where
     its bus stood.
6. 05_4: the "flash" of the triangles and corners is a fill pulse, since
   `indicate` would scale a triangle off its axes. The corners appear, then
   the corners and bars fade.
7. 05_5: the area labels sit in the panel, not inside the triangles. The
   short triangle is one scene unit square, too small for "½ × 2 × 2 = 2".
8. 05_6: the "× 5" is an arrow under the brackets' lengths, between the two
   gaps' midpoints, not a brace.
9. 05_7: the count "15 + 10 = 25" appears in the panel after the passengers
   flash, before the hold.
10. 05_8: the first half-size triangle's outline appears, then the three
    more that fill the long one, lagged (`tiling(2)` = 3 + 1), with the card
    "gap × 2 → waiting × 4". The draft says "then the whole 10".

## Tests

Not compiled. I ported the arithmetic and geometry to Python and checked
them there (below).

- **`twelve_minutes.rs`:**
  - `a_minute_up_is_a_minute_across`
  - `the_stretch_is_the_timetables_first_twelve_minutes`
  - `the_numbers_are_the_stretches_and_a_wrong_fact_is_refused`: a negative
    control on F-W2.long_total
  - `waits_print_as_the_screen_writes_them`
  - `every_bar_stands_inside_the_frame`
  - `the_rows_under_the_line_keep_apart`
  - `the_fixed_words_carry_no_numbers`
- **`one_a_minute.rs`:**
  - `the_chapter_runs_and_says_its_lines_in_order`
  - `the_card_is_the_storyboards_exact_version`
  - `the_sums_print_in_the_bars_order`
  - `each_average_is_colored_once`: with a negative control
  - `the_ghosts_ring_your_mornings_clear_of_the_markers`
  - `the_picture_stays_apart_and_on_screen`
  - `the_fixed_words_carry_no_numbers`
- **`triangles.rs`:**
  - `the_chapter_runs_and_says_its_lines_in_order`
  - `the_wait_falls_steadily_and_jumps_at_each_bus`
  - `the_trace_is_the_sawtooth`
  - `each_triangle_holds_its_gaps_waiting`
  - `the_corners_a_bar_adds_are_the_corners_it_leaves`
  - `the_tiles_fill_the_long_triangle`: also checks the halves and refuses a
    3-minute tile
  - `a_shrunk_tile_keeps_its_place`
  - `the_readout_writes_the_wait_as_a_clock`
  - `the_picture_stays_apart_and_on_screen`
  - `the_fixed_words_carry_no_numbers`

Three tests pass only after integration:

- the two `the_chapter_runs_and_says_its_lines_in_order` tests need the
  lines in `script.tsv`;
- `the_card_is_the_storyboards_exact_version` needs the 04 section in
  `storyboard.md`.

Each `runs` test plays its whole chapter headless at 10 fps with the scratch
voice. That covers every phrase the chapter waits for, every fact it reads
and every check the scene makes.

## Checks run (no Cargo)

- **rustfmt:** `rustfmt --edition 2024 --check` on each new file is clean
  with the repo's `rustfmt.toml`.
- **Static scans:**
  - Unused imports: the only finding is `Timed`. It is a trait used through
    `.rt(` (2, 2 and 5 calls), each on an `Animation` value, as in
    `draw.rs`.
  - Dead items: none.
  - Every field of `Numbers`, `Stretch`, `Hour`, `WaitAxis` and `Tile`, and
    every pub helper, is read outside tests.
- **Phrases, facts and K02:**
  - 8 + 8 lines are said in draft order.
  - Each of the 24 + 23 phrases the chapters wait for occurs once in its
    line, after the previous one. This includes those in loop arrays.
  - All 40 fact keys the files name are oracle keys.
  - K02 (153 characters) is the draft's exact version, word for word.
- **SPOKEN rows:**
  - All 43 rows hold: phrase in line, value equal to the oracle's.
  - The guard passes for all 16 lines.
  - A stricter check finds every number word inside a row's phrase or a
    listed pronoun "one".
  - Negative controls (a wrong value, a missing phrase) are caught.
- **Pause overrides:** as in step 5.
- **Python port of the tests' arithmetic and geometry:** 1565 checks pass,
  with constants parsed from the Rust sources and exact `Fraction`s:
  - the mixed-number forms, refusing ⅔, ¼ and −½;
  - the sums and the guess's other form;
  - each bar's corners: equal, ⅛ each, with the slant through the bar's
    top middle;
  - each bar's area equal to the area under its slant, and the gaps'
    totals equal to 2 and 50;
  - the 25 tiles covering the long triangle exactly, by lattice points off
    every edge: no hole, no overlap, nothing outside;
  - the four halves, likewise;
  - the tile counts 15 and 10;
  - shrunk tiles staying inside their places;
  - the sawtooth, and the short ride's stop;
  - the readouts;
  - both chapters' estimated text boxes.

  The negative controls are all caught: a missing tile, inverted tiles
  allowed past the slant, a third-minute corner, an overlapping panel row,
  and an average equal to the count.
- **Oracle:** `oracle.py --self-test` reports 0 failures. Its output, 166
  facts, is the copy the checks used.
- **Timing emulation:** the scratch voice at 2.6 words a second plus 0.4 s a
  line, with a 0.6 s breath. The beats are transcribed from the code.

  | | storyboard.py's rule | scratch-voice emulation |
  |---|---|---|
  | 04 | 84.8 s | 96.8 s |
  | 05 | 79.2 s | 95.3 s |

  Three findings were fixed in `triangles.rs` before this handoff:
  - 05_1's stretch now fades in before its line;
  - the 05_2 rides run at 0.5 s a minute, and the jump now comes on its
    words;
  - the 05_7 tiles fly with the "× 25" (2.2 s), and the count waits until
    after the flashes.

## Unresolved

1. **Uncompiled.** I checked the API use by reading the pinned fmn
   (`a399d626`), not by compiling. These are the calls most worth watching
   in the first build:
   - In 05_7, an `AnimationGroup` of lagged `replacement_transform`s of
     detached copies, passed to `Stage::prepare` beside a fade. The group
     comes through the blanket `IntoAnimation` impl, and `begin_animations`
     roots its members.
   - In 05_2, the `always_redraw` readout swapped in for a still text at the
     same anchor, and the updater-driven rider.
   - In 05_1, `grow_from_edge` on the wait axis.
   - In 05_4, the bars widened by `AnimBuilder::stretch` about their centres.
2. **Timing.**
   - On the scratch voice, 05_7's ratio flash lands 0.9 s after "five times
     as wide". The authored 0.5–1.0 s pause after "Twenty-five times."
     would absorb it.
   - 05_2's long ride runs 2.9 s past its line. I kept that as watching
     time.
   - Both are for root's measured pass. Neither is a listening claim.
3. **Text boxes.** Box sizes in the layout tests are estimates (0.0045 units
   a point a character), not measured glyph boxes.
4. **Glyphs.** "…" and "→" are unprobed in this crate until the chapters are
   registered.
5. **Intensities.** 0.5 and 0.5 are proposals.
