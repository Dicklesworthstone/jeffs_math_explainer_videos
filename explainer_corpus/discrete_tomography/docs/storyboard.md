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
column counts 1, 2, 5, 2, 1) fits exactly one. So the two questions, *does any
picture fit?* and *is it the only one?*, are kept apart from the first chapter
on.

| Chapter | Spine | Status |
|---|---|---|
| `01_same_shadows` | the face and its two shadows; hidden, could another picture cast them? (prediction); the other picture differs at a rectangle's alternating corners, and swapping filled and empty there keeps every count: a switch; 64 pictures fit, each within 3 switches of the face (Ryser's theorem on the card); the face's rows with columns 5, 1, 1, 1, 1 fit nothing, because that column needs a square in every row and the middle row is empty; the two questions | source written 2026-10-19 (`.9.1`): scene, facts, oracle and tests; not compiled, rendered, narrated or reviewed in this lane |
| 02 *(planned)* water through the grid | Gale's network drawn on the grid itself: each row a tap releasing its count, each square a pipe carrying at most one unit into its column, each column a drain needing its count; augmenting paths, including one that takes water back from a square; a full flow is a picture (one of the 64), and the flow never asks which one (`model::max_flow`, `model::reconstruct`) | not written |
| 03 *(planned)* where the water stops | the no-fit shadows: the flow stops at 8 of 9; the cut names the first column, which can get at most one square from each nonempty row (Gale's count, min(r, k) per row for k columns); Gale's test for every k; feasible is decided, uniqueness is not (`model::bottleneck`, `model::gale_ryser`) | not written |
| 04 *(planned)* the only picture | Ryser: any two pictures with the same shadows are joined by switches, so a picture with no switch is the only one; the tree; checking the picture the flow built for a switch settles uniqueness (`model::switches`, `model::reachable`) | not written |
| 05 *(planned)* more shadows, and the limits | a third direction can tell some of the 64 apart, but reconstruction from three or more directions is NP-hard in general (Gardner, Gritzmann and Prangenberg 1999, as cited by Batenburg 2006); real CT scanners measure continuous densities along many angles, not counts of filled squares | not written; sources for the complexity claim not yet read in the original |

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

## Color roles

| Idea | Color |
|---|---|
| a filled square, in every picture and every thumbnail | blue `#58C4DD` |
| a switch: its rectangle, its corners' rings, its word | `hero` gold `#FFD866`, strokes and words only |
| a beam counting along a row or down a column | teal `#5CD0B3`, strokes only |
| a count that matches the face's | green `#83C167` |
| what no picture can do: the count it fails, the cross, "no picture fits" | red `#FC6255` |
| counts, questions | `ink` |
| grid lines, labels, tags | `faint` and `dim` |

## 01_same_shadows: The same shadows (~83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Here is a little face on a grid of squares. | A five by five grid of faint lines; on "a little face", the face's nine blue squares appear one by one *(face)* | lagged reveal, grid | |
| 01_2 | Count the filled squares in each row, and in each column. Those counts are the picture's two shadows. | Teal beams run along the rows one after another and leave each row's count at its right end, 2, 2, 0, 2, 3 *(face.rows)*; on "and in each column", beams run down the columns and leave 1, 3, 1, 3, 1 below them *(face.cols)*; on "two shadows", the dim labels "row shadow" and "column shadow" | lagged reveal | |
| 01_3 | Hide the picture and keep the shadows. ⟨predict⟩ Could a different picture cast the same shadows? | The blue squares fade; the empty grid and both rows of counts stay; at the question's end a gold "?" in a draining ring, "pause for more time" under it, four seconds, the voice silent | tracker | |
| 01_4 | It can. Every row count matches, and every column count matches. | The grid and its counts slide left and the face returns, "the face" above it; "another picture" appears on the right *(twin)*; on "Every row count", the face's row counts flash and the other picture's appear beside it one by one, green; on "every column count", the same for the columns | lagged reveal, highlight | |
| 01_5 | They differ in four squares, the corners of a rectangle, with filled and empty corners alternating. | On "four squares", gold rings on the four squares in both pictures, joined by a thin gold rectangle *(switch, differ.cells)*; on "alternating", the rings flash in turn around the rectangle, filled, empty, filled, empty | highlight | |
| 01_6 | Swap filled and empty at those corners. Each of their rows and columns loses one square and gains one. | On the right, the two filled corners fade as the two empty corners fill: the other picture becomes the face *(switch)*; on "loses one square", a dim "−1 +1" by each of the two rows' and two columns' counts, which flash *(loses, gains)* | highlight | |
| 01_7 | That move is a switch. It changes the picture, but neither shadow. | The tags go; the gold word "switch" beside the right-hand rectangle; on "neither shadow", every count on screen flashes | highlight | |
| 01_8 | How many pictures cast these shadows? Sixty-four, and each is at most three switches from the face. | The right-hand picture, the marks and the names go; on the right a gallery of every picture with these shadows fills in four waves, the face (white frame) first, then the 14 one switch away, the 33 two away, the 16 three away *(fits, rings)*, finishing on "Sixty-four" with "64 pictures" above it; on "at most three", the caption becomes "64 pictures, each within 3 switches of the face" *(farthest)* and the big face takes three switches, each one's gold rectangle flashing as its corners swap and the thumbnail it reaches lighting *(walk)*; a "precisely" lower-third: any two pictures with the same row and column counts are joined by a chain of switches (H. J. Ryser, 1957) *(ryser)* | lagged reveal, highlight, lower-third | |
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
