# 471 notes: launching 08 on the second render host with the existing 352 worker (CC3)

CC3, 2026-11-09 about 05:00 UTC. This is source reading only. I contacted no host, signalled nothing and launched nothing. `cc3-render-queue-offload471.md` is kept unchanged; these notes correct it.

Symbols:
- `CSD` is the build host, `CSS` the second render host.
- `W` is the worker's `CSS` root constant, and `CQ = W/qualification01`.
- `D` is the worker's `D` tree on `CSD`; 471's `R` is the same tree (the launcher's `R` constant).

## Corrections to 471

1. **Time.** 471's header says about 05:00 UTC. It was written at about 04:46Z and last edited at about 04:48Z.
2. **Space.** Root's live df shows 30,088,244 KiB available on `CSS`'s root volume (28.7 GiB) and 91,475,328 KiB on the SSD (87.2 GiB; 471 said about 98).
   - The guard's 10 GiB admission is possible on the root volume, so 471's item 3 (a new SSD worker) and the reason behind its "withdraw nothing" no longer hold.
   - The permit and its transfer use existing 352 modes. Root has done both: permit 474 and transfer exit 0.
3. **SIGSTOP and the lock queue.** 471's withdrawal step 2 is wrong: after the STOP, `/proc/locks` will not show the stopped waiter as `->`.
   - The pending signal ends the waiter's interruptible flock wait, and the kernel drops its blocked request. The request re-queues only when the call restarts after SIGCONT.
   - So check `->` before the STOP. After it, poll until the state is T, then check that the PID is no holder and the job folder is absent.
   - CC0's `cc0-08-withdrawal-check477.md` (`728d4fed`) has the corrected steps. CC0 handles them.

## The 08 launch (`CSS`, after the `CSD` waiter is withdrawn and recorded)

Root's `css-remaining-after-transfer386.py` (`4631ec64`) launched 12, 14 and 05 in this form. Here it is for 08, without `--after`:

```
/usr/bin/python3 -I -B CQ/css-final-repair352.py launch --chapter 08_loop_sums \
    --permit CQ/repair352-permits/<08 permit 474 file name> --permit-sha256 <full sha256, 92119d3a…> \
    --permit-transfer <permit 474 transfer job name> --threads 16
```

- **Names.** The worker is `1074051b`.
  - The permit must sit directly in `CQ/repair352-permits` with a name starting `08_loop_sums-`.
  - The transfer job name must start `css-repair-permit352-08-loop-sums`.
  - Following 370 and 375, these would be `08_loop_sums-css-permit474.json` and `css-repair-permit352-08-loop-sums474`. Use the actual names.
- **Session.** Run it in a new tmux session: `launch` blocks until the render ends.

## Material constraints

1. **No `--after`.** It reads only `W/jobs/<name>/resource-receipt.json`, and 442's receipt is under `RECOVERY`, so it would wait 12 h and then refuse. 442 is terminal, so the lane is free.
2. **Withdraw first.** Nothing in `launch` looks at `CSD`. The `CSD` duplicate check in `manifest` ran only once, when the manifest was made. Launch only after root has recorded:
   - 08's guard 1940830 killed;
   - its job folder `D/jobs/csd-reviewed-final-08-loop-sums352` absent;
   - `final-08_loop_sums-inputs352.json` still in `CSD` Q.

   If the folder exists, 08 stays on `CSD` and is not launched here.
3. **What launch refuses before writing anything.** A refusal at any of these points can be fixed and launch rerun:
   - the permit is not at the sha root passes, not in `repair352-permits`, or its name lacks the chapter prefix;
   - the transfer job name lacks the prefix, its receipt is not a clean success, or its `transfer-result.json` does not show it wrote this permit;
   - `check()` fails: any other `css-reviewed-final-08-loop-sums*` job, the alias, a receipt, permit fields (host `css`, the 344 key set, timeline sha), or the preview binding with its 69 sheets;
   - `W/jobs/css-reviewed-final-08-loop-sums352` exists;
   - fewer than 16 CPUs.
4. **One shot after that.** Launch then writes `CQ/final-08_loop_sums-inputs352.json` with `x`, and the guard creates `W/jobs/css-reviewed-final-08-loop-sums352` when it takes the lane. Any result but success leaves both, and 352 cannot launch 08 on `CSS` again. That includes a refused admission, a resource stop and a render error. Recovery would need a new version, as gallery needed 440 and 442.
5. **Admission and limits** (`guard_css74` `4f7edaba`, after it takes the lane).
   - It admits when `sbh check` exits 0, at least 10 GiB is free on the root volume, `W/jobs` is under 56 GiB, and `W/work` has grown at most 20 GiB over `allocation-baseline.json`.
   - During the run it stops below 8 GiB free, or when jobs plus work grow more than 4 GiB.
6. **The 03:15Z stop.** Gallery352's stop (`css05-resource-stop438.json`):
   - free space fell from 22.4 to 7.64 GiB, almost all between 03:09 and 03:14:45Z;
   - the job's counted growth stayed under 6 MB, so something outside the guard's counting used about 14.8 GiB in five minutes;
   - with 28.7 GiB now, the margin to the stop is 20.7 GiB;
   - a repeat during 08's run would spend its one attempt (item 4).

   Root's fresh live check should know what that consumer was before launching.
7. **Expected run.**
   - 14,959 frames, about 125 min at about 2.0 fps, outputs about 70 MB.
   - Success means a receipt with `finished`, exit 0, inputs equal to final inputs, no stop, and `render-summary.json` written.
8. **Optional pre-check of `check()` before the withdrawal.** This is root's choice; it uses `render` outside its guard.

   ```
   /usr/bin/python3 -I -B CQ/css-final-repair352.py render --chapter 08_loop_sums --job css-reviewed-final-08-loop-sums352 \
       --inputs CQ/final-08_loop_sums-inputs352.json --permit CQ/repair352-permits/<08 permit 474 file name> --threads 16
   ```

   - Run it only while `W/jobs/css-reviewed-final-08-loop-sums352` is absent.
   - `render` runs the full `check()`, writing nothing, then stops reading the absent receipt.
   - **Pass:** `FileNotFoundError` on `…/css-reviewed-final-08-loop-sums352/resource-receipt.json`. Any `AssertionError` is a `check()` refusal, found before 08 leaves the `CSD` queue.
   - Nothing reads `--inputs` before that point, and nothing creates it.

The 371 export and import follow the actual render. This is not picture, motion, listening, fine-sync, final-AV or publication acceptance.
