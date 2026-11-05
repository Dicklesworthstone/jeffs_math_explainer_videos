# O3 fix: bind the phrase-mark plan to the prepare receipt (87do.63, Mail 4094)

LavenderPike, 2026-11-05. `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`.

**Root's request (Mail 4094):**
- a minimal hash binding of `phrase-marks-plan.json` in the prepare receipt, verified before `timings` reads the plan;
- built in this NEW folder only, preserving the sealed L1 fix and Bronze's base;
- tests: a positive real-shape test, a refusal for a mark moved within its sentence, and refusals for a missing or mismatched plan;
- no new framework.

**Status: staged, not installed.** The base of this delta is the L1 staged helper (`P/logarithm-stage09-mark-interval-fix01/staged/prepare_audio.py`, `2f080ab7…`, sealed in `60f5f63a…`). Nothing sealed was edited.

## The gap

`prepare` writes `phrase-marks-plan.json` next to `spans.json`. Before this delta, `prepare-receipt.json` hashed the spans but not the plan, and `timings` read the plan back unchecked.

With the L1 fix, `timings` refuses a plan mark that leaves its sentence. A mark moved within its own sentence was still written into `phrase-timings.json`, and so reached the kit (P04 shows the L1 helper doing this).

## The delta

`prepare_audio.patch` (+7/−3) applies to the L1 staged helper, giving `staged/prepare_audio.py` (`08a26927…`).

| Where | Change |
|---|---|
| `cmd_prepare` | the receipt gains `phrase_marks_plan_sha256`, the hash of the plan file it has just written (one line; the plan is written before the receipt) |
| `cmd_timings` | the plan is no longer read first. After the existing binding and `spans.json` checks, `timings` refuses a missing plan, then a plan whose hash is not the receipt's `phrase_marks_plan_sha256`, and only then reads it. Both refusals come before `verify` runs and before anything is written |

**What the binding covers.** The whole file. Any changed byte is refused, including a whitespace-only change (P05).

**Folders from earlier helpers.** A receipt written by an earlier helper has no `phrase_marks_plan_sha256`, so its folder is refused with "prepare again into a new folder". `prepare` and `timings` must come from the same new helper.

**The refusals:**

```
phrase-marks-plan.json is missing from the prepared folder
phrase-marks-plan.json is not the plan this prepare receipt binds: it changed since prepare, or the receipt was written
without the binding; prepare again into a new folder
```

## Evidence

`checks/outputs/plan_binding_checks.txt`: 6/6, byte-equal over two runs.

| Check | What it shows |
|---|---|
| P01 | Bronze's stage09 bundle (`2bb8e280…`, 14 files) and the L1 bundle (`60f5f63a…`, 7 files) verify; the L1 staged helper is `2f080ab7…` |
| P02 | The patch is exactly the diff L1 staged → staged. Applying the L1 patch and then this one to Bronze's `c6c854ce…` gives the staged file. Only `cmd_prepare` (+1/−1) and `cmd_timings` (+7/−2) change |
| P03 | **Positive, real shape.** The staged `cmd_prepare` runs on the real binding for chapter 08: fixture selection with 08_5 by its clarity path, recognizer values, and decode rows with exactly `cmd_decode`'s keys. The decoded WAVs are zero-filled stand-ins of the decoded length. It writes `phrase_marks_plan_sha256` equal to the plan file's hash. Prepare's own assemble command runs, using the pinned assembler copy with `--require-directions`. The staged `timings` then accepts the bound plan and writes 08_5's marks (267 at 22.42 s, 306 at 25.379999 s) |
| P04 | **Mark moved within its sentence.** 08_5's first mark +0.05 s in the plan stays inside sentence 4, so the L1 interval check passes it. The L1 helper writes it; the staged helper refuses before writing anything |
| P05 | **Missing or mismatched.** A missing plan is refused; the L1 helper raised `FileNotFoundError` instead. A receipt without the hash is refused. A plan with one added space and the same marks is refused. A mark moved out of its sentence with the receipt's hash rewritten to match is still refused by the L1 interval check, so the two guards are independent |
| P06 | **Bronze's sealed `run_checks.py`**, run unchanged through the sealed L1 wrapper with `HELPER` pointed at the staged file: 79/79. Its transcript equals his sealed `run_checks-01.txt` after the helper hash line. That covers his full-film prepare with answers then timings, chapters 01 and 04, and every plant, all now passing through the binding |

## For root, when integrating

1. **Build the next helper** from Bronze's `c6c854ce…` plus the L1 patch plus this patch, or copy `staged/prepare_audio.py`. Set `HERE` to the new folder, as the L1 HANDOFF says.
2. **Prepare and time with the same new helper.** A folder prepared by an earlier helper is refused by the new `timings`.
3. **My sealed L1 check M09** writes its prepare receipt by hand without the new key. It targets the L1 helper and stays valid there; against this helper its timings calls would be refused at the binding.

## Limits and what was not done

- No Rust compiled; no production decode, assembly, timing or selection; no recognition run; nothing heard.
- The fixture assembly is of zero-filled stand-ins in scratch.
- The in-process calls replace only `make_out`, so outputs land in scratch.
- `prepare-receipt.json` itself is not hashed by `timings`. Its hash is recorded in the written `phrase-timings.json` (`prepare_receipt_sha256`), unchanged from before.
- Not independent: I wrote L1 and this delta. Bronze is reviewing L1 and can review this delta.

## Files

| File | What it is |
|---|---|
| `HANDOFF.md` | This note |
| `prepare_audio.patch` | The delta, against the L1 staged helper `2f080ab7…` |
| `staged/prepare_audio.py` | L1 + this delta (`08a26927…`); `HERE` unchanged |
| `checks/plan_binding_checks.py` | P01–P06; read-only on the repository, writes only under a fresh scratch folder |
| `checks/outputs/plan_binding_checks.txt` | Its output at seal |

**Replay**, from the repository root:

```
python3 -I -B P/logarithm-stage09-plan-binding-fix01/checks/plan_binding_checks.py NEW_SCRATCH
```
