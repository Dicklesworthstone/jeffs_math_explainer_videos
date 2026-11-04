# logarithm_mysteries — storyboard (rebuild candidate)

Bead `mx-video-program-program-87do.63`. Author RubyRaven (staging only), 2026-11-04.
Status: **written, not compiled, not rendered, no narration synthesised, nothing heard.**
Every timing below is a design proposal until root renders it against the ElevenLabs
Oliver takes.

Narration: ElevenLabs **Oliver** (owner requirement, 2026-11-04), synthesised serially by
root's guarded lane. The script is 68 cues, 317 sentences, about 3,800 words. At an
assumed 150 to 165 words a minute that is roughly 23 to 26 minutes of speech before
holds and animation gaps. This is not a measurement.

## The film's spine

One question runs through every chapter: *which steady motion got us here?*

1. **Ratios want a ruler** (00–01). Hearing compares notes by ratio, so the keyboard is a
   ruler on which multiplying is a step. The rule pos(ab) = pos(a) + pos(b) forces where
   3 goes, and the answer, log₂ 3, is irrational.
2. **The machine** (02). On a slide rule, adding two lengths multiplies two numbers. One
   worked product is predicted and then revealed. The second product runs off the end of
   the scale and is caught by the other index, so the reader must supply the decimal
   point.
3. **Folding forgets** (03). One decade bends into a dial, and the dial forgets the
   decade. Lifting the dial into a spiral gives the decade back.
4. **Home ground** (04). eˣ, with slope equal to height, is one-to-one onto the positive
   numbers. ln is the inverse reading on (0, ∞) only. Doubling time is ln 2 divided by
   the rate.
5. **The complex exponential folds by itself** (05–08). A vertical line wraps a circle
   endlessly, so −1 has a whole column of logarithms. Carrying a logarithm round a loop
   gains 2πi, which proves no continuous logarithm exists on a loop around zero. A cut
   buys the principal Log, and the product rule then keeps a lap count.
6. **Motions** (09–11). Read a matrix as a velocity field: its exponential is where the
   steady flow ends after one unit of time. The identity has infinitely many logarithms
   (ellipse flows) and the principal one is pinned by eigenvalues. Some real matrices
   have no real logarithm: a negative determinant rules one out, and diag(−2, −½) has
   none even with determinant +1. Complex entries cure everything except singular
   matrices.
7. **What it forgets** (12). The four panels recap the film, then the viewer gets a
   take-away puzzle.

## Colour roles (draw.rs)

| role | colour | used for |
|---|---|---|
| positions | blue | lengths along a ruler, exponents, logarithms, unit square |
| numbers | green | numbers being multiplied, outputs, transformed grids |
| answer | hero (warm yellow) | the asked-for value, the letter F, prediction bar |
| laps | purple | decades on the dial, turns of the angle, 2πik |
| moving copy | teal | the sliding C scale, second axis arrow |
| excluded | red | zero, cuts, impossible targets, wrong guesses |

## Prediction holds (draw::predict, THINK = 4.0 s each, music alone, picture still)

| cue | question | hold placed after sentence |
|---|---|---|
| 00_4 | the half-octave note | the question |
| 01_3 | where does 3 go | the question |
| 02_2 | 2.5 × 3.2 on the slide rule | the question |
| 04_6 | how long to double at 5 % | the question |
| 05_5 | which inputs give −1 | the question |
| 06_2 | where the carried logarithm returns | the question |
| 08_4 | the broken step in −1 = 1 | the question |
| 11_3 | can one steady flow end on diag(−2, −½) | sentence 8 |
| 12_4 | how many steady spins give the half turn | the final hint, before finish |

Root decides from the real takes whether 4 s is right. The 12_4 hold is the film's last
picture; a longer silent tail may suit it better.

## Chapter beats

Each chapter's file opens with its beat list. Here are the chapters the earlier rounds
did not cover.

**11_unreachable.** The plane panel is on the left (x −5.8 … −1.4), a signed-area gauge
at x 0.9, and the text column from x 2.2.

- 11_1: the mirror. A straight slide from I to the mirror squashes the F flat half way.
  This is labelled as a straight slide, since every continuous path from I to the mirror
  passes through det = 0.
- 11_2: the unit square under the steady flow X = matrix.flow, with a live det readout
  and needle. Forwards to t = 1 (1.221) and t = 3 (1.822), backwards to t = −2 (0.670):
  never zero. The red zero line, the green positive region, and the mirror's −1 marker
  out of reach. Precisely line: Liouville, det = e^{∫ tr}, steady or not.
- 11_3: "det < 0: no real logarithm". Then "det > 0: enough?", the card
  A = diag(−2, −½), the axis arrows ×2 flipped and ×½ flipped, det = +1, and
  A = (−I) × diag(2, ½). Prediction.
- 11_4: "not enough". The series A = e^X = I + X + … gives XA = AX; distinct stretches
  force X to keep each axis. A diagonal flow then only stretches (shown both ways). Two
  flows in turn (a half-turn spin, then a stretch) reach A, but no single steady flow
  does.
- 11_5: −I is reached by the steady half-turn spin, e^{πJ} = −I. The 2×2 rule, then
  Culver's statement in full, with the citation as the precisely line.
- 11_6: complex entries. e^{iπ} = −1; log A = diag(ln 2 + iπ, −ln 2 + iπ) is checked by
  exponentiating; Higham Thm 1.27; singular matrices have none, since det e^X = e^{tr X}
  ≠ 0.

**12_what_it_forgets.** Four framed panels.

- A (top left): reals. The eˣ curve with one probe hit, and the ratio ruler's equal ×2
  hops.
- B (top right): the dial, with 0.24, 2.4, 24 and 240 on one mark.
- C (bottom left): the plane and the column ±πi, ±3πi converging on −1, then the red cut
  and Log zw = Log z + Log w + 2πik.
- D (bottom right): the circle and both ellipse flows, dots home together at t = 1,
  beside "diag(−2, −½): no single steady real flow ends here".

At 12_3 sentence 2 a gold "?" lands in each panel. At sentence 3 the panels clear for
"Which steady motion got us here?". 12_4 shows a cube seen obliquely, with a gold fixed
axis and a faint ghost of its start, turning half a lap. Then the question and three
hints (axis, direction, laps) in spoken order, and the last hold.

### The take-away's answer (for the description or pinned comment, not shown)

The steady spins about **the same axis** reach it, in **either direction**, by any
**odd number of half turns**: π, 3π, 5π … radians per unit time. That makes infinitely
many real logarithms in so(3).

The two slowest, a half turn each way, tie. Equivalently, the rotation has eigenvalue −1
on the matrix "cut", so no principal logarithm exists.

No other axis works, because a nontrivial rotation's axis is its fixed line. The tests in
`what_it_forgets.rs` check all three claims numerically.

## Drawing aids and simplifications (each one stated on screen)

- 05_3: the wound circle's trace spreads outward a little each lap (LAP_SPREAD 0.045) so
  the laps stay visible. It is labelled as a drawing aid.
- 09_1: the example matrices slide straight from before to after, labelled "not a flow".
  Chapter 11 pays this off.
- 11_1: the straight slide to the mirror, labelled as a straight slide.
- 10 and 11: the grids are clipped to a box (`draw::clipped_grid`). The flows are exact,
  and only the far corners are hidden.

## Precisely lines (lower thirds) and their sources

| where | statement | source |
|---|---|---|
| 00 | equal temperament, A4 = 440 Hz; real pianos tuned slightly stretched | ISO 16 |
| 02 | about three significant figures; the user places the decimal point | slide-rule practice |
| 04 | 100 × ln 2 = 69.3; seventy is the round number near it (rule of 70) | facts real.rule_of_seventy |
| 07 | Log holomorphic off (−∞, 0], −π < Im < π; with Arg(−x) = π, Im Log fills (−π, π] | DLMF §4.2 |
| 08 | for nonzero z, w, k is −1, 0 or 1 | Im Log z + Im Log w ∈ (−2π, 2π] |
| 10 | principal matrix logarithm | Higham, Functions of Matrices (2008), Thm 1.31 |
| 11 | det = e^{∫ tr} for any real linear flow | Liouville's formula |
| 11 | real logarithm exists ⇔ the Jordan blocks of each negative eigenvalue come in even counts | W. J. Culver, Proc. AMS 17 (1966) 1146–1151, doi 10.1090/S0002-9939-1966-0202740-6 |
| 11 | every invertible complex matrix has a logarithm | Higham (2008), Thm 1.27 |

How far each source has been checked:

- **Culver.** Volume, pages and the theorem's wording were matched against secondary
  statements (arXiv 2512.23735 and the matrix-logarithm encyclopaedia entry). The
  paper's own text has not been read.
- **Higham theorem numbers.** These were carried from the earlier review round and have
  not been re-read here.

## Known screen risks for root's render review

- TeX at this pin. Matrices are drawn as `matrix_card` primitives, never TeX
  environments. `\operatorname`, `\tfrac`, `\text{...}`, `\Rightarrow`, `\neq`, `\quad`
  and `\qquad` follow the corpus's usage; `\phantom`, `\cdots` and `smallmatrix` are
  avoided.
- Text widths are estimated (0.55 em per character, em = size/144 scene units) in tests,
  not measured. The glyph probe and a frame review will show the truth.
- `matrix_card` geometry was corrected in this round: it had been about three times too
  large (brackets at ±2.4 units for a size-32 card). This affects the cards in 09 and 10
  too, and none of them have been seen.
