# Review 713: exponential applications-stage04, chapters 08 and 09 (CC0)

cc0, 2026-11-10, 00:11 UTC. Independent source review of CC2's sealed stage. **Nothing was compiled, rendered, synthesized or run through Cargo.** No audio heard or checked. No picture, motion, sync or AV acceptance is claimed. No file in `applications-stage04` or `expanded-stage08` was touched.

Subject hashes recomputed and equal to `native-handoff693.md` and `source-check01.json`: `decay_scene.rs` `f70bb723…`, `circuit_scene.rs` `cf014a0f…`, `script.tsv` `bb63ad2c…`, `applications_model.rs` `a8656956…`.

## Verdict

The mathematics, the derivations and the cue hooks hold. The teaching order is sound. I found **no compile error** against the a399d62 engine and the kit.

Fix these before CC2 integrates: one wrong on-screen state (F1), one spoiled prediction (F2) and two strict Clippy stops (C1, C2). F3 to F6 are small repairs worth making in the same pass. Two script wordings need a decision while no takes exist.

## Must fix

| # | Where | Problem | Fix |
|---|---|---|---|
| **F1** | `circuit_scene.rs:1168-1180`, `1214`, call `1388` | **09_8 shows the wrong capacitor.** 09_7 changes the label to "200 microfarads" in teal and restores only the resistance. `practical_settling` does not receive `values`, so the label stays for all of 09_8. Beside it the screen says "this instrument settles at 5.48 s" and "the model's gap at 6 s is still 0.030 V". Those are 100 µF numbers. At 200 µF, RC is 2 s, settling is 10.96 s and the 6 s gap is 0.60 V. | Pass `values` in. At the start of 09_8, transform `values.capacitance` back to `farads_label(m::DEMO)` in ink at the same position, as line 1179 does for the resistance. |
| **F2** | `decay_scene.rs:705-714`, `722-724` | **The 08_4 answer is printed on the axis.** The y labels 100, 50, 25 and 12.5 fade in during 08_4's first sentence, about 17 s before "50 → ?" and the hold. The only value directly below the question's 50 is 25. The README requires the endpoint concealed through the hold, and `the_endpoint_stays_concealed_through_the_authored_hold` checks only the trace. | Keep building all five labels, because 08_6 indexes `y_labels[..4]`. In 08_4 show only `y_labels[0]`, `[1]` and `[4]`. Fade in `[2]` (25) with `second_label` at line 818 and `[3]` (12.5) with `third_step` at line 854. The blue curve can stay: reading it is a legitimate route to the prediction. |
| **C1** | `circuit_scene.rs:1498` | `assert!(BATTERY_LONG > BATTERY_SHORT);` compares two constants. This project's strict Clippy has stopped on `assertions_on_constants` for this form. | `const _: () = assert!(BATTERY_LONG > BATTERY_SHORT);` |
| **C2** | `circuit_scene.rs:369` | `i % 2 == 0` with `i: usize`. Strict Clippy stopped on `manual_is_multiple_of` in the parrondo build for this form. | `i.is_multiple_of(2)` |

C1 and C2 are inferred from earlier stops in this project and from the lint's rule. Clippy was not run.

## Should fix

| # | Where | Problem | Fix |
|---|---|---|---|
| **F3** | `decay_scene.rs:1009-1015` | "One old half-life now leaves 0.25" contradicts the axis it sits on. The axis has read "fraction remaining" since 08_6. The red curve ends at 0.354 × 0.25 = 0.088, not 0.25. The 0.25 is a share of what was present at the warming. | Say what it is, in 08_5's multiplying language: "One old half-life now multiplies by 0.25". Centre it near x = -1.1 so the longer line clears the dashed line at x = -3.53. |
| **F4** | `decay_scene.rs:1118-1125` | 08_8 says the bar "represents an amount", and the readout says 100.0, but the axis still reads "fraction remaining" with 1, 1/2, 1/4, 1/8. The teal step trace is also in amount units. | Pass `marks` in and morph `f.y_name` and `y_labels[..4]` back to the amount labels at the start of 08_8. The handles stay valid after the 08_6 morph. |
| **F5** | `circuit_scene.rs:1063`, `1304-1307` | `b.clock` is rewound to 0 in 09_6 and never moves again. Through 09_8 the readouts stay at 0.00 V, 12.00 V and 1.20 mA, and the column stays empty, while the voice says the voltage "approaches the battery voltage" and the inset shows 4 to 6 s. A second clock also breaks the file's stated one-clock rule. | Drop `zoom_clock`. After the assumptions clear, `rewind(s, b.clock, ZOOM_START_S, 0.6)`, then drive `zoom_curve` and `zoom_spot` from `b.clock` with the existing 3.2 s run. The readouts then end at 11.97 V and 0.03 V, agreeing with the "0.030 V" text. At 2.45 words/s this still lands before 09_8's third sentence. |
| **F6** | `circuit_scene.rs:792-799`, `801`, `810-817`, `854` | In 09_3's last sentence, `V(t)=V_B-g(t)` (about 2.10 wide, centred at -4.7, right edge -3.65) overlaps "the decay law, for a new quantity" (about 3.25 wide, centred at -2.4, left edge -4.03) on the same row. The overlap is about 0.38 units, by CM widths. | Add `same_law` to the clear at line 801 and remove it from `formulas` at line 854. |

## Script wording (no takes exist yet)

- **W1, 08_5 last sentence: change it.** "the half-life of this constant-rate model" collides with 08_2, where the gold rule is literally the constant-rate rule, and that rule has no half-life. The screen already says "constant k". Proposed: "That repeating interval is the half-life of this model, as long as k stays constant." It also sets up the temperature change in 08_7.
- **W2, 09_7 third sentence: accept CC2's "Rescale".** "Stretch the time axis" reads the wrong way, since the trace is squeezed left and the card says "divide its times by 2".

Neither changes a sentence count, so no hook moves. `script.tsv` and the bindings that carry its hash would need re-pinning.

## Low

- **08_5 reveal:** the voice says twenty-five by about 1.6 s, but the trace reaches its endpoint at 1.95 s and the label at 2.25 s (`decay_scene.rs:804-818`). Shorten the 1.6 s run to about 1.0 s.
- **End of 08_2:** at `t = TIME_MAX` the blue slope arrow runs past the axis and its tip lands inside the blue bar, at about (1.43, -2.02) with the bar starting at x = 1.14 (`decay_scene.rs:276-279`). Use `BLUE_RUN.min(TIME_MAX - t)`.
- **09_6 hold:** the current readout stays 1.20 mA beside the new "20,000 ohms" for about 10 s. This keeps the answer hidden, but the pairing is false. Accept it or fade that readout; the live value would need a stored handle.
- **Title:** "A falling curve does not name its cause" stays from 08_1 through 08_5, including the rate law and the half-life.
- **08_7 right panel:** the horizontal axis has no name.

## Checked and sound

- **Mathematics and units:** R·C = 1 s; 12/e gives 7.59 V, 36.8% and 63.2%; ln(240) = 5.48 s; 12e⁻⁶ = 0.030 V; V(4 s) = 11.78 V, inside the inset; i(0) = 1.20 and 0.60 mA; doubling R halves i(0) and doubles τ, while doubling C keeps i(0); g' = -V' = -g/RC; A = A₀/(1+kA₀t) with (1/A)' = k; second-order fractions 0.50 and 0.67; the rescaled later segment lands on the curve. The opening curve really is the 08_7 curve (`k A0 = 1`), so "the opening curve had this law" is true. Recomputed: 1 of 20 molecules left at 3.6.
- **Circuit:** the long plate is on top, so conventional current runs right to left on the bottom wire, as drawn.
- **Hooks:** all 67 are in range (35 + 32, equal to the handoff). I read each against its sentence or phrase and found none on the wrong words. The four phrases exist, and the kit's sentence split agrees with `phrase.rs`.
- **Timing, by a scratch estimate at 2.45 words/s:** no cue overruns its speech apart from the authored 4 s breaths and about 0.35 s in 09_1 and 09_5 after their holds. Late hooks: 08_5 second sentence 0.62 s (above), 08_8 third sentence 0.39 s, 08_2 third sentence 0.09 s.
- **Predictions, apart from F2:** no teal trace before 08_5, and the sample stops at 0.45 of an interval (36.6) and is rewound. No slower trace and no change to `b.factor` before 09_7. Both cues run `hold`, `mark`, then a 4 s breath, matching stage06.
- **API, read against a399d62 and the kit:** `prepare` returns `fmn::Result`, so `tint` type-checks; `DashError` converts; `Polygon` and `Circle` `build` are infallible as used, while `Line` and `Arrow` return the `Result` the code expects; `Some(linear)` coerces to the field's `fn` type; `Stage` derefs to the arena methods; `Mob` is `Copy`; `add_to_scene` moves an object to the top, so the re-adds work as z-order; `Arrow` scales its tip at short lengths. `Vec3` is only combined through `add`. No object is faded out twice.
- **Tests:** I read all 22 by hand; the literals agree with my arithmetic. None were run.

## Limits

Widths are CM estimates, and the layout lint on a draft remains the real check. The glyph worries in the handoff are unresolved and belong to the probe. The timing is an estimate, not narration.

**Side note for the running build, outside this review:** `expanded-stage06/native-crate/src/limit_model.rs:326` has the same form as C1, `assert!(DISPLAY_CELLS < BOUND_CELLS);` on two constants, so build697's Clippy step may stop there. The line is also in stage07 and stage08. I did not find a passing Clippy run that covers it.
