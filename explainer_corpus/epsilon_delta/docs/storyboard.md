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

- **02_the_shrinking_region**: built (its beats are below, task
  mx-video-program-program-87do.58.2). It rebuilds chapter 01's resting
  picture with chapter 01's own builders, then clears the panel for a zoom
  window. Two strips are kept apart by their ends. The curve is above 1/2 on
  x_n < x ≤ 1, which holds x = 1. The gap to the limit is at least 1/2 on
  x_n ≤ x < 1, which holds the moving point and not x = 1, where the gap is
  zero. The power grows to 1000, below one pixel of the final picture from
  n = 898. The zoom window is as wide as the plot, so its ×10, ×100 and
  ×1000 are true sideways stretches; its heights are drawn at 1/3, and the
  label says so. The width 1 − x_n shrinks like ln 2/n, stated on screen as a
  limit with the width positive (its bounds are tested, not shown). It
  ends on the need for a guarantee for every x at once, and an order of
  choices.

Root's silent scratch drafts (draft04, Mail 3871) run 93.17 s for 01 and
87.27 s for 02, longer than the ~85 s chapter target and than the estimates
in the chapter headings below. Root kept the prediction time and sped
nothing up. Pacing is settled only with qualified narration, which is still
on hold. The 02_4 sub-pixel marker became an arrow after review 3881; root's
draft05 (central15, Mail 3910) keeps 02 at 87.27 s, with cues identical to
draft04.

After chapter 02 the picture rests on the plot at n = 1000 with its
under-axis strip, the window at ×1000 and the two summary lines. Chapter 03
in the arc table needs the brief's page-level attribution check before it
is written; if that check is not ready, the challenge game (04) can come
next, opening on this picture.

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
| the strip where the curve is above 1/2 (chapter 02) | lavender, as the half line |
| the strip where the gap is at least 1/2 (chapter 02) | red, as the gap |
| an end mark: filled when the end belongs to the strip, hollow when it does not | the strip's colour |
| the zoom window's border and the magnifier on the plot | `dim`, connectors `faint` |
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

Chapter 02 adds **strip** (a stretch of the x axis where something holds;
the narration's strip is the gap strip from 02_3 on), **stretch** (the
zoom, always "sideways" and always with its true factor on screen, beside
the share its heights are drawn at) and **width**
(of a strip, 1 − x_n). It never says the strip "disappears": it says
narrower, thinner than a pixel, never empty.

Numbers: decimals are said digit by digit after "point" as the screen
writes them ("point seven two nine" for 0.729); one half is said "one half"
and shown 1/2; zero and one are the interval's ends and the limit's two
values.

## Misconceptions these chapters defuse

| Wrong belief at this point | Beat that defuses it |
|---|---|
| "a limit of unbroken curves is unbroken" | 01_5 to 01_6: the prediction, then the jump |
| "the plot shows the limit, so it proves it" | 01_7: eight curves at finite resolution prove nothing; x^(n+1) = x · x^n with x < 1 does |
| "the curves converge, so near 1 they are eventually close to the limit everywhere" | 01_9 to 01_10: the moving point keeps a gap of exactly 1/2 for every n |
| "the failure means some point never converges" | 01_11: every fixed point gets within 1/2 (0.9 from n = 7, 0.99 from 69, 0.999 from 693); only the order of choices differs |
| "this is a failure of uniform continuity" | the narration never says so; every x^n is uniformly continuous on [0, 1]; chapter 08 names uniform convergence |
| "where the curve is above 1/2, it is far from its limit" | 02_1 to 02_3: at x = 1 the curve is above 1/2 and the gap is zero; the two strips differ exactly at their ends |
| "a strip thinner than a pixel is gone" | 02_5 to 02_6: stretched ×1000 it fills about 69% of the window |
| "the width is ln 2/n" | 02_7: it shrinks like ln 2/n; the card calls n·w(n) → ln 2 a limit, not a finite equality, and w(n) > 0 |

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

Chapter 02's units:
- **02u1** (02_1 to 02_3, 68 words): the two strips and their ends.
- **02u2** (02_4, 02_5, 18 words): the narrowing, ending on its question.
- **02u3** (02_6 to 02_9, 99 words): the answer, the rate, the moving
  point, and the bridge to a guarantee.

The question 02_5 is followed by the same 4 s think window, its answer
("No.") hidden until the window ends. The stretches in 02_6 land on "a
hundred times" and "a thousand times". The chapter ends on a hold.

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

## 02_the_shrinking_region: The shrinking region (~80 s at the kit's scratch rate)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Shade every x where the curve is still above one half. That's a strip from the crossing up to one, and x equals one belongs to it. *(C1.half, C1.domain.right)* | Chapter 01's resting picture, rebuilt by its own builders. The table, notes, fixed point and cards fade. A magnifier box on the plot around the last tenth, x from 0.9 to 1, at its true width, with two faint lines to a zoom window at the right, as wide as the plot and a third as tall: "n = 50, stretched ×10 sideways; heights ×1/3" (C2.zoom.0, C2.lens.heights), ends 0.9 and 1 above it. In the window, the same x⁵⁰, the half line, the gold limit with its hollow mark at (1, 0) and point at (1, 1), the moving point and its red bar. At "That's a strip", under the window a lavender strip from x_50 to 1, hollow at x_50 (the curve is exactly 1/2 there), filled at 1, and the legend: curve above 1/2 on x_n < x ≤ 1. At "x equals one belongs to it", the filled end glows | magnifier (zoom); highlight | |
| 02_2 | But at one, the curve and the limit are both at height one. The gap there is zero. *(C1.domain.right, C1.limit.at_one, C2.gap_at_one)* | At "both at height one", a glow at the window's top right corner, where the curve ends on the gold point; at the window's top left: "at x = 1: curve 1, limit 1, gap 0" | highlight | |
| 02_3 | So the gap is at least one half only on a strip that starts at the crossing and stops just short of one. *(C1.half, C1.domain.right)* | At "a strip that starts", under the lavender strip a red one from x_50 to 1, filled at x_50 (the gap is exactly 1/2 there), hollow at 1 (the gap is zero there), with the legend: gap at least 1/2 on x_n ≤ x < 1. The same red strip appears under the main plot's x axis, at its true width | highlight; morph that keeps identity | |
| 02_4 | As the power grows, that strip gets narrower. *(C2.powers; C2.width.n50.6dp to n1000.6dp on screen)* | The main plot's bright curve morphs through x¹⁰⁰, x²⁰⁰, x⁵⁰⁰, x¹⁰⁰⁰, the moving point and its bar riding toward 1; the window, still ×10, morphs with it, its strips narrowing (their end marks fade once the ends are too close to tell apart); "n = 100: width 0.006908" and on to "n = 1000: width 0.000693". Then a short gold arrow from the right points at where the main plot's strip is, at its height between the hollow mark and the axis numeral, without widening it, and two dim lines: "the plot is 1,296 pixels wide in the 4K picture; from n = 898 on, the strip is narrower than one pixel" (C2.pixels.plot_4k, C2.subpixel_from) | morph that keeps identity; live readout | |
| 02_5 | ⟨predict⟩ Once it's thinner than a pixel, is the trouble gone? | A "?" in the think ring, which drains for 4 s with "pause for more time" under it; the answer stays hidden until it ends | predict pause | think 4 s, the answer hidden until it ends |
| 02_6 | No. Stretch the end of the interval sideways, ten times, a hundred times, a thousand times. The strip is still there. *(C2.zoom.0, C2.zoom.1, C2.zoom.2)* | The "?" and the ring go. The window is at ×10. At "a hundred times", window and magnifier morph to ×100 (ends 0.99 and 1); at "a thousand times", to ×1000 (ends 0.999 and 1): the strip for n = 1000 fills most of the window again, its end marks back. The magnifier on the plot shrinks to its true width, about 1.3 pixels of the 4K picture at ×1000 | magnifier (zoom); morph that keeps identity | |
| 02_7 | Reset the window to x to the tenth. Raise the power tenfold and stretch tenfold, then again. It looks almost the same. Its width shrinks like point six nine over n, and the strip is never empty. *(C2.restart_power, C2.zoom.step, C2.ln2.2dp; C2.share.n10.3dp, n100, n1000 on screen)* | The magnifier, readout and pixel lines go; the main plot stays at x¹⁰⁰⁰ (only the window resets). On "Reset the window to x to the tenth", the window goes back to n = 10 at ×10, its label saying so, with the row "n = 10, stretched ×10: the strip fills 0.670 of the window". At "Raise the power tenfold", n = 100 at ×100 and its row, 0.691; at "then again", n = 1000 at ×1000 and its row, 0.693: nearly the same picture each time. At "Its width shrinks", lower-third, precisely: w(n) = 1 − 2^(−1/n) > 0; n·w(n) → ln 2 ≈ 0.6931. This is a limit, not a finite equality (C2.ln2.4dp; short enough to read in this sentence; the two-sided bounds behind it are in model.rs, oracle.py and the README) | morph that keeps identity; table; lower-third | |
| 02_8 | And the moving point is always inside it, with its gap of one half. *(C1.witness.gap)* | The lower-third goes. At "always inside it", glows on the window's moving point and on the red strip's filled left end | highlight | |
| 02_9 | A picture shows a few powers, at finite resolution. What we need is a guarantee for every point at once, and an order for who chooses what. | At "What we need", the rows go and two lines replace them: "a guarantee for every x at once" and, dim, "who chooses first: how close, which power, which point?" | card | hold |
