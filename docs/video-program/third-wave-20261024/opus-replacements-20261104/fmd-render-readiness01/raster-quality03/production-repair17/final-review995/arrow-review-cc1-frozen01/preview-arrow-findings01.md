# Preview arrow (01_6 "coming up" loop): independent review of root candidate01

CC1, 2026-11-10. This is a scratch-only, source-bound review. Nothing was compiled, built, rendered or installed, and there is no rendered acceptance. Narration (ElevenLabs Frederick Surrey) and every cue schedule are untouched.

## Verdict

**buff(0.0) solves it.** Root's one-line candidate01 (`.buff(0.0)`) fixes the collapsed arrow and its wrong direction.

- **Endpoint overlap.** The apex now ends exactly on the ring's stroke centreline. That is the arrow touching the loop point it names, which is the film's own "reaches both its points" convention. I do not count it as a defect.
- **Recommendation.** Adopt the candidate01 picture.
- **Optional extra.** candidate03 (below) draws exactly the same picture and adds one native test. When root runs the guarded compile, that test checks the pinned Arrow buff semantics and guards against the arrow collapsing again.

## Inputs (sha256)

| What | File | sha256 |
|---|---|---|
| build268 base | `root-boundary760-source/workspace/.../chapters/the_bump.rs` (bound at `review-build268-terminal272/source-copy.json:55`) | 8e93012e5b7598b9a99ffcdb07e436341f052d1175455f5581e108ee86e6ddb3 |
| root candidate01 | `final-review995/preview-arrow-candidate01-the_bump.rs` (`diff` vs base: only `+        .buff(0.0)` at line 372) | c52c0472196cbc68f3fca2b47fc2ad7d6cc9c3ba6e5ee8f356b882e4ffc27564 |
| root receipt / patch | `preview-arrow-candidate01.json` / `.patch` | a90658a9… / 35c2ce5d… |
| fmn line.rs (Arrow) | installed checkout `a399d62/crates/fmn-library/src/line.rs` = git blob at pin a399d626 | 4f11e1873a2af84a067c19c9808576ac31220e0234d9d697a3272c183d0bc010 |
| fmn space_ops.rs | `a399d62/crates/fmn-geom/src/space_ops.rs` | b7fa51ea7d05d9a49a7fa4f387349cdce3a6551abee6cd89ce793e2c6524429f |
| draw.rs (Plane) | build268 | 9bb07b3ff030d1ac0885d1382097bde596e8cae419d86b51933e12e7eeed8218 |
| explainer_kit plot.rs (Frame2::p) | repo HEAD `crates/explainer_kit/src/plot.rs`. **Not bound** by build268 source-copy.json; this is an assumption, but the stills match the mirror that uses it | 15e8c470ece7844da1f86562e214ad5185454abb079bc5ceb5f6615cf792246d |
| script.tsv | build268 workspace; the 01_6 text is unchanged by any candidate | 234f725781bde0bdf35afbc21fd2d6ec0f73f99a95723294259cc1b75a813d3c |

## Native Arrow semantics at a399d626 (line.rs 462–758, 928–944)

- **`Arrow::new` defaults:**
  - buff MED_SMALL_BUFF = 0.25, trimmed from **each** end;
  - thickness 3.0, so the shaft width is 3.0 × 0.015;
  - tip_width_ratio 5, tip angle π/3;
  - caps: tip length ≤ 0.5 × length, width ≤ 0.1 × length.
- **`key_dimensions` is evaluated at `length − buff`** (one buff, not two). The ends are then trimmed by buff each.
- **Outline.** It is drawn pointing left, with shaft x ∈ [0, trimmed − tip_length] and the apex at −tip_length.
- **Rotation.** The outline is rotated by `angle_of_vector(trimmed vect) − angle_of_vector(apex − arrow_start)`, where `angle_of_vector` = atan2 and `normalize(0)` = 0.
- **Placement.** It is shifted so `arrow_start` = ½(p[0] + p[−3]) lands on the trimmed start.
- **Consequence.** With trimmed length ≈ 0, the shaft length is negative, and the direction comes from floating-point residue.

## Mirror (arrow-mirror01.py → arrow-mirror01.out)

The preview arrow runs from `lp.p(0)` = (5.6, 0) to `lp.p(polar(0.8, 0.9))` = (5.9108, 0.3917).

- **Length:** 0.5 scene units, since lp.unit() = 0.625.
- **Ring:** radius 0.5, drawn with stroke 4.0 = 0.04 units, so the band is r ∈ [0.48, 0.52].
- **Scale:** tile px = 67.5 per unit; 4K px = 270 per unit.

| | build268 (buff 0.25) | candidate01 (buff 0.0) |
|---|---|---|
| key_dimensions length | 0.25 | 0.5 |
| shaft width / tip width / tip length | 0.025 / 0.125 / 0.108 (width capped) | 0.045 / 0.225 / 0.195 (no cap: ratios 1.11, 1.28) |
| trimmed vector | (0, +1.11e-16): residue | (0.3108, 0.3917) |
| shaft length | **−0.108** | 0.305 |
| rotation applied | −90° | −128.43° |
| head direction | **90° (straight up)** vs the intended 51.57° | **51.57°** = arg polar(0.8, 0.9) |
| tail / apex radius | both **0.25** (a detached head mid-radius) | tail 0, apex **0.5000** (stroke centreline) |
| apex, tile px | (868.5, 256.8) | (879.0, 243.6); 4K ≈ (3515.9, 974.3) |
| head size, tile px (4K px) | 7.3 × 8.4 (29 × 34) | 13.2 × 15.2 (53 × 61); shaft 3.0 (12.2) |
| winding at shaft centre | **0**: the reversed shaft cuts a hollow slot in the head | 1 |

## Stills read (evidence)

- **Sheets read in full:**
  - sheet-0023: f5530, 5531, 5591, 5748, 5760, 5784, 5940, 6000;
  - sheet-0024: f6048, 6060, 6084, 6113, 6155, 6215, 6269, 6273.
- **Crops:** 8× crops of the ring panel, made with crop995.py from overview tiles of the 960×540 JPEG sheets:
  - `s0023-slot3-f5591-x8.png` (a57229d9…);
  - `s0024-slot7-f6269-x8.png` (b65f6f7e…).
- **What the stills show:**
  - f5530 and f5531 show the ring with no arrow (fade chunk 1).
  - From f5591 through f6273, every still shows the same broken mark:
    - a detached head of about 7.5 × 8.5 tile px;
    - apex near tile (868.5, 256.5), about halfway out along the radius;
    - pointing **straight up**;
    - with a **hollow vertical slot** in its middle.
  - This matches the mirror's prediction for buff 0.25 to within about 0.5 px, in position, size, direction and hollow centre. So the mirror's output for buff 0.0 rests on a mirror that reproduced the broken picture.
- Root's `independent-preview-arrow-crop01.png` has the same sha as CC0's crop (51aef117…), so it is a copy, not separate evidence. My crops are separate.

## Answers to the three questions

1. **Collapse.** Fixed. The arrow becomes a full 0.5-unit arrow (shaft 0.305 + head 0.195 × 0.225), and neither cap binds.
2. **Direction.** Fixed, and no longer misleading.
   - The tail is at zero and the head points outward at 51.57°, ending on the loop.
   - This is the film's "arrow from zero" grammar: turning.rs:107–110 `reaching_arrow` (buff 0, "an arrow from zero") and draw.rs:1407–1409 (chain arrows, buff 0).
   - Under narration 01_6 ("…count roots we never find…"), the icon stands for an arrow from zero to a point on the loop. A still icon cannot show the turning itself; the source docstring says "turning", and the narration does not describe the arrow.
   - The build268 mark, an upward caret at r = 0.25, was the misleading one.
3. **Endpoint overlap.** Small, and conventional.
   - The apex lies on the band's centreline. The last 0.02 units of the tip (5.4 px at 4K, 1.4 tile px) lie on the band's inner half, at most 6.2 px wide at 4K.
   - The band's outer half (5.4 px at 4K) continues past the apex, so the ring stays continuous.
   - The head's base corners are at r = 0.325, which keeps them 0.155 units (about 42 px at 4K) inside the band.
   - The arrow is in fade chunk 2, after the ring in chunk 1. I assume, but did not trace in the renderer, that it draws over the ring.
   - **Not recommended:** stopping the apex at the inner edge, i.e. ending at polar(0.768, 0.9). After antialiasing, the 0–1 px gap would read as falling short, and it breaks the reaching convention.

## Timing and scope (scope-check01.py → scope-check01.out)

- **Timing.** For both candidate01 and candidate03, everything before `fn previews(` is byte-identical to build268 (13,594 chars). That covers construct(), every wait_for, fade, pause and hold, and the 01_6 chunking.
- **Mobs.** previews() still has the out-op sequence push ×4, extend, insert, giving 7 mobs. So the chunks stay:
  - [caption, portrait, ring];
  - [arrow, half-disk, zero backing];
  - [zero dot].
  This matches the stills: f5591 has the half-disk without the ink dot, which appears by f5748.
- **No new calls.** No animation or narration call is added.

## Other default-buff arrows in build268 (all chapters, `.orig` excluded)

Every other `Arrow::new` either sets `.buff(0.0)` / `FLIP_BUFF` or is long enough not to collapse (collapse means length − 2·buff < tip length):

- continuing.rs:779 `hop`: 1.154 units, trimmed to 0.654 > tip 0.195. Intact, but shortened 0.25 at each end; this is pre-existing and not in scope.
- gallery.rs:503 `up`: 5.87 units.
- gallery.rs:511 `across`: 2.75 units.
- two_planes.rs:389 `carry`: about 7 units, between the planes.

The preview arrow is the only collapsed one. No other arrow constructors are used.

## Optional candidate03 (same picture + native guard test)

- `candidate03-the_bump.rs`: **90b51c2b30877911bbb5f5e14ee810268e14efb24e7c4e2a55f7f7e128c06ee7**
  - Patch over root candidate01: `candidate03-over-candidate01.patch` 12687130d0156f714c0386b5e2b4115ad4df789ad4ab35a9ece83417c14a4d43.
  - Patch over build268: `candidate03-over-build268.patch` 0be547f3d936c22d1ede7526f8c4bb264e753acf858e34b60c6826dd54bbcbba.
  - Generator: `make-candidate03.py` 7d89d5bef522487992914c7e379f65d851e5e4ff4e865e0450f33599123d8e2e. It refuses any input other than c52c0472 and refuses to overwrite.
- **Changes:**
  - consts `PREVIEW_LOOP_R = 0.8` (now shared by the ring and the arrow) and `PREVIEW_ARROW_ARG = 0.9`;
  - `fn preview_arrow(lp: &Plane) -> fmn::Result<VMobject>`, which uses the same builder calls as candidate01;
  - previews() calls `preview_arrow(&lp)?`;
  - new test `the_preview_arrow_reaches_the_loop_head_first`. It reads the real outline, like `turning.rs` `the_arrows_reach_their_points`, and asserts:
    - the projection onto zero → loop point spans [0, len] within 1e-6;
    - the farthest points lie on the axis (|across| < 1e-6), i.e. the head, not shaft corners, is at the loop.
- **rustfmt:** `rustfmt --edition 2024 --check --config-path rustfmt.toml` passes on candidate03, candidate01 and the base.
- **Test mirror (test-mirror01.py → test-mirror01.out, 47c04776…):**

  | Case | Result |
  |---|---|
  | build268 | FAIL (0.126 to 0.258 of 0.5) |
  | candidate | PASS |
  | reversed arrow | FAIL (far corners across 0.0225) |
  | right arrow tested against 1.2 rad | FAIL |

  All four are as expected.
- **Preserved trial:** `candidate02-the_bump.rs` 7e5b2c5678ec85de16974560c85fe6def7d45760e09d0cbd75f11feba5368f8e failed rustfmt on its `Ok(…build()?)` layout. Its patches are f76d06c9… (over build268) and 2f353e0f… (over candidate01). Do not use it.
- **Limits:**
  - The test duplicates previews()' plane literal `Plane::new(v(5.6, 0.0), 1.5, …)`.
  - It is uncompiled. It relies on `m.points()` on a VMobject (as turning.rs does) and on `use super::*` reaching `Plane`, `v`, `C` and `ZERO` (as the existing the_bump tests do).

## Native checks NOT run here (root's to run)

1. **Guarded compile and tests** with candidate03, or with candidate01 if the test is not wanted.
   - Expect `the_preview_arrow_reaches_the_loop_head_first` to PASS against the pinned fmn a399d626.
   - Native positive control: in a scratch copy, remove `.buff(0.0)` from `preview_arrow`. Expect FAIL near "0.126… to 0.257… of 0.5".
2. **Strict Clippy** (`-D warnings`) on the crate.
3. **Narrow native visual check** of the 01_6 window. Suggested: f5591 plus one frame near f6100, with the ring cropped around 4K (3432, 1080).
   - The tail should sit at the ring centre.
   - The apex should touch the ring at about 4K (3516, 974).
   - The ring's outer edge should be continuous past the apex.
   - There should be no floating head.
4. **Integration:** a new chapter render and master. This review is source and stills only, and it is not rendered acceptance.
