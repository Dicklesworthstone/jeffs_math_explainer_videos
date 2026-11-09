# ADR-0012 — `joint_type`'s names mean what they say, and the round join is the default

**Status:** Accepted
**Date:** 2026-07-26
**Bead:** fm-oac (W5: strokes)
**Amends:** policy under D-05; resolves the question G0-2 finding L6 left to this bead; trues up §10.3

## Context

Two facts settled by G0-2's look study, both measured rather than remembered.

**The round join is what the mathematics gives.** A true curve-distance stroke's
coverage is the signed excess `min over segments of (distance − half_width)`.
Two segments sharing an anchor contribute two distance fields whose minimum is
*exactly* a round join — no notch, no gap, no bookkeeping — and the same
expression puts a round cap on every open end. L6's calibration capture
`fmn-joints-and-caps.png` renders four identical rows on purpose: under this
mechanism the Reference's four joint types collapse to one, and the round result
shows no notch or gap at any vertex where the Reference does. L6 explicitly left
to this bead the question of whether `miter`/`bevel` are then offered as
overrides.

**The Reference's constants are swapped.** From `stroke/geom.glsl:110-122`:

```glsl
if      (joint_type == BEVEL_JOINT) miter_factor = 0.0;
else if (joint_type == MITER_JOINT) miter_factor = 1.0;
else { mcat1 = -0.8; mcat2 = mix(mcat1, -1.0, 0.5);      // = -0.9
       miter_factor = smoothstep(mcat1, mcat2, cos_angle); }
float shift = (cos_angle + mix(-1, 1, miter_factor)) / sin_angle;
```

`miter_factor = 0` gives `shift = (cos θ − 1)/sin θ = −tan(θ/2)`, the **pointed**
corner. `miter_factor = 1` gives `+cot(θ/2)`, the **blunt** one. So the constant
named `BEVEL_JOINT` (Python `"bevel"`, code 2) draws a mitre and `MITER_JOINT`
(`"miter"`, code 3) draws a bevel. The captures corroborate it independently:
measured apex widths were auto 4 px, bevel 6 px, **miter 9 px** — miter visibly
*blunter*. There is also no `miter_limit` constant anywhere; L6 derived the de
facto limit from `auto`'s crossover at `cos_angle = −0.8`, giving
`√(1 + 3²) = 3.1623` half-widths of miter length.

§10.3 asks for "round/bevel/miter joins with a real miter limit plus a smooth
`auto` join tuned in the look study", so the overrides are in scope; what needed
deciding is which corner each of the four API values draws.

## Decision

1. **`Auto` and `NoJoint` render the round join.** `Auto` because L6 measured it
   equal to the Reference's `bevel` at the calibration angle and the round result
   is cleaner at every vertex; `NoJoint` because the Reference's "no join" is a
   *notch* — a visible gap at the outer corner, an artifact of a ribbon pipeline
   — and D-05 grants no obligation to reproduce one. A distance field cannot
   produce a notch without being told to carve one, and it will not be told.

2. **`Bevel` cuts flat and `Miter` comes to a point.** Our names mean what they
   say. This deliberately does **not** reproduce the Reference's swapped
   behaviour: a scene that asks for `"bevel"` and gets a bevel is faithful to
   what its author asked for, and the alternative is to propagate a misnamed
   constant into a second engine forever. The numeric codes remain accepted
   unchanged for source compatibility (`JointType::from_code`), so nothing
   fails to load; what changes is which corner code 2 and code 3 draw.

3. **The miter limit is `MITER_LIMIT = √10 ≈ 3.1623` half-widths of miter
   length**, L6's measured de facto value, and exceeding it falls back to the
   bevel — the classical behaviour, and the only thing standing between a
   near-reversal and a spike of unbounded length.

4. **The overrides are edits to the round join, not a separate construction.**
   A bevel's region is a subset of the round join's, so it is applied as `max`
   with the chord's half-plane inside the corner wedge; a miter's contains the
   round join's near the tip, so it is `min` with the two offset half-planes.
   Both are confined to the wedge whose nearest path point is the anchor. The
   consequence is structural rather than careful: for the round settings
   `join_wedges` returns nothing, so the round path is bit-identical whether or
   not the join machinery exists — asserted pixel-for-pixel by test rather than
   argued.

## Consequences

- **User-visible**, and recorded in **BN-06**: a scene that set `joint_type`
  explicitly to `"bevel"` or `"miter"` renders the *other* corner from the
  Reference's. A scene that left it alone — which is the overwhelming majority,
  since `Auto` is the default — renders a round join that is cleaner than
  `auto`'s and measurably close to it. `no_joint` loses its notch.
- **Easier:** the default path has no join code in it at all, which keeps the
  hot loop one `min` over segments and keeps `certified` mode's arithmetic
  identical between a scene with corners and one without.
- **Forbidden:** reintroducing the notch, and reading joint semantics from the
  Reference's *names* anywhere in the tree. `JointType` already stores the
  numeric codes (`fmn-mobject/src/uniforms.rs`), which is what makes this ruling
  a one-place decision instead of a convention.
- **Created:** nothing new. The Look Gallery's joints-and-caps panel already
  exists and now has three distinct rows to review instead of four identical
  ones; that review is the gallery's, not a bead's.
- **Plan true-up:** §10.3 is amended in the same commit as this ADR.
