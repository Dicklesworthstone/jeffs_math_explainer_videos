# Recovery 1005: review of CC2's draft master993-audio-seams01.py against guard_csd02

2026-11-10 19:28 UTC, CC0 (LavenderPike).

**Reviewed bytes:** `master993-audio-seams01.py`, sha256 `984f5a6e26b3dd984746b3f10977f69dcd5e2ba6a58183d22e5e245dc5a6ce50`, 22,098 bytes, 419 lines. This is CC2's unlaunched draft in its own session folder; it is unchanged by this review.

**Scope:**
- What was done:
  - The source was read in full.
  - Its synthetic selftest was run.
  - The guard chain was read from local copies at its pinned hashes.
- What was not done:
  - No decode or ffprobe run.
  - No ssh and no read on the CSD.
  - No job started.
  - No edit to the helper, the repo, the TODO or Beads.

No audio heard or checked. Nothing here is hearing, fine-sync, picture, AV or release acceptance.

## Verdict

There are two blocking defects, D1 and D2. Both are small fixes in `decode()`, `measure()` and `launch()`. Fix them before any real run.

The measurement arithmetic has no defect: window slicing, sample alignment and the WAV layout all check out.

Root's four observations are answered in the section "Root's four points" below.

## What guard_csd02 does (the facts the findings rest on)

The local `guard_csd02.py` is at `f1cae1dbc80075311b0e1a2abf8637439d2d8d0edbc643e771e2eefe2b3bdfb6`, which is the TODO L4 hash. Its `load()` asserts the rest of the chain:
- owner guard `1c19a9b8…`;
- bounded runner `run_bounded_phase_oliver_owner48g_target14g_20261102.py` at `819746fb…`;
- stopper `owned-process-stop02.py` at `a2fe7985…`.

Local copies at those hashes were read.

1. **One exclusive lane.** The runner (:101-105) holds a `flock` on `D/expanded-native-lane.lock` for the whole job. Later guarded jobs queue on that lock.
2. **The child's process setup** (:147-148):
   - The child starts with `start_new_session=True` and cwd `D`, and its output goes to `job/command.log`.
   - The helper therefore leads its own session and group.
   - The helper's own `Popen` of ffmpeg does not ask for a new session, so ffmpeg is in the helper's group.
3. **No time limit.** The loop (:150-166) ends only when the child exits or a disk limit trips.
4. **`stop_owned` runs in two cases only:**
   - a disk-limit stop (:158);
   - an exception in the guard itself (:171-173), for example SIGINT.

   The stopper installs the guard as a subreaper (stop02 :17-25). When it runs, it walks every descendant of the guard:
   - That includes ffmpeg in any group, and orphans the subreaper adopted.
   - It sends SIGTERM through pidfds (:69).
   - It sends SIGKILL to survivors at 20 s (:78-83).
   - It raises at 30 s (:84-85).
5. **When the child exits by itself, nothing sweeps** (:161-170). There is no `stop_owned` and no `waitpid` for adopted orphans. A descendant that is still alive then outlives the guard.

## Blocking defects

### D1. The decode has no effective time bound and nothing kills ffmpeg when the helper exits

Lines 282-291, and the ffprobe call at 332-333.

**What the code does:**
- `proc.stdout.read(1 << 23)` loops until EOF.
- `proc.wait(timeout=3600)` is reached only after EOF, so its 3600 s bounds nothing.
- The ffprobe `check_output` (332) has no timeout.
- No `try`/`finally` kills and reaps ffmpeg.

**What that means under guard_csd02:**
- **A stall holds the lane indefinitely.** By fact 3 the guard has no time limit. A decoder that stops writing without exiting would block the helper in `read()` and hold the exclusive lane (fact 1). Every queued CSD job would wait until someone stopped it by hand.
- **ffmpeg can outlive the guard** (fact 5).
  - This needs the helper to end by an exception while ffmpeg is alive, for example a `TimeoutExpired` at 291, or any error inside the loop.
  - ffmpeg is then adopted by the guard. The guard's normal-exit path does not run `stop_owned`.
  - A writing ffmpeg would end at its next write, on the broken pipe. A stalled one survives the guard.

**How the sibling helper handles it:** `csd-final-verify995.py` (`69fa3ded…`) runs on the same lane against the same master.
- It gives its subprocesses timeouts; its ffprobe has `timeout=600` (:985).
- It wraps its guarded command in `/usr/bin/timeout 86400` (:1263-1264).

**Fix, using numbers that already exist (no new limit):**
1. Bound the run in one of two ways:
   - wrap the measure command in `launch()` the way 995 does;
   - or make the helper's own 3600 s a monotonic deadline over the whole read loop, for example with `select` on `proc.stdout`.
2. Give ffprobe a timeout, as 995 does.
3. In `decode()`, add `finally: if proc.poll() is None: proc.kill(); proc.wait()`.

A stall is unlikely when decoding a local file that has already been verified. The defect is that nothing bounds it on an exclusive lane.

### D2. The guard file is neither bound nor checked

Lines 380-386.

**What the code does:**
- The input set in `launch()` is {helper, master993 receipt, resolved python, PINS}.
- `guard_csd02.py` is neither a row of that set nor hash-checked.

**The rule and the sibling:**
- TODO L4 (adopted 2026-11-09): "prelaunch require exact guard_csd02 SHA f1cae1db…".
- verify995 puts `GUARD` in its inputs (:654).

**What that means under guard_csd02:**
- The guard's `load()` asserts the owner guard, runner and stopper hashes itself. Once `guard_csd02.py` is fixed, the whole supervisor chain is fixed.
- The one unpinned link is `guard_csd02.py`. As written, whatever file sits at `D/guard_csd02.py` would run.
- The receipt's `inputs_sha256` would not show which guard supervised the run.

**Fix:**
1. Before `INPUTS` is written, assert `sha(GUARD) == 'f1cae1dbc80075311b0e1a2abf8637439d2d8d0edbc643e771e2eefe2b3bdfb6'`.
2. Add `GUARD` to `files`.

Adding the stopper and the guard's two pinned files as extra rows is optional. It records them, but adds no safety, because guard_csd02 already asserts them when it loads.

**Not needed:** `measure()`'s partial policy check (stop free 8, growth 4, line 323) is the same as verify995's `admitted()` (:937). With the guard pinned, guard_csd02's own asserts fix the policy at 10/8/4/56/20. verify995 also checks that the receipt's command names its helper (:938-939); `measure()` checks only its own argv. That is worth copying, but it is not blocking.

## Root's four points

1. **Blocking `stdout.read` before `wait(timeout=3600)`.** Confirmed; see D1.
2. **No `own_group` or signal cleanup.** This is not needed for custody under guard_csd02.
   - On every guard stop, the stopper reaches ffmpeg as a descendant, whatever its group.
   - If the helper dies on the default SIGTERM, nothing is lost: `seams.json` is written last, with `'x'`.
   - support671's escaped-descendant tracking exists for an orphaned ffmpeg that leaves the supervisor's tree; its docstring cites launchd adopting it. On the CSD, the stopper's subreaper keeps such orphans in the tree.
   - `own_group()`'s session-leader assert would also conflict with a `/usr/bin/timeout` wrapper.
   - The gap that does matter is the guard's normal-exit path (fact 5), which the `finally` in D1 closes.
3. **The launch input set excludes the guard and its dependency chain.** Confirmed for the guard file itself; see D2. The chain below it is self-pinned by guard_csd02.
4. **`lane_status` accepts a terminal failure (183-193).** This is consistent with the 995 controller and reads as intended.
   - **How the controller runs:** `csd-final-after-master995.py` (`9a055c26…`) runs check → verify → overview → native in order. It stops at the first job whose receipt fails `success()` (:40-45).
   - **After a failed (or not admitted) verify995:** no 995 picture job will start, so waiting for them would never end.
   - **After a successful verify995:** both picture receipts must be finished, whatever their exit.
   - **If overview fails:** native never starts, `lane_status` never becomes terminal, and `launch()` refuses. That fails closed, and root decides.
   - **What `lane_status` covers:** ordering only. Exclusivity comes from the guard's flock. It does not check live lane holders or waiters (TODO L2); root does that at launch.

## Diagnostic, not release gating

`seams.json` is a raw measurement of the pinned master993 bytes, plus the post-gain WAV when its provenance holds.
- It has no thresholds.
- Its `not_claimed` block marks as unclaimed:
  - hearing;
  - ASR and words;
  - pacing and fine sync;
  - AV and release acceptance.
- It feeds no release gate.

A completed seams job is not hearing, fine-sync, AV or release acceptance, whatever its numbers.

It does not record the 995 lane state. If root runs it after a failed verify995, root's ledger should say so beside the result. A successor helper could instead copy `lane` into `seams.json`; that is optional.

## Checked, no defect

**Streaming:**
- Windows are cut by absolute byte offset (287-289), which is exact for any chunk length.
- Each window is asserted to be 2W×8 bytes (296).
- The 14 boundaries recomputed from the local summary copy (`38b52105…` = the pin) match: 15 rows, 133,662 frames, equal to `BOUNDARIES`.
- The nearest boundary is 3,503,200 samples from either end, so W + PAD fits.

**Alignment:**
- Windows use the absolute decoded index.
- A positive lag compares decoded[t] with WAV[t−L]; the selftest confirms the sign at +1024.
- The lags 0, ±1, ±1024 and ±2112 cover one sample, one AAC frame and the usual priming.
- The decoded-length difference is recorded, not asserted. That is acceptable for a diagnostic.
- ffprobe asserts AAC, 48 kHz, 2 channels (335).

**WAV provenance:**
- The writer path, in the pinned `cli.rs` (`3c5221fe…`, read from a local copy at that hash), `master_video`:
  - calls `apply_master_gain`;
  - then `write_master(F32)` to `out/master/<crate>.master.wav`;
  - then `master::join` (`master.rs` `7d1952db…`), which encodes exactly that file with `-c:a aac -b:a 256k -ar 48000`.
- The F32 header in `wav.rs` (`0f41938b…`) is 80 bytes, matching the helper's field table.
- Local copies match the pins:
  - `combined-master993.py` is `1955ca32…`;
  - the master993 receipt's inputs hash is `a610afcd…`, equal to `PINS[MINPUTS]`; the same receipt shows exit 0, policy 10/8/4/56/20, and a job window of 17:28:09–17:42:48 UTC;
  - the local transfer receipt gives the movie as `80244e88…`, equal to `PINS[MOVIE]`.

**Selftest:**
- Command: `python3 -I -B master993-audio-seams01.py selftest`, local Python 3.15.0.
- Output: `selftest OK (synthetic arrays only)`, exit 0.
- The dispatch (416-417) runs before the CSD path assert, and the selftest opens no file.

## Not done

- No `check` mode run (it needs the CSD).
- No decode, no ffprobe, no CSD read, no job.
- CC2's file was rehashed after the review: still `984f5a6e…`.
- No audio heard or checked.
