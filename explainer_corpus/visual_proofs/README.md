# Proofs You Can See

Video I turns twelve classical theorems of sums, series, geometry and volume
into one rearrangement principle: **the pieces did not change, so the total did
not change.**

- **Sums you can count by rearranging.** Odd numbers pack into a growing square
  (`1 + 3 + 5 + … + (2n − 1) = n²`); two staircases of `1 + 2 + … + n` lock
  into an `n × (n + 1)` rectangle; three pyramid stacks of `1² + 2² + … + n²`
  fill a box of base `n × (n + 1)` and height `n + ½`; the `k`-th cube unfolds
  into `k` squares that tile a big square of side `1 + 2 + … + n`.
- **Infinite series inside one square.** Halving the remainder fills the unit
  square (`½ + ¼ + ⅛ + … = 1`); colouring one L-shaped third of each remainder
  shows `¼ + 1/16 + 1/64 + … = ⅓`.
- **Two squares in one frame.** Four `a-b-c` right triangles in a square of
  side `a + b` leave `c²` when tilted and `a² + b²` when slid to the corners
  (Pythagoras); cutting `(a + b)²` at `a` and `b` leaves `a² + 2ab + b²`, and
  folding a strip of width `b` inside `a²` leaves `a² − b² = (a − b)(a + b)`.
- **Three dimensions from slices and corners.** A cube cut at `a` and `b` along
  each edge falls into eight blocks (`(a + b)³ = a³ + 3a²b + 3ab² + b³`);
  sliding the slabs of a skewed pyramid leaves every horizontal slice unchanged
  (Cavalieri's principle), and three identical square pyramids of height equal
  to their base edge lock along their diagonals into a single cube (`V = ⅓ b² h`).
- **When the eyes lie.** Curry's 13-by-5 "missing square" triangle has slopes
  `2/5 = 0.400` and `3/8 = 0.375` (`1.04°` apart): the bend along the diagonal
  has area `½` on each rearrangement (`8 × 3 − 5 × 5 = −1`), proving why visual
  proofs require verifying that edges meet straight and pieces fit without gap
  or overlap.

Quizzes with a visible countdown run through the whole video.

**Status: in production.** All fifteen chapters, the facts and their
independent oracle (`./x visual_proofs facts-check`, 243 keys), the `michael`
narration, and the procedural score stems are built and under dense-read review
prior to the 4K master.

```bash
./x visual_proofs test          # tests, clippy -D warnings, fmt --check
./x visual_proofs facts-check   # every number against oracle.py
./x visual_proofs storyboard    # the G2 checks; writes script.tsv
```

- `docs/brief.md`: the owner's request in his words, and every default chosen.
- `docs/fact-sheet.md`: every number, every source with what was and was not
  seen, the simplifications with their exact versions, and the edge test
  behind each always sentence.
- `docs/storyboard.md`: the words and pictures, one row per narration line.
- `docs/description.md`: the companion notes, exact statements, programmer
  notes, and sources.
- `src/model.rs`: the mathematics as the video works it; `src/facts.rs`:
  every number the video shows or says; `oracle.py`: the same numbers by
  independent rational arithmetic and brute-force enumeration.
