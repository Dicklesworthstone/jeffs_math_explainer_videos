# Exponential sections A–C: review of the native candidates

**Reviewer:** WhiteSwan, for BlueSalmon's Mail 1827, bead 87do.62.

**Evidence:** I read the source, the pinned API and the brief, ran `rustfmt --check`, and ran the Python checks in `checks/`. There was no Cargo build, test run, Clippy, glyph probe, native facts-check or render. I looked at no pictures, because none exist. No Oliver take exists for these lines, so nothing has been listened to and every timing below is arithmetic from the Kit's scratch rule.

## This is not fully an independent review

Mail 1827 says I did not author these candidates. That is true of series01 and product01, and of root's fact export in facts01. It is not true of rate02.

| Candidate | My relationship to it | What this review is worth |
|---|---|---|
| `exponential-expanded-rate02/` | `rate_law.rs`, `rate_model.rs` and `script-slice.tsv` are byte-identical to my own repair in `exponential-expanded-rate01-independent-review/candidate/` | **Self-review.** Rate02 still needs another reviewer before anyone counts it as independently accepted. |
| `exponential-rate-facts01/` | Its `rate_model.rs` is my model plus root's `append_facts`, and they match once that function is removed. The export, oracle and wiring are root's. | Independent for the export, oracle and wiring only. |
| `exponential-expanded-series01/`, `exponential-expanded-product01/` | New to me | Independent. |

The patches I staged here are also my own work, so they need a reviewer other than me.

## Verdict

**The mathematics in the cue text is correct, but series01 and product01 should not be integrated as they stand.**

- **Series01** would fail facts-check on 11 rows, by prediction from a Python mirror. Its prediction hold shows the wrong sentence, and the uniform-convergence conclusion never reaches the screen.
- **Product01** would fail Clippy on a test-only function and has no fact rows. It also removes the diagonal totals just before the sentence that depends on them.
- **Rate02 and facts01** hold, subject to the self-review above. The fact export agrees with its oracle on all 166 rate rows in the mirror.
- **Both proof chapters label more than they demonstrate.** The patches fix the beats and add motion to the coefficient table. The tail bound and the enlarged diagonal still need a design decision (S8, P6).
- **The scope has a gap.** The brief's step that names `e` and earns `E(x)=e^x` has no cue (P10).

Sections A–C are not a replacement for the full scope. Sections D–I are still open.

## Bindings

Folder: `docs/video-program/third-wave-20261024/opus-replacements-20261104/`.

| File reviewed | sha256 |
|---|---|
| `exponential-expanded-rate02/rate_law.rs` | `093f37e54b73b8e4634c8fea365ae8110a374e565747cbb86d17a1ebfa9f120a` |
| `exponential-expanded-rate02/rate_model.rs` | `48f3dae06f4b696afe24012b3bafa85ac59522925527d704674c13d4cc71189d` |
| `exponential-expanded-rate02/script-slice.tsv` | `1b5257beac7dd0e08a300c4625d8914f80ebc8af5bed718b7556587f6317e63e` |
| `exponential-rate-facts01/candidate/oracle.py` | `f042a80e4c5689b4a33f746fd599078da4d1fb7f0add929fb3cf183f180574fa` |
| `exponential-rate-facts01/candidate/src/facts.rs` | `afff33feaf5ef85cfb68f9ad9c041e630e14e4cef52a636d420f6e33e2bd7c14` |
| `exponential-rate-facts01/candidate/src/main.rs` | `5c3c665feb9e2c61a4a51e21dcd2f8d42672e01c2de8d7cb4375d8f77b1c1e2d` |
| `exponential-rate-facts01/candidate/src/rate_model.rs` | `8d896cf9d23d6a6f87289d7cbe25c89ee54537161e77a93978c21b4ae91d2edc` |
| `exponential-rate-facts01/oracle.tsv` | `d2930a2a1627c7a3035aae05feadffbe3aa6401a86856a27327c35e015931709` |
| `exponential-expanded-series01/factorials.rs` | `5b7ecd2ae11e9bc718a3cbc95499d26e59f2beaad306ebe2de28210522ce54a8` |
| `exponential-expanded-series01/series_model.rs` | `058392239fd864f88ea9ec9ba79508192c243a3e933d5cb3712e63c2c54165bc` |
| `exponential-expanded-series01/script-slice.tsv` | `fe8480fbdef3524caee8e53ec507010ed052700319176a12d920282b4c23fef7` |
| `exponential-expanded-series01/oracle.py` | `f18530ae11973f838c18016bc6a60cb18c997fdbc5f99c9dd6e4737f9ebc0edc` |
| `exponential-expanded-series01/oracle01.tsv` | `ebf22c6c83fc2027d05dbf8d234229bce1c7e295dfc91e9d07e738a08aeda9a1` |
| `exponential-expanded-series01/README.md` | `f521030184e00496379e45ae9f21370e2542e0e8542863b2ed7fe5c60fc0bec7` |
| `exponential-expanded-product01/product_grid.rs` | `afa94c76ba69854416df7a32be903c24fc6ab804195d7ebed1bbba599827b89e` |
| `exponential-expanded-product01/product_model.rs` | `ee9d24de06f13be46f6d7cc7e783358bac33478f636e6eb1ee8bd158ac5d7baf` |
| `exponential-expanded-product01/script-slice.tsv` | `a66c795969e6975ff2f44bab420cf470e4a8d92e67378ae7f98316285d7cce06` |
| `exponential-expanded-product01/oracle.py` | `360226c5b97fa48ba2c0322434b7c9a9b8bbc9672dfd7f015e3013c5bdfea16c` |
| `exponential-expanded-product01/README.md` | `8189300f1ef5c7053ef6d7ae0976bd15770bef3479b19ff098cf6c8ded70f23c` |

| Patch | sha256 | Applies to |
|---|---|---|
| `patches/series-repair.patch` | `2ea6c1247df11bb52133afa4d4f4337559f95c4de7c2885f41ae909d5aa26953` | `factorials.rs`, `series_model.rs`, `oracle.py` in series01 |
| `patches/product-repair.patch` | `32946317587f296df366bc356fea3303341109268f738a5ef8a4f5b281d83255` | `product_grid.rs`, `product_model.rs`, `oracle.py` in product01 |

Apply each with `patch -p1` inside a copy of its folder. `base/` holds the exact originals and `candidate/` the patched results. `SHA256SUMS` covers this whole folder. No original was changed.

## Findings

Severity: **Blocker** stops a gate. **High** means a wrong picture, a missing step in the teaching, or a required rule broken. Timings use factor 1.0, with factors 0.8 and 1.25 in brackets where they matter. Line numbers after "the brief" refer to `docs/video-program/research-20261016/evidence/exponential-logarithm-sister-videos-20261101.md`.

### Series: `exponential-expanded-series01/`

| # | Severity | Where | Problem | Repair | Patched? |
|---|---|---|---|---|---|
| S1 | High | `factorials.rs:106`, `:172`, `:174` | Three indexed strings land on the wrong beats | Index the intended strings on their sentences | Yes |
| S2 | Blocker at facts-check | `series_model.rs:51`, `:57`; `oracle.py:21`, `:26` | 11 rows are predicted to differ, and the screen prints 0.08437 | Five-place heights, rounded-up bound, exact tie assert | Yes |
| S3 | High, on-screen maths | `factorials.rs:40–41` | Both inequalities are stated without their hypotheses | Add the hypotheses; show the actual ratio | Yes |
| S4 | High, teaching | `factorials.rs:139` | The table is gone before 02_4 and 02_6 refer to its rows | Bring the table back for those cues | **No, design choice** |
| S5 | High | `factorials.rs:98–105` | Connector lines appear, but no term moves | Move each term with `Transform` | Yes |
| S6 | High, score | `factorials.rs:108–109` | No `predict` mark and a bare `wait`, so the think gap is invisible downstream | `hold`, `mark`, `music_breath` | Yes |
| S7 | High, timing | `factorials.rs:49–55`, `:77–96`, `:106`, `:125–137`, `:184–187` | The picture leads or lags the voice by seconds | Parallel fades and sentence hooks | Yes |
| S8 | High, demonstrate vs label | `factorials.rs:185–201` | The tail bound is TeX only | Bars under a geometric envelope | **No, design choice** |
| S9 | Medium | `factorials.rs:157–169` | 300 `Line` objects; only the last curve is coloured; no degree labels; `x≥0` only | One polyline per curve; dim the previous curve | Partly |
| S10 | Low | Listed below | Smaller source and picture points | Listed below | Partly |

**S1.** The string lists are out of step with the beats:
- The prediction hold draws `TEXT[22]`, "A finite polynomial loses its highest power", when it should ask `TEXT[21]`, "Which coefficient makes the constant column agree?".
- 02_4 then draws `TEXT[23]`, "The infinite tail needs a proof", in place of the finite-polynomial line.
- `:174` waits for sentence 3 and then immediately holds, so nothing lands on "But dots are not a proof."
- `TEXT[24]`, "Uniform convergence on every bounded interval", is never drawn, so the chapter's main analytic conclusion is voice-only.

**S2.** `checks/fact_mirror.py` predicts 11 differences:
- Ten curve heights at six places (for example `factorials.curve.2.1.y`, native 1.025312, oracle 1.025313). These are exact decimal ties, which the native sum and the exact fraction round apart.
- `factorials.tail.bound`. The native path gives 0.08437499999999999, which prints **0.08437**. The oracle and the README say 0.08438.

The printed 0.08437 is still above the true supremum of the tail (0.083252), but it is below the bound the formula gives (27/320 = 0.084375). The README's "upward-rounded" is therefore not what the code does. Five places is tie-free across all curve rows: mismatches by precision were 3:0, 4:4, 5:0, 6:10, 7:0, 8:0, 9:2. The staged oracle asserts that exactly.

**S3.** The voice says "eventually" and the screen doesn't:
- `TEX[1]` claims `R/(k+1) ≤ 1/2` with no condition. It is false at R=10, k=1.
- `TEX[2]` needs `N+2>R`. At R=10, N=4 the factor `1−R/(N+2)` is −2/3, so the printed bound is negative.
- The worked read-out uses ratio 1/4 at R=3/2, N=4, while the voice only ever says one half. The staged read-out now prints the ratio so the two can be reconciled on screen.

**S4.** "Removes its highest power" (02_4) and "the shifted coefficient rows now match all the way across" (02_6) refer to rows that were wiped at `:139`, 02_3 sentence 4. My recommendation is to bring the table back in 02_4 with an empty cell under `x⁴` and the next term arriving, then in 02_6 extend both rows with dots. It needs layout decisions a first preview should inform, so I did not stage it.

**S5.** The brief asks to "Animate each derivative coefficient downward into its matching power column" (`exponential-logarithm-sister-videos-20261101.md:191–192`). Sentence 3 says differentiation "moves each term one column left". The candidate draws the result and a static line. The patch moves a gold copy from each box into its new column, picking up its power.

**S6.** `kit.mark` is what `crates/explainer_kit/src/captions.rs:533`, `tools/narration_delivery.py:249` and `tools/narration_director.py:202–204` read. Only breath and listen rows get score release windows, so a bare `stage.wait(4.0)` is an unauthored silence. Rate02 already does this correctly (`rate_law.rs:136–138`). The same defect is in product01 (P3).

**S7.** Before and after, from the replay:

| Beat | Original (`checks/timing.out.txt`) | Patched (`checks/candidate_checks.out.txt`) |
|---|---|---|
| 02_1 table complete, after "boxes" begins | 4.63 s (6.01 / 2.90) | 0.60 s |
| 02_1 question on screen, relative to the sentence asking it | 3.01 s early (0.53 late / 5.76 early) | 0.35 s after it starts |
| 02_3 first curve starts, after "watch" begins | 7.50 s | 0.70 s |
| 02_3 last curve done, relative to the cue's end | 1.68 s after (3.26 after / 0.30 before) | 2.12 s before (0.54 / 4.10 before) |
| 02_5 opening with no formula | 8.79 s (7.04 / 10.99) | None: the interval appears at once |

Most of the 02_3 delay comes from `clear` fading 45 objects in series at 0.12 s each, which spends 5.4 s while the voice has moved on. The think time before the answer is 4.3 s in the patch. Total length barely changes: 142.87 s to 141.75 s.

**S8.** The brief wants "A zoomable tail bound" beside the overlay "rather than presenting agreement of a few pixels as proof" (`:217–218`). The candidate has formulas and one read-out. My recommendation is bars of `R^k/k!` beyond degree N under a geometric envelope, with the bar heights added to the facts. It is new picture work, and it is where this chapter would most clearly demonstrate its argument.

**S9.** The patch draws one polyline per curve and dims the previous one with `set_stroke`. It must not use a recolour there: at fmn pin a399d626, `SetColor` also sets fill alpha (`fmn-mobject/src/animate.rs`), which would fill an open path. Degree labels and the negative half of the interval are not staged. The next cue says "from minus R to R" while the chart shows only `x≥0`, so a preview should decide whether that reads as a contradiction.

**S10.**
- Axis labels were literals (`:143–146`); the patch uses the constants. A hand-rolled `map` (`:140`) and box lines (`:78–83`) remain, where `Frame2` (`crates/explainer_kit/src/plot.rs:81`) and `rect` (`plot.rs:46`) exist.
- Headers `x²`, `x³`, `x⁴` were served by IBM Plex beside Computer Modern, a mixed font. The patch draws them as TeX.
- `TEXT[13,17,18,19]` are never drawn. They are harmless, so I left them to keep the indices stable.
- The test `derivative_coefficients_match_the_previous_power` only checks the factorial recurrence, so its name overclaims. Left unchanged.
- `README.md:7` cites 324 tail samples, but `oracle.py` doesn't compute them, so that check can't be re-run from what was kept.
- The 02_6 formula is up from the cue's start and stays still for about 25 s. Not staged.

### Product: `exponential-expanded-product01/`

| # | Severity | Where | Problem | Repair | Patched? |
|---|---|---|---|---|---|
| P1 | Blocker at Clippy | `product_model.rs:6` | `choose` is used only by tests in a binary crate | `#[cfg(test)]` | Yes |
| P2 | High, AGENTS rule 2 | `product_model.rs`; `oracle.py` | No fact rows for 16 denominators, totals or the spoken "sixteen"; the oracle prints JSON | `append_facts` and a key/value oracle | Yes |
| P3 | High, score | `product_grid.rs:137–138` | No `predict` mark, bare `wait` | As S6 | Yes |
| P4 | High, teaching | `product_grid.rs:141–143` | Totals are hidden at 03_3, so 03_5's key sentence has no picture | Keep totals in a column | Yes |
| P5 | Medium | `product_grid.rs:66–87`, `:104–112` | No vertical rules, no role labels, no dots; rules end at 0.5 not 0.55 | Rules both ways, labels, dots | Yes |
| P6 | Medium | `product_grid.rs:119–134` | Cells never move into a sum; no enlarged diagonal | Enlarge degree two and send copies to the sum | **No, judge on first preview** |
| P7 | Medium | `product_grid.rs:149` | Plain `Transform` between unrelated TeX | `TransformMatchingTex` with `Kit::tex_spans` | **No, native API check needed** |
| P8 | Medium | `product_grid.rs:159`; `TEXT[3]` | The completeness caveat is never drawn; the general identity is never shown | Show both on their sentences | Yes |
| P9 | Low | `product_grid.rs:66`, `:135`, `:175` onward | Early reveals in 03_1, 03_2 and 03_6 | Sentence hooks | Mostly |
| P10 | Medium/High, scope | No cue | The step naming `e` is missing | New cue, C4 below | **No, script decision** |

**P2.** AGENTS rule 2 requires every number to come from the facts. The drawn denominators are correct, because `math_exact` found the grid exact, but nothing binds them. Printing JSON to stdout would also pollute `oracle_diff`, because `crates/explainer_kit/src/facts.rs:122` turns any non-tab line into a key. The patch sends the summary to stderr and derives each cell by a separate route: `C(i+j,i)/(i+j)!` in the oracle against `i!·j!` natively.

**P4.** 03_5 sentence 2 says "The diagonals are precisely the terms of E at x plus y." That is the proof's central observation, and the original has removed the totals by then. In the patch the totals `1`, `x+y`, `(x+y)²/2` and `(x+y)³/6` stay in one column, so the terms of `E(x+y)` build up in view. The working formulas use a second column, and `TEX[8]` states the sum on that sentence.

**P5.** "Each square contains one product" is voiced over a grid with only horizontal lines, and the rules appear after the cells. The patch draws rules both ways first and adds `E(x)`, `E(y)`, `\cdots` and `\vdots`, so the edges read as two series that continue.

**P6.** The brief says "Enlarge the degree-two diagonal before generalizing" (`:224–225`), and the README lists diagonal legibility as something to inspect. The patched picture still colours cells without moving them. The right panel may already be enough once the totals stay, so I would decide on the first preview rather than build it blind.

**P7.** `TransformMatchingTex` has a precedent (`explainer_corpus/square_packing/src/chapters/the_turned_tile.rs:249`) but needs spans chosen per pair. I did not stage it uncompiled. The README already lists formula morph frames as something to inspect.

**P8.** `TEXT[3]`, "Complete diagonals: degrees 0 through 3", is the guard against reading 16 cells as the whole answer, and it is never drawn. The brief also asks for the identity (`:244`), which the voice speaks at 03_4 sentence 2 with nothing on screen. Both now land on their sentences.

**P9.** Originally, before the voice reaches them: edges 2.32 s early, `x+y` 1.92 s, the question 2.49 s, `E(0)=1` 1.50 s, positivity 4.25 s. All are now on their sentences. Positivity is a two-step reveal, with the law at sentence 2 and `>0` at sentence 3. Sixteen serial cell fades ran past the next two sentences, so the patch shortens them to 0.2 s each; they now finish 1.29 s (0.31 / 2.51) before sentence 5. "Never vanishes" is still voice-only.

### Rate and facts

| # | Severity | Where | Finding |
|---|---|---|---|
| R0 | Process | rate02 | Self-review, as above. Needs another reviewer. |
| R1 | Medium | `rate_law.rs:94`, `:108` | Fact export is sound: 166 of 166 rows by mirror, a closed-form oracle independent of the recurrence, bindings and baselines correct, no dead code. Literals "time: 0 to 1" and "constant rate = 1", tick slopes and the 0–3 axis are outside the facts. |
| R2 | Low | `exponential-expanded-rate02/README.md:3`, `:6` | Stale: says 244 words and two tests; the slice has 248 words and the model has 4 tests. |
| R3 | High at integration | `script-slice.tsv` in all three; chapter keys | Cue ids `01_*`–`03_*` collide with the canonical film's, so `check_script_owners` will trip. Assign final ids before fresh bindings. |
| R4 | Medium, teaching | 01_1; canonical chapters | 01_1 starts cold and relies on a connection to an opening that isn't written. Canonical chapters overlap this material, and placement is undecided. The brief's picture was replaced by the two-panel design root accepted; I agree with that choice. |

## Native API use

The brief says "Use native FrankenManim trackers, curves, TeX and transformations throughout" (`:166–167`). All the source is native, with no prototype code. Use of the richer tools is thin, though.

| Candidate | Trackers or updaters | `Transform` | Notes |
|---|---|---|---|
| rate02 | None | Yes, in a helper (`rate_law.rs:39`) | Computed geometry, predict marker and sentence hooks are good |
| series01 | None | **None** (the patch adds moves) | 300 `Line` segments for five curves |
| product01 | None | One (`product_grid.rs:149`) | Colour changes run in parallel, which is good |

By reading, I found no compile error in series or product, and each API form I staged has a precedent at the pin: `polyline`, `set_stroke` (`fmn-mobject/src/animate.rs:601`), `prepare`/`play_prepared`, `mark`, `music_breath`. Nothing has been type-checked, so a first native build may still find errors.

## Script corrections, reported separately

No takes exist for these lines, so changing them costs nothing now. **None is applied.** The patches keep every cue's words unchanged.

| # | Cue | Problem | Proposal |
|---|---|---|---|
| C1 | 02_6 | "We still owe two explanations" silently drops 01_5's other debt, that the stepped approximations reach the curve | Say three, and "answer all three", or pay the debt explicitly |
| C2 | 03_5, 03_4 | By ear, "E of x plus y" can mean `E(x)+y`, and "x plus y to the m" can mean `x+y^m` | "E of the sum, x plus y"; "the sum x plus y, raised to the m" |
| C3 | 02_6, 03_2, 03_3 | The script lint warns on colons | Replace them with sentence breaks; check sentence indices afterwards |
| C4 | New 03_7 | P10: the brief's step at `:245–247` has no cue | "Now we can name it. Call E of one by the letter e. The law says E of two is e times e, and E of three is e cubed. Roots give fractions, and continuity fills in the rest. That is when writing E of x as e to the x becomes a result, not a label." |
| C5 | 02_1 | The screen asks "Which coefficient…", the voice "what must the coefficient of x be" | Consider "What must a be?" on screen |

## What holds

- **The cue mathematics is correct**, checked in exact arithmetic where it can be (`checks/math_exact.out.txt`): coefficients, the shift, the ratio argument, 27/320, uniform convergence, term-by-term differentiation, regrouping by absolute convergence, non-vanishing, positivity and monotonicity.
- **Drawn values share one model with the facts** in series01 and rate02. The product grid's 16 denominators and diagonals 0–3 are exact, and complete through degree 3.
- **Answers stay hidden during think time** in all three chapters.
- **Sentence indices match the Kit's splitter.** A script confirmed the 25 literal waits in the patched scenes name real sentences, and I checked the three loop-driven ones by reading.
- **All checked plain text fits the frame** by font metrics (`checks/text_fit.out.txt`).

## What each patch changes

**Series.** Findings S1, S2, S3, S5, S6, S7, part of S9 and part of S10. New: `tail_ratio`, `tail_label`, the fact `factorials.tail.ratio`, two source-order tests and a tail-label test. `TEXT` is unchanged, so its indices still hold. The oracle now has 626 rows.

**Product.** Findings P1, P2, P3, P4, P5, P8 and most of P9. New: `append_facts` (26 rows), `TEX[7–10]`, two source-order tests. Its replayed length is unchanged at 144.21 s.

The patches are larger than "small": 417 and 336 lines, because moving beats touches many lines. If you want less, the model and oracle changes stand alone from the scene changes, but then the screen keeps printing 0.08437 until `factorials.rs` changes.

### Integration work that remains for root

1. Merge the three oracles into the crate's single `oracle.py`. Regenerate `oracle01.tsv` and `oracle01.json`, which the patches leave stale.
2. Call `series_model::append_facts` and `product_model::append_facts` from `facts.rs`. Facts01 already wires `rate_model` (`candidate/src/facts.rs:35`). Until they are called, they are dead code under Clippy.
3. Register the modules, chapters, glyph strings and final cue ids (R3), then make fresh direction bindings.
4. Refresh the stale READMEs (R2, S10).

## Checks

| Check | Result |
|---|---|
| `checks/fact_mirror.py`, originals | Rate 166/166 agree; both oracles reproduce their stored output; **series 11 differ** (S2). Exits 1, as expected. |
| `checks/math_exact.py` | PASS |
| `checks/timing.py`, originals | Figures in S7 and P9; series 142.87 s, product 144.21 s |
| `checks/text_fit.py` | Checked strings on frame; `²³⁴` come from Plex |
| `checks/source_scan.py` | Index, predict-mark, test-only-name and stale-statement findings |
| `checks/candidate_checks.py`, patched | PASS: patches reproduce `candidate/` from `base/`; `rustfmt` accepts all four Rust files; series 626/626 and product 26/26 by mirror; source-order tests hold on the candidates and fail on the bases |
| Same script, originals | `rustfmt --check` exits 0 on 7 Rust files. The end-to-end skill's script lint has 0 errors, with warnings as C3 and for cues over four sentences. |

The mirrors are Python doubles, which predict native output but are not native output. The source-order tests were replayed by Python, not run by Cargo. TeX widths in the new right-hand column are hand estimates of about 1.4–3.1 scene units against about 4.6 available, not measurements.

## What this review does not establish

- **Native status:** uncompiled, unrendered. No Cargo, test, Clippy, glyph probe or native facts-check ran on the originals or the patches.
- **Pictures:** no independent picture review exists. My layout judgements, including the dense right panel in 03_4–03_5, come from coordinates and must be checked on the first preview.
- **Narration:** ElevenLabs Oliver, not Michael or FrankenTTS. No take exists for any line, so listening, phrase timing and fine sync are all open, and all timings here may move.
- **Independence:** rate02 and both staged patches need a reviewer other than me.
- **Scope:** sections A–C only. Sections D–I are outside this review, and the final audiovisual review and publication are open.

## Decisions for root

1. Apply the two patches, or take only parts of them?
2. Which script corrections to accept, especially C1, C2 and the new cue C4? They should be settled before Oliver synthesis.
3. Whether to build the two design items now (S4 table return, S8 tail bars) or after a first preview. I would build S4 and S8 before picture review, and leave P6 and P7 until after it.
4. Who gives rate02 its independent review?
