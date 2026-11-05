# Complex colour plates: 720p comparison (Mail 2775)

LavenderPike, 2026-11-04. A read-only comparison of root's three lab runs. I ran no native job, and made no canonical edit, Beads change or synthesis. All narration stays ElevenLabs Oliver.

## Answer

**Select `cells2` for the whole film**, by `complex-color-repair03/patches/select-cells2.patch` on top of patch A. It does best on both faults. The dark lattice is gone: 0.0% dark rows on `z`, against 21.1% under `now`. The stairs fall to 0.45 of `tile`'s, the median over the eight contoured subjects. It is also the first rung where the rings read round.

**Fallback: `rows2`**, by `select-rows2.patch`, if root will not pay for `cells2` once the chapter run has measured it.

**`cells4` is rejected.** It cannot run within the engine's limit, and I propose no change to that limit.

This is a selection from 720p lab stills. It is not visual acceptance. **`cells2` has never been rendered in a fade or a repaint, or at 4K**, so the selection depends on the probes listed below.

I authored the lab, the measure and the candidates. This is an author's reading, not an independent review.

## The candidates

Measured on `z` at 720p unless a column says otherwise.

| Candidate | Dark rows | Darkest row, luma | Fade, brightest row against ideal | Stairs, share of `tile` | Seconds a frame, share of `now` | Knots on the 6.4 plate, % of limit | Verdict |
|---|---|---|---|---|---|---|---|
| `now` | 21.1% | −8.29 | +3.57 | 1.02 | 1.00 | about 13 | The fault |
| `overlap` | 0.2% | −1.18 | **+16.68** | not compared | not compared | not counted | Reject: bright lines in fades |
| `overlap06` | 7.3% | −1.67 | +7.52 | not compared | 0.98 | not counted | Reject: faint rows, fade lines, and it does nothing for the stairs |
| `tile` | 0.4% | −1.57 | +0.26 | 1.00 | 0.88 | about 13 | Fixes seams only |
| `tile_p3` | 0.0% | −0.80 | not run | 0.76 | 0.95 | about 13 | Keeps the stairs |
| `blend` | 0.2% | −1.39 | +0.26 | 0.89 | 0.82 | 6.5 | Cheapest, keeps the stairs, directional on near-level edges |
| `rows2` | 0.0% | −0.42 | not run | 0.70 | 1.32 | 13.1 | **Fallback** |
| `cells2` | 0.0% | −0.41 | **not run** | 0.45 | 1.82 | 25.9 | **Selected** |
| `cells4` | did not run | | | | | **102.5** | Reject: over the limit |

- **Stairs** are rms against the per-pixel truth after a 2×2 box, as a share of the same subject's `tile`. The share is the median over the subjects, leaving out `z`, which has no contours. Per-subject values are in `checks/tables.out` section 2.
- **Seconds** are the median over the stills, from one debug-build run of each scene at 720p. They rank candidates and bound nothing at 4K. `cells2` ranged 1.53 to 2.04, `rows2` 1.22 to 1.42.
- **Row and fade numbers** are from `checks/measure_lab.out` sections A and D.

### Why `cells4` is out

The failure was `LimitExceeded { resource: "retained profile knots", requested: 1048823, limit: 1048576 }`, at engine pin a399d626, `crates/fmn-render/src/plan.rs:110`.

A blended strip of n cells holds 2n + 9 knots. Then 713 rows × (2 × 731 + 9) = 1,048,823, which is exactly the failure's number. The whole 6.4 plate needs 731 × 1471 = 1,075,301, or 102.5% of the limit, so the job stopped at row 713 of 731. This is a count, and a faster machine would not change it.

At `cells2` the same formula leaves room, though I derived these figures and did not observe them:

| Scene at `cells2` | Knots | % of limit |
|---|---|---|
| 6.4 plate | 271,206 | 25.9 |
| Lacunary, `continuing.rs:675` | 193,743 | 18.5 |
| Mosaic of six panels, `gallery.rs:857` | 437,166 | 41.7 |
| Mosaic and the 5.0 zoom, if both are live | 603,332 | 57.5 |
| A whole 16:9 frame of plate, which no scene does | 747,195 | 71.3 |

## What I read by eye

These are lab stills and fades at exact pixels. They are not film frames, encoded video or 4K. A note for every image, with its hash, is in `coverage/evidence.tsv`.

- **Lattice:** plain at 100% under `now`, most of all in flat fields such as `tan`'s olive. `tile` and every rung above it are clean, including through fades on `z`, `tan` and `bump`.
- **`overlap`:** bright lines at the partly faded frames, gone at the hold. `overlap06` has fainter lines, and faint rows that remain at 3×.
- **Stairs:** visible at 100% under `now`, `tile` and `blend`. On long edges (`zoom`), the gain at `rows2` is clear, and there is a second gain at `cells2`.
- **`rows2` against `cells2`:** `rows2` halves the near-level steps, but it squashes rings sideways near centres (`z2`, `tan`). `cells2` keeps them round and is plainly nearest truth.
- **`blend`:** its long steps on level edges read as directional (`sin`), as `REPAIR.md` warned. I did not see the muddiness I expected on `sinrecip` or `recip`.
- **Plate edge:** `now` and `overlap06` have a dim first row. `tile` and up meet the background cleanly, and I found no fault in the harder edge.
- **Repaints, `z2` to z³ − 1:** `now`, `blend` and `rows2` all cross cleanly.

## Findings

**F1, High: the selected candidate has the least evidence in motion.** `cells2` has no fade and no repaint at any size, and `rows2` has no fade. My plan left them out. Fades are exactly where `overlap` failed after looking clean as a still. The probe patch adds a `z` fade for both and a `z2` repaint for `cells2`, and probes B and C run them.

**F2, High: the real cost of `cells2` is unmeasured.** The 1.82 is stills only.
- A fade or repaint makes every row's style new at each step. By my derivation from the source, compaction on the 6.4 plate would fire about every 3 frames under `cells2`, against about every 7 under `now` and `rows2`, and every 15 under `blend`.
- The lab's fades run at 3 fps, so they cannot show this.
- The Painter re-evaluates its function at each of 12 blend steps or 20 enhance steps, uncached, and `cells2` has four times the cells. That is unmeasured on gamma, zeta and lacunary, none of which are in the lab.

The chapter run closes this, and root owns the decision on cost.

**F3, Medium: `cells2` moves the black cell to two new plates.** `shade()` has no colour for a value of exactly 0 or a non-finite value, so `draw.rs` paints the background there. `checks/tables.out` section 5 tests every painted plate in f64:

| Rung | Painted plates with a cell exactly on a zero or pole |
|---|---|
| `now` (the film today) | `reading_color` plane: Identity, Square and Recip at 0 |
| `rows2` | `reading_color` and `residues` Bump at ±i; zeta plane at −2; mosaic Gamma at 0, −1, −2, −3, −4 |
| `cells2` | **Lacunary at 0 (`continuing.rs:675`); \|z\|² at 0 (`one_point.rs:410`)** |

- **Why these two:** 5.4 / 0.0175 rounds to 309, an odd count, where today's 154 is even.
- **No mark covers them:** `Func::zeros` is empty for both functions, so `marks` draws nothing there.
- **Size:** today's cell is 4 px wide at 720p, darkest luma 16.7. A `cells2` cell is 4.7 px at 4K, and blending should carry it about two cells along its row, about 9 px. That size is derived, because neither plate has been rendered.
- **Severity:** it sits on a true zero, so it does not misstate the mathematics. It is still a black speck that `cells2` adds, so read it in probe A. If it reads badly, the smallest repair I can see is one more row and column on those two plates. I have not written that repair.

I did not check which of the `rows2` hits are covered by marks.

**F4, Medium: no rung gets the `sin(1/z)` core right** (`rigidity.rs:416`). Truth is grey there.
- `now`, `tile` and `blend` show false blocks.
- `rows2` shows horizontal ribbons.
- `cells2` shows finer false loops, though it is the nearest overall (rms 4.49 against `tile`'s 9.22).
- `tile_p3` goes grey, in blocks, which suggests more samples help the core.

The probe patch adds a one-frame `cells2_p3` on this subject, so root can judge whether this one plate is worth it.

**F5, Medium: some film plates are not in the lab.**

| Gap | Answer |
|---|---|
| Late lacunary plate (Mail 2711, chapter 14 at 153.5–173 s) | **Confirmed not in the lab.** The probe patch adds it, and the chapter run reads that window |
| \|z\|² plate | The probe patch adds it |
| `outside_mask` (`continuing.rs:139`, opening bump) | Chapter run only |
| `tan`, which the film repaints from `sin` (`gallery.rs:591`) and the lab built directly | Chapter run only |
| Gamma, zeta, mosaic | Chapter run only |
| `boundary.rs:171` bump | Same function and width as `openbump`; not added |

**F6, my own errors.**
- `cells4` was in my plan. My stub tests could not count knots, so they passed a plate that cannot render.
- `sheets/recip.png` has a blank truth cell, from a negative slice in my frozen script's sheet code. `evidence/stairs-recip-2x.png` is the corrected view.
- Section F of the measure compares repaints with a mirror on the `now` grid, which marks finer grids down: `rows2` got r 0.948. I also read `rows2` as stretched by eye. Both were wrong. `checks/repaint_geometry.out` compares per pixel with no grid: the best fit is stretch 1.000, shift 0, and `rows2` is the closest of the three (rms 6.48, against `blend` 7.61 and `now` 8.78).
- Seven sheets reached me as JPEG, and the widest parts were shrunk. The viewing proof caught this, so I cut them into exact PNG parts and read those.

## Predictions against measurements

| Prediction (`REPAIR.md`) | Measured at 720p | |
|---|---|---|
| `now`'s dark rows fall where the mirror puts them | r +1.000 on `z`; r ≥ 0.990 on all 18 scenes | Held |
| `tile` on `z` under 1% dark rows | 0.4% | Held |
| `tile` and `blend` fades near 0, rms under 1 | −0.16 to +0.26, rms up to 0.39 | Held |
| `overlap` fade about +19 | +16.68 | Held |
| `overlap06` fade about +15 at 4K | +7.52, where 0.6 px at 4K is only 0.2 px | **Not settled by 720p.** Moot for selection: it leaves the stairs |
| Hold frame equals the still | rms 0.000 on all 11 fades | Held |
| `flat_control` fails, as planted | rms 70.10 | Held |
| Stair order `now` > `blend` > `rows2` > `cells2` | Holds on all nine subjects; desk values were 4K and do not compare in size | Held |
| `cells4` runs | Failed at row 713 of 731 | **Wrong** |

All 16 of the measure's controls pass, and a second run gives the same numbers.

## Remaining bounded probes

No probe has been run, and nothing here repeats a finished job. `L` is the lab binary rebuilt on a tree with A + B + `patches/probe-plate-lab.patch`. That patch is uncompiled. Its desk checks: it applies and reverses with `patch -p1` back to B's bytes, `rustfmt --check` passes, and the lab's own frame-total test now expects 154.

It adds subjects `lacunary` and `abs2` and candidate `cells2_p3`, plus the stills, fade and repaint below. To stay under the 160-frame limit it removes the `cells4` stills and the `z` fades of `overlap`, `overlap06` and `blend`. `blend`'s `z` fade matched `tile`'s at 720p, and its `tan` and `bump` fades stay in the plan.

**4K, 52 lossless frames:**

```
$L NEW_A --res 3840x2160 --stage stills   --subjects z,z2,sinrecip,lacunary,abs2 --candidates now,rows2,cells2,cells2_p3
$L NEW_B --res 3840x2160 --stage fades    --subjects z  --candidates now,tile,rows2,cells2 --no-truth
$L NEW_C --res 3840x2160 --stage repaints --subjects z2 --candidates now,rows2,cells2 --no-truth
```

| Run | Frames | What it settles |
|---|---|---|
| A | 15, plus 5 truth portraits | Stairs and lattice at 4K, F3's black cells, F4's core, the lacunary plate |
| B | 28 | `cells2` and `rows2` in a fade at 4K, with `now` and `tile` as references |
| C | 9 | `cells2` in a repaint, with `now` and `rows2` as references |

Measure each with `python3 -B checks/measure_probe.py NEW_x --sheets NEW_SHEETS`. It imports the frozen measure unchanged and adds the two functions. `cells2_p3` must be viewed directly, since the sheets have fixed candidate lists. For scale, both 720p jobs together took under four minutes, and 4K has nine times the pixels. No 4K timing exists.

**Encoded check:** put `fades/z__cells2` (7 frames) and one lacunary still through the film's final encoder settings, decode them, and read them. The lab stops at PNG, and fine cells are the likeliest thing an encoder would soften.

**Chapter run, 720p, on a tree with A + `select-cells2.patch`:**

```
target/debug/complex_analysis_visual render 04_reading_color 05_gallery 14_continuation \
  --res 1280x720 --fps 30 --scratch-voice --lint --crf 26 --preset ultrafast --threads 4 --out NEW
```

This is the first build of `select-cells2.patch`: root's Mail 2746 build covered A and B, and the patch's own strip test (at most 400 rows and columns) has not run. Confirm first that the binary accepts several targets. 720p is enough here because knots come from records and do not depend on resolution, so the same compactions fire. Read chapter 14's window at 153.5–173 s for Mail 2711.

Compare frames a second with preview02, knowing the source has moved since:

| Chapter | Frames | Seconds | fps |
|---|---|---|---|
| 04 | 3,566 | 554.5 | 6.4 |
| 05 | 4,956 | 933.7 | 5.3 |
| 14 | 6,121 | 570.7 | 10.7 |
| Together | 14,643 | 2,058.9 | |
| Whole film | 62,060 | 5,389.5 | |

**Decision rule.** Keep `cells2` if all of these hold. Otherwise take `rows2`, if it clears the same fade and repaint tests. If both fail in a fade, `blend` is what remains: it cleared fades at 720p, and it keeps the stairs.
- The 4K controls hold.
- Fade rows stay within about 1 luma of ideal.
- Hold against still is rms 0.000.
- No `LimitExceeded` occurs in the chapter run.
- Root accepts the measured frames a second.

## Provenance

Nothing was copied or altered. `provenance/originals.tsv` lists all 245 original files with hashes, and `provenance/scene-sources.tsv` says which lab holds each of the 79 scenes (65 stills, 11 fades, 3 repaints).

| Lab folder | Files |
|---|---|
| `lab720-01` (kept after the `cells4` failure) | 128 |
| `lab720-fades02` | 102 |
| `lab720-controls02` | 15 |

`checks/consolidate.py` proves that:
- `plates.tsv` is byte-equal across the three labs (`853de9f7…`);
- both jobs used the same binary (`c2832822…`, a debug build);
- the lab's crate files equal my frozen candidate;
- A and B were bound as inputs to both jobs.

The jobs were `complex-color-plates720-01` (exit 1) and `complex-color-remaining720-02` (exit 0). There is no `cells4` folder.

Patches: A `572e09d2…`, B `25aa77fd…`, `select-cells2.patch` `a17665e5…`, `select-rows2.patch` `c58f2975…`, `probe-plate-lab.patch` `3c28dd96…`.

## Limits

- 720p lab frames of one plate on an empty stage. They are not film frames, 4K or encoded, and there is no audio. Every statement about what I read refers to still images. I did not watch or listen to anything.
- The notes in `coverage/evidence.tsv` were typed by eye as I read each image. `checks/viewing_proof.out` proves that all 56 images reached me at exact pixels, directly or through parts (RESULT 56 of 56). It proves delivery, not what was noticed.
- Seconds come from one debug run per scene. Compaction counts and the knot table are derived from source, apart from the one failure they reproduce exactly.
- The 160 bin and 48 example tests and Clippy are root's result from Mail 2746. I did not rerun them. I ran no Cargo here, so the probe patch and `select-cells2.patch` are uncompiled.
- `coverage/evidence.tsv` covers what I looked at. It does not replace root's own reading.

## Reproduce

From the repo root, with no Cargo and no render. These only read root's lab folders and write here and in a new scratch folder.

```
python3 -B <bundle>/checks/run_checks.py NEW_SCRATCH SESSION_ID
```

It re-runs consolidation, the frozen measure (requiring the same numbers as the saved run), the repaint geometry, the tables, the probe measure's self-test, the evidence table and the viewing proof. `SESSION_ID` is this session's transcript, `6d8d91ab-386b-435d-91e3-ad665c8e7f76`, which only the viewing proof needs. The last run was all OK.
