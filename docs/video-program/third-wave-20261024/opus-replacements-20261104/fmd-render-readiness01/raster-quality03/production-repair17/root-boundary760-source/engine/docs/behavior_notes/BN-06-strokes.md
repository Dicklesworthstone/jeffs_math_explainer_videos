# BN-06 — Strokes: true curve distance, round caps, and joins that mean what they are called

**Status:** Draft
**Workstream:** W5 (Lumen) · **Bead:** fm-oac · **Plan:** §10.3, Appendix B
**Family:** BN-06 is the renderer's note (register rule 2). This file is its
**stroke** half; the fill's half is
[BN-06-analytic-fill.md](BN-06-analytic-fill.md).

## What changed

A FrankenManim stroke is the set of points within its half-width of the *actual
curve*, measured by true distance. The Reference builds strokes as polyline
ribbons of at most 32 segments per curve with a per-vertex offset construction —
which is why it has four joint types, butt caps, and a visible notch. None of
that survives, and the differences below all follow from the same replacement.

### 1. Open ends have round caps

The distance to an open end is radial, so the stroke's boundary there is a
semicircle. The Reference's butt caps were a property of its ribbon geometry, not
a design choice; round is the deliberate one.

**Migration:** a stroked open path is very slightly longer at each end — by half
its stroke width, in the direction of the tangent. `get_width()` and every
positional query are unaffected, since they read points and not pixels. If a
scene depended on a butt cap sitting exactly on the end anchor (an arrow tip
butted against a line, say), the tip now overlaps by a half-width; that overlap
is invisible when the two are the same colour and is the reason `ArrowTip`
attaches by arc length rather than by pixel.

### 2. Corners are round, and `no_joint` no longer notches

Two segments sharing an anchor contribute two distance fields whose minimum is
exactly a round join. G0-2's look study measured this against the Reference's
four types and found the round result cleaner at every vertex: the Reference's
`auto` and `bevel` are indistinguishable at the calibration angle (apex widths
4 px and 6 px against `miter`'s 9 px), and `no_joint` leaves a visible notch at
the outer corner.

`joint_type` still accepts all four values, and `auto` — the default — renders
the round join. `no_joint` also renders the round join: its notch is a ribbon
artifact, and a distance field cannot produce a gap without being told to carve
one.

**Migration:** nothing to change. Corners look cleaner. If you were setting
`joint_type="no_joint"` to *get* the notch, there is no longer a way to.

### 3. `bevel` and `miter` are swapped relative to the Reference

This is the one that can surprise you, and it is a fixed Reference defect rather
than a new choice.

In `stroke/geom.glsl`, the constant named `BEVEL_JOINT` produces `miter_factor =
0`, which gives `shift = −tan(θ/2)` — the **pointed** corner. `MITER_JOINT`
produces `miter_factor = 1`, which gives `+cot(θ/2)` — the **blunt** one. So the
Reference's `"bevel"` draws a mitre and its `"miter"` draws a bevel. Captured apex
widths confirm it: `miter` measures visibly *blunter* than `bevel`.

FrankenManim's names describe what they draw. `joint_type="bevel"` cuts the corner
flat; `joint_type="miter"` brings it to a point, with a real miter limit of
`√10 ≈ 3.1623` half-widths of miter length — the Reference's own de facto limit,
derived from `auto`'s crossover at `cos θ = −0.8`, since there is no
`miter_limit` constant to copy. Past the limit a miter falls back to a bevel.

**Migration:** if your scene sets `joint_type` explicitly to `"bevel"` or
`"miter"` *and* you want the corner the Reference drew, swap the two. If you set
it because you wanted the corner the word names, you now get it. The full trace
and reasoning are in
[ADR-0012](../adr/0012-joint-type-names-mean-what-they-say.md).

### 4. Width and colour interpolate by arc length

The Reference interpolates per-vertex `stroke_width` and `stroke_rgba` across its
ribbon, so a width ramp advances with the *parameterization* rather than with
distance along the curve — and manim's three inconsistent arc-density
conventions mean the parameterization is not uniform. FrankenManim interpolates
both by **true arc length** (BN-03's layer), so a stroke whose width varies does
so uniformly along the curve, as the eye expects.

Two consequences, both tested:

- **Reparameterizing a curve leaves the stroke identical.** Subdividing every
  segment at its midpoint changes the point array and not the curve; it does not
  move one pixel.
- A width ramp on a path whose segments have unequal arc lengths no longer
  bunches at the short ones.

**Migration:** ramped strokes look more even. A scene that tuned a `stroke_width`
array against the old bunching will look different — better, but different.

## Not a difference

- **The antialiasing profile is the Reference's, deliberately.** G0-2 finding L1
  measured it as `smoothstep(0.5, −0.5, excess/aaw)` over a band declared at
  1.5 px and measured at 1.560 px (RMS 0.0031, under one 8-bit level), and both
  the curve and the width are kept. Stroke edges should *feel* the same.
- **`STROKE_WIDTH_CONVERSION` is unchanged**: 0.01 scene units per width unit, so
  `DEFAULT_STROKE_WIDTH = 4.0` is 5.4 px at 1920×1080, exactly as before.

## Evidence

- `crates/fmn-render/src/stroke.rs` — the stroke, and the measurements quoted
  above (round-cap circularity, join arc with no notch and no gap, the
  reparameterization law, the per-segment width ordering, slab containment, the
  degenerate corpus).
- `crates/fmn-geom/src/distance.rs` — the nearest-point primitive, validated
  against brute force over a corpus with a straight segment, a cusp, a
  near-straight and an out-of-plane curve.
- [`docs/g0/G0-2-look-study-ratification.md`](../g0/G0-2-look-study-ratification.md)
  — findings L1 (the AA band) and L6 (the swapped constants, the measured apex
  widths, the derived miter limit).
- [`ADR-0012`](../adr/0012-joint-type-names-mean-what-they-say.md) — the joint
  ruling.
- Plan §10.3, and Appendix C row **C-13** (the swapped joint constants, as a
  Reference defect with its ruling).
