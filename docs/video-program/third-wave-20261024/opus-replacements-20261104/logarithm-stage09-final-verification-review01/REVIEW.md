# Review: the stage09 final verification helper (87do.63)

BronzeJaguar, 2026-11-05. This answers root's Mail 4246. It reviews LavenderPike's sealed `logarithm-stage09-final-verification01` (Mail 4229). `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`; `W` = root's geometry stage; `WS` = `W/logarithm-stage09-workspace01`; `EL` = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104`.

## Verdict

**Do not run the sealed helper on the stage09 master as it is.**
- **F1 (High, blocking).** `check` and `write-job` refuse root's real 80-cue narration. A one-line fix is staged.
- **With F1 fixed, the helper's core holds up against the kit and the real data:**
  - cue matching;
  - frame counts;
  - frame extraction;
  - timeline parsing;
  - Ruby's job schema, both v1 and v2.
- **Two lower findings (F2, F3)** concern the evidence kept after the run. They have narrow fixes, which are in the same staged file.
- **Lavender's own 11 checks pass, unchanged, on the fixed file.**

## Findings

### F1 (High, blocking): the report-order gate refuses the real narration

`gather()` (helper line 462) runs, before captions, holds or the picture plan:

```python
need(list(report["lines"]) == list(script), "report.json's cues differ from the script")
```

**Why it refuses:**
- The canonical assembler writes `report.json` through `narration_cache.encoded`, which is `json.dumps(..., sort_keys=True)`. So the report's lines are in text order.
- Stage09 has cue ids `06_20` and `11_20`, and the script plays them after `06_6` and `11_7`. In text order they come before `06_3` and `11_3`.
- **Result:** 17 of the 80 positions differ, and the statement refuses root's real `prepared01/narration/report.json` (`77dd97b0…`). K05 runs the helper's own statement, unchanged, on that file.

**Why the checks missed it:**
- F2's Nov 2 cue ids happen to sort in script order.
- F10's fixture report was written in script order.

**The order is not needed here:**
- The kit reads the lines into a map: `captions.rs` `read_report` gives a `HashMap`, and lines are looked up by id.
- Every other use in the helper is also by key.
- The order that matters is checked on the timelines' placements (`read_schedule`).

**Fix:** `need(sorted(report["lines"]) == sorted(script), …)`.
- Root's real report is accepted.
- A report missing `11_20`, or with an extra cue, is refused (K09).

Ruby's `bind` is not affected. Its two order checks read the prepare receipt and the phrase timings, and both are in script order.

### F2 (Low): after the run, `verify-job` does not prove the pictures or sheets are the job's

- `verification.json` records each picture's frame and path, but no hash, and it records no hash for any sheet.
- After the run, `verify-job` re-hashes only the master. For the SRT it compares only the caption screen's recorded hash.
- **On a stand-in finished job (K07),** `verify-job` still passes when:
  - a sheet is replaced;
  - a picture is replaced;
  - the bound SRT is changed after the job.
- The guard proves the inputs did not change during the run. Nothing proves that the sheets root views later are the bytes the job wrote.
- **Fix (staged):**
  - `run_job` stores `sha256` for each picture, and `sheet_sha256` for the sheets;
  - `verify-job` re-hashes them and the SRT and VTT.
- On the same fixture, each tamper is refused (K09).

### F3 (Low): the strict check keeps only the last decode's log

- Root's `verify_video_preserve_logs01.sh` makes one `mktemp` log. Both decodes write it with `>`.
- On macOS, `VERIFY_HWACCEL=auto` runs videotoolbox second, so `decode-logs/verify_decode.*` holds the hardware decode only.
- The software decode survives as the strict log's `full decode (…)` line and its first five error lines.
- F11 ran with `VERIFY_HWACCEL=none`, so this path was never exercised.
- HANDOFF says "its decode logs stay with the job". That is true of one log, not both.
- **Fix (staged; root's script unchanged):**
  - `run_job` copies the strict log's `full decode (…)` lines into `verification.json` as `strict_video_decodes`;
  - `verify-job` requires the first one to read `0 error lines, ffmpeg exit 0`.
- Root then sees, in `verification.json`, whether the second decode ran and how it ended.

### Notes (no change needed)

- **N1. The picture plan is large.**
  - Root's real narration totals 1800.8 s, so the film is over 30 minutes.
  - The cue-local samples alone are at least 1020 distinct frames, at least 255 sheets for root to view. That count is before any chapter, hold or layout sample.
  - `check` prints the real count first.
  - The pictures are small: the 960x540 PNGs in K06 are 25–39 kB.
  - The strict check's software decode took 7 s for the 296 s Nov 2 master.
- **N2. Per-chapter frame counts.**
  - `run_job` compares the master's packets with the sum of the 13 timelines' frames.
  - On the Nov 2 render, each chapter mp4 has exactly its timeline's frames (7/7), so the sum loses nothing there.
  - A per-chapter `-count_packets` in `check` would pin every chapter offset. It is cheap, but optional.
- **N3. Already in Lavender's HANDOFF:**
  - the hardware second decode is unexercised;
  - the strict script passes a quiet fixed-gain master at the ceiling.

## Root's four questions

### Cue matching (K03, K04)

- **How captions are assigned.** The helper gives each caption to the last cue placed at or before its start + 1 ms.
- **What the kit's writer does.** For a sample-timed report, the writer starts each cue's first sentence at `offset + start_frame / fps` and rounds stamps to the ms. The relevant lines are in `captions.rs` and are asserted present.
- **On the real Nov 2 SRT (64 captions):**
  - each of the 14 cues gets its own first caption, within 0.333 ms of its placement;
  - the caption before each one starts at least 3.998 s earlier, so the 1 ms window cannot move one.
- **On stage09's real report:**
  - it is sample-timed too;
  - each cue's sentences read exactly its script text;
  - each cue's `out_samples + gap_after_samples` equals its WAV;
  - `sentence_onsets` gives 364 onsets;
  - `check_phrases` accepts all 80 entries against the real WAVs and script, with 33 marks.

### Full-frame counts (K06)

- **Presentation times.** On the real Nov 2 master, every one of the 17783 video packets sits exactly at n/60 s (time base 1/15360, B-frame delay 2, start pts 0). So the plan's frame numbers are presentation order.
- **Counts.** Packets equal the timelines' frames, as a total (Lavender's F4) and per chapter (N2).
- **Extraction.**
  - The test frames are 5 frames in motion, where the frame and both neighbours differ.
  - At each, the job's own `-ss frame/60 -frames:v 1` picture equals an exact `select=eq(n,f)` pick of that frame, and differs from frames f−1 and f+1.
  - So each sheet label names the frame shown.

### Persistence extraction (K07, K08)

- **What the job keeps:** the strict log, one decode log, the loudness log, the caption screen and the pictures.
- **What `verification.json` records:**
  - the `note:` lines;
  - the media facts;
  - the loudness Summary;
  - the caption-screen counts;
  - each picture's frame, time and reasons.
- **The gaps are F2 and F3.** The ebur128 Summary parse and the `pictures_viewed: false` gate are sound (Lavender's F9 and F11).

### Guard and input binding (K01, K02; read against Ruby's code)

- **`bind_master` checks the master's whole command chain:**
  - receipt, command and input hash;
  - every bound row still current;
  - the CONFIG's step, job, lane, narration, render work, chapters and cues;
  - the bound binary;
  - the exact `argv` and `captions_argv`;
  - the success lines;
  - the summaries.
- **What Ruby's `write_step` binds** includes every row the helper requires:
  - the whole narration tree, including `report.json` and `phrase-timings.json` (v2's narration binding lists 181 files in all, with the receipts and decoded WAVs);
  - the 48 stage rows, including `script.tsv`, the oliver16 map and `mod.rs`;
  - all files of the timeline job;
  - each render's mp4, timeline, cue log, cues and layout.
- **Layout files.** The kit writes `layout.tsv` on every `--lint` render, empty or not, so Ruby's `is_file()` filter drops none.
- **The job's own inputs** bind its 141 rows.
  - Its preconditions refuse a run outside its own receipt, and moved media.
  - `verify-job` ties the receipt to the inputs file.
- **The timeline parser** equals a mirror of the kit's `ChapterTimeline::parse` on all 560 real timelines in the lane. These hold 269 `mark` records (label, then frame) and 26 `breath` records.

## Evidence: real, emulated and fixture

| Evidence | Kind |
|---|---|
| K02 (560 timelines), K03 (Nov 2 SRT), K06 (Nov 2 master and chapters) | real kit output from root's lane |
| K04, K05 (root's `prepared01/narration`: report, 80 WAVs, phrase timings) | **real stage09 production audio**, read only |
| K07, K09 fixtures | stand-in files labelled in their bytes; they exercise `verify-job` only |
| Lavender's F2, F4, F5, F9 | real Nov 2 outputs |
| Lavender's planted modes (F3, F4, F7, and the refusals in F5, F6, F8, F9, F11) | edits of copies, or stand-in inputs |
| Lavender's F7, F10 | Ruby's real `JOB_TEMPLATE` and `step_config`, with stand-in media and a stand-in report written in script order (which is why F1 was missed) |
| Lavender's F11 | `run_job` end to end on the real Nov 2 master, under an emulated guard receipt, software decode only |

**Not yet tested on any final production output.** No stage09 master, captions, chapter render, score or finished timeline exists yet. The real `logarithm-stage09-timeline01` job was still running when I checked, and I did not read it.

## Ruby's schema (coordination)

- **v2 changes none of the helper's dependencies.** Ruby's `logarithm-stage09-render-preparation02` keeps these byte-identical to v1:
  - `job_name`;
  - `SUCCESS`;
  - `output_rows`;
  - `step_config`;
  - `write_step`;
  - `JOB_TEMPLATE`.
- **Only `bind`/build changed**, and v2's binding still holds the whole narration tree.
- **The real timeline job's CONFIG names:**
  - N = root's `EL/production-audio-stage09-01/prepared01/narration`;
  - series 01;
  - `WS/logarithm-stage09-ws-build02-binary`;
  - render work `L/logarithm-stage09-render-work01`.

**What the helper depends on.** Any later change by Ruby to these needs a matching helper change:
1. **Job names:**
   - `logarithm-stage09-{timeline,score,master}<SS>`;
   - `-render-<chapter>-<SS>[-rN]`;
   - `L/logarithm-stage09-render-work<SS>`.
2. **Job files:** `J/<job>.py`, with one `CONFIG = json.loads(…)` line, and `J/<job>-inputs.json`, whose row 0 is the script.
3. **The receipt and log:**
   - phase native;
   - command exactly `python3 -I -B J/<job>.py`;
   - the log ending with the exact timeline, render and master success lines.
4. **The master's CONFIG:**
   - `step`, `job`, `lane`, `narration`, `render_work`, `chapters`, `cues`, `timeline_job`;
   - `render_jobs`, in chapter order;
   - `binary` and `binary_sha256`;
   - `argv` = `[B, master, --narration, N, --score, L/<score job>/scored/stems, --out, work]`;
   - `captions_argv` = `[B, captions, all, --narration, N, --out, work]`.
5. **The summaries:**
   - `master-summary.json` {master, srt, vtt} with {path, sha256}, naming `logarithm_mysteries_master.mp4`, `.en.srt` and `.en.vtt`;
   - `render-summary.json` {chapter, mp4_sha256};
   - `timeline-summary.json` {fps, chapters [chapter, frames, cues], cues, total_frames}.
6. **The master's inputs must bind:**
   - the binary;
   - `script.tsv`, the oliver16 map and `mod.rs`;
   - `N/report.json` and `N/phrase-timings.json`;
   - the timeline summary and its 13 outputs;
   - each chapter's mp4, `timeline.tsv` and `layout.tsv`.

## The staged fix

`verify_stage09_final_fix01.py` (`ea9cd27f…`) is the sealed helper plus `verify_stage09_final_fix01.patch`: +20/−7, 4 hunks.

| Hunk | Function | Change |
|---|---|---|
| 1 | `gather` | F1: compare the cue sets |
| 2 | `run_job` | F3: keep the strict log's `full decode (…)` lines |
| 3 | `run_job` | F2: hash each picture and sheet, and record `strict_video_decodes` and `sheet_sha256` |
| 4 | `verify_job` | F2: re-hash the pictures, sheets, SRT and VTT. F3: require the clean software-decode line |

Results:
- **Lavender's checks.** Her `run_checks.py`, unchanged, passes 11/11 on it (`checks/lavender_checks_on_fix.py`, which swaps only her `HELPER` constant). Two runs gave the same transcript, and F11's run wrote the decode line and the hashes.
- **K09:**
  - the patch equals `difflib(sealed, fixed)`;
  - the fixed gate accepts the real report;
  - the tampers are refused;
  - `REPO` resolves correctly. The file must sit directly in a folder under `P`, because `REPO = HERE.parents[5]`.

**Not applied anywhere.** Lavender's sealed bundle is unchanged. Root, or Lavender in a new folder, installs the fix.

**Commands with the fix.** These are from the repository root, with absolute paths and the values above:
```
V=P/logarithm-stage09-final-verification-review01/verify_stage09_final_fix01.py   # or root's installed copy
python3 -I -B $V check --lane L --jobs WS/jobs --crate WS/explainer_corpus/logarithm_mysteries \
  --narration REPO/EL/production-audio-stage09-01/prepared01/narration --series 01
```
`write-job` and `verify-job` are as in Lavender's HANDOFF.

## Limits

- **Independence.** I wrote neither helper. My checks load Lavender's in process and mirror the kit's lines. Each mirrored line must still be in its pinned source, or the run stops with exit 3.
- **No production work:**
  - no guard launch, production decode, render, assembly or Cargo;
  - no canonical, Beads, TODO or sealed file edited.
- **The only media work:**
  - ffmpeg on the existing Nov 2 master: 160x90 framemd5 of its first 1800 frames, and 5 `-ss` picks plus 15 exact picks at 960x540, all into scratch;
  - ffprobe packet reads.
- **Root's production audio** was read only.
- **Nothing was heard or viewed.** Pictures were compared by hash only.

## Files and replay

| File | What it is |
|---|---|
| `REVIEW.md` | This note |
| `checks/review_checks.py` | K01–K09: read-only on the repository, W and the lanes; writes only under NEW_SCRATCH |
| `checks/lavender_checks_on_fix.py` | Lavender's checks, unchanged, on the fixed helper |
| `checks/outputs/review_checks-01.txt` | K01–K09: 9/9 PASS; two runs byte-equal |
| `checks/outputs/lavender-run_checks-replay01.txt` | Her sealed checks replayed: 11/11, byte-equal to her `run_checks.txt` |
| `checks/outputs/lavender-run_checks-on-fix01.txt` | Her checks on the fix: 11/11; two runs byte-equal |
| `verify_stage09_final_fix01.py`, `verify_stage09_final_fix01.patch` | The staged fix |

From the repository root, run the second line first, because K09 reads its output:

```
python3 -I -B P/logarithm-stage09-final-verification01/checks/run_checks.py NEW_SCRATCH --w W
python3 -I -B P/logarithm-stage09-final-verification-review01/checks/lavender_checks_on_fix.py NEW_SCRATCH W
python3 -I -B P/logarithm-stage09-final-verification-review01/checks/review_checks.py NEW_SCRATCH --w W
```
