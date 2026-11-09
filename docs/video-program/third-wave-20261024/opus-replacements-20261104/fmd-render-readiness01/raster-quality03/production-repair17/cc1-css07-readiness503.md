# 503: CSS 07_one_point readiness on the existing 352 worker (CC1)

CC1 (BronzeJaguar), 2026-11-09. CSS was probed read-only at 06:04:46Z, 06:06:04Z and about 06:07Z.
- **Contacted:** CSS only. Over ssh, I ran one-shot Python reads: file hashes, `/proc`, and the running 08 job's own receipt.
- **Not done:** no writes, signals, launches, transfers or permit edits. The 504 transfer, the 08 render and the CSD 07 waiter were left alone.
- **CSD:** not contacted.

Names used below:
- **CSSR:** the CSS worker root, `jmev-css-render-worker-20261107`.
- **D:** the CSD root `jmev-stroke-probe-20261106`. On CSS, D is the alias into `CSSR/work/frozen-runtime75`.

Both sit in each host's data projects folder, exactly as in `css-permit504-transfer.sh`.

## Verdict

**Yes.** The existing CSS 352 runtime has every chapter-07 input that `check()` and `render()` read, at its pinned hash. Root's permit 504 differs from 366 only in `host`, which 352 requires to be `css`.

Three things must happen before the 07 render can start:
1. The running CSS 08 render must reach terminal success.
2. The waiting 504 permit transfer must then complete; it is the only waiter on the CSS lane.
3. Root must take the CSD 07 waiter (PID 1869541) out of the CSD queue, as with 08 in record 480.

Nothing is missing on CSS, and no new helper is needed.

## Files

| File | sha256 |
|---|---|
| `07_one_point-css-permit504.json` (root) | `f03572c6bfac1035efd422599d41bad81a28d1d8564663dc01313bfcc8ba57ce` |
| `07_one_point-csd-permit366.json` (source permit) | `d321fb6d13e017336ff335247cb9907ba9f63e971b206818c2970df1e613e598` |
| `css-07-permit-transfer504.json` (root's manifest) | `2cf5ee79ba1b8c6550d120b177841e67599b9f2729bfdd4d1fd676c058de03f2` |
| `css-permit504-transfer.sh` | `df50beb07dab9f9d2e0dd3e835926179c816737d1f9cb99097dd501c6e68f500` |
| `css-08-permit-transfer474.json` (precedent) | `9263a758055c3ca2cd0819b4648efddcfad8c17e0e46027a23417cb39b23a510` |
| `css-permit474-transfer.sh` (precedent) | `12380dc7ac933864a5e65a27881ea838a830d0d49e112cbc0d955d8912989ae3` |
| `css-final-repair352.py` (the worker) | `1074051b6063f1cd400de58422f7491d2d011f764884b63c4968b03feaf01a8a` |
| `guard_css74.py` | `4f7edaba7963af580245a6809af0386becba8056daec5d65626cc0ff1ef8ce73` |
| `css-remaining-after-transfer386.py` (root's launch pattern) | `4631ec649ce710d5c3efde5b62410295c1195dc7168779eadeba992186359443` |
| `css-runtime-delivery305.json` (runtime 309 record) | `9992cea712da1da1679009e0f1f113b3ccd63b05b2350188f8d13145e637cfd2` |
| `css05-resource-stop438.json` | `a31cfee5db36657f893a0b329e1850d8e25694f9ebf305826d5d6324146287e4` |
| `csd-launch-final352.py` | `06451123b958db9d4806e2645fd84d8e0b37d604867dae9f25d4cb3c5155859e` |

The probe scripts and outputs are kept outside Q:

| File | sha256 |
|---|---|
| `probe503.py` | `0d7105f684cc9895808428f408a073b34d5b02a38a28f70cc3745692744634ba` |
| `probe503.out` | `4539b6e69e9829ae22c4086d2bec2e0d1ca1163cfbda51fe79429cb583d4145a` |
| `probe503b.py` | `91a49bfcfd663e948558e081a1485bc9d104355df1fa00f8178ce7184ddbbdeb` |
| `probe503b.out` | `0609704e1c0f8328e9260a07b7215122280a11d3f58e71daa63527e0b10dd617` |
| `probe503c.out` | `d415f81b9c12ab845833b3153f7fe26bd353e7afc5b39b130d81f9745ba3ed1e` |

## Source permit and host-only fields

- **The diff.** `diff 366 504` is one line: `"host": "csd"` becomes `"css"`. Everything else is identical:
  - binary `983954c5`;
  - preview job 274;
  - frames 6491;
  - timeline `5bbc64ab`;
  - the evidence, source-binding and independent-review hashes;
  - review scope and open reviews;
  - every acceptance flag false.
- **The precedent.** It is the same change root made for 08: permit 474 from 375, manifest 474, then transfer 474 through `tmux complex-css-permit474`, which exited 0 at 04:57:13Z.
- **The transfer scripts.** `css-permit504-transfer.sh` differs from `css-permit474-transfer.sh` only in three names: the manifest, its sha and the job (`css-repair-permit352-07-one-point504`).
- **Already checked on CSD.** The CSD launcher calls the worker's `check()` before it queues a guard (`csd-launch-final352.py`, lines 48–50). So the 07 waiter's permit 366, with the same evidence bytes, already passed the same `review_basis` and `preview_binding` logic on CSD. The 352 worker states that this logic gives the same result on CSS through the alias.

## CSS state at 06:04–06:07Z (read only)

**The worker, guard and alias:**
- `css-final-repair352.py` is at `1074051b` and `guard_css74.py` at `4f7edaba`.
- The D alias is exact.
- The host is `superserver`, with 64 CPUs in affinity, which is enough for `--threads 16`.

**The 504 permit transfer:**
- The landed manifest `CSSR/qualification01/css-07-permit-transfer504.json` is at `2cf5ee79`.
- The frozen inputs file `css-repair-permit352-07-one-point504-inputs.json` (`82006450…`) lists the guard, the manifest and the worker, each at its hash.
- Guard PID 525805 is sleeping, and its job folder `jobs/css-repair-permit352-07-one-point504` does not exist yet. So it is waiting for the lane lock and has not been admitted. It is the only waiter: no other `guard_css74` or 352 process exists besides the 08 chain.
- **The 11 manifest destinations:**
  - The permit (`repair352-permits/07_one_point-css-permit504.json`) is absent, as required: a permit is written once.
  - Three evidence files are also absent and will be written with it: the transition frames305 manifest, transition review305, and coverage-grid305b 07 sampling.
  - Seven are already present at the same hash: preview274 07 sampling, progress305, build268 qualification and source copy, timeline269 comparison, rebind269 summary, and cc3 review315. Receive accepts these as already present.
  - Expect a receive with 4 written and 7 already present, and no "existing destination differs" refusal.

**What `check()` and `render()` read for 07** (all through the frozen copy; every one matches):

| Input | Hash |
|---|---|
| preview274 movie `07_one_point.mp4` | `2517772c` (= MOVIES and the permit's `preview_movie_sha256`) |
| preview274 `07_one_point/sampling.json` | `efa82174` |
| timeline269 07 timeline, cues, cuelog | `5bbc64ab` (= the permit's `timeline_sha256`), `ea5cf408`, `22a0e73f` |
| preview274 07 timeline, cues, cuelog | the same three hashes |
| timeline21 07 timeline | `c2e3c183` |
| score21 07 music stem | `d2692c6c`; no `.parts.tsv`, so a single stem |
| score21 qa `07_one_point.json` | `dca64c8e`, `passed: true` |
| rebind269 narration `07_1`–`07_6.wav` | all six at their runtime 309 hashes |

Also:
- Delivery 309's narration holds `07_1`–`07_6.wav`, and its `verified_cues` is 105.
- The timeline header reads `chapter 07_one_point`, `fps 60`, `frames 6491`, equal to the permit.
- Build268's chapters and score21's chapters (`all_qa_passed`) both include 07.

**Absent, as `launch()` requires:** any `jobs/css-reviewed-final-07-one-point*` folder, and `qualification01/final-07_one_point-inputs352.json`.

**The lane and space:**
- **08 is running.** Job `css-reviewed-final-08-loop-sums352` was admitted at 05:02:07Z. Render PID 170431 and native PID 170895 (with its ffmpeg child) had used 51,286 CPU-seconds by 06:06Z. There is no `finished`, exit, error or resource stop, and no `render/` output folder yet.
- **Free space.** The guard's statvfs gives 33.78 GiB free, against 10 GiB admission and an 8 GiB stop.
- **Work and target.** The work tree is 1.18 GB against the 56 GiB cap. The target tree is 2.12 GB above a baseline of 0 (2026-11-07), against the 20 GiB cap.
- **08's free-space use is small so far.** Free went 28.35 → 34.19 GiB by 05:23Z, through space freed elsewhere, then 34.19 → 33.78 GiB by 06:06Z.

**A warning from CSS 05 (record 438).** The real CSS limit is free space, not the walked trees. 05 (9,879 frames) was admitted with 24.0 GB free and fell to the 8 GiB stop in 31 minutes, while `work_bytes` and `target_bytes` stayed flat. It was stopped (exit −15). 08 has not shown that drain. If 08's finishing step uses space, 07's admission margin drops by the same amount. The guard refuses admission below 10 GiB (exit 75, nothing launched), so this fails closed.

## Exact next readiness, in order (root's actions; nothing here was run)

1. **Wait for CSS 08 to reach terminal success.** That means `jobs/css-reviewed-final-08-loop-sums352/resource-receipt.json` with `finished`, exit 0, inputs equal to final inputs, and no error or resource stop, plus its `render-summary.json`. Do not start a second 08.
2. **Let guard 525805 admit the 504 receive.** It is the only waiter, so it should be next. Success looks like this:
   - the `css-repair-permit352-07-one-point504` receipt exits 0;
   - `transfer-result.json` has `kind permit`, `all_hashes_match true` and written 4 / already present 7;
   - `repair352-permits/07_one_point-css-permit504.json` is at `f03572c6`.

   This is the first permit transfer to wait behind a render: 474 ran while the lane was free. The stream must survive the wait in the open ssh pipeline in `tmux complex-css-permit504`. If it fails (no header, or truncated), receive places nothing, but the job name 504 is used up. A retry needs a new job name under the same prefix, `css-repair-permit352-07-one-point`, and a new `freeze-transfer`. The permit name can stay, because it is still absent. Do not retry while 525805 lives.
3. **Run root's actual CSS check for 07, read only, after step 2,** as root did for 08 at 05:00:54Z. This calls the 352 worker's `check('07_one_point', <placed permit>)` on CSS. Its parts are the review basis (sampled runs from the bound frame records), the independent review315 view, 6491 frames and the carried rows. It reads the four files that only step 2 places, so it cannot pass earlier.
4. **Move the CSD 07 waiter (root alone).** Take PID 1869541 out of CSD's queue as in record 480: STOP, confirm state T, no job folder and not the lane holder, then KILL only that waiter.
   - This must happen before the CSS 07 launch. CSS cannot see CSD's jobs, and permit 366 stays valid on CSD: two valid permits name two hosts.
   - If CSD has already admitted 07, that is, `csd-reviewed-final-07-one-point352` exists, do not launch 07 on CSS.
5. **Launch once on CSS,** in a new tmux session, as root's 08 launcher (168285) and `css-remaining-after-transfer386.py` do:

   ```
   /usr/bin/python3 -I -B CSSR/qualification01/css-final-repair352.py launch \
     --chapter 07_one_point \
     --permit CSSR/qualification01/repair352-permits/07_one_point-css-permit504.json \
     --permit-sha256 f03572c6bfac1035efd422599d41bad81a28d1d8564663dc01313bfcc8ba57ce \
     --permit-transfer css-repair-permit352-07-one-point504 \
     --after css-reviewed-final-08-loop-sums352 --threads 16
   ```

   - `launch` refuses unless all of these hold: the permit sits in `repair352-permits`; transfer 504 succeeded and wrote that permit; 08 is a terminal success (`--after`); `check()` passes; and the 07 job and its inputs file do not exist.
   - It then writes `final-07_one_point-inputs352.json` and runs the guard for job `css-reviewed-final-07-one-point352`.
   - The render verifies 3840×2160 at 60/1 with 6491 frames; h264 yuv420p at `crf=16.0`; AAC at 48 kHz, 2 channels; the soundtrack capture at 6491×800 samples; and the timeline, cues and cuelog tables.

## Limits

- This is a snapshot from three CSS reads. 08's end time and the space its finishing step uses are unknown.
- I did not run `check()` for 07 on CSS (step 3 needs step 2's files), compile, render or encode anything, or contact CSD. The state of the CSD 07 waiter comes from root's records.
- This is readiness only. All final picture, motion, audio-review, fine-sync, captions, independent AV, master and publication checks remain open. Frederick remains the narration.
