# 471: moving the five queued 352 chapters from the build host to the second render host's SSD (CC3)

CC3, 2026-11-09 ~05:00 UTC. This is an assessment only, from Q's files and root's TODO records. I contacted neither host, stopped nothing and edited nothing. `CSD` is the build host, `CSS` the second render host and `RECOVERY` the SSD root of 440/442.

## Answer

**No move preserves the exact existing bindings.** Every chapter moved to `CSS` needs five new things. One of them is a code version.

| # | What is missing | Why |
|---|---|---|
| 1 | A permit naming host `css` | All five permits name `csd`: 06 `c43801cc`, 07 `d321fb6d`, 08 `c2d0b172`, 10 `917aca2c`, 11 `f6222de0`. Both `CSS` workers refuse a permit whose `host` is not `css` (`css-final-repair352.py:269`, `:751`). A new permit is the same JSON with `host` changed, authored by root. |
| 2 | A permit transfer carrying the newer evidence | 3 to 6 evidence or independent-review files per chapter are absent from all four `CSS` closures (392/396/397/424). By the local copies of Q, the files plus the sheets they name are about 1.6 MB for 06, about 6 MB plus two coverage-grid samplings for 07, under 0.1 MB for 08, about 19 MB for 10 and about 18 MB for 11. The existing path is `manifest`/`send` on `CSD`, then a guarded `receive` on `CSS`. That receive places files under `PERMITS` and `FROZEN`, both on `CSS`'s root volume. |
| 3 | Space on `CSS`'s root volume, or an SSD worker version | `CSS`'s root volume had 71 MiB available at 03:21Z (`host-recovery439.json`), and `guard_css74` admits only with 10 GiB free there. The generic worker (`1074051b`) puts jobs, tmp and cache on that volume. So did the receive above. Gallery352 died there at 03:15Z: free space fell from 22 GiB to 7.6 GiB in six minutes while the job's own counted bytes did not grow. The SSD worker 442 (`ce9baf6a`) is gallery-only: `assert chapter == '05_gallery'`, the failed-gallery352 chain, a fixed `JOB` in `launch_css_ssd442.py`, and legacy launch disabled. Using the SSD for 06–11 therefore needs a new worker and launcher version. That is a code change, outside this assignment and root's call. |
| 4 | A new master draft and controller | Draft 460 binds the five as `CSD` jobs `csd-reviewed-final-<ch>352` with the `CSD` worker, and controller 464 asserts that job name (`cc2-master-controller464.py:130`). |
| 5 | An export and an import per moved chapter | Each needs a `CSS` export and a `CSD` import, like 01/04/12/14. Each import is itself a job on `CSD`'s one native lane. The 371 closures list about 2.2 GB of foreign files each (392: 2.23 GB, 424: 2.25 GB); how much of that is new on `CSD` was not measured. |

**What `CSS` already has.** The runtime does cover the five. The four `CSS` closures carry the frozen runtime with binary 268, timeline269, rebind269 narration and the score21 stems. That includes 880, 170, 1282, 292 and 479 rows naming 06, 07, 08, 10 and 11, the preview243 movies for 06/08 and the preview274 movies for 07/10/11. Root can confirm on `CSS` with the read-only `css-final-repair352.py preflight` (exit 0 when nothing is missing). Note that `preflight` does not check permits.

## The queue on `CSD`, and how to withdraw a waiter without a duplicate

**What a waiting chapter has written.** From the sources: `csd-launch-final352.py`, `guard_csd02.py` (`f1cae1db`) and the base runner (`819746fb`).
- The launcher has written `Q/final-<ch>-inputs352.json`. It used open mode `x`, so the same launcher can never be rerun for that chapter.
- The guard then blocks in `fcntl.flock(LOCK_EX)` on `R/expanded-native-lane.lock`.
- The job folder `R/jobs/csd-reviewed-final-<ch>352` and its receipt are created only after the guard holds the lane.
- So a waiter has created nothing to undo, and an absent job folder proves it was never admitted.

Root's 04:24Z read of `/proc` on `CSD` found these waiting guards:

| Chapter | Frames | Waiting guard PID | Launcher PID (tmux) |
|---|---|---|---|
| 07_one_point | 6,491 | 1869541 | 1865118 (`jmev-final07-352`) |
| 10_rigidity | 7,442 | 1882972 | 1881681 (`jmev-final10-352`) |
| 11_counting | 11,932 | 1892282 | 1890401 (`jmev-final11-352`) |
| 06_one_derivative | 10,030 | 1940827 | 1939748 (`jmev-final06-352`) |
| 08_loop_sums | 14,959 | 1940830 | 1939820 (`jmev-final08-352`) |

344 must never be signalled: guard 982043, worker 982185, native 983027. Neither must the guard or launcher of any chapter that stays.

**Withdrawing one waiter, race-free.** Root does this, one chapter at a time, only while 344 still holds the lane, and only when that chapter's `CSS` launch is otherwise ready:
1. `kill -STOP <guard PID>`. A stopped waiter cannot take the lane: flock waiters retry the lock in their own context, so a stopped one never runs to acquire it, and the lane goes to a runnable waiter.
2. Check three things:
   - `R/jobs/csd-reviewed-final-<ch>352` is absent;
   - `/proc/locks` lists the PID only as a blocked waiter (`->`) on the lock file's inode;
   - the holder is 344's guard.

   If the folder exists, the chapter was admitted. Run `kill -CONT` and leave it on `CSD`.
3. `kill -KILL <guard PID>`. A SIGTERM would stay pending while the process is stopped.
4. Check again:
   - the PID is gone;
   - its launcher has exited nonzero (`check=True`);
   - the job folder is still absent;
   - the inputs file is still in place.

   Record these four facts before the `CSS` launch.

**Picking which chapters stay.** `CSD` does not admit waiters in a guaranteed order. To keep a particular chapter on `CSD`, keep its waiter and withdraw the others. The `CSS` worker's own refusal of `CSD` jobs (`:276`) reads `FROZEN`'s copy, not the live host. The withdrawal record is therefore the only proof against a duplicate.

## Expected benefit

Speeds:
- **`CSS`:** about 2.0 fps for a 4K chapter, measured from chapter 14: 11,756 frames from about 01:06Z to its exit at 02:43:39Z. The lane is exclusive, so chapters run one after another. Gallery 442 (9,879 frames, native from about 03:29Z) should free the lane at about 04:50–05:00Z.
- **`CSD`:** 344 (16,504 frames) started at 20:58:43Z and had not finished at 04:24Z. That is a mean under 0.62 fps under load 711 with swap exhausted.

| Chapter | Frames | `CSS` at 2.0 fps | `CSD` at under 0.62 fps |
|---|---|---|---|
| 06 | 10,030 | 84 min | over 4.5 h |
| 07 | 6,491 | 54 min | over 2.9 h |
| 08 | 14,959 | 125 min | over 6.7 h |
| 10 | 7,442 | 62 min | over 3.3 h |
| 11 | 11,932 | 99 min | over 5.3 h |
| all five, serial | 50,854 | about 7.1 h | over 22.8 h after 344, if the load persists; about 7.1 h if `CSD` recovers |

Storage is not the constraint:
- `CSS` job outputs run about 4.7 KB a frame (01: 28 MB, 12: 19 MB, 14: 55 MB), so the five need about 0.25 GB on the SSD. The SSD has about 98 GiB, against the guard's 10 GiB admission and 8 GiB stop.
- On `CSD`, the moved chapters would add five imports' worth of files instead of five render jobs.

The gain is real only if `CSD` stays overloaded. It arrives only after items 1–5 exist and are reviewed; until then a withdrawn waiter is a lost place in the queue, with nothing ready on `CSS`.

## Recommendation

1. **Withdraw nothing yet.** Today no existing path can render these chapters on `CSS`: the root volume is full, and the SSD worker is gallery-only.
2. **If root wants the offload:** authorize the new versions (permits, permit transfer, SSD worker and launcher, master draft and controller). Review them, then move one chapter at a time, largest first (08, 11, 06): withdraw by the four steps above, transfer, launch on `CSS`, then the next.
3. **The bottleneck itself lies outside this project:** per root's 04:20Z diagnostic, a frankengraphdb cargo build with rustc at about 95 and 58 GiB RSS, and a cass index at about 35 GiB. Only the owner can decide about those. Our own load on `CSD` is small: the five waiters are idle, and 344's renderer showed 166,556 KiB RSS.
4. **Two dates to watch:**
   - If 344 is under a 12 h native timeout, it ends near 08:58Z.
   - When 344 ends, one of the five waiters is admitted. That chapter can no longer move.

Not done: no host was contacted, no process signalled, no file staged. No permit, worker or controller was written. Everything above is source reading plus root's recorded observations.
