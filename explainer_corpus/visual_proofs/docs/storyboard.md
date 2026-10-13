# Storyboard — Proofs You Can See

The hand-edited source of this video's words and pictures. `python3 -I
tools/storyboard.py visual_proofs` checks it (gate G2) and writes
`script.tsv` from the words column. Facts: `docs/fact-sheet.md` and
`oracle.py` (every number below names its fact key); brief: `docs/brief.md`;
reviews and their dispositions: `docs/reviews/visual_proofs/`.

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
  `model::QUIZZES`, and its numbers are the `F-Q.<name>.*` facts.
- **A question to carry** (01_2, 06_1, 08_1, 09_1, 10_3, 15_2): the question
  as a card with no ring, held about two seconds.

**The card.** Every proof ends on the same two-line card, in the same place:
**Shown** (the exact statement, with its range) and **Assumed** (the fit the
picture took for granted, with a tick and the reason once it is checked).
It first appears at 03_6, passes seven times, and fails once: at 14_7 the
Assumed line gets a red cross. Chapter 15 is that card read back.

**The cubes.** Every solid is made of unit cubes drawn by the scene itself:
a fixed viewpoint, each cube's three visible faces in three shades of its
colour, painted back to front. Solids move by sliding and by quarter and
half turns; the camera never moves. A flat picture (the board) sits beside
or under a solid, never in perspective.

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
silent fit (01_1, 01_2) and the first complete proof (03). The video ends on
the rigorous form of the opening picture: the cubes' layer for a size nobody
drew, with every length named (15_4).

**Dependence.** The proofs lean on each other in the order shown: the
layers of 03 are the shape of 05; the staircase of 04 is the side of 05's
square and the reason of 06; two copies in 04 become three in 07; the
L-shaped layer returns in three dimensions in 09 and as the cut square in
12; the two-arrangement argument of 13 is the one that fails in 14.

**Length.** The G2 estimate below is 15.6 minutes over 15 chapters, inside
the order's 12 to 16. If the owner wants it shorter, chapter 10 (the halved
square, about 49 seconds) folds into one beat of chapter 11, and 12_7 (the
fourth power, about 14 seconds) goes to the description.

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses
a data colour.

| Idea | Color |
|---|---|
| size 1: the first layer, the one cube, a block with no short side | pink `#EA87B3` |
| size 2: the second layer, the two cube's slabs, a block with one short side | copper `#CD853F` |
| size 3 | green `#83C167` |
| size 4 | blue `#58C4DD` |
| size 5 | violet `#994DE5` |
| a copy or a share, where nothing is being sized (04, 07 to 11): first, second, third | blue, copper, pink |
| the piece moving now, an answer as it lands, the next size (the sixth layer, the fifth slabs) | `hero` gold `#FFD866`, never as a resting fill |
| what does not fit: the wrong guess, the bent edge, the sliver, a failed tick | `warn` red `#FC6255` |
| pieces that only fill: the four triangles, a ghost of the last size | `dim` `#8A93A3` and `faint` `#3B4352` |
| the quiz card, its ring, the card's text, outlines | `ink` `#ECEFF4` |

The five sizes run in the order of the owner's picture (red, yellow, green,
blue, purple). Every pair among the five sizes, `hero`, `warn`, `dim` and
`ink` is at least CIEDE2000 21 apart (least 21.6, blue against `dim`); the
first draft's amber was 14.9 from `hero` and its violet 15.4 from `dim`, so
both moved. A test in the crate pins the table before any draft.

## Vocabulary (one word per idea)

**picture** and **proof**; **piece** (anything that moves) and **fit**;
**tile** (a unit square) and **cube** (a unit cube, and the five cubes);
**layer** (the L-shaped layer, and a stack's layers), with its **corner**
and two **arms**; **staircase** (flat); **stack** (Siu's solid) and **box**;
**slab** (a square slice, one tile thick); **block** (a part of the cut
cube); **colour** (a share of the quartered square); **copy**; **pattern**
and **check** (the hook); **took for granted** (spoken) for the card's
**Assumed**; **your turn** (every quiz). A triangle's sides are its two
**shorter sides** and its **long side**. Never "obviously", "clearly",
"trivially", "simply", "it is easy to see". The formal names appear once
each, in a lower-third: gnomon, triangular number, induction, geometric
series, Pascal's triangle, proof without words.

## Sound plan (for the score bead)

Nothing here is auditioned yet; the drafts of P5 are silent but for the
scratch voice.

- **Countdowns:** one mallet note a second, stepping down, the last three
  always the same, landing on the tonic as the answer appears (as in Video
  F).
- **Pieces landing** (01_2, 05_2 to 05_6, 07_3, 09_2, 13_3): each piece is
  one soft note of the chapter's chord, so a finished picture is a finished
  chord.
- **14_2:** the same landing notes, and the chord ends one note short.
- **15_6:** the motif resolves here and nowhere earlier.

## 01_cold_open (23 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Take five cubes, each a little bigger than the last. Slice every one of them into flat square slabs. | Five cubes in a row, of side 1 to 5, in the five size colours. Each comes apart into its slabs: 1, 2, 3, 4 and 5 of them (`F-K.slabs` in all). No numbers on screen | the cubes, drawn by projection; lagged reveal | |
| 01_2 | ⟨predict⟩ Lay the slabs flat, and they close up into one perfect square. No gap, and nothing left over. Why would that ever fit? | The slabs glide down onto a board and lock into one square (`F-K.side` on a side); two of them split in two on the way (`F-K.cut_slabs`). The finished square holds, then the question sets itself over it | morph that keeps identity (each slab keeps its colour from cube to board); a question card | |
| 01_3 | *(title)* | **Proofs You Can See**, over the square, which dims | title card | listen 3 s (the motif: three notes, then the chord) |

- **Cold open** (plan rule 20): the owner's first picture as a silent fit,
  the payoff as a puzzle, then the title. About 20 seconds before the title.
- **Honesty:** the slabs are `model::cubes::square(5)`, piece for piece; the
  two that split are the two the model cuts.

## 02_five_for_five (74 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Here is that fact in plain numbers. One cubed is one. Add two cubed, and the total is nine. Add three cubed, thirty-six. Then one hundred. Then two hundred and twenty-five. | The five cubes small along the top. Under them a running total counts up as each is added: 1, 9, 36, 100, 225 (`F-C.cube_totals`) | count-up from a tracker; lagged reveal | |
| 02_2 | Every total is a perfect square. One, three, six, ten and fifteen, each multiplied by itself. Five checks, and five squares. | Each total turns into its square: 1 × 1, 3 × 3, 6 × 6, 10 × 10, 15 × 15 (`F-C.cube_roots`). Five small ticks | a formula morph for each total; highlight | |
| 02_3 | So is it always true? Checking never finishes. And there is a worse problem. A pattern can break. | A sixth slot with a question mark; then the row slides up, small, and the words "a pattern can break" take the stage | a card for the last sentence | |
| 02_4 | Put points on a circle, and join every pair with a straight line. One point, one region. Two points, two. Three points, four. Four points, eight. Five points, sixteen. | A circle. The points arrive one at a time, unevenly spaced (`model::circle::DRAWN`); every pair is joined; the regions are tinted so they can be counted. A two-row table fills in: points 1 2 3 4 5, regions 1 2 4 8 16 (`F-C.regions`) | a table built from primitives; a live count of regions; lagged reveal | |
| 02_5 | ⟨predict⟩ Your turn. Pause the video if you want more time. How many regions will six points make? | The sixth point lands; its five lines are not drawn yet. The card: "6 points: ? regions" | your-turn card; the countdown ring | listen 4 s (countdown) |
| 02_6 | Thirty-one. Count them. The doubling pattern said thirty-two, and the pattern was wrong. | The five lines draw. Every region lights in turn while a counter runs to 31 (`F-Q.regions.answer`). In the table, "32" (`F-C.doubling`) is struck out in red and 31 written beside it. Lower-third: "precisely: 31 when no three lines cross at one point (six evenly spaced points give 30). Leo Moser, 'On the Danger of Induction', 1949" (`F-C.three_through_one.regions`, `F-X.moser`) | live counter; highlight; lower-third | hold |
| 02_7 | Five checks in a row prove nothing. To know the cubes always make a square, we need a reason that does not care where we stopped. And the best reasons are the ones you can see. | The cubes' row comes back under the circle's row: both read "five for five". The circle's sixth entry is red; the cubes' sixth is still a question mark. Small tag, bottom right: "for programmers: five passing tests are not a proof either" | a table; a labelled aside | |

- **The obvious approach fails on honest data** (plan rule 2): the circle's
  counts are computed from the drawn points by Euler's formula on the exact
  crossings, not from the closed form (`model::circle::regions`).
- **Edge kept on screen:** 31 needs the points in general position; the
  lower-third says so and gives the other way the pattern breaks.
- **Misconception defused:** "a pattern that holds five times is as good as
  proved".

## 03_layers (63 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Start small. One square tile. To make the next bigger square, wrap an L-shaped layer around it. That takes three more tiles. | One pink tile. A copper L of three tiles slides in round its corner: a 2 by 2 square | a grid; lagged reveal | |
| 03_2 | Wrap it again, and the layer takes five. Then seven. Then nine. | Layers three, four and five in green, blue and violet, each with its count beside it: 1, 3, 5, 7, 9 (`F-O.layers`). A 5 by 5 square (`F-O.square`) | lagged reveal; a live count of tiles | |
| 03_3 | ⟨predict⟩ Your turn. How many tiles will the next layer take? | The square with the faint outline of a sixth layer. The card: "next layer: ? tiles" | your-turn card; the countdown ring | listen 4 s (countdown) |
| 03_4 | Eleven. Every layer is two tiles longer than the one before, one more at the end of each arm. So the layers are the odd numbers, and they always close up into a square. | The sixth layer arrives in gold (`F-Q.next_layer.answer`). A ghost of the fifth layer sits against it: the two extra tiles, one at the end of each arm, flash | highlight; a morph from the old layer to the new | |
| 03_5 | One plus three plus five plus seven plus nine is five times five. And you can see why it would still work after a hundred layers. | The sum written out: 1 + 3 + 5 + 7 + 9 = 5 × 5 (`F-O.sum`); then 1 + 3 + 5 + … + 199 = 100 × 100 (`F-O.hundredth`, `F-O.sum.hundred`). Lower-third: "gnomon: the Greek word for this L-shaped layer (Aristotle, Physics III.4)" | a formula that grows from the numbers; lower-third | |
| 03_6 | Now look at what the picture showed, and what it took for granted. It took for granted that every layer is two longer than the last. We checked that. Remember this card. | The card, first time. **Shown:** the first n odd numbers add up to n × n. **Assumed:** each layer is 2 longer than the last, ticked, with "one more tile on each arm" | the card | hold |

- **The payoff, early** (plan rule 3): a whole proof in a minute, with the
  device that will fail in chapter 14.
- **The next size:** the step from one layer to the next is the proof.

## 04_staircases (54 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | A second warm-up. Build a staircase of tiles. One, two, three, four, five. That is fifteen tiles, but adding them one at a time is slow. | A blue staircase rises column by column: 1, 2, 3, 4, 5 (`F-T.columns`), and a counter reaches 15 (`F-T.count`) | lagged reveal; a live count | |
| 04_2 | So make a copy of the staircase, and turn it upside down. | A copper copy lifts off the first and turns half way round above it | a morph: one rigid half turn | |
| 04_3 | ⟨predict⟩ Your turn. The two staircases lock into a rectangle. How wide is it, and how tall? | The copy hovers over the first. The card: "? wide, ? tall" | your-turn card; the countdown ring | listen 5 s (countdown) |
| 04_4 | Five wide and six tall. Each column of one staircase meets a column of the other, and every pair comes to six. Thirty tiles in the rectangle, so fifteen in each staircase. | The copy settles: a rectangle 5 by 6 (`F-Q.rectangle.answer`), 30 tiles (`F-T.rectangle.cells`). Under the columns: 1 + 5, 2 + 4, 3 + 3, 4 + 2, 5 + 1 | highlight; brace labels | |
| 04_5 | It works at any height. Two staircases always make a rectangle one taller than it is wide. So the staircase is half of that rectangle. | In letters, with word labels: 1 + 2 + … + n = n (n + 1) ÷ 2. Then the hundred-step staircase: 5,050 (`F-T.count.hundred`). The card. **Shown:** 1 + 2 + … + n is half of n × (n + 1). **Assumed:** the copy fills the gaps exactly, ticked, with "column k meets column n + 1 − k". The 15 lifts off in gold and waits in a corner. Lower-third: "triangular numbers" | a formula earned from the numbers; the card | hold |

- **Why it is here:** fifteen is the side of the cubes' square, and "two
  copies" is the idea Siu takes to three.
- **Left out:** the story of young Gauss and 1 to 100 (the earliest account
  names no numbers; fact sheet §Sources).

## 05_slabs (82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Back to the cubes. A cube five tiles on a side is five slabs, each five by five. A cube of side four is four slabs. Fifteen slabs in all. | The five cubes; each comes apart into its slabs. "15 slabs" (`F-K.slabs`) joins the waiting 15 | the cubes; lagged reveal | |
| 05_2 | Lay them flat as L-shaped layers. The single tile is the corner. The two cube wraps around it with two slabs. One fits the corner of the L. The other must be cut in half. | A board. The pink tile. One copper 2 by 2 slab takes the corner of the new layer; the other splits into two halves, one at the end of each arm. The square is 3 on a side (`F-K.sides`) | morph from solid to board; highlight on the cut | |
| 05_3 | The three cube needs no cutting. One slab on the corner, one on each arm, and the square is six on a side. | Three green 3 by 3 slabs: corner, arm, arm. Side 6 | morph | |
| 05_4 | ⟨predict⟩ Your turn. The four cube has four slabs. How many of them will have to be cut? | The 6 by 6 square, the outline of the next layer, four blue slabs waiting. The card: "slabs to cut: ?" | your-turn card; the countdown ring | listen 6 s (countdown) |
| 05_5 | Just one. A slab goes on the corner. Each arm is six long, room for one and a half slabs. So two go in whole, and one is cut in half. | Blue corner slab; each arm 4 wide and 6 long (`F-K.arm.4`), labelled "6 ÷ 4 = 1½ slabs" (`F-K.arm.4.slabs`). The answer: 1 (`F-Q.cut_slabs.answer`). Side 10 | highlight; brace labels | |
| 05_6 | The five cube wraps around with no cut at all. Now the square is fifteen on a side. One plus two plus three plus four plus five. | Five violet slabs; each arm 5 by 10, two whole slabs (`F-K.arm.5.slabs`). The side braced "1 + 2 + 3 + 4 + 5", and the waiting gold 15 lands on it (`F-K.side`) | brace; morph | |
| 05_7 | Five cubes, one square. Fifteen times fifteen. Two hundred and twenty-five tiles, and not one left over. | A counter runs over the tiles to 225 (`F-K.unit_squares`). 1³ + 2³ + 3³ + 4³ + 5³ = 15² | count-up; a formula | hold |

- **The owner's first picture**, built slab by slab: 15 slabs, two of them
  cut, 17 pieces (`F-K.pieces`).
- **Misconception defused:** "the cut slabs are a fudge". The cut is forced
  and counted: an even cube's arms need half a slab each (05_5).

## 06_every_cube (65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | ⟨predict⟩ That was five cubes. Would a sixth one fit? | The 15 by 15 square with the outline of a layer 6 wide. "6³ = 216 tiles" (`F-K.sixth.cube`) waits beside it | a question card | |
| 06_2 | Its layer must be six wide. The corner takes one slab. Each arm is six wide and fifteen long, and fifteen is two and a half sixes. So the two arms take five slabs. Five and one make six. Exactly one cube. | The sixth layer builds in gold: a 6 by 6 corner, arms 6 by 15 (`F-K.arm.6`), "15 ÷ 6 = 2½ slabs" (`F-K.arm.6.slabs`). The square is 21 on a side (`F-K.sixth.side`) | highlight; brace labels; morph | |
| 06_3 | That was no accident. The arm is as long as the old square's side, and that side is a staircase. Five sixes, divided by two. So the arms always take one slab fewer than the cube has, and the corner takes the last. | The reason, numbers first and then letters with word labels: arm = 1 + 2 + … + 5 = 5 × 6 ÷ 2, which is 5 half slabs of side 6; then arm = (k − 1) × k ÷ 2 = (k − 1) half slabs; two arms = k − 1 slabs; with the corner, k slabs | a derivation that turns numbers into letters | |
| 06_4 | So every cube fits, at every size. The cubes up to any number make a square whose side is the staircase up to that number. | 1³ + 2³ + … + n³ = (1 + 2 + … + n)², boxed. The card. **Shown:** the cubes up to n³ add up to the square of 1 + 2 + … + n. **Assumed:** each arm is a whole number of half slabs, ticked, with "the staircase formula" | a formula in a box; the card | hold |
| 06_5 | This argument is about a thousand years old. Al-Karaji proved the rule with L-shaped layers around the year one thousand. Nine centuries earlier, Nicomachus had noticed that every cube is a run of odd numbers. | A line of dates: Nicomachus, about 100 (`F-X.nicomachus`); Aryabhata, 499, "states the rule" (`F-X.aryabhata`); al-Karaji, about 1000 (`F-X.al_karaji`). Then the 15 by 15 square redrawn as fifteen thin layers, the odd numbers 1 to 29, coloured in runs: 1; 3 + 5; 7 + 9 + 11; and so on (`F-K.odd_runs`). Source line: "the cut slabs: A. L. Fry, Mathematics Magazine, 1985" (`F-X.fry`) | a number line of dates; a morph: each thick layer splits into its thin ones | |

- **Earned formula** (plan rule 6): 6 and 15 first, then the letters.
- **Inevitable in hindsight** (plan rule 19): chapter 04 is the reason
  chapter 05 worked.
- **History, exactly:** Nicomachus did not state the rule that carries his
  name (Heath); "nine centuries" is `F-X.al_karaji.after_nicomachus`.

## 07_three_stacks (52 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Cubes were easy to slice. What about the squares themselves? One, four, nine, sixteen. Stack them, biggest at the bottom, every layer pushed into the same corner. Thirty cubes. | A stack builds: layers of 16, 9, 4 and 1 cubes (`F-S.layers`), one corner shared. A counter reaches 30 (`F-S.stack`) | the cubes; a live count | |
| 07_2 | ⟨predict⟩ Two staircases made a rectangle. Your turn. How many of these stacks does it take to build a solid box? | The stack and a ghost copy. The card: "? stacks make a box" | your-turn card; the countdown ring | listen 5 s (countdown) |
| 07_3 | Three. Here are three copies of the stack. The first stays put. The second tips onto its back and slides in. The third tips onto its side and settles over both. | Three equal stacks in blue, copper and pink (`F-Q.stacks.answer`). The second turns and slides against the first; the third turns the other way and comes down over them (`model::stacks::TURNS`). Each turn is shown as a turn | the cubes; rigid turns, never a mirror image (`F-S.turns`) | |
| 07_4 | They lock together with no gap. Four wide, five deep and four tall, with a step of ten cubes on top that covers exactly half of it. | The locked block with braces 4, 5, 4. On top a staircase of 10 cubes (`F-S.top`) over a top face of 20 squares (`F-S.top.layer`) | braces; highlight | hold |

- **The owner's fourth picture.** The three stacks are
  `model::stacks::placed(4, 0..3)`, cube for cube.

## 08_the_box (55 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | ⟨predict⟩ The top is exactly half full. How do you make it flat without adding a thing? | The block, its top step in gold | a question card | |
| 08_2 | Slice the top layer flat, through the middle of every cube. Lift the upper slice, turn it half way round, and lower it into the empty half. | A cut sweeps through the top layer at half height. The upper slice, 10 half cubes, lifts, turns half way round and comes down into the gap (`model::stacks::turned_half`): 20 half cubes (`F-S.top.halves`), level | the cubes; one rigid half turn | |
| 08_3 | Now it is a box. Four wide, five deep, and four and a half tall. That makes ninety. Three stacks of thirty. | Braces 4, 5, 4½ (`F-S.box`). 4 × 5 × 4½ = 90 (`F-S.box.volume`) = 3 × 30 (`F-S.three_stacks`) | braces; a formula | |
| 08_4 | At any height the box is as wide as the stack is tall, one more than that deep, and a half more than that tall. And one stack, the sum of the squares, is a third of the box. | Numbers first: 30 = ⅓ × 4 × 5 × 4½. Then, with word labels, 1² + 2² + … + n² = ⅓ n (n + 1)(n + ½). Lower-third: "precisely: the textbook form is n (n + 1)(2n + 1) ÷ 6" | a formula earned from the numbers; lower-third | hold |
| 08_5 | This picture is not ancient. Man-Keung Siu, of the University of Hong Kong, published it in nineteen eighty-four, under the title Proof Without Words. | The formula as Siu printed it. Source line: "Man-Keung Siu, Mathematics Magazine 57 (1984), p. 92" (`F-X.siu`) | a card | |

- **A "huh"** (plan rule 17): the half in n + ½ is a real slice of real
  cubes.

## 09_every_height (46 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | ⟨predict⟩ But that was a stack of four layers. Why would five fit? | The block as it was before the slice, in its three colours | a question card | |
| 09_2 | Give each stack one more layer, a five by five slab at its base. Lift the block. One slab slides underneath. One goes behind. One goes beside. | Three gold 5 by 5 slabs, one for each stack. The block rises one cube; the slabs slide under it, behind it and beside it (`model::stacks::new_base`). The result is 5 by 6 by 5 with a step of 15 on top (`F-S.grown.box`, `F-S.grown.top`); each slab takes its stack's colour | the cubes; a morph | |
| 09_3 | The block is one bigger in every direction, and nothing overlaps. It is the L-shaped layer again. A square grows by two arms and a corner. This grows by three slabs. | Beside the block, small: the flat square gaining its L. The two growths pulse together | highlight; a split picture | |
| 09_4 | So if the stacks fit at one height, they fit at the next. And at height one, three single cubes do fit. That covers every height. | Three single cubes make a box 1 by 2 by 1½ (`F-S.one.box`). A chain of blocks, 1, 2, 3, 4, 5 layers, each growing into the next. Lower-third: "the name for this kind of reason: induction" | lagged reveal; lower-third | |
| 09_5 | So this picture passes the card. | The card. **Shown:** three stacks fill a box n by n + 1 by n + ½, at every height. **Assumed:** the three stacks are the same shape, ticked, with "turned, never mirrored"; the next height fits, ticked, with "three new slabs" | the card | hold |

- **The next size, made visible:** `model::stacks::grows` checks that the
  solid of height n + 1 is the solid of height n lifted by one, plus one new
  base slab for each stack (`F-S.grows.through`).
- **For Ruth:** this is the step Siu's printed picture leaves to the reader.

## 10_halves (50 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Pictures can even add up sums that never end. Take a square of area one. Colour half of it. Then half of what is left. Then half of what is left again. | A unit square. Pieces of ½, ¼, ⅛ fill in, each labelled with its fraction (`F-H.pieces`) | lagged reveal; fractions set as mathematics | |
| 10_2 | After five pieces, thirty-one thirty-seconds of the square is coloured. The gap is one thirty-second, exactly the size of the last piece. | Five pieces. Readouts: "filled 31/32" (`F-H.filled`), "gap 1/32" (`F-H.gap`). The gap and the last piece flash as equals | live readouts from the pieces; highlight | |
| 10_3 | ⟨predict⟩ Will the gap ever close? | The corner grows to fill the frame: more pieces, and the gap still there | a question card; a zoom made by scaling the picture | |
| 10_4 | Not after any number of steps. But it does drop below any size you care to name. Under a thousandth after ten pieces. Under a millionth after twenty. | A counter of pieces: at 10 (`F-H.pieces_to.thousandth`) the gap reads 1/1,024; at 20 (`F-H.pieces_to.millionth`), 1/1,048,576 | a live counter | |
| 10_5 | So no number smaller than one is big enough to be the total. That is exactly what it means to say this endless sum equals one. | ½ + ¼ + ⅛ + … = 1. Lower-third: "precisely: after n pieces the sum is 1 − 1/2ⁿ. Geometric series". The card. **Shown:** the pieces fill the square up to a gap the size of the last piece. **Assumed:** each piece is half of what was left, ticked | a formula; lower-third; the card | hold |

- **What "equals" means** (for Ruth): no partial sum reaches 1
  (`F-H.checked.through`), and every number below 1 is passed.
- **A published version:** Warren Page, 1981 (`F-X.page`); the picture is
  older than any source found, so no one is named on screen.

## 11_a_third (70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | Now a stranger one. Cut the square into four quarters. Colour one. Leave two alone. And cut the last one into quarters again. | A unit square in four quarters. The lower left one fills blue; the upper left and lower right stay outlined; the upper right is cut in four | lagged reveal | |
| 11_2 | Inside it, do the same. Colour one, leave two, cut the last. And again, and again, without end. | Levels two to eight march into the corner (`model::series::quarters`); the first three carry their fractions ¼, 1/16, 1/64 (`F-R.pieces`) | a cascade; a zoom made by scaling | |
| 11_3 | ⟨predict⟩ Your turn. A quarter, plus a sixteenth, plus a sixty-fourth, and so on forever. What fraction of the whole square is coloured? | The card: "¼ + 1/16 + 1/64 + … = ?" | your-turn card; the countdown ring | listen 6 s (countdown) |
| 11_4 | One third. At every size there are three equal squares, one coloured and two not. So give the other two their own colours. | The answer ⅓ (`F-Q.share.answer`). Size by size, the three equal squares pulse together; the upper ones fill copper and the right-hand ones pink (`F-R.shares`) | highlight; lagged reveal | |
| 11_5 | Three colours, and each one gets a square of every size. So they share the big square equally, all but a corner that shrinks below any size you can name. Each colour holds one third. | Readouts after three sizes: each colour 21/64 (`F-R.share`), the corner 1/64 (`F-R.left`). The corner grows to fill the frame: it is the same picture again. Lower-third: "precisely: after n cuts each colour holds (1 − 1/4ⁿ) ÷ 3" | live readouts; a zoom made by scaling; lower-third | |
| 11_6 | Archimedes added quarters of quarters like these, over two thousand years ago, to measure a slice of a parabola. This picture is Sunday Ajose and Roger Nelsen's, from nineteen ninety-four. | Lower-third: "Archimedes, Quadrature of the Parabola, Prop. 23: a finite sum and its remainder". Source line: "S. A. Ajose and R. B. Nelsen, Mathematics Magazine, 1994" (`F-X.ajose_nelsen`). The card. **Shown:** three colours share the square equally, short of all of it by the uncut corner. **Assumed:** the three squares of each size are equal, ticked, with "quarters of one square" | lower-third; the card | hold |

- **The owner's second picture.** The next size is the picture itself, one
  scale down.
- **Archimedes, exactly:** his Proposition 23 is the finite sum with its
  remainder, a third of the last piece, which is `F-R.short`: each colour
  falls short of a third by a third of the corner.

## 12_the_cut_cube (79 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Pictures can do algebra too. Take a square, five on a side, and cut each side into a long part and a short part. Three and two. | A 5 by 5 square of tiles. One cut across and one cut down, each at 3 (`F-B.parts`) | a grid; lines that draw | |
| 12_2 | Four pieces. A big square, a small square and two equal rectangles. Nine, six, six and four make twenty-five. A square wearing an L-shaped layer. | The pieces 9, 6, 6, 4 (`F-B.square.each`) in the size colours by how many short sides they have; 9 + 6 + 6 + 4 = 25 (`F-B.square.total`). The L of 6 + 6 + 4 pulls away and returns. Lower-third: "(a + b)² = a² + 2ab + b²: Euclid, Elements II.4, in words" (`F-X.euclid`) | highlight; a formula; lower-third | |
| 12_3 | Now do the same to a cube. Cut all three of its sides, three and two. | A cube 5 on a side, three cuts through it | the cubes, as whole blocks | |
| 12_4 | ⟨predict⟩ Your turn. How many blocks does the cube fall into, and how many of each kind? | The cut cube, closed. The card: "? blocks" | your-turn card; the countdown ring | listen 6 s (countdown) |
| 12_5 | Eight blocks, in four kinds. One big cube. Three flat blocks. Three thin ones. One small cube. One, three, three, one. | The blocks drift apart and gather by kind, coloured by how many short sides each has: 1, 3, 3, 1 (`F-Q.kinds.answer`, `F-B.cube.kinds`) | morph that keeps identity; lagged reveal | |
| 12_6 | Twenty-seven, three eighteens, three twelves and eight make one hundred and twenty-five. Five cubed. And nothing depended on three and two. Slide the cut, and it is still eight blocks. | Volumes 27, 18, 12, 8 (`F-B.cube.each`); 27 + 54 + 36 + 8 = 125 (`F-B.cube.by_kind`, `F-B.cube.total`). The cut slides to 4 and 1 and back: every block changes size and none appears or vanishes. (a + b)³ = a³ + 3a²b + 3ab² + b³, each term in its blocks' colour | live recomputation from the cut; a formula | |
| 12_7 | And the fourth power? Nobody can draw a block with four sides. But every block of the cube appears twice, once with a long fourth side and once with a short one. One, four, six, four, one. | Two copies of the eight blocks, tagged "× 3" and "× 2", the second set off along a dashed direction, as in the owner's picture: 16 (`F-B.fourth.blocks`). Rows 1 2 1, 1 3 3 1, 1 4 6 4 1, each number the sum of the two above it (`F-B.fourth.kinds`, `F-B.fourth.from_cube`); 81 + 216 + 216 + 96 + 16 = 625 (`F-B.fourth.by_kind`, `F-B.fourth.total`). The card. **Shown:** the cut square and the cut cube, for any two lengths. **Counted, not drawn:** the fourth power. Lower-third: "Pascal's triangle" | a table built from primitives; the card; lower-third | hold |

- **The owner's third picture.** The second and third powers are pictures;
  the fourth is bookkeeping the pictures taught, and the card says so.
- **The next size** here is any pair of lengths: the cut slides (12_6).

## 13_four_triangles (83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | One more, and it may be the most famous picture in mathematics. A right triangle with shorter sides three and four. Set four copies inside a square, one in each corner. | A square 7 on a side. Four grey triangles, one in each corner (`model::pythagoras::corners`), sides labelled 3 and 4 (`F-P.legs`) | lagged reveal | |
| 13_2 | They leave a tilted square in the middle, with the triangle's long side as its side. | The tilted square in gold outline, one side marked "long side" | highlight | |
| 13_3 | Now slide the triangles. Nothing turns, nothing flips. Two pairs close into rectangles, and the space left over is two squares. One on the side of three, one on the side of four. | Three triangles slide (`F-P.slides`). What is left: a 3 by 3 square and a 4 by 4 square (`F-P.squares`) | morph: slides only | |
| 13_4 | ⟨predict⟩ Your turn. Same triangles, same big square. So how big was the tilted square? | Both arrangements side by side. The card: "tilted square = ? tiles" | your-turn card; the countdown ring | listen 6 s (countdown) |
| 13_5 | Nine plus sixteen. Twenty-five. The triangles did not change and neither did the big square, so the space left over cannot change. And twenty-five is five times five, so the long side is five. | 49 − 24 = 25, both ways (`F-P.big`, `F-P.triangles`, `F-Q.tilted.answer`). 25 = 5 × 5 (`F-P.long_side`) | a formula; highlight | |
| 13_6 | It holds for every right triangle. Change the shape, and both arrangements follow. | The two shorter sides slide through other lengths, both arrangements redrawn live; at 2 and 5 the readout is 4 + 25 = 29 (`F-P.other.tilted`), "no whole long side" (`F-P.other.whole_side`). a² + b² = c², with word labels | live recomputation from the triangle; a formula | |
| 13_7 | But this picture took a lot for granted. That the tilted shape is truly a square, and that the triangles meet in straight lines. Both are true, because a right triangle's two sharp angles make a right angle. But somebody had to check. | A corner grows to fill the frame: on the big square's straight side, the two sharp angles and the tilted corner between them. The card. **Shown:** a² + b² = c² for every right triangle. **Assumed:** the tilted shape is a square, ticked; the sides are straight, ticked, with "the two sharp angles add to a right angle". Lower-third: "Nobody knows who first drew this. A close cousin is in a third-century Chinese commentary on the Zhoubi suanjing" | a zoom made by scaling; the card; lower-third | hold |

- **Why it is here** although the owner finds it famous: it is the honest
  twin of chapter 14, the same argument with its checks done.
- **History, exactly:** never "Pythagoras's own proof", never "a Chinese
  proof from 200 BC" (fact sheet §Sources).

## 14_the_missing_square (84 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | Now watch the very same argument go wrong. Four pieces, two triangles and two L-shapes, make a triangle thirteen wide and five tall. | A grid. The four pieces in their first arrangement (`model::lie::first`), outlined 13 by 5 (`F-L.frame`) | a grid; lagged reveal | |
| 14_2 | Slide the pieces, as before. Nothing turns, nothing flips. The same triangle again. But now one square is empty. | Each piece slides (`F-L.slides`) into the second arrangement. One cell is empty (`F-L.hole.at`) and pulses red | morph: slides only; highlight | |
| 14_3 | Same pieces, same outline, and a square has vanished. If this were a proof, it would prove that thirty-two equals thirty-three. | "pieces: 12 + 5 + 8 + 7 = 32" (`F-L.pieces`, `F-L.pieces.total`); "second picture: 32 + 1 = 33" (`F-L.outline.second`) | a formula; highlight | |
| 14_4 | ⟨predict⟩ Your turn. The flaw is on the long sloping edge. Which triangle is steeper, the big one or the small one? | Both pictures, the sloping edges in ink. The card: "steeper: big or small?" | your-turn card; the countdown ring | listen 6 s (countdown) |
| 14_5 | The small one. It climbs two in five. The big one climbs three in eight. Point four against point three seven five. So the long edge is not straight. It bends. | Slopes 2/5 and 3/8 (`F-L.slopes`), 0.400 and 0.375 (`F-L.slopes.thousandths`); the answer (`F-Q.slopes.answer`). A true straight edge from corner to corner (`F-L.straight_slope`) drawn in ink. The picture stretches tenfold upward, so the bend shows | highlight; a zoom made by scaling one direction | |
| 14_6 | In the first picture the edge sags below the true line. In the second it bulges above. The sliver between them is never more than an eighth of a square thick. But its area is exactly one square. | The two outlines laid over each other (`model::lie::outline`): 32 and 33 tiles (`F-L.outline.first`, `F-L.outline.second`), a true triangle 32½ (`F-L.true_triangle`). The sliver between them in red: thickest 1/8 (`F-L.sliver.thickest`), area 1 (`F-L.sliver`) | highlight; live readouts of the areas | hold |
| 14_7 | Neither picture was ever a triangle. The trick goes back to Paul Curry, a New York magician, in nineteen fifty-three. Martin Gardner found this triangle form and published it three years later. | The card, failing. **Shown:** nothing. **Assumed:** the long edge is straight, with a red cross. Source line: "P. Curry, 1953; M. Gardner, Mathematics, Magic and Mystery, 1956" (`F-X.curry`, `F-X.gardner`) | the card, with its cross | |

- **Honest picture:** the pieces are drawn exactly, on the grid; nothing is
  fudged to hide the bend. At true scale each bend is 1/13 of a square off
  the straight line (`F-L.dip`, `F-L.bulge`), about a thirteenth of a tile.
- **The quiz is fair for Maya:** two in five against three in eight is a
  fraction comparison she can do.

## 15_what_makes_a_proof (57 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | Two pictures, one argument. Same pieces, two arrangements, compare what is left. Once it gave a famous theorem. Once it gave nonsense. | The two rearrangements side by side, small, each with its card: one ticked, one crossed | a split picture | |
| 15_2 | ⟨predict⟩ So what makes a picture a proof? | Both cards, enlarged | a question card | |
| 15_3 | Three things. Every piece is accounted for. Every fit has a reason, not just a look. And the reason does not depend on the size you happened to draw. | Three lines set themselves, large, one for each sentence. Beside each, small pictures of the proofs that leaned on it: the tiles counted, the straight edges, the next layer | a card; lagged reveal | |
| 15_4 | That is why the cubes needed a second chapter. The picture showed five cubes. The reason covered every cube. | The opening square returns. Then one layer of it alone, with every length named: width k, arm 1 + 2 + … + (k − 1), which is (k − 1) half slabs | morph; labels set as mathematics: the rigorous form of the opening picture | |
| 15_5 | A program rebuilt every picture in this video, piece by piece, up to forty cubes and twenty-four layers. That is still only checking. The proof is the reason. | Receipts: "rebuilt and checked by code: cubes up to 40 (`F-K.tiled.through`), stacks up to 24 layers (`F-S.filled.through`), every sum up to 1,000 (`F-K.sum.through`)", and under it "checking is not proof" | a receipts card | |
| 15_6 | A picture cannot check itself. But when you can say why every piece fits, it does what a page of algebra rarely does. It shows you why. | The 15 by 15 square once more, in full colour. Names to search: "proofs without words"; Roger B. Nelsen, *Proofs Without Words* (1993, 2000) (`F-X.nelsen`); Mathematics Magazine | a closing card; names to search | hold |

- **Ends on the rigorous form of the opening picture** (plan rule 15).
- **The receipts card is honest about itself:** the repository's own checks
  are checks, like the five cubes of chapter 02.
