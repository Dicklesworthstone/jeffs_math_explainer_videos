# Logarithm remote render and master

LavenderPike, 2026-11-05, for BlueSalmon (bead 87do.63, Mails 5281, 5359 and 5419). This folder replaces
`S/render-master01.py` on mac-mini-max; that file stays as it is. Nothing here ran on the remote, and nothing was launched.

- P = `docs/video-program/third-wave-20261024/opus-replacements-20261104`
- EL = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104`
- S = `EL/final-picture-repair01`
- O = `EL/remote-mac-mini-max01`

## How to run it

Run it after continuation03 finishes, so that `O/remote-binding01.json`, timeline01, score01 and both comparisons exist.
`O/tools01.json` already exists; this helper only reads it.

1. Transfer this whole folder to the same path. `--seal` refuses if any sealed file is missing.
2. In tmux on mac-mini-max, from the repository root:

```sh
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin"
helper=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-remote-final-production01/render_master_remote.py
seal=<the seal in the handoff mail>
/opt/homebrew/bin/python3 -I -B "$helper" check --seal "$seal"
/opt/homebrew/bin/python3 -I -B "$helper" run --seal "$seal"
```

The PATH line matters: the guard needs `sbh`. The helper refuses without it.

`check` writes nothing. It exits 0 when ready, 1 when refused and 3 when a prerequisite isn't finished yet. It also reports
each chapter's state and the plan as JSON. `run` makes 13 renders, then the master, one guarded job at a time. It exits 0
only after every output gate passes, and prints the master, SRT and VTT hashes.

| Situation | What to do |
|---|---|
| `schedule-comparison01.json` has rows in `not_as_predicted` | Read them, then pass `--approved-schedule-sha256 <that file's hash>` |
| A stem changed in a chapter whose schedule didn't change | Read `score-comparison01.json`, then pass `--approved-score-sha256 <that file's hash>` |
| Either comparison has `hard_failures` | No flag passes this. It stops. |
| A chapter or master attempt failed | `run` stops there and never retries by itself. Read the job's `command.log`, then add `--retry <chapter>=r2` or `--retry master=r2`. |
| The run was interrupted | Run the same command again. Chapters that passed and still hold their hashes are skipped. |

A retry moves that attempt's outputs to `O/jobs/logarithm-remote-render-aside01/<failed job>/` with a `moved.json`. Nothing
is deleted.

**Limit on a master retry:** verifier02 accepts `--master-job logarithm-remote-master01-r2`, but root's `O/asr01.py`
asserts the base name, so a retried master stops at ASR. It fails closed, and Bronze has reported it. A retried master
needs an ASR change that root decides on.

## What the review's defects became

| Defect | Fix |
|---|---|
| D1 resume | `status_of` skips only a job that passed and whose outputs verify. A failed attempt blocks. `--retry` names the exact next attempt. Job files are written once, and the driver holds a lock. `admission()` tests first, so a refused launch doesn't use up a job name. |
| D2 comparisons | `gather` waits for both comparison files and reads them. Hard failures refuse. Any reported delta needs root's exact file hash. |
| D3 old target rows | Omitted: `input_rows` holds no target or engine rows. |
| D4 bindings | The binding, the live binary, the narration, the timeline and score configs, every frozen row, and the crate and bound files are all compared, again before each job. |
| D5 movie hash | `verify_render` runs before a skip and before the master. The master compares each movie with its render summary before and after it runs. |
| D6 expected counts | The master log must show the kit's plan lines, the 13 prediction holds in map order, 13 replay lines, the master report and 364 sentences. The hold ends must appear in the captions, and the SRT and VTT must agree. |
| D7 captions after the master | Unchanged: captions still follow the master, and a named master retry recovers a failure. |

Every chapter and the master are also read back from the files: h264 High, 3840x2160, 60 fps, yuv420p, bt709, the exact
frame count, stereo 48 kHz AAC and x264's own record of CRF 16. Each chapter must also have the full cue table, the
timeline rows, the rendered lint and a whole soundtrack capture.

## Contract with Ruby and Bronze

| Item | Value |
|---|---|
| Stage | `logarithm-remote`, lane `O/jobs`, guard `O/guard01.py` |
| Jobs | `-build01`, one passing `-rebindNN`, `-assemble01`, `-timeline01`, `-score01`, `-render-<chapter>-01` (`-r2` to `-r9`), `-master01` (`-r2` to `-r9`) |
| Work folder | `O/jobs/logarithm-remote-render-work01` |
| Job files | `O/render-jobs01/<job>.py`, `.config.json`, `-inputs.json`. Each `.py` is `job_worker.py` byte for byte. |
| Read from Ruby | Binding schema `logarithm-remote-binding-v1`, `schedule-comparison01.json`, `score-comparison01.json`, `O/job-scripts/` |
| Read from Bronze's `tools` | `O/tools01.json`, schema `logarithm-remote-tools-v1`; the fetched file has hash 188b7650 |

**Rebind identity.** No job name is assumed, and failed rebind01 stays on the lane unread. The binding's `files` holds the
manifest whose hash is in the assembly receipt. That manifest must hold this binary and exactly one rebind receipt row. The
row must equal the receipt on disk, and that receipt must pass every gate, show phase `native` and end in
`rebind_audio.py --apply`. This is Bronze's rule in verifier02, including the binary row.

**`H.mentions` cross-check (Mail 5359).** Neither the driver nor the worker contains or calls that heuristic. The binary is
identified only by its hash.

**continuation03.** The binding changes Ruby listed in Mail 5390 (`remote_helper_sha256`, two more `files` rows,
`checks.compiled_build`) need no change here. Those two files must stay unmoved, because every bound file is rechecked.

## Two defects the checks found late

1. **The worker could die at the start of a job.** The preserved guard rewrites its live receipt in place just after it
   starts the job, and a rehearsal chapter read it mid-write and crashed with a JSON error. On the real lane that would
   fail a chapter for nothing. `live_receipt()` now rereads until the file parses, for at most about 10 s, then refuses.
   The driver already allowed for this.
2. **The read set missed a file.** The real delivery contract opens the hold audit named in the disposition evidence. The
   helper now walks all of `assembly.json` and `report.json`, not only the units.

## Evidence

All of this ran on the final bytes, on this host, under Python 3.14.6.

| Check | Result | What it uses |
|---|---|---|
| `checks/rehearse_remote_production.py`, in `checks/outputs/rehearsal-results.json` | 120 of 120 | The real driver, worker and guard source in a throwaway repository, with a stand-in binary, ffmpeg, ffprobe and sbh. Includes Bronze's real `bind_master`, `check_binding` and `check_rebind`. |
| `checks/stage09_parsers.py`, in `checks/outputs/stage09-parsers.json` | Accepts 13 holds, 13 replays, 364 sentences, 475 captions, 7547 and 116245 frames; 5 of 5 negative controls refuse | Stage09's real log, captions, one chapter and the master |
| `checks/read_set_trace.py`, in `checks/outputs/read-set-trace.json` | 250 files opened, 0 outside the frozen rows; 162 are covered only by the narration walk | The real delivery tool on stage09's narration |

The real `O/tools01.json` also matched the helper's schema and host, and the bound python3 folder resolves to that
python3.14, on this host's keg layout. Stage09's real outputs total 1.5 GiB across all 14 jobs, inside the guard's 4 GiB
growth cap per job.

## Limits

- **The rehearsal uses stand-ins.** Its media and binary are small fakes, so it tests the logic of these gates. It doesn't
  show that a real render passes them.
- **The parsers saw stage09's files**, not the repaired film's, which don't exist yet.
- **Rust kit reads aren't traced.** A file only the kit opens would first show up at runtime, as a failed job.
- **Ruby's real timeline and score job scripts weren't available here.** `check` reads them on the host and refuses with a
  reason if they differ, before any job starts.
- **The rehearsal ran once on these final bytes.** Root asked not to repeat passed checks, so there is no second run to
  compare.
- **Not independent.** I wrote the v4 and stage09 worker template, the rebind helper, the stage09 audio work,
  final-verification02 and this helper. This folder needs a separate review.
- **Technical gates only.** Nothing here is listening, picture review, fine sync or AV acceptance.
