# gallery review repair 249: chapter 05, F2 of review240

Dispatch-0 (cc0), 2026-11-08. **Candidate source for root to integrate. Nothing has been applied, compiled, rendered or synthesised.** I wrote only this note and `gallery-review-repair249.rs`. Root owns integration, the build, the render, verification, TODO and Beads. I make no acceptance claim.

The narration is unchanged: ElevenLabs Frederick, same script, cues, takes and timing. No audio was heard or checked.

## Source

- **Base.** `production-source-preserved214/workspace/explainer_corpus/complex_analysis_visual/src/chapters/gallery.rs`, sha `7d1cf20fd25c8d9233f3c14426b0196fd4fd6673e4b25becba47b50f2635b282`, 1346 lines. This is the production19 file named in that folder's `receipt.json`. Build238 (`review-repair-build238.py`) does not replace `gallery.rs`, so this is also the gallery source of the current build. I re-hashed the base after writing the candidate, and it is unchanged.
- **Candidate.** `gallery-review-repair249.rs`, sha `d57e003a4067f3e67abb654e98c19ca0274c9b7edae5ac26da756757e33230b5`, 1443 lines. It is a copy of the base with 100 lines added and 3 removed (diff below).
- **Finding.** F2 of `cc0-final05-picture-review240.md`: 05_9 says the tiles between zeros and poles are "nearly square", but z³ − 1 has a critical point at z = 0, where the cell on screen is large and not square.
- **Not touched.** F1 (picture-plate fades) belongs to CC2's fade investigation246. This candidate adds no fade code and does not touch `draw.rs`. The new marks are text and polylines that ride two existing `fade_in_all` / `fade_out_all` calls, so they follow whatever those calls do after CC2's work.

## What changes on screen

At "nearly square" in 05_9, two things arrive with the gold outline and "one tile of z³ − 1, traced exactly", in the same existing 0.5 s fade. Both leave in the existing closing 0.5 s fade.

1. **A note** (`UNLESS_NOTE`), 18 pt, dim, centred under the cubic's panel at (−4.5, −0.1): "nearly square unless the derivative is 0, as at z = 0".
2. **A small white ×** at z = 0 on the cubic's panel (`CRITICAL_MARK`): two ink strokes of width 3, reaching ±0.12 in plane units on each axis. A cross reads as neither a zero mark (a white dot on a dark disc) nor a pole mark (a pink ring).

The wording matches the condition candidate231 gave 06 ("unless the derivative is 0"). It adds the place on screen where the condition fails. The voice says "nearly square" as the note arrives, so the part only on screen is "unless the derivative is 0, as at z = 0".

## Mathematics

- **The condition.** The enhanced portrait's tiles are preimages of a square grid under log f. Where (log f)′ = f′/f is finite and nonzero, log f is conformal, so little tiles are nearly square. The voice already sets zeros and poles aside, so between them the condition is f′ ≠ 0. This agrees with the later script: 07_2 says angles get multiplied where the derivative is zero, and 15_2 says "away from critical points".
- **The counterexample on screen.** For f(z) = z³ − 1, f′(z) = 3z² is 0 only at z = 0, which is not a zero of f (f(0) = −1).
  - log f(0) = ±iπ. The band step is s = π/6, and with `BAND_OFFSET` 0.5 the band lines sit at (k − ½)s. Since π = 6s, z = 0 is at the centre of a tile in both size and angle.
  - Near 0, f(z) + 1 = z³, which is three-to-one. So the tile's preimage around 0 is one cell with 4 × 3 = 12 corners, not a little square.
  - The Python mirror finds the twelve corners at |z| = 0.687 and 0.750, at least 0.30 apart. The traced tile's sides are about 0.33.
  - This matches the 05_1 crop in review240, where the whole inside of the unit circle is one cyan cell.
- **A detail for the picture review.** The zoom window's lower-left corner, −0.036 + 0.413i (|z| = 0.415), lies inside this cell. So the zoom already shows a piece of it at that corner. Nothing is changed for this.

## Layout arithmetic

These are scene units, not measured on a render.

**Calibration.** In existing final05 frame 9600, the two 20 pt labels are centred at x = −4.50. Their ink widths are 2.811 and 3.863, against CMR10 estimates of 2.842 and 3.857 (ems × size × 0.01). The ink of "one tile …" spans y −0.44 to −0.76.

**The note.** It is 22.48 ems × 0.18 = 4.05 wide, spanning x −6.525 to −2.475 at y −0.1. Its clearances:

| From | Clearance |
|---|---|
| The cubic panel's bottom edge (0.25) | 0.20 |
| The tile label's ink top | 0.19 |
| The link from the zoom box's lower-right corner to the zoom's lower-left, at the note's top | 0.25 |
| The frame's left edge | 0.59 |

The left margin is slightly tighter than `next`'s (0.68). Centring at −4.4 instead would match `next` and leave 0.15 to the link. I kept −4.5 so all three labels share one centre.

**The ×.** It sits at scene (−4.5, 1.317) and is about 0.19 across (about 50 px at 4K). Its clearances:

- 0.228 below the zoom box's bottom edge. The box's left edge starts at that height, so nothing crosses it.
- 0.646 from each zero mark's centre (each mark has radius 0.11).

The links run away from it.

## Timing

**Unchanged.** The chapter keeps the same 136 time-bearing calls in the same order with the same run times. These include fades, shows, leaves, waits, the breath and the line starts. The new mobs are made without play time and only join the lists of two existing fades. If integration changes nothing else, the re-rendered 05 should still have 9,879 frames.

**Reading time.** These figures are estimated from the existing final05 one-second samples and the cuelog:

| Event | Time (s) |
|---|---|
| "nearly square" | ≈153.92 (the tile label is about 17% faded in at 154.0) |
| Sentence 4 | between 157.0 and 157.6 |
| End of the 05_9 take | 160.547 (frame 8692 plus 376,320 samples at 24 kHz) |
| Closing fade begins | ≈163.15 |

- **On screen.** The note is up for about 9.2 s. About 2.6 s of that is free of speech, plus the short pauses between sentences.
- **Under the program's 15 characters a second rule of thumb:**
  - Before this change, the text on screen that the voice does not say came to about 43 characters (≈2.9 s): the tile label, plus "next:" and "one".
  - With the note it is about 82 characters (≈5.5 s).

So the 05_9 ending was already near the rule, and the note roughly doubles what must be read without the voice.

05_9 has no reading window, and adding one would change timing, which this candidate must not do. Whether the ending needs more time is a question for root's picture review and the audio and sync acceptance that remains open.

## Exact diff (base → candidate)

```diff
--- a/src/chapters/gallery.rs
+++ b/src/chapters/gallery.rs
@@ -13,7 +13,8 @@
 //!   a strip up the line Re s = ½ with the first four zeros, each found by
 //!   Newton's method in the scene (the "precisely" of 05_8 is literal).
 //! - The six pictures side by side, then one tile of z³ − 1 traced exactly
-//!   (Newton along the band lines) to show it is nearly square.
+//!   (Newton along the band lines) to show it is nearly square, and the one
+//!   place on the cubic where tiles are not: z = 0, where the derivative is 0.
 
 use std::f64::consts::TAU;
 
@@ -64,6 +65,15 @@
 pub const FOUR: &str = "this strip shows four zeros, located by Newton's method";
 pub const ONE_TILE: &str = "one tile of z³ − 1, traced exactly";
 pub const NEXT: &str = "next: what one complex derivative demands";
+/// 05_9's condition on "nearly square", which the voice leaves out. The cubic on screen fails it
+/// at one point: its derivative 3z² is 0 at z = 0, which is no zero of z³ − 1, and the cell
+/// there is one large cell, not a little square.
+pub const UNLESS_NOTE: &str = "nearly square unless the derivative is 0, as at z = 0";
+/// Where the note stands: centred under the cubic's panel, above the tile's label.
+pub const UNLESS_AT: (f64, f64) = (-4.5, -0.1);
+/// Half the reach of each arm of the × at z = 0 on the cubic's panel (plane units). A cross,
+/// so it reads as neither a zero (a white dot) nor a pole (a pink ring).
+pub const CRITICAL_MARK: f64 = 0.12;
 /// The walker's output box (05_1, 05_2): it says what the box holds, and that sizes are
 /// squashed to fit (`draw::squash`).
 pub const OUT_BOX: &str = "output f(z), sizes squashed";
@@ -955,7 +965,15 @@
         line.wait_for(stage, "nearly square")?;
         let outline = zp9.path(stage, &tile(), hero(), 6.0)?;
         let tile_says = kit.text(stage, ONE_TILE, 20.0, hero(), v(-4.5, -0.6))?;
-        fade_in_all(stage, &[outline, tile_says], 0.5)?;
+        // the condition the voice leaves out, and where the cubic breaks it: 3z² is 0 at z = 0,
+        // inside one large cell. Both ride the outline's fade in and the closing fade out.
+        let h = CRITICAL_MARK;
+        let cross = [
+            cubic.path(stage, &[C::new(-h, -h), C::new(h, h)], Palette::ink(), 3.0)?,
+            cubic.path(stage, &[C::new(-h, h), C::new(h, -h)], Palette::ink(), 3.0)?,
+        ];
+        let unless = kit.text(stage, UNLESS_NOTE, 18.0, Palette::dim(), v(UNLESS_AT.0, UNLESS_AT.1))?;
+        fade_in_all(stage, &[outline, tile_says, cross[0], cross[1], unless], 0.5)?;
         line.wait_sentence(stage, 4)?;
         let next = kit.text(stage, NEXT, 20.0, Palette::ink(), v(-4.5, -1.4))?;
         fade_in_all(stage, &[next], 0.4)?;
@@ -964,6 +982,8 @@
         let mut all = vec![frames[0], panel_names[0], boxed, zborder9, outline, tile_says, next];
         all.extend(panel_marks[0].iter().copied());
         all.extend(links);
+        all.extend(cross);
+        all.push(unless);
         fade_out_all(stage, &all, 0.5)?;
         painters[0].leave(stage, 0.5)?;
         zoom.leave(stage, 0.5)?;
@@ -1282,7 +1302,7 @@
                 "zoom.show(stage, TILE_BEATS[2])?;",
                 "fade_in_all(stage, &framing, TILE_BEATS[3])?;",
                 "line.wait_for(stage, \"nearly square\")?;",
-                "fade_in_all(stage, &[outline, tile_says], 0.5)?;",
+                "fade_in_all(stage, &[outline, tile_says, cross[0], cross[1], unless], 0.5)?;",
             ],
         );
         let from = nine.find("in little tiles").unwrap();
@@ -1324,6 +1344,83 @@
         assert!(Func::Cubic.zeros(x, y).is_empty());
     }
 
+    /// "Nearly square" fails where the derivative is 0, and the cubic on screen has such a point.
+    /// 3z² is 0 only at z = 0, which is no zero; f(0) = −1 sits at a tile's centre in size and in
+    /// angle, and each corner of that tile has three preimages near 0, so the cell there has
+    /// twelve corners and reaches about 0.75 from 0. The × marks it inside the cubic's panel,
+    /// clear of the zoom's box and of the zero marks; the note sits under the panel, above the
+    /// tile's label and clear of the lower link; and both ride fades that are already there. The
+    /// note's width is 22.5 ems at 18 (CMR10) and the half heights are estimates: the pictures
+    /// decide.
+    #[test]
+    fn the_note_marks_where_the_tiles_are_not_square() {
+        let s = model::band_step();
+        let (_, cubic) = mosaic()[0];
+        assert_eq!(Func::Cubic.eval(ZERO), Some(-ONE));
+        assert!(Func::Cubic.zeros((-0.5, 0.5), (-0.5, 0.5)).is_empty());
+        let w = (-ONE).ln();
+        let mid = |x: f64| {
+            let k = x / s + model::BAND_OFFSET;
+            (k - k.floor() - 0.5).abs() < 1e-9
+        };
+        assert!(mid(w.re) && mid(w.im), "{w:?}");
+        let mut corners = Vec::new();
+        for (a, b) in [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
+            let corner = (w + C::new(a, b).scale(s / 2.0)).exp();
+            let u = ONE + corner;
+            for k in 0..3 {
+                let z = C::polar(u.abs().cbrt(), (u.arg() + TAU * f64::from(k)) / 3.0);
+                assert!((z * z * z - ONE - corner).abs() < 1e-9, "{z:?}");
+                assert!(z.abs() > 0.55 && z.abs() < 0.8, "{z:?}");
+                corners.push(z);
+            }
+        }
+        for (k, a) in corners.iter().enumerate() {
+            assert!(corners[k + 1..].iter().all(|b| (*a - *b).abs() > 0.1));
+        }
+        // the × stays inside the panel, clear of the zoom's box and of the three zero marks
+        let zp = zoom_plane();
+        let reach = CRITICAL_MARK * std::f64::consts::SQRT_2;
+        assert!(cubic.contains(C::new(-reach, -reach)) && cubic.contains(C::new(reach, reach)));
+        assert!((zp.frame.y.0 - CRITICAL_MARK) * cubic.unit() > 0.15);
+        for z in Func::Cubic.zeros(cubic.frame.x, cubic.frame.y) {
+            assert!((z.abs() - reach) * cubic.unit() > 0.11 + 0.15, "{z:?}");
+        }
+        // the note: under the panel, above the tile's label at y = −0.6, inside the frame, and
+        // clear of the link from the box's lower right corner to the zoom's lower left
+        let (x, y) = UNLESS_AT;
+        let (half_h, half_w) = (0.15, 22.5 * 18.0 * 0.01 / 2.0);
+        assert!((x - cubic.frame.center[0]).abs() < 1e-12);
+        assert!(cubic.frame.bottom() - (y + half_h) > 0.15);
+        assert!(y - half_h - (-0.6 + 0.16) > 0.15);
+        assert!(x - half_w > -6.96 + 0.3);
+        let a = cubic.p(C::new(zp.frame.x.1, zp.frame.y.0));
+        let b = zp.p(C::new(zp.frame.x.0, zp.frame.y.0));
+        let top = y + half_h;
+        let link_x = a[0] + (b[0] - a[0]) * (top - a[1]) / (b[1] - a[1]);
+        assert!(link_x - (x + half_w) > 0.15, "{link_x}");
+        // made once, in on the outline's fade, out on the closing fade, and nothing else timed
+        let nine = beat("05_9", "kit.finish");
+        assert!(nine.contains("let tile_says = kit.text(stage, ONE_TILE, 20.0, hero(), v(-4.5, -0.6))?;"));
+        runs_in_order(
+            nine,
+            &[
+                "line.wait_for(stage, \"nearly square\")?;",
+                "let cross = [",
+                "let unless = kit.text(stage, UNLESS_NOTE, 18.0,",
+                "fade_in_all(stage, &[outline, tile_says, cross[0], cross[1], unless], 0.5)?;",
+                "line.wait_sentence(stage, 4)?;",
+                "kit.music_breath(stage, 2.0)?;",
+                "all.extend(cross);",
+                "all.push(unless);",
+                "fade_out_all(stage, &all, 0.5)?;",
+            ],
+        );
+        assert_eq!(nine.matches("UNLESS_NOTE").count(), 1);
+        assert_eq!(nine.matches("unless").count(), 3);
+        assert_eq!(nine.matches("cross").count(), 4);
+    }
+
     #[test]
     fn the_mosaic_fits_the_frame() {
         let panels = mosaic();
```

## Required TEXT registry addition (`chapters/mod.rs`, root's file)

The candidate needs exactly one glyph supplement, `gallery::UNLESS_NOTE`, among the other `gallery::` entries.

**Already prepared by root.** `mod-review-repair256.rs` (sha `e1f4d1b87f4b36f979e7800f7b2048117b9a4809b2372f385350698f5c3e7cbd`, not deployed) differs from `mod-review-repair236.rs` (`b5817c50…`, the copy build238 uses) by exactly this one line. It lists the entry once, at line 356:

```diff
     gallery::ONE_TILE,
     gallery::NEXT,
+    gallery::UNLESS_NOTE,
     gallery::OUT_BOX,
```

That satisfies the requirement. The order of `TEXT` does not matter, because `no_supplement_is_empty_or_duplicated` checks a set. No other `mod.rs` change is needed.

- **No duplicate.** The string differs from `one_derivative::UNLESS_ZERO` ("unless the derivative is 0"), which `mod-review-repair236.rs` already lists. `no_supplement_is_empty_or_duplicated` would reject an exact repeat.
- **Glyphs.** It uses only ASCII letters, the digit 0, a comma and "=". "=" is already drawn by `gallery::CRITICAL` and `gallery::ONE_POLE`.
- **Nothing else.** `UNLESS_AT` and `CRITICAL_MARK` are numbers, not strings. No `TEX`, `MONO`, `READING_WINDOWS` or `TO_READ` entry is needed. The 05_8 site of `each_card_outlasts_its_reading_window` is unchanged; the Python mirror below checks this.

**Optional, root's call:** `docs/storyboard.md`, row 05_9, picture column. No test reads this text. A suggested addition: "With the outline, a small white × at z = 0 on the cubic's panel and a dim note under it: "nearly square unless the derivative is 0, as at z = 0"."

## Tests

- **New: `the_note_marks_where_the_tiles_are_not_square`** (candidate lines 1347–1423). It checks:
  - **The mathematics:**
    - f(0) = −1;
    - there is no zero near 0;
    - log f(0) sits at a band centre in both directions;
    - the twelve corner preimages (each satisfies z³ − 1 = e^corner to 1e−9) are distinct and lie at radius between 0.55 and 0.8.
  - **The ×'s clearances:** inside the panel, clear of the zoom box and clear of the zero marks.
  - **The note's placement:** under the panel, above the tile label, inside the frame and clear of the lower link.
  - **Source order:** the note and the × are made after "nearly square", ride the outline's fade, and are cleared in the closing fade. They are named nowhere else in 05_9.
- **Changed: `the_tile_is_ready_for_the_words_that_name_it`.** Its expected outline-fade line now lists the two new mobs. This is the only line the source change forces; nothing is loosened.
- **Existing tests that cover the change and should still pass:**
  - `the_strip_arrives_on_a_clear_column_and_under_the_pole`: the new names do not contain its forbidden substrings.
  - `the_tile_is_traced_exactly_and_is_nearly_square`.
  - `the_mosaic_fits_the_frame`.
  - In `mod.rs`: `each_card_outlasts_its_reading_window` and `no_supplement_is_empty_or_duplicated`, the latter after the registry line.
- **The corner check and the branch cut.** `-ONE` is (−1, −0.0), so `(-ONE).ln()` is −πi. Corners with b = −1 then have angle −π − s/2, outside the principal range (−π, π].
  - My first draft (sha `1d707048…`) compared principal logs: `((z * z * z - ONE).ln() - w).abs() < s`. That comparison fails by 2π for 6 of the 12 corners, (a, b) = (±1, −1), k = 0..2. CC1's independent mirror (review255 relay) found this failure in `1d707048`.
  - The candidate compares values instead: `(z * z * z - ONE - corner).abs() < 1e-9`, with `corner = (w + C::new(a, b).scale(s / 2.0)).exp()`. Since exp has period 2πi, this is exactly the logs compared modulo 2πi, so no branch is involved. It is stronger than the old check (1e-9 instead of within s). CC1's mirror gives a residual of about 1.2e−16 on all twelve corners.
  - The geometric bounds are unchanged: radius strictly between 0.55 and 0.8, and pairwise distance at least 0.1. The actual radii are 0.6874 and 0.7501, and the minimum distance is 0.300.
- **Compile note for root.** `f64::from(k)` over `0..3` relies on integer fallback to `i32`, as the existing `f64::from(k - 1)` in this file does.

## Checks run

- **Python mirror** (outside the repository) of:
  - the new test's numbers;
  - the changed and new source-order checks;
  - the 05_8 reading-window site in `mod.rs`;
  - a comparison of the timed calls: base 136 against candidate 136, identical in names and run times;
  - line widths, all at most 120.

  All pass. The mirror reproduces Rust's −0.0 sign, so w = −πi as in Rust. The branch-cut failure of the first draft is described under Tests; CC1 found it independently.
- **`rustfmt --check --edition 2024`** with the repository `rustfmt.toml` (max_width 120) passes on the candidate and on the base. An earlier draft with longer local names was reflowed by rustfmt's call-width rule, which would have broken the source-reading tests. The names are now `cross` and `unless`, and the call stays on one line.
- **Not run:** the Rust build, Clippy, the tests, the glyph probe or any render. All Rust is uncompiled.

## For root

1. Put the candidate in place of `chapters/gallery.rs` in the build workspace (base `7d1cf20f…`).
2. Use `mod-review-repair256.rs` (`e1f4d1b8…`) as `mod.rs`. It already has the registry line.
3. Run the guarded build: fmt, Clippy and tests (including the new and the changed test), facts-check, and the glyph probe for the new string.
4. Render 05 under guard and check:
   - the note's legibility at 18 pt dim and its left margin;
   - the × on the cyan cell, between about 154 and 163.6 s;
   - both leaving with the closing fade;
   - the frame count (9,879).
5. Commission an independent picture review of the new 05 render.

## Limits

This candidate establishes no visual, motion, audio, fine-sync or final AV acceptance. No audio was heard or checked. The layout and reading figures are arithmetic from source and existing sampled stills, not measurements of a new render. F1 is left to investigation246.
