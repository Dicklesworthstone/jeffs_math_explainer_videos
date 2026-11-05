# Remote ASR, revision 02

BronzeJaguar, 2026-11-05, for BlueSalmon, bead 87do.63. This answers the ASR corrections assignment and Mails 5451 and 5452.

**Nothing here has run on mac-mini-max.** No SSH, launch, render, build or production edit. `fw transcribe`, the real guard
and the real verifier were not run. All checks use stand-ins on this host, so they prove the helper's logic only.
`O/asr01.py` and `O/asr-source-bindings01.json` are unchanged. **ASR isn't hearing proof: listening and fine sync remain
separate and open.**

- P = `docs/video-program/third-wave-20261024/opus-replacements-20261104`
- O = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01`

## Start it

Transfer this folder to the same path. Then, from the repository root:

```sh
bash P/logarithm-remote-asr02/start_asr02.sh logarithm-remote-verify01 logarithm-remote-master01 02
```

- **Arguments:** the verify job, the master job, then an attempt number of 02 or higher. Name a retry directly, for example
  `logarithm-remote-verify02 logarithm-remote-master01-r2 03`.
- **Timing:** it can start before verify finishes, because it waits on the exact records below.
- **After any refusal:** read `O/asrNN.log`, then run with the next number. Each number is used once, and nothing is replaced.

| Step | What happens | Stops on |
|---|---|---|
| Before logging | Checks names, unused paths, `sbh` on PATH and the pinned interpreter | any failure, nothing used |
| 1 Wait | Reads the selected master and verify receipts, then `O/verify-postcheckNN.json` | failed job, later attempt, 48 h, or 30 min with no passing record after verify finishes |
| 2 Preflight | Guard, `--phase asr --preflight-only`, spare job `logarithm-remote-asrNN-preflight` | exit 75 |
| 3 ASR | All gates, writes `O/asr-inputsNN.json`, guard runs job `logarithm-remote-asrNN` | any gate |

It writes `O/asrNN.log` and `O/asrNN.exit`, plus the two job folders and the manifest. In the wait a master or verify failure
stops it at once. A refused step 2 or 3 leaves `logarithm-remote-asrNN` unused unless the guard itself refuses at step 3.

## What root's controller must do

Save `verify-job`'s stdout, and nothing else, to `O/verify-postcheckNN.json`, where NN is the verify job's number. `verify01`
gives your path from Mail 5451. A plain `>` is fine: an empty or unparsable file counts as not ready, so a `REFUSED:` record
stops at the 30-minute bound.

## What changed

`docs/asr01-to-asr02.diff` is the whole change to `asr01.py`.

| Item | Fix |
|---|---|
| F1 | Refuses the retired hash. `master-summary.json` must match the proof on master path and hash, both caption hashes, and the tools and binding hashes |
| F2 | Exact receipt checks for verify and master: admitted, exit 0, finished, no stop or error, phase `native`, exact command, frozen inputs, script row, exact last log line. The proof must say `strict_video_check: passed` and `pictures_viewed: false`. The post-check must match job, `verification.json` hash, master hash and frames |
| F3 | Selected jobs by argument, and a later attempt beside the selection refuses |
| F4 | Attempt numbers and the preflight, so a refused start doesn't use up a job name |
| F5 | The old exit-file waiter is gone. The new wait reads only the selected jobs' records |
| F6 | Bindings ffmpeg, `tools01.json`, the proof's `worker` row and the live interpreter must agree by hash |
| F7 | Wrapper uses `pwd -P`, so `$PWD` is physical |
| Mail 5452 race | `run()` read the guard's live receipt once, so a read inside the guard's rewrite would fail ASR. It now rereads for about 10 s, then refuses, as Lavender's worker does. **`O/asr01.py` line 111 still has this race** |

**Unchanged, proved by AST against `O/asr01.py`:** `read`, `sha`, `check_files`, `script_rows`, `tokens`, `boundary_review`, and
all of `run()` from the decode on. That covers the decode arguments, 1800 s timeout, every `fw` argument, 7200 s and 7260 s
timeouts, outputs and scope strings. Model and engine still come from `O/asr-source-bindings01.json`, and the guard phase
is still `asr`.

**Verifier independence:** no verifier hash or path is pinned, so verifier03 needs no change here while its success line and
post-check keys stay the same. The ASR manifest binds the verify job's script, which holds its helper's hash.

## Changes from the assignment

- **The wrapper doesn't run `verify-job`.** Mail 5451 makes root's controller the writer, and ASR02 requires its record.
- **A wait exists.** The assignment said no stale waiter, and Mail 5451 asked for a wait on exact terminal records. Direct
  `asr02.py` without `--wait` still refuses at once.

## Traps

- **`passed()` is stricter than the verifier's `ran()`** (RubyRaven, Mail 5492). It needs exactly `[tools python, -I, -B, script]`,
  so a verify job started by a plain `python3` passes the verifier and refuses here. Root's controller uses the tools path.
- **`wait()` doesn't recheck the bound sources** (RubyRaven, Low). A moved script, model or `fw` refuses in step 3, after the wait.
  No job name is used. Left as is, so root's and Ruby's read of these bytes still holds.
- **The timeline is fixed at `logarithm-remote-timeline01`,** as in asr01. A timeline retry refuses here.
- **Scripts folders are fixed:** `O/job-scripts` for verify and `O/render-jobs01` for master. A verify job written with
  `--job-dir` elsewhere refuses.
- **Exact command strings:** a verify job written through a logical `$PWD` path refuses, because the receipt's script path
  differs from the physical one. Write it from `pwd -P`.
- **Guard lane lock:** steps 2 and 3 queue behind any job holding the lane, as every guarded job does.

## Checks

From the repository root, with a fresh scratch path: `python3 -I -B P/logarithm-remote-asr02/checks/run_checks.py SCRATCH`.
It takes about 30 seconds and writes only small files there. The transcript is `checks/outputs/run_checks.txt`.

The real `asr02.py` and `start_asr02.sh` run as subprocesses in a tiny scratch repository. Stand-ins: guard, ffmpeg, `fw`, the
tools file and every record. A fixture copy differs from the delivered file only in the retired-hash literal, the wait's
timing literals, and one PATH line, each in its own case.

- **Four good runs:** normal film started through a symlink, both retries beside failed attempts, a start before the records
  exist, and a live receipt that is partial for a second.
- **Refusals:** every new assert fires in at least one case, by its own message, and each case proves no job name or manifest
  was used. A compound assert is exercised through one of its conditions, not all of them.
- **Source agreement, not run:** constants and keys against verifier02, success line and paths against Lavender's seal
  `a7078217`. Verifier03 keeps those constants and keys, as its diff shows.

**Mutation sweep, scratch only, not delivered.** 14 removals across the wrapper lines, the reread, the wait's bound and the
name gates: 10 were caught and 4 survived.

- **The two `pwd -P` lines:** each covers for the other. Removing both together was caught.
- **The two `newest` calls inside `wait()`: a real, narrow test gap.** Nothing tests "a later attempt appearing stops the wait
  at once". `bound_proof` runs the same check before any manifest or job, and that is tested, so a stale selection still can't
  reach ASR. Without those two calls the only cost would be a longer wait. Left untested as instructed.
- **Asserts with their own message weren't mutated,** since the transcript shows each one firing.

## Limits

- **Stand-ins throughout.** Worker Python 3.14, the real guard and `sbh` weren't exercised. These checks ran under local Python
  3.15. The guard stand-in copies its arguments and receipt keys from source reading.
- **No real verify or master job exists yet.** Their forms come from the sealed source, not real records.
- **`fw transcribe` has never run on the worker,** only `--version` (root's Mail 5290). `O/fw-0.11.0`'s hash comes from that mail.
- **Not independent.** I wrote the review, verifier02 and this. RubyRaven read these exact bytes and found no launch or identity
  bug (Mail 5492). That was source reading, not acceptance.
- **A passing ASR run gives transcript leads for triage only.** It doesn't show natural delivery, pronunciation or sync, and this
  bundle accepts nothing.
