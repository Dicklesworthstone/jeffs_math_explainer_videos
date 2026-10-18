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
the middle), **gap** (the square between the corner tiles), **proof**;
from chapter 02 **mark** (a point a third or two thirds along a side),
**piece** (what the marks cut a side into), **shadow** (a tile's extent on
the floor or the wall), **pin** (where a floor mark meets a wall mark) and
**strictly inside** (inside, not at an edge); from chapter 05 **cut** (the
part of the points' line inside a tile) and **slip through** (a tile
dodging both points).
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
| marks and pins | purple `#9A72AC` | the floor and wall marks, the four pins, a shadow's bar |
| a caught pin | yellow `#FFFF00` | a thin ring round a mark or pin strictly inside a tile (highlight only); layers keep the red overlaps over the tiles and the pins and rings over both |

## 3 The arc

| Act | Chapters | Spine |
|---|---|---|
| I. A turn that helps | 01 the turned tile | the obvious box is three wide; turning the middle tile lets a squeeze reach 2 + 1/√2; that picture shows five tiles fit, not that nothing smaller does |
| II. Why straight tiles need three | 02 four pins | marks at a third and two thirds cut each side into pieces shorter than a tile, so every straight tile in a box narrower than three holds a pin strictly inside; tiles cannot share one, so at most four fit; at exactly three the pins sit on edges and five fit; the turned tile holds no pin, so the argument cannot reach it |
| III. Can it wiggle? | 03 a turn by a hair | the pins miss the turned tile (02's last picture), so what holds it? With the box, the corner tiles and the middle tile's centre held, the corners lie on the circle inside the middle tile; any turn moves the sides off them, so they sink in by ½(1 − cos δ), the square of the turn, tiny but never zero; magnified honestly; a local fact about one motion, not a proof about all arrangements |
| IV. Points no tile can dodge: the corners | 04 the corner lemma | Friedman's Lemma 1, proved by projections: any tile in a corner, its centre within one tile of both walls, covers the point one tile from both walls; turning widens the shadow, the walls push the centre out by half of it, and the point stays within s (1 − s/2) = ½ − ½ (s − 1)² ≤ ½ along the tile's sides; equality only for a straight tile on a wall; drop an assumption and it fails; in the box of 2.71 this settles the four corner blocks of centres and leaves the edges and the middle |
| V. Points no tile can dodge: the edges | 05 near a side | Friedman's Lemma 2 with the points a gap apart, one tile up: a tile centred between them, within one tile of the floor, cuts their line above its centre, so dodging both means that cut fits between them; lower is narrower, so the tile rests on the floor, one shadow s tall, and its cut, the base of the corner above the line, beats the gap unless the points stand above h = s − gap (s² − 1)/2; h bends down, so it is least at an end, exactly one straight, where the points stand (not at an eighth, as Friedman's sketch has it); drop the floor, raise the points or widen the gap and a tile slips through; in the box of 2.71 this settles the four side strips and leaves the middle |
| VI. Points no tile can dodge: the middle | 06 a triangle of points | Friedman's Lemma 3 for centres in the middle (not boarded) |
| VII. Nothing smaller fits | 07 four points, five tiles | every tile in the box of 2.71 covers one of the four points; the scale-up that turns that into "five tiles need at least 2 + 1/√2" (not boarded) |
| VIII. Beyond five | 08 what is proved and what is found | the same three questions for ten and eleven tiles: a record found by search is a fit, not a proof (not boarded) |

Chapters 06 to 08 are boarded only after their mathematics is in `model.rs`
with an oracle, one lemma a chapter, so the lower bound gets the time it
needs; until chapter 07 is checked, nothing claims that 2 + 1/√2 is the
smallest box.

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

## 02_four_pins: Four pins (budgeted from the measured chapter 01 timings: about 81 s)

The proof, in the order the picture builds it (`model.rs` §four pins has the
written proof; every step's exact check is a fact). The box is the turned
packing's, 2 + 1/√2 wide, so the pins can meet the turned tile at the end.
Below three, every piece is shorter than a tile; a box narrower than one tile
holds no tile at all (on screen in 02_6's "precisely" line). The two
assumptions are said in words: the tiles are straight, and their insides do
not overlap.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Why do straight tiles need a box three wide? Take a narrower box, and mark its floor a third and two thirds of the way along. | The empty box of the turned packing *(turned.side.2dp)*; at "mark its floor", two purple ticks on the floor at a third and two thirds, labelled ⅓ and ⅔ *(model::marks)* | lagged reveal | |
| 02_2 | The marks cut the floor into three equal pieces, and each is shorter than one tile. | Live readouts "box 2.71" and "each piece 0.90 < 1" *(turned.side.2dp, pins.piece.2dp)*; one tracker widens the box towards three and back, the marks moving with it, the piece readout staying under 1 | tracker, live readout | |
| 02_3 | So a straight tile's shadow on the floor can't fit inside one piece. A mark always lands strictly inside it, away from its ends. | A straight blue tile slides along the floor and back, its shadow a purple bar under the floor; the mark strictly inside the shadow wears a yellow halo at every frame *(pins.lemma)* | tracker, highlight | |
| 02_4 | Mark the left wall the same way. Then every straight tile has one of these four pins strictly inside it. | Wall ticks; four purple pins where the marks cross *(pins.count)*; the tile wanders through the box on a loop (`model::wander`), the pin strictly inside it haloed at every frame *(pins.straight_lemma)* | tracker, highlight, grid | |
| 02_5 | Two tiles can't share a pin, because their insides never overlap. ⟨predict⟩ So how many straight tiles fit? | The wandering tile leaves; four blue tiles in the corners, each with its own pin haloed *(pins.owners.straight)*; the think ring drains over four seconds, the voice silent | lagged reveal, tracker | listen 4 s (think) |
| 02_6 | Four at most, one per pin. So five straight tiles need a box at least three wide. | A fifth straight tile lands in the middle: it holds all four pins and the corners overlap it in red *(pins.owners.straight, planted.straight_squeezed)*; "at most 4 straight tiles"; a "precisely" line: any box narrower than 3, and one narrower than one tile holds none *(pins.count, pins.threshold, pins.lemma.half)* | highlight | |
| 02_7 | At exactly three, each piece is a whole tile long. Tiles can then touch the marks with their edges only, and five fit. | The box widens to three with the five straight tiles (`model::corners_and_middle`): the red overlaps shrink to nothing and every halo goes out as the pins reach tile edges *(pins.lemma.at3, pins.owners.at3, straight.check)*; "3 ÷ 3 = 1" | tracker, formula | |
| 02_8 | But this only counts straight tiles. In the turned packing, no pin is inside the turned tile, so the pins can't rule out its smaller box. | The middle tile becomes gold and turns (`model::turning`), then the squeeze (`model::squeezed`) carries the box back to 2.71, the pins with it: each corner tile's pin haloed, none on the gold tile *(pins.owners.turned)*; "no pin inside" by the gold tile | tracker, highlight | |

## 03_a_turn_by_a_hair: A turn by a hair (budgeted from measured timings: about 80 s)

Held throughout, and said: the box of 2.71, the four corner tiles and the
middle tile's centre. Only the middle tile turns. The exact turn shown is
the one whose half-angle has tangent 1/20 (about 5.7°), then twice that; the
board draws them at true scale, where the bite is under a pixel, and the
lens draws one corner 44 times larger with the same geometry and a scale
bar. The depth ½(1 − cos δ) is exact for every turn below an eighth of a
turn; the triangle the lens shows is the whole overlap only up to 30°, and
the turns shown are far inside that (`model.rs` § a turn by a hair).

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | The pins miss the turned tile, but something still holds it. Fix the box, the corner tiles and the middle tile's centre. | The turned packing at true scale; a dot on the middle tile's centre; "held fixed: the box, the corner tiles, the centre" | lagged reveal | |
| 03_2 | Each corner it touches sits half a tile from that centre, on the circle that just fits inside the middle tile. | The circle inside the gold tile, its radius drawn and labelled ½ *(hair.inner2 = hair.inradius2)*; the four held corners as green dots on it | circle, highlight | |
| 03_3 | Now turn the middle tile just a little. ⟨predict⟩ Do the corners still only touch it? | The picture holds; the think ring drains over four seconds, the voice silent | tracker | listen 4 s (think) |
| 03_4 | No. The circle turns in place, but the sides now meet it elsewhere, so every corner on it ends up inside the tile. | One tracker turns the gold tile about its held centre by the exact small turn (`model::hair`) *(hair.turn, hair.degrees.1dp)*; small dots ride the middles of its sides, off the green corners; the overlaps, under a pixel here, are drawn in red *(hair.check)* | tracker, live readout | |
| 03_5 | Here is one corner, magnified forty four times. Turn twice as far, and it sinks almost four times deeper. | A square lens joined to the lower right corner: the same geometry 44 times larger *(hair.lens.mag)*, a 0.01-tile scale bar *(hair.lens.bar)*, the red triangle of overlap *(hair.area, hair.vertices)*; a live "bite" readout 0.0025 *(hair.depth.4dp)*; the turn doubles *(hair.degrees2.1dp)*, the bite reads 0.0100 *(hair.depth2.4dp)*, "×3.99 deeper" *(hair.ratio.2dp)*; the window holds it *(hair.lens.fits)* | tracker, live readout, highlight | |
| 03_6 | A circle leaves its tangent slowly, so the bite grows like the square of the turn. It is tiny, but never zero. | Beside the lens, bite = ½(1 − cos turn) ≈ ¼ turn², turn in radians about the fixed centre *(hair.depth)*; ever smaller turns still bite *(hair.tiny)* | formula | |
| 03_7 | In the box of three, the corners sat on the outer circle, and the tile turned freely. Held here, it cannot turn at all. | A small box of three in the corner of the panel, its corners on the circle round the turning tile *(turn.inner2, hair.roomy)*; on the board the tile turns the other way, and the corners bite in again *(hair.minus, hair.check.minus)*; then back to contact | tracker | |
| 03_8 | This shows only that this one turn is blocked. Whether any arrangement fits a smaller box needs a proof about all of them. | "blocked: this one turn, with all else held" and "a smaller box? a proof about every arrangement"; the turned packing as it was | box | |

## 04_the_corner_lemma: The corner lemma (budgeted from measured timings: about 80 s)

Friedman's Lemma 1 (fact sheet S2): any unit square inside the first
quadrant whose centre is in [0, 1]² contains (1, 1). His proof reduces to a
square touching both axes; the video's proof is by projections, written out
in `model.rs` § the corner lemma, with no reduction. "Covers" means inside or
on the edge. The corner view shows the quadrant two and a half times larger
than the box of chapter 01; the last beat returns to the box at its usual
scale.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | To rule out a smaller box, we need points no tile can dodge. Start with a tile in a corner. | A corner of a box, close up: the floor and the left wall; a purple point one tile from both *(corner.point)*; a blue tile in the corner | lagged reveal | |
| 04_2 | If its centre is within one tile of both walls, it always covers this point, one tile across and one tile up. | The square of centres within one tile of both walls, outlined; the tile wanders through varied allowed positions and turns (`model::corner_wander`: examples; the proof covers every one), its centre a dot, a ring on the point yellow while it is inside and green on the edge | tracker, highlight | |
| 04_3 | A turned tile casts a wider shadow on the floor. ⟨predict⟩ So could a turned tile slip past the point? | The tile's shadow, a purple bar on the floor, with a live readout from 1 to 1.41 *(corner.shadow.min, corner.shadow.max.2dp)*; the picture holds while the think ring drains | tracker, live readout | listen 4 s (think) |
| 04_4 | No. The walls keep its centre at least half a shadow out, so across and up, the point is at most one minus half a shadow away. | The tile pushed into the corner as it turns (`model::corner_pushed`), the centres it allows shaded, the point's gap across and its gap up drawn from the centre (each at most 1 − s/2; the straight-line distance is larger, √2 − 1 at an eighth of a turn, which is why 04_5 measures along the tile's sides) | tracker | |
| 04_5 | Along the tile's own sides, that is at most the shadow times one minus half the shadow, which never passes one half. | The tile's two half-sides drawn from its centre; live readouts of the point's distance along each, both under 0.50 *(corner.bound.max, corner.bound.eighth)*; s (1 − s/2) = ½ − ½ (s − 1)² ≤ ½ | formula, live readout | |
| 04_6 | Half a tile is how far a tile reaches, so the point stays inside. Only a straight tile resting on a wall just touches it. | The tile turns back to straight in the corner, then slides up the left wall: the ring turns green, the point on its edge *(corner.edges, corner.turned, corner.margin.eighth)* | tracker, highlight | |
| 04_7 | Let the centre stray past one tile, or drop a wall, and the point can escape. | First the bound, its caption and the edge status leave (they hold only under the assumptions) and "an assumption dropped" arrives; the straight tile slides out past one tile: the ring turns red and the live distance along its side passes 0.50 *(corner.fail.centre)*; at "drop a wall", a turned tile pokes through both walls, its centre too close: red again *(corner.fail.wall)* | tracker, highlight | |
| 04_8 | In the box of two point seven one, this settles centres near the corners. The edges and the middle come next. | The box of 2.71 at the usual scale: Friedman's four points *(corner.box.points)*; the region of every centre, its four corner blocks green *(corner.box.block, corner.box.covered)*, the cross between them grey *(corner.box.left, corner.box.middle)*: "settled: centres within one tile of a corner" and "next: the edges and the middle" | box | |

## 05_near_a_side: Near a side (budgeted against 04's measured timing: about 80 s)

Friedman's Lemma 2 (fact sheet S2): for 0 < x ≤ 1, 0 < y ≤ 1 and
x + 2y < 2√2, any unit square inside the first quadrant whose centre is in
[1, 1 + x] × [0, y] contains (1, y) or (1 + x, y). The video's case is the
box's: x the gap, 1/√2, and y = 1. The proof is reconstructed in `model.rs`
§ the side strip (straight up; dodging is fitting between; lower is
narrower; the cut over the top corner; the slip height h, a parabola
opening downward in the shadow, least at an end). Friedman's sketch puts
the least at 45°, which is false for the gap (h is 1 straight and 3√2/4 at
an eighth); the screen says only "lowest at the straight end, not the
eighth". "Covers" means inside or on the edge. The side view shows the
floor of the box of 2.71 at 2.2 scene units a tile; the first and last
beats show the box at its usual scale.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Near a side, one point is not enough, so take two. A tile centred between them, within a tile of the floor, covers one. | The box of 2.71 as 04 left it, its bottom strip of centres purple; then the floor close up, the corner blocks of centres green: 04's stray tile misses the corner's point, its ring faint; the second point a gap along, its ring green *(strip.one, strip.points)*; the strip of centres the tile's turn allows; the tile wanders through varied allowed positions and turns (`model::strip_wander`: examples; the proof covers every one), each ring yellow while its point is inside and green on the edge | lagged reveal, tracker, highlight | |
| 05_2 | Turned an eighth, its top is a narrow corner. ⟨predict⟩ Could it slip up between the points? | Its shadow s, a purple bar under the floor, and live readouts of the turn and the shadow; the tile rolls on the floor to an eighth of a turn between the points; the picture holds while the think ring drains | tracker, live readout | listen 4 s (think) |
| 05_3 | No. The points are at most half a tile above its centre, and a tile reaches that far straight up. | From the centre straight up: the distance to the points' line (at most 1 − s/2 ≤ ½) and the half tile the tile reaches, past the line; the tile lifted to 0.95 keeps both | tracker | |
| 05_4 | Dodging both, its cut on their line must fit between them. Lower is narrower, so rest it on the floor. | The cut along the points' line, drawn, and live readouts of the gap and the cut, read off the frame *(turned.gap.2dp)*; the tile lowered to the floor, the cut shrinking to 0.83 *(strip.cut.eighth.2dp)* | tracker, live readout | |
| 05_5 | Resting, it stands one shadow tall. Split its corner above their line into two right triangles, whose bases add to the cut. It is shorter than the gap only when the points stand higher than this. | A purple bar beside the tile, one shadow tall; the tile rolls to 30° so the two triangles differ; the corner above the points' line split at its top into two gold right triangles: the vertical d = s − y (y the points' height), the turn θ at the top inside the right one, the bases labelled d cot θ and d tan θ; cut = d tan θ + d cot θ = d/(sin θ cos θ); that equation replaced by cut < gap only if y > s − gap sin θ cos θ; that replaced, with s² = 1 + 2 sin θ cos θ, by y > s − gap (s² − 1)/2 = h | formula | |
| 05_6 | That height bends down, so it is lowest at an end, exactly one when straight. The points stand at one, so nothing slips through. | A plot of h against the shadow, from straight to an eighth, above its chord, the points' height a purple line, the eighth's end labelled 1.06 *(strip.height.eighth.2dp)*; the tile rolls back to straight on the floor and a dot runs along the curve to its lower end, labelled 1 *(strip.height.straight)*: "lowest at the straight end, not the eighth" *(strip.least)*; both points on the straight tile's top edge, rings green: "straight, on the floor: on the edge" *(strip.edges, strip.turned)* | plot, tracker, highlight | |
| 05_7 | Drop the floor, raise the points, or widen the gap, and a tile slips through. | First the formula, the plot and the edge status leave (they hold only under the assumptions) and "an assumption dropped" arrives; from the straight tile on the floor: an eighth-turned tile half a tile up pokes through the floor, both rings red *(strip.fail.floor)*; the points raised to 1.05, the straight tile passes under them *(strip.fail.high: x + 2y < 2√2 still holds)*; the gap widened to a whole tile, the gap readout 1.00, an eighth-turned tile on the floor slips between *(strip.fail.wide)* | tracker, highlight | |
| 05_8 | Turned to every side, this settles centres near a side. Only the middle is left. | The box of 2.71 at the usual scale: the corner blocks green, the four side strips turning green *(strip.box.strip, strip.box.covered)*, the middle square grey *(strip.box.left)*: "settled: centres near a corner or a side" and "next: the middle" | box | |
