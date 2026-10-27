# inspection_paradox: work status

Bead `mx-video-program-program-87do.13.1`, owner GoldGate (pane %12, session
`jeffs_math_explainer_videos--third-wave-20261024`). Root BlueSalmon owns
Beads, shared files, the catalog and every build, render and synthesis.

**State: `batch_pending`.** The latest handoff is the film's script-first
narration plan (below), in `docs/narration-prep-20261024/` and frozen for
root. Earlier handoffs, also frozen:

- the native 12 (Mail 6939). Root hash-verified and froze it in
  `narration_lab/third-wave-20261024/root/tend-1336/inspection-12-intake/`.
  Its patch is unapplied and depends on the 10/11 patch;
- the native 10/11 (Mail 6926). Root froze its four files, matching their
  hashes and lengths (Mail 6927, receipt in
  `narration_lab/third-wave-20261024/root/tend-1232/inspection-10-11-intake/`).
  Its patch is still unapplied, and 12's patch is built on top of it. Its
  open items stand: 10_3 → 10_4 must never be a coherent seam, and the
  `F-P1.long_calls` fact is only a proposal;
- the native 08/09 (Mail 6909). Root preserved its four files (Mail 6910)
  and has since been integrating them. By 12:46Z the live `main.rs`,
  `mod.rs`, storyboards, overrides, `script.tsv`, `the_formula.rs` and
  `at_random.rs` carried root's changes. Those are root's bytes now, and I
  did not touch them;
- the native 06/07 (Mail 6894), which root has since registered;
- the native 04/05 (Mail 6880), whose five hashes root matched (Mail 6882);
- the ghost-ring repair (Mail 6862), which root is qualifying;
- the native 02/03 (Mail 6858), which root has integrated;
- the compile repair (Mail 6841);
- the WildBass repair.

I do not close or change the Bead. `.13.2` stays open pending the repair
review.

## Script-first narration plan (2026-10-24, 14:30 UTC)

Assignment: ROOT1336-I-NARRATION. It asked for the complete film's usable
narration plan, made with the narration skill and
`docs/video-program/NARRATION-CONSISTENCY.md`. The plan needed:

- coherent whole-thought units with exact cue IDs and verbatim text, and a
  coverage check;
- no unit across a deliberate visual overrun or prediction, in particular
  10_3 → 10_4;
- semantic pause intent authored before any timing, with totals kept apart
  from additions;
- every picture hold mapped to its source event;
- the demanding passages listed;
- a director-schema map, an integration note, risks, and full hashes.

It ran under reservations 8379 (the new directory) and 8380 (this file).

**The projection.** I copied the live 01–09 to an isolated scratch copy and
applied the 10/11 patch, then the 12 patch, each extracted from its frozen
note. The 12 patch on its own is refused on five files; I kept that
dependency control. `tools/storyboard.py`'s own parse, check and
script_text gave 92 cues, sha256
`8a65ec15cfafd499006ac2590013562744eb81d478fd50e1b67ea8968920ad50`. That
equals the projection the 12 handoff gave, and the live script is a byte
prefix of it.

Files in `docs/narration-prep-20261024/`:

| file | sha256 | bytes |
|---|---|---|
| `README.md` | `6c1388ac82271fd3ec0448ce5baa0cc61597169fc12ca8bdb2825516fcba6e97` | 13052 |
| `narration_units.json` | `03f92b4865be0e3cb52b3c7559dade91669642c34ab16d9bdd8ffff6ae25aee4` | 19812 |
| `director-map.json` | `47243f7a436ba39255f2f6774c38e046eb2e446ae9cec20659b251c79041aebf` | 48884 |
| `seams.json` | `2102a1e57db1d17bd696a6659f008a0c73d4d3957f436157256217978bc269ab` | 53622 |
| `projected-script.tsv` | `8a65ec15cfafd499006ac2590013562744eb81d478fd50e1b67ea8968920ad50` | 13703 |
| `check_plan.py` | `47c898e5cd78da679a0d0b84e63ef2346654b473ee6093f6f0525e95e7c9194d` | 12123 |
| `checks.txt` | `ebb03daf6832eba7d32f0f2dd2cc848ec3c566b87d8ec0db06258a6ff44aec14` | 6709 |

**The units.** There are 66 units, of 4–83 words and at most 3 cues each.
26 seams sit inside units, and every one is animation-only with at least
0.5 s of estimated slack. The boundaries are:

- 12 predictions, 13 reading holds and 8 music windows, all read from the
  construct sources;
- 3 deliberate overruns: 01_4|01_5, 05_2|05_3 and 10_3|10_4;
- 4 estimated overruns and 9 tight seams;
- 11 chapter ends;
- 5 eligible seams kept as boundaries by size or thought.

Root's native scratch-voice previews of 01–09 agree. Every source hold
shows as a gap over 1.05 s, and no coherent-eligible seam held or overran.
Only the 08 and 09 previews are bound to today's bytes. Chapters 10–12
have no preview, so their seams rest on source reading alone.

**The map.** It has 73 overrides: the patched file's 36 unchanged, plus 37
new. The new ones cover questions, calculations, new concepts, contrasts
and 16 protected phrases, such as "half of four | squared" and operators
kept with their denominators. None sits at a cue end. It also has 47
visual holds, each bound to its file, its sha256 and the line span of its
call. Every range is the total quiet heard. The map's `accounting` block
gives what to add at each kind of boundary.

**Checks run.** I ran `check_plan.py` in place, from the repo root:

- 242 checks with 0 failures;
- the repo's `coherent_narration.read_units`, and its CLI `check`, which
  printed `{"cues": 92, "units": 66}`;
- the coverage, counted independently;
- an independent source re-scan of every in-unit seam (none holds);
- `narration_director.validate_map` and `directions`;
- every visual hold's evidence lines;
- the anchors' uniqueness.

All 14 negative controls were refused:

- the units: omission, repetition, reorder, changed text, a cross-chapter
  unit, a unit over 100 words, the wrong script, and 10_3 + 10_4. That
  last one is accepted by the schema reader and refused by the seam check;
- the map: an ambiguous phrase, an absent phrase, a duplicate boundary,
  changed evidence, non-adjacent hold cues, and the wrong script.

Against the live 67-cue `script.tsv` the plan is refused, as it must be.

**Controls that failed, and what changed.** All are kept:

- the 12-before-10/11 patch order;
- my scanner's first pass missed `kit.music_breath` windows and loop
  anchors, which I fixed before drawing any unit;
- hold kinds were relabelled where a music window follows `kit.hold`;
- checker run 1 failed at the 10 and 11 chapter ends, which end on
  `kit.hold` and not `stand`. Its output is in `checks.txt`.

**Sources.** Terada's reading is テラダ, トラヒコ, from the NDL authority
record 00083550. The IPA [toɾaçiko] is from Wikipedia's "Torahiko". I found
no pronunciation source for Grofman; it is an open lookup. "route" is
still the owner's choice. `lexicon.json` is unchanged.

**Not done.** I made no edits to registered, shared, frozen or delivered
files: the hashes above and the inputs in `seams.json` still match. I ran
no Cargo, synthesis, render, ASR or `storyboard.py` main, and touched no
Beads, accounts or panes. I created no new voice, reference or detector.
The Michael conditioning is the Accepted B anchor `baseline.json`
(`56318a06…`), unchanged. All of these are pending and root's:

- integration and the regenerated script;
- native compile and tests;
- the bounded synthesis experiment;
- FW/ASR;
- the seam guard in a narrated render;
- the director audit;
- probes;
- owner listening.

A plan and passing checks do not prove natural pacing, rendered clarity
or English-accent acceptance.

## Native chapter 12 (2026-10-24, 13:50 UTC)

Assignment: ROOT1232-I12 (Mail 6927). It asked for the film's last chapter,
`12_one_more_timetable`, keeping the film's stop, passengers, markers, wait
bars and curve. The chapter had to:

- return from the planners to arrivals at random;
- move the red bus to four minutes, with the triangles and the answer
  withheld until the countdown ends;
- show the two areas, add them, divide by the time and return to the U;
- separate the average wait from the train's margin, keeping the on-time
  assumption, and call ten minutes the worst case in this model, not a
  guarantee about real delays or the whole journey.

It also asked for the draft's 12_4 to be repaired: forty square minutes,
not forty minutes of waiting, divided by twelve minutes.

**What I wrote.** One new file the build does not compile yet, and
`docs/integration-12.md`:

| file | sha256 | bytes |
|---|---|---|
| `src/chapters/one_more_timetable.rs` | `e067a6d02a7f4206b4e370c821a7e94ef7ceb8607ba10f7982ca398a153ad4c2` | 76303 |
| `docs/integration-12.md` | `ae6a1c46b758f47a4792afce66d315b2f14295a76edfb15c10a8d221d8877e2b` | 64391 |

- Nine cues where the draft had six. 12_1 sends the planners back to the
  moments they were ready. 12_2 is the prediction. 12_3 and 12_4 give the
  areas in square minutes and divide them by the twelve minutes, units
  included. 12_5 is 07's U. 12_6 to 12_8 are the cost, the margin and the
  margin's condition (K10). 12_9 is the closing line, then the names and
  then the receipts, each held for its own reading time.
- Two new facts, `F-Q1.short` (4) and `F-Q1.long` (8), bind the spoken
  gaps. They go in `facts.rs` and the oracle through the patch, both
  root's files.

**No edits to shared, registered or delivered files.** The patch is
against the pending 10/11 integration, so it applies only after the 10/11
patch. On the live tree `git apply --check` fails, as it should. The note
gives root:

1. the order: the 10/11 integration first, then this patch (363 lines,
   sha256 `7d84592dbe539f5fc73c960c6810ee99a31d8c8c2c437fd87f734f482f287cb9`,
   seven files), then the `storyboard.py` step, with every before and
   after hash in full;
2. 19 SPOKEN rows, 4 pause overrides, the two facts and an oracle
   self-test check;
3. the authored prediction, inspection, visual and reading holds;
4. the glyph and pronunciation risks, the controls that failed during the
   work, and every gate not run.

**Checks (Python and rustfmt, not native execution):**

- rustfmt is clean on the file, and on a crate copy with both patches
  applied (every module), with a broken-module negative control.
- The patch applies after 10/11, and the seven results equal my patched
  files.
- All imports and bindings are used. Every call is to something a
  compiled chapter, the pending 11 or a compiled corpus crate also calls.
  These are names, not signatures; the signatures `last` uses were read.
- A Python port of the tests passes 114 checks, with negative controls.
- The integration tests, emulated on the patched copies, pass with six
  negative controls.
- The patched oracle's self-test passes, and its new check fails on a
  wrong copy.
- The overrides pass `narration_director.directions`, with three
  controls.
- The scratch-voice emulation gives 139.6 s with the holds; storyboard.py
  gives ~106 s. Nothing was compressed.

**Not verified:** compilation, Clippy, the tests, `facts-check`, the
headless run, glyph probes, rendered layout, pacing, pronunciation and
listening.

## Native chapters 10 and 11 (2026-10-24)

Assignment: Mail 6910. It asked for real, unregistered native chapters 10
and 11 from the current draft, with an integration patch and note, full
authored prose and semantic pause intentions. Its constraints:

- the same people and timetable as the bridge;
- the 2/10 classroom built from twelve individually sampled students
  before its equation, with the answer protected until after the
  prediction;
- for profiling, time share kept apart from call share, and the ideal
  sampling and aliasing assumptions qualified;
- in 11, the original stop and the computed `planned_arrivals`, keeping
  the counterexample: five sixths in long gaps while every wait is one
  minute;
- conditions visible and transitions open to inspection;
- no compression;
- readable attribution cards;
- wrong-model controls, exact SPOKEN bindings and pinned APIs.

**What I wrote.** Two new files the build does not compile yet, plus
`docs/integration-10-11.md`:

| file | sha256 (first 16) | bytes |
|---|---|---|
| `src/chapters/same_sum.rs` | `6a6cf394f33a5205` | 66106 |
| `src/chapters/back_to_the_stop.rs` | `3c98fc169d961cbd` | 68150 |

- **10** has nine cues. The draft's question became its own cue, 10_4,
  because the twelve answers outlast "Ask every student in turn …". A new
  10_8 shows the in-step clock, and K08 now says "on average and over a
  long run".
- **11** keeps seven cues, with three wording changes the note lists:
  11_3's square minutes, and 11_5's and 11_7's "the triangles' average".

**No edits to shared, registered or delivered files.** The base is the
live production after root's 08/09 integration. The note lists the hashes,
and those files still match. I ran no Cargo, TTS, render or `storyboard.py`
main. The integration note gives root:

1. one exact patch (413 lines, sha256 `c42705490f51d03c`). `git apply
   --check` passes on the live tree. It covers:
   - the `mod.rs` registration;
   - the two authored sections, appended to `storyboard.md` and replacing
     the draft's 10/11 sections;
   - 28 `SPOKEN` rows;
   - 5 pause overrides;
   - `main.rs`'s stale "01 through 05" doc line;
2. the `storyboard.py` step, with the expected `script.tsv` hashes (an
   emulation that reproduces the live file exactly);
3. the glyph note: ⅔ comes by fallback from IBM Plex Sans (a cmap
   reading);
4. every cue's waits, the authored picture and reading holds (10_3 → 10_4
   must never be a coherent seam), and the departures from the draft.

**Checks (Python and rustfmt, not native execution):**

- rustfmt is clean on both files and on a patched copy of the whole crate,
  with a broken-module negative control.
- No unused imports or bindings.
- Every method or helper the files call is also called by a
  root-compiled chapter (names, not signatures).
- All 48 waits are in order, all 41 fact keys are in the oracle, and K08
  and K09 are the sections' exact versions.
- The SPOKEN rows pass the test's logic, with four negative controls.
- The overrides pass `narration_director.directions`, with two negative
  controls.
- The crate's integration tests, emulated on the patched copies, pass with
  three negative controls.
- A Python port of the tests' arithmetic, text and geometry passes 13,243
  checks, with six negative controls.
- The scratch-voice emulation gives 116.5 s for 10 and 96.7 s for 11
  (storyboard.py: ~98 s and ~81 s, against the draft's ~72 s and ~76 s).
  Nothing was compressed.

**Not verified:** compilation, Clippy, the headless runs, glyph probes,
rendered layout, pacing, pronunciation and listening.

## Native chapters 08 and 09 (2026-10-24)

Assignment: root's message after freezing the Mail 6894 files. It asked for
real, unregistered native chapters 08 and 09 from the reviewed draft and the
same 2/10-minute timetable, with these constraints:

- numbers become letters through the same triangles;
- explicit units;
- the spread defined by geometry before its name;
- the square argument given time, and not replaced by the finite check;
- the prediction protected;
- the seeded picture kept distinct from the Poisson model;
- memorylessness taught before the formula confirms it;
- no early six or twelve;
- the 3 / 4:20 / 6 scale with a longer-than-six example;
- every condition kept.

**What I wrote.** Two new files the build does not compile yet, plus
`docs/integration-08-09.md`:

| file | sha256 (first 16) | bytes |
|---|---|---|
| `src/chapters/the_formula.rs` | `658ecf773468bdf2` | 85788 |
| `src/chapters/at_random.rs` | `00d407d0074f4212` | 58941 |

**No edits to shared, registered or delivered files.** The note lists
their hashes, and they still match. 04–07 are untouched. I ran no Cargo,
TTS, render or `storyboard.py`. The integration note gives root:

1. one exact patch (233 lines, sha256 `7797357c9ee02cb0`). `git apply
   --check` passes on the live tree. It covers:
   - the `mod.rs` registration;
   - both draft sections for `storyboard.md`, byte for byte, with their
     hashes;
   - 38 `SPOKEN` rows;
   - 3 pause overrides;
2. the `storyboard.py` step, with the expected `script.tsv` hash (an
   emulation);
3. the glyph note: § is in Computer Modern, and ≥ comes by fallback from
   Noto Sans Math (a cmap reading);
4. every cue's waits, the authored silent holds, the departures from the
   draft, and one optional 08_1 prose correction ("two square minutes").

**Checks (Python and rustfmt, not native execution):**

- rustfmt is clean on both files and on a patched copy of the whole crate.
- No unused imports and no dead items.
- All 55 waits are in order, all 37 fact keys are in the oracle, and the
  cards are the draft's.
- The SPOKEN rows pass the test's logic with four negative controls.
- The overrides pass `narration_director.directions` with two negative
  controls.
- A Python port of the tests' arithmetic and geometry passes 10,677 checks,
  with three negative controls. The port found and fixed one would-be test
  failure in 09's layout.
- The scratch-voice emulation gives 148.1 s for 08 and 111.0 s for 09,
  against the draft's ~85 s and ~77 s. Nothing was compressed.

**Not verified:** compilation, Clippy, the headless runs, glyph probes,
rendered layout, pacing and listening.

## Native chapters 06 and 07 (2026-10-24, 10:00 UTC)

Assignment: Mail 6882. It asked for only the new, unregistered chapters 06
and 07 from the accepted draft, with these constraints:

- the same timetable and passenger identities;
- bus and passenger weighting made visibly different before formula work;
- the no-late-bus counterexample, the predictions and the inspection time
  kept;
- halos for text emphasis, no overlapping number crossfades, and no
  fill-dimmed brackets;
- no compressing to an 85-second estimate.

**What I wrote.** Two new files the build does not compile yet, plus
`docs/integration-06-07.md`:

| file | sha256 (first 16) | bytes |
|---|---|---|
| `src/chapters/two_averages.rs` | `78325de488aa368a` | 52025 |
| `src/chapters/no_bus_late.rs` | `83d8c3f7140d7f25` | 53935 |

There is no new shared helper, and neither file imports the 04/05 files.

**No edits to shared or delivered files.** Every shared and delivered file
still has its starting hash; the note lists them. I ran no Cargo, TTS or
render. The integration note gives root, in order:

1. the `mod.rs` lines;
2. the two draft sections to copy into `storyboard.md` byte for byte, with
   their hashes;
3. the `storyboard.py` commands;
4. 36 `SPOKEN` rows, rustfmt-formatted;
5. 4 pause overrides;
6. the glyph note (⅔, from IBM Plex Sans by fallback, as ⅙ and ⅚ are).

**How it meets the brief.**

- **The same timetable and passengers.** Both chapters rebuild chapter
  03's hour: the same markers, tags, brackets and sixty passengers. 06_5
  rides through your own long gap. 07's dots repaint live as the red bus
  moves.
- **The weighting is visible first.**
  - 06_1 builds one bar per bus, born at its tag, into a row that levels at
    6.
  - 06_2 builds one bar per passenger, born at its answer, into a second
    row of sixty bars.
  - The passenger level (8⅔) grows before the ⅙/⅚ sum.
  - The draft's own bus sum stays in 06_1 after the bus row. It is flagged
    for root, and can move to 06_4.
- **The counterexample and the experiment.**
  - 07_1 shows every bus on time and the printed timetable rows. 07_2 shows
    the bunching inset, K04, and our gaps as simply uneven.
  - 07_3 to 07_7 vary one tracker, the red bus's offset, with the sign's
    buses and average gap held fixed and haloed. The triangles, dots,
    point, readout and U all follow it. The sweep runs on through the 5 s
    silent hold.
- **Prediction and inspection time.** There are `think` countdowns in 06_3
  and 07_3, read holds for K03, the names and K04, the ride, the sweep, and
  a `stand` at each end.

**Checks run.** None of these is a compile, a render or a listen; the note
lists each check and its result.

- rustfmt is clean, and the scans find no dead items. `Timed` is a known
  false positive.
- Phrases and fact keys hold. K03 and K04 equal the draft's.
- The 36 SPOKEN rows and the 4 overrides pass, the overrides through the
  director's own `directions`.
- A Python port of the tests' arithmetic and geometry passes 22,978 checks,
  with negative controls caught.
- The oracle self-test reports 0 failures.

The port caught one real defect, now fixed in both files: the layout tests
modelled the lower-third band on the strict screen bounds, so as written
they would have failed.

**Timing (scratch-voice emulation, not a measurement).**

| | emulation | storyboard.py's rule |
|---|---|---|
| 06 | 83.7 s | 70.8 s |
| 07 | 94.5 s | 82.1 s |

Nothing was cut to fit.

**Reservations:** 8026–8029 (the two chapter files, the integration note,
this file), released with the handoff.

## Native chapters 04 and 05 (2026-10-24, 09:10 UTC)

Assignment: root's message after Mail 6862. It asked for only the new,
unregistered chapters 04 and 05 from the reviewed storyboard, a new helper
if essential, and this file. Everything already delivered stays frozen.

**What I wrote.** Three new files the build does not compile yet, plus
`docs/integration-04-05.md`:

| file | sha256 (first 16) | bytes |
|---|---|---|
| `src/chapters/twelve_minutes.rs` | `61206a6f6ae1ff08` | 29823 |
| `src/chapters/one_a_minute.rs` | `747ac26e1ed363d3` | 33437 |
| `src/chapters/triangles.rs` | `44df1f81e273cf67` | 44523 |

`twelve_minutes.rs` is the shared helper. It holds the stretch, its
passengers, every number both chapters print (read from the facts and
checked against the stretch) and the stretch picture.

**No edits to shared files.** I changed no shared helper, registry,
`main.rs`, model, facts, storyboard or script. I ran no Cargo, TTS or
render. The integration note gives root, in order:

1. the `mod.rs` lines;
2. the two draft sections to copy into `storyboard.md` byte for byte, with
   their hashes;
3. the `storyboard.py` commands;
4. 43 `SPOKEN` rows, rustfmt-formatted;
5. 5 pause overrides;
6. the glyph note.

**How it meets the brief.**

- **Same timetable and passengers.** 04_1 zooms chapter 03's hour into its
  first twelve minutes. It transforms the same line, markers, brackets and
  dots.
- **Waiting against the whole gap.** Each passenger's wait is a bar beside
  its gap's bracket (04_3, 04_4). The 05_2 ride shows the wait is the whole
  gap only just after a bus, then shrinks to nothing. The triangles are as
  wide as the gap and as tall.
- **Actual positions and computed counts.** Every number is read from the
  facts and checked in `twelve_minutes::numbers`. The tile counts are
  checked against F-A1.
- **Definitions first.** Averages come after their waits, the sums before
  the division, the bars-equal-triangles step before any area, "× 5" before
  "× 25".
- **Observation and prediction time.** `think` countdowns in 04_5 and 05_6,
  a K02 read hold, a silent long ride, a held and breathing tiling, and a
  `stand` at each end.

**Checks run.** None of these is a compile, a render or a listen; the note
lists each check and its result.

| check | result |
|---|---|
| `rustfmt --check` | clean |
| static import and dead-item scans | only `Timed`, a trait used via `.rt(` |
| phrases (8 + 8 lines, 24 + 23 phrases) | in order and unique |
| fact keys | all 40 in the oracle |
| K02 | the draft's exact version |
| SPOKEN rows | 43 rows and the guard hold; negative controls caught |
| overrides through `narration_director.directions` | 5 of 5 land; negative controls refused |
| Python port of the tests' arithmetic and geometry | 1565 checks; every negative control caught |
| `oracle.py --self-test` | 0 failures |

**Timing (scratch-voice emulation).**

| chapter | emulation | storyboard.py's rule |
|---|---|---|
| 04 | 96.8 s | 84.8 s |
| 05 | 95.3 s | 79.2 s |

Before handing off I fixed three late pictures in 05: 05_1, the 05_2 jump
and the 05_7 tiling. Two items are left for root's measured pass:

- the 05_7 ratio flash lands 0.9 s late on the scratch voice;
- the 05_2 long ride runs 2.9 s past its line, kept as watching time.

**Not done, and not claimed.** No compilation, test run, render, narration
or listening. The text boxes are estimated, and "…" and "→" are unprobed
here. Ten picture departures from the draft are listed, with reasons, in
the integration note.

## Ghost-ring repair, chapter 01 (2026-10-24, 07:55 UTC)

Assignment: root's message after Mail 6858. I authored `the_stop.rs` and
this file. No `draw.rs` change was needed. The old source is preserved as
`previous/ghost-ring-20261024/the_stop.rs`, an identical copy.

**The defect, in root's retained render**
`narration_lab/third-wave-20261024/root/native-qualification/inspection-opening03/output/01_the_stop.mp4`
(scratch voice, 15 fps, 97.4 s). I looked at frames at 36.5, 37.6, 39.0,
41.5, 43.0, 45.0, 47.0, 52.0, 60.0, 70.0, 76.4, 77.0, 85.0 and 90.0 s:

- At 47.0 s the lucky ghost at :11½, already dimmed, is a filled
  translucent gold disc. The unlucky ghost at :02½, just faded in at full
  opacity, is a white dot in a gold ring.
- From 52 s until the title, both dimmed ghosts are discs. They tint the
  bottoms of the red :02 and blue :12 markers. The :12 tick shows only
  faintly through the lucky one.
- The "you" in the strip at the top stays a ring. The 01_7 flash leaves
  opacity alone.

Cause: `fade_to` is `set_opacity`. At fmn pin `a399d626` that writes the
alpha of every colour column in the family. The ring is `line_style`, an
outline whose fill has alpha 0, and that fill went to 0.45 along with the
stroke.

**The fix.** `dim_ghost` dims each painted part of the ghost (each leaf of
its family) by what it paints with. All parts dim together, over the old
run times of 0.3 s and 0.4 s:

- A part with a fill (the dot) is dimmed by `fade_to` on that part alone.
  That is what the dot got before.
- A part with no fill (the ring) is dimmed by
  `set_stroke(None, None, Some(GHOST_DIM))`, the form Parrondo03 uses.

`dim_ghost` reads each part's alpha columns, so it does not depend on the
order of parts inside `you()`. Both old `fade_to` calls (old lines 331 and
343) now call it. Words, facts, run times and on-screen text are unchanged.
The only new string is an error message.

**The regression test**, `a_dimmed_ghost_keeps_its_ring_an_outline`, runs a
headless scene with two ghosts of you:

- The whole ghost has two parts: a dot with fill 1, and a ring with fill 0
  and stroke 1.
- After `dim_ghost`, the dot's fill is 0.45, the ring's stroke is 0.45, and
  the ring's fill is still 0.
- The control applies the old dimming to the second ghost. It must give
  the ring a fill of 0.45, the disc seen in the render. If the control ever
  fails, the pinned behaviour has changed and `fade_to` would work again.

**Ring and marker: not separated.** At full opacity (47.0 s) the ring
crosses the lower corner of the red :02 marker, but the two still read as
different things: a tall coloured bar above the line, and a white dot in a
ring on it. What made them ambiguous in the dimmed frames was the disc's
fill washing over the markers, and this fix removes it. Moving a ghost
would misplace the moment it stands for: 2½ and 11½ minutes, the waits of
9:30 and 0:30 in F-M1. Root's new preview decides. If the dimmed ring still
reads as part of a marker, I will propose a change then.

| File | Before | After |
|---|---|---|
| `src/chapters/the_stop.rs` | `badd29d03d9b2ddf`, 21120 B | `a2f33067c980449d`, 26102 B |
| `previous/ghost-ring-20261024/the_stop.rs` | (new copy) | `badd29d03d9b2ddf`, 21120 B |
| `docs/WORK_STATUS.md` | `b7b87331846fe04d` | this section |

Unchanged (sha256, first 16 hex):

| File | Hash |
|---|---|
| `draw.rs` | `e51741b39e65e31a` |
| `main.rs` | `c367132d2a2b105a` |
| `mod.rs` | `2ed8357840f234df` |
| `timetable.rs` (02/03 handoff) | `248b9b273dc3556a` |
| `two_ways_to_count.rs` (02/03 handoff) | `cd0e6c515b46ea94` |
| `stretch_one_gap.rs` (02/03 handoff) | `2c3c55df1f584823` |
| `integration-02-03.md` (02/03 handoff) | `e96c5bca468e6bfe` |

### Results actually run (07:40–07:55 UTC, from the repo root)

- `rustfmt --check --edition 2024 --config-path rustfmt.toml
  explainer_corpus/inspection_paradox/src/main.rs` exits 0. The first run
  showed one diff, a line break in the new `alpha`, so I ran rustfmt on
  `the_stop.rs` alone.
- The dead-item text scan finds nothing, and the import scan of
  `the_stop.rs` finds no unused import. `alpha`, `leaves`, `dim_ghost` and
  `fade_to` all have non-test callers.
- Against the preserved copy the diff is +124 −2. The two removed lines are
  the two old `fade_to` calls.
- Checked by hand at the pin:
  - `Stage::prepare(impl IntoAnimation)` returns `Box<dyn Animation>`.
  - `AnimBuilder::set_stroke` takes `Option<Srgb>`, `Option<f64>` and
    `Option<f64>`.
  - `family` returns the mob first, then its members.
  - `get` gives the `Entry` whose `buffer.read_column(&str)` returns
    record-major RGBA.

### Limits of the ghost-ring repair

- Not compiled, tested or rendered. The new test, like the chapter test,
  waits for root's build. The alphas it expects are what the pinned code
  should produce; only `cargo test` shows that it does.
- Root's new preview must confirm that the dimmed ring stays an outline and
  that the ghosts and markers read apart.
- For the 02/03 integration: rewording 02_3 to "inside a gap" also means
  changing `two_ways_to_count.rs:334`, `line.wait_for(stage, "inside one gap")`,
  to wait for the new phrase. Otherwise the chapter refuses to render. That
  file is frozen for root, and I have not changed it.

## Repair of WildBass's review (2026-10-24, 06:40 UTC)

Assignment: root Mail 6819 and
`narration_lab/third-wave-20261024/root/tend-0610/inspection-repair.txt`.
The review is
`docs/video-program/third-wave-20261024/reviews/inspection-paradox-WildBass.md`.
The source before the repair is root's intake snapshot,
`narration_lab/third-wave-20261024/root/intake-0550/inspection_paradox/`. It
is untouched: all 18 listed hashes still match, and its extra
`.ee/hook-state/session_start_orient.last.json` (181 bytes) is preserved.
I worked from the repo root.

| Finding | Resolution | Where |
|---|---|---|
| S1 | The chapter ends with `kit.finish(stage)`. The headless test now asserts that the chapter left a timeline and that the timeline runs past the end of the last line | `the_stop.rs` |
| M1 | Planners keep their own ready moments. Each stands one minute before the first bus at or after their dot plus one minute, so nobody turns up early and every dot moves only forward. That gives 10 before each blue bus and 2 before each red, still 5/6 in long gaps, every wait 1 minute. The film says they aren't spread through a gap any more (on average 1/6 of the gap ahead, against 1/2), never that they leave long gaps. Unlike the review's sketch, the two dots per twelve minutes ready only half a minute before a bus take the next one; the counts are the same. A facts guard refuses if the planners' long-gap share stops equalling the time share. The oracle builds the planners on its own, by bisection on its bus list | `model::planned_arrivals` and its test; F-K2 (`per_bus` and `before_red_share` removed; `per_blue`, `per_red`, `never_early`, `mean_wait_clock`, `stand_at`, `wait_share_of_gap` added); `oracle.planners`; draft 11_6, 11_7, notes, arc, spine; brief |
| M2 | K07 says "and spread evenly in time", with the :02:30 counterexample in the notes | draft, notes for 09 |
| M3 | 08_8's card reads "every choice of up to 4 gaps (repeats allowed, order ignored) … 2,375 choices". The oracle now checks 2,375 against 1,091 sets and 30,936 ordered lists | draft; `model::Sweep` doc; oracle; brief |
| M4 / title | Root's title, "Why Your Bus Wait Is Longer Than You Expect", is used everywhere this lane owns. The title card still splits at " Than" | `glyphs.rs`, both storyboards (heading, 01_7), brief, `main.rs` docs, `Cargo.toml` description |
| T1 | A new 12_5 pays off the train: 4:20 is the average cost, but allow the full ten minutes, the wait just after a red bus leaves. The closing line is now 12_6 | draft 12; `model::longest_wait` and its test; F-W1.allowance(_clock) with a guard; oracle (worst second of the hour, rounded up); notes for 01 and 12 |
| T2 | 04_8 starts from the viewer's guess: "Half of six is the average of one and five, as if both gaps had the same number of passengers." The picture builds "half of 6 = 3 = (1 + 5) ÷ 2" | draft 04 |
| S2 | Labels read "wait 0:30 as you arrive" and "wait 9:30 as you arrive", under the middle of each bar, with a dim "one morning, sped up" or "another morning, sped up" beside the bus. The first morning's bar and words leave, and its ghost dims, before the second morning starts. The storyboard says "shrinks steadily" and names the two different days. `draw::together` lost its only caller and is removed (it would have been dead code under clippy) | `the_stop.rs`, `draw.rs`, 01_4 and notes in both storyboards (verbatim) |
| S3 | `model::offset_wait_rational(d: Q)` covers any offset from 0 to 12 and refuses others. `model::clock_rounded` rounds halves up to whole seconds and then calls the exact `clock`, which is unchanged and still refuses anything else. The test covers whole-minute agreement, 2.5 → "4:01", halves up, the carry, a half-second bound at all 121 tenths, and the 13 whole-second tenths. The oracle computes the same in integer half-seconds | `model.rs`; F-O1 (5 keys added); oracle `slider_wait`, `readout`; draft 07_5 notes |
| S4 | The oracle's section comments carry the twelve-chapter numbers | `oracle.py` |
| Feller | Removed. The 09_7 card reads "names to search: waiting-time paradox, bus paradox", from Masuda and Hiraoka's abstract, which I re-read through the arXiv export API. The brief keeps Feller under "Located, not read", without a section number | draft 09_7 and notes; brief |

### Results actually run (06:20–06:40 UTC, from the repo root)

```text
python3 -I -B explainer_corpus/inspection_paradox/oracle.py --self-test
  exit 0; 76 "ok" lines; "0 failure(s)"   (was 65)
python3 -I -B explainer_corpus/inspection_paradox/oracle.py
  exit 0; 166 facts   (was 155)
python3 -I tools/storyboard.py --check inspection_paradox
  exit 0; 01_the_stop 83.7 s, 7 beats; script.tsv current (unchanged, sha256 e598c79c…)
python3 -I tools/lint_script.py explainer_corpus/inspection_paradox/script.tsv --chapters 01_the_stop
  exit 0; "lint: 7 lines, 0 errors, 0 warnings -> PASS"
full draft through tools/storyboard.py's parse/check/script_text (script to the scratchpad)
  exit 0; 12 chapters, 15.7 min, each 63.7-84.8 s (a first wording of 04_8 gave 85.2 s and was cut)
lint_script.py on that full-draft script, all 12 chapter keys
  exit 0; "lint: 87 lines, 0 errors, 0 warnings -> PASS"
diff against previous/draft02-registry12/script.tsv
  only 04_8, 11_6, 11_7 and the new 12_5 change; the old 12_5 is 12_6
rustfmt --check --config-path rustfmt.toml explainer_corpus/inspection_paradox/src/main.rs
  exit 0 (three diffs fixed by hand first)
```

Python over the files (scratchpad scripts, not Rust test results):
- facts.rs adds 166 literal keys, exactly the oracle's;
- a line-by-line emulation of the new Rust (planners, allowance, rational offset, rounded readout and the two guards) gives the oracle's value for all 17 new keys;
- the 4 storyboard sections are verbatim in the draft, K01 matches, the scene says the 7 lines in order, all 11 waited-for phrases and 7 fact keys exist, the fixed words carry no digits and are all in their test, and all 10 SPOKEN rows hold;
- the static scan (`parrondo-GoldGate/source_scan.py`) flags only false positives: `ON_ARRIVAL`, used as an inline format argument, and the operator trait methods.

Not run, for root's native batch: the crate's build, `cargo test` (34 Rust
tests, was 32), clippy `-D warnings`, `facts-check`, the glyph probe (new
title and the 01_4 words), the layout lint, renders, narration and
listening. The 01_4 note at x = −1.4 on the bars' row and the bar-centred
labels are reasoned, not seen. `kit.finish` after the title fade is the
corpus pattern, but its hold and timeline are proven only by the test
once it runs.

## Acknowledgment (2026-10-24, 04:58 UTC)

- I read AGENTS.md (including the 2026-10-24 amendment), the video-program
  README, plan sections 0, 1, 3 and 4 (and 5, the pedagogy doctrine), the four
  named skills (end to end, franken_manim, narration, de-slopify), MOR_CASE,
  MOR_CLAIMS, SELECTION, the channel-stats record and the catalog entries for
  `inspection_paradox` and `bus_bunching`.
- I reserved `explainer_corpus/inspection_paradox/**` through Agent Mail as
  GoldGate (reservation 7528), with the audit file as the one exception root
  granted (7529).
- Audit of C001–C005 is written and corrected to the validator schema:
  `docs/video-program/third-wave-20261024/.mor/run-20261024T043200Z/audit/AUD01-C001-C005.jsonl`
  (Mail 6765 and 6773 to root).
- This crate is in the root manifest's `exclude` list (root's decision, Mail
  6769). I don't edit shared files and I run no Cargo command. The Rust
  source here stays **uncompiled** until root's integration pass.
- Later direction applied: Mail 6776 (more setup and visual intuition;
  acknowledged 05:09 UTC, plan in Mail 6783) and Mail 6790 (production
  registry, active script and storyboard hold only implemented native
  chapters; the full draft kept separately).

## Handoff (2026-10-24, 05:45 UTC; figures as of then, see the repair above)

Paths are relative to `explainer_corpus/inspection_paradox/`.

| Path | What |
|---|---|
| `docs/storyboard-draft.md` | **The full draft**: twelve chapters, 86 lines, 2,221 spoken words, every beat's words, picture, features and hold, prediction pauses, callbacks, precisely cards K01–K09, pause intentions and pronunciation notes per chapter, the arc and the spine table |
| `docs/storyboard.md` | Production storyboard: chapter 01 only, with the vocabulary, color roles and scope. Every section after its preamble is a verbatim copy from the draft |
| `script.tsv` | Active narration, 7 lines (01_1–01_7), generated by `tools/storyboard.py` |
| `src/chapters/mod.rs` | Registry with exactly one chapter, `01_the_stop`. There are no stand-in or refusal scenes (the `Unwritten` entries are removed; the old file is in `previous/draft02-registry12/mod.rs`) |
| `src/chapters/the_stop.rs` | Chapter 01 scene, plus 3 tests: a headless run at 10 fps with the scratch voice that says all seven lines in order; the K01 card matching the storyboard word for word; no digits in fixed words |
| `src/chapters/draw.rs` | Hour line, bus markers, you, groups, playback, line timing, think countdown (3 tests). `steady` is the linear tracker move, documented as not `tween`; the kit's `tween` and `live_bar` are used directly |
| `src/chapters/glyphs.rs` | Digits and the whole title for the glyph probe |
| `src/model.rs` | Exact rationals, the timetable, every average, `lucky_morning`/`unlucky_morning` shared by the facts and the scene (new test: both mornings catch the :12 bus) |
| `src/facts.rs` | 155 facts computed from the model, 6 pins, 5 consistency guards (2 tests) |
| `src/main.rs` | `Explainer`; tests: SPOKEN (10 rows covering every number word of the active script), the number-word guard with a narrow pronoun exemption and its negative control, no "always late", storyboard/registry/script agreement, the full draft holding the storyboard verbatim |
| `src/palette.rs`, `src/score.rs` | Color roles drawn by chapter 01 (`short_gap` and `spread` removed until their chapters; colors kept in the storyboard table; 1 test); D-major identity (2 tests). Totals: 32 Rust tests across the crate, none run |
| `oracle.py` | Independent exact arithmetic, second by second, never the closed form; `--self-test` |
| `docs/brief.md` | Claims, sources read and not read, competition check (search listings only, nothing watched) |
| `lexicon.json` | Pronunciation risks only. Nothing respelled, probed or listened to |
| `docs/pause-overrides.json` | Chapter 01's four authored pauses in `tools/narration_director.py` override format |
| `previous/draft01/`, `previous/draft02-registry12/` | The first draft, and the twelve-chapter storyboard, script and registry before the Mail 6790 split, unchanged |

Not written: `docs/fact-sheet.md`. Its content already exists in three places:
- the brief (claims, terms, sources);
- the draft's notes (misconceptions, the K cards);
- `facts.rs` with `oracle.py` (every number).

A separate ledger would only duplicate them.

### Results actually run (05:43 UTC, from the repo root)

```text
python3 -I -B explainer_corpus/inspection_paradox/oracle.py --self-test
  exit 0; 65 "ok" lines; "0 failure(s)"
python3 -I -B explainer_corpus/inspection_paradox/oracle.py
  exit 0; 155 facts
python3 -I tools/storyboard.py --check inspection_paradox
  exit 0; 01_the_stop 83.7 s, 7 beats; script.tsv current
python3 -I tools/lint_script.py explainer_corpus/inspection_paradox/script.tsv --chapters 01_the_stop
  exit 0; "lint: 7 lines, 0 errors, 0 warnings -> PASS"
full draft, through tools/storyboard.py's own parse/check/script_text
  (script written to the session scratchpad, not the repo):
  exit 0; 12 chapters, 15.3 min, each 50.8-84.8 s, no problems
  its script is byte-identical (cmp exit 0) to previous/draft02-registry12/script.tsv
lint_script.py on that full-draft script with all 12 chapter keys
  exit 0; "lint: 86 lines, 0 errors, 0 warnings -> PASS"
cd explainer_corpus/inspection_paradox && rustfmt --check src/main.rs   (repo rustfmt.toml)
  exit 0 (rustfmt follows every module from main.rs)
```

The full-draft check, for reuse:

```bash
python3 -I -c "
import pathlib, sys
sys.path.insert(0, 'tools')
import storyboard as s
ch = s.parse(pathlib.Path('explainer_corpus/inspection_paradox/docs/storyboard-draft.md'))
problems, report = s.check('inspection_paradox', ch)
print('\n'.join(report)); [print('PROBLEM', p) for p in problems]
pathlib.Path(sys.argv[1]).write_text(s.script_text('inspection_paradox', ch))
sys.exit(1 if problems else 0)
" OUT.tsv
```

Source-level emulations of the data the Rust tests read. These are Python
over the files, **not** Rust test results:
- facts.rs adds 155 literal keys, all present in the oracle output, none
  generated by `format!`;
- all 10 SPOKEN rows: each phrase is in its line and each value equals the
  oracle's;
- the number-word guard flags no line;
- both pronoun phrases occur;
- the 4 storyboard sections appear verbatim in the draft;
- K01 matches the storyboard;
- every phrase `the_stop.rs` waits for is in its line, and every fact key it
  reads exists;
- each of the 4 pause overrides names a phrase occurring exactly once, under
  the director's tokenizer.

### For root's native batch (none of this has run)

- The crate's build, `cargo test`, `facts-check` (value equality, Rust
  against the oracle), the glyph probe, fmt and clippy `-D warnings`.
- I checked by hand that every `pub` item has a non-test user, and every
  fmn/kit call against the pinned source: `AnimBuilder` args are taken once
  (hence `steady`), `Polygon::build` returns a `VMobject`, and
  `run_scene(program, config, seed, sink)`. A compiler may still disagree.
- Layout is reasoned, not seen:
  - the right panel and the sign card sit above K01's lower-third, estimated
    at 3 lines, top near y = −2.42;
  - the ghost's ring touches the marker of the bus half a minute away, by
    design.

  The layout lint and actual frames decide.

## Limitations

- No Cargo build, test, clippy or fmt (beyond rustfmt) has run on any Rust
  file here.
- No narration, synthesis, render, contact sheet or listening has happened.
  Pause ranges are editorial hypotheses, not timing evidence. Nothing here
  shows rendered clarity, natural pacing or English-accent acceptance.
- Chapters 02–12 exist as words and pictures only. They are not registered,
  and root assigns them.
- Competition check: web search listings only. I watched no videos.
- Feller is no longer cited (repair above). A table of contents on a
  secondary mirror is all I saw of it.
- The de-slopify read covered the seven active lines. The 86-line draft has
  the mechanical lint only.
