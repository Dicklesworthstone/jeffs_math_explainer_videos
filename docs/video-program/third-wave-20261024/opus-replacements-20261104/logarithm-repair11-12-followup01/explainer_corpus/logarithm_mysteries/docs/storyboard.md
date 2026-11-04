# logarithm_mysteries — storyboard (rebuild candidate, repair round 2)

Bead `mx-video-program-program-87do.63`. Author RubyRaven (staging only), 2026-11-04.
This round answers root's cross-review (Mail 803). The round-1 storyboard is preserved
at `../../snapshots/handoff-mail785/explainer_corpus/logarithm_mysteries/docs/storyboard.md`
(relative to this crate).

Status: root has built and previewed earlier snapshots of chapters 11 and 12 (preview02,
scratch voice). The beats below for those chapters are this follow-up's: **not compiled,
not rendered, nothing heard.** Every timing below is a design proposal until root renders
it against the ElevenLabs Oliver takes.

Narration: ElevenLabs **Oliver** (owner requirement, 2026-11-04), synthesised serially by
root's guarded lane. The script is 364 sentences and 4,592 words, in 80 cues once root's
05_2 and 06_6 splits and the 11_7 and 12_6 splits are all in (75 before them; no word
changes). At an assumed
150 to 165 words a minute that is roughly 28 to 31 minutes of speech, plus thirteen 4 s
prediction holds and animation gaps. This is not a measurement.

## The film's spine

One question runs through every chapter: *which steady motion got us here?*

1. **Ratios want a ruler** (00–01). Hearing compares notes by ratio, so the keyboard is a
   ruler on which multiplying is a step. The rule pos(ab) = pos(a) + pos(b) forces where
   3 goes, and the answer, log₂ 3, is irrational.
2. **The machine** (02). On a slide rule, adding two lengths multiplies two numbers. One
   worked product is predicted and then revealed. The second runs off the end of the
   scale, is caught by the other index, and leaves the decimal point to the reader.
3. **Folding forgets** (03). One decade bends into a dial, and the dial forgets the
   decade. Lifting the dial into a spiral gives the decade back.
4. **Home ground** (04). eˣ, with slope equal to height, is one-to-one onto the positive
   numbers. ln is the inverse reading on (0, ∞) only. Doubling time is ln 2 divided by
   the rate.
5. **The complex exponential folds by itself** (05–08). i × i = −1 is predicted and
   seen as two quarter turns. A vertical line wraps a circle endlessly, so −1 has a whole
   column of logarithms. Carrying a logarithm round a loop gains 2πi. A loop off to the
   side does not; the obvious "keep the angle in (−π, π]" choice breaks at −1, and then
   the proof shows every choice must break. A cut buys the principal Log, whose two cut
   sides go to the strip's two edges (predicted). The product rule then keeps a lap
   count, with Log(z²) predicted first.
6. **Motions** (09–11). Read a matrix as a velocity field: its exponential is where the
   steady flow ends after one unit of time. 09_1 compares a matrix's two ends without
   drawing any path. The identity has infinitely many logarithms (ellipse flows); the
   principal one is chosen by a turning rate kept strictly inside (−π, π), earned on
   screen before the word eigenvalue appears. Some real matrices have no real logarithm.
   A negative determinant rules one out. diag(−2, −½) has none even with determinant +1:
   the film *shows* why, with two routes of a probe vector. Complex entries cure
   everything except singular matrices.
7. **What it forgets** (12). Four panels recap the film. Then the 3D take-away is taught
   (a marked rod, a followed corner), posed, and answered on screen.

## Colour roles (draw.rs)

| role | colour | used for |
|---|---|---|
| positions | blue | lengths along a ruler, exponents, logarithms, unit square |
| numbers | green | numbers being multiplied, outputs, transformed grids |
| answer | hero (warm yellow) | the asked-for value, the letter F, the prediction bar, "just above the cut" |
| laps | purple | decades on the dial, turns of the angle, 2πik, route one in 11, "just below the cut" |
| moving copy | teal | the sliding C scale, route two in 11 |
| excluded | red | zero, cuts, impossible targets, wrong guesses, the gap between routes |

## Prediction holds (draw::predict, THINK = 4.0 s each, music alone, picture still)

Every hold is the last beat of its cue: the question ends cue N and cue N+1 answers it.
That puts one take per cue, and a scratch check confirmed it for all thirteen.

| cue | question | answered in |
|---|---|---|
| 00_4 | the half-octave note (with the tempting wrong answer, 660 Hz) | 00_5 |
| 01_3 | where does 3 go | 01_4 |
| 02_2 | 2.5 × 3.2 on the slide rule | 02_3 |
| 04_6 | how long to double at 5 % | 04_7 |
| **05_1** | what is i × i? | 05_2 |
| 05_5 | which inputs give −1 | 05_6 |
| 06_2 | where the carried logarithm returns | 06_3 |
| **06_4** | does a loop off to the side bring its logarithm back? | 06_5 |
| **07_3** | where do the cut's two sides go on the strip? | 07_4 |
| **08_1** | what angle does Log report for z²? | 08_2 |
| 08_4 | the broken step in −1 = 1 | 08_5 |
| 11_3 | can one steady flow end on diag(−2, −½)? | 11_4–11_6 |
| **12_5** | how many steady spins give the half turn? (after three hints) | 12_6 |

Bold rows are new this round. Root decides from the real takes whether 4 s is right.

## Authored reading holds

A hold for reading is authored, with its own music window, and not left to whatever a cue
schedule happens to leave. Screen text is held for at least its length at 15 characters a
second, longer where something else on screen competes for the eye.

| after | what is read | how long |
|---|---|---|
| 11_20 | Culver's rule, four lines, 159 characters, then each block row's verdict in turn, 107 characters, with a gold mark at the row being read | 10.6 s + 7.13 s = 17.73 s of music after the voice |

The rule replaces the words in the column when its sentence starts, so nothing else in that
column competes with it. The next line waits for the window. Root re-scores the music and
may shorten the row walk after listening; the rule's own 10.6 s is the floor.

## 05–08: what each cue predicts or shows (repair 5)

Mail 803 asked that every cue in 05–08 carry a concrete prediction or a visible payoff,
with no arbitrary shortening. Nothing was cut. Four predictions (05_1, 06_4, 07_3, 08_1)
and four worked payoffs (05_2, 06_6, 07_2, 08_3) were added.

| cue | prediction or visible payoff |
|---|---|
| 05_1 | z, w and zw as arrows, with lengths multiplying and angles adding; ×i turns z into iz, "same length, a quarter turn". Then the point i itself. **Predict:** i × i = ? |
| 05_2 | two gold quarter arcs land on −1: "i × i = −1". The length ruler (green + teal segments add, so lengths multiply) and the angle ruler (plain +). e^{x+iy} = eˣ(cos y + i sin y), with x setting length and y setting angle. **Worked:** e^{ln 2 + iπ/2} = 2i, with a blue arrow to 2i (fact `complex.worked`) |
| 05_3 | the input grid mapped live: horizontals to rays, verticals to circles wound again and again |
| 05_4 | 0, ±2πi and 4πi in the strip all land on the single output 1 |
| 05_5 | **Predict:** which inputs land on −1? |
| 05_6 | the column πi, 3πi, −πi … two pi apart, every dot landing on −1 |
| 05_7 | every nonzero number's column, one real part; zero has none (eˣ is never 0) |
| 06_1 | the number walks the unit circle; its logarithm climbs the column, computed by `model::lift` |
| 06_2 | πi/2 at a quarter, πi at halfway. **Predict:** the value on return |
| 06_3 | 2πi: one rung higher; again; and back down |
| 06_4 | **new.** A side circle, "0 is not inside", walked a third of the way with a gold start ring. **Predict:** will it come back? |
| 06_5 | it returns: "the logarithm returns: change 0". Winding counts, and the rule 2πi per lap |
| 06_6 | **new payoff.** The candidate "keep the angle between −π and π" walks beside ours. A gap-in-laps gauge sits at 0, then at −1 a red ring, a drop arrow and "πi → −πi, gap 0 → −1" (fact `loop.principal_gap`). Then the proof's setup: theirs, the "differ by 2πik" TeX, "whole rungs only", "fixed" |
| 06_7 | both back to one: theirs returns, ours +2πi, so the gap changed; a contradiction |
| 07_1 | the cut; a walker blocked from going round; "no loop can go around 0 now" |
| 07_2 | (−∞, 0] removed; Log z = ln\|z\| + i Arg z; d/dz Log = 1/z. **Worked:** gold arc to i and purple arc to −i, "Log i = πi/2", "Log(−i) = −πi/2" (fact `branch.log_i_pi`) |
| 07_3 | the cut plane unrolls into the strip: circles to verticals, rays to horizontals. Then it rolls back, and the cut's two sides are marked, gold just above and purple just below (hidden until now, so the first unroll gives nothing away). **Predict:** where do they go? |
| 07_4 | **new.** It unrolls again: gold runs along the top edge and purple along the bottom; "angle just above the cut: near π; just below: near −π"; a dashed red 2π jump |
| 07_5 | other cuts: the upward axis with window (−3π/2, π/2), a spiral; "any curve from 0 out to infinity that never crosses itself"; the branch point |
| 07_6 | the convention on the cut (Arg = π) and its jump; exp(Log z) = z for every z ≠ 0, but Log(e^w) = w only on the strip with its top edge |
| 08_1 | the real rule; z at 3π/4 squared, angles adding to 3π/2; the angle line with the principal window. **Predict:** Log(z²) = ? i |
| 08_2 | "outside"; the gold dot at −π/2, "same direction as −½π"; Log z + Log z = 3π/2 i against Log(z²) = −π/2 i; a purple arrow of one lap |
| 08_3 | the slide rule's dropped decade beside Log's dropped lap; Log(zw) = Log z + Log w + 2πik; precisely: k ∈ {−1, 0, 1}. **New:** "for z²: k = −1, one lap down", computed from the facts |
| 08_4 | the four-step −1 = 1 derivation. **Predict:** which step broke? |
| 08_5 | the combining step boxed; √z = e^{½ Log z}; two half turns against a zero angle; halving the lost lap leaves a factor of −1 |

## Chapters 09–12 (repairs 1–4)

**09_1 (repair 4).** No path is drawn. A faint ghost of the plane before stays put, each
example's plane after is laid over it, and dashed chords pair marked points with where
they land: a comparison of two ends. The script now says the matrix gives only where
each point ends up, "the travel comes next".

**10_3–10_4 (repair 2).** The round-1 sentence about "possibly complex stretch
directions" is gone.

- 10_3: for numbers the principal angle stays within a half turn. Matrix flows turn too:
  at a steady rate, sweeping round stretched circles, perhaps spiralling, or not at all.
  A turning line shows the spiral flow (0.872, inside), a stretch flow and the zero flow
  (0), and the ellipse flows (±2π, outside).
- 10_4: the same rule picks exactly one logarithm when A sends no direction to zero or
  to a negative multiple. That is the principal logarithm, real for real A, and 0 for
  the identity. The textbook line (eigenvalues) comes last.

**11_1 (repair 4).** The mirror is compared before and after: a dot on each of four
corners and on its image, joined by dashes that stay in the open gap between the two Fs.
That pairing is a comparison of two ends, and nothing travels along it. Then the plain
average ½(I + mirror) appears in red, with a red dot at each dash's middle. The dashes
leave, and the collapse is drawn: the whole grid moves along the straight line of
matrices from I to the singular midpoint ½(I + mirror), and stops there. It is a path to
that midpoint and never to the reflection: "det 0 … squashes the plane flat onto a line"
(fact `matrix.mirror_average_det`).

**11_4–11_20 (repair 1).**

- 11_4: "suppose A = e^X, X real". The probe u = (1, 1) has two routes, first X then
  e^X (purple) and first e^X then X (teal), shown with the sample flow; the column's labels
  are typeset, and a gold dot on the plane carries its own words, "same end point". Then
  the series and `X X^k = X^{k+1} = X^k X` ⇒ XA = AX.
- 11_5: the probe stays alone on the plane, still named, until the words send it along
  both routes. Then the same two routes for A = diag(−2, −½) and a candidate X with
  off-diagonal entries. They end apart; the red gap bar waits for those words, with a live
  readout: "each gap part = ±1.5 × an off-diagonal entry of X × a part of u, where 1.5 =
  (−½) − (−2)". The off-diagonal entries tween to 0, and the ends meet only when b = c = 0.
- 11_6: X is diagonal, so its flow stretches each axis by e^{tp} > 0 and never reverses
  one. The axes are white, as in 11_7, so the gold F never lies on them. A's target F,
  outlined in red, needs both reversed. The stretch flow goes home first, so nothing cuts;
  two flows run in turn on "running one after the other" and get there; no single one does.
- 11_7: −I by the spin, with the plane back at rest at once. Equal stretches mean
  (−1) − (−1) = 0, so there is no gap and every X commutes with −I. Then the 2×2 rule.
- 11_20 (11_7's words from "In higher dimensions,"): the rows, each row a matrix from the
  facts. Culver's exact condition takes the column in four gold lines, with the citation as
  the precisely line, and gets its authored hold.
- 11_8: the title changes, and "A = diag(−2, −½): no real logarithm" stands under it while
  the voice says what disappears.

**12_1–12_3 (follow-up).** Panel A's ruler numbers only the doublings 1, 2, 4, 8, with a
tick at every whole number, and the keyboard fades in under it on "the keyboard and the
slide rule". 12_2 folds that idea for real: a ruler flat across panel B bends shut, its ends
meeting at the bottom, and turns its 1 to the top, so the 10 lands on the 1. Only then do
the dial's numbers, pointer and caption come. Panel C stands a little higher with shorter
axes, its labels are white, and −1 stays on top of the cut. Panel D's dots fade in.

**12_4–12_21 (repair 3, follow-up).**

- 12_4: an opaque cube (only faces toward the viewer are drawn) with a gold rod through
  the centres of its top and bottom faces; parts hidden by the solid are not drawn
  (`hidden`, a slab test). Corner circles about the rod preview the motion. One corner
  is followed with "time" and "laps so far" readouts to the half turn.
- 12_5: the question and the hints: laps, direction, axis. Then the hold.
- 12_6, 12_20 and 12_21: the answer on the cube, one hint at a time, 12_6's old words split
  before "Direction next:" and before "The two slowest,". Each new spin starts under a short
  dip of the picture, never a jump in plain view.
- 12_6: 1½ and 2½ laps land the same.
- 12_20: ½ lap the other way lands the same.
  - A spin about another axis, (0, 1, 1), drags a pin copied from the rod's upper stub off
    the rod (red line, pin, arc). The top face's centre itself stays inside the solid.
    The line and the arcs keep clear of the two rings, and the corner lands one edge from
    the end ring. For about t 0.48 to 0.84 the pin points nearly at the viewer, and its tip
    and arc carry the drag.
  - "so: infinitely many spins, all about the rod".
- 12_21: the tie and the family.
  - A turning line shows ±π and ±3π with "…" past each end, with ±π tying on the green
    window's open edge.
  - Precisely: "eigenvalue −1, so no principal log (Higham §4 Fact 1)".
  - On "a convention broke the same tie": "for the number −1, Log(−1) = iπ by convention".
  - On "the honest answer is the whole family": gold dots on all four spins and "here: the
    whole family".

The take-away is now answered on screen, not left for the description.

## Drawing aids and simplifications (each stated on screen)

- 05_3: the wound circle's trace spreads outward a little each lap (LAP_SPREAD 0.045) so
  the laps stay visible. It is labelled as a drawing aid.
- 09_1: an endpoint comparison, never a path.
- 11_1: the dashes compare two ends. The collapse that follows is a path, drawn only as
  far as the singular midpoint ½(I + mirror), and never on to the reflection.
- 07_3–07_4: the cut's marked sides are drawn 0.07 rad from the cut (MARKED) so the two
  stay apart on screen.
- 10 and 11: the grids are clipped to a box (`draw::clipped_grid`). The flows are exact,
  and only the far corners are hidden.

## Precisely lines (lower thirds) and their sources

Quotes and the reading level are in `docs/sources.md`.

| where | statement | source |
|---|---|---|
| 00 | equal temperament, A4 = 440 Hz; real pianos tuned slightly stretched | ISO 16 (carried from round 1) |
| 02 | about three significant figures; the user places the decimal point | slide-rule practice |
| 04 | 100 × ln 2 = 69.3; seventy is the round number near it (rule of 70) | facts real.rule_of_seventy |
| 07 | Log holomorphic off (−∞, 0], −π < Im < π; with Arg(−x) = π, Im Log fills (−π, π] | DLMF §4.2 (carried from round 1) |
| 08 | for nonzero z, w on the principal branch, k is −1, 0 or 1 | derived: Im Log ∈ (−π, π] |
| 10 | the principal matrix logarithm | Higham, MIMS EPrint 2005.21, §4 Fact 1; Gallier arXiv 0805.0245 Thm 3.11(a) |
| 11 | det = e^{∫ tr} for any real linear flow | Liouville's formula |
| 11_20 | the real-log condition, attributed to Culver "as stated by" Higham and Gallier | Higham §4 Fact 3; Gallier Thm 3.4 (Culver unread) |
| 11_8 | every invertible complex matrix has a logarithm | Gallier Thm 3.3 (Jordan form) |
| 12_21 | eigenvalue −1, so no principal log | Higham §4 Fact 1's definition, plus the step in sources.md |

## Known screen risks for root's render review

- TeX at this pin. Matrices are drawn as `matrix_card` primitives, never TeX
  environments. `\operatorname`, `\tfrac`, `\text{...}`, `\Rightarrow`, `\neq`, `\quad`,
  `\qquad` and `\;` follow the corpus's usage; `\phantom`, `\cdots` and `smallmatrix` are
  avoided.
- Text widths in chapters 11 and 12 use CMR10 advances in their tests (em × size × 0.01),
  within about 2% of lint-measured lines. TeX runs about 6% wider. A frame review shows the
  truth.
- 12's cube: the visibility (`hidden`) and face culling are tested numerically. Preview02
  showed the earlier axis; the new axis (0, 1, 1), its dips and the fold have not been seen.
- `always_redraw` is used throughout: the routes in 11, the cube and its traces in 12,
  the gap gauge in 06, and the cut-plane grid in 07. Render cost is unmeasured.
