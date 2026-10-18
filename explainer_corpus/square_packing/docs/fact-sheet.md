# Fact sheet — Why Turning a Square Can Make the Whole Box Smaller

Every number the video shows or says, where it comes from, and every source
with what was and was not seen. `./x square_packing facts-check` runs the
video's `facts` (computed in `src/model.rs` and `src/facts.rs`) against
`oracle.py`, which rebuilds every packing by its own route: the turned tile
built directly as a diamond, the box solved from another condition, and
overlap decided by exact polygon clipping and area instead of the video's
separating-axis test. Both work exactly on numbers a + b√2.

Tile order in every list: 0 lower left, 1 lower right, 2 upper left, 3 upper
right, 4 the middle. `a-b` is a pair of tiles.

## Numbers

| Key | Value | Where shown or said | How it is computed |
|---|---|---|---|
| `tiles` | 5 | 01_1 "Five" | the video's count |
| `grid.full`, `grid.full.holds` | 2, 4 | 01_2 "Four fit … two wide" | the largest k with k² ≤ 5 |
| `grid.side` | 3 | 01_3 "three wide"; the ghost box | the smallest k with k² ≥ 5; pinned: case page S1 |
| `grid.room` | 4 | 01_3 "room for four more"; four empty cells | 3² − 5 |
| `straight.check`, `straight.touching` | fits; 0-4,1-4,2-4,3-4 | 01_3's picture | exact check of the straight five in the box of three |
| `turn.reach2`, `turn.inner2` | 1/2, 1/2 | 01_5 "a circle that only touches its neighbours" | squared distance from the turning tile's centre to its corners, and from the box's centre to each inner corner |
| `turned.degrees` | 45 | 01_5 readout | the eighth turn's angle |
| `turned.gap`, `turned.gap.2dp` | 1/2*sqrt(2), 0.71 | 01_9 "one over the square root of two"; the 1/√2 label | the gap's diagonal is 1 and a square's diagonal is √2 times its side (`model::turned_gap`) |
| `turned.side`, `.2dp`, `.5dp` | 2 + 1/2*sqrt(2), 2.71, 2.70711 | 01_9 "two plus that, about two point seven one"; formula; "precisely" line; 01_11 "fits: 2.71" | a tile, the gap and a tile; pinned: S1 (2 + √2/2, 2.7071067811…) and S2 (2 + 1/√2) |
| `turned.diagonal2` | 1 | 01_8 "one tile apart" | squared distance between opposite inner corners at the stop |
| `turned.check`, `turned.touching` | fits; 0-4,1-4,2-4,3-4 | 01_6 "no two tiles ever overlap"; 01_7 | exact check at the stop |
| `turned.walls` | 8 | — (the corner tiles' wall contacts) | exact check |
| `turned.midpoints` | 4 | 01_7 "touches the middle of a side" | inner corners that are midpoints of the turned tile's sides |
| `planted.squeeze_further` | overlap:0-4,1-4,2-4,3-4 | 01_10 "the corners bite in" | the box 1/100 narrower than the stop |
| `planted.middle_turned_more` | overlap:0-4,1-4,2-4,3-4 | — (chapter 04's subject) | a further exact turn, cos 399/401, sin 40/401 |
| `planted.middle_shifted` | overlap:1-4,3-4 | — | the middle tile 1/100 to the right |
| `planted.box_smaller` | escape:1,2,3 | — | the same tiles, the box 1/100 smaller |
| `planted.straight_squeezed` | overlap:0-4,1-4,2-4,3-4 | — (why the turn is needed) | the box of three 1/100 narrower, the middle tile straight |
| `source.gobel` | 1979 | 01_11 source line | S2, reference [7] |

The frames of the two motions are checked in f64 by `model`'s tests (2,001
frames each): the turn in the box of three never overlaps and touches only
when straight; the squeeze never overlaps from 3 down to the stop, touches
only at the stop, and overlaps all four corner tiles a hair past it. Between
sampled frames the argument is in the docs of `model::turning` and
`model::squeezed`.

## Sources

"Read" below means read through a web-fetch tool that returns a model's
summary of the page with quotations, on 2026-10-17; quotations are as that
tool returned them. The one exception is S2's Lemmas 1 and 2, whose statements and proof
sketches were also read as text from the saved HTML page (2026-10-17).

| id | Source | Used for | Seen |
|---|---|---|---|
| S1 | J. Levy, *Squares in squares*, case n = 5, https://jlevy.github.io/squares/cases/5.html, and its tutorial, https://jlevy.github.io/squares/tutorial.html | the side 2 + √2/2 = 2.70710678118654752440084436210485; "four squares sit in the corners and the fifth is rotated 45° in the middle"; "A 3×3 grid would need side 3, so the tilt buys 0.293"; lower bound credited to Göbel 1979 by "unavoidable points"; rigidity reported as "not infinitesimally rigid but second-order rigid at fixed side" (its T-012), the infinitesimal motions being the middle square's turn about its centre | read (summary). The page's own lower-bound status: "external proof (not read here)". Its rigidity results are not used in chapter 01 |
| S2 | Erich Friedman, "Packing Unit Squares in Squares: A Survey and New Results", *Electronic Journal of Combinatorics*, Dynamic Survey DS7 (version of 14 August 2009), https://doi.org/10.37236/28 | s(5) = 2 + 1/√2, Göbel; the reference "F. Göbel, Geometrical Packing and Covering Problems, in Packing and Covering in Combinatorics, A. Schrijver (ed.), Math Centrum Tracts 106 (1979) 179-199"; for chapter 05: "Theorem 2: The set P={ (1,1), (1,1+1/√2), (1+1/√2,1), (1+1/√2,1+1/√2) } is unavoidable in [0,2+1/√2]²", proved from Lemmas 1–3 (Lemma 1: "Any unit square inside the first quadrant whose center is in [0,1]² contains the point (1,1)"; Lemma 3: "If the center of a unit square u is contained in [triangle] ABC, and each side of the triangle has length no more than 1, then u contains A, B, or C") | read (summary of the HTML version; Lemmas 1 and 2 as text). Lemma 2: "Let 0<x≤1, 0<y≤1, and x+2y<2√2. Then any unit square inside the first quadrant whose center is contained in [1,1+x]x[0,y] contains either the point (1,y) or the point (1+x,y)." Chapter 05 uses x = 1/√2, y = 1, with the proof reconstructed in `model.rs` § the side strip. **Error in the sketch:** it says f(θ) = cos θ + sin θ − x sin θ cos θ is least at 45°; for x < 2√2 − 2 ≈ 0.83, the gap included, it is least straight: f(0) = 1 < f(45°) = 3√2/4 (`strip.least`, `strip.height.*`). The lemma stands because its y ≤ 1 covers the straight end. Lemma 3 is not yet reconstructed |
| S3 | F. Göbel 1979 (as cited in S2) | the packing's author and year (01_11) | **not seen**; the attribution rests on S1 and S2 |

**Competing explanations.** One web search (2026-10-17) for a video
explaining the five-tile packing found none. Text treatments it listed, not
read: Wikipedia "Square packing"; MathWorld "Square Packing"; xkcd 2740
"Square Packing" (the seventeen-square record) and its explainxkcd page. No
video was watched. A fuller competition review remains open.

**Never said or shown**: that the picture proves nothing smaller fits; that
the packing is rigid (chapter 04 will recompute it); that five is "the first
case where tilting beats the grid" (S1 says so; not yet checked against the
cases below five); anything about counts beyond five.
