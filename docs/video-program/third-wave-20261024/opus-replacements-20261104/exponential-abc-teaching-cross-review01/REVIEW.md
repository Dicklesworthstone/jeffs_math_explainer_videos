# Exponential A–C teaching repair: cross-review

**Reviewer:** WhiteSwan, for BlueSalmon Mail 2019 (`.62`). **Under review:** LavenderPike's frozen `exponential-abc-teaching-repair01/`, `REPAIR.md` first, then its three patches.

**Evidence:** I read the source and ran the checks in `checks/`. Rust ran only as cut-out functions and a facts harness (neither links fmn), and as metadata type-checks. No Cargo, codegen for the scenes, render, TTS, Beads or commit. No pictures or Oliver takes exist for these lines, so I saw and heard nothing. **This gives no native, picture or listening acceptance.** All timing for these lines is scratch-voice arithmetic.

## Verdict

**The mathematics holds, and the patches reproduce their candidates. The repair is fit to integrate, with one correction before fine sync.** `until_clause` can never read word marks, so seven reveals could not be moved by fine sync. `patches/phrase-timing.patch` and `candidate/phrase.rs` replace it with the corpus `until_phrase`. With no marks, the timing is identical at every call site, so no length or cue map changes.

| # | Severity | Finding | Action |
|---|---|---|---|
| F1 | Medium, before fine sync | `until_clause` ignores word marks and hides script drift | Apply the staged patch |
| F2 | Low, script | "Now we can name it" points at the function, but a number gets named | Optional one-line patch, before any 03_7 take |
| F3 | Instructions | The wiring notes leave out the oracle join and scene rows, and "registry" is ambiguous | Use the exact lines below |
| T1, T2 | Optional | `u` has no row label; `4d = 1/6` is on screen under a second | One line each, your call once you have pictures |

## My prior involvement

This is not independent of the whole lineage. Read these results with that in mind:

- **`rate02` is my own earlier repair.** `rate.patch` sits on top of it. Only its five added lines are new to me.
- **I drafted cue 03_7.** It is C4 in my `exponential-expanded-review01/REVIEW.md`. F2 corrects my own sentence. The reciprocal clause is LavenderPike's.
- **My earlier findings fed this repair,** S2, C1 and C4 for example. `series_model.rs` cites my S2.
- **Part of the bound's lineage is mine.** The `RATIO_ALL` line, `(k+1≥2R)` included, is identical to one in my earlier `series-repair.patch`, and the old hypothesis `N+2>R` was the one my review analysed.
- **New to me:** the factor-2 total with `N+2≥2R`, the recurrence chain, the 03_7 picture, `until_clause` and the rest of the repair's `series.patch` and `product.patch`.
- **`phrase.rs`, both patches here and the check scripts are my own work,** so they need a reader other than me.

## What holds

**Tail bound.** With `t_k = R^k/k!` and `R = 3/2`, in exact fractions (`checks/math_exact.py`):

| Degree | `2·t_{N+1}` | Printed | True tail at `R` | Rounded up by |
|---|---|---|---|---|
| 4 | 0.1265625 | 0.12657 | 0.08325157 | under 1e-5 |
| 8 | 0.00021188 | 0.00022 | 0.00012430 | under 1e-5 |

- After the first omitted term, each ratio is at most `R/(N+2)`, so `N+2≥2R` makes each at most ½. A sweep of 160 allowed cases found no failure.
- The hypothesis does real work. Without it the claim is false in 68 of the other swept cases, starting at `R = 3/2, N = 0`, where `2·t_1 = 3` is under the true tail of about 3.48.
- Pairing the factor 2 with the old hypothesis `N+2>R` is false in 36 cases, so old and new can't be mixed. The old 0.08438 was true on its own ratio-¼ argument.
- `tail_bound`, `tail_label` and `halving_from`, cut verbatim and run, refuse `N = 0`, switch over at `N+2 = 2R` for `R = 3`, and print both figures. Ratio row and ceilings match to print precision. The first ratio at most ½ is index 3.
- "The same argument works for the derivative series" is sound: the shift is exact, and the same bound holds one index down.

**Recurrence.** Coefficient matching alone gives `1/k!` through `k = 12`. The chain's divisors are 1, 2, 3, 4, each the next whole number. The spoken list, coefficient row and running products all agree. The general reason `(k+1)u_{k+1} = u_k` is never displayed, but 02_1 voices it, and `SHIFTED` shows its closed form at 02_5. So the reasoning has no gap. `0!` is never voiced, but the row visibly turns `1` into `0!`; I'd leave that. The `indicate(…, None)` pulses here all land on text, so the flat-colour problem from algebra 11's filled tokens doesn't arise (source reading).

**03_7 logic.** In exact interval arithmetic (the logic, not the screen):

- `E(2) = e·e` and `E(3) = e³`.
- `E(1/2)² = e`, and `-E(1/2)` squares to `e` too, so "E > 0" on the screen line is needed.
- `E(q)⁴ = e^(4q)` at all 21 quarter points, negatives included.
- The reciprocal clause is required: without it the marks stop at 0, and continuity alone never reaches a negative input. The brief (`docs/video-program/research-20261016/evidence/exponential-logarithm-sister-videos-20261101.md:245`) lacks this step, so the candidate is right to differ.
- The three clouds mark each quarter point from −2 to 3 exactly once.
- For irrational `x`, "result, not a label" means agreement with the only continuous extension, which is what the brief means too. No change needed.

**Patches.** All three go from `base/` to every candidate file at fuzz 0. The combined oracle reproduces `a7f288a7…`.

**TeX and glyphs, from source only.** `xrightarrow`, `div`, `tfrac`, `left`, `text`, `cdots` and `binom` are all in `fmd-math/src/commands.rs` at the locked `126a530`. N13's claim holds as far as I can read: all scene text goes through the Kit calls that record glyph sources. The glyph probe itself is still yours to run.

## F1: `until_clause` cannot reach fine sync

**Defect.** Five call sites use it, covering seven reveals: 02_4 (1), 02_6 (3), 03_2 (1), 03_7 (2). It takes a fixed share of a hard-coded sentence and never reads `phrase_marks`. So `phrase-timings.json`, the Kit's route for fine sync, can't move these reveals. The route is live: 12 such files exist under `narration_lab`.

**How much it matters.** I measured a character share against the word marks on one other Oliver film, golay (recognition estimates, not heard boundaries):

| Words | n | Median error | p95 | Max | Mean, signed |
|---|---|---|---|---|---|
| Any inside a sentence | 584 | 0.23 s | 0.58 s | 0.79 s | +0.22 s (share is late) |
| Right after a comma or colon | 57 | 0.18 s | 0.48 s | 0.65 s | +0.14 s |

In algebra 10 a phrase-rule share put two boxes 1.43 s and 1.01 s after the recognised words (`algebra10-repair03-picture-review01/REVIEW.md:128`). So fine sync can matter here, and these reveals need a way to receive it.

**Drift is also silent.** This is `checks/phrase_timing.py`, sections B and C:

| Change | `until_clause` | `until_phrase` |
|---|---|---|
| A word mark exists | Stays on its share (planted mark: 0.70 s off) | Takes the mark |
| "reciprocals" is reworded away | Returns 11.58 s from the stale constant | Error naming the phrase |
| A colon in 02_6 becomes a full stop | Fixed `s3` lands 4.17 s early | Follows the text |
| The unmerged canonical 03_2 is a different line | Returns 2.56 s | Error |

Its four sentence constants copy `script.tsv`, and only the bundle's `source_checks`, outside the crate, compares them. The patched tests bind each phrase to the real script inside the crate. A missing clause also panics today rather than returning an error, and there are two copies of the helper.

**Correction.**

- `candidate/phrase.rs` holds the corpus logic once for this crate, plus five unit tests. All three functions match `logarithm_mysteries/src/chapters/unreachable.rs`, whitespace and trailing commas aside.
- `patches/phrase-timing.patch` removes `until_clause`, the four constants and the unused imports, and swaps the five call sites and the test needles. It keeps `DEBTS`.
- Apply it with `patch -p1` from a folder holding `series01/` and `product01/`, after Lavender's patches.

**Proof.**

- With no marks, both rules gave the same time at all seven reveals, on scratch timing and on invented uneven onsets: largest difference 0.00e+00 s. Every phrase occurs once in its line, and "collect them" doesn't match "collecting the".
- `phrase_seconds` and its tests ran as real Rust, and a mutated mark lookup fails them.
- The corrected crate type-checks under `rustc -D warnings`, plain `rustc` in test cfg and `clippy-driver -D warnings` in both cfgs (`dead_code` allowed in test cfg). It fails without `mod phrase;`.

**Limits.**

- `phrase_time` and `until_phrase` are type-checked only.
- The script must be merged first. The two new binding tests fail until it is, which is intended.
- A cue id that was never said waits for nothing, as `until_sentence` does. It is not an error.
- The corpus cue tools know the name `until_phrase`. `check_cues.py` and `bind_final80.py` aren't wired to this crate, and the latter's regex wants `&kit, stage`.

## F2: the first sentence of 03_7

"Now we can name it" follows "Those familiar properties now follow from what we built." So "it" reads as the function, which already has a name, and the next sentence names a number. The brief says "Name `e=E(1)`".

`patches/03_7-wording.patch`: **"Now we can name a number. Call E of one by the small letter e."**

- Still five sentences, so every `until_sentence` index stands, and both phrases still resolve.
- The line grows from 23.48 s to 24.25 s at scratch rate.
- The script lint on that slice reports 0 errors. 03_7's only warning is its sentence count, which is the same as before.
- "Small letter" is there because E and e sound alike. I can't judge that without hearing it, so take only the first change if you prefer.

## F3: wiring, in exact lines

`REPAIR.md` says to wire the models and call the exporters "in the registry". Use these lines; `checks/patch_check.py` type-checks them in this shape.

1. **`src/main.rs`,** alongside facts01's `mod rate_model;`: `mod phrase;`, `mod product_model;`, `mod series_model;`. `phrase.rs` belongs in `src/`, because it includes `../script.tsv`.
2. **`src/facts.rs`: "the registry" means `facts()`, not `chapters::registry()`.** After `crate::rate_model::append_facts(&mut facts)?;` and before `facts.self_check()?;`:
   ```rust
   crate::series_model::append_facts(&mut facts)?;
   crate::product_model::append_facts(&mut facts)?;
   ```
   `native_facts` called these from its harness after `facts()` returned, so this form hadn't been run. It now prints 883 rows, all equal to the oracle. Without the lines it prints 183.
3. **`oracle.py`: the notes don't mention merging.** `facts_check.py` runs one `oracle.py`. A single file holding facts01's, then series01's, then product01's oracle reproduces `a7f288a7…` on stdout, 883 rows, using only the standard library. Their summaries go to stderr, which `facts_check.py` doesn't compare. The route itself (N5) is still yours.
4. **`src/chapters/mod.rs`: the notes never name the scenes.** Add `mod factorials;`, `mod product_grid;`, `mod rate_law;` and a `Chapter` row for each. Keys and intensity are yours:
   ```rust
   Chapter { key: "…", make: |kit| Box::new(rate_law::RateLaw { kit }), intensity: 0.4 },
   Chapter { key: "…", make: |kit| Box::new(factorials::Factorials { kit }), intensity: 0.4 },
   Chapter { key: "…", make: |kit| Box::new(product_grid::ProductGrid { kit }), intensity: 0.4 },
   ```
5. **Don't rerun the bundle's `timing_replay` or `source_checks` on patched files.** They look for `until_clause` and the constants. Their no-mark timing carries over by section A of `phrase_timing.py`.

## T1, T2: optional, for the recurrence

Neither is a defect I can prove without pictures. Both lines type-check.

- **T1, `u` has no label.** It is voiced nowhere in 02 or 03, and its row has no label, though the rows either side are "function" and "denominator". Before `reveal_all(stage, &chain, 0.4)?;` in `factorials_appear`:
  `chain.push(k.text(stage, "coefficient", 23.0, Palette::dim(), v(LABEL_X, CHAIN_Y))?);`
- **T2, the last match goes by fast.** `4d = 1/6` fades in for 0.4 s and straight into a 0.5 s crossfade. `d = 1/24` then stands about 0.6 s before clearing at 1.52 s (scratch). 02_2's third step has a `stage.wait(0.5)?` here. After `let fourth = print_equation(stage, k, table, 4)?;` add the same. It ends at 1.4 s at scratch rate; a first sentence shorter than that would make the clear slightly late.

## Bindings

| Path | sha256 |
|---|---|
| `docs/video-program/third-wave-20261024/opus-replacements-20261104/exponential-abc-teaching-repair01/REPAIR.md` | `5ac054edc9a4d863fe4cac4ea88aeedc7384d8f63e2b864fca6f47a342687822` |
| `docs/video-program/third-wave-20261024/opus-replacements-20261104/exponential-abc-teaching-repair01/HASHES.sha256` | `0b7717acf0b9408e1fa1c5092226a73a786f416943e5a2025b2a9684dba351eb` |
| `docs/video-program/third-wave-20261024/opus-replacements-20261104/exponential-abc-teaching-repair01/candidate/series01/factorials.rs` | `9d465dd088a6e76b2d3c73ef78cd8c73d6116e71c745547a553477d3e249dc53` |
| `docs/video-program/third-wave-20261024/opus-replacements-20261104/exponential-abc-teaching-repair01/candidate/product01/product_grid.rs` | `5626940d420c228c064b8c4728cdc69115fbe95c329c204d8d5fbd41aa40c3e4` |
| `docs/video-program/third-wave-20261024/opus-replacements-20261104/exponential-rate-facts01/candidate/src/facts.rs` | `afff33feaf5ef85cfb68f9ad9c041e630e14e4cef52a636d420f6e33e2bd7c14` |
| `docs/video-program/third-wave-20261024/opus-replacements-20261104/exponential-rate-facts01/candidate/oracle.py` | `f042a80e4c5689b4a33f746fd599078da4d1fb7f0add929fb3cf183f180574fa` |
| `docs/video-program/third-wave-20261024/opus-replacements-20261104/exponential-abc-teaching-cross-review01/candidate/phrase.rs` | `7ab27a09372cc42e03dad92a8d938d6470d44bcb9f964a26a2a64ca7177245ae` |
| `docs/video-program/third-wave-20261024/opus-replacements-20261104/exponential-abc-teaching-cross-review01/patches/phrase-timing.patch` | `49b7367ab896306d059b09dd8a62e76a251433b95d56ad891dcebb08c85733ca` |
| `docs/video-program/third-wave-20261024/opus-replacements-20261104/exponential-abc-teaching-cross-review01/patches/03_7-wording.patch` | `42fc61ab6f278490129d15e744f7ca4b7d354e0f17b78b1a030c606bcdc2f485` |

Results after the patches, which aren't stored as files here:

| File | sha256 |
|---|---|
| `factorials.rs` | `ca48d460151a418ab13ca8b9995ac22f068ea0b975021f81a2ed93e580cdc4b0` |
| `product_grid.rs` | `d06c45e6a7d9387df4b378b74df1af2833e469e35dfbe94f70e205312d07e0b3` |
| `product01/script-slice.tsv` | `00650efcacdba6661c928344be0c8d8435b1705bbf1987288784016af36b6da1` |

## What ran and what didn't

| Ran | Result |
|---|---|
| `checks/math_exact.py` | 73 PASS, 0 FAIL |
| `checks/phrase_timing.py` | 37 PASS, 0 FAIL |
| `checks/patch_check.py`: patch round trips, rustfmt, 13 string walks by Python mirror, metadata type-check and Clippy against existing rmeta (fmn pin a399d626), facts through `facts()` | 51 PASS, 0 FAIL |
| `checks/make_patch.py` | Generates both patches from the frozen candidates |

**Not run:** Cargo build or tests; the scene tests in Rust; `Script::parse` against a merged script; a narrator run; the glyph probe; `facts_check.py`; any render, take, listening or picture check. The Kit metadata came from the current dirty tree, and it's a type-check only. I read the bundle's `.out` files and didn't rerun its checks, except that `patch_check` repeats the type-check and facts comparison in my own form.

The bundle and canonical source were not edited. Root integrates and owns the heavy lane.
