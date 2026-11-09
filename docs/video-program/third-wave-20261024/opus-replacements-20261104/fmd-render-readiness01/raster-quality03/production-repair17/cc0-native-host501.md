# 501: fastest existing native host for the 268 source (CC0)

cc0, 2026-11-09, 05:54 UTC. Q is this folder. This was read-only and bounded:
- on mac-mini-max, one SSH probe of system facts and one read-only Python check (it hashed named files only and wrote nothing);
- on mac-studio, which is this host, local reads.

There was no broad scan, no build and no change on either host.

## Result

**Use mac-mini-max, on its internal APFS volume.** It is lightly loaded and has no stuck I/O. It already holds a byte-exact copy of the owner Mac's original source, and its cargo caches cover all of 268's lock.

Neither host has a ready 268 workspace, an exact native binary or a warm target for this build configuration. Either host needs a cold release build.

| | mac-mini-max | mac-studio (this host) |
|---|---|---|
| hardware | Mac16,11, 14 cores, 64 GiB | Mac17,15, 36 cores, 256 GiB |
| load (1 / 5 / 15 min) | 5.44 / 6.06 / 7.65 | 28.7 / 41.4 / 47.6 |
| processes in `U` | 0 of 1,163 | 0 of the 182 that `ps` could see from this session |
| swap used; memorystatus level | 3.1 of 4.0 GiB; 71 | 3.9 of 5.0 GiB; 58 |
| free space | internal APFS 101.6 GiB; `USB_NVME` (APFS) 8.5 GiB | internal APFS 149 GiB |
| `~/scratch/jmev-geometry-native-20261104-01` | **4,126 of 4,126 files equal** to the original (engine 4,057, repo-view 68, budget) | the original |
| `nightly-2026-08-31-aarch64` | yes | yes |
| 268 lock: registry crates | 231 of 231 cached | 231 of 231 cached |
| 268 lock: git sources at the locked revisions | all 6 repos (23 packages) | all 23 packages |
| `complex_analysis_visual` binaries | none in the repo target | none for this configuration; the earlier `geometry-kit01` target is gone |
| repo checkout as scene source | 28 of 70 equal: **do not use** | — |
| fork failures in this session | none seen | 2 at 05:49:23Z and 05:49:36Z ("resource temporarily unavailable"), then 2 silent failures by 05:50:01Z |

**Why not mac-studio.** It has more cores, but it is carrying a load of 29 to 48 and refused forks during this check. A cargo build spawns many processes.

## Staging on mac-mini-max (root's decision)

1. **Use a root on the internal volume**, not `USB_NVME`, which has only 8.5 GiB free.
2. **Re-map two files.** The 492 tree is mapped to `/Volumes/EERCHExport/jmev-complex-native-20261109/source`, which does not exist on mac-mini-max. Only two files carry that root: `engine-budget.toml` and the crate's `Cargo.toml`.
   - For a mac-mini-max root, those two need the same mapping to the new root.
   - The 492 manifest's back-mapping gives the hashes to check them against: `7277a6c0` and `7644838e`.
   - The other 4,195 files do not depend on the root.
3. **Copy, don't edit in place.** Copying the 49 MB tree with modes kept is the simple route. Editing mac-mini-max's original copy in place would also work, but it would alter that original.
4. **Build from the crate directory.** Run cargo from `workspace/explainer_corpus/complex_analysis_visual` with 268's flags, so that `.cargo/config.toml` applies.

## Not checked

- mac-mini-old;
- any build-speed estimate or benchmark;
- other lanes' plans for mac-mini-max, beyond its current load;
- mac-mini-max's `~/projects` as mounted over NFS on mac-studio, which I did not use.

No change was made on either host. No audio heard or checked.
