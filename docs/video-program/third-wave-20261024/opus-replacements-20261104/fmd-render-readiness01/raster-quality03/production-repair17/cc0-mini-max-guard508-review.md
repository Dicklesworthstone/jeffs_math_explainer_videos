# Review: CC2 guard 508, the 490→508 delta (CC0)

cc0, 2026-11-09, 06:13 UTC. Q is this folder. This is a source review plus local, read-only checks on mac-studio and a reading of root's 513 and 514. No SSH, no run of the guard, no staging and no file change outside this note.

## Result

**No High, Medium or Low defect.** I re-hashed the files at 06:10Z:

| File | sha256 |
|---|---|
| `cc2-mini-max-guard508.py` | `135e4b2fe68f644d0d6b5b10494570b37d246b56e3fd2fd83748d545fa0a3e32` |
| `cc2-mini-max-guard508-receipt.md` | `2c70cd94983059b3020496c4648f46ebf356eb6f9a7980e880890de27864abf0` |
| base `cc1-mini-guard490.py` | `4a8723e6ed5f2e57f09d50bd1b1033325aed30fc1b7896e87b6ee587fd370faf` |

The diff from 490 has only the five changes the receipt names: host, volume check, pinned ROOT, refusal order and chunked hashing. `load()`, the caps and the pins are the same bytes as in 490.

## Checked independently

**On mac-studio** (also macOS with a Data volume), I imported the sealed file and called its own check functions. Its top level only reads the account database.
- **ROOT pin.** For this account, the file builds ROOT as home + `scratch/jmev-complex-native-20261109`. The sha256 of that text is `ROOT_SHA256`.
- **Volume.** `/sbin/mount` has exactly one `/System/Volumes/Data` line, and 508's parser reads it as `apfs`. `volume_problems` is empty.
- **Why 490's check had to go.** `os.path.ismount('/System/Volumes/Data')` is False here, so 490's check would refuse this volume.
- **Device and path.** `~/scratch` has the Data volume's `st_dev`, because `/Users` is a firmlink. `resolve()` returns the path unchanged. So the tree check can pass on a real ROOT.
- **Refusal order.** `refusals()` returns only the host refusal here. It comes first, before any mount read or hashing.
- **506's paths.** The two Cargo files of `mini-max-source506` hold 21 absolute paths in total, all under `ROOT/source`.

**The global reassignment works.** The pinned original is `819746fb`.
- Its only module-level statements that use ROOT, WORK, TARGET or REPO are `REPO = ROOT.parents[4]`, `WORK = ROOT / 'expanded-review-01'` and `TARGET = REPO / 'target'`. Those are the globals 508 sets again.
- No function default and no decorator captures them.
- ROOT is read only in `main()`, for the lock.

So 508's assignments reach every use at run time.

**On mac-mini-max, from root's records.**
- **513** (`root-mini-max-guard-stage513.json`) staged 508 at `ROOT/qualification`, at the sealed hash.
- **514** (`root-mini-max-preflight514.json`, `3768481e`) is an admitted native preflight, recorded 06:07:15Z–06:07:17Z:
  - sbh reports `mount_point` `/System/Volumes/Data` and `volume_role` Data, for the path ROOT/repo.
  - That ROOT's text also hashes to `ROOT_SHA256`, so the pin matches the real mac-mini-max path.
  - Policy is 20/8/4/80/20, work and target are both at 0 bytes, and there are no samples.
- **Not proven.** 514 does not name the wrapper that ran. If it ran through 508, then all of 508's refusals passed there.

## Notes (no change to 508)

- **N1. Cold-build size, one data point.** Receipt item 3 says no receipt in this guard family shows a cold build against the 4 GiB per-job cap.
  - In 268's resource receipt (`review-build268-terminal272/resource-receipt.json`), CSD's shared target held 2,322,567,168 bytes (2.16 GiB) at admission. 268 grew it by 41,885,696 bytes.
  - So a target that already held this crate set's release build was about half the cap.
  - That is a Linux figure, and that target may hold other files. Use it as a guide only. An exit 75 is still possible.
- **N2. The other launch risks are in 507, not 508.** These are the environment and the Cargo configs:
  - the environment: no `RUSTFLAGS`, `RUSTC` or rustc wrappers; toolchain-first `PATH`; the RCH shim;
  - Cargo configs found in ancestor folders or in `CARGO_HOME`.

  `mini-max-native-build507.py` (`8e5db00a`) checks both (`FORBIDDEN_ENV`, `config_problems`). I read only those lines; I did not review 507.
- **N3. The receipt's decisions are root's.** I found nothing against them:
  - the lane lock under ROOT, so guard01 jobs can still run alongside;
  - the absolute-path input manifest;
  - the outer time bound signalling the child's process group;
  - helper writes kept out of the guarded folders.

## Not checked

- any run on mac-mini-max beyond reading root's 513 and 514;
- CC2's `test508.py`;
- 507 as a whole;
- the stop paths, which are the original's unchanged code.

No audio heard or checked. This accepts no build, render or AV result, and it authorises no launch.
