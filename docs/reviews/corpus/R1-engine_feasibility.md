<!-- R1 storyboard review, 2026-10-11, from the reviewer's final answer (packet tools/review.py; storyboards at 2ad2579) -->

# R1 engine-feasibility review: storyboards for Videos A, B and C

**Reviewer:** engine_feasibility.

**Pin:** franken_manim `e2eec632` (franken_markdown `bd00a247`), read in cargo's checkout `~/.cargo/git/checkouts/franken_manim-6641d0e09642f63e/e2eec63`. Engine paths are relative to that root; kit paths are relative to this repo.

I built, rendered and edited nothing. Every fps figure is either someone else's measurement (the source is named) or labelled as my estimate.

## What I checked and how

- **Read:** the reviewer prompt; the three R1 packets; plan §6.3–6.6, §9.7, §10.7 and §11.7; the plan-stage engine review (`docs/reviews/plan_r1/engine_feasibility.md`); the skill's SKILL.md, STATUS-LEDGER and AUTHORING-COOKBOOK; the kit's `kit.rs`, `plot.rs`, `play.rs` and `encode.rs`; and `examples/spike_s5_camera.rs`.
- **Read only through `kit.rs`'s wrappers:** `narration.rs`.
- **Verified in code at the pin:** the facts K1–K11 below. Two of them overturn guidance in the plan and the kit (E-X1, E-X2).
- **Counted, not estimated:** mobject and segment counts. I took them from the storyboards, fact sheets and `docs/research/bakery_algorithm/scripts/tree_count.py`.
- **Not checked:**
  - real raster cost per shape at 4K (that is spike work);
  - whether S-5's 6.1 fps carries over to Video B's cube;
  - test status for `Axes` placement and boolean ops (the ledger calls `Axes` PARTIAL);
  - SVG import budgets.

## Engine facts at the pin this review rests on

- **K1. Group moves cost O(M²).** Every shift, scale and rotate dedupes the family with `Vec::contains` (`crates/fmn-mobject/src/positional.rs:689-697`; shift → `shift_many` → `apply_affine_many`, `:805-816`). The kit's limit is 2,000 members (`crates/explainer_kit/src/kit.rs:33`).
- **K2. Every frame copies the whole arena.** Each captured frame snapshots every slot and clones each live slot's `submobjects` and `parents` vectors (`crates/fmn-mobject/src/stage.rs:2033-2060`; `crates/fmn-anim/src/frame.rs:77-90`). Cost is O(live mobjects) per frame.
- **K3. Native `always_redraw` reclaims its slots.**
  - It deletes the content it replaces (`crates/fmn-mobject/src/dynamics.rs:224-247`, in since ee9074fe, 2026-07-22).
  - `delete` puts the slot on a free list (`stage.rs:1143-1178`), and `alloc` reuses it (`stage.rs:636-646`).
  - Deletion is deferred only while an entry is pinned (`stage.rs:1144-1146`). `Stage::pin` has no native caller outside tests; only the Python portal pins. So fm-92gu is a portal leak.
  - In our native scenes `always_redraw` therefore does **not** grow the arena. It does rebuild, re-lay-out and re-tessellate its content every frame.
- **K4. Per-point paint switches on an expensive fill path.**
  - The retained planner reads the first and last record's `fill_rgba` (`crates/fmn-render/src/plan.rs:1411-1446`).
  - If any record's paint differs, it builds a `FillProfile` (`plan.rs:986-994`; `crates/fmn-render/src/fill_profile.rs:196-247`), and the fill stops being "flat" (`fill.rs:2905-2915`, used at `engine.rs:1613` and `:1652`).
  - A non-flat fill makes every pixel evaluate a mean-value interpolant over **all** of that mobject's stations, every contour included (`fill.rs`, `param_at_contours`, the `for i in 0..n` loops). The station count is at least 3 per contour, plus the profile's knots (`fill.rs:2337-2427`; `crates/fmn-render/src/fill/color_field.rs:23ff`).
  - Flat fills skip the field entirely (`fill.rs:2898-2904`).
- **K5. Retained caps.** Instances, input curves, segments, shapes, styles and profile knots are each capped at 2²⁰ (`plan.rs:105-110`).
- **K6. Camera-only programs and no tile cache on the camera route.**
  - The retained route refuses every non-vector program (images, surfaces, point clouds) with `CameraRequired` (`crates/fmn-render/src/retained.rs:219-228`).
  - The camera path has no pixel-tile cache: compare `render_with_camera` (`:261-276`) with `render_into_cached` (`:240-251`).
  - Camera captures always run the certified CPU engine (`crates/fmn/src/rendering.rs:255`).
- **K7. Camera behaviour.**
  - `CameraRig` uses normalized linear interpolation, so each arc must stay under π (`crates/fmn-scene/src/camera_rig.rs:18`).
  - `is_fixed_in_frame` is honoured in 3D (`crates/fmn-render/src/three_d.rs:2441`, `:2461`, `:3055`) and set by `Stage::set_fixed_in_frame` (`positional.rs:1211`).
  - Drawing is painter order with a per-fragment depth test (`crates/fmn-mobject/src/order.rs:38-43`).
  - VMobjects default to `depth_test = false` (`uniforms.rs:105`); solids default to true (`crates/fmn-library/src/solids.rs:369`).
- **K8. Glow is expensive.** `v_highlight` adds `n_layers` full clones with widened strokes (`crates/fmn-library/src/vmobject.rs:1196-1219`).
- **K9. TransformMatchingTex has no key map.** It pairs only identical source slices (`crates/fmn-library/src/text_matching.rs:35-41`). `Tex::parts_by_tex` selects a substring's glyphs (`tex.rs:397`).
- **K10. Native and usable:**
  - SVG import with even-odd fills (`svg.rs:62`, `svg_paint.rs:172`);
  - boolean ops (`boolean_ops.rs`);
  - `StreamLines` / `AnimatedStreamLines`, animated by a passing flash per line (`fields.rs:1286`, `:1650`);
  - `TracedPath` (`fields.rs:1846`) and `MoveAlongPath` (`fmn-anim/src/movement.rs:254`);
  - `exmark` (`matchers.rs:287`) and `Prism` (`solids.rs:1710`);
  - multi-line Text (`text.rs:374`).

  There is **no** clip or mask primitive.
- **K11. Kit limits.**
  - The raw-input cap is 256 GiB (`crates/explainer_kit/src/encode.rs:47`), i.e. 22,093 4K NV12 frames or about 368 s per chapter.
  - `render_camera` requires the delivery capability (`encode.rs:296-313`).
  - The kit has no fixed-in-frame helper.
  - `dot_cloud` is a group of separate `Dot`s (`plot.rs:60-68`).
- **Measured elsewhere:**
  - The camera route ran at 6.1 fps at 4K60 on the 44×11 prism field (S-5, plan §6.6).
  - On this Mac, Hoeffding's 4K60 renders at about 15 fps with x264 (34 min for 30,431 frames).

## Corpus-wide findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| E-X1 | engine | plan §6.6 rule 3; §10.7; A 07_1; B 07_7; S-3 | major | "One VMobject per row strip with per-point fill" (plan-stage review F-16) gives the right mobject count but the wrong paint model at the pin (K4). As soon as paints differ inside one mobject, every pixel of that strip runs an interpolant over at least 3 stations per cell plus knots: at least 768 per 256-cell strip, at least 300 per 100-cell row. The interpolant also mixes neighbouring cells' parameters, so cell edges can bleed. Flat-painted mobjects never build the field. My order-of-magnitude estimate from station counts: 1–3 s per 4K frame for B's table whenever strips move, and 0.2–1 s per frame for A 07_1's flashing grid. | Rewrite rule 3 as "flat paint per mobject". Either group cells by row × colour class (each mobject one flat colour, many subpaths), or use one mobject per cell up to about 10k. Leave a hairline gap between cells so no two mobjects share an edge. S-3 compares the three representations (spikes section). |
| E-X2 | engine | `kit.rs:8-11`; plan §6.6, §9.7 ("arena slots are never reclaimed"), R16; S-8 | major | The arena-growth premise is stale for native scenes (K3), and the plan and kit are steering design around a leak that native scenes don't have. The "small readouts only" rule still holds, but the reason is per-frame rebuild, layout and tessellation, not arena growth. | Correct the kit comment and the plan text. Shrink S-8 to a confirmation run (spikes section). Keep the rule. |
| E-X3 | engine | A 05_7–05_9; B 03_2–03_5 | minor | The kit cannot pin a live readout to the screen on the camera route. `Kit::live_number` rebuilds plain children every frame (`kit.rs:375-403`), and a fixed-in-frame flag on the container never reaches the fresh child. | Add `Kit::live_number_hud` and `text_hud` that call `s.set_fixed_in_frame(m, 1.0, true)` inside the closure (K7). |
| E-X4 | engine | A 02_6, 04_7, 06_5; B 06_1 | minor | With no key map (K9), "100" cannot glide into "k": changed tokens crossfade. 04_7's "each number swapped for its letter, one at a time" works only as a chain. | Do one substitution per TransformMatchingTex, so the rest of the line glides. Or select parts with `parts_by_tex` and `replacement_transform` them in pairs. |
| E-X5 | engine | A 01_1 glow; B XOR flashes | minor | `v_highlight` multiplies shapes and wide-stroke coverage (K8). | Use at most 3 glow layers, and only on hero objects. Or put one translucent halo disc behind each object, which costs a single flat shape. |
| E-X6 | engine | A 05_7–05_9; B 03 | minor | Camera frames get no tile cache and run on the certified CPU engine (K6). A still on the camera route costs as much as an orbit. | Keep camera segments to the lines that need 3D. Put cards and arithmetic after the boundary on the retained route (S-9 proved the join). |
| E-X7 | engine | all open spikes | minor | With NVENC down, x264's ~15 fps caps every 4K spike number, so spike timings measure the encoder. | Time spikes raster-only: `max_frames` plus `--silent` (cookbook §14), or PNG/Y4M output. Log ms per frame split into updaters, snapshot and raster. |

## Video A: How to Count Fish You've Never Seen

**Key picture per chapter, and verdicts**

| ch | key picture | non-trivial beats → verdict |
|---|---|---|
| 01 | The seeded lake (first catch gold, second catch green) folds into the 2×2 table with "≈ 8,100" in the dark cell; then one 451-fish dip is shown to swing 31× from spot to spot. | 01_1–01_2 swarm → table: feasible with a change (E-A1). 01_5–01_6 net, 10×10 grid and heat map: feasible (100 flat cells). |
| 02 | The bar "100 of 1,000", then 1,000/? → 10,000. | 02_6: feasible; changed tokens crossfade (E-X4). |
| 03 | A god's-eye lake whose density stays clumpy while the marked-fraction map flattens to about 10%. | 03_3 zoom: feasible with a change (E-A1). 03_4: dynamics **not feasible as written** (E-A3); maps feasible with a change (E-A2). 03_5: feasible. |
| 04 | 4,845 netful glyphs pour into five bins: 1,365 / 2,275 / 1,050 / 150 / 5. | 04_3: feasible. 04_4: open spike S-4; feasible with a change (E-A4). 04_6–04_7: feasible as a chain (E-X4). |
| 05 | The k = 3 bar's height, traced over N = 17…60, becomes the score curve; then an orbit around the 44×11 lit landscape. | 05_4: feasible (11 bars). 05_7–05_9: route already proven (E-A11); the crest curve doesn't read (E-A5); labels need a change (E-A6). |
| 06 | Mn − kN becomes ab − k·u on the table, and a seesaw levels at 8,100. | 06_3 strike-throughs: feasible (`parts_by_tex` plus drawn lines). 06_5: feasible with a change (E-A7). 06_6: feasible (rotate by placement). |
| 07 | 100×100 mini-lakes flash their estimates; the k-bell's bars slide along the hyperbola into a right-skewed histogram. | 07_1: feasible with a change (E-A8). 07_3: feasible by placement (non-uniform scale plus shift per bar). 07_5 code panel: feasible. |
| 08 | The relative score curve over lake size, with [8,400, 12,073] shaded. | 08_2 scrolling zeros: feasible (one long Text shifted). 08_5, 08_7: feasible. |
| 09 | The table with a gauge on the dark cell, swung by each failure mode. | 09_8: feasible with a change (E-A9). |
| 10 | France, two code reviewers and Kosovo each fill the same table's dark cell. | 10_1: feasible (E-A10). 10_4 Venn: feasible (boolean ops). 10_6: feasible. |

**Heavy beats.** Counts are counted; costs come from the code; fps is unmeasured unless marked.

| beat | as written | with the fixes | per-frame work |
|---|---|---|---|
| 01_1–01_2, 03_4 fish | 10,000 fish implied (A-F27) | 1,000 glyphs × (body + tail) ≈ 2,000, plus ≤ 100 halos | 1,000 shifts and 1,000 tail rotations by placement (trivial per 2-member family, K1); snapshot of ≈ 2.1k slots; ~1,000 small shapes rastered, all tiles dirty. S-2 decides. |
| 03_4 maps | maps "from a large simulated population" | 10,000 fish as plain Rust state; 18 flat region cells | about 0.5 M particle-steps per frame in plain Rust at the planned time-lapse (well under the frame budget, estimate); 18 recolours per frame. |
| 04_4 netfuls | 4,845 × (4 dots + frame) ≈ 24,200 mobjects, all moving | ≤ 300 in flight (≈ 1,500 mobjects) plus ≈ 650 pre-built stack rows | As written: a ~24k-slot snapshot with vector clones (K2), 4,845 updater shifts and ~24k shapes rastered per frame. With the fix: ≈ 2.2k mobjects. |
| 05_7–05_9 | 484 prisms, 2 planes, a curve, labels | same | **Measured** 6.1 fps on this exact field (S-5): ≈ 25 s is ≈ 1,500 frames, ≈ 4 min. |
| 07_1 grid | 100 rows with per-point paint | 10,000 flat cells | As written: a per-pixel interpolant over ≥ 300 stations for each row whose cells change (E-X1). With the fix: ≈ 17–56 cells restyled per frame (10,000 trials over 3–10 s); the rest stays tile-cached. |

**Findings**

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| E-A1 | engine | 01_1–01_2; 03_2–03_3 | major | The lake is Feller's 10,000 fish (A-F27; 1,900 seen, 8,100 dark), but S-2 tests only 600–1,500 moving fish. The storyboard never says what one glyph stands for, yet 03_3 counts single fish ("fifty fish, five marked; ten fish, one marked"), which a scaled lake cannot show. Its "zoom (group scaling)" on a ~2,000-member lake sits at the K1 limit on every frame. | Fix one scale for the whole video: **1 glyph = 10 fish** (1,000 glyphs, cells of 10 / 90 / 90 / 810, so the dark cell physically holds 810 dark glyphs), with a key on screen. Fallback at S-2's low end: 1 glyph = 20 fish (5 / 45 / 45 / 405). For 03_3, draw two 1:1 insets from the simulation's actual fish in those patches, labelled "every fish", instead of scaling the lake group. Move fish into the table by per-fish placement along precomputed paths, never by a Transform of the swarm. If S-2's 10,000-dot row is fast enough, go 1:1 and drop the key. |
| E-A2 | engine | 03_4; 09_8 | major | If the marked-fraction map is recomputed from the **visible** glyphs (the showcase matrix's "live recomputation from moving objects"), it will not visibly flatten. With 1,000 glyphs and 100 marked, a sparse region holds ~25 glyphs, a binomial SD of ≈ 6 points around 10%. The fact sheet's 10,000-fish run gives ≈ 2 points (its 0.020). | Simulate all 10,000 fish as plain Rust state in one dt updater (positions, not mobjects). Draw a 1-in-10 subsample. Recolour both 3×3 maps (18 flat cells) from all 10,000 every frame. Seed with SplitMix64 as A-F27 does; check the numbers shown on screen against `A-CH03.*` at Monte Carlo tolerance (fact sheet item 12). |
| E-A3 | engine | 03_4 "fish circle their favourite haunts along the habitat's contour lines" | major (not feasible as written) | The oracle's dynamics will not look like circling. From the A-F27 habitat (σ = 0.08, amplitude 40) at r = σ, by my own arithmetic (not checked by the oracle): \|∇U\| ≈ 12, circulation c·\|∇U\| ≈ 0.24 lake widths per time unit, Péclet ≈ 1.9. At a time-lapse that shows the flattening (t = 0 → 100 in about 12 s, Δt ≈ 0.14 per frame), noise alone moves a fish ≈ √(2·0.01·0.14) ≈ 0.05 lake widths (about 100 px at 4K) per frame, against ≈ 0.03 of circulation. The swarm will shimmer. Slower time-lapses are worse, because noise scales with √Δt and drift with Δt. | Keep the honest simulation for the maps (E-A2). Draw the visible glyphs from a smoothed copy of their paths (e.g., an exponential moving average over about 0.5 s of screen time), labelled "time-lapse". Let 20–30 `AnimatedStreamLines` along U's contours carry the idea of circling (K10). Do not raise c or D for looks without re-running `A-CH03`. |
| E-A4 | engine | 04_4 (S-4) | major | As written, all 4,845 glyphs move at once: ≈ 24,200 mobjects snapshotted with two vector clones each per frame (K2), 4,845 updater shifts, and ~24k small shapes rastered over dirty tiles. It is the heaviest retained beat in the corpus. | Stream at most ~300 glyphs at a time. Pre-build each bin's final stack as rows of flat-paint multi-subpath mobjects (one gold and one blue per row, ≈ 650 mobjects) at opacity 0, and reveal each row as its glyphs land: a style edit with no point rewrites. All 4,845 netfuls still end up drawn, so the count stays exact. S-4 measures both designs. |
| E-A5 | engine | 05_8 crest curve "along the floor" | major | The crest k = Mn/N runs under the tallest bars. At S-5's tilt (φ = 1.05), with 0.2 × 0.3 bars on a 0.25 × 0.4 pitch (`spike_s5_camera.rs:36`, `:60-61`), the floor there is mostly hidden. A VMobject curve defaults to `depth_test = false`, so it paints over the bars and floats in front of them; with the depth test on, the bars hide it (K7). Either way it does not read as a floor curve. | Draw the crest as a depth-tested ridge line over the bar tops (z = P_N(K = k) at k = Mn/N). Or show it on the floor in the top-down still before the bars rise. Or drop the bars to about 0.3 opacity while it draws. Test all three in one S-5 re-run (one frame each). |
| E-A6 | engine | 05_7 "axes labelled in words" | minor | Text in 3D lies flat in its plane and foreshortens as the camera turns. S-5's three 120° arcs would turn labels edge-on. The kit has no HUD helper (E-X3). | Fix the labels in frame for the still and fade them during the orbit. Or rotate 3D Text to face the orbit's middle heading and keep the total orbit to about 90°. |
| E-A7 | engine | 06_5 "TransformMatchingTex onto the table" | minor | TransformMatchingTex works only between two Tex objects, and the table's cells are separate mobjects. | Pick a, b, k and u in `Mn − kN = ab − k·u` with `parts_by_tex`, `replacement_transform` each into its cell label, and fade the rest. |
| E-A8 | engine | 07_1 "100 row VMobjects recoloured in place" | major | E-X1 applies: each recolour rebuilds a row's fill profile and re-runs the per-pixel interpolant for that row on every flash. My estimate is 0.2–1 s per 4K frame over a ~10 s beat. | Use 10,000 flat squares, each recoloured as its trial lands. A 10,000-slot snapshot is cheap (K2), and untouched cells stay tile-cached. |
| E-A9 | engine | 09_8 non-flattening split view | minor | This needs a second simulated run in which the marks never mix; it cannot be a replay of 03. | Use E-A2's machinery with a seeded no-mixing run. The ≈ 6,250 and ≈ 25,000 gauge values come from numpy-seeded A-F22 keys, so pin them as recorded values (fact sheet item 12). |
| E-A10 | engine | 10_1 France outline | minor | This needs an outline asset. | `svg_mobject_with_paints` (K10) on a public-domain outline such as Natural Earth, or a `Polygon` from its coordinates. Name the source on the receipts card. |
| E-A11 | engine | 05_7 | delight | The hardest 3D shot in the corpus has already been rendered: 44 lake sizes (17…60) × 11 counts (0…10) is exactly S-5's 44×11 field, done at 4K60 with sound. | — |

## Video B: The Arithmetic Where 3 × 7 = 9

**Key picture per chapter, and verdicts**

| ch | key picture | non-trivial beats → verdict |
|---|---|---|
| 01 | A tower and ~1,000 phone glyphs, each with its own losses; a carousel with a long tail; a sticker album counting to 97,876. | 01_1, 01_4: feasible with a cap (E-B4). 01_5: feasible with a change (E-B1). 01_6: feasible. |
| 02 | Equation cards beside a cards × pieces grid; solving by ex-or. | feasible. |
| 03 | GF(2)³'s eight cards on cube corners, with the lit span growing 1 → 2 → 4; then the danger meter spiking at the end of the fill. | 03_2–03_5 camera: feasible on the proven route with changes (E-B3). 03_6: feasible. |
| 04 | Failure at 71 / 42 / 23 / 12%, and the pentagon whose letters cancel. | feasible. |
| 05 | The 256-hour clock with the top bit falling into the gutter; 7-hour vs 16-hour tables. | 05_3, 05_5, 05_6: feasible (≤ 256 flat cells). |
| 06 | Bits → polynomial; FOIL with the doubled terms vanishing; the overflow folding back as 0x1D. | 06_1: feasible with a change (E-B5). 06_3: a native fit (only `2x²` and `2x` fade). 06_5–06_8: feasible. |
| 07 | The 255-point doubling star untangling into the byte clock; the 256×256 table sorted into diagonal stripes. | 07_2–07_5: feasible (labels: E-B6). 07_7: open spike S-3; feasible only with a change of representation (E-B2). 07_8: feasible. |
| 08 | Pies of 1/256 vs 1/2; histograms of spare cards needed. | feasible. |
| 09 | A sparse pieces/packets graph; a peeling cascade that stalls. | feasible (keep the graph to about 200 edges or fewer). |
| 10 | The sparse XOR band with a thin gold band of byte rows; ablation counters 70.7% vs 0.646%; the pentagon walk 3A = 15. | 10_2's 27×27 grid: feasible (729 flat cells). The rest: feasible. |
| 11 | Measured 6,316 / 24 / 1 failures against the standard's 1/100, 1/10⁴, 1/10⁶. | feasible. |

**Heavy beats**

| beat | as written | with the fixes | per-frame work |
|---|---|---|---|
| 01_1 phones | ~1,000 phones plus losses | flat-paint groups: at most ~2,000 mobjects, or ~50 if grouped per row | static and tile-cached |
| 01_5 carousel | ~1,000 live counters | ≤ 6 colour classes plus one histogram | As written: ~1,000 DecimalNumber layouts per frame, with thousands of glyph mobjects created and deleted (K3). With the fix: a handful of recolour events per loop. |
| 03_2–03_5 cube | 8 corners, 12 edges, ≤ 3 translucent planes, ~20 arrows, labels | same | Camera route with no tile cache (K6). Unmeasured, but far lighter than S-5's 484 prisms. About 50 s ≈ 2,900 frames, so 5–8 min at 6–10 fps (estimate). |
| 07_2–07_5 star and rings | 255 dots, 1 polyline, ≤ 510 ticks and labels | same | Ring rotation dedupe ≈ 510² ≈ 2.6 × 10⁵ comparisons per frame (K1): fine. |
| 07_7 table | 256 strips with per-point paint | 256 × K flat mobjects per orientation (K = 16 or 32) | As written: an interpolant over ≥ 768 stations per strip on each repaint (estimated 1–3 s per frame during the reorder). With the fix: 4,096–8,192 flat mobjects and 262,144 segments, under 2²⁰ (K5). |

**Findings**

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| E-B1 | engine | 01_5 "a loop counter per phone" | major | About 1,000 live counters means about 1,000 `always_redraw` numbers rebuilt and laid out every frame. With slots reclaimed (K3) the arena holds, but that is still thousands of glyph mobjects created and deleted per frame, and digits are unreadable on a phone of at most ~90 px at 4K anyway. | Recolour each phone by the loop in which it finished (≤ 6 flat classes). Add one histogram of phones per loop, with the tail labelled (all 10⁶ phones done needs 5–6 loops, B-F24). Keep a single live loop counter on the carousel. |
| E-B2 | engine | 07_7 (S-3); plan §10.7 | major | S-3 as specified tests the representation that is expensive at the pin (E-X1): every strip builds a profile and a per-pixel interpolant over at least 768 stations, and neighbouring cells can bleed. Separately, byte 0 has no click count. | Colour cells by click count (0…254) on a cyclic palette quantized to 16–32 classes. Use one flat mobject per row × class, with hairline gaps between cells. Reorder rows by `shift_many` on each row's K mobjects. At a hold, swap to a pre-built column × class set (pixel-identical when no edges are shared), then reorder columns. Pin row and column 0 at the edge in dim grey. Fallbacks are unchanged (64×64 block; GF(16)). |
| E-B3 | engine | 03_2–03_5 camera segment | minor | About 50 s on the camera route (no tile cache, certified CPU, K6). Corner labels foreshorten. 03_5's live product needs a HUD (E-X3). Edges are VMobjects, so they paint over the planes unless depth-tested (K7). | End the segment after 03_4's answer and do 03_5's arithmetic on the retained route (the S-9 join), or keep it with a fixed-in-frame HUD. Keep the total orbit to about 90° (arcs < π) so labels stay readable. Draw the planes translucent and last, as in S-5. Turn on `depth_test` for edges that should pass behind the planes. |
| E-B4 | engine | 01_1, 01_4 | minor | A phone is at most ~90 px square at 4K, so a "loss pattern" reads only as a few red flecks. An uncapped arrow swarm costs 2 mobjects per arrow. | Draw losses as flat red flecks grouped one mobject per row. Cap the request arrows at 200–300, moved by placement. |
| E-B5 | engine | 06_1 "bits → Tex morph" | minor | Bits written as Text cannot join a TransformMatchingTex with a Tex polynomial (plan §6.5; K9). | Typeset the bit row in Tex and `replacement_transform` each 1-bit (via `parts_by_tex`) into its power of x, or crossfade. |
| E-B6 | engine | 07_2–07_5 | minor | 255 labels on a circle of radius 3.5 units sit about 23 px apart at 4K, so three-digit labels collide. Labels on the inner ring turn upside down as it rotates 25 clicks. | Label only the nodes the narration names (1, 2, …, 128, 29, 58, 150, 3, 167). Counter-rotate the remaining labels by placement each frame, which is cheap. |

## Video C: The Algorithm Lamport Is Proudest Of

**Key picture per chapter, and verdicts.** Nothing in Video C exceeds about 1k mobjects, and it has no camera route unless the optional cube stays.

| ch | key picture | verdict |
|---|---|---|
| 01 | Two lanes read 41 and write 42; the progress square's 6 staircases, 4 of them through the middle. | feasible |
| 02 | The square with a red "both inside" point; 14 staircases, 8 through it. | feasible |
| 03 | Five private slates; customer 4 reads 0, 3, 0, 2 and writes 4. | feasible (`exmark` for the ✗, K10) |
| 04 | Both customers take 6; the lexicographic grid swept by a gold scanline. | feasible |
| 05 | The unwritten-ticket collision; amber flags; 28 → 322 staircases, none through the red point. | 05_1–05_5: feasible. 05_6: cheap for the engine, but the picture doesn't read as written (E-C1). Optional cube: feasible (E-C2). |
| 06 | One fast run with a live slate table and a step counter. | feasible (E-C4) |
| 07 | Green's doorway interval sliding against Blue's flag read, in three zones. | feasible |
| 08 | The proof zones, with magenta garbage reads landing only in the zone whose value is unused. | feasible |
| 09 | FCFS on the lanes; a ticket odometer; the 2³⁵ arithmetic. | 09_3: feasible with a change (E-C3) |
| 10 | Store-buffer outboxes let both customers walk in; a fence drains a tray. | feasible (E-C5) |
| 11 | Chapter 04's grid relabelled as (clock, process); quote cards. | feasible |

**Heavy beat.**

- **05_6 (counted):** 322 polylines × 14 edges ≈ 4.5k segments, plus the 28-path set and the lattice: about 400 mobjects. A group zoom costs ≈ 400² = 1.6 × 10⁵ comparisons per frame (K1).
- **Optional cube:** with three customers each runs 10 steps (`tree_count.py` `make_bakery`), so the lattice is 10×10×10. About 200 sampled 3D polylines would be ≈ 6,000 segments on the camera route.

**Findings**

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| E-C1 | engine | 05_6 "28 (4 through the red square) morph to 322 (none)" (S-6) | major | The engine cost is small; the picture is the problem. With flags each customer runs 7 steps (`make_bakery`), so the square is a 7×7 lattice with 112 unit edges. 322 paths × 14 edges ≈ 4,500 edge uses, about 40 paths per edge: plain polylines collapse onto at most 112 visible segments, and "322" cannot be seen. The no-flag set lives on a 4×4 lattice (4 steps each), so no path corresponds to a path on the other side of a morph. | Fan each staircase by a small per-path offset (within ±0.3 of a cell), or draw edge width ∝ path count. Replace the morph with a sequence: the 28 collapse into a "28 orders, 4 bad" card; the lattice regrows to 7×7; the 322 draw in with a lagged reveal and a counter; the both-inside point (6, 6) stays empty and pulses. S-6 chooses fan or width at 1080p. |
| E-C2 | engine | 05 engine note (optional 3D cube) | minor | Feasible on the camera route (K6–K7), on a 10×10×10 lattice. Sampling scheduler choices uniformly would over-represent orders with few branch points. | If the cube stays, sample orders uniformly using the subtree counts `tree_count.count` already computes. Draw "two inside" beams as depth-tested translucent prisms, last. Keep arcs under π. The counter to 2,126,816,238 is exact in f64. Cut it first if chapter 05 runs long. |
| E-C3 | engine | 09_3 odometer | minor | No clip or mask exists at the pin (K10), so rolling digits cannot be cropped to a window. | Slide and fade the digits (opacity falls with distance from the window centre), or intersect the ≤ 6 visible digits with the window using boolean ops. |
| E-C4 | engine | 06_1–06_4 live slate table | minor | Ten values (five flags, five tickets) under `always_redraw` would be re-typeset every frame, although they change only at steps. | Swap each value with `replacement_transform` at its step event, driven by the shared step clock (§11.7). |
| E-C5 | engine | 10_4 code panel | minor | The line `let t = 1 + number.iter().map(\|n\| n.load(SeqCst)).max().unwrap(); // each slate read once` is about 93 monospace characters, i.e. ≈ 0.15 scene units per character across the 13.9 usable units. That is small for a panel. | Move `// each slate read once` onto its own line above; the layout lint will confirm the fit. |

## Open spikes: what each must answer

- **S-2 (fish):**
  1. Raster-only ms per frame at 4K for 600, 1,000 and 1,500 fish (body plus tail child, placement-only motion; 10% glowing, compared as ≤ 3-layer `v_highlight` vs one halo), plus about 25 `AnimatedStreamLines`.
  2. An added row: 10,000 plain `Dot`s moved by placement. At about 10 fps or more, Video A can be 1:1.
  3. At 1080p, check that 1,000 glyphs read as a lake and that the smoothed display paths (E-A3) read as swimming.

  Outcome: fix the fish-per-glyph scale once for the whole video.
- **S-3 (256×256 table):**
  1. Compare three representations: (a) 256 strips with per-point paint, as specified; (b) 256 × K flat mobjects with K = 16 and 32; (c) 65,536 flat cells.
  2. For each, measure ms per frame static, during a full row reorder, and at the row-to-column swap. Check segment and style counts against 2²⁰.
  3. Check pixel colours at cell centres and edges; (a) is the one at risk of bleeding (K4).
  4. Measure PSNR across the swap frame (it should be infinite with hairline gaps).

  Pass: the fastest pixel-exact representation under about 0.5 s per moving 4K frame. Otherwise fall back to the 64×64 block or GF(16).
- **S-4 (netfuls):**
  1. Compare as planned (≈ 24k mobjects, all moving) with streamed (≤ 300 in flight plus ≈ 650 stack rows revealed by opacity): raster-only ms per frame, and the snapshot cost alone at 24k slots (K2).
  2. Diff the final frames of both designs; they must match, so the cheaper one is honest.
- **S-6 (staircases):**
  1. This is a legibility test, not a performance test: fan offset vs edge width at 1080p for 322 paths on the 7×7 lattice, plus the 4×4 → 7×7 transition (E-C1).
  2. If the cube stays, measure camera-route fps for about 200 polylines on the 10×10×10 lattice with translucent beams.
- **S-8 (arena growth):**
  1. Code says native `always_redraw` reclaims its slots (K3). Confirm on Video A's chapter 03 maps, written both ways (18 `always_redraw` cells vs 18 cells recoloured in place), by comparing the arena slot count (or ms per frame) at t = 0 and t = 80 s. Expect both to stay flat.
  2. If flat, close S-8 and correct the kit comment and plan text (E-X2).
  3. If not flat, find what pins or leaks: Transform targets, or per-frame Text builds outside `always_redraw`.

## Render budget on this Mac (estimates)

x264 caps output at about 15 fps (≈ 65 ms per frame), so any beat slower than that sets the pace. Every chapter is far under the kit's 368 s raw-input cap (K11).

| video | length | frames | at ~15 fps | camera surcharge | heavy beats as written | total with the fixes |
|---|---|---|---|---|---|---|
| A | 12.3 min | 44,280 | ≈ 49 min | 05_7–05_9: ≈ 1,500 frames at 6.1 fps, ≈ +2.4 min | 07_1 ≈ +4–6 min; 04_4 unknown (S-4) | ≈ 52–55 min |
| B | 13.3 min | 47,880 | ≈ 53 min | 03: ≈ 2,900 frames at 6–10 fps, ≈ +2–5 min | 07_7 ≈ +5–15 min | ≈ 56–60 min |
| C | 11.8 min | 42,480 | ≈ 47 min | none (optional cube ≈ +2–4 min) | none | ≈ 47 min |
