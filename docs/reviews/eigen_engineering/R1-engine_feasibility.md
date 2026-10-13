# R1 review (storyboard): eigen_engineering, engine feasibility

Reviewer: an agent given `tools/review_prompts/engine_feasibility.md` (with the
current skill copies) and the R1 packet, 2026-10-13. Verbatim (home paths
shortened).

## What I read, and how each verdict was reached

- **Read in full:** the reviewer prompt; the R1 packet; the plan's §2.1, §6.5 and §6.6; the current skill copies (`SKILL.md`, STATUS-LEDGER, AUTHORING-COOKBOOK, NATIVE-API, TEX-AND-TEXT, PERFORMANCE-AND-GPU); the kit's `kit.rs`, `play.rs`, `plot.rs`, `props.rs`, `animatic.rs`, `glyphs.rs`; all of `explainer_corpus/mental_math/src/chapters/draw.rs`.
- **Not read:** SCENE-PATTERNS.md from the end-to-end skill, which the brief listed. No verdict below rests on it.
- **Read in part:** `fair_coin/src/chapters/draw.rs` (styles, bars, the square, its updaters) and `toss_twice.rs` (first 260 lines); the signatures of K's `src/model/*.rs`.
- **Engine source at the pin** (cargo's git checkout of franken_manim at `a399d62`): `fmn-text/src/shape.rs` and `font.rs`, `fmn-library/src/numbers.rs`, `line.rs`, `poly.rs`, `fmn-mobject/src/positional.rs`, `dynamics.rs`, `fmn-anim/src/indication.rs`.
- **Math engine:** the pin uses franken_markdown `126a5304`, later than the skill docs' `3ed1d332`. I read `fmd-math/src/commands.rs` and `faces.rs` there. The bundled faces are the same set as before (CM Unicode regular, italic, bold, bold italic, typewriter; IBM Plex Sans; a Noto Sans Math subset), with no CM math faces, so the plan's glyph facts still apply.
- **Glyph coverage:** I parsed the `cmap` table of each bundled font with a small script and listed every non-ASCII character in the storyboard's picture and features columns against the Text fallback chain (CM, then Noto Sans Math, then IBM Plex Sans).
- **Pixels seen:** two images only. Video F's draft contact sheet for chapter 01 shows the quiz card with its draining ring and numeral. Video E's glyph probe page shows `− ×` visibly heavier than `+ ÷ =` in Text.
- **Not done:** no cargo, no render, no timing. Every cost figure is an extrapolation from the plan's spike numbers and is labelled so. Facts and numbers are unchecked; they are not my remit.

## Key picture per chapter

| ch | key picture | feasible? |
|---|---|---|
| 01 | A slender deck from above with walking dots, then swaying on a tracker beside a 70 mm ruler | With a change: 150 to 250 dots in the kit's `Swarm`, no outlines under them (E20) |
| 02 | Two carts, three zigzag springs, push arrows, the 2×2 table read as two sums | With a change: springs by one guarded updater (E17); the quiz card needs a generic form (E7) |
| 03 | The arrow plane: blue sweeps, orange is K times blue every frame, an angle readout, gold lines at 45° and 135° | As written, once the readout rounds (E4) |
| 04 | The orange arrow splits along the gold lines, each piece is divided, the pieces add tip to tail | As written, with ½ and ⅙ set in TeX (E11) |
| 05 | The truss draws itself, a 21×21 table fills member by member, the truss deforms ×200, members take colour and weight | With a change: sparse cells (E13); the truss's frame and paint on trackers (E3) |
| 06 | A ladder of 21 stiffnesses; load shares divide and turn blue; halves and sixths become letters | With a change: a true log ladder merges rungs (E5); the morph needs shared TeX source (E16) |
| 07 | Truss modes swinging at their own rates with mass discs, beside K φ = ω² M φ | With a change: a dt clock (E23) and `\varphi` (E8); up to 132 small paths rewritten a frame, unmeasured |
| 08 | A deck end-on twisting with wind arrows that follow the angle; two pulses out of step; a walker counter | As written |
| 09 | Air slices become carts and springs, then standing waves; a 40×40 table; Helmholtz's sentence | With a change: 40×40 as cells, 6 to 8 drawn springs (E10) |
| 10 | A floor heat map per mode, a histogram with a count-up to 1,964,040, peaks that merge at 20 Hz | With a change: one summed curve in short pieces (E6); probe the formula (E9) |
| 11 | Rays in the hall, 65 peaks, three tick lines, a 306-cell section with three mode maps | With a change: rectangular strips with the absent cells in the background colour (E14) |
| 12 | Icons and arrows fold into a 2×2 table; 70/30 bars; the arrow plane on a table that is not symmetric | As written |
| 13 | Stacks of rounds closing on a dashed line at 80; a red stack leaving the frame; a 44×44 grid | With a change: grid as lines (E22); the lower-third in TeX (E2) |
| 14 | The factory plane: three lines, a green region, corners, a sliding line of equal profit, a dot on the edges | As written; the sliding line's ends are recomputed (E21) |
| 15 | Turn, stretch, turn back on a grid of arrows; a fan of 21 patterns; shear and quarter-turn planes; the bridge again | With a change: arrows rebuilt, thumbnails as single mobjects (E19) |

## Beat verdicts (non-trivial pictures)

Extrapolations use these spike figures from plan §6.6:
- S-8: two 3×3 maps plus 18 live numbers cost 36 to 46 CPU-ms a frame.
- S-4: 4,845 moving dots ran at 11.5 fps at 4K.
- S-2: 1,000 fish cost about 700 CPU-ms a frame; long strokes under moving things cost 12×.
- S-3: 255 strips of 255 cells over 2,040² px cost 6.6 CPU-s a frame at 4K, growing with cells per strip.
- S-6: 322 dense strands ran at 0.38 fps at 4K.

| beat | asks for | verdict | mobjects and per-frame work |
|---|---|---|---|
| 01_1, 01_2, 08_5, 08_6, 15_7 | dots walking on a deck that sways | with a change (E20) | about 200 dots moved by placement plus one deck polygon rewritten; roughly a fifth of S-2's 1,000-fish load |
| 01_5 | two counters in step | as written | two live integers off one clock; truncation at 0 places is the right behaviour for "swings completed" |
| 01_6 | three small moving pictures | as written | one small truss (33 paths), a 6×10 strip map, two bars |
| 02_1 to 02_3, 02_7, 03_2, 04_4 | carts sliding, springs redrawn, stretched thin and squeezed thick | as written; better by updater (E17) | 3 zigzags of about 25 segments; S-8 class |
| 02_5, 12_2 | table from primitives, rows light | as written | about 10 mobjects; row light is a translucent band, not `indicate` (E12) |
| ten quiz beats | violet card, draining ring, seconds numeral | with a change (E7) | ring is one 64-step polyline rewritten a frame, up to 6 numerals toggled; negligible |
| 03_3 to 03_5, 12_6, 15_5 | orange arrow recomputed from a sweeping blue one, live angle | as written (E4, E18) | blue by `rotate`, orange one mobject rebuilt, one readout; two planes in 15_5 |
| 04_2 to 04_4 | arrow splits, pieces shrink, add tip to tail | as written | arrow-to-arrow transforms keep the same point count |
| 05_1 | truss draws itself, counters to 12 and 21 | as written | 21 lines, 12 dots, 2 supports, lagged `show_creation` |
| 05_3 | 441 cells filled member by member, mirror flash | with a change (E13) | 97 flat cells plus 44 lattice lines; at most 16 cells change per member |
| 05_5, 05_6 | deformed shape ×200, every joint's arrow grows | with a change (E3) | about 60 small paths rewritten a frame for about 2 s |
| 05_7 | members recoloured and re-weighted by force, gauge | with a change (E3) | 21 stroke colour and width writes from one tracker; no transforms on members |
| 06_1, 06_3 | ladder of 21 bars; patterns play | with a change (E5) | 21 bars static; one small truss moving (33 paths) |
| 06_4 | five arrows become 21 bars, bars divide and turn blue, letters arrive | with a change (E16) | 21 rectangle transforms; about 30 glyph transforms |
| 06_5, 06_6 | shape rebuilt from k patterns, two readouts | as written (E4) | two overlaid trusses, a step tracker k, two live numbers |
| 06_7 | a ladder of a million | with a change (E22) | at most 200 hairlines in at most 10 mobjects, static |
| 07_2, 07_3, 07_5, 07_6 | modes swinging, each at its own rate | with a change (E23) | 33 paths per truss; 99 at three trusses, 132 at four; every frame dirty in those regions; my guess is 2 to 3 times S-8, unmeasured |
| 08_1 to 08_3 | deck twisting end-on, arrows follow the angle, two pulses | as written | cross-section by `rotate`; 3 to 5 arrows rebuilt |
| 09_1 to 09_3 | slices crowd and thin; slices become carts; three standing waves | with a change (E10) | 120 thin rectangles moved by placement in 09_3; 40 lagged transforms in 09_2 |
| 10_1 | floor map recoloured as nx, ny, nz step | as written (E14) | 20 strips of 30 cells; about 0.2 CPU-s a changed frame at 4K, scaled from S-3 |
| 10_4 | histogram rising under a marker, count-up to 1,964,040 | as written (E4) | about 40 bars, 9 glyphs rebuilt a frame |
| 10_6, 11_2 | thousands of peaks that merge | with a change (E6) | 60 to 80 short stroke pieces plus one fill, static once drawn |
| 11_1 | rays bouncing | as written | 5 to 10 precomputed bounce polylines, `show_creation` |
| 11_5 | 306 cells, three mode maps | with a change (E14) | 3 × 12 strips; about 0.3 CPU-s a changed frame at 4K, scaled from S-3; static if not animated |
| 13_2, 13_5 | stacks of blocks, one leaving the frame | as written | up to 13 blocks a stack; keep registered labels inside the frame for the lint |
| 13_7 | 44×44 grid | with a change (E22) | 90 lines, static |
| 14_2, 14_3 | lines, filled region, moving line, dot on the edges | as written (E21) | about 20 mobjects; one 2-point line rewritten |
| 15_1, 15_2 | turn, stretch, turn back; fan of 21 patterns | with a change (E19) | 16 to 25 arrows rebuilt a frame; 21 single-mobject thumbnails, static |

**Budget.** Nothing in the storyboard approaches a spike's cost cliff if E6, E10, E20 and E22 are followed. The largest group is under 500 members; the kit's limit is 2,000. The video has no camera-route segment.

## Findings

| id | persona | chapter/line | severity | finding | suggested fix |
|---|---|---|---|---|---|
| E1 | engine_feasibility | 06_4, 07_1, 13_3, 15_1, 15_3, 15_6 (and 05_3) | blocker | Five characters in the picture column (ᵀ ᵏ ᵢ ⱼ ⁻) are in no bundled face. `Text` returns `UnmappedChar` for such a character (`fmn-text/src/shape.rs`), and the animatic passes the picture text to `Kit::text`. By my reading of the code, the animatics of chapters 06, 07, 13 and 15 cannot build; 05_3's ᵀ falls after the card's seven-line cut. Not run. K has no `renders/` directory, so nothing has hit it yet. Videos G and J each have one such row. | Write those formulas in the picture column as TeX source or ASCII (`Q^T`, `V^-1`, `q_i`, `C^k`). In real scenes they are `Tex` only. Optionally have the kit's `animatic::plain` transliterate. |
| E2 | engine_feasibility | 03_6, 05_3, 07_1, 07_6, 13_3 | major | These lower-thirds carry formulas, but the kit's `lower_third` sets plain Text wrapped by character count. In Text, ² comes from IBM Plex and ᵀ ᵢ ⱼ ⁻ fail. The only TeX lower-third is `lower_third_tex` in F's `draw.rs` (a label plus one TeX line), and that file is untracked in the working tree. 07_1 mixes a sentence with two formulas, which needs `TexText`; no scene in this corpus has probed it. | Move `lower_third_tex` into the kit. For 07_1, keep the sentence in the lower-third and put φᵢᵀMφⱼ = 0 on stage under the main equation. For 05_3's loop, use the kit's `CodePanel` in mono with ASCII: `K[dofs] += k * outer(g, g)`. |
| E3 | engine_feasibility | 05_5 to 05_7, 06_1, 07_2 to 07_6, 01_6 | major | The truss "redrawn from the solved moves times a tracker" works through E's and F's idiom: fixed mobjects, one updater, `set_points`. But `set_points` resets a mobject's placement to identity (`positional.rs`). Any `.animate().shift` or `.scale` on the truss or its group ("the truss shrinks to the left", 06_1) is undone at the next write. A colour transform on a member while its tracker moves fights the updater. Animating one member splits a rooted group. | Build one `Truss` prop whose origin, unit, shape amplitudes, force blend and lit member are all trackers read by one guarded updater that writes points, stroke colour and stroke width. Move and shrink by tweening origin and unit, as E's `SquareView` does. Fade explicit `mobs()` lists. The same applies to the cart rig. |
| E4 | engine_feasibility | 03_3, 03_4, 06_6, 08_6, 10_4, 14_3 | major | `DecimalNumber` truncates at 0 places (`numbers.rs`, `format_number`), and `Kit::live_number` and `live_value` pass the value straight through. An angle of 29.98° reads "29" where the caption says 30. A force error that the fact rounds to 82 would read 81 if the raw value is below 82. A profit of 459.9999 from the dot's position reads 459. A count interpolated through N(f) can read 2,245. Negative values print an en dash from CM, while static strings use U+2212 from Noto Sans Math, so two different minus signs can sit side by side in 02_3 and 03_2. | Round in the closure to the fact's precision before `DecimalNumber`. Use `live_count` (it rounds) for integers. Show signed values as static strings built from facts. The kit owner could make `live_number` round. |
| E5 | engine_feasibility | 06_1, 06_3, 06_4 | major | A ladder "on a scale of stiffness" cannot show 21 separate rungs. From `K-P1.mn_per_m`, 340.18 and 355.17 are 0.019 decades apart, and 418.32 and 430.55 are 0.013. On a 6-unit ladder spanning the 2.59 decades from 3.12 to 1,226, that is 0.043 and 0.029 units: 12 and 8 px at 4K, 3 and 2 px in the 540p draft. The rungs merge. | Space 21 rows evenly by rank and make each bar's length the log of its stiffness. Put the load-share bars in a second column on the same rows. "Soft at the bottom" and "× 393" survive. |
| E6 | engine_feasibility | 10_6, 11_2, 15_5 | major | "The frequency line fills with peaks … until they merge" must not be thousands of peak mobjects or one stroke with thousands of segments. The stroke shader tests every segment of a shape at each pixel it shades (S-6, S-2). | Cheap and honest: sum the response of the true modes from `Room::modes_below` (29 below 20 Hz, a few hundred by 60 Hz) at about 1,000 sample points. Draw the sum as 60 to 80 polylines of at most 16 segments, revealed left to right with lagged `show_creation`. Fade a flat fill under the curve to the right of the 20 Hz marker for the wash. End the axis near 60 Hz, not 1,000. The same helper draws 11_2's 65 peaks. |
| E7 | engine_feasibility | all ten quiz beats | major | A quiz card with a draining ring and a seconds numeral exists only in F's `draw.rs` (`QuizCard`, `your_turn`); I saw it rendered in F's chapter-01 draft sheet. It is bound to F's model: one TeX problem, a TeX answer that lands where the ring was, a 5.6 × 4.1 card. K's cards hold a table with two lists, a truss, two small trusses, three rooms, blocks and a plane's corners, and the next beat works the answer. The kit has `predict_pause` and no countdown card. | Put a generic card in `explainer_kit::props` (violet edge, ring, numeral, "pause for more time", `sonify` countdown, a content slot, no landing answer). G, H, J and K declare the same device. Until then K copies F's ring code (about 120 lines) into its own `draw.rs`. |
| E8 | engine_feasibility | 07_1, 07_6 | major | `\phi` maps to U+03D5 (`commands.rs:194`), which CM italic lacks, so it resolves to CM Regular: an upright ϕ among italic letters. `\varphi` maps to U+03C6, which CM italic has. This is read from code and the fonts' `cmap` tables, not seen rendered. | Probe `\phi`, `\varphi` and Text φ side by side first. Expect to write `\varphi` everywhere, including φᵢᵀMφⱼ. Also put ω beside a Latin w on the sheet. |
| E9 | engine_feasibility | 10_1 | major | The room formula is the most fragile string: a tall drawn radical over three `\left( \right)` fractions with squares, with subscripts at the smallest script size. TEX-AND-TEXT calls `\sqrt` rows the box oracle's worst outliers and drawn big delimiters about twice too heavy (fm-u5sb). "Words under Lx, Ly, Lz" as `\underbrace` inside the radical adds a third nesting level. The brief forbids f for frequency. | Probe `\text{tone} = \frac{c}{2}\sqrt{\left(\frac{n_x}{L_x}\right)^2 + \cdots}` at its final size (52 to 56 pt). Place "length, width, height" as separate Text under the positions of `occurrences("L_x")`. Fallback: three stacked terms without `\left` and `\right`. |
| E10 | engine_feasibility | 09_2 | major | "The table … growing to 40 by 40" with numerals gives entries 0.1 units wide (7 px in the draft): unreadable, and 1,600 text mobjects if the zeros are set. Forty carts with 41 zigzag springs across the frame are 0.16-unit scribbles, about 700 stroke segments rewritten a frame. | Show numerals up to 6×6. Then grow to 40×40 as 118 flat cells (40 + 39 + 39 non-zero), scaled in once as one group. Morph 6 to 8 slices into carts with real springs, and draw the row of 40 as blocks with straight links. |
| E11 | engine_feasibility | 04_2, 04_3, 05_2, 05_7, 10_2, 11_1 | major | In Text, ² ³ ¹ ½ ⅙ exist only in IBM Plex Sans, so "6,000 mm²", "N per mm²", "18,630 m³", "1,579 m²", "½ of 1, 1" and "½ ÷ 3 = ⅙" would show a sans glyph inside a serif label. S-7 found the same for Video B's polynomials. | Set them in `Tex`: `6{,}000\ \text{mm}^2`, `\tfrac12`, `\tfrac16`. Add a small `unit_tex` helper to `draw.rs`. |
| E12 | engine_feasibility | 02_5, 02_7, 03_2, 05_2, 05_3, 08_7, 09_6, 11_4, 12_4, 12_5 | major | Ten beats say "highlight", "pulse", "glow" or "lights". `indicate(…, None)` recolours to `#FFFF00` and floods any fill (`indication.rs`). A fade-in and an `indicate` on one mobject in one play leave it invisible. | Text: `indicate` with the role's own colour, in a play after the fade. Strokes: tween the stroke colour, or use the kit's `glow`. Filled shapes (cells, bars, cards): a scale-only swell or an outline box, never `indicate`. Lit truss members go through the truss's tracker (E3). |
| E13 | engine_feasibility | 05_3 | minor | "441 fixed squares recoloured in place" is feasible, but 344 of them never change (`K-T2.nonzero` is 97). 441 outlined squares would be 441 closed strokes. "Darker for stiffer" disappears on a dark ground. The cell packs about nine events into one two-sentence line. | Use 97 flat cells with no outline, addressable by row and column, over a lattice of 44 lines. Make stiffer cells brighter and more opaque. Drive the 21 member steps with one tracker over about 5 s. Mirror flash: a passing flash along the diagonal, then an opacity pulse of mirrored pairs. |
| E14 | engine_feasibility | 10_1, 11_5 | minor | The kit's `RowStrips` is the right tool, but `RowStrips::build` refuses rows of unequal length, and the section is 30 × 12 with 54 cells absent. On a floor plan, stepping nz changes nothing. Deep teal `#26867C` at half strength is close to the background. | Use 12 strips of 30 with the absent cells in the background colour, drawn first, or add a mask to `RowStrips`. Add a side strip for nz. Run the scale teal, then a neutral lighter than the ground, then magenta, and check it on the sheet. Cut the hall's outline into short open pieces if the map under it changes. |
| E15 | engine_feasibility | 06_4, 13_3, 15_1, 15_3, 15_6 | minor | ∑, ⋅, ×, ≈, ⊤ and the math minus come only from Noto Sans Math, a heavier sans (E's probe page shows it for − and ×). `^\top` would be a heavy sans tee. The minus in `V^{-1}` is at script size. | Probe `Q^{T}`, `Q^{\mathsf T}` and `Q^\top` side by side and pick one. Probe `\sum_i \frac{q_i\cdot f}{\lambda_i}` at display size (Hoeffding shipped `\sum_i`). Accept the weight or bump the size by 4 pt. |
| E16 | engine_feasibility | 06_4 | minor | `TransformMatchingTex` pairs equal source slices. "½ ÷ 3 = ⅙" and `u = \sum…` share almost none, so the morph would be a cross-fade. The carts' line also belongs to another scene. | Re-set the carts' answer in chapter 06 in fraction form, then chain three forms with shared slices: numbers in `\frac{…}{…}`, then `\frac{q_1\cdot f}{\lambda_1}q_1 + \frac{q_2\cdot f}{\lambda_2}q_2`, then the sum. Take span maps with `Kit::tex_spans`. Never index TeX children by number. |
| E17 | engine_feasibility | 02_1 to 02_3, 09_2 | minor | Springs by `always_redraw` work, but they rebuild every frame even at rest, cannot be faded without clearing updaters, cannot be the target of 09_2's morph, and cannot have their stroke width animated. Zigzag corners grow acute when squeezed, under the default `JointType::Auto`. | Use fixed polylines written by one guarded updater (`set_points` plus `set_stroke` width), as F's `Strip` does. Check the most squeezed spring on a 4K still; if corners spike, use `with_joint_type(Bevel)` or a smooth coil. |
| E18 | engine_feasibility | 12_6, 15_5 | minor | `Arrow` narrows its shaft and tip when short (`line.rs`, `key_dimensions`: width capped at a tenth of the length). In 12_6 the orange arrow is 0.2 to 0.75 of the blue one, so it would thin and thicken through the sweep. | Raise `max_width_to_length_ratio`, or pick the plane's unit so the shortest orange arrow is at least ten shaft widths long. One `ArrowPlane` helper takes the matrix. |
| E19 | engine_feasibility | 15_1, 15_2 | minor | "Stretch each by its eigenvalue" as a non-uniform scale distorts arrow heads. The fan of 21 patterns as full trusses is 693 records. | Rebuild the arrows from the transformed vectors off a tracker. Draw each thumbnail as one multi-subpath VMobject, static (`Truss::still`), and use the same for 06_1 and the quiz cards. |
| E20 | engine_feasibility | 01_1, 01_2, 08_5, 08_6, 15_7 | minor | The walkers are the video's only particle load. Under moving things, long or closed strokes cost 12× in S-2. | Use 150 to 250 single dots in the kit's `Swarm`, one flat fill each. Deck and river as flat fills with no outline; cables cut into short pieces. Do not draw `K-B2.on_deck`'s 2,000. |
| E21 | engine_feasibility | 14_3 | minor | I found no clip or mask on the retained route in NATIVE-API. A long line of equal profit shifted outward would run past the plot window. | Recompute the line's two ends against the window from the profit tracker (one 2-point `set_points`). The region is one flat polygon with no outline, and the three limits are separate lines. |
| E22 | engine_feasibility | 13_7, 06_7 | minor | A 44×44 grid as cells is 1,936 mobjects, 64 under the kit's group limit, to show an empty table. "A ladder of a million" cannot be literal. | Draw the grid as 90 lines. Draw the long ladder as at most 200 hairlines in at most 10 mobjects, with a break mark and the number. |
| E23 | engine_feasibility | 07_2 to 07_6, 01_5, 08_1 to 08_3 | minor | Swings that run across several plays and waits cannot be tracker tweens inside each play. | One dt-driven clock (the pattern of `Swarm::drive`) with a rate tracker to start and stop it; every swinging prop reads it. Slow 5.9 : 11.8 : 14.9 Hz to about 0.5, 1.0 and 1.26 on screen so 30 fps drafts stay smooth. Never fade or recolour by transform while the clock is rewriting a shape. |
| E24 | engine_feasibility | 01_5, 02_2, 09_6, 12_1, 13_1 | minor | Hands, a walker, a humming figure, a field, a workshop and a household are icons. `ImageMobject` is camera route only (plan §2.1). The library has an SVG module that no scene in this corpus has used. | Draw them from primitives (a few polygons each), or probe the SVG import on one icon before relying on it. |
| E25 | engine_feasibility | glyph registry, 01_7 | minor | K's `glyphs.rs` has an empty `TEX` list, and its comment says plain text is IBM Plex Sans; `Kit::text` sets no font, so it is Computer Modern (`DEFAULT_FAMILY`). The 56-character title at 44 to 48 pt is, by my arithmetic, about 13 units wide in a 13.9-unit ink area. | Put every string of the table below into `glyphs::TEX` and `TEXT` now and run `./x eigen_engineering probe` before any chapter is built. Break the title over two lines. |
| E26 | engine_feasibility | 15, 02, 05, 09, 12; 05_3, 14_2 | minor | The brief says scratch drafts ran about 9% over the estimate on Video E. Chapters 15 (78 s), 02 and 05 (76 s) and 09 and 12 (75 s) would then land at 82 to 85 s, on the editorial limit. No engine cap is hit: the kit raises the raw-input cap to 256 GiB. 05_3 and 14_2 each ask for eight or more reveals inside one short line, with few phrases to hang them on. | Name a split point now (15_4's hold; 05_4's quiz). Move 14_2's plane into the second half of 14_1. |
| E27 | engine_feasibility | 10_4, 11_5 | minor | `Room::modes_below(1000)` would return 1.96 million tuples. `hall_section()` runs a Jacobi sweep on a 306×306 table (it is cached in a static). Every render and every glyph probe of those chapters pays for whatever `construct` computes. | Build the histogram from `count_below` at the band edges, cached. Call `modes_below` only up to 60 or 100 Hz. Keep model calls out of updaters. |
| E28 | engine_feasibility | whole video | delight | Every picture is flat, so the video has no camera-route segment (6.1 fps at 4K60 in S-5), no two-route join, and one encoder path. The storyboard's own engine notes already choose fixed mobjects recoloured in place. | Keep it so; do not add a 3D room. |

## Every TeX and special-character string (item i)

Faces are from the pinned `commands.rs`, `faces.rs` and the fonts' `cmap` tables. "Probe" means it must be seen on a sheet before a chapter depends on it.

| on screen | write as | where the glyphs come from | risk |
|---|---|---|---|
| K u = f; K q = λ q | `K u = f`, `K q = \lambda q` | letters and λ from CM text italic | low |
| K = Q Λ Qᵀ | `K = Q \Lambda Q^{\mathsf T}` (or `^{T}`) | Λ upright CM; ⊤ only in Noto Sans Math | probe three transposes (E15) |
| u = Σ (qᵢ·f/λᵢ) qᵢ | `u = \sum_i \frac{q_i\cdot f}{\lambda_i}\,q_i` | ∑ and ⋅ from Noto Sans Math | probe; heavier operator (E15) |
| K φ = ω² M φ; φᵢᵀMφⱼ = 0 | `K\varphi = \omega^2 M\varphi`; `\varphi_i^{\mathsf T} M \varphi_j = 0` | `\varphi` from CM italic; `\phi` falls to upright CM Regular | probe first (E8) |
| k = EA/L and its numbers | `k = \frac{EA}{L}`; `200\ \text{GPa}\times 6{,}000\ \text{mm}^2 \div 4\ \text{m}` | × from Noto, ÷ from CM | low; units in `\text{}` |
| C = V Λ V⁻¹; Cᵏ = V Λᵏ V⁻¹; x = (I − C)⁻¹ d | `C^k = V\Lambda^k V^{-1}`; `x = (I - C)^{-1} d` | minus from Noto at script size | probe (E15) |
| √3 ≈ 1.7 (not in any beat yet) | `\sqrt{3} \approx 1.7` | radical drawn; ≈ from Noto | low |
| ½, ⅙ | `\tfrac12`, `\tfrac16` | CM digits | low in TeX; in Text they are IBM Plex (E11) |
| room formula | see E9 | drawn radical, three tall paren pairs | highest; probe at final size |
| "Eigentöne", "…hat gewisse Eigentöne", "Eigenwerte" | Text, italic in a quote card | ö and … are in CM regular, italic and typewriter | low |
| ÷ · ° – “ ” < in Text | Text | Computer Modern | fine |
| × − ≈ → in Text | Text | Noto Sans Math, visibly heavier than CM's + ÷ = | accept, or set in TeX |
| ² ³ ¹ ½ ⅙ in Text | use TeX | IBM Plex Sans | E11 |
| ᵀ ᵏ ᵢ ⱼ ⁻ in Text | never | no face | build error (E1) |

**Known-bad Greek at the pin.**
- θ draws as the cursive form, ρ as ϱ, `\epsilon` equals `\varepsilon`, and α reads as a Latin a (plan §2.1).
- The alternates table in `faces.rs` maps ϑ→θ, ϱ→ρ and ϵ→ε whenever no face carries the variant codepoint.
- K's picture and features columns use none of θ, ρ, ε or α; I checked by listing every non-ASCII character in them.
- λ, Λ, ω and Σ are on no bad list. φ is the open question (E8).

**Probe order:** φ forms; transposes; the sum; the room formula; `^{-1}`; the halves and sixths; units with squares; the Text rows with × and −; a `TexText` line with islands if 07_1 keeps its mixed lower-third.

## Build order

**Before any chapter**
- Fill the glyph registry and read the probe sheet (E25). Fix φ and the transpose form for the whole video.
- Gate each real scene behind a cargo feature (`ch01` … `ch15`) as F's `chapters/mod.rs` does, so several authors can build side by side while the rest stay animatics.

**`draw.rs` core, one author, lands first**
1. Staging copied from E and F: `say`/`Spoken`/`wait_for`, `fade_in_all`, `fade_out_all`, `swap`, `tween_all`, `tween_linear`, `flat`, `line_style`, `rgb`, `group`, the caption slot, `tag`, `QuoteCard`, `lower_third_tex`. E, F and K would each carry a copy, so the kit owner may prefer to take them.
2. Role colours with the ΔE test, plus `unit_tex` and rounded number formatting from facts (E4, E11).
3. `Clock`: dt-driven time and a rate tracker (E23).
4. `QuizCard` with a content slot, and the plain question card (E7).
5. `Table2`: numerals, brackets from lines, a row band, headed lists. Used in 02, 12, 15 and four quiz cards.
6. Plot pieces on `Frame2`: an axis with ticks (linear and log), a number line with marks, `curve_pieces` (short strokes, E6), bars and stacks, the ranked ladder (E5), a line clipped to the window (E21).

**Then in parallel**
- **Act I (02 to 04):** the `Carts` rig (n carts, frame on trackers, springs, push arrows) and `ArrowPlane` (any 2×2 rule, angle tracker, rounded readout, gold and grey lines). Both are reused in 06_3, 09_2, 12_6 and 15.
- **Act II (05 to 07):** the `Truss` prop (E3) with `still()` thumbnails, and `CellGrid` (sparse flat cells, E13), reused for 09_2's band.
- **Act III (09 to 11):** `HeatMap` over the kit's `RowStrips` with a mask and the diverging scale (E14); the slices; the summed response curve.
- **Acts IV and V (12 to 15):** reuse `ArrowPlane`, `Table2` and the plots; the arrow field of 15_1 (E19).
- **01 and 08:** `Deck` with walkers over the kit's `Swarm` (E20).

**Draft first, as the spikes this video lacks:** chapter 07 (most paths rewritten per frame) and chapter 10 (strips and the summed curve), each at 4K with `--max-frames 600`, before the other chapters copy their patterns.
