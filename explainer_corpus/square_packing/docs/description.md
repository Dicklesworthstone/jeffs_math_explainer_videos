<!--
Draft delivery description for square_packing (2026-10-18). Every number
outside the Sources is a fact of docs/fact-sheet.md, computed in src/facts.rs
and recomputed by oracle.py, or a cited source's; docs/description_numbers.tsv
binds each one (this video's fact keys carry no tag prefix, so each binding is
a row: a `derived` row evaluates the fact it names), and
`python3 -I tools/description_numbers.py square_packing` checks them all.
The chapter list carries no timestamps: they come from the final narrated cut,
which does not exist yet. The eight chapters are accepted as silent drafts
only, and the narration voice waits on the owner's listening. Nothing here
approves publication.
-->

# Why Turning a Square Can Make the Whole Box Smaller

Five equal square tiles: what is the smallest square box that holds them? A
box three tiles wide works, with room for four more. Turning the middle tile
an eighth of a turn lets the box shrink to exactly 2 + 1/√2 tiles wide, about
2.71. This video builds that packing and checks it exactly, then proves the
harder half: that no narrower box holds five tiles. Along the way it keeps
three questions apart: does a packing fit, can it move there, and could
anything smaller fit? A picture answers the first, a motion the second, and
only an argument about every arrangement the third.

## Chapters

1. The turned tile
2. Four pins
3. A turn by a hair
4. The corner lemma
5. Near a side
6. A triangle of points
7. Four points, five tiles
8. What is proved and what is found

## Precisely

**The box.** The box is B = 2 + 1/√2 exactly: a corner tile, the square gap
between the corner tiles, and another corner tile. The gap's diagonal runs
between two inner corners that touch opposite sides of the turned tile, so it
is one tile long, and the gap's side is 1/√2. B is about 2.71, or 2.70711 to
five places; both are roundings, and every argument uses B itself. The
packing is checked in exact arithmetic on numbers a + b√2: no two tiles
overlap, every tile is inside, and every contact is exactly zero.

**Straight tiles need three (chapter 2).** Marks at a third and two thirds of
each side cut it into pieces shorter than a tile, so in a box narrower than
three every straight tile, lined up with the walls, holds one of four pins
strictly inside, and tiles cannot share one: at most four straight tiles fit.
This argument covers straight tiles only. The turned tile holds no pin, which
is why it can beat three.

**A local fact (chapter 3).** With the box, the corner tiles and the middle
tile's centre held where they are, turning the middle tile by a small angle
δ, other than zero and at most an eighth of a turn either way, pushes the
inner corners into it by ½(1 − cos δ): tiny, but never zero for such a turn.
(A quarter turn brings a square back onto itself, so this is a statement
about small turns only.) That shows this one motion is blocked. It is not a
proof that no other arrangement fits a smaller box.

**Points no tile can dodge (chapters 4 to 6).** Every unit tile inside the
box B wide covers one of four points, one tile in from each pair of walls:
(1, 1), (1, 1 + 1/√2), (1 + 1/√2, 1) and (1 + 1/√2, 1 + 1/√2). "Covers"
means inside or on the edge. The three lemmas come from Erich Friedman's
survey, with proofs reconstructed here:

- Corner blocks (Lemma 1). The point a tile in a corner must cover lies at
  most s(1 − s/2) = ½ − ½(s − 1)² ≤ ½ from the tile's centre along its
  sides, s being the tile's shadow on a wall.
- Side strips (Lemma 2). A tile resting on the floor slips between the two
  points only if they stand higher than s − x(s² − 1)/2, x being the gap.
  The survey's sketch says this height is lowest at 45°. For the gap it is
  lowest for a straight tile instead: exactly 1, against 3√2/4 at 45°. The
  sketch's claim fails for every gap below 2√2 − 2, about 0.83. The lemma
  still holds, because it also assumes the points are at most one tile up.
- The middle square (Lemma 3). If a tile missed all three corners of a
  triangle with sides at most one around its centre, two corners would lie
  past opposite sides of the tile (more than a tile apart), or all three past
  two neighbouring sides (beyond one diagonal, so the triangle misses the
  centre).

Covering allows the edge. Where five tiles fit in the box B wide, each point
lies on the edges of two tiles and inside none, so tiles cannot simply be
counted against points.

**Nothing smaller fits (chapter 7).** Suppose five tiles fit a box s < B
wide. Enlarge the whole picture by λ = B/s > 1: the box becomes B wide, every
tile becomes λ wide, and their insides still do not overlap. Inside each
grown tile, keep the unit tile with the same centre: it lies in the box B
wide, so it covers a point, and that point is strictly inside the grown tile,
at least (λ − 1)/2 deep. Five grown tiles would each need their own point,
and there are four. So no box narrower than B holds five tiles, and the
turned packing shows B does: B is the smallest box for five. This is Göbel's
result (1979), as presented in Friedman's survey. At λ = 1 the depth is zero,
which is why the points can sit on edges where five fit. The video draws no
five tiles in a box narrower than B: that is the supposition ruled out.

**Beyond five (chapter 8).** Ten tiles fit a box 3 + 1/√2 wide, about 3.71:
nine straight and one turned in a gap, as in the middle of five. Eleven
fit a box 1939/500 = 3.878 wide, with five tiles turned by cos = 105/137,
sin = 88/137, about 40° (39.97°). Each is checked exactly: every tile a true
unit square, none overlapping, all inside. A check shows a box is big enough,
and nothing more. That no narrower box holds ten is a published theorem of
the paper this figure comes from (Stromquist, 2003), not proved here. The
eleven-tile box was chosen for exact arithmetic: it is not the smallest
known, and the video says nothing about the smallest box for eleven.

## What the video does not claim

- That a picture of a packing proves the box is the smallest. For five, the
  proof is chapters 4 to 7.
- That 2.71 or 2.70711 is the box: both are roundings of B.
- That the straight-tile argument of chapter 2 covers turned tiles.
- That the blocked turn of chapter 3 proves anything beyond that one motion.
- A proof of any lower bound for ten or eleven tiles. That no narrower box
  holds ten is Stromquist's published theorem, credited and not reproduced
  here; nothing is claimed about the smallest box for eleven.

## Sources

- S1: J. Levy, *Squares in squares*, the case of five,
  https://jlevy.github.io/squares/cases/5.html (the side 2 + √2/2 and its
  decimal; the packing).
- S2: Erich Friedman, "Packing Unit Squares in Squares: A Survey and New
  Results", Electronic Journal of Combinatorics, Dynamic Survey DS7 (version
  of 14 August 2009), https://doi.org/10.37236/28 (Theorem 2 and Lemmas 1 to
  3; the HTML version at
  https://erich-friedman.github.io/papers/squares/squares.html).
- S3: F. Göbel, "Geometrical Packing and Covering Problems", in A. Schrijver
  (ed.), Packing and Covering in Combinatorics, Math Centrum Tracts 106
  (1979), 179–199, as cited in S2 (not seen).
- S4: W. Stromquist, "Packing 10 or 11 unit squares in a square", Electronic
  Journal of Combinatorics 10 (2003) #R8,
  https://www.combinatorics.org/ojs/index.php/eljc/article/download/v10i1r8/pdf/
  (the ten-tile figure).
- S5: the family of W. Trump's eleven-tile construction, as reconstructed by
  Ellsworth, §2 of
  https://github.com/jlevy/squares/blob/main/packing/resources/web/n11-optimality-2026-09-29/source/PROOF.md
  (pinned original: Queuingtheorydotcom/11SquaresOptimal at
  f9e0de713a0949d1bc6a0fa6b59d96edf6c3d65c); the eleven-tile packing shown is
  one exact member of it, not the smallest known.
