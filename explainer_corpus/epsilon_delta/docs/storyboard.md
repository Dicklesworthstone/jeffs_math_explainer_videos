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
| I. A limit that jumps | 01 a limit that jumps (built); 02 the shrinking region (built) | unbroken curves x^n on [0, 1], a limit that is 0 then 1; a fixed point waits a finite time, the moving point x_n = 2^(−1/n) keeps a gap of 1/2; a thin region is not an empty one |
| II. The guarantee | 03 what the old objection was about (built), 04 the challenge game (built), 05 why written proofs hide the search (built) | a nonzero increment divided out, then set to zero, against a limit that never uses zero; the tolerance band chosen first, then the input band whose whole image fits; δ may depend on ε and the point, never on the x the challenger picks next |
| III. Making δ | 06 Sprague's recentring (built), 07 Davis's bound, 08 fixed or moving, precisely | δ = √(4 + ε) − 2 for x² at 2, the simpler δ = min(1, ε/5), the cubic's C = 22; the naive ε/4 fails at ε = 1 with h = 6/25; pointwise against uniform as an order of choices |
| IV. Repair and other languages | 09 the three-piece bridge, 10 sequences and bounds, 11 preimages, 12 what the definition does not supply | uniform convergence repairs the opening; one bad sequence refutes continuity; open sets pull back to open sets; completeness is a separate assumption |

One picture runs through act I: the plot of [0, 1] with the curves, the gold
limit, the teal fixed point and the lavender moving point. Act II's picture
is the tolerance band and the input band over the same kind of plot, so the
moving point returns as the challenger's choice. Chapters 01 and 02 name no
one. Chapter 03 names one person and one year, George Berkeley and 1734,
from the passage read at page level (the README's sources), and calls its
own example modern; no other name or date is spoken, and a test holds it.

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
draft04. Root's later silent drafts run 92.60 s for 03 (draft08, central27),
93.03 s for 04 (draft10, central41) and 93.30 s for 05 (draft13,
central49). All three chapters are built and root-checked on source,
mathematics and sampled frames, not on audio.

After chapter 02 the picture rests on the plot at n = 1000 with its
under-axis strip, the window at ×1000 and the two summary lines. Chapter 03
opens on it:

- **03_what_the_old_objection_was_about**: built (its beats are below,
  task mx-video-program-program-87do.58.3). It rebuilds chapter 02's
  resting picture with chapter 02's own builders and keeps the axes and the
  bright curve. Berkeley's objection in The Analyst (1734, §§XIII–XIV) is
  to a step in an argument for the fluxion of a power: suppose an increment,
  divide by it, then suppose it is nothing and keep what the division gave.
  The chapter shows that step on its own modern example, the square at
  x = 1/2, where the chord's slope is exactly 1 + h for every h ≠ 0. Setting
  h to zero merges the two points: the quotient is 0/0, and any line through
  the point fits. The limit never uses h = 0. The slopes at ±1/2, ±1/4 and
  ±1/10 land on a panel around a hollow mark at h = 0. Every h with
  0 < |h| < t is within t of 1, for each tolerance t drawn. The chapter
  ends on the band and window at t = 1/20 and the question of who picks the
  tolerance, which is chapter 04's.

After chapter 03 the picture rests on the plot (x², the base point, the
chord at h = −1/10, the gold line of slope 1) and the slope panel with its
band and window at 1/20, the limit, the line "next: who picks the
tolerance, and when?" and the precisely card. The challenge game (04)
opens on it:

- **04_the_challenge_game**: built (its beats are below, task
  mx-video-program-program-87do.58.4). It rebuilds chapter 03's last
  picture with chapter 03's own builders (`old_objection::final_picture`).
  The game has strict turns: the challenger's tolerance ε, then the
  defender's window δ > 0, then any h in the window but 0.
  - The honest answer δ = ε wins for every pick, because the slope is
    exactly |h| from 1.
  - A lucky pick does not save the wider window 1/5. It loses at
    h = (ε + δ)/2 = 0.15, the model's `window_failure`, whose pick is
    capped at 1/2 so that 1/2 + h stays in [0, 1].
  - A limit leaves h = 0 out; continuity puts it in and aims at the value
    there, and the third rule becomes |h| < δ. Filled with 1, the graph is
    continuous. Filled with 1.5, no window works, and the screen says why
    for every window, not just the one played: any δ holds the pick
    h = ½·min(δ, 2/5), whose value is at least 3/10 from 1.5
    (`fill_failure`; h = 0.05 for δ = 1/10).
  - A defender who saw a nonzero h first could answer with a window too
    narrow to hold it (`blind_window`); h = 0 is in every window but passes,
    g(0) being within ε of itself. So δ may depend on ε, the point and the
    curve, never on the later h.
  - Last, continuity of x² at x = 1, the end of its domain [0, 1], against
    1/2. The window runs from 2^(−1/2) ≈ 0.7071, which it leaves out (x² is
    exactly 1/2 there), up to 1, which it holds; δ ≈ 0.2929. No Sprague or
    Davis method is used: the bound is x² > 1/2 exactly when x > 2^(−1/2).
  - The precise domain conditions follow the Toronto MAT237 notes §1.2
    (README sources).

After chapter 04 the picture rests on the plot (x², the band within 1/2 of
1, the window from 2^(−1/2) to 1 with its end marks and its edge's label,
the highlighted piece), the right column's domain note, tolerance and four
rows, and the line "next: how a window is found, and why a written proof
hides the search". Chapter 05 opens on it:

- **05_why_written_proofs_hide_the_search**: built (its beats are below,
  task mx-video-program-program-87do.58.5). It rebuilds chapter 04's last
  picture with chapter 04's own builders (`challenge_game::end_picture`).
  Its example is continuity of x² at x = 1/2 on the stated domain [0, 1].
  - It first shows a finished proof whose δ = (2/3)ε comes unexplained.
  - Then the search, backward from the goal |x² − 1/4| < ε:
    - the gap factors as |x − 1/2|·|x + 1/2|;
    - on [0, 1] the second factor is at most 3/2, reached at x = 1. This
      is a deliberately generous bound, since near 1/2 it is about 1;
    - so |x − 1/2| < (2/3)ε is enough. That is δ (the model's
      `found_window`, ε/(1 + a)).
  - The prediction is quantitative: against ε = 1/10, where the window
    found is 1/15 ≈ 0.067, about how large could δ be and still work? The
    answer comes only after the think window: the widest symmetric window,
    measured where the curve leaves the band on the right, is ≈ 0.092
    (`widest_window_rounded`, certified, not derived on screen). 1/15 is
    smaller and still enough.
  - The finished proof is then read forward, line by line, in the
    definition's order, saying both conditions on x: in the domain and in
    the window.
  - It overlaps neither of the next two chapters' methods: Sprague's exact
    solution of a recentred bound, and Davis's coefficient sum.
  - It names no one.

After chapter 05 the picture rests on the plot (x², the band within 1/10
of 1/4, the base point, the window found and the widest window's outline,
the edge where the curve leaves the band), the finished proof lit, the
search's six rows, the two comparison rows and "next: making that search
systematic". Chapter 06 opens on it:

- **06_sprague_recentring**: built (its beats are below, task
  mx-video-program-program-87do.58.6). It rebuilds chapter 05's last
  picture with chapter 05's own builders (`hidden_search::end_picture`),
  then changes the setting on screen: a number line shows chapter 05's
  domain [0, 1] and point 1/2, then every real number and the point 2.
  - Its example is Sprague's: x² on all real numbers at a = 2, where it is
    4, shown magnified near (2, 4).
  - Recentred, h = x − 2, the change is exactly h² + 4h, and on either side
    of 2 its size is at most |h|² + 4|h|.
  - The prediction: near 2 the curve is nearly straight, slope 4, so try
    δ = ε/4, 1/40 against 1/10; does every h in it pass? No, but almost:
    0.0249 moves x² by 0.10022 (`slope_window_miss`). The slope alone loses
    at every tolerance, since at its edge the change is ε + ε²/16.
  - Both terms share the budget: adding 4 completes the square,
    (|h| + 2)² < 4 + ε, so δ = √(4 + ε) − 2, positive for every ε > 0, and
    every h with |h| < δ, either sign or h = 0, keeps |x² − 4| below ε.
  - At 1/10, δ ≈ 0.0248456731. The usual cap, |h| < 1, keeps |x + 2| < 5
    and gives δ = min(1, ε/5) = 0.02; both work.
  - It names Sprague, from his note as read (the README's sources), and
    leaves the general bound for any polynomial, Davis's, to chapter 07.

After chapter 06 the picture rests on the number line (every real number,
the point 2), the magnified square with its band within 1/10 of 4,
Sprague's window filled, the capped window's outline and the red ring
where the slope's window lost; the right column's eleven rows; the note's
citation and "next: a bound for any polynomial".

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
| the base point (1/2, 1/4) of chapter 03, chosen first and held | teal, as the fixed point |
| the increment h, the far point and the window of h (chapter 03) | lavender, as what moves; the window translucent |
| a chord, the change h + h², and the graph of the chord's slope against h | green `#83C167` for the chord and the graph (the change is the curve's, blue) |
| where the slopes go (chapter 03): the line of slope 1, the level 1, the limit, the tolerance band | gold, as the limit; the band translucent |
| the step that fails: h set to zero, 0/0, the fan of lines that all fit | red, as the gap and the jump |
| chapter 04's challenger: the tolerance rule (gold, as the band), the defender's window rule (lavender, as the window), the challenger's pick (lavender, a dot on the graph); a pick that misses the band | the band's and window's colours; a miss is ringed red |
| the hole filled with a value (chapter 04) | green, as the graph |
| chapter 05's finished proof: bright when shown or read, dim while the search runs; the search's rows | ink, the proof dimmed to `dim`; the found δ in lavender, as the window |
| the widest window (chapter 05), an outline only, and where the curve leaves the band | lavender stroke; the edge ringed in `hero` |
| chapter 06's domain on the number line, and its point | blue, as the curve; the point teal, as the base point |
| chapter 06's windows: the slope's (dashed edges), Sprague's (filled), the capped one (outline) | lavender, as the window |
| where the slope's window loses (chapter 06), and its row | red, as the step that fails |
| axes and their numbers | `dim` |
| the prediction ring, the title, the trace of an unbroken curve, a glow | `hero` gold `#FFD866` |

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

Chapter 03 adds **h** (the increment, said as the letter, never
"infinitesimal"), **slope** (of a chord, the change over h), **tolerance**
(how close to the limit a slope must be) and **modern notation** (our
example, said as ours before any step is shown). "Vanish" is said only
for the old argument's step, which Berkeley wrote as "let the Increments
vanish"; the modern step says "keep h nonzero" and "the limit". The
narration says Berkeley "questioned" and "objected"; it never says the old
calculus was wrong, and it says the old answer, one, is right.

Chapter 04 names the two quantities: **epsilon** is the tolerance and
**delta** the window, each said once with its word in 04_1. After that the
narration keeps to **tolerance** and **window**, and the screen writes ε
and δ. It also adds **challenger**, who picks the tolerance and then the
input; **defender**, who picks the window; and **pick**, an input the
challenger chooses. The narration says "continuity" and "limit" as two
different checks: the limit leaves the point out, and continuity puts it
in and aims at the value there.

Chapter 05 adds the **search** (the scratchwork, backward from the goal)
and the **proof** (the same argument, written forward from δ). It adds
**generous** for a bound chosen bigger than needed, and **widest** for
the widest window that works. It never calls the found δ best or optimal,
and it never derives the widest one.

Chapter 06 adds the **budget** (the tolerance ε, shared by the terms of
the change) and the **cap** (the usual first step, keeping h within 1). It
says "the size of h" for |h|, and "on either side" for both signs of h.

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
| "the slope at h = 0 is 1, since 1 + h is 1 there" | 03_4 to 03_5: at h = 0 the quotient is 0/0 and every line through the point fits; 1 + h equals the quotient only for h ≠ 0 |
| "early calculus was nonsense until the limit came along" | 03_3, 03_6: Berkeley questioned a step, the narration says so in those words, and the answer one is right |
| "a limit is the value you get by plugging in" | 03_6 to 03_7: the hollow mark at h = 0; the window 0 < \|h\| < t leaves h = 0 out, and the limit is reached without it |
| "this definition settles every question about limits" | 03_8: whether a limit exists at all is a separate question, saved for the end |
| "a few picks that land prove the window works" | 04_2: the reason for every pick is \|Q(h) − 1\| = \|h\| < δ = ε; 04_3: a lucky pick lands and the window still loses at 0.15 |
| "the defender may answer after seeing the input" | 04_5 to 04_6: then even the misfilled graph would win, with a window too narrow to hold the pick |
| "a limit and continuity are the same check" | 04_4: the limit leaves h = 0 out; filled with 1.5, the limit is still 1 but continuity fails for every window |
| "a window is symmetric even at the end of the domain" | 04_7: at x = 1 only x ≤ 1 is in the domain; the window is 2^(−1/2) < x ≤ 1 |
| "a written proof shows how δ was found" | 05_1 to 05_5: the proof opens with δ = (2/3)ε; the search that found it runs the other way, from the goal |
| "δ must be the widest window" | 05_6 to 05_7: at ε = 1/10 the widest is about 0.092; 1/15 is smaller and still works; a window only has to work for every input |
| "bounding a factor needs the factor near the point" | 05_4: the bound 3/2 holds on the whole domain [0, 1]; it is generous on purpose, and still enough |
| "near the point the curve is straight, so the slope alone sets δ" | 06_4 to 06_5: δ = ε/4 loses, by a hair, at 0.0249; h² is tiny, not zero, and the window must hold for every input |
| "without an interval, nothing bounds x + 2, so no δ can work on all real numbers" | 06_6 to 06_7: solving the budget gives δ = √(4 + ε) − 2 > 0 for every ε, with no cap |
| "Sprague's δ only works for small tolerances" | 06_7: it is positive for every ε > 0; at ε = 5 it is 1, at ε = 21 it is 3 (tests, not said) |

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

Chapter 03's units:
- **03u1** (03_1, 03_2, 65 words): Berkeley's objection named, then the
  modern step on the square up to the slope one plus h.
- **03u2** (03_3, 03_4, 38 words): the old step and Berkeley's objection
  to it, ending on the question.
- **03u3** (03_5 to 03_8, 97 words): the answer zero over zero, the limit
  reached with h never zero, the tolerance, and what comes next.

The question 03_4 is followed by the same 4 s think window, its answer
("None.") hidden until the window ends. The chapter ends on a hold.

Chapter 04's units:
- **04u1** (04_1 to 04_3, 77 words): the turns, the honest round and the
  wide window that loses.
- **04u2** (04_4, 04_5, 49 words): limit against continuity, ending on the
  question.
- **04u3** (04_6 to 04_8, 76 words): the answer and the rule for δ, the end
  of the square's domain, and what comes next.

The question 04_5 is followed by the same 4 s think window, its answer
("Yes") hidden until the window ends. The chapter ends on a hold.

Chapter 05's units:
- **05u1** (05_1 to 05_3, 59 words): the finished proof, the goal and the
  factoring.
- **05u2** (05_4 to 05_6, 50 words): the domain's bound, the window found,
  ending on the question.
- **05u3** (05_7 to 05_9, 91 words): the answer, the proof read forward,
  and the two directions.

The question 05_6 is followed by the same 4 s think window, its answer
("About point zero nine two") hidden until the window ends. The chapter
ends on a hold.

Chapter 06's units:
- **06u1** (06_1 to 06_4, 76 words): the new setting, h, the bound, and
  the slope's guess, ending on the question.
- **06u2** (06_5 to 06_7, 70 words): the answer, the budget for both
  terms, and δ for every ε.
- **06u3** (06_8, 06_9, 54 words): the value at one tenth, the cap's δ,
  and what comes next.

The question 06_4 is followed by the same 4 s think window, its answer
("No, but almost") hidden until the window ends. The chapter ends on a
hold.

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

## 03_what_the_old_objection_was_about: What the old objection was about (~84 s at the kit's scratch rate)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | The guarantee has to answer an old objection. In seventeen thirty four, George Berkeley questioned a step in an argument for how fast a power changes. *(C3.year; The Analyst §§XIII–XIV, pp. 5–6 of Wilkins's edition, read 2026-10-19)* | Chapter 02's resting picture, rebuilt by its own builders: the plot at x¹⁰⁰⁰ over its dim curves, the limit, the half line, the moving point and its gap bar, the strip under the axis, the window at ×1000 with its strips, end marks and legends, and the two summary lines. Everything but the axes, the bright curve and "a guarantee for every x at once" fades. At "George Berkeley" that line goes and, at the right, "George Berkeley, The Analyst (1734)" and, dim, "§§XIII–XIV, pp. 5–6: the fluxion of any power" (C3.year, C3.page.first, C3.page.last) | card | |
| 03_2 | Here is that kind of argument for a square, in modern notation. Move x from one half by h, not zero. The height changes by h plus h squared. Divide by h, and the slope is one plus h. *(C3.power, C3.base, C3.h.to, C3.change, C3.quotient)* | At "for a square" the bright x¹⁰⁰⁰ morphs into y = x², its label swapping, and under the heading: our modern example: y = x² at x = 1/2. At "Move x from one half", a teal dashed line up to the base point (1/2, 1/4), 1/2 under the axis, a teal dot; then the far point (1, 1) for h = 1/2 in lavender, the run h along y = 1/4 (C3.h.0). At "The height changes", the rise up to the far point in blue, labelled h + h², and the first step: (1/2 + h)² − (1/2)² = h + h². At "Divide by h", the chord through both points in green, across the plot, and the second step: (h + h²)/h = 1 + h, with (h ≠ 0) beside it in lavender | morph that keeps identity; derivation | |
| 03_3 | Then the argument lets h vanish, so one plus h becomes one. Berkeley objected that h was something for the division and nothing afterward, yet the result was kept. *(C3.quotient, C3.limit; The Analyst §§XIII–XIV, pp. 5–6 of Wilkins's edition)* | At "lets h vanish", the third step in red: then h = 0, so 1 + h = 1. At "Berkeley objected", his sentence, verbatim with two phrases left out where the ellipses stand: “the former Supposition that the Increments were something … is destroyed, and yet a Consequence of that Supposition … is retained.” and, dim, "Berkeley, The Analyst, §XIII, p. 6 (Wilkins's 2002 edition)" (C3.quote.page, C3.edition). The book and its year head the column above it; that Berkeley writes o for the increment and treats every power of x is in the README, not on screen. The sentence stays up through 03_4's question | derivation; quote | |
| 03_4 | ⟨predict⟩ So what slope does h equal to zero give? *(C3.h.to)* | Berkeley's sentence and the third step stay while the question is asked; when it ends they cross-fade into a "?" in the think ring under the first two steps, which stay. The ring drains for 4 s with "pause for more time" under it; the answer stays hidden until it ends | predict pause | think 4 s, the answer hidden until it ends |
| 03_5 | None. Both points are then the same point, and the change is zero over zero. *(C3.at_zero)* | The "?", the chord, the run, the rise and their labels go. The far point glides down the curve onto the base point at an even speed. At "zero over zero", seven red dashed lines through the point, at slopes from −1/2 to 3, and the step in red: h = 0: ((1/2 + 0)² − (1/2)²)/0 = 0/0, with, dim, "every line through the point fits: no slope" | highlight; derivation | |
| 03_6 | One is still the right answer, reached without h ever being zero. Shrink h from both sides, and the slopes close in on one. *(C3.limit, C3.h.to; C3.h.0 to C3.h.5, C3.slope.0 to C3.slope.5 on screen)* | The fan, the 0/0, the steps, the example and the heading go. At the right, a panel of the chord's slope against h, h from −0.6 to 0.6 with −1/2, 0 and 1/2 under it and 1 beside the slope axis: a gold dashed line at slope 1 with a hollow green mark on it at h = 0, the formula Q(h) = ((1/2 + h)² − (1/2)²)/h = 1 + h (h ≠ 0) above, and "slope of the chord, against h". At "Shrink h", the far point and its chord return at h = 1/2, a green dot lands on the panel at (1/2, 1.5), and "h = 1/2: slope 1.5" under it; the far point hops to h = 1/4, 1/10, −1/2, −1/4 and −1/10, the chord turning with it, each slope landing on the panel with its readout (1.25, 1.1, 0.5, 0.75, 0.9). At "close in on one", the readout goes, the graph of 1 + h is drawn through the dots in two pieces that stop at the hollow mark, and on the plot the gold line of slope 1 through the base point | plot; morph that keeps identity | |
| 03_7 | For every h within one tenth of zero, other than zero, the slope is within one tenth of one. Any tolerance works the same way, so the limit is one. *(C3.delta.0, C3.h.to, C3.eps.0, C3.limit; C3.eps.1, C3.delta.1 on screen)* | At "h within one tenth of zero", a lavender window over 0 < \|h\| < 1/10 up the panel, labelled 0 < \|h\| < 1/10 at its foot; at "the slope is within one tenth", a gold band from 0.9 to 1.1 across the panel, labelled tolerance 1/10. Inside the window the graph stays inside the band, reaching its corners only at \|h\| = 1/10, which the window leaves out. At "Any tolerance", band and window narrow together to 1/20, their labels with them, and then a lower-third, precisely: for every h ≠ 0, ((1/2 + h)^2 − 1/4)/h = 1 + h, so for every tolerance t > 0, 0 < \|h\| < t puts it within t of 1. At h = 0 it is 0/0, which the limit never uses (C3.base.height, C3.at_zero). It stays to the chapter's end, through 03_8 (about 17 s fully shown at the scratch rate). At "the limit is one", under the panel in gold: lim_{h→0} Q(h) = 1 | highlight; lower-third; morph that keeps identity | |
| 03_8 | This never needs the step Berkeley questioned. Who picks the tolerance, and when, comes next. Whether a limit exists at all is a separate question, for the end. *(The Analyst §§XIII–XIV, pp. 5–6 of Wilkins's edition)* | The precisely card stays. At "never needs", a glow at the hollow mark. At "Who picks the tolerance", under the limit and above the card, dim: "next: who picks the tolerance, and when?" | highlight | hold |

## 04_the_challenge_game: The challenge game (~84 s at the kit's scratch rate)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Make it a game with strict turns. The challenger names a tolerance, epsilon. The defender answers with a window, delta. Then the challenger picks any h in it but zero. *(C3.h.to)* | Chapter 03's last picture, rebuilt by its own builders. The limit, the "next:" line, the precisely card, the formula, the panel's title and the six hop dots go; the band and window at 1/20 stay. Over the panel, one rule with each turn: in gold "first the challenger picks a tolerance ε > 0", in lavender "then the defender picks a window δ > 0", in ink "then the challenger picks any h, 0 < \|h\| < δ" | card | |
| 04_2 | Against one tenth, the defender answers one tenth. Every h the challenger can pick lands in the band, since each slope is as far from one as h is from zero. *(C4.eps, C4.delta, C3.limit, C3.h.to; C4.pick.0 to C4.pick.2, C4.pick_slope.0 to C4.pick_slope.2 on screen)* | At "Against one tenth" the band narrows to within 1/10 of 1, labelled ε = 1/10; at "answers one tenth" the window to \|h\| < 1/10, labelled δ = 1/10, the hole still left out. At "Every h the challenger", a lavender pick lands on the graph at h = −0.08, then moves to 0.04 and 0.09, a glow at each landing and its readout in the panel's bottom right: "h = −0.08: slope 0.92", "h = 0.04: slope 1.04", "h = 0.09: slope 1.09". At "since each slope", under the panel in green: \|Q(h) − 1\| = \|h\| < δ = ε, the reason for every pick at once | morph that keeps identity; highlight | |
| 04_3 | Catching some h is not enough. Answer one fifth instead, and point one five lands outside. *(C4.wide, C4.catch; C4.lucky, C4.lucky_slope, C4.catch_slope on screen)* | At "Catching some h", the pick moves to h = 0.05: "h = 0.05: slope 1.05", in the band. At "Answer one fifth", the window widens to δ = 1/5 and the green reason goes (it no longer holds). At "point one five", the pick moves to h = 0.15, inside the window, and its slope 1.15 lands above the band: a red ring, and the readout "h = 0.15: slope 1.15" in red. This pick is the model's `window_failure`, (1/10 + 1/5)/2 | morph that keeps identity; highlight | |
| 04_4 | A limit leaves zero out. Continuity puts zero in and aims at the value there. Fill the hole with one, and it is continuous. Fill it with one and a half, and no window works. *(C3.h.to, C4.fill.good, C4.fill.bad; C4.fill.witness, C4.fill.witness_value, C4.fill.room, C4.fill.gap on screen)* | The pick, its ring and readout go; the window returns to δ = 1/10. At "leaves zero out", under the panel: limit: 0 < \|h\| < δ ⇒ \|Q(h) − 1\| < ε, and a glow at the hollow mark. At "Continuity puts zero in", under it: continuity: \|h\| < δ ⇒ \|g(h) − g(0)\| < ε, and a lower-third, precisely: a limit at a checks only the domain's points other than a, and needs some in every window; continuity at a point a of the domain checks a too, against f(a). Here g(h) = 1 + h for h ≠ 0, and g(0) is the filled value (C3.quotient). The third rule becomes continuity's: "then the challenger picks any h, \|h\| < δ", h = 0 let in. At "Fill the hole with one", a green dot fills the hollow mark. At "one and a half", the dot rises to (0, 1.5), leaving the hollow mark behind, and the band moves to within 1/10 of 1.5, its label under it. A pick lands at h = 0.05, inside the window, its value 1.05 far below the band: a red ring and "h = 0.05: g = 1.05". Over it, the argument for every window, so that one losing window is not all: "any δ: h = ½ min(δ, 2/5)" and, in red, "\|g(h) − 1.5\| ≥ 3/10 > ε". The pick shown is δ = 1/10's; the model's `fill_failure` gives the same pick for every δ, capped to stay in the domain | morph that keeps identity; lower-third; highlight | |
| 04_5 | ⟨predict⟩ But if the defender could see h first, could this broken graph still win? | A "?" in the think ring in the plot's empty top left, which drains for 4 s with "pause for more time" under it; the answer stays hidden until it ends | predict pause | think 4 s, the answer hidden until it ends |
| 04_6 | Yes, always, with a window too narrow to hold it. So the window may depend on the tolerance, the point and the curve, never on the later h. *(C4.cheat.delta on screen)* | The "?" goes. At "too narrow", the window shrinks to δ = 1/40, leaving the pick at 0.05 outside it: its ring, readout and the argument go, and in dim "h = 0.05: not in the window" (the model's `blind_window`, \|h\|/2). Over it, dim, the case no window can shut out: "h = 0 is in every window;" "\|g(0) − g(0)\| = 0 < ε": it passes anyway, so only nonzero picks can be hidden. At "So the window may depend", the precisely card gives way to a line under the definitions: δ may depend on ε, the point and the curve, never on the h picked after it; a glow at the defender's rule | morph that keeps identity; highlight | |
| 04_7 | Continuity at x equals one, where the square's domain ends. Against one half, the window starts where x squared is one half, and every x from there to one passes. *(C4.end.x, C4.end.eps, C4.end.power, C4.end.level; C4.end.edge.4dp, C4.end.delta.4dp, C1.domain.left, C1.domain.right on screen)* | Everything on the right, the rules, the definitions, the δ line, and on the plot the base point, chord, far point and gold line go; the square, its label and the axes stay. At "x equals one", a glow at (1, 1) and, dim at the right, "domain [0, 1]: no x beyond 1". At "Against one half", a gold band on the plot from y = 1/2 up past 1, and ε = 1/2 around 1 at the right. At "the window starts", a lavender window from x = 2^(−1/2) to 1, a hollow mark at its left end (x² is exactly 1/2 there, not within) and a filled mark at 1, which belongs; under the axis 2^(−1/2) ≈ 0.7071. At "every x from there", the square's piece over the window lights up inside the band, and the rows: 2^(−1/2) < x ≤ 1: 1/2 < x² ≤ 1; so \|x² − 1\| < 1/2 for every such x; dim, x = 2^(−1/2): x² = 1/2 exactly, not within; δ = 1 − 2^(−1/2) ≈ 0.2929 | plot; highlight; derivation | |
| 04_8 | A winning answer covers every input in its window. Finding them is a search that written proofs hide. | At "covers every input", a glow over the window. At "Finding them", centred under everything, dim: "next: how a window is found, and why a written proof hides the search" | highlight | hold |

## 05_why_written_proofs_hide_the_search: Why written proofs hide the search (~84 s at the kit's scratch rate)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | A finished proof opens with a choice like delta equals two thirds of epsilon, and never says where it came from. Here is the search. *(C5.ratio)* | Chapter 04's last picture, rebuilt by its own builders; everything but the plot's axes, the square and its label goes. Top right, a finished proof in three lines: Let ε > 0 and δ = (2/3)ε. If x ∈ [0, 1] and \|x − 1/2\| < δ, then \|x² − 1/4\| = \|x − 1/2\|·\|x + 1/2\| ≤ (3/2)\|x − 1/2\| < (3/2)δ = ε (C5.a, C5.value, C5.bound, C1.domain.left, C1.domain.right). At "never says where it came from", a glow at its δ = (2/3)ε; at "Here is the search", the proof dims | derivation; highlight | |
| 05_2 | Start from what the challenger demands. x squared must be within epsilon of a quarter, for every x near one half. *(C5.value, C5.a)* | At "what the challenger demands", the search's first row: goal: \|x² − 1/4\| < ε; on the plot a gold band around y = 1/4, labelled within ε of 1/4, and the base point (1/2, 1/4) with its teal guide and 1/2 under the axis. At "for every x near one half", a lavender window around x = 1/2, labelled δ = ? | derivation; plot | |
| 05_3 | That gap is the distance from one half, times x plus one half. *(C5.a)* | The second row: \|x² − 1/4\| = \|x − 1/2\| · \|x + 1/2\| | derivation | |
| 05_4 | On the domain, zero to one, x plus one half is never more than three halves. That is the deliberate, generous step. *(C1.domain.left, C1.domain.right, C5.a, C5.bound; C5.near on screen)* | At "On the domain", the third row: x ∈ [0, 1] ⇒ \|x + 1/2\| ≤ 3/2, and a glow at x = 1 on the axis, where the factor is largest. At "deliberate, generous step", dim: near x = 1/2 it is only about 1 | derivation; highlight | |
| 05_5 | So a distance below two thirds of epsilon is enough. That is where delta came from. *(C5.ratio)* | At "So a distance below", the fourth row: (3/2)\|x − 1/2\| < ε when \|x − 1/2\| < (2/3)ε. At "That is where delta came from", in lavender: δ = (2/3)ε, the window's label swaps to it, and a glow at the dimmed proof's δ | derivation; highlight | |
| 05_6 | ⟨predict⟩ Against one tenth, about how large could delta be, and still work? *(C5.eps; C5.delta, C5.delta.3dp on screen)* | At "Against one tenth", the band's label becomes ε = 1/10 and, under the search, in lavender: ε = 1/10: δ = 1/15 ≈ 0.067, the window found there (the band and window are drawn to this scale throughout). At "about how large could delta be", a "?" in the think ring in the plot's empty top left, which drains for 4 s with "pause for more time" under it. No widest outline and no number for it until the ring ends | predict pause | think 4 s, the answer hidden until it ends |
| 05_7 | About point zero nine two, where the curve leaves the band. One fifteenth is smaller, and still enough. A window only has to work for every input, not be the widest. *(C5.widest.3dp, C5.delta)* | The "?" goes. At "About point zero nine two", the answer: a lavender outline of the widest symmetric window, ≈ 0.092 either side of 1/2, and, dim: widest window: δ ≈ 0.092. At "where the curve leaves the band", a hero ring there, on the right (x² = 0.35 exactly there, so not within), and a glow on it. At "One fifteenth is smaller", a glow at the window found's row. The widest is measured from the curve and certified (`widest_window_rounded`), not derived | plot; highlight | |
| 05_8 | Now read the proof forward, as it is written. Given epsilon, take delta as two thirds of epsilon. For every x in the domain and in the window, the gap is below three halves of delta, which is epsilon. *(C5.ratio, C5.bound)* | The proof's lines light again one at a time, each with a glow: at "Given epsilon" its first, at "For every x in the domain and in the window" its second (x ∈ [0, 1] and \|x − 1/2\| < δ), at "the gap is below" its third | highlight; derivation | |
| 05_9 | The search ran backward from the goal. The proof runs forward, and hides it. Next, ways to make that search systematic. | At "The search ran backward", glows at the search's goal, then at its δ; at "The proof runs forward", glows at the proof's δ, then at its last line, ε. At "Next, ways", centred under everything, dim: "next: making that search systematic" | highlight | hold |

## 06_sprague_recentring: Sprague's recentring (~85 s at the kit's scratch rate)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Leave the interval. Take x squared on all real numbers, at two, where it is four. *(C3.power, C6.a, C6.value; C1.domain.left, C1.domain.right, C5.a on screen)* | Chapter 05's last picture, rebuilt by its own builders, goes. Top left, a number line from −1 to 3, with 0, 1 and 2 under it: its stretch [0, 1] lit blue, a teal dot at 1/2, and top right, dim: was: x ∈ [0, 1], a = 1/2. At "on all real numbers", the whole line lights, its ends opened by chevrons, and the dot moves to 2; under it: now: x ∈ ℝ, a = 2, a² = 4 (the value at the point; x itself ranges over every real number). At "where it is four", under the line, "magnified near x = 2" over the square across a stretch of x one tenth wide, with 2 and 4 on its axes and the teal point (2, 4) | plot; highlight | |
| 06_2 | Let h be x minus two. x squared minus four is h squared plus four h. *(C6.a, C3.power, C6.value, C6.slope)* | At "Let h be": h = x − 2. At "x squared minus four is", beside it: x² − 4 = h² + 4h | derivation | |
| 06_3 | On either side, its size is at most h squared plus four times the size of h. *(C3.power, C6.slope)* | At "its size is at most": \|x² − 4\| = \|h² + 4h\| ≤ \|h\|² + 4\|h\| | derivation | |
| 06_4 | ⟨predict⟩ Near two the curve is nearly straight, slope four. Try delta as epsilon over four, one fortieth against one tenth. Does every h in that window pass? *(C6.a, C6.slope, C6.naive.delta, S.eps)* | At "nearly straight", a glow at (2, 4). At "Try delta", two dashed lavender lines at 2 ± 1/40, labelled ε/4. At "against one tenth", the gold band \|y − 4\| < 1/10, labelled ε = 1/10, and in lavender: slope only: δ = ε/4 = 1/40. At "Does every h", a "?" in the think ring in the right column's empty lower half, which drains for 4 s with "pause for more time" under it. No input is shown losing until it ends: the curve leaves the band 0.00015 inside the dashed edge, about 2 pixels at 4K | predict pause | think 4 s, the answer hidden until it ends |
| 06_5 | No, but almost. h equal to point zero two four nine moves x squared by point one zero zero two two. *(C6.naive.h, C3.power, C6.naive.change.5dp)* | The "?" goes. At "No, but almost", a red ring where the curve crosses the band's top at x = 2.0249, inside the dashed edge, with a glow. At "moves x squared by", in red: h = 0.0249: \|x² − 4\| ≈ 0.10022 > 1/10 (exactly 0.10022001; `slope_window_miss`) | highlight | |
| 06_6 | Budget both terms. Adding four completes a square, the size of h plus two, squared, below four plus epsilon. *(C6.added, C6.half_slope, C3.power)* | At "Budget both terms", a glow at \|h\|² + 4\|h\|. At "completes a square": \|h\|² + 4\|h\| + 4 = (\|h\| + 2)² < 4 + ε | derivation; highlight | |
| 06_7 | So delta is the square root of four plus epsilon, minus two, positive for every epsilon. Every h within delta of zero, on either side, keeps the gap below epsilon. *(C6.added, C6.half_slope, C6.h.at_a)* | At "So delta is the square root", in lavender: δ = √(4 + ε) − 2 > 0 for every ε > 0, with a glow at "positive for every epsilon". At "Every h within delta": 0 ≤ \|h\| < δ ⇒ \|x² − 4\| ≤ \|h\|² + 4\|h\| < δ² + 4δ = ε; on the plot the dashed lines go and the window \|x − 2\| < δ fills in lavender, labelled δ, the red ring just outside it | derivation; plot | |
| 06_8 | Against one tenth, about point zero two four eight. The usual way first keeps h within one, so x plus two stays below five. Delta is the smaller of one and epsilon over five, point zero two. *(S.eps, C6.delta.4dp, C6.cap, C6.a, C6.cap.factor, S.delta_simple.decimal; S.delta_sprague.10dp on screen)* | At "about point zero two four eight", in lavender: ε = 1/10: δ ≈ 0.0248456731. At "The usual way": \|h\| < 1 ⇒ \|x + 2\| < 5: δ = min(1, ε/5) = 0.02, and on the plot that window's lavender outline, labelled min(1, ε/5) under the curve. At "the smaller of one", a glow on its δ | derivation; plot; highlight | |
| 06_9 | Both work. Sprague's note solves the budget instead of capping it. Next, a bound for any polynomial. | At "Both work", glows on the two δ rows. At "Sprague's note", under the plot, dim, the note as read: A. H. Sprague, A Note on δ and ε, Amer. Math. Monthly 67 (1960), p. 780 (C6.note.volume, C6.note.year, C6.note.page). At "Next, a bound", centred under everything, dim: "next: a bound for any polynomial" | highlight | hold |


