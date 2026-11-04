# Logarithm cue checker and storyboard repair

**Author:** RubyRaven, for BlueSalmon's Mail 1636 (Bead 87do.63).

**This is a validator repair, for root to review independently before any use.** I wrote the checker and its tests, so neither is independent evidence of the other. Nothing here was compiled, rendered, synthesized, measured or heard. No scene source, script, hold or canonical file changed. Narration stays ElevenLabs Oliver.

## Result

`docs/check_cues.py` now passes stage04 as it stands, retired files and test modules included, and it refuses more real defects than the old one did.

| Check | Result | Output |
|---|---|---|
| New checker on stage04's bytes | Exit 0: 80 cues, 364 sentences, 80 said | `checks/real_bytes.out` |
| Old checker on the same bytes | Exit 1, 27 failures, as in the final80 finding | `checks/real_bytes.out` |
| Planted defects in stage04's bytes | 39 plants, 0 wrong | `checks/real_bytes.out` |
| Fixture regression cases | 102 cases, 0 wrong, on Python 3.9.6 and 3.15.0 | `checks/regression-py39.out`, `checks/regression-py315.out` |
| Each rule switched off in turn | 28 of 28 noticed by the suite | `checks/suite_has_teeth.out` |
| Storyboard against script, source and the binder's parser | 0 problems | `checks/storyboard_check.out` |
| Patches applied and reversed on scratch copies | 0 problems | `checks/patch_roundtrip.out` |

## Patches

| Patch | sha256 | Base |
|---|---|---|
| `patches/01-check-cues.patch` | `e9befe3e92a46fb42964238306bd895cbb829c088ed5b40b13d914b0b6775422` | `docs/check_cues.py`, `cb053481…`. Stage04 and canonical are the same bytes. Adds `docs/test_check_cues.py` |
| `patches/02-storyboard.patch` | `52555b25c1ea19ecef6008c833837e599d98020266ea7eb21f62da4e2e7353d1` | Stage04's `docs/storyboard.md`, `b1f9f3dd…`. **Doesn't apply to canonical's storyboard, which differs** |

The patched files are in `explainer_corpus/logarithm_mysteries/docs/`. `base.sha256` pins the inputs and `bundle.sha256` pins every file here except itself.

## What the checker does now

The old one ran a regex over the whole text of every `*.rs` in `src/chapters/`. The new one splits Rust into tokens, so comments and strings can't look like calls. It reads only `mod.rs` and the modules `mod.rs` declares, with `#[cfg(test)]` and `#[test]` items removed.

| Mail 1636 asked for | How it's met |
|---|---|
| Check registered scene modules | The registry in `mod.rs` sets which modules run and their order. Other declared modules (`draw`, `thumbnail`, `mod`) are read too, and any hook in them is refused |
| Exclude cfg(test) code | Items are removed by token, including `draw.rs`'s test helper before its tests module, which a text split misses. A `mod tests` not marked `cfg(test)` stays scene code |
| Don't silently ignore real errors | A `say` or `until_sentence` it can't read is refused, not skipped. So is any cfg other than plain `cfg(test)`, `cfg!`, `#[path]`, `include!`, a test attribute on something that isn't an item, an unclosed string or comment, or a file that isn't UTF-8 |
| Nonempty registry | Refused if empty, missing, not a plain `vec![Chapter { .. }, ..]`, or if it runs a module twice |
| Missing registered file | Refused, as is a registered module with no declaration or a declared module with no file |
| Duplicate, missing, out-of-order cues | Refused and named separately, across chapters as well as within one. A cue said in the wrong chapter is refused too |
| Bad `until_sentence` | Unknown cue, not the current line, index out of range, or going backwards |
| No counterfeits | Comments, block comments, strings, raw strings, test items and retired files can't stand in for a `say` |

Retired files stay on disk and are named in the output as not compiled. In stage04 that's `bookends.rs`, `matrix_log.rs`, `product_law.rs`, `real_inverse.rs`, `winding.rs` and `doing_nothing.rs.orig`. If one is declared again, it gets read.

**The one computed hook.** `product_rule.rs:339`, `kit.until_sentence(stage, "08_4", k + 1)`, was invisible to the old regex. The new checker refuses computed indices unless they're in `REVIEWED_COMPUTED`, which has this one row. Its range is 1 to the length of the literal array `steps_src`, so 1 to 4 of the cue's 7 sentences, and both ends are checked. If the array, the fill loop, the single `push` or the hook's loop changes, the row stops passing until someone reads it again.

The output format is otherwise the same: first line, `FAIL` with indented failures or a last line of `OK`, exit 1 or 0. There are a few new informational lines.

## Did the gate weaken?

The matrix in `checks/real_bytes.out` compares the old checker on registered scene code, the only setting where it passes, with the new one on all of stage04. 3 of the 39 plants don't apply to that cut copy, and their old column shows `-`.

- **Every plant the old checker refuses as a defect, the new one refuses too.**
- **The new checker refuses 9 plants the old one passed:** a real `say` replaced by a comment, a block comment or a raw string; the computed hook pushed past its cue, its loop reversed, or its array grown past the cue; a `cfg(not(test))`; an empty registry; a registered module with no declaration.
- **The old checker refuses 3 plants the new one passes.** This is the one direction to read closely: a comment repeating a real `say`, a raw string holding hook text with the real `say` kept, and a test fixture naming a bad hook. None of the three is a compiled call. The 27 stage04 failures are the same kind of thing, 14 naming retired files and 13 from test strings.
- **The digit listing lost nothing.** The old checker's 182 entries are all among the new one's 187.

The suite's "84 wrong" against the old checker in `checks/suite_has_teeth.out` also counts differences in message wording, so it isn't a count of defects the old one missed. The matrix above is the fair comparison.

## Storyboard

- **`10_4` is in the authored reading table.** The row gives 197 characters and the 13.2 s `FINE_PRINT_S`, and calls the window conditional: music fills only `FINE_PRINT_S − read_for`, so it depends on the take and can be zero. I claim no length.
- **Question cue and question-as-last-sentence are now different things.** The prediction table has a new last column. 11 cues end on their question. `00_4` doesn't, because the wrong answer follows it, and `12_5` doesn't, because the hints follow it. The check compares every row with the script's sentences. **The wording of `00_4` and `12_5` is unchanged**, and the script isn't in either patch.
- **One table cell changed for accuracy.** `12_5`'s "(after three hints)" became "(three hints follow)", because the hints come after the question.

**If root adopts the storyboard and reruns the binder.** I ran the frozen binder's actual code on a scratch stage04 copy that differs only in the storyboard:

- 5 of its 8 outputs stay byte for byte the same, including `narration_units.json` and the `oliver12` map.
- 3 outputs change. `hold-audit.json` now lists `10_4` in the table and nothing missing. Its hash goes from `74af971e…` to `f76769e6…`, and the 15 disposition rows bind that hash. `inputs.sha256` changes on the storyboard's line.

The frozen final80 bundle stays valid against stage04, which I didn't touch.

## Limitations

The second, third and fourth are pinned as `KNOWN GAP` cases in the regression file, so they show up in its output. Branches are the exception: no case pins them.

- **It reads text.** It isn't a Rust parser and doesn't compile anything. The lexer is my own, tested on these fixtures and stage04's compiled chapter files.
- **File order stands in for run order.** A `say` in a function nothing calls counts as said. A `say` in a loop or a macro body counts once, however many times it runs. Branches aren't followed. Only a build and a real timeline can settle these.
- **Other cue helpers are counted, not checked.** In stage04: `until_phrase` 22, `until_share` 3, `phrase_time` 12, `phrase_seconds` 3, `sentence_time` 3, `sentence_starts` 11. The old checker didn't check them either. A bad cue in one still passes here.
- **Only `src/chapters/` is read.** A grep of `main.rs`, `model.rs`, `score.rs` and `facts.rs` found no hooks in stage04. A cue moved out of chapters would show as said 0 times, but a stray extra `say` wouldn't show at all.
- **The computed row is a person's judgment.** Its guards tie the range to the array and the loops, but they don't prove `steps` holds exactly what the fill loop pushes.
- **It can refuse valid code.** A registry built another way, a feature flag, or a custom test attribute all fail closed. A custom test attribute's contents count as scene code. Root would have to widen the checker on purpose.
- **The sentence rule is unchanged, not re-proved.** I compared it by reading `narration.rs:623`, and didn't run it against the kit.
- **The suite shows it can fail, not that it's complete.** Switching off 28 rules is my own list.
- **The `.orig` is only named.** `doing_nothing.rs.orig` still shouldn't be installed.
- **Frozen tools are unaffected but old.** `logarithm-final80-bound-plan01/checks/run_check_cues.py` runs stage04's own old checker on purpose, and would need the helper files in its cut copy to run this one.

Canonical's current 75-cue crate also passes the new checker on a scratch copy (exit 0, 75 cues, 364 sentences, 75 said). That's for information only, and no gate reads it.

## Run it

From the repo root, with `R` set to this folder and `EMPTY` to a new empty directory each time:

```
python3 -I -B $R/explainer_corpus/logarithm_mysteries/docs/test_check_cues.py $EMPTY
python3 -I -B $R/checks/real_bytes.py $EMPTY
python3 -I -B $R/checks/storyboard_check.py $EMPTY
python3 -I -B $R/checks/patch_roundtrip.py $EMPTY
python3 -I -B $R/checks/suite_has_teeth.py $EMPTY
shasum -a 256 -c $R/base.sha256
shasum -a 256 -c $R/bundle.sha256
```

`patch_roundtrip.py` needs its directory outside any git repository. `checks/make_patches.py` rewrites the patches from stage04 and the candidates. Once installed, the checker keeps its old command from the crate root: `python3 -I -B docs/check_cues.py`.

## Installing

Root's call, after review. Apply `01` where the checker's base bytes are in place. Apply `02` only on a storyboard with stage04's bytes, or redo its edits by hand on another base. Neither patch needs the other.
