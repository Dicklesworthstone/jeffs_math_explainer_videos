# Review 530: recovery 528, delta only (CC0)

cc0, 2026-11-09, 22:40 UTC. Q is this folder. B is `narration_lab/third-wave-20261024/root/five-film-release-20261027`.

| Reviewed | sha256 |
|---|---|
| `cc2-apportionment-preview-recovery528.py`, 495 lines, sealed by CC2 | `4206b9fc00966c5d13f78d0a90aa11a5a4343dbf2b8304d5b40de8d29cabb838` |
| `cc2-apportionment-preview-recovery528.md`, CC2's notes | `c3c7e125ceed4c5cc6d6f6b51cdac34c6aa0aede8c51ad69d5cf6873449dc5c0` |
| Baseline `cc2-apportionment-preview-recovery516.py` | `81f39578c44b676c626c466bc7a84421693278183da24af3976b3c388763a231` |

**This review did not gate the run.** A usage limit interrupted it. Root launched 528 at 06:54Z, and decision 539 (`14ff9728…`) records this review as "not completed or claimed". The job exited 0 at 07:42:03Z. Everything below is after the fact.

**Method.** I read the full diff against 516, CC2's notes, its test files and output, the guard's stop path, and the completed job's receipt, summary and log. I made one small check of my own (L1) and timed one `ps`. I ran no guard, render, decode or ffprobe, and hashed no movie.

## Verdict

**526 M1 is fixed for the cases 528 claims, and I found no blocker, High or Medium defect.** Cleanup is identity-safe and there is no name kill.

There are **two Low findings, both about whether a statement is true on a rare path**: L1 in the helper and its notes, L2 in the notes only. Neither affected the completed run, so that run needs no action. Both matter only if 528 is reused, and root's decision 539 calls these limits "documented", which is only partly right.

## Identity-safe descendant cleanup: holds

- **Only four signal calls exist.**
  - Line 196 sends to a target only after a fresh snapshot matches its pgid, start time and command.
  - Line 212 sends SIGSTOP and SIGCONT to the helper's own child, and only while `child.poll()` is None, so that pid cannot have been reused.
  - Lines 222 and 256 use killpg on the helper's own group.
- **No name kill.** There is no pkill or killall. `escaped()` selects by ppid walk and pgid alone; the command is only part of the identity compared afterwards.
- **Pid, not foreign group.** Signalling the pid is stricter than the killpg I suggested in 526. A child of an escaped process would be found by the same walk.
- **Only fmn's ffmpeg can escape.** In `crates`, outside test files, there is no `process_group`, `setsid`, `pre_exec` or `set_pgroup`. The kit's own spawns (`cli.rs:428` python3, `master.rs` ffmpeg and ffprobe) are plain `Command`s and stay in the group. `tools/video_evidence.py` starts a new session, but the kit does not call it.
- **Order is right.** Lines 220 to 224 stop the binary, snapshot, send SIGTERM to targets and then to the group, then continue the binary. On SIGTERM alone the binary dies first, and the tracked list covers that case.
- **The delta is only what CC2 lists.** My diff shows no hunk in `chapter`, `stems`, `wav`, `listing`, `success`, `pressure` or `guard_binding`. That agrees with CC2's T1.

## CC2's stand-in test

I read `test528.py` (`0941c247…`), `child528.py` (`0f69a2f1…`), `renderer528.py` (`49ab2a0d…`) and the output (`9f8aa179…`, 12 of 12). The hashes match the notes. I did not rerun it.

- **T2** reproduces M1 on 516: after a group SIGTERM the helper ends -9, and the new-group `/bin/sleep` is still alive.
- **T3 and T5** use a real grandchild in its own group whose parent dies first. 528 ends it by identity from the tracked list. T3 is the guard's stop; T5 is a render that exits 1.
- **T6 and T7** exercise a start-time mismatch and an unavailable `ps`.

The test's limits are N3 and N4 below.

## Evidence from the completed run

Summary `6fb0d3bedb86ab365f2163dac2e43972307e596c61aee6213c4b64ff3af151fa`; receipt `3643dd298dc6317bee6bfb9286d14298817db305af6107b58f7666dc6a42f863`, exit 0, inputs `305def16…` at start and end. The summary's `helper_sha256` is `4206b9fc…`.

| Chapter | Polls | Missed | Escaped seen |
|---|---|---|---|
| 09_cost | 106 | 0 | 2 |
| 10_rounding_rules | 300 | 0 | 2 |
| 11_webster_quota | 191 | 0 | 2 |
| 12_both_promises | 528 | 0 | 2 |
| 13_population | 307 | 0 | 2 |
| 14_no_rule | 334 | 0 | 2 |
| 15_recap | 273 | 0 | 2 |
| **Total** | **2,039** | **0** | **14** |

- **What it shows.** On the real binary, two descendants per chapter sat outside the helper's group. 526 M1 came from source alone, so this is its first observation on the real binary.
- **What it does not name.** The record holds counts only. That they were ffmpeg is my inference from the spawn path. Which two of the probe, encoder and mux were caught is unknown.
- **What it does not show.** The stop path never ran on the real binary: the job exited 0 and its folder holds no `cleanup.json`. Stopping real ffmpeg and the real binary is still shown only by stand-ins and source.

## Findings

- **L1 (Low): `settle()` treats "could not look" as "nothing left".**
  - **Source.** `alive_targets()` returns `[]` when `ps` fails (line 207). `settle()` then returns `[]` (lines 390 and 391), which `run_owned` reads as no leftovers.
  - **Direct check.** I imported the sealed helper without running `main()`, tracked one live `/bin/sleep` of my own, and broke `PS`. `settle()` returned `[]`, `missed` rose by 2, and the target was still alive.
  - **Why it is a wrong statement.** Docstring lines 34 and 35 and the notes' check 4 say a `ps` failure leaves that sweep "recorded as not run". That holds for `hit()`, not for `settle()`, where only `missed` grows. T7 does not reach this path, because it stops by timeout.
  - **Record gap.** On a non-zero exit, line 444 (`assert code == 0`) does not print the tracker, so even `missed` is lost.
  - **Not signalling is correct**, since the target is unverified. Only the report is wrong.
  - **This run:** no effect. `missed` is 0 in all seven chapters and includes `settle()`'s snapshots.
  - **Fix if reused.** Have `alive_targets()` return None on failure, put a `settle` outcome in the tracker, and print the tracker in that assertion. Failing an exit 0 for this should not be needed, on the expectation (not verified here) that fmn waits for its ffmpeg before publishing.
- **L2 (Low): the notes' "Not covered" limit is wrong about the guard.** They say that after a SIGKILL sent to the helper alone, "the guard's 20 s SIGKILL ends the group".
  - **Source.** In guard `819746fb`, `stop_owned` acts only `if child.poll() is None` (line 77). It is called on a resource stop or an exception. Once the helper is gone, the loop leaves at line 161 and sends nothing.
  - **Actual behaviour.** The guard records -9 and finishes, which frees the lane lock, while the binary and its ffmpeg run on until they end by themselves. The 20 s SIGKILL applies only while the helper is still alive 20 s after the guard's own SIGTERM.
  - **Scope.** The behaviour is the same as 516 and 458. The docstring does not make this claim.
  - **This run:** did not occur.
  - **Fix.** One sentence in the notes.

## Other limits in the notes

- **Guard-stop gap: true, and slightly wider than stated.** A poll adds `ps` time to the 1 s wait. 15_recap took 349.6 s of render for 273 polls, about 1.3 s per poll.
- **Partial files and start-time precision: true as stated.** The comment at line 233, "a reused pid is never signalled", is stronger than the notes' "negligible, but not proven". The notes are the accurate version.
- **Cost: the process count is right.** CPU is not stated. One `ps` here took 0.23 CPU seconds (3,522 processes, load 43, a single sample), so about 0.2 of a core while tracking.

## Notes (no change needed)

- **N1. The whole sweep can outlast the guard's 20 s.** It makes at least three `ps` calls, each allowed 10 s, plus the 6 s grace. Normally it takes at most about 7 s (one `ps` took 0.26 s). If the guard's SIGKILL lands first, the group still ends and the targets have had SIGTERM, but no `cleanup.json` is written. "Under the guard's 20 s" is true of the grace only.
- **N2. Wrong reason in one record case.** An exited, unreaped target shows `<defunct>` on macOS (33 rows in my sample), so `same()` is false and `hit()` logs "another process now". Not signalling it is right.
- **N3. T4 does not observe SIGSTOP.** `child_exit` -15 would also result with no stop at all, so "the binary was stopped" comes from source. T4 does show the binary was not left stopped.
- **N4. No same-name decoy was alive during T3 or T4.** T6's sleep is outside the tree, but its identity is injected by hand. The source settles the point, because nothing compares names.
- L1, N1 and N2 went to CC2 as notes on the same bytes before its seal message reached me. The sealed bytes do not address them.

## Not checked

- running 528 or any guard, and the behaviour of real ffmpeg or the real binary on SIGTERM or SIGSTOP;
- MANIFEST528 (`305def16…`, 3,874 rows by root's record), which I did not read;
- chapters 09 to 15: no file hashed or probed, and the figures above are the helper's own records;
- whether any process outlived the run;
- CC2's T1 and T8, read as output only.

No audio heard or checked. This review accepts no picture, sound, sync, AV or release result, and it authorises nothing; root owns launches.
