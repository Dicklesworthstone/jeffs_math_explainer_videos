# Exponential A–C teaching repair

Staged by LavenderPike for BlueSalmon, Mail 1961. Narrator: ElevenLabs Oliver only. Nothing here was synthesized, and no take is claimed or selected.

**Verdict.** The source repair is complete for sections A–C, and every source check below passes. All three scenes and their models type-check and pass Clippy at the pin. The repair is not yet accepted as film. It has never linked, run or rendered, so no picture or sound exists to judge.

**Self-review disclosure.** I wrote the review this repairs (`exponential-abc-cross-review02`, Mail 1954) and I wrote these repairs. My checks test arithmetic, order and types. They cannot test my own teaching judgement. **This repair still needs an independent review.** The frozen review is untouched, and `checks/bytes_check.py` confirms it against its delivered hash.

Full A–I remains the scope, and nothing was shortened. Only A–C is touched here. D–I stay open.

## What each finding became

| Finding | Repair | Where |
|---|---|---|
| **N1** factorials with no picture | The rule becomes a chain, `u_0 →÷1→ u_1 →÷2→ …`. On "We get one, one…" each value lights up while its divisions gather below it (`1·2·3·4`). On "Those denominators are factorials" those products morph to `4!`. That is the chapter's first factorial sign, and a source test enforces this. | 02_3 s1–s3 |
| **N2** polynomial and dots never drawn | `y = 1 + ax + bx² + cx³ + dx⁴` stands over the columns from 02_1 to the end of 02_4. The real dots land at 02_4 s2, on both rows, and pulse on "But dots are not a proof". | 02_1 s1, 02_4 |
| **N3** matching labelled, not shown | Coefficients fall out of their terms into boxes. A copy of each term moves one column left and picks up its power, with a connector. A frame moves to the column being matched. That column's equation prints under it, gets solved, and the value rises into the next box, where the next column reads it. | 02_1 s2–s4, 02_2, 02_3 s0 |
| **S4** table gone too early | The polynomial survives the table and the chart. The factorials rise into it, and 02_4 uses it. 02_6 rebuilds the two rows, with their dots, by moving copies left. A frame walks the columns on "all the way across". | 02_3 s4, 02_4, 02_6 s1 |
| **S8** tail bars and zoom | Nine bars at `R = 3/2` with the exact ratio row (`3/2, 3/4, 1/2, 3/8, …`). The tail bars enlarge by copy into a ×16 inset, under green halving ceilings, with dots after the last bar. The geometric sum and the printed bounds follow. | 02_5 s1–s3 |
| **N10** derivative series voice-only | Gold copies of the bars land one column left, on the bars they equal, beside `d/dx x^k/k! = x^(k-1)/(k-1)!`. | 02_5 s4 |
| "Uniform" (brief `:212–218`) | Bars and dot go live on one tracker and sweep from `R` to `−R` under ceilings that don't move. | 02_5 s5 |
| **N11** the half shown early | Two steps: `|x|/(k+1) ≤ R/(k+1)` at s1. The half and its hypothesis arrive at s2, on "Eventually", while the ratios that reach one half turn green. | 02_5 s1–s2 |
| **N8** `a_0` clash | The recurrence now uses `u_k`. Tests hold it to that. | 02_3 s1 |
| **N9** `y'=y` never written | `y'=y, y(0)=1` lands at 01_5 s0, and 02_1 opens on the same equation. | `rate_law.rs`, 02_1 s0 |
| **N13** orphaned `TEXT` | Closed by construction: the series scene has no `TEXT` or `TEX` arrays. The glyph probe captures actual strings. | `factorials.rs` |
| **N7** rows to degree, static | A frame on one row, then on "collect them by" it becomes a line along the degree-three diagonal. | 03_2 s0 |
| **N6** diagonals and totals unlinked | On "The diagonals are precisely the terms", each complete diagonal and its total take one shared colour (teal, purple, hero, ink). Squares past degree three stay dim. | 03_5 s2 |
| **C1** | Accepted: 02_6 now lists all three debts, in payment order. They are also on screen. | 02_6 |
| **C2** | Accepted. | 03_4, 03_5 |
| **C4 + P10** | Accepted with the reciprocal clause: new cue 03_7 names `e`. | 03_7 |
| **C5** | Accepted, screen only: "What must a be?". | 02_1 s4 |
| **C3** | Not accepted, as ordered. Colons stay and the lint warns about them. | |

## The pictures, cue by cue

These describe the source's intent, not anything seen.

| Cue | On screen |
|---|---|
| 01_5 | s0 the rule in symbols. s2 the existing promise. |
| 02_1 | s0 title and rule. s1 the polynomial. s2 boxes and headers, coefficients fall. s3 derivative row, four terms move left. s4 frame on the constant column, "What must a be?". Think gap: the answer stays hidden. |
| 02_2 | s0 `a = 1`, rises. s1 pulse `bx²` and `2b`. s2 `2b = 1`. s3 `b = 1/2`, rises. s4 pulse `3c`. s5 `3c = 1/2`, `c = 1/6`. |
| 02_3 | s0 `4d = 1/6`, `d = 1/24`. s1 derivative row leaves, chain and recurrence. s2 products gather. s3 `k!`. s4 factorials rise into the polynomial, table leaves, axes, and each partial sum lights its own term. s5 note, at degree three. |
| 02_4 | s0 `y'=`. s1 derivatives land one column left, a hole under the last term, caption. s2 two more terms fill the hole and reopen it, then dots. s3 dots pulse, "The infinite tail needs a proof". s4 the two obligations. |
| 02_5 | s0 the interval and a dot at `R`. s1 bars and ratios. s2 green ratios, full inequality. s3 inset, ceilings, sum, bound, read-outs. s4 shift left. s5 the sweep. |
| 02_6 | s0 `E =` row with dots. s1 `E' =` row by copies, frame walks, definition. s2 `y = E` solves. s3 three debts, each on its clause. s4 they light. |
| 03_2 | s0 row frame, then diagonal line. s1 onward as before. |
| 03_5 | s2 shared colours, then the series. Otherwise as before. |
| 03_7 | s0 an empty input line, −2 to 3. s1 `e = E(1) = 1 + 1 + 1/2 + 1/6 + ⋯ ≈ 2.71828`. s2 `E(2), E(3)` and the whole-number marks. s3 the root with `E>0` and the fractions; on "reciprocals" `E(−x)` and the negatives; on "continuity" a solid line. s4 `E(x) = e^x`. |

## Exact amended words, and hook impacts

Originals stay in `base/` and in their source folders. Rate words are unchanged.

**02_6 (C1).** "Call the sum E of x. The shifted coefficient rows now match all the way across: E prime equals E, and E of zero is one. We built a solution to our drawing rule. We still owe three explanations: why adding inputs multiplies outputs, why the stepped approximations reach this curve, and why it is the only solution with that starting value. The series gives us a way to answer all three."

**03_4 s2 (C2).** "The binomial theorem combines that diagonal into the sum x plus y, raised to the m, divided by m factorial."

**03_5 s2–s3 (C2).** "The diagonals are precisely the terms of E at the sum, x plus y. Therefore E of x times E of y equals E of the sum, x plus y."

**03_7, new (C4 with the reciprocal clause).** "Now we can name it. Call E of one by the letter e. The law says E of two is e times e, and E of three is e cubed. Roots give fractions, reciprocals give negatives, and continuity fills in the rest. That is when writing E of x as e to the x becomes a result, not a label."

Word count across the slices: 936 to 1,010, in 18 cues. Oliver needs 02_6, 03_4, 03_5 and 03_7.

| Change | Hook impact |
|---|---|
| C1, 02_6 | Still 5 sentences. It had no hooks. It now has s1–s4 and three clause hooks in s3. |
| C2, 03_4 and 03_5 | No new sentence ends, so hook indices are unchanged. Character shares shift slightly. |
| C4, 03_7 | New cue: hooks s1–s4 and two clause hooks in s3. Root sets the final id. |
| C5 | None. |
| Words unchanged, hooks added | 02_1 s1. 02_2 s1, s2, s4. 02_3 s3, s5. 02_4 s1, s2, s4 and a clause. 02_5 s2, s4. 03_2 a clause in s0. |

**Contextual takes.** Rewording a cue breaks kept contextual takes on its neighbours. If any exist, recheck 02_5 and 03_3 (next cue reworded), 03_6 (next cue is new), and the cues that follow 02_6 and 03_7 (previous words changed). I have not checked any take.

## Decisions for root

1. **The printed bound changed.** It was `0.08438` with ratio ¼ and hypothesis `N+2>R`. It is now `0.12657` (degree 4) and `0.00022` (degree 8), which is `2·t_{N+1}` with hypothesis `N+2≥2R`. The voice says "at most one half" and the ceiling halves, so the old number no longer matched the picture. This supersedes S2 and S3's printed value. Their five-place and round-up rules stay. Old rows `factorials.tail.order/ratio/bound` are gone.
2. **No dots at 02_1.** This differs from brief `:190` on purpose. The voice says "a polynomial" and then "A finite polynomial cannot finish the job", so the first picture is finite and 02_4 earns the dots. If you prefer the brief's form, add `DOTS` at `column(5)` at 02_1 s1. 02_4's terms 5 and 6 would then need a rethink.
3. **`until_clause` is new in these two scenes.** It replaces fixed waits inside four sentences (02_4 s4, 02_6 s3, 03_2 s0, 03_7 s3). It shares a sentence's real window by characters, so it is an estimate, not a heard boundary, and it ignores word marks. The logarithm chapters' `until_phrase` honours marks but needs `script.tsv` beside the crate, so that swap is yours at integration. If any of those four sentences is reworded, update its constant. `source_checks.py` compares them with the slices.
4. **Chapter lengths grow.** Scratch timing at factor 1.0: rate 105.28 s (unchanged), series 145.05 s (was 142.35), product 171.58 s (was 144.81).

## Left alone

- **N12:** still 0.13 s late (0.60 s at factor 0.8), left for Oliver's alignment.
- **N14:** the rate rows the scene never draws.
- **N15, in part:** `2b, 3c, 4d` now come from the model. The axis `"0"` and the product's `TEX` denominators are still typed. The repair also adds typed numbers inside formula text: `E(2)`, `E(3)`, `1+1+\frac12+\frac16`, `\tfrac12`, and the `\frac12`, `2R` of the inequalities. `math_exact` checks them, but AGENTS rule 2 may want them built from the model or exempted on purpose.
- **S9:** no "degree n" label. Terms lighting with their curves do that job, which is a judgement for picture review.
- **S10:** the chart keeps its hand-written `map`.
- **P6, P7 at the product:** left until after the first preview, as the review advised.
- **Static stretches of 6 s or more at factor 1.0:** the series scene has only its think gap (8.86 s). The rate scene has 13.41 s in 01_5, down from 21.14 s. The product scene's longest is 12.12 s at the end of 03_5. These are pre-existing, except the new 9.12 s from 03_6's last reveal to the start of 03_7's first change.
- **02_3 s5:** the note lands 0.43 s late at factor 0.8 only.

## Facts and oracles

883 rows, up from 835: factorials 667 (+41), product 33 (+7), rate 166, growth 6, limit 6, series 2, slide 3. New rows cover equations, values, running products, ratios, bars, ceilings, the bounds, `product.e` and the input line. Both new oracles use exact fractions. The bars come from running products of the ratios. `e` is proved between a partial sum and that sum plus its tail bound. The kit's real `oracle_diff` over all three oracles is empty, and the old bound planted back is caught. Combined oracle sha256: `a7f288a76bf1d1c6e7c5113484624ae891f6068c5466ba3e9425f0674d69c871`.

## Integration notes

- Each patch is relative to its stage, for `patch -p1` from a folder holding that stage's `base/` files. `candidate/` holds the same result.
- Wire `mod rate_model` (facts01's), `series_model`, `product_model`, then `series_model::append_facts` and `product_model::append_facts` in the registry.
- `factorials::TEXT` and `TEX` no longer exist, so drop any reference. Don't pass `product_grid::TEXT` as a glyph supplement unless you want it probed.
- `factorials.rs` needs `use fmn::library::TransformMatchingTex;`, already in the file. It and `product_grid.rs` import `explainer_kit::narration::{frames_covering, wait_frames}`.
- Scene source tests use `include_str!` on the scene's own file name, so keep those names or update them.
- 03_2's line declares `k.allow_crossing(along)`, because it is meant to run over the cells.
- **Root's:** prediction-map bindings (`01_1→01_2`, `02_1→02_2`, `03_2→03_3`), final cue ids, the facts-check route (N5), script merge, Oliver takes. The scenes keep `k.hold`, `k.mark("predict")` and `music_breath(4.0)` at all three.

## Bindings

| File | sha256 |
|---|---|
| `candidate/rate02/rate_law.rs` | `0d472d7098fb1534b8ed4e9fd8c61b1b41f02f32cc2926b45d2c356fc2ab9402` |
| `candidate/rate02/script-slice.tsv` | `1b5257beac7dd0e08a300c4625d8914f80ebc8af5bed718b7556587f6317e63e` |
| `candidate/series01/factorials.rs` | `9d465dd088a6e76b2d3c73ef78cd8c73d6116e71c745547a553477d3e249dc53` |
| `candidate/series01/series_model.rs` | `f5115801e279a5c6e2ed7fc6c3aea569f07ab6a85aae3bf1f36d4ba440e71203` |
| `candidate/series01/oracle.py` | `a9d2f041b43dcc68682a9b75f996ef4ba481cf9dac3c2f790fa3e2614e191318` |
| `candidate/series01/script-slice.tsv` | `a12508f016a06938625c9b23d33e26f3b7cb14a49fe4344833feeafe8d653851` |
| `candidate/product01/product_grid.rs` | `5626940d420c228c064b8c4728cdc69115fbe95c329c204d8d5fbd41aa40c3e4` |
| `candidate/product01/product_model.rs` | `b978fcd3ff5eafe34278faf97425a87f725dac7ad4236282dcfdd13c2e8d5a6c` |
| `candidate/product01/oracle.py` | `b132aa72b248f76f307700233babee70eef0741ebec4794e7335f1cd54678bd0` |
| `candidate/product01/script-slice.tsv` | `d94c33e0164724ab17a31739d95cc34b8f5a37ee0ec3590992b6a3bef4649bc1` |
| `patches/rate.patch` | `e4b105fc24410ece7078237edd5b43f97d4622d195254e24cc7d337a0c3038f0` |
| `patches/series.patch` | `c0cf5be40ef34959d36affcb05778445c4ebaf4e121be22a25278b5ca5df42b3` |
| `patches/product.patch` | `8f4f9e5f74a7f924bd4e0972cb6a4e15ef86960e05f6bf004cbea0a38d3fcd4e` |

`HASHES.sha256` covers the whole folder. Base files equal their sources, listed in `checks/bytes_check.py`.

## What ran

Run from the repo root: `python3 -I -B <this folder>/checks/run_checks.py <scratch dir>`. Outputs sit beside each script as `.out`.

| Check | Result |
|---|---|
| `bytes_check` | Base equals sources. All three patches replay forward and reverse at fuzz 0, with no offset. Frozen review intact. No machine paths. These bindings match. |
| `scene_typecheck` | `rustc -D warnings` and `clippy-driver -D warnings` pass in normal and test cfg, against existing a399d62 metadata and fresh kit metadata from current source. |
| `scene_controls` | Five planted defects each fail as they should. Each stage alone also type-checks. |
| `native_facts` | Real `Facts` and `oracle_diff`. 883 rows match. Model tests pass: series 9, product 5, rate 4. Whole registry crate 20. Clippy clean. |
| `math_exact` | 85 exact checks of the new claims, with edges: the hypothesis one degree below, the ties at `R/3`, `x = 0` in the sweep, why `E>0` is needed, why continuity is needed. Named-input checks are examples, not proofs. Same author as the repair. |
| `timing_replay` | My transcription, checked against each literal hook, at factors 0.8, 1.0, 1.25. Answers stay hidden through all three gaps, and the first factorial sign lands on its sentence at each factor. |
| `source_checks` | Scene tests compiled and run: rate 3, factorials 12, product 5. Removing a hook fails the order test. `rustfmt` clean. No digits in the words. Skill lint: 0 errors. |
| `text_fit` | Plain-text extents from the font tables are in frame, and neighbouring labels clear. TeX widths are my rough estimates only. |

## Not run, and so not known

- **No Cargo build, link or run of any scene.** Types are checked; behaviour is not. New runtime risks: one tracker driving 15 `always_redraw` mobjects, the swap to live bars with the ceilings re-added on top, a lagged group played through `rt(1.0)`, many `Transform`s between unlike TeX shapes (`1·2·3` to `3!` to a term), and `transform_from_copy` on bars.
- **No glyph probe.** `\xrightarrow{\div i}`, `\left(\right)`, `\tfrac`, `\text` and `≤`, `×` through fallback need it. `\binom` in `TEX[3]` is still unprobed.
- **No layout lint, no render, no contact or cue sheets.** No pixel has been judged, including all colours, the ×16 inset's readability, the solid line over the marks, and how legible the 03_5 colours are.
- **No Oliver takes, no listening, no fine sync, no word alignment.** All timing is scratch voice at 2.6 words per second, and clause hooks are character estimates.
- **No independent review of these repairs.** See the disclosure above.
- **No canonical edits, Beads, commits, synthesis or heavy jobs.** D–I are not touched.
