# 499: mac-mini-old storage diagnosis (CC1)

CC1 (BronzeJaguar), 2026-11-09, probes at 05:47:22Z and 05:49:12Z. This was read only: two bounded ssh sessions running `ps`, `mount`, `hdiutil info`, `diskutil`, `iostat`, `lsof`, `df`, `top`, `system_profiler` and `log show`.
- No signal was sent to any process, and no mount or image was changed.
- Nothing was written on the mini, and no folder inside USBNVME16TB or EERCHExport was listed.
- The ExFAT volume was not queried directly; `df` and `diskutil` were not run against it.
- The transfer, the receiver and other jobs were left alone.

The raw outputs are kept outside Q, because they hold other projects' command lines and home paths:

| File | sha256 |
|---|---|
| `probe1.sh` | `1aa2294668d9e602a7297f9ed3123df86fffada2beb9157620c5ae5b421db65a` |
| `probe1.out` | `1501d90f691ca7c9377a77d5f6d97c1d0bb680f7f674e4411f7d9c44fd835c61` |
| `probe2.sh` | `e647389ae1ec4ca4838d44c64aa7d5e3c303eceb88b445498ab81b1a64ec2927` |
| `probe2.out` | `5426610d39236e53c9903d3ba77780bd3e72276e25dd571ae2f4c57b3c4906d9` |

## Answer

**EERCHExport is a file on the ExFAT drive.** It is an APFS volume inside a sparse disk image, and that image is a single file on the ExFAT USB drive. Every write into EERCHExport therefore goes through three layers:
1. APFS;
2. `diskimages-helper` (pid 39821, user space);
3. the user-space FSKit exFAT service (`UVFSService`, pid 77740), and then the USB drive.

**That exFAT service is stuck,** and the drive is saturated with small I/O from other jobs. During the sample the image itself moved nothing: 0 transfers per second.

**No faster local APFS location with 20 GiB or more exists on the mini today.** The internal SSD has 12.9 GB free. Every other APFS volume is also an image on the same ExFAT drive.

## Evidence

**What backs EERCHExport.** `hdiutil info` and `diskutil list` show:
- `/Volumes/EERCHExport` is `disk16s1` (APFS volume UUID `1248E0B7-1F40-4B91-9A94-ED2F7DBCE297`).
- Its container is `disk16`, and its physical store is `disk15s2` of the image `disk15`.
- That image is the sparse disk image `ee-rch-export-2.sparseimage`, in the folder `temp_agent_space/tmp` on `/Volumes/USBNVME16TB`. Its helper is pid 39821, and it is writable and 240 GiB in size.
- `df`: 249,637,500 KiB available and 1,680,744 KiB used. `lsof +f` found no open files on the volume at 05:49Z.

**The physical store.** USBNVME16TB is `disk6s2`, the only partition besides EFI on `disk6`.
- It is a 16.0 TB external USB disk, media name `RAlD0` as printed, behind a JMicron bridge (USB `152d:0586`) at a 10 Gb/s link.
- It is mounted `exfat, local, nodev, nosuid, noowners, noatime, fskit`. The `fskit` flag means macOS 26.5 serves it with the user-space FSKit exFAT module, through `UVFSService`.
- It is local, not a network file system.

**Why writes stall:**
- **The service is stuck.** `UVFSService` (pid 77740, up since Nov 5 17:23Z) was in state `Us`, uninterruptible wait, at 17.5% CPU with 110 minutes of CPU time. `top` lists it as `stuck`.
- **18 processes were in uninterruptible wait** at 05:49Z, against 13 that `top` listed as stuck at 05:47:38Z. These have a USBNVME16TB path in their command line:
  - two `find` walks over the drive's projects folder (up to depth 8), looking for `.rch-target*` folders;
  - an `lsd` long listing of a cargo-target `deps` folder on the drive;
  - a `cargo-rch-real test` run from a toolchain on the drive;
  - four `fdop` and `dopplerprint` jobs reading audio from the drive.

  These were also in uninterruptible wait, but their command lines name no path on the drive: two `rustc` example builds, `mem_log.sh`, `rch-wkr capabilities` and a Python process reading stdin. All of these belong to other projects and lanes, and I left them alone.
- **The drive is saturated with small I/O.** Over 10 s, `iostat` showed `disk6` at about 11,700 transfers per second of 4.0 KB, about 46 MB/s. At that size the drive is limited by operations per second, not by its 10 Gb/s link.
- **The image itself moved nothing.** Over the same samples, `disk15`, the EERCHExport image, showed 0 transfers.
- **The CPU had no idle time.** `top` showed 0.0% idle at 05:47:36Z, with `fdop-3c51eae` at 796% of the 10 CPUs. The exFAT service and the image helper both run in user space, so they compete for that CPU. This is likely a contributing factor; I did not measure it separately.
- **Logs:** `log show` over the last 15 minutes returned no entries from `UVFSService`, `diskimages-helper` or `fskit`. So there is no logged error, and none is ruled out.

I could not attribute I/O per process: `fs_usage` and `iotop` need root, and I used neither. The probes show contention and a stuck service on the shared exFAT path. They do not show what caused the stall.

**Receiver 40106 was gone by 05:47:22Z.** `ps -p 40106` returned only its header at 05:47:22Z and again at 05:49:12Z. No process named EERCHExport or the image, and nothing had files open on the volume. I did not see it exit, and I sent it no signal. The 82 files root counted stay on the volume; I did not list them.

## Other APFS locations

| Location | Backing | Free | Faster? |
|---|---|---|---|
| Internal `Macintosh HD - Data` (`disk3s5`, container `disk3`) | internal Apple Fabric SSD | **12.9 GB** container free (12,554,268 KiB available); no Time Machine local snapshots | Fast, but below the guard's 20 GiB admission and too small for a build |
| `/Volumes/ExtData` (`disk14s1`) | `ExtData.sparseimage` on the same ExFAT drive | 24.0 GB | No: same path, same service |
| Second EERCHExport (`disk8s1`, not mounted) | `ee-rch-export.sparseimage` on the same ExFAT drive | 116.7 GB unallocated; 140.6 GB used | No: same path. It also shares the volume name |
| `/nix` (`disk3s7`) | internal container | shares the 12.9 GB | No space |

No other physical disk is attached: `diskutil list` shows only `disk0` (internal, 500 GB) and `disk6` (the USB drive). So every APFS location with room for a build is on the stalled exFAT path.

## Practical options (decisions for root and the owner; none were taken)

1. **Quiet the ExFAT drive first.** Stop or wait out the jobs stuck on USBNVME16TB, then measure EERCHExport writes with one bounded test before any transfer. The stuck processes belong to other projects, and only their owners or the owner can decide this. This is the only option that makes the current image usable without touching hardware or partitions.
2. **Free internal space.** About 40 GB or more free on the internal SSD would allow a native APFS root there, avoiding the image and FSKit layers. Freeing it is the owner's decision; I deleted and moved nothing. With only 12.9 GB free, the guard's 20 GiB admission refuses it as it stands.
3. **Native APFS on the USB drive.** An APFS partition on the drive (809 GiB free by root's earlier probe) would remove the image and FSKit layers. It still shares the drive with the other jobs' I/O. It needs repartitioning or reformatting, which is a mount and data-safety decision for the owner, not something to try live.
4. **Another drive.** A separate APFS-formatted external drive is hardware.

Moving the image, or switching it to a sparse bundle on the same ExFAT drive, does not remove the stuck layer.

## Hazards seen (left alone)

- **ExtData is attached twice, both writable.** `ExtData.sparseimage` has two attachments: helper 1842 as `disk4`/`disk5`, not mounted, and helper 77920 as `disk13`/`disk14`, mounted at `/Volumes/ExtData`. Both carry the same container UUID `60489A5C-0DFA-498C-B383-16A1902A0939`. Two writable attachments of one image file can corrupt it. The owner should decide which to detach. I changed nothing.
- **Two volumes named EERCHExport.** The mounted one is the `-2` image, volume `1248E0B7…`. The older `ee-rch-export.sparseimage` is attached but not mounted (`disk8s1`, volume `64A4C843-EEB3-4B9C-A511-BC04C4C758DD`). If the mounts change, a different volume could appear at `/Volumes/EERCHExport`. Guard 490 checks the mount path and type, not the volume UUID. A successor could pin `1248E0B7…` if root wants that.
- **The host name reads `mac-mini-old`,** so guard 490's host check passes on this machine.

## Limits

- This is a snapshot of about two minutes, from two sessions. The I/O rates are averages over 2 s intervals.
- No per-process I/O attribution and no write test. I did not inspect the data on the volume.
- Not reviewed or verified by another agent.
- This makes no claim about any build, render, picture, master or AV acceptance; all of those remain open. There is no audio-review claim. Frederick remains the narration.
