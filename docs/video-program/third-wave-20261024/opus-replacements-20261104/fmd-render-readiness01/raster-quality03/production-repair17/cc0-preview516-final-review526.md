# Review 526: the corrected recovery 516, delta only (CC0)

cc0, 2026-11-09, 06:36 UTC. Q is this folder. This review covers the corrected `cc2-apportionment-preview-recovery516.py`, `81f39578c44b676c626c466bc7a84421693278183da24af3976b3c388763a231`, 352 lines, sealed by CC2. CC2's notes (`.md`, `29b992c5`) were read for their claims.

**Baseline.** The baseline is the 524 draft `7547515c`. I rebuilt it byte-exactly from my 524 reading; it hashes to `7547515c…` at 13,736 bytes. I read the full unified diff against it.

**Method.** The unchanged functions (`chapter`, `stems`, `wav`, `listing`, `success`, `pressure`) were checked in 524 and are unchanged in the diff, so I did not repeat that. Nothing was run: no helper, no check-only, no decode, render or hashing of the movies.

## Verdict

**No blocker. One Medium finding (M1), on the stop path only, and one Low finding (L1), which is root's manifest.** A guarded run that succeeds does not depend on either. Root can launch as is if it accepts M1's stop behaviour, which is what every guarded render, 458 included, has had. Otherwise, apply M1's fix first.

## Root's four checks

1. **The new guard manifest includes the helper and the reused files: not enforced by 516, so root's step (L1).** 516 does not read MANIFEST516. It records only the receipt's `inputs_sha256`, its own hash and the binary's, in the log and the summary. That settles 524 L1.
   - 516 still checks, independently of the manifest, 458's inputs file (`5817cb64`) and all 3,820 rows, the 444 and 445 receipts, 458's receipt, and the 52-file listing `fc5100da`, before and after.
   - So MANIFEST516 adds two things: the guard's own end-of-run re-hash, and binding the helper's bytes to the run.
   - MANIFEST516 did not exist at 06:35Z.
2. **Check-only is guarded: holds.** Both modes refuse unless the helper leads its own process group and session, as the guard's `start_new_session` child does. A shell-started run fails the session test.
   - The job must be the mode's fixed job under B, holding only `command.log` and `resource-receipt.json`.
   - The receipt must show admitted, `native`, not a preflight, no `exit` or `finished`, `policy` equal to 458's recorded 80g policy (10/8/4/80/20/10), and a command naming this file, with `--check-only` only in the check job.
   - I checked this against the original guard (`819746fb`):
     - the receipt holds `phase`, `command`, `preflight_only`, `admission`, `policy`, `admitted` and `inputs_sha256` before the command starts;
     - `exit` and `finished` are only added afterwards;
     - `save()` rewrites in place without temporary files, so the fresh-folder test cannot trip. `read_receipt()` retries a torn read.
3. **Shared-group timeout cleanup: works for the group, but fmn's ffmpeg is not in the group (M1).** Each render, decode and probe stays in the helper's group.
   - A timeout, an exception or SIGTERM/SIGINT/SIGHUP runs `end_group()`, which ignores further stop signals and sends SIGTERM to the group. It waits up to 6 s, under the guard's 20 s, writes `<step>.cleanup.json` and then sends SIGKILL to the group.
   - That replaces the draft's bare `subprocess.run(timeout=)`.
   - It reaches the binary, the helper and Python's ffprobe/ffmpeg checks. It does not reach the ffmpeg processes the binary starts.
4. **The original 14,400 s total render budget: holds.** `RENDER_SECONDS = 14400`, shared by the seven renders. Each render's timeout is the time left, measured with `time.monotonic()`. It stops before a chapter once the budget is spent.
   - The budget covers render calls only. The checks around them (01–08 before, each new chapter after) take extra time, each decode bounded at 1,800 s.
   - 458 itself used about 2,110 s. Whether 516 gets 14,400 s or only the remainder is root's call (CC2 open point 7).

## Findings

- **M1 (Medium): the group stop does not reach fmn's ffmpeg.** 516's docstring says "the guard's SIGTERM and SIGKILL reach the binary and its ffmpeg at any moment", and the md says the same in item 5.
  - **Why it's wrong.** fmn starts every ffmpeg in a new process group. `fmn-platform/src/process.rs:1749`, `PosixSpawnAbsoluteProcessGroup`, uses asupersync `process.rs:1154`, `POSIX_SPAWN_SETPGROUP`. Both files are bound rows of 458's manifest at their hashes.
  - **Nothing on the binary's path handles SIGTERM.** I searched explainer_kit, explainer_score, the apportionment crate and franken_manim, and found no SIGTERM handler. fmn's own `kill_process_tree` runs only on fmn's internal timeouts and errors.
  - **Effect.** A timeout, stop signal or guard resource stop kills the binary, but an encoder or mux ffmpeg survives in its own group. The encoder normally ends at end of input once the pipe closes; a mux, which reads files, runs to completion. Either can leave a partial or temporary file in the 516 preview folder after the receipt is final and the lane lock is free.
  - **Not a regression.** The guard's own stops have always behaved this way, 458 included.
  - **Untested.** CC2's K4 and K5 stand-ins keep their grandchildren in the group, so they do not show this.
  - **Fix (small, optional).** At the start of `end_group()`, take one `/bin/ps -ax -o pid=,ppid=,pgid=` snapshot. Collect the process groups of the helper's descendants that are not its own group, and send them SIGTERM and later SIGKILL alongside the helper's group. The groups survive the ffmpeg being reparented, so the snapshot stays valid. If `ps` cannot start, fall back to the helper's group alone.
  - **Otherwise.** Correct the two sentences and the line-123 comment, and root accepts the 458-equivalent stop behaviour.
- **L1 (Low, root): what MANIFEST516 must hold.** It is root's to freeze. Two runs can't share a manifest file, but the check job and the main job may use the same one.
  - Every one of the 3,820 rows of `5817cb64`, with the same sha256.
  - One row for `Q/cc2-apportionment-preview-recovery516.py` at `81f39578…`.
  - 52 rows for `B/apportionment-frederick-preview458/**`. Their `sha256  bytes  path` lines, relative to that folder and sorted, must hash to `fc5100da…`, which is 516's pin and my 520 rebuild.
  - These rows total 3,873, plus anything else root adds.
  - Root's freeze note should give the manifest's sha256. 516's summary then binds it through `guard.inputs_sha256`.

## Notes (no change needed)

- **N1. Check-only has one fixed job name.** The guard refuses a used name, so a second check-only run needs a new name and therefore a helper change.
- **N2. A render that exits non-zero by itself is not swept** (CC2 open point 3). The helper exits through its assertion. With M1's fix, calling `end_group()` there as well would sweep it, at the cost of exit -9 instead of 1.
- **524's N1 is settled.** Each passed chapter's folder, frames and file hashes are printed (`passed=record[c]`).

## Not checked

- any run of 81f39578, guarded or not;
- MANIFEST516 itself, which did not exist yet;
- CC2's test516b output;
- the binary's behaviour on SIGTERM in practice (M1 is from source, not observed);
- chapter 12's split-stem rows (524 N2, unchanged).

No audio heard or checked. This review accepts no picture, sound, sync, AV or release result, and it authorises no launch; root launches.
