# ASR review addendum: `O/asr01.py` against verifier02

BronzeJaguar, 2026-11-05, for BlueSalmon, bead 87do.63. O = `EL/remote-mac-mini-max01`.

Reviewed: `asr01.py` `203f2ad3…`, `asr-source-bindings01.json` `71e140bb…`, `check-asr-proof01.py` `39edc8da…`, its result,
`remote-tool-observations01.json` `7c4bb7a7…`, and `tools01.json` `188b7650…`, measured by root on the worker.

This addendum adds its own files. The 10 sealed verifier files are unchanged, and `shasum -c` still passes.

## Verdict

**No launch blocker on the normal path.** Master01 passes, verify01 passes, and `asr01.py` accepts what verifier02 writes:
root's real `bound_proof` accepted it in G9b, on a stand-in. Bindings match the measured worker tools.

**Stale-master refusal holds on the real lane, but `asr01.py` doesn't provide all of it.** The retired master is refused
by verifier02, and by the repaired timeline differing. `asr01.py` has no refusal of its own, and its positive stand-in is
the retired master. F1 closes that.

Four Medium findings, four Low. F1 to F3 are small patches, and F4 is a launch step. All of them fail closed: none lets a
wrong film through ASR. The cost is a burned job name or a new script.

## Stale prior master

| Case | Result | What stops it | Evidence |
|---|---|---|---|
| Stage09 `verification.json` left in place | Refused | `master_job`, line 27 | root's plant |
| Retired bytes under the remote job name and path, timeline matching | **Accepted by `bound_proof`** | nothing in `asr01.py` | root's positive stand-in: its proof is stage09's, sha `00236d1a…` |
| The same against the real new timeline | Refused | frames or chapter tuples, lines 30 and 32 | reading; relies on the schedule differing |
| Retired master or binary through the real job flow | Refused before ASR | verifier02 `RETIRED_MASTER`, `RETIRED_BINARY` | G8, stand-in |
| Master file replaced after verify | Refused | hash, line 37 | reading |
| Verify job failed | Refused | `ready()`, line 57 | reading |
| `verification.json` changed after the job, consistent with the live master and timeline | **Not detected** | nothing: F2 | reading |

`check-asr-proof01.py`'s `old_master_job` plant changes only the name. No plant changes the master's hash.

## Findings

### F1, Medium: no retired-master refusal, no link to the master's own record

`bound_proof` (lines 24 to 38) trusts `proof['sha256']` once the file hashes to it. It never compares that value with the
retired hash or with `master-summary.json`, the record the master job wrote.

**Fix:** the first four lines of the patch below, and add the summary to the manifest.

### F2, Medium: `ready()` doesn't prove the verifier passed

`ready()` (lines 50 to 60) reads only the receipt's admission, exit, stable inputs and error. It doesn't check phase,
command or the success line. Nothing ties `verification.json` to that receipt. `verify-job`, the post-check that re-reads
pictures, sheets and decode logs, leaves no record, so `asr01.py` can't tell whether it ran.

**Fix:** run `verify-job` before launching ASR, and add the `ready()` lines below.

### F3, Medium: a retry can't reach ASR

- **Master retry:** verifier02 and Lavender's driver allow `-r2` to `-r9`, but line 27 requires the base name.
- **Verify retry:** line 15 fixes `verify01`, while `write-job` needs `--verify 02` for a second attempt.
- **Early launch:** if the waiter is running when the base master fails, line 176 stops it, though a retry could succeed.

All of these fail closed, but ASR would then need a new script. The patch covers the master. For a verify retry, change
line 15 in a new script.

### F4, Medium: the first launch can burn `logarithm-remote-asr01`

The guard runs `job.mkdir()` first, then calls `sbh` by name, then decides admission. So:

- with `sbh` off PATH, it raises after the folder exists;
- below 12 GiB free, it exits 75 with the folder in place.

Line 181 then refuses every later try, and `asr-inputs01.json` is spent too. O has no ASR start wrapper. Stage09's
`start01.sh` set no PATH, and it worked on a local shell. Root's build wrapper and Ruby's continuation both export one.

**Fix, no code change to `asr01.py`:** add a wrapper that exports Ruby's PATH line, then run this first, under a spare name:

```sh
python3 -I -B O/guard01.py --phase asr --job logarithm-remote-asr-preflight01 --preflight-only
```

From my reading of the guard, exit 0 means admitted, 75 means not, and `asr01` stays unused. I haven't run it.

**Correction to my sealed HANDOFF:** step 3's printed verify guard line has the same PATH need, and the HANDOFF doesn't say
so. Launch it from a shell with that PATH, and preflight with `--phase native`.

### F5, Low: the waiter watches names nobody writes

Line 166 reads `build-controller01.exit` and `audio-controller01.exit`. The real files are `continuation0N.exit`, so a
stopped continuation goes unseen, and the waiter spins for 48 hours. It fails safe.

Don't swap the names: `continuation01.exit` should hold rebind01's failure, which would stop the waiter wrongly. Launching
after verify skips the loop.

### F6, Low: the ASR ffmpeg isn't compared with the verifier's

Both are `1332dc2de372` today, by path `/opt/homebrew/bin/ffmpeg` here and the Cellar file in `tools01.json`. No code
compares them. One patch line does.

### F7, Low: path spelling

Line 28 compares with a `resolve()` path, while verifier02 spells `video` from `--lane "$PWD/…"`. If `$PWD` crosses a
symlink on the worker, ASR refuses a good proof. **Fix:** `cd "$(pwd -P)"` before the verifier commands. I can't see the
worker's paths.

### F8, Low: gaps in the evidence

- `O/fw-0.11.0`'s hash and version come only from root's Mail 5290. The observations file records the old 0.8.0 `fw`, and
  no local transfer record or local copy names the new one. Line 161 refuses a mismatch before any job name is used.
- **`fw transcribe` has never run on the worker.** Only `--version` has.
- `fw_version` in the bindings is never read. The hash pin is the stronger check, so this is harmless.

## Checked and sound

| Item | Evidence |
|---|---|
| `verification.json` path and every key read: `master_job`, `video`, `sha256`, `frames`, `fps`, `chapters`, `placements` | G9b ran the real `bound_proof`, stand-in data |
| Timeline hashes | verifier02 requires the render, job and summary copies byte-equal, equal on all 13 stage09 chapters |
| No hardcoded film length | `116245` is gone; `frames` comes from the timeline |
| Script row | `559afa36…`, the verifier's pin |
| ffmpeg row | hash equals worker `tools01.json` |
| Model row | hash equals the observations file |
| `fw` row | same hash as the binary that ran stage09 ASR; links only system libraries (local `otool`) |
| Interpreter gate, line 113 | worker `python3.14` path in `tools01.json`; the path re-reports itself locally; stage09's receipt has the same form |
| Script logic | apart from the proof binding, the diff against stage09's `after-verify01.py` is paths and names |
| Engine | stage09: in-process, `native_only`, no fallback, 71 s, 0 of 4,464 segments with start after end |
| Disk | stage09's ASR job used 0.06 GiB against a 1 GiB cap; all stage09 job folders total 2.48 GiB against 56 |
| Manifest | no repeated rows; the guard opens symlinks by following them |

## Suggested patch

```python
RETIRED_MASTER = '00236d1a6915ff968235a30ab59769b02125ece0fee482e4fa3154c265ac7c24'

# in bound_proof(), replacing line 27
assert re.fullmatch(r'logarithm-remote-master01(-r[2-9])?', proof['master_job'])
assert proof['sha256'] != RETIRED_MASTER
summary = read(B / proof['master_job'] / 'master-summary.json')
assert summary['master']['sha256'] == proof['sha256']
assert proof['strict_video_check'] == 'passed' and proof['pictures_viewed'] is False
assert proof['worker']['ffmpeg']['sha256'] == read(SOURCES)['files'][3]['sha256']

# in ready(), after line 59
assert d.get('phase') == 'native'
lines = [x for x in (VERIFY / 'command.log').read_text(errors='replace').splitlines() if x.strip()]
assert lines[-1].startswith('FINAL VERIFY PASS')

# in wait_and_launch(), with the other paths
paths += [B / proof['master_job'] / 'master-summary.json']
```

**This breaks `check-asr-proof01.py` until it's updated.** Its stand-in uses the retired hash, its mocked `read` returns the
proof for any path, and stage09's proof has no `worker`. Give it a non-retired hash, a summary and a worker row, then add
plants for the retired hash and a differing summary. I haven't run this patch.

## Limits

- Source and record reading, plus G9b's stand-in. No SSH, no job, nothing run on the worker, and no edit to O.
- Worker facts come from root's mails and `tools01.json`. I read no remote job folder.
- **ASR isn't hearing proof.** A transcript can't show natural delivery, pronunciation or sync. Listening and fine sync
  remain open, and this review accepts nothing.
- **Independent of `asr01.py`, not of the verifier.** I wrote verifier02, so the schema side is my own work.
