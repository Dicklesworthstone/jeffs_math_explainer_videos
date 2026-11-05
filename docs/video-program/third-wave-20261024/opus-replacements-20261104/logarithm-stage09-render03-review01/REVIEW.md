# Review of render-preparation03's soundtrack capture binding (87do.63, root Mail 4467)

LavenderPike, 2026-11-05. `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`.

**Subject:** RubyRaven's sealed `P/logarithm-stage09-render-preparation03` (SHA256SUMS `aa645803`, helper `6e5a602d`).
I reviewed it against the kit code that ws-build02 compiled and against the real Nov 2 logarithm render and master.

**Bounded source-review result for the capture-binding change: clean.** Within that change I found no defect that would
change or block final production. There is one Low hardening item (L1) and one nit (N1). This is a source and scratch
review only; it is not AV acceptance.

**Not covered: the unchanged code's adequacy.** I did not review the code version 3 keeps from version 2 for adequacy.
I therefore missed the timeline step's 2400 s timeout, which stopped root's timeline01 (see the next section).

**What I did not do:**
- I launched, built, rendered and scored nothing, and ran no Cargo, kit, guard or native job.
- I changed no live input.
- I did not read the lane folder of root's running `logarithm-stage09-timeline01`. K6 reads its job script and inputs
  file only.
- I edited no Beads, TODO or memory file, and no file outside this folder.

## After this review: timeline01 timed out (root Mail 4524)

Root reports that `logarithm-stage09-timeline01` exited 1 at 07:06:39Z. Its kit run hit `subprocess.TimeoutExpired`
at 2400 s, after nine chapters had completed.

- **Where it comes from.** The timeout is the job template's `run(CONFIG["argv"], 2400)` in `timeline()`
  (`prepare_stage09_render.py:715`). This is version 2 code that version 3 keeps unchanged (Ruby's S1).
- **What changes.**
  - Root intends to apply Bronze's timing-corrections01 through a new prepared audio. That needs a new binding and a
    series 02, written by Ruby's render-preparation04, which raises the timeline timeout to 7200 s.
  - K6's handover path, in which version 3 consumes timeline01 without a rerun, will therefore not be used. K6 remains
    a true statement about timeline01's job files.
- **What follows.** I will review the timeout-only 04 delta once Ruby seals it, in a new folder that keeps this one
  intact. It will check every step's timeout against measured run times.

## What the change does, read against the kit

**The kit** (W `repo-view`, the bytes ws-build02 bound; K1):
- **Capture** (`encode.rs:187–204`, `cli.rs:666–687`):
  - `render <CH> --crf` gives fmn's ffmpeg a runner that copies each job's `-i …/soundtrack.wav` into
    `WORK/soundtrack/<CH>/soundtrack_<n>.wav` before that job runs. The counter starts from 0 for each runner.
  - Each attempt of the transient-EPERM loop (`cli.rs:639–712`) first moves an existing `soundtrack/<CH>` to
    `soundtrack/previous/<CH>.vN`. It then builds a fresh runner and resets the narrator. The cue log is written after
    the loop (`cli.rs:750`), so it belongs to the attempt that succeeded.
  - A failed copy is only a warning (`encode.rs:200–203`).
- **Choice** (`cli.rs:987–1000`): `master` null-tests the last `soundtrack_<n>.wav` before the first gap, and requires
  the replay of the cue log to equal it byte for byte (`master.rs:126–154`). The master's audio is the replay; the
  capture is the evidence that the replay is what fmn muxed.
- **The logarithm crate** calls `run_with_contract`, so no film-side options hook replaces the delivery runner that
  carries the capture folder (K1).

**Ruby's version 3:**
- `kit_capture` (`prepare_stage09_render.py:392–399`) and the job's `capture` (`:689–695`) implement exactly that choice.
- The render job records the chosen capture's path, hash and count (`:771–778`).
- `verify-step` re-checks the choice and the bytes (`:402–409`, `:605`).
- `output_rows` adds the capture to the rows a later step freezes (`:467`), so the master's inputs bind all 13 captures.
- The master job refuses before and after the kit if any chapter's choice or bytes changed (`:791–802`). It records
  what it consumed, and `verify-step` checks that record against CONFIG and each render summary (`:611–616`).
- `written_by` (`:412–424`) accepts root's version 2 timeline01 without a rerun.

Every hunk of `docs/v2-to-v3.diff` was read against this.

## Findings

### L1 (Low, hardening): the render job records any bytes as the capture

**The cause.** The job's only test of the capture is that a file exists (`prepare_stage09_render.py:771–773`).
`verify-step` then checks only that the path and hash are unchanged.

**Failure scenario.**
- The kit copies the WAV with `std::fs::copy` and only warns when the copy fails (`encode.rs:200–203`).
- A copy that fails partway, for example on a full disk, leaves a partial `soundtrack_0.wav`.
- The render job exits 0 and records the partial file, and `verify-step` accepts it.
- The fault first shows when the master's null test fails, after all 13 renders.

**The cost.**
- That chapter needs a `--retry`.
- A new master is also needed. `write_step` refuses an existing `logarithm-stage09-master01` script, inputs file or
  folder, so root has to move those aside or start a new series.

**Fix (optional, about 6 lines in the job's `render()`).** Refuse unless the capture is a 48 kHz, s16, stereo WAV with
exactly `frame_to_sample(frames, 60)` frames, all present. This is the rule Ruby gives in her HANDOFF.
- K5 shows that the job accepts a header-only capture and a half-written one, and that this rule refuses both.
- K3 shows that all 7 real Nov 2 captures pass it.

**Correctness is not at stake.** The null test fails closed.

### N1 (nit): a master CONFIG without `soundtracks` crashes `verify-step`

`verify_job` reads `cfg["soundtracks"]` without a check (`prepare_stage09_render.py:611`). A master written by version 2,
whose writer `written_by` accepts, therefore stops with `KeyError: 'soundtracks'` instead of a `REFUSED:` line (K7).

**Reachability:** no version 2 master exists and none is planned.

**Fix:** `need("soundtracks" in cfg, f"{job}: written before version 3")` before line 611.

## Evidence

All results are from `checks/review_checks.py`: **10/10 PASS**. Two runs into differently named fresh scratch folders
gave byte-equal transcripts. The sealed transcript is `checks/outputs/review_checks.txt`.

| Check | Kind | What it shows |
|---|---|---|
| R0 | seals | render-preparation03 (5/5), render-preparation02 (5/5) and final-verification02 (7/7) verify at their sealed hashes |
| K1 | the compiled kit sources, read-only | The bytes ws-build02 bound give the capture and choice described above. `encode.rs` and `master.rs` equal the repository's. In `cli.rs` the capture block and `master_video` equal the repository's; the files differ only in the film options hook, which the logarithm `main.rs` does not use |
| K2 | planted layouts in scratch | The helper's and the job's rule give the kit's choice on 11 layouts: no folder, an empty folder, 0, 0–2, a gap, no 0, a kit set-aside copy, a dangling link, a link to a file, a directory, and a padded `00`. The directory case is then refused by `check_capture` and by the job. The expected answers are read from `cli.rs:988–991`; the Rust is not executed |
| K3 | **real Nov 2 output and logs**, read-only | Chapter 04's render job hit a transient EPERM on attempt 1. The kit moved that attempt's capture folder to `previous/04_principal_product.v1` and rendered attempt 2. The master's log then null-tests all 7 chapters "bit for bit", with sample counts equal to `frame_to_sample(frames, 60)`. Both rules choose each chapter's `soundtrack_0.wav`, never the set-aside copy. Each is whole: s16, stereo, 48 kHz, the full length present. The file times run set-aside, then capture, then mp4, then master |
| K4a | Ruby's checks, replayed | 24/24, with a transcript byte-equal to her sealed one |
| K4b | final-verification02 on Ruby's scratch v3 master | My `bind_master` accepts the v3 master: 401 bound rows, 13 of them captures. With one capture byte changed, it refuses ("1 bound inputs moved since it ran"), and so does v3's `verify-step`. Restored, both accept |
| K5 | L1, planted in scratch | The job's test accepts a whole, a header-only and a half-written capture. The length rule accepts only the whole one |
| K6 | **root's live timeline01**, job script and inputs file only | It meets what v3's `verify-step` checks before its receipt. Its narration equals binding01's and what v3's `bind` derives from root's `--audio O`. Its binary is binding01's ws-build02 binary. Its header names version 2 (`b867aaa2`), whose bytes its 239 rows bind, and row 0 is the script |
| K7 | N1, in Ruby's scratch lane | A master CONFIG without `soundtracks`, bound consistently, gives a traceback and exit 1. Restored, the step exits 0 |
| U1 | | The 79 files read outside scratch are unchanged, in bytes (sizes and times for the large Nov 2 mp4s) and modification times |

**Final verifier.** final-verification02 needs no change for version 3 (K4b):
- It reads only keys version 3 keeps.
- Its re-hash of every bound master input now also covers the 13 captures.
- Root's `verify-step --step master` checks the captures against the render summaries before final verification.

## Limits

- **Not run on production.**
  - No stage09 render, capture or master exists yet.
  - Timeline01 was running. Its receipt, log and summary can only be checked after it exits.
  - Nothing here is evidence about final pictures or sound.
  - No listening, viewing, fine sync or acceptance is claimed.
- **What stands in for the kit.**
  - The real kit's capture and null test are evidenced only by the Nov 2 logs and files (K3).
  - Ruby's end-to-end path ran a stand-in binary under an emulated guard (K4).
- **Independence.**
  - I did not write version 3 or its checks.
  - K4b exercises my own final-verification02, so that check is not independent of my code.

## Files

| File | What it is |
|---|---|
| `REVIEW.md` | This note |
| `checks/review_checks.py` | R0, K1–K7, U1 |
| `checks/outputs/review_checks.txt` | Its output at seal |

**Replay.** Run from the repository root; it takes about 30 s, including Ruby's checks. SCRATCH must be a new
folder outside the repository, and W is root's geometry stage folder.

```
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-render03-review01/checks/review_checks.py SCRATCH --w W
```
