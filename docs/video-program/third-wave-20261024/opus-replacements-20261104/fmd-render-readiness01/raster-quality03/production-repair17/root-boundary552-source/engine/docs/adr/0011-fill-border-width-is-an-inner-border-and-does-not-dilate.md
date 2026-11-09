# ADR-0011 — `fill_border_width` is an inner border, and it does not dilate the silhouette

**Status:** Accepted
**Date:** 2026-07-26
**Bead:** fm-5oi (W5: analytic fill)
**Amends:** policy under D-05 (correct by default, documented when different); trues up §10.2 and Appendix B

## Context

§10.2 asks for "`fill_border_width` as a principled inner border stroke
(GL_MAX composite hack retired)". Implementing it required knowing what the
Reference's knob actually does, so it was traced rather than assumed:

- `shader_wrapper.py:284-291` builds `fill_border_program` from the **stroke**
  shaders, not the fill's.
- `shader_wrapper.py:315-316` binds `fill_rgba` to the `stroke_rgba` attribute
  and `fill_border_width` to `stroke_width`. So the border is a stroke drawn in
  the fill's own colour, at the same `STROKE_WIDTH_CONVERSION = 0.01` scaling
  every stroke width uses.
- `shader_wrapper.py:418-420` renders it with `glBlendFunc(GL_ONE, GL_ONE)` and
  `glBlendEquation(GL_MAX)`, commented "Now add border, just taking the max
  alpha".
- The stroke geometry shader centres a stroke on the path.
- `string_mobject.py:50` and `numbers.py:41` default it to `0.5`, against
  `VMobject`'s `0.0` — so `Text` and `DecimalNumber` ship with it on.

Max-compositing two equal colours cannot darken an interior, so **the entire
observable effect of the Reference's fill border is that the filled silhouette
grows by half the border width**, with the stroke's sharper antialiasing on the
outer rim. At the `Text` default that is 0.675 px of growth at 1920×1080. It is
a compensation for the fill pipeline's coverage resolution — G0-2 finding L3
measured that fill at five levels, 2.3 bits, against the stroke's continuous
smoothstep — and not a border in any sense a user would recognise.

That creates a conflict with §10.2's word "inner", because the arithmetic of an
inner band is different in kind. An inner band is a **subset** of the fill
region, so its coverage is pointwise no greater than the fill's; under
max-compositing in the same colour it cannot change coverage at all. Taken
literally and applied to coverage, "inner border" is vacuous.

## Decision

§10.2's "inner" stands, and the vacuousness is the finding rather than a
problem. Specifically:

1. **`fill_border_width` names a band of `border_width_px` pixels measured
   *inward* from the boundary**, where `border_width_px` is the Reference's own
   conversion (`STROKE_WIDTH_CONVERSION` scene units per width unit, then scene
   units to pixels — 1.35 px per width unit at 1920×1080).

2. **The band never changes coverage, and the silhouette never moves.** This is
   structural, not a tolerance: nothing on the coverage path reads
   `fill_border_width`. The Reference's `w/2` growth is **retired** under D-05 —
   it corrects a defect analytic coverage does not have, and D-05 grants no
   quirk-replication obligation. This is user-visible and is recorded as
   **BN-06**.

3. **For a flat fill the knob is exactly a no-op**, provably and by test. That
   is what makes retiring the growth safe rather than a silent change of
   appearance: the only fills whose pixels move are the ones whose pixels the
   band was always going to decide.

4. **The band's remaining job is colour.** Inside it, the colour is the boundary
   ramp evaluated at the nearest boundary point — crisp — instead of
   `GradientField`'s mean value interpolant, which is smooth by construction.
   The band's inner edge is antialiased with G0-2's measured profile
   (`smoothstep(0.5, −0.5, d/aaw)` at the calibrated 1.5 px), because the border
   is a stroke and that is a stroke's edge.

   Measurement locates the effect exactly. An interpolant converges to its
   boundary data, so away from the ramp's seam the field and the boundary agree
   to floating point one pixel in and the border changes nothing. At the seam —
   where a closed path's end meets its start and the ramp jumps back — they do
   not converge: the field blends the jump and reads `½` where the boundary
   reads `0`. Scanning a ring one pixel inside a 24-pixel circle, the maximum
   disagreement is **0.500**, at the seam, at every inset from 0.5 px to 4 px.
   So the knob's whole job is: *within the band, the gradient's seam is crisp
   instead of blurred.*

5. **Alternatives rejected, and why.** Giving the band its own alpha (an opaque
   rim on a translucent fill) would be a visible, arguably useful behaviour the
   Reference does not have, and inventing API semantics is not this bead's
   remit. Reproducing the `w/2` dilation would import a compensation for a
   defect we replaced, against D-05, and would render `Text` bolder than
   correct.

## Consequences

- **Easier:** the fill's coverage path takes no style input at all, which keeps
  §10.4's interior tile class exactly `1` and therefore keeps occlusion pruning
  bit-exact (G0-8b finding F13). A border that could raise coverage would have
  broken that.
- **Forbidden:** any later route by which `fill_border_width` moves a silhouette.
  If a Look Gallery panel argues for the Reference's boldening, the fix is
  Scribe's glyph weight or the AA policy, not a dilation smuggled through a
  border.
- **User-visible:** text and numbers set with the Reference's default
  `fill_border_width = 0.5` render ~0.675 px narrower than the Reference at
  1080p, at their true weight. **BN-06** carries the migration guidance.
- **Created:** nothing new. The nearest-boundary query this needed is
  `fmn_geom::distance::nearest_on_quadratic`, landed in Chisel under D-04 as the
  shared primitive §10.3's strokes (fm-oac) are defined by, so fm-oac consumes
  it rather than writing a second one.
- **Plan true-up:** §10.2's `fill_border_width` clause and Appendix B's
  `quadratic_bezier/fill` row are amended in the same commit as this ADR.
