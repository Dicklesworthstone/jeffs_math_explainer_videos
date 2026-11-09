# BN-07 — Stroke uniform accessors corrected (C-2), and `use_winding_fill` accepted as a no-op (C-7)

**Status:** Draft (W3, fm-jru). Consumed by W5 (Lumen's `StyleTable`,
which synchronizes from the uniform inventory) and the Parity Ledger.

## What changed

Two Appendix-C rulings are owned by the uniform inventory (§8.4).

### C-2 — `get_scale_stroke_with_zoom` reads the correct uniform

In the Reference (`vectorized_mobject.py`), the accessor reads the wrong
key:

```python
def get_scale_stroke_with_zoom(self) -> bool:
    return self.uniforms["flat_stroke"] == 1.0
```

`scale_stroke_with_zoom` and `flat_stroke` are independent uniforms, so
the Reference's getter reports the *flat-stroke* state under the
*scale-with-zoom* name — a latent bug (the two default to the same value,
so it is invisible until either is set). FrankenManim's
`get_scale_stroke_with_zoom` reads
[`Uniforms::scale_stroke_with_zoom`](../../crates/fmn-mobject/src/uniforms.rs),
the correct field (D-05). `get_flat_stroke` reads `flat_stroke`. Both are
locked by unit tests that set the two flags independently and assert each
accessor reflects its own field.

### C-7 — `use_winding_fill` is an accepted no-op

`use_winding_fill` is already dead in the shipped Reference — the ear-clip
fill path it once toggled is not used by the OpenGL fill, and the method
body is literally `return self` ("Only keeping this here because some old
scene call it"). FrankenManim's fill is analytic nonzero-winding coverage
evaluated on the curves (§10.2), which never needed the flag either. We
therefore **accept the API for source compatibility as an explicit
no-op**: the flag is stored on the uniform inventory (so a round-trip
through `mobject.uniforms` is faithful) and readable, but it changes no
rendered bits. A test toggles it on a real fixture scene and asserts that
no point moves and the bounding box is untouched.

## Migration guidance

### Native-built shapes accept inherited rendering options

The shared native-constructor style route now accepts `joint_type`,
`anti_alias_width`, `scale_stroke_with_zoom`, `depth_test`, and
`is_fixed_in_frame`. These options use the existing live setters and reach
the generated family, including dashed paths and number glyphs. They are
no longer rejected merely because a class bypasses `VMobject.__init__`
while constructing its geometry in Rust. Omitted options retain the
constructor's native defaults. Unknown joints and negative or non-finite
anti-alias widths are refused before this style pass mutates the family.

### SurfaceMesh styles apply to the visible wireframe

`SurfaceMesh(..., stroke_width=..., stroke_color=..., depth_test=...,
joint_type=...)` applies those settings to every generated path, not just
the point-free group. The Reference left constructor settings on that empty
parent, so the visible paths used unrelated VMobject defaults. Native and
Python wireframes now agree on the explicitly requested style. A default
mesh uses width `1`, `GREY_A`, depth testing and `no_joint`; specify width
`4`, `auto` joints and `depth_test=False` to request the old visible defaults.

The Python constructor also consumes the source's **current sampled grid**,
including rotations, nonuniform stretches, live point edits and authored
`get_unit_normals` overrides. Any `Surface`/`ParametricSurface` can supply
the grid. It never calls the original UV function again, rebuilds a solid
at its original resolution, or recenters an asymmetric wireframe. A mesh
is a snapshot; construct another mesh (or use `always_redraw`) to follow
subsequent source changes.

- Code that called `get_scale_stroke_with_zoom()` and happened to rely on
  it returning `flat_stroke` (almost certainly none — the coincidence was
  never documented) now gets the honest answer. Read `get_flat_stroke()`
  for the flat-stroke state.
- `use_winding_fill(...)` continues to accept calls and does nothing, as
  in the Reference. There is no behavior to migrate; the analytic fill is
  correct without it.

## Uniform inventory (for reference)

The complete typed per-object inventory now lives in
[`Uniforms`](../../crates/fmn-mobject/src/uniforms.rs): `is_fixed_in_frame`
(a float *mix*, per the kept camera model — not a bool), the `shading`
triple (reflectiveness, gloss, shadow), four clip-plane slots,
`anti_alias_width`, `joint_type`, `flat_stroke`, `scale_stroke_with_zoom`,
`stroke_behind`, `depth_test`, and the C-7 `use_winding_fill` no-op.
