# Final verification helper, version 02 (87do.63, root Mail 4376)

LavenderPike, 2026-11-05. `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`.

Root asked me to inspect BronzeJaguar's staged F1/F2/F3 fix and produce version 02, keeping version 01 and the review
intact. Version 02 adds:
- the real report key-order regression;
- missing and extra cue refusal;
- picture, sheet and caption hash checks;
- the decode results, recorded and checked.

The existing positive checks are kept, and no native job is launched.

**Status: ready for root to replay.** Version 02 is `verify_stage09_final.py` in this folder.

- **Earlier bundles:** version 01 (`P/logarithm-stage09-final-verification01`, `a0dc3d50`) and Bronze's review
  (`P/logarithm-stage09-final-verification-review01`, `def2f967`) are unchanged and still verify.
- **Checks:** `checks/run_checks.py` passes 15/15, byte-equal over two runs into fresh scratch folders.
- **Not run on stage09 outputs:** no stage09 master exists yet. Root's `logarithm-stage09-timeline01` was running,
  and nothing of it was read.
- **What it does not do:** launch a job, render, view a picture or listen. It accepts nothing.

## Inspection of Bronze's staged fix (`verify_stage09_final_fix01.py`, `ea9cd27f`)

- **The patch is what it says.** `verify_stage09_final_fix01.patch` equals `difflib` of the sealed version 01 against
  the fix (+20/−7, 4 hunks). The fix is version 01 plus that patch and nothing else.
- **F1 (the cue-set gate) is right.**
  - Root's real `prepared01/narration/report.json` (`77dd97b0…`) lists the 80 cues sorted as text. 17 of 80 positions
    differ from the script, starting at `06_20` and `11_20`.
  - Version 01's statement refuses that file. `sorted(…) == sorted(…)` accepts the same set and refuses a missing or
    extra cue.
  - Script order is still enforced where it matters: `read_schedule` requires the timeline placements in script
    order. The kit reads the report's lines as a map.
- **F2 (hashes) is right.**
  - `run_job` records each picture's `sha256` and the sheets' `sheet_sha256`.
  - `verify-job` re-hashes those and the SRT and VTT.
- **F3 (the decode record) is right on the cause.**
  - `verify_video_preserve_logs01.sh` (pinned `3f4c68db…`, lines 159–178) makes one `mktemp` log. Each decode rewrites
    it with `>`.
  - With `VERIFY_HWACCEL` unset, the script's `auto` runs videotoolbox second on macOS. The kept log then holds only
    the hardware decode, and the strict log's `full decode (…)` lines are the record of both.
  - Bronze's check matched `": 0 error lines, ffmpeg exit 0, "` anywhere in the first line. I tightened it (below).

Bronze's three hunks are in version 02 unchanged, except where the additions below touch them.

## What version 02 is

Version 02 is Bronze's fix plus these additions (`docs/review-fix01-to-v2.diff`):

| Finding | In Bronze's fix | Added here |
|---|---|---|
| F1 report order | `gather`: compare `sorted(report["lines"])` with `sorted(script)` | none |
| F2 evidence after the run | `run_job` records each picture's `sha256` and `sheet_sha256`; `verify-job` re-hashes them and the SRT and VTT | `verify-job` also requires verification.json's `srt_sha256` and `vtt_sha256` to equal the job's. A verification.json without the version 02 fields is refused, not a `KeyError` |
| F3 decode record | `run_job` records the strict log's `full decode (…)` lines as `strict_video_decodes`; `verify-job` requires the first to read clean | The first line must start with the script's exact software label: `full decode (video + audio, software, stops at the first error): 0 error lines, ffmpeg exit 0, `. `run_job` enforces this too, before any picture is taken. `verify-job` requires the recorded lines to equal the strict log's. `run_job` prints the lines to the job's log before the count and success lines |

Unchanged from version 01:
- the module and job names;
- the guard line;
- the success line;
- `check`'s output;
- `write-job`'s 141 input rows;
- the picture plan;
- Ruby's schema (`render-preparation02` keeps the step code of version 01, as Bronze and Ruby report).

Full deltas: `docs/v1-to-v2.diff` (helper) and `docs/checks-v1-to-v2.diff` (checks).

## Commands (from the repository root; every path absolute)

`W` and `WS` are root's, as before (`WS` = `$W/logarithm-stage09-workspace01`). The series is 01, the series of
root's running `logarithm-stage09-timeline01`. The three commands need the finished master job
`logarithm-stage09-master01`. Until it exists, `check` is refused at the master's receipt.

```
V=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-final-verification02/verify_stage09_final.py
L="$PWD/narration_lab/third-wave-20261024/root/five-film-release-20261027"
N="$PWD/narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/production-audio-stage09-01/prepared01/narration"
J="$WS/jobs"
C="$WS/explainer_corpus/logarithm_mysteries"

python3 -I -B $V check      --lane "$L" --jobs "$J" --crate "$C" --narration "$N" --series 01
python3 -I -B $V write-job  --lane "$L" --jobs "$J" --crate "$C" --narration "$N" --series 01 --verify 01 --job-dir "$J"
#   root launches the line write-job prints, of this form:
#   python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py \
#     --phase native --job logarithm-stage09-final-verify01 --inputs $J/logarithm-stage09-final-verify01-inputs.json \
#     -- python3 -I -B $J/logarithm-stage09-final-verify01.py
python3 -I -B $V verify-job --lane "$L" --jobs "$J" --crate "$C" --narration "$N" --series 01 --verify 01 --job-dir "$J"
```

- `N` is `audio.narration` in Ruby's `WS/narration-binding01.json` (her Mail 4329). The master's CONFIG must name the
  same folder.
- **`check`** is read-only. It prints the master's hash, the 13 chapters, 80 cues and total frames, the caption facts,
  the holds, the phrase marks, the layout findings, and the planned pictures and sheets. Bronze's N1 estimates at
  least 1020 pictures (255 sheets) for the real 1800.8 s of narration; `check` prints the real count.
- **`write-job`** writes `$J/logarithm-stage09-final-verify01.py` and its 141-row inputs file, then prints the guard
  line. It refuses an existing script, inputs file or job folder; use another `--verify` number then.
- **The job.** With `VERIFY_HWACCEL` unset in the guard's environment, root's strict check runs its videotoolbox second
  decode. Both decode lines then land in `verification.json` and in the job's log; F15 shows this on the real Nov 2
  master.
- **`verify-job`** checks the finished job:
  - its receipt, log and outputs;
  - every picture's and sheet's hash, and the SRT and VTT;
  - the decode record.

  It ends with the open items.
- **The sheets for root** are `$L/logarithm-stage09-final-verify01/sheet-NNN.jpg`.

## Evidence

The checks are read-only on the repository and W, and write only under a fresh scratch folder. They print in the order
F1–F7, F10, F12, F13, F8, F9, F11, F14, F15: F12 and F13 reuse F10's fixture, and F14 and F15 reuse F11's job.

| Check | Kind | What it shows |
|---|---|---|
| F1–F10 | as in version 01 | Unchanged, run on version 02: the W crate, real Nov 2 kit outputs (schedule, captions, master frames, 79 layout files, picture plan), planted captions and counts, command binding on Ruby's sealed template, the job's guard binding, the measurement parsers, and the composed command line on the fixture. F5 now leaves out the lane's `logarithm-stage09-*` folders, where root's live jobs write; it reads the same 79 files |
| F11 | real Nov 2 master, emulated guard receipt | Version 01's end-to-end run: strict check passed (`VERIFY_HWACCEL=none`), 17783 frames, I −19.7 LUFS, true peak −1.5 dBTP, 5 pictures in 2 sheets. Now also: every picture and sheet hash in `verification.json` equals the file, one decode line (the clean software one), and the log ends with the success line. The SRT and VTT it binds are scratch copies of the real files |
| **F12** | **root's real stage09 report** (read only) and F10's fixture | The real `report.json` lists the script's cues sorted, with 17 of 80 positions moved. Its exact bytes were bound into the fixture, with the master's inputs row and receipt re-hashed so only the report decides. **Version 02 `check` passes; version 01 `check` on the same files is refused** with `report.json's cues differ from the script`. Restored, version 02 passes again |
| **F13** | the real report, changed in scratch | Without `11_20`, without `00_1`, or with an extra cue `13_1`, each bound consistently: all refused by the cue-set gate |
| **F14** | F11's finished job | One change at a time, each restored. All refused, and verify-job passes again after each restore: <br>• a picture replaced by another's bytes; <br>• a sheet replaced by the other's bytes; <br>• verification.json without its sheet hashes; <br>• the SRT changed; <br>• the VTT changed; <br>• verification.json naming another SRT |
| **F15** | a second real run on the Nov 2 master, `VERIFY_HWACCEL=auto` (root's default) | The strict check ran both decodes (software, then videotoolbox), both clean. `verification.json` records the two lines, equal to the strict log's. The job keeps one decode log. Refused when planted in verification.json, and both jobs pass again after each restore: <br>• the software line reading 1 error line; <br>• ffmpeg exit 69; <br>• no decode line; <br>• the hardware line first; <br>• the hardware line dropped from the record |

The transcript replaces each decode's elapsed seconds with `<seconds>`, so runs compare byte for byte.

## Open for root

- **Root replays** the checks, reconciles the review findings, and then uses version 02 (Mail 4376).
- **Root views every sheet.** Listening, fine sync, final ASR and acceptance remain root's.
- **The phase.** `--phase native` follows Ruby's render jobs.
- **Bronze's N2.** An optional per-chapter `-count_packets` in `check` was not added. On the Nov 2 render, the master's
  total equals the timelines' sum and each chapter's own count (7/7).

## Limits

- **No production work:**
  - no Cargo, build, render, assembly, synthesis or recognition;
  - no guard launch;
  - no production decode.
- **No edits outside this folder:**
  - no root, canonical, sealed, Beads, TODO or memory file;
  - nothing in the live timeline, audio or binding.
- **The only ffmpeg work** was two runs of `run_job` on the existing Nov 2 master, plus F9's frame, all into scratch.
- **Root's production report was read only.** F12 and F13 bind copies of its bytes into the scratch fixture.
- **F7, F10, F12 and F13 use stand-in media,** labelled in their bytes. F12's planned-picture count comes from
  stand-in timelines, not stage09's.
- **Not independent.** I wrote version 01 and these checks. Bronze's fix is his; the additions are mine.

## Files

| File | What it is |
|---|---|
| `HANDOFF.md` | This note |
| `verify_stage09_final.py` | Version 02 of the helper |
| `checks/run_checks.py` | F1–F15 |
| `checks/outputs/run_checks.txt` | Its output at seal |
| `docs/v1-to-v2.diff` | The helper, version 01 to version 02 |
| `docs/review-fix01-to-v2.diff` | Bronze's staged fix to version 02: the additions only |
| `docs/checks-v1-to-v2.diff` | The checks, version 01 to version 02 |

**Replay.** Run from the repository root; it takes about 100 s. W is root's geometry stage folder holding
`logarithm-stage09/`.

```
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-final-verification02/checks/run_checks.py NEW_SCRATCH --w W
```
