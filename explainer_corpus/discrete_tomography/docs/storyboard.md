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
| `03_where_the_water_stops` | the face's rows with columns 5, 1, 1, 1, 1: the method's real units, eight straight down, then the stop (prediction: could taking water back get the ninth through?); no: the first column needs five, each row sends it at most one, only four rows have water; everywhere the water can still go, taking back included, misses that column, and the cut there lets four units into it and four out of the other drains, eight in all (`facts::no_fit_flow`, `Flow::cut`); any k columns get at most min(count, k) from a row; Gale's table, the k largest columns against what the rows can give (`model::gale_table`); with equal totals the test is complete, because the columns the water cannot reach always ask too much; the face's shadows pass and 64 pictures fit: the test decides feasibility, not uniqueness | source and silent visuals verified by root on 2026-10-19 (`.9.3`). First native run: fmt, strict clippy, 43 tests, 96/96 facts against the oracle's 38 checks, 69 glyph sources. An independent image review of the first draft found three faults: teal reach strokes crossing the cut-off first column, a reach label cut short ("taking back") and the 218-character Gale card whole for only 7.27 s, mostly under speech. All three were repaired in this lane. After the repairs: fmt, strict clippy, a build, the 4 changed scene tests (the other 40 unchanged) and 68 glyph sources without failure. Silent draft06: 2,591 frames at 30 fps, 86.37 s by scratch-voice timing (sha256 `dd54be19d333…`), empty layout lint; the reach and theorem frames were read, and the 176-character card stands whole 11.93 s. It runs 1.37 s past 85 s for that reading hold, by root's decision. No narration or audio exists or is claimed |
| `04_the_only_picture` | the water's picture for the face's shadows (chapter 02's) has twelve switches; swapping one's corners keeps every count, so a second picture fits; the tree's shadows only, and the method's real units build the tree itself (prediction: could another picture cast them?); no: a switch needs two rows that each have a filled square where the other is empty, and of any two of the tree's rows one has every square of the other, so the tree has no switch; any two pictures with the same shadows are joined by a chain of switches (Ryser 1957, Theorem 3.1, scoped on the card), and from the tree none can start, so it is the only one; the water answers whether any picture fits, a switch in its picture whether it is the only one (`model::switches`, `model::rows_nested`, `model::built_switches`, `facts::tree_flow`) | source and silent visuals verified by root on 2026-10-19 (`.9.4`, committed at fb90763): fmt, strict all-target clippy, 53 tests, 114/114 facts against the oracle, 93 glyph sources without failure and a build; silent draft07, 2,470 frames at 30 fps, 82.33 s by scratch-voice timing, empty layout lint, six frames and all nine glyph pages read by root; an independent frame review passed with no repairs (the tree's real rerouting, no answer during the prediction's hold, the 12 switches recounted, the nesting; the Ryser card whole 11.03 s, the closing questions with their answers 11.93 s and 9.13 s); no narration or audio exists or is claimed |
| `05_a_third_shadow` | back to the face's 64 pictures and a third shadow: the counts along the diagonals that run from lower left to upper right; a switch keeps every row and column count but always changes a diagonal count (the opening switch takes a square from two diagonals and gives both to a third); every one of the 64 tested against the face's diagonal counts (prediction: how many pass?); two pass, the face and one other picture, six squares apart, every row, column and diagonal through them losing one square and gaining one; here even three shadows lie; the two questions kept apart, inside a bounded model (`model::diagonal_counts`, `facts::diagonal_survivors`). No complexity claim and no comparison with real scanners: the complexity paper (Gardner, Gritzmann and Prangenberg 1999) could not be read in the original | source and silent visuals verified by root on 2026-10-19 (`.9.5`): fmt, strict clippy, 59 tests, a build, 136/136 facts and 102 glyph sources without failure, the 20 pinned inputs unchanged; an independent source and mathematics review passed (an independent recount of the 64, the diagonals, the survivors and the six squares). Draft09's frames passed except the "2 of 64" caption, readable about 0.45 s; after that repair and the card's "one diagonal direction", fmt, strict clippy, the 4 changed scene tests, a build and a render passed. Silent draft10: 2,116 frames at 30 fps, 70.53 s by scratch-voice timing (sha256 `be0a3f0f29d0…`), empty layout lint, eight frames read by root; the independent frame review measured the caption readable about 18.6 s with the right-hand picture's timing unchanged and no answer before the hold ends (a five-frame overlap of the caption's two copies as it moves is a known cosmetic note). No narration or audio exists or is claimed |

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

## 03_where_the_water_stops: Where the water stops (~83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Back to the shadows no picture fits. The face's rows, and columns asking for five, one, one, one and one. | Chapter 02's plumbing: the empty grid, the taps 2, 2, 0, 2, 3 *(face.rows)*; on "and columns asking", the drains 5, 1, 1, 1, 1 *(no_fit.cols)* and "through: 0 of 9"; then the method's real units, eight, each route coming in as its tap counts down, its square filling, its drain and "through" counting as it arrives *(no-fit trace)* | lagged reveal, highlight | |
| 03_2 | The water stops at eight of nine. ⟨predict⟩ Could taking water back get the ninth through? | On "stops at eight", "through: 8 of 9", the bottom tap's 1 and the first drain's 1 flash *(flow, left)*; the think ring, four seconds, the voice silent, nothing moving | highlight, tracker | |
| 03_3 | No. The first column needs five, but each row sends it at most one, and only four rows have water. | The ring goes; on "The first column", a red outline round it and "asks 5" under its drain *(first.needs)*; on "each row sends", a ring on each of its five squares, one per row; on "only four rows", a red cross on the middle row's square there, and that row's 0 flashes *(rows_with_water)* | highlight | |
| 03_4 | Taking back just moves water between pipes. Four into the first column, four out of the other drains. Eight is all that can pass. | Teal strokes along every row and column the water can still reach, taking back included: every row with water and every column but the first *(reached)*, each row's stroke running over the reached columns only and stopping clear of the first column's red outline, "where the water can still go, taking back included"; on "Four into", the four wet squares in the first column flash, "4 pipes into the first column" *(cut.into_unreached)*; on "four out of", the other four drains flash, "4 out of the other drains" *(cut.reached_drains)*; on "Eight is all", "4 + 4 = 8, all that can pass" *(cut)* | highlight | |
| 03_5 | Count any group of columns this way. Each row gives it at most one square per column, and no more than its count. | The strokes and words go; on the right, Gale's table: "k" 1 to 5 and "rows can give"; on "Each row gives", "a row gives k columns at most min(its count, k)" and the row of what the rows can give, 4, 8, 9, 9, 9 *(gives)* | table, lagged reveal | |
| 03_6 | If the group asks for more than the rows can give, no picture fits. Checking the hungriest group of each size is enough. | "largest columns first", 5, 1, 1, 1, 1 *(sorted)*, and "the k largest ask", 5, 6, 7, 8, 9 *(asks)*; on "no picture fits", the first size's 5 and 4 turn red *(failing)*; on "the hungriest", the largest-first row flashes | table, highlight | |
| 03_7 | With equal totals, that test never misses. When water stops short, the columns it can't reach always form a group that asks too much. | The last size's 9 and 9 turn green, "equal totals" over them *(totals)*; on "When water stops short", a "precisely" lower-third (a picture fits exactly when row and column totals match and, for every k, the k largest columns ask for at most the sum over rows of min(row count, k); Gale, 1957; Ryser, 1957) *(gale, ryser)*, and the red first column, its "asks 5" and the red 5 and 4 flash together; the card stands whole, the line finished, until 11.7 s after its fade-in (176 characters at 15 a second) | lower-third, highlight | |
| 03_8 | The face's shadows pass, and sixty-four pictures fit them. The test says that one fits, not which, and not whether only one. | The water, marks and card clear; the taps and drains become the face's, 2, 2, 0, 2, 3 and 1, 3, 1, 3, 1, and the table its columns, 3, 3, 1, 1, 1, asking 3, 6, 7, 8, 9 against 4, 8, 9, 9, 9, all green *(face.sorted, face.asks, face.gives)*; on "sixty-four", "64 pictures fit these shadows" *(fits)*; on "The test says", "it says one fits, not which, not whether only one"; a held beat, the fade | table, highlight | |

### Notes for 03

- **The stop and the cut are the method's own.** The eight units are the
  paths `model::max_flow` sent on these shadows (`facts::no_fit_flow`,
  replayed by `model::replay`); none takes water back. When no path is left,
  the rows and columns the labeling still reaches, taking back included,
  are rows 0, 1, 3, 4 and columns 1 to 4 (`Flow::cut`; Ford and Fulkerson's
  Lemma 2). Every unit that gets through crosses that cut: into the
  unreached first column through one of the four reached rows' pipes, or
  out of one of the four reached drains. So no flow, whatever it takes
  back, gets past 4 + 4 = 8 (`DT-F9.cut`), the flow the method found.
- **Prediction (03_2).** The question ends unit 03u1; the think ring drains
  for four seconds with the voice silent and nothing on the board moving;
  "No." opens 03u2.
- **The general count (03_5).** A row of count r gives a group of k columns
  at most one square each and at most r in all: min(r, k). Tested on every
  group of columns of every 4 × 4 picture (`model` test) and by the oracle.
- **Largest first (03_6).** For a fixed size k the rows can give the same,
  Σ min(r, k), whichever k columns are chosen; the k largest ask the most,
  so checking them checks every group of that size.
- **Equal totals (03_7).** Rows 2, 2 with columns 1, 1 pass every size of
  the table, yet their totals are 4 and 2 and nothing fits (`model` test):
  the totals condition is separate, and the card states it.
- **The card (03_7).** It keeps every condition: row and column totals
  match; every k; the k largest columns (the sorted condition: the first k
  of the counts sorted largest first, the table's second line); at most the
  sum over rows of min(row count, k); Gale and Ryser. The first wording,
  218 characters, stood whole only 7.27 s, mostly under speech (root's
  draft, 2026-10-19). The card now has 176 characters on two lines, with
  "min(row count, k)" never split. It holds until 176 / 15 = 11.7 s after
  its fade-in, past the line's end; in root's draft06 it stands whole
  11.93 s. Fifteen characters a second is the
  corpus's provisional scene convention, not a listening measure.
- **Why the test never misses (03_7).** With equal totals, if the water
  stops short at flow F, the cut is worth F: the counts of the taps it
  cannot reach, plus one unit for each pipe from a reached row into a
  cut-off column, plus the counts of the reached drains. Since F is less
  than the total, the cut-off columns ask for more than the unreached taps
  plus those pipes, and that is at least Σ min(r, k) for k cut-off columns.
  So the columns the water cannot reach always form a group asking too
  much; checked on every 4 × 4 pair of shadows with equal totals (`model`
  test) and on every 3 × 3 pair by the oracle.
- **Feasible, not unique (03_8).** The face's shadows pass every size and 64
  pictures fit them (`DT-F3.fits`). The test says a picture exists, not
  which or how many; chapter 04 answers uniqueness with switches.
- **Sources.** Gale 1957, p. 1080 (read: the dual sequence, the theorem, and
  the flow into k columns, Σ min(a_i, k)); Ryser 1957, Theorem 2.1 (read: the
  same condition as majorization); Ford and Fulkerson 1957, Lemma 2 (read:
  the labeled nodes at termination give a minimal cut of the flow's value).
- **Budget.** 172 spoken words, 8 lines: about 83 s by the Golay measures.
  The eight units take about 3.1 s inside 03_1. The card's reading hold
  adds silence after 03_7. Root's silent draft06 runs 86.37 s by
  scratch-voice timing, 1.37 s past 85 s; root's direction (2026-10-19)
  is that a slightly longer chapter is better than a rushed card. No
  voice has measured it.
- **Narration units.** 03u1 to 03u4 are a proposed grouping: the setup and
  the question (03u1), the answer and the cut (03u2), the general count and
  why it is complete (03u3), feasibility against uniqueness (03u4).
  Boundaries are not frozen and nothing is synthesized.

## 04_the_only_picture: The only picture (~83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Water builds one picture. Can that picture alone tell us whether it is the only one? | Chapter 02's grid on the left, the face's counts 2, 2, 0, 2, 3 left of the rows and 1, 3, 1, 3, 1 under the columns *(face.rows, face.cols)*; on "builds one picture", the water's picture from chapter 02, 01010/01100/00000/01010/10011, square by square, "the water's picture" above it *(built)* | lagged reveal, grid | |
| 04_2 | It can. This one has twelve switches. Swap the corners of any of them, and every count stays. A second picture fits. | On "twelve switches", the gold rectangles of its twelve switches one after another and "12 switches" in gold *(face.switches)*; on "Swap the corners", all but the first go, gold rings on its corners, and its filled corners empty as its empty corners fill *(face.first, face.second)*; on "every count stays", its two rows' and two columns' counts flash and every count turns green; on "A second picture", "a second picture, the same shadows" above the grid *(face.second.fits)* | highlight | |
| 04_3 | Now the shadows of a tree. Rows of one, three, five, one and one, columns of one, two, five, two and one. The water builds a picture. | The picture, its marks and its counts go; on "Rows of", taps 1, 3, 5, 1, 1 left of the rows under "to send" *(tree.rows)*; on "columns of", drains 1, 2, 5, 2, 1 under the columns, "to drain", and "through: 0 of 11" *(tree.cols, tree.total)*; on "The water builds", the method's real units, eleven, each route coming in as its tap counts down and going as its drain and "through" count; the last takes the top row's unit back out of the first column and the top row sends it on to the middle one *(tree.trace, tree.takebacks)* | lagged reveal, motion | |
| 04_4 | It is the tree itself. ⟨predict⟩ Could another picture cast the same shadows? | The counters, all 0, become the tree's counts, green; "through: 11 of 11" and the labels go; on "the tree itself", the wet squares flash and "the water's picture" stands above them *(tree.built.is_tree)*; the think ring, four seconds, the voice silent | highlight, tracker | |
| 04_5 | No. A switch needs two rows that each have a filled square where the other is empty. | The ring goes; on "A switch needs", on the right a small copy of the face's water picture from 04_1 with its first switch's gold rectangle *(built, face.first)*; on "each have a filled square", gold rings on the switch's two filled corners, each in a column where the other row is empty, and "each row has a filled square" over "where the other row is empty" | highlight | |
| 04_6 | But of any two rows in the tree, one has every square the other has. So the tree has no switch. | The small copy goes; on "of any two rows", an outline round each tree row's squares, shortest first, each run inside the columns of every longer one *(tree.nested)*, and "of any two rows, one has all the other's squares"; on "no switch", "0 switches among 100 rectangles" in gold *(tree.built.switches, tree.rectangles)* | lagged reveal, highlight | |
| 04_7 | Any two pictures with the same shadows are joined by a chain of switches. From the tree, no chain can start, so no other picture fits. | A "precisely" lower-third: any two pictures of filled and empty squares on one grid with the same row counts and the same column counts are joined by a finite chain of switches (Ryser, 1957) *(ryser)*, held whole until it can be read; on "no other picture fits", "1 picture fits these shadows" in green *(tree.fits)* | lower-third, highlight | |
| 04_8 | So the water answers whether any picture fits. A switch in the water's picture answers whether it's the only one. | The card, the outlines and the words go; on the right, chapter 01's two questions, each with its answer as it is spoken: "Does any picture fit?", "yes, exactly when the totals match and every unit gets through"; "Is it the only one?", "yes, exactly when the water's picture has no switch" *(questions)*; held until they can be read, then the fade | highlight | |

### Notes for 04

- **The pictures are the method's own.** 04_1's picture is chapter 02's
  water (`facts::face_flow`, replayed by `model::replay`), not a picture
  chosen for the chapter; its 12 switches are `model::switches` of it, and
  the one swapped is the first in order, rows 0 and 1 with columns 2 and 3
  (`DT-F10.face.first`). The second picture differs from it in those four
  squares and is one of the 64. The tree's picture is built by the method's
  real eleven units on the tree's shadows (`facts::tree_flow`), the last
  one taking water back; it is the tree itself (`DT-F10.tree.built`).
- **One switch is enough (04_2).** A switch takes one square from each of
  its two rows and two columns and gives one back (`DT-X1`, every switch of
  every 4 × 4 picture), so it makes a different picture with the same
  shadows: the water's picture is not the only one.
- **Prediction (04_4).** The question ends unit 04u2; the think ring drains
  for four seconds with the voice silent and nothing moving; "No." opens
  04u3. A viewer who looks for an alternating rectangle in the tree finds
  none.
- **Why the tree has no switch (04_5, 04_6).** A switch needs two rows and
  two columns with the corners alternating: each of the two rows has a
  filled square in a column where the other is empty. If one row holds
  every square the other holds, that cannot happen. So a picture has no
  switch exactly when, of any two rows, one holds every square of the
  other (`model::rows_nested`; `DT-X5` on every 4 × 4 picture, a `model`
  test on every 4 × 4 and 3 × 5 picture). The tree's rows hold {2},
  {1, 2, 3}, {0, …, 4}, {2} and {2}: nested. None of its 100 rectangles
  alternates (the oracle reads every one corner by corner).
- **The theorem, scoped (04_7).** Ryser 1957, Theorem 3.1: two m by n
  matrices of 0s and 1s with equal row sum vectors and equal column sum
  vectors are transformable into each other by a finite number of
  interchanges. The card says it in the video's words, for pictures of
  filled and empty squares on one grid with the same row counts and the
  same column counts. A chain from the tree to another picture would start
  with a switch of the tree; there is none, so the tree is the only picture
  with its shadows (`DT-F6.tree.fits` = 1, by enumeration). The theorem
  joins pictures by chains, not always by one switch: two 3 × 3 pictures
  with one square in each row and column, six squares apart, are two
  switches apart (`model` test; the oracle). It says nothing about more
  than two directions, counts with noise, or continuous densities.
- **The water's picture settles it (04_8).** Any picture that fits would
  do; the water's is the one at hand. On every pair of 4 × 4 shadows some
  picture casts, the flow's picture has no switch exactly when it is the
  only one (`DT-X4`; `model::built_switches` and its test). Feasibility
  (chapters 02 and 03: equal totals and the water getting every unit
  through) and uniqueness (this chapter: no switch) are two questions with
  two tests; chapter 05 takes up more shadows and the limits.
- **Reading holds.** The card stands whole at least its characters / 15
  seconds after its fade-in, and the closing questions and answers at least
  theirs after the first fades in (the corpus's provisional scene
  convention, not a listening measure).
- **Sources.** Ryser 1957, Theorem 3.1 (read). Batenburg 2006, Theorem 2
  (read; it cites Ryser's 1963 monograph, not read here) states the same
  uniqueness criterion with switching components; here it is derived from
  Theorem 3.1 directly, as above.
- **Budget.** 161 spoken words, 8 lines: about 78 s by the Golay measures.
  The eleven units run about 2.5 s past 04_3's last word, the card's hold
  about 0.9 s past 04_7's and the closing hold about 2.7 s past 04_8's, so
  about 84 s in all, near the 85 s limit; no voice has measured it.
- **Narration units.** 04u1 to 04u4 are a proposed grouping: the question
  and the switch (04u1), the tree and the prediction (04u2), the answer and
  why (04u3), the two questions answered (04u4). Boundaries are not frozen
  and nothing is synthesized.

## 05_a_third_shadow: A third shadow (~71 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Back to the face's sixty-four pictures. Take one more shadow and count the filled squares along each diagonal. | The face on the left, "the face" above it, its counts 2, 2, 0, 2, 3 left of the rows and 1, 3, 1, 3, 1 under the columns *(face.rows, face.cols)*; on "along each diagonal", teal beams run along the nine diagonals from lower left to upper right, one after another, and each leaves its count one step past its upper-right end: 0, 1, 1, 2, 1, 1, 1, 2, 0 *(face.diag)*, and the dim label "diagonal shadow" | lagged reveal, grid | |
| 05_2 | A switch keeps every row and column count, but always changes a diagonal count. This one takes a square from two diagonals and gives both to a third. | On "A switch keeps", chapter 01's opening switch, its gold rectangle and rings; its filled corners empty as its empty corners fill, and the row and column counts flash, unchanged *(switch)*; on "always changes", the diagonal counts it changes become the other picture's, red: 0, 3 and 1 *(twin.diag, twin.lose, twin.gain)*; on "This one takes", those three flash | highlight | |
| 05_3 | So none of the fourteen pictures one switch from the face can pass. ⟨predict⟩ How many of all sixty-four do? | The switch is undone and the face's diagonal counts return; chapter 01's gallery of 64 on the right, the face's frame white, "64 pictures with the face's rows and columns" above it *(fits)*; on "none of the fourteen", the fourteen thumbnails one switch from the face fade *(one_switch, one_switch.pass)*; on "all sixty-four", the face's diagonal counts flash; the think ring, four seconds, the voice silent, nothing moving | lagged reveal, tracker | |
| 05_4 | Two. Sixty-two fail, but one other picture casts all three of the face's shadows. | The ring goes; on "Sixty-two fail", the other forty-eight that fail fade, and the caption becomes "2 of 64 cast the face's diagonal shadow too" *(survivors, dropped)*; on "one other picture", the gallery goes as that caption moves over the right side, where it stays, and the other picture stands on the right, "another picture" above it, its row, column and diagonal counts all green *(other)* | highlight | |
| 05_5 | They differ in six squares. Each row, column and diagonal through them loses one square and gains one, so all three shadows stay the same. | On "six squares", gold rings on the six squares in both pictures *(other.differ)*; on "Each row", the three rows', three columns' and three diagonals' counts through them flash on the right, and "each row, column and diagonal through them: −1 +1" *(other.lines, other.loses, other.gains)*; on "all three shadows", every count on screen flashes | highlight | |
| 05_6 | Two shadows left sixty-four pictures. A third still leaves two, so here even three shadows lie. | On "A third still leaves two", the "2 of 64" caption goes and "3 shadows, 2 pictures" stands above the two pictures *(shadows, survivors)* | highlight | |
| 05_7 | So keep the two questions apart. Does any picture fit? And is it the only one? | The other picture, its counts and marks go; a "precisely" lower-third bounding the model: everything in this video is about 5 by 5 pictures of filled and empty squares, counted exactly along rows, columns and here one diagonal direction *(size)*; on the right, chapter 01's two questions, each with its answer for these three shadows as it is asked: "Does any picture fit?", "yes: the face, and one more"; "Is it the only one?", "no: 2 pictures cast all 3 shadows" *(questions, survivors, only_one)*; held until they can be read, then the fade | lower-third, highlight | |

### Notes for 05

- **The 64 are chapter 01's.** The pictures tested are exactly the ones
  that fit the face's row and column counts (`facts::gallery`, the same 64
  `model::fits` enumerates); the third shadow is `model::diagonal_counts`,
  the squares on each diagonal from lower left to upper right (diagonal k:
  row plus column equals k, nine of them on a 5 × 5 grid). The survivors
  are `facts::diagonal_survivors`: the face and 01010/01100/00000/00011/11010,
  two switches from the face.
- **A switch always changes a diagonal count (05_2).** Its four corners
  stand on three or four diagonals. The diagonal through its top-left
  corner holds no other corner: rows r0 < r1 and columns c0 < c1 give
  r0 + c0 less than each of r0 + c1, r1 + c0 and r1 + c1. So that diagonal
  gains or loses a square, whichever way the switch goes (`DT-X6`: no switch
  of any 4 × 4 picture keeps the diagonal shadow; a `model` test; none of
  the face's 14 switches, `DT-F11.face.switches.keeping`). The opening
  switch empties squares on diagonals 1 and 7 and fills two on diagonal 4.
- **Prediction (05_3).** Because a single switch always changes a
  diagonal count, none of the 14 pictures one switch from the face can pass
  (`DT-F11.one_switch.pass` = 0); the viewer can reason that far. The
  question ends unit 05u1; the think ring drains for four seconds with the
  voice silent and nothing moving; "Two." opens 05u2. Nothing on screen
  shows which of the other 50 pass before then.
- **Six squares, every line balanced (05_5).** The face holds (1, 3),
  (3, 0) and (4, 2); the other picture holds (1, 2), (3, 3) and (4, 0)
  instead. Rows 1, 3 and 4, columns 0, 2 and 3 and diagonals 3, 4 and 6
  each hold one square of each, so each loses one and gains one
  (`facts::line_changes`; the oracle walks every line). It plays the part a
  switch plays for two shadows; no general claim is made about such moves.
- **What is and is not claimed (05_6, 05_7).** Only this example: with
  rows, columns and these diagonals, two pictures fit the face's counts.
  The video does not say that a third direction, or any number of
  directions, settles uniqueness or fails to; it makes no claim about how
  hard reconstruction is in general. For the notes only, this example
  again: the other diagonal direction alone also leaves two of the 64, and
  both diagonal directions together leave the face alone (the oracle).
- **The complexity claim is omitted.** The planned line said
  reconstruction from three or more directions is NP-hard in general
  (Gardner, Gritzmann and Prangenberg, Discrete Mathematics 202 (1999)
  45–71). On 2026-10-19 the original could not be read: the publisher's
  page and its PDF returned 403 to this lane's fetches, and the two
  university repositories found answered with a bot challenge, which was not
  bypassed. Abstracts and secondary citations are not used as proof, so the
  video says nothing about it; it can return if the paper is read and its
  exact problem, directions and data assumptions are recorded.
- **No scanner comparison.** The card bounds the model to the video's own
  pictures and counts; nothing is said about real tomography.
- **Reading holds.** The card stands whole at least its characters / 15
  seconds after its fade-in, the closing questions and answers at least
  theirs after the first fades in (the provisional scene convention). The
  "2 of 64" result was first readable for only about 0.45 s in draft09
  (independent frame review, 2026-10-19); it now stays over the other
  picture from "one other picture" until 05_6's words replace it, without
  delaying that picture.
- **Budget.** 136 spoken words, 7 lines: about 67 s by the Golay measures,
  about 71 s with the closing hold; no voice has measured it.
- **Narration units.** 05u1 to 05u4 are a proposed grouping: the new shadow
  and the question (05u1), the answer and the six squares (05u2), the
  conclusion (05u3), the two questions (05u4). Boundaries are not frozen and
  nothing is synthesized.
