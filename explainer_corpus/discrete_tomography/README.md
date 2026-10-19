# Two Shadows Can Lie

A picture made of filled squares casts two shadows: how many squares each row
holds and how many each column holds. This video asks what those counts can
tell you. A little face's shadows fit 64 different pictures; when the four
corners of a rectangle alternate filled, empty, filled, empty, swapping filled
and empty there (a switch) turns one into another without changing a single
count; the same row counts with other column counts of the same total fit no
picture at all. Later chapters build a picture by letting water flow through
the grid (Gale, 1957), show where the water stops when none fits, and prove
that a picture is the only one exactly when it has no switch (Ryser, 1957).

**Status (2026-10-19, revision `v1-michael-20261019`):** chapter 01
(`01_same_shadows`, `.9.1`) is source and silent visuals verified by root at
6bae9ea (strict clippy, tests, oracle facts, glyph probe, a 540p30 silent
draft and its frames). Chapter 02 (`02_water_through_the_grid`, `.9.2`: the
scene, the recorded flow trace and its replay checker, the facts, the oracle
and the tests) is source and silent visuals verified by root at fa6fd0f:
fmt, strict clippy, 35 tests, a release build and 73/73 facts against the
oracle; a silent 960x540 draft of 81.73 s (scratch-voice timing) with no
layout findings, whose cue ends, unit-transit, prediction and reveal frames
and glyph pages root read. Chapter 03 (`03_where_the_water_stops`, `.9.3`:
the scene, the cut and Gale's table from the model, the facts, the oracle
and the tests) is source and silent visuals verified by root: fmt, strict
clippy, all 43 tests then present and 96/96 facts against the oracle;
after the scene repairs, a build, its 4 changed tests and 68 glyph sources
without failure; a silent draft of 86.37 s (scratch-voice timing) with no
layout findings, whose reach-stroke and theorem-card frames root and an
independent reviewer read. It runs 1.37 s past the 85 s target because the
Gale card now stands whole 11.93 s, a readability repair root accepted.
Chapter 04 (`04_the_only_picture`, `.9.4`: the scene, the tree's flow,
the switch checks, the facts, the oracle and the tests) is source and silent
visuals verified by root: fmt, strict all-target clippy, 53 tests, 114/114
facts against the oracle, 93 glyph sources without failure and a build; a
silent draft of 82.33 s (2,470 frames, scratch-voice timing) with no layout
findings, six of whose frames and all nine glyph pages root read. An
independent reviewer checked its frames: the tree's water reroutes for real,
no answer shows during the prediction's hold, the face's water picture's 12
switches recount, and the tree's rows nest; the Ryser card stands whole
11.03 s, the two closing questions with their answers 11.93 s and 9.13 s.
This lane compiled, rendered and synthesized nothing. No chapter has narration, and no continuous audio or
whole-video acceptance is claimed; the owner's 2026-10-19 rejection of the
first-wave narration applies, and the narration units are a proposed text
grouping only. Chapter 05 is planned in the storyboard's arc, not written.

| File | What it holds |
|---|---|
| `src/model.rs` | the mathematics: pictures, shadows, switches, every picture that fits, walks by switches, Gale's network and its largest flow with every augmenting path it sends, the replay that rebuilds the water path by path, refusing a path that repeats a row or column or breaks a capacity (row and column balance asserted as an invariant), the cut where the water stops, Gale's table and test, the bottleneck; rectangles, rows that nest, and the water's picture with its switches |
| `src/facts.rs` | every number the video shows or says, and the edge checks on every 4 × 4 picture |
| `oracle.py` | the same numbers computed independently (standard library only), and 45 cross-checks |
| `src/chapters/same_shadows.rs` | chapter 01's scene; its counts, squares and switch swap are chapter 04's too |
| `src/chapters/water.rs` | chapter 02's scene: the real trace, drawn from replayed states only; its plumbing (counters, units in transit) is chapter 03's too |
| `src/chapters/stops.rs` | chapter 03's scene: where the no-fit water stops, the cut there, and Gale's table; its reading hold is chapter 04's too |
| `src/chapters/only.rs` | chapter 04's scene: the water's picture and its switches, the tree's water, why the tree has no switch, Ryser's theorem |
| `src/chapters/draw.rs` | the grid, its squares and counts, switches, thumbnails, the think ring, line timing |
| `docs/storyboard.md` | the words and pictures (the source of `script.tsv`), the arc of the planned chapters |
| `narration_units.json` | the script's lines grouped into whole thoughts for a future take (four a chapter), each the exact text of its lines |
| `docs/fact-sheet.md` | every number's source and every non-numeric claim's evidence |
| `docs/brief.md`, `docs/protected.tsv` | the brief; the words every revision must keep |

```bash
python3 -I -B oracle.py                                   # every number, 45 cross-checks on stderr
python3 -I ../../tools/storyboard.py --check discrete_tomography
python3 -I ../../tools/lint_script.py script.tsv
./x discrete_tomography protected                        # docs/protected.tsv against the script
./x discrete_tomography facts-check                      # the Rust facts against the oracle (builds)
```

## Sources

- **Ryser 1957** (Canad. J. Math. 9, 371–377; read in the scanned original):
  Theorem 3.1, "Let A and A* be two m by n matrices composed of 0's and 1's,
  possessing equal row sum vectors and equal column sum vectors. Then A is
  transformable into A* by a finite number of interchanges." An interchange
  swaps a 2 × 2 minor between the two alternating types: the video's switch.
  Theorem 2.1 is the existence condition by majorization; a note added in
  proof credits Gale with it independently.
- **Gale 1957** (Pacific J. Math. 7, 1073–1082; read): filling a matrix with
  zeros and ones to prescribed row and column sums is feasible exactly when
  the row sums and the column sums have the same total and, with the column
  sums in decreasing order, every k of them total at most the first k terms of
  the row sums' dual sequence (Gale states the problem for equal totals); the
  proof is a flow, each row able to send at most min(a_i, k) into k columns.
  It also gives a direct "n-step" construction. Gale illustrates it with
  families seated on buses.
- **Ford and Fulkerson 1957** ("A simple algorithm for finding maximal
  network flows and an application to the Hitchcock problem", Canad. J.
  Math. 9, 210–218; read pp. 210–213): the labeling method works on a matrix
  of room, each arc's capacity "diminished by the flow from P_i to P_j and
  increased by the flow from P_j to P_i" (so water can be taken back); each
  step sends a positive whole number, so it ends; Lemma 1, the result is a
  flow; Lemma 2, it is a largest one, the labeled nodes giving a minimal cut.
  The model labels breadth first; that choice is the one Edmonds and Karp
  analysed in 1972, a paper **not read here** and not named in the video.
- **Batenburg 2006** (Leiden thesis; read §1.1, §2.2): Ryser and Gale
  independently; Gale first framed the two-projection problem as a network
  flow; a picture has another with the same projections exactly when it
  contains a switching component (his Theorem 2, citing Ryser's 1963
  monograph, not read here); the number of solutions can be exponential in
  the image size; reconstruction from more than two projections is NP-hard
  (citing Gardner, Gritzmann and Prangenberg 1999, **not read here**).

## Limits that remain open

- **Competition.** Five bounded web searches (two restricted to YouTube)
  found no comparable video and one unopened title; no video was watched.
  A complete review of the closest treatments, including nonogram and CT
  explainers, is still owed before the script locks.
- **Scope.** Two directions, exact counts, filled-or-empty squares. Real
  tomography is continuous, noisy and many-angled; the NP-hardness claim for
  three or more directions is known here only through Batenburg's citation.
- **Review.** The silent drafts of chapters 01 to 04 were rendered and
  read by root (layout lint empty for all four; chapter 02's two cards
  legible in two lines). Chapter 03's first draft failed review three ways:
  teal reach strokes crossed the cut-off first column, the reach label was
  cut short, and the 218-character Gale card stood whole only 7.27 s. All
  three were repaired (strokes over the reached columns only, the full
  label, a 176-character card held for reading) and read again in the next
  draft. Chapter 04's draft passed root's frame reading and an independent
  frame review with no repairs. Nothing was compiled or rendered in this
  lane. The drafts use the scratch voice: real
  timing, the runs of units against their words, and every question and
  reveal against a voice remain unmeasured.
- **Narration.** Not synthesized. The pacing notes in the storyboard (the
  think window at 01_3, the question and answer at 01_8, the closing
  questions) are direction for a take, not evidence of one.
