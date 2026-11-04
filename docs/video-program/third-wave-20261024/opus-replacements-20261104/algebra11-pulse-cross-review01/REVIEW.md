# Algebra chapter 11: pulse repair cross-review

WhiteSwan, for BlueSalmon Mail 2038. Narration stays ElevenLabs Oliver; this review touches no narration.

**Verdict: root's hunk is sound and I found no defect in it.** It keeps the old pulse's factor, pivot, duration and rate, stops the repaint, and returns every mobject exactly to where it began. This rests on reading the source at fmn pin a399d626 and on running the engine's own arithmetic cut out of that source.

The proposed regression test, `patches/pulse-regression-test.patch`, compiles and passes Clippy in type-check but **has never run**, because fmn cannot be linked here. Native compilation, real pictures and Oliver listening all remain open.

## Author overlap

This review is independent of root's hunk, but not of the chapter:

- **I suggested this remedy.** Finding H of my earlier review (`algebra11-patch-cross-review01/REVIEW.md:147`, Mail 1909) named the defect and said "Filled tokens need a scale-only pulse or a `halo`". Root's hunk is the first of those.
- **My earlier code is in the base.** All 22 non-empty lines added by my `algebra11-patch-cross-review01/patches/permutations-followup.patch` are in `candidate02/permutations.rs`. None of them is in the pulse hunk, which is root's.
- **The test and the checks here are mine.** They need another reader, and their native result needs root's lane.

## The hunk

The two files differ in one place, `fn pulse` at `candidate02/permutations.rs:806`. One line is replaced by ten, a net +9. Root's patch applies at fuzz 0, gives candidate02's hash, and reverses. The file has no `indicate`, `yellow` or `add_updater` left.

| Property | Old `indicate(…, None).rt(0.6)` | New `animate().scale(1.25)` | Evidence |
|---|---|---|---|
| Size change | `stage.scale(target, 1.25)` | the same call | `fmn-anim/src/transform.rs:350`, `fmn-mobject/src/animate.rs:683` |
| Pivot | the box centre | the same | `fmn-mobject/src/positional.rs:942`; pivot held on all 36 frames in `engine_cut.py` |
| Duration | 0.6 s, 36 frames at 60 fps | the same | `frames_covering` run in `engine_cut.py` |
| Rate | `rate::there_and_back` | the same function | `fmn-anim/src/indication.rs:72`; `fmn-core/src/rate.rs:53` |
| Colour | every `*rgba` field of the target set to yellow | not touched | `fmn-anim/src/transform.rs:311` |
| Target built | at begin | at prepare | `fmn-mobject/src/animate.rs:627`; `pulse` prepares and plays at once, so nothing changes in between |
| Purity class | `Pure` | `Pure` | `fmn-anim/src/transform.rs:488`, `fmn-anim/src/method_animation.rs:55` |
| Animation name | "Indicate" | "MethodAnimation" | no consumer found in `crates/`, `fmn-scene` or `fmn/src` |

The frame count is unchanged, so no hold or cue time moves. Only source line numbers do.

**Style.** The target is a copy with only its placement changed, so every record field matches the start. `lock_matching_data` (`fmn-anim/src/transform.rs:228`) locks matching fields, and locked fields are skipped by interpolation. It only stands aside when a family has updaters, and this file adds none. So disk and numeral should keep their exact colours on every frame. Stroke widths do not change, as before. This is from reading: only the proposed test can show it running.

**End state.** `there_and_back(1.0)` is exactly 0, so the last frame interpolates at 0, and `(1 - 0) * start + 0 * target` is the start exactly. `engine_cut.py` ran this on 360 start placements and pivots: the end equalled the start and frame 18 equalled the target, both with `==`. No replacement happens at clean-up, and teardown unlocks the fields (`fmn-anim/src/transform.rs:259`), so later `recolor` and `travel` calls are free to change them.

`rate_func` is what makes this work. Without it the default `smooth` rate leaves the mobject at 1.25 for good, at only 1.125 at the midpoint (run as a control).

**Map rows.** Exactly six rows changed, each by +9 and each naming candidate02's hash: 11_2 to 4124, 11_3 to 4174, 11_12 to 4186, 11_15 to 4198, 11_6 to 4285, 11_10 to 4342. Every permutations row's line still holds `k.hold(stage)?;`, and the nearest `say` above it is that row's cue. Nothing else in the map changed, and candidate02's units name candidate02's map hash.

## Notes for root

Neither note is a defect in the hunk.

**1. Nine of the 13 sites now emphasise by size alone, and only pictures can judge that.** They used to flash yellow as well. No chapter 11 line says yellow, flash, highlight, lit, lights up or glow, so the voice does not depend on the colour. The geometry is the same as the old route's, so any overlap at the peak is not new.

I would look first at line 1557, where only an ink outline swells and its number stays still, and at line 3197, where the swelling text is small. At the four token sites, check that the numeral stays readable on its disk at the peak. At 3205 and 3206 the new route should be better: the gold and blue that were just painted no longer flash yellow.

| Line | What swells | Kind | Cue and sentence index (from 0) | Voice from that anchor |
|---|---|---|---|---|
| 1412 | cards 4, top and bottom | token | 11_4, 5 | So the trails form disjoint cycles. |
| 1557 | one chip's frame | stroke | 11_11, 1 | The difference between the two swapped labels reverses. |
| 2637 | card 4 | token | 11_14, 4 | Card four never moves, so it adds no condition. |
| 3066 | swap counter | text | 11_20, 2 | They are not an empty collection… |
| 3130 | the subgroup test | maths | 11_20, 6 | So a times b inverse stays even, and the test passes. |
| 3197 | first cell on each side | text | 11_21, 2 | Follow every arrangement by one extra swap of the first two cards. |
| 3205 | all right cells, blue | text | 11_21, 3 | Each even arrangement becomes odd… |
| 3206 | all left cells, gold | text | 11_21, 3 | (same sentence, straight after 3205) |
| 3590 | the written cycles | maths | 11_23, 3 | One shuffle was enough to read the cycles… |
| 3773 | the prediction and the count | maths, text | 11_24, 3 | Watch the staircase break apart and come back. |
| 3836 | both return counts | text | 11_25, 3 | Watching the hands would not tell you that… |
| 4085 | card 1 | token | 11_1, 3 | Let's follow card one to its new position… |
| 4093 | card 2 | token | 11_2, 2 | The label two stays where it is… |

The anchor is the last `until_sentence` before the pulse. The pulse can start later, because other animations run first at some sites.

**2. The same fault exists outside this chapter.** `transitivity.rs:694` pulses three disks with dark numbers (grouped at `:685–686`, cue 03_9) through the flat `indicate` at `transitivity.rs:145`. That is the one further case I confirmed, from source only. I ruled out `cyclic.rs:717` (ink text and a marker) and did not audit the rest. Twelve `indicate` calls remain outside `permutations.rs`, so chapter 11 will now emphasise differently from chapters that keep the yellow flash. Whether that matters is a call for root or the owner.

## The proposed test

`patches/pulse-regression-test.patch` appends `mod pulse_tests` after candidate02's last line. It calls the real `pulse`, `card`, `chip`, `k.text` and `k.tex` in a real headless `fmn::run_scene`, and reads each frame through `FramePacket::materialize_stage`. It needs no refactor. It pulses all four kinds together: token, stroked frame, text and maths.

| Test | It requires |
|---|---|
| `a_pulse_peaks_at_its_factor_halfway_and_keeps_each_centre` | 36 frames; index 17 at alpha exactly 0.5; there, each width and height 1.25 times the start and each centre unmoved (within 1e-5); the token's width rising strictly to the peak and falling strictly after |
| `a_pulse_never_repaints_any_member_on_any_frame` | every `*rgba` column of every family member equal to the start on every frame, exactly; at the peak, the token's fills still not all one colour |
| `a_pulse_ends_exactly_where_it_began` | last frame at alpha exactly 1; placement, uniforms, z-index and colours of every member equal to the start, both in that frame and live after `pulse` returns |
| `a_pulsed_token_can_still_be_recoloured_moved_and_pulsed_again` | `recolor` changes the colours and `travel` reaches its target after a pulse, so no lock is left behind; a second pulse returns exactly, so nothing compounds |

To run it in root's native lane, apply the patch to a copy of `candidate02/permutations.rs` in the crate and run `cargo test --bin abstract_algebra_expanded pulse_tests`. I have not timed it.

Then make each edit below in `fn pulse`, one at a time. All four compile in test cfg, so a failure will be an assertion and not a build error. **These outcomes are predictions from reading; none has run.**

| Control edit | Predicted failures |
|---|---|
| A: restore `indicate(stage.arena_mut(), mob, 1.25, None)?.rt(0.6)` | only the repaint test, on frame 1 |
| B: delete the `rate_func` line | the peak test (1.125, not 1.25), the end test (stays at 1.25), and the second-pulse test |
| C: `.scale(1.0)` | the peak test |
| D: `run_time: Some(0.5)` | the peak test, at the frame count (30) |

**Binding consequence.** The patch only appends, so every candidate02 line keeps its number and every hold row's line still holds its marker. The file's hash does change, to `64bbaa3a3a37816a0a40d23ca474a0dc9aed23db45a9323b53303d2a084c422f`. The `sha256` of the six permutations rows would have to follow, and then the units' map hash. I did not stage a third map.

The test reads engine state, not pixels. It shows the numeral keeps a different colour from its disk, but says nothing about how legible that is.

## What ran and what did not

| Check | Result | Covers |
|---|---|---|
| `checks/pulse_check.py` | 58 PASS, 0 FAIL | patch fidelity, map rows, the site census, the test patch, whole-crate type-check, controls, author overlap |
| `checks/engine_cut.py` | 11 PASS, 0 FAIL | rate, frame plan, placement and interpolation, on engine functions cut verbatim, std only |

The type-check ran `rustc --emit=metadata` with `-D warnings` and `clippy-driver`, in normal and test cfg. It used a scratch copy of canonical `src/`, with the patched file in place of `permutations.rs`. An early run of my check failed to apply the patch and still passed those type-checks, so the script now stops if the module is missing, and plants a type error and a Clippy fault to prove the module is read.

**Not done:** the test was never executed, and there was no fmn link, Cargo build, render or frame read. No take was made or compared, and nothing was listened to. The metadata comes from the existing `target/debug/deps` rmeta at pin a399d62 and the current uncommitted Kit tree. `engine_cut.py` runs no `Stage`, scene or animation object. **This is not native, picture or listening acceptance.**

## What this review binds

| File | SHA-256 |
|---|---|
| `narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/permutations-repair-stage01/candidate/permutations.rs` | `4f861c3b6704dfd89774909f88168bc7252a66cc54a4c5d62f1f547060c28473` |
| `narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/permutations-repair-stage01/candidate02/permutations.rs` | `0ba22ebd8903fe8c856776398b836cd50397ee44840518f6c28ef146aebfb958` |
| `narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/permutations-repair-stage01/pulse-color-preservation.patch` | `ef769b6f5f7afbb3bcef3bf1c832d074020c7cb82df5ffd2a79ce6357c8bea86` |
| `narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/permutations-repair-stage01/pulse-repair02.json` | `92ab96772f076125dc69e3f1bde87539920c6adf008ae189710ab0e76e43ee28` |
| `narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/permutations-repair-stage01/candidate02/narration-direction-map-oliver25.json` | `129b3ef924a7d3546c2040531733415a54362d7c25a13257c6c79160f1f8bf62` |
| `explainer_corpus/abstract_algebra_expanded/script.tsv` | `83619d48676b32d5d93bf7527477aa752a902dd16927c548e4b3052cc2d689c0` |
| `docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra11-pulse-cross-review01/patches/pulse-regression-test.patch` | `04ac3a348d90d3b01135f238a598872c7cd115546dde2107fc2920a9d1a8442f` |

Canonical source, root's staging and my earlier reviews were not edited. Other files in this folder are listed in `SHA256SUMS`.
