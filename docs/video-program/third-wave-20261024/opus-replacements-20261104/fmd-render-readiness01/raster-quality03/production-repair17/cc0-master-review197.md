# cc0 master review 197: complex analysis assembly chain

Dispatch-0 (cc0), 2026-11-08 UTC. **Review only.** No helper, plan, permit, TODO or Bead was changed, and nothing was launched. This file is the only file written.

The start of the request was cut off. I took it to mean: review the deployed complex master chain in `production-repair17/` for wrong chapter bindings, missing repaired 08/09 permits, and audio, caption or schedule errors. The chain is `trj-master-after-imports184.py` → `trj-master-after-chapters184.py` → `trj-final-master184.py`, using `master-plan183/chapter-plan183.json`. Downstream of it are `trj-verify-after-master40.py`/`trj-final-verify19-02.py` and `trj-pictures-after-verify182.py`/`final-picture-plan182.json`. If a different chain was meant, nothing here applies to it.

## Result

**No confirmed actionable defect.** Every bound file I could read locally agrees with the rest of the chain. One dependency set can only be checked on hz2; it is listed at the end. If it is incomplete, the master will stop before it starts and produce nothing wrong.

## What I checked

**Chapter bindings.**

- All 15 plan rows (worker, job, inputs and permit) match the commands in the selected jobs' guard receipts. The held 08 job and the old 09 job are not in the plan.
- The plan's chapter order equals the build19 registry: `trj-master-after-chapters184.py:26-29`.
- The only non-standard job names accepted are the exact repaired 08 and 09 pairs, at `trj-master-after-chapters184.py:48-58` and `trj-final-master184.py:98-108`:
  - 09: `hz2-repaired-final158.py` with job `hz2-repaired-final-09-boundary158`.
  - 08: `csd-repaired-final170.py` with job `csd-repaired-final-08-loop-sums170`.
- The plan's `import_inputs` names match what the import helpers write:
  - `repaired09-return162.py:19` and `repaired08-return171.py:19`.
  - `trj-prepare-import161.py:15`. The corrected 07 relay keeps the name `import-hz2-final07-86-inputs.json`.
- Import receipts already finished with exit 0: 87, 124 and 97 (03:18Z, 03:21Z, 03:24Z).

**Repaired 08 and 09 permits.**

| Chapter | Permit | Binary | Timeline | Frames | Permit time | Render start |
|---|---|---|---|---|---|---|
| 08 | `repaired08-review181/final-render-permit181.json`, sha `dae3e84f…`, the same sha as the picture-plan source | `5a66979e…` | `338f16a8…` | 14,959 | 02:31:58Z | 02:32:34Z |
| 09 | `repaired09-review158/final-render-permit158.json`, sha `5e2c0ab2…` | `352aa5ff…` | `4cbb007a…` | 16,504 | 01:33:04Z | 01:34:47Z |

- Both permits list no unresolved defects, and both were written before their render started.
- Each permit is checked again three times:
  - by `check()` before the wait: `trj-master-after-chapters184.py:59`;
  - by `check()` after the wait: `:107`;
  - inside the master job: `trj-final-master184.py:132`, and `:141` asserts `summary['permit_sha256'] == sha(permit)`.
- All of 08's `check()` inputs reach TRJ:
  - `csd-launch-final170.py:25-45` flattens the timeline165 and preview166 manifests.
  - Those manifests already flatten the rebind165, build164 and timeline21 manifests and job folders: `two-label-after-build165.py:11-15,29-30` and `loop-label-after-timeline166.py:18-21`.
  - `repaired08-return171.py` exports those rows.

**Audio and captions.**

- `master` and `captions all` run with the TRJ chapter worker's narration for all 15 chapters: `trj-final-master184.py:160`, which takes `NARRATION` from `trj-final-chapter19-02.py:12`, the delivery02 folder.
- The 08 and 09 timelines point at the rebind165 and repair-delivery157 copies. Those copies are byte-identical; `hz2-repair-delivery157.py:24-26` checks that only `source` changes.
- Every line row is checked by content. `trj-final-verify19-02.py:31` asserts that `sha(Path(x[4])) == sha(N/(x[1]+'.wav'))`, so any repaired WAV that differed would stop the verifier.
- The repaired tables equal the timeline21 tables, with only the narration path changed: `csd-repaired-final170.py:31-33` and `hz2-repaired-final158.py:31-33`. Captions are checked exactly against the 105-cue `production-repair19` script, in script order: `trj-final-verify19-02.py:22,34-35`.
- The master's `exact_caption_check=False` at `trj-final-master184.py:187-192` is handled by that downstream check.
- The score stem rows are checked against the same score21 stems for every chapter (`verify_tables`). The replays must equal all 15 chapters: `trj-final-master184.py:169`. Verify requires `native_bit_identical_replays == 15`: `trj-verify-after-master40.py:52`.

**Frames and the picture plan.**

- The permit frames add up to 133,662: `trj-final-master184.py:156` and `trj-verify-after-master40.py:52`.
- `final-picture-plan182.json` starts are contiguous, with 08 at 52,106 and 09 at 67,065.
- The plan samples 08 and 09 from the timeline21 tables. Those are timing-identical to the repaired tables, so every sample frame is still valid.

**Schedule.** The binding deadline is the verify waiter's: `trj-verify-after-master40.py:36` waits 48 h.

- That waiter was confirmed live at 2026-11-07T20:17:37Z (`bead-verification-controller40.json`), so master01 must finish by about 2026-11-09T20:17Z.
- The slowest finished final render ran at 1.023 frames/s (03, on TRJ). At that rate, the last three renders end by about 06:40Z on Nov 8:

  | Chapter | Started | Frames | Ends by about |
  |---|---|---|---|
  | 08 | 02:32Z | 14,959 | 06:40Z |
  | 09 | 01:35Z | 16,504 | 06:05Z |
  | 14 | 01:11Z | 11,756 | 04:25Z |

- The imports take minutes; import 97 finished 5 minutes after its render.
- `master` and `captions all` are capped at 4 h each (`trj-final-master184.py:166`).
- The worst case still leaves more than 24 h before the deadline. The other waits are later or don't apply:
  - `trj-master-after-imports184.py:28`: 48 h from about 02:37Z on Nov 8.
  - `trj-pictures-after-verify182.py:31`: 48 h from about 02:34Z on Nov 8.
  - `trj-master-after-chapters184.py:67`: 24 h, but its wait loop starts only after every source receipt has finished.

## Not a confirmed defect: one check that needs hz2

`trj-master-after-chapters184.py:59` runs the 09 worker's `check()` on TRJ.

- That `check()` reads files that `hz2-repaired-final158.py:86-92` does not require to be in `repaired-final09-inputs158.json`.
- `repaired09-return162.py:44-56` exports only that manifest's rows plus the 158 job folder.

Some of these files are already on TRJ:

- The runtime-transfer47, encoder-stop03 and delivery49 receipts are in `final10-export87.json`.
- The timeline21 08/09 tables and receipt will arrive with export 171.

The rest are on TRJ only if inputs158 lists them. Each must be listed under its `/data/projects/jmev-stroke-probe-20261106/...` or `/data/projects/jmev-hz2-render-worker-20261107/...` path, not its resolved `work/frozen-runtime47` path:

- `D/jobs/csd-two-label-build68/resource-receipt.json` and `D/jobs/csd-two-label-build68/qualification.json`
- `D/jobs/csd-two-label-rebind69/resource-receipt.json`
- `D/jobs/csd-two-label-timeline69/resource-receipt.json`
- `D/jobs/csd-two-label-timeline69/timeline-output/08_loop_sums.{timeline,cues,cuelog}.tsv` (`hz2-repaired-final158.py:31-33` compares the 08 tables as well as the 09 tables)
- `H/jobs/hz2-repair-runtime-transfer156/resource-receipt.json` and `H/jobs/hz2-repair-delivery157/resource-receipt.json`
- the six evidence files in `H/qualification01/production19/repaired09-review158/` named by permit158 (`hz2-repaired-final158.py:43`)

If any is missing, the controller stops at line 59 before `trj-production19-master01` exists. Nothing wrong is produced, but the master then needs a new controller. Before 09's render finishes, someone with hz2 access can make one read-only check: list these paths, and confirm each appears as a row in `repaired-final09-inputs158.json`. The hz2 chapter 07/10/12/15 launchers bound their `check()` receipts under `D/` and `H/` paths (`final10-export87.json`), so the 158 launcher probably did the same. That manifest is not in this folder, so I could not confirm it.

## Limits

- Everything above was read from local copies in this folder. There was no SSH, and I read no remote file.
- `repaired-final09-inputs158.json` and the deployed copies on TRJ were not read.
- No master, picture, caption or audio output exists yet, so none was checked.
- None of this is acceptance of the pictures, the sound, the sync or the final AV.
