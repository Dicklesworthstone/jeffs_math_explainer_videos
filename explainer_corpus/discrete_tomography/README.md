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

**Status (2026-10-19, `mx-video-program-program-87do.9.1`, revision
`v1-michael-20261019`):** chapter 01 (`01_same_shadows`) exists as source: the
scene, the model, the facts, the independent oracle and the tests. This lane
compiled, tested, rendered and synthesized nothing; root runs the native
build, tests, glyph probe and draft and inspects the frames. Chapters 02 to 05
are planned in the storyboard's arc, not written. No narration exists, and
the owner's 2026-10-19 rejection of the first-wave narration applies: the
narration units are a text plan only.

| File | What it holds |
|---|---|
| `src/model.rs` | the mathematics: pictures, shadows, switches, every picture that fits, walks by switches, Gale's network and its largest flow, Gale's test, the bottleneck |
| `src/facts.rs` | every number the video shows or says, and the edge checks on every 4 × 4 picture |
| `oracle.py` | the same numbers computed independently (standard library only), and 14 cross-checks |
| `src/chapters/same_shadows.rs` | chapter 01's scene |
| `src/chapters/draw.rs` | the grid, its squares and counts, switches, thumbnails, the think ring, line timing |
| `docs/storyboard.md` | the words and pictures (the source of `script.tsv`), the arc of the planned chapters |
| `narration_units.json` | the script's lines grouped into four whole thoughts for a future take, each the exact text of its lines |
| `docs/fact-sheet.md` | every number's source and every non-numeric claim's evidence |
| `docs/brief.md`, `docs/protected.tsv` | the brief; the words every revision must keep |

```bash
python3 -I -B oracle.py                                   # every number, 14 cross-checks on stderr
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
- **Review.** Not compiled, rendered or looked at in this lane. The layout is
  checked by tests on its constants only; text widths (the gallery's caption,
  01_8) are estimates until the layout lint and the frames are read.
- **Narration.** Not synthesized. The pacing notes in the storyboard (the
  think window at 01_3, the question and answer at 01_8, the closing
  questions) are direction for a take, not evidence of one.
