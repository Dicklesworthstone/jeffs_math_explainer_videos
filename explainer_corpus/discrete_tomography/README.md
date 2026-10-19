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
draft and its frames); no audio is claimed. Chapter 02
(`02_water_through_the_grid`, `.9.2`: the scene, the recorded flow trace and
its replay checker, the facts, the oracle and the tests) is source and silent
visuals verified by root: fmt, strict clippy, 35 tests, a release build and
73/73 facts against the oracle; a silent 960x540 draft of 81.73 s
(scratch-voice timing) with no layout findings, whose cue ends, unit-transit,
prediction and reveal frames and glyph pages root read. This lane compiled,
rendered and synthesized nothing. Neither chapter has narration, and no
continuous audio or whole-video acceptance is claimed. Chapters 03 to 05 are
planned in the storyboard's arc, not written. No narration exists, and the owner's
2026-10-19 rejection of the first-wave narration applies: the narration units
are a proposed text grouping only.

| File | What it holds |
|---|---|
| `src/model.rs` | the mathematics: pictures, shadows, switches, every picture that fits, walks by switches, Gale's network and its largest flow with every augmenting path it sends, the replay that rebuilds the water path by path, refusing a path that repeats a row or column or breaks a capacity (row and column balance asserted as an invariant), Gale's test, the bottleneck |
| `src/facts.rs` | every number the video shows or says, and the edge checks on every 4 × 4 picture |
| `oracle.py` | the same numbers computed independently (standard library only), and 31 cross-checks |
| `src/chapters/same_shadows.rs` | chapter 01's scene |
| `src/chapters/water.rs` | chapter 02's scene: the real trace, drawn from replayed states only |
| `src/chapters/draw.rs` | the grid, its squares and counts, switches, thumbnails, the think ring, line timing |
| `docs/storyboard.md` | the words and pictures (the source of `script.tsv`), the arc of the planned chapters |
| `narration_units.json` | the script's lines grouped into four whole thoughts for a future take, each the exact text of its lines |
| `docs/fact-sheet.md` | every number's source and every non-numeric claim's evidence |
| `docs/brief.md`, `docs/protected.tsv` | the brief; the words every revision must keep |

```bash
python3 -I -B oracle.py                                   # every number, 31 cross-checks on stderr
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
- **Review.** Both chapters' silent drafts were rendered and read by root
  (layout lint empty for both; chapter 02's two cards legible in two lines);
  nothing was compiled or rendered in this lane. The drafts use the scratch voice: real timing, the
  02_4 run of units against 02_4's words, and every question and reveal
  against a voice remain unmeasured.
- **Narration.** Not synthesized. The pacing notes in the storyboard (the
  think window at 01_3, the question and answer at 01_8, the closing
  questions) are direction for a take, not evidence of one.
