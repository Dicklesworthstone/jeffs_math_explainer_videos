# Storyboard — Why Turning a Square Can Make the Whole Box Smaller

The hand-edited source of this video's words and pictures. `python3 -I
tools/storyboard.py square_packing` checks it (gate G2) and writes
`script.tsv` from the words column. Facts: `docs/fact-sheet.md` and
`oracle.py` (every number below names its fact key); brief: `docs/brief.md`.

Words are spoken exactly as written: no colons, semicolons, dashes or
parentheses in them, numbers spelled as they are said, no single letters.
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a question for the
viewer; the beat ends with the question and the picture holds before the next
beat answers it.

**Three questions, never mixed.** The video keeps apart *does it fit?* (a
packing, checked tile against tile and tile against wall), *can it move
there?* (a motion whose every frame is a packing) and *could anything smaller
fit?* (a proof about every possible arrangement). A picture answers the
first; a motion the second; only an argument about all arrangements the
third. A frame in which tiles overlap is never called a packing or a path: it
is shown in `warn` red as what fails.

## 1 Vocabulary (one word per idea)

**tile** (the unit squares; never "square" for a piece, never "unit"),
**box** (the container; never "container" or "frame"), **wide** (the box's
side), **straight** (lined up with the walls), **turn** (never "rotate" or
"tilt"), **squeeze** (the box shrinking with the corner tiles), **touch**,
**overlap**, **fit**, **inner corner** (the corner of a corner tile nearest
the middle), **gap** (the square between the corner tiles), **proof**.
"Packing" is the formal word, on screen once (01_11), never spoken in
chapter 01.

## 2 Colour roles

| Role | Colour | What wears it |
|---|---|---|
| the turned tile | hero gold `#FFD866` | the middle tile, all video long |
| corner tiles | blue `#58C4DD` | the four straight tiles in the corners |
| the box | ink `#ECEFF4` | the outline that shrinks |
| room, reach, ghosts | faint `#3B4352` | empty cells, the circle of reach, the old box of three |
| a touch | green `#83C167` | contact dots (never the only carrier: a dot, not a tint) |
| what fails | warn red `#FC6255` | overlap patches, the word "overlap" |
| measures | dim `#8A93A3` | the diagonal, the gap's side, labels |

## 3 The arc

| Act | Chapters | Spine |
|---|---|---|
| I. A turn that helps | 01 the turned tile | the obvious box is three wide; turning the middle tile lets a squeeze reach 2 + 1/√2; that picture shows five tiles fit, not that nothing smaller does |
| II. Why straight tiles need three | 02 four pins | four pins placed in any box narrower than three: every straight tile in that box swallows one, so at most four straight tiles fit (the argument of the brief, exact for straight tiles only) |
| III. Why the turn escapes | 03 slipping between pins | the turned tile holds none of the four pins; the same picture as 01, now with the pins: a turned tile is not caught by the straight argument |
| IV. Can it wiggle? | 04 a turn by a hair | local motion at the fixed box: the middle tile's turn costs nothing at first and then bites by the square of the turn (reported by the case page; recomputed in `model.rs` before it is scripted) |
| V. Nothing smaller fits | 05 four points no tile can dodge | Göbel's four unavoidable points for any tile, turned or not, in the box 2 + 1/√2 (Friedman's Theorem 2 with Lemmas 1–3), and the scale-up that turns "touch" into "too many" |
| VI. Beyond five | 06 what is proved and what is found | the same three questions for ten and eleven tiles: a record found by search is a fit, not a proof |

Chapters 02 to 06 are not boarded yet: each is boarded only after its
mathematics is in `model.rs` with an oracle, and chapter 05 only after the
lower-bound proof is reconstructed and checked (the slate's gate: if it cannot
be taught honestly, the video ends on construction versus proof instead).

The payoff is previewed in chapter 01 itself (the box of 2.71 against the
grid's 3); the video ends on the rigorous form of the opening picture: the
same five tiles with Göbel's four points drawn in, each tile on one.

## 01_the_turned_tile: The turned tile (88.3 s measured in the scratch draft at 8f154b5; trimmed to about 82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Five equal square tiles. How small a square box holds them all? | Five tiles, four blue and one gold, arrive one by one in a loose row *(tiles)*; the question as on-screen text at the top | lagged reveal | |
| 01_2 | Four fit in a box two wide. The fifth has nowhere to go. | The four blue tiles slide into a box two wide *(grid.full, grid.full.holds)*; the gold tile waits outside it | slide | |
| 01_3 | The obvious box is three wide, with room for four more. | The box outline grows to three *(grid.side)*; the blue tiles slide to its corners, the gold tile to its middle *(straight.check)*; four empty cells outlined in faint, one by one *(grid.room)* | morph, lagged reveal, grid | |
| 01_4 | Every tile lines up with the walls. ⟨predict⟩ Could turning one shrink the box? | The empty cells fade; the gold tile is indicated; a think ring beside the question drains over four seconds with the voice silent, and fades as 01_5 begins | highlight (indicate), tracker | listen 4 s (think) |
| 01_5 | Turn the middle tile. Its corners stay on a circle that only touches its neighbours, so nothing collides. | A faint circle through the four inner corners *(turn.reach2 = turn.inner2)*; the gold tile turns an eighth of a turn about its centre, a live readout of the turn counting 0° to 45° *(turned.degrees)* | tracker, live readout, circle | |
| 01_6 | Now squeeze. The corner tiles slide in, and no two tiles ever overlap. | One tracker drives the box width from 3 to 2.71 *(grid.side, turned.side.2dp)*; every tile and the outline are redrawn from `model::squeezed` each frame, a live readout of the width under the box; the box of three stays as a faint ghost | tracker, live readout | |
| 01_7 | They stop when each inner corner touches the middle of a side of the turned tile. | Four green dots at the inner corners, each the midpoint of a side of the gold tile *(turned.touching, turned.midpoints)* | flash | |
| 01_8 | Across the square gap, opposite inner corners are now one tile apart. | The square gap outlined in faint; its diagonal drawn between two opposite inner corners, labelled 1 *(turned.diagonal2)* | grid | |
| 01_9 | A square's diagonal is its side times the square root of two. So the gap is one over the square root of two, and the box is two plus that, about two point seven one. | Beside the box, a derivation: gap × √2 = 1, then gap = 1/√2 *(turned.gap)*, then box = 2 + 1/√2 ≈ 2.71 *(turned.side, turned.side.2dp)*, each step a TransformMatchingTex of the one before; a "precisely" lower-third 2 + √2/2 = 2.70711… *(turned.side.5dp)* | formula, TransformMatchingTex | |
| 01_10 | Any further, and the corners bite in. | The squeeze goes on past the stop: red overlap patches where the corner tiles bite the gold one, the word "overlap" in red *(planted.squeeze_further)*; then it comes back to the stop and the patches go, all within the line | tracker, highlight | |
| 01_11 | This shows five tiles fit. It can't show that nothing smaller fits. That needs a proof. | The packing alone; to its right a tick beside "fits" over 2.71 *(turned.check)* and a question mark beside "smaller?"; a source line "this packing: F. Göbel, 1979" *(source.gobel)*; then the breath and the fade | box | |
