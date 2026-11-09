# 477: withdrawing the waiting 08 guard on CSD, a source check (CC0)

cc0, 2026-11-09, 04:59 UTC. Q is this folder, R is `/data/projects/jmev-stroke-probe-20261106` on CSD.

This is source reading only. I contacted no host, signalled no process and staged nothing. It checks CC3's 471 withdrawal steps against the pinned sources and the Linux file-lock code.

## Result

Withdrawing the 08 waiter is safe. It can leave 08 provably never admitted, and it leaves 344 untouched. That holds only if root follows 471's four steps **with two corrections**:

- **C1. 471's step 2 check is wrong for a stopped waiter.** Once `kill -STOP` takes effect, PID 1940830 is **not** in `/proc/locks` at all, not even as a `->` line. Here is why, from `fs/locks.c` (current master):
  - `flock_lock_inode_wait()` sleeps in `wait_event_interruptible()`. SIGSTOP is a pending signal, so the wait ends. The loop then calls `locks_delete_block()`, which unlinks the request from the holder's `flc_blocked_requests`.
  - `locks_show()` prints its `->` lines only from that list.
  - After `kill -CONT`, the syscall restarts and the request queues again.

  So "listed only as a blocked waiter" is the check to make **before** the STOP. **After** the STOP, the check is that 1940830 is not a holder (it should be absent entirely), and that the holder line on the lane inode is still 344's guard 982043.
- **C2. Confirm the stop before the folder check.** `kill` returns before the target has stopped. Root should poll until `/proc/1940830/stat` shows state `T`, and only then read the job folder. The STOP is what makes the folder check race-free.

## What the sources say a waiter has done

These are the pinned bytes, read here:
- launcher `csd-launch-final352.py` `06451123`;
- `guard_csd02.py` `f1cae1db`, which loads the 56g guard `1c19a9b8`, then the base runner `819746fb`, then `owned-process-stop02.py` `a2fe7985`.

**The launcher.**
1. It writes `Q/final-08_loop_sums-inputs352.json` with `open('x')`.
2. It runs the guard with `subprocess.run(..., check=True)`, and does nothing after that call. It has no retry.
3. Because of the `'x'` open, rerunning this launcher for 08 fails before any guard starts. So 08 cannot be queued on CSD again through it.

**The guard, before the lock.** `load()` writes nothing and installs no signal handler:
- it checks the host name and the pins;
- it sets `ROOT = R` and `WORK = R/jobs`;
- `owned-process-stop02.install()` only makes the process a subreaper and replaces `stop_owned`.

**The base runner `main()`**, in this order:
1. `WORK.mkdir(exist_ok=True)`. `R/jobs` already exists.
2. Open `R/expanded-native-lane.lock` in append mode.
3. `fcntl.flock(LOCK_EX)`. **The waiter is blocked here.**
4. Only after the lock is held: `job.mkdir()`, the `sbh` check, the receipt with `admitted`, the input check and the child process.

So a waiter holds no lock, has no child, and has written nothing. **`R/jobs/csd-reviewed-final-08-loop-sums352` absent is the proof that 08 was never admitted**, because `admitted` can only be recorded inside that folder.

**The later duplicate guards.**
- 371's import of 08 applies the same rule on the live CSD (`csd-css352-import371.py:505`). It refuses while `R/jobs/csd-reviewed-final-08-loop-sums*` matches anything.
- The CSS worker's check (`css-final-repair352.py:276`) reads only the FROZEN copy. That leaves the withdrawal record and 371's import check as the two live CSD checks.

**Linux behaviour.**
- A blocked flock request holds nothing, so killing it cannot release or alter the holder's lock.
- An unlock wakes waiters without granting the lock: each retries `flock_lock_inode()` itself (`__locks_wake_up_blocks`). A stopped waiter therefore cannot take the lane.
- SIGKILL ends a stopped process. SIGTERM would stay pending, because the process is stopped and has no handler.

**344.** 344 runs under the same `guard_csd02.py` lane (`csd-launch-boundary-final344.py:101`). Its guard, 982043, holds that lane. Signalling only 1940830, by number, leaves 344's lock, processes and files untouched. Root should not use:
- `pkill` or any pattern match;
- a negative (process-group) PID;
- tmux `kill-session` or `kill-server`;
- anything that opens the lock file.

Every check below is a read of `/proc` or `R/jobs`.

## The prerequisites and steps (471, corrected)

**0. Before any signal, record all of these:**
- 982043 is alive and is the holder line (no `->`) on the lane inode, which is `maj:min:inode` from `stat` of the lock file.
- `/proc/1940830/cmdline` is `guard_csd02.py --phase native --job csd-reviewed-final-08-loop-sums352 --inputs Q/final-08_loop_sums-inputs352.json -- …`.
- 1940830's parent is 1939820 (`csd-launch-final352.py --chapter 08_loop_sums`).
- 1940830 is listed as `->` on that inode, and no process has 1940830 as its parent.
- `R/jobs/csd-reviewed-final-08-loop-sums*` matches nothing.
- The sha of the inputs file.
- The CSS side is ready: 476 sealed and reviewed, permit 474 on CSS, and CSS `preflight` passed. Without that, withdrawing only loses 08's place (CC3 471).

**1.** `kill -STOP 1940830`, then poll until the state is `T` (C2).

**2.** Check:
- the glob still matches nothing;
- 1940830 is not a holder (C1: expect it absent);
- the holder is still 982043.

If the folder exists, or if 1940830 is the holder (it took the lane just before the stop), run `kill -CONT 1940830` and leave 08 on CSD.

**3.** `kill -KILL 1940830`.

**4.** Check:
- `/proc/1940830` is gone;
- 1939820 has exited nonzero (`CalledProcessError` from `check=True`);
- the glob still matches nothing;
- the inputs sha is unchanged;
- 982043 is still the holder;
- the other four waiters are still listed as `->`.

These records are 08's "never admitted on CSD" proof.

## Not checked

- CSD's actual kernel version (I read current `fs/locks.c`, where this structure is long-standing);
- root's live PIDs since 04:24Z;
- the tmux pane setup;
- anything on CSS.

No audio heard or checked. This accepts no render, import or master, and it authorises no withdrawal; root does that.
