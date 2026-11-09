# BN-04 — Color: correct compositing, familiar gradients

**Status:** Draft (W1, fm-0dg). Finalized when Lumen's compositor lands (W5).

## What changed

Classic manim has no single color model. Colors pass through the `colour`
library (which stores HSL internally and wobbles rgb values on round-trip),
composite in whatever space the GPU pipeline happens to be in, and encode
however the driver and ffmpeg agree to. Two installs can and do disagree.

FrankenManim has exactly one pipeline (§6.3):

1. **Decode:** sRGB-encoded user colors decode to linear light
   (IEC 61966-2-1) at the render boundary.
2. **Composite:** all blending is Porter–Duff source-over on premultiplied
   linear-light RGBA.
3. **Encode:** the output transfer function is applied once, at the frame
   boundary, per the negotiated output format.

## What deliberately did NOT change

Manim's gradient *aesthetic* is part of the look, and the look is a product
requirement. Two Reference formulas are therefore kept bit-for-bit, applied
to sRGB-encoded components exactly as `manimlib/utils/color.py` applies
them:

- `interpolate_color(c1, c2, α) = sqrt(lerp(c1², c2², α))` per channel —
  the root-mean-square blend that keeps manim's gradients bright in the
  middle instead of muddy.
- `average_color(...)` — the RMS mean, same property.

These operate on *user-space colors* (styles, gradients, colormaps), before
the decode step. Compositing never uses them.

## The Reference's 0.70 % fill overshoot, which we do not inherit (C-12)

Worth knowing before anyone compares a filled shape side by side, because the
difference looks like a colour bug and is the opposite of one.

The Reference's fill pipeline scales fill alpha by `0.95` before its signed-alpha
winding blend (`quadratic_bezier/fill/frag.glsl:32`), which bounds the
`−a/(1−a)` singularity at `a = 1`, and then un-scales the resolved texture by
`1.06` (`shader_wrapper.py:489`). But the exact inverse of `0.95` is
`1.05263…`, so the round trip is `1.06 × 0.95 = 1.007`: **a 0.70 % overshoot
applied to rgb *and* alpha of every filled shape in every frame the Reference has
ever rendered.**

FrankenManim's fill computes coverage analytically and has no winding-blend
scaling, so there is nothing to un-scale and nothing to overshoot. A side-by-side
therefore shows our fills uniformly about 0.7 % darker and slightly less opaque
than the Reference's.

**Migration:** nothing to change; this is us being right. It is written down so a
Look Gallery reviewer measuring a filled panel does not read a systematic 0.7 %
as a regression in our colour pipeline. Measured and traced in G0-2's look study
(§8 of the ratification note); see also
[BN-06's fill note](BN-06-analytic-fill.md), which owns the rest of the fill's
differences.

## Migration guidance

- Scenes that read back composited pixel values will see different (more
  physically correct) results than classic manim, most visibly where
  translucent mobjects overlap: linear-light blending does not darken
  midtones the way gamma-space blending does.
- Gradients, `set_color_by_gradient`, colormaps, and `average_color` match
  classic manim to floating-point tolerance; no visual change.
- An **Oklab interpolation option** exists for users who want
  perceptually uniform ramps. It is opt-in, never a silent replacement;
  the default remains the Reference formula. Today it is Rust-only
  (`fmn_core::color::interpolate_color_oklab`); `fmn-python` does not
  expose it.

## Native pointwise callback colors

`set_color_by_rgb_func` and `set_color_by_rgba_func` accept vectorized Python
callbacks over live `(N, 3)` point tables. They write the native object's actual
paint fields: both `fill_rgba` and `stroke_rgba` for a `VMobject` (as `set_color`
does), and `rgba` for ordinary point/point-cloud/surface objects. Vector objects
have no generic `rgba` field; targeting it previously raised before any paint
could reach the renderer. Arbitrary GLSL injection remains excluded.

An `(N, 3)` RGB or `(N, 4)` RGBA result supplies per-record colors. A single color
vector or a `(1, 3)` / `(1, 4)` row also broadcasts to the current records. Components stay in the public encoded
color space; the existing render boundary, not this adapter, decodes them. Complex,
nonfinite, malformed and f32-unrepresentable outputs refuse before writing either
of the current vector object's paint lanes. Invalid RGB opacity refuses before
calling the function. Each invocation has a 1,048,576-point processing budget.

The canonical `functional_color` adapter keeps deterministic family order, live
point inputs and ordinary setter overrides. Shared descendants are evaluated once;
empty containers skip callback evaluation. Every callback result is frozen and
validated before any paint is published, so a late callback failure leaves the
entire family's earlier paint intact. Reentrant calls and callback-induced
family/geometry changes reject the obsolete paint plan. Authored side effects
are not rolled back. Each setter receives an independent copy: mutating the fill
setter's input cannot alter the later stroke write. Arbitrary setter failures are
not rollback-safe. See [the callable-field contract](BN-04-functional-color-fields.md)
for the full family and ownership semantics.

**Migration:** use the same public callback methods on vector shapes, text glyphs,
point clouds and surfaces. A callback need not fabricate a nonexistent `rgba`
field. This does not change the interpolation field or grant shader execution.

## Pointwise acceptance

`crates/fmn-python/tests/pointwise_color.py` exercises the actual native records,
live geometry, public callback/setter order, copies, mixed families, animation
builders and point-dependent stroke pixels. The complementary
`pointwise_paint_integration.py` requires complete native fill profiles and covers
interior-only color changes, opaque-endpoint/transparent-interior compositing,
real text and mathematics glyphs, and byte-identical PNG sequences at one and
four render threads. `demo/python/pointwise_paint.py` uses one live color field
for native strokes and fills on the shared scene clock.

## Evidence

- `crates/fmn-render/src/engine.rs` — the compositor this note was waiting for
  (fm-ig3). Steps 2 and 3 are literal there: a row accumulator of
  `fmn_core::color::PremulRgba` under `over`, narrowed once at writeback into a
  linear-light `Rgba16F` raw frame, with the output transfer function applied
  after that by `fmn_frame::convert::rgba16f_to_rgba8`.
- `crates/fmn-render/src/plan.rs::decode_rgba` — **step 1, which had no
  implementation until the engine needed it.** The record buffer holds
  sRGB-encoded components because `mobject.data` is API surface; the render IR's
  `Style` documents linear light; and nothing between them decoded anything. A
  mid-tone would have composited at its encoded value — visibly wrong, and wrong
  in the direction that reads as a lighting choice rather than a bug. The decode
  happens once per interned style row, through `fmn_frame::transfer::srgb_decode`
  rather than this crate's own `srgb_eotf`, because the former rides fmn-dmath
  and ADR-0010's first binding property requires it.
- `crates/fmn-core/tests/parity.rs::color_operations_match_the_reference` —
  418 fixture rows generated from the pinned Reference
  (`3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13`). They pass unchanged
  across ADR-0014's routing of `srgb_eotf`/`srgb_oetf`/Oklab onto fmn-dmath.
- `crates/fmn-core/tests/color_oracles.rs` — decode/encode identity on all
  8-bit code points, premultiply round-trip, source-over algebra
  (opaque-replaces, transparent-identity, associativity).
