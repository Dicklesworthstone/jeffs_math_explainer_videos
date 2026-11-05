# Final verification helper for the logarithm stage09 master (87do.63, Mail 4124)

LavenderPike, 2026-11-05. `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`.

**Root's request (BlueSalmon, Mail 4124):**
- Build a guarded final verification helper for the full stage09 master that RubyRaven's render route produces.
- Reuse root's strict verification and loudness helpers.
- Adapt them to the actual 13-chapter registry, the 80-cue script, the captions, and the layout, hold and phrase-mark sampling.
- All paths are arguments.
- Verify the actual master's frames, A/V and caption equality.
- No invented acceptance and no production launch: root views all sheets and owns the final review.
- Test the actual parsers, plus planted bad captions, a planted frame count and the required command binding.

**Status: ready for root.**
- **Not run on stage09 outputs:** no stage09 master exists yet.
- **What it has run on:** the Nov 2 logarithm machine render in the release lane. That run covered the read-only checks, root's strict check, the measurements and the picture extraction (§ Evidence).
- **What it does not do:** launch any job, view any picture or listen.

## Commands (from the repository root; every path absolute)

| Name | What it is |
|---|---|
| `L` | the guard's lane: `narration_lab/third-wave-20261024/root/five-film-release-20261027` |
| `J` | the render route's job folder (Ruby's `WS/jobs`) |
| `C` | the workspace crate (`WS/explainer_corpus/logarithm_mysteries`) |
| `N` | the bound narration (`audio.narration` in Ruby's `narration-bindingMM.json`) |
| `SS` | the render series |
| `NN` | a new verification number |
| `D` | the folder for the verification job script (for example `J`) |

```
V=P/logarithm-stage09-final-verification01/verify_stage09_final.py
python3 -I -B $V check      --lane L --jobs J --crate C --narration N --series SS
python3 -I -B $V write-job  --lane L --jobs J --crate C --narration N --series SS --verify NN --job-dir D
#   root launches the printed line:
#   python3 -I -B P/engine-secondary-validation01/guard_owner56g_target20g.py --phase native \
#     --job logarithm-stage09-final-verifyNN --inputs D/logarithm-stage09-final-verifyNN-inputs.json \
#     -- python3 -I -B D/logarithm-stage09-final-verifyNN.py
python3 -I -B $V verify-job --lane L --jobs J --crate C --narration N --series SS --verify NN --job-dir D
```

- **`check`** is read-only and prints JSON:
  - the master's path and hash;
  - the 13 chapters, 80 cues and total frames;
  - the caption facts;
  - holds, phrase marks and layout findings per chapter;
  - the number of planned pictures and sheets.
- **`write-job`** writes two new files, then prints the guard line:
  - `D/logarithm-stage09-final-verifyNN.py`: its CONFIG carries the whole plan and the helper's hash;
  - `D/logarithm-stage09-final-verifyNN-inputs.json`: 141 rows for 80 cues, the job script first.
- **Refusals by `write-job`:** an existing script, an existing inputs file or an existing `L/logarithm-stage09-final-verifyNN`.
- **`verify-job`** reads the finished job: its receipt, log, outputs and every picture and sheet. It ends with the open items.
- **The pictures for root** are `L/logarithm-stage09-final-verifyNN/sheet-NNN.jpg`, four per sheet, each picture labelled with its time, frame and first reason.

## Ruby's output schema (read from her sealed `prepare_stage09_render.py`, `9ddc2003…`)

**Job names:**
- `logarithm-stage09-timeline<SS>`;
- `-score<SS>`;
- `-render-<chapter>-<SS>[-rN]`;
- `-master<SS>`.

**Locations:**
- The scripts are `J/<job>.py`, each with one `CONFIG = json.loads(...)` line, and their inputs are `J/<job>-inputs.json` (row 0 is the script).
- Every job runs in the lane through the guard.

**The master's command** (`step_config`, lines 511–514):
- `argv`: `[binary, "master", "--narration", N, "--score", L/<score job>/scored/stems, "--out", <render work>]`
- `captions_argv`: `[binary, "captions", "all", "--narration", N, "--out", <render work>]`
- The render work folder is `L/logarithm-stage09-render-work<SS>/`.

**The files `master-summary.json` names:**
- `logarithm_mysteries_master.mp4`;
- `logarithm_mysteries.en.srt`;
- `logarithm_mysteries.en.vtt`.

**What the master binds:** Ruby's `write_step` (lines 540–545) binds every file this helper requires:
- the narration tree (with `report.json`, the installed `phrase-timings.json` and the 80 WAVs);
- the 48 stage rows (with `script.tsv`, `docs/narration-direction-map-oliver16.json` and `src/chapters/mod.rs`);
- the timeline job's summary and `timeline-output/`;
- each render job's chapter `mp4`, `timeline.tsv` and `layout.tsv`.

**Confirmation:** I asked Ruby to confirm this schema in Mail 4141. No reply had arrived at seal time.

## What `check` verifies (and `write-job` repeats before writing)

**Pins.** These root tools match the hashes this helper pins:
- the strict check `verify_video_preserve_logs01.sh` (`3f4c68db…`);
- the caption screen `caption-interval-audit01.py` (`3cc26436…`);
- the guard wrapper (`1c19a9b8…`) and its source (`819746fb…`).

**Registry and script:**
- The crate registers exactly the 13 stage09 chapters in order.
- `script.tsv` reads by the canonical assembler's rule as 80 cues.

**Command binding** (`bind_master`):
- The master's receipt shows the guard ran `python3 -I -B J/<master job>.py`, passed (Ruby's `receipt_ok`), and has an input hash equal to the inputs file's.
- Every bound row still has its hash.
- Its CONFIG names the master step, this job and this lane, the given narration and render work, the 13 chapters and the script's 80 cues.
- The binary is a bound row at its hash.
- `argv` and `captions_argv` are exactly the two commands above.
- The log ends with Ruby's master success line.
- The master, SRT and VTT are the files `master-summary.json` names, at its hashes.
- Each chapter's render job passed with its success line, and its `render-summary.json` hash equals the chapter `mp4` the master bound.
- The timeline job passed.

**Schedule:**
- Each chapter's render-work timeline has the same schedule as the timeline job's bound `timeline-output` file, record for record, by a port of the kit's `ChapterTimeline::parse`.
- Chapter, fps, frames and cue order match the timeline summary.
- Each cue plays `N/<cue>.wav` and ends inside its chapter.
- The chapters sum to the summary's total.
- The 80 placements are in script order.

**Captions** (the kit's exact SRT and VTT writers):
- The SRT parses equal to the VTT.
- Times are in order, without overlap, and inside the video.
- Each caption has one or two lines.
- The whole text equals the script.
- Cue by cue, the captions that start during a cue (1 ms tolerance for the kit's millisecond rounding) read exactly that cue's script text, and the narration report's sentences equal it too.

**Phrase timings:**
- Each entry in the installed `phrase-timings.json` names a script cue, its WAV hash and its text hash.
- Its marks follow the kit's rule: inside the WAV, seconds nondecreasing, bytes increasing.

**Holds:**
- The oliver16 map has 15 holds.
- Each hold names a script cue, and its `next_cue` must be the cue the film actually plays next.

**Layout:**
- Every finding row in the 13 `layout.tsv` files must parse to its `@t.ts` anchor.
- An unrecognized row is refused, never skipped.

## What the guarded job does (`run_job`)

1. **Preconditions.**
   - The guard's receipt must show this exact script running.
   - The master, SRT and VTT must still have the hashes written into the job.
   - Refused if any output or any `frame-*`/`sheet-*` file already exists in the job folder. ffmpeg 8.1.2's `-n` exits 0 and keeps an old file, as WhiteSwan noted in Mail 4189.
2. **Root's strict check**:
   - Settings: `EXPECT_RES=3840x2160 EXPECT_FPS=60 EXPECT_BT709=1 EXPECT_LUFS=-16 EXPECT_TP_MAX=-1.0`.
   - Writes `strict-video-verification.log`, which must end `VERIFY PASS`.
   - `TMPDIR` points at the job's new `decode-logs/` folder, so the decode logs the script keeps stay with the job.
   - The script's `note:` lines are copied into `verification.json`. For example, root's script passes a fixed-gain master quieter than −16 LUFS when its true peak sits at the ceiling, and says so in a note.
3. **ffprobe, plus a counted packet pass:**
   - exactly one h264 3840x2160 60/1 video stream and one AAC 48 kHz stereo stream;
   - **video packets equal to the sum of the 13 timelines' frames**;
   - video duration within one frame of that;
   - audio within one AAC frame plus 1 ms (the kit's master gate).
4. **Encoded loudness:** `ebur128=peak=true` with `-vn` (root's precedent decoded the video too). The output is `encoded-loudness.log`, and its Summary values (I, LRA, true peak) are copied into `verification.json`.
5. **Root's caption screen** on the bound SRT writes `caption-screen.json`.
6. **One picture per planned frame:**
   - `ffmpeg -ss <frame/60> -frames:v 1 -vf scale=960:540` gives `frame-NNNN.png`;
   - each must be a non-empty file;
   - the sheets are 2×2 JPEGs (`sheet-NNN.jpg`), written exclusively.
7. **`verification.json`** records everything above, with `"pictures_viewed": false`. Last come the count line and the success line:

```
FINAL VERIFY PASS: strict video check, frame count, A/V length and exact captions passed; every sheet still needs root's viewing.
```

**`verify-job` checks the finished job:**
- the receipt, the input hash and the script hash;
- the success line;
- the master's hash and frames;
- the pictures are exactly the planned frames, and every picture and sheet exists;
- `pictures_viewed` is false;
- the strict log ends `VERIFY PASS` and its decode logs exist;
- the caption screen read the bound SRT.

## The picture plan

**Root's Nov 3 sampling, unchanged:**
- per chapter: opening, middle and closing, plus every 600 frames;
- per cue: start, middle and end;
- each sentence onset and +30 frames;
- each post-cue hold at +12/+60/+120;
- each layout finding at −12/0/+12/+60/+120, plus every 300 frames after the chapter's first finding.

**Additions:**
- **Sentence onsets** come from the kit's `report_starts` rule (running `out_samples + gap_after_samples` over the rate), which is where the film plays them. Root's precedent used the report's `start` field, a position in the source take.
- **Every phrase mark** in `phrase-timings.json`, at +0, +30, +60 and +120 frames. The real file will carry 33 marks (Bronze, Mail 4206). Each picture is labelled with the mark's byte and its first four words.
- **Each of the 15 oliver16 holds** also gets its last frame before the next cue (−12 frames).
- **Each timeline `mark` record**, at the mark and at +30.

**Picture counts:**
- On the Nov 2 render (7 chapters, 14 cues, 296 s), this plan picks 195 frames, against the precedent's 140. All 140 of the precedent's frames recur.
- The stage09 count depends on its real timings. `check` prints it before anything is launched.

## Evidence

`checks/outputs/run_checks.txt` passes 11/11, byte-equal over two runs. The checks are read-only on the repository and on W, and write only under a fresh scratch folder.

| Check | What it shows |
|---|---|
| F1 | W's stage09 crate: the registry is the 13 chapters in order; `read_script` equals the canonical assembler's (80 cues); oliver16's 15 holds each name the script's next cue; the 4 pins match |
| F2 | **Real kit output** (Nov 2 machine render): 7 render-work timelines equal the timeline job's; 14 cues play the bound WAVs; 17783 frames. 64 captions: SRT = VTT, and the text equals the narration report's sentences as a whole and cue by cue |
| F3 | **Planted bad captions**, nine byte edits of copies of those real files, all refused with their own reason: <br>• a word changed in the VTT only, or in both; <br>• 02_2's first caption moved into 02_1; <br>• an overlap; <br>• the last caption running past the video; <br>• three lines; <br>• SRT numbering skipped; <br>• no `WEBVTT` header; <br>• comma stamps in the VTT |
| F4 | **The real master:** ffprobe and the counted packets give 17783 video frames, equal to the timelines; video 296.383333 s, audio 296.383 s. Planted and refused: the timelines one frame longer; a dropped frame; audio 50 ms long; 1920 wide; a third stream |
| F5 | All 79 real non-empty `layout.tsv` files in the lane (326 finding rows) parse; a checkpoint without its time is refused; a time inside the detail does not count |
| F6 | The plan on the real render (above). Refused: phrase timings for another WAV; marks out of byte order; a hold map naming another next cue |
| F7 | **Command binding** on a master script written by Ruby's own sealed `JOB_TEMPLATE` and `step_config`, with labelled stand-in media. The positive control passes. Eleven plants are refused: <br>• the receipt ran another script; <br>• exit 1; <br>• the inputs file edited; <br>• a chapter mp4 changed; <br>• three consistent forgeries, where the script's `argv --out`, `captions_argv` narration or CONFIG narration is changed and its inputs row and receipt are re-hashed to match; <br>• the script relabelled as a render step; <br>• the log without the success line; <br>• the master changed after its job; <br>• the timeline job failed. <br>It passes again after every plant is restored |
| F8 | The written job script refuses: run directly; a receipt for another script; a master whose bytes changed; a helper whose hash changed. Nothing is written |
| F9 | A real ebur128 log's Summary parses, and a log cut before it is refused. The job's frame command decodes the real master's frame 600 to 960x540. Five pictures make 2 sheets. Root's caption screen reads the real SRT (64 captions, 0 fast, 0 long lines) |
| F10 | **The command line, composed.** The fixture is F7's plus 80 one-second silent stand-in WAVs, a report and phrase timings in their writers' shapes, captions in the kit's format, and the real oliver16 holds. On it: <br>• `check` passes (13 chapters, 80 cues, 15 holds, 1 mark); <br>• `write-job` writes 141 current input rows with the script first, and prints the guard line; <br>• writing it again is refused; <br>• `verify-job` before a launch is refused; <br>• an SRT edited after the master is refused |
| F11 | **`run_job` end to end on the real Nov 2 master**, five planned pictures, under an emulated guard receipt with the hardware second decode skipped (`VERIFY_HWACCEL=none`): <br>• root's strict check passed, its note carried; <br>• 17783 frames; <br>• encoded I −19.7 LUFS, true peak −1.5 dBTP; <br>• caption screen clean; <br>• 2 sheets; <br>• `verify-job` passes on the result, and refuses a missing sheet and a `verification.json` that claims viewing. <br>A job folder already holding `frame-0003.png` is refused before any measurement |

## Open for root

- **Root views every sheet.** The helper extracts pictures and never claims they were seen.
- **Listening, fine sync, final ASR and acceptance remain root's.** The caption check compares text and timing windows, not speech.
- **The launch:** root launches the guard line. The job also runs the strict check's hardware second decode, which F11 skipped.
- **The phase:** `--phase native` follows Ruby's render jobs. Root may prefer another phase for a decode-only job.
- **The job folder `D`** is root's choice. Using `J` keeps every stage09 job script in one place, and the name `logarithm-stage09-final-verifyNN` does not collide with Ruby's step names. I asked her to confirm this in Mail 4141.

## Limits and what was not done

- No Cargo, build, render, synthesis, recognition, production decode or assembly; no guard launch.
- The only ffmpeg work was on the existing Nov 2 master, plus one frame for F9, all into scratch.
- No sealed, root, canonical, Beads or TODO file was edited.
- **F7 and F10 use stand-in media**, labelled in their bytes: a binary that is never run, silent WAVs and a non-video master. Their scripts, receipts and inputs files follow Ruby's and the guard's shapes.
- **The guard itself was not run.** F11 emulates its receipt.
- **Not independent:** I wrote both the helper and the checks.

## Correction to my earlier O3 bundle

`P/logarithm-stage09-plan-binding-fix01/HANDOFF.md` says the delta is "+7/−3". **The correct count is +8/−3** (`cmd_prepare` +1/−1, `cmd_timings` +7/−2), as Bronze's Mail 4151 N1 found. That bundle stays sealed and unedited; its patch and staged file are correct, and only the summary count was wrong.

## Files

| File | What it is |
|---|---|
| `HANDOFF.md` | This note |
| `verify_stage09_final.py` | The helper: `check`, `write-job`, `verify-job`, and `run_job` for the guarded job |
| `checks/run_checks.py` | F1–F11 |
| `checks/outputs/run_checks.txt` | Its output at seal |

**Replay** (from the repository root; about 20 s; W is root's geometry stage folder holding `logarithm-stage09/`):

```
python3 -I -B P/logarithm-stage09-final-verification01/checks/run_checks.py NEW_SCRATCH --w W
```
