# Rate-law chapter: independent source review

**Reviewer:** WhiteSwan, for BlueSalmon's Mails 1581 and 1588, bead 87do.62. I did not write root's candidate.

**Evidence:** I read the source and the pinned API, ran `rustfmt --check`, and ran the Python arithmetic in `checks/`. There was no native build, no test run, no render and no audio. This is a source verdict, not acceptance.

## Verdict

**The maths and the native design hold, but the candidate doesn't compile yet, and the picture runs well ahead of the voice.**

- One line won't compile (F1).
- One spoken sentence is false as worded, and another claims too much (F2, F3).
- The picture is mostly complete long before the voice describes it, then stays still through the chapter's key caveat (F4–F9).

All of this is small to fix. The corrections are staged here, and root's candidate is untouched.

## Bindings

| File in `exponential-expanded-rate01/` | Root sha256 | Candidate sha256 |
|---|---|---|
| `rate_law.rs` | `45c9333ff1294b37d8426f2b0df75706de0f1d9d9076b48ac6904aa51997ac9e` | `093f37e54b73b8e4634c8fea365ae8110a374e565747cbb86d17a1ebfa9f120a` |
| `rate_model.rs` | `e3e3f7e99c46f07450eeb9a93b408c36d8d12cdb02fd979af408e56701516ce1` | `48f3dae06f4b696afe24012b3bafa85ac59522925527d704674c13d4cc71189d` |
| `script-slice.tsv` | `f3374d28a78e4757c73c3187272cbeb9ed88228cfb883706221c0a4a1422c89d` | `1b5257beac7dd0e08a300c4625d8914f80ebc8af5bed718b7556587f6317e63e` |

| Patch | sha256 | Covers |
|---|---|---|
| `patches/rate-law-source.patch` | `0cfe1a4f8aed242eb845cfc7196600a5a39a6f8f19016f969510409ffa221f01` | `rate_law.rs`, `rate_model.rs` |
| `patches/script-wording.patch` | `c2d2d44c04c0df0571a95979c6a5051a8a6e9b3acff007b9e85b26b5a583da4c` | `script-slice.tsv` |

Apply them with `-p1` inside a copy of root's folder. The two patches are independent: the wording changes keep every cue's sentence count, so all the `until_sentence` indices still hold.

Also read, not changed: `rational-oracle01.json` (`8d685af24394d0f3`), root's `README.md` and the before-timing version of `rate_law.rs`.

## Findings

Line numbers are root's `rate_law.rs`. Timings use the Kit's scratch rule at factor 1.0, with the range for factors 0.8–1.25 in brackets. **They are arithmetic, not measurement,** because no Oliver take exists for this script.

| # | Severity | Problem | Staged fix |
|---|---|---|---|
| F1 | Blocker | `:98` reads `.y`, but `Vec3` is `[f64; 3]` | Index with `[1]` |
| F2 | High, maths | 01_5 says "there is only one", which is false as spoken | Reword |
| F3 | High, maths | "These straight steps are approximations" also covers the exact left rule | Reword, and move the footer to the right panel |
| F4 | High, timing | The whole of 01_1's picture is up by 1.0 s | Reveal by sentence |
| F5 | High, timing | The footer appears 49 s before its words | Show it at 01_3 sentence 4 |
| F6 | High, teaching | "Read the height" is never shown | Read-outs show height and slope |
| F7 | High, timing | 01_3 goes still for its last 14.8 s | Pace the steps, and show the kept slope |
| F8 | Medium, teaching | A stale "slope 2.44", and "closer together" has no lasting evidence | Show the endpoints so far |
| F9 | Medium, timing | The promise appears 12 s before its words | Wait for sentence 2 |
| F10 | Low, score | The think gap isn't an authored breath | `music_breath` |
| F11 | Integration | Printed numbers aren't in `facts.rs` | Rows listed below, for root |

### F1: `.y` doesn't compile (found independently; root raised the same line in Mail 1588)

`rate_law.rs:98` is `point(left, 0.0, f64::from(y)).y`. At fmn pin a399d626, `Vec3` is `[Semantic; 3]` and `Semantic` is `f64` (`fmn-core/src/types.rs:5`, `:13`), so an array has no field `y`. The corpus indexes instead, as in `bakery_algorithm/src/chapters/scoreboard.rs:465`.

This is the only compile error I found by reading. A first native build can still find others.

### F2: "only one" is false without the starting height

01_5 says: "We have not yet proved that the stepped approximations reach such a curve, or that there is only one." "Such a curve" is "a curve whose slope equals its height". Every `y = Ce^t` is one, including `y = 0`, so there are infinitely many. The brief's own proof needs the start: "start at the same height" (`:286`) and `y′=y, y(0)=1` (`:296`).

New wording: "…or that only one such curve starts at height one."

### F3: the left rule's steps are exact

01_3 says "These straight steps are approximations" right after describing both drawings, and "the true slope would change too" is true only of the right rule. On the left, the steps lie exactly on `y = 1 + t`. The footer at `:69–75` makes the same claim across both panels.

New wording: "Its straight steps are approximations", following "The right drawing keeps steepening." The footer moves under the right panel.

### F4: 01_1 is complete before its words

`:109` and `:123` show both rules, both slope read-outs and the question by 1.0 s.

| Item | On screen | Voice reaches it | Early by |
|---|---|---|---|
| "rate = current height" | 0.6 s | 7.1 s | 6.5 s |
| "Will the two curves stay together?" | 1.0 s | 13.3 s | 12.3 s (9.6–15.6) |

So the viewer reads the question for about 12 s before being asked it, against the skill's "say before the picture".

The fix reveals the left rule at sentence 2, the right rule at 3, ticks and read-outs at 4, and the question at 5. Those ticks were also the only mark of the start, so I added a small dot at height one in each panel for "Start at height one".

### F5: the footer is 49 s early

"straight steps: an approximation" (`:69–75`) is on screen from 0.6 s, before any step exists. The voice says it about 50 s later, so it's early by 49.4 s (40.5–60.6). It now appears at 01_3 sentence 4.

### F6: the height the rule depends on is never on screen

The voice says "read the height" three times, but the axes only have ticks at 0, 1, 2 and 3, and the read-out says only "slope 1.25". So "slope equals height" has to be taken on trust. The left read-out stays at "slope 1.00" with no sign that the left height has changed too, which is the whole contrast between the rules.

The read-outs now print both, from the model:

| Panel | After step 1 | After step 4 |
|---|---|---|
| Left | `height 1.25: slope 1.00` | `height 2.00: slope 1.00` |
| Right | `height 1.25: slope 1.25` | `height 2.44: slope 2.44` |

### F7: 01_3 stops changing at 4.8 s

All three steps finish by 5.4 s, in a line of 19.6 s. Nothing changes for the last 14.8 s (10.9–19.7), including the chapter's main caveat: "During a step the true height would already have changed."

The fix places the steps with the sentences:

- Step 2's read-out lands at sentence 1.
- Step 3 lands at sentence 2 ("The left drawing…").
- Step 4 lands at sentence 3 ("The right drawing…").
- The footer lands at sentence 4.
- At sentence 5 the read-out becomes `last step: slope 1.95, height now 2.44`, while the steeper tick sits at the end of the shallower segment.

That leaves 5.9 s still (4.7–7.5) while the viewer reads it. The tightest margin is at factor 0.8, where steps finish 0.11–0.15 s before their next sentence. If a voice runs faster, `until_sentence` simply doesn't wait and the step lands slightly late, so nothing breaks.

### F8: a stale read-out, and no lasting evidence for "closer together"

Root's comment at `:171–172` correctly says the old slope would mislabel the new approximation, and `:173` removes the tick. But the read-out still says "slope 2.44" until 9.7 s into 01_4 (7.8–12.0). The 4-step endpoint also never gets a label.

"Closer together" is true: the end gap falls from 0.1244 to 0.0721. But only the end of the curves can show it, because the blue stroke is 0.035 scene units wide:

| t | 4 vs 8 gap | 8 vs 16 gap |
|---|---|---|
| 0.25 | 0.016 | 0.009 |
| 0.50 | 0.041 | 0.023 |
| 0.75 | 0.077 | 0.044 |
| 1.00 | 0.129 | 0.075 |

Root's label shows only the latest endpoint. The new read-out keeps them all: `4 steps: 2.441`, then `4, 8 steps: 2.441, 2.566`, then `4, 8, 16 steps: 2.441, 2.566, 2.638`.

### F9: the promise is early

`:225` shows "Next: construct an exact solution" 0.4 s into 01_5. The voice reaches it at 12.5 s, so it's early by 12.1 s (9.6–15.2). It now waits for sentence 2.

### F10: the think gap isn't an authored breath

`:126` uses `stage.wait(4.0)`. It works: the `predict` mark alone protects the gap in the director (`tools/narration_director.py:202–204`, `:363`). But only breath and listen rows get the score's `release_windows` and `recede` (`explainer_score/src/score.rs:2726`, `:2734`). The gap is 4.9 s, over `HOLD_MERGE` (`:1138`), so any audible difference is probably small.

`kit.music_breath(stage, 4.0)` is a one-line change that matches logarithm's `predict`. Listening will have to judge it.

### F11: numbers bypass `facts.rs` (for root's integration)

The hard rule is that every on-screen number comes from `facts.rs`, pinned against an oracle. The candidate prints from `rate_model`, and `rational-oracle01.json` isn't wired to anything. I didn't add this, since `facts.rs` and `oracle.py` aren't in this candidate. `exponential_magic`'s `facts.rs` already has the pattern, so these rows fit it:

| Key | Values |
|---|---|
| `rate.height.1`–`.4` | 1.25, 1.56, 1.95, 2.44 |
| `rate.line_height.1`–`.4` | 1.25, 1.50, 1.75, 2.00 |
| `rate.end.4`, `.8`, `.16` | 2.441, 2.566, 2.638 |

These agree with the crate's existing `limit.4` and `limit.16` at five places. The staged test `every_printed_number_is_the_exact_rational_rounded` pins the same strings until that wiring exists.

## What holds

| Area | Result | Evidence |
|---|---|---|
| Maths | All 3 oracle tables reproduced: every endpoint, fraction and display string. The recurrence equals `(1+1/n)^k` | `Fraction` run, section 1 |
| "Climbs slightly higher" | True. The 8-step drawing is identical to the 4-step until t = 1/8, then above it | Exact 1/16 grid, section 2 |
| Rust model | Root's two tests are exact in binary for these n | Reading; never run |
| Think-time rule | Only 1.00 and 1.00 are visible during the gap, so the answer doesn't leak | Reading |
| Native API | Everything other than F1 exists at the pin or has corpus precedent: prelude exports, `Stage::prepare` and `wait` (`fmn/src/lib.rs:1449`, `:1473`), `Mob::animate` (`fmn-mobject/src/animate.rs:231`), `fade_out` (`fmn-anim/src/fading.rs:80`), `kit.mark` on `&Stage` | Reading; not type-checked |
| Lifetimes | `morph` keeps the on-screen handles, and targets stay detached. Each `fade_out` is single, with its handle still in the scene. Coarse segments stay on purpose. `old_right` doesn't dim anything twice | Reading |
| Grey lines | `set_color_with_opacity` also sets fill alpha (`animate.rs:763–767`), but a straight two-point `Line` has no area to fill | Reasoning; unseen |
| Geometry | Ticks stay under the top of the axis and clear of neighbours (last right tick ends at x = 5.54, y = 1.03). Slopes draw at 12.9° to 29.3° on unequal axes, consistently. All text fits, with no overlaps, by estimated widths; the widest read-out is 3.71 in a 4.5 panel | Section 3–4; layout lint not run |
| Length | About 105 s (85–129) is not a defect: `AGENTS.md:87` supersedes the 85 s cap | Arithmetic |

`kit.hold` directly before `kit.say` is redundant under `LineGaps::Constant`, but harmless, so I left it.

## Change map

| Fix | Candidate `rate_law.rs` | Test |
|---|---|---|
| F1 | `:89` | None; a build proves it |
| Start dots | `:99–104` | None |
| F4 | `:107–135` | `each_reveal_waits_for_the_sentence_that_names_it` |
| F10 | `:138` | `the_think_gap_is_authored_and_nothing_new_appears_during_it` |
| F7 | `:151–153`, `:173–175`, `:207–216` | The reveal test above |
| F6 | `:178`, plus the first read-outs | `every_printed_number_is_the_exact_rational_rounded` (`rate_model.rs:72`) |
| F3, F5 | `:198–206` | The reveal test above |
| F8 | `:224–231`, `:252` | `no_refinement_runs_under_the_old_slope_or_its_read_out`; `each_refinement_climbs_above_the_coarser_drawing_and_the_end_gap_shrinks` (`rate_model.rs:96`) |
| F9 | `:274` | The reveal test above |
| F2, F3 wording | `script-slice.tsv` | Word and sentence counts in the checks |

The label helpers are at `rate_model.rs:28–43`. The slice grows from 244 to 248 words, and every cue stays under 100.

## Left for root

**Decisions**

1. **The brief's picture.** The brief asks for a growing blue bar and a tangent arrow on one value tracker, and "native FrankenManim trackers, curves, TeX" (`:166–167`, `:171`). Root's two stepped panels use no tracker or TeX, and the line "At height two, climb twice as fast" (`:179`) is gone. I think the stepped panels teach well, and F6 covers the missing height, so I didn't change this. If you want a physical height, a vertical line from the axis to the endpoint would morph with each tick and leave with `right_slope`.
2. **01_2's read-outs land at sentence 2.** That's root's timing repair and I kept it. The steeper right tick then lands 4.5 s (3.6–5.6) before "The right rule asks for a steeper slope". Judge this on the first picture pass.
3. **01_5 is still for 14 s before the promise.** Its open questions have no picture. Listing them on screen is an option, not a defect.

**Integration**

- Add `mod rate_model` and `mod rate_law`, plus a registry row.
- Renumber the cues: `01_1`–`01_3` already exist at `script.tsv:3–5`. The literal ids are also in the new tests' strings, so renumber those too.
- Register the glyphs, including the runtime read-out characters: letters from `height`, `slope`, `last step`, `now`, `steps`, plus digits, `.`, `,` and `:`.
- Add the F11 facts and oracle rows.
- Declare `01_1` to `01_2` as a prediction hold in the final map, or native captions fail at `final_check` (see my `PREDICTION-MARKER-REVIEW.md`). That pair can't be a sounding coherent seam.

## Checks

**Run on the final bytes**

| Check | Result |
|---|---|
| `rustfmt --check --edition 2024`, root's two files and both candidate files | Exit 0 on all four. Parsing and formatting only, not types |
| `checks/rate_checks.py` | 64 of 64, 0 failures. Output in `rate_checks.out.txt` |
| Patch replay on a clean copy of root's three files | All three reproduce the candidate hashes |
| Machine-path scan of this folder | Clean |

**Not run**

- **Native:** Cargo build, tests and Clippy. **Root's Rust and all my changes are uncompiled, and every Rust test here has never run.** The Python mirrors check only numbers and source order.
- **Rust against the oracle:** the f64 recurrence was run in Python, not Rust.
- **Pictures:** no render, frames or layout lint. Text morphs across glyph counts are legal by reading (`fmn-library/src/text.rs:466–469`, `fmn-anim/src/transform.rs:517`), but how they look is unseen. Whether 3.5-weight lines read as "slightly higher" is unseen too.
- **Sound:** no narration exists for this slice, so nothing was heard, and there's no fine sync or independent AV review. Oliver's timing is unknown, so every time above could move.
- **Review of my own changes:** the staged candidate is my work and nobody independent has read it.

No canonical, Beads, memory or git edits were made.
