# BN-06 — The fill: analytic coverage, a defined gradient field, and a border that does not grow the shape

**Status:** Draft
**Workstream:** W5 (Lumen) · **Beads:** fm-5oi, fm-gmr · **Plan:** §10.2, §10.4, Appendix B
**Family:** BN-06 is the renderer's note (register rule 2). This file is its
**fill** half; the stroke half is [BN-06-strokes.md](BN-06-strokes.md).

## What changed

FrankenManim's fill computes **nonzero-winding coverage analytically on the
quadratic curves**. There is no triangulation, no signed-alpha winding trick, no
full-frame supersampled canvas, and no `GL_MAX` composite pass. Standard mode
may selectively recompose a complex native cell at 2×2 or 4×4, resolving it
immediately without retaining a high-resolution image; certified mode always
uses the canonical analytic path. Three consequences are visible in output,
and each is deliberate.

### 1. Edges are continuously antialiased, not quantized to five levels

The Reference's fill fragment shader carries no antialiasing term at all: a
fragment is in or out, and the only smoothing is the fill canvas's 2×2
supersampling, which yields **five** coverage levels — about 2.3 bits (G0-2
finding L3). Its *strokes*, by contrast, use a continuous smoothstep over a
1.5 px band.

Our fill's coverage is the exact area of the pixel inside the path, computed from
the curves' own integrals. Every intermediate value is available, so a filled
edge is as smooth as a stroked one.

Adaptive AA does not replace the native coverage kernel. A simple-edge cell
keeps its analytic result. When two or more independent boundaries occupy one
cell, standard mode evaluates the same kernel over 2×2 subcells (4×4 at four
crossings): the curve-area integral stays the curve-area integral, and an
admitted rectangle/disc/rounded-rectangle hint stays that exact primitive.
Subcells composite in painter order and average directly into the native pixel.
That selective path corrects overlapping-edge compositing; it does not quantize
an ordinary fill edge back into box samples. Forced 2×/4× remain available for
A/B comparisons.

**Migration:** filled shapes with no stroke look cleaner at their edges. Nothing
to change. If you added a stroke purely to hide fill stair-stepping, you no
longer need it — though keeping it is harmless.

### 2. `fill_border_width` no longer makes the shape bigger

This is the one that can shift a layout, so it is the one to read.

In the Reference, `fill_border_width` is not a border: `shader_wrapper.py` draws
it with the **stroke** shaders, in the fill's own colour, max-composited into the
fill canvas. Max of two equal colours cannot darken an interior, so its whole
observable effect is that **the filled silhouette grows by half the border
width**, with a sharper outer edge than the fill itself has. It exists to
compensate for the five-level fill coverage described above.

FrankenManim treats it as what §10.2 calls it — an **inner** border, a band
measured inward from the boundary — and an inner band is a subset of the filled
region, so it cannot change coverage. **The silhouette does not move.** The
reasoning and the alternatives are in
[ADR-0011](../adr/0011-fill-border-width-is-an-inner-border-and-does-not-dilate.md).

Concretely: `Text` and `DecimalNumber` default `fill_border_width = 0.5`, which
at 1920×1080 is 0.675 px of growth in the Reference — 0.3375 px on each side of
every stem. Under FrankenManim those glyphs render at their true weight, so text
is very slightly lighter than a Reference capture of the same scene.

**Migration:** nothing to change for correctness; positions, bounding boxes, and
`get_width()` were never affected by the Reference's growth either, because it
happened in the shader and not in the point data. If you are matching a Reference
render side by side and the text looks a hair light, that is this, and it is the
correct weight rather than the remembered one. If you *want* heavier text, set a
stroke — which is what a stroke is for.

For a **flat** fill the knob is now provably a no-op. For a **gradient** fill it
still does something, described next.

### 3. Gradient fills use a defined interpolation field

The Reference triangulates a filled path and lets the GPU interpolate the
per-point `fill_rgba` across the triangle fan, so the colour you get in the
interior depends on how the shape happened to be triangulated — subdivide a curve
and the gradient moves.

FrankenManim's interior colour is a **specified field**: the boundary ramp is
parameterized by true arc length, and the interior is its mean value interpolant
over 64 boundary stations placed at fixed arc-length fractions. A shape with
multiple contours is evaluated as a set of separately closed cyclic polygons:
the same signed mean-value weights are accumulated across every contour, so a
counter-wound inner contour contributes as a hole instead of acquiring a
fictional edge to the outer boundary. The 64-station budget is shared in
proportion to global arc length with a minimum of three stations per contour;
only shapes with more than 21 contours grow the budget to `3 * contours`.
Three properties follow, all tested:

- **Subdividing a path does not change its gradient.** Same curve, same colours.
- **Every contour closes to itself.** A hole never samples a bridge to another
  subpath, and its boundary ramp is interpolated exactly on that boundary.
- **The field is a function of geometry alone**, so restyling never rebuilds it.

`fill_border_width`'s remaining job lives here. Inside the band the colour comes
from the boundary ramp directly rather than from the smoothed field, so the
gradient's **seam** — where a closed path's end meets its start and the ramp
jumps from its last colour back to its first — is crisp inside the band and
blended outside it. The Reference has the same seam (its triangle fan has a hard
edge between its last and first vertex); what changes is that ours is a
consequence of a stated field rather than of a triangulation.

**Migration:** gradients look smoother and are stable under any operation that
resamples a path. If your scene relied on a particular triangulation's banding,
it will not reproduce — and there was no way to depend on that deliberately.

## Not a difference

Two things worth stating because they *look* like candidates:

- **Open subpaths fill as if closed.** So does the Reference
  (`get_triangulation` closes each subpath), and so does SVG's `fill` rule. No
  change.
- **Self-intersections follow the nonzero rule.** A pentagram's core is wound
  twice and is therefore inside, exactly as in the Reference. No change.

## Evidence

- `crates/fmn-render/src/fill.rs` — the fill, its oracles, and the measurements
  quoted above (exact enclosed area by Green's theorem; subdivision invariance of
  geometry and of single- and multi-contour gradients; exact native/subcell area
  agreement; per-contour closure and hole-boundary interpolation; boundary
  crossing classification; the 0.500 seam disagreement; the flat-fill no-op).
- `crates/fmn-render/src/engine.rs` — adaptive thresholds, tile/cell
  instrumentation, the G0-2 two-AA-band stroke eligibility rule, fused resolve,
  certified policy invariance, and the version-1 forced-4× visual budget.
- [`docs/g0/G0-2-look-study-ratification.md`](../g0/G0-2-look-study-ratification.md)
  — finding L1 (the 1.5 px smoothstep, measured 1.560 px) and finding L3 (the
  Reference's five-level fill coverage).
- [`ADR-0011`](../adr/0011-fill-border-width-is-an-inner-border-and-does-not-dilate.md)
  — the `fill_border_width` ruling and the Reference trace behind it.
- Plan §10.2 and Appendix B's `quadratic_bezier/fill` row.
- Appendix C row **C-12** carries the Reference's ×0.95/×1.06 fill round-trip (a
  0.70 % overshoot on rgb *and* alpha of every filled shape it has ever
  rendered), written up in [BN-04](BN-04-color.md). It is a compositing
  difference rather than a coverage one, so it is cross-referenced rather than
  restated here — but it is the reason a side-by-side of a flat filled shape
  shows ours uniformly ~0.7 % darker.
