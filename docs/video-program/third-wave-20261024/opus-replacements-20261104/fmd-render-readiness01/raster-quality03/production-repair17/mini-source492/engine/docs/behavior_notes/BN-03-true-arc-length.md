# BN-03 — True arc length under the original names

**Status:** Draft (W2, fm-xci). Consumed by W7 (dashes, tips, tracers)
and Choreo's `MoveAlongPath`.

## What changed

Classic manim's proportion/length layer is chord-distance heuristics —
three mutually inconsistent approximations:

- `get_arc_length` returns a blend of anchor-polyline and full-polygon
  length (`interpolate(inner, outer, 1/3)`), not arc length;
- `point_from_proportion` weights curves by *chord* length and walks the
  curve's parameter linearly within each — parameter speed, not arc
  speed;
- `quick_point_from_proportion` assumes all curves have equal length.

FrankenManim keeps the names and fixes the math (D-05):

- `get_arc_length` returns the actual arc length. A quadratic Bézier's
  speed is the norm of a linear function, so each curve's length is a
  **closed form** (exact to rounding; logarithm via fmn-dmath — no
  quadrature, no tolerance knob). Degenerate cases (lines, interior
  cusps) have exact branches.
- `point_from_proportion` — and therefore `MoveAlongPath`, dashes, tips,
  and tracers — places by **true arc length**: constant-speed motion, as
  every user always assumed it worked. Inversion is Newton on the exact
  partial-length form with a bisection safeguard, fixed iteration count.
- `quick_point_from_proportion` remains, verbatim, as the documented
  fast approximation — a labeled option, never a silent substitute.
- The per-path `ArcLengthTable` is a §10.8 retained artifact keyed by
  the geometry revision only: transforms and style changes never rebuild
  it (`CachedArcLength`, locked by test).

## Migration guidance

Paths animate at slightly different (correct) pacing than the Python
engine. Example: a `MoveAlongPath` over a path made of one short curve
followed by one long curve. In classic manim the mover crosses the short
curve in (roughly) half the run time — chord weighting per curve, then
parameter speed inside it — so its apparent speed visibly jumps at the
seam. Here it moves at constant speed for the whole run: the short curve
takes its fair share of time and no more. Scenes that (usually
inadvertently) relied on the speed jump will look smoother; keyframe
timings tied to "the mover reaches the seam at t/2" shift to the true
arc-length fraction.

`get_arc_length` values change too: the Reference's blend systematically
overestimates curved paths (its outer polygon dominates). Dash counts and
tip placements derived from it move accordingly — by design.

Points sampled with `point_from_proportion` move as well, and a decision
taken on them can flip near its threshold. `_2021/bertrands_paradox.py`
`CompareFirstTwoMethods` builds random chords from `circle.pfp(random())` and
colors those longer than the inscribed triangle's side. One chord in a
thousand changes color (corpus differential row
`bn03-chord-length-threshold`).

Tip attachment also follows the actual curve. Classic manim aims a curved
tip from the final control handle and then shortens the shaft by applying
`put_start_and_end_on` to the whole path. The head can therefore follow a
component-local approximation while the shaft is chord-rescaled away from
its original curve. FrankenManim aims the head along the true terminal
quadratic derivative and cuts the shaft back by true arc length, preserving
the remaining curve. Straight shafts are unchanged. Scenes that deliberately
compensated for the classic curved-tip tilt or rescaling should remove that
compensation; custom `ArrowTip` subclasses keep their Python identity and
style while the native placement algebra transforms their point run.

## Evidence

- `crates/fmn-geom/src/arclength.rs`; `crates/fmn-geom/tests/arclength.rs`
  (closed form vs a 20k-interval Simpson oracle at 1e-9; the analytic
  parabola √2 + asinh 1; exact line/cusp/point branches; inverse
  round-trip; constant-speed metamorphic test on an uneven path — with a
  demonstration that the quick heuristic diverges there; revision-keyed
  cache tests).
- Reference: `VMobject.get_arc_length` / `point_from_proportion` /
  `quick_point_from_proportion` at the pinned commit.
- `crates/fmn-library/src/tip.rs` (true terminal tangents, reusable built-tip
  placement, true-length trimming, straight/curved/degenerate tests).
- `crates/fmn-python/tests/bridge.py` (authored `TipableVMobject` MRO and full
  detached/scene-bound lifecycle with curved placement and family identity).
