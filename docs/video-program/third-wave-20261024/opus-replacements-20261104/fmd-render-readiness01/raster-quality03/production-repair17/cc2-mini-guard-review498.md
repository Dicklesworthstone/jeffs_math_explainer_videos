# Review 498: CC1 mini guard 490 against the pinned guards (CC2)

CC2 (RubyRaven), 2026-11-09, about 05:55 UTC, task 498. Q is this folder. ROOT is 490's `/Volumes/EERCHExport/jmev-complex-native-20261109`.

**Independence.** I wrote none of the files reviewed here. The guard and its tests are CC1's, the two pinned guards are root's, and 492 is CC0's.

**What I did and did not do.**
- I read the source, reran CC1's 12 cases unchanged, and added 10 small cases of my own. All of this ran in my session scratch folder, outside the repo, against copies.
- No ssh, staging, transfer, build, launch or Beads edit.
- The only file I wrote in Q is this one, reserved in Agent Mail as RubyRaven, id 6858.

**The sealed sha.** CC1 confirmed (Mail 17829) that `4a8723e6` is sealed and will not change. Root's staging record 500 names the same sha.

| Input | sha256 |
|---|---|
| `cc1-mini-guard490.py` (sealed, 162 lines) | `4a8723e6ed5f2e57f09d50bd1b1033325aed30fc1b7896e87b6ee587fd370faf` |
| `cc1-mini-guard490-receipt.md` | `50ee070846be4e36a0d9d529f81311a339b19b5c0bc3275a6491e816046436b0` |
| `engine-secondary-validation01/guard_owner56g_target20g.py` (pinned wrapper) | `1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097` |
| `run_bounded_phase_oliver_owner48g_target14g_20261102.py` (pinned original) | `819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da` |
| root's `remote-mac-mini-max01/guard01.py` (the pattern) | `cc57457f1ca26f1d4ddb5780e2f7cf4160860bbd456057d380fcb6c028c27291` |
| `mini-source492-manifest.json` / `mini-source492-receipt.md` | `e5a6d12f…` / `ca1e3dbe…` |
| `cc0-mac-source-parity489.md` | `60cb355efa166c029f0c92141515022112ca2fc7b755eac4472674d26daf80a3` |
| `root-mini-guard-stage500.json` / `root-mini-source-transfer497-failure.json` | `2425862f…` / read at 05:50Z |
| `cc1-mini-storage499.md` (CC1's read-only mini probes, Mail 17839) | `59893547f7329002b6098b74e77bb1fec7235434826f8ec45ebbe9eaaeea101a` |

All three pinned files were re-hashed at 05:52Z and are at their pins.

## Verdict

- **No code defect.** 490 loads the two pinned guards unchanged and changes only the globals it lists. Every refusal it adds fails closed, before anything is created.
- **No change to 490 is needed before the build.** The items below are about how root runs it (the input manifest, the command, the interpreter and an outer time bound), and about the storage under it, which the guard cannot see.
- **One Medium (M1), operational.** The guard's own checks and stops run on a sparse image whose ExFAT drive is stalled now (CC1's 499).
- **Three Lows (L1–L3), each fail-closed.** Each one costs a job name or time, and none of them can import or build anything wrong.
- **Five notes (N1–N5).**

## Composition: what 490 changes, and what stays the original's

**Loading.** 490's `load()` imports the wrapper from `REPO/<repository path>`.
- The wrapper finds its REPO as `parents[5]` of its own resolved path. That is `ROOT/repo`, and 490 asserts it.
- The wrapper's `load()` checks 819746fb and requires exactly two `14 * GIB` sites and one policy literal. It rewrites those three in memory to `TARGET_GROWTH_CAP_GIB`, sets 20, and executes the original with `__file__` set to the original's path.
- The original's module level only defines functions and constants. It creates nothing.

**Globals.** The original's functions read the module globals each time they are called, so setting them after `exec` takes effect.

| Global | Used by the original at | 490 sets |
|---|---|---|
| `ROOT` | the lane lock, `ROOT/expanded-native-lane.lock` (original:101) | ROOT |
| `REPO` | `statvfs` (55–56), sbh's cwd (114–115), the child's cwd (147) | ROOT/repo |
| `WORK` | `mkdir` (100), job folders (106–107), the baseline (122, 137–139), work bytes (58) | ROOT/jobs |
| `TARGET` | target bytes (58), `CARGO_TARGET_DIR` (140–141) | ROOT/target |
| `WORK_CAP_GIB`, `ADMISSION_GIB['native']`, `TARGET_GROWTH_CAP_GIB` | admission (127–129), stops (154–156), policy (119–121) | 80, 20, 20 |

- **What stays unchanged:** `STOP_FREE_GIB` 8, `CAPS` (native 4), every other phase's admission, and the child environment.
- **The asserts.** 490's lines 155–157 assert every value in the table. CC1's T8 and T9 show them in a loaded guard and in the preflight receipt's policy (20/8/4/80/20). My rerun reproduces both.
- **The original was not edited** and is not re-implemented. Its flock, `job.mkdir()`, input hashing before and after, process-group stops and exit codes run as written.

**The same pattern as root's guard01.** Root's `guard01.py` lane on mac-mini-max loads the same wrapper and then sets ROOT, WORK and TARGET in the same way. 490 also sets REPO, because its checkout copy lives in ROOT/repo. It also pins the two files by constant, where guard01 uses a pins file.

**What 490 adds before loading.** These are read-only refusals, exit 1:
- the host;
- an apfs mount at exactly one mount line;
- real, symlink-free folders on that volume (`st_dev`);
- both pins;
- Python 3.11 or later;
- sbh on PATH;
- no cache variable on exfat or msdos.

CC1's T1–T7 cover these. My M1 and E1 add a mount point with ` (` in its name, and a `CARGO_HOME` symlink that resolves into an ExFAT mount, which is refused.

## Resource, lock, input and baseline semantics

**Resources.**
- **Admission:** sbh's exit is 0, free space is at least 20 GiB, work is under 80 GiB, and the target is at most 20 GiB above the baseline.
- **Stop:** free space under 8 GiB, or this job's work plus target growth over 4 GiB, or work over 80 GiB, or the target more than 20 GiB above the baseline. The guard checks every 10 s and stops the process group with SIGTERM, then SIGKILL (exit 75).
- Free space is `statvfs(ROOT/repo)`, which is the image's own figure (see M1). Growth is counted only in ROOT/jobs and ROOT/target. Nothing measures the internal disk, where CARGO_HOME, RUSTUP_HOME and TMPDIR default to (12.9 GB free per 499), unless root points them at the image (CC1 item 6).

**The lock.** The `flock` is taken on `ROOT/expanded-native-lane.lock` before admission, and admission and inputs are checked under it. It serializes the jobs started through this ROOT, that is through 490 (see N2).

**Inputs.**
- `check_inputs` needs `{"files": [{"path": <absolute>, "sha256": …}]}`. It hashes every row under the lock, before launch and again after the command.
- A failure before launch raises after `job.mkdir()` and before the first `save()`. That leaves an empty job folder (CC1 T12), so the job name is spent.

**The baseline.** `allocation-baseline.json` in ROOT/jobs is written once, with `x`, by the first run that is admitted, is not a preflight, and whose inputs pass. It is never reset. A preflight writes none (CC1 T9). The second run leaves it byte-identical (CC1 T10). With an empty TARGET, its `target_bytes` is 0, so the target can never hold more than 20 GiB on this ROOT.

## APFS path mapping

- **The staged Cargo paths match.** All 21 absolute paths in 492's staged `engine-budget.toml`, crate `Cargo.toml` and `.cargo/config.toml` lie under `ROOT/source` (my P1). That is 490's ROOT, and they lie outside ROOT/repo, ROOT/jobs and ROOT/target.
- **The cwd does not.** The staged `.cargo/config.toml` sits at `ROOT/source/.cargo`. It is an ancestor of the crate folder `ROOT/source/workspace/explainer_corpus/complex_analysis_visual`, but not of the guard's cwd `ROOT/repo` (P2). See L3.
- **Mapping 492's manifest.** Its 4,197 rows map to absolute paths under ROOT/source (I1d). Its own rows are not in the guard's format (L1).

## M1 (Medium, operational): the guard's checks and stops run on a sparse image over a stalled ExFAT drive

**What 499 found** (CC1's read-only probes at 05:47Z and 05:49Z):
- EERCHExport is an APFS volume inside the sparse image `ee-rch-export-2.sparseimage`, 240 GiB, on the ExFAT drive USBNVME16TB.
- That drive is served by the user-space FSKit exFAT service, which was in uninterruptible wait. 18 processes were in uninterruptible wait, and the image's disk showed 0 transfers.

**Why it matters to the guard.** Every check the original makes during a run touches the image:
- `statvfs(ROOT/repo)`;
- a full `os.walk` of ROOT/jobs and ROOT/target every 10 s;
- the receipt writes.

None of these has a time limit. If the image stalls during a build, `sample()` blocks, and the stop checks (8 GiB free, 4 GiB per job, 80 and 20 GiB) do not run until it returns. The child's own writes stall too, so this is a hang rather than an overrun. But the job holds the lane lock for as long as it lasts, and the original has no wall-clock bound.

**Free space.** The guard's free-space figures are the image's own. 499 gives about 238 GiB available inside the image, and root's earlier probe gave 809 GiB free on the ExFAT drive. So the sparse image cannot overcommit today, but that depends on other jobs' use of the same drive. Admission and stop do not see the drive's free space.

**For root before the first admitted run:**
1. **Quiet the drive first.** Take 499's option 1: quiet the drive, then one bounded write test on EERCHExport. Do not admit a build while the FSKit service is stuck.
2. **An outer bound that reaches the child.** If root wraps the guard in an outer wall-clock bound, killing the guard does not stop the build:
   - the original starts the child in its own session (`start_new_session=True`);
   - a SIGTERM to Python ends it without running its `except` and `finally`, so `stop_owned` never runs.

   The bound must also signal the child's process group, whose id is the child's pid. The receipt on disk may not hold that pid yet, because the first `save()` after launch comes after the first `sample()`. Root's process-group-safe outer bound (Mail 5931) should be checked for this case.

## L1 (Low): 492's manifest cannot be the guard's `--inputs`

- 492's `files` rows are `[relative path, sha256, bytes]` lists. Given those rows, the original's `check_inputs` raises `TypeError` (my I1a). A dict row with a relative path raises `ValueError: manifest paths must be absolute` (I1b).
- Either failure happens after admission and `job.mkdir()`, so it spends a job name with an empty folder.
- **What root needs:** a separate input manifest in the guard's form, with each path written as `ROOT/source/<492 path>` and its 492 sha. 4,197 rows map cleanly (I1d), and an absolute dict row passes (I1c).
- If transfer 497 has to be redone, this manifest also proves the tree complete before launch. Record 497 shows a timeout with the remote state "pending recheck". A partial tree fails the first hash check, before anything launches.

## L2 (Low): on Python below 3.11 the version refusal is never reached

- `refusals()` runs `tree_problems()` before `tool_problems()`. `tree_problems()` hashes both pins with `hashlib.file_digest`, which only exists from 3.11.
- On Python 3.9.6, with both pins present, `refusals()` raises `AttributeError` (my V1b, run with this Mac's `/usr/bin/python3`). The 3.11 message is never printed.
- This still fails closed: exit 1 with a traceback, and the checks create nothing.
- **For root:** start 490 with an absolute 3.11-or-later interpreter path, not a bare `python3`, unless the mini's PATH is known to resolve to one.

## L3 (Low): Cargo will not find the staged config from the guard's cwd

This confirms CC1's item 3 and 489 item 5.
- **The cause.** The child runs with cwd ROOT/repo. Cargo finds `.cargo/config.toml` from the cwd and its ancestors, not from `--manifest-path`. A `cargo build --manifest-path …` run from there would build without `-Z threads=4` and `checksum-freshness`, so it would not match 268's flags.
- **The fix.** Have the command change into the crate folder, as 492 says and as 268 did. For example: `/bin/sh -c 'cd ROOT/source/workspace/explainer_corpus/complex_analysis_visual && exec cargo …'`.
- **Why not copy the config up.** Copying it to ROOT or ROOT/repo, CC1's other option, would put it outside 492's bound layout. Every job on this ROOT would then read it.
- **The same applies to rustflags.** The child inherits the launching environment. If `RUSTFLAGS` or `CARGO_ENCODED_RUSTFLAGS` is set there, Cargo ignores the config's `build.rustflags`. The guard does not check this.

## Notes

- **N1. Two volumes named EERCHExport (from CC1's 499).**
  - The mounted volume is `1248E0B7…`, from the `-2` image. The older image's volume `64A4C843…` is attached but not mounted.
  - 490 checks the mount path, the apfs type, and that ROOT, its pins and its folders are on that volume's `st_dev`. It does not check the volume UUID.
  - If the other volume were ever mounted at `/Volumes/EERCHExport` and held a ROOT with the same pins, the guard would accept it.
  - Root can check `diskutil info /Volumes/EERCHExport` against `1248E0B7…` before each launch. A successor can pin the UUID if root wants.
- **N2. The lock serializes only this ROOT's jobs.**
  - Receipt item 9 says the lock "serializes every guarded job on the mini". That holds only for jobs started through 490.
  - Transfer 497, any other guard's lane, and anything 486 starts some other way are not serialized with it.
  - Root should not start a guarded build while a transfer into ROOT/source is running. The input check refuses changed or missing listed files, but not added ones.
- **N3. The 4 GiB per-job cap and a cold build.**
  - 988 native receipts of this guard family in the 56g wrapper's WORK show at most 1.48 GiB of target growth in one job. Every one of them started on an existing target of 3.81–16.69 GiB.
  - None of them is a cold build into an empty target, so they say nothing about one. A cold build may well pass 4 GiB and stop with exit 75. That fails closed.
  - Cargo keeps finished units after a stop, so a rerun would resume and could add up to 4 GiB more. Building a cold target in several capped runs would sidestep the cap's intent, so that is root's and the owner's call, not a default.
  - Seeding TARGET before the first admitted run makes the seed the baseline (CC1 item 5).
- **N4. Use `--locked`.** A Cargo.lock rewrite would change an input. The after-command check would then raise `ValueError` after a finished build, so `--locked` in the command makes Cargo refuse up front instead.
- **N5. CC1's stand-in hard links share files with the repo.** CC1's run01 stand-in trees hard-link the two pinned files, so each now has a link count of 5. A write through any of those links would change root's pinned guard. Both were at their pins at 05:52Z. When I reran CC1's test, I gave it sha-checked copies, so the counts did not change.

## Tests (my scratch, outside the repo; Python 3.15.0, `-I -B`; no host contacted)

| Run | Result |
|---|---|
| CC1 `test490.py` (`76a015cd…`), copied from CC1's scratch (Mail 17829), run unedited against sha-checked copies of the two pins and the guard | 12 of 12, empty stderr. The output is byte-identical to CC1's `test490.out.json` (`56f6ddca…`). T10 runs `/usr/bin/true` twice under the composed guard in a stand-in tree. |
| My `delta498.py` (`9347a145…`), run01 | 10 of 10 (output `f8faa3f1…`): V1a, V1b (Python 3.9.6), I1a–I1d, P1, P2, E1, M1 |

**How my cases work.**
- Only `refusals()`, `env_problems()`, `mounts()` on fixed text, and the original's `check_inputs()` are called.
- The host, volume and mount checks are replaced in memory. No guarded command runs.
- In V1b, the stand-in ROOT holds only the `repo` folder that my setup made. The guard created nothing.

## Not done, not claimed

- **Not exercised:**
  - anything on the mini by me: the host name, the image type and the stall are CC1's 499 probes, not mine; sbh's real answer and `statvfs` on the image are untested;
  - the resource-stop path;
  - a real build;
  - whether the pins are staged at ROOT/repo.
- **Not accepted:** no build, chapter, picture or master. This review authorizes no staging or launch; root decides the command and manifest.
- **Still open:** final timestamps, pictures, motion, listening review, fine sync and independent AV review. Frederick remains the narration.
