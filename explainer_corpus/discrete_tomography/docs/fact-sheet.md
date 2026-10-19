# Fact Sheet — Two Shadows Can Lie

Every number shown or spoken. Each row's value is computed in `src/facts.rs`
(from `src/model.rs`) and recomputed independently by `oracle.py`; `./x
discrete_tomography facts-check` compares the two outputs key by key. Pinned
rows also carry a literal checked at startup. Pictures are written row by row
from the top, `1` filled and `0` empty, rows joined by `/`; squares are
(row, column) from 0.

The oracle shares no code with the model: it generates pictures column by
column (the model row by row), finds switches from pairs of filled squares
(the model from pairs of rows times pairs of columns), takes the largest flow
as the smallest cut in closed form and by depth-first augmenting paths (the
model by shortest augmenting paths), and decides feasibility by Gale's own
dual-sequence test and n-step construction.

| id | quantity | shown / said | computed in | oracle key | value | source |
|---|---|---|---|---|---|---|
| F1 | the grid | a 5 × 5 grid (01_1) | `model::FACE` | `DT-F1.size` | `5,5` | pinned |
| F1 | shadows, one per direction | "two shadows" (01_2) | rows and columns | `DT-F1.directions` | `2` | pinned |
| F1 | the face | the picture (01_1, 01_4, 01_8's first thumbnail) | `model::face` | `DT-F1.face` | `01010/01010/00000/10001/01110` | pinned: the pattern |
| F1 | its row and column counts | 2, 2, 0, 2, 3 and 1, 3, 1, 3, 1 (01_2 on) | `Picture::row_counts`, `col_counts` | `DT-F1.face.rows`, `.cols` | `2,2,0,2,3`; `1,3,1,3,1` | computed |
| F1 | its filled squares; its empty row | 9 squares; "the middle row is empty" (01_10) | `Picture::count`; rows with count 0 | `DT-F1.face.total`, `.empty_rows` | `9`; `2` | computed |
| F1 | a picture fits its shadows (Gale's test) | — (chapter 03) | `model::gale_ryser` | `DT-F1.face.gale` | `true` | computed |
| F2 | the opening switch | the gold rectangle (01_5) | `model::OPENING` | `DT-F2.switch` | rows `0,3`; columns `1,4` | computed |
| F2 | the other picture and its counts | "another picture" (01_4); green counts | `facts::twin` | `DT-F2.twin`, `.twin.rows`, `.twin.cols` | `00011/01010/00000/11000/01110`; `2,2,0,2,3`; `1,3,1,3,1` | computed |
| F2 | squares where they differ | "four squares" (01_5); the rings | `Picture::differences` | `DT-F2.differ`, `.differ.cells` | `4`; `0,1;0,4;3,1;3,4` | computed |
| F2 | what each of the switch's rows and columns loses and gains | "loses one square and gains one" (01_6); "−1 +1" | `Switch::sides` per line | `DT-F2.loses`, `.gains` | `1`, `1` | computed |
| F3 | pictures with the face's shadows | "Sixty-four" (01_8); the gallery, "64 pictures" | `model::fits` | `DT-F3.fits` | `64` | pinned (the oracle's column-by-column enumeration) |
| F3 | switches in the face | — | `model::switches` | `DT-F3.face.switches` | `14` | computed |
| F3 | pictures 0, 1, 2, 3 switches from the face | the gallery's four waves (01_8) | `model::reachable` | `DT-F3.rings` | `1,14,33,16` | computed |
| F3 | the farthest | "at most three switches" (01_8); the caption | same | `DT-F3.farthest` | `3` | computed |
| F3 | every picture that fits is reached by switches | the walk; the card | same against `fits` | `DT-F3.connected` | `true` | computed |
| F3 | the walk's end and length | the big picture's three switches (01_8) | `facts::walk` | `DT-F3.walk.end`, `.walk.steps` | `00011/01100/00000/01010/11010`; `3` | computed |
| F4 | other column counts | 5, 1, 1, 1, 1 (01_9) | `model::NO_FIT_COLS` | `DT-F4.cols` | `5,1,1,1,1` | computed |
| F4 | both totals | "keep their total"; "both shadows add up to 9" (01_9) | `Shadows::row_total`, `col_total` | `DT-F4.totals` | `9,9` | computed |
| F4 | pictures that fit them | "no picture fits" (01_9) | `model::fits` | `DT-F4.fits` | `0` | pinned |
| F4 | Gale's test; the largest flow; the cut | — (chapter 03) | `gale_ryser`; `max_flow` in `reconstruct`; the columns the flow cannot reach | `DT-F4.gale`, `.flow`, `.cut` | `false`; `8`; `0` | computed; the oracle lists every cut and keeps the smallest source side |
| F4 | the bottleneck: the column, what it asks, what the rows can give | the red 5 and five rings (01_10) | `model::bottleneck`, `can_give` | `DT-F4.bottleneck`, `.bottleneck.asks`, `.bottleneck.can_give` | `0`; `5`; `4` | computed |
| F5 | the face's shadows: the largest flow, and its picture fits | — (chapter 02) | `max_flow`, `reconstruct` | `DT-F5.face.flow`, `.face.built` | `9`; `true` | computed; the oracle builds by Gale's n-step method |
| F6 | the tree, its counts, pictures that fit, switches | — (chapter 04) | `model::tree` | `DT-F6.tree`, `.tree.rows`, `.tree.cols`, `.tree.fits`, `.tree.switches` | `00100/01110/11111/00100/00100`; `1,3,5,1,1`; `1,2,5,2,1`; `1`; `0` | computed |
| F7 | questions on screen | "two questions" (01_11) | `same_shadows::QUESTIONS` | `DT-F7.questions` | `2` | computed; the oracle reads the scene's source |
| S1 | Ryser's year | the card (01_8) | `facts::RYSER_YEAR` | `DT-S1.ryser` | `1957` | pinned: Canad. J. Math. 9 (1957) 371–377 |
| S2 | Ford and Fulkerson's year | the card (02_9) | `facts::FORD_FULKERSON_YEAR` | `DT-S2.ford_fulkerson` | `1957` | pinned: Canad. J. Math. 9 (1957) 210–218 |
| S3 | Gale's year | the card (02_9) | `facts::GALE_YEAR` | `DT-S3.gale` | `1957` | pinned: Pacific J. Math. 7 (1957) 1073–1082 |

### Chapter 02 (the face's shadows as plumbing; the method's real paths)

The paths are the ones `model::max_flow` sent, recorded as it sent them, and
every state is rebuilt from them by `model::replay`, which refuses a path
that visits a row or a column twice or breaks a capacity; the balance of
every row and column, which those rules imply, is asserted after each path
as an invariant. The oracle derives its own nine paths (room per directed
edge in a dictionary, the same breadth-first scan order) and checks every
state with its own bookkeeping.

| id | quantity | shown / said | computed in | oracle key | value | source |
|---|---|---|---|---|---|---|
| F8 | a square's pipe carries at most | "at most one unit" (02_3); "square: carries at most 1 unit" | `model::SQUARE` | `DT-F8.square` | `1` | pinned |
| F8 | paths, and what each sends | nine units, "one unit at a time" (02_4) | `Flow::paths` | `DT-F8.paths`, `DT-F8.room` | `9`; `1` | computed |
| F8 | the paths, rows and columns in order | every route drawn (02_2, 02_4, 02_6, 02_7) | `facts::face_flow` | `DT-F8.trace` | `r0c0,r0c1,r1c1,r1c2,r3c1,r3c3,r4c3,r4c4,r4c0r0c3` | computed |
| F8 | straight paths; paths that take water back | eight, then one (02_4 to 02_7) | `Augmenting::takes_back` | `DT-F8.direct`, `.takebacks` | `8`; `1` | computed |
| F8 | before the take-back: through, the wet squares, a path without taking back | "Eight are through" (02_5) | `model::replay`, `model::forward_room` | `DT-F8.before.through`, `.before.wet`, `.before.forward_room` | `8`; `11000/01100/00000/01010/00011`; `false` | computed |
| F8 | the stuck row, what it has left, its dry squares' columns (drains full) | "The bottom row has one left, but its dry squares lead to full drains" (02_5) | same | `DT-F8.stuck.row`, `.stuck.left`, `.stuck.dry_full` | `4`; `1`; `0,1,2` | computed |
| F8 | the take-back: the column it enters, the row it takes back from, the column that row sends on to, what that column still needs | "the first column", "the top row" (02_6); "the fourth column, which still needs one" (02_7) | the ninth path's steps | `DT-F8.takeback.column`, `.row`, `.to_column`, `.to_needs` | `0`; `0`; `3`; `1` | computed |
| F8 | short without taking back; through at the end | "one unit short", "without taking back: 8 of 9"; "through: 9 of 9" (02_7) | total − before; the last state | `DT-F8.short`, `DT-F8.through` | `1`; `9` | computed |
| F8 | the water's picture: pattern, fits, is the face, the only one, switches from the face | "cast the face's shadows, but they aren't the face", "one of the sixty-four" (02_8); "whether that's the only one" (02_9) | the last replayed state | `DT-F8.built`, `.built.fits`, `.built.is_face`, `.built.only_one`, `.built.from_face` | `01010/01100/00000/01010/10011`; `true`; `false`; `false`; `3` | computed |
| X1 | every switch of every 4 × 4 picture keeps both shadows and changes exactly the four corners | "It changes the picture, but neither shadow" (01_7) | `facts::edges` | `DT-X1.pairs`, `.failed` | `294912`; `0` | computed (edge test) |
| X2 | 4 × 4 pairs of shadows some picture casts; how many have all their pictures joined by switches | the card (01_8): Ryser's Theorem 3.1 | same | `DT-X2.shadow_pairs`, `.connected` | `16145`; `16145` | computed (edge test) |
| X3 | 4 × 4 pictures alone with their shadows; with no switch; disagreements | chapter 04's statement | same | `DT-X3.alone`, `.no_switch`, `.disagree` | `6902`; `6902`; `0` | computed (edge test) |

`oracle.py` also checks, on stderr: the count of fits by a memoized count over
columns; that every one of the 64 has a switch and every two of them are at
most 3 switches apart; both flow methods and the listed cuts agree; Gale's
construction is stuck on the no-fit shadows and lands among the 64 on the
face's; Gale's test, Gale's construction and the closed-form flow each agree
with brute force on all 390,625 pairs of 4 × 4 shadows with counts 0 to 4; and
the slate's toy enumeration (3 × 3: 512 pictures, 328 pairs of shadows, 98
shared, largest 6) recomputed. For chapter 02 it checks that exactly one path
takes water back and that it is the last; that the stuck row's dry squares
all lead to full drains and the water taken back had been sent there by that
row; that the flow reaches the smallest cut; that its checker refuses eight
broken traces (a unit wetting a wet square, from an empty tap, into a full
drain, taking back from a dry square, two units at once, a path that skips
its column, the take-back path on its own, and the cyclic walk source, row 0,
column 0, row 0, column 1, sink) and accepts the real one, and that with its
no-repeat rule switched off the same checker accepts the cyclic walk (so that
rule alone refuses it); and that on every 3 × 3 pair of shadows with equal
totals its trace replays and reaches the smallest cut.

## Non-numeric claims

| claim | where | evidence |
|---|---|---|
| A switch changes a picture and neither shadow | 01_6, 01_7 | X1; `model` test on every switch of every 4 × 4 picture; Ryser 1957 §3 ("the resulting matrix A* has row sum vector R and column sum vector S") |
| Any two pictures with the same row and column counts are joined by a chain of switches | the 01_8 card | Ryser 1957, Theorem 3.1, p. 373 (read); X2 |
| A picture is the only one with its shadows exactly when it has no switch | chapter 04 (planned) | Batenburg 2006, Theorem 2 (citing Ryser's 1963 monograph, not read); follows from Theorem 3.1; X3 |
| A picture fits exactly when the totals agree and every k columns ask for at most what the rows can give k columns | chapter 03 (planned) | Gale 1957, p. 1080 (read: the dual sequence, the theorem, and the flow into k columns, Σ min(a_i, k)); Ryser 1957, Theorem 2.1 and its "added in proof" note crediting Gale |
| A full flow in Gale's network is a picture that fits; a picture that fits is a full flow | 02_3 ("Get every unit through, and the wet squares are a picture that fits"), the 02_9 card | Gale 1957, pp. 1079–1080 (read); Batenburg 2006, §2.2 (read) |
| Sending whole units along paths with room, taking back where needed, ends at a largest flow | 02_9 ("Water finds a picture whenever one exists") and its card | Ford and Fulkerson 1957, pp. 212–213 (read: the labeling method on the room matrix, termination because each step sends a positive whole number, Lemma 1 that the result is a flow, Lemma 2 that it is a largest one); with Gale's equivalence, a picture exists exactly when both shadows share one total and that flow reaches it (rows 1, 1 with columns 2, 1 reach their row total and fit nothing) |
| Without taking back, this fill stops one unit short | 02_7 | after the first eight paths no row with water left has a dry square into a drain with room (`DT-F8.before.forward_room` = false; `model` test; the oracle's own check) |
| The water's picture is not the only one | 02_8, 02_9 | `DT-F8.built.is_face` = false and `DT-F3.fits` = 64: the method finds a picture, not the face, and says nothing about uniqueness |
| With three or more directions, reconstruction is NP-hard in general | chapter 05 (planned) | Batenburg 2006, §1.1, citing Gardner, Gritzmann and Prangenberg, Discrete Math. 202 (1999) 45–71; **the original not read** |
