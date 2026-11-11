# CC3's 993/994 successors: independent integration check (CC0, review01)

CC0 (LavenderPike), 2026-11-11T00:25Z.

**Read:**
- CC3's `combined-master-repaired01.py` (`17388c9d…`) and `csd-native-repair-stage01.py` (`432301fa…`), with their handoff and diffs. Root's preserved copies are byte-identical to them.
- Their dependencies: CC3's run helper (`6dceac47…`) and final extraction helper (`5fa242b8…`); my materializer impl02 (`e5abe22e…`) and gate draft (`2a855393…`); master993 (`1955ca32…`) and 994 (`f0641032…`); root's chain launchers and the scope02 controller.

**Scope:**
- Text reads plus one local static script of my own (`cross01.py`, output `cross01.out`).
- Nothing was run against CSD records, and nothing was launched, built, rendered or decoded. No production file was touched.
- No audio heard or checked.
- No picture, motion, AV, caption-playback or publication acceptance.

## Verdict

**No concrete integration fault.** cross01 passes all 43 checks, and the read-through below found nothing that would refuse a correct chain or admit a wrong one. The notes in the last section are operational points for root, not defects.

## 1. The original gates are kept

**master successor against 993 (by AST, top level):**
- Changed: the docstring, `VERSION`, `JOB`, `INPUTS`, `chapter_gate`, `gates`, `launch`, `master` and the main guard.
- Added: `REPAIRED`, `REPAIRED_KINDS`, `BASE_MANIFEST(_SHA)`, `REPAIRED_GATE_NAME/SHA` and `unresolved`.
- Nothing removed. Every other function is identical, including `film()`, `schedule()`, `preflight()`, `replays_of()`, the retained, combined and native521 branches, and `claimed()`.
- CC3's `master-vs-993.diff` equals a fresh diff of the two files (+56 −20).

**What each change keeps from 993:**

| Function | 993 | Successor |
|---|---|---|
| `chapter_gate` | 09 must be `native844` | each repaired chapter must be its `REPAIRED` kind |
| `gates()` | exactly one boundary row | exactly 4 repaired-kind rows, and no `boundary313` or `native844` row |
| `gates()` | base 625, other rows equal | base `8da77ed7`, its 11 other rows equal |
| `launch()` | waits on the `native844` stage receipt | waits on the stage receipts of both kinds |
| `master()` | boundary replay from the 09 row | boundary replay from the 09 row, now `native-combined01`; it reads `staged[-1]`, `binary_sha256` and `capture_check_before_master` |

The gate draft still returns all three of those fields for 09, and `staged` stays 5 tables plus the capture, as `master()` unpacks it.

**stage successor against 994:**
- 994's `main()` body became `stage()`, and `compose()` was added. The `alias_target`, `add` and `success` functions are unchanged.
- Each 994 assert is kept or generalized:
  - `>= 814` became "holds every preview extraction frame". For 09 this is stronger, since the preview extraction already holds prior775 and the plan.
  - `cd1900aa` became the chapter's scene sha.
  - The 16504-frame checks became the chapter's frame count.
  - The rglob no-symlink rule now covers the chain's delivery narration, build, final and final extraction.
- CC3's `stage-vs-994.diff` differs from Python's difflib only in where it aligns one blank line; the counts agree (+128 −32).

## 2. Interfaces between the files (cross01)

**One table of values.** For each of the four chapters, the run helper, materializer, gate and master agree on 17 fields:
- kind, frames, scene file and sha, staged capture, preview PCM, normalized table;
- binary name, tests, delivery record, comparison record, source folder;
- build, delivery, timeline, preview and pictures jobs.

The scene path the run proofs bind equals the materializer's.

**Names:**
- Final jobs are `csd-native-repair-final01-<ch>`, admissions are `…-picture-admission.json`, and extraction jobs are `csd-native-repair-pictures01-<ch>`.
- `RUNNER` is the same in all three CC3 helpers, and the stage's `PICTURES` is the extraction helper's own `HELPER`.
- The base manifest `8da77ed7` is pinned identically in the stage, materializer, gate and master. Its path is the one master993's summary records.
- **Guard inputs:** the stage's five chain inputs names per kind equal the `INPUTS` constants of root's actual launchers:

| Chain | Launchers |
|---|---|
| combined01 | build launch, after-build, after-delivery, after-timeline, scope02 controller |
| counting02 | the same five counting02 launchers |

  The run helper's `pictures_inputs` agrees with them.

**Calls:**
- All 22 `base.<name>` the gate reads are defined in the master successor.
- The stage's worker argv is the 14 items, in the order the materializer and gate assert.
- Stage and compose read only `materialized.json` keys the materializer writes (`chapter`, `original_manifest_sha256`, `unchanged_other_manifest_rows`).
- The materializer's row keys are exactly the gate's set.
- The job regex is the same in all three.
- The materializer's and gate's required prerequisites are among the stage's seven, and the gate's extraction receipts for preview and final are among them.

**Root's scope02 08 extraction (+37 frames) goes through the whole chain:**
1. The run helper's `proofs()` requires only `plan ∪ prior <= selected` (run:212), so the extra frames are accepted.
2. The final extraction keeps every admitted preview frame (`retained()`, then `choose()`).
3. The stage, materializer and gate each require final ⊇ preview.

**CC2 F3 is enforced by the code.** The final's inputs bind the admission. The stage adds those rows, then calls `add(run.admission(ch))`. `add()` refuses on "input drift", so an admission edited after its final launched stops the stage. Afterwards the materializer and gate re-hash against the stage's inputs.

## 3. Notes for root (operational, not defects)

**N1. Interpreter (behaviour kept from 992/994/968).** The gate hard-codes `'/usr/bin/python3'` as the first item of both the stage command and the final command. The stage, final launcher and run helper all use `sys.executable`. So the final launcher, the after-final controller and the stage must each be started as literally `/usr/bin/python3`. Started any other way, everything runs and only the master check refuses, after all four stages. CC3's note 5 says this for after-final only.

**N2. Order for filling the pins.** Each step depends on the one before:
1. Both `binary_sha256` values in the run helper (CC2 S1).
2. Then the run helper's sha, which is `RUNNER_SHA` in the stage and `FINAL_RUNNER_SHA` in the gate.
3. The materializer's `STAGE_VERSION` and `FINAL_JOBS`.
4. Then its sha, which is `WORKER_SHA` in both the stage and the gate.
5. The gate's remaining pins: `BINARY_SHA`, which equals the run helper's values; `STAGE_VERSION` and `FINAL_JOBS`, which equal the materializer's; `ADMISSIONS`; `MIN_PREVIEW`.
6. Then the gate's sha, which is `REPAIRED_GATE_SHA` in the master.

Any mismatch refuses (fail closed), but only late.

**N3. `MIN_PREVIEW['08_loop_sums']`** in my gate should be the actual scope02 08 extraction size. The predicted size is 484; read the actual value from its `pictures.json`.

**N4. Still to verify on CSD (CC3's open point 4):** that the combined01 and counting02 build job folders contain no symlinks, since the stage's rglob rule now covers them.

**N5. Top-level manifest keys.** compose copies 8da77ed7's top-level keys (`draft534`, `draft578`, `draft625`) unchanged, as 991 and the materializer do. The master reads only `chapters`.

## Files (this folder)

- `cross01.py` and `cross01.out`: my independent cross-file check.
- `review01.md`: this file.
