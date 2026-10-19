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
| `01_same_shadows` | the face and its two shadows; hidden, could another picture cast them? (prediction); the other picture differs at a rectangle's alternating corners, and swapping filled and empty there keeps every count: a switch; 64 pictures fit, each within 3 switches of the face (Ryser's theorem on the card); the face's rows with columns 5, 1, 1, 1, 1 fit nothing, because that column needs a square in every row and the middle row is empty; the two questions | source and silent visuals verified by root on 2026-10-19 at 6bae9ea (`.9.1`: strict clippy, tests, 48/48 oracle facts, glyph probe, a 540p30 silent draft with no layout findings, cue-end and question/reveal frames read); no narration or audio exists or is claimed |
| `02_water_through_the_grid` | Gale's network drawn on the grid itself: each row a tap that lets out its count, each square a pipe carrying at most one unit down into its column, each column a drain holding at most its count; the method's real augmenting paths, one unit each (`model::max_flow` records them, `model::replay` rebuilds and checks every state); eight go straight down, then the bottom row is stuck without taking back (prediction); the ninth unit takes the top row's unit back out of the first column, and the top row sends it to the fourth; every unit through, a picture with the face's shadows that is not the face, one of the 64; the water finds a picture whenever one exists and never says whether it is the only one | source and silent visuals verified by root on 2026-10-19 (`.9.2`): fmt, strict release all-target clippy, 35 tests, release build, 73/73 facts against the oracle's 31 checks; a silent 960x540 30 fps draft, 2,452 frames, 81.73 s by scratch-voice timing, no layout findings; all 9 cue ends, 12 frames of the first and ninth units in transit (counters leave and arrive correctly, the fourth column's unit in transit before "through: 9 of 9"), 6 prediction and reveal frames (the answer held back through the hold) and all 5 glyph pages read; the take-back trace and counters independently reviewed; no narration or audio exists or is claimed |
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
From chapter 02: **tap** (where a row's water comes in; never "source"),
**pipe** (a square seen as plumbing), **drain** (where a column's water
leaves; never "sink"), **unit** (one unit of water: one filled square),
**wet** and **dry** (a pipe carrying a unit or not), **path** (one unit's way
from a tap to a drain), **room** (what a pipe can still carry), **take back**
(water pushed back out of a pipe; the literature's "reverse edge" or
"residual capacity" is named in the description, not the video).

## Color roles

| Idea | Color |
|---|---|
| a filled square, in every picture and every thumbnail | blue `#58C4DD` |
| a switch: its rectangle, its corners' rings, its word | `hero` gold `#FFD866`, strokes and words only |
| a route along rows and columns: a beam counting (01), a unit of water's path (02) | teal `#5CD0B3`, strokes only |
| water taken back out of a square, and the way it goes back (02) | purple `#9A72AC` |
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

## 02_water_through_the_grid: Water through the grid (~83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Turn the grid into pipes. Each row's tap lets out its count in units of water. | The empty five by five grid on the left; on "Each row's tap", a counter left of each row, 2, 2, 0, 2, 3 *(face.rows)*, under the dim label "to send"; on the right, "tap: lets out its row's count" | lagged reveal, grid | |
| 02_2 | A unit runs along its row, turns down through a square, and drains out of that column. | A counter under each column, 1, 3, 1, 3, 1 *(face.cols)*, label "to drain"; the method's first real unit *(trace, path 1: top row, first column)*: a blue drop appears at the top row's tap as its counter drops to 1, runs in along the top row, the first square fills on "turns down", and on "drains out" the drop falls into the first drain, whose counter drops to 0 as it arrives | lagged reveal, motion | |
| 02_3 | A square carries at most one unit, and each drain holds at most its count. Get every unit through, and the wet squares are a picture that fits. | The wet square flashes, "square: carries at most 1 unit" *(square)*; on "each drain holds", the drain counters flash, "drain: holds at most its column's count"; on "Get every unit through", "every unit through: a picture that fits" and the counter "through: 1 of 9" | highlight | |
| 02_4 | Send one unit at a time, along any path with room. | The legend gives way to a "for programmers" lower-third (each unit takes a path found breadth first: in from a tap with water left, forward into dry squares, back out of wet ones, down into a drain with room); units 2 to 8 of the real trace, quickly: each route comes in teal as its tap counts down, its square fills, and the route goes as its drain counts down and "through" counts up *(trace, paths 2 to 8)* | lower-third (code), highlight | |
| 02_5 | Eight are through. The bottom row has one left, but its dry squares lead to full drains. ⟨predict⟩ Is the water stuck? | The card goes; "through: 8 of 9" *(before.through)*; on "The bottom row", its tap's 1 flashes *(stuck.left)*; on "its dry squares", teal routes from the bottom tap through its three dry squares to their drains, all at 0, which flash *(stuck.dry_full)*; the think ring, four seconds, the voice silent | highlight, tracker | |
| 02_6 | No. Send it into the first column anyway, and take back the unit the top row sent there. | The ring goes; on "Send it into", a drop leaves the bottom tap, whose counter drops to 0, runs in along the bottom row and waits at the bottom-left square; on "take back", in one move, that square fills, a purple route runs up the first column, and the top row's square there empties into a purple drop: the top row's unit, taken back *(trace, path 9; takeback.column, takeback.row)* | motion, highlight | |
| 02_7 | The top row sends that unit to the fourth column, which still needs one. Without taking back, this fill would have stopped one unit short. | On "sends that unit", the purple drop runs along the top row to the fourth square, which fills, and falls into the fourth drain, whose counter drops to 0 as it arrives: every counter reads 0, "through: 9 of 9" *(takeback.to_column, takeback.to_needs, through)*; on "Without taking back", dim "without taking back: 8 of 9" *(short)* | motion | |
| 02_8 | The wet squares cast the face's shadows, but they aren't the face. They're one of the sixty-four. | The counters become the wet picture's counts, green: 2, 2, 0, 2, 3 and 1, 3, 1, 3, 1; on "but they aren't", white outlines on the face's nine squares over the water's picture, "outlined: the face" *(built.is_face)*; on "They're one of", chapter 01's gallery of 64 on the right, the face's frame white, the water's picture's frame green and flashing, "the water's picture is one of 64" *(built, built.fits, built.from_face)* | lagged reveal, highlight | |
| 02_9 | Water finds a picture whenever one exists. It never says whether that's the only one. | A "precisely" lower-third: the water ends at a largest flow (Ford and Fulkerson, 1957); a picture fits exactly when both shadows share one total and the flow reaches it (Gale, 1957) *(ford_fulkerson, gale)*; on "It never says", the whole gallery flashes; a held beat, the fade | lower-third, highlight | |

### Notes for 02

- **The trace is the real one.** `model::max_flow` records every augmenting
  path it sends (Ford and Fulkerson's labeling, breadth first; nodes scanned
  in the order source, rows top to bottom, columns left to right, sink);
  `model::replay` rebuilds the water from those paths alone and refuses any
  path that visits a row or a column twice, wets a wet square, takes water
  back from a dry one, overdraws a tap or overfills a drain. Those rules keep
  every row and column balanced; the replay asserts that balance after every
  path as an invariant they imply, not as a separately tested way to fail.
  The scene draws only replayed states and, after every path, checks its wet
  squares and every counter against them. `oracle.py` derives the same nine
  paths with its own residual bookkeeping, checks each state with its own
  rules, and shows its checker refuses broken paths, among them the cyclic
  walk source, row 0, column 0, row 0, column 1, sink, which only the
  no-repeat rule catches.
- **Mid-path.** Between paths every drawn state is a flow (capacities kept,
  every row and column balanced). A tap counts down when a drawn unit leaves
  it; its drain and "through" count when that unit arrives; in between, the
  unit in transit is its drop (02_2, 02_6, 02_7) or its route (02_4), so the
  bottom tap reads 0 while its third unit is on its way. Inside the ninth
  path the unit in transit is a drop: the blue
  drop waits at the bottom-left square until "take back", when that square
  fills and the top row's square in the same column empties in one move;
  from then until the fourth square fills, the top row's two units are its
  wet square and the purple drop; once it fills, the top row's two units are
  its two wet squares, and the drop falling down the fourth column is that
  column's unit in transit (three wet squares, two drained) until it reaches
  the drain.
- **Prediction (02_5).** The question ends unit 02u2; the think ring drains
  for four seconds, the voice silent; "No." opens 02u3. Nothing moves the
  ninth unit before "No."
- **Feasible, not unique.** The water's picture fits (`DT-F8.built.fits`) and
  is not the face (`DT-F8.built.is_face`); it is 3 switches from the face,
  as far as any of the 64 (`DT-F8.built.from_face`). 02_9 says the method
  never decides uniqueness; chapter 04 does, with switches.
- **"Without taking back."** Exact for this fill: after the first eight
  paths no row with water left has a dry square into a drain with room
  (`DT-F8.before.forward_room` = false), so a fill that never takes back
  stops at 8 of 9 (`DT-F8.short`).
- **Sources.** Gale 1957 (the network and the equivalence with filling the
  grid); Ford and Fulkerson 1957, Canad. J. Math. 9, 210–218 (read: the
  labeling method on a matrix of room "diminished by the flow from P_i to P_j
  and increased by the flow from P_j to P_i", its termination for whole-number
  capacities, Lemma 1 that the result is a flow and Lemma 2 that it is a
  largest one, with the labeled nodes giving a minimal cut). Breadth-first
  labeling is the choice Edmonds and Karp (1972) analysed; that paper was
  not read here, and the video does not name it.
- **Budget.** 169 spoken words, 9 lines: about 83 s by the Golay measures
  (2.6 words a second plus 0.4 s a line, thought gaps about 0.82 s, a think
  window about 5.2 s, a tail of 2 s). Units 2 to 8 take about 2.7 s of
  02_4's 4.6 s (0.39 s each: departure, square, arrival). Root's silent
  draft, at scratch-voice timing, runs 81.73 s.
- **Narration units.** 02u1 to 02u4 are a proposed grouping into whole
  thoughts: the setup (02u1), the run of units ending on the prediction
  question (02u2), the answer and the take-back (02u3), the picture and what
  the water cannot say (02u4). Generation boundaries are not frozen; they may
  change after an actual listening comparison. The owner's 2026-10-19
  rejection of the first-wave narration applies, and nothing is synthesized.
