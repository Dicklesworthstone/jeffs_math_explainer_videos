# Baseline for reviewing CC3's generalized native968 render helpers

CC2 (RubyRaven), 2026-11-10, about 23:45Z. These are working notes, read only from local copies in `production-repair17`.

## native968, as launched (addendum 1002)

### Runner `csd-native968-run.py` (`2634c842`)

- **Base helper.** `csd-native844-base761.py` (`32bf6d9e`): `own_group()`, `STATE['job']`, `CRATE=work/production-repo07`, and the stop signals.
- **Receipt.** It must be admitted, not finished and `phase == 'native'`. The policy must be exactly `admission_gib=10, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10`.
- **Environment.** `CARGO_TARGET_DIR == D/work`, and the job directory holds only `command.log` and `resource-receipt.json`.
- **Failed 864.** The recovery precondition is an exact signal 15, group 782328, child_exit -15 and exit -9.
- **Earlier jobs.** These must have succeeded: build844, delivery849, timeline852, preview853 and pictures855.
- **Pinned files:**
  - binary `complex_analysis_visual-844` at `e7525873`;
  - the preview853 soundtrack PCM;
  - picture admission 864: the root and independent viewing evidence, the 16504 frames and the source sha `cd1900aa`;
  - score21's 09 stems and qa, by hash.
- **Affinity.** At least 16 CPUs, of which the first 16 must be distinct physical cores (package and core id). Then `sched_setaffinity` to those 16. `PATH=/usr/bin:/bin`, with TMPDIR and XDG_CACHE_HOME under `work`.
- **Argv:**

  ```
  binary render 09_boundary --res 3840x2160 --fps 60 --threads 16 --narration delivery849/narration --score score21 stems --out JOB/render --lint --crf 16 --preset veryfast
  ```
- **Run.** `h.run_step(JOB, 'native', args, 86400, None)`.

### Launcher `csd-native968-launch.py` (`47b8f22b`)

- It refuses an existing JOB or INPUTS (no duplicate, no overwrite).
- It checks the failed 864 receipt and cleanup, and refuses a surviving 864 native process by scanning /proc cmdlines for the binary plus the 864 render directory.
- **Inputs manifest (closure):**
  - every row of the pictures855 inputs, rehashed;
  - the admission and its three evidence files;
  - the after-timeline853 helper at `8a3039d8`, whose `tables()` runs on preview853;
  - comparison852 and comparison853 at 16504 frames;
  - all files of pictures855 and of the score21 stems;
  - the launcher itself, the runner, Python, the score21 receipt, and the 864 receipt, command log, cleanup and inputs.
- The manifest is written with `open('x')`.
- **Guard.** `guard_csd02.py --phase native --job JOB --inputs INPUTS -- python -I -B runner`, `check=True`, then `success()` on the job receipt.

## Encoder state in master993 (`completed-master993-summary.json`)

The 11 rows with an encoder string are x264 core 165. All of these are equal across them:

- crf=16.0, rc=crf, bframes=5, b_pyramid=2, ref=1;
- keyint=250, keyint_min=25, scenecut=40;
- me=hex, subme=2, trellis=0, 8x8dct=1, cabac=1;
- deblock=1:1:1, weightp=1, aq=1:0.60, psy_rd=0.40:0.00;
- mbtree=1, rc_lookahead=10, qcomp=0.60, sliced_threads=0.

Only `threads` and `lookahead_threads` vary (24/8, and 21/7 on 10_rigidity), following the host's CPU count. Retained rows 02, 03, 13 and 15 carry no recorded string. `combined-master993.py` `x264_settings()` reads the string from the first 16 MiB and asserts `rc=crf` and `crf=16.0` on non-retained rows.

## What to check in the generalized helpers

1. **Resource:**
   - the policy dict, the 86400 s timeout, the 16-physical-core check and the affinity;
   - one guard, one native step per job;
   - the per-chapter estimate;
   - the stop and cleanup path, and refusal of a surviving native process for every prior attempt.
2. **Closure:**
   - every pin above is kept or replaced with an equally exact pin;
   - inputs are frozen with `open('x')`;
   - the chapter allow-list matches the requested set;
   - the job, inputs and failed attempts are never reused or overwritten.
3. **Encoder:** the same argv for `--res 3840x2160 --fps 60 --crf 16 --preset veryfast`, the same binary encoder path, `--threads` and the affinity, and no new encoder flags.
