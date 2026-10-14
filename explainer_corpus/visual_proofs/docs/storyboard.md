# Storyboard — Proofs You Can See

The hand-edited source of this video's words and pictures. `python3 -I
tools/storyboard.py visual_proofs` checks it (gate G2) and writes
`script.tsv` from the words column. Facts: `docs/fact-sheet.md` and
`oracle.py` (every number below names its fact key); brief: `docs/brief.md`;
reviews and their dispositions: `docs/reviews/visual_proofs/` (R1.md: every
finding, and where each fix landed).

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4), and no
single letters (n, a and b live on screen with word labels). `*(…)*` is an
annotation, never spoken. `⟨predict⟩` opens a question for the viewer; the
beat ends with the question and the picture holds before the next beat
answers it.

**Two kinds of question, two looks.**

- **Your turn** (a quiz): a `⟨predict⟩` beat whose last column says
  `listen N s (countdown)`. A card with the question in `ink`; a ring that
  drains over N seconds with the seconds left in its middle; the line "pause
  for more time" under it; the narration silent and the music alone, one
  mallet note a second. At zero the answer lands in gold where the ring was.
  The next beat then works it on the same picture. Every quiz is a row of
  `model::QUIZZES`, and its numbers are the `F-Q.<name>.*` facts. A quiz the
  viewer cannot derive (02_5, 07_2) says "guess" in its words.
- **A question to carry** (01_3, 06_1, 08_1, 09_1, 10_3, 15_2): the question
  as a card with no ring, held about two seconds.

**The card.** Every proof ends on the same two-line card, in the same place:
**Shown** (the exact statement, with its range: "for every whole number n
from 1 …") and **Assumed** (the fit the picture took for granted, with a tick
and the reason once it is checked). It first appears at 03_6, passes seven
times (03_6, 04_6, 06_4, 09_5, 10_5, 11_6, 13_7), says "counted, not built"
once (12_7), and fails once: at 14_7 the Assumed line gets a red cross.
Chapter 15 is that card read back. **Its picture.** The Assumed line is a
close-up of the fit it names, on the same magnifier ring every time (03: the
last tile of an arm beside the square's last row; 04: a column pair; 06: an
arm on a slab ruler; 09: the three turns; 10: a piece against half of what was
left; 11: three twins; 13: the corner where two sharp angles meet), the tick
drawn over the close-up; the sentence is its caption. At 14_7 the same ring
closes on the long edge and shows the bend, and the cross is drawn on the
bend. **The shelf.** When a card has passed, its picture shrinks to a
thumbnail on a shelf along the top edge, left to right; at 14_7 the card
refuses to file and leaves a gap shaped like the sliver; 15_3's small pictures
are the shelf. Ticks are `ink`, crosses `warn`, and `warn` is never the only
carrier of "wrong" (a cross, or a hatch).

**The cubes.** Every solid is made of unit cubes drawn by the scene itself:
a fixed viewpoint, each cube's three visible faces in three shades of its
colour, painted back to front. Solids move by sliding and by quarter and
half turns; the camera never moves. A flat picture (the board) sits beside
or under a solid, never in perspective. A cut is a seam line, never a plane
through a solid, and a ghost is an outline.

**The next size.** Each proof is shown at one size and then asked the same
question: would the next size fit? The answer is always a picture of the
step from one size to the next (a new layer, a new column, three new slabs,
the same square again at a smaller scale, a cut that slides). The lie has
no such step.

**Model numbers.** There are none: every number in this video is a count of
pieces or an exact fraction. The words "checked by code up to …" are on
screen wherever a reach is quoted (15_5).

## 2 The arc

| Act | Chapters | Spine |
|---|---|---|
| I. Why a picture | 01 cold open, 02 five for five | the cubes' slabs close into a square; the same fact in numbers, five times; a pattern that is right five times and wrong the sixth; so a reason is needed |
| II. Layers | 03 layers, 04 staircases, 05 slabs, 06 every cube | odd numbers as L-shaped layers; two staircases; the cubes laid flat as layers; why the next cube always fits, and that the staircase is what makes it fit |
| III. Copies | 07 three stacks, 08 the box, 09 every height | three stacks lock; the half layer; why the next height always fits: three new slabs, the L-shaped layer in three dimensions |
| IV. Every scale, every length | 10 halves, 11 a third, 12 the cut cube | a sum that never ends; three colours with equal shares at every scale; a square, a cube and a fourth power cut once along each side |
| V. A picture can lie | 13 four triangles, 14 the missing square, 15 what makes a proof | the same four triangles two ways; the same argument giving nonsense; the card read back: every piece, every fit, every size |

The payoff is previewed twice before any reason is given: the cold open's
silent fit (01_2, 01_3) and the first complete proof (03). The video ends on
the rigorous form of the opening picture: the cubes' layer for a size nobody
drew, with every length named (15_4). The five-colour square is not off
screen for long: 06_5 recolours it into fifteen thin layers, 09 to 10 hands
its outline to the unit square, the shelf carries its thumbnail through 03 to
14, and 15_6 un-dims it.

**Dependence.** The proofs lean on each other in the order shown: the
layers of 03 are the shape of 05; the staircase of 04 is the side of 05's
square and the reason of 06 (the arm is the staircase's pieces paired); two
copies in 04 become three in 07, and the empty half of 08_2 is 04's second
staircase; the L-shaped layer returns in three dimensions in 09 and as the cut
square in 12; the two-arrangement argument of 13 is the one that fails in 14.

**Length.** The G2 estimate below is 16.8 minutes over 15 chapters, over the
order's 12 to 16 by the words R1 added to give each proof its reason. If the owner wants it shorter, chapter 10 (the
halved square, about 55 seconds) folds into one beat of chapter 11, and 12_7
(the fourth power, about 20 seconds) goes to the description.

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses
a data colour.

| Idea | Color |
|---|---|
| size 1: the first layer, the one cube | pink `#EA87B3` |
| size 2: the second layer, the two cube's slabs | copper `#CD853F` |
| size 3 | green `#83C167` |
| size 4 | blue `#58C4DD` |
| size 5 | violet `#994DE5` |
| the staircases of 04: column k is size k, so the 15 is the five cubes' sides end to end; the copy is the same five colours, turned, in an ink outline | the five size colours |
| the squares on the sides of the right triangle in 13: 3, 4 and 5 are cube 3's, 4's and 5's slabs; the legs 2 and 5 of 13_6 are copper and violet, and the 29 is `ink` | green, blue, violet |
| a copy or a share, where nothing is being sized: first, second, third (07 stacks and 11 shares, with a legend chip the first time); in 10 the pieces are blue, the newest gold; in 14 the L of 8 is blue and the L of 7 pink | blue, copper, pink |
| a block of the cut square, cube or fourth power, by how many of its sides are short (12, with a legend chip): none violet, one blue, two green, three copper, four pink, so a bigger block is higher on the ramp | the size ramp, from the top |
| the piece moving now, an answer as it lands, the next size (the sixth layer, the fifth slabs), a gold apex dot on the stack that is moving | `hero` gold `#FFD866`, never as a resting fill |
| what does not fit: the wrong guess, the bent edge, the sliver, the hole, a failed tick (with a cross or a hatch, never colour alone) | `warn` red `#FC6255` |
| pieces that only fill: the four triangles (13, 14), a ghost of the last size, a ghost that carries a question (an outline in `dim`, never `faint`) | `dim` `#8A93A3` |
| texture and grid | `faint` `#3B4352` |
| the quiz card, its ring, the card's text, ticks, outlines | `ink` `#ECEFF4` |

The five sizes run in the order of the owner's picture (red, yellow, green,
blue, purple). Every pair among the five sizes, `hero`, `warn`, `dim` and
`ink` is at least CIEDE2000 21 apart (least 21.6, blue against `dim`); the
first draft's amber was 14.9 from `hero` and its violet 15.4 from `dim`, so
both moved. A test in the crate pins the table before any draft. Under a
deuteranopia simulation copper and green, and `warn` and copper, are close
(7.3 and 2.5), so a size never rests on hue alone (its count is beside it the
first time it appears) and `warn` always carries a cross or a hatch.

## Vocabulary (one word per idea)

**picture** and **proof** (a picture that shows why, and says what it took
for granted); **piece** (anything that moves) and **fit**; **tile** (a unit
square: also in 14, never "square") and **cube** (a unit cube, and the five
cubes); **layer** (the L-shaped layer: one tile wide in 03, and as wide as its
cube from 05 on; the **thin layers** of 06_5 are chapter 03's), with its
**corner** and two **arms**, **row** and **column**; **staircase** (flat);
**stack** (Siu's solid) with its **levels** (never "layers"), and **box**;
**slab** (a flat piece one tile thick: a cube's slice, a slice of the block, a
new base); **block** (a part of the cut cube); **colour** (a share of the
quartered square); **copy**; **pattern** and **check** (the hook); **guess**
(a quiz the viewer cannot derive); **took for granted** (spoken) for the
card's **Assumed**; **your turn** (every quiz); **counted two ways** (the
takeaway, said at 05_7 and 13_5). A triangle's sides are its two **shorter
sides** and its **long side**; a cut block's sides are **short** or **long**.
Never "obviously", "clearly", "trivially", "simply", "it is easy to see". The
formal names appear once each, in a lower-third: gnomon, triangular number,
mathematical induction, geometric series, Pascal's triangle, proof without
words, Pythagoras's theorem.

## Sound plan (for the score bead)

Nothing here is auditioned yet; the drafts of P5 are silent but for the
scratch voice. Foreground cues obey plan §7.3 (at most one per sentence under
speech, 1.5 s apart, at most six in 30 s); anything denser is a bespoke
sonification WAV placed at known frames, not score marks.

- **Countdowns:** one mallet note a second, stepping down, the last three
  always the same. The last note is the tonic pitch over a different chord
  each time, and over the tonic chord only at the last quiz (14_4), as in
  Video F.
- **Pieces landing** (01_2, 05_2 to 05_6, 07_3, 09_2, 13_3): one sonification
  WAV per beat in a music-alone window or at most one chord-bloom per
  sentence at the layer's, stack's or arrangement's last landing, the other
  landings silent; the cut slab gets one soft "split" ping, the one place a
  single sound is information. A finished picture is a finished chord.
  14_2 keeps its idea: three of the four bloom, the fourth is missing.
- **A passed card** closes on its chapter's chord without the three-note
  resolution. The title (01_4) plays the motif's three notes arriving one at a
  time and leaves the chord unsounded; the motif resolves at 15_6 and nowhere
  earlier ("pieces, then the whole").
- **Silence, authored:** `silence:thirty_one` for half a second before
  "Thirty-one." (02_6, the counter's ticks the only sound until 31) and
  `silence:cross` at 14_7 so the red cross lands on nothing, a muted wood
  thud and never a buzzer; the music returns at 15_1 a register lower.
- **Sonification that teaches:** 10_4: each guess passed is one soft note,
  climbing, and the guess of one, which nothing passes, holds the tonic.
  Candidate (the score bead decides): 10_3 to 10_4 as the halving heard as
  rhythm, each note half the bar left, never reaching the downbeat.
- **Identity:** the key and lead instrument are to be distinct from Videos G
  and K, which share this video's F major, 88 BPM and four chords (R1, C10);
  the score bead auditions.

## Build notes (from R1, for the build beads)

- **Solids are unproven at the pin for pieces that re-sort while they move.**
  Run a spike before 07 to 09: three stacks plus three slabs, a quarter turn
  twice, a slide, 4K60, 10 s on this Mac, accept at 10 fps and no mis-ordered
  face on a five-frame mid-turn sheet; fallback, 07 to 09 as whole chapters
  on the camera route with depth-tested planar faces. Resting pictures and
  12 use fair_coin's static-order `Cube`. Choreograph every turn as lift,
  turn clear, slide in; each stack's second and third copies are two quarter
  turns, not one (E1, E7).
- **A piece of many tiles is one convex polygon per slab, never a multi-
  subpath tile soup** (the renderer bins by instance hull); unit grids are
  short per-slab mobjects or the kit's row strips, and counting beats
  recolour strips in place (E2). Thin layers are built thin from the start
  and separated, never `Transform`ed between different subpath counts (E12).
- **Turns are `rotate` or an arc path, never `animate().rotate`**; add a test
  that a turned piece keeps its area at five instants (E3).
- **02_4 to 02_6 need region polygons**: `model::circle::faces`, a walk of
  the planar graph, tested equal to `regions()` for 1 to 7 points and for the
  three-through-one set (E6).
- **Cuts are seam lines and ghosts outlines** (E9). A cut that slides (12_6)
  gives each block a per-axis scale from trackers, or rebuilds its eight
  boxes' faces each frame, and shows lengths to one decimal between the
  stops (E8). Text-per-term colour uses glyph ranges, not `t2c` (E11).
- **Every on-screen string goes into `glyphs.rs` and the S-7 probe is read
  before a chapter is drafted** (`⅓`, `÷`, `\div`, `\cdots`, superscripts,
  the mixed formula-and-words lower-third as two mobjects: E10).
- **Readouts of fractions are pre-set states swapped in as pieces land**, the
  10_3 zoom is capped near 2^10 and the counter carries the rest (E13). A
  finished square has no inset between pieces (E15); the chain of five blocks
  in 09_4 is outline-only stacks, the grid only on the block being explained
  (E16).
- **Facts to add when a beat needs them**: `F-H.pieces_to.tenth` (4) and
  `.hundredth` (7) if 10_4's guesses start at 0.9 and 0.99; per-level
  strips of 11_5 at levels 1 and 2 (1/12, 1/48).

## 01_cold_open (25 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Take five cubes, each a little bigger than the last. Slice them into slabs. | Five cubes in a row, of side 1 to 5, in the five size colours. Each comes apart into its slabs: 1, 2, 3, 4 and 5 of them (`F-K.slabs` in all). No numbers on screen | the cubes, drawn by projection; lagged reveal | |
| 01_2 | Cut two slabs in half, lay everything flat, and the pieces close up into one perfect square. | The slabs glide down onto a board in layer order, cube 1 first, and lock into one square (`F-K.side` on a side); the two slabs that are cut (`F-K.cut_slabs`) split with a bright seam, and the seam stays visible on the four halves in the finished square. Each of the 17 pieces (`F-K.pieces`) lands as one note | morph that keeps identity (each slab keeps its colour from cube to board); seams | listen 2 s (music alone: the pieces' notes) |
| 01_3 | ⟨predict⟩ No gap, and nothing left over. Why would that ever fit? | The finished square holds, then the question sets itself over it | a question card | |
| 01_4 | *(title)* | **Proofs You Can See**, over the square, which dims | title card | listen 3 s (the motif: three notes arriving one at a time) |

- **Cold open** (plan rule 20): the owner's first picture as a silent fit,
  the payoff as a puzzle, then the title. About 20 seconds before the title.
- **Honesty:** the slabs are `model::cubes::square(5)`, piece for piece; the
  two that split are the two the model cuts, and the words say so (R1: the
  first draft hid the cuts until 05_2).

## 02_five_for_five (75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Add the cubes one at a time, and keep the total. | The five cubes small along the top. Each drops its slabs onto a growing tower of totals; each cube's own number shows beside the total as it joins (+ 1, + 8, + 27, + 64, + 125: `F-K.cubes`), and a woodblock tick marks each total: 1, 9, 36, 100, 225 (`F-C.cube_totals`) | count-up from a tracker; lagged reveal | |
| 02_2 | Every total is a perfect square. One, three, six, ten and fifteen, each multiplied by itself. Five checks, and five squares. | Each total turns into its square: 1 × 1, 3 × 3, 6 × 6, 10 × 10, 15 × 15 (`F-C.cube_roots`). Five small ticks | a formula morph for each total; highlight | |
| 02_3 | So is it always true? Checking one case at a time never finishes. And there is a worse problem. A pattern can break. | A sixth slot with a question mark; then the row slides up, small, and the words "a pattern can break" take the stage | a card for the last sentence | |
| 02_4 | Put points on a circle, spread unevenly, and join every pair with a straight line. The lines cut the circle into pieces, and each piece is a region. One point, one region. Two points, two. Three points, four. Four points, eight. Five points, sixteen. | A circle. The points arrive one at a time, unevenly spaced (`model::circle::DRAWN`, no region of the six under 0.9 percent of the disc); every pair is joined; the regions are tinted so they can be counted. A two-row table fills in: points 1 2 3 4 5, regions 1 2 4 8 16 (`F-C.regions`) | a table built from primitives; a live count of regions; lagged reveal | |
| 02_5 | ⟨predict⟩ Your turn. Pause the video if you want more time. Guess before you count. How many regions will six points make? | The sixth point lands; its five lines are not drawn yet. The card: "6 points: ? regions" | your-turn card; the countdown ring | listen 4 s (countdown) |
| 02_6 | Thirty-one. Count them. The doubling pattern said thirty-two, and the pattern was wrong. | The five lines draw. Regions light in two passes, thirty quickly, a pause, then the last, while a counter runs to 31 (`F-Q.regions.answer`). In the table, "32" (`F-C.doubling`) is struck out in red and 31 written beside it. Lower-third, after the counter stops: "precisely: 31 when no three lines cross at one point inside the circle (six evenly spaced points give 30). Leo Moser, 'On the Danger of Induction', 1949" (`F-C.three_through_one.regions`, `F-X.moser`) | live counter; highlight; lower-third | hold |
| 02_7 | Five checks in a row prove nothing about the sixth. We need a reason that does not care where we stopped. Such a reason is called a proof, and the best ones you can see. | The cubes' row comes back under the circle's row: both read "five for five". The circle's sixth entry is red; the cubes' sixth is still a question mark. Small tag, bottom right: "for programmers: five passing tests are not a proof either" | a table; a labelled aside | |

- **The obvious approach fails on honest data** (plan rule 2): the circle's
  counts are computed from the drawn points by Euler's formula on the exact
  crossings, not from the closed form (`model::circle::regions`).
- **Edge kept on screen and in the words:** 31 needs the points spread so
  that no three lines meet at one point inside the circle; 02_4 says
  "spread unevenly", the lower-third gives the other way the pattern breaks.
  The description keeps the formula 1 + C(n, 2) + C(n, 4) (1, 2, 4, 8, 16, 31,
  57, 99) and "evenly spaced: 6 points give 30, 8 give 88".
- **Misconception defused:** "a pattern that holds five times is as good as
  proved".

## 03_layers (70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Start small. One square tile. To make the next bigger square, wrap an L-shaped layer around it. That takes three more tiles. | One pink tile. A copper L of three tiles slides in round its corner: a 2 by 2 square. The L's two straight parts are labelled "arm" and the tile at the bend "corner" | a grid; lagged reveal | |
| 03_2 | Wrap it again, and the layer takes five. Then seven. Then nine. | Layers three, four and five in green, blue and violet, each with its count beside it: 1, 3, 5, 7, 9 (`F-O.layers`). A 5 by 5 square (`F-O.square`) | lagged reveal; a live count of tiles | |
| 03_3 | ⟨predict⟩ Your turn. The square is five by five. How many tiles will the next layer take? | The square with the outline in `dim` of a sixth layer. The card: "5 by 5: next layer ? tiles" | your-turn card; the countdown ring | listen 4 s (countdown) |
| 03_4 | Eleven. One tile at the end of every row, one on top of every column, and one for the corner. Each new square has one more row and one more column, so each layer is two longer. | The sixth layer lands in gold tile by tile: one at the end of each of the five rows (a live count 1 to 5), one on top of each of the five columns (1 to 5), then the corner tile: 11 (`F-Q.next_layer.answer`). The square is now 6 by 6 and has one more row and one more column, said in the picture by a gold row and a gold column | highlight; a live count; lagged reveal | |
| 03_5 | One plus three plus five plus seven plus nine is five times five. The same picture works a hundred layers on. Ninety-nine rows, ninety-nine columns, one corner. | The sum written out: 1 + 3 + 5 + 7 + 9 = 5 × 5 (`F-O.sum`). The square pulls back by scaling until its tiles are specks, a 99 by 99 square drawn as row strips; the same ripple runs once as three gold pieces (99 end tiles, 99 top tiles, one corner), and the readout computes the hundredth layer, 199 (`F-O.hundredth`), and the square, 100 × 100 = 10,000 (`F-O.sum.hundred`). Lower-third: "gnomon: the Greek word for this L-shaped layer (Aristotle, Physics III.4)" | a zoom made by scaling; a formula that grows from the numbers; lower-third | |
| 03_6 | Look at what the picture showed, and what it took for granted. Each arm is as long as the side it wraps, at every size. That is the reason. A picture that shows why is a proof. Remember this card. | The card, first time. **Shown:** for every whole number n from 1, the first n odd numbers add up to n × n. **Assumed:** an L of two arms, each as long as the side of the square it wraps, and one corner tile, makes the next square, ticked, with the close-up of an arm's last tile beside the square's last row | the card | hold |

- **The payoff, early** (plan rule 3): a whole proof in a minute, with the
  device that will fail in chapter 14.
- **The next size:** the step from one layer to the next is the proof, and
  the reason is seen at every size (an arm is as long as the side it wraps),
  not read off five cases.
- **Ruth's change:** the first draft's card said "we checked that", which is
  the move chapter 02 takes apart; the card now gives the reason.

## 04_staircases (60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | One more trick for the cubes. Build a staircase of tiles. One, two, three, four, five. That is fifteen tiles, but adding them one at a time is slow. | A staircase rises column by column in the five size colours (column k is size k): 1, 2, 3, 4, 5 (`F-T.columns`), and a counter ticks 1, 3, 6, 10 and lands on 15 (`F-T.counts`, `F-T.count`) | lagged reveal; a live count | |
| 04_2 | So make a copy of the staircase, and turn it half way round. | A copy lifts off the first and turns half way round beside it, in the same five colours, with an ink outline: one rigid half turn | a morph: one rigid half turn | |
| 04_3 | ⟨predict⟩ Your turn. The two staircases lock into a rectangle. How wide is it, and how tall? | The copy hovers beside the first, turned, not in place. The card: "? wide, ? tall" | your-turn card; the countdown ring | listen 5 s (countdown) |
| 04_4 | Five wide and six tall. Each column of one staircase meets a column of the other, and every pair comes to six. Thirty tiles in the rectangle, so fifteen in each staircase. | The copy settles: a rectangle 5 by 6 (`F-Q.rectangle.answer`), 30 tiles (`F-T.rectangle.cells`). Under the columns: 1 + 5, 2 + 4, 3 + 3, 4 + 2, 5 + 1, each pair a pink and a violet, a copper and a blue | highlight; brace labels | |
| 04_5 | It works at any height. Two staircases always make a rectangle one taller than it is wide. So the staircase is half of that rectangle. | A sweep: the staircase grows from 1 to 12 columns while the pair labels update (1 + 12, 2 + 11, …). In letters, with word labels: 1 + 2 + … + n = n (n + 1) ÷ 2. Then the hundred-step staircase: 5,050 (`F-T.count.hundred`). Lower-third: "triangular numbers" | a formula earned from the numbers; a live sweep | |
| 04_6 | Remember fifteen. The cubes' square is fifteen on a side. | The card. **Shown:** for every whole number n from 1, 1 + 2 + … + n is half of n × (n + 1). **Assumed:** column k of one staircase meets column n + 1 − k of the other, so every pair adds to the same n + 1, ticked, with 1 + 5, 2 + 4, 3 + 3 at five columns. The 15 lifts off in gold and waits in a corner | the card | hold |

- **Why it is here:** fifteen is the side of the cubes' square, and "two
  copies" is the idea Siu takes to three. The staircase's columns are the
  five cubes' sides, so 06_3 pairs them again.
- **Left out:** the story of young Gauss and 1 to 100 (the earliest account
  names no numbers; fact sheet §Sources).

## 05_slabs (83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | A cube five tiles on a side is five slabs, each five by five. A cube of side four is four slabs. Fifteen slabs in all. | The five cubes; each comes apart into its slabs. "15 slabs" (`F-K.slabs`) joins the waiting 15 | the cubes; lagged reveal | |
| 05_2 | Lay them flat as L-shaped layers, each as wide as its cube. The single tile goes first. The two cube wraps around it. One slab fits the corner of the L. The other must be cut in half. | A board. The pink tile. One copper 2 by 2 slab takes the bend of the new layer; the other splits into two halves, one at the end of each arm, the seam kept. The square is 3 on a side (`F-K.sides`) | morph from solid to board; highlight on the cut | |
| 05_3 | The three cube needs no cutting. One slab on the corner, one on each arm, and the square is six on a side. | Three green 3 by 3 slabs: corner, arm, arm. Side 6 | morph | |
| 05_4 | ⟨predict⟩ Your turn. The four cube has four slabs. How many of them will have to be cut? | The 6 by 6 square, the outline of the next layer, four blue slabs waiting. Each arm is marked "4 wide, 6 long" (`F-K.arm.4`). The card: "slabs to cut: ?" | your-turn card; the countdown ring | listen 8 s (countdown) |
| 05_5 | Just one. Each arm is six long, room for one and a half slabs. So two go in whole, and one is cut in half. | Blue corner slab; each arm 4 wide and 6 long, labelled "6 ÷ 4 = 1½ slabs" (`F-K.arm.4.slabs`). The cut slab lifts and its two halves go one to each arm before the answer lands: 1 (`F-Q.cut_slabs.answer`). Side 10 | highlight; brace labels | |
| 05_6 | The five cube wraps around with no cut at all. Now the square is fifteen on a side. One plus two plus three plus four plus five. | Five violet slabs; each arm 5 by 10, two whole slabs (`F-K.arm.5.slabs`). As the layers land the row 1, 3, 6, 10, 15 from 02_2 lights on the sides (`F-K.sides`). The side braced "1 + 2 + 3 + 4 + 5", and the waiting gold 15 lands on it (`F-K.side`) | brace; morph | |
| 05_7 | Five cubes, one square. The same two hundred and twenty-five tiles, counted two ways. Cube by cube, or fifteen times fifteen. | A counter runs over the tiles to 225 (`F-K.unit_squares`). 1³ + 2³ + 3³ + 4³ + 5³ = 15² | count-up; a formula | hold |

- **The owner's first picture**, built slab by slab: 15 slabs, two of them
  cut, 17 pieces (`F-K.pieces`).
- **Misconception defused:** "the cut slabs are a fudge". The cut is forced
  and counted: an even cube's arms need half a slab each (05_5), and the
  cold open said so.
- **Counted two ways** (the video's takeaway) is first said here: the pile of
  225 tiles by cube and by square.

## 06_every_cube (82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | ⟨predict⟩ That was five cubes. Would a sixth one fit? | The 15 by 15 square with the outline of a layer 6 wide. "6³ = 216 tiles" (`F-K.sixth.cube`) waits beside it | a question card | |
| 06_2 | Its layer must be six wide. The corner takes one slab. Each arm is six wide and fifteen long, and fifteen is two and a half sixes. So the two arms take five slabs. Five and one make six. Exactly one cube. | The sixth layer builds in gold: a 6 by 6 corner, arms 6 by 15 (`F-K.arm.6`), "15 ÷ 6 = 2½ slabs" (`F-K.arm.6.slabs`). The square is 21 on a side (`F-K.sixth.side`). The row from 02_7 returns for a beat and its question mark becomes 441 = 21 × 21 (225 + 216: five for five becomes six for six) | highlight; brace labels; morph | |
| 06_3 | The arm is as long as the five smaller cubes' sides laid end to end. One and five make six. Two and four make six. Three is half of six. Pair them up for any cube, and the arms always take one slab fewer than the cube has. | The arm is ruled by the old square's edge, already ruled 1, 2, 3, 4, 5 by the layers' feet: faint projection lines run from the left edge's marks to the right arm and from the bottom edge to the top arm; each arm takes its seams at 1, 3, 6, 10 and its five 6-wide pieces fill pink, copper, green, blue, violet, each with its numeral. In each arm the violet 5 slides out sideways and comes back above the pink 1, and the two close into a 6 by 6 slab with a gold seam; likewise copper 2 and blue 4; the green 3 stays, and the two greens, one in each arm, flash and are braced "half + half". "1 + 5", "2 + 4" as in 04_4. The strips for 7 cubes (pieces 1 to 6, three pairs, nothing cut) and 8 cubes (pieces 1 to 7, three pairs and the 4 alone) pair themselves beside it in two lagged passes. Then one line in letters, with word labels: pieces 1, 2, …, k − 1, each with its partner makes k (the strips' arms are `F-K.arm.7` and `F-K.arm.8`, facts to add: build notes) | highlight; brace labels; morph; a derivation that turns numbers into letters | |
| 06_4 | So every cube fits, at every size. The cubes up to any number make a square. Its side is one plus two and so on, up to that number. | 1³ + 2³ + … + n³ = (1 + 2 + … + n)², boxed. The card. **Shown:** for every whole number n from 1, the cubes up to n³ add up to the square of 1 + 2 + … + n. **Assumed:** each cube's layer wraps the square the cubes before it made, and its arms are k − 1 half slabs, ticked, with the pairing and "the first cube is the corner tile" | a formula in a box; the card | hold |
| 06_5 | Cut every cube's layer into thin layers, one tile wide. Nicomachus noticed that every cube is a run of odd numbers. Two cubed is three plus five. Three cubed is seven plus nine plus eleven. So this square is chapter three's again, made of fifteen layers. | The 15 by 15 square redrawn as fifteen thin layers, the thin layers of chapter 03 recolouring from their layer colours to their cube's, in lighter and darker steps of it, the odd numbers 1 to 29 in runs: 1; 3 + 5; 7 + 9 + 11; and so on (`F-K.odd_runs`), with the rule under them: cube k is k odd numbers from k(k − 1) + 1, so 5³ = 21 + 23 + 25 + 27 + 29. Under that, the sum: the first 15 odd numbers make 15 × 15 | a morph: each thick layer splits into its thin ones; a formula | |
| 06_6 | Al-Karaji proved the rule with L-shaped layers around the year one thousand. The slabs cut in half are much younger, from nineteen eighty-five. | A line of dates: Nicomachus, about 100 (`F-X.nicomachus`); Aryabhata, 499, "states the rule" (`F-X.aryabhata`); al-Karaji, about 1000 (`F-X.al_karaji`); Fry, 1985, for the cut slabs (`F-X.fry`). Source line: "the cut slabs: A. L. Fry, Mathematics Magazine, 1985" | a number line of dates | |

- **Earned formula** (plan rule 6): 6 and 15 first, then the letters.
- **Inevitable in hindsight** (plan rule 19): chapter 04 is the reason
  chapter 05 worked; the arm's pieces are the staircase's columns, paired.
- **History, exactly:** Nicomachus did not state the rule that carries his
  name (Heath); the first spoken date is the idea (al-Karaji's layers), the
  last the cut slab (1985). A second proof of the cubes' sum sits in 06_5:
  chapter 03's result with n = 1 + 2 + … + 5.

## 07_three_stacks (62 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Staircases added up the numbers. Now add up the squares. One, four, nine, sixteen. Stack them, biggest at the bottom, every level pushed into the same corner. Thirty cubes. | One slab of each of cubes 1 to 4 (the 1 by 1, 2 by 2, 3 by 3 and 4 by 4 faces just on the board) lifts off and stacks: levels of 16, 9, 4 and 1 cubes (`F-S.layers`), one corner shared. A counter reaches 30 (`F-S.stack`) | the cubes; a live count; a morph from the board | |
| 07_2 | ⟨predict⟩ Two staircases made a rectangle. Your turn. Guess how many of these stacks make a solid box, if cubes may be cut in half? | The stack and a ghost copy (outline). The card: "? stacks make a box" | your-turn card; the countdown ring | listen 7 s (countdown) |
| 07_3 | Three. Here are three copies of the stack, each turned to fit. | Three equal stacks in blue, copper and pink (`F-Q.stacks.answer`), with a legend chip "first, second, third copy". A gold dot marks each stack's one-cube apex. The second turns twice and slides in against the first; the third turns the other way and comes down over them (`model::stacks::TURNS`): each is lift, turn clear, slide in, shown as turns, keyed in three poses | the cubes; rigid turns (`F-S.turns`) | |
| 07_4 | Take the block apart one slab at a time. Four slabs of twenty cubes, and a top of ten. Each time, the blue square gives up an L-shaped layer, and the other two stacks take it. | Four gold seams on the block's front and right faces; the step lifts and slides right, then slabs 3, 2, 1, until five slabs stand in a row, face up: 20, 20, 20, 20 (`F-S.top.layer`) and 10 (`F-S.top`). Blue shrinks 16, 9, 4, 1 (`F-S.layers`); the L it sheds pulses gold on its slab and again on the slab above, where it is copper (the top arm) and pink (the right arm and the corner). Three tallies: 30, 30, 30 (`F-S.stack`). Every slab is a full 4 by 5 board | the cubes; lagged slides; highlight | |
| 07_5 | Put it back, with no gap. Four wide, five deep and four tall, with a staircase of ten cubes on top that covers exactly half of it. | The slabs slide back and settle: the block with braces 4, 5, 4 (`F-S.block`). On top a staircase of 10 cubes (`F-S.top`) over a top face of 20 squares (`F-S.top.layer`) | braces; highlight | hold |

- **The owner's fourth picture.** The three stacks are
  `model::stacks::placed(4, 0..3)`, cube for cube. The lock is seen in the
  slabs, not on trust: each slab is full and each cube lands in exactly one
  place.
- **The quiz is honest** (R1, Ruth): with whole cubes no number of stacks
  makes a box (90 cubes have no box with every side at least 4); the answer
  three is a box of four and a half. One or two stacks have 30 or 60 cubes,
  under the 64 of the least box round one stack.
- **Left out:** "never mirrored". Siu's stack is its own mirror image, so a
  mirrored copy is a turned copy; the tick could not fail.

## 08_the_box (59 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | ⟨predict⟩ The top is exactly half full. How do you make it flat without adding a thing? | The block, its top step in gold and its empty half outlined in `dim` and pulsed once | a question card | |
| 08_2 | The empty half is the step turned round, two staircases again. Ten cubes cannot cover twenty places, so slice every one in half. Lift the upper halves, turn them half way round, and lower them into the empty half. | A flat plan of the 4 by 5 top face beside the block: ten gold step cells, ten dashed empty cells, a gold dot at the centre; under the columns "1 + 4, 2 + 3, 3 + 2, 4 + 1 = 5"; the step turns half way round the dot onto the empty cells. Then in the block: seam lines on the step's cubes at half height, the upper slice (10 half cubes) lifts at least one cube clear, turns half way round and comes down into the gap (`model::stacks::turned_half`): 20 half cubes (`F-S.top.halves`), level | the cubes; one rigid half turn; a flat plan | |
| 08_3 | Now it is a box. Four wide, five deep, and four and a half tall. That makes ninety. Three stacks of thirty. | Braces 4, 5, 4½ (`F-S.box`), the level top held two seconds with the music thinned before the braces. 4 × 5 × 4½ = 90 (`F-S.box.volume`) = 3 × 30 (`F-S.three_stacks`) | braces; a formula | |
| 08_4 | With any number of levels, the box is that many cubes wide, one more deep, and that many and a half tall. And one stack, the sum of the squares, is a third of the box. | Numbers first: 30 = ⅓ × 4 × 5 × 4½. Then, with word labels lit in turn as each is said, 1² + 2² + … + n² = ⅓ n (n + 1)(n + ½). Lower-third: "precisely: the textbook form is n (n + 1)(2n + 1) ÷ 6" | a formula earned from the numbers; lower-third | hold |
| 08_5 | This picture is not ancient. Man-Keung Siu, of the University of Hong Kong, published it in nineteen eighty-four, under the title Proof Without Words. | Siu's formula. Source line: "Man-Keung Siu, Mathematics Magazine 57 (1984), p. 92" (`F-X.siu`) | a card | |

- **A "huh"** (plan rule 17): the half in n + ½ is a real slice of real
  cubes. The slice is only the second move: the first is that the empty half
  is chapter 04's second staircase, at every height.

## 09_every_height (52 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | ⟨predict⟩ But that was a stack of four levels. Why would five fit? | The block as it was before the slice, as its five slabs in its three colours | a question card | |
| 09_2 | Give each stack one more level, a five by five slab at its base. Every slab of the block, four by five, becomes five by six, with an L-shaped layer of two arms and a corner. And one more slab goes underneath. | The five slabs of the block stand in a row. Slab 1 gains a gold L round its rectangle (right arm 5, back arm 4, corner 1); slabs 2, 3, 4 gain theirs in a lagged wave; the step's slab gains a column of 5. A gold 5 by 6 slab slides in beneath. Gold turns to the stacks' colours (under blue, behind copper, beside pink) and the slabs close up. The result is 5 by 6 by 5 with a step of 15 on top (`F-S.grown.box`, `F-S.grown.top`), each new base slab 25 cubes (`F-S.grown.slab`) | the cubes; lagged waves; a morph | |
| 09_3 | Three new slabs of five by five, and nothing overlaps. Like the L-shaped layer, it is what you add to get the next size. | Beside the block, small: the flat square gaining its L, and the three slabs. The two growths pulse together | highlight; a split picture | |
| 09_4 | So if the stacks fit at one height, they fit at the next. And at height one, three single cubes make a box one by two by one and a half. That covers every height. | Three single cubes make a box 1 by 2 by 1½ (`F-S.one.box`). A chain of blocks, 1, 2, 3, 4, 5 levels, each growing into the next, as outline-only stacks. Lower-third: "mathematical induction: the first case, and each case leads to the next. Not guessing from cases, as in chapter 2" | lagged reveal; lower-third | |
| 09_5 | So this picture passes the card. | The card. **Shown:** for every whole number n from 1, three stacks of n levels fill a box n by n + 1 by n + ½. **Assumed:** each stack is the first one turned, ticked; the step and its turned copy fill the top, ticked, with "two staircases make a rectangle" (chapter 4); the next height fits, ticked, with "three new slabs" | the card | hold |

- **The next size, made visible:** `model::stacks::grows` checks that the
  solid of height n + 1 is the solid of height n lifted by one, plus one new
  base slab for each stack (`F-S.grows.through`). The picture shows it as
  the step each slab takes (an L, and one slab beneath), so every new cube is
  visible, which the lift-and-slide was not from one viewpoint.
- **For Ruth:** this is the step Siu's printed picture leaves to the reader.

## 10_halves (55 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Pictures can even add up sums that never end. Take a square of area one. Colour half of it. Then half of what is left. Then half of what is left again. | The board's outline of 09 becomes a unit square, now of area one. Pieces of ½, ¼, ⅛ fill in, each labelled with its fraction (`F-H.pieces`), the newest gold and the rest blue | lagged reveal; fractions set as mathematics | |
| 10_2 | After five pieces, thirty-one out of thirty-two equal parts of the square are coloured. The gap is one thirty-second, exactly the size of the last piece. | Five pieces. Readouts: "filled 31/32" (`F-H.filled`), "gap 1/32" (`F-H.gap`). The gap and the last piece flash as equals | live readouts from the pieces; highlight | |
| 10_3 | ⟨predict⟩ Will the gap ever close? | The corner grows to fill the frame: more pieces, and the gap still there | a question card; a zoom made by scaling the picture | |
| 10_4 | Not after any number of steps. Guess a total a thousandth short of one, and ten pieces pass it. A millionth short, and twenty pieces pass it. So no number smaller than one is big enough to be the total. And nothing ever passes one. | Under the square a line from 0 to 1, as long as the square's side, with a tick for each filled part and the square's edge at its right end: the pieces are all inside the square. A guess card drops on the line at 0.999; the ticks run along and piece 10 passes it (`F-H.pieces_to.thousandth`, gap 1/1,024: `F-H.gap_at.thousandth`) and the card takes a red cross; then 0.999999 and piece 20 (`F-H.pieces_to.millionth`, `F-H.gap_at.millionth`); the red crosses pile into a band from 0 to the edge. Each pass is one soft note, climbing | a live line; guess cards; a zoom capped near 2^10 | |
| 10_5 | One is the only number left. That is exactly what it means to say this endless sum equals one. | The last guess, 1, lands on the edge. No piece passes it, so it has no cross, and it turns gold. ½ + ¼ + ⅛ + … = 1. Lower-third: "precisely: after n pieces the sum is 1 − 1/2ⁿ, below one every time and above every number below one eventually. Geometric series". The card. **Shown:** the pieces fill the square up to a gap the size of the last piece, which never closes and goes below any size. **Assumed:** every piece lies inside what was left, so the pieces never overlap and never leave the square, ticked | a formula; lower-third; the card | hold |

- **What "equals" means** (for Ruth): no partial sum reaches 1
  (`F-H.checked.through`), and every number below 1 is passed; for pieces
  that are all positive that is the whole definition.
- **A published version:** Warren Page, 1981 (`F-X.page`); the picture is
  older than any source found, so no one is named on screen.
- **For programmers** (a tag at 10_4, bottom right): "in 64-bit floats this
  sum reaches exactly 1.0 after 54 pieces; the machine cannot see the gap".

## 11_a_third (74 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | Now a stranger one. Cut the square into four quarters. Colour one. Leave two alone. And cut the last one into quarters again. | A unit square in four quarters. The lower left one fills blue; the upper left and lower right stay outlined; the upper right is cut in four | lagged reveal | |
| 11_2 | Inside it, do the same. Colour one, leave two, cut the last. And again, and again, without end. | Levels two to eight march into the corner (`model::series::quarters`); the first three carry their fractions ¼, 1/16, 1/64 (`F-R.pieces`) | a cascade; a zoom made by scaling | |
| 11_3 | ⟨predict⟩ Your turn. A quarter, plus a sixteenth, plus a sixty-fourth, and so on forever. What fraction of the whole square is coloured? | The card: "¼ + 1/16 + 1/64 + … = ?" | your-turn card; the countdown ring | listen 6 s (countdown) |
| 11_4 | One third. At every size there are three equal squares. Slide one onto another and it fits. So give the other two their own colours. | The answer ⅓ (`F-Q.share.answer`). Level by level the copper square slides, as an outline, down onto the blue one and fits, and the pink slides left onto it and fits (copper is blue slid up by its own side, pink is blue slid right), then they slide back; levels 4 to 8 in one lagged pass. Then the upper ones fill copper and the right-hand ones pink (`F-R.shares`) | highlight; lagged reveal; slides | |
| 11_5 | Three colours, and each one gets a square of every size. Share the last corner three ways, and each colour, with its share of the corner, is exactly one third. And the corner shrinks below any size you can name. | Readouts after three sizes: each colour 21/64 (`F-R.share`), the corner 1/64 (`F-R.left`), 21/64 + 21/64 + 21/64 + 1/64 = 1. The corner is cut into three strips, one for each colour: "21/64 + 1/192 = 1/3" (`F-R.short`). The corner grows to fill the frame: it is the same picture again, and the strip is a quarter as big each time. Lower-third: "precisely: after n cuts each colour holds (1 − 1/4ⁿ) ÷ 3, and each strip is a third of the corner" | live readouts; a zoom made by scaling; lower-third | |
| 11_6 | Archimedes added quarters of quarters like these, over two thousand years ago, to measure a slice of a parabola. Sunday Ajose and Roger Nelsen published a picture of this sum in nineteen ninety-four. | The card first. **Shown:** three colours share the square equally, short of all of it by the uncut corner. **Assumed:** the three squares of each size are equal, ticked, with "each slides onto the other". Then the lower-third, after the tick: "Archimedes, Quadrature of the Parabola, Prop. 23: a finite sum and its remainder, the same remainder, a third of the last piece". Source line: "S. A. Ajose and R. B. Nelsen, Mathematics Magazine, 1994" (`F-X.ajose_nelsen`) | the card; lower-third | hold |

- **The owner's second picture.** The next size is the picture itself, one
  scale down.
- **Archimedes, exactly:** his Proposition 23 is the finite sum with its
  remainder, a third of the last piece, which is `F-R.short`: each colour
  falls short of a third by a third of the corner, so each colour plus its
  strip is exactly a third at every level, and what is left over shrinks by
  four each time.

## 12_the_cut_cube (83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Pictures can do algebra too. Take a square, five on a side, and cut each side into a long part and a short part. Three and two. | A 5 by 5 square of tiles. One cut across and one cut down, each at 3 (`F-B.parts`). A legend chip: a block's colour says how many of its sides are short (violet none, blue one, green two, copper three, pink four) | a grid; lines that draw | |
| 12_2 | Four pieces. A big square, a small square and two equal rectangles. Nine, six, six and four make twenty-five. A square wearing an L-shaped layer. | The pieces 9, 6, 6, 4 (`F-B.square.each`) coloured by how many short sides they have: 9 violet, the two 6s blue, 4 green; 9 + 6 + 6 + 4 = 25 (`F-B.square.total`). The L of 6 + 6 + 4 pulls away and returns. Lower-third: "(a + b)² = a² + 2ab + b²: Euclid, Elements II.4, in words" (`F-X.euclid`) | highlight; a formula; lower-third | |
| 12_3 | Now do the same to a cube. Cut its width, its depth and its height, each into three and two. | A cube 5 on a side, three cuts through it as seam lines | the cubes, as whole blocks | |
| 12_4 | ⟨predict⟩ Your turn. A block's kind is how many of its sides are short. How many blocks does the cube fall into, and how many of each kind? | The cut cube, closed. The card: "? blocks; ? of each kind" | your-turn card; the countdown ring | listen 8 s (countdown) |
| 12_5 | Eight blocks, in four kinds. One, three, three, one. | The blocks drift apart and gather by kind, coloured by how many short sides each has: 1, 3, 3, 1 (`F-Q.kinds.answer`, `F-B.cube.kinds`) | morph that keeps identity; lagged reveal | |
| 12_6 | The blocks add up to one hundred and twenty-five. Five cubed. And nothing depended on three and two. Slide the cut, and it is still eight blocks. | Volumes 27, 18, 12, 8 (`F-B.cube.each`); 27 + 54 + 36 + 8 = 125 (`F-B.cube.by_kind`, `F-B.cube.total`). The cut slides to 4 and 1 and back: every block changes size and none appears or vanishes, a violet block growing as the cut moves. (a + b)³ = a³ + 3a²b + 3ab² + b³, each term in its blocks' colour | live recomputation from the cut; a formula | |
| 12_7 | And the fourth power? Every block of the cube appears twice, once with a long fourth length and once with a short one. One, four, six, four, one. We can count its blocks, even though no one can build it in our space. | Two copies of the eight blocks, tagged "× 3" and "× 2", the second set off along a dashed direction, as in the owner's picture: 16 (`F-B.fourth.blocks`). Then rows 1 2 1, 1 3 3 1, 1 4 6 4 1, each number the sum of the two above it (`F-B.fourth.kinds`, `F-B.fourth.from_cube`); then one line, 81 + 216 + 216 + 96 + 16 = 625 (`F-B.fourth.by_kind`, `F-B.fourth.total`). The card, after the rows have held. **Shown:** the cut square and the cut cube, for any two lengths. **Counted, not built:** the fourth power. Lower-third: "Pascal's triangle" | a table built from primitives; the card; lower-third | hold |

- **The owner's third picture.** The second and third powers are pictures;
  the fourth is bookkeeping the pictures taught, and the card says so. The
  fourth power is a four-dimensional block, which cannot be built in
  three-dimensional space; flat drawings of one exist, and none is attempted.
- **The next size** here is any pair of lengths: the cut slides (12_6).

## 13_four_triangles (84 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | One more, and it may be the most famous picture in mathematics. A right triangle with shorter sides three and four. Set four copies inside a square. | A square 7 on a side. Four grey triangles, one in each corner (`model::pythagoras::corners`), sides labelled 3 and 4 (`F-P.legs`) | lagged reveal | |
| 13_2 | They leave a tilted square in the middle, with the triangle's long side as its side. | The tilted square in gold outline, one side marked "long side" | highlight | |
| 13_3 | Now slide the triangles. Nothing turns, nothing flips. Two pairs close into rectangles, and the space left over is two squares. One on the side of three, one on the side of four. | Three triangles slide (`F-P.slides`). What is left: a 3 by 3 square in green and a 4 by 4 square in blue, the slab colours of cubes 3 and 4 (`F-P.squares`) | morph: slides only | |
| 13_4 | ⟨predict⟩ Your turn. Same triangles, same big square. So how big was the tilted square? | Both arrangements side by side. The card: "tilted square = ? tiles" | your-turn card; the countdown ring | listen 6 s (countdown) |
| 13_5 | Nine plus sixteen. Twenty-five. The same pile, counted two ways. The triangles did not change and neither did the big square, so the space left over cannot change. And twenty-five is five times five, so the long side is five. | 49 − 24 = 25, both ways (`F-P.big`, `F-P.triangles`, `F-Q.tilted.answer`). The tilted square fills violet, a 5 by 5 slab, the colour of cube 5's slabs: 3² + 4² = 5² reads as three of the cold open's slabs. 25 = 5 × 5 (`F-P.long_side`) | a formula; highlight | |
| 13_6 | It holds for every right triangle. This is the theorem that carries Pythagoras's name. Change the shape, and both arrangements follow. | The two shorter sides slide through other lengths, both arrangements redrawn live; at 2 and 5 the squares are copper and violet and the readout is 4 + 25 = 29 (`F-P.other.tilted`), "no whole long side" (`F-P.other.whole_side`), and the argument does not care. a² + b² = c², with word labels. Lower-third: "Pythagoras's theorem" | live recomputation from the triangle; a formula | |
| 13_7 | But this picture took two things for granted. That the tilted shape is a square, and that the edges are straight. Both are true, because a right triangle's two sharp angles make a right angle. | A corner grows to fill the frame: on the big square's straight side, the two sharp angles and the tilted corner between them. The card. **Shown:** for every right triangle, a² + b² = c². **Assumed:** the tilted shape is a square, ticked, with "the two sharp angles add to a right angle"; the sides are straight, ticked; the pairs close into a by b rectangles, ticked. After the card has held three seconds alone, the lower-third: "Nobody knows who first drew this picture. A close cousin is in a third-century Chinese commentary on the Zhoubi suanjing" | a zoom made by scaling; the card; lower-third | hold |

- **Why it is here** although the owner finds it famous: it is the honest
  twin of chapter 14, the same argument with its checks done.
- **History, exactly:** never "Pythagoras's own proof", never "a Chinese
  proof from 200 BC" (fact sheet §Sources).
- **Counted two ways:** said at 13_5, the sentence the first draft left
  unspoken.

## 14_the_missing_square (83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | Four pieces, two triangles and two L-shapes, that seem to make a triangle thirteen wide and five tall. | A grid. The four pieces in their first arrangement (`model::lie::first`), outlined 13 by 5 (`F-L.frame`): the triangles in `dim`, the L of 8 blue, the L of 7 pink | a grid; lagged reveal | |
| 14_2 | Slide the pieces, as before. Nothing turns, nothing flips. The outline looks the same. But now one tile is empty. | Each piece slides (`F-L.slides`) into the second arrangement. The two arrangements then sit side by side with the grid visible for three seconds, music thinned, so the viewer counts. One cell is empty (`F-L.hole.at`) and pulses red | morph: slides only; highlight | hold |
| 14_3 | Same pieces, same outline, and a tile has vanished. If this were a proof, it would prove that thirty-two equals thirty-three. | "pieces: 12 + 5 + 8 + 7 = 32" (`F-L.pieces`, `F-L.pieces.total`); "second picture: 32 + 1 = 33" (`F-L.outline.second`) | a formula; highlight | |
| 14_4 | ⟨predict⟩ Your turn. The flaw is on the long sloping edge. Which triangle is steeper, the big one or the small one? | Both pictures, the sloping edges in ink, with rise and run written on them as the ring runs: 3 up, 8 across; 2 up, 5 across. The card: "steeper: big or small?" | your-turn card; the countdown ring | listen 8 s (countdown) |
| 14_5 | The small one. It climbs two in five. The big one climbs three in eight. So the long edge is not straight. It bends. | Slopes 2/5 and 3/8 (`F-L.slopes`), 0.400 and 0.375 (`F-L.slopes.thousandths`); the answer (`F-Q.slopes.answer`). A true straight edge from corner to corner (`F-L.straight_slope`) drawn as an ink ruler. A lens scales the group about the bend, eight times in both directions (the slopes as decimals, 0.400 and 0.375, are on the edges), so the bend shows without any stretch: the corner hangs off the ruler by 1/13 of a tile, measured straight up (`F-L.dip`) | highlight; a zoom made by scaling | |
| 14_6 | One edge sags below the true line, the other bulges above. Cut that sliver along the grid, and its pieces slide into the empty tile and fill it. The sliver was exactly one tile. | The two outlines laid over each other at true scale (`model::lie::outline`): 32 and 33 tiles (`F-L.outline.first`, `F-L.outline.second`), a true triangle 32½ (`F-L.true_triangle`). The sliver in red: thickest 1/8 (`F-L.sliver.thickest`). The grid's lines cut it into 17 pieces; they slide, lagged, each by whole tiles and nothing turning, into the empty tile, and fill it (`F-L.sliver` = 1). Lower-third, last: "2, 3, 5, 8, 13: neighbours in the Fibonacci numbers. That is why the slopes are so close" | highlight; live readouts of the areas; slides | hold |
| 14_7 | Neither picture was ever a triangle. The trick goes back to Paul Curry, a New York magician, in nineteen fifty-three. Martin Gardner found triangle forms like this one and published them three years later. | The card, failing. **Shown:** nothing. **Assumed:** the long edge is straight, with a red cross drawn on the bend. Two seconds of silence with the cross on screen, then the source line: "P. Curry, 1953; M. Gardner, Mathematics, Magic and Mystery, 1956" (`F-X.curry`, `F-X.gardner`) | the card, with its cross | |

- **Honest picture:** the pieces are drawn exactly, on the grid; nothing is
  fudged to hide the bend, and no part of the picture is stretched. At true
  scale each bend is 1/13 of a tile off the straight line (`F-L.dip`,
  `F-L.bulge`), about a thirteenth of a tile, which the lens of 14_5 shows
  without distortion.
- **Where the tile went:** the sliver between the outlines has area exactly
  one, and the picture slides its pieces into the hole.
- **The quiz is fair for Maya:** two in five against three in eight is a
  fraction comparison she can do, with rise and run on the edges.
- **The chapter does not announce the lie:** the first slide is watched, and
  the narrator says "go wrong" only through 14_3's "if this were a proof".

## 15_what_makes_a_proof (60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | Two pictures, one argument. Same pieces, two arrangements, compare what is left. Once it gave a famous theorem. Once it gave nonsense. | The two rearrangements side by side, small, each with its card: one ticked, one crossed | a split picture | |
| 15_2 | ⟨predict⟩ So what makes a picture a proof? | Both cards, enlarged | a question card | |
| 15_3 | Three things. Every piece is accounted for. Every fit has a reason, not just a look. And the reason does not depend on the size you happened to draw. | Three lines set themselves, large, one for each sentence. Beside each, small pictures of the proofs that leaned on it, from the shelf: the tiles counted, the straight edges, the next layer | a card; lagged reveal | |
| 15_4 | That is why the cubes needed a second chapter. The picture showed five cubes. The reason covered every cube. | The opening square returns. Then one layer of it alone, with every length named: width k, arm 1 + 2 + … + (k − 1), which is (k − 1) half slabs, the pieces paired as in 06_3 | morph; labels set as mathematics: the rigorous form of the opening picture | |
| 15_5 | A program rebuilt these pictures piece by piece, the cubes up to forty, and Siu's box up to twenty-four levels. That is still only checking. A check up to some size says nothing about the next size. | Receipts under the picture: "rebuilt and checked by code: cubes up to 40 (`F-K.tiled.through`), stacks up to 24 levels (`F-S.filled.through`), every sum up to 1,000 (`F-K.sum.through`)", and "a check up to some size says nothing about the next size". Small tag, bottom right: "for programmers: two implementations compared; neither is a proof" | a receipts strip | |
| 15_6 | A picture cannot check itself. But when you can say why every piece fits, it does what checking cases never does. It shows you why. | The 15 by 15 square once more, un-dimmed, in full colour. Names to search: "proofs without words"; Roger B. Nelsen, *Proofs Without Words* (1993) and *Proofs Without Words II* (2000) (`F-X.nelsen`); Mathematics Magazine | a closing card; names to search | hold |

- **Ends on the rigorous form of the opening picture** (plan rule 15).
- **The receipts are honest about themselves:** the repository's own checks
  are checks, like the five cubes of chapter 02, and each reach sits next to
  its own picture.
