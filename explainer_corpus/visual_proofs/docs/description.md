<!--
Draft video description for Video I (P3, 2026-10-13). Every statement below
rests on a row of docs/fact-sheet.md (the bracketed tags: F- keys, S sources)
or on the source named in the sentence. It holds the exact versions of what
the narration simplifies (plan rule 11), the programmer notes that have no
room on screen, and what review round R1 and the claim walk sent "to the
description". The release package adds the chapter timestamps and the
repository link, and trims this to what the upload page accepts, linking this
file for the rest.
-->

# Proofs You Can See

Eight proofs you can see, and one picture that lies. Each proof is shown at
one size and then asked the same question: would the next size fit? The
answer is always a picture of the step from one size to the next. At the end,
the same argument gives nonsense, and the video says what separates the two.

Nine of the questions are "your turn" quizzes with a countdown. Pause
whenever you want more time.

## Chapters

(Timestamps come with the finished cut.)

1. Cold open
2. Five for five
3. Layers
4. Staircases
5. Slabs
6. Every cube
7. Three stacks
8. The box
9. Every height
10. Halves
11. A third
12. The cut cube
13. Four triangles
14. The missing square
15. What makes a proof

## The proofs, exactly

Each statement is for every whole number n from 1 (or every pair of
positive lengths), as the card in the video says.

- **Odd numbers as layers.** An L of two arms, each as long as the side k of
  the square it wraps, and one corner tile has 2k + 1 tiles and makes the
  (k + 1) by (k + 1) square, so 1 + 3 + … + (2n − 1) = n². The hundredth odd
  number is 199 and the first hundred add to 10,000 [F-O.hundredth,
  F-O.sum.hundred]. The L-shaped layer is the Greek *gnomon*; Aristotle's
  *Physics* III.4 uses the word, and reading it as the odd numbers is
  Heath's [S17].
- **Two staircases.** The staircase of columns 1 to n and its half-turned
  copy lock into a rectangle n wide and n + 1 tall, so 1 + 2 + … + n =
  n(n + 1) ÷ 2; a hundred steps make 5,050 [F-T.count.hundred].
- **The sum of cubes.** 1³ + 2³ + … + n³ = (1 + 2 + … + n)². The layer of
  cube k is k slabs of k by k wrapped round the square of side 1 + … + (k −
  1); each arm is k wide and as long as that side, which is the earlier
  cubes' sides laid end to end. Pairing piece j with piece k − j makes whole
  slabs, and for an even cube the one piece left over is half a slab, so one
  slab is cut in two (halves at the ends of the two arms). Arms for the
  seventh and eighth cubes: 7 by 21 (3 slabs) and 8 by 28 (3½ slabs)
  [F-K.arm.7, F-K.arm.8]. After five cubes the square is 15 by 15 = 225; the
  sixth cube takes it to 21 by 21 = 441 [F-K.side, F-K.sixth.total, F-K.unit_squares]. Cut into
  one-tile layers, cube k is the k consecutive odd numbers from k(k − 1) + 1,
  so the five cubes are the first fifteen odd numbers [F-K.odd_runs].
- **Who did what.** Nicomachus (about the year 100) noticed that each cube
  is a run of consecutive odd numbers (1; 3 + 5; 7 + 9 + 11) and did not state
  the rule that carries his name [S2, S3]. Aryabhata states the rule in 499
  [S4]. Al-Karaji, around the year 1000, proved it with a figure of gnomons
  [S3]. The picture with a slab cut in half for each even cube is Alan Fry's,
  *Mathematics Magazine* 58(1), January 1985, p. 11 [S5]; the printed page was
  not seen, and earlier L-shaped pictures exist (Golomb, 1965; Love, 1977)
  [S6]. [F-X.nicomachus, F-X.aryabhata, F-X.al_karaji, F-X.fry] [F-K.odd_runs]
- **Siu's three stacks.** Man-Keung Siu, *Mathematics Magazine* 57(2), March
  1984, p. 92 [S8]: three stacks of n levels fit a box n by n + 1 by n + ½
  once the cubes that stick out on top are sliced flat and their upper halves
  turned into the empty half; so 1² + … + n² = ⅓ n (n + 1)(n + ½) = n(n + 1)
  (2n + 1) ÷ 6 [F-S.box, F-S.box.volume]. The empty half is the step
  half-turned, which is the two staircases again. The next height is the same
  solid lifted one cube with three new slabs of (n + 1) by (n + 1): under,
  behind and beside it [F-S.grown.slab, F-S.grows.through]. With whole cubes
  three stacks cannot make a box (90 cubes have no box with every side at
  least 4); six whole stacks can, 4 by 5 by 9, found by exhaustive search and
  checked cell by cell; six is also the fewest for whole cubes, since four and
  five stacks do not tile their only candidate boxes (4 by 5 by 6 and 5 by 5 by
  6), by exact-cover search [F-D.whole.side_least, F-D.whole.boxes,
  F-D.whole.filled, F-D.whole.fewest]. Three is the fewest under the video's rule, by the
  size of the box [F-S.fewest, F-S.fewest.box, F-D.fewest.cubes]: a stack is 4 by 4 by 4 at its
  widest in every direction, so the only box that can hold one and has 30, 60
  or 90 cubes is 4 by 5 by 4½. That is an argument from necessary conditions
  for boxes on the cube lattice (whole width and depth, height in halves), not
  a search over every packing. [F-X.siu, F-S.stack, F-S.sums]
- **Halves.** After n pieces the filled part is 1 − 1/2ⁿ and the gap is
  exactly the last piece, 1/2ⁿ; the gap is never zero, and it is below every
  size you name from some n on. "Equals one" here means that no partial sum
  reaches one and every number below one is passed; for pieces that are all
  positive, that is the whole definition [F-H.filled, F-H.gap,
  F-H.checked.through]. Warren Page published a picture of this sum in
  *Mathematics Magazine* 54(4), September 1981, p. 201 [S9]; no source was
  found that dates the picture earlier, so no one is named in the video. [F-X.page]
- **Quarters.** After n cuts each colour holds (1 − 1/4ⁿ) ÷ 3 of the square,
  short of one third by a third of the uncut corner (1/4ⁿ ÷ 3), so each
  colour plus its share of the corner is exactly one third at every n, and
  1/4 + 1/16 + 1/64 + … = 1/3 [F-R.share, F-R.left, F-R.short]. The picture
  is Sunday Ajose and Roger Nelsen's (Sunday A. Ajose and Roger B. Nelsen,
  *Mathematics Magazine* 67(3), June 1994, p. 230; attribution partly
  verified) [S10]. Archimedes, *Quadrature of the Parabola*, Proposition 23,
  is a finite identity with a remainder, A + B + … + Z + ⅓Z = (4/3)A; he does
  not sum an infinite series [S11]. [F-X.ajose_nelsen] [F-R.limit, F-R.shares]
- **The cut square, cube and fourth power.** Cut every side of a cube of
  side a + b once, at a: the 2ᵈ blocks have j short sides in C(d, j) cases
  and their volumes add to (a + b)ᵈ; Euclid's *Elements* II.4 is the square,
  in words [S12]. With a = 3 and b = 2 the cube's blocks are 27, 18, 12 and 8
  in 1, 3, 3 and 1 cases, and the fourth power's sixteen blocks have volumes
  81, 54, 36, 24 and 16 in 1, 4, 6, 4 and 1 cases, adding to 81 + 216 + 216 +
  96 + 16 = 625 = 5⁴ [F-X.euclid, F-B.parts, F-B.cube.each, F-B.cube.kinds, F-B.fourth.kinds, F-B.fourth.each,
  F-B.fourth.by_kind, F-B.fourth.total]. A block with four lengths at right
  angles cannot be built in three-dimensional space, so the fourth power is
  counted, not drawn: each count is the sum of two neighbours in the row
  above [F-B.fourth.from_cube]. Slide the cut anywhere between the ends and
  the cube is still eight blocks; at the very end of a side seven of them
  have no volume. Past the middle the part called long is the smaller one;
  the identity does not care which is which.
- **Four triangles.** Four copies of the 3-4-5 triangle in a 7 by 7 square
  leave a tilted square of 49 − 24 = 25; slid without turning they leave a 3
  by 3 and a 4 by 4 [F-P.big, F-D.four_triangles.side, F-P.triangles, F-P.tilted, F-P.squares]. That
  second arrangement is the cut square of the section above, (a + b)² = a² +
  2ab + b², read against the four triangles. It
  holds for every pair of shorter sides a and b, because the tilted shape has
  four sides of length √(a² + b²) and four right angles (the two sharp
  angles of a right triangle make a right angle), and with 2 and 5 the long
  side is not a whole number (the tilted square is 29) [F-P.other.tilted,
  F-P.other.whole_side]. Nobody is named: no source found says who first drew
  this picture; a close cousin is in a third-century commentary on the
  *Zhoubi suanjing*, whose figure is a different one, and Loomis credits the
  two-arrangement figure to A. D. Wheeler (1859), the earliest printing found
  and not a claimed first [S13]. It is not "Pythagoras's own proof" and not "a
  Chinese proof from 200 BC". [F-P.legs, F-P.other.legs, F-X.wheeler]
- **The picture that lies.** Four pieces of 12, 5, 8 and 7 tiles add to 32,
  the triangle they seem to fill is 13 by 5 and would hold 32½ [F-L.pieces,
  F-L.pieces.total, F-L.true_triangle]. The two slopes are 3/8 and 2/5 and
  the straight edge from corner to corner is 5/13 [F-L.slopes,
  F-L.straight_slope]; each bend is 1/13 of a tile off that straight edge
  (measured straight up) [F-L.dip, F-L.bulge]. The two outlines have area 32
  and 33 and the sliver between them is exactly one tile, at most 1/8 thick;
  cut along the grid it falls into 17 pieces that slide by whole tiles into
  the one empty tile [F-L.outline.first, F-L.outline.second, F-L.sliver,
  F-L.sliver.thickest, F-L.sliver.pieces, F-L.sliver.fold]. The sliver is the
  parallelogram with sides (5, 2) and (8, 3), whose area is 8 × 2 − 3 × 5 = 1:
  2, 3, 5, 8 and 13 are neighbours in the Fibonacci numbers, which is also why
  the slopes are only 1/40 apart [F-L.slopes.differ]. According to Martin
  Gardner, the idea is Paul Curry's, a New York amateur magician, in 1953;
  Gardner found triangular forms of it and published them in *Mathematics,
  Magic and Mystery* (Dover, 1956; pp. 139 and 145) [S14, F-X.curry,
  F-X.gardner]. Gardner's figures were not seen, so that his triangle is the
  13 by 5 one is inferred from his text.

## For programmers

- **Regions of a circle.** With n points on a circle and no three lines
  meeting at one point inside, the lines cut the disc into 1 + C(n, 2) +
  C(n, 4) regions: 1, 2, 4, 8, 16, 31, 57, 99 for n = 1 to 8, the first seven
  drawn in the video and the eighth drawn the same way in the oracle
  [F-C.closed_form, F-D.circle.general_position].
- **Evenly spaced points** need not be in that
  position: six give only 30, because the three long lines meet in the middle
  [F-C.evenly_spaced.regions], and eight give 88, not 99, because several lines
  meet inside (counted exactly in the oracle, in Q(√2)
  [F-C.evenly_spaced.regions.eight, F-D.circle.general_position]). The pattern is in Leo Moser's note "On the
  Danger of Induction", *Mathematics Magazine* 23(2), November–December
  1949, p. 109 [S1, F-X.moser].
- **Five tests in a row.** The doubling holds for 1 to 5 points and the cube
  totals 1, 9, 36, 100, 225 are squares: two patterns that pass five checks.
  Five passing tests prove nothing about the sixth [F-C.regions, F-D.circle.doubling_holds,
  F-C.cube_totals].
- **A loop against a formula.** Adding 1 + 2 + … + n one at a time is n
  steps; n(n + 1) ÷ 2 is one multiplication and a halving, and n(n + 1) is
  always even, so the integer division is exact. The picture is why it is
  right: two staircases make a rectangle [F-T.count.hundred].
- **Floats.** In 64-bit floating point the running sum ½ + ¼ + ⅛ + … is
  exactly 1.0 after 54 pieces: the machine cannot see the gap that the video
  says never closes [F-H.float.reaches_one, F-D.float.sum_at_reach].
- **Induction is a base case and a step.** The step from height n to n + 1
  in Siu's box is a function that returns the next solid from the last one;
  the chapter's check applies it for n = 1 to 24 and compares the cells
  [F-S.grows.through]. Like a recursive function or a loop invariant, it is
  the first case plus the step that is true for every n; checking n = 1 to 24
  is not that.
- **Two implementations.** The Rust model that draws the pictures and a
  Python oracle that rebuilds each by another route print the same
  facts, key for key. Neither is a proof; each reach (`.through` in the fact sheet) is
  how far the slow check was run: the cubes up to 40, Siu's box up to 24
  levels, every sum up to 1,000 [F-K.tiled.through, F-S.filled.through,
  F-K.sum.through].

## Try it

```python
# facts: F-C.closed_form, F-H.float.reaches_one, F-S.grows.through
from math import comb
[1 + comb(n, 2) + comb(n, 4) for n in range(1, 9)]   # 1, 2, 4, 8, 16, 31, 57, 99  [F-D.circle.general_position]

s, k = 0.0, 0                                        # the halved square in floats
while s < 1.0:
    k += 1
    s += 2.0 ** -k
print(k)                                             # 54

n = 4                                                # Siu's three stacks of n levels
s0 = {(x, y, z) for z in range(n) for x in range(n - z) for y in range(n - z)}
s1 = {(x, y, z) for y in range(1, n + 1) for x in range(y) for z in range(n - y, n)}
s2 = {(x, y, z) for x in range(n) for y in range(x + 1) for z in range(n - x, n + 1)}
box = {(x, y, z) for x in range(n) for y in range(n + 1) for z in range(n)}
step = {(x, y, n) for x in range(n) for y in range(x + 1)}   # the staircase on top
assert not (s0 & s1 or s0 & s2 or s1 & s2) and s0 | s1 | s2 == box | step
# the three are the same solid turned; n = 1 to 24 all pass. The box is n by n + 1
# by n + 1/2 once the staircase's cubes are sliced and its upper halves turned.
```

## Not said, on purpose

That Nicomachus stated or proved the sum of cubes; that Aryabhata was
certainly first; that Archimedes summed an infinite series; that Euclid wrote
the formula in letters or cut a cube; that the two arrangements are
Pythagoras's own proof or a Chinese proof from 200 BC; that Curry invented the
13 by 5 triangle, published it in 1953, or was a mathematician; that Gauss
added 1 to 100; that Page invented the halved square; that Moser proved or
named the circle problem. [F-L.frame, F-X.curry]

## Receipts

Every number in the video is computed by the scenes from the data they draw,
and an independent oracle checks it by another route. Sources, what was and
was not seen (no printed figure of the Mathematics Magazine pictures was
looked at), and the edge test behind every "always" are in
`docs/fact-sheet.md`.

Names to search: proofs without words; Roger B. Nelsen, *Proofs Without
Words* (MAA, 1993) and *Proofs Without Words II* (MAA, 2000); Mathematics
Magazine; Man-Keung Siu; gnomon; the missing square puzzle; Moser's circle
problem. [F-X.nelsen]
