# Storyboard — Two Shadows Can Lie

The hand-edited source of this video's words and pictures.
`python3 -I tools/storyboard.py discrete_tomography` checks it (gate G2) and
writes `script.tsv` from the words column. Every number below is a fact in
`docs/fact-sheet.md`, computed in `src/facts.rs` and recomputed by
`oracle.py`.

Words are spoken exactly as written, one thought at a time: no colons,
semicolons, dashes or parentheses in them. `*(…)*` is an annotation, never
spoken. `⟨predict⟩` opens a prediction question; the beat ends with the
question and the picture holds (a visible four-second countdown, "pause for
more time") before the next beat answers it.

## The arc

One tiny example runs through the video: **the face**, a 5 × 5 picture
(rows 01010, 01010, 00000, 10001, 01110), whose row counts 2, 2, 0, 2, 3 and
column counts 1, 3, 1, 3, 1 fit 64 pictures. The same row counts with column
counts 5, 1, 1, 1, 1 (same total) fit none. A tree (row counts 1, 3, 5, 1, 1;
column counts 1, 2, 5, 2, 1) fits exactly one, and reasoning one row or column
at a time rebuilds it; on the face's shadows the same reasoning settles only
the empty row. Rows and columns 4, 4, 1, 1, 1 fit nothing although no row or
column settles a square and each column alone has room: the two columns of four
ask for eight, the rows can give them seven. So the two questions, *does any
picture fit?* and *is it the only one?*, are kept apart from the first chapter
on, and the video ends on what a program can promise from two shadows.

The owner's 2026-10-20 direction is a complete episode of 10 to 15 minutes;
this storyboard is the accepted eleven-chapter expansion
(`narration_lab/.../duration-and-storage-20261020/PinkDuck/EXPANSION.md`,
root's decisions of 2026-10-20). Chapter lengths below are estimates, not
measurements; no narration of the new lines exists.

| Chapter | Spine | Status |
|---|---|---|
| `01_same_shadows` | the face and its two shadows; hidden, could another picture cast them? (prediction); the switch; 64 pictures within 3 switches (Ryser's theorem on the card); the face's rows with columns 5, 1, 1, 1, 1 fit nothing; the two questions | unchanged since root's verification (`.9.1`, 6bae9ea) and the F2/F6 repair (`ec5eb74`); narrated in review01 (unaccepted) |
| `02_square_by_square` | why shadows (Batenburg's nanocrystal motivation, scoped); the title; reasoning one row or column at a time settles every square of the tree; on the face it settles the empty row and nothing else (prediction); twenty squares filled in some of the 64 and empty in others | new source, 2026-10-20, uncompiled |
| `03_water_through_the_grid` | trying every picture (prediction: how many?) does not scale; the plumbing; both directions of the water's promise; one unit at a time, slowed so each is followed | 02_1 to 02_4's source kept; new beats uncompiled |
| `04_taking_water_back` | stuck at eight (prediction); take back; it keeps every count; in the picture one square empties and two fill; the water always finishes; one of the 64; never says whether the only one | 02_5 to 02_9's source kept; new beats uncompiled |
| `05_where_the_water_stops` | the face's rows with 5, 1, 1, 1, 1: stops at eight (prediction); where the water can still reach; every unit must leave it through four pipes or four drains; one column asks too much | 03_1 to 03_3's source kept; new beats uncompiled |
| `06_a_group_that_asks_too_much` | 4, 4, 1, 1, 1: no square forced, each column alone has room (prediction: does a picture fit?); the two columns of four ask eight, the rows can give seven; the water stops at ten and cuts off exactly that pair; Gale's table; why the hungriest group of each size suffices | new source, uncompiled |
| `07_why_the_test_never_misses` | the cut argument on 06's numbers, 3 + 4 + 3; whenever the water stops short; the Gale card; equal totals (prediction: rows 2, 2 with columns 1, 1); the face passes | 03_7, 03_8's words kept; new source, uncompiled |
| `08_the_only_picture` | twelve switches; the tree's water (prediction, a callback to 02); nesting; a switch is what stopped the reasoning; about one 4 × 4 picture in ten is alone | 04_1 to 04_6's source kept (F3, F5); new beats uncompiled |
| `09_chains_of_switches` | two 3 × 3 pictures six squares apart (prediction: one switch?); two switches; Ryser's theorem as the tree's missing step; the 64 reached by switches, a way to list them | new source, uncompiled |
| `10_more_shadows` | the diagonal shadow (05_1 to 05_6, F1/F2); 48 of the 64 alone with three shadows, not the face; the mirror and the other diagonals; both diagonals single out the face, this picture only | 05_1 to 05_6's source kept; new beats uncompiled |
| `11_what_two_shadows_promise` | four steps a program can follow; the face (prediction: what does it report?); what no program can say; the two questions, the bounded model | new source, uncompiled |

## Vocabulary (one word per idea)

**picture** (never "image" or "matrix" in the voice); **grid**, **square**
(never "pixel" or "cell"); **filled** and **empty** (never 1 and 0 in the
voice); **count** (how many filled squares one row or column holds); **shadow**
(all the counts in one direction: the row shadow, the column shadow; never
"projection" or "sum" in the voice); **cast** (a picture casts its shadows);
**fit** (a picture fits shadows when its counts are theirs); **switch** (two
rows, two columns, corners alternating filled and empty, swapped; the
literature's "interchange" or "switching component" is named in the
description, not the video); **total** (all the squares one shadow counts).
From chapter 02: **reasoning** (settling a square from one row's or one
column's count alone), **sure** and **decided** and **settle** (a square every
picture that fits agrees on), **guess** (a choice the shadows do not force).
From chapter 03: **tap** (where a row's water comes in; never "source"),
**pipe** (a square seen as plumbing), **drain** (where a column's water
leaves; never "sink"), **unit** (one unit of water: one filled square),
**wet** and **dry** (a pipe carrying a unit or not), **path** (one unit's way
from a tap to a drain), **room** (what a pipe can still carry), **take back**
(water pushed back out of a pipe; the literature's "reverse edge" or
"residual capacity" is named in the description, not the video), **search**
(one breadth-first look for a path). From chapter 05: **area** (everywhere the
water can still reach, drawn in teal). From chapter 06: **group** and **pair**
(columns taken together). From chapter 09: **chain** (switches one after
another). From chapter 10: **mirror** (the face reversed left to right). From
chapter 11: **program** (any procedure that sees only the shadows).

## Color roles

| Idea | Color |
|---|---|
| a filled square, in every picture and every thumbnail | blue `#58C4DD` |
| a switch: its rectangle, its corners' rings, its word | `hero` gold `#FFD866`, strokes and words only |
| a route along rows and columns: a beam counting (01, 02), a unit of water's path (03 on), where the water can still reach (05, 06) | teal `#5CD0B3`, strokes only |
| water taken back out of a square, and the way it goes back (04) | purple `#9A72AC` |
| a count that matches the face's, a line reasoning has settled (02) | green `#83C167` |
| what no picture can do: the count it fails, the cross, "no picture fits" | red `#FC6255` |
| counts, questions | `ink` |
| grid lines, labels, tags, squares reasoning has emptied (02) | `faint` and `dim` |

## 01_same_shadows: The same shadows (~83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Here is a little face on a grid of squares. | A five by five grid of faint lines; on "a little face", the face's nine blue squares appear one by one *(face)* | lagged reveal, grid | |
| 01_2 | Count the filled squares in each row, and in each column. Those counts are the picture's two shadows. | Teal beams run along the rows one after another and leave each row's count at its right end, 2, 2, 0, 2, 3 *(face.rows)*; on "and in each column", beams run down the columns and leave 1, 3, 1, 3, 1 below them *(face.cols)*; on "two shadows", the dim labels "row shadow" and "column shadow" | lagged reveal | |
| 01_3 | Hide the picture and keep the shadows. ⟨predict⟩ Could a different picture cast the same shadows? | The blue squares fade; the empty grid and both rows of counts stay; at the question's end a gold "?" in a draining ring, "pause for more time" under it, four seconds, the voice silent | tracker | |
| 01_4 | It can. Every row count matches, and every column count matches. | The grid and its counts slide left and the face returns, "the face" above it; "another picture" appears on the right *(twin)*; on "Every row count", the face's row counts flash and the other picture's appear beside it one by one, green; on "every column count", the same for the columns | lagged reveal, highlight | |
| 01_5 | They differ in four squares, the corners of a rectangle, with filled and empty corners alternating. | On "four squares", gold rings on the four squares in both pictures, joined by a thin gold rectangle *(switch, differ.cells)*; on "alternating", the rings flash in turn around the rectangle, filled, empty, filled, empty | highlight | |
| 01_6 | Swap filled and empty at those corners. Each of their rows and columns loses one square and gains one. | On the right, the two filled corners fade as the two empty corners fill: the other picture becomes the face *(switch)*, then its name changes to "now the face" with sequential 0.15-second fades; on "loses one square", a dim "−1 +1" by each of the two rows' and two columns' counts, which flash *(loses, gains)* | highlight | |
| 01_7 | That move is a switch. It changes the picture, but neither shadow. | The tags go; the gold word "switch" beside the right-hand rectangle; on "neither shadow", every count on screen flashes | highlight | |
| 01_8 | How many pictures cast these shadows? Sixty-four, and each is at most three switches from the face. | The right-hand picture, the marks and the names go; on the right a gallery of every picture with these shadows fills in four waves, the face (white frame) first, then the 14 one switch away, the 33 two away, the 16 three away *(fits, rings)*, finishing on "Sixty-four" with "64 pictures" above it; on "at most three", the caption becomes "64 pictures, each within 3 switches of the face" *(farthest)* and the big face takes three switches, each one's gold rectangle flashing as its corners swap and the thumbnail it reaches lighting *(walk)*; a "precisely" lower-third: any two pictures with the same row and column counts are joined by a chain of switches (Ryser, 1957) *(ryser)* | lagged reveal, highlight, lower-third | |
| 01_9 | Change the column counts but keep their total, and no picture fits at all. | The gallery, caption and card go and the big picture empties; the column counts change to 5, 1, 1, 1, 1 *(no_fit.cols)*; on "keep their total", "both shadows add up to 9" *(no_fit.totals)*; on "no picture fits", the same words in red *(no_fit.fits)* | highlight | |
| 01_10 | This column needs a square in every row, but the middle row is empty. | The first column's 5 turns red and white rings mark its five squares, one in each row *(bottleneck.asks)*; on "the middle row", that row's 0 turns red and a red cross falls on the ringed square in it *(empty_rows, bottleneck)* | highlight | |
| 01_11 | So shadows raise two questions. Does any picture fit? And is it the only one? | On the right, "Does any picture fit?" as it is asked, then "Is it the only one?" *(questions)*; a held beat, then the fade | | |

### Notes for 01

- **Prediction (01_3).** The question ends narration unit 01u1 and the line.
  The think ring drains for four seconds with the voice silent; the answer
  "It can." opens unit 01u2. The other picture appears only after "It can"
  begins (the quiz rule: no answer on screen during the think time).
- **Question and answer in one breath (01_8).** "How many pictures cast these
  shadows?" is rhetorical, not a prediction: its answer follows in the same
  unit (01u3). Direction for the take: a natural question, then a short
  thinking gap before "Sixty-four" (about 0.6 to 0.8 s of total silence,
  counting the take's own), not a think window. The gallery's last wave lands
  with "Sixty-four"; the number appears only then.
- **The closing questions (01_11).** Two questions for the chapters to come,
  not a prediction: a short gap between them (about 0.4 s), then a held beat
  of 1.2 s before the fade.
- **Exact claims.** "It changes the picture, but neither shadow" holds for
  every switch (`DT-X1`, every switch of every 4 × 4 picture; `model` tests).
  "Each is at most three switches from the face" is this picture's
  breadth-first distance (`DT-F3.farthest`), not a general bound. The card
  states Ryser's Theorem 3.1 exactly (any two pictures with equal row and
  column counts are joined by a finite chain of switches); it is checked on
  every 4 × 4 pair of shadows (`DT-X2`). "No picture fits" is enumeration
  (`DT-F4.fits` = 0) with the visible reason: the column asks for 5, one from
  each row, and only 4 rows hold any square (`DT-F4.bottleneck.can_give`).
- **Scope.** The video's pictures are grids of filled and empty squares and
  its shadows are counts along rows and columns. The beams are a picture of
  counting, not a claim about how any scanner works.
- **Budget.** 161 spoken words. By the measures the Golay chapters used (the
  kit's scratch voice at 2.6 words a second plus 0.4 s a line, thought gaps
  about 0.82 s, a think window about 5.2 s, a tail of 2 s) about 83 s; the
  narrated draft measures the real voice.
- **Narration units.** `narration_units.json` groups the lines into four
  whole thoughts (01u1 to 01u4) as the text plan for a future take. It
  selects no synthesis route: the owner rejected the first-wave narration on
  2026-10-19, and nothing from this lane has been synthesized.

## 02_square_by_square: Square by square (~76 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | This puzzle has a serious side. One reason people study it is rebuilding tiny crystals from counts of atoms along lines, often taken from only a few directions. | The title "Two Shadows Can Lie" at the top as an empty five by five grid fades in; teal beams sweep three rows and two columns, as in 01_2, and leave no counts (counting, not a scanner); on "tiny crystals", the title goes and a "precisely" lower-third: counting atoms along columns through a nanocrystal from few images (Batenburg, 2006, section 1.9.1), here flat grids and exact counts, no microscope *(batenburg)*; it stands its characters / 15 seconds from its fade-in, into 02_2. No picture of a crystal | lagged reveal, lower-third, grid | |
| 02_2 | How would you rebuild a picture from its shadows? Start with the squares you can be sure of. | The card stays; on "squares you can be sure of", a legend on the right: a blue square "sure: filled", a small dim dot "sure: empty" | | |
| 02_3 | Take the shadows of a tree. Rows of one, three, five, one and one, columns of one, two, five, two and one. | The card goes once read; on "Rows of", 1, 3, 5, 1, 1 right of the rows *(tree.rows)*; on "columns of", 1, 2, 5, 2, 1 under the columns *(tree.cols)*; every square undecided | lagged reveal | |
| 02_4 | The row of five must be full, and so must the column of five. | On "The row of five", the middle row's five squares fill left to right and its 5 turns green; on "the column of five", the middle column's other four fill top to bottom and its 5 turns green *(reason.tree.full)* | lagged reveal, highlight | |
| 02_5 | That already gives the first and last columns their one square, and the rows of one theirs. Everything else in them stays empty. | On "the first and last columns", their 1s turn green and dim dots fall on their other four squares; on "the rows of one", the three 1s turn green and dots fall on their remaining squares *(reason.tree.met)* | lagged reveal, highlight | |
| 02_6 | The second row needs two more, and exactly two squares are left open. Every square is decided, and the picture is the tree. | On "exactly two squares", the second row's two open squares ring white, then fill; every count green; on "the picture is the tree", "every square decided" above the grid *(reason.tree.last, reason.tree.solved)* | highlight | |
| 02_7 | Now the face's shadows. Its empty row is easy. All five squares there are empty. | The tree clears; the face's counts 2, 2, 0, 2, 3 and 1, 3, 1, 3, 1 round the empty grid *(face.rows, face.cols)*; on "Its empty row", the 0 turns green and dots fall on the middle row's five squares *(reason.face.steps)* | highlight | |
| 02_8 | Then what? ⟨predict⟩ Can you be sure of any other square? | The think ring, four seconds, the voice silent, nothing moving | tracker | |
| 02_9 | No. Every other square is filled in some of the sixty-four pictures, and empty in others. | The ring goes; on "Every other square", each of the twenty undecided squares shows how many of the 64 have it filled, shaded by that share: 12, 46, 12, 46, 12 in the first, second and fourth rows, 28, 54, 28, 54, 28 in the bottom row; "filled in how many of the 64" on the right *(reason.face.filled_in, reason.face.free)* | highlight | |
| 02_10 | So reasoning row by row and column by column gets stuck here. Whatever you guess next, some of the sixty-four agree. | On "Whatever you guess", the top-left square rings white; on the right "filled: 12 of 64" and "empty: 52 of 64" *(reason.face.guess)*; a held beat, the fade | highlight | |

### Notes for 02

- **The reasoning is the model's own.** `model::line_solve` sweeps the rows
  top to bottom and then the columns left to right, applying to each line
  the first of two rules that fires (a count its filled squares already meet
  empties the line's undecided squares; a count that needs every undecided
  square fills them), until a whole sweep settles nothing; it stops at a
  line whose count cannot be met. On the tree it makes eight deductions,
  `r2+5, c0-4, c2+4, c4-4, r0-2, r1+2, r3-2, r4-2` (`DT-F12.tree.steps`), and
  settles every square. The scene voices them grouped (the two full lines;
  the lines whose count is met; the last row) and checks with
  `model::deduce` that each, in that order, is the same deduction.
- **The face.** The same rules settle only the middle row (`r2-5`,
  `DT-F12.face.steps`), with no contradiction. By enumeration the only
  squares alike in all 64 pictures are those five (`DT-F12.face.forced`), so
  "Every other square is filled in some … and empty in others" is exact;
  each of the twenty is filled in 12 to 54 of the 64 (`DT-F12.face.filled_in`)
  and empty in at least 10, so any one next guess agrees with some of the 64.
  Nothing is claimed about a run of guesses, nor about reasoning in general.
- **Prediction (02_8).** The question ends unit 02u3; the think ring drains
  for four seconds, the voice silent; "No." opens 02u4. No number appears
  before "No."
- **02_1's source.** Batenburg 2006, §1.9.1, pp. 31–32 (read 2026-10-20 by
  PinkDuck; verified by root): a technique "made it possible to count the
  number of atoms in each projected column of a nanocrystal", rebuilding the
  atoms' positions from such counts was "one of the main envisioned
  applications", and "recording many images is often impossible". The line
  claims only that; the card keeps the video's exact-grid scope. The card
  writes "section" (no "§" glyph is probed).
- **The title.** "Two Shadows Can Lie" stands from the chapter's first frame
  until the card replaces it on "tiny crystals"; it adds no time.
- **Narration units.** 02u1 (02_1, 02_2), 02u2 (02_3 to 02_6), 02u3 (02_7,
  02_8), 02u4 (02_9, 02_10). A text plan; nothing is synthesized.

## 03_water_through_the_grid: Water through the grid (~62 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Why not try every picture? Each square is filled or empty. ⟨predict⟩ How many pictures does this grid have? | The empty five by five grid on the left; on "Each square is filled", one square fills and empties again; the think ring, four seconds, the voice silent | tracker | |
| 03_2 | Two choices for each of twenty-five squares, over thirty-three million. On a twenty by twenty grid, the count has a hundred and twenty-one digits. Trying every picture doesn't scale. | The ring goes; on "Two choices", "2 choices for each of 25 squares" on the right; on "over thirty-three million", "33,554,432 pictures" *(count.pictures)*; on "twenty by twenty", "20 by 20: a count 121 digits long" *(count.big)* | | |
| 03_3 | Here's a method that never guesses. Turn the grid into pipes. Each row's tap lets out its count in units of water. | On "Turn the grid", the words go; on "Each row's tap", a counter left of each row, 2, 2, 0, 2, 3 *(face.rows)*, under the dim label "to send"; on the right, "tap: lets out its row's count" | lagged reveal, grid | |
| 03_4 | A unit runs along its row, turns down through a square, and drains out of that column. | storyboard 02_2 of the five-chapter cut: the drain counters 1, 3, 1, 3, 1 *(face.cols)*, "to drain"; the method's first real unit *(trace, path 1)*: a drop leaves the top row's tap, runs in, the square fills on "turns down", and on "drains out" it falls into the first drain | lagged reveal, motion | |
| 03_5 | A square carries at most one unit, and each drain holds at most its count. Get every unit through, and the wet squares are a picture that fits. | as before: the wet square flashes, "square: carries at most 1 unit" *(square)*; on "each drain holds", the drains flash, "drain: holds at most its column's count"; on "Get every unit through", "every unit through: a picture that fits" and "through: 1 of 9" | highlight | |
| 03_6 | It works the other way too. Any picture that fits gets every unit through, one down each filled square. | The legend goes; on the right a small copy of the face drawn as plumbing, its own taps 2, 2, 0, 2, 3 and drains 1, 3, 1, 3, 1; on "one down each filled square", nine drops run at once, each along its row to one of the face's squares and down; its taps and drains read 0; "the face: every unit through" *(face)* | motion | |
| 03_7 | So asking whether any picture fits is asking whether every unit can get through. | The copy goes; on the right "Does any picture fit?" and under it "Can every unit get through?", joined by a green "the same question" | highlight | |
| 03_8 | Send one unit at a time, along any path with room. | The words give way to the "for programmers" lower-third (each unit takes a path found breadth first: in from a tap with water left, forward into dry squares, back out of wet ones, down into a drain with room); units 2 to 8 of the real trace at about 0.7 s each, so each can be followed *(trace, paths 2 to 8)*; the card stands until read, then the fade | lower-third (code), highlight | |

### Notes for 03

- **The count (03_1, 03_2).** A square is filled or empty (two choices,
  `DT-F13.choices`), the grid has 25 squares, so 2^25 = 33,554,432 pictures
  (`DT-F13.pictures`; "over thirty-three million", `DT-F13.millions`); a 20
  by 20 grid has 2^400, a number 121 digits long (`DT-F13.big.digits`,
  computed exactly by repeated doubling). "Doesn't scale" is the claim, not
  "cannot be done": a computer can try 33 million pictures.
- **Prediction (03_1).** The viewer can answer by the counting rule: two
  choices, twenty-five times over. The question ends unit 03u1; "Two
  choices" opens 03u2; no number appears before it.
- **Both directions (03_5, 03_6).** A full flow's wet squares are a picture
  that fits; any picture that fits is a full flow, one unit down each filled
  square (Gale 1957, pp. 1079–1080, read; the fact sheet's non-numeric
  claims). So "does any picture fit?" is "can every unit get through?".
- **The units (03_8).** `send_paced` runs the real paths at a pace of 1.8
  (about 0.7 s a unit); chapters 05, 06 and 08 keep the quick pace.
- **The trace is the real one** (unchanged from the five-chapter cut's
  chapter 02): `model::max_flow` records every augmenting path,
  `model::replay` rebuilds and checks every state; the oracle derives the
  same paths with its own bookkeeping.
- **Narration units.** 03u1 (03_1), 03u2 (03_2, 03_3), 03u3 (03_4, 03_5),
  03u4 (03_6, 03_7), 03u5 (03_8).

## 04_taking_water_back: Taking water back (~76 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Eight are through. The bottom row has one left, but its dry squares lead to full drains. ⟨predict⟩ Is the water stuck? | Chapter 03's last state fades in: the eight wet squares, every counter, "to send", "to drain", "through: 8 of 9" *(before.through)*; on "The bottom row", its tap's 1 flashes *(stuck.left)*; on "its dry squares", teal routes from the bottom tap through its three dry squares to their drains, all at 0, which flash *(stuck.dry_full)*; the think ring, four seconds, the voice silent | highlight, tracker | |
| 04_2 | No. Send it into the first column anyway, and take back the unit the top row sent there. | as before: on "Send it into", a drop leaves the bottom tap and waits at the bottom-left square; on "take back", in one move, that square fills, a purple route runs up the first column, and the top row's square there empties into a purple drop *(trace, path 9; takeback.column, takeback.row)* | motion, highlight | |
| 04_3 | The top row sends that unit to the fourth column, which still needs one. Without taking back, this fill would have stopped one unit short. | as before: the purple drop runs to the fourth square, which fills, and falls into the fourth drain; every counter 0, "through: 9 of 9"; on "Without taking back", dim "without taking back: 8 of 9" *(takeback.to_column, takeback.to_needs, through, short)* | motion | |
| 04_4 | Taking back breaks no count. The top row still sends two, the first column still drains one, and the fourth column gets the unit it was missing. | On "The top row still sends two", "top row: sends 2" on the right, green, as the top tap flashes; on "the first column still drains one", "first column: drains 1", as its drain flashes; on "the fourth column gets", "fourth column: 2 then 3", as its drain flashes *(reroute.counts)* | highlight | |
| 04_5 | In the picture, one square emptied and two filled. The bottom row took over the first column, and the top row moved to the fourth. | On "one square emptied", a purple ring on the top-left square; on "two filled", blue rings on the bottom-left square and the top row's fourth *(reroute.emptied, reroute.filled)*; on "took over", a short teal route from the bottom tap to the first column; on "moved to the fourth", one from the top tap to the fourth column | highlight | |
| 04_6 | Every search that finds a path sends one more unit, and there are only nine to send. So the water always finishes. | The rings, routes and words go; "through: 9 of 9" flashes; a "for programmers" lower-third: every path sends one unit; at most 9 send water (Ford and Fulkerson, 1957) *(paths, ford_fulkerson)*, standing its characters / 15 seconds from its fade-in | lower-third (code), highlight | |
| 04_7 | The wet squares cast the face's shadows, but they aren't the face. They're one of the sixty-four. | as before: the counters become the wet picture's counts, green; on "but they aren't", white outlines on the face's squares, "outlined: the face" *(built.is_face)*; on "They're one of", chapter 01's gallery, the water's picture framed green, "the water's picture is one of 64" *(built, built.fits)* | lagged reveal, highlight | |
| 04_8 | Water finds a picture whenever one exists. It never says whether that's the only one. | as before: the "precisely" card (Ford and Fulkerson, 1957; Gale, 1957) *(ford_fulkerson, gale)*; on "It never says", the gallery flashes; a held beat, the fade | lower-third, highlight | |

### Notes for 04

- **Taking back keeps every count (04_4, 04_5).** Before the ninth path the
  wet squares are 11000/01100/00000/01010/00011 (`DT-F8.before.wet`); after
  it, 01010/01100/00000/01010/10011 (`DT-F8.built`). Square (0, 0) empties,
  (0, 3) and (4, 0) fill (`DT-F14.emptied`, `DT-F14.filled`); the top row
  sends 2 before and after, the first column drains 1, the fourth goes from
  2 to 3 (`DT-F14.*`). "Took over" and "moved to" describe exactly those
  three squares.
- **The water always finishes (04_6).** Each search that finds a path sends
  one unit (`DT-F8.room`), and there are only as many units as the total
  (9 here): at most nine searches send water and one more finds none
  (`DT-F14.searches` = 10). Ford and Fulkerson 1957, pp. 212–213 (read:
  termination because each step sends a positive whole number). The card
  carries the short form root chose; the proof is in these notes and the
  facts.
- **Prediction (04_1).** The question ends unit 04u1; "No." opens 04u2.
  Nothing moves the ninth unit before "No."
- **The opening state.** The chapter opens on the replayed state after
  eight paths (`model::replay`), drawn from the facts, not carried over
  from chapter 03's frames.
- **Narration units.** 04u1 (04_1), 04u2 (04_2, 04_3; the five-chapter cut's
  02u3), 04u3 (04_4 to 04_6), 04u4 (04_7, 04_8; the old 02u4).

## 05_where_the_water_stops: Where the water stops (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Back to the shadows no picture fits. The face's rows, and columns asking for five, one, one, one and one. | as before: the empty grid, the taps 2, 2, 0, 2, 3 *(face.rows)*; on "and columns asking", the drains 5, 1, 1, 1, 1 *(no_fit.cols)* and "through: 0 of 9"; the method's real units, eight *(no-fit trace)* | lagged reveal, highlight | |
| 05_2 | The water stops at eight of nine. ⟨predict⟩ Could taking water back get the ninth through? | as before: on "stops at eight", "through: 8 of 9", the bottom tap's 1 and the first drain's 1 flash *(flow, left)*; the think ring, four seconds, the voice silent, nothing moving | highlight, tracker | |
| 05_3 | No. The first column needs five, but each row sends it at most one, and only four rows have water. | as before: a red outline round the first column and "asks 5" *(first.needs)*; a ring on each of its five squares; a red cross on the middle row's square there *(rows_with_water)* | highlight | |
| 05_4 | Look at everywhere the water can still reach, taking back included. Every row with water, and every column but the first. | The rings go; on "Look at everywhere", teal strokes along every reached row (over the reached columns only, clear of the red outline) and every reached column, "where the water can still reach, taking back included" *(reached)* | highlight | |
| 05_5 | Every unit that gets through must leave that area somewhere. Down a pipe into the first column, or out through one of the other drains. | On "must leave that area", the teal strokes flash; on "Down a pipe", the four wet squares in the first column flash *(cut.into_unreached)*; on "out through", the four other drains flash *(cut.reached_drains)* | highlight | |
| 05_6 | Four pipes lead into the first column, and the other drains hold four. Taking back only moves water around inside, so eight is all that can pass. | On "Four pipes", "4 pipes into the first column"; on "the other drains", "4 out of the other drains"; on "eight is all", "4 + 4 = 8, all that can pass" *(cut)* | highlight | |
| 05_7 | Here, the reason was a single column asking too much. | The strokes and words go; the red outline, "asks 5" and a new "rows with water: 4" flash *(first.needs, rows_with_water)*; a held beat, the fade | highlight | |

### Notes for 05

- **The stop and the cut are the method's own** (as in the five-chapter
  cut's chapter 03): the eight units are `facts::no_fit_flow`'s paths; the
  rows and columns the labeling still reaches are rows 0, 1, 3, 4 and
  columns 1 to 4 (`Flow::cut`; Ford and Fulkerson's Lemma 2). The only tap
  the water cannot reach is the middle row's, count 0
  (`DT-F9.cut.unreached_taps`), so every unit leaves the reach through one of
  the four pipes into the first column or one of the four reached drains:
  "Down a pipe … or out through one of the other drains" is exact here.
- **Prediction (05_2).** The question ends unit 05u1; "No." opens 05u2.
- **Narration units.** 05u1 (05_1, 05_2; the old 03u1), 05u2 (05_3), 05u3
  (05_4 to 05_6), 05u4 (05_7).

## 06_a_group_that_asks_too_much: A group that asks too much (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Is checking each column on its own always enough? Try rows of four, four, one, one and one, and columns the same. | The plumbing again, empty; on "rows of four", taps 4, 4, 1, 1, 1 under "to send"; on "columns the same", drains 4, 4, 1, 1, 1 under "to drain", and "through: 0 of 11" *(stall.rows, stall.cols, stall.total)* | lagged reveal, grid | |
| 06_2 | No count is zero or five, so no row or column forces a single square. And each column of four has five rows with water. ⟨predict⟩ Does any picture fit? | On "No count", "every count is between 1 and 4" on the right *(stall.counts)*; on "each column of four", "each column of 4: 5 rows with water", green *(stall.rows_with_water)*; the think ring, four seconds, nothing moving | highlight, tracker | |
| 06_3 | No. Take the two columns of four together. They ask for eight squares. | The ring goes; one red outline around the first two columns together, "ask 8" under them *(stall.pair, stall.pair.asks)* | highlight | |
| 06_4 | A row of four can give that pair at most two, one in each column. A row of one gives at most one. Two, two, one, one and one make seven. | On "A row of four", rings on the two squares of each row of four inside the pair and "2" beside those rows; on "A row of one", "1" beside each of the last three rows; on "make seven", "2 + 2 + 1 + 1 + 1 = 7, they ask 8" *(stall.pair.by_row, stall.pair.gives)* | highlight | |
| 06_5 | Eight asked, seven possible. No picture fits, and those single-row and single-column checks didn't catch it. | "no picture fits" in red *(stall.fits)* | highlight | |
| 06_6 | The water agrees. It stops at ten of eleven, and the columns it can't reach are exactly those two. | The rings and row numbers go; the method's real ten units, none taking water back *(stall.trace)*, "through: 10 of 11"; on "can't reach", teal strokes along the reached rows (the top two, over the last three columns) and the reached columns, the red outline flashing *(stall.reached, stall.cut_off)* | motion, highlight | |
| 06_7 | Count any group of columns this way. Each row gives it at most one square per column, and no more than its count. | The words go; on the right Gale's table for these shadows: "k" 1 to 5 and "rows can give"; on "Each row gives", "a row gives k columns at most min(its count, k)" and 5, 7, 9, 11, 11 *(stall.gives)* | table, lagged reveal | |
| 06_8 | If the group asks for more than the rows can give, no picture fits. Checking the hungriest group of each size is enough. | "largest columns first", 4, 4, 1, 1, 1, and "the k largest ask", 4, 8, 9, 10, 11 *(stall.sorted, stall.asks)*; on "no picture fits", size 2's 8 and 7 turn red *(stall.failing)*; on "the hungriest", the largest-first line flashes | table, highlight | |
| 06_9 | For a given size, the rows can give every group the same. So the hungriest group is the one to check. | On "every group the same", two other pairs light in turn under the table, the first and third columns ("ask 5, rows can give 7") and the third and fourth ("ask 2, rows can give 7") *(stall.other_pairs)*; on "the hungriest", the largest-first line flashes; a held beat, the fade | table, highlight | |

### Notes for 06

- **The example (`model::STALL_ROWS`, `STALL_COLS`).** Rows and columns 4,
  4, 1, 1, 1, both totals 11. Every count is between 1 and 4
  (`DT-F15.counts`), so neither of chapter 02's rules fires on any line
  (`DT-F15.settled` = 0, no contradiction): "no row or column forces a single
  square" is exact. Each column of four has 5 rows with water, so each
  column alone passes the single-column check. Nothing fits (`DT-F15.fits` =
  0, enumeration). The first two columns ask 8; a row gives that pair at
  most min(count, 2): 2, 2, 1, 1, 1, so 7 (`DT-F15.pair.*`). Gale's table
  fails only at size 2 (`DT-F15.failing`).
- **06_5, scoped.** "those single-row and single-column checks didn't catch
  it" names the two checks the viewer has run (chapter 02's rules, and each
  column against the rows with water); it is not a claim about every
  possible method (root's wording, 2026-10-20).
- **The water's stop is the method's own.** `facts::stall_flow` replays the
  paths `model::max_flow` sends on these shadows: ten units, none taking back
  (`DT-F15.trace`; the oracle derives its own). The rows and columns it still
  reaches are rows 0, 1 and columns 2, 3, 4 (`Flow::cut`), so the columns cut
  off are exactly the first two (`DT-F15.cut_off`), the bottleneck. The scene
  draws whatever trace the model computes and checks the cut against the
  bottleneck; the reach does not depend on the paths taken.
- **06_9.** For a fixed size k the rows can give any k columns the same,
  Σ min(r, k); for pairs, 7. The pairs shown, columns {0, 2} and {2, 3}, ask
  5 and 2 (`DT-F15.other_pairs`); so the hungriest pair, the two largest
  columns, is the one to check.
- **Prediction (06_2).** The viewer has two honest checks that both pass;
  the answer needs the new idea. The question ends unit 06u1; "No." opens
  06u2.
- **Narration units.** 06u1 (06_1, 06_2), 06u2 (06_3 to 06_5), 06u3 (06_6),
  06u4 (06_7 to 06_9).

## 07_why_the_test_never_misses: Why the test never misses (~74 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | With equal totals, that test never misses. When water stops short, the columns it can't reach always form a group that asks too much. | Chapter 06's stopped water fades in: ten wet squares, the counters, "through: 10 of 11", the reach strokes and the red outline round the first two columns *(stall)*; on "the columns it can't reach", the outline flashes | highlight | |
| 07_2 | Here's why, with the shadows we just tried. Those two columns can only be fed from the rows the water can't reach, which hold three units. Or through the four pipes into them from the rows it can. | On "the rows the water can't reach", the last three taps flash and "rows it can't reach: 3 units" on the right; on "the four pipes", white rings on the four squares where the top two rows meet the first two columns, "pipes into the pair: 4" *(stall.cut.unreached_taps, stall.cut.pipes)* | highlight | |
| 07_3 | The water used every one of those, and the three drains it can reach, and still came up one short. So those columns ask for more than three plus four. | On "used every one", "through: 10 = 3 + 4 + 3" as the three reached drains flash; on "one short", "one short of 11"; on "more than three plus four", "they ask 11 − 3 = 8, more than 3 + 4 = 7" *(stall.cut, stall.short)* | highlight | |
| 07_4 | That count works whenever the water stops short, on any shadows with equal totals. | On "whenever", the "precisely" lower-third: a picture fits exactly when row and column totals match and, for every k, the k largest columns ask for at most the sum over rows of min(row count, k) (Gale, 1957; Ryser, 1957) *(gale, ryser)*, standing its characters / 15 seconds from its fade-in | lower-third, highlight | |
| 07_5 | Equal totals matter. Rows of two and two, with columns of one and one, pass every size of the test. ⟨predict⟩ Does a picture fit? | The card and the board go; a two by two grid with taps 2, 2 and drains 1, 1 *(unequal.rows, unequal.cols)*; on "pass every size", its table on the right, "the k largest ask" 1, 2 against "rows can give" 2, 4, both green *(unequal.failing)*; the think ring, four seconds | table, tracker | |
| 07_6 | No. Together the rows ask for four squares, and the columns only two. | The ring goes; on "four squares", "rows: 4, columns: 2" in red under the table, "nothing fits" *(unequal.totals, unequal.fits)* | highlight | |
| 07_7 | The face's shadows pass, and sixty-four pictures fit them. The test says that one fits, not which, and not whether only one. | The small grid and its table go; the face's taps and drains on the five by five grid and its table, all green: 3, 3, 1, 1, 1; 3, 6, 7, 8, 9; 4, 8, 9, 9, 9 *(face.sorted, face.asks, face.gives)*; on "sixty-four", "64 pictures fit these shadows" *(fits)*; on "The test says", "it says one fits, not which, not whether only one"; a held beat, the fade | table, highlight | |

### Notes for 07

- **Why the test never misses (07_2 to 07_4).** When the water stops, every
  tap it cannot reach, every pipe from a reached row into a cut-off column
  and every reached drain is full, and together they carry the flow (Ford
  and Fulkerson's Lemma 2, read): 3 + 4 + 3 = 10 (`DT-F15.cut.*`). The total
  is 11, so the cut-off columns ask 11 − 3 = 8, more than 3 + 4 = 7. And 7
  bounds what any picture could put in those columns: a row the water cannot
  reach gives at most its count, a reached row at most one a column. In
  general the cut-off columns ask (total − reached drains), more than
  (unreached taps + pipes), which is at least Σ min(r, k): checked on every
  4 × 4 equal-total pair (`model` test) and on every 3 × 3 pair (oracle).
- **Equal totals (07_5, 07_6).** Rows 2, 2 and columns 1, 1 pass both sizes
  of the table, their totals are 4 and 2, and nothing fits (`DT-F16.*`; the
  model test and the oracle check of the five-chapter cut, now facts).
- **Prediction (07_5).** The viewer has just seen that the test needs equal
  totals; the question ends unit 07u2, "No." opens 07u3.
- **The card (07_4)** is the five-chapter cut's 176-character Gale card,
  now after the argument instead of inside it; it stands its characters / 15
  seconds (11.7 s) from its fade-in.
- **Narration units.** 07u1 (07_1 to 07_4), 07u2 (07_5), 07u3 (07_6), 07u4
  (07_7; the old 03u4).

## 08_the_only_picture: The only picture (~66 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Water builds one picture. Can that picture alone tell us whether it is the only one? | as before (storyboard 04_1 of the five-chapter cut): the grid, the face's counts; on "builds one picture", the water's picture, 01010/01100/00000/01010/10011, square by square, "the water's picture" *(built)* | lagged reveal, grid | |
| 08_2 | It can. This one has twelve switches. Swap the corners of any of them, and every count stays. A second picture fits. | as before (F3): on "This one has", "12 switches" and twelve thumbnails in a 4 by 3 grid, each with one computed switch's corners ringed *(face.switches)*; on "Swap the corners", thumbnail 0 framed gold and the grid's rings on that switch's corners; its filled corners empty as its empty corners fill *(face.first, face.second)*; on "every count stays", the counts flash and turn green; on "A second picture", "a second picture, the same shadows" *(face.second.fits)* | highlight | |
| 08_3 | Now the tree's shadows from before. The water builds a picture. | The second picture, its label, counts and thumbnails stay until the label has stood its characters / 15 seconds (F5), then fade together; on "from before", once they are gone, the taps 1, 3, 5, 1, 1 and drains 1, 2, 5, 2, 1, "to send", "to drain", "through: 0 of 11" *(tree.rows, tree.cols, tree.total)*; on "The water builds", the method's real eleven units, the last taking the top row's unit back *(tree.trace, tree.takebacks)* | lagged reveal, motion | |
| 08_4 | It is the tree itself. ⟨predict⟩ Could another picture cast the same shadows? | as before: the counters become the tree's counts, green; on "the tree itself", the wet squares flash, "the water's picture" *(tree.built.is_tree)*; the think ring, four seconds | highlight, tracker | |
| 08_5 | No. A switch needs two rows that each have a filled square where the other is empty. | as before (F5): a small copy of the water's picture from 08_1 with its first switch's rectangle, "the switch from before"; on "each have a filled square", rings on its two filled corners and "each row has a filled square" over "where the other row is empty" | highlight | |
| 08_6 | But of any two rows in the tree, one has every square the other has. So the tree has no switch. | as before (F5): on "of any two rows", an outline round each tree row's squares, shortest first *(tree.nested)*; once read, the small copy's words go and "of any two rows, one has all the other's squares"; on "no switch", "0 switches among 100 rectangles" *(tree.built.switches, tree.rectangles)* | lagged reveal, highlight | |
| 08_7 | And a switch is what stopped reasoning on the face. No reasoning can settle a square that some switch changes. | The words on the right go; a small copy of the face, chapter 02's twenty undecided squares shaded behind its squares, one of its fourteen switches' corners ringed gold *(face.switches, reason.face.free)*; "20 squares some switch changes" and "the shadows cannot settle them" beside it | highlight | |
| 08_8 | How common is a single answer? On a four by four grid, only about one picture in ten is the only one with its shadows. | The copy and its words go; "4 by 4 grids: 65,536 pictures" on the right; on "only about one", "6,902 are the only picture with their shadows, about 1 in 10" *(X3.alone, X3.pictures)*; a held beat, the fade | highlight | |

### Notes for 08

- **The pictures are the method's own** (the five-chapter cut's chapter 04):
  08_1's picture is chapter 03's water (`facts::face_flow`), its 12 switches
  `model::switches` of it; the tree's picture is built by the method's real
  eleven units (`facts::tree_flow`), the last one taking water back.
- **08_3 and F5.** The tree's counts were read aloud in chapter 02, so the
  line does not repeat them; the anchors "Rows of" and "columns of" are gone.
  F5's hold (the second picture's label stands until its characters / 15
  seconds have passed from its fade-in) still ends before the taps and drains
  come in: the scene waits for the later of that hold and "from before".
- **Prediction (08_4), a callback.** Chapter 02 settled every square of the
  tree, so a viewer can answer "no" from memory; the new content is *why*.
  The question and the protected phrase are unchanged (root, 2026-10-20).
- **08_7, exact.** A square some switch changes is filled in one picture
  that fits and empty in another, so no reasoning from the shadows can settle
  it. On the face those are the twenty squares outside the middle row
  (`DT-F12.face.free`; every one of the 64 is reached by switches,
  `DT-F3.connected`). No general claim about reasoning is voiced.
- **08_8.** 6,902 of the 65,536 pictures on a 4 × 4 grid are the only one
  with their shadows (`DT-X3.alone`, `DT-X3.pictures`, every 4 × 4 picture);
  10.5 %, "about one in ten" (`DT-X3.alone.tenths` = 1, the share rounded to
  tenths), the screen showing both numbers.
- **Narration units.** 08u1 (08_1, 08_2; the old 04u1), 08u2 (08_3, 08_4),
  08u3 (08_5 to 08_7), 08u4 (08_8).

## 09_chains_of_switches: Chains of switches (~78 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | One worry remains. These two pictures cast the same shadows, one square in every row and column. ⟨predict⟩ Can a single switch turn one into the other? | Two three by three pictures, 100/010/001 on the left and 010/001/100 on the right, each with row and column counts 1, 1, 1 in green *(perm.left, perm.right, perm.per_line)*; the think ring, four seconds | tracker | |
| 09_2 | No. They differ in six squares, and a switch changes only four. | The ring goes; on "six squares", gold rings on the six squares where they differ, in both *(perm.differ)*; on "only four", "a switch changes 4 squares" *(twin.differ)* | highlight | |
| 09_3 | But two switches do it. First this rectangle, then this one. | On "First this rectangle", the left picture's switch on rows 0, 1 and columns 0, 1 flashes gold and its corners swap: 010/100/001; on "then this one", the switch on rows 1, 2 and columns 0, 2 swaps: the right-hand picture *(perm.chain, perm.distance)*; "2 switches" | highlight | |
| 09_4 | Any two pictures with the same shadows are joined by a chain of switches. | The "precisely" lower-third: any two pictures of filled and empty squares on one grid with the same row counts and the same column counts are joined by a finite chain of switches (Ryser, 1957) *(ryser, X2)*, standing its characters / 15 seconds from its fade-in | lower-third | |
| 09_5 | That's the step the tree needed. If another picture cast its shadows, a chain of switches would lead there, and the chain's first step would be a switch of the tree. | The three by three pictures go; the tree on the left with its counts; on the right a faint empty grid with a large "?"; on "a chain of switches", three short arrows from the tree to the "?", the first gold *(tree)* | highlight | |
| 09_6 | From the tree, no chain can start, so no other picture fits. | The first arrow turns red and the chain and the "?" go; "1 picture fits these shadows" in green *(tree.fits)* | highlight | |
| 09_7 | The same chains tie the face's sixty-four together, each at most three switches from the face. Start from any picture that fits, try every switch, and every switch of what you find, and you reach them all. | The tree goes; chapter 01's gallery in its four waves from the face, 1, 14, 33, 16 *(rings)*; on "Start from any picture", the water's picture's frame turns green and the thumbnails light out from it switch by switch, 1, 12, 37, 14, until all 64 have lit *(perm.rings_from_built, connected)*; a "for programmers" lower-third: list every picture that fits: start from one, apply every switch, keep each new picture, repeat until nothing is new (Ryser, 1957: this reaches them all) | lagged reveal, lower-third (code), highlight | |
| 09_8 | So the water answers whether any picture fits. A switch in the water's picture answers whether it's the only one. | The gallery and card go; chapter 01's two questions, each with its answer as it is spoken: "Does any picture fit?", "yes, exactly when the totals match and every unit gets through"; "Is it the only one?", "yes, exactly when the water's picture has no switch" *(questions)*; held until read, then the fade | highlight | |

### Notes for 09

- **Where the theorem is used (09_5, 09_6).** "No switch" alone rules out
  only a second picture four squares away; 09_1 to 09_3 show that pictures
  with the same shadows can be farther apart (six squares, two switches:
  `DT-F17.perm.*`, the chain `DT-F17.chain` found by `model::walk`). Ryser
  1957, Theorem 3.1 (read) joins any two pictures with the same shadows by a
  chain of switches; from the tree, which has no switch, no chain can start,
  so it is the only picture with its shadows (`DT-F6.tree.fits` = 1, by
  enumeration). Checked on every 4 × 4 pair of shadows (`DT-X2.disconnected`
  = 0).
- **Listing every picture (09_7).** From the water's picture, switches reach
  all 64 in rings of 1, 12, 37, 14 (`DT-F17.rings_from_built`); the listing
  is correct because of Theorem 3.1. No running time is claimed.
- **Prediction (09_1).** The question ends unit 09u1; "No." opens 09u2.
- **The card (09_4)** is the five-chapter cut's 04_7 card (scoped to the
  theorem: two directions, filled-or-empty squares, one grid).
- **Narration units.** 09u1 (09_1), 09u2 (09_2, 09_3), 09u3 (09_4 to 09_6),
  09u4 (09_7), 09u5 (09_8; the old 04u4).

## 10_more_shadows: A third shadow, and a fourth (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Back to the face's sixty-four pictures. Take one more shadow and count the filled squares along each diagonal. | as before (storyboard 05_1, F1): the face, its counts; on "along each diagonal", teal beams along the nine diagonals leave faint guides behind the squares and the counts 0, 1, 1, 2, 1, 1, 1, 2, 0 *(face.diag)*, "diagonal shadow" | lagged reveal, grid | |
| 10_2 | A switch keeps every row and column count, but always changes a diagonal count. This one takes a square from two diagonals and gives both to a third. | as before (F1, F2): the opening switch, "one switch away", the changed counts red, the two losing diagonals lit on "This one takes", the gaining one on "gives both" *(twin.diag, twin.lose, twin.gain)* | highlight | |
| 10_3 | So none of the fourteen pictures one switch from the face can pass. ⟨predict⟩ How many of all sixty-four do? | as before: the switch undone; the gallery of 64; the fourteen one switch away fade *(one_switch.pass)*; the think ring | lagged reveal, tracker | |
| 10_4 | Two. Sixty-two fail, but one other picture casts all three of the face's shadows. | as before: the others fade, "2 of 64 cast the face's diagonal shadow too" *(survivors, dropped)*, the other picture on the right, all its counts green *(other)* | highlight | |
| 10_5 | They differ in six squares. Each row, column and diagonal through them loses one square and gains one, so all three shadows stay the same. | as before: gold rings on the six squares, the nine lines' counts flash, "each row, column and diagonal through them: −1 +1" *(other.differ, other.lines)* | highlight | |
| 10_6 | Two shadows left sixty-four pictures. A third still leaves two, so here even three shadows lie. | as before: "3 shadows, 2 pictures" *(shadows, survivors)* | highlight | |
| 10_7 | Was the face unlucky? Forty-eight of the sixty-four are the only picture with all three of their own shadows. The face is not one of them. | The other picture and the marks go; on the right the 64 thumbnails, grouped by their diagonal counts: the 48 alone framed green, then the six pairs and the group of four in dim frames *(more.groups, more.alone)*; on "not one of them", the face and its partner flash | lagged reveal, highlight | |
| 10_8 | The face looks the same in a mirror. So along the other diagonals its counts are the first ones mirrored, and the picture that survives them is this one's mirror image. | The gallery goes; on the right a copy of the face; on "the other diagonals", teal beams from upper left to lower right leave its counts past their upper-left ends, the first counts in mirror order *(more.falling)*; on "mirror image", two thumbnails under the pictures, the other survivor of chapter 10's first test and the survivor of the other diagonals, a dim mirror line between them *(more.falling.other)* | highlight | |
| 10_9 | Count both diagonal shadows, and only the face is left. That's this picture, not a rule. | The thumbnails go; on "only the face", "4 shadows, 1 picture" above *(more.both)*; on "not a rule", "this face only", dim; a held beat, the fade | highlight | |

### Notes for 10

- **10_1 to 10_6** are the five-chapter cut's chapter 05 lines 05_1 to 05_6,
  unchanged with F1 and F2 (`ec5eb74`); their notes stand: the diagonal
  shadow `model::diagonal_counts`, `DT-X6`, the survivors
  `facts::diagonal_survivors`, the six squares `facts::line_changes`.
- **10_7.** All 64 share the face's rows and columns, so one of them is the
  only picture with its three shadows exactly when no other of the 64 has
  its diagonal counts. Grouped by diagonal counts there are 55 groups, 48 of
  one picture, six of two and one of four (`DT-F18.*`); the face's group has
  two. An enumeration over these 64, not a claim about other pictures.
- **10_8.** The face is its own mirror image (`DT-F18.face.mirror`), so its
  counts along the other diagonals (`model::falling_counts`, upper left to
  lower right) are its first diagonals' counts in mirror order: 0, 2, 1, 1, 1,
  2, 1, 1, 0 from the bottom-left corner (`DT-F18.falling`) against 0, 1, 1,
  2, 1, 1, 1, 2, 0 from the top-left (`DT-F11.face.diag`). (The editorial
  candidate wrote "count the same"; that holds only in mirror order, found
  while pinning the facts on 2026-10-20, and the line now says so.) The other diagonals alone
  leave the face and 01010/00110/00000/11000/01011, the mirror image of
  `DT-F11.other`, also two switches and six squares from the face.
- **10_9.** Rows, columns and both diagonal directions leave only the face
  (`DT-F18.both`). "That's this picture, not a rule": nothing is claimed
  about how many directions settle a picture in general, nor about how hard
  reconstruction is (the Gardner, Gritzmann and Prangenberg claim stays
  omitted: its paper was not read).
- **The right-hand copy (10_8)** shows only the other diagonals' counts, so
  they never stand where the face's row counts or first diagonal counts do.
- **Narration units.** 10u1 (10_1 to 10_3; the old 05u1), 10u2 (10_4, 10_5;
  the old 05u2), 10u3 (10_6; the old 05u3), 10u4 (10_7 to 10_9).

## 11_what_two_shadows_promise: What two shadows can promise (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | So here's what a program can promise from two shadows. Check that the totals match, and send the water. | The face's counts round an empty grid on the left; on the right, "for programmers:" and a list that builds as it is spoken: on "Check that the totals match", "1. do both totals match?"; on "send the water", "2. send the water" *(steps)* | lagged reveal | |
| 11_2 | If every unit gets through, the wet squares are a picture that fits. If not, where the water stops names a group of columns asking too much. | On "If every unit", "3. every unit through? a picture that fits"; on "If not", under it "otherwise the columns it can't reach ask too much" | | |
| 11_3 | Then look for a switch in that picture. None means it's the only one. Any switch means there are others, and switches reach every one of them. | On "look for a switch", "4. no switch? the only one"; on "Any switch", under it "a switch? others, and switches reach them all" | | |
| 11_4 | Try the four steps on the face's shadows. ⟨predict⟩ What does the program report? | On "the face's shadows", the face's counts flash; the think ring, four seconds | tracker | |
| 11_5 | Every unit gets through, but the water's picture has twelve switches. So it reports a picture that fits, and that it isn't the only one. | The ring and the list go; on "Every unit", the water's picture square by square on the grid, "every unit through: 9 of 9"; on "twelve switches", "the water's picture: 12 switches", gold; on "So it reports", "a picture that fits, not the only one" *(through, face.switches, built.only_one)* | lagged reveal, highlight | |
| 11_6 | What no program can say is which of the sixty-four was really there. Two of the face's own squares are filled in only twelve of them. | On "which of the sixty-four", "which of the 64 was there? the shadows don't say"; on "Two of the face's own squares", the grid shows the face and two of its squares ring white, "each filled in only 12 of 64" *(reason.face.rarest)* | highlight | |
| 11_7 | So keep the two questions apart. Does any picture fit? And is it the only one? | The grid and words go; a "precisely" lower-third bounding the model: every picture in this video is a small grid of filled and empty squares, counted exactly along rows, columns and, for the face, diagonals; nothing here models a scanner *(size)*; on the right, chapter 01's two questions, each with its answer as it is asked: "Does any picture fit?", "the water says, when the totals match"; "Is it the only one?", "with rows and columns, a switch says; these four shadows singled out the face" *(questions)*; held until read, then the fade | lower-third, highlight | |

### Notes for 11

- **The four steps (11_1 to 11_3)** are exact for two directions: Gale 1957
  and Ford and Fulkerson 1957 for steps 1 to 3, Ryser 1957 Theorem 3.1 for
  step 4 (all read). The list's length is `DT-F19.steps`, read from the
  scene's source. Nothing is said about three or more directions.
- **Prediction (11_4), a transfer.** The viewer runs the four steps on the
  face's shadows: the totals match, every unit gets through
  (`DT-F8.through` = 9), the water's picture has twelve switches
  (`DT-F10.face.switches`), so it is not the only one
  (`DT-F8.built.only_one` = false). The question ends unit 11u2; "Every unit
  gets through" opens 11u3.
- **What no program can say (11_6).** Every picture that fits casts the same
  shadows, so nothing computed from the shadows alone can tell them apart: a
  matter of information, not of computing power. Squares (3, 0) and (3, 4)
  are filled in the face and in only 12 of the 64 (`DT-F12.face.rarest`,
  `.rarest.count` = 2): the picture that was there need not look typical.
- **The closing card and answers (11_7)** replace the five-chapter cut's
  05_7 versions: the scope now covers every grid the video shows, and the
  answers are the whole video's ("these four shadows singled out the face",
  root's wording, makes no claim about other directions). Both need their
  characters / 15 read holds, measured from their fade-ins.
- **Narration units.** 11u1 (11_1 to 11_3), 11u2 (11_4), 11u3 (11_5, 11_6),
  11u4 (11_7; the old 05u4).
