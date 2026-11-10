# Independent review 711: exponential stage05 and stage06 source repairs

Reviewer: CC3 (WhiteSwan), 2026-11-09. Target: `narration_lab/third-wave-20261024/root/elevenlabs-exponential-20261101/expanded-stage06/native-crate`, with `expanded-stage05` and `expanded-stage03` compared.

Scope: source reading of the stage05 and stage06 changes, the Python oracle, and a small Python mirror of the 05_6 numbers and the ruler geometry. No Rust was compiled or run, and nothing was built, rendered or synthesized. No SSH, source edits or Beads. This makes no runtime, Clippy, picture, motion, listening, fine-sync or AV claim. Text widths and clearances are estimates from CMR metrics and source constants, and were not measured on frames. Every source stage is preserved as found.

Independence: CC1 wrote review 674, the fixes in 690 and the display-count repair in 696. I wrote none of them. I read 674 as prior evidence and judged the actual stage06 source myself.

| Bound to | sha256 (computed by me) |
|---|---|
| stage06 `native-crate`, 43 files, digest of `sha  path` lines sorted by path | `af10728022b856385ceb22055abb7c38dd3acbd0284470d37cb499753edf2d89` (equals handoff 696) |
| stage05 `native-crate`, same digest | `f4f274a3778b16e82470adcabe3a49b44383ea015f10b3833adbdfc13f2ad1b5` (equals 696) |
| `src/chapters/calculating_ruler.rs` | `d6f39c774909dff576c8d3a734caa9d7e6c3a5991273787bfc1373dff2a75400` |
| `src/chapters/tiny_multipliers.rs` | `5eb82f5bb1d1e1ec9b3966069dcf07c781cb7129f54bcbe78dce13c606cf88d6` |
| `src/limit_model.rs` | `51754f661e021856a6600537ea02c721edcd15b297ccefd8f95bd99aded93576` |
| `src/slide_model.rs` | `cdb15646c135252a3bc79a3d15f0ec7617242581d0119b117a922e33af50f37f` |
| `script.tsv` | `a4f38c38e2e72963312b238d7c3241ac8b78ea817abb5d8eb09fb83b9bf7cfea` |
| `oracle.py` | `91c8c0d38f46c16cd0f1add77de7ae7bbe698b87f2926b7854e292df1a57c89b` |

Also read: `cc1-exponential-review674.md`, `expanded-stage05/review-fixes690.md`, `expanded-stage06/handoff696.md`, `exponential-build697.py` (Clippy step only), kit `kit.rs`, `play.rs` and `facts.rs`, and engine sources at pin a399d62 for the new calls.

## Result

- The mathematics holds. I found no error in the 16-cell against 1024-cell separation, the bound, the slide-rule alignments or the spoken claims.
- All nine 674 findings are repaired in source or superseded (table below). The ruler repair is a real change of geometry, and every reading I computed lands on the mark the voice names.
- All 21 ruler and 05_6 sentence hooks point at the sentence that says the thing, by my reading of `script.tsv`.
- One concrete expected build stop: a new constant-only `assert!` (F1).
- One semantic gap in the 16 against 1024 picture: the two distance rows carry no cell count (F2).
- Pedagogy Lows only otherwise (F3, F4).

Root's move: fix F1 before the guarded build, and take F2 in the same one-stage change so build697 re-pins only once.

## Findings, most important first

### F1. Medium-High, expected build stop: constant-only assert in the new test (`src/limit_model.rs:326`)

`assert!(DISPLAY_CELLS < BOUND_CELLS);` has two `const` operands, so its value is fixed at compile time. Build697 runs Clippy with `--all-targets --no-deps -- -D warnings` (`exponential-build697.py:609`). Clippy's `assertions_on_constants` should reject this. I did not run Clippy. The same prediction is recorded as R1 in `logarithm-repair08-10-independent-source-review01/REVIEW.md`, and this repository's convention is `const { assert!(...) };` (for example `explainer_corpus/logarithm_mysteries/src/chapters/undoing_growth.rs:1068`). A scan of every `assert!` in the stage06 crate whose argument is only capitals, digits and operators finds this line alone. The new test's other three assertions are not constant.

Exact fix, one line: `const { assert!(DISPLAY_CELLS < BOUND_CELLS) };`

Left alone, the build stops at Clippy after compiling and running the tests, and the fix then needs a new stage and a re-pin of 697. The handoff 698 already expects a possible Clippy stop; this one is avoidable now.

### F2. Medium: the 05_6 distance rows have no cell count, beside a chart labelled 16 cells (`tiny_multipliers.rs:1670-1685`)

Stage06 names the drawing ("drawn with 16 cells, so the gaps show") and the bound ("numerical bound, 1024 cells: total ≤ 0.00514"). The two rows under the bound read `x=1.5: distance 0.00492` and `x=-1.5: distance 0.00025`, with no count. These are 1024-cell values, computed with `last = BOUND_CELLS`.

| Cells | Distance at x = 1.5 | Distance at x = −1.5 | Gaps in columns 0 to 9, summed |
|---|---|---|---|
| 16 (the chart) | 0.28710 | 0.01613 | 0.28709 |
| 1024 (the card) | 0.00492 | 0.00025 | 0.00492 |

Those values are exact rational arithmetic from my mirror. Nothing on the card is false. A viewer who reads the rows as describing the bars in front of them is off by about 58 and 65 times, and the bars' visible gaps add to far more than the printed bound. The voice then says "and it holds here too", right after the redraw, where "here" can be heard as the drawing.

Exact fix, TeX only, both rows: `x={RADIUS}:\ \text{{distance at {last} cells}}\ {:.5}` (and the same for `-RADIUS`). Estimated width at size 25 is about 4.2 to 4.4, so the row spans about 1.6 to 5.9 around `CARD_X`, within the header's own span (1.51 to 5.99). The source-order test strings do not change.

Optional voice change, since no takes exist: "... and the same cut, and it holds at minus three halves too."

### F3. Low-Medium, pedagogy: 07_3 never shows that the second distance is the distance for three (`calculating_ruler.rs:279-287`)

The voice says "the distance for two, followed by the distance for three, is the distance for six". The second bar is drawn directly from the 2 mark to the 6 mark. The arithmetic is right (its length is `distance(3)`), but the picture never connects that piece to the distance from 1 to 3, which is how 07_2 defined it. This is unchanged since stage03 and was not in 674. It matters because it is the step the slide rule then performs by hand.

Suggested fix: draw the bar from `position(1.0)` to `position(b)` on `SECOND_BAR_Y`, then `slide` it by `v(m::distance(a), BAR_Y - SECOND_BAR_Y)` on "followed by". This needs `crate::phrase::until_phrase`, or a sentence-level split if root prefers no new hook.

### F4. Lows

| Where | Issue | Fix |
|---|---|---|
| 07_4 sentence 1, `calculating_ruler.rs:301` | "Each end mark of a scale is called an index" has no hook; the first index name appears after the sentence 2 shift. Name without picture. | On sentence 1, briefly name or flash the upper 1 and 10 |
| 07_2 sentence 2, `:256` | "One belongs at the beginning" has no hook. Height 1 has no distance segment and lands with the others on sentence 1, so nothing marks it as the voice speaks. | Flash the 1 label or its dot on sentence 2 |
| 07_7 sentence 3, `:363-367` | Lower 1.2 is an unlabelled minor tick, and 674's item 8 reasoning applies equally. The caption does say "Read 1.2". | One `pointer(&k, s, value)` call, hidden with `joined` |
| 07_2 sentence 1, `:236-251` | The rail is at y −.30, the middle of the graph, so heights 1 to 5 rise and 6 to 10 fall onto a line not drawn until after they land. The comment says "on the way down". | Picture check; if it reads oddly, draw the rail first |

## 674 findings against stage06

| 674 | Judgement | Basis |
|---|---|---|
| 1 High, cursor hides reading | Repaired | Rails ±.30, ticks facing the joint, labels ±.57. Cursors end at ±.36 (±.375 with the round cap), label ink starts at ±.499, for every reading. Alignment line at every alignment, named for the index ones. |
| 2 Medium-High, `y'=xy` | Repaired | `\frac{dy}{dt}=x\,y\quad(x\ \text{fixed})`, picture only. Time is on screen as "one unit of time", and `t` itself is not spoken. |
| 3 Medium-High, no horizontal distance | Repaired | Blue segment from the height axis to the curve, on 07_1 sentences 3 and 4 and then 07_2 sentence 0. Each drops by `LOWER_Y - y` at unchanged x, and `point` shares `position`'s mapping, so its end is the mark. |
| 4 Medium, labels on curve | Repaired | Offset (−.22, +.155). Label ink is about .064 from its own segment and .065 from the next. |
| 5 Medium, gaps do not shrink | Repaired | `LABELS` 1 to 10: gaps 1.656 down to .252, strictly decreasing. |
| 6 Medium, vocabulary | Repaired | Index defined in 07_4; "upper", "lower", "restore the decimal place" on screen. See F4 for show-then-name. |
| 7 Medium-Low, timing | Repaired | Reset before 07_5, 07_9 shift on sentence 1, "Read 1.2" before "12". |
| 8 Low-Medium, 2.5 unmarked | Repaired | Pointer on 07_5 sentence 1, removed at 07_6. No answer leaks. |
| 9 Lows | Repaired or superseded | The 05_6 row is superseded by 696. A-C header comments deliberately left. The oracle limitation still stands. |

690's correction to 674 is right: `Vec3` is `[f64; 3]` at the pin (`fmn-core/src/types.rs:5,13`), so `p + v(...)` could not compile. Stage05 writes the coordinates out.

## Checked, no finding

- **16 against 1024:** the bars morph to 16 before the sweep, so "the same bars" compares 16 cells with 16 cells at ±1.5. The live bars use the same `shown_n`, and `chain` does not depend on the sign. 16 is in `BAR_COUNTS`, and `BOUND_CELLS` is the last of `HEAD_COUNTS`.
- **The bound:** the distance is at most the sum of gaps, since all terms are bounded by magnitude. The gaps and the tail use `|x|`, so the same total serves −1.5. True head plus true tail at 1024 is 0.00504; the printed 0.00514 is larger because the head and the tail bound are each rounded up. Both distances sit under it.
- **Ruler readings, computed:** upper 3 over lower 6 (x 1.2798); upper 4 over lower 10 (x 2.5); upper 3 at x 2.9355 past the rail end at 2.5; upper 10 over lower 4 with upper 3 over 1.2 (x −2.5645, strip left −5.189); upper 1 over lower 2 for 6 ÷ 3 (x −1.3443). `m::distance` is in scene units, so the 07_9 offset is correct.
- **Copy step:** `sources` and `moving` are both 92 strokes plus 10 labels in the same order. `transform_from_copy` sets the replace flag (`fmn-anim/src/transform.rs:630-638`), so the `moving` mobs are in the scene when `shift` runs.
- **Calls read at the pin (not compiled):** `grow_from_edge` signature; `Timed` covers any `Animation` (`play.rs:12`); `AnimateBuilder::move_to` matches the existing `glide`; `Facts::add` takes `impl Display`; `LEFT` comes through the prelude.
- **05_6 code:** imports all used; the `assert_in_order` strings occur in order; `new_count` replaces `panel.count` in the returned screen. The new label starts about .29 to the right of the live reading.
- **Oracle:** I reran `oracle.py`; its output is byte-equal to `oracle06.tsv` (`8b0f35cfe6a32fad85c67357fed190a004c6ed247358e7e031fa02daf3fa5da0`). The four new rows match my exact values. As 674 noted, the F rows check drift, not the geometry choice.

## Exact steps for root

1. Have the owner of the next stage apply F1 (one line) and, if accepted, F2 (two TeX strings, optional voice words), in a new stage with stage06 preserved.
2. Re-pin build697 to that stage's digest before the guarded launch.
3. Decide F3 and F4 now or at picture review. F3 and the first F4 row change no facts.

If root prefers to launch on stage06 unchanged, expect the Clippy stop at F1 and treat F2 as a picture-review item.

## Not established here

- That any of this Rust compiles, that the 95 tests pass, or what Clippy reports. F1 is a prediction.
- Native facts equal to the oracle, the glyph probe (including `\frac{dy}{dt}`, `\text{fixed}` and new strings), or the layout lint.
- Whether the 16-cell gaps read at 12 to 18 px, whether the eight 07_2 reveals (2.8 s) fit sentence 0, or how the 102-mark copy and the rail landing look. `until_sentence` is a no-op when late, so an overrun delays the next step and does not fail.
- Any Frederick take, narration timing, listening, fine sync, picture, motion, independent AV or release result. Chapters outside this crate were not reviewed.
