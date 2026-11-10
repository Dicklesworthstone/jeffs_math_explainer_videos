# Exponential ending, chapters 10 to 15: independent source review 721

CC2, 2026-11-09. Read-only review of `ending-stage07` for root and CC1 (integration 720). `E` is `narration_lab/third-wave-20261024/root/elevenlabs-exponential-20261101`; files below are in `E/ending-stage07/native-crate`.

**Scope: source review only.** I read `handoff706.md`, all 27 new script rows, both models and the five new chapter files plus `ending_draw.rs`. Nothing was compiled, run, rendered, listened to or synthesized, so this is not runtime, picture, motion, listening or audiovisual evidence. Layout comes from source constants, with text widths estimated at Computer Modern metrics. Timings use a scratch 2.45 words per second. The handoff's external sources were not reopened. `check706b.py` was not run because it can write (line 149); every number below was recomputed separately.

| Reviewed bytes | sha256 |
|---|---|
| `native-crate` tree, 51 files (matches handoff) | `c1a92bf7f3b91ac90a1d1778cdfa591353a5c805316ced9b056e1e3f9291bd36` |
| `script.tsv` (matches handoff) | `a97a39d06d90f5f1e4e4227194f98207cacea59ee9a78e061af98ad79b000130` |
| `oracle07.tsv` (matches handoff) | `5dc90ca0bc20e5877842e0f406cf8549a42462802ac1a2159de2d6a541bfdbf9` |
| `handoff706.md` | `9a3e839a7e2b87fa323a3030033094e27d8f8ff1146609b7bf2523b2e7f3a75e` |

## Verdict

Every spoken and displayed number I checked is right, and the arithmetic of the finance chapter is correct. The derivation of the circle in 13_2 to 13_4 is sound and properly earned, and 14_4 handles the notation honestly.

Three findings need repair before any Frederick Surrey take is requested. H1 is a false premise that carries the whole move to complex inputs. H2 is a prediction whose on-screen inputs give the wrong answer. H3 contradicts the film's own definition of its ruler. Three picture defects (M7 to M9) are certain from the source. None of this blocks integration 720, but wording repairs should land before synthesis, and all repairs below add explanation without cutting depth.

Inserting sentences shifts `until_sentence` indexes in the same cue, so rerun the hook scan after H1, H2, M1, M4, M5 and M6. M1 and M6 add spoken numbers that need facts and spoken-number checks.

## High

| ID | Where | Defect and evidence | Repair |
|---|---|---|---|
| **H1** | `script.tsv:65` (13_1: "But our machinery only ever added and multiplied. So it runs on any numbers that can do those two things"); cards `turning_rate.rs:124`, `:127` | False as stated, and the film contradicts it. The multiplication law rests on 03_5's "one analytic permission slip", absolute convergence (`script.tsv:16`). 14_2 says the limit proof "only ever compared the sizes of terms" (`:71`). The machinery also divides by factorials and needs products in either order. Counterexamples: arithmetic mod 5 adds and multiplies, yet 5! is 0 there, so 1/5! has no meaning. For A = [[0,1],[0,0]] and B = [[0,0],[1,0]], exp(A)exp(B) = [[2,1],[1,1]] while exp(A+B) = [[1.5431,1.1752],[1.1752,1.5431]]. This sentence is the film's only extension of the law to complex inputs, and 14_4 ("still holds", `:73`) and 14_5 (`:74`) stand on it. | Replace both sentences: "But look at what our machinery really asked for. It added and multiplied, in either order. It divided by whole numbers. And it measured sizes, because regrouping the grid needed the sizes of the terms to have a finite sum. Complex numbers can do all of that." Cards: the four requirements, then "complex numbers do all of it". |
| **H2** | `changing_rates.rs:272`, `:276-283`, `:286`, `:289`; `script.tsv:53`, `:54`, `:56`; `handoff706.md:73` | At the 10_2 hold the screen shows x1.50, x1.25, x1.00, x0.75, the product formula, and a path already at 1.875 with x0.75 still to come. Following the film's instruction gives 1.40625, "above". Stopping the path one cell short does not hide the ending, so the handoff's claim fails. The next words are "Exactly where it began", and 10_5 reports "near one point four" with no reason. The reason: each cell reads the rate at its left edge, where this falling rate is highest, so four cells sum r w to 1/2, not 0. Read at the midpoints, the same four cells give 0.84595, below the start. | Keep the numbers and make the conflict the hook. Draw the last step to 1.40625, then ask: "Four cells finish near one point four. But these cells are crude. As they shrink, does the amount finish above its start, below its start, or exactly where it began?" In 10_5, after the four-cell sentence, add: "Every cell read its rate at its left edge, where this falling rate is highest." Update the test at `changing_rates.rs:488-492` and the handoff row. |
| **H3** | `script.tsv:77` (15_2); `log_bridge.rs:48-50`; `money_limits.rs:564`, `:573`, `:586`; against `script.tsv:44` | 07_3 defines the ruler: "The position of a number is then its base ten logarithm." 15_2 says the backward machine is the natural logarithm and "On the ruler, a number's position is its logarithm", and the screen says "value at position x = E(x)" and "position of x = ln x". Those hold only in a unit the film's ruler does not use. 12_2 has the same mismatch: "distance on the ruler" is labelled +0.405 (ln 1.5), while the film's ruler gives log10 1.5 = 0.17609 of a scale, 0.9685 scene units at `SPAN` 5.5. | 15_2: "Measure the ruler in the exponential's own unit, and a number's position is its natural logarithm. Our slide rule used the same marks with a longer unit." Say the same unit on the row and on the 12_2 name, for example "distance in E's own input". |

## Medium

| ID | Where | Defect and evidence | Repair |
|---|---|---|---|
| **M1** | `script.tsv:61` (12_2, "A typical run has about fifty of each, so it ends near"); `money_limits.rs:609-618` | The handoff means the median run, but "ends near" is not true of typical runs. Each extra head multiplies the result by 5/2, exactly 50 heads has chance 7.96%, and 45 to 55 heads (72.87% of runs) end anywhere from 0.000053 to 0.503. | "A run with exactly fifty of each ends at zero point nine to the fiftieth power, about half a percent of where it began. That is the middle run. Five heads more or fewer changes the result by a factor of about a hundred, so runs scatter widely, but the middle of them sits far below the start." Add "the middle run" to the card. |
| **M2** | `script.tsv:64` (12_5, "averaging distances on the ruler, not the multipliers"); `money_limits.rs:951` | Overbroad. Averaging the multipliers is exactly right for the average over runs, and 12_2 says so ("really is about one hundred thirty-one"). The rule is right only for the typical run. | "Chance in the multipliers splits the question. For the average over many runs, average the multipliers. For a typical run, average the distances on the ruler." Change the table row to match. |
| **M3** | `turning_rate.rs:149-158`, `:166`, `:174`; `handoff706.md:76` | Answer motion before the 13_2 question. Multiplying by i is one jump, but 13_1 sweeps an arrow continuously using `on_circle(now)`, which is E(i now), so its tip traces half the unit circle before "Where will the output travel?" The test at `:555-569` only bars `.unit_circle(`. | Show the jumps: arrow at 1, arrow at i with a right-angle mark, arrow at -1, with no tracker. Bar a live tracker inside `the_quarter_turn` in the test. |
| **M4** | `script.tsv:65` (13_1); `turning_rate.rs:21`, `:132-143`, `:187-207` | i is used without being defined, and letters come before numbers. The script says "complex numbers can" and "Multiply by i", but never that i times i is minus one, so the quarter turn is asserted. Chapter 13 also has no situation before its abstraction, while its best one, the spring, waits until 13_5. | Open 13_1 with the spring: "Start with a weight on a spring. Its acceleration is minus its position. Apply a rate twice and you get the rate squared, and no real rate has a negative square. So add one new number, i, whose square is minus one, and keep every other rule." Then a worked case: "One and a half across and one up. Multiply by i, and it becomes minus one plus one and a half i." Change `EXAMPLE` to (1.5, 1), which stays inside the axes. 13_5 becomes the payoff. |
| **M5** | `script.tsv:52` (10_1) | The chapter has no situation. "A rate that starts at one, and falls in a straight line through zero" describes nothing a viewer can picture, against the project rule of a situation before an abstraction. | After "So change it", add a clock-driven example: "Picture algae in a pond through a spell of cooling weather. At first each bit of algae makes more. As the water cools, that rate falls, passes through zero, and becomes dying back." Then give the invented straight-line rate as this example. |
| **M6** | `script.tsv:58` (11_2, "Continuous rates are the ones that add"); `money_limits.rs:267-291` | "Continuous rate" of a yearly multiplier is never defined, and `\ln 1.2` appears on screen though 07_3 only named base ten and the natural logarithm is first named in 15_2. What is said and what is shown do not meet. | Before that sentence: "Ask which input of E gives one point two. It is about zero point one eight two. That input is the year's continuous rate. For zero point nine it is about minus zero point one oh five." Show `E(0.18232)=1.2`, `E(-0.10536)=0.9` and `E(0.07696)=1.08` in place of the `\ln` line. |
| **M7** | `euler_identity.rs:157`, `:215`, `:360` | Orphan object, certain from source. The green `landing` ring is never faded: it is not returned from `they_land` and not in line 360's list. It stays through 14_5 and 14_6 at (-3.35, -0.30), offset (-0.05, -0.05) from the origin of the new near plane, radius 0.14, around the tails of the arrows. | Return it with `landing_name`, fade it at line 360, and bind the order in a test. |
| **M8** | `changing_rates.rs:255`, `:428-435` | Overlap, certain from source. `this_rate` ("r(t)=1-t", about 1.6 wide, at (3.6, 1.0)) is never faded, and the 10_5 row "4 cells: 1.40625" (about 1.94 wide) lands at (3.6, 1.05), printing over it. | Return `this_rate` and fade it at the start of 10_5. The rate line remains in the panel and the clock line restates R(t). |
| **M9** | `money_limits.rs:302`, `:334`, `:353`, `:356`, `:370` | Overprint, certain from source. In 11_3 the good and bad withdrawals are vertical strokes on the same x = -3.6 and coincide from y = 0.25 down to -0.95. The retained "90" label is centred at (-3.6, -0.05), on those strokes. "108" also stays with its path gone. | Offset the two drops by about 0.12 either side of the column. Move "90" to dx = -0.45, dy = -0.12, and keep "108" only if the 11_3 subtraction points to it. |
| **M10** | `crates/explainer_kit/src/narration.rs:41`, `:1164-1168`; all six `k.hold` calls; `handoff706.md:80` | Holds are at most `BREATH`, 0.6 s, as written. The handoff admits this, and 10_2 and 11_2 ask for arithmetic. | Author the think time per pair and set it against real takes. As a proposal only: three to four seconds for 10_2, 11_2 and 12_1, two to three for 12_3, 13_2 and 14_1, then decided by listening. |
| **M11** | `script.tsv:76` (15_1); `log_bridge.rs:42-43` | "Five descriptions, and they turned out to be one object" overclaims for the fourth: 2^x obeys f(a+b) = f(a)f(b) too, with slope 0.69315 at zero. Rows one to three pin E. The ruler row depends on H3. | "A rule that adding inputs multiplies outputs, once its slope at zero is one." Add `E'(0)=1` to the row. |

## Low and wording

| ID | Where | Defect and evidence | Repair |
|---|---|---|---|
| **L1** | `script.tsv:63` (12_4, "cannot tell you"); `money_limits.rs:879` | An unconditional sentence that fails at its edge: exact data separates them at once (ratio 1.0519 at t = 0.5). Picture evidence: the curves are 0.029 scene units apart at 0.5, under one 0.04 stroke, but 0.116 apart at t = 1. | "An early stretch of noisy data can hardly tell you which curve you are on." |
| **L2** | `money_limits.rs:790-818`, `:822` | The voice says "half a unit" and "one unit", but the time axis has no ticks. The blue curve stops exactly on the capacity line, so it can read as reaching capacity rather than running off the chart. | Add time ticks. Extend the axis slightly above capacity and let blue cross it, with a separate draw limit so `cap.top` stays as it is. |
| **L3** | `money_limits.rs:727` | The moral says "one run's path" and no path is drawn in 12_2. | Draw one exact alternating run on the ruler: +0.405, -0.511, repeated, reaching -5.268 after 100 tosses. No randomness is needed. |
| **L4** | `euler_identity.rs:182-201`; `script.tsv:71` | The four rows and polygons finish in 1.5 s while the 19-word sentence takes about 7.8 s, so 1024 appears while the voice is on sixteen. No new chapter uses `phrase::until_phrase`. | Hook "sixty-four" and "one thousand twenty-four" by phrase. |
| **L5** | `turning_rate.rs:253` | The red inward arrow lies exactly over the gold position arrow in 13_2. | Offset one by about 0.06. |
| **W1** | `script.tsv:58` (11_2 first sentence) | "A quoted percentage is a convention for describing a multiplier" follows a cue where one quoted rate gave three multipliers. | "A yearly percentage change is a way of describing that year's multiplier." |
| **W2** | `script.tsv:61`; `money_limits.rs:681` | Said "about one hundred thirty-one", shown 131.50 (exact 131.5013). | "about one hundred thirty-one and a half". |
| **W3** | `script.tsv:70` (14_1); `euler_identity.rs:135` | "Every step runs along a tangent" of the drawn circle is true only of the first step. With four cells the step lines lie 1, 1.2716, 1.6169 and 2.0559 from zero. | "Every step runs at a right angle to its own arrow, so every step ends farther from zero than it began." |
| **W4** | `script.tsv:71` | "the square root of one plus pi over n squared" is ambiguous when heard. | "one plus the square of pi over n". |
| **W5** | `script.tsv:63` | "The same caution" points back past the reciprocal argument. | "The caution about early data applies to an economy's growth rate too." |
| **W6** | `script.tsv:70` | 13_4 has already given E(i pi) = -1, so 14_1 tests agreement more than it asks for discovery. | "The walk says one place. As the cells get smaller, do these paths agree?" |
| **W7** | `script.tsv:78` | "But what if we know the output and ask for the input?" repeats what 15_2 has just done. | Lead with the new difficulty: "So far, each question had one answer." |
| **W8** | `script.tsv`, no occurrence | The file is `euler_identity.rs`, yet the film never says Euler. | Name "Euler's formula" after 13_4 and "Euler's identity" after 14_4, once earned. Look up the name and history first, per the production doctrine. |

## Prediction pairs

| Pair | Inputs on screen at the hold | Leak | Result |
|---|---|---|---|
| 10_2 to 10_3 | Present | Inputs point to the wrong answer | **H2** |
| 11_2 to 11_3 | 100, x1.2, x0.9, -50 | None; 63 and 48 hidden | Clean |
| 12_1 to 12_2 | 1.5, 0.6, 100 tosses, mean 1.05 | None | Clean |
| 12_3 to 12_4 | Both laws, curves to t = 0.5 | None | Clean; ticks missing (L2) |
| 13_2 to 13_3 | Three velocity arrows | Sweep in 13_1 | **M3** |
| 14_1 to 14_2 | Four-cell path ending at (-2.32, 1.20) | No mark at -1 | Clean; see W6 |

All six hold lengths are open (M10). The 142 literal `until_sentence` hooks are in range and non-decreasing, and I checked the 5 loop hooks by hand against their cues. The sentence count is the kit's rule.

## Decisive numbers, independently computed

Exact rationals where the value is rational; 80-digit decimals otherwise. Nothing was imported from the crate.

| Claim | Computed |
|---|---|
| Left-edge cells, 4 / 16 / 64 / 1024 | 1.40625 / 1.08705 / 1.02105 / 1.00130 |
| Peak E(1/2) | 1.648721 |
| Credits 1, 2, 4, 12, 365, continuous | 112.00, 112.36, 112.55, 112.68, 112.75, 112.75 (112.74746 and 112.74969 before rounding) |
| ln 1.2, ln 0.9, ln 1.08 | 0.18232, -0.10536, 0.07696 |
| Withdrawal, either order | 70 then 63; 40 then 48; undisturbed 108 less 45 or 60 |
| First head count above the start | 56 (outcome(55) = 0.50330, outcome(56) = 1.25825) |
| P(at least 56), P(at least 60) | 13.5627%, 2.8444% |
| Average, 1.05^100 | 131.501258 |
| Share of average from 60 heads or more | 99.4857% |
| 0.9^50; P(heads at most 50) | 0.00515378; 0.53979 |
| Capacity ratio at t = 0.5, 1 | 1.05190, 1.13746 |
| Unlimited reaches capacity; reciprocal gaps | 2.52573; 11.5000, 6.9751, 4.2306 (ratio 0.606531 = e^(-1/2)) |
| Tangent end, n = 4 | (-2.32060, 1.20370), distance from -1 is 1.78686 |
| Miss at n = 16 / 64 / 1024 | 0.35638 / 0.08010 / 0.00483 |
| Term sizes for k = 0 to 6 | 1.00, 3.14, 4.93, 5.17, 4.06, 2.55, 1.34 |
| Partial sums to k = 8 | (1,0), (1,3.14), (-3.93,3.14), (-3.93,-2.03), (0.12,-2.03), (0.12,0.52), (-1.21,0.52), (-1.21,-0.08), (-0.98,-0.08) |
| Sixteen terms, miss from -1 | 4.24e-6 |
| n(2^(1/n) - 1), n = 1, 4, 16, 64, 1024; ln 2 | 1.00000, 0.75683, 0.70838, 0.69691, 0.69338; 0.69315 |

## What holds

- Chapter 10: the area argument and the exact proof by the chapter 06 test, including `TEST_LINE`; continuity is the right hypothesis.
- Chapter 11: all arithmetic; the undisturbed-minus-cost check; the two-line variation-of-constants formula. Disclaimers appear in voice and on screen, and the scope stays arithmetic.
- Chapter 12: the coin figures, the share of the average, the logistic ratios and the exact reciprocal decay.
- Chapter 13: series exists by the same ceilings; term-by-term rate law; x prime is minus y; the conserved distance; speed one; arc length as angle. "Nobody assumed this" is fair.
- Chapter 14: the excess-length bound, turning angles approaching pi (2.663 at n = 4, as the handoff says), the series growing while k is under pi, and the honest treatment of e to the i pi.
- Chapter 15: all backward forms, including the series interval, and no trespass on the companion film.
- The two models contain no complex exponential, so the three constructions are separate. `choose` returns `u128`, so 100 choose 50 does not overflow.
- Handoff counts checked: 51 files, 27 cues, 2,534 words, 37 authored tests.

## Open after this review

Native build, tests and Clippy; native facts against the oracle; glyph probe; layout lint; draft frames and motion; Frederick Surrey takes; fine sync; listening and independent audiovisual review. Any repair above changes these bytes, so the hashes here will not apply to `expanded-stage09`.
