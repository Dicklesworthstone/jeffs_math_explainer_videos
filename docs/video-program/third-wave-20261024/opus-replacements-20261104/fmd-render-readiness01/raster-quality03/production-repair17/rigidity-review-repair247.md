# Rigidity review repair 247 (cc1): candidate for review241 L1 and L3

**Status:** candidate only, for root to decide on and integrate.
- Nothing has been compiled, tested, built or rendered, and nothing is accepted.
- I made no canonical, `draw.rs`, TODO or Beads edits, and no commit.
- **M1** (plates not fading) is not touched here. It belongs to CC2's fade investigation 246.
- Narration (ElevenLabs Frederick) is unchanged.

## Files

| File | sha256 | Bytes |
|---|---|---|
| Base: `production-source-preserved214/workspace/explainer_corpus/complex_analysis_visual/src/chapters/rigidity.rs` (production19, source host csd `production-repair19/workspace`, per `receipt.json`) | `c577e2c02049aa468f2d5c3ad935d74004b5e321901a2cd73c1f1f54c1b0187e` | 29,556 |
| Canonical `explainer_corpus/complex_analysis_visual/src/chapters/rigidity.rs` (today) | same `c577e2c0…`, identical to the base | 29,556 |
| **Candidate** `rigidity-review-repair247.rs` | `0bcfbc7d7962b2ab8fa2d27895fce104ab6cb71ce5aac26fdd0b97dca04a3df0` | 32,888 |
| Exact diff below (base → candidate) | `f2252809690c1ceb5738af8ce275eebef3573d68f64cbdca1833b4600b73e875` | 123 lines; 80 insertions, 7 deletions |

The candidate touches one file. It adds no on-screen string, so the TEXT registry in `mod.rs`,
the facts, `oracle.py`, `script.tsv` and the cues are all unaffected.

## L1: the plate foot under the "precisely" card (88.0–103.6 s)

**Cause.**
- The 10_5 lower third is two lines. Its geometry comes from `explainer_kit::props::lower_third`:
  height 0.3 + 2 × 0.36 = 1.02, foot 0.2 above the frame's foot (y = −4), stroke 1.5.
- So its top is at y = −2.78, and its stroke's outer edge is at −2.7725 (4K row 1828.6).
- The centred sin(1/z) plate's foot is at −2.8, and its 2.0 border reaches −2.81 (row 1838.7).
- The predicted overlap is 0.0375 units, or 10.1 px at 4K. Review 241 measured 8–10 px in three
  columns, so the model matches the render.

**Change.** In `sin_recip_plane()`, the centre moves from `v(-2.8, 0.0)` to `v(-2.8, 0.1)`. Only
the position changes; the width (5.6) and the window (±0.42 both ways) stay the same.

| | Plate foot (border edge) | Gap to the band's edge | Title centre (top + 0.3) |
|---|---|---|---|
| production19 | −2.81 (row 1838.7) | **−0.0375 (−10.1 px at 4K)** | 3.10 |
| candidate 247 | −2.71 (row 1811.7) | **+0.0625 (+16.9 px at 4K, 8.4 px at 1080p)** | 3.20 (frame top 4.0) |

Why a move rather than a resize:
- Every mark on the plate is placed with `sp.p(…)`: the 24 zeros and their backings, the "1/π"
  and "0" cards, the red cross, the axes, the axis names, the border and the title. A move
  shifts them all together, so the zeros, the cross clearance (tested by
  `inner > CROSS_ARM`, which compares x only) and the scale (6.667 scene units per unit) are
  unchanged.
- `picture_cells` depends only on the plate's width and height, so the plate's cell count and
  its `crisp` stroke widths are unchanged.
- 0.1 units is exactly 27 px at 4K, so the strip edges stay on whole pixels.
- The right-hand column (x ≈ 4.3) and the 10_6 sine plate are not moved. The sine plate's foot
  (−2.925) already clears the one-line names bar (top −3.14) by 0.198.

## L3: the gold limit point q on U's gold fill (13.0–16.7 s)

**Change.**
- q gets a dark `ground()` backing of radius `Q_DOT + 0.03` = 0.11 under its gold dot (0.08).
  This is the same r + 0.03 backing the 10_4 zeros already use (base lines 434–436) and the
  `zero_mark` style ("so it reads on any colour").
- To make this testable, q and its points are lifted out into `limit_point()` and
  `piling_points()`, with radius constants `PILE_DOT`, `Q_DOT` and `Q_BACKING`. The values are
  identical to production19's inline ones (0.6, −0.2; t = 0.9 · 0.62^k for k = 0..8;
  (1, 0.35); 0.06 / 0.08).

**Timeline unchanged.**
- Production19 made 10 separate 0.12 s fade-ins: 9 white dots, then q.
- The candidate makes 9 for the white dots, then one 0.12 s `fade_in_all` of `[backing, q]`.
  That is still 10 plays and 1.20 s.
- Both marks join `piling`, so they leave in the same 10_2 fade-out as before.
- Mobs are created in nearly the same order; only the backing is new, between the white dots
  and q.

**Contrast** (WCAG, from review 241's measured colours and the kit's `#0E1116` ground):
- before: q against the fill, 1.48:1;
- after: the backing against the fill 9.49:1, and q against its backing 14.02:1.

**What the backing covers.** It is faded in with q, so it sits on top of the white dots, just as
q already did. Distances are from q, in scene units:

| Point k | Distance | Shows past q's dot (production19) | Shows past the backing (candidate) |
|---|---|---|---|
| 0–3 | 0.954 … 0.227 | whole | whole (clear by ≥ 0.057) |
| 4 | 0.141 | whole | 0.091 of 0.12 (loses an inner bite) |
| 5 | 0.087 | 0.067 crescent | 0.037 crescent (10 px at 4K) |
| 6 | 0.054 | 0.034 crescent | 0.004 (effectively hidden) |
| 7, 8 | 0.034, 0.021 | ≤ 0.014 | hidden (already under q) |

The pile-up stays readable: five whole or nearly whole points, then a crescent, then the ringed
q. If root prefers to keep k = 6's sliver, `Q_BACKING = Q_DOT + 0.02` leaves 0.014 (about 4 px at
4K), but it then no longer matches the 10_4 zeros. I did not choose that option.

## Tests added (uncompiled)

1. **`the_points_pile_up_inside_u_at_a_marked_point_inside_u`.** This is the theorem's condition
   as drawn.
   - q and every piling point lie in U, at least one dot radius from U's edge; q's backing is
     also inside U.
   - The distances shrink by exactly 0.62 at each step, so the points pile up at q.
   - The last point reaches under q.
   - The backing leaves points 0–3 whole, and points 0–5 at least 0.03 visible.
   - My Python mirror (not delivered) gives: edges ≥ 0.81 (q 1.05), k = 3 clear by 0.167,
     k = 5 showing 0.037.
2. **`the_sin_recip_plate_stops_above_the_two_line_lower_third`.**
   - It wraps `EXACT` with the kit's public `animatic::wrap` at 92 characters and asserts two
     lines. The mirror gives 89 + 40 characters, and the render shows two lines.
   - It rebuilds the band's top from the kit's formula and asserts the plate foot clears it by
     more than 0.05 (13.5 px at 4K). The candidate clears it by 0.0625.
   - **Positive control:** with production19's centred plate this test fails (−0.0375).
   - It also asserts the title stays inside the frame (3.5 < 4.0).
   - The kit constants are mirrored in the test because `LOWER_THIRD_CHARS` and `line_h` are
     private. If the kit changes, the test must follow it.

The existing tests are unaffected: the cross clearance compares x only, and `contains` uses the
unchanged window.

## Checks run, and gaps

- `rustfmt 1.10.0-nightly --check` (edition 2024, `max_width = 120`) passes on the candidate and
  on the base. This checks formatting and parsing only, not type-checking.
- A Python mirror of all the geometry and contrast numbers above is in my scratch folder and not
  delivered.
- **Not done:** `cargo check`, `cargo test`, Clippy, render, and new stills.
- **Kit provenance gap:** `preserved214` has no `explainer_kit` files. I read the local
  `crates/explainer_kit`:
  - `props.rs` `73606e89…`, `plot.rs` `15e8c470…`, `animatic.rs` `631fbdfd…`;
  - the relevant code dates from 2026-10-10 and 2026-10-11, and `props.rs` was last changed
    2026-10-11;
  - the kit's formula predicts the rendered overlap to within about 2 px;
  - but I have not proven the production19 kit bytes are the same.
- `Frame2::bottom()`, `fmn::core::constants::FRAME_Y_RADIUS` (pin `a399d626`) and the public
  `explainer_kit::animatic::wrap` all exist in those sources. Only a compile would prove the
  candidate builds.

## For root, if adopted

1. Run `cmp` on canonical `rigidity.rs` against `c577e2c0…` first. If it has changed, apply the
   diff instead of copying.
2. Copy the candidate in, then run cargo test and strict Clippy for the crate.
3. Re-render chapter 10. Its timeline should be unchanged at 7,442 frames.
4. Check stills at:
   - 12.0–16.7 s: q's ring and the pile-up;
   - 62.3–104 s: the plate's top, its title and the "Im" name;
   - 88–103.6 s: the plate foot against the "precisely" card.
5. Every other cue, duration, wording and teaching step is unchanged.

Listening, fine sync, motion review and independent final AV acceptance all remain open.

## Exact diff (base production19 → candidate 247)

```diff
--- a/explainer_corpus/complex_analysis_visual/src/chapters/rigidity.rs
+++ b/explainer_corpus/complex_analysis_visual/src/chapters/rigidity.rs
@@ -104,6 +104,30 @@
     out
 }
 
+/// The point inside U where the points of 10_1 pile up.
+pub fn limit_point() -> Vec3 {
+    v(BLOB_CENTRE[0] + 0.6, BLOB_CENTRE[1] - 0.2)
+}
+
+/// The points that pile up at [`limit_point`] q: q + t(1, 0.35), with t = 0.9 · 0.62^k.
+pub fn piling_points() -> Vec<Vec3> {
+    let q = limit_point();
+    (0..9)
+        .map(|k| {
+            let t = 0.9 * 0.62f64.powi(k);
+            v(q[0] + t, q[1] + 0.35 * t)
+        })
+        .collect()
+}
+
+/// Radius of each piling dot.
+pub const PILE_DOT: f64 = 0.06;
+/// Radius of the gold dot at q.
+pub const Q_DOT: f64 = 0.08;
+/// Radius of q's dark backing: 0.03 wider than the dot, as for the zeros of 10_4. Without it
+/// the gold q sits on U's gold fill at 1.48:1.
+pub const Q_BACKING: f64 = Q_DOT + 0.03;
+
 /// A disk outline and its translucent fill.
 fn disk(stage: &mut Stage<'_>, at: Vec3, r: f64) -> fmn::Result<Mob> {
     let pts: Vec<Vec3> = (0..72)
@@ -146,8 +170,10 @@
 
 /// The window of sin(1/z) in 10_4–10_5.
 pub fn sin_recip_plane() -> Plane {
-    // centred, so that its foot stops above where a two-line lower third starts
-    Plane::new(v(-2.8, 0.0), 5.6, (-0.42, 0.42), (-0.42, 0.42))
+    // 0.1 above centre, so that its foot and frame stop above where 10_5's two-line lower third
+    // starts (centred, the foot ran 0.04 into that band); moved, not resized, so the zeros,
+    // cards and cross keep their places on the plate
+    Plane::new(v(-2.8, 0.1), 5.6, (-0.42, 0.42), (-0.42, 0.42))
 }
 
 /// The window of sine in 10_6.
@@ -212,17 +238,20 @@
             .add(Polygon::new(edge.clone()).style(flat(hero(), 0.18)).build());
         fade_in_all(stage, &[whole], 0.3)?;
         line.wait_sentence(stage, 3)?;
-        let q = v(BLOB_CENTRE[0] + 0.6, BLOB_CENTRE[1] - 0.2);
+        let q = limit_point();
         let mut piling = Vec::new();
-        for k in 0..9 {
-            let t = 0.9 * 0.62f64.powi(k);
-            piling.push(dot(stage, v(q[0] + t, q[1] + 0.35 * t), 0.06, Palette::ink()));
+        for p in piling_points() {
+            piling.push(dot(stage, p, PILE_DOT, Palette::ink()));
         }
-        piling.push(dot(stage, q, 0.08, hero()));
+        // q on a dark backing, as the zeros of 10_4 are, so the gold point reads on U's gold fill
+        let q_marks = [dot(stage, q, Q_BACKING, ground()), dot(stage, q, Q_DOT, hero())];
         let pile_says = kit.text(stage, PILING, 20.0, Palette::ink(), v(right, -0.6))?;
         for m in &piling {
             fade_in_all(stage, &[*m], 0.12)?;
         }
+        // both in the one 0.12 s step that q alone took, so the timeline is unchanged
+        fade_in_all(stage, &q_marks, 0.12)?;
+        piling.extend(q_marks);
         fade_in_all(stage, &[pile_says], 0.4)?;
         let search = names_now(&kit, stage, &["identity theorem"])?;
         wait_until(stage, line.end())?;
@@ -698,6 +727,50 @@
     }
 
     #[test]
+    fn the_points_pile_up_inside_u_at_a_marked_point_inside_u() {
+        let q = limit_point();
+        let pts = piling_points();
+        // the condition the picture shows: the points and the point they pile up at are in U
+        assert!(inside_u(q) && edge_distance(q) > Q_BACKING);
+        let d: Vec<f64> = pts.iter().map(|p| (p[0] - q[0]).hypot(p[1] - q[1])).collect();
+        for (k, &p) in pts.iter().enumerate() {
+            assert!(inside_u(p) && edge_distance(p) > PILE_DOT);
+            // each point is closer to q by the same ratio: they pile up at q and nowhere else
+            if k > 0 {
+                assert!((d[k] / d[k - 1] - 0.62).abs() < 1e-12);
+            }
+        }
+        assert!(d[8] < Q_DOT, "the last points reach under q");
+        // q's backing covers only points already at q: the first four stay whole, and the
+        // first six still show at least 0.03 past it
+        for (k, &dk) in d.iter().enumerate() {
+            if k < 4 {
+                assert!(dk - PILE_DOT > Q_BACKING);
+            }
+            if k < 6 {
+                assert!(dk + PILE_DOT - Q_BACKING > 0.03);
+            }
+        }
+    }
+
+    #[test]
+    fn the_sin_recip_plate_stops_above_the_two_line_lower_third() {
+        use fmn::core::constants::FRAME_Y_RADIUS;
+        // explainer_kit::props::lower_third: lines wrap at 92 characters; the band is 0.3
+        // plus 0.36 a line high, its foot 0.2 above the frame's, its stroke 1.5 wide. A plot
+        // frame's border strokes 2.0. A stroke w is w / 100 scene units wide.
+        let lines = explainer_kit::animatic::wrap(EXACT, 92, 3);
+        assert_eq!(lines.len(), 2, "10_5's precisely card is two lines");
+        let band_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * lines.len() as f64 + 0.015 / 2.0;
+        let sp = sin_recip_plane();
+        let foot = sp.frame.bottom() - 0.02 / 2.0;
+        // 0.05 is 13.5 px at 4K; the centred plate ran 0.0375 into the band
+        assert!(foot - band_top > 0.05, "plate foot {foot} against band top {band_top}");
+        // its title sits 0.3 above the top; the title stays well inside the frame
+        assert!(sp.frame.top() + 0.3 + 0.3 < FRAME_Y_RADIUS);
+    }
+
+    #[test]
     fn the_trig_values_drawn_are_the_facts_and_the_identity_holds_off_the_line() {
         let z = C::new(0.0, 3.0);
         let (s, c) = (z.sin(), z.cos());
```
