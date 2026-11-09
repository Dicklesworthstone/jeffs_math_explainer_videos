# G0-2 — The renderer look study (fm-k77)

**Status:** Ratified, 2026-07-25. **Verdict: the 3b1b look is KEPT, and every
constant that defines it is now a measured number rather than a remembered
one.** Six of §20.1 spike 2's decisions are fixed below with their evidence.
W5's completed interior colour field now makes the last comparison panel
reviewable; §6 records the G1 PASS verdict (`different-but-fine`,
Behavior-Noted in BN-06 fill) under ADR-0018.

**Why this spike exists (D-04, R2).** The Reference's look constants are the
aesthetic DNA of 3Blue1Brown and are kept deliberately — but they were tuned
for a GPU pipeline full of workarounds: signed-alpha winding fills, a ×0.95/
×1.06 composite round-trip, ≤32-segment polyline ribbons standing in for
strokes. Porting a constant onto honest mathematics changes what it *does*.
Only a side-by-side closes that loop, and only before W5 scales, because
fm-5oi (analytic fill), fm-oac (strokes) and fm-6cf (the cubic→quad converter)
all consume the numbers below.

**What ran, and where.** Two independent lines of evidence, deliberately kept
separate so they can disagree:

1. **The analytic half** — the Reference's own GLSL and Python at the pin
   `3b1b/manim @ 6199a00d`, read as source. Every constant quoted below carries
   its `file:line`.
2. **The empirical half** — measurements of the one-time Reference captures
   (fm-o3i, `gallery/reference_captures/`, 1920×1080, Mesa llvmpipe under Xvfb,
   MSAA off). The measurement scripts are `scripts/g0_2_measure.py`.

Where they agree, the number is settled. Where they disagreed, the disagreement
was the finding — see L2 and L5.

**The rendered evidence.** `docs/g0/g0-2-renders/` holds all six panels at the
capture resolution, in the *same pixel coordinates* as the Reference stills:
`gradient_fills` from production Lumen, three settled panels from the compact
analytic calibration fixture, `lighting_3d` from Lumen's integrated 3D
path, and `text_sample` from Scribe (fmn-text over the bundled faces) through
production Lumen. They are produced by `g0_2_look` in `spikes/g0-8-accelerator`. The
Reference stills stay private (§15.3); ours are our own primitives and are
committed.

---

## 1. The verdict, in one paragraph

Nothing about the 3b1b look needs to change, and three things about the
Reference's *mechanism* should. The look constants survive intact: the AA band
is 1.5 px of Hermite smoothstep (measured: a band of 1.56 px fitted to within
RMS 0.0033, under one 8-bit level, against a declared 1.5), the colour ramp is
`sqrt(lerp(c², c², α))` exactly, `average_color` is exactly RMS, the lighting
is `mix(colour, white, reflectiveness·max(n·l,0) + gloss·e^{-3(1-r·v)²})` with a
`mix(·, black, shadow·max(-n·l,0))` branch and `shading = (0.3, 0.2, 0.4)` on
surfaces, and `GlowDot` falls off as exactly `(1-r/R)²`. What does not survive
is three pieces of GPU scaffolding, each of which our mathematics simply does
not have: the fill's signed-alpha winding trick and its ×0.95/×1.06 round-trip
(which overshoots its own inverse by 0.70 %), the fill's 2×2 box supersampling
(which quantizes every fill edge to **five** coverage levels — measured, L3),
and the ribbon stroke's joint machinery (whose `bevel`/`miter` names are
inverted relative to their geometry, L6). Replacing those three with analytic
coverage, a continuous edge, and a distance-field join is where "the same feel,
cleaner" is actually earned, and the side-by-side shows all three.

---

## 2. The decisions

Every row is binding on the beads named. Constants are given in the units the
implementation will use.

| # | Decision | Value | Basis |
|---|---|---|---|
| (b) | **AA profile** | `smoothstep(0.5, -0.5, d/aaw)`, i.e. `t=clamp(0.5-d/aaw,0,1); t²(3-2t)` | source + measurement agree (L1) |
| (b) | **AA band width** | `anti_alias_width = 1.5` **pixels**, converted by `aaw = 1.5 × pixel_size` | `vectorized_mobject.py:96`; measured 1.56 px (L1) |
| (a) | **Default fill AA** | analytic coverage, **no supersampling** | strictly finer than the Reference's 5 levels (L3) |
| (a) | **Escalation trigger** | per-tile, when a pixel's coverage cannot be evaluated exactly — see §4 | deferred to fm-gmr, criterion fixed here |
| (c) | **Join model** | round, from the distance field | L6; §10.3 |
| (c) | **`auto` crossover** | `miter_factor = smoothstep(-0.8, -0.9, cos θ)` | `stroke/geom.glsl:36,116-118` |
| (c) | **Effective miter limit** | **3.1623** half-widths (`√(1+3²)` at the crossover) | derived from the above (L6) |
| (d) | **Colour interpolation** | `sqrt((1-α)·c₁² + α·c₂²)`, component-wise, on non-linearized [0,1] RGB | exact match, 0.000000 error (L4) |
| (d) | **`average_color`** | per-channel RMS, `sqrt(mean(c²))` | exact match (L4) |
| (d) | **Compositing space** | linear light | §6.3; the kept ramp is a γ=2 approximation to it, max error 9.4/255 (L4) |
| (e) | **Lighting** | `add_light` verbatim — see §3 | source + sphere measurement agree to 0.4 % (L5) |
| (e) | **`shading` defaults** | Mobject `(0,0,0)`; **Surface `(0.3, 0.2, 0.4)`** | `mobject.py:83`, `surface.py:47` |
| (e) | **Light position** | `(-10, 10, 10)` | `camera.py:42` |
| (e) | **`dark_shift`** | **not** part of the lighting model — see L5 | `textured_surface/frag.glsl:16` |
| — | **Glow falloff** | `alpha *= (1 - r/R)^glow_factor`, `glow_factor = 2.0` for `GlowDot` | source + measured exponent 1.986 (L7) |
| — | **Glow AA band** | `anti_alias_width = 2.0` px for `DotCloud`/`GlowDot` | `dot_cloud.py:42` |
| (f) | **cubic→quad tolerance** | **0.1 px** default, expressed as a scene-unit tolerance at render scale | 77× tighter than the Reference's worst case (L8) |

Two conversions everything above depends on, derived once (`camera.py:176-177`):
at 1920×1080 with the default frame, `pixel_size = FRAME_WIDTH/1920 =
14.2222…/1920 = 1/135`, so **135 px per scene unit**, and stroke width converts
at `STROKE_WIDTH_CONVERSION = 0.01` scene units per width unit
(`stroke/vert.glsl:21`) = **1.35 px per width unit**. `DEFAULT_STROKE_WIDTH =
4.0` is therefore 5.4 px.

---

## 3. The lighting model, kept verbatim

From `manimlib/shaders/inserts/finalize_color.glsl:16-46`, with `shading =
(reflectiveness, gloss, shadow)`:

```
if shading == (0,0,0):            return colour        # the Mobject default
bright = max(dot(to_light, n), 0) * reflectiveness
      +  gloss * exp(-3 * (1 - dot(reflect(-to_light, n), to_camera))^2)
rgb    = mix(rgb, WHITE, bright)
if dot(to_light, n) < 0:
    rgb = mix(rgb, BLACK, max(-dot(to_light, n), 0) * shadow)
```

Three properties worth carrying into the port explicitly, because each is a
place a "cleaner" reimplementation would drift:

- The `-3` in the specular exponent is **hard-coded**, not a uniform.
- `bright` is **not clamped**; `reflectiveness + gloss > 1` overshoots past
  white by construction.
- Brightening lerps toward white and darkening toward black — neither is a
  multiply, so the model is not energy-conserving and is not meant to be. It
  is a *look*, and D-04 keeps it.

Surfaces are lit **per-vertex** and Gouraud-interpolated (`surface/vert.glsl:15-19`),
not per-fragment. That is a deliberate Reference choice, and it is the one
place where matching the look and improving the mathematics may conflict; §12's
3D work owns the call, and this note does not pre-empt it.

---

## 4. The one decision measurement cannot fix yet

**(a), the tile-escalation thresholds.** §10.4 wants interiors as vectorized
spans with supersampling only at complex edges. The *defaults* are decided
above — analytic coverage, no supersampling — because analytic coverage is
exact for an ordinary edge and strictly finer than what the Reference produces
(L3). What cannot be fixed by measuring the Reference is the *threshold at
which a tile escalates*, because the Reference has no such concept: it
supersamples every fill everywhere, unconditionally.

The criterion is fixed here: **a tile escalates when analytic coverage stops
being exact within a pixel** — that is, when more than one boundary crosses a
single pixel cell (thin features narrower than a pixel, cusps, near-tangential
crossings, dense glyph stems). The per-piece monotone split makes "how many
boundary crossings land in this cell" available during the native analytic
pass; no geometry walk or supersampled canvas is retained after the cell is
resolved.

**fm-gmr's measured thresholds are now binding:** two independent boundary
crossings select a 2×2 fused resolve; four or more select 4×4. A partial fill
on the general path records crossings during the monotone-piece coverage walk;
an admitted primitive hint whose boundary falls between all six interior probes
is conservatively treated as two crossings. A centre-missed stroke probes the
four 2× positions. G0-2's join study found that a stroke narrower than roughly
two AA bands is wholly governed by the analytic smoothstep, so such a stroke
does not contribute complexity; eligible overlapping strokes escalate when
their independent contributions reach two. The 2/4 crossing thresholds and
two-band stroke threshold are geometric facts, not autotuned values, and
therefore do not depend on machine load or thread count.

On the W5 corpus (12,544 native cells), adaptive classified 29 cells at 2× and
34 at 4×: 13,204 native-equivalent cell-sample units including the classifier
pass versus 200,704 for full-frame forced 4×, a **93.42% reduction in
coverage-grid work** (not a wall-time claim). Against forced 4×, the measured
linear-channel error is 0.219482421875 maximum and
0.008526512734628627 RMS; the version-1 blocking budgets are 0.23 and 0.009.
The escalated fill route keeps the same exact coverage kernel on subcells
(curve-area integral or admitted primitive hint), so where native and sampled
coverage are both exact their resolved areas agree.
`certified` normalizes adaptive, forced 2× and forced 4× requests to the
canonical analytic path and is byte-identical under all three.

---
## 5. Findings

Numbered L1…L8 for the look study. (The F-series belongs to the accelerator
spike, G0-8/G0-8b; the two do not share a sequence.)

### L1 — The AA band is 1.5 px of smoothstep, and the captured pixels say so

`quadratic_bezier/stroke/frag.glsl:14-15` computes
`smoothstep(0.5, -0.5, |d|/aaw - hw/aaw)`, and `stroke/geom.glsl:144` sets
`aaw = max(anti_alias_width * pixel_size, 1e-8)` with the VMobject default
`anti_alias_width = 1.5` (`vectorized_mobject.py:96`, whose preceding comment
reads `# Measured in pixel widths`). The ribbon is emitted `width + aaw` wide
(`geom.glsl:149-155`), i.e. half a band of bleed on each side, so the transition
is exactly one `aaw` centred on the geometric edge.

Measuring it took two attempts, and the first was wrong in an instructive way.
Per-scanline "10–90 % ramp" on the square's edges gave **1.07 px** on the
vertical edge and **1.59 px** on the horizontal one — an anisotropy the shader
cannot produce, since pixels are square here (`pixel_size` is one number). The
cause: an axis-aligned edge lands at exactly one sub-pixel phase, so each
scanline shows the same one or two intermediate samples and the "ramp" measured
is the phase, not the profile.

A circular boundary sweeps every phase. Fitting the captured circle's edge by
least squares (720 contour points, residual median 0.041 px) and binning ~7,800
near-boundary pixels by exact signed distance reconstructs the profile at
sub-pixel resolution:

| d (px) | −0.875 | −0.625 | −0.375 | −0.125 | +0.125 | +0.375 | +0.625 |
|---|---|---|---|---|---|---|---|
| coverage | 0.9776 | 0.8525 | 0.6444 | 0.4109 | 0.1831 | 0.0391 | 0.0004 |

A two-parameter fit gives **band = 1.560 px, centre offset −0.220 px, RMS
0.0033** — under one 8-bit level across the whole profile. The 4 % excess over
the declared 1.5 sits inside the capture's 8-bit quantization and the centring
uncertainty. **Decision: keep 1.5 px and keep smoothstep.** The measured 10–90 %
width is 0.99 px; a implementation that reports much more or much less than
that has drifted.

`samples: int = 0` (`camera.py:46`) — MSAA is off, so this is pure shader
coverage and the comparison is clean.

### L2 — The captures disagreed with the shader until the geometry was right

Recorded because it is the methodological point of this note. Two of the four
"measurements" in the first pass were artefacts of sampling the wrong pixels:
the glow profile ran into the neighbouring dot (giving a non-monotone falloff
with the 50 % crossing *outside* the 25 % crossing), and the gradient sample
fell in the gap between the square and the circle (returning the background
colour as the "midpoint"). Both looked like plausible numbers. The rule that
caught them: a measurement that contradicts the source is a bug in the
measurement until proven otherwise — and the source was available the whole
time. The colour question was then moved off the pixels entirely (L4), which is
why it is the sharpest result here.

### L3 — The Reference's fill AA has five coverage levels; ours is continuous

`quadratic_bezier/fill/frag.glsl` contains **no AA term at all** — the interior
test is a hard `discard` (`:41`). Fill antialiasing comes from supersampling:
`shader_wrapper.py:447-452` allocates a texture at `(2*w, 2*h)` and resolves it
1:1, i.e. a 2×2 box downsample of a binary test. That predicts exactly five
coverage levels per pixel: {0, ¼, ½, ¾, 1}.

Measured on a bare fill edge (a filled circle with `stroke_width=0`, so nothing
covers the fill boundary), binning by signed distance: **4 distinct intermediate
levels, at 0.2471, 0.4981, 0.7452, 0.7567** — 0.25/0.50/0.75, the fourth being
an 8-bit rounding twin of the third. Against the stroke's continuous smoothstep
(L1, RMS 0.0033) the contrast is stark and it is not a matter of taste: a fill
edge in the Reference carries **2.3 bits** of coverage information.

**This is the concrete "at-least-as-good" claim for fills.** §10.2's analytic
coverage is continuous, so it is strictly finer on every edge, and it needs no
4× oversampled f16 render target to be so. It is also why decision (a) defaults
to no supersampling: we would be adding cost to *lose* precision.

### L4 — The colour model is exactly what §6.3 kept, and it is a γ=2 stand-in for linear light

Interrogated directly rather than inferred, since `manimlib/utils/color.py` is
pure Python. Over BLUE_E→YELLOW at α ∈ {0, ¼, ½, ¾, 1}, `interpolate_color`
matches `sqrt((1-α)c₁² + αc₂²)` to **0.000000** and diverges from a naive lerp
by up to **0.177**. `average_color` matches per-channel RMS exactly. Both are
computed on `hex2rgb` values — a plain ÷255, with **no sRGB decode**
(`color.py:22-27`), so the squaring is a fixed gamma-2 approximation.

The question §6.3 actually asks is whether that kept form composites correctly
in linear light. Answer, across the whole ramp:

| α | 0.1 | 0.2 | 0.3 | 0.5 | 0.7 | 0.9 |
|---|---|---|---|---|---|---|
| max channel err vs true linear-light lerp | 0.0333 | 0.0370 | 0.0356 | 0.0278 | 0.0174 | 0.0059 |

Worst case **0.037 = 9.4/255**, at α ≈ 0.2, and exact at both endpoints.

**Decision: keep the form, composite in linear light.** γ=2 is a deliberate,
bounded approximation of the sRGB transfer function, the endpoints are exact,
and the ≤9.4/255 mid-ramp difference *is* the 3b1b gradient aesthetic §6.3
commits to preserving. This is BN-04's evidence.

Two Reference inconsistencies to not replicate: `get_colormap_from_colors`
(`color.py:145-163`) and the GLSL `float_to_color` both interpolate **linearly**,
with no squaring — so the Reference already contradicts itself between its
gradient path and its colormap path.

### L5 — The lighting model is confirmed to 0.4 % by the sphere, and `dark_shift` is not part of it

`add_light` (§3) predicts that brightening is `mix(rgb, white, t)` and darkening
`mix(rgb, black, s)`. Those are strong claims: a *single* t must explain all
three channels. From the captured sphere (BLUE_E, `Surface` shading
`(0.3, 0.2, 0.4)`):

- brightest pixel ⇒ t = **(0.3700, 0.3696, 0.3675)**, spread **0.0025**
- darkest pixel ⇒ s = **(0.3929, 0.4017, 0.3986)**, spread **0.0089**

Three independent channels agreeing to under 1 % is the model, confirmed. And
`s/shadow = (0.982, 1.004, 0.996) ≈ 1`, i.e. the darkest point has `n·l = -1`
exactly as a sphere must — which pins **shadow = 0.4** to within 0.4 % from the
pixels alone.

**`dark_shift` is not in the lighting model.** The bead's parameter list names
"dark_shift 0.2" alongside reflectiveness/gloss/shadow. At this pin the only
`dark_shift` in the tree is `textured_surface/frag.glsl:16`, where it is the
half-width of a `smoothstep(-0.2, +0.2, dp)` crossfade between a **light and a
dark texture** on a textured surface. It is a two-texture blend, not a shading
term, and `finalize_color` never sees it. Recorded so the port does not
manufacture a lighting parameter that never existed.

### L6 — `bevel` and `miter` are named backwards, and `auto` is a smoothstep with an implied miter limit of 3.16

`stroke/geom.glsl:110-122`:

```glsl
if      (joint_type == BEVEL_JOINT) miter_factor = 0.0;
else if (joint_type == MITER_JOINT) miter_factor = 1.0;
else { mcat1 = -0.8; mcat2 = mix(mcat1, -1.0, 0.5);      // = -0.9
       miter_factor = smoothstep(mcat1, mcat2, cos_angle); }
float shift = (cos_angle + mix(-1, 1, miter_factor)) / sin_angle;
```

`miter_factor = 0` gives `shift = (cosθ-1)/sinθ = -tan(θ/2)` — the **pointed**
corner. `miter_factor = 1` gives `shift = (cosθ+1)/sinθ = +cot(θ/2)` — the
**blunt** one. So the constant named `BEVEL_JOINT` (Python `"bevel"`, code 2)
produces the mitre, and `MITER_JOINT` (`"miter"`, code 3) produces the blunt
cut. **The numeric codes are authoritative; the names are not.** Our
`JointType` already stores the codes (`fmn-mobject/src/uniforms.rs`), which is
the right choice — but any code that reasons from the *names* will be wrong.

The captures corroborate it independently. At the calibration zig-zag's vertex
the turn is θ = −81.15°, so `cos θ = 0.154 > -0.8` and `auto` must equal
`bevel`. Measured apex widths: auto 4 px, bevel 6 px, **miter 9 px** — auto and
bevel alike to within the crudeness of the metric, miter visibly different and
blunter. The side-by-side shows the same thing by eye, along with `no_joint`'s
notch.

Because `auto` crosses over at `cos θ = -0.8` (θ = 143.13°), its worst-case
extension is `|shift| = tan(71.565°) = 3.0`, i.e. `√(1+9) = **3.1623**`
half-widths. That is the Reference's **de facto miter limit**, and it is the
number §10.3 should adopt — there is no `miter_limit` constant to copy.

**Our stroke has one join.** A true curve-distance stroke produces a round join
from the distance field alone; the four Reference types collapse to one. The
committed `fmn-joints-and-caps.png` renders four identical rows on purpose —
that *is* the finding, stated visually. Whether `miter`/`bevel` are then offered
as explicit overrides is fm-oac's call; the look study's position is that the
round join is correct, is what the mathematics already gives, and shows no
notch or gap at any of the vertices where the Reference does.

### L7 — Glow is `(1-r/R)²`, from both directions

`true_dot/frag.glsl:26` is `frag_color.a *= pow(1 - r, glow_factor)` with
`glow_factor = 2.0` for `GlowDot` (`dot_cloud.py:171`; `DotCloud`'s own default
is 0.0). A plain power law in normalized radius, reaching exactly zero at the
rim — not a Gaussian.

Measured independently by averaging 16 radial directions from the centre dot
(staying clear of its neighbours, cf. L2) and fitting `I/I₀ = (1-r/R)^k`:
**k = 1.986**, residual RMS 0.0156. Model comparison over the whole profile:
`(1-r/R)²` max error **0.0123**; `(1-r/R)^1.5` 0.105; `exp(-3(r/R)²)` 0.271.
The quadratic wins by an order of magnitude.

### L8 — The Reference's cubic→quad conversion is off by up to 7.7 px, which is what makes (f) easy

The Reference converts every cubic with a **fixed two-quadratic** approximation
and no error bound (`bezier.py:343`). Maximum deviation, measured against a
4001-sample cubic, converted to pixels at 135 px/unit:

| case | n quads | max deviation |
|---|---|---|
| quarter-circle-ish | 2 | **0.39 px** |
| gentle S | 2 | **4.18 px** |
| half-circle-ish | 2 | **5.78 px** |
| near-cusp | 2 | **7.32 px** |
| strong C | 2 | **7.71 px** |

A quarter circle is fine; a general cubic is not. **7.7 px is over five times
the entire AA band** — a deviation nobody could call antialiasing.

**Decision: 0.1 px default tolerance** for fm-6cf's error-bounded converter.
The justification is the AA band: coverage changes over 1.5 px, so an error of
0.1 px perturbs it by ~7 % of one band edge — invisible — while being **77×
tighter than the Reference's worst measured case**. "Curve fidelity visibly
exceeds the Reference's" is thereby a measured statement, not an aspiration.
Expressing the tolerance in pixels (converted to scene units at render scale)
rather than in scene units is deliberate: it is a visibility criterion, so it
must scale with resolution.

---

## 6. The side-by-side

`docs/g0/g0-2-renders/` vs `gallery/reference_captures/`, same pixel grid.
Verdicts in §16.3's vocabulary:

| panel | verdict | what to look at |
|---|---|---|
| `self_intersections` | **at-least-as-good** | The Reference's star core is visibly **lighter** than its limbs: winding number 2 composites the fill twice through the signed-alpha trick. Ours renders one uniform colour, which is what the nonzero rule means. |
| `joints_and_caps` | **different-but-fine** (Behavior-Noted) | Four different corners become one round join; no notch at any vertex, and round caps at the ends. L6. |
| `glow` | **at-least-as-good** | Falloff character matches (L7). Registration is close, not exact — our disc reads slightly larger, since `GlowDot`'s `radius` parameter and the visible extent are not the same quantity. |
| `gradient_fills` | **different-but-fine (Behavior-Noted)** | This now runs through production Lumen's §10.2 field. The Reference's hard **diagonal seam** comes from per-vertex colour interpolated across its triangle fan; ours is the smooth, true-arclength boundary ramp extended by mean value coordinates. Gradient direction, opacity, stroke ramp, and capture registration agree. The smoother, subdivision-stable field is the deliberate BN-06 behavior fm-5oi landed. G1 PASS 2026-08-20: visual side-by-side by `GreenPeak` under ADR-0018; no regression candidate. |
| `lighting_3d` | **at-least-as-good** | The integrated Lumen path uses the exact capture inputs: `Sphere(radius=2)`, the Reference's `(101, 51)` UV grid and parameterization, BLUE_E, and `frame.reorient(20, 70)`. Silhouette, light direction, and retained Gouraud shading coincide closely. Whole-frame normalized RMSE is **0.00212857** and normalized SSIM distortion is **0.000145104**; these are smoke alarms, not gates. |
| `text_sample` | **different-but-fine** (Behavior-Noted; ratified 2026-08-24 by delegated blind review, §9) | The capture scene rebuilt on Scribe: `Text(…, font_size=60)` over the italic `Text(…, font_size=32)`, body `next_to(title, DOWN, buff=0.5)`, group centred, laid out by fmn-text over the bundled Computer Modern regular + italic faces and rendered by production Lumen. The face itself is the deliberate D-08/BN-05 divergence — the capture went through Pango to the host's default (a mono face on the capture box); ours is the sovereign bundled default, identical on every machine. What must correspond, and does: centring and `next_to` spacing, the 60/32 size ratio, the regular/italic contrast (true italic faces, not a shear), the em dash, and the AA edge character. Whole-frame normalized RMSE is **0.08914188** — it sees the intentional font change, so it is registered as a smoke alarm only. |
| `math_formula` | **different-but-fine** (Behavior-Noted BN-05; ratified 2026-08-24 by delegated blind review, §9) | The capture scene rebuilt on Scribe: `Tex(r"e^{i\pi} + 1 = 0", font_size=96)` centred, laid out by fmd-math over the bundled Computer Modern and rendered by production Lumen — the capture went through real LaTeX + dvisvgm on the capture box, so this is the native-typesetting claim tested against the genuine article. What must correspond, and does: the string, the face (Computer Modern both sides), the display scale (ink heights 123 vs 125 px), and exact bounding-box centring (both ink centres sit at (960, 540)). Where BN-05's metrics-differ-from-LaTeX divergence shows: fmd-math's inter-atom spacing is tighter — total advance 585 px vs 625 px (6.4 % narrower) — so glyph registration drifts up to ~20 px toward the ends while the centre holds. Smoke alarms, not gates: whole-frame normalized RMSE **0.05328678**, global SSIM **0.395720620**, symmetric chamfer edge distance **5.847 px** — the same regime as the accepted `text_sample` row (0.08914188 / 0.345615765 / 6.404 px): thin shifted strokes on a dark frame, the metrics seeing the registration drift, not the style. |

The production gradient panel is reproduced with:

```bash
cargo run --release --locked \
  --manifest-path spikes/g0-8-accelerator/Cargo.toml \
  --bin g0_2_look -- docs/g0/g0-2-renders --gradient-only
```

Its Rgba16F framebuffer SHA-256 is
`8c6d52060c318e948f6d08bf1eb0f45ba566e7d317971e3b95a542fb51fb3de9`;
the committed canonical PNG SHA-256 is
`5c49c5224b36c497eab0d636623b7a602dde2f39f1e05dbb4e441d71c40f1345`.
The registered whole-frame normalized RMSE against the private capture is
**0.0483394**. That metric sees the intentionally different interior field and
is a smoke alarm, not a pass threshold.

The lighting panel is reproduced with:

```bash
cargo run --release \
  --manifest-path spikes/g0-8-accelerator/Cargo.toml \
  --bin g0_2_look -- docs/g0/g0-2-renders --lighting-only
```

Its Rgba16F framebuffer SHA-256 is
`b36e4b44bf35c23e78b7ae3fc251c510b2bf0048e8e3b586f4fd14bc278c4f44`.

The text panel (fm-gfn) is reproduced with:

```bash
cargo run --release \
  --manifest-path spikes/g0-8-accelerator/Cargo.toml \
  --bin g0_2_look -- docs/g0/g0-2-renders --text-only
```

Its Rgba16F framebuffer SHA-256 is
`30cd1e3b370cff79c758470bed88422a3b85353fe71eace4d895f45d82469897`;
the committed canonical PNG SHA-256 is
`8f175ae968fcc0aeaa30e1595b492b9d24d5d59d287362efb2346fc7c0fec143`.

The math panel (fm-gjl7) is reproduced with:

```bash
cargo run --release \
  --manifest-path spikes/g0-8-accelerator/Cargo.toml \
  --bin g0_2_look -- docs/g0/g0-2-renders --math-only
```

Its committed canonical PNG SHA-256 is
`39f17d2f5775da04d1f64e4f661081d77a5f135d41a7c14ad3039c8af12be4e0`.
The Reference side was captured in supplement mode on 2026-08-23
(`gallery/reference_captures/math_formula.png`, SHA-256
`c2eb1124091572f076c6badb10ba8716aff7d84a6f05751df5b0b7534d20807c`,
recorded in `PROVENANCE.json` `supplements`), because the panel postdates
the 2026-07-25 capture set; existing captures are never re-made.

 [.gitignore#5AD6]

One honest limit on this gallery, now that all seven captured scenes are
rendered: **registration is close, not pixel-exact** for the four analytic
panels: they are placed from bounding boxes measured off the captures, which
include stroke extent, so shapes can sit a few pixels off. This is an aesthetic
comparison, never a pixel gate (D-16), and a few pixels of placement does not
affect any constant decided above.

---

## 7. Decisions recorded

- **D-04 stands, with evidence.** Every kept constant is now measured, not
  assumed: 1.5 px smoothstep, `sqrt`-of-squares colour, RMS averaging,
  `(0.3, 0.2, 0.4)` surface shading with the light at `(-10, 10, 10)`,
  `(1-r/R)²` glow.
- **Three mechanisms are replaced, not ported**: the signed-alpha winding fill
  and its ×0.95/×1.06 round-trip, the fill's 2×2 box supersampling, and the
  ribbon joint machinery. Each is a §16.3 "at-least-as-good" claim with a
  measurement behind it (L3, L6) or a visible artefact in the side-by-side.
- **`dark_shift` is struck** from the lighting parameter list (L5). It belongs
  to the textured-surface shader and is not a shading term.
- **The `bevel`/`miter` name inversion is recorded** (L6). Ports must key on
  the numeric codes.
- **The effective miter limit is 3.1623 half-widths** (L6), derived rather than
  copied, since the Reference has no such constant.
- **fm-gmr owns the escalation threshold** (§4); this note fixes the criterion
  and the default, not the number.

## 8. Follow-ups

- **fm-5oi** (closed) — analytic fill consumed (a) and L3 and landed §10.2's
  mean-value-coordinate interior colour field. Its production
  `gradient_fills` panel is the G1 PASS `different-but-fine` verdict above.
- **fm-oac** — strokes. Consumes (b), (c), L6, and the 3.1623 miter limit.
- **fm-6cf** — the cubic→quad converter. Consumes (f)'s 0.1 px default and L8's
  measured baseline.
- **fm-gmr** — adaptive AA. Owns the escalation threshold per §4.
- **BN-04** gains L4's error table as its evidence.
- **Appendix C** should gain the ×0.95/×1.06 round-trip: `1/0.95 = 1.05263`,
  but the Reference multiplies by `1.06`, a **0.70 % systematic overshoot** on
  every fill's colour *and* alpha (`shader_wrapper.py:489`). We inherit no such
  factor; the divergence is worth a Behavior Note so it is not mistaken for a
  colour bug.
- **A Reference defect worth reporting upstream, not replicating**:
  `get_scale_stroke_with_zoom` returns `uniforms["flat_stroke"] == 1.0` — the
  wrong uniform (`vectorized_mobject.py:433-438`).
- **The lighting is Gouraud, per-vertex** (§3). §12 decides whether to keep
  that or evaluate per-fragment; keeping the look may not require keeping the
  interpolation.

## 9. Delegated ratification (ADR-0018) — 2026-08-24

Both pending rows (`text_sample`, `math_formula`) are **ratified as
`different-but-fine`** by delegated fresh-eyes review under ADR-0018, with
the delegate's two concrete defect claims raised, pixel-verified, and
**refuted** — recorded here so the ratification is auditable, not a rubber
stamp.

**Protocol.** The delegate reviewed BLIND: both panels of each pair were
copied to neutral labels `X`/`Y` (assignment fixed by a recorded seeded draw,
`docs/g0/ratification-mapping-2026-08-24.json`, seed 20260824) before the delegate saw anything, and the
prompt named neither pipeline's provenance. Delegate: the Antigravity CLI
agent (`agy --print`, Gemini 3.x Pro-class), 2026-08-24, host
`sensedemobox`. Evidence viewed: the actual 1920×1080 panels —
`text_sample_{X,Y}.png` and `math_formula_{X,Y}.png` under the review
scratch — byte-identical to the committed fixtures (SHA-256s in
`docs/g0/ratification-mapping-2026-08-24.json`; fmn text `8f175ae9…`, Reference text `e65ffc3b…`, fmn math
`39f17d2f…`, Reference math `c2eb1124…`). Grok Build was tried first as the
G1-precedent delegate and was unavailable (usage balance exhausted, HTTP
402); the Gemini CLI was deprecated (IneligibleTierError); Antigravity ran.

**Blind findings, decoded against the mapping.** The delegate ranked the
Reference capture first on BOTH pairs — a taste preference that is itself
the `different-but-fine` content, not a regression finding — and raised two
specific defect claims:

1. *"fmn math panel clips the top of the exponent `i`, removing its dot."*
   **Refuted at pixel level.** The fmn dot is a whole glyph: rows above the
   binarized ink top carry a smooth anti-aliasing ramp (`279 462 447 225` at
   row 477, rising to full ink at 478). The dot core reads 7 px against the
   capture's 9 px at the same binarization threshold — somewhat smaller than
   the render's overall ~2 % scale difference alone would predict, as
   expected when thresholding erodes a small feature's anti-aliased edge on
   both sides; the decisive point is the ramp itself, which a hard clip
   cannot leave behind.
2. *"The Reference capture clips descenders flat (p, y, comma); the fmn text
   shows wide math-mode-style letter spacing."* **Both halves refuted.**
   Full-resolution crops of the capture's subtitle band show every descender
   intact (the commas, the `p` of "kept"); and the fmn subtitle is normal
   Computer Modern *italic* — cohesive cursive letterforms with ordinary word
   spacing, not isolated letters. The face difference (Consolas-oblique
   capture vs bundled CM italic) is the documented BN-05 divergence, and the
   delegate's preference for the capture's face on pair 1 is the same
   recorded taste call the drafted verdict already carries.

**Verdicts.** `text_sample`: **different-but-fine (BN-05), ratified** —
layout, size ratio, true-italic contrast, em dash, and AA character
correspond; the face is the deliberate sovereign-font divergence and the
blind delegate's preference for the capture face is noted, not overruled.
`math_formula`: **different-but-fine (BN-05), ratified** — same face both
sides, identical ink centring, the 6.4 % tighter inter-atom advance is the
recorded metrics divergence, and the delegate's sole defect claim dissolved
under magnification. Neither panel is a regression; no correction bead is
required. Reviewer-of-record for this ratification: the in-session swarm
(ox-alpha session, 2026-08-24) executing ADR-0018's machinery; the
program owner retains the standing right to re-open any ratified row.
