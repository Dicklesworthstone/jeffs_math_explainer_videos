# Exponential expanded-stage03: independent source review (674)

CC1 (BronzeJaguar), 2026-11-09. Target: `narration_lab/third-wave-20261024/root/elevenlabs-exponential-20261101/expanded-stage03/native-crate`, seven chapters, 51 cues, 3,112 words. Bound to `source-check04.json`: all 43 hashes rechecked just before writing, no drift.

**Scope.** Source reading against script and oracle, plus the Python oracle and a Python float mirror of the Rust formulas. No Rust was run, and nothing was built, rendered, synthesized or edited. This is not picture, motion, audio-review or independent AV acceptance.

**Verdict.** The D and E mathematics holds; I found no proof or domain defect. Root's 06_8 repair is correct. F's arithmetic and alignments are right, but its picture does not yet teach what the voice says. Findings 1 and 3 should be fixed before F's first draft render. F has no takes, so its wording is still cheap to change.

## Findings, ranked

Paths are relative to the crate. `CR` is `src/chapters/calculating_ruler.rs`. Coordinates in fixes are estimates from source geometry, not rendered results.

**1. High. F layout and pedagogy: the reading geometry hides the reading (`CR` 45-61, 66-68).**
Every `cursor()` runs y −1.4 to 1.05, through the C labels (y .37) and the D labels (y −1.13). Computed from `slide_model.rs`, each cursor strikes the digits being read:

| Cue | Cursor x | Struck labels |
|---|---|---|
| 07_4 | 1.280 | C3, D6 |
| 07_6 answer | 2.500 | C4, D10 |
| 07_6 outside | 2.935 | C3 |
| 07_7 | −2.565 | C3 |
| 07_9 | −1.344 | C1, D2 |

No `allow_crossing` is set, so the lint's `crossed` check should fire. Separately, the rails sit 1.30 apart (C tick foot .60, D rail −.70) and both tick downward, so "the upper three sits above the lower six" is a long-distance comparison. The index alignments (upper 1 over lower 2, upper 1 over 2.5, upper 10 over lower 4) get no mark at all, although they are the action being taught.
Fix: give `rail()` a side parameter. C ticks down with labels above, D ticks up with labels below, as on a real rule. Rails at y ±.30, labels at ±.57, cursor y −.36 to .36. Add a thin marker at the index being aligned. Move the C/D side labels with the rails. `slide_model.rs`, facts and oracle do not change.

**2. Medium-High. D notation: `RATE_RULE = y'=xy` (`src/chapters/tiny_multipliers.rs:48`, shown at 491 on 04_2 sentence 1).**
Chapter 06 uses y(x) with y′ = d/dx, so a viewer can read this as dy/dx = x·y, which is solved by exp(x²/2). Here x is a fixed rate and the variable is time. The voice is right; only the formula is ambiguous.
Fix, TeX only: `\frac{dy}{dt}=x\,y` with a short "x fixed" note. No voice change.

**3. Medium-High. F pedagogy: the first picture does not show the idea it names (`CR` 96-127).**
07_2 opens "Watch the horizontal distance that produces each height", but no horizontal distance is ever drawn; the dots only drop vertically. In 07_1 all eight dots and labels appear together on sentence 3, "What input produces two?", which also answers "What input produces three?" before it is asked. Height labels fade at 07_2 sentence 1, the moment the voice says "write the height beside it".
Fix: draw a horizontal segment from the vertical axis to the curve for 2 on 07_1 sentence 3, for 3 on sentence 4, the rest after. Drop those segments onto the rail, so the distance itself becomes the mark.

**4. Medium. F layout: gold height labels sit on the gold curve (`CR` 108).**
With offset (.2, .14), labels 3, 4, 5, 6 and 8 intersect the curve (clearance −.015 to −.040) and 2 grazes it (+.009).
Fix: `p + v(-0.22, 0.14)` gives clearance .098 to .178 for all eight.

**5. Medium. F script against picture: "The gaps shrink as the labels grow" (07_2; `src/slide_model.rs:5`).**
`LABELS` skips 7 and 9, so the labelled gaps run 5-6 .435, 6-8 .687, 8-10 .533, which do not shrink. Whole-number gaps do shrink (1.656 down to .252).
Fix: add 7 and 9 to `LABELS` and to the matching list in `oracle.py` (rows become 1,100; recheck finding 4's clearance). Or qualify the sentence.

**6. Medium. F vocabulary: one word per idea.**
- "Index" is never defined (07_6 last sentence, 07_7). Say "the end mark of the strip" once, or define it on 07_4.
- On-screen "C: sliding", "D: fixed", "Beyond D" (`CR` 169-170, 210) use names never spoken; the voice says upper and lower. Use "upper: sliding", "lower: fixed", "Beyond the lower scale".
- "restore the decade" (`CR` 223) against the voice's "decimal place". Use "restore the decimal place".

**7. Medium-Low. F timing.**
- 07_5 says "before we move anything" while the strip slides home (`CR` 181-182). Do the reset before `k.say(s, "07_5")`.
- 07_9's shift (`CR` 251) lands before sentence 1 gives the instruction. Add `k.until_sentence(s, "07_9", 1)?` before it.
- 07_7's caption shows "12" on sentence 3, one sentence before "Our answer is twelve". Show "Read 1.2" first and extend it on sentence 4.

**8. Low-Medium. 07_5 asks about 2.5 with no mark on the lower scale.** It is an unlabelled tick at x −.811, between labels 2 and 3. Add a temporary "2.5" pointer for the prediction hold; the answer stays hidden.

**9. Lows.**

| Where | Issue | Fix |
|---|---|---|
| 06_9, `only_solution.rs:24` | "three constructions": y′=y characterises and does not construct (brief's own wording, root may keep it) | "three descriptions" |
| `only_solution.rs:87` | `E(x+y)=E(x)E(y)` beside y the function | `E(a+b)=E(a)E(b)` |
| 06_8 | root's semicolon adds a lint warning | "…and zero k keeps it constant, while a zero start stays at zero." Six sentences and the hook survive |
| 07_1, 07_3, 07_10 | colon or semicolon lint warnings | recast at the next script pass |
| `tiny_multipliers.rs:1581` | 05_6 chart returns to n=16 while the card says 1024 cells and the voice says "the same number of cells" | label the chart as a display count, or keep 1024 |
| `limit_model.rs:313-320` | test name says "climbs by less and less"; it never asserts shrinking increments | add the assertion |
| `CR` 77 | title y 3.0; other chapters use 3.1 to 3.35 | align if not deliberate |
| `CR` 166-167 | "Copy the scale" is a plain fade-in | slide a copy up from the fixed rail |
| chapter header comments | "Type-checked at the pin" describes earlier candidates, not stage03 | reword at integration |
| `oracle.py` F rows | 80-digit Decimal, but the same closed form and right-index rule, so it catches float and format drift, not a wrong geometry choice | state this in the evidence |

## Checked, no finding

- **D:** finite expansion, chain in [0,1], ceilings, tail handled before head, both signs, the 05_7 numbers. (1−3/n)^n strictly increases for n=4..1024 (.00391 to .04957, exact). (1+1/n)^n increments shrink for n=1..399. Warning sum uses columns 1..=n, so "column k can reach one over k" holds. Tight values: chain(1024,100) .00675; totals .07707, .01978, .00514; tails .12657, .00022.
- **E:** product test, level implies constant, multiply by E(x). The ledger is not circular: E′=E comes from term-by-term differentiation (02_5), E(x)E(−x)=1 from the grid (03_5), and only the product rule, chain rule and zero-slope-means-constant (named as an assumed tool) are added. Interval domain in 06_7 is sound.
- **06_8 repair:** only script line 40 changed; six sentences kept; hook phrase present. On-screen curves start at 2 plus the y(0)=0 line, and no negative start is claimed. The `k.tex` fix is exactly one inserted `&[],`.
- **F:** arithmetic and alignments are correct (left and right index, division). I found no compile defect by reading: helpers match D-E patterns and kit signatures, and `AnimateBuilder::shift` exists at the pin. That is not compilation.
- **Hooks:** all 51 cues are said in script order, all sentence indices in range, phrases unique. Rechecked: `rate_law.rs` 144-174 is branch-per-step code, a scanner artefact.
- **`\binom`:** the tall-delimiter fix 370abc30 is an ancestor of pin a399d626. Still inspect it on the probe.
- **A-C:** script statements read sound. Not reviewed to the D-F depth.
- **Runtime budget:** the longest chapter is 07, about 240 s at 150 wpm, under the 368 s raw cap. Estimate only.

## Evidence: authored against run

| Group | Authored `#[test]` | Run evidence |
|---|---|---|
| A-C | 46 | TODO records 46 tests and 883 facts at the 2026-11-04 build |
| D-E | 44 | `exponential-de-native01/HANDOFF.md` records author-run tests (41 model, 23 scene) and native facts equal to oracle 1079, on byte-identical D-E files in a separate integration; its type-check did not link |
| F | 3 | Authored, never run |
| **Total** | **93** | **None run on the integrated stage03 bytes.** Root reports rustfmt only |

Run by me (Python only): `oracle.py` gives 1,098 unique rows equal to `oracle03.tsv`, first 1,079 rows sha 38df7e3c. The float mirror matches 215/215 tiny/only/ruler rows; it is a mirror, not native output. `lint_script.py`: 0 errors, 52 warnings.

## Still open

Guarded native build, tests, Clippy and facts/oracle equality on stage03; glyph probe (including `′`, `≤`, `×`, `÷`, `→` weight and fallback); layout lint; 101-bar morph runtime; the "?" curve reading; all pictures and motion; Frederick takes, fine sync, audio-review and independent AV. Chapters G-I are outside this review.
