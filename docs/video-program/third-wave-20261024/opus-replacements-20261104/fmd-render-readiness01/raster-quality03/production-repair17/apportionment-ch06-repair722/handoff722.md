# Handoff 722: apportionment chapter 06 candidate set

Author: CC3 (WhiteSwan). Status: staged for root's review, then binding and build. No Cargo step has run, so the Rust is uncompiled and its tests have never executed. Nothing was rendered. Canonical files are untouched, and the 719 set is preserved unchanged (candidate `d6c0235a0df000603f30322dade099b240e2f3cc100813c235d3efac5315d64b`, handoff `5f8aa089634f206d063b03abbc3d42818f9bc852c5ccf22ee1a162fbec90e580`).

## Two choices to confirm

### 1. The year is bound and validated, but is not a printed `Facts` key

Printing it fails `facts-check`. I ran the real `tools/description_numbers.py` three ways, each with `--oracle` and the comma-free `--facts` its TSV header describes:

| Oracle output | Exit | Mismatch |
|---|---|---|
| canonical | 0 | none |
| this candidate | 0 | none |
| canonical plus `history.seaton.census_year` = 1880 | 1 | `description_numbers.tsv:5: 1880 ('After the 1880 census') is used by nothing` |

The film's TSV is `#! legacy`. The legacy branch at `tools/description_numbers.py:1021` credits any number equal to a fact and continues before `row["used"] += 1` at line 1050, and an unused row is an error at lines 1064-1065. So a printed 1880 makes the very row you asked to bind to count as stale.

What the candidate does instead, inside the named scope:
- `facts.rs` reads the year from the sourced TSV row. The row must be the only `year` row naming Seaton, must cite census PDF p. 13, and must state that census. The scene calls `crate::facts::seaton_census_year()` and contains no year string.
- `oracle.py` validates the same row and the spoken "After the eighteen eighty census" on every run, including the run `facts_check.py` makes, and raises on any disagreement. It prints nothing new.

The cost is that the year is outside the `facts-check` key comparison, and both sides read one row. That validates the row and the narration; it is not two independent derivations. A Rust test pins "1880" exactly.

If you prefer the repo's usual pin: add `f.pin("history.seaton.census_year", 1880, "1880", YOUNG_P13)?` after `range_high`, add `emit("history.seaton.census_year", 1880)` to the oracle, read it with `fact(..)` in the scene, drop the row reader, and delete TSV row 5. That needs a `description_numbers.tsv` edit outside this scope and removes the row.

### 2. "Reduced" means reduced by the towns' common factor

`step_fraction` divides POP and total by their common gcd (1000), so the labels stay "+4/39 turn", "+9/39 turn" and "+26/39 turn" with no literal 39. Lowest terms for each fraction separately would print "+4/39", "+3/13" and "+2/3". I did not do that because chapter 05 shows the /39 labels and 05_2 says "nine thirty ninths" and "twenty six thirty ninths", and you asked to preserve existing values. To change it, edit `step_fraction` and the expected labels in its test.

## Changes from 719

| Item | Where in the candidate | Change |
|---|---|---|
| House constants | `not_every_time.rs:35-36` | `HOUSE_NOW` = 16, `HOUSE_NEXT` = `HOUSE_NOW` + 1. These now feed `house_trackers`, `three_dials`, `ranking`, `share`, `run_trackers`' target, `hamilton` and the question. |
| Question | `not_every_time.rs:58` | `question()` is built from the constants and `NAMES[ASH]`. The text is unchanged from 719. |
| Step labels | `not_every_time.rs:38-55, 90-96` | `gcd`, `step_fraction` and `step_label`. The wedge length uses the same `step_fraction`, and each value equals 719's as an f64. The loop is now `.iter().enumerate()`, because with `POP[i]` in a helper the old range loop would trip `needless_range_loop`. |
| Year | `not_every_time.rs:196`, `facts.rs:23-54` | Read from the sourced row as above. |
| Oracle | `oracle.py:33, 35, 236-258, 557, 600` | `seaton_census_year()`, called on every run and reported under `--check`. |
| Storyboard | `storyboard.md:118-121` | Picture cells of 06_1 to 06_4 describe the new picture. |

The wedges stay as the supplied-input diagram. F3, F6, F7, F8 and F9 are still not implemented.

Tests added, none yet executed:
- `the_printed_steps_are_the_exact_shares`: exact labels, exact cross-multiplication against POP and total, numerators equal to the oracle-compared `speed39.*` facts, and the three numerators and denominator share no factor.
- `the_question_reads_the_house_constants`: exact text, Ash last at `HOUSE_NOW`, seats 2, 4, 11 at `HOUSE_NEXT`.
- `the_census_year_is_the_sourced_row`: "1880" from the real TSV, the spoken phrase in `SCRIPT`, and refusal of a missing row, a repeated row, a changed year and a changed page.

## Timing and cues

The scene has 41 timing calls in both files, in the same order with the same phrases, run times and lags, and 6 `say` calls for 06_1 to 06_6. Three calls differ in text only: `&[q]` became `&cleared` and `&tags` became `&arrivals` (both from 719), and the tracker target `17.0` became `HOUSE_NEXT as f64`, which is the same value. Four planted changes (added pause, changed run time, reworded hook, changed tracker target) were each reported. As in 719, the sixth item in the tags' lagged group shortens each tag's own fade slightly, while the group still ends at 1.5 s. Cue frames can only be confirmed by a build.

## Pins

| Canonical path under `explainer_corpus/apportionment/` | Before sha256 (`before-*`, read-only) | Candidate sha256 |
|---|---|---|
| `src/chapters/not_every_time.rs` | `d3e88f7bcad146b9078ddc7d84588f6569339cd59cd3b2c225aa963bfadf7c94` | `04499441ccde5be5f827b9768da26bbd0672f21e283728b298809f827d7225e9` |
| `src/facts.rs` | `b26bf1e409958a2ac9fb430f215cc19b4dbe6f3b350e0f28820a9056f1a18c8c` | `e9b86e26f7e285ddc397a8948462492c824e77a637a4904e501466a7ece78f1d` |
| `oracle.py` | `d9a5a6ab098cae541ad7b3ebd5aca90c0dfb43f34fe7f03ed7e91e94b2a9b803` | `8c1dd3af893059601f1c6a195a3292e7274d796edd4701adfefd6fb3d780c94f` |
| `docs/storyboard.md` | `564e86db6c12c75a651bfcb6912619e2b3219337442095b64b78f2b49e930176` | `17a46b19463c07dd4713527574bcb46d56e8dc4dba6b648cc1daac4817f8ffbd` |

| Diff in this directory | sha256 | Contents |
|---|---|---|
| `repair722.diff` | `feb84a349a20572fd2ae4e436f1b32fb1eee2fa20df17b5d3b19c102ef6da710` | canonical to candidate, all four files, repo-relative |
| `delta-from-719.diff` | `ada461e305a8fafbaff5af7ec0929c9565db5ce5ed3828a03c32efb34ad261b4` | the scene only, 719 to 722 |

From the repo root, `git apply --check` on `repair722.diff` exits 0. Patching a scratch copy of the four canonical files reproduced all four candidate hashes.

Inputs that must stay as they are: `script.tsv` `b9344e1123170e7e3898d8aea2123e0b659bcd571085008fc97894e7d69da624`, `docs/description_numbers.tsv` `f8e46262ea565e4f4c27554fde9b46fc88bcafefc5662a377b1d06cab31f9cc1`.

Binding: your build-host evidence covers only `not_every_time.rs`, as recorded in 719. For `facts.rs`, `oracle.py`, `storyboard.md` and `description_numbers.tsv` my only evidence is that `apportionment-export652.json` lists each at the hash above. I have not seen their build-host hashes.

## For binding

- `facts.rs` now includes `docs/description_numbers.tsv` at compile time, in non-test code (`facts.rs:23`), so that file must be in the build input set.
- `facts.rs` and `oracle.py` are shared by all 15 chapters, so their hashes change for every chapter's inputs. No printed value changes: the candidate oracle's stdout is byte-identical to canonical (443 lines), and the body of `facts()` is untouched.
- `oracle.py` now reads `docs/description_numbers.tsv` and `script.tsv` beside itself. `facts_check.py` runs it from the crate directory, and the paths are derived from `__file__`.

## Checks run

- `rustfmt 1.10.0-nightly --check`, edition 2024, max width 120: exit 0 on both Rust candidates and both Rust before copies. Parse and format only.
- The candidate oracle ran for real in a scratch tree with `--check`, exit 0. Six planted faults each exited 1: year in the number column, year in the reason, page, row removed, row repeated, narration year changed.
- Exact arithmetic mirror of the labels and of the wedge floats against 719.
- Storyboard: only lines 118-121 changed, each still has 5 cells, and ids, words, features and holds are identical. `check_video` compares only the words.
- `git status` shows canonical `explainer_corpus/apportionment` and `tools` unmodified.

## Not established

Not run: compile, Clippy, the Rust tests, `facts-check` as a whole, layout lint, render. The Rust logic was checked by reading it against its definitions and by the Python twin only. I did not re-read Young's text; the year's evidence is the existing TSV row and its citation. Nothing here is picture, listening, motion, AV or release evidence. The final review still needs Cedar's crossing of the top tick (0.015 gap, as in 719) and the other picture checks listed in `handoff719.md`.
