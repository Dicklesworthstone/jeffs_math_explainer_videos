# Complex 11: acted normalization and counting repair

WhiteSwan, for BlueSalmon, answering Mail 2648. Staged only. Nothing here has been built with Cargo, rendered, synthesized or heard. The frozen `complex11-counting-review-repair01/` is untouched, and `checks/baseline.sha256` pins its handoff, seal and three patches.

## Status

- **The gap from my last review is closed in source.** "Divide by its leading coefficient" now has an acted picture, not just a formula.
- **The compression card** now says `output size compressed; direction and turns around zero unchanged`.
- **The reach label and the `a` repair are carried over unchanged.** The storyboard patch is new and replaces the frozen 02.
- **A concrete masking defect is fixed:** the distance line and the disk's rim both cross the stationary `0` in 11_8.
- **No script word moves.** Root's `script.tsv` is the script, and 11_7 is still the only cue needing a new take.

**This is not independent.** I designed and wrote the demonstration, the reach repair, the `a` patch and the glyph patch, and I wrote every check here. Nobody else has read this code.

## 1. What to apply

Apply with `patch -p1` from the repo root.

| Item | Applies to | Gives | SHA-256 of the item |
|---|---|---|---|
| `patches/counting-from-root.patch` | root's `counting.rs` `7c5a41c6fd37ad5d…` | staged `counting.rs` `dba35730d15b3c5e4ff7fb0fe7444265424b920d87861562b11d8468f9d290e4` | `40cbb8f9d5d0ae6110ecb916b13442935e2ea91254bc1c3469583894de336296` |
| `patches/mod-registry.patch` | canonical `mod.rs` `57c46d67161490cb…` | `mod.rs` `2414a6bd8c1ae26af60385d6e5ee80bfc70a1d7dae1ae6aeccdcc97c75b60549` | `3918dbafa2efca8b5d200e4db52207d282c5471d5df67d20197fa38f7cd98df6` |
| `patches/storyboard.patch` | canonical `storyboard.md` `be30f73484bd69cb…` | `storyboard.md` `6deb9f01cbcc505a2a08e5d8bc673bfe3f035c499824091058e86e903acee1c4` | `089ca7d9ba443fe23d2a254111e9ec652a341c4fe9f5419ed125cd924cb6f49e` |
| `holds/narration-direction-map-oliver11.json`: copy into the crate's `docs/` | new file | candidate map | `759e0eba1d40fbd2ff4f715719493dccc35eed27da0f730c6e8c8344a7f1bb91` |
| `patches/narration-units.patch` | canonical `narration_units.json` `3c4f53d797385729…` | plan `3989e1855f617734c38caae8d0113d4915b5c44fbde600f674e5779ed4977f28` | `34199ed235b12d8a243a47c22550f76af63963418c668f46286478ff62df2c23` |

Two more patches are for reading only:

- `patches/counting-delta-on-01-03.patch` is the same result taken from the frozen 01 + 03 file `20ad375dcce1c117…`. Sections 3 and 4 describe this delta. Its hash is `6cda7825bea2b108018120fe4924aa4dd944cab4e9ebc3d70f4f19fb04d4c7f0`.
- `patches/direction-map-from-oliver10.patch` shows the map's delta. Its hash is `3eff9945f0dd279de1b8720fec6579824c4024ed534b55b64f2afea504c90601`.

How this maps to the frozen handoff:

| Frozen patch | Status here |
|---|---|
| 01 reach label | Included. The round trip through 01 + 03 + delta gives the same bytes. |
| 02 storyboard | **Replaced by `storyboard.patch`. Do not apply both.** It has the same root words, plus the 11_6 card text, the full 11_7 picture and the device column. |
| 03 `a` name | Included. |

**If the glyph patch goes in too**, either order gives `counting.rs` `138b5d84d0c9240dbd3a66f786481132d59ab3bc7f22e2b11c1e0adc9195e770` and `mod.rs` `eb10b3157919cbc0dc215e0eaf565bd0546094667b742060a61f0cee1238d363`. It moves no hold or helper line. Only the map's `sha256` fields would change, and then the plan's map hash after them.

## 2. What the viewer sees in 11_7

The example is `f(z) = 2i(z⁵ − 3z + 1)`, the film's own quintic times a nonzero number.

| Voice | Picture |
|---|---|
| "The same trick proves that every nonconstant…" | The loop, the five white root marks, the colour field and the output curve stay from 11_6. The claim appears over the field. |
| "…**complex polynomial** has roots." | The row `f(z) = 2iz⁵ − 6iz + 2i` appears, with its leading `2i` gold. Then the whole picture turns: every output times 2i, so the colours go a quarter of the wheel, the bands of equal size slide, and the curve turns and grows about a fixed 0. **The marks do not move.** |
| "**Divide** by its leading coefficient," | `every output ÷ 2i  (2i ≠ 0)` appears over the output plane, and the picture turns back. |
| "**which leaves** its roots unchanged." | It lands on exactly 11_6's colours and curve. The row becomes `p(z) = f(z) ÷ 2i = z⁵ − 3z + 1`. `0 ÷ 2i = 0, and nothing else lands on 0` appears over the output plane, and the card `same five roots` appears under the lowest mark. |
| "**Now it starts** with z to the n…" | The row gives way to `p(z) = zⁿ + ⋯`. |
| "**On a huge circle**…" | All of it clears for the wide plane, as before. |

Why this is the demonstration:

- **The argument is visible.** One tracker drives both the field and the curve, so the viewer sees the whole function change while its zeros stay put. Every point where all the colours meet stays under a mark at every step.
- **Both directions are stated.** `0 ÷ 2i = 0` keeps the old roots, and `nothing else lands on 0` rules out new ones.
- **`2i ≠ 0` is on screen**, and it is why the division is allowed.
- **Nothing covers the argument.** The rows sit over the field and the output plane. The card is the only item on the field, below the lowest mark and clear of all five.

## 3. Timing delta

There are 112 timed calls in root's file and 123 here. All 11 new calls fall between 11_7's `say` and its wait for sentence 3, so nothing after that anchor moves because of them. No timed call of root's is gone or changed, and there is no new cue or fixed wait.

```
line.wait_for("complex polynomial")     fade_in 0.3     line.sentence(1)
tween turned -> 1.0 (0.7 to 1.4 s)      line.wait_sentence(1)     fade_in 0.3
line.until("which leaves", 1.2)         tween turned -> 0.0       fade_out 0.2
fade_in 0.3 (results)                   fade_out 0.3 (the p(z) row, after sentence 2)
```

**Decision for root:** Mail 2648 asks for sentence 1/2 keys. Sentence 1 and sentence 2 are keyed as asked. I also used two phrases inside existing sentences, because the first turn has to happen before "Divide" for the division to undo anything. These phrase times are not invented, since they come from the script's words and the measured sentence. But without phrase marks they are **character-share estimates**, not word times. If you want sentence keys only, say so and I'll stage that variant.

Budget on estimated sentence times, not Oliver's:

| Rate | Sentence starts (s) | Turn out / back (s) | Each act's end against its sentence (s; 0 or less means on time) |
|---|---|---|---|
| Old scratch rate | 0.0, 5.9, 11.4, 16.1 | 1.40 / 2.47 | −0.67, 0.00, −2.19, −4.04 |
| 15% faster | 0.0, 5.0, 9.7, 13.7 | 1.40 / 2.05 | −0.32, 0.00, −1.78, −3.33 |
| 30% faster | 0.0, 4.2, 8.0, 11.3 | 1.36 / 1.64 | 0.00, 0.00, −1.38, −2.62 |

The card still fits its window: 65 characters against 7.2 s clean, with a floor of 4.33 s and a ceiling of 8.67 s. This mirrors the crate's reading-window rule, but that crate test has not been run.

## 4. Registry delta, holds and masking

**Registry (`mod-registry.patch`, 5 rows added, none removed):**

- `TEXT` gains `counting::EVERY_OUTPUT`, `counting::ZERO_KEPT` and `counting::SAME_ROOTS`.
- `TEX` gains `counting::BEFORE_DIVIDING_TEX` and `counting::AFTER_DIVIDING_TEX`.
- `LEADING_KEY` is only a colour key, so it has no row. It is written `2{i}` in the last term so the literal match colours only the leading `2i`.

**Holds:** relocated by their markers, each bound to the staged `counting.rs` hash.

| Cue | Line before | Line now | Marker |
|---|---|---|---|
| 11_5 → 11_6 | 429 | 440 | `let asked = predict(…)` |
| 11_6 → 11_7 | 492 | 503 | `reading_window(…)` |
| 11_8 → 12_1 | 547 | 604 | `reading_window(…)` |

- These pass root's `validate_oliver_plan` rules as re-implemented in my check, and the repo's real `direction_contract` with its map path pointed here. A control confirms it refuses a wrong script hash.
- The `mod.rs` helper rows take the new hash, with lines 31 and 58 unchanged. The `draw.rs` rows and the other 20 holds are unchanged, and `hold_kinds` and all 23 overrides are the same.
- The map name `oliver11` is only a suggestion. If you rename it, the plan's map path changes with it.
- `tools/narration_director.py validate_map` expects `evidence` and `reason` fields that this map's rows don't have. The canonical map already lacks them, so I left that alone.

**The stationary `0` masking is real.** Compared with preview02's frame 535, the digit's box (180 pixels) changes on these frames:

| Frame | 546 | 547 | 548 | 549 | 544, 550, 551 |
|---|---|---|---|---|---|
| Pixels changed | 63 | 29 | 26 | 3 | 0 |

The cause is that the distance line and then the disk's rim pass over it. The fix puts a ground-colour plate (0.105 scene units, 9.45 px) under the name, and promotes both above the line and the disk just before the sweep. On the desk composites the box changes by 0 pixels. **The cost is a short break in the line and the rim as they pass.** `checks/desk-composites/zero-plate-frames-546-548.png` shows before and after.

## 5. Evidence

| Check | Result | What it does not prove |
|---|---|---|
| `checks/source_checks.py` | 115 PASS, 0 FAIL: baselines, patch round trips and reversals, timeline and string deltas, the real `check_video` built by rustc on the patched storyboard and root's script, holds, widths, glyphs, both orders with the glyph patch | Pictures or voice |
| `checks/standalone_tests.py` | 13 staged tests pass with rustc, using the real `model.rs` and root's script. 57 planted defects were each caught, and the old script fails. | Not `cargo test`. A shim stands in for parts of the crate, and most crate tests and all `mod.rs` tests did not run |
| `checks/typecheck_stub.py` | `Turning`, 11_7 up to the wide plane, and the plate lines type-check against hand-copied stubs. 6 controls were caught. | Not `cargo check`. A wrong stub would hide a real error |
| `checks/desk_composites.py` | Control: the desk field at t = 0 is within 24 levels of frame 356 on 95.1% of pixels. I looked at composites a, b, c, d and the plate sheet. | Not renders: words are hand-set, and timing is unknown |
| `checks/anchor_probe.py` | Its controls pass (section 6) | A tool for your frames. It has seen no frame of this code |
| rustfmt | Clean on the staged file | Clippy did not run |

**The mathematics in the tests:**

- The example equals the quintic times its coefficient.
- Every step of the turn keeps the root count and the winding, and the curve never reaches 0.
- The tracker runs from 1 to the coefficient without reaching 0.
- The curve at t = 0 is the walk's own curve.

**Not edited:** `facts.rs` and the oracle. `2i` is an illustrative constant pinned by `the_example_is_the_quintic_times_its_leading_coefficient`, the same pattern as `LEASH_TEX`. If you want it as a fact row, that is your call.

## 6. What root needs to do

1. Apply section 1, then run the guarded build, tests and Clippy. The new tests are in `counting.rs`, and the crate's reading-window and registry tests have not yet seen these strings.
2. Read the glyph probe for the new strings. **`≠` in `kit.text` and `\div` in this crate's TeX need eyes.** The desk font lacks `≠` and `−`, so I drew them as strokes. Both glyphs are in use elsewhere (`≠` in `one_point::KEPT`, `\div` in `eigen_engineering`), but neither is proven in these rows.
3. Render chapter 11, then run the probe with frames picked under Oliver:
   ```
   python3 -I -B <folder>/checks/anchor_probe.py <frames-dir> REF=<n> TURNED=<n> LANDED=<n> BEFORE=<n> SWEEP=<n>,… ZERO_SWEEP=<n>,…
   ```
   Its docstring defines each moment. On preview02's real pixels the marks pass and the turn, the `0` and the label fail, and the field and curve metrics separate the film's t = 0 from the model's t = 1. Four planted defects also fail.
4. Check these at preview:
   - **The takeover seam:** `painter.leave` fades under the new strips, and the two pictures should match. The model's field was within 24 levels of frame 356 on 96.2% of the probe's cells, so this is likely but unseen.
   - **Repaint cost:** 25,600 cells, up to about 96 repaints (an estimate). This is unmeasured.
   - **Steps in the turn:** 48 steps each way, not yet seen in motion.
   - **Mid-turn look:** composite b shows a brief phase seam along the real axis. It is correct model behaviour, but it may need your eye.
   - **Phrase timing:** see the decision in section 3.
5. Synthesize 11_7 from root's wording through the guarded ElevenLabs Oliver lane. No other cue's text changes.

## 7. Gaps

- **Uncompiled:** no Cargo build, test or Clippy, only the rustc stand-ins above.
- **No render, listening, fine sync or final AV review.** I have not heard "nonconstant" or any 11_7 take, and 11_7's takes are root's. Every timing here is an estimate.
- **Not independent:** see Status. A fresh reader would be most useful on whether the turn teaches the point, and on the `nothing else lands on 0` wording.
- **`tools/storyboard.py --check`** already fails on canonical, so I did not use it as a gate. I built the real `check_video` instead.
- **Cross-lane:** BronzeJaguar's Mail 2818 proposes `output turns count` for the closing recap. It doesn't conflict with this wording, and that mail asked for replies only on conflict, so I sent none.

`SHA256SUMS` seals this folder.
