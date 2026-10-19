# Storyboard — Epsilon and Delta: Why Getting Closer Wasn't Enough

The hand-edited source of this video's words and pictures (first chapter
written 2026-10-19 for task mx-video-program-program-87do.58.1, revision
`v1-michael-20261019`). `python3 -I tools/storyboard.py epsilon_delta`
checks it (gate G2) and writes `script.tsv` from the words column. Numbers:
`src/facts.rs` and `oracle.py` (every number below names its fact key);
brief and sources: `docs/video-program/briefs/epsilon-delta.md` and this
crate's `README.md`.

Words are spoken exactly as written: no colons, semicolons, dashes or
parentheses in them (voice rules, plan §5.4). `*(…)*` is an annotation, never
spoken. `⟨predict⟩` opens a prediction question; the beat ends with the
question and the picture holds before the next beat answers it.

## 2 The arc

The owner asked for the difficulty of epsilon and delta, the foundations of
calculus, a counterintuitive failure, the two notes that make delta
systematic, and the topological view. The spine is one question, asked of
every picture: **who chooses each quantity, and before what?** Getting
closer is not a guarantee until that order is fixed.

| Act | Chapters (brief §"Episode shape") | Spine |
|---|---|---|
| I. A limit that jumps | 01 a limit that jumps (built); 02 the shrinking region | unbroken curves x^n on [0, 1], a limit that is 0 then 1; a fixed point waits a finite time, the moving point x_n = 2^(−1/n) keeps a gap of 1/2; a thin region is not an empty one |
| II. The guarantee | 03 what the old objection was about, 04 the challenge game, 05 why written proofs hide the search | the tolerance band chosen first, then the input band whose whole image fits; δ may depend on ε and the point, never on the x the challenger picks next |
| III. Making δ | 06 Sprague's recentring, 07 Davis's bound, 08 fixed or moving, precisely | δ = √(4 + ε) − 2 for x² at 2, the simpler δ = min(1, ε/5), the cubic's C = 22; the naive ε/4 fails at ε = 1 with h = 6/25; pointwise against uniform as an order of choices |
| IV. Repair and other languages | 09 the three-piece bridge, 10 sequences and bounds, 11 preimages, 12 what the definition does not supply | uniform convergence repairs the opening; one bad sequence refutes continuity; open sets pull back to open sets; completeness is a separate assumption |

One picture runs through act I: the plot of [0, 1] with the curves, the gold
limit, the teal fixed point and the lavender moving point. Act II's picture
is the tolerance band and the input band over the same kind of plot, so the
moving point returns as the challenger's choice. Historical names and dates
stay out of the narration until their page-level attribution is qualified
(brief); chapter 03 is written only after that check.

### The exact continuation after chapter 01

Chapter 01 ends on the plot at rest: the curves dim, the gold limit with
its hollow mark and its point, the lavender half line with the moving point
at x_50 and its red gap, the teal fixed point sunk below the half line, and
the two cards "fix x first" and "let x move with n". Chapter 02 opens on
that picture:

- **02_the_shrinking_region** (not yet built): the region near x = 1 where
  the n-th curve is still above 1/2 is (x_n, 1], and its width 1 − x_n
  shrinks like ln 2/n without ever being empty; a pixel-width warning (a
  region thinner than a pixel is still a region); the same ε = 1/2 named as
  a tolerance for the first time, still without symbols; it ends on the
  question of what "as close as we want" must promise. Numbers come from
  `model.rs` (`half_height`, `first_within`); no new source is needed.

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses a
data colour.

| Idea | Color |
|---|---|
| the curve being looked at, x^n for the current n | blue `#58C4DD` |
| the curves already drawn, kept behind it | dim blue `#2C5566` |
| the limit f (flat at zero, a single point at height one) and its formula | gold `#F0AC5F` |
| the fixed point x = 0.9, its heights and its table | teal `#5CD0B3` |
| the moving point x_n, the half line it sits on, its table and formula | lavender `#C59BF2` |
| the gap that survives at the moving point, and the jump | red `#FC6255` |
| axes and their numbers | `dim` |
| the prediction ring, the title, the trace of an unbroken curve | `hero` gold `#FFD866` |

Colour is never the only cue: the fixed point is a dashed vertical line with
its value under the axis, the moving point sits on a dashed horizontal line
labelled 1/2, and every table has its header.

## Vocabulary (one word per idea)

**curve** (the graph of x^n; never "function" in the narration of chapter
01); **unbroken** (the narration's word for continuous in chapter 01; the
screen's lower-third says "continuous" once, with its exact meaning on
[0, 1]); **height** (the value x^n); **the limit** (f, the shape the curves
approach point by point); **jump** (the discontinuity at 1); **gap** (the
distance between a curve's height and the limit's at the same x); **fixed
point** and **moving point**, never "the point" without one of them once
both exist; **half height** (y = 1/2). Epsilon, delta, pointwise and uniform
are not said until chapters 02 and 08 name what these pictures show.

Numbers: decimals are said digit by digit after "point" as the screen
writes them ("point seven two nine" for 0.729); one half is said "one half"
and shown 1/2; zero and one are the interval's ends and the limit's two
values.

## Misconceptions this chapter defuses

| Wrong belief at this point | Beat that defuses it |
|---|---|
| "a limit of unbroken curves is unbroken" | 01_5 to 01_6: the prediction, then the jump |
| "the plot shows the limit, so it proves it" | 01_7: eight curves at finite resolution prove nothing; x^(n+1) = x · x^n with x < 1 does |
| "the curves converge, so near 1 they are eventually close to the limit everywhere" | 01_9 to 01_10: the moving point keeps a gap of exactly 1/2 for every n |
| "the failure means some point never converges" | 01_11: every fixed point gets within 1/2 (0.9 from n = 7, 0.99 from 69, 0.999 from 693); only the order of choices differs |
| "this is a failure of uniform continuity" | the narration never says so; every x^n is uniformly continuous on [0, 1]; chapter 08 names uniform convergence |

## Narration units and timing

Michael, with his approved English accent, reads each unit as one coherent
take, cut back into its lines (`narration_units.json`, checked against this
script's bytes). The units are complete thoughts of 48 to 71 words: the
curves through the question (01u1: 01_1 to 01_5, 71 words), the answer and
why arithmetic proves it (01u2: 01_6, 01_7, 48), and the moving point
through the contrast (01u3: 01_9 to 01_11, 68). 01u1 ends on its question, so the question
is read as a question at a unit's end; its following cue, 01_6, is the
context to read it with if the synthesis supplies following context. The
title card (01_8) is silent.

Semantic gaps, authored in the scene rather than left to a uniform breath:
the question 01_5 is followed by a 4 s think window with a draining ring,
and the answer stays hidden until it ends; the reveal 01_6 follows the
window's breath; 01_10's last words ("the gap stays one half") get a 0.6 s
beat before the conclusion 01_11; the chapter ends on a hold. Everything
else uses the kit's thought-classed line gaps.

**Hold (2026-10-19):** the owner rejected the first wave's narrated cuts
and withdrew them; the rejected narration method must not be carried into
new synthesis. This grouping and timing are a proposal for the root's next
qualified method, not an instruction to synthesize. No take exists.

## 01_a_limit_that_jumps: A limit that jumps (~82 s at the kit's scratch rate)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Take the line y equals x, from zero to one. Square every height, and it sags. Higher powers sag further. *(C1.domain.left, C1.domain.right, C1.squared)* | Cold open. Axes on [0, 1] by [0, 1] fade in; the line y = x is drawn in blue from left to right, its label at the plot's top left. At "Square every height", a dim copy stays where it was and the line morphs into y = x², every point moving straight down to its square. At "Higher powers", x³, x⁴, x⁵, x¹⁰, x²⁰ and x⁵⁰ in turn, each leaving its predecessor dim: eight curves in all (C1.powers) | morph that keeps identity; lagged curve family | |
| 01_2 | Each curve is unbroken, from zero up to one. | Ink dots mark (0, 0) and (1, 1), where every curve starts and ends; a bright stroke traces x⁵⁰ from end to end without lifting | highlight (a trace) | |
| 01_3 | Pin one point, x equals nine tenths. Its heights go point nine, point eight one, point seven two nine, down toward zero. *(C1.fixed.x, C1.fixed.n1 to n3, C1.limit.below_one)* | A teal dashed line at x = 0.9, "0.9" under the axis. Right of the plot a table, headers n and 0.9ⁿ: rows 1 → 0.9, 2 → 0.81 and 3 → 0.729 arrive with their words, each with a teal dot on its curve; at "down toward zero" the rest arrive together: 4 → 0.6561, 5 → 0.59049, 10 → ≈ 0.3487, 20 → ≈ 0.1216, 50 → ≈ 0.0052 (above n = 5 rounded to four places, marked ≈; C1.fixed.n*.4dp) | table built from primitives; lagged reveal | |
| 01_4 | At x equals one, nothing moves. One to any power is one. *(C1.domain.right, C1.endpoint.height)* | A gold ring glows around (1, 1), where every curve ends; at "One to any power", 1ⁿ = 1 beside it | highlight | |
| 01_5 | ⟨predict⟩ So what shape are the curves heading for? | Right: a large "?" inside the think ring, which drains for 4 s with "pause for more time" under it; the answer stays hidden until it ends | predict pause | think 4 s, the answer hidden until it ends |
| 01_6 | Flat at zero all the way to one, then a single point at height one. Unbroken curves, and a limit that jumps. *(C1.limit.below_one, C1.domain.right, C1.limit.at_one)* | The "?" and the end dots go; every curve dims. In gold the limit: a flat stretch along y = 0 drawn from 0 to a hollow mark at (1, 0), with f(x) = 0 for 0 ≤ x < 1 at the right. At "then a single point", a filled gold point at (1, 1) and f(1) = 1 under the first formula. At "a limit that jumps", a red dashed line just right of x = 1 spans the drop, labelled jump, and under the formulas, dim: "at x = 1, the right end of [0, 1], x arrives only from the left" | equation; highlight | |
| 01_7 | A picture of a few curves can't prove that. Arithmetic can. Below one, each new power multiplies the height by the same number less than one. *(C1.curves, C1.max_power, C1.domain.right)* | Under the plot: "8 curves drawn, up to n = 50" (dim). At "Arithmetic can": right, x^(n+1) = x · xⁿ over 0 ≤ x < 1, and under the table "each row: × 0.9" | formula | |
| 01_8 | *(title)* | Everything crossfades into the title card: Epsilon and Delta, and under it Why Getting Closer Wasn't Enough | title card | listen 2 s (the motif: left open) |
| 01_9 | Now let the point move. On each curve, mark where it crosses half height. *(C1.half)* | The plot returns: axes, the gold limit, every curve dim, y = x bright. At "half height", a lavender dashed line at y = 1/2 with 1/2 by the axis, and a lavender dot where y = x crosses it. Right, a table with headers n and "crossing at x =", first row 1 → 0.5000 (C1.witness.n1.4dp) | table built from primitives | |
| 01_10 | There the curve is at one half and the limit is zero. As the power grows, the crossing creeps toward one, but the gap stays one half. *(C1.half, C1.limit.below_one, C1.domain.right, C1.witness.gap)* | A red bar drops from the dot to the gold limit; under the table "gap at the crossing: 1/2". At "As the power grows", the bright curve morphs through x² to x⁵⁰ while the dot and its bar ride right along the half line, each crossing joining the table: 0.7071, 0.7937, 0.8409, 0.8706, 0.9330, 0.9659, 0.9862 (C1.witness.n*.4dp); the bar keeps its height. Then x = 2^(−1/n) under the table | morph that keeps identity; live recompute; table | a 0.6 s beat before the answer |
| 01_11 | So it matters who picks the point. Fix it first, and the gap shrinks to zero. Let it move with the curve, and the gap never does. *(C1.limit.below_one; C1.wait.tolerance, C1.wait.x0.9, C1.wait.x0.99, C1.wait.x0.999 on screen)* | The teal dashed line returns at 0.9, its dot on x⁵⁰ sunk below the half line. Right, a card "fix x first", "gap below 1/2 from", and rows 0.9: n = 7, 0.99: n = 69, 0.999: n = 693 (the first n whose gap is below 1/2). At "Let it move with the curve", a second card "let x move with n", x = 2^(−1/n), "gap = 1/2, for every n" in red. Lower-third, precisely: every curve is continuous on [0, 1], and at each fixed x the gap to f shrinks to 0; but the n-th curve crosses 1/2 at x = 2^(−1/n) below 1, where f is 0, a gap of exactly 1/2 for every n | lower-third; table | hold |
