# Review 253: rigidity repair candidate 247 (review241 L1 and L3)

cc3, 2026-11-08. This is an independent source review of cc1's candidate against production19.
I wrote only this file. I ran no build, test, Clippy, render, extraction or synthesis, and made
no canonical, TODO, Beads or memory edit. Narration (ElevenLabs Frederick) is unchanged. Nothing
here is a picture, listening, fine-sync or final-AV acceptance.

## Verdict

There is no blocker and no Medium. The candidate does what its note claims, and every number the
note gives reproduces from the source:

- **L1 (card clearance).** Moving the sin(1/z) plate up 0.1 turns a 10.1 px overlap at 4K into a
  16.9 px gap. Every mark on the plate moves with it.
- **L3 (q's ring).** q gets a dark backing. It is layered correctly and leaves with the pile in
  the same fade. Contrast rises from 1.48:1 to 9.49:1 for the ring against the fill, and 14.02:1
  for q against the ring.
- **Schedule.** The cue and duration schedule is unchanged.
- **Tests.** Both new tests are valid. One has a working positive control.

That leaves three render checks (R1–R3) and four test notes (T1–T4). M1 (plates not fading) is
outside this candidate and this review.

## 1. Files

All paths are relative to `production-repair17/` (Q).

| Role | File | sha256 |
|---|---|---|
| base (production19; `receipt.json` line 61) | `production-source-preserved214/workspace/explainer_corpus/complex_analysis_visual/src/chapters/rigidity.rs` | `c577e2c02049aa468f2d5c3ad935d74004b5e321901a2cd73c1f1f54c1b0187e` |
| canonical `explainer_corpus/complex_analysis_visual/src/chapters/rigidity.rs` (checked today) | — | same `c577e2c0…` |
| **candidate** | `rigidity-review-repair247.rs` | `0bcfbc7d7962b2ab8fa2d27895fce104ab6cb71ce5aac26fdd0b97dca04a3df0` |
| cc1 handoff | `rigidity-review-repair247.md` | `5cad2306f309e7d7d5aff16abf0d50fe3da6b4e9884b7e798ac7e7acdbcd5c1c` |
| root kit binding | `rigidity-kit-binding254.json` | `707f4c8346af40f202373e204945425f9b55a85419ac56c8d295fd179608ae75` |
| motivating review | `cc1-final10-picture-review241.md` | `5e337665fe5120d8…` |

- **The diff.** I produced the base → candidate diff myself. It has 123 lines, and its body
  equals the diff printed in cc1's note line for line. It has four hunks:
  - new helpers and constants (candidate lines 107–129);
  - `sin_recip_plane()` (172–176);
  - the 10_1 pile-up (241–254);
  - two tests (730, 757).

  No other scene line changes, and no on-screen string is added, so `mod.rs` TEXT, facts,
  `oracle.py`, `script.tsv` and cues are untouched.
- **Kit.** The production19 `Cargo.toml` depends on the csd `repo-view` kit path. Root's binding254
  shows production19's `props.rs` (`73606e89…`), `plot.rs` (`15e8c470…`) and `animatic.rs`
  (`631fbdfd…`) equal the local files. I re-hashed the local files, and they match. The kit
  arguments below use only those three files, except where T4 says otherwise.
- **Engine.** fmn pin `a399d626`.

## 2. Image and card clearance (L1)

- **The band.** 10_5's card is `precisely_now` → `precisely_card` → `explainer_kit::props::precisely`
  → `lower_third` (`props.rs:485`). `precisely_card` only sets the back's fill opacity to 1, which
  does not change its geometry.
  - The band is h = 0.3 + 0.36 × lines high (`props.rs:507–508`), centred at −4 + 0.2 + h/2.
  - Its back is `card()` (`props.rs:44`): corner radius 0.12, stroke 1.5.
  - With two lines, its top is −2.78, and the stroke's outer edge is −2.7725 (4K row 1828.6).
- **The plate.** The border is `Frame2::border` with stroke 2.0 (`plot.rs:117–124`), reached
  through `draw::Plane::border`. The window is `Plane::new(c, 5.6, ±0.42, ±0.42)`, which is
  5.6 × 5.6.

| | Plate bottom / top | Border outer foot | Gap to band (4K) | Title centre |
|---|---|---|---|---|
| production19, centre y 0.0 | −2.80 / 2.80 | −2.810 (row 1838.7) | **−0.0375 (−10.1 px)** | 3.10 |
| candidate, centre y 0.1 | −2.70 / 2.90 | −2.710 (row 1811.7) | **+0.0625 (+16.9 px)** | 3.20 |

- **The model matches the render.** The predicted −10.1 px agrees with review241's measured
  8–10 px. That also supports the stroke-width convention (w/100 scene units) the test relies on.
- **Every plate mark moves with the window.** I read all of 10_4–10_5 (candidate 425–528). Each
  mark is placed through `sp`:
  - `Painter::build(…, sp, …)`, `sp.axes`, `axis_names(…, &sp)` and `sp.border`;
  - the title at `sp.frame.left() + 0.9, sp.frame.top() + 0.3`;
  - the 24 zeros and their backings at `sp.p(z)`;
  - the "1/π" card, the "0" card and the red cross at `sp.p(…)`.

  The right column is at `right = 4.3`, away from the plate, and nothing in it points at a plate
  position.
- **The painter is unaffected.** `picture_cells` uses only `plane.frame.width` and `height`
  (`draw.rs:890–893`), so the cell grid and the crisp stroke widths do not change.
- **Other bands.** The one-line names bar of 10_4 has its top at −3.1325, well below the new foot
  (−2.71). The title's top stays inside the frame. The 10_6 sine plate is unchanged.
- **Why a move rather than a resize.** A move keeps the plate's scale (6.667 scene units per unit),
  the zero placements and the x-only cross clearance test (`inner > CROSS_ARM`). I agree with cc1's
  choice.

## 3. q ring: layering and contrast (L3)

- **Marks.** `draw::dot` (`draw.rs:494`) is an opaque `Circle` (`flat(color, 1.0)`) with no
  z-index, and `ground()` is `#0E1116`. Both are already imported by rigidity.rs (candidate line 18).
  - The backing is `dot(q, Q_BACKING = 0.11, ground())`.
  - q is `dot(q, Q_DOT = 0.08, hero())`.
- **Order.** `fade_in_all(stage, &q_marks, 0.12)` begins the backing, then q. fmn roots them in
  that order, and `add_to_scene` appends with a stable sort on z-index. So q is drawn above its
  backing, and both are drawn above the white dots, the disks and U's fills, which were all added
  earlier.
  - This matches the 10_4 zero style: an r + 0.03 `ground()` backing, then the dot (candidate 464).
  - Neither mark is visible before its fade, because the dots are only added to the arena.
- **Exit.** `piling.extend(q_marks)` puts both into the existing 10_2 `fade_out_all(&gone, 0.5)`
  (candidate 265). `piling` has no other reader, so each mark fades out exactly once, with no
  second-fade flash.
- **Contrast.** WCAG ratios, from review241's measured q (251, 220, 110) and fill (206, 183, 88):
  q against the fill is 1.48:1 (review241's figure), the backing against the fill 9.49:1, and q
  against the backing 14.02:1. These reproduce cc1's numbers.
- **Coverage.** Distances are from q, in scene units; the white dots have radius 0.06. These
  reproduce cc1's table.

| k | d | Visible past q alone (production19) | Visible past the backing (candidate) |
|---|---|---|---|
| 0–3 | 0.954, 0.591, 0.367, 0.227 | whole | whole (k = 3 clear by 0.057) |
| 4 | 0.141 | whole | 0.091 of 0.12 |
| 5 | 0.087 | 0.067 | 0.037 (≈10 px at 4K) |
| 6 | 0.054 | 0.034 | 0.004 (≈1 px; effectively hidden) |
| 7, 8 | 0.034, 0.021 | 0.014, 0.001 | hidden |

## 4. Mathematical geometry

- **U and q.** U is `blob_radius(θ) = 2 + 0.35 cos(3θ + 0.4) + 0.25 sin 2θ` about (−2.4, −0.3).
  - q = (−1.8, −0.5) is inside U, 1.0498 from its edge. `blob(720)` and a 20,000-point boundary
    give the same value.
  - Every piling point is inside U, at least 0.812 from the edge.
- **Accumulation.** The points are q + t(1, 0.35) with t = 0.9·0.62ᵏ. They are distinct, each is
  0.62 times as far from q as the one before, and they accumulate at q only. So they form a set
  whose limit point lies in U. This is exactly the hypothesis in `EXACT` ("agree on a set with a
  limit point in U") and in `PILING`. `EXACT` is unchanged and states the identity theorem
  correctly: connected open U, holomorphic f and g, a limit point in U.
- **The helpers change nothing.** `limit_point()` and `piling_points()` give the same values as
  production19's inline code. `PILE_DOT` 0.06 and `Q_DOT` 0.08 are the old literals.

## 5. Cue and duration schedule

I compared the `construct` bodies' time-bearing calls with the same read-only listing used in
review235. It covers fades, plays, sentence and phrase waits, holds, predicts, `wait_until` and
`finish`.

- **Call sites:** 75 in the base and 76 in the candidate. The only difference is one inserted
  `fade_in_all(…, 0.12)`, the `q_marks` step. Every other call has the same order and arguments.
- **The pile-up loop** now runs 9 times instead of 10. So the pile-up is still ten 0.12 s plays,
  1.20 s in total.
- **No cue moves.** No `say`, `wait_for`, `wait_sentence` or reading window changes, so the cue
  schedule is the same.
- **Frame count.** cc1 predicts 7,442 frames for chapter 10. I did not verify that count; I
  verified only that no duration changes.

## 6. Test validity

- **`the_points_pile_up_inside_u_at_a_marked_point_inside_u` (candidate 730).**
  - It uses the scene's own `limit_point`, `piling_points` and radius constants, so it binds the
    drawn picture rather than a copy.
  - My mirror passes every assertion. The ratio check survives rounding: its error is about 1e-14,
    against a 1e-12 tolerance.
  - The "first six show ≥ 0.03" assertion passes by only 0.0074, at k = 5. That is intended: it
    fails if `Q_BACKING` grows past about 0.117.
  - `inside_u` is the existing test-module helper (candidate 698), and `edge_distance` is a public
    scene function.
- **`the_sin_recip_plate_stops_above_the_two_line_lower_third` (candidate 757).**
  - It passes: the gap is 0.0625 against a 0.05 threshold.
  - Its positive control works: the base's centred plate gives −0.0375, so the test fails on
    production19.
  - Its kit constants (0.2, 0.3, 0.36, 1.5 and 2.0 strokes) match the bound `props.rs` and
    `plot.rs`.

## 7. Render checks for root (not defects)

- **R1, q's ring and the pile.** The dark ring sits above dots 4–6, so the visible stream ends at
  dot 5's crescent and then a 0.03 dark gap before the gold. On stills at 13–16.7 s, check that
  this does not read as the points stopping short of q.
  - The 10_4 zeros already use this style.
  - If it does read that way, there are two options. One is to draw the backing under the white
    dots, as a z-order change at the same timing. The other is cc1's `Q_DOT + 0.02`, which keeps
    k = 6 at about 4 px.
  - I do not require either.
- **R2, ring width.** The ring is 0.03 wide: 8 px at 4K, about 4 px when viewed at 1080p. The
  contrast is strong, but check that the ring reads at 1080p viewing size.
- **R3, plate edges and pixels.** The new edges fall on whole pixels at 4K (729 and 783 px from
  centre) and at 720p (243 and 261 px). They fall on half pixels only at 1080p (364.5 and
  391.5 px). That matters only if a 1080p render is made.

## 8. Test notes

- **T1.** The band test mirrors the kit constants, because `line_h` and `LOWER_THIRD_CHARS` are
  private. It will not follow a kit change by itself; cc1 says so too.
- **T2.** The band test checks the two-line count with the kit's first `wrap(EXACT, 92, 3)` pass
  only (89 + 40 characters). `lower_third` narrows the wrap if a line overflows. That assumption
  is robust: the text stays two lines at every narrowing down to 68 characters, and first becomes
  three at 64. The render shows two lines.
- **T3.** The title assertion (`top + 0.3 + 0.3 < 4.0`, i.e. 3.5) is a coarse proxy for the
  title's top edge. It is true, but weak.
- **T4.** Two compile dependencies fall outside binding254's three files:
  - `explainer_kit::animatic` must be public in the kit's `lib.rs`. Locally it is (`lib.rs:15`
    `pub mod animatic`, last changed `f9293808`, 2026-10-27, clean tree). That file is not bound to
    production19.
  - `fmn::core::constants::FRAME_Y_RADIUS` exists at `a399d626`: fmn `lib.rs:975` has
    `pub mod core { pub use fmn_core::*; }`, fmn-core `lib.rs:13` has `pub mod constants`, and the
    constant is at `constants.rs:24`, equal to 4.0.

  The rest also looks type-correct by reading:
  - `[Mob; 2]` passes to `&[Mob]` and to `Vec::extend`;
  - `const Q_BACKING = Q_DOT + 0.03`;
  - the `powi(k)` index type;
  - the public `sp.frame` and `Frame2::bottom` / `top`.

  Only root's build proves it compiles.

## 9. Not established

- I did not compile, run Clippy, test or render anything; root owns the build and Beads.
- I made no picture, pixel, motion, listening, fine-sync or final-AV judgement.
- Contrast figures use review241's measured colours, not new frames.
- M1 (plates not fading, CC2's investigation 246) is untouched.
- Frederick narration, the script, cues and takes are unchanged by this candidate.
