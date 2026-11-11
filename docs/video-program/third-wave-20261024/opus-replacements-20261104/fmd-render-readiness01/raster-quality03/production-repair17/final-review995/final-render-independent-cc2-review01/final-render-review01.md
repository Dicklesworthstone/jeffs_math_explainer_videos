# Review of CC3's repaired-final 4K render helpers

CC2 (RubyRaven), 2026-11-10, 23:58Z. Independent review, per root's assignment, of the sealed CC3 delivery `final-render-cc3`. Only these three areas are covered:

- **Resource:** the same as native968?
- **Closure:** every input pinned and frozen?
- **Encoder:** the same settings?

I wrote no part of these helpers. Everything on CSD was read-only. I launched nothing and built nothing.

**Verdict: no blocker.** There are two Lows and some notes.

## What I read

Snapshot of the delivery: all six files, read-only, unchanged at 23:57Z.

| File | sha256 |
|---|---|
| `csd-native-repair-final01-run.py` | `6dceac47` |
| `csd-native-repair-final01-launch.py` | `3c292758` |
| `check-final-render-cc3.py` | `b5fc5893` |
| `check-final-render-cc3.out` | `5c53a8a3` |
| `run-vs-968.diff` | `a77afa72` |
| `launch-vs-968.diff` | `f3f412f4` |

**Baseline.**
- 968 run `2634c842` and 968 launch `47b8f22b`.
- `completed-native968-resource-receipt.json`.
- The master993 encoder strings.

**My checks.**
- I reran CC3's checker on my copy: 106 passed, 0 failed, the same output as CC3's.
- Two read-only CSD probes:
  - `csd_helper_probe01.py`: pins, receipts, the guard lock, the working directory and jobs.
  - `csd_encoder_probe01.py`: extradata and x264 options.
- Base761 `run_step` on CSD.
- The combined01 preview and timeline runners on CSD.
- The final968 resource receipt, and the free-space trend in recent receipts.

## Resource: the same as 968

**Policy.** The policy dict equals 968's exactly:

```
admission_gib=10, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10
```

The runner asserts the dict against the receipt, as 968 does.

**Base761.**
- Pinned at `32bf6d9e`, which I confirmed on CSD.
- Same calls: `own_group()`, `STATE['job']`, `CRATE = work/production-repo07`, and the stop-signal handlers.
- One native step per job: `run_step(JOB, 'native', args, 86400, None)`, the same 24 h timeout as 968.

**Cores and environment.** The 16-distinct-physical-cores block is byte-identical to 968 lines 107 to 116, as is the `sched_setaffinity` to those 16. The environment is unchanged:

- `PATH=/usr/bin:/bin`;
- TMPDIR and XDG_CACHE_HOME under `work`;
- the assertion `CARGO_TARGET_DIR == D/work`.

**Guard.** The command is 968's plus `--chapter CH`, and launch ends with `success()` on the job receipt.

**Serialization.** `guard_csd02.py` (`f1cae1db`) runs everything through the owner guard (`1c19a9b8` loads `819746fb`). That guard takes a blocking `flock(expanded-native-lane.lock, LOCK_EX)` before admission. So two guarded jobs never run at once, whatever the launcher does. The launcher's /proc scan also refuses a live repaired-final render or runner, which just refuses earlier.

**Working directory.**
- `run_step` starts the child with `cwd=CRATE`, which is `work/production-repo07`.
- The combined01 preview and timeline runners set the same `h.CRATE` (lines 48 and 47).
- In the combined01 sources, the only files read at run time are read by test code; render reads nothing relative to its working directory.

So the final does not run in a different directory from the preview it relies on.

**Rough per-chapter time.** This is a rough frame-proportional figure from 968, not a speed claim. 968 rendered 16504 frames in 4 h 17 min (12:40:47Z to 16:57:43Z). In proportion:

| Chapter | Hours |
|---|---|
| 01 | about 1.6 |
| 08 | about 3.9 |
| 09 | about 4.3 |
| 11 | about 3.1 |

That is about 13 h one after another. Each stays well inside its 86400 s timeout.

## Closure: pinned and frozen

**Order of checks.** Launch checks every proof (`run.proofs(chapter)`) and the unresolved pins before the job name or the inputs file exists. It refuses an existing JOB or INPUTS, and it writes INPUTS with `open('x')`.

**Command assertion.** The runner asserts that the receipt's command is exactly `[sys.executable, '-I', '-B', RUNNER, '--chapter', CH]`. This matches how the guard records the child argv: 968's receipt holds `['/usr/bin/python3','-I','-B',...run.py]`. On CSD:

- `sys.executable` is `/usr/bin/python3`;
- `Q` resolves to itself, with no symlinks;
- so `Path(__file__).resolve()` equals the string launch passes.

**Pins on CSD.** All of these match, read at 23:52Z:

- **build01 binary.** `complex_analysis_visual-combined01` hashes to its qualification `binary_sha256`, `aed99576b7a9cc42353c43963e1828acf3465426e028fc424eaf6712133725ce`. Tests 247, clippy true, probe true.
- **delivery01.** 105 byte-identical wavs, 20 repaired cues, and 105 wavs in `narration/`.
- **Normalized timelines and tables.** timeline01 and preview01: frames and normalized sha equal the pins for 01 and 08. The preview tables equal timeline01's `actual_sha256`.
- **Preview PCM.** Pins match: 01 is `79fc6e7d` and 08 is `2c277faa`. 09 was still being written by preview01 at the time.
- **Scenes in `work/native-combined01`.** `the_bump.rs` is `90b51c2b`, `loop_sums.rs` is `a8d62f56`, and `boundary.rs` is `4b689791`.
- **Stems.** All score21 stems and qa files for 01/08/09/11 match the pins. The glob sets are exact.
- **Sampling lists.** All four match, including 11's 848-frame list `25965a51`. prior775 is `2a186477`.

**Not present yet, as expected.**

- the picture inputs files and the pictures jobs;
- `work/native-counting02` and every counting02 job;
- any `repair-final` job or staged helper.

**Frozen inputs.** Launch hashes:

- every proof file;
- every row of the pictures inputs manifest (rehashed against its row);
- every file in the pictures job, the score21 stems and the narration;
- the launcher, Python, the runner, the pinned base761 and the score21 receipt.

`success()` requires `inputs_sha256 == final_inputs_sha256`. So a file changing between launch and the end of the job fails the job. The runner repeats `proofs()` under the guard.

**Admission.** The 864-style admission is kept:

- root viewed every selected frame (`viewed_frame_indices == frames`);
- the independent viewer's frames are a non-empty subset of those frames, with `authored_repair` false;
- both name this chapter's `source_sha256`, movie and extraction.

M1 holds: the runner carries no review scope.

## Encoder: unchanged

`render_args` is 968's argv with only the chapter, binary and narration replaced:

```
render CH --res 3840x2160 --fps 60 --threads 16 --narration N --score score21/stems --out JOB/render --lint --crf 16 --preset veryfast
```

The diff adds no encoder flag. Two pieces of evidence on CSD (`csd-encoder-evidence01.md`):

- **Finals.** All 15 master993 `final/` chapters and final968's 09 share extradata `caf9d50d` (High@5.2, ref 1, reorder 2, bt709 matrix). Their x264 options are equal except threads and lookahead_threads.
- **Combined01 binary.** Its preview01 output has the same options except crf 18. crf 18 is the preview's own flag. So the combined01 binary's encoder path keeps the final settings when given `--crf 16`.

counting02 is not built yet, so its binary is unobserved. It gets the same argv.

## Lows

**L1. The checker turns red once root fills a binary pin.** `check-final-render-cc3.py` line 56 requires both `binary_sha256` values to be `UNRESOLVED`. The next lines require `proofs()` to refuse for that reason. After root writes the build01 value (`aed99576...` above), those checks fail by design. They are a pre-fill snapshot, not a test to rerun. Either:

- record the 106/106 output as the sealed pre-fill result and do not rerun it after the fill; or
- read the pin back from `qualification.json` on CSD at fill time, as the runner itself does.

No new gate is needed.

**L2. CSD free space is now 17.9 GiB, against 131 GiB at 968's end.** All figures are from the job receipts and `df /` (the jobs live on `/`).

| Time | Free |
|---|---|
| 968 finished, 16:57Z | 130.8 GiB |
| pictures995, 20:13Z | 43.7 GiB |
| build01, 22:22Z | 18.9 GiB |
| now, 23:57Z | 17.9 GiB, 99% used |

Over the same period the guard's own measures barely moved: work went from 21.3 to 21.66 GiB and target from 6.53 to 6.55 GiB. `D/jobs` is 23.1 GB in total. So the drop came from outside this project's guarded jobs. It has been flat since 22:22Z.

968's own growth was small: work 20.57 to 20.65 GB, target 7.03 to 7.14 GB. So a final should fit above the 8 GiB stop. But an outside writer could still trip the guard's stop mid-render. A stopped final keeps its job name, and launch then refuses that name. A retry would need a successor prefix, as 968 needed after 864.

This is an operational note for root at launch time, not a helper defect.

## Notes

- **N1.** The checker is static: 106 assertions and no mutants. That is acceptable under root's "no extra gates" direction. The CSD-side facts it cannot see are covered by the probe above.
- **N2.** Two counting02 items stay open until build02 exists:
  - the 250-test qualification and its `binary_sha256` pin;
  - the encoder strings of a counting02 output.
- **N3.** A note for my own 08 evidence later. The runner requires the independent JSON to carry:
  - `movie_sha256` equal to the extraction's movie hash;
  - `extraction_sha256` equal to `sha(pictures.json)`;
  - `source_sha256` `a8d62f56`;
  - empty `concrete_defects`;
  - `safe_to_advance_to_final4k`;
  - non-empty `viewed_frame_indices` within the selected frames;
  - `authored_repair` false.

## Not claimed

These are not claimed: native, picture, motion, listening, fine-sync or AV acceptance, and no speed claim. The ElevenLabs Frederick narration is unchanged and not assessed.

## Files in this review

All in my scratch, `native968-generalized-review01`:

- `final-render-review01.md` (this file)
- `baseline-notes01.md`
- `csd-encoder-evidence01.md`
- `csd_encoder_probe01.py` and `.out.json`
- `csd_helper_probe01.py` and `.out.json`
- `cc2-check-run1.out`
- `snapshot01/`, holding the six delivered files
