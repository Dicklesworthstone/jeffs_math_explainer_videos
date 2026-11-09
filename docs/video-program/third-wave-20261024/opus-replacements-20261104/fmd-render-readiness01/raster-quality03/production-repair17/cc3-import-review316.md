# Review 316: retained-final importer 311 against master 293 and 307 B1

cc3, 2026-11-08. Independent read-only review. I did not edit any CC2 file, run a transfer, export, send, prepare or receive, start a job, or change Beads, TODO or source. Paths use the helper's own names: D (the import host's project root), Q (`D/qualification01/production-repair17`), and TRJ, CSS and HZ2 (the three source roots).

## Files reviewed

| File | sha256 |
|---|---|
| `Q/csd-retained-import311.py`, the helper (858 lines) | `862968830a076c2030599b65e1f48b776501f87d99045cb662570045e613d5ea` |
| `Q/csd-retained-import311-test.py` (595 lines) | `ff39db345cc719ed667c0af836d89dbc3d7177b9a9832741e5f20abe9fe6d74a` |
| `Q/cc2-import311-notes.md`, the completed handoff | `22f3a0ce7d17024f8520e38c31ada1966e276729569c5e500c851e5fa46fa1e8` |
| `Q/cc0-combined-master293.py` (570 lines) | `8745c179d42a3ae05f5c47236e673c29dbb020d78da1de04802737a07daa06e5` |
| `Q/cc2-master-review307.md`, B1 at :37–57 | `dcf5e307ee94956318d9859f9aee91fe570be3c367f8c88b3e4a43783b41e837` |

I read the helper, the notes and 293 in full, and the test where it writes. The helper is byte-identical to `Q/import311-readonly-snapshot305.py`.

Root's actual records, read-only:

| File | sha256 |
|---|---|
| `Q/retained-export305-trj.json` | `3c6b52e27c3b97b928d8a1bca93b4056a3ee67caa1f6d5e3c74291fd3f1755b2` |
| `Q/retained-export305-css.json` | `589a03b6097fbe5e70e3a1157816bf1b42a495d270cc54913986d30ecf79e2c8` |
| `Q/retained-export305-hz2.json` | `dd354d223b2e8c96ac00addb71f4b727f5d26d904b9f1f501164b2916f448820` |
| `Q/retained-export305-{trj,css,hz2}-receipt.json` | `9fd9c05b…8bf77`, `8e2b5626…c7d39`, `0cae8080…2ec70` |
| `Q/retained-csd-preflight305.json` | `a2a558fe8a18c51e04b6ea819d1f16c4fe0e54019e2cb9e2de0957453b9c558c` |
| `Q/import311-check305-{trj,css,hz2}.json` | `0e5c1a73…`, `11c63ae0…`, `4e1d57c1…` |
| `Q/import311-stage305-{csd,trj,css,hz2}.json` | `e00b5274…`, `d4a285ac…`, `cc895f21…`, `5a5aa1d5…` |

Guard chain, from local copies at the hashes Q binds:
- the guard (293 `GUARD`): `f1cae1dbc80075311b0e1a2abf8637439d2d8d0edbc643e771e2eefe2b3bdfb6`;
- the owner wrapper `guard_owner56g_target20g.py`: `1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097`;
- the bounded runner `run_bounded_phase_oliver_owner48g_target14g_20261102.py`: `819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da`.

Retained workers: `trj-final-chapter19-02.py` `788f7566…`, `css-final-chapter78.py` `8edccab9…`, `hz2-final-chapter53.py` `20f5c3a9…`.

## Result

**No High or Medium defects. One Low finding, which needs a sentence in the notes and no code change.**

The helper can run as written for all three hosts. Root's actual exports pass the helper's own manifest and closure rules, and 293 can consume what a completed import leaves. 307 B1 closes only when root runs the three imports and puts `rows293` into 293's manifest. 293's own `check` and `launch` then decide; this review does not stand in for them.

## Finding

### L1 (Low, notes only). An attempt that never records `finished` blocks every later attempt for that host

- **Helper:** `earlier_attempts()` (:585–598) refuses attempt N whenever an earlier attempt's job folder exists without a `finished` receipt (:590–595). A missing receipt reads as `{}` (:593), so it counts too.
- **Runner:** it creates the job folder (runner :107) before the first receipt save (:133 or :143). It writes `finished` only in its `finally` block (:176–178), and installs no signal handler. So `finished` is never written in these cases:
  - SIGTERM, SIGHUP or SIGKILL to the guard process;
  - a host restart;
  - the guard dying between :107 and :143.
- **Effect:** `prepare --attempt 2` and every later attempt then refuse, with `earlier attempt still running or unrecorded`.
- **The refusal is correct.** The child runs in its own session (runner :147–148) and does not inherit the lane lock (default `close_fds`). A receive could therefore still be writing after its guard has died and the lane has been freed.
- **The notes gap:** notes §10 (:203–210) says "use the next attempt number" after a failure before or during placement. That does not hold in this case. The way forward is root's decision once no receive process remains, not a new attempt number.
- **Fix:** one sentence in §10. No new machinery is needed. SIGINT is not affected: it raises KeyboardInterrupt, and the `finally` block runs.

## What I checked

### 1. Actual source closure pins

- **Pins.** The `CHAPTERS` pins (:102–151) equal 293 `RETAINED` (:82–115) for worker, job, inputs, permit, permit sha, movie sha and frames on all four chapters. They also equal the local terminal records:
  - `final02-terminal167`, `final03-terminal115`, `final13-terminal200` and `final15-terminal81`;
  - fields checked: receipt, summary and inputs hashes, terminal success, policy 10/8/4/56/20, summary chapter, frames, binary and permit, and command flags.
- **Export rows match the local records.** In the actual exports, the row for each chapter's resource receipt, render summary and movie has the hash of the local terminal copy. The worker, permit and render-inputs rows equal the pins. That is 16 of 16 rows.
- **Export closure.** Each export ran `manifest_problems` and `closure_problems` on the source's real bytes (:657–662) and exited 0 with no refusal and no unavailable file. All three were run with helper `86296883…` (the export receipts). `beyond_rule97` is empty for all three, so every closure path is inside the 97 rule.
- **My re-check.** Re-running `manifest_problems` locally on the three exports gives `[]` for each. The receipt totals equal the manifest totals.

| Host | Foreign files / bytes | Shared D rows / bytes | Guard-measured growth 2·S |
|---|---|---|---|
| trj (02, 03) | 158 / 165,726,504 | 589 / 1,266,896,765 | 0.309 GiB |
| css (13) | 858 / 1,512,774,545 | 52 / 874,042,513 | 2.818 GiB |
| hz2 (15) | 854 / 1,492,418,495 | 52 / 874,039,915 | 2.780 GiB |

- **Against the earlier exports.** CSS and HZ2 differ from `final13-export97.json` and `final15-export56.json` by exactly one row each: the earlier export script's own file.
  - CSS lacks `qualification01/css-final13-export97.py` (1,576 B).
  - HZ2 lacks `qualification01/production19/hz2-final15-export56.py` (1,622 B).
  - No row changed hash, and no row was added.
- **Shared rows across exports.** The three exports share 595 unique paths with no hash conflict across them. Root's preflight finds the same 595 at their hashes on the import host, with `problems: []`. It also finds all three foreign roots absent and not links.

### 2. Safe, no-overwrite transfer

- **Source side is read-only.** `export` writes only to stdout and refuses symlinks in the closure (:617–666). `send` re-hashes every foreign file before the first byte (:679–682) and again while streaming (:684–692).
- **Shared rows are never written.** D rows are marked `shared` (:221–227), checked at :285–286, and never streamed. `plan_problems` requires each one to be a symlink-free regular file at its hash before any write (:487–490).
- **Staging writes only inside the job.**
  - The runner creates the job folder and refuses an existing one (runner :107). `incoming` is created inside it (:779).
  - Each new file is opened `'xb'` (:529).
  - An identical existing file is read to devnull, so the stream stays aligned (:529, :537).
  - The hash and trailing-data checks run before any placement (:536–538). The closure is then checked again from the staged bytes (:782–788).
- **Placement replaces nothing.** `os.link` refuses an existing path (:564). The staged name is kept (:552–571). A differing existing destination is refused before the first write (:499–500). The alias is created only when absent, or accepted only if it already points at this physical root (:475–478, :574–582).
- **Results never overwrite.** `import-result.json` (:802) and the guard inputs (:739) are opened `'x'`. `prepare` also refuses an existing job or inputs name (:727–730).
- **Root's checks agree.** On the import host, root's `check` for attempt 1 reports, per host:
  - `problems: []`;
  - `new_files` equal to the foreign count, and `existing_identical: 0`;
  - alias `absent`;
  - `job_exists: false`.

  The staged helper and the three manifests there have the hashes above (`import311-stage305-csd.json`).
- **The test.** The CC2 test passed 12 of 12 here under Python 3.15.0, with `--tmp` set to my scratch folder (tree `import311-test-2dedge4q`). It wrote only there. The repository's status count was the same before and after.

### 3. Guard policy and exclusive lane

- **Command.** `guard_command` (:210–213) runs 293's `GUARD`, `--phase native`, with `--inputs Q/<job>-inputs.json`. Then comes `--` and the exact receive command (:205–207).
- **Receive checks the receipt.** `receive` asserts:
  - the receipt is admitted, with no `exit` or `finished` (:757);
  - the policy equals `POLICY` 10/8/4/56/20 (:758, :62–63);
  - the recorded command is this receive (:759);
  - the inputs file is at the admitted hash, and every row in it still matches (:760–767);
  - each shared row is frozen at its manifest hash (:774).
- **Guard settings.** The guard asserts the host name and native admission 10, stop 8, cap 4, work 56 and target 20. It sets `WORK = D/jobs` and `TARGET = D/work`. With `ROOT = D`, the lane lock is `D/expanded-native-lane.lock`. The runner takes it with a blocking `flock` before admission (runner :101–105). The same lock serves 293's own launch (293 :461), so the three imports run one at a time and never overlap the master.
- **Growth accounting.** The runner's `allocated()` deduplicates hard links within one walk (runner :36–52). It walks `D/jobs` and `D/work` separately (:55–58), so an import measures 2·S, as the comment at :65–69 says. All three fit `GROWTH_LIMIT` = 3.5 GiB (:69). The smallest margin is HZ2's, at 0.720 GiB. Block rounding adds at most one block per file in each tree.
- **Free space.** The free-space rule needs 10 GiB free before the import and 9 GiB left after it (:716–717). That is consistent with the 8 GiB stop. Preflight free is 112,376,020,992 B (104.66 GiB). All three imports together use 3,170,919,544 B (2.953 GiB) of disk.

### 4. Stable success receipts

- **Predicates agree.** 293 `success()` (:129–134) checks the fields the runner writes at :167–178, and the helper's `terminal()` (:177–181) checks the same ones.
- **Exit 0 means the result exists.** `receive` returns 0 only after `import-result.json` is written with `'x'` (:802–806). A success receipt therefore always has an import result beside it.
- **Receipts are never reused.** Every attempt gets a new job name (:193–194). The runner refuses an existing folder (runner :107), and nothing rewrites a finished receipt.
- **Failures stop 293.** A failed or timed-out attempt finishes with a non-zero exit: 124 from the 14,400 s timeout, or 75 on a resource stop. 293's `launch` then stops there and does not retry (:439–447).

### 5. 293 can consume the imported paths

- **Row fields.** `rows293` (:745–746, :800) gives exactly 293's `IMPORT_KEYS` (:117). `imported()` (:259–271) then holds:
  - the receipt is `D/jobs/<job>/resource-receipt.json`;
  - the inputs file is under Q;
  - the receipt's `inputs_sha256` is the sha of that inputs file (runner :73, :136).
- **Workers load through the alias.** `chapter_gate` reads worker, inputs and permit with `absolute(real=False)` (:292). Each worker's literal `ROOT` equals its host constant (:315–316).
- **Worker `check()` runs as written on the import host.** The three workers differ only in:
  - root, narration folder, check receipts, delivery-inputs name and job prefix;
  - the PATH line and default thread count.

  None of them asserts a host-only alias (css worker `check()` :22–39). They read imported files from ROOT and the import host's own files from D, all held in the closure.
- **Stem files are in the closure.** `verify_tables` reads the score21 stems. Chapter 13 uses `13_real_integral.music.f32.parts.tsv` and its two part WAVs; the others use the single stem. All of these are shared rows in their export.
- **Soundtrack captures match.** The export walks each job folder in full (:643). The imported `render/soundtrack/<chapter>` folder therefore holds what the source holds, so the glob at 293 :370 finds the same captures.
- **Device checks hold.** `D/work` is required to be on the `D/jobs` device at :473–474, :484, :559, :568 and :795, so 293's `st_dev` check (:377–379) holds.
- **The master's hard links work and add no measured growth.**
  - `os.link` at 293 :500 resolves the alias directory and links a regular file on the same device.
  - Each inode is already counted in `D/jobs` through its kept staged name, so the master's links into `D/jobs/<293 job>/final` add no measured growth there.

## Observations (no change asked)

1. **Files that must stay unchanged until 293 has finished.** The TRJ import inputs will bind 592 files:
   - the helper, the manifest and the guard;
   - 589 shared rows, including 12 under `Q/audio17` and 316 Frederick take downloads and requests under `D/work/production-repo07/narration_lab/…`.

   The CSS and HZ2 inputs bind 55 files each. 293 re-hashes all of them in `imported()` (:268–269), again at the end of its master job (293 :541–542), and in the runner's final input check (runner :168–170).

   The master and captions consumers run with their cwd in `D/work/production-repo07` (293 :521) and write to `--out`. The notes say "the import touches nothing there" (:201). That is true for writes, but the import does freeze these files for 293. They are take and evidence files, so a change is unlikely.
2. **What `imported()` binds.** `imported()` binds provenance only: the helper, manifest, guard and shared rows that `prepare` lists (:736–740). The imported foreign bytes are bound by 293's own per-file hashes: render inputs :331–332, summary outputs :345–346, permit evidence :217–218, tables and narration :234–255, staged outputs :378–379.
3. **The lane wait has no timeout.** The 4-hour timeout (:72–74) starts with the receive child, but the `flock` wait before admission (runner :105) has no limit. Notes §4 (:62–63) already says to start the pipe only while the lane is free.
4. **A torn receipt read fails safe.** `receive` reads its own receipt once (:756). The runner rewrites that file with a non-atomic `write_text` (runner :110–111), including once soon after the child starts (:151–160). A read during a rewrite would fail as invalid JSON. The failure is closed, costs one attempt number, and finishes with a recorded exit. 293's master reads its own receipt the same way (:472).
5. **The measured growth stays, but there is room.** The double count is permanent, as notes §8 says. On the notes' figures of 6.3 GiB in `D/jobs` and 2.2 GiB in `D/work` at preview274, all three imports bring them to about 9.3 and 5.2 GiB.
   - The 56 GiB work cap is absolute.
   - The target growth since the baseline cannot exceed `D/work`'s total, so it stays under 20 GiB without root's baseline figure.

   The guard re-measures at every admission.
6. **Superseded open items.** Notes §5 (:98) and §12 (:239–240) list the TRJ sizes and TRJ's non-local inputs as unknown. Root's export305, preflight305 and check305 records now settle both. No edit is needed for the import to run.

## Method and limits

- My checks were scratch scripts that read Q and wrote nothing in the repository. They:
  - loaded the helper and 293 as modules;
  - compared pins against terminal copies;
  - ran `manifest_problems` on the exports;
  - diffed the exports against export97/56;
  - grouped the shared rows;
  - diffed the three workers.
- I did not read `Q/import311-root-review305.json`, so that this review stays independent of root's.
- I did not run 293, and I did not reach any remote host.
- This review makes no listening, picture, motion, caption, 4K or AV acceptance claim. The Frederick narration files are compared only by hash records. 308 and 315 are unchanged.
