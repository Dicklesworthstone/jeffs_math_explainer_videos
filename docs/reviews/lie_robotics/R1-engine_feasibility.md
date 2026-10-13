# R1 review (storyboard): lie_robotics, engine feasibility

Reviewer: engine_feasibility. Stage: R1, the storyboard of Video G, "Why Robot Arms Think in Lie Groups". Date: 2026-10-13. Engine: franken_manim at the workspace pin `a399d626` (FRANKEN_MANIM.pin, root Cargo.toml:17), franken_markdown `126a5304` for TeX and fonts (the pin's own Cargo.toml:45). Route judged: the retained 2D route, one scene per chapter, every three-dimensional picture drawn by the video's own projection.

Counts: 0 blockers, 6 major, 12 minor, 2 delight.

Paths below: repository files are relative to the repository root; `fmn@pin:` is cargo's checkout of the pin, `~/.cargo/git/checkouts/franken_manim-6641d0e09642f63e/a399d62/`; `fmd@pin:` is `~/.cargo/git/checkouts/franken_markdown-2f7478214ec48ef9/126a530/`.

## What I checked and how

- **Read:** the reviewer prompt; the whole packet (brief, storyboard, fact sheet); plan §2.1, §3.3, §6.1 to §6.7; the capability survey (made at `fb1c1b9e`, so every engine claim I lean on I re-read in the pin's source).
- **Engine source at the pin:** `fmn-mobject` `dynamics.rs` (`always_redraw`), `stage.rs` (scene list, `set_z_index`, updaters, the clock mirror), `order.rs`, `positional.rs` (`set_points`, `set_placement`), `placement.rs`; `fmn-render` `plan.rs` (what recompiles, the compaction limits), `engine.rs` (what the retained route refuses), `stroke.rs`, `retained.rs`; `fmn-library` `arc.rs`, `line.rs`, `poly.rs`, `tip.rs`, `brace.rs`, `numbers.rs`, `matrix.rs`, `vmobject.rs`, `lib.rs`; `fmn` `prelude.rs`, `lib.rs`; `fmn-text` `font.rs`; `fmd-math` `commands.rs`, `token.rs`, `parse.rs`, `faces.rs`.
- **Kit and corpus:** all of `kit.rs`, `props.rs`, `play.rs`, `animatic.rs`, `glyphs.rs`; `storyboard.rs` to line 110, `layout.rs` to line 230, the wait, listen and sonify functions of `narration.rs`, the parts of `palette.rs`, `plot.rs`, `cli.rs` and `encode.rs` cited below. Video E `draw.rs` (styles, the square, the cube by projection) and its registry; all of Video B's `cube.rs`, and the cited parts of its `fold.rs`, `polynomials.rs`, `byte_clock.rs`; Video A `small.rs` (gauge, arcs, sectors, dashes); the cited parts of Video C's `proudest.rs`, `fair.rs`, `draw.rs`; Hoeffding `shuffle.rs` (the two redraws). Video G's `main.rs`, `chapters/mod.rs`, `chapters/glyphs.rs`, `score.rs`, and the signatures of `model.rs` with the functions cited. Video F's R1 engine review (its quiz finding) and that finding's disposition.
- **Skill notes:** STATUS-LEDGER, TEX-AND-TEXT, PERFORMANCE-AND-GPU and INCIDENT-CORPUS in full; NATIVE-API §3 to §9; AUTHORING-COOKBOOK by its headings; SCENE-PATTERNS §0; the explainer skill's INCIDENTS for the entries cited.
- **Computed by me (scripts outside the repository):** the cmap of every bundled face against every non-ASCII character in the storyboard's picture column; a raster of the bundled outlines for ω, θ, ϑ, α, the fractions and the fallback symbols (FreeType, not the engine's layout); how long each axis the video draws appears under candidate views; the singular values of Video E's oblique projection; the storyboard tool's length formula per chapter (16.1 min, matching the brief); √1.1052; the magnification 15_4 needs.
- **Not done:** I ran no cargo and rendered nothing (the order forbids it). Every mobject count and every cost is an estimate. No TeX string was typeset by the engine: "parses" below means the command is in the pinned command table, not that the layout was seen. The 211 facts are unchecked by me (not this lane), except the handful I recomputed and name.

## Comprehension check: the key picture of each chapter, and whether it can be built

- **01_the_book.** Two projected books (three visible faces each) turned by rotation matrices on trackers beside a violet quiz card whose ring drains for 5 s, then the arm (8 round-capped strokes, 8 discs) under the dim formula: feasible on the retained route as redrawn projected shapes, but not "by placement" (E1); the two end states decide which side the camera stands (E4); the quiz card does not exist yet (E5).
- **02_one_turn.** One book swinging about a gold corner diagonal with an angle readout to 120°, three small pairs collapsing to an axis, then a 3 by 3 grid filling from three edge arrows: feasible; the axis has to be cut where it passes through the book (E3), the readout has to round (E9), the grid is hand-built (E12).
- **03_three_angles.** A book inside three nested rings with three live angles, two axes swinging into one red line, a dial with a shrinking bar, a quote card: feasible; each ring is cut into arcs and depth-sorted with the book (E3, E11), the bar is `kit::live_bar`, the "ways to turn" readout must not move during the countdown (E8), and the picture names ring axes the model does not use (E14).
- **04_halfway.** A circle from above with heading arrows, a number line that bends onto the circle, and a close-up where the circle looks straight: all flat, feasible; the wrap is a computed curve written from an alpha (no morph), and the close-up is redrawn geometry, not a scaled group (E10).
- **05_spin.** An orange arrow through the book with a curled arrow, a one-second clock ring, three edges landing on their neighbours' ghosts, Rodrigues' formula built from a split corner, a curved surface with its flat piece: feasible; the curled arrow is an arc carried through the projection with a tip (E11), the formula is unprobed (E6), the lower third is mathematics (E12), and this chapter is nearest the cap (E17).
- **06_the_base.** A base seen from above driving a half circle with a teal trail, a live pose, a dashed red wrong path and a gold centre, a code panel: feasible and the cheapest chapter, the only one that really is placement-only (E20).
- **07_screws.** A book corkscrewing along a gold axis and leaving a helix, a door, a drawer and a cap, a ghost and its replay about an off-centre vertical axis, a tally and three boxes: feasible; the helix is precomputed in short pieces, shown as the book passes, and sorted with the axis and the book (E3, E11).
- **08_seven_screws.** The arm upright with seven numbered axis lines, curled arrows, the elbow swinging everything past it, and the formula appearing right to left: feasible; the arm, its axes and its arrows are one sorted list (E2), the numbers ride by placement, and the formula is one typeset line revealed by source span (appendix 4).
- **09_by_hand.** The arm with four braces, two factors glowing in turn while the forearm and then the whole arm swing, a live tip readout: feasible; `line_brace` for the lengths, three `kit.live_value` numbers for the tip (E9).
- **10_velocities_add.** Twist arrows at the gripper adding tip to tail, seven columns rising into a 6 by 7 grid, three lever lines, the grid running live: feasible; 10_6 is the heaviest beat of the video (42 live numbers and the arm, E7), and arrows need a guard when they get short (E15).
- **11_lost_directions.** A readout dropping from 6 to 4 as the arm stretches, wiggles with their arrows, a gauge and a bar against a limit line, the wrist in close-up cross-fading to the gimbal: feasible; the live rank spoils the quiz and flickers in the wiggles unless it reads a nominal pose (E8); the close-up is the same world at another scale (E19).
- **12_the_spare_joint.** A human arm in outline with its elbow arc, the robot's elbow travelling a gold circle while "0.000 m" holds, seven arrows cancelling, an obstacle: feasible; `Arm::swing` already gives the joint angles, the circle is an arc through the projection (E11), the cancelling arrows need the guard (E15).
- **13_steps.** The arm over a table with a pink goal, a gold screw axis with its six numbers, a four-box loop with a code panel, three Newton steps beside a table and logarithmic bars, two nearly identical books: feasible, nothing heavy.
- **14_the_straight_path.** Two 3 by 3 grids and their blend, a gripper squashed through the blended matrix, the screw path beside a straight line with a 7.6 cm gap: feasible; a matrix that is not a turn is applied before projecting, which the video's own projection gives for nothing (E19).
- **15_everywhere.** A quadrotor, a map, a 29-joint stick figure, two spinning books with volume readouts, the one-degree leftover magnified, a table, the opening books above the arm, receipts: feasible; three drawings have no data in the model (E16), the swelling is 5% a side and needs a reference (E18), the magnification is about 400 times and has to happen inside the projection (E10), and the chapter sits at the cap (E17).

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| E1 | engine_feasibility | 01_1 features ("3D turns by placement"); every beat whose features say "3D turn", "moving arm", "morph to the gimbal", "ghost", "highlight" on a 3D object | major | **A 3D turn cannot be a placement on the retained route, and nothing done to a redrawn object survives the next frame.** The route refuses any vector point whose z is not exactly 0.0, and any fixed-in-frame, shaded or depth-tested style: `FrameJobError::CameraProjectionRequired` (fmn@pin:crates/fmn-render/src/engine.rs:1523-1531, 1568-1582). So `stage.rotate(m, a, axis, ..)` about anything but `OUT`, `flip`, `Rotating` about a tilted axis, and the kit's `text_hud` and `live_number_hud` all end the render (loudly, which is good). By the same test a half turn about an in-plane axis should fail too (sin π is 1.2e-16 in f64, not 0; read, not run). The director's plan (project in Rust, rebuild) is the right one; the storyboard's features column says the opposite once and a builder reads that column. Second half: a redrawn object is rebuilt every frame, so `fade_in`, `indicate`, `Transform`, `.animate().shift/scale` on it do nothing or flash (E-14, E-22; SCENE-PATTERNS §0.4). | Strike "by placement" in 01_1; write "projected, redrawn from trackers". In `chapters/draw.rs` make z = 0 structural: one `View::screen(p) -> Vec3` that ends in `explainer_kit::v(x, y)` (plot.rs:10), and no other way to make a screen point. Every change to a 3D object is a tracker the redraw reads: pose, opacity (fades, ghosts, the cross-fade to the gimbal at 11_6), highlight level per joint, zoom and centre. A unit test: every point `draw` emits has z == 0.0. Sketch: appendix 1. |
| E2 | engine_feasibility | all 3D beats; sharpest at 01_5, 01_6, 07_2, 07_4, 12_4, 13_1, 13_2, 14_4 | major | **Draw order between scene roots is fixed when they are added; only inside one redrawn family can it change per frame.** The scene list is stable-sorted by `(z_index, position)` on every add and nowhere else (fmn@pin:crates/fmn-mobject/src/stage.rs:815-856); `set_z_index` on a member already in the scene does not re-sort (stage.rs:997-1004); a child's z_index orders nothing (order.rs:52-57); a family draws depth-first in child order (order.rs:20-23). So if the arm is one redraw, the mug another and the screw axis a third, the gripper can never pass behind the axis and in front of the mug: whichever root was added last is always on top. | One redraw root per 3D world: the arm, table, mug, goal marker, screw axis, helix, twist arrows and ghosts of a beat go into one list, sorted back to front, built as one `v_group` inside one `stage.always_redraw` (the family order is the closure's order, fresh each tick: dynamics.rs:237-262). Text, readouts, the quiz card and lower thirds are separate roots added after it, so they sit above; `stage.scene_mut().bring_to_front(&[..])` (runtime.rs:966; Video A small.rs:587) when a world is added late. Two books side by side that never overlap may be two roots. |
| E3 | engine_feasibility | 01_1, 02_1, 02_3, 05_1 to 05_6, 15_4 (a line through the book); 03_1 to 03_3, 11_6 (nested rings); 07_2, 14_4 (helix) | major | **Sorting whole primitives by depth draws three of the video's pictures wrong.** A line through a box is behind it on one side and in front on the other; a ring around the book is in front for half its length and behind for the other half, and three nested rings interleave; a helix winds round its own axis. No order of whole pieces is right for these. Two smaller traps: a sort on nearly equal depths flips from frame to frame (popping), and dimming by opacity makes round caps double up where links overlap at a joint. | (a) Convex solids (the book, a box gripper) are culled, never sorted: draw only the faces whose turned normal points at the viewer, at most three (Video E does this: fair_coin draw.rs:1100-1112). (b) Cut an axis or arrow at the box: clip the segment against the book in the book's frame (slab test), draw the far piece before the faces and the near piece after; the inside piece is left out or drawn dashed and dim. (c) Cut each ring into 8 arcs and sort arcs, book and pivots by the depth of their midpoints. (d) The helix is 8-segment pieces sorted with the axis pieces and the book. (e) `sort_by` with `total_cmp` on a stable order of insertion, so ties never flip. (f) Dim by mixing the colour toward `#0E1116` (`fmn::core::color::interpolate_color`), opacity 1. |
| E4 | engine_feasibility | brief "Three-dimensional pictures"; storyboard "one fixed three-quarter view"; decided by 01_2, 02_1, 02_3, 05_4, 15_5 | major | **The one fixed view is not free: chapter 01's own sentence pins the camera's side, and two obvious choices hide a hero axis.** Computed (parallel view, x right, y away, z up): the covers end facing (0, −1, 0) and (1, 0, 0) (`G-B.*.cover`); a cover facing the right wall is a back face unless the camera stands on the viewer's right. From the right at 32° round and 22° up, a unit length along x, y, z draws at 0.87, 0.62, 0.93; chapter 02's axis (1, 1, −1)/√3 at 0.92; chapter 05's (1, 1, 1)/√3 at 1.00; the cover facing right shows 0.49 of its area and the one facing the viewer 0.79. From the left at the same angles chapter 02's gold axis draws at 0.30 and the right-facing cover is hidden. The isometric view (45°, 35.26°) looks straight down (−1, 1, −1)/√3, which is `G-B.between.axis` and the 90° row of 15_5: that axis becomes a dot. Second: the precedent a builder will copy, Video E's `CubePose::project` (fair_coin draw.rs:885-890), is an oblique projection with singular values 1.00 and 1.15: a turning book would draw up to 15% longer than it is and rings would breathe. Right for a still cube of chances, wrong for a video about rigid turns. | Write the view into the storyboard's 3D paragraph: a parallel projection onto an orthonormal basis, camera on the viewer's right, 30° to 35° round and 20° to 25° up, never 45°/35°. Do not copy E's `project`. Add a test in `draw.rs`: every axis and arrow the chapters draw (x, y, z, the two diagonals, each cover normal) has projected length at least 0.5, and both end covers of 01_2 face the viewer. If 15_5 draws the 90° leftover axis, it shows at 0.30: label it, or do not draw it. |
| E5 | engine_feasibility | all 15 quiz beats (01_1, 02_2, 03_2, 04_2, 05_4, 06_2, 07_6, 08_3, 09_5, 10_4, 11_2, 12_2, 13_4, 14_2, 15_3); 05_3's clock ring | major | **There is no quiz card to reuse, and the kit's listen window cannot play an animation.** No ring, countdown or quiz prop exists in `explainer_kit` or in any explainer: Video F's chapters are all still animatics, and its R1 deferred the card to its build bead (docs/reviews/mental_math/R1.md:174, "a crate-local quiz card on `kit.sonify` and one tween"). `Kit::listen` logs the window and then waits (narration.rs:609-627), so a tween cannot run inside it. `DecimalNumber` with 0 places truncates toward zero (fmn@pin:crates/fmn-library/src/numbers.rs:15-16), so a 5 → 0 tracker shows 4 through the first second. The brief promises "the same card as Videos F and J"; three crate-local copies will drift. | Build one `quiz_card` with F's and J's lanes, in `explainer_kit::props` (question, choices, ring, "pause for more time", `answer()` that lights one choice in place). The ring needs no tween: updaters run on every frame of a wait, in scene order (stage.rs:2258-2276; the kit's own tests rely on it, props.rs:660-700, 724-735), and they see the scene clock (`Stage::time`, stage.rs:647-668; set before updaters run, fmn-scene runtime.rs:1932). So: `kit.hold`, note t0, an `always_redraw` arc whose angle is the time left, a `kit.live_value` digit returning the ceiling, then `kit.listen(stage, n, "NN_chapter:NN_k")`. Sketch: appendix 5. G's countdowns are music alone, so `listen` (not F's `sonify`) is the right window; what the music does inside it is the score lane's to settle (`listen_chord` and `themes` are empty today: lie_robotics score.rs:45-46). Decide once whether the 2 s "?" of `predict_pause` precedes the ring: the length estimate counts both (tools/storyboard.py:22-24). |
| E6 | engine_feasibility | 01_6, 05_3, 05_6, 08_6, 09_2, 09_3, 09_6, 10_3, 15_6; every 3 by 3 and 6 by 7 grid | major | **None of the video's formulas has been typeset at the pin, and its glyph list is empty.** `chapters/glyphs.rs` holds the digits and the title; `TEX` and `MONO` are `&[]`. In the pinned command table `\omega`, `\dot`, `\cdots`, `\sin`, `\cos`, `\neq`, `\cdot`, the `bmatrix` family all parse (fmd@pin:fmd-math/src/commands.rs:198, 259, 448-453, 590, 298, 790-792), and a literal `°` in math is read as an ordinary symbol (parse.rs:300-309). In this repository only `\cdots`, `\cdot`, `\,`, `\tfrac`, nested scripts and `\text` have shipped (raptorq fold.rs:439; bakery fair.rs:567). `\omega`, `\dot q`, `\sin`, `\cos`, `\neq`, a `°` inside TeX and every matrix environment are in no probe and no chapter. From the outlines (my raster, not the engine): ω in the italic face is a clear omega beside a w; θ still draws as ϑ, so the brief's choice of t stands. Risks a sheet will show: the 1 of `S_1` inside an exponent is at second-script size (about 22 px tall at 1080p at 48 pt); the weight of `\cdots` and `≠` (Noto, heavier than CM); where the dot sits on `\dot q`; whether `°` in math sits at the right height. | Put every planned string into `glyphs::TEX`, `TEXT`, `MONO` now and run `./x lie_robotics probe` before P5: the four formulas exactly as the storyboard writes them, `e^{[\omega]}`, `e^{[S_2]\,90°}\, e^{[S_4]\,90°}\, M` and the three-factor line of 09_6, `7 - 6 = 1`, `3 + 1 + 3 = 7`, `(170 + (-170)) \div 2 = 0`, one 3 by 3 grid entry set, and the quiz card's choices at the card's size. Read the sheet at full size for ω against w, the exponent subscripts, the dots. Set the hero line at 56 pt or more. If `°` in math sits wrong, write `90^{\circ}` or keep degrees in Text beside the formula. Colour and reveal by source span, never by a `t2c` needle like `q` (it matches every q). |
| E7 | engine_feasibility | 10_6, 03_1 to 03_3, 02_4, 07_3, 15_2; all moving 3D beats | minor | **No performance cliff found; the idiom is proven at this size but not timed for this shape of work.** Counts (estimates): a book 3 faces + 3 to 5 axis and arrow pieces; the gimbal about 45 small strokes and faces; the arm 8 links + 8 discs + up to 20 axes, arrows and markers; 10_6 adds 42 live numbers (about 210 glyphs laid out per frame). Evidence: Hoeffding's shuffle rebuilds 96 lines and a 16-cell heat map with numbers every frame from one `always_redraw` each (hoeffding_d shuffle.rs:194-218, 231-262), and that chapter rendered 4K60 at about 60 fps on the GPU host at an earlier revision (PERFORMANCE-AND-GPU, the 07_shuffle measurements); S-8 measured 18 live numbers and two maps at 36 to 46 CPU-ms a frame with no growth over 80 s (plan §6.6); Video E's chapters with set-points updaters draft at 200 to 290 fps at 540p30 (its lane's log, `~/jmev_orch/fair_coin/logs/p5_draft_all_1.log`; those frames are mostly still). What is new here: every moving frame makes shapes the plan has never seen. Each is hashed and compiled (plan.rs:36-47, 880-900), old rows pile up, and past 4,096 unreachable shapes the whole plan is rebuilt (plan.rs:102-116, 696-722): at 60 changed shapes a frame that is once every 68 frames. A hiccup, not a cliff, but unmeasured. Strokes cost per pixel by segment count (kit.rs:22-24, S-2): nothing here has a long stroke if E11 is followed. | Before building fifteen chapters on it, one 10 s spike at 4K60 of the heaviest picture (the arm moving through three poses, 42 live numbers, the formula and a lower third): record fps at the start and the end. Fallbacks if it disappoints, cheapest first: skip the rebuild on still frames (the slots idiom of Video E, appendix 1); 10_6's grid as one redraw that builds 42 numbers (as Hoeffding's heat map) or numbers that step at 10 Hz; `Stage::set_placement` on fixed outlines (appendix 1, idiom C). Never one polyline with hundreds of segments under a moving object. |
| E8 | engine_feasibility | 11_1 to 11_3 ("live readout from the pose"); 03_2, 03_3 ("ways to turn") | minor | **Two live readouts answer their own quiz during the countdown, and one flickers.** `model::rank` drops a column under 1e-6 (model.rs:592-596), so a rank read from the arm's joint trackers says 6 on every frame of the stretch and 4 on the last one: the moment the arm arrives, which is inside 11_2's countdown, before 11_3 says "No". The same for "ways to turn: 3 → 2" when the middle ring reaches 90° in 03_2. And in 11_3 "each joint wiggles in turn": wiggling joint 4 or 6 bends the arm, so a live rank jumps to 5 or 6 and back with every wiggle. | Give each such readout its own inputs: a `shown` pose the scene sets at the answer beat (11_3, 03_3), not the drawn pose. Drawn pose = nominal pose + wiggle; the readout and the orange arrows of 11_3 are computed from the nominal pose. Bring the readout in only with the answer, or freeze it through the countdown. |
| E9 | engine_feasibility | 02_3, 03_1, 09_2, 09_3, 09_6, 12_4, 14_4, 15_2, 15_4; every live number | minor | **Live numbers have three small habits that show on screen.** (1) 0 decimal places truncates, and `kit.live_number` does not round (kit.rs:549-557; `live_count` does, kit.rs:615): an angle computed as 119.99999999 reads 119° on its last frame, and "turned 45°" can read 44°. (2) A live minus is an en dash from Computer Modern (numbers.rs:60), a typed "−" in Text is U+2212 from Noto Sans Math: the readout "–0.526" and a static "−0.526" beside it differ in weight and length. (3) `Kit::live_*` have no unit, and `Kit::note` is private (kit.rs:354), so a crate-local "120°" readout is invisible to the layout lint's overlap check. Also 09_2's picture prints the tip as (0.420, 0, 0.886): a three-place live y prints 0.000. | Round inside the closure for every 0-place readout (`kit.live_value(stage, move \|s\| x(s).round(), 0, ..)`), or show one decimal. Build static numbers that sit beside live ones through `DecimalNumber` too, so both use the same minus. A number with a unit is a kit live number with its right edge pinned (`edge = RIGHT`) and a static "°" or "m" placed after it. Tip readout: three `kit.live_value` at 3 places between static brackets and commas; write (0.420, 0.000, 0.886) in the storyboard. Negative zero is already suppressed (numbers.rs:15-17). |
| E10 | engine_feasibility | 04_4, 15_4 ("scaled group as a zoom"); 11_6 (close-up) | minor | **The zoom the features column names does not apply to these pictures.** Scaling a group is the route's only zoom (plan §6.3), and it scales filled arrows and text but not stroke widths, cannot clip (clip planes belong to the camera route, engine.rs:1525-1528), and has no effect on a redrawn family. 15_4 needs far more than a zoom of the frame: a corner 0.15 m from the axis moves 46 µm for 0.0175°, so making the gap 0.15 scene units takes about 3,300 units a metre, 400 times the scale a book is drawn at (I took 8 units a metre). At that scale the faces run thousands of units off the frame and the lint reports them as `outside` (layout.rs:121-145). | Do the zoom where the picture is computed. 3D: `View` carries `scale` and `at` as trackers; tween them; clip faces and lines to the frame rectangle in Rust before building them (convex polygon against a rectangle, a dozen lines). Flat (04_4): redraw the circle as an `Arc` of radius R·zoom positioned so the 170° point stays put, only the part inside the frame, with the orange line and ticks computed at that zoom. If an inset is wanted instead, compute the inset's own drawing and join it to its source with two lens lines (Video B byte_clock.rs:856-859); nothing can be cropped at a window's edge. For 15_4 show the two edges and the gap with a brace, and say on screen "× 400". |
| E11 | engine_feasibility | 03_1 to 03_3, 05_1, 05_3, 07_2, 08_2, 12_4, 14_4; "traced path" at 06_3, 07_2, 12_3, 12_4, 14_4 | minor | **Rings, curled arrows and traced paths are cheaper and truer than "polylines".** A parallel projection is an affine map, and an affine map of a quadratic Bézier is a quadratic Bézier: a ring is `Arc` or `Circle` points (8 curves) carried through rotation and projection with `VMobject::map_points` (vmobject.rs:362), an exact ellipse, against 48 or more straight segments whose cost is paid per pixel per segment (kit.rs:22-24). `TracedPath` (fields.rs:2012-2032) adds an anchor every 1/15 s to one stroke that it rewrites every frame: a 6 s trail grows to 90 segments, each pixel under it paying for all of them, and being one stroke on the screen it cannot be cut and depth-sorted with the book it winds round. All the traced curves here are known in advance. | Ring: 8 `Arc::new().start_angle(k·π/4).angle(π/4)` pieces mapped through the view, sorted (E3). Curled arrow round an axis: one 300° `Arc`, mapped, then `fmn::library::tip::attach_tip(arc, ArrowTip::new().color(c), TipEnd::End)` (tip.rs:40; `curved_arrow` at arc.rs:255 is the flat one). Elbow circle, the base's trail, the screw's helix: build the whole curve once (helix in 8-segment `Polygon::polyline` pieces), then reveal it with `show_creation` at a linear rate matched to the mover, or by setting pieces' opacity as the mover passes. Still geometry, nothing recompiled. |
| E12 | engine_feasibility | 05_3 (lower third `e^{[\omega]}`); 02_6, 10_3, 10_6, 14_2 (grids); 03_5 (quote card) | minor | **Three kit props are narrower than the features column assumes.** `props::lower_third` sets plain text in Computer Modern, at most 3 lines of 92 characters (props.rs:479-518): it cannot set `e^{[\omega]}`. The `Matrix` family exists at the pin (`TexMatrix`, `DecimalMatrix`, `IntegerMatrix`, `MobjectMatrix`: fmn-library matrix.rs:422-666) but has never shipped here, its brackets are drawn about twice too heavy (plan §2.1, fm-u5sb), and it returns one built object, with no handle on a column that rises alone or an entry that runs live. A quote card exists twice, crate-local both times (fair_coin draw.rs:1206; bakery draw.rs:387). All fifteen of the storyboard's text lower thirds fit one 92-character line (counted: the longest is 12_5 at 90). | A crate-local TeX band for 05_3: the label as Text, the body as one Tex, centred on one height (there is no baseline API for mixing, plan §6.5); or move the symbol out of the band and beside the curved arrow. Grids by hand, as the features column already says: cells on a lattice (`kit.tex` still, `kit.live_value` live), two `polyline` brackets (bakery cpu.rs:931). Lift the quote card into `explainer_kit::props` with E's lane, or copy E's. |
| E13 | engine_feasibility | 02_2, 14_3 (¼ ⅓ ½ ¾); 02_5, 05_5 (≠ beside =); 06_1 (wheel senses in the code panel); every label | minor | **Where each glyph comes from at the pin (cmaps read with a script).** Kit text is Computer Modern, not IBM Plex as `chapters/glyphs.rs` says (the kit never selects a family: kit.rs:365; fmn-text font.rs:214). Every non-ASCII character in the picture column has a bundled face, so the animatic and the labels build: ° · ÷ – … are in CM; − ≠ × are Noto Sans Math (heavier, sans); ¼ ½ ¾ ⅓ are IBM Plex Sans only, a sans fraction inside serif text. ✗ is in no face. Shipped before: ½ ¼ (Hoeffding counting.rs:164), ⅓ (capture_recapture everywhere.rs:387), · in lower thirds; ° and ≠ never. CM Typewriter has no U+2212, so the storyboard's "(− + + −)" in a code panel takes its minus from a proportional face and the four columns of signs stop lining up. A red ≠ (Noto) beside a green = (CM) are two weights. | Fractions on the quiz card and at 14_3 as TeX (`\tfrac{1}{3}`, the form C and Hoeffding ship). Code-panel signs as ASCII, straight from `G-P.wheels.*` (which is ASCII). Draw ≠ as the CM = with one slanted `Line` over it, so the pair of panels match. Ticks and crosses: `fmn::library::matchers::checkmark` / `exmark` (bakery draw.rs:802). Correct the comment in `glyphs.rs`. |
| E14 | engine_feasibility | 03_1 (picture) | minor | **The picture names ring axes the model does not have.** The storyboard: "outer about the vertical, middle about the left to right line, inner about the book's own long axis". The model: `rings(yaw, pitch, roll)` is yaw about z, pitch about y, roll about x (model.rs:280-284), and the storyboard's own axes are x right, y away. So the middle ring turns about the line running away from the viewer, and the inner about the left to right line; with the spine on the left (01_1) the book's long axis is y, which is neither ring's. The drawing will come from the model, so the picture text is what is off. Not spoken; nothing in the narration names an axis. | Rewrite the picture cell: "outer about the vertical, middle about the line running away from the viewer, inner about the left to right line", and turn the book for this chapter so its long side lies along x if "its own long axis" is to stay. The lock picture is unharmed: at 90° of the middle ring the inner axis stands vertical, on top of the outer one, in any view. |
| E15 | engine_feasibility | 05_1, 05_2, 10_2, 10_5, 12_5; any arrow seen nearly end on | minor | **A projected arrow can shrink to nothing, and `Arrow` is not built for that.** `Arrow::new(a, b)` is a filled outline whose head is capped at half its length (line.rs:487-510); at zero length its direction comes back as the zero vector and its outline is scaled from a length of 1e-8 (line.rs:616-619; fmn-geom space_ops.rs:163-170). What that draws I did not see: a speck at best. Arrows here shrink to nothing three ways: growing from a point (05_1), cancelling (12_5, "the seven orange arrows cancel to nothing"), and pointing at the viewer (10_5's third arrow is 0.00 m by design). Thickness is in Reference units times 0.015 (line.rs:485): 4.0, not 0.04 (I-01). | In `draw.rs`, one `arrow3(view, from, to, colour)` that returns nothing below 0.03 scene units of projected length (10_5's zero arrow is then a dot with the label "0.00 m", which is the point of the beat) and otherwise `Arrow::new(a, b).buff(0.0).thickness(4.0).style(flat(c, 1.0)).build()` (Video A small.rs:524). Arrows are drawn from the facts, never from a hand-set length. |
| E16 | engine_feasibility | 15_1 (quadrotor, map, the 29-joint figure); 12_2 (human arm); 07_3 (door, drawer, cap); 01_5, 13_1 (mug, table); 06_1 (the base's outline and wheels) | minor | **Nine drawings have no data behind them.** The model holds the arm, the book's turns, the rings, the base's motion; it has no geometry for the Unitree figure (the facts are a count, 29, and a root link's name), the mug, the human arm, the quadrotor, the map, the door, drawer and cap, or the base's outline (the brief rightly shows no chassis dimension). All are easy on the engine (a few faces and strokes each, still or moved by one tracker); the work is authoring, and the risk is a figure that claims more than the facts hold. | Schematics in `draw.rs`, each labelled as one where it could be read as a model: the stick figure with exactly 29 joint dots split 12 + 3 + 14 as the fact sheet's source note gives them, pinned by a test against `G-C.g1.joints`; the base as a rounded rectangle with four hatched wheels and the tag "model". Budget a day for these; they are most of chapter 15's build. |
| E17 | engine_feasibility | 05 (all), 15 (all); 02_5, 06_1, 15_1 | minor | **Two chapters sit near the cap with pictures that need silent time.** By the storyboard tool's own formula (recomputed: total 16.1 min, as the brief says) 05_spin is 79.9 s and 15_everywhere 81.1 s; the formula counts words, breaths, pauses and countdowns, not pictures that outlast their words. 05 has three set pieces that take real time (a one-second spin shown honestly, twice; a formula built from a corner; a surface). Three beats ask one sentence or two to carry six or more reveals: 02_5 (a turn and its undo, three ticks, the word, two panels, a 17-word lower third, in about 8 s of speech), 06_1 (a base, three arrows, two readouts and a seven-line code panel in about 15 s), 15_1 (two scenes, a figure, a tally, a lower third in about 14 s). The cap is pacing, not safety: the kit raises the raw-frame limit to 256 GiB (encode.rs:45-49, 248). | Draft 05 and 15 first among the late chapters and read their lengths. If 05 passes 85 s, split after 05_5 (the spin and its quiz; then the formula and the name). Move 02_5's two panels and count to a beat of their own or into 02_4's hold. In drafts today each quiz beat costs its countdown plus about 2.3 s of "?" (animatic.rs:115-120). |
| E18 | engine_feasibility | 15_2 | minor | **"The left one visibly grows" is true by 5% a side.** The determinant 1.1052 is the growth of a volume; the spin leaves the axis direction alone, so each of the other two directions grows by √1.1052 = 1.051 (recomputed: (1 + 10⁻⁴)⁵⁰⁰ = 1.0513). Five percent over ten seconds on a turning book is under what an eye catches without a reference. | Draw the true book's outline under the swollen one (a thin ghost at volume 1.000), or stand the two books on a ruled mat; let the live readout carry the number. Both books are drawn from the stepped matrices: precompute the 1,001 states once and let the redraw read `step = tracker.round()`. |
| E19 | engine_feasibility | 11_6, 14_3, 15_4; the 3D plan as a whole | delight | **Doing the projection in Rust buys three things the retained route does not have.** A close-up or a pan of a 3D picture is two trackers in the view (the route has no camera move at all, plan §2.1); a matrix that is not a turn (14_3's blend, 15_2's swollen book) is applied before projecting and simply draws as what it is, squashed; and curves stay exact (E11). The model already has what the pictures need: `Arm::skeleton`, `Arm::joint`, `along_screw`, `turn`, `Arm::swing` (model.rs:417, 411, 264, 121, 523), and Video E's cube is a compiled template for the book (fair_coin draw.rs:939-1114). | Keep the view's `scale` and `at` as trackers from the first chapter, so 11_6 and 15_4 are tweens, not new code. If the lit camera-route arm is ever taken up, make it its own segment and cross-fade into it: that route is perspective (raptorq cube.rs:56-66), this view is parallel, so the boundary frames cannot match. |
| E20 | engine_feasibility | 06_3, 06_4, 06_5 | delight | **Chapter 06's picture is its theorem, in one engine call.** "A steady velocity carries the base around a fixed point" is `Rotating::new(base).with_angle(π).with_about_point(centre)` at a linear rate, with `centre` from `G-P.centre`: the base is moved by placement about the gold dot the next beat reveals (Video B hook.rs:751 ships the call). The teal trail is a still `Arc` about the same point drawn by `show_creation` over the same seconds, and the live pose is read back from where the base is (`kit.live_value` over `s.get_center(base)`), so readout, trail and gold dot cannot disagree. | Build 06 early: it needs no projection, exercises the quiz card, a code panel, a live readout from a moving object and a traced path, and renders fast. |

## Appendix: API notes for the builder

Sketches are written against signatures read at the pin and in the kit; none was compiled (no cargo in this lane). Colour functions such as `steel()` and `quiz()` stand for the crate's role table.

### 1. Per-frame redraw

**Viable?** Yes, for 30 to 60 small mobjects a frame. `Stage::always_redraw(f: impl Fn(&mut Stage) -> Mob + 'static) -> Mob` (fmn@pin:crates/fmn-mobject/src/dynamics.rs:237) builds once, then on every update tick calls `f`, swaps the container's children for the fresh family and deletes the old one, whose arena slots are reused (dynamics.rs:246-300; S-8). The container is not rooted: `stage.add_to_scene(m)?`. `f` is `Fn`, not `FnMut`: a cache inside it needs a `Cell`. It cannot return an error: `.expect(..)` on builders. The kit wraps it only for numbers and one bar (`live_number`, `live_value`, `live_count`, `live_bar`: kit.rs:503, 638, 580, 672); for a group, call it directly, as Hoeffding (shuffle.rs:194-218: 96 lines in one `v_group`) and Video B (byte_clock.rs:757-768) do.

**What a changed shape costs.** The retained plan compares revisions per mobject and never reads the points of one that did not change (fmn-render plan.rs:1-19, 842-846). A mobject with new geometry or a new identity is hashed (SHA-256 of its points) and compiled (plan.rs:880-900); a shape equal to one already in the table is interned, not recompiled. A recolour, a z change and any affine placement leave the outline alone (plan.rs:36-47; tests at plan.rs:2295-2330, 2361-2374). Old shape rows are reclaimed by rebuilding the whole plan once 4,096 shapes or 65,536 segments are unreachable (plan.rs:102-116, 696-722). Hard caps: 2²⁰ instances, curves, retained shapes and segments.

**Known cliffs.** Strokes: cost per pixel grows with the stroke's segment count, a long or closed stroke under moving things cost 12 times in S-2, and 322 dense strands fell to 0.38 fps at 4K in S-6 (plan §6.6; kit.rs:22-24). Fills: one flat fill per mobject; per-point fill colours blend into a field and are wrong at the pin (S-3; kit.rs:18-21). Groups: never transform a group of more than 2,000 members per frame (kit.rs:15-17). Arena growth: none measured at the pin (S-8). Thick strokes are not a cliff: a stroke is a distance field, width is a number (fmn-render stroke.rs:11-34); I found no width limit.

**Idiom A, the default: one sorted `v_group` per 3D world.**

```rust
// chapters/draw.rs
use explainer_kit::{BACKGROUND, hex, v};
use fmn::core::color::interpolate_color;
use fmn::prelude::*;

use crate::model::{Arm, M3, Q, V3, add, apply, dot, scale};

/// The one fixed view: a parallel projection onto an orthonormal basis
/// (x right, y away from the viewer, z up).
#[derive(Clone, Copy, Debug)]
pub struct View {
    right: V3,
    up: V3,
    toward: V3,     // from the scene to the camera
    pub scale: f64, // scene units per metre
    pub at: Vec3,   // where the world's origin lands
}

impl View {
    /// Camera `az` round to the viewer's right and `el` above (radians).
    pub fn new(az: f64, el: f64, scale: f64, at: Vec3) -> Self {
        let (sa, ca, se, ce) = (az.sin(), az.cos(), el.sin(), el.cos());
        Self {
            right: [ca, sa, 0.0],
            up: [-sa * se, ca * se, ce],
            toward: [sa * ce, -ca * ce, se],
            scale,
            at,
        }
    }
    /// A world point on screen; z is exactly 0.0 (the route refuses anything else).
    pub fn screen(&self, p: V3) -> Vec3 {
        v(self.at[0] + self.scale * dot(self.right, p), self.at[1] + self.scale * dot(self.up, p))
    }
    /// Bigger is nearer the viewer.
    pub fn near(&self, p: V3) -> f64 {
        dot(self.toward, p)
    }
    pub fn faces_viewer(&self, normal: V3) -> bool {
        dot(self.toward, normal) > 1e-9
    }
}

#[derive(Clone, Copy)]
pub enum Prim {
    /// A flat quad, corners counter-clockwise from outside.
    Face { corners: [V3; 4], color: Srgb, opacity: f64 },
    /// A round-capped stroke; `width` in Reference units (0.01 scene units each).
    Link { a: V3, b: V3, width: f64, color: Srgb },
    /// A ball: a disc of `radius` scene units.
    Ball { at: V3, radius: f64, color: Srgb },
}

impl Prim {
    fn centre(&self) -> V3 {
        match *self {
            Prim::Face { corners: c, .. } => scale(add(add(c[0], c[1]), add(c[2], c[3])), 0.25),
            Prim::Link { a, b, .. } => scale(add(a, b), 0.5),
            Prim::Ball { at, .. } => at,
        }
    }
}

/// One frame of a 3D world: back to front, farther things mixed toward the
/// background (opaque, so overlapping caps never double up).
pub fn draw(s: &mut fmn::mobject::Stage, view: &View, mut prims: Vec<Prim>, depth: (f64, f64)) -> Mob {
    // sort_by is stable: equal depths keep the order they were pushed in
    prims.sort_by(|p, q| view.near(p.centre()).total_cmp(&view.near(q.centre())));
    let back = hex(BACKGROUND);
    let parts = prims.iter().map(|p| {
        let t = ((view.near(p.centre()) - depth.0) / (depth.1 - depth.0)).clamp(0.0, 1.0);
        let dim = |c: Srgb| interpolate_color(back, c, 0.55 + 0.45 * t);
        match *p {
            Prim::Face { corners, color, opacity } => Polygon::new(corners.map(|c| view.screen(c)))
                .style(Style::default().fill(dim(color), opacity).stroke(back, 1.5, opacity))
                .build(),
            Prim::Link { a, b, width, color } => Line::new(view.screen(a), view.screen(b))
                .style(Style::default().fill(color, 0.0).stroke(dim(color), width, 1.0))
                .build()
                .expect("a link has length"),
            Prim::Ball { at, radius, color } => Dot::new()
                .point(view.screen(at))
                .radius(radius)
                .style(Style::default().fill(dim(color), 1.0).stroke(back, 1.5, 1.0))
                .build(),
        }
    });
    s.add(v_group(parts))
}

/// The arm: 8 links through the skeleton's 9 points and a ball at each joint.
pub fn arm_prims(arm: &Arm, q: &Q, steel: Srgb, hero: Option<(usize, Srgb)>) -> Vec<Prim> {
    let pts = arm.skeleton(q); // base, seven joints, tip (model.rs:417)
    let mut out = Vec::with_capacity(16);
    for (i, w) in pts.windows(2).enumerate() {
        out.push(Prim::Link { a: w[0], b: w[1], width: if i < 4 { 20.0 } else { 15.0 }, color: steel });
    }
    for (i, &at) in pts[1..=7].iter().enumerate() {
        let color = match hero {
            Some((j, c)) if j == i => c,
            _ => steel,
        };
        out.push(Prim::Ball { at, radius: 0.13, color });
    }
    out
}

/// A book (half sizes `h`, metres) at orientation `r` and centre `p`: only the
/// faces that look at the viewer, at most three. The cover is the +z face.
pub fn book_prims(view: &View, r: &M3, p: V3, h: V3, cover: Srgb, pages: Srgb) -> Vec<Prim> {
    // corner k: bit i set means the +h[i] side; the faces as Video E lists them
    const FACES: [([usize; 4], usize, f64); 6] = [
        ([0, 2, 6, 4], 0, -1.0),
        ([1, 5, 7, 3], 0, 1.0),
        ([0, 4, 5, 1], 1, -1.0),
        ([2, 3, 7, 6], 1, 1.0),
        ([0, 1, 3, 2], 2, -1.0),
        ([4, 6, 7, 5], 2, 1.0),
    ];
    let corner = |k: usize| {
        let local = [0, 1, 2].map(|i| if k >> i & 1 == 1 { h[i] } else { -h[i] });
        add(apply(r, local), p)
    };
    FACES
        .iter()
        .filter_map(|&(idx, axis, sign)| {
            let mut n = [0.0; 3];
            n[axis] = sign;
            view.faces_viewer(apply(r, n)).then(|| Prim::Face {
                corners: idx.map(corner),
                color: if axis == 2 && sign > 0.0 { cover } else { pages },
                opacity: 1.0,
            })
        })
        .collect()
}
```

In a chapter:

```rust
let view = View::new(32.0 * DEG, 22.0 * DEG, 4.0, v(-2.6, -2.3));
let q: [Mob; N] = std::array::from_fn(|_| stage.add_value_tracker(0.0));
let arm = Rc::new(Arm::iiwa14());
let world = {
    let arm = Rc::clone(&arm);
    stage.always_redraw(move |s| {
        let now: Q = q.map(|t| s.tracker_value(t).unwrap_or(0.0));
        draw(s, &view, arm_prims(&arm, &now, steel(), None), (-1.0, 1.0))
    })
};
stage.add_to_scene(world)?;
kit.say(stage, "08_4")?;
play!(stage; tween(q[ELBOW], 90.0 * DEG, 2.0)?); // everything past the elbow swings
```

`book_prims` never sorts: a convex box is culled (the corner table is Video E's, fair_coin draw.rs:939-946). `draw` as written sorts single primitives by their own centres, which is right for the arm's links and balls. A composite must keep its inner order: sort `(key, Prim)` pairs instead, and give every piece of a composite one key. The sort is stable, so a book with its axis is pushed as far piece, faces, near piece under the depth of the book's centre, the axis cut where it enters and leaves the box (E3), and those five or six pieces stay in that order while the composite as a whole is sorted against the table, the other book and the arrows. Labels that ride on 3D points (the joint numbers of 08_1, the lengths of 09_1) are built once with `kit.text` and moved in an updater with `s.move_to(label, view.screen(p), ORIGIN)`: placement only, no typesetting per frame.

**Idiom B, cheaper where the order never changes: fixed slots written in place, only when something moved.** Video E's compiled code: build the faces once as unit quads, keep an invisible anchor in the scene, and in `stage.add_updater(anchor, move |s, _| { .. }, true)?` (`FnMut`, so it may hold `let mut last`) compare this frame's tracker values with the last ones and return early when they match; otherwise `s.set_points(m, Polygon::new(quad).build().points())` and `s.set_fill(m, None, Some(shade_or_0), None, false)` per face (fair_coin draw.rs:1024-1056, 1076-1114; the square at 566-596). A still book then costs nothing, where idiom A rebuilds (and the plan re-interns) it every frame. `set_points` wants a point run in the engine's quadratic layout, so always pass a builder's `.points()`, never bare corners (positional.rs:451-469). Use B for a lone convex book, A where the order between pieces changes (the arm, the gimbal).

**Idiom C, optional and unproven here: placement after all.** Under a parallel view each flat face, each ring and each straight link is an affine image of a fixed outline, and `Stage::set_placement(mob, Placement::new(linear, translation))` (positional.rs:597; placement.rs:31, row-major, `world = linear · object + translation`) sets any affine map outright with no shape recompile (plan.rs:2295). Keep the third row and column as (0, 0, 1) and translation z at 0 so z stays exactly 0; hide a face whose map is singular instead of drawing it; reorder with `stage.replace_children(holder, &sorted)` (stage.rs:1081). Not needed at this video's size; spike it before relying on it.

### 2. Flat polygons and painter's order

- Scene roots draw in the order of the scene list: a stable sort by `(z_index, position)` run on every `add_to_scene` and nowhere else (stage.rs:815-856). Equal z_index means insertion order.
- `stage.set_z_index(m, z, recurse)` changes nothing until the next add; `stage.scene_mut().bring_to_front(&[m])?` re-adds, `bring_to_back(&[m])?` prepends (fmn-scene runtime.rs:966-971; Video A small.rs:587, 624).
- Inside a root, the family draws depth-first in child order; a child's z_index is ignored (order.rs:20-23, 52-57).
- Inside a redraw the order can change every frame: the closure returns a new family each tick and its child order is whatever the closure sorted (dynamics.rs:252-256). With fixed mobjects, `stage.replace_children(parent, &sorted)` does the same from an updater.
- Flat colour: `Polygon::new(points).style(Style::default().fill(c, 1.0).stroke(c, 0.0, 0.0)).build()` returns a `VMobject` (poly.rs:112-151). Abutting fills show a hairline of background at the default antialias band; Video E gives each face a 1.6-wide stroke in the background colour so the seams read as edges (fair_coin draw.rs:993), Video B narrows the band with `set_anti_alias_width(m, 0.5, false)` (positional.rs:1314).

### 3. Constructors (all in `fmn::prelude` unless a path is given)

| Need | Call | Where |
|---|---|---|
| thick round-capped stroke | `Line::new(a, b).style(Style::default().fill(c, 0.0).stroke(c, w, 1.0)).build()?`; `w` in Reference units, 0.01 scene units each; caps are always round | line.rs:51-112; stroke.rs:23-25; props.rs:288; palette.rs:83 (`stroke_style`) |
| width or colour varying along one stroke | write the `stroke_width` or `stroke_rgba` lane of a polyline-tagged `Mobject` (`m.buffer.write_range("stroke_rgba", 0, ..)`) | props.rs:286-300 (the S-3 strips) |
| open polyline (helix piece) | `Polygon::polyline(points).style(..).build()` or `polyline(points)` | poly.rs:122, 672; raptorq byte_clock.rs:241; fair_coin draw.rs:559 |
| dashed straight line | `DashedLine::new(a, b).dash_length(0.12).positive_space_ratio(0.5).style(..).build()?`; one child per dash, so keep dashes long in anything redrawn | line.rs:233-338; fair_coin draw.rs:702 |
| dashed arc, ellipse, any path | `dashed_vmobject(&source, num_dashes, positive_space_ratio, dash_offset) -> Result<VMobject, DashError>` | vmobject.rs:1107; capture_recapture small.rs:388 |
| arrow with a head | `Arrow::new(a, b).buff(0.0).thickness(4.0).style(flat(c, 1.0)).build()?`: one filled outline; thickness × 0.015 scene units | line.rs:471-616; small.rs:524; bakery draw.rs:863 |
| stroked shaft with a tip | `fmn::library::StrokeArrow::new(a, b)` | line.rs:780-906 |
| arc | `Arc::new().radius(r).start_angle(a0).angle(da).arc_center(c).style(..).build()?` (angle signed) | arc.rs:39-148; small.rs:474 |
| arc through two points | `ArcBetweenPoints::new(a, b).angle(t).style(..).build()?` | arc.rs:170-232 |
| flat curled arrow | `curved_arrow(start, end, angle, style) -> Result<VMobject, GeomError>` (never shipped here) | arc.rs:255 |
| curled arrow round a 3D axis | an `Arc` through `.map_points(\|p\| view.screen(turned(p)))`, then `fmn::library::tip::attach_tip(arc, fmn::library::ArrowTip::new().color(c), fmn::library::TipEnd::End)` | vmobject.rs:362; tip.rs:40; poly.rs:381-455 |
| sector, annulus | `fmn::library::AnnularSector::new().inner_radius(..).outer_radius(..).start_angle(..).angle(..).arc_center(..).build()?` | arc.rs:568-700; small.rs:593 |
| brace on a length | `fmn::library::line_brace(a, b, direction).build()`; `Brace::around(&vmob, dir)` needs a detached `VMobject` | brace.rs:450, 125 |
| tick, cross | `fmn::library::matchers::checkmark(c)`, `exmark(c)` | matchers.rs:265, 287 |
| a move about a point (flat) | `Rotating::new(m).with_angle(a).with_about_point(p)` | raptorq hook.rs:751 |
| a shape driven by an alpha | `fmn::animation::UpdateFromFunc::new_alpha(anchor, move \|s, _, alpha\| { .. })` | bakery draw.rs:364 |

### 4. TeX and text at the pin

| String | Status |
|---|---|
| `T(q) = e^{[S_1]q_1} e^{[S_2]q_2} \cdots e^{[S_7]q_7} M` | every piece parses; `\cdots` and nested scripts shipped (raptorq fold.rs:439); the whole line unprobed |
| `e^{[\omega]t} = I + \sin t\,[\omega] + (1 - \cos t)\,[\omega]^2` | `\omega`, `\sin`, `\cos` in the command table (commands.rs:198, 448-453); never shipped, unprobed |
| `V = J(q)\,\dot q` | `\dot` is an accent (commands.rs:590); never shipped, unprobed |
| `\omega` | from CM italic, as all lower-case Greek (faces.rs:180-195); in my raster of the bundled outline it is a clear omega, rounder than w. `\theta` and `\vartheta` both draw ϑ: keep t |
| degree sign | in Text: in CM, fine by cmap, never shipped. In TeX: a literal `°` becomes an ordinary symbol (parse.rs:300-309), unprobed; `^\circ` takes its ring from Noto (heavier). On a number: `DecimalNumber::unit("°")` is plain text (numbers.rs:27-30, 206-216), not exposed by the kit |
| 3 by 3 and 6 by 7 grids | `TexMatrix`, `DecimalMatrix`, `IntegerMatrix`, `MobjectMatrix` exist (matrix.rs:422-666), square brackets only and heavy (fm-u5sb), never shipped here; `bmatrix` parses (commands.rs:790-792), unprobed. Build by hand (E12) |
| ≠ | Text and `\neq`: Noto Sans Math, heavier than CM's =; never shipped |
| · | CM (and CM Typewriter); shipped in every `names_to_search` band (props.rs:527) |
| ⅓ ¼ ½ ¾ in Text | IBM Plex Sans only; ½ ¼ ⅓ shipped; a sans fraction in serif text. Prefer `\tfrac` |
| − × in Text or TeX | Noto Sans Math in both; consistent with each other, heavier than + and = |
| a live minus | U+2013 from CM (numbers.rs:60) |
| ✗ | no face; `exmark` |
| `\mathfrak`, `\textcolor`, `\not`, `\hspace`, `\texttt`, `\cancel`, `\boxed` | not in the pin's command table (counted: 0 entries each in commands.rs; plan §2.1 calls them build errors): the brief already avoids fraktur; code is `kit.mono`; a box is `SurroundingRectangle` |

Rules that bite: a bare `%` in TeX opens a comment, silently (fmd-math token.rs:59; Video F's E12): "34%" and "9%" at 03_4 stay Text. `t2c` needles are raw substrings. A Tex's children follow TeX's emission order: select them by source span, as Video B does (`kit.tex_spans`, then the children whose span lies in a byte range: raptorq polynomials.rs:37-50, used at fold.rs:440-445). That is also how to build 08_6's line right to left: typeset the finished line once, hide it, and `fade_in` the children of `M`, then `e^{[S_7]q_7}`, then `\cdots` (while joints 6 to 3 light), then the two left factors. Building eight separate formulas would shift the line at every step. The glyph probe only fails on a build error; a wrong look is found by reading the sheet (glyphs.rs:144-157).

### 5. The quiz card

Nothing to reuse (E5). The simplest ring that drains over N seconds with the seconds left in its middle:

```rust
/// The ring and its number, draining over `seconds` while the music plays alone.
pub fn countdown(
    kit: &Kit,
    stage: &mut Stage<'_>,
    at: Vec3,
    r: f64,
    seconds: f64,
    label: &str,
) -> fmn::Result<[Mob; 3]> {
    kit.hold(stage)?; // the question ends first; listen's own hold is then zero
    let t0 = stage.scene().time().to_f64();
    let left = move |s: &fmn::mobject::Stage| (seconds - (s.time() - t0)).clamp(0.0, seconds);
    let track = stage.arena_mut().add(
        Circle::new()
            .radius(r)
            .arc_center(at)
            .style(stroke_style(Palette::faint(), 8.0))
            .build(),
    );
    stage.add_to_scene(track)?;
    let ring = stage.always_redraw(move |s| {
        let f = left(&*s) / seconds;
        let arc = Arc::new()
            .radius(r)
            .arc_center(at)
            .start_angle(PI / 2.0)
            .angle(-TAU * f.max(1e-3)) // clockwise from twelve; never a zero arc
            .style(Style::default().fill(quiz(), 0.0).stroke(quiz(), 8.0, if f > 0.0 { 1.0 } else { 0.0 }))
            .build()
            .expect("an arc of at most one turn");
        s.add(arc)
    });
    stage.add_to_scene(ring)?;
    let digit = kit.live_value(stage, move |s| left(s).ceil(), 0, false, 64.0, quiz(), at, ORIGIN)?;
    kit.listen(stage, seconds, label)?; // a wait: updaters run on every frame of it
    for m in [ring, digit] {
        stage.clear_updaters(m, true);
    }
    Ok([track, ring, digit])
}
```

Read, not run: `Stage::time` inside an updater is the scene clock (stage.rs:647-668; runtime.rs:1932, 1963). If a test shows otherwise, the proven variant is a tracker stepped by `stage.add_dt_updater(anchor, move |s, _, dt| { .. }, false)?` on an anchor added to the scene before the ring (updaters run in scene order: stage.rs:2258-2263); the kit's tests already rely on dt updaters ticking through `wait_frames` (props.rs:660-700, 724-735). The digit is the ceiling, so it reads 5 through the first second and 0 only on the last frame (`.ceil().max(1.0)` if 0 should never show). Fade the ring and digit by name before the answer takes their place; never `indicate` them. 05_3's one-second clock is the same ring filling instead of draining.

### 6. Live readouts from a moving object

```rust
// the tip's x, y, z in metres, from the joint trackers, every frame
let tip = {
    let arm = Rc::clone(&arm);
    move |s: &fmn::mobject::Stage| arm.tip(&q.map(|t| s.tracker_value(t).unwrap_or(0.0))).p
};
for (i, x) in [3.4, 4.6, 5.8].into_iter().enumerate() {
    let tip = tip.clone();
    kit.live_value(stage, move |s| tip(s)[i], 3, false, 30.0, teal(), v(x, 2.4), RIGHT)?;
}
// how many directions, from the pose the scene has declared (E8), not the drawn one
let j = Rc::clone(&arm);
kit.live_value(stage, move |s| rank(&j.jacobian(&shown.map(|t| s.tracker_value(t).unwrap_or(0.0)))) as f64,
               0, false, 44.0, Palette::ink(), v(4.9, 1.2), ORIGIN)?;
// an angle climbing to 120: round, because 0 places truncates
let turn = stage.add_value_tracker(0.0);
kit.live_value(stage, move |s| (120.0 * s.tracker_value(turn).unwrap_or(0.0)).round(),
               0, false, 44.0, blue(), v(4.6, 0.4), RIGHT)?;
let unit = kit.text_left(stage, "°", 44.0, blue(), v(4.66, 0.4))?;
stage.add_to_scene(unit)?;
play!(stage; tween(turn, 1.0, 2.5)?);
```

Signatures: `Kit::live_number(stage, tracker, places, sign, size, color, at, edge)` (kit.rs:503), `Kit::live_value(stage, f: impl Fn(&fmn::mobject::Stage) -> f64 + 'static, places, sign, size, color, at, edge)` (kit.rs:638), `Kit::live_count(stage, tracker, size, color, at)` (rounds, thousands separators: kit.rs:580), `live_bar(stage, tracker, origin, unit, height, color)` (kit.rs:672), `tween(tracker, value, run_time)` (play.rs:49). Each appears at once, at the tracker's present value, when it is made: create it at the moment it should appear. Pin the edge the digits should not move from. A readout from a placed object reads the arena: `move |s| s.get_center(base)[0]`. Clear its updaters before fading it (`fade_all` does this only at the chapter's end: play.rs:61-80).

### 7. The animatic

A chapter is an `Animatic` until its scene exists: `Chapter { key, make: |kit| Box::new(Animatic::new(kit, STORYBOARD, key)), intensity }` (lie_robotics chapters/mod.rs:14-23; animatic.rs:35-49), and all fifteen are registered that way, so `./x lie_robotics draft` (960×540 at 30 fps, scratch voice, layout lint) and `./x lie_robotics sheets` run today. What a chapter looks like (animatic.rs:83-155): the key and title in dim 26 pt at the top; per beat a card with the beat's id in gold at top left, the picture cell as wrapped text (58 characters a line, at most 7 lines, then " …") at 34 pt, and the features cell on one dim line at the foot. Spoken beats call `kit.say(id)` and cross-fade the cards; a ⟨predict⟩ beat adds `props::predict_pause` (a gold "?" at lower right, 0.3 s in and 2.0 s held, marked for the score); `listen N s` becomes `kit.listen(stage, N, "key:id")`; `hold` is a 2.5 s music breath; a silent card with neither waits 2.5 s; the chapter ends with `kit.hold`, `fade_all(0.8)`, `kit.finish`. Every character the cards need is in a bundled face (checked by cmap, not rendered). Replace a chapter by swapping its registry line for a scene struct, as Video E does (fair_coin chapters/mod.rs:21-45); keep the listen labels in the form `"NN_chapter:NN_k"` so timelines made from animatics and from scenes agree. Note that `./x lie_robotics probe` today would probe the cards' prose; the planned strings reach it only through `glyphs::TEXT`, `TEX` and `MONO` (glyphs.rs:105-118).

### 8. Length, cost and features that do not exist

Length: by the tool's formula no chapter passes 85 s (05: 79.9, 15: 81.1, 01: 68.5, 13: 68.4, 03: 67.3; total 16.1 min); see E17 for where pictures will outrun words. Cost: no cliff (E7); the heaviest beats are 10_6, 03_1 to 03_3 and 15_2; nothing needs splitting for cost, and no chapter needs the camera route.

| Features column says | At the pin | Use instead |
|---|---|---|
| 3D turns by placement (01_1) | refused: z ≠ 0 (engine.rs:1568-1582) | projected shapes redrawn from trackers (E1) |
| countdown ring on a tracker | no ring prop; `kit.listen` only waits | an `always_redraw` arc reading the clock (appendix 5) |
| scaled group as a zoom (04_4, 15_4) | scales fills and text, not strokes; no clipping; nothing on redrawn content | zoom in the view or the mapping, clipped in Rust (E10) |
| morph to the gimbal (11_6) | a redrawn family cannot be the source or target of a `Transform` | cross-fade by two opacity trackers in one world |
| morph that keeps identity (04_3, the number line wrapping) | `Transform` between a line and a circle interpolates control points, not curvature | a polyline written from an alpha: an arc of fixed length and curvature α/R (`UpdateFromFunc::new_alpha` + `set_points`), tick labels moved by placement |
| traced path | `TracedPath` exists: one growing stroke, an anchor every 1/15 s, rewritten each frame, not sortable in depth | the known curve built once in short pieces, revealed (E11) |
| TeX lower third (05_3) | `lower_third` is plain text | a crate-local band (E12) |
| matrix | the `Matrix` family, heavy brackets, one object | a hand-built grid (E12) |
| quote card | crate-local in E and C | lift or copy (E12) |
| ghost | no "ghost" object | the same primitives again at a dim colour and opacity below 1, in the same sorted list |
| highlight on a joint or an axis | `indicate` shows nothing on redrawn content; its default yellow floods fills on still ones | a highlight tracker the redraw reads; on still mobjects `indicate(st, m, scale, Some(rgb))` |
| live bar, gauge, code panel, lower third, receipts, names to search, prediction pause | exist: `live_bar` (kit.rs:672); Video A's gauge (small.rs:474-530); `CodePanel::build` and `slide_in` (props.rs:437, 466); `lower_third`, `precisely` (props.rs:485, 521); `receipts_card` (props.rs:532); `names_to_search` (props.rs:526); `predict_pause` (props.rs:215) | as they are |
