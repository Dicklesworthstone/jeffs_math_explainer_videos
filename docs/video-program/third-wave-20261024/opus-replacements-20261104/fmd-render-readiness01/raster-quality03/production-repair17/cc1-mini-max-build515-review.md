# 518: process-control review of native build helpers 507, 515 and 521 (CC1), sealed

CC1 (BronzeJaguar), 2026-11-09, about 06:30 UTC, task 518. Q is this folder. This is an independent review of actual native launch safety and 268 parity only. I made no ssh, mini, build, render, launch, staging, permit or tracker change. I edited none of the helpers.

**What I read in full:**

| File | sha256 | Lines |
|---|---|---|
| `Q/mini-max-native-build507.py` | 8e5db00a40c58ce37fc7643de7eff0421f8aa86ac33eb7b31043d7d2c62c5cfb | 296 |
| `Q/mini-max-native-build515.py` | 34e52adb444ab17a095c46868d6a625ecd6f98811a04b093316db4d4f16a01a7 | 370 |
| `Q/mini-max-native-build521.py` | 9af15207ade5b74d756872c0af316da1cfad26bd7b2885f29801118fee544b60 | 371 |
| `Q/review-repair-build268.py` | 5f34e83711ee31fdd452f6c3435a610c662293532fb1bd5c6474cb279ebad3a0 | 42 |
| `Q/cc2-mini-max-guard508.py` | 135e4b2fe68f644d0d6b5b10494570b37d246b56e3fd2fd83748d545fa0a3e32 | — |

I also read the original guard's process and sampling code (`run_bounded_phase_oliver_owner48g_target14g_20261102.py`, 819746fb3311…).

## Verdict

**521 is safe to launch under guard508 as far as process control goes.** I found no blocking defect in it, and its build gates match 507's byte for byte, so 268 parity carries over.

Do not launch 507 or 515. Each leaves step processes that the guard cannot stop (B1 and B3 below).

## 507 (superseded)

- **B1 (blocking for 507).** `subprocess.run(timeout=)` kills cargo only. Its rustc, linker and test grandchildren are left in 507's group. 507 then exits, the guard sees its child dead and stops nothing, and the orphans keep writing ROOT/target after the lane lock is released.
- **B2.** On the guard's SIGTERM, 507 dies at once from the default action. The guard's `stop_owned` then sees a dead child and skips its SIGKILL, so a group member that ignores SIGTERM survives.

## 515 (superseded): the launch race, confirmed

- **B3 (blocking for 515).** `child = subprocess.Popen(..., process_group=0)` sits outside the `try`, and `on_stop` raises `Stopped` asynchronously. A stop signal can land in either of two windows:
  - between Popen returning and the `try` taking hold;
  - inside Popen after the fork, for example in its read of the exec-status pipe. CPython's Popen cleanup closes pipes but does not kill a started child.

  In both cases `stop_group` never runs. The helper exits 143 within milliseconds, the guard sees its child dead, and the step keeps running in a group the guard's killpg never reaches.
- **Structural residual.** Because of `process_group=0`, a SIGKILL of the helper always strands the step group. That includes the guard's own 20 s escalation.

CC3 reached the same conclusion independently and fixed it in 521.

## 521: what makes it safe

1. **One group.** Steps no longer set a group or session, so cargo, rustc, the linker and the test binaries inherit the helper's group. That group is the one the guard creates with `start_new_session=True`. The guard's SIGTERM, and its SIGKILL at 20 s, reach every process at any moment, even if the helper cannot run its handler at all.
2. **`own_group()` fails closed.** It requires pid == pgid == sid, i.e. the helper must be the guard's command itself. Under guard508 this holds, because the original guard runs the command directly with `start_new_session=True`. A wrapper such as `env`, `sh -c` or `caffeinate` makes it refuse before any step starts.
3. **`end_group()` always ends with `killpg(group, SIGKILL)`.** That SIGKILL ends the helper too, so where a signal lands no longer matters:
   - **Before the handlers are installed:** no step exists yet.
   - **Inside Popen:** `STATE['child']` is still None, so there is no wait; the record is written and the SIGKILL follows. A child that has forked but not yet exec'd is in the group, so it dies.
   - **A side note on the window between fork and exec:** Popen takes CPython's fork_exec path here, because `cwd` is set and that rules out posix_spawn. A SIGTERM landing between fork and exec may be absorbed by the inherited Python handler. The closing SIGKILL covers this case as well.
4. **Guard compatibility.** `stop_owned` sends SIGTERM to the group and waits 20 s. `end_group` needs at most 6 s plus one small JSON write. The receipt records exit -9, and the guard still returns 75 on a resource stop. On a step timeout the helper ends at -9 on its own, and the guard records that exit.
5. **No step leaves the group.** In the 506 tree, the only spawn into a new process group is fmn-platform's ffmpeg route through asupersync's `posix_spawn.absolute_path.new_process_group`. None of the six steps reaches it:
   - the tests only build MP4 options (`main.rs`, `native_image_route_keeps_export_configuration`);
   - the probe renders Y4M into a VirtualFs (`plate_fade_probe.rs:213–219`);
   - `facts` and `chapters` only print (explainer_kit `cli.rs:380`, `cli.rs:386`).

   Cargo, rustc and the linker are not in the 506 tree. I know of nothing in them that changes process group, but I did not read their source.

## Local check (stand-in steps, no Cargo)

**Setup.** A driver loads 521 unchanged and changes only `CRATE`, to a scratch folder. It runs main()'s own `own_group()`, the handler installation and `STATE['job']`, then 521's `run_step`. The runner starts the driver as the guard does, with `start_new_session=True`, and stops it with the original guard's own `stop_owned`, imported unchanged after a hash check. Group membership comes from `ps -A -o pid=,pgid=`.

**The stand-in steps.** "Stubborn" is a `sh` step and a grandchild, both ignoring SIGTERM (`trap '' TERM; … exec /bin/sleep 120`).

| Case | Result |
|---|---|
| G1: guard stop mid-step | Driver, step and grandchild shared pgid 80520; the driver led its session and group. `stop_owned` returned after 6.0 s; driver exit -9; group empty afterwards. `stubborn.cleanup.json`: reason `signal 15`, `child_exit` null. |
| G2: step timeout of 2 s | Exit -9 after 7.19 s; group empty. Cleanup record reason `TimeoutExpired(…, 2)`. |
| G3: guard stop between steps | `stop_owned` returned at once; exit -9; group empty. The record is named after the step that had just passed (`quick.cleanup.json`); see L2. |
| G4: step exits 0 but leaves `sleep 20` | Helper exit 0, no record; the `sleep 20` was still in the group afterwards; see L1. |

**Files.** All are in my session scratch, not in the repo; the run used Python 3.15.0 on this Mac.

| File | sha256 |
|---|---|
| `test518.py` | 3e19faf549c8e72e85c379bb624133c3593db61837c0596021f8a8dddd485f28 |
| `driver518.py` | 7bf5ac56fa9ba6f13d8897e6544bdf61f1cbdedba04df266f8ac9581c5ae70b0 |
| Output `test518-01.json` | e31c444d49796adc3b62a2105402b9d91fa08c701cecc549bda026e4b04fc58e |

This is independent of CC3's own test521. It agrees with CC3's L521, T521 and S521 results.

## Lows in 521 (none blocks the build)

- **L1. A normal step exit is not checked for leftovers.** 268 and 507 behave the same way, and 521's docstring says so. Cargo waits for its own children, so no leftover is expected from these six steps. G4 shows what one would look like.
- **L2. A stop between steps names the last finished step.** The record is `<that step>.cleanup.json` with `child_exit: null`. Read the reason and the null exit, not the file name, before concluding a step was cut short.
- **L3. A Popen failure (such as a missing binary) or any step exception also ends at exit -9.** The reason is only in `<step>.cleanup.json`; there is no traceback in the command log.
- **L4. A rare wait corner.** If the signal lands while Popen's wait holds its internal reap lock, `end_group`'s wait cannot reap the child. It then waits the full 6 s and records `child_exit` null. That is still well inside 20 s.

## Operational notes for root's launch

These are guard properties, not 521 defects.

- **To stop a running job, signal the helper's group:** `kill -TERM -- -<pid>`, where pid is the guard receipt's `pid`. 521 then ends everything within about 6 s.
- **Do not signal only the guard, or close its tmux pane.** The guard installs no handler, so SIGTERM or SIGHUP kills it at once and leaves the helper and its steps running without the lane lock. CC2's 498 M1 makes the same point.
- **Run 521 exactly as the guard's command:** `python3 -I -B mini-max-native-build521.py --job JOB`, with an absolute 3.11+ interpreter and no wrapper.

## 268 parity

Outside the process-control code, the diff from 507 to 521 changes only output names:
- the 507→521 labels in messages;
- `preflight521.json`;
- `complex_analysis_visual-521`;
- the `mini-max-native-build521` schema.

These match 507 byte for byte: the cargo flags, the test regex (233 passed, 0 failed, 1 ignored), the probe regex, clippy `--no-deps -D warnings`, `-C metadata=jmev_review_repair268`, facts 7d528509, the 15 chapters, listing 770e5239 (4,197 files), the environment set and the forbidden-environment checks.

Against 268, the differences remain the documented ones:
- no fmt or copy step (the 506 listing check replaces them);
- version and commit pins instead of the Linux binary hashes;
- `cargo-clippy` run with `CARGO` set, in place of `cargo clippy`;
- `RUST_TEST_THREADS=4` in place of CPU affinity;
- a 14,400 s timeout for the cold test step;
- a fixed 15-chapter list in place of build05's.

## Not done

- **Steps:** the check used `sh` and `sleep`, not Cargo. The real step processes, the mini's Python version and a real build were not exercised.
- **Guard paths not exercised:** the sampling-driven resource stop itself (I called `stop_owned` directly), a handler blocked past 20 s, and the guard's own SIGKILL.
- **Open:** final timestamps, pictures, motion, audio-review, fine sync and independent AV acceptance.
