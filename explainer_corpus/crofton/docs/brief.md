# Brief — Measure a Curve Without Following It

Topic owner: GentleLantern (Bead `mx-video-program-program-87do.22.1`).
What the film says, where each statement was checked, and what was not
read, watched or listened to. Numbers are facts printed by `oracle.py`
(keys in `code`); the Rust `src/facts.rs` prints the implemented subset
from an independent model. Root's 2026-10-25 native25 qualification covers
chapters01–15:180 release tests and198 native facts against302 oracle keys,
with no mismatches. This does not claim302 implemented native facts.

## The claim, and its edges

**The identity (Cauchy–Crofton).** For a rectifiable plane curve of length
L, with straight lines spread by the measure dp dθ (θ the lines' direction
over half a turn, p their signed distance from a fixed point), the number
of crossings n(p, θ) satisfies L = ½ ∫∫ n dp dθ. On a sheet of parallel
lines `gap` apart, slid and turned uniformly, the average count is
2L ÷ (π × gap), so one crossing is worth π × gap ÷ 2 (`coin.worth`, π cm
on the film's 2 cm sheet). The measure is the one, up to scale, that rigid
motions of the plane leave unchanged; the film shows why that matters
(slides alone, or a few fixed directions, give biased answers) rather than
proving the uniqueness.

**What the film never says.**

- A single placement, or ten, or a thousand random drops, measures the
  length. They give estimates with a spread (`drops.*`); the identity is
  about the average over every placement, which no finite sample reaches.
- The count is the length. The count depends on the gap; the count times
  a crossing's worth does not (root's review, Mail 6787: the old
  `π × mean travel ÷ gap` was right only at gap 2 and is replaced by
  `mean travel ÷ gap × π × gap ÷ 2` in both the oracle and the model, with a
  spacing control at gaps 1, 2 and 4 and a scale control).
- A real coastline has a length. The sheet measures the curve as drawn, at
  the detail it was drawn (the draft's chapters 26 and 27).
- That a seeded sample's spread is the method's. Every spread and count
  range the voice states is over every placement (below); the seeded draws
  are pictures, said only as "in a thousand drops" (CrimsonGrove's F3).
- That a smaller divider step always walks further. Each step cuts less off
  its bend, but the total is a sawtooth: from 0.25 to 4 cm in 0.05 cm
  steps a longer step walks further at 46 of 75 increments, and 1.9 cm
  walks 21.6 cm against 2 cm's 22.6 (`dividers.1_9.length`; F4).
- That one line can cross the coast many times. No line of positive area
  meets it more than six times (`chart.line_max`); a whole sheet's count
  reaches 13 (`drops.count_range`). About a third of the lines across the
  card miss the coast (`chart.miss_percent`, 35%), not most (F1).

**Barbier's route, and its fine print.** Implemented chapters 12 to 15
follow Barbier 1860's argument without integrals: crossings add over
pieces, so a curve's average count is the sum of its pieces' averages; a
straight stick's average is therefore proportional to its length (additivity
gives rational multiples; extending to every length uses that a longer
stick never averages fewer crossings); polygons, then curves, follow; a
coin as wide as the gap crosses twice except at isolated tangent placements,
which have probability zero under a continuous uniform slide. Its average
of two fixes the constant.
The voice states the steps; the limit from polygons to curves is named,
not proved.

Root checkpoint, 2026-10-25 04:26 UTC: chapters14/15 are registered and
qualified in `narration_lab/third-wave-20261024/root/tend-0404/RESULTS.md`.
Silent native25 previews last158.066667 and197.533333 seconds. Root reviewed
the implemented support geometry, exceptions, arc rearrangement, rendered
labels and actual sampled reading holds. The independent checker has2270
assertions and eight altered-claim controls; these are not Rust mutation
trials. Prior113 cue texts and119 direction entries are preserved exactly;
current totals are131 cues and142 authored directions. Chapters16–28,
narration performance, listening, fine synchrony, music, captions and the
final4K60 package remain open. Historical lane notes below retain their
original scope and are not a current compilation status.

## Sources

| Source | Read? | Used for |
|---|---|---|
| J.-É. Barbier, "Note sur le problème de l'aiguille et le jeu du joint couvert", *J. Math. Pures Appl.* (2) 5 (1860) 273–286 | read from a scan | p. 273 Laplace's remark on using probability to rectify curves; pp. 275–276 the expectation argument and the polygon induction; p. 277 (n° 6) "sans même avoir une seule intégration à effectuer", the round coin then deformed; p. 279 Théorème I (mean crossings 2Ll ÷ π, mean projection 2l ÷ π "comme pour le cercle"); pp. 283–285 (n° 8) curves of constant width, "leur longueur ne dépend que de leur largeur". p. 278 gives l ÷ (πa) for the wire, apparently a factor 2 off his own Théorème I: not quoted |
| A. Cauchy, memoir of 1832 | not read | the shadow form (mean projection = 2L ÷ π) and its discrete version, cited through Barbier pp. 276–277 ("insérée en 1832 dans un cahier lithographié, puis imprimée dans les Comptes rendus de 1841") and Bair et al., *Real Analysis Exchange* 45(1) (2020) |
| M. W. Crofton, *Phil. Trans. R. Soc.* 158 (1868) 181–199 | bibliography checked | the name and date of the formula (draft 19) |
| H. Steinhaus, *Ber. Sächs. Akad.* 82 (1930) 120–130; *Colloq. Math.* 3 (1954) 1–13 | not read | the longimeter (draft 24) and the Vistula remark (draft 26), the latter only as Mandelbrot quotes it |
| B. Mandelbrot, "How long is the coast of Britain?", *Science* 156 (1967) 636–638 | read from a scan | p. 636: Steinhaus's Vistula remark ("ten, hundred or even thousand times as great as the length read off the school map"); Richardson's law L(G) = M·G^(1−D), D = 1.25 for the west coast of Britain (draft 27's 19% per halving is 2^0.25 − 1) |
| L. F. Richardson, *General Systems Yearbook* 6 (1961) 139 | not read | cited through Mandelbrot 1967 only |
| J. F. Ramaley, "Buffon's noodle problem", *Amer. Math. Monthly* 76(8) (1969) 916–918 | bibliography checked | the name "Buffon's noodle" (draft 13) |
| S. Steinerberger, arXiv 2603.27807 (2026) | read through a web text extraction, not checked against the PDF | the longimeter's construction only. Its six-direction range, −2.26% to 1.15%, is not used: the closed forms are (2 + √3)π/12 − 1 = −2.2951% (a stick along a family) and (π/12) ÷ sin(π/12) − 1 = +1.1515% (midway), checked by `oracle.py` against a search over angles. Its +1.15% matches; its −2.26% is what a stick 0.0771° off a family gives (`oracle.py`'s bisection), as a sampled angle grid would, so it stays unexplained and out of the film. The draft's "−2.3% to +1.2%" is the closed form, rounded (CrimsonGrove's F5) |
| Wikipedia, "Steinhaus longimeter"; C. Staecker's printable TikZ longimeter | read | three square grids 30° apart (six directions), spacing 3.82 mm = 12 ÷ π mm |
| L. A. Santaló, *Integral Geometry and Geometric Probability* | not read | uniqueness of the invariant measure, as a background statement only |

## Competition check (bounded; search results only, nothing watched)

These are titles that searches returned. None was watched, so nothing here
describes their content beyond the title and listing.

- 3Blue1Brown, "Alice, Bob, and the average shadow of a cube" (listed
  20 Dec 2021): the 3D surface-area cousin (Cauchy's formula for convex
  bodies), not the plane-curve length identity.
- "Buffon's Noodle Problem, the Easy Way | The Theory of Probability"
  (YouTube `watch?v=_Ys4QFRfuoE`; channel not visible in the result).
- C. Staecker (Fairfield), a longimeter video in his antique devices
  playlist.
- "Steinhaus Longimeter Review / HowTo" (mentioned in a Gigazine article).
- No Numberphile or Mathologer result for Crofton, Buffon's noodle or the
  longimeter.

Editorial hypothesis only (no view or causal claim): a film that keeps one
drawn coast throughout, separates the exact identity from finite estimates,
and ends on the scale question seems not to be covered by the titles
above.

## Assumptions in the model

- The coast is four straight runs and seven circular arcs joined
  smoothly, length exactly 15/2 + 1399π/240 cm (`coast.length_exact`).
  Every count is computed on that exact curve: crossings by geometry, the
  average over directions by the closed-form shadow (Rust) or by sliding
  (oracle), never by a fitted constant.
- Dividers follow Richardson's rule: each step lands where the coast,
  followed onward, first comes exactly one step away.
- A line through a joint, an end or a tangent point is a placement of
  measure zero, so averages do not depend on how it is counted; every
  sheet shown on screen is placed where it keeps clearest of them
  (`shown_slide` in `oracle.py`), and the draft's slide stops short of the
  one tangent flicker it passes near (`SLIDE_SHOWN`).
- Random drops use a fixed seed (`SEED_DROPS` in `oracle.py`); the draft
  frames their closeness to 25.8 as luck and shows the spread, which is
  computed over every placement, not from the draws.

## How the count distributions are computed

How often each count turns up, for a whole sheet and for one line (the
chart), decides the draft's 19_1, 21, 22 and 25. Three computations, each
with its own counting:

- **`oracle.py`, closed form.** Over a direction β the heights where a
  count can change (ends, joints, arcs' tangent heights) are each
  a cos β + b sin β + c. Their order on the sheet changes only at
  directions where two heights differ by a whole number of gaps or one
  reaches a line (for one line, where two heights meet), and those
  directions are solved for directly. Between them every stretch's width
  is again a cos β + b sin β + c, integrated exactly by a three-point rule
  that is exact for that form (intervals under 2 × 10⁻⁴ rad use the midpoint
  value, off by under 10⁻¹¹); the slide is integrated exactly too. A guard
  fails if the order changes inside an interval (a missed direction), and a
  test breaks the direction list on purpose to see the guard fire. This is
  closed form up to floating-point rounding and that one midpoint; the film
  does not call its results exact.
- **`oracle.py`, grids.** Midpoint rules over 1,800, 3,600, 7,200, 14,400
  and 28,800 directions, counting crossings by direct line and piece
  intersection. The largest difference from the closed form falls from
  7.2e-6 to 2.0e-6, 7.0e-7, 1.8e-7 and 7.8e-8, and at every grid every
  shown spread, range, share and percentage rounds as the closed form's.
- **`src/model.rs`, 1,800 directions.** Its own midpoint rule, its own
  change heights and its own line meetings, printed through `facts.rs` and
  compared key by key (`facts-check`). A line-by-line Python
  transliteration of this uncompiled Rust reproduces all 20 new values.

Checks with known answers: a coin as wide as the gap crosses twice with
chance 1; Buffon's short needle crosses once with chance 2l ÷ (π × gap);
every line meeting a circle crosses it twice, and those lines fill as much
of the chart as its rim is long; the coast's sheets average
2L ÷ (π × gap) at gaps 2, 4 and 8; its single lines total 2L, and the lines
meeting it fill the perimeter of its convex hull, 32.73 (Cauchy), computed
separately; a square of side 2 shows four crossings 4/π − 1 of the time. A
single direction is the negative control: a 6 cm stick averages 3 lines
across it and 6/π over every direction.

## The pin lines (draft 16 to 18)

Lines through two pins dropped evenly on the card meet a figure with weight
the cube of their chord across the card (Crofton's chord formula,
`pin_rate_exact`, checked against 10,000 pin pairs). On this card the
chord, in every direction, is even and concave in the distance from the
centre, so its cube is largest through the centre: a stick moved off the
middle without turning always meets fewer pin lines (exact checks at
three angles and five centres). Turning as well can gain (lying across the
middle 12.44%, moved 3 cm right and stood upright 19.62%), so 18_4 says
"without turning". The corner stick's 12.4% is still above the even
sheet's 8%: "fewer" is against the middle, not against fairness.

## Protected facts (chapter 01)

Card 16 × 9 cm (`card.width`); ends 14.7 cm apart (`coast.ends_apart`); the
ruler fits the 3.5 cm beach (`ruler.beach_contact`) and touches the
headland once (`ruler.headland_touches`); 2 cm dividers: 11 steps and
0.64 cm, 22.6 cm (`dividers.2_0.*`), step 6 cutting across 3.08 cm of
coast; 1 cm: 25 steps, 25.1 cm; 1.9 cm: 11 steps, 21.6 cm (the draft's
25_6; chapter 01 claims nothing about it). The true length (25.81 cm) stays off
screen and out of the voice until the draft's 06_4.

## Not established by anything here

Rendered scene clarity, natural pacing, voice timing and English-accent
acceptance: nothing has been rendered, synthesized or listened to.
Pronunciation risks are listed in `../lexicon.json` for later
qualification.
