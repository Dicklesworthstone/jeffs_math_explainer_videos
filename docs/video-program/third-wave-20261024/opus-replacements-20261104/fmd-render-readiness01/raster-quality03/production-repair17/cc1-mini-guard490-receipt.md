# Guard 490: mac-mini-old native lane, preparation receipt (CC1)

CC1 (BronzeJaguar), 2026-11-09 UTC, task 490. Prepared only. Nothing ran on the mini: no ssh, staging, build, transfer, render or tracker edit. Root stages and launches it; CC2 reviews the delta after seal.

## Files

| File | sha256 | |
|---|---|---|
| `cc1-mini-guard490.py` | `4a8723e6ed5f2e57f09d50bd1b1033325aed30fc1b7896e87b6ee587fd370faf` | new, 162 lines, sealed |
| `engine-secondary-validation01/guard_owner56g_target20g.py` | `1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097` | loaded and pinned, unchanged |
| `run_bounded_phase_oliver_owner48g_target14g_20261102.py` | `819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da` | root's original, loaded by the 56g wrapper and pinned, unchanged; not tracked in git |
| `remote-mac-mini-max01/guard01.py` | `cc57457f1ca26f1d4ddb5780e2f7cf4160860bbd456057d380fcb6c028c27291` | root's pattern, read only, not loaded |
| `cc0-mac-source-parity489.md` | `60cb355efa166c029f0c92141515022112ca2fc7b755eac4472674d26daf80a3` | CC0's record, read for layout only |

No other file in Q was written. The tests are kept outside Q, as with 485 (hashes under Results).

## Owner authorization and configuration

- **Task 490:** rendering on mac-mini-old, with a fresh, isolated root. The caps are 20 GiB native admission, an 8 GiB stop, 4 GiB per job, 80 GiB work and 20 GiB target growth.
- **The follow-up:** USBNVME16TB is ExFAT. The root moves to the writable APFS image EERCHExport, which root creates with mode 0700. Cargo and its caches must not use ExFAT.

The file loads the 56g wrapper, which checks 819746fb and names the target cap. It then sets these globals on the loaded module, as root's `guard01.py` lanes do:

| Global | 490 | 56g wrapper | original |
|---|---|---|---|
| `ROOT` (holds the lane lock) | `/Volumes/EERCHExport/jmev-complex-native-20261109` | the original's folder | the original's folder |
| `REPO` (child cwd, statvfs, sbh cwd) | `ROOT/repo` | the checkout | the checkout |
| `WORK` (job folders, baseline) | `ROOT/jobs` | five-film-release | expanded-review-01 |
| `TARGET` (`CARGO_TARGET_DIR`) | `ROOT/target` | checkout `target` | checkout `target` |
| `WORK_CAP_GIB` | 80 | 56 | 48 |
| `ADMISSION_GIB['native']` | 20 | 10 | 20 |
| `TARGET_GROWTH_CAP_GIB` | 20 | 20 | 14 (literal) |
| `STOP_FREE_GIB`, `CAPS` | 8; native 4, tts 1, asr 1, preview 4 | same | same |

`load()` asserts all of these values after it sets them.

**Kept unchanged** (the original's code, not edited, not re-implemented):
- the exclusive `flock` on `ROOT/expanded-native-lane.lock`, taken before admission;
- `job.mkdir()`, which refuses a used job name;
- the input manifest, hashed before and after the command;
- process-group stops (SIGTERM, then SIGKILL);
- the sbh admission call;
- the stop and growth checks;
- the allocation baseline, written once with mode `x` by the first admitted run in this WORK and never reset;
- the child environment, the receipt and the exit codes.

**Refused before loading.** The file exits 1 with a message and creates nothing unless every one of these holds:
- the short host name is `mac-mini-old`;
- `/Volumes/EERCHExport` is a mount point with exactly one mount line, of type `apfs`;
- `ROOT` and `REPO` are real, symlink-free folders on that volume, and so are `WORK` and `TARGET` when they exist;
- `REPO` holds the 56g wrapper and the original at their repository paths and pins;
- Python is 3.11 or later;
- `sbh` is on PATH;
- none of `CARGO_HOME`, `RUSTUP_HOME`, `TMPDIR` or `SCCACHE_DIR`, when set, lies on an `exfat` or `msdos` mount.

## Root's run on the mini

```
python3 -I -B cc1-mini-guard490.py --phase native --job NAME --preflight-only
python3 -I -B cc1-mini-guard490.py --phase native --job NAME --inputs MANIFEST -- COMMAND...
```

The paths are absolute constants, so the file can sit anywhere. Use a new job name for each attempt. The preflight takes the lock and records sbh's answer and the policy. It writes no baseline and launches nothing.

## Known limitations and decisions for root

1. **Host name.** `mac-mini-old` is the alias in the fleet record. The machine's `hostname -s` is unverified. A different name is refused, which fails closed. The fix is a successor file with the actual name, not an edit to this one.
2. **The pinned files.** The original is not tracked in git, so a checkout does not carry it. Root says both pinned sources are already staged on the mini. They must sit at `REPO/<repository path>` under the new root.
3. **Cargo config.** The child's cwd is `REPO`, as in the original. Cargo reads `.cargo/config.toml` from the cwd and its ancestors, not from the crate's folder. For 489's item 5, either the config sits at `REPO` or `ROOT`, or the command changes into the crate before calling cargo.
4. **The per-job cap.** The 4 GiB per job counts work and target growth together. A cold build into an empty `TARGET` may pass it, which stops the process group with exit 75.
5. **The target cap.** It is 20 GiB above the baseline. The baseline is the state at the first admitted run. If `TARGET` is still empty then, the target can hold at most 20 GiB. If root fills `TARGET` from a cache first, the cache becomes the baseline. Decide whether to fill it before the first admitted run. These caps are the owner's and are unchanged; CC3's 486 should size its plan against them.
6. **Space the guard does not watch.** The guard measures free space only on EERCHExport (`statvfs(REPO)`), and growth only in `WORK` and `TARGET`. By default, `CARGO_HOME`, `RUSTUP_HOME` and `TMPDIR` are on the internal disk, which has 12 GiB free. Registry downloads, toolchain installs and linker temporaries land there unless root sets those variables to APFS folders. The guard then checks that they are not on ExFAT or FAT. This is 486's call.
7. **noowners.** The image is mounted with owners disabled, so ownership is ignored on it and the 0700 mode gives no isolation from other local users. The guard does not check modes.
8. **sbh on the mini.** The preflight receipt records sbh's stdout. Root should confirm that sbh evaluated the volume that holds `REPO`.
9. **Other phases.** These keep the original's caps (tts and asr: 12 GiB admission and 1 GiB growth; preview: 20 and 4). The lane lock serializes every guarded job on the mini.
10. **Exit codes:**
    - child's exit, or 0 for a passed preflight;
    - 75: not admitted, or a resource stop;
    - 1: a refusal before load;
    - 2: an argparse error.

    The original raises `ValueError` on a changed input and `FileExistsError` on a reused job name.

## Results (local; this Mac, Python 3.15.0)

`test490.py` (`76a015cd1eb63d2b4e35dc59f19596fb5d5f0a38604e603052177f50fadad4b7`) passes 12 of 12 cases. Its output is `test490.out.json` (`56f6ddcab5f4c5f04e8307c83131d25eaac4c9018f96e5895f661dfc08e23e6b`).

The stand-in trees hold hard links of the two pinned files. Every case loads the real 56g wrapper and the real original through the real `cc1-mini-guard490.py`. Only four things are replaced: the host name, the mount table, sbh's presence and sbh's answer. The guarded command is `/usr/bin/true`.

| Case | Shows |
|---|---|
| T1 | With the real constants here, it refuses at the host check, before the mount table is read; the mini root does not exist here |
| T2 | Mount lines parse to (point, type) from fixed text and from this Mac's own table. apfs passes; exfat is refused; two lines for one point are refused |
| T3 | `CARGO_HOME` or `TMPDIR` on exfat, or `TMPDIR` on msdos, is refused; `CARGO_HOME` on the APFS volume passes |
| T4 | A tree with both pinned files on one file system passes the tree and pin checks |
| T5 | One byte appended to the original or to the 56g wrapper is refused by that pin |
| T6 | A symlinked `REPO` is refused |
| T7 | Without sbh, `load()` refuses and no jobs folder is made |
| T8 | After load, the globals and caps in the table above hold, and the module runs the original's file |
| T9 | The original's preflight: admitted under the lock at `ROOT`, nothing launched. The receipt policy is 20/8/4/80/20; sbh is asked `--need 20G` with cwd `REPO`; no baseline |
| T10 | Two guarded runs exit 0 with inputs hashed before and after. The first writes the baseline (target 0 bytes); the second leaves it byte-identical |
| T11 | A reused job name raises `FileExistsError`, and the old receipt is untouched |
| T12 | A changed input raises `ValueError` before launch; the new job folder is left empty |

`ast` parses the guard file. All three pinned files were re-hashed after the run and are unchanged.

**Not exercised:**
- the resource-stop path and the SIGTERM and SIGKILL stops (the original's code, unchanged);
- a real sbh answer;
- statvfs on the image;
- the mini's own `mount` output.

## Limits

- This is minimal local execution evidence on stand-ins, not a run on the mini or a build. CC0's 489 and CC3's 486 supply the source and the toolchain and cache recipe; root supplies the command and manifest. CC3's APFS requirements check is pending, and nothing here launches anything.
- This makes no claim about any chapter, picture or master. All final master, technical, picture, audio-review, fine-sync, independent AV and public playback checks remain open. There is no audio-review claim. Frederick remains the narration.
