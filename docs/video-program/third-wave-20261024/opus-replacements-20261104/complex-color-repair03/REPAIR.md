# Complex colour plates: stripes and stair-steps (04-A, 04-A2, S2)

LavenderPike for BlueSalmon, Mail 2523, with Mails 2542, 2591 and 2614 (chapters 10, 12, 14). Bead `.39`. 2026-11-04.

**Status: staged, not installed. Nothing here has been through Cargo, linked, run or rendered. No claim is made about how any candidate looks at 4K.** The film's default stays as it is, so applying patch A alone should change no frame. Narration stays ElevenLabs Oliver; no cue or word changes, and `script.tsv` stays `2968b0fd7654`. No mathematics or depth is cut.

This is not an independent review. I have repaired this film's pictures before (`complex-picture-repair-06-10`). BronzeJaguar wrote the chapters.

## Short version

- **Stripes.** The engine's coverage rule, applied to the film's row geometry, predicts the dark rows on all 20 lossy 720p patches I tested (seam-only r +0.978 to +0.998). Each strip's half-pixel antialiasing band lets the background through at row boundaries.
- **At 4K the desk model keeps them.** It predicts the same depth (−7.5 luma on a flat colour), with rows 3 times further apart.
- **Mail 2167's overlap (0.012) swaps the still defect for a fade defect.** It predicts bright rows of up to +19.7 luma on about 577 of 1,502 pixel rows at 4K. A 0.6 px overlap predicts up to +15.3 on 153 rows.
- **The candidate fix is `PLATE_TILE`: antialiasing 0 and strokes 0.0001 units wider.** Under the film's adaptive antialiasing, the model predicts no deviation at any size or opacity, with no added curves. It removes stripes only.
- **Stair-steps are the cells themselves, 9.45 px at 4K.** Tiling does nothing for them. There is a ladder of finer or blended cells, with its cost, and **nothing in the ladder reaches the supersampled picture**. Which rung is worth its cost needs a person looking at root's frames.

## Run this first

From the repo root, in root's guarded lane. Patches are repo-root-relative.

```
B=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-color-repair03
python3 -I -B $B/checks/patch_roundtrip.py SCRATCH   # confirms the source hashes still hold, touches only SCRATCH
patch -p1 --fuzz=0 -i $B/patches/A-plate-core.patch
patch -p1 --fuzz=0 -i $B/patches/B-plate-lab.patch
cd explainer_corpus/complex_analysis_visual

cargo test --release --bin complex_analysis_visual                         # whole film crate; cost unmeasured
                                                                           # chapters::draw::tests alone is the new Stage tests
cargo test --release --example plate_lab lab_tests
cargo build --release --example plate_lab
L=target/release/examples/plate_lab
$L NEW_DIR --res 3840x2160 --dry-run                 # prints the plan, 153 frames; writes nothing

$L NEW_720 --res 1280x720                            # smoke: gives real seconds and MB a frame
python3 -B ../../$B/checks/measure_lab.py NEW_720 --sheets NEW_720_SHEETS

$L NEW_4K --res 3840x2160                            # only after the smoke's controls hold
python3 -B ../../$B/checks/measure_lab.py NEW_4K --sheets NEW_4K_SHEETS
```

`NEW_*` must not exist; the lab and the measure refuse a folder that does. Each patch needs fuzz 0 against the bytes bound below. If `patch_roundtrip.py` reports a moved hash, tell me before forcing anything.

If 153 4K frames is too many, these two give the main answers in 36. Run the measure on each folder. A control whose scene is missing is skipped, not failed:

```
$L NEW_A --res 3840x2160 --stage stills --subjects z,sinrecip --candidates now,tile,blend,cells2      # 8 frames
$L NEW_B --res 3840x2160 --stage fades --subjects z --candidates now,overlap,tile,blend --no-truth    # 28 frames
```

To choose a rung afterwards, apply one of `patches/select-{tile,blend,rows2,cells2}.patch` on top of A. Each changes the one constant `PLATE` and the tests that hold it. `rows2` and `cells2` also raise the strip-budget test from 200 to 400 rows.

## Expected cost and inputs

**Inputs:** the crate's sources and `script.tsv` (only because `draw.rs` compiles it in). No TTS, audio, cue timing, TeX, ffmpeg or network. The film binary is not touched: the lab is its own example program, found by Cargo's autodiscovery, with no `Cargo.toml` edit.

**Outputs:** one PNG sequence per scene, truth crops, `plates.tsv`, `receipt.tsv`.

**The plan:**

| Scenes | Subjects × candidates | Frames |
|---|---|---|
| Stills | 6 × 8, plus 3 × 5, plus 2 finest-rung, plus 2 controls | 67 |
| Fades, 7 frames each | `z` × 5, and `tan`, `bump` × 3 each | 77 |
| Repaints, 3 frames each | `z2` × 3 | 9 |
| **Total** | hard limit 160 in the source | **153** |

**Time and disk are unmeasured.** I have no 4K per-frame figure for these plates, and none can be taken here. The smoke run gives real seconds and MB a frame in `receipt.tsv` and in section G of the measure. Multiplying by 9 for pixel count is a rough guide only, because 2×2 sampling and the PNG encoder may not scale that way. Truth at 4K is at most about 48 million model evaluations per subject (`z`, `z2`) and about 314 million for all nine, single-threaded, computed from the plate sizes, with 59 MB of crops, 9 MB at most. `--no-truth` skips it. Evaluation cost differs by function, and I have not timed it.

**What should change between candidates**, from the layouts, untimed, on the 6.4 × 6.4 plate. Multiples of the film now:

| Candidate | Rows × cols | Model evals per repaint | Curves per row (raster per pixel scales with it) | Colour records per fade or repaint frame |
|---|---|---|---|---|
| `now`, `tile` | 183 × 183 | 1 | 1 | 1 |
| `tile_p3` | 183 × 183 | 4 | 1 | 1 |
| `blend` | 183 × 183 | 1 | 1.01 | 0.51 |
| `rows2` | 366 × 183 | 2 | 1.01 | 1.02 |
| `cells2` | 366 × 366 | 4 | 1.99 | 2.01 |
| `cells4` | 731 × 731 | 16 | 3.97 | 7.96 |

## The evidence so far

All of this is desk work. Run with `python3 -B $B/checks/run_checks.py SCRATCH`. Outputs are saved in `checks/*.out`.

### Why the rows go dark (`seam_cause.out`)

The check predicts the dark rows from each plate's position, cell count and band, knowing nothing about the frames. It then compares with preview02 frames (720p, lossy, CRF 26). "Seam-only r" removes the seam-free model from both sides first, because sin z's real horizontal bands otherwise dominate.

| Patches | r same rows | r one row off | Seam-only r | rms %, frame / mirror |
|---|---|---|---|---|
| 04 `z`, `z²` (5) | +0.993 to +0.997 | −0.323 to −0.357 | +0.993 to +0.997 | 1.69–1.82 / 1.75–1.79 |
| 05 `tan` (2) | +0.987, +0.992 | −0.338, −0.349 | +0.987, +0.992 | 1.83, 1.94 / 1.76, 1.83 |
| 10 `sin(1/z)` (3) | +0.989 to +0.992 | −0.082 to −0.142 | +0.986 to +0.990 | 1.83–1.87 / 1.98–2.01 |
| 10 `sin z` (3) | +1.000 | +0.453 to +0.485 | +0.996 to +0.998 | 5.42–6.26 / 5.42–6.26 |
| 12 `z`, `z²`, `1/z`, bump (5) | +0.988 to +0.996 | −0.158 to −0.337 | +0.985 to +0.996 | 1.58–1.79 / 1.73–1.91 |
| 14 opening bump (2) | +0.979, +0.992 | −0.275, −0.310 | +0.978, +0.991 | 1.59, 1.67 / 1.77, 1.80 |

A plate moved 1.3 px as a wrong-lattice control is low everywhere except sin z (+0.298 to +0.327), where the real bands carry r. That is why the seam-only column is there.

**Two real mid-fade frames** keep their cell colours. On each, the mirror's picture with the same cell colours lands far closer than a control with each row flattened:

| Frame | Fitted opacity | Pixel rms, mirror with cell colours | Pixel rms, flattened control |
|---|---|---|---|
| `12_residues/frame-0130` | 0.345 | 2.97 | 34.58 |
| `14_continuation/frame-0003` | 0.086 | 1.96 | 12.93 |

The seam-only r on those two frames is weak (+0.465, +0.083), so they say little about row depth during a fade. At those opacities the row detail is close to the encoder's floor.

### What the model predicts for seams (`seam_predict.out`)

Flat colour of luma 128, reading plate, 4K. Lowest or highest deviation from ideal in luma, and pixel rows off by more than 1, of the 1,502 modelled. These choose what to measure.

| Candidate | Opacity 1 | 0.75 | 0.5 | 0.25 | Rows at 2×2 |
|---|---|---|---|---|---|
| `now`, aa 0.5 | −7.5, 125 | −4.9, 115 | −2.7, 95 | −1.0, 0 | 159 |
| `tile`, aa 0 + 0.0001 | 0 | 0 | 0 | 0 | 74 |
| aa 0.5 + 0.6 px | 0 | +9.1, 147 | +14.5, 153 | +15.3, 153 | 66 |
| aa 0.5 + 0.012 (Mail 2167) | 0 | +11.9, 567 | +18.9, 577 | +19.7, 577 | 0 |

Across 21 plates at 720p, 1080p and 2160p, with aa 0:

- **With the guard:** no row off under adaptive antialiasing, the film's path.
- **Without the guard:** one row at 1080p (−7.5), an exact coverage tie.
- **Canonical (one sample at the pixel centre, no classifier):** the same setup would show +18.9 on 61–86 rows at opacity 0.5, even with the guard. The film should not take this path: the engine refuses ffmpeg video under certified determinism (`fmn/src/rendering.rs:749`), `AaPolicy` defaults to adaptive, and my grep finds neither the kit nor `main.rs` setting either. If those settings ever change, recheck this.

Largest gap between rows: +6.18e−05 px without the guard, −8.98e−03 px with it. The planted-hole controls fire (−111.4 at 0.3 px).

### Stair-steps (`stair_model.out`)

Desk model at 4K, all on tiling rows. rms is encoded 0–255 against the portrait at 16 samples a pixel, in a window on each plate's busiest region. A reference, not a target, and these rank candidates without saying what looks better.

| Candidate | `z²` | `z³−1` zoom | `tan` | `sin(1/z)` | `sin z` | `1/z` small | bump, ch 14 |
|---|---|---|---|---|---|---|---|
| now, steps | 7.26 | 5.09 | 11.33 | 17.86 | 10.15 | 10.70 | 10.09 |
| `blend` | 6.19 | 4.31 | 9.79 | 15.72 | 9.03 | 9.25 | 8.68 |
| `rows2` | 5.15 | 3.42 | 8.48 | 13.90 | 7.16 | 8.03 | 7.40 |
| `cells2` | 4.13 | 2.87 | 6.58 | 11.31 | 6.13 | 6.59 | 5.87 |
| cells 0.0117, blend (not a named rung) | 3.17 | 2.19 | 5.19 | 8.98 | 4.73 | 5.09 | 4.52 |
| `cells4` | 2.57 | 1.75 | 4.17 | 7.49 | 3.88 | 4.23 | 3.68 |

Things to weigh when looking:

- **`sin(1/z)` is the hardest plate.** It stays highest at every rung. If a finer rung costs too much across the film, one plate can take it alone, with one changed call at `rigidity.rs:416`: `Painter::build_styled(stage, Some(Func::SinRecip), sp, 1.0, PLATE_CELLS2)`. The same works for any plate.
- **Blending acts only along a row.** Edges that run across rows still step until the rows get finer. `sin z`'s horizontal bands gain least from `blend`, and that may read as directional.
- **Blending mixes neighbouring hues at dense cells.** Your Mail 2542 says the zero and pole clustering is intentional. A blended strip holds each cell's own colour only at the cell's centre, so a dense centre can read softer or muddier than now. Please look at this on `sinrecip`.

## What the bundle contains

| Path | What it is |
|---|---|
| `patches/A-plate-core.patch` | New `src/plate_layout.rs`, `mod plate_layout;` in `main.rs`, and in `draw.rs`: `PlateStyle`, the `PLATE_*` rungs, `Picture`, `portrait_styled`, `Painter::build_styled`, tests. `PLATE` stays `PLATE_NOW` |
| `patches/B-plate-lab.patch` | New `examples/plate_lab.rs` |
| `patches/select-*.patch` | On top of A: flip the film's choice |
| `candidate/` | The four files as they stand after A and B |
| `checks/` | The checks below and their saved outputs. `run_checks.py` lists and runs them |
| `make_patches.py` | Rebuilds `patches/` from `candidate/` |

No change to `explainer_kit`, `RowStrips` or the engine. `RowStrips` is shared with five other films, so a tiling or blended strip must stay local to this crate. `PLATE_NOW` goes through the same `RowStrips::build` and the same half-pixel band in the same order, so source reading says its frames should match. A test compares the records. Only a render can compare the frames.

### How a fade should keep vertex colours

Three layers of evidence, in rising order of what they prove:

1. **Source reading** of `fading.rs`, `transform.rs` and `align.rs`: a fade writes alpha only, per record.
2. **Rust test** `fades_keep_every_vertex_colour`: reads each record's colours at every frame on a real `Stage`, for `now`, `tile` and `blend`. **Not run.**
3. **Real pixels** in `measure_lab.py` sections D and E: each fade frame against its own hold frame at the fitted opacity, the hold against the still, and `flat_control`, a planted flatten that has to fail. **Waits on root's run.**

### What the measure reports

`measure_lab.py` exits 1 if a control fails, and prints measures without a verdict. Its controls are:

- `now` shows a dark lattice on `z` (at least 3% of rows)
- the mirror lands on `now`'s rows (r at least 0.9)
- `flat_control` fails the colour test
- Mail 2167's overlap shows bright rows mid-fade
- each fade was caught part way through
- no receipt line reports moved or missing records

**If the model is right, the 4K run shows:**

| Measure | Expected | If not |
|---|---|---|
| `now` on `z` | Dark rows at the mirror's rows. Depth scales with the colour; −7.5 is the flat-128 figure | The mirror is wrong above 720p. Stop and tell me |
| `tile` on `z`, section A | Under 1% of rows dark; one or two are the picture's own background cell at the zero | Tiling does not work as read. Stop |
| `tile` and `blend`, fades | Row deviation near 0, pixel rms under 1 | The guard or the classifier reading is wrong |
| `overlap` and `overlap06`, fades | Bright rows, up to about +19 and +15 | Double cover reads differently than modelled |
| Hold frame against still | rms 0 | The fade path changes the picture: a defect to report |

## Tests

**Run here, for real, passing:**

- 9 pure layout tests in `plate_layout.rs`, 3 of them planted-defect controls (`plate_layout_tests.out`).
- 3 tests of `share_of` and `same_colours`, lifted out of the sources by `pure_helpers.py`.
- The `measure_lab.py` self-test: synthetic frames from the mirror, with four planted defects, each caught. That proves the measures can fail. It says nothing about a real render.
- 32 type-check, Clippy and rustfmt lines (`candidate_typecheck.out`). The film and the lab in normal and test cfg, and the film under each select patch, by `rustc --emit=metadata` and `clippy-driver -D warnings` against existing rmeta.
- Patch round trip at fuzz 0, forward and reverse, with bytes compared at each stage (`patch_roundtrip.out`).
- The source-scan strings in `every_colour_picture_is_crisp` and `the_plates_are_the_films`, evaluated in Python on today's text (`bindings.out`). The two Rust tests themselves have not run.

**Type-checked only, never run, because nothing linked to the engine can run here:**

- In `draw.rs`: `plate_now_writes_exactly_what_the_kit_writes`, `tiling_changes_the_band_and_the_width_and_nothing_else`, `a_blended_row_carries_its_layout_and_recolours_in_place`, `fades_keep_every_vertex_colour`, `control_a_flattened_or_uneven_lane_is_caught`, `the_film_draws_what_it_drew`, `every_rung_keeps_a_full_picture_under_the_group_limit`, `four_samples_are_the_cells_mean_and_one_sample_is_its_centre`, and the updated `every_colour_picture_is_crisp` and strip-budget test.
- In the lab: `the_plates_are_the_films`, `the_full_plan_fits_and_has_its_controls`, `same_colours_fails_on_what_it_is_for`, `a_plate_box_covers_the_plate`.

These can fail on first run. `fades_keep_every_vertex_colour` is the most likely, since it assumes the updater sees interpolated records on each frame.

## Coverage of what root flagged

| Flag | Plate | Mirror check | Lab subject | Fix path |
|---|---|---|---|---|
| 04-A stripes | Every picture in 04 and 05, worst on `tan` | Yes | `z`, `z2`, `tan`, `zoom` | `Painter` |
| 04-A2 stairs | The same, worst on the zoom | Stair model only | `z2`, `tan`, `zoom`, centre 0.613822 + 1.063171i from `tile()` | `Painter` |
| S2 stairs | 07 `\|z\|²` (`one_point.rs:410`), 09 bump (`boundary.rs:171`) | No | **Not in the lab.** Same builder and `CELL`, so the rungs apply; root's film render has to show them | `Painter` |
| Mail 2542, ch 10 | `sin(1/z)`, `sin z` | Yes, 6 patches | `sinrecip`, `sin` | `Painter` |
| Mail 2591, ch 12 | Small plates, bump | Yes, 5 patches and a real fade | `recip`, `bump` with fades | `Painter` |
| Mail 2614, ch 14 | Opening bump | Yes, 2 patches and a real fade | `openbump`, stills only | `Painter` |

Every plate in the film goes through `Picture::build`: all the `Painter::build` sites, and `portrait` at `the_bump.rs:367`. So the plates not in the lab (exp, gamma, zeta, lacunary, `|z|²`, the mosaic, counting, boundary) take the same fix, unmeasured. **Chapter 14's radial dim is separate:** `outside_mask` is a flat fill drawn over the plate, unchanged here and not in the lab, so it still needs looking at with the rest of that scene.

## Bound to these bytes

`patch_roundtrip.py` checks these first. Root edits canonical between syncs.

| File | sha256 |
|---|---|
| `explainer_corpus/complex_analysis_visual/src/chapters/draw.rs` | `4eba4bb18b989ac7075519031913ad4974e471a495a56a80fde4f796d9159545` |
| `explainer_corpus/complex_analysis_visual/src/main.rs` | `dcc3e6281e42f7bbc6f0b068122d9086547a0f75221b1652eebe3278946327c1` |
| `explainer_corpus/complex_analysis_visual/script.tsv` | `2968b0fd76540cbcd4f50290925e8acfec1aa995bea76a04934dac12f959929e` |
| `crates/explainer_kit/src/props.rs` (read, unchanged) | `73606e8921ec18c8492ec3a7355a755e269acabd174cafed1fabfca14379d436` |
| Engine | fmn `a399d626` |

**Other lanes:** `complex02-05-repair03.patch` and the four `complex06-09-repair03` patches touch neither `draw.rs` nor `main.rs`, and move none of the plate literals the lab pins. `checks/bindings.out` holds their hashes as checked. If a chapter changes a plate later, `the_plates_are_the_films` will say so.

## Considered and not built

- **Soft rows** (a band of about one cell in pixels, with the width and position to match). I probed this in a scratch model: rms 3.65 on `z²` at cells 0.0175, no 2×2 cost, and a fade ripple of at most 1.7 luma. I left it out for three reasons. It needs pixels per unit inside a scene, which `stage.config()` does not give. It softens the plate's top and bottom edges with overhang. And it changes the fade curve (opacity 0.5 reads 111 where it should read 94). It is the most promising thing I did not build, if you want a next step.
- **A textured image plate.** `ImageMobject` exists, but `RenderOptions.camera`'s docs tie textured images to the certified camera route. I did not explore this and make no claim about it.

## Exact limits

- **No Cargo build, link, run or render of any candidate or of the lab.** The Rust was type-checked and linted, and the plate layout's pure tests ran.
- **No pictures of any candidate exist.** The simulated crops I looked at to rank candidates are not in this bundle and are not evidence of a render.
- **The mirror has been tested against real frames at 720p only**, lossy, on the current film. Every 1080p and 4K number, and everything about `tile`, `blend` and the overlaps, is prediction. It leaves out x-direction detail inside a pixel, the encoder, and anything drawn over the plate.
- **The lab stops at lossless PNG.** Nothing here tests a 4K encode. A final MP4 may soften or reinforce any of this, so a candidate that wins here still needs an encoded check.
- **The two real fade frames are sampled and lossy.** They support colour retention at those moments, nothing about continuous motion.
- **Row heights alternate by one pixel** at every size (9 and 10 px at 4K), under every candidate at that row height. Hard row edges could make that easier or harder to see, and I can't tell from here.
- **Tiling makes the plate's top and bottom edges hard rather than half a pixel soft.** Most plates draw a border over them.
- **`tan` is built directly in the lab.** The film reaches it by repainting `sin` on the same plate (`gallery.rs:591`).
- **No listening and no sync.** Nothing here touches audio. Sampled pictures are not continuous AV, and root's native 4K acceptance stays pending.

## Unrun, for root

1. Apply A and B, then `cargo test` as above. Tell me about any failure.
2. The 720p smoke, then the measure. Its controls must hold.
3. The 4K run or the 36-frame subset. Read the measure, then view the sheets.
4. Choose a rung, per film or per plate. **I am not recommending a stair-step rung ahead of those frames.** For stripes alone, `select-tile` is what the model favours, if the run confirms it.
5. Rebuild, then check real film scenes at 4K, including an encoded check and fades.
