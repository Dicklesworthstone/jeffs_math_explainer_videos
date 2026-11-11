# F1 recheck: CC0 materializer impl02

CC2 (RubyRaven), 2026-11-11, about 00:30Z. This rechecks my stage-review01 F1 against CC0's Mail 25075. CSD was read-only.

**Verdict: F1 is fixed.** I found no new defect.

## What I read

From CC0's `final4k-impl02`, unchanged since my snapshot:

| File | sha256 |
|---|---|
| `cc0-native-materialize-repaired02.py` | `e5abe22e` |
| `handoff03.md` | `6dbccbdc` |
| `checks02.py` | `ae82b36c` |
| `checks02.out` | `45458664` |
| diff from impl01 | `91c48c0e` |
| diff from 991 | `dec85ebd` |

## Checks

1. **Text diff from impl01 (`85557673`).** Only the docstring changes, plus two statements: `part` becomes `prefix = 'stems/'+CHAPTER+'.music.f32.wav'`, and the comprehension uses `startswith(prefix)`. 991's 09 literal assertion is unchanged.

2. **The selection on actual stem names.** I cut both drafts' selection statements out by AST and ran them on a read-only copy of CSD's `score21-summary.json` (`7652466c`, equal on CSD):

   | Chapter | impl01 | impl02 |
   |---|---|---|
   | 01 | refuses | `01_the_bump.music.f32.wav` |
   | 08 | part0000 to 0002 | the same |
   | 09 | part0000 to 0002 | the same; still equals 991's literals |
   | 11 | part0000 and 0001 | the same |

   No `.parts.tsv` or `qa` file is taken. The 08, 09 and 11 sets equal the stem sets pinned in CC3's run helper.

3. **A real scored 01 cuelog.** I used `final01-terminal142/01_the_bump.cuelog.tsv`, from a render with score21 stems. It has one stem row: `score/stems/01_the_bump.music.f32.wav` at `bb98d112`, with 13 fields. On CSD that is both the file's sha and the summary digest.

   Replaying impl02's stem loop on it gives:
   - every stem row's name is in `stem_names`;
   - every digest equals the summary's;
   - `set(stems) == stem_names` with equal counts.

4. **CC0's checks02.** I reran it on a fresh copy. Its output and both regenerated diffs are byte-identical to CC0's: "all checks passed", including the 5 unresolved pins.

## Still standing from stage-review01

These are root's, and root has accepted them:

- **F3.** Write the complete `review_scope` before each final, with the exact runner admission names.
- **F4.** One runner, with both pins filled, staged before any final.
- **L1.** No `claimed()` keys in evidence.

`STAGED993` and everything for 11 remain open until the finals and the counting02 chain exist.

## Not claimed

No picture, motion, listening, fine-sync, AV or speed claim.

## Files

In `stage-review02`:

- `f1-recheck02.md` (this file)
- `f1_recheck01.py`, and `f1_recheck02.py`, the corrected AST match that produced the results
- `f1_recheck02.out.json`
- `csd-copy01/score21-summary.json`
- `rerun01/`
- `snapshot01/`
