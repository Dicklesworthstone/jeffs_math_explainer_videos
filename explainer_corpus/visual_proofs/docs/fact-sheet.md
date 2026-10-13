# Fact sheet — Proofs You Can See

Every number Video I shows or says, with where it comes from, and every
source with what was and was not seen. `./x visual_proofs facts-check` runs
the video's `facts` (computed in `src/model.rs` and `src/facts.rs` from the
pictures themselves) against `oracle.py`, which rebuilds every picture by
its own route; both print the keys below. A test in `src/facts.rs` fails if
a key or value here drifts from what the video computes.

A key that ends in `.through` is the reach of a slow check: the largest size
up to which the picture was rebuilt piece by piece and found to fit with
nothing left over. A reach is a check, not a proof (15_5 says so on screen);
the proof of each picture is the step named under "the next size".

## Sources

What was seen is stated for each; a search snippet is called a snippet.
Journal pages at the publisher were not reachable, so the Mathematics
Magazine citations rest on Crossref's publisher records and on the Sources
list printed in Nelsen's book (read as the text of a scan of a later
printing). **No printed figure was looked at**: which printed picture is
which rests on descriptions, and is marked so. The owner's reference images
are other people's pictures and are not in this repository.

| id | Source | Used for | Seen |
|---|---|---|---|
| S1 | Leo Moser, "On the Danger of Induction", *Mathematics Magazine* 23(2), November–December 1949, p. 109 | the circle's regions: 1, 2, 4, 8, 16 and then 31 for 6 points, "no three chords concurrent" | the note's text (OCR of the issue): "if we go to n = 6 we find that f(6) = 31"; its formula is garbled in the OCR and is not quoted. Martin Gardner, *Mathematical Circus* (1979), ch. 14, calls it a "problem discovered by Leo Moser" (snippet); MathWorld: "sometimes known as Moser's circle problem"; OEIS A000127 (mirror) |
| S2 | Nicomachus of Gerasa, *Introduction to Arithmetic* (about 100 CE), II.20.5, trans. M. L. D'Ooge (Macmillan, 1926), pp. 263–264; II.9.3, p. 243 | cubes as runs of odd numbers; squares as sums of odd numbers | "the first one makes the potential cube; the next two, added together, the second; the next three, the third"; and (II.9.3) "the third, 1 plus 3 plus 5, is the second in actuality" |
| S3 | T. L. Heath, *A History of Greek Mathematics*, vol. 1 (Oxford, 1921), pp. 77, 82–84, 108–109 | what Nicomachus did not say; al-Karaji's proof; the gnomons | "Nicomachus does not give this formula, but it was known to the Roman agrimensores"; al-Karaji ("Alkarkhi … of the tenth-eleventh century") "has a proof of the theorem now in question by means of a figure with gnomons drawn in it"; "The odd numbers successively added were called gnomons" |
| S4 | Aryabhata, *Aryabhatiya* (499 CE), Ganitapada stanza 22, trans. W. E. Clark (Chicago, 1930), pp. 37–38 | the first explicit statement of the sum-of-cubes rule that was read | "The square of the sum of the (original) series is the sum of the cubes." The date 499 is from D. Pengelley (arXiv 1912.03527) |
| S5 | Alan L. Fry, "Proof without Words: Sum of Cubes", *Mathematics Magazine* 58(1), January 1985, p. 11 | the slab picture with one slab cut in half for an even cube | Crossref record; Nelsen p. 86. The picture by description only (S. M. Blinder's Wolfram Demonstration, 2020, citing Fry): "For an even summand, one of the square slabs is cut in half for each end of the L-shape." **Partly verified**: the printed page was not seen |
| S6 | Solomon W. Golomb, "3121. A Geometric Proof of a Famous Identity", *The Mathematical Gazette* 49(368), May 1965, pp. 198–200; J. Barry Love, "Proof without Words: Cubes and Squares", *Mathematics Magazine* 50(2), March 1977, p. 74 | earlier pictures of the same L-shaped layers; not named on screen | Cambridge Core page (Golomb); Crossref (Love). Golomb's figure has overlaps and equal holes, not halves (Gardner's description, snippet) |
| S7 | Wikimedia Commons, `File:Nicomachus_theorem_3D.svg`, by the user Cmglee, 2015 (CC BY-SA 3.0 or GFDL); Wikipedia credits the idea to Ned Gulley, MathWorks blog, 4 March 2010 | the three-dimensional drawing in the owner's image | the file page. The video redraws the idea from `model::cubes`; the image is not copied |
| S8 | Man-Keung Siu, "Proof without Words: Sum of Squares", *Mathematics Magazine* 57(2), March 1984, p. 92 | the three stacks and the box n by n + 1 by n + ½ | Crossref record; Nelsen p. 77, "Sums of Squares I —Man-Keung Siu". Siu: University of Hong Kong, 1975 to 2005 (HKU Honorary University Fellows page; his own page reads "Professor M.K. Siu (蕭文強教授)"). Not to be confused with J. O. Chilaka's item of the same title, 56(2), 1983, p. 90 |
| S9 | Warren Page, "Proof without Words: Geometric Sums", *Mathematics Magazine* 54(4), September 1981, p. 201 | a published version of the halved square | Crossref; Nelsen p. 118, under the general identity r + r(1 − r) + r(1 − r)² + … = 1. No source was found that dates the halved square earlier or says Page originated it, so nobody is named on screen |
| S10 | Sunday A. Ajose and Roger B. Nelsen, "Proof without Words: Geometric Series", *Mathematics Magazine* 67(3), June 1994, p. 230 | the quartered square for 1/4 + 1/16 + … = 1/3 | Crossref (both authors); archived MAA page ("by Sunday A. Ajose (East Carolina University) and Roger B. Nelsen (Lewis and Clark College)"). That this is the square picture and Rick Mabry's (72(1), 1999, p. 63) the triangle: Wikipedia's citations and cut-the-knot. **Partly verified** |
| S11 | Archimedes, *Quadrature of the Parabola*, Proposition 23, in T. L. Heath, *The Works of Archimedes* (Cambridge, 1897), p. 249 | the same quarters, as a finite sum with its remainder | "Given a series of areas A, B, C, D, … Z, of which A is the greatest, and each is equal to four times the next in order, then" A + B + … + Z + ⅓Z = (4/3)A. Heath: "Archimedes does not express it in this way" (as an infinite series) |
| S12 | Euclid, *Elements*, Book II, Proposition 4 (Heath's translation, 2nd ed. 1926, vol. 1, p. 379) | the cut square | "If a straight line be cut at random, the square on the whole is equal to the squares on the segments and twice the rectangle contained by the segments." Euclid has no cube analogue (the enunciations of Books VII to XIII were read) |
| S13 | Elisha S. Loomis, *The Pythagorean Proposition* (1927; 2nd ed. 1940); C. Cullen, *Astronomy and Mathematics in Ancient China: the Zhou bi suan jing* (Cambridge, 1996); Heath on Euclid I.47 | what may be said about who first drew the two arrangements | Heath: "no positive evidence" that Pythagoras argued this way. Cullen (snippets): the Zhoubi was "probably assembled under the Western Han dynasty during the first century BC"; its diagram is Zhao Shuang's, third century CE, and is a different figure (four triangles round a small square inside the square on the long side). Loomis credits the two-arrangement figure to A. D. Wheeler, 1859, the earliest printing found, not a claimed first |
| S14 | Martin Gardner, *Mathematics, Magic and Mystery* (New York: Dover, 1956), pp. 139, 145 | the missing-square puzzle | p. 139: "The inventor of this delightful paradox is Paul Curry, a New York City amateur magician. In 1953 he conceived the brilliant notion of cutting and rearranging the parts of a figure to form an identical figure with a hole inside its perimeter." p. 145: "My own small contribution to this growing number of paradoxes is the discovery of simple triangular forms". Read as overlapping snippets of a lending-only scan; Gardner's figures were not seen, so that his triangle is the 13 by 5 one is inferred from his text |
| S15 | Roger B. Nelsen, *Proofs Without Words: Exercises in Visual Thinking* (MAA, 1993); *Proofs Without Words II: More Exercises in Visual Thinking* (MAA, 2000) | the source map; names to search | front matter of the scans. Nelsen: "PWWs began to appear in Mathematics Magazine about 1975". Pages in the first book (a later printing): sums of integers 69–70, odd integers 71, squares 77, cubes 84–87, geometric sums 118, geometric series 121, Pythagoras 3 |
| S16 | Brian Hayes, "Gauss's Day of Reckoning", *American Scientist* 94(3), May–June 2006, p. 200 | why the story of young Gauss and 1 to 100 is not told | of the earliest account (Sartorius, 1856): "There is no mention of the numbers from 1 to 100, or any other specific arithmetic progression"; "the 1-100 series makes its first appearance in 1938" |
| S17 | Aristotle, *Physics* III.4, 203a | "gnomon" in the lower-third of 03_5 | "If the gnomons are placed round the one, and without the one, in the one construction the figure that results is always different, in the other it is always the same" (Internet Classics Archive). The odd-number reading is Heath's and the standard one; Aristotle states no formula |

**Never said or shown** (each was a lead that the sources did not bear out):
that Nicomachus stated or proved the sum-of-cubes rule; that Aryabhata was
certainly first; that Golomb's figure cuts squares in half, or that there
is a paper by Warren Lushbaugh; that Archimedes "summed an infinite
series"; that Euclid wrote the formula in letters or cut a cube; that the
two arrangements are "Pythagoras's own proof" or "a Chinese proof from 200
BC"; that Curry invented the 13 by 5 triangle, published in 1953, or was a
mathematician; that Gauss added 1 to 100; that Page invented the halved
square; that Moser proved or named the circle problem.

## Terms (the jargon ledger)

| Term | First said or shown | As spoken | On screen |
|---|---|---|---|
| region | 02_4 | "join every pair with a straight line. One point, one region" | the tinted pieces of the disc |
| layer (L-shaped) | 03_1 | "wrap an L-shaped layer around it" | lower-third at 03_5: "gnomon" |
| the card | 03_6 | "what the picture showed, and what it took for granted" | **Shown** / **Assumed** |
| staircase | 04_1 | "Build a staircase of tiles. One, two, three, four, five" | lower-third at 04_5: "triangular numbers" |
| slab | 01_1, 05_1 | "A cube five tiles on a side is five slabs, each five by five" | the slices of a cube |
| arm, corner | 03_4, 05_2 | "one more at the end of each arm"; "The single tile is the corner" | the two straight parts of a layer and the square where they meet |
| stack | 07_1 | "Stack them, biggest at the bottom, every layer pushed into the same corner" | Siu's solid |
| induction | 09_4 (screen only) | not spoken | lower-third: "the name for this kind of reason: induction" |
| geometric series | 10_5 (screen only) | not spoken | lower-third |
| block | 12_4 | "How many blocks does the cube fall into" | the parts of the cut cube |
| Pascal's triangle | 12_7 (screen only) | not spoken | lower-third |
| long side, shorter sides | 13_1 | "A right triangle with shorter sides three and four" | hypotenuse and legs are never said |

## Simplification → exact

| Line | Said | Exact |
|---|---|---|
| 02_6 | "Thirty-one." | 31 regions when no three lines pass through one point inside the circle; six evenly spaced points give 30 (`F-C.three_through_one.regions`). On screen in the lower-third |
| 03_5 | "it would still work after a hundred layers" | for every whole number n ≥ 1, 1 + 3 + … + (2n − 1) = n² |
| 06_4 | "The cubes up to any number make a square whose side is the staircase up to that number." | for every whole number n ≥ 1, 1³ + … + n³ = (1 + … + n)² = (n(n + 1)/2)² |
| 06_5 | "Al-Karaji proved the rule with L-shaped layers around the year one thousand." | Heath: "a proof … by means of a figure with gnomons drawn in it"; "tenth-eleventh century" |
| 08_4 | "one stack, the sum of the squares, is a third of the box" | 1² + … + n² = ⅓ n (n + 1)(n + ½) = n(n + 1)(2n + 1)/6, for every whole number n ≥ 1. On screen in the lower-third |
| 09_4 | "That covers every height." | induction on the number of layers: the solid of height n + 1 is the solid of height n lifted by one cube with three new n + 1 by n + 1 slabs, under, behind and beside (`model::stacks::grows`); height 1 is three cubes |
| 10_5 | "this endless sum equals one" | the partial sums 1 − 1/2ⁿ are all below 1 and exceed every number below 1. On screen in the lower-third |
| 11_5 | "Each colour holds one third." | after n cuts each colour holds (1 − 1/4ⁿ)/3; the limit is 1/3. On screen in the lower-third |
| 11_6 | "Archimedes added quarters of quarters like these" | Proposition 23 is a finite identity with a remainder, A + … + Z + ⅓Z = (4/3)A; he does not sum an infinite series (S11) |
| 12_6 | "nothing depended on three and two" | (a + b)³ = a³ + 3a²b + 3ab² + b³ for all positive lengths a and b; the video slides the cut through whole and half values |
| 12_7 | "every block of the cube appears twice" | (a + b)⁴ = a·(a + b)³ + b·(a + b)³; the count with j short sides is C(4, j) = C(3, j) + C(3, j − 1) |
| 13_6 | "It holds for every right triangle." | for all positive legs a and b: the tilted shape has four sides of length √(a² + b²) and four right angles, and its area is (a + b)² − 2ab |
| 14_6 | "never more than an eighth of a square thick" | the two bent edges are 1/8 apart, measured straight up, from x = 5 to x = 8, and closer elsewhere (`F-L.sliver.thickest`); each bend is 1/13 off the straight line (`F-L.dip`, `F-L.bulge`) |
| 14_7 | "The trick goes back to Paul Curry … in nineteen fifty-three. Martin Gardner found this triangle form" | Gardner (1956): Curry "conceived the brilliant notion" in 1953, shown as a rectangle; "My own small contribution … is the discovery of simple triangular forms" (S14) |

## Misconceptions

| Chapter | The wrong belief | The beat that defuses it |
|---|---|---|
| 02 | a pattern that holds five times is as good as proved | 02_6: 31, not 32 |
| 03 | a picture of five layers is only about five layers | 03_4: every layer is two longer than the last |
| 05 | cutting a slab is a fudge | 05_5: an even cube's arm is a whole number of slabs and a half |
| 07 | three stacks might be mirror images | 07_3 and 09_5: turned, never mirrored (`F-S.turns`) |
| 08 | the half in n + ½ is a trick of algebra | 08_2: a layer of cubes sliced flat |
| 09 | a picture drawn at four layers proves it for four layers | 09_2: three new slabs take any height to the next |
| 10 | "equals one" means "gets close to one" | 10_5: no number below one is big enough |
| 13 | the rearrangement proves it by itself | 13_7: the tilted shape must be a square and the edges straight |
| 14 | pieces that look as if they fit, fit | 14_5: two in five against three in eight |
| 15 | a program's check is a proof | 15_5: "That is still only checking." |

## Facts

Each table: the key, its value as `proofs facts` prints it, and what it is.

### F-C: the pattern, and the circle that breaks one (chapter 02)

**The drawn points.** Seven points of the unit circle, in the order the
chapter adds them, each from a parameter p/d as ((d² − p²)/(d² + p²),
2pd/(d² + p²)): p/d = 12/11, −19/6, 3/17, 17/3, −5/13, −8/7, 4/7. The video
uses the first six; the seventh is for `F-C.regions.seven`. **Regions** are
counted from the drawing: every pair of points joined; corners = the points
plus the distinct crossings inside; edges = the arcs plus each line's
stretches between corners; regions = edges − corners + 1. **Three through
one point:** the six points ±(3/5, 4/5), ±(5/13, 12/13), ±(15/17, 8/17),
whose three long lines all pass through the centre.

| key | value | what it is |
|---|---|---|
| `F-C.cube_totals` | `1,9,36,100,225` | 1³, 1³ + 2³, …, up to 5³: the running totals of 02_1 |
| `F-C.cube_roots` | `1,3,6,10,15` | the whole number whose square each total is |
| `F-C.regions` | `1,2,4,8,16,31` | regions made by the first 1, 2, …, 6 drawn points |
| `F-C.doubling` | `1,2,4,8,16,32` | what doubling predicts for 1 to 6 points: 2^(k − 1) |
| `F-C.regions.seven` | `57` | regions made by all seven drawn points |
| `F-C.lines` | `15` | lines joining the six points (pairs of points) |
| `F-C.crossings` | `15` | distinct crossings inside the circle for the six drawn points |
| `F-C.fours` | `15` | sets of four among six points; equal to the crossings exactly when no three lines meet at one inside point |
| `F-C.closed_form` | `1,2,4,8,16,31,57` | 1 + C(n, 2) + C(n, 4) for n = 1 to 7 |
| `F-C.three_through_one.crossings` | `13` | distinct crossings when three lines pass through one point |
| `F-C.three_through_one.regions` | `30` | regions in that case: one fewer than 31 |

### F-O: L-shaped layers (chapter 03)

**Layer k** (from 1) is the unit cells (x, y), 0 ≤ x, y < k, with
max(x, y) = k − 1.

| key | value | what it is |
|---|---|---|
| `F-O.layers` | `1,3,5,7,9` | cells in layers 1 to 5 |
| `F-O.totals` | `1,4,9,16,25` | cells in the first 1, 2, …, 5 layers together |
| `F-O.sum` | `25` | cells in the five layers |
| `F-O.square` | `5x5` | the square the five layers make |
| `F-O.next` | `11` | cells in the sixth layer (the quiz) |
| `F-O.tiled.through` | `60` | reach of the slow check: layers 1 to n tile the n by n square, and layer n has 2n − 1 cells |
| `F-O.sum.through` | `1000` | reach of the whole-number check: 1 + 3 + … + (2n − 1) = n² |
| `F-O.hundredth` | `199` | the hundredth odd number |
| `F-O.sum.hundred` | `10000` | the first hundred odd numbers added |

### F-T: two staircases (chapter 04)

**The staircase** of n columns: column x (from 0) holds the cells (x, 0) to
(x, x). **The copy** is the same cells turned half round: (x, y) goes to
(n − 1 − x, n − y). Together they tile the rectangle n wide and n + 1 tall.

| key | value | what it is |
|---|---|---|
| `F-T.columns` | `1,2,3,4,5` | cells in each column of the five-column staircase |
| `F-T.count` | `15` | cells in the staircase |
| `F-T.rectangle` | `5x6` | the rectangle two staircases make, wide by tall |
| `F-T.rectangle.cells` | `30` | cells in the two staircases together |
| `F-T.counts` | `1,3,6,10,15` | staircases of 1 to 5 columns |
| `F-T.tiled.through` | `60` | reach of the slow check: the two staircases tile the n by n + 1 rectangle |
| `F-T.sum.through` | `1000` | reach of the whole-number check: 2(1 + 2 + … + n) = n(n + 1) |
| `F-T.count.hundred` | `5050` | 1 + 2 + … + 100 |

### F-K: the cubes' slabs (chapters 01, 05, 06)

**Layer k** is cube k's k slabs, each k by k, laid round the square of side
s = 1 + 2 + … + (k − 1). One slab is the corner, at (s, s). The upright arm
is the cells with s ≤ x < s + k, 0 ≤ y < s; the top arm is its mirror image
in the diagonal. An odd cube puts (k − 1)/2 whole slabs in each arm. An even
cube puts (k − 2)/2 whole slabs in each arm and cuts one slab into two
halves, k by k/2: one at the foot of the upright arm (y from 0), the other,
turned a quarter turn, at the far end of the top arm (x from 0).

| key | value | what it is |
|---|---|---|
| `F-K.cubes` | `1,8,27,64,125` | unit cubes in cubes 1 to 5 |
| `F-K.sides` | `1,3,6,10,15` | side of the square after 1, 2, …, 5 layers |
| `F-K.side` | `15` | side of the finished square |
| `F-K.unit_squares` | `225` | unit squares in all the pieces |
| `F-K.slabs` | `15` | distinct slabs among the pieces |
| `F-K.cut_slabs` | `2` | slabs that are cut in two (the two cube's and the four cube's) |
| `F-K.pieces` | `17` | pieces on the board: 13 whole slabs and 4 halves |
| `F-K.layer_areas` | `1,8,27,64,125` | unit squares in each layer: the cubes |
| `F-K.arm.4` | `4x6` | each arm of the four cube's layer: wide by long |
| `F-K.arm.4.slabs` | `3/2` | how many 4 by 4 slabs fill one such arm |
| `F-K.arm.5` | `5x10` | each arm of the five cube's layer |
| `F-K.arm.5.slabs` | `2` | how many 5 by 5 slabs fill one such arm |
| `F-K.sixth.cube` | `216` | unit squares in the sixth layer: 6³ |
| `F-K.sixth.side` | `21` | side of the square after six layers |
| `F-K.arm.6` | `6x15` | each arm of the sixth layer |
| `F-K.arm.6.slabs` | `5/2` | how many 6 by 6 slabs fill one such arm |
| `F-K.odd_runs` | `1|3,5|7,9,11|13,15,17,19|21,23,25,27,29` | Nicomachus's runs: the k consecutive odd numbers, from k(k − 1) + 1, that add up to k³, for k = 1 to 5 (runs separated by a bar) |
| `F-K.tiled.through` | `40` | reach of the slow check: the pieces of layers 1 to n tile the square of side 1 + … + n, and layer n has n³ unit squares |
| `F-K.sum.through` | `1000` | reach of the whole-number check: 1³ + … + n³ = (1 + … + n)² |

**The next size** (06_3): each arm of layer k is k wide and k(k − 1)/2
long, which is k − 1 half slabs; two arms take k − 1 slabs and the corner
takes one.

### F-S: three stacks and a box (chapters 07, 08, 09)

**One stack** of n layers: layer l (from 0, the bottom) is a square of side
n − l, every layer in the same corner: cells (i, j, l) with 0 ≤ i, j < n − l.
**The box** has x from 0 to n − 1 (wide), y from 0 to n (deep) and z from 0
(tall). In it the three stacks are the cells
- stack 0: x < n − z and y < n − z, for 0 ≤ z < n;
- stack 1: 1 ≤ y ≤ n, x < y, z ≥ n − y, z < n;
- stack 2: y ≤ x < n, n − x ≤ z ≤ n.

They fill every cell with z < n, and at z = n the cells with y ≤ x: a
staircase of 1 + 2 + … + n cubes, all from stack 2. **The slice:** each of
those top cubes is cut at half height; its upper half goes to the column
(n − 1 − x, n − y), at the lower half's height. **The next size:** each
stack at height n is its own base slab (side n) plus the same stack at
height n − 1 lifted by one cube; the three base slabs lie at z = 0 (under),
at y = n (behind) and at x = n − 1 (beside).

| key | value | what it is |
|---|---|---|
| `F-S.layers` | `16,9,4,1` | cubes in the layers of a four-layer stack, bottom first |
| `F-S.stack` | `30` | cubes in one stack |
| `F-S.three_stacks` | `90` | cubes in three stacks |
| `F-S.block` | `4x5x4` | the locked block before the slice: wide by deep by the number of full layers |
| `F-S.box` | `4x5x9/2` | the box after the slice: wide by deep by tall |
| `F-S.box.volume` | `90` | its volume |
| `F-S.top` | `10` | cubes that stick out above the full layers |
| `F-S.top.halves` | `20` | half cubes after the slice |
| `F-S.top.layer` | `20` | squares on the block's top face |
| `F-S.sums` | `1,5,14,30,55` | 1², 1² + 2², …, up to 5² |
| `F-S.turns` | `1,1,1` | determinant of each stack's turn: 1 means turned, −1 would mean mirrored |
| `F-S.filled.through` | `24` | reach of the slow check: the three stacks, sliced, fill the box n by n + 1 by n + ½ exactly once (counted in half-height cells) |
| `F-S.grown.box` | `5x6x5` | the block of five-layer stacks before its slice |
| `F-S.grown.top` | `15` | cubes in its top step |
| `F-S.grown.slab` | `25` | cubes in each new base slab |
| `F-S.grows.through` | `24` | reach of the slow check of the next size: each stack at height n is its new base slab plus itself at height n − 1, lifted by one |
| `F-S.one.cubes` | `3` | cubes in three stacks of one layer |
| `F-S.one.box` | `1x2x3/2` | their box |
| `F-S.sum.through` | `1000` | reach of the whole-number check: 6(1² + … + n²) = n(n + 1)(2n + 1) |

### F-H: a square halved again and again (chapter 10)

**Piece k** (from 1) is half of what is left of the unit square: odd steps
keep the left half of what is left, even steps the bottom half, so what is
left shrinks into the top right corner.

| key | value | what it is |
|---|---|---|
| `F-H.pieces` | `1/2,1/4,1/8,1/16,1/32` | areas of pieces 1 to 5 |
| `F-H.filled` | `1/2,3/4,7/8,15/16,31/32` | area filled after 1, 2, …, 5 pieces |
| `F-H.gap` | `1/32` | what is left after five pieces |
| `F-H.pieces_to.thousandth` | `10` | the first number of pieces after which the gap is under 1/1000 |
| `F-H.gap_at.thousandth` | `1/1024` | the gap then |
| `F-H.pieces_to.millionth` | `20` | the first number of pieces after which the gap is under 1/1,000,000 |
| `F-H.gap_at.millionth` | `1/1048576` | the gap then |
| `F-H.checked.through` | `40` | reach of the exact check: after n pieces, filled + gap = 1, the gap equals the last piece, and filled < 1 |

### F-R: a square quartered again and again (chapter 11)

**Level j** (from 1): the corner square still uncut, of side 1/2^(j − 1),
with its lower left corner at (1 − 2s, 1 − 2s) where s = 1/2^j, is cut into
four squares of side s. The lower left one is the counted colour, the upper
left and lower right ones are the other two colours, and the upper right
one is cut again.

| key | value | what it is |
|---|---|---|
| `F-R.pieces` | `1/4,1/16,1/64` | area of the counted square at levels 1 to 3 |
| `F-R.share` | `1/4,5/16,21/64` | the counted colour's total after 1, 2 and 3 levels |
| `F-R.left` | `1/4,1/16,1/64` | the uncut corner after 1, 2 and 3 levels |
| `F-R.shares` | `3` | equal squares kept at each level: the three colours |
| `F-R.limit` | `1/3` | what each colour's total tends to |
| `F-R.short` | `1/192` | how far the counted colour is from 1/3 after three levels: a third of the uncut corner (Archimedes's remainder, S11) |
| `F-R.checked.through` | `20` | reach of the exact check: at level n the three squares are equal, three shares plus the corner are 1, and a share is short of 1/3 by a third of the corner |

### F-B: a square, a cube and a fourth power, cut (chapter 12)

**The cut:** every side of a cube of side a + b in d dimensions is cut once,
at a. Each of the 2^d blocks has, along each side, the long part a or the
short part b; a block's **kind** is how many of its sides are short. The
video's numbers: a = 3, b = 2.

| key | value | what it is |
|---|---|---|
| `F-B.parts` | `3,2` | the long part and the short part |
| `F-B.square.blocks` | `4` | pieces of the cut square |
| `F-B.square.kinds` | `1,2,1` | how many pieces have 0, 1, 2 short sides |
| `F-B.square.each` | `9,6,4` | area of one piece of each kind |
| `F-B.square.by_kind` | `9,12,4` | area of each kind together |
| `F-B.square.total` | `25` | all the pieces |
| `F-B.square.power` | `25` | (3 + 2)² |
| `F-B.cube.blocks` | `8` | blocks of the cut cube |
| `F-B.cube.kinds` | `1,3,3,1` | how many blocks have 0, 1, 2, 3 short sides |
| `F-B.cube.each` | `27,18,12,8` | volume of one block of each kind |
| `F-B.cube.by_kind` | `27,54,36,8` | volume of each kind together |
| `F-B.cube.total` | `125` | all the blocks |
| `F-B.cube.power` | `125` | (3 + 2)³ |
| `F-B.fourth.blocks` | `16` | blocks of the cut fourth power |
| `F-B.fourth.kinds` | `1,4,6,4,1` | how many have 0 to 4 short sides |
| `F-B.fourth.each` | `81,54,36,24,16` | four-dimensional volume of one block of each kind |
| `F-B.fourth.by_kind` | `81,216,216,96,16` | volume of each kind together |
| `F-B.fourth.total` | `625` | all the blocks |
| `F-B.fourth.power` | `625` | (3 + 2)⁴ |
| `F-B.fourth.from_cube` | `1+3,3+3,3+1` | the counts 4, 6, 4 as sums of neighbours in 1, 3, 3, 1 |
| `F-B.filled.cuts` | `100` | cuts rebuilt cell by cell and found to fill their cube exactly: dimensions 1 to 4, a and b from 1 to 5 |

### F-P: four triangles, two ways (chapter 13)

**First arrangement**, in the square with corners (0, 0) and (a + b,
a + b): triangles with their right angles at the four corners, (0,0) (a,0)
(0,b); (s,0) (s,a) (a,0); (s,s) (b,s) (s,a); (0,s) (0,b) (b,s), where
s = a + b. The **tilted shape** is (a,0) (s,a) (b,s) (0,b). **The slides**
to the second arrangement: the first triangle stays; the second moves by
(0, b), the third by (−b, −a), the fourth by (a, 0). What is left is the
square [0, a] × [b, s] and the square [a, s] × [0, b].

| key | value | what it is |
|---|---|---|
| `F-P.legs` | `3,4` | the two shorter sides |
| `F-P.big` | `49` | the big square, in tiles |
| `F-P.triangle` | `6` | one triangle |
| `F-P.triangles` | `24` | the four triangles |
| `F-P.tilted` | `25` | the tilted square (shoelace area of its four corners) |
| `F-P.squares` | `9,16` | the two squares left by the second arrangement |
| `F-P.long_side` | `5` | the whole number whose square is the tilted square |
| `F-P.slides` | `0 0,0 4,-4 -3,3 0` | how far each triangle slides, x then y, triangle by triangle |
| `F-P.other.legs` | `2,5` | a second pair of shorter sides, shown at 13_6 |
| `F-P.other.tilted` | `29` | its tilted square |
| `F-P.other.whole_side` | `none` | the whole number whose square that is, if any |
| `F-P.checked.through` | `60` | reach of the exact check, for all pairs of shorter sides up to this: the tilted shape has four equal sides and four right angles, its area is a² + b², the slid triangles pair into two a by b rectangles, and those with the two squares tile the big square |

### F-L: the picture that lies (chapter 14)

**The pieces** on the unit grid, first arrangement: the big triangle (8,0)
(8,3) (0,0); the small triangle (13,3) (13,5) (8,3); the L of 7 squares:
row y = 0, x from 8 to 12, and row y = 1, x = 8 and 9; the L of 8 squares:
row y = 1, x from 10 to 12, and row y = 2, x from 8 to 12. **The slides** to
the second arrangement: big triangle (5, 2), small triangle (−8, −3), the L
of 8 (0, −1), the L of 7 (−3, 0). **The outlines** are four-cornered:
(0,0) (13,0) (13,5) and the bend, which is (8, 3) in the first arrangement
and (5, 2) in the second.

| key | value | what it is |
|---|---|---|
| `F-L.pieces` | `12,5,8,7` | areas of the big triangle, the small triangle, the L of 8 and the L of 7 |
| `F-L.pieces.total` | `32` | the four pieces together |
| `F-L.frame` | `13x5` | the triangle the pieces seem to fill, wide by tall |
| `F-L.true_triangle` | `65/2` | the area a true triangle of that size has |
| `F-L.slopes` | `3/8,2/5` | rise over run of the big triangle's long side, then the small one's |
| `F-L.straight_slope` | `5/13` | rise over run of a straight edge from corner to corner |
| `F-L.outline.first` | `32` | area inside the first arrangement's outline |
| `F-L.outline.second` | `33` | area inside the second arrangement's outline |
| `F-L.sliver` | `1` | area between the two outlines |
| `F-L.sliver.thickest` | `1/8` | the most the two bent edges are apart, measured straight up |
| `F-L.holes.first` | `0` | empty unit squares in the first arrangement |
| `F-L.holes.second` | `1` | empty unit squares in the second |
| `F-L.hole.at` | `7 1` | the empty square's lower left corner, x then y |
| `F-L.dip` | `1/13` | how far the first bend is below the straight edge |
| `F-L.bulge` | `1/13` | how far the second bend is above it |
| `F-L.slides` | `5 2,-8 -3,0 -1,-3 0` | how far each piece slides, x then y: big triangle, small triangle, L of 8, L of 7 |
| `F-L.slopes.thousandths` | `375,400` | the two slopes in thousandths: 0.375 and 0.400 |
| `F-L.bends.on_the_edge` | `0` | how many of the two bends lie on the straight edge |

### F-Q: the quizzes

Every countdown in the storyboard is one of these (a test in `main.rs`).

| key | value | what it is |
|---|---|---|
| `F-Q.count` | `9` | quizzes with a countdown |
| `F-Q.names` | `regions,next_layer,rectangle,cut_slabs,stacks,share,kinds,tilted,slopes` | their names, in order |
| `F-Q.regions.line` | `02_5` | the storyboard line that asks: "How many regions will six points make?" |
| `F-Q.regions.seconds` | `4` | its countdown |
| `F-Q.regions.answer` | `31` | regions for the six drawn points |
| `F-Q.next_layer.line` | `03_3` | "How many tiles will the next layer take?" |
| `F-Q.next_layer.seconds` | `4` | its countdown |
| `F-Q.next_layer.answer` | `11` | cells in the sixth layer |
| `F-Q.rectangle.line` | `04_3` | "How wide is it, and how tall?" |
| `F-Q.rectangle.seconds` | `5` | its countdown |
| `F-Q.rectangle.answer` | `5x6` | the rectangle of two five-column staircases |
| `F-Q.cut_slabs.line` | `05_4` | "The four cube has four slabs. How many of them will have to be cut?" |
| `F-Q.cut_slabs.seconds` | `6` | its countdown |
| `F-Q.cut_slabs.answer` | `1` | slabs of the four cube that are cut |
| `F-Q.stacks.line` | `07_2` | "How many of these stacks does it take to build a solid box?" |
| `F-Q.stacks.seconds` | `5` | its countdown |
| `F-Q.stacks.answer` | `3` | stacks in Siu's box |
| `F-Q.share.line` | `11_3` | "What fraction of the whole square is coloured?" |
| `F-Q.share.seconds` | `6` | its countdown |
| `F-Q.share.answer` | `1/3` | the limit of one colour's share |
| `F-Q.kinds.line` | `12_4` | "How many blocks does the cube fall into, and how many of each kind?" |
| `F-Q.kinds.seconds` | `6` | its countdown |
| `F-Q.kinds.answer` | `1,3,3,1` | blocks of the cut cube by number of short sides |
| `F-Q.tilted.line` | `13_4` | "So how big was the tilted square?" |
| `F-Q.tilted.seconds` | `6` | its countdown |
| `F-Q.tilted.answer` | `25` | the tilted square on shorter sides 3 and 4, in tiles |
| `F-Q.slopes.line` | `14_4` | "Which triangle is steeper, the big one or the small one?" |
| `F-Q.slopes.seconds` | `6` | its countdown |
| `F-Q.slopes.answer` | `small` | the triangle with the greater rise over run |

### F-X: cited numbers

Years and pages as the sources above give them; the oracle prints the same
literals, and the two spans are subtractions.

| key | value | what it is |
|---|---|---|
| `F-X.moser` | `1949,109` | S1: year, page |
| `F-X.nicomachus` | `100,II.20.5` | S2: about the year, and the passage |
| `F-X.aryabhata` | `499,22` | S4: year, stanza |
| `F-X.al_karaji` | `1000` | S3: about the year |
| `F-X.fry` | `1985,11` | S5: year, page |
| `F-X.siu` | `1984,92` | S8: year, page |
| `F-X.page` | `1981,201` | S9: year, page |
| `F-X.ajose_nelsen` | `1994,230` | S10: year, page |
| `F-X.euclid` | `II.4` | S12: book and proposition |
| `F-X.curry` | `1953` | S14: the year Gardner gives for Curry's idea |
| `F-X.gardner` | `1956,139,145` | S14: year of the book, and the two pages quoted |
| `F-X.nelsen` | `1993,2000` | S15: the years of the two books |
| `F-X.al_karaji.after_nicomachus` | `900` | years from Nicomachus to al-Karaji, as round dates: the "nine centuries" of 06_5 |
| `F-X.gardner.after_curry` | `3` | years from Curry's idea to Gardner's book: the "three years later" of 14_7 |

## Always and never (edge tests)

Every sentence of the script that says always, every, any, never or
nothing, with the test that tries its edges (`src/facts.rs` and
`src/model.rs`; the script's own list is `docs/claims.md` once P3 lands).

| Line | Sentence | Edge tried | Test |
|---|---|---|---|
| 03_4 | "they always close up into a square" | n = 1 (one tile, no arms) to 60 | `l_shaped_layers_build_every_square` |
| 04_5 | "Two staircases always make a rectangle one taller than it is wide." | n = 1 (two tiles, 1 by 2) to 60 | `two_staircases_make_a_rectangle` |
| 06_3 | "the arms always take one slab fewer than the cube has" | k = 1 (no arms), k = 2 (the first cut), odd and even to 40 | `the_cubes_slabs_tile_the_square`, `the_edges_of_every_always` |
| 06_4 | "every cube fits, at every size" | as above; the sum to n = 1000 | `F-K.tiled.through`, `F-K.sum.through` |
| 06_5 | "every cube is a run of odd numbers" | k = 1 to 200, and that the runs use every odd number once | `each_cube_is_a_run_of_consecutive_odd_numbers` |
| 07_3 | (shown) the stacks are turned, never mirrored | the determinant of each turn | `three_stacks_fill_siu_s_box` |
| 08_4 | "At any height the box is …" | n = 1 (three cubes) to 24 | `three_stacks_fill_siu_s_box` |
| 09_3 | "nothing overlaps" | the three new slabs and the lifted solid, n = 1 to 24 | `each_height_is_the_last_one_on_three_new_slabs` |
| 09_4 | "That covers every height." | the step for n = 1 to 24, and height 1 | `F-S.grows.through`, `F-S.one.box` |
| 10_4 | "Not after any number of steps." | every partial sum to 60 pieces is below 1 | `the_edges_of_every_always` |
| 10_5 | "no number smaller than one is big enough" | the gap after n pieces is exactly 1/2ⁿ, to 40 | `F-H.checked.through` |
| 11_4 | "At every size there are three equal squares" | levels 1 to 20 | `F-R.checked.through` |
| 12_6 | "nothing depended on three and two … it is still eight blocks" | a and b from 1 to 5, including a = b and a < b | `the_cut_cube_s_blocks_fill_it_in_every_dimension` |
| 12_7 | "every block of the cube appears twice" | the 16 blocks counted by kind against two shifted copies of 1, 3, 3, 1 | `F-B.fourth.from_cube` |
| 13_3, 14_2 | "Nothing turns, nothing flips." | each slide is one translation, the same for every corner of the piece | `four_triangles_two_ways`, `the_lying_picture_is_not_a_triangle` |
| 13_6 | "It holds for every right triangle." | equal sides (1, 1), (7, 7); far apart (1, 60), (60, 1); all pairs to 60 | `the_edges_of_every_always`, `F-P.checked.through` |
| 14_6 | "never more than an eighth of a square thick" | the gap at every whole x from 0 to 13 | `F-L.sliver.thickest`, `the_lying_picture_is_not_a_triangle` |
| 14_7 | "Neither picture was ever a triangle." | neither bend lies on the straight edge | `F-L.bends.on_the_edge` |
| 02_7 | "Five checks in a row prove nothing." | the doubling holds for 1 to 5 drawn points and fails at 6 and 7 | `the_edges_of_every_always` |
