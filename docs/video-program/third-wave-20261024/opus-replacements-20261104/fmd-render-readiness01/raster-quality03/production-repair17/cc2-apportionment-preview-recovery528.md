# Recovery 528: 516 with a stop that reaches fmn's ffmpeg (CC2)

CC2 (RubyRaven), 2026-11-09, about 06:50 UTC, task 528 (root, through the owner's session), after CC0's review 526.

**What this is.**
- A narrow successor of 516 for CC0 526 M1. Only the stop path changes.
- 516 is kept sealed and unchanged (`81f39578…`, notes `29b992c5…`). It was never run. 528 takes its place for the launch.
- Prepared only. I launched no guard, build, render or decode, and I edited no Beads, TODO or memory file.
- preview458, timeline444, score445, 462, 468 and every file they wrote are unchanged. The narration is unchanged: narration437, bound by hash.
- Root launches 528 only after CC0's delta review.

## Files

| File | sha256 | |
|---|---|---|
| `cc2-apportionment-preview-recovery528.py` | `4206b9fc00966c5d13f78d0a90aa11a5a4343dbf2b8304d5b40de8d29cabb838` | new, 495 lines, sealed |
| `cc2-apportionment-preview-recovery516.py` | `81f39578c44b676c626c466bc7a84421693278183da24af3976b3c388763a231` | sealed predecessor, unchanged, never run |
| `cc2-apportionment-preview-recovery516.md` | `29b992c55547f5fdba6e53f77b6976fe4615d47a554f8e3d3900b0756c51916d` | its notes, unchanged; every 516 fact not repeated here still holds |
| `cc0-preview516-final-review526.md` | read | CC0's delta review of 516: no blocker; M1 and L1 |

Q is this folder. B is `narration_lab/third-wave-20261024/root/five-film-release-20261027`.

## The finding (CC0 526 M1)

- fmn starts every ffmpeg in a new process group: the `-version` probe, the encoder and the audio mux. I read these lines at the bound commits with `git show`; the working trees are at newer commits.
  - asupersync c48399f `src/process.rs` (blob sha256 `b381737d…`): `spawn_exact_image_unix` (1099) calls `set_pgroup(Pid::from_raw(0))` (1147) with `POSIX_SPAWN_SETPGROUP | POSIX_SPAWN_SETSIGDEF` (1154) and records `process_group: pid` (1185).
  - fmn-platform a399d62 `crates/fmn-platform/src/process.rs` (`4dc7045a…`) reaches it through `PosixSpawnAbsoluteProcessGroup` (1749). That is the mapping at 1808–1811 and the Linux and macOS host default at 1818–1823.
  - Both blob hashes are those of the bound rows in 458's manifest.
- CC0 searched explainer_kit, explainer_score, the apportionment crate and franken_manim and found no SIGTERM handler; I did not repeat that search. fmn's own `kill_process_tree` (asupersync 827, killpg at 938; fmn-platform wrapper at 2196) runs only on fmn's internal timeouts and errors.
- So 516's `end_group()` and the guard's `stop_owned` (SIGTERM to the group, SIGKILL 20 s later) kill the binary but not the ffmpeg it started. 458 had the same behaviour.
- Once the binary dies, launchd adopts the ffmpeg. After that, a ppid walk from the helper cannot find it.

CC0 and I agreed on six requirements for the fix. Each one is met as follows:

| CC0's check | How 528 meets it |
|---|---|
| 1. Targets only from the helper's own descendants, found by a ppid walk on one snapshot; never by name, and never its own group twice | `escaped()` walks one `/bin/ps -ww -A` snapshot by ppid from the helper's pid and keeps only processes outside its group. Nothing matches by name: there is no pkill or killall. The own group is signalled once with SIGTERM and once with SIGKILL. |
| 2. Identity re-checked before every signal, the delayed SIGKILL included | `hit()` takes a fresh snapshot and signals a target only if its pid still has the same pgid, start time (`lstart`) and command. Anything else is skipped and logged as `gone` or `another process now`. |
| 3. The spawn race: prevent it or state it as a limit | On the helper's own stops (timeout, exception, a signal while the binary lives), the binary gets SIGSTOP before the snapshot, so it cannot start another ffmpeg. The guard's group SIGTERM kills the binary first, so a 1 s tracker covers that case, and the gap is stated under Limits. |
| 4. If ps cannot start, fall back to the own group and record that the sweep did not run | `look()` counts a miss. That sweep is recorded as `not run: ps could not start; no target was signalled`. The own group is still ended. |
| 5. cleanup.json lists targets found, signalled and skipped, and the notes match the code | `<chapter>.cleanup.json` holds the reason, step, group, child_exit, targets, each signal's `sent` and `skipped` (with the reason), polls and missed. The docstring lists the same nine steps. |
| 6. A light test with a stand-in grandchild in its own group, without decodes or hashing | test528, below |

## What 528 changes

- **Tracking during each render.** `run_owned()` now waits in 1 s steps (`POLL_SECONDS`). After each step it takes one ps snapshot and adds every escaped descendant to `STATE['targets']`, keyed by pid and start time. The list survives the binary's death, so the guard's own group SIGTERM, which kills the binary first, still leaves the helper able to end that ffmpeg by identity. The render timeout is still the time left of the shared 14,400 s.
- **`end_group()`** runs on a timeout, an exception, SIGTERM, SIGINT or SIGHUP. It:
  1. ignores further stop signals;
  2. sends SIGSTOP to a binary that is still running;
  3. takes a fresh snapshot and adds new escaped descendants;
  4. sends SIGTERM to every verified target;
  5. sends SIGTERM to its own group, then SIGCONT to the binary;
  6. waits up to 6 s (`TERM_GRACE`, under the guard's 20 s);
  7. sends SIGKILL to the targets that are still verified as the same process;
  8. writes the record;
  9. sends SIGKILL to its own group, itself included, so the receipt shows -9.

  If any step from 2 to 7 raises, the error is recorded, and steps 8 and 9 still run.
- **A render that exits by itself** (answers CC0 526 N2 and 516 open point 3). `settle()` gives any tracked escaped descendant 6 s to end on its own; an encoder stops at end of input. Any that remain the same process then get SIGTERM and, 2 s later, SIGKILL, and the helper fails with exit 1. Leftovers are swept by identity, and the exit stays 1 instead of -9.
- **Records.** Each passed chapter also carries `tracker` (polls, missed, escaped_seen) in the printed line and the summary.
- **Names only.** The jobs are `apportionment-frederick-preview528` and `…-check`, the summary is `preview528-summary.json`, the printed label is `RECOVERY528 COMMAND`, and the refusal messages say 528.

**Unchanged from 516** (test T1 and an AST comparison of the top-level statements):
- The functions `sha`, `say`, `own_group`, `on_stop`, `read_receipt`, `guard_binding`, `success`, `listing`, `pressure`, `stems`, `wav` and `chapter` are AST-equal; `own_group` and `read_receipt` differ only in the number 516/528 in their messages.
- At the top level, only the docstring, `JOB`/`CHECK_JOB` and `STATE` differ, and `POLL_SECONDS` and `PS` are new. Every pin, the policy, `RENDER_SECONDS = 14400`, `TERM_GRACE`, the x264 string and the encoder tags are 516's.
- `main()` performs the same steps in the same order. `render()` uses the same command and flags. `render_all()` keeps the pressure stop and the shared budget.

## Root's commands

Run them from the repository root, with `PY=/opt/homebrew/opt/python@3.15/bin/python3.15`:

```
PY -I -B Q/guard_owner80g_target20g.py --phase native --job apportionment-frederick-preview528-check \
   --inputs MANIFEST528 -- PY -I -B Q/cc2-apportionment-preview-recovery528.py --check-only
PY -I -B Q/guard_owner80g_target20g.py --phase native --job apportionment-frederick-preview528 \
   --inputs MANIFEST528 -- PY -I -B Q/cc2-apportionment-preview-recovery528.py
```

- Both modes run only as the guard's own command; the helper refuses otherwise. The check-only mode null-decodes the eight 01–08 movies.
- The output is the new `B/apportionment-frederick-preview528/preview/`. The cleanup records, if any, go in the job folder.
- **MANIFEST528 is root's to freeze** (CC0 526 L1, with the helper row now 528's):
  - every one of the 3,820 rows of `5817cb64`, at the same sha256;
  - one row for `Q/cc2-apportionment-preview-recovery528.py` at `4206b9fc…`;
  - 52 rows for `B/apportionment-frederick-preview458/**`, whose sorted `sha256  bytes  path` lines (relative to that folder) hash to `fc5100da…`;
  - so 3,873 rows plus anything root adds. Root's freeze note gives the manifest's sha256; 528 records it through `guard.inputs_sha256`.
- **Lane order and budget** are as in 516's open points 7–9: by default the 14,400 s budget is whole, about 30–45 minutes of rendering are expected, and root decides when the job runs.

## Evidence: test528 run01 (stand-ins only), 06:46–06:48Z

`test528.py` (`0941c247…`) ran once on this Mac with child `child528.py` (`0f69a2f1…`) and renderer stand-in `renderer528.py` (`49ab2a0d…`). The output is `9f8aa179…`: **12 of 12 passed**, Python 3.15.0. These files are kept outside the repository and wrote only there.
- No helper's `main()`, render, decode, ffprobe or input hashing ran.
- The only processes were Python stand-ins, `/bin/ps` and `/bin/sleep`.
- The stand-in binary works as fmn does: it starts `/bin/sleep 300` with `posix_spawn` and `setpgroup=0` (a group of its own) and has no SIGTERM handler.
- The helper stand-in runs a helper's real `run_owned()` with its real `on_stop` handlers, as the guard's `start_new_session` child.
- Afterwards no stand-in from the test was left running. I checked that with ps.

| Case | Shows |
|---|---|
| T1 | The functions listed above are AST-equal to sealed 516's. Changed: `end_group`, `main`, `render`, `render_all`, `run_owned`. Added: `table`, `escaped`, `look`, `same`, `hit`, `alive_targets`, `poke`, `sweep`, `settle` |
| T2 | **Control on sealed 516:** after a guard-style `killpg(SIGTERM)` 2.5 s into the run, the helper ends with -9, and the stand-in ffmpeg, in its own group, **is still running**. This reproduces CC0 526 M1. The test then ended it by its verified identity (T2b) |
| T3 | **528, the same guard-style stop:** the helper ends with -9. The record lists the target (pid = pgid, `/bin/sleep`), SIGTERM `sent` to it, and SIGKILL `skipped: gone`, after 2 polls. Nothing is left in either group |
| T4 | **528, its own 4 s timeout:** the reason is `TimeoutExpired`. The binary was stopped, the target was sent SIGTERM, and the binary took its SIGTERM after SIGCONT (child_exit -15). The helper ends with -9 and nothing is left |
| T5 | **528, a render that exits 1 and leaves its ffmpeg:** after the 6 s grace the target is sent SIGTERM by identity. The helper fails with exit 1 (`escaped descendants outlived the render`) and writes no cleanup record. Nothing is left |
| T6 | **Identity:** a target whose pid is alive with another start time is skipped (`another process now`) and survives. The true identity is signalled and ends, and a later SIGKILL is skipped as `gone` |
| T7 | **ps unavailable:** every poll counts as missed (3 polls, 6 misses). Both sweeps record `not run` and signal nothing. The own group is still ended (-9). The escaped stand-in survives, as documented; the test ended it by identity (T7b) |
| T8 | With stand-in renders of 5,000 s, the timeouts are 14,400, 9,400 and 4,400 s, and it stops before 12 with 09–11 done. At 5,334 processes it stops before 09 without rendering (T8b) |
| T9 | 516 and 528 are unchanged by the test |

## Limits

- **The guard-stop gap.** Suppose an ffmpeg starts less than one 1 s poll before a stop that kills the binary first, which is the guard's group SIGTERM. Then the helper never saw it, and it survives. An encoder ends at end of input once its pipe closes; a mux, which reads files, runs to completion.
  - Each render attempt starts only a few ffmpeg processes (the probe, the encoder, the mux), so such a window is short and rare. I did not count the starts per chapter from the source.
  - The helper's own stops have no gap: the binary is stopped before the snapshot. SIGSTOP takes effect when the binary next leaves the kernel, and ps starts milliseconds later.
- **Partial files.** An ffmpeg that receives SIGTERM may still close its output, and a stopped binary never publishes. A stopped chapter can therefore leave partial or temporary files in `preview528/`. Only chapters that pass `chapter()` count; nothing is deleted.
- **Identity precision.** ps reports `lstart` to the second. A recycled pid would have to have the same pgid, the same command and the same start second to pass, and macOS assigns pids in increasing order. I take that as negligible, but it is not proven.
- **Cost.** One `/bin/ps -A` per second of rendering (about 3,500 rows) is about 2,700 short processes over 45 minutes, each of which ends before the next. Under EAGAIN, ps fails, and the poll counts as missed instead of crashing.
- **Not covered:** a signal the helper cannot catch (SIGKILL sent to the helper alone). As before, the guard's 20 s SIGKILL ends the group, and escaped ffmpeg behaves as in 458.
- **Process pressure, chapter 12's split stem, layouts, 462/468 and the budget** are as in 516's open points 2 and 4–7.

## What I ran before 528 (already disclosed in 516's notes)

- **The deviation, already disclosed.** `test516.py` run01 (`8d52a733`, output `6cfed89a`) ran the draft 7547515c's pre-render checks **unguarded** three times: `--check-only` once, and two `main()` refusal cases with `render()` replaced.
  - That was 24 null decodes of the 540p movies and three passes over the 1.57 GB of inputs.
  - It happened before root's direction that check-only must be guarded, and 516's notes ("Evidence and what I ran") disclose it.
  - I have not repeated it.
- `check516.py` run02 (`1aeff388`, output `175077ac`) was read only. It ran ffprobe and eight null decodes on this Mac, also disclosed in 516's notes.
- For 528 I ran only test528 above. I ran no decode, ffprobe, render or input hashing, and no guard.

## No claims

- This is technical evidence from source reading and stand-in process tests. It accepts no picture, motion, listening, loudness, fine sync, captions, 4K or independent AV result.
- It authorizes no launch beyond root's guarded run after CC0's review, and no publication.
- I heard no audio, looked at no frame, and have no result for 09–15. The binary's behaviour on SIGTERM comes from the source (my reading of the spawn path, and CC0's search for a handler); I did not observe it on the binary.
