# Chapters 06 and 07: native source and integration

2026-10-24, GoldGate, for root (the assignment in Mail 6882). Native scene
source for `06_two_averages` and `07_no_bus_late` sits in two new files.
Nothing registers them, so the build does not compile them yet. None of it
has been compiled, rendered, narrated or listened to. Every check below is a
Python model or a source reading, not native execution.

## Files

| file | sha256 (first 16) | bytes | what it is |
|---|---|---|---|
| `src/chapters/two_averages.rs` | `78325de488aa368a` | 52025 | chapter 06, `TwoAverages` |
| `src/chapters/no_bus_late.rs` | `83d8c3f7140d7f25` | 53935 | chapter 07, `NoBusLate` |

No new shared helper. Each chapter keeps its few local helpers (a halo, a
box, a level line) private in its own file. Neither imports the 04/05 files
(`twelve_minutes.rs`, `one_a_minute.rs`, `triangles.rs`), so 06/07 can be
registered on top of either production state.

I built on the following files and did not edit them. They match the
starting state of this assignment:

| file | sha256 (first 8) |
|---|---|
| `main.rs` | 3ab2f991 |
| `chapters/mod.rs` | e587dd9e |
| `draw.rs` | e51741b3 |
| `timetable.rs` | 66c43d84 |
| `glyphs.rs` | 7aacf404 |
| `model.rs` | bc3be4ad |
| `facts.rs` | a3cd5b3e |
| `palette.rs` | aeb18f55 |
| `script.tsv` | 8d544850 |
| `docs/storyboard.md` | b9befb21 |
| `docs/storyboard-draft.md` | b2eb19fc |
| `docs/pause-overrides.json` | 0ab89237 |
| `oracle.py` | 47a2c058 |

The delivered 01–05 sources are unchanged too:

| file | sha256 (first 8) |
|---|---|
| `the_stop.rs` | 1432b763 |
| `two_ways_to_count.rs` | 8e5d0386 |
| `stretch_one_gap.rs` | 9087c041 |
| `twelve_minutes.rs` | 61206a6f |
| `one_a_minute.rs` | 747ac26e |
| `triangles.rs` | 44df1f81 |
| `integration-04-05.md` | b841a390 |

## Integration edits, in order (root's)

I suggest doing these after the 04/05 integration
(`docs/integration-04-05.md`), so the storyboard and the registry stay
contiguous.

1. **`src/chapters/mod.rs`:** declare the two modules in alphabetical order
   (`pub mod no_bus_late;` after `pub mod glyphs;`, and
   `pub mod two_averages;` after `pub mod timetable;`). Then register both
   chapters after 05:

   ```rust
   Chapter {
       key: "06_two_averages",
       make: |kit| Box::new(two_averages::TwoAverages { kit }),
       intensity: 0.5,
   },
   Chapter {
       key: "07_no_bus_late",
       make: |kit| Box::new(no_bus_late::NoBusLate { kit }),
       intensity: 0.45,
   },
   ```

   The intensities are proposals. Then update the module comment to
   "chapters 01 through 07", with chapters 08 to 12 still in the draft.

2. **`docs/storyboard.md`:** after the 05 section, append these two sections
   byte for byte from `docs/storyboard-draft.md`, notes included:
   - `## 06_two_averages: Ask the buses, ask the passengers (~70 s)`
   - `## 07_no_bus_late: No bus was late (~82 s)`

   Keep them in the draft too, because main.rs's
   `the_full_draft_holds_the_storyboard_verbatim` needs both copies. Each
   hash covers `"\n## "` plus the section, with trailing whitespace
   stripped:

   | section | sha256 (first 16) | bytes |
   |---|---|---|
   | 06 | `b0a978957ec9ce44` | 3194 |
   | 07 | `6805f1aeda3a644f` | 4264 |

   The `~70 s` and `~82 s` in the headings are the draft's estimates, not
   limits. Neither chapter was cut to fit them (see Timing).

3. **`script.tsv`:** run `python3 -I tools/storyboard.py inspection_paradox`
   (this slug only), then
   `python3 -I tools/storyboard.py --check inspection_paradox`. This adds
   the fourteen lines 06_1 to 07_7.

4. **`src/main.rs`, `SPOKEN`:** append the rows below. They are rustfmt's
   layout for this repo (max_width 120), run through rustfmt in a copy of
   main.rs's context.
   - Each phrase is in its draft line, and each value equals the oracle's
     value for its key.
   - All 24 distinct keys are added literally in `facts.rs`.
   - 06_3, 06_6, 07_1 and 07_2 say no number word, so they have no row.
   - No line uses a pronoun "one", so `PRONOUN_ONES` needs no change.
   - "the two ways of counting" (06_1) is bound to F-S1.averages (2): the
     two ways give the two averages that 06_4 names.

   ```rust
           // 06: two averages, over buses and over passengers
           ("06_1", "the two ways of counting", "F-S1.averages", "2"),
           ("06_1", "the tags say six minutes", "F-S1.bus_mean_gap", "6"),
           ("06_2", "One in six", "F-S1.person_short_share", "1/6"),
           ("06_2", "says two minutes", "F-T1.short", "2"),
           ("06_2", "Five in six", "F-S1.person_long_share", "5/6"),
           ("06_2", "say ten", "F-T1.long", "10"),
           (
               "06_4",
               "Eight minutes and forty seconds",
               "F-S1.person_gap_clock",
               "8:40",
           ),
           ("06_4", "A sixth of the passengers", "F-S1.person_short_share", "1/6"),
           ("06_4", "passengers say two", "F-T1.short", "2"),
           ("06_4", "five sixths", "F-S1.person_long_share", "5/6"),
           ("06_4", "say ten", "F-T1.long", "10"),
           ("06_4", "two honest averages", "F-S1.averages", "2"),
           ("06_5", "waits half that gap", "F-S1.wait_share_of_gap", "1/2"),
           ("06_5", "Half of eight minutes forty", "F-S1.wait_share_of_gap", "1/2"),
           ("06_5", "Half of eight minutes forty", "F-S1.person_gap_clock", "8:40"),
           ("06_5", "is four minutes twenty", "F-S1.half_person_gap_clock", "4:20"),
           ("06_7", "left four minutes twenty", "F-S1.age_clock", "4:20"),
           ("06_7", "Behind you four twenty", "F-S1.age_clock", "4:20"),
           ("06_7", "ahead of you four twenty", "F-W1.mean_wait_clock", "4:20"),
           (
               "06_7",
               "the eight forty of your gap",
               "F-S1.age_plus_wait_clock",
               "8:40",
           ),
           // 07: no bus was late; only the spacing changes
           ("07_3", "six minutes after the blue", "F-O1.even_offset", "6"),
           ("07_3", "Every gap is six minutes", "F-O1.even_gap", "6"),
           ("07_4", "Three minutes", "F-O1.even_wait", "3"),
           ("07_4", "Half of six", "F-O1.even_gap", "6"),
           ("07_4", "Half of six", "F-S1.wait_share_of_gap", "1/2"),
           ("07_5", "The same ten buses an hour", "F-G1.buses_every_offset", "10"),
           ("07_5", "average gap of six minutes", "F-G1.mean_gap_every_offset", "6"),
           ("07_6", "Two minutes apart", "F-T1.offset", "2"),
           ("07_6", "it's four minutes twenty", "F-O1.wait.d2", "4:20"),
           ("07_6", "Six apart", "F-O1.best_offset", "6"),
           ("07_6", "it's three", "F-O1.min_wait", "3"),
           ("07_6", "the two buses come together", "F-T1.routes", "2"),
           ("07_6", "every twelve minutes", "F-O1.together_period", "12"),
           ("07_6", "the average is six", "F-O1.max_wait", "6"),
           ("07_7", "from three minutes to six", "F-O1.min_wait", "3"),
           ("07_7", "from three minutes to six", "F-O1.max_wait", "6"),
   ```

5. **`docs/pause-overrides.json`:** append these four entries to
   `overrides`, then change `_comment` to say chapters 01 to 07. The ranges
   come from the draft's pause notes. They are editorial hypotheses, not
   measurements.

   I ran each entry through `tools/narration_director.py`'s own
   `directions` on the draft's 87 lines. Each entry names exactly one
   boundary and replaces the punctuation default there. The two "Same
   buses, same gaps" commas default to `clause`; the entries make them
   `connected`, as the draft asks. Two negative controls were refused:
   "same", which occurs twice in 06_4, and a phrase not in its cue.

   ```json
   {"video": "inspection_paradox", "cue": "06_4", "after": "Eight minutes and forty seconds", "role": "thought", "range_s": [0.5, 1.0], "reason": "The prediction's answer lands, and the gold level settles, before the reasons."},
   {"video": "inspection_paradox", "cue": "06_4", "after": "Same buses", "role": "connected", "range_s": [0.0, 0.28], "reason": "\"Same buses, same gaps\" is one run into the two averages: no clause break inside it."},
   {"video": "inspection_paradox", "cue": "06_4", "after": "same gaps", "role": "connected", "range_s": [0.0, 0.28], "reason": "The run carries straight on to \"two honest averages\"."},
   {"video": "inspection_paradox", "cue": "07_4", "after": "Three minutes", "role": "thought", "range_s": [0.4, 0.9], "reason": "The prediction's answer, while the ten equal triangles land."}
   ```

   The draft also asks for a 0.5–1.0 s thought "after 'two honest
   averages.'". That boundary is 06_4's cue end. Its default
   (`cue_transition`, 0.45–1.4 s) already contains the range, and the
   scene's 0.6 s breath before 06_5 falls inside it. No existing override
   sits at a cue end, so I added none there. Root can add one if the
   measured cut needs it.

   The 07_5 "listen 5 s" is a visual hold, not a narration gap. The scene
   holds with `kit.music_breath(stage, 5.0)` while the sweep runs, so it
   needs no entry.

6. **Glyphs:** 06's passenger sum prints "8⅔" (U+2154), which
   `glyphs::DIGITS` does not list. The probe collects whatever registered
   scenes typeset, so no `glyphs.rs` edit is needed for it to be probed.
   The bundled Computer Modern lacks ⅔, but it also lacks ½, ⅙, ⅚, × and −,
   which earlier chapters already print. At pin `a399d626`,
   `FontBook::glyph_fallback` takes such a character from the chain
   Computer Modern → Noto Sans Math → IBM Plex Sans, and I read IBM Plex
   Sans Regular's cmap: it maps ⅔ along with the others. So ⅔ should come
   from the same face as ⅙ and ⅚. That is a cmap reading, not a probe run.
   Adding ⅔ to `DIGITS` would keep that list complete, but it is optional.

Then root's usual order: `cargo fmt --check`, Clippy with `-D warnings`,
the crate's tests, `inspection_paradox glyphs`, and a preview.

## How the source meets the assignment

**The same timetable and the same passengers.**

- 06_1 rebuilds chapter 03's hour with `timetable::timetable` (no sign, as
  the draft's picture has none):
  - markers at `model::bus_times(OFFSET)`;
  - `timetable::bracket` over every gap, labelled from the facts;
  - each bus's tag (`model::gap_before_each_bus`);
  - the sixty passengers at `model::passengers()`, each colored by its gap.
- 06_5's ride runs through *your* gap, the long gap holding
  `timetable::your_moment()` (6½ min). The scene refuses to run if that gap
  is not a long one.
- 07 opens on the same hour, markers and dots. In 07_3 and 07_5 the red
  markers slide, and the dots repaint live by the same gold/teal rule as
  the triangles. They are the same sixty passengers, not new ones.

**Bus weighting and passenger weighting differ visibly before the
passenger formula.**

- 06_1, averaged over buses: one bar per bus is born at its tag and flies
  into a row, as tall as its tag says. The row is ten equal slots, gold
  first, as in 02_6. A level line grows at the row's average, labelled
  "6 min", beside the caption "one bar per bus / 5 say 2, 5 say 10".
- 06_2, averaged over passengers: each dot shows its answer. At "One in
  six" the short gaps' answers turn teal and their bars fly from the
  answers into a second row. At "Five in six" the gold ones follow. The
  caption reads "one bar per passenger / 10 say 2, 50 say 10". It is the
  same rule and the same width, now with sixty bars, fifty of them tall.
- 06_3 asks with both rows on screen and no passenger level (the draft's
  prediction note).
- 06_4: at "Eight minutes and forty seconds" the passenger row's level
  grows (gold "8 min 40 s"). Only then, at "A sixth of the passengers",
  does the sum "⅙ × 2 + ⅚ × 10 = 8⅔ min = 8 min 40 s" appear.
- **For root to decide:** the draft's 06_1 picture writes the bus sum
  "(5 × 2 + 5 × 10) ÷ 10 = 6 min" in 06_1, which is before the passenger
  row exists. I kept it there, at "the number on the sign", after the bus
  row and its level, so it reads as that row's arithmetic. The weighting
  contrast (ten bars against sixty, two levels) still comes before any
  share or the ⅙/⅚ formula. If root reads "before formula work" strictly,
  the bus sum's fade can move to 06_4 beside the passenger sum. The bus
  level and its "6 min" label would still show 06_1's "six minutes".

**The half-gap and inspection time.**

- 06_5: the rows and sums leave, and the boxes rise. You appear in your
  gold gap at "turns up", and the part still ahead of you is drawn over the
  gap as you ride steadily (2.6 s) through it. At "waits half that gap" you
  come to its middle, and "average: half the gap" appears with K03.
- The passenger box gets a halo at "Half of eight minutes forty". The green
  "½ × 8 min 40 s = 4 min 20 s" appears at "four minutes twenty", and gets
  a halo at "The same answer".
- The chapter holds until K03 has been up for `read_s(K03)` (10 s).
- 06_6 holds the names lower-third for its reading time. 06_7 ends on
  `stand`.

**The no-late-bus counterexample.**

- 07_1: a green dot on every bus and "every bus on time" at "No bus broke
  down". The
  timetable's printed rows, ":00 :12 …" and ":02 :14 …", appear at
  "written into the timetable".
- 07_2: the bunching inset. Two markers on a loop, the one behind closing
  from half a loop to its nearest and never passing (a tested property).
  K04 appears at "a different story". At "simply uneven" the long brackets
  flash: those brackets are the timetable's own gaps, not delays.

**The controlled spacing experiment, one cause at a time.**

- 07_3: one tracker `d`, the red bus's offset after the blue. Updaters keep
  every red marker at its offset, clamped clear of the blue ones (tested
  for every offset), and repaint the dots. `d` tweens from 2 to 6 at "Go
  back to the even timetable". The even brackets ("6") replace ours, which
  have already faded, so old and new numbers never overlap. Then the
  question and its countdown.
- 07_4: ten triangles (teal and gold by turn), each as tall as its gap is
  wide, plus the green "average wait: 3 min" box. At "Half of six" the
  even brackets flash, and at "what the sign suggests" the sign's "every 6
  min" gets a halo. At "agree" the boxes "over buses: 6 min" and "over
  passengers: 6 min" appear.
- 07_5: the plot frame, the slider's name "red bus after blue (min)", the
  point and the live readout. The readout first fades in as a still text
  and is then swapped for the `always_redraw` one at the same anchor, so it
  never crossfades.
- At "The same ten buses" and "the same average gap" the sign's two lines
  get halos: what stays fixed is shown first. At "Only the wait changes"
  the scene clock drives `d` through 6 → 0 → 12 → 2 (1.6 + 3.8 + 1.6 s),
  on through the 5 s silent hold. The triangles, dots, markers, point and
  readout all follow the one tracker.
- 07_6: `d` returns to each spoken offset in turn, and a green point plus
  its clock mark the U: 4:20 at 2, 3:00 at 6, 6:00 at 0. At "every twelve
  minutes" the blue markers flash, each with its red marker just beside it.
- 07_7: the sign's card gets a halo at "same sign", and marks 1 and 2 get
  halos at "from three minutes to six". At "Only the spacing changed" `d`
  goes back to our 2 minutes.

**Mail 6882's cautions.**

- Text emphasis uses separate stroke-only halo outlines. Flashes are only
  on shapes: markers, bracket lines and dots.
- Neither file uses `set_opacity` or any dimming of brackets. Dim things
  are created in the dim color.
- No old/new number pair crossfades. The 07_3 brackets are sequential, and
  the readout swap is between identical texts.

## Departures from the draft's picture column

1. **06_1:** besides the draft's sum, the bus row has its bars, a level
   line labelled "6 min" and a caption. The sum appears after the row (see
   above).
2. **06_2:** the draft's "tiny tag of its gap" on each dot is a small dim
   answer ("2" or "10") under the dot. It takes its gap's color as the
   voice gives each share. The draft's tally "10 say 2, 50 say 10" is the
   passenger row's caption.
3. **06_4:** before the sum, the passenger row's level grows, gold, at "8
   min 40 s". At "Same buses, same gaps" the bus markers flash.
4. **06_5:** "one gap highlighted" is your gap's gold span, with the part
   still ahead drawn over it as you ride.
5. **06_6:** before the names, the boxes get halos in turn, the long
   brackets flash at "long gaps last longer", and the long-gap passengers
   flash at "get picked more".
6. **06_7:** the gap from inside is drawn to scale, on its own row under
   the line, so its arrows never cross the hour's markers. Each arrow is
   its average long (4⅓ min, 0.6 a minute), and the gold bracket under
   both is 8⅔ min. Every label says "on average", so a single morning's
   gap is not implied.
7. **07_1:** the "every bus on time" mark is a small green dot over every
   marker, with the words once in the middle.
8. **07_5:** the slider is the plot's own offset axis, named "red bus after
   blue (min)". The draft writes "d min", but the video introduces no
   letters before chapter 08. The sweep runs 6 → 0 → 12 → 2: it starts
   from the even timetable where 07_4 left it and ends at ours. The draft
   says "from 0 to 12 and back to 2".
9. **07_5:** the readout between whole minutes is the draft's one rounding
   (`model::clock_rounded`, F-O1.between_readout "4:01" at 2.5 min).
   Nothing spoken or boxed uses it.
10. **07_7:** halos on the sign and on the 3:00 and 6:00 marks, then the
    return to 2 minutes at "Only the spacing changed".

## Tests

Not compiled. I ported the arithmetic and geometry to Python and checked
them there (below).

- **`two_averages.rs`** (9):
  - `the_chapter_runs_and_says_its_lines_in_order`
  - `the_printed_words_are_the_drafts`
  - `the_card_is_the_drafts_exact_version`: K03, against the draft
  - `the_numbers_are_the_timetables_and_a_wrong_fact_is_refused`: negative
    controls on F-S1.person_gap, F-S1.passengers_in_long and F-S1.age_clock
  - `each_row_levels_off_at_its_average`
  - `the_gap_from_inside_is_to_scale`
  - `numbers_print_as_the_screen_writes_them`
  - `the_picture_stays_apart_and_on_screen`
  - `the_fixed_words_carry_no_numbers`
- **`no_bus_late.rs`** (11):
  - `the_chapter_runs_and_says_its_lines_in_order`
  - `the_printed_words_are_the_drafts`: including K04
  - `the_plan_is_the_models_and_a_wrong_fact_is_refused`: negative controls
    on F-O1.wait.d2, F-O1.even_wait, F-T1.red_times and
    F-O1.between_readout
  - `the_readout_rounds_only_between_whole_seconds`
  - `the_red_bus_never_covers_a_blue_one`
  - `the_triangles_hold_the_average_wait`
  - `the_plot_traces_the_models_u`
  - `the_sweep_runs_end_to_end_and_home`
  - `the_bus_behind_closes_but_never_passes`
  - `the_picture_stays_apart_and_on_screen`
  - `the_fixed_words_carry_no_numbers`

Only the two `the_chapter_runs_and_says_its_lines_in_order` tests need
integration, because they read the lines from `script.tsv`. The card and
printed-word tests read `storyboard-draft.md`, which already holds both
chapters. Each `runs` test plays its whole chapter headless at 10 fps with
the scratch voice. That covers every phrase the chapter waits for, every
fact it reads and every check its scene makes.

## Checks run (no Cargo)

- **rustfmt:** `rustfmt --edition 2024 --check` is clean on both new files
  with the repo's `rustfmt.toml`.
- **Static scans:**
  - Unused imports: the only finding is `Timed` in `two_averages.rs`. It is
    a trait used through `.rt(` in `grow_with`.
  - Dead items: none in either file.
- **Phrases, facts and cards:**
  - 7 + 7 lines are said in draft order.
  - Each of the 23 + 25 phrases the chapters wait for occurs once in its
    line, after the previous one. This includes 06_2's loop array.
  - All 43 fact keys the files name are oracle keys.
  - K03 (150 characters, built from three fixed pieces and three facts) and
    K04 (136 characters) are the draft's exact versions.
  - By the kit's greedy wrap at 92 characters, both are two lines.
- **SPOKEN rows:**
  - All 36 rows hold.
  - The guard passes for all 14 lines.
  - A stricter check finds every number word inside some row's phrase.
  - Negative controls (a wrong value, a missing phrase) are caught.
- **Pause overrides:** as in step 5.
- **Python port of the tests' arithmetic and geometry:** 22,978 checks
  pass, with constants parsed from the Rust sources and exact `Fraction`s
  where the Rust compares exactly. They cover:
  - the 26 picture-against-fact checks of `averages` and the 85 of `plan`,
    each with the Rust tests' negative controls refused;
  - the printed sums, tallies, labels, boxes and card;
  - rows levelling at 6 and 8⅔, gold first, every slot inside;
  - the inside gap to scale;
  - the readout at all 13 whole offsets and at 2.5 min, clamped past either
    end;
  - the closed-form slider wait equal to `oracle.py`'s independent
    second-by-second `slider_wait` at every whole offset and at 2.5 min,
    and the rounding equal to the oracle's `readout` at all 121 tenths;
  - red never covering blue at 1201 offsets × 5;
  - the triangles' area over the hour equal to the wait at every tenth;
  - the U's lowest point at 6;
  - the sweep's ends, its smoothness (largest step 0.0066 min per
    1/10000), and that it outlasts the silent window;
  - the loop closing and never passing;
  - both chapters' estimated text boxes, apart and on screen. The tightest
    pairs are 0.104 units (06) and 0.103 units (07).

  The port's own negative controls are caught too: a box moved onto the
  sign, and half-tall triangles.
- **Oracle:** `oracle.py --self-test` reports 0 failures. Its output (166
  facts) is identical to the copy the checks used.
- **Found and fixed by the port:** both layout tests had modelled the
  lower-third band at x ±7.1 and y −4.0, exactly on their strict
  `on_screen` bounds, so as written they would have failed. They now use
  `props::lower_third`'s band: 0.4 in from each side (FRAME_X_RADIUS
  7.111), foot at −3.8, 0.3 + 0.36 a line tall. No other chapter file uses
  that pattern.

## Timing

These figures are the scratch voice at 2.6 words a second plus 0.4 s a
line, with a 0.6 s breath. The beats are transcribed by hand from the code.
They are emulations, not measurements.

| | storyboard.py's rule | scratch-voice emulation |
|---|---|---|
| 06 | 70.8 s | 83.7 s |
| 07 | 82.1 s | 94.5 s |

Neither chapter was compressed toward an estimate. The extra time is the
row building, the 06_5 ride, K03's and K04's reading holds, the two
countdowns, and the 07_5 sweep with its silent hold. In the emulation the
sweep (7.0 s) ends 0.66 s after the 5 s window.

These pictures land after their phrase:

| line | phrase | late by | why |
|---|---|---|---|
| 07_4 | "Half of six" | 0.73 s | it follows the triangles and the box; the authored 0.4–0.9 s thought after "Three minutes." absorbs it |
| 07_4 | "what the sign suggests" | 0.32 s | |
| 06_6 | first words | 0.5 s | the previous picture's fade |
| 07_3 | first words | 0.5 s | the previous picture's fade |

Everything else lands within 0.2 s.

## Unresolved

1. **Uncompiled.** I checked the API use against the pinned fmn
   (`a399d626`) by reading it, not by compiling. These are the calls most
   worth watching in the first build:
   - in 07, the `add_updater`/`add_dt_updater` anchors (`move_reds`,
     `paint`, `reshape`, `follow`, `drive`) on fresh `VMobject`s added to
     the scene, reached through `Stage`'s deref;
   - in 07_5, the `always_redraw` readout swapped in with
     `remove_from_scene`/`add_to_scene`;
   - in 06_5, the `ride` updater rewriting the ahead span's points with
     `set_points`;
   - in 06, `grow_from_edge` on the level lines and the 06_7 arrows.
2. **The bus sum's place** (see above): kept in 06_1 per the draft. It can
   move to 06_4.
3. **Facts.** 07_4's "over passengers: 6 min" prints F-O1.even_gap, after
   `plan` checks it equals `model::passengers_gap` at the even offset. A
   separate fact (say F-O1.even_person_gap) would make the receipt name it
   directly. That is optional, and `facts.rs` is root's.
4. **Text boxes.** Box sizes in the layout tests are estimates (0.0045
   units a point a character), not measured glyph boxes.
5. **Glyphs.** ⅔ is unprobed until the chapters are registered.
6. **Intensities.** 0.5 and 0.45 are proposals.
7. **No listening claims.** Nothing here proves rendered clarity, natural
   pacing or English-accent consistency.
