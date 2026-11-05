# complex13-14-cross-review01: chapter 13 radius repair and chapter 14 pole order

WhiteSwan, for BlueSalmon, answering Mail 2875. Bead 87do.39. Narration stays ElevenLabs Oliver.

**Verdict: I found no concrete fault in either repair, so there is no candidate patch.** Integrate `real_integral.rs` `a5ef573d…` and `continuing.rs` `c24f264b…` as they stand, then qualify them natively.

**That verdict rests on source, engine code and old pictures.** The crate was not compiled, nothing was rendered or heard, and no picture of either repair exists. Native pictures still decide two looks: A9 (the gap in the spoke behind "i") and B14 (the ring drawn once).

## What was asked

| Question | Answer | Evidence |
|---|---|---|
| Does pre-rooting put things where Bronze says? | **Yes.** I confirm root's reading: `add_to_scene` removes, re-appends, then runs a stable `sort_by` on z_index, and nothing in these chapters or the kit sets a z_index. A fade's begin roots only what is off stage and never re-roots. Its clean-up re-adds nothing: only `with_replace_flag` does that, and `fading.rs` never calls it | `order_walk.out.txt` quotes the pin's lines |
| 13_3 draw list during the walk | `spoke < arrows[0] < i_back < ring < "i" < zdot < … < r_back < R`. Spoke under arrow, ring, backing and name on every call that shows it, and the walker over the spoke's end | My own walk over 37 frame-producing calls |
| 14_3 draw list | Region, gold disk and all eight green disks stay under the mark on all 35 calls with the mark up. One mark only, top of the list at the chain's end | Same walk |
| Visible flash? | **None expected.** No frame-producing call lies between any re-add and its fade, and the first captured sample comes after interpolation. R and its backing are rooted at full paint, but the updater registers with `call = true` and writes opacity 0 before any frame can be captured. `i_back` is rooted at full paint with no fade, which shows nothing: on frame 88 only the letter lies inside it, and the backing is the frame's own ground colour to 3 levels | Pin, `geometry.out.txt` |
| Updater lifetime | **Sound.** The next statement after the walk clears it, well before the fade at sentence 2, so it cannot write opacity over `fade_out`. Opacity is exactly 1 at the walk's end. The growth updater names none of the removed mobs | `order_walk.out.txt` |
| `set_opacity` on R and the backings | **Safe.** It also writes stroke alpha, but both styles have zero stroke width, so no outline appears. The old trap only hits stroked open lines | Style code read |
| Geometry | **Holds, with small differences** (N1). R's box leaves the gold at 14.13° and R starts at 16.04°; its backing would notch the gold until 15.88°, also before it shows. Clearances below | `geometry.out.txt` |
| Teaching | **No regression, three small gains.** R is up well before the voice says it. The spoke behind the ring coincides with the trace touching its maximum. In 14_3 the dark hole over the region suits "every z except 1", and rims now visibly end at the pole | Below |

**Old frames confirm the 14_3 fault and the target look.** Zooms of 135, 153 and 162 show the gold rim and then the green disks wrecking the mark. Frames 165 and 188 show the second mark on top, which is the look the repair holds from the start.

## Findings

No blocker or concrete fault. These four need no code change.

**N1. The handoff's bands are slightly narrow, because they read at height 1 only.** Counting the letter's height and the stroke:

| Contact | Handoff | Recomputed | Scratch rate |
|---|---|---|---|
| Spoke on the letter "i" | 103.9–107.3°, 0.06 s | 102.6–109.1° | 0.117 s |
| Spoke on `i_back` | 100.5–110.6°, 0.18 s | 99.7–111.6° | 0.213 s |
| Spoke on the mark's backing | 82.9–97.1°, 0.26 s | 82.1–97.9° | 0.281 s |

So `i_back` costs about **1.8 times** the letter's own contact, not 3 times. That makes the backing slightly cheaper than the handoff suggests.

The two cuts never overlap: the spoke runs clear of both discs for 1.82° between them, 2 frames at the scratch rate. At most 18 px (by `i_back`) or 23 px (by the mark) of a 188 px spoke is hidden at any moment. **Keep `i_back` for the first native pass and judge it at A9.** Bronze's fallback (drop it, with the test lines that name it) stands.

**N2. A3 asks for R "just starting to show" at 16°, when R's opacity is still exactly 0.** It starts at 16.04°. Keep A3 for the gold line, and also extract **17° (12% R, the first frames where a notch could show) and 20° (50% R)**. Times come from Bronze's formula, `T0 + W × deg / 180`. I measured a start of 37.23 s and a 3.22 s walk on frames 90–95, with a worst fit of 0.18°, so the formula holds on the scratch voice.

**N3. "No precedent" is too narrow, and the risk is lower than the handoffs say.** Two precedents sit in the workspace at this same pin:
- **Root then fade:** `eigen_engineering/src/chapters/marks.rs:608` calls `add_to_scene(m)` then `fade_in_all(stage, &[m], 0.8)`, with the same `fade_in_all` body.
- **Opacity in an updater:** `bakery_algorithm/src/chapters/scoreboard.rs:721` calls `s.set_opacity` inside a dt updater.

I read that source only and did not look at those films' pixels. It lowers compile risk, but the first native render is still the proof.

**N4. The name "1" is 10.3 px clear of the backing on frame 103, not 12.** That is within the stated tolerance class and changes nothing.

Bronze's other "seen, not changed" items are accurate and match canonical: the walker and path pieces under later disks, the lit segment over the arrow head, and the arc walker over the top arrow head. None is a regression.

## Clearances

Measured at 720p on the old frames, with backings as ideal discs. Each figure is good to about a pixel.

| What | Gap |
|---|---|
| R's backing to the spoke | 10.2 px |
| R's backing to the half circle | 36.1 px |
| R's backing to the mark's backing | 19.4 px |
| R's backing to `i_back` | 18.5 px |
| R's backing to the gold | 0.4 px at the first ramp step, at opacity 0 and rising away; on frames from 10% opacity on, it hides nothing but the axis |
| `i_back` to the axis | 16.4 px |
| `i_back` to the mark's backing | 5.6 px |
| Ring to lit segment at R = 10 | 5.2 px |
| "i" to lit segment at R = 10 | 7.7 px |
| Walker's edge to the mark's backing in 14_3 | 0.714 plane units |

R's far corner is 11.4 px from its backing's centre, inside the 12.2 px radius. The letter "i" is 8.1 px against 9.0 px. R's end place holds only R on frames 96–110 and is empty on 111, so the backing's fade shows nothing.

**R arrives in time.** It is fully up before "R squared" whenever "so the bump" comes later than 1.65 s into the cue. That phrase has 52 letters before it, and the scratch voice takes 4.32 s. This is a share of letters, not a measured word time. At the 1.5 s floor the ramp lasts 4 frames, which is quick but harmless.

## Bindings

- Both patches apply to canonical, give the staged bytes, and reverse cleanly.
- Root's `combined-repair-stage02/candidate` holds the same two hashes.
- All four hold rows pass root's rule on the staged files: 13_5 at 631, 13_11 at 813, 14_3 at 522 and 14_4 at 531.
- **Timeline:** 13_3 is unchanged, 20 timed calls. 14_3 goes from 23 to 22, and the one removed is the second mark's 0.2 s fade, followed directly by the sentence 6 hook.
- Strings and phrase hooks in the film code are unchanged, and `script.tsv` is still `2968b0fd7654`.
- The `continuing.rs` plane literals asserted by the colour plate patches are still there. No patch that applies to canonical stops applying on the staged bytes.
- `pole_top` appears nowhere else in the crate. I did not open `closing15/`, as Mail 2875 asked.

## Prioritise at the native preview

Extract at exact times, not on the 0.5 s grid:

- **A3** plus the new 17° and 20° poses: R's ramp and the gold line.
- **A6:** the ring clean at the top.
- **A7:** the axis stops at R's backing, and the spoke passes through the gap.
- **A9:** the gap behind "i", which is the one judgement call.
- **A11:** no dark disc left after sentence 2.
- **B2, B4, B7:** fills and rims stop at the backing even while faint. This is what no source check can prove.
- **B12, B13:** all eight disks, and no pop where the second mark used to arrive.
- **B14:** whether one draw of the ring still reads as well.

## What was run

All in `checks/`, each with its saved `.out.txt`. All four report `RESULT PASS`.

| Check | Result | What it cannot show |
|---|---|---|
| `order_walk.py`, written without reading Bronze's scripts | 39 PASS. Canonical faults seen as positive controls (48 in 13_3, 137 in 14_3); 11 of 11 planted defects seen | Pixels. It models the draw list from source text |
| `standalone_tests.py`, `rustc` only | **Bronze's six tests ran for the first time and pass,** with the real `model.rs` and `Plane`/`Frame2` bodies lifted from source. `root_under` compiles verbatim against a stub. 20 of 20 planted defects caught. Clippy gives zero rows on the lifted code, and its control fires | Not `cargo test`. `construct` is not type-checked and `fmn` is not involved |
| `geometry.py` | 42 PASS: bands, clearances, the walker's rate and pixel checks | Old source only. Times are scratch rate or the floor |
| `bindings.py` | 51 PASS, including `rustfmt --check` on both staged files | Later merges, and any audio time |

`checks/baseline.sha256` pins the 31 inputs read. Both sealed folders verified with no failures.

**Pictures read by eye:** chapter 13 frames 88, 93 and 130; chapter 14 frames 103, 135, 147, 153, 162, 165 and 188, as crops and zooms of the mark; both synthetic sheets, which are not evidence. **Measured without viewing:** chapter 13 frames 90–96, 100, 105, 108, 110 and 111. All are preview02: canonical source, scratch voice, 720p. They show the old fault and the end state, never the repair.

## Limits and independence

- **I am not independent of this code.** I reviewed these chapters in `complex-preview01-review-11-15` (Mail 1623) and wrote `complex-picture-repair-11-15` (Mail 1655). **My `14-continuing.patch` introduced `pole_top`, the second mark this repair removes,** and my `13-real-integral.patch` wrote the placement note for R. So I am judging the replacement of my own earlier cure. Someone who has not worked on these chapters has not looked at this.
- **My reading was not blind.** I read Bronze's handoffs and outputs before writing my checks. The scripts are my own and recompute everything, but they test the questions those handoffs framed.
- **Not run:** no Cargo of any kind, no render, no native picture, no ASR, no TTS and no listening. Compile risk looks low from reading, but only the crate build can settle it.
- **Oliver:** every time here is scratch rate, a letter share or the 1.5 s floor, and none is Oliver's. The walk's length follows the "so the bump" hook, so real Oliver marks must precede sync acceptance, as Mail 2875 says. This review makes no listening, pacing or fine-sync claim, and independent final AV review is still open.
