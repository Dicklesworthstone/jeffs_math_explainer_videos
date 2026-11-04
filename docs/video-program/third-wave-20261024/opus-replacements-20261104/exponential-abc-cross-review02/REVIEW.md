# Exponential sections A–C: independent source and patch review

**Reviewer:** LavenderPike, for BlueSalmon's Mail 1897, bead 87do.62.

**Authorship:** I wrote none of rate01, rate02, facts01, series01, product01 or WhiteSwan's patches, and I had no part in them before Mail 1897. I read WhiteSwan's `REVIEW.md` and its check outputs first, as instructed, so this review is independent in code and route but not blind to its conclusions. It is the independent review of rate02 that WhiteSwan's R0 asked for.

**Scope:** sections A–C only. Sections D–I of the full scope stay open. No pictures or Oliver takes exist for these cues, so this review makes no picture, listening or fine-sync claim. All timings are the Kit's scratch-voice arithmetic.

## Verdict

| Candidate | Integration verdict | Condition |
|---|---|---|
| `exponential-expanded-rate02/rate_law.rs`, `script-slice.tsv` | **Integrate as source** | No blocking defect. Take the model from facts01, not rate02. Medium N9 and Low N12 can follow. |
| `exponential-rate-facts01/` | **Integrate as source** | Type-checks alone under `-D warnings`, so it can land first. |
| WhiteSwan's series patch | **Apply whole** | Clearly better than the base and it passes every check here. **The chapter still does not teach its own title:** N1, N2, N3 and S4/S8 stay open. |
| WhiteSwan's product patch | **Apply whole** | Sound. Medium follow-ups N6, N7, P6, P7. The cue naming `e` (P10) is still missing. |

**I found no mathematical error** in the 17 cues, the printed formulas or the printed numbers.

**The scenes now have type-check evidence.** The three scenes and their models type-check against the pinned fmn and the Kit's current source, and pass Clippy under `-D warnings` in normal and test cfg. Four planted defects each failed as they should, so this check can fail. It was metadata only, so nothing ran.

**Before any Oliver synthesis, settle these words:** the "dots" sentence (N2), and C1, C2, C3 and C4 below. After takes exist, each of these costs a retake.

## Defects new in this review

Paths: `F` = `exponential-expanded-review01/candidate/series01/factorials.rs`; `P` = `.../candidate/product01/product_grid.rs`; `R` = `exponential-expanded-rate02/rate_law.rs`. `s3` means sentence index 3 by the Kit's splitter. Brief lines refer to `docs/video-program/research-20261016/evidence/exponential-logarithm-sister-videos-20261101.md`.

| # | Severity | Where | Defect |
|---|---|---|---|
| N1 | High, teaching | `F:147–158` | "Those denominators are factorials" has no picture, and no factorial sign reaches the screen for another 42 s |
| N2 | High, teaching and words | `F:82–88`, `F:203–205` | The polynomial and any ellipsis are never drawn, yet the voice says "But dots are not a proof" |
| N3 | High, teaching | `F:130–141` | Coefficient matching is labelled, not shown: nothing links the cells being equated |
| N4 | High at integration | `tools/narration_delivery.py:316–318`, `tools/final_check.py:416–447` | Three prediction holds must be declared in the map; WhiteSwan's list omits this and its patches add two of the marks |
| N5 | Medium at integration | `tools/facts_check.py:111`, root `Cargo.toml:15` | `./x exponential_magic facts-check` cannot run as the crate is configured |
| N6 | Medium, teaching | `P:215–217`, `P:196` | At the proof's central sentence, "The diagonals are precisely the terms", nothing links the grid's diagonals to the totals |
| N7 | Medium, teaching | `P:155–157` | The switch from rows to total degree plays over 5.21 s of unchanged picture, 12.50 s counting what comes before it |
| N8 | Medium, notation | `F:41`, `F:144` | `a_0=1` shares the screen with the table's `a = 1`, whose `a` is really `a_1` |
| N9 | Medium, teaching | `R:273–282`, `F:82–84` | The equation `y'=y` is never written until `E'=E` in 02_6; 21.14 s static around 01_5 |
| N10 | Medium, on-screen obligation | `F:238` | The derivative series, an on-screen obligation in the brief, is voice-only |
| N11 | Low | `F:42`, `F:218` | `≤ 1/2` appears 5.66 s before "Eventually" |
| N12 | Low, timing | `R:270` | The only late hook in the three scenes: 01_4 s4 is 0.13 s late (0.60 s at factor 0.8) |
| N13 | Low | `F:16–18`, `F:46` | The patch orphans `x²`, `x³`, `x⁴` in `TEXT`, against its own comment |
| N14 | Low | `rate_model.rs:45`, `:57–58` (facts01) | 54 of the 166 rate rows are strings the scene never draws, and the doc comment says otherwise |
| N15 | Low, AGENTS rule 2 | `F:111`, `F:162`, `R:83`, `P:16–18` | More typed numbers on screen than R1 lists |

### Detail on the High findings

**N1.** The chapter is titled "Where the factorials come from". At 02_3 s3 the scene has no hook: it goes from `until_sentence(…, 2)` at `:147` to `until_sentence(…, 4)` at `:157`, and then wipes the table.
- The sentence window is 2.24 s (1.80 at factor 0.8, 2.81 at 1.25).
- The screen shows only the values `b = 1/2`, `c = 1/6`, `d = 1/24`, from `coefficient(n)`.
- The first `!` drawn is inside `TEX[2]`, the tail bound, at 02_5 s3 (`F:222`), 41.75 s later.

Repair: at s3 reveal `\frac1{0!},\ \frac1{1!},\ \frac1{2!},\ \frac1{3!},\ \frac1{4!}`, or the products `1\cdot2\cdot3\cdot4` that show why. 2.24 s is too short if it wipes at s4, so leave it out of `table` and let it stand under the chart at about y −2.75 until 02_4 begins. The facts `factorials.denominator.n` already exist.

**N2.** The brief opens this picture with "Begin with `1 + ax + bx² + cx³ + …`" (`:190`). `source_checks` finds no polynomial and no `cdots`, `ldots` or `…` anywhere in the scene. Three things follow:
- 02_1 s0–s1 ("a function whose derivative equals itself", "Try a polynomial") show the title alone for 6.57 s.
- The letters `a b c d` are never defined. "b times x squared" has to be worked out from column headers.
- 02_4 s3 says "But dots are not a proof" over a chart. WhiteSwan's S4 notes the table is gone; this sentence has no referent even if the table returns, because the table has no dots either.

Repair: s0 reveals `y'=y,\quad y(0)=1` at about (3.6, 2.25), the same equation as N9. s1 reveals `1+ax+bx^2+cx^3+dx^4` above the headers at about y 2.7. At 02_4 s2 ("With infinitely many terms") reveal `1+x+\frac{x^2}{2!}+\frac{x^3}{3!}+\cdots`. On the chart, about (2.2, −1.6) is free, between the axis and the degree-0 line at y −1.2. The alternative is to change the word "dots", which must happen before synthesis.

**N3.** 02_2 is the step the whole table exists for, and root's Mail 1827 asks whether visuals demonstrate rather than label. There is no `animate()` in the cue. After `a = 1` lands the picture does not change for 10.82 s, while the voice says "two b times x" and "must match the x term above". The labels also sit under the function row's letters, so `a = 1` appears under `2b`, not under the constant column where it is read off. The equations `2b = 1` and `3c = 1/2` are said in words and never printed.

Repair: at 02_2 s0, s2 and s5, light the column being matched, both rows together, and print its equation in that column at y 0.05: `a = 1`, `2b = 1`, `3c = 1/2`, then `4d = 1/6` at 02_3 s2. Build the right sides from `coefficient(n-1)`. Then solve each in place with a `Transform`: `b = 1/2` at s3, `c = 1/6` later in s5, `d = 1/24` in 02_3 s2. Each value then sits under the derivative-row term it came from. At about 1.0 scene units wide these fit the 1.1 column pitch, tightly. The think-gap test's `coefficient(n)` needle moves with this code.

**N4.** `kit.mark(…, "predict")` makes the native captions hold the cue's last line to the next cue's start (`captions.rs:524–536`). `narration_delivery.py` honours that only for a pair declared in the direction map's `visual_holds` with `kind: prediction`, and `final_check.py` reports a defect otherwise ("captions outside their audio"). WhiteSwan's rate01 review flagged `01_1→01_2`. S6 and P3 add `02_1→02_2` and `03_2→03_3`, and neither is in its "Integration work that remains".

Repair: declare all three, with final ids, each exactly once. None of the three can be a sounding coherent seam (`narration.rs:1209–1216` requires zero inserted samples), so the direction plan must split takes at them.

**N5.** `facts_check.py:111` runs `re.search(r'\[\[bin\]\]…', manifest).group(1)`. This crate's manifest has no `[[bin]]`, so that raises before any build. The crate also has its own `[workspace]` and is excluded from the root workspace, so `-p exponential_magic` from the root would not resolve either. `complex_analysis_visual` and `logarithm_mysteries` have the same shape, so root may already have another route. If so, use it: this crate's `facts` output against `python3 -I -B oracle.py`. I ran only the regex, not the command. `checks/native_facts.out` covers the values in the meantime.

### Repairs for the Medium and Low findings

- **N6.** Every total is green and only degree 3 is lit (from `P:196`). The one change at s2 is `TEX[8]` appearing in the work column, so nothing ties a diagonal to its total. At s2, colour each complete diagonal and its total with one shared colour, four colours in all. A 0.3 s parallel change fits the 3.75 s window.
- **N7.** During 03_2 s0, briefly light one row, then the first diagonal, so the change of reading is seen before degree zero lands at s1.
- **N8.** Rename the recurrence, for example `u_{k+1}=\frac{u_k}{k+1},\quad u_0=1`. No test names this string.
- **N9.** The rule exists on screen only in words (`rate = current height`, `R:111`). At 01_5 s0 reveal `y'=y,\quad y(0)=1` at about (2.75, −3.2), below the right panel. It carries straight into 02_1.
- **N10.** At 02_5 s4 reveal `\frac{k\,x^{k-1}}{k!}=\frac{x^{k-1}}{(k-1)!}` at about y −1.5, between the read-out and the uniform-convergence card. It proves more than the voice says: the derivative series is the same series re-indexed (`math_exact` checks this through degree 30).
- **N11.** Reveal in two steps, as P9 already does for positivity: `\frac{|x|}{k+1}\leq\frac{R}{k+1}` at s1, the full `TEX[1]` at s2.
- **N12.** Shorten `stage.wait(1.0)` on the first refinement pass to 0.5 s. Recheck on Oliver's alignment.
- **N13.** The glyph probe captures each chapter's actual strings (`glyphs.rs:66–104`) and treats `glyph_text` as a supplement (`:105–116`). So rate_law's missing `TEXT` is no coverage gap, and the "register glyph strings" step isn't needed for coverage. Don't pass this `TEXT` as a supplement, or the probe will print Plex superscripts the film never draws. Removing the entries shifts indices the tests pin.
- **N14.** The read-out and `last_step` rows for n = 8 and 16 are harmless. They include the tie `rate.8.1.right_readout` (1.125, printed 1.12 by both sides). Drop them, or correct the comment. The scene also passes a literal `1.0` at `R:118`, `R:178` where the facts use `START`.
- **N15.** Beyond R1's rate literals: `"2b"`, `"3c"`, `"4d"` (`F:111`), the axis `"0"` (`F:162`), and denominators in `TEX[0..2]` of the product scene. All are correct (`math_exact`). AGENTS rule 2 still calls a typed number a defect, so build them from the model or exempt them on purpose.

## WhiteSwan's findings, tested

| Finding | Status now |
|---|---|
| S2, 11 differing rows | **Confirmed in real Rust:** the same 11 keys on the base, through the real `Facts`. The candidate has none. |
| P1, `choose` | **Confirmed by compiler:** the base fails `-D warnings` with "function `choose` is never used". The candidate passes. |
| R3, cue ids | **Confirmed:** 9 of 17 ids collide (`01_1–3`, `02_1–2`, `03_1–4`), and `check_script_owners` would see two owners for prefix `01`. The glyph probe runs that check as well. |
| "166 of 166", "626/626", "26/26" | **Confirmed natively.** One registry of 835 rows, all three models; the Kit's own `oracle_diff` is empty. |
| "By reading, I found no compile error" | **Now type-checked.** Passes. |
| Source-order tests "replayed by Python" | **Now compiled and run:** 7 of 7 pass, and 12 of 12 model and facts tests. |
| "Sentence indices match the Kit's splitter" | **Confirmed** with `sentences()` cut verbatim from `narration.rs:623`. Colons do not split. |
| "Answers stay hidden" | **Confirmed by reading and replay.** Each answer waits 4.90 s after the voice ends. |
| Totals 141.75 s, 144.21 s | **Agree to 0.60 s** (142.35, 144.81). That difference is the chapter's opening breath in `say`, which their replay omits. Rate, which their replay did not cover, is 105.28 s (86.21 at 0.8, 129.13 at 1.25). |
| "All checked plain text fits" | **Stands.** `candidate_checks` did measure the patched strings. TeX widths are still estimates, theirs and mine. |
| S4 table return, S8 tail bars | **Agree, both High and still open.** N1–N3 touch the same table, so repair them together. S8 is a stated brief obligation (`:217`). |
| S9, degree labels | **Agree, open.** Five curves with no degree shown. One "degree n" label that morphs with each curve would do. |
| P6, enlarged diagonal | **Agree:** decide on the first preview. |
| P7, `TransformMatchingTex` | **Native question answered.** The 03_3 and 03_6 morphs type-check and pass Clippy in that form, with `use fmn::library::TransformMatchingTex;` because it is not in the prelude (`checks/p7_variant.py`). Each morph's target becomes the live handle, as in `the_turned_tile.rs:248–252`, `:487`. Whether the pairing reads well is a picture question. |
| P10, naming `e` | **Agree, and it is required:** the brief asks for it (`:245`), and section D's voice relies on the name (`:259`). |

### Script proposals

| # | Verdict | Note |
|---|---|---|
| C1 | **Accept** | 01_5 owes the stepped limit and uniqueness. 02_6 lists uniqueness and multiplication, so the stepped limit silently drops out. Later sections pay all three: C multiplication, D the limit (brief `:249`), E uniqueness (`:283`). D and E both lean on the series, so "answer all three" stays true. |
| C2 | **Accept** | Cheap with no takes. The voice should not depend on the formula being read at that moment. |
| C3 | **Optional, and it moves hooks** | A full stop at 03_2's colon moves hooks s3, s4, s5 to s4, s5, s6. At 03_3's colon it moves s3 to s4. 02_6 has no hooks. No test catches this, because the source-order tests pin only order, and neither cue's hooks are in them. |
| C4 | **Accept, with one clause** | "Roots give fractions" is sound because 03_6 has already proved positivity. Continuity from positive rationals never reaches negative inputs, though, so add the reciprocal: 03_6 already shows `E(x)E(-x)=1`. |
| C5 | **Accept** | Low stakes, and it reads better once N2 prints the letter `a`. |

### WhiteSwan's decisions for root

1. **Apply both patches whole.** A model-only subset keeps the wrong beats and the printed 0.08437.
2. **Accept C1, C2, C4 and C5.** C3 only with the hook moves above.
3. **Build S4 and S8 before picture review,** together with N1–N3. Leave P6 and P7 until after it.
4. **Rate02's independent review is this one.**

## Integration list: additions to WhiteSwan's

In order:

1. **Settle words and final ids first:** N2, C1–C5, R3. Ids sit in every `say`, every hook and the test needles.
2. **Facts:** add `mod series_model;` and `mod product_model;`, and call both `append_facts` between `candidate/src/facts.rs:35` and `:36`, before `self_check`. Tested as one registry: 835 unique keys.
3. **Oracle:** plain concatenation of the three scripts works: 835 rows, stdout identical to running them apart, sha256 `4969329af38f70378a27b4b3ed31f386542d6eb07b8b7da2c20522c674e8fe99`. The product oracle's JSON goes to stderr, which `facts_check.py` ignores (`:119–127`).
4. **Land each scene with its model and facts call.** Without the call, `append_facts` fails dead-code under `-D warnings`. Facts01 alone is fine.
5. **Declare the three holds and split takes there** (N4).
6. **Decide the facts-check route** (N5).
7. **Don't pass `factorials::TEXT` as a glyph supplement** (N13).
8. **Refresh stale text:** R2, S10, and the "not compiled" module headers once a native build exists.

`rustfmt --check` passes on all six staged scene and model files. `facts.rs` reports 4 hunks in both the candidate and canonical, so that is pre-existing and facts01 adds none.

## Native API

| Level | What |
|---|---|
| **Type-checked at the pin** | `Transform::new`, `Stage::play`/`prepare`/`play_prepared`/`wait`, `polyline(…).style(…).build()`, `Line::new(…).style(…).build()?`, `Circle::new()…moved_to(…)`, `animate().set_anim_args(…)?.set_stroke(…)?`, `set_color_with_opacity`, `fade_in`, `fade_out`, `Timed::rt`, `Kit: Clone`, `kit.say`, `until_sentence`, `hold`, `mark`, `music_breath`, `text`, `tex`, `finish` |
| **Read in the pinned source** | Plain `Transform` leaves the handle in the scene, so repeated morphs on one handle and never-rooted targets are right (`fmn-anim/src/transform.rs:579–588`). I found no mob faded out twice. `SetColor` also writes the fill lane (`fmn-mobject/src/animate.rs:763–768`). That is harmless on `R:242`'s straight segments, which enclose no area, so the Kit's open-fill lint will not flag them (`crates/explainer_kit/src/layout.rs:87`, `:686`). The patch correctly avoids it on polylines (`F:183–190`). `color_diagonal` recolours TeX, whose `ink_style` has zero stroke width (`palette.rs:75–80`). |
| **Present in the macro table, not run** | `\binom`, `\mathbb`, `\cdots`, `\vdots`, `\leq`, `\geq`, `\infty`, `\quad`, `\in` all exist in `fmd-math/src/commands.rs` at the locked 126a530. No TeX was typeset. |
| **Thin against the brief** | No trackers, `Frame2` or `Axes` in any scene, against "Use native FrankenManim trackers, curves, TeX and transformations" (brief `:166–167`). Rate's discrete steps justify it there. `Frame2` could replace the hand-rolled `map` at `F:159`, as S10 says. Low. |

## What holds

- **Maths** (`checks/math_exact.out`, exact fractions):
  - Rate: each refinement equals the coarser drawing until its first step ends, then lies strictly above it. The largest gap shrinks from 0.1244 to 0.0721, and every drawing stays under `e^t`.
  - Coefficients are `1/k!` through degree 30.
  - The tail: 27/320, true tail 0.083252 < 0.084375 < the printed 0.08438. Both hypotheses are needed: `≤1/2` is exact at k+1 = 2R, and without N+2 > R the bound goes negative. The formula held on 400 random pairs.
  - The chart stays inside its axes.
  - The grid: 16 denominators by hand, complete through degree 3 and incomplete at 4 (3 of 5).
  - The identity through m = 24, the three morph forms equal, and the multiplication law, non-vanishing and positivity. Positivity is a property of the sum: at x = −5 the partial sums still change sign (S₄ = 13.71, S₅ = −12.33).
- **Facts match the drawing.** Rate and series scenes and facts call the same functions: `proportional_steps`, `constant_steps`, `sum`, `plot_x`, `coefficient`. Product's facts and its `cell_tex` and `total_tex` compute the same `factorial` expressions separately, and a model test pins six printed strings. Each oracle takes a separate route: closed form against the Euler loop, exact fractions against the float recurrence, binomial against `i!j!`.
- **Bytes.** All 21 of WhiteSwan's bindings and 27 sums verify. Rate02 equals the rate01 review's candidate. Base equals the originals. Both patches replay forward and back at fuzz 0 with no offset and leave the script slices alone. Canonical still matches facts01's `before_sha256`. No slice contains a digit.
- **Timing.** 46 of the 47 sentence hooks are on time at factors 0.8, 1.0 and 1.25. N12 is the one exception.
- **Rate teaching, by source reading.** The two-panel design earns its comparison: same start, shared first step, then read-outs that differ. The reveals land on their sentences, and the think gap is authored (`R:136–138`).

## Errors in my own checks

All are in the checks, not the candidates:
- `math_exact` first asserted that a finer drawing is strictly above at every t>0. That is false: they share the first step. I corrected it.
- The first scene type-check failed twice because of my harness: Cargo's environment variables, then a search path for my scratch `explainer_score`.
- A crude probe for `!` matched Rust macros, so I replaced it with an index check.
- The first P7 variant failed on the missing import, which is the finding above.
- I suspected the 85 s chapter limit (AGENTS rule 8 superseded it) and stale text-fit coverage (`candidate_checks` does cover it). Neither is a finding.

## What was not run

- **No Cargo, no linking, no scene execution.** The scene check is `rustc --emit=metadata` against existing `target/debug/deps` metadata for fmn, whose dep-info names the a399d62 checkout, plus Kit and score metadata emitted fresh into scratch. Scenes can still fail at run time, for example on a TeX parse.
- **Not exercised:** glyph probe, layout lint, oracle through `uv`, `./x facts-check`, Clippy with the canonical chapters in the same crate, renders. I did not re-run WhiteSwan's script lint.
- **Mail:** I read 1827 and 1897. A search on the bead returned no others, so I have not read the bodies of 1831 or 1889.
- **No pictures exist.** Layout judgements and repair coordinates come from source coordinates and estimated widths, and need the first preview.
- **No narration.** ElevenLabs Oliver remains the narrator. No take exists for these lines, and I did no synthesis, take selection or listening. Every window here is scratch timing and will move.
- **Full scope:** sections D–I, the introductory ruler and later applications are untouched by this review.

## Contents

| Path | What it is |
|---|---|
| `REVIEW.md` | This file |
| `checks/run_checks.py` | Runs all eight checks; `python3 -I -B <folder>/checks/run_checks.py SCRATCH_DIR` from the repo root |
| `checks/bytes_check` | Bindings, sums, patch replay, canonical bases |
| `checks/native_facts` | Real `Facts`, all oracles, model tests, Clippy on models |
| `checks/scene_typecheck`, `scene_controls` | Scene type-check, negative controls, base and stage-alone variants |
| `checks/p7_variant` | `TransformMatchingTex` type-check |
| `checks/math_exact` | Independent exact arithmetic |
| `checks/timing_replay` | All three scenes on the real splitter |
| `checks/source_checks` | Scene tests run for real, rustfmt, ids, oracle merge, screen-versus-voice probes |
| `HASHES.sha256` | Every file here, relative to this folder |

No patch is staged here: Mail 1897 asked for defects and a verdict. Nothing outside this folder was changed.
