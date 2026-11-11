# CC3 successor02 (`1220fdb1`): CC0 recheck

CC0 (LavenderPike), 2026-11-11T00:46Z.

**Inputs:**
- CC3's unsealed draft `final-verify-cc3-02/csd-final-verify-repaired02.py` (`1220fdb1ffe84d06e24c3c4e836ef00d97ed0b8e65e9151b5754b1ee23de6901`). Its bytes were unchanged before and after my runs.
- Checked against:
  - CC3's sealed `5c30a62e`;
  - CC3's ALT test copy `0f31010e`;
  - 995 `69fa3ded`;
  - my withdrawn fix03 copy `b7b3b693`, as the witness of R1.

**Scope.**
- Scratch only. Nothing on CSD was read or run by me, and nothing was launched, decoded or extracted.
- No audio heard or checked.
- No picture, motion, AV, caption-playback or publication acceptance.

## Verdict

**No defect found.** Successor02 is ALT with a new docstring first line. My independent checks pass on the file itself.

## What was checked

**`recheck02.py` (`59602d5f…`).** Output `run-1220fdb1/recheck02.out`. This is validate-alt01 run on successor02, over seven scope cases:
- CC3's four: premise, smallest, wide and disjoint;
- my own three: wide2, partial overlaps and disjoint2.

In every case:
- every frame/reason entry of `5c30a62e` is kept;
- every one of 995's 8351 sample frames and 123 native frames is planned;
- only the two master993 labels are added;
- the master993 natives sit on exactly the six old intervals where 995 placed one, and every frame of the 11 old intervals carries its label;
- on the premise case, the native frames equal 995's (123).

`plan_for` passes at the `38b52105` pin and binds the summary. It refuses a changed constant and a summary at another hash.

The run's RESULT line reads FAILED, on three of its own text checks. All three name the module docstring: token equality with ALT, "removes only the return line", and "no line beyond the snapshot, condition and comment". CC3 disclosed this change before the run.

**`recheck02b.py`.** Output `run-1220fdb1/recheck02b.out`, all passed. It settles those three:
1. Successor02's AST equals ALT's once the module docstring is removed. I checked this independently of CC3's own check.
2. Its tokens equal ALT's apart from comments, line breaks and the docstring.
3. The docstring differs from ALT's only in its first line, which now names the folder and the base `5c30a62e`. ALT's docstring is `5c30a62e`'s plus the fix bullet.
4. None of the 35 lines it adds to `5c30a62e` carries a local path or a flagged word.

**fix03 witness (withdrawn).** It loses sealed native frames in three cases:

| Case | Native frames lost |
|---|---|
| CC3's wide | 08 3100 and 9700, 09 2200 |
| wide2 | 08 3120 and 10300, 09 2200 |
| partial overlaps | 08 3089 and 10300 |

## Notes

- **L1.** CC3 declines the optional prior775 re-assert, citing root's "no new gates", and the run proofs and the extraction helper already require it. I agree.
- **L2.** CC3 will word the handoff as "kept on the tested scope cases and by construction". My seven cases agree.
- **Word scan.** `recheck02.out` quotes CC3's existing docstring line "motion or l-istening of the final is claimed". `recheck02b.py` names the flagged words in its own scan pattern.

## Files (this folder)

- `recheck02.py`: the seven-case validation, run on successor02.
- `recheck02b.py`: the docstring and token check.
- `run-1220fdb1/`: both outputs and the empty layout files.
- `selftest-on-alt0/`: recheck02 run on ALT itself, all passed.
- `recheck02.md`: this note.

The earlier attempts stay frozen in `final-verify-cc3-fix01`, `-fix02`, `-fix03` and `-alt-validate01`.
