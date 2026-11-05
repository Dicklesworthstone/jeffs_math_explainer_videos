# logarithm-stage09 render preparation 04

LavenderPike, 2026-11-05. Bead 87do.63; root Mails 4524, 4539 and 4552. `P` =
`docs/video-program/third-wave-20261024/opus-replacements-20261104`, `L` = root's lane
(`narration_lab/third-wave-20261024/root/five-film-release-20261027`), `W` = root's geometry stage folder.

**What this is.** Version 4 of `prepare_stage09_render.py`: a copy of RubyRaven's sealed version 3
(`P/logarithm-stage09-render-preparation03`, SHA256SUMS `aa645803`, helper `6e5a602d`) with the three narrow repairs
root asked for, and nothing else. Versions 2 and 3 are untouched (check A4).

**Status.** Sealed for root. It is not independently reviewed: RubyRaven reviews it next (Mail 4552).

**What I did not do.**
- I ran no job, Cargo, kit, guard, render, score or master, and launched nothing native.
- I changed no live audio, binding, timeline, job or receipt. Root's failed timeline01 was read only (S2, U1).
- I edited no Beads, TODO or memory file, and no file outside this folder.

## What changed from version 3

`docs/v3-to-v4.diff` lists every changed line: 57 added, 5 removed, in 8 hunks. Check X0 proves the list below is
complete by comparing every helper function, every job-template function, every job timeout and every module constant
with version 3's.

| # | Change | Where (version 4) | Root's words |
|---|---|---|---|
| 1 | Timeline kit run timeout 2400 s → 7200 s | job template `timeline()`, line 762 | "timeline timeout7200" |
| 2 | Render job refuses a capture that is not the whole soundtrack | job template: `import wave` (671), `frame_to_sample` (723), `incomplete` (728), 5 lines in `render()` (821–825) | "earlycapture exact48k/s16/stereo/completeexpectedframes check" |
| 3 | Master `verify-step` refuses, instead of raising `KeyError`, when the master's CONFIG has no `soundtracks` or a render summary has no `soundtrack` | `verify_job`, lines 632–638 | "clear missing-soundtracks refusal" |
| – | Binding schema `…-binding-v3` → `…-binding-v4` | `bind`, line 370 | consequence of a new helper hash |
| – | Writer refusal text "not version 2 or 3" → "not version 2 or 4" | `written_by`, line 441 | consequence |
| – | Docstring: a version 4 header above version 3's description | lines 2–24 | |

**Unchanged (X0, S1):**
- 31 of 34 helper functions and 14 of 18 job-template functions are byte-identical to version 3's. `bind` and
  `written_by` differ only in the two strings above.
- All 31 module constants, including the success lines, `SCOPE`, the pinned hashes and `VOICE`.
- Every other job timeout: coherent verify 900 s, preview 14400 s, score 7200 s, render 43200 s, master 14400 s,
  captions 1200 s.
- Resource guards: the jobs still run only under root's guard, and every job's input rows are written as before.
- Bind's audio checks, job names, argv, render work folders and the master's capture checks.
- Jobs written by version 2 (`b867aaa2`) are still accepted (A5, T1).
- Jobs written by version 3 are not accepted. None exist: root ran only version 2's timeline01.

### Repair 2: the capture completeness rule

**The problem (render03-review01, L1).** The kit copies each ffmpeg job's `soundtrack.wav` to
`WORK/soundtrack/<CH>/soundtrack_<n>.wav` and only prints a warning if the copy fails (`encode.rs:200–203`).
Version 3's render job tested only that the chosen capture was a file. A header-only or half-written capture
therefore passed the render. It would only have failed at the master's null test, after all 13 renders.

**The new rule.** After its existing checks, the render job opens the capture the kit's master will choose (the
same rule as version 3). It refuses unless all of these hold:
- the format is 48 kHz, 16-bit, 2 channels;
- the header's frame count equals `frame_to_sample(frames, fps)`, which is `explainer_score::units::frame_to_sample`
  (800 samples per frame at 60 fps);
- the data actually holds that many frames.

`frames` and `fps` come from the render's own `CH.timeline.tsv`. The job has just checked that file's schedule
against the 60 fps timeline. An unreadable file is refused too.

**What a refusal looks like.** The job prints one `REFUSED:` line naming the capture and every measured value, exits
1, and writes no `render-summary.json`. For example (E3):

```
REFUSED: 03_folded_decade: the soundtrack capture soundtrack_1.wav is not the whole soundtrack the master's null test needs: 48000 Hz, 2 channels, 16 bits, 768000 frames in the header and 384000 present; the chapter needs 48000 Hz, 2 channels, 16 bits and 768000 frames
```

Recovery is the existing procedure: move `CH.mp4` aside, then write and launch `--retry N`. The kit itself sets the
failed attempt's capture folder aside (E4).

**Cost.** The rule reads the capture once. The longest chapter (11, 248.4 s of narration) is about 48 MB.

## Timeout evidence (repair 1)

**What failed.** Root's timeline01 ran the kit's whole-film `render all --timeline-only` as one process. It hit 2400 s
during chapter 09, after nine chapters (receipt exit 1 at 07:06:39Z). A timeout loses the whole run, because the kit
step cannot resume chapter by chapter.

**Kit time per chapter**, from the kit's own lines in timeline01's preserved `command.log`:

| Chapter | Frames | Seconds | fps |
|---|---:|---:|---:|
| 00 | 7,547 | 229.5 | 32.9 |
| 01 | 8,060 | 96.1 | 83.9 |
| 02 | 5,575 | 63.5 | 87.7 |
| 03 | 4,679 | 132.9 | 35.2 |
| 04 | 11,725 | 397.5 | 29.5 |
| 05 | 10,811 | 266.3 | 40.6 |
| 06 | 10,571 | 592.0 | 17.9 |
| 07 | 10,191 | 393.6 | 25.9 |
| 08 | 8,738 | 180.3 | 48.5 |
| **00–08** | **77,897** | **2,351.7** | **33.1** |

**Projection for chapters 09–12.**
- The nine finished chapters have 62.4–65.6 timeline frames per second of narration (64.3 overall).
- Chapters 09–12 have 590.0 s of narration (88.8, 98.5, 248.4 and 154.3 s), so about 37,960 frames.
- The whole film is therefore about 115,860 frames.

| Assumption | Projected kit time | Margin under 7200 s |
|---|---:|---:|
| 09–12 at the measured overall rate, 33.1 fps | about 3,500 s | 2.06× |
| 09–12 at the slowest chapter's rate, 17.9 fps | about 4,470 s | 1.61× |
| The whole film at 17.9 fps | about 6,470 s | 1.11× |

The last row is the worst case: the margin is thin if the host is as loaded for the whole run as it was during
chapter 06. The coherent verify runs before the kit, under its own 900 s timeout.

**Root's guard has no wall-clock limit.** It stops a job only on its resource limits, so the job's own timeout is the
only time bound.

**The other steps' timeouts are unchanged, as root instructed (Mail 4539).** For reference only, here is a rough
extrapolation from the Nov 2 logarithm film. That film had 17,783 frames, and its jobs took: timeline 210 s, score
188 s, renders 207–600 s, master 166 s.
- Stage09 has 6.5× as many frames.
- timeline01 ran 2.6× slower per frame than the Nov 2 timeline (33.1 against 84.7 fps).
- Scaling by both gives: score about 3,100 s (2.3× under 7200 s); master about 2,800 s (5.2× under 14400 s);
  chapter 11's render about 9,700 s (4.5× under 43200 s).

None of these extrapolations is measured. After the timeline's, score's margin is the smallest.

## Commands for root

Run these from the repository root. `O2` is root's corrected prepared audio (absolute path) once root has made it.

```
H=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-render-preparation04/prepare_stage09_render.py
WS="$W/logarithm-stage09-workspace01"
python3 -I -B $H check-audio --w "$W" --ws "$WS" --audio "$O2"                                # writes nothing
python3 -I -B $H bind        --w "$W" --ws "$WS" --build 02 --audio "$O2" --attempt 03        # writes WS/narration-binding03.json
python3 -I -B $H step        --w "$W" --ws "$WS" --binding 03 --series 02 --step timeline     # prints the guard line
```

Then, in order, verifying each step before writing the next:
1. `verify-step … --binding 03 --series 02 --step timeline`;
2. `step`/`verify-step … --step score`;
3. `step`/`verify-step … --step render --chapter CH` for the 13 chapters (`--retry N` after a failure, with the
   earlier `CH.mp4` moved aside);
4. `step`/`verify-step … --step master`;
5. final-verification02 with `--series 02` and `--narration "$O2/narration"`.

**Notes:**
- `narration-binding01.json` (version 2) and `narration-binding02.json` (version 3) stay as they are. Version 4
  refuses binding02 as another helper's (A6), so a version 4 binding is needed even for the same audio.
- timeline01 stays as it is. Version 4 refuses it by its receipt (exit 1; A6), so series 02 starts with its own
  timeline. Series 02 writes its chapters to the new `L/logarithm-stage09-render-work02`.
- `check-audio` and `bind` apply version 3's audio checks, unchanged. I could not run them on the corrected audio
  because it does not exist yet. If they refuse it, the refusal comes from those unchanged checks; please send it to
  me rather than editing this bundle.
- final-verification02 already takes `--series`, pins no render-helper hash, and reads only keys that version 4
  writes exactly as version 3 does. render03-review01's K4b showed it accepts a version 3 master.

**For the final verifier.** The schema is version 3's. Only these names change: the binding row
`WS/narration-binding03.json`, the series-02 job names, `render-work02`, and the job script headers, which name
version 4's hash (in `SHA256SUMS` here).

## Checks: 33 PASS, 0 FAIL

These are RubyRaven's version 3 checks, re-pointed at version 4, with the new checks added. Her stand-in binary now
writes real 48 kHz 16-bit stereo WAV captures as long as the chapter. Without that, every stand-in render would fail
the new rule. Two runs into differently named fresh scratch folders gave byte-equal transcripts. The first line of
`checks/outputs/run_checks.out` names this helper's and these checks' hashes.

| Check | What it shows |
|---|---|
| X0 | The diff from version 3 is exactly the table above. Every other function, timeout and constant is version 3's. |
| A1 | **Positive check on actual old kit output:** all 7 real Nov 2 captures pass the new completeness rule. The kit's set-aside copy is not chosen. |
| A2–A5 | Real prepared01 passes `check-audio`; real ws-build02 passes `build-verify`. The seals of versions 2 and 3 verify. Root's timeline01 was written by version 2, which version 4 still accepts as a writer. |
| A6 | Real timeline01 is refused by its receipt (exit 1), and real binding02 is refused as version 3's. |
| C1 | The rule on planted captures: a whole one passes. Header-only, half-written, one frame short, one frame long, mono, 44.1 kHz, 24-bit and not-a-WAV are each refused, with the measured values. |
| B0–B1, T1–T3 | As in version 3: version 4 binds beside version 2's binding and takes over version 2's timeline01 in scratch; other writers are refused. |
| R1–R3, G0–G3, M1–M4 | Version 3's capture checks, unchanged in result: 13 renders (one refused, then retried), the gap rule, a changed byte, and the master. All of them now run on real WAV captures. |
| N1 | Repair 3: a consistently bound master CONFIG without `soundtracks`, and a render summary without `soundtrack`, each give a `REFUSED:` line with exit 1 and no traceback. Restored, the master passes again. |
| E1 | Series 02 from version 4's own binding03. The timeline job's script runs the kit with 7200 s, and "2400)" does not appear in it. The job runs and verifies, and its writer is version 4. |
| E2, E3 | **End to end:** a render whose last capture is header-only, and then one that is half-written, each fails in its own job (exit 1, no summary). `verify-step` refuses both. |
| E4 | The retry with a whole capture passes and verifies. The kit's set-aside copies of the two failed attempts are not chosen. |
| E5 | Control: both refused captures are files, which is all version 3's render job tested, so version 3 would have recorded either one. |
| S1 | Everything version 3 kept from version 2 is still version 2's; `timeline` differs only in its timeout. |
| S2, U1 | Scratch timeline01 is unchanged. On the real side: prepared01 and its timings, the Nov 2 render work, the version 2 and 3 bundles, root's failed timeline01 folder and the installed helper are all unchanged, and no new binding or job was added. |

**Replay.** Run from the repository root; it takes about 35 s. SCRATCH must be a new absolute folder outside the
repository. It needs about 0.3 GB, because the stand-in captures are full-length WAVs.

```
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-render-preparation04/checks/run_checks.py SCRATCH --w W
```

## Limits

- **The kit is stood in for.** The end-to-end checks use a stand-in binary under an emulated guard. The only real
  kit output the completeness rule was tested on is the 7 Nov 2 captures (A1); no stage09 capture exists yet.
- **7200 s is a projection** from one partial run, with the margins given above.
- **Not independent.** I wrote this version 4 and adapted its checks. RubyRaven wrote version 3 and the original checks.
- **No listening, viewing, fine sync or acceptance is claimed.**

## Files

| File | What it is |
|---|---|
| `prepare_stage09_render.py` | Version 4 |
| `docs/v3-to-v4.diff` | Unified diff from version 3's helper |
| `checks/run_checks.py` | The 33 checks |
| `checks/outputs/run_checks.out` | Their output at seal |
| `HANDOFF.md` | This note |
