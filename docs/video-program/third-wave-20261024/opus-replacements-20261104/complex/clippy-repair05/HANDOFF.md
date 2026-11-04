# complex_analysis_visual: strict-Clippy repair, staged

Grant: Agent Mail 1361. Evidence: `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-opus-rebuild-build04/command.log` (exit 101; fmt, 95 tests and build passed; Clippy failed).

**Status: uncompiled. No Cargo, Clippy, test run, render or synthesis happened.** Canonical, reading-repair02 and Beads were not touched. Only root's guarded build can show Clippy is clean.

Patch: `clippy-repair05.patch`, sha256 `e6d4405bf25b6979e06e811bf1c2dc0be68fdc6aba423648aca98376ebe3f245`. Six files, +47/−18, no `allow` or `expect`. No narration, cue, picture or fact changes.

## Baseline (canonical at 09:32Z, unchanged since before build04)

Full hashes are in `MANIFEST.json`. `base/` holds these bytes, `candidate/` the result.

| File under `explainer_corpus/complex_analysis_visual/src/` | base sha256 (first 16) | candidate |
|---|---|---|
| `model.rs` | `d481bc8f8b20a4f8` | `e32b72591da65c13` |
| `chapters/draw.rs` | `5963fc53c6ad8401` | `07965225f2e051ae` |
| `chapters/residues.rs` | `0c7e39a7c0bfd2d9` | `d56dec7933dbde71` |
| `chapters/real_integral.rs` | `afc7b069c011af6c` | `b7bc79e0bfab5fc3` |
| `chapters/counting.rs` | `5b32c4877cc21c61` | `57c5e5159cfb3943` |
| `chapters/one_point.rs` | `99e47ff96cbda6f4` | `76877afc1a75abb9` |

## The findings and the fixes

The log's counts reconcile: bin 9 = six dead items + three shared; test 8 = `leave`, `Quartic`, three constant assertions + the same three shared. Cargo collapses identical reports, which leaves 12 distinct findings (`leave` prints twice, 13 reports).

| # | Base line | Finding | Fix | Why this form |
|---|---|---|---|---|
| 1 | `residues.rs:8` | unused `TAU` (bin) | import it in `mod tests` | only the tests at `:333`, `:370` use it |
| 2 | `draw.rs:415` | `Plane::contains` unused (bin) | `#[cfg(test)]` | many chapter tests use it |
| 3 | `draw.rs:807` | `Walker::leave` unused (both) | **removed** | no caller in either target, so `cfg(test)` would still be dead |
| 4 | `real_integral.rs:66` | `antiderivative` unused (bin) | `#[cfg(test)]`; doc at `:40` drops the link to it | `the_antiderivative_shown_is_right` keeps testing it |
| 5 | `model.rs:86` | `C::sqrt` unused (bin) | `#[cfg(test)]` | its test stays |
| 6 | `model.rs:384` | `Func::Quartic` never constructed (both) | **removed**, with its 2 match arms | nothing builds it; `quartic_roots` stays live and tested |
| 7 | `counting.rs:846` | constant assertion | `const { assert!(COARSE_ROW_X >= -6.96) };` | now a compile-time check, the form already in apportionment, bakery_algorithm, crofton |
| 8 | `one_point.rs:804` | constant assertion | `const { assert!(MOSAIC_CORE < model::SQUEEZE_RADII[1]) };` | same |
| 9 | `residues.rs:358` | constant assertion | `const { assert!(BIG_R < 1.6) };` | same |
| 10 | `model.rs:147` | `*` inside `Div` | `Mul::mul(self, o.recip())` | same call, same bits |
| 11 | `model.rs:349` | excessive precision | `25.010_857_580_145_69` | same double, bits `403902c78ff7a3b2`; the oracle keeps its digits |
| 12 | `model.rs:649` | `!(reach < w.abs())` | `reach.partial_cmp(&w.abs()) != Some(Ordering::Less)` | NaN, ties and infinity all still refuse |

Two tests are added to `model.rs`, because nothing pinned this behaviour before:

- `a_tie_or_a_nan_never_certifies_a_winding`: clear of the reach passes; a tie, a NaN value, a NaN bound and an infinite bound each give `None`.
- `division_is_the_product_with_the_reciprocal`: bit-exact with `a * b.recip()`; a zero divisor stays non-finite and `finite` gives `None`.

No test or assertion is removed. Per file, tests go counting 9→9, one_point 7→7, draw 5→5, real_integral 6→6, residues 3→3, model 15→17.

## Every changed line, for the merge (base line numbers)

| File | Base line | Change |
|---|---|---|
| `model.rs` | after 11 | + `use std::cmp::Ordering;` |
| | after 85 | + `#[cfg(test)]` above `pub fn sqrt` |
| | 147 | `self * o.recip()` → comment + `Mul::mul(self, o.recip())` |
| | 349 | literal |
| | 383–384 | − doc and `Quartic,` |
| | 424, 463 | − `Func::Quartic` arms |
| | 649 | one line → comment + four formatted lines |
| | after 1012 | + the two tests, 28 lines |
| `draw.rs` | after 414 | + `#[cfg(test)]` above `pub fn contains` |
| | 805–810 | − blank, doc and `Walker::leave` |
| `real_integral.rs` | 40 | doc text |
| | after 65 | + `#[cfg(test)]` above `pub fn antiderivative` |
| `residues.rs` | 8 | drop `TAU` from the import |
| | after 323 | + `use std::f64::consts::TAU;` and a blank line |
| | 358 | const assertion |
| `counting.rs` | 846 | const assertion |
| `one_point.rs` | 804 | const assertion |

## Merging with reading-repair02

Rehearsed on scratch copies against their stage at 09:32Z (their patch sha256 `12236b014419cc071dc765d93898575d38d13ddcc4648a06576c09f00c9aa4fe`). **Their stage may still be moving, so re-run the rehearsal if it changes.**

- They stage `draw.rs`, `real_integral.rs`, `counting.rs`, `one_point.rs`, and not `model.rs` or `residues.rs`. No line changes in both.
- `git apply` refuses one file only: `real_integral.rs`. Their insertion before base line 64 sits in my hunk's context.
- `git merge-file` merges all six with 0 conflicts, and `#[cfg(test)]` lands directly above `pub fn antiderivative`. The merged crate passes `rustfmt --check`.
- Their added lines use none of `sqrt`, `antiderivative`, `Quartic`, `leave`, `TAU`. Their `.contains(` calls are all on text, none on a `Plane`.
- Their source-reading tests look at chapter cue bodies, `reading_window` and `precisely_over`, none of which I change.

**If canonical still has the six baseline hashes**, from the repository root:

```bash
git apply docs/video-program/third-wave-20261024/opus-replacements-20261104/complex/clippy-repair05/clippy-repair05.patch
```

**If reading-repair02 went in first** (the order I rehearsed), from the repository root:

```bash
STAGE=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex/clippy-repair05
for f in src/model.rs src/chapters/draw.rs src/chapters/residues.rs src/chapters/real_integral.rs src/chapters/counting.rs src/chapters/one_point.rs; do
  p=explainer_corpus/complex_analysis_visual/$f
  git merge-file "$p" "$STAGE/base/$p" "$STAGE/candidate/$p"
done
```

Done by hand, `real_integral.rs` is two lines: replace the doc line (base 40), and put `#[cfg(test)]` above `pub fn antiderivative`.

## Validation

The gate is root's, unchanged, as in `build01.sh`:

```bash
export CARGO_INCREMENTAL=0
M=explainer_corpus/complex_analysis_visual/Cargo.toml
cargo fmt --manifest-path $M --check
cargo test --locked --offline --manifest-path $M
cargo build --locked --offline --manifest-path $M
cargo clippy --locked --offline --manifest-path $M --all-targets --no-deps -- -D warnings
```

Then `<binary> facts` must equal `python3 -I -B explainer_corpus/complex_analysis_visual/oracle.py` at exactly 120 facts, and `<binary> chapters` must run.

Expected: 97 tests with this patch alone (95 + 2), plus the 6 `#[test]` lines their current patch adds if merged; 120 facts, unchanged; Clippy with no errors.

The source-level check here, run from the repository root with a new empty scratch folder:

```bash
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex/clippy-repair05/checks/clippy_repair_check.py <new-scratch-folder>
```

- `checks/check-run01.out.txt`: PASS, exit 0. Covers the 12 findings at their logged lines, the exact edit, no `allow`, the truth table on 100 pairs, the literal's bits, the const assertions' values, forward and reverse patch round trips, `git apply --check`, a whole-crate rustfmt parse and the merge rehearsal.
- `checks/check-negative-control.out.txt`: with candidate replaced by base, 21 checks fail, exit 1.

## Remaining risks

1. **Nothing compiled.** The two new tests and three `const` blocks have never been through rustc. The check script mirrors their logic in Python, and I read their signatures and types by hand; rustfmt only proves they parse.
2. **Clearing the lints is inferred.** It comes from the log, and for `Div` from my memory of Clippy's logic: `suspicious_arithmetic_impl` fires on a binary operator, and `Mul::mul(…)` is not one. If that lint still fires, send it back; I would not add an `allow`.
3. **New lints are possible** on the new lines, and Clippy reports only what it reached, though both targets did report here.
4. **Two items are removed outright**: `Walker::leave` and `Func::Quartic`. If later work wants either, it needs restoring. No staged or canonical code uses them today.
5. **Canonical or reading-repair02 may move.** Re-check the six hashes before `git apply`; use the merge route otherwise.
6. **No picture, listening, fine-sync or AV claim.** No drawing path or fact value changes by intent, and that too is unproven until built and rendered.
