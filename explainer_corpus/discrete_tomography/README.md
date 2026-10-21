# Two Shadows Can Lie

A picture made of filled squares casts two shadows: how many squares each row
holds and how many each column holds. This video asks what those counts can
tell you. A little face's shadows fit 64 different pictures; when the four
corners of a rectangle alternate filled, empty, filled, empty, swapping filled
and empty there (a switch) turns one into another without changing a single
count; the same row counts with other column counts of the same total fit no
picture at all. The eleven chapters then try the obvious method (reasoning
one row or column at a time settles a tree and stalls on the face), show that
trying every picture does not scale, build a picture by letting water flow
through the grid (Gale, 1957; Ford and Fulkerson, 1957), take water back and
show why the water always finishes, show where it stops when none fits and
why that stop never misses a group of columns that asks too much (rows and
columns 4, 4, 1, 1, 1: no line settles a square, yet nothing fits), prove
that a picture is the only one exactly when it has no switch and why that
needs Ryser's chain theorem (1957), add the diagonal shadows, and end on what
a program can promise from two shadows.

**Status (2026-10-20, revision `v1-michael-20261019`, eleven chapters):** the
owner's 2026-10-20 direction is a complete episode of 10 to 15 minutes; root
accepted the eleven-chapter expansion for source implementation with its
decisions (02_1 kept, the 08_4 callback kept, two wordings fixed, the short
04_6 card, read holds from each card's completed fade-in). The storyboard
estimates 12.7 minutes; nothing new is measured. Chapter 01's source and
silent visuals were verified by root at 6bae9ea and its F2/F6 repair is
`ec5eb74`. The five-chapter cut's chapters 02 to 05 were verified by root
(fa6fd0f, 8888687, fb90763, 48357c5; F1 to F6 `ec5eb74`); their lines are
now chapters 03 to 05, 07, 08 and 10, with 39 of the 43 current lines
byte-identical. The new source (chapters 02, 06, 07, 09 and 11, and the new
beats of 03, 04, 05, 08 and 10; the model's line reasoning and other
diagonals; the facts and the oracle's new sections) was written by PinkDuck
on 2026-10-20 and has **not been compiled, tested, built or rendered**: the
oracle passes its 67 cross-checks and emits every fact key, and the cheap
source checks pass (`IMPLEMENTATION.md` in the lane's candidate directory).
Root's serialized native lane (fmt, clippy, tests, facts-check, glyph probe,
silent drafts, frames read) comes next. No chapter's new narration exists,
and no continuous audio or whole-video acceptance is claimed; the owner's
2026-10-19 rejection of the first-wave narration applies, and the narration
units are a proposed text grouping only.

| File | What it holds |
|---|---|
| `src/model.rs` | the mathematics: pictures, shadows, switches, every picture that fits, walks by switches, reasoning one row or column at a time, Gale's network and its largest flow with every augmenting path it sends, the replay that rebuilds the water path by path (refusing a path that repeats a row or column or breaks a capacity), the cut where the water stops, Gale's table and test, the bottleneck; rectangles, rows that nest, both diagonal shadows and the mirror, and the water's picture with its switches |
| `src/facts.rs` | every number the video shows or says, and the edge checks on every 4 × 4 picture |
| `oracle.py` | the same numbers computed independently (standard library only), and 67 cross-checks |
| `src/chapters/same_shadows.rs` | chapter 01: the face, the switch, the 64, the impossible columns; its counts, squares, switch swap and gallery are later chapters' too |
| `src/chapters/reasoning.rs` | chapter 02: the title, the scoped crystal card, the tree settled line by line, the face stalled after its empty row, how often each square is filled |
| `src/chapters/water.rs` | chapters 03 and 04: why not every picture, the plumbing both ways, the real trace drawn from replayed states, the take-back and what it does, why the water finishes; its plumbing is chapters 05 to 08's too |
| `src/chapters/stops.rs` | chapter 05: where the no-fit water stops and why eight is all; its table, reach and outline helpers and the Gale card are chapters 06 and 07's |
| `src/chapters/group.rs` | chapter 06: rows and columns 4, 4, 1, 1, 1, the pair that asks too much, the water's stop there, Gale's table |
| `src/chapters/misses.rs` | chapter 07: the cut argument on chapter 06's numbers, the Gale card, equal totals, the face passing |
| `src/chapters/only.rs` | chapter 08: the water's picture and its switches, the tree's water, why the tree has no switch, what a switch does to reasoning, the 4 × 4 count |
| `src/chapters/chains.rs` | chapter 09: two 3 × 3 pictures two switches apart, Ryser's theorem as the tree's missing step, the 64 reached by switches, the two questions answered |
| `src/chapters/third.rs` | chapter 10: the diagonal shadow and its two survivors, the 64 grouped by their diagonals, the mirror and the other diagonals, four shadows and one picture |
| `src/chapters/promise.rs` | chapter 11: the four steps a program can follow, the face's report, what no program can say, the bounded model and the two questions |
| `src/chapters/draw.rs` | the grid, its squares, shades, dots and counts, switches, thumbnails, the think ring, line timing |
| `docs/storyboard.md` | the words and pictures (the source of `script.tsv`) and the arc |
| `narration_units.json` | the script's lines grouped into whole thoughts for a future take, each the exact text of its lines |
| `docs/fact-sheet.md` | every number's source and every non-numeric claim's evidence |
| `docs/brief.md`, `docs/protected.tsv` | the brief; the words every revision must keep |

```bash
python3 -I -B oracle.py                                   # every number, 67 cross-checks on stderr
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
- **Batenburg 2006** (Leiden thesis; read §1.1, §2.2 and §1.9.1, pp. 31–32):
  Ryser and Gale independently; Gale first framed the two-projection problem
  as a network flow; a picture has another with the same projections exactly
  when it contains a switching component (his Theorem 2, citing Ryser's 1963
  monograph, not read here); §1.9.1 (chapter 02's card, read 2026-10-20 and
  verified by root): a technique "made it possible to count the number of
  atoms in each projected column of a nanocrystal", rebuilding the atoms'
  positions was "one of the main envisioned applications", and "recording
  many images is often impossible". He also states that reconstruction from
  more than two projections is NP-hard (citing Gardner, Gritzmann and
  Prangenberg 1999, **not read here**: the original could not be fetched on
  2026-10-19, so the video does not make that claim).

## Limits that remain open

- **Competition.** Five bounded web searches (two restricted to YouTube)
  found no comparable video and one unopened title; no video was watched.
  A complete review of the closest treatments, including nonogram and CT
  explainers, is still owed before the script locks.
- **Scope.** Exact counts of filled-or-empty squares on small grids: the
  face's and the tree's 5 × 5, rows and columns 4, 4, 1, 1, 1, a 2 × 2 and
  two 3 × 3 examples, and every 4 × 4 picture for the edge checks; two
  directions in chapters 01 to 09, and the face's two diagonal directions in
  chapter 10, whose results (two pictures fit three shadows; the face alone
  fits four) are this example's only ("this picture, not a rule"). The video
  says nothing about how hard reconstruction is in general, and nothing
  about real tomography or scanners beyond chapter 02's scoped motivation.
- **Review.** The five-chapter cut's silent drafts were rendered and read by
  root, with independent frame reviews (the repairs and dwell measurements
  are recorded in the storyboard's earlier status and the lane's handoffs).
  The eleven-chapter source has no draft yet.
- **Narration.** Not synthesized for the new lines. Exact-unit audio reuse
  of the five-chapter review cut's takes is only a candidate, root's to
  decide, after the whole submitted text and its context binding match.
