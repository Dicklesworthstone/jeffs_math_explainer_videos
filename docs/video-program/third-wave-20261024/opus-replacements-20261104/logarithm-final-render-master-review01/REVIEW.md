# Review of `render-master01.py` (87do.63)

LavenderPike, 2026-11-05, for BlueSalmon. Read-only: no production file, job or lane was touched.

- `S` = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/final-picture-repair01`
- `B` = `narration_lab/third-wave-20261024/root/five-film-release-20261027`
- Driver reviewed: `S/render-master01.py`, 752fe43f, 190 lines, unchanged since 16:33.
- "Worker" below means the stage09 `logarithm-stage09-master02.py` (c43e7b83) that the driver edits. Worker line numbers are that file's.

## Result

**I found no launch-blocking defect.** In throwaway repositories, the real driver ran 13 renders and the master through root's real guard code and the edited worker, using a stand-in binary. Root's added gates (full timeline, full cues, lint file) would have passed all 13 real stage09 chapters, and each one stops the chain when its condition is broken.

**Three things are worth fixing before launch.** D1 and D3 can stop the chain with no way to resume. Stage09's 13 renders took 17,876 s of lane time, and 9.6 hours end to end with other jobs between them. D2 can start that chain on a score that failed Ruby's verification. The rehearsal demonstrates all three, except the `not_as_predicted` case in D2.

| # | Priority | Location | Problem |
|---|---|---|---|
| D1 | Before launch | `render-master01.py:136,142,157,173,188-190` | No resume or retry. Any failure leaves every rerun refused at chapter 00. |
| D2 | Before launch | `render-master01.py:72,79-84,187` | The driver does not wait for, or read, Ruby's verify and compare results. |
| D3 | Before launch | `render-master01.py:118-122` | Every job freezes 4,347 build rows that no render reads, including 59 in a shared `target/` folder. |
| D4 | Worth fixing | `render-master01.py:144-145` | The binary and narration are never tied to what the timeline and score used. |
| D5 | Worth fixing | `render-master01.py:161-164` | Chapter movies are not compared to their render summaries before the master. |
| D6 | Worth fixing | worker `master()`; `S/workspace/tools/narration_delivery.py:242-251` | A lost prediction hold passes silently. Nothing checks for 13 holds or 364 sentences. |
| D7 | Optional | worker lines 191-192 | Captions run after the master, so a caption refusal strands a finished master. |
| D8 | Cosmetic | `render-master01.py:143` | Render configs keep the old `captions_argv`, and the worker docstring still says stage09. |

D2, D4 and D5 are the ones that could let a wrong input through. D1 and D3 only stop the chain. D6 could lose a caption hold without notice.

## Findings

Fix sketches are not run. Root owns the design and every production edit.

### D1. No resume or retry

Names are fixed at `-01`. Line 142 refuses if the job folder, script, config or manifest exists, and the guard keeps every attempt's folder. So one failure ends the chain, and a rerun cannot continue.

Demonstrated:
- **Non-admission:** the guard refused chapter 00 once (exit 75). A rerun with a healthy host hit the assertion at line 142.
- **Failure at chapter 02:** chapters 00 and 01 finished. A rerun refused at chapter 00, although that chapter had succeeded.
- **Partial launch:** line 173 opens the manifest before `freeze()` runs. A refused row left the script, the config and an empty manifest. After the row was restored, the rerun still refused, and no job had started.

This can happen in practice. Stage09 had `logarithm-stage09-timeline01` exit 1, and `complex-stage06-timeline01` exited 1 after four hours. Ruby's handoff also names this retry gap as your call.

Fix:
1. Skip a chapter whose job succeeded and has `render-summary.json`.
2. Add a retry name, for example version 4's `-r2` form, read from a small map you write. Line 157 must use the same map, so the master follows the attempt that succeeded.
3. Run `freeze()` before writing anything, then add the script and config hashes to its result.
4. Worker lines 154-155 refuse a chapter whose `.mp4` exists. If a job fails after publishing its movie, move the movie aside before retrying.

### D2. Ruby's verification is not waited for

`score-after-final-picture-timeline01.sh:26-27` runs `verify-step` and `compare` after the guard returns. `wait_for` at line 187 returns as soon as the score's receipt is finished. Renders can therefore start before those steps end, and they start even if the steps fail.

These gates are then skipped:
- `score_gate`: stem format, stem length of frames × 800, and QA length.
- `full_tables`: the three tables agreeing with each other and with the WAVs.

Line 79 onward checks only QA `passed` and metric `pass`.

Demonstrated: the chain completed with `hard_failures` in `score-comparison01.json` and with no stem files on disk at all. That run used a stand-in binary. The real `Kit::finish` refuses a wrong stem length at the end of a chapter (`kit.rs:204-212`), so the realistic cost is one lost render plus D1, not a bad film. `not_as_predicted` is the case this rehearsal does not test: it is informational, and as written nobody has to read it before 5 hours of renders start.

`compare` writes its file only after `verify` passes (`prepare_final_picture_pipeline.py:477-490`), so that file's existence is the proof. The schedule comparison is written before its own `hard_failures` refusal, so the list must be read too.

Fix (covers D4 as well):

```python
BINDING = S / 'final-picture-binding01.json'
SCHEDULE, SCORED = S / 'schedule-comparison01.json', S / 'score-comparison01.json'

def compared():
    """Ruby's verify and compare run after the guard returns; these files are the only proof."""
    try:
        found = [read(SCHEDULE), read(SCORED)]
    except (FileNotFoundError, json.JSONDecodeError):
        return False
    assert not any(d['hard_failures'] for d in found)
    return True
```

Then:
- Require `compared()` in the wait at line 187 and again in `prerequisites()`.
- In `prerequisites()`, assert `read(BINDING)['build']['binary_sha256'] == sha(BIN)` and `read(BINDING)['audio']['narration'] == str(N)`.
- Add the three files to `paths` at line 148.
- Decide how `not_as_predicted` is handled. I suggest refusing when it is not empty, until you pass the approved hash of `schedule-comparison01.json`.

### D3. Build rows in every render manifest

`freeze()` copies both manifests into each job. That is 4,347 build rows: 59 under `target/geometry-kit01/debug` and 4,057 under the engine tree. A stage09 render manifest had 334 rows and none of these. Across 14 jobs, every row is checked at least 42 times (one `freeze()` assert and two guard checks per job).

No render reads these rows, because the binary already exists. If one moves, a good chapter fails. Demonstrated: a stand-in row changed during chapter 05, the worker exited 0, the `.mp4` was kept, the guard recorded `reviewed input changed`, and the chain stopped. That leads straight to D1.

The risk is real, though not current:
- `build-freeze01.json` records one row in that same folder that had already moved, and you excluded it.
- The 59 remaining rows are logarithm outputs last changed at 00:34 today, so they look quiet now.
- `target/geometry-kit01` is 3.23 GiB on a disk with about 20 GiB free, so it is an obvious reclaim candidate.

Fix, smallest first:
- In the row loop at lines 119-122: `if Path(row['path']).is_relative_to(R / 'target'): continue`. The assemble manifest repeats these rows, and this loop covers both.
- Better: keep only rows a render can read (`S/workspace`, the audio rows), and bind each build manifest by its own hash, which line 123 already does.

Hashing cost is not a concern. The build rows total 318 MB.

### D4. Binary and narration are not tied to earlier jobs

Line 144 takes `binary_sha256=sha(BIN)` at each launch, so the worker's check at line 211 only proves the file has not changed since that launch. Ruby's binding records the hash the timeline and score used, but nothing compares them. Demonstrated: a binding holding another hash did not stop the chain. The fix is in D2.

### D5. Chapter movies are not compared before the master

Lines 161-162 use the soundtrack hash from each `render-summary.json`, but not `mp4_sha256` or `timeline_sha256`. `WORK` is not frozen between renders, and the master's manifest freezes whatever is there at that moment. The kit's picture gates at the master check frame counts and timestamps, not content.

Demonstrated: a chapter movie changed after its render, and the master still passed with the changed bytes frozen.

Fix, in the loop at line 161: read the summary, then assert both hashes against `WORK`. This gap is in my worker template as well as in the driver.

### D6. A lost prediction hold passes silently

If a `predict` mark falls outside its window, `narration_delivery.py:242-251` and `captions.rs:524-536` simply do not extend the caption. Nothing refuses. The worker records no counts.

Stage09's master log shows the numbers to expect: 13 prediction holds, 80 verified cues, and `captions: 364 sentences as 475 captions`. The repairs changed the source files of chapters 04, 05, 06, 11 and 12, and source markers are not timing.

Fix, after `launch('master')`: read `B/logarithm-final-picture-master01/command.log`, parse the JSON line with `"consumer": "master"`, and assert 13 holds and 80 cues. Also assert the `364 sentences` line. Leave the 475 caption count as a report, since it is not a contract.

I did not rehearse this finding, because the stand-in binary has no real contract output.

### D7. Captions run after the master

Worker lines 191-192 run the master, then captions. A caption refusal leaves a finished master, no summary and a failed job, and worker lines 179-181 then refuse a rerun. Demonstrated.

Captions need only the timelines in `WORK` and `report.json`, so the order can swap. In `worker_text()`, replace the two-line string `run(CONFIG["argv"], 14400)\n    run(CONFIG["captions_argv"], 1200)`, which appears only in `master()`. Stage09 passed with the current order, so this is optional.

### D8. Cosmetic

Render configs inherit the stage09 `captions_argv`, which names the old binary and is unused. The generated worker's docstring still says stage09. Neither has any effect.

## Checked and held

| Item | Evidence |
|---|---|
| Full-table gates | Timeline and cue tables were equal on 13 of 13 stage09 chapters, timeline job against 4K render. My main concern; it did not hold up. |
| Lint gate | `.layout.tsv` exists for all 13 stage09 chapters. |
| Cue logs | They differ by design (the render adds its stem row). The driver is right not to compare them. |
| Worker edits | The diff against the old worker is exactly the five intended changes. `--check-template` passes. |
| Interpreter | `python3` and the absolute 3.15 path report the same `sys.executable`, and the guard's command matched in the rehearsal. |
| Render flags | Identical to stage09: `--res 3840x2160 --fps 60 --lint --crf 16 --preset veryfast`. |
| Timeline schema | The narration path is the same absolute string in Ruby's bind and in the driver, and there is no symlink in the repository path. |
| QA and summary schema | 13 reports. The keys match, including the null `pass` metrics. |
| Guard timing | `finished` is the last field saved, so `successful()` cannot read a half-final receipt. |
| Master freeze of `WORK` | Stage09 did the same with 78 rows and stayed stable. The kit only adds new files there. |
| Growth | Stage09: timeline 0.03, score 0.69, largest render 0.09, master 1.02 GiB, against a 4 GiB cap per job. |
| Lane room | About 46.6 + 2.3 GiB against 56. |
| Kit actually compiled | `W/repo-view` differs only in `cli.rs` and `glyphs.rs`. I read the `cli.rs` diff: it adds an `adjust` hook that this film's `run_with_contract` leaves as a no-op. The other 17 files, including `captions.rs`, `master.rs`, `kit.rs` and `encode.rs`, are identical. |
| 4K raw cap | 256 GiB. Stage09's longest chapter was 16,550 frames. Chapter 04 is predicted at 11,749 and chapter 12 at 10,257. |
| Soundtrack completeness | Worker `incomplete()`, the kit's null test, the sample count, the audio-length gate and the contract's stem-hash check in the cue log all remain. Stems are frozen. |
| Caption coverage | `video_captions` errors on any timeline line missing from the report, or on a sentence-span mismatch, so all 80 cues are covered by construction. |
| Closing hold | It is the last line of the film, so no caption extends over the 7 s tail. That matches `prediction_end` in both the kit and the contract. |
| Ruby's hold gate | Passes on my rebind rehearsal's outputs, 15 holds plus the closing hold. |

## Disk reading

This is not a driver defect, but it bears on D1.

At 22:01Z the disk had 20.5 GiB free, against admission 10 and stop 8. The running `complex-stage06-optimized-timeline01` started at 48.5 GiB and has dipped to 16.8 GiB, while the lane stayed at 46.61 and `target/` at 16.69. So about 30 GiB moved outside both, and I did not find where. Stage09's chain ran with 86 to 113 GiB free, so it never tested this margin.

This chain should need about 2.3 GiB in the lane, plus the build in `target/`. A fresh debug build took 1.34 GiB. The release build is unmeasured, and it must stay under the 4 GiB job cap and the 7.13 GiB left in the target budget. A repeat of that swing during a render would hit the 8 GiB stop and end the chain, which is why D1 matters.

## Limits

- **Nothing real ran.** No build, rebind, assembly, timeline, score, render, master or real job. I did not use the real lane, and no `logarithm-final-picture-*` job folder existed at my last check.
- **The rehearsal proves control flow only.** The binary, tables, stems, receipts and narration are stand-ins. It says nothing about the real kit, its timing, or real stems.
- **Stage09 evidence comes from a prior run of older source.** Table equality on 13 chapters there is strong evidence for the new gates, not proof for this build.
- **Ruby's bundle was read where it meets this driver,** at seal 3df900b0. This is not a full review of it, and it has not run on real outputs either.
- **Not checked:** every path through `coherent verify`. 16 rows in the old render manifest have no counterpart in the new one. They are planning files, receipts and helpers, and I did not trace each one, though the tool resolves its map inside `S/workspace`.
- **No listening, picture review, fine-sync or AV acceptance.** Nothing was heard or viewed. The closing hold and the changed bars remain unmeasured. A strict verify, ASR and a caption screen are still needed after the master; the driver stops at the master.

## Disclosure

This is not independent for the worker. I wrote `prepare_stage09_render.py` version 4, which generated the worker this driver reuses, so D5 and D7 sit partly in my own template. I also wrote `rebind_audio.py`, the stage09 audio preparation, the timing inspection, the voice repair and `final-verification02`. I did not write `render-master01.py`, the guard or Ruby's pipeline.

## Files

| File | What it is |
|---|---|
| `REVIEW.md` | This review |
| `checks/rehearse_render_master.py` | Throwaway-repository rehearsal: real driver, worker and guard code, stand-in binary |
| `checks/outputs/rehearsal-results.json` | 19 of 19 as expected; two runs byte-equal. A `finding` row passing means the defect appeared as described |
| `checks/stage09_evidence.py` | Read-only stage09 and manifest facts |
| `checks/outputs/stage09-evidence.json` | Its output; two runs byte-equal |

Run either script from the repository root, as its docstring says. Neither touches the real lane.
