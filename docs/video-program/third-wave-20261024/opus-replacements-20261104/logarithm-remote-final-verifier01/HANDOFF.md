# Remote final verifier for the repaired logarithm master

BronzeJaguar, 2026-11-05, for BlueSalmon, bead 87do.63. Answers Mail 5282. Source and helper preparation only.

**Nothing here has run on mac-mini-max or on a repaired master.** No SSH, no job, no render, decode, ASR, build or
synthesis. No Beads, TODO, memory or production files were touched. All check groups pass, but they run on this local
host against stage09 records and stand-ins, so they validate helper behaviour only. **The helper accepts nothing; real
final media review remains open.**

The sealed `logarithm-final-picture-verifier01` is untouched and still verifies 8/8.

## Paths

| Short | Path |
|---|---|
| R | repository root, the working directory |
| P | `docs/video-program/third-wave-20261024/opus-replacements-20261104` |
| O | `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01` |
| S | `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/final-picture-repair01` |

## What root does on the worker

**1. Transfer** this whole folder, plus the two files in `transfer-extra01.json`, to the same repository-relative paths.
Those two (the strict check and the caption screen) are in none of your transfer records, and Ruby's
`transfer-files02.txt` doesn't list them either. The other 4 helper pins are already in your records at the pinned hash.

**2. Measure the tools, once,** before any render, since Lavender's master reads the same file. `tools` is the only
writer of `O/tools01.json`. Use the interpreter the jobs will use:

```sh
/opt/homebrew/bin/python3 -I -B P/logarithm-remote-final-verifier01/verify_remote_final.py tools \
  --out "$PWD/O/tools01.json" \
  --ffmpeg /opt/homebrew/Cellar/ffmpeg/8.1.2_1/bin/ffmpeg --ffprobe /opt/homebrew/Cellar/ffmpeg/8.1.2_1/bin/ffprobe
```

It refuses off mac-mini-max, refuses an existing file, and refuses a Pillow older than 10.1. The file records what the
host actually has: `python3` as launched, Python and Pillow versions, and each tool's path, hash and `-version` line.

**3. After the master passes,** from R on the worker. Replace P, O and S with the paths above; `set --` works in both
zsh and bash.

```sh
V="$PWD/P/logarithm-remote-final-verifier01/verify_remote_final.py"
PY=/opt/homebrew/bin/python3
set -- --lane "$PWD/O/jobs" --jobs "$PWD/O/render-jobs01" --series 01 \
  --crate "$PWD/S/workspace/explainer_corpus/logarithm_mysteries" \
  --narration "$PWD/O/jobs/logarithm-remote-assemble01/narration01" \
  --integration "$PWD/S/integration.json" --binding "$PWD/O/remote-binding01.json" --tools "$PWD/O/tools01.json"

$PY -I -B "$V" check "$@"                                                     # read only, every gate
$PY -I -B "$V" write-job "$@" --verify 01 --job-dir "$PWD/O/job-scripts"      # prints the guard line
# root launches the printed line, from R, on the worker
$PY -I -B "$V" verify-job "$@" --verify 01 --job-dir "$PWD/O/job-scripts"     # after the guarded run
```

`--job-dir` is my suggestion; any folder works. `write-job` refuses an existing script, inputs file or job folder, so a
second attempt needs `--verify 02`. The printed line runs `O/guard01.py --phase native --job logarithm-remote-verify01`.
`verification.json` lands at `O/jobs/logarithm-remote-verify01/verification.json`, where root's `asr01.py` reads it.

**4. Fetch for review.** Viewing happens off the worker. `verify-job` prints `verification_sha256`, and
`verification.json` holds `sheet_sha256` for every sheet, so compare the fetched files against those. The sheets are
`sheet-NNN.jpg` in the job folder.

## What changed from the sealed verifier

`docs/final-picture-to-remote.diff` is the whole change, and G1 proves it matches the helper exactly.

**Kept:** all 636 anchors, with `anchors.json` byte-identical. All parsers, measurement, sheets, captions, strict flow,
`EXPECT`, `OUTPUTS`, all earlier pins, and the hold, closing, source, binding, map and audio gates. G1 lists every
identical definition in `checks/outputs/run_checks.txt`.

**Changed or added:**

| What | Now |
|---|---|
| Names | stage `logarithm-remote`, schema `logarithm-remote-binding-v1`, printed guard line on `O/guard01.py` |
| Host | every command and the job refuse off mac-mini-max |
| Tools | python3, ffmpeg and ffprobe come from `O/tools01.json`, re-hashed live, and never from a PATH lookup |
| Strict check PATH | set explicitly: pinned folders, then `/usr/bin:/bin`. It refuses if another ffmpeg or ffprobe would answer |
| Pillow | proved at `tools`, `check`, `write-job` and job start. Sheets run last, so a failure there would waste the job |
| Same ffmpeg | the master's `tools_sha256` must equal the tools file used here |
| Rebind | `logarithm-remote-rebind01` must pass, and the order is now build, rebind, assembly, timeline, score, renders, master |
| Timeline bytes | render copy, timeline job copy and summary hash must be byte-equal. Before, only the parsed schedule was compared |
| Master binds | now also required: `integration.json`, the unit plan and the map |
| Comparisons | `O/schedule-comparison01.json` and `O/score-comparison01.json`: any hard failure refuses; `not_as_predicted` is reported |
| Render attempts | Lavender's `(-r[2-9])?` replaces `(-r[1-9])?` |
| `verification.json` | adds `worker`, `tools_sha256`, `comparisons`. Every earlier key stays |

The timeline byte gate exists because your `asr01.py` compares those hashes exactly. A difference now stops here, not
after a long job. On all 13 stage09 chapters the four hashes were equal.

## Contract

| Item | Value | Confirmed by |
|---|---|---|
| Lane, guard | `O/jobs`, `O/guard01.py` | root, Mail 5282 |
| Build | `logarithm-remote-build01`, `O/logarithm_mysteries-release01`, `O/build-inputs01.json` | root; real manifest read |
| Rebind, assemble | `logarithm-remote-rebind01`, `logarithm-remote-assemble01` | Ruby, sealed source read |
| Narration | `O/jobs/logarithm-remote-assemble01/narration01` | Ruby, Mail 5297; root, Mail 5295 |
| Binding | `O/remote-binding01.json`, keys as read from Ruby's `gates()` | Ruby, sealed source read |
| Timeline, score | `logarithm-remote-timeline01`, `-score01`, success lines unchanged | Ruby, Mail 5297 |
| Comparisons | beside the binding, in `O` | Ruby, sealed source read |
| Render, master | `logarithm-remote-render-<chapter>-01` with `(-r[2-9])?`, `logarithm-remote-master01` | Lavender, Mail 5296 |
| Render work | `O/jobs/logarithm-remote-render-work01` | Lavender, Mail 5296 |
| Job scripts | `O/render-jobs01`, sidecar config line unchanged | Lavender, Mail 5296 |
| Success lines, summaries | unchanged, keys only added | Lavender, Mail 5296 |
| Tools file | `O/tools01.json`, written only by `tools` | root, Mail 5290; Lavender, Mail 5296 |
| Verify job | `logarithm-remote-verify01` | root; matches `asr01.py` |

## Traps

- **Run `tools` once, before render.** If it is rerun with different bytes, `tools_sha256` differs from the master's and
  `check` refuses, correctly.
- **A Homebrew change on the worker** refuses at `check` or job start, because tools are pinned by hash. That also covers
  an `ffmpeg` upgrade between master and verify. It's a real difference, so re-measure and re-master rather than patch.
- **`/opt/homebrew/bin/ffmpeg` is a link** to the Cellar path. Pass either; the file records the resolved path.
- **Any crate edit after binding** refuses, because `check` compares the binding's crate tree.
- **`check` re-hashes every master row,** including the 4K chapters, so expect minutes.
- **An anchor crossing its next cue refuses by design.** Fix `anchors.json` in a new path rather than widening the check.
- **Local disk:** each check run writes about 2 MB, with WAVs by read-only link. That's within your 0.8 GiB warning.

## Checks

From R, on this host, with a fresh scratch path:
`python3 -I -B P/logarithm-remote-final-verifier01/checks/run_checks.py SCRATCH`. The transcript is
`checks/outputs/run_checks.txt`.

| Group | Evidence kind | Shows |
|---|---|---|
| G1 | sealed verifier against this one, by AST | exact changes; sealed bundle intact; anchors byte-identical |
| G2 | real stage09 records | all 1356 sealed pictures reproduced, reasons and order included |
| G3 | real stage09 records | all 494 flagged frames come back from their bases |
| G4 | real stage09 records | every old frame and reason kept, anchor first |
| G5 | **synthetic** shifted schedule | anchors move with it; `was` changes nothing |
| G6 | planted | 15 resolver faults refused |
| G7 | real holds, then planted | 15 holds; 18 hold and closing faults refused |
| G8 | **stand-in** remote-layout fixture | the real HOST refuses here; planted faults each stop at their expected gate |
| G9a | **stand-in** | `check`, `write-job`, repeat and early `verify-job`, through the real `main` |
| G9b | **stand-in** tools, 20 blank pictures | `run_job` and `verify-job` flow; PATH gives exactly the pinned tools; root's real `bound_proof` accepts the output |
| G10 | delivered files | no local paths |
| G11 | root's real local records | build manifest pins all 6 sources; the 2 files to send are exactly `transfer-extra01.json` |

G8 and G9 patch `HOST` inside the check process only. The helper has no override, and the real value is shown refusing
first.

## Limits

- **Stand-ins throughout.** The worker's Python 3.14, ffmpeg 8.1.2 and Pillow 12.1.1 were never exercised here; these
  checks ran under local Python 3.15. `run_job` used stand-in ffmpeg, ffprobe and strict check. The media code is the
  sealed verifier's, which itself was never run on a repaired master, but the same steps did run on the stage09 master.
- **Real outputs don't exist yet:** no remote binding, `tools01.json`, oliver17, narration, timelines, renders or master.
- **Closing timing is unmeasured:** 330 frames of hold, then exactly 90 of fade, from the kit source. A surprise refuses
  with the measured numbers.
- **Passing proves technical facts only.** No frame was viewed or sound heard. Independent picture review, listening and
  final ASR remain open.
- **Not independent.** I wrote the O1, O2, O3 repairs, reviewed sheets 113–225, and wrote this helper's sealed parent. An
  independent reader should check it.
- **No Rust was compiled for this work.**
