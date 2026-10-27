# Chapters 02 and 03: native source and integration

2026-10-24, GoldGate, for root (Mail 6832). Native scene source for
`02_two_ways_to_count` and `03_stretch_one_gap` sits in three new files the
build does not compile yet: nothing registers them. None of it has been
compiled, rendered, narrated or listened to.

## Files

| file | sha256 (first 16) | bytes | what it is |
|---|---|---|---|
| `src/chapters/timetable.rs` | `248b9b273dc3556a` | 19049 | shared helper: gaps, brackets, spans, tags, passengers, the sign, playback |
| `src/chapters/two_ways_to_count.rs` | `cd0e6c515b46ea94` | 32042 | chapter 02, `TwoWaysToCount` |
| `src/chapters/stretch_one_gap.rs` | `2c3c55df1f584823` | 37024 | chapter 03, `StretchOneGap` |

Unchanged and frozen: `main.rs` c367132d…, `chapters/mod.rs` 2ed83578…,
`chapters/draw.rs` e51741b3…, `the_stop.rs` badd29d0…, `glyphs.rs` 7aacf404…,
`model.rs` bc3be4ad…, `facts.rs` a3cd5b3e…, `palette.rs` aeb18f55…,
`script.tsv` e598c79c…, `docs/storyboard.md` 2b049070…,
`docs/storyboard-draft.md` fcda4bf8…, `oracle.py` 47a2c058….

## Integration edits, in order (root's)

1. `src/chapters/mod.rs`: declare the modules and register both chapters.
   Then update the module comment, which says only chapter 01 is
   registered. The intensities are proposals.

   ```rust
   pub mod draw;
   pub mod glyphs;
   pub mod stretch_one_gap;
   pub mod the_stop;
   pub mod timetable;
   pub mod two_ways_to_count;
   ```

   ```rust
   Chapter {
       key: "02_two_ways_to_count",
       make: |kit| Box::new(two_ways_to_count::TwoWaysToCount { kit }),
       intensity: 0.45,
   },
   Chapter {
       key: "03_stretch_one_gap",
       make: |kit| Box::new(stretch_one_gap::StretchOneGap { kit }),
       intensity: 0.5,
   },
   ```

2. `docs/storyboard.md`: append the `## 02_two_ways_to_count` and
   `## 03_stretch_one_gap` sections, notes included, copied byte for byte
   from `docs/storyboard-draft.md`. Keep them in the draft too; main.rs's
   `the_full_draft_holds_the_storyboard_verbatim` requires both copies.
3. `script.tsv`: run `python3 tools/storyboard.py inspection_paradox` (this
   slug only), then `python3 tools/storyboard.py --check inspection_paradox`.
   This adds the fourteen lines 02_1 to 03_7.
4. `src/main.rs`, `SPOKEN`: append the rows below. Each phrase is in its
   draft line, and each value equals the oracle's value for its key. 02_3
   has no row because no fact backs it (see "Unresolved" below).

   ```rust
           // 02: two ways to count
           ("02_1", "Two minutes", "F-T1.short", "2"),
           ("02_1", "ten minutes", "F-T1.long", "10"),
           ("02_1", "Five short gaps", "F-T1.short_count", "5"),
           ("02_1", "five long ones", "F-T1.long_count", "5"),
           ("02_2", "Half the tags say two minutes", "F-S1.bus_short_share", "1/2"),
           ("02_2", "say two minutes", "F-T1.short", "2"),
           ("02_2", "half say ten", "F-T1.long", "10"),
           ("02_2", "they average six", "F-S1.bus_mean_gap", "6"),
           ("02_5", "Five in six", "F-S1.person_long_share", "5/6"),
           ("02_5", "one in the middle of every minute", "F-W2.per_minute", "1"),
           ("02_5", "Sixty in the hour", "F-S1.passengers", "60"),
           ("02_5", "fifty of them", "F-S1.passengers_in_long", "50"),
           ("02_6", "fifty of the sixty minutes", "F-T1.long_minutes", "50"),
           ("02_6", "of the sixty minutes", "F-T1.hour", "60"),
           ("02_6", "Half the gaps", "F-T1.long_share_of_gaps", "1/2"),
           ("02_6", "five sixths of the time", "F-T1.long_time_share", "5/6"),
           ("02_7", "Each gap gets one tag", "F-S1.tags_per_gap", "1"),
           // 03: stretch one gap
           ("03_1", "six minutes after the blue", "F-O1.even_offset", "6"),
           ("03_1", "every gap is six minutes", "F-O1.even_gap", "6"),
           ("03_2", "Half the buses end one", "F-G1.bus_share_every_offset", "1/2"),
           ("03_2", "half the passengers stand in one", "F-G1.share.d6", "1/2"),
           ("03_3", "The same ten buses", "F-G1.buses_every_offset", "10"),
           ("03_3", "every six minutes on average", "F-G1.mean_gap_every_offset", "6"),
           ("03_4", "Half of them still end", "F-G1.bus_share_every_offset", "1/2"),
           (
               "03_4",
               "Seven of every twelve, then eight, nine, ten",
               "F-G1.spoken_of_12",
               "7,8,9,10",
           ),
           ("03_4", "of every twelve", "F-T1.period", "12"),
           ("03_6", "a twelve-minute gap", "F-O1.together_period", "12"),
           ("03_6", "a bus every six minutes", "F-G1.mean_gap_every_offset", "6"),
           ("03_7", "counts each gap once", "F-S1.tags_per_gap", "1"),
   ```

   03_5's "the blue one" is already exempt (`PRONOUN_ONES`).

5. `docs/pause-overrides.json`: the draft's pause intentions for 02 and 03,
   in the file's format. Each `after` phrase occurs once in its cue. The
   other intentions ("five sixths of the time.", "The two counts agree.")
   fall at line ends, where the gap between cues already sits.

   ```json
   {"video": "inspection_paradox", "cue": "02_3", "after": "That's not how you see it", "role": "thought", "range_s": [0.35, 0.7], "reason": "The turn from the bus company's view to yours."},
   {"video": "inspection_paradox", "cue": "02_5", "after": "Five in six", "role": "thought", "range_s": [0.5, 1.0], "reason": "The answer lands before the passengers start to drop in."},
   {"video": "inspection_paradox", "cue": "03_4", "after": "the gap that's growing", "role": "thought", "range_s": [0.3, 0.6], "reason": "The frozen 1/2 settles before the passengers' count moves."},
   {"video": "inspection_paradox", "cue": "03_6", "after": "All of them", "role": "thought", "range_s": [0.5, 1.0], "reason": "The last red marker lands beside the blue."}
   ```

6. Optional:
   - Move `timetable::short_gap()` (teal) into `palette.rs`.
   - Make `draw::MARKER_W` public. A chapter 03 test keeps a copy of 0.14,
     with a comment; the scene itself measures the marker.

The glyph probe needs nothing new. Every fixed string is a constant in its
chapter, and the new characters "(" and ")" come from 03_6's readout, which
the probe collects when it runs the scene.

## How the source meets the assignment

- **Same timetable, tags and passengers.** Both chapters build from
  `timetable::timetable(.., OFFSET)`, `model::gap_before_each_bus` and
  `model::passengers()`. You are the passenger at 6½ minutes, in the middle
  of the hour's first long gap. Chapter 03 opens on chapter 02's picture
  (tags 10 and 2; 50 gold dots and 10 teal) and ends on it.
- **1/2 against 5/6, geometry before symbols.** 02_6 flies one cell per tag
  and one dot per passenger from their places on the line into two rows of
  equal width, gold first. The gold run is half of one row and five sixths
  of the other before any word or fraction appears. "Half the gaps" brings
  the tags' caption. "five sixths" brings the passengers' caption and
  "50/60 = 5/6".
- **The controlled offset experiment.**
  - One tracker, the red bus's offset, is the only thing that moves.
    Updaters keep the red markers, the gold gaps and every passenger's color
    true to it between whole minutes. The ten buses and the sign stay.
  - At +6 the two counts agree, 1/2 and 1/2: the negative control.
  - Each minute step brings new tags, a new passengers' readout and a new
    point on a plot. The points fall on the line from (0, none) to
    (12, all).
  - 03_4's steps land on "eight", "nine" and "ten", followed by a 3 s hold
    with music only.
- **Prediction holds.** 02_4 shows no dots until 02_5, and 03_5 stops the red
  markers at +2 with nothing moving. Both use `draw::think`.
- **No draft script reference in the current build.** The new files say
  only their own ids through `say`. They are unregistered, so the build
  does not compile them, and the `SPOKEN` and storyboard checks see none
  of their lines.

## Departures from the draft's picture column

- 02: the sign fades at 02_3, when the bus company's view gives way to
  yours.
- 02_3: your moment is 6½ minutes, not chapter 01's mornings (2½ and 11½).
  Half a minute from a bus, your ring (radius 0.16) would cross the marker
  (half width 0.07; half a minute is 0.1 on the line).
- 03_2: the "gold outline" is a thick gold span over the line, from each red
  marker to the next blue one.
- 03_3: the plot and its first point appear with the first step, in 03_3,
  rather than in 03_4.
- 03_6:
  - The red bus steps 2 → 1 → 0 (0.6 s a step) rather than sliding in one
    move, so the plot gets its 11-of-12 point. The lower readout shows
    "11 of every 12" for those 0.6 s, unspoken.
  - At +0 each red marker is drawn one marker width right of its blue one.
    The passengers' colors follow the offset, not the drawn marker.
  - The readout ends in "..." rather than "…", which `glyphs::DIGITS`
    lacks.

## Tests

These pass without integration (pure; no script lines needed). Their
arithmetic was also checked in a Python port; the Rust has not been compiled.

- timetable.rs:
  - `every_gap_is_tagged_on_the_bus_that_ends_it`, at offsets 1 to 6 and 2
  - `every_passenger_stands_in_one_gap`
  - `you_stand_in_a_long_gap_clear_of_every_bus`
  - `the_moving_line_is_the_line`
  - `brackets_keep_to_their_gaps`
  - `the_signs_fixed_words_carry_no_numbers`
  - `a_tracker_reaches_whole_items_only`
- two_ways_to_count.rs:
  - `the_pictures_counts_are_its_facts_and_a_wrong_fact_is_refused`
  - `the_rows_are_the_two_counts_to_scale`
  - `the_rows_text_stays_apart_and_on_screen`
  - `the_fixed_words_carry_no_numbers`
- stretch_one_gap.rs:
  - `the_plan_is_the_model_at_every_offset_and_a_wrong_fact_is_refused`
  - `the_live_rule_is_the_models_rule_at_every_whole_offset`
  - `gold_is_between_a_red_marker_and_the_next_blue`
  - `the_red_marker_never_covers_the_blue`
  - `the_points_lie_on_the_straight_line_inside_the_frame`
  - `the_spoken_steps_are_the_readouts`
  - `the_panel_stays_apart_and_on_screen`
  - `the_fixed_words_carry_no_numbers`

These need integration steps 1 to 3: each chapter's
`the_chapter_runs_and_says_its_lines_in_order`. It runs the whole chapter
headless at 10 fps with the scratch voice, and so exercises every phrase
the chapter waits for, every fact it reads and every check the scene makes
before it draws.

## Unresolved

1. **Uncompiled and unrendered.**
   - Checked against the pinned fmn and explainer_kit APIs by reading, not
     by rustc.
   - Each new file passes `rustfmt --check` (edition 2024, the repo's
     `rustfmt.toml`).
   - Text scans found no unused items, imports or struct fields; one unread
     field was removed.
   - The frames are unseen.
2. **Length.**

   | | storyboard.py's rule | scratch-voice emulation |
   |---|---|---|
   | 02 | 81.5 s | 87.6 s |
   | 03 | 83.8 s | 93.1 s |

   No picture overruns its line. The difference comes from:
   - `draw::think`: 4.4 s, where the rule counts 2 s.
   - `draw::stand`: 2.6 s, plus the 0.6 s fade; the rule counts neither.
   - The scratch voice: 2.6 words a second plus 0.4 s a line, against the
     rule's 2.68 words a second.

   Chapter 01 carries the same overheads. Measure the real lengths at
   qualification. Cutting them means changing the frozen words or the
   frozen think and stand.
3. **02_3's "inside one gap"** says a number word with no fact behind it,
   so the guard would fail on 02_3. Options:
   - Add a fact, e.g. each moment lies in exactly one gap. The helper's
     `every_passenger_stands_in_one_gap` already tests it, and the oracle
     would need it too.
   - Reword to "inside a gap". This is a words change; the chapter waits
     for the phrase, so `two_ways_to_count.rs` changes with it.

   Root's decision.
4. **Layout widths are estimates** (0.0045 units a point a character), not
   measured.
5. **03_4's spoken steps** (0.4 s each) assume "eight, nine, ten" fall about
   half a second apart. That is where `Spoken::at` puts them by character
   share, on either voice.
6. **During a step, the new tags fade in at their destination** while the
   red marker slides; they don't travel with it.
7. **Chapter 01 findings.** Its source is frozen, so I changed nothing.
   - `the_stop.rs:331` and `:343` dim each morning's ghost with
     `fade_to` → `set_opacity`. At the pinned fmn, that also writes the
     fill alpha of the stroke-only ring, which would turn it into a disc.
     For a stroked shape, use `set_stroke(None, None, Some(o))`.
   - Both mornings (2½ and 11½ minutes) sit half a minute from a bus, so
     the ring crosses that bus's marker.

   Both are probable from the source; neither is seen in a frame.
