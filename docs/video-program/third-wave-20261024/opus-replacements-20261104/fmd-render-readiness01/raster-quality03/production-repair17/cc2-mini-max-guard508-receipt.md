# Guard 508: mac-mini-max native lane, preparation receipt (CC2)

CC2 (RubyRaven), 2026-11-09, about 06:05 UTC, task 508 (root's Mail 17862, confirmed by the owner's session).

**What this is.**
- Prepared only. Nothing ran on mac-mini-max: no ssh, staging, build, transfer, launch or Beads edit.
- Root stages and launches it. CC0 reviews it independently.
- Guard 490 and every mac-mini-old file are kept unchanged.

## Files

| File | sha256 | |
|---|---|---|
| `cc2-mini-max-guard508.py` | `135e4b2fe68f644d0d6b5b10494570b37d246b56e3fd2fd83748d545fa0a3e32` | new, 178 lines, sealed |
| `cc1-mini-guard490.py` | `4a8723e6ed5f2e57f09d50bd1b1033325aed30fc1b7896e87b6ee587fd370faf` | the predecessor, unchanged |
| `engine-secondary-validation01/guard_owner56g_target20g.py` | `1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097` | loaded and pinned, unchanged |
| `run_bounded_phase_oliver_owner48g_target14g_20261102.py` | `819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da` | root's original, loaded by the wrapper, pinned, unchanged |
| `mini-max-source506-manifest.json` / `-receipt.md` | `2c63f5c8…` / `b3d5091d…` | CC0's tree, read for its root and format |
| `root-mini-max-guard-sources510.json` / `root-mini-max-source-transfer511.json` | read at 06:00Z | root's staging of the two pins and the 4,197 files |
| `cc2-mini-guard-review498.md` | `b56c7d861d4f728fae774ec938211dbeada019a553ee7ad913d8a6986ed49268` | my review of 490; L2 is fixed here, and the rest carries over |

No other file in Q was written. The tests are kept outside Q (hashes under Results).

## What 508 is

508 is 490 with five changes. `load()`, the caps, and the globals it sets are byte-identical to 490's.

| | 490 | 508 |
|---|---|---|
| host | `mac-mini-old` | `mac-mini-max` |
| volume | `/Volumes/EERCHExport`, apfs | `/System/Volumes/Data`, apfs, the internal Data volume |
| ROOT | `VOLUME/jmev-complex-native-20261109` | the running account's home (from the account database, not `$HOME`) + `scratch/jmev-complex-native-20261109`, refused unless the sha256 of its path text is `6bcd5dfb…`, which is root's path in Mail 17862. The file names no account. |
| mount check | `os.path.ismount` and exactly one apfs mount line | exactly one apfs mount line only. `os.path.ismount('/System/Volumes/Data')` is False on macOS (test S2), so 490's check would refuse this volume. |
| order of refusals | host, then mounts, tree, tools, env | host, then Python and sbh, then the ROOT pin, then mounts, tree, env |
| file hash | `hashlib.file_digest` (3.11+) | chunked `hashlib.sha256`, so Python 3.9 gets the version refusal instead of a traceback (review 498 L2, test S4) |

Unchanged from 490:
- REPO = ROOT/repo, WORK = ROOT/jobs, TARGET = ROOT/target;
- work cap 80, native admission 20, target growth 20, stop 8, per job 4, and the original's other phases;
- the exclusive `flock` on `ROOT/expanded-native-lane.lock`, the immutable job folders, the input hashing before and after, the process-group stops, the baseline, the child environment, the receipt and the exit codes. These are the original's code, not edited.

**Refused before loading** (exit 1, nothing created) unless every one of these holds:
- the host is `mac-mini-max`;
- Python is 3.11 or later, and sbh is on PATH;
- ROOT is the pinned path;
- `/System/Volumes/Data` has exactly one mount line, of type apfs;
- ROOT and REPO, and WORK and TARGET when present, are real, symlink-free folders with that volume's `st_dev`;
- both pins are at `REPO/<repository path>`;
- no `CARGO_HOME`, `RUSTUP_HOME`, `TMPDIR` or `SCCACHE_DIR` is on an ExFAT or FAT mount.

## Facts this rests on (read, not run by me)

- **The host name.** Root's guard01 lane checks the host name `mac-mini-max` the same way, and it has admitted jobs there (receipts in `remote-mac-mini-max01/jobs`).
- **sbh evaluates the Data volume.** The guard01 receipt `logarithm-remote-verify01` (2026-11-05) records sbh's JSON for the checkout's path as `mount_point` `/System/Volumes/Data`, apfs, `volume_role` Data. Its `volume_available_bytes` (34,397,712,384) equals the guard's own `statvfs` free bytes at admission. That is the same measurement 508 makes from ROOT/repo.
- **The interpreter.** Those receipts ran under `/opt/homebrew/opt/python@3.14/bin/python3.14` on mac-mini-max.
- **Space and caches.** CC0's 501 gives 101.6 GiB free on the internal APFS volume, load 5.4, and all 231 registry crates and 6 git sources of 268's lock cached.
- **The source tree.** 506's two staged Cargo files hold 21 absolute paths. Each lies under `ROOT/source`, and that ROOT hashes to 508's pin (checked by hash, without printing the path). Root's 511 reports 4,197 files with listing sha `770e5239`, under ROOT/source.

## Root's run on mac-mini-max

```
PY=/opt/homebrew/opt/python@3.14/bin/python3.14      # or another absolute 3.11+ path
$PY -I -B cc2-mini-max-guard508.py --phase native --job NAME --preflight-only
$PY -I -B cc2-mini-max-guard508.py --phase native --job NAME --inputs MANIFEST -- COMMAND...
```

The guard's paths are absolute, so the file can sit anywhere. Use a new job name for each attempt.

**Before the first admitted run:**
1. **Pins at REPO.** The two pinned files must be at `ROOT/repo/<repository path>`. Record 510 gives their hashes but not where they were staged.
2. **The input manifest.** 506's manifest rows are `[relative path, sha256, bytes]` lists, which the original's `check_inputs` rejects (review 498 L1). Root needs `{"files": [{"path": "ROOT/source/<path>", "sha256": …}, …]}` with the 4,197 rows, written with ROOT as an absolute path.
3. **The command.** The child's cwd is ROOT/repo. The command must change into `ROOT/source/workspace/explainer_corpus/complex_analysis_visual` before calling cargo, so that `ROOT/source/.cargo/config.toml` applies (review 498 L3).
   - Leave `RUSTFLAGS` and `CARGO_ENCODED_RUSTFLAGS` unset.
   - `--locked` keeps a Cargo.lock rewrite from failing the after-check.
   - All locked crates are cached (501), so `--offline` is possible. That is root's call.
4. **Run a preflight first.** Check that its receipt's sbh `mount_point` is `/System/Volumes/Data` and that its path is under ROOT.

## Decisions and limitations for root

1. **The lane lock is ROOT's, as in 490.** It serializes the jobs started through 508.
   - Root's existing guard01 lane on the same host keeps its own lock, in the checkout's `remote-mac-mini-max01` folder. So a guard01 job (ASR, render or master) can run at the same time as a 508 build.
   - If the build must be exclusive across the host, either start no guard01 job during it, or take a successor that sets `guard.ROOT` to guard01's folder. The original uses ROOT only for the lock, so WORK, TARGET and REPO could stay here.
   - I kept 490's layout because helper 507 targets this ROOT.
2. **Space.**
   - **What admission and stop see:** they read the Data volume's free space, which is the shared APFS container. So they also cover `~/.cargo`, `~/.rustup` and TMPDIR, which live on the same container. 490 could not do that.
   - **What growth counts:** only ROOT/jobs and ROOT/target.
3. **The cold build.**
   - **The per-job cap.** It is 4 GiB, counting work and target growth together.
   - **No precedent.** No receipt in this guard family shows a cold build into an empty target. The incremental maximum is 1.48 GiB, over 988 receipts (review 498 N3). A cold build may stop with exit 75. Rerunning to resume past the cap is root's and the owner's call.
   - **The baseline.** It is TARGET at the first admitted, non-preflight run.
4. **An outer time bound must reach the child** (review 498 M1). The child runs in its own session, and a SIGTERM to the guard skips `stop_owned`. Signal the child's process group (pgid = child pid) as well.
5. **Helper 507.** The guard owns `ROOT/expanded-native-lane.lock`, `ROOT/jobs/` and, through Cargo, `ROOT/target/`. `ROOT/repo` holds the pins, and `ROOT/source` is the bound input tree. A helper that writes into any of these outside a guarded command would change what the guard measures or binds.
6. **noowners and modes** do not apply: the Data volume honours ownership. The guard checks no modes.
7. **Exit codes, from the original:**
   - the child's exit, or 0 for a passed preflight;
   - 75: not admitted, or a resource stop;
   - 1: a refusal before load;
   - 2: an argparse error;
   - `ValueError` on a changed input, and `FileExistsError` on a reused job name.

## Results (local; this Mac, Python 3.15.0 and `/usr/bin/python3` 3.9.6)

`test508.py` (`dfd287e4553a881b7ef1bd1304292f8c2d0c8343d3c86d79189d35d237df5a8c`) passes 8 of 8 on the sealed file (run02, output `826b46ab…`).

**What it uses:**
- this Mac's real mount table and real `st_dev`;
- stand-in ROOTs in a session scratch folder on this Mac's Data volume, outside the repo, holding copies of the two pins;
- stand-ins for the host name and for sbh's answer. The real sbh is never run.

The only guarded command is `/usr/bin/true`.

| Case | Shows |
|---|---|
| S1 | With the real constants on this Mac (`mac-studio`), it refuses at the host check before reading mounts, and ROOT is not created |
| S2 | This Mac's real table has exactly one apfs line for `/System/Volumes/Data`, and 508 accepts it. `os.path.ismount` is False for it |
| S3 | ROOT from this account's home hashes to the pin; another ROOT is refused |
| S4 | On Python 3.9.6, `refusals()` returns the 3.11 refusal rather than raising |
| S5 | A stand-in ROOT on the Data volume passes the real volume, device, tree and pin checks |
| S6 | A one-byte change to the original is refused by its pin; a symlinked REPO is refused before hashing |
| S7 | After load, ROOT/REPO/WORK/TARGET and the caps 20/8/4/80/20 hold, and the module runs the original's file |
| S8 | The original's preflight takes the lock at ROOT, asks sbh for 20G from REPO, records the policy 20/8/4/80/20 and writes no baseline. Then one guarded `/usr/bin/true` exits 0 with inputs hashed before and after, and the baseline is written (target 0 bytes) |

**Run01** (`0fcffd61…`) also passed 8 of 8, but on an earlier draft of the guard (`4bf5fb66…`). The only change since is one docstring sentence. In run01, S8 also did not yet check "no baseline after the preflight". I added that check and reran in a new folder, keeping run01.

**Not exercised:**
- anything on mac-mini-max;
- sbh's real answer;
- the resource-stop path and the SIGTERM and SIGKILL stops (the original's code, unchanged);
- a real build.

## Limits

- This is minimal local evidence on stand-ins, not a run on mac-mini-max or a build.
- It makes no claim about any chapter, picture or master. Final timestamps, pictures, motion, listening review, fine sync, independent AV review and public playback checks remain open. Frederick remains the narration.
