# complex-optimized-recovery-review01

WhiteSwan, for root. This is the staged recovery of `complex-stage06-timeline01` using the already-built release binary. Frozen once sealed. I launched nothing.

## Start

```
docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-optimized-recovery-review01/start01.sh
```

- **New folder:** `start01.sh` creates `EC/optimized-timeline-recovery01/` with `mkdir`, so the folder must not exist yet. A mail reservation is fine, but creating the folder first makes both `start01.sh` and `--check` refuse.
- **New job:** `complex-stage06-optimized-timeline01`, with output in its `timeline-output/`.
- **Guard:** the pinned, unchanged guard runs it: 56 GiB work, 20 GiB target growth, exclusive lane, 10 GiB admission, 8 GiB stop, 4 GiB per job.
- **One attempt:** if the guard refuses admission or the run fails, the job folder and manifest stay. A retry needs new names (`02`) in a copy of the helper and start script.

## What the run does

It is `timeline-after-audio04.py`'s run with three changes: the binary, the job and output names, and the render timeout (43200 s). All of that helper's checks stay: 15 chapters, 105 cues, facts against the oracle, narration verify, table checks. It adds:

- byte equality of the facts with the failed run and the release build;
- a line-exact comparison of the 12 finished tables, using `preview-after-score01.py`'s `normalized`, written to `prior-table-comparison.json`;
- the failed job's 22 files and the release binary in the manifest (5085 files), so the guard hashes them at the start and end;
- refusal of any added file in the narration or workspace folders.

The summary sets `full_baseline_equivalence_established: false` and lists chapters 05–15 as having no prior table.

## Assumptions corrected from the receipts

- **Timeout:** the 14400 s was the helper's own `subprocess.run` limit. The guard has no wall clock, and the receipt shows `exit` 1 with no resource stop.
- **Baseline:** `T.BIN` is the debug build. It finished only 01–04, and there is no `timeline-summary.json`.
- **Prior tables:** they sit directly in `complex-stage06-timeline01/timeline-output/`. The `timeline/` folder under it holds only four MP4s.
- **Candidate:** it was built from the same frozen inputs, and only the build profile differs. No manifest listed it until now.
- **43200 s:** this is a bound, not an estimate. It is the single-render limit `final-render01.py` already uses, and no speed is claimed.

## Downstream requirements

- **Old controllers can't be rerun:** `score-after-timeline01.py` and everything after it call `T.success(T.JOB)`, which raises on the failed receipt. The guard re-executes each helper as a fresh process, so an in-memory override doesn't carry over.
- **New copies:** make new copies with new job and manifest names that import `optimized_timeline_recovery01.py` in place of `timeline-after-audio04.py`. It provides every `T` name those helpers use, including the new `JOB`, `BIN` and `MANIFEST`. No such copy has been written or run yet.
- **Binary choice:** those copies then run the release binary, which is root's promotion decision. This run compares tables and facts only, and only 01–04 have prior tables.

## Remaining limits

- **Partial failure leaves no comparison.** `compare_prior` runs only after a full render exits 0. After a timeout or a guard stop, the finished chapters' tables remain in the new job, but `prior-table-comparison.json` isn't written.
- **A table mismatch is found only at the end.** The render isn't stopped early.
- **Free space is the likeliest stop.** It was 49.6 GiB at 20:45Z, down from 52.3 GiB at 20:34Z and 120.3 GiB at the failed run's admission. The guard stops the job below 8 GiB, whatever is using the space. At the same time work was 46.6 of 56 GiB and target growth 12.9 of 20 GiB.
- **What the rehearsal covers.** `checks/rehearse_recovery.py` runs `--run` with a fake binary and oracle, with narration verify stubbed. `--launch` was never run, and the release binary has never rendered a frame.

## Replay

```
R=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-optimized-recovery-review01
(cd $R && shasum -a 256 -c SHA256SUMS)
python3 -I -B $R/optimized_timeline_recovery01.py --check
python3 -I -B $R/checks/rehearse_recovery.py
```

Run these from the repository root. `--check` reads only. The rehearsal writes only to a temporary folder. Both stop being valid replays once `start01.sh` has run, because `--check` then refuses the used names.
